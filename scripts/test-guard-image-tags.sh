#!/bin/sh
# test-guard-image-tags.sh — fixture suite for guard-image-tags.sh
#
# Feeds hand-written `docker/metadata-action` tag lists to the guard and
# asserts BOTH the exit code and the reason substring, so a fixture cannot pass
# for the wrong reason. No network, no Docker, no GitHub Actions.
#
# Accepted prerelease shapes covered: the 2-name collapsed set and the 4-name
# expanded set, because a pinned metadata-action may produce either.
# Rejected shapes covered: empty list, prerelease `latest`, prerelease name
# outside the allowed set, each missing stable alias (version, v-version,
# major.minor, major, latest), extra alias, a ref outside IMAGE, and a
# duplicate name.
#
# Run: sh scripts/test-guard-image-tags.sh

set -eu

ROOT=$(cd "$(dirname "$0")/.." && pwd)
GUARD="$ROOT/scripts/guard-image-tags.sh"
IMAGE=ghcr.io/coruairc/deepseek-build
RELEASE_VERSION=0.1.0

PASS=0
FAIL=0
FAILED_NAMES=""

# ---------------------------------------------------------------------------
# Harness helpers (same voice as scripts/test-install.sh)
# ---------------------------------------------------------------------------

say() { printf '%s\n' "$*"; }

pass() {
    PASS=$((PASS + 1))
    say "PASS: $1"
}

fail() {
    FAIL=$((FAIL + 1))
    FAILED_NAMES="$FAILED_NAMES $1"
    say "FAIL: $1"
    if [ -n "${2:-}" ]; then
        say "      detail: $2"
    fi
}

# tags <ref>... -> newline-separated list (a "" argument becomes a blank line)
tags() {
    printf '%s\n' "$@"
}

# check_v <name> <ok|fail> <is_prerelease> <version> <tags> [expected-substring...]
# Runs one fixture with a non-default RELEASE_VERSION (used for rc shapes).
check_v() {
    _v_name=$1
    _v_expect=$2
    _v_prerelease=$3
    _v_version=$4
    shift 4
    _saved_version=$RELEASE_VERSION
    RELEASE_VERSION=$_v_version
    check "$_v_name" "$_v_expect" "$_v_prerelease" "$@"
    RELEASE_VERSION=$_saved_version
}

# check <name> <ok|fail> <is_prerelease> <tags> [expected-substring...]
check() {
    _name=$1
    _expect=$2
    _prerelease=$3
    _list=$4
    shift 4
    if _out=$(IMAGE="$IMAGE" TAGS="$_list" RELEASE_VERSION="$RELEASE_VERSION" \
        IS_PRERELEASE="$_prerelease" sh "$GUARD" 2>&1); then
        _rc=0
    else
        _rc=$?
    fi
    if [ "$_expect" = "ok" ] && [ "$_rc" -ne 0 ]; then
        fail "$_name" "expected exit 0, got ${_rc}; output: ${_out}"
        return
    fi
    if [ "$_expect" = "fail" ] && [ "$_rc" -eq 0 ]; then
        fail "$_name" "expected a non-zero exit, got 0; output: ${_out}"
        return
    fi
    if [ "$_expect" = "fail" ]; then
        case "$_out" in
            *"::error::"*) ;;
            *)
                fail "$_name" "failure did not emit a ::error:: line; output: ${_out}"
                return
                ;;
        esac
    fi
    _missing=""
    for _needle in "$@"; do
        case "$_out" in
            *"$_needle"*) ;;
            *) _missing="${_missing} [${_needle}]" ;;
        esac
    done
    if [ -n "$_missing" ]; then
        fail "$_name" "output missing:${_missing}; output: ${_out}"
        return
    fi
    pass "$_name"
}

# ---------------------------------------------------------------------------
# Accepted shapes
# ---------------------------------------------------------------------------

check stable-exact ok false "$(tags \
    "${IMAGE}:0.1.0" \
    "${IMAGE}:v0.1.0" \
    "${IMAGE}:0.1" \
    "${IMAGE}:0" \
    "${IMAGE}:latest")" \
    "guard-image-tags: OK 5 names (stable)" \
    "resolved tag names (5):" \
    "  latest"

check stable-shuffled ok false "$(tags \
    "${IMAGE}:latest" \
    "${IMAGE}:0" \
    "${IMAGE}:v0.1.0" \
    "${IMAGE}:0.1" \
    "${IMAGE}:0.1.0")" \
    "guard-image-tags: OK 5 names (stable)"

check stable-with-blank-lines ok false "$(tags \
    "${IMAGE}:0.1.0" \
    "" \
    "${IMAGE}:v0.1.0" \
    "${IMAGE}:0.1" \
    "${IMAGE}:0" \
    "" \
    "${IMAGE}:latest")" \
    "guard-image-tags: OK 5 names (stable)"

check prerelease-two-names ok true "$(tags \
    "${IMAGE}:0.1.0" \
    "${IMAGE}:v0.1.0")" \
    "guard-image-tags: OK 2 names (prerelease)" \
    "resolved tag names (2):"

check prerelease-four-names ok true "$(tags \
    "${IMAGE}:0.1.0" \
    "${IMAGE}:v0.1.0" \
    "${IMAGE}:0.1" \
    "${IMAGE}:0")" \
    "guard-image-tags: OK 4 names (prerelease)" \
    "  0.1" \
    "  0.1.0"

check_v prerelease-rc-two-names ok true 0.1.0-rc1 "$(tags \
    "${IMAGE}:0.1.0-rc1" \
    "${IMAGE}:v0.1.0-rc1")" \
    "guard-image-tags: OK 2 names (prerelease)"

check_v prerelease-rc-four-names ok true 0.1.0-rc1 "$(tags \
    "${IMAGE}:0.1.0-rc1" \
    "${IMAGE}:v0.1.0-rc1" \
    "${IMAGE}:0.1" \
    "${IMAGE}:0")" \
    "guard-image-tags: OK 4 names (prerelease)"

# ---------------------------------------------------------------------------
# Rejected shapes
# ---------------------------------------------------------------------------

check empty-list fail false "" "tag list is empty"

check prerelease-with-latest fail true "$(tags \
    "${IMAGE}:0.1.0" \
    "${IMAGE}:v0.1.0" \
    "${IMAGE}:0.1" \
    "${IMAGE}:0" \
    "${IMAGE}:latest")" \
    "'latest' must not be present"

check prerelease-outside-set fail true "$(tags \
    "${IMAGE}:0.1.0" \
    "${IMAGE}:v0.1.0" \
    "${IMAGE}:0.3")" \
    "'0.3' is not allowed for a prerelease"

check stable-missing-version fail false "$(tags \
    "${IMAGE}:v0.1.0" \
    "${IMAGE}:0.1" \
    "${IMAGE}:0" \
    "${IMAGE}:latest")" \
    "'0.1.0' must appear exactly once"

check stable-missing-vversion fail false "$(tags \
    "${IMAGE}:0.1.0" \
    "${IMAGE}:0.1" \
    "${IMAGE}:0" \
    "${IMAGE}:latest")" \
    "'v0.1.0' must appear exactly once"

check stable-missing-minor fail false "$(tags \
    "${IMAGE}:0.1.0" \
    "${IMAGE}:v0.1.0" \
    "${IMAGE}:0" \
    "${IMAGE}:latest")" \
    "'0.1' must appear exactly once"

check stable-missing-major fail false "$(tags \
    "${IMAGE}:0.1.0" \
    "${IMAGE}:v0.1.0" \
    "${IMAGE}:0.1" \
    "${IMAGE}:latest")" \
    "'0' must appear exactly once"

check stable-missing-latest fail false "$(tags \
    "${IMAGE}:0.1.0" \
    "${IMAGE}:v0.1.0" \
    "${IMAGE}:0.1" \
    "${IMAGE}:0")" \
    "'latest' must appear exactly once"

check stable-extra-alias fail false "$(tags \
    "${IMAGE}:0.1.0" \
    "${IMAGE}:v0.1.0" \
    "${IMAGE}:0.1" \
    "${IMAGE}:0" \
    "${IMAGE}:latest" \
    "${IMAGE}:0.2")" \
    "'0.2' is not in the allowed set"

check ref-outside-image fail false "$(tags \
    "${IMAGE}:0.1.0" \
    "${IMAGE}:v0.1.0" \
    "${IMAGE}:0.1" \
    "${IMAGE}:0" \
    "${IMAGE}:latest" \
    "ghcr.io/someone-else/deepseek-build:0.1.0")" \
    "is not under"

check duplicate-version fail false "$(tags \
    "${IMAGE}:0.1.0" \
    "${IMAGE}:v0.1.0" \
    "${IMAGE}:0.1" \
    "${IMAGE}:0" \
    "${IMAGE}:latest" \
    "${IMAGE}:0.1.0")" \
    "duplicate tag name"

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------

say ""
if [ "$FAIL" -ne 0 ]; then
    say "FAILED:$FAILED_NAMES"
    say "${PASS}/$((PASS + FAIL)) assertions passed, ${FAIL} failed"
    exit 1
fi
say "${PASS}/${PASS} assertions passed, 0 failed"
