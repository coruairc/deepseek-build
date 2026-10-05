# KNOWN ISSUES — deepseek-build

Honest list of what is incomplete or broken at this revision. Nothing here is
hidden behind a "temporary" flag; where code remains it is called out.

## Build & tests

- **`cargo test --workspace` does not compile.** Many `#[cfg(test)]` modules and
  `tests/**` files still reference symbols deleted during the Phase 1 cleanup
  (e.g. `WebSearchInput`, `MediaGenOutput`, `ImageGenConfig`,
  `emit_announcements`, `heap_profile`, external-OTEL types). `cargo check`
  (non-test) and `cargo build --release` are green. Fixing the test suite is a
  dedicated pass.
- **`cargo clippy` was only run on the crates touched per slice**, not across the
  whole workspace, after the large deletions.

## Network / egress

- **HARD egress gate: green.** `scripts/check-egress.sh` reports `HARD egress
  gate: OK`, and `--strict` also exits 0 (SOFT branding is zero after the
  rebrand).
- **Runtime egress (observed):** a single-turn prompt contacts only
  `api.deepseek.com:443` plus one local `AF_UNIX` socket. `--version`/`--help`
  make no outbound connections. This was measured with an `LD_PRELOAD`
  connect-logger because `strace` and network namespaces were unavailable in the
  dev environment; reproduce with `strace -f -e trace=connect` where available.
- **Live smoke test not run.** No real `DEEPSEEK_API_KEY` was provided, so the
  end-to-end chat / streamed chat / 3-turn thinking tool-call chain / usage-cache
  assertions against the live API are unverified. The request path, model list,
  and error handling were verified as far as a 401 response (dummy key) allows.

## Phase 1 cleanup — remaining stubs / undeleted code

These contain **no network egress** but were not fully deleted. They are the
remaining work:

- `xai-grok-shell/src/upload/**` — restored inert, routed to
  `file_utils_compat` (no storage HTTP).
- `xai-grok-shell/src/session/repo_changes/mod.rs` — pure serde types only.
- `xai-grok-shell/src/session/feedback_manager.rs` — inert; local-only.
- `xai-grok-shell/src/cloud_config/**` — the former `xai-grok-cloud-config`
  crate, vendored inert into the shell.
- `xai-grok-telemetry/src/{external.rs,otel_layer.rs,trace_context.rs}` — no-op
  OTLP/trace stubs.
- Pager voice UI (`xai_grok_voice.rs` stub) and `views/announcements.rs` (no-op).
- `xai-computer-hub-{core,sdk,mcp-adapter}` crates and their consumers
  (workspace hub-auth, leader observability, workspace-client/daemon) — present,
  de-egressed but **not deleted**.
- `xai-grok-shell/src/remote/**` relay/gateway clients and `agent/relay.rs` —
  present with neutralized hosts; the `agent headless` / `serve` relay subcommands
  still exist and should be deleted.
- `prod/mc/model-api-types` storage wire types remain (used by the inert upload
  shim).

## Phase 2 — adapter

- **Backend removal (D5) not done:** the `Responses` and `Messages` backends and
  their sampler paths still exist. DeepSeek uses `ChatCompletions` (the default),
  so this is surface-area reduction, not a functional gap.
- **Wiremock suite not added.** Behavior is covered by unit tests:
  `cargo test -p xai-grok-sampling-types --lib` (streaming, tool-call assembly,
  reasoning round-trip + sanitizer, typographic-quote repair) and
  `cargo test -p xai-grok-sampler --lib stream::chat_completions`. A dedicated
  429/5xx-backoff mock test is still missing.
- **Non-stream reasoning capture** is not separately hardened.
- **Auto model routing** (Flash/no-reasoning vs Pro/high) is not implemented.

## Phase 3 — TUI

- Not implemented: collapsible reasoning block, enriched status bar (cache hit
  rate / running cost / context usage), model + reasoning-effort hotkeys,
  per-turn/session cost breakdown. The agent works without them.

## Phase 4 — hardening

- Not verified end-to-end: read-before-write enforcement is still config-time only
  (no runtime gate); plan/permission/compaction/resume/subagents/MCP/AGENTS.md/
  checkpoints were not exercised against the real API.

## Rebrand / config

- Binary, config dir (`~/.deepseek-build`), ACP namespace
  (`deepseek-build/*`), and user-facing strings are rebranded.
- **Most `GROK_*` environment variables remain** (624 distinct names). Only the
  home var (`DEEPSEEK_BUILD_HOME`, with `GROK_HOME` fallback) and the production
  endpoint prefix were renamed. This does not affect the egress gates.
- **No migration from `~/.grok`** to `~/.deepseek-build`; point `GROK_HOME` at the
  old directory if you want the existing sessions/config.

## Licensing

- `LICENSE`, `NOTICE`, `THIRD-PARTY-NOTICES`, and `third_party/NOTICE` are
  preserved. Upstream files are not relicensed. Legal sign-off (DECISIONS O1) is
  still recommended before any public distribution.
