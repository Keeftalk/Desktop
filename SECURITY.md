# Security Policy

Keeftalk Desktop follows a zero-knowledge, offline-first security architecture with zero advertisements and zero behavioral telemetry.

## Reporting a Vulnerability

Please **do not** open a public GitHub issue for security vulnerabilities.

Report vulnerabilities privately via **GitHub Private Vulnerability Reporting** (`Security -> Advisories -> Report a vulnerability` on `Keeftalk/Desktop`).

Include:
1. Summary and affected crate/component (`keeftalk-crypto`, `keeftalk-database`, `keeftalk-auth`, `keeftalk-platform`, etc.).
2. Reproduction steps or proof of concept.
3. Suggested mitigation if known.

## Credential & Secret Hygiene

- Never commit `.env`, `config/local.toml`, service-role keys, signing certificates, or developer tokens.
- Production builds must store local database passphrases and wrapped keys exclusively in the OS credential store (Windows Credential Manager, macOS Keychain, Linux Secret Service) without silent plaintext fallbacks.
