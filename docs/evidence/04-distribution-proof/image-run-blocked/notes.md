# image-run-blocked — REL-05 is environment-blocked on this host

**The run did not happen. REL-05 remains open.** This item records the exact
commands, their digest pin, and the observable a future run must capture. It is
`result: skipped` because an environment block is the honest result — `ok` would
claim an execution that never occurred.

## Why it is blocked

The Docker CLI is installed (`Docker version 29.10.0`) but no daemon is running on
this host, and starting one needs a password this session does not have:

```
$ docker info
Client:
 Version:    29.10.0
 ...
Server:
Cannot connect to the Docker daemon at unix:///var/run/docker.sock. Is the docker daemon running?
$ echo $?
1
```

`systemctl is-active docker` → `failed`. The verbatim capture is
[`../image-registry/docker-info.log`](../image-registry/docker-info.log). No
`docker pull`, `docker run`, `docker build` or `docker login` was attempted; the
registry facts in `../image-registry/` come from anonymous HTTPS and
`docker manifest inspect` (a registry client, no daemon).

## What to run

[`commands.txt`](commands.txt) holds the three commands, one per line,
digest-pinned to the published index:

```
sha256:8f5c484ed067143aab93aa90b1441101fd643a37924f1868eeaaa103d1f199be
```

1. `docker pull ghcr.io/coruairc/deepseek-build@sha256:8f5c484e…`
2. `docker run --rm …@sha256:8f5c484e… --version`
3. the headless turn: `-p "Run exactly one shell command: printf 'in-image-ok\n'
   and report its stdout." --output-format json --permission-mode auto
   --sandbox devbox`, with `-e DEEPSEEK_API_KEY`, `-v "$PWD":/work -w /work` and
   `-v "$HOME/.deepseek-build":/home/app/.deepseek-build`.

`DEEPSEEK_BUILD_HOME=/home/app/.deepseek-build` is set explicitly so the config
mount is used regardless of how the runtime resolves `HOME` for `--user`. The
mounted host directory must be writable by the uid passed to `--user` (this
host's uid 1000 matches the image's user).

## What the run must show

- **Pull:** succeeds, and `docker image inspect --format '{{index .RepoDigests 0}}'`
  reports `sha256:8f5c484e…` — the same index digest the registry serves.
- **`--version`:** `deepseek-build 0.1.0-proof1 (b14d28eb2bc7)` — the tag version
  and the release commit.
- **Headless turn:** exit 0; the JSON contains a completed shell tool call whose
  output contains `in-image-ok`; `estimated_cost_usd` is present (with
  `"estimate": true`); `usage` is non-zero. A run that exits 0 without the
  sentinel in a tool result proves nothing.
- **Sandbox:** record whether `--sandbox devbox` **applied** or the binary warned
  `warning: sandbox could not be applied: …`. `docker/Dockerfile` installs no
  `bubblewrap`; the devbox profile needs bwrap only for a `/data` write-deny
  (`/data` is absent in the image), so the expected path is kernel enforcement
  (Landlock) — which Docker's default seccomp profile may block on older
  releases. Do not call the run "sandboxed" if that warning appears.
- **Registry facts to re-check at run time:** package visibility (`public` as
  observed on 2026-11-26, so no login is needed; if a later change makes it
  private, run `echo "$CR_PAT" | docker login ghcr.io -u coruairc
  --password-stdin` with a token carrying `read:packages` and record that).

## Unblock action

1. Start the daemon on a host that has one:
   `sudo systemctl enable --now docker` (then `docker info` must exit 0). If the
   user should not need sudo per run, add it to the `docker` group and re-login.
2. Run [`commands.txt`](commands.txt) unchanged.
3. Record the outputs as a new evidence directory (this item is `skipped`; a
   correction is a new item, never an edit of this one) and only then mark REL-05
   complete.
4. `gh auth refresh -s read:packages` is needed **only** if the package is
   private. It is public as observed, so it is not part of the unblock.

## Files

| File | Meaning |
|------|---------|
| `commands.txt` | the three digest-pinned commands, one per line |
| `report.json` | `result: skipped`, `requirements: [REL-05]`, one `docker info` step |
| `docker-info.log` | the step's captured output (the absent-daemon failure) |
| `notes.md` | this file |

## Related

- Registry-side proof of the same digest: [`../image-registry/`](../image-registry/)
- CI build/publish of the image: [`../tag-run/`](../tag-run/)
- README container section: `README.md` → `## Container`
