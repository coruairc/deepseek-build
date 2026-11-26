---
phase: 04-distribution-proof
plan: 05
subsystem: distribution
tags: [evidence, seal, validator, sha256, deviation, leak-scan, rel-04, rel-05, rel-06, pkg-04]
requires:
  - plan: "04-01"
    provides: "tag-push/ and tag-run/ evidence items (REL-04 provenance and assets)"
  - plan: "04-02"
    provides: "installer-proof/ and installer-latest/ items, plus check-readme-distribution.py"
  - plan: "04-03"
    provides: "homebrew-formula/ item and the in-tree formula"
  - plan: "04-04"
    provides: "image-registry/ and image-run-blocked/ items (REL-04 registry side, REL-05 skipped)"
provides:
  - "scripts/check-distribution-evidence.py — eight-key schema + relative text-artifact + recomputed-sha256 validator with a four-case --self-test"
  - "docs/evidence/04-distribution-proof/README.md — the nine-item index with requirement ids, results, what each proves and the verbatim re-run command"
  - "docs/evidence/04-distribution-proof/DEVIATION-keep-prerelease.md — the named deviation against ROADMAP Phase 4 Success Criteria item 4 and 04-CONTEXT.md §Cleanup"
  - "docs/evidence/04-distribution-proof/seal/ — the ninth evidence item (validator output, self-test, leak scan, retention re-assertion, report)"
  - "scripts/README.md — one-line entries for the five Phase 4 scripts"
  - "honest ROADMAP/REQUIREMENTS/STATE rows: Phase 4 is 5/5 plans executed but REL-04 Partial and REL-05 blocked"
affects: [05-windows-support, 06-documentation-truth-and-freeze]

actuals:
  tokens: 12704          # chars/4 over the 12 changed files (848 lines added, 50,816 bytes); the estimate (90k) was ~7x high
  tasks: 2
  commits: 2             # measured 2db19b39..HEAD; the metadata commit carrying this file is the 3rd

tech-stack:
  added: []              # Python stdlib only — nothing installed
  patterns:
    - "A seal validates, never rewrites: a broken item is a new directory with its own report, and the validator's checks are never weakened to make an item pass"
    - "REL-05 can never drift to `ok`: the checker refuses any report that claims it without a successful step named `pull`"
    - "A report cannot hash a log that attests the report that hashes it: seal.log records the eight-report run; the nine-report run is the post-finish command"
    - "A deviation names the exact roadmap line and CONTEXT decision it contradicts, lists what is retained and the invariants, and contains no deletion command"

key-files:
  created:
    - scripts/check-distribution-evidence.py
    - docs/evidence/04-distribution-proof/README.md
    - docs/evidence/04-distribution-proof/DEVIATION-keep-prerelease.md
    - docs/evidence/04-distribution-proof/seal/seal.log
    - docs/evidence/04-distribution-proof/seal/retention-check.sh
    - docs/evidence/04-distribution-proof/seal/notes.md
    - docs/evidence/04-distribution-proof/seal/report.json
    - docs/evidence/04-distribution-proof/seal/validator-self-test-4-4-mutations-on-a-temp-copy.log
    - docs/evidence/04-distribution-proof/seal/validator-eight-produced-reports-schema-and-hashes.log
    - docs/evidence/04-distribution-proof/seal/leak-scan-docs-evidence-and-the-phase-planning-directory.log
    - docs/evidence/04-distribution-proof/seal/retention-no-latest-no-v0-1-0-snapshot-and-deepseek-untouched.log
  modified:
    - scripts/README.md
    - .planning/ROADMAP.md
    - .planning/REQUIREMENTS.md
    - .planning/STATE.md

key-decisions:
  - "The seal item's `requirements` are REL-04, REL-06, PKG-04 — not REL-05. The plan's action text said `--requirements REL-04,REL-05,REL-06,PKG-04`, but the plan's own invariant forbids `ok` for any item claiming REL-05 without a successful pull step. The invariant wins: the seal validates the evidence for REL-04/06/PKG-04 and records REL-05 as open."
  - "The validator refuses to run while any `.evidence-pending.json` exists, so the seal's own validator step runs against a temp copy of the eight produced `report.json` files (artifact paths still resolve against the repo root). The post-finish run validates all nine, including the seal."
  - "The seal's fourth step (`seal/retention-check.sh`) re-asserts the deviation's invariants read-only over the network, because the dispatch required the `:latest` / `v0.1.0` / snapshot confirmation with commands and output; it mutates nothing."
  - "Phase 4 is recorded as 5/5 plans executed with two requirements partially/blocked. The ROADMAP checkbox stays unticked and the progress row says `Executed — REL-04 Partial (pull not executed), REL-05 blocked`."

patterns-established:
  - "A phase seal is an evidence item: its report hashes the validator log, the index, the deviation and its own notes, so the seal is validated by the same rule it enforces."
  - "Every `report.json.command` in the index is quoted verbatim, so the index cannot drift from the record it points at."

requirements-completed: []   # deliberately empty — REL-04 stays Partial (pull not executed) and REL-05 blocked; REL-06/PKG-04 were closed by 04-02/04-03 and are only re-validated here

coverage:
  - id: D1
    description: "Mechanical validator over every Phase-4 evidence report: eight-key order, item/date/host/result shapes, relative UTF-8 text artifacts <= 1 MiB with recomputed sha256, and the REL-05 no-false-pass rule"
    verification:
      - kind: other
        ref: "python3 scripts/check-distribution-evidence.py docs/evidence/04-distribution-proof --self-test (self-test 4/4 PASS; 9/9 PASS, 9 reports validated)"
        status: pass
    human_judgment: false
  - id: D2
    description: "Nine-item evidence index with requirement ids, results, one sentence per item and the verbatim re-run command from report.json"
    verification:
      - kind: other
        ref: "docs/evidence/04-distribution-proof/README.md is a hashed artifact of seal/report.json and the validator passes it"
        status: pass
    human_judgment: false
  - id: D3
    description: "The keep-the-prerelease deviation named against ROADMAP Phase 4 Success Criteria item 4 and 04-CONTEXT.md §Cleanup, with the retained refs, the reason, the invariants and no deletion command"
    verification:
      - kind: other
        ref: "grep -c 'Success Criteria' DEVIATION-keep-prerelease.md == 1; grep -c 'v0.1.0-proof1' == 4; no gh release delete / tag deletion command present"
        status: pass
    human_judgment: true
    rationale: "Whether the recorded reason and the named roadmap line are the right ones is a reading judgment; automation only proves the strings and the absence of a deletion command."
  - id: D4
    description: "Leak scans over the phase evidence, docs/evidence + the phase planning directory, and all of docs/evidence"
    verification:
      - kind: other
        ref: "scripts/key-leak-check.sh --scan docs/evidence/04-distribution-proof (exit 0); --scan docs/evidence .planning/phases/04-distribution-proof (exit 0); --scan docs/evidence (exit 0)"
        status: pass
    human_judgment: false
  - id: D5
    description: "Honest ROADMAP/REQUIREMENTS/STATE updates: Phase 4 5/5 plans executed, REL-04 Partial and REL-05 blocked named in the row/notes; no cleanup claimed"
    verification:
      - kind: other
        ref: "grep the ROADMAP Phase 4 plans line and progress row, REQUIREMENTS REL-04/REL-05 rows, STATE current position (read, not executed)"
        status: pass
    human_judgment: true
    rationale: "Honesty of a status record is a reading judgment; the commands listed prove the strings exist, not that they tell the whole truth."

duration: 13min
completed: 2026-11-26
status: complete
---

# Phase 4 Plan 5: Evidence Sealed, Deviation Recorded, Leak Scan Green Summary

**Nine distribution-evidence items validated by schema and recomputed sha256 (self-test 4/4, 9/9 PASS), the keep-the-prerelease decision named against the roadmap's cleanup line, and a clean leak scan across the tree**

## Performance

- **Duration:** 13 min
- **Started:** 2026-11-26T06:55:00Z
- **Completed:** 2026-11-26T07:08:00Z
- **Tasks:** 2
- **Files modified:** 12 (11 created, 1 modified in the task commits; four planning files updated separately)

## Accomplishments

- `scripts/check-distribution-evidence.py` re-reads all nine `report.json` files: exact eight-key order, `item` == directory, requirement subset, date/host/result shapes, `ok` implies non-empty all-ok steps, and every artifact is repository-relative UTF-8 text ≤ 1 MiB whose recorded sha256 equals the file's current hash. The `--self-test` stages a temp copy and rejects a deleted artifact, a corrupted hash, `image-run-blocked` flipped to `ok`, and a NUL byte (`self-test: 4/4 PASS`).
- The real tree validates: `distribution-evidence: 8/8 PASS` at seal time, `9/9 PASS` / `9 reports validated` after the `seal` report itself was written. Exactly one item is `skipped` (`image-run-blocked`, REL-05), and the checker refuses any item that claims REL-05 as `ok` without a successful `pull` step.
- `docs/evidence/04-distribution-proof/README.md` is the nine-item index: requirement ids, result, one sentence per item, and the verbatim re-run command from that item's `report.json`, plus the deviation link and the aggregate outcome (REL-04 registry-proven with the pull deferred, REL-05 open, REL-06 closed, PKG-04 closed for the formula plus the documented activation).
- `DEVIATION-keep-prerelease.md` names `ROADMAP.md` → `### Phase 4: Distribution Proof` → `Success Criteria` item 4 and `04-CONTEXT.md` §Cleanup / §Decided autonomously; it records the retained tag `v0.1.0-proof1`, its six-asset pre-release and its two GHCR tags as the milestone's only macOS artifact, and the invariants (v0.1.0 unpublished, no stable release, `:latest` absent before and after). It contains no deletion command.
- Leak scans exit 0 three ways: the phase evidence directory, `docs/evidence` + the phase planning directory, and all of `docs/evidence`. The final post-seal scans also exit 0.
- `seal/retention-check.sh` re-asserted read-only that the GHCR tags are exactly `0.1.0-proof1` and `v0.1.0-proof1` with `:latest` absent, no `v0.1.0` tag or release exists, `releases/latest` is 404, and the `deepseek` branch and `deepseek-v0.1-snapshot` tag are still `4b305bd0`.
- `scripts/README.md` gained exactly five one-line entries (`distribution-proof.py`, `prove-install.sh`, `check-readme-distribution.py`, `check-homebrew-formula.py`, `check-distribution-evidence.py`) with no existing line reflowed.
- The status records are honest: `ROADMAP.md` shows 5/5 plans executed with REL-04 Partial and REL-05 blocked and keeps the phase checkbox unticked; `REQUIREMENTS.md` keeps REL-04/REL-05 unchecked and annotates them; `STATE.md` records 21/21 plans executed but a phase still open.

## Task Commits

Each task was committed atomically:

1. **Task 1: Validate the eight evidence items and index them with re-run commands** - `aefcd13c` (feat)
2. **Task 2: Record the keep-prerelease deviation and seal the evidence** - `13002b2f` (feat)

**Plan metadata:** the docs commit carrying this SUMMARY, STATE.md, ROADMAP.md and REQUIREMENTS.md (3rd commit of the plan)

## Files Created/Modified

- `scripts/check-distribution-evidence.py` — schema/hash/text validator with a four-case self-test; refuses a mid-write tree
- `docs/evidence/04-distribution-proof/README.md` — the nine-item index with re-run commands
- `docs/evidence/04-distribution-proof/DEVIATION-keep-prerelease.md` — the named cleanup deviation
- `docs/evidence/04-distribution-proof/seal/seal.log` — validator output over the eight produced reports
- `docs/evidence/04-distribution-proof/seal/retention-check.sh` — read-only no-`:latest`/no-`v0.1.0`/snapshot check
- `docs/evidence/04-distribution-proof/seal/notes.md` — validator counts, skipped item, leak scans, aggregate status
- `docs/evidence/04-distribution-proof/seal/report.json` — the ninth item (`result: ok`)
- `docs/evidence/04-distribution-proof/seal/*.log` — the four recorded steps
- `scripts/README.md` — five new one-line entries
- `.planning/ROADMAP.md`, `.planning/REQUIREMENTS.md`, `.planning/STATE.md` — honest 5/5-executed status

## Decisions Made

- **The seal does not claim REL-05.** The plan's action said `--requirements REL-04,REL-05,REL-06,PKG-04`, but its own must-have forbids `ok` for any item claiming REL-05 without a successful pull step. Recorded `REL-04,REL-06,PKG-04`; REL-05 is recorded as open in the notes and index. This is the one place the plan was internally inconsistent.
- **`seal.log` records the eight-report run, not nine.** A report cannot hash a log that attests the report that hashes it; the nine-report run is the post-`finish` verification command, documented in `seal/notes.md`.
- **The seal's validator step runs on a temp copy.** The checker refuses a tree with any `.evidence-pending.json`, which necessarily exists while the seal item is being recorded; the step copies the eight `report.json` files to a temp phase dir and validates there with artifact paths resolving against the repo root.
- **Retention is re-asserted, not assumed.** The dispatch required the `:latest`/`v0.1.0`/snapshot confirmation with commands and output, so `seal/retention-check.sh` performs read-only registry/ref/release reads as the seal's fourth step.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Seal requirements could not include REL-05**
- **Found during:** Task 2 (Record the deviation by name, scan for leaks, finish the seal)
- **Issue:** The plan's Task 2 action records the seal with `--requirements REL-04,REL-05,REL-06,PKG-04` and `finish --result ok`, while the plan's own truth says the checker must fail any item that claims REL-05 with `result: ok` unless a successful `pull` step is recorded. Both cannot hold; the seal has no pull.
- **Fix:** Recorded the seal with `REL-04,REL-06,PKG-04` (REL-05 stays open and is named in `notes.md` and the index). This is the honest resolution: adding REL-05 would either fail the checker or force weakening the invariant the seal exists to enforce.
- **Files modified:** `docs/evidence/04-distribution-proof/seal/report.json`, `docs/evidence/04-distribution-proof/seal/notes.md`, `docs/evidence/04-distribution-proof/README.md`
- **Verification:** nine reports validate; the self-test proves the REL-05 mutation is rejected
- **Committed in:** `13002b2f`

**2. [Rule 3 - Blocking] The validator cannot run over the real tree while the seal item is pending**
- **Found during:** Task 2 (recording the validator step)
- **Issue:** `scripts/check-distribution-evidence.py` refuses any `.evidence-pending.json` (the plan's own mid-write rule), but `scripts/evidence.py` only records a step while an item is pending. A validator step over the real tree would therefore always be refused.
- **Fix:** The recorded step validates a temp copy of the eight produced `report.json` files (artifacts still resolve against the repo root); the real tree is validated after `finish` (9/9), and that later run is a verification command, not a step. The checker's pending refusal is unchanged.
- **Files modified:** `docs/evidence/04-distribution-proof/seal/validator-eight-produced-reports-schema-and-hashes.log`
- **Verification:** the step exits 0 with `8/8 PASS`; the post-finish run is `9/9 PASS`
- **Committed in:** `13002b2f`

**3. [Rule 2 - Missing Critical] The deviation's invariants are re-asserted against the live refs and registry**
- **Found during:** Task 2 (write the deviation)
- **Issue:** The plan recorded the retention invariants only as prose and cited earlier evidence; the dispatch required confirming no `:latest`, no `v0.1.0`, and untouched `deepseek`/`deepseek-v0.1-snapshot` with the commands and output.
- **Fix:** Added `seal/retention-check.sh`, a read-only check (anonymous registry baseline, `git ls-remote`, `gh api` GETs) recorded as the seal's fourth step and hashed as an artifact.
- **Files modified:** `docs/evidence/04-distribution-proof/seal/retention-check.sh`, `.../seal/report.json`
- **Verification:** the script exits 0; its output is committed as `retention-no-latest-no-v0-1-0-snapshot-and-deepseek-untouched.log`
- **Committed in:** `13002b2f`

### Plan-vs-dispatch note

The plan's success criteria say "No commit made; the working tree is left for the reviewer", but the execution dispatch explicitly required atomic commits with `(04-05):` subjects followed by this SUMMARY. The dispatch governs execution mechanics; two task commits plus the metadata commit were made, and every plan verification was run before each commit.

---

**Total deviations:** 3 auto-fixed (1 bug/contradiction, 1 blocking, 1 missing critical)
**Impact on plan:** All three keep the seal honest and mechanically checkable; no scope creep beyond the dispatch's required retention confirmation.

## Issues Encountered

- The initial task-1 index row for `seal` was written before the seal report existed; it was written with its final content and re-validated as a hashed artifact after `finish`. No correction was needed.
- The repository already had an untracked `.planning/phases/05-windows-support/05-SPIKE-LOG.md` unrelated to this plan; it was left untouched.

## Known Stubs

None. REL-05 is not a stub: it is an explicitly recorded environment block (`result: skipped`) with exact commands, and it remains open by design.

## User Setup Required

None for this plan. Closing REL-04's pull / REL-05 still requires a host with a container runtime (`sudo systemctl enable --now docker`), as recorded in `image-run-blocked/notes.md`; `brew install` remains an author step on macOS.

## Next Phase Readiness

- Phase 5 (Windows Support) can plan from this sealed evidence; the evidence convention and the validator carry forward.
- Phase 4 remains open: REL-04 is Partial (the pull did not happen) and REL-05 is blocked (no daemon). ROADMAP Phase 4 criteria 1 and 2 are unmet, and the phase is recorded as 5/5 plans executed, not complete.

## Self-Check: PASSED

- `docs/evidence/04-distribution-proof/seal/report.json` exists and is the ninth validated report
- `aefcd13c` and `13002b2f` are ancestors of HEAD
- `python3 scripts/check-distribution-evidence.py docs/evidence/04-distribution-proof` exits 0 with `9/9 PASS`

---
*Phase: 04-distribution-proof*
*Completed: 2026-11-26*
