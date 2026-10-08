# TESTING — deepseek-build

A manual checklist for the personal `deepseek-build` agent. Every step states
what to run and what to expect; expected results are exact strings, exit codes,
or observable state.

- **Core pass (§1–§2, §4):** ~20–30 minutes with a warm `target/` — build check,
  first run + missing-key path, and the interactive checklist.
- **Live checks (§3) and installer/container/release (§5):** another ~25–40
  minutes; §3 needs a real key, §5 does not.

> Status (2026-10-09): the release build is green and the binary runs; all test
> targets compile, but a full `cargo test --workspace` run is **not green yet**
> (`xai-grok-login --lib` currently 405 passed / 33 failed / 1 ignored under
> investigation — see [`KNOWN-ISSUES.md`](KNOWN-ISSUES.md)). The live DeepSeek
> smoke test and the runtime egress audit were last green on the author's machine
> (2026-10-07). Four surfaces have never been exercised: see
> [Not exercised yet](#not-exercised-yet).

## 0. Prerequisites (once)

- Rust **1.94.0** via rustup (pinned by `rust-toolchain.toml`; do not bump) and
  `protoc` on `PATH` (e.g. `/usr/bin/protoc`).
- For the live egress script: a C compiler (`cc`) and `python3`.
- For §5: Docker, and `git`/`gh` for the release dry run.
- A real key, exported in your shell — **never** paste it into a prompt, a
  tracked file, or a transcript:
  `export DEEPSEEK_API_KEY=sk-...` (fallback name `DEEPSEEK_BUILD_API_KEY`).

## 1. Build (3 min once toolchain is warm)

| # | Run | Expect |
|---|-----|--------|
| 1 | `source "$HOME/.cargo/env"` | no output |
| 2 | `cargo build --release -p xai-grok-pager-bin` | exit 0, ends `Finished release ...` (a cold first build takes 10+ min of compile time — not counted in the 20–30 min pass) |
| 3 | `ls -lh target/release/deepseek-build` | an executable; last measured 147M (153,624,024 bytes) |
| 4 | `./target/release/deepseek-build --version` | `deepseek-build 1.0.45 (<12-hex commit>)`, exit 0 |

## 2. Key handling and first run (3 min)

| # | Run | Expect |
|---|-----|--------|
| 5 | `[ -n "${DEEPSEEK_API_KEY:-}" ] && echo key-present` (or check `DEEPSEEK_BUILD_API_KEY`) | `key-present`; if nothing prints, export the key before continuing |
| 6 | Missing-key path, no key involved: `T=$(mktemp -d); env -u DEEPSEEK_API_KEY -u DEEPSEEK_BUILD_API_KEY HOME="$T/home" DEEPSEEK_BUILD_HOME="$T/dbh" ./target/release/deepseek-build -p hi; echo "exit=$?"; rm -rf "$T"` | stderr `Not signed in: set DEEPSEEK_API_KEY (or DEEPSEEK_BUILD_API_KEY) to authenticate.`; `exit=1`; fail-closed before any model request (verified 2026-10-09 against a release binary built at `46aef092`) |
| 7 | Fresh first run: `BIN="$PWD/target/release/deepseek-build"; T=$(mktemp -d); (cd /tmp && HOME="$T/home" "$BIN"); rm -rf "$T"` | `$T/home/.deepseek-build/` (or `$DEEPSEEK_BUILD_HOME/`) is created with `sessions/`, `docs/user-guide/` (28 files), `config.toml`, `logs/`; the TUI starts with the DeepSeek whale welcome logo. With the key exported: "Logged in with API key" and the prompt accepts input. Without a key: stays on the welcome screen with a one-line sign-in error (last observed: `OIDC not configured`). There is **no interactive key entry** — quit and export the key instead. `q` quits cleanly. |

## 3. Live scripts (8–12 min; a real key needed for steps 8, 9 and 13)

| # | Run | Expect |
|---|-----|--------|
| 8 | `scripts/smoke-test.sh` | 8 checks, `8 passed, 0 failed`, exit 0. Covers: `/models` lists `deepseek-v4-pro` + `deepseek-flash`; plain + streamed chat; a 3-turn thinking tool chain (`num_turns >= 3`); usage carries `reasoning_tokens` + cache fields; `deepseek-flash` and the hidden `deepseek-v4-flash` alias accepted; `cached_tokens == prompt_cache_hit_tokens` on a cache hit. Without a key: `FAIL  API key present — set DEEPSEEK_API_KEY or DEEPSEEK_BUILD_API_KEY`, then `0 passed, 1 failed`, exit 1. The key is never printed. |
| 9 | `scripts/egress-check.sh` (5–10 min) | `EGRESS-CHECK PASS`, `EGRESS_VIOLATIONS=0`, exit 0. Runs a real model-driven session (shell, edit, `web_fetch`, local MCP, forced compaction, resume) under an LD_PRELOAD connect/send logger; the destination table shows only `api.deepseek.com:443`, the `web_fetch` host, loopback, and AF_UNIX. Without a key it exits 2 with `FAIL API key missing`. LD_PRELOAD limits apply — see `KNOWN-ISSUES.md`; `strace -f -e trace=connect,sendto,sendmsg` is the kernel-level alternative where available. |
| 10 | `scripts/key-leak-check.sh` (no real key) | `PASS fake key absent from N saved file(s) and captured output`, exit 0. Runs the binary with a fake key in an isolated `HOME` and scans logs/sessions/stdout/stderr. |
| 11 | `scripts/check-egress.sh --strict` (no key) | `HARD egress gate: OK`, exit 0, no SOFT branding hits. |
| 12 | `scripts/sandbox-run.sh --version` (no key) | prints the version; needs no network. |
| 13 | `scripts/sandbox-run.sh -p "print hello"` (key) | the session completes; any destination other than the provider + loopback is refused and printed as `sandbox-run: BLOCKED connect ...`. To see the `web_fetch` host refused, ask it in the same session to fetch `https://example.com/` (IPv6 refusals print as `non-ip` — cosmetic gap; `web_fetch` must be enabled) |

## 4. Interactive checklist (12–18 min; a real key)

Work in a scratch git repo: `mkdir -p /tmp/dsb-scratch && cd /tmp/dsb-scratch && git init`.
Launch the binary and approve tool calls as prompted.

| # | Do | Expect |
|---|----|--------|
| 14 | Ask: "create hello.py that prints hello" | a file-write tool call → permission prompt → after approve the file exists and a diff is shown |
| 15 | Multi-file refactor: "rename `mul` to `multiply` everywhere in this repo" | every affected file edited; `grep -rn 'mul(' .` finds nothing; any tests still pass |
| 16 | Plan mode: `Shift+Tab` until Plan (or `/plan`) | the mode indicator changes; the agent investigates read-only and asks before acting; launching with `--no-plan` disables it. Known gap: headless `--permission-mode plan` is read-only but does not emit a plan for a bare edit request |
| 17 | Permission modes: `Shift+Tab` cycling, `/auto`, and `--allow`/`--deny`/`--permission-mode` flags | modes change behavior; a dangerous `rm -rf`-class command still prompts with a warning even in auto modes |
| 18 | Reasoning: `/think` | toggles every reasoning block in the transcript (same toggle as `Ctrl+E`). Fold keys: `e` folds/unfolds the selected entry, `E` every entry, `Ctrl+E` all thinking blocks |
| 19 | `/model` (alias `/m`) | menu lists `deepseek-v4-pro` + `deepseek-flash`; the hidden `deepseek-v4-flash` alias is not shown but is accepted by `-m deepseek-v4-flash`; the switch changes the model shown |
| 20 | `/effort <level>` | offers `none`/`low`/`high`/`max`; the next turn honors it (`none` stops reasoning) |
| 21 | `/usage` (alias `/cost`) | opens the usage modal (tabs Context usage / Usage limit / Session info); the session block shows `Input tokens: N (M cached)`, `Output tokens: N (R reasoning)`, `Total tokens`, `Model calls`, and `Cost:` (currently `not available (not reported)` — the provider reports no cost). In minimal mode the same block prints to the scrollback |
| 22 | Status line: add `[ui.status_line]` with `type = "builtin"` and `items = ["cwd","model","context","effort","tokens","cache","cost","turn-timer","session-name"]` to `~/.deepseek-build/config.toml`, restart | the row renders those segments. Item names are kebab-case (`turn-timer`, `session-name`) |
| 23 | Resume: quit and relaunch with `-c` (most recent for this directory) or `-r <id>`, or use `/resume` inside the TUI | history reloads; `-r <id>` reuses the same session id |
| 24 | Headless: `./target/release/deepseek-build -p "Reply with exactly: pong"` | reply on stdout, exit 0 |
| 25 | Headless JSON: `./target/release/deepseek-build -p "say hi" --output-format json` | one JSON object with `text`, `stopReason`, `sessionId`, `requestId`; when tokens were recorded: `usage` (`input_tokens`, `cache_read_input_tokens`, `cache_creation_input_tokens`, `output_tokens`, `reasoning_tokens`, `total_tokens`), `num_turns`, `modelUsage.<model>.modelCalls`, plus `estimated_cost_usd` and `"estimate": true` (local peak-rate estimate, D14). `total_cost_usd` appears only when the provider reported a complete cost — the DeepSeek API currently reports none, so it is omitted; `usage_is_incomplete` / `cost_is_partial` flag degraded payloads |
| 26 | Headless permission cancel: `./target/release/deepseek-build -p "run rm -rf /tmp/dsb-does-not-exist"` (no `--always-approve`) | the gated tool is cancelled in headless mode (no TTY to prompt); stderr carries the one-line hint naming `--always-approve` and `--permission-mode auto`; exit is non-zero. Re-run with `--always-approve` to see it proceed |
| 27 | MCP: add a small stdio server to `~/.deepseek-build/config.toml` (`[mcp_servers.echo]` with `command`/`args`), relaunch, run `/mcps` | the server shows connected; its tool is listed to the model and can be called; a broken server shows an error rather than hanging |
| 28 | `web_fetch` ask-gate: enable `[features] web_fetch = true`, keep the default domain allowlist, then ask "fetch https://example.com/ and give me the page title" | a permission prompt naming the URL appears first (the host is not on the static allowlist); approve → the page is fetched; deny → the tool is refused and the turn continues |

## 5. Installer, container, release (10–15 min; no key)

| # | Run | Expect |
|---|-----|--------|
| 29 | `sh scripts/test-install.sh` | `ALL PASS (25 assertions)`, exit 0. Builds a fake release tree and serves it on 127.0.0.1; covers success, latest-resolution, checksum mismatch (installs nothing), unsupported OS, tarball 404, uninstall, rerun-update, unwritable dir. No network beyond loopback. |
| 30 | Container — production `docker/Dockerfile`: **NOT YET EXERCISED.** It expects a staged context assembled from the CI release tarballs (`deepseek-build-amd64` / `deepseek-build-arm64` + `LICENSE`, `NOTICE`, `THIRD-PARTY-NOTICES`, `README.txt`). The image is `debian:bookworm-slim`, runs as uid 1000, and `ENTRYPOINT`s `deepseek-build` with licenses in `/usr/share/doc/deepseek-build/`. Once a release exists: `docker buildx build --platform linux/amd64 -f docker/Dockerfile -t deepseek-build:test <ctx>` | image builds; `docker run --rm deepseek-build:test --version` prints the version |
| 31 | Container — local smoke (`docker/Dockerfile.localtest`): stage a directory with your built binary copied as `deepseek-build`, plus `LICENSE`, `NOTICE`, `THIRD-PARTY-NOTICES` and a `README.txt`, then `docker build -f docker/Dockerfile.localtest -t deepseek-build:localtest .` — **NOT YET EXERCISED** | `docker run --rm deepseek-build:localtest --version` works; `docker run -it --rm -e DEEPSEEK_API_KEY -v "$PWD":/work deepseek-build:localtest` starts the TUI |
| 32 | Release dry run — **NOT YET EXERCISED in CI.** Follow the test-tag procedure in [`AGENTS.md`](AGENTS.md) §6 (`v0.0.1-rc1`). No `v*` tag has ever been pushed and `gh release list` is empty (checked 2026-10-09) | all 4 native builds succeed; each tarball has `deepseek-build` + `LICENSE` + `NOTICE` + `THIRD-PARTY-NOTICES` + `README.txt` at top level; `SHA256SUMS` lists all 4; the release gets 4 tarballs + `SHA256SUMS` + `install.sh`; the installer-installed binary prints the tag version |

## Not exercised yet

> - **Interactive TUI by a human.** It has been driven under a pty (scripted keystrokes, 2026-10-07), never used by a person.
> - **Release workflow in CI.** Never run; no `v*` tag pushed; no GitHub release exists.
> - **Container image.** Never built or pulled, by CI or locally.
> - **Live scripts on any machine but the author's.** `smoke-test.sh` (8/8) and `egress-check.sh` (`EGRESS_VIOLATIONS=0`) passed there on 2026-10-07 and have not been reproduced elsewhere.

## 6. Test suite status

- `cargo check --workspace --tests` is green (all test targets compile). A full
  `cargo test --workspace` is **not green yet**: `xai-grok-login --lib` alone is
  currently **405 passed / 33 failed / 1 ignored** (scratch `HOME`/`DEEPSEEK_BUILD_HOME`,
  2026-10-08 23:54), clustered in `refresh::oidc_refresher` (16),
  `manager::tests` (9), `flow`, `external_auth`, `storage` — the inert OAuth/OIDC
  scaffolding; another workstream is investigating. The earlier four
  `xai-fast-worktree` NFS/Grove failures were dropped (tests of a backend this
  tree stubs off) — see [`KNOWN-ISSUES.md`](KNOWN-ISSUES.md) and
  `docs/DROPPED-TESTS.md` before quoting numbers.
- Latest recorded per-crate runs, all 0 failures: `xai-grok-pager` 9876 (+4
  ignored), `xai-grok-pager-minimal` 94, `xai-grok-shell` 6221,
  `xai-grok-workspace` 1693, `xai-grok-sampler` 186, `xai-grok-config` 467,
  `xai-chat-state` 391, `xai-grok-agent` 576, `xai-grok-status-line` 24.
- `scripts/smoke-test.sh` is the live end-to-end check.
