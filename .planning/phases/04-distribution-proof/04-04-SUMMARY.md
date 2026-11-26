---
phase: 04-distribution-proof
plan: 04
subsystem: distribution
tags: [ghcr, oci-index, registry, docker, rel-04, rel-05, readme, evidence]
requires:
  - plan: "04-01"
    provides: "tag-run/run.json image_digest (sha256:8f5c484e…), the two pushed tag names, and ghcr-baseline/baseline.json (:latest absent)"
  - plan: "04-02"
    provides: "the README install section this plan leaves untouched, plus check-readme-distribution.py's install stage"
provides:
  - "scripts/distribution-proof.py registry — anonymous-HTTPS index/platform/tags/:latest/config read, with docker manifest inspect and gh api fallbacks (no daemon)"
  - "docs/evidence/04-distribution-proof/image-registry — registry.json, docker-info.log, 8-step ok report, notes"
  - "docs/evidence/04-distribution-proof/image-run-blocked — result skipped, digest-pinned commands.txt, notes, report"
  - "README ## Container — real index and platform digests, two tags and no :latest, public visibility, headless-only contract, both mounts, not-executed statement"
  - "the registry-side half of REL-04 and the exact REL-05 reproduce commands for a host with a container runtime"
affects: [04-05]

actuals:
  tokens: 28677          # chars/4 over the 18 changed files (114,711 bytes); the estimate (115k) was ~4x high
  tasks: 2
  commits: 2             # measured 06ec92f6..HEAD; the metadata commit carrying this file is the 3rd

tech-stack:
  added: []              # stdlib Python only — no package manifest changed, nothing installed
  patterns:
    - "Registry reads over an anonymous pull token + urllib; `docker manifest inspect` is a registry client and never contacts a daemon"
    - "An OCI index is judged by its platform manifests; buildx attestation descriptors (`unknown/unknown`, annotated) are recorded separately and never counted as platforms"
    - "An environment-blocked item is `result: skipped` with digest-pinned commands — never `ok`, never a claim of execution"

key-files:
  created:
    - docs/evidence/04-distribution-proof/image-registry/registry.json
    - docs/evidence/04-distribution-proof/image-registry/docker-info.log
    - docs/evidence/04-distribution-proof/image-registry/report.json
    - docs/evidence/04-distribution-proof/image-registry/notes.md
    - docs/evidence/04-distribution-proof/image-run-blocked/commands.txt
    - docs/evidence/04-distribution-proof/image-run-blocked/report.json
    - docs/evidence/04-distribution-proof/image-run-blocked/notes.md
  modified:
    - scripts/distribution-proof.py
    - README.md
    - docs/evidence/04-distribution-proof/image-registry/notes.md   # committed in task 2 with the README pointer

key-decisions:
  - "The index carries four descriptors: two platform manifests (linux/amd64, linux/arm64) and two buildx attestation manifests (`unknown/unknown`). registry.json records the attestations in their own key; the assertion counts exactly two *platform* manifests, which is what a pull resolves."
  - "The registry read went over the anonymous ghcr.io token, so the observed visibility is `public` and the README omits the login command (the plan required it only when private)."
  - "The config blob was readable, so the README cites the registry config (`WorkingDir=/work`, `User=1000:1000`, `Entrypoint=[\"deepseek-build\"]`) rather than falling back to docker/Dockerfile."
  - "`:latest` is unchanged absence: `latest_status: absent` / `latest_digest: null` both before (baseline, recorded post-push in 04-01) and after, and the index digest is not a `latest` target."
  - "REL-04 stays **Partial** and REL-05 stays **Pending/blocked**. The registry half is proven; the pull and the in-image run did not happen and are recorded, not claimed."
  - "The README says the container *contract* is headless-only (one `-p` prompt, no interactive recipe provided or tested) rather than claiming the binary has no TUI — it does, and a false statement would be worse than the nuance."

patterns-established:
  - "A registry proof records the transport that answered, so a later reader can tell an anonymous read (public) from a credentialed one."
  - "An absent `:latest` is proven against a recorded baseline value, not against an assumption."
  - "An environment block is evidence: the failing `docker info` is committed verbatim and the deferred command list is digest-pinned."

requirements-completed: []   # deliberately empty — REL-04 is Partial (pull deferred with REL-05); REL-05 remains open

coverage:
  - id: D1
    description: "Registry-side image proof: OCI index digest equal to the CI digest, both platform manifests, the exact tag set, `:latest` unchanged, config blob"
    requirement: REL-04
    verification:
      - kind: automated
        ref: "python3 scripts/distribution-proof.py registry --tag v0.1.0-proof1 --baseline … --expect-digest sha256:8f5c484e… (8/8 PASS)"
        status: pass
      - kind: automated
        ref: "docs/evidence/04-distribution-proof/image-registry/report.json (8 steps, result ok)"
        status: pass
      - kind: automated
        ref: "python3 scripts/distribution-proof.py fixtures (45/45 PASS)"
        status: pass
    human_judgment: false
  - id: D2
    description: "The pull and the in-image tool-executing turn are environment-blocked and recorded as skipped with exact commands"
    requirement: REL-05
    verification:
      - kind: automated
        ref: "docs/evidence/04-distribution-proof/image-run-blocked/report.json (result skipped, requirements [REL-05])"
        status: pass
      - kind: automated
        ref: "python3 scripts/check-readme-distribution.py --stage container --digest … --visibility public (7/7 PASS)"
        status: pass
    human_judgment: true
    rationale: "The run itself cannot be executed here (no Docker daemon). Only a host with a container runtime can close REL-05; this plan records the exact commands and the expected observable."

duration: 11min
completed: 2026-11-26
status: complete
---

# Phase 4 Plan 04: The registry serves the CI digest, and the in-image run is recorded as blocked rather than claimed

**`ghcr.io/coruairc/deepseek-build:v0.1.0-proof1` reads back as an OCI index at the CI's own digest `sha256:8f5c484e…`, with linux/amd64 `sha256:1d216b60…` and linux/arm64 `sha256:25dc0d25…`, exactly two tags and no `:latest` — read anonymously over HTTPS with no daemon; the pull and the headless in-image turn are `result: skipped` with digest-pinned commands, because this host's Docker daemon is absent.**

## Performance

- **Duration:** ~11 min
- **Started:** 2026-11-26T06:42Z
- **Completed:** 2026-11-26T06:53Z
- **Tasks:** 2/2
- **Files modified:** 18 committed files (1 script, 1 README, 16 evidence files); no product source touched
- **Estimate vs actual:** the plan estimated 115,000 tokens; the realized diff is 114,711 bytes ≈ **28,677 tokens** (chars/4) — the estimate was ~4x high

## Accomplishments

- **REL-04's registry half is proven from the host.** A registry read — not the CI log — returns the same index digest the pipeline reported, both platform manifests with their media types and sizes, exactly the two pushed tag names, and `:latest` at 404. `registry.json` records which transport answered (`anonymous-http`) and every fallback it tried.
- **The config blob was readable and matches the Dockerfile contract**: `WorkingDir=/work`, `User=1000:1000`, `Entrypoint=["deepseek-build"]`, and only `PATH`/`DEBIAN_FRONTEND` in the environment — no API key variable. That is stronger than the plan's fallback (cite `docker/Dockerfile`).
- **The index's four descriptors are told apart honestly.** Buildx wrote two `unknown/unknown` attestation manifests alongside the two platforms; they are recorded in `attestation_manifests`, and the assertion is "exactly two **platform** manifests", which is what a pull resolves.
- **`:latest` did not move — it still does not exist.** `latest_status: absent`, `latest_digest: null` after, equal to the baseline file's values, and the index digest is not a `latest` target. The baseline's own post-push limitation is stated, not hidden.
- **REL-05 is recorded as environment-blocked.** The failing `docker info` is committed verbatim (`exit=1`, `Cannot connect to the Docker daemon at unix:///var/run/docker.sock. Is the docker daemon running?`), and `commands.txt` carries the digest-pinned pull, `--version`, and the sandbox-on headless turn (`-p … --output-format json --permission-mode auto --sandbox devbox`). `report.json` is `result: skipped`.
- **The README container section now describes the published image and nothing else**: the real digests, the two tags and the absence of `:latest`, the observed public visibility, the headless-only contract, both mounts, and a plain statement that the image was **not executed** on this host.

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement `registry` and prove the digest, platforms, tags and unchanged `:latest`** — `78361acf` (feat)
2. **Task 2: Record REL-05 as deferred with exact commands, and correct the README container section** — `1516211b` (feat)

**Plan metadata:** `docs(04-04): complete …` commit carrying this SUMMARY plus STATE/ROADMAP/REQUIREMENTS.

## The evidence, as recorded

### Registry (`image-registry/registry.json`)

| Field | Value |
|-------|-------|
| Transport | `anonymous-http` (`https://ghcr.io/token?service=ghcr.io&scope=repository:coruairc/deepseek-build:pull`, then the v2 API) — no daemon |
| Visibility | `public` (the anonymous read answered 200) |
| Index digest | `sha256:8f5c484ed067143aab93aa90b1441101fd643a37924f1868eeaaa103d1f199be` — **equals `tag-run/run.json`'s `image_digest`** |
| Content type | `application/vnd.oci.image.index.v1+json` |
| `linux/amd64` | `sha256:1d216b609ed9c646cb56a5e6294bc3de5276b2823d2a90a0252b49d90b99886e`, 1810 B, `application/vnd.oci.image.manifest.v1+json` |
| `linux/arm64` | `sha256:25dc0d256765ff129b14749732f5d158b1757a540828c643410a3f142dcb6f58`, 1810 B, `application/vnd.oci.image.manifest.v1+json` |
| Attestations | `sha256:2ca7c01f…b5aa2` (838 B, amd64) and `sha256:b0ca3b1d…d19df` (838 B, arm64), platform `unknown/unknown`, `vnd.docker.reference.type=attestation-manifest` |
| Tags | `["0.1.0-proof1", "v0.1.0-proof1"]` — no `latest` |
| `:latest` after | `absent` (404), `latest_digest: null` |
| Baseline `:latest` | `absent`, `null` (from `ghcr-baseline/baseline.json`) |
| Config blob | `read`: `sha256:a3f442dd7160476285a75f6c893c85a93958fd010664112dccca898d062b17f7`, 4091 B |
| Config values | `User=1000:1000`, `WorkingDir=/work`, `Entrypoint=["deepseek-build"]`, `env_names=[PATH, DEBIAN_FRONTEND]` |
| `docker info` | `exit 1`: `Cannot connect to the Docker daemon at unix:///var/run/docker.sock. Is the docker daemon running?` |

`registry.json` also carries `transports_tried` (`manifest_status: 200`, `latest_status: 404`) and the eight assertion results. `report.json` is `result: ok`, 8 steps.

The `registry` subcommand tries, in order: anonymous HTTPS (manifest, tags, `latest`, config blob); `docker manifest inspect` (registry client — it cannot report the index digest or the tag list, so that fallback leaves `index_digest: null` and the assertion names the gap); `gh api /users/coruairc/packages/container/deepseek-build/versions` (a credentialed read — 403 without `read:packages`, which is an authorization gap, not a registry result).

### The deferred item (`image-run-blocked/`)

`report.json`:

```json
{
  "item": "image-run-blocked",
  "requirements": ["REL-05"],
  "date": "2026-11-26T06:51:38Z",
  "host": "Linux/x86_64",
  "command": "docker info (the pull and run in commands.txt were not executed: no daemon)",
  "steps": [{"name": "docker info: the daemon is absent, so nothing could be pulled or run",
             "ok": false,
             "detail": "exit 1: Cannot connect to the Docker daemon at unix:///var/run/docker.sock. Is the docker daemon running?"}],
  "artifacts": [{"path": "docs/evidence/04-distribution-proof/image-run-blocked/commands.txt",
                 "sha256": "93a2849d5198fd8eb3bf70d89dc6144409aed48025e5990b30b895ec0c5c6221"}],
  "result": "skipped"
}
```

`commands.txt` (verbatim, three commands, one per line):

```sh
docker pull ghcr.io/coruairc/deepseek-build@sha256:8f5c484ed067143aab93aa90b1441101fd643a37924f1868eeaaa103d1f199be
docker run --rm ghcr.io/coruairc/deepseek-build@sha256:8f5c484ed067143aab93aa90b1441101fd643a37924f1868eeaaa103d1f199be --version
docker run --rm --user "$(id -u):$(id -g)" -e DEEPSEEK_API_KEY -e DEEPSEEK_BUILD_HOME=/home/app/.deepseek-build -v "$PWD":/work -w /work -v "$HOME/.deepseek-build":/home/app/.deepseek-build ghcr.io/coruairc/deepseek-build@sha256:8f5c484ed067143aab93aa90b1441101fd643a37924f1868eeaaa103d1f199be -p "Run exactly one shell command: printf 'in-image-ok\n' and report its stdout." --output-format json --permission-mode auto --sandbox devbox
```

Expected observable (in `notes.md`): exit 0; a completed shell tool call whose output contains
`in-image-ok`; `estimated_cost_usd` present with `"estimate": true`; `--version` printing
`deepseek-build 0.1.0-proof1 (b14d28eb2bc7)`; and a record of whether `--sandbox devbox` applied or
the binary warned `warning: sandbox could not be applied: …` (the Dockerfile ships no `bubblewrap`;
devbox needs bwrap only for a `/data` write-deny, which the image lacks, so Landlock is the expected
path). Unblock: start the daemon (`sudo systemctl enable --now docker`); `gh auth refresh -s
read:packages` only if the package becomes private (it is public as observed). `notes.md` says in one
sentence: **"The run did not happen. REL-05 remains open."**

## README container section — before and after

**Before** (stale): a `> **Not yet built or published.**` warning blaming the rc4 macOS timeout, a
`:latest` pull against `ghcr.io/<owner>/…`, six literal `<owner>` placeholders, an interactive `-it`
recipe, no digest, no sandbox flag, no config mount, and "package visibility depends on settings".

**After** (measured): the `v0.1.0-proof1` published state as a table —

- index `sha256:8f5c484e…` (`application/vnd.oci.image.index.v1+json`);
- `linux/amd64` `sha256:1d216b60…`, `linux/arm64` `sha256:25dc0d25…`;
- tags `0.1.0-proof1`, `v0.1.0-proof1`, **no `:latest`**;
- visibility **public** (anonymous token read);
- config `WorkingDir=/work`, `User=1000:1000`, `Entrypoint=["deepseek-build"]`, no API key variable;

a digest-pinned `docker pull`; the headless-only contract (`-p`, `--output-format json`,
`--permission-mode auto`, `--sandbox devbox`); the two mounts (`$PWD:/work` with `-w /work`, and
`$HOME/.deepseek-build:/home/app/.deepseek-build`); the `--version` check; and an explicit
**"The image was not executed on the build host"** paragraph linking the deferred item. No `<owner>`
placeholder remains and the not-built warning is gone.

`python3 scripts/check-readme-distribution.py --stage container --digest <index> --visibility public`
→ **7/7 PASS**; the install stage (04-02) still **12/12 PASS**; `scripts/key-leak-check.sh --scan
docs/evidence` → **PASS**.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing critical functionality] The index has four descriptors, not two**
- **Found during:** Task 1
- **Issue:** The plan's must-have says "exactly two manifests". The real index also carries two
  buildx attestation manifests (platform `unknown/unknown`, annotated
  `vnd.docker.reference.type=attestation-manifest`), so a literal "two descriptors" assertion would
  fail on the artifact the pipeline actually produced.
- **Fix:** `registry.json` records `manifests` (the two platforms) and `attestation_manifests`
  (the two attestations). The assertion is "exactly two *platform* manifests, each with a digest and
  positive size", and a fixture proves an attestation is not counted as a platform while a third real
  platform is rejected.
- **Files modified:** `scripts/distribution-proof.py`, `docs/evidence/04-distribution-proof/image-registry/registry.json`
- **Commit:** `78361acf`

**2. [Rule 2 - Missing critical functionality] The image sets no `HOME`, so the config mount needs an explicit home**
- **Found during:** Task 2
- **Issue:** The plan's run command mounts `$HOME/.deepseek-build` at `/home/app/.deepseek-build` and
  passes `--user "$(id -u):$(id -g)"`. The image config's environment has no `HOME` (only `PATH`,
  `DEBIAN_FRONTEND`), and with a non-1000 uid the passwd lookup may not yield `/home/app`, so the
  binary would resolve a different config dir and silently not use the mount.
- **Fix:** Added `-e DEEPSEEK_BUILD_HOME=/home/app/.deepseek-build` (the documented home override) to
  the deferred command and the README, and documented that the mounted host directory must be
  writable by the uid passed to `--user`.
- **Files modified:** `docs/evidence/04-distribution-proof/image-run-blocked/commands.txt`, `README.md`
- **Commit:** `1516211b`

**3. [Rule 1 - Bug] "Headless-only" cannot mean "ships no TUI"**
- **Found during:** Task 2
- **Issue:** The obvious wording ("it ships the agent binary, not a terminal UI") is false: the image
  runs the same binary as the release tarballs, which contains the TUI. A false statement to satisfy
  a checker is worse than the truth.
- **Fix:** The README states the container **contract** is headless-only — one `-p` prompt, no
  interactive container recipe provided or tested — and explicitly notes the binary also carries the
  TUI. The checker's `headless` requirement and the phase criterion are still satisfied.
- **Files modified:** `README.md`
- **Commit:** `1516211b`

### Plan-level deviations (recorded by name)

**4. ROADMAP Phase 4 criterion 1 — "pulled from the registry" — is NOT satisfied.**
Criterion 1 reads: *"A throwaway pre-release tag produces the GHCR image, and the image is pulled from
the registry — not just built locally — as the artifact a user would get."* The image **was not
pulled**: there is no Docker daemon on this host (`docker-info.log`, `exit 1`) and it cannot be
started without a password. The registry-side half (built, published, digest/platform/tag identity) is
proven; the pull is deferred with REL-05 and the digest-pinned command is recorded in
`image-run-blocked/commands.txt`. Criterion 1 therefore stays **open**, as does criterion 2's run
half. This is the honest ceiling of this host, not an oversight.

**5. The baseline is post-push, not pre-push** (inherited from 04-01, restated here because this plan
consumes it). `ghcr-baseline/baseline.json` was measured after the tag push; the `:latest` invariant
rests on the post-push 404 plus the workflow's structural rule and the container log's own
`Pre-release — :latest unchanged` line. `registry.json` compares against that recorded value, not an
invented "before".

**6. The plan says "No commit made"; the dispatch requires commits.** The plan's success criteria
end with "No commit made; the working tree is left for the reviewer", but the executing dispatch
instructed one atomic commit per task plus a metadata commit. Followed the dispatch: `78361acf`,
`1516211b`, and the metadata commit. (Same conflict and resolution as 04-01's deviation 7.)

## Issues Encountered

- `docker manifest inspect` (used during reconnaissance and left as the documented fallback) works
  without a daemon. `docker info` fails with the absent-daemon line above; `systemctl is-active
  docker` → `failed`. No `pull`/`run`/`build`/`login` was executed and no daemon start was attempted.
- `gh api /users/coruairc/packages/container/deepseek-build/versions` remains 403 without
  `read:packages`; the anonymous path answered, so the package is public and no login is needed.
- The evidence item was recorded twice: the first `finish` was followed by one more `registry` run
  (for the plan's verify command), which changed `checked_at` and therefore the artifact hash. It was
  re-recorded end-to-end before commit so `report.json`'s digests are over the committed bytes — the
  same ordering trap 04-01 hit.

## Requirements

- **REL-04 — still Partial.** The registry half is now proven from the host (index digest equal to the
  CI digest, both platforms, the tag set, `:latest` unchanged). The requirement's words are "built,
  published, **and pulled**"; the pull did not happen and is deferred with REL-05. Not marked complete.
- **REL-05 — Pending / environment-blocked.** `image-run-blocked/report.json` is `result: skipped`
  with `requirements: ["REL-05"]`; the commands are digest-pinned and the notes state the run did not
  happen. This plan adds no claim of execution.
- **REL-06, PKG-04** — closed earlier (04-02, 04-03); untouched here.

## Known Stubs

None. The `registry` stub from 04-01 is now implemented and exercised; no placeholder data flows into
the README or the evidence.

## Threat Flags

None. No new outbound destination: the harness talks to `ghcr.io` (already used by `baseline`) and
runs `gh api`, and no secret was read or written. No token, bearer header or `~/.docker/config.json`
content appears in any committed file (the scripts record only the transport name and HTTP status;
`key-leak-check --scan docs/evidence` is green). No package manager install ran.

## Self-Check: PASSED

- All 7 key files created by this plan exist on disk, plus the two `report.json` files and `notes.md`
  under `image-registry/`; `README.md` and `scripts/distribution-proof.py` are the only tracked files
  modified outside `docs/evidence/04-distribution-proof/`.
- Both task commits are ancestors of HEAD: `78361acf`, `1516211b` (measured `06ec92f6..HEAD` = 2).
- Every artifact path recorded in the two `report.json` files resolves on disk and its sha256 matches
  (`registry.json` `9da69bdd…`, `docker-info.log` `7485c1be…`, `commands.txt` `93a2849d…`).
- Re-ran the plan's verification: fixtures 45/45; `registry` 8/8 assertions with
  `transport=anonymous-http`, index `sha256:8f5c484e…`, platforms `linux/amd64` + `linux/arm64`,
  no `latest`; `grep -Eic` over `docker-info.log` = 1; container-stage checker 7/7; install-stage
  checker 12/12; `image-run-blocked/report.json` `result: skipped` with `[REL-05]`;
  `commands.txt` carries `--sandbox devbox` and four `sha256:` mentions; `notes.md` contains
  "did not happen".
- `scripts/key-leak-check.sh --scan docs/evidence` exits 0.
