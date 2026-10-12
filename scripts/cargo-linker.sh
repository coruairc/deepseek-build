#!/bin/sh
# Linker wrapper referenced by `.cargo/config.toml`.
#
# Prefers lld (faster, lower peak memory) and silently falls back to the system
# C compiler, so a machine without lld still builds. Extra arguments passed by
# rustc are forwarded unchanged.
set -eu

driver=""
for cand in cc gcc clang; do
    if command -v "$cand" >/dev/null 2>&1; then
        driver="$cand"
        break
    fi
done
if [ -z "$driver" ]; then
    echo "cargo-linker: no C compiler (cc/gcc/clang) found on PATH" >&2
    exit 1
fi

if command -v ld.lld >/dev/null 2>&1 || command -v lld >/dev/null 2>&1; then
    exec "$driver" -fuse-ld=lld "$@"
fi
exec "$driver" "$@"
