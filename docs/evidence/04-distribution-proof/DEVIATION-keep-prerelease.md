# Deviation: the throwaway pre-release is kept, not cleaned up

**Status:** recorded deviation, deliberate, in force.
**Contradicts:** `ROADMAP.md` → `### Phase 4: Distribution Proof` → `Success Criteria` item 4.
**Decision source:** `.planning/phases/04-distribution-proof/04-CONTEXT.md` §Cleanup and §Decided autonomously.

## The roadmap line

`ROADMAP.md` Phase 4 success criterion 4 reads:

> The throwaway tag, its release and the image tag it created are cleaned up,
> with no `:latest` left pointing at a throwaway build and the dry-run evidence
> committed.

This phase does **not** perform that cleanup. It does the opposite, by the
decision recorded before the tag existed in
[`04-CONTEXT.md`](../../../.planning/phases/04-distribution-proof/04-CONTEXT.md):

> §Cleanup — The throwaway prerelease is **kept**, not deleted, because it is
> the only macOS artifact and the author wants a installable macOS build.
> Recorded explicitly as a deviation from the roadmap's "clean up" line, with
> the reason. No `:latest` is left pointing at it and `v0.1.0` stays unpublished.

and in §Decided autonomously: *"[Cleanup] Q: 'Delete the prerelease after
proving it?' → Selected: 'keep it' — it is the macOS artifact the author asked
for; deletion would remove the only way to install on macOS."*

## What is retained

| Retained ref | What it is | Evidence |
|--------------|------------|----------|
| Tag `v0.1.0-proof1` | An annotated tag on `b14d28eb2bc71a046b789e0a705848d651685d35`; tag object `5006c519…` | [`tag-push/`](tag-push/) |
| GitHub pre-release `v0.1.0-proof1` | `prerelease: true`, `draft: false`, exactly six assets: four tarballs, `install.sh`, `SHA256SUMS` | [`tag-run/release.json`](tag-run/release.json) |
| GHCR image tags `0.1.0-proof1` and `v0.1.0-proof1` | Two tags on the multi-arch OCI index `sha256:8f5c484e…` | [`image-registry/registry.json`](image-registry/registry.json) |

The four tarballs are the only macOS artifacts this milestone has: the two
`*-apple-darwin.tar.gz` assets are ad-hoc signed and verified from the
extracted tarball by the release run itself. Deleting the release would remove
the only install path for macOS and the only macOS binary a tap formula can
point at (`packaging/homebrew/deepseek-build.rb` pins both of them).

## Invariants the retention keeps

The roadmap line has three parts; the retention keeps every part that protects
users, and skips only the deletion:

- **`v0.1.0` is unpublished.** No `refs/tags/v0.1.0` exists on `origin`, no
  `v0.1.0` release exists (404), and the release list is `v0.1.0-proof1`,
  `v0.0.1-rc5`, `v0.0.1-rc4` — every entry a pre-release. See
  [`tag-push/`](tag-push/) and [`tag-run/release.json`](tag-run/release.json).
- **No release is classified stable**, so `releases/latest` has no stable
  target: the REST endpoint `repos/coruairc/deepseek-build/releases/latest`
  returns 404 and the browser URL redirects to the release index, not to a
  tag page. See [`installer-latest/`](installer-latest/).
- **`:latest` is unchanged.** The pre-push baseline recorded
  `latest_status: absent` / `latest_digest: null`
  ([`ghcr-baseline/baseline.json`](ghcr-baseline/baseline.json)); after the
  push the registry reports the same: the tag list is exactly the two proof
  names, `latest` absent, and the index digest is not a `latest` target
  ([`image-registry/registry.json`](image-registry/registry.json)).

The earlier throwaway releases `v0.0.1-rc4` and `v0.0.1-rc5` are also still
published; this phase deleted none of them either.

## What this deviation does not do

- It executes no cleanup and records no cleanup command. Nothing here deletes
  a tag, a release, a GHCR version or a workflow run, and nothing instructs a
  later reader to.
- It does not publish `v0.1.0` or move `:latest`. Those remain the author's
  decisions at freeze time.
- It does not change any success criterion into a pass: ROADMAP Phase 4's
  criteria 1 and 2 (the image is **pulled** and runs a turn) stay unmet and are
  recorded as such through REL-04 `Partial` and REL-05 open.

## If the pre-release is ever removed

Removal is the author's decision, not this milestone's. If it happens, the
macOS install path goes with it, the Homebrew formula in the tree would point
at deleted assets, and the Phase 4 evidence would keep describing a release
that no longer exists. Any such removal should be its own dated decision that
re-points `packaging/homebrew/deepseek-build.rb` at `v0.1.0` after the author
publishes it, and should add a correction item under `docs/evidence/` rather
than edit this one.
