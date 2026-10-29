use crate::config::read_password_from_keychain;
use crate::model::{mode_to_rwx, FileEntry, Protocol};
use std::path::Path;
use std::process::Command;

/// Always false in production mode (all connections are live).
pub fn is_demo_host(_host: &str) -> bool {
    false
}

/// Parses and cleans up user-entered host input that might include protocol schemes (ftp://, sftp://)
/// and/or an embedded directory path (e.g., `ftp://ftp.vslg.cz/uc-vslg` or `host:port/path`).
pub fn sanitize_host_and_path(raw_host: &str) -> (Option<Protocol>, String, Option<u16>, String) {
    let mut s = raw_host.trim();
    let mut proto = None;
    let lower = s.to_lowercase();
    if lower.starts_with("ftp://") {
        proto = Some(Protocol::Ftp);
        s = &s[6..];
    } else if lower.starts_with("ftps://") {
        proto = Some(Protocol::FtpsImplicit);
        s = &s[7..];
    } else if lower.starts_with("sftp://") {
        proto = Some(Protocol::Sftp);
        s = &s[7..];
    } else if lower.starts_with("scp://") {
        proto = Some(Protocol::Scp);
        s = &s[6..];
    }

    let (host_and_port, path) = if let Some(idx) = s.find('/') {
        (&s[..idx], s[idx..].to_string())
    } else {
        (s, "/".to_string())
    };

    let (host, port) = if let Some((h, p_str)) = host_and_port.split_once(':') {
        let p = p_str.parse::<u16>().ok();
        (h.to_string(), p)
    } else {
        (host_and_port.to_string(), None)
    };

    (proto, host, port, path)
}

/// Lists a remote directory using the configured protocol (SFTP, SCP, FTP, FTPS).
/// Returns `Ok(Vec<FileEntry>)` when the remote server responds, or `Err(String)` on network error.
pub fn list_remote_directory(
    protocol: Protocol,
    host: &str,
    port: u16,
    username: &str,
    password: Option<&str>,
    remote_path: &str,
    show_hidden: bool,
) -> Result<Vec<FileEntry>, String> {
    let (detected_proto, clean_host, parsed_port, extra_path) = sanitize_host_and_path(host);
    let final_proto = detected_proto.unwrap_or(protocol);
    let final_host = if clean_host.is_empty() { host } else { &clean_host };
    let final_port = parsed_port.unwrap_or(port);
    let final_path = if (remote_path.is_empty() || remote_path == "/") && !extra_path.is_empty() && extra_path != "/" {
        &extra_path
    } else {
        remote_path
    };

    match final_proto {
        Protocol::Sftp | Protocol::Scp => {
            list_via_ssh(final_host, final_port, username, final_path, show_hidden)
        }
        Protocol::Ftp | Protocol::FtpsExplicit | Protocol::FtpsImplicit => {
            list_via_ftp_curl(
                final_proto,
                final_host,
                final_port,
                username,
                password,
                final_path,
                show_hidden,
            )
        }
    }
}

fn list_via_ssh(
    host: &str,
    port: u16,
    username: &str,
    remote_path: &str,
    show_hidden: bool,
) -> Result<Vec<FileEntry>, String> {
    let target = if username.is_empty() {
        host.to_string()
    } else {
        format!("{}@{}", username, host)
    };
    let escaped_path = remote_path.replace('\'', "'\\''");
    let remote_cmd = format!(
        "LC_ALL=C.UTF-8 ls -la '{}' 2>/dev/null || LC_ALL=en_US.UTF-8 ls -la '{}' 2>/dev/null || LC_ALL=C ls -la '{}'",
        escaped_path, escaped_path, escaped_path
    );

    let output = Command::new("/usr/bin/ssh")
        .args([
            "-o",
            "BatchMode=yes",
            "-o",
            "ConnectTimeout=8",
            "-o",
            "StrictHostKeyChecking=accept-new",
            "-p",
            &port.to_string(),
            &target,
            &remote_cmd,
        ])
        .output()
        .map_err(|e| format!("Failed to spawn ssh: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            "SSH connection failed (verify host, port, or SSH key)".to_string()
        } else {
            stderr
        });
    }

    let stdout = decode_remote_listing(&output.stdout);
    Ok(parse_unix_ls_output(&stdout, remote_path, show_hidden))
}

/// Maps a Windows-1250 byte (> 0x7F) to its Unicode char.
pub fn decode_cp1250_byte(b: u8) -> char {
    match b {
        0x80 => '€',
        0x82 => '‚',
        0x84 => '„',
        0x85 => '…',
        0x86 => '†',
        0x87 => '‡',
        0x89 => '‰',
        0x8A => 'Š',
        0x8B => '‹',
        0x8C => 'Ś',
        0x8D => 'Ť',
        0x8E => 'Ž',
        0x8F => 'Ź',
        0x91 => '‘',
        0x92 => '’',
        0x93 => '“',
        0x94 => '”',
        0x95 => '•',
        0x96 => '–',
        0x97 => '—',
        0x99 => '™',
        0x9A => 'š',
        0x9B => '›',
        0x9C => 'ś',
        0x9D => 'ť',
        0x9E => 'ž',
        0x9F => 'ź',
        0xA0 => '\u{00A0}',
        0xA1 => 'ˇ',
        0xA2 => '˘',
        0xA3 => 'Ł',
        0xA4 => '¤',
        0xA5 => 'Ą',
        0xA6 => '¦',
        0xA7 => '§',
        0xA8 => '¨',
        0xA9 => '©',
        0xAA => 'Ş',
        0xAB => '«',
        0xAC => '¬',
        0xAD => '\u{00AD}',
        0xAE => '®',
        0xAF => 'Ż',
        0xB0 => '°',
        0xB1 => '±',
        0xB2 => '˛',
        0xB3 => 'ł',
        0xB4 => '´',
        0xB5 => 'µ',
        0xB6 => '¶',
        0xB7 => '·',
        0xB8 => '¸',
        0xB9 => 'ą',
        0xBA => 'ş',
        0xBB => '»',
        0xBC => 'Ľ',
        0xBD => '˝',
        0xBE => 'ľ',
        0xBF => 'ż',
        0xC0 => 'Ŕ',
        0xC1 => 'Á',
        0xC2 => 'Â',
        0xC3 => 'Ă',
        0xC4 => 'Ä',
        0xC5 => 'Ĺ',
        0xC6 => 'Ć',
        0xC7 => 'Ç',
        0xC8 => 'Č',
        0xC9 => 'É',
        0xCA => 'Ę',
        0xCB => 'Ë',
        0xCC => 'Ě',
        0xCD => 'Í',
        0xCE => 'Î',
        0xCF => 'Ď',
        0xD0 => 'Đ',
        0xD1 => 'Ń',
        0xD2 => 'Ň',
        0xD3 => 'Ó',
        0xD4 => 'Ô',
        0xD5 => 'Ő',
        0xD6 => 'Ö',
        0xD7 => '×',
        0xD8 => 'Ř',
        0xD9 => 'Ů',
        0xDA => 'Ú',
        0xDB => 'Ű',
        0xDC => 'Ü',
        0xDD => 'Ý',
        0xDE => 'Ţ',
        0xDF => 'ß',
        0xE0 => 'ŕ',
        0xE1 => 'á',
        0xE2 => 'â',
        0xE3 => 'ă',
        0xE4 => 'ä',
        0xE5 => 'ĺ',
        0xE6 => 'ć',
        0xE7 => 'ç',
        0xE8 => 'č',
        0xE9 => 'é',
        0xEA => 'ę',
        0xEB => 'ë',
        0xEC => 'ě',
        0xED => 'í',
        0xEE => 'î',
        0xEF => 'ď',
        0xF0 => 'đ',
        0xF1 => 'ń',
        0xF2 => 'ň',
        0xF3 => 'ó',
        0xF4 => 'ô',
        0xF5 => 'ő',
        0xF6 => 'ö',
        0xF7 => '÷',
        0xF8 => 'ř',
        0xF9 => 'ů',
        0xFA => 'ú',
        0xFB => 'ű',
        0xFC => 'ü',
        0xFD => 'ý',
        0xFE => 'ţ',
        0xFF => '˙',
        _ => b as char,
    }
}

/// Maps an ISO-8859-2 byte (> 0x7F) to its Unicode char.
pub fn decode_iso8859_2_byte(b: u8) -> char {
    match b {
        0xA1 => 'Ą',
        0xA2 => '˘',
        0xA3 => 'Ł',
        0xA4 => '¤',
        0xA5 => 'Ľ',
        0xA6 => 'Ś',
        0xA7 => '§',
        0xA8 => '¨',
        0xA9 => 'Š',
        0xAA => 'Ş',
        0xAB => 'Ť',
        0xAC => 'Ź',
        0xAD => '\u{00AD}',
        0xAE => 'Ž',
        0xAF => 'Ż',
        0xB0 => '°',
        0xB1 => 'ą',
        0xB2 => '˛',
        0xB3 => 'ł',
        0xB4 => '´',
        0xB5 => 'ľ',
        0xB6 => 'ś',
        0xB7 => 'ˇ',
        0xB8 => '¸',
        0xB9 => 'š',
        0xBA => 'ş',
        0xBB => 'ť',
        0xBC => 'ź',
        0xBD => '˝',
        0xBE => 'ž',
        0xBF => 'ż',
        0xC0 => 'Ŕ',
        0xC1 => 'Á',
        0xC2 => 'Â',
        0xC3 => 'Ă',
        0xC4 => 'Ä',
        0xC5 => 'Ĺ',
        0xC6 => 'Ć',
        0xC7 => 'Ç',
        0xC8 => 'Č',
        0xC9 => 'É',
        0xCA => 'Ę',
        0xCB => 'Ë',
        0xCC => 'Ě',
        0xCD => 'Í',
        0xCE => 'Î',
        0xCF => 'Ď',
        0xD0 => 'Đ',
        0xD1 => 'Ń',
        0xD2 => 'Ň',
        0xD3 => 'Ó',
        0xD4 => 'Ô',
        0xD5 => 'Ő',
        0xD6 => 'Ö',
        0xD7 => '×',
        0xD8 => 'Ř',
        0xD9 => 'Ů',
        0xDA => 'Ú',
        0xDB => 'Ű',
        0xDC => 'Ü',
        0xDD => 'Ý',
        0xDE => 'Ţ',
        0xDF => 'ß',
        0xE0 => 'ŕ',
        0xE1 => 'á',
        0xE2 => 'â',
        0xE3 => 'ă',
        0xE4 => 'ä',
        0xE5 => 'ĺ',
        0xE6 => 'ć',
        0xE7 => 'ç',
        0xE8 => 'č',
        0xE9 => 'é',
        0xEA => 'ę',
        0xEB => 'ë',
        0xEC => 'ě',
        0xED => 'í',
        0xEE => 'î',
        0xEF => 'ď',
        0xF0 => 'đ',
        0xF1 => 'ń',
        0xF2 => 'ň',
        0xF3 => 'ó',
        0xF4 => 'ô',
        0xF5 => 'ő',
        0xF6 => 'ö',
        0xF7 => '÷',
        0xF8 => 'ř',
        0xF9 => 'ů',
        0xFA => 'ú',
        0xFB => 'ű',
        0xFC => 'ü',
        0xFD => 'ý',
        0xFE => 'ţ',
        0xFF => '˙',
        _ => b as char,
    }
}

/// Intelligently decodes raw bytes from FTP / SSH directory listings.
/// Supports native UTF-8 and falls back to Windows-1250 (CP1250) or ISO-8859-2 (Latin-2)
/// so Czech and Central European characters (ě, š, č, ř, ž, ý, á, í, é, ů, ú, ď, ť, ň)
/// are always correctly decoded instead of turning into replacement characters ().
pub fn decode_remote_listing(bytes: &[u8]) -> String {
    // 1. If valid UTF-8, use directly
    if let Ok(s) = std::str::from_utf8(bytes) {
        return s.to_string();
    }

    // 2. Not valid UTF-8: detect between Windows-1250 and ISO-8859-2
    let mut has_cp1250_specific = false;
    let mut iso_specific_count = 0;

    for &b in bytes {
        if matches!(b, 0x8A | 0x8D | 0x8E | 0x9A | 0x9D | 0x9E) {
            // Š, Ť, Ž, š, ť, ž in Windows-1250
            has_cp1250_specific = true;
            break;
        } else if matches!(b, 0xA9 | 0xAB | 0xAE | 0xB9 | 0xBB | 0xBE) {
            // Š, Ť, Ž, š, ť, ž in ISO-8859-2
            iso_specific_count += 1;
        }
    }

    let use_iso = !has_cp1250_specific && iso_specific_count > 0;

    let mut decoded = String::with_capacity(bytes.len() * 3 / 2);
    for &b in bytes {
        if b < 0x80 {
            decoded.push(b as char);
        } else if use_iso {
            decoded.push(decode_iso8859_2_byte(b));
        } else {
            decoded.push(decode_cp1250_byte(b));
        }
    }
    decoded
}

pub fn percent_encode_path(path: &str) -> String {
    let mut encoded = String::with_capacity(path.len() * 2);
    for byte in path.as_bytes() {
        match *byte {
            b'/' => encoded.push('/'),
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(*byte as char);
            }
            b => {
                encoded.push_str(&format!("%{:02X}", b));
            }
        }
    }
    encoded
}

/// Encodes a single Unicode character to its single-byte Windows-1250 (CP1250) representation.
pub fn encode_cp1250_char(c: char) -> Option<u8> {
    if (c as u32) < 0x80 {
        return Some(c as u8);
    }
    for b in 0x80..=0xFF {
        if decode_cp1250_byte(b) == c {
            return Some(b);
        }
    }
    None
}

/// Percent-encodes path using Windows-1250 byte mapping for non-ASCII characters.
/// Necessary for legacy Windows IIS FTP servers that store files in ANSI/CP1250 rather than UTF-8.
pub fn percent_encode_path_cp1250(path: &str) -> String {
    let mut encoded = String::with_capacity(path.len() * 2);
    for c in path.chars() {
        match c {
            '/' => encoded.push('/'),
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => {
                encoded.push(c);
            }
            _ => {
                if let Some(b) = encode_cp1250_char(c) {
                    encoded.push_str(&format!("%{:02X}", b));
                } else {
                    for byte in c.to_string().as_bytes() {
                        encoded.push_str(&format!("%{:02X}", byte));
                    }
                }
            }
        }
    }
    encoded
}

fn list_via_ftp_curl(
    protocol: Protocol,
    host: &str,
    port: u16,
    username: &str,
    password: Option<&str>,
    remote_path: &str,
    show_hidden: bool,
) -> Result<Vec<FileEntry>, String> {
    let (detected_proto, clean_host, parsed_port, extra_path) = sanitize_host_and_path(host);
    let final_proto = detected_proto.unwrap_or(protocol);
    let final_host = if clean_host.is_empty() { host } else { &clean_host };
    let final_port = parsed_port.unwrap_or(port);
    let target_path = if (remote_path.is_empty() || remote_path == "/") && !extra_path.is_empty() && extra_path != "/" {
        extra_path
    } else {
        remote_path.to_string()
    };

    let scheme = match final_proto {
        Protocol::FtpsImplicit => "ftps",
        _ => "ftp",
    };
    let clean_path = if target_path.is_empty() || target_path == "/" {
        "/".to_string()
    } else if target_path.ends_with('/') {
        target_path.clone()
    } else {
        format!("{}/", target_path)
    };
    let encoded_path = percent_encode_path(&clean_path);
    let url = format!("{}://{}:{}{}", scheme, final_host, final_port, encoded_path);

    let pwd = password
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| read_password_from_keychain(host, username))
        .unwrap_or_default();

    let mut cmd = Command::new("/usr/bin/curl");
    cmd.args([
        "-sS",
        "--connect-timeout",
        "10",
        "--max-time",
        "25",
        "--disable-epsv",
        "-Q",
        "*OPTS UTF8 ON",
    ]);

    if protocol == Protocol::FtpsExplicit {
        cmd.args(["--ssl", "--insecure"]);
    } else if protocol == Protocol::FtpsImplicit {
        cmd.args(["--ssl-reqd", "--insecure"]);
    }

    if !username.is_empty() {
        if !pwd.is_empty() {
            cmd.args(["--user", &format!("{}:{}", username, pwd)]);
        } else {
            cmd.args(["--user", username]);
        }
    }

    cmd.arg(&url);

    let output = cmd
        .output()
        .map_err(|e| format!("Failed to spawn curl: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let code = output.status.code().unwrap_or(-1);

        // Check if server returned 550 / no files found (empty directory)
        if code == 19
            || code == 78
            || stderr.contains("550")
            || stderr.to_lowercase().contains("no files")
            || stderr.to_lowercase().contains("not found")
        {
            return Ok(parse_unix_ls_output("", &target_path, show_hidden));
        }

        let err_msg = if !stderr.is_empty() {
            stderr
        } else {
            match code {
                3 => format!("Invalid characters in remote URL '{}'", url),
                6 => format!("Could not resolve host '{}'", host),
                7 => format!("Failed to connect to {}:{} (connection refused or host unreachable)", host, port),
                28 => format!("Connection to {}:{} timed out (check port and firewall)", host, port),
                67 => "FTP login failed: incorrect username or password".to_string(),
                _ => format!("FTP connection error (curl exit code {})", code),
            }
        };
        return Err(err_msg);
    }

    let stdout = decode_remote_listing(&output.stdout);
    Ok(parse_unix_ls_output(&stdout, &target_path, show_hidden))
}

/// Upload a local file or directory to the remote server with progress tracking and real-time server command logging.
pub fn upload_to_remote_with_progress_and_log<F, L>(
    protocol: Protocol,
    host: &str,
    port: u16,
    username: &str,
    password: Option<&str>,
    local_path: &Path,
    remote_target_path: &str,
    mut on_progress: F,
    mut on_log: L,
) -> Result<(), String>
where
    F: FnMut(u64, u64) + Send,
    L: FnMut(crate::model::ServerLogKind, String) + Send,
{
    upload_to_remote_impl(
        protocol,
        host,
        port,
        username,
        password,
        local_path,
        remote_target_path,
        &mut on_progress,
        &mut on_log,
    )
}

fn upload_to_remote_impl(
    protocol: Protocol,
    host: &str,
    port: u16,
    username: &str,
    password: Option<&str>,
    local_path: &Path,
    remote_target_path: &str,
    on_progress: &mut (dyn FnMut(u64, u64) + Send),
    on_log: &mut (dyn FnMut(crate::model::ServerLogKind, String) + Send),
) -> Result<(), String> {
    let (detected_proto, clean_host, parsed_port, _) = sanitize_host_and_path(host);
    let final_proto = detected_proto.unwrap_or(protocol);
    let final_host = if clean_host.is_empty() { host } else { &clean_host };
    let final_port = parsed_port.unwrap_or(port);

    if local_path.is_dir() {
        let _ = remote_mkdir(final_proto, final_host, final_port, username, password, remote_target_path);
        if let Ok(entries) = std::fs::read_dir(local_path) {
            for entry in entries.flatten() {
                let child_local = entry.path();
                let child_name = entry.file_name().to_string_lossy().to_string();
                let child_remote = format!("{}/{}", remote_target_path.trim_end_matches('/'), child_name);
                let _ = upload_to_remote_impl(
                    final_proto,
                    final_host,
                    final_port,
                    username,
                    password,
                    &child_local,
                    &child_remote,
                    on_progress,
                    on_log,
                );
            }
        }
        return Ok(());
    }

    let file_size = std::fs::metadata(local_path).map(|m| m.len()).unwrap_or(0);

    match final_proto {
        Protocol::Sftp | Protocol::Scp => {
            let target = if username.is_empty() {
                format!("{}:{}", final_host, remote_target_path)
            } else {
                format!("{}@{}:{}", username, final_host, remote_target_path)
            };
            on_log(crate::model::ServerLogKind::Command, format!("scp -P {} {} -> {}", final_port, local_path.display(), target));
            let mut child = Command::new("/usr/bin/scp")
                .args([
                    "-r",
                    "-o",
                    "BatchMode=yes",
                    "-o",
                    "ConnectTimeout=15",
                    "-P",
                    &final_port.to_string(),
                    &local_path.to_string_lossy(),
                    &target,
                ])
                .spawn()
                .map_err(|e| e.to_string())?;

            loop {
                on_progress(file_size / 2, file_size);
                match child.try_wait() {
                    Ok(Some(status)) => {
                        if status.success() {
                            on_progress(file_size, file_size);
                            on_log(crate::model::ServerLogKind::Response, format!("226 Upload dokončen: {}", local_path.file_name().unwrap_or_default().to_string_lossy()));
                            return Ok(());
                        } else {
                            return Err(format!("SCP upload exited with status {}", status));
                        }
                    }
                    Ok(None) => {
                        std::thread::sleep(std::time::Duration::from_millis(80));
                    }
                    Err(e) => return Err(e.to_string()),
                }
            }
        }
        Protocol::Ftp | Protocol::FtpsExplicit | Protocol::FtpsImplicit => {
            let scheme = if final_proto == Protocol::FtpsImplicit {
                "ftps"
            } else {
                "ftp"
            };
            let clean_path = if remote_target_path.starts_with('/') {
                remote_target_path.to_string()
            } else {
                format!("/{}", remote_target_path)
            };
            let encoded_path = percent_encode_path(&clean_path);
            let url = format!("{}://{}:{}{}", scheme, final_host, final_port, encoded_path);
            let pwd = password
                .map(|s| s.to_string())
                .filter(|s| !s.is_empty())
                .or_else(|| read_password_from_keychain(final_host, username))
                .unwrap_or_default();

            let mut cmd = Command::new("/usr/bin/curl");
            cmd.args([
                "-v",
                "--connect-timeout",
                "15",
                "--speed-limit",
                "1",
                "--speed-time",
                "30",
                "--disable-epsv",
                "--ftp-create-dirs",
                "-Q",
                "*OPTS UTF8 ON",
            ]);
            if final_proto == Protocol::FtpsExplicit {
                cmd.args(["--ssl", "--insecure"]);
            } else if final_proto == Protocol::FtpsImplicit {
                cmd.args(["--ssl-reqd", "--insecure"]);
            }
            if !username.is_empty() {
                if !pwd.is_empty() {
                    cmd.args(["--user", &format!("{}:{}", username, pwd)]);
                } else {
                    cmd.args(["--user", username]);
                }
            }
            cmd.args(["-T", &local_path.to_string_lossy(), &url]);
            cmd.stderr(std::process::Stdio::piped());

            let mut child = cmd.spawn().map_err(|e| e.to_string())?;
            let stderr = child.stderr.take();

            let (tx_log, rx_log) = std::sync::mpsc::channel::<(crate::model::ServerLogKind, String)>();
            let reader_thread = std::thread::spawn(move || {
                if let Some(stderr) = stderr {
                    use std::io::BufRead;
                    let reader = std::io::BufReader::new(stderr);
                    for line in reader.lines().flatten() {
                        let trimmed = line.trim();
                        if trimmed.is_empty() { continue; }
                        let (kind, msg) = if trimmed.starts_with("> PASS ") || trimmed == "> PASS" {
                            (crate::model::ServerLogKind::Command, "> PASS *****".to_string())
                        } else if trimmed.starts_with("> ") {
                            (crate::model::ServerLogKind::Command, trimmed.to_string())
                        } else if trimmed.starts_with("< ") {
                            let k = if trimmed.starts_with("< 5") || trimmed.starts_with("< 4") {
                                crate::model::ServerLogKind::Error
                            } else {
                                crate::model::ServerLogKind::Response
                            };
                            (k, trimmed.to_string())
                        } else {
                            (crate::model::ServerLogKind::Status, trimmed.to_string())
                        };
                        let _ = tx_log.send((kind, msg));
                    }
                }
            });

            let mut last_server_error: Option<String> = None;
            loop {
                while let Ok((kind, msg)) = rx_log.try_recv() {
                    if kind == crate::model::ServerLogKind::Error || msg.starts_with("< 5") || msg.starts_with("< 4") {
                        last_server_error = Some(msg.clone());
                    }
                    on_log(kind, msg);
                }

                match child.try_wait() {
                    Ok(Some(status)) => {
                        let _ = reader_thread.join();
                        while let Ok((kind, msg)) = rx_log.try_recv() {
                            if kind == crate::model::ServerLogKind::Error || msg.starts_with("< 5") || msg.starts_with("< 4") {
                                last_server_error = Some(msg.clone());
                            }
                            on_log(kind, msg);
                        }
                        if status.success() {
                            on_progress(file_size, file_size);
                            return Ok(());
                        } else {
                            let detail = last_server_error.as_deref().map(|s| format!(" ({})", s)).unwrap_or_default();
                            return Err(format!("Curl upload selhal s kódem {}{}", status, detail));
                        }
                    }
                    Ok(None) => {
                        std::thread::sleep(std::time::Duration::from_millis(60));
                    }
                    Err(e) => return Err(e.to_string()),
                }
            }
        }
    }
}

/// Upload a local file or directory to the remote server with progress tracking.
#[allow(dead_code)]
pub fn upload_to_remote_with_progress<F>(
    protocol: Protocol,
    host: &str,
    port: u16,
    username: &str,
    password: Option<&str>,
    local_path: &Path,
    remote_target_path: &str,
    on_progress: F,
) -> Result<(), String>
where
    F: FnMut(u64, u64) + Send,
{
    upload_to_remote_with_progress_and_log(
        protocol,
        host,
        port,
        username,
        password,
        local_path,
        remote_target_path,
        on_progress,
        |_, _| {},
    )
}

/// Download a remote file to local filesystem with progress tracking and real-time server command logging.
pub fn download_from_remote_with_progress_and_log<F, L>(
    protocol: Protocol,
    host: &str,
    port: u16,
    username: &str,
    password: Option<&str>,
    remote_source_path: &str,
    local_target_path: &Path,
    expected_size: u64,
    mut on_progress: F,
    mut on_log: L,
) -> Result<(), String>
where
    F: FnMut(u64, u64) + Send,
    L: FnMut(crate::model::ServerLogKind, String) + Send,
{
    if let Some(parent) = local_target_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let (detected_proto, clean_host, parsed_port, _) = sanitize_host_and_path(host);
    let final_proto = detected_proto.unwrap_or(protocol);
    let final_host = if clean_host.is_empty() { host } else { &clean_host };
    let final_port = parsed_port.unwrap_or(port);

    match final_proto {
        Protocol::Sftp | Protocol::Scp => {
            let source = if username.is_empty() {
                format!("{}:{}", final_host, remote_source_path)
            } else {
                format!("{}@{}:{}", username, final_host, remote_source_path)
            };
            on_log(crate::model::ServerLogKind::Command, format!("scp -P {} {} -> {}", final_port, source, local_target_path.display()));
            let mut child = Command::new("/usr/bin/scp")
                .args([
                    "-r",
                    "-o",
                    "BatchMode=yes",
                    "-o",
                    "ConnectTimeout=15",
                    "-P",
                    &final_port.to_string(),
                    &source,
                    &local_target_path.to_string_lossy(),
                ])
                .spawn()
                .map_err(|e| e.to_string())?;

            loop {
                let cur = std::fs::metadata(local_target_path).map(|m| m.len()).unwrap_or(0);
                if cur > 0 || expected_size > 0 {
                    on_progress(cur, expected_size);
                }
                match child.try_wait() {
                    Ok(Some(status)) => {
                        if status.success() {
                            let len = std::fs::metadata(local_target_path).map(|m| m.len()).unwrap_or(0);
                            let total = if expected_size > 0 { expected_size.max(len) } else { len };
                            on_progress(total, total);
                            on_log(crate::model::ServerLogKind::Response, format!("226 SCP přenos dokončen: {}", local_target_path.file_name().unwrap_or_default().to_string_lossy()));
                            return Ok(());
                        } else {
                            return Err(format!("SCP download exited with status {}", status));
                        }
                    }
                    Ok(None) => {
                        std::thread::sleep(std::time::Duration::from_millis(60));
                    }
                    Err(e) => return Err(e.to_string()),
                }
            }
        }
        Protocol::Ftp | Protocol::FtpsExplicit | Protocol::FtpsImplicit => {
            let scheme = if final_proto == Protocol::FtpsImplicit {
                "ftps"
            } else {
                "ftp"
            };
            let clean_path = if remote_source_path.starts_with('/') {
                remote_source_path.to_string()
            } else {
                format!("/{}", remote_source_path)
            };
            let pwd = password
                .map(|s| s.to_string())
                .filter(|s| !s.is_empty())
                .or_else(|| read_password_from_keychain(final_host, username))
                .unwrap_or_default();

            let run_curl_download = |target_url: &str, on_prog: &mut F, on_lg: &mut L| -> Result<(), (std::process::ExitStatus, Option<String>)> {
                let mut cmd = Command::new("/usr/bin/curl");
                cmd.args([
                    "-v",
                    "--connect-timeout",
                    "15",
                    "--speed-limit",
                    "1",
                    "--speed-time",
                    "30",
                    "--disable-epsv",
                    "-Q",
                    "*OPTS UTF8 ON",
                ]);
                if final_proto == Protocol::FtpsExplicit {
                    cmd.args(["--ssl", "--insecure"]);
                } else if final_proto == Protocol::FtpsImplicit {
                    cmd.args(["--ssl-reqd", "--insecure"]);
                }
                if !username.is_empty() {
                    if !pwd.is_empty() {
                        cmd.args(["--user", &format!("{}:{}", username, pwd)]);
                    } else {
                        cmd.args(["--user", username]);
                    }
                }
                cmd.args(["-o", &local_target_path.to_string_lossy(), target_url]);
                cmd.stderr(std::process::Stdio::piped());

                let mut child = cmd.spawn().map_err(|e| {
                    on_lg(crate::model::ServerLogKind::Error, format!("Nepodařilo se spustit curl: {e}"));
                    // Synthesize dummy exit failure
                    (std::process::ExitStatus::default(), Some(e.to_string()))
                })?;
                let stderr = child.stderr.take();

                let (tx_log, rx_log) = std::sync::mpsc::channel::<(crate::model::ServerLogKind, String, u64)>();
                let reader_thread = std::thread::spawn(move || {
                    if let Some(stderr) = stderr {
                        use std::io::BufRead;
                        let reader = std::io::BufReader::new(stderr);
                        for line in reader.lines().flatten() {
                            let trimmed = line.trim();
                            if trimmed.is_empty() { continue; }

                            let mut detected_size = 0u64;
                            if trimmed.starts_with("< 150") || trimmed.starts_with("< 213") {
                                if let Some(start_paren) = trimmed.rfind('(') {
                                    if let Some(end_bytes) = trimmed[start_paren + 1..].find(" byte") {
                                        if let Ok(num) = trimmed[start_paren + 1..start_paren + 1 + end_bytes].trim().parse::<u64>() {
                                            detected_size = num;
                                        }
                                    }
                                } else if trimmed.starts_with("< 213") {
                                    if let Some(size_str) = trimmed.split_whitespace().nth(1) {
                                        if let Ok(num) = size_str.parse::<u64>() {
                                            detected_size = num;
                                        }
                                    }
                                }
                            }

                            let (kind, msg) = if trimmed.starts_with("> PASS ") || trimmed == "> PASS" {
                                (crate::model::ServerLogKind::Command, "> PASS *****".to_string())
                            } else if trimmed.starts_with("> ") {
                                (crate::model::ServerLogKind::Command, trimmed.to_string())
                            } else if trimmed.starts_with("< ") {
                                let k = if trimmed.starts_with("< 5") || trimmed.starts_with("< 4") {
                                    crate::model::ServerLogKind::Error
                                } else {
                                    crate::model::ServerLogKind::Response
                                };
                                (k, trimmed.to_string())
                            } else {
                                (crate::model::ServerLogKind::Status, trimmed.to_string())
                            };

                            let _ = tx_log.send((kind, msg, detected_size));
                        }
                    }
                });

                let mut detected_total = expected_size;
                let mut last_server_error: Option<String> = None;
                loop {
                    while let Ok((kind, msg, sz)) = rx_log.try_recv() {
                        if sz > 0 {
                            detected_total = sz;
                        }
                        if kind == crate::model::ServerLogKind::Error || msg.starts_with("< 5") || msg.starts_with("< 4") {
                            last_server_error = Some(msg.clone());
                        }
                        on_lg(kind, msg);
                    }

                    let cur = std::fs::metadata(local_target_path).map(|m| m.len()).unwrap_or(0);
                    if cur > 0 || detected_total > 0 {
                        on_prog(cur, detected_total);
                    }

                    match child.try_wait() {
                        Ok(Some(status)) => {
                            let _ = reader_thread.join();
                            while let Ok((kind, msg, sz)) = rx_log.try_recv() {
                                if sz > 0 {
                                    detected_total = sz;
                                }
                                if kind == crate::model::ServerLogKind::Error || msg.starts_with("< 5") || msg.starts_with("< 4") {
                                    last_server_error = Some(msg.clone());
                                }
                                on_lg(kind, msg);
                            }
                            if status.success() {
                                let final_len = std::fs::metadata(local_target_path).map(|m| m.len()).unwrap_or(0);
                                let total = if detected_total > 0 { detected_total.max(final_len) } else { final_len };
                                on_prog(total, total);
                                return Ok(());
                            } else {
                                return Err((status, last_server_error));
                            }
                        }
                        Ok(None) => {
                            std::thread::sleep(std::time::Duration::from_millis(60));
                        }
                        Err(e) => {
                            return Err((std::process::ExitStatus::default(), Some(e.to_string())));
                        }
                    }
                }
            };

            let encoded_path = percent_encode_path(&clean_path);
            let url = format!("{}://{}:{}{}", scheme, final_host, final_port, encoded_path);

            match run_curl_download(&url, &mut on_progress, &mut on_log) {
                Ok(()) => Ok(()),
                Err((status, last_server_error)) => {
                    let code = status.code().unwrap_or(-1);
                    // If file not found (code 78) and path contains non-ASCII characters,
                    // automatically retry using Windows-1250 (CP1250) byte encoding.
                    if code == 78 && clean_path.chars().any(|c| (c as u32) >= 0x80) {
                        on_log(
                            crate::model::ServerLogKind::Status,
                            "Soubor nebyl nalezen v UTF-8 kódování, automaticky opakuji stažení s kódováním Windows-1250 (CP1250)...".to_string(),
                        );
                        let cp1250_path = percent_encode_path_cp1250(&clean_path);
                        let url_cp1250 = format!("{}://{}:{}{}", scheme, final_host, final_port, cp1250_path);
                        if let Ok(()) = run_curl_download(&url_cp1250, &mut on_progress, &mut on_log) {
                            return Ok(());
                        }
                    }

                    let detail = if let Some(err) = &last_server_error {
                        format!(" ({})", err)
                    } else if code == 78 {
                        " (soubor nebyl na serveru nalezen nebo server odmítl přístup k názvu)".to_string()
                    } else {
                        String::new()
                    };
                    Err(format!("Curl download selhal s kódem {}{}", status, detail))
                }
            }
        }
    }
}

/// Download a remote file to the local filesystem with progress.
#[allow(dead_code)]
pub fn download_from_remote_with_progress<F>(
    protocol: Protocol,
    host: &str,
    port: u16,
    username: &str,
    password: Option<&str>,
    remote_source_path: &str,
    local_target_path: &Path,
    on_progress: F,
) -> Result<(), String>
where
    F: FnMut(u64, u64) + Send,
{
    download_from_remote_with_progress_and_log(
        protocol,
        host,
        port,
        username,
        password,
        remote_source_path,
        local_target_path,
        0,
        on_progress,
        |_, _| {},
    )
}

/// Download a remote file to the local filesystem.
#[allow(dead_code)]
pub fn download_from_remote(
    protocol: Protocol,
    host: &str,
    port: u16,
    username: &str,
    password: Option<&str>,
    remote_source_path: &str,
    local_target_path: &Path,
) -> Result<(), String> {
    download_from_remote_with_progress(protocol, host, port, username, password, remote_source_path, local_target_path, |_, _| {})
}

/// Recursively downloads a remote directory to local filesystem with progress.
pub fn download_remote_dir_with_progress(
    protocol: Protocol,
    host: &str,
    port: u16,
    username: &str,
    password: Option<&str>,
    remote_source_dir: &str,
    local_target_dir: &Path,
    on_progress: &mut (dyn FnMut(u64, u64) + Send),
) -> Result<(), String> {
    let _ = std::fs::create_dir_all(local_target_dir);
    let children = list_remote_directory(protocol, host, port, username, password, remote_source_dir, true)?;
    for child in children {
        if child.name == ".." || child.name == "." {
            continue;
        }
        let child_local = local_target_dir.join(&child.name);
        if child.is_dir {
            let _ = download_remote_dir_with_progress(protocol, host, port, username, password, &child.path, &child_local, &mut *on_progress);
        } else {
            let _ = download_from_remote_with_progress(protocol, host, port, username, password, &child.path, &child_local, &mut *on_progress);
        }
    }
    Ok(())
}

/// Recursively downloads a remote directory to local filesystem.
#[allow(dead_code)]
pub fn download_remote_dir(
    protocol: Protocol,
    host: &str,
    port: u16,
    username: &str,
    password: Option<&str>,
    remote_source_dir: &str,
    local_target_dir: &Path,
) -> Result<(), String> {
    let mut dummy = |_, _| {};
    download_remote_dir_with_progress(protocol, host, port, username, password, remote_source_dir, local_target_dir, &mut dummy)
}

fn month_to_num(m: &str) -> u32 {
    match m.to_lowercase().as_str() {
        "jan" => 1,
        "feb" => 2,
        "mar" => 3,
        "apr" => 4,
        "may" => 5,
        "jun" => 6,
        "jul" => 7,
        "aug" => 8,
        "sep" => 9,
        "oct" => 10,
        "nov" => 11,
        "dec" => 12,
        _ => 1,
    }
}

pub fn format_ftp_date(month: &str, day: &str, time_or_year: &str) -> String {
    let m = month_to_num(month);
    let d = day.parse::<u32>().unwrap_or(1);

    if time_or_year.contains(':') {
        let current_year = unsafe {
            let mut tm: libc::tm = std::mem::zeroed();
            let now = libc::time(std::ptr::null_mut());
            if !libc::localtime_r(&now, &mut tm).is_null() {
                tm.tm_year + 1900
            } else {
                2026
            }
        };
        let time_parts: Vec<&str> = time_or_year.split(':').collect();
        let hh = time_parts.first().and_then(|h| h.parse::<u32>().ok()).unwrap_or(0);
        let mm = time_parts.get(1).and_then(|m| m.parse::<u32>().ok()).unwrap_or(0);
        let ss = time_parts.get(2).and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
        format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", current_year, m, d, hh, mm, ss)
    } else {
        let y = time_or_year.parse::<i32>().unwrap_or(2026);
        format!("{:04}-{:02}-{:02} 00:00:00", y, m, d)
    }
}

fn format_dos_date(date_part: &str, time_part: &str) -> String {
    let d_parts: Vec<&str> = date_part.split(|c| c == '-' || c == '/' || c == '.').collect();
    if d_parts.len() == 3 {
        let m = d_parts[0].parse::<u32>().unwrap_or(1);
        let d = d_parts[1].parse::<u32>().unwrap_or(1);
        let mut y = d_parts[2].parse::<i32>().unwrap_or(2026);
        if y < 100 {
            y += if y > 70 { 1900 } else { 2000 };
        }
        let is_pm = time_part.to_uppercase().contains("PM");
        let is_am = time_part.to_uppercase().contains("AM");
        let clean_time = time_part.trim_matches(|c: char| c.is_alphabetic());
        let t_parts: Vec<&str> = clean_time.split(':').collect();
        let mut hh = t_parts.first().and_then(|h| h.parse::<u32>().ok()).unwrap_or(0);
        let mm = t_parts.get(1).and_then(|m| m.parse::<u32>().ok()).unwrap_or(0);
        let ss = t_parts.get(2).and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
        if is_pm && hh < 12 {
            hh += 12;
        } else if is_am && hh == 12 {
            hh = 0;
        }
        format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", y, m, d, hh, mm, ss)
    } else {
        format!("{} {}", date_part, time_part)
    }
}

pub fn parse_datetime_to_ts(s: &str) -> i64 {
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.is_empty() {
        return 0;
    }
    let d_parts: Vec<&str> = parts[0].split('-').collect();
    if d_parts.len() < 3 {
        return 0;
    }
    let y = d_parts[0].parse::<i32>().unwrap_or(1970);
    let m = d_parts[1].parse::<i32>().unwrap_or(1);
    let d = d_parts[2].parse::<i32>().unwrap_or(1);

    let (hh, mm, ss) = if parts.len() > 1 {
        let t_parts: Vec<&str> = parts[1].split(':').collect();
        let hh = t_parts.first().and_then(|x| x.parse::<i32>().ok()).unwrap_or(0);
        let mm = t_parts.get(1).and_then(|x| x.parse::<i32>().ok()).unwrap_or(0);
        let ss = t_parts.get(2).and_then(|x| x.parse::<i32>().ok()).unwrap_or(0);
        (hh, mm, ss)
    } else {
        (0, 0, 0)
    };

    let mut tm: libc::tm = unsafe { std::mem::zeroed() };
    tm.tm_year = y - 1900;
    tm.tm_mon = m - 1;
    tm.tm_mday = d;
    tm.tm_hour = hh;
    tm.tm_min = mm;
    tm.tm_sec = ss;
    tm.tm_isdst = -1;
    let t = unsafe { libc::mktime(&mut tm) };
    if t < 0 { 0 } else { t as i64 }
}

/// Parses standard Unix `ls -la` or Windows/DOS directory listings into `Vec<FileEntry>`.
pub fn parse_unix_ls_output(raw: &str, remote_dir: &str, show_hidden: bool) -> Vec<FileEntry> {
    let mut entries = Vec::new();
    let clean_dir = remote_dir.trim_end_matches('/');

    if remote_dir != "/" && !remote_dir.is_empty() {
        let parent = match clean_dir.rfind('/') {
            Some(0) | None => "/".to_string(),
            Some(idx) => clean_dir[..idx].to_string(),
        };
        entries.push(FileEntry {
            name: "..".to_string(),
            path: parent,
            is_dir: true,
            is_symlink: false,
            size: 0,
            modified: "".to_string(),
            modified_ts: i64::MAX,
            created: None,
            permissions_str: "drwxr-xr-x".to_string(),
            octal_mode: 0o755,
            owner: "root".to_string(),
            selected: false,
            space_marked: false,
            calculating_size: false,
            calculated_dir_size: None,
        });
    }

    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("total ") {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        // Case 1: Standard Unix (drwxr-xr-x ...)
        if parts[0].len() >= 10
            && (parts[0].starts_with('d') || parts[0].starts_with('-') || parts[0].starts_with('l'))
        {
            let perm = parts[0];
            let is_dir = perm.starts_with('d');
            let is_symlink = perm.starts_with('l');
            let octal_mode = rwx_to_octal(perm);

            let (owner, size, modified, raw_name) = if parts.len() >= 9 {
                let owner = parts[2].to_string();
                let size = parts[4].parse::<u64>().unwrap_or(0);
                let mod_str = format_ftp_date(parts[5], parts[6], parts[7]);
                let name = if let Some(idx6) = line.find(parts[6]) {
                    if let Some(idx7) = line[idx6 + parts[6].len()..].find(parts[7]) {
                        let actual = idx6 + parts[6].len() + idx7 + parts[7].len();
                        line[actual..].trim_start().to_string()
                    } else {
                        parts[8..].join(" ")
                    }
                } else {
                    parts[8..].join(" ")
                };
                (owner, size, mod_str, name)
            } else if parts.len() >= 8 {
                let owner = parts[2].to_string();
                let size = parts[3].parse::<u64>().unwrap_or(0);
                let mod_str = format_ftp_date(parts[4], parts[5], parts[6]);
                let name = if let Some(idx5) = line.find(parts[5]) {
                    if let Some(idx6) = line[idx5 + parts[5].len()..].find(parts[6]) {
                        let actual = idx5 + parts[5].len() + idx6 + parts[6].len();
                        line[actual..].trim_start().to_string()
                    } else {
                        parts[7..].join(" ")
                    }
                } else {
                    parts[7..].join(" ")
                };
                (owner, size, mod_str, name)
            } else {
                continue;
            };

            let raw_clean = raw_name.trim_end_matches(['/', '\\']).to_string();
            let name = if is_symlink {
                raw_clean
                    .split(" -> ")
                    .next()
                    .unwrap_or(&raw_clean)
                    .trim()
                    .to_string()
            } else {
                raw_clean
            };

            if name == "." || name == ".." {
                continue;
            }
            if !show_hidden && name.starts_with('.') {
                continue;
            }

            let path = if clean_dir.is_empty() {
                format!("/{}", name)
            } else {
                format!("{}/{}", clean_dir, name)
            };

            entries.push(FileEntry {
                name,
                path,
                is_dir,
                is_symlink,
                size: if is_dir { 0 } else { size },
                modified: modified.clone(),
                modified_ts: parse_datetime_to_ts(&modified),
                created: None,
                permissions_str: mode_to_rwx(octal_mode, is_dir, is_symlink),
                octal_mode,
                owner,
                selected: false,
                space_marked: false,
                calculating_size: false,
                calculated_dir_size: None,
            });
        }
        // Case 2: Windows / DOS FTP listings
        // Handles:
        // "04-20-22  03:00PM       <DIR>          folder_name"
        // "04-20-22  03:00 PM      <DIR>          folder_name"
        // "04-20-22  03:00PM              123456  file_name.ext"
        // "04-20-22  03:00 PM             123456  file_name.ext"
        else if let Some(dir_pos) = parts.iter().position(|p| p.eq_ignore_ascii_case("<DIR>") || p.eq_ignore_ascii_case("[DIR]")) {
            if dir_pos + 1 < parts.len() {
                let modified = if dir_pos >= 2 {
                    format_dos_date(parts[0], parts[1])
                } else {
                    parts[..dir_pos].join(" ")
                };
                let raw_name = if let Some(idx) = line.find(parts[dir_pos]) {
                    line[idx + parts[dir_pos].len()..].trim_start().to_string()
                } else {
                    parts[dir_pos + 1..].join(" ")
                };
                let name = raw_name.trim_end_matches(['/', '\\']).to_string();

                if name == "." || name == ".." {
                    continue;
                }
                if !show_hidden && name.starts_with('.') {
                    continue;
                }

                let path = if clean_dir.is_empty() {
                    format!("/{}", name)
                } else {
                    format!("{}/{}", clean_dir, name)
                };

                entries.push(FileEntry {
                    name,
                    path,
                    is_dir: true,
                    is_symlink: false,
                    size: 0,
                    modified: modified.clone(),
                    modified_ts: parse_datetime_to_ts(&modified),
                    created: None,
                    permissions_str: "drwxr-xr-x".to_string(),
                    octal_mode: 0o755,
                    owner: String::new(),
                    selected: false,
                    space_marked: false,
                    calculating_size: false,
                    calculated_dir_size: None,
                });
            }
        } else if parts.len() >= 3 && parts[0].chars().any(|c| c.is_ascii_digit()) && (parts[0].contains('-') || parts[0].contains('/') || parts[0].contains('.')) {
            // Might be a DOS file with size column
            let size_pos = parts.iter().enumerate().skip(1).position(|(i, p)| {
                i < parts.len() - 1 && p.chars().all(|c| c.is_ascii_digit()) && !p.is_empty()
            }).map(|idx| idx + 1);

            if let Some(pos) = size_pos {
                let size = parts[pos].parse::<u64>().unwrap_or(0);
                let modified = if pos >= 2 {
                    format_dos_date(parts[0], parts[1])
                } else {
                    parts[..pos].join(" ")
                };
                let raw_name = if let Some(idx_date) = line.find(parts[0]) {
                    if let Some(idx_size) = line[idx_date + parts[0].len()..].find(parts[pos]) {
                        let actual = idx_date + parts[0].len() + idx_size + parts[pos].len();
                        line[actual..].trim_start().to_string()
                    } else {
                        parts[pos + 1..].join(" ")
                    }
                } else {
                    parts[pos + 1..].join(" ")
                };
                let name = raw_name.trim_end_matches(['/', '\\']).to_string();

                if name == "." || name == ".." {
                    continue;
                }
                if !show_hidden && name.starts_with('.') {
                    continue;
                }

                let path = if clean_dir.is_empty() {
                    format!("/{}", name)
                } else {
                    format!("{}/{}", clean_dir, name)
                };

                entries.push(FileEntry {
                    name,
                    path,
                    is_dir: false,
                    is_symlink: false,
                    size,
                    modified: modified.clone(),
                    modified_ts: parse_datetime_to_ts(&modified),
                    created: None,
                    permissions_str: "-rw-r--r--".to_string(),
                    octal_mode: 0o644,
                    owner: String::new(),
                    selected: false,
                    space_marked: false,
                    calculating_size: false,
                    calculated_dir_size: None,
                });
            }
        }
    }

    entries
}

fn rwx_to_octal(perm: &str) -> u32 {
    let bytes = perm.as_bytes();
    if bytes.len() < 10 {
        return 0o755;
    }
    let flags = [
        (1, 0o400, b'r'),
        (2, 0o200, b'w'),
        (3, 0o100, b'x'),
        (4, 0o040, b'r'),
        (5, 0o020, b'w'),
        (6, 0o010, b'x'),
        (7, 0o004, b'r'),
        (8, 0o002, b'w'),
        (9, 0o001, b'x'),
    ];
    let mut mode = 0;
    for (idx, bit, ch) in flags {
        if bytes[idx] == ch {
            mode |= bit;
        }
    }
    mode
}

pub fn run_remote_ssh_cmd(
    host: &str,
    port: u16,
    username: &str,
    remote_cmd: &str,
) -> Result<(), String> {
    let target = if username.is_empty() {
        host.to_string()
    } else {
        format!("{}@{}", username, host)
    };
    let output = Command::new("/usr/bin/ssh")
        .args([
            "-o",
            "BatchMode=yes",
            "-o",
            "ConnectTimeout=8",
            "-p",
            &port.to_string(),
            &target,
            remote_cmd,
        ])
        .output()
        .map_err(|e| e.to_string())?;
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

fn run_remote_ftp_quote(
    protocol: Protocol,
    host: &str,
    port: u16,
    username: &str,
    password: Option<&str>,
    quotes: &[&str],
) -> Result<(), String> {
    let (detected_proto, clean_host, parsed_port, _) = sanitize_host_and_path(host);
    let final_proto = detected_proto.unwrap_or(protocol);
    let final_host = if clean_host.is_empty() { host } else { &clean_host };
    let final_port = parsed_port.unwrap_or(port);

    let scheme = if final_proto == Protocol::FtpsImplicit {
        "ftps"
    } else {
        "ftp"
    };
    let url = format!("{}://{}:{}/", scheme, final_host, final_port);
    let pwd = password
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| read_password_from_keychain(final_host, username))
        .unwrap_or_default();

    let mut cmd = Command::new("/usr/bin/curl");
    cmd.args([
        "-sS",
        "--connect-timeout",
        "10",
        "--max-time",
        "20",
        "--disable-epsv",
        "-Q",
        "*OPTS UTF8 ON",
    ]);
    if protocol == Protocol::FtpsExplicit {
        cmd.args(["--ssl", "--insecure"]);
    } else if protocol == Protocol::FtpsImplicit {
        cmd.args(["--ssl-reqd", "--insecure"]);
    }
    if !username.is_empty() {
        if !pwd.is_empty() {
            cmd.args(["--user", &format!("{}:{}", username, pwd)]);
        } else {
            cmd.args(["--user", username]);
        }
    }
    for q in quotes {
        cmd.args(["-Q", q]);
    }
    cmd.arg(&url);

    let output = cmd.output().map_err(|e| e.to_string())?;
    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(if stderr.is_empty() {
            "FTP command failed".to_string()
        } else {
            stderr
        })
    }
}

pub fn remote_mkdir(
    protocol: Protocol,
    host: &str,
    port: u16,
    username: &str,
    password: Option<&str>,
    full_remote_path: &str,
) -> Result<(), String> {
    if is_demo_host(host) {
        return Ok(());
    }
    match protocol {
        Protocol::Sftp | Protocol::Scp => {
            let esc = full_remote_path.replace('\'', "'\\''");
            run_remote_ssh_cmd(host, port, username, &format!("mkdir -p '{}'", esc))
        }
        Protocol::Ftp | Protocol::FtpsExplicit | Protocol::FtpsImplicit => {
            let quote = format!("MKD {}", full_remote_path);
            run_remote_ftp_quote(protocol, host, port, username, password, &[&quote])
        }
    }
}

pub fn remote_rename(
    protocol: Protocol,
    host: &str,
    port: u16,
    username: &str,
    password: Option<&str>,
    old_path: &str,
    new_path: &str,
) -> Result<(), String> {
    if is_demo_host(host) {
        return Ok(());
    }
    match protocol {
        Protocol::Sftp | Protocol::Scp => {
            let esc_old = old_path.replace('\'', "'\\''");
            let esc_new = new_path.replace('\'', "'\\''");
            run_remote_ssh_cmd(
                host,
                port,
                username,
                &format!("mv '{}' '{}'", esc_old, esc_new),
            )
        }
        Protocol::Ftp | Protocol::FtpsExplicit | Protocol::FtpsImplicit => {
            let q1 = format!("RNFR {}", old_path);
            let q2 = format!("RNTO {}", new_path);
            run_remote_ftp_quote(protocol, host, port, username, password, &[&q1, &q2])
        }
    }
}

pub fn remote_delete(
    protocol: Protocol,
    host: &str,
    port: u16,
    username: &str,
    password: Option<&str>,
    remote_path: &str,
) -> Result<(), String> {
    if is_demo_host(host) {
        return Ok(());
    }
    match protocol {
        Protocol::Sftp | Protocol::Scp => {
            let esc = remote_path.replace('\'', "'\\''");
            run_remote_ssh_cmd(host, port, username, &format!("rm -rf '{}'", esc))
        }
        Protocol::Ftp | Protocol::FtpsExplicit | Protocol::FtpsImplicit => {
            // Try DELE first (file), then RMD (directory)
            let q_file = format!("DELE {}", remote_path);
            if run_remote_ftp_quote(protocol, host, port, username, password, &[&q_file]).is_ok() {
                return Ok(());
            }
            let q_dir = format!("RMD {}", remote_path);
            run_remote_ftp_quote(protocol, host, port, username, password, &[&q_dir])
        }
    }
}

pub fn remote_chmod(
    protocol: Protocol,
    host: &str,
    port: u16,
    username: &str,
    password: Option<&str>,
    remote_path: &str,
    octal_mode: u32,
    recursive: bool,
) -> Result<(), String> {
    if is_demo_host(host) {
        return Ok(());
    }
    match protocol {
        Protocol::Sftp | Protocol::Scp => {
            let esc = remote_path.replace('\'', "'\\''");
            let flag = if recursive { "-R " } else { "" };
            run_remote_ssh_cmd(
                host,
                port,
                username,
                &format!("chmod {}{:04o} '{}'", flag, octal_mode, esc),
            )
        }
        Protocol::Ftp | Protocol::FtpsExplicit | Protocol::FtpsImplicit => {
            let quote = format!("SITE CHMOD {:04o} {}", octal_mode, remote_path);
            run_remote_ftp_quote(protocol, host, port, username, password, &[&quote])
        }
    }
}

pub fn calculate_remote_dir_size(
    protocol: Protocol,
    host: &str,
    port: u16,
    username: &str,
    password: Option<&str>,
    remote_path: &str,
) -> Result<u64, String> {
    if is_demo_host(host) {
        return Ok(0);
    }

    if matches!(protocol, Protocol::Sftp | Protocol::Scp) {
        let (detected_proto, clean_host, parsed_port, _) = sanitize_host_and_path(host);
        let _ = detected_proto;
        let final_port = parsed_port.unwrap_or(port);
        let final_host = if clean_host.is_empty() { host } else { &clean_host };
        let mut cmd = std::process::Command::new("ssh");
        cmd.arg("-p")
            .arg(final_port.to_string())
            .arg("-o")
            .arg("BatchMode=yes")
            .arg("-o")
            .arg("ConnectTimeout=3")
            .arg(format!("{}@{}", username, final_host))
            .arg(format!("du -sk \"{}\"", remote_path));
        if let Ok(out) = cmd.output() {
            if out.status.success() {
                let s = String::from_utf8_lossy(&out.stdout);
                if let Some(first_word) = s.split_whitespace().next() {
                    if let Ok(kb) = first_word.parse::<u64>() {
                        return Ok(kb * 1024);
                    }
                }
            }
        }
    }

    // Fallback: shallow recursive listing
    let mut total = 0u64;
    let mut stack = vec![remote_path.to_string()];
    let mut count = 0;
    while let Some(curr) = stack.pop() {
        count += 1;
        if count > 100 {
            break;
        }
        if let Ok(children) = list_remote_directory(protocol, host, port, username, password, &curr, true) {
            for child in children {
                if child.name == ".." || child.name == "." {
                    continue;
                }
                if child.is_dir {
                    stack.push(child.path);
                } else {
                    total += child.size;
                }
            }
        }
    }
    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_utf8() {
        let raw = "příliš žluťoučký kůň úpěl ďábelské ódy".as_bytes();
        assert_eq!(decode_remote_listing(raw), "příliš žluťoučký kůň úpěl ďábelské ódy");
    }

    #[test]
    fn test_decode_cp1250_czech() {
        let cp1250_bytes = [
            0x50, 0xF8, 0xED, 0x6C, 0x6F, 0x68, 0x61, 0x20, 0x2D, 0x20, 0x76, 0x79, 0xFA, 0xE8,
            0x74, 0x6F, 0x76, 0xE1, 0x6E, 0xED, 0x2E, 0x70, 0x64, 0x66,
        ];
        assert_eq!(decode_remote_listing(&cp1250_bytes), "Příloha - vyúčtování.pdf");
    }

    #[test]
    fn test_decode_cp1250_all_czech_chars() {
        let cp1250_spec = [0x9A, 0x9D, 0x9E, 0x20, 0x8A, 0x8D, 0x8E];
        assert_eq!(decode_remote_listing(&cp1250_spec), "šťž ŠŤŽ");
    }

    #[test]
    fn test_decode_iso8859_2() {
        let iso_bytes = [0xB9, 0xBB, 0xBE, 0x20, 0xA9, 0xAB, 0xAE];
        assert_eq!(decode_remote_listing(&iso_bytes), "šťž ŠŤŽ");
    }

    #[test]
    fn test_percent_encode_cp1250() {
        // En-dash \u{2013} in CP1250 is 0x96 -> %96
        let path = "/test/02 Gravitacni pole – sila.docx";
        let encoded = percent_encode_path_cp1250(path);
        assert_eq!(encoded, "/test/02%20Gravitacni%20pole%20%96%20sila.docx");
    }

    #[test]
    fn test_parse_listing_preserves_multiple_spaces() {
        let dos_listing = "06-26-24  11:52PM               188756 01  Pojmy ze zakladniho kurzu Fyziky.pdf\r\n\
                           04-20-22  03:00PM       <DIR>          folder   with   spaces\r\n";
        let entries = parse_unix_ls_output(dos_listing, "/dir", false);
        assert_eq!(entries.len(), 3); // ".." + 2 entries
        assert_eq!(entries[1].name, "01  Pojmy ze zakladniho kurzu Fyziky.pdf");
        assert_eq!(entries[2].name, "folder   with   spaces");

        let unix_listing = "drwxr-xr-x 2 filip staff 4096 Oct 28 11:32 01  Pojmy  test.pdf\r\n";
        let entries_unix = parse_unix_ls_output(unix_listing, "/dir", false);
        assert_eq!(entries_unix[1].name, "01  Pojmy  test.pdf");
    }
}
