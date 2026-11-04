use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::SystemTime;

#[derive(Clone, Debug)]
pub struct DiskUsageNode {
    pub id: usize,
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub size: u64,
    pub files_count: usize,
    pub dirs_count: usize,
    pub modified_str: String,
    pub children: Vec<DiskUsageNode>,
    pub is_expanded: bool,
}

#[derive(Clone, Debug)]
pub struct ExtensionUsage {
    pub extension: String,
    pub category: String,
    pub size: u64,
    pub percentage: f32,
    pub count: usize,
}

#[derive(Clone, Debug)]
pub struct TopFileUsage {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub modified_str: String,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DiskUsageTab {
    Tree,
    Extensions,
    TopFiles,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct CategoryBreakdown {
    pub build_bytes: u64,
    pub deps_bytes: u64,
    pub archives_bytes: u64,
    pub git_bytes: u64,
    pub media_bytes: u64,
    pub other_bytes: u64,
}

#[derive(Clone, Debug)]
pub struct FlatTreeRow {
    pub node_id: usize,
    pub depth: usize,
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub size: u64,
    pub parent_size: u64,
    pub percentage_of_parent: f32,
    pub files_count: usize,
    pub dirs_count: usize,
    pub modified_str: String,
    pub has_children: bool,
    pub is_expanded: bool,
}

#[derive(Clone, Debug)]
pub struct DiskUsageScanResult {
    pub root_node: DiskUsageNode,
    pub extensions: Vec<ExtensionUsage>,
    pub top_files: Vec<TopFileUsage>,
    pub categories: CategoryBreakdown,
    pub total_size: u64,
    pub total_files: usize,
    pub total_dirs: usize,
}

#[derive(Clone, Debug)]
pub struct DiskUsageProgress {
    pub files: usize,
    pub dirs: usize,
    pub bytes: u64,
    pub current_dir: String,
}

pub fn scan_directory_usage(
    root: &Path,
    cancel_flag: Arc<AtomicBool>,
    mut progress_cb: impl FnMut(DiskUsageProgress),
) -> Option<DiskUsageScanResult> {
    let mut id_gen = 0usize;
    let mut total_files = 0usize;
    let mut total_dirs = 0usize;
    let mut total_bytes = 0u64;

    let mut ext_map: HashMap<String, (u64, usize, String)> = HashMap::new();
    let mut all_files: Vec<TopFileUsage> = Vec::new();
    let mut categories = CategoryBreakdown::default();

    let mut last_progress = std::time::Instant::now();

    let root_node = scan_node(
        root,
        &mut id_gen,
        &cancel_flag,
        &mut total_files,
        &mut total_dirs,
        &mut total_bytes,
        &mut ext_map,
        &mut all_files,
        &mut categories,
        &mut progress_cb,
        &mut last_progress,
    )?;

    // Extensions breakdown
    let mut extensions: Vec<ExtensionUsage> = ext_map
        .into_iter()
        .map(|(ext, (size, count, cat))| {
            let pct = if total_bytes > 0 {
                (size as f32 / total_bytes as f32) * 100.0
            } else {
                0.0
            };
            ExtensionUsage {
                extension: ext,
                category: cat,
                size,
                percentage: pct,
                count,
            }
        })
        .collect();
    extensions.sort_by(|a, b| b.size.cmp(&a.size));

    // Top 100 files
    all_files.sort_by(|a, b| b.size.cmp(&a.size));
    if all_files.len() > 100 {
        all_files.truncate(100);
    }

    Some(DiskUsageScanResult {
        total_size: root_node.size,
        root_node,
        extensions,
        top_files: all_files,
        categories,
        total_files,
        total_dirs,
    })
}

fn scan_node(
    path: &Path,
    id_gen: &mut usize,
    cancel_flag: &Arc<AtomicBool>,
    total_files: &mut usize,
    total_dirs: &mut usize,
    total_bytes: &mut u64,
    ext_map: &mut HashMap<String, (u64, usize, String)>,
    all_files: &mut Vec<TopFileUsage>,
    categories: &mut CategoryBreakdown,
    progress_cb: &mut impl FnMut(DiskUsageProgress),
    last_progress: &mut std::time::Instant,
) -> Option<DiskUsageNode> {
    if cancel_flag.load(Ordering::Relaxed) {
        return None;
    }

    let node_id = *id_gen;
    *id_gen += 1;

    let file_name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string_lossy().to_string());

    let metadata = fs::symlink_metadata(path).ok()?;
    let is_dir = metadata.is_dir();
    let mod_ts = metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let modified_str = format_unix_timestamp(mod_ts);

    if !is_dir {
        let size = metadata.len();
        *total_files += 1;
        *total_bytes += size;

        // Categorize file
        categorize_file(path, size, ext_map, categories);

        all_files.push(TopFileUsage {
            name: file_name.clone(),
            path: path.to_path_buf(),
            size,
            modified_str: modified_str.clone(),
        });

        if last_progress.elapsed().as_millis() >= 80 {
            *last_progress = std::time::Instant::now();
            progress_cb(DiskUsageProgress {
                files: *total_files,
                dirs: *total_dirs,
                bytes: *total_bytes,
                current_dir: path
                    .parent()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| path.to_string_lossy().to_string()),
            });
        }

        return Some(DiskUsageNode {
            id: node_id,
            name: file_name,
            path: path.to_path_buf(),
            is_dir: false,
            size,
            files_count: 1,
            dirs_count: 0,
            modified_str,
            children: Vec::new(),
            is_expanded: false,
        });
    }

    // Directory
    *total_dirs += 1;
    if last_progress.elapsed().as_millis() >= 80 {
        *last_progress = std::time::Instant::now();
        progress_cb(DiskUsageProgress {
            files: *total_files,
            dirs: *total_dirs,
            bytes: *total_bytes,
            current_dir: path.to_string_lossy().to_string(),
        });
    }
    let mut dir_size = 0u64;
    let mut dir_files = 0usize;
    let mut dir_subdirs = 0usize;
    let mut children = Vec::new();

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if cancel_flag.load(Ordering::Relaxed) {
                return None;
            }

            let child_path = entry.path();
            let child_name = entry.file_name().to_string_lossy().to_string();

            // Skip symlinks to avoid infinite recursion and cross-filesystem traversal
            if let Ok(ft) = entry.file_type() {
                if ft.is_symlink() {
                    continue;
                }
            }

            // Skip cloud storage providers (Google Drive, iCloud, OneDrive, Dropbox, etc.)
            if is_cloud_storage_dir(&child_name, &child_path) {
                continue;
            }

            if let Some(child_node) = scan_node(
                &child_path,
                id_gen,
                cancel_flag,
                total_files,
                total_dirs,
                total_bytes,
                ext_map,
                all_files,
                categories,
                progress_cb,
                last_progress,
            ) {
                dir_size += child_node.size;
                dir_files += child_node.files_count;
                if child_node.is_dir {
                    dir_subdirs += 1 + child_node.dirs_count;
                }
                children.push(child_node);
            }
        }
    }

    // Sort children descending by size (TreeSize standard)
    children.sort_by(|a, b| b.size.cmp(&a.size));

    Some(DiskUsageNode {
        id: node_id,
        name: file_name,
        path: path.to_path_buf(),
        is_dir: true,
        size: dir_size,
        files_count: dir_files,
        dirs_count: dir_subdirs,
        modified_str,
        children,
        is_expanded: false,
    })
}

fn categorize_file(
    path: &Path,
    size: u64,
    ext_map: &mut HashMap<String, (u64, usize, String)>,
    categories: &mut CategoryBreakdown,
) {
    let path_str = path.to_string_lossy();
    let ext = path
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy().to_lowercase()))
        .unwrap_or_else(|| "(no extension)".to_string());

    let cat_desc = match ext.as_str() {
        ".rs" | ".c" | ".cpp" | ".h" | ".hpp" | ".py" | ".js" | ".ts" | ".tsx" | ".jsx"
        | ".go" | ".java" | ".html" | ".css" | ".json" | ".toml" | ".yaml" | ".yml"
        | ".sh" | ".sql" | ".php" | ".rb" | ".swift" => "Source Code",
        ".rlib" | ".o" | ".dylib" | ".so" | ".a" | ".dll" => "Build / Objects",
        ".zip" | ".gz" | ".tar" | ".tgz" | ".7z" | ".rar" | ".bz2" | ".xz" | ".dmg" | ".iso"
        | ".pkg" => "Archives & Disk Images",
        ".mp4" | ".mov" | ".mkv" | ".avi" | ".mp3" | ".wav" | ".flac" | ".m4a" | ".aac" => {
            "Media / Video & Audio"
        }
        ".png" | ".jpg" | ".jpeg" | ".gif" | ".webp" | ".svg" | ".ico" | ".psd" | ".tiff"
        | ".heic" => "Images",
        ".pdf" | ".doc" | ".docx" | ".xls" | ".xlsx" | ".ppt" | ".pptx" | ".txt" | ".md"
        | ".csv" => "Documents",
        _ => "Other",
    };

    let entry = ext_map
        .entry(ext.clone())
        .or_insert((0, 0, cat_desc.to_string()));
    entry.0 += size;
    entry.1 += 1;

    // Macro breakdown for top category bar
    if path_str.contains("/.git/") || path_str.ends_with("/.git") {
        categories.git_bytes += size;
    } else if path_str.contains("/target/") || path_str.contains("/build/") || path_str.contains("/dist/")
        || ext == ".rlib" || ext == ".o"
    {
        categories.build_bytes += size;
    } else if path_str.contains("/node_modules/")
        || path_str.contains("/.cargo/")
        || path_str.contains("/vendor/")
    {
        categories.deps_bytes += size;
    } else if matches!(
        ext.as_str(),
        ".zip" | ".gz" | ".tar" | ".tgz" | ".7z" | ".rar" | ".bz2" | ".xz" | ".dmg" | ".iso"
    ) {
        categories.archives_bytes += size;
    } else if matches!(
        ext.as_str(),
        ".mp4" | ".mov" | ".mkv" | ".avi" | ".mp3" | ".wav" | ".flac" | ".png" | ".jpg"
            | ".jpeg" | ".gif" | ".webp"
    ) {
        categories.media_bytes += size;
    } else {
        categories.other_bytes += size;
    }
}

pub fn is_cloud_storage_dir(name: &str, path: &Path) -> bool {
    let lower_name = name.to_lowercase();
    if lower_name == "google drive"
        || lower_name == "googledrive"
        || lower_name == "onedrive"
        || lower_name == "dropbox"
        || lower_name == "icloud drive"
        || lower_name == "cloudstorage"
        || lower_name == "mobile documents"
        || lower_name == "com~apple~clouddocs"
        || lower_name == "box sync"
        || lower_name == "box"
        || lower_name == ".trash"
        || lower_name == ".trashes"
        || lower_name == ".fseventsd"
        || lower_name == ".spotlight-v100"
        || lower_name == ".documentrevisions-v100"
        || lower_name.starts_with("onedrive -")
        || lower_name.starts_with("google drive -")
    {
        return true;
    }

    let path_str = path.to_string_lossy();
    if path_str.contains("/Library/CloudStorage")
        || path_str.contains("/Library/Mobile Documents")
        || path_str.contains("/Google Drive")
        || path_str.contains("/OneDrive")
        || path_str.contains("/Dropbox")
        || path_str.contains("/.Trash")
        || path_str.contains("/.Trashes")
    {
        return true;
    }

    false
}

pub fn flatten_tree(root: &DiskUsageNode, out: &mut Vec<FlatTreeRow>) {
    flatten_node_recursive(root, 0, root.size, out);
}

fn flatten_node_recursive(
    node: &DiskUsageNode,
    depth: usize,
    parent_size: u64,
    out: &mut Vec<FlatTreeRow>,
) {
    let pct = if parent_size > 0 {
        (node.size as f32 / parent_size as f32) * 100.0
    } else {
        0.0
    };

    out.push(FlatTreeRow {
        node_id: node.id,
        depth,
        name: node.name.clone(),
        path: node.path.clone(),
        is_dir: node.is_dir,
        size: node.size,
        parent_size,
        percentage_of_parent: pct.min(100.0),
        files_count: node.files_count,
        dirs_count: node.dirs_count,
        modified_str: node.modified_str.clone(),
        has_children: !node.children.is_empty(),
        is_expanded: node.is_expanded,
    });

    if node.is_expanded {
        for child in &node.children {
            flatten_node_recursive(child, depth + 1, node.size, out);
        }
    }
}

pub fn toggle_node_expanded(root: &mut DiskUsageNode, target_id: usize) -> bool {
    if root.id == target_id {
        root.is_expanded = !root.is_expanded;
        return true;
    }
    for child in &mut root.children {
        if toggle_node_expanded(child, target_id) {
            return true;
        }
    }
    false
}

pub fn set_all_expanded(node: &mut DiskUsageNode, expanded: bool) {
    if node.is_dir {
        node.is_expanded = expanded;
        for child in &mut node.children {
            set_all_expanded(child, expanded);
        }
    }
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
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_disk_usage_scan_and_tree() {
        let temp_dir = std::env::temp_dir().join("ruster_disk_usage_test");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(temp_dir.join("sub_a")).unwrap();
        fs::create_dir_all(temp_dir.join("sub_b")).unwrap();

        // Create test files
        let mut f1 = File::create(temp_dir.join("sub_a").join("code.rs")).unwrap();
        f1.write_all(&[0u8; 1024]).unwrap();

        let mut f2 = File::create(temp_dir.join("sub_b").join("archive.zip")).unwrap();
        f2.write_all(&[0u8; 4096]).unwrap();

        let cancel_flag = Arc::new(AtomicBool::new(false));
        let res = scan_directory_usage(&temp_dir, cancel_flag, |_| {}).expect("Scan should succeed");

        assert_eq!(res.total_files, 2);
        assert_eq!(res.total_dirs, 3);
        assert!(res.total_size >= 5120);

        let ext_names: Vec<String> = res.extensions.iter().map(|e| e.extension.clone()).collect();
        assert!(ext_names.contains(&".rs".to_string()));
        assert!(ext_names.contains(&".zip".to_string()));

        let mut root = res.root_node;
        root.is_expanded = true;
        for c in &mut root.children {
            c.is_expanded = true;
        }

        let mut rows = Vec::new();
        flatten_tree(&root, &mut rows);
        assert!(rows.len() >= 4);

        let first_child_id = root.children[0].id;
        assert!(toggle_node_expanded(&mut root, first_child_id));
        assert!(!root.children[0].is_expanded);

        set_all_expanded(&mut root, true);
        assert!(root.children[0].is_expanded);

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
