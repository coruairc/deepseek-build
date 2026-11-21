---
phase: 01-truth-baseline
plan: "01"
subsystem: infra
tags: [sandbox, bwrap, deny-paths, unix-permissions, eacces, tracing, serial_test]

# Dependency graph
requires:
  - phase: none
    provides: "Mature rc5 lineage; no artifact of 01-04 is used (the dependency only serialised the mandated order)"
provides:
  - "A pure endpoint-error classifier plus a parent-traversability probe in xai-grok-sandbox"
  - "Both non-traversable parent shapes (ancestor EACCES and parent-last-component EACCES) skip instead of aborting"
  - "A fixture matrix that pins every fatal arm, including the one the filesystem cannot produce"
  - "A devbox genuine re-exec scenario with a real deny obligation on hosts without /data"
  - "An upper-layer proof that bwrap_reexec_for_profile survives a non-traversable automatic-socket parent"
affects: [01-truth-baseline plan 01-05 (test gate), Phase 2 proof]

actuals:
  tokens: 5263        # chars/4 over `git diff` (21,050 chars, 374 insertions / 9 deletions)
  tasks: 3
  commits: 0          # intentional: the dispatch forbids committing; changes left in the working tree

tech-stack:
  added: []
  patterns:
    - "Pure decision function + impure probe: classify_endpoint_error(kind, parent_traversable) makes a fail-closed arm testable by construction"
    - "lstat(parent/.) as a no-dependency traversal probe, consulted only on the error path"

key-files:
  created: []
  modified:
    - crates/codegen/xai-grok-sandbox/src/runtime_sockets.rs
    - crates/codegen/xai-grok-sandbox/src/runtime_sockets_tests.rs
    - crates/codegen/xai-grok-sandbox/src/lib.rs
    - crates/codegen/xai-grok-sandbox/tests/deny_paths_e2e.rs

key-decisions:
  - "Shape B skips on canonicalize(parent) == PermissionDenied; shape A skips only when the endpoint lstat is PermissionDenied AND lstat(parent/.) fails. Everything else stays Fatal."
  - "NotFound keeps its original, log-free skip; only the two non-traversable arms emit one tracing::debug! (path only)."
  - "The devbox genuine scenario is retargeted onto [profiles.genuine] extends = \"devbox\" with an explicit deny, so the plan is non-empty on a host with no /data; skip_if_enforcement_unavailable is untouched."
  - "Home-derived process-wide caches (grok home, daemon tmp dirs, daemon writable locations) are pinned before the lib.rs test redirects HOME, so no concurrent first use caches a path inside the fixture."

patterns-established:
  - "Fail-closed-by-construction: any error arm the filesystem cannot build is expressed as a pure function and pinned by a table test."
  - "Skip logging is one debug event with the candidate path; no stderr write from library code."

requirements-completed: [SBOX-01, SBOX-02]

coverage:
  - id: D1
    description: "Runtime-socket deny materialization skips a candidate whose parent is not traversable, in both shapes, and stays fatal otherwise"
    requirement: SBOX-01
    verification:
      - kind: unit
        ref: "crates/codegen/xai-grok-sandbox/src/runtime_sockets_tests.rs#endpoint_error_classification_table + the six-fixture matrix"
        status: pass
      - kind: integration
        ref: "cargo test -p xai-grok-sandbox --lib (383 passed; 0 failed; 0 ignored)"
        status: pass
    human_judgment: false
  - id: D2
    description: "A root-only / non-traversable deny-path parent no longer aborts bwrap startup; deny_paths_e2e is green unmodified with no new ignores or skip conditions"
    requirement: SBOX-02
    verification:
      - kind: integration
        ref: "cargo test -p xai-grok-sandbox --test deny_paths_e2e (13 passed; 0 failed; 1 ignored)"
        status: pass
      - kind: unit
        ref: "crates/codegen/xai-grok-sandbox/src/lib.rs#a_non_traversable_socket_parent_does_not_abort_reexec"
        status: pass
    human_judgment: false

duration: ~40min (approximate; two harness restarts during the run)
completed: 2026-11-21
status: complete
---

# Phase 01 Plan 01: Sandbox runtime-socket deny traversal fix Summary

**Runtime-socket deny materialization now skips candidates under non-traversable parents in both shapes — ancestor EACCES and a last-component EACCES — while every other resolution error stays fatal, proven by a six-fixture matrix, a pure classification table, and an unmodified green `deny_paths_e2e`.**

## Performance

- **Duration:** ~40 min (approximate; the harness restarted twice, so wall-clock is reconstructed)
- **Started:** ~2026-11-21T18:45Z (approx)
- **Completed:** 2026-11-21T19:15Z
- **Tasks:** 3/3
- **Files modified:** 4 (all inside `files_modified`)
- **Base HEAD:** `6d3cfcb9d57024662f8947dcf8478a882d9cc9e7` (`feat(01-04): make deepseek-flash the catalog default`)
- **Commits:** none — the dispatch forbids committing; everything is left in the working tree for review

## Accomplishments

- `materialize_runtime_socket_deny_paths_from` no longer returns an error when a runtime-socket candidate's parent is unsearchable, which is what aborted `bwrap_reexec_for_profile` startup.
- Added the classifier the plan specified:

  ```rust
  pub(crate) enum EndpointError { Missing, UntraversableParent, Fatal }
  pub(crate) fn classify_endpoint_error(kind: io::ErrorKind, parent_traversable: bool) -> EndpointError
  pub(crate) fn parent_is_traversable(parent: &Path) -> bool   // lstat(parent/".").is_ok()
  ```

- Shape B (an ancestor is unsearchable) is a new `PermissionDenied` arm on `dunce::canonicalize(parent)`; shape A (the parent itself is the unsearchable last component) is the endpoint `lstat` error classified with the traversal probe. `PermissionDenied` under a traversable parent remains `Fatal` (LSM denial), and the symlink / `InvalidInput` / dedupe arms are byte-identical.
- `devbox_genuine_reexec_applies_enforcement` now exercises a `[profiles.genuine] extends = "devbox"` profile with an explicit `deny = ["secret.pem"]`, giving the genuine re-exec a non-empty plan on a host with no `/data`; both marker strings and the `/data` readability check are unchanged.
- Added an upper-layer unit test proving `bwrap_reexec_for_profile(Custom("hardened"), ws)` is `Some` while `<HOME>/.docker` is mode 000, and that the skipped candidate is absent from the runtime-socket handoff.

## Fixture matrix (unit, `runtime_sockets_tests.rs`)

| Test | Shape | Expected | Result |
|---|---|---|---|
| `unsearchable_parent_directory_is_skipped` | A: `p/` mode 000, live socket inside, `canonicalize(p)` succeeds, endpoint `lstat` EACCES | `Ok`, candidate absent | PASS (ran; euid 1000) |
| `unsearchable_ancestor_is_skipped` | B: `hidden/` mode 000, `canonicalize(hidden/p)` EACCES | `Ok`, candidate absent | PASS |
| `missing_parent_is_skipped` | parent NotFound, no endpoint | `Ok`, empty | PASS |
| `unreadable_endpoint_under_traversable_parent_is_materialized` | 0755 parent, endpoint mode 000 | path returned | PASS |
| `symlink_endpoint_is_fatal` | endpoint symlink | error contains `endpoint is a symlink` | PASS |
| `unsearchable_sibling_does_not_drop_a_materializable_candidate` | mode-000 parent + 0755 parent in one call | `Ok`, exactly the 0755 candidate | PASS |
| `parent_alias_dedupe_survives_a_skipped_candidate` | `run`/`var` symlink alias + skipped shape-A candidate | one canonical path | PASS |
| `endpoint_error_classification_table` | `(NotFound,true/false)`, `(PermissionDenied,false)`, `(PermissionDenied,true)`, `(InvalidInput,true)` | Missing / Missing / UntraversableParent / Fatal / Fatal | PASS |

The two DAC-dependent fixtures and the two filesystem fixtures above self-skip with a printed reason when `libc::geteuid() == 0`; on this host (euid 1000) they executed and passed. `read_deny_verify`'s suite was run without touching its source: **14 passed; 0 failed; 0 ignored**.

Other fixtures present in the file and still green: `materialized_socket_path_is_stable_after_mode_zero_mask`, `materialized_socket_paths_reject_endpoint_symlinks`, `materialized_socket_paths_propagate_parent_resolution_errors`, the alias/handoff/dedupe tests, and the profile-level socket-deny tests.

`#[serial(bwrap_env)]` was added on the call site of `deny_list_covers_system_rootless_and_per_user_endpoints` (it reads `home_dir()` twice around `runtime_socket_deny_paths()`); no skip condition was added anywhere.

## Verification (commands and actual results, this host, 2026-11-21)

```
scripts/disk-guard.sh 150
  disk-guard: 1001 GB free on the target filesystem (minimum 150 GB) -> rc=0

flock -w 900 /tmp/team-build.lock cargo test -p xai-grok-sandbox --lib runtime_sockets
  test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 356 filtered out

flock -w 900 /tmp/team-build.lock cargo test -p xai-grok-sandbox --lib read_deny_verify
  test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 369 filtered out

flock -w 900 /tmp/team-build.lock cargo test -p xai-grok-sandbox --lib a_non_traversable_socket_parent_does_not_abort_reexec
  test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 382 filtered out

flock -w 900 /tmp/team-build.lock cargo test -p xai-grok-sandbox --test deny_paths_e2e
  test result: ok. 13 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out

flock -w 900 /tmp/team-build.lock cargo test -p xai-grok-sandbox --lib
  test result: ok. 383 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

cargo fmt --all --check            -> rc=0
flock -w 900 /tmp/team-build.lock cargo clippy -p xai-grok-sandbox --all-targets
  Finished `dev` profile ... 0 errors; only the pre-existing warnings
  (xai-grok-config dead_code; two clippy.toml disallowed-method entries)
```

Baseline (supplied by the orchestrator, not re-run by me on the pre-change tree): `12 passed; 1 failed; 1 ignored`, the failure being `devbox_genuine_reexec_applies_enforcement` with `FAIL: bwrap_reexec_for_profile returned None outside bwrap`. Post-change: `13 passed; 0 failed; 1 ignored` — the single ignored test is the pre-existing `subprocess_entry` harness entry point, and no `#[ignore]` was added.

Diff review: the symlink arm, both `InvalidInput` arms (no parent / no file name), the dedupe arm and `skip_if_enforcement_unavailable` are unchanged; `git diff` is confined to the four listed files.

## Decisions Made

- **Shape A requires the probe, shape B does not.** `realpath(parent)` succeeds when the parent itself is the unsearchable directory, so skipping only on `canonicalize` EACCES would miss the `/run/containerd`-shaped case. The endpoint error therefore consults `parent_is_traversable`, while the `canonicalize` `PermissionDenied` arm skips directly (an ancestor was unsearchable).
- **The probe is an error-path-only syscall.** `parent_is_traversable` is called inside the endpoint `Err` arm, so the happy path takes no extra `lstat`.
- **Keep `NotFound` silent.** Only the two non-traversable arms log, one `tracing::debug!` per skip with the candidate path and nothing else.
- **Retarget, do not guard.** Giving the devbox genuine scenario an explicit deny is a stronger fix than skipping it when the plan is empty: the re-exec path is exercised everywhere with the same assertions.

## Deviations from Plan

### Auto-fixed / plan-consistent adjustments

**1. [Rule 2 - Missing critical safeguard] Pin home-derived process-wide caches before redirecting `HOME`**
- **Found during:** Task 3 (`a_non_traversable_socket_parent_does_not_abort_reexec`)
- **Issue:** `xai_dirs::grok_home()`, `command::policy::daemon_tmp_dirs()` and `WritableLocations::daemon()` are `OnceLock`-cached and derive from `HOME`. A first use during the redirected-`HOME` window would cache a path inside the fixture the test deletes.
- **Fix:** The test calls all three before `EnvGuard::set("HOME", …)`, pinning the real values.
- **Files modified:** `crates/codegen/xai-grok-sandbox/src/lib.rs`
- **Verification:** full lib run green (383 passed).

**2. [Plan detail] The handoff assertion was added alongside the args assertion**
- **Found during:** Task 3
- **Issue:** runtime-socket denies travel in `__GROK_BWRAP_RUNTIME_SOCKET_DENY` (env), not in argv, so the plan's "args contain no path under `.docker`" check is vacuous on its own.
- **Fix:** Kept the args assertion and added a parse of the handoff env asserting no denied path starts with the non-traversable `.docker` directory.
- **Files modified:** `crates/codegen/xai-grok-sandbox/src/lib.rs`
- **Verification:** the test passes post-fix; it returns `None` pre-fix because resolution fails and the plan must refuse.

**3. [Style] `cargo fmt --all` reformatted the new code**
- No semantic change; `cargo fmt --all --check` is clean.

No dependency changes, no new egress, no `#[ignore]`, no new condition in `skip_if_enforcement_unavailable`.

## Issues Encountered

- The harness restarted twice mid-run; the interrupted shell command was a `cargo fmt --all && cargo fmt --all --check`, re-run and confirmed `rc=0`. No partial edits were left behind.
- `deny_paths_e2e` legitimately runs in ~0.03 s here (bwrap re-exec of the test binary is fast); the `devbox_genuine` test was re-run alone with `--nocapture` to confirm it did not soft-skip — no `skipping:` line was printed and the `OK: devbox enforcement applied inside genuine bwrap` marker is asserted by the parent.

## Risks

- **`HOME` mutation is process-global.** The new lib.rs test mutates `HOME` under `#[serial(bwrap_env)]`; tests outside that key that read `xai_dirs::home_dir()` concurrently could observe the temp home. The three cached globals are pinned, and no other test in this crate mutates `HOME` process-wide, but this is the residual parallel-test risk.
- **Shape B assumes `canonicalize` EACCES means non-traversable.** An LSM denial on an ancestor would also be skipped; the locked decision accepts this for the `canonicalize` arm (the endpoint arm keeps the explicit traversal probe).
- **The retargeted genuine scenario depends on `prepare_hook_write_deny` succeeding** for a strict-based custom profile (the hook plan is required for non-devbox profiles). It passed here; on a host with a broken hook-source configuration `bwrap_deny_plan` would still refuse for an unrelated, correctly-fatal reason.
- **No commit was made**, per the dispatch; the working tree carries all four files for review.

## Self-Check: PASSED

- `crates/codegen/xai-grok-sandbox/src/runtime_sockets.rs` — modified (classifier + both arms).
- `crates/codegen/xai-grok-sandbox/src/runtime_sockets_tests.rs` — modified (eight tests + `dac_enforced`).
- `crates/codegen/xai-grok-sandbox/src/lib.rs` — modified (test-module helper + upper-layer test).
- `crates/codegen/xai-grok-sandbox/tests/deny_paths_e2e.rs` — modified (retarget + profile plumbing).
- Commit check: intentionally N/A — committing was forbidden; `git status --short` shows exactly the four `M` entries above.


## Review follow-up (orchestrator, 2026-11-21)

- **WR-01 resolved:** the upper-layer proof moved out of the in-process lib test (which mutated `HOME`)
  into the `deny_paths_e2e` subprocess as scenario `socket_parent_unsearchable` / test
  `unsearchable_socket_parent_does_not_abort_reexec`. The subprocess already receives a fixture `HOME`,
  so no process-global env is mutated by the test runner. The lib-level test was removed; its handoff
  assertion is preserved inside the subprocess against the literal
  `__GROK_BWRAP_RUNTIME_SOCKET_DENY` env.
- **WR-02 resolved:** `parent_is_traversable`'s doc now states `is_ok()` semantics and that intermediate
  symlinks are resolved by the kernel.
- Re-run: `cargo test -p xai-grok-sandbox --test deny_paths_e2e` → **14 passed; 0 failed; 1 ignored**.
