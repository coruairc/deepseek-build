# 06-01 — DECISIONS §E/§F and TESTING §2/§6

**Executed:** 2026-11-26 · **Status:** complete (`3f60decf`, `f05a5aa5`)

- §E: legal sign-off re-homed to stable `v0.1.0`; O2/O5/O10 closed; O4 deferred to open-build;
  O6–O9 opened with their real status (Windows, REL-05, ACC-01, freeze). D9 amended to keep the
  personal-tool intent while recording that the release pipeline exists.
- §F: Phases 1–5.1 recorded as executed, with the real outcome (tree, proof and prerelease exist;
  stable `v0.1.0` does not). Removed the stale "Phase 5 skipped / release build aborted".
- TESTING §2/§6: OIDC count and the stale sign-in line removed; the gate is named as
  `scripts/test-gate.sh` with its three steps and the knowingly stale `test-gate.expected`
  (308/219, five new test binaries since).
