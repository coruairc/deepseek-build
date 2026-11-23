---
phase: 01-truth-baseline
plan: 06
subsystem: testing
tags: [determinism, flake, sigkill, process-death, evidence, asciinema-v2, repetition]

requires:
  - phase: 01-truth-baseline (plans 01-01..01-05)
    provides: the counted test gate, the evidence convention (evidence.py + docs/evidence/), and the falsified TEST-03 claim this plan closes
provides:
  - deadline-bounded process-death poll (`wait_until_dead`, 5 s / 20 ms) on all five `restore_fetch_tests.rs` death assertions, with a Linux-only bounds test
  - a fourth Phase-1 evidence item (docs/evidence/01-truth-baseline/determinism/) built on a real 35-run post-fix repetition
  - corrected KNOWN-ISSUES.md, TESTING.md and 01-05-SUMMARY.md (2-in-35 falsification, fix, artifact path)
affects: [01-truth-baseline re-verification, 02-live-session-proof, any phase quoting workspace test counts]

actuals:
  tokens: 63210   # chars/4 over this plan's realized diff (tracked edits + untracked evidence item)
  tasks: 3
  commits: 0      # intentional: plan forbids commits; the phase reviewer gates the merge
plan_head_before: 98c2f1b8
plan_head_after: 98c2f1b8

tech-stack:
  added: []        # stdlib Python + system awk/script/flock only; no install, no manifest change
  patterns:
    - "eventual property -> deadline-bounded poll, not an instantaneous /proc snapshot"
    - "repetition evidence: per-run raw cargo log (uncommitted) -> one headerless projection line -> committed projection log -> awk count gates"
    - "script(1) -T capture of the same execution -> evidence.py cast -> asciinema v2; the cast is not a re-run"

key-files:
  created:
    - docs/evidence/01-truth-baseline/determinism/determinism.cast
    - docs/evidence/01-truth-baseline/determinism/report.json
    - docs/evidence/01-truth-baseline/determinism/notes.md
    - docs/evidence/01-truth-baseline/determinism/repetition-35-runs.log
    - docs/evidence/01-truth-baseline/determinism/targeted-repetition-20-named-test-runs.log
    - docs/evidence/01-truth-baseline/determinism/post-fix-repetition-35-full-crate-runs.log
    - docs/evidence/01-truth-baseline/determinism/ (4 projection step logs + leak-scan step log)
  modified:
    - crates/codegen/xai-grok-workspace/src/restore_fetch_tests.rs
    - KNOWN-ISSUES.md
    - TESTING.md
    - .planning/phases/01-truth-baseline/01-05-SUMMARY.md
    - docs/evidence/README.md

key-decisions:
  - "Poll the death instead of asserting it at an instant: the failure was an async-SIGKILL observation race, not a bug in teardown (restore_fetch.rs untouched)"
  - "5 s deadline / 20 ms interval: generous enough for reaping, short enough that a never-dying process still fails well inside the suite's 5 s prompt-return bound"
  - "Same correction on all five death sites, because the sibling tests share the identical race even though only the TERM-immune grandchild rotated"
  - "Linux-only bounds test (u32::MAX is not a pid on the kill(0) path), with a 100 ms lower band so the determinism test is not itself timing-fragile"
  - "A repetition red is committed red; no assertion, band or log was edited to force green"

patterns-established:
  - "Assert the eventual property with a bounded poll; keep instantaneous upper bounds evaluated before any polling"
  - "Proof-of-repetition is a committed per-run projection log with fail-closed awk counts, cross-checked against the cast of the same execution"

requirements-completed: [TEST-03]

coverage:
  - id: D1
    description: "All five process-death assertions in restore_fetch_tests.rs observe death through a deadline-bounded poll (5 s / 20 ms), and the poll's bound is unit-pinned on Linux"
    requirement: TEST-03
    verification:
      - kind: unit
        ref: "20x cargo test -p xai-grok-workspace --lib -- --exact restore_fetch::tests::fetch_timeout_sigkills_term_immune_grandchild — 20/20 ok, each 1 passed / 0 failed"
        status: pass
      - kind: unit
        ref: "cargo test -p xai-grok-workspace --lib -- restore_fetch — 31 passed / 0 failed incl. wait_until_dead_is_bounded_and_deadline_respecting"
        status: pass
      - kind: integration
        ref: "5x cargo test -p xai-grok-workspace --lib -- --test-threads=4 — each 1695 passed / 0 failed"
        status: pass
    human_judgment: false
  - id: D2
    description: "A real post-fix repetition of 35 full-crate runs is green and committed as the determinism evidence item (cast + 8-key report + projection log + notes)"
    requirement: TEST-03
    verification:
      - kind: integration
        ref: "docs/evidence/01-truth-baseline/determinism/repetition-35-runs.log — 35 headerless lines, every one `rc=0 named=ok` with `test result: ok. 1695 passed; 0 failed`, one distinct passed count"
        status: pass
      - kind: other
        ref: "docs/evidence/01-truth-baseline/determinism/report.json — 8 keys, requirements [TEST-03], 7/7 steps ok, result ok; 4 artifact sha256 match; deterministic.cast valid asciinema v2 (3 events) and carries all 35 projection lines"
        status: pass
      - kind: other
        ref: "scripts/key-leak-check.sh --scan docs/evidence — PASS exit 0 over the finished item; no /home path in the item"
        status: pass
    human_judgment: false
  - id: D3
    description: "KNOWN-ISSUES.md, TESTING.md and 01-05-SUMMARY.md record the 2-in-35 falsification, the fix and the evidence path; no artifact-free determinism claim remains"
    requirement: TEST-03
    verification:
      - kind: other
        ref: "grep: all three cite docs/evidence/01-truth-baseline/determinism (01-05-SUMMARY adds the 'Determinism correction' section and marks D3's five-run verification fail); KNOWN-ISSUES and TESTING name 1308903a and the 2-in-35 failure"
        status: pass
    human_judgment: false

duration: ~83min
completed: 2026-11-23
status: complete
---

# Phase 1 Plan 06: TEST-03 gap closure (deterministic death assertions + 35-run evidence) Summary

**A deadline-bounded death poll (5 s / 20 ms) replaced the instant `/proc` check at all five process-death sites, and a real 35/35 post-fix repetition is committed as the fourth Phase-1 evidence item**

## Performance

- **Duration:** ~83 min (first edit 18:23Z → summary 19:46Z; 30m31s of it the 35-run repetition)
- **Started:** 2026-11-23T18:23:00Z
- **Completed:** 2026-11-23T19:46:43Z
- **Tasks:** 3/3
- **Files modified:** 5 tracked (one Rust test file, three docs, the evidence README) + the new `docs/evidence/01-truth-baseline/determinism/` item (11 files)

## Accomplishments

- `restore_fetch::tests::fetch_timeout_sigkills_term_immune_grandchild` — the only test that rotated (2 failures in the verification's 35 runs, `restore_fetch_tests.rs:731`) — now passes 20 consecutive exact runs and 35 consecutive full-crate runs.
- `wait_until_dead(pid, deadline)` (`PROCESS_DEATH_DEADLINE = 5 s`, `PROCESS_DEATH_POLL_INTERVAL = 20 ms`) is wired at all five process-death assertion sites; every test's prompt-return upper bound is still evaluated **before** any polling, no assertion direction changed, and `restore_fetch.rs` is byte-identical to HEAD.
- A Linux-only unit test (`wait_until_dead_is_bounded_and_deadline_respecting`) pins the helper: absent pid → immediate `true`; own live pid → `false` after ≥ 100 ms of a 150 ms deadline, returning before 2 s.
- `docs/evidence/01-truth-baseline/determinism/` records the real post-fix repetition: `determinism.cast` (v2, 3 events, the same execution), `report.json` (8 keys, 7/7 steps ok, `result: ok`), `repetition-35-runs.log` (35 headerless projection lines, every one `rc=0 named=ok test result: ok. 1695 passed; 0 failed`), `notes.md`, and the targeted/repetition/step logs.
- `KNOWN-ISSUES.md`, `TESTING.md` and `01-05-SUMMARY.md` now record the verifier's 2-in-35 falsification on `1308903a`, the cause, the fix and the committed evidence path; the old five-run claim is explicitly superseded and D3's five-run verification is `fail`.

## Task Commits

**No commits made — the plan forbids them; the phase reviewer gates the merge.** All work is uncommitted in the working tree (`git diff HEAD` + the untracked `docs/evidence/01-truth-baseline/determinism/`). `plan_head_before == plan_head_after == 98c2f1b8`; `commits: 0` is intentional, not a lost commit. The pre-existing uncommitted `.planning/ROADMAP.md`, `.planning/STATE.md`, `.planning/config.json` and `.planning/state.json` were not mine and were left untouched.

## Files Created/Modified

- `crates/codegen/xai-grok-workspace/src/restore_fetch_tests.rs` — `PROCESS_DEATH_DEADLINE`, `PROCESS_DEATH_POLL_INTERVAL`, `wait_until_dead`, five call sites, `wait_until_dead_is_bounded_and_deadline_respecting`; +75/−5 lines.
- `docs/evidence/01-truth-baseline/determinism/determinism.cast` — asciinema v2 capture of the repetition's projection stream (3 events).
- `docs/evidence/01-truth-baseline/determinism/report.json` — 8 locked keys, `requirements: [TEST-03]`, 7 ok steps, 4 hashed artifacts, `result: ok`.
- `docs/evidence/01-truth-baseline/determinism/repetition-35-runs.log` — the completion proof (35 lines).
- `docs/evidence/01-truth-baseline/determinism/notes.md` — failure, fix, environment, table, cast provenance, re-run recipe.
- `docs/evidence/01-truth-baseline/determinism/targeted-repetition-20-named-test-runs.log` — 20/20 exact-test runs (normalized; per-test `… ok` lines dropped by the convention).
- `docs/evidence/01-truth-baseline/determinism/post-fix-repetition-35-full-crate-runs.log` — the captured step's output (identical 35 lines).
- `docs/evidence/01-truth-baseline/determinism/all-35-runs-exited-zero.log`, `every-run-s-summary-is-ok-with-zero-failed.log`, `every-run-exercised-the-named-test.log`, `all-runs-report-one-passed-count.log`, `leak-scan-over-docs-evidence.log` — projection/leak step records.
- `KNOWN-ISSUES.md` — workspace bullet rewritten around the 35-run falsification, the poll fix and the item path; `git_gate` widening history kept; "Quarantined tests: none".
- `TESTING.md` — determinism bullet rewritten in the same voice with the evidence link; "reported, not artifact-backed" and "did not rotate" claims deleted.
- `.planning/phases/01-truth-baseline/01-05-SUMMARY.md` — five-run section retitled SUPERSEDED, D3 five-run verification `status: fail` with the falsification appended, third verification entry pointing at the new item, "Determinism correction (2026-11-23, 01-06)" section appended.
- `docs/evidence/README.md` — one `determinism` bullet under "Re-running the Phase-1 items".

## Decisions Made

- **Poll, don't quarantine.** The verifier's capture showed an observation race (instant `/proc` read after an asynchronous SIGKILL), so the honest fix is the eventual property with a deadline; quarantine would have hidden a real assertion.
- **5 s / 20 ms.** 5 s bounds a pathological reap without letting a never-dying process pass; 20 ms keeps 35-run wall time negligible relative to the ~50 s per-run build. The existing `< 5 s` / `< 2 s` prompt-return bounds stay first, so teardown latency cannot be hidden by polling.
- **All five sites.** `fetch_timeout_kills_process_group_with_grandchild` (leader + grandchild), `fetch_child_drop_tears_down_process`, and the TERM-immune leader + grandchild share the same race; fixing one would leave the next rotation waiting.
- **Linux-only bounds test.** `u32::MAX as i32 == -1` is not a valid probe for the non-Linux `kill(0)` `pid_alive`, so the helper's bound is pinned where `/proc` exists; the 100 ms lower band keeps the determinism test from being timing-fragile itself.
- **The cast is the same execution, not a re-run.** `script -qec` wrapped the `evidence.py step` that ran the 35-run driver, so the cast, the step's exit status and the projection log all describe one execution.

## Deviations from Plan

**1. [mechanical] The captured step ran through a throwaway wrapper script**

- **Found during:** Task 3(c)
- **Issue:** The plan wraps `scripts/evidence.py step … -- sh <TMP>/determinism-rep.sh` directly in `script -qec '…'`. Nesting the step's `--name "post-fix repetition 35 full-crate runs"` inside `script`'s `-c` string invites quoting drift.
- **Fix:** A throwaway `<TMP>/capture-repetition.sh` (never in the repo) `exec`s exactly that `evidence.py step … -- sh <TMP>/determinism-rep.sh`; `script -qec 'sh <TMP>/capture-repetition.sh'` captured it. Same command, same step name, same recorded exit status.
- **Files modified:** none in the repo (throwaway only).
- **Verification:** `report.json` step `post-fix repetition 35 full-crate runs` → `exit 0: run 35/35: rc=0 named=ok …`; the projection log and the step log are byte-identical (same sha256).

**2. [environmental, recorded] Every repetition run recompiled the crate test target**

- **Found during:** Task 3(c)
- **Issue:** Each `cargo test -p xai-grok-workspace --lib` invocation re-ran the `Compiling xai-grok-tools` → `Compiling xai-grok-workspace` chain (`Finished test profile … in 43.05–48.76 s`), so a run is ~50 s wall although the harness finished in 5.89–6.42 s; the 35-run repetition took 30m31s (inside the plan's 20–35 min expectation).
- **Fix:** none attempted — no flag, manifest or environment was changed to speed it up; the plan's command is the command of record.
- **Verification:** 35/35 raw logs contain the `Compiling xai-grok-workspace` line; `notes.md` states the wall-vs-harness difference.

**Total deviations:** 2 (1 mechanical, 1 environmental/recorded). **Impact on plan:** none on scope, assertions or evidence semantics; no fallback branch was needed.

## Issues Encountered

- The verifier's failure mode was reproduced conceptually, not by re-running to catch a red: the fix targets the exact observation (`returned from teardown` vs `reaped`) at the exact line (`restore_fetch_tests.rs:731`), and the falsified file was byte-identical to HEAD before the change (sha256 `62a2656a…`).
- The new bounds test bumps the crate's passed count from 1694 to 1695. `scripts/test-gate.expected` counts top-level `test result:` lines (targets), not tests, so it correctly stays at 308/219/89 and was not touched.
- A server restart interrupted only my polling sleeps; the 35-run capture survived as an intact process tree and finished at `19:38:44Z` with `CAPTURE_RC=0` (the same adoption situation 01-05 documented).

## User Setup Required

None - no external service configuration required.

## Verification (all executed this session)

| Check | Result |
|---|---|
| `cargo test -p xai-grok-workspace --lib -- --exact restore_fetch::tests::fetch_timeout_sigkills_term_immune_grandchild` ×20 | 20/20 green, each `1 passed; 0 failed` (2.40 s) |
| `cargo test -p xai-grok-workspace --lib -- restore_fetch` | 31 passed / 0 failed incl. the bounds test |
| `cargo test -p xai-grok-workspace --lib -- --test-threads=4` ×5 (task 2) | 5/5 green, each `1695 passed; 0 failed` |
| 35-run post-fix repetition (captured step) | **35/35** green, each `1695 passed; 0 failed`, named test ok; 30m31s |
| Projection gates (4 awk counts over the committed log) | 35 rc=0 lines, 35 `test result: ok.`, 35 `0 failed`, 35 `named=ok`, one distinct passed count (`1695 passed;`) |
| `report.json` schema | exactly 8 keys, `requirements: [TEST-03]`, 7/7 steps ok, `result: ok` |
| Cast | v2 header, 3 `[elapsed, "o", str]` events, all 35 projection lines present |
| Artifact hashes | 4/4 sha256 match `report.json` |
| `scripts/key-leak-check.sh --scan docs/evidence` | PASS, exit 0 (recorded as a step too) |
| `/home/[a-z]` grep over the item | no hits |
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy -p xai-grok-workspace --all-targets` | exit 0, no warning naming `restore_fetch_tests.rs` |
| Final `cargo test -p xai-grok-workspace --lib -- --test-threads=4` | exit 0, `1695 passed; 0 failed` (bounds test `ok`) |
| Diff review | five death sites call `wait_until_dead`; no test/assertion removed or reordered past an elapsed bound; no `assert_timeout_about` band moved; no `#[ignore]` in the file; `restore_fetch.rs` sha256 unchanged (`7bc753de…`) |
| Frozen files | `scripts/test-gate.expected`, `Cargo.toml`, `Cargo.lock`, `deny.toml`, and the three existing evidence items unchanged |

## Next Phase Readiness

- TEST-03 is closed with an artifact a reviewer can recount: 35/35 runs, one projection line each, cross-checked against the cast of the same execution.
- The evidence convention now has a repetition item that later phases can copy for any flake claim.
- **Reviewer gate outstanding:** the 01-06 diff (one Rust test file + five docs + the new item) needs PASS/PASS_WITH_NOTES before the orchestrator commits. No commits were made.

## Self-Check: PASSED

- FOUND: `crates/codegen/xai-grok-workspace/src/restore_fetch_tests.rs` (`wait_until_dead`, `PROCESS_DEATH_DEADLINE`, `PROCESS_DEATH_POLL_INTERVAL`, 5 call sites, bounds test)
- FOUND: `docs/evidence/01-truth-baseline/determinism/{determinism.cast,report.json,notes.md,repetition-35-runs.log,targeted-repetition-20-named-test-runs.log,post-fix-repetition-35-full-crate-runs.log}`
- FOUND: corrected `KNOWN-ISSUES.md`, `TESTING.md`, `.planning/phases/01-truth-baseline/01-05-SUMMARY.md`, `docs/evidence/README.md`
- VERIFIED: no commits claimed (`HEAD` still `98c2f1b8`); artifact sha256s match the report; the 35-line projection log passes all count gates

---
*Phase: 01-truth-baseline*
*Completed: 2026-11-23*
