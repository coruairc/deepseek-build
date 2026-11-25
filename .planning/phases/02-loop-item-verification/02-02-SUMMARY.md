---
phase: 02-loop-item-verification
plan: 02
subsystem: evidence
tags: [egress, ld-preload, evidence, python-stdlib, proof-02]

requires:
  - phase: 02-loop-item-verification
    provides: "the committed live-smoke transcript of one scripts/egress-check.sh session (02-01)"
provides:
  - "runtime-egress evidence item: report.json, destination-check.py, destinations.txt, notes.md"
  - "a re-runnable PROOF-02 verdict: 4 rows PASS, EGRESS_VIOLATIONS=0, API_CONNECTS=7, WEB_FETCH_CONNECTS=1"
affects: [02-03, 02-06, verify-work]

actuals:
  tokens: 2398
  tasks: 2
  commits: 3
  plan_head_before: d636051304dd4504b95f66daf10b268387519d6c
  plan_head_after: be933d90dbdb486bb969a61a5a7bd824abb8049e

tech-stack:
  added: []
  patterns:
    - "Evidence-local, stdlib-only checker committed inside the item; report.json pins its sha256"

key-files:
  created:
    - docs/evidence/02-loop-item-verification/runtime-egress/report.json
    - docs/evidence/02-loop-item-verification/runtime-egress/destination-check.py
    - docs/evidence/02-loop-item-verification/runtime-egress/destinations.txt
    - docs/evidence/02-loop-item-verification/runtime-egress/notes.md
  modified: []

key-decisions:
  - "Read the 02-01 transcript back instead of performing a second live run"
  - "Adopted the coherent on-disk item after a concurrent writer replaced destination-check.py between artifact recording and commit"
  - "Committed the item per the orchestrator's instruction, which overrides the plan's 'leave the working tree uncommitted' line"

requirements-completed: [PROOF-02]

coverage:
  - id: D1
    description: "runtime-egress/report.json: fixed key set/order, requirements [PROOF-02], result ok, four ok steps, three sha256-pinned artifacts"
    requirement: PROOF-02
    verification:
      - kind: other
        ref: "python3 -c \"...report.json schema assertion...\" (plan <verify>, exit 0)"
        status: pass
    human_judgment: false
  - id: D2
    description: "destination-check.py: stdlib-only re-runnable verdict; exits 1 on each of the six failure conditions and skips the dashed separator"
    requirement: PROOF-02
    verification:
      - kind: other
        ref: "python3 docs/evidence/02-loop-item-verification/runtime-egress/destination-check.py docs/evidence/02-loop-item-verification/live-smoke/live-smoke-transcript.txt /dev/null"
        status: pass
      - kind: other
        ref: "six synthetic controls (FAIL row, bad reason, no table, EGRESS_VIOLATIONS=1, API_CONNECTS=0, WEB_FETCH_CONNECTS=0) each exit 1; separator-deleted copy reports the same 4 rows"
        status: pass
    human_judgment: false
  - id: D3
    description: "destinations.txt: the parsed destination table, four rows, every row PASS with a permitted reason"
    requirement: PROOF-02
    verification:
      - kind: other
        ref: "test -s .../destinations.txt && grep -c PASS .../destinations.txt (4)"
        status: pass
    human_judgment: false
  - id: D4
    description: "notes.md: provenance (same session, read back), the supporting verdict lines, the shim's blind spots, and both re-run commands"
    requirement: PROOF-02
    verification:
      - kind: other
        ref: "grep for 'same session', 'cannot see', 'destination-check.py', 'EGRESS_VIOLATIONS=0' (all present)"
        status: pass
    human_judgment: true
    rationale: "Automation asserts the required statements are present; whether the blind-spot framing is honest and sufficient is a reviewer judgement."
  - id: D5
    description: "The measured session is provably the PROOF-01 session, not a second run: the transcript is unchanged by this plan and pinned by the sha256 02-01 recorded"
    requirement: PROOF-02
    verification:
      - kind: other
        ref: "git diff --name-only d6360513..HEAD -- docs/evidence/02-loop-item-verification/live-smoke/ (empty) and report.json artifact sha256 e31161c4… == 02-01's recorded digest"
        status: pass
    human_judgment: false

duration: 3min
completed: 2026-11-25
status: complete
---

# Phase 2 Plan 2: Runtime egress measured on the live session Summary

**PROOF-02 committed as `runtime-egress`: the 02-01 egress-check session contacted only `api.deepseek.com`, `example.com`, loopback and AF_UNIX — four table rows all `PASS`, `EGRESS_VIOLATIONS=0`, `API_CONNECTS=7`, `WEB_FETCH_CONNECTS=1`, re-checkable with a stdlib-only script.**

## Performance

- **Duration:** 3 min (recorded from plan start; context reads preceded the timer)
- **Started:** 2026-11-25T21:25:57Z
- **Completed:** 2026-11-25T21:29:06Z
- **Tasks:** 2
- **Files created:** 8 (4 item files + 4 step logs); **modified:** 0

## Measurements recorded

**No live run was performed by this plan.** The measurement is a read-back of the
session 02-01 captured; no key was exported, no socket was opened, and no network
call was made.

Source transcript: `docs/evidence/02-loop-item-verification/live-smoke/live-smoke-transcript.txt`
sha256 `e31161c4b33c5f1d1c8e7b32fab39f28b3bcd44c7b059d71120f6bd73ff389c0`
(identical to the digest 02-01 recorded for the same file).

Parsed observed-destinations table (`destinations.txt`, produced by
`destination-check.py`, not hand-written):

| count | family | destination | port | host | status | reason |
|---|---|---|---|---|---|---|
| 10 | unix | /run/systemd/resolve/io.systemd.Resolve | 0 | - | PASS | unix |
| 7 | inet | 3.173.21.63 | 443 | api.deepseek.com | PASS | allowlisted |
| 2 | unix | /var/run/dbus/system_bus_socket | 0 | - | PASS | unix |
| 1 | inet6 | 2606:4700:10::6814:179a | 443 | example.com | PASS | allowlisted |

- **Rows:** 4. **Connects:** 20.
- **By family:** unix 2 rows / 12 connects; inet 1 row / 7 connects; inet6 1 row / 1 connect.
- **By reason:** `unix` 2 rows / 12 connects; `allowlisted` 2 rows / 8 connects. No other reason appears.

Connect counters asserted greater than zero by the checker: **`API_CONNECTS=7`**
and **`WEB_FETCH_CONNECTS=1`**; **`EGRESS_VIOLATIONS=0`**.

**Negative control:** a synthetic copy of the transcript with the dbus row flipped
to `FAIL  NOT-ALLOWED` makes the checker exit 1 with
`row status is not PASS: FAIL`. The other five failure conditions were each
exercised and each exit 1 (bad reason outside the permitted set, no table,
`EGRESS_VIOLATIONS=1`, `API_CONNECTS=0`, `WEB_FETCH_CONNECTS=0`). A copy of the
transcript with the dashed separator row deleted reports the same **4** rows, so
the separator is skipped rather than mis-parsed.

**Leak scan:** `scripts/key-leak-check.sh --scan docs/evidence/02-loop-item-verification/runtime-egress`
→ `PASS  no key-shaped string in: …`, exit 0.

## Accomplishments

- `runtime-egress` item committed with the Phase-1 fixed `report.json` schema,
  `result: ok`, the four named steps all `ok`, and three sha256-pinned artifacts
  (checker, parsed table, and the shared live-smoke transcript).
- `destination-check.py` is a standard-library-only re-runnable verdict: it parses
  the observed-destinations table, requires every row `PASS` with a reason in
  `{allowlisted, loopback, unix, unspecified}`, requires `EGRESS_VIOLATIONS=0`,
  and requires both connect counters to be greater than zero — so a run that
  contacted nothing, or contacted a non-permitted host, fails.
- `destinations.txt` is that checker's output, so a reviewer re-runs the parse
  instead of trusting a number in prose.
- `notes.md` states the same-session provenance, the exact verdict lines, the
  instrument's blind spots in `scripts/egress-check.sh`'s own terms, and both
  re-run commands.

## Task Commits

Each task was committed atomically:

1. **Task 1: Record the egress verdict and the parsed destination table end to end** — `b8dd636d` (docs)
2. **Task 1 reconciliation: make the recorded hashes match the committed checker** — `de4ecfbe` (fix)
3. **Task 2: Write the honesty note** — `be933d90` (docs)

**Plan metadata:** committed as `docs(02-02): complete runtime-egress plan`.

## Files Created/Modified

- `docs/evidence/02-loop-item-verification/runtime-egress/report.json` — fixed-schema report, `PROOF-02`, `result: ok`
- `docs/evidence/02-loop-item-verification/runtime-egress/destination-check.py` — stdlib-only parse and verdict
- `docs/evidence/02-loop-item-verification/runtime-egress/destinations.txt` — the four parsed rows
- `docs/evidence/02-loop-item-verification/runtime-egress/notes.md` — provenance, blind spots, re-run
- `docs/evidence/02-loop-item-verification/runtime-egress/egress-*.log` — the four recorded step outputs (normalized)

## Decisions Made

- **Read the recording, not a second session.** PROOF-02 says "that same session";
  a fresh run would be a weaker claim and would need a key this plan deliberately
  did not touch. The transcript is pinned by sha256, so a substitution is visible.
- **Adopt the coherent on-disk item after the hash mismatch.** The item directory
  was written by two overlapping processes (see Deviations). The on-disk checker
  and `destinations.txt`/`report.json` were mutually consistent and passed all six
  negative controls plus the separator control when re-verified here, so they were
  kept rather than overwriting another session's work; the committed digest now
  vouches for a file that exists.
- **Keep `notes.md` coarse about the shim.** The blind-spot section uses the
  script's own vocabulary and stops short of claiming content-level isolation.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Commit pinned a digest that no committed file had**
- **Found during:** Task 1, immediately after the tracer verify
- **Issue:** A second process wrote the same `runtime-egress` directory while this
  plan ran: it replaced `destination-check.py` between `evidence.py artifact`
  (which hashed this plan's version, `1700a16e…`) and `git add` (which therefore
  staged the newer, equivalent `d8c88992…`), then rewrote `destinations.txt`, the
  step log and `report.json`. Commit `b8dd636d` therefore contained a
  `report.json` vouching for a checker the tree did not have.
- **Fix:** Re-verified the coherent on-disk set end to end (schema, artifact
  digests against disk, four steps ok, every row `PASS`, six negative controls,
  separator control, leak scan) and committed it as `de4ecfbe` so every recorded
  digest matches a committed file.
- **Files modified:** `runtime-egress/destinations.txt`,
  `runtime-egress/egress-destination-table-parsed-and-permitted.log`,
  `runtime-egress/report.json`
- **Verification:** report.json artifact digests recomputed from disk — all match;
  `destination-check.py` exits 0 on the transcript and 1 on each synthetic failure.
- **Committed in:** `de4ecfbe`

### Plan-text deviation

**2. [Rule 4 - Process] Committed the plan, though `<output>` says to leave it uncommitted**
- **Found during:** Task 1 (before the first commit)
- **Issue:** The plan's `<output>` ends with "Leave the working tree uncommitted for
  review (C1: one logical commit per plan, made after the reviewer passes)", while
  the orchestrator's dispatch instructs atomic per-task commits with `(02-02):`
  subjects and asks for a SHA range.
- **Fix:** Followed the orchestrator: three commits, scoped `(02-02)`. The plan's
  substantive `<output>` requirements (the recorded facts below) are all honoured.
  Raised here rather than resolved silently.
- **Impact:** The C1 reviewer-gated single commit is not what the history shows; a
  reviewer can still review `d6360513..be933d90` as one logical change.

### Environment deviation

**3. GSD state helpers unavailable**
- `gsd_run` / `gsd-tools` are not on PATH in this environment, so STATE.md and
  ROADMAP.md were not advanced by this executor. The orchestrator owns post-plan
  sync; ROADMAP still reads `2/6` for Phase 2 while three plans (02-01, 02-04,
  02-02) are complete. Nothing in the plan's files depends on that update.

---

**Total deviations:** 1 auto-fixed (Rule 1), 1 process deviation (Rule 4 raised),
1 environment limitation.
**Impact on plan:** No scope creep. The item is internally consistent and its
claims are supported by committed files only.

## Issues Encountered

- **Concurrent writer in the same working tree.** A second process produced the
  same `runtime-egress` item (its own `report.json`, date `21:26:58Z`) while this
  plan was recording. Files stopped changing at `22:26:59` local; the tree was
  stable for the reconciliation commit and for both verify passes afterwards. If
  another executor is still running 02-02, it may produce a duplicate
  `02-02-SUMMARY.md` and a no-op commit; the item directory itself is coherent as
  committed here.
- **`same session` needed one line.** The first `notes.md` draft wrapped "same
  session" across a line break, so the plan's `grep -q 'same session'` failed;
  rephrased so the phrase and its provenance sentence sit together.

## User Setup Required

None — no external service configuration and no key was used.

## Next Phase Readiness

- PROOF-02 is proven by committed files: a reviewer can re-run
  `destination-check.py` against the committed transcript and reproduce every
  number in this summary.
- 02-03 (`key-leak`, PROOF-03) can proceed: the artifacts it must scan now exist.
- The phase-level caveat stands: this reads a recording of one session on the
  author's build host; it is not a new measurement.

## Self-Check: PASSED

- `report.json` present with `result: ok`, `requirements: [PROOF-02]`, four ok steps, three artifacts — verified.
- `destination-check.py`, `destinations.txt`, `notes.md` present on disk — verified.
- Commits `b8dd636d`, `de4ecfbe`, `be933d90` present on `main` — verified.
- `scripts/key-leak-check.sh --scan docs/evidence/02-loop-item-verification/runtime-egress` exit 0 — verified.

---

*Phase: 02-loop-item-verification*
*Completed: 2026-11-25*
