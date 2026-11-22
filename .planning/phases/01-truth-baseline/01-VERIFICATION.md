---
phase: 01-truth-baseline
verified: 2026-11-22T05:20:00Z
status: deferred
verification_type: goal-backward
score: "5/5 code present; 2/5 execution proofs deferred"
requirements_verified: [SBOX-01, SBOX-02, COST-01, COST-02, COST-03, DEF-01, EVID-02]
requirements_deferred: [TEST-01, TEST-02, TEST-03, EVID-01]
---

# Phase 1 Verification — Truth Baseline

**Verdict: DEFERRED (tests skipped by owner, 2026-11-22).**

The owner directed on 2026-11-22 that no test, gate, smoke or egress run be
performed for the remaining milestone plans. This report therefore separates
**what is present in the tree and was verified by reading the committed code**
from **what needs an execution run**. Nothing below is inferred from an
unexecuted run.

## Goal-backward check

**Goal:** the sandbox stops aborting on this host, the cost figure and the
default model stop lying, and every claim carries a committed artifact, so a
green suite actually means something.

| # | Success criterion | State | Basis |
|---|---|---|---|
| 1 | `deny_paths_e2e` passes unmodified; non-traversable parent skips; traversable-but-unreadable still errors; `sandbox-run.sh` reports IPv6 | **Code present, execution DEFERRED** | `0bf46c84` (01-01), `882ad0de` (01-02); `runtime_sockets_tests.rs` fixture matrix; `scripts/egress_guard.c` |
| 2 | Workspace suite green and completeness-counted; `config-docs` a named step; flake deterministic or quarantined | **Gate code present, run DEFERRED** | `scripts/test-gate.sh`, `scripts/test-gate.expected`; 2026-11-21 run #2 is the last recorded green (308/219/89) |
| 3 | `docs/evidence/` holds a cast + normalized `report.json` per item, convention documented | **Partially met** | `ipv6-report` complete; `test-gate` and `cost-estimate` are `result: skipped` per the owner deferral; convention complete in `docs/evidence/README.md` |
| 4 | Flash prices at Flash rates; TUI figure labelled an estimate; headless honours `[ui.status_line.pricing]` | **Code present, run DEFERRED** | `e834f647` (01-03); `pricing.rs`, `segments.rs::format_cost`, `headless.rs` pricing table |
| 5 | `deepseek-flash` is the catalog default with the at-ceiling caveat and revert path documented | **Code present, run DEFERRED** | `6d3cfcb9` (01-04); `default_models.json`, `resolution.rs`, README + user guide + `docs/TUNING.md` |

## Requirement status

| Req | Status | Where |
|---|---|---|
| SBOX-01 | code present (run deferred) | `xai-grok-sandbox` endpoint classifier + traversability probe (`0bf46c84`) |
| SBOX-02 | code present (run deferred) | `deny_paths_e2e` fixture + `scripts/egress_guard.c` IPv6 naming (`882ad0de`) |
| TEST-01 | DEFERRED | `scripts/test-gate.sh` complete; confirmation run not executed |
| TEST-02 | DEFERRED | `config-docs` is gate step 2; not executed |
| TEST-03 | DEFERRED | serial key unified, waits widened; five-run repetition not executed |
| EVID-01 | partially met | `ipv6-report` complete; two items `result: skipped` |
| EVID-02 | met | `docs/evidence/README.md` + per-item `notes.md` re-run recipes |
| COST-01 | code present (run deferred) | `xai-grok-status-line/src/pricing.rs` Flash row |
| COST-02 | code present (run deferred) | `views/status_line/segments.rs` `~$… est.` |
| COST-03 | code present (run deferred) | `headless.rs` resolves the status-line pricing table |
| DEF-01 | code present (run deferred) | `default_models.json` + `resolution.rs` + four doc surfaces |

## Deferred verification commands

Run these when testing resumes, in this order:

```sh
scripts/test-gate.sh                    # expect GATE PASS, results == 308
cargo test -p xai-grok-sandbox --test deny_paths_e2e
sh scripts/test-sandbox-run.sh         # expect ALL PASS (6 assertions)
cargo test -p xai-grok-pager --lib status_line
cargo test -p xai-grok-pager --lib headless
scripts/key-leak-check.sh --scan docs/evidence
```

## Risks carried forward

- The expected count (`308`) was derived on the author's host; this host was
  never confirmed. A different count must be re-derived with
  `--refresh-expected` on a green run, never hand-edited.
- Two evidence items are placeholders by owner decision; a release claim needs
  them upgraded.
