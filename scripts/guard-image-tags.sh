#!/bin/sh
# guard-image-tags.sh — fail loudly on a bad docker/metadata-action tag list.
#
# Contract (all required in the environment):
#   IMAGE            image name without a tag, e.g. ghcr.io/coruairc/deepseek-build
#   TAGS            the newline-separated `steps.meta.outputs.tags` value
#   RELEASE_VERSION the version derived by the `version` job (e.g. 0.1.0)
#   IS_PRERELEASE   "true" or "false"
#
# It MUST run after `Extract metadata (tags + labels)` and BEFORE the first
# `docker/build-push-action` step. Those build steps publish the tag list
# themselves — including `:latest` for a stable tag — so a guard placed after
# them is already too late. A non-semver `v*` tag makes `type=ref,event=tag`
# emit e.g. `:v0.1` and `:latest`; this script refuses both before any push.
#
# Accepted lists:
#   stable (IS_PRERELEASE=false), each name exactly once:
#     <version>  v<version>  <major>.<minor>  <major>  latest
#   prerelease (IS_PRERELEASE=true):
#     <version> and v<version> REQUIRED exactly once each;
#     <major>.<minor> and <major> OPTIONAL (at most once each) — a pinned
#     metadata-action collapses the semver aliases to the full prerelease
#     string (2 names) on some paths and emits them expanded (4 names) on
#     others, so both shapes are accepted;
#     `latest` is FORBIDDEN, and so is any name outside
#     {<version>, v<version>, <major>.<minor>, <major>}.
#
# DECISION — the major alias is kept: `{{major}}` resolves to `0` for `v0.1.0`,
# and that `0` tag is ACCEPTED and published as `ghcr.io/...:0`, not suppressed.
# Do not add an `enable=` conditional for it; the stable list is exactly the
# five names above, so dropping `0` would fail this guard.
#
# Every failure prints a GitHub `::error::` annotation naming the reason and
# exits 1. On success it prints `guard-image-tags: OK <n> names (<kind>)`.
#
# Usage (CI):   scripts/guard-image-tags.sh
# Usage (test): IMAGE=... TAGS=... RELEASE_VERSION=... IS_PRERELEASE=... \
#               sh scripts/guard-image-tags.sh
#
# POSIX sh on purpose: no bash arrays and no pipefail (this repo lints with
# `shellcheck --shell=sh`).

set -eu

fail() {
    printf '::error::guard-image-tags: %s\n' "$*" >&2
    exit 1
}

[ -n "${IMAGE:-}" ] || fail "IMAGE is not set"
[ -n "${RELEASE_VERSION:-}" ] || fail "RELEASE_VERSION is not set"
[ -n "${IS_PRERELEASE:-}" ] || fail "IS_PRERELEASE is not set"
case "${IS_PRERELEASE}" in
    true|false) ;;
    *) fail "IS_PRERELEASE must be 'true' or 'false' (got '${IS_PRERELEASE}')" ;;
esac
case "${IMAGE}" in
    *:*) fail "IMAGE '${IMAGE}' must not carry a tag" ;;
esac

names_file=$(mktemp)
tags_file=$(mktemp)
trap 'rm -f "${names_file}" "${tags_file}"' EXIT

# Read from files, never a pipeline: `fail` must be able to exit the script,
# and a pipeline would run the loop body in a subshell where `exit` is a no-op.
printf '%s\n' "${TAGS:-}" > "${tags_file}"

: > "${names_file}"
while IFS= read -r ref || [ -n "${ref}" ]; do
    [ -z "${ref}" ] && continue
    case "${ref}" in
        "${IMAGE}:"*) name="${ref#"${IMAGE}:"}" ;;
        *) fail "tag ref '${ref}' is not under '${IMAGE}:'" ;;
    esac
    case "${name}" in
        ''|*:*|*/*|*[[:space:]]*)
            fail "tag ref '${ref}' does not yield a plain tag name (got '${name}')"
            ;;
    esac
    printf '%s\n' "${name}" >> "${names_file}"
done < "${tags_file}"

if [ ! -s "${names_file}" ]; then
    fail "tag list is empty; docker/metadata-action produced no tags for this ref"
fi

duplicates=$(sort "${names_file}" | uniq -d | tr '\n' ' ')
if [ -n "${duplicates}" ]; then
    fail "duplicate tag name(s): ${duplicates}"
fi

count_name() {
    grep -Fxc -e "$1" "${names_file}" || true
}

require_once() {
    count=$(count_name "$1")
    if [ "${count}" -ne 1 ]; then
        fail "'$1' must appear exactly once ($2); found ${count}"
    fi
}

require_absent() {
    count=$(count_name "$1")
    if [ "${count}" -ne 0 ]; then
        fail "'$1' must not be present ($2); found ${count}"
    fi
}

version="${RELEASE_VERSION}"
case "${version}" in
    *.*.*) ;;
    *) fail "RELEASE_VERSION '${version}' is not X.Y.Z[-<pre>]" ;;
esac
major="${version%%.*}"
rest="${version#*.}"
minor="${rest%%.*}"
major_minor="${major}.${minor}"
case "${major}" in
    ''|*[!0-9]*) fail "RELEASE_VERSION '${version}' does not start with a numeric major" ;;
esac

require_once "${version}" "the full version"
require_once "v${version}" "the v-prefixed full version"

if [ "${IS_PRERELEASE}" = "true" ]; then
    require_absent "latest" "a prerelease never moves :latest"
    kind="prerelease"
else
    require_once "${major_minor}" "the major.minor alias"
    require_once "${major}" "the major alias ({{major}} = 0 is intentional and published)"
    require_once "latest" "a stable release moves :latest"
    kind="stable"
fi

while IFS= read -r name; do
    [ -z "${name}" ] && continue
    case "${name}" in
        "${version}"|"v${version}") continue ;;
    esac
    if [ "${IS_PRERELEASE}" = "true" ]; then
        case "${name}" in
            "${major_minor}"|"${major}") continue ;;
        esac
        fail "tag name '${name}' is not allowed for a prerelease (allowed: '${version}', 'v${version}', '${major_minor}', '${major}')"
    fi
    case "${name}" in
        "${major_minor}"|"${major}"|"latest") continue ;;
    esac
    fail "tag name '${name}' is not in the allowed set {'${version}', 'v${version}', '${major_minor}', '${major}', 'latest'}"
done < "${names_file}"

name_count=$(wc -l < "${names_file}")
name_count=$(printf '%s' "${name_count}" | tr -d ' ')

# Print the resolved names, not just a count: the Phase-4 release run quotes
# this block to record exactly which refs were published.
printf 'guard-image-tags: resolved tag names (%s):\n' "${name_count}"
while IFS= read -r name; do
    [ -n "${name}" ] || continue
    printf '  %s\n' "${name}"
done < "${names_file}"
printf 'guard-image-tags: OK %s names (%s)\n' "${name_count}" "${kind}"
