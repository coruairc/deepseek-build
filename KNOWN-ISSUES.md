# KNOWN ISSUES — deepseek-build

Honest list of what is incomplete or broken at this revision. Nothing here is
hidden behind a "temporary" flag; where code remains it is called out. Claims are
against the tree on `dsb/integration` after the live-smoke, egress, polish and
real-use passes (2026-10-07), with the test state refreshed 2026-10-09.

## Build & tests

- **A full `cargo test --workspace` run has NOT gone green yet.** All test
  targets compile, but the first full-workspace run (2026-10-08) surfaced
  failures in `xai-grok-login` before its log was cut off mid-run. A follow-up
  run of that crate alone with a scratch `HOME`/`DEEPSEEK_BUILD_HOME` also failed:
  `xai-grok-login --lib` = **405 passed / 33 failed / 1 ignored** (one flaky
  sleep-gate test skipped; log `/tmp/opencode/v2t/login-clean.log` on the author's
  machine, 2026-10-08 23:54). The failures cluster in
  `refresh::oidc_refresher` (16), `manager::tests` (9), `flow`, `external_auth`
  and one `storage` test — the inert OAuth/OIDC scaffolding this fork no longer
  wires up. Another workstream is still investigating: **do not claim the
  workspace green until a full run finishes with zero failures.** Separately, the
  same first full run surfaced four `xai-fast-worktree` failures that per-crate
  runs could not see (they only compile under the `metadata` feature, which
  workspace feature unification enables): they test the NFS/Grove backend that
  this tree stubs off in `nfs_off.rs`. Dropped with a ledger entry
  (`docs/DROPPED-TESTS.md`); `xai-fast-worktree` is then 455 passed / 0 failed with
  `--features metadata`.
- **All test targets compile** (`cargo check --workspace --tests` green).
  Latest recorded green per-crate runs: `xai-grok-pager` 9876 passed / 0 failed /
  4 ignored (2026-10-08, `a16308b8` + repairs), `xai-grok-pager-minimal` 94
  passed / 0 failed.
- **`xai-grok-shell` lib tests: green in the last per-crate run (2026-10-07).**
  6221 passed / 0 failed / 4 ignored. Integration tests compile after dropping
  the 8 remote-settings prefetch tests for the deleted `remote_config` /
  `managed_config` modules. The 2026-10-08 workspace run did not reach a recorded
  result for this crate before its log was cut off; treat 6221 as the last
  measured number, not as part of a green workspace run.
- **`xai-grok-workspace` test target: compiles and passes** (1693 passed; a
  pre-existing `session::git_gate` / `restore_fetch` timing flake rotates under
  whole-suite saturation and passes in isolation). The last 25 failures were
  tests of the deleted computer hub, not regressions. **Coverage gap to
  re-home:** the local harness `SessionToolHandle`/`create_local_harness` runs
  the toolset without the deleted hub pre-run/sandbox integration, so
  equivalent local coverage must be re-established (see
  `docs/DROPPED-TESTS.md`).
- **Per-crate test results (last recorded per-crate runs, 2026-10-08):**
  sampling-types 240, sampler 186, telemetry 156, status-line 24, brand 2,
  version 2, config 467, tools-api 15, chat-state 391, session-events 16,
  active-sessions 6, foreign-sessions 53, config-types 45, http 14, mcp 279,
  hooks 279, agent 576 — all 0 failures **in those runs**. These do not cover
  `xai-grok-login`, `xai-grok-pager` (last 9876) or `xai-fast-worktree`. Because
  workspace feature unification compiles feature-gated tests that per-crate
  runs miss (the `xai-fast-worktree` case below), a per-crate pass is necessary
  but not sufficient evidence for the workspace run.
- **Non-test build is green:** `cargo check -p xai-grok-pager-bin` and
  `cargo build --release -p xai-grok-pager-bin` succeed; the binary runs
  (`target/release/deepseek-build`, 147 MiB at the last measured build
  `46aef0920857`, `deepseek-build 1.0.45 (<commit>)`).
- **Step 6 mutation check:** breaking the sandbox truncation cap (`5_000`→
  `6_000`) made `sandbox_host_caps_task_output_polls_at_5k` fail, confirming the
  sandbox tests still detect the behavior; the change was reverted.
- **Tags/releases:** on `origin` only `last-known-good-2026-10-06` (`b3847ff8`,
  before the pager test repair, still contains egress code — not safe to run) and
  `parked-test-repair-2026-10-06` (`c7d7f7e4`); two local-only tags
  (`pre-merge-main-2026-10-07`, `pre-merge-integration-2026-10-07`). **No `v*`
  release tag has ever been pushed and no GitHub release exists** (checked
  against `origin` and `gh release list` on 2026-10-09).

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
- **Live smoke (Step 1, 2026-10-07):** `scripts/smoke-test.sh` = 8/8 PASS on the
  author's machine. See the smoke results below.

## Step 1 — live adapter results

> All results below are from the author's machine (2026-10-07) and have not been
> reproduced on another host.

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

> Author's machine, 2026-10-07; not reproduced elsewhere.

- **Worked:** simple edit; fixing a failing test (it ran `python3 test_calc.py`);
  a multi-file rename (`mul`→`multiply` across 4 files); session resume with
  context recall. Warm-turn cache hit rate 98–99%.
- **Plan mode gap:** `--permission-mode plan` is read-only (no file changed) but
  does not send `session/set_mode`, write `plan_mode.json`, or inject the
  plan-mode reminder; a bare "change X" request therefore ends as a cancelled
  turn with empty output instead of a plan. A prompt that explicitly asks for a
  plan produces one.
- **Headless cannot prompt:** a permission-gated action (e.g. `rm -rf`) ends as
  `permission_cancelled` with a one-line stderr message naming `--always-approve`
  and a non-zero exit (`d3a9c176`). `--always-approve` is required for unattended
  actions.
- **Cost:** the provider still reports no `cost_usd_ticks`, so `total_cost_usd`
  is omitted. The headless `--output-format json` result carries a local estimate
  instead: `estimated_cost_usd` + `estimate: true`, priced from the shipped
  DeepSeek peak-rate table (D14), omitted when no tokens were recorded. The eval
  harness records it per task (`scripts/evals/`).

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

## Unexercised surfaces (no human, no CI)

- **Interactive TUI by a human: never.** It has been driven under a pty with
  scripted keystrokes (2026-10-07: welcome logo, `/model`, `/effort`, `/think`,
  status line), but no person has used it interactively. §4 items in
  [`TESTING.md`](TESTING.md) are the checklist for that pass.
- **Release workflow in CI: never run.** `.github/workflows/release.yml` has
  never executed; no `v*` tag has been pushed; no GitHub release exists
  (`gh release list` empty, verified 2026-10-09). The test-tag procedure in
  [`AGENTS.md`](AGENTS.md) §6 is untested end to end.
- **Container image: never built or pulled.** Neither the production
  `docker/Dockerfile` (needs staged release tarballs) nor
  `docker/Dockerfile.localtest` has been exercised; the README's GHCR pull
  examples cannot work until a release is published.
- **Live scripts on any machine but the author's: never.** `scripts/smoke-test.sh`
  (8/8) and `scripts/egress-check.sh` (`EGRESS_VIOLATIONS=0`) passed there on
  2026-10-07 and have not been reproduced elsewhere.
- **`scripts/test-install.sh`: green locally** (25/25 assertions) against a
  loopback fake release; it never touches the real release pipeline.

## Rebrand / config

- Binary (`deepseek-build`), config dir (`~/.deepseek-build`), ACP namespace
  (`deepseek-build/*`), theme, user-facing strings, and the shipped user-guide
  docs' user-home paths are rebranded.
- **User-visible env vars renamed** with legacy fallbacks:
  `GROK_SANDBOX`→`DEEPSEEK_BUILD_SANDBOX`, `GROK_AGENT_DASHBOARD`→
  `DEEPSEEK_BUILD_AGENT_DASHBOARD`. `DEEPSEEK_BUILD_HOME` is primary and
  `GROK_HOME` is the legacy fallback. **Internal-only `GROK_*` names remain**
  (~580 distinct identifiers at `e6b664cc`), per D2; these are not user-facing.
- No automatic migration from `~/.grok`; set `GROK_HOME` (legacy) to reuse it.
- **User-guide prose still has pre-removal content (counted 2026-10-09).** The
  docs under `crates/codegen/xai-grok-pager/docs/user-guide/` (28 files, shipped
  in the binary, extracted to `~/.deepseek-build/docs/user-guide/`) still describe
  removed xAI OAuth/OIDC login flows (129 line hits across 14 files) and use
  `grok <command>` examples where the binary is now `deepseek-build` (**304
  occurrences of a `grok <word>` command across 25 of 28 files**, e.g. `grok -p`,
  `grok inspect`, `grok mcp`, `grok login`). The home-dir paths are fixed; the
  prose and command examples need a follow-up editing pass. Until then, read the
  command examples as `deepseek-build <command>`.

## Licensing

- `LICENSE`, `NOTICE`, `THIRD-PARTY-NOTICES`, and `third_party/NOTICE` are
  preserved. Upstream files are not relicensed. Legal sign-off (DECISIONS O1) is
  still recommended before any public distribution.
