# deepseek-build

A personal, DeepSeek-first terminal coding agent, forked from
[xai-org/grok-build](https://github.com/xai-org/grok-build) (Apache-2.0).

`deepseek-build` is a full-screen TUI that reads and edits your codebase, runs
shell commands, and manages long-running tasks, driven by DeepSeek's
OpenAI-compatible Chat Completions API. It is a work in progress (see
[Status](#status) and [`KNOWN-ISSUES.md`](KNOWN-ISSUES.md)).

> This is a personal-use tool. There is no release pipeline, packaging, or
> support. It currently builds from source on Linux/macOS.

## Status

| Area | State |
|------|-------|
| Phase 0 — audit & plan | Done (`PLAN.md`, `DECISIONS.md`) |
| Phase 1 — cleanup | Done: upload/exfil, telemetry/Sentry/OTLP, xAI voice/Imagine/web_search, auto-update, announcements, cloud-config/remote control, session share/relay, and xAI model/endpoint strings are removed or neutralized. **HARD and SOFT egress gates are green**; rebrand (binary, config dir, ACP namespace, strings) done. |
| Phase 2 — DeepSeek adapter | Core done: Chat Completions-only backend (`Responses`/`Messages` deleted), `thinking` control, `reasoning_content` round-trip + sanitizer, typographic-quote repair, cache helpers, DeepSeek catalog (default `deepseek-v4-pro`), wiremock suite. Live smoke test still needs a real key. |
| Phase 3 — TUI | Implemented: reasoning fold (`e`/`E`/`Ctrl+E`), collapsible thinking, status line (model/context/effort/tokens/cache/cost), `/model` `/effort` `/think` `/theme`, Monokai default theme + whale logo. |
| Phase 4 — harden existing features | Partial: runtime read-before-write guard and dangerous-command warnings are in; plan mode, permissions, resume, subagents, MCP, `AGENTS.md`, and checkpoints exist and need end-to-end exercising. |
| Phase 5 — testable product | In progress: release build, `TESTING.md`, `KNOWN-ISSUES.md`, `SECURITY.md`, and the sandbox/egress scripts are in place. |

See [`AGENTS.md`](AGENTS.md) for a precise resume-from-here handoff.

## Security & network

There is no telemetry, analytics, or auto-update. The only network destinations
are:

1. the configured model provider (default `https://api.deepseek.com`), and
2. user-configured MCP servers.

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

## Build & run

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
A 20–30 minute manual test plan lives in [`TESTING.md`](TESTING.md); known gaps
are in [`KNOWN-ISSUES.md`](KNOWN-ISSUES.md).

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
