# Phase 3: Pipeline Decoupling & Version - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-11-25
**Phase:** 03-pipeline-decoupling-version
**Areas discussed:** CI proof execution, pipeline structure, tag-list guard, version contract, macOS artifacts

---

## CI proof execution (REL-02)

| Option | Description | Selected |
|--------|-------------|----------|
| Skip the CI dispatch; push untested | Make the workflow edits and push them without a separate dispatch. The first real evidence is Phase 4's throwaway pre-release tag run, which builds all four targets. | ✓ |
| Push a temp branch and dispatch | Push `ci/phase3-macos-proof`, run `gh workflow run release.yml --ref …`, watch both macOS legs produce tarballs. No release, no `:latest`. | Superseded |
| Defer the CI proof to Phase 4 | Local work only; fold the macOS proof into the throwaway-tag run. | Equivalent, but stated as a deferral rather than a decision |

**User's choice:** Skip the CI dispatch; push untested.
**Notes:** No temp branch and no `workflow_dispatch` in Phase 3. REL-02 is proven (or falsified) by Phase 4's tag run, which is a real four-target release build. Annotated in CONTEXT.md as knowingly accepted.

---

## Pipeline structure

| Option | Description | Selected |
|--------|-------------|----------|
| Two jobs, steps duplicated | `build-linux` / `build-macos`, artifact names unchanged, `container` depends on linux only. | ✓ |
| One reusable workflow | Extract the shared steps into a reusable workflow called twice. | |
| Keep one matrix, add an `if` | Leave the single `build` job and gate `container` on the Linux legs only. | |

**User's choice:** Accepted the recommendation (two jobs).
**Notes:** `publish` must still wait for macOS because its verify step names all four tarballs.

---

## Tag-list guard (REL-08)

| Question | ✅ Recommended | Alternative(s) |
|---|---|---|
| Where does the empty/junk tag-list guard go? | A pre-push step asserting the resolved tag list, plus strict semver validation in the `version` job | Guard inside the build-push `with:` block; guard only in the manifest step |

**User's choice:** Accepted all.
**Notes:** Research established that `metadata-action` does not fail on no-match and that `latest=auto` can push `:latest` before the manifest inspect fails, so the guard must precede the first push.

---

## Version contract (PKG-01, PKG-02)

| Question | ✅ Recommended | Alternative(s) |
|---|---|---|
| Workspace inheritance scope? | The four crates at `0.0.1-rc5`; other literals stay | Convert every manifest; only bump the four literals in place |
| Cargo.lock? | Regenerate and commit | Leave stale |

**User's choice:** Accepted all.
**Notes:** Mixing `version.workspace = true` with literal versions is legal and keeps the diff bounded.

---

## macOS artifacts (REL-02, REL-03, REL-07)

| Question | ✅ Recommended | Alternative(s) |
|---|---|---|
| DWARF handling | `debug = false` + link-time strip | External `strip` after signing (signature risk) |
| Signature check | `codesign --verify --strict` on the extracted tarball + `--version` exec | Pre-tar binary only |
| Timeout strategy | Raise the macOS job's own timeout; env overrides only if the run proves they are needed | Raise globally; LTO change everywhere |

**User's choice:** Accepted all.
**Notes:** One cache-warming run is permitted only if cold cache is demonstrably the cause.

---

## the agent's Discretion

- Exact timeout numbers, whether the tag guard is a shell step or a metadata `if`, whether macOS needs profile env overrides, and internal step ordering within each split job.

## Deferred Ideas

- `ci.yml` on push (REL-V2-01), debug-symbol sidecars (REL-V2-02), publishing `v0.1.0`, and the Phase 4 image pull/run — all explicitly outside this phase.
