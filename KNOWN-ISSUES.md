# KNOWN ISSUES — deepseek-build

Honest list of what is incomplete or broken at this revision. Nothing here is
hidden behind a "temporary" flag; where code remains it is called out. Claims are
against the tree on `dsb/integration` (post deletion-branches; fmt commit
`20f47c7d`).

## Build & tests

- **`cargo test --workspace` does not compile.** The only failing crate is
  **`xai-grok-workspace`** (all other crates' test targets compile). Its
  `#[cfg(test)]` modules still reference production code removed by the
  deletion branches: `crate::hub` / `crate::hub_server` / `crate::mcp`,
  `xai_computer_hub_sdk`, `build_session_routed_handlers`, `SharedAuthProvider`,
  the MCP session bridge types (`WorkspaceMcpBinding`, `McpServerOutcome`,
  `FakeHubRegistry`), and donation/observability methods. Affected files:
  `xai-grok-workspace/src/{handle_tests,host_kind_tests}.rs`,
  `sandbox/real_wiring_tests.rs` (now repaired via the local harness),
  `permission/hub_gate_tests.rs` (deleted), `permission/hub_permission.rs`
  (test module).
- **Non-test build is green:** `cargo check -p xai-grok-pager-bin` and
  `cargo build --release -p xai-grok-pager-bin` succeed; the binary runs
  (`target/release/deepseek-build`, ~147 MB, `deepseek-build 1.0.45`).
- **Tags:** `last-known-good-2026-10-06` (`b3847ff8`, last green test compile;
  still contains egress code — not safe to run) and `parked-test-repair-2026-10-06`
  (`c7d7f7e4`, current parked state).
- `scripts/synstrip/` (a `syn`-based item stripper) and `scripts/line_loop.py`
  (compiler-driven repair loop) are committed for the future test-repair pass.

## Network / egress

- **HARD and SOFT gates: green.** `scripts/check-egress.sh --strict` prints
  `HARD egress gate: OK` and exits 0 (verified during the Phase 5 docs pass).
- **Runtime egress (observed):** a single-turn prompt contacts only
  `api.deepseek.com:443` plus one local `AF_UNIX` socket. `--version`/`--help`
  make no outbound connections. This was measured with an `LD_PRELOAD`
  connect-logger because `strace` and network namespaces were unavailable in the
  dev environment; reproduce with `strace -f -e trace=connect` where available.
  The same `LD_PRELOAD` approach is what `scripts/sandbox-run.sh` ships.
- **Live smoke test skipped.** `DEEPSEEK_API_KEY` is not exported in the agent
  shell, so the end-to-end chat / streamed chat / 3-turn thinking tool-call chain /
  usage-cache assertions are unverified. The request path is verified as far as a
  401 (dummy key) allows.
- **Extended runtime egress not exercised.** With no key the model cannot drive
  tools / MCP / compaction / resume, so the extended session test could not run.
  Measured (LD_PRELOAD connect-logger; `strace` absent): `--version` = no connects;
  a headless turn = only `api.deepseek.com:443` + one local AF_UNIX socket.

## Phase 1 cleanup — stubs

All previously-listed stubs are now **deleted**: `xai-grok-shell/src/{upload,cloud_config,remote}`,
`session/repo_changes`, `xai-grok-telemetry/src/{external.rs,otel_layer.rs,trace_context.rs}`,
`xai-grok-pager/src/views/announcements.rs`, `xai-grok-feedback`, the pager
feedback/voice UI, and all `xai-computer-hub-{core,sdk,mcp-adapter}` crates +
consumers. `xai-grok-telemetry` remains but is inert (no `track`/`send`).

Remaining **network-capable code not justified** by provider/MCP/web_fetch (recommend removal):

- `xai-grok-shell/src/agent/session_registry_client.rs` (~678 lines) — a reqwest
  client posting to `{proxy}/sessions/*` (`{proxy}` is `api.deepseek.com`). Gated
  off for API-key auth, so it does not run, but the code is present.
- `common/xai-tracing` — a real OTLP gRPC exporter (`fastrace.rs::init_fastrace`;
  `opentelemetry-otlp` / `tonic` / `fastrace-opentelemetry`). Invoked only from the
  standalone `xai-workspace-server` binary when `GROK_WORKSPACE_OTLP_ENDPOINT` is
  set; **not** called by the TUI. Still compiled into dependents.
- `xai-grok-http::shared_upload_client()` — dead helper, no callers.
- `xai-grok-hooks` HTTP handlers — user-configured HTTPS hook URLs.

## Phase 2 — adapter

- **Backend removal (D5): done.** `ApiBackend` now has only `ChatCompletions`;
  the `Responses` and `Messages` conversation modules and sampler stream paths are
  deleted.
- **Wiremock suite: present.** `crates/codegen/xai-grok-sampler/tests/chat_completions_wire.rs`
  covers streaming text + reasoning, a 3-turn tool chain replaying
  `reasoning_content` on every assistant tool call, missing-`reasoning_content`
  backfill, typographic-quote tool-call JSON repair, and a partial/malformed
  stream. Retry/backoff is covered in `tests/test_actor.rs` (429 and 500).
- **Non-stream reasoning capture** is implemented in the conversation conversion
  but has no dedicated mock test (the wiremock suite drives the streaming path).
- **Automatic model routing is not wired.** The `model_routing` module
  (`crates/codegen/xai-grok-sampler/src/model_routing.rs`) and
  `SamplingClient::route_turn` exist, but the policy defaults to `off`, no config
  key populates it, and no turn path calls `route_turn`. Manual `/model` +
  `/effort` selection is what works today. The model catalog
  (`xai-grok-models/default_models.json`) defines `deepseek-v4-pro`,
  `deepseek-flash`, and the hidden `deepseek-v4-flash` alias.

## Phase 3 — TUI

Implemented: reasoning block folding (`e` toggles the selected entry, `E` folds
all entries, `Ctrl+E` toggles all thinking blocks), `/think`, `/model`,
`/effort`, `/theme`, the DeepSeek Monokai default theme with the whale logo, and
a configurable status line (`[ui.status_line]`) whose items include `model`,
`context`, `effort`, `tokens`, `cache`, and `cost` (default items are
`cwd`/`model`/`context`). Remaining: the status row does not yet surface the
auto-routing decision (see the `TODO(auto-routing)` in
`views/status_line/segments.rs`), which is moot until routing is wired.

## Phase 4 — hardening

- **Read-before-write is enforced at runtime.** The write/edit/apply-patch tools
  consult a per-session read tracker and deny a write with a clear message when
  the file was not read first.
- **Dangerous-command findings are surfaced.** `rm -rf`-class commands remain
  gated even under an allow grant and carry a security warning; the classifier
  sees them rather than auto-allowing.
- **Not yet exercised end-to-end against the real API:** plan mode, permission
  modes, compaction, resume, subagents, MCP, `AGENTS.md` loading, and
  checkpoints/undo.

## Rebrand / config

- Binary (`deepseek-build`), config dir (`~/.deepseek-build`), ACP namespace
  (`deepseek-build/*`), theme, and user-facing strings are rebranded.
- **Most `GROK_*` environment variables remain** (~600 distinct names). Only the
  home var (`DEEPSEEK_BUILD_HOME`, with `GROK_HOME` fallback) and the production
  endpoint prefix were renamed. This does not affect the egress gates.
- **No migration from `~/.grok`** to `~/.deepseek-build`; point `GROK_HOME` at the
  old directory if you want the existing sessions/config.

## Licensing

- `LICENSE`, `NOTICE`, `THIRD-PARTY-NOTICES`, and `third_party/NOTICE` are
  preserved. Upstream files are not relicensed. Legal sign-off (DECISIONS O1) is
  still recommended before any public distribution.
