# egress

Proves PROOF-02: the runtime egress boundary on a real session — only the
configured provider, the `web_fetch` host and AF_UNIX sockets, zero violations.

## Status: DEFERRED (tests skipped by owner, 2026-11-22)

No run was performed. This item is `result: skipped`; no cast or destination
list exists until testing resumes with a throwaway key.

## What the measurement must show

- `api.deepseek.com:443` only for model traffic.
- The one host the session's `web_fetch` tool was asked to fetch (default
  `example.com`), over 443.
- AF_UNIX sockets (resolver, dbus, MCP stdio) and loopback.
- `EGRESS_VIOLATIONS=0`; nothing else public.

`strace` is absent on the author's host, so the measurement uses the
`LD_PRELOAD` connect/sendto/sendmsg logger in `scripts/egress-check.sh`. Its
coverage limits are documented at the top of that script (no raw syscalls,
`io_uring`, static children; IP-based allowlisting). Use
`strace -f -e trace=connect,sendto,sendmsg` where available as ground truth.

## Re-run

```sh
ITEM=docs/evidence/02-loop-item-verification/egress
scripts/evidence.py begin --item-dir "$ITEM" --item egress \
    --requirements PROOF-02 \
    --command "scripts/egress-check.sh (real session, LD_PRELOAD logger)"

script -qec 'scripts/egress-check.sh' -T "$ITEM/run.timing" "$ITEM/run.typescript"

# One step per property; each greps the captured transcript.
scripts/evidence.py step --item-dir "$ITEM" --name "zero egress violations" \
    -- sh -c "grep -qF 'EGRESS_VIOLATIONS=0' '$ITEM/run.typescript'"
scripts/evidence.py step --item-dir "$ITEM" --name "only the provider and the web_fetch host" \
    -- sh -c "grep -qE 'api\.deepseek\.com:443' '$ITEM/run.typescript'"

scripts/evidence.py cast --typescript "$ITEM/run.typescript" \
    --timing "$ITEM/run.timing" --out "$ITEM/egress.cast"
scripts/evidence.py artifact --item-dir "$ITEM" --path "$ITEM/egress.cast"
scripts/evidence.py finish --item-dir "$ITEM" --result ok
```

If the run reports any destination outside the allowlist, stop: that is an
egress invariant violation, not a measurement to record and move past.
