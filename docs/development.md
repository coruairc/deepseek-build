<!-- generated-by: gsd-doc-writer -->

# Development

How to build, test, and gate a change to `deepseek-build`. The manual checklist with expected output is
[`../TESTING.md`](../TESTING.md); the plan is linked in [§8](#8-where-the-plan-lives), not restated here.

## 1. Toolchain and prerequisites

- **Rust 1.94.0**, pinned by `rust-toolchain.toml` (components `rustfmt` and `clippy`; linux `x86_64` and `aarch64` targets). Do not bump it — every CI build job installs exactly 1.94.0.
- **`protoc` on `PATH`.** `crates/build/xai-proto-build` fails when `GITHUB_ACTIONS` is set and no `protoc` is found; CI installs it and exports `PROTOC`. The repo's `bin/protoc` dotslash wrapper must not be used in CI (dotslash is absent there and it would try a network fetch).
- **`cc` and `make`.** `tikv-jemalloc-sys` builds jemalloc from source and fails without `make`.
- **`jq`** (the gate's test-binary enumeration), **`python3`** (`scripts/evidence.py`, release checkers), **`flock`** (build lock), **`actionlint` + `shellcheck`** (release pipeline checks).

`.cargo/config.toml` sets `[build] incremental = false` so debug/test artifacts stay bounded. `[profile.dev]` says `incremental =
true`, but `[build]` overrides it — intentional, do not drop it. `[profile.dev]`/`[profile.test]` use `debug = "line-tables-only"`,
and third-party crates get no debug info. A fresh `cargo build --workspace --tests` is about 24 GB in about 7 minutes (measured
2026-10-12); the old settings accumulated 244 GB of `target/debug`.

Before any workspace build run `scripts/disk-guard.sh`: it prints free space and exits non-zero below its threshold (default 60 GB)
on the filesystem holding `CARGO_TARGET_DIR` or `./target`. Point `CARGO_TARGET_DIR` at one shared `target/` across worktrees
instead of building one per checkout.

## 2. Build

```bash
source "$HOME/.cargo/env"
cargo check -p xai-grok-pager-bin          # fast loop
cargo build --release -p xai-grok-pager-bin
./target/release/deepseek-build --version
```

`[profile.release]` sets `panic = "abort"` and `strip = "symbols"`; the artifact is `target/release/deepseek-build` (about 147 MiB
at the last measured linux-x86_64 build). A source build prints `deepseek-build 0.1.0 (<commit>)`; the release pipeline injects a
tag version instead (§6).

**Distribution uses a different profile.** Every release job runs
`cargo build --profile release-dist --locked -p xai-grok-pager-bin`, producing `target/release-dist/deepseek-build`. `release-dist` inherits `release` and adds thin LTO, `codegen-units = 1`, `strip = "debuginfo"`, `debug = false`, and `split-debuginfo = "off"`. Do not ship a plain `--release` binary.

## 3. Test gate

`scripts/test-gate.sh` is the full gate. It first runs `scripts/disk-guard.sh 150` and refuses to start below 150 GB free (the
script's own default is 60 GB), then re-execs itself under `flock -w 900 /tmp/team-build.lock`, so two runs cannot overlap. Three
steps:

1. `cargo test --workspace --no-fail-fast -- --test-threads=4`
2. `cargo test -p xai-grok-shell --lib --features config-docs -j 1` — `--workspace` never enables `config-docs`, so this is a named step.
3. `scripts/key-leak-check.sh --scan docs/evidence`

Both cargo steps run with `DEEPSEEK_API_KEY`, `DEEPSEEK_BUILD_API_KEY`, and `XAI_API_KEY` unset, so an exported key cannot change
results or reach a log.

The verdict is arithmetic, never the tail of the log: `results` (one top-level `test result:` line per cargo target block) must
equal `binaries` (unique test executables from the build graph) plus `doctests`. In full mode it must also equal
`scripts/test-gate.expected`, which records `results=310`, `binaries=221`, `doctests=89` (refreshed 2026-11-26 at `e79aef5b`, clean
tree). Adding or removing a test binary changes the count and fails the gate until the file is refreshed.

- `--package <crate>` (repeatable) runs a subset, prints `GATE MODE: SUBSET`, and skips the expected check.
- `--refresh-expected` rewrites the file only on a fully green run with the derived equality intact, stamping commit, tree state, and UTC date. A red run never writes it; never hand-edit it.
- `--log-dir <dir>` keeps both cargo logs. A green run otherwise deletes its temp logs; a red run keeps them and prints the path.

Day to day use `cargo test -p <crate>`; save the full gate for the end.

## 4. Egress and branding gates

`scripts/check-egress.sh` greps `crates`, `prod`, and `third_party` (excluding `LICENSE`, `NOTICE`, `THIRD-PARTY*`, and `*.md`) for
forbidden destinations — old xAI hosts, Sentry, Mixpanel, `code.grok.com`, `gs://`, `s3://`, storage paths. The only allowed
destinations are the configured model provider, user-configured MCP servers, and loopback. `--strict` also makes the SOFT report
fatal: `x.ai/`, `Grok Build`, `SpaceXAI`, `SuperGrok`, `grok-<digit>`. The tree currently passes (`HARD egress gate: OK`, exit 0,
checked 2026-11-27).

A single line may opt out with `egress-allow: <reason of at least 8 characters>`. It is for test fixtures that must name a forbidden
host to prove it is rejected — never production code, and the reason is visible in review. Adding an outbound destination is a
stop-and-ask change.

`cargo test -p xai-grok-pager-bin --test branding_surface` runs the real binary's `--help`, `agent --help`, `--version`,
`completions bash`, and a keyless headless run, and fails on `grok`, `xai`, or `x.ai`. `scripts/key-leak-check.sh` runs the binary
with a fake key in an isolated `HOME` and scans every written file plus captured stdout/stderr; `--scan <path>...` is the additive
mode for committed evidence (file names only).

## 5. Release pipeline

`.github/workflows/release.yml` triggers on a `v*` tag; `workflow_dispatch` runs the same builds but creates no release (release
creation is tag-only). Six jobs:

- `version` (`ubuntu-22.04`) — derives the version once and rejects non-semver tags.
- `build-linux` (`ubuntu-22.04`, `ubuntu-22.04-arm`) — gnu x86_64/aarch64, 90-minute timeout.
- `build-macos` (`macos-15`, `macos-15-intel`) — both darwin targets, 240-minute timeout.
- `build-windows` (`windows-latest`) — msvc `.zip`, 90-minute timeout.
- `publish` (`ubuntu-22.04`) — needs all three build jobs; creates the GitHub release.
- `container` (`ubuntu-22.04`) — needs `version` + `build-linux` only; pushes to GHCR.

Each leg builds natively (no cross-compilation, no QEMU) with `--profile release-dist --locked`, then smoke-checks that `--version`
reports the derived version and a commit. `publish` verifies all four tarballs and the Windows zip arrived, that
`install.sh`/`install.ps1` exist, and writes `SHA256SUMS`.

**Tag-list guard.** After `docker/metadata-action` resolves names and before the first push, `scripts/guard-image-tags.sh` asserts
the list. Stable must publish exactly `{version, v<version>, <major>.<minor>, <major>, latest}`; a pre-release must publish
`version` and `v<version>`, may include the semver aliases, and must **not** include `latest`. The guard runs before the push
because the build steps publish the whole list, including `:latest`; a guard afterwards is too late. `:latest` is the stable
pointer, so a pre-release that moved it would make an rc the repo's latest image.

**Pre-release classification.** `version` requires `vX.Y.Z` or `vX.Y.Z-<pre>` (build metadata is rejected) and sets
`is_prerelease=true` when the version contains a hyphen. That output feeds both `action-gh-release`'s `prerelease:` and the
container job, which moves `:latest` only when `is_prerelease` is `false`.

**macOS signing.** The Intel linker emits an unsigned binary, so `build-macos` runs `codesign --force --sign -` on both arches after
the build and before packaging, verifies the signature, and verifies it again on the binary extracted from the tarball. The workflow
uses an ad-hoc signature — not a Developer ID — and does not notarize; an unsigned artifact would fail the check for a non-defect.

Static checks: `sh scripts/check-release-pipeline.sh` (actionlint, shellcheck, workflow structure, tag guard, tag shape, installer
on loopback).

## 6. Versioning

`[workspace.package] version = "0.1.0"` in the root `Cargo.toml`. Four crates inherit it with `version.workspace = true`:
`xai-grok-pager-bin`, `xai-grok-pager`, `xai-grok-shell`, and `xai-grok-version`.

`xai-grok-version` resolves the version as `option_env!("GROK_VERSION")`, falling back to `env!("CARGO_PKG_VERSION")`;
`IS_DEV_BUILD` is true when `GROK_VERSION` is unset. `xai-grok-pager-bin`'s `build.rs` stamps `<version> (<12-hex commit>)`, so a
source build prints `0.1.0 (<commit>)` and a tag build prints the tag. Do not edit the workspace version to match a tag: the
pipeline injects `GROK_VERSION`, and the tag is the version source.

`v0.1.0` is **not published**. The only published artifact is the pre-release `v0.1.0-proof1`; `v0.0.1-rc2` through `v0.0.1-rc5` are
also published prereleases. The Windows zip is authored but has no published artifact and has not run on Windows.

## 7. Evidence convention

Claims that a gate or live run happened are recorded under `docs/evidence/` — see [`evidence/README.md`](evidence/README.md). Each
item is re-runnable:

```text
docs/evidence/NN-<group-slug>/<item-slug>/
    <item-slug>.cast     asciinema v2 recording
    report.json          fixed-schema summary (the artifact to read)
    notes.md             optional: what the cast shows and how to re-run it
    <step-slug>.log      per-step output, normalized
```

`report.json` has exactly these keys, in order: `item`, `requirements`, `date`, `host`, `command`, `steps`, `artifacts`, `result`.
`scripts/evidence.py` drives the lifecycle: `begin`, `step` (runs a command, records its real exit status, tees a log), `artifact`
(sha256 relative to the repo root), `finish` (refuses `--result ok` if a step failed, no steps ran, or an artifact is missing),
`cast` (converts `script -T` output to asciinema v2), and `normalize`.

Normalization is mandatory: the repository root, the user's home, and the temp directory become `<REPO>`, `<HOME>`, and `<TMP>` in
every log and in `report.json`; environment dumps are never captured. A finished item is frozen — a correction is a new item
directory, not a re-run of `finish`. The gate's third step scans the committed tree.

## 8. Where the plan lives

[`planning.md`](planning.md) maps the plan, its decision record, and the committed proof. The root [`../PLAN.md`](../PLAN.md) holds
the audit and working plan; [`../DECISIONS.md`](../DECISIONS.md) holds locked decisions and wins on conflict;
[`../AGENTS.md`](../AGENTS.md) holds the snapshot (§0) and change rules (§8); [`../KNOWN-ISSUES.md`](../KNOWN-ISSUES.md) lists known
gaps.

Do not restate them here; update the owning file with the change.
