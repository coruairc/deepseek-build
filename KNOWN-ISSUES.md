# KNOWN ISSUES — deepseek-build

Honest list of what is incomplete or broken at this revision. Nothing here is
hidden behind a "temporary" flag; where code remains it is called out. Claims are
against the tree on branch `dsb/docs` (based on `dsb/integration` at `d7b6f00a`).

## Build & tests

- **`cargo test --workspace` does not compile.** Verified on this branch with
  `cargo test --workspace --no-run` (Rust 1.94.0): it stops with
  `xai-grok-agent (lib test)` (11 errors) and `xai-grok-workspace (lib)` +
  `(lib test)` (71 errors between them). The failures are stale test / test-support
  references to Phase 1 deletions, for example `xai_file_utils`, `upload`,
  `implementations::web_search`, `grok_build::{image_gen,video_gen}`,
  `SendFeedbackTool`, `persist_and_enqueue_tool_state`, and
  `SessionContext::{web_search_config,image_gen_config,video_gen_config}` (the
  `test-support` factory still sets those fields). The pager's `#[cfg(test)]`
  modules also still `use crate::xai_grok_voice`, but the voice module was
  deleted, so `xai-grok-pager` tests will fail once the build reaches them.
- **Non-test `cargo check` compiles.** `cargo check -p xai-grok-pager-bin`
  succeeds (verified on this branch); only test / `test-support`-gated targets hit
  the errors above. `cargo build --release` was reported green by the earlier
  handoff and was not rebuilt during this docs pass. Repairing the test suite is
  a dedicated pass.
- **`cargo clippy` was only run on the crates touched per slice**, not across the
  whole workspace, after the large deletions.

## Network / egress

- **HARD and SOFT gates: green.** `scripts/check-egress.sh --strict` prints
  `HARD egress gate: OK` and exits 0 (verified during the Phase 5 docs pass).
- **Runtime egress (observed):** a single-turn prompt contacts only
  `api.deepseek.com:443` plus one local `AF_UNIX` socket. `--version`/`--help`
  make no outbound connections. This was measured with an `LD_PRELOAD`
  connect-logger because `strace` and network namespaces were unavailable in the
  dev environment; reproduce with `strace -f -e trace=connect` where available.
  The same `LD_PRELOAD` approach is what `scripts/sandbox-run.sh` ships.
- **Live smoke test not run.** No real `DEEPSEEK_API_KEY` was provided, so the
  end-to-end chat / streamed chat / 3-turn thinking tool-call chain / usage-cache
  assertions against the live API are unverified. The request path was verified
  as far as a 401 response (dummy key) allows.

## Phase 1 cleanup — remaining stubs / undeleted code

These contain **no network egress** but were not fully deleted:

- `xai-grok-shell/src/upload/**` — restored inert, routed to
  `file_utils_compat` (no storage HTTP).
- `xai-grok-shell/src/session/repo_changes/mod.rs` — pure serde types only.
- `xai-grok-shell/src/session/feedback_manager.rs` — inert; local-only. The
  `/feedback` command and modal remain, but sending performs no upload.
- `xai-grok-shell/src/cloud_config/**` — the former `xai-grok-cloud-config`
  crate, vendored inert into the shell.
- `xai-grok-telemetry/src/{external.rs,otel_layer.rs,trace_context.rs}` — no-op
  OTLP/trace stubs.
- `xai-grok-pager/src/views/announcements.rs` — no-op.
- `xai-computer-hub-{core,sdk,mcp-adapter}` crates (under `crates/common/`) and
  their consumers (`xai-grok-workspace`, `xai-grok-workspace-{client,daemon}`,
  `xai-grok-tools`, `xai-grok-mcp`, shell) — present, de-egressed but **not
  deleted**.
- `xai-grok-shell/src/remote/**` — remote model/agent clients, present with
  neutralized hosts. The relay/WebSocket layer and `agent/relay.rs` were deleted.
- `prod/mc/model-api-types` storage wire types remain (used by the inert upload
  shim).

**Removed since the earlier handoff:** xAI voice dictation (the pager
`xai_grok_voice.rs` stub and capture subprocess), session share, and the remote
WebSocket relay / agent-serve subcommands.

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
