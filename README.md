<p align="center">
  <img src="assets/icon.png" alt="Ruster App Icon" width="128" height="128">
</p>

<h1 align="center">Ruster</h1>

<p align="center">
  <strong>Native, lightning-fast dual-pane SFTP & SCP client for macOS built with Rust & GPUI.</strong>
</p>

<p align="center">
  <a href="https://github.com/profectus/ruster/releases"><img src="https://img.shields.io/github/v/release/profectus/ruster?style=flat-square&color=2563eb" alt="GitHub Release"></a>
  <a href="https://github.com/profectus/ruster/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square" alt="License: MIT"></a>
  <a href="https://apple.com/macos"><img src="https://img.shields.io/badge/platform-macOS%2012%2B-lightgrey.svg?style=flat-square" alt="macOS 12+"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/built%20with-Rust%20%26%20GPUI-orange.svg?style=flat-square" alt="Rust & GPUI"></a>
  <a href="https://buymeacoffee.com/filipstencl"><img src="https://img.shields.io/badge/Buy%20Me%20a%20Coffee-filipstencl-FFDD00.svg?style=flat-square&logo=buy-me-a-coffee&logoColor=black" alt="Buy Me A Coffee"></a>
</p>

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="assets/screenshot-dark.png">
    <source media="(prefers-color-scheme: light)" srcset="assets/screenshot-light.png">
    <img alt="Ruster Dual-Pane Interface" src="assets/screenshot-dark.png" width="88%">
  </picture>
</p>

---

## Screenshots

| Dark Theme | Light Theme |
| :---: | :---: |
| <img src="assets/screenshot-dark.png" width="100%" alt="Ruster Dark Mode"> | <img src="assets/screenshot-light.png" width="100%" alt="Ruster Light Mode"> |

---

## Features

- **GPU-Accelerated Performance** — Built on Zed's modern GPUI framework with 120fps fluid scrolling, instant startup, and zero bloat.
- **Classic Dual-Pane Commander** — Effortlessly browse and manage local and remote files side by side. Toggle between dual-pane and single-pane modes (`Cmd+D`) or swap panes (`Cmd+U`).
- **Light & Dark Themes** — Pixel-perfect Light and Dark themes with native macOS dark titlebar synchronization.
- **10 Built-in World Languages** — Full localization available directly from Settings (`Cmd+,`):
  - 🇬🇧 English, 🇨🇿 Čeština, 🇩🇪 Deutsch, 🇪🇸 Español, 🇫🇷 Français, 🇮🇹 Italiano, 🇵🇱 Polski, 🇵🇹 Português, 🇨🇳 简体中文, 🇯🇵 日本語.
- **Site Manager & Bookmarks** — Save connections with custom environment color badges (Production, Staging, Dev), private key authentication, and custom SSH ports.
- **Background Transfer Queue** — Multi-item queue with real-time transfer speeds, progress bars, auto-retry, and batch controls.
- **Remote File Editing & Auto-Upload** — Edit remote files directly in your favorite local editor with automatic upload on save.
- **Visual chmod Calculator** — Inspect and set permissions via octal input and interactive bit checkboxes with recursion support.
- **Directory Synchronization** — Compare local and remote folders and synchronize missing or newer files with dry-run previews.
- **Instant Quick Filter** — Filter thousands of files in milliseconds using `Cmd+F`.

---

## Installation

### Option 1: Via Homebrew (Recommended)

You can install Ruster directly via Homebrew:

```bash
# Add the Ruster tap and trust it
brew tap profectus/ruster https://github.com/profectus/ruster.git
brew trust profectus/ruster

# Install Ruster
brew install --cask ruster
```

### Option 2: Direct Download

1. Download the latest `Ruster.zip` from [GitHub Releases](https://github.com/profectus/ruster/releases).
2. Unpack `Ruster.zip`.
3. Drag `Ruster.app` into your `/Applications` folder.

> [!TIP]
> **macOS Gatekeeper ("App cannot be opened" / "Apple cannot verify...")**:  
> Since Ruster is an open-source project and built without a paid Apple Developer certificate, macOS Gatekeeper may show a security prompt upon first opening. To open it:
> - In **System Settings → Privacy & Security**, scroll down and click **Open Anyway**.
> - Or in Terminal, run: `xattr -dr com.apple.quarantine /Applications/Ruster.app`
> - Or install via Homebrew with: `brew install --cask --no-quarantine ruster`

---

## Updates

### Updating via Homebrew
Whenever a new version is released, simply run:

```bash
brew update && brew upgrade --cask ruster
```

### In-App Update Notifications
Ruster includes a built-in update checker:
- **Automatic Check**: On launch, Ruster quietly checks GitHub Releases for new updates. If a newer version is available, an update window notifies you with release details.
- **Manual Check**: You can manually verify updates at any time by choosing **Ruster → Check for Updates...** from the menu bar or clicking **Check for Updates** in Settings (`Cmd+,`).
- If you use Homebrew, the dialog provides a one-click button to copy `brew update && brew upgrade --cask ruster` to your clipboard.

---

## Keyboard Shortcuts

| Shortcut | Action |
| :--- | :--- |
| <kbd>Cmd</kbd> + <kbd>O</kbd> | Open Site Manager |
| <kbd>Cmd</kbd> + <kbd>D</kbd> | Toggle Dual Pane / Single Pane |
| <kbd>Cmd</kbd> + <kbd>U</kbd> | Swap Left and Right Panes |
| <kbd>Cmd</kbd> + <kbd>R</kbd> | Refresh File Lists |
| <kbd>Cmd</kbd> + <kbd>F</kbd> | Toggle Quick Filter |
| <kbd>Cmd</kbd> + <kbd>,</kbd> | Settings & Language Dropdown |
| <kbd>Cmd</kbd> + <kbd>Shift</kbd> + <kbd>.</kbd> | Toggle Hidden Files |
| <kbd>Cmd</kbd> + <kbd>Shift</kbd> + <kbd>G</kbd> | Go to Folder |
| <kbd>Cmd</kbd> + <kbd>I</kbd> | File Permissions (chmod) |
| <kbd>F5</kbd> | Copy / Transfer Selected |
| <kbd>F6</kbd> | Move / Rename Selected |
| <kbd>F7</kbd> | Create New Folder |
| <kbd>F8</kbd> / <kbd>Backspace</kbd> | Delete Selected |
| <kbd>Space</kbd> | Multi-select Toggle |

---

## Building from Source

### Prerequisites
- macOS 12.0 (Monterey) or later
- [Rust toolchain](https://rustup.rs/) (stable)
- Xcode Command Line Tools (`xcode-select --install`)

### Build Steps

```bash
# Clone the repository
git clone https://github.com/profectus/ruster.git
cd ruster

# Run tests
cargo test

# Build and package the release application bundle
./bundle.sh
```

The compiled and signed `.app` bundle will be ready at `target/release/Ruster.app`, and a distributable archive will be at `target/release/Ruster.zip`.

---

## Support the Project

If you love Ruster and want to support its ongoing development and maintenance, you can buy me a coffee! Any support is greatly appreciated.

<p align="center">
  <a href="https://buymeacoffee.com/filipstencl" target="_blank">
    <img src="https://cdn.buymeacoffee.com/buttons/v2/default-yellow.png" alt="Buy Me A Coffee" height="50">
  </a>
</p>

---

## License

This project is licensed under the [MIT License](LICENSE).
