# Phase 3: Pipeline Decoupling & Version - Context

**Gathered:** 2026-11-25
**Status:** Ready for planning

<domain>
## Phase Boundary

Decouple the release pipeline so a slow macOS leg can no longer erase the container image, make the
macOS artifacts provable inside a real job budget with no DWARF shipped, settle the version contract
(`0.1.0` through `[workspace.package]`) before any tag exists, and verify `release.yml` against the
`v0.1.0` tag shape.

**In scope:** `.github/workflows/release.yml`, the release/profiles in the workspace `Cargo.toml`, the
four crate manifests at `0.0.1-rc5`, `Cargo.lock`, `install.sh`'s validation contract (read/verified,
not rewritten), and a static tag-shape verification. No CI proof run and no push in this phase.

**Out of scope:** publishing `v0.1.0` or any release; adding a `deepseek-*` trigger; publishing debug
symbol sidecars (`REL-V2-02`); a general `ci.yml` (`REL-V2-01`); the GHCR image pull and in-image run
(Phase 4); docs reconciliation (Phase 5).

</domain>

<decisions>
## Implementation Decisions

### Pipeline structure
- `build-linux` and `build-macos` are separate jobs, each carrying only its own OS's matrix legs and
  dependency/toolchain steps, with the artifact names unchanged (`release-<target>`). — **Reversibility:** costly — `publish` and `container` `needs:` plus every downstream `download-artifact` name depend on the split; undoing it means re-touching the whole publish graph.
- `publish: needs: [version, build-linux, build-macos]` and `container: needs: [version, build-linux]`. Publish must wait for macOS because its verify step names all four tarballs; container must not, so a slow or failed macOS leg can no longer skip the image.
- `workflow_dispatch` on a branch stays release-inert: no job-level `if` moves to `publish`; only the release-creation step keeps `github.ref_type == 'tag'`, and `container` keeps its tag gate. A branch dispatch therefore builds and uploads workflow artifacts only.
- **No separate CI proof run.** The workflow changes are pushed without a dispatch. REL-02 is proven by Phase 4's throwaway pre-release tag run, which builds all four targets for real and publishes the tarballs; if a macOS leg still exceeds its budget there, that run is the finding and Phase 3 is re-opened. — **Reversibility:** costly — accepted knowingly: the author chose to push untested rather than spend a runner cycle on a temp branch; the first real evidence is the Phase-4 tag run.

### Tag-list guard (REL-08)
- A guard runs **before** the first `build-push-action` step and asserts the resolved tag list: every expected semver tag present, no unexpected `latest`, and a non-empty result — fail loudly otherwise.
- The `version` job also validates semver strictly, so a non-semver `v*` tag (`v0.1`) is rejected before any build or push starts.

### Version (PKG-01, PKG-02)
- `[workspace.package] version = "0.1.0"`; the four crates at `0.0.1-rc5` (`xai-grok-shell`, `xai-grok-pager`, `xai-grok-pager-bin`, `xai-grok-version`) switch to `version.workspace = true`; `Cargo.lock` regenerated. The other ~83 literal `0.1.0` manifests may stay literal (mixing is legal and keeps the diff bounded).
- Both version paths are asserted explicitly: the CI `GROK_VERSION` stamp reaching `--version`, and a source build's `CARGO_PKG_VERSION` reaching `--version`.

### macOS artifacts (REL-02, REL-03, REL-07)
- `release-dist` must ship no DWARF: `debug = false` plus link-time strip, never an external strip applied after signing.
- The macOS check runs `--version` from the **extracted tarball** with the signature intact (`codesign --verify --strict`), not the pre-tar binary.
- The macOS job gets its own larger timeout; if a cold build still exceeds it, per-job profile env overrides (LTO off / codegen-units) apply to macOS only and leave Linux settings untouched.
- One cache-warming run is allowed only if cold cache is demonstrably the cause.

### Decided autonomously
- [Pipeline] Q: "One reusable workflow or two duplicated jobs?" → Selected: "two jobs, steps duplicated" — cheap to undo; a reusable workflow adds indirection for two consumers whose OS steps differ.
- [Version] Q: "Adopt workspace inheritance in this phase?" → Selected: "yes, for the four crates" — required by PKG-01; reverting is a manifest edit.
- [Docs] Q: "Reconcile KNOWN-ISSUES/AGENTS/README in this phase?" → Selected: "only if a Phase-3 fact changes them" — Phase 5 owns docs reconciliation; a premature rewrite would describe a tree that still moves.
- [Trigger] Q: "Add a `deepseek-*` release trigger while editing the workflow?" → Selected: "no" — locked decision; it would risk classifying a snapshot as a stable release.

### the agent's Discretion
- Exact timeout numbers, whether the tag guard is a shell step or a metadata `if`, whether macOS needs profile env overrides at all (the proof run decides), and the internal step ordering inside each split job.

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `release.yml` `version` job: tag parse, prerelease classification, `GROK_VERSION` output.
- `install.sh` + `scripts/test-install.sh`: the installer's checksum/version validation contract that PKG-03 verifies against.
- `Swatinem/rust-cache` per job; `crates/codegen/xai-grok-pager-bin/build.rs` version stamp; `xai-grok-version` crate.

### Established Patterns
- Every action pinned to a full SHA; least-privilege `permissions: {}` at workflow level with per-job elevation.
- Native runners only (no cross-compilation, no QEMU); tarballs assembled by one shared packaging step shape.
- Artifact names are the contract between `build`, `publish` and `container`.

### Integration Points
- `publish` verifies all four tarball names; `container` stages per-arch contexts from the two Linux tarballs and copies `docker/Dockerfile`.
- `docker/metadata-action` computes tags; `build-push-action` pushes by digest and `imagetools` assembles the manifest.
- `--version` is the observable that ties `GROK_VERSION`/`CARGO_PKG_VERSION` to the artifact.

</code_context>

<specifics>
## Specific Ideas

- The rc4 run `40111152104` is the recorded evidence that 90 minutes is insufficient on the macOS arm64 runner; the author's own host builds the same tree in ~28m38s without LTO and >40 min at thin LTO.
- `release-dist` today: `strip = false`, `debug = 1`, `split-debuginfo = "off"` — so it emits DWARF and must change.
- Splitting the jobs and bumping the manifests re-keys `rust-cache`, so the proof run is a cold build on all four legs.
- `:latest` is pushed by the build-push steps themselves (metadata `latest=auto`), so the guard must precede the first push.

</specifics>

<deferred>
## Deferred Ideas

- `ci.yml` on push (REL-V2-01) and debug-symbol sidecars (REL-V2-02) — v0.1.x.
- Publishing `v0.1.0` itself — the author publishes it after the freeze.
- Pulling and running the image, and the README GHCR block — Phase 4.

</deferred>
