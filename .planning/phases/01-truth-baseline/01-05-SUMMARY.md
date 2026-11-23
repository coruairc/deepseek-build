---
phase: 01-truth-baseline
plan: 05
subsystem: testing
tags: [test-gate, evidence, asciinema-v2, determinism, serial-test, leak-scan]

requires:
  - phase: 01-truth-baseline (plans 01-01..01-04)
    provides: sandbox deny predicate, IPv6 refusal naming, cost estimate label, default model — the items this gate and evidence prove
provides:
  - scripts/test-gate.sh with arithmetic completeness assertion and checked-in expected count (308 = 219 + 89)
  - scripts/evidence.py + docs/evidence/ convention (cast + fixed-schema report.json + normalized paths)
  - three recorded Phase-1 evidence items (test-gate, ipv6-report, cost-estimate)
  - determinism fixes (bare serial key unification, widened git_gate waits) with a recorded 5-run repetition
affects: [02-live-session-proof, 03, 04, 05, any phase appending docs/evidence]

actuals:
  tokens: 40933    # chars/4 over the realized plan diff (WIP b0516ef1 + uncommitted), incl. untracked evidence files
  tasks: 3
  commits: 0       # intentional: plan forbids commits; reviewer gates the merge (WIP checkpoint b0516ef1 carries tasks 1-2)
plan_head_before: 00aebc866c375e7b0b6fc3f80483e81bd0ef1fc8
plan_head_after: 00aebc866c375e7b0b6fc3f80483e81bd0ef1fc8

tech-stack:
  added: []        # stdlib-only Python helper; composes existing flock/jq/awk/script/shellcheck
  patterns:
    - "counted completeness: results == build-graph executables + Doc-tests lines == checked-in expected"
    - "evidence item: asciinema v2 cast + 8-key report.json + notes.md, <REPO>/<HOME>/<TMP> normalization"
    - "expected count moves only via --refresh-expected on a fully green run, with commit/date provenance"

key-files:
  created:
    - scripts/test-gate.sh
    - scripts/test-gate.expected
    - scripts/evidence.py
    - docs/evidence/README.md
    - docs/evidence/01-truth-baseline/test-gate/ (cast, report.json, notes.md, normalized gate log, 4 step logs)
    - docs/evidence/01-truth-baseline/ipv6-report/ (cast, report.json, notes.md, 6 step logs)
    - docs/evidence/01-truth-baseline/cost-estimate/ (cast, report.json, notes.md, 4 step logs)
  modified:
    - scripts/key-leak-check.sh (--scan mode, additive)
    - scripts/README.md
    - crates/codegen/xai-grok-shell/src/agent/folder_trust.rs (#[serial] on record_and_lookup_round_trip)
    - crates/codegen/xai-grok-shell/src/util/config/consent_tests.rs (named serial key -> bare)
    - crates/codegen/xai-grok-workspace/src/session/git_gate_tests.rs (widened waits only)
    - KNOWN-ISSUES.md
    - TESTING.md

key-decisions:
  - "Adopted the orphaned-but-intact confirmation run #3 after the server restart instead of re-running (orchestrator-approved); its evidence.py step record carries the real exit status"
  - "test-gate.log is the confirmation run #3 step-1 log; run #2's seeded numbers live in the provenance step and the expected file header"
  - "Expected count stays 308/219/89 — confirmation reproduced it exactly; no --refresh-expected on resume"

patterns-established:
  - "Gate honesty: the verdict is the printed arithmetic and exit code, never a log tail or status word"
  - "Evidence steps run real commands via evidence.py step; finish refuses ok over a failed step"
  - "Later phases append docs/evidence/NN-<slug>/; they never rewrite an earlier item"

requirements-completed: [TEST-01, TEST-02, TEST-03, EVID-01, EVID-02]

coverage:
  - id: D1
    description: "One gate that proves it ran every target: workspace step green, completeness asserted against build graph and checked-in count"
    requirement: TEST-01
    verification:
      - kind: integration
        ref: "scripts/test-gate.sh (full confirmation run #3, 2026-11-23): results=308 == binaries 219 + doctests 89 == expected 308, exit 0"
        status: pass
      - kind: integration
        ref: "scripts/test-gate.sh --package xai-grok-status-line --package xai-grok-sandbox (subset mechanism): 9 == 7 + 2, exit 0"
        status: pass
    human_judgment: false
  - id: D2
    description: "config-docs feature step runs as a named gate step and is green"
    requirement: TEST-02
    verification:
      - kind: integration
        ref: "gate step 2: cargo test -p xai-grok-shell --lib --features config-docs -j 1 — 6201 passed / 0 failed / 4 ignored, rc=0 (run #3)"
        status: pass
    human_judgment: false
  - id: D3
    description: "Timing flake made deterministic: bare serial key unification + widened git_gate waits; the 5-run sample was falsified by the verification's 35-run repetition and superseded by the 01-06 bounded death poll (35/35, artifact-backed), no quarantine"
    requirement: TEST-03
    verification:
      - kind: unit
        ref: "5x cargo test -p xai-grok-workspace --lib -- --test-threads=4 (task 2, 2026-11-21): each 1694 passed / 0 failed — reported, not artifact-backed (prior session; IN-04); FALSIFIED by the phase verification's 35-run repetition on 1308903a: 2 failures in 35, the captured one restore_fetch::tests::fetch_timeout_sigkills_term_immune_grandchild (grandchild <pid> must not remain live, restore_fetch_tests.rs:731)"
        status: fail
      - kind: integration
        ref: "run #3 workspace step: 31913 passed / 0 failed / 386 ignored incl. git_gate/restore_fetch"
        status: pass
      - kind: integration
        ref: "01-06 post-fix repetition: 35x cargo test -p xai-grok-workspace --lib -- --test-threads=4, each 1695 passed / 0 failed with the named test ok — docs/evidence/01-truth-baseline/determinism/repetition-35-runs.log (report.json result: ok)"
        status: pass
    human_judgment: false
  - id: D4
    description: "Three Phase-1 evidence items, each with cast + schema-exact report.json + normalized paths; leak scan passes over docs/evidence inside the gate"
    requirement: EVID-01
    verification:
      - kind: other
        ref: "python3 8-key schema assert over docs/evidence/01-truth-baseline/*/report.json — 3/3 OK; sha256 of every artifact matches report"
        status: pass
      - kind: other
        ref: "scripts/key-leak-check.sh --scan docs/evidence — PASS, exit 0 (also gate step 3, rc=0)"
        status: pass
    human_judgment: false
  - id: D5
    description: "Evidence convention documented well enough to re-run (layout, fixed keys, normalization, re-run recipes)"
    requirement: EVID-02
    verification:
      - kind: other
        ref: "docs/evidence/README.md; every recipe exercised by the recorded items (test-gate subset/full, ipv6 harness, cost suites)"
        status: pass
    human_judgment: true
    rationale: "Documentation prose quality and re-runnability for a future reader is a reviewer judgment; the recipes were exercised but the page itself needs a human read"

duration: 70min (this continuation; tasks 1-2 completed in prior sessions 2026-11-21)
completed: 2026-11-23
status: complete
---

# Phase 01 Plan 05: Test gate + evidence Summary

**A counted test gate (308 results = 219 binaries + 89 doctests, confirmed twice) plus a re-runnable evidence convention with three recorded Phase-1 items, all leak-scanned and path-normalized**

## Performance

- **Duration:** ~70 min for this continuation (task 3 finish + confirmation run + verification); tasks 1–2 executed 2026-11-21 in prior sessions
- **Started:** 2026-11-23T13:12:42Z (this dispatch)
- **Completed:** 2026-11-23T14:22:08Z
- **Tasks:** 3/3
- **Files modified:** 21 in WIP checkpoint + 14 uncommitted (evidence items, docs, one comment fix)

## The gate's four numbers and expected-count provenance

| Run | Date (UTC) | Mode | results | binaries | doctests | expected | Exit |
|---|---|---|---|---|---|---|---|
| #2 (seed) | 2026-11-21T21:37:20Z | `--refresh-expected` | 308 | 219 | 89 | written: 308 | 0 |
| #3 (confirmation) | 2026-11-23 | plain | 308 | 219 | 89 | 308 | 0 |

- Provenance header in `scripts/test-gate.expected`: refreshed by `scripts/test-gate.sh --refresh-expected`, commit `882ad0de`, 2026-11-21T21:37:20Z. The historical "307 targets" figure (2026-10-09, different tree) was never seeded.
- Run #3 verdict (printed, then re-derived independently with `awk` from its log): step 1 rc=0, step 2 rc=0, `results == binaries + doctests` PASS (308 == 308), `results == expected (308)` PASS, leak scan rc=0, **GATE PASS**. Raw `test result:` lines are 309 (one nested re-run line from `xai_tty_utils::stderr_redirect_roundtrip_subprocess`); `results` counts one top-level result per cargo target block.
- Run #3 totals: step 1 **31,913 passed / 0 failed / 386 ignored**; step 2 (`config-docs`) **6,201 passed / 0 failed / 4 ignored** (artifact-backed post-review by the supplementary excerpt `docs/evidence/01-truth-baseline/test-gate/step2-config-docs-result-line.log`, IN-04).
- The expected file was **not edited** on resume.

## Five-run repetition (task 2, cited — not re-run; SUPERSEDED by 01-06: the verification's 35-run repetition falsified it)

Five consecutive `cargo test -p xai-grok-workspace --lib -- --test-threads=4` runs (2026-11-21, under the shared lock and gate environment) each reported **1694 passed / 0 failed** — no rotation, so no `restore_fetch` band moved and **no test is quarantined**. The raw per-run summary logs were not committed (IN-04): these numbers rest on the prior executor's report; run #3's committed step-1 log independently shows `xai-grok-workspace` 1694/0 once (`test-gate.log` line 1934). Widened waits (old → new): root-cache sleep 100 → 500 ms; walk-timeout TTL 20 → 100 ms with sleep 40 → 250 ms; late-OK walk timeout 40 → 100 ms with sleep 50 → 250 ms; joiner TTL 500 ms → 5 s; sustained-invalidate outer timeout ×3 → ×10. Serial keys: `record_and_lookup_round_trip` gained bare `#[serial_test::serial]`; the consent test's named key `serial(DEEPSEEK_BUILD_HOME)` became bare, so the two env-global tests exclude each other. No assertion changed direction; `restore_fetch_tests.rs` untouched.

## Three evidence items (EVID-01/02)

| Item | Requirements | Artifacts |
|---|---|---|
| `docs/evidence/01-truth-baseline/test-gate/` | TEST-01, TEST-02, EVID-01 | `test-gate.cast` (12 events, subset-mode mechanism), `test-gate.log` (run #3 step-1 log, normalized, 2755 lines), `report.json` (4 ok steps incl. the full-run verdict), `notes.md`, 4 step logs |
| `docs/evidence/01-truth-baseline/ipv6-report/` | SBOX-02 | `ipv6-report.cast` (8 events), `report.json` (6 ok assertion steps), `notes.md`, 6 step logs |
| `docs/evidence/01-truth-baseline/cost-estimate/` | COST-02, COST-03 | `cost-estimate.cast` (774 events), `report.json` (4 ok steps; details carry `~$0.37 est.`, `~$0.0042 est.`, `~$0.01 est.`, `~$0.0001 est.` and `tui_and_headless_report_the_same_cost_for_one_override`), `notes.md`, 4 step logs |

- Cost suites in the capture: `status_line` filter 104 passed / 0 failed; `headless` filter 193 passed / 0 failed. `notes.md` states **no pty session renders the status line today** — the cast shows the suite that pins the strings; no TUI session was invented.
- Every `report.json` has exactly the 8 locked keys; every artifact sha256 verified against its file; casts are valid asciinema v2; all paths normalized (`<REPO>/<HOME>/<TMP>`).
- **Leak scan:** `scripts/key-leak-check.sh --scan docs/evidence` → PASS, exit 0, no file names — both as gate step 3 (rc=0) and re-run over the final tree.

## Verification block (all executed this session)

| Check | Result |
|---|---|
| `scripts/test-gate.sh` exit 0, four numbers + both comparisons | PASS (run #3; counts re-derived with awk) |
| `scripts/key-leak-check.sh --scan docs/evidence` | PASS, exit 0, no file name |
| 8-key schema on all `*/report.json` | 3/3 OK |
| `grep '<HOME>' test-gate/report.json` + `! grep -rnE '/home/[a-z]' docs/evidence` | PASS |
| `! grep -rnE 'sk-[A-Za-z0-9]{20,}' docs/evidence` | PASS |
| `bash -n test-gate.sh`, `sh -n test-sandbox-run.sh`, `shellcheck --shell=sh` (3 scripts) | PASS |
| `python3 ast.parse` on `scripts/evidence.py` | PASS |
| `cargo fmt --all --check` | PASS |
| `git diff` review: no Cargo.toml/lockfile/deny.toml; no test lost an assertion | PASS (task-2 diff widens waits only; 308 targets in both runs) |
| Clippy on touched crates (`xai-grok-shell`, `xai-grok-workspace`, `--all-targets` + config-docs variant) | 0 errors, exit 0; no warnings in touched files (pre-existing warnings elsewhere untouched) |

## Task Commits

**No commits made — the plan forbids them; the phase reviewer gates the merge.** Tasks 1–2 and the partial task-3 work sit in WIP checkpoint `b0516ef1` (prior executor); this dispatch's work is uncommitted in the working tree for review (`git diff HEAD` + untracked `docs/evidence/` files). `plan_head_before == plan_head_after == 00aebc86`; `commits: 0` is intentional, not a lost commit.

## Deviations from Plan

**1. Server-restart interruption; adopted the surviving confirmation run #3 (orchestrator-approved)**
- **Found during:** Task 3 resume
- **Issue:** The restart note declared run #3 killed mid-compilation. In fact the previous executor's detached `evidence.py step` process tree (wrapper PID 7830 under the surviving pty daemon) was alive with both stdout pipes intact, past compilation and running tests.
- **Fix:** Adopted the live, properly-wrapped plain confirmation run instead of killing it and starting a competing build; verified on exit that the step record landed with the real exit status (`exit 0: GATE PASS`) and re-derived the counts independently from the log.
- **Verification:** pending-state step record + gate stdout log + awk counts (309 raw / 308 top-level / 89 doctests, 0 FAILED lines).

**2. cost-estimate item was already begun (restart note said "not started")**
- **Found during:** Task 3 resume
- **Issue:** The killed executor had run `begin` (13:10:04Z) and written `notes.md` before dying.
- **Fix:** Continued with `step/artifact/finish` only; did not re-run `begin`. Verified notes.md's strings and test names against the source before recording steps.

**3. [Rule 1 - cosmetic] Stray `#` in test-gate.sh header comment**
- **Found during:** Task 3 verification prep
- **Issue:** Line 22 ended `…never the verdict.#` (comment typo from task 1).
- **Fix:** Removed the stray character after the gate run finished (never edit a script while bash is executing it). Comment-only; `bash -n` + shellcheck re-verified clean.
- **Files modified:** `scripts/test-gate.sh` (uncommitted)

**4. test-gate.log now carries run #3, not run #2**
- **Found during:** Task 3 evidence assembly
- **Issue:** The item log left by the prior executor was run #2's; the item's command of record is the confirmation run.
- **Fix:** Replaced with run #3's step-1 log, normalized via `evidence.py normalize`. Run #2's numbers remain in the provenance step log and the expected-file header; notes.md documents both runs.

---

**Total deviations:** 4 (2 resume-state reconciliations, 1 cosmetic auto-fix, 1 evidence assembly decision)
**Impact on plan:** None on scope or correctness. No expected-count edit, no weakened assertion, no new dependency, no outbound destination.

## Issues Encountered

- The "killed" run #3 was alive — resolved by adoption (deviation 1), avoiding a duplicate 45-minute workspace run and a flock collision.
- `scripts/check-egress.sh --strict` remains red at base on three committed test fixtures (Phase 5 owns it, FREEZE-03) — unchanged, out of scope.
- Pre-existing `process_key_from_model_env_key` can print an exported key on failure — gate unsets the three key vars; recorded in KNOWN-ISSUES.md, follow-up outside this phase.

## Known Stubs

None. No placeholder data, no skipped tests added, no unrun verification.

## Threat Flags

None — no new network endpoint, auth path, or trust-boundary surface. T-01-13/14/16/17 mitigations are the gate's counted assertion, the leak scan + normalization, real-exit-step recording, and refresh-only provenance respectively; all exercised this session.

## Next Phase Readiness

- The gate is the release-claim prerequisite and is green twice over; Phase 2 appends `docs/evidence/02-<slug>/` items using the documented recipe.
- **Reviewer gate outstanding:** the full 01-05 diff (WIP `b0516ef1` + uncommitted) needs PASS/PASS_WITH_NOTES before the orchestrator commits.
- KNOWN-ISSUES.md / TESTING.md carry the recorded numbers, provenance, the devbox obligation note, and "quarantined tests: none".

## Self-Check: PASSED

- FOUND: scripts/test-gate.sh, scripts/test-gate.expected, scripts/evidence.py, docs/evidence/README.md
- FOUND: docs/evidence/01-truth-baseline/{test-gate,ipv6-report,cost-estimate}/{*.cast,report.json,notes.md}
- FOUND: WIP commit b0516ef1 (ancestor of HEAD 00aebc86); no commits claimed by this dispatch
- VERIFIED: artifact sha256s match report.json entries; gate counts re-derived from the run log

## Post-review fixes (2026-11-23, iteration 1)

The phase review (`01-REVIEW.md`, verdict FAIL: 1 blocker + 6 warnings + 6 info)
was fixed in full — **13/13 findings fixed, 0 skipped** — by the review-fix
dispatch. Details per finding: `01-REVIEW-FIX.md`; disposition rows: `fixed` in
`01-REVIEW-DISPOSITION.md`. **No commits** (dispatch override; the orchestrator
commits after re-review PASS). All fixes are uncommitted in the working tree.

**What changed:**
- `scripts/key-leak-check.sh` — CR-01: `--scan` branches grep rc 0/1/else and
  fails closed (exit 1) on grep errors, printing diagnostics only. Default
  (no-arg) mode byte-identical (single-hunk diff inside the `--scan` branch).
- `scripts/test-gate.sh` — WR-01: subset runs end `GATE PASS (SUBSET — not the
  phase gate)`; WR-02: the `--refresh-expected` writer gained a `# tree:
  clean/dirty` stamp — **defective as first written** (computed inside the
  truncating redirect group, so every refresh said `dirty`); corrected in the
  iteration-2 re-review pass below.
- `scripts/evidence.py` — WR-03: zero-step `finish --result ok` refused
  (exit 2); launch failures (OSError) recorded as failed steps. WR-04: step
  record persisted before the echo; BrokenPipeError-tolerant echo. WR-05:
  artifact sha256s recomputed after normalization. WR-06: cast normalization
  carries partial substitution prefixes (and incomplete UTF-8 tails) across
  timing chunks. IN-02: explicit `Script done` trailer strip; missing/empty
  timing over a non-empty body exits 2. IN-06: `errors="replace"` in
  normalize/finish reads.
- `docs/evidence/README.md` — IN-01 recipe reordered (capture first, defined
  typescript path, `grep -F` without `-q`); IN-05 text-only/binary-skip
  limitation documented; WR-05 artifact ordering rule; WR-03 finish semantics.
- `TESTING.md` — IN-03 step 11 expects the known `--strict` red baseline
  (exit 1, 3 HARD hits; Phase 5/FREEZE-03 owns the fix); IN-04 markers.
- `KNOWN-ISSUES.md`, `test-gate/notes.md`, this SUMMARY — IN-04: step-2
  6,201/0/4 now artifact-backed by the NEW supplementary
  `docs/evidence/01-truth-baseline/test-gate/step2-config-docs-result-line.log`
  (verbatim excerpt of the raw run-#3 step-2 log, normalized, scan-clean);
  five-run 1694×5 marked "reported, not artifact-backed (prior session)".
  WR-02 provenance caveat added to notes.md (run #2's stamp `882ad0de` predates
  WIP `b0516ef1`; run #3 re-confirmed on the merged tree).

**Verification (all executed this session, main checkout):**
- `bash -n` + `shellcheck --shell=sh` on both shell scripts: clean;
  `ast.parse` on evidence.py: clean; `cargo fmt --all --check`: exit 0 (no
  Rust changes).
- CR-01 reproduction: mode-000 synthetic-canary file → FAIL exit 1 (grep rc=2,
  diagnostics only); readable canary → FAIL with file name; clean → PASS;
  temp dir removed. Full `--scan docs/evidence`: PASS exit 0.
- Subset gate run (keys unset, under the gate's flock): exit 0, 9 == 7 + 2,
  leak scan rc=0, final line `GATE PASS (SUBSET — not the phase gate)`; its
  step 2 independently re-confirmed 6201/0/4.
- 31/31 evidence.py temp-dir smoke checks: launch-failure recording, zero-step
  refusal, real broken-pipe record survival, stale-digest refresh, non-UTF-8
  normalize, split-prefix cast (home/repo/UTF-8 splits) with a red proof
  against the pre-fix HEAD code, trailer strip, timing errors.
- Frozen integrity re-checked after fixes: 3/3 report.json 8-key schema, 4/4
  artifact sha256 match; casts not regenerated; `/home/[a-z]` and `sk-` greps
  over docs/evidence (incl. the new excerpt): no hits; concatenated cast data
  clean. `check-egress.sh --strict` re-run: exit 1, 3 HARD hits (matches the
  IN-03 expectation). No Cargo.toml/lockfile/deny.toml change.

**Deviations:** WR-06 used the review-allowed partial-prefix carry (keeps exact
timing boundaries) instead of whole-body re-slicing; WR-01 left `GATE FAIL`
bare per the review snippet's scope; IN-03 wording notes the SOFT scan is not
reached when HARD fails (observed). The five-run repetition remains
executor-reported (raw logs not on disk) — marked as such everywhere quoted.

## Post-review fixes (2026-11-23, iteration 2 — re-review)

The iteration-2 re-review (`01-REVIEW.md`, 2026-11-23T15:16:30Z) found 12 of the
13 iteration-1 findings genuinely fixed, **WR-02 regressed**, and four new Info
residuals. In scope: WR-02 regression + IN-08, IN-09, IN-10; IN-07 deferred.
**No commits** (dispatch override; the orchestrator commits after re-review
PASS). Details: `01-REVIEW-FIX.md`; disposition: `01-REVIEW-DISPOSITION.md`.

**What changed:**
- `scripts/test-gate.sh` — WR-02 regression: the refresh writer computes the
  working-tree state into `tree_state` **before** the
  `{ … } >"$EXPECTED_FILE"` group opens (the redirect truncates the file first,
  so the old inline check always saw its own write and `# tree: clean` was
  unreachable). Harness in a throwaway git repo, using a copy of the real script
  with stubbed cargo/disk-guard: committed clean tree → `# tree: clean`;
  untracked file added → `# tree: dirty (uncommitted changes present at refresh
  time)`. IN-10: with no `--log-dir`, the exit trap now removes the mktemp dir
  only on exit 0; a red run keeps the step logs and prints the directory.
  Harness: the failing run's printed `step1-tests.log` exists afterwards; a
  green run's temp dir is gone; exit codes unchanged.
- `scripts/evidence.py` — IN-08: `finish` refuses `--result ok` (exit 2) naming
  every recorded artifact missing from disk; for `fail`/`skipped` the missing
  artifact is dropped from the report with a stderr note, so no result mode can
  keep a stale digest. IN-09: the cast timing guard now requires at least one
  **positive-length** pair over a non-empty body (all non-positive pairs →
  exit 2), closing the last silent single-event t=0 path; docstring aligned.
- `docs/evidence/README.md` — IN-08 finish semantics documented.
- `KNOWN-ISSUES.md` — IN-07 deferral ledger line.
- `docs/evidence/01-truth-baseline/test-gate/notes.md` — run #4 recorded; the
  expected-file provenance now names commit `c159a3e7` and the dirty marker;
  the WR-02 caveat is rewritten as resolved.
- `scripts/test-gate.expected` — re-seeded by the real post-fix
  `--refresh-expected` run #4 (the only sanctioned writer): header commit
  `c159a3e7`, `# tree: dirty (uncommitted changes present at refresh time)`
  (the tree carried the uncommitted review fixes), 2026-11-23T16:08:34Z; numbers
  **unchanged** at 308/219/89 and the diff is header-only. The expected file is
  not a hashed artifact, so the frozen `report.json`/cast/log are untouched.

**Real full gate (run #4, post-fix writer, keys unset, under flock):**
`env -u DEEPSEEK_API_KEY -u DEEPSEEK_BUILD_API_KEY -u XAI_API_KEY
scripts/test-gate.sh --refresh-expected --log-dir /tmp/opencode/gate-refresh-postfix`
→ exit 0, `results=308 (raw 309) binaries=219 doctests=89`, both equality
checks PASS, step 3 leak scan PASS, `GATE PASS`; step 1 **31,913 passed / 0
failed / 386 ignored**, step 2 **6,201 / 0 / 4** — identical to the frozen
evidence.

**Verification (iteration 2, main checkout):** `bash -n` + `shellcheck
--shell=sh` both shell scripts clean; `ast.parse` evidence.py clean; IN-08/IN-09
temp-dir smoke 16/16; WR-02/IN-10 harness 12/12; 8-key schema 3/3; sha256 of the
4 frozen artifacts 4/4 MATCH after all edits; `scripts/key-leak-check.sh --scan
docs/evidence` PASS; `/home/[a-z]` and `sk-` greps no hits; `test-gate.expected`
diff header-only (numbers unchanged); `git diff` only the intended files.

**Deferred:** IN-07 (default no-arg `key-leak-check.sh` fail-open on grep
rc≥2) — pre-existing and frozen by plan 01-05 task 3(b); the `--scan` mode (the
gate step) fails closed via CR-01. Recorded as `deferred` in
`01-REVIEW-DISPOSITION.md` and in `KNOWN-ISSUES.md`.

## Determinism correction (2026-11-23, 01-06)

The five-run repetition above was falsified. The phase verification ran the
phase's own command (`cargo test -p xai-grok-workspace --lib --
--test-threads=4`) 35 times on `1308903a` (this report's
`restore_fetch_tests.rs` is identical at `98c2f1b8`, sha256 `62a2656a…`) and
caught **2 failures in 35** — the captured one is
`restore_fetch::tests::fetch_timeout_sigkills_term_immune_grandchild`
("grandchild <pid> must not remain live", `restore_fetch_tests.rs:731`): the
test read `/proc/<pid>` immediately after teardown, while SIGKILL delivery and
reaping are asynchronous. "Zero failures" in five runs was a lucky sample.

Plan 01-06 fixed the observation, not the assertion: `wait_until_dead(pid,
deadline)` polls every 20 ms and fails at a 5 s deadline, applied to all five
process-death assertion sites in `restore_fetch_tests.rs`; the prompt-return
bounds are still asserted before any polling, no assertion direction changed,
no band moved, `restore_fetch.rs` is untouched, and a Linux-only test pins the
helper's bound. No test is quarantined.

Post-fix evidence: 20/20 exact-test runs and **35/35 full-crate runs green**
(each `1695 passed; 0 failed`), recorded as the plan-06 evidence item
`docs/evidence/01-truth-baseline/determinism/` — `determinism.cast` + 8-key
`report.json` (`result: ok`) + `repetition-35-runs.log` (one headerless
projection line per run, every line `rc=0 named=ok test result: ok.`) +
`notes.md`. This section corrects D3 above; the rest of the report's history is
unchanged.

---
*Phase: 01-truth-baseline*
*Completed: 2026-11-23*