# Homebrew formula (macOS tap)

`deepseek-build.rb` in this directory is a Homebrew **binary** formula for the two macOS
tarballs of the `v0.1.0-proof1` pre-release. It downloads the release tarball and installs
the binary — there is no source build, no `head`, no `patch` and no Xcode dependency.

## State

The formula is **in-tree and not published**. The tap repository
`coruairc/homebrew-deepseek-build` does **not** exist; creating it is the author step below,
and this phase deliberately does not create a public repository.

No `brew install` has been executed against this formula: the tarballs contain macOS
binaries and the build host is Linux, so an install here would copy a Mach-O file into a
Linux prefix and prove nothing. What *has* run, on this host, with Homebrew 7.0.16
(Homebrew for Linux; no system `ruby` is installed, so `brew`'s bundled Ruby is what
evaluated the file):

| Check | Command | Result |
|-------|---------|--------|
| Style | `brew style local/deepseek-test/deepseek-build` | `1 file inspected, no offenses detected` |
| Audit | `HOMEBREW_SIMULATE_MACOS_ON_LINUX=1 brew audit --strict --formula …` | exit 0, no offenses |
| Audit (online) | `HOMEBREW_SIMULATE_MACOS_ON_LINUX=1 brew audit --strict --online --formula …` | exit 0, no offenses |
| Table integrity | `python3 scripts/check-homebrew-formula.py … --self-test --online` | `homebrew-formula: 19/19 PASS`, `self-test: 3/3 PASS` |
| Resolve (arm64) | `brew ruby` + `Homebrew::SimulateSystem.with(os: :macos, arch: :arm)` | the `on_arm` URL and `2b61dfab…` |
| Resolve (x86_64) | same, `arch: :intel` | the `on_intel` URL and `ffe0f3c5…` |
| Fetch + checksum | `HOMEBREW_SIMULATE_MACOS_ON_LINUX=1 brew fetch --formula …` | 53,850,000 bytes, checksum verified |
| Mismatch control | same fetch against a copy with one wrong `sha256` | rejected: `reports different checksum` |

`HOMEBREW_SIMULATE_MACOS_ON_LINUX=1` is Homebrew's own switch for evaluating a macOS-only
formula on a Linux host; without it, `brew` cannot load this formula at all and says
`formula requires at least a URL`, because no `on_macos` block applies on Linux. That is a
property of auditing a macOS-only formula on Linux, not a defect in the macOS formula: the
`on_arm`/`on_intel` branches supply the URL on the platform the formula targets.

The full output is committed under
`docs/evidence/04-distribution-proof/homebrew-formula/` (`report.json` plus one step log per
check).

## Activate (author step, once)

Nothing in this repository publishes the tap. To make the formula installable, the author
creates the public tap repository and copies the formula into it:

```sh
gh repo create coruairc/homebrew-deepseek-build --public \
  --description "Homebrew tap for deepseek-build"
git clone git@github.com:coruairc/homebrew-deepseek-build.git
mkdir -p homebrew-deepseek-build/Formula
cp packaging/homebrew/deepseek-build.rb homebrew-deepseek-build/Formula/
cd homebrew-deepseek-build
git add Formula/deepseek-build.rb
git commit -m "deepseek-build 0.1.0-proof1 (macOS arm64 + x86_64)"
git push origin HEAD
```

Consumers then run:

```sh
brew tap coruairc/deepseek-build
brew install deepseek-build
brew test deepseek-build
```

`brew tap coruairc/deepseek-build` resolves `https://github.com/coruairc/homebrew-deepseek-build`
by Homebrew's naming rule (the `homebrew-` prefix is implied), which is why the repository
must be named exactly that. The tap repository must be public.

The tap is **macOS only**. On Homebrew for Linux the formula has no URL for the current
platform and `brew` reports `formula requires at least a URL`; Linux users should use
`install.sh` instead (see the top-level `README.md`).

## What a user gets

- `deepseek-build` on `PATH`, installed from the release tarball byte-for-byte.
- A `dsb` symlink to the same binary (the shortcut `install.sh` also creates).
- `LICENSE`, `NOTICE`, `THIRD-PARTY-NOTICES` and `README.txt` under
  `$(brew --prefix)/share/doc/deepseek-build/`.
- No build step: `brew install` extracts the published tarball, which Homebrew verifies
  against the formula's `sha256` before installing.

## Signature and quarantine

The macOS binaries are **ad-hoc signed** (Phase 3 signs every darwin artifact) and **not
notarized**; they carry no Apple Developer ID signature. That is sufficient for a tap:

- A tarball fetched by `brew` (or `curl`) carries **no** `com.apple.quarantine` attribute,
  so Gatekeeper does not prompt on first run.
- A copy downloaded through a browser may be quarantined. If macOS refuses to run it, clear
  the attribute: `xattr -d com.apple.quarantine /path/to/deepseek-build`.
- Notarization is **not** required to distribute through a Homebrew tap. It becomes
  relevant only if the binary is distributed as a signed `.app`/`.pkg` outside a package
  manager.

## When `v0.1.0` is published

The formula is pinned to the throwaway proof pre-release on purpose. When `v0.1.0` is
published:

1. Take the two new darwin hashes from that release's `SHA256SUMS` — never from a local
   build:

   ```sh
   curl -fsSL https://github.com/coruairc/deepseek-build/releases/download/v0.1.0/SHA256SUMS \
     | grep apple-darwin
   ```

2. Update `version`, both `url`s and both `sha256`s in `packaging/homebrew/deepseek-build.rb`.
3. Re-run the checker against the new sums and the new release metadata:

   ```sh
   python3 scripts/check-homebrew-formula.py \
     --formula packaging/homebrew/deepseek-build.rb \
     --sums <new SHA256SUMS> --release-json <new release.json> --self-test --online
   ```

4. Only then copy the file into the tap repository and commit it there.

The `livecheck` block is an explicit `skip`: `brew livecheck` would otherwise query
`api.github.com` for the latest tag, and a proof pre-release is not a release to track.
Remove the `skip` and point the formula at `v0.1.0` in step 2 if automatic bumping is wanted
later.
