---
last_mapped_commit: 221a1a591ef3af5f90d59d73ae7899d110f28b37
last_mapped_at: 2026-11-21
---
# Codebase Structure

**Analysis Date:** 2026-11-21

## Directory Layout

```
deepseek-build/
├── Cargo.toml              # Workspace root (95 members). Prefer per-crate manifests.
├── Cargo.lock              # Lockfile. Do not hand-edit.
├── rust-toolchain.toml     # Pins Rust 1.94.0. Do not bump.
├── rustfmt.toml            # Format settings.
├── clippy.toml             # Bans: dunce::canonicalize, extra-CA HTTP, ProcessScope, xai-dirs.
├── deny.toml               # cargo-deny licenses/bans/sources.
├── install.sh              # POSIX installer (GitHub release assets only).
├── LICENSE, NOTICE, THIRD-PARTY-NOTICES
├── AGENTS.md               # Resume handoff and crate map. Trust code over this file.
├── DECISIONS.md            # Locked decisions. Wins on conflict with other docs.
├── PLAN.md                 # Audit and phased plan (older than the tree).
├── README.md               # Public description.
├── KNOWN-ISSUES.md         # Known failures and unexercised surfaces.
├── TESTING.md              # How to test. Not a generated map.
├── bin/protoc              # Vendored protoc used by proto builds.
├── crates/
│   ├── codegen/            # Product crates (TUI, agent, sampler, tools, …).
│   ├── common/             # Shared runtime blocks (tools, compaction, circuit breaker).
│   └── build/              # Protobuf build helper only.
├── prod/mc/model-api-types/  # Shared model API types package.
├── third_party/            # Vendored mermaid/layout crates. Do not "upgrade" in place.
├── scripts/                # Egress, install, eval, template encryption.
├── docs/                   # Audits and tuning notes (not the user guide).
├── docker/                 # Container build files. Image has not been the product entry.
├── .github/workflows/      # release.yml (tag v*).
└── .cargo/                 # Build settings (incremental off). Not a secret store.
```

Workspace members are listed in the root `Cargo.toml` `[workspace] members` array. Counted from that list: 81-style product crates under `crates/codegen/` (including `ptyctl` and `ptyctl-cli`), 8 under `crates/common/`, 1 under `crates/build/`, `prod/mc/model-api-types`, and 4 under `third_party/` — 95 members total.

## Directory Purposes

**`crates/codegen/`:**

- Purpose: Everything that ships in the agent except the small shared cores.
- Contains: one directory per crate, each with its own `Cargo.toml` and `src/`.
- Key files: `xai-grok-pager-bin/src/main.rs` (binary), `xai-grok-pager/src/lib.rs` (TUI), `xai-grok-shell/src/lib.rs` (agent), `xai-grok-sampler/src/lib.rs` (HTTP actor).

**`crates/codegen/xai-grok-pager-bin/`:**

- Purpose: Composition root. Artifact name `deepseek-build`, package name `xai-grok-pager-bin`.
- Contains: `src/main.rs`, `src/agent_command.rs`, `build.rs`, `tests/` (including branding surface).
- Key files: `Cargo.toml` `[[bin]] name = "deepseek-build"`.

**`crates/codegen/xai-grok-pager/`:**

- Purpose: Fullscreen TUI, headless driver, CLI subcommand implementations, scrollback.
- Contains: `src/app/` (event loop, views, effects), `src/acp/` (spawn and leader bridge), `src/headless/`, `src/scrollback/`, `src/views/`, `src/slash/`, `docs/user-guide/`.
- Key files: `src/app/mod.rs` (`run`), `src/app/event_loop.rs`, `src/headless.rs`, `src/acp/spawn.rs`, `docs/user-guide/26-config-reference.md` (hand-edited; do not regenerate).

**`crates/codegen/xai-grok-pager-minimal/` and `xai-grok-pager-render/`:**

- Purpose: Scrollback-native minimal mode (fn-pointer seam) and render primitives (theme, terminal, syntax).
- Contains: minimal-mode install hooks; render modules re-exported from `xai-grok-pager/src/lib.rs`.
- Key files: minimal install is called from `pager-bin/src/main.rs`, not from the pager library.

**`crates/codegen/xai-grok-shell/`:**

- Purpose: ACP agent, session/turn loop, tool execution wiring, leader, config facade.
- Contains: `src/agent/` (stdio, leader, `mvp_agent`), `src/session/` (actor, persistence, compaction glue), `src/leader/`, `src/tools/`, `src/sampling/`, `src/config/`.
- Key files: `src/agent/app.rs`, `src/agent/mvp_agent/acp_agent.rs`, `src/session/acp_session_impl/turn.rs`, `src/session/acp_session_impl/sampler_turn.rs`, `src/session/acp_session_impl/tool_calls.rs`, `src/session/acp_session_impl/spawn.rs`, `src/session/persistence.rs`.
- Contributor note: `crates/codegen/xai-grok-shell/AGENTS.md` — config reference stays hand-edited.

**`crates/codegen/xai-grok-shell-base/`, `xai-grok-shell-terminal/`, `xai-grok-shell-session-support/`:**

- Purpose: Pieces split out of the shell so the shell crate can re-export them without owning every type.
- Contains: CPU profile / env (`shell-base`), PTY session (`shell-terminal`), session support types.

**`crates/codegen/xai-chat-state/`:**

- Purpose: Conversation actor.
- Contains: `src/actor/`, `src/handle.rs`, `src/commands.rs`, `src/persistence.rs`, `src/usage.rs`.
- Key files: `src/lib.rs` (actor diagram and re-exports).

**`crates/codegen/xai-grok-sampler/` and `xai-grok-sampling-types/`:**

- Purpose: Chat Completions client and wire types.
- Contains: `src/client.rs`, `src/actor/`, `src/stream/chat_completions.rs`, `src/retry.rs`, `tests/` (wiremock). Types crate holds `ApiBackend`, requests, `TokenUsage`.
- Key files: `xai-grok-sampling-types/src/types.rs` (`ApiBackend::ChatCompletions` only).

**`crates/codegen/xai-grok-config/`, `xai-grok-config-types/`, `xai-grok-env/`:**

- Purpose: Layered config, endpoint presets, env overlay.
- Contains: `src/endpoints.rs`, `src/config_layers.rs`, `src/loader.rs`, `src/mcp_servers.rs`, `src/signed_policy/`.
- Key files: `endpoints.rs` constant `XAI_API_BASE_URL_DEFAULT`.

**`crates/codegen/xai-grok-models/`:**

- Purpose: Embedded model catalog.
- Contains: `src/lib.rs`, `default_models.json` (loaded with `include_str!`).
- Key files: edit `default_models.json` to change baked-in IDs; runtime resolution is CLI, then env, then config, then remote settings, then these defaults (`src/lib.rs`).

**`crates/codegen/xai-grok-agent/`:**

- Purpose: Agent definitions, discovery, system prompt.
- Contains: `src/prompt/`, `src/discovery.rs`, `templates/prompt.md`, `templates/subagent_prompt.md`, `templates/apply_patch_prompt.md`.
- Key files: after template edits, `scripts/encrypt_templates.py` regenerates `src/prompt/prompt_encrypted.rs`.

**`crates/codegen/xai-grok-tools/` and `xai-grok-tools-api/`:**

- Purpose: Built-in tools and their protobuf API.
- Contains: `src/implementations/` (`grok_build/`, `read_file/`, `search_tool/`, `lsp/`, `memory/`, `skills/`), `src/bridge.rs`, `src/registry/`. Proto: `xai-grok-tools-api/proto/grok-tools.proto`.

**`crates/codegen/xai-grok-workspace/`, `xai-grok-workspace-client/`, `xai-grok-workspace-daemon/`, `xai-grok-workspace-types/`:**

- Purpose: Filesystem and VCS operations, folder trust, permission manager, optional workspace daemon.
- Contains: `src/permission/` (manager, grants, sandbox gate), `src/file_system/`, `src/worktree/`, `src/folder_trust.rs`.
- Key files: `src/permission/manager/mod.rs` (`spawn_permission_manager`).

**`crates/codegen/xai-grok-permission-rules/`:**

- Purpose: Pure permission policy (no actor).
- Contains: `src/rules.rs` (`DefaultPermissionMode`), `src/exec_risk.rs`, `src/resolution.rs`, `src/managed_policy/`.

**`crates/codegen/xai-grok-mcp/`:**

- Purpose: MCP client (stdio and HTTP), OAuth, elicitation.
- Contains: `src/mcp_http_client.rs`, `src/servers.rs`, `src/oauth.rs`, `src/wire.rs`.

**`crates/codegen/xai-grok-sandbox/`, `xai-grok-extra-ca/`, `xai-dirs/`, `xai-grok-brand/`:**

- Purpose: OS sandbox, TLS client policy, home/path resolution, user-facing brand strings.
- Key files: `xai-grok-brand/src/lib.rs`.

**`crates/codegen/xai-grok-login/` and `xai-grok-auth/`:**

- Purpose: Credential plumbing. Live path is the `DEEPSEEK_API_KEY` env var (`xai-grok-login/src/auth_method.rs`). xAI OIDC login is deleted; do not restore it.

**`crates/common/`:**

- Purpose: Crates shared below the product layer.
- Contains:
  - `xai-tool-runtime/` — `Tool` trait and dispatch (`src/tool.rs`, `src/dispatch.rs`).
  - `xai-tool-protocol/` — IDs, capabilities, generated stubs (`generated/`).
  - `xai-tool-types/` — shared tool value types.
  - `xai-grok-compaction/` — compaction engine (`src/steps/`, `src/history/`, `src/templates/`).
  - `xai-circuit-breaker/`, `xai-interjection-core/`, `xai-message-delivery-core/`, `xai-test-utils/`.

**`crates/build/xai-proto-build/`:**

- Purpose: Protobuf codegen helper. Not a runtime crate.
- Contains: `src/`, `test_data/*.proto`.

**`prod/mc/model-api-types/`:**

- Purpose: Shared model API types (renamed from `cli-chat-proxy-types`). Import this instead of duplicating wire enums.

**`third_party/`:**

- Purpose: Vendored layout/render crates: `dagre_rust`, `graphlib_rust`, `mermaid-to-svg`, `ordered_hashmap`.
- Generated: no. Committed: yes. Do not reformat or rewrite as part of a product change.

**`scripts/`:**

- Purpose: Gates and harnesses, not library code.
- Key files: `check-egress.sh`, `egress-check.sh`, `key-leak-check.sh`, `disk-guard.sh`, `encrypt_templates.py`, `test-install.sh`, `smoke-test.sh`, `sandbox-run.sh`, `evals/` (21 auto-checked tasks).

**`docs/`:**

- Purpose: Internal audits (`LICENSE-AUDIT.md`, `INTERNAL-GROK-ENV-NAMES.md`, `TUNING.md`). The user guide lives under the pager crate, not here.

**`docker/`:**

- Purpose: `Dockerfile` and `Dockerfile.localtest`. Not the development entrypoint.

**`.github/workflows/`:**

- Purpose: `release.yml` — tag `v*` builds four native targets and attaches tarballs plus `install.sh`.

## Key File Locations

**Entry Points:**

- `crates/codegen/xai-grok-pager-bin/src/main.rs`: process entry, subcommand match, headless vs TUI.
- `crates/codegen/xai-grok-pager/src/app/mod.rs`: `run` — interactive startup.
- `crates/codegen/xai-grok-pager/src/app/event_loop.rs`: TUI `select!` loop.
- `crates/codegen/xai-grok-pager/src/headless.rs`: `run_single_turn`.
- `crates/codegen/xai-grok-pager/src/acp/spawn.rs`: `spawn_grok_shell` (in-process agent thread).
- `crates/codegen/xai-grok-shell/src/agent/app.rs`: `run_stdio_agent`, `run_leader`.
- `crates/codegen/xai-grok-shell/src/agent/mvp_agent/acp_agent.rs`: ACP `prompt`.

**Configuration:**

- `crates/codegen/xai-grok-config/src/config_layers.rs`: layer precedence.
- `crates/codegen/xai-grok-config/src/endpoints.rs`: provider base URL.
- `crates/codegen/xai-grok-config/src/loader.rs`: disk load.
- `crates/codegen/xai-grok-brand/src/lib.rs`: `~/.deepseek-build`, `DEEPSEEK_BUILD_*`.
- `crates/codegen/xai-grok-pager/docs/user-guide/26-config-reference.md`: user-facing key list.
- `clippy.toml`, `rustfmt.toml`, `rust-toolchain.toml`, `deny.toml`: repo policy.

**Core Logic:**

- `crates/codegen/xai-grok-shell/src/session/acp_session_impl/turn.rs`: turn loop and `build_request`.
- `crates/codegen/xai-grok-shell/src/session/acp_session_impl/sampler_turn.rs`: `run_turn_via_sampler`.
- `crates/codegen/xai-grok-shell/src/session/acp_session_impl/tool_calls.rs`: tool batches.
- `crates/codegen/xai-grok-shell/src/session/acp_session_impl/tool_dispatch.rs`: workspace dispatch.
- `crates/codegen/xai-chat-state/src/actor/`: conversation state.
- `crates/codegen/xai-grok-sampler/src/client.rs`: `POST {base}/chat/completions`.
- `crates/codegen/xai-grok-sampler/src/stream/chat_completions.rs`: SSE → `SamplingEvent`, quote repair.
- `crates/codegen/xai-grok-workspace/src/permission/manager/mod.rs`: permission actor.
- `crates/common/xai-grok-compaction/src/`: compaction steps.

**Testing:**

- Per-crate `src/**/*_tests.rs` and `tests/` integration tests (sampler wiremock under `crates/codegen/xai-grok-sampler/tests/`).
- `crates/codegen/xai-grok-pager-pty-harness/`: PTY e2e scenarios.
- `crates/codegen/xai-grok-test-support/`: mock inference server, ACP client, headless runner.
- `scripts/test-install.sh`, `scripts/smoke-test.sh`, `scripts/evals/`.
- `crates/codegen/xai-grok-pager-bin/tests/`: branding surface.

## Naming Conventions

**Files:**

- Rust modules: `snake_case.rs`. Large features are directories with `mod.rs` (`app/`, `session/`, `permission/`).
- Tests sit beside the code as `*_tests.rs` or a `tests.rs` included via `#[cfg(test)] #[path = "..."]`, plus crate-level `tests/*.rs` for integration.
- User docs: numbered markdown under `crates/codegen/xai-grok-pager/docs/user-guide/` (`26-config-reference.md`).
- Crates: directory name matches the package name (`xai-grok-sampler/`).

**Directories:**

- `crates/codegen/xai-grok-*` for product crates that kept the upstream prefix.
- `crates/codegen/xai-*` for shared product utilities that were never `grok`-prefixed (`xai-chat-state`, `xai-dirs`, `xai-acp-lib`).
- `crates/common/` for the tool/compaction cores.
- Session implementation files live in `acp_session_impl/` as one concern per file (`turn.rs`, `mcp.rs`, `cancel.rs`), not one giant impl.

**Symbols:**

- User-facing strings: `xai_grok_brand::NAME` and related constants.
- Internal types may still say `Grok`, `xai_api_base_url`, or `x_grok_session_id`. Those names are stable internals. Do not rename them as part of a feature.
- Env vars that users set: `DEEPSEEK_API_KEY`, `DEEPSEEK_BUILD_*` (`xai-grok-brand::ENV_PREFIX`).

## Where to Add New Code

**New user-facing CLI flag or subcommand:**

- Flag/parsing: `crates/codegen/xai-grok-pager/src/app/cli.rs` (`PagerArgs` / `Command`).
- Dispatch arm: `crates/codegen/xai-grok-pager-bin/src/main.rs`.
- Implementation: a `*_cmd.rs` module next to `doctor_cmd.rs` / `mcp_cmd.rs` under `crates/codegen/xai-grok-pager/src/`, exported from `src/lib.rs`.
- If the flag changes help text or version output, update branding coverage in `crates/codegen/xai-grok-pager-bin/tests/`.

**New TUI view or slash command:**

- View: `crates/codegen/xai-grok-pager/src/views/`.
- Slash command: `crates/codegen/xai-grok-pager/src/slash/`.
- Wire it through `app/actions.rs` and `app/effects/` so `event_loop.rs` stays a thin `select!`.
- Scrollback block: `crates/codegen/xai-grok-pager/src/scrollback/blocks/`.

**New session behavior (plan mode, compaction trigger, resume rule):**

- Primary code: `crates/codegen/xai-grok-shell/src/session/acp_session_impl/` (new file, `mod` it from the impl directory).
- Persistence: `crates/codegen/xai-grok-shell/src/session/persistence.rs` and, if the transcript shape changes, `crates/codegen/xai-chat-state/src/persistence.rs`.
- Tests: `*_tests.rs` beside the new file, using `SamplerHandle::noop()` the way existing session tests do.

**New model-request field or stream repair:**

- Wire type: `crates/codegen/xai-grok-sampling-types/`.
- HTTP and SSE: `crates/codegen/xai-grok-sampler/src/client.rs` and `src/stream/chat_completions.rs`.
- Turn integration: `session/acp_session_impl/sampler_turn.rs` and `turn.rs`.
- Wire tests: `crates/codegen/xai-grok-sampler/tests/`.
- Do not put this logic in the pager.

**New built-in tool:**

- Implementation: `crates/codegen/xai-grok-tools/src/implementations/` (follow `grok_build/` or `read_file/`).
- Register it on the bridge/registry (`src/bridge.rs`, `src/registry/`).
- If it needs a new protocol ID, extend `crates/common/xai-tool-protocol/` and the proto in `crates/codegen/xai-grok-tools-api/proto/grok-tools.proto`.
- Permission rules: `crates/codegen/xai-grok-permission-rules/` if the tool has allow/deny patterns; sandbox launch in `xai-grok-tools/src/sandbox_launch.rs` when it spawns a process.
- Do not call the tool from the TUI. The session dispatches through `tool_dispatch.rs`.

**New MCP behavior:**

- Client: `crates/codegen/xai-grok-mcp/src/`.
- Session attachment: `crates/codegen/xai-grok-shell/src/session/acp_session_impl/mcp.rs` and `mcp_init.rs`.
- Config keys: `crates/codegen/xai-grok-config/src/mcp_servers.rs`, then a hand edit of `26-config-reference.md`.

**New config key:**

- Types and load: `crates/codegen/xai-grok-config/` (and `xai-grok-config-types` if the type is shared).
- Shell resolution: `crates/codegen/xai-grok-shell/src/agent/config.rs` or `src/config/`.
- Document in `crates/codegen/xai-grok-pager/docs/user-guide/26-config-reference.md` if it is in `FEATURES`, `UNMIRRORED_BOOLEAN_FEATURES`, or `KNOWN_MCP_SERVER_FIELDS`.
- Removed-flow keys go in `UNDOCUMENTED_REMOVED_FLOW_KEYS`, not the page (`crates/codegen/xai-grok-shell/AGENTS.md`).

**New prompt guidance:**

- Edit `crates/codegen/xai-grok-agent/templates/prompt.md` (or `subagent_prompt.md`).
- Run `python3 scripts/encrypt_templates.py`.
- Keep `test_encrypted_templates_not_stale` green.

**Utilities:**

- Path/home: `crates/codegen/xai-dirs/`.
- HTTP client construction: `crates/codegen/xai-grok-extra-ca/`.
- Process/TTY: `crates/codegen/xai-tty-utils/`.
- Brand strings: `crates/codegen/xai-grok-brand/src/lib.rs`.
- Shared test doubles: `crates/codegen/xai-grok-test-support/` or `crates/common/xai-test-utils/`.

**Do not add code here:**

- Deleted surfaces: shell `src/upload/`, `src/cloud_config/`, `src/remote/`, `agent/relay.rs`, pager voice/announcements stubs, `xai-computer-hub-*` crates. They are absent on purpose.
- `third_party/` unless the vendored library itself is the task.
- `target/` (build output, gitignored).

## Special Directories

**`target/`:**

- Purpose: Cargo build output. Release binary is `target/release/deepseek-build`.
- Generated: yes.
- Committed: no.

**`crates/codegen/xai-grok-agent/src/prompt/prompt_encrypted.rs`:**

- Purpose: XOR-obfuscated prompt templates.
- Generated: yes, by `scripts/encrypt_templates.py`.
- Committed: yes. Do not hand-edit; change `templates/` and regenerate.

**`crates/common/xai-tool-protocol/generated/`:**

- Purpose: Generated protocol stubs (includes `swift/` and `kotlin/` trees).
- Generated: yes.
- Committed: yes. Change the proto source, then regenerate; do not patch generated files by hand.

**`scripts/evals/results/`:**

- Purpose: Eval harness outputs.
- Generated: yes when evals run.
- Committed: only if a results snapshot was intentionally checked in; treat new result files as local output unless the task says to keep them.

**`.planning/`:**

- Purpose: GSD planning documents, including this map (`codebase/`).
- Generated: no (written by planning workflows).
- Committed: according to the planning workflow, not by this mapper.

**`.opencode/`:**

- Purpose: OpenCode local tooling and its `node_modules`. Not part of the Rust workspace.
- Generated: dependencies yes.
- Committed: do not add product code here.

**`~/.deepseek-build` (runtime, not in the repo):**

- Purpose: per-user config, sessions, leader socket/log. Directory name comes from `xai_grok_brand::CONFIG_DIR_NAME`.
- Generated: at runtime.
- Committed: no.

---

*Structure analysis: 2026-11-21*
