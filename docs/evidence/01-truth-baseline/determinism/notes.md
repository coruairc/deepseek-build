# determinism

Proves TEST-03: the `session::git_gate` / `restore_fetch` timing suite is
deterministic under the phase's own command, with a committed repetition a
reviewer can recount instead of trusting an executor's recollection.

## The failure this item answers

The Phase-1 verification ran the phase's own command 35 times on HEAD
`1308903a` and caught **two failures**: the captured one was
`restore_fetch::tests::fetch_timeout_sigkills_term_immune_grandchild` panicking
at `restore_fetch_tests.rs:731` with `grandchild <pid> must not remain live`.
`restore_fetch_tests.rs` was never modified by Phase 1 — it is byte-identical
at `98c2f1b8` and `1308903a` (sha256
`62a2656acde3b233218c9e87bb71bdb358440bb688d831af55f35378b2790dd3`) — and no
`#[ignore]` quarantine existed.

Cause: the test read `/proc/<pid>/stat` immediately after teardown returned.
SIGKILL delivery and reaping are asynchronous, so a process that is already
doomed can still be observed as live for a short window; an instantaneous
snapshot is the wrong assertion for an eventual property.

Fix (plan 01-06): `wait_until_dead(pid, deadline)` polls `pid_alive` every
20 ms until it reports dead, and fails at a 5 s deadline so a process that
never dies still fails. `PROCESS_DEATH_DEADLINE = 5 s` and
`PROCESS_DEATH_POLL_INTERVAL = 20 ms`; all five process-death assertion sites
in `restore_fetch_tests.rs` use it. The prompt-return bounds
(`start.elapsed() < 5 s` / `< 2 s`) are still asserted **before** any polling,
no assertion changed direction, no `assert_timeout_about` band moved, and the
production file `restore_fetch.rs` is untouched. A Linux-only regression test
(`wait_until_dead_is_bounded_and_deadline_respecting`) pins the helper: an
absent pid returns `true` immediately, and the test's own live pid is still
reported alive across a 150 ms deadline (at least 100 ms of observation) and
returns before 2 s.

## The repetition

Command of record, run 35 times in a row:

```sh
env -u DEEPSEEK_API_KEY -u DEEPSEEK_BUILD_API_KEY -u XAI_API_KEY \
    cargo test -p xai-grok-workspace --lib -- --test-threads=4
```

Environment: the three API-key variables unset for every run, the whole
repetition under `flock -w 900 /tmp/team-build.lock`, one shared
`CARGO_TARGET_DIR` (`<REPO>/target`), no competing build started during it.
The tree already carried the warmed build; each run still re-linked the crate's
test target (the same `Compiling xai-grok-tools` → `Compiling
xai-grok-workspace` chain, `Finished test profile … in 43–49 s`), so a run is
about 50 s wall while the test harness itself finished in 5.89–6.42 s.

| Quantity | Value |
|---|---|
| Runs | 35 |
| `rc` | 0 on every run |
| Named test `restore_fetch::tests::fetch_timeout_sigkills_term_immune_grandchild` | `ok` in every run |
| `test result:` line | `ok. 1695 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` in every run |
| Distinct passed count | exactly one: `1695 passed;` |
| Wall time | 2026-11-23T19:08:13Z → 2026-11-23T19:38:44Z (30m31s) |

A fast signal ran first, in the same item: 20 consecutive runs of the exact
failing test (`--exact
restore_fetch::tests::fetch_timeout_sigkills_term_immune_grandchild`), each
`1 passed; 0 failed` in 2.40 s — `targeted-repetition-20-named-test-runs.log`
(after normalization, its per-test `… ok` lines are dropped by the evidence
convention; its `test result:` lines and the `targeted run NN/20: rc=0` markers
remain).

The repetition is the projection proof. `repetition-35-runs.log` is 35
**headerless** lines, one per run, each derived from that run's real cargo log:

```
run NN/35: rc=<rc> named=<ok|FAILED|absent> <that run's `test result:` line>
```

The driver stops at the first non-zero run and copies that run's raw log into
this directory as `FAILED-run-NN.log`; the projection log would then end at
that line and the count steps below would fail. A red repetition is committed
as red.

Raw per-run cargo logs (35 files, ~5 MB) lived in
`<TMP>/opencode/det-01-06/determinism-raw/run-NN.log` and are **not
committed**; the projection log is the durable record.

## What the cast shows

`determinism.cast` is a `script -qec … -T` capture of the **same execution**
as the `post-fix repetition 35 full-crate runs` step — not a re-run:

```sh
script -qec 'scripts/evidence.py step --item-dir docs/evidence/01-truth-baseline/determinism \
    --name "post-fix repetition 35 full-crate runs" -- sh <TMP>/determinism-rep.sh' \
    -T <TMP>/repetition.timing docs/evidence/01-truth-baseline/determinism/repetition.typescript
scripts/evidence.py cast --typescript … --timing … --out determinism.cast
```

The cast carries the 35 projection lines (3 events: `script(1)` wrote the body
in two chunks). The raw typescript and timing file were intermediates and are
not committed. The step's real exit status is in `report.json`
(`exit 0: run 35/35: rc=0 named=ok …`).

## Steps

| Step | What it records |
|---|---|
| `targeted repetition 20 named-test runs` | 20/20 exact-test runs green before the long repetition |
| `post-fix repetition 35 full-crate runs` | the repetition's real exit status, captured live (cast source) |
| `all 35 runs exited zero` | `awk` over the committed log: 35 lines matching `^run NN/35: rc=0 ` |
| `every run's summary is ok with zero failed` | `awk`: 35 lines, 35 `test result: ok.`, 35 carrying `0 failed` |
| `every run exercised the named test` | `awk`: 35 `named=ok` markers |
| `all runs report one passed count` | `awk`: exactly one distinct `<N> passed;` value |
| `leak scan over docs evidence` | `scripts/key-leak-check.sh --scan docs/evidence` → PASS, exit 0 |

The projection steps are read-only counts over the committed log; a short log
or a non-zero `rc` makes them exit non-zero, and the log is never edited to make
them pass.

## Re-run

```sh
ITEM=docs/evidence/01-truth-baseline/determinism
env -u DEEPSEEK_API_KEY -u DEEPSEEK_BUILD_API_KEY -u XAI_API_KEY \
    cargo test -p xai-grok-workspace --lib -- --test-threads=4   # expect: 1695 passed; 0 failed
```

For the full repetition, loop that command 35 times (one run's `test result:`
line per iteration), recording one `run NN/35: rc=… named=… <line>` per run.
Every run must show `rc=0`, `named=ok` and `0 failed`. With no `--log-dir`
equivalent here, keep the raw per-run logs outside the repo (`<TMP>`), as this
item did; only the projection log, cast, step logs and `report.json` are
committed.

Artifacts and their sha256 are in `report.json`; the report is `ok` only
because every recorded step genuinely passed.
