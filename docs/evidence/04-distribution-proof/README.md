# Phase 4 — Distribution Proof: evidence index

Nine items back this phase's claims. Eight were recorded by plans 04-01 to
04-04; the ninth, [`seal`](seal/), is this plan's own record. Every item is a
directory with a `report.json`, the captured logs and a `notes.md`; the
convention (the fixed eight-key report, path normalization, the frozen-item
rule) is [`docs/evidence/README.md`](../../README.md).

`python3 scripts/check-distribution-evidence.py docs/evidence/04-distribution-proof`
re-reads all nine reports, re-asserts the schema and recomputes every artifact
sha256, so a report cannot drift from the bytes it claims to prove. Add
`--self-test` to mutate a temp copy four ways (missing artifact, wrong hash,
REL-05 marked `ok`, NUL byte) before the real run. Every artifact is text and
under 1 MiB; `find docs/evidence/04-distribution-proof -type f -size +1M`
is empty.

## Items

| Item | Requirements | Result | What it proves | Re-run (verbatim `report.json.command`) |
|------|--------------|--------|----------------|------------------------------------------|
| [`tag-push`](tag-push/) | REL-04 | `ok` | The one tag-only push: `v0.1.0-proof1` pushed alone from `b14d28eb`, the remote tag peels to the same commit, `origin/main` (`6e9d8d88…`) and `deepseek`/`deepseek-v0.1-snapshot` (`4b305bd0…`) did not move, no `v0.1.0` tag exists, and the image-tag guard accepts both the pushed two-name and the predicted four-name prerelease lists. | `read-only re-measurement of the single tag push: sh scripts/check-release-pipeline.sh, scripts/guard-image-tags.sh on the pushed and predicted tag lists, git ls-remote for the tag/main/snapshot/v0.1.0 refs, and gh run list for the push run` |
| [`tag-run`](tag-run/) | REL-04 | `ok` | Release run [45724842211](https://github.com/coruairc/deepseek-build/actions/runs/45724842211) is green (7/7 jobs; macOS 47.8 and 61.5 min against the 240-minute budget, container success); the release is a pre-release with exactly six assets, every digest equal to its `SHA256SUMS` line and GitHub's asset digest, both macOS binaries ad-hoc signed and verifiable from the extracted tarball, and no `v0.1.0`/`latest` release exists. | `python3 scripts/distribution-proof.py run --sha b14d28eb2bc71a046b789e0a705848d651685d35 --out run.json; python3 scripts/distribution-proof.py release --tag v0.1.0-proof1 --dir <TMP>/assets --out release.json; sha256sum -c SHA256SUMS in the asset dir` |
| [`ghcr-baseline`](ghcr-baseline/) | REL-04 | `ok` | The pre-push registry baseline the `:latest` invariant is measured against (tags exactly `0.1.0-proof1` and `v0.1.0-proof1`, `:latest` absent), the harness's 35 offline negative fixtures, and a `v0.0.1-rc5` release download whose SHA-256 matches both its `SHA256SUMS` line and GitHub's digest. | `python3 scripts/distribution-proof.py fixtures && python3 scripts/distribution-proof.py probe --tag v0.0.1-rc5 --out docs/evidence/04-distribution-proof/ghcr-baseline/probe-rc5.json && python3 scripts/distribution-proof.py baseline --out docs/evidence/04-distribution-proof/ghcr-baseline/baseline.json` |
| [`installer-proof`](installer-proof/) | REL-06 | `ok` | A pinned `VERSION=v0.1.0-proof1` install from the real release: the release asset `install.sh` is byte-identical to the tree (`31e114d4…`), the host tarball's sha256 equals its `SHA256SUMS` line and GitHub's digest (`298c9f27…`), and the installed binary prints `deepseek-build 0.1.0-proof1 (b14d28eb2bc7)`; `install.sh` itself is unmodified. | `sh scripts/prove-install.sh --version v0.1.0-proof1 --workdir <TMP>/dsb-proof1c.3kbDzq (plus the v0.0.1-rc5 thin slice) — real github.com release assets, scratch HOME/INSTALL_DIR, no fake-tree hook` |
| [`installer-latest`](installer-latest/) | REL-06 | `ok` | The unpinned `latest` behaviour, observed rather than assumed: both `/releases/latest/download/…` URLs return 404, `/releases/latest` redirects to the release index while the REST `releases/latest` returns 404, and an unpinned install exits non-zero having installed nothing. | `sh scripts/prove-install.sh --latest-behaviour --workdir <TMP>/dsb-latest2.2Pd9TM; python3 scripts/check-readme-distribution.py --stage install` |
| [`homebrew-formula`](homebrew-formula/) | PKG-04 | `ok` | `packaging/homebrew/deepseek-build.rb` pins the two `v0.1.0-proof1` darwin URLs and SHA-256 values; Homebrew 7.0.16 reports `no offenses detected` on `brew style` and exit 0 on `brew audit --strict [--online]` (`HOMEBREW_SIMULATE_MACOS_ON_LINUX=1`), both arch branches resolve, `brew fetch` verified 53,850,000 bytes and a wrong-hash control was rejected. `brew install` is a documented macOS author step, not run here. | `Homebrew 7.0.16 on Linux: throwaway local tap, brew style + brew audit --strict [--online] (HOMEBREW_SIMULATE_MACOS_ON_LINUX=1), per-arch brew ruby resolution, brew fetch checksum + mismatch control; brew install NOT RUN (macOS binaries on a Linux host)` |
| [`image-registry`](image-registry/) | REL-04 | `ok` | An anonymous HTTPS read of `ghcr.io/coruairc/deepseek-build:v0.1.0-proof1`: OCI index `sha256:8f5c484e…` equal to the CI `image_digest`, `linux/amd64` + `linux/arm64` platform manifests, exactly the two proof tags and no `:latest` (absent after == absent baseline), the package public, and the config blob read (`WorkingDir=/work`, `User=1000:1000`, `Entrypoint=["deepseek-build"]`, no API-key variable). | `python3 scripts/distribution-proof.py fixtures && python3 scripts/distribution-proof.py registry --tag v0.1.0-proof1 --baseline docs/evidence/04-distribution-proof/ghcr-baseline/baseline.json --expect-digest sha256:8f5c484ed067143aab93aa90b1441101fd643a37924f1868eeaaa103d1f199be --out docs/evidence/04-distribution-proof/image-registry/registry.json` |
| [`image-run`](image-run/) | REL-05 (run half) | `ok` | The published image, pulled **by index digest**, runs a real headless tool-executing turn: `--version` prints `deepseek-build 0.1.0-proof1 (b14d28eb2bc7)`, the turn ran `printf 'in-image-ok\n'` and returned it with `estimated_cost_usd: 0.000347148` / `estimate: true` (`deepseek-flash`). The same item records the limitation: **`bwrap` is absent from the image**, so the in-process sandbox cannot be applied inside the container. | `docker pull ghcr.io/…@sha256:8f5c484e…; docker run … --version; docker run … -p "…printf 'in-image-ok'…" --output-format json --permission-mode auto; docker run --entrypoint sh … -c '! command -v bwrap'` |
| [`image-run-blocked`](image-run-blocked/) | REL-05 | `skipped` | **Superseded by `image-run`.** Historical record of the pre-reboot state: `docker info` exited 1 and the commands were parked for a host with a runtime. | `docker info` |
| [`seal`](seal/) | REL-04, REL-06, PKG-04 | `ok` | This plan's mechanical seal: the eight reports above validate (`distribution-evidence: 8/8 PASS` at seal time), the self-test rejects all four mutations, the leak scan is clean, the retention invariants hold (no `:latest`, no `v0.1.0`, snapshot/branch untouched), and the keep-the-prerelease deviation is recorded. REL-05 is deliberately **not** in this item's `requirements`: the checker refuses `ok` for any item claiming it without a recorded pull. | `python3 scripts/check-distribution-evidence.py docs/evidence/04-distribution-proof --self-test; scripts/key-leak-check.sh --scan docs/evidence .planning/phases/04-distribution-proof` |

## Deviation: the pre-release is kept

Roadmap Phase 4 success criterion 4 says the throwaway tag, its release and the
image tags are cleaned up. They are **not**: the tag `v0.1.0-proof1`, its
GitHub pre-release (six assets) and its GHCR tags are retained, because that
release is the milestone's only macOS artifact. The decision and its invariants
are recorded in [`DEVIATION-keep-prerelease.md`](DEVIATION-keep-prerelease.md).

## Aggregate outcome

REL-04 is **partial**: the image is built and published by the tag run and the
registry side is proven from this host (index digest equal to the CI digest,
both platforms, two tags, no `:latest`), but the pull named by the roadmap was
not executed because no container runtime exists on this host. REL-05's **run half is now proven** (`image-run`): the image was pulled by digest and completed a real headless shell turn. Its **sandbox clause is not met**: the image ships no `bwrap`, so `--sandbox` cannot be enforced inside the container. REL-05 therefore stays **open** on that clause, and the README no longer claims otherwise.
REL-06 is **closed**: the pinned install from the real release prints the tag
version and the unpinned `latest` path was observed to 404. PKG-04 is
**closed** for the in-tree formula plus the documented tap activation (`brew
install` itself is an author step on macOS). Phase 4 is 5/5 plans executed,
with 2 requirements partially/blocked — not a fully complete phase.
