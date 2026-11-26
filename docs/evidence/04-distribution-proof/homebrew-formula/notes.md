# homebrew-formula

Proves the in-tree Homebrew formula (`packaging/homebrew/deepseek-build.rb`) against the
release's own `SHA256SUMS` and against Homebrew itself — and records, rather than hides, the
one thing this host cannot do: install a macOS binary.

## What the 15 steps show

| Step | What it asserts | Log |
|------|-----------------|-----|
| `checker: 15 offline assertions` | 15/15 PASS over the formula text + release sums + release.json | `checker-15-offline-assertions.log` |
| `checker: --online 19 assertions` | + both URLs answer `200` with the release's `Content-Length` | `checker-online-19-assertions.log` |
| `checker: --self-test 3 mutations` | wrong sha256, missing `on_intel`, non-github.com host are each rejected by a named assertion | `checker-self-test-3-mutations.log` |
| `checker.log: committed log carries 19/19 and 3/3` | the committed `checker.log` still holds both summary lines | `checker-log-committed-log-carries-19-19-and-3-3.log` |
| `tap: local/deepseek-test carries the committed formula` | the throwaway tap's copy hashes equal to the in-tree file | `tap-local-deepseek-test-carries-the-committed-formula.log` |
| `brew style: no offenses` | `1 file inspected, no offenses detected` | `brew-style-no-offenses.log` |
| `brew audit --strict: exit 0` | no offenses | `brew-audit-strict-exit-0.log` |
| `brew audit --strict --online: exit 0` | no offenses, including the online checks | `brew-audit-strict-online-exit-0.log` |
| `brew ruby: on_arm resolves the aarch64 asset` | Homebrew resolves the aarch64 URL and `2b61dfab…` | `brew-ruby-on-arm-resolves-the-aarch64-asset.log` |
| `brew ruby: on_intel resolves the x86_64 asset` | Homebrew resolves the x86_64 URL and `ffe0f3c5…` | `brew-ruby-on-intel-resolves-the-x86-64-asset.log` |
| `brew fetch: intel tarball downloads and verifies` | the real download passes Homebrew's checksum gate | `brew-fetch-intel-tarball-downloads-and-verifies.log` |
| `brew fetch: 53,850,000 bytes, sha256 ffe0f3c5` | the fetched bytes are the release asset | `brew-fetch-53-850-000-bytes-sha256-ffe0f3c5.log` |
| `control: a wrong sha256 is rejected` | a formula with a wrong hash is refused (`reports different checksum`) | `control-a-wrong-sha256-is-rejected.log` |
| `linux host: audit names the macOS-only limitation` | without simulation, audit says `formula requires at least a URL` | `linux-host-audit-names-the-macos-only-limitation.log` |
| `brew install: NOT RUN` | the reason and the exact macOS re-run commands | `brew-install-not-run.log` |

`brew fetch` is the strongest Homebrew-level fact available here: Homebrew downloaded the
x86_64 tarball and its own checksum gate accepted `ffe0f3c5…`. The wrong-sha256 control then
proves that gate is actually engaged — it fails with
`Formula wrongsha (0.1.0-proof1) reports different checksum`.

## What is NOT claimed

- **No `brew install`.** The tarballs hold macOS Mach-O binaries and this host is Linux; an
  install would copy a darwin binary into a Linux prefix and prove nothing. The step is
  recorded as NOT RUN with the re-run commands for macOS.
- **No public tap.** `coruairc/homebrew-deepseek-build` does not exist. The tap used here is
  a throwaway local one under Homebrew's own taps directory.
- **No Ruby-semantics check by the Python checker.** `scripts/check-homebrew-formula.py`
  asserts text only; the Ruby behaviour is the `brew` steps above. No system `ruby` is
  installed — `brew`'s bundled Ruby is what ran.

## How to re-run

```sh
export HOMEBREW_NO_AUTO_UPDATE=1 HOMEBREW_NO_ANALYTICS=1
eval "$(/home/linuxbrew/.linuxbrew/bin/brew shellenv)"

# 1. the text-level checker (offline and online)
python3 scripts/check-homebrew-formula.py \
  --formula packaging/homebrew/deepseek-build.rb \
  --sums docs/evidence/04-distribution-proof/tag-run/SHA256SUMS \
  --release-json docs/evidence/04-distribution-proof/tag-run/release.json \
  --self-test --online

# 2. a throwaway tap that carries the formula
brew tap-new --no-git local/deepseek-test
brew trust local/deepseek-test          # Homebrew 7 refuses load from an untrusted tap
install -m 644 packaging/homebrew/deepseek-build.rb \
  "$(brew --repository)/Library/Taps/local/homebrew-deepseek-test/Formula/deepseek-build.rb"

# 3. style and audit (the macOS-only formula needs Homebrew's simulated macOS on Linux)
brew style local/deepseek-test/deepseek-build
HOMEBREW_SIMULATE_MACOS_ON_LINUX=1 brew audit --strict --formula local/deepseek-test/deepseek-build
HOMEBREW_SIMULATE_MACOS_ON_LINUX=1 brew audit --strict --online --formula local/deepseek-test/deepseek-build

# 4. per-arch resolution and a real fetch
HOMEBREW_SIMULATE_MACOS_ON_LINUX=1 brew ruby docs/evidence/04-distribution-proof/homebrew-formula/resolve-branch.rb arm
HOMEBREW_SIMULATE_MACOS_ON_LINUX=1 brew ruby docs/evidence/04-distribution-proof/homebrew-formula/resolve-branch.rb intel
HOMEBREW_SIMULATE_MACOS_ON_LINUX=1 brew fetch --force --formula local/deepseek-test/deepseek-build
```

`HOMEBREW_SIMULATE_MACOS_ON_LINUX=1` is Homebrew's own mechanism for evaluating a macOS
formula on Linux (`Homebrew::EnvConfig.simulate_macos_on_linux?`); it does not fabricate a
download or a checksum, it only lets the `on_macos` branch be selected. Without it the
formula cannot load at all on Linux, which step 14 records.

## The values this item pins

| Arch | URL | sha256 |
|------|-----|--------|
| arm64 | `https://github.com/coruairc/deepseek-build/releases/download/v0.1.0-proof1/deepseek-build-0.1.0-proof1-aarch64-apple-darwin.tar.gz` | `2b61dfab870b397b49ed810f53e073714582e01dbf5c6f9d166656367c51324b` |
| x86_64 | `https://github.com/coruairc/deepseek-build/releases/download/v0.1.0-proof1/deepseek-build-0.1.0-proof1-x86_64-apple-darwin.tar.gz` | `ffe0f3c5f76be9ee6fa2dff1d53c34de6dce66d3553a094ee62524f58f57b3a7` |

Both are lines of `tag-run/SHA256SUMS` and both equal GitHub's asset digest in
`tag-run/release.json`; sizes are 49,591,197 and 53,850,000 bytes.
