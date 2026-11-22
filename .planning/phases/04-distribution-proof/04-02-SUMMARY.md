---
phase: 04-distribution-proof
plan: "02"
status: complete
requirements: [REL-05]
verification: "docs only; in-image run DEFERRED (tests skipped by owner, 2026-11-22)"
---

# Plan 04-02: Headless image documentation

## Changes

- `README.md` Container section rewritten from the tree:
  - the image is **headless-only**; the interactive `docker run -it` example is
    replaced with version-check and `-p` headless examples;
  - a **Mounts** table (`/work`, optional `~/.deepseek-build`, `--user`);
  - a **Keeping the sandbox on** section: the sandbox is profile-selected with no
    disable flag, Linux needs `bubblewrap` plus user namespaces, a deny plan that
    cannot be bound makes the process refuse rather than degrade, and the
    minimal flags (`--security-opt seccomp=unconfined --cap-add SYS_ADMIN`) are
    documented with a preference for user namespaces over `--privileged`;
  - the stale "not yet built on rc4 / needs every target" banner is corrected:
    `container` now needs `build-linux` only, so the next tag builds the image
    even if a macOS leg is slow.
- `KNOWN-ISSUES.md`: the container entry records the decoupling, that no GHCR
  package exists yet, and that the new documentation is unexercised.

## Verification

- **DEFERRED (tests skipped by owner, 2026-11-22).** No image was built or
  pulled; the run flags are documented, not exercised.
