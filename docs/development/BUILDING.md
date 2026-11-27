# Building Keeftalk Desktop

## Debug Build
```bash
cargo build --workspace
```

## Release Build
```bash
# Linux / macOS
./scripts/build.sh

# Windows PowerShell
.\scripts\build.ps1
```

Or with Cargo directly:
```bash
cargo build --workspace --release
```

The release binary is output at `target/release/keeftalk-desktop` (or `keeftalk-desktop.exe` on Windows).
