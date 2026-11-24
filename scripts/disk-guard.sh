#!/bin/sh
# Refuse to start a heavy build when the filesystem holding the cargo target
# dir is low on space. Prints the free space and exits non-zero below the
# threshold (default 60 GB).
#
# Why 60 GB: a measured full `cargo build --workspace --tests` is ~24 GB
# (2026-10-12), so the default leaves ~2.5x headroom. The old "150 GB" figure
# came from the incident where incremental builds + full debug info grew
# target/debug to 244 GB; both are off now (`[build] incremental = false`,
# `debug = "line-tables-only"`), so 150 GB is no longer the bar.
#
# Usage: scripts/disk-guard.sh [minimum_gb]
# Call it before any workspace build or test; stop when it fails.

set -eu

min="${1:-60}"
root=$(unset CDPATH; cd -- "$(dirname -- "$0")/.." && pwd)
target="${CARGO_TARGET_DIR:-$root/target}"

probe="$target"
if [ ! -e "$probe" ]; then
    probe="$root"
fi

free_kb=$(df -Pk -- "$probe" | awk 'NR==2 { print $4 }')
free_gb=$((free_kb / 1024 / 1024))
echo "disk-guard: ${free_gb} GB free on the target filesystem (minimum ${min} GB): ${probe}"

[ "$free_gb" -ge "$min" ]
