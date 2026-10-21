# Changelog

All notable changes to `deepseek-build` are recorded here. Dates are UTC.

## v0.1.0 — unreleased

First release of `deepseek-build`, an independent DeepSeek-first terminal coding
agent forked from [xai-org/grok-build](https://github.com/xai-org/grok-build)
(Apache-2.0). Not affiliated with DeepSeek or xAI. No `v*` tag has been pushed
yet, so this entry describes the tree, not a published release.

### Added

- DeepSeek provider over the OpenAI-compatible Chat Completions API (default
  base URL `https://api.deepseek.com`), with models `deepseek-v4-pro`
  (default) and `deepseek-flash`, plus the hidden legacy alias
  `deepseek-v4-flash`.
- Thinking control and reasoning effort `none` / `low` / `high` / `max`;
  `reasoning_content` round-trip across tool-call turns, with backfill when it
  is missing and typographic-quote repair.
- Env-only auth: `DEEPSEEK_API_KEY` or `DEEPSEEK_BUILD_API_KEY`; a missing key
  prints one line naming both and exits 1.
- Local cost estimates from a configurable peak-rate price table
  (`[ui.status_line.pricing.<model>]`); headless JSON reports
  `estimated_cost_usd` with `"estimate": true`.
- TUI: reasoning folding (`e` / `E` / `Ctrl+E`, `/think`), `/model`, `/effort`,
  `/theme`, Monokai default theme with whale logo, configurable status line
  with `model`, `context`, `effort`, `tokens`, `cache`, and `cost` items.
- `install.sh`: checksum-verified install to `${INSTALL_DIR:-~/.local/bin}`,
  `VERSION` pinning, `--uninstall`, and a `dsb` shortcut symlink.
- Container definitions (`docker/Dockerfile`, `docker/Dockerfile.localtest`)
  and a tag-triggered release workflow for four targets.
- Scripts: `smoke-test.sh`, `egress-check.sh`, `check-egress.sh`,
  `key-leak-check.sh`, `sandbox-run.sh`, `test-install.sh`, `disk-guard.sh`,
  and an eval harness under `scripts/evals/`.

### Changed

- User-facing name is `deepseek-build`: binary, `~/.deepseek-build` config dir,
  `DEEPSEEK_BUILD_*` env vars, `deepseek-build/*` ACP namespace. Internal
  `xai-grok-*` crate names are unchanged (D2). Legacy `GROK_*` env fallbacks are
  ignored.
- `web_fetch` is permission-gated and off by default.

### Removed

- Telemetry and analytics (Mixpanel, Sentry, OTLP), upload/exfiltration paths,
  auto-update, announcements, remote/cloud config, remote control and session
  share relay, the xAI OAuth/OIDC network stack, and the xAI-hosted
  `web_search`, voice, and Imagine tools.
- `Responses` and `Messages` provider backends; only Chat Completions remains.
- User-guide pages for removed flows, and guide references to commands that
  do not exist in this build (`clone`, `worktree redirect`, `agent serve`,
  `update`).

### Known issues

See [`KNOWN-ISSUES.md`](KNOWN-ISSUES.md). Notably: `--version` still prints the
inherited `1.0.45`; per-repo config is still read from `.grok/`; the release
workflow and container image have never run; the TUI has not been used by a
human.
