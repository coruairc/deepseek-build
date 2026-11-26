# installer-proof

REL-06, pinned half: the **published** installer, run against the **published**
release, installs the pinned version and the installed binary reports the tag.

This item is the requirement's evidence. `scripts/test-install.sh` (39/39, the
loopback fake-release tree) is **not** used here and proves nothing about the
real assets; it remains an unchanged offline unit check of `install.sh`'s
script logic. Nothing in this item touched 127.0.0.1.

## Commands

```sh
sh scripts/prove-install.sh --version v0.1.0-proof1 --workdir <TMP>/dsb-proof1.3T82SU
sh scripts/prove-install.sh --version v0.0.1-rc5    --workdir <TMP>/dsb-rc5b.FLvQ19
```

`install.log` holds both transcripts: Section 1 is the thin slice against
`v0.0.1-rc5` (the release that already existed before this phase, used to
validate the harness on known-good bytes), Section 2 is the `v0.1.0-proof1`
proof. Both exited 0 with `13/13 PASS`.

The script refuses to run if the installer's test-only override hooks are
present in the environment, so it cannot silently certify a local fake tree.
Its names are assembled from a suffix and never written out in the file; the
plan's guard (`grep` for a hook name in `scripts/prove-install.sh` returns
nothing) holds. The URL is printed and asserted to be on `github.com`.

## The release URL and the three hashes (host x86_64 linux)

```
https://github.com/coruairc/deepseek-build/releases/download/v0.1.0-proof1/deepseek-build-0.1.0-proof1-x86_64-unknown-linux-gnu.tar.gz
```

| Source | sha256 |
|--------|--------|
| local `sha256sum` of the downloaded tarball | `298c9f275d695fa43872bd44a0fe9da25ee66bd86a6051ac233689063ccbdaf3` |
| the release's own `SHA256SUMS` line | `298c9f275d695fa43872bd44a0fe9da25ee66bd86a6051ac233689063ccbdaf3` |
| GitHub's asset `digest` | `sha256:298c9f275d695fa43872bd44a0fe9da25ee66bd86a6051ac233689063ccbdaf3` |

Size: 55,967,373 bytes downloaded, 55,967,373 bytes in GitHub's asset metadata.
`sha256sum -c` against the release `SHA256SUMS` (host line): `: OK`.

The installer asset is the same file this tree ships:

| Source | sha256 |
|--------|--------|
| downloaded release asset `install.sh` | `31e114d4a29dbbeb0d3e2e442a72a6ecc14ad3f4b99956fbff23f42c0cabd5e5` |
| `sha256sum install.sh` in the working tree | `31e114d4a29dbbeb0d3e2e442a72a6ecc14ad3f4b99956fbff23f42c0cabd5e5` |
| GitHub's asset `digest` | `sha256:31e114d4a29dbbeb0d3e2e442a72a6ecc14ad3f4b99956fbff23f42c0cabd5e5` |

`install.sh` is byte-unchanged in this phase: the release asset pushed by
04-01's `publish` job is the file in the tree, so the proof exercised exactly
the bytes a user downloads.

## What the run did

The downloaded asset was executed as

```sh
HOME=<TMP>/dsb-proof1.3T82SU/home INSTALL_DIR=<TMP>/dsb-proof1.3T82SU/bin \
    VERSION=v0.1.0-proof1 sh <asset>/install.sh
```

- exit 0;
- printed `Checksum OK (298c9f27…)` — i.e. it verified the tarball before
  extracting;
- printed `Installed v0.1.0-proof1`;
- installed an executable `deepseek-build` and the `dsb` symlink in the scratch
  `INSTALL_DIR` (no sudo, nothing written outside the scratch tree);
- the installed binary printed **`deepseek-build 0.1.0-proof1 (b14d28eb2bc7)`**.

That version string is the tag's target commit (`b14d28eb2bc7…`), matching
04-01's release record.

The `sha256sum -c` and the second download are independent of the installer's
own check: the proof fetched the tarball once itself to hash it, then the
installer fetched it again and verified it. Both fetched the same bytes.

## Re-run

```sh
sh scripts/prove-install.sh --version v0.1.0-proof1 --workdir "$(mktemp -d)"
python3 scripts/check-readme-distribution.py --stage install   # the README claims
```

Needs `gh` authenticated (any read scope for a public repo), `curl`,
`sha256sum`/`shasum`, ~400 MiB free, and the release to still exist. No Docker,
no key, no sudo, no package install.
