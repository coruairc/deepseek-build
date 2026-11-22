---
phase: 02-loop-item-verification
plan: "04"
status: complete
subsystem: permissions
requirements: [PERM-01,PERM-02,PERM-03,PERM-04]
verification: "DEFERRED (tests skipped by owner, 2026-11-22)"
metrics:
  duration: "short (no runs executed)"
  completed: "2026-11-22"
---

# Plan 02-04: Permission surface (PERM-01…04)

**Status:** code/docs complete; every verification step is **DEFERRED (tests
skipped by owner, 2026-11-22)**.

## Changes

- **Code:** `crates/codegen/xai-grok-pager/src/headless.rs` — the headless
  permission-cancelled error now names both escapes:
  `re-run with --always-approve, or use --permission-mode auto`. The stderr hint
  already named both; the JSON error path now matches it (PERM-02).
- **Docs:** `22-permissions-and-safety.md` gained a headless fail-closed section
  (both paths, non-zero exit, nothing auto-approved) and the
  "`--permission-mode plan` is not a permission mode" section (three permission
  modes; plan mode is `/plan`/`Shift+Tab`, TUI-only, no headless arming flag).
  `14-headless-mode.md` states the same hint and the deny-wins rule.
- **KNOWN-ISSUES:** the stale "Plan mode gap" entry is corrected to record that
  `--permission-mode plan` parses to `ask` and that plan mode is a session mode.
- The interactive prompt's allow-once / allow-for-session / reject rows and the
  high-signal warning already exist (`prompt_widget`); exercising them is
  deferred.

## Verification

- **DEFERRED (tests skipped by owner, 2026-11-22).** No live, permission or
  resume run was executed. Evidence items are committed as `result: skipped`
  with the rerun recipe in `notes.md`.
