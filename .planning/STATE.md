---
gsd_state_version: "1.0"
milestone: v0.1.0
current_phase: 3
current_phase_name: Pipeline Decoupling & Version
status: executing
stopped_at: Phase 3 plan 03-01 complete (1/4) — build matrix split and tag guard landed; REL-02 open until the Phase 4 tag run
last_updated: "2026-11-25T23:27:22.000Z"
last_activity: 2026-11-25
last_activity_desc: 03-01 split build-linux/build-macos, added the workflow checker and the image tag-list guard
state_head: ba3b49125154f4c8c5d3fb8879180898d40ddc82
progress:
  total_phases: 6
  completed_phases: 2
  total_plans: 21
  completed_plans: 13
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-11-23)

**Core value:** One DeepSeek key in, a trustworthy daily coding agent out: the right model on every turn, zero traffic to any host you did not configure, and a release you can install with confidence.
**Current focus:** Phase 3 — Pipeline Decoupling & Version

## Current Position

Phase: 3 — Pipeline Decoupling & Version
Plan: 03-01 of 4 complete; next is 03-02 (workspace version + `Cargo.lock` regeneration).
Status: Plan 03-01 complete — checker 15/15, guard fixtures 18/18, actionlint and shellcheck clean. REL-02 is still open (Phase 4 tag run).
Last activity: 2026-11-25 23:26 UTC — 03-01 split `build-linux`/`build-macos` and guarded the image tag list.

Progress: [██████░░░░] 62%

## Performance Metrics

**Velocity:**
- Total plans completed: 13
- Average duration: —
- Total execution time: 0 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 01 | 6 | 6 | - |
| 02 | 6 | 6 | - |
| 03 | 1 | 4 | - |

**Recent Trend:**
- Last 5 plans: —
- Trend: —

*Updated after each plan completion*
**Per-Plan Metrics:**

| Plan | Duration | Tasks | Files |
|------|----------|-------|-------|
| Phase 03 P01 | 5m | 2 tasks | 5 files |
| Phase 02 P04 | 102m | 3 tasks | 14 files |
| Phase 02 P05 | 35m | 2 tasks | 9 files |
| Phase 02 P06 | 55m | 2 tasks | 19 files |
| Phase 02 P03 | 4m | 2 tasks | 11 files |

## Accumulated Context

### Decisions

Full log in PROJECT.md Key Decisions. Those shaping current work:

- Routing is cut from this milestone entirely — `route_turn`, `[models] routing`, session pins, the `Route` status-line item, `/route` and per-model cost are v2. `/model` and `/effort` are the manual switches. Do not wire, display, or price routing in any phase.
- What survives from the cost work is the honesty bug fix: `deepseek-v4-flash` priced in the table, the TUI figure labelled an estimate, headless honouring `[ui.status_line.pricing]`.
- `deepseek-flash` becomes the catalog default, with the at-ceiling caveat (63/63 both models) and the revert path (`/model deepseek-v4-pro`, `-m deepseek-v4-pro`) documented.
- Evidence per loop item = asciinema cast + `report.json` under `docs/evidence/`, absolute paths normalized. Phase 1 shipped the convention and four items (ipv6, test-gate, cost-estimate, determinism); Phase 2 is its first external consumer.
- **Determinism claims need an artifact-backed repetition:** a reported "5 runs, no rotation" was falsified by the Phase-1 verifier (2 failures in 35 runs). Fixed with a bounded 5 s/20 ms process-death poll and a committed 35/35 repetition item. Never accept a flake claim without a committed projection log — Phase 2's live/egress proofs inherit this standard.
- The sandbox fix comes before the clean-suite gate: only the product fix may zero the bwrap pair — never a skip, an `#[ignore]`, or a widened predicate.
- Publish tag is `v0.1.0` (no `deepseek-*` trigger added); the existing `deepseek-v0.1-snapshot` tag at `4b305bd0` is superseded in docs, never moved. This milestone does not publish `v0.1.0`.
- The freeze marker is non-triggering, and the author's signed acceptance note must be committed before branch `deepseek` or the marker tag exists.
- Container keeps the sandbox on, with documented run flags; no new outbound destination anywhere.
- The ~435 internal `GROK_*` env names and the `xai-grok-*` crate names are out of scope for this milestone (D2); renaming belongs to open-build.
- **Phase 2 live-proof key (user decision, 2026-11-23):** use the existing `DEEPSEEK_API_KEY` from the
  environment for PROOF-01/02/03; the author rotates/revokes it on platform.deepseek.com after the run
  (this also resolves the Phase-1 local-log exposure). The key is treated as compromised-by-policy:
  no key material in any committed artifact, and PROOF-03's revocation step is satisfied by the
  author's confirmed rotation, recorded as evidence.
- **Parallel lanes from Phase 2 (user decision, 2026-11-21):** Phase 1 finishes as one sequential lane.
  From Phase 2 onward, plans with disjoint `files_modified` may run in separate git worktrees under one
  shared `CARGO_TARGET_DIR`, one `flock` build lock, and one merge step per wave. This overrides the
  default "one implementer, one working tree" rule for this project; dependent plans (live-session
  proof, artifact builds, freeze) stay sequential. No plan is parallelized with a direct dependency.
- [Phase 02]: Permission warning is the wire title's first line, painted above the question; chrome height grows by one.
- [Phase 02]: Both headless denial lines are composed from PERMISSION_DENIAL_FLAGS naming --always-approve and --permission-mode auto.
- [Phase 02]: Pty session-grant proof uses ask mode without Bash(*) ask, because that rule skips the session-grant short-circuit.
- [Phase 02]: Resume proofs read request bodies, not logs: one TestSandbox plus run_headless_in_sandbox_borrowed for every invocation; the sandbox config.toml must declare a [model.*] entry because the mock catalog is not fetched.
- [Phase 02]: A resumed plan turn injects the sparse reminder (reminder_count is odd); the permission chip after resume is a config fact, not session memory — plan state and plan.md come from plan_mode.json/plan.md.
- [Phase 02]: PERM-03 resumes by the session id read from `<grok home>/sessions/*/*` with no `--permission-mode` on either launch; `--continue` would prove recency, not the named session.
- [Phase 02]: PERM-04 asserts the reminder sentence in the captured request body **and** `plan_mode.json` state=Active; the reminder alone does not prove the edit gate armed.
- [Phase 02]: A failed edit block finishes collapsed, so a pty test must expand the row (real SGR double-click) before asserting a plan-mode refusal on screen; parse paths from the unwrapped tool result, never from wrapped screen text.
- [Phase 02]: `GROK_PTY_CAST_DIR` must be absolute: cargo runs the test binary with its crate directory as cwd, so a repo-relative cast dir writes beside the crate.
- [Phase 02]: The key-leak gate is the default-mode canary (product writes no key material; 52 saved files) plus scoped `--scan` over the committed artifacts, which fails closed. The tracked-file `sk-`-shape baseline (2 files / 4 lines at HEAD `eef4714e`) is recorded as a measurement with no equality gate.
- [Phase 02]: The canary's own `run exit:` cannot distinguish a provider 401 from a dead network, so the provider-path step records a same-conditions fake-key probe that observed `status=401 Unauthorized` from `api.deepseek.com`.
- [Phase 02]: PROOF-03's revocation is an author attestation: one dated sentence with no key fragment or account identifier, plus a step proving only that the sentence is present. Nothing in the item claims a tool verified the revocation.
- [Phase 03]: `build-linux`/`build-macos` are separate jobs; `publish` needs both (its verify step names four tarballs) and `container` needs `build-linux` only, so a slow macOS leg cannot erase the image. Artifact names stay `release-<target>`.
- [Phase 03]: Both CI builds pass `--locked`; 03-02's regenerated `Cargo.lock` must land before any tag or the build fails on a stale lock. No workflow runs in Phase 3.
- [Phase 03]: The image-tag guard requires all five stable aliases and accepts both prerelease shapes (2-name collapsed, 4-name expanded); `latest` is forbidden on a prerelease. `{{major}}` = `0` for `v0.1.0` is published, not suppressed.

### Pending Todos

None yet.

### Blockers/Concerns

- **Phase 2 host block — cleared on resume (2026-11-25):** the 2026-11-23 block was the remote
  Alpine box (no toolchain, 31 GB free, no key). This resume is on the author's build host:
  `rustc 1.94.0`, `scripts/disk-guard.sh` exit 0 (961 GB free). 02-01 has since executed.
  The inherited key was rejected; the login-shell key probed `models_ok` and is the one to
  rotate after Phase 2. The value was not read.
- **Acceptance gate (human, blocking):** ACC-01 needs a 20–30 minute TUI session on a real repo clone by the author. FREEZE-01/02/03 must not run until the signed note is committed — this is the milestone's only human-blocking dependency.
- **Snapshot tag is public:** `deepseek-v0.1-snapshot` already exists on origin at `4b305bd0`. It must be recorded as superseded in docs and left untouched; moving or deleting it is destructive.
- **Throwaway key for Phase 2 — RESOLVED (2026-11-23):** the author chose to use the existing env
  `DEEPSEEK_API_KEY` and rotate it after the run (see Decisions). PROOF-01/02/03 are unblocked.
  The author confirmed the rotation/revocation on 2026-11-25; 02-03 records it as a dated
  attestation (no API can verify a revocation).
- **Research base drift:** STACK/ARCHITECTURE read `main` @ `221a1a59`, PITFALLS @ `9e6780d9`. Re-verify `release.yml` edges and manifest versions before Phase 3.
- **macOS time budget (MEDIUM):** measure the macOS job log before choosing between timeout, strip and LTO override. Both arches must fit the job timeout; one cache-warming run is allowed if cold cache is the cause.
- **Dry-run side effects:** a dry run is a real push. 03-01 removed both known silent paths (the image no longer waits for macOS; the tag list is guarded and the version job rejects non-semver tags). The dispatch verification must still create no release and move no `:latest`, and no throwaway tag may be created until Phase 4.
- **`--locked` lock prerequisite:** both build jobs now pass `--locked`. 03-02 (workspace `0.1.0` + four inheriting crates) must regenerate `Cargo.lock` and land on `main` before the Phase 4 tag, or the build fails on the stale lock. REL-02 (both macOS legs inside the budget) remains open until that tag run.
- **No `ci.yml`:** the gate is a green run on the author's host, wrapped in the shared build lock with `scripts/disk-guard.sh` first.
- **Egress gate is red at base (Phase 5 owner):** `scripts/check-egress.sh --strict` exits 1 on three
  committed test fixtures — `crates/codegen/xai-grok-shell-base/src/util/mod.rs:422-423`
  (`api.x.ai`, `grok.com`) and `crates/codegen/xai-grok-shell/src/agent/config_tests.rs:2768`
  (`gs://bucket`). Not caused by Phase 1; it must be green for FREEZE-03.
- **Credential test prints an exported key (hygiene):** with `DEEPSEEK_API_KEY` exported, an
  `xai-grok-shell` credential assertion prints the key in its failure message. It reached only a local
  log, which was redacted; nothing was committed. The gate now unsets the key vars; the assertion should
  be redacted too. The key used for the Phase-2 live runs is rotated/revoked per the author's
  02-03 attestation.

## Deferred Items

| Category | Item | Status | Deferred At | Milestone |
|----------|------|--------|-------------|-----------|
| Routing | ROUT-V2-01…10 — `route_turn`, `[models] routing`, pins, `Route` item, `/route`, provider-agnostic registry | Tracked in REQUIREMENTS.md §v2 | 2026-11-21 | post-freeze (open-build) |
| Cost | COST-V2-01…04 — per-model session pricing, `/usage` spend, off-peak switch, bucket disjointness | Tracked in REQUIREMENTS.md §v2 | 2026-11-21 | post-freeze |
| Pipeline | REL-V2-01/02 — `ci.yml`, debug symbol sidecars | Tracked in REQUIREMENTS.md §v2 | 2026-11-21 | v0.1.x |
| Product | PROD-V2-01/02 — undo/checkpoint product, automatic Flash-for-research subagents | Tracked in REQUIREMENTS.md §v2 | 2026-11-21 | v0.1.x |
| Naming | ~435 internal `GROK_*` env names; `xai-grok-*` crate/package names | Out of scope (D2) | 2026-11-21 | open-build |
| Release | Publishing `v0.1.0` itself | Author publishes it | 2026-11-21 | after freeze |
| Review | IN-07 — default-mode `key-leak-check.sh` fail-open on grep rc≥2 (pre-existing; default mode frozen by plan 01-05 task 3(b); `--scan` fails closed) | Ledgered in KNOWN-ISSUES.md | 2026-11-23 | v0.1.x |

## Session Continuity

Last session: 2026-11-25
Stopped at: Phase 3 plan 03-01 complete (1/4) — release graph split, tag-list guard and semver gate landed and fixture-proven; no tag, push or dispatch was created. Next: 03-02 (workspace version + `Cargo.lock`).
Resume file: .planning/phases/03-pipeline-decoupling-version/03-01-SUMMARY.md
