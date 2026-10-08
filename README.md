# deepseek-build

A personal, DeepSeek-first terminal coding agent, forked from
[xai-org/grok-build](https://github.com/xai-org/grok-build) (Apache-2.0).

`deepseek-build` is a full-screen TUI that reads and edits your codebase, runs
shell commands, and manages long-running tasks, driven by DeepSeek's
OpenAI-compatible Chat Completions API. It is a work in progress (see
[Status](#status) and [`KNOWN-ISSUES.md`](KNOWN-ISSUES.md)).

> A release pipeline is in place that builds binaries for linux-x86_64,
> linux-aarch64, darwin-arm64, and darwin-x86_64, with a checksum file and an
> installer script attached to each GitHub release. **No release has been
> published yet** — no `v*` tag has been pushed, so
> [Releases](https://github.com/coruairc/deepseek-build/releases) is currently
> empty. Until the first tag, build from source (see
> [Building from source](#building-from-source)).

## Status

| Area | State |
|------|-------|
| Phase 0 - audit & plan | Done (`PLAN.md`, `DECISIONS.md`) |
| Phase 1 - cleanup | Done: upload/exfil, telemetry/Sentry/OTLP, xAI voice/Imagine/web_search, auto-update, announcements, cloud-config/remote control, session share/relay, and xAI model/endpoint strings are removed or neutralized. **HARD and SOFT egress gates are green**; rebrand (binary, config dir, ACP namespace, strings) done. |
| Phase 2 - DeepSeek adapter | Core done: Chat Completions-only backend (`Responses`/`Messages` deleted), `thinking` control, `reasoning_content` round-trip + sanitizer, typographic-quote repair, cache helpers, DeepSeek catalog (default `deepseek-v4-pro`), wiremock suite. Live smoke run 8/8 PASS (2026-10-07, author's machine). |
| Phase 3 - TUI | Implemented: reasoning fold (`e`/`E`/`Ctrl+E`), collapsible thinking, status line (model/context/effort/tokens/cache/cost), `/model` `/effort` `/think` `/theme`, Monokai default theme + whale logo. |
| Phase 4 - harden existing features | Partial: runtime read-before-write guard and dangerous-command warnings are in; plan mode, permissions, resume, subagents, MCP, `AGENTS.md`, and checkpoints exist and need an end-to-end pass. A full `cargo test --workspace` run is **not green yet** (see [`KNOWN-ISSUES.md`](KNOWN-ISSUES.md)). |
| Phase 5 - testable product | In progress: release build, `TESTING.md`, `KNOWN-ISSUES.md`, `SECURITY.md`, and the sandbox/egress scripts are in place. Release pipeline (CI tarballs + installer) and container definitions are added but **have never run**: no tag pushed, no release, no image. |

See [`AGENTS.md`](AGENTS.md) for a precise resume-from-here handoff.

## Install

> **No release has been published yet** (checked 2026-10-09): the `curl | sh`
> and release-asset steps below describe the installer and the intended flow,
> but they cannot run until the first `v*` tag is pushed. Build from source in
> the meantime (see [Building from source](#building-from-source)); the installer
> itself has been exercised against a local fake release
> (`sh scripts/test-install.sh`, 25/25 assertions).

Once a release exists:

```sh
curl -fsSL https://github.com/coruairc/deepseek-build/releases/latest/download/install.sh | sh
```

Piping curl to `sh` executes whatever that URL serves, at download time, with no
chance to review it first. The slower, recommended alternative: download,
inspect, then run.

```sh
curl -fsSLO https://github.com/coruairc/deepseek-build/releases/latest/download/install.sh
less install.sh        # inspect it
sh install.sh
```

The installer is plain POSIX `sh`. It:

- detects OS and arch (supported targets below) and picks the matching tarball;
- downloads the release tarball and `SHA256SUMS` over HTTPS from GitHub
  Releases;
- verifies the checksum **before** extracting anything;
- installs the binary to `${INSTALL_DIR:-$HOME/.local/bin}` (no sudo), prints a
  PATH hint if needed, and supports `--uninstall`; rerunning it updates in
  place;
- makes no network requests other than those two GitHub downloads - no
  telemetry.

Pin a version with `VERSION`:

```sh
VERSION=v0.1.0 sh install.sh
curl -fsSL .../install.sh | VERSION=v0.1.0 sh
```

(This works because `VERSION=v0.1.0` before a command is a POSIX env assignment,
not shell syntax embedded in the script.)

Uninstall:

```sh
curl -fsSL .../install.sh | sh -s -- --uninstall
# or, after downloading:
sh install.sh --uninstall
```

Manual install, if you prefer to drive every step:

1. Download the tarball for your platform and `SHA256SUMS` from the
   [releases page](https://github.com/coruairc/deepseek-build/releases).
2. Verify: `SHA256SUMS` lists all four tarballs, so filter to yours:

   ```sh
   grep <your-tarball> SHA256SUMS | sha256sum -c -
   # macOS: grep <your-tarball> SHA256SUMS | shasum -a 256 -c -
   ```

3. Extract and move `deepseek-build` somewhere on `PATH`.

### Supported platforms

| Target | Typical OS notes |
|--------|------------------|
| `x86_64-unknown-linux-gnu` | Linux x86_64, glibc >= 2.35 (Ubuntu 22.04+) |
| `aarch64-unknown-linux-gnu` | Linux arm64, glibc >= 2.35 (Ubuntu 22.04+) |
| `aarch64-apple-darwin` | macOS 14 (Sonoma)+ on Apple silicon |
| `x86_64-apple-darwin` | macOS 13 (Ventura)+ on Intel |

macOS builds come from macOS 15 runners; the listed minimums are conservative.
The macOS binaries are **not codesigned and not notarized**, so first launch
may need a right-click **Open**, or
`xattr -d com.apple.quarantine <path-to-binary>`.

> Independent project, not affiliated with DeepSeek. "DeepSeek" is a trademark
> of its respective owner; this project merely uses their public API.
>
> Forked from [xai-org/grok-build](https://github.com/xai-org/grok-build)
> (Apache-2.0); see [`LICENSE`](LICENSE) and [`NOTICE`](NOTICE) for terms and
> attribution.

## Container

Official releases also publish a multi-arch (linux/amd64, linux/arm64) image to
GitHub Container Registry. It is **not affiliated with DeepSeek**.

> **Not yet built or published.** The container job in the release workflow runs
> only on a tag push, and no tag has been pushed, so there is no GHCR image to
> pull and the `docker pull` commands below cannot work yet (checked 2026-10-09).
> The production image is assembled from a staged context containing the release
> tarball's binary as `deepseek-build-amd64`/`deepseek-build-arm64` plus the
> license files; see `docker/Dockerfile` and the container step in
> `.github/workflows/release.yml`. To smoke-test locally with a binary you built,
> use `docker/Dockerfile.localtest` (also not yet exercised — see
> [`KNOWN-ISSUES.md`](KNOWN-ISSUES.md)).

Once a release exists:

```sh
# Pull (replace <owner> with the GitHub account that published it)
docker pull ghcr.io/<owner>/deepseek-build:latest

# Interactive session in a repo, key from your environment
docker run -it --rm \
  -e DEEPSEEK_API_KEY \
  -v "$PWD":/work \
  ghcr.io/<owner>/deepseek-build:latest
```

Host-owned files: the image runs as uid 1000; pass your own ids so files written
into the mount stay yours.

```sh
docker run -it --rm --user "$(id -u):$(id -g)" \
  -e DEEPSEEK_API_KEY -v "$PWD":/work \
  ghcr.io/<owner>/deepseek-build:latest
```

`--version` / headless:

```sh
docker run --rm ghcr.io/<owner>/deepseek-build:latest --version
docker run --rm -e DEEPSEEK_API_KEY ghcr.io/<owner>/deepseek-build:latest -p "say hi"
```

The image is built from the release tarballs (`docker/Dockerfile`, per-arch
binary via `TARGETARCH`), contains no API key, and ships the license files in
`/usr/share/doc/deepseek-build/`. Package visibility depends on the GHCR package
settings (packages default to private for personal accounts).

## Security & network

There is no telemetry, analytics, or auto-update. The only network destinations
are:

1. the configured model provider (default `https://api.deepseek.com`),
2. user-configured MCP servers, and
3. when you enable the `web_fetch` tool (`[features] web_fetch = true`), the
   hosts that tool fetches — off by default, limited to a built-in documentation
   allowlist (`[toolset.web_fetch] allowed_domains` overrides it), and gated per
   non-allowlisted host by a permission prompt.

See [`SECURITY.md`](SECURITY.md#network--privacy) for the full statement and how
to verify the boundary. Run the static guard at any time:

```sh
scripts/check-egress.sh          # HARD gate must be OK; prints a SOFT branding report
scripts/check-egress.sh --strict # also fails on remaining branding/ACP-namespace strings
```

To watch runtime egress, run the binary through
`scripts/sandbox-run.sh` (see [`TESTING.md`](TESTING.md)).

## Requirements

- [`rustup`](https://rustup.rs) with the pinned toolchain (Rust 1.94.0, from
  `rust-toolchain.toml`). Do not bump it.
- `protoc` on `PATH` (e.g. `/usr/bin/protoc`). `dotslash` is not needed.

## Building from source

Until the first release is published (see [Install](#install)), this is the only
way to get the binary:

```sh
# One-time: install the pinned toolchain
rustup toolchain install 1.94.0 --component rustfmt clippy

# Validate a crate quickly
cargo check -p xai-grok-pager-bin

# Build the binary (artifact: target/release/deepseek-build)
cargo build --release -p xai-grok-pager-bin

# Run
target/release/deepseek-build
```

> Internal crate/package names still carry the upstream `xai-grok-*` prefix
> (a deliberate scope decision, see `DECISIONS.md` D2). The user-facing binary,
> config dir (`~/.deepseek-build`), ACP namespace, and strings are
> `deepseek-build`.

## Configuration

Set your DeepSeek API key in the environment:

```sh
export DEEPSEEK_API_KEY=sk-...          # preferred
# or: export DEEPSEEK_BUILD_API_KEY=sk-...
```

Models: `deepseek-v4-pro` (default) and `deepseek-flash`, plus a hidden
`deepseek-v4-flash` legacy alias; each offers `none`/`low`/`high`/`max`
reasoning effort. A custom OpenAI-compatible `base_url` is supported via model
config.

## Testing the binary safely

`scripts/sandbox-run.sh` runs the binary with only `api.deepseek.com` reachable.
A manual test plan lives in [`TESTING.md`](TESTING.md) (a 20–30 minute core pass
plus live-script and release/container checks); known gaps and the surfaces that
have never been exercised are in [`KNOWN-ISSUES.md`](KNOWN-ISSUES.md).

## Repository layout

Same as upstream (`crates/codegen/...`, `crates/common/...`, `prod/mc/...`).
Key crates:

| Crate | Purpose |
|-------|---------|
| `xai-grok-pager-bin` | binary `deepseek-build` |
| `xai-grok-pager` | TUI |
| `xai-grok-shell` | agent runtime, sessions, turns |
| `xai-grok-sampler` | HTTP/streaming client (`reqwest`) |
| `xai-grok-sampling-types` | wire + conversation types (DeepSeek adapter lives here) |
| `xai-grok-tools` | tool implementations |
| `xai-grok-mcp` | MCP client |

## License & attribution

First-party code is Apache-2.0. See [`LICENSE`](LICENSE),
[`NOTICE`](NOTICE), [`THIRD-PARTY-NOTICES`](THIRD-PARTY-NOTICES), and
[`third_party/NOTICE`](third_party/NOTICE). Upstream is
[xai-org/grok-build](https://github.com/xai-org/grok-build); upstream files are
not relicensed.
