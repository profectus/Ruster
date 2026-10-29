use std::collections::HashMap;
use std::path::PathBuf;
use gpui::{
    div, prelude::*, px, rgb, rgba, Context, FocusHandle, KeyDownEvent,
    MouseButton, MouseMoveEvent, Point, Render, ScrollHandle, ScrollStrategy,
    UniformListScrollHandle, Window,
};

use crate::config::{
    cleanup_remote_edit_temp, load_app_state, load_saved_sites, save_app_state,
    save_password_to_keychain, save_sites_to_disk,
};
use crate::icons::render_icon;
use crate::i18n::{I18n, Locale};
use crate::model::{
    apply_local_chmod, build_local_tree, calculate_local_dir_size, copy_local_item_with_progress,
    create_local_folder, default_saved_sites, delete_local_item, is_editable_file,
    move_local_item_with_progress, read_local_directory, rename_local_item, sort_entries,
    ActiveModal, ActivePane, ActiveTransferState, BottomPanelTab, DateFormat, FileEntry, PaneFlash, Protocol,
    RemoteSession, SavedSiteProfile, ServerLogEntry, ServerLogKind, SiteEditField, SortColumn,
    SyncAction, SyncItem, SyncState,
    TransferDirection, TransferErrorAction, TransferErrorPromptState, TransferQueueItem, TransferStatus, TreeNode, WatchedRemoteFile,
};
use crate::network::{
    calculate_remote_dir_size,
    download_from_remote_with_progress_and_log,
    list_remote_directory, remote_chmod, remote_delete, remote_mkdir, remote_rename,
    sanitize_host_and_path,
    upload_to_remote_with_progress_and_log,
};
use crate::ui::menus::install_app_menus;
use crate::{
    CheckForUpdates, CopySelected, DeleteModal, EditSelected, GoToFolderModal, MoveSelected,
    NewFolderModal, OpenInTerminal, OpenSelected, OpenSiteManager, PermissionsModal, RefreshPanes, RenameModal,
    RevealInFinder, SelectAllItems, SwapPanes, SwitchLanguage, SynchronizeModal, ToggleDualPane,
    ToggleFunctionBar, ToggleHiddenFiles, ToggleLocalTree, ToggleQuickFilter, ToggleRemoteTree, ToggleTheme,
    ToggleTransferQueue,
};

#[derive(Clone, Debug)]
pub struct ContextMenuState {
    pub pane: ActivePane,
    pub position: Point<gpui::Pixels>,
}

#[derive(Clone, Debug)]
pub struct DraggedFileItem {
    pub source_pane: ActivePane,
    pub entries: Vec<FileEntry>,
    pub position: Point<gpui::Pixels>,
}

impl DraggedFileItem {
    pub fn new(source_pane: ActivePane, entries: Vec<FileEntry>) -> Self {
        Self {
            source_pane,
            entries,
            position: Point::default(),
        }
    }

    pub fn position(mut self, pos: Point<gpui::Pixels>) -> Self {
        self.position = pos;
        self
    }
}

impl Render for DraggedFileItem {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let label = if self.entries.len() == 1 {
            self.entries.first().map(|e| e.name.clone()).unwrap_or_default()
        } else {
            format!("{} items", self.entries.len())
        };

        div()
            .pl(self.position.x)
            .pt(self.position.y)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1p5()
                    .px_2p5()
                    .py_1()
                    .rounded(px(6.0))
                    .bg(rgba(0x2563ebdd))
                    .text_xs()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(rgb(0xffffff))
                    .shadow_lg()
                    .child(render_icon("icons/file.svg", 13.0, 0xffffff))
                    .child(label),
            )
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ResizeColumnState {
    pub col_index: usize,
    pub start_mouse_x: f32,
    pub initial_size_w: f32,
    pub initial_modified_w: f32,
    pub initial_permissions_w: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct ResizeQueuePanelState {
    pub start_mouse_y: f32,
    pub initial_height: f32,
}

pub struct RusterView {
    pub(crate) focus_handle: FocusHandle,
    pub(crate) locale: Locale,
    pub(crate) theme: crate::theme::ThemeMode,
    pub(crate) active_pane: ActivePane,
    pub(crate) show_local_pane: bool,
    pub(crate) swap_panes: bool,
    pub(crate) show_local_tree: bool,
    pub(crate) show_remote_tree: bool,
    pub(crate) show_hidden: bool,
    pub(crate) show_function_bar: bool,
    pub(crate) show_permissions: bool,
    pub(crate) date_format: DateFormat,
    pub(crate) show_queue: bool,
    pub(crate) show_local_filter: bool,
    pub(crate) local_filter_query: String,
    pub(crate) show_remote_filter: bool,
    pub(crate) remote_filter_query: String,
    pub(crate) session_dropdown_open: bool,
    pub(crate) context_menu: Option<ContextMenuState>,
    pub(crate) favorite_paths: Vec<PathBuf>,
    pub(crate) favorites_menu_pane: Option<ActivePane>,

    pub(crate) local_scroll: UniformListScrollHandle,
    pub(crate) remote_scroll: UniformListScrollHandle,
    pub(crate) console_scroll: ScrollHandle,

    pub(crate) local_dir: PathBuf,
    pub(crate) local_entries: Vec<FileEntry>,
    pub(crate) local_tree: Vec<TreeNode>,
    pub(crate) local_selected: usize,
    pub(crate) local_sort_col: SortColumn,
    pub(crate) local_sort_asc: bool,

    pub(crate) right_dir: PathBuf,
    pub(crate) right_entries: Vec<FileEntry>,
    pub(crate) right_tree: Vec<TreeNode>,
    pub(crate) right_selected: usize,

    pub(crate) remote_flash: Option<PaneFlash>,
    pub(crate) remote_flash_id: usize,

    pub(crate) sessions: Vec<RemoteSession>,
    pub(crate) active_session_idx: usize,
    pub(crate) remote_sort_col: SortColumn,
    pub(crate) remote_sort_asc: bool,

    pub(crate) transfer_queue: Vec<TransferQueueItem>,
    pub(crate) max_concurrent_transfers: usize,
    pub(crate) next_transfer_id: usize,
    pub(crate) next_batch_id: usize,
    pub(crate) expanded_batches: std::collections::HashSet<usize>,
    pub(crate) expanded_folders: std::collections::HashSet<usize>,

    pub(crate) col_size_width: f32,
    pub(crate) col_modified_width: f32,
    pub(crate) col_permissions_width: f32,
    pub(crate) resizing_column: Option<ResizeColumnState>,
    pub(crate) queue_panel_height: f32,
    pub(crate) resizing_queue_panel: Option<ResizeQueuePanelState>,

    pub(crate) active_modal: ActiveModal,
    pub(crate) modal_input_text: String,
    pub(crate) saved_sites: Vec<SavedSiteProfile>,
    pub(crate) selected_site_idx: usize,
    pub(crate) draft_protocol: Protocol,
    pub(crate) active_site_field: SiteEditField,
    pub(crate) site_password_input: String,
    pub(crate) site_cursor_pos: usize,
    pub(crate) modal_cursor_pos: usize,
    pub(crate) site_error_message: Option<String>,
    pub(crate) site_notice: Option<(String, SiteNoticeKind)>,
    pub(crate) site_is_connecting: bool,
    pub(crate) remote_error_banner: Option<String>,
    pub(crate) active_transfer: Option<ActiveTransferState>,
    pub(crate) transfer_cancel_signal: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,

    pub(crate) server_log: Vec<ServerLogEntry>,
    pub(crate) bottom_tab: BottomPanelTab,
    pub(crate) show_transfer_log: bool,
    pub(crate) transfer_error_prompt: Option<TransferErrorPromptState>,

    pub(crate) chmod_mode: u32,
    pub(crate) chmod_recursive: bool,

    pub(crate) watched_remote_files: Vec<WatchedRemoteFile>,
    pub(crate) editable_extensions: String,
    pub(crate) settings_editing_exts: bool,
    pub(crate) settings_cursor_pos: usize,
    pub(crate) settings_lang_dropdown_open: bool,

    pub(crate) update_status: Option<crate::update::UpdateStatus>,
    pub(crate) update_copied_notice: bool,
    pub(crate) sync_state: SyncState,

    pub(crate) window_width: f32,
    pub(crate) window_height: f32,
    pub(crate) last_window_size_saved: std::time::Instant,
}

#[derive(Clone, Debug)]
enum TransferKind {
    Download {
        protocol: Protocol,
        host: String,
        port: u16,
        username: String,
        password: Option<String>,
        remote_path: String,
        local_path: PathBuf,
        is_dir: bool,
        remove_source: bool,
    },
    Upload {
        protocol: Protocol,
        host: String,
        port: u16,
        username: String,
        password: Option<String>,
        local_path: PathBuf,
        remote_path: String,
        is_dir: bool,
        remove_source: bool,
    },
    Local {
        src: PathBuf,
        dest: PathBuf,
        remove_source: bool,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SiteNoticeKind {
    Success,
    Info,
    Error,
}

#[derive(Clone, Debug)]
pub(crate) struct TransferTask {
    queue_id: usize,
    filename: String,
    source_display: String,
    target_display: String,
    size_bytes: u64,
    kind: TransferKind,
}

#[derive(Clone, Debug)]
enum TransferProgressUpdate {
    BatchInitialized {
        total_items: usize,
        total_bytes: u64,
    },
    ItemStarted {
        filename: String,
        src: String,
        target: String,
        size_bytes: u64,
        queue_id: usize,
    },
    Progress {
        queue_id: usize,
        bytes_transferred: u64,
        total_bytes: u64,
        speed_mbps: f64,
        eta_seconds: Option<u64>,
    },
    ItemDone {
        queue_id: usize,
        size_bytes: u64,
    },
    ItemFailed {
        queue_id: usize,
        error: String,
    },
    PromptError {
        queue_id: usize,
        filename: String,
        src: String,
        target: String,
        error: String,
        resp_tx: std::sync::mpsc::Sender<TransferErrorAction>,
    },
    DiscoveredItems {
        items: Vec<TransferQueueItem>,
    },
    Log {
        kind: ServerLogKind,
        message: String,
    },
    AllFinished,
}

fn check_and_mark_completed_parents(queue: &mut [TransferQueueItem]) {
    for _ in 0..10 {
        let mut changed = false;
        for i in 0..queue.len() {
            if queue[i].is_dir && queue[i].status != TransferStatus::Completed {
                let dir_id = queue[i].id;
                let mut child_count = 0;
                let mut completed_count = 0;
                let mut total_bytes = 0u64;
                for c in queue.iter() {
                    if c.parent_id == Some(dir_id) {
                        child_count += 1;
                        if c.status == TransferStatus::Completed {
                            completed_count += 1;
                        }
                        total_bytes += c.size_bytes;
                    }
                }
                if child_count > 0 && child_count == completed_count {
                    queue[i].status = TransferStatus::Completed;
                    queue[i].size_bytes = total_bytes;
                    queue[i].transferred_bytes = total_bytes;
                    changed = true;
                }
            }
        }
        if !changed {
            break;
        }
    }
}


fn scan_local_folder_for_queue(
    src_dir: &std::path::Path,
    dest_prefix: &str,
    batch_id: usize,
    parent_id: usize,
    direction: TransferDirection,
    protocol: Protocol,
    next_id: &mut usize,
    queue: &mut Vec<TransferQueueItem>,
) {
    if let Ok(entries) = std::fs::read_dir(src_dir) {
        for entry_res in entries.flatten() {
            let child_path = entry_res.path();
            let name = entry_res.file_name().to_string_lossy().to_string();
            if name == ".." || name == "." {
                continue;
            }
            let is_child_dir = entry_res.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
            let size = if is_child_dir {
                0
            } else {
                entry_res.metadata().map(|m| m.len()).unwrap_or(0)
            };
            let child_id = *next_id;
            *next_id += 1;
            let child_dest = format!("{}/{}", dest_prefix.trim_end_matches('/'), name);
            queue.push(TransferQueueItem {
                id: child_id,
                batch_id,
                filename: name.clone(),
                source_path: child_path.to_string_lossy().to_string(),
                target_path: child_dest.clone(),
                direction,
                protocol,
                size_bytes: size,
                transferred_bytes: 0,
                speed_mbps: 0.0,
                status: TransferStatus::Queued,
                is_dir: is_child_dir,
                parent_id: Some(parent_id),
            });
            if is_child_dir {
                scan_local_folder_for_queue(
                    &child_path,
                    &child_dest,
                    batch_id,
                    child_id,
                    direction,
                    protocol,
                    next_id,
                    queue,
                );
            }
        }
    }
}

fn scan_remote_folder_for_queue(
    protocol: Protocol,
    host: &str,
    port: u16,
    username: &str,
    password: Option<&str>,
    remote_dir: &str,
    local_dir: &std::path::Path,
    batch_id: usize,
    parent_id: usize,
    next_id: &mut usize,
    queue: &mut Vec<TransferQueueItem>,
    tx_log: &std::sync::mpsc::Sender<TransferProgressUpdate>,
) {
    let _ = tx_log.send(TransferProgressUpdate::Log {
        kind: ServerLogKind::Command,
        message: format!("CWD {}", remote_dir),
    });
    let _ = tx_log.send(TransferProgressUpdate::Log {
        kind: ServerLogKind::Command,
        message: "PASV".to_string(),
    });
    let _ = tx_log.send(TransferProgressUpdate::Log {
        kind: ServerLogKind::Command,
        message: "LIST".to_string(),
    });
    if let Ok(entries) = crate::network::list_remote_directory(protocol, host, port, username, password, remote_dir, true) {
        let _ = tx_log.send(TransferProgressUpdate::Log {
            kind: ServerLogKind::Response,
            message: format!("226 Nalezeno {} položek v {}", entries.len(), remote_dir),
        });
        for child in entries {
            if child.name == ".." || child.name == "." {
                continue;
            }
            let child_id = *next_id;
            *next_id += 1;
            let child_local = local_dir.join(&child.name);
            queue.push(TransferQueueItem {
                id: child_id,
                batch_id,
                filename: child.name.clone(),
                source_path: child.path.clone(),
                target_path: child_local.to_string_lossy().to_string(),
                direction: TransferDirection::Download,
                protocol,
                size_bytes: child.size,
                transferred_bytes: 0,
                speed_mbps: 0.0,
                status: TransferStatus::Queued,
                is_dir: child.is_dir,
                parent_id: Some(parent_id),
            });
            if child.is_dir {
                scan_remote_folder_for_queue(
                    protocol,
                    host,
                    port,
                    username,
                    password,
                    &child.path,
                    &child_local,
                    batch_id,
                    child_id,
                    next_id,
                    queue,
                    tx_log,
                );
            }
        }
    }
}

impl RusterView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let saved_state = load_app_state();
        let local_dir = saved_state.local_dir;
        let right_dir = saved_state.right_dir;
        let show_local_pane = saved_state.show_local_pane;
        let show_hidden = saved_state.show_hidden;
        let mut local_entries = read_local_directory(&local_dir, show_hidden);
        sort_entries(&mut local_entries, SortColumn::Name, true);
        let local_initial_idx = if local_entries.get(0).map(|e| e.name == "..").unwrap_or(false) && local_entries.len() > 1 { 1 } else { 0 };
        if let Some(first) = local_entries.get_mut(local_initial_idx) {
            if first.name != ".." {
                first.selected = true;
            }
        }
        let local_tree = build_local_tree(&local_dir, show_hidden);
        let mut right_entries = read_local_directory(&right_dir, show_hidden);
        sort_entries(&mut right_entries, SortColumn::Name, true);
        let right_initial_idx = if right_entries.get(0).map(|e| e.name == "..").unwrap_or(false) && right_entries.len() > 1 { 1 } else { 0 };
        if let Some(first) = right_entries.get_mut(right_initial_idx) {
            if first.name != ".." {
                first.selected = true;
            }
        }
        let right_tree = build_local_tree(&right_dir, show_hidden);

        let mut saved_sites = load_saved_sites().unwrap_or_else(default_saved_sites);
        saved_sites.sort_by(|a, b| {
            a.display_name()
                .to_lowercase()
                .cmp(&b.display_name().to_lowercase())
                .then_with(|| a.host.to_lowercase().cmp(&b.host.to_lowercase()))
                .then_with(|| a.username.to_lowercase().cmp(&b.username.to_lowercase()))
                .then_with(|| a.port.cmp(&b.port))
        });
        let draft_protocol = saved_sites
            .first()
            .map(|s| s.protocol)
            .unwrap_or(Protocol::Sftp);

        let view = Self {
            focus_handle: cx.focus_handle(),
            locale: saved_state.locale,
            theme: saved_state.theme,
            active_pane: ActivePane::Local,
            show_local_pane,
            swap_panes: false,
            show_local_tree: false,
            show_remote_tree: false,
            show_hidden,
            show_function_bar: saved_state.show_function_bar,
            show_permissions: saved_state.show_permissions,
            date_format: saved_state.date_format,
            show_queue: false,
            show_local_filter: false,
            local_filter_query: String::new(),
            show_remote_filter: false,
            remote_filter_query: String::new(),
            session_dropdown_open: false,
            context_menu: None,
            favorite_paths: crate::config::load_favorite_paths(),
            favorites_menu_pane: None,

            local_scroll: UniformListScrollHandle::new(),
            remote_scroll: UniformListScrollHandle::new(),
            console_scroll: ScrollHandle::new(),

            local_dir,
            local_entries,
            local_tree,
            local_selected: local_initial_idx,
            local_sort_col: SortColumn::Name,
            local_sort_asc: true,

            right_dir,
            right_entries,
            right_tree,
            right_selected: right_initial_idx,

            remote_flash: None,
            remote_flash_id: 0,

            sessions: Vec::new(),
            active_session_idx: 0,
            remote_sort_col: SortColumn::Name,
            remote_sort_asc: true,

            transfer_queue: Vec::new(),
            max_concurrent_transfers: saved_state.max_concurrent_transfers,
            next_transfer_id: 1,
            next_batch_id: 1,
            expanded_batches: std::collections::HashSet::new(),
            expanded_folders: std::collections::HashSet::new(),

            col_size_width: 95.0,
            col_modified_width: if saved_state.date_format == DateFormat::Date { 85.0 } else { 145.0 },
            col_permissions_width: 85.0,
            resizing_column: None,
            queue_panel_height: saved_state.queue_panel_height,
            resizing_queue_panel: None,

            active_modal: ActiveModal::None,
            modal_input_text: String::new(),
            saved_sites,
            selected_site_idx: 0,
            draft_protocol,
            active_site_field: SiteEditField::None,
            site_password_input: String::new(),
            site_cursor_pos: 0,
            modal_cursor_pos: 0,
            site_error_message: None,
            site_notice: None,
            site_is_connecting: false,
            remote_error_banner: None,
            active_transfer: None,
            transfer_cancel_signal: None,

            server_log: Vec::new(),
            bottom_tab: BottomPanelTab::Transfers,
            show_transfer_log: false,
            transfer_error_prompt: None,

            chmod_mode: 0o755,
            chmod_recursive: false,

            watched_remote_files: Vec::new(),
            editable_extensions: saved_state.editable_extensions,
            settings_editing_exts: false,
            settings_cursor_pos: 0,
            settings_lang_dropdown_open: false,

            update_status: None,
            update_copied_notice: false,
            sync_state: SyncState::default(),

            window_width: saved_state.window_width,
            window_height: saved_state.window_height,
            last_window_size_saved: std::time::Instant::now(),
        };

        cleanup_remote_edit_temp();
        Self::start_remote_edit_watcher(cx);
        view.apply_theme_to_window();
        view.trigger_background_update_check(cx);

        view
    }

    pub(crate) fn log_server(&mut self, kind: ServerLogKind, msg: impl Into<String>) {
        let time_str = unsafe {
            let t = libc::time(std::ptr::null_mut());
            let mut tm: libc::tm = std::mem::zeroed();
            libc::localtime_r(&t, &mut tm);
            format!("{:02}:{:02}:{:02}", tm.tm_hour, tm.tm_min, tm.tm_sec)
        };
        self.server_log.push(ServerLogEntry {
            time: time_str,
            kind,
            message: msg.into(),
        });
        while self.server_log.len() > 200 {
            self.server_log.remove(0);
        }
        let last_idx = self.server_log.len().saturating_sub(1);
        self.console_scroll.scroll_to_item(last_idx);
    }

    pub fn i18n(&self) -> I18n {
        I18n::new(self.locale)
    }

    pub(crate) fn reload_local(&mut self) {
        self.local_entries = read_local_directory(&self.local_dir, self.show_hidden);
        self.local_tree = build_local_tree(&self.local_dir, self.show_hidden);
        sort_entries(
            &mut self.local_entries,
            self.local_sort_col,
            self.local_sort_asc,
        );
        if self.local_selected >= self.local_entries.len() {
            self.local_selected = self.local_entries.len().saturating_sub(1);
        }
        if !self.local_entries.iter().any(|e| e.selected) {
            if let Some(e) = self.local_entries.get_mut(self.local_selected) {
                if e.name != ".." {
                    e.selected = true;
                }
            }
        }
    }

    pub(crate) fn filtered_local_indices(&self) -> Vec<usize> {
        let q = if self.show_local_filter {
            self.local_filter_query.trim().to_lowercase()
        } else {
            String::new()
        };
        self.local_entries
            .iter()
            .enumerate()
            .filter(|(_, e)| {
                q.is_empty() || e.name == ".." || e.name.to_lowercase().contains(&q)
            })
            .map(|(i, _)| i)
            .collect()
    }

    pub(crate) fn filtered_remote_indices(&self) -> Vec<usize> {
        let q = if self.show_remote_filter {
            self.remote_filter_query.trim().to_lowercase()
        } else {
            String::new()
        };
        if self.sessions.is_empty() {
            self.right_entries
                .iter()
                .enumerate()
                .filter(|(_, e)| {
                    q.is_empty() || e.name == ".." || e.name.to_lowercase().contains(&q)
                })
                .map(|(i, _)| i)
                .collect()
        } else if let Some(session) = self.sessions.get(self.active_session_idx) {
            session
                .entries
                .iter()
                .enumerate()
                .filter(|(_, e)| {
                    q.is_empty() || e.name == ".." || e.name.to_lowercase().contains(&q)
                })
                .map(|(i, _)| i)
                .collect()
        } else {
            Vec::new()
        }
    }

    pub(crate) fn ensure_local_selection(&mut self) {
        let visible = self.filtered_local_indices();
        if visible.is_empty() {
            return;
        }
        if !visible.contains(&self.local_selected) {
            let first_non_parent = visible.iter().copied().find(|&idx| {
                self.local_entries.get(idx).map(|e| e.name != "..").unwrap_or(false)
            });
            self.local_selected = first_non_parent.unwrap_or(visible[0]);
        }
        for (i, e) in self.local_entries.iter_mut().enumerate() {
            e.selected = (i == self.local_selected) && e.name != "..";
        }
    }

    pub(crate) fn ensure_remote_selection(&mut self) {
        let visible = self.filtered_remote_indices();
        if visible.is_empty() {
            return;
        }
        if !visible.contains(&self.right_selected) {
            if self.sessions.is_empty() {
                let first_non_parent = visible.iter().copied().find(|&idx| {
                    self.right_entries.get(idx).map(|e| e.name != "..").unwrap_or(false)
                });
                self.right_selected = first_non_parent.unwrap_or(visible[0]);
            } else if let Some(session) = self.sessions.get(self.active_session_idx) {
                let first_non_parent = visible.iter().copied().find(|&idx| {
                    session.entries.get(idx).map(|e| e.name != "..").unwrap_or(false)
                });
                self.right_selected = first_non_parent.unwrap_or(visible[0]);
            }
        }
        if self.sessions.is_empty() {
            for (i, e) in self.right_entries.iter_mut().enumerate() {
                e.selected = (i == self.right_selected) && e.name != "..";
            }
        } else if let Some(session) = self.sessions.get_mut(self.active_session_idx) {
            for (i, e) in session.entries.iter_mut().enumerate() {
                e.selected = (i == self.right_selected) && e.name != "..";
            }
        }
    }

    pub(crate) fn reload_right(&mut self) {
        if self.sessions.is_empty() {
            self.right_entries = read_local_directory(&self.right_dir, self.show_hidden);
            self.right_tree = build_local_tree(&self.right_dir, self.show_hidden);
            sort_entries(
                &mut self.right_entries,
                self.remote_sort_col,
                self.remote_sort_asc,
            );
            if self.right_selected >= self.right_entries.len() {
                self.right_selected = self.right_entries.len().saturating_sub(1);
            }
            if !self.right_entries.iter().any(|e| e.selected) {
                if let Some(e) = self.right_entries.get_mut(self.right_selected) {
                    if e.name != ".." {
                        e.selected = true;
                    }
                }
            }
        } else if let Some(session) = self.sessions.get_mut(self.active_session_idx) {
            let path = session.current_path.clone();
            let show_hidden = self.show_hidden;
            if let Ok(real_entries) = list_remote_directory(
                session.protocol,
                &session.host,
                session.port,
                &session.username,
                session.password.as_deref(),
                &path,
                show_hidden,
            ) {
                session.entries = real_entries;
                sort_entries(
                    &mut session.entries,
                    self.remote_sort_col,
                    self.remote_sort_asc,
                );
                if !session.entries.iter().any(|e| e.selected) {
                    if let Some(e) = session.entries.get_mut(session.selected_index) {
                        if e.name != ".." {
                            e.selected = true;
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn navigate_local(&mut self, path: PathBuf) {
        let expanded = if path.to_string_lossy().starts_with('~') {
            if let Ok(home) = std::env::var("HOME") {
                let s = path.to_string_lossy();
                let sub = s.trim_start_matches('~').trim_start_matches('/');
                PathBuf::from(home).join(sub)
            } else {
                path
            }
        } else {
            path
        };
        if expanded.exists() && expanded.is_dir() {
            self.local_dir = expanded;
            self.local_selected = 0;
            self.reload_local();
            self.local_scroll.scroll_to_item(0, ScrollStrategy::Top);
            self.persist_app_state();
        }
    }

    pub(crate) fn navigate_right_local(&mut self, path: PathBuf) {
        let expanded = if path.to_string_lossy().starts_with('~') {
            if let Ok(home) = std::env::var("HOME") {
                let s = path.to_string_lossy();
                let sub = s.trim_start_matches('~').trim_start_matches('/');
                PathBuf::from(home).join(sub)
            } else {
                path
            }
        } else {
            path
        };
        if expanded.exists() && expanded.is_dir() {
            self.right_dir = expanded;
            self.right_selected = 0;
            self.reload_right();
            self.remote_scroll.scroll_to_item(0, ScrollStrategy::Top);
            self.persist_app_state();
        }
    }

    pub fn theme(&self) -> crate::theme::Theme {
        crate::theme::Theme::new(self.theme)
    }

    pub(crate) fn add_favorite_path(&mut self, path: PathBuf) {
        if !self.favorite_paths.contains(&path) {
            self.favorite_paths.push(path);
            let _ = crate::config::save_favorite_paths(&self.favorite_paths);
        }
    }

    pub(crate) fn remove_favorite_path_by_value(&mut self, path: &std::path::Path) {
        self.favorite_paths.retain(|p| p != path);
        let _ = crate::config::save_favorite_paths(&self.favorite_paths);
    }

    pub fn set_theme(&mut self, theme: crate::theme::ThemeMode, cx: &mut Context<Self>) {
        self.theme = theme;
        self.apply_theme_to_window();
        self.persist_app_state();
        cx.notify();
    }

    pub fn toggle_theme(&mut self, cx: &mut Context<Self>) {
        let next = self.theme.toggle();
        self.set_theme(next, cx);
    }

    pub fn set_locale(&mut self, locale: crate::i18n::Locale, cx: &mut Context<Self>) {
        self.locale = locale;
        install_app_menus(cx, self.locale);
        self.persist_app_state();
        cx.notify();
    }

    pub fn apply_theme_to_window(&self) {
        crate::set_macos_window_appearance(self.theme.is_dark());
    }

    pub fn trigger_background_update_check(&self, cx: &mut Context<Self>) {
        cx.spawn(move |this: gpui::WeakEntity<RusterView>, cx: &mut gpui::AsyncApp| {
            let mut cx = cx.clone();
            async move {
                let res = std::thread::spawn(|| crate::update::fetch_latest_release()).join();
                if let Ok(Ok(Some(info))) = res {
                    let _ = this.update(&mut cx, |view, cx| {
                        view.update_status = Some(crate::update::UpdateStatus::Available(info));
                        view.active_modal = ActiveModal::Update;
                        cx.notify();
                    });
                }
            }
        })
        .detach();
    }

    pub fn check_for_updates(&mut self, user_initiated: bool, cx: &mut Context<Self>) {
        self.update_copied_notice = false;
        if user_initiated {
            self.active_modal = ActiveModal::Update;
            self.update_status = Some(crate::update::UpdateStatus::Checking);
            cx.notify();
        }

        cx.spawn(move |this: gpui::WeakEntity<RusterView>, cx: &mut gpui::AsyncApp| {
            let mut cx = cx.clone();
            async move {
                let res = std::thread::spawn(|| crate::update::fetch_latest_release()).join();
                let _ = this.update(&mut cx, |view, cx| {
                    match res {
                        Ok(Ok(Some(info))) => {
                            view.update_status = Some(crate::update::UpdateStatus::Available(info));
                            view.active_modal = ActiveModal::Update;
                        }
                        Ok(Ok(None)) => {
                            if user_initiated {
                                view.update_status = Some(crate::update::UpdateStatus::UpToDate(
                                    env!("CARGO_PKG_VERSION").to_string(),
                                ));
                                view.active_modal = ActiveModal::Update;
                            }
                        }
                        Ok(Err(err)) => {
                            if user_initiated {
                                view.update_status = Some(crate::update::UpdateStatus::Error(err));
                                view.active_modal = ActiveModal::Update;
                            }
                        }
                        Err(_) => {
                            if user_initiated {
                                view.update_status = Some(crate::update::UpdateStatus::Error(
                                    "Failed to check for updates".to_string(),
                                ));
                                view.active_modal = ActiveModal::Update;
                            }
                        }
                    }
                    cx.notify();
                });
            }
        })
        .detach();
    }

    pub fn copy_brew_command(&mut self) {
        self.update_copied_notice = true;
        let cmd = std::process::Command::new("pbcopy")
            .stdin(std::process::Stdio::piped())
            .spawn();
        if let Ok(mut child) = cmd {
            if let Some(mut stdin) = child.stdin.take() {
                use std::io::Write;
                let _ = stdin.write_all(b"brew update && brew upgrade --cask ruster");
            }
            let _ = child.wait();
        }
    }

    pub fn start_in_app_update(&mut self, version: String, cx: &mut Context<Self>) {
        self.update_status = Some(crate::update::UpdateStatus::Updating {
            version: version.clone(),
            message: "Příprava aktualizace...".to_string(),
        });
        cx.notify();

        let (tx, rx) = std::sync::mpsc::channel::<Result<String, String>>();
        let ver = version.clone();

        std::thread::spawn(move || {
            let res = crate::update::perform_in_app_update(&ver, |msg| {
                let _ = tx.send(Ok(msg));
            });
            match res {
                Ok(()) => {
                    let _ = tx.send(Ok("__DONE__".to_string()));
                }
                Err(e) => {
                    let _ = tx.send(Err(e));
                }
            }
        });

        cx.spawn(move |this: gpui::WeakEntity<RusterView>, cx: &mut gpui::AsyncApp| {
            let mut cx = cx.clone();
            async move {
                loop {
                    let mut got_msg = false;
                    while let Ok(msg) = rx.try_recv() {
                        got_msg = true;
                        match msg {
                            Ok(text) if text == "__DONE__" => {
                                let _ = this.update(&mut cx, |view, cx| {
                                    view.update_status = Some(crate::update::UpdateStatus::ReadyToRestart {
                                        version: version.clone(),
                                    });
                                    cx.notify();
                                });
                                return;
                            }
                            Ok(progress_text) => {
                                let _ = this.update(&mut cx, |view, cx| {
                                    view.update_status = Some(crate::update::UpdateStatus::Updating {
                                        version: version.clone(),
                                        message: progress_text,
                                    });
                                    cx.notify();
                                });
                            }
                            Err(err) => {
                                let _ = this.update(&mut cx, |view, cx| {
                                    view.update_status = Some(crate::update::UpdateStatus::Error(err));
                                    cx.notify();
                                });
                                return;
                            }
                        }
                    }

                    if !got_msg {
                        cx.background_executor().timer(std::time::Duration::from_millis(50)).await;
                    }
                }
            }
        }).detach();
    }

    pub(crate) fn persist_app_state(&self) {
        let _ = save_app_state(
            self.show_local_pane,
            self.show_hidden,
            self.show_function_bar,
            self.show_permissions,
            self.date_format,
            &self.local_dir,
            &self.right_dir,
            self.max_concurrent_transfers,
            &self.editable_extensions,
            self.window_width,
            self.window_height,
            self.queue_panel_height,
            self.theme,
            self.locale,
        );
    }

    pub(crate) fn open_go_to_folder_modal(&mut self) {
        self.modal_input_text = match self.active_pane {
            ActivePane::Local => self.local_dir.to_string_lossy().to_string(),
            ActivePane::Remote => {
                if self.sessions.is_empty() {
                    self.right_dir.to_string_lossy().to_string()
                } else {
                    self.sessions
                        .get(self.active_session_idx)
                        .map(|s| s.current_path.clone())
                        .unwrap_or_else(|| "/".to_string())
                }
            }
        };
        self.modal_cursor_pos = self.modal_input_text.chars().count();
        self.active_modal = ActiveModal::GoToFolder;
    }

    pub(crate) fn confirm_go_to_folder(&mut self) {
        let text = self.modal_input_text.trim().to_string();
        if !text.is_empty() {
            match self.active_pane {
                ActivePane::Local => {
                    self.navigate_local(PathBuf::from(text));
                }
                ActivePane::Remote => {
                    if self.sessions.is_empty() {
                        self.navigate_right_local(PathBuf::from(text));
                    } else {
                        self.navigate_remote(text);
                    }
                }
            }
        }
        self.active_modal = ActiveModal::None;
    }

    pub(crate) fn navigate_remote(&mut self, path: String) {
        if self.sessions.is_empty() {
            self.navigate_right_local(PathBuf::from(path));
            return;
        }
        self.show_queue = true;
        self.log_server(ServerLogKind::Command, format!("CWD {}", path));
        self.log_server(ServerLogKind::Command, "PASV".to_string());
        self.log_server(ServerLogKind::Command, "LIST".to_string());

        let show_hidden = self.show_hidden;
        let session_info = self.sessions.get(self.active_session_idx).map(|s| {
            (s.protocol, s.host.clone(), s.port, s.username.clone(), s.password.clone())
        });

        if let Some((proto, host, port, user, pwd)) = session_info {
            match list_remote_directory(
                proto,
                &host,
                port,
                &user,
                pwd.as_deref(),
                &path,
                show_hidden,
            ) {
                Ok(real_entries) => {
                    self.log_server(ServerLogKind::Response, format!("226 Přenos adresáře dokončen ({} položek)", real_entries.len()));
                    if let Some(session) = self.sessions.get_mut(self.active_session_idx) {
                        session.current_path = path;
                        session.entries = real_entries;
                        sort_entries(
                            &mut session.entries,
                            self.remote_sort_col,
                            self.remote_sort_asc,
                        );
                        session.selected_index = 0;
                    }
                    self.remote_scroll.scroll_to_item(0, ScrollStrategy::Top);
                    self.remote_error_banner = None;
                }
                Err(err) => {
                    self.log_server(ServerLogKind::Error, format!("Chyba při načítání složky: {}", err));
                    self.remote_error_banner = Some(format!("Nelze otevřít složku {}: {}", path, err));
                }
            }
        }
    }

    pub(crate) fn move_selection(&mut self, delta: isize) {
        match self.active_pane {
            ActivePane::Local => {
                for e in &mut self.local_entries {
                    e.selected = false;
                }
                let visible = self.filtered_local_indices();
                if visible.is_empty() {
                    return;
                }
                let current_pos = visible.iter().position(|&idx| idx == self.local_selected);
                let next_pos = match current_pos {
                    Some(pos) => (pos as isize + delta).clamp(0, visible.len() as isize - 1) as usize,
                    None => if delta > 0 { 0 } else { visible.len() - 1 },
                };
                let next_idx = visible[next_pos];
                self.local_selected = next_idx;
                if let Some(e) = self.local_entries.get_mut(next_idx) {
                    if e.name != ".." {
                        e.selected = true;
                    }
                }
                self.local_scroll.scroll_to_item(next_pos, ScrollStrategy::Top);
            }
            ActivePane::Remote => {
                let visible = self.filtered_remote_indices();
                if visible.is_empty() {
                    return;
                }
                if self.sessions.is_empty() {
                    for e in &mut self.right_entries {
                        e.selected = false;
                    }
                    let current_pos = visible.iter().position(|&idx| idx == self.right_selected);
                    let next_pos = match current_pos {
                        Some(pos) => (pos as isize + delta).clamp(0, visible.len() as isize - 1) as usize,
                        None => if delta > 0 { 0 } else { visible.len() - 1 },
                    };
                    let next_idx = visible[next_pos];
                    self.right_selected = next_idx;
                    if let Some(e) = self.right_entries.get_mut(next_idx) {
                        if e.name != ".." {
                            e.selected = true;
                        }
                    }
                    self.remote_scroll.scroll_to_item(next_pos, ScrollStrategy::Top);
                } else if let Some(s) = self.sessions.get_mut(self.active_session_idx) {
                    for e in &mut s.entries {
                        e.selected = false;
                    }
                    let current_pos = visible.iter().position(|&idx| idx == s.selected_index);
                    let next_pos = match current_pos {
                        Some(pos) => (pos as isize + delta).clamp(0, visible.len() as isize - 1) as usize,
                        None => if delta > 0 { 0 } else { visible.len() - 1 },
                    };
                    let next_idx = visible[next_pos];
                    s.selected_index = next_idx;
                    if let Some(e) = s.entries.get_mut(next_idx) {
                        if e.name != ".." {
                            e.selected = true;
                        }
                    }
                    self.remote_scroll.scroll_to_item(next_pos, ScrollStrategy::Top);
                }
            }
        }
    }

    pub(crate) fn toggle_space_mark(&mut self, direction: isize, cx: &mut Context<Self>) {
        match self.active_pane {
            ActivePane::Local => {
                let idx = self.local_selected;
                if let Some(entry) = self.local_entries.get_mut(idx) {
                    if entry.name != ".." {
                        entry.space_marked = !entry.space_marked;
                        if entry.is_dir && entry.space_marked {
                            entry.calculating_size = true;
                            entry.calculated_dir_size = None;
                            let path = PathBuf::from(&entry.path);
                            let target_path = entry.path.clone();
                            let (tx, rx) = std::sync::mpsc::channel::<u64>();
                            std::thread::spawn(move || {
                                let size = calculate_local_dir_size(&path);
                                let _ = tx.send(size);
                            });
                            cx.spawn(move |this: gpui::WeakEntity<RusterView>, cx: &mut gpui::AsyncApp| {
                                let mut cx = cx.clone();
                                async move {
                                    loop {
                                        match rx.try_recv() {
                                            Ok(size) => {
                                                let _ = this.update(&mut cx, |view, cx| {
                                                    if let Some(e) = view.local_entries.iter_mut().find(|e| e.path == target_path) {
                                                        e.calculating_size = false;
                                                        if e.space_marked {
                                                            e.calculated_dir_size = Some(size);
                                                        }
                                                    }
                                                    cx.notify();
                                                });
                                                break;
                                            }
                                            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                                                let _ = this.update(&mut cx, |view, cx| {
                                                    if let Some(e) = view.local_entries.iter_mut().find(|e| e.path == target_path) {
                                                        e.calculating_size = false;
                                                    }
                                                    cx.notify();
                                                });
                                                break;
                                            }
                                            Err(std::sync::mpsc::TryRecvError::Empty) => {}
                                        }
                                        cx.background_executor().timer(std::time::Duration::from_millis(60)).await;
                                    }
                                }
                            }).detach();
                        } else if entry.is_dir && !entry.space_marked {
                            entry.calculating_size = false;
                            entry.calculated_dir_size = None;
                        }
                    }
                }
            }
            ActivePane::Remote => {
                if self.sessions.is_empty() {
                    let idx = self.right_selected;
                    if let Some(entry) = self.right_entries.get_mut(idx) {
                        if entry.name != ".." {
                            entry.space_marked = !entry.space_marked;
                            if entry.is_dir && entry.space_marked {
                                entry.calculating_size = true;
                                entry.calculated_dir_size = None;
                                let path = PathBuf::from(&entry.path);
                                let target_path = entry.path.clone();
                                let (tx, rx) = std::sync::mpsc::channel::<u64>();
                                std::thread::spawn(move || {
                                    let size = calculate_local_dir_size(&path);
                                    let _ = tx.send(size);
                                });
                                cx.spawn(move |this: gpui::WeakEntity<RusterView>, cx: &mut gpui::AsyncApp| {
                                    let mut cx = cx.clone();
                                    async move {
                                        loop {
                                            match rx.try_recv() {
                                                Ok(size) => {
                                                    let _ = this.update(&mut cx, |view, cx| {
                                                        if let Some(e) = view.right_entries.iter_mut().find(|e| e.path == target_path) {
                                                            e.calculating_size = false;
                                                            if e.space_marked {
                                                                e.calculated_dir_size = Some(size);
                                                            }
                                                        }
                                                        cx.notify();
                                                    });
                                                    break;
                                                }
                                                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                                                    let _ = this.update(&mut cx, |view, cx| {
                                                        if let Some(e) = view.right_entries.iter_mut().find(|e| e.path == target_path) {
                                                            e.calculating_size = false;
                                                        }
                                                        cx.notify();
                                                    });
                                                    break;
                                                }
                                                Err(std::sync::mpsc::TryRecvError::Empty) => {}
                                            }
                                            cx.background_executor().timer(std::time::Duration::from_millis(60)).await;
                                        }
                                    }
                                }).detach();
                            } else if entry.is_dir && !entry.space_marked {
                                entry.calculating_size = false;
                                entry.calculated_dir_size = None;
                            }
                        }
                    }
                } else if let Some(s) = self.sessions.get_mut(self.active_session_idx) {
                    let idx = s.selected_index;
                    let session_idx = self.active_session_idx;
                    let proto = s.protocol;
                    let host = s.host.clone();
                    let port = s.port;
                    let user = s.username.clone();
                    let pass = s.password.clone();
                    if let Some(entry) = s.entries.get_mut(idx) {
                        if entry.name != ".." {
                            entry.space_marked = !entry.space_marked;
                            if entry.is_dir && entry.space_marked {
                                entry.calculating_size = true;
                                entry.calculated_dir_size = None;
                                let target_path = entry.path.clone();
                                let (tx, rx) = std::sync::mpsc::channel::<Option<u64>>();
                                let p = target_path.clone();
                                std::thread::spawn(move || {
                                    let res = calculate_remote_dir_size(
                                        proto,
                                        &host,
                                        port,
                                        &user,
                                        pass.as_deref(),
                                        &p,
                                    ).ok();
                                    let _ = tx.send(res);
                                });
                                cx.spawn(move |this: gpui::WeakEntity<RusterView>, cx: &mut gpui::AsyncApp| {
                                    let mut cx = cx.clone();
                                    async move {
                                        loop {
                                            match rx.try_recv() {
                                                Ok(maybe_size) => {
                                                    let _ = this.update(&mut cx, |view, cx| {
                                                        if let Some(session) = view.sessions.get_mut(session_idx) {
                                                            if let Some(e) = session.entries.iter_mut().find(|e| e.path == target_path) {
                                                                e.calculating_size = false;
                                                                if e.space_marked {
                                                                    e.calculated_dir_size = maybe_size;
                                                                }
                                                            }
                                                        }
                                                        cx.notify();
                                                    });
                                                    break;
                                                }
                                                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                                                    let _ = this.update(&mut cx, |view, cx| {
                                                        if let Some(session) = view.sessions.get_mut(session_idx) {
                                                            if let Some(e) = session.entries.iter_mut().find(|e| e.path == target_path) {
                                                                e.calculating_size = false;
                                                            }
                                                        }
                                                        cx.notify();
                                                    });
                                                    break;
                                                }
                                                Err(std::sync::mpsc::TryRecvError::Empty) => {}
                                            }
                                            cx.background_executor().timer(std::time::Duration::from_millis(60)).await;
                                        }
                                    }
                                }).detach();
                            } else if entry.is_dir && !entry.space_marked {
                                entry.calculating_size = false;
                                entry.calculated_dir_size = None;
                            }
                        }
                    }
                }
            }
        }
        self.move_selection(direction);
    }

    pub(crate) fn select_range_keyboard(&mut self, direction: isize) {
        match self.active_pane {
            ActivePane::Local => {
                let visible = self.filtered_local_indices();
                if visible.is_empty() {
                    return;
                }
                if let Some(e) = self.local_entries.get_mut(self.local_selected) {
                    if e.name != ".." {
                        e.selected = true;
                    }
                }
                let current_pos = visible.iter().position(|&idx| idx == self.local_selected);
                let next_pos = match current_pos {
                    Some(pos) => (pos as isize + direction).clamp(0, visible.len() as isize - 1) as usize,
                    None => if direction > 0 { 0 } else { visible.len() - 1 },
                };
                let next_idx = visible[next_pos];
                self.local_selected = next_idx;
                if let Some(e) = self.local_entries.get_mut(next_idx) {
                    if e.name != ".." {
                        e.selected = true;
                    }
                }
                self.local_scroll.scroll_to_item(next_pos, ScrollStrategy::Top);
            }
            ActivePane::Remote => {
                let visible = self.filtered_remote_indices();
                if visible.is_empty() {
                    return;
                }
                if self.sessions.is_empty() {
                    if let Some(e) = self.right_entries.get_mut(self.right_selected) {
                        if e.name != ".." {
                            e.selected = true;
                        }
                    }
                    let current_pos = visible.iter().position(|&idx| idx == self.right_selected);
                    let next_pos = match current_pos {
                        Some(pos) => (pos as isize + direction).clamp(0, visible.len() as isize - 1) as usize,
                        None => if direction > 0 { 0 } else { visible.len() - 1 },
                    };
                    let next_idx = visible[next_pos];
                    self.right_selected = next_idx;
                    if let Some(e) = self.right_entries.get_mut(next_idx) {
                        if e.name != ".." {
                            e.selected = true;
                        }
                    }
                    self.remote_scroll.scroll_to_item(next_pos, ScrollStrategy::Top);
                } else if let Some(s) = self.sessions.get_mut(self.active_session_idx) {
                    if let Some(e) = s.entries.get_mut(s.selected_index) {
                        if e.name != ".." {
                            e.selected = true;
                        }
                    }
                    let current_pos = visible.iter().position(|&idx| idx == s.selected_index);
                    let next_pos = match current_pos {
                        Some(pos) => (pos as isize + direction).clamp(0, visible.len() as isize - 1) as usize,
                        None => if direction > 0 { 0 } else { visible.len() - 1 },
                    };
                    let next_idx = visible[next_pos];
                    s.selected_index = next_idx;
                    if let Some(e) = s.entries.get_mut(next_idx) {
                        if e.name != ".." {
                            e.selected = true;
                        }
                    }
                    self.remote_scroll.scroll_to_item(next_pos, ScrollStrategy::Top);
                }
            }
        }
    }

    pub(crate) fn select_range_local(&mut self, target_idx: usize) {
        let start = self.local_selected.min(target_idx);
        let end = self.local_selected.max(target_idx);
        for (i, e) in self.local_entries.iter_mut().enumerate() {
            if i >= start && i <= end && e.name != ".." {
                e.selected = true;
            } else {
                e.selected = false;
            }
        }
        self.local_selected = target_idx;
    }

    pub(crate) fn select_range_remote(&mut self, target_idx: usize) {
        if self.sessions.is_empty() {
            let start = self.right_selected.min(target_idx);
            let end = self.right_selected.max(target_idx);
            for (i, e) in self.right_entries.iter_mut().enumerate() {
                if i >= start && i <= end && e.name != ".." {
                    e.selected = true;
                } else {
                    e.selected = false;
                }
            }
            self.right_selected = target_idx;
        } else if let Some(s) = self.sessions.get_mut(self.active_session_idx) {
            let start = s.selected_index.min(target_idx);
            let end = s.selected_index.max(target_idx);
            for (i, e) in s.entries.iter_mut().enumerate() {
                if i >= start && i <= end && e.name != ".." {
                    e.selected = true;
                } else {
                    e.selected = false;
                }
            }
            s.selected_index = target_idx;
        }
    }

    pub(crate) fn select_all_in_active_pane(&mut self) {
        match self.active_pane {
            ActivePane::Local => {
                for entry in &mut self.local_entries {
                    if entry.name != ".." {
                        entry.selected = true;
                    }
                }
            }
            ActivePane::Remote => {
                if self.sessions.is_empty() {
                    for entry in &mut self.right_entries {
                        if entry.name != ".." {
                            entry.selected = true;
                        }
                    }
                } else if let Some(s) = self.sessions.get_mut(self.active_session_idx) {
                    for entry in &mut s.entries {
                        if entry.name != ".." {
                            entry.selected = true;
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn open_selected_item(&mut self, cx: &mut Context<Self>) {
        match self.active_pane {
            ActivePane::Local => {
                if let Some(entry) = self.local_entries.get(self.local_selected).cloned() {
                    let p = PathBuf::from(&entry.path);
                    if entry.is_dir {
                        self.navigate_local(p);
                    } else {
                        cx.open_with_system(&p);
                    }
                }
            }
            ActivePane::Remote => {
                if self.sessions.is_empty() {
                    if let Some(entry) = self.right_entries.get(self.right_selected).cloned() {
                        let p = PathBuf::from(&entry.path);
                        if entry.is_dir {
                            self.navigate_right_local(p);
                        } else {
                            cx.open_with_system(&p);
                        }
                    }
                } else if let Some(s) = self.sessions.get(self.active_session_idx) {
                    if let Some(entry) = s.entries.get(s.selected_index).cloned() {
                        if entry.is_dir || entry.is_symlink {
                            self.navigate_remote(entry.path);
                        } else {
                            self.open_remote_file_for_editing(entry, None, cx);
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn open_selected_item_with_app(&mut self, cx: &mut Context<Self>) {
        let app_path = match Self::pick_application_prompt() {
            Some(p) => p,
            None => return,
        };

        match self.active_pane {
            ActivePane::Local => {
                if let Some(entry) = self.local_entries.get(self.local_selected).cloned() {
                    let p = PathBuf::from(&entry.path);
                    let _ = std::process::Command::new("open")
                        .arg("-a")
                        .arg(&app_path)
                        .arg(&p)
                        .spawn();
                }
            }
            ActivePane::Remote => {
                if self.sessions.is_empty() {
                    if let Some(entry) = self.right_entries.get(self.right_selected).cloned() {
                        let p = PathBuf::from(&entry.path);
                        let _ = std::process::Command::new("open")
                            .arg("-a")
                            .arg(&app_path)
                            .arg(&p)
                            .spawn();
                    }
                } else if let Some(s) = self.sessions.get(self.active_session_idx) {
                    if let Some(entry) = s.entries.get(s.selected_index).cloned() {
                        if !entry.is_dir {
                            self.open_remote_file_for_editing(entry, Some(app_path), cx);
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn navigate_up(&mut self) {
        match self.active_pane {
            ActivePane::Local => {
                if let Some(parent) = self.local_dir.parent().map(|p| p.to_path_buf()) {
                    self.navigate_local(parent);
                }
            }
            ActivePane::Remote => {
                if self.sessions.is_empty() {
                    if let Some(parent) = self.right_dir.parent().map(|p| p.to_path_buf()) {
                        self.navigate_right_local(parent);
                    }
                } else {
                    let parent = self
                        .sessions
                        .get(self.active_session_idx)
                        .and_then(|s| s.entries.first())
                        .filter(|e| e.name == "..")
                        .map(|e| e.path.clone());
                    if let Some(p) = parent {
                        self.navigate_remote(p);
                    }
                }
            }
        }
    }

    pub(crate) fn reveal_selected_in_finder(&self, cx: &mut Context<Self>) {
        let entry = match self.active_pane {
            ActivePane::Local => self.local_entries.get(self.local_selected),
            ActivePane::Remote => {
                if self.sessions.is_empty() {
                    self.right_entries.get(self.right_selected)
                } else {
                    None
                }
            }
        };
        if let Some(entry) = entry {
            cx.reveal_path(std::path::Path::new(&entry.path));
        } else {
            let dir = match self.active_pane {
                ActivePane::Local => Some(&self.local_dir),
                ActivePane::Remote if self.sessions.is_empty() => Some(&self.right_dir),
                _ => None,
            };
            if let Some(dir) = dir {
                cx.reveal_path(dir);
            }
        }
    }

    pub(crate) fn open_selected_in_terminal(&self, _cx: &mut Context<Self>) {
        let target_dir: Option<PathBuf> = match self.active_pane {
            ActivePane::Local => {
                if let Some(entry) = self.local_entries.get(self.local_selected) {
                    let p = std::path::Path::new(&entry.path);
                    if entry.is_dir {
                        Some(p.to_path_buf())
                    } else {
                        p.parent().map(|p| p.to_path_buf()).or_else(|| Some(self.local_dir.clone()))
                    }
                } else {
                    Some(self.local_dir.clone())
                }
            }
            ActivePane::Remote => {
                if self.sessions.is_empty() {
                    if let Some(entry) = self.right_entries.get(self.right_selected) {
                        let p = std::path::Path::new(&entry.path);
                        if entry.is_dir {
                            Some(p.to_path_buf())
                        } else {
                            p.parent().map(|p| p.to_path_buf()).or_else(|| Some(self.right_dir.clone()))
                        }
                    } else {
                        Some(self.right_dir.clone())
                    }
                } else {
                    None
                }
            }
        };

        if let Some(dir) = target_dir {
            #[cfg(target_os = "macos")]
            {
                let _ = std::process::Command::new("open")
                    .arg("-a")
                    .arg("Terminal")
                    .arg(&dir)
                    .spawn();
            }
            #[cfg(not(target_os = "macos"))]
            {
                let _ = std::process::Command::new("x-terminal-emulator")
                    .arg("--working-directory")
                    .arg(&dir)
                    .spawn();
            }
        }
    }

    pub(crate) fn copy_items(
        &mut self,
        items: Vec<FileEntry>,
        from_pane: ActivePane,
        remove_source: bool,
        cx: &mut Context<Self>,
    ) {
        if items.is_empty() {
            return;
        }

        let direction = match from_pane {
            ActivePane::Local => TransferDirection::Upload,
            ActivePane::Remote => TransferDirection::Download,
        };

        let batch_id = self.next_batch_id;
        self.next_batch_id += 1;

        let mut tasks = Vec::new();

        match from_pane {
            ActivePane::Local => {
                if self.sessions.is_empty() {
                    for entry in &items {
                        let qid = self.next_transfer_id;
                        self.next_transfer_id += 1;
                        let src = PathBuf::from(&entry.path);
                        let dest = self.right_dir.join(&entry.name);
                        let is_dir = entry.is_dir;
                        self.transfer_queue.push(TransferQueueItem {
                            id: qid,
                            batch_id,
                            filename: entry.name.clone(),
                            source_path: entry.path.clone(),
                            target_path: dest.to_string_lossy().to_string(),
                            direction: TransferDirection::Upload,
                            protocol: Protocol::Sftp,
                            size_bytes: entry.size,
                            transferred_bytes: 0,
                            speed_mbps: 0.0,
                            status: TransferStatus::Queued,
                            is_dir,
                            parent_id: None,
                        });
                        if is_dir {
                            scan_local_folder_for_queue(
                                &src,
                                &dest.to_string_lossy(),
                                batch_id,
                                qid,
                                TransferDirection::Upload,
                                Protocol::Sftp,
                                &mut self.next_transfer_id,
                                &mut self.transfer_queue,
                            );
                        }
                        tasks.push(TransferTask {
                            queue_id: qid,
                            filename: entry.name.clone(),
                            source_display: entry.path.clone(),
                            target_display: dest.to_string_lossy().to_string(),
                            size_bytes: entry.size,
                            kind: TransferKind::Local {
                                src,
                                dest,
                                remove_source,
                            },
                        });
                    }
                } else if let Some(session) = self.sessions.get(self.active_session_idx) {
                    let protocol = session.protocol;
                    let host = session.host.clone();
                    let port = session.port;
                    let username = session.username.clone();
                    let target_dir = session.current_path.clone();
                    let password = session.password.clone();

                    for entry in &items {
                        let qid = self.next_transfer_id;
                        self.next_transfer_id += 1;
                        let target_path = format!("{}/{}", target_dir.trim_end_matches('/'), entry.name);
                        let is_dir = entry.is_dir;
                        self.transfer_queue.push(TransferQueueItem {
                            id: qid,
                            batch_id,
                            filename: entry.name.clone(),
                            source_path: entry.path.clone(),
                            target_path: target_path.clone(),
                            direction: TransferDirection::Upload,
                            protocol,
                            size_bytes: entry.size,
                            transferred_bytes: 0,
                            speed_mbps: 0.0,
                            status: TransferStatus::Queued,
                            is_dir,
                            parent_id: None,
                        });
                        if is_dir {
                            scan_local_folder_for_queue(
                                std::path::Path::new(&entry.path),
                                &target_path,
                                batch_id,
                                qid,
                                TransferDirection::Upload,
                                protocol,
                                &mut self.next_transfer_id,
                                &mut self.transfer_queue,
                            );
                        }
                        tasks.push(TransferTask {
                            queue_id: qid,
                            filename: entry.name.clone(),
                            source_display: entry.path.clone(),
                            target_display: target_path.clone(),
                            size_bytes: entry.size,
                            kind: TransferKind::Upload {
                                protocol,
                                host: host.clone(),
                                port,
                                username: username.clone(),
                                password: password.clone(),
                                local_path: PathBuf::from(&entry.path),
                                remote_path: target_path,
                                is_dir: entry.is_dir,
                                remove_source,
                            },
                        });
                    }
                }
            }
            ActivePane::Remote => {
                if self.sessions.is_empty() {
                    for entry in &items {
                        let qid = self.next_transfer_id;
                        self.next_transfer_id += 1;
                        let src = PathBuf::from(&entry.path);
                        let dest = self.local_dir.join(&entry.name);
                        let is_dir = entry.is_dir;
                        self.transfer_queue.push(TransferQueueItem {
                            id: qid,
                            batch_id,
                            filename: entry.name.clone(),
                            source_path: entry.path.clone(),
                            target_path: dest.to_string_lossy().to_string(),
                            direction: TransferDirection::Download,
                            protocol: Protocol::Sftp,
                            size_bytes: entry.size,
                            transferred_bytes: 0,
                            speed_mbps: 0.0,
                            status: TransferStatus::Queued,
                            is_dir,
                            parent_id: None,
                        });
                        if is_dir {
                            scan_local_folder_for_queue(
                                &src,
                                &dest.to_string_lossy(),
                                batch_id,
                                qid,
                                TransferDirection::Download,
                                Protocol::Sftp,
                                &mut self.next_transfer_id,
                                &mut self.transfer_queue,
                            );
                        }
                        tasks.push(TransferTask {
                            queue_id: qid,
                            filename: entry.name.clone(),
                            source_display: entry.path.clone(),
                            target_display: dest.to_string_lossy().to_string(),
                            size_bytes: entry.size,
                            kind: TransferKind::Local {
                                src,
                                dest,
                                remove_source,
                            },
                        });
                    }
                } else if let Some(session) = self.sessions.get(self.active_session_idx) {
                    let protocol = session.protocol;
                    let host = session.host.clone();
                    let port = session.port;
                    let username = session.username.clone();
                    let password = session.password.clone();

                    for entry in &items {
                        let qid = self.next_transfer_id;
                        self.next_transfer_id += 1;
                        let target_path = self.local_dir.join(&entry.name);
                        let is_dir = entry.is_dir;
                        self.transfer_queue.push(TransferQueueItem {
                            id: qid,
                            batch_id,
                            filename: entry.name.clone(),
                            source_path: entry.path.clone(),
                            target_path: target_path.to_string_lossy().to_string(),
                            direction: TransferDirection::Download,
                            protocol,
                            size_bytes: entry.size,
                            transferred_bytes: 0,
                            speed_mbps: 0.0,
                            status: TransferStatus::Queued,
                            is_dir,
                            parent_id: None,
                        });
                        tasks.push(TransferTask {
                            queue_id: qid,
                            filename: entry.name.clone(),
                            source_display: entry.path.clone(),
                            target_display: target_path.to_string_lossy().to_string(),
                            size_bytes: entry.size,
                            kind: TransferKind::Download {
                                protocol,
                                host: host.clone(),
                                port,
                                username: username.clone(),
                                password: password.clone(),
                                remote_path: entry.path.clone(),
                                local_path: target_path,
                                is_dir: entry.is_dir,
                                remove_source,
                            },
                        });
                    }
                }
            }
        }

        self.dispatch_transfer_tasks(tasks, direction, batch_id, cx);
    }

    pub(crate) fn dispatch_transfer_tasks(
        &mut self,
        tasks: Vec<TransferTask>,
        direction: TransferDirection,
        batch_id: usize,
        cx: &mut Context<Self>,
    ) {
        if tasks.is_empty() {
            return;
        }

        let total_items = tasks.len();
        let total_bytes: u64 = tasks.iter().map(|t| t.size_bytes).sum();
        let first_filename = tasks.first().map(|t| t.filename.clone()).unwrap_or_default();
        let first_src = tasks.first().map(|t| t.source_display.clone()).unwrap_or_default();
        let first_target = tasks.first().map(|t| t.target_display.clone()).unwrap_or_default();
        let first_size = tasks.first().map(|t| t.size_bytes).unwrap_or(0);

        self.active_transfer = Some(ActiveTransferState {
            direction,
            total_items,
            current_item_index: 1,
            current_filename: first_filename,
            source_path: first_src,
            target_path: first_target,
            file_bytes_transferred: 0,
            file_bytes_total: first_size,
            total_bytes_transferred: 0,
            total_bytes,
            speed_mbps: 0.0,
            eta_seconds: None,
            is_completed: false,
            error_message: None,
        });
        // Do not popup a blocking modal window - keep workflow smooth like FileZilla
        self.active_modal = ActiveModal::None;
        self.show_queue = true;
        self.bottom_tab = BottomPanelTab::Split;
        cx.notify();

        let (tx, rx) = std::sync::mpsc::channel::<TransferProgressUpdate>();
        let cancel_signal = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        self.transfer_cancel_signal = Some(std::sync::Arc::clone(&cancel_signal));
        let cancel_signal_bg = std::sync::Arc::clone(&cancel_signal);
        let cancel_signal_ui = std::sync::Arc::clone(&cancel_signal);

        let bg_tasks = tasks.clone();
        let max_workers = if let Some(session) = self.sessions.get(self.active_session_idx) {
            session.max_connections.max(self.max_concurrent_transfers).clamp(1, 10)
        } else {
            self.max_concurrent_transfers.clamp(1, 10)
        };

        // Spawn background worker thread
        std::thread::spawn(move || {
            let mut disc_items = Vec::new();
            let mut next_disc_id = 1_000_000 + batch_id * 10000;
            for t in &bg_tasks {
                if cancel_signal_bg.load(std::sync::atomic::Ordering::SeqCst) {
                    let _ = tx.send(TransferProgressUpdate::AllFinished);
                    return;
                }
                match &t.kind {
                    TransferKind::Download { protocol, host, port, username, password, remote_path, local_path, is_dir, .. } => {
                        if *is_dir {
                            scan_remote_folder_for_queue(
                                *protocol,
                                host,
                                *port,
                                username,
                                password.as_deref(),
                                remote_path,
                                local_path,
                                batch_id,
                                t.queue_id,
                                &mut next_disc_id,
                                &mut disc_items,
                                &tx,
                            );
                        }
                    }
                    TransferKind::Upload { protocol, local_path, remote_path, is_dir, .. } => {
                        if *is_dir {
                            let mut local_next_id = next_disc_id;
                            scan_local_folder_for_queue(
                                local_path,
                                remote_path,
                                batch_id,
                                t.queue_id,
                                TransferDirection::Upload,
                                *protocol,
                                &mut local_next_id,
                                &mut disc_items,
                            );
                            next_disc_id = local_next_id;
                        }
                    }
                    TransferKind::Local { src, dest, .. } => {
                        if src.is_dir() {
                            let mut local_next_id = next_disc_id;
                            scan_local_folder_for_queue(
                                src,
                                &dest.to_string_lossy(),
                                batch_id,
                                t.queue_id,
                                TransferDirection::Download,
                                Protocol::Sftp,
                                &mut local_next_id,
                                &mut disc_items,
                            );
                            next_disc_id = local_next_id;
                        }
                    }
                }
            }
            if !disc_items.is_empty() {
                let _ = tx.send(TransferProgressUpdate::DiscoveredItems { items: disc_items.clone() });
            }

            // Extract leaf files for actual parallel transfer
            let mut file_tasks = std::collections::VecDeque::new();

            for t in &bg_tasks {
                match &t.kind {
                    TransferKind::Download { local_path, is_dir, .. } => {
                        if *is_dir {
                            let _ = std::fs::create_dir_all(local_path);
                        } else {
                            file_tasks.push_back(t.clone());
                        }
                    }
                    TransferKind::Upload { protocol, host, port, username, password, remote_path, is_dir, .. } => {
                        if *is_dir {
                            let _ = remote_mkdir(*protocol, host, *port, username, password.as_deref(), remote_path);
                        } else {
                            file_tasks.push_back(t.clone());
                        }
                    }
                    TransferKind::Local { dest, .. } => {
                        if dest.is_dir() || t.filename.ends_with('/') {
                            let _ = std::fs::create_dir_all(dest);
                        } else {
                            file_tasks.push_back(t.clone());
                        }
                    }
                }
            }

            for item in &disc_items {
                if item.is_dir {
                    match item.direction {
                        TransferDirection::Download => {
                            let _ = std::fs::create_dir_all(&item.target_path);
                        }
                        TransferDirection::Upload => {
                            if let Some(t) = bg_tasks.first() {
                                if let TransferKind::Upload { protocol, host, port, username, password, .. } = &t.kind {
                                    let _ = remote_mkdir(*protocol, host, *port, username, password.as_deref(), &item.target_path);
                                }
                            }
                        }
                    }
                } else {
                    let kind = match item.direction {
                        TransferDirection::Download => {
                            let (host, port, username, password) = if let Some(t) = bg_tasks.first() {
                                match &t.kind {
                                    TransferKind::Download { host, port, username, password, .. } => (host.clone(), *port, username.clone(), password.clone()),
                                    _ => (String::new(), 21, String::new(), None),
                                }
                            } else {
                                (String::new(), 21, String::new(), None)
                            };
                            TransferKind::Download {
                                protocol: item.protocol,
                                host,
                                port,
                                username,
                                password,
                                remote_path: item.source_path.clone(),
                                local_path: PathBuf::from(&item.target_path),
                                is_dir: false,
                                remove_source: false,
                            }
                        }
                        TransferDirection::Upload => {
                            let (host, port, username, password) = if let Some(t) = bg_tasks.first() {
                                match &t.kind {
                                    TransferKind::Upload { host, port, username, password, .. } => (host.clone(), *port, username.clone(), password.clone()),
                                    _ => (String::new(), 21, String::new(), None),
                                }
                            } else {
                                (String::new(), 21, String::new(), None)
                            };
                            TransferKind::Upload {
                                protocol: item.protocol,
                                host,
                                port,
                                username,
                                password,
                                local_path: PathBuf::from(&item.source_path),
                                remote_path: item.target_path.clone(),
                                is_dir: false,
                                remove_source: false,
                            }
                        }
                    };

                    file_tasks.push_back(TransferTask {
                        queue_id: item.id,
                        filename: item.filename.clone(),
                        source_display: item.source_path.clone(),
                        target_display: item.target_path.clone(),
                        size_bytes: item.size_bytes,
                        kind,
                    });
                }
            }

            let total_files = file_tasks.len();
            let total_bytes: u64 = file_tasks.iter().map(|t| t.size_bytes).sum();
            let _ = tx.send(TransferProgressUpdate::BatchInitialized { total_items: total_files, total_bytes });

            if file_tasks.is_empty() {
                let _ = tx.send(TransferProgressUpdate::AllFinished);
                return;
            }

            let num_workers = max_workers.clamp(1, 8).min(file_tasks.len());
            let work_queue = std::sync::Arc::new(std::sync::Mutex::new(file_tasks));
            let pause_lock = std::sync::Arc::new(std::sync::Mutex::new(false));
            let pause_cvar = std::sync::Arc::new(std::sync::Condvar::new());
            let skip_all_errors = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
            let cancelled = cancel_signal_bg;
            let mut worker_handles = Vec::new();

            for _ in 0..num_workers {
                let q = std::sync::Arc::clone(&work_queue);
                let tx = tx.clone();
                let pause_lock = std::sync::Arc::clone(&pause_lock);
                let pause_cvar = std::sync::Arc::clone(&pause_cvar);
                let skip_all_errors = std::sync::Arc::clone(&skip_all_errors);
                let cancelled = std::sync::Arc::clone(&cancelled);

                let h = std::thread::spawn(move || {
                    loop {
                        // Wait if paused due to an error in another worker
                        {
                            let mut paused = pause_lock.lock().unwrap();
                            while *paused && !cancelled.load(std::sync::atomic::Ordering::SeqCst) {
                                paused = pause_cvar.wait(paused).unwrap();
                            }
                        }

                        if cancelled.load(std::sync::atomic::Ordering::SeqCst) {
                            break;
                        }

                        let task = {
                            let mut guard = q.lock().unwrap();
                            guard.pop_front()
                        };
                        let Some(task) = task else {
                            break;
                        };

                        loop {
                            if cancelled.load(std::sync::atomic::Ordering::SeqCst) {
                                break;
                            }

                            let queue_id = task.queue_id;
                            let task_size = task.size_bytes;
                            let filename = task.filename.clone();

                            let _ = tx.send(TransferProgressUpdate::ItemStarted {
                                filename: filename.clone(),
                                src: task.source_display.clone(),
                                target: task.target_display.clone(),
                                size_bytes: task_size,
                                queue_id,
                            });

                            let start_time = std::time::Instant::now();
                            let tx_prog = tx.clone();
                            let mut on_prog = move |cur_bytes: u64, total_bytes: u64| {
                                let real_total = if total_bytes > 0 { total_bytes } else { task_size };
                                let elapsed = start_time.elapsed().as_secs_f64();
                                let speed = if elapsed > 0.05 {
                                    (cur_bytes as f64 / 1_048_576.0) / elapsed
                                } else {
                                    0.0
                                };
                                let eta = if speed > 0.01 && real_total > cur_bytes {
                                    Some(((real_total.saturating_sub(cur_bytes)) as f64 / (speed * 1_048_576.0)) as u64)
                                } else {
                                    None
                                };
                                let _ = tx_prog.send(TransferProgressUpdate::Progress {
                                    queue_id,
                                    bytes_transferred: cur_bytes,
                                    total_bytes: real_total,
                                    speed_mbps: speed,
                                    eta_seconds: eta,
                                });
                            };

                            let tx_log = tx.clone();
                            let mut on_log = move |kind: ServerLogKind, message: String| {
                                let _ = tx_log.send(TransferProgressUpdate::Log { kind, message });
                            };

                            let res = match &task.kind {
                                TransferKind::Download { protocol, host, port, username, password, remote_path, local_path, remove_source, .. } => {
                                    let r = download_from_remote_with_progress_and_log(
                                        *protocol,
                                        host,
                                        *port,
                                        username,
                                        password.as_deref(),
                                        remote_path,
                                        local_path,
                                        task_size,
                                        &mut on_prog,
                                        &mut on_log,
                                    );
                                    if r.is_ok() && *remove_source {
                                        let _ = remote_delete(*protocol, host, *port, username, password.as_deref(), remote_path);
                                    }
                                    r
                                }
                                TransferKind::Upload { protocol, host, port, username, password, local_path, remote_path, remove_source, .. } => {
                                    let r = upload_to_remote_with_progress_and_log(
                                        *protocol,
                                        host,
                                        *port,
                                        username,
                                        password.as_deref(),
                                        local_path,
                                        remote_path,
                                        &mut on_prog,
                                        &mut on_log,
                                    );
                                    if r.is_ok() && *remove_source {
                                        let _ = delete_local_item(local_path, false);
                                    }
                                    r
                                }
                                TransferKind::Local { src, dest, remove_source } => {
                                    if *remove_source {
                                        move_local_item_with_progress(src, dest, &mut on_prog).map_err(|e| e.to_string())
                                    } else {
                                        copy_local_item_with_progress(src, dest, &mut on_prog).map(|_| ()).map_err(|e| e.to_string())
                                    }
                                }
                            };

                            if let Err(err_str) = res {
                                let _ = tx.send(TransferProgressUpdate::Log {
                                    kind: ServerLogKind::Error,
                                    message: format!("Chyba přenosu {}: {}", filename, err_str),
                                });

                                if skip_all_errors.load(std::sync::atomic::Ordering::SeqCst) {
                                    let _ = tx.send(TransferProgressUpdate::ItemFailed { queue_id, error: err_str });
                                    break;
                                }

                                // Stop other workers from picking up new tasks while user decides
                                {
                                    let mut p = pause_lock.lock().unwrap();
                                    *p = true;
                                }

                                let (resp_tx, resp_rx) = std::sync::mpsc::channel();
                                let _ = tx.send(TransferProgressUpdate::PromptError {
                                    queue_id,
                                    filename: filename.clone(),
                                    src: task.source_display.clone(),
                                    target: task.target_display.clone(),
                                    error: err_str.clone(),
                                    resp_tx,
                                });

                                let action = resp_rx.recv().unwrap_or(TransferErrorAction::Cancel);
                                match action {
                                    TransferErrorAction::Retry => {
                                        // Loop again to retry transferring this task
                                        continue;
                                    }
                                    TransferErrorAction::Skip => {
                                        let _ = tx.send(TransferProgressUpdate::ItemFailed { queue_id, error: err_str });
                                        {
                                            let mut p = pause_lock.lock().unwrap();
                                            *p = false;
                                            pause_cvar.notify_all();
                                        }
                                        break;
                                    }
                                    TransferErrorAction::SkipAll => {
                                        skip_all_errors.store(true, std::sync::atomic::Ordering::SeqCst);
                                        let _ = tx.send(TransferProgressUpdate::ItemFailed { queue_id, error: err_str });
                                        {
                                            let mut p = pause_lock.lock().unwrap();
                                            *p = false;
                                            pause_cvar.notify_all();
                                        }
                                        break;
                                    }
                                    TransferErrorAction::Cancel => {
                                        cancelled.store(true, std::sync::atomic::Ordering::SeqCst);
                                        {
                                            let mut guard = q.lock().unwrap();
                                            guard.clear();
                                        }
                                        let _ = tx.send(TransferProgressUpdate::ItemFailed { queue_id, error: err_str });
                                        {
                                            let mut p = pause_lock.lock().unwrap();
                                            *p = false;
                                            pause_cvar.notify_all();
                                        }
                                        break;
                                    }
                                }
                            } else {
                                let _ = tx.send(TransferProgressUpdate::Log {
                                    kind: ServerLogKind::Response,
                                    message: format!("226 Přenos dokončen: {}", filename),
                                });
                                let _ = tx.send(TransferProgressUpdate::ItemDone {
                                    queue_id,
                                    size_bytes: task_size,
                                });
                                // If unpausing after successful retry
                                {
                                    let mut p = pause_lock.lock().unwrap();
                                    if *p {
                                        *p = false;
                                        pause_cvar.notify_all();
                                    }
                                }
                                break;
                            }
                        }
                    }
                });
                worker_handles.push(h);
            }

            for h in worker_handles {
                let _ = h.join();
            }

            let _ = tx.send(TransferProgressUpdate::AllFinished);
        });

        // Spawn GPUI task to pump messages into UI
        cx.spawn(move |this: gpui::WeakEntity<RusterView>, cx: &mut gpui::AsyncApp| {
            let mut cx = cx.clone();
            async move {
                loop {
                    let mut has_update = false;
                    let mut finished = false;

                    while let Ok(msg) = rx.try_recv() {
                        has_update = true;
                        let _ = this.update(&mut cx, |view, _| {
                            if cancel_signal_ui.load(std::sync::atomic::Ordering::SeqCst) {
                                view.transfer_queue.clear();
                                view.active_transfer = None;
                                return;
                            }
                            match msg {
                                TransferProgressUpdate::BatchInitialized { total_items, total_bytes } => {
                                    if let Some(t) = &mut view.active_transfer {
                                        t.total_items = total_items;
                                        t.total_bytes = total_bytes;
                                    }
                                }
                                TransferProgressUpdate::ItemStarted { filename, src, target, size_bytes, queue_id } => {
                                    if let Some(t) = &mut view.active_transfer {
                                        t.current_filename = filename.clone();
                                        t.source_path = src;
                                        t.target_path = target;
                                        t.file_bytes_transferred = 0;
                                        t.file_bytes_total = size_bytes;
                                    }
                                    if let Some(q) = view.transfer_queue.iter_mut().find(|q| q.id == queue_id) {
                                        q.status = TransferStatus::Transferring;
                                    }
                                }
                                TransferProgressUpdate::Progress { queue_id, bytes_transferred, total_bytes, speed_mbps, eta_seconds } => {
                                    if let Some(t) = &mut view.active_transfer {
                                        t.speed_mbps = speed_mbps;
                                        t.eta_seconds = eta_seconds;
                                        t.file_bytes_transferred = bytes_transferred;
                                    }
                                    if let Some(q) = view.transfer_queue.iter_mut().find(|q| q.id == queue_id) {
                                        q.transferred_bytes = bytes_transferred;
                                        if total_bytes > 0 && (q.size_bytes == 0 || q.size_bytes < total_bytes) {
                                            q.size_bytes = total_bytes;
                                        }
                                        q.speed_mbps = speed_mbps;
                                        q.status = TransferStatus::Transferring;
                                    }
                                }
                                TransferProgressUpdate::ItemDone { queue_id, size_bytes } => {
                                    if let Some(q) = view.transfer_queue.iter_mut().find(|q| q.id == queue_id) {
                                        q.status = TransferStatus::Completed;
                                        let final_sz = if q.size_bytes > 0 {
                                            q.size_bytes
                                        } else if size_bytes > 0 {
                                            size_bytes
                                        } else {
                                            q.transferred_bytes
                                        };
                                        q.size_bytes = final_sz;
                                        q.transferred_bytes = final_sz;
                                    }
                                    check_and_mark_completed_parents(&mut view.transfer_queue);
                                    if let Some(t) = &mut view.active_transfer {
                                        t.total_bytes_transferred += size_bytes;
                                    }
                                }
                                TransferProgressUpdate::ItemFailed { queue_id, error } => {
                                    if let Some(q) = view.transfer_queue.iter_mut().find(|q| q.id == queue_id) {
                                        q.status = TransferStatus::Failed;
                                    }
                                    if let Some(t) = &mut view.active_transfer {
                                        t.error_message = Some(error);
                                    }
                                }
                                TransferProgressUpdate::PromptError { queue_id, filename, src, target, error, resp_tx } => {
                                    if let Some(t) = &mut view.active_transfer {
                                        t.error_message = Some(error.clone());
                                    }
                                    view.transfer_error_prompt = Some(TransferErrorPromptState {
                                        queue_id,
                                        filename,
                                        source: src,
                                        target,
                                        error,
                                        resp_tx,
                                    });
                                    view.active_modal = ActiveModal::TransferError;
                                }
                                TransferProgressUpdate::DiscoveredItems { items } => {
                                    for it in &items {
                                        if it.is_dir {
                                            view.expanded_folders.insert(it.id);
                                        }
                                        if let Some(pid) = it.parent_id {
                                            view.expanded_folders.insert(pid);
                                        }
                                    }
                                    view.transfer_queue.extend(items);
                                }
                                TransferProgressUpdate::Log { kind, message } => {
                                    view.log_server(kind, message);
                                }
                                TransferProgressUpdate::AllFinished => {
                                    finished = true;
                                }
                            }
                        });
                    }

                    if has_update {
                        let _ = this.update(&mut cx, |_, cx| cx.notify());
                    }

                    if finished {
                        let _ = this.update(&mut cx, |view, cx| {
                            view.transfer_cancel_signal = None;
                            if cancel_signal_ui.load(std::sync::atomic::Ordering::SeqCst) {
                                view.transfer_queue.clear();
                                view.active_transfer = None;
                            } else {
                                check_and_mark_completed_parents(&mut view.transfer_queue);
                                if let Some(t) = &mut view.active_transfer {
                                    t.is_completed = true;
                                    t.file_bytes_transferred = t.file_bytes_total;
                                }
                                view.transfer_queue.retain(|q| q.status != TransferStatus::Completed);
                            }
                            view.reload_local();
                            view.reload_right();
                            cx.notify();
                        });
                        break;
                    }

                    cx.background_executor().timer(std::time::Duration::from_millis(60)).await;
                }
            }
        }).detach();
    }

    pub(crate) fn cancel_all_transfers(&mut self, cx: &mut Context<Self>) {
        if let Some(signal) = self.transfer_cancel_signal.take() {
            signal.store(true, std::sync::atomic::Ordering::SeqCst);
        }
        if let Some(prompt) = self.transfer_error_prompt.take() {
            let _ = prompt.resp_tx.send(TransferErrorAction::Cancel);
        }
        self.transfer_queue.clear();
        self.active_transfer = None;
        if self.active_modal == ActiveModal::TransferProgress || self.active_modal == ActiveModal::TransferError {
            self.active_modal = ActiveModal::None;
        }
        self.reload_local();
        self.reload_right();
        cx.notify();
    }

    pub(crate) fn handle_transfer_error_action(
        &mut self,
        action: TransferErrorAction,
        cx: &mut Context<Self>,
    ) {
        if action == TransferErrorAction::Cancel {
            self.cancel_all_transfers(cx);
            return;
        }
        if let Some(prompt) = self.transfer_error_prompt.take() {
            let _ = prompt.resp_tx.send(action);
        }
        self.active_modal = ActiveModal::None;
        cx.notify();
    }

    pub(crate) fn copy_selected_item(&mut self, remove_source: bool, cx: &mut Context<Self>) {
        let items: Vec<FileEntry> = match self.active_pane {
            ActivePane::Local => {
                let marked: Vec<FileEntry> = self
                    .local_entries
                    .iter()
                    .filter(|e| e.is_any_selected() && e.name != "..")
                    .cloned()
                    .collect();
                if marked.is_empty() {
                    self.local_entries
                        .get(self.local_selected)
                        .filter(|e| e.name != "..")
                        .cloned()
                        .into_iter()
                        .collect()
                } else {
                    marked
                }
            }
            ActivePane::Remote => {
                if self.sessions.is_empty() {
                    let marked: Vec<FileEntry> = self
                        .right_entries
                        .iter()
                        .filter(|e| e.is_any_selected() && e.name != "..")
                        .cloned()
                        .collect();
                    if marked.is_empty() {
                        self.right_entries
                            .get(self.right_selected)
                            .filter(|e| e.name != "..")
                            .cloned()
                            .into_iter()
                            .collect()
                    } else {
                        marked
                    }
                } else if let Some(session) = self.sessions.get(self.active_session_idx) {
                    let marked: Vec<FileEntry> = session
                        .entries
                        .iter()
                        .filter(|e| e.is_any_selected() && e.name != "..")
                        .cloned()
                        .collect();
                    if marked.is_empty() {
                        session
                            .entries
                            .get(session.selected_index)
                            .filter(|e| e.name != "..")
                            .cloned()
                            .into_iter()
                            .collect()
                    } else {
                        marked
                    }
                } else {
                    Vec::new()
                }
            }
        };

        if items.is_empty() {
            return;
        }

        let pane = self.active_pane;
        self.copy_items(items, pane, remove_source, cx);
    }

    pub(crate) fn open_new_folder_modal(&mut self) {
        self.modal_input_text.clear();
        self.modal_cursor_pos = 0;
        self.active_modal = ActiveModal::NewFolder;
    }

    pub(crate) fn confirm_new_folder(&mut self) {
        let name = self.modal_input_text.trim().to_string();
        if name.is_empty() {
            return;
        }
        match self.active_pane {
            ActivePane::Local => {
                if create_local_folder(&self.local_dir, &name).is_ok() {
                    self.reload_local();
                    if let Some(pos) = self.local_entries.iter().position(|e| e.name == name) {
                        self.local_selected = pos;
                        self.local_scroll.scroll_to_item(pos, ScrollStrategy::Top);
                    }
                }
            }
            ActivePane::Remote => {
                if self.sessions.is_empty() {
                    if create_local_folder(&self.right_dir, &name).is_ok() {
                        self.reload_right();
                        if let Some(pos) = self.right_entries.iter().position(|e| e.name == name) {
                            self.right_selected = pos;
                            self.remote_scroll.scroll_to_item(pos, ScrollStrategy::Top);
                        }
                    }
                } else if let Some(session) = self.sessions.get_mut(self.active_session_idx) {
                    let full_path =
                        format!("{}/{}", session.current_path.trim_end_matches('/'), name);
                    let _ = remote_mkdir(
                        session.protocol,
                        &session.host,
                        session.port,
                        &session.username,
                        session.password.as_deref(),
                        &full_path,
                    );
                    self.reload_right();
                }
            }
        }
        self.active_modal = ActiveModal::None;
    }

    pub(crate) fn open_rename_modal(&mut self) {
        let current_name = match self.active_pane {
            ActivePane::Local => self
                .local_entries
                .get(self.local_selected)
                .map(|e| e.name.clone()),
            ActivePane::Remote => {
                if self.sessions.is_empty() {
                    self.right_entries
                        .get(self.right_selected)
                        .map(|e| e.name.clone())
                } else {
                    self.sessions
                        .get(self.active_session_idx)
                        .and_then(|s| s.entries.get(s.selected_index))
                        .map(|e| e.name.clone())
                }
            }
        };
        if let Some(name) = current_name {
            if name != ".." {
                self.modal_input_text = name;
                self.modal_cursor_pos = self.modal_input_text.chars().count();
                self.active_modal = ActiveModal::Rename;
            }
        }
    }

    pub(crate) fn confirm_rename(&mut self) {
        let new_name = self.modal_input_text.trim().to_string();
        if new_name.is_empty() {
            return;
        }
        match self.active_pane {
            ActivePane::Local => {
                if let Some(entry) = self.local_entries.get(self.local_selected).cloned() {
                    if entry.name != ".."
                        && rename_local_item(&self.local_dir, &entry.name, &new_name).is_ok()
                    {
                        self.reload_local();
                        if let Some(pos) =
                            self.local_entries.iter().position(|e| e.name == new_name)
                        {
                            self.local_selected = pos;
                        }
                    }
                }
            }
            ActivePane::Remote => {
                if self.sessions.is_empty() {
                    if let Some(entry) = self.right_entries.get(self.right_selected).cloned() {
                        if entry.name != ".."
                            && rename_local_item(&self.right_dir, &entry.name, &new_name).is_ok()
                        {
                            self.reload_right();
                            if let Some(pos) =
                                self.right_entries.iter().position(|e| e.name == new_name)
                            {
                                self.right_selected = pos;
                            }
                        }
                    }
                } else if let Some(session) = self.sessions.get_mut(self.active_session_idx) {
                    let cur_dir = session.current_path.trim_end_matches('/').to_string();
                    let proto = session.protocol;
                    let host = session.host.clone();
                    let port = session.port;
                    let user = session.username.clone();
                    let pwd = session.password.clone();
                    if let Some(entry) = session.entries.get_mut(session.selected_index) {
                        if entry.name != ".." {
                            let new_path = format!("{}/{}", cur_dir, new_name);
                            let _ = remote_rename(proto, &host, port, &user, pwd.as_deref(), &entry.path, &new_path);
                            self.reload_right();
                        }
                    }
                }
            }
        }
        self.active_modal = ActiveModal::None;
    }

    pub(crate) fn open_delete_modal(&mut self) {
        let valid = match self.active_pane {
            ActivePane::Local => self
                .local_entries
                .get(self.local_selected)
                .map(|e| e.name != "..")
                .unwrap_or(false),
            ActivePane::Remote => {
                if self.sessions.is_empty() {
                    self.right_entries
                        .get(self.right_selected)
                        .map(|e| e.name != "..")
                        .unwrap_or(false)
                } else {
                    self.sessions
                        .get(self.active_session_idx)
                        .and_then(|s| s.entries.get(s.selected_index))
                        .map(|e| e.name != "..")
                        .unwrap_or(false)
                }
            }
        };
        if valid {
            self.active_modal = ActiveModal::DeleteConfirm;
        }
    }

    pub(crate) fn confirm_delete(&mut self) {
        match self.active_pane {
            ActivePane::Local => {
                let items_to_delete: Vec<FileEntry> = {
                    let marked: Vec<FileEntry> = self
                        .local_entries
                        .iter()
                        .filter(|e| e.is_any_selected() && e.name != "..")
                        .cloned()
                        .collect();
                    if marked.is_empty() {
                        self.local_entries
                            .get(self.local_selected)
                            .filter(|e| e.name != "..")
                            .cloned()
                            .into_iter()
                            .collect()
                    } else {
                        marked
                    }
                };
                for entry in items_to_delete {
                    let _ = delete_local_item(std::path::Path::new(&entry.path), entry.is_dir);
                }
                self.reload_local();
            }
            ActivePane::Remote => {
                if self.sessions.is_empty() {
                    let items_to_delete: Vec<FileEntry> = {
                        let marked: Vec<FileEntry> = self
                            .right_entries
                            .iter()
                            .filter(|e| e.is_any_selected() && e.name != "..")
                            .cloned()
                            .collect();
                    if marked.is_empty() {
                        self.right_entries
                            .get(self.right_selected)
                            .filter(|e| e.name != "..")
                            .cloned()
                            .into_iter()
                            .collect()
                    } else {
                        marked
                    }
                };
                for entry in items_to_delete {
                    let _ = delete_local_item(std::path::Path::new(&entry.path), entry.is_dir);
                }
                self.reload_right();
            } else if let Some(session) = self.sessions.get(self.active_session_idx) {
                let marked: Vec<FileEntry> = session
                    .entries
                    .iter()
                    .filter(|e| e.is_any_selected() && e.name != "..")
                    .cloned()
                    .collect();
                let items_to_delete = if marked.is_empty() {
                    session
                        .entries
                        .get(session.selected_index)
                        .filter(|e| e.name != "..")
                        .cloned()
                        .into_iter()
                        .collect()
                } else {
                    marked
                };
                for entry in items_to_delete {
                    let _ = remote_delete(
                        session.protocol,
                        &session.host,
                        session.port,
                        &session.username,
                        session.password.as_deref(),
                        &entry.path,
                    );
                }
                self.reload_right();
            }
        }
    }
        self.active_modal = ActiveModal::None;
    }

    pub(crate) fn open_chmod_modal(&mut self) {
        let mode = match self.active_pane {
            ActivePane::Local => self
                .local_entries
                .get(self.local_selected)
                .map(|e| e.octal_mode)
                .unwrap_or(0o755),
            ActivePane::Remote => {
                if self.sessions.is_empty() {
                    self.right_entries
                        .get(self.right_selected)
                        .map(|e| e.octal_mode)
                        .unwrap_or(0o755)
                } else {
                    self.sessions
                        .get(self.active_session_idx)
                        .and_then(|s| s.entries.get(s.selected_index))
                        .map(|e| e.octal_mode)
                        .unwrap_or(0o755)
                }
            }
        };
        self.chmod_mode = mode;
        self.active_modal = ActiveModal::Permissions;
    }

    pub(crate) fn apply_chmod(&mut self) {
        let new_mode = self.chmod_mode;
        let recursive = self.chmod_recursive;
        match self.active_pane {
            ActivePane::Local => {
                if let Some(entry) = self.local_entries.get(self.local_selected).cloned() {
                    if entry.name != ".." {
                        let _ = apply_local_chmod(
                            std::path::Path::new(&entry.path),
                            new_mode,
                            recursive,
                        );
                        self.reload_local();
                    }
                }
            }
            ActivePane::Remote => {
                if self.sessions.is_empty() {
                    if let Some(entry) = self.right_entries.get(self.right_selected).cloned() {
                        if entry.name != ".." {
                            let _ = apply_local_chmod(
                                std::path::Path::new(&entry.path),
                                new_mode,
                                recursive,
                            );
                            self.reload_right();
                        }
                    }
                } else if let Some(session) = self.sessions.get_mut(self.active_session_idx) {
                    let proto = session.protocol;
                    let host = session.host.clone();
                    let port = session.port;
                    let user = session.username.clone();
                    let pwd = session.password.clone();
                    if let Some(entry) = session.entries.get_mut(session.selected_index) {
                        let _ = remote_chmod(proto, &host, port, &user, pwd.as_deref(), &entry.path, new_mode, recursive);
                        self.reload_right();
                    }
                }
            }
        }
        self.active_modal = ActiveModal::None;
    }

    pub(crate) fn open_sync_modal(&mut self, cx: &mut Context<Self>) {
        self.sync_state.items.clear();
        self.sync_state.has_compared = false;
        self.sync_state.is_comparing = false;
        self.sync_state.progress_msg = None;
        self.sync_state.error_message = None;
        self.active_modal = ActiveModal::Synchronize;
        self.run_sync_comparison(cx);
        cx.notify();
    }

    pub(crate) fn run_sync_comparison(&mut self, cx: &mut Context<Self>) {
        self.sync_state.is_comparing = true;
        self.sync_state.progress_msg = None;
        self.sync_state.error_message = None;
        self.sync_state.has_compared = false;
        self.sync_state.items.clear();
        cx.notify();

        let local_dir = self.local_dir.clone();
        let is_remote_session = !self.sessions.is_empty();
        let session_info = self.sessions.get(self.active_session_idx).map(|s| {
            (
                s.protocol,
                s.host.clone(),
                s.port,
                s.username.clone(),
                s.password.clone(),
                s.current_path.clone(),
            )
        });
        let right_dir = self.right_dir.clone();
        let recursive = self.sync_state.recursive;
        let direction = self.sync_state.direction;
        let criterion = self.sync_state.criterion;
        let mirror_delete = self.sync_state.mirror_delete;
        let show_hidden = self.show_hidden;

        enum SyncCompareMsg {
            Progress(String),
            Finished(Result<Vec<SyncItem>, String>),
        }

        let (tx, rx) = std::sync::mpsc::channel::<SyncCompareMsg>();

        let tx_thread = tx.clone();
        std::thread::spawn(move || {
            let tx1 = tx_thread.clone();
            let _ = tx_thread.send(SyncCompareMsg::Progress("Scanning local files...".to_string()));
            let local_map = match crate::sync::scan_local_dir_with_progress(
                &local_dir,
                recursive,
                show_hidden,
                move |rel, count| {
                    let disp = if rel.is_empty() { "root" } else { rel };
                    let _ = tx1.send(SyncCompareMsg::Progress(format!(
                        "Scanning local: {} ({} items)",
                        disp, count
                    )));
                },
            ) {
                Ok(m) => m,
                Err(e) => {
                    let _ = tx_thread.send(SyncCompareMsg::Finished(Err(e)));
                    return;
                }
            };

            let tx2 = tx_thread.clone();
            let remote_map = if is_remote_session {
                if let Some((proto, host, port, user, pass, rem_path)) = session_info {
                    let _ = tx_thread.send(SyncCompareMsg::Progress(format!(
                        "Connecting & scanning remote: {}...",
                        rem_path
                    )));
                    match crate::sync::scan_remote_dir_with_progress(
                        proto,
                        &host,
                        port,
                        &user,
                        pass.as_deref(),
                        &rem_path,
                        recursive,
                        show_hidden,
                        move |curr, count| {
                            let _ = tx2.send(SyncCompareMsg::Progress(format!(
                                "Scanning remote: {} ({} items)",
                                curr, count
                            )));
                        },
                    ) {
                        Ok(m) => m,
                        Err(e) => {
                            let _ = tx_thread.send(SyncCompareMsg::Finished(Err(e)));
                            return;
                        }
                    }
                } else {
                    HashMap::new()
                }
            } else {
                let tx3 = tx_thread.clone();
                let _ = tx_thread.send(SyncCompareMsg::Progress("Scanning target folder...".to_string()));
                match crate::sync::scan_local_dir_with_progress(
                    &right_dir,
                    recursive,
                    show_hidden,
                    move |rel, count| {
                        let disp = if rel.is_empty() { "root" } else { rel };
                        let _ = tx3.send(SyncCompareMsg::Progress(format!(
                            "Scanning target: {} ({} items)",
                            disp, count
                        )));
                    },
                ) {
                    Ok(m) => m,
                    Err(e) => {
                        let _ = tx_thread.send(SyncCompareMsg::Finished(Err(e)));
                        return;
                    }
                }
            };

            let _ = tx_thread.send(SyncCompareMsg::Progress("Comparing file lists...".to_string()));
            let items = crate::sync::compare_directories(
                &local_map,
                &remote_map,
                direction,
                criterion,
                mirror_delete,
            );

            let _ = tx_thread.send(SyncCompareMsg::Finished(Ok(items)));
        });

        cx.spawn(move |this: gpui::WeakEntity<RusterView>, cx: &mut gpui::AsyncApp| {
            let mut cx = cx.clone();
            async move {
                loop {
                    let mut got_msg = false;
                    while let Ok(msg) = rx.try_recv() {
                        got_msg = true;
                        match msg {
                            SyncCompareMsg::Progress(status) => {
                                let _ = this.update(&mut cx, |view, cx| {
                                    view.sync_state.progress_msg = Some(status);
                                    cx.notify();
                                });
                            }
                            SyncCompareMsg::Finished(res) => {
                                let _ = this.update(&mut cx, |view, cx| {
                                    view.sync_state.is_comparing = false;
                                    view.sync_state.progress_msg = None;
                                    view.sync_state.has_compared = true;
                                    match res {
                                        Ok(items) => {
                                            view.sync_state.items = items;
                                            view.sync_state.error_message = None;
                                        }
                                        Err(err) => {
                                            view.sync_state.error_message = Some(err);
                                        }
                                    }
                                    cx.notify();
                                });
                                return;
                            }
                        }
                    }

                    if !got_msg {
                        cx.background_executor()
                            .timer(std::time::Duration::from_millis(40))
                            .await;
                    }
                }
            }
        })
        .detach();
    }

    pub(crate) fn execute_synchronization(&mut self, cx: &mut Context<Self>) {
        let selected_items: Vec<SyncItem> = self
            .sync_state
            .items
            .iter()
            .filter(|i| i.selected)
            .cloned()
            .collect();

        if selected_items.is_empty() {
            self.active_modal = ActiveModal::None;
            cx.notify();
            return;
        }

        let is_remote_session = !self.sessions.is_empty();
        let session_info = self.sessions.get(self.active_session_idx).map(|s| {
            (
                s.protocol,
                s.host.clone(),
                s.port,
                s.username.clone(),
                s.password.clone(),
                s.current_path.clone(),
            )
        });
        let local_root = self.local_dir.clone();
        let right_root = self.right_dir.clone();

        let batch_id = self.next_batch_id;
        self.next_batch_id += 1;

        let mut tasks = Vec::new();
        let mut deletes_remote = Vec::new();
        let mut deletes_local = Vec::new();

        for item in selected_items {
            let rel_path = item.rel_path;
            match item.action {
                SyncAction::DeleteRemote => {
                    if is_remote_session {
                        if let Some((_, _, _, _, _, rem_root)) = &session_info {
                            let rem_full = format!("{}/{}", rem_root.trim_end_matches('/'), rel_path);
                            deletes_remote.push((rem_full, item.is_dir));
                        }
                    } else {
                        let target_path = right_root.join(&rel_path);
                        deletes_local.push((target_path, item.is_dir));
                    }
                }
                SyncAction::DeleteLocal => {
                    let local_path = local_root.join(&rel_path);
                    deletes_local.push((local_path, item.is_dir));
                }
                SyncAction::Upload => {
                    let qid = self.next_transfer_id;
                    self.next_transfer_id += 1;
                    let local_path = local_root.join(&rel_path);

                    if is_remote_session {
                        if let Some((proto, host, port, user, pass, rem_root)) = &session_info {
                            let rem_full = format!("{}/{}", rem_root.trim_end_matches('/'), rel_path);
                            self.transfer_queue.push(TransferQueueItem {
                                id: qid,
                                batch_id,
                                filename: item.filename.clone(),
                                source_path: local_path.to_string_lossy().to_string(),
                                target_path: rem_full.clone(),
                                direction: TransferDirection::Upload,
                                protocol: *proto,
                                size_bytes: item.local_size,
                                transferred_bytes: 0,
                                speed_mbps: 0.0,
                                status: TransferStatus::Queued,
                                is_dir: item.is_dir,
                                parent_id: None,
                            });
                            tasks.push(TransferTask {
                                queue_id: qid,
                                filename: item.filename.clone(),
                                source_display: local_path.to_string_lossy().to_string(),
                                target_display: rem_full.clone(),
                                size_bytes: item.local_size,
                                kind: TransferKind::Upload {
                                    protocol: *proto,
                                    host: host.clone(),
                                    port: *port,
                                    username: user.clone(),
                                    password: pass.clone(),
                                    local_path,
                                    remote_path: rem_full,
                                    is_dir: item.is_dir,
                                    remove_source: false,
                                },
                            });
                        }
                    } else {
                        let right_path = right_root.join(&rel_path);
                        self.transfer_queue.push(TransferQueueItem {
                            id: qid,
                            batch_id,
                            filename: item.filename.clone(),
                            source_path: local_path.to_string_lossy().to_string(),
                            target_path: right_path.to_string_lossy().to_string(),
                            direction: TransferDirection::Upload,
                            protocol: Protocol::Sftp,
                            size_bytes: item.local_size,
                            transferred_bytes: 0,
                            speed_mbps: 0.0,
                            status: TransferStatus::Queued,
                            is_dir: item.is_dir,
                            parent_id: None,
                        });
                        tasks.push(TransferTask {
                            queue_id: qid,
                            filename: item.filename.clone(),
                            source_display: local_path.to_string_lossy().to_string(),
                            target_display: right_path.to_string_lossy().to_string(),
                            size_bytes: item.local_size,
                            kind: TransferKind::Local {
                                src: local_path,
                                dest: right_path,
                                remove_source: false,
                            },
                        });
                    }
                }
                SyncAction::Download => {
                    let qid = self.next_transfer_id;
                    self.next_transfer_id += 1;
                    let local_path = local_root.join(&rel_path);

                    if is_remote_session {
                        if let Some((proto, host, port, user, pass, rem_root)) = &session_info {
                            let rem_full = format!("{}/{}", rem_root.trim_end_matches('/'), rel_path);
                            self.transfer_queue.push(TransferQueueItem {
                                id: qid,
                                batch_id,
                                filename: item.filename.clone(),
                                source_path: rem_full.clone(),
                                target_path: local_path.to_string_lossy().to_string(),
                                direction: TransferDirection::Download,
                                protocol: *proto,
                                size_bytes: item.remote_size,
                                transferred_bytes: 0,
                                speed_mbps: 0.0,
                                status: TransferStatus::Queued,
                                is_dir: item.is_dir,
                                parent_id: None,
                            });
                            tasks.push(TransferTask {
                                queue_id: qid,
                                filename: item.filename.clone(),
                                source_display: rem_full.clone(),
                                target_display: local_path.to_string_lossy().to_string(),
                                size_bytes: item.remote_size,
                                kind: TransferKind::Download {
                                    protocol: *proto,
                                    host: host.clone(),
                                    port: *port,
                                    username: user.clone(),
                                    password: pass.clone(),
                                    remote_path: rem_full,
                                    local_path,
                                    is_dir: item.is_dir,
                                    remove_source: false,
                                },
                            });
                        }
                    } else {
                        let right_path = right_root.join(&rel_path);
                        self.transfer_queue.push(TransferQueueItem {
                            id: qid,
                            batch_id,
                            filename: item.filename.clone(),
                            source_path: right_path.to_string_lossy().to_string(),
                            target_path: local_path.to_string_lossy().to_string(),
                            direction: TransferDirection::Download,
                            protocol: Protocol::Sftp,
                            size_bytes: item.remote_size,
                            transferred_bytes: 0,
                            speed_mbps: 0.0,
                            status: TransferStatus::Queued,
                            is_dir: item.is_dir,
                            parent_id: None,
                        });
                        tasks.push(TransferTask {
                            queue_id: qid,
                            filename: item.filename.clone(),
                            source_display: right_path.to_string_lossy().to_string(),
                            target_display: local_path.to_string_lossy().to_string(),
                            size_bytes: item.remote_size,
                            kind: TransferKind::Local {
                                src: right_path,
                                dest: local_path,
                                remove_source: false,
                            },
                        });
                    }
                }
            }
        }

        // Execute deletions if any
        if !deletes_local.is_empty() {
            for (p, is_dir) in deletes_local {
                let _ = crate::model::delete_local_item(&p, is_dir);
            }
        }
        if !deletes_remote.is_empty() {
            if let Some((proto, host, port, user, pass, _)) = session_info {
                std::thread::spawn(move || {
                    for (rem_path, _is_dir) in deletes_remote {
                        let _ = crate::network::remote_delete(proto, &host, port, &user, pass.as_deref(), &rem_path);
                    }
                });
            }
        }

        self.active_modal = ActiveModal::None;
        if !tasks.is_empty() {
            let dir = tasks.first().map(|t| match t.kind {
                TransferKind::Upload { .. } => TransferDirection::Upload,
                TransferKind::Download { .. } => TransferDirection::Download,
                TransferKind::Local { .. } => TransferDirection::Upload,
            }).unwrap_or(TransferDirection::Upload);
            self.dispatch_transfer_tasks(tasks, dir, batch_id, cx);
        } else {
            self.reload_local();
            self.reload_right();
            cx.notify();
        }
    }

    pub(crate) fn get_clipboard_text(&self, cx: &mut Context<Self>) -> Option<String> {
        if let Some(item) = cx.read_from_clipboard() {
            if let Some(t) = item.text() {
                if !t.is_empty() {
                    return Some(t);
                }
            }
        }
        if let Ok(output) = std::process::Command::new("/usr/bin/pbpaste").output() {
            if output.status.success() {
                let s = String::from_utf8_lossy(&output.stdout).to_string();
                if !s.is_empty() {
                    return Some(s);
                }
            }
        }
        None
    }

    pub(crate) fn current_site_field_val(&self) -> String {
        if let Some(site) = self.saved_sites.get(self.selected_site_idx) {
            match self.active_site_field {
                SiteEditField::Name => site.name.clone(),
                SiteEditField::Host => site.host.clone(),
                SiteEditField::Port => {
                    if site.port == 0 {
                        String::new()
                    } else {
                        site.port.to_string()
                    }
                }
                SiteEditField::Username => site.username.clone(),
                SiteEditField::Password => self.site_password_input.clone(),
                SiteEditField::None => String::new(),
            }
        } else {
            String::new()
        }
    }

    pub(crate) fn set_current_site_field_val(&mut self, new_val: String) {
        if let Some(site) = self.saved_sites.get_mut(self.selected_site_idx) {
            match self.active_site_field {
                SiteEditField::Name => site.name = new_val,
                SiteEditField::Host => site.host = new_val,
                SiteEditField::Port => {
                    let digits: String = new_val.chars().filter(|c| c.is_ascii_digit()).collect();
                    site.port = digits.parse::<u16>().unwrap_or(0);
                }
                SiteEditField::Username => site.username = new_val,
                SiteEditField::Password => self.site_password_input = new_val,
                SiteEditField::None => {}
            }
        }
    }

    pub(crate) fn sort_saved_sites(&mut self) {
        let current_id = self.saved_sites.get(self.selected_site_idx).map(|s| {
            (s.name.clone(), s.host.clone(), s.username.clone(), s.port)
        });

        self.saved_sites.sort_by(|a, b| {
            a.display_name()
                .to_lowercase()
                .cmp(&b.display_name().to_lowercase())
                .then_with(|| a.host.to_lowercase().cmp(&b.host.to_lowercase()))
                .then_with(|| a.username.to_lowercase().cmp(&b.username.to_lowercase()))
                .then_with(|| a.port.cmp(&b.port))
        });

        if let Some((name, host, username, port)) = current_id {
            if let Some(pos) = self.saved_sites.iter().position(|s| {
                s.name == name && s.host == host && s.username == username && s.port == port
            }) {
                self.selected_site_idx = pos;
            }
        }
    }

    pub(crate) fn open_site_manager(&mut self) {
        if self.saved_sites.is_empty() {
            self.saved_sites = default_saved_sites();
        }
        self.sort_saved_sites();
        if self.selected_site_idx >= self.saved_sites.len() {
            self.selected_site_idx = 0;
        }
        if let Some(site) = self.saved_sites.get(self.selected_site_idx) {
            self.draft_protocol = site.protocol;
        }
        self.session_dropdown_open = false;
        self.context_menu = None;
        self.active_modal = ActiveModal::SiteManager;
        self.active_site_field = SiteEditField::Name;
        self.site_cursor_pos = self.current_site_field_val().chars().count();
        self.site_password_input.clear();
        self.site_error_message = None;
        self.site_notice = None;
        self.site_is_connecting = false;
    }

    pub(crate) fn connect_selected_site(&mut self, cx: &mut Context<Self>) {
        if self.site_is_connecting {
            return;
        }
        self.sort_saved_sites();
        let _ = save_sites_to_disk(&self.saved_sites);
        if let Some(mut site) = self.saved_sites.get(self.selected_site_idx).cloned() {
            let (detected_proto, clean_host, parsed_port, initial_path) =
                sanitize_host_and_path(&site.host);

            if clean_host.trim().is_empty() {
                self.site_error_message = Some("Please enter a Host / IP address".to_string());
                self.active_site_field = SiteEditField::Host;
                self.site_cursor_pos = 0;
                return;
            }

            if let Some(p) = detected_proto {
                self.draft_protocol = p;
                site.protocol = p;
            }
            site.host = clean_host.clone();
            if let Some(p) = parsed_port {
                site.port = p;
            }
            if let Some(stored) = self.saved_sites.get_mut(self.selected_site_idx) {
                stored.host = clean_host.clone();
                stored.port = site.port;
                stored.protocol = site.protocol;
            }

            let pwd = if !self.site_password_input.is_empty() {
                let p = self.site_password_input.clone();
                let _ = save_password_to_keychain(&site.host, &site.username, &p);
                site.password = Some(p.clone());
                if let Some(stored) = self.saved_sites.get_mut(self.selected_site_idx) {
                    stored.password = Some(p.clone());
                }
                Some(p)
            } else {
                site.password.clone()
            };

            let new_id = self.sessions.len() + 1;
            let protocol = self.draft_protocol;
            let remote_dir = if initial_path.is_empty() {
                "/".to_string()
            } else {
                initial_path
            };

            self.show_queue = true;
            self.bottom_tab = BottomPanelTab::Split;
            self.log_server(ServerLogKind::Status, format!("Připojování k {}:{} ({})...", site.host, site.port, site.protocol.label()));
            self.log_server(ServerLogKind::Command, format!("USER {}", site.username));

            self.site_is_connecting = true;
            self.site_error_message = None;
            self.site_notice = None;
            cx.notify();

            let host_clone = site.host.clone();
            let port = site.port;
            let user_clone = site.username.clone();
            let pwd_clone = pwd.clone();
            let remote_dir_clone = remote_dir.clone();
            let show_hidden = self.show_hidden;

            let (tx, rx) = std::sync::mpsc::channel::<Result<Vec<FileEntry>, String>>();

            std::thread::spawn(move || {
                let res = list_remote_directory(
                    protocol,
                    &host_clone,
                    port,
                    &user_clone,
                    pwd_clone.as_deref(),
                    &remote_dir_clone,
                    show_hidden,
                );
                let _ = tx.send(res);
            });

            let site_clone = site.clone();
            cx.spawn(move |this: gpui::WeakEntity<RusterView>, cx: &mut gpui::AsyncApp| {
                let mut cx = cx.clone();
                async move {
                    loop {
                        match rx.try_recv() {
                            Ok(res) => {
                                let _ = this.update(&mut cx, |view, cx| {
                                    view.site_is_connecting = false;
                                    match res {
                                        Ok(entries) => {
                                            view.log_server(ServerLogKind::Response, "230 Uživatel úspěšně přihlášen.".to_string());
                                            view.log_server(ServerLogKind::Command, format!("CWD {}", remote_dir));
                                            view.log_server(ServerLogKind::Response, format!("226 Přenos adresáře dokončen ({} položek)", entries.len()));
                                            view.sessions.push(RemoteSession {
                                                id: new_id,
                                                title: site_clone.host.clone(),
                                                protocol,
                                                host: site_clone.host.clone(),
                                                port: site_clone.port,
                                                username: site_clone.username.clone(),
                                                password: pwd,
                                                env_color: site_clone.env_color,
                                                current_path: remote_dir,
                                                entries,
                                                selected_index: 0,
                                                max_connections: site_clone.max_connections,
                                            });
                                            view.active_session_idx = view.sessions.len() - 1;
                                            view.active_pane = ActivePane::Remote;
                                            view.active_modal = ActiveModal::None;
                                            view.active_site_field = SiteEditField::None;
                                            view.session_dropdown_open = false;
                                            view.site_error_message = None;
                                            view.trigger_remote_flash(site_clone.env_color, false, cx);
                                        }
                                        Err(err) => {
                                            view.log_server(ServerLogKind::Error, format!("Chyba připojení: {}", err));
                                            view.site_error_message = Some(err);
                                        }
                                    }
                                    cx.notify();
                                });
                                break;
                            }
                            Err(std::sync::mpsc::TryRecvError::Empty) => {
                                cx.background_executor().timer(std::time::Duration::from_millis(50)).await;
                            }
                            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                                let _ = this.update(&mut cx, |view, cx| {
                                    view.site_is_connecting = false;
                                    view.site_error_message = Some("Connection thread disconnected".to_string());
                                    cx.notify();
                                });
                                break;
                            }
                        }
                    }
                }
            }).detach();
        }
    }

    pub(crate) fn is_editable_remote_file(&self, filename: &str) -> bool {
        is_editable_file(filename, &self.editable_extensions)
    }

    pub(crate) fn open_remote_file_for_editing(&mut self, entry: FileEntry, custom_app: Option<PathBuf>, cx: &mut Context<Self>) {
        let Some(session) = self.sessions.get(self.active_session_idx) else { return; };
        if custom_app.is_none() && !self.is_editable_remote_file(&entry.name) {
            self.log_server(
                ServerLogKind::Response,
                format!("Soubor '{}' nemá povolenou příponu pro přímou editaci (Nastavení -> Editace souborů).", entry.name),
            );
            return;
        }

        let temp_base = std::env::temp_dir()
            .join("ruster_remote_edit")
            .join(format!("session_{}", session.id));
        let safe_rel = entry.path.trim_start_matches('/').to_string();
        let local_path = temp_base.join(&safe_rel);

        if let Some(p) = local_path.parent() {
            let _ = std::fs::create_dir_all(p);
        }

        let existing_watched_path = self.watched_remote_files.iter()
            .find(|w| w.session_id == session.id && w.remote_path == entry.path && w.local_path.exists())
            .map(|w| w.local_path.clone());

        if let Some(existing_path) = existing_watched_path {
            if let Some(ref app) = custom_app {
                let app_name = app.file_name().and_then(|s| s.to_str()).unwrap_or("aplikaci");
                self.log_server(
                    ServerLogKind::Response,
                    format!("[Remote Edit] Otevírám lokální kopii '{}' v '{}'...", entry.name, app_name),
                );
                let _ = std::process::Command::new("open").arg("-a").arg(app).arg(&existing_path).spawn();
            } else {
                self.log_server(
                    ServerLogKind::Response,
                    format!("[Remote Edit] Otevírám lokální kopii '{}' v editoru...", entry.name),
                );
                cx.open_with_system(&existing_path);
                let _ = std::process::Command::new("open").arg(&existing_path).spawn();
            }
            return;
        }

        let session_id = session.id;
        let protocol = session.protocol;
        let host = session.host.clone();
        let port = session.port;
        let username = session.username.clone();
        let password = session.password.clone();
        let remote_path = entry.path.clone();
        let filename = entry.name.clone();
        let size = entry.size;
        let local_path_clone = local_path.clone();

        self.log_server(
            ServerLogKind::Command,
            format!("[Remote Edit] Stahuji '{}' do dočasné složky pro úpravu...", remote_path),
        );

        let (tx, rx) = std::sync::mpsc::channel::<Result<WatchedRemoteFile, String>>();
        std::thread::spawn(move || {
            let res = download_from_remote_with_progress_and_log(
                protocol,
                &host,
                port,
                &username,
                password.as_deref(),
                &remote_path,
                &local_path_clone,
                size,
                |_cur, _tot| {},
                |_kind, _msg| {},
            );
            match res {
                Ok(()) => {
                    let meta = std::fs::metadata(&local_path_clone);
                    let mtime = meta.as_ref().ok().and_then(|m| m.modified().ok()).unwrap_or(std::time::SystemTime::now());
                    let actual_size = meta.as_ref().ok().map(|m| m.len()).unwrap_or(size);
                    let _ = tx.send(Ok(WatchedRemoteFile {
                        session_id,
                        session_name: format!("{}@{}", username, host),
                        protocol,
                        host,
                        port,
                        username,
                        password,
                        remote_path,
                        local_path: local_path_clone,
                        filename,
                        last_mtime: mtime,
                        last_size: actual_size,
                        is_uploading: false,
                    }));
                }
                Err(e) => {
                    let _ = tx.send(Err(e));
                }
            }
        });

        let app_to_open = custom_app;
        cx.spawn(move |this: gpui::WeakEntity<RusterView>, cx: &mut gpui::AsyncApp| {
            let mut cx = cx.clone();
            async move {
                loop {
                    match rx.try_recv() {
                        Ok(Ok(watched)) => {
                            let local_path = watched.local_path.clone();
                            let fname = watched.filename.clone();
                            let app_ref = app_to_open.clone();
                            let _ = this.update(&mut cx, |view, cx| {
                                view.watched_remote_files.retain(|w| w.local_path != local_path);
                                view.watched_remote_files.push(watched);
                                if let Some(ref app) = app_ref {
                                    let app_name = app.file_name().and_then(|s| s.to_str()).unwrap_or("vybrané aplikaci");
                                    view.log_server(
                                        ServerLogKind::Response,
                                        format!("[Remote Edit] '{}' byl stažen a otevřen v '{}'. Úpravy se automaticky nahrají zpět na server.", fname, app_name),
                                    );
                                    let _ = std::process::Command::new("open").arg("-a").arg(app).arg(&local_path).spawn();
                                } else {
                                    view.log_server(
                                        ServerLogKind::Response,
                                        format!("[Remote Edit] '{}' byl stažen a otevřen v editoru. Úpravy se automaticky nahrají zpět na server.", fname),
                                    );
                                    cx.open_with_system(&local_path);
                                    let _ = std::process::Command::new("open").arg(&local_path).spawn();
                                }
                                cx.notify();
                            });
                            break;
                        }
                        Ok(Err(err)) => {
                            let _ = this.update(&mut cx, |view, cx| {
                                view.log_server(
                                    ServerLogKind::Error,
                                    format!("[Remote Edit] Chyba při stahování souboru pro úpravu: {}", err),
                                );
                                cx.notify();
                            });
                            break;
                        }
                        Err(std::sync::mpsc::TryRecvError::Disconnected) => break,
                        Err(std::sync::mpsc::TryRecvError::Empty) => {
                            cx.background_executor().timer(std::time::Duration::from_millis(50)).await;
                        }
                    }
                }
            }
        }).detach();
    }

    #[cfg(target_os = "macos")]
    pub(crate) fn pick_application_prompt() -> Option<PathBuf> {
        pick_application_prompt_macos()
    }

    #[cfg(not(target_os = "macos"))]
    pub(crate) fn pick_application_prompt() -> Option<PathBuf> {
        None
    }

    pub(crate) fn start_remote_edit_watcher(cx: &mut Context<Self>) {
        cx.spawn(move |this: gpui::WeakEntity<RusterView>, cx: &mut gpui::AsyncApp| {
            let mut cx = cx.clone();
            async move {
                loop {
                    cx.background_executor().timer(std::time::Duration::from_millis(800)).await;

                    let pending = this.update(&mut cx, |view, _| {
                        let mut list = Vec::new();
                        for w in &mut view.watched_remote_files {
                            if w.is_uploading || !w.local_path.exists() {
                                continue;
                            }
                            if let Ok(meta) = std::fs::metadata(&w.local_path) {
                                if let Ok(mtime) = meta.modified() {
                                    if mtime > w.last_mtime || meta.len() != w.last_size {
                                        w.is_uploading = true;
                                        w.last_mtime = mtime;
                                        w.last_size = meta.len();
                                        list.push(w.clone());
                                    }
                                }
                            }
                        }
                        list
                    });

                    let Ok(to_upload) = pending else { break; };

                    for item in to_upload {
                        let (tx, rx) = std::sync::mpsc::channel::<Result<(), String>>();
                        let session_id = item.session_id;
                        let local_path = item.local_path.clone();
                        let remote_path = item.remote_path.clone();
                        let filename = item.filename.clone();
                        let protocol = item.protocol;
                        let host = item.host.clone();
                        let port = item.port;
                        let username = item.username.clone();
                        let password = item.password.clone();

                        let _ = this.update(&mut cx, |view, cx| {
                            view.log_server(
                                ServerLogKind::Command,
                                format!("[Remote Edit] Detekována změna souboru '{}'. Automaticky nahrávám zpět na server...", filename),
                            );
                            cx.notify();
                        });

                        let remote_path_upload = remote_path.clone();
                        std::thread::spawn(move || {
                            std::thread::sleep(std::time::Duration::from_millis(200));
                            let res = upload_to_remote_with_progress_and_log(
                                protocol,
                                &host,
                                port,
                                &username,
                                password.as_deref(),
                                &local_path,
                                &remote_path_upload,
                                |_cur, _tot| {},
                                |_kind, _msg| {},
                            );
                            let _ = tx.send(res);
                        });

                        loop {
                            match rx.try_recv() {
                                Ok(res) => {
                                    let _ = this.update(&mut cx, |view, cx| {
                                        if let Some(w) = view.watched_remote_files.iter_mut().find(|w| w.session_id == session_id && w.remote_path == remote_path) {
                                            w.is_uploading = false;
                                            if let Ok(meta) = std::fs::metadata(&w.local_path) {
                                                if let Ok(mtime) = meta.modified() {
                                                    w.last_mtime = mtime;
                                                    w.last_size = meta.len();
                                                }
                                            }
                                        }
                                        match res {
                                            Ok(()) => {
                                                view.log_server(
                                                    ServerLogKind::Response,
                                                    format!("[Remote Edit] Soubor '{}' byl úspěšně automaticky uložen na server.", filename),
                                                );
                                                if let Some(s) = view.sessions.get(view.active_session_idx) {
                                                    if s.id == session_id {
                                                        let parent = std::path::Path::new(&remote_path)
                                                            .parent()
                                                            .map(|p| p.to_string_lossy().to_string())
                                                            .unwrap_or_else(|| "/".to_string());
                                                        if s.current_path == parent || s.current_path == remote_path {
                                                            let cur = s.current_path.clone();
                                                            view.navigate_remote(cur);
                                                        }
                                                    }
                                                }
                                            }
                                            Err(err) => {
                                                view.log_server(
                                                    ServerLogKind::Error,
                                                    format!("[Remote Edit] Chyba při nahrávání souboru '{}': {}", filename, err),
                                                );
                                            }
                                        }
                                        cx.notify();
                                    });
                                    break;
                                }
                                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                                    let _ = this.update(&mut cx, |view, _| {
                                        if let Some(w) = view.watched_remote_files.iter_mut().find(|w| w.session_id == session_id && w.remote_path == remote_path) {
                                            w.is_uploading = false;
                                        }
                                    });
                                    break;
                                }
                                Err(std::sync::mpsc::TryRecvError::Empty) => {
                                    cx.background_executor().timer(std::time::Duration::from_millis(50)).await;
                                }
                            }
                        }
                    }
                }
            }
        }).detach();
    }

    pub(crate) fn trigger_remote_flash(&mut self, color: u32, is_disconnect: bool, cx: &mut Context<Self>) {
        self.remote_flash_id = self.remote_flash_id.wrapping_add(1);
        let flash_id = self.remote_flash_id;

        self.remote_flash = Some(PaneFlash {
            color,
            opacity: 1.0,
            is_disconnect,
        });
        cx.notify();

        let total_duration_ms = if is_disconnect { 450 } else { 750 };
        let frame_interval_ms = 16;
        let total_frames = (total_duration_ms / frame_interval_ms).max(1);

        cx.spawn(move |this: gpui::WeakEntity<RusterView>, cx: &mut gpui::AsyncApp| {
            let mut cx = cx.clone();
            async move {
                for frame in 1..=total_frames {
                    cx.background_executor()
                        .timer(std::time::Duration::from_millis(frame_interval_ms as u64))
                        .await;

                    let progress = frame as f32 / total_frames as f32;
                    let opacity = (1.0 - progress).powi(2);

                    let still_running = this.update(&mut cx, |view, cx| {
                        if view.remote_flash_id != flash_id {
                            return false;
                        }
                        if let Some(flash) = &mut view.remote_flash {
                            flash.opacity = opacity;
                        }
                        cx.notify();
                        true
                    });

                    if still_running.is_err() || still_running.ok() == Some(false) {
                        return;
                    }
                }

                let _ = this.update(&mut cx, |view, cx| {
                    if view.remote_flash_id == flash_id {
                        view.remote_flash = None;
                        cx.notify();
                    }
                });
            }
        }).detach();
    }

    pub(crate) fn disconnect_active_session(&mut self, cx: &mut Context<Self>) {
        if !self.sessions.is_empty() {
            let session = self.sessions.remove(self.active_session_idx);
            let session_id = session.id;
            let env_color = session.env_color;
            self.watched_remote_files.retain(|w| w.session_id != session_id);
            if self.active_session_idx >= self.sessions.len() {
                self.active_session_idx = self.sessions.len().saturating_sub(1);
            }
            if self.sessions.is_empty() {
                self.show_queue = false;
                self.active_pane = ActivePane::Local;
                self.reload_right();
            }
            self.trigger_remote_flash(env_color, true, cx);
        }
        self.session_dropdown_open = false;
    }

    pub(crate) fn handle_key_down(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) {
        let key = event.keystroke.key.as_str();
        let modifiers = &event.keystroke.modifiers;

        if self.active_modal == ActiveModal::Settings {
            if key == "escape" {
                self.settings_lang_dropdown_open = false;
                self.settings_editing_exts = false;
                self.persist_app_state();
                self.active_modal = ActiveModal::None;
                cx.notify();
                return;
            }
            if key == "enter" {
                self.settings_editing_exts = false;
                self.persist_app_state();
                self.active_modal = ActiveModal::None;
                cx.notify();
                return;
            }
            if self.settings_editing_exts {
                let is_paste = (modifiers.platform || modifiers.control) && (key == "v" || key == "V");
                if is_paste {
                    if let Some(pasted) = self.get_clipboard_text(cx) {
                        let clean = pasted.trim_matches(['\r', '\n']).to_string();
                        let mut chars: Vec<char> = self.editable_extensions.chars().collect();
                        let pos = self.settings_cursor_pos.min(chars.len());
                        let count = clean.chars().count();
                        for (i, ch) in clean.chars().enumerate() {
                            chars.insert(pos + i, ch);
                        }
                        self.settings_cursor_pos = pos + count;
                        self.editable_extensions = chars.into_iter().collect();
                        cx.notify();
                        return;
                    }
                }
                let is_clear_or_select_all = (modifiers.platform || modifiers.control)
                    && (key == "a" || key == "A" || key == "backspace");
                if is_clear_or_select_all && key == "backspace" {
                    self.editable_extensions.clear();
                    self.settings_cursor_pos = 0;
                    cx.notify();
                    return;
                }
                match key {
                    "backspace" => {
                        let mut chars: Vec<char> = self.editable_extensions.chars().collect();
                        if self.settings_cursor_pos > 0 && !chars.is_empty() {
                            self.settings_cursor_pos -= 1;
                            chars.remove(self.settings_cursor_pos);
                            self.editable_extensions = chars.into_iter().collect();
                            cx.notify();
                        }
                    }
                    "left" => {
                        self.settings_cursor_pos = self.settings_cursor_pos.saturating_sub(1);
                        cx.notify();
                    }
                    "right" => {
                        self.settings_cursor_pos = (self.settings_cursor_pos + 1).min(self.editable_extensions.chars().count());
                        cx.notify();
                    }
                    _ => {
                        if key.chars().count() == 1 {
                            let ch = key.chars().next().unwrap();
                            if !ch.is_control() {
                                let mut chars: Vec<char> = self.editable_extensions.chars().collect();
                                let pos = self.settings_cursor_pos.min(chars.len());
                                chars.insert(pos, ch);
                                self.settings_cursor_pos = pos + 1;
                                self.editable_extensions = chars.into_iter().collect();
                                cx.notify();
                            }
                        }
                    }
                }
                return;
            }
        }

        // If SiteManager is open and a field is focused, route typing to that field
        if self.active_modal == ActiveModal::SiteManager {
            if key == "escape" {
                self.active_modal = ActiveModal::None;
                self.active_site_field = SiteEditField::None;
                cx.notify();
                return;
            }
            if key == "enter" {
                self.connect_selected_site(cx);
                cx.notify();
                return;
            }
            if key == "tab" {
                if modifiers.shift {
                    self.active_site_field = match self.active_site_field {
                        SiteEditField::None | SiteEditField::Name => SiteEditField::Password,
                        SiteEditField::Host => SiteEditField::Name,
                        SiteEditField::Port => SiteEditField::Host,
                        SiteEditField::Username => SiteEditField::Port,
                        SiteEditField::Password => SiteEditField::Username,
                    };
                } else {
                    self.active_site_field = match self.active_site_field {
                        SiteEditField::None | SiteEditField::Password => SiteEditField::Name,
                        SiteEditField::Name => SiteEditField::Host,
                        SiteEditField::Host => SiteEditField::Port,
                        SiteEditField::Port => SiteEditField::Username,
                        SiteEditField::Username => SiteEditField::Password,
                    };
                }
                self.site_cursor_pos = self.current_site_field_val().chars().count();
                cx.notify();
                return;
            }
            if self.active_site_field != SiteEditField::None {
                // Left Arrow
                if key == "arrowleft" || key == "left" {
                    if modifiers.platform {
                        self.site_cursor_pos = 0;
                    } else {
                        self.site_cursor_pos = self.site_cursor_pos.saturating_sub(1);
                    }
                    cx.notify();
                    return;
                }

                // Right Arrow
                if key == "arrowright" || key == "right" {
                    let len = self.current_site_field_val().chars().count();
                    if modifiers.platform {
                        self.site_cursor_pos = len;
                    } else {
                        self.site_cursor_pos = (self.site_cursor_pos + 1).min(len);
                    }
                    cx.notify();
                    return;
                }

                // Home / End
                if key == "home" {
                    self.site_cursor_pos = 0;
                    cx.notify();
                    return;
                }
                if key == "end" {
                    self.site_cursor_pos = self.current_site_field_val().chars().count();
                    cx.notify();
                    return;
                }

                // Clipboard paste (Cmd+V or Ctrl+V)
                let is_paste = (modifiers.platform || modifiers.control) && (key == "v" || key == "V");
                if is_paste {
                    if let Some(pasted) = self.get_clipboard_text(cx) {
                        let clean = pasted.trim_matches(['\r', '\n']).to_string();
                        let val = self.current_site_field_val();
                        let mut chars: Vec<char> = val.chars().collect();
                        let pos = self.site_cursor_pos.min(chars.len());
                        let count = clean.chars().count();
                        for (i, ch) in clean.chars().enumerate() {
                            chars.insert(pos + i, ch);
                        }
                        self.site_cursor_pos = pos + count;
                        self.set_current_site_field_val(chars.into_iter().collect());
                        cx.notify();
                        return;
                    }
                }

                // Select all / Clear (Cmd+A or Ctrl+A or Cmd+Backspace)
                let is_clear_or_select_all = (modifiers.platform || modifiers.control)
                    && (key == "a" || key == "A" || key == "backspace");
                if is_clear_or_select_all {
                    self.set_current_site_field_val(String::new());
                    self.site_cursor_pos = 0;
                    cx.notify();
                    return;
                }

                // Backspace (delete before cursor)
                if key == "backspace" {
                    let val = self.current_site_field_val();
                    let mut chars: Vec<char> = val.chars().collect();
                    if self.site_cursor_pos > 0 && self.site_cursor_pos <= chars.len() {
                        chars.remove(self.site_cursor_pos - 1);
                        self.site_cursor_pos -= 1;
                        self.set_current_site_field_val(chars.into_iter().collect());
                    }
                    cx.notify();
                    return;
                }

                // Delete (delete at cursor)
                if key == "delete" {
                    let val = self.current_site_field_val();
                    let mut chars: Vec<char> = val.chars().collect();
                    if self.site_cursor_pos < chars.len() {
                        chars.remove(self.site_cursor_pos);
                        self.set_current_site_field_val(chars.into_iter().collect());
                    }
                    cx.notify();
                    return;
                }

                // Normal character insertion
                if !modifiers.platform && !modifiers.control && key.chars().count() == 1 {
                    let val = self.current_site_field_val();
                    let mut chars: Vec<char> = val.chars().collect();
                    let pos = self.site_cursor_pos.min(chars.len());
                    let count = key.chars().count();
                    for (i, ch) in key.chars().enumerate() {
                        chars.insert(pos + i, ch);
                    }
                    self.site_cursor_pos = pos + count;
                    self.set_current_site_field_val(chars.into_iter().collect());
                    cx.notify();
                    return;
                }
            }
            return;
        }

        // If a text-input modal (New Folder / Rename / GoToFolder) is open, route typing to modal_input_text
        if matches!(
            self.active_modal,
            ActiveModal::NewFolder | ActiveModal::Rename | ActiveModal::GoToFolder
        ) {
            let is_paste = (modifiers.platform || modifiers.control) && (key == "v" || key == "V");
            if is_paste {
                if let Some(pasted) = self.get_clipboard_text(cx) {
                    let clean = pasted.trim_matches(['\r', '\n']).to_string();
                    let mut chars: Vec<char> = self.modal_input_text.chars().collect();
                    let pos = self.modal_cursor_pos.min(chars.len());
                    let count = clean.chars().count();
                    for (i, ch) in clean.chars().enumerate() {
                        chars.insert(pos + i, ch);
                    }
                    self.modal_cursor_pos = pos + count;
                    self.modal_input_text = chars.into_iter().collect();
                    cx.notify();
                    return;
                }
            }
            let is_clear_or_select_all = (modifiers.platform || modifiers.control)
                && (key == "a" || key == "A" || key == "backspace");
            if is_clear_or_select_all {
                self.modal_input_text.clear();
                self.modal_cursor_pos = 0;
                cx.notify();
                return;
            }
            if key == "left" {
                self.modal_cursor_pos = self.modal_cursor_pos.saturating_sub(1);
                cx.notify();
                return;
            }
            if key == "right" {
                self.modal_cursor_pos = (self.modal_cursor_pos + 1).min(self.modal_input_text.chars().count());
                cx.notify();
                return;
            }
            if key == "home" {
                self.modal_cursor_pos = 0;
                cx.notify();
                return;
            }
            if key == "end" {
                self.modal_cursor_pos = self.modal_input_text.chars().count();
                cx.notify();
                return;
            }
            match key {
                "escape" => {
                    self.active_modal = ActiveModal::None;
                    cx.notify();
                }
                "enter" => {
                    if self.active_modal == ActiveModal::NewFolder {
                        self.confirm_new_folder();
                    } else if self.active_modal == ActiveModal::Rename {
                        self.confirm_rename();
                    } else if self.active_modal == ActiveModal::GoToFolder {
                        self.confirm_go_to_folder();
                    }
                    cx.notify();
                }
                "backspace" => {
                    let mut chars: Vec<char> = self.modal_input_text.chars().collect();
                    if self.modal_cursor_pos > 0 && self.modal_cursor_pos <= chars.len() {
                        chars.remove(self.modal_cursor_pos - 1);
                        self.modal_cursor_pos -= 1;
                        self.modal_input_text = chars.into_iter().collect();
                    }
                    cx.notify();
                }
                "delete" => {
                    let mut chars: Vec<char> = self.modal_input_text.chars().collect();
                    if self.modal_cursor_pos < chars.len() {
                        chars.remove(self.modal_cursor_pos);
                        self.modal_input_text = chars.into_iter().collect();
                    }
                    cx.notify();
                }
                _ => {
                    if !modifiers.platform && !modifiers.control && key.chars().count() == 1 {
                        let mut chars: Vec<char> = self.modal_input_text.chars().collect();
                        let pos = self.modal_cursor_pos.min(chars.len());
                        let count = key.chars().count();
                        for (i, ch) in key.chars().enumerate() {
                            chars.insert(pos + i, ch);
                        }
                        self.modal_cursor_pos = pos + count;
                        self.modal_input_text = chars.into_iter().collect();
                        cx.notify();
                    }
                }
            }
            return;
        }

        // If any other modal is open, Escape closes it
        if self.active_modal != ActiveModal::None {
            if key == "escape" {
                self.active_modal = ActiveModal::None;
                cx.notify();
            } else if key == "enter" && self.active_modal == ActiveModal::DeleteConfirm {
                self.confirm_delete();
                cx.notify();
            }
            return;
        }

        // Quick filter typing when filter bar is open on active pane
        let active_filter_open = match self.active_pane {
            ActivePane::Local => self.show_local_filter,
            ActivePane::Remote => self.show_remote_filter,
        };

        if active_filter_open {
            if key == "escape" {
                let has_active_query = match self.active_pane {
                    ActivePane::Local => !self.local_filter_query.is_empty(),
                    ActivePane::Remote => !self.remote_filter_query.is_empty(),
                };
                if has_active_query {
                    match self.active_pane {
                        ActivePane::Local => {
                            self.local_filter_query.clear();
                            self.ensure_local_selection();
                        }
                        ActivePane::Remote => {
                            self.remote_filter_query.clear();
                            self.ensure_remote_selection();
                        }
                    }
                } else {
                    self.show_local_filter = false;
                    self.show_remote_filter = false;
                    self.local_filter_query.clear();
                    self.remote_filter_query.clear();
                    self.ensure_local_selection();
                    self.ensure_remote_selection();
                }
                cx.notify();
                return;
            }

            // Cmd+A / Ctrl+A or Cmd+Backspace: Clear filter
            let is_clear = (modifiers.platform || modifiers.control)
                && (key == "a" || key == "A" || key == "backspace");
            if is_clear {
                match self.active_pane {
                    ActivePane::Local => {
                        self.local_filter_query.clear();
                        self.ensure_local_selection();
                    }
                    ActivePane::Remote => {
                        self.remote_filter_query.clear();
                        self.ensure_remote_selection();
                    }
                }
                cx.notify();
                return;
            }

            // Paste into filter (Cmd+V / Ctrl+V)
            let is_paste = (modifiers.platform || modifiers.control) && (key == "v" || key == "V");
            if is_paste {
                if let Some(pasted) = self.get_clipboard_text(cx) {
                    let clean = pasted.trim_matches(['\r', '\n']).to_string();
                    match self.active_pane {
                        ActivePane::Local => {
                            self.local_filter_query.push_str(&clean);
                            self.ensure_local_selection();
                        }
                        ActivePane::Remote => {
                            self.remote_filter_query.push_str(&clean);
                            self.ensure_remote_selection();
                        }
                    }
                    cx.notify();
                    return;
                }
            }

            // Backspace deletes last character of filter query
            if key == "backspace" {
                match self.active_pane {
                    ActivePane::Local => {
                        if !self.local_filter_query.is_empty() {
                            self.local_filter_query.pop();
                            self.ensure_local_selection();
                            cx.notify();
                        }
                    }
                    ActivePane::Remote => {
                        if !self.remote_filter_query.is_empty() {
                            self.remote_filter_query.pop();
                            self.ensure_remote_selection();
                            cx.notify();
                        }
                    }
                }
                return;
            }

            // If not holding platform/ctrl modifiers, check for character input
            if !modifiers.platform && !modifiers.control {
                let typed_char: Option<String> = if let Some(ref kc) = event.keystroke.key_char {
                    if !kc.is_empty() && !kc.chars().any(|c| c.is_control()) {
                        Some(kc.clone())
                    } else {
                        None
                    }
                } else if key.chars().count() == 1 {
                    let ch = key.chars().next().unwrap();
                    if !ch.is_control() {
                        Some(key.to_string())
                    } else {
                        None
                    }
                } else if key == "space" {
                    Some(" ".to_string())
                } else if key == "period" {
                    Some(".".to_string())
                } else if key == "minus" {
                    Some("-".to_string())
                } else if key == "underscore" {
                    Some("_".to_string())
                } else {
                    None
                };

                if let Some(text) = typed_char {
                    if key != "enter" && key != "tab" && key != "escape" && key != "up" && key != "down" && key != "left" && key != "right" {
                        match self.active_pane {
                            ActivePane::Local => {
                                self.local_filter_query.push_str(&text);
                                self.ensure_local_selection();
                            }
                            ActivePane::Remote => {
                                self.remote_filter_query.push_str(&text);
                                self.ensure_remote_selection();
                            }
                        }
                        cx.notify();
                        return;
                    }
                }
            }
        }

        match key {
            "up" => {
                self.context_menu = None;
                if modifiers.shift {
                    self.select_range_keyboard(-1);
                } else {
                    self.move_selection(-1);
                }
                cx.notify();
            }
            "down" => {
                self.context_menu = None;
                if modifiers.shift {
                    self.select_range_keyboard(1);
                } else {
                    self.move_selection(1);
                }
                cx.notify();
            }
            "enter" => {
                self.context_menu = None;
                self.open_selected_item(cx);
                cx.notify();
            }
            "backspace" => {
                self.context_menu = None;
                self.navigate_up();
                cx.notify();
            }
            "tab" => {
                self.context_menu = None;
                if self.show_local_pane {
                    self.active_pane = match self.active_pane {
                        ActivePane::Local => ActivePane::Remote,
                        ActivePane::Remote => ActivePane::Local,
                    };
                    cx.notify();
                }
            }
            " " | "space" => {
                self.context_menu = None;
                if modifiers.shift {
                    self.toggle_space_mark(-1, cx);
                } else {
                    self.toggle_space_mark(1, cx);
                }
                cx.notify();
            }
            "escape" => {
                self.context_menu = None;
                self.session_dropdown_open = false;
                self.favorites_menu_pane = None;
                if self.show_local_filter || self.show_remote_filter {
                    self.show_local_filter = false;
                    self.show_remote_filter = false;
                    self.local_filter_query.clear();
                    self.remote_filter_query.clear();
                    self.ensure_local_selection();
                    self.ensure_remote_selection();
                } else {
                    match self.active_pane {
                        ActivePane::Local => {
                            for e in self.local_entries.iter_mut() {
                                e.selected = false;
                                e.space_marked = false;
                            }
                        }
                        ActivePane::Remote => {
                            if self.sessions.is_empty() {
                                for e in self.right_entries.iter_mut() {
                                    e.selected = false;
                                    e.space_marked = false;
                                }
                            } else if let Some(s) = self.sessions.get_mut(self.active_session_idx) {
                                for e in s.entries.iter_mut() {
                                    e.selected = false;
                                    e.space_marked = false;
                                }
                            }
                        }
                    }
                }
                cx.notify();
            }
            "f2" => {
                self.open_rename_modal();
                cx.notify();
            }
            "f4" => {
                self.open_selected_item(cx);
                cx.notify();
            }
            "f5" => {
                self.copy_selected_item(false, cx);
                cx.notify();
            }
            "f6" => {
                self.copy_selected_item(true, cx);
                cx.notify();
            }
            "f7" => {
                self.open_new_folder_modal();
                cx.notify();
            }
            "f8" => {
                self.open_delete_modal();
                cx.notify();
            }
            "f9" => {
                self.open_chmod_modal();
                cx.notify();
            }
            _ => {}
        }
    }

}

impl Render for RusterView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !window.is_fullscreen() {
            let cur_size = window.viewport_size();
            let cur_w: f32 = cur_size.width.into();
            let cur_h: f32 = cur_size.height.into();
            if cur_w >= 600.0 && cur_h >= 400.0 {
                if (cur_w - self.window_width).abs() > 2.0 || (cur_h - self.window_height).abs() > 2.0 {
                    self.window_width = cur_w;
                    self.window_height = cur_h;
                    if self.last_window_size_saved.elapsed() > std::time::Duration::from_millis(400) {
                        self.last_window_size_saved = std::time::Instant::now();
                        self.persist_app_state();
                    }
                }
            }
        }

        if !self.focus_handle.is_focused(window) {
            window.focus(&self.focus_handle);
        }
        let t = self.i18n();
        let theme = self.theme();

        div()
            .id("ruster-root")
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _, cx| {
                this.handle_key_down(event, cx);
            }))
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _, cx| {
                if let Some(state) = this.resizing_column {
                    let current_x = f32::from(event.position.x);
                    let delta = current_x - state.start_mouse_x;
                    match state.col_index {
                        0 => {
                            this.col_size_width = (state.initial_size_w - delta).clamp(45.0, 300.0);
                        }
                        1 => {
                            let new_size = (state.initial_size_w + delta).clamp(45.0, 300.0);
                            let actual_delta = new_size - state.initial_size_w;
                            this.col_size_width = new_size;
                            let min_mod = if this.date_format == DateFormat::Date { 45.0 } else { 55.0 };
                            this.col_modified_width = (state.initial_modified_w - actual_delta).clamp(min_mod, 300.0);
                        }
                        2 => {
                            let min_mod = if this.date_format == DateFormat::Date { 45.0 } else { 55.0 };
                            let new_mod = (state.initial_modified_w + delta).clamp(min_mod, 300.0);
                            let actual_delta = new_mod - state.initial_modified_w;
                            this.col_modified_width = new_mod;
                            this.col_permissions_width = (state.initial_permissions_w - actual_delta).clamp(40.0, 200.0);
                        }
                        _ => {}
                    }
                    cx.notify();
                }
                if let Some(state) = this.resizing_queue_panel {
                    let current_y = f32::from(event.position.y);
                    let delta = current_y - state.start_mouse_y;
                    let max_h = (this.window_height - 180.0).max(120.0);
                    this.queue_panel_height = (state.initial_height - delta).clamp(60.0, max_h);
                    cx.notify();
                }
            }))
            .on_mouse_up(MouseButton::Left, cx.listener(|this, _, _, cx| {
                let mut changed = false;
                if this.resizing_column.is_some() {
                    this.resizing_column = None;
                    changed = true;
                }
                if this.resizing_queue_panel.is_some() {
                    this.resizing_queue_panel = None;
                    this.persist_app_state();
                    changed = true;
                }
                if changed {
                    cx.notify();
                }
            }))
            // Native macOS Menu & Shortcut Action Handlers
            .on_action(cx.listener(|this, _: &OpenSiteManager, _, cx| {
                if this.active_modal != ActiveModal::None { return; }
                this.open_site_manager();
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &ToggleDualPane, _, cx| {
                if this.active_modal != ActiveModal::None { return; }
                this.show_local_pane = !this.show_local_pane;
                if !this.show_local_pane {
                    this.active_pane = ActivePane::Remote;
                }
                this.persist_app_state();
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &SwapPanes, _, cx| {
                if this.active_modal != ActiveModal::None { return; }
                this.swap_panes = !this.swap_panes;
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &ToggleLocalTree, _, cx| {
                if this.active_modal != ActiveModal::None { return; }
                this.show_local_tree = !this.show_local_tree;
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &ToggleRemoteTree, _, cx| {
                if this.active_modal != ActiveModal::None { return; }
                this.show_remote_tree = !this.show_remote_tree;
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &ToggleHiddenFiles, _, cx| {
                if this.active_modal != ActiveModal::None { return; }
                this.show_hidden = !this.show_hidden;
                this.reload_local();
                this.persist_app_state();
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &ToggleTransferQueue, _, cx| {
                if this.active_modal != ActiveModal::None { return; }
                if !this.sessions.is_empty() {
                    return;
                }
                this.show_queue = !this.show_queue;
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &ToggleFunctionBar, _, cx| {
                if this.active_modal != ActiveModal::None { return; }
                this.show_function_bar = !this.show_function_bar;
                this.persist_app_state();
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &ToggleQuickFilter, _, cx| {
                if this.active_modal != ActiveModal::None { return; }
                let should_show = !(this.show_local_filter && this.show_remote_filter);
                this.show_local_filter = should_show;
                this.show_remote_filter = should_show;
                if !should_show {
                    this.local_filter_query.clear();
                    this.remote_filter_query.clear();
                }
                this.ensure_local_selection();
                this.ensure_remote_selection();
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &RefreshPanes, _, cx| {
                if this.active_modal != ActiveModal::None { return; }
                this.reload_local();
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &SwitchLanguage, _, cx| {
                if this.active_modal != ActiveModal::None { return; }
                this.active_modal = ActiveModal::Settings;
                this.settings_lang_dropdown_open = true;
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &ToggleTheme, _, cx| {
                if this.active_modal != ActiveModal::None { return; }
                this.toggle_theme(cx);
            }))
            .on_action(cx.listener(|this, _: &OpenSelected, _, cx| {
                if this.active_modal != ActiveModal::None { return; }
                this.open_selected_item(cx);
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &EditSelected, _, cx| {
                if this.active_modal != ActiveModal::None { return; }
                this.open_selected_item(cx);
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &CopySelected, _, cx| {
                if this.active_modal != ActiveModal::None { return; }
                this.copy_selected_item(false, cx);
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &MoveSelected, _, cx| {
                if this.active_modal != ActiveModal::None { return; }
                this.copy_selected_item(true, cx);
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &NewFolderModal, _, cx| {
                if this.active_modal != ActiveModal::None { return; }
                this.open_new_folder_modal();
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &RenameModal, _, cx| {
                if this.active_modal != ActiveModal::None { return; }
                this.open_rename_modal();
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &DeleteModal, _, cx| {
                if this.active_modal != ActiveModal::None { return; }
                this.open_delete_modal();
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &PermissionsModal, _, cx| {
                if this.active_modal != ActiveModal::None { return; }
                this.open_chmod_modal();
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &SynchronizeModal, _, cx| {
                if this.active_modal != ActiveModal::None { return; }
                this.open_sync_modal(cx);
            }))
            .on_action(cx.listener(|this, _: &RevealInFinder, _, cx| {
                if this.active_modal != ActiveModal::None { return; }
                this.reveal_selected_in_finder(cx);
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &OpenInTerminal, _, cx| {
                if this.active_modal != ActiveModal::None { return; }
                this.open_selected_in_terminal(cx);
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &SelectAllItems, _, cx| {
                if this.active_modal != ActiveModal::None { return; }
                this.select_all_in_active_pane();
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &GoToFolderModal, _, cx| {
                if this.active_modal != ActiveModal::None { return; }
                this.open_go_to_folder_modal();
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &CheckForUpdates, _, cx| {
                this.check_for_updates(true, cx);
            }))
            .relative()
            .flex()
            .flex_col()
            .size_full()
            .bg(theme.root_bg())
            .text_color(theme.text_primary())
            // 1. Minimalist Top Bar
            .child(self.render_top_bar(cx))
            // 2. Dual or Single Pane Workspace
            .child(
                div()
                    .flex_1()
                    .flex()
                    .w_full()
                    .overflow_hidden()
                    .when(!self.show_local_pane, |el| {
                        el.child(
                            div()
                                .size_full()
                                .flex()
                                .flex_col()
                                .overflow_hidden()
                                .child(self.render_remote_pane(cx)),
                        )
                    })
                    .when(self.show_local_pane && !self.swap_panes, |el| {
                        el.child(
                            div()
                                .w_1_2()
                                .h_full()
                                .flex()
                                .flex_col()
                                .overflow_hidden()
                                .child(self.render_local_pane(cx)),
                        )
                        .child(div().w(px(1.0)).h_full().bg(theme.border()))
                        .child(
                            div()
                                .w_1_2()
                                .h_full()
                                .flex()
                                .flex_col()
                                .overflow_hidden()
                                .child(self.render_remote_pane(cx)),
                        )
                    })
                    .when(self.show_local_pane && self.swap_panes, |el| {
                        el.child(
                            div()
                                .w_1_2()
                                .h_full()
                                .flex()
                                .flex_col()
                                .overflow_hidden()
                                .child(self.render_remote_pane(cx)),
                        )
                        .child(div().w(px(1.0)).h_full().bg(theme.border()))
                        .child(
                            div()
                                .w_1_2()
                                .h_full()
                                .flex()
                                .flex_col()
                                .overflow_hidden()
                                .child(self.render_local_pane(cx)),
                        )
                    }),
            )
            // 4. Transfer Queue & Server Console Dock (auto-shown on server activity/transfer, always on remote server)
            .when(!self.sessions.is_empty() || self.show_queue, |el| {
                el.child(self.render_queue_resize_handle(cx))
                    .child(self.render_transfer_queue_panel(cx))
            })
            // 5. Bottom Function Bar (Total Commander F3..F8 buttons)
            .when(self.show_function_bar, |el| {
                el.child(self.render_bottom_function_bar(cx))
            })
            // 5. Connection Dropdown Popover
            .when(self.session_dropdown_open, |el| {
                el.child(self.render_session_dropdown_popover(cx))
            })
            // 6. Right-Click Context Menu
            .when_some(self.context_menu.clone(), |el, menu| {
                el.child(self.render_context_menu(&menu, cx))
            })
            // 7. Modals
            .when(self.active_modal == ActiveModal::SiteManager, |el| {
                el.child(self.render_site_manager_modal(cx))
            })
            .when(self.active_modal == ActiveModal::Permissions, |el| {
                el.child(self.render_chmod_modal(cx))
            })
            .when(self.active_modal == ActiveModal::Synchronize, |el| {
                el.child(self.render_sync_modal(cx))
            })
            .when(self.active_modal == ActiveModal::NewFolder, |el| {
                el.child(self.render_text_input_modal(
                    t.modal_new_folder_title(),
                    t.btn_create(),
                    true,
                    cx,
                ))
            })
            .when(self.active_modal == ActiveModal::Rename, |el| {
                el.child(self.render_text_input_modal(
                    t.modal_rename_title(),
                    t.btn_save(),
                    false,
                    cx,
                ))
            })
            .when(self.active_modal == ActiveModal::DeleteConfirm, |el| {
                el.child(self.render_delete_modal(cx))
            })
            .when(self.active_modal == ActiveModal::GoToFolder, |el| {
                el.child(self.render_text_input_modal(
                    t.modal_go_to_folder_title(),
                    t.btn_go(),
                    false,
                    cx,
                ))
            })
            .when(self.active_modal == ActiveModal::TransferProgress, |el| {
                el.child(self.render_transfer_modal(cx))
            })
            .when(self.active_modal == ActiveModal::TransferError, |el| {
                if let Some(prompt) = self.transfer_error_prompt.clone() {
                    el.child(self.render_transfer_error_modal(prompt, cx))
                } else {
                    el
                }
            })
            .when(self.active_modal == ActiveModal::Settings, |el| {
                el.child(self.render_settings_modal(cx))
            })
            .when(self.active_modal == ActiveModal::Update, |el| {
                el.child(self.render_update_modal(cx))
            })
    }
}

impl Drop for RusterView {
    fn drop(&mut self) {
        self.persist_app_state();
    }
}

#[cfg(target_os = "macos")]
#[allow(unexpected_cfgs)]
fn pick_application_prompt_macos() -> Option<PathBuf> {
    use objc::{class, msg_send, sel, sel_impl};
    use std::ffi::CStr;

    unsafe {
        let panel: *mut objc::runtime::Object = msg_send![class!(NSOpenPanel), openPanel];
        if panel.is_null() {
            return None;
        }
        let _: () = msg_send![panel, setCanChooseFiles: 1usize];
        let _: () = msg_send![panel, setCanChooseDirectories: 0usize];
        let _: () = msg_send![panel, setAllowsMultipleSelection: 0usize];
        let _: () = msg_send![panel, setResolvesAliases: 1usize];
        let _: () = msg_send![panel, setTreatsFilePackagesAsDirectories: 0usize];

        let apps_path: *mut objc::runtime::Object = msg_send![
            class!(NSString),
            stringWithUTF8String: b"/Applications\0".as_ptr()
        ];
        if !apps_path.is_null() {
            let apps_url: *mut objc::runtime::Object = msg_send![
                class!(NSURL),
                fileURLWithPath: apps_path
            ];
            if !apps_url.is_null() {
                let _: () = msg_send![panel, setDirectoryURL: apps_url];
            }
        }

        let app_ext: *mut objc::runtime::Object = msg_send![
            class!(NSString),
            stringWithUTF8String: b"app\0".as_ptr()
        ];
        if !app_ext.is_null() {
            let types_array: *mut objc::runtime::Object = msg_send![
                class!(NSArray),
                arrayWithObject: app_ext
            ];
            if !types_array.is_null() {
                let _: () = msg_send![panel, setAllowedFileTypes: types_array];
            }
        }

        let res: isize = msg_send![panel, runModal];
        if res == 1 {
            let url: *mut objc::runtime::Object = msg_send![panel, URL];
            if !url.is_null() {
                let path: *mut objc::runtime::Object = msg_send![url, path];
                if !path.is_null() {
                    let utf8: *const libc::c_char = msg_send![path, UTF8String];
                    if !utf8.is_null() {
                        let cstr = CStr::from_ptr(utf8);
                        let s = cstr.to_string_lossy().into_owned();
                        if !s.is_empty() {
                            return Some(PathBuf::from(s));
                        }
                    }
                }
            }
        }
        None
    }
}

