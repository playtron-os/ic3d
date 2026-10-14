#!/bin/sh
# Stage a copy of the crate in .cache/crates-io that builds against the crates.io
# releases of iced and wgpu. crates.io rejects the git forks the repository uses.
set -eu

ICED_VERSION=0.14
WGPU_VERSION=27
out=.cache/crates-io

rm -rf "$out"
mkdir -p "$out"
git ls-files -z --cached --others --exclude-standard |
    xargs -0 sh -c 'for f; do if [ -f "$f" ]; then cp --parents "$f" "$0"; fi; done' "$out"
cd "$out"

sed -i \
    -e "s|^git = \"https://github.com/playtron-os/iced.git\"\$|version = \"$ICED_VERSION\"|" \
    -e "s|^git = \"https://github.com/playtron-os/wgpu.git\"\$|version = \"$WGPU_VERSION\"|" \
    -e '/^rev = "/d' \
    Cargo.toml
rm -f Cargo.lock

# wgpu 28 renamed RenderPipelineDescriptor::multiview to multiview_mask.
grep -rl 'multiview_mask:' src | xargs -r sed -i 's/multiview_mask:/multiview:/'

if grep -nE '^[^#]*(git|rev|branch) *=' Cargo.toml; then
    echo "crates-io.sh: Cargo.toml still has a git dependency" >&2
    exit 1
fi
