---
phase: 05-documentation-truth-freeze
plan: "03"
status: complete
requirements: [DOCS-04]
verification: "docs only (tests skipped by owner, 2026-11-22)"
---

# Plan 05-03: User-guide truth pass

## Changes

- `18-sandbox.md`: new **Running inside a container** section — the image is
  headless-only, bwrap needs user namespaces, the minimal docker flags, prefer
  user namespaces over `--privileged`, and a deny plan that cannot be bound
  makes the process refuse rather than expose the path.
- `25-status-line.md`: new **Cost levers** section — Flash is the default and the
  cheapest model, `/model deepseek-v4-pro` and `-m deepseek-v4-pro` switch and
  `/model deepseek-flash` returns; `/effort low` / `--effort low` was the only
  measured win in `docs/TUNING.md` E3; the figure is peak-priced and an override
  halves it for off-peak.
- Already in place from earlier plans (no change needed): the Flash at-ceiling
  caveat and revert path (`11-custom-models.md`, `05-configuration.md`,
  `README.md`), the estimate label (`25-status-line.md`), and the headless
  denial hint on both paths (`14-headless-mode.md`,
  `22-permissions-and-safety.md`).

## Verification

- **DEFERRED (tests skipped by owner, 2026-11-22).** Documentation only.
