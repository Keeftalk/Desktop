# Testing Keeftalk Desktop

## Running Unit & Workspace Tests
```bash
# Linux / macOS
./scripts/test.sh

# Windows PowerShell
.\scripts\test.ps1
```

Or via Cargo:
```bash
cargo test --workspace --all-targets
```

## Linting & Formatting
```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
```
