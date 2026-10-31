use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DiffLineType {
    Unchanged,
    Added,
    Deleted,
    Modified,
}

#[derive(Clone, Debug)]
pub struct DiffLine {
    pub left_line_num: Option<usize>,
    pub right_line_num: Option<usize>,
    pub left_text: String,
    pub right_text: String,
    pub line_type: DiffLineType,
}

#[derive(Clone, Debug)]
pub struct FileComparison {
    pub file_a_path: PathBuf,
    pub file_b_path: PathBuf,
    pub file_a_name: String,
    pub file_b_name: String,
    pub file_a_size: u64,
    pub file_b_size: u64,
    pub is_binary: bool,
    pub are_identical: bool,
    pub diff_lines: Vec<DiffLine>,
    pub difference_count: usize,
}

pub fn compare_files(path_a: &Path, path_b: &Path) -> Result<FileComparison, String> {
    if !path_a.exists() {
        return Err(format!("File not found: {}", path_a.display()));
    }
    if !path_b.exists() {
        return Err(format!("File not found: {}", path_b.display()));
    }

    let meta_a = std::fs::metadata(path_a).map_err(|e| e.to_string())?;
    let meta_b = std::fs::metadata(path_b).map_err(|e| e.to_string())?;

    let size_a = meta_a.len();
    let size_b = meta_b.len();

    let name_a = path_a.file_name().unwrap_or_default().to_string_lossy().to_string();
    let name_b = path_b.file_name().unwrap_or_default().to_string_lossy().to_string();

    let is_binary_a = check_is_binary(path_a)?;
    let is_binary_b = check_is_binary(path_b)?;
    let is_binary = is_binary_a || is_binary_b;

    if is_binary {
        let are_identical = size_a == size_b && files_equal_bytes(path_a, path_b)?;
        return Ok(FileComparison {
            file_a_path: path_a.to_path_buf(),
            file_b_path: path_b.to_path_buf(),
            file_a_name: name_a,
            file_b_name: name_b,
            file_a_size: size_a,
            file_b_size: size_b,
            is_binary: true,
            are_identical,
            diff_lines: Vec::new(),
            difference_count: if are_identical { 0 } else { 1 },
        });
    }

    // Text comparison
    let lines_a = read_file_lines(path_a)?;
    let lines_b = read_file_lines(path_b)?;

    let are_identical = lines_a == lines_b;
    let diff_lines = compute_line_diff(&lines_a, &lines_b);
    let difference_count = diff_lines
        .iter()
        .filter(|l| l.line_type != DiffLineType::Unchanged)
        .count();

    Ok(FileComparison {
        file_a_path: path_a.to_path_buf(),
        file_b_path: path_b.to_path_buf(),
        file_a_name: name_a,
        file_b_name: name_b,
        file_a_size: size_a,
        file_b_size: size_b,
        is_binary: false,
        are_identical,
        diff_lines,
        difference_count,
    })
}

fn check_is_binary(path: &Path) -> Result<bool, String> {
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let mut buffer = [0u8; 4096];
    let n = file.read(&mut buffer).map_err(|e| e.to_string())?;
    Ok(buffer[..n].contains(&0))
}

fn files_equal_bytes(path_a: &Path, path_b: &Path) -> Result<bool, String> {
    let mut f_a = File::open(path_a).map_err(|e| e.to_string())?;
    let mut f_b = File::open(path_b).map_err(|e| e.to_string())?;

    let mut buf_a = [0u8; 65536];
    let mut buf_b = [0u8; 65536];

    loop {
        let n_a = f_a.read(&mut buf_a).map_err(|e| e.to_string())?;
        let n_b = f_b.read(&mut buf_b).map_err(|e| e.to_string())?;

        if n_a != n_b || buf_a[..n_a] != buf_b[..n_b] {
            return Ok(false);
        }
        if n_a == 0 {
            return Ok(true);
        }
    }
}

fn read_file_lines(path: &Path) -> Result<Vec<String>, String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);
    let mut lines = Vec::new();
    for line in reader.lines() {
        lines.push(line.map_err(|e| e.to_string())?);
    }
    Ok(lines)
}

/// Computes a side-by-side / unified line diff using Longest Common Subsequence (LCS)
pub fn compute_line_diff(lines_a: &[String], lines_b: &[String]) -> Vec<DiffLine> {
    let n = lines_a.len();
    let m = lines_b.len();

    // To prevent high memory usage on massive files, limit LCS table size
    if n * m > 2_000_000 {
        return fallback_fast_diff(lines_a, lines_b);
    }

    // Standard DP table for LCS length
    let mut dp = vec![vec![0u32; m + 1]; n + 1];
    for i in 0..n {
        for j in 0..m {
            if lines_a[i] == lines_b[j] {
                dp[i + 1][j + 1] = dp[i][j] + 1;
            } else {
                dp[i + 1][j + 1] = dp[i + 1][j].max(dp[i][j + 1]);
            }
        }
    }

    // Backtrack to build diff ops
    let mut ops = Vec::new();
    let mut i = n;
    let mut j = m;

    while i > 0 || j > 0 {
        if i > 0 && j > 0 && lines_a[i - 1] == lines_b[j - 1] {
            ops.push((Some(i - 1), Some(j - 1), DiffLineType::Unchanged));
            i -= 1;
            j -= 1;
        } else if j > 0 && (i == 0 || dp[i][j - 1] >= dp[i - 1][j]) {
            ops.push((None, Some(j - 1), DiffLineType::Added));
            j -= 1;
        } else if i > 0 {
            ops.push((Some(i - 1), None, DiffLineType::Deleted));
            i -= 1;
        }
    }

    ops.reverse();

    // Convert ops into formatted DiffLines, grouping contiguous Deleted + Added into Modified
    let mut result = Vec::new();
    let mut idx = 0;

    while idx < ops.len() {
        if ops[idx].2 == DiffLineType::Deleted && idx + 1 < ops.len() && ops[idx + 1].2 == DiffLineType::Added {
            let left_idx = ops[idx].0.unwrap();
            let right_idx = ops[idx + 1].1.unwrap();
            result.push(DiffLine {
                left_line_num: Some(left_idx + 1),
                right_line_num: Some(right_idx + 1),
                left_text: lines_a[left_idx].clone(),
                right_text: lines_b[right_idx].clone(),
                line_type: DiffLineType::Modified,
            });
            idx += 2;
        } else {
            let (l_opt, r_opt, l_type) = ops[idx];
            result.push(DiffLine {
                left_line_num: l_opt.map(|x| x + 1),
                right_line_num: r_opt.map(|x| x + 1),
                left_text: l_opt.map(|x| lines_a[x].clone()).unwrap_or_default(),
                right_text: r_opt.map(|x| lines_b[x].clone()).unwrap_or_default(),
                line_type: l_type,
            });
            idx += 1;
        }
    }

    result
}

fn fallback_fast_diff(lines_a: &[String], lines_b: &[String]) -> Vec<DiffLine> {
    let max_len = lines_a.len().max(lines_b.len());
    let mut result = Vec::with_capacity(max_len);

    for i in 0..max_len {
        let l_opt = lines_a.get(i);
        let r_opt = lines_b.get(i);
        match (l_opt, r_opt) {
            (Some(l), Some(r)) => {
                let l_type = if l == r {
                    DiffLineType::Unchanged
                } else {
                    DiffLineType::Modified
                };
                result.push(DiffLine {
                    left_line_num: Some(i + 1),
                    right_line_num: Some(i + 1),
                    left_text: l.clone(),
                    right_text: r.clone(),
                    line_type: l_type,
                });
            }
            (Some(l), None) => {
                result.push(DiffLine {
                    left_line_num: Some(i + 1),
                    right_line_num: None,
                    left_text: l.clone(),
                    right_text: String::new(),
                    line_type: DiffLineType::Deleted,
                });
            }
            (None, Some(r)) => {
                result.push(DiffLine {
                    left_line_num: None,
                    right_line_num: Some(i + 1),
                    left_text: String::new(),
                    right_text: r.clone(),
                    line_type: DiffLineType::Added,
                });
            }
            (None, None) => break,
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identical_lines() {
        let a = vec!["hello".to_string(), "world".to_string()];
        let b = vec!["hello".to_string(), "world".to_string()];
        let diff = compute_line_diff(&a, &b);
        assert_eq!(diff.len(), 2);
        assert!(diff.iter().all(|l| l.line_type == DiffLineType::Unchanged));
    }

    #[test]
    fn test_added_and_deleted_lines() {
        let a = vec!["one".to_string(), "two".to_string()];
        let b = vec!["one".to_string(), "two and half".to_string(), "three".to_string()];
        let diff = compute_line_diff(&a, &b);
        assert_eq!(diff.len(), 3);
        assert_eq!(diff[0].line_type, DiffLineType::Unchanged);
        assert_eq!(diff[1].line_type, DiffLineType::Modified);
        assert_eq!(diff[2].line_type, DiffLineType::Added);
    }
}
