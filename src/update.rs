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
