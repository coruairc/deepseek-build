# KNOWN ISSUES — deepseek-build

Honest list of what is incomplete or broken at this revision. Nothing here is
hidden behind a "temporary" flag; where code remains it is called out. Claims are
against the tree on `dsb/integration` after the live-smoke, egress, polish and
real-use passes (2026-10-07).

## Build & tests

- **`cargo test --workspace` does not compile.** The only failing crate is
  **`xai-grok-workspace`** (all other crates' test targets compile). Its
  `#[cfg(test)]` / `test_support` modules still reference production code removed
  by the deletion branches: `crate::hub` / `crate::hub_server` / `crate::mcp`,
  `xai_computer_hub_sdk`, `build_session_routed_handlers`, `SharedAuthProvider`,
  the MCP session bridge types (`WorkspaceMcpBinding`, `McpServerOutcome`,
  `FakeHubRegistry`), the removed `SessionContext.auth_provider` field, and
  donation/observability methods. Affected files:
  `xai-grok-workspace/src/{handle_tests,host_kind_tests}.rs`,
  `session/tool_config.rs` (`test_support`), `permission/hub_permission.rs`
  (test module), `permission/hub_gate_tests.rs`. Repair tooling exists
  (`scripts/synstrip/`, `scripts/line_loop.py`); this is Step 6 and is not done.
- **Non-test build is green:** `cargo check -p xai-grok-pager-bin` and
  `cargo build --release -p xai-grok-pager-bin` succeed; the binary runs
  (`target/release/deepseek-build`, `deepseek-build 1.0.45`).
- **Tags:** `last-known-good-2026-10-06` (`b3847ff8`, last green test compile;
  still contains egress code — not safe to run).

## Network / egress

- **HARD and SOFT gates: green.** `scripts/check-egress.sh --strict` prints
  `HARD egress gate: OK` and exits 0.
- **Extended runtime egress verified (2026-10-07)** with
  `scripts/egress-check.sh` — an LD_PRELOAD logger that allows-and-logs
  `connect`/`sendto`/`sendmsg`/`sendmmsg`/`getaddrinfo` from the binary and its
  dynamic children. A real model-driven session (shell, file edit, `web_fetch`
  of `https://example.com/`, a local stdio MCP tool, forced auto-compaction,
  resume, headless) observed **only**:
  - `api.deepseek.com:443` over IPv6/NAT64 (`64:ff9b::3ad:153f`),
  - `example.com:443` for the `web_fetch` call,
  - AF_UNIX sockets (`/run/systemd/resolve/io.systemd.Resolve`,
    `/var/run/dbus/system_bus_socket`).
  `EGRESS_VIOLATIONS=0`. Non-loopback public destinations must be on port 443.
- **`scripts/sandbox-run.sh` is stricter**: it allows only the provider +
  loopback, so the `web_fetch` host is refused (the session still completes).
  It has a cosmetic gap: its shim formats only AF_INET, so IPv6 refusals print as
  `non-ip`, and the provider's IPv6 NAT64 attempt is refused before the client
  falls back to the allowlisted IPv4 address.
- **LD_PRELOAD limits (inherent, not fixed):** raw `syscall(SYS_connect|sendto)`,
  `io_uring`, statically-linked or setuid children, and children that scrub
  `LD_PRELOAD` are not intercepted; allowlisting is by resolved IP, so another
  hostname on the same shared CDN edge IP would pass. `strace` is absent in this
  environment. For kernel-level ground truth use
  `strace -f -e trace=connect,sendto,sendmsg` where available.
- **Removed and verified gone:** `xai-grok-shell/src/agent/session_registry_client.rs`
  and all remote session-registry writers; `common/xai-tracing` and the OTLP
  exporter stack (`opentelemetry`, `opentelemetry-otlp`, `opentelemetry-proto`,
  `opentelemetry-http`, `tracing-opentelemetry`, `fastrace-opentelemetry`) —
  `Cargo.lock` has 0 `opentelemetry` entries; `xai-grok-http::shared_upload_client()`.
  `tonic` remains (tools/tools-api/workspace/login).
- **User-configured HTTP hooks** (`xai-grok-hooks`) are intentionally kept.

## API key handling

- **Env-only auth:** `DEEPSEEK_API_KEY` (primary) or `DEEPSEEK_BUILD_API_KEY`
  (fallback). No interactive key flow. Missing key prints one line:
  `Not signed in: set DEEPSEEK_API_KEY (or DEEPSEEK_BUILD_API_KEY) to authenticate.`
- **Never logged:** `SamplerConfig`'s `Debug` redacts `api_key`; the sampler's
  header-conversion errors and the new-session debug line no longer echo it.
  `scripts/key-leak-check.sh` runs the binary with a fake key and scans debug logs
  and saved sessions — currently PASS. A unit test (`debug_redacts_api_key`)
  pins the redaction.
- **Live smoke (Step 1, 2026-10-07):** `scripts/smoke-test.sh` = 8/8 PASS. See
  the smoke results below.

## Step 1 — live adapter results

- `GET /models` returns exactly `deepseek-flash`, `deepseek-v4-pro`.
- Plain and streamed headless chat OK; the hidden `deepseek-v4-flash` alias maps
  to `deepseek-flash`.
- A 5-turn thinking tool-call chain ran (`num_turns=5`, `modelCalls=5`,
  `reasoning_tokens>0`).
- Usage mapping confirmed: harness `input_tokens` = DeepSeek
  `prompt_cache_miss_tokens`, `cache_read_input_tokens` = `prompt_cache_hit_tokens`,
  `reasoning_tokens` = `completion_tokens_details.reasoning_tokens`,
  `total_tokens` = input + cache_read + output. On a cache hit
  `cached_tokens == prompt_cache_hit_tokens` (verified 2432). No adapter mismatch.
- `tool_choice=required` in thinking mode → HTTP 400 (documented in O3).

## Step 4 — real-use results (scratch repo, headless)

- **Worked:** simple edit; fixing a failing test (it ran `python3 test_calc.py`);
  a multi-file rename (`mul`→`multiply` across 4 files); session resume with
  context recall. Warm-turn cache hit rate 98–99%.
- **Plan mode gap:** `--permission-mode plan` is read-only (no file changed) but
  does not send `session/set_mode`, write `plan_mode.json`, or inject the
  plan-mode reminder; a bare "change X" request therefore ends as a cancelled
  turn with empty output instead of a plan. A prompt that explicitly asks for a
  plan produces one.
- **Headless cannot prompt:** a permission-gated action (e.g. `rm -rf`) ends as
  `permission_cancelled` with empty output and exit 0 — safe (the canary dir
  survived) but indistinguishable at the CLI from other cancellations.
  `--always-approve` is required for unattended actions.
- **Cost is not exposed** in headless output: DeepSeek returns no
  `cost_usd_ticks`, so `total_cost_usd` is omitted. Cache hit rate is derivable
  from `usage`.

## Phase 2 — adapter

- **Backend removal (D5): done.** `ApiBackend` has only `ChatCompletions`.
- **Wiremock suite present** (`crates/codegen/xai-grok-sampler/tests/chat_completions_wire.rs`):
  streaming text + reasoning, 3-turn tool chain replaying `reasoning_content`,
  missing-`reasoning_content` backfill, quote repair, partial/malformed stream;
  retry/backoff in `tests/test_actor.rs`.
- **Automatic model routing is not wired** (`model_routing` defaults to `off`);
  manual `/model` + `/effort` is what works. Catalog defines `deepseek-v4-pro`,
  `deepseek-flash`, and the hidden `deepseek-v4-flash` alias.

## Phase 3 — TUI

- Implemented: reasoning folding (`e`/`E`/`Ctrl+E`), `/think`, `/model`,
  `/effort`, `/theme`, Monokai theme with whale logo, configurable status line
  (`[ui.status_line]`, items include `model`, `context`, `effort`, `tokens`,
  `cache`, `cost`).
- The status row does not surface the auto-routing decision (moot until routing
  is wired).

## Phase 4 — hardening

- Read-before-write enforced; dangerous commands (`rm -rf`-class) stay gated with
  a warning.
- Compaction, MCP, resume, headless were exercised headlessly (Step 2). Plan
  mode and permission *prompts* were not exercised interactively.

## Rebrand / config

- Binary (`deepseek-build`), config dir (`~/.deepseek-build`), ACP namespace
  (`deepseek-build/*`), theme, and user-facing strings are rebranded.
- **User-visible env vars renamed** with legacy fallbacks:
  `GROK_SANDBOX`→`DEEPSEEK_BUILD_SANDBOX`, `GROK_AGENT_DASHBOARD`→
  `DEEPSEEK_BUILD_AGENT_DASHBOARD`. `DEEPSEEK_BUILD_HOME` is primary and
  `GROK_HOME` is the legacy fallback. **Internal-only `GROK_*` names remain**
  (~580 distinct), per D2; these are not user-facing.
- No automatic migration from `~/.grok`; set `GROK_HOME` (legacy) to reuse it.

## Licensing

- `LICENSE`, `NOTICE`, `THIRD-PARTY-NOTICES`, and `third_party/NOTICE` are
  preserved. Upstream files are not relicensed. Legal sign-off (DECISIONS O1) is
  still recommended before any public distribution.
