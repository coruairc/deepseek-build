---
phase: "02"
slug: loop-item-verification
created: "2026-11-22"
discuss_mode: autonomous-carry-forward
---

# Phase 2 — Loop-Item Verification: Context

## Goal

Prove the daily loop — permissions, resume, plan mode — and the runtime egress
boundary against a real model-driven session, with committed artifacts.

## Locked decisions carried in

- Evidence convention from Phase 1 (`docs/evidence/NN-<slug>/<item>/`): cast +
  eight-key `report.json` + `notes.md`; later phases append, never rewrite.
- The only outbound destinations are the configured provider
  (`api.deepseek.com`) and the `web_fetch` host the user asks for; MCP servers
  are user-configured over stdio/HTTP. A new destination is a stop-and-ask.
- The sandbox stays on; no key material in logs, sessions or evidence.
- Plan mode is a session mode (TUI `/plan`), not a permission mode; the three
  permission modes are `ask`, `auto`, `always-approve`.

## Owner decision — tests and live runs deferred (2026-11-22)

The owner directed that no test, gate, smoke or egress run be performed for the
remaining milestone plans, and that verification steps be recorded as
`DEFERRED (tests skipped by owner, 2026-11-22)` instead of blocking. This phase
therefore **writes the harness recipes, the evidence scaffolding and the doc
corrections**, and records every live/permission/resume run as deferred. The
three evidence items are `result: skipped` until a keyed run happens.

No throwaway `DEEPSEEK_API_KEY` is available in this environment; PROOF-01/02/03
remain blocked on that owner-provided secret even after testing resumes.
