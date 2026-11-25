# runtime-egress

Proves PROOF-02: the one real session captured for PROOF-01 contacted only the
configured provider, the `web_fetch` host, loopback and AF_UNIX sockets. This
item re-reads that session's committed recording; it did not start a second
live run.

## What this measures

The observed `connect()`/`sendto()` destinations of the real model-driven
session recorded in `../live-smoke/live-smoke-transcript.txt`, classified by
`scripts/egress-check.sh`'s own allowlist:

- the configured provider (`api.deepseek.com`, resolved at run time),
- the `web_fetch` host that script names (`example.com`),
- loopback and the unspecified address,
- AF_UNIX sockets (the resolver, dbus, local MCP stdio pipes),
- and nothing else.

The claim is about destinations only: which addresses the session opened
sockets to. It is not a claim about the bytes sent over them.

## Where the numbers come from

Three facts a reader needs in order to judge the item:

1. The measured session is the PROOF-01 session. This is the same session
   recorded under `../live-smoke/`, read back from its committed transcript —
   not a fresh run. No second live session was started and no key was exported.
2. The verdict lines were printed by `scripts/egress-check.sh` during that
   session and are quoted from the transcript unchanged: `EGRESS_VIOLATIONS=0`,
   `EGRESS-CHECK PASS`, and `egress allowlist: all observed destinations are
   allowed`.
3. The two connect counters are asserted greater than zero — `API_CONNECTS=7`
   and `WEB_FETCH_CONNECTS=1` — so an idle run that contacted nothing cannot
   pass. `API_CONNECTS` proves the provider was really reached;
   `WEB_FETCH_CONNECTS` proves the `web_fetch` host was really reached.

`destination-check.py` re-derives all of it from the transcript: it parses the
observed-destinations table, requires every row `PASS` with a reason in
`{allowlisted, loopback, unix, unspecified}`, requires `EGRESS_VIOLATIONS=0`,
and requires both connect counters to be greater than zero. `destinations.txt`
is that checker's output, not a hand-written table.

## Steps

| Step | Assertion |
|---|---|
| `egress: zero violations` | the transcript contains `EGRESS_VIOLATIONS=0` |
| `egress: allowlist verdict passed` | the transcript contains `egress allowlist: all observed destinations are allowed` |
| `egress: destination table parsed and permitted` | `destination-check.py` finds the table, every row is `PASS` with a permitted reason, `EGRESS_VIOLATIONS=0`, and both connect counters are greater than zero |
| `egress: provider and web_fetch both contacted` | the transcript contains the script's own `EGRESS-CHECK PASS` verdict |

## What this measurement cannot see

The instrument is an `LD_PRELOAD` shim that interposes the libc calls and
allows and logs every `connect()`, `sendto()`, `sendmsg()` and `sendmmsg()`. In
the script's own terms it therefore cannot see:

- a statically linked child, a setuid child, or a child that scrubs
  `LD_PRELOAD` — the shim is never loaded there;
- a program issuing raw syscalls (`syscall(SYS_connect|sendto|sendmsg)`),
  `io_uring`, or anything else that bypasses the libc entry points;
- payloads: it observes destinations, not content, and allowlisting is by
  resolved IP, so another hostname on the same shared CDN edge IP would pass;
- a destination after the fact as "reached": a connection that failed before
  `connect()` returned is still logged.

A zero-violation verdict is therefore evidence that the boundary held for the
calls the shim could intercept. It is not proof of content-level isolation, and
it does not cover a process that never went through libc.

## Re-run

Re-check this item against the committed transcript (no key, no network):

```sh
python3 docs/evidence/02-loop-item-verification/runtime-egress/destination-check.py docs/evidence/02-loop-item-verification/live-smoke/live-smoke-transcript.txt /dev/null
```

For a fresh live measurement on the build host, export the key in the parent
shell and run the measuring script itself:

```sh
export DEEPSEEK_API_KEY=...
scripts/egress-check.sh
```

A new run is a new item: a new directory with its own `report.json`, never a
rewrite of this one. This item is a reading of the PROOF-01 recording, so a
live re-run will produce its own counters rather than correcting these.
