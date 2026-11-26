# deepseek-build

An independent, DeepSeek-first terminal coding agent: a full-screen TUI and a
headless mode for reading and editing code, running shell commands, and
completing multi-step tasks over DeepSeek's OpenAI-compatible Chat Completions API.

This is an independent project, not affiliated with DeepSeek. Apache-2.0,
forked from [xai-org/grok-build](https://github.com/xai-org/grok-build).

## Install

The only published artifacts are the pre-release **`v0.1.0-proof1`**. `v0.1.0`
is not published. GitHub's `latest` path resolves stable releases only, so
`releases/latest/download/install.sh` returns **404**: pin `VERSION`.

### Linux

```sh
curl -fsSL https://github.com/coruairc/deepseek-build/releases/download/v0.1.0-proof1/install.sh \
  | VERSION=v0.1.0-proof1 sh
```

The installer verifies the tarball's SHA-256 against `SHA256SUMS` before
extracting, installs `deepseek-build` to `${INSTALL_DIR:-$HOME/.local/bin}`
(no sudo), and creates a `dsb` shortcut. `sh install.sh --uninstall` removes
both. x86_64 and aarch64 tarballs are published.

### macOS (Apple silicon and Intel)

```sh
brew tap coruairc/deepseek-build
brew trust coruairc/deepseek-build   # Homebrew's trust gate for third-party taps
brew install deepseek-build
```

The formula installs the prebuilt tarball for the host architecture and a `dsb`
shortcut; it resolves to `0.1.0-proof1` and never builds from source. The tap is
macOS-only; Linux uses `install.sh` above.

Tarball fallback, if you prefer not to use Homebrew:

```sh
curl -fsSL https://github.com/coruairc/deepseek-build/releases/download/v0.1.0-proof1/install.sh \
  | VERSION=v0.1.0-proof1 sh
```

The macOS binaries are ad-hoc signed and not notarized; they carry no Developer
ID. Homebrew- and curl-fetched tarballs are not quarantined. A browser download
may be; if macOS blocks the first launch, right-click **Open**, or run:

```sh
xattr -d com.apple.quarantine /path/to/deepseek-build
```

### Windows

The release workflow authors a `.zip` and `install.ps1` for x86_64, but this
path is **authored, not yet published or executed**: no Windows artifact exists
on a release and the installer has not run on Windows. Only `--sandbox off` is
accepted there; no in-process sandbox is available on Windows.

## Quick start

```sh
export DEEPSEEK_API_KEY=sk-...   # or DEEPSEEK_BUILD_API_KEY
cd your-repo
dsb
```

Without either variable, the TUI prompts once with a masked field and saves the
key to `~/.deepseek-build/credentials.json` (mode `0600`); the environment
wins. Headless runs never prompt.

`deepseek-flash` is the default model. Switch to `deepseek-v4-pro` with
`/model deepseek-v4-pro` or `-m deepseek-v4-pro`; `/effort low` is the measured
cost lever. Headless: `dsb -p "..." --output-format json` adds usage and
`estimated_cost_usd`.

## What it does

- Reads and edits files, runs shell commands, searches the repo, and runs
  background tasks.
- Asks before edits and non-read-only shell commands by default; supports
  permission modes, allow/deny rules, and a plan mode.
- Persists sessions; `/resume`, `-c`, or `-r <id>` reopens one.
- Connects user-configured MCP servers over stdio or HTTP.
- Loads project rules from `AGENTS.md`, plus skills, hooks, and custom agents;
  switch agents in-session with `/agent` (history preserved — the TUI relaunches
  into the resumed session).
- Sandbox is opt-in (`--sandbox workspace`, `read-only`, or `strict`),
  kernel-enforced on Linux (Landlock) and macOS (Seatbelt); off by default.
- No telemetry or auto-update. The only egress is the model provider,
  configured MCP servers, and opt-in `web_fetch` hosts.
- `/usage` reports token usage, the cached input share, and a local cost
  estimate; the optional status line can show `cache` and `cost`.

## Configuration

| Path | Purpose |
|------|---------|
| `~/.deepseek-build/config.toml` | User settings: models, MCP servers, permissions, hooks. `DEEPSEEK_BUILD_HOME` relocates the directory. |
| `.deepseek/config.toml` | Project scope: MCP servers, plugins, permission rules. `.grok/` is a read-only compatibility fallback. |
| `~/.deepseek-build/credentials.json` | Saved API key, mode `0600`, written by the TUI prompt. `DEEPSEEK_API_KEY` / `DEEPSEEK_BUILD_API_KEY` win. |

## Container

```sh
docker pull ghcr.io/coruairc/deepseek-build@sha256:8f5c484ed067143aab93aa90b1441101fd643a37924f1868eeaaa103d1f199be
```

Public multi-arch image (linux/amd64, linux/arm64), headless only: one `-p`
prompt per invocation. The image ships no sandbox — it has no `bubblewrap`, so
do not pass `--sandbox`; the container boundary is the isolation. Mount the repo
at `/work` and state at `/home/app/.deepseek-build`:

```sh
docker run --rm --user "$(id -u):$(id -g)" -e DEEPSEEK_API_KEY \
  -e DEEPSEEK_BUILD_HOME=/home/app/.deepseek-build \
  -v "$PWD":/work -w /work \
  -v "$HOME/.deepseek-build":/home/app/.deepseek-build \
  ghcr.io/coruairc/deepseek-build@sha256:8f5c484ed067143aab93aa90b1441101fd643a37924f1868eeaaa103d1f199be \
  -p "..." --output-format json --permission-mode auto
```

## Build from source

Rust 1.94.0 is pinned by `rust-toolchain.toml`; `protoc`, `cc`, and `make` are
required. Build the release binary:

```sh
cargo build --release -p xai-grok-pager-bin   # target/release/deepseek-build
```

## Documentation

- [`KNOWN-ISSUES.md`](KNOWN-ISSUES.md) — known gaps and unexercised surfaces.
- [`TESTING.md`](TESTING.md) — test plan and safe local runs.
- [`crates/codegen/xai-grok-pager/docs/user-guide/`](crates/codegen/xai-grok-pager/docs/user-guide/README.md) — configuration, commands, sandbox, MCP, headless mode.
- [`docs/evidence/`](docs/evidence/README.md) — committed proof of distribution and runtime checks.

## License

Apache-2.0. See [`LICENSE`](LICENSE), [`NOTICE`](NOTICE), and
[`THIRD-PARTY-NOTICES`](THIRD-PARTY-NOTICES). Upstream is
[xai-org/grok-build](https://github.com/xai-org/grok-build); upstream files are
not relicensed. "DeepSeek" is a trademark of its respective owner; this project
merely uses DeepSeek's public API.
