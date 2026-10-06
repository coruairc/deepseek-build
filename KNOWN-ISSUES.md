# KNOWN ISSUES — deepseek-build

Honest list of what is incomplete or broken at this revision. Nothing here is
hidden behind a "temporary" flag; where code remains it is called out. Claims are
against the tree on `dsb/integration` at `b19276ba`.

## Build & tests

- **REGRESSION at `b19276ba`: `cargo test --workspace --no-run` does not compile.**
  The deletion branches (`dsb/del-upload`, `dsb/del-cloud`, `dsb/del-hub`,
  `dsb/del-ui`) removed production code that many *test-only* modules still
  reference: `crate::hub` / `crate::hub_server` / `crate::mcp`,
  `xai_computer_hub_sdk`, `build_session_routed_handlers`, `SharedAuthProvider`,
  the MCP session bridge types (`WorkspaceMcpBinding`, `McpServerOutcome`,
  `FakeHubRegistry`), donation/observability methods, and more. Affected test
  files include `xai-grok-workspace/src/{handle_tests,host_kind_tests}.rs`,
  `sandbox/real_wiring_tests.rs`, `permission/hub_gate_tests.rs`,
  `permission/hub_permission.rs` (test-gated), and others across the workspace.
- **Last known-good tag:** `last-known-good-2026-10-06` at `b3847ff8` — the last
  commit where `cargo test --workspace --no-run` exited 0 and the per-crate
  suites passed (`xai-grok-shell` 6638, `xai-grok-workspace` 2018,
  `xai-grok-tools` 3310, `xai-grok-pager` 9976, `xai-grok-pager-render` 1209,
  `xai-grok-agent` 576, `xai-chat-state` 391, `xai-grok-status-line` 22,
  `xai-grok-telemetry` 164, `xai-fast-worktree` 333, …). `main` currently points
  at `b19276ba`, which holds the deletion commits but a broken test compile.
- **Non-test build is green:** `cargo check -p xai-grok-pager-bin` and
  `cargo build --release` succeed at `b19276ba`; the binary runs.
- **`cargo clippy` / `cargo fmt` were only run on crates touched per slice**, not
  workspace-wide.
- A `syn`-based item stripper was prototyped at `/tmp/opencode/synstrip` to remove
  top-level test items referencing deleted symbols; it works but the test fallout
  is large enough that a per-crate iterative pass is still required.

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
- The pager feedback modal UI remains, but the shell feedback manager was deleted;
  `/feedback` is inert (no upload).
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
