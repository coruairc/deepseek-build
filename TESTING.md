# TESTING — deepseek-build

A 20–30 minute manual test plan for the personal `deepseek-build` agent.

> Status: the non-test build is green and the binary runs. **`cargo test
> --workspace` does not compile yet — only the `xai-grok-pager` (~252 errors)
> and `xai-grok-pager-minimal` (2) lib test targets fail**, on test-only
> references to deleted Phase-1 features (`xai_grok_feedback`, removed
> fields/methods, a 13→12 constructor arity). Every other lib test target
> compiles and passes (see [`KNOWN-ISSUES.md`](KNOWN-ISSUES.md)). The live
> DeepSeek smoke test and the extended egress audit have been run and pass;
> rerun them with the scripts below.

## 1. Build (3 min)

```sh
source "$HOME/.cargo/env"          # rustup, pinned toolchain 1.94.0
cargo build --release -p xai-grok-pager-bin
ls -lh target/release/deepseek-build
./target/release/deepseek-build --version
# => deepseek-build <version> (<short sha>)
```

Expected: build succeeds; `--version` prints `deepseek-build`; no `grok` in the
top-level `--help` beyond the word "clone" in internal descriptions.

## 2. API key via environment (1 min)

```sh
export DEEPSEEK_API_KEY=sk-...      # preferred
# or: export DEEPSEEK_BUILD_API_KEY=sk-...
```

- With **no** key: `deepseek-build -p hi` prints exactly
  `Not signed in: set DEEPSEEK_API_KEY (or DEEPSEEK_BUILD_API_KEY) to authenticate.`
- The key is never written to logs, sessions, or crash output. Prove it:
  `scripts/key-leak-check.sh` → `PASS fake key absent ...`.

## 3. Automated scripts (5–8 min)

| Command | Expected |
|---|---|
| `scripts/smoke-test.sh` | `8 passed, 0 failed`: `/models` lists `deepseek-v4-pro`+`deepseek-flash`; plain + streamed chat; a 3-turn thinking tool chain; usage has `reasoning_tokens`/`cache_read_input_tokens`; flash alias accepted; `cached_tokens == prompt_cache_hit_tokens` on a cache hit |
| `scripts/egress-check.sh` | `PASS EGRESS-CHECK PASS`, `EGRESS_VIOLATIONS=0`; destinations limited to `api.deepseek.com`, the `web_fetch` host, loopback/AF_UNIX |
| `scripts/key-leak-check.sh` | `PASS fake key absent ...` |
| `scripts/check-egress.sh --strict` | `HARD egress gate: OK`, exit 0 |
| `scripts/sandbox-run.sh -p "print hello"` | session completes; `web_fetch`/other external hosts are refused with `sandbox-run: BLOCKED connect ...` |

None of these scripts ever print the key.

## 4. Manual checklist (12–18 min)

Work in a scratch git repo: `git init /tmp/dsb-scratch && cd /tmp/dsb-scratch`.
Start with `deepseek-build` and approve tool actions in the TUI as needed.

| # | Step | Expected |
|---|------|----------|
| 1 | Run `deepseek-build` in the scratch dir | TUI starts with the DeepSeek whale welcome logo; no crash, no `~/.grok` access (only `~/.deepseek-build`) |
| 2 | Simple edit: "create hello.py printing hello" | A file-write tool call; permission prompt; approve → file created, diff shown |
| 3 | Fix a failing test: add `calc.py`/`test_calc.py` with a bug, ask it to fix the test | It runs `python3 test_calc.py`, fixes the code (not the test), reports pass |
| 4 | Multi-file refactor: "rename `mul` to `multiply` everywhere" | Edits all affected files; no stale `mul(` remains; tests still pass |
| 5 | Reasoning visibility | Thinking shown as a collapsible block, distinct from the answer |
| 6 | Fold keys: `e`, `E`, `Ctrl+E` | `e` folds/unfolds the selected entry; `E` folds all; `Ctrl+E` toggles all thinking blocks |
| 7 | `/think` | Shows/hides reasoning blocks |
| 8 | Plan mode (`/plan` or Shift+Tab) | Read-only investigation; asks approval before acting; `--no-plan` disables it. **Headless `--permission-mode plan` is read-only but does not emit a plan for a bare edit request (known gap).** |
| 9 | Permission modes (Shift+Tab; `/auto`; `--allow`/`--deny`/`--permission-mode`) | Modes change behavior; a dangerous `rm -rf` still prompts with a warning. In headless, a gated action is cancelled with a one-line stderr message naming `--always-approve` and a non-zero exit |
| 10 | Read-before-write | Editing a file not read first is denied with a clear message |
| 11 | Model switch (`/model`, alias `/m`) | Lists `deepseek-v4-pro`, `deepseek-flash`, hidden `deepseek-v4-flash`; switch works |
| 12 | Reasoning effort (`/effort <level>`) | Offers `none`, `low`, `high`, `max`; the request honors the chosen level |
| 13 | Status line | Shows cwd/model/context by default; `[ui.status_line] items` adds `effort`, `tokens`, `cache`, `cost`, `turn_timer`, `session_name` |
| 14 | Cost/cache (`/usage`) | Token usage and cache-hit/miss shown; cost when a price table is configured. Headless `--output-format json` exposes `estimated_cost_usd` + `estimate: true` (local peak-rate estimate, omitted with no tokens) and `total_cost_usd` only when the provider reported a complete cost |
| 15 | Themes (`/theme`, alias `/t`) | Switches `deepseek-monokai` (default) or another palette; `auto` follows the system |
| 16 | Resume (`-c` / `--resume` / `/resume`) | Previous session reloads with history; `-r <id>` reuses the same id |
| 17 | Headless (`deepseek-build -p "..."`) | Non-interactive reply on stdout, exit 0; `--output-format json` includes `usage` |
| 18 | `web_fetch` ask-gate | Fetching a non-allowlisted URL asks permission first; approve → fetches; deny → refuses |
| 19 | MCP (`[mcp_servers.<name>]` in `~/.deepseek-build/config.toml`; `/mcps`) | Configured stdio/HTTP MCP server connects; its tool appears and can be called |
| 20 | `AGENTS.md` | Instructions in `AGENTS.md` are followed |
| 21 | Checkpoints/undo (`/rewind`, alias `/undo`) | Restores a prior turn/conversation state |
| 22 | `scripts/check-egress.sh --strict` | `HARD egress gate: OK`, exit 0 (SOFT branding zero) |
| 23 | Voice dictation (`/voice`) | **N/A — removed** |
| 24 | Session share / relay | **N/A — removed** |
| 25 | Feedback upload (`/feedback`) | Local/inert: performs no network upload |
| 26 | Automatic model routing (Flash vs Pro) | **N/A — not wired** (policy defaults to `off`) |

## 5. Egress verification

Preferred (needs `strace`):

```sh
strace -f -e trace=connect,sendto,sendmsg ./target/release/deepseek-build -p "hi" 2>&1 | grep -i connect
```

Where `strace` is unavailable, `scripts/egress-check.sh` compiles an LD_PRELOAD
logger (`scripts/egress_log.c`) and fails on any destination outside
{`api.deepseek.com`, the `web_fetch` host, loopback, AF_UNIX}. It logs
`connect`/`sendto`/`sendmsg`/`sendmmsg` from the binary and dynamic children.
It cannot see raw `syscall(2)`/`io_uring`/statically-linked children — see
`KNOWN-ISSUES.md`.

## 6. Test suite

- `cargo test --workspace` **does not compile yet** — only `xai-grok-pager`
  (~252 errors) and `xai-grok-pager-minimal` (2) fail, on test-only references
  to deleted Phase-1 features. Every other lib test target compiles and passes,
  e.g. `cargo test -p xai-grok-shell --lib` (6221 pass),
  `cargo test -p xai-grok-workspace --lib` (1693 pass),
  `cargo test -p xai-grok-sampler --lib`, `-p xai-grok-telemetry --lib`,
  `-p xai-grok-sampling-types --lib`.
- `scripts/smoke-test.sh` is the live end-to-end check.
