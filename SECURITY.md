# Security Policy

Please report security vulnerabilities via our HackerOne program:

https://hackerone.com/x

Do not open public GitHub issues for security reports.

> Note (fork): the HackerOne program above is inherited from upstream
> [xai-org/grok-build](https://github.com/xai-org/grok-build). This personal
> `deepseek-build` fork has no separate disclosure program; the contact above
> applies to the upstream codebase.

## Network & privacy

`deepseek-build` is a local terminal agent. It has no telemetry, no analytics,
and no auto-update. The **only** network destinations are:

1. the configured model provider — default `https://api.deepseek.com`
   (`/chat/completions` and `/models`), and
2. any MCP servers **you** configure (`[mcp_servers.<name>]` in
   `~/.deepseek-build/config.toml`), plus loopback / local `AF_UNIX` sockets.

Your prompt, files, and tool output are sent to the configured provider so it
can answer; nothing is sent anywhere else. The API key is read from
`DEEPSEEK_API_KEY` (or `DEEPSEEK_BUILD_API_KEY`) and is never logged.

### Verifying the egress boundary

Two scripts in `scripts/` let you confirm this:

```sh
scripts/check-egress.sh          # static guard: forbidden hosts must be absent; prints a SOFT branding report
scripts/check-egress.sh --strict # also fails on remaining branding/ACP-namespace strings
```

`check-egress.sh` scans `crates`, `prod`, and `third_party` for legacy xAI,
telemetry, and upload destinations (`api.mixpanel.com`, `sentry.io`, `auth.x.ai`,
`gs://`, `s3://`, …). It must print `HARD egress gate: OK` and exit 0.

```sh
scripts/sandbox-run.sh [deepseek-build args...]   # e.g. --version, -p "hi"
```

`sandbox-run.sh` runs the binary under an `LD_PRELOAD` shim that permits
`connect()` only to loopback, `AF_UNIX`, and the resolved addresses of
`$DEEPSEEK_BUILD_EGRESS_HOST` (default `api.deepseek.com`). Any other attempt is
refused with `ECONNREFUSED` and printed as
`sandbox-run: BLOCKED connect to <ip>`. Set `BIN=` to point at a different
binary.

Where available, `strace -f -e trace=connect ./target/release/deepseek-build -p "hi"`
gives the same picture at the syscall level. Observed destinations for
`--version`/`--help` are none; a single prompt contacts only `api.deepseek.com:443`
plus one local `AF_UNIX` socket. See [`TESTING.md`](TESTING.md) for the procedure.
