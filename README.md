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
| Phase 1 — cleanup | Partial: upload/exfil, telemetry/Sentry/OTLP, xAI voice/Imagine/web_search, auto-update, announcements, cloud-config/remote control, and xAI model/endpoint strings are removed or neutralized. **HARD egress gate is green.** Rebrand (name/config/env/ACP namespace) is **not** done. |
| Phase 2 — DeepSeek adapter | Core done: `thinking` control, `reasoning_content` round-trip + sanitizer, `tool_choice` downgrade in thinking mode, typographic-quote repair, cache helpers, DeepSeek catalog (default `deepseek-v4-pro`). Wiremock suite + live smoke test **not** done. |
| Phase 3 — TUI | Not started |
| Phase 4 — harden existing features | Not started |
| Phase 5 — testable product | Not started (release build was aborted; see handoff) |

See [`AGENTS.md`](AGENTS.md) for a precise resume-from-here handoff.

## Security & network

The only network destinations are:

1. the configured model provider (default `https://api.deepseek.com`), and
2. user-configured MCP servers.

Run the guard at any time:

```sh
scripts/check-egress.sh          # HARD gate must be OK; prints a SOFT branding report
scripts/check-egress.sh --strict # also fails on remaining branding/ACP-namespace strings (post-rebrand)
```

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

# Build the binary (artifact: target/release/xai-grok-pager)
cargo build --release -p xai-grok-pager-bin

# Run
target/release/xai-grok-pager
```

> Internal crate/package names still carry the upstream `xai-grok-*` prefix
> (a deliberate scope decision, see `DECISIONS.md` D2). The user-facing binary
> and strings will be renamed to `deepseek-build` in the rebrand slice.

## Configuration

Set your DeepSeek API key in the environment:

```sh
export DEEPSEEK_API_KEY=sk-...          # preferred
# or: export DEEPSEEK_BUILD_API_KEY=sk-...
```

Models: `deepseek-v4-pro` (default) and `deepseek-flash` (a hidden
`deepseek-v4-flash` legacy alias is also present). A custom OpenAI-compatible
`base_url` is supported via model config.

## Testing the binary safely

`scripts/sandbox-run.sh` (see the handoff notes) is intended to run the binary
with only `api.deepseek.com` reachable. Manual test steps live in
`TESTING.md` (to be written in the Phase 5 slice).

## Repository layout

Same as upstream (`crates/codegen/...`, `crates/common/...`, `prod/mc/...`).
Key crates:

| Crate | Purpose |
|-------|---------|
| `xai-grok-pager-bin` | binary `xai-grok-pager` |
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
