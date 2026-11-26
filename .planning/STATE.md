---
gsd_state_version: "1.0"
milestone: v0.1.0
current_phase: 4
current_phase_name: Distribution Proof
status: executing
stopped_at: Phase 4 plan 04-01 complete (1/5) — v0.1.0-proof1 is pushed tag-only at b14d28eb; run 45724842211 is green (7/7 jobs, macOS 47.8/61.5 min, container success, two prerelease image tags, no :latest) and the release carries exactly six verified assets; next is 04-02 (installer proof)
last_updated: "2026-11-26T06:10:00.000Z"
last_activity: 2026-11-26
last_activity_desc: 04-01 recorded the single throwaway tag push, the green seven-job run, the six-asset pre-release and the absent :latest; REL-02 and the macOS half of REL-03 closed; REL-04 stays Partial (pull environment-blocked, 04-04)
state_head: 48dec6fcd58cdea05dabf6bd475e13c30dafa675
progress:
  total_phases: 6
  completed_phases: 3
  total_plans: 21
  completed_plans: 17
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-11-23)

**Core value:** One DeepSeek key in, a trustworthy daily coding agent out: the right model on every turn, zero traffic to any host you did not configure, and a release you can install with confidence.
**Current focus:** Phase 4 — Distribution Proof

## Current Position

Phase: 4 — Distribution Proof
Plan: 04-01 of 5 complete (1/5). Next is 04-02 (installer proof against the real release).
Status: Plan 04-01 complete — `v0.1.0-proof1` was pushed **tag-only** at `b14d28eb` and produced a green seven-job run (`45724842211`): both macOS legs at 47.8/61.5 min against the 240 min budget with sign+verify steps green (**REL-02 and the macOS half of REL-03 closed**), `container` success pushing only `0.1.0-proof1` + `v0.1.0-proof1` with no `:latest`, and a pre-release with exactly six assets whose bytes match both `SHA256SUMS` and GitHub's digests. REL-04 stays **Partial**: built/published/registry-verified, but the pull is environment-blocked (no Docker daemon) and belongs to 04-04.
Last activity: 2026-11-26 06:07 UTC — 04-01 added `scripts/distribution-proof.py` and the three Phase-4 evidence items.

Progress: [████████░░] 81%

## Performance Metrics

**Velocity:**
- Total plans completed: 17
- Average duration: —
- Total execution time: 0 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 01 | 6 | 6 | - |
| 02 | 6 | 6 | - |
| 03 | 4 | 4 | - |
| 04 | 1 | 5 | - |

**Recent Trend:**
- Last 5 plans: —
- Trend: —

*Updated after each plan completion*
**Per-Plan Metrics:**

| Plan | Duration | Tasks | Files |
|------|----------|-------|-------|
| Phase 04 P01 | 10m | 3 tasks | 38 files |
| Phase 03 P04 | 14m | 2 tasks | 5 files |
| Phase 03 P03 | 5m | 2 tasks | 4 files |
| Phase 03 P02 | 12m | 2 tasks | 7 files |
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
- [Phase 03]: Version truth is `[workspace.package] version = "0.1.0"`; only `xai-grok-shell`, `xai-grok-pager`, `xai-grok-pager-bin` and `xai-grok-version` inherit it, and the other literal-version manifests stay literal on purpose. The lock diff is exactly those four package versions.
- [Phase 03]: `--version` is proven on both paths by `crates/codegen/xai-grok-pager-bin/tests/version_paths.rs` — `0.1.0` with `GROK_VERSION` unset, `9.9.9-test` with it set (a real rebuild, so the stamp path cannot pass by inheriting the source value). The test also names `"0.1.0"` so a silent revert fails.
- [Phase 03]: The 0.1.0 bump re-keys `Swatinem/rust-cache` (manifest/lock hash), so the Phase 4 tag run starts cold on all four legs. Expected, not a defect.
- [Phase 03]: `release-dist` ships no DWARF: `debug = false` + link-time `strip = "debuginfo"` (symbol table kept, so crash-handler backtraces still name frames), `split-debuginfo = "off"` retained. Proven on a real build (0 `.debug` / 1 `.symtab`); an external strip after signing is forbidden.
- [Phase 03]: `build-macos` ad-hoc signs (`codesign --force --sign -`) after the build and verifies the **extracted tarball** — `codesign --verify --strict`, a `Signature=` line, both `--version` needles, no `__DWARF`, no `*.dSYM` — before the upload. The pre-tar smoke stays as the faster failure.
- [Phase 03]: macOS budget is 240 minutes and Linux stays 90; no `CARGO_PROFILE_*` override. If 240 is still too small, `CARGO_PROFILE_RELEASE_DIST_LTO=off` + `CARGO_PROFILE_RELEASE_DIST_CODEGEN_UNITS=16` on `build-macos` only is the pre-decided next lever.
- [Phase 03]: `check-release-workflow.py` is 19/19 (A16 sign ordering, A17 extracted-artifact verify, A18 timeouts, A19 no profile override) plus a 5-case Linux fixture that runs the verify step's real body against stub `codesign`/`otool`. REL-02 and the CI half of REL-03 remain open until the Phase 4 tag run.
- [Phase 03]: The `v0.1.0` tag shape is proven against the workflow's own derive body, extracted and executed at test time: `v0.1.0` -> `version=0.1.0`/`is_prerelease=false`, `v0.1.0-rc1` and `v0.1.0-proof1` -> prerelease, branch -> `0.0.0-<sha7>`, and `v0.1`/`v1`/`v0.1.0.0`/`v0.1.0+build.5`/`1.0.0` all rejected non-zero with a named error. `scripts/check-release-tag-shape.py` is 17/17.
- [Phase 03]: `v0.1.0-proof1` is the Phase 4 throwaway tag: covered in the derive matrix, the image-tag guard fixture (exactly `{0.1.0-proof1, v0.1.0-proof1}`, no `latest`) and the installer harness (`deepseek-build-0.1.0-proof1-<target>.tar.gz`).
- [Phase 03]: `install.sh`'s `VERSION=0.1` leniency is recorded, not fixed — it passes the `v[0-9]*` prefix check and then 404s; the `version` job's semver gate makes that shape unreachable for a real release. `install.sh` is byte-unchanged and the installer harness is 58/58.
- [Phase 03]: `scripts/check-release-pipeline.sh` is the one-command static gate (actionlint, shellcheck, workflow checker 19/19 + fixture 5/5, guard 20/20, tag-shape 17/17, installer 58/58) with every tally pinned, so a shrunken checker fails the gate.
- [Phase 04]: The one tag is `v0.1.0-proof1`, pushed **tag-only** at `b14d28eb`; `origin/main` (`6e9d8d88…`) and `deepseek-v0.1-snapshot` (`4b305bd0…`) are unchanged and no `v0.1.0` tag/release exists. The release is kept (CONTEXT.md decision) — deletion is recorded as a deviation in 04-05.
- [Phase 04]: The pinned `docker/metadata-action` collapses a pre-release's tag list to **two** names (`0.1.0-proof1`, `v0.1.0-proof1`), not four; the guard accepts both shapes. Assert pushed names from the container log's `pushing manifest` lines, never from the workflow patterns.
- [Phase 04]: The multi-arch index digest is `sha256:8f5c484e…`; the platform manifests inside it are `1d216b60…` (linux/amd64) and `25dc0d25…` (linux/arm64). The `build-push-action` outputs (`7f33b41b…`, `85f9de25…`) are buildkit per-platform **indexes** — don't mix the pairs.
- [Phase 04]: Registry reads are daemonless: an anonymous ghcr.io pull token over HTTPS, or `docker manifest inspect`. `gh api /users/…/packages/…` 403s on this token (`read:packages` missing) — that is an auth gap, not a missing image.
- [Phase 04]: A release is trusted only on two independent digests: the release's own `SHA256SUMS` **and** GitHub's asset `digest`. Downloaded tarballs stay outside the repo; evidence is text-only (a `-size +1M` gate enforces it).

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
- **REL-05 / REL-04 pull — environment block (2026-11-26):** this host has the `docker` CLI but **no daemon** (`Cannot connect to the Docker daemon`). The image is proven registry-side; pulling and running it (REL-05, and REL-04's pulled half) is deferred to 04-04's exact-command record and must never be claimed as executed here.
- **README baseline drift:** `README.md` no longer contains "not codesigned and not notarized" (0 matches; 03-03 wrote the ad-hoc-signed truth). 04-02/04-03 must re-measure, not assert the old value of 1.
- **macOS time budget — mitigation landed, proof pending (2026-11-25, 03-03):** `build-macos` now carries 240 minutes (Linux 90) and `release-dist` no longer emits DWARF; no profile override was added. REL-02 still needs the Phase 4 tag run: read both macOS job durations there, and if 240 is still not enough apply `CARGO_PROFILE_RELEASE_DIST_LTO=off` / `CARGO_PROFILE_RELEASE_DIST_CODEGEN_UNITS=16` to `build-macos` only.
- **Dry-run side effects:** a dry run is a real push. 03-01 removed both known silent paths (the image no longer waits for macOS; the tag list is guarded and the version job rejects non-semver tags). 03-04 closed the tag-shape half locally (`v0.1.0-proof1` is pre-verified as a prerelease with no `:latest`, and the installer URL shape is exercised), so the Phase 4 tag run is the first remaining real-world risk. The dispatch verification must still create no release and move no `:latest`, and no throwaway tag may be created until Phase 4.
- **`--locked` lock prerequisite — RESOLVED (2026-11-25, 03-02):** both build jobs pass `--locked`; 03-02 regenerated `Cargo.lock` for the `0.1.0` workspace version (exactly four package versions) and `cargo check --workspace --locked` is green, so the Phase 4 tag run will not fail on a stale lock. REL-02 (both macOS legs inside the budget) remains open until that tag run.
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
Stopped at: Phase 3 plan 03-04 complete (4/4) — the `v0.1.0` tag shape is proven against `release.yml`'s own derive body (17/17), the installer harness is 58/58 incl. `0.1.0`/`0.1.0-proof1`, the guard fixture is 20/20, and `sh scripts/check-release-pipeline.sh` runs every static gate at pinned tallies. Next: Phase 4 (`04-01`); REL-02 and the macOS half of REL-03 need the Phase 4 tag run.
Resume file: .planning/phases/03-pipeline-decoupling-version/03-04-SUMMARY.md
