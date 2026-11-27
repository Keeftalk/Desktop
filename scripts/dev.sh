#!/usr/bin/env bash
set -euo pipefail

export KEEFTALK_MODE="${KEEFTALK_MODE:-mock}"
export RUST_LOG="${RUST_LOG:-info,keeftalk=debug}"

cargo run -p keeftalk-app -- "$@"
