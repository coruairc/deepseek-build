---
gsd_state_version: "1.0"
milestone: v0.1.0
current_phase: 05
current_phase_name: Documentation Truth & Freeze
status: blocked
stopped_at: Milestone audit NOT CERTIFIED — ACC-01, throwaway key, and tag permission are owner-gated
last_updated: "2026-11-22T07:10:00.000Z"
last_activity: 2026-11-22
last_activity_desc: Phase 05 docs complete; acceptance/freeze blocked; audit recorded
state_head: 66ce4c7d955ea6dadccd18164f09337f62316fc1
progress:
  total_phases: 5
  completed_phases: 4
  total_plans: 22
  completed_plans: 20
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-11-21)

**Core value:** One DeepSeek key in, a trustworthy daily coding agent out: the right model on every turn, zero traffic to any host you did not configure, and a release you can install with confidence.
**Current focus:** Phase 05 — Documentation Truth & Freeze

## Current Position

Phase: 05 (Documentation Truth & Freeze) — BLOCKED at 05-04
Plan: 3 of 5 (05-01..03 complete; 05-04/05-05 blocked)
Status: Blocked on owner decisions — see Needs Human
Last activity: 2026-11-22 — docs complete; acceptance and freeze blocked

Progress: [█████████░] 90%

## Performance Metrics

**Velocity:**
- Total plans completed: 20 of 22 (05-04/05-05 blocked)
- Average duration: — (no timing captured under the skip-testing run)
- Total execution time: —

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 01 Truth Baseline | 5 | 5 | long-running; tests deferred |
| 02 Loop-Item Verification | 5 | 5 | scaffolding only; runs deferred |
| 03 Pipeline Decoupling & Version | 4 | 4 | code/config only; CI deferred |
| 04 Distribution Proof | 3 | 3 | docs only; proof blocked (never push/tag) |
| 05 Documentation Truth & Freeze | 3 | 5 | docs complete; 05-04/05-05 blocked (ACC-01, never tag) |

**Recent Trend:**
- Last 5 plans: 05-01…05-03 docs; 05-04/05-05 blocked; 04-01…03 docs/scaffolding; 03-01…04 code/config
- Trend: no execution proofs; documentation and code landed, tests deferred

*Updated after each plan completion*

## Accumulated Context

### Decisions

Full log in PROJECT.md Key Decisions. Those shaping current work:

- Routing is cut from this milestone entirely — `route_turn`, `[models] routing`, session pins, the `Route` status-line item, `/route` and per-model cost are v2. `/model` and `/effort` are the manual switches. Do not wire, display, or price routing in any phase.
- What survives from the cost work is the honesty bug fix: `deepseek-v4-flash` priced in the table, the TUI figure labelled an estimate, headless honouring `[ui.status_line.pricing]`.
- `deepseek-flash` becomes the catalog default, with the at-ceiling caveat (63/63 both models) and the revert path (`/model deepseek-v4-pro`, `-m deepseek-v4-pro`) documented.
- Evidence per loop item = asciinema cast + `report.json` under `docs/evidence/`, absolute paths normalized. Phase 1 defines the convention; Phase 2 is its first consumer.
- The sandbox fix comes before the clean-suite gate: only the product fix may zero the bwrap pair — never a skip, an `#[ignore]`, or a widened predicate.
- Publish tag is `v0.1.0` (no `deepseek-*` trigger added); the existing `deepseek-v0.1-snapshot` tag at `4b305bd0` is superseded in docs, never moved. This milestone does not publish `v0.1.0`.
- The freeze marker is non-triggering, and the author's signed acceptance note must be committed before branch `deepseek` or the marker tag exists.
- Container keeps the sandbox on, with documented run flags; no new outbound destination anywhere.
- The ~435 internal `GROK_*` env names and the `xai-grok-*` crate names are out of scope for this milestone (D2); renaming belongs to open-build.
- **Parallel lanes from Phase 2 (user decision, 2026-11-21):** Phase 1 finishes as one sequential lane.
  From Phase 2 onward, plans with disjoint `files_modified` may run in separate git worktrees under one
  shared `CARGO_TARGET_DIR`, one `flock` build lock, and one merge step per wave. This overrides the
  default "one implementer, one working tree" rule for this project; dependent plans (live-session
  proof, artifact builds, freeze) stay sequential. No plan is parallelized with a direct dependency.

### Pending Todos

None yet.

### Blockers/Concerns

Owner-gated (three; nothing else blocks the milestone):

- **ACC-01 — acceptance session (human, blocking).** A 20–30 minute TUI session
  on a clone of a real repo, then a signed note committed. FREEZE-01/02/03 must
  not run before it. The checklist is
  `.planning/phases/05-documentation-truth-freeze/05-04-ACCEPTANCE-CHECKLIST.md`.
- **A throwaway `DEEPSEEK_API_KEY`.** Needed for PROOF-01/02/03 and the runtime
  egress measurement; revoke after the run.
- **Tag-push permission.** REL-04/05/06 and any release dry run need a real
  pushed throwaway tag; this run is never-push/never-tag.

Directed deferral:

- **Tests are deferred milestone-wide (owner decision, 2026-11-22).** No test,
  gate, smoke, egress or release-build run is performed; the current pass allows
  no cargo command at all. Every verification step is recorded as
  `DEFERRED (tests skipped by owner, 2026-11-22)`.

Resolved in this pass (kept so the record is not stale):

- **Egress fixtures (was a FREEZE-03 blocker).** The three HARD hits are fixed in
  source without weakening the scan: the rejected-URL fixtures are built from
  fragments and the legacy bucket value is `local-bucket`. Re-running the gate is
  deferred.
- **Credential-test key print.** `process_key_from_model_env_key` now unsets both
  key variables and asserts without debug-formatting the key.
- **Container/dry-run shape.** `container` needs `build-linux` only and the
  tag-list guard exists, so a slow macOS leg can no longer skip the image.
- **Snapshot tag.** `deepseek-v0.1-snapshot` at `4b305bd0` is superseded and must
  not be moved; recorded in `KNOWN-ISSUES.md`.

Remaining non-owner debt:

- **`cargo fmt` was not run** on the 2026-11-22 source edits (no cargo allowed);
  formatting is by hand.
- **macOS time budget** is unmade: it needs a measured job log.
- **No `ci.yml`**: the gate remains a host-run script.
- **Formatting/parent drift:** the planning research read `main` @ `221a1a59`;
  re-read `release.yml`/manifests before any future phase.

## Deferred Items

| Category | Item | Status | Deferred At | Milestone |
|----------|------|--------|-------------|-----------|
| Routing | ROUT-V2-01…10 — `route_turn`, `[models] routing`, pins, `Route` item, `/route`, provider-agnostic registry | Tracked in REQUIREMENTS.md §v2 | 2026-11-21 | post-freeze (open-build) |
| Cost | COST-V2-01…04 — per-model session pricing, `/usage` spend, off-peak switch, bucket disjointness | Tracked in REQUIREMENTS.md §v2 | 2026-11-21 | post-freeze |
| Pipeline | REL-V2-01/02 — `ci.yml`, debug symbol sidecars | Tracked in REQUIREMENTS.md §v2 | 2026-11-21 | v0.1.x |
| Product | PROD-V2-01/02 — undo/checkpoint product, automatic Flash-for-research subagents | Tracked in REQUIREMENTS.md §v2 | 2026-11-21 | v0.1.x |
| Naming | ~435 internal `GROK_*` env names; `xai-grok-*` crate/package names | Out of scope (D2) | 2026-11-21 | open-build |
| Release | Publishing `v0.1.0` itself | Author publishes it | 2026-11-21 | after freeze |

## Deferred Execution (owner, 2026-11-22)

| Area | State | Why |
|------|-------|-----|
| Phase 1 test gate + evidence runs | DEFERRED | tests skipped by owner |
| Phase 2 live smoke, egress, key-leak, permission/resume verification | DEFERRED | tests skipped + no throwaway key |
| Phase 3 CI runs (macOS budget, dispatch) | DEFERRED | no push/CI run allowed; workflow written only |
| Phase 4 tag dry run, GHCR pull, installer proof | BLOCKED | owner rule: never push or tag |
| Phase 5 acceptance session, marker tag | BLOCKED | owner decision (ACC-01) + never tag |

## Needs Human

| Phase | State | Resume |
|-------|-------|--------|
| 05 | needs_human — ACC-01 (author's 20–30 min TUI session + signed note) | commit the note, then `/gsd-autonomous --from 05` |
| 02/05 | needs_human — throwaway `DEEPSEEK_API_KEY` for PROOF-01/02/03 and the runtime egress measurement | provide a revocable key, then re-run Phase 2 |
| 04 | blocked — tag-push permission for the distribution dry run (REL-04/05/06) | grant it, then `/gsd-autonomous --from 04` |

## Session Continuity

Last session: 2026-11-22
Stopped at: all non-owner work complete; three owner gates remain (ACC-01, throwaway key, tag permission)
Resume file: none needed — this file plus `.planning/v0.1.0-MILESTONE-AUDIT.md` are authoritative. The 2026-11-21 `.continue-here.md` pause snapshot and the old `.planning/HANDOFF.json` are superseded.
