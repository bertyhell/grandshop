#!/usr/bin/env bash
# Bumps the minor version of Grandshop (0.14.0 -> 0.15.0) in Cargo.toml and Cargo.lock.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")"

current=$(sed -n 's/^version = "\([0-9]*\.[0-9]*\.[0-9]*\)"$/\1/p' Cargo.toml | head -n 1)
if [[ -z "$current" ]]; then
    echo "Could not find the version in Cargo.toml" >&2
    exit 1
fi

IFS=. read -r major minor _ <<< "$current"
next="$major.$((minor + 1)).0"

# Only the first version line of Cargo.toml is the package's own
sed -i "0,/^version = \"$current\"$/s//version = \"$next\"/" Cargo.toml
# Cargo.lock lists the package right after its name
sed -i "/^name = \"grandshop\"$/{n;s/^version = \".*\"$/version = \"$next\"/}" Cargo.lock

echo "$current -> $next"
