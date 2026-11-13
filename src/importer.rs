use crate::config::read_password_from_keychain;
use crate::model::{Protocol, SavedSiteProfile};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const PWALG_SIMPLE_MAGIC: u8 = 0xA3;
const PWALG_SIMPLE_FLAG: u8 = 0xFF;
const PWALG_SIMPLE_INTERNAL: u8 = 0x00;
const PWALG_SIMPLE_INTERNAL2: u8 = 0x02;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ImportSourceType {
    RusterBackup,
    WinScp,
    SshConfig,
}

impl ImportSourceType {
    pub fn default_extension(&self) -> &'static [&'static str] {
        match self {
            ImportSourceType::RusterBackup => &["json"],
            ImportSourceType::WinScp => &["ini", "reg", "txt"],
            ImportSourceType::SshConfig => &["config", "txt"],
        }
    }
}

#[derive(Clone, Debug)]
pub struct ImportOptions {
    pub import_sites: bool,
    pub save_passwords_to_keychain: bool,
    pub import_settings: bool,
    pub import_favorites: bool,
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self {
            import_sites: true,
            save_passwords_to_keychain: true,
            import_settings: true,
            import_favorites: true,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ExportOptions {
    pub export_sites: bool,
    pub export_passwords: bool,
    pub export_settings: bool,
    pub export_favorites: bool,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            export_sites: true,
            export_passwords: true,
            export_settings: true,
            export_favorites: true,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ImportSummary {
    pub servers_imported: usize,
    pub servers_skipped: usize,
    pub passwords_saved: usize,
    pub settings_imported: bool,
    pub favorites_imported: usize,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RusterBackup {
    pub version: u32,
    pub exported_at: String,
    pub app_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<RusterSettingsBackup>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub favorites: Option<Vec<PathBuf>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sites: Option<Vec<RusterSiteBackup>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RusterSiteBackup {
    pub name: String,
    #[serde(default)]
    pub group: String,
    pub folder: String,
    pub protocol: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    pub auth_type: String,
    pub env_color: u32,
    pub max_connections: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RusterSettingsBackup {
    pub show_local_pane: bool,
    pub show_hidden: bool,
    pub show_function_bar: bool,
    pub show_permissions: bool,
    pub date_format: String,
    pub max_concurrent_transfers: usize,
    pub editable_extensions: String,
    pub theme: String,
    pub locale: String,
}

/// URL percent-decoding helper for WinSCP session and folder names (e.g., %20 -> space, %2F -> /)
pub fn percent_decode(input: &str) -> String {
    let mut bytes = Vec::with_capacity(input.len());
    let mut chars = input.as_bytes().iter();
    while let Some(&b) = chars.next() {
        if b == b'%' {
            let h1 = chars.next().copied();
            let h2 = chars.next().copied();
            if let (Some(h1), Some(h2)) = (h1, h2) {
                if let Ok(val) = u8::from_str_radix(
                    std::str::from_utf8(&[h1, h2]).unwrap_or(""),
                    16,
                ) {
                    bytes.push(val);
                    continue;
                } else {
                    bytes.push(b'%');
                    bytes.push(h1);
                    bytes.push(h2);
                    continue;
                }
            } else {
                bytes.push(b'%');
                if let Some(h1) = h1 {
                    bytes.push(h1);
                }
                continue;
            }
        }
        bytes.push(b);
    }
    String::from_utf8(bytes).unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).into_owned())
}

/// Decrypts a WinSCP obfuscated/encrypted password string.
/// WinSCP stores passwords using a reversible bitwise XOR/NOT algorithm keyed with username + hostname.
pub fn decrypt_winscp_password(hex_str: &str, username: &str, hostname: &str) -> Option<String> {
    let hex_clean = hex_str.trim().trim_matches('"');
    if hex_clean.len() < 4 || hex_clean.len() % 2 != 0 {
        return None;
    }

    let mut dec_bytes = Vec::with_capacity(hex_clean.len() / 2);
    for chunk in hex_clean.as_bytes().chunks_exact(2) {
        let hex_byte = u8::from_str_radix(std::str::from_utf8(chunk).ok()?, 16).ok()?;
        // WinSCP character deobfuscation: inverse of (~Ch) ^ PWALG_SIMPLE_MAGIC
        let dec_byte = !(hex_byte ^ PWALG_SIMPLE_MAGIC);
        dec_bytes.push(dec_byte);
    }

    if dec_bytes.is_empty() {
        return None;
    }

    let mut idx = 0;
    let flag = dec_bytes[idx];
    idx += 1;

    let length: usize;
    if flag == PWALG_SIMPLE_FLAG {
        if idx >= dec_bytes.len() {
            return None;
        }
        let ver = dec_bytes[idx];
        idx += 1;
        if ver == PWALG_SIMPLE_INTERNAL {
            if idx >= dec_bytes.len() {
                return None;
            }
            length = dec_bytes[idx] as usize;
            idx += 1;
        } else if ver == PWALG_SIMPLE_INTERNAL2 {
            if idx + 1 >= dec_bytes.len() {
                return None;
            }
            length = ((dec_bytes[idx] as usize) << 8) | (dec_bytes[idx + 1] as usize);
            idx += 2;
        } else {
            return None;
        }
    } else {
        length = flag as usize;
    }

    if idx >= dec_bytes.len() {
        return None;
    }
    let shift = dec_bytes[idx] as usize;
    idx += 1;

    // Skip `shift` random dummy padding bytes
    idx += shift;
    if idx >= dec_bytes.len() {
        return None;
    }

    let end = (idx + length).min(dec_bytes.len());
    let plain_raw = &dec_bytes[idx..end];
    let plain_str = String::from_utf8(plain_raw.to_vec())
        .unwrap_or_else(|_| String::from_utf8_lossy(plain_raw).into_owned());

    // WinSCP prefixes the payload with (UserName + HostName)
    let prefix = format!("{}{}", username, hostname);
    let prefix_char_len = prefix.chars().count();
    let result = if !prefix.is_empty()
        && plain_str
            .chars()
            .take(prefix_char_len)
            .collect::<String>()
            .eq_ignore_ascii_case(&prefix)
    {
        plain_str.chars().skip(prefix_char_len).collect()
    } else {
        plain_str
    };

    Some(result)
}

/// Parses WinSCP exported settings (either `WinSCP.ini` or Windows Registry `.reg` files).
pub fn parse_winscp_content(content: &str) -> Vec<SavedSiteProfile> {
    let mut sites = Vec::new();

    #[derive(Default)]
    struct RawSession {
        raw_name: String,
        protocol_num: Option<u32>,
        ftps_num: Option<u32>,
        host: String,
        port: Option<u16>,
        username: String,
        raw_password: Option<String>,
        public_key: Option<String>,
        remote_directory: Option<String>,
    }

    let flush_session = |sess: Option<RawSession>, out: &mut Vec<SavedSiteProfile>| {
        let Some(s) = sess else { return };
        if s.host.trim().is_empty() {
            return;
        }

        // Determine Protocol
        let protocol = match s.protocol_num.unwrap_or(0) {
            5 => match s.ftps_num.unwrap_or(0) {
                1 | 2 => Protocol::FtpsExplicit,
                3 => Protocol::FtpsImplicit,
                _ => Protocol::Ftp,
            },
            1 => Protocol::Scp,
            _ => Protocol::Sftp,
        };

        let port = s.port.unwrap_or_else(|| protocol.default_port());
        let username = if s.username.is_empty() {
            std::env::var("USER").unwrap_or_else(|_| "root".to_string())
        } else {
            s.username
        };

        // Decrypt password if present
        let password = s.raw_password.as_deref().and_then(|raw| {
            decrypt_winscp_password(raw, &username, &s.host)
        });

        let auth_type = if password.is_some() {
            "Password".to_string()
        } else if let Some(ref pk) = s.public_key {
            if !pk.is_empty() {
                format!("SSH Key ({})", pk)
            } else {
                "Password".to_string()
            }
        } else {
            "Password".to_string()
        };

        // Decode session name and extract WinSCP folder hierarchy into `group`,
        // while `folder` holds the initial remote directory path.
        let decoded = percent_decode(&s.raw_name).replace('\\', "/");
        let (group, name) = if let Some((g, n)) = decoded.rsplit_once('/') {
            (
                g.trim().trim_matches('/').to_string(),
                n.trim().to_string(),
            )
        } else {
            (String::new(), decoded.trim().to_string())
        };
        let folder = s
            .remote_directory
            .as_deref()
            .map(str::trim)
            .filter(|d| !d.is_empty())
            .map(|d| {
                if d.starts_with('/') {
                    d.to_string()
                } else {
                    format!("/{}", d)
                }
            })
            .unwrap_or_default();

        let env_color = match protocol {
            Protocol::Ftp => 0xd97706,          // Amber
            Protocol::FtpsExplicit | Protocol::FtpsImplicit => 0x2563eb, // Blue
            Protocol::Sftp => 0x059669,         // Emerald
            Protocol::Scp => 0x7c3aed,          // Purple
        };

        out.push(SavedSiteProfile {
            name,
            group,
            folder,
            protocol,
            host: s.host,
            port,
            username,
            password,
            auth_type,
            env_color,
            max_connections: 2,
        });
    };

    let mut current_sess: Option<RawSession> = None;

    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
            continue;
        }

        // Section header
        if line.starts_with('[') && line.ends_with(']') {
            let section = &line[1..line.len() - 1];

            // Match [Sessions\SessionName] or [HKEY_CURRENT_USER\...\Sessions\SessionName]
            let session_id = if let Some((_, rest)) = section.split_once("Sessions\\") {
                Some(rest)
            } else if let Some((_, rest)) = section.split_once("Sessions/") {
                Some(rest)
            } else {
                None
            };

            if let Some(sess_name) = session_id {
                let sess_name = sess_name.trim();
                if sess_name.is_empty()
                    || sess_name.eq_ignore_ascii_case("Default%20Settings")
                    || sess_name.eq_ignore_ascii_case("Default Settings")
                {
                    flush_session(current_sess.take(), &mut sites);
                    continue;
                }

                flush_session(current_sess.take(), &mut sites);
                current_sess = Some(RawSession {
                    raw_name: sess_name.to_string(),
                    ..Default::default()
                });
            } else {
                flush_session(current_sess.take(), &mut sites);
            }
            continue;
        }

        let Some(ref mut sess) = current_sess else {
            continue;
        };

        let Some((raw_key, raw_val)) = line.split_once('=') else {
            continue;
        };

        let key = raw_key.trim().trim_matches('"').to_lowercase();
        let val_clean = raw_val.trim().trim_matches('"');

        // Handle registry dword:00000015 hex format vs standard decimal
        let parse_u32 = |v: &str| -> Option<u32> {
            if let Some(hex_part) = v.strip_prefix("dword:") {
                u32::from_str_radix(hex_part, 16).ok()
            } else {
                v.parse::<u32>().ok()
            }
        };

        let is_quoted = raw_val.trim().starts_with('"');
        let decode_val = |v: &str| -> String {
            let decoded = percent_decode(v);
            if is_quoted {
                decoded.replace("\\\\", "\\")
            } else {
                decoded
            }
        };

        match key.as_str() {
            "fsprotocol" => {
                sess.protocol_num = parse_u32(val_clean);
            }
            "ftps" => {
                sess.ftps_num = parse_u32(val_clean);
            }
            "hostname" => {
                sess.host = decode_val(val_clean);
            }
            "portnumber" => {
                if let Some(p) = parse_u32(val_clean) {
                    sess.port = Some(p as u16);
                }
            }
            "username" => {
                sess.username = decode_val(val_clean);
            }
            "password" => {
                sess.raw_password = Some(val_clean.to_string());
            }
            "publickeyfile" => {
                sess.public_key = Some(decode_val(val_clean));
            }
            "remotedirectory" => {
                sess.remote_directory = Some(decode_val(val_clean));
            }
            _ => {}
        }
    }

    flush_session(current_sess.take(), &mut sites);
    sites
}

/// Generates a full Ruster JSON backup string.
pub fn create_ruster_backup(
    state: &crate::config::AppState,
    sites: &[SavedSiteProfile],
    favorites: &[PathBuf],
    options: &ExportOptions,
) -> Result<String, String> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let mut backup = RusterBackup {
        version: 1,
        exported_at: format!("{}", now),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        settings: None,
        favorites: None,
        sites: None,
    };

    if options.export_settings {
        backup.settings = Some(RusterSettingsBackup {
            show_local_pane: state.show_local_pane,
            show_hidden: state.show_hidden,
            show_function_bar: state.show_function_bar,
            show_permissions: state.show_permissions,
            date_format: state.date_format.as_str().to_string(),
            max_concurrent_transfers: state.max_concurrent_transfers,
            editable_extensions: state.editable_extensions.clone(),
            theme: state.theme.as_str().to_string(),
            locale: state.locale.as_code().to_string(),
        });
    }

    if options.export_favorites {
        backup.favorites = Some(favorites.to_vec());
    }

    if options.export_sites {
        let mut site_backups = Vec::with_capacity(sites.len());
        for s in sites {
            let password = if options.export_passwords {
                read_password_from_keychain(&s.host, &s.username)
            } else {
                None
            };
            site_backups.push(RusterSiteBackup {
                name: s.name.clone(),
                group: s.group.clone(),
                folder: s.folder.clone(),
                protocol: s.protocol.label().to_string(),
                host: s.host.clone(),
                port: s.port,
                username: s.username.clone(),
                password,
                auth_type: s.auth_type.clone(),
                env_color: s.env_color,
                max_connections: s.max_connections,
            });
        }
        backup.sites = Some(site_backups);
    }

    serde_json::to_string_pretty(&backup).map_err(|e| format!("Serialization error: {}", e))
}

/// Parses a Ruster JSON backup string.
pub fn parse_ruster_backup(json_str: &str) -> Result<RusterBackup, String> {
    serde_json::from_str::<RusterBackup>(json_str).map_err(|e| format!("Invalid backup JSON: {}", e))
}

/// Converts a `RusterSiteBackup` back to a `SavedSiteProfile`.
pub fn site_backup_to_profile(s: &RusterSiteBackup) -> SavedSiteProfile {
    let protocol = match s.protocol.trim() {
        "FTP" => Protocol::Ftp,
        "FTPS (Explicit)" | "FTPS Explicit" => Protocol::FtpsExplicit,
        "FTPS (Implicit)" | "FTPS Implicit" => Protocol::FtpsImplicit,
        "SCP" => Protocol::Scp,
        _ => Protocol::Sftp,
    };

    SavedSiteProfile {
        name: s.name.clone(),
        group: s.group.clone(),
        folder: s.folder.clone(),
        protocol,
        host: s.host.clone(),
        port: s.port,
        username: s.username.clone(),
        password: s.password.clone(),
        auth_type: s.auth_type.clone(),
        env_color: s.env_color,
        max_connections: s.max_connections.clamp(1, 10),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_percent_decode() {
        assert_eq!(percent_decode("Hello%20World"), "Hello World");
        assert_eq!(percent_decode("Group%2FServer%201"), "Group/Server 1");
        assert_eq!(percent_decode("Simple"), "Simple");
    }

    #[test]
    fn test_decrypt_winscp_password() {
        // Test vector generated by standard WinSCP obfuscation routine:
        // Plaintext: "secret123", Username: "admin", Hostname: "ftp.example.com"
        let hex = "A35C415F85329B3D383135323A282C7239243D312C3039723F33312F393F2E39286D6E6F";
        let dec = decrypt_winscp_password(hex, "admin", "ftp.example.com");
        assert_eq!(dec, Some("secret123".to_string()));
    }

    #[test]
    fn test_parse_winscp_ini() {
        let sample = r#"
[Sessions\Default%20Settings]
FSProtocol=5

[Sessions\Production%20FTP]
FSProtocol=5
Ftps=0
HostName=ftp.production.cz
PortNumber=21
UserName=prod_user
Password=A35C415F85329B3D383135323A282C7239243D312C3039723F33312F393F2E39286D6E6F

[Sessions\Clients%2FStaging%20SFTP]
FSProtocol=0
HostName=sftp.client.com
PortNumber=2222
UserName=client_deploy
"#;
        let sites = parse_winscp_content(sample);
        assert_eq!(sites.len(), 2);

        let ftp = &sites[0];
        assert_eq!(ftp.name, "Production FTP");
        assert_eq!(ftp.group, "");
        assert_eq!(ftp.folder, "");
        assert_eq!(ftp.protocol, Protocol::Ftp);
        assert_eq!(ftp.host, "ftp.production.cz");
        assert_eq!(ftp.port, 21);
        assert_eq!(ftp.username, "prod_user");

        let sftp = &sites[1];
        assert_eq!(sftp.name, "Staging SFTP");
        assert_eq!(sftp.group, "Clients");
        assert_eq!(sftp.folder, "");
        assert_eq!(sftp.protocol, Protocol::Sftp);
        assert_eq!(sftp.host, "sftp.client.com");
        assert_eq!(sftp.port, 2222);
        assert_eq!(sftp.username, "client_deploy");
    }

    #[test]
    fn test_parse_winscp_registry_reg() {
        let sample = r#"
Windows Registry Editor Version 5.00

[HKEY_CURRENT_USER\Software\Martin Prikryl\WinSCP 2\Sessions\Secure%20FTPS]
"FSProtocol"=dword:00000005
"Ftps"=dword:00000001
"HostName"="ftps.bank.cz"
"PortNumber"=dword:00000015
"UserName"="bank_user"
"#;
        let sites = parse_winscp_content(sample);
        assert_eq!(sites.len(), 1);
        let site = &sites[0];
        assert_eq!(site.name, "Secure FTPS");
        assert_eq!(site.protocol, Protocol::FtpsExplicit);
        assert_eq!(site.host, "ftps.bank.cz");
        assert_eq!(site.port, 21);
        assert_eq!(site.username, "bank_user");
    }

    #[test]
    fn test_ruster_backup_roundtrip() {
        let state = crate::config::load_app_state();
        let sites = vec![SavedSiteProfile {
            name: "Test Site".to_string(),
            group: "Work".to_string(),
            folder: "/var/www".to_string(),
            protocol: Protocol::Sftp,
            host: "test.example.com".to_string(),
            port: 22,
            username: "tester".to_string(),
            password: Some("mypassword".to_string()),
            auth_type: "Password".to_string(),
            env_color: 0x2563eb,
            max_connections: 2,
        }];
        let favorites = vec![PathBuf::from("/Users/test")];
        let options = ExportOptions::default();

        let json = create_ruster_backup(&state, &sites, &favorites, &options).unwrap();
        let parsed = parse_ruster_backup(&json).unwrap();

        assert_eq!(parsed.version, 1);
        assert!(parsed.settings.is_some());
        assert_eq!(parsed.favorites.unwrap().len(), 1);
        let loaded_sites = parsed.sites.unwrap();
        assert_eq!(loaded_sites.len(), 1);
        assert_eq!(loaded_sites[0].name, "Test Site");
        assert_eq!(loaded_sites[0].host, "test.example.com");
    }

    #[test]
    fn test_winscp_percent_encoded_username_and_password() {
        // Construct WinSCP v2 obfuscated password for payload: "vslg\\ftpstu" + "ftp.vslg.cz" + "secret123"
        let payload = b"vslg\\ftpstuftp.vslg.czsecret123";
        let mut raw = vec![0xFFu8, 0x00u8, payload.len() as u8, 0x00u8];
        raw.extend_from_slice(payload);
        let hex: String = raw
            .into_iter()
            .map(|b| format!("{:02X}", (!b) ^ 0xA3))
            .collect();

        let ini = format!(
            "[Sessions\\Test%20FTP]\nFSProtocol=5\nHostName=ftp.vslg.cz\nPortNumber=21\nUserName=vslg%5Cftpstu\nPassword={}\n",
            hex
        );
        let sites = parse_winscp_content(&ini);
        assert_eq!(sites.len(), 1);
        assert_eq!(sites[0].username, "vslg\\ftpstu");
        assert_eq!(sites[0].password.as_deref(), Some("secret123"));
    }
}
