# Acceptance — 2026-11-26

**Build tested:** `deepseek-build 0.1.0 (bc95bc47f13f)` — a release build of `main` at
`bc95bc47`, installed over `~/.local/bin/deepseek-build` and run as `dsb`.
**Session:** the author's own terminal, a clone of a real repository, ~20–30 minutes of normal
daily-loop use.

## The author's attestation

> "works awesome is much faster than opencode. Few 1-5 work good."

(Referring to the five checks requested: edit approval with the three outcomes; rejecting a
destructive command; `/model` + `/effort low` surviving `--resume`; `/plan` writing only the plan
file; and the cost figure being labelled an estimate.)

## Findings the author reported

1. **Thoughts cannot be expanded with `Ctrl+E` the way grok-build does it.** `Ctrl+E`
   (`ExpandAllThinking`) is bound only when the scrollback is focused and toggles *all* thinking
   blocks; the author expects it to expand the thought in front of them.
2. **Focus is not restored after closing a thought.** Clicking a thought and pressing `Esc` leaves
   the text box unreachable until it is clicked again.

Both are recorded as post-acceptance fixes. The acceptance applies to the build above; the fixes
that follow it are listed in `KNOWN-ISSUES.md` and are re-checked before the freeze.

## What this does not claim

- It does not claim the API-key popup (added after this build) was exercised.
- It does not claim Windows or container behaviour; those are covered separately.
- It does not claim the two findings above are fixed.

## Follow-up (2026-11-27)

The author kept using later builds of the same tree and confirmed the fixes that followed the
sign-off:

- `/agent` switches **in place** (no restart) after the router fix, and the picker closes on
  selection;
- `Ctrl+E` toggles the thinking blocks, which now start **expanded** — "great exactly what i
  invisioned, using it now, to review this repo and works fine";
- the `session_start` hook no longer reports `${n}` as a missing environment variable.

This is recorded as a continuation of the same acceptance, not a new one. The signed session above
remains the build `0.1.0 (bc95bc47)`; the follow-ups were verified by the author on
`0.1.0 (c4eb34d8)`.
