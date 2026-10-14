#!/bin/sh
# Stage a copy of the crate in .cache/crates-io that depends on our iced and wgpu
# forks as published on crates.io (iced-kora, wgpu-kora), since crates.io rejects
# git dependencies. playtron-os/iced publishes them; see its kora/README.md.
set -eu

ICED_VERSION=0.15
WGPU_VERSION=28
out=.cache/crates-io

rm -rf "$out"
mkdir -p "$out"
git ls-files -z --cached --others --exclude-standard |
    xargs -0 sh -c 'for f; do if [ -f "$f" ]; then cp --parents "$f" "$0"; fi; done' "$out"
cd "$out"

sed -i \
    -e "s|^git = \"https://github.com/playtron-os/iced.git\"\$|package = \"iced-kora\"\nversion = \"$ICED_VERSION\"|" \
    -e "s|^git = \"https://github.com/playtron-os/wgpu.git\"\$|package = \"wgpu-kora\"\nversion = \"$WGPU_VERSION\"|" \
    -e '/^rev = "/d' \
    Cargo.toml
rm -f Cargo.lock

if grep -nE '^[^#]*(git|rev|branch) *=' Cargo.toml; then
    echo "crates-io.sh: Cargo.toml still has a git dependency" >&2
    exit 1
fi
