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
    let remote_cmd = format!("LC_ALL=C ls -la '{}'", escaped_path);

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

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(parse_unix_ls_output(&stdout, remote_path, show_hidden))
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

    let stdout = String::from_utf8_lossy(&output.stdout);
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

            loop {
                while let Ok((kind, msg)) = rx_log.try_recv() {
                    on_log(kind, msg);
                }

                match child.try_wait() {
                    Ok(Some(status)) => {
                        let _ = reader_thread.join();
                        while let Ok((kind, msg)) = rx_log.try_recv() {
                            on_log(kind, msg);
                        }
                        if status.success() {
                            on_progress(file_size, file_size);
                            return Ok(());
                        } else {
                            return Err(format!("Curl upload selhal s kódem {}", status));
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
            cmd.args(["-o", &local_target_path.to_string_lossy(), &url]);
            cmd.stderr(std::process::Stdio::piped());

            let mut child = cmd.spawn().map_err(|e| e.to_string())?;
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
            loop {
                while let Ok((kind, msg, sz)) = rx_log.try_recv() {
                    if sz > 0 {
                        detected_total = sz;
                    }
                    on_log(kind, msg);
                }

                let cur = std::fs::metadata(local_target_path).map(|m| m.len()).unwrap_or(0);
                if cur > 0 || detected_total > 0 {
                    on_progress(cur, detected_total);
                }

                match child.try_wait() {
                    Ok(Some(status)) => {
                        let _ = reader_thread.join();
                        while let Ok((kind, msg, sz)) = rx_log.try_recv() {
                            if sz > 0 {
                                detected_total = sz;
                            }
                            on_log(kind, msg);
                        }
                        if status.success() {
                            let final_len = std::fs::metadata(local_target_path).map(|m| m.len()).unwrap_or(0);
                            let total = if detected_total > 0 { detected_total.max(final_len) } else { final_len };
                            on_progress(total, total);
                            return Ok(());
                        } else {
                            return Err(format!("Curl download selhal s kódem {}", status));
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
                let name = parts[8..].join(" ");
                (owner, size, mod_str, name)
            } else if parts.len() >= 8 {
                let owner = parts[2].to_string();
                let size = parts[3].parse::<u64>().unwrap_or(0);
                let mod_str = format_ftp_date(parts[4], parts[5], parts[6]);
                let name = parts[7..].join(" ");
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
                let raw_name = parts[dir_pos + 1..].join(" ");
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
                let raw_name = parts[pos + 1..].join(" ");
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
