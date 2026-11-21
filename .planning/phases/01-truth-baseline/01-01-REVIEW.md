---
phase: 01-truth-baseline
plan: 01
reviewed: 2026-11-21T19:24:27Z
depth: deep
files_reviewed: 5
files_reviewed_list:
  - crates/codegen/xai-grok-sandbox/src/runtime_sockets.rs
  - crates/codegen/xai-grok-sandbox/src/runtime_sockets_tests.rs
  - crates/codegen/xai-grok-sandbox/src/lib.rs
  - crates/codegen/xai-grok-sandbox/tests/deny_paths_e2e.rs
  - .planning/phases/01-truth-baseline/01-01-SUMMARY.md
findings:
  critical: 0
  warning: 2
  info: 8
  total: 10
status: issues_found
verdict: PASS_WITH_NOTES
---

# Phase 01 Plan 01: Code Review Report (Sandbox deny materialization)

**Reviewed:** 2026-11-21T19:24:27Z
**Base:** `6d3cfcb9` (all change under review is uncommitted in the working tree)
**Depth:** deep
**Files Reviewed:** 5
**Status:** issues_found (no blockers)

## Summary

The predicate is narrow and the fail-closed arm is real code, not a comment. I traced every arm of
`materialize_runtime_socket_deny_paths_from` (`runtime_sockets.rs:140-214`):

- Happy path unchanged: candidate-parent `canonicalize` → endpoint `lstat` → symlink reject → dedupe push.
- Shape B (new arm, `:168-174`) skips on `canonicalize(parent) == PermissionDenied`. Shape A
  (`:187-202`) routes the endpoint `lstat` error through the pure classifier
  `classify_endpoint_error(kind, parent_is_traversable(parent))` and skips only
  `PermissionDenied && !parent_traversable`. `PermissionDenied` under a traversable parent, and every
  other kind except `NotFound`, return the contextual error (`:199`). `NotFound → Missing` preserves
  the pre-existing silent skip.
- A skip is `continue`, so remaining candidates still materialize and alias dedupe still collapses
  `/var/run` → `/run` (`:209-211`); ordering is preserved. `merge_runtime_socket_denies`
  (`:321-351`) retains an explicit user deny for a skipped static-policy path because the alias branch
  compares against `auto_sockets`, which no longer holds the skipped entry — I walked the
  `.docker/desktop` vs `.docker/run` case and it stays fail-closed.
- Handoff: a skipped candidate never enters `auto_sockets`, therefore never enters the
  `__GROK_BWRAP_RUNTIME_SOCKET_DENY` JSON nor the read-deny arg list. `verify_runtime_socket_deny`
  (`read_deny_verify.rs:433`) is untouched, and the inside-bwrap decode only validates that handed
  paths are members of the static policy, so the skipped entry is simply absent from the inside plan.
- Security of the skip itself: it is sound because the sandboxed child has the *same* uid and no
  capabilities. I verified on this host that bwrap maps uid 1000 → 1000 and `CapEff` is `0`
  (and `bwrap_reexec_command_ex` additionally passes `--cap-drop ALL`, `lib.rs:329`), so a directory
  the outer process cannot traverse is equally unreachable inside the namespace. No new exposure.
- Log content is the candidate path only (`:169-172`, `:193-196`); no env, no user data, no stderr.

Test quality is good: the shape-A fixture really is shape A (I proved with a standalone probe that
`canonicalize` of a user-owned mode-000 directory **succeeds** while `lstat(dir/x)` and
`lstat(dir/.)` both return EACCES), the shape-B fixture is genuinely shape B, and both are red before
the fix because the old code fell through to `Err(error) => return Err(with_context(error))`. The
retargeted `devbox_genuine` scenario keeps both markers and the `/data` check, adds no `#[ignore]`,
and leaves `skip_if_enforcement_unavailable` untouched.

Two quality warnings and eight notes; none blocks.

## Warnings

### WR-01: the new `HOME`-redirecting test is serialized only against `bwrap_env`, while other tests in the same test binary read `home_dir()` outside that key

**File:** `crates/codegen/xai-grok-sandbox/src/lib.rs:1181-1252` (mutation at `:1218`)
**Issue:** `a_non_traversable_socket_parent_does_not_abort_reexec` mutates process-global `HOME`
(`EnvGuard::set("HOME", …)`, `:1218`) under `#[serial(bwrap_env)]`. That key only excludes other
`bwrap_env` tests. Several tests in the same `--lib` binary resolve profiles (or call the materializer)
without that key and therefore read `home_dir()` while the fixture `HOME` is installed:

- `runtime_sockets_tests.rs:469-499` `custom_user_socket_deny_kept_when_restrict_network_false` →
  `assert_no_auto_socket_denies_except` (`:67-79`) → `runtime_socket_deny_paths()`;
- `profiles.rs:464` — `ProfileName::Strict::resolve` derives `home.join("Library")` from `home_dir()`
  on every call;
- `profiles.rs:236` — `read_write_grant_path`'s default `~/.grok/sessions` comparison.

A read that lands between the two sides of one such assertion yields fixture-home-derived paths and a
spurious mismatch (or, worse, a silently weaker assertion). The SUMMARY itself lists this as a residual
risk. Five consecutive full `--lib` runs were green here, so the probability is low, but it is a
suite-flake vector introduced by this change, and the plan's mitigation was scoped to one reader.
**Fix:** either move the upper-layer proof into `deny_paths_e2e` as a subprocess scenario (the fixture
already has process isolation, which is what this test really wants), or introduce one crate-wide
serial key (e.g. `home_env`) and put it on every test that resolves a profile or calls `home_dir()`,
or inject the home directory into the candidate list so the test needs no `HOME` mutation at all.

### WR-02: `parent_is_traversable`'s doc comment misstates what the predicate actually does

**File:** `crates/codegen/xai-grok-sandbox/src/runtime_sockets.rs:129-136`
**Issue:** The comment says `lstat("parent/.")` "fails with `EACCES` exactly when the directory is not
traversable" and "it follows no symlink". The implementation is
`symlink_metadata(parent.join(".")).is_ok()`, so:

1. *any* probe error (ENOENT from a concurrent parent removal, ELOOP, ENOTDIR) is folded into "not
   traversable", not only EACCES;
2. `parent` is an **intermediate** path component, so its symlinks are resolved (`/var/run` →
   `/run`); only a symlink in the final component is not followed, and there is no final-component
   symlink here.

The behaviour is safe (a mis-probe can only turn skip into Fatal, because the endpoint `lstat` already
returned EACCES), but the comment is the specification a future editor will follow — if someone
"fixes" it to match the prose by matching `ErrorKind::PermissionDenied` exactly, a race becomes an
aborted startup. This wording was dictated by the plan (`01-01-PLAN.md:104-107`), so it is a
plan-text defect carried into the code.
**Fix:** state the real contract, e.g. "Returns `false` for any `lstat("parent/.")` error. `parent` is
resolved as an intermediate component (a `/var/run`-style alias is followed); the `.` final component
is not a symlink. A failure means the process cannot enter the directory, which is the only condition
under which an endpoint `EACCES` may be skipped."

## Info

### IN-01: shape B skips ancestor `EACCES` regardless of origin (DAC, ACL, or LSM)

**File:** `crates/codegen/xai-grok-sandbox/src/runtime_sockets.rs:168-174`
**Issue:** Unlike shape A, the `canonicalize(parent)` arm does not ask *why* the ancestor denied
traversal, so an LSM/ACL denial on an ancestor is silently skipped where shape A would keep it fatal
(if the endpoint `lstat` even produced EACCES). The locked decision accepts this shape and the SUMMARY
records it as a risk, and the child inherits the same LSM label/ACL, so it is consistent — noted for
the record, no action.

### IN-02: the skip's safety argument depends on the child having no `CAP_DAC_OVERRIDE`

**File:** `crates/codegen/xai-grok-sandbox/src/runtime_sockets.rs:190-198`, `lib.rs:329`
**Issue:** Skipping is correct only because the sandboxed child cannot reach what the outer process
cannot traverse. I verified the current invocation: bwrap maps uid 1000 → 1000 and `CapEff` = `0`
(and `--cap-drop ALL` is passed). If someone later adds `--uid 0` or `--cap-add DAC_OVERRIDE`, a
user-owned mode-000 parent (e.g. a broken `$HOME/.docker`) would become traversable inside the
namespace while this code has already declined to mask it. Suggest a one-line comment next to either
`--cap-drop ALL` or the skip arm recording that coupling; no code change needed today.

### IN-03: the `canonicalize`-NotFound branch still bypasses the classifier

**File:** `crates/codegen/xai-grok-sandbox/src/runtime_sockets.rs:175-183`
**Issue:** The nested `symlink_metadata(&candidate)` error at `:181` is returned fatal for *every*
kind, including `PermissionDenied`. In steady state that is unreachable (a NotFound parent cannot
yield an EACCES endpoint), but it is a TOCTOU hole through which a permissions change between the two
syscalls aborts startup with exactly the failure SBOX-01 removes. Either route it through
`classify_endpoint_error` for consistency or add a comment saying the fatal arm here is deliberate.

### IN-04: "does not abort bwrap startup" is proven at command construction, not by an exec

**File:** `crates/codegen/xai-grok-sandbox/src/lib.rs:1177-1252`, `tests/deny_paths_e2e.rs:1140-1174`
**Issue:** The upper-layer test asserts `bwrap_reexec_for_profile(...)` is `Some` and that neither the
args nor the handoff reference the non-traversable tree — which is where the abort actually happened
(`bwrap_deny_plan` → `resolve_profile_with_runtime_sockets`). No scenario in `deny_paths_e2e` ever
execs bwrap with a non-traversable automatic-socket parent, and the retargeted `devbox_genuine`
scenario exercises a *different* obligation (explicit deny on a devbox-extending profile with
`restrict_network = false`, so its runtime-socket policy is empty — `profiles.rs:381-392`). This
satisfies the plan as written; an e2e scenario (fixture `HOME`, `chmod 000 <home>/.docker`, genuine
re-exec) would close the last gap cheaply since `dac_enforced` already exists in the test file.

### IN-05: no test asserts the "one debug event per skip" success criterion

**File:** `crates/codegen/xai-grok-sandbox/src/runtime_sockets.rs:169-172`, `:193-196`
**Issue:** The success criterion "both non-traversable shapes skip with one debug event each and no
stderr output" is satisfied by inspection only. Both arms emit exactly one `tracing::debug!` with
`path = %candidate.display()` and there is no `eprintln!`/`println!` in the production path, so the
claim holds; no assertion pins it against a future second log or a level change. Low value to test,
listed for completeness.

### IN-06: SUMMARY fixture/honesty details are slightly off

**File:** `.planning/phases/01-truth-baseline/01-01-SUMMARY.md:121`, `:125`, `:30-36`
**Issue:** Three small inaccuracies, none affecting the result:
1. "The two DAC-dependent fixtures … self-skip" — there are **four** euid-guarded fixtures in
   `runtime_sockets_tests.rs` (`unsearchable_parent_directory_is_skipped`,
   `unsearchable_ancestor_is_skipped`, `unsearchable_sibling_does_not_drop_a_materializable_candidate`,
   `parent_alias_dedupe_survives_a_skipped_candidate`) plus the `lib.rs` test. On a root CI host, five
   of the eight new tests pass without exercising the fix.
2. "no skip condition was added anywhere" (`:125`) is contradicted by those new `dac_enforced` guards;
   the prohibition in SBOX-02 is scoped to `deny_paths_e2e`, which is genuinely clean, so the sentence
   should say so.
3. `key-files.created: []` omits the SUMMARY file itself (untracked, as expected, but the field claims
   nothing was created).

### IN-07: the new upper-layer test inherits a hook-snapshot host dependency

**File:** `crates/codegen/xai-grok-sandbox/src/lib.rs:1180-1224`
**Issue:** `[profiles.hardened] extends = "strict"` makes `requires_hook_write_deny` true
(`lib.rs:54-66`), so `bwrap_deny_plan` calls `prepare_hook_write_deny` and returns `None` on an
unresolvable host hook snapshot — the test then fails at `.expect(...)` instead of self-skipping. The
crate already has `test_util::skip_if_host_hook_write_deny_unresolvable()` (`test_util.rs:7`) for
exactly this, and the existing `bwrap_reexec_for_profile_devbox_extends_composes_data_and_read_deny`
(`lib.rs:1140-1147`) has the same latent exposure, so this is not new fragility — but this test is the
new SBOX-02 proof and would benefit from the guard or a comment. (A devbox-based profile cannot be
used instead: `skip_dbus = true` and `restrict_network = false` would leave the runtime-socket policy
empty and the test would no longer be red-first.)

### IN-08: the devbox genuine scenario hard-requires `PROFILE_ENV`, and the built-in `Devbox` loses its e2e re-exec path

**File:** `crates/codegen/xai-grok-sandbox/tests/deny_paths_e2e.rs:610`, `:215-217`
**Issue:** `subprocess_devbox_genuine` now calls `profile_from_env()`, which panics with a bare
`SANDBOX_E2E_PROFILE` message if a future caller omits the env var. Separately, the named
`ProfileName::Devbox` profile no longer has an end-to-end re-exec test (the retarget uses a
devbox-extending custom); `is_devbox_based` treats both identically and the unit tests cover it, so
coverage is not meaningfully weakened. Both are notes, not defects.

## Verification performed

All commands run in `/home/cein_orourke/Projects/deepseek-build` at base `6d3cfcb9`; heavy cargo
commands under `flock -w 900 /tmp/team-build.lock`. 1002 GB free on `/`.

| # | Command | Result |
|---|---------|--------|
| 1 | `git diff HEAD --stat` / `git status --short` | 4 modified files, **+374/−9**; 1 untracked `01-01-SUMMARY.md`; no manifest/lockfile change; nothing else touched |
| 2 | `git diff HEAD` review | symlink, both `InvalidInput`, dedupe and `skip_if_enforcement_unavailable` arms unchanged; `read_deny_verify.rs` not in the diff |
| 3 | `cargo fmt --all --check` | exit 0 |
| 4 | `cargo test -p xai-grok-sandbox --lib runtime_sockets` | **27 passed**, 0 failed, 0 ignored (matches SUMMARY) |
| 5 | `cargo test -p xai-grok-sandbox --lib read_deny_verify` | **14 passed**, 0 failed, 0 ignored (matches SUMMARY) |
| 6 | `cargo test -p xai-grok-sandbox --lib` × 5 consecutive | **383 passed**, 0 failed, 0 ignored each run (matches SUMMARY; no flake observed, see WR-01) |
| 7 | `cargo test -p xai-grok-sandbox --test deny_paths_e2e` | **13 passed, 0 failed, 1 ignored** (`subprocess_entry`, pre-existing); matches SUMMARY; no new `#[ignore]` |
| 8 | same, `-- --nocapture` | no `skipping:` line — no test took `skip_if_enforcement_unavailable`; the green run is real, not a soft skip |
| 9 | `cargo clippy -p xai-grok-sandbox --all-targets` | exit 0, **0 errors**; only the two pre-existing `clippy.toml` "does not refer to a reachable function" warnings (plus the pre-existing `xai-grok-config` dead-code warning) |
| 10 | standalone Rust probe (`/tmp/opencode/probe`) on the probe semantics | `std::env::home_dir` honours the redirected `HOME`; `canonicalize(dir mode 000)` = **Ok**; `canonicalize(dir/child)` = EACCES; `lstat(dir/x)` = EACCES; `lstat(dir/.)` = EACCES — confirms shape A and shape B are exactly as described and that both new fixtures are red before the fix |
| 11 | `bwrap --bind / / -- cat /proc/self/uid_map`, `grep CapEff` | uid/gid 1000 → 1000, `CapEff: 0` — the skip cannot expose a child-reachable path (see IN-02) |
| 12 | Red-first reasoning + probe cross-check | pre-fix, shape B hits `Err(error) => return Err(...)` and shape A hits `Err(error) => return Err(with_context(error))` → `resolve_profile_with_runtime_sockets` errors → `bwrap_deny_plan` returns `None`; `a_non_traversable_socket_parent_does_not_abort_reexec` would fail at `.expect(...)`. Red-first confirmed by construction and by the probe |

NOT RUN: `cargo test --workspace` (not part of this plan's gate; 01-05 owns it). I touched the mtime
of `src/runtime_sockets.rs` to force a non-cached clippy pass; content is unchanged and `git status`
is identical to the pre-review state.

## Verification honesty (SUMMARY assessment)

- Every claimed cargo count reproduces exactly (`27`, `14`, `383`, `13 passed / 1 ignored`), and the
  clippy/fmt claims are accurate. The baseline `12 passed; 1 failed; 1 ignored` is explicitly labelled
  as supplied by the orchestrator rather than re-run — honest, and consistent with the plan's premise
  (`/data` absent on this host, so `bwrap_deny_plan(Devbox, …)` is empty and `Some(...)` cannot happen).
- The two documented deviations are real and are improvements: the `OnceLock` pinning is genuinely
  needed (the test otherwise walks `HOME` while `grok_home()` is first initialised), and the handoff
  assertion is necessary because runtime-socket denies travel in the env var, not argv.
- The red-first claim is true, but only stated as a comment; the report records no captured failing
  run. I confirmed it independently (row 12), so the claim stands.
- WR-01 (the `HOME` race) is disclosed in SUMMARY Risks; my finding is that the mitigation is narrower
  than the sentence implies.
- IN-06 records the only places where the SUMMARY narrative overstates or miscounts. No fabricated
  numbers.

## Verdict

**PASS_WITH_NOTES** — SBOX-01 and SBOX-02 (test half) are implemented as specified: both
non-traversable shapes skip and nothing else does, the fail-closed row is pinned by a pure table that
the filesystem cannot fake, a skip cannot short-circuit the loop or break alias dedupe, skipped
candidates are absent from the handoff and from the inside-bwrap plan, `verify_runtime_socket_deny` is
untouched, the devbox genuine scenario keeps both markers with no new ignore or skip, and the
production log line carries only the path. There is no blocker and no security regression.

Fix WR-01 (test isolation for the `HOME` redirect) and WR-02 (correct the probe's doc comment) before
merging; IN-03 and IN-07 are cheap consistency hardening in the same files, and the rest are notes.
The uncommitted work is safe to commit once WR-01/WR-02 are addressed.

---

_Reviewed: 2026-11-21T19:24:27Z_
_Reviewer: gsd-code-reviewer (subagent)_
_Depth: deep_
