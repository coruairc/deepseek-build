# Evidence

Committed proof that a phase item actually ran. Each item is a directory a
later reader can re-run without trusting a transcript that was edited by hand.

## Layout

```
docs/evidence/NN-<phase-slug>/<item-slug>/
    <item-slug>.cast     asciinema v2 recording (or a focused `script -T` capture)
    report.json          fixed-schema summary; the artifact a reviewer reads
    notes.md             optional: what the cast shows and how to re-run it
    <step-slug>.log      per-step captured output, normalized
```

Phase 1 items live under `docs/evidence/01-truth-baseline/`. A later phase
**appends** a new `NN-<phase-slug>/` tree; it never rewrites an earlier item.
Correcting an item means a new directory with its own `report.json` and a
`notes.md` that says what replaced what.

## `report.json`

Exactly these keys, in this order (the verifier asserts the set):

| Key | Type | Meaning |
|-----|------|---------|
| `item` | string | the item slug (`test-gate`, `ipv6-report`, …) |
| `requirements` | array of strings | requirement ids the item proves (`TEST-01`, …) |
| `date` | string | UTC start time, `YYYY-MM-DDTHH:MM:SSZ` |
| `host` | string | system/arch only, e.g. `Linux/x86_64` — never a hostname or user |
| `command` | string | the command that produced the item |
| `steps` | array of objects | `{name, ok, detail}`; `detail` is `exit <n>: <first failure line or last non-empty line>` |
| `artifacts` | array of objects | `{path, sha256}`; `path` is relative to the repository root |
| `result` | string | `ok` \| `fail` \| `skipped` |

Steps are produced by `scripts/evidence.py step`, which runs the command and
records its real exit status (a command that fails to launch is recorded as a
failed step). `finish --result ok` refuses when any step failed **or no steps
were recorded**, or when a recorded artifact no longer exists on disk (it
cannot vouch for a missing file); for a `fail`/`skipped` result a missing
artifact is dropped with a note instead of keeping a stale sha256. So a report
cannot overstate a run. The step record is saved before the output is echoed,
so a broken stdout pipe cannot lose it.

## Normalization

Every captured log and every string in `report.json` replaces absolute
prefixes:

| Actual prefix | Replaced with |
|---|---|
| repository root (`$PWD` of the checkout) | `<REPO>` |
| the user's home directory | `<HOME>` |
| the system temp directory (`/tmp`) | `<TMP>` |

An environment dump is never captured. `scripts/key-leak-check.sh --scan
docs/evidence` runs over the committed tree (and as the gate's last step) and
prints file names only if the canary or an `sk-` key shape appears. A cast's
data records pass through the same substitution, so a recorded screen cannot
carry an absolute home path either.

**Scan limitation (text-only evidence).** The scan uses `grep -I`, which
skips binary files: a key inside a zipped or compiled artifact would not be
found. The evidence convention is therefore text-only — casts, logs, JSON and
Markdown. If a binary artifact is ever added, it needs a separate scan pass
without `-I` (still file-names-only) before the item is committed.

**Artifact ordering rule.** `finish` normalizes every step log, so an
artifact's sha256 must be taken over the *normalized* file: normalize a
externally produced log (`evidence.py normalize --path FILE`) **before**
`artifact` records it, or rely on `finish` recomputing every artifact digest
after normalization (it refreshes a stale digest and says so). Casts are
never rewritten by `finish`. A finished item's `report.json` and hashed
artifacts are frozen: corrections are a new item directory, never a re-run of
`finish` over the old one.

## Recording an item

```sh
ITEM=docs/evidence/01-truth-baseline/ipv6-report
scripts/evidence.py begin --item-dir "$ITEM" --item ipv6-report \
    --requirements SBOX-02 --command "sh scripts/test-sandbox-run.sh"

# Capture FIRST: script(1)'s -T writes the timing file the converter needs,
# and the assertion steps below grep the typescript this produces.
script -qec 'sh scripts/test-sandbox-run.sh' -T "$ITEM/run.timing" "$ITEM/run.typescript"
scripts/evidence.py cast --typescript "$ITEM/run.typescript" \
    --timing "$ITEM/run.timing" --out "$ITEM/ipv6-report.cast"

# One step per assertion; each runs a real check and records its exit status.
# grep -F (not -q) so the step detail carries the matched PASS line.
scripts/evidence.py step --item-dir "$ITEM" --name "ipv6-refused: names the destination" \
    -- sh -c "grep -F 'PASS: ipv6-refused: blocked line names 2001:db8::1' '$ITEM/run.typescript'"

scripts/evidence.py artifact --item-dir "$ITEM" --path "$ITEM/ipv6-report.cast"
scripts/evidence.py finish --item-dir "$ITEM" --result ok
```

Headless items use `script -T`. A TUI item uses the pty harness: run one
`#[ignore]`d pty test with `GROK_PTY_CAST_DIR=docs/evidence/01-truth-baseline/<item>`
and let `write_cast_if_requested` emit asciinema v2. `asciinema` itself is not
installed and must not be.

A file produced outside `steps` (for example the gate's own cargo log) is
normalized with `scripts/evidence.py normalize --path FILE` before it is
recorded as an artifact.

## Re-running the Phase-1 items

- **test-gate** — `scripts/test-gate.sh --refresh-expected` seeds the count from
  a green full run; `scripts/test-gate.sh` confirms it. The cast is a
  `script -T` capture of the subset-mode run (the mechanism).
- **ipv6-report** — `sh scripts/test-sandbox-run.sh` (offline; no network beyond
  loopback).
- **cost-estimate** — `cargo test -p xai-grok-pager --lib status_line` and
  `cargo test -p xai-grok-pager --lib headless`; the report records the strings
  the suite pins.

After recording: `scripts/key-leak-check.sh --scan docs/evidence` must exit 0.
