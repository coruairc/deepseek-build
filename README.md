<!-- generated-by: gsd-doc-writer -->
# deepseek-build

An independent, DeepSeek-first terminal coding agent — a full-screen TUI plus
headless mode for code edits, shell commands, and multi-step tasks.

Independent project, not affiliated with DeepSeek. Apache-2.0, forked from [xai-org/grok-build](https://github.com/xai-org/grok-build).

## Install

`v0.1.0-proof1` is the pinned pre-release. GitHub's `latest` path resolves stable
releases only, so it 404s: pin `VERSION`.

**Linux**

```sh
curl -fsSL https://github.com/coruairc/deepseek-build/releases/download/v0.1.0-proof1/install.sh \
  | VERSION=v0.1.0-proof1 sh
```

Verifies the tarball's SHA-256 before extracting, installs to `${INSTALL_DIR:-$HOME/.local/bin}` (no sudo), and creates a `dsb` shortcut.

**macOS** (binaries are ad-hoc signed and not notarized)

```sh
brew tap coruairc/deepseek-build
brew trust coruairc/deepseek-build
brew install deepseek-build
```

**Windows** — packaging is authored in the release workflow, but no Windows artifact is published or executed yet.

Details: [docs/install.md](docs/install.md).

## Uninstall

- **Linux:** `curl -fsSL https://github.com/coruairc/deepseek-build/releases/download/v0.1.0-proof1/install.sh | sh -s -- --uninstall` (add the same `INSTALL_DIR` if you used one).
- **macOS:** `brew uninstall deepseek-build`, then optionally `brew untap coruairc/deepseek-build`.
- **Windows:** not applicable yet.

This does not delete `~/.deepseek-build` (config and saved key). Remove it with `rm -rf ~/.deepseek-build`. Details: [docs/install.md](docs/install.md).

## Quick start

```sh
export DEEPSEEK_API_KEY=sk-...   # or DEEPSEEK_BUILD_API_KEY
cd your-repo && dsb
```

No key? The TUI prompts once (masked) and saves it to `~/.deepseek-build/credentials.json`
(0600); the environment wins. `/model` switches models; `/effort low` is the measured cost lever.

## What it does

- Reads and edits files, runs shell commands, searches the repo, and runs background tasks.
- Asks before edits and non-read-only shell commands by default; supports permission
  modes, allow/deny rules, and plan mode.
- Persists sessions (`/resume`, `-c`, or `-r <id>`), connects user-configured MCP servers
  over stdio or HTTP, and loads `AGENTS.md` rules, skills, hooks, and custom agents.
- Sandbox is opt-in (`--sandbox workspace`, `read-only`, or `strict`), kernel-enforced on
  Linux (Landlock) and macOS (Seatbelt); off by default. No telemetry or auto-update; the
  only egress is the model provider, configured MCP servers, and opt-in `web_fetch` hosts.
- `/usage` reports token usage, the cached input share, and a local cost estimate; the
  optional status line can show `cache` and `cost`.

## Container

A public multi-arch image (linux/amd64, linux/arm64) is published, but it is
**headless only** — one `-p` prompt per invocation. It ships no sandbox (no
`bubblewrap`), so do not pass `--sandbox`; the container boundary is the
isolation. Mount the repo at `/work` and state at `/home/app/.deepseek-build`:

```sh
docker run --rm --user "$(id -u):$(id -g)" -e DEEPSEEK_API_KEY \
  -e DEEPSEEK_BUILD_HOME=/home/app/.deepseek-build \
  -v "$PWD":/work -w /work -v "$HOME/.deepseek-build":/home/app/.deepseek-build \
  ghcr.io/coruairc/deepseek-build@sha256:8f5c484ed067143aab93aa90b1441101fd643a37924f1868eeaaa103d1f199be \
  -p "..." --output-format json --permission-mode auto
```

Details: [docs/container.md](docs/container.md).

## Build from source

Rust 1.94.0 is pinned by `rust-toolchain.toml`; see [docs/development.md](docs/development.md) for the toolchain and build commands.

## Documentation

- [`docs/`](docs/) — index of [install](docs/install.md), [configuration](docs/configuration.md), [container](docs/container.md), and [development](docs/development.md) notes.
- [User guide](crates/codegen/xai-grok-pager/docs/user-guide/README.md) — commands, configuration, sandbox, MCP, and headless mode.
- [`KNOWN-ISSUES.md`](KNOWN-ISSUES.md) — known gaps and unexercised surfaces.
- [`TESTING.md`](TESTING.md) — test plan and safe local runs.
- [`docs/planning.md`](docs/planning.md) — milestone plan, roadmap, requirements, and evidence.
- [`docs/evidence/`](docs/evidence/README.md) — committed proof of distribution and runtime checks.

## License

Apache-2.0. See [`LICENSE`](LICENSE), [`NOTICE`](NOTICE), and [`THIRD-PARTY-NOTICES`](THIRD-PARTY-NOTICES).
Upstream is [xai-org/grok-build](https://github.com/xai-org/grok-build). "DeepSeek" is a
trademark of its respective owner; this project merely uses DeepSeek's public API.
