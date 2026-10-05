# TESTING — deepseek-build

A 20–30 minute manual test plan for the personal `deepseek-build` agent.

> Status caveat: `cargo test --workspace` is a separate, slower verification (see
> the "Test suite" note at the end). The interactive product path — DeepSeek chat,
> tools, permissions, plan mode, sessions, headless, MCP — is testable today.
> See [`KNOWN-ISSUES.md`](KNOWN-ISSUES.md) for what is unfinished.

## 1. Build

```sh
source "$HOME/.cargo/env"          # rustup, pinned toolchain 1.94.0
cargo build --release -p xai-grok-pager-bin
# artifact (the package keeps the internal xai-grok-* name, D2):
ls -lh target/release/deepseek-build
./target/release/deepseek-build --version
# => deepseek-build <version>
```

If you already have the release binary, skip the build. `scripts/sandbox-run.sh`
expects it at `target/release/deepseek-build` (override with `BIN=`).

## 2. API key

```sh
export DEEPSEEK_API_KEY=sk-...      # preferred
# or: export DEEPSEEK_BUILD_API_KEY=sk-...
```

The key is read from the environment. The only network destinations are the
configured provider (`https://api.deepseek.com`) and any user-configured MCP
servers; see [Network & privacy](SECURITY.md#network--privacy).

## 3. Safe sandbox run (recommended first)

```sh
scripts/sandbox-run.sh --version
scripts/sandbox-run.sh -p "print hello and exit"
```

`scripts/sandbox-run.sh` installs an `LD_PRELOAD` shim that permits `connect()`
only to loopback, `AF_UNIX`, and the resolved addresses of
`$DEEPSEEK_BUILD_EGRESS_HOST` (default `api.deepseek.com`); everything else is
refused with `ECONNREFUSED` and logged as `sandbox-run: BLOCKED connect to <ip>`.
Use `BIN=/path/to/deepseek-build` to point it at another build.

Quick non-interactive smoke:

```sh
./target/release/deepseek-build -p "Reply with exactly: pong"
```

Expected: prints `pong` (or a short reply) and exits 0. With a bad/absent key you
get a clear `401` from `https://api.deepseek.com/chat/completions` plus the
model/auth line (`Model: deepseek-v4-pro`, `Auth: ApiKey`).

## 4. Manual checklist

Do these in a scratch git repo (`git init /tmp/dsb-scratch && cd /tmp/dsb-scratch`).
Each row should take roughly a minute. Rows marked **N/A — removed** or
**N/A — not implemented** are known gaps, not failures.

| # | Step | Expected |
|---|------|----------|
| 1 | Run `deepseek-build` in the scratch dir | TUI starts with the DeepSeek whale welcome logo; no crash |
| 2 | Ask "create hello.py printing hello" | It proposes a file write; permission prompt appears |
| 3 | Approve the write | File created; diff/status shown |
| 4 | Ask for a multi-file refactor | Multiple edits, streamed answer, tool calls |
| 5 | Reasoning visibility | Thinking shown as a collapsible block, visually distinct from the answer |
| 6 | Fold keys: `e`, `E`, `Ctrl+E` | `e` folds/unfolds selected entry; `E` folds every entry; `Ctrl+E` toggles all thinking blocks |
| 7 | `/think` | Shows/hides reasoning blocks on/off |
| 8 | Plan mode (`/plan`, or Shift+Tab to cycle modes) | Read-only investigation; asks for approval before acting; `--no-plan` disables it |
| 9 | Permission modes (Shift+Tab; `/auto`, `/always-approve`; flags `--allow`/`--deny`/`--permission-mode`) | Modes change behavior; a dangerous command (e.g. `rm -rf`) still prompts and carries a warning |
| 10 | Read-before-write | Editing a file without reading it first is denied with a clear message |
| 11 | Model switch (`/model`, alias `/m`) | Lists `deepseek-v4-pro`, `deepseek-flash`, and the hidden `deepseek-v4-flash`; switch works |
| 12 | Reasoning effort (`/effort <level>`) | Offers `none`, `low`, `high`, `max` (per-model catalog); request honors the chosen level |
| 13 | Status line | Shows cwd/model/context by default; `[ui.status_line] items` adds `effort`, `tokens`, `cache`, `cost`, `turn_timer`, `session_name` |
| 14 | Cost/cache (`/usage`) | Token usage shown; cache-hit/miss counters and cost when a price table is configured |
| 15 | Themes (`/theme`, alias `/t`) | Switches `deepseek-monokai` (default) or another palette (`deepseek-day`, `tokyonight`, `rosepine-moon`, `oscura-midnight`, `terminal`), or `auto` to follow the system |
| 16 | Resume (`-c` / `--resume` / `/resume`) | Previous session reloads with history |
| 17 | Headless (`deepseek-build -p "..."`) | Non-interactive reply on stdout, exit 0 |
| 18 | `web_fetch` ask-gate | Fetching a non-allowlisted URL asks permission first |
| 19 | MCP (`[mcp_servers.<name>]` in `~/.deepseek-build/config.toml`; `/mcps`) | Configured stdio/HTTP MCP server connects; its tools appear; `/mcps` shows status |
| 20 | `AGENTS.md` | Instructions in `AGENTS.md` are followed |
| 21 | Checkpoints/undo (`/rewind`, alias `/undo`) | Restores a prior turn / conversation state |
| 22 | `scripts/check-egress.sh --strict` | Prints `HARD egress gate: OK` and exits 0 (SOFT branding is zero) |
| 23 | Voice dictation (`/voice`) | **N/A — removed** (feature deleted; command is fail-closed/hidden) |
| 24 | Session share / relay (`/share`, remote relay) | **N/A — removed** (share command and WebSocket relay deleted) |
| 25 | Feedback upload (`/feedback`) | Command/modal still exists but is local/inert: it performs **no** network upload |
| 26 | Automatic model routing (Flash vs Pro) | **N/A — not wired.** The `model_routing` module and `route_turn` exist, but the policy defaults to `off` and no turn path calls it |
| 27 | Wiremock/live adapter smoke | **N/A here — see below.** Covered by `crates/codegen/xai-grok-sampler/tests/chat_completions_wire.rs`; live run needs a real key |

## 5. Egress verification (no `strace` needed)

The development environment had no `strace`, so an `LD_PRELOAD` connect-logger
was used. To reproduce where `strace` is available:

```sh
strace -f -e trace=connect ./target/release/deepseek-build -p "hi" 2>&1 | grep -i connect
```

Observed in development with a dummy key:

- `--version` / `--help`: **no** outbound connects.
- Single-turn prompt: **only** `api.deepseek.com:443` plus one local `AF_UNIX`
  socket. The request reached `https://api.deepseek.com/chat/completions` and
  returned 401 for the dummy key.
- `scripts/check-egress.sh --strict`: `HARD egress gate: OK`, exit 0.

`scripts/sandbox-run.sh --version` needs the release binary; with no
`target/release/deepseek-build` present it exits 127 before running anything.
Build first (or pass `BIN=`).

## 6. Test suite and live smoke

- `cargo test --workspace` compiles a large graph and is not part of the 20–30
  minute run; its current status is tracked in
  [`KNOWN-ISSUES.md`](KNOWN-ISSUES.md#build--tests).
- Adapter behavior is covered by
  `crates/codegen/xai-grok-sampler/tests/chat_completions_wire.rs` (wiremock):
  streaming text + reasoning, a 3-turn tool chain replaying `reasoning_content`,
  missing-`reasoning_content` backfill, typographic-quote tool-call JSON repair,
  and a partial/malformed stream. Retry/backoff is covered by
  `crates/codegen/xai-grok-sampler/tests/test_actor.rs` (429 and 500).
- **Live smoke (blocked without a key):** GET `/models` to confirm
  `deepseek-v4-pro` / `deepseek-flash`, a plain chat, a streamed chat, a 3-turn
  thinking tool-call chain, and `prompt_cache_hit_tokens` /
  `prompt_tokens_details.cached_tokens` plus `reasoning_tokens` in usage.

## 7. Troubleshooting

- `Unauthorized (401)` — set a real `DEEPSEEK_API_KEY`.
- Connection refused / DNS failure — check network; only `api.deepseek.com` (and
  user MCP servers) are used.
- Config/state lives under `~/.deepseek-build/` (the legacy `~/.grok` is **not**
  migrated automatically; point `GROK_HOME` at the old directory to reuse it).
- `sandbox-run: BLOCKED connect to <ip>` — the binary tried a destination outside
  the allowlist; capture the IP and report it.
