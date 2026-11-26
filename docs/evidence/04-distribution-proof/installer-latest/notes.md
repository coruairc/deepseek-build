# installer-latest

REL-06, unpinned half: **what the `latest` path really does** while only
pre-releases exist. It does not install anything. This item records the
observation instead of assuming it, and the README now says the same thing.

This is not a defect report against `install.sh`: the script asks GitHub for
`releases/latest/download/...`, and GitHub excludes pre-releases from `latest`
by definition. `install.sh` is byte-unchanged (see the `installer-proof` item);
no flag or behaviour was added to work around the absence of a stable release.

## Commands

```sh
sh scripts/prove-install.sh --latest-behaviour --workdir <TMP>/dsb-latest2.2Pd9TM
python3 scripts/check-readme-distribution.py --stage install
```

Observed 2026-11-26T06:19:23Z, exit 0, `10/10 PASS`.

## What was observed

Asset used for the failing run — the real release asset, not a fake tree:

```
https://github.com/coruairc/deepseek-build/releases/download/v0.1.0-proof1/install.sh
sha256 31e114d4a29dbbeb0d3e2e442a72a6ecc14ad3f4b99956fbff23f42c0cabd5e5
```

| URL | code (`-L`) | effective URL |
|-----|-------------|---------------|
| `https://github.com/coruairc/deepseek-build/releases/latest` | 302 → **200** | `https://github.com/coruairc/deepseek-build/releases` (the release index, not a tag page) |
| `https://github.com/coruairc/deepseek-build/releases/latest/download/SHA256SUMS` | **404** | itself |
| `https://github.com/coruairc/deepseek-build/releases/latest/download/install.sh` | **404** | itself |
| `https://api.github.com/repos/coruairc/deepseek-build/releases/latest` | **404** | REST `latest` = newest non-draft, non-prerelease |

Note on the first row: GitHub's **web** URL `/releases/latest` does not return
404 with no stable release; it redirects (302) to the repository's release
index, which is a 200 page. The **REST** endpoint
`repos/:owner/:repo/releases/latest` is the one that 404s, and that is the fact
04-01 recorded. Both are written down here because "404" alone would describe
the REST call, not what a browser sees. The two `download` URLs — the ones a
user's shell actually calls — are genuine 404s.

## What a user sees

```
$ curl -fsSL .../releases/latest/download/install.sh | sh
==> Resolving the latest release
curl: (22) The requested URL returned error: 404
```

Exit code 22, nothing installed, `INSTALL_DIR` not even created. The installer's
failure line names the URL indirectly: `curl -f` turns the 404 into exit 22
without echoing the URL, which is why `probe.log` records the URL and its status
code next to the transcript.

## Consequence, as written in the README

`README.md` §Install now states that `releases/latest` resolves only stable
releases, that pre-releases are excluded by definition, that both download URLs
return 404 as of 2026-11-26, and that `VERSION=v0.1.0-proof1` must be pinned
until a stable release exists. No unpinned install is claimed to work.

## Re-run

```sh
sh scripts/prove-install.sh --latest-behaviour --workdir "$(mktemp -d)"
```

Needs `gh`/`curl` and network; no Docker, no key, no sudo, no package install.
If a stable release is ever published, the script stops with a message instead
of asserting a 404 that is no longer true, and this item is superseded by a new
one.
