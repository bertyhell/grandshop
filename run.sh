#!/usr/bin/env bash
# Builds (when needed) and starts Grandshop. Extra arguments go to the app,
# e.g. ./run.sh photo.jpg opens that photo.
set -euo pipefail
# Work from any folder without changing it, so relative paths in arguments keep working
manifest="$(cd "$(dirname "$0")" && pwd)/Cargo.toml"

# rustup installs cargo here, but not every shell has it on the PATH
if ! command -v cargo >/dev/null && [ -f "$HOME/.cargo/env" ]; then
    source "$HOME/.cargo/env"
fi

exec cargo run --release --manifest-path "$manifest" -- "$@"
