---
phase: 02-loop-item-verification
verified: 2026-11-25T23:25:03Z
status: gaps_found
score: "11/11"
covered_files:
  - ".planning/phases/02-loop-item-verification/02-01-PLAN.md"
  - ".planning/phases/02-loop-item-verification/02-01-SUMMARY.md"
  - ".planning/phases/02-loop-item-verification/02-02-PLAN.md"
  - ".planning/phases/02-loop-item-verification/02-02-SUMMARY.md"
  - ".planning/phases/02-loop-item-verification/02-03-PLAN.md"
  - ".planning/phases/02-loop-item-verification/02-03-SUMMARY.md"
  - ".planning/phases/02-loop-item-verification/02-04-PLAN.md"
  - ".planning/phases/02-loop-item-verification/02-04-SUMMARY.md"
  - ".planning/phases/02-loop-item-verification/02-05-PLAN.md"
  - ".planning/phases/02-loop-item-verification/02-05-SUMMARY.md"
  - ".planning/phases/02-loop-item-verification/02-06-PLAN.md"
  - ".planning/phases/02-loop-item-verification/02-06-SUMMARY.md"
  - "KNOWN-ISSUES.md"
  - "crates/codegen/xai-grok-pager-bin/tests/headless_loop_items.rs"
  - "crates/codegen/xai-grok-pager-pty-harness/src/scenarios/plan_approval_resume.rs"
  - "crates/codegen/xai-grok-pager-pty-harness/tests/pty_e2e/permission_mode_resume.rs"
  - "crates/codegen/xai-grok-pager-pty-harness/tests/pty_e2e/permission_prompt_outcomes.rs"
  - "crates/codegen/xai-grok-pager-pty-harness/tests/pty_e2e/plan_mode_read_only.rs"
  - "crates/codegen/xai-grok-pager-pty-harness/tests/pty_e2e_shell_tools.rs"
  - "crates/codegen/xai-grok-pager/docs/user-guide/22-permissions-and-safety.md"
  - "crates/codegen/xai-grok-pager/src/app/acp_handler/permissions.rs"
  - "crates/codegen/xai-grok-pager/src/headless.rs"
  - "crates/codegen/xai-grok-pager/src/views/permission_view.rs"
  - "crates/codegen/xai-grok-shell/src/session/acp_session_tests/plan_mode_edit_gate_tests.rs"
  - "docs/evidence/02-loop-item-verification/key-leak/attestation-dated-revocation-sentence-present.log"
  - "docs/evidence/02-loop-item-verification/key-leak/baseline-whole-tree-key-shape-grep-measured.log"
  - "docs/evidence/02-loop-item-verification/key-leak/canary-product-wrote-no-key-material.log"
  - "docs/evidence/02-loop-item-verification/key-leak/canary-the-provider-path-really-ran.log"
  - "docs/evidence/02-loop-item-verification/key-leak/key-leak.cast"
  - "docs/evidence/02-loop-item-verification/key-leak/notes.md"
  - "docs/evidence/02-loop-item-verification/key-leak/report.json"
  - "docs/evidence/02-loop-item-verification/key-leak/scan-committed-evidence-tree-is-clean.log"
  - "docs/evidence/02-loop-item-verification/key-leak/scan-every-path-this-phase-committed-is-clean.log"
  - "docs/evidence/02-loop-item-verification/key-leak/scan-item-directory-clean-after-the-note.log"
  - "docs/evidence/02-loop-item-verification/live-smoke/adapter-smoke-transcript.txt"
  - "docs/evidence/02-loop-item-verification/live-smoke/adapter-smoke.cast"
  - "docs/evidence/02-loop-item-verification/live-smoke/live-smoke-transcript.txt"
  - "docs/evidence/02-loop-item-verification/live-smoke/live-smoke.cast"
  - "docs/evidence/02-loop-item-verification/live-smoke/loop-edit-tool-ran.log"
  - "docs/evidence/02-loop-item-verification/live-smoke/loop-resumed-the-same-session.log"
  - "docs/evidence/02-loop-item-verification/live-smoke/loop-shell-tool-ran.log"
  - "docs/evidence/02-loop-item-verification/live-smoke/loop-web-fetch-made-a-real-connection.log"
  - "docs/evidence/02-loop-item-verification/live-smoke/notes.md"
  - "docs/evidence/02-loop-item-verification/live-smoke/report.json"
  - "docs/evidence/02-loop-item-verification/live-smoke/smoke-adapter-checks-all-passed.log"
  - "docs/evidence/02-loop-item-verification/permission-mode/notes.md"
  - "docs/evidence/02-loop-item-verification/permission-mode/perm-03-permission-chip-matches-after-resume-by-session-id.log"
  - "docs/evidence/02-loop-item-verification/permission-mode/perm-04-headless-permission-mode-plan-arms-plan-mode.log"
  - "docs/evidence/02-loop-item-verification/permission-mode/permission-mode-before-quit.cast"
  - "docs/evidence/02-loop-item-verification/permission-mode/permission-mode.cast"
  - "docs/evidence/02-loop-item-verification/permission-mode/report.json"
  - "docs/evidence/02-loop-item-verification/permission-prompt/notes.md"
  - "docs/evidence/02-loop-item-verification/permission-prompt/perm-01-three-outcomes-and-the-warning-row-on-a-real-screen.log"
  - "docs/evidence/02-loop-item-verification/permission-prompt/perm-02-auto-mode-probe-line-recorded.log"
  - "docs/evidence/02-loop-item-verification/permission-prompt/perm-02-both-denial-paths-name-both-flags.log"
  - "docs/evidence/02-loop-item-verification/permission-prompt/permission-prompt.cast"
  - "docs/evidence/02-loop-item-verification/permission-prompt/report.json"
  - "docs/evidence/02-loop-item-verification/plan-read-only/notes.md"
  - "docs/evidence/02-loop-item-verification/plan-read-only/plan-read-only-always-approve.cast"
  - "docs/evidence/02-loop-item-verification/plan-read-only/plan-read-only-auto.cast"
  - "docs/evidence/02-loop-item-verification/plan-read-only/plan-read-only-default.cast"
  - "docs/evidence/02-loop-item-verification/plan-read-only/report.json"
  - "docs/evidence/02-loop-item-verification/plan-read-only/sess-02-plan-mode-refuses-out-of-plan-edits-in-every-permission-mode.log"
  - "docs/evidence/02-loop-item-verification/plan-read-only/sess-02-the-gate-takes-no-permission-policy.log"
  - "docs/evidence/02-loop-item-verification/resume-fidelity/notes.md"
  - "docs/evidence/02-loop-item-verification/resume-fidelity/report.json"
  - "docs/evidence/02-loop-item-verification/resume-fidelity/resume-fidelity.cast"
  - "docs/evidence/02-loop-item-verification/resume-fidelity/sess-01-resumed-model-effort-and-plan-state.log"
  - "docs/evidence/02-loop-item-verification/resume-fidelity/sess-03-drafted-plan-survives-quit-and-resume.log"
  - "docs/evidence/02-loop-item-verification/runtime-egress/destination-check.py"
  - "docs/evidence/02-loop-item-verification/runtime-egress/destinations.txt"
  - "docs/evidence/02-loop-item-verification/runtime-egress/egress-allowlist-verdict-passed.log"
  - "docs/evidence/02-loop-item-verification/runtime-egress/egress-destination-table-parsed-and-permitted.log"
  - "docs/evidence/02-loop-item-verification/runtime-egress/egress-provider-and-web-fetch-both-contacted.log"
  - "docs/evidence/02-loop-item-verification/runtime-egress/egress-zero-violations.log"
  - "docs/evidence/02-loop-item-verification/runtime-egress/notes.md"
  - "docs/evidence/02-loop-item-verification/runtime-egress/report.json"
  - "docs/evidence/README.md"
covered_digest: "v4:sha256:d623559b8dd93464fe24879290b59988fa05a14cc34529e849e954df05bcfdbd"
behavior_unverified: 0
overrides_applied: 0
gaps:
  - truth: "scripts/test-gate.expected reflects the new headless_loop_items integration target, so scripts/test-gate.sh can run its completeness assertion green"
    status: failed
    reason: "02-04 added the phase's one new test binary but scripts/test-gate.sh --refresh-expected refused on two pre-existing xai-grok-agent failures and wrote nothing. The file still records results=308 / binaries=219 / doctests=89; 02-04's summary says the gate derived results=309 / binaries=220 / doctests=89. 02-04's declared acceptance criterion (binaries>=220, results>=309) therefore fails on the committed file, and a green full run would fail the gate's expected-count comparison. 02-VALIDATION.md also lists this refresh as an unchecked Wave-0 requirement (`wave_0_complete: false`)."
    artifacts:
      - path: "scripts/test-gate.expected"
        issue: "Stale counts: results=308, binaries=219, doctests=89; the plan requires binaries>=220 and results>=309 after the new target. The file header still names commit c159a3e7 / 2026-11-23."
      - path: ".planning/phases/02-loop-item-verification/02-04-SUMMARY.md"
        issue: "Records the refusal honestly, but its `status: complete` and the phase `6/6` claim leave the plan acceptance criterion unmet without a re-opened item."
      - path: ".planning/phases/02-loop-item-verification/02-VALIDATION.md"
        issue: "Wave-0 requirement 'scripts/test-gate.expected — refreshed once, by 02-04' is still unchecked (`status: draft`, `wave_0_complete: false`)."
    missing:
      - "A green full run (fix or quarantine the two pre-existing xai-grok-agent failures first), then scripts/test-gate.sh --refresh-expected — never a hand edit — and commit the refreshed counts."
      - "If the full run cannot be made green in this milestone, record an explicit owner and a later phase for the refresh instead of leaving the phase's validation file in draft with the Wave-0 item unchecked."
---

# Phase 2: Loop-Item Verification — Verification Report

**Phase Goal:** As the author, I want the daily loop — permissions, resume, plan mode — and the runtime egress boundary verified against a real model-driven session with a throwaway key, so that "it works" rests on committed artifacts rather than on a pty scenario I wrote myself.

**Verified:** 2026-11-25T23:25:03Z
**Status:** `gaps_found` — every requirement and every roadmap success criterion is verified; one plan-level/validation acceptance criterion (the test-gate count refresh) is not met.
**Re-verification:** No — initial verification. There is no earlier `02-VERIFICATION.md` in this phase directory.

## User Flow Coverage (MVP mode)

Phase 2 is `mode: mvp` in `ROADMAP.md`. The user story's outcome clause is "it works rests on committed artifacts rather than on a pty scenario I wrote myself." The flow below is the author's daily loop as the evidence items record it.

| # | User-flow step | Expected | Evidence in the codebase | Verdict |
|---|---|---|---|---|
| 1 | Export a real key and run one model-driven session | Shell, file edit and `web_fetch` all execute against `api.deepseek.com` / `example.com` | `docs/evidence/02-loop-item-verification/live-smoke/live-smoke-transcript.txt`: `PASS shell tool ran …`, `PASS edit tool ran …`, `PASS web_fetch made a real connection to example.com`, `API_CONNECTS=7`, `WEB_FETCH_CONNECTS=1`; `report.json` 5/5 steps ok | ✓ VERIFIED |
| 2 | Measure egress on the same session | Only provider, `web_fetch` host and AF_UNIX; zero violations | `runtime-egress/report.json`; `destination-check.py` re-run → `parsed 4 rows; API_CONNECTS=7; WEB_FETCH_CONNECTS=1`, exit 0; table: 7×`api.deepseek.com`, 1×`example.com`, 12×AF_UNIX; `EGRESS_VIOLATIONS=0` | ✓ VERIFIED |
| 3 | Scan for key material and revoke the key | Tree/artifacts clean; revocation recorded, not faked | `key-leak/report.json` 7/7 steps ok; `--scan docs/evidence` exit 0; dated `Revoked:` attestation with "recorded rather than verified" | ✓ VERIFIED |
| 4 | Use the interactive permission prompt | Allow-once, allow-for-session, reject; warning on a high-signal command | `permission-prompt/report.json` 3/3 steps ok; `permission_prompt_outcomes_and_warning` pty test; `permission-prompt.cast` contains all three labels and `Security warning:` | ✓ VERIFIED |
| 5 | Hit headless denial | Fail closed; hint names `--always-approve` and `--permission-mode auto` on stderr and JSON | `permission-prompt/perm-02-both-denial-paths-name-both-flags.log` lines 290–331; test asserts exit 1, both flags, `stopReason: "cancelled"`, no new result key | ✓ VERIFIED |
| 6 | Resume a session | Model, effort, plan state restored; permission chip matches | `resume-fidelity/report.json` 2/2 steps ok; `sess-01` log `started_model=deepseek-v4-pro … resumed_model=deepseek-v4-pro … resumed_effort=high … override_effort=low`; `sess-03` log model before/after and plan body; `permission-mode` pty chip test | ✓ VERIFIED (PERM-03 caveat below) |
| 7 | Enter plan mode | Read-only except the plan file in default, auto, always-approve | `plan-read-only/report.json` 2/2 steps ok; pty log shows the refusal and the executed plan-file edit in all three modes; unit log `11 passed; 0 failed` | ✓ VERIFIED |
| 8 | Read the result later | A reviewer can re-run every claim from committed files | `docs/evidence/README.md` `## Re-running the Phase-2 items` names all seven items; every item has `report.json` + `notes.md` (+ cast where a pty was used) | ✓ VERIFIED |

## Roadmap Success Criteria

| # | Success criterion | Status | Evidence |
|---|---|---|---|
| 1 | Live smoke on a real model-driven session with a throwaway key — shell, edit, `web_fetch` — transcript committed | ✓ VERIFIED | `docs/evidence/02-loop-item-verification/live-smoke/`; `report.json` `result: ok`, `requirements: ["PROOF-01"]`, 5/5 steps; transcript lines quoted above; `live-smoke.cast`/`adapter-smoke.cast` content matches the transcripts exactly (line-by-line ratio 1.0); `adapter-smoke-transcript.txt` `8 passed, 0 failed` and real usage (`cache_read=44544`) |
| 2 | Egress on that same session: provider, `web_fetch` host, AF_UNIX only; zero violations; committed | ✓ VERIFIED | `runtime-egress/`; `report.json` 4/4 steps; `live-smoke-transcript.txt` table and counters; `destination-check.py` exit 0; `destinations.txt` is its output; report pins the same transcript sha256 `e31161c4…` that 02-01 recorded; explicitly a read-back, not a second run |
| 3 | `key-leak-check.sh` green over tree and artifacts; no key material in logs/sessions/evidence; key revoked | ✓ VERIFIED (with scan-scope caveat) | `key-leak/`; default canary `PASS fake key absent from 52 saved file(s) and captured output`; `--scan docs/evidence` exit 0; `--scan docs .planning/phases/02-loop-item-verification` exit 0; whole-tree baseline measured 2 files / 4 lines / 4379 tracked files; revocation is a dated author attestation, explicitly not tool-verified. `--scan .` exits 1 on the scanner's own canary and two `#[cfg(test)]` redaction strings — see weaknesses. |
| 4 | Prompt three outcomes + warning; headless denial both paths; mode visible/matches after resume; `--permission-mode plan` arms plan mode; `KNOWN-ISSUES.md` corrected, code unchanged | ✓ VERIFIED (PERM-03 caveat) | `permission-prompt/` (PERM-01/02) and `permission-mode/` (PERM-03/04); pty/headless test code and step logs; `KNOWN-ISSUES.md:216` correction; `git log` shows no change to `apply_headless_permission_mode`; `TESTING.md` unchanged |
| 5 | Resume restores model/effort/plan; plan mode read-only except the plan file in every permission mode; drafted plan survives quit/resume as evidence; provenance documented | ✓ VERIFIED | `resume-fidelity/` (SESS-01/03/04) and `plan-read-only/` (SESS-02); logs/casts quoted below; `resume-fidelity/notes.md` provenance table; `crates/codegen/xai-grok-pager/docs/user-guide/22-permissions-and-safety.md:64` provenance paragraph |

## Requirement Verdicts

| Requirement | Verdict | Exact artifact + step that proves it |
|---|---|---|
| **PROOF-01** | ✓ VERIFIED | `docs/evidence/02-loop-item-verification/live-smoke/report.json` — `result: "ok"`, 5 steps all `ok`: `smoke: adapter checks all passed` (`8 passed, 0 failed`), `loop: shell tool ran`, `loop: edit tool ran`, `loop: web_fetch made a real connection`, `loop: resumed the same session`. Backing transcript lines are in `live-smoke-transcript.txt`; cast matches it line-for-line. |
| **PROOF-02** | ✓ VERIFIED | `docs/evidence/02-loop-item-verification/runtime-egress/report.json` — 4 steps all `ok`; `destination-check.py` re-run exits 0 and reproduces `destinations.txt` byte-for-byte (`parsed 4 rows; API_CONNECTS=7; WEB_FETCH_CONNECTS=1`); the transcript's table has 4 rows, every one `PASS` with reason `unix`/`allowlisted`, and `EGRESS_VIOLATIONS=0`. The report pins the live-smoke transcript sha256 `e31161c4…`. |
| **PROOF-03** | ✓ VERIFIED | `docs/evidence/02-loop-item-verification/key-leak/report.json` — 7 steps all `ok`: canary PASS (52 saved files), provider 401, `--scan docs/evidence` exit 0, `--scan docs .planning/…` exit 0, baseline 2/4/4379, dated revocation sentence present, final item scan exit 0. `notes.md` carries exactly one `Revoked:` line and says "recorded rather than verified"; no key material anywhere in `docs/evidence`. |
| **PERM-01** | ✓ VERIFIED | `permission-prompt/report.json` step `perm-01: three outcomes and the warning row on a real screen` (`1 passed; 0 failed`), plus the pty test's real assertions: allow-once runs and removes `build/`, always-allow runs and the identical repeat settles without a prompt, reject prints `permission was denied` and never reaches the model reply; `permission-prompt.cast` contains `Yes, proceed`, `Always allow:`, `No, reject (type to add feedback)`, `Security warning:` (counts 5/5/4/3). |
| **PERM-02** | ✓ VERIFIED | `permission-prompt/report.json` step `perm-02: both denial paths name both flags`; `perm-02-both-denial-paths-name-both-flags.log` shows the plain stderr block and the JSON stderr block each carrying the `Turn ended: …` hint and the `Error: permission-gated tool cancelled; …` line with **both** flags; the test asserts exit 1, both flags, `stopReason: "cancelled"`, and no new result key. The auto-mode probe line is recorded (observed, not asserted), as the plan required. |
| **PERM-03** | ✓ VERIFIED (caveat) | `permission-mode/report.json` step `perm-03: permission chip matches after resume by session id` (`1 passed; 0 failed`); the pty test launches with `[ui] permission_mode = "always-approve"` and **no** `--permission-mode` on either launch, resumes `-r <sessionId>` (read from the one session directory), asserts `contains("always-approve")` before quit and after resume, and `transcript_restored=true`. Caveat: the log's chip line is the constant `CHIP`, and the cast's final frame shows `unknown · plan`; see weaknesses. |
| **PERM-04** | ✓ VERIFIED | `permission-mode/report.json` step `perm-04: headless --permission-mode plan arms plan mode`; `perm-04-…log` line 317 matches `Plan mode is active. Do not make any edits or writes to the system.` in the captured request body, and line 319 shows the same session's `plan_mode.json` `state=Active`; `KNOWN-ISSUES.md:216` is the corrected plan-mode entry; `git log` shows no change to `apply_headless_permission_mode`. |
| **SESS-01** | ✓ VERIFIED | `resume-fidelity/report.json` step `sess-01: resumed model, effort and plan state`; `sess-01-resumed-model-effort-and-plan-state.log` line 379: `started_model=deepseek-v4-pro started_effort=high resumed_model=deepseek-v4-pro resumed_effort=high override_model=deepseek-v4-pro override_effort=low plan_reminder_full=false plan_reminder_sparse=true`. The test asserts equality of model/effort and the plan reminder on the request tail, and that `--effort low` wins on the third resume. |
| **SESS-02** | ✓ VERIFIED | `plan-read-only/report.json` step `sess-02: plan mode refuses out-of-plan edits in every permission mode` (`1 passed`) and `sess-02: the gate takes no permission policy` (`11 passed`); `sess-02-plan-mode-refuses…log` shows the refusal and `plan-file edit ran` for `default`, `auto` and `always-approve`; the unit test calls `plan_mode_edit_gate` directly with no permission policy and asserts `RejectNonPlanFile` for `SearchReplace`/`ApplyPatch`, `Allow` for the plan file. |
| **SESS-03** | ✓ VERIFIED | `resume-fidelity/report.json` step `sess-03: drafted plan survives quit and resume`; `sess-03-drafted-plan-survives-quit-and-resume.log`: `SESS-03: model deepseek-flash before and after the quit (summary and request body); plan body shown after resume`; `resume-fidelity.cast` contains `Plan GBT3703Repro`, the approval chrome (`request changes`, `quit plan`), and `GBT3703IMPLEMENTED`. |
| **SESS-04** | ✓ VERIFIED | `resume-fidelity/notes.md` "Provenance of each resumed setting" table names model/effort (session record; explicit flag wins), plan state/plan file (session directory), and permission mode (config/CLI, **not** in the session record). The user guide paragraph at `crates/codegen/xai-grok-pager/docs/user-guide/22-permissions-and-safety.md:64` says the same. Code references spot-checked: `agent/mvp_agent/session_setup.rs:1675,1697`; `session/plan_mode.rs:84`; `session/acp_session_impl/spawn.rs:654`; `app/event_loop.rs:1098–1103,1504`; `settings/registry.rs:1269`. |

## Evidence Item Audit

| Item | Requirements | `result` | Steps | Artifacts (sha256 vs disk) | Cast / notes |
|---|---|---|---|---|---|
| `live-smoke` | PROOF-01 | `ok` | 5/5 ok | 4/4 match | 2 casts; `live-smoke-transcript.txt`; notes |
| `runtime-egress` | PROOF-02 | `ok` | 4/4 ok | 3/3 match (incl. the live-smoke transcript) | no cast by design (read-back); notes |
| `key-leak` | PROOF-03 | `ok` | 7/7 ok | 2/2 match (`key-leak.cast`, `notes.md`) | cast + 7 step logs; notes |
| `permission-prompt` | PERM-01, PERM-02 | `ok` | 3/3 ok | 4/4 match | cast; 3 step logs; notes |
| `permission-mode` | PERM-03, PERM-04 | `ok` | 2/2 ok | 4/4 match | 2 casts; 2 step logs; notes |
| `resume-fidelity` | SESS-01, SESS-03, SESS-04 | `ok` | 2/2 ok | 3/3 match | cast; 2 step logs; notes |
| `plan-read-only` | SESS-02 | `ok` | 2/2 ok | 5/5 match | 3 casts; 2 step logs; notes |

- **Total artifact check:** 25 artifact entries across the seven `report.json` files; every sha256 recomputed from disk matched (0 mismatches, 0 missing).
- **Schema check:** every `report.json` has exactly the fixed key order `item, requirements, date, host, command, steps, artifacts, result`; every `host` is `Linux/x86_64`; every `result` is `ok`; every step is `ok` (25/25).
- **Step-log provenance:** every step's log exists and is normalized; no `.evidence-pending.json`, `*.typescript` or `*.timing` intermediates remain in any item.
- **Cast/transcript agreement:** `live-smoke.cast` vs `live-smoke-transcript.txt` and `adapter-smoke.cast` vs `adapter-smoke-transcript.txt` were concatenated and compared line-by-line; both ratio 1.0 (same capture, not a re-run).
- **Key-leak item honesty:** `notes.md` has exactly one `Revoked:` line, dated `2026-11-25`, and says the sentence is present but the revocation cannot be verified; no tool claim is made. `key-leak.cast` shows the canary PASS line and the 52-file count.
- **README convention:** `docs/evidence/README.md` gained `## Re-running the Phase-2 items` (+50/−0) naming all seven items with commands and expected verdicts.
- **Path normalization:** no `/home/<user>`, `/Users/<user>` or `/var/folders` path in any `report.json`, log, cast or note. Normalized `<REPO>`, `<HOME>`, `<TMP>` are used. A few hand-written `notes.md` re-run commands still contain the build host's `/tmp/opencode/...` binary path; that is a `notes.md` command, not a captured log or `report.json` string, and no home path leaks.

## Negative Control (runtime-egress `destination-check.py`)

Command: `python3 docs/evidence/02-loop-item-verification/runtime-egress/destination-check.py <synthetic> /tmp/out`

| Control | Expected | Actual | Result |
|---|---|---|---|
| Committed transcript (positive) | exit 0, 4 rows | `parsed 4 rows; API_CONNECTS=7; WEB_FETCH_CONNECTS=1` | PASS |
| Synthetic `FAIL` row | non-zero | exit 1, `row status is not PASS: FAIL` | PASS (the requested negative control) |
| Reason changed to `frobnicated` | non-zero | exit 1, `row reason outside permitted set: frobnicated` | PASS |
| `EGRESS_VIOLATIONS=1` | non-zero | exit 1, `EGRESS_VIOLATIONS= is absent or not 0` | PASS |
| `API_CONNECTS=0` | non-zero | exit 1, `API_CONNECTS= is absent or not greater than zero` | PASS |
| `WEB_FETCH_CONNECTS=0` | non-zero | exit 1, `WEB_FETCH_CONNECTS= is absent or not greater than zero` | PASS |
| No table | non-zero | exit 1, `no observed-destinations table found` | PASS |
| Dashed separator deleted | still 4 rows | exit 0, `parsed 4 rows; API_CONNECTS=7; WEB_FETCH_CONNECTS=1` | PASS |
| Extra fabricated row labelled `PASS allowlisted` | ideally non-zero | exit 0 (accepted) | **LIMITATION** — the checker validates status/reason, not the destination's identity; it trusts `egress-check.sh`'s classification. The committed transcript is the source of the table, so this does not change the committed verdict. |

## Leak Scans (real exit codes)

| Command | Exit | Output |
|---|---|---|
| `scripts/key-leak-check.sh --scan docs/evidence` | **0** | `PASS  no key-shaped string in: docs/evidence` |
| `scripts/key-leak-check.sh --scan docs .planning/phases/02-loop-item-verification` | **0** | `PASS  no key-shaped string in: docs .planning/phases/02-loop-item-verification` |
| `scripts/key-leak-check.sh --scan docs/evidence/02-loop-item-verification/{item}` (each of the seven) | **0** | `PASS` per item (the item reports record the same) |
| `scripts/key-leak-check.sh --scan .` | **1** | `FAIL  key-shaped string found in 3 file(s): ./scripts/key-leak-check.sh`, `crates/codegen/xai-grok-secrets/src/sanitizer.rs`, `crates/codegen/xai-grok-telemetry/src/redact_common.rs` — the scanner's own fake-key constant and two deliberate `#[cfg(test)]` canaries. No real key material. |
| `grep -RInE '/home/[A-Za-z0-9_.-]+|/Users/[A-Za-z0-9_.-]+' docs/evidence/02-loop-item-verification` | no match | no absolute home path in a committed item |
| `grep -RInE 'sk-[A-Za-z0-9]{20,}' docs/evidence/02-loop-item-verification` | no match | no key-shaped string in the evidence tree |

## Plan Artifact & Test Audit

| Plan | Required files exist? | Declared test status | `#[ignore]` / skip reason |
|---|---|---|---|
| 02-01 | Yes — `live-smoke/{report.json,live-smoke.cast,adapter-smoke.cast,live-smoke-transcript.txt,adapter-smoke-transcript.txt,notes.md}` + five step logs | Live scripts ran; report steps 5/5 ok | n/a (live scripts; no `#[ignore]`) |
| 02-02 | Yes — `runtime-egress/{report.json,destination-check.py,destinations.txt,notes.md}` + four step logs | Checker re-run exit 0; six negative controls exit 1; separator control 4 rows | n/a |
| 02-03 | Yes — `key-leak/{report.json,key-leak.cast,notes.md}` + seven step logs; `docs/evidence/README.md` section | Canary + two scoped scans + baseline + attestation all recorded | n/a (human attestation for revocation, as required) |
| 02-04 | Yes — pty test, `headless_loop_items.rs`, `permission-prompt/` item; **`scripts/test-gate.expected` exists but is not refreshed** | `permission_prompt_outcomes_and_warning` 1 passed; `headless_permission_denial_names_both_flags` 1 passed; `scripts/test-gate.sh --refresh-expected` **failed** (unrelated reds) | Pty test is `#[ignore = "PTY e2e; requires PAGER_BINARY"]` — recorded reason; evidence ran it with `--ignored` |
| 02-05 | Yes — `resume-fidelity/` item; `headless_resume_restores_model_effort_and_plan_state`; extended `plan_approval_resume`; user-guide paragraph | Both step logs 1 passed; `cargo fmt`/clippy claims from the summary; report 2/2 ok | `headless_loop_items` not ignored; `plan_approval_resume` not ignored; pty test ignore reason documented |
| 02-06 | Yes — `permission-mode/` and `plan-read-only/` items; three test files; `plan_mode_edit_gate_tests.rs`; `KNOWN-ISSUES.md` correction | `permission_mode_chip_matches_after_resume` 1 passed; `headless_permission_mode_plan_arms_plan_mode` 1 passed; `plan_mode_read_only_in_every_permission_mode` 1 passed; `plan_mode_edit_gate` 11 passed | Three pty tests are `#[ignore = "PTY e2e; requires PAGER_BINARY"]` — recorded; unit/integration tests not ignored |

- **Plan-level gap:** 02-04's `must_haves.artifacts` and acceptance criterion require a refreshed `scripts/test-gate.expected` (`binaries>=220`, `results>=309`). The file is still `results=308`, `binaries=219`, `doctests=89`; the plan's own assertion fails. `02-04-SUMMARY.md` and `deferred-items.md` record why (`--refresh-expected` refused on the two pre-existing `xai-grok-agent` tests), but the phase's validation file remains `draft` with the Wave-0 item unchecked. This is the only gap that keeps the phase from `passed`.
- **Protected files:** `TESTING.md`, `permission_cursor.rs`, `prompter.rs`, `19-plan-mode.md`, `26-config-reference.md` are untouched across `6e9d8d88..HEAD`; `KNOWN-ISSUES.md` has the corrected plan-mode entry and the normalizer overlap entry; the old `does not send session/set_mode` claim is gone.
- **No product code change for PERM-04:** `git log -S "apply_headless_permission_mode"` over the phase range is empty; the only `headless.rs` change is the PERM-02 denial copy in `4599b561`.

## Anti-Patterns / Overclaiming

| Item | Finding | Severity | Impact |
|---|---|---|---|
| `scripts/test-gate.expected` | Stale expected counts after a new test target; plan acceptance criterion unmet | 🛑 Gap | `scripts/test-gate.sh` full mode would fail its completeness assertion; see `gaps` |
| `permission-mode/notes.md` + test | The eprintln prints the constant `CHIP` (`"chip before quit = 'always-approve', after -r … = 'always-approve'"`), not a measured value; `contains(CHIP)` can match the resumed process's welcome frame; the committed cast's final frame is `unknown · plan` and only the initial frame carries `always-approve`; the log's `plan · always-approve` pair is not in the cast | ⚠️ Warning | PERM-03's claim is still test-asserted (the screen at assertion time contained the chip), but the artifact is weaker than the notes' "matching chip on the restored frame" wording. The notes do state the chip is a config fact, not session memory. |
| `headless_loop_items.rs` PERM-04 | The `plan_mode.json` half is conditional: if `plan_state_files` is empty the test prints and passes; the reminder assertion is the non-vacuous one. The recorded run did write `state=Active`. | ⚠️ Warning | The regression test would not fail a future run that arms the reminder but not the edit gate, despite the summary's "asserts two independent facts". The recorded evidence has both. |
| `key-leak` whole-tree scope | `--scan .` exits 1 on the scanner's own fake key and two `#[cfg(test)]` redaction strings; the item's gate is the default canary + `--scan docs/evidence` + `--scan docs .planning/…`, with the whole-tree count recorded as a measurement, not an equality gate. The item does not claim `--scan .` is green. | ⚠️ Warning | The roadmap's phrase "green over the tree" is satisfied by the default canary, not by a literal whole-tree `--scan`; no real key is present. The actual live session's saved-session store was not committed/scanned; the canary uses a fake key in an isolated HOME. |
| `02-06-SUMMARY.md` | `actuals.commits: 0` and "Working tree intentionally left uncommitted" contradict the committed `5e1e66cc` (`test(02-06): …`) and the summary's own "Commit policy" section; `plan_head_after` is `5e1e66cc`. | ℹ️ Info | Documentation inconsistency in the SUMMARY, not in the artifact; the commit exists and the evidence is committed. |
| `02-VALIDATION.md` | `status: draft`, `nyquist_compliant: false`, `wave_0_complete: false`, all per-task status boxes `⬜ pending` | ℹ️ Info | The phase validation strategy was never signed off; the Wave-0 gate item is the same stale `test-gate.expected` gap. |
| `runtime-egress` checker | A fabricated `PASS allowlisted` row is accepted (checked synthetically); the checker cannot distinguish a fabricated table row from a real one. `notes.md` states the shim's blind spots but not this checker limitation. | ℹ️ Info / limitation | The committed table is `egress-check.sh`'s output and the transcript is pinned by sha256; a reviewer re-parses, not re-measures. |
| `live-smoke` item | The key's live/throwaway status is attested by the notes and the provider traffic (`API_CONNECTS=7`, real `cache_read=44544`), not independently reproducible without the key/host; no key material is in the item. | ℹ️ Info / inherent | PROOF-01's live nature rests on the committed transcript; the revocation is PROOF-03's attestation. |

## Could Not Verify (and why)

- **The live run's key authenticity and the revocation.** No key and no build host are available here, and platform.deepseek.com exposes no revocation API. The transcript's real provider traffic and the notes' parent-shell statement are the evidence; `key-leak/notes.md` correctly records the revocation as the author's dated attestation, not a tool-verified fact.
- **The private pty binaries' provenance.** The pty steps used `/tmp/opencode/02-04/deepseek-build`, `/tmp/opencode/02-05/deepseek-build` and `/tmp/opencode/02-06/deepseek-build`; those copies are not committed and their build commit is asserted by the summaries, not derivable from the cast (the cast shows `0.0.1-rc5` only). The pty test logs and casts are the committed record.
- **The exact screen state at PERM-03's assertion.** The test calls `screen_contents()` and `contains(CHIP)` at a moment the cast does not snapshot; the cast is a raw output stream. The committed cast's final frame shows `unknown · plan`, and the initial frame carries `always-approve`. Reconstructing the exact assertion frame would require replaying the pty harness with the private binary, which is outside this read-only verification.
- **The full `scripts/test-gate.sh` run.** It would run the whole workspace suite (forbidden here and ~20–30 min), so the `results=309 / binaries=220` figure comes from `02-04-SUMMARY.md`, not a re-derivation. The stale expected file is statically confirmed: `scripts/test-gate.expected` has 308/219/89, and the plan's assertion requires `binaries>=220` / `results>=309`.
- **The actual live session's saved files.** They live in the build host's HOME and were not committed; the key-leak canary proves the product's write path with a fake key in an isolated HOME, and the committed evidence tree is clean. The real key is not present in any committed file.
- **The pty casts' input keys.** Asciinema records output, not the injected input; the three outcomes and the chip selections are proven by the pty test's assertions and tool results, not by reading the cast's absent input records.

## Weaknesses (Top Three)

1. **The phase's test gate is stale (the only blocker).** `scripts/test-gate.expected` still says `results=308 / binaries=219 / doctests=89` while 02-04 added `headless_loop_items`; 02-04's acceptance criterion requires `binaries>=220 / results>=309`, and `scripts/test-gate.sh` full mode would fail. The plan's `--refresh-expected` was correctly refused on unrelated red `xai-grok-agent` tests, but the phase is marked 6/6 and `02-VALIDATION.md` remains draft. Fix: get a green run, refresh with the script (not by hand), or re-defer with an explicit owner.
2. **PERM-03's chip evidence is weaker than the summary's wording.** The log's "chip before/after = 'always-approve'" is the `CHIP` constant, not a measured string; `contains(CHIP)` can match the resumed process's welcome frame; the committed cast's final frame is `unknown · plan`, and only the initial frame carries `always-approve`. The pty test's `transcript_restored=true` does prove the resumed session; the chip-on-the-restored-frame claim is not cleanly visible in the cast. The notes do admit the chip is a config fact.
3. **The whole-tree key-leak claim is scoped, not literal.** `--scan .` exits 1 (the scanner's own canary + two `#[cfg(test)]` redaction strings); the item's green claims are the default canary and the two scoped `--scan` runs, with the whole-tree count as a measurement. That is defensible and well documented, but the roadmap's "green over the tree" is not a literal `--scan .` green, and the actual live session's saved-session store was not committed/scanned. The revocation is an attestation by design, correctly not presented as verified.

## Gaps Summary

All 11 requirement IDs (PROOF-01/02/03, PERM-01…04, SESS-01…04) are proven by committed artifacts, and all five roadmap success criteria are met. The phase is `gaps_found` only because one plan-level/validation acceptance criterion is not met: `scripts/test-gate.expected` was never refreshed after 02-04 added the `headless_loop_items` test target. The stale file is a concrete, reproducible failure of 02-04's own declared assertion (`binaries>=220`, `results>=309`) and of `02-VALIDATION.md`'s Wave-0 requirement; it does not affect the loop-item evidence. A green full run followed by `scripts/test-gate.sh --refresh-expected`, or an explicit re-deferral with an owner and a later phase, closes the gap.

---

_Verified: 2026-11-25T23:25:03Z_
_Verifier: the agent (gsd-verifier)_
