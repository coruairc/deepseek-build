<!-- generated-by: gsd-doc-writer -->
# Configuration

deepseek-build reads settings from a per-user directory, a project directory in
the repository, and a credentials file; `deepseek-build inspect` shows which
files and values won.

## Files and precedence

| Path | Scope | Holds |
|------|-------|-------|
| `~/.deepseek-build/config.toml` | User | Models, UI, tools, features, permissions, `[mcp_servers]`, `[hooks]`. `DEEPSEEK_BUILD_HOME` relocates the directory. |
| `.deepseek/config.toml` | Project | MCP servers, plugins, permission rules, `[mcp] max_output_bytes`. Searched cwd → repo root. |
| `~/.deepseek-build/credentials.json` | User | The saved API key, mode `0600`, written by the TUI prompt. |

Layers, lowest priority first:

1. Compiled defaults.
2. `$DEEPSEEK_BUILD_HOME/config.toml` (default `~/.deepseek-build/config.toml`).
3. Project `.deepseek/config.toml`.
4. `DEEPSEEK_BUILD_CONFIG` (inline JSON) or `DEEPSEEK_BUILD_CONFIG_PATH` (JSON/TOML file) overlay. Allowlisted soft settings only — it cannot change permissions, auth, trust, or egress.
5. `DEEPSEEK_BUILD_*` environment variables.
6. CLI flags such as `-m`, `--effort`, `--sandbox`, `--always-approve`.

Project files are found by walking from the working directory up to the git
root (outside a repo, only the working directory); levels merge repo-root first
and cwd last, so the deepest file wins, and each level contributes at most one
`config.toml`. Within a level `.deepseek/` wins over `.grok/`, which is kept
as a **read-only compatibility fallback** so an existing `.grok/` checkout
keeps working. Claude/Cursor compatibility reads apply only where a surface
documents them, with precedence `.deepseek → .grok → .claude / .cursor` (and
`.agents/` between `.grok/` and vendor dirs for skills).

## API key

The key resolves `DEEPSEEK_API_KEY` → `DEEPSEEK_BUILD_API_KEY` →
`~/.deepseek-build/credentials.json`.

With neither variable set and no usable saved key, an interactive launch opens
a masked **DeepSeek API key** prompt: input renders as `*`, Enter validates and
saves, and Esc or Ctrl-C quits with `Not signed in: set DEEPSEEK_API_KEY (or
DEEPSEEK_BUILD_API_KEY) to authenticate.` An unreadable or too-permissive saved
file also opens the prompt, so you can replace it.

The file must be owner-only (`0600`); saves are atomic and re-assert the mode.
A `0644` file is refused, and the error names the path and the fix, not the key:

```bash
chmod 600 ~/.deepseek-build/credentials.json
```

Headless never prompts: a `-p` run fails closed with that same message and
`deepseek-build agent` reports the missing key to its client, so scripts and CI
must set one of the variables. The key is never logged, shown in an error, or
written into a saved session: its in-memory value has a redacting `Debug`, and
sessions persist conversation items, not credentials.

## Model and effort

`deepseek-flash` is the default for new sessions (`[models] default`), picked
for cost and speed. Both catalog models pass 63/63 on the 21-task eval set, so
it cannot separate them; switch when a task needs more reasoning:

- TUI: `/model deepseek-v4-pro`
- Headless or CLI: `-m deepseek-v4-pro` (that is `--model`)
- Config: `[models] default = "deepseek-v4-pro"`

Reasoning effort is per session and per model: `/effort low` in the TUI,
`--effort low` on the CLI (alias of `--reasoning-effort`), or
`[models] default_reasoning_effort = "low"`. On `deepseek-v4-pro` that is the
measured cost lever — −9% cost and −63% reasoning tokens at the same 63/63
pass rate; the default stays `high` for harder work. See [`TUNING.md`](TUNING.md).

Prices are estimates, not a bill: the usage payload carries tokens but no
price, so deepseek-build prices locally at peak list rates. Headless JSON emits
`estimated_cost_usd` with `"estimate": true`; the status line and `/usage` agree.

## Permissions and plan mode

Permission modes set the baseline; rules refine it. Set the mode with
`[ui] permission_mode`, `Shift+Tab`, `/always-approve` or `/auto`, or
`--permission-mode` / `--always-approve`:

| Mode | Behavior |
|------|----------|
| `default` / `ask` | Read-only tools and built-in read-only shell commands run; everything else prompts. |
| `acceptEdits` | File edits run without a prompt. |
| `auto` | The safety check approves routine work; other calls are blocked or escalated. |
| `dontAsk` | Only pre-approved tools and built-in read-only handling run. |
| `bypassPermissions` / `always-approve` | Tool calls run in general; `deny` rules, hooks, and some shell `ask` rules still apply. |

Rules live in the user `config.toml` and project `.deepseek/config.toml` (plus
`.claude/settings.json`). All sources merge into one set, and `deny` > `ask` >
`allow` applies across scopes, so a project `allow` cannot beat a global `deny`:

```toml
[permission]
allow = ["Bash(cargo test *)", "Read"]
deny = ["Bash(rm -rf *)"]      # verbose form: [[permission.rules]]
```

`--allow` and `--deny` add rules for one invocation. Plan mode is read-only
except for the plan file: `/plan` (or `--permission-mode plan`, or `Shift+Tab`)
starts it, `plan.md` in the session directory is auto-approved, and any other
edit is rejected in every mode, including always-approve. See
[Permissions and safety](../crates/codegen/xai-grok-pager/docs/user-guide/22-permissions-and-safety.md),
[Plan mode](../crates/codegen/xai-grok-pager/docs/user-guide/19-plan-mode.md).

## MCP, hooks, skills, agents

| Surface | User | Project |
|---------|------|---------|
| MCP servers | `[mcp_servers.<name>]` in `config.toml`; `~/.claude.json` compat | `.deepseek/config.toml`, `.mcp.json` at any level, `.cursor/mcp.json` compat |
| Hooks | `~/.deepseek-build/hooks/*.json`; `[hooks]` in `config.toml` | `.deepseek/hooks/*.json`; `.claude/settings.json`, `.cursor/hooks.json` compat |
| Skills | `~/.deepseek-build/skills/`; `.claude/skills/`, `.cursor/skills/` compat | `.deepseek/skills/`; `.agents/skills/`; `.grok/` fallback |
| Agents | `~/.deepseek-build/agents/`; `~/.claude/agents/` compat | `.deepseek/agents/`; `.grok/` fallback; `.claude/agents/` compat |
| Plugins | `~/.deepseek-build/plugins/` (trusted automatically) | `.deepseek/plugins/` (requires trust); `.claude/plugins/` compat |

Project MCP servers, hooks, skills, plugins, and LSP servers are gated by folder
trust; grant it with `/hooks-trust` or `--trust`, and a named project MCP server
replaces a global one. `/agent` switches the session's agent and keeps the
conversation (alias `/agents`; bare `/agent` opens a picker of built-in, user,
and project agents). Details:
[MCP servers](../crates/codegen/xai-grok-pager/docs/user-guide/07-mcp-servers.md),
[Skills](../crates/codegen/xai-grok-pager/docs/user-guide/08-skills.md),
[Plugins](../crates/codegen/xai-grok-pager/docs/user-guide/09-plugins.md),
[Hooks](../crates/codegen/xai-grok-pager/docs/user-guide/10-hooks.md).

## Status line and cost

The status line is off by default. Enable it under `[ui.status_line]` in the
user `config.toml`; the `cost` item shows the session estimate:

```toml
[ui.status_line]
type = "builtin"
items = ["model", "effort", "tokens", "cache", "cost", "context"]

[ui.status_line.pricing.deepseek-v4-pro]
cache_hit = 0.044     # USD per 1M tokens
cache_miss = 1.32
output = 3.96
```

The `cost` item renders as `~$0.37 est.` (four decimals below one cent) and is
hidden below $0.0001 rather than shown as zero. It honours
`[ui.status_line.pricing]` per model; unset rates inherit v4 Pro, the shipped
table prices at peak, and any provider-reported cost wins. Read at startup;
edits apply next launch. Full item list:
[Status line](../crates/codegen/xai-grok-pager/docs/user-guide/25-status-line.md).

## Reference

The complete `config.toml` field list, including types, requirements-file
support, and managed-layer behavior, is the hand-edited
[26-config-reference.md](../crates/codegen/xai-grok-pager/docs/user-guide/26-config-reference.md) —
the source of truth for keys. This page covers scopes and common settings;
conceptual guidance is in
[05-configuration.md](../crates/codegen/xai-grok-pager/docs/user-guide/05-configuration.md).
