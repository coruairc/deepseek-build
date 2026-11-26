# image-registry — what the registry serves for `v0.1.0-proof1`, read without a daemon

Requirement: REL-04 (registry half). Item recorded 2026-11-26 by plan 04-04.

## Read this first: this is a registry read, not a pull

`registry.json` proves what `ghcr.io` **serves** for `ghcr.io/coruairc/deepseek-build:v0.1.0-proof1`:
one OCI index, two platform manifests, the tag set, and that `:latest` does not exist. It does **not**
prove that a container can run: this host has the Docker CLI but no daemon (see `docker-info.log`),
so no `docker pull`, `docker run`, `docker build` or `docker login` was attempted by this item.
The in-image run (REL-05) is recorded separately as environment-blocked in
[`../image-run-blocked/`](../image-run-blocked/).

## The measured facts

| Field | Value |
|-------|-------|
| Tag read | `v0.1.0-proof1` |
| Transport | `anonymous-http` — `https://ghcr.io/token` (pull scope) then the v2 manifest/tags/blob API |
| Visibility | `public` — the anonymous pull token answered `200` for the manifest |
| Index digest | `sha256:8f5c484ed067143aab93aa90b1441101fd643a37924f1868eeaaa103d1f199be` |
| Content type | `application/vnd.oci.image.index.v1+json` |
| `linux/amd64` manifest | `sha256:1d216b609ed9c646cb56a5e6294bc3de5276b2823d2a90a0252b49d90b99886e`, 1810 bytes, `application/vnd.oci.image.manifest.v1+json` |
| `linux/arm64` manifest | `sha256:25dc0d256765ff129b14749732f5d158b1757a540828c643410a3f142dcb6f58`, 1810 bytes, `application/vnd.oci.image.manifest.v1+json` |
| Tags | `0.1.0-proof1`, `v0.1.0-proof1` (exactly these two) |
| `:latest` | **404 / absent** — `latest_status: absent`, `latest_digest: null` |
| Baseline `:latest` | `absent` / `null` (from `../ghcr-baseline/baseline.json`) |
| Config blob | **read** — `sha256:a3f442dd7160476285a75f6c893c85a93958fd010664112dccca898d062b17f7`, 4091 bytes |
| `User` / `WorkingDir` / `Entrypoint` | `1000:1000` / `/work` / `["deepseek-build"]` |
| Config env names | `PATH`, `DEBIAN_FRONTEND` — no `API_KEY` name |
| `docker info` | exit 1: `Cannot connect to the Docker daemon at unix:///var/run/docker.sock. Is the docker daemon running?` |

The index digest is byte-for-byte the `image_digest` the CI run itself reported in
`../tag-run/run.json` (`sha256:8f5c484e…`), so the registry serves the artifact the pipeline built.

### The index has four descriptors, two of which are attestations

The raw index carries two extra `unknown/unknown` descriptors — buildx's provenance/SBOM
attestation manifests, each annotated `vnd.docker.reference.type=attestation-manifest` and pointing
at its platform manifest. They are recorded separately in `registry.json`
(`attestation_manifests`) and are never counted as platforms:

| Attestation | Size | Attests |
|-------------|------|---------|
| `sha256:2ca7c01fbfc2a33ac532632997e32254df269d88f27c08e6c7e1adcad84b5aa2` | 838 | `sha256:1d216b60…` (amd64) |
| `sha256:b0ca3b1d32738af71312a7d9fdec4a4150ea1f0620d1464aafa739c2f2ed19df` | 838 | `sha256:25dc0d25…` (arm64) |

That is why the platform assertion is "exactly two **platform** manifests" rather than "four
descriptors": a reader running `docker manifest inspect` sees all four.

## How it was measured

`python3 scripts/distribution-proof.py registry --tag v0.1.0-proof1 --baseline
docs/evidence/04-distribution-proof/ghcr-baseline/baseline.json --expect-digest
sha256:8f5c484ed067143aab93aa90b1441101fd643a37924f1868eeaaa103d1f199be --out …`.

`registry` tries three transports in order and records the one that answered in `transport` and
`transports_tried`:

1. **anonymous `ghcr.io`** — `GET /token?service=ghcr.io&scope=repository:coruairc/deepseek-build:pull`,
   then `GET /v2/coruairc/deepseek-build/manifests/v0.1.0-proof1` with the OCI index/list accept set,
   `/tags/list`, `/manifests/latest`, and — when the manifest is readable — the amd64 manifest's
   config blob. Pure `urllib`; no Docker socket.
2. **`docker manifest inspect`** — a registry client that prints the index's descriptors. It does not
   print the index digest or the tag list, so this fallback records `index_digest: null` and lets the
   assertion name the gap instead of guessing. **No Docker daemon is contacted.**
3. **`gh api /users/coruairc/packages/container/deepseek-build/versions`** — a credentialed read;
   `403` for this account's token (`read:packages` is missing), recorded as an authorization gap,
   never as a registry result.

Transport 1 answered, so `visibility` is `public` (an anonymous read succeeded) and the config blob
could be read. No token value, `Authorization` header, `gh auth token` output or `~/.docker/config.json`
is recorded anywhere in this item; the scripts print only the transport name and HTTP status.

## `:latest` — unchanged absence

There is still no `:latest`: the manifest read answers `404`, and the baseline file (recorded
post-push in 04-01) also has `latest_status: absent`, `latest_digest: null`. Both sides agree, and the
container job's own log recorded the prerelease branch (`Pre-release — :latest unchanged`). The
baseline's honest limitation is documented in `../ghcr-baseline/notes.md`: the pre-push moment was
never captured, so the invariant rests on the post-push 404 plus the workflow's structural rule that
only a stable tag creates `:latest`.

## What is still not proven

- **The pull.** `docker pull`/`docker run` were not run: the daemon is absent (`docker-info.log`) and
  cannot be started without a password (`sudo` prompts). The registry read is the ceiling of this host.
- **A run inside the image.** REL-05 is deferred with the exact digests and commands in
  `../image-run-blocked/`.

## Files

| File | Meaning |
|------|---------|
| `registry.json` | the machine-read registry payload: transport, digests, platforms, attestations, tags, `:latest` comparison, config blob |
| `docker-info.log` | the one `docker info` run, verbatim, with its `exit=1` line |
| `report.json` | the item record: 8 assertion steps, `result: ok` |
| `*.log` | per-step assertion output (normalized) |

## Re-run

```sh
python3 scripts/distribution-proof.py fixtures
python3 scripts/distribution-proof.py registry \
  --tag v0.1.0-proof1 \
  --baseline docs/evidence/04-distribution-proof/ghcr-baseline/baseline.json \
  --expect-digest sha256:8f5c484ed067143aab93aa90b1441101fd643a37924f1868eeaaa103d1f199be \
  --out /tmp/registry.json
```
