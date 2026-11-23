# test-gate

Proves TEST-01 (the workspace step exits 0 and is completeness-checked by
target count), TEST-02 (the `config-docs` feature step runs as a named step)
and EVID-01's recorded-run half: one gate that cannot be green by accident.

## The recorded full runs

Three full runs on this tree, all green, all with the three API-key variables
unset for every cargo step:

| Run | Date (UTC) | Mode | results | binaries | doctests | expected | Exit |
|---|---|---|---|---|---|---|---|
| #2 (seed) | 2026-11-21 | `--refresh-expected` | 308 | 219 | 89 | written: 308 | 0 |
| #3 (confirmation) | 2026-11-23 | plain | 308 | 219 | 89 | 308 | 0 |
| #4 (post-review re-seed) | 2026-11-23 | `--refresh-expected` | 308 | 219 | 89 | written: 308 | 0 |

Run #3's verdict, as printed by the gate (never inferred from a log tail):

```
gate step 1: PASS (rc=0)   cargo test --workspace --no-fail-fast -- --test-threads=4
gate step 2: PASS (rc=0)   cargo test -p xai-grok-shell --lib --features config-docs -j 1
gate counts: results=308 (raw test-result lines: 309) binaries=219 doctests=89
check results == binaries + doctests: PASS (308 == 308)
check results == expected (308): PASS
gate step 3: PASS (rc=0)   scripts/key-leak-check.sh --scan docs/evidence
GATE PASS
```

Step 1 totals: 31,913 passed / 0 failed / 386 ignored. Step 2 (`config-docs`):
6,201 passed / 0 failed / 4 ignored — artifact-backed by the supplementary
excerpt `step2-config-docs-result-line.log` (the raw step-2 cargo log lived in
`<TMP>/opencode/gate-run3/` and was not committed in full; the excerpt carries
its `test result:` line verbatim, added post-review for IN-04). Raw
`test result:` lines are 309 because
`xai_tty_utils`'s `stderr_redirect_roundtrip_subprocess` re-runs its own test
binary with inherited stdout; `results` counts one top-level result per cargo
target block, so the nested line is excluded.

**Expected-count provenance.** `scripts/test-gate.expected` is now seeded by
run #4's post-review `--refresh-expected` (header: commit `c159a3e7`,
`# tree: dirty (uncommitted changes present at refresh time)`,
2026-11-23T16:08:34Z, the same three numbers 308/219/89). It was previously
seeded by run #2 (commit `882ad0de`, 2026-11-21T21:37:20Z) — that header is
history now. Run #3 confirmed the same numbers without touching the file.
The expected file is not one of the item's hashed artifacts (the hashed
artifacts are `test-gate.cast` and `test-gate.log`), so re-seeding it does not
alter the frozen `report.json`. The historical "307 targets" figure
(2026-10-09, a different tree) was never seeded. Only a fully green
`--refresh-expected` may move the number.

Run #4 printed the same verdict shape as run #3 (both cargo steps PASS, four
counts 308/219/89, `results == binaries + doctests` PASS, leak scan PASS,
`GATE PASS`, exit 0), and its own step logs were re-derived to the same totals
— step 1 31,913 passed / 0 failed / 386 ignored, step 2 6,201 / 0 / 4 — at
`<TMP>/opencode/gate-refresh-postfix/`.

**Provenance caveat (WR-02, resolved in iteration 2).** Run #2's stamped commit
`882ad0de` (committed 2026-11-21T19:38:36Z) *predates* the WIP commit
`b0516ef1` (22:34:33Z) that carries `test-gate.sh` itself and the determinism
fixes: the seed run ran on a dirty tree, and checking out `882ad0de` alone
cannot reproduce it. The stronger reproducibility evidence remains run #3,
which re-confirmed 308/219/89 on the merged tree. The iteration-1 attempt to
stamp the tree state landed the check **inside** the redirect group that writes
the expected file, so the file always looked modified and `# tree: clean` was
unreachable (re-review WR-02 regression); it is fixed by computing the state
before the redirect opens, demonstrated in a throwaway repo for both branches
(`# tree: clean` on a committed clean tree, `# tree: dirty …` after adding an
untracked file). Run #4's real header carries the honest `dirty` marker: HEAD
`c159a3e7` plus the uncommitted review fixes were what the count was measured
on. Only the refresh writer rewrites this file.

**Interruption note.** Run #3 was started by the previous executor on
2026-11-23, survived a server restart as an orphaned but intact
`evidence.py step` process tree (the pty daemon kept its stdout pipes open),
and was adopted by the continuation executor rather than restarted. Its step
record carries the real exit status; the counts above were re-derived
independently from the run's log with `awk`.

## What the cast shows

`test-gate.cast` is a `script -T` capture of a **subset-mode** run — the
mechanism, not the phase gate:

```sh
scripts/test-gate.sh --package xai-grok-status-line --package xai-grok-sandbox
```

It prints `GATE MODE: SUBSET — not the phase gate`, runs both cargo steps
under the team build lock, proves `results == binaries + doctests` on real
cargo output (9 == 7 + 2), reports the expected-count check as SKIPPED, runs
the leak scan, and exits 0. The full-run evidence is `test-gate.log` (run #3's
normalized step-1 log) and the step records in `report.json`, not the cast.
The cast's final line is the bare `GATE PASS` of the pre-review script; after
the WR-01 fix a subset run ends with `GATE PASS (SUBSET — not the phase gate)`
so the tail cannot masquerade as the full gate. The committed cast is frozen
evidence of the mechanism and is not regenerated.

## Steps

| Step | What it records |
|---|---|
| `expected-count provenance (run 2, --refresh-expected)` | the expected file's contents: provenance header + the three seeded numbers |
| `toolchain provenance (rustc version, cargo path)` | rustc 1.94.0 (pinned) and the cargo binary used |
| `full gate confirmation (run 3)` | the gate's real exit status and printed verdict (four numbers, both comparisons, leak scan) |
| `subset-mode mechanism run (cast source)` | the cast's exit status: the subset gate green end to end |

## Re-run

```sh
scripts/test-gate.sh                  # expect: GATE PASS, results=308, exit 0
scripts/test-gate.sh --refresh-expected   # only to move the count, after a green run
```

With no `--log-dir`, a red run keeps its temporary log directory and prints the
path so the failing step log can still be read; a green run removes it (IN-10).

To regenerate the cast:

```sh
T=$(mktemp -d)
script -qec 'scripts/test-gate.sh --package xai-grok-status-line --package xai-grok-sandbox' \
    -T "$T/run.timing" "$T/run.typescript"
scripts/evidence.py cast --typescript "$T/run.typescript" --timing "$T/run.timing" \
    --out docs/evidence/01-truth-baseline/test-gate/test-gate.cast
```

The raw typescript/timing intermediates are not committed; the cast, the
normalized gate log and the step logs are (`<REPO>/<HOME>/<TMP>`).