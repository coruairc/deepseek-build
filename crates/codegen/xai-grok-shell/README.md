# deepseek-build

`deepseek-build` (shortcut `dsb`) is a terminal coding agent driven by
DeepSeek's OpenAI-compatible Chat Completions API. It is a fork of
[xai-org/grok-build](https://github.com/xai-org/grok-build) (Apache-2.0).

This file is written into your config folder (`~/.deepseek-build/`, or
`$DEEPSEEK_BUILD_HOME`) on each new version for offline reference.

## Quick start

```bash
export DEEPSEEK_API_KEY="sk-..."   # key from platform.deepseek.com
deepseek-build                      # interactive TUI
deepseek-build -p "Explain this codebase"   # one-shot, headless
deepseek-build --version
```

The only model provider is DeepSeek (`https://api.deepseek.com`). The default
model is `deepseek-v4-pro`; switch with `/model`, and control reasoning with
`/think` and `/effort`.

## Configuration

- User config: `~/.deepseek-build/config.toml`
- System config: `/etc/deepseek-build/`
- Per-project config and instructions: `AGENTS.md` and the `.grok/` folder in
  the repository (the project folder name is kept for compatibility; see
  `KNOWN-ISSUES.md` in the source repository).

## More documentation

Full documentation, known issues and release notes live in the source
repository: <https://github.com/coruairc/deepseek-build>.
