# image-run

## What this proves

The published GHCR image — pulled by **index digest**, not by tag — is a working
`deepseek-build` binary that completes a real headless, tool-executing turn:

- `docker run … --version` → `deepseek-build 0.1.0-proof1 (b14d28eb2bc7)`
- a headless prompt asking for one shell command ran `printf 'in-image-ok\n'`,
  returned `in-image-ok`, stopped with `end_turn`, and reported
  `estimated_cost_usd: 0.000347148` with `estimate: true`.
- the model used was `deepseek-flash`; the turn took two model calls.

This is the run half of REL-05, executed on 2026-11-26 on the author's build host
with a working key. The key was passed to `docker run` as `-e DEEPSEEK_API_KEY`
and never appears in any recorded command or file.

## What this does NOT prove — the sandbox is not applied inside the image

`bwrap` is **not installed in the image** (`command -v bwrap` finds nothing), so
the in-process sandbox cannot be applied there. `--sandbox devbox` is accepted by
the CLI but there is no bubblewrap to enforce it; the container boundary is the
only isolation in this artifact.

Consequences, stated plainly:

- The roadmap's original wording for this phase ("run flags that keep the sandbox
  on") is **not achievable in this image** and is recorded as a deviation rather
  than papered over.
- The README container block has been corrected to say the sandbox is unavailable
  inside the image and why.
- `KNOWN-ISSUES.md` records the finding.

Adding `bubblewrap` to the image would not by itself fix it: bwrap needs user
namespaces, which a default unprivileged container blocks, so enabling it would
require weakening the container's own isolation (`--cap-add SYS_ADMIN` or
`--privileged`). That trade-off is not made here.

## Steps

| Step | Assertion |
|---|---|
| `image: pulled by index digest` | the image is retrievable by digest and reports that digest |
| `image: binary reports the tag version` | the entrypoint prints `0.1.0-proof1` with the tag's commit |
| `image: headless turn ran a shell command` | the captured JSON contains `in-image-ok` from the tool call |
| `image: cost estimate present` | the JSON carries `estimate: true` |
| `image: bwrap absent …` | the image has no bubblewrap, so the sandbox cannot apply |

## Re-run

```sh
IMG=ghcr.io/coruairc/deepseek-build@sha256:8f5c484ed067143aab93aa90b1441101fd643a37924f1868eeaaa103d1f199be
docker pull "$IMG"
docker run --rm "$IMG" --version
docker run --rm --entrypoint sh "$IMG" -c 'command -v bwrap || echo no-bwrap'
DEEPSEEK_API_KEY=... docker run --rm --user "$(id -u):$(id -g)" \
  -e DEEPSEEK_API_KEY -e DEEPSEEK_BUILD_HOME=/home/app/.deepseek-build \
  -v "$PWD":/work -w /work -v "$HOME/.deepseek-build":/home/app/.deepseek-build \
  "$IMG" -p "Run exactly one shell command: printf 'in-image-ok\n' and report its stdout." \
  --output-format json --permission-mode auto
```

A new run is a new item; this one is not rewritten.
