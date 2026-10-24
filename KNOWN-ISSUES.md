# KNOWN ISSUES — deepseek-build

Honest list of what is incomplete or broken at this revision. Nothing here is
hidden behind a "temporary" flag; where code remains it is called out. Claims are
against the tree on `dsb/integration` after the live-smoke, egress, polish and
real-use passes (2026-10-07), with the test state refreshed 2026-10-09 and the
rebrand/docs notes refreshed 2026-10-21 against `main` (`90586ab0`).

## Build & tests

- **A full `cargo test --workspace` run is green except for two host-only
  failures.** The last full run (`cargo test --workspace --no-fail-fast --
  --test-threads=4`, 2026-10-09) reported **31,939 passed / 2 failed / 388
  ignored across 307 targets**. The 2 failures are
  `xai-grok-sandbox --test deny_paths_e2e` (`devbox_genuine_reexec_applies_enforcement`,
  `read_deny_empty_set_verifies_inside_bwrap`): `bwrap` fails with
  `Can't mkdir parents for /run/containerd/containerd.sock: Permission denied`
  because this host's `/run/containerd` is root-only. The crate is byte-identical
  to `last-known-good-2026-10-06`, and a bare `bwrap` with the same bind flags
  reproduces the failure, so these are environment, not code — they should pass
  where `/run/containerd` is readable. A full run on the final `cc6e6948` tree was
  in flight at session end.
- **`xai-grok-login`: green — 413 → 395 passed / 0 failed / 1 ignored.** The
  lane was red (405/33/1) because inert OAuth/OIDC scaffolding tests could never
  pass, and one e2e test deadlocked. Fixed 2026-10-09: the dead xAI login/OIDC
  surface was deleted (dropping 18 tests of removed behavior, ledgered), the
  sleep-gate test was rewritten to drive the real drain with a controllable
  refresher (was a deterministic hang), and the API-key env vars are now guarded so
  a developer's exported key can neither change results nor leak. See
  `docs/DROPPED-TESTS.md`.
- **`xai-grok-shell`: 6197 passed / 0 failed / 4 ignored** on the final tree (was
  6221; the 24-test drop is inside the deleted login surface). `xai-grok-pager`
  9876 / 0 / 4 and `xai-grok-pager-minimal` 94 / 0 unchanged.
- **All test targets compile** (`cargo check --workspace --tests` green).
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
- **Release workflow in CI: Linux only.** Tags `v0.0.1-rc2`/`rc3` failed on the
  Linux protoc 3.12 build; `v0.0.1-rc4` (run 40111152104) built both Linux
  targets, but `aarch64-apple-darwin` hit the 90-minute `timeout-minutes` and
  cancelled the run before `publish`. The Linux-only prerelease
  [`v0.0.1-rc4`](https://github.com/coruairc/deepseek-build/releases/tag/v0.0.1-rc4) was published by hand from that run's artifacts
  (2026-10-24); the x86_64 installer was verified against it. macOS builds are
  dropped for now (a per-target timeout change needs a token with the
  `workflow` scope).
- **Container image: never built or pulled.** The `container` job `needs` the
  whole build matrix, so the rc4 macOS cancellation skipped it; there is no
  GHCR package (checked 2026-10-24). Neither `docker/Dockerfile` nor
  `docker/Dockerfile.localtest` has been exercised.
- **Live scripts on any machine but the author's: never.** `scripts/smoke-test.sh`
  (8/8) and `scripts/egress-check.sh` (`EGRESS_VIOLATIONS=0`) passed there on
  2026-10-07 and have not been reproduced elsewhere.
- **`scripts/test-install.sh`: green locally** (39/39 assertions) against a
  loopback fake release; it never touches the real release pipeline.

## Rebrand / config

- Binary (`deepseek-build`), config dir (`~/.deepseek-build`), ACP namespace
  (`deepseek-build/*`), theme, user-facing strings, and the shipped user-guide
  docs' user-home paths are rebranded. `install.sh` additionally creates a `dsb`
  shortcut symlink (install-time only — the binary name itself is unchanged, D1).
- **Legacy env-var fallbacks are gone (2026-10-12).** Only the branded names
  are read: `DEEPSEEK_BUILD_HOME`, `DEEPSEEK_BUILD_SANDBOX`,
  `DEEPSEEK_BUILD_AGENT_DASHBOARD`, `DEEPSEEK_BUILD_THEME` /
  `LC_DEEPSEEK_BUILD_THEME`, and `DEEPSEEK_BUILD_APPEARANCE` /
  `LC_DEEPSEEK_BUILD_APPEARANCE`. The old `GROK_HOME`, `GROK_SANDBOX`,
  `GROK_AGENT_DASHBOARD`, `GROK_THEME`, and `GROK_APPEARANCE` names (including
  the `LC_` forms) are ignored, with tests pinning that. No automatic
  migration from `~/.grok`: set `DEEPSEEK_BUILD_HOME` to reuse it. Every
  `GROK_*` name named in the docs, `--help`, or product error text was renamed
  to `DEEPSEEK_BUILD_*` (2026-10-12); the remaining ~435 internal-only names
  (1,529 occurrences across 284 files) are listed in
  [`docs/INTERNAL-GROK-ENV-NAMES.md`](docs/INTERNAL-GROK-ENV-NAMES.md) for the
  open-build branch.
- **User-guide swept (2026-10-12).** The shipped guide no longer uses
  `grok <command>` examples, and the pages/sections describing removed flows
  (OAuth/OIDC/login, external auth providers, remote settings, managed config,
  external OpenTelemetry) are deleted (`02-authentication.md`,
  `24-monitoring-usage.md`, plus pruned sections elsewhere). Remaining prose
  reads `deepseek-build <command>`.

- **Project-level directory is still `.grok/` (2026-10-21).** Per-user state
  is `~/.deepseek-build`, but per-repo config, skills, agents, hooks, plugins,
  rules, sandbox profiles, and rewind checkpoints are still read from and
  written to `<repo>/.grok/` (e.g. `xai-grok-sandbox/src/profiles.rs`,
  `xai-grok-hooks/src/discovery.rs`, `xai-grok-workspace`). The user guide
  documents `.grok/` because that is what the code does; renaming it is a code
  change across several crates and has not been made.
- **Version string.** `deepseek-build --version` prints the inherited crate
  version `1.0.45` (`crates/codegen/xai-grok-pager-bin/Cargo.toml`), while the
  first planned release tag is `v0.1.0` (`CHANGELOG.md`). They will disagree
  until the crate version is reset or the tag is chosen to match.
- **Leftover sign-in surface.** `login` / `logout` subcommands and the `/login`
  slash command are still registered, and `login --help` still describes
  OAuth via `api.deepseek.com`, although the OAuth/OIDC stack was deleted and
  auth is env-only (D13). Use `DEEPSEEK_API_KEY`; do not rely on `login`.
- **Grove/clone config keys documented but inert.** The `clone` subcommand and
  `worktree redirect` commands do not exist (their guide pages were removed or
  replaced 2026-10-21), but `26-config-reference.md` still lists `[cli] grove`
  / `[clone]` keys and `DEEPSEEK_BUILD_CLONE` / `DEEPSEEK_BUILD_GROVE`.
- **Building from source needs `make`.** `tikv-jemalloc-sys` compiles jemalloc
  and its build script fails without `make` (observed 2026-10-21 on a host that
  had `cc` but no `make`).

## TUI (Step 2 checks, 2026-10-22)

- **Esc leaves picker text in the prompt.** Closing a `/model` or `/effort`
  picker with Esc keeps the typed command, so the next slash command is
  appended to it (`/model /effort`). Clear the prompt by hand.
- **No separate first-run notice.** A first launch shows only the welcome card
  ("Thanks for trying deepseek-build"); there is no one-time setup notice.

## Binary size (Step 3d, 2026-10-23)

- **Release profile left unchanged; LTO does not fit this build box.** Baseline
  `cargo build -j 2 --release --bin deepseek-build` at `61757a9c` (strip +
  panic=abort, no LTO): 152,670,552 bytes, 28m 38s.
  - `lto = "fat"` + `codegen-units = 1` at `-j 2`: rustc was killed by the OOM
    killer compiling `xai-grok-shell` (16 GB box).
  - `lto = "thin"` (default codegen-units) at `-j 1`: still compiling after the
    40-minute timeout (reached the `xai-grok-pager` crates, never linked); no
    size measured.
  - Retry thin or fat LTO on a machine with more RAM and cores, e.g. in CI via
    the existing `release-dist` profile (already thin LTO).

## Test-suite failures on the build box (Step 3e, 2026-10-23)

`cargo test --workspace -j 1` at `03393baa`: 31,901 passed, 8 failed, 388
ignored. One failure is a real upstream bug (pager reset overlay, being fixed on
`fe/fix-reset-overlay-dim`). The other 7 come from the box environment, not from
our changes:

- **3 `xai-grok-shell` credential tests fail when an API key is exported.**
  `agent::config::tests::has_own_credentials_guards_session_vs_external_key`,
  `agent::config::tests::process_key_from_model_env_key` and
  `sampling::error::tests::forbidden_subscription_error_no_hint_without_api_key`
  assume no key is set in the environment. They pass with the keys unset:
  `env -u DEEPSEEK_API_KEY -u XAI_API_KEY cargo test -p xai-grok-shell --lib -j 1 -- has_own_credentials_guards_session_vs_external_key process_key_from_model_env_key forbidden_subscription_error_no_hint_without_api_key`
- **`xai-grok-hooks` `runner::http::tests::validate_hook_url_accepts_and_rejects`
  (`http.rs:972`).** The box's fake-IP DNS resolves `nonexistent.invalid` to
  `198.18.0.1`, so the "unresolvable host" case resolves. Passes on normal DNS.
  Repro: `cargo test -p xai-grok-hooks --lib -j 1 -- validate_hook_url_accepts_and_rejects`
- **`xai-grok-sandbox` `devbox_genuine_reexec_applies_enforcement`.** bwrap
  re-exec can't create namespaces inside this container. Repro:
  `cargo test -p xai-grok-sandbox --test deny_paths_e2e -j 1 -- devbox_genuine_reexec_applies_enforcement`
- **2 `xai-proto-build` tests:** `debug_redact::tests::classifies_marked_fields_by_oneof_membership`
  and `debug_redact::tests::first_marked_field_finds_annotations_and_ignores_comments`.
  They look for protoc via `dotslash`, which isn't installed; CI uses apt protoc.
  Repro with system protoc:
  `PROTOC=/usr/bin/protoc cargo test -p xai-proto-build --lib -j 1`

### Ticket: SSRF guard does not block 198.18.0.0/15

`is_blocked_ip` in `xai-grok-hooks` does not treat `198.18.0.0/15` (RFC 2544
benchmarking range, used by fake-IP DNS resolvers) as blocked. A hook URL whose
host resolves there is allowed. Not an rc2 blocker; add the range to the
blocklist with a regression test in a follow-up.

## Licensing

- `LICENSE`, `NOTICE`, `THIRD-PARTY-NOTICES`, and `third_party/NOTICE` are
  preserved. Upstream files are not relicensed. Legal sign-off (DECISIONS O1) is
  still recommended before any public distribution.
