---
gsd_state_version: '1.0'
status: planning
progress:
  total_phases: 5
  completed_phases: 0
  total_plans: 22
  completed_plans: 0
  percent: 0
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-11-21)

**Core value:** One DeepSeek key in, a trustworthy daily coding agent out: the right model on every turn, zero traffic to any host you did not configure, and a release you can install with confidence.
**Current focus:** Phase 1 — Truth Baseline

## Current Position

Phase: 1 of 5 (Truth Baseline)
Plan: 0 of 5 in current phase
Status: Ready to plan
Last activity: 2026-11-21 — Roadmap revised: routing and per-model cost cut to v2; 5 phases, 41/41 requirements mapped

Progress: [░░░░░░░░░░] 0%

## Performance Metrics

**Velocity:**
- Total plans completed: 0
- Average duration: —
- Total execution time: 0 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| - | - | - | - |

**Recent Trend:**
- Last 5 plans: —
- Trend: —

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

### Pending Todos

None yet.

### Blockers/Concerns

- **Acceptance gate (human, blocking):** ACC-01 needs a 20–30 minute TUI session on a real repo clone by the author. FREEZE-01/02/03 must not run until the signed note is committed — this is the milestone's only human-blocking dependency.
- **Snapshot tag is public:** `deepseek-v0.1-snapshot` already exists on origin at `4b305bd0`. It must be recorded as superseded in docs and left untouched; moving or deleting it is destructive.
- **Throwaway key required for Phase 2:** the live smoke, egress measurement and key-leak scan need a `DEEPSEEK_API_KEY` that is revoked after the run. Without it, PROOF-01/02/03 are blocked, not retried.
- **Research base drift:** STACK/ARCHITECTURE read `main` @ `221a1a59`, PITFALLS @ `9e6780d9`. Re-verify `release.yml` edges and manifest versions before Phase 3.
- **macOS time budget (MEDIUM):** measure the macOS job log before choosing between timeout, strip and LTO override. Both arches must fit the job timeout; one cache-warming run is allowed if cold cache is the cause.
- **Dry-run side effects:** a dry run is a real push. `container` currently depends on the whole matrix and the metadata-action tag list has no empty-list guard, so Phase 3 must land before any throwaway tag, and the dispatch verification must create no release and move no `:latest`.
- **No `ci.yml`:** the gate is a green run on the author's host, wrapped in the shared build lock with `scripts/disk-guard.sh` first.

## Deferred Items

| Category | Item | Status | Deferred At | Milestone |
|----------|------|--------|-------------|-----------|
| Routing | ROUT-V2-01…10 — `route_turn`, `[models] routing`, pins, `Route` item, `/route`, provider-agnostic registry | Tracked in REQUIREMENTS.md §v2 | 2026-11-21 | post-freeze (open-build) |
| Cost | COST-V2-01…04 — per-model session pricing, `/usage` spend, off-peak switch, bucket disjointness | Tracked in REQUIREMENTS.md §v2 | 2026-11-21 | post-freeze |
| Pipeline | REL-V2-01/02 — `ci.yml`, debug symbol sidecars | Tracked in REQUIREMENTS.md §v2 | 2026-11-21 | v0.1.x |
| Product | PROD-V2-01/02 — undo/checkpoint product, automatic Flash-for-research subagents | Tracked in REQUIREMENTS.md §v2 | 2026-11-21 | v0.1.x |
| Naming | ~435 internal `GROK_*` env names; `xai-grok-*` crate/package names | Out of scope (D2) | 2026-11-21 | open-build |
| Release | Publishing `v0.1.0` itself | Author publishes it | 2026-11-21 | after freeze |

## Session Continuity

Last session: 2026-11-21
Stopped at: Roadmap revised to 5 phases; Phase 1 ready to plan
Resume file: None
