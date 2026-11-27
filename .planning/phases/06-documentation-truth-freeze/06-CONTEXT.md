# Phase 6: Documentation Truth & Freeze - Context

**Gathered:** 2026-11-26
**Status:** Ready for planning

<domain>
## Phase Boundary

Make the docs describe the tree that exists, take the author's signed acceptance of a real TUI session,
and cut the DeepSeek entity onto a designated branch with a non-triggering marker tag.

**Freeze target changed by the author (2026-11-26):** the branch is **`deepseek-build`**, not `deepseek`.
The existing `deepseek` branch (origin `4b305bd0`) and the `deepseek-v0.1-snapshot` tag are left
untouched and recorded as superseded. A separate `open-build` branch is cut from `main` afterwards —
that is not part of this phase.

**Blocking gate:** `ACC-01` — the author's 20–30 minute TUI session on a clone of a real repo, with a
signed note committed — must land **before** `FREEZE-01/02/03` run. No branch and no marker tag may
precede the sign-off.

</domain>

<decisions>
## Implementation Decisions

### Docs truth (DOCS-01…04)
- `DECISIONS.md` §E/§F is rewritten from the tree: §E's open items are closed or re-homed, §F's phase
  log records Phases 1–6 as executed, D9's personal-tool intent survives, and the release pipeline's
  existence is stated rather than denied. — **Reversibility:** costly — these files are the contract other agents read; a wrong statement propagates.
- `TESTING.md` §2/§6 loses the OIDC test counts and the stale sign-in line; the counts it keeps are the
  ones the suite actually reports (target count + pass/fail/ignored).
- `KNOWN-ISSUES.md` records: `deepseek-v0.1-snapshot` at `4b305bd0` superseded; the `--permission-mode
  plan` correction (already landed in 02-06); the sandbox findings; and the true unexercised list.
- The user guide gains: Flash default + at-ceiling caveat + revert path; the estimate label; headless
  permission failure; container run flags; the measured cost levers (`/model`, `--effort low`); and the
  Windows tier once Phase 5 lands.

### Freeze mechanics (FREEZE-01/02/03)
- Branch `deepseek-build` is created **at the freeze commit on `main`** and pushed only with the
  author's explicit approval.
- The marker tag must be **non-triggering**: it must not match `v*`. Proposed name
  `deepseek-build-freeze-2026-11-26`. `git rev-parse <tag>^{}` must equal the branch tip, and no
  workflow run may be created.
- `deepseek-v0.1-snapshot` at `4b305bd0` is verified unchanged and recorded as superseded. It is never
  moved or deleted.
- `v0.1.0` is **not** published. The author publishes it.
- The pre-freeze checklist is re-run after every other phase lands: completeness-asserted clean suite,
  `check-egress.sh --strict` exit 0 with a dated record, `branding_surface`, `key-leak-check.sh`,
  `cargo-deny`, the Phase-2 runtime artifacts, and the Phase-4 dry-run evidence.

### The egress gate (open blocker, needs a decision)
- `scripts/check-egress.sh --strict` is **red at base** on three test-only strings:
  `xai-grok-shell-base/src/util/mod.rs:422-423` (`api.x.ai`, `grok.com`, a negative security assertion)
  and `xai-grok-shell/src/agent/config_tests.rs:2768` (`gs://bucket`, a config fixture).
- Weakening a security gate is not an autonomous decision. The plan must present the two honest
  options — a line-scoped, reasoned `egress-allow:` suppression, or rewriting/removing the fixtures —
  and let the author choose. Until then, FREEZE-03 cannot pass and the phase must say so.

### Phase-1 verification staleness
- Phase 1's `VERIFICATION.md` is stale (source changed after the verifier ran). FREEZE-03's checklist
  requires a fresh verification; either re-run `execute-phase 1` or record an explicit acceptance.
  This is part of the checklist, not a docs edit.

### Decided autonomously
- [Branch] Q: "Move the existing `deepseek` branch?" → Selected: "no, a new `deepseek-build` branch" — the author named it; moving a published ref is destructive.
- [Tag] Q: "Reuse `deepseek-v0.1-snapshot`?" → Selected: "no, a new non-triggering marker" — the old tag is published and recorded as superseded.
- [Docs] Q: "Rewrite the user guide wholesale?" → Selected: "targeted edits only" — it was swept on 2026-10-12 and is mostly accurate; wholesale rewrite adds risk without evidence.

### the agent's Discretion
- The exact marker tag name, the ordering inside the checklist, and the wording of the acceptance note
  template the author signs.

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `docs/evidence/README.md` (the evidence convention and the Phase-2 re-run section).
- `scripts/test-gate.sh` (completeness assertion), `scripts/check-egress.sh`, `scripts/key-leak-check.sh`,
  `deny.toml`.
- `crates/codegen/xai-grok-pager/docs/user-guide/` (swept 2026-10-12).

### Established Patterns
- `KNOWN-ISSUES.md` is the honest ledger; correcting it is preferred over touching working code.
- User-visible strings resolve through `xai-grok-brand`; `branding_surface` gates CLI help/version.
- Freeze markers are non-triggering by name; `v*` is the only release trigger.

### Integration Points
- The pre-freeze checklist reads artifacts from Phases 1–5; it is the cross-phase integration gate.
- The marker tag's commit must equal `main`'s tip at freeze time.

</code_context>

<specifics>
## Specific Ideas

- The author's acceptance session is the milestone's only human-blocking dependency; everything else in
  this phase can be prepared before they return.
- A freeze that ships a red egress gate or a stale verification would certify something untrue, which is
  the one failure mode this milestone exists to prevent.

</specifics>

<deferred>
## Deferred Ideas

- Publishing `v0.1.0` — the author.
- Cutting `open-build` from `main` — after the freeze, author-directed.
- A Windows sandbox backend — open-build.

</deferred>

## Freeze continuation decisions (2026-11-27)

- The author accepted Windows as **left for a Windows user to test**. WIN-03 (ConPTY), WIN-05
  (installer executed) and WIN-06 (acceptance record) stay open; the audit must say so.
- REL-05 stays **partial**: the in-image turn runs; the image ships no `bwrap`, so `--sandbox` is
  not enforceable there (already recorded in README, KNOWN-ISSUES and the evidence index).
- The freeze branch is **`deepseek-build`**; the marker tag is **`deepseek-build-freeze-2026-11-27`**
  (must not match `v*`); `open-build` is created **from `main`** afterwards.
- The two verification records (Phase 1 stale, Phase 2 `gaps_found` for a since-closed gap) are
  refreshed rather than hand-edited.
- `/gsd-autonomous` is the supported way to run this: Phase 5 is listed under **Deferred
  Verification** in `STATE.md` so the run skips it, and Phase 6 runs 06-05 to completion (the
  pushes are approved by the author's invocation of the plan).
