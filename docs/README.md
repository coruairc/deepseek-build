<!-- generated-by: gsd-doc-writer -->
# Documentation

Detailed documentation for `deepseek-build`. The [top-level README](../README.md) is
the short version; this page indexes everything under `docs/`.

## Guides

| Document | Covers |
|----------|--------|
| [Install and uninstall](install.md) | Installing from the published release, upgrading, verifying the download, removing it completely, troubleshooting, and the Windows status. |
| [Configuration](configuration.md) | Config files and their precedence, the API key, models and effort, permissions and plan mode, and MCP servers, hooks, skills, and agents. |
| [Container image](container.md) | The GHCR image and its platforms, pulling and verifying it, the headless run flags, and why `--sandbox` cannot be used inside it. |
| [Development](development.md) | Toolchain, build commands, the test gate, egress and branding gates, the release pipeline, versioning, and the evidence convention. |
| [Planning](planning.md) | The milestone plan, roadmap, requirements, per-phase artifacts, and how to read them. |

## User guide

The in-product reference lives outside this directory:
[deepseek-build User Guide](../crates/codegen/xai-grok-pager/docs/user-guide/README.md) —
getting started, keyboard shortcuts, slash commands, configuration, theming, MCP,
skills, plugins, hooks, models, sessions, sandbox, plan mode, headless mode, and the
config reference.

## Project records

- [DECISIONS.md](../DECISIONS.md) — locked design decisions; wins on conflict.
- [KNOWN-ISSUES.md](../KNOWN-ISSUES.md) — known gaps and unexercised surfaces.
- [TESTING.md](../TESTING.md) — the test plan and safe local runs.
- [AGENTS.md](../AGENTS.md) — codebase map, current state, and rules for changing the code.
- [evidence/README.md](evidence/README.md) — the committed-proof convention and report schema.

## Reference

- [TUNING.md](TUNING.md) — measured tuning and evaluation log, including failed experiments.
- [DROPPED-TESTS.md](DROPPED-TESTS.md) — ledger of tests removed with their behavior.
- [LICENSE-AUDIT.md](LICENSE-AUDIT.md) — dependency license audit; raw output in `license-deny-report.txt`.
- [INTERNAL-GROK-ENV-NAMES.md](INTERNAL-GROK-ENV-NAMES.md) — snapshot of remaining internal `GROK_*` names.
- [acceptance-2026-11-26.md](acceptance-2026-11-26.md) — the author's signed acceptance of the `0.1.0` build.

## Conventions

An evidence item is a cast (or `script -T` capture) plus a `report.json` and a
`notes.md`, committed so a later reader can re-run it. Planning artifacts are
per-phase: `CONTEXT` (decisions), `PLAN` (the contract), `SUMMARY` (what happened),
and `VERIFICATION` (the verdict), in `.planning/phases/NN-<slug>/`.
