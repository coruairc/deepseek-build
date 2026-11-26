---
gsd_state_version: "1.0"
milestone: v0.1.0
current_phase: 4
current_phase_name: Distribution Proof
status: executing
stopped_at: Phase 4 plan 04-04 complete (4/5) — the GHCR image is proven registry-side from the host (anonymous HTTPS read: OCI index sha256:8f5c484e… == the CI digest, linux/amd64 sha256:1d216b60… + linux/arm64 sha256:25dc0d25…, exactly two tags, no :latest [absent after == absent baseline], package public, config blob read with WorkingDir=/work, User=1000:1000, Entrypoint=["deepseek-build"]); the in-image run is image-run-blocked result:skipped with digest-pinned commands because the Docker daemon is absent, and the README container section now carries the real digests, the headless-only contract and both mounts; next is 04-05
last_updated: "2026-11-26T06:55:00.000Z"
last_activity: 2026-11-26
last_activity_desc: 04-04 proved the GHCR index, platforms, tags and unchanged :latest from the registry without a daemon, recorded REL-05 as environment-blocked with exact commands, and corrected the README container truth; REL-04 stays Partial, REL-05 stays open
state_head: 1516211baa637fde432de2a79b5cba4c91a17c02
progress:
  total_phases: 6
  completed_phases: 3
  total_plans: 21
  completed_plans: 20
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-11-23)

**Core value:** One DeepSeek key in, a trustworthy daily coding agent out: the right model on every turn, zero traffic to any host you did not configure, and a release you can install with confidence.
**Current focus:** Phase 4 — Distribution Proof

## Current Position

Phase: 4 — Distribution Proof
Plan: 04-04 of 5 complete (4/5). Next is 04-05 (seal the evidence, keep-prerelease deviation, leak scan).
Status: Plan 04-04 complete — the **GHCR image is proven registry-side from the host**, without a daemon. An anonymous `ghcr.io` pull token read the manifest: OCI index `sha256:8f5c484ed067…` (**equal to the CI run's `image_digest`**), `linux/amd64` `sha256:1d216b60…` and `linux/arm64` `sha256:25dc0d25…` (both 1810 B, plus two buildx attestation manifests recorded separately), tags exactly `0.1.0-proof1` + `v0.1.0-proof1`, and **`:latest` absent** (`latest_digest: null`, equal to the baseline). The package is **public** (anonymous read), and the config blob was readable: `WorkingDir=/work`, `User=1000:1000`, `Entrypoint=["deepseek-build"]`, env names only `PATH`/`DEBIAN_FRONTEND`. `image-registry` is 8/8 `result: ok`; fixtures are 45/45. **The pull and the in-image turn did not happen**: `docker info` exits 1 (`Cannot connect to the Docker daemon at unix:///var/run/docker.sock. Is the docker daemon running?`), and `image-run-blocked` is **`result: skipped`** with the digest-pinned pull, `--version` and the sandbox-on headless turn (`-p`, `--output-format json`, `--permission-mode auto`, `--sandbox devbox`) recorded for a host with a runtime. The README `## Container` section now carries the real digests, the two tags and the absence of `:latest`, the observed public visibility, the headless-only contract and both mounts (`/work`, `/home/app/.deepseek-build`), and states the image was not executed here (`check-readme-distribution.py --stage container` 7/7; install stage still 12/12). **REL-04 stays Partial** (pull deferred), **REL-05 stays Pending/blocked**, and ROADMAP criterion 1 ("pulled from the registry") is recorded as unmet.
Last activity: 2026-11-26 06:53 UTC — 04-04 added the `registry` subcommand, the `image-registry` and `image-run-blocked` evidence items, and the corrected README container section.

Progress: [██████████] 95%

## Performance Metrics

**Velocity:**
- Total plans completed: 19
- Average duration: —
- Total execution time: 0 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 01 | 6 | 6 | - |
| 02 | 6 | 6 | - |
| 03 | 4 | 4 | - |
| 04 | 4 | 5 | - |

**Recent Trend:**
- Last 5 plans: —
- Trend: —

*Updated after each plan completion*
**Per-Plan Metrics:**

| Plan | Duration | Tasks | Files |
|------|----------|-------|-------|
| Phase 04 P04 | 11m | 2 tasks | 18 files |
| Phase 04 P03 | 17m | 2 tasks | 22 files |
| Phase 04 P02 | 15m | 2 tasks | 30 files |
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
- [Phase 04]: The pinned install is proven from the **release asset** `install.sh` (sha256 `31e114d4…`, byte-identical to the tree), with the host tarball fetched from the real `/releases/download/v0.1.0-proof1/…` URL and its sha256 compared three ways (`298c9f27…` in all three). The installed binary prints `deepseek-build 0.1.0-proof1 (b14d28eb2bc7)`. `scripts/prove-install.sh` refuses to run if the installer's fake-tree hooks are set; their literal names are assembled from a suffix so the plan's grep guard stays meaningful.
- [Phase 04]: `releases/latest` does **not** 404 in a browser: it 302s to `…/releases` (the index, 200). The two `/releases/latest/download/…` URLs 404, and so does the REST `repos/:owner/:repo/releases/latest`. Record all three; do not compress them into "404".
- [Phase 04]: README install claims are gated by `scripts/check-readme-distribution.py` (whitespace-normalized so a wrapped phrase still matches). The `container` stage is written now and is 04-04's to satisfy.
- [Phase 04]: `install.sh` is byte-unchanged. The scratch `HOME` carries a `.curlrc` with `no-progress-meter` so the captured transcript stays readable; it changes curl's verbosity only, never the download or the checksum.
- [Phase 04]: The Homebrew formula is a **binary** formula pinned to `v0.1.0-proof1` — the only published tag with macOS artifacts (`v0.1.0` is 404). Both URLs sit inside `on_macos do on_arm … on_intel … end`, so the formula carries no top-level `url`; two `url` lines, two `sha256` lines, no `head`/`branch`/`patch`, no Xcode dependency.
- [Phase 04]: `livecheck do skip` is explicit. Omitting livecheck is **not** "no other host": `brew livecheck` would then use Homebrew's `:github_latest` strategy and query `api.github.com`. `brew audit --strict --online` still exits 0 with the skip.
- [Phase 04]: A macOS-only formula cannot be loaded by Homebrew on Linux (`formula requires at least a URL`); evaluate the macOS branches with Homebrew's own `HOMEBREW_SIMULATE_MACOS_ON_LINUX=1`, never by hand-waving. `brew fetch` is the strongest host-available proof (download + checksum gate), and it is paired with a wrong-sha256 control that is rejected (`reports different checksum`). `brew install` stays NOT RUN — macOS Mach-O on a Linux host.
- [Phase 04]: The tap repository `coruairc/homebrew-deepseek-build` is **not** created; `packaging/homebrew/README.md` holds the author's one-time `gh repo create`/copy/push steps, the consumer `brew tap`/`install`/`test`, the ad-hoc-signed/not-notarized position, and the `v0.1.0` repoint recipe. PKG-04 covers the in-tree formula plus that documented activation.
- [Phase 04]: The image proof records **which transport answered**. `anonymous-http` (an anonymous ghcr.io pull token + the v2 API) means the package is **public**; `docker manifest inspect` cannot distinguish public from private (the CLI may hold a credential) and cannot print the index digest or tag list; `gh api …/packages` without `read:packages` is a 403 authorization gap, never a registry result. The real read was anonymous, so the README omits `docker login` and the package is documented as public.
- [Phase 04]: An OCI index is asserted on its **platform** manifests. buildx also writes one `unknown/unknown` attestation manifest per platform (`vnd.docker.reference.type=attestation-manifest`); `registry.json` records those in `attestation_manifests`, and a fixture proves a third real platform is rejected while an attestation is not counted.
- [Phase 04]: `:latest` is proven against a **recorded baseline**, not an assumption: `latest_status: absent` / `latest_digest: null` after == the baseline's values, and the index digest is never a `latest` target. The baseline's own post-push limitation stays documented.
- [Phase 04]: REL-05 is environment-blocked and recorded as `result: skipped`, never `ok`. `image-run-blocked/commands.txt` pins the index digest, keeps the sandbox on (`--sandbox devbox`) and sets `DEEPSEEK_BUILD_HOME=/home/app/.deepseek-build` because the image config carries no `HOME`; the notes state in one sentence that the run did not happen. **ROADMAP Phase 4 criterion 1 ("pulled from the registry") stays unmet** and is named in `04-04-SUMMARY.md`.

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
- **REL-05 / REL-04 pull — environment block, now recorded (2026-11-26, 04-04):** this host has the `docker` CLI but **no daemon** (`Cannot connect to the Docker daemon at unix:///var/run/docker.sock`, `exit 1`). The image is proven registry-side (`image-registry`, 8/8 ok, index `sha256:8f5c484e…` == the CI digest); the pull and the in-image turn are `image-run-blocked` **`result: skipped`** with digest-pinned commands and must never be claimed as executed here. Closing REL-05 needs a host with a container runtime (`sudo systemctl enable --now docker`); `gh auth refresh -s read:packages` only if the package becomes private (it is public as observed).
- **README baseline drift — RESOLVED (2026-11-26, 04-02):** `README.md` never contained "not codesigned and not notarized" at this HEAD (03-03 had already written the ad-hoc-signed truth), and its "No release has been published yet" banner was falsified by rc1–rc5 and proof1. 04-02 rewrote the banner, the Status row, the install commands, the unpinned paragraph and the macOS note; `scripts/check-readme-distribution.py --stage install` is 12/12 and rejects the stale claims. 04-03/04-04 must keep it green.
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

Last session: 2026-11-26
Stopped at: Phase 4 plan 04-04 complete (4/5) — the GHCR image is proven registry-side from the host without a daemon (`image-registry`: index `sha256:8f5c484e…` == the CI digest, both platform manifests, exactly two tags, no `:latest`, public, config blob read), the in-image run is recorded as environment-blocked (`image-run-blocked`, `result: skipped`, digest-pinned commands, sandbox on), and the README container section is corrected (`check-readme-distribution.py --stage container` 7/7; install stage 12/12). REL-04 stays Partial and REL-05 Pending; ROADMAP criterion 1 stays unmet. Next: 04-05 (seal the evidence, keep-prerelease deviation, leak scan).
Resume file: None
