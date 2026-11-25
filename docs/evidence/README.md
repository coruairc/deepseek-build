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
- **determinism** — the phase's own 35-run repetition:
  `env -u DEEPSEEK_API_KEY -u DEEPSEEK_BUILD_API_KEY -u XAI_API_KEY cargo test
  -p xai-grok-workspace --lib -- --test-threads=4`, looped 35 times, projecting
  one `run NN/35: rc=… named=… <test result:>` line per run into
  `determinism/repetition-35-runs.log`. Every run must show `rc=0`, `named=ok`
  and `0 failed`; `notes.md` has the recipe, and the cast is the projection
  stream of that same execution (not a re-run).

After recording: `scripts/key-leak-check.sh --scan docs/evidence` must exit 0.

## Re-running the Phase-2 items

Phase-2 items live under `docs/evidence/02-loop-item-verification/`. The two
live items need a real key exported in the parent shell and a build host; the
rest are offline. The four pty commands need a `PAGER_BINARY` copy of the
release binary and an **absolute** `GROK_PTY_CAST_DIR` (cargo runs the test
binary with its crate directory as cwd, so a repo-relative path writes beside
the crate).

- **live-smoke** — export `DEEPSEEK_API_KEY` in the parent shell (never in the
  captured command), then
  `BIN=<private copy of target/release/deepseek-build> script -qec
  'scripts/egress-check.sh' -T run.timing run.typescript` and the same
  `script -qec 'scripts/smoke-test.sh'` capture. Expect `EGRESS-CHECK PASS`
  from the first and `8 passed, 0 failed` from the second.
- **runtime-egress** — `python3
  docs/evidence/02-loop-item-verification/runtime-egress/destination-check.py
  docs/evidence/02-loop-item-verification/live-smoke/live-smoke-transcript.txt
  /dev/null` re-derives the measurement from the committed transcript (no key,
  no network). Expect `parsed 4 rows; API_CONNECTS=7; WEB_FETCH_CONNECTS=1`,
  exit 0.
- **key-leak** — `scripts/key-leak-check.sh` (its own fake key, real endpoint)
  expects `PASS  fake key absent from N saved file(s) and captured output`;
  then `scripts/key-leak-check.sh --scan docs/evidence` expects `PASS  no
  key-shaped string in: docs/evidence`, exit 0. The revocation half is a human
  action recorded as a dated attestation in `key-leak/notes.md`; it has no
  re-run.
- **permission-prompt** — `PAGER_BINARY=… GROK_PTY_CAST_DIR=<absolute item
  path> cargo test -p xai-grok-pager-pty-harness --test pty_e2e_shell_tools --
  --ignored --nocapture permission_prompt_outcomes_and_warning`, then
  `cargo test -p xai-grok-pager-bin --test headless_loop_items -- --nocapture
  headless_permission_denial`. Expect `test result: ok. 1 passed; 0 failed`
  from each.
- **permission-mode** — the same pty target with
  `permission_mode_chip_matches_after_resume`, then
  `cargo test -p xai-grok-pager-bin --test headless_loop_items -- --nocapture
  headless_permission_mode_plan_arms_plan_mode`. Expect `test result: ok. 1
  passed; 0 failed` from each.
- **resume-fidelity** — `cargo test -p xai-grok-pager-bin --test
  headless_loop_items -- --nocapture
  headless_resume_restores_model_effort_and_plan_state`, then
  `cargo test -p xai-grok-pager-pty-harness --test plan_approval_resume --
  --nocapture`. Expect `test result: ok. 1 passed; 0 failed` from each.
- **plan-read-only** — the pty target with
  `plan_mode_read_only_in_every_permission_mode` (three permission modes), then
  `cargo test -p xai-grok-shell --lib plan_mode_edit_gate`. Expect `test
  result: ok. 1 passed; 0 failed` and `11 passed; 0 failed` respectively.

After recording: `scripts/key-leak-check.sh --scan docs/evidence` must exit 0.
