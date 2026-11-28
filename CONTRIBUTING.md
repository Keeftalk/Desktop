# Contributing to Keeftalk Desktop

Thank you for helping build **Keeftalk Desktop**!

## Core Principles
1. **Open Source & Free:** Never gate local application features behind subscriptions or telemetry.
2. **Zero-Knowledge Privacy:** Never log plaintext messages, keys, passwords, or tokens.
3. **Never Commit Directly to `main`:** Always create a topic branch (`feature/...`, `fix/...`, `docs/...`, `ci/...`) and open a Pull Request targeting `main`.

## Local Workflow
```bash
git clone https://github.com/YOUR_USERNAME/Desktop.git
cd Desktop
./scripts/test.sh
./scripts/dev.sh --headless-check
```

## Pull Request Checklist
1. Format with `cargo fmt --all`
2. Verify `cargo test --workspace` passes
3. Verify no `.env` or private credentials are staged
4. Use conventional commit messages (`feat(ui): ...`, `fix(db): ...`, `docs: ...`)
