# ACC-01 — Owner acceptance session (checklist)

Prepared by Backend, 2026-11-22. This file is a checklist only. It is **not** the signed acceptance note.
The note is written only after the owner reports back, in the owner's own words.

## Setup (5 min)
1. Binary built from `backend/gsd-auto` (`deepseek-build --version` should print 0.1.0).
2. Export your own key on the test machine: `export DEEPSEEK_API_KEY=...` (never paste it into chat or commit it).
3. Clone a real repo you know well into a scratch dir and `cd` into it.
4. Run `deepseek-build`.

## Session (20–30 min) — try each, note anything odd
- [ ] Ask a question about the codebase (read/search tools work, answers are grounded).
- [ ] Ask for a small real edit; review the diff; accept it.
- [ ] Ask it to run a shell command (e.g. the repo's tests or `git status`); check the approval prompt appears.
- [ ] Deny one tool call and confirm it respects the denial.
- [ ] Long-ish task that triggers several turns; watch the model shown and the cost/token estimate.
- [ ] Quit and resume the session.
- [ ] Anything that mentions Grok/xAI, logs in to xAI, or contacts a host other than api.deepseek.com.

## Report back to Backend
- Overall: sign off yes / no
- What worked, what broke (with rough steps)
- Machine/OS used, repo used (name only)

Backend then commits `05-04-ACCEPTANCE.md` with that report, which unblocks FREEZE-01..03.
