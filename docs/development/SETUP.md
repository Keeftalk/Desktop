# Developer Setup Guide

Welcome to **Keeftalk Desktop**! You can clone, build, and run Keeftalk Desktop completely locally without creating a Supabase account or configuring private credentials.

---

## 1. Prerequisites

- **Rust:** Stable toolchain (`rustup` recommended; `rust-toolchain.toml` pins `stable`, `rustfmt`, `clippy`).
- **Git:** 2.40+

### Linux (Debian / Ubuntu / Fedora)
Install native development headers for desktop windowing, keyring, and TLS:
- **Fedora:** `sudo dnf install pkg-config openssl-devel libsecret-devel wayland-devel libxkbcommon-devel sqlite-devel`
- **Ubuntu / Debian:** `sudo apt install pkg-config libssl-dev libsecret-1-dev libwayland-dev libxkbcommon-dev libsqlite3-dev`

### Windows 10 / 11
- Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) (Desktop development with C++).
- Install Rust via [rustup.rs](https://rustup.rs/).

### macOS
- Install Xcode Command Line Tools: `xcode-select --install`
- Install Rust via [rustup.rs](https://rustup.rs/).

---

## 2. Quick Start (Local Mock / Demo Mode)

```bash
git clone https://github.com/Keeftalk/Desktop.git
cd Desktop

# Linux / macOS
./scripts/dev.sh --headless-check

# Windows PowerShell
.\scripts\dev.ps1 --headless-check
```

Or invoke Cargo directly:
```bash
KEEFTALK_MODE=mock cargo run -p keeftalk-app -- --headless-check
```

---

## 3. Configuration & Self-Hosting

Copy `.env.example` to `.env` or `config/development.example.toml` to your OS config directory:
- **Linux:** `~/.config/keeftalk/config.toml`
- **Windows:** `%APPDATA%\Keeftalk\Keeftalk\config\config.toml`
- **macOS:** `~/Library/Application Support/tn.keeftalk.Keeftalk/config.toml`

Never commit `.env`, `config/local.toml`, or private credentials.
