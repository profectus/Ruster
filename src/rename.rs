use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use regex::Regex;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CaseTransform {
    None,
    Lower,
    Upper,
    Title,
}

impl CaseTransform {
    pub fn label(&self) -> &'static str {
        match self {
            CaseTransform::None => "Ponechat (Leave as is)",
            CaseTransform::Lower => "malá písmena (lowercase)",
            CaseTransform::Upper => "VELKÁ PÍSMENA (UPPERCASE)",
            CaseTransform::Title => "První Velká (Title Case)",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ExtTransform {
    Keep,
    Lower,
    Upper,
}

impl ExtTransform {
    pub fn label(&self) -> &'static str {
        match self {
            ExtTransform::Keep => "Ponechat (Keep)",
            ExtTransform::Lower => ".ext (lowercase)",
            ExtTransform::Upper => ".EXT (UPPERCASE)",
        }
    }
}

#[derive(Clone, Debug)]
pub struct RenamePreview {
    pub original_name: String,
    pub new_name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub has_change: bool,
    pub has_collision: bool,
}

#[derive(Clone, Debug)]
pub struct MultiRenameConfig {
    pub find_pattern: String,
    pub replace_pattern: String,
    pub use_regex: bool,
    pub case_sensitive: bool,
    pub counter_start: u32,
    pub counter_step: u32,
    pub counter_digits: usize,
    pub case_transform: CaseTransform,
    pub ext_transform: ExtTransform,
}

impl Default for MultiRenameConfig {
    fn default() -> Self {
        Self {
            find_pattern: String::new(),
            replace_pattern: String::new(),
            use_regex: false,
            case_sensitive: false,
            counter_start: 1,
            counter_step: 1,
            counter_digits: 2,
            case_transform: CaseTransform::None,
            ext_transform: ExtTransform::Keep,
        }
    }
}

pub fn compute_rename_previews(
    items: &[(PathBuf, String, bool)],
    config: &MultiRenameConfig,
) -> Vec<RenamePreview> {
    let mut results = Vec::with_capacity(items.len());
    let mut seen_names = HashSet::new();

    let compiled_regex = if config.use_regex && !config.find_pattern.is_empty() {
        let pattern = if config.case_sensitive {
            config.find_pattern.clone()
        } else {
            format!("(?i){}", config.find_pattern)
        };
        Regex::new(&pattern).ok()
    } else {
        None
    };

    let mut current_counter = config.counter_start;

    for (idx, (path, orig_name, is_dir)) in items.iter().enumerate() {
        let (stem, ext) = split_stem_ext(orig_name, *is_dir);

        let counter_str = format!("{:0width$}", current_counter, width = config.counter_digits);

        // 1. Process stem with find & replace
        let mut new_stem = if config.find_pattern.is_empty() {
            if config.replace_pattern.is_empty() {
                stem.to_string()
            } else {
                expand_placeholders(&config.replace_pattern, stem, ext, &counter_str, idx + 1)
            }
        } else if let Some(ref re) = compiled_regex {
            let replaced_tmpl = expand_placeholders(&config.replace_pattern, stem, ext, &counter_str, idx + 1);
            re.replace_all(stem, replaced_tmpl.as_str()).to_string()
        } else {
            let expanded_replace = expand_placeholders(&config.replace_pattern, stem, ext, &counter_str, idx + 1);
            if config.case_sensitive {
                stem.replace(&config.find_pattern, &expanded_replace)
            } else {
                replace_case_insensitive(stem, &config.find_pattern, &expanded_replace)
            }
        };

        // 2. Apply case transformation to stem
        new_stem = match config.case_transform {
            CaseTransform::None => new_stem,
            CaseTransform::Lower => new_stem.to_lowercase(),
            CaseTransform::Upper => new_stem.to_uppercase(),
            CaseTransform::Title => to_title_case(&new_stem),
        };

        // 3. Apply extension transformation
        let new_ext = match config.ext_transform {
            ExtTransform::Keep => ext.to_string(),
            ExtTransform::Lower => ext.to_lowercase(),
            ExtTransform::Upper => ext.to_uppercase(),
        };

        let new_name = if new_ext.is_empty() {
            new_stem
        } else {
            format!("{}.{}", new_stem, new_ext)
        };

        let has_change = &new_name != orig_name;
        let is_duplicate = seen_names.contains(&new_name.to_lowercase());
        seen_names.insert(new_name.to_lowercase());

        results.push(RenamePreview {
            original_name: orig_name.clone(),
            new_name,
            path: path.clone(),
            is_dir: *is_dir,
            has_change,
            has_collision: is_duplicate,
        });

        current_counter = current_counter.saturating_add(config.counter_step);
    }

    results
}

fn split_stem_ext(name: &str, is_dir: bool) -> (&str, &str) {
    if is_dir {
        return (name, "");
    }
    match name.rfind('.') {
        Some(dot_idx) if dot_idx > 0 && dot_idx < name.len() - 1 => {
            (&name[..dot_idx], &name[dot_idx + 1..])
        }
        _ => (name, ""),
    }
}

fn expand_placeholders(
    template: &str,
    orig_stem: &str,
    orig_ext: &str,
    counter_str: &str,
    _index_num: usize,
) -> String {
    template
        .replace("[N]", orig_stem)
        .replace("[n]", orig_stem)
        .replace("[E]", orig_ext)
        .replace("[e]", orig_ext)
        .replace("[C]", counter_str)
        .replace("[c]", counter_str)
}

fn replace_case_insensitive(haystack: &str, needle: &str, replacement: &str) -> String {
    let lower_h = haystack.to_lowercase();
    let lower_n = needle.to_lowercase();

    let mut result = String::new();
    let mut last_idx = 0;

    for (start_idx, _) in lower_h.match_indices(&lower_n) {
        result.push_str(&haystack[last_idx..start_idx]);
        result.push_str(replacement);
        last_idx = start_idx + needle.len();
    }
    result.push_str(&haystack[last_idx..]);
    result
}

fn to_title_case(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut capitalize_next = true;

    for c in s.chars() {
        if c.is_alphanumeric() {
            if capitalize_next {
                result.extend(c.to_uppercase());
                capitalize_next = false;
            } else {
                result.extend(c.to_lowercase());
            }
        } else {
            result.push(c);
            capitalize_next = true;
        }
    }
    result
}

pub fn execute_local_renames(previews: &[RenamePreview]) -> Result<usize, String> {
    let mut renamed_count = 0;

    // To prevent conflicts when A -> B and B -> C, rename to temp files first if needed
    // or perform direct rename
    for p in previews {
        if !p.has_change || p.has_collision {
            continue;
        }
        let target_path = p.path.with_file_name(&p.new_name);
        fs::rename(&p.path, &target_path)
            .map_err(|e| format!("Failed to rename '{}' to '{}': {}", p.original_name, p.new_name, e))?;
        renamed_count += 1;
    }

    Ok(renamed_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rename_counter_and_replace() {
        let items = vec![
            (PathBuf::from("/tmp/photo_a.jpg"), "photo_a.jpg".to_string(), false),
            (PathBuf::from("/tmp/photo_b.jpg"), "photo_b.jpg".to_string(), false),
        ];

        let config = MultiRenameConfig {
            find_pattern: "photo_".to_string(),
            replace_pattern: "vacation_[C]_".to_string(),
            counter_start: 1,
            counter_step: 1,
            counter_digits: 2,
            ..Default::default()
        };

        let previews = compute_rename_previews(&items, &config);
        assert_eq!(previews.len(), 2);
        assert_eq!(previews[0].new_name, "vacation_01_a.jpg");
        assert_eq!(previews[1].new_name, "vacation_02_b.jpg");
    }

    #[test]
    fn test_case_transform() {
        let items = vec![
            (PathBuf::from("/tmp/my_file.PNG"), "my_file.PNG".to_string(), false),
        ];

        let config = MultiRenameConfig {
            case_transform: CaseTransform::Upper,
            ext_transform: ExtTransform::Lower,
            ..Default::default()
        };

        let previews = compute_rename_previews(&items, &config);
        assert_eq!(previews[0].new_name, "MY_FILE.png");
    }
}
