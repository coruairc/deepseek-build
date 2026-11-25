# Phase 4: Distribution Proof - Context

**Gathered:** 2026-11-26
**Status:** Ready for planning

<domain>
## Phase Boundary

Prove the distribution artifacts from a real throwaway pre-release tag: the GHCR image is built and
published, the installer is proven against the real release (pinned and unpinned), the macOS tarballs
exist on the release, and a Homebrew tap formula is authored. The image is **not** pulled or run in
this phase — no container runtime is available on the build host — so that half is recorded as an
environment-blocked item with exact reproduce commands.

**In scope:** one tag push (`v0.1.0-proof1`), the four tarballs and `SHA256SUMS` it produces, the
GHCR image digest, `install.sh` pinned/unpinned behaviour, the README GHCR/install section,
`packaging/homebrew/deepseek-build.rb`, and the committed evidence.

**Out of scope:** publishing `v0.1.0`; moving `:latest`; creating the public tap repo (author action);
running the image (no runtime); Windows artifacts (Phase 5); docs reconciliation (Phase 6).

</domain>

<decisions>
## Implementation Decisions

### The tag
- Exactly one tag is pushed: `v0.1.0-proof1`. **Only the tag**, never `main`: a tag push carries its own
  commits and Actions reads the workflow from that ref. — **Reversibility:** costly — it creates a public prerelease and a GHCR image; deleting either later is possible but visible.
- The tag is a prerelease by the workflow's own hyphen rule and must never be classified stable.

### The image
- Build and publish are proven by the release run itself. Verification is registry-side (image digest,
  manifest list, tags) via `gh api` / `docker manifest inspect`, not by pulling.
- `REL-05` (a real headless turn inside the pulled image) is **environment-blocked**: no Docker daemon,
  no containerd. The item records the exact `docker pull` / `docker run` commands, the digest to pin,
  and the expected output; it does not claim the run happened.
- No `:latest` may be created or moved by the throwaway tag. The tag-list guard from Phase 3 is the
  enforcement point and is re-verified against this run's metadata.

### Installer
- Pinned: `VERSION=v0.1.0-proof1 sh install.sh` from the real release, asserting `deepseek-build
  --version` prints the tag version.
- Unpinned: the `latest` resolution path is established by checking the real redirect/URL, not by the
  fake-tree harness.

### Homebrew
- The formula lives in-tree at `packaging/homebrew/deepseek-build.rb` with `on_arm`/`on_intel` URLs and
  sha256s from the real release. The tap repo (`coruairc/homebrew-deepseek-build`) is **not** created
  here; the activation command and the expected `brew install` flow are documented and recorded as an
  author step. Notarization is not required for a tap (no quarantine attribute on curl-downloaded
  tarballs); an ad-hoc signature is, and Phase 3 guarantees it.

### Cleanup
- The throwaway prerelease is **kept**, not deleted, because it is the only macOS artifact and the
  author wants a installable macOS build. Recorded explicitly as a deviation from the roadmap's
  "clean up" line, with the reason. No `:latest` is left pointing at it and `v0.1.0` stays unpublished.

### Decided autonomously
- [Tag] Q: "Push main first?" → Selected: "push the tag only" — cheap to undo; avoids moving the mainline for a proof.
- [Image] Q: "Build the image locally to prove it?" → Selected: "prove it from the registry side" — no runtime exists; a local build would prove a different artifact than a user pulls.
- [Homebrew] Q: "Create the tap repo?" → Selected: "no, the author decides it" — a new public repo is the author's decision.
- [Cleanup] Q: "Delete the prerelease after proving it?" → Selected: "keep it" — it is the macOS artifact the author asked for; deletion would remove the only way to install on macOS.

### the agent's Discretion
- Exact evidence item slugs, whether the image proof is one item or two, and the wording of the README
  section.

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `.github/workflows/release.yml` (post-Phase-3 split): `version`, `build-linux`, `build-macos`, `publish`, `container`.
- `install.sh` + `scripts/test-install.sh` (fake-tree harness; the real-release proof is new).
- `docker/Dockerfile` (production) and `docker/Dockerfile.localtest`.
- `docs/evidence/` convention and `scripts/evidence.py`.

### Established Patterns
- Evidence = asciinema cast + `report.json` + `notes.md`, paths normalized, leak-scanned.
- Every claim has a recorded command and an artifact hash.

### Integration Points
- `publish` creates the release from the four `release-<target>` artifacts.
- `container` stages per-arch contexts from the two Linux tarballs and pushes by digest.
- `install.sh` resolves `VERSION` or `latest` from the GitHub release URL.

</code_context>

<specifics>
## Specific Ideas

- The tag run is also REL-02's proof (both macOS legs must produce tarballs inside their budget), so the
  Phase-4 evidence doubles as the Phase-3 macOS proof.
- The GHCR package may be private by default; if a pull is attempted later it may need
  `gh auth refresh -s read:packages`.
- No `:latest` movement is the one hard invariant of this phase.

</specifics>

<deferred>
## Deferred Ideas

- Pulling and running the image (REL-05) — blocked on a container runtime; exact commands recorded.
- Creating the Homebrew tap repo — author action.
- `ci.yml`, debug symbols — v0.1.x.

</deferred>
