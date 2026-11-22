---
phase: 01-truth-baseline
plan: 05
reviewed: 2026-11-22T05:13:34Z
depth: deep
files_reviewed: 26
files_reviewed_list:
  - scripts/test-gate.sh
  - scripts/test-gate.expected
  - scripts/evidence.py
  - scripts/key-leak-check.sh
  - scripts/README.md
  - docs/evidence/README.md
  - docs/evidence/01-truth-baseline/ipv6-report/report.json
  - docs/evidence/01-truth-baseline/ipv6-report/notes.md
  - docs/evidence/01-truth-baseline/ipv6-report/ipv6-report.cast
  - docs/evidence/01-truth-baseline/ipv6-report/ipv4-refused-blocked-line-names-203-0-113-9.log
  - docs/evidence/01-truth-baseline/ipv6-report/ipv6-allowlist-resolves-ipv6-permits-it-and-matches-the-v4-mapped-spelling.log
  - docs/evidence/01-truth-baseline/ipv6-report/ipv6-loopback-1-and-ffff-127-0-0-1-permitted.log
  - docs/evidence/01-truth-baseline/ipv6-report/ipv6-refused-blocked-line-names-2001-db8-1.log
  - docs/evidence/01-truth-baseline/ipv6-report/ipv6-refused-no-non-ip-placeholder.log
  - docs/evidence/01-truth-baseline/ipv6-report/script-sandbox-run-sh-runs-end-to-end-bin-bin-echo.log
  - docs/evidence/01-truth-baseline/test-gate/test-gate.log
  - docs/evidence/01-truth-baseline/test-gate/report.json
  - docs/evidence/01-truth-baseline/test-gate/notes.md
  - docs/evidence/01-truth-baseline/cost-estimate/report.json
  - docs/evidence/01-truth-baseline/cost-estimate/notes.md
  - crates/codegen/xai-grok-shell/src/agent/folder_trust.rs
  - crates/codegen/xai-grok-shell/src/util/config/consent_tests.rs
  - crates/codegen/xai-grok-workspace/src/session/git_gate_tests.rs
  - KNOWN-ISSUES.md
  - TESTING.md
  - .planning/phases/01-truth-baseline/01-05-SUMMARY.md
findings:
  critical: 0
  warning: 6
  info: 6
  total: 12
status: issues_found
verdict: CHANGES_REQUIRED
---

# Phase 01 Plan 05: Code Review Report (Test gate and evidence)

**Reviewed:** 2026-11-22T05:13:34Z
**Base:** `882ad0de` → `67e18bde` (01-05 commits `b0516ef1`/`866ca00a`/`7fb772ba`/`84e83f10`/`67e18bde`)
**Depth:** deep, **static only** — the owner deferred all cargo/gate/shellcheck runs on 2026-11-22; nothing below was executed as a build or test.
**Status:** issues_found — no critical/security blocker, but three truth-baseline defects must be fixed before the phase closes.

## Summary

The gate's *shape* is right and it cannot go green purely by reading the log tail. I re-derived the recorded log by hand: `test-gate.log` contains 219 `     Running ` headers, 89 `   Doc-tests ` headers, and 309 `test result:` lines; the script's block awk yields `results=308`, and `219 + 89 == 308`, so the recorded numbers are internally consistent and the nested self-re-exec line is genuinely excluded. The five evidence/`<REPO>` normalisations hold (no `/home`, `/Users`, `/root`, `/workspace`, or `sk-[A-Za-z0-9]{20,}` string survives anywhere under `docs/evidence`), the ipv6 cast and step logs decode to the six committed `PASS` lines, and both artifact sha256 values match their `report.json` entries. `evidence.py` writes exactly the eight locked keys and refuses `--result ok` over a failed step; `key-leak-check.sh --scan` is additive and prints filenames only; the determinism edits widen waits in one direction and change no assertion.

However, this plan's own contract is "the gate cannot be green by accident" and "the expected count only moves on a green run." Three defects undercut those claims: (1) the completeness arithmetic does not actually require each target block to have emitted a `test result:` line (the `got` flag is dead except for the final block); (2) `--refresh-expected` writes the expected file *before* the leak-scan step, so a red (exit 1) run can still rewrite the count, contradicting the plan's "a red run must never write the file"; and (3) `scripts/test-gate.expected` records commit `882ad0de`, a commit that does **not** contain `scripts/test-gate.sh`, so the count is not reproducible at the named commit. Separately, the plan SUMMARY makes a factual claim ("no repetition was run", "every test … DEFERRED") that is directly contradicted by `.continue-here.md` ("5/5 … green (1694 passed)") and by KNOWN-ISSUES/TESTING. For a phase whose deliverable is an honest baseline, those are must-fix.

The two `result: skipped` evidence items are honest: zero steps, no cast, and notes that state no run was performed. `ipv6-report` is genuine (six passing steps + matching cast hash), earned 2026-11-21 before the owner's deferral.

I had to exclude concurrent, out-of-scope working-tree edits that appeared during the review (`.github/workflows/release.yml`, several `Cargo.toml`, `headless.rs`, user-guide pages, a new `.planning/phases/02-loop-item-verification/`). They are not part of 01-05 and were not reviewed.

## Warnings

### WR-01: SUMMARY claims no repetition ran, but `.continue-here.md`, KNOWN-ISSUES and TESTING record five green runs

**File:** `.planning/phases/01-truth-baseline/01-05-SUMMARY.md:107-109, 140` vs `.planning/phases/01-truth-baseline/.continue-here.md:46`, `KNOWN-ISSUES.md:79-81`, `TESTING.md:151`
**Issue:** The SUMMARY says "The five-run repetition … are **DEFERRED** … **no repetition was run**" and the requirement table says "five-run repetition **DEFERRED**". But the committed `.continue-here.md` (written at pause commit `7fb772ba`) says "**5/5** `xai-grok-workspace --lib` runs green (1694 passed)", and both `KNOWN-ISSUES.md` and `TESTING.md` state "five consecutive `--lib -- --test-threads=4` runs each reported `1694 passed; 0 failed`". One of the two sets of artifacts is false. `test-gate.log:1934` proves at least *one* run had `1694 passed`, so the response's premise that nothing ran is doubtful; regardless, a truth-baseline phase cannot ship two committed artifacts that contradict each other about whether a test ran. The SUMMARY's blanket "Every test, gate and verification step in this plan is DEFERRED" is also inaccurate: the full gate (which seeded `test-gate.expected`), the `ipv6-report` harness, and the workspace repetition all executed on 2026-11-21, before the 2026-11-22 deferral.
**Fix:** reconcile the SUMMARY to the contemporaneous record, e.g. "Five `xai-grok-workspace --lib` runs passed (1694/0) on 2026-11-21 on the author's host (see `.continue-here.md`); they were **not re-confirmed on this host**, and the owner deferred re-running on 2026-11-22." Do not say "no repetition was run."

### WR-02: `test-gate.expected` names a commit that does not contain the gate, so the count is not reproducible there

**File:** `scripts/test-gate.expected:2` (`# commit: 882ad0de63781a7fac3c4aa371968d1cdf28f6ac`), `scripts/test-gate.sh:272`
**Issue:** `git cat-file -e 882ad0de:scripts/test-gate.sh` fails ("exists on disk, but not in 882ad0de"); the script and the test-file determinism edits were uncommitted when `--refresh-expected` ran (expected date `2026-11-21T21:37:20Z`, script first committed `b0516ef1` at `22:34Z`). `--refresh-expected` records `git rev-parse HEAD` (`test-gate.sh:272`), which on a dirty tree is the *base* commit, not the tree that produced the count. A reader who checks out `882ad0de` to reproduce `results=308` will not find the gate at all.
**Fix:** either re-derive the count on the commit that contains `test-gate.sh` + the test edits (e.g. `b0516ef1`/`84e83f10`) so the header names a reproducing commit, or record provenance that cannot mislead, e.g. `# tree: dirty at 882ad0de (gate uncommitted)` plus the tree hash.

### WR-03: `--refresh-expected` can rewrite the expected count on a red run (leak-scan failure)

**File:** `scripts/test-gate.sh:265-280` (refresh write) vs `:282-299` (leak scan, `fail=1` on rc≠0), exit at `:301-304`
**Issue:** The plan's Task-1 action states "A red run must never write the file." The refresh write happens at step 7 and is gated only on `fail != 0` *at that moment*, which covers step 1, step 2 and the derived equality — but the leak scan is step 8 and sets `fail=1` *after* the write. So `scripts/test-gate.sh --refresh-expected` on a tree whose `docs/evidence` contains a canary/key-shape will rewrite `test-gate.expected` and then exit 1. The expected count (cargo results) is arguably independent of the leak scan, but the invariant as written ("red run never writes") is violated, and a maintainer could see a rewritten expected file from a run that printed `GATE FAIL`.
**Fix:** move the refresh write to after the leak scan and gate it on the final `fail` value, e.g.:
```sh
if [ "$REFRESH" -eq 1 ] && [ "$fail" -eq 0 ]; then
    ...write...
fi
```
placed just before the final `if [ "$fail" -ne 0 ]` exit.

### WR-04: the completeness awk counts block *headers*, not blocks that emitted a `test result:` line

**File:** `scripts/test-gate.sh:182-196`
**Issue:** In the awk, a block is counted with `if (open) { total++ }` at the next header, *without testing `got`*; `got` is only consulted in `END` for the final block. Therefore any non-final `     Running `/`   Doc-tests ` block that never emits a `test result:` line is still counted, and the asserted invariant degenerates to "#block headers == binaries + doctests" rather than "#blocks with a result == binaries + doctests". This is weaker than the file's own claim ("results (top-level `test result:` lines, one per cargo target block)", `:17-19`) and than the SUMMARY/TESTING wording ("`results` counts one top-level `test result:` line per cargo target block"). Today the hole is masked because a target that prints a header but no result line implies a crash, which makes cargo exit non-zero and the gate fail on `rc1` — so the gate still cannot go green accidentally, but the arithmetic itself does not enforce the documented rule.
**Fix:** test `got` for every block:
```awk
/^     Running / || /^   Doc-tests / { if (open && got) { total++ } open = 1; got = 0; next }
/^test result:/ { got = 1 }
END { if (open && got) { total++ } print total + 0 }
```
(Verified against the committed log: no block lacks a `test result:` line, so the corrected form still yields `308` and does not move the checked-in count.)

### WR-05: the committed `test-gate` evidence is step-1 only, but is described as "the full run" that "reports results=308"

**File:** `docs/evidence/01-truth-baseline/test-gate/test-gate.log` (whole), `test-gate/notes.md:13-18`, `test-gate/report.json:9`
**Issue:** `test-gate.log` is the step-1 cargo log only: it begins with `warning:`/`Compiling` and ends with `   Doc-tests xai_workflow`; it contains no `gate step 2`, no `gate counts`, and no `GATE PASS` (grep for `^gate `/`^GATE` returns nothing). Yet `notes.md` calls it "the 2026-11-21 full run" and says "It reports `results=308`, `binaries=219`, `doctests=89`", and `report.json.command` says "`scripts/test-gate.sh` (full run)". The log does not contain those numbers (they are derivable: I computed Running=219, Doc-tests=89, raw results=309) and contains nothing about the `config-docs` step or the leak scan. Thus the committed evidence cannot substantiate the "step 2 PASS" and "leak scan PASS" claims asserted in `KNOWN-ISSUES.md`/`TESTING.md`. The report is honestly `result: skipped`, so this is not a fabricated `ok` — but the prose overstates what the artifact shows.
**Fix:** either commit the gate's own stdout (which contains the counts, `gate step 2: PASS`, and `GATE PASS`), or retitle the artifact as the step-1 log and change the notes to "the counts are derived from this log; step 2 and the leak scan were not captured in a committed artifact."

### WR-06: the documented re-run recipe cannot reproduce the committed step logs (`grep -qF`)

**File:** `docs/evidence/README.md:64-66` vs `docs/evidence/01-truth-baseline/ipv6-report/report.json:11-39`
**Issue:** The README's step recipe uses `grep -qF 'PASS: …'`, which prints nothing. `evidence.py step` captures that empty output to the `.log` and `summarize()` returns `exit 0: ` (no detail). The committed ipv6 item, however, has six single-line `.log` files containing the `PASS:` text and `detail` values of `exit 0: PASS: …`, which can only come from a non-quiet `grep -F`. So the documented recipe does not reproduce the committed artifacts; a reader following it gets empty logs and an empty `detail`.
**Fix:** drop `-q` in the README example (`grep -F 'PASS: …' <captured>`), matching what was actually recorded.

## Info

### IN-01: `cast` docstring says both framing lines are dropped; the code drops only the leading line

**File:** `scripts/evidence.py:44-48, 244-247`
**Issue:** The docstring says the `Script started`/`Script done` framing lines are dropped, but `cmd_cast` only slices off everything before the first newline. `Script done` is absent today only because `script -T` does not record it in the timing byte stream, so it falls outside the cumulative slice by luck, not by explicit handling.
**Fix:** either say "the leading `Script started` line is dropped" or strip a trailing `Script done on …` record before writing.

### IN-02: `--package` accepts a leading hyphen, so a crate name can be forwarded as a cargo flag

**File:** `scripts/test-gate.sh:83-90`
**Issue:** The guard `'' | *[!A-Za-z0-9_-]*` allows `--package -foo` (`-` is in the class); `PACKAGE_ARGS` then carries `-p -foo` and cargo fails with a confusing CLI error. No injection surface (no `eval`, args are separate words), but the check does not mean "a crate name."
**Fix:** reject a leading `-` (`case "$2" in -*) … exit 2;; esac`) before accepting.

### IN-03: `normalize_text` does a naive substring replace of `/tmp`, which can corrupt unrelated text

**File:** `scripts/evidence.py:75-84`
**Issue:** `text.replace(str(TMP_DIR), "<TMP>")` rewrites any occurrence of `/tmp`, including `/tmpfoo` or a path segment. Low risk for current captures, but a normalizer should match a path prefix boundary.
**Fix:** use a regex with a boundary, e.g. `re.sub(re.escape(actual) + r"(?=/|$)", token, text)`.

### IN-04: stray trailing `#` in the header comment

**File:** `scripts/test-gate.sh:24`
**Issue:** `#   The tail of the log is never the verdict.#` ends with a stray `#`. Cosmetic, but it is in the purpose block.
**Fix:** drop the trailing `#`.

### IN-05: `--scan` uses `grep -R`, which follows symlinks out of the scanned tree

**File:** `scripts/key-leak-check.sh:63`
**Issue:** `grep -RIlE … "$@"` follows symlinks, so a symlink under `docs/evidence` could pull files from outside the tree into the scan (and a symlinked binary is skipped by `-I`). Not a leak, but the scan is broader than "the committed tree."
**Fix:** use `grep -rIlE` (lowercase `-r`) and `--` as already done.

### IN-06: `finish --result ok` is allowed with zero steps

**File:** `scripts/evidence.py:196-205`
**Issue:** `finish` only refuses `ok` when a recorded step failed; an item with no steps at all can be finished `ok`. The current items are honest (`ipv6-report` has six steps; the two deferred items are `skipped`), but the helper would permit a zero-evidence `ok` report.
**Fix:** optionally refuse `--result ok` when `not state["steps"]` (and require at least one artifact), so an `ok` report always has earning evidence.

---

## Verification performed (static only)

All commands were read-only over the committed tree; no `cargo`, `scripts/test-gate.sh`, `shellcheck`, network, or test run was executed.

| # | Check | Result |
|---|-------|--------|
| 1 | `git diff --name-status 882ad0de..67e18bde` | 01-05 scope as listed; `restore_fetch_tests.rs` genuinely unmodified |
| 2 | `awk` block count over `test-gate.log` | `results=308`; Running headers `219`; Doc-tests `89`; raw `test result:` `309` |
| 3 | Scan for a block header with no `test result:` line | none in the committed log (so WR-04's fix keeps `308`) |
| 4 | `grep -rnE '/home/[a-z]\|/Users/\|/root/\|/workspace\|cein\|corourke' docs/evidence` | none |
| 5 | `grep -rnE 'sk-[A-Za-z0-9]{20,}' docs/evidence` | none |
| 6 | `sha256sum` of `ipv6-report.cast` and `test-gate.log` vs `report.json` | both match (`e18c86…`, `803c1f…`) |
| 7 | `python3` JSON parse of all three `report.json` | each has exactly the eight keys |
| 8 | `python3 -c 'ast.parse(…)' scripts/evidence.py`; `bash -n` on both scripts | parse/syntax OK |
| 9 | `git cat-file -e 882ad0de:scripts/test-gate.sh` | **absent** at the recorded provenance commit (WR-02) |
| 10 | grep `serial(DEEPSEEK_BUILD_HOME)` | none left in `xai-grok-shell`; all remaining are in `xai-grok-pager` (separate process), so the bare-key unification excludes the two targeted tests |
| 11 | verify cost-estimate-referenced test names/strings exist | `cost_renders_the_estimate_at_fixed_width_per_branch`, `tui_and_headless_report_the_same_cost_for_one_override`, `json_result_honours_status_line_pricing_override`, and `~$0.0001/0.0042/0.01/0.37 est.` all present |
| 12 | cast time convention vs repo writer | `pty-harness/src/lib.rs:305-306` emits `spawned_at.elapsed().as_secs_f64()` (absolute); `evidence.py`'s `running += elapsed` accumulation matches |

NOT RUN (per owner deferral and the static-only constraint): `scripts/test-gate.sh`, any `cargo test`/`cargo build`, `shellcheck`, `actionlint`, `scripts/test-sandbox-run.sh`, the leak-scan live mode. Verification of counts therefore relies on re-deriving the numbers from the committed `test-gate.log` rather than a fresh run.

## Verdict

**CHANGES_REQUIRED.** There is no critical/security blocker and no fabricated cast or `ok` report: the arithmetic of the recorded run is sound, both artifact hashes match, the skipped items are honest, and the determinism edits are one-directional. But three defects strike at what this phase exists to guarantee and are small, fixable edits that should land before the phase closes:

1. **WR-01** — the plan SUMMARY asserts a test repetition was never run, contradicting three other committed artifacts. A truth baseline cannot carry contradictory test claims.
2. **WR-03** — `--refresh-expected` writes the expected count before the leak scan, so a red (exit 1) run can still move the checked-in number, violating "a red run must never write the file."
3. **WR-02** — the expected-count provenance names a commit that does not contain the gate, so the number is not reproducible at the commit it cites.

WR-04 (the dead `got` flag in the completeness awk) and WR-05/WR-06 (evidence scope and the non-reproducing re-run recipe) should be fixed in the same pass; IN-01..IN-06 are optional cleanups. Once WR-01/WR-02/WR-03 are corrected, this is a PASS_WITH_NOTES ship.

---

_Reviewed: 2026-11-22T05:13:34Z_
_Reviewer: gsd-code-reviewer (adversarial subagent, static-only)_
_Depth: deep_

## Resolution (2026-11-22)

`CHANGES_REQUIRED` findings were fixed in `58cc8ac3`: summary truth (WR-01),
dirty-tree provenance (WR-02), leak-scan-before-refresh (WR-03), a result line
required per block (WR-04), and the evidence-doc corrections (WR-05/06).

Still open and owner-gated: **W-1** (two evidence items are `result: skipped`)
and **W-2** (the expected count is not re-confirmed on this host).
