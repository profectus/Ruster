# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
