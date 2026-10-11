# deepseek-build User Guide

Learn how to install, configure, and extend deepseek-build, the terminal-based AI coding assistant from DeepSeek.

---

## Tier 1: Essential User Docs

Start here. These guides cover what you need on your first day.

| # | Document | Description |
|---|----------|-------------|
| 1 | [Getting Started](01-getting-started.md) | Installation, first launch, authentication, basic interaction, and key concepts |
| 2 | [Keyboard Shortcuts](03-keyboard-shortcuts.md) | Reference for every key binding and mouse action in the TUI |
| 3 | [Slash Commands](04-slash-commands.md) | Every `/` command, including goals, deep research, and workflow run management |
| 4 | [Configuration](05-configuration.md) | `config.toml`, `pager.toml`, environment variables, and file locations |

---

## Tier 2: Core Feature Docs

Customize and extend deepseek-build.

| # | Document | Description |
|---|----------|-------------|
| 5 | [Theming and Appearance](06-theming.md) | Themes, the `/theme` command, `pager.toml`, and color-support detection |
| 6 | [MCP Servers](07-mcp-servers.md) | External tool integrations through the Model Context Protocol |
| 7 | [Skills](08-skills.md) | Reusable prompt packages in the SKILL.md format |
| 8 | [Plugins](09-plugins.md) | Bundle and share skills, commands, agents, hooks, and MCP servers; install from, author, and manage marketplaces |
| 9 | [Hooks](10-hooks.md) | Lifecycle scripts and HTTP callbacks for pre- and post-tool-use events |
| 10 | [Custom Models](11-custom-models.md) | Bring-your-own-key, Ollama, and OpenAI-compatible endpoints |
| 11 | [Project Rules (AGENTS.md)](12-project-rules.md) | Per-directory AGENTS.md instructions and their precedence |
| 12 | [Memory](13-memory.md) | Cross-session knowledge persistence with `/flush`, `/dream`, and hybrid search |

---

## Tier 3: Advanced Usage Docs

Automate, script, and integrate deepseek-build with other systems.

| # | Document | Description |
|---|----------|-------------|
| 13 | [Headless Mode and Scripting](14-headless-mode.md) | `deepseek-build -p`, output formats, CI/CD integration, and piping |
| 14 | [Agent Mode and IDE Integration](15-agent-mode.md) | ACP stdio transport, server mode, and SDK integration |
| 15 | [Subagents and Personas](16-subagents.md) | Parallel child sessions, agent types, personas, and capability modes |
| 16 | [Session Management](17-sessions.md) | Save, load, resume, rewind, compact, and the session persistence format |
| 17 | [Sandbox Mode](18-sandbox.md) | OS-level filesystem and network isolation profiles |
| 18 | [Plan Mode](19-plan-mode.md) | Structured planning, plan-file edits, and approval before coding |
| 19 | [Background Tasks and Monitoring](20-background-tasks.md) | `background: true`, `/loop`, `monitor`, and `Ctrl+B` to demote |
| 20 | [Terminal Support and Troubleshooting](21-terminal-support.md) | tmux, SSH, truecolor, clipboard, and OSC 52 |
| 21 | [Permissions and Safety](22-permissions-and-safety.md) | Modes (always-approve, auto, ask), rules, matching, hooks, and examples |
| 22 | [Agent Dashboard](23-dashboard.md) | Central overview of local sessions and forks |
| 23 | [Status Line](25-status-line.md) | The bottom status row: built-in segments, command scripts, and the stdin JSON contract |
| 24 | [Configuration Reference](26-config-reference.md) | Field list for `config.toml` |
