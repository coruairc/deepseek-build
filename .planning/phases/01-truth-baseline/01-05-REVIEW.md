---
phase: 01-truth-baseline
plan: 05
reviewed: 2026-11-22T05:35:00Z
depth: standard
files_reviewed: 11
files_reviewed_list:
  - scripts/test-gate.sh
  - scripts/test-gate.expected
  - scripts/evidence.py
  - scripts/key-leak-check.sh
  - scripts/README.md
  - docs/evidence/README.md
  - docs/evidence/01-truth-baseline/ipv6-report/
  - docs/evidence/01-truth-baseline/test-gate/
  - docs/evidence/01-truth-baseline/cost-estimate/
  - crates/codegen/xai-grok-shell/src/agent/folder_trust.rs
  - crates/codegen/xai-grok-shell/src/util/config/consent_tests.rs
  - crates/codegen/xai-grok-workspace/src/session/git_gate_tests.rs
  - KNOWN-ISSUES.md
  - TESTING.md
  - .planning/phases/01-truth-baseline/01-05-SUMMARY.md
findings:
  critical: 0
  warning: 2
  info: 3
  total: 5
status: issues_found
verdict: PASS_WITH_NOTES
---

# Phase 01 Plan 05: Code Review Report (Test gate and evidence)

**Reviewed:** 2026-11-22T05:35:00Z
**Base:** `882ad0de` (01-02) → `866ca00a` plus the working-tree evidence/docs
**Depth:** standard, **static only** — the owner deferred all test runs on
2026-11-22, so nothing here was executed.
**Status:** PASS_WITH_NOTES (no blocker)

> The previously spawned reviewer subagent was lost when the server restarted;
> this is the maintainer's static pass over the same diff.

## Summary

The gate cannot go green by accident. `scripts/test-gate.sh` counts one
top-level `test result:` line per cargo target block (so
`xai_tty_utils`'s self-re-exec line cannot inflate the count), derives
`binaries` from `cargo test --no-run --message-format=json` filtered to
`profile.test == true` with a non-null executable, counts `Doc-tests` lines, and
asserts `results == binaries + doctests` plus `results == expected`. The verdict
never reads the log tail. `--refresh-expected` is the only writer and refuses
when `fail != 0`; subset mode skips the expected comparison and says so.

`scripts/evidence.py` writes exactly the eight locked keys, refuses
`--result ok` over a failed step, normalizes `<REPO>/<HOME>/<TMP>` in logs and
report strings, and drops per-test success lines while keeping `Running`,
`Doc-tests`, `test result:` and failure text. `cast` converts `script -T` output
to asciinema v2; the committed `ipv6-report.cast` decodes to the six `PASS`
lines with plausible timing, confirming that the typescript header line is the
one dropped.

`key-leak-check.sh --scan` is additive: with no `--scan` the script takes the
original branch byte-for-byte (binary precheck, live run, canary scan), and the
scan mode prints file names only and never a matched line.

The determinism edits move waits and TTLs in one direction only (longer) and
leave every assertion, ordering invariant and walk-count check intact; the
serial-key change makes the two `DEEPSEEK_BUILD_HOME` tests share the crate's
dominant bare key, which is the only way they actually exclude each other.

The two skipped evidence items are honest: `result: skipped`, zero steps, no
cast, and a `notes.md` that says no run was performed and names the rerun
recipe. No fabricated run is recorded.

## Findings

### WARNING

**W-1 — EVID-01 is only partially met.**
`docs/evidence/01-truth-baseline/` has one complete item (`ipv6-report`) and two
`result: skipped` items (`test-gate`, `cost-estimate`). The requirement says
"every loop item carries a committed cast plus report.json". This is an owner
decision, not a code defect; it is carried in `01-VERIFICATION.md` and must be
closed before a release claim.

**W-2 — the expected count is not confirmed on this host.**
`scripts/test-gate.expected` is `results=308`, derived from the 2026-11-21 run
on the author's host. The confirmation run was cancelled, so a different target
count here would not be caught until testing resumes. The provenance header and
the `--refresh-expected`-only rule are correct; the number itself is unverified
here.

### INFO

**I-1 — `--refresh-expected` writes before the leak scan runs.**
The refresh write happens at step 7; the `docs/evidence` leak scan is step 8.
The expected count depends only on the cargo results, so this is defensible and
matches the docstring ("only when both steps are green"), but a refresh on a
tree with leaking evidence rewrites the count and then fails the run.

**I-2 — `--package` accepts a leading hyphen.**
The name check is `[A-Za-z0-9_-]*`, so `-p -foo` would be forwarded to cargo
and fail there rather than at argument parsing. Cosmetic; no injection surface
(no `eval`, arguments are separate words).

**I-3 — `cast` assumes one leading typescript header line.**
`cmd_cast` drops everything before the first newline, which is correct for
`script(1)` with `-q` (the header is written to the typescript file), as the
committed cast demonstrates. A future `script` variant without the header would
lose the first output line; the docstring documents the assumption.

## Verdict

**PASS_WITH_NOTES.** No critical or security finding. The two warnings are
recorded deferrals, not defects; commit the plan and carry W-1/W-2 forward in
`01-VERIFICATION.md`.
