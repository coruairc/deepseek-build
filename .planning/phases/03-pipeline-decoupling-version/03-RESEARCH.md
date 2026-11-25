# Phase 3: Pipeline Decoupling & Version - Research

**Researched:** 2026-11-25
**Domain:** GitHub Actions release pipeline (`release.yml`), Cargo workspace versioning, macOS distribution artifacts
**Confidence:** HIGH for every in-tree claim (read this session at `4dded936`, file:line cited). HIGH for Cargo/rustc/metadata-action documented behaviour (official docs fetched). MEDIUM for macOS code-signature mechanics (mechanism inferred, no macOS host here). LOW for cache-save-on-cancellation timing.
**No CONTEXT.md exists for this phase** — the constraints below are inherited from `PROJECT.md` Key Decisions, `ROADMAP.md` §Phase 3 and `.planning/STATE.md` Blockers.

## Summary

The pipeline defect is exactly as recorded: one `build` job runs all four legs, `publish` and `container` both `needs: [version, build]`, and `container` carries `if: github.ref_type == 'tag'`. On `v0.0.1-rc4` the `aarch64-apple-darwin` leg hit the 90-minute job timeout, so the run failed before `publish` and the container job never started — no GHCR image exists (`KNOWN-ISSUES.md:262-274`). The fix is structural: two build jobs with their own matrices, `container` depending only on the Linux one, `publish` still depending on both because its verify step hardcodes all four tarball names (`release.yml:283-295`).

Two non-obvious facts dominate the plan. First, **the shipping profile deliberately ships DWARF**: `[profile.release-dist]` sets `strip = false` and `debug = 1` (`Cargo.toml:388-394`), and the workflow tars `target/release-dist/deepseek-build` directly with no strip, no `dsymutil`, no `codesign` step anywhere (`release.yml:219`, `release.yml:243`; verified by reading the whole file). `debug = 1` is Cargo "limited" — module-level info, *more* than line tables, not less. Second, **splitting the matrix and bumping the version both invalidate `Swatinem/rust-cache`**, because its key includes the job id and a hash of every `Cargo.toml`/`Cargo.lock`; so the first post-split run and the first post-bump run are cold for all four legs — exactly the condition `REL-02`'s "one cache-warming run" anticipates.

The version work is bounded and mechanical: add `version = "0.1.0"` under `[workspace.package]` (which today has only `edition` and `license`, `Cargo.toml:106-108`) and switch exactly four manifests — `xai-grok-shell`, `xai-grok-pager`, `xai-grok-pager-bin`, `xai-grok-version` — from `version = "0.0.1-rc5"` to `version.workspace = true`. No crate depends on them with a version requirement, so nothing else in the graph moves. `pager-bin`'s `build.rs` and `xai-grok-version`'s `VERSION` const are the two places a version surfaces, and `--version` uses the *former*.

**Primary recommendation:** land the split and the macOS time budget in one change (split alone re-keys the cache), make `release-dist` emit no DWARF via `debug = 0` + a link-time `strip` (never an external strip after signing), guard the metadata tag list by exact `tag-names` match *before* the first push (because `latest` is already in the tag list for stable tags), and run the REL-02 dispatch proof **after** the Cargo.toml version bump so the proof is not invalidated by the cache-key change.

## Phase Requirements

| ID | Research establishes |
|----|----------------------|
| REL-01 | Current single-job shape and exact step inventory (`release.yml:97-255`); minimal split = two jobs, same ten steps, OS-conditional steps deleted per job; `container: needs: [version, build-linux]`. |
| REL-02 | Dispatch run shape is already artifact-only: `publish` job has no `if` but its release step is `if: github.ref_type == 'tag'`; `container` is tag-only. Timeout evidence and levers below. |
| REL-03 | No codesign/strip step exists today; smoke runs the pre-tar binary (`release.yml:197`), not the extracted one. Ordering rule and verification commands below. |
| REL-07 | `release-dist` emits DWARF today (`strip = false`, `debug = 1`); `split-debuginfo = "off"` already prevents `.dSYM`. |
| REL-08 | metadata-action has no failure on no-match; buildx fails only for a *fully* empty list; a junk `type=ref` tag is the silent-success path that moves `:latest`. |
| PKG-01 | `[workspace.package]` has no `version`; 4 crates at `0.0.1-rc5`; 83 (not 82) at literal `0.1.0`; lock has the four entries. |
| PKG-02 | CI path already asserted (`release.yml:192-206`); source path is `pager-bin/build.rs` fallback to `CARGO_PKG_VERSION`, with no test pinning it. |
| PKG-03 | `v0.1.0` traverses trigger, parse, prerelease regex and installer URL construction unchanged; only Docker's `{{major}}` → `0` and the README "not codesigned" line need attention. |

## Constraints Inherited (no CONTEXT.md)

- **Publish tag is `v0.1.0`; no `deepseek-*` trigger is added** (`PROJECT.md` Key Decisions; `STATE.md:74`). The existing `v*` trigger already handles it; a snapshot can never be classified stable.
- **macOS ships both arches in v0.1.0 — the timeout is a pipeline defect, not a reason to abandon** (`PROJECT.md:165`).
- **No new outbound destination, no new upload path** (`release.yml:9-23`; D6/D10/D13). The only allowed uploads are intra-run artifacts, the GitHub release and GHCR; `container` keeps the sandbox on and the Dockerfile is untouched.
- **Precondition:** the push remote is SSH (`git remote -v` → `git@github.com:coruairc/deepseek-build.git`), so editing `.github/workflows/release.yml` needs no `workflow`-scoped token (`ROADMAP.md` §Phase 3 Preconditions). The earlier rc4-era block was a credential problem.
- **A dry run is a real push** (`STATE.md:110`): the dispatch verification must create no release and move no `:latest`.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Build orchestration / matrix | CI workflow (`release.yml`) | — | Only place `needs:` and `if:` can express "image does not wait for macOS". |
| Version derivation | CI `version` job | `build.rs` at compile time | CI derives from the ref; `build.rs` stamps it into the binary. |
| Version source of truth | Cargo manifest (`[workspace.package]`) | — | `CARGO_PKG_VERSION` is compiled in; CI can only override it. |
| Artifact packaging | build job (shell) | — | Tarball is assembled from `target/<profile>/` in the same job. |
| Image tagging | `container` job (metadata-action + buildx) | — | Tag set is decided before any push; the manifest step only assembles digests. |
| Signature integrity | macOS toolchain (ld64/codesign) | CI verify step | Only the linking/signing toolchain can produce a valid signature; CI can only verify. |
| Installer contract | `install.sh` (client) | release asset list | Installer builds URLs from the tag; the release must match that shape. |

## What Exists Today

### `release.yml` job graph (462 lines, only workflow in the repo)

| Job | `needs` | `if` | Timeout | Runner |
|-----|---------|------|---------|--------|
| `version` | — | — | 10 | `ubuntu-22.04` |
| `build` (4-leg matrix) | `version` | — | **90** | ubuntu-22.04 / ubuntu-22.04-arm / macos-15 / macos-15-intel |
| `publish` | `[version, build]` | **none on the job** | 15 | `ubuntu-22.04` |
| `container` | `[version, build]` | **`github.ref_type == 'tag'`** | 30 | `ubuntu-22.04` |

`[VERIFIED: .github/workflows/release.yml:97-113]`

```yaml
  build:
    name: build (${{ matrix.target }})
    needs: version
    runs-on: ${{ matrix.runner }}
    timeout-minutes: 90
    strategy:
      fail-fast: false
      matrix:
        include:
          - target: x86_64-unknown-linux-gnu
            runner: ubuntu-22.04
          - target: aarch64-unknown-linux-gnu
            runner: ubuntu-22.04-arm
          - target: aarch64-apple-darwin
            runner: macos-15
          - target: x86_64-apple-darwin
            runner: macos-15-intel
```

Per-leg steps, in order (`release.yml:114-255`): checkout `ref: github.ref`; Linux apt `protobuf-compiler cmake` + `PROTOC` (`if: runner.os == 'Linux'`); macOS `brew install protobuf` + `PROTOC` (`if: runner.os == 'macOS'`); rustup 1.94.0 (two OS-conditional variants); `Swatinem/rust-cache` with `key: ${{ matrix.target }}`, `cache-on-failure: true`; build with `GROK_VERSION: ${{ needs.version.outputs.version }}`; smoke `--version`; package; upload as `release-${{ matrix.target }}`.

The build and stamp are one command (`[VERIFIED: release.yml:186-188]`):

```yaml
        run: cargo build --profile release-dist -p xai-grok-pager-bin
        env:
          GROK_VERSION: ${{ needs.version.outputs.version }}
```

The tarball is five top-level entries, no nested dir (`release.yml:243`):

```
tar -czf "deepseek-build-${RELEASE_VERSION}-${RT}.tar.gz" -C "$stage" deepseek-build LICENSE NOTICE THIRD-PARTY-NOTICES README.txt
```

**No signing or strip step exists.** `release.yml` contains no `codesign`, `dsymutil` or strip invocation (read in full; the only `strip` settings in the tree are Cargo profile keys), so the tarball is the linker's output byte-for-byte. The README states the outcome (`README.md:122`): `The macOS binaries are **not codesigned and not notarized**`.

### Version stamping — three sites, two paths

`[VERIFIED: crates/codegen/xai-grok-pager-bin/build.rs:27-30]`

```rust
    let version = std::env::var("GROK_VERSION")
        .or_else(|_| std::env::var("CARGO_PKG_VERSION"))
        .unwrap_or_else(|_| "0.0.0".to_string());
    println!("cargo:rustc-env=VERSION_WITH_COMMIT={version} ({commit})");
```

`[VERIFIED: crates/codegen/xai-grok-version/src/lib.rs:11-14]`

```rust
pub const VERSION: &str = match option_env!("GROK_VERSION") {
    Some(v) => v,
    None => env!("CARGO_PKG_VERSION"),
};
```

`--version` is built from `full_version()` — i.e. the value `main.rs` injects from `env!("VERSION_WITH_COMMIT")` (`crates/codegen/xai-grok-pager-bin/src/main.rs:1714-1722`, `:1761`; the `--json` form reads the env var directly at `:1906`). **So the human-visible source-build version comes from `xai-grok-pager-bin`'s manifest, not from `xai-grok-version`'s.** Both must inherit, but pager-bin is the one that drives `--version`. Both `build.rs` files declare `cargo:rerun-if-env-changed=GROK_VERSION`, so the two paths can be exercised on one host by setting/unsetting the var.

### Profiles

`[VERIFIED: Cargo.toml:372-394]`

```toml
[profile.release]
panic = "abort"
strip = "symbols"

[profile.release-dist]
inherits = "release"
lto = "thin"
codegen-units = 1
strip = false
debug = 1
split-debuginfo = "off"
```

`[VERIFIED: Cargo.toml:106-108]` — `[workspace.package]` contains exactly `edition = "2024"` and `license = "Apache-2.0"`; there is **no `version` key**.

### Installer contract (`install.sh`)

Pinned path accepts `vX.Y.Z` or `X.Y.Z`, rejects anything not starting `v[0-9]`, and builds `deepseek-build-<version>-<target>.tar.gz` (`[VERIFIED: install.sh:274-286]`):

```sh
    case "$VERSION_TAG" in
        v[0-9]*) ;;
        *) die "Invalid VERSION: $VERSION (expected vX.Y.Z or X.Y.Z)" ;;
    esac
    TARBALL="deepseek-build-${VERSION_NUM}-${TARGET}.tar.gz"
```

`scripts/test-install.sh` fakes the whole release tree on loopback (`FAKE_VERSION=0.0.0-test`, `scripts/test-install.sh:24`, `:145-148`), so it exercises a hyphenated pre-release shape, not `0.1.0`.

## Findings

### F1 — Minimal split that preserves behaviour

- **`build-linux`**: matrix of the two Linux entries only; steps checkout → apt/protoc → rustup → cache → build → smoke → package → upload. The `if: runner.os == 'macOS'` steps are deleted, not left dead.
- **`build-macos`**: matrix of the two macOS entries; steps checkout → brew/protoc → rustup → cache → build → smoke → package → upload. `cmake` is preinstalled on macOS runners (`release.yml:148`); only `protoc` is installed.
- Nothing else moves. Artifact names stay `release-<target>`, so `publish`'s `pattern: release-*` and `container`'s two named downloads are unchanged (`release.yml:276-281`, `:363-373`).
- `needs` becomes: both build jobs `needs: version`; `publish: needs: [version, build-linux, build-macos]`; `container: needs: [version, build-linux]`.
- **`publish` must keep waiting for macOS.** Its verify step hardcodes all four names and exits 1 on a missing file (`release.yml:283-295`), so a `needs` that omitted `build-macos` would fail anyway — but it would fail *after* a release was possible. Express it in `needs`.
- **`container` must not** wait: its two downloads are Linux-only (`release.yml:363-373`), so the macOS legs are irrelevant to it.
- Consequence: a macOS failure now leaves `container` green (image published from the tagged commit) and turns `publish` red. That is the intended direction: no release rather than a release missing a promised target (STACK's rejected alternative is `continue-on-error`).
- `fail-fast: false` should stay on the macOS job so both arches attempt. The rc4 timeout cancelled the arm64 leg itself; `publish` and `container` were then skipped because both needed the whole matrix — the split removes that coupling, not the leg's timeout.

### F2 — Dispatch already produces artifacts only; exact shape

`[VERIFIED: .github/workflows/release.yml:261-266]` — the publish job has **no `if:` at job scope**:

```yaml
  publish:
    name: Publish GitHub release
    needs: [version, build]
    runs-on: ubuntu-22.04
    timeout-minutes: 15
```

The only gate is on the release-creation step (`[VERIFIED: release.yml:322-324]`):

```yaml
      - name: Create the GitHub release
        if: github.ref_type == 'tag'
        uses: softprops/action-gh-release@...
```

`[VERIFIED: release.yml:347-352]` — container is tag-only: `needs: [version, build]`, `if: github.ref_type == 'tag'`.

**Run shape that produces artifacts only:** Actions → *Run workflow* on a **branch** (`workflow_dispatch: {}`, `release.yml:43`). Then: `version` sets `version=0.0.0-<sha7>` and `is_prerelease=true` (the `-` matches `release.yml:86`); both build jobs run and upload `release-*`; `publish` runs, downloads and merges all four, re-generates `SHA256SUMS` **but uploads nothing** (the upload lives inside the skipped release step); `container` is skipped entirely. No release, no GHCR push, no `:latest` move. The run's artifacts are the evidence REL-02 needs.

Two limits of that shape: (a) it never exercises `github.ref_type == 'tag'`, so it cannot prove the prerelease classification or the release asset list — that is PKG-03/Phase 4 work; (b) it cannot prove `install.sh` against a real release, since no `SHA256SUMS` asset is published. A dispatch that fails on macOS also turns `publish` red (its verify step), which is a useful gate but means a dispatch is not "green" until both macOS legs pass.

### F3 — metadata-action: what an unmatched list actually does

The action's pinned README (same SHA as the workflow pin) documents `type=semver` as: *"Will be used on a push tag event and requires a valid semver Git tag"*, and every other `type=ref` entry as conditional on its event (`event=tag` only matches tag refs). Consequences for the current list (`release.yml:406-410`):

- **Stable `v0.1.0`** → `0.1.0`, `0.1`, `0`, `v0.1.0`, plus `latest` (`flavor` default `latest=auto` adds `latest` for `type=semver`/`type=ref,event=tag`). Note the `0` tag: the README's *Major version zero* section says *"Docker tag `0` **should not** be generated if you're using `type=semver` with `{{major}}` pattern"* and shows `enable=${{ !startsWith(github.ref, 'refs/tags/v0.') }}`.
- **Pre-release `v0.1.0-rc1`** → semver patterns all resolve to the full `0.1.0-rc1` (README: *"Pre-release (rc, beta, alpha) will only extend `{{version}}` … as tag"*), plus `v0.1.0-rc1`; **no `latest`**. The source is explicit: `procSemver` sets `latest = false` when `semver.prerelease(vraw)` is truthy, and `setVersion` keeps the *first* tag's `latest` value, so the later `type=ref,event=tag` entry (which passes `latest = true`) cannot re-add it (`src/meta.ts`). Confirms the current `is_prerelease` branch is consistent with the action.
- **Non-semver tag matching `v*` (e.g. `v0.1`)** → semver entries emit nothing; `type=ref,event=tag` emits `v0.1` **and `latest`**. This is the silent-success case: the tag list is non-empty, buildx pushes, `:latest` is moved by the build step itself, `is_prerelease=false`, and only the final `imagetools inspect ":0.1"` fails. Damage already done.
- **Branch ref (dispatch)** → semver and `event=tag` both emit nothing → `tags` output is empty. The action does not fail. Today `container` is tag-gated so this is unreachable; if that `if` were ever relaxed, buildx rejects the push with `tag is needed when pushing to registry` ([CITED: github.com/docker/buildx/blob/master/build/build.go] — `return nil, nil, errors.Errorf("tag is needed when pushing to registry")`). So the *build* step is loud for a fully-empty list; the *manifest assembly* step is the silent one — `while IFS= read -r tag … [ -z "$tag" ] && continue` over an empty here-string iterates zero times, then the `:latest` branch can still run (`release.yml:450-461`).

**Guard that fails loudly, placed before the first push:** after the metadata step, assert `steps.meta.outputs.tags` is non-empty **and** that `steps.meta.outputs.tag-names` contains the exact version from the `version` job (exact line match, not substring — `:0.1.0` is a prefix of `:0.1.0-rc1`). `tag-names` is a documented output. Pair it with a semver check in the `version` job (`^[0-9]+\.[0-9]+\.[0-9]+(-…)?$`), which is what actually kills `v0.1` before any push.

### F4 — Where the version is stamped, and what the bump breaks

Three independent sites: (1) `[workspace.package] version` → `CARGO_PKG_VERSION`; (2) `xai-grok-pager-bin/build.rs:27-30` → `VERSION_WITH_COMMIT` (the `--version` text and `--json`); (3) `xai-grok-version/src/lib.rs:11-14` → `VERSION` (ACP `shellVersion`, headers, logs, config `version_overrides` gating). CI overrides all three at once with `GROK_VERSION` (`release.yml:188`).

**Exactly four manifests must switch to `version.workspace = true`** — the only four at `0.0.1-rc5`:

```
crates/codegen/xai-grok-shell/Cargo.toml:4
crates/codegen/xai-grok-pager/Cargo.toml:3
crates/codegen/xai-grok-pager-bin/Cargo.toml:3
crates/codegen/xai-grok-version/Cargo.toml:4
```

**What "82 crates keep a literal version" gets wrong:** the tree has **83** literal `version = "0.1.0"` member manifests (`python3` count over the 95 `members`), plus 3 at `0.1.220-alpha.4`, 1 at `0.0.0`, the 3 odd third-party crates (`0.0.5`/`0.0.2`/`0.0.3`), and `xai-grok-shared`, which has **no `version` key at all** (Cargo defaults it to `0.0.0`). So after the change: 4 inherit, 83 stay literal, 8 are unrelated. Nothing breaks — Cargo permits mixing literal and inherited versions, and no crate depends on the four with a version requirement (`grep -rn 'xai-grok-(version|pager|pager-bin|shell) *= *\{[^}]*version'` → only `Cargo.toml:350` `xai-grok-version = { path = … }`, no version). The real risks are (a) a second source of truth that a later bump forgets, and (b) `Cargo.lock` going stale: the four entries at `Cargo.lock:12663-12664, 12766-12767, 13145-13146, 13558-13559` must change, and **the workflow builds without `--locked`** (`release.yml:186`), so CI would silently regenerate a stale lock instead of failing. Adding `--locked` to the build step is the cheap enforcement.

Runtime side effect to note, not a defect: `installed_semver()` jumps from `0.0.1-rc5` to `0.1.0`, which changes `[[version_overrides]]` gating and the leader-version comparison (`leader_is_older_than`) — a running rc5 leader would be considered older and relaunched.

### F5 — macOS signature and strip ordering

Today the artifact is exactly what the linker produced: no strip, no re-sign. On Apple Silicon the linker emits an **ad-hoc** signature [ASSUMED — standard ld64 behaviour]; there is no Developer ID signature and nothing to notarize (`README.md:122`). The requirement's "signature intact" therefore means *the ad-hoc/linker signature must still validate after the profile change*.

Mechanics that matter:

- rustc's `-C strip=val` *"controls stripping of debuginfo and similar auxiliary data from binaries **during linking**"* ([CITED: doc.rust-lang.org/rustc/codegen-options/index.html#strip]) — so a Cargo-level `strip` is part of producing the final Mach-O. That is the safe mechanism [ASSUMED: the linker signature covers the stripped output].
- An **external** `/usr/bin/strip` on the finished binary rewrites it after signing; a modified signed binary no longer validates, and on arm64 the kernel refuses to exec it. If a post-link strip is ever used, it must be followed by `codesign --force --sign - <binary>` before packaging. [ASSUMED — mechanism is standard Apple signing behaviour, not measured in this repo.]
- `tar` does not alter file bytes, so the signature survives packaging; what needs checking is that the extracted file still execs. **The current smoke runs the pre-tar binary (`release.yml:197`), not the extracted one**, so REL-03 needs a new step: `tar -xzf … -C "$tmp" && "$tmp/deepseek-build" --version` plus `codesign --verify --strict --verbose=2` (and `codesign -dv` to see `Signature=adhoc`). On arm64 the exec itself is the signature test; `codesign --verify` makes the failure legible instead of `killed: 9`.

### F6 — REL-07: `release-dist` ships DWARF today

Yes. `[profile.release]` has no `debug` key (Cargo default `false`) and `strip = "symbols"`; `[profile.release-dist]` then **overrides both**: `strip = false` (= `"none"`, [CITED: doc.rust-lang.org/cargo/reference/profiles.html#strip]) and `debug = 1`. Cargo documents `1`/`"limited"` as *"debug info without type or variable-level information … more detailed module-level info than `line-tables-only`"* — i.e. real DWARF, more than STACK.md's "line-table" description. `split-debuginfo = "off"` is doing its job: on macOS it *"prevents the final execution of `dsymutil` to generate debuginfo"* ([CITED: doc.rust-lang.org/rustc/codegen-options/index.html#split-debuginfo]), so there is no `.dSYM` today and none will appear.

Required change: `debug = false` (or `0`) **and** a strip setting in `release-dist`. Two defensible choices: `strip = "symbols"` (matches `[profile.release]`, smallest, but rustc warns it can make backtraces incomprehensible — the tree ships `xai-crash-handler`) or `strip = "debuginfo"` (removes DWARF, keeps the symbol table, backtraces still name frames). Verify on Linux with `readelf -S target/release-dist/<bin> | grep '\.debug'` (expect empty) and on macOS with `otool -l | grep -c __DWARF` = 0 plus `find target -name '*.dSYM'` empty. A cheap profile-level check that does not require the 147 MiB binary: build a small bin (e.g. `ptyctl-cli`) with `--profile release-dist` and inspect that.

### F7 — The 90-minute budget: evidence and available levers

Evidence (all in-tree, one incident):

- `KNOWN-ISSUES.md:264-268`: *"`v0.0.1-rc4` (run 40111152104) built both Linux targets, but `aarch64-apple-darwin` hit the 90-minute `timeout-minutes` and cancelled the run before `publish`."*
- `PROJECT.md:123-126` repeats it and adds *"which skipped the container job — so no GHCR image exists"*.
- `HANDOFF-2026-11-23.md:88`: *"macOS legs time out (90-min job timeout) — Phase 3 must split …"*.
- `STATE.md:109`: *"measure the macOS job log before choosing between timeout, strip and LTO override. Both arches must fit the job timeout; one cache-warming run is allowed."*
- Runner size: the arm64 macOS runner is **3 vCPU (M1) / 7 GB** while `macos-15-intel` is 4 vCPU / 14 GB ([CITED: docs.github.com/en/actions/reference/runners/github-hosted-runners]). Compare the author's 16 GB box: `--release` (no LTO) took 28m38s; thin LTO `-j 1` was still compiling at 40 min; fat LTO OOM-killed at `-j 2` (`KNOWN-ISSUES.md:345-357`). `release-dist` adds `codegen-units = 1` on top of thin LTO — slower than any of those measurements. **On 3 cores, 90 minutes is not a plausible ceiling.**

Levers already supported by the tree, cheapest first:

1. **Split** (REL-01) so the image is no longer hostage to the leg.
2. **Raise `timeout-minutes` on `build-macos` only** (90 → 180 or 240). No Cargo change; the Linux legs keep their budget and `container`/`publish` keep theirs.
3. **Per-leg profile override**, no `Cargo.toml` edit: `CARGO_PROFILE_RELEASE_DIST_LTO: "off"` and `CARGO_PROFILE_RELEASE_DIST_CODEGEN_UNITS: "16"` on the macOS job (Cargo documents profile overrides via config/env: [CITED: doc.rust-lang.org/cargo/reference/profiles.html]). Each target is a separate binary, so Linux keeps the LTO build.
4. **`debug = false` + cache**: F6 removes debug-info work everywhere; the cache is already per-target (`release.yml:174-179`) but see the re-keying facts below.

Two cache facts the plan must account for (`[CITED: raw.githubusercontent.com/Swatinem/rust-cache/<pinned>/README.md]`):

- The automatic key includes the **job id** (`add-job-id-key` default `true`) and a hash of every `Cargo.toml`/`Cargo.lock`. **Splitting `build` into `build-linux`/`build-macos` changes the job id, so every leg starts cold.** **Bumping the four manifests changes `Cargo.lock`, so every leg starts cold again.** Therefore the REL-02 proof run must come *after* PKG-01, or the version bump re-cold-starts the cache that was just warmed.
- `cache-on-failure: true` saves on *failed* jobs, but a job killed at `timeout-minutes` is a cancellation; whether the post-job cache save completes is not established here [ASSUMED/LOW]. Do not rely on a timed-out run to warm the cache — the warming run should be one that is allowed to finish (hence lever 2 before lever 4).

## Implementation Approach (what a minimal change looks like)

1. **One edit to `release.yml` for REL-01 + macOS levers**: duplicate the ten steps into `build-linux`/`build-macos`, delete the now-dead OS-conditional steps in each, set the three `needs` edges from F1, raise `timeout-minutes` on macOS only (and add `env:` profile overrides there if the log demands it — do not touch the shared `Cargo.toml` for a per-target speedup). Update the header comments (`release.yml:1-7`, `:92-96`) so they still describe the graph.
2. **`Cargo.toml`**: `[workspace.package] version = "0.1.0"`; four manifests → `version.workspace = true`; regenerate the lock with `cargo metadata --format-version 1 --offline >/dev/null` and assert `git diff Cargo.lock` is four version lines. Add `--locked` to the CI build so a stale lock is loud.
3. **`release-dist`**: `debug = false`, `strip = "debuginfo"` (or `"symbols"`). No post-build external strip.
4. **Packaging/smoke**: add the extracted-tarball smoke + `codesign --verify --strict` on macOS (REL-03). Keep the existing pre-tar smoke; it is the faster failure.
5. **Guard**: semver validation in the `version` job; exact `tag-names` assertion after the metadata step and before both build-push steps; leave the manifest loop as the assembler.
6. **Version assertion (PKG-02)**: the CI half already exists (`release.yml:192-206`). The source half needs a deliberate pin — a test comparing `VERSION` to `env!("CARGO_PKG_VERSION")` is tautological, so pin `"0.1.0"` under `if option_env!("GROK_VERSION").is_none()`, or extend `branding_surface.rs` to assert `deepseek-build 0.1.0 (<hex>)` when unstamped. Toggle `GROK_VERSION` to exercise both paths (both `build.rs` files declare `rerun-if-env-changed`).

## Risks / Unknowns

| # | Risk | Why it matters | Mitigation in-tree |
|---|------|----------------|--------------------|
| R1 | Split + version bump re-key the cache (F7) | The first post-change macOS run is cold; if it times out it may not warm the cache | Raise timeout and/or LTO override in the same change; run the proof last |
| R2 | `:latest` is moved by the *build* steps, not only by the explicit branch | Any guard after the build is too late | Guard before `build-push-action`; note `flavor` `latest=auto` adds `latest` to the tag list for stable tags |
| R3 | Non-semver tag matching `v*` (e.g. `v0.1`) is the silent path | `:latest` moves and the job fails at the last inspect | Semver check in the `version` job is the root fix; REL-08 guard is the second line |
| R4 | macOS arm64 rejects a modified signature | An external strip would brick the artifact at exec time | Cargo-level strip only; extracted-tarball exec + `codesign --verify` |
| R5 | `debug = false` + `strip` interacts with the crash handler's backtraces | `strip="symbols"` degrades them | Prefer `strip="debuginfo"`, or accept parity with `[profile.release]` |
| R6 | A dispatch cannot prove tag-only behaviour | PKG-03 needs `github.ref_type == 'tag'` | Static fixtures for the derive step + Phase 4 throwaway tag |
| R7 | Transitional install: 0.0.1-rc5 leader vs 0.1.0 client | Leader relaunch on version change | Expected; not a defect, but do not treat a relaunch as a regression |

## Validation Architecture

### Test framework / commands

| Property | Value |
|----------|-------|
| Framework / config | `cargo test` (Rust 1.94.0) + POSIX shell harnesses; `Cargo.toml`, `rust-toolchain.toml`, `.github/actionlint.yaml`; no `ci.yml` (`STATE.md:111`) |
| Quick run | `actionlint .github/workflows/*.yml && shellcheck --shell=sh install.sh scripts/test-install.sh` (<10 s) |
| Full suite | `scripts/disk-guard.sh && flock -w 900 /tmp/team-build.lock cargo test --workspace --no-fail-fast` |

### Requirement → verification map

| Req | Behaviour | Type | Command | Exists? |
|-----|-----------|------|---------|---------|
| REL-01 | Job graph shape | static | `python3` + PyYAML assert `container.needs == [version, build-linux]`, `publish.needs` includes `build-macos` | ❌ Wave 0 |
| REL-01 | Workflow syntax | lint | `actionlint .github/workflows/release.yml` | ✅ (actionlint 1.7.7 present) |
| REL-02 | macOS tarballs inside timeout on dispatch | CI | the dispatch run + artifact list; requires the raised budget | ❌ needs a real run |
| REL-03 | Extracted-tarball `--version` + signature | CI (macOS) | `tar -xzf … && dist/deepseek-build --version && codesign --verify --strict` | ❌ Wave 0 step |
| REL-07 | No DWARF in `release-dist` | unit-ish | `cargo build --profile release-dist -p ptyctl-cli && readelf -S target/release-dist/ptyctl \| grep -c '\.debug'` → 0 | ❌ Wave 0 check |
| REL-08 | Empty/junk tag list fails before push | static + shell fixture | run the guard's shell with `TAGS=""`, `TAGS="…:v0.1"`, `TAGS="…:0.1.0"` | ❌ Wave 0 |
| PKG-01 | Version + lock | build | `cargo metadata --format-version 1 --offline >/dev/null`; `git diff Cargo.lock` = 4 version lines; `grep` four manifests | ❌ after edit |
| PKG-02 | CI stamp | CI | existing `release.yml:199` grep (`deepseek-build 0.1.0 (`) | ✅ |
| PKG-02 | Source `CARGO_PKG_VERSION` | unit/bin | pin test in `xai-grok-version` or `branding_surface.rs`; `GROK_VERSION=` build then `--version` | ❌ Wave 0 |
| PKG-03 | `v0.1.0` parse/prerelease/installer | shell fixture | derive-step fixtures (`v0.1.0` → 0.1.0/false; `v0.1.0-rc1` → true; `v0.1` → reject) + `sh scripts/test-install.sh` | ❌ Wave 0 (extend harness) |

### Sampling rate

- Per task: `actionlint` + `shellcheck` + the targeted `python3` assertion (<30 s).
- Per wave: `cargo test -p xai-grok-version -p xai-grok-pager-bin` (+ `--test branding_surface`, which shells out to the built binary) and `sh scripts/test-install.sh`.
- Phase gate: one green dispatch with all four `release-*` artifacts, plus the local checks; the full workspace suite is Phase 5's gate, not this phase's.

### Wave 0 gaps

- [ ] A `release.yml` structure assertion (PyYAML) — nothing checks `needs:` today — plus a pure-shell fixture for the tag guard (empty / junk / valid).
- [ ] Extracted-tarball + `codesign --verify` macOS steps.
- [ ] `release-dist` DWARF check (cheap small-bin variant) and the source-build version pin (the tautology trap in F4/step 6).
- [ ] `install.sh` `VERSION=0.1.0` (stable, no hyphen) case in `scripts/test-install.sh`.

## Environment Availability

| Dependency | Required by | Available | Version | Fallback |
|------------|-------------|-----------|---------|----------|
| `actionlint` | workflow lint | ✓ | 1.7.7 (STACK; `.github/actionlint.yaml` present) | — |
| `shellcheck` | `install.sh` / harness lint | ✓ | present | `sh -n` |
| `python3` + PyYAML, `jq` | static/JSON assertions | ✓ | 3.x / PyYAML 6.0.4 / jq present | `yq`/grep |
| Rust 1.94.0 | version/lock/DWARF checks | ✓ on the author's host (STATE:99-100); not on this planning host | 1.94.0 | none — cargo checks need the host |
| GitHub Actions (macOS runners, GHCR) | REL-02/03 | remote | `macos-15` (3 vCPU/7 GB), `macos-15-intel` (4/14) | none for macOS proof |
| Docker/buildx | REL-08 runtime proof | not needed | — | guard tested as pure shell |

**Missing with no fallback:** the macOS runner and a real dispatch — REL-02/03 can only be proven in CI. Everything else is local.

## Security Domain

`security_enforcement` is enabled (ASVS level 1).

| ASVS category | Applies | Control |
|---------------|---------|---------|
| V2 / V3 / V6 | no | No auth, session or new-crypto surface in this phase. |
| V4 Access Control | yes | `permissions: {}` at workflow scope; `contents: write` only on `publish`; `packages: write` only on `container` (`release.yml:45-48`, `:266-267`, `:353-355`) |
| V5 Input Validation | yes | Tag/ref parsing; do not interpolate `${{ github.ref_name }}` into `run:` — pass through `env:` (the derive step already uses `$GITHUB_REF_NAME`, `release.yml:72-82`) |
| Supply chain | yes | Every action pinned to a full SHA (`release.yml:25-35`); the guard is a shell step, not a new action |

| Threat pattern | STRIDE | Mitigation |
|----------------|--------|------------|
| Tag name injected into shell | Tampering | `env:` indirection, quoted expansion (`set -euo pipefail`) |
| Unpinned action / over-broad token | Tampering / Elevation | Keep SHA pins, add no new actions; per-job `permissions`, `persist-credentials: false` |
| Wrong image tagged `:latest` | Tampering | Semver gate + exact `tag-names` assertion before push |
| Artifact substitution between jobs | Spoofing | Same-run `download-artifact`, explicit filenames + SHA256SUMS |

## Assumptions Log

| # | Claim | Section | Risk if wrong |
|---|-------|---------|---------------|
| A1 | Apple Silicon requires a valid (at least ad-hoc) signature to exec; external strip invalidates it | F5 | An external strip would ship a binary that dies with `killed: 9` |
| A2 | Cargo-level `strip` is applied at link time, so the linker signature covers the stripped output | F5/F6 | If false, every strip needs an explicit re-sign step |
| A3 | A timed-out job may not complete the post-job cache save | F7 | The "warming run" may warm nothing; the proof run stays cold |
| A4 | macOS `tar` does not add `._*`/xattr entries to this five-file archive | F5 | Tarball contents differ from the asserted list |
| A5 | `debug = 1` violates the "no DWARF" property even though no `.dSYM` is produced | F6 | REL-07 would be considered already satisfied when it is not |

## Open Questions

1. **Does the rc4 arm64 leg fail on time or on memory?** The log for run `40111152104` would answer it (`gh run view 40111152104 --log`). Needed to choose between timeout, LTO override, and both.
2. **Is `xai-grok-crash-handler` quality a requirement for release builds?** Decides `strip = "debuginfo"` vs `"symbols"`. Answerable by reading `crates/codegen/xai-crash-handler` and any backtrace expectations.
3. **Should `--locked` be added to the CI build?** It makes PKG-01 enforceable but turns any future manifest edit without a lock update into a red build. A one-line decision in the plan.
4. **Does the `version` job need a semver check for `v0.1`?** The `v*` trigger admits it today; nothing currently rejects it. Recommend yes (F3).

## Sources

### Primary (HIGH)
- `.github/workflows/release.yml` (read in full, 462 lines) — job graph, steps, guards, tag list, manifest loop.
- `Cargo.toml:106-108, 372-394`; `Cargo.lock` entries; the four `0.0.1-rc5` manifests; `crates/codegen/xai-grok-version/*` and `crates/codegen/xai-grok-pager-bin/{Cargo.toml,build.rs,src/main.rs}`.
- `install.sh:274-286`; `scripts/test-install.sh:24, 145-148`.
- `KNOWN-ISSUES.md:262-274, 345-357`; `PROJECT.md:121-127, 165`; `ROADMAP.md` §Phase 3; `STATE.md:74, 96-112`; `HANDOFF-2026-11-23.md:88`; `README.md:105-124`.
- `crates/codegen/xai-grok-shell/build.rs:71-98` (build-time ripgrep download per macOS triple) — a CI network dependency, not a new destination.

### Secondary (MEDIUM — official docs, fetched this session)
- cargo reference profiles (`strip`, `debug`, `split-debuginfo`, env/config overrides): `https://doc.rust-lang.org/cargo/reference/profiles.html`.
- rustc codegen options (`strip` "during linking", `split-debuginfo` "prevents … dsymutil"): `https://doc.rust-lang.org/rustc/codegen-options/index.html`.
- `docker/metadata-action` README and `src/meta.ts` at the pinned SHA: semver requirements, `procSemver`/`setVersion` latest rule, Major version zero, `tag-names` output.
- `docker/buildx` `build/build.go`: `tag is needed when pushing to registry`.
- GitHub-hosted runner specs (macOS arm64 3 vCPU/7 GB; Intel 4/14): `https://docs.github.com/en/actions/reference/runners/github-hosted-runners`.
- `Swatinem/rust-cache` README at the pinned SHA: job-id + manifest hash in the key, `cache-on-failure`.

### Tertiary (LOW / ASSUMED)
- Apple ad-hoc signing behaviour on arm64 and post-signing strip invalidation (A1/A2).
- Cache save on job cancellation (A3).
- macOS tar/xattr behaviour (A4).

## Metadata

**Confidence breakdown:**
- Standard stack / pipeline shape: HIGH — read from the tree, no ambiguity.
- Version mechanics: HIGH — three sites read; count discrepancy (83 vs 82) verified by script.
- macOS signature/strip and timeout levers: MEDIUM — docs support the ordering, but no macOS host to measure and run `40111152104`'s log would settle which lever suffices.

**Research date:** 2026-11-25
**Valid until:** 2027-01-25 for the Cargo/rustc/profile facts; **re-verify before execution** if `main` moves — `release.yml` and the four manifests are the only sources of truth, and the phase's own precondition (`ROADMAP.md`) says to re-verify them.
