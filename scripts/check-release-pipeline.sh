#!/bin/sh
# check-release-pipeline.sh — one command that runs every static release gate.
#
# Gates, in order (the first failure exits non-zero):
#   1. actionlint .github/workflows/release.yml
#   2. shellcheck --shell=sh on install.sh and this phase's shell scripts
#   3. python3 scripts/check-release-workflow.py    (job graph + macOS artifact fixture)
#   4. sh scripts/test-guard-image-tags.sh          (the image tag-list guard)
#   5. python3 scripts/check-release-tag-shape.py   (the v0.1.0 tag shape)
#   6. sh scripts/test-install.sh                   (installer against a loopback fake tree)
#
# The expected tallies are pinned on purpose: a checker or fixture that
# silently stopped running assertions fails this gate instead of passing on a
# shorter list. Update a tally in the same change as the checker it counts.
#
# Uses no cargo. The only network is the installer harness's own HTTP server on
# 127.0.0.1. Creates no branch, tag, release or registry write.
#
# Run: sh scripts/check-release-pipeline.sh

set -eu

ROOT=$(cd "$(dirname "$0")/.." && pwd)
cd "$ROOT"

say() { printf '==> %s\n' "$*"; }
die() { printf '!!  %s\n' "$*" >&2; exit 1; }

capture() {
    # capture <label> <command...> — run, keep the combined output in CAP_OUT,
    # and die with the full output if the command fails.
    _label=$1
    shift
    if ! CAP_OUT=$("$@" 2>&1); then
        printf '%s\n' "$CAP_OUT"
        die "$_label failed"
    fi
}

expect() {
    # expect <label> <haystack> <needle>
    case "$2" in
        *"$3"*) ;;
        *)
            printf '%s\n' "$2"
            die "$1: expected '$3' — a checker or fixture that stopped running assertions is a failure, not a pass (update the pinned tally only with the matching checker change)"
            ;;
    esac
}

say "1/6 actionlint: .github/workflows/release.yml"
actionlint .github/workflows/release.yml || die "actionlint failed"

say "2/6 shellcheck: install.sh and the phase's shell scripts"
shellcheck --shell=sh \
    install.sh \
    scripts/test-install.sh \
    scripts/guard-image-tags.sh \
    scripts/test-guard-image-tags.sh \
    scripts/check-release-pipeline.sh || die "shellcheck failed"

say "3/6 release-workflow structure checker (plus the macOS artifact fixture)"
capture "release-workflow checker" python3 scripts/check-release-workflow.py
printf '%s\n' "$CAP_OUT"
expect "release-workflow checker" "$CAP_OUT" "release-workflow: 19/19 assertions PASS"
expect "macOS artifact fixture" "$CAP_OUT" "artifact-step fixture: 5/5 PASS"

say "4/6 image tag-list guard fixture"
capture "image tag-list guard fixture" sh scripts/test-guard-image-tags.sh
printf '%s\n' "$CAP_OUT" | tail -n 1
expect "image tag-list guard fixture" "$CAP_OUT" "20/20 assertions passed, 0 failed"

say "5/6 v0.1.0 tag-shape checker"
capture "tag-shape checker" python3 scripts/check-release-tag-shape.py
printf '%s\n' "$CAP_OUT"
expect "tag-shape checker" "$CAP_OUT" "tag-shape: 17/17 assertions PASS"

say "6/6 installer harness (loopback fake release tree)"
capture "installer harness" sh scripts/test-install.sh
printf '%s\n' "$CAP_OUT" | tail -n 1
expect "installer harness" "$CAP_OUT" "ALL PASS (58/58 assertions)"

say "release-pipeline: all gates PASS"
