use std::path::PathBuf;
use std::process::Command;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub release_url: String,
    pub release_notes: String,
}

#[derive(Clone, Debug)]
pub enum UpdateStatus {
    Checking,
    Available(UpdateInfo),
    Updating { version: String, message: String },
    ReadyToRestart { version: String },
    UpToDate(String),
    Error(String),
}

/// Compares semver version strings like "0.2.0" and "0.1.0" or "v1.0.0" and "0.9.5".
/// Returns true if `latest` is strictly newer than `current`.
pub fn is_newer_version(latest: &str, current: &str) -> bool {
    let parse_parts = |v: &str| -> Vec<u32> {
        let clean = v.trim().trim_start_matches('v').trim_start_matches('V');
        clean
            .split('.')
            .map(|part| {
                // Take only leading numeric digits if suffixes like -beta or -rc exist
                let num_str: String = part.chars().take_while(|c| c.is_ascii_digit()).collect();
                num_str.parse::<u32>().unwrap_or(0)
            })
            .collect()
    };

    let latest_parts = parse_parts(latest);
    let current_parts = parse_parts(current);

    let max_len = latest_parts.len().max(current_parts.len());
    for i in 0..max_len {
        let l = latest_parts.get(i).copied().unwrap_or(0);
        let c = current_parts.get(i).copied().unwrap_or(0);
        if l > c {
            return true;
        } else if l < c {
            return false;
        }
    }
    false
}

/// Checks GitHub Releases API for the latest version of MacSCP.
/// Uses system `curl` to avoid extra HTTP dependencies.
pub fn fetch_latest_release() -> Result<Option<UpdateInfo>, String> {
    let current_version = env!("CARGO_PKG_VERSION").to_string();

    let output = Command::new("curl")
        .args([
            "-s",
            "--max-time",
            "6",
            "-H",
            "Accept: application/vnd.github.v3+json",
            "-H",
            "User-Agent: MacSCP-App",
            "https://api.github.com/repos/profectus/macSCP/releases/latest",
        ])
        .output()
        .map_err(|e| format!("Failed to execute curl: {e}"))?;

    if !output.status.success() {
        return Err("Curl request failed".to_string());
    }

    let body = String::from_utf8_lossy(&output.stdout);
    if body.trim().is_empty() || body.contains("\"message\": \"Not Found\"") {
        return Ok(None);
    }

    // Simple JSON field extraction without heavy json parser dependencies
    let tag = extract_json_string(&body, "tag_name").unwrap_or_default();
    let html_url = extract_json_string(&body, "html_url")
        .unwrap_or_else(|| "https://github.com/profectus/macSCP/releases/latest".to_string());
    let release_notes = extract_json_string(&body, "body").unwrap_or_default();

    if tag.is_empty() {
        return Ok(None);
    }

    let clean_tag = tag.trim_start_matches('v').trim_start_matches('V').to_string();

    if is_newer_version(&clean_tag, &current_version) {
        Ok(Some(UpdateInfo {
            current_version,
            latest_version: clean_tag,
            release_url: html_url,
            release_notes,
        }))
    } else {
        Ok(None)
    }
}

fn extract_json_string(json: &str, field: &str) -> Option<String> {
    let needle = format!("\"{field}\":");
    let pos = json.find(&needle)?;
    let rest = &json[pos + needle.len()..];
    let start_quote = rest.find('"')?;
    let content_start = &rest[start_quote + 1..];

    let mut result = String::new();
    let mut escaped = false;
    for c in content_start.chars() {
        if escaped {
            match c {
                'n' => result.push('\n'),
                'r' => result.push('\r'),
                't' => result.push('\t'),
                '\"' => result.push('\"'),
                '\\' => result.push('\\'),
                other => result.push(other),
            }
            escaped = false;
        } else if c == '\\' {
            escaped = true;
        } else if c == '"' {
            return Some(result);
        } else {
            result.push(c);
        }
    }
    None
}

pub fn find_brew_binary() -> Option<PathBuf> {
    for p in &["/opt/homebrew/bin/brew", "/usr/local/bin/brew"] {
        let path = PathBuf::from(p);
        if path.exists() {
            return Some(path);
        }
    }
    if let Ok(output) = Command::new("which").arg("brew").output() {
        if output.status.success() {
            let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !s.is_empty() {
                return Some(PathBuf::from(s));
            }
        }
    }
    None
}

pub fn locate_target_app() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        let mut curr = exe.as_path();
        while let Some(parent) = curr.parent() {
            if curr.extension().and_then(|e| e.to_str()) == Some("app") {
                return curr.to_path_buf();
            }
            curr = parent;
        }
    }
    PathBuf::from("/Applications/MacSCP.app")
}

pub fn restart_app() {
    let target_app = locate_target_app();
    let script = format!(
        "sleep 0.5 && /usr/bin/open -n \"{}\"",
        target_app.to_string_lossy()
    );
    let _ = Command::new("/bin/sh")
        .args(["-c", &script])
        .spawn();

    std::process::exit(0);
}

pub fn perform_in_app_update(version: &str, mut on_progress: impl FnMut(String)) -> Result<(), String> {
    on_progress("Ověřuji instalaci...".to_string());

    // 1. If Homebrew is managing macscp, try upgrading through brew first
    if let Some(brew) = find_brew_binary() {
        let is_cask = Command::new(&brew)
            .args(["list", "--cask", "macscp"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if is_cask {
            on_progress("Aktualizuji repozitář Homebrew...".to_string());
            let _ = Command::new(&brew).arg("update").output();

            on_progress("Instaluji novou verzi přes Homebrew...".to_string());
            let res = Command::new(&brew)
                .args(["upgrade", "--cask", "--no-ask", "macscp"])
                .output();

            if let Ok(out) = res {
                if out.status.success() {
                    let _ = Command::new("/usr/bin/xattr")
                        .args(["-dr", "com.apple.quarantine", "/Applications/MacSCP.app"])
                        .output();
                    return Ok(());
                }
            }
        }
    }

    // 2. Direct download from GitHub releases
    on_progress(format!("Stahuji balíček v{} z GitHubu...", version));
    let tmp_dir = PathBuf::from("/tmp/macscp_update");
    let _ = std::fs::remove_dir_all(&tmp_dir);
    std::fs::create_dir_all(&tmp_dir).map_err(|e| format!("Nelze vytvořit dočasnou složku: {e}"))?;

    let zip_path = tmp_dir.join("MacSCP.zip");
    let download_url = format!("https://github.com/profectus/macSCP/releases/download/v{version}/MacSCP.zip");

    let curl_res = Command::new("/usr/bin/curl")
        .args([
            "-L",
            "--fail",
            "-s",
            "--max-time", "120",
            "-o", zip_path.to_str().unwrap(),
            &download_url,
        ])
        .output()
        .map_err(|e| format!("Chyba při stahování: {e}"))?;

    if !curl_res.status.success() {
        return Err("Stažení nové verze se nezdařilo".to_string());
    }

    on_progress("Rozbaluji novou verzi...".to_string());
    let extract_dir = tmp_dir.join("extracted");
    let _ = std::fs::create_dir_all(&extract_dir);

    let ditto_res = Command::new("/usr/bin/ditto")
        .args([
            "-xk",
            zip_path.to_str().unwrap(),
            extract_dir.to_str().unwrap(),
        ])
        .output()
        .map_err(|e| format!("Chyba při rozbalování: {e}"))?;

    if !ditto_res.status.success() {
        return Err("Nepodařilo se rozbalit archiv aplikace".to_string());
    }

    let new_app = extract_dir.join("MacSCP.app");
    if !new_app.exists() {
        return Err("V archivu nebyl nalezen soubor MacSCP.app".to_string());
    }

    // Remove Gatekeeper quarantine
    let _ = Command::new("/usr/bin/xattr")
        .args(["-dr", "com.apple.quarantine", new_app.to_str().unwrap()])
        .output();

    on_progress("Instaluji novou verzi do /Applications...".to_string());
    let target_app = locate_target_app();

    let _ = Command::new("/bin/rm")
        .args(["-rf", target_app.to_str().unwrap()])
        .output();

    let cp_res = Command::new("/bin/cp")
        .args([
            "-R",
            new_app.to_str().unwrap(),
            target_app.to_str().unwrap(),
        ])
        .output()
        .map_err(|e| format!("Chyba při kopírování: {e}"))?;

    if !cp_res.status.success() {
        return Err("Nepodařilo se přepsat stávající aplikaci".to_string());
    }

    let _ = Command::new("/usr/bin/xattr")
        .args(["-dr", "com.apple.quarantine", target_app.to_str().unwrap()])
        .output();

    let _ = std::fs::remove_dir_all(&tmp_dir);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparisons() {
        assert!(is_newer_version("0.2.0", "0.1.0"));
        assert!(is_newer_version("v0.2.0", "0.1.0"));
        assert!(is_newer_version("v1.0.0", "0.9.9"));
        assert!(is_newer_version("0.1.1", "0.1.0"));
        assert!(!is_newer_version("0.1.0", "0.1.0"));
        assert!(!is_newer_version("v0.1.0", "0.1.0"));
        assert!(!is_newer_version("0.0.9", "0.1.0"));
        assert!(!is_newer_version("0.0.1", "0.1.0"));
        assert!(is_newer_version("1.0.0-rc1", "0.9.0"));
    }

    #[test]
    fn test_json_extractor() {
        let sample = r#"{
            "tag_name": "v0.2.0",
            "html_url": "https://github.com/profectus/macSCP/releases/tag/v0.2.0",
            "body": "Awesome new release\nBug fixes"
        }"#;

        assert_eq!(extract_json_string(sample, "tag_name"), Some("v0.2.0".to_string()));
        assert_eq!(
            extract_json_string(sample, "html_url"),
            Some("https://github.com/profectus/macSCP/releases/tag/v0.2.0".to_string())
        );
        assert!(extract_json_string(sample, "body").unwrap().contains("Awesome new release"));
    }
}
