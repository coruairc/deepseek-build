# AGENTS.md — deepseek-build handoff

Read this first when resuming. It records exact state, what remains, and the
commands to continue. Companion docs: [`DECISIONS.md`](DECISIONS.md) (locked
decisions; wins on conflict), [`PLAN.md`](PLAN.md) (audit + phased plan),
[`README.md`](README.md). Latest dated handoff:
[`HANDOFF-2026-10-09.md`](HANDOFF-2026-10-09.md).

`deepseek-build` is a personal, DeepSeek-first terminal coding agent forked from
[xai-org/grok-build](https://github.com/xai-org/grok-build) (Apache-2.0): a
full-screen TUI plus headless mode that reads and edits code, runs shell
commands, and talks to DeepSeek through the OpenAI-compatible Chat Completions
API. Internal crate/package names keep the upstream `xai-grok-*` prefix (D2);
only the user-facing surface is `deepseek-build`. [`README.md`](README.md) is the
public description; §7 maps the code and §8 lists the rules for changing it.

## 0. Snapshot

- **Branch:** `dsb/integration` (the integration branch; all work merged here).
  `main` is a strict ancestor — fast-forward it after pushing.
- **Remote:** `origin git@github.com:coruairc/deepseek-build.git`.
- **Toolchain:** Rust **1.94.0** via rustup (pinned in `rust-toolchain.toml`). Do not bump.
  `protoc` on PATH (`/usr/bin/protoc`); `dotslash` not needed. `strace` absent (use
  `scripts/egress-check.sh`).
- **Binary:** release binary built via `cargo build --release -p xai-grok-pager-bin`
  (artifact `target/release/deepseek-build`; ~147 MiB on linux-x86_64; package
  `xai-grok-pager-bin`, D2 keeps the internal package name). Installed on the
  author's machine at `~/.local/bin/deepseek-build` as
  `deepseek-build 1.0.45 (cc6e6948c8db)`. **Rebrand done**: config dir
  `~/.deepseek-build`, ACP namespace `deepseek-build/*`, Monokai theme + DeepSeek
  whale logo, brand constants in `crates/codegen/xai-grok-brand/`; prompt templates
  carry no Grok Build strings.
- **Egress gate:** `scripts/check-egress.sh --strict` → **exit 0** (HARD OK, SOFT zero).
- **Build:** `cargo check -p xai-grok-pager-bin` and `cargo build --release` are green.
  **All test targets compile**; the last full `cargo test --workspace --no-fail-fast`
  was **31,939 passed / 2 failed / 388 ignored across 307 targets**. The 2 failures
  are host-environment `xai-grok-sandbox --test deny_paths_e2e` bwrap failures
  (root-owned `/run/containerd`), not code — the crate is byte-identical to the
  `last-known-good-2026-10-06` tag. A full run on `cc6e6948` is in flight; see
  `KNOWN-ISSUES.md`.
- **Runtime egress (observed):** a real model-driven session (shell, edits, web_fetch,
  local MCP, compaction, resume) contacts only `api.deepseek.com:443`, the web_fetch
  host, and AF_UNIX sockets — `scripts/egress-check.sh` reports
  `EGRESS_VIOLATIONS=0`. Measured with an `LD_PRELOAD` connect/send logger
  (`strace` absent). Last verified 2026-10-07 on the author's machine.
- **Eval harness (Phase E):** `scripts/evals/` — 21 auto-checked tasks, JSONL metrics,
  comparison table, both-ways fixture validation. E2 baseline: both models 63/63,
  Pro `$0.00702`/task vs Flash `$0.00154`/task at ~97% cache hit. E3 batch done;
  results in [`docs/TUNING.md`](docs/TUNING.md) (only reasoning-effort reduction
  won; nothing became a new default).
- **Headless cost:** `--output-format json` emits `estimated_cost_usd` +
  `estimate: true` (local peak-rate estimate, D14) alongside usage.
- **Release/container:** pipeline + installer + `docker/Dockerfile` exist and
  `scripts/test-install.sh` is 39/39, but **no `v*` tag has been pushed, no release
  exists, and no container image has been built or pulled** (checked 2026-10-09).
  Dry-run procedure: §6.
- **Status:** usable with a real `DEEPSEEK_API_KEY`. See `TESTING.md`,
  `scripts/smoke-test.sh`, `scripts/egress-check.sh`, `scripts/sandbox-run.sh`,
  `KNOWN-ISSUES.md`. Remaining: confirm one full green workspace run (host-only
  sandbox failures excepted); exercise the release workflow + container via the §6
  dry run; interactive TUI driven only under a pty, never by a human; user-guide
  swept 2026-10-12 (removed-flow pages deleted, examples use the real binary name).

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
  exits 0 (SOFT zero). Legacy env-var fallbacks were removed 2026-10-12
  (`255f17d0`); the remaining `GROK_*` names are primary internals (not gated).
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
    `--uninstall`, rerun-to-update, prints the non-affiliation line. Also creates a `dsb`
    shortcut symlink to `deepseek-build` (never clobbering a real file). `shellcheck --shell=sh`
    clean; `dash -n` clean.
  - `scripts/test-install.sh` — local fake-release HTTP harness: success, latest-resolution,
    checksum mismatch (installs nothing), unsupported OS, tarball 404, uninstall (removes the
    `dsb` symlink, preserves a foreign one), rerun-update, unwritable dir, pre-existing `dsb`
    file or foreign `dsb` symlink. **39/39 assertions pass.**
  - `deny.toml` + `docs/LICENSE-AUDIT.md` + `docs/license-deny-report.txt` — cargo-deny
    `licenses/bans/sources` all `ok`. Copyleft: only vendored libgit2 (GPL-2.0-only WITH
    libgit2-linking-exception; full text in THIRD-PARTY-NOTICES, shipped in tarballs). Weak
    copyleft MPL-2.0 crates listed in the audit. No GPL-3/AGPL/LGPL/CDDL.
  - README `## Install` section: curl one-liner, download-inspect-run alternative, VERSION
    pinning, uninstall, manual verify, supported-platform table, attribution + non-affiliation.
  - **Not yet done:** no tag has been pushed, so no release exists yet. The release
    workflow has never run in CI, and the container image has never been built or
    pulled. See §6 for the test-tag procedure.

## 3. What is LEFT (ordered)

1. ~~**Backend removal (D5)**~~ **DONE**: `ApiBackend` has only `ChatCompletions`.
2. ~~**Rebrand (D1/D2) → SOFT gate zero**~~ **DONE** (`dsb/rebrand`, `8fcbff78`). Legacy
   `GROK_*` fallbacks were removed 2026-10-12 (`255f17d0`). Remaining sub-item: rename the
   ~480 primary `GROK_*` env vars to `DEEPSEEK_BUILD_*` (counted 2026-10-12,
   `git grep -oE '\bGROK_[A-Z0-9_]+'`; not gated).
3. **Finish real deletion of Phase-1 stubs** (see §5): shell `src/upload/*`,
   `session/repo_changes`, `feedback_manager`, feedback UI, `share`, vendored
   `shell/src/cloud_config/**`, `xai-computer-hub-{core,sdk,mcp-adapter}` and consumers,
   shell `src/remote/**` relay clients, `agent/relay.rs`. All inert (no network); this is
   cleanup, not security. The dead **xAI login/OIDC surface was deleted** 2026-10-09
   (`cc6e6948`, −1,352 lines).
4. ~~**Phase 2 proof**~~ **DONE** (wiremock suite in `xai-grok-sampler/tests/`, live smoke
   8/8 PASS; see `KNOWN-ISSUES.md` "Phase 2"). Live-verify items are recorded in
   `DECISIONS.md` O3.
5. ~~**Phase 3 TUI**~~ **DONE** (reasoning folding, `/think`, `/model`, `/effort`, `/theme`,
   configurable status line incl. `cost`; see `KNOWN-ISSUES.md` "Phase 3").
6. **Phase 4 (verify then fix):** plan mode, permission modes + dangerous-command detection,
   read-before-write enforcement, cache-aware compaction, session resume, subagents (Flash for
   research), MCP client, AGENTS.md loading, checkpoints/undo, headless mode.
7. ~~**Phase 5 quality bar**~~ release pipeline + installer + docs **DONE** (see §2). Still owed:
   exercise the release workflow + container via the §6 dry run, and confirm one full
   `cargo test --workspace` run green on a clean host (the only known failures are
   host-environment bwrap ones; see §0).
8. ~~**Phase E (tuning/eval harness)**~~ **DONE**: `scripts/evals/` (21 auto-checked tasks),
   E2 baseline + E3a–d measured, results in `docs/TUNING.md`.

## 4. Gates (from the task) and current status

1. `cargo build --release` — **PASS** (artifact `target/release/deepseek-build`, ~147 MiB
   linux-x86_64). `cargo test --workspace` — all targets compile; last full run
   31,939 passed / 2 failed / 388 ignored across 307 targets, the 2 being host-only
   `xai-grok-sandbox` bwrap failures (see §0). `cargo fmt --all --check` clean;
   clippy 0 errors on the touched crates (16 pre-existing warnings).
2. HARD egress zero — **PASS**. SOFT zero after rebrand — **PASS**
   (`scripts/check-egress.sh --strict` exits 0). `opentelemetry`/`aws-config`/
   `google-cloud-storage` have zero dependents; `jsonschema`'s `$ref` fetch path closed.
3. Runtime egress test via `strace -f -e trace=connect` — **NOT RUN** (`strace` absent; the
   `LD_PRELOAD` equivalent in `scripts/egress-check.sh` is used instead, last green
   2026-10-07).
4. Adapter wiremock + live smoke — **DONE** (wiremock suite present; live smoke 8/8 PASS on
   the author's machine, 2026-10-07; re-confirmed 2026-10-09 on the merged tree).
5. Independent verification sub-agent — ran 2026-10-09 (V1/V2/V3/V4/V6 workstreams; see
   `HANDOFF-2026-10-09.md`).

Stop and ask only if: a gate fails twice, a decision is not in `DECISIONS.md`, a change would
add an outbound destination, or the live smoke test needs a key not provided.

## 5. Known gotchas / incomplete (do not paper over)

- **All test targets compile.** The last full `cargo test --workspace` was 31,939 passed /
  2 failed / 388 ignored across 307 targets; the 2 are host-environment `xai-grok-sandbox`
  bwrap failures (root-owned `/run/containerd`), not code. A full run on `cc6e6948` is in
  flight. See `KNOWN-ISSUES.md`.
- **Unexercised:** interactive TUI by a human (only pty-driven), the release workflow in CI
  (no tag, no release), the container image (never built/pulled), and the live scripts on any
  machine other than the author's. See `KNOWN-ISSUES.md` "Unexercised surfaces".
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
sh scripts/test-install.sh        # 39/39 assertions
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

## 7. Codebase map

95 workspace members: 81 product crates under `crates/codegen/`, 8 shared
building blocks under `crates/common/`, 1 protobuf helper under `crates/build/`,
the `prod/mc/model-api-types` types package, and 4 vendored crates under
`third_party/`.

| Layer | Crates | Role |
|-------|--------|------|
| Binary | `xai-grok-pager-bin` | The `deepseek-build` binary: CLI/subcommand dispatch, composition root |
| TUI | `xai-grok-pager` | Fullscreen TUI, headless single turn (`src/headless.rs`), session/plan/permission surfaces |
| TUI | `xai-grok-pager-minimal`, `xai-grok-pager-render` | Scrollback-native minimal mode (reached through the pager's fn-pointer seam) and render primitives |
| Agent runtime | `xai-grok-shell`, `xai-grok-shell-base` | ACP stdio agent (`agent/app.rs`), session/turn loop (`session/acp_session_impl/`), tool-call execution, compaction wiring, persistence |
| Conversation state | `xai-chat-state` | Actor owning message history and request assembly |
| Prompts | `xai-grok-agent` | Agent definitions/discovery and the XOR-obfuscated system-prompt templates |
| Model adapter | `xai-grok-sampler`, `xai-grok-sampling-types` | Chat Completions HTTP actor (SSE, retries, `reasoning_content` round-trip) and its wire types (`ThinkingConfig`, `TokenUsage`) |
| Catalog & endpoints | `xai-grok-models`, `xai-grok-env`, `xai-grok-config` | Embedded model catalog (DeepSeek entries), endpoint presets (`endpoints.rs` → `https://api.deepseek.com`), layered config |
| Tools | `xai-grok-tools`, `xai-tool-runtime` (+ `xai-grok-tools-api`, `xai-tool-protocol`, `xai-tool-types`) | Built-in tools and the `Tool` trait / dispatch runtime |
| Workspace & permissions | `xai-grok-workspace`, `xai-grok-permission-rules` | FS/VCS ops, permission manager, modes/rules, dangerous-command detection |
| Context | `xai-grok-compaction`, `xai-compaction-transcript` | Compaction engine and transcript store |
| Sessions | `xai-grok-session-events`, `xai-grok-active-sessions`, `xai-grok-memory` | Typed session event log, active-session registry, memory |
| Integrations | `xai-grok-mcp`, `xai-grok-hooks`, `xai-grok-plugin-marketplace` | MCP client, hook system, plugin marketplace |
| Safety & platform | `xai-grok-sandbox`, `xai-grok-extra-ca`, `xai-dirs`, `xai-grok-brand` | OS sandbox, TLS client policy, home/path resolution, brand constants |
| Auth | `xai-grok-login` | Credential plumbing; the live path is the `DEEPSEEK_API_KEY` env var |
| Test infra | `xai-grok-test-support`, `xai-grok-pager-pty-harness` | Mock inference server, ACP client, headless runner, PTY e2e scenarios |

`xai-tool-runtime`, `xai-tool-protocol`, `xai-tool-types`, and
`xai-grok-compaction` live under `crates/common/`; every other crate named above
is under `crates/codegen/`.

Data flow: binary → TUI (or a headless single turn) → ACP session in
`xai-grok-shell` → the turn loop assembles a request via `xai-chat-state` →
`xai-grok-sampler` streams from `{base_url}/chat/completions` (default
`https://api.deepseek.com`) → tool calls run through `xai-grok-tools` /
`xai-grok-workspace` under the permission system, with user-configured MCP
servers over stdio/HTTP alongside. [`PLAN.md`](PLAN.md) §1 is the long-form map,
written 2026-10-02 before the backend removal and rebrand — where it conflicts
with the code, trust the code.

## 8. Rules for changing the code

- **Toolchain:** Rust 1.94.0 is pinned (`rust-toolchain.toml`) — do not bump.
  Keep `cargo fmt --all --check` clean and clippy at 0 errors; the tree carries
  16 pre-existing warnings.
- **Clippy bans** (repo-root `clippy.toml`): use `dunce::canonicalize` instead of
  the `std`/`Path`/`tokio::fs` versions; build HTTP clients through
  `xai_grok_extra_ca`; enroll child processes via `xai_tty_utils::ProcessScope`;
  resolve home directories via `xai-dirs`. Tests may index/slice freely;
  production crates opt in to `#![deny(clippy::indexing_slicing)]`.
- **Egress invariants (D6/D10/D13):** delete exfil/telemetry code, never disable
  it; the only outbound destinations are the configured model provider and
  user-configured MCP servers; API keys never reach logs, saved sessions, panic
  text, or commits. `scripts/check-egress.sh --strict` must stay exit 0 and
  `scripts/key-leak-check.sh` green. A change that adds an outbound destination is
  a stop-and-ask.
- **Branding (D1/D2):** user-visible strings resolve through `xai-grok-brand`
  constants; internal `xai-grok-*` names stay. After touching CLI help, version,
  or other user-facing output, run the `branding_surface` integration test
  (`crates/codegen/xai-grok-pager-bin/tests/`). Env vars: `DEEPSEEK_BUILD_*` /
  `DEEPSEEK_API_KEY`; legacy `GROK_*` fallbacks were removed 2026-10-12 and are
  ignored (tests pin this).
- **Prompt templates:** the files under `crates/codegen/xai-grok-agent/templates/`
  are XOR-obfuscated into `src/prompt/prompt_encrypted.rs`; after editing any
  template run `python3 scripts/encrypt_templates.py` and keep
  `test_encrypted_templates_not_stale` green.
- **Commits and docs:** small, logical commits, one change each (DECISIONS C1);
  keep build + tests green after every merge (C3); deletions must be real
  deletions (C4). Update `KNOWN-ISSUES.md` when status changes and refresh this
  file's §0 snapshot at session end. `DECISIONS.md` wins on conflict; its
  execution rules list the stop-and-ask conditions.
