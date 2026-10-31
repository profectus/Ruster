use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use regex::Regex;

#[derive(Clone, Debug)]
pub struct SearchResult {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub size: u64,
    pub modified_str: String,
    pub modified_ts: i64,
}

#[derive(Clone, Debug)]
pub struct SearchQuery {
    pub root_dir: PathBuf,
    pub name_pattern: String,
    pub containing_text: String,
    pub case_sensitive: bool,
    pub use_regex: bool,
    pub recursive: bool,
    pub min_size_bytes: Option<u64>,
    pub max_size_bytes: Option<u64>,
    pub modified_within_days: Option<u32>,
}

impl Default for SearchQuery {
    fn default() -> Self {
        Self {
            root_dir: PathBuf::from("."),
            name_pattern: "*".to_string(),
            containing_text: String::new(),
            case_sensitive: false,
            use_regex: false,
            recursive: true,
            min_size_bytes: None,
            max_size_bytes: None,
            modified_within_days: None,
        }
    }
}

pub fn execute_search(
    query: SearchQuery,
    cancel_flag: Arc<AtomicBool>,
    mut on_found: impl FnMut(SearchResult),
) {
    let compiled_name_regex = if query.use_regex && !query.name_pattern.is_empty() {
        let pattern = if query.case_sensitive {
            query.name_pattern.clone()
        } else {
            format!("(?i){}", query.name_pattern)
        };
        Regex::new(&pattern).ok()
    } else {
        None
    };

    let compiled_text_regex = if query.use_regex && !query.containing_text.is_empty() {
        let pattern = if query.case_sensitive {
            query.containing_text.clone()
        } else {
            format!("(?i){}", query.containing_text)
        };
        Regex::new(&pattern).ok()
    } else {
        None
    };

    let min_timestamp = query.modified_within_days.map(|days| {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let seconds = (days as u64) * 86400;
        now.saturating_sub(seconds)
    });

    search_dir_recursive(
        &query.root_dir,
        &query,
        &compiled_name_regex,
        &compiled_text_regex,
        min_timestamp,
        &cancel_flag,
        &mut on_found,
    );
}

fn search_dir_recursive(
    current_dir: &Path,
    query: &SearchQuery,
    name_regex: &Option<Regex>,
    text_regex: &Option<Regex>,
    min_timestamp: Option<u64>,
    cancel_flag: &AtomicBool,
    on_found: &mut impl FnMut(SearchResult),
) {
    if cancel_flag.load(Ordering::Relaxed) {
        return;
    }

    let entries = match fs::read_dir(current_dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    let mut subdirs = Vec::new();

    for entry in entries.flatten() {
        if cancel_flag.load(Ordering::Relaxed) {
            return;
        }

        let path = entry.path();
        let fname = entry.file_name();
        let name_str = fname.to_string_lossy();

        if name_str == "." || name_str == ".." {
            continue;
        }

        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        let is_dir = meta.is_dir();
        let size = if is_dir { 0 } else { meta.len() };

        let mtime_secs = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);

        // Filter: Date
        if let Some(min_ts) = min_timestamp {
            if mtime_secs < min_ts {
                if is_dir && query.recursive {
                    subdirs.push(path);
                }
                continue;
            }
        }

        // Filter: Size
        if !is_dir {
            if let Some(min_s) = query.min_size_bytes {
                if size < min_s {
                    continue;
                }
            }
            if let Some(max_s) = query.max_size_bytes {
                if size > max_s {
                    continue;
                }
            }
        }

        // Filter: Name pattern
        let matches_name = if let Some(re) = name_regex {
            re.is_match(&name_str)
        } else {
            wildcard_match(&name_str, &query.name_pattern, query.case_sensitive)
        };

        if matches_name {
            // Filter: Containing text (files only)
            let matches_text = if !is_dir && !query.containing_text.is_empty() {
                file_contains_text(&path, &query.containing_text, query.case_sensitive, text_regex)
            } else {
                query.containing_text.is_empty()
            };

            if matches_text {
                let mod_str = format_unix_timestamp(mtime_secs);
                on_found(SearchResult {
                    name: name_str.to_string(),
                    path: path.clone(),
                    is_dir,
                    size,
                    modified_str: mod_str,
                    modified_ts: mtime_secs as i64,
                });
            }
        }

        if is_dir && query.recursive {
            subdirs.push(path);
        }
    }

    if query.recursive {
        for dir in subdirs {
            search_dir_recursive(
                &dir,
                query,
                name_regex,
                text_regex,
                min_timestamp,
                cancel_flag,
                on_found,
            );
        }
    }
}

pub fn wildcard_match(text: &str, pattern: &str, case_sensitive: bool) -> bool {
    let p = pattern.trim();
    if p.is_empty() || p == "*" || p == "*.*" {
        return true;
    }

    let (t, p) = if case_sensitive {
        (text.to_string(), p.to_string())
    } else {
        (text.to_lowercase(), p.to_lowercase())
    };

    let t_chars: Vec<char> = t.chars().collect();
    let p_chars: Vec<char> = p.chars().collect();

    let mut t_idx = 0;
    let mut p_idx = 0;
    let mut star_idx = None;
    let mut match_idx = 0;

    while t_idx < t_chars.len() {
        if p_idx < p_chars.len() && (p_chars[p_idx] == '?' || p_chars[p_idx] == t_chars[t_idx]) {
            t_idx += 1;
            p_idx += 1;
        } else if p_idx < p_chars.len() && p_chars[p_idx] == '*' {
            star_idx = Some(p_idx);
            match_idx = t_idx;
            p_idx += 1;
        } else if let Some(s_idx) = star_idx {
            p_idx = s_idx + 1;
            match_idx += 1;
            t_idx = match_idx;
        } else {
            return false;
        }
    }

    while p_idx < p_chars.len() && p_chars[p_idx] == '*' {
        p_idx += 1;
    }

    p_idx == p_chars.len()
}

fn file_contains_text(
    path: &Path,
    needle: &str,
    case_sensitive: bool,
    re: &Option<Regex>,
) -> bool {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return false,
    };

    let mut reader = BufReader::new(file);

    // Skip binary files if null bytes in first 1024 bytes
    let mut header = [0u8; 1024];
    if let Ok(n) = reader.read(&mut header) {
        if header[..n].contains(&0) {
            return false;
        }
    }
    // Reopen for scanning
    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return false,
    };
    let reader = BufReader::new(file);

    let lower_needle = if case_sensitive { String::new() } else { needle.to_lowercase() };

    for line_res in reader.lines() {
        if let Ok(line) = line_res {
            if let Some(regex) = re {
                if regex.is_match(&line) {
                    return true;
                }
            } else if case_sensitive {
                if line.contains(needle) {
                    return true;
                }
            } else if line.to_lowercase().contains(&lower_needle) {
                return true;
            }
        }
    }

    false
}

fn format_unix_timestamp(ts: u64) -> String {
    if ts == 0 {
        return "-".to_string();
    }
    unsafe {
        let mut tm: libc::tm = std::mem::zeroed();
        let time_val = ts as libc::time_t;
        libc::localtime_r(&time_val, &mut tm);
        format!(
            "{:04}-{:02}-{:02} {:02}:{:02}",
            tm.tm_year + 1900,
            tm.tm_mon + 1,
            tm.tm_mday,
            tm.tm_hour,
            tm.tm_min
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wildcard_matching() {
        assert!(wildcard_match("hello.txt", "*.txt", false));
        assert!(wildcard_match("HELLO.TXT", "*.txt", false));
        assert!(!wildcard_match("hello.png", "*.txt", false));
        assert!(wildcard_match("document_2026.pdf", "doc*2026.???", false));
        assert!(wildcard_match("file.rs", "*", false));
    }
}
