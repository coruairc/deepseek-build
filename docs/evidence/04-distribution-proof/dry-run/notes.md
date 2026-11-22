# dry-run

Proves REL-04 and REL-05: a throwaway pre-release tag produces a GHCR image, the
image is pulled from the registry (not just built locally), and it runs a real
headless turn that executes a shell command with the sandbox on.

## Status: BLOCKED — DEFERRED (tests skipped by owner, 2026-11-22; owner rule: never push/tag)

This run is not allowed to push or tag, so no throwaway tag exists and no image
was built or pulled. This item is `result: skipped`.

The code side is in place: `release.yml` splits `build-linux`/`build-macos` and
`container` needs `build-linux` only (so a slow macOS leg cannot skip the image),
and `README.md` documents the image as headless-only with its mounts and the
flags that keep the sandbox on.

## Re-run (only with tag-push permission and a key)

```sh
# 1. Throwaway pre-release tag from the commit to prove.
git tag v0.1.0-rc9
git push origin v0.1.0-rc9
gh run watch "$(gh run list --workflow=release.yml --limit 1 --json databaseId -q '.[0].databaseId')"

# 2. Pull the image from the registry (not a local build).
docker pull ghcr.io/coruairc/deepseek-build:0.1.0-rc9

# 3. Headless turn that runs a shell command, sandbox on.
docker run --rm --security-opt seccomp=unconfined --cap-add SYS_ADMIN \
  -e DEEPSEEK_API_KEY -v "$PWD":/work \
  ghcr.io/coruairc/deepseek-build:0.1.0-rc9 \
  -p "create hello.txt containing hi, then print it"

# 4. Capture with script -T, record steps with scripts/evidence.py, then:
docker rmi ghcr.io/coruairc/deepseek-build:0.1.0-rc9
gh release delete v0.1.0-rc9 --yes
git push origin :refs/tags/v0.1.0-rc9
git tag -d v0.1.0-rc9
# confirm :latest was not moved by the pre-release
```

If the image cannot create namespaces, the process must fail closed rather than
run unsandboxed; record that as a finding before relaxing any flag.
