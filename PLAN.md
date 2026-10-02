# PLAN — Turning Grok Build into a DeepSeek-first coding agent (`<PROJECT_NAME>`)

Status: **Phase 0 complete (audit + plan). No code changed.**
Auditor: senior Rust engineer. Date: 2026-10-02.
Upstream: xAI Grok Build, Apache-2.0. `SOURCE_REV=559751fdcec02d413e4c57c8832ab275e4f44980`.

Scope of this document: findings, keep/rip-out decisions, a phased implementation plan
with diff-size estimates and risks, and the open questions that need a decision before
Phase 1.

> Naming: the project name is **`deepseek-build`** (see [`DECISIONS.md`](DECISIONS.md)).
> Earlier drafts used a `<PROJECT_NAME>` placeholder.

## Implementation progress (updated 2026-10-02)

| Area | Status |
|------|--------|
| Phase 0 audit/plan | Complete |
| Phase 1a egress removal | **Not done** — ~2,923 forbidden host strings remain. |
| Phase 1b rebrand | **Not done** |
| Phase 1c egress guard | Done: `scripts/check-egress.sh` (currently red) |
| Upstream NOTICE | Done |
| Phase 2 adapter core | Done (`cdbf5aa3`): thinking control, reasoning round-trip + sanitizer, typographic-quote repair, cache helpers, DeepSeek default catalog. |
| Phase 2 mock-server suite / credential E2E / auto-router | **Not done** |
| Phase 3 TUI | **Not done** |
| Phase 4 adapt existing features | **Not done** |

---

## 0. Executive summary

- **Scale.** 104 workspace crates, 3,689 `.rs` files, ~2.0M LOC. The product binary is
  `xai-grok-pager` (package `xai-grok-pager-bin`), installed upstream as `grok`.
- **Architecture is sound and reusable.** The LLM layer is protocol-shaped (`ApiBackend`
  = `ChatCompletions` / `Responses` / `Messages`) with per-model `base_url` + auth scheme,
  over a hand-rolled `reqwest` sampler. DeepSeek is OpenAI-compatible chat completions, so
  the existing `ChatCompletions` path is the right seam — and it already has a
  `reasoning_content` field and folds reasoning into assistant messages. Phase 2 is an
  adapter + hardening job, not a rewrite.
- **Security: the repo contains real data-exfiltration infrastructure, and it is not purely
  cosmetic.** `xai-file-utils` + `xai-grok-shell/src/upload/*` implement a spill-to-disk
  queue that uploads repository changes/dedup blobs/patches, session state, full prompts,
  tool definitions, images, unified logs, memory archives, heap dumps, auth diagnostics and
  more, to GCS (`gs://`), S3 (`s3://`) or the `cli-chat-proxy` `/v1/storage/*` API. The
  repository *also* has product telemetry (Mixpanel + a custom events endpoint), Sentry
  crash reporting, external OTLP export, remote config/feature flags, a remote-control
  relay, and an auto-updater pointed at `x.ai` / Google Cloud Storage. All of this must be
  **deleted**, not disabled.
- **The "server-side flag" concern is confirmed.** In a stock OSS build the baked telemetry
  defaults are empty and the mode defaults to `Disabled`, but the gates are reachable from
  several directions: env (`GROK_TELEMETRY_ENABLED`, `GROK_TELEMETRY_TRACE_UPLOAD`,
  `GROK_TRACE_UPLOAD_*`), `requirements.toml`, and the remote `GET {proxy}/settings`
  response (`telemetry_enabled`, `telemetry_mode`, `trace_upload_enabled`). A server
  response can turn uploads on. Deleting the code is the only safe end state.
- **xAI coupling is deep but mechanical.** ~1,372 Rust files mention `grok`/`xai`, ~623
  distinct `GROK_*` env vars, `~/.grok` config dir, `auth.x.ai` login, Grok model catalog,
  `x.ai/*` ACP extension methods, Grok Imagine image/video, voice STT, xAI plugin
  marketplace.
- **Licensing is compatible.** First-party code is Apache-2.0 (`LICENSE` = "Copyright
  2023-2026 SpaceXAI"). 106/107 first-party `Cargo.toml` say Apache-2.0; one says MIT.
  Vendored `third_party/*` are MIT/Apache-2.0. Ports from openai/codex (Apache-2.0) and
  sst/opencode (MIT) are attributed in-crate. No first-party copyleft. One transitive
  concern to preserve: **libgit2 GPL-2.0 + linking exception** (noted in `THIRD-PARTY-NOTICES`).
  We must keep `LICENSE`, `THIRD-PARTY-NOTICES`, `third_party/NOTICE`, and the in-crate
  notices to satisfy Apache-2.0 §4.

---

## 1. Workspace architecture map

### 1.1 Crate topology (the spine)

| Layer | Crates |
|---|---|
| Binary / CLI | `xai-grok-pager-bin` (binary `xai-grok-pager`), `xai-grok-pager` |
| TUI / render | `xai-grok-pager`, `xai-grok-pager-render`, `xai-grok-pager-minimal`, `xai-ratatui-*` |
| Agent runtime | `xai-grok-shell` (session/turn), `xai-grok-shell-base` |
| Prompt/definition | `xai-grok-agent` |
| Sampling/LLM | `xai-grok-sampler`, `xai-grok-sampling-types`, `xai-grok-models`, `xai-chat-state` |
| Tools | `xai-grok-tools`, `xai-grok-tools-api`, `xai-tool-runtime`, `xai-tool-protocol`, `xai-tool-types` |
| Permissions | `xai-grok-permission-rules`, `xai-grok-workspace/permission` |
| Context | `xai-chat-state`, `xai-grok-compaction`, `xai-compaction-transcript`, `xai-token-estimation` |
| Persistence | `xai-sqlite-journal`, `xai-grok-session-events`, `xai-grok-active-sessions`, `xai-grok-foreign-sessions` |
| Subagents | `xai-grok-subagent-resolution` |
| Undo | `xai-hunk-tracker`, `xai-grok-workspace/session/{file_state,checkpoint,checkpoint_store}.rs` |
| Config | `xai-grok-config`, `xai-grok-config-types`, `xai-grok-env`, `xai-dirs`, `xai-grok-paths` |
| MCP | `xai-grok-mcp`, config types in `xai-grok-config` |
| **To delete (data egress)** | `xai-file-utils`, `xai-grok-telemetry`, `xai-mixpanel`, `xai-grok-otel`, `xai-crash-handler`, `xai-grok-feedback`, `xai-grok-update` |
| **To delete (xAI account/cloud)** | `xai-grok-login`, `xai-grok-cloud-config`, `xai-grok-announcements`, `xai-grok-voice`, `xai-grok-egress-proxy` (repurpose/keep?), `xai-grok-diag-server` (local), `xai-grok-bundle` (xAI artifact download) |
| Vendored | `third_party/{mermaid-to-svg,dagre_rust,graphlib_rust,ordered_hashmap}` |

### 1.2 Entry points

- `crates/codegen/xai-grok-pager-bin/src/main.rs:2036` `fn main`; `:2133` `async_main`.
  Dispatches subcommands (`Command` enum in `xai-grok-pager/src/app/cli.rs:8`):
  `Agent`, `Doctor`, `Setup`, `Mcp`, `Plugin`, `Models`, `Leader`, `Worktree`,
  `Sessions`, `Usage`, `Share`, `Export`, `Trace`, `Memory`, `Update`, `Login`, `Logout`, …
- TUI: `xai-grok-pager::app::run` (`crates/codegen/xai-grok-pager/src/app/mod.rs:707`).
- Headless single turn: `xai-grok-pager/src/headless.rs:804` `run_single_turn`.
- ACP stdio: `xai-grok-shell/src/agent/app.rs:220` `run_stdio_agent`.
- Relay/leader: `agent/app.rs:314` `run_headless`, `:698` `run_leader`;
  `xai-grok-shell/src/agent/server.rs:610` `run_agent_server`.
- Startup side effects to strip: `main.rs:560-576` and `:1259-1298` install remote settings;
  `main.rs:2089-2108` installs the crash handler; telemetry/Sentry/update boot around there.

### 1.3 Provider / model abstraction (the DeepSeek seam)

- **Backend switch:** `ApiBackend { ChatCompletions, Responses, Messages }` —
  `xai-grok-sampling-types/src/types.rs:1104-1112` (default `ChatCompletions`).
- **Request config:** `SamplingConfig` (`types.rs:1169-1213`: `base_url`, `model`,
  `api_backend`, `extra_headers`, `reasoning_effort`, `context_window`, `max_retries`, …);
  sampler-side `SamplerConfig` (`xai-grok-sampler/src/config.rs:38-125`).
- **Client:** `SamplingClient` owns a pooled `reqwest::Client`
  (`xai-grok-sampler/src/client.rs:307-324`, ctor `:510`); auth schemes
  `AuthScheme::{Bearer, XApiKey}` (`config.rs:20`). All HTTP is hand-rolled; `async-openai`
  is used for wire types only.
- **Endpoints:** appended to `base_url`:
  `chat/completions` (`client.rs:958` non-stream, `:1096` stream),
  `responses` (`:1326`/`:1476`), `messages` (`:1683`/`:1809`).
- **Streaming:** SSE via `eventsource-stream`; layer-2 transforms in
  `xai-grok-sampler/src/stream/{chat_completions,responses,messages}.rs`; normalized
  `SamplingEvent::ChannelToken` with `SamplingChannel { Text, Reasoning, Narration }`
  (`events.rs:15-21,50`). Chat path already maps `delta.reasoning_content` → Reasoning
  (`stream/chat_completions.rs:162-180`) and accumulates `tool_calls`.
- **Messages & reasoning:** `ChatRequestMessage.reasoning_content: Option<String>`
  (`types.rs:246`), `ChatResponseMessage.reasoning_content` (`:491`), delta
  (`ChatChunkDelta.reasoning_content` `:628`). Internal `ConversationItem::{Assistant,
  Reasoning(rs::ReasoningItem), ToolResult, BackendToolCall}` (`conversation.rs:69-87`);
  chat/messages backends fold sibling `Reasoning` into the following assistant message
  (`conversation/chat_completions.rs:184-218`). Invalid tool args are sanitized to `{}`
  before replay (`conversation.rs:39-52`). **This is exactly where DeepSeek
  `reasoning_content` round-trip belongs.**
- **Retries:** `xai-grok-sampler/src/retry.rs` (`classify_error` `:95-163`, backoff/jitter
  `:42-71`), loop in `actor/request_task.rs:352-503`. Errors in
  `xai-grok-sampling-types/src/error.rs:133-191`.
- **Usage/cache:** `Usage` (`types.rs:531-564`, incl. `prompt_tokens_details.cached_tokens`,
  `completion_tokens_details.reasoning_tokens`), normalized `TokenUsage`
  (`conversation.rs:774-788`), ledger `xai-chat-state/src/usage.rs:31-153`.
- **Defaults:** `xai-grok-models/default_models.json` (default `grok-4.6`,
  `api_backend: responses`); resolution precedence in
  `xai-grok-shell/src/agent/remote_config/resolution.rs:136`.

### 1.4 Agent loop

- Session command loop: `xai-grok-shell/src/session/acp_session_impl/run_loop.rs:354`
  `run_session` (select at `:522`; `SessionCommand::Prompt` `:725`).
- Turn entry: `turn.rs:451` `handle_prompt` → `:503` `handle_turn_input_inner`.
- **Core loop:** `turn.rs:2564` `process_conversation_turn` → `:2615` `process_conversation_turn_inner`,
  loop at `:2747`. Iteration: build tool specs `:2925`, build request `:2966`, sample
  `:3050` → `run_turn_via_sampler` (`sampler_turn.rs:1716`); if no tool calls → `Completed`
  `:3716`; else execute `:3818` → `execute_tool_calls` (`tool_calls.rs:335`) and continue.
- Turn bound: `max_turns: Option<usize>` (`handle.rs:63`), default `None` (unbounded),
  CLI `--max-turns` (`app/cli.rs:670`). Auto-compaction inline `:2869`, preflight `:3865`.

### 1.5 Tool dispatch

- `trait Tool` (`xai-tool-runtime/src/tool.rs:37`) + type-erased `ToolDyn` (`:322`).
- Registry/builder: `xai-grok-tools/src/registry/types.rs:511`; schema gen `:2160`;
  built-in registration list `:699-796` (Bash, ReadFile, SearchReplace, ListDir, Grep,
  TodoWrite, Task, SendSubagentMessage, WebSearch, WebFetch, Lsp, ImageGen/Edit/Video,
  Enter/ExitPlanMode, AskUserQuestion, Monitor, Scheduler, codex & opencode ports, memory,
  …). `ToolBridge` `bridge.rs:69`.
- Session dispatch: `tool_calls.rs:335` → `:484` batch → `:1445` `prepare_tool_call`;
  `tool_dispatch.rs:45` `dispatch_observed` → `WorkspaceOps::call_tool_with_context`.
- Hooks: `PreToolUse`/`PostToolUse` gate in `tool_calls.rs`.

### 1.6 Permission system

- Modes: `DefaultPermissionMode { Default, AcceptEdits, Plan, Auto, DontAsk, BypassPermissions }`
  (`xai-grok-permission-rules/src/rules.rs:8`); agent config `PermissionMode` (`xai-grok-agent/src/config.rs:921`);
  TUI kinds (`xai-grok-pager/src/app/actions.rs:993`).
- Rules/resolution: `permission-rules/src/{types,rules,resolution,policy}.rs`; managed/enterprise
  policy in `permission-rules/src/managed_policy/*`.
- Manager: `xai-grok-workspace/src/permission/manager/mod.rs:269` `PermissionHandle::request`.
- Dangerous-command detection: `workspace/src/permission/auto_mode/security_findings.rs:11`,
  `permission-rules/src/exec_risk.rs`, `bash_command_splitting.rs`.
- **Read-before-write is NOT runtime-enforced** — only a config-time invariant that a
  Read-capable tool exists (`search_replace/mod.rs:837`, `registry/types.rs:2295`), plus
  hashline anchors. This is a Phase 4 gap.

### 1.7 Context assembly & compaction

- Request build: `xai-chat-state/src/actor/request_builder.rs:23`; pruning `:128-134`.
- Compaction engine: `common/xai-grok-compaction/*` (full-replace code compaction,
  intra/inter), transcript store `xai-compaction-transcript`.
- Host wiring: `xai-grok-shell/src/session/compaction.rs` (`run_compact_inner` `:922`,
  `should_auto_compact` `:2095`, `check_preflight_overflow` `:2208`).
- **Prompt ordering:** system prompt at index 0 (`prompt_build.rs:315`); dynamic
  first-user prefix (OS/shell/cwd/**date**/rules/skills/MCP) at index 1
  (`prompt_build.rs:525-581`, `session/user_message.rs:27-52`). Tool definitions are
  registry insertion order (`registry/types.rs:1469`). Reasoning siblings are kept
  top-level to keep prefix byte-stable for KV cache (`conversation.rs:83-86`).

### 1.8 Session persistence

- Layout `~/.grok/sessions/<encoded-cwd>/<id>/` (`xai-grok-config/src/paths.rs:134`):
  `summary.json`, `updates.jsonl` (source of truth), `chat_history.jsonl` (derived),
  `usage.json`, `plan.json`, checkpoints, etc. (`storage/mod.rs:30-38`).
- Persistence actor `xai-grok-shell/src/session/persistence.rs` (loss/durability contract
  `:1-20`). Typed event log `xai-grok-session-events`. Resume:
  `mvp_agent/session_setup.rs:923` `load_session_inner`. Foreign (Claude/Codex) scan
  `xai-grok-foreign-sessions`. Active registry `~/.grok/active_sessions.json`.

### 1.9 Config system

- Dirs: `$GROK_HOME` else `~/.grok` (`xai-dirs/src/lib.rs:67-95`); system `/etc/grok`.
- Files: `config.toml`, `managed_config.toml`, `requirements.toml`, `sandbox.toml`,
  `trusted_folders.toml` (`xai-grok-config/src/loader.rs:91-110`).
- Precedence (low→high): `/etc/grok/managed` → `$GROK_HOME/managed` → `$GROK_HOME/config` →
  user `requirements` → `/etc/grok/requirements` → macOS MDM (`ai.x.grok`); implementation
  `config_layers.rs:50,127`, booleans `resolved.rs:108-136`, features
  `config-types/src/registry.rs` (`Feature::resolve` `:450`).
- **Remote config is a major egress/control surface**: `xai-grok-cloud-config` fetches
  `GET {proxy}/settings`, managed config `{proxy}/deployment/config`, model catalog
  `{proxy}/models`; caches `settings_cache.json`, `models_cache.json`, signed
  `requirements.toml` (Ed25519). `RemoteSettings` has ~100 server-controlled flags
  (`remote_settings.rs:279+`) including `telemetry_*`, `trace_upload_enabled`,
  `sharing_enabled`, `oauth2_*`, subagent limits, etc.

### 1.10 MCP / plugins / subagents

- MCP transports `Stdio` / `StreamableHttp` / `Sse` (`xai-grok-config/src/mcp_server_config.rs:27-59`);
  connection code `xai-grok-mcp/src/servers.rs:4830` `start_mcp_server`; OAuth loopback
  `xai-grok-mcp/src/oauth.rs:360`. Config discovery merges `config.toml`, project
  `.grok`, plugins, `~/.claude.json`, Cursor `mcp.json` (`mcp_servers.rs:1-7`).
- Plugins: `xai-grok-plugin-marketplace` (official source `xai-org/plugin-marketplace`,
  `lib.rs:26-33`); discovery in `xai-grok-agent/src/plugins/*`; clone cache
  `~/.grok/marketplace-cache/`.
- Hooks: `xai-grok-hooks` (command + HTTPS-only HTTP handlers, `runner/http.rs:85`).
- Subagents: `xai-grok-subagent-resolution`, task tool coordinator
  (`xai-grok-tools/.../task/coordinator/*`), spawn `mvp_agent/subagent_spawn.rs:203`.
  Agent defs discovered from `.grok/agents/`, `.claude/agents/` (`xai-grok-agent/src/discovery.rs`).

---

## 2. Security audit

### 2.1 Policy for the final product

Allowed outbound network:
1. The **configured model provider** (DeepSeek by default; user may point `base_url` at a
   custom OpenAI-compatible host).
2. **User-configured MCP servers** (stdio/local or explicitly configured HTTP).
Everything else must be removed from the shipped binary.

### 2.2 Findings — data egress / telemetry / remote control

| # | Surface | Location | Sends | Destination | Action |
|---|---|---|---|---|---|
| A1 | **Repo/session trace upload** | `crates/codegen/xai-file-utils/src/{queue,gcs,s3,storage_client,upload_config}.rs` | repo patches/dedup blobs, session archives, full prompts, tool definitions, images, unified logs, memory archives, heap dumps, auth diagnostics | `gs://*`, `s3://*`, and `POST {proxy}/v1/storage/{batch_upload,batch_upload_json}` + signed URLs | **DELETE crate** |
| A2 | Upload call sites | `xai-grok-shell/src/upload/*` (4,510 LOC), `session/repo_changes/`, `session/feedback_manager.rs`, `heap_profile/monitor.rs`, `extensions/{share,feedback_trace}.rs` | same as A1 | same | **DELETE dirs + calls** |
| A3 | Memory archive upload | `xai-grok-memory/src/archive.rs`, `xai-grok-shell/src/upload/memory.rs` | agent memory archive | GCS at session finalize | **DELETE** |
| A4 | Session share | `xai-grok-shell/src/extensions/share.rs`, `xai-grok-pager/src/share_cmd.rs` | full exported session → signed URL, public link | proxy + cloud storage | **DELETE** |
| A5 | Feedback + signals | `xai-grok-feedback/*`, `feedback_manager.rs` | ratings, drafts, screenshots, LOC attribution, turn deltas | `{GROK_FEEDBACK_BASE_URL}` (proxy) | **DELETE** |
| B1 | Product telemetry events | `xai-grok-telemetry/src/client.rs` | product/usage events, user/team/session IDs | `{GROK_TELEMETRY_EVENTS_URL}` | **DELETE crate** |
| B2 | Mixpanel | `xai-mixpanel/src/lib.rs` (`API_BASE_URL` `:32`) | track/engage analytics | `https://api.mixpanel.com` | **DELETE crate** |
| B3 | Sentry crash reporting | `xai-grok-telemetry/src/sentry.rs:39-58` | exceptions, stacktraces, tags | `SENTRY_DSN` (baked/env) | **DELETE** |
| B4 | Crash handler | `xai-crash-handler/*` | local crash formatting + Sentry hook | Sentry / local | **DELETE or strip Sentry** |
| B5 | External OTLP | `xai-grok-otel/*`, `xai-grok-telemetry/src/config.rs:141-173` | logs/metrics/traces, optionally prompts/tool content | `otel_endpoint` / `OTEL_EXPORTER_OTLP_ENDPOINT` | **DELETE crate** |
| C1 | Remote settings | `xai-grok-cloud-config/src/settings_fetch.rs:28-87` | client version, user id, token; receives ~100 flags | `GET {proxy}/settings` | **DELETE crate** |
| C2 | Managed config sync | `xai-grok-cloud-config/src/managed_config/supervisor.rs:287-348` | identity, pulls signed `requirements.toml`/`managed_config.toml` | `GET {proxy}/deployment/config` | **DELETE** |
| C3 | Models catalog | `xai-grok-shell/src/remote/client.rs`, `model_source/oai.rs` | identity; receives model list | `{proxy}/models`, `{xai_api}/models` | **Replace with static/local config** |
| C4 | Remote announcements | `xai-grok-announcements/*` | — | via settings | **DELETE** |
| C5 | Remote-control relay / gateway | `xai-grok-env/src/lib.rs:20-26`, `shell/src/remote/client.rs:10` | session/control messages | `wss://code.grok.com/ws/code-agent`, `wss://grok.com/ws/gw/`, `https://code.grok.com` | **DELETE relay modes** |
| D1 | Auto-update | `xai-grok-update/src/{auto_update,version}.rs` (`CLI_BASE_URL_PRIMARY` `version.rs:17`, fallback `:20-21`) | version check + binary download | `https://x.ai/cli`, `https://storage.googleapis.com/grok-build-public-artifacts/cli` | **DELETE crate; replace w/ no auto-update or user-owned releases** |
| D2 | Subagent bundle download | `xai-grok-bundle/*` | downloads xAI agent/persona bundle | via proxy/asset server | **DELETE or vendor static bundle** |
| E1 | Auth/login | `xai-grok-login/*` (28k LOC) | credentials/tokens | `https://auth.x.ai`, `https://accounts.x.ai`; `{proxy}/user`; `{xai_api}/api-key` | **DELETE; replace with DeepSeek API-key auth** |
| E2 | xAI API key env | `xai-grok-env/src/registry.rs:1-11` | `XAI_API_KEY`, `GROK_AUTH`, `GROK_DEPLOYMENT_KEY`, … | → provider | **Replace with `DEEPSEEK_API_KEY`** |
| F1 | Voice STT | `xai-grok-voice/src/config.rs:33,121` | microphone audio | `wss://api.x.ai/v1/stt` | **DELETE feature** |
| F2 | Imagine image/video | `xai-grok-tools/.../image_gen`, `video_gen` (`XAI_IMAGINE_MODEL`, `XAI_VIDEO_MODEL`) | prompts/images; S3 output | xAI Imagine + S3 | **DELETE tools** |
| F3 | Hosted web search | `xai-grok-tools/.../web_search/client.rs:198` | search queries | `{base_url}/responses` (xAI) | **Remove or replace with user-configured search** |
| F4 | Web fetch | `xai-grok-tools/.../web_fetch/mod.rs` | arbitrary URL fetch | any host | **Keep but gate/ask** (user-facing tool) |
| F5 | Plugin marketplace git | `xai-grok-plugin-marketplace/src/lib.rs:26-33`, `git.rs` | git clone | `github.com/xai-org/plugin-marketplace.git` | **Remove official source; allow user sources** |
| G1 | Egress proxy | `xai-grok-egress-proxy/*` | *local* HTTP/CONNECT proxy for sandbox site policy | loopback only | **Keep (local), audit for any telemetry** |
| G2 | Diag server | `xai-grok-diag-server/*` | local `/ready`,`/statusz`,`/logs` | loopback only | **Keep (local)** |
| G3 | Workspace daemon/client | `xai-grok-workspace-daemon`, `xai-grok-workspace-client` | local/remote workspace RPC | loopback / configured | **Audit; likely remove cloud mode** |

`prod/mc/cli-chat-proxy-types/` contains the wire types for the above (`feedback_types`,
`storage_types`, `client_metrics_types`, `deployment_config_types`, `subagent_bundle`).
Delete what depends on removed features.

### 2.3 Upload infrastructure deep-dive (the "known" flagged code)

- `xai-file-utils/src/lib.rs:8`: *"Local data collection: upload queueing and S3-compatible
  blob storage."* `upload_config.rs:120` `DEDUP_GCS_PREFIX = "repo_changes_dedup"`;
  `PatchReference`/`FileReference`/`DedupMetadata` describe archived repo content.
- `queue.rs:1-9`: *"Spill-to-disk upload queue for cloud storage trace artifacts…
  prevents data loss when uploads fail transiently."* — i.e. it retries hard.
- `storage_client.rs:1-5`: routes through `cli-chat-proxy` with the user's grok.com token;
  endpoints `/storage/limits`, `/storage/exists`, `/storage/batch_exists`,
  `/storage/batch_upload`, `/storage/batch_upload_json`.
- `xai-grok-shell/src/session/repo_changes/mod.rs` re-exports the archive schema
  (`ARCHIVE_SCHEMA_VERSION`, dedup prefixes).
- Gate: `resolve_telemetry_mode` / `resolve_trace_upload` in
  `shell/src/agent/config.rs:1850-1925` and `TraceUploadEndpoints` (`:150-245`); a
  deployment key or any xAI auth token yields an `UploadMethod::Proxy`. Privacy vetoes only
  ZDR / data-collection-disabled accounts (`is_trace_upload_blocked_for` `:239-245`).

**Conclusion:** this is genuine exfiltration-capable infrastructure. Delete the entire
dependency edge, do not merely flip flags.

### 2.4 Network destination inventory (post-removal target)

Keep:
- `<configured provider base_url>` (default `https://api.deepseek.com`).
- User-configured MCP servers (stdio; and explicitly user-set HTTP URLs).
- Nothing else by default.

A CI check will enforce this (Phase 1).

---

## 3. xAI / Grok coupling inventory

| Category | Examples | Notes |
|---|---|---|
| Binary/package | package `xai-grok-pager-bin`, bin `xai-grok-pager`, upstream alias `grok` | rename to `<PROJECT_NAME>` |
| Config dir | `~/.grok`, `$GROK_HOME`, `/etc/grok` (`xai-dirs`, `xai-grok-config/paths.rs`) | rename to `~/.<project>` |
| Env vars | ~623 distinct `GROK_*` (e.g. `GROK_HOME`, `GROK_AUTH`, `GROK_MODELS_BASE_URL`, `GROK_TELEMETRY_*`) | rename prefix |
| Endpoints | `api.x.ai/v1`, `cli-chat-proxy.grok.com/v1`, `assets.grok.com`, `code.grok.com`, `grok.com/ws/gw`, `auth.x.ai`, `accounts.x.ai` | remove/replace |
| Model catalog | `grok-4.6`, `grok-4.5`, `grok-imagine-*`, `grok-3*`, `grok-4.7` | replace with DeepSeek |
| Auth | OAuth2/OIDC/device flow, `auth.json`, `X-XAI-Token-Auth`, team/ZDR flags | replace with API key |
| ACP extensions | `x.ai/*` methods (billing, cloud_workspace, feedback, rewind, share, …) | rename namespace to `<project>/*` |
| Header names | `x-grok-conv-id`, `x-grok-session-id`, `x-grok-agent-id`, `x-grok-model-override`, … (`sampler/client.rs:68-90`) | rename/strip |
| Features | voice STT, Imagine image/video, `/gboom` easter egg, SuperGrok upsell (`grok.com/supergrok`), xAI plugin marketplace, remote relay | delete |
| Prompt/brand strings | "Grok Build", "SpaceXAI", system prompt labels, README/docs/media.x.ai assets | rename/rewrite |
| Compat paths | `.claude/`, `.claude.json`, Cursor `.cursor/` config/agents/hooks | keep (interop) but audit |

`README.md`, `crates/codegen/xai-grok-pager/docs/user-guide/*`, and the media/logo URLs in
README all need rewriting.

---

## 4. Licensing

- **First-party:** Apache-2.0. Root `LICENSE` = *"Copyright 2023-2026 SpaceXAI"*.
  106 crates declare `license = "Apache-2.0"`; one declares MIT (fine, permissive).
- **Attribution obligations (Apache-2.0 §4):** keep `LICENSE` and the NOTICE/attribution
  chain. There is **no root `NOTICE` file**; attribution currently lives in
  `THIRD-PARTY-NOTICES` (763 KB), `third_party/NOTICE`, and in-crate notices.
  **Action:** preserve all three; add `<PROJECT_NAME>` change notices for modified
  first-party files where required, and keep the upstream Apache-2.0 copyright.
- **Vendored `third_party/`:** mermaid-to-svg (MIT), dagre_rust/graphlib_rust/ordered_hashmap
  (Apache-2.0). `third_party/NOTICE` also references `nfsserve` (BSD-3) and `fuser` (MIT),
  but those directories are **not present** in this tree — verify/clean the notice.
- **Ported code:** `xai-grok-tools/src/implementations/codex/*` (openai/codex, Apache-2.0)
  and `.../opencode/*` (sst/opencode, MIT), with an Apache §4(b) change notice in
  `crates/codegen/xai-grok-tools/THIRD_PARTY_NOTICES.md`. **Keep.**
- **Copyleft in transitive deps:** `THIRD-PARTY-NOTICES` records `MPL-2.0` (46), `GPL-2.0` (4)
  and `LGPL` (1) entries — notably **libgit2 GPL-2.0 + linking exception**. These are
  dependency-level, not first-party, and are compatible with shipping an Apache-2.0
  aggregate, but the notices must ship. **No action beyond preservation**, flag if we
  replace `git2`/`gix`.
- **Risk:** the upstream repo has *no* NOTICE and its `CONTRIBUTING.md` says external
  contributions are not accepted. We are relicensing the aggregate under our own project
  while preserving Apache-2.0 attribution; legal review recommended before public release.

---

## 5. Keep / rip-out matrix

**Keep and build on**
- Agent loop, session persistence/resume, tool runtime + registry, permission engine,
  compaction, TUI/ratatui stack, config layering primitives, MCP client, hooks, plugin
  framework (with user-owned sources), subagent resolution, checkpoints/rewind, egress
  proxy (local), diag server (local), token estimation, markdown/mermaid stack.

**Keep but rework**
- `xai-grok-sampler` / `xai-grok-sampling-types` — keep the `ChatCompletions` path, harden
  reasoning round-trip and stream repair; drop xAI headers and Responses/Messages backends
  if unused (they can stay for custom providers, but de-emphasize).
- Provider/model config + auth — replace xAI OAuth with a DeepSeek API-key credential
  provider and custom-`base_url` support.
- Config dirs/env — rename and simplify; drop requirements/MDM/remote layers initially.
- `web_fetch` — keep, default to ask; add an allow/deny policy.
- `web_search` — replace with user-configured provider or remove.

**Delete outright**
- `xai-file-utils` (upload queue, GCS/S3/storage client).
- `xai-grok-telemetry`, `xai-mixpanel`, `xai-grok-otel`, `xai-crash-handler` (Sentry path),
  `xai-grok-feedback`.
- `xai-grok-update` auto-update.
- `xai-grok-login`, `xai-grok-cloud-config`, `xai-grok-announcements`.
- `xai-grok-voice`.
- `xai-grok-shell/src/upload/*`, `session/repo_changes/`, `extensions/share.rs`,
  `heap_profile/*`, `feedback_manager.rs` analytics paths, `share_cmd.rs`.
- Imagine image/video generation tools.
- Remote relay/gateway/leader-cloud modes; `xai-grok-bundle` network download; the
  `Share` command; xAI marketplace default source.
- All `x.ai/*` extension methods not needed by the TUI.

**Needs verification before deletion (may have local value)**
- `xai-grok-codebase-graph` (8.7k LOC) — no outbound calls found; keep.
- `xai-grok-workspace-daemon`/`-client` (3.0k/0.9k) — mostly local; audit cloud RPC.
- `xai-grok-egress-proxy` (local) — keep; strip any metrics export.
- `xai-grok-diag-server` (local) — keep.

---

## 6. Phased implementation plan

Guiding rules: one logical change per commit; `cargo fmt`, `cargo clippy`, `cargo test`
before declaring a phase done; stop and ask on large architectural decisions; verify
DeepSeek API behavior against official docs and live/mocked responses.

### Phase 1 — Cleanup and rebrand

**1a. Delete data egress (commit-per-crate where possible)**
1. Delete `xai-file-utils`; remove `upload/*`, `session/repo_changes/`, heap-profile upload,
   feedback analytics, `extensions/share.rs`, `share_cmd.rs`, `xai-grok-memory` archive
   upload.
2. Delete `xai-grok-telemetry`, `xai-mixpanel`, `xai-grok-otel`; replace all
   `xai_grok_telemetry::*` call sites with no-op/local `tracing` (or delete the calls).
   Remove Sentry from `xai-crash-handler` (keep local crash formatting if desired).
3. Delete `xai-grok-feedback` and feedback UI/commands.
4. Delete `xai-grok-update`; remove `update` command/auto-update; stub "manual update".
5. Delete `xai-grok-cloud-config`, `xai-grok-announcements`, remote settings consumption;
   replace `RemoteSettings` with a minimal static defaults struct where still referenced,
   or remove the field. (Large call-site surface — do in slices.)
6. Delete `xai-grok-login`, `xai-grok-cloud-config` auth enrichment, xAI credential env vars;
   add a `DeepSeek`/generic API-key credential provider behind the existing
   `AuthCredentialProvider`/`HeaderInjector` seam.
7. Delete `xai-grok-voice`, image/video tools, remote relay/leader-cloud, xAI marketplace
   source, `xai-grok-bundle` download.
8. Prune `prod/mc/cli-chat-proxy-types` to only types still used.

**1b. Rebrand**
- Rename package/binary/crates from `xai-grok-*` → `<project>-*` (mechanical, optional
  initially — crate renames create large churn; at minimum rename the binary, config dir,
  env prefix, and user-visible strings). *Decision needed: rename all crates or only the
  binary/config/strings?* (see §7).
- `~/.grok` → `~/.<project>`; `GROK_*` → `<PROJECT>_*`; `x.ai/*` → `<project>/*`.
- Rewrite README, user guide, logos, install URLs, `SOURCE_REV` story.

**1c. CI guard**
- Add a test/CI job that scans non-test Rust for outbound hosts (`https?://`, `wss?://`,
  `reqwest::Client`, `ClientBuilder`) and fails unless the host is in an allowlist
  (provider + MCP + loopback + doc links). Prefer a source-scan test crate so it runs in
  `cargo test`.

**Estimated diff:** net **−90k to −130k LOC** deleted (telemetry 24.9k, login 28.1k,
file-utils 13.9k, update 7.5k, cloud-config 5.6k, voice 5.6k, feedback 2.6k, otel 1.4k,
crash 1.7k, upload dir 4.5k, plus call-site edits across ~369 importing files). Rebrand
touches ~1,372 files across several mechanical commits. Expect ~30–50 commits.

**Risks:** telemetry is imported almost everywhere; deleting the crate forces call-site
edits (mechanical but broad). `RemoteSettings` is read in many places; removing it is the
single riskiest slice — do it incrementally with a compatibility shim. Build times are
high; use `cargo check -p <crate>`.

### Phase 2 — DeepSeek provider adapter

0. **Verify model names/params against official DeepSeek docs** (do not rely on memory;
   user supplied `deepseek-v4-pro` / `deepseek-v4-flash`, unconfirmed).
1. Add DeepSeek model entries (`base_url`, `api_backend = ChatCompletions`,
   `AuthScheme::Bearer`, context window, max tokens, reasoning efforts). Default model =
   DeepSeek; keep custom `base_url` support.
2. Credential path: `DEEPSEEK_API_KEY` env (+ config file), no OAuth.
3. **`reasoning_content` round-trip hardening** (the core requirement):
   - Capture `reasoning_content` from every assistant message (stream + non-stream).
   - Replay it on every assistant message in a tool-call chain (chat backend already folds
     sibling `Reasoning`; verify and make it unconditional).
   - **Final-pass sanitizer**: before send, any assistant message in a tool-call chain with
     empty/absent `reasoning_content` gets a non-empty placeholder to avoid the API's
     "reasoning_content must be passed back" 400.
   - Tests for: chain with missing reasoning on intermediate messages, multi-step chains,
     provider that omits reasoning in some replies.
4. **Thinking control**: map on/off + `reasoning_effort {none,low,high,max}` to config and a
   runtime toggle; optional auto-router (Flash/no-reasoning for trivial, Pro/high for
   complex) behind a flag.
5. **Stream separation**: reasoning channel already exists (`SamplingChannel::Reasoning`).
   Ensure DeepSeek deltas route there and answer/tool calls stay separate.
6. **Cache-friendly prompting**: system prompt + tool definitions byte-identical & stable
   order (already largely true); ensure no date/dynamic state in prefix — move dynamic
   state to the tail. Add a test asserting prefix stability across turns.
7. **Quirk normalization**:
   - Stray thinking/signature events when thinking is off → drop.
   - Typographic Unicode quotes (U+201C/201D/etc.) in tool-call JSON → normalize before
     parse.
   - Partial/malformed tool-call arguments in streams → repair; retry once.
8. **Retries/backoff** for 429/5xx; clear errors; no silent failures (reuse `retry.rs`,
   adjust classification for DeepSeek error envelope).
9. **Usage parsing** incl. cache hit/miss tokens (`prompt_cache_hit_tokens` /
   `prompt_cache_miss_tokens` as confirmed by docs).
10. **Mock-server test suite** (`wiremock`/`mockito`): streaming, tool-call chains, missing
    `reasoning_content`, malformed JSON args, 429/backoff.

**Estimated diff:** +2k to +4k LOC (adapter + tests), minus dead xAI backend bits.
**Risks:** exact DeepSeek schema (reasoning param name, cache fields, tool-call streaming
shape) must be confirmed from docs; Unicode-quote repair needs a strict "only fix, never
corrupt" approach.

### Phase 3 — DeepSeek-native TUI UX

- Collapsible reasoning block per turn + keybinding.
- Status bar: model, thinking level, tokens in/out, cache hit rate, running cost
  (configurable price table), context-window usage.
- Hotkeys + slash commands to switch model / reasoning effort mid-session.
- Per-turn and per-session cost & cache breakdown.

**Estimated diff:** +1.5k to +3k LOC (mostly `xai-grok-pager`, `xai-grok-pager-render`,
`xai-grok-status-line`).
**Risks:** TUI state plumbing touches many views; keep changes additive.

### Phase 4 — Competitive agent features (verify-then-build)

Already present (verify, polish only): plan mode + approval, permission modes, LLM
compaction, session resume, subagents, MCP, AGENTS.md, checkpoints/undo, headless mode,
custom OpenAI-compatible endpoints.

Likely gaps to build/fix:
- **Read-before-write enforcement** (currently config-time only — add a runtime gate).
- **Dangerous-command detection** — exists; harden and surface.
- **Subagents on Flash** for cheap research — wire model selection.
- **Headless mode** is xAI-relay-flavored; provide a clean non-interactive CLI mode.
- **Custom endpoints** — ensure a first-class "OpenAI-compatible provider" config.

**Estimated diff:** +2k to +6k LOC depending on gaps confirmed.
**Risks:** scope creep; verify before building to avoid duplicating existing features.

### Phase 5 — Quality bar

- Benchmark small real coding tasks; Pro vs Flash; report cost/task.
- Docs: README, install, config reference, security/privacy statement listing every
  network destination (should be: provider + user MCP only).
- Release pipeline for Linux/macOS/Windows binaries (GitHub Actions), checksums, signing.

**Estimated diff:** +1k to +2k LOC of scripts/docs + CI.
**Risks:** long build matrix; signing/notarization logistics.

---

## 7. Resolved decisions

Canonical, evolving record lives in [`DECISIONS.md`](DECISIONS.md). Summary of what was
resolved with the stakeholder on 2026-10-02:

1. **Project name = `deepseek-build`.** Binary `deepseek-build`; config dir
   `~/.deepseek-build`; env prefix `DEEPSEEK_BUILD_*`; ACP extension namespace
   `deepseek-build/*`.
2. **Phase 1 rename scope = user-facing surface only.** Rename the binary, config dir, env
   prefix, and all user-visible strings. Internal `xai-grok-*` crate/package names are left
   as-is for a later, isolated mechanical pass.
3. **Auto-update = removed entirely.** No update-check or binary-download egress.
4. **Web tools = drop `web_search`; keep `web_fetch` ask-gated.**
5. **Provider backends = `ChatCompletions` only.** Remove `Responses` and `Messages`.
6. **Delete, never disable,** all exfiltration/telemetry/remote-control code.
7. **Keep `LICENSE` and all notices; do not relicense upstream files;** add a `NOTICE`
   attribution to `xai-org/grok-build` (Apache-2.0).
8. **Toolchain = Rust 1.94.0 via rustup; do not bump.**
9. **Phase 5 skipped** — this is a personal tool.

Open questions (legal sign-off; toolchain install feasibility) are tracked in
`DECISIONS.md`.
