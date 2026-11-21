# Stack Research — shipping `deepseek-build` v0.1.0

**Domain:** personal, DeepSeek-only terminal coding agent (Rust CLI + fullscreen TUI + headless),
distributed as four native tarballs plus one multi-arch container image
**Researched:** 2026-11-21
**Confidence:** HIGH on what is already settled; MEDIUM on the CI wall-clock arithmetic (see §2)
**Scope:** this is a *subsequent* milestone on a mature tree. The question is **not** "what
framework" — the framework layer is finished and locked. The question is **which stack and tooling
decisions are still open before a trustworthy freeze**, and what each should be.

Every claim below is traced to a file in this repository. Anything I could not read is marked
**UNVERIFIED** inline and never presented as fact.

---

## How to read this document

Two labels are used throughout:

- **[SETTLED]** — already decided and implemented in the tree. Listed once so the roadmap does not
  re-open it. Do not spend phase time here.
- **[OPEN]** — a decision this milestone still has to take. These are the actionable part.

The milestone's five target work items map onto the open decisions as:

| Target work | Decisions |
|---|---|
| Wire `route_turn` into the session sampler | §5 (routing surfaces), §9 (config docs) |
| Fix sandbox runtime-socket deny materialization | §8 |
| Both macOS arches green in CI | §1, §2, §3, §6, §7 |
| Build **and pull** a GHCR image | §4, §6 |
| Manifests → `0.1.0`, tag shape | §6 |

---

## 1. CI topology — [OPEN] · confidence HIGH

### The defect, precisely

`.github/workflows/release.yml` has **one** `build` job with a 4-leg matrix
(`x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, `aarch64-apple-darwin`,
`x86_64-apple-darwin`) and a single `timeout-minutes: 90` at the **job** level
(`release.yml:97-113`). `container` and `publish` both declare `needs: [version, build]`
(`release.yml:263`, `release.yml:349`).

Consequence, confirmed by the recorded runs: when the `aarch64-apple-darwin` leg hit the 90-minute
timeout, the whole `build` job failed, so **`publish` and `container` were both skipped** —
`v0.0.1-rc4` shipped Linux-only by hand and `v0.0.1-rc5` has no macOS assets and no GHCR image
(`KNOWN-ISSUES.md:195-206`, `CONCERNS.md:198-201`).

A `needs:` edge cannot point at a single matrix leg. **The only way to stop a slow macOS leg from
killing the container image is to split the build into two jobs.**

### Recommendation

Split into `build-linux` (2 legs) and `build-macos` (2 legs), then:

```
container  needs: [version, build-linux]        # no macOS dependency
publish    needs: [version, build-linux, build-macos]
```

and give each matrix leg its own budget via a matrix field:

```yaml
strategy:
  fail-fast: false
  matrix:
    include:
      - target: aarch64-apple-darwin
        runner: macos-15
        timeout: 180        # macOS gets headroom
      - target: x86_64-unknown-linux-gnu
        runner: ubuntu-22.04
        timeout: 90
# job level:
timeout-minutes: ${{ matrix.timeout }}
```

**Why this and not something else:** `fail-fast: false` is already set, so the other legs keep
building when one is slow; the problem is purely that `needs` gates everything on the *whole*
matrix. Splitting is the minimal change that makes the container image and the Linux tarballs
independent of macOS wall-clock time. Raising the timeout alone fixes nothing structural — a
cancelled macOS leg would still skip the image.

**Prerequisite (operational, not technical):** editing `.github/workflows/*` requires a token with
the `workflow` scope. `KNOWN-ISSUES.md:201-202` records that this blocked the per-target timeout
change after rc4. Schedule the token before the pipeline phase, or the whole macOS/container item
stalls. **This is the single highest-risk unknown in the milestone.**

---

## 2. Distribution build profile — [OPEN] · confidence HIGH

### What ships today is not what the comments describe

`Cargo.toml:388-394`:

```toml
[profile.release-dist]
inherits = "release"      # panic = "abort", strip = "symbols"
lto = "thin"
codegen-units = 1
strip = false
debug = 1
split-debuginfo = "off"
```

The comment above it (`Cargo.toml:385-387`) says `strip = false` exists "so CI can extract .debug
sidecars (Linux) or .dSYM bundles (macOS) before stripping post-build". **No such step exists in
`release.yml`** — grepping the workflow for `strip`, `dsymutil`, `split-debug`, or `debug` returns
nothing. The workflow tars `target/release-dist/deepseek-build` directly
(`release.yml:219`, `release.yml:243`).

So every published tarball and the container image carry full line-table DWARF. That is:

- larger tarballs and a larger GHCR image (the binary is already ~147 MiB **without** LTO — measured
  at `--release`, `KNOWN-ISSUES.md:64-65` and `CONCERNS.md:148`),
- slower `actions/upload-artifact` / `download-artifact` / GHCR push,
- a debug-info disclosure in a public artifact.

### Recommendation

Pick one and make it true:

- **Preferred:** `strip = "symbols"`, `debug = 0`, drop `split-debuginfo`. Simplest, smallest,
  matches the stated "personal tool" intent. Debugging a released binary is a nice-to-have this
  project has never had.
- **If sidecars are wanted:** keep `debug = 1` and add the missing `objcopy --only-keep-debug` /
  `dsymutil` + `strip` step before packaging. This costs CI minutes on the leg that is already
  over budget — so it is the wrong trade for macOS arm64.

### The other half: LTO and `codegen-units = 1` on a 3-core runner

This is the actual cause of the 90-minute timeout. Evidence from the author's own measurements
(`KNOWN-ISSUES.md:278-287`):

| Configuration | Result on the author's box |
|---|---|
| `--release`, no LTO, `-j 2` | 152,670,552 bytes, **28m 38s** |
| `lto = "fat"`, `cgu = 1`, `-j 2` | OOM-killed (16 GB) |
| `lto = "thin"`, `-j 1` | still compiling after **40 min**, never linked |

A 95-member / 1,246-crate workspace (`Cargo.lock`) with `codegen-units = 1` serializes the largest
crates. On a small arm64 macOS runner (recalled as 3 cores — **UNVERIFIED**, not recorded anywhere
in this repo) that is plausibly beyond 90 minutes even with a warm cache.

**Cargo lets you override profile keys per job without touching `Cargo.toml`.** For the
`aarch64-apple-darwin` leg only:

```yaml
env:
  CARGO_PROFILE_RELEASE_DIST_LTO: "false"
  CARGO_PROFILE_RELEASE_DIST_CODEGEN_UNITS: "16"
```

**Why acceptable:** each target is a separate binary with no cross-target ABI contract; Linux keeps
the LTO build. **Why not the default everywhere:** the `release-dist` profile is also what makes
the Linux artifact the size it is, and Linux legs finish comfortably.

**Ordering of remedies** (cheapest first): (1) split the jobs so the image is unblocked, (2) fix
`strip`, (3) per-leg timeout, (4) env-override LTO/CGU on darwin-arm64 only. Do (1) and (2)
regardless — they are correctness fixes, not workarounds.

---

## 3. Runner matrix — [SETTLED] · confidence HIGH

| Target | Runner | Note |
|---|---|---|
| `x86_64-unknown-linux-gnu` | `ubuntu-22.04` | older glibc base, matches `bookworm-slim` runtime |
| `aarch64-unknown-linux-gnu` | `ubuntu-22.04-arm` | native arm, no QEMU |
| `aarch64-apple-darwin` | `macos-15` | native Apple Silicon |
| `x86_64-apple-darwin` | `macos-15-intel` | native Intel; `macos-13` is retired |

Native-per-target is the right call and is already implemented (`release.yml:92-113`). The
`.github/actionlint.yaml` already declares `macos-15-intel` so actionlint 1.7.7 does not reject the
label. **Do not introduce cross-compilation or QEMU** — `aws-lc-sys` (0.39.1, cmake-driven),
`tikv-jemalloc-sys` (0.7.1, needs `make`), and vendored `libgit2-sys` (0.18.2) all make cross builds
a worse problem than the timeout.

---

## 4. GitHub Actions pins — [SETTLED] · confidence HIGH

All third-party actions are pinned to full commit SHAs, which is the correct posture and should not
be relaxed (`release.yml:24-35`). As recorded in the file:

| Action | Version | SHA (from `release.yml`) |
|---|---|---|
| `actions/checkout` | v7.0.1 | `3d3c42e5aac5ba805825da76410c181273ba90b1` |
| `actions/upload-artifact` | v7.0.1 | `043fb46d1a93c77aae656e7c1c64a875d1fc6a0a` |
| `actions/download-artifact` | v8.0.1 | `3e5f45b2cfb9172054b4087a40e8e0b5a5461e7c` |
| `Swatinem/rust-cache` | v2.9.2 | `6323deb102c322ba6fcbdcafc7e3dddab59af2b6` |
| `softprops/action-gh-release` | v3.0.3 | `efb35369e0ad2afab669f228072c1b0d510eae64` |
| `docker/setup-buildx-action` | v3.12.0 | `8d2750c68a42422c14e847fe6c8ac0403b4cbd6f` |
| `docker/login-action` | v3.7.0 | `c94ce9fb468520275223c153574b00df6fe4bcc9` |
| `docker/metadata-action` | v5.10.0 | `c299e40c65443455700f0fdfc63efafe5b349051` |
| `docker/build-push-action` | v6.19.2 | `10e90e3645eae34f1e60eeb005ba3a3d33f178e8` |

**The workflow's upload allowlist is a hard constraint and stays as written**
(`release.yml:9-23`): `upload-artifact` for intra-run transport, the GitHub release, and GHCR only.
No S3, no codecov, no external mirror. Nothing in this research proposes a new one.

### [OPEN] One unnecessary unpinned fetch

The Linux leg installs Rust with `curl --proto '=https' ... https://sh.rustup.rs | sh`
(`release.yml:165`), while the macOS leg uses the cleaner
`rustup toolchain install 1.94.0 --profile minimal` (`release.yml:172`). GitHub-hosted runners ship
rustup on both OSes, so the Linux `curl | sh` is an avoidable unpinned network fetch and a shell-pipe
supply-chain surface.

**Recommendation:** use `rustup toolchain install 1.94.0 --profile minimal` on both legs. Removes a
fetch, keeps `rust-toolchain.toml` as the single source of truth. Confidence HIGH.

### [OPEN] Caching shape

`Swatinem/rust-cache@v2.9.2` with `key: ${{ matrix.target }}` and `cache-on-failure: true`
(`release.yml:175-179`) is the right starting point. Two cautions:

- GitHub's per-repository cache budget is finite (recalled as 10 GB — **UNVERIFIED**). Four
  `target/` directories from a workspace whose *test* build is ~24 GB (`CONCERNS.md:141-145`) will
  thrash. Add `save-if:` so tag runs **restore** but do not **save** (the branch build owns the
  cache), which stops four tag legs from evicting each other mid-run.
- Do **not** add `sccache` with a GitHub Actions cache backend without an explicit decision: the
  workflow's upload allowlist does not name it, even though `rust-cache` already uses the same
  GitHub cache service. Treat it as a constraint question, not a free optimization.

---

## 5. Model routing surfaces — [OPEN] · confidence HIGH

This is the "stack" part of the routing item: what code and config surface must exist, with no new
crates.

### What exists

`crates/codegen/xai-grok-sampler/src/model_routing.rs` is complete and pure:
`ModelRoutingPolicy { Off (default), Low, High, Auto }`, `TurnComplexity { Trivial, Complex }`,
`ModelRoute { model, reasoning_effort, thinking }`, `ModelRoutingConfig { policy, flash_model,
pro_model, pro_effort, flash_effort, manual_override }`, plus `classify_turn` and `resolve_route`.
`manual_override` wins over the policy — the pin semantics the milestone wants are already encoded.

`SamplingClient::route_turn(&self, request, manual_override) -> Option<ModelRoute>`
(`client.rs:611-623`) is the only consumer, and nothing outside that module calls it.

### What is missing (three concrete gaps)

1. **No caller.** All four shell construction sites pass `model_routing: Default::default()`
   (`xai-grok-shell/src/session/acp_session_impl/sampler_turn.rs:729`,
   `src/agent/config.rs:4328`, `src/agent/subagent/mod.rs:779`, `src/tools/config.rs:212`). The turn
   path that must call `route_turn` is `sampler_turn.rs`.
2. **No user setting.** `grep model_routing` across `xai-grok-config` and `xai-grok-config-types`
   returns nothing — the policy is not in the config registry, so a user cannot select `auto` in
   `config.toml`. Wiring it as a user setting means a new registry key, a hand edit to
   `crates/codegen/xai-grok-pager/docs/user-guide/26-config-reference.md`, and a green
   `config_docs` run (`AGENTS.md` §8, `CONCERNS.md:46-50`).
3. **No status-line surface.** `StatusLineItem` has `Cwd, Model, Context, Cost, TurnTimer,
   SessionName, Effort, Tokens, Cache` (`xai-grok-status-line/src/config.rs:345-358`) — no
   "routed" concept. Either let the existing `Model` item render the *routed* model for the turn
   (cheapest, no new config key, no doc churn) or add a variant (enum + `parse` +
   `varies_mid_turn` + config reference + tests).

### Semantics to preserve, and one subtlety to document

`route_turn` sets `pro_model: self.defaults.model.clone()` — deliberately, per its comment: routing
never silently changes the base model id, so it only *downgrades* to Flash. With the default session
model `deepseek-v4-pro` that yields the intended Flash-vs-Pro split. **But if a session is pinned to
`deepseek-flash`, the "pro" route is also Flash.** That is consistent with "a pin wins", and it
should be stated in the docs rather than discovered.

**Recommendation:** wire the caller in `sampler_turn.rs`, expose `policy` as a config key
(`[model_routing] policy = "auto" | "off" | "low" | "high"`), render the routed model through the
existing `Model` status item, and have `/model` / `/effort` populate `manual_override`. No new
crates, no new outbound destination. Confidence HIGH.

---

## 6. Versioning, tag shape, and the image — [OPEN] · confidence HIGH

### 6.1 Manifests to `0.1.0` is a 4-file change, not 95

`[workspace.package]` has **no** `version` key (`Cargo.toml:106-108`), so each member carries its
own. Measured across the tree:

| Version | Count | Where |
|---|---|---|
| `0.1.0` | 82 | most members — already correct |
| `0.0.1-rc5` | 4 | `xai-grok-pager-bin`, `xai-grok-pager`, `xai-grok-shell`, `xai-grok-version` |
| `0.1.220-alpha.4` | 3 | `xai-grok-workspace`, `xai-grok-tools`, `xai-grok-tools-api` |
| `0.0.0` | 2 | `xai-proto-build`, `xai-grok-markdown/fuzz` |

**Recommendation:** add `version = "0.1.0"` to `[workspace.package]` and switch the four shipping
crates to `version.workspace = true`. That is a bounded 5-line change, keeps one place to bump next
time, and leaves the upstream leftovers alone (they are not shipped and are not in scope).
Regenerate `Cargo.lock`. `xai-grok-version` already falls back to `CARGO_PKG_VERSION`
(`xai-grok-version/src/lib.rs:13-16`), so a source build then prints `0.1.0` and a release build
still takes the tag via `GROK_VERSION`.

**Do not** try to convert all 92 member manifests — that is a 92-file churn with no user-visible
benefit and it collides with the freeze.

### 6.2 The tag shape breaks four places today

Current trigger and parser (`release.yml:39-90`) assume a bare `v` prefix:

```yaml
on: { push: { tags: ['v*'] } }
...
case "$GITHUB_REF_NAME" in v?*) version="${GITHUB_REF_NAME#v}" ;;
```

A `deepseek-v0.1.0` tag fails four separate ways:

| Location | Failure |
|---|---|
| `release.yml:42` trigger | `deepseek-v*` does not match `v*` → **workflow never runs** |
| `release.yml:74-78` | even if triggered, `v?*` does not match → `::error::release tags must match 'v*'` and `exit 1` |
| `release.yml:86` prerelease regex `^[0-9]+\.[0-9]+\.[0-9]+-` | `deepseek-v0.1.0-rc1` would be classified **stable** (`is_prerelease=false`) — a real bug |
| `docker/metadata-action` `type=semver` (`release.yml:407-409`) | `deepseek-v0.1.0` does not parse → no `{{version}}`/`{{major}}.{{minor}}`/`{{major}}` tags; only `type=ref,event=tag` survives |
| `install.sh:274-281` | `v[0-9]*` validation rejects `deepseek-v0.1.0`, and the URL is built as `download/v$VERSION_NUM/...` |

**Recommendation — normalize in exactly one place.** In the `version` job, derive `VERSION` from a
single regex that accepts both shapes:

```
^(?:deepseek-)?v?([0-9]+\.[0-9]+\.[0-9]+(?:[-+].*)?)$
```

Add `deepseek-v*` to the trigger list. Run the prerelease test against the *captured* version
(not the raw ref), so `deepseek-v0.1.0-rc1` is correctly a prerelease. In the container job, replace
`type=semver` with explicit `type=match` patterns (or pass the normalized version as an explicit
`type=raw,value=`), because `type=semver` cannot see a `deepseek-` prefix. Extend `install.sh`'s
`case` to accept `deepseek-v*` and strip the prefix before building the download URL.

**Alternative worth stating:** keep release tags `v0.1.0` and use `deepseek-v0.1-snapshot` **only**
as the freeze marker on the `deepseek` branch (it must not match the release trigger). That is
zero code change but does not satisfy "release.yml fires on the DeepSeek tag shape". Support both;
the author then chooses at publish time.

### 6.3 Container image — [OPEN] · confidence HIGH

The design is sound and needs no re-architecture: `debian:bookworm-slim` (glibc ~2.36, comfortably
older than the `ubuntu-22.04` build base's glibc 2.35 — correct direction), non-root uid 1000,
`ENTRYPOINT ["deepseek-build"]`, per-arch native buildx legs pushed by digest then assembled with
`docker buildx imagetools create` (`docker/Dockerfile`, `release.yml:347-462`). The Dockerfile never
compiles Rust, which is exactly right — CI hands it the tarball.

Five fixes, in priority order:

1. **Decouple from macOS** — §1. Without this the image is never built (rc4/rc5 evidence).
2. **Pin the base image by digest.** `FROM debian:bookworm-slim` is a floating tag; a silent base
   change is not compatible with a "trustworthy freeze". Use
   `FROM debian:bookworm-slim@sha256:<digest>` with a dated comment. The digest must be fetched at
   implementation time (this research has no network) — **value UNVERIFIED by design**.
3. **Guard the manifest loop.** `release.yml:450-454` iterates `steps.meta.outputs.tags`; if that
   list is empty (the `type=semver` failure in §6.2) the loop is a no-op and the job still exits 0
   with nothing pushed. Add a `[ -n "$tags" ]` assertion before the loop.
4. **Prove the pull, in the job that pushed.** After `imagetools inspect`, run
   `docker run --rm "${IMAGE}:${VERSION}" --version` and assert the version string. That satisfies
   "built **and pulled**" without adding a new destination — it reads back from the same GHCR the
   job just wrote.
5. **Fix the binary that goes in.** With `strip = false, debug = 1` (§2) the image carries DWARF.
   Fixing §2 fixes the image size for free.

Also note: the image is headless-only (no TTY). Say so in the README GHCR block, which currently
shows `docker run -it --rm` examples (`README.md:153-175`).

---

## 7. Linux binary hardening — [OPEN] · confidence HIGH

`CONCERNS.md:91-95` is correct and unaddressed: `[profile.release-dist]` says link hardening lives
per-target in `.cargo/config.toml`, but `-Wl,-z,relro,-z,now,-z,noexecstack` exists **only** under
the two `*-unknown-linux-musl` sections (`.cargo/config.toml:48-67`). The shipped targets are
`*-unknown-linux-gnu`, whose rustflags are only `force-unwind-tables` (plus `target-cpu=neoverse-v2`
on arm64, `.cargo/config.toml:39-46`).

**Recommendation:** add the same `link-arg` to both gnu sections. Compatible with the
`scripts/cargo-linker.sh` wrapper (it prefers lld and falls back to `cc`; both accept `-z` flags).
Verify on the built tarball with `readelf -l` (`GNU_RELRO` present) and `readelf -d`
(`BIND_NOW`/`FLAGS`). This is a one-line-per-target change with a two-command proof — cheap and it
closes a recorded security finding before a public freeze.

---

## 8. Sandbox runtime-socket denies — [OPEN] · confidence HIGH

### Root cause, in code

`materialize_runtime_socket_deny_paths_from` (`xai-grok-sandbox/src/runtime_sockets.rs:97-152`)
canonicalizes each deny candidate's **parent** and treats only `NotFound` as "skip". Every other
error is returned. On a host where `/run/containerd` is root-owned, `dunce::canonicalize("/run/containerd")`
returns `EACCES` → `Err` → `bwrap` refuses to start a partial sandbox → the two
`deny_paths_e2e` failures recorded in `KNOWN-ISSUES.md:21-32`.

The static candidate list includes `/run/containerd/containerd.sock` and
`/var/run/containerd/containerd.sock` (`runtime_sockets.rs:17-25`), so this is hit on any host with
a root-only containerd runtime directory — a common Linux layout, not a contrived case.

### Recommendation

Treat `PermissionDenied` on the **parent** the same way `NotFound` is treated, but only when the
endpoint itself is not visible to this process:

```
canonicalize(parent) -> Err(EACCES):
    symlink_metadata(candidate)
        Err(NotFound | EACCES)  -> skip candidate      // we could never mask it anyway
        Ok(_)                   -> return Err(...)     // visible but unmaskable: fail closed
        Err(other)              -> return Err(other)
```

This is a new match arm mirroring the existing `NotFound` branch, so it preserves the invariant that
matters: **a deny path the process can see but cannot mask still fails closed.** It only stops
aborting on paths the process cannot see.

**No new dependency.** `libc` is already used in this module (`getuid`), and `dunce::canonicalize`
is already the sanctioned call (`clippy.toml` bans the `std` version).

**Test without root:** create `tmp/deny/containerd.sock`, `chmod 0o000` the `tmp/deny` directory
(`std::os::unix::fs::PermissionsExt`), and assert the function skips that candidate. Owner
permission bits deny traversal on Linux, so this is deterministic and needs no `chown`. Add the
inverse case — a directory the test *can* traverse but whose endpoint is unreadable — asserting
`Err`, so the fail-closed path stays pinned. Confidence HIGH on the diagnosis; MEDIUM on the exact
test fixture (POSIX semantics recalled, not read).

---

## 9. Config-docs coupling — [SETTLED, with a tripwire] · confidence HIGH

`crates/codegen/xai-grok-pager/docs/user-guide/26-config-reference.md` is hand-edited and must be
updated in the same commit as any change to `FEATURES`, `UNMIRRORED_BOOLEAN_FEATURES`, or
`KNOWN_MCP_SERVER_FIELDS`; removed-flow keys go in `UNDOCUMENTED_REMOVED_FLOW_KEYS` instead
(`AGENTS.md` §8, `CONCERNS.md:46-50`). The `config_docs` tests (`--features config-docs`) enforce it.

**Implication for this milestone:** the new `[model_routing]` key (§5) and any new status-line item
both land in that file, in the same commit, with the `config_docs` tests run. This is not a decision —
it is a sequencing constraint the roadmap should encode in the routing phase's acceptance criteria.

---

## 10. Toolchain provisioning — [SETTLED] · confidence HIGH

- Rust **1.94.0**, edition 2024, pinned in `rust-toolchain.toml`. Do not bump (D8,
  `PROJECT.md:124`). `rust-toolchain.toml` lists only the two Linux targets; native macOS builds do
  not need them listed, so no change.
- `protoc` is hard-required by `crates/build/xai-proto-build` when `GITHUB_ACTIONS` is set; CI sets
  `PROTOC` explicitly to avoid the repo's dotslash wrapper (which would attempt a fetch)
  (`release.yml:123-155`).
- `cmake` is needed by `aws-lc-sys` 0.39.1; `make` by `tikv-jemalloc-sys` 0.7.1
  (`KNOWN-ISSUES.md:264-266`).
- `cargo-deny` 0.20.2 with `deny.toml` covering all four release targets — already correct and
  already aligned with the release matrix (`deny.toml:4-14`). No CI job runs it; that is a local
  gate today.

**One small robustness fix [OPEN]:** the macOS dependency step says "cmake is preinstalled" and only
installs protobuf (`release.yml:144-155`). Make it `brew install protobuf cmake` and assert both
paths — idempotent, and it removes a silent dependency on runner-image contents.

---

## Recommended stack at a glance

| Concern | Choice | Status | Confidence |
|---|---|---|---|
| Language / toolchain | Rust 1.94.0, edition 2024 | SETTLED (D8) | HIGH |
| CI topology | Split `build-linux` / `build-macos`; container needs linux only | **OPEN — do this** | HIGH |
| Dist profile | `strip = "symbols"`, `debug = 0`; drop the unfulfilled sidecar comment | **OPEN — do this** | HIGH |
| macOS arm64 time | matrix `timeout` + `CARGO_PROFILE_RELEASE_DIST_LTO=false` / `CGU=16` on that leg | **OPEN** | MEDIUM |
| Runners | 4 native legs, no cross/QEMU | SETTLED | HIGH |
| Actions | SHA-pinned set above; unify rustup install | SETTLED / small OPEN | HIGH |
| Caching | `rust-cache` per-target, `save-if` branch-only | **OPEN** | MEDIUM |
| Container base | `debian:bookworm-slim` **pinned by digest** | SETTLED / pin OPEN | HIGH |
| Image proof | `docker run … --version` in the pushing job | **OPEN — do this** | HIGH |
| Tag shape | One normalization regex, both `v*` and `deepseek-v*` | **OPEN — do this** | HIGH |
| Version | `[workspace.package] version = "0.1.0"` + 4 crates inherit | **OPEN — do this** | HIGH |
| Linux hardening | RELRO/NX link-args on both gnu targets | **OPEN — do this** | HIGH |
| Sandbox denies | Skip `EACCES` parent, keep fail-closed when endpoint is visible | **OPEN — do this** | HIGH |
| Routing surfaces | Caller + config key + existing `Model` item + `/model` pin | **OPEN** | HIGH |
| New crates | **None** | — | HIGH |

---

## Installation / verification commands

```sh
# macOS / Linux CI toolchain (replaces the curl|sh pipe)
rustup toolchain install 1.94.0 --profile minimal

# dist build with per-leg overrides (no manifest edit)
CARGO_PROFILE_RELEASE_DIST_LTO=false \
CARGO_PROFILE_RELEASE_DIST_CODEGEN_UNITS=16 \
  cargo build --profile release-dist -p xai-grok-pager-bin

# prove Linux hardening on the artifact
readelf -l target/release-dist/deepseek-build | grep GNU_RELRO
readelf -d target/release-dist/deepseek-build | grep -E 'BIND_NOW|FLAGS'

# sandbox deny materialization, targeted
cargo test -p xai-grok-sandbox --lib runtime_sockets
cargo test -p xai-grok-sandbox --test deny_paths_e2e

# routing
cargo test -p xai-grok-sampler --lib model_routing
cargo test -p xai-grok-status-line --lib

# config docs (after adding [model_routing])
cargo test -p xai-grok-shell --features config-docs

# image proof, in the pushing job
docker run --rm "ghcr.io/<owner>/deepseek-build:${VERSION}" --version

# local pipeline gates
actionlint .github/workflows/*.yml
shellcheck --shell=sh install.sh scripts/test-install.sh
sh scripts/test-install.sh        # 39/39
cargo-deny check licenses bans sources
```

---

## Alternatives considered

| Recommended | Alternative | Why not (or when it would be right) |
|---|---|---|
| Split build jobs | Raise the single 90-min timeout | Still gates `container` and `publish` on the slowest leg; a timeout skips the image. Only correct if nothing else consumes the build. |
| Split build jobs | `continue-on-error` on the macOS leg | Publishes a release silently missing a promised target — the exact rc5 outcome the milestone exists to end. |
| `strip = "symbols"` | Ship `.debug`/`.dSYM` sidecars | Nothing consumes them today; costs CI minutes on the leg already over budget. |
| Env-override LTO off for darwin-arm64 | Remove `release-dist` everywhere | Loses the size win on Linux, which finishes comfortably. |
| Native arm64 runner | Cross-compile `aarch64-apple-darwin` from `macos-15-intel` | `aws-lc-sys`/`jemalloc`/`libgit2-sys` cross builds are a larger problem than the timeout; the workflow's "no cross-compilation" stance is correct. |
| One normalization regex for tags | Support only `deepseek-v*` | Breaks the already-published `v0.0.1-rc*` lineage and `install.sh`'s `VERSION=v0.1.0` contract. |
| `[workspace.package] version` | Convert all 92 manifests | 92-file churn, zero user-visible gain, collides with the freeze. |
| Skip `EACCES` parents | Skip all denies when the parent is unreadable | Removes the masks entirely on such hosts — weaker than the intended default. |
| Existing `Model` status item | New `routed` status item | Extra enum variant + config reference + docs churn for one string; add it later if users want both visible. |

---

## What NOT to use

| Avoid | Why | Use instead |
|---|---|---|
| Cross-compilation or QEMU for any of the four targets | Native legs already exist and are correct; C build deps make cross builds brittle | The four native runners in `release.yml:105-113` |
| `lto = "fat"` anywhere | OOM-killed on the author's 16 GB box at `-j 2` (`KNOWN-ISSUES.md:281-282`) | `thin`, or no LTO on the arm64 mac leg |
| A per-leg `continue-on-error` | Ships a release missing an advertised target | Fix the leg, or split so the other artifacts ship independently |
| `sccache` with a GitHub Actions cache backend | Not on the workflow's upload allowlist (`release.yml:9-23`); needs an explicit decision | `Swatinem/rust-cache` as configured |
| Any new telemetry, codecov, S3, or external mirror step | Violates the hard constraint at the top of `release.yml` and D6/D10/D13 | Nothing — the allowlist is complete |
| `type=semver` alone with a `deepseek-` tag | Silently produces no version tags | `type=match` or explicit `type=raw,value=` |
| Unpinned base image (`FROM debian:bookworm-slim`) | A base change silently alters a frozen artifact | Digest pin with a dated comment |
| Runtime pricing/catalog fetch | New outbound destination; forbidden | The shipped price table (D14) |
| Renaming `xai-grok-*` crates or `GROK_*` env names | Locked by D2 / deferred by the rc5 decision | Leave them; only the user-facing surface is branded |
| Recreating any deleted exfil/telemetry/upload path | Those deletions are the egress guarantee | Do not |

---

## Stack patterns by variant

**If the author decides `deepseek-v*` is the only release shape from v0.1.0 onward**
→ Keep the normalization regex anyway (it costs three lines) so `VERSION=v0.1.0 sh install.sh`
and any re-run of an rc tag still work. Do not delete the `v*` branch of the trigger until the
`v0.0.1-rc*` assets are irrelevant.

**If the macOS arm64 leg is still over budget after the profile overrides**
→ Raise only that leg's `timeout` (matrix field), and leave `container` and `publish` edges as in §1.
Never let the leg be the reason the image is skipped.

**If the freeze slips and macOS must be cut**
→ Ship `build-linux` + `container` and make `publish` `needs` only what was built, with the tarball
verification loop reduced to the targets actually produced. This is a deliberate scope cut, not a
timeout failure — record it in `DECISIONS.md` if taken. `PROJECT.md:150` currently says both macOS
arches ship in v0.1.0, so this requires a decision change.

**If `route_turn` wiring turns out to need per-turn model ids on the wire**
→ That is an adapter change in `xai-grok-sampler`, not a new crate. The request already carries
`model`; `ModelRoute.model` is the value to apply. Keep it on the existing Chat Completions path.

---

## Version compatibility

| Component | Compatible with | Notes |
|---|---|---|
| Rust 1.94.0 | edition 2024, all workspace crates | Pinned (D8). Do not bump. |
| `release-dist` profile | `--profile release-dist` only | Default `--release` is `strip = "symbols"`, `panic = "abort"`, no LTO — local artifact, not the dist one. |
| `debian:bookworm-slim` (glibc ~2.36) | `ubuntu-22.04` build base (glibc 2.35) | Correct direction: build on older, run on newer. |
| `aws-lc-sys` 0.39.1 | cmake on the runner | Linux installs it; macOS assumes it — assert it (§10). |
| `tikv-jemalloc-sys` 0.7.1 | `make` present | Fails without `make` (`KNOWN-ISSUES.md:264-266`). |
| `protoc` (apt on 22.04 / brew on macOS) | `xai-proto-build` | Must be exported as `PROTOC`; the dotslash wrapper would fetch. |
| `docker/build-push-action` v6.19.2 + buildx | per-arch native push + `imagetools create` | No QEMU. |
| `actions/upload-artifact` v7 / `download-artifact` v8 | `merge-multiple: true` pattern `release-*` | Already used in `publish`. |

---

## Open questions the roadmap must resolve

1. **`workflow`-scope token.** Every change in §1/§2/§6/§7 edits `.github/workflows/release.yml`.
   `KNOWN-ISSUES.md:201-202` records that this blocked the previous attempt. If the token is not
   available, the entire macOS/container/tag-shape block cannot land — schedule it first.
2. **Release tag shape, final answer.** Both shapes are supportable (§6.2); the author picks which
   one v0.1.0 uses. This determines the `docker/metadata-action` tag set and the README's pull
   example.
3. **Debug sidecars: wanted or not?** §2 assumes not. If they are, the extraction step must be
   written and budgeted, and the macOS arm64 leg needs correspondingly more headroom.
4. **Cache budget.** The 10 GB per-repo cache figure is recalled, not read (**UNVERIFIED**). Before
   tuning `rust-cache`, confirm it against the repository's actual cache usage; the `save-if`
   recommendation is safe either way.
5. **`ci.yml`.** There is no test workflow — `release.yml` is the only one. `PROJECT.md:62` wants
   "`cargo test --workspace` exits 0 on the author's host", which does not require CI. Adding a
   `fmt`/`clippy`/test workflow is defensible but is a scope addition with the same `workflow`-scope
   prerequisite. Decide explicitly rather than by default.

---

## Sources

All read in this repository on 2026-11-21; no external network was used.

| File | What was taken |
|---|---|
| `.planning/PROJECT.md` | Active requirements, out-of-scope, constraints, key decisions |
| `.planning/codebase/STACK.md` | Settled stack (languages, runtime, frameworks, pins) — deliberately not repeated |
| `.planning/codebase/CONCERNS.md` | RELRO gap, release-CI capacity, routing gap, config-docs coupling, sandbox failure mode |
| `.github/workflows/release.yml` | Trigger, matrix, timeouts, `needs` edges, action pins, container job, upload allowlist |
| `docker/Dockerfile`, `docker/Dockerfile.localtest` | Base image, tool set, uid, `TARGETARCH` staging contract |
| `.cargo/config.toml` | Per-target rustflags, missing gnu hardening, `incremental = false`, `RUST_MIN_STACK` |
| `Cargo.toml` | `[workspace.package]`, `[profile.release-dist]`, `[patch.crates-io]` |
| `Cargo.lock` | 1,246 packages; `aws-lc-sys` 0.39.1, `tikv-jemalloc-sys` 0.7.1, `libgit2-sys` 0.18.2 |
| `rust-toolchain.toml` | 1.94.0 pin, declared targets |
| `deny.toml` | cargo-deny 0.20.2, four-target graph, license/source policy |
| `install.sh` | `VERSION` normalization, download URLs, `v[0-9]*` validation |
| `KNOWN-ISSUES.md` | rc2–rc5 run history, LTO measurements, `make` requirement, `workflow`-scope blocker |
| `DECISIONS.md` | D1–D14, especially D2, D8, D10, D13, D14 |
| `crates/codegen/xai-grok-sampler/src/model_routing.rs`, `src/client.rs` | Routing types, `route_turn`, `pro_model` semantics |
| `crates/codegen/xai-grok-sampler/src/config.rs`, `xai-grok-shell/src/**` | Four `Default::default()` call sites |
| `crates/codegen/xai-grok-status-line/src/config.rs` | `StatusLineItem` variants and `varies_mid_turn` |
| `crates/codegen/xai-grok-sandbox/src/runtime_sockets.rs` | Deny candidate list, materialization error handling |
| `crates/codegen/xai-grok-version/src/lib.rs` | `GROK_VERSION` → `CARGO_PKG_VERSION` fallback |
| `crates/codegen/xai-grok-shell/build.rs` | Build-time ripgrep download and per-target asset mapping |
| `crates/codegen/xai-grok-pager-bin/Cargo.toml`, per-crate manifests | Version inventory (82 × `0.1.0`, 4 × `0.0.1-rc5`) |
| `.github/actionlint.yaml` | actionlint 1.7.7, `macos-15-intel` declaration |

**Explicitly UNVERIFIED (recalled, not read here):** macOS runner core count and memory; GitHub's
per-repository cache size limit; `docker/metadata-action` semver parsing behavior for a
`deepseek-`-prefixed tag; macOS Gatekeeper/quarantine behavior for tarball downloads; Cargo
`CARGO_PROFILE_<PROFILE>_<KEY>` env-override support (standard, but not demonstrated in this tree).

---
*Stack research for: `deepseek-build` v0.1.0 freeze*
*Researched: 2026-11-21*
