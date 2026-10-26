# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
- Initial release of MacSCP: Native dual-pane SFTP/SCP client for macOS built with Rust and GPUI.
- Dual-pane file management (local and remote).
- Fast multi-threaded transfers over SFTP, SCP, and FTP.
- Keyring integration for secure password storage.
- Remote file editing with external editors (VS Code, TextEdit, etc.).
- Light and dark theme support.
- Multi-language interface support (English, Czech, German, Spanish, French, Italian, Polish, Portuguese, Chinese, Japanese).
- Automatic updates via GitHub Releases.
