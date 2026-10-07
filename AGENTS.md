# AGENTS.md — deepseek-build handoff

Read this first when resuming. It records exact state, what remains, and the
commands to continue. Companion docs: [`DECISIONS.md`](DECISIONS.md) (locked
decisions; wins on conflict), [`PLAN.md`](PLAN.md) (audit + phased plan),
[`README.md`](README.md).

## 0. Snapshot

- **Branch:** `dsb/integration` (the integration branch; all work merged here).
- **Remote:** `origin git@github.com:coruairc/deepseek-build.git`.
- **Toolchain:** Rust **1.94.0** via rustup (pinned in `rust-toolchain.toml`). Do not bump.
  `protoc` on PATH (`/usr/bin/protoc`); `dotslash` not needed.
- **Binary:** `target/release/deepseek-build` (~192 MB) — package `xai-grok-pager-bin`
  (D2 keeps the internal package name). **Rebrand done**: config dir `~/.deepseek-build`,
  ACP namespace `deepseek-build/*`, Monokai theme + DeepSeek whale logo, brand constants
  in `crates/codegen/xai-grok-brand/`.
- **Egress gate:** `scripts/check-egress.sh --strict` → **exit 0** (HARD OK, SOFT zero).
- **Build:** `cargo check -p xai-grok-pager-bin` and `cargo build --release` are green.
  **`cargo test --workspace` does not compile yet: only `xai-grok-pager` (~252 errors)
  and `xai-grok-pager-minimal` (2) lib test targets fail** on test-only references to
  deleted Phase-1 features (`xai_grok_feedback`, removed fields/methods, a 13→12
  constructor arity). Every other lib test target compiles and passes
  (shell 6221, workspace 1693, sampler 186, config 467, chat-state 391, agent 576, …).
- **Runtime egress (observed):** a real model-driven session (shell, edits, web_fetch,
  local MCP, compaction, resume) contacts only `api.deepseek.com:443`, the web_fetch
  host, and AF_UNIX sockets — `scripts/egress-check.sh` reports
  `EGRESS_VIOLATIONS=0`. Measured with an `LD_PRELOAD` connect/send logger
  (`strace` absent).
- **Status:** usable with a real `DEEPSEEK_API_KEY`. See `TESTING.md`,
  `scripts/smoke-test.sh`, `scripts/egress-check.sh`, `scripts/sandbox-run.sh`,
  `KNOWN-ISSUES.md`. Remaining: the pager/pager-minimal test-target repair; the
  local-harness sandbox/pre-run coverage gap noted in `KNOWN-ISSUES.md`; DeepSeek
  prompt tuning (`docs/TUNING.md`, not started); interactive TUI verified under a pty
  but not by a human.

## 1. How to resume (another machine)

```sh
git clone git@github.com:coruairc/deepseek-build.git
cd deepseek-build
git switch dsb/integration          # or: git switch -c dsb/integration origin/dsb/integration

# toolchain
rustup toolchain install 1.94.0 --component rustfmt clippy
source "$HOME/.cargo/env"

# sanity
cargo check -p xai-grok-pager-bin
scripts/check-egress.sh
```

Slice branches are pushed too (`dsb/telemetry`, `dsb/tools`, `dsb/autoupdate`,
`dsb/exfil`, `dsb/auth`, `dsb/misc`, `dsb/cloud`, `dsb/hard`, `dsb/hard2`);
they are already merged, so you normally stay on `dsb/integration`. To
re-parallelize, create fresh worktrees off `dsb/integration`:

```sh
git worktree add ../wt/<name> -b dsb/<name> dsb/integration
```

The slice worktrees on the original machine live under `/home/cein_orourke/wt/dsb-*`
(not part of the repo).

## 2. What is DONE

- **Phase 0:** `PLAN.md` audit + phased plan; `DECISIONS.md` locked decisions; `NOTICE`.
- **Phase 1 (partial, committed via slice branches, merged on `dsb/integration`):**
  - `mixpanel` crate deleted (`988bad45`).
  - Telemetry: xAI OTLP stack + `xai-grok-otel` deleted, Sentry sink deleted, external
    OTEL stream stubbed; `xai-mixpanel` gone (`dsb/telemetry`, `83d26ac7`).
  - xAI voice / Imagine image+video / `web_search` tools deleted; xAI plugin-marketplace
    official source removed (`dsb/tools`, `eb165260`).
  - Auto-update (`xai-grok-update`), remote announcements (`xai-grok-announcements`),
    subagent bundle (`xai-grok-bundle`) crates deleted (`dsb/autoupdate`, `6cd2277d`).
  - Upload/exfil: `xai-file-utils` deleted (GCS/S3/storage client/queue); heap_profile
    deleted; `xai-grok-workspace/src/upload` deleted; feedback/share upload handlers
    deleted (`dsb/exfil`, `bb949731`). Some shell upload/feedback paths were **stubbed**,
    not fully deleted — see §5.
  - Auth: xAI OAuth/OIDC/device-code/JWKS/refresh network stack deleted; API-key path
    (`DEEPSEEK_API_KEY` / `DEEPSEEK_BUILD_API_KEY`) added (`dsb/auth`, `3f56baf7`).
  - Misc: announcement banner UI + SuperGrok billing/upsell removed; computer-hub **not**
    removed (`dsb/misc`, `ef7e4d89`).
  - `xai-grok-cloud-config` crate deleted, code vendored inert into
    `xai-grok-shell/src/cloud_config/` (`dsb/cloud`, `02a551c0`).
  - **HARD egress gate cleared to zero** across production + tests
    (`dsb/hard`, `f0c2d43d`; `dsb/hard2`, `b68327ca`).
  - `prod/mc/cli-chat-proxy-types` renamed to `prod/mc/model-api-types`.
- **Endpoints foundation (me):** compiled endpoints → `https://api.deepseek.com` with
  loopback relay/gateway placeholders (`xai-grok-env`, `xai-grok-config/endpoints.rs`);
  xAI model entries removed; default model `deepseek-v4-pro`, plus `deepseek-flash` and a
  hidden `deepseek-v4-flash` alias (`633d80df`).
- **Phase 2 adapter core (me):** `ThinkingConfig` + `thinking` field, thinking/effort →
  `tool_choice` downgrade, `reasoning_content` final-pass sanitizer (backfills all prior
  assistant turns when tools are present), typographic-quote tool-call JSON repair,
  `TokenUsage` cache hit/miss helpers, DeepSeek default catalog (`cdbf5aa3`, `28296f7e`).
  Tests: `cargo test -p xai-grok-sampling-types --lib` (298), sampler stream tests (13).
- **Rebrand (D1/D2) done** (`dsb/rebrand`, `8fcbff78`): brand constants in
  `crates/codegen/xai-grok-brand/`, binary `deepseek-build`, config dir `~/.deepseek-build`,
  ACP namespace `deepseek-build/*`, user-visible strings. `scripts/check-egress.sh --strict`
  exits 0 (SOFT zero). Most `GROK_*` env vars still carry the old name (not gated).
- **DeepSeek prompt guidance:** `crates/codegen/xai-grok-agent/templates/prompt.md` now includes a
  `<deepseek_model>` section (prefix-cache discipline, thinking budget, parallel-first tool use,
  verification, large-context guidance) and drops the obfuscated "released by xAI" line. The
  runtime prompt is XOR-obfuscated: after editing any template run
  `python3 scripts/encrypt_templates.py` (restored) and verify with
  `test_encrypted_templates_not_stale` (needs a compiling test suite).
- **Step 5b — release pipeline + installer (DONE, on `dsb/integration`):**
  - `.github/workflows/release.yml` — tag `v*` (and `workflow_dispatch`) builds 4 targets
    **natively** (`ubuntu-22.04`, `ubuntu-22.04-arm`, `macos-15`, `macos-15-intel`), packages
    `deepseek-build-<ver>-<target>.tar.gz` (binary + LICENSE + NOTICE + THIRD-PARTY-NOTICES +
    README.txt), generates `SHA256SUMS`, and attaches tarballs + `SHA256SUMS` + `install.sh` to
    the GitHub release. All actions pinned to SHAs. Only uploads to the GitHub release (plus
    GitHub-internal `upload-artifact` between jobs). `actionlint` clean.
  - `install.sh` (repo root, also a release asset) — POSIX sh, `set -eu`, OS/arch detection,
    downloads tarball + `SHA256SUMS` from **github.com only** (no API host), verifies SHA-256
    **before** extracting, installs to `${INSTALL_DIR:-$HOME/.local/bin}` (no sudo), PATH hint,
    `--uninstall`, rerun-to-update, prints the non-affiliation line. `shellcheck --shell=sh`
    clean; `dash -n` clean.
  - `scripts/test-install.sh` — local fake-release HTTP harness: success, latest-resolution,
    checksum mismatch (installs nothing), unsupported OS, tarball 404, uninstall, rerun-update,
    unwritable dir. **25/25 assertions pass.**
  - `deny.toml` + `docs/LICENSE-AUDIT.md` + `docs/license-deny-report.txt` — cargo-deny
    `licenses/bans/sources` all `ok`. Copyleft: only vendored libgit2 (GPL-2.0-only WITH
    libgit2-linking-exception; full text in THIRD-PARTY-NOTICES, shipped in tarballs). Weak
    copyleft MPL-2.0 crates listed in the audit. No GPL-3/AGPL/LGPL/CDDL.
  - README `## Install` section: curl one-liner, download-inspect-run alternative, VERSION
    pinning, uninstall, manual verify, supported-platform table, attribution + non-affiliation.
  - **Not yet done:** no tag has been pushed, so no release exists yet. See §6 for the test-tag
    procedure.

## 3. What is LEFT (ordered)

1. **Backend removal (D5):** delete the `Responses` and `Messages` backends and their
   wiring so only `ChatCompletions` remains. In `xai-grok-sampling-types` (`ApiBackend`,
   `conversation/responses.rs`, `conversation/messages.rs`, `rs` types) and
   `xai-grok-sampler` (`client.rs` responses/messages paths, `stream/{responses,messages}.rs`).
   This is adapter-owned — keep it single-owner.
2. ~~**Rebrand (D1/D2) → SOFT gate zero**~~ **DONE** (`dsb/rebrand`, `8fcbff78`). Remaining
   sub-item: rename the remaining `GROK_*` env vars to `DEEPSEEK_BUILD_*` (624 distinct; not
   gated).
3. **Finish real deletion of Phase-1 stubs** (see §5): shell `src/upload/*`,
   `session/repo_changes`, `feedback_manager`, feedback UI, `share`, vendored
   `shell/src/cloud_config/**`, `xai-computer-hub-{core,sdk,mcp-adapter}` and consumers,
   shell `src/remote/**` relay clients, `agent/relay.rs`.
4. **Phase 2 proof:**
   - `wiremock` suite: streaming, a **3+ turn tool-call chain replaying `reasoning_content`**,
     missing `reasoning_content` backfill, malformed tool-call JSON, partial stream,
     429/5xx retry with backoff.
   - non-stream reasoning capture + retries + clear error messages.
   - cache-stable prefix test (system prompt + tool defs byte-identical / stable order across
     turns; dynamic state at the tail).
   - **Live smoke test (STOP condition): needs `DEEPSEEK_API_KEY`** — GET `/models` to confirm
     `deepseek-v4-pro` / `deepseek-flash`, a plain chat, a streamed chat, a 3-turn thinking
     tool-call chain, and confirm `prompt_tokens_details.cached_tokens` /
     `prompt_cache_hit_tokens` and `reasoning_tokens` in usage. Fix + record any doc mismatch.
5. **Phase 3 TUI:** collapsible reasoning block (keybind), status bar (model, thinking level,
   tokens, cache hit rate, running cost from a config price table, context usage), slash
   commands/hotkeys to switch model + `reasoning_effort {none,low,high,max}` + toggle thinking,
   auto model routing (Flash/no-reasoning vs Pro/high) + manual override, per-turn/session
   cost + cache breakdown.
6. **Phase 4 (verify then fix):** plan mode, permission modes + dangerous-command detection,
   read-before-write enforcement, cache-aware compaction, session resume, subagents (Flash for
   research), MCP client, AGENTS.md loading, checkpoints/undo, headless mode.
7. **Phase 5:** release build + one-line run command, `TESTING.md` (20–30 min manual checklist),
   `scripts/sandbox-run.sh` (only `api.deepseek.com` reachable), `KNOWN-ISSUES.md`, `strace`
   runtime egress test, independent verification sub-agent.

## 4. Gates (from the task) and current status

1. `cargo build --release` — **PASS** (`target/release/deepseek-build`). `cargo test --workspace`
   — **FAIL** (tests don't compile yet). Workspace-wide clippy/fmt not re-verified.
2. HARD egress zero — **PASS**. SOFT zero after rebrand — **PASS**
   (`scripts/check-egress.sh --strict` exits 0).
3. Runtime egress test via `strace -f -e trace=connect` — **NOT RUN**.
4. Adapter wiremock + live smoke — **NOT DONE** (live needs the key).
5. Independent verification sub-agent — was run once earlier for the adapter; **re-run at the
   end**.

Stop and ask only if: a gate fails twice, a decision is not in `DECISIONS.md`, a change would
add an outbound destination, or the live smoke test needs a key not provided. **The key is
currently not provided**, so the live smoke test is blocked.

## 5. Known gotchas / incomplete (do not paper over)

- **Tests do not compile.** Many `#[cfg(test)]` modules and `tests/**` files still reference
  symbols deleted in Phase 1 (`WebSearchInput`, `MediaGenOutput`, `ImageGenConfig`,
  `emit_announcements`, `heap_profile`, `external::config`, etc.). `cargo check` (non-test) is
  green; `cargo test` is not. Budget a dedicated pass.
- **Stubs left (no network, but present):** `xai-grok-telemetry/src/{external.rs,otel_layer.rs,
  trace_context.rs}` (no-op OTEL), shell `src/file_utils_compat.rs`, shell
  `session/repo_changes/mod.rs` (pure serde types), shell `session/feedback_manager.rs`,
  shell `src/upload/**` (restored inert, routed to compat), pager voice stub
  (`pager/src/xai_grok_voice.rs`), pager `views/announcements.rs` (no-op).
- **Not removed:** `xai-computer-hub-{core,sdk,mcp-adapter}` + consumers; shell `src/remote/**`
  relay/gateway clients; `agent/relay.rs`; vendored `shell/src/cloud_config/**`; remote
  settings/campaigns types (fetch is inert).
- **`xai-grok-cloud-config`** exists only as `xai-grok-shell/src/cloud_config/` now.
- **`xai-grok-*` crate/package names remain** (decision D2); only the user-facing surface will
  be renamed.
- Do not edit `xai-grok-env`, `xai-grok-config/src/endpoints.rs`, `xai-grok-models`,
  `xai-grok-sampling-types`, `xai-grok-sampler` in a generic scrub — those are adapter/endpoint
  owned.
- `THIRD-PARTY-NOTICES`, `third_party/NOTICE`, `LICENSE`, `NOTICE` must stay.

## 6. Useful commands

```sh
source "$HOME/.cargo/env"

cargo check -p xai-grok-pager-bin --message-format=short
cargo test  -p xai-grok-sampling-types --lib
cargo test  -p xai-grok-sampler --lib stream::chat_completions
cargo build --release -p xai-grok-pager-bin

scripts/check-egress.sh           # HARD gate
scripts/check-egress.sh --strict  # + branding (post-rebrand)

# Step 5b release pipeline / installer
actionlint .github/workflows/*.yml
shellcheck --shell=sh install.sh scripts/test-install.sh
sh scripts/test-install.sh        # 25/25 assertions
cargo-deny check licenses bans sources
```

### Test-tag procedure (do NOT tag a real release until the dry run is green)

```sh
# 1. Push a throwaway tag to exercise the workflow end to end.
git tag v0.0.1-rc1
git push origin v0.0.1-rc1

# 2. Watch the run (4 native builds + publish).
gh run list --workflow=release.yml --limit 3
gh run watch "$(gh run list --workflow=release.yml --limit 1 --json databaseId -q '.[0].databaseId')"

# 3. Inspect the release it created (prerelease, because of the -rc1 suffix).
gh release view v0.0.1-rc1

# 4. Clean up: delete the release and the tag (local + remote).
gh release delete v0.0.1-rc1 --yes
git push origin :refs/tags/v0.0.1-rc1
git tag -d v0.0.1-rc1
```

What to check on the run: all 4 build jobs succeed; each tarball contains
`deepseek-build`, `LICENSE`, `NOTICE`, `THIRD-PARTY-NOTICES`, `README.txt` at top level;
`SHA256SUMS` lists all 4 tarballs; the release has 4 tarballs + `SHA256SUMS` + `install.sh`;
then run the installer against the real release:
`curl -fsSL https://github.com/coruairc/deepseek-build/releases/download/v0.0.1-rc1/install.sh | VERSION=v0.0.1-rc1 sh`
and confirm `deepseek-build --version` prints the tag version.
