---
phase: 01-truth-baseline
plan: 05
status: complete
subsystem: test-gate-and-evidence
tags: [test-gate, determinism, evidence, evid-01, evid-02, test-01, test-02, test-03]
requirements: [TEST-01, TEST-02, TEST-03, EVID-01, EVID-02]
dependency_graph:
  requires: ["01-01", "01-02", "01-03", "01-04"]
  provides:
    - "scripts/test-gate.sh with counted completeness and a checked-in expected count"
    - "scripts/evidence.py + docs/evidence/README.md (the evidence convention)"
    - "scripts/key-leak-check.sh --scan"
  affects:
    - "how every later phase records proof"
tech_stack:
  added: []
  patterns:
    - "gate verdict is arithmetic (results == binaries + doctests == expected), never a log tail"
    - "evidence item = cast + fixed-key report.json + notes.md under docs/evidence/"
key_files:
  created:
    - scripts/test-gate.sh
    - scripts/test-gate.expected
    - scripts/evidence.py
    - docs/evidence/README.md
    - docs/evidence/01-truth-baseline/ipv6-report/
    - docs/evidence/01-truth-baseline/test-gate/
    - docs/evidence/01-truth-baseline/cost-estimate/
  modified:
    - scripts/key-leak-check.sh
    - scripts/README.md
    - crates/codegen/xai-grok-shell/src/agent/folder_trust.rs
    - crates/codegen/xai-grok-shell/src/util/config/consent_tests.rs
    - crates/codegen/xai-grok-workspace/src/session/git_gate_tests.rs
    - KNOWN-ISSUES.md
    - TESTING.md
decisions:
  - "The completeness proof counts one top-level `test result:` line per cargo target block; nested self-re-exec output is excluded."
  - "The expected count may only move through --refresh-expected on a fully green run."
  - "Only waits and upper bounds moved for the timing tests; no assertion direction or ordering invariant changed."
commits: 2
actuals:
  tokens: 0
  tasks: 3
  commits: 2
metrics:
  duration: "long-running (interrupted once, then tests deferred by the owner)"
  completed: "2026-11-22"
---

# Phase 1 Plan 5: Test gate and evidence convention Summary

One gate that cannot be green by accident, and one evidence convention a later
reader can re-run. The code is complete and committed. **Every test, gate and
verification step in this plan is DEFERRED (tests skipped by owner,
2026-11-22)** — the owner directed that no further test, gate or smoke run be
performed for the rest of the milestone.

## Per-task changes

### Task 1 — one gate that proves it ran every target

`scripts/test-gate.sh` (new, bash):

1. `scripts/disk-guard.sh 150` first; `TEST_GATE_MIN_GB` lowers the floor for a
   smaller host without changing the default.
2. Re-execs once under `flock -w 900 /tmp/team-build.lock`, so the exit code is
   the gate's, not flock's.
3. Step 1 `cargo test --workspace --no-fail-fast -- --test-threads=4`, status
   captured (no `tee`), log kept.
4. Step 2 `cargo test -p xai-grok-shell --lib --features config-docs -j 1`.
5. Counts `results` (one per target block), `doctests` (`   Doc-tests ` lines)
   and `binaries` (unique test executables from
   `cargo test --no-run --message-format=json` via `jq`), then asserts
   `results == binaries + doctests` and, in full mode, `results == expected`.
6. `--package` subset mode (prints `GATE MODE: SUBSET`, skips the expected
   comparison), `--refresh-expected` (writes only after a fully green run, with
   commit/date/count provenance), `--log-dir`.
7. Leak scan over `docs/evidence` as the final step, part of the exit code.

`scripts/test-gate.expected` is seeded from the 2026-11-21 green full run:
`results=308`, `binaries=219`, `doctests=89`, commit `882ad0de`.

**Verification: DEFERRED (tests skipped by owner, 2026-11-22).** The intended
`bash -n` / `shellcheck` / subset-run checks were not re-run on this host;
`shellcheck` is not installed here. The script is syntax-checked with `bash -n`.

### Task 2 — timing flake and the process-global serial key

- `agent::folder_trust::tests::record_and_lookup_round_trip` gained
  `#[serial_test::serial]`; `util::config::consent::tests::set_consent_answer_is_monotonic_per_account`
  dropped its named key (`serial(DEEPSEEK_BUILD_HOME)`), so both share the
  crate's dominant bare key and actually exclude each other. Bodies unchanged.
- `xai-grok-workspace` waits widened, invariants untouched:
  - `joiners_do_not_start_a_second_walk`: gate TTL 500 ms → 5 s.
  - `walk_timeout_after_ttl_does_not_resurrect_snapshot`: TTL 20 → 100 ms,
    walk timeout 40 → 100 ms, post-call sleep 40 → 250 ms (sleep > TTL kept).
  - `late_ok_after_waiter_timeout_is_snapshotted`: walk timeout 40 → 100 ms,
    sleep after `notify_waiters` 50 → 250 ms (sleep > timeout kept).
  - `root_cache_expires_so_a_new_nested_repo_is_rediscovered`: sleep
    100 → 500 ms (past the 80 ms test-only `ROOT_CACHE_TTL`).
  - `sustained_invalidate_during_inflight_run_returns_after_at_most_two_walks`:
    outer timeout multiplier 3 → 10; walk-count assertion unchanged.
  - `status_and_diffs_never_overlap_odb_walks` and all `restore_fetch` bands
    left unchanged: no recorded rotation named them.
- The five-run repetition and the `config-docs` suite are **DEFERRED (tests
  skipped by owner, 2026-11-22)**. No test is quarantined: a quarantine needs a
  named rotating test and a reproducer, and no repetition was run.

### Task 3 — evidence convention and the recorded run

- `scripts/evidence.py` (new, stdlib only): `begin`, `step`, `artifact`,
  `finish`, `cast`, `normalize`. `finish` refuses `--result ok` over a failed
  step and writes exactly the eight locked keys; it normalizes `<REPO>`,
  `<HOME>`, `<TMP>` and drops per-test success lines.
- `scripts/key-leak-check.sh`: additive `--scan <path>...` mode (no binary
  precheck, no live run; file names only; exit 1 on a hit). No-argument
  behaviour unchanged.
- `docs/evidence/README.md`: layout, fixed keys, normalization table, the
  no-env-dump rule, the recording recipe, and the append-never-rewrite rule.
- `scripts/README.md`: one-line entries for `test-gate.sh` and `evidence.py`.
- Items under `docs/evidence/01-truth-baseline/`:
  - `ipv6-report/` — **complete**: cast + `report.json` + six step logs + notes,
    earned by `sh scripts/test-sandbox-run.sh`.
  - `test-gate/` — **`result: skipped`**: the 2026-11-21 run log is kept, with
    `notes.md` stating it is not a re-confirmation; no cast.
  - `cost-estimate/` — **`result: skipped`**: no run, no cast; `notes.md` names
    the code and the tests that pin the strings.
- `KNOWN-ISSUES.md` / `TESTING.md`: record the 2026-11-21 result, the owner
  deferral, and the host-tooling gaps (`pkg-config`, `shellcheck`,
  `actionlint`).

## Requirements

| Req | State |
|---|---|
| TEST-01 | Code complete; gate run **DEFERRED (tests skipped by owner, 2026-11-22)** |
| TEST-02 | `config-docs` is a named gate step; run **DEFERRED** |
| TEST-03 | Determinism fixes applied; five-run repetition **DEFERRED** |
| EVID-01 | Convention + `ipv6-report` complete; `test-gate` and `cost-estimate` **skipped** by owner decision |
| EVID-02 | Complete: `docs/evidence/README.md` documents layout, keys, normalization and re-run recipes |

## Deviations

1. **Gate confirmation and all later test runs cancelled by the owner
   (2026-11-22).** Only run #2 (2026-11-21) is green; the expected count was not
   re-derived on this host.
2. **`TEST_GATE_MIN_GB` added** (commit `866ca00a`): the hard-coded 150 GB floor
   could not run on a 63 GB host. Default unchanged.
3. **`shellcheck` / `actionlint` unavailable here**; only `bash -n` was run.
4. **`openssl-sys` needs `OPENSSL_LIB_DIR` / `OPENSSL_INCLUDE_DIR`** on this
   host because `pkg-config` is absent; recorded in `KNOWN-ISSUES.md`.
5. Evidence casts/reports for `test-gate` and `cost-estimate` are not invented;
   the reports are `result: skipped` with notes explaining the deferral.
