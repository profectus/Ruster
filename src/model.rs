#![allow(dead_code)]

use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ActivePane {
    Local,
    Remote,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Protocol {
    Sftp,
    Scp,
    FtpsExplicit,
    FtpsImplicit,
    Ftp,
}

impl Protocol {
    pub fn label(&self) -> &'static str {
        match self {
            Protocol::Sftp => "SFTP",
            Protocol::Scp => "SCP",
            Protocol::FtpsExplicit => "FTPS (Explicit)",
            Protocol::FtpsImplicit => "FTPS (Implicit)",
            Protocol::Ftp => "FTP",
        }
    }

    pub fn short_badge(&self) -> &'static str {
        match self {
            Protocol::Sftp => "SFTP",
            Protocol::Scp => "SCP",
            Protocol::FtpsExplicit => "FTPS",
            Protocol::FtpsImplicit => "FTPS",
            Protocol::Ftp => "FTP",
        }
    }

    pub fn default_port(&self) -> u16 {
        match self {
            Protocol::Sftp | Protocol::Scp => 22,
            Protocol::FtpsExplicit | Protocol::Ftp => 21,
            Protocol::FtpsImplicit => 990,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SortColumn {
    Name,
    Size,
    Modified,
    Permissions,
}

#[derive(Clone, Debug)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub size: u64,
    pub modified: String,
    pub modified_ts: i64,
    pub permissions_str: String,
    pub octal_mode: u32,
    pub owner: String,
    pub selected: bool,
    pub space_marked: bool,
    pub calculating_size: bool,
    pub calculated_dir_size: Option<u64>,
}

impl FileEntry {
    pub fn is_any_selected(&self) -> bool {
        self.selected || self.space_marked
    }

    pub fn formatted_size(&self, folder_label: &str) -> String {
        if self.name == ".." {
            return "—".to_string();
        }
        if self.is_dir {
            if self.space_marked && self.calculating_size {
                return "...".to_string();
            }
            if let Some(calc) = self.calculated_dir_size {
                return format_bytes(calc);
            }
            return folder_label.to_string();
        }
        format_bytes(self.size)
    }

    pub fn effective_size(&self) -> u64 {
        if self.is_dir {
            self.calculated_dir_size.unwrap_or(0)
        } else {
            self.size
        }
    }
}

pub fn format_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    let b = bytes as f64;
    if b >= GB {
        format!("{:.2} GB", b / GB)
    } else if b >= MB {
        format!("{:.1} MB", b / MB)
    } else if b >= KB {
        format!("{:.1} KB", b / KB)
    } else {
        format!("{} B", bytes)
    }
}

pub fn mode_to_rwx(mode: u32, is_dir: bool, is_symlink: bool) -> String {
    let file_type = if is_symlink {
        'l'
    } else if is_dir {
        'd'
    } else {
        '-'
    };

    let flags = [
        (0o400, 'r'),
        (0o200, 'w'),
        (0o100, 'x'),
        (0o040, 'r'),
        (0o020, 'w'),
        (0o010, 'x'),
        (0o004, 'r'),
        (0o002, 'w'),
        (0o001, 'x'),
    ];

    let mut s = String::with_capacity(10);
    s.push(file_type);
    for (bit, ch) in flags {
        if mode & bit != 0 {
            s.push(ch);
        } else {
            s.push('-');
        }
    }
    s
}

pub fn read_local_directory(dir: &Path, show_hidden: bool) -> Vec<FileEntry> {
    let mut entries = Vec::new();

    let clean_str = dir.to_string_lossy().trim_end_matches('/').to_string();
    let clean_path = if clean_str.is_empty() {
        PathBuf::from("/")
    } else {
        PathBuf::from(clean_str)
    };

    if clean_path != Path::new("/") {
        if let Some(parent) = clean_path.parent() {
            let parent_str = parent.to_string_lossy();
            let parent_target = if parent_str.is_empty() {
                "/".to_string()
            } else {
                parent_str.to_string()
            };
            entries.push(FileEntry {
                name: "..".to_string(),
                path: parent_target,
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
    }

    if let Ok(read_dir) = fs::read_dir(&clean_path) {
        for item in read_dir.flatten() {
            let name = item.file_name().to_string_lossy().to_string();
            if !show_hidden && name.starts_with('.') {
                continue;
            }

            let path_buf = item.path();
            let ft = item.file_type().ok();
            let is_symlink = ft.as_ref().map(|f| f.is_symlink()).unwrap_or(false);
            let mut is_dir = ft.as_ref().map(|f| f.is_dir()).unwrap_or(false);

            let symlink_meta = fs::symlink_metadata(&path_buf).ok();
            let meta = fs::metadata(&path_buf).ok().or(symlink_meta.clone());

            let (size, raw_mode, permissions_str, modified, modified_ts, owner) =
                if let Some(ref m) = meta {
                    is_dir = m.is_dir();
                    let size = if is_dir { 0 } else { m.len() };
                    let raw_mode = m.permissions().mode() & 0o777;
                    let perm = mode_to_rwx(raw_mode, is_dir, is_symlink);

                    let (mod_str, mod_ts) = m
                        .modified()
                        .ok()
                        .and_then(|st| st.duration_since(UNIX_EPOCH).ok())
                        .map(|dur| {
                            let secs = dur.as_secs() as i64;
                            let formatted = unsafe {
                                let mut tm: libc::tm = std::mem::zeroed();
                                let time_t_val = secs as libc::time_t;
                                if !libc::localtime_r(&time_t_val, &mut tm).is_null() {
                                    format!(
                                        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                                        tm.tm_year + 1900,
                                        tm.tm_mon + 1,
                                        tm.tm_mday,
                                        tm.tm_hour,
                                        tm.tm_min,
                                        tm.tm_sec
                                    )
                                } else {
                                    "-".to_string()
                                }
                            };
                            (formatted, secs)
                        })
                        .unwrap_or_else(|| ("-".to_string(), 0));

                    let uid = m.uid();
                    let owner = if uid == 0 {
                        "root".to_string()
                    } else {
                        std::env::var("USER").unwrap_or_else(|_| uid.to_string())
                    };
                    (size, raw_mode, perm, mod_str, mod_ts, owner)
                } else {
                    (
                        0,
                        if is_dir { 0o755 } else { 0o644 },
                        if is_dir { "drwxr-xr-x".to_string() } else { "-rw-r--r--".to_string() },
                        "-".to_string(),
                        0,
                        "unknown".to_string(),
                    )
                };

            entries.push(FileEntry {
                name,
                path: path_buf.to_string_lossy().to_string(),
                is_dir,
                is_symlink,
                size,
                modified,
                modified_ts,
                permissions_str,
                octal_mode: raw_mode,
                owner,
                selected: false,
                space_marked: false,
                calculating_size: false,
                calculated_dir_size: None,
            });
        }
    }

    sort_entries(&mut entries, SortColumn::Name, true);
    entries
}

pub fn sort_entries(entries: &mut [FileEntry], column: SortColumn, ascending: bool) {
    entries.sort_by(|a, b| {
        if a.name == ".." {
            return std::cmp::Ordering::Less;
        }
        if b.name == ".." {
            return std::cmp::Ordering::Greater;
        }
        if a.is_dir != b.is_dir {
            return b.is_dir.cmp(&a.is_dir);
        }

        let ord = match column {
            SortColumn::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            SortColumn::Size => a.size.cmp(&b.size),
            SortColumn::Modified => a.modified_ts.cmp(&b.modified_ts),
            SortColumn::Permissions => a.octal_mode.cmp(&b.octal_mode),
        };

        if ascending {
            ord
        } else {
            ord.reverse()
        }
    });
}

// --- Real Local File System Operations ---

pub fn create_local_folder(base_dir: &Path, folder_name: &str) -> std::io::Result<PathBuf> {
    let target = base_dir.join(folder_name.trim());
    fs::create_dir_all(&target)?;
    Ok(target)
}

pub fn rename_local_item(base_dir: &Path, old_name: &str, new_name: &str) -> std::io::Result<()> {
    let from = base_dir.join(old_name);
    let to = base_dir.join(new_name.trim());
    fs::rename(from, to)
}

pub fn delete_local_item(path: &Path, is_dir: bool) -> std::io::Result<()> {
    if is_dir {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    }
}

pub fn apply_local_chmod(path: &Path, mode: u32, recursive: bool) -> std::io::Result<()> {
    let perm = fs::Permissions::from_mode(mode);
    fs::set_permissions(path, perm.clone())?;
    if recursive && path.is_dir() {
        if let Ok(entries) = fs::read_dir(path) {
            for item in entries.flatten() {
                let p = item.path();
                let _ = apply_local_chmod(&p, mode, true);
            }
        }
    }
    Ok(())
}

pub fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&from, &to)?;
        } else {
            fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

pub fn copy_local_item_with_progress<F>(
    source: &Path,
    target_dir: &Path,
    mut on_progress: F,
) -> std::io::Result<u64>
where
    F: FnMut(u64, u64),
{
    let file_name = source
        .file_name()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid file name"))?;

    let target = if target_dir == source.parent().unwrap_or(Path::new("")) {
        let stem = source.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
        let ext = source.extension().and_then(|s| s.to_str());
        let new_name = match ext {
            Some(e) => format!("{}_copy.{}", stem, e),
            None => format!("{}_copy", stem),
        };
        target_dir.join(new_name)
    } else {
        target_dir.join(file_name)
    };

    if source.is_dir() {
        copy_dir_all(source, &target)?;
        on_progress(1, 1);
        Ok(0)
    } else {
        let mut src_file = fs::File::open(source)?;
        let total = src_file.metadata().map(|m| m.len()).unwrap_or(0);
        let mut dest_file = fs::File::create(&target)?;
        let mut buf = [0u8; 131072]; // 128 KB
        let mut copied = 0u64;
        use std::io::{Read, Write};
        loop {
            let n = src_file.read(&mut buf)?;
            if n == 0 {
                break;
            }
            dest_file.write_all(&buf[..n])?;
            copied += n as u64;
            on_progress(copied, total);
        }
        dest_file.flush()?;
        on_progress(total, total);
        Ok(copied)
    }
}

pub fn copy_local_item(source: &Path, target_dir: &Path) -> std::io::Result<u64> {
    copy_local_item_with_progress(source, target_dir, |_, _| {})
}

pub fn move_local_item_with_progress<F>(
    source: &Path,
    target_dir: &Path,
    mut on_progress: F,
) -> std::io::Result<()>
where
    F: FnMut(u64, u64),
{
    let file_name = source
        .file_name()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid file name"))?;
    let target = target_dir.join(file_name);

    let total = source.metadata().map(|m| m.len()).unwrap_or(0);
    if fs::rename(source, &target).is_ok() {
        on_progress(total, total);
        return Ok(());
    }
    copy_local_item_with_progress(source, target_dir, &mut on_progress)?;
    delete_local_item(source, source.is_dir())?;
    Ok(())
}

pub fn move_local_item(source: &Path, target_dir: &Path) -> std::io::Result<()> {
    move_local_item_with_progress(source, target_dir, |_, _| {})
}

#[derive(Clone, Debug)]
pub struct RemoteSession {
    pub id: usize,
    pub title: String,
    pub protocol: Protocol,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: Option<String>,
    pub env_color: u32,
    pub current_path: String,
    pub entries: Vec<FileEntry>,
    pub selected_index: usize,
    pub max_connections: usize,
}

#[derive(Clone, Debug)]
pub struct SavedSiteProfile {
    pub name: String,
    pub folder: String,
    pub protocol: Protocol,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: Option<String>,
    pub auth_type: String,
    pub env_color: u32,
    pub max_connections: usize,
}

impl SavedSiteProfile {
    pub fn display_name(&self) -> &str {
        if !self.name.trim().is_empty() {
            self.name.trim()
        } else if !self.host.trim().is_empty() {
            self.host.trim()
        } else {
            "<New Site>"
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EnvironmentColor {
    pub color: u32,
    pub name_en: &'static str,
    pub name_cs: &'static str,
}

pub const ENV_COLORS: [EnvironmentColor; 5] = [
    EnvironmentColor {
        color: 0xef4444, // Red
        name_en: "Red",
        name_cs: "Červená",
    },
    EnvironmentColor {
        color: 0xf59e0b, // Orange
        name_en: "Orange",
        name_cs: "Oranžová",
    },
    EnvironmentColor {
        color: 0x2563eb, // Blue
        name_en: "Blue",
        name_cs: "Modrá",
    },
    EnvironmentColor {
        color: 0x10b981, // Green
        name_en: "Green",
        name_cs: "Zelená",
    },
    EnvironmentColor {
        color: 0x8b5cf6, // Purple
        name_en: "Purple",
        name_cs: "Fialová",
    },
];

#[derive(Clone, Debug)]
pub struct WatchedRemoteFile {
    pub session_id: usize,
    pub session_name: String,
    pub protocol: Protocol,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: Option<String>,
    pub remote_path: String,
    pub local_path: PathBuf,
    pub filename: String,
    pub last_mtime: std::time::SystemTime,
    pub last_size: u64,
    pub is_uploading: bool,
}

pub fn default_editable_extensions() -> String {
    "txt, md, json, js, ts, tsx, jsx, html, htm, css, scss, php, py, rb, rs, c, cpp, h, hpp, go, java, xml, yaml, yml, ini, conf, cfg, sh, bash, zsh, env, sql, log, csv, toml".to_string()
}

pub fn is_editable_file(filename: &str, editable_exts: &str) -> bool {
    let lower_name = filename.to_lowercase();
    let exts: Vec<&str> = editable_exts
        .split(|c: char| c == ',' || c == ' ' || c == ';')
        .map(|s| s.trim().trim_start_matches('.'))
        .filter(|s| !s.is_empty())
        .collect();

    if let Some(pos) = lower_name.rfind('.') {
        let ext = &lower_name[pos + 1..];
        if exts.iter().any(|&e| e.eq_ignore_ascii_case(ext)) {
            return true;
        }
    }
    if exts.iter().any(|&e| e.eq_ignore_ascii_case(&lower_name) || e.eq_ignore_ascii_case(lower_name.trim_start_matches('.'))) {
        return true;
    }
    false
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TransferDirection {
    Upload,
    Download,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TransferStatus {
    Transferring,
    Completed,
    Queued,
    Failed,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BottomPanelTab {
    Transfers,
    Console,
    Split,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ServerLogKind {
    Status,
    Command,
    Response,
    Error,
}

#[derive(Clone, Debug)]
pub struct ServerLogEntry {
    pub time: String,
    pub kind: ServerLogKind,
    pub message: String,
}

#[derive(Clone, Debug)]
pub struct TransferQueueItem {
    pub id: usize,
    pub batch_id: usize,
    pub filename: String,
    pub source_path: String,
    pub target_path: String,
    pub direction: TransferDirection,
    pub protocol: Protocol,
    pub size_bytes: u64,
    pub transferred_bytes: u64,
    pub speed_mbps: f64,
    pub status: TransferStatus,
    pub is_dir: bool,
    pub parent_id: Option<usize>,
}

impl TransferQueueItem {
    pub fn progress_percent(&self) -> f32 {
        if self.status == TransferStatus::Completed {
            return 100.0;
        }
        if self.size_bytes == 0 {
            return 0.0;
        }
        ((self.transferred_bytes as f64 / self.size_bytes as f64) * 100.0).clamp(0.0, 100.0) as f32
    }
}

#[derive(Clone, Debug)]
pub struct ActiveTransferState {
    pub direction: TransferDirection,
    pub total_items: usize,
    pub current_item_index: usize,
    pub current_filename: String,
    pub source_path: String,
    pub target_path: String,
    pub file_bytes_transferred: u64,
    pub file_bytes_total: u64,
    pub total_bytes_transferred: u64,
    pub total_bytes: u64,
    pub speed_mbps: f64,
    pub eta_seconds: Option<u64>,
    pub is_completed: bool,
    pub error_message: Option<String>,
}

impl ActiveTransferState {
    pub fn file_percent(&self) -> f32 {
        if self.file_bytes_total == 0 {
            if self.is_completed {
                100.0
            } else {
                0.0
            }
        } else {
            ((self.file_bytes_transferred as f64 / self.file_bytes_total as f64) * 100.0).clamp(0.0, 100.0) as f32
        }
    }

    pub fn overall_percent(&self) -> f32 {
        if self.is_completed {
            return 100.0;
        }
        if self.total_bytes > 0 {
            let cur = self.total_bytes_transferred + self.file_bytes_transferred;
            ((cur as f64 / self.total_bytes as f64) * 100.0).clamp(0.0, 100.0) as f32
        } else if self.total_items > 0 {
            let done_items = self.current_item_index.saturating_sub(1);
            let partial = self.file_percent() / 100.0;
            (((done_items as f32 + partial) / self.total_items as f32) * 100.0).clamp(0.0, 100.0)
        } else {
            0.0
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ActiveModal {
    None,
    SiteManager,
    Permissions,
    Synchronize,
    NewFolder,
    Rename,
    DeleteConfirm,
    GoToFolder,
    TransferProgress,
    Settings,
    Update,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SyncDirection {
    LocalToRemote,
    RemoteToLocal,
    TwoWay,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SyncCriterion {
    TimeAndSize,
    TimeOnly,
    SizeOnly,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SyncAction {
    Upload,
    Download,
    DeleteRemote,
    DeleteLocal,
}

#[derive(Clone, Debug)]
pub struct SyncItem {
    pub id: usize,
    pub rel_path: String,
    pub filename: String,
    pub action: SyncAction,
    pub is_dir: bool,
    pub local_size: u64,
    pub remote_size: u64,
    pub local_mtime: i64,
    pub remote_mtime: i64,
    pub reason: String,
    pub selected: bool,
}

#[derive(Clone, Debug)]
pub struct SyncState {
    pub direction: SyncDirection,
    pub criterion: SyncCriterion,
    pub recursive: bool,
    pub mirror_delete: bool,
    pub is_comparing: bool,
    pub has_compared: bool,
    pub progress_msg: Option<String>,
    pub error_message: Option<String>,
    pub items: Vec<SyncItem>,
}

impl Default for SyncState {
    fn default() -> Self {
        Self {
            direction: SyncDirection::LocalToRemote,
            criterion: SyncCriterion::TimeAndSize,
            recursive: true,
            mirror_delete: false,
            is_comparing: false,
            has_compared: false,
            progress_msg: None,
            error_message: None,
            items: Vec::new(),
        }
    }
}

impl SyncState {
    pub fn selected_count(&self) -> usize {
        self.items.iter().filter(|i| i.selected).count()
    }

    pub fn selected_bytes(&self) -> u64 {
        self.items.iter().filter(|i| i.selected).map(|i| match i.action {
            SyncAction::Upload => i.local_size,
            SyncAction::Download => i.remote_size,
            _ => 0,
        }).sum()
    }

    pub fn all_selected(&self) -> bool {
        !self.items.is_empty() && self.items.iter().all(|i| i.selected)
    }

    pub fn set_all_selected(&mut self, selected: bool) {
        for item in &mut self.items {
            item.selected = selected;
        }
    }

    pub fn toggle_item(&mut self, id: usize) {
        if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
            item.selected = !item.selected;
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SiteEditField {
    None,
    Name,
    Host,
    Port,
    Username,
    Password,
}

pub fn default_saved_sites() -> Vec<SavedSiteProfile> {
    vec![SavedSiteProfile {
        name: String::new(),
        folder: String::new(),
        protocol: Protocol::Sftp,
        host: String::new(),
        port: 22,
        username: String::new(),
        password: None,
        auth_type: String::new(),
        env_color: 0x2563eb,
        max_connections: 2,
    }]
}

pub fn default_transfer_queue() -> Vec<TransferQueueItem> {
    Vec::new()
}

pub fn sample_remote_entries(_path: &str) -> Vec<FileEntry> {
    Vec::new()
}

pub fn default_sessions() -> Vec<RemoteSession> {
    Vec::new()
}

pub fn initial_local_dir() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/"))
}

pub fn initial_right_dir() -> PathBuf {
    std::env::var("HOME")
        .map(|h| PathBuf::from(h).join("Downloads"))
        .ok()
        .filter(|p| p.exists())
        .or_else(|| std::env::var("HOME").ok().map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("/"))
}

#[derive(Clone, Debug)]
pub struct TreeNode {
    pub depth: usize,
    pub label: String,
    pub path: PathBuf,
    pub is_current: bool,
}

pub fn build_local_tree(current_dir: &Path, show_hidden: bool) -> Vec<TreeNode> {
    let clean_str = current_dir.to_string_lossy().trim_end_matches('/').to_string();
    let clean_path = if clean_str.is_empty() {
        PathBuf::from("/")
    } else {
        PathBuf::from(clean_str)
    };

    let mut nodes = Vec::new();
    let mut ancestors: Vec<PathBuf> = clean_path.ancestors().map(|p| p.to_path_buf()).collect();
    ancestors.reverse();

    for (depth, anc) in ancestors.iter().enumerate() {
        let label = if depth == 0 {
            "/".to_string()
        } else {
            anc.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "/".to_string())
        };
        nodes.push(TreeNode {
            depth,
            label,
            path: anc.clone(),
            is_current: anc == &clean_path,
        });
    }

    let child_depth = ancestors.len();
    if let Ok(read_dir) = fs::read_dir(&clean_path) {
        let mut child_dirs: Vec<(String, PathBuf)> = read_dir
            .flatten()
            .filter_map(|item| {
                let name = item.file_name().to_string_lossy().to_string();
                if !show_hidden && name.starts_with('.') {
                    return None;
                }
                let path = item.path();
                let is_dir = item.file_type().map(|f| f.is_dir()).unwrap_or(false);
                if is_dir {
                    Some((name, path))
                } else {
                    None
                }
            })
            .collect();
        child_dirs.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
        for (name, path) in child_dirs {
            nodes.push(TreeNode {
                depth: child_depth,
                label: name,
                path,
                is_current: false,
            });
        }
    }

    nodes
}

pub fn calculate_local_dir_size(path: &std::path::Path) -> u64 {
    let mut total = 0u64;
    let mut stack = vec![path.to_path_buf()];
    let mut visited_count = 0;

    while let Some(dir) = stack.pop() {
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                visited_count += 1;
                if visited_count > 250_000 {
                    return total;
                }
                if let Ok(ft) = entry.file_type() {
                    if ft.is_symlink() {
                        continue;
                    }
                    if ft.is_dir() {
                        stack.push(entry.path());
                    } else if ft.is_file() {
                        if let Ok(meta) = entry.metadata() {
                            total += meta.len();
                        }
                    }
                }
            }
        }
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_editable_file() {
        let exts = default_editable_extensions();
        assert!(is_editable_file("index.php", &exts));
        assert!(is_editable_file("style.css", &exts));
        assert!(is_editable_file("app.js", &exts));
        assert!(is_editable_file("server.ts", &exts));
        assert!(is_editable_file("data.json", &exts));
        assert!(is_editable_file("README.md", &exts));
        assert!(is_editable_file("config.ini", &exts));
        assert!(is_editable_file("notes.txt", &exts));
        assert!(is_editable_file(".env", &exts));
        assert!(is_editable_file("index.HTML", &exts));
        assert!(is_editable_file("MAIN.PHP", &exts));

        assert!(!is_editable_file("archive.zip", &exts));
        assert!(!is_editable_file("image.png", &exts));
        assert!(!is_editable_file("video.mp4", &exts));
        assert!(!is_editable_file("app.exe", &exts));
    }

    #[test]
    fn test_file_filtering() {
        let make_entry = |name: &str, is_dir: bool| FileEntry {
            name: name.to_string(),
            path: format!("/test/{}", name),
            is_dir,
            is_symlink: false,
            size: 1024,
            modified: "2026-01-01".to_string(),
            modified_ts: 0,
            permissions_str: "rw-r--r--".to_string(),
            octal_mode: 0o644,
            owner: "user".to_string(),
            selected: false,
            space_marked: false,
            calculating_size: false,
            calculated_dir_size: None,
        };

        let entries = vec![
            make_entry("..", true),
            make_entry("invoice.pdf", false),
            make_entry("REPORT.PDF", false),
            make_entry("document.docx", false),
        ];

        let filter_entries = |query: &str| -> Vec<String> {
            let q = query.trim().to_lowercase();
            entries
                .iter()
                .filter(|e| q.is_empty() || e.name == ".." || e.name.to_lowercase().contains(&q))
                .map(|e| e.name.clone())
                .collect()
        };

        // Empty filter returns everything
        assert_eq!(filter_entries("").len(), 4);

        // .pdf filter returns ".." and the two PDF files (case-insensitive)
        let pdfs = filter_entries(".pdf");
        assert_eq!(pdfs, vec!["..", "invoice.pdf", "REPORT.PDF"]);

        // pdf filter (without dot) also matches
        let pdfs_no_dot = filter_entries("pdf");
        assert_eq!(pdfs_no_dot, vec!["..", "invoice.pdf", "REPORT.PDF"]);

        // docx filter matches only document.docx
        let docx = filter_entries("docx");
        assert_eq!(docx, vec!["..", "document.docx"]);

        // non-matching query only returns ".."
        let none = filter_entries("xyz");
        assert_eq!(none, vec![".."]);

        // Count of found items never counts ".."
        let found_count = |query: &str| -> usize {
            filter_entries(query).into_iter().filter(|n| n != "..").count()
        };
        assert_eq!(found_count(""), 3);
        assert_eq!(found_count(".pdf"), 2);
        assert_eq!(found_count("pdf"), 2);
        assert_eq!(found_count("docx"), 1);
        assert_eq!(found_count("xyz"), 0);
    }

    #[test]
    fn test_environment_colors() {
        assert_eq!(ENV_COLORS.len(), 5);
        assert_eq!(ENV_COLORS[0].name_en, "Red");
        assert_eq!(ENV_COLORS[0].color, 0xef4444);
        assert_eq!(ENV_COLORS[1].name_en, "Orange");
        assert_eq!(ENV_COLORS[1].color, 0xf59e0b);
        assert_eq!(ENV_COLORS[2].name_en, "Blue");
        assert_eq!(ENV_COLORS[2].color, 0x2563eb);
        assert_eq!(ENV_COLORS[3].name_en, "Green");
        assert_eq!(ENV_COLORS[3].color, 0x10b981);
        assert_eq!(ENV_COLORS[4].name_en, "Purple");
        assert_eq!(ENV_COLORS[4].color, 0x8b5cf6);
    }
}
