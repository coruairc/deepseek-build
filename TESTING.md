# TESTING — deepseek-build

A 20–30 minute manual test plan for the personal `deepseek-build` agent.

> Status caveat: Phases 3 (TUI features) and 4 (hardening) are not finished, and
> `cargo test --workspace` does not compile yet. See [`KNOWN-ISSUES.md`](KNOWN-ISSUES.md).
> The core agent (DeepSeek chat, tools, permissions, sessions) is testable.

## 1. Build

```sh
source "$HOME/.cargo/env"          # rustup, toolchain 1.94.0
cargo build --release -p xai-grok-pager-bin
# artifact:
ls -lh target/release/deepseek-build
./target/release/deepseek-build --version
# => deepseek-build <version>
```

## 2. API key

```sh
export DEEPSEEK_API_KEY=sk-...      # preferred
# or: export DEEPSEEK_BUILD_API_KEY=sk-...
```

The key is read directly; the only network destination is `https://api.deepseek.com`.

## 3. Safe sandbox run (recommended first)

```sh
scripts/sandbox-run.sh --version
scripts/sandbox-run.sh -p "print hello and exit"
```

`scripts/sandbox-run.sh` blocks outbound connections to anything except the
configured provider (and loopback), so you can watch behavior safely. See the
script header for details.

Quick non-interactive smoke:

```sh
./target/release/deepseek-build -p "Reply with exactly: pong"
```

Expected: prints `pong` (or a short reply) and exits 0. With a bad/absent key you
get a clear `401 ... api.deepseek.com/chat/completions` error and the model/auth
line (`Model: deepseek-v4-pro`, `Auth: ApiKey`).

## 4. Manual checklist

Do these in a scratch git repo (`git init /tmp/dsb-scratch && cd /tmp/dsb-scratch`).

| # | Step | Expected |
|---|------|----------|
| 1 | `deepseek-build` in the scratch dir | TUI starts; first-run onboarding; no crash |
| 2 | Ask "create hello.py printing hello" | It proposes a file write; permission prompt appears |
| 3 | Approve the write | File created; diff/status shown |
| 4 | Ask for a multi-file refactor (e.g. split a function) | Multiple edits, tool calls, streamed answer |
| 5 | Reasoning visibility | Thinking text (if any) is distinguishable from the answer |
| 6 | Enter plan mode (`/plan` or the plan hotkey) | Read-only investigation; asks to approve before acting |
| 7 | Permission modes (`/permissions` or flags `--allow`/`--deny`) | Modes change behavior; dangerous commands prompt |
| 8 | Model switch (`/model`) | Lists `deepseek-v4-pro`, `deepseek-flash`, `deepseek-v4-flash`; switch works |
| 9 | Reasoning effort | `/model`/effort control accepts none/low/high/max; request honors it |
| 10 | Usage/cost | Token usage shown; cache-related counters when available |
| 11 | Resume (`-c` / `/resume`) | Previous session reloads with history |
| 12 | Headless (`deepseek-build -p "..."`) | Non-interactive reply on stdout, exit 0 |
| 13 | `web_fetch` ask-gate | Fetching a URL asks permission first (unless allowed) |
| 14 | MCP | Configure a stdio MCP server in `~/.deepseek-build/config.toml`; tools appear |
| 15 | `AGENTS.md` | Put instructions in `AGENTS.md`; the agent follows them |
| 16 | Checkpoints/undo (`/rewind` or `/undo`) | Restores prior file state / conversation |
| 17 | `scripts/check-egress.sh --strict` | HARD OK and no SOFT output (exit 0) |

## 5. Egress verification (no strace needed)

The environment used for development had no `strace`, so an `LD_PRELOAD`
connect-logger was used. To reproduce:

```sh
# compile a connect logger (see AGENTS.md / the dev history) or use strace:
strace -f -e trace=connect ./target/release/deepseek-build -p "hi" 2>&1 | grep -i connect
```

Observed in development with a dummy key:

- `--version` / `--help`: **no** outbound connects.
- Single-turn prompt: **only** `api.deepseek.com:443` plus one local `AF_UNIX`
  socket. The request reached `https://api.deepseek.com/chat/completions` and
  returned 401 for the dummy key.

## 6. Troubleshooting

- `Unauthorized (401)` — set a real `DEEPSEEK_API_KEY`.
- Connection refused / DNS failure — check network; only `api.deepseek.com` is used.
- Config/state lives under `~/.deepseek-build/` (the legacy `~/.grok` is **not**
  migrated automatically; set `GROK_HOME` if you want the old data).
- `cargo test` does not compile yet — that is a known issue, not a build failure.
