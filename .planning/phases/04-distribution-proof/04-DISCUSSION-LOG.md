# Phase 4: Distribution Proof - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-11-26
**Phase:** 04-distribution-proof
**Areas discussed:** tag push, image verification, installer proof, Homebrew tap, cleanup

---

## Tag push

| Option | Description | Selected |
|--------|-------------|----------|
| `v0.1.0-proof1`, tag only | Push the tag and its commits without moving `main`. Prerelease by the hyphen rule. | ✓ |
| `v0.1.0-rc1`, tag only | Shorter name, still a prerelease, less obviously a proof tag. | |
| No tag | Phase 4 waits; no artifacts, no registry image. | |

**User's choice:** Approve `v0.1.0-proof1`.
**Notes:** Explicit approval for this one tag push. No `v0.1.0`, no `main` push, no `:latest`.

---

## Image verification

| Option | Description | Selected |
|--------|-------------|----------|
| Prove from the registry side | Digest, manifest list and tags via `gh api` / `docker manifest inspect`; record pull/run commands without executing them. | ✓ |
| Build locally | Requires a runtime; proves a different artifact than a user pulls. | |
| Wait for the author to start Docker | Blocks the phase until a human action. | |

**User's choice:** Leave Docker off; I'll pull later.
**Notes:** REL-05 is recorded as environment-blocked with the exact commands, not claimed as done.

---

## Installer proof

| Option | Description | Selected |
|--------|-------------|----------|
| Real release, pinned + unpinned | `VERSION=v0.1.0-proof1 sh install.sh`, then establish the `latest` path from the real redirect. | ✓ |
| Fake-tree harness only | Already exercised (39/39); proves nothing about the real assets. | |

**User's choice:** Accepted the recommendation.
**Notes:** The unpinned path's actual behaviour must be observed, not assumed.

---

## Homebrew tap

| Option | Description | Selected |
|--------|-------------|----------|
| Formula in-tree, tap repo left to the author | `packaging/homebrew/deepseek-build.rb` with real URLs/sha256s; the repo creation is one author command. | ✓ |
| Create the public tap repo now | A new public repo under the author's account without them present. | |
| Skip Homebrew | Leaves macOS users with a tarball only. | |

**User's choice:** Asked for Homebrew; repo creation deliberately left to the author.
**Notes:** No notarization needed for a tap; an ad-hoc signature on arm64 is required and Phase 3 provides it.

---

## Cleanup

| Option | Description | Selected |
|--------|-------------|----------|
| Keep the prerelease | It is the only macOS artifact and the author wants an installable macOS build. No `:latest`, no `v0.1.0`. | ✓ |
| Delete tag + release + image tag | Matches the roadmap's literal cleanup line; removes the only macOS artifact. | |

**User's choice:** Accepted the recommendation (keep).
**Notes:** Recorded as an explicit deviation from the roadmap's cleanup line, with the reason.

---

## the agent's Discretion

- Evidence item slugs, whether the image proof is one item or two, and the README wording.

## Deferred Ideas

- Pulling and running the image (REL-05) — needs a container runtime.
- Creating the Homebrew tap repo — author action.
