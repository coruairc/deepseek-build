---
phase: 01-truth-baseline
reviewed: 2026-11-23T20:06:38Z
depth: standard
files_reviewed: 22
files_reviewed_list:
  - crates/codegen/xai-grok-workspace/src/restore_fetch_tests.rs
  - crates/codegen/xai-grok-workspace/src/restore_fetch.rs
  - docs/evidence/01-truth-baseline/determinism/report.json
  - docs/evidence/01-truth-baseline/determinism/notes.md
  - docs/evidence/01-truth-baseline/determinism/determinism.cast
  - docs/evidence/01-truth-baseline/determinism/repetition-35-runs.log
  - docs/evidence/01-truth-baseline/determinism/post-fix-repetition-35-full-crate-runs.log
  - docs/evidence/01-truth-baseline/determinism/targeted-repetition-20-named-test-runs.log
  - docs/evidence/01-truth-baseline/determinism/all-35-runs-exited-zero.log
  - docs/evidence/01-truth-baseline/determinism/every-run-s-summary-is-ok-with-zero-failed.log
  - docs/evidence/01-truth-baseline/determinism/every-run-exercised-the-named-test.log
  - docs/evidence/01-truth-baseline/determinism/all-runs-report-one-passed-count.log
  - docs/evidence/01-truth-baseline/determinism/leak-scan-over-docs-evidence.log
  - docs/evidence/README.md
  - KNOWN-ISSUES.md
  - TESTING.md
  - .planning/phases/01-truth-baseline/01-05-SUMMARY.md
  - .planning/phases/01-truth-baseline/01-06-SUMMARY.md
  - .planning/phases/01-truth-baseline/01-06-PLAN.md
  - .planning/phases/01-truth-baseline/01-VERIFICATION.md
  - scripts/evidence.py
  - docs/evidence/01-truth-baseline/test-gate/notes.md
findings:
  critical: 0
  warning: 1
  info: 4
  total: 5
status: issues_found
---

# Phase 1 Plan 06: Code Review Report (TEST-03 gap closure)

**Reviewed:** 2026-11-23T20:06:38Z
**SHA:** `98c2f1b8` (all 01-06 work uncommitted: `git diff HEAD` + the untracked `docs/evidence/01-truth-baseline/determinism/`)
**Depth:** standard
**Files Reviewed:** 22
**Status:** issues_found
**Merge gate: PASS_WITH_NOTES** — 0 blockers. The one warning is a documentation-wording correction (WR-01); it does not weaken the fix or the evidence and need not hold the commit. The orchestrator may commit this delta.

## Summary

No structural pre-pass was provided; every finding below is narrative. Scope is the 01-06 gap-closure delta only.

### What was verified (this review, not quoted from the executor)

| Check | Result |
|---|---|
| `git diff HEAD -- restore_fetch_tests.rs` | Five process-death sites now call `wait_until_dead`; `pid_alive` byte-identical; all five original messages verbatim; the `< 2 s` / `< 5 s` prompt-return bounds still evaluated **before** any polling; no `assert_timeout_about` band touched; no `#[ignore]` in the file |
| `git show HEAD:…/restore_fetch.rs \| sha256sum` | `7bc753de…` — production file unchanged |
| `flock … cargo test -p xai-grok-workspace --lib -- restore_fetch::tests` (keys unset) | **31 passed / 0 failed** (2.40 s), including `wait_until_dead_is_bounded_and_deadline_respecting` and `fetch_timeout_sigkills_term_immune_grandchild`; 1695 total tests confirmed by the filtered count |
| `cargo fmt --all --check`, `git diff HEAD --check` | exit 0, clean |
| `report.json` | exactly 8 keys, `requirements: ["TEST-03"]`, 7/7 steps `ok`, `result: ok` |
| Artifact sha256 | 4/4 match the files on disk |
| `repetition-35-runs.log` | 35 lines, run numbers 01–35 all distinct, 35 `rc=0`, 35 `named=ok`, 35 `test result: ok.`, one distinct passed count (`1695 passed;`) |
| `determinism.cast` | valid asciinema v2 (80×24, 3 `[elapsed,"o",str]` events, non-decreasing time); all 35 projection lines present verbatim in the event data |
| `scripts/key-leak-check.sh --scan docs/evidence` | `PASS … docs/evidence`, exit 0; no `/home/[a-z]` hit in the item |
| Frozen files | `scripts/test-gate.expected`, `Cargo.toml`, `Cargo.lock`, `deny.toml`, the three existing evidence items: no diff |
| Docs | `KNOWN-ISSUES.md`, `TESTING.md`, `01-05-SUMMARY.md` all cite the determinism item and no longer present the five-run sample as proof; the old section is retitled SUPERSEDED; D3's five-run entry is `status: fail` with a third, artifact-backed entry added |

**The fix is sound and does not weaken the test.** The poll has a hard 5 s deadline (a never-dying process still fails with the original message at the original site), the bounds test pins that a live pid is not reported dead inside a 150 ms deadline (returns `false` with elapsed ≥ 100 ms and < 2 s), and no natural-exit path can satisfy the assertions inside the deadline: the sibling `sleep 30` grandchildren cannot exit naturally in 5 s, and the TERM-immune leader/grandchild loop forever. `wait_until_dead` also checks `pid_alive` before the deadline return, so a process that dies at the deadline is still accepted (no off-by-one that turns an on-time death into a failure).

**Independent proof that the 35-run evidence is genuine.** The uncommitted raw per-run logs are still on disk at `/tmp/opencode/det-01-06/determinism-raw/`. I cross-checked them myself: 35 files, 35 distinct sha256, 35 distinct `Finished \`test\` profile … in` values (44.29–48.37 s), each raw log contains `test restore_fetch::tests::fetch_timeout_sigkills_term_immune_grandchild ... ok` and `test result: ok. 1695 passed; 0 failed`, and every one of the 35 committed projection lines matches its raw log's `test result:` line exactly (0 mismatches); the mtimes span 29.6 minutes. The evidence is a real repetition, not a fabricated projection log. The residual design limitation (raw logs are not committed) is IN-01 and is inherent to the plan's chosen shape.

**Working-tree caution (not a 01-06 defect).** During this review the tree gained two tracked-file deletions that were not in the status at review start and are unrelated to plan 01-06: `.planning/HANDOFF.json` and `.planning/phases/01-truth-baseline/.continue-here.md` (last committed in `7fb772ba`). This review did not touch them; they look like another session's resume bookkeeping. The orchestrator should stage the 01-06 paths **by name** and decide on those deletions separately — do not sweep them into the 01-06 commit.

## Warnings

### WR-01: Three docs attribute both 35-run failures to the named test; the committed verification only identifies "the captured one"

**File:** `KNOWN-ISSUES.md:82-85`, `TESTING.md:155-159`, `.planning/phases/01-truth-baseline/01-05-SUMMARY.md:86,363-366`
**Issue:** All three say the verification's repetition "caught 2 failures in 35 **of** `restore_fetch::tests::fetch_timeout_sigkills_term_immune_grandchild`". The only committed record of that sample — `01-VERIFICATION.md:89` and `:138` — says it "produced 2 failures. **The captured one** is `restore_fetch::tests::fetch_timeout_sigkills_term_immune_grandchild`…"; the identity of the second failure is not recorded anywhere in the repository (`01-06-PLAN.md` asserts it, but that plan is the claim under review). In a phase whose acceptance rule is that every claim carries an artifact, these three sentences state more than their artifact supports. The consequence matters for TEST-03's scope: if the second failure was a *different* test, closure rests on the 0/35 post-fix sample alone, not on the cause-targeted fix, and the current wording hides that. This is not a defect in the fix or in the new evidence item.
**Fix:** Either (a) record the verifier's evidence for both failures (raw output or the second panic text) in `notes.md`/`01-VERIFICATION.md` if it exists, or (b) word it as `determinism/notes.md:9-12` already does — the model to copy:

```markdown
… the verifier's 35-run repetition of `cargo test -p xai-grok-workspace --lib
-- --test-threads=4` on `1308903a` produced **2 failures**; the captured one was
`restore_fetch::tests::fetch_timeout_sigkills_term_immune_grandchild`
(`grandchild <pid> must not remain live`, `restore_fetch_tests.rs:731`).
```

Keep the `2 in 35` number (it is correct) and the fix/evidence citation (also correct).

## Info

### IN-01: The cast corroborates duration, not per-run progress; the binding to real runs lives only in uncommitted raw logs

**File:** `docs/evidence/01-truth-baseline/determinism/determinism.cast:2-4`, `notes.md:82-101`
**Issue:** All three cast events are at elapsed `1831.558s` because `evidence.py step` buffers the child's output and writes it at the end. The cast therefore proves a ~30m31s execution and that the projection lines were emitted through `script`, but it does not show 35 cargo invocations. `repetition-35-runs.log` + sha256 are mutually consistent, yet a self-consistent fabrication would also pass that pair — the only binding to real runs is the `<TMP>` raw logs, which are not committed. For this instance I verified those raw logs exist and agree exactly (see Summary), so the evidence is genuine; the note is about a later reader.
**Fix (future repetition items):** capture the driver directly with `script -T` instead of wrapping `evidence.py step`, or have the driver print a timestamp per run, so the cast carries 35 incremental events; optionally project the raw logs' `Finished … in` line as a second durable line. No change needed to the 01-06 item.

### IN-02: Summary typo names a non-existent file

**File:** `.planning/phases/01-truth-baseline/01-06-SUMMARY.md:82`
**Issue:** coverage D2 says "`deterministic.cast` valid asciinema v2 (3 events)"; the artifact is `determinism.cast` (the same summary's key-files list has it right).
**Fix:** `s/deterministic.cast/determinism.cast/`.

### IN-03: Ambiguous antecedent in the KNOWN-ISSUES correction

**File:** `KNOWN-ISSUES.md:83`
**Issue:** "on `1308903a` (this file is identical at `98c2f1b8`)" — inside `KNOWN-ISSUES.md`, "this file" reads as `KNOWN-ISSUES.md`; the intended referent is `restore_fetch_tests.rs`. `TESTING.md:156` ("identical file also at…") is clearer.
**Fix:** "…on `1308903a` (`restore_fetch_tests.rs` is identical at `98c2f1b8`)".

### IN-04: The committed targeted-repetition log cannot show which test ran

**File:** `docs/evidence/01-truth-baseline/determinism/targeted-repetition-20-named-test-runs.log`
**Issue:** Normalization drops the per-test `test … ... ok` line, so the durable artifact is 20 × `test result: ok. 1 passed; 0 failed; … 1694 filtered out` plus `targeted run NN/20: rc=0`; the only thing binding those to the named test is the step name in `report.json`. It is not load-bearing (the 35-run log carries `named=ok` per run), but the artifact alone does not evidence its own claim.
**Fix (future items):** have the targeted driver emit a `named=ok` marker per run (as the 35-run driver does) before normalization, or keep the per-test line by not routing the evidence through the scrub for `--exact` runs.

---

_Reviewed: 2026-11-23T20:06:38Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_

## Disposition (orchestrator, 2026-11-23)

- **WR-01 applied:** KNOWN-ISSUES.md, TESTING.md, 01-05-SUMMARY.md and 01-06-SUMMARY.md now say "the captured one was …" and record that the second failure's identity was not captured, matching `determinism/notes.md` and `01-VERIFICATION.md`. The 2-in-35 count is kept.
- **IN-02:** already correct in the committed summary (`determinism.cast`); no change needed.
- **IN-03 applied:** KNOWN-ISSUES.md antecedent clarified to "`restore_fetch_tests.rs` is identical at `98c2f1b8`".
- **IN-01 / IN-04:** carried as guidance for future repetition evidence items (capture the driver directly with `script -T`; emit per-run `named=ok` markers before normalization). No change to the 01-06 item.
- **R5 portability note (non-Linux `pid_alive` and zombies):** out of scope, Linux-only releases; noted for open-build.
