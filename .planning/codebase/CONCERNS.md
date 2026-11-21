---
last_mapped_commit: 221a1a591ef3af5f90d59d73ae7899d110f28b37
last_mapped_at: 2026-11-21
---
<!-- refreshed: 2026-11-21 -->
# Codebase Concerns

**Analysis Date:** 2026-11-21

Scope is `main` at `221a1a59` (the rc5 lineage), which this branch is rebased onto. Phase-1 stub paths named in earlier handoffs (`xai-grok-shell/src/upload`, `src/cloud_config`, `src/remote`, `agent/relay.rs`, pager voice/announcements, `xai-computer-hub-*`) are absent. Do not recreate them.

`DECISIONS.md` (last updated 2026-10-07) and `TESTING.md` disagree with this tree. Trust the code, then `AGENTS.md` / `KNOWN-ISSUES.md`, and treat the phase log in `DECISIONS.md` as stale.

## Tech Debt

**Stale decision log and test checklist:**
- Issue: `DECISIONS.md` §F still says rebrand is not done, Phase 3 is not started, Phase 5 has no binary, and O2 (toolchain) is "being resolved". `TESTING.md` §6 still reports `xai-grok-login --lib` as 405 passed / 33 failed in OIDC scaffolding, and §2 still quotes a last-observed `OIDC not configured` sign-in line. Login/OIDC was deleted (`cc6e6948` is an ancestor). The live missing-key string is in `crates/codegen/xai-grok-pager/src/headless.rs`.
- Files: `DECISIONS.md`, `TESTING.md`, `KNOWN-ISSUES.md`
- Impact: A later agent can "finish" work that is already done, or chase deleted OIDC tests. D9 ("skip Phase 5") also conflicts with the release pipeline that now exists (`.github/workflows/release.yml`, `install.sh`, `docker/Dockerfile`).
- Fix approach: Rewrite `DECISIONS.md` §E/§F and `TESTING.md` §2/§6 from the tree. Keep D9's "personal tool" intent, but record that the release pipeline was added anyway and is not a green CI release.

**Automatic model routing is implemented and never called:**
- Issue: `ModelRoutingPolicy` defaults to `Off`. `SamplingClient::route_turn` is the only caller of `resolve_route`, and nothing outside `crates/codegen/xai-grok-sampler/src/client.rs` calls `route_turn`. Shell construction sites set `model_routing: Default::default()` (`crates/codegen/xai-grok-shell/src/tools/config.rs`, `src/agent/config.rs`, `src/agent/subagent/mod.rs`, `src/session/acp_session_impl/sampler_turn.rs`).
- Files: `crates/codegen/xai-grok-sampler/src/model_routing.rs`, `crates/codegen/xai-grok-sampler/src/client.rs`
- Impact: `/model` and `/effort` are the only routing that works. Flash-vs-Pro cost control from `docs/TUNING.md` cannot turn on by setting a policy the turn loop never reads.
- Fix approach: Either call `route_turn` from the sampler turn in `sampler_turn.rs` (and surface the decision on the status line), or delete the unused policy until it is wired. Do not document it as a user setting until a caller exists.

**Internal `GROK_*` names are still read, not just listed:**
- Issue: `docs/INTERNAL-GROK-ENV-NAMES.md` (2026-10-12) lists 435 names / 1,529 occurrences / 284 files. User-facing fallbacks (`GROK_HOME`, `GROK_SANDBOX`, `GROK_THEME`, `GROK_APPEARANCE`) are ignored, but product code still reads other `GROK_*` variables. Examples: `GROK_RELAY_SYNC_ENABLED` in `crates/codegen/xai-grok-config-types/src/mcp.rs`; `GROK_CAMPAIGNS` in `crates/codegen/xai-grok-config/src/config_layers.rs`; `GROK_TELEMETRY_SPECIAL_USER` in `crates/codegen/xai-grok-telemetry/src/id.rs`; `GROK_DEBUG_LOG` / `GROK_OPEN_DASHBOARD_AT_STARTUP` in `crates/codegen/xai-grok-pager-bin/src/main.rs`; `GROK_FORCE_LEGACY_CONSOLE` in `crates/codegen/xai-grok-pager-render/src/glyphs.rs`; build-script `GROK_SHELL_BUNDLE_RG_PATH` and `GROK_SHELL_RG_DOWNLOAD_BASE` in `crates/codegen/xai-grok-shell/build.rs`.
- Files: `docs/INTERNAL-GROK-ENV-NAMES.md` plus the call sites above
- Impact: Operators following `DEEPSEEK_BUILD_*` docs cannot see or set these knobs. A leftover `GROK_TELEMETRY_SPECIAL_USER` or `GROK_CAMPAIGNS` in the environment still changes behavior. Crate names stay `xai-grok-*` by D2; that part is intentional (`DECISIONS.md` O4).
- Fix approach: Rename live readers to `DEEPSEEK_BUILD_*` in one pass, keep tests that pin ignored legacy names, and regenerate `docs/INTERNAL-GROK-ENV-NAMES.md`. Do not rename crate packages in the same change.

**Deleted computer-hub still described as live:**
- Issue: `crates/common/xai-tool-protocol` still documents routing, registration, and handshake fields as "the computer hub" (`src/capabilities.rs`, `src/handshake.rs` `computer_hub_version`, `src/frames.rs`, `src/connection.rs`, `src/methods.rs`). `crates/codegen/xai-grok-workspace-types/src/types/` is a parallel DTO layer whose module docs are `TODO: align with` the canonical shell/tool types.
- Files: `crates/common/xai-tool-protocol/src/`, `crates/codegen/xai-grok-workspace-types/src/types/`
- Impact: New code will reintroduce hub assumptions or drift the wire DTOs from the types the session actually uses (`McpServerStatus` / `LspServerStatus` are aliases; free-form hook event strings accept typos — see `types/session.rs` and `types/plugins.rs`).
- Fix approach: Rewrite the protocol comments to the local tool path. Collapse the workspace-type aliases onto the canonical types, or mark the DTOs as an explicit lossy boundary with tests.

**XOR prompt templates:**
- Issue: System prompts are not the files under `crates/codegen/xai-grok-agent/templates/`. Runtime reads XOR bytes in `crates/codegen/xai-grok-agent/src/prompt/prompt_encrypted.rs`. `template.rs` says this is obfuscation, not a security boundary (seeds are in-repo). A stale encrypt step decrypts to invalid UTF-8 and `expect`s.
- Files: `crates/codegen/xai-grok-agent/templates/`, `crates/codegen/xai-grok-agent/src/prompt/template.rs`, `scripts/encrypt_templates.py`
- Impact: Editing the markdown without `python3 scripts/encrypt_templates.py` ships the old prompt. The `expect` aborts the process (`panic = "abort"` in `[profile.release]` and `[profile.dev]` in `Cargo.toml`).
- Fix approach: After any template edit, run `python3 scripts/encrypt_templates.py` and `cargo test -p xai-grok-agent --lib prompt::template::tests::test_encrypted_templates_not_stale`. Do not treat the XOR step as a secrecy control.

**Hand-edited config reference:**
- Issue: `crates/codegen/xai-grok-pager/docs/user-guide/26-config-reference.md` must be edited by hand when `FEATURES`, `UNMIRRORED_BOOLEAN_FEATURES`, or `KNOWN_MCP_SERVER_FIELDS` change. Removed-flow keys belong in `UNDOCUMENTED_REMOVED_FLOW_KEYS`, not the page (`crates/codegen/xai-grok-shell/AGENTS.md`).
- Files: `crates/codegen/xai-grok-pager/docs/user-guide/26-config-reference.md`, `crates/codegen/xai-grok-shell/src/config_docs/mod.rs`
- Impact: A new key that is only in the registry is invisible to users; a key left on the page documents a deleted flow.
- Fix approach: Update the page in the same commit as the registry, then run the `config_docs` tests (`--features config-docs`).

## Known Bugs

**Sandbox e2e fails when `/run/containerd` is root-only (recorded; still the test's shape):**
- Symptoms: `KNOWN-ISSUES.md` records `xai-grok-sandbox --test deny_paths_e2e` failures in `read_deny_empty_set_verifies_inside_bwrap` and `devbox_genuine_reexec_applies_enforcement`: bwrap `Can't mkdir parents for /run/containerd/containerd.sock: Permission denied`. Last full workspace run (2026-10-09) was 31,939 passed / 2 failed / 388 ignored. This mapping did not re-run that suite.
- Files: `crates/codegen/xai-grok-sandbox/tests/deny_paths_e2e.rs`, `crates/codegen/xai-grok-sandbox/src/runtime_sockets.rs` (`/run/containerd/containerd.sock`), `crates/codegen/xai-grok-sandbox/src/lib.rs` (`bwrap_reexec_command_ex` `--ro-bind`s deny paths and refuses to start on a partial sandbox)
- Trigger: Linux host where the containerd runtime directory exists and is not traversable by the test user. `skip_if_enforcement_unavailable` only skips missing bwrap or an unsupported sandbox, not `EACCES` on a deny-path parent.
- Workaround: Run where `/run/containerd` is readable, or do not treat these two failures as product regressions. If the same bind is applied to a real session, sandbox startup fails closed rather than skipping the unreadable socket — that is safer than a partial sandbox, but it means "sandbox on" cannot start on this class of host.
- Fix approach: When materializing runtime-socket denies, skip candidates whose parent is not traversable (same as `NotFound`), and add a test that a root-only parent does not abort bwrap. Keep the fail-closed path for denies the process can see but cannot mask.

**`scripts/sandbox-run.sh` drops IPv6 provider addresses and prints them as `non-ip`:**
- Symptoms: The allowlist is built with `AF_INET` only. An `AF_INET6` connect is allowed only for `::1`. Blocked IPv6 destinations are reported as `non-ip` because the log path formats only `AF_INET`.
- Files: `scripts/sandbox-run.sh`
- Trigger: Provider resolved via NAT64/IPv6 first (observed in `KNOWN-ISSUES.md` for `api.deepseek.com`). The session can still complete after falling back to the allowlisted IPv4 address. `web_fetch` hosts are refused on purpose (stricter than `scripts/egress-check.sh`).
- Workaround: Use `scripts/egress-check.sh` for the runtime allowlist that includes the fetch host. Do not read `non-ip` as "no destination".
- Fix approach: Resolve `EGRESS_ALLOW_HOST` with `AF_UNSPEC`, allow the resulting IPv6 addresses, and `inet_ntop` `AF_INET6` in the blocked line.

**`git_gate` / `restore_fetch` timing flake (recorded, not re-run):**
- Symptoms: `KNOWN-ISSUES.md` says `session::git_gate` / `restore_fetch` fails under whole-suite saturation and passes in isolation.
- Files: `crates/codegen/xai-grok-workspace/src/session/git_gate.rs` (`SNAPSHOT_TTL` 1s; test `ROOT_CACHE_TTL` 80ms), `crates/codegen/xai-grok-workspace/src/restore_fetch.rs` (60s shared fetch budget)
- Trigger: Workspace suite saturation, not a logic change.
- Workaround: Re-run the crate in isolation before treating a red `xai-grok-workspace` as a regression.
- Fix approach: Replace wall-clock joins in the tests with a controllable clock, or mark the saturated cases `#[ignore]` with a single deterministic test of the epoch-retry cap (`MAX_EPOCH_RETRIES`).

**Headless cannot approve a permission prompt:**
- Symptoms: A gated tool (for example `rm -rf`) ends as `permission_cancelled`, non-zero, with stderr telling the user to pass `--always-approve`.
- Files: `crates/codegen/xai-grok-pager/src/headless.rs` (message near the permission-cancelled path)
- Trigger: Headless/`-p` without `--always-approve` or a non-interactive permission mode that already allows the command.
- Workaround: `--always-approve` for unattended runs. That bypasses the prompt; it does not make the prompt work.
- Fix approach: Keep the fail-closed behavior. Document it next to headless in the user guide. Do not add a hidden auto-approve.

**Cost is always the peak rate, and the provider sends no price:**
- Symptoms: `total_cost_usd` is omitted. Headless JSON emits `estimated_cost_usd` plus `estimate: true`. The table has no clock, so off-peak (half of peak, per D14) is never applied unless the user overrides `[ui.status_line.pricing.<model>]`.
- Files: `crates/codegen/xai-grok-status-line/src/pricing.rs`, `DECISIONS.md` D14
- Trigger: Any run outside DeepSeek peak hours (01:00-04:00 and 06:00-10:00 UTC Mon-Fri, excluding Chinese holidays).
- Workaround: Halve the three rates in config to price off-peak. The status-line `cost` item and the JSON field are estimates, not invoices.
- Fix approach: Optional: add an explicit off-peak flag. Do not scrape the pricing page at runtime (new outbound destination).

## Security Considerations

**GNU release binaries do not get the RELRO/NX flags the musl targets get:**
- Risk: `[profile.release-dist]` in `Cargo.toml` says linker hardening (RELRO, NX stack) is set per-target in `.cargo/config.toml`. Those `-Wl,-z,relro,-z,now,-z,noexecstack` flags exist only under `[target.x86_64-unknown-linux-musl]` and `[target.aarch64-unknown-linux-musl]`. The native release jobs use `x86_64-unknown-linux-gnu` / `aarch64-unknown-linux-gnu`, whose rustflags are only `force-unwind-tables` (and `target-cpu=neoverse-v2` on arm64).
- Files: `.cargo/config.toml`, `Cargo.toml` `[profile.release-dist]`, `.github/workflows/release.yml`
- Current mitigation: `panic = "abort"` and symbol stripping on default `--release`. Not a substitute for full RELRO.
- Recommendations: Add the same link-args to the gnu target sections before the next release build. Confirm with `readelf -l` (`GNU_RELRO`) and `-W,now` on the tarball binary.

**Release build downloads ripgrep from GitHub and bundles it:**
- Risk: `crates/codegen/xai-grok-shell/build.rs` runs on release (and whenever `GROK_SHELL_BUNDLE_RG_PATH` is set). If the override path is unset it `reqwest::blocking::get`s `https://github.com/BurntSushi/ripgrep/releases/download/15.0.0/ripgrep-15.0.0-<triple>.tar.gz` and compiles the binary into the crate. The mirror override is still named `GROK_SHELL_RG_DOWNLOAD_BASE`.
- Files: `crates/codegen/xai-grok-shell/build.rs`
- Current mitigation: Debug `cargo check` skips the download. A local `GROK_SHELL_BUNDLE_RG_PATH` skips the network. Windows without that path skips bundling and uses `rg` on `PATH`.
- Recommendations: Vendor the pinned tarball (checksum in-repo) or require the path override in release CI. Rename the env var when the other `GROK_*` readers move. This is a build-time outbound destination; do not add another one without an explicit decision.

**User HTTP hooks are an outbound path D10 does not name:**
- Risk: D10 allowlists the model provider and user-configured MCP servers. `xai-grok-hooks` still POSTs to user `https://` hook URLs. `validate_hook_url` blocks non-https and private IPs at DNS time only; the comment in `http.rs` states DNS rebinding can swap in a blocked address before send. Redirects are disabled.
- Files: `crates/codegen/xai-grok-hooks/src/runner/http.rs`
- Current mitigation: https-only, no redirects, private-IP reject at resolution, TLS via `xai_grok_extra_ca`.
- Recommendations: Pin the resolved addresses into the connector (or re-check the connected peer) before treating hooks as safe. If hooks stay, amend D10 so the allowlist matches the code. Do not enable them by default.

**MCP OAuth consent URL is logged when the browser fails to open:**
- Risk: `open_consent_browser` `tracing::warn!`s `url = %auth_url`. Authorization URLs can carry short-lived secrets. The TUI never shows the URL (the TODO says stderr would corrupt the alternate screen).
- Files: `crates/codegen/xai-grok-mcp/src/oauth.rs`
- Current mitigation: Tokens on disk go to `$DEEPSEEK_BUILD_HOME/mcp_credentials.json` with Unix mode `0o600` (`crates/codegen/xai-grok-mcp/src/credentials.rs`). `Debug` for the store prints only `entry_count`. Non-Unix `ensure_owner_only_permissions` is a no-op.
- Recommendations: Log the server name, not the URL. Surface the URL through an ACP notification (the existing TODO). On Windows, set an owner-only ACL before writing `mcp_credentials.json`.

**API key handling (current controls, still easy to regress):**
- Risk: Auth is env-only (`DEEPSEEK_API_KEY`, fallback `DEEPSEEK_BUILD_API_KEY`). A key in a log, session file, or panic text violates D13.
- Files: `crates/codegen/xai-grok-sampler` (`SamplerConfig` `Debug` redaction, per `KNOWN-ISSUES.md`), `scripts/key-leak-check.sh`
- Current mitigation: Redacting `Debug`, header-conversion errors that do not echo the key, `scripts/key-leak-check.sh`. No `.env` file is present in the repo.
- Recommendations: Keep the leak check on any change to sampler logging or session persistence. Do not add a key file or an interactive key prompt.

**Plugin installs are unpinned by default:**
- Risk: Marketplace sources are empty unless configured (`load_sources` returns an empty vec). When a user adds a git source, `require_sha` defaults off (`crates/codegen/xai-grok-plugin-marketplace/src/config.rs`), so a branch tip can change under an install. `DEEPSEEK_BUILD_MARKETPLACE_REQUIRE_SHA=1` or `[marketplace] require_sha = true` is tighten-only.
- Files: `crates/codegen/xai-grok-plugin-marketplace/src/config.rs`, `crates/codegen/xai-grok-shell/src/extensions/marketplace.rs` (purges legacy default-skills installs)
- Current mitigation: No baked official xAI marketplace source. Legacy default-skills installs are purged.
- Recommendations: Default `require_sha` on for new installs. Leave existing unpinned catalogs working only behind an explicit opt-out if that migration is required.

**Egress observation is not kernel-grounded here:**
- Risk: `scripts/egress-check.sh` is an `LD_PRELOAD` logger. It misses raw `syscall(SYS_connect)`, `io_uring`, static or setuid children, and children that scrub `LD_PRELOAD`. Allowlisting by resolved IP accepts any other name on the same address. `strace` is not assumed present.
- Files: `scripts/egress-check.sh`, `scripts/check-egress.sh`, `KNOWN-ISSUES.md`
- Current mitigation: Static `scripts/check-egress.sh --strict` (recorded exit 0). Runtime check last recorded 2026-10-07 on the author's machine only (`EGRESS_VIOLATIONS=0`: `api.deepseek.com:443`, the `web_fetch` host, AF_UNIX).
- Recommendations: Re-run the runtime check on this tree before calling egress done. Where `strace` exists, use `strace -f -e trace=connect,sendto,sendmsg` as the ground truth D3 asks for.

**`web_fetch` is a deliberate extra destination:**
- Risk: D4 keeps permission-gated `web_fetch`. SSRF policy blocks non-public addresses, including `198.18.0.0/15`, unless `allow_local` is set, and even then only explicit loopback hosts (`crates/codegen/xai-grok-tools/src/implementations/grok_build/web_fetch/ssrf.rs`).
- Files: that `ssrf.rs`, `crates/codegen/xai-grok-egress-proxy/src/lib.rs` (local HTTP/CONNECT proxy; starting it does not confine child networking)
- Current mitigation: Permission ask (D4), SSRF address classes, egress proxy that refuses to be treated as enforcement by itself.
- Recommendations: Do not add `web_search` back. Do not document the egress proxy as a sandbox. Child confinement stays in `xai-grok-sandbox`.

## Performance Bottlenecks

**Workspace builds are large even after the lean profile:**
- Problem: A fresh `cargo build --workspace --tests` was measured at about 24 GB and 7 minutes (2026-10-12, recorded in `AGENTS.md` and `TESTING.md`). The previous incremental settings accumulated 244 GB under `target/debug`.
- Files: `.cargo/config.toml` (`[build] incremental = false`), `Cargo.toml` `[profile.dev]` (`incremental = true`, overridden by the cargo config), `[profile.dev] debug = "line-tables-only"`, `[profile.dev.package."*"] debug = false`, `scripts/disk-guard.sh` (default 150 GB free)
- Cause: 95 workspace members, heavy TUI/shell crates, and test binaries. Incremental caches are disabled on purpose so they cannot grow without bound; that makes rebuilds slower.
- Improvement path: One shared `CARGO_TARGET_DIR`. Per-crate `cargo test -p` during work. `scripts/disk-guard.sh` before a workspace build; stop if it fails. `cargo clean` or `rm -rf target/debug` after a gate run. Do not drop `[build] incremental = false` and do not raise debug info without a disk plan. A later tag-line commit lowered the guard to 60 GB; this tree still requires 150 GB — keep the stricter guard until a measured plan says otherwise.

**Release binary size:**
- Problem: Last measured `target/release/deepseek-build` is about 147 MiB (`TESTING.md`, `AGENTS.md`). Distribution builds are supposed to use `--profile release-dist` (thin LTO, `codegen-units = 1`), which is slower (~2.2x vs default `--release` per the `Cargo.toml` comment) and was not what produced the 147 MiB figure.
- Files: `Cargo.toml` `[profile.release]`, `[profile.release-dist]`, `crates/codegen/xai-grok-pager-bin`
- Cause: Large static binary (bundled ripgrep on release, TUI, tools, sandbox). A later tag-line note records an LTO size attempt that OOM'd (fat) or timed out (thin); that attempt is not in this tree.
- Improvement path: Measure `release-dist` separately from `--release`. Do not turn on fat LTO on a 16 GB machine. Size work belongs on a machine with a disk and RAM plan.

**Git status/diff walks:**
- Problem: Concurrent libgit2 `statuses()` / diff walks on a shallow monorepo pin CPU and tens of GB.
- Files: `crates/codegen/xai-grok-workspace/src/session/git_gate.rs`, `crates/codegen/xai-grok-workspace/src/restore_fetch.rs`
- Cause: Pack `pread` under a process mutex. The gate joins identical in-flight work and keeps a 1s snapshot (`MAX_SLOTS` 64). Restore fetch is a targeted `git fetch` of one SHA, not a full unshallow, with a 60s budget.
- Improvement path: Keep the gate. Do not replace it with unbounded `git fetch origin`. If the flake returns, fix the test clock before raising the timeout.

**Deep session futures and stack size:**
- Problem: `.cargo/config.toml` sets `RUST_MIN_STACK=67108864` because session turn futures overflow libtest's 2 MiB thread. Production session and ACP runtime threads are 8 MiB (`SESSION_THREAD_STACK_SIZE` / `RUNTIME_THREAD_STACK_SIZE` in `crates/codegen/xai-grok-shell/src/session/acp_session_impl/spawn.rs` and `crates/codegen/xai-grok-pager/src/acp/spawn.rs`).
- Files: those spawn sites, `.cargo/config.toml`
- Cause: Large async state machines, not a hot loop.
- Improvement path: Do not shrink `RUST_MIN_STACK` to make tests "faster". If a production turn overflows 8 MiB, `panic = "abort"` kills the process — box the deep turn future instead of raising every thread to 64 MiB.

## Fragile Areas

**God modules:**
- Files: `crates/codegen/xai-grok-workspace/src/permission/manager/mod.rs` (~334 KB), `crates/codegen/xai-grok-pager/src/views/extensions_modal.rs` (~320 KB), `crates/codegen/xai-grok-pager/src/app/app_view.rs` (~266 KB), `crates/codegen/xai-grok-pager/src/app/event_loop.rs` (~248 KB), `crates/codegen/xai-grok-tools/src/implementations/grok_build/bash/mod.rs` (~261 KB), `crates/codegen/xai-grok-mcp/src/servers.rs` (~224 KB), `crates/codegen/xai-grok-shell/src/agent/config.rs` (~217 KB), `crates/codegen/xai-grok-tools/src/computer/local/terminal.rs` (~220 KB)
- Why fragile: Permission mode, bash gating, and the TUI event loop share files with their tests and unrelated arms. A small edit recompiles a huge crate and is easy to land in the wrong `impl`.
- Safe modification: Change one behavior, add a focused test next to the existing module tests, and run `cargo test -p <crate>` rather than a workspace suite. Do not split these files unless that split is the task.
- Test coverage: The pager and shell crates have large in-module test files (several over 250 KB). Coverage is broad and slow, not absent. The gap is the deleted-hub integration path (below), not these modules having zero tests.

**Sandbox integration after the hub deletion:**
- Files: `crates/codegen/xai-grok-workspace/src/handle.rs` (`create_local_harness` — "no hub connection needed"), `crates/codegen/xai-grok-workspace/src/sandbox/real_wiring_tests.rs`, `docs/DROPPED-TESTS.md`
- Why fragile: The old tests dispatched through `SessionRoutedToolHandler::handle_call` (pre-run approval, mode-layer write guard, sandbox pin). That dispatch is gone. The local harness runs the toolset directly and skips that integration. Retained tests cover `off`, enforce-without-backend refusal, daemon-off, mode-verb engage, and proxy-off.
- Safe modification: Add a local-harness test that asserts the permission gate and the sandbox pin still wrap a shell call. Do not restore `crate::hub`.
- Test coverage: Explicitly dropped. This is the coverage gap to re-home.

**Plan mode headless wiring vs the written bug:**
- Files: `crates/codegen/xai-grok-pager/src/headless.rs` `apply_headless_permission_mode`, `crates/codegen/xai-grok-shell/src/session/plan_mode.rs`, `crates/codegen/xai-grok-shell/src/session/acp_session_impl/session_mode.rs`
- Why fragile: `KNOWN-ISSUES.md` still says `--permission-mode plan` does not send `session/set_mode`, write `plan_mode.json`, or inject the reminder. This tree's headless path sends `SetSessionModeRequest` for `plan` and the comment says that is what arms the reminder and the edit gate. Interactive plan approval is still only pty-scripted (`crates/codegen/xai-grok-pager-pty-harness/src/scenarios/plan_approval_resume.rs`).
- Safe modification: Do not "fix" the headless gap by adding a second set-mode call. Verify with a headless run, then update `KNOWN-ISSUES.md`. A human TUI pass is still required for the prompt UI.
- Test coverage: Unit/pty coverage exists. No human interactive pass (below).

**Config and prompt edit traps:**
- Files: listed under Tech Debt (config reference, encrypted templates).
- Why fragile: The generated or hand-synced artifact is what ships, not the file a reader edits first.
- Safe modification: Follow the regenerate / `config_docs` steps in the same change.
- Test coverage: `test_encrypted_templates_not_stale` and `config_docs` tests. They only help if they are run.

## Scaling Limits

**Disk:**
- Current capacity: Lean workspace test build ~24 GB. Guard refuses work under 150 GB free (`scripts/disk-guard.sh`).
- Limit: Filling the filesystem mid-compile. A second `target/` in a worktree doubles that.
- Scaling path: Shared `CARGO_TARGET_DIR`. Stop when the guard fails. Do not enable incremental caches to "go faster".

**Release CI:**
- Current capacity: `.github/workflows/release.yml` builds four native targets and can push a GHCR image. On the runs for `v0.0.1-rc2` (failure), `v0.0.1-rc3` (failure), `v0.0.1-rc4` (cancelled), and `v0.0.1-rc5` (cancelled), macOS arm64 was the job that did not finish, and both "Publish GitHub release" and "Build and push container image" were skipped or never reached. `gh release view v0.0.1-rc5` still shows a pre-release with only the two Linux tarballs, `install.sh`, and `SHA256SUMS` (no macOS tarballs). Package listing was not permitted from this environment (HTTP 403/404); the skipped container job is the evidence that those runs did not push an image.
- Limit: A cancelled arm64 job skips publish. Partial Linux assets can exist beside a skipped publish job (rc5).
- Scaling path: Fix the arm64-darwin job (the failure/cancel point on every rc run) before tagging again. Do not point users at rc5 as a build of this branch.

**Git object walks:**
- Current capacity: 64 concurrent gate slots, 1s snapshot, 60s restore-fetch budget, depth-1 only on already-shallow clones.
- Limit: A shallow monorepo status walk that misses the snapshot still holds the process mutex and memory.
- Scaling path: Stay on the targeted fetch. Do not raise the budget to hide a hung `git`.

**Token and cost accounting:**
- Current capacity: Estimates use the 2026-10-07 peak table (`deepseek-flash` 0.006/0.30/1.20, `deepseek-v4-pro` 0.044/1.32/3.96 USD per 1M). Unnamed models fall back to v4 Pro, not zero (D14).
- Limit: A price change on the provider is invisible. Peak pricing overstates off-peak spend by 2x.
- Scaling path: Config override. No runtime fetch of the pricing page.

## Dependencies at Risk

**Rust 1.94.0 pin:**
- Risk: `rust-toolchain.toml` pins 1.94.0 (D8). Clippy and the crate graph are validated on that compiler only.
- Impact: A bump can fail `cargo fmt` / clippy or change behavior. `clippy.toml` still carries `large-error-threshold = 256` for a tonic issue.
- Migration plan: Do not bump. `tonic` remains a dependency of tools/tools-api/workspace/login (`KNOWN-ISSUES.md`); do not add new gRPC services on it.

**Bundled ripgrep 15.0.0:**
- Risk: Release builds download that exact upstream release (see Security).
- Impact: A missing or changed GitHub asset fails `cargo build --release -p xai-grok-shell`.
- Migration plan: Vendor plus checksum, or a required local path.

**`jsonschema` external `$ref`:**
- Risk: Re-enabling `resolve-http` / `resolve-file` would let workflow output schemas fetch URLs.
- Files: `crates/codegen/xai-grok-shell/src/session/workflow/schema_contract.rs` (`RejectExternalSchemaRefs`, 256 KiB schema cap)
- Impact: A dependency feature change becomes an outbound fetch.
- Migration plan: Keep those features off. Keep every schema compile on `compile_contract_schema`.

**Legal / license (O1):**
- Risk: Aggregate distribution as `deepseek-build` while upstream files stay Apache-2.0 has no legal sign-off (`DECISIONS.md` O1). `cargo-deny` is recorded clean; libgit2 is GPL-2.0-only WITH libgit2-linking-exception (`docs/LICENSE-AUDIT.md`).
- Impact: A public release before sign-off is the open product risk, not an engineering blocker.
- Migration plan: Do not relicense upstream files. Get O1 before a non-rc public release. Keep `LICENSE`, `NOTICE`, `THIRD-PARTY-NOTICES`, `third_party/NOTICE`.

## Missing Critical Features

Phase 4 items in `AGENTS.md` §3 are present in this tree. They are not unimplemented. The gap is exercise and one coverage hole, not absence:

| Item | Where it lives | What is actually missing |
|------|----------------|--------------------------|
| Plan mode | `xai-grok-shell/src/session/plan_mode.rs`, headless `apply_headless_permission_mode` | Human interactive confirmation. The written "flag does nothing" bug does not match this tree. |
| Permission modes and dangerous-command detection | `xai-grok-shell/src/util/config/permissions.rs`, `xai-grok-workspace/src/permission/auto_mode/security_findings.rs` | Interactive prompts were not exercised by a person. Headless cancels instead of prompting. |
| Read-before-write | `xai-grok-tools/src/implementations/editor_infra/read_before_write.rs` (used by opencode edit/write and codex apply_patch) | Nothing structural. Keep new write tools on this helper. |
| Compaction | `crates/common/xai-grok-compaction`, checkpoint files under session `compaction_checkpoints/` | Re-run on a non-author machine. |
| Session resume | `xai-grok-pager/src/headless.rs` `resume_session_in_new_worktree` | Same. |
| Subagents | `xai-grok-shell/src/agent/subagent/mod.rs` | Flash-for-research is not automatic while routing is unwired. |
| MCP client | `crates/codegen/xai-grok-mcp` | OAuth URL is log-only if the browser fails (above). |
| AGENTS.md loading | `xai-grok-tools/src/types/agents_md_tracker.rs`, `xai-grok-workspace-types/src/rpc/agents_md.rs`, compaction re-inject | Nothing structural. |
| Checkpoints / undo | compaction checkpoint notifications in `xai-grok-shell/src/extensions/notification.rs`; `[ui.contextual_hints].undo` defaults on | Not the same as a user-facing undo of every tool. Do not assume a full undo product from the hint flag. |
| Headless mode | `xai-grok-pager/src/headless.rs` | Cannot prompt. Cost is a local estimate. |

**Not a missing feature, but not shipped:**
- Problem: No successful four-target release and no container image from the runs above. `docker/Dockerfile` expects staged release tarballs. `docker/Dockerfile.localtest` has not been built in this checkout.
- Blocks: Installer-from-GitHub and the README GHCR examples for *this* branch. rc5's Linux tarballs are a different commit (`4b305bd0`).

**Legal sign-off (O1)** blocks a public non-rc release, not local use.

## Test Coverage Gaps

**Full workspace suite not re-run on this tree since 2026-10-09:**
- What's not tested: The recorded 31,939 / 2 / 388 result predates nothing in this HEAD (HEAD is 2026-10-12, and the 2026-10-09 run was described as in-flight on an ancestor). No newer workspace result is recorded in `KNOWN-ISSUES.md`. `TESTING.md` §6 is older and wrong (login OIDC failures).
- Files: workspace `Cargo.toml`, `KNOWN-ISSUES.md`, `TESTING.md`
- Risk: Quoting either document as "current green/red" is false. Per-crate passes miss feature-unification tests (`KNOWN-ISSUES.md` cites `xai-fast-worktree`).
- Priority: High before any release tag from this branch. Run `scripts/disk-guard.sh` first. Expect the two bwrap failures on a host with root-owned `/run/containerd`; do not "fix" them by deleting the tests.

**Sandbox pre-run integration:**
- What's not tested: Permission gate + sandbox pin + mode-layer write guard on one shell call through the local harness.
- Files: `crates/codegen/xai-grok-workspace/src/handle.rs`, `src/sandbox/real_wiring_tests.rs`, `docs/DROPPED-TESTS.md`
- Risk: A refactor can drop the gate and the remaining tests still pass.
- Priority: High.

**Interactive TUI by a human:**
- What's not tested: Welcome logo, `/model`, `/effort`, `/think`, status line, plan approval, and permission prompts have been driven under a pty (2026-10-07, `TESTING.md` §4) and not by a person. `crates/codegen/xai-grok-pager-pty-harness` is the scripted stand-in.
- Files: `TESTING.md` §4, `crates/codegen/xai-grok-pager-pty-harness/`
- Risk: Layout, focus, and prompt bugs that pty scripts do not cover.
- Priority: High for a personal tool that is primarily a TUI. Not a substitute for the unit suite.

**Release workflow and container:**
- What's not tested: A green end-to-end release. The workflow has run and has not succeeded (see Scaling Limits). `scripts/test-install.sh` is 39/39 against a loopback fake release only.
- Files: `.github/workflows/release.yml`, `docker/Dockerfile`, `docker/Dockerfile.localtest`, `scripts/test-install.sh`, `install.sh`
- Risk: Shipping rc5's partial Linux assets, or tagging this branch and repeating the arm64 cancel.
- Priority: High before telling anyone to install from GitHub. Medium for local use of a `cargo build --release` binary.

**Live scripts off the author's machine:**
- What's not tested: `scripts/smoke-test.sh` (recorded 8/8 on 2026-10-07) and `scripts/egress-check.sh` (`EGRESS_VIOLATIONS=0` the same day) have no second host.
- Files: `scripts/smoke-test.sh`, `scripts/egress-check.sh`
- Risk: Adapter or egress regressions that wiremock does not catch. Wiremock lives in `crates/codegen/xai-grok-sampler/tests/chat_completions_wire.rs`.
- Priority: Medium. Needs a key that must not be written into the repo or the transcript.

**Model routing:**
- What's not tested: Any production caller. Unit tests in `model_routing.rs` cover the pure function only.
- Files: `crates/codegen/xai-grok-sampler/src/model_routing.rs`
- Risk: Turning the policy on in config does nothing.
- Priority: Medium if routing is a goal; Low if `/model` remains the product.

---

*Concerns audit: 2026-11-21*
