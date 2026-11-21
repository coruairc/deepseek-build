---
last_mapped_commit: 221a1a591ef3af5f90d59d73ae7899d110f28b37
last_mapped_at: 2026-11-21
---
<!-- refreshed: 2026-11-21 -->
# Architecture

**Analysis Date:** 2026-11-21

## System Overview

`deepseek-build` is a terminal coding agent. The user-facing binary is `deepseek-build` (package `xai-grok-pager-bin`). Internal crate names keep the upstream `xai-grok-*` prefix. The process is a composition root over a fullscreen TUI (or a headless single turn) that talks to an in-process ACP agent. The agent owns the turn loop, conversation state, and tool execution. Model I/O is a dedicated sampler actor that streams OpenAI-compatible Chat Completions from `{base_url}/chat/completions` (default `https://api.deepseek.com`).

```text
┌──────────────────────────────────────────────────────────────────────────┐
│  Composition root  `deepseek-build`                                      │
│  `crates/codegen/xai-grok-pager-bin/src/main.rs`                         │
├────────────────────────────┬─────────────────────────────────────────────┤
│  Fullscreen TUI            │  Headless single turn (`-p` / prompt file)  │
│  `xai-grok-pager`          │  `xai-grok-pager/src/headless.rs`            │
│  `app::run` → event loop   │  `run_single_turn`                           │
│  `app/event_loop.rs`       │                                              │
└─────────────┬──────────────┴──────────────────┬──────────────────────────┘
              │  in-process ACP channels         │
              │  `xai-grok-pager/src/acp/spawn.rs` (`spawn_grok_shell`)
              │  or leader IPC `acp/leader_bridge.rs`
              ▼
┌──────────────────────────────────────────────────────────────────────────┐
│  ACP agent  `xai-grok-shell`                                             │
│  `agent/mvp_agent/acp_agent.rs`  prompt()                                │
│  `session/acp_session_impl/turn.rs`  process_conversation_turn           │
│  `session/acp_session_impl/tool_calls.rs`  execute_tool_calls            │
├──────────────────────────────────────────────────────────────────────────┤
│  Chat state actor  `xai-chat-state`  ChatStateHandle::build_request      │
│  Sampler actor     `xai-grok-sampler`  SamplerHandle → POST              │
│                    `{base}/chat/completions`  (SSE)                      │
└─────────────┬────────────────────────────────────────────────────────────┘
              │ tool calls
              ▼
┌──────────────────────────────────────────────────────────────────────────┐
│  Tools + workspace + permissions                                         │
│  `xai-grok-tools`  implementations                                       │
│  `xai-tool-runtime`  Tool / ToolDispatch                                 │
│  `xai-grok-workspace`  FS/VCS + permission manager                       │
│  `xai-grok-sandbox`  OS confinement for child processes                  │
│  `xai-grok-mcp`  user-configured MCP (stdio / HTTP)                      │
└──────────────────────────────────────────────────────────────────────────┘
```

Stdio ACP (`deepseek-build agent stdio`) skips the TUI and enters the same agent through `xai-grok-shell/src/agent/app.rs` `run_stdio_agent`. Leader mode (`run_leader`) serves multiple IPC clients from one agent process.

## Component Responsibilities

| Component | Responsibility | File |
|-----------|----------------|------|
| Binary / CLI dispatch | Parse `PagerArgs`, install crash handler, minimal-mode hooks, route subcommands, TUI, or headless | `crates/codegen/xai-grok-pager-bin/src/main.rs` |
| Agent runtime seam | Stdio/leader backend selection (only `Backend::Shell`) | `crates/codegen/xai-grok-pager/src/agent_runtime.rs` |
| TUI app | Config load, ACP connect, terminal init, event loop, restore | `crates/codegen/xai-grok-pager/src/app/mod.rs` |
| Event loop | `tokio::select!` over terminal, ACP, tasks, ticks, config watch | `crates/codegen/xai-grok-pager/src/app/event_loop.rs` |
| ACP spawn | In-process GrokShell on a dedicated OS thread; ACP channel pair | `crates/codegen/xai-grok-pager/src/acp/spawn.rs` |
| Headless | One prompt, ACP lifecycle, stdout emitters (`plain` / `json` / streaming) | `crates/codegen/xai-grok-pager/src/headless.rs` |
| Stdio / leader agent | ACP stdio server or leader IPC server | `crates/codegen/xai-grok-shell/src/agent/app.rs` |
| ACP agent | `prompt`, session create/resume/close | `crates/codegen/xai-grok-shell/src/agent/mvp_agent/acp_agent.rs` |
| Session actor | Turn loop, tool batches, compaction hooks, persistence | `crates/codegen/xai-grok-shell/src/session/acp_session.rs` and `session/acp_session_impl/` |
| Chat state | Message history and request assembly, no locks inside the actor | `crates/codegen/xai-chat-state/src/lib.rs` |
| Sampler | HTTP streaming, retries, cancellation, doom-loop recovery | `crates/codegen/xai-grok-sampler/src/lib.rs` |
| Wire types | `ApiBackend::ChatCompletions`, `ThinkingConfig`, `TokenUsage` | `crates/codegen/xai-grok-sampling-types/src/types.rs` |
| Endpoints | Default base `https://api.deepseek.com`; env/config overrides | `crates/codegen/xai-grok-config/src/endpoints.rs` |
| Model catalog | Embedded defaults from `default_models.json` | `crates/codegen/xai-grok-models/src/lib.rs` |
| Prompts | Agent discovery and XOR-obfuscated system prompt | `crates/codegen/xai-grok-agent/` (`templates/prompt.md`, `src/prompt/`) |
| Tools | Built-in tool implementations and bridge | `crates/codegen/xai-grok-tools/src/lib.rs` |
| Tool contract | `Tool`, `ToolDispatch`, `ToolError`, streams | `crates/common/xai-tool-runtime/src/lib.rs` |
| Permissions | Mode/rule evaluation and the permission-manager actor | `crates/codegen/xai-grok-permission-rules/src/rules.rs`, `crates/codegen/xai-grok-workspace/src/permission/` |
| Workspace | FS/VCS ops, folder trust, sandbox gates | `crates/codegen/xai-grok-workspace/src/lib.rs` |
| MCP | User-configured servers (stdio/HTTP), elicitation, OAuth | `crates/codegen/xai-grok-mcp/src/lib.rs` |
| Brand | User-visible name, config dir, env prefix | `crates/codegen/xai-grok-brand/src/lib.rs` |
| Auth | Live path is `DEEPSEEK_API_KEY` (fallback env in `auth_method.rs`) | `crates/codegen/xai-grok-login/src/auth_method.rs` |

## Pattern Overview

**Overall:** Actor-oriented ACP agent behind a TUI (or headless) client, with a single Chat Completions adapter.

**Key characteristics:**

- **ACP is the session boundary.** The pager never calls the sampler directly. It sends Agent Client Protocol requests over in-process channels (`xai-acp-lib`) or, in leader mode, over a Unix socket bridged in `crates/codegen/xai-grok-pager/src/acp/leader_bridge.rs`.
- **Actors own mutable state.** `ChatStateActor`, `SamplerActor`, and the permission manager each run on a dedicated task and expose a handle. The session turn loop is itself a session actor (`run_session` in `session/acp_session_impl/run_loop.rs`).
- **One wire protocol.** `ApiBackend` has a single variant, `ChatCompletions` (`crates/codegen/xai-grok-sampling-types/src/types.rs`). Do not reintroduce a Responses or Anthropic Messages backend.
- **Composition root is the binary, not the pager library.** `xai-grok-pager-minimal` depends on `xai-grok-pager`. The binary installs minimal-mode function-pointer hooks at startup (`xai-grok-pager-bin/Cargo.toml`, `main.rs` calls `xai_grok_pager_minimal::install()`).
- **Egress is closed.** Model traffic goes to the configured provider base. MCP traffic goes only to user-configured servers. Do not add a new outbound host from a feature change.

## Layers

**Composition / process:**

- Purpose: CLI parse, sandbox policy heal, tracing, crash handler, dispatch.
- Location: `crates/codegen/xai-grok-pager-bin/src/main.rs`
- Contains: `fn main`, subcommand match, headless vs `app::run` branch.
- Depends on: `xai-grok-pager`, `xai-grok-shell`, `xai-grok-login`, `xai-crash-handler`, `xai-grok-telemetry`.
- Used by: the `deepseek-build` process only.

**Presentation (TUI / headless):**

- Purpose: Render scrollback, route keys, drive ACP, emit headless JSON.
- Location: `crates/codegen/xai-grok-pager/src/`
- Contains: `app/` (views, effects, event loop), `scrollback/`, `views/`, `headless/`, `slash/`, `acp/`.
- Depends on: `xai-grok-pager-render`, `xai-grok-shell` (config, spawn types), `agent-client-protocol`.
- Used by: the binary. Must not depend on `xai-grok-pager-minimal`.

**Agent protocol:**

- Purpose: ACP server, session lifecycle, subagents, leader IPC.
- Location: `crates/codegen/xai-grok-shell/src/agent/`
- Contains: `app.rs` (`run_stdio_agent`, `run_leader`), `mvp_agent/`, `init.rs`, `subagent/`.
- Depends on: session actor, auth, models, telemetry.
- Used by: pager spawn, `agent` subcommand, leader.

**Turn / session:**

- Purpose: Assemble a turn, call the sampler, execute tools, compact, persist.
- Location: `crates/codegen/xai-grok-shell/src/session/acp_session_impl/`
- Contains: `turn.rs`, `sampler_turn.rs`, `tool_calls.rs`, `tool_dispatch.rs`, `spawn.rs`, `run_loop.rs`, `compaction` wiring via `session/compaction.rs`.
- Depends on: `xai-chat-state`, `xai-grok-sampler`, `xai-grok-tools`, `xai-grok-workspace`.
- Used by: `MvpAgent::prompt`.

**Conversation state:**

- Purpose: Own `Vec<ConversationItem>`, sampling config, prompt index, token estimates; build the request the sampler sends.
- Location: `crates/codegen/xai-chat-state/src/`
- Contains: `actor/`, `handle.rs`, `commands.rs`, `persistence.rs`.
- Depends on: sampling types, compaction transcript types.
- Used by: the session actor only (via `ChatStateHandle`).

**Model adapter:**

- Purpose: POST Chat Completions, parse SSE, retry, sanitize `reasoning_content`, repair typographic tool-call JSON.
- Location: `crates/codegen/xai-grok-sampler/src/` plus `crates/codegen/xai-grok-sampling-types/`
- Contains: `client.rs` (HTTP), `actor/` (concurrency), `stream/chat_completions.rs` (pure transform), `retry.rs`.
- Depends on: `reqwest` built through `xai-grok-extra-ca`, `eventsource-stream`.
- Used by: session sampler turn, compaction side-calls.

**Tools and side effects:**

- Purpose: Execute model tool calls under permission and sandbox policy.
- Location: `crates/codegen/xai-grok-tools/src/implementations/`, `crates/common/xai-tool-runtime/`, `crates/codegen/xai-grok-workspace/`
- Contains: bash/read/edit/search implementations under `implementations/grok_build/`, permission manager under `workspace/src/permission/manager/`.
- Depends on: `xai-tool-protocol`, `xai-grok-sandbox`, `xai-grok-permission-rules`.
- Used by: `execute_tool_calls` → `dispatch_tool` / `WorkspaceOps::call_tool`.

**Configuration and identity:**

- Purpose: Layered TOML, endpoints, brand strings, API key.
- Location: `crates/codegen/xai-grok-config/`, `xai-grok-env/`, `xai-grok-brand/`, `xai-grok-login/`
- Contains: `config_layers.rs` (system managed → managed → user → `DEEPSEEK_BUILD_CONFIG` overlay → requirements), `endpoints.rs`.
- Depends on: filesystem under `~/.deepseek-build` (`xai-grok-brand::CONFIG_DIR_NAME`).
- Used by: every entrypoint before the first model call.

## Data Flow

### Primary request path (interactive TUI)

1. `fn main` parses `PagerArgs` and, when there is no headless prompt, calls `xai_grok_pager::app::run` (`crates/codegen/xai-grok-pager-bin/src/main.rs` around the `app::run` call).
2. `app::run` loads effective config, resolves leader mode, and connects an ACP client (`crates/codegen/xai-grok-pager/src/app/mod.rs`). Default path spawns the agent in-process via `spawn_grok_shell` (`crates/codegen/xai-grok-pager/src/acp/spawn.rs`).
3. `event_loop::run` selects on terminal input and the ACP channel (`crates/codegen/xai-grok-pager/src/app/event_loop.rs`). A user submit becomes an ACP `session/prompt`.
4. `MvpAgent::prompt` looks up the session handle (`crates/codegen/xai-grok-shell/src/agent/mvp_agent/acp_agent.rs`).
5. `SessionActor::process_conversation_turn` loops: resolve tools, `chat_state_handle.build_request`, then `run_turn_via_sampler` (`session/acp_session_impl/turn.rs`, `sampler_turn.rs`).
6. `SamplerHandle::submit_and_collect` posts to `SamplingClient`'s `chat/completions` endpoint and streams SSE (`crates/codegen/xai-grok-sampler/src/client.rs`, `src/stream/chat_completions.rs`). Default base is `XAI_API_BASE_URL_DEFAULT` = `https://api.deepseek.com` in `crates/codegen/xai-grok-config/src/endpoints.rs`. The field name is still `xai_api_base_url`; do not rename it in a drive-by.
7. If the assistant message contains tool calls, `execute_tool_calls` batches them (`session/acp_session_impl/tool_calls.rs`). Dispatch goes through workspace ops and the permission manager spawned in `session/acp_session_impl/spawn.rs` (`spawn_permission_manager_with_pin`).
8. Tool results are pushed back onto chat state and the loop samples again until a non-tool stop. ACP session updates stream to the pager, which renders them in scrollback (`crates/codegen/xai-grok-pager/src/scrollback/`).

### Headless single turn

1. `main` detects `-p` / `--prompt-file` / `--prompt-json` (or `--memory-flush`) and calls `headless::run_single_turn` (`xai-grok-pager-bin/src/main.rs`, `xai-grok-pager/src/headless.rs`).
2. Headless sets `AgentMode::Headless`, spawns the same in-process agent (`spawn_grok_shell`), drives ACP initialize / authenticate / prompt, and writes stdout in the selected output format.
3. The turn loop underneath is the same session actor path as the TUI.

### Stdio ACP and leader

1. `deepseek-build agent stdio` builds `AgentRuntime` and calls `run_stdio` → `run_stdio_agent` (`agent_runtime.rs`, `agent/app.rs`). ACP JSON-RPC is stdin/stdout. The agent body runs on a `tokio::task::LocalSet`.
2. `run_leader` binds a socket, takes `LeaderLock`, and serves multiple pager clients (`agent/app.rs`, `leader/server.rs`). The pager bridges that socket in `acp/leader_bridge.rs`.

**State management:**

- Conversation items live only inside `ChatStateActor`. Callers use `ChatStateHandle` (command channel + oneshot queries). Do not keep a second mutable transcript in the TUI.
- Sampler in-flight requests live inside `SamplerActor`. Cancellation is dropping the `submit_and_collect` guard.
- Permission mode and grants live in the permission-manager actor (`xai-grok-workspace/src/permission/manager/mod.rs`). Rules and `DefaultPermissionMode` (`default`, `acceptEdits`, `plan`, `auto`, `dontAsk`, `bypassPermissions`) are pure data in `xai-grok-permission-rules`.
- Scrollback is UI state in `xai-grok-pager/src/scrollback/`. It is a projection of ACP updates, not the source of truth for the next model request.
- Sessions persist through `xai-grok-shell/src/session/persistence.rs` and `xai-chat-state/src/persistence.rs`. Resume replays that store; the TUI does not reconstruct history by scraping the terminal.

## Key Abstractions

**ACP session (`SessionActor`):**

- Purpose: One conversation's turn loop, tool context, MCP attachments, and compaction.
- Examples: `crates/codegen/xai-grok-shell/src/session/acp_session.rs`, `session/acp_session_impl/turn.rs`
- Pattern: `Arc<SessionActor>` methods, `spawn_local` for the run loop, commands over a mailbox (`session/commands.rs`).

**ChatStateHandle:**

- Purpose: Build `ConversationRequest` from history + tool specs + memory reminder.
- Examples: `crates/codegen/xai-chat-state/src/handle.rs`, `src/actor/`
- Pattern: Same actor shape as `xai-hunk-tracker` (documented in `xai-chat-state/src/lib.rs`). The turn loop `expect`s the actor to be alive; do not drop it mid-turn.

**SamplerHandle / SamplingClient:**

- Purpose: Concurrent Chat Completions with retry classification.
- Examples: `crates/codegen/xai-grok-sampler/src/handle.rs`, `src/client.rs`, `src/stream/chat_completions.rs`
- Pattern: Three layers — raw HTTP stream, pure `SamplingEvent` transform, actor that owns cancellation and retries. `ApiBackend` is a single-variant marker.

**Tool + ToolDispatch:**

- Purpose: One contract for built-in tools and MCP tools.
- Examples: `crates/common/xai-tool-runtime/src/tool.rs`, `src/dispatch.rs`; implementations in `crates/codegen/xai-grok-tools/src/implementations/`
- Pattern: Implement `Tool`, register through the tools bridge (`xai-grok-tools/src/bridge.rs`). Session code calls workspace ops, not individual tool structs.

**Permission manager:**

- Purpose: Allow / deny / prompt before a side effect, including dangerous-command detection.
- Examples: `crates/codegen/xai-grok-workspace/src/permission/manager/mod.rs`, `crates/codegen/xai-grok-permission-rules/src/exec_risk.rs`
- Pattern: Spawned once per owning session (`owns_permission_manager` in `acp_session_impl/spawn.rs`). Child sessions inherit a handle; they must not spawn a second manager.

**Brand constants:**

- Purpose: Every user-visible name.
- Examples: `crates/codegen/xai-grok-brand/src/lib.rs` (`NAME`, `CONFIG_DIR_NAME`, `ENV_PREFIX`)
- Pattern: Read constants. Do not hardcode `deepseek-build` or `DEEPSEEK_BUILD_*` in new UI strings when a constant exists.

## Entry Points

**`deepseek-build` binary:**

- Location: `crates/codegen/xai-grok-pager-bin/src/main.rs` (`fn main`, `[[bin]] name = "deepseek-build"`)
- Triggers: user invocation, installer, release tarball.
- Responsibilities: process identity, requirements check, subcommand dispatch, headless vs TUI.

**Interactive TUI:**

- Location: `crates/codegen/xai-grok-pager/src/app/mod.rs` `run`
- Triggers: bare `deepseek-build` (no `-p`).
- Responsibilities: connect ACP, own the terminal, run `event_loop::run`.

**Headless:**

- Location: `crates/codegen/xai-grok-pager/src/headless.rs` `run_single_turn`
- Triggers: `-p`, `--prompt-file`, `--prompt-json`, `--memory-flush`.
- Responsibilities: one ACP prompt, structured stdout, no fullscreen terminal.

**Stdio agent:**

- Location: `crates/codegen/xai-grok-shell/src/agent/app.rs` `run_stdio_agent`
- Triggers: `deepseek-build agent stdio`, external ACP clients.
- Responsibilities: newline-framed ACP on stdin/stdout.

**Leader:**

- Location: `crates/codegen/xai-grok-shell/src/agent/app.rs` `run_leader`
- Triggers: `deepseek-build agent leader` or pager leader mode.
- Responsibilities: single-instance flock, IPC accept loop, shared sessions.

**Mermaid worker:**

- Location: `crates/codegen/xai-grok-pager/src/app/mermaid_worker.rs` (`maybe_run_render_subprocess` from `main`)
- Triggers: re-exec of the same binary as a render subprocess.
- Responsibilities: exit before the normal CLI if the worker env is set.

**Other CLIs (same binary):** `doctor`, `mcp`, `plugin`, `memory`, `sessions`, `models`, `worktree`, `workspace`, `export`, `wrap`, `login`/`logout`, `completions`. Each is a `Command` arm in `main.rs` and a `*_cmd` module under `xai-grok-pager/src/`.

**Secondary binaries (not the product):**

- `ptyctl` / `ptyctl-cli` — headless PTY controller (`crates/codegen/ptyctl/`).
- Workspace daemon — process lifecycle for a workspace server (`crates/codegen/xai-grok-workspace-daemon/`).

## Architectural Constraints

- **Threading:** The binary builds a multi-thread Tokio runtime (`rt-multi-thread` in `xai-grok-pager-bin/Cargo.toml`). The interactive agent runs on its own OS thread (`acp/spawn.rs`); its session loop uses `spawn_local` (`run_loop.rs`). `run_stdio_agent` wraps the agent in `LocalSet` because the ACP agent is not `Send`. The TUI event loop is one task; stdin is a dedicated reader thread (`app/reader_thread.rs`). Do not assume session state is `Send`.
- **Global state:** `xai-grok-models` `LazyLock` over `default_models.json`; process identity and OTEL gate in `xai-grok-telemetry` / `agent/otel_gate.rs`; `xai_fsnotify::set_runtime_handle` for watchers that outlive a session (`agent/app.rs`); jemalloc as the global allocator when the `jemalloc` feature is on (`pager-bin/src/main.rs`). Endpoints `Default` reads env vars at construction (`endpoints.rs`) — construct it once per process boot, not per request.
- **Circular imports:** `xai-grok-pager` must not depend on `xai-grok-pager-minimal`. The binary is the only crate allowed to link both. `xai-grok-shell` re-exports `xai-grok-shell-base`, `xai-grok-http`, `xai-grok-models`, and `xai-grok-login`; new features should depend on the leaf crate, not grow the shell re-export surface, unless pager already imports that path.
- **Indexing:** Production crates use `#![deny(clippy::indexing_slicing)]` (pager, shell, sampler, tools, chat-state, models). Use `get` / explicit bounds. Tests may index.
- **Paths:** Canonicalize with `dunce::canonicalize`, not `std::fs::canonicalize` (repo `clippy.toml`). Home directories go through `xai-dirs` / `xai_grok_shell::util::grok_home`. Child processes enroll via `xai_tty_utils::ProcessScope`. HTTP clients are built through `xai_grok_extra_ca`.
- **Egress:** Only the configured model base and user-configured MCP servers. A change that adds a destination stops for approval. Do not recreate deleted upload, telemetry OTLP, voice, or computer-hub crates.
- **Prompt templates:** Files under `crates/codegen/xai-grok-agent/templates/` are XOR-obfuscated into generated Rust. After any template edit, run `python3 scripts/encrypt_templates.py`.
- **Adapter ownership:** Do not scrub or casually rename `xai-grok-env`, `xai-grok-config/src/endpoints.rs`, `xai-grok-models`, `xai-grok-sampling-types`, or `xai-grok-sampler`. Those crates own the DeepSeek adapter.
- **Internal names:** Crate and package names stay `xai-grok-*`. User-facing strings go through `xai-grok-brand`. Legacy `GROK_*` env fallbacks are gone; remaining `GROK_*` identifiers are internal.

## Anti-Patterns

### Calling the sampler from the TUI

**What happens:** A pager view or slash command builds an HTTP request to `api.deepseek.com` itself.
**Why it's wrong:** Auth, retries, `reasoning_content` round-trip, and cost accounting all live in the session actor and sampler. A side channel skips permission context and session persistence.
**Do this instead:** Send an ACP prompt or a session command. Side model calls (compaction, titles) go through the session's `SamplerHandle` (`session/acp_session.rs` holds `sampler_handle`).

### Second conversation store in the UI

**What happens:** Scrollback entries are treated as the transcript to resume or to send back to the model.
**Why it's wrong:** Scrollback is a render projection (`xai-grok-pager/src/scrollback/`). Compaction and tool-result truncation happen in chat state.
**Do this instead:** Read history through `ChatStateHandle` or session persistence (`xai-grok-shell/src/session/persistence.rs`).

### Linking `xai-grok-pager-minimal` from the pager library

**What happens:** A render-mode change adds `xai-grok-pager-minimal` to `xai-grok-pager`'s `Cargo.toml`.
**Why it's wrong:** `xai-grok-pager-minimal` already depends on `xai-grok-pager`. Cargo rejects the cycle. The binary exists specifically to install the fn-pointer seam (`xai-grok-pager-bin/Cargo.toml`).
**Do this instead:** Extend the seam in `xai-grok-pager/src/minimal/` and install it from `xai_grok_pager_minimal::install()` in `main.rs`.

### New `ApiBackend` variant or raw `/chat/completions` client

**What happens:** A feature posts JSON with `reqwest` directly, or adds a Responses API variant.
**Why it's wrong:** `ApiBackend` is intentionally one variant (`sampling-types/src/types.rs`). Streaming, typographic-quote repair (`stream/chat_completions.rs`), and retry classification will not run.
**Do this instead:** Extend `SamplingClient` / `stream_chat_completions` and keep the session on `run_turn_via_sampler`.

### Recreating deleted exfil / telemetry crates

**What happens:** A stub `upload/`, `cloud_config/`, OTEL exporter, or computer-hub crate is added back "for compatibility".
**Why it's wrong:** Those paths were deleted so the egress gate stays closed. Comments in `xai-tool-protocol` that still say "computer hub" are leftover names, not a license to restore the crates.
**Do this instead:** Leave the paths absent. Protocol comments are not live network.

### `std` canonicalize or ad-hoc HTTP clients

**What happens:** New code calls `std::fs::canonicalize` or `reqwest::Client::new()`.
**Why it's wrong:** Windows/verbatim path behavior and the TLS extra-CA policy are enforced by clippy bans (`clippy.toml`).
**Do this instead:** `dunce::canonicalize` and `xai_grok_extra_ca`.

## Error Handling

**Strategy:** Protocol boundaries use `acp::Error`. Process/CLI boundaries use `anyhow::Result`. Sampler failures are structured `SamplingError` / `SamplingErrorInfo` and classified before they surface. Tool failures are `xai_tool_runtime::ToolError` and become tool-result messages, not process crashes, unless the session actor itself is gone.

**Patterns:**

- CLI arms in `main.rs` print to stderr and `process::exit` for usage/config failures, or return `anyhow::Result` from the async dispatch.
- `process_conversation_turn` returns `Result<TurnOutcome, acp::Error>` (`turn.rs`). Transient API failures retry inside `run_turn_via_sampler` (`sampler_turn.rs`) using `xai_grok_sampler::classify_error` and jittered backoff (`sampler/src/retry.rs`).
- Empty max-token continuations are salvaged in the turn loop (`length_salvage`) rather than failing the prompt immediately.
- `chat_state_handle.build_request(...).await.expect("chat state actor should be alive")` is a liveness assert, not a user-facing error. If you add a path that can shut the actor down mid-turn, replace that expect with an `acp::Error` — do not leave a panic on the user path.
- Permission denials are manager verdicts, not panics. Unknown `permissions.defaultMode` strings fail `FromStr` and fall back without widening the parent mode (`xai-grok-permission-rules/src/rules.rs`).
- Headless tracing defaults to `off` so stdout stays machine-readable (`HEADLESS_ENTRYPOINT` in `main.rs`).

## Cross-Cutting Concerns

**Logging:** `tracing` spans at the turn (`session.process_conversation_turn`), tool batch (`tools.execute`), and request build (`turn.build_request`). Structured product logs go through `xai_grok_telemetry::unified_log` (for example `shell.turn.inference_start`). Do not log API keys, bearer suffixes beyond the existing attribution helper, or full prompt bodies at info.

**Validation:** Config layers merge in `xai-grok-config/src/config_layers.rs` and are checked by `validate_requirements` before the runtime starts (`main.rs`). Tool arguments are parsed in the session (`tool_dispatch.rs`, `mcp_argument_coercion.rs`). Tool-call JSON that fails because of typographic quotes is repaired in `stream/chat_completions.rs` before it is treated as a model bug.

**Authentication:** The live credential is `DEEPSEEK_API_KEY` (`xai-grok-login/src/auth_method.rs`, constant `XAI_API_KEY_ENV_VAR`). The sampler receives a bearer resolver (`SamplerConfig` in `xai-grok-sampler/src/config.rs`), not a copied key in session files. Interactive login text tells the user to set that variable (`xai-grok-login/src/flow.rs`). MCP servers may carry their own bearer file (`xai-grok-config/src/mcp_bearer_token_file.rs`); that token must not be written into saved sessions or logs.

**Compaction:** Triggered from the session, implemented in `crates/common/xai-grok-compaction/` with transcript types in `crates/codegen/xai-compaction-transcript/`. Image budgets live in `xai-chat-state/src/image_budget.rs`. Do not compact by deleting scrollback rows.

**Hooks and plugins:** Hook registry and plugin marketplace are `xai-grok-hooks` and `xai-grok-plugin-marketplace`. Session dispatch is `acp_session_impl/hook_dispatch.rs`. User-facing config reference is hand-edited at `crates/codegen/xai-grok-pager/docs/user-guide/26-config-reference.md`; keep it in sync with the registries rather than regenerating it.

---

*Architecture analysis: 2026-11-21*
