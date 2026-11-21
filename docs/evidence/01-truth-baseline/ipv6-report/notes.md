# ipv6-report

Proves SBOX-02's second half: `scripts/sandbox-run.sh`'s `LD_PRELOAD` guard
resolves the allow host over both families and names an IPv6 destination
instead of reporting it as `non-ip`.

## What the cast shows

`ipv6-report.cast` is a `script -T` capture of `sh scripts/test-sandbox-run.sh`.
The harness compiles `scripts/egress_guard.c` into a temp directory, drives
small `python3` connectors through it, and prints six `PASS:` lines followed by
`ALL PASS (6 assertions)`. No network beyond loopback: every address is a
numeric literal (`2001:db8::1`, `203.0.113.9`) or `localhost` from `/etc/hosts`.

## Steps

Each step in `report.json` runs `grep -F` over the captured transcript for the
corresponding `PASS:` line, so a regression that drops, renames or flips an
assertion fails the step and the item.

| Step | Assertion |
|---|---|
| `ipv6-refused: blocked line names 2001:db8::1` | the refusal prints the destination address |
| `ipv6-refused: no 'non-ip' placeholder` | the blocked line never falls back to `non-ip` |
| `ipv6-loopback: ::1 and ::ffff:127.0.0.1 permitted` | IPv6 loopback and its v4-mapped spelling are allowed |
| `ipv6-allowlist: resolves IPv6, permits it, and matches the v4-mapped spelling` | `EGRESS_ALLOW_DEBUG=1` lists a `family=inet6` allowlist entry and it is permitted |
| `ipv4-refused: blocked line names 203.0.113.9` | the IPv4 path still names the destination |
| `script: sandbox-run.sh runs end to end (BIN=/bin/echo)` | the wrapper itself still executes |

## Re-run

```sh
sh scripts/test-sandbox-run.sh        # expect: ALL PASS (6 assertions), exit 0
```

To regenerate the cast:

```sh
ITEM=docs/evidence/01-truth-baseline/ipv6-report
script -qec 'sh scripts/test-sandbox-run.sh' -T "$ITEM/run.timing" "$ITEM/run.typescript"
scripts/evidence.py cast --typescript "$ITEM/run.typescript" --timing "$ITEM/run.timing" \
    --out "$ITEM/ipv6-report.cast"
```

The raw typescript/timing intermediates are not committed; the cast and the
step logs are, and both are normalized (`<REPO>/<HOME>/<TMP>`).
