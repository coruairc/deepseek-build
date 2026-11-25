---
phase: 02-loop-item-verification
plan: 03
subsystem: testing
tags: [evidence, key-hygiene, leak-scan, canary, attestation, proof-03]

# Dependency graph
requires:
  - phase: 02-loop-item-verification
    provides: "02-01/02-02's live session and egress reading; 02-04/02-05/02-06's evidence items and pty/headless conventions"
provides:
  - "docs/evidence/02-loop-item-verification/key-leak/ — report.json (result ok, PROOF-03), key-leak.cast, notes.md"
  - "the default-mode canary verdict: PASS  fake key absent from 52 saved file(s) and captured output"
  - "scoped --scan verdicts: docs/evidence and docs + .planning/phases/02-loop-item-verification both exit 0"
  - "the measured whole-tree key-shape baseline: 2 files / 4 lines / 4379 tracked files, no equality gate"
  - "the author's dated revocation attestation, with a 'recorded, not verified' statement beside it"
  - "docs/evidence/README.md 'Re-running the Phase-2 items' naming all seven items with their commands"
affects: [phase-5-docs]

actuals:
  tokens: 4400
  tasks: 2
  commits: 2
plan_head_before: eef4714eb6665a725022867aaeeb6c6d4f414c69
plan_head_after: 97b1f27806c572e09ba0b7bd3df41cc5e65131c1

tech-stack:
  added: []
  patterns:
    - "A canary's own `run exit:` cannot distinguish a provider 401 from a dead network; a same-conditions fake-key probe records the provider's answer in the step log"
    - "A whole-tree key-shape count is recorded as a measurement, never an equality gate; the artifact gate is `--scan`, which fails closed"
    - "A revocation is a dated author attestation plus a step that proves only the sentence's presence — never presented as a verified fact"

key-files:
  created:
    - docs/evidence/02-loop-item-verification/key-leak/report.json
    - docs/evidence/02-loop-item-verification/key-leak/key-leak.cast
    - docs/evidence/02-loop-item-verification/key-leak/notes.md
    - docs/evidence/02-loop-item-verification/key-leak/attestation-dated-revocation-sentence-present.log
    - docs/evidence/02-loop-item-verification/key-leak/canary-product-wrote-no-key-material.log
    - docs/evidence/02-loop-item-verification/key-leak/canary-the-provider-path-really-ran.log
    - docs/evidence/02-loop-item-verification/key-leak/scan-committed-evidence-tree-is-clean.log
    - docs/evidence/02-loop-item-verification/key-leak/scan-every-path-this-phase-committed-is-clean.log
    - docs/evidence/02-loop-item-verification/key-leak/scan-item-directory-clean-after-the-note.log
    - docs/evidence/02-loop-item-verification/key-leak/baseline-whole-tree-key-shape-grep-measured.log
  modified:
    - docs/evidence/README.md

key-decisions:
  - "The provider-path step records a same-conditions fake-key probe: the suggested typescript grep would have matched the script's own `401/other error is expected` text, proving nothing"
  - "The tracked-file baseline (2 files / 4 lines) is a measurement, not a gate; the gate is the canary plus the two scoped `--scan` runs, and `--scan` fails closed (`IN-07` covers only the default mode)"
  - "The revocation is the author's dated attestation: one sentence, no key fragment or account identifier, and a step that proves only the sentence is present"

patterns-established:
  - "The canary typescript is line-oriented evidence; convert it to a cast only after the steps that grep it have been recorded, then delete the raw intermediates"
  - "A prior human checkpoint that is already satisfied is recorded as an attestation in the item, not re-asked"

requirements-completed: [PROOF-03]

coverage:
  - id: D1
    description: "The default-mode canary runs the release binary with its own fake key in an isolated HOME and reports the fake key absent from 52 saved files and captured output"
    requirement: PROOF-03
    verification:
      - kind: other
        ref: "docs/evidence/02-loop-item-verification/key-leak/report.json#canary: product wrote no key material"
        status: pass
    human_judgment: false
  - id: D2
    description: "The provider path really ran: a same-conditions fake-key probe observed a 401 from api.deepseek.com"
    requirement: PROOF-03
    verification:
      - kind: other
        ref: "docs/evidence/02-loop-item-verification/key-leak/report.json#canary: the provider path really ran"
        status: pass
    human_judgment: false
  - id: D3
    description: "The committed artifacts carry no key material: --scan over docs/evidence and over docs + the phase planning directory both exit 0"
    requirement: PROOF-03
    verification:
      - kind: other
        ref: "docs/evidence/02-loop-item-verification/key-leak/report.json#scan: committed evidence tree is clean"
        status: pass
      - kind: other
        ref: "docs/evidence/02-loop-item-verification/key-leak/report.json#scan: every path this phase committed is clean"
        status: pass
    human_judgment: false
  - id: D4
    description: "The whole-tree key-shape baseline is measured (2 files / 4 lines / 4379 tracked files) and described, with no equality gate"
    requirement: PROOF-03
    verification:
      - kind: other
        ref: "docs/evidence/02-loop-item-verification/key-leak/report.json#baseline: whole-tree key-shape grep measured"
        status: pass
    human_judgment: false
  - id: D5
    description: "The key revocation is recorded as the author's dated attestation, not as a verified fact"
    requirement: PROOF-03
    verification:
      - kind: other
        ref: "docs/evidence/02-loop-item-verification/key-leak/report.json#attestation: dated revocation sentence present"
        status: pass
    human_judgment: true
    rationale: "No API exists to check a revocation; the step proves only that the dated sentence is present. Only the author can confirm the action on platform.deepseek.com."
  - id: D6
    description: "docs/evidence/README.md names all seven Phase-2 items with the command that reproduces each"
    requirement: PROOF-03
    verification:
      - kind: other
        ref: "docs/evidence/README.md#Re-running the Phase-2 items"
        status: pass
    human_judgment: false

duration: 4 min
completed: 2026-11-25
status: complete
---

# Phase 02 Plan 03: Key-leak scan and revocation attestation Summary

**Default-mode canary proves the product writes no key material, scoped `--scan` proves the committed artifacts clean, and the author's key revocation is recorded as a dated attestation rather than a verified fact**

## Performance

- **Duration:** 4 min
- **Started:** 2026-11-25T23:10:10Z
- **Completed:** 2026-11-25T23:13:44Z
- **Tasks:** 2
- **Files modified:** 11

## Accomplishments

- The default-mode canary ran the release binary with the script's own fake key in an isolated `HOME`, debug logging on, against the real provider: `PASS  fake key absent from 52 saved file(s) and captured output`.
- The provider path was actually reached: a same-conditions fake-key probe saw `sampler transport prewarmed origin=https://api.deepseek.com status=401 Unauthorized elapsed_ms=513` and `Unauthorized (401) from https://api.deepseek.com/chat/completions`.
- Both scoped scans exited 0: `--scan docs/evidence` and `--scan docs .planning/phases/02-loop-item-verification` printed `PASS  no key-shaped string`.
- The whole-tree baseline was measured, not gated: 2 matching files, 4 matching lines, 4379 tracked files, all four hits inside `#[cfg(test)]` modules of the two redaction crates.
- The `key-leak` item is sealed (`result: ok`, 7 steps, `PROOF-03`) with `notes.md` and `key-leak.cast` hashed; the note carries one dated revocation sentence and states that no API can verify it.
- `docs/evidence/README.md` gained `## Re-running the Phase-2 items`, naming all seven items with their reproducing commands and expected verdict tokens.

## Task Commits

Each task was committed atomically:

1. **Task 1: Run the canary, scan the artifacts, leave the item pending** — `5c579a98` (docs)
2. **Task 2: Record the dated revocation attestation and seal the item** — `97b1f278` (docs)

**Plan metadata:** _(this commit)_ (docs)

## Files Created/Modified

- `docs/evidence/02-loop-item-verification/key-leak/key-leak.cast` — the default-mode canary run as asciinema v2 (3 events)
- `docs/evidence/02-loop-item-verification/key-leak/report.json` — fixed schema, `result: ok`, 7 steps, `requirements: ["PROOF-03"]`, 2 hashed artifacts
- `docs/evidence/02-loop-item-verification/key-leak/notes.md` — what was scanned, the measured baseline, the dated attestation, the re-run commands
- `docs/evidence/02-loop-item-verification/key-leak/*.log` — one normalized log per recorded step (canary, provider probe, two scans, baseline, attestation, final item scan)
- `docs/evidence/README.md` — appended `## Re-running the Phase-2 items`; the Phase-1 bullets and schema sections are untouched (diff is +50 lines, no deletions)

## Decisions Made

- **The provider-path step records a probe, not just a typescript grep.** The canary script redirects the binary's stdout/stderr into a temp directory and deletes it in its `EXIT` trap, so its typescript carries only `run exit: <n> (401/other error is expected with a fake key)` — a grep for `401` there would match the script's own explanatory text. The step instead runs the same release binary with the same committed fake key against the same endpoint and records the provider's actual 401 in the log. `notes.md` states this explicitly.
- **The whole-tree count is an observation.** 2 files / 4 matching lines at HEAD `eef4714e` (the plan expected the same numbers at `1efb05ac`). No `-eq` gate was added; the real gate is the canary plus the two fail-closed `--scan` runs, with `IN-07` cited for the default mode's fail-open case.
- **The revocation is an attestation.** The author confirmed the rotation/revocation; the item records a single dated sentence and a "recorded, not verified" statement. The `attestation:` step proves only that the sentence is present. No account identifier, key fragment or key length is recorded.
- **The satisfied human checkpoint was not re-asked.** The author's confirmation arrived with the dispatch; the dated sentence was written with the UTC timestamp of that writing (`2026-11-25T23:11:13Z`) and explicitly says the exact platform-action time was not recorded.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Provider-path step needed a real network signal, not the typescript's own hint**
- **Found during:** Task 1 (canary capture)
- **Issue:** The plan's suggested `grep -Ei 'authentication|401|unauthorized|api\.deepseek\.com'` over the canary typescript would match the script's own line `run exit: 1 (401/other error is expected with a fake key)`. That line prints whether the provider answered or the host had no network, so the step would have proved nothing.
- **Fix:** The step greps `run exit:` (as planned) and additionally runs a same-conditions probe — the same release binary, the same fake key committed in `scripts/key-leak-check.sh`, the same endpoint, an isolated `HOME` — capturing `status=401 Unauthorized` from `api.deepseek.com` into the step log. All `sk-` shapes are redacted from that log defensively.
- **Files modified:** `docs/evidence/02-loop-item-verification/key-leak/canary-the-provider-path-really-ran.log`, `notes.md`
- **Verification:** Step recorded ok; `report.json` detail `exit 0: provider observed: 401 from api.deepseek.com`; `--scan` over the item still exits 0.
- **Committed in:** `5c579a98` (Task 1 commit)

**2. [Rule 1 - Bug] The steps table made the attestation grep match twice**
- **Found during:** Task 2 (recording the attestation step)
- **Issue:** The `## Steps` table described the check with the literal `grep -F 'Revoked: '`, so `grep -c 'Revoked: ' notes.md` returned 2 instead of the plan-level verification's exactly one.
- **Fix:** Reworded the table row to "a literal `grep -F` for the dated revocation prefix"; the note now contains exactly one `Revoked: ` line, the dated sentence.
- **Files modified:** `docs/evidence/02-loop-item-verification/key-leak/notes.md`
- **Verification:** `grep -c 'Revoked: ' notes.md` → 1; the final `--scan docs/evidence` exits 0.
- **Committed in:** `97b1f278` (Task 2 commit)

---

**Total deviations:** 2 auto-fixed (1 blocking, 1 bug)
**Impact on plan:** Both strengthen the honesty of the item rather than widen scope; no product code, no dependency, no new outbound destination.

## Issues Encountered

- The canary's `script -T` capture is a single second (the fake-key 401 round trip took under 0.5 s), so the cast has 3 events. The script's saved-file count (52) and the probe's own 513 ms prewarm line are the substance, not the cast duration.
- `evidence.py`'s step detail carries only the last non-empty line, so the baseline step's `detail` is `exit 0: 4379`; the full three-number output is in `baseline-whole-tree-key-shape-grep-measured.log` and in `notes.md`.
- Three-to-four untracked `.planning/phases/03-…`, `04-…`, `05-…`, `06-…` directories appeared in the working tree during this run. They are not this plan's (it writes only `docs/evidence/…` and `docs/evidence/README.md`) and were left untouched.

## User Setup Required

None — no external service configuration required. The key revocation was a human action already confirmed by the author.

## Next Phase Readiness

- PROOF-03 is complete and Phase 2 is 6/6: all seven evidence items (`live-smoke`, `runtime-egress`, `key-leak`, `permission-prompt`, `permission-mode`, `resume-fidelity`, `plan-read-only`) are committed under `docs/evidence/02-loop-item-verification/`.
- Ready for `/gsd-verify-work 2`, then Phase 3 (pipeline decoupling).
- The revocation remains an attestation by construction; a verifier should read it as such, not as a tool-verified fact.

---
*Phase: 02-loop-item-verification*
*Completed: 2026-11-25*

## Self-Check: PASSED

- `docs/evidence/02-loop-item-verification/key-leak/report.json` — FOUND (result ok, 7 steps, PROOF-03)
- `docs/evidence/02-loop-item-verification/key-leak/key-leak.cast` — FOUND (3 events)
- `docs/evidence/02-loop-item-verification/key-leak/notes.md` — FOUND and hashed (one `Revoked: ` line, no key shape)
- `docs/evidence/README.md` — FOUND, `## Re-running the Phase-2 items` names all seven items; Phase-1 sections unchanged (50 added, 0 deleted in the plan range)
- Commit `5c579a98` — FOUND (Task 1)
- Commit `97b1f278` — FOUND (Task 2)
- `scripts/key-leak-check.sh --scan docs/evidence` — exit 0
