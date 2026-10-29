use crate::model::{
    default_editable_extensions, initial_local_dir, initial_right_dir, Protocol, SavedSiteProfile,
};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Clone, Debug)]
pub struct AppState {
    pub show_local_pane: bool,
    pub show_hidden: bool,
    pub show_function_bar: bool,
    pub show_permissions: bool,
    pub date_format: crate::model::DateFormat,
    pub local_dir: PathBuf,
    pub right_dir: PathBuf,
    pub max_concurrent_transfers: usize,
    pub editable_extensions: String,
    pub window_width: f32,
    pub window_height: f32,
    pub queue_panel_height: f32,
    pub theme: crate::theme::ThemeMode,
    pub locale: crate::i18n::Locale,
}

pub fn config_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/Users".to_string());
    let ruster_dir = PathBuf::from(&home).join(".config").join("ruster");
    let legacy_dir = PathBuf::from(&home).join(".config").join("macscp");
    if !ruster_dir.exists() && legacy_dir.exists() {
        let _ = fs::rename(&legacy_dir, &ruster_dir);
    }
    ruster_dir
}

pub fn sites_file_path() -> PathBuf {
    config_dir().join("sites.conf")
}

pub fn state_file_path() -> PathBuf {
    config_dir().join("state.conf")
}

pub fn favorites_file_path() -> PathBuf {
    config_dir().join("favorites.conf")
}

/// Loads favorite locations from `~/.config/ruster/favorites.conf`.
/// If the file does not exist or is empty, returns standard default local paths.
pub fn load_favorite_paths() -> Vec<PathBuf> {
    let path = favorites_file_path();
    if let Ok(content) = fs::read_to_string(&path) {
        let mut paths = Vec::new();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let p = PathBuf::from(line);
            if !paths.contains(&p) {
                paths.push(p);
            }
        }
        if !paths.is_empty() {
            return paths;
        }
    }

    let home = std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/Users"));
    let mut defaults = Vec::new();
    defaults.push(home.clone());
    let downloads = home.join("Downloads");
    if downloads.exists() {
        defaults.push(downloads);
    }
    defaults
}

/// Saves favorite paths to `~/.config/ruster/favorites.conf`.
pub fn save_favorite_paths(favorites: &[PathBuf]) -> std::io::Result<()> {
    let dir = config_dir();
    fs::create_dir_all(&dir)?;
    let mut out = String::from("# Ruster Favorite Locations\n");
    for path in favorites {
        out.push_str(&path.to_string_lossy());
        out.push('\n');
    }
    fs::write(favorites_file_path(), out)
}

pub fn cleanup_remote_edit_temp() {
    let dir = std::env::temp_dir().join("ruster_remote_edit");
    if dir.exists() {
        let _ = fs::remove_dir_all(&dir);
    }
    let legacy = std::env::temp_dir().join("macscp_remote_edit");
    if legacy.exists() {
        let _ = fs::remove_dir_all(&legacy);
    }
}

pub fn load_app_state() -> AppState {
    let default_local = initial_local_dir();
    let default_right = initial_right_dir();
    let mut show_local_pane = true;
    let mut show_hidden = false;
    let mut show_function_bar = true;
    let mut show_permissions = false;
    let mut date_format = crate::model::DateFormat::Date;
    let mut local_dir = default_local.clone();
    let mut right_dir = default_right.clone();
    let mut max_concurrent_transfers = 2usize;
    let mut editable_extensions = default_editable_extensions();
    let mut window_width = 1280.0f32;
    let mut window_height = 800.0f32;
    let mut queue_panel_height = 140.0f32;
    let mut theme = crate::theme::ThemeMode::Light;
    let mut locale = crate::i18n::Locale::En;

    if let Ok(content) = fs::read_to_string(state_file_path()) {
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((k, v)) = line.split_once('=') {
                match k.trim() {
                    "show_local_pane" => {
                        if let Ok(b) = v.trim().parse::<bool>() {
                            show_local_pane = b;
                        }
                    }
                    "show_hidden" => {
                        if let Ok(b) = v.trim().parse::<bool>() {
                            show_hidden = b;
                        }
                    }
                    "show_function_bar" => {
                        if let Ok(b) = v.trim().parse::<bool>() {
                            show_function_bar = b;
                        }
                    }
                    "show_permissions" => {
                        if let Ok(b) = v.trim().parse::<bool>() {
                            show_permissions = b;
                        }
                    }
                    "date_format" => {
                        date_format = crate::model::DateFormat::from_str(v.trim());
                    }
                    "local_dir" => {
                        let p = PathBuf::from(v.trim());
                        if p.exists() && p.is_dir() {
                            local_dir = p;
                        }
                    }
                    "right_dir" => {
                        let p = PathBuf::from(v.trim());
                        if p.exists() && p.is_dir() {
                            right_dir = p;
                        }
                    }
                    "max_concurrent_transfers" => {
                        if let Ok(n) = v.trim().parse::<usize>() {
                            max_concurrent_transfers = n.clamp(1, 10);
                        }
                    }
                    "editable_extensions" => {
                        let val = v.trim();
                        if !val.is_empty() {
                            editable_extensions = val.to_string();
                        }
                    }
                    "window_width" => {
                        if let Ok(w) = v.trim().parse::<f32>() {
                            if w >= 600.0 {
                                window_width = w;
                            }
                        }
                    }
                    "window_height" => {
                        if let Ok(h) = v.trim().parse::<f32>() {
                            if h >= 400.0 {
                                window_height = h;
                            }
                        }
                    }
                    "queue_panel_height" => {
                        if let Ok(h) = v.trim().parse::<f32>() {
                            queue_panel_height = h.clamp(60.0, 800.0);
                        }
                    }
                    "theme" => {
                        theme = crate::theme::ThemeMode::from_str(v.trim());
                    }
                    "locale" => {
                        locale = crate::i18n::Locale::from_code(v.trim());
                    }
                    _ => {}
                }
            }
        }
    }

    AppState {
        show_local_pane,
        show_hidden,
        show_function_bar,
        show_permissions,
        date_format,
        local_dir,
        right_dir,
        max_concurrent_transfers,
        editable_extensions,
        window_width,
        window_height,
        queue_panel_height,
        theme,
        locale,
    }
}

pub fn save_app_state(
    show_local_pane: bool,
    show_hidden: bool,
    show_function_bar: bool,
    show_permissions: bool,
    date_format: crate::model::DateFormat,
    local_dir: &std::path::Path,
    right_dir: &std::path::Path,
    max_concurrent_transfers: usize,
    editable_extensions: &str,
    window_width: f32,
    window_height: f32,
    queue_panel_height: f32,
    theme: crate::theme::ThemeMode,
    locale: crate::i18n::Locale,
) -> std::io::Result<()> {
    let dir = config_dir();
    fs::create_dir_all(&dir)?;
    let content = format!(
        "show_local_pane={}\nshow_hidden={}\nshow_function_bar={}\nshow_permissions={}\ndate_format={}\nlocal_dir={}\nright_dir={}\nmax_concurrent_transfers={}\neditable_extensions={}\nwindow_width={}\nwindow_height={}\nqueue_panel_height={}\ntheme={}\nlocale={}\n",
        show_local_pane,
        show_hidden,
        show_function_bar,
        show_permissions,
        date_format.as_str(),
        local_dir.to_string_lossy(),
        right_dir.to_string_lossy(),
        max_concurrent_transfers,
        editable_extensions,
        window_width.max(600.0),
        window_height.max(400.0),
        queue_panel_height.clamp(60.0, 800.0),
        theme.as_str(),
        locale.as_code(),
    );
    fs::write(state_file_path(), content)
}

/// Loads saved sites from `~/.config/ruster/sites.conf`.
pub fn load_saved_sites() -> Option<Vec<SavedSiteProfile>> {
    let path = sites_file_path();
    let content = fs::read_to_string(path).ok()?;
    let mut sites = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 8 {
            let name = parts[0].to_string();
            let folder = parts[1].to_string();
            let protocol = match parts[2] {
                "SFTP" => Protocol::Sftp,
                "SCP" => Protocol::Scp,
                "FTPS Explicit" => Protocol::FtpsExplicit,
                "FTPS Implicit" => Protocol::FtpsImplicit,
                "FTP" => Protocol::Ftp,
                _ => Protocol::Sftp,
            };
            let host = parts[3].to_string();
            let port = parts[4].parse::<u16>().unwrap_or(protocol.default_port());
            let username = parts[5].to_string();
            let auth_type = parts[6].to_string();
            let env_color = u32::from_str_radix(parts[7].trim_start_matches("0x"), 16)
                .unwrap_or(0x2563eb);
            let max_connections = parts.get(8).and_then(|p| p.parse::<usize>().ok()).unwrap_or(2).clamp(1, 10);

            let password = read_password_from_keychain(&host, &username);
            sites.push(SavedSiteProfile {
                name,
                folder,
                protocol,
                host,
                port,
                username,
                password,
                auth_type,
                env_color,
                max_connections,
            });
        }
    }

    if sites.is_empty() {
        None
    } else {
        sites.sort_by(|a, b| {
            a.display_name().to_lowercase().cmp(&b.display_name().to_lowercase())
                .then_with(|| a.host.to_lowercase().cmp(&b.host.to_lowercase()))
                .then_with(|| a.username.to_lowercase().cmp(&b.username.to_lowercase()))
                .then_with(|| a.port.cmp(&b.port))
        });
        Some(sites)
    }
}

/// Saves sites to `~/.config/ruster/sites.conf`.
pub fn save_sites_to_disk(sites: &[SavedSiteProfile]) -> std::io::Result<()> {
    let dir = config_dir();
    fs::create_dir_all(&dir)?;
    let mut out = String::from("# Ruster Saved Connections\n");
    for s in sites {
        out.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t0x{:06x}\t{}\n",
            s.name.replace('\t', " "),
            s.folder.replace('\t', " "),
            s.protocol.label(),
            s.host.replace('\t', ""),
            s.port,
            s.username.replace('\t', ""),
            s.auth_type.replace('\t', " "),
            s.env_color,
            s.max_connections.clamp(1, 10),
        ));
    }
    fs::write(sites_file_path(), out)
}

#[derive(Clone, Debug, PartialEq)]
pub enum SshImportResult {
    Success(Vec<SavedSiteProfile>),
    FileNotFound,
    NoHostsFound,
}

/// Parses SSH config string content into a list of saved site profiles.
pub fn parse_ssh_config_str(content: &str) -> Vec<SavedSiteProfile> {
    let default_user = std::env::var("USER").unwrap_or_else(|_| "root".to_string());
    let mut imported = Vec::new();

    let mut current_alias: Option<String> = None;
    let mut hostname: Option<String> = None;
    let mut user: Option<String> = None;
    let mut port: Option<u16> = None;
    let mut identity_file: Option<String> = None;

    let flush_host = |alias: Option<String>,
                      hostname: Option<String>,
                      user: Option<String>,
                      port: Option<u16>,
                      identity_file: Option<String>,
                      out: &mut Vec<SavedSiteProfile>| {
        if let Some(alias) = alias {
            if alias.contains('*') || alias.contains('?') {
                return;
            }
            let host = hostname.unwrap_or_else(|| alias.clone());
            let username = user.unwrap_or_else(|| default_user.clone());
            let port = port.unwrap_or(22);
            let auth_type = if let Some(id) = identity_file {
                format!("SSH Key ({})", id)
            } else {
                "SSH Agent / Key".to_string()
            };
            out.push(SavedSiteProfile {
                name: alias,
                folder: "~/.ssh/config".to_string(),
                protocol: Protocol::Sftp,
                host,
                port,
                username,
                password: None,
                auth_type,
                env_color: 0x059669,
                max_connections: 2,
            });
        }
    };

    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut tokens = line.split_whitespace();
        let Some(key) = tokens.next() else {
            continue;
        };
        let val = tokens.collect::<Vec<_>>().join(" ");
        if val.is_empty() {
            continue;
        }

        match key.to_lowercase().as_str() {
            "host" => {
                flush_host(
                    current_alias.take(),
                    hostname.take(),
                    user.take(),
                    port.take(),
                    identity_file.take(),
                    &mut imported,
                );
                current_alias = Some(val.split_whitespace().next().unwrap_or(&val).to_string());
            }
            "hostname" => {
                hostname = Some(val);
            }
            "user" => {
                user = Some(val);
            }
            "port" => {
                port = val.parse::<u16>().ok();
            }
            "identityfile" => {
                identity_file = Some(val);
            }
            _ => {}
        }
    }

    flush_host(
        current_alias.take(),
        hostname.take(),
        user.take(),
        port.take(),
        identity_file.take(),
        &mut imported,
    );

    imported
}

/// Parses `~/.ssh/config` and returns discovered SSH/SFTP hosts or an import status.
pub fn import_from_ssh_config() -> SshImportResult {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/Users".to_string());
    let ssh_config_path = PathBuf::from(&home).join(".ssh").join("config");
    if !ssh_config_path.exists() {
        return SshImportResult::FileNotFound;
    }
    let content = match fs::read_to_string(&ssh_config_path) {
        Ok(c) => c,
        Err(_) => return SshImportResult::FileNotFound,
    };

    let imported = parse_ssh_config_str(&content);
    if imported.is_empty() {
        SshImportResult::NoHostsFound
    } else {
        SshImportResult::Success(imported)
    }
}

/// Saves a password into the native macOS Keychain (`security add-generic-password`).
pub fn save_password_to_keychain(host: &str, username: &str, password: &str) -> bool {
    if password.is_empty() {
        return false;
    }
    let service = format!("Ruster ({})", host);
    Command::new("/usr/bin/security")
        .args([
            "add-generic-password",
            "-U",
            "-s",
            &service,
            "-a",
            username,
            "-w",
            password,
        ])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Reads a password from the native macOS Keychain (`security find-generic-password`).
pub fn read_password_from_keychain(host: &str, username: &str) -> Option<String> {
    let service = format!("Ruster ({})", host);
    let output = Command::new("/usr/bin/security")
        .args([
            "find-generic-password",
            "-s",
            &service,
            "-a",
            username,
            "-w",
        ])
        .output()
        .ok()?;
    if output.status.success() {
        let pwd = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !pwd.is_empty() {
            return Some(pwd);
        }
    }

    // Fallback to legacy keychain service name
    let legacy_service = format!("MacSCP ({})", host);
    let legacy_output = Command::new("/usr/bin/security")
        .args([
            "find-generic-password",
            "-s",
            &legacy_service,
            "-a",
            username,
            "-w",
        ])
        .output()
        .ok()?;
    if legacy_output.status.success() {
        let pwd = String::from_utf8_lossy(&legacy_output.stdout).trim().to_string();
        if !pwd.is_empty() {
            return Some(pwd);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ssh_config_str() {
        let sample = r#"
# Global configuration (should be ignored)
Host *
    ServerAliveInterval 60

Host staging-server
    HostName 10.0.0.15
    User deployer
    Port 2222
    IdentityFile ~/.ssh/id_ed25519

Host prod-app
    HostName app.production.net
    User ubuntu
"#;
        let sites = parse_ssh_config_str(sample);
        assert_eq!(sites.len(), 2);

        assert_eq!(sites[0].name, "staging-server");
        assert_eq!(sites[0].host, "10.0.0.15");
        assert_eq!(sites[0].username, "deployer");
        assert_eq!(sites[0].port, 2222);
        assert_eq!(sites[0].auth_type, "SSH Key (~/.ssh/id_ed25519)");

        assert_eq!(sites[1].name, "prod-app");
        assert_eq!(sites[1].host, "app.production.net");
        assert_eq!(sites[1].username, "ubuntu");
        assert_eq!(sites[1].port, 22);
        assert_eq!(sites[1].auth_type, "SSH Agent / Key");
    }

    #[test]
    fn test_parse_ssh_config_empty_or_wildcards() {
        let empty = "";
        assert!(parse_ssh_config_str(empty).is_empty());

        let wildcards = "Host *\n    User test\nHost ?\n    Port 22\n";
        assert!(parse_ssh_config_str(wildcards).is_empty());
    }

    #[test]
    fn test_app_state_permissions_and_date_format() {
        let sample_state = "show_permissions=true\ndate_format=date_time\n";
        let mut show_permissions = false;
        let mut date_format = crate::model::DateFormat::Date;
        for line in sample_state.lines() {
            if let Some((k, v)) = line.split_once('=') {
                match k.trim() {
                    "show_permissions" => {
                        if let Ok(b) = v.trim().parse::<bool>() {
                            show_permissions = b;
                        }
                    }
                    "date_format" => {
                        date_format = crate::model::DateFormat::from_str(v.trim());
                    }
                    _ => {}
                }
            }
        }
        assert_eq!(show_permissions, true);
        assert_eq!(date_format, crate::model::DateFormat::DateTime);

        // Test default state: permissions is false (hidden), date_format is Date
        let default_state = "";
        let mut def_show_permissions = false;
        let mut def_date_format = crate::model::DateFormat::Date;
        for line in default_state.lines() {
            if let Some((k, v)) = line.split_once('=') {
                match k.trim() {
                    "show_permissions" => {
                        if let Ok(b) = v.trim().parse::<bool>() {
                            def_show_permissions = b;
                        }
                    }
                    "date_format" => {
                        def_date_format = crate::model::DateFormat::from_str(v.trim());
                    }
                    _ => {}
                }
            }
        }
        assert_eq!(def_show_permissions, false);
        assert_eq!(def_date_format, crate::model::DateFormat::Date);
    }

    #[test]
    fn test_favorites_serialization() {
        let temp = std::env::temp_dir().join(format!("ruster_fav_test_{}", std::process::id()));
        let paths = vec![
            PathBuf::from("/Users/test"),
            PathBuf::from("/Users/test/Documents"),
            PathBuf::from("/Users/test/Downloads"),
        ];
        let mut out = String::from("# Ruster Favorite Locations\n");
        for p in &paths {
            out.push_str(&p.to_string_lossy());
            out.push('\n');
        }
        let _ = fs::write(&temp, out);

        let content = fs::read_to_string(&temp).unwrap();
        let mut loaded = Vec::new();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            loaded.push(PathBuf::from(line));
        }
        assert_eq!(loaded, paths);

        // Test delete
        loaded.retain(|p| p != &PathBuf::from("/Users/test/Documents"));
        assert_eq!(loaded.len(), 2);
        assert!(!loaded.contains(&PathBuf::from("/Users/test/Documents")));

        let _ = fs::remove_file(&temp);
    }
}
