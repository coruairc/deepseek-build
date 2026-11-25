# live-smoke

Proves PROOF-01 from two real executions, not from an assertion. The key was
exported in the parent shell before `script -qec`. It was not typed inside
either captured command, and neither capture was run under `set -x`.

## What the cast shows

Two scripts ran in two separate sessions, both against the plan's pinned binary
copy (`BIN=<TMP>/opencode/02-01/deepseek-build`, version `deepseek-build
0.0.1-rc5 (6e9d8d88dca1)`), not the shared cargo output path.

`live-smoke.cast` / `live-smoke-transcript.txt` is one `script -T` capture of
`scripts/egress-check.sh`. That session is the one that exercises the three
PROOF-01 capabilities: a shell command, a file edit, and a real `web_fetch`.
The same transcript is the file plan 02-02 reads back for PROOF-02. It also
shows the resume of that session, the local stdio MCP call, auto-compaction,
`EGRESS_VIOLATIONS=0`, and the stricter `sandbox-run.sh` comparison (the
`web_fetch` host is refused there, as that guard's allowlist is provider plus
loopback only). It proves nothing about the adapter smoke.

`adapter-smoke.cast` / `adapter-smoke-transcript.txt` is one `script -T`
capture of `scripts/smoke-test.sh`. It is the adapter-level check. A green item
means every adapter check passed (`8 passed, 0 failed`). It proves nothing
about egress.

No key material is in either capture. `scripts/key-leak-check.sh --scan` over
this directory exited 0 before `finish`. Raw `run-*.typescript` and
`run-*.timing` intermediates were deleted rather than committed.

Normalization overlap, not a second work directory: `mktemp` created a
directory whose path contained the substring `/tmp` twice (`/tmp/tmp.…`).
`scripts/evidence.py` replaces every `/tmp` prefix, so those work-dir lines
render as `<TMP><TMP>.…`. The binary pin renders once, as
`<TMP>/opencode/02-01/deepseek-build`. The scripts were not edited.

## Steps

Each step in `report.json` is a real `grep -F` recorded by `scripts/evidence.py
step`. The loop steps read `live-smoke-transcript.txt`. The smoke step reads
`adapter-smoke-transcript.txt`.

| Step | Assertion |
|---|---|
| `smoke: adapter checks all passed` | the adapter script's own summary line `8 passed, 0 failed` |
| `loop: shell tool ran` | `shell tool ran (shell_out.txt contains shell-egress)` |
| `loop: edit tool ran` | `edit tool ran (edit_me.txt now:` |
| `loop: web_fetch made a real connection` | `web_fetch made a real connection to` |
| `loop: resumed the same session` | `resume reused session id` |

## Re-run

Export the key in the parent shell first. Do not put it in the captured
command. Point `BIN` at a private copy of the release binary.

```sh
export BIN=/tmp/opencode/02-01/deepseek-build
unset DEEPSEEK_BUILD_API_KEY DEEPSEEK_BASE_URL
script -qec 'scripts/egress-check.sh' -T run-egress.timing run-egress.typescript
script -qec 'scripts/smoke-test.sh' -T run-smoke.timing run-smoke.typescript
```

Expect `EGRESS-CHECK PASS` from the first and `8 passed, 0 failed` from the
second. Convert with `scripts/evidence.py cast`, normalize the copied
typescript, and delete the raw timing/typescript files before commit.

## What this item does not show

- It does not measure runtime egress as a separate PROOF-02 item. 02-02 reads
  `live-smoke-transcript.txt` from this same egress-check session; do not start
  a second live run for that measurement.
- It does not revoke the key. That human step is PROOF-03 (plan 02-03).
- The adapter smoke does not exercise shell, edit, or `web_fetch`.
- The sandbox-run comparison is not the PROOF-01 session. In that comparison
  the `web_fetch` host is blocked on purpose.
