#!/usr/bin/env bash
#
# test-gate.sh — the phase test gate: run the whole workspace plus the
# `config-docs` feature step and prove the run was complete.
#
# What it runs, in order:
#   1. scripts/disk-guard.sh 150 (the tree's floor; stop below it)
#   2. the whole run inside flock -w 900 /tmp/team-build.lock
#   3. step 1: cargo test --workspace --no-fail-fast -- --test-threads=4
#   4. step 2: cargo test -p xai-grok-shell --lib --features config-docs -j 1
#      --workspace never enables `config-docs`, so step 2 is a named step.
#
# What it asserts:
#   - both cargo steps exit 0 (their status is captured, never inferred);
#   - results (top-level `test result:` lines, one per cargo target block)
#     equals binaries (unique test executables from the build graph) + doctests
#     (count of `   Doc-tests ` lines) — a self-consistent completeness proof;
#   - in full mode, results also equals the checked-in count in
#     scripts/test-gate.expected. Only `--refresh-expected`, on a fully green
#     run, rewrites that file; a red run never writes it.
#   - (added with the evidence convention) the leak scan over docs/evidence.
#   The tail of the log is never the verdict.
# Flags:
#   --package <crate>    run only the named package(s) (repeatable). Prints
#                        GATE MODE: SUBSET — not the phase gate and skips the
#                        expected-count comparison.
#   --refresh-expected   re-run and, only when both steps are green and the
#                        derived equality holds, rewrite scripts/test-gate.expected
#                        with commit/tree-state/date/count provenance.
#   --log-dir <dir>      keep the two cargo logs in <dir> (default: a mktemp
#                        dir, removed only when the run is green; a red run
#                        keeps it and prints the path).
#
# The three API-key variables are unset for every cargo step, so an exported
# developer key can neither change results nor reach a log.
#
# This is a bash script (see the shebang); pipefail is intentional. The repo
# also lints scripts with `shellcheck --shell=sh` for consistency, so declare
# that here.
# shellcheck disable=SC3040
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
EXPECTED_FILE="$root/scripts/test-gate.expected"
LOCK_FILE="/tmp/team-build.lock"
LOCK_WAIT=900

usage() {
    cat <<'EOF'
Usage: scripts/test-gate.sh [--package <crate>]... [--refresh-expected] [--log-dir <dir>]
EOF
}

# 1. Refuse to start when the target filesystem is below the tree's floor.
if ! "$root/scripts/disk-guard.sh" 150; then
    echo "gate: refusing to run — disk guard failed" >&2
    exit 1
fi

# 2. Serialise the whole run on the team build lock by re-exec'ing once, so the
# exit code is this gate's, not flock's. This happens before argument parsing
# so "$@" reaches the locked re-exec intact.
if [ "${TEST_GATE_LOCKED:-}" != "1" ]; then
    TEST_GATE_LOCKED=1 exec flock -w "$LOCK_WAIT" "$LOCK_FILE" "$0" "$@"
fi

PACKAGE_ARGS=""
PACKAGES_SHOWN=""
SUBSET=0
REFRESH=0
LOG_DIR=""

while [ $# -gt 0 ]; do
    case "$1" in
        --package)
            if [ $# -lt 2 ]; then
                echo "gate: --package requires a crate name" >&2
                exit 2
            fi
            case "$2" in
                '' | *[!A-Za-z0-9_-]*)
                    echo "gate: invalid crate name: $2" >&2
                    exit 2
                    ;;
            esac
            PACKAGE_ARGS="$PACKAGE_ARGS -p $2"
            PACKAGES_SHOWN="${PACKAGES_SHOWN}${PACKAGES_SHOWN:+ }$2"
            SUBSET=1
            shift 2
            ;;
        --refresh-expected)
            REFRESH=1
            shift
            ;;
        --log-dir)
            if [ $# -lt 2 ]; then
                echo "gate: --log-dir requires a directory" >&2
                exit 2
            fi
            LOG_DIR="$2"
            shift 2
            ;;
        -h | --help)
            usage
            exit 0
            ;;
        *)
            echo "gate: unknown argument: $1" >&2
            usage >&2
            exit 2
            ;;
    esac
done

if [ "$SUBSET" -eq 1 ] && [ "$REFRESH" -eq 1 ]; then
    echo "gate: --refresh-expected needs the full workspace run, not a subset" >&2
    exit 2
fi

if [ "$SUBSET" -eq 1 ]; then
    SELECTION="$PACKAGE_ARGS"
    echo "GATE MODE: SUBSET — not the phase gate (packages: $PACKAGES_SHOWN)"
else
    SELECTION="--workspace"
    echo "GATE MODE: FULL"
fi

# With no --log-dir, the logs live in a temp dir. A green run removes it; a
# red run keeps it and prints the path, because the log is the gate's own
# failure evidence and a reader must be able to open it after the exit.
if [ -n "$LOG_DIR" ]; then
    mkdir -p "$LOG_DIR"
else
    LOG_DIR=$(mktemp -d)
    # $status is assigned by the trap itself, at exit time.
    # shellcheck disable=SC2154
    trap 'status=$?; if [ "$status" -eq 0 ]; then rm -rf "$LOG_DIR"; else echo "gate: failing run kept its logs at $LOG_DIR" >&2; fi' EXIT
fi
LOG_STEP1="$LOG_DIR/step1-tests.log"
LOG_STEP2="$LOG_DIR/step2-config-docs.log"

fail=0

# 3. Step 1: the workspace run. Capture the status; never pipe through tee.
echo "gate step 1: cargo test $SELECTION --no-fail-fast -- --test-threads=4"
set +e
# shellcheck disable=SC2086
env -u DEEPSEEK_API_KEY -u DEEPSEEK_BUILD_API_KEY -u XAI_API_KEY \
    cargo test $SELECTION --no-fail-fast -- --test-threads=4 \
    >"$LOG_STEP1" 2>&1
rc1=$?
set -e
if [ "$rc1" -eq 0 ]; then
    echo "gate step 1: PASS (rc=0, log $LOG_STEP1)"
else
    echo "gate step 1: FAIL (rc=$rc1, log $LOG_STEP1)"
    fail=1
fi

# 4. Step 2: the config-docs feature step.
echo "gate step 2: cargo test -p xai-grok-shell --lib --features config-docs -j 1"
set +e
env -u DEEPSEEK_API_KEY -u DEEPSEEK_BUILD_API_KEY -u XAI_API_KEY \
    cargo test -p xai-grok-shell --lib --features config-docs -j 1 \
    >"$LOG_STEP2" 2>&1
rc2=$?
set -e
if [ "$rc2" -eq 0 ]; then
    echo "gate step 2: PASS (rc=0, log $LOG_STEP2)"
else
    echo "gate step 2: FAIL (rc=$rc2, log $LOG_STEP2)"
    fail=1
fi

# 5. Count, from step 1's log and from the build graph. Only test-result lines
# from the first log count; step 2's log is a separate named step.
#
# `results` counts one top-level result per cargo target block (a block starts
# at `     Running ` or `   Doc-tests `). A test that re-runs its own test
# binary with inherited stdout emits a nested `test result:` line inside the
# block (today: xai_tty_utils's stderr_redirect_roundtrip_subprocess); that line
# is not a target, so it is excluded. `raw_results` is printed for transparency.
raw_results=$(awk '/^test result:/ { n++ } END { print n + 0 }' "$LOG_STEP1")
results=$(
    awk '
        /^     Running / || /^   Doc-tests / {
            if (open) { total++ }
            open = 1
            got = 0
            next
        }
        /^test result:/ { got = 1 }
        END {
            if (open && got) { total++ }
            print total + 0
        }
    ' "$LOG_STEP1"
)
doctests=$(awk '/^   Doc-tests / { n++ } END { print n + 0 }' "$LOG_STEP1")

set +e
# shellcheck disable=SC2086
binaries=$(
    env -u DEEPSEEK_API_KEY -u DEEPSEEK_BUILD_API_KEY -u XAI_API_KEY \
        cargo test $SELECTION --no-run --message-format=json 2>/dev/null \
        | jq -r 'select(.reason=="compiler-artifact" and .profile.test==true and .executable!=null) | .executable' \
        | sort -u \
        | wc -l \
        | tr -d '[:space:]'
)
enum_rc=$?
set -e
if [ "$enum_rc" -ne 0 ]; then
    echo "gate: build-graph enumeration failed (rc=$enum_rc)" >&2
    fail=1
fi

is_uint() {
    case "$1" in
        '' | *[!0-9]*) return 1 ;;
        *) return 0 ;;
    esac
}
for value in "$results" "$doctests" "$binaries"; do
    if ! is_uint "$value"; then
        echo "gate: count is not a number: '$value'" >&2
        fail=1
    fi
done

echo "gate counts: results=$results (raw test-result lines: $raw_results) binaries=$binaries doctests=$doctests"

derived=$((binaries + doctests))
if [ "$results" -eq "$derived" ]; then
    echo "check results == binaries + doctests: PASS ($results == $derived)"
else
    echo "check results == binaries + doctests: FAIL ($results != $derived)"
    fail=1
fi

# 6. Full mode compares against the checked-in expected count. Refresh mode
# skips the comparison: that is the one operation allowed to move the number.
if [ "$SUBSET" -eq 1 ]; then
    echo "check results == expected: SKIPPED (subset mode)"
elif [ "$REFRESH" -eq 1 ]; then
    echo "check results == expected: SKIPPED (refresh mode)"
elif [ ! -f "$EXPECTED_FILE" ]; then
    echo "check results == expected: FAIL ($EXPECTED_FILE is missing)"
    echo "gate: seed it from a fully green run with: scripts/test-gate.sh --refresh-expected" >&2
    fail=1
else
    expected=$(awk -F= '$1 == "results" { gsub(/[^0-9]/, "", $2); print $2 }' "$EXPECTED_FILE" | tail -n 1)
    if ! is_uint "$expected"; then
        echo "check results == expected: FAIL ($EXPECTED_FILE has no results= line)"
        fail=1
    elif [ "$results" -eq "$expected" ]; then
        echo "check results == expected ($expected): PASS"
    else
        echo "check results == expected ($expected): FAIL (got $results)"
        echo "gate: a count change needs a new green run and an explicit --refresh-expected" >&2
        fail=1
    fi
fi

# 7. Write the expected file only from a fully green run with the derived
# equality intact. A red run never writes it.
if [ "$REFRESH" -eq 1 ]; then
    if [ "$fail" -ne 0 ]; then
        echo "gate: --refresh-expected refused — the run is not green" >&2
        exit 1
    fi
    # Stamp the working-tree state too: HEAD alone can predate uncommitted
    # changes the count was measured on (a dirty-tree seed is not reproducible
    # from the stamped commit alone). Compute the state BEFORE the group
    # redirection opens: the redirect truncates $EXPECTED_FILE first, so a
    # check inside the group would always see the file it is writing and
    # could never report "clean".
    tree_state="clean"
    if [ -n "$(git -C "$root" status --porcelain)" ]; then
        tree_state="dirty (uncommitted changes present at refresh time)"
    fi
    {
        echo "# refreshed by: scripts/test-gate.sh --refresh-expected"
        echo "# commit: $(git -C "$root" rev-parse HEAD)"
        echo "# tree: $tree_state"
        echo "# date: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
        echo "# a green full run: results == binaries + doctests"
        echo "results=$results"
        echo "binaries=$binaries"
        echo "doctests=$doctests"
    } >"$EXPECTED_FILE"
    echo "gate: wrote $EXPECTED_FILE"
fi

# 8. Leak scan over the committed evidence (no network). Runs after the count
# assertions and is part of the exit code.
if [ -d "$root/docs/evidence" ]; then
    echo "gate step 3: scripts/key-leak-check.sh --scan docs/evidence"
    set +e
    (cd "$root" && "$root/scripts/key-leak-check.sh" --scan docs/evidence) \
        >"$LOG_DIR/step3-key-leak.log" 2>&1
    rc3=$?
    set -e
    if [ "$rc3" -eq 0 ]; then
        echo "gate step 3: PASS (rc=0, log $LOG_DIR/step3-key-leak.log)"
    else
        echo "gate step 3: FAIL (rc=$rc3, log $LOG_DIR/step3-key-leak.log)"
        fail=1
    fi
else
    echo "gate step 3: SKIPPED (docs/evidence is missing)"
fi

if [ "$fail" -ne 0 ]; then
    echo "GATE FAIL"
    exit 1
fi
# The final verdict is mode-aware: a subset run must not end with the same
# bare "GATE PASS" as the full phase gate (the tail is never the verdict,
# but readers and automation key on the last line).
if [ "$SUBSET" -eq 1 ]; then
    echo "GATE PASS (SUBSET — not the phase gate)"
else
    echo "GATE PASS"
fi
