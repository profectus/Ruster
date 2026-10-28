# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.8] - 2026-10-28

### Fixed
- **Window Size Height Inflation on Launch**: Resolved a macOS titlebar height mismatch where saving `window.bounds()` (window frame including titlebar) caused the content area to expand by ~40px on every app restart. Switched to `window.viewport_size()`, ignored size updates in fullscreen mode, and clamped initial bounds to primary display dimensions.
- **Settings Toggle Switch & Text Overflow**: Fixed long localized setting labels pushing and squeezing toggle switches into 20px circles. Added `flex_shrink_0()` to switch pills, `whitespace_normal()` and `min_w_0()` for multiline wrapping, and unified concise description texts across all supported languages.

## [0.3.4] - 2026-10-28

### Added
- **Automatic Windows-1250 (CP1250) Fallback**: When downloading files from legacy Windows IIS FTP servers that store filenames in ANSI/CP1250 instead of UTF-8, Ruster now automatically recovers and retries with CP1250 byte percent-encoding if UTF-8 fails with error 78.
- **Server Error Details in Dialog**: Transfer error modals now capture and display exact error responses from the remote server (e.g. `550 Access denied`) instead of generic exit codes.

### Fixed
- **Multiple Consecutive Spaces in FTP Listings**: Fixed directory parser collapsing consecutive spaces into a single space, resolving "File not found" (error 78) download failures on files with double spaces.
- **Smooth Batch Progress Bar**: Fixed progress bar resetting and turning dark gray between files; progress bar remains active and smoothly increments across the entire batch.
- **Modal Filename Wrapping**: Prevented long filenames and error descriptions from overflowing the transfer error dialog by enabling multiline wrapping.
- **Parallel Connections Respect Settings**: Ensured transfers respect both site-specific and global concurrent connection settings for true parallel downloads and uploads.

## [0.3.3] - 2026-10-28

### Added
- **Vertically Resizable Transfer Queue**: The bottom transfer queue panel can now be resized vertically by dragging its top border handle, resets to default on double-click, and remembers its height across app restarts.
- **FTP UTF-8 & Czech Character Encoding Support**: Added automatic UTF-8 negotiation (`OPTS UTF8 ON`) and encoding fallback (Windows-1250, ISO-8859-2) for FTP listings so Czech accented characters display properly.
- **Persistent Show Hidden Files**: Toggling hidden files (`Cmd+Shift+.` or in Settings) now persists to configuration across app restarts.

### Changed
- **Immediate Settings Auto-Save & Streamlined UI**: Removed the redundant "Uložit a zavřít" (Save & Close) footer button from Settings modal since all configuration changes save immediately to disk.
- **Centralized Search Action**: Removed redundant search icons from individual pane headers in favor of the primary search action in the main toolbar.

### Fixed
- **Queue Cleanup on Cancel**: Canceling transfers from the queue, progress bar, or error dialog now immediately stops worker threads and empties the transfer queue cleanly.
- **Removed Broken File List Scrollbar**: Cleaned up the non-functional scrollbars from file listing panels.

## [0.3.2] - 2026-10-28

### Added
- **Remote Synchronization Quick Action**: Added a direct synchronization icon to the top toolbar when connected to an active remote server, opening the directory synchronization modal with one click.
- **Interactive Transfer Error Recovery**: Transfers now pause on failed uploads/downloads/copies and present an action dialog with options to Retry (Zkusit znovu), Skip (Přeskočit), Skip All (Přeskočit vše), or Cancel (Zrušit přenos).
- **Collapsible Transfer Log**: The server transfer log is now hidden by default in the bottom transfer panel, allowing the file queue and progress bars to occupy full width. Added a quick toggle button to reveal/hide the log on demand.

## [0.3.1] - 2026-10-26

### Changed
- **Official Rebranding to Ruster**: Completed the rebranding of all binaries, `.app` bundle, Homebrew cask formula, and configurations to **Ruster**.
- **Social & Share Assets**: Added new high-resolution social share and GitLab Open Graph previews with brand badges and refreshed screenshot titles.

## [0.3.0] - 2026-10-26

### Changed
- **Project Rebranding to Ruster**: Renamed the entire project, binaries, app bundle, Homebrew cask, and configurations from MacSCP to **Ruster**.

### Added
- **One-Click In-App Updater**: Added a one-click **"Aktualizovat nyní"** (Update Now) button directly to the update modal. Automatically updates via Homebrew if managed by Homebrew, or downloads, unpacks, un-quarantines, and replaces `/Applications/Ruster.app` directly from GitHub releases without opening the Terminal.
- **One-Click Restart**: Added a **"Restartovat Ruster"** button upon update completion that cleanly re-launches the newly updated Ruster instance.

## [0.2.9] - 2026-10-26

### Added
- **Transfers & Concurrent Connections Setting in Settings Modal**: Added configuration control for maximum concurrent transfers (1 to 10) in global application Settings (`Cmd+,`).
- **Concurrent Transfer Visual Indicators**: Batch header in the bottom transfer bar now shows total aggregate speed across all active worker threads and displays the number of active parallel transfers (e.g. `— 3 paralelně`).

## [0.2.8] - 2026-10-26

### Removed
- **Unformatted Changelog Box in Update Modal**: Removed the unformatted raw Markdown/URL changelog box from the update dialog in favor of a clean dialog layout with version comparisons, direct Homebrew upgrade command, and a primary GitHub release action button.

## [0.2.7] - 2026-10-26

### Fixed
- **Modal Overlay Hover Leak**: Suppressed file row tooltips and drag handlers whenever any modal dialog (Connection Manager, Synchronize, Settings, Permissions) or context menu is open, preventing background file tooltips from appearing over open modal dialogs.

## [0.2.6] - 2026-10-26

### Added
- **Asynchronous Connection & Progress Feedback**:
  - Connection attempts from Connection Manager now run completely in the background via dedicated worker thread and non-blocking polling, preventing window freezes while establishing SSH/FTP connections.
  - Interactive "Connecting..." button state with animated spinner and click-lock to prevent repeated connection requests.
  - Informational connecting banner displayed directly in the Site Manager dialog (`Connecting to <host>:<port>...`) with error fallback on connection failure.
  - Full localization in English, Czech, German, Spanish, French, Italian, Polish, Portuguese, Chinese, and Japanese.

## [0.2.5] - 2026-10-26

### Fixed
- **Non-Blocking Directory Synchronization**:
  - Replaced blocking thread join with a non-blocking background thread and asynchronous message queue, eliminating UI freezes and beachball cursors during directory comparison.
  - Added real-time scanning progress indicator in the Synchronize modal (`Scanning local...`, `Scanning remote...`, count of discovered items).
  - Maintained 120fps UI responsiveness so the window can be moved, resized, or closed freely while scanning remote files.

## [0.2.4] - 2026-10-26

### Fixed
- **File Name Tooltip Position**: Added vertical spacing (`margin-top: 14px`) to the full file name hover tooltip so it renders comfortably below the cursor and does not obstruct the active row.

## [0.2.3] - 2026-10-26

### Added
- **SSH Config Import Feedback & Notifications**:
  - Renamed Site Manager import button to clear action label (`Import SSH` / `Importovat SSH`) across all 10 supported languages.
  - Interactive status banner in the Site Manager providing immediate, unambiguous feedback:
    - Success notification with exact count of newly imported servers.
    - Informational notice when all servers in `~/.ssh/config` are already present in the saved list.
    - Helpful alert when `~/.ssh/config` does not exist or does not contain any valid host configurations.
  - Full localization in English, Czech, German, Spanish, French, Italian, Polish, Portuguese, Chinese, and Japanese.
  - Unit tests covering SSH config parsing, wildcard host exclusion, and multilingual import messages.

## [0.2.2] - 2026-10-26

### Added
- **Per-Panel File & Directory Filtering (`Cmd+F`)**:
  - Filter bar toggled in both panels simultaneously with `Cmd+F` or magnifying glass icon buttons in the panel headers and main toolbar.
  - Independent filtering per panel: each panel maintains its own filter query and applies it strictly to its own entries.
  - Clear visual focus indicator with active border, icon color, and cursor. Easily switch between panel filters with `Tab` or mouse click.
  - Real-time result counter (`visible/total`) in the filter bar and status bar that excludes the parent `..` directory.
  - Seamless clearing via `Escape` (clears active query on first press, closes both filter bars on second press).
- **Environment & Connection Colors**:
  - 5 customizable environment colors selectable in the Site Manager (Red, Orange, Blue, Green, Purple).
  - Selected items in the panel are highlighted with the connection's custom accent color.
  - Minimalist colored indicator dot displayed next to the server name in the remote panel header and session switcher.
- **Connection Manager Double-Click Connect**:
  - Double-clicking any saved profile in the Site Manager immediately connects to the server.
- **Full File Name Hover Tooltip**:
  - Hovering over any file or directory row reveals a native GPUI tooltip with the complete file name after a 500ms delay, preventing issues with long truncated names.

## [0.2.1] - 2026-10-26

### Fixed
- **Synchronize Modal Layout**: Fixed horizontal overflowing of synchronization options (checkboxes and compare button) by introducing flex-wrapping and concise, polished localization strings across all 10 supported languages.
- **Site Manager Modal Layout**: Pinned the *Save* and *Connect* footer buttons with `flex_shrink_0()` to prevent them from being pushed outside the modal dialog when connection errors or long hostnames/messages are displayed.
- **Error Banners Overflow Protection**: Added dedicated scrollable containers (`overflow_y_scroll()`) for connection and synchronization error messages to prevent layout displacement.

## [0.2.0] - 2026-10-26

### Added
- **Full Directory Synchronization (`Cmd+S`)**:
  - Compare local and remote (or dual local) directories with recursive subfolder scanning.
  - Direction selection: **Local ➔ Remote (Upload)**, **Remote ➔ Local (Download)**, and **Both Directions ⇄ (Two-Way)**.
  - Flexible comparison criteria: By modification timestamp & file size, timestamp only, or size only.
  - Optional **Mirror Mode**: Remove obsolete target files that no longer exist in the source directory.
  - Interactive Diff Preview: Review missing, newer, and size-differing files with per-file checkboxes, summary stats, and "Select All" / "Deselect All" toggles.
  - Seamless background execution via the transfer queue with real-time transfer progress, speed, and auto-refresh on completion.
  - Complete internationalization across all 10 supported languages.

## [0.1.1] - 2026-10-26

### Fixed
- Fixed rounded corner clipping artifacts in light mode where square header and footer backgrounds bled outside modal dialog curves (Settings, Site Manager, Update).
- Fixed modal body flex sizing in Settings dialog to ensure scrollable content does not compress the footer button.

## [0.1.0] - 2026-10-25

### Added
- Initial release of Ruster (originally MacSCP): Native dual-pane SFTP/SCP client for macOS built with Rust and GPUI.
- Dual-pane file management (local and remote).
- Fast multi-threaded transfers over SFTP, SCP, and FTP.
- Keyring integration for secure password storage.
- Remote file editing with external editors (VS Code, TextEdit, etc.).
- Light and dark theme support.
- Multi-language interface support (English, Czech, German, Spanish, French, Italian, Polish, Portuguese, Chinese, Japanese).
- Automatic updates via GitHub Releases.
