#!/usr/bin/env bash
# Builds an optimised Grandshop binary in target/release.
# Pass --package to also build the installers for this OS into target/packages.
set -euo pipefail
cd "$(dirname "$0")"

# rustup installs cargo here, but not every shell has it on the PATH
if ! command -v cargo >/dev/null && [ -f "$HOME/.cargo/env" ]; then
    source "$HOME/.cargo/env"
fi

if [ "${1:-}" = "--package" ]; then
    cargo packager --release
    echo "Installers are in target/packages"
else
    cargo build --release
    echo "Built target/release/grandshop"
fi
