$ErrorActionPreference = "Stop"

if (-not $env:KEEFTALK_MODE) { $env:KEEFTALK_MODE = "mock" }
if (-not $env:RUST_LOG) { $env:RUST_LOG = "info,keeftalk=debug" }

cargo run -p keeftalk-app -- @args
