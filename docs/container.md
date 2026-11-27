<!-- generated-by: gsd-doc-writer -->
# Container image

A public, digest-pinned `deepseek-build` image in GitHub Container Registry (GHCR): the
release binary on `debian:bookworm-slim` for hosts with Docker but no Rust toolchain.

## What the image is

- **Image:** `ghcr.io/coruairc/deepseek-build`, pushed on tag pushes by the
  [`container` job](../.github/workflows/release.yml); public, anonymous pulls work.
- **Platforms:** `linux/amd64` + `linux/arm64` from one OCI index, digest
  `sha256:8f5c484ed067143aab93aa90b1441101fd643a37924f1868eeaaa103d1f199be`
  (`v0.1.0-proof1`; tags `v0.1.0-proof1`, `0.1.0-proof1`, no `:latest`).
- **Build:** the job unpacks the Linux release tarballs, stages each binary next to
  [`docker/Dockerfile`](../docker/Dockerfile), and installs it at `/usr/local/bin/deepseek-build` (+ `dsb`).
- **Runtime contract:** `ENTRYPOINT ["deepseek-build"]`, `USER 1000:1000`,
  `WORKDIR /work`. No API key or key-named environment variable is baked in.
- **Contents:** `ca-certificates`, `git`, `ripgrep`, `curl`, `openssh-client`,
  `bash`, `procps`, `libgcc-s1`; no compiler toolchain and no `bubblewrap`.

## Pull and verify

```bash
IMG=ghcr.io/coruairc/deepseek-build@sha256:8f5c484ed067143aab93aa90b1441101fd643a37924f1868eeaaa103d1f199be
docker pull "$IMG"
docker run --rm "$IMG" --version
```

The last command prints `deepseek-build 0.1.0-proof1 (b14d28eb2bc7)`. The image
ships no `bwrap`; see [The sandbox is not available](#the-sandbox-is-not-available-inside-the-image).

## Run a headless turn

Use the image headlessly: one `-p` prompt per invocation. This is the command
recorded by the in-image evidence run:

```bash
export DEEPSEEK_API_KEY=...   # your key; passed by name, never baked in
docker run --rm --user "$(id -u):$(id -g)" \
  -e DEEPSEEK_API_KEY \
  -e DEEPSEEK_BUILD_HOME=/home/app/.deepseek-build \
  -v "$PWD":/work -w /work \
  -v "$HOME/.deepseek-build":/home/app/.deepseek-build \
  ghcr.io/coruairc/deepseek-build@sha256:8f5c484ed067143aab93aa90b1441101fd643a37924f1868eeaaa103d1f199be \
  -p "Run exactly one shell command: printf 'in-image-ok\n' and report its stdout." \
  --output-format json --permission-mode auto
```

The flags that matter:

- `--user "$(id -u):$(id -g)"` — run as your host uid/gid, so files written to
  `/work` belong to you.
- `-e DEEPSEEK_API_KEY` — copies the key from your environment by name; it is not
  stored in the image or recorded commands.
- `-e DEEPSEEK_BUILD_HOME=...` — pins the state/config directory, since `--user`
  overrides the image's `USER 1000:1000`.
- `-v "$PWD":/work -w /work` — mounts the repository the agent should edit.
- `-v "$HOME/.deepseek-build":...` — mounts host config/state so settings and
  sessions persist.
- `-p "..."` — one single-turn prompt; the response goes to stdout and the
  process exits.
- `--output-format json` — one final JSON object: `text`, `stopReason`, `usage`,
  `num_turns`, `modelUsage`, `estimated_cost_usd`, `estimate`.
- `--permission-mode auto` — a background classifier reviews tool calls, so the
  shell command runs without an interactive prompt.

## The sandbox is not available inside the image

The image ships no `bubblewrap` (`bwrap` is absent), so the in-process sandbox
cannot be applied. `--sandbox` is accepted by the CLI, but nothing in the image can
enforce it — **do not pass `--sandbox` or recommend `--sandbox devbox` for this
image.** The container boundary is the isolation: non-root user, only the intended
workspace mounted, no `SYS_ADMIN` or `--privileged`. Verify with
`docker run --rm --entrypoint sh <image> -c 'command -v bwrap || echo no-bwrap'` → `no-bwrap`.

Adding `bubblewrap` would not fix this by itself: bwrap needs user namespaces,
which a default unprivileged container blocks, so enabling it would weaken the
container's own isolation — a trade-off deliberately not made.

## Cleanup

```bash
docker rmi ghcr.io/coruairc/deepseek-build@sha256:8f5c484ed067143aab93aa90b1441101fd643a37924f1868eeaaa103d1f199be
```

A digest pull creates no tag: `docker images` shows `<none>` in the TAG column and `docker images --digests` shows the digest; removing by digest is reliable.

## Not verified here

The in-image run was executed on the author's host on 2026-11-26; see its
[`notes`](evidence/04-distribution-proof/image-run/notes.md) and
[`report.json`](evidence/04-distribution-proof/image-run/report.json). Observed on `linux/amd64`:

- the digest pull and the `--version` string above;
- a headless turn running `printf 'in-image-ok\n'`, returning `in-image-ok` with
  `stopReason: end_turn` and `estimated_cost_usd: 0.000347148` (`estimate: true`,
  `deepseek-flash`, two model calls), plus `bwrap` absent inside the image.

Not exercised: interactive TUI use in the container (evidence covers headless
`-p` turns only), a `linux/arm64` run, and any host other than the author's.
