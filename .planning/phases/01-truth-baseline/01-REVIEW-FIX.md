---
phase: 01-truth-baseline
fixed_at: 2026-11-23T16:09:30Z
review_path: .planning/phases/01-truth-baseline/01-REVIEW.md
iteration: 2
findings_in_scope: 4
fixed: 4
skipped: 0
status: all_fixed
---

# Phase 1 (plan 01-05): Code Review Fix Report — Iteration 2

**Fixed at:** 2026-11-23T16:09:30Z
**Source review:** `.planning/phases/01-truth-baseline/01-REVIEW.md` (iteration-2
re-review, 2026-11-23T15:16:30Z)
**Iteration:** 2

**Summary:**
- Findings in scope: 4 (WR-02 regression, IN-08, IN-09, IN-10 — dispatch scope)
- Fixed: 4
- Skipped: 0
- Deferred by dispatch: 1 (IN-07 — ledgered below and in
  `01-REVIEW-DISPOSITION.md`)

**Commit protocol:** the same override as iteration 1 — NO commits, no `git
add`/`stash`/`reset`. All fixes are uncommitted in the main checkout for
re-review; the orchestrator commits after the reviewer's PASS. No worktree was
created (dispatch override + team rule "single working tree"). The
orchestrator-owned `.planning/` state files (`STATE.md`, `config.json`,
`state.json`, `ROADMAP.md`) were not touched.

**Evidence integrity:** the three finished `report.json` files and their four
hashed artifacts were verified untouched (8-key schema 3/3, sha256 4/4 match,
before and after the fixes). `scripts/test-gate.expected` was re-seeded by the
sanctioned `--refresh-expected` writer (run #4): identical 308/219/89 and a
header-only diff; the expected file is not a hashed artifact. No `finish` was
re-run on a finished item; no `report.json` was hand-edited; the three
committed casts were not regenerated.

**Verification location:** every check below ran in the main checkout
(`/home/cein_orourke/Projects/deepseek-build`, base `c159a3e7` + the uncommitted
01-05 delta), not in a worktree — the results are reproducible from the tree
the reviewer sees.

## Fixed Issues

### WR-02: `--refresh-expected` tree-state stamp always "dirty" — the `clean` branch was unreachable [regression of the iteration-1 fix]

**Files modified:** `scripts/test-gate.sh`;
`docs/evidence/01-truth-baseline/test-gate/notes.md` (provenance rewrite);
`.planning/phases/01-truth-baseline/01-05-SUMMARY.md` (wording)
**Commit:** none (dispatch override)
**Applied fix:** the tree state is now computed into `tree_state` BEFORE the
`{ … } >"$EXPECTED_FILE"` redirection group opens, and the group only echoes
it:

```sh
tree_state="clean"
if [ -n "$(git -C "$root" status --porcelain)" ]; then
    tree_state="dirty (uncommitted changes present at refresh time)"
fi
{
    echo "# refreshed by: scripts/test-gate.sh --refresh-expected"
    echo "# commit: $(git -C "$root" rev-parse HEAD)"
    echo "# tree: $tree_state"
    ...
} >"$EXPECTED_FILE"
```

The redirect truncates the file before the group runs, so the old inline check
always saw its own write; computing before the redirect cannot regress. The
rest of the header format is unchanged.
**Verified (both branches, real script):** throwaway git repo containing a COPY
of the real `scripts/test-gate.sh` with stubbed `cargo`/`disk-guard`, committed
`test-gate.expected`: (1) clean tree → refresh exits 0 and the header reads
`# tree: clean`; (2) after committing the refreshed file and adding an
untracked marker → refresh exits 0 and the header reads `# tree: dirty
(uncommitted changes present at refresh time)`. Temp repos removed.
**Verified (real refresh, run #4):** the full gate (below) rewrote
`scripts/test-gate.expected` with commit `c159a3e7`, the honest
`# tree: dirty (uncommitted changes present at refresh time)` marker (the tree
carries the uncommitted review fixes), 2026-11-23T16:08:34Z, and **unchanged**
numbers 308/219/89; the diff against HEAD is header-only.

### IN-08: `finish` kept a recorded sha256 for an artifact missing at finish

**Files modified:** `scripts/evidence.py` (`cmd_finish`, module docstring);
`docs/evidence/README.md` (finish semantics)
**Commit:** none (dispatch override)
**Applied fix:** before normalization, `finish` checks every recorded artifact
against disk. `--result ok` refuses with exit 2 naming each missing artifact
(fail-closed, like a failed step: a report cannot vouch for a file it cannot
read). For `--result fail`/`skipped` the missing artifact is dropped from the
report with an explicit stderr note, so no result mode can carry a stale digest.
Existing artifacts still get the iteration-1 post-normalization digest
recompute.
**Verified:** temp-dir smoke — artifact recorded, file deleted → `finish
--result ok` exits 2 naming the artifact and writes no `report.json`; `finish
--result fail` exits 0, notes the drop, and the report's `artifacts` is `[]`;
an existing artifact still succeeds, prints `refreshed sha256 …`, and the
report digest equals the file on disk.

### IN-09: cast timing with only non-positive lengths still degraded to one t=0 event

**Files modified:** `scripts/evidence.py` (`cmd_cast` guard, module docstring)
**Commit:** none (dispatch override)
**Applied fix:** the guard is now
`if body and not any(length > 0 for _, length in timing)` — a non-empty body
with zero parsed pairs *or* no positive-length pair exits 2 with the
"no usable `elapsed bytes` pairs (no positive byte counts)" diagnostic. The
docstring says "no positive-length pairs", matching the code.
**Verified:** temp-dir smoke — timing `0.1 0 / 0.2 0 / 0.3 -5` over a non-empty
body → exit 2, diagnostic, no cast written; empty timing → still exit 2 (IN-02
regression check); one positive pair → converts with one event; a zero pair
alongside a positive pair → converts.

### IN-10: a failing gate deleted the step logs it pointed at

**Files modified:** `scripts/test-gate.sh` (temp log dir + exit trap; header
comment)
**Commit:** none (dispatch override)
**Applied fix:** with no `--log-dir`, the mktemp dir is now removed by the EXIT
trap only when the exit status is 0; a failing run keeps the directory and
prints `gate: failing run kept its logs at <dir>` to stderr. Exit codes are
unchanged, and an explicit `--log-dir` behaves exactly as before.
**Verified:** the real script under stubbed cargo in a throwaway repo — a
failing run exits 1, its printed `step1-tests.log` path exists afterwards, and
the kept-logs line names the directory; a green run exits 0, prints no
kept-logs line, and its temp dir is gone. Temp repos removed.

## Deferred Issues

### IN-07: default (no-arg) `key-leak-check.sh` still treats grep rc ≥ 2 as "no hit"

**File:** `scripts/key-leak-check.sh:102-105`
**Reason:** pre-existing; default mode frozen by plan 01-05 task 3(b); `--scan`
mode (the gate step) now fails closed (CR-01). The reviewer explicitly allowed
deferral, and the dispatch directed it. Ledgered in `KNOWN-ISSUES.md`
("API key handling") with the follow-up (same rc 0/1/else branch with
diagnostics only, then pinned in TESTING step 10) and recorded as `deferred` in
`01-REVIEW-DISPOSITION.md`.
**Original issue:** an unreadable file or I/O error (grep rc 2) leaves `hits=0`
and the script prints PASS without a complete scan.

## Verification log (iteration 2, main checkout)

| # | Check | Result |
|---|---|---|
| 1 | `bash -n scripts/test-gate.sh && shellcheck --shell=sh scripts/test-gate.sh scripts/key-leak-check.sh` | exit 0, no output |
| 2 | `python3 -c 'import ast; ast.parse(open("scripts/evidence.py").read())'` | parses |
| 3 | WR-02 + IN-10 harness (real script copy, stubbed cargo/disk-guard, throwaway git repos) | 12/12 PASS: clean → `# tree: clean`; dirty → dirty marker; failing run keeps + prints its step log; green run removes it; temp repos removed |
| 4 | IN-08/IN-09 temp-dir smoke | 16/16 PASS (missing-artifact refusal + drop, digest recompute, all-non-positive timing refusal, empty-timing regression, positive/mixed timing) |
| 5 | REAL full gate run #4: `env -u DEEPSEEK_API_KEY -u DEEPSEEK_BUILD_API_KEY -u XAI_API_KEY scripts/test-gate.sh --refresh-expected --log-dir /tmp/opencode/gate-refresh-postfix` (the script re-execs under `flock -w 900 /tmp/team-build.lock`) | exit 0; step 1 PASS, step 2 PASS; `results=308 (raw 309) binaries=219 doctests=89`; `results == binaries + doctests` PASS (308 == 308); leak scan PASS; `GATE PASS`. Step 1 totals 31,913 / 0 / 386; step 2 6,201 / 0 / 4 — **identical to the frozen evidence** |
| 6 | `scripts/test-gate.expected` after run #4 | commit `c159a3e7`, `# tree: dirty (uncommitted changes present at refresh time)`, 2026-11-23T16:08:34Z, results=308 binaries=219 doctests=89; `git diff HEAD` header-only (numbers unchanged) |
| 7 | 8-key schema assert on all 3 `report.json` + sha256 of every artifact vs disk | 3/3 exact keys in order; 4/4 MATCH (after all edits — frozen artifacts untouched) |
| 8 | `scripts/key-leak-check.sh --scan docs/evidence` | PASS, exit 0 |
| 9 | `grep -rnE '/home/[a-z]' docs/evidence` and `grep -rnE 'sk-[A-Za-z0-9]{20,}' docs/evidence` | no hits (rc=1 both) |
| 10 | Scope: `git diff HEAD --name-only` / `git status --short` | only the intended scripts, docs and evidence/planning files; no Cargo.toml/lockfile/deny.toml change; no `report.json` change |

## Deviations / notes

- **No commits** (dispatch override): the fixer's atomic-commit protocol stays
  suspended; everything is uncommitted for re-review.
- **IN-10 shellcheck pragma:** the trap assigns `status=$?` inside its own
  command string; shellcheck's textual analysis reports SC2154, so the trap
  carries a one-line `# shellcheck disable=SC2154` with the reason.
- **IN-08 policy:** chose the reviewer's fail-closed option for `--result ok`
  and the drop-with-note option for non-ok results, so no result mode can keep
  a digest for a missing file.
- **Expected-file re-seed:** run #4 is the only writer permitted to touch
  `scripts/test-gate.expected`; the refresh was explicitly authorized by the
  dispatch after the harness proved both tree-state branches.

---

_Fixed: 2026-11-23T16:09:30Z_
_Fixer: the agent (gsd-code-fixer)_
_Iteration: 2_

# Appendix — iteration-1 report (superseded; retained for the record)


## Fixed Issues

### CR-01: `--scan` fails open — grep exit 2 treated as "no key found" [BLOCKER]

**Files modified:** `scripts/key-leak-check.sh`
**Applied fix:** the `--scan` verdict now branches on grep's exit code:
rc=0 → FAIL with file names (unchanged); rc=1 → PASS; any other rc (≥2, error)
→ `FAIL  leak scan could not complete (grep rc=N)`, prints grep's captured
stderr diagnostics (`$WORK/scan-err.txt`, never matched lines), exit 1.
The script runs `set -uo pipefail` without `-e`, so capturing `$?` is safe.
The diff is a single hunk inside the `--scan` branch — the default (no-arg)
mode is byte-identical in behavior.
**Verified:** synthetic-canary reproduction in a temp dir (no real key
material): a mode-000 file containing `sk-FAKE-LEAKCANARY-…` → grep rc=2 →
**FAIL exit 1** with only the `Permission denied` diagnostic; the same file
made readable → FAIL exit 1 naming the file (match branch intact); a clean
dir → PASS exit 0. Temp dir removed. Full `--scan docs/evidence` → PASS exit 0.

### WR-01: Subset mode ends with the identical verdict line "GATE PASS"

**Files modified:** `scripts/test-gate.sh`
**Applied fix:** the final verdict is mode-aware: subset runs end with
`GATE PASS (SUBSET — not the phase gate)`; full runs keep the bare `GATE PASS`.
Scope kept to the review's snippet: the `GATE FAIL` line is unchanged (the
finding concerns the pass verdict masquerading as the full gate).
**Verified:** real subset run
`scripts/test-gate.sh --package xai-grok-status-line --package xai-grok-sandbox`
(keys unset, gate's own `flock -w 900 /tmp/team-build.lock`): exit 0,
`results=9 == binaries 7 + doctests 2`, expected check SKIPPED, leak scan
rc=0, and the last line is exactly `GATE PASS (SUBSET — not the phase gate)`.
Full-mode branch confirmed by inspection (no full re-run needed). The frozen
subset cast still shows the pre-fix bare `GATE PASS`; `notes.md` now says so
explicitly (the cast is historical evidence and was not regenerated).

### WR-02: `--refresh-expected` stamps HEAD even on a dirty tree

**Files modified:** `scripts/test-gate.sh` (refresh WRITER only),
`docs/evidence/01-truth-baseline/test-gate/notes.md` (unhashed)
**Applied fix:** the refresh header now stamps the working-tree state after
the commit line: `# tree: dirty (uncommitted changes present at refresh time)`
when `git status --porcelain` is non-empty, else `# tree: clean`. The header
comment was updated to "commit/tree-state/date/count provenance". Per the
dispatch constraint, history was NOT re-stamped: `scripts/test-gate.expected`
is frozen and untouched; instead `notes.md` gained a "Provenance caveat
(WR-02)" paragraph recording that run #2's stamp `882ad0de`
(2026-11-21T19:38:36Z) predates WIP `b0516ef1` (22:34:33Z) — the seed ran on a
dirty tree and the stamped commit alone cannot reproduce it — and that run #3
re-confirmed 308/219/89 on the merged tree (the stronger evidence).
**Verified:** `bash -n` + `shellcheck --shell=sh` clean; writer logic by
inspection (no `--refresh-expected` run — it needs a green FULL run, which
these fixes do not require); `git status` confirms `test-gate.expected`
untouched. Future refreshes carry the tree state.

### WR-03: `finish --result ok` accepted with zero steps; launch failure unrecorded

**Files modified:** `scripts/evidence.py` (`cmd_finish`, `cmd_step`),
`docs/evidence/README.md` (finish semantics sentence)
**Applied fix:** `finish --result ok` now refuses with exit 2 when
`state["steps"]` is empty ("no steps were recorded"), before the existing
failed-step refusal. `cmd_step` wraps `subprocess.run` in `try/except OSError`:
a command that never launches appends `{name, ok: False, detail: "exit n/a:
<error>"}`, saves pending state, writes the error to stderr, and exits 1 — so
a subsequent `finish --result ok` cannot succeed over an unrecorded failure.
**Verified:** temp-dir smoke suite — nonexistent command → exit 1 + failed
step recorded with `exit n/a:` detail; `finish --result ok` → exit 2 naming
the failed step; `finish --result fail` → exit 0, schema-exact report;
fresh item with zero steps → `finish --result ok` exit 2 ("no steps"),
`--result skipped` allowed.

### WR-04: step record saved after stdout echo — broken pipe loses the record

**Files modified:** `scripts/evidence.py` (`cmd_step`)
**Applied fix:** the step record is appended and `save_pending` runs BEFORE
the output echo; the echo (`write` + `flush`) is wrapped in
`try/except BrokenPipeError`, and on a broken pipe stdout is pointed at
devnull (`os.dup2`) so the interpreter's final flush stays quiet. The exit
status is still the command's.
**Verified:** real broken-pipe reproduction — `Popen` with `stdout=PIPE`, read
end closed before the child echoes: exit 0 (the command's status), no
traceback on stderr, step record present in pending state
(`exit 0: survived`), and `finish --result ok` succeeds over it. Ordering also
confirmed by code inspection.

### WR-05: `finish` rewrites step logs after artifacts were hashed — stale sha256

**Files modified:** `scripts/evidence.py` (`cmd_finish`),
`docs/evidence/README.md` (new "Artifact ordering rule" paragraph)
**Applied fix:** after the normalization loop, `finish` recomputes every
artifact's sha256 from the file on disk (`REPO_ROOT / artifact["path"]`) and
refreshes a stale digest, printing which artifact changed. The README now
documents the ordering rule: normalize externally produced logs BEFORE
`artifact` records them, or rely on the finish-time recompute; casts are never
rewritten; finished items are frozen (corrections are a new item directory).
**Verified:** temp-dir smoke — step log containing an absolute repo/home path,
`artifact` records the RAW digest, `finish` announces the refresh, and the
report's sha256 equals the normalized file on disk (≠ raw digest). The three
finished reports were NOT touched: all 4 committed artifact sha256s still
match (re-checked after the fixes).

### WR-06: per-chunk cast normalization — split-prefix escape

**Files modified:** `scripts/evidence.py` (`cmd_cast` + new helpers
`_split_carry`, `_strip_script_trailer`)
**Applied fix:** chose the review's explicitly-allowed alternative — a
partial-prefix carry between chunks (equivalent to normalizing the assembled
body, while keeping the timing file's exact event boundaries). Each raw chunk
is split into (emit, carry): trailing bytes that are a proper prefix of any
substitution target (`<REPO>`/`<HOME>`/`<TMP>` sources) — or an incomplete
UTF-8 sequence — are held and prepended to the next chunk; the emitted part is
decoded and normalized. After the loop, any bytes the timing file did not
cover plus the final carry are flushed, so the concatenated event data is
exactly the normalized body (no output silently dropped). The three committed
casts were NOT regenerated (they were produced by the old code and verified
clean, including concatenated-event analysis).
**Verified:** synthetic typescript+timing pair in a temp dir where
`/home/cein_orourke` is split across two timing chunks (12 bytes | rest):
the resulting cast's concatenated event data contains `<HOME>`, NO `/home/`
fragment, equals `echo <HOME>/x\n` exactly, and keeps 2 events. Red proof: the
pre-fix code (`git show HEAD:scripts/evidence.py`) on the same pair leaks the
split `/home/` fragment with no `<HOME>`. Also verified: a repo-path split and
a multi-byte UTF-8 split (€ = `e2 82 ac` across chunks) → `<REPO>` present,
euro sign intact, exact normalized body. Committed casts re-checked: sha256
4/4 match, concatenated data clean.

### IN-01: Evidence README recipe ordered wrong; undefined `<captured>`

**Files modified:** `docs/evidence/README.md`
**Applied fix:** the `script -T` capture + `cast` block now comes BEFORE the
assertion step; `<captured>` is replaced with the defined
`'$ITEM/run.typescript'`; `grep -qF` became `grep -F` so the step detail
carries the matched PASS line. A comment states capture-first explicitly.
**Verified:** inspection; the recipe's grep string matches the committed
`ipv6-refused-blocked-line-names-2001-db8-1.log` content verbatim
(`PASS: ipv6-refused: blocked line names 2001:db8::1`).

### IN-02: cast docstring overstates framing removal; empty timing degrades silently

**Files modified:** `scripts/evidence.py` (`cmd_cast`, module docstring)
**Applied fix:** explicit strip of a trailing `Script done …` line
(`_strip_script_trailer`, tolerates a trailing newline) instead of relying on
the timing file not covering it; missing timing file → exit 2; timing with no
usable pairs over a non-empty body → exit 2 (the old silent single-event t=0
fallback is gone). Docstring corrected to describe the actual mechanism.
**Verified:** smoke — a trailer COVERED by the timing file is stripped
(concatenated data `hello\n`, no `Script done`); missing timing → exit 2;
empty timing + non-empty body → exit 2.

### IN-03: TESTING.md contradicts itself on `check-egress.sh --strict`

**Files modified:** `TESTING.md` (§3 step 11)
**Applied fix:** step 11's expectation is now the known red baseline: exit 1,
`HARD EGRESS VIOLATION: forbidden destination/host strings found:` with
`(3 hard hits total; …)` on the three pre-existing strings (`api.x.ai`,
`grok.com`, `gs://bucket`), pointing at KNOWN-ISSUES.md and Phase 5 / FREEZE-03
as the fix owner — consistent with the status banner. Wording notes the script
exits at the HARD stage before the SOFT scan runs (observed behavior).
**Verified:** ran `scripts/check-egress.sh --strict` this session: exit 1,
exactly the documented HARD violation line and `(3 hard hits total; …)`.

### IN-04: cited totals (step-2 6,201/0/4; five-run 1694×5) not artifact-backed

**Files modified:** NEW
`docs/evidence/01-truth-baseline/test-gate/step2-config-docs-result-line.log`;
`docs/evidence/01-truth-baseline/test-gate/notes.md`; `KNOWN-ISSUES.md`;
`TESTING.md`; `.planning/phases/01-truth-baseline/01-05-SUMMARY.md`
**Applied fix:** the step-2 raw log EXISTS at
`/tmp/opencode/gate-run3/step2-config-docs.log`; its `Running` line (319) and
`test result:` line (6528) were extracted verbatim into the new supplementary
log (with a provenance header), normalized via `evidence.py normalize`
(`/tmp` → `<TMP>`), and referenced from notes.md, KNOWN-ISSUES.md, TESTING.md
and the SUMMARY wherever 6,201/0/4 is quoted. The five-run repetition raw logs
are NOT on disk, so those numbers are now explicitly marked "**reported, not
artifact-backed (prior session)**" in KNOWN-ISSUES.md, TESTING.md and the
SUMMARY (section heading + D3 verification ref), with the note that run #3's
committed step-1 log independently shows `xai-grok-workspace` 1694/0 once
(`test-gate.log` line 1934). No finished report was touched; the new file is
supplementary, not a re-finish.
**Verified:** excerpt lines diff-identical to raw-log lines 319/6528;
`--scan` over the new file → PASS; full `--scan docs/evidence` → PASS exit 0;
`/home/[a-z]` and `sk-` greps over docs/evidence (incl. the new file) → no
hits. Bonus: the post-fix subset gate re-ran step 2 live and re-confirmed
`6201 passed; 0 failed; 4 ignored` (51.94 s).

### IN-05: `--scan` skips binary files (`grep -I`)

**Files modified:** `docs/evidence/README.md` (new "Scan limitation (text-only
evidence)" paragraph); `scripts/key-leak-check.sh` comment now points at it
**Applied fix:** documented the limitation in the README's scan section: the
evidence convention is text-only (casts, logs, JSON, Markdown); a binary
artifact would need a separate scan pass without `-I` (still file-names-only)
before commit. Chose the documentation option over a second scan pass (keeps
the gate's scan behavior unchanged).
**Verified:** inspection; scan behavior unchanged (full scan PASS).

### IN-06: `normalize`/`finish` strict UTF-8 crashes on non-UTF-8 input

**Files modified:** `scripts/evidence.py` (`cmd_finish` log read,
`cmd_normalize`)
**Applied fix:** `read_text(encoding="utf-8", errors="replace")` in both
places, matching `cmd_step`/`cmd_cast`.
**Verified:** smoke — a file with invalid bytes (`\xff\xfe`) plus an absolute
home path normalizes with exit 0 and carries `<HOME>`.

## Skipped Issues

None — all 13 in-scope findings were fixed.

## Verification log (this fix session, main checkout)

| # | Check | Result |
|---|---|---|
| 1 | `bash -n scripts/test-gate.sh && shellcheck --shell=sh scripts/test-gate.sh scripts/key-leak-check.sh` | exit 0, no output (also `bash -n` key-leak-check.sh) |
| 2 | `python3 -c 'import ast; ast.parse(open("scripts/evidence.py").read())'` | parses; no `__pycache__` created |
| 3 | `scripts/key-leak-check.sh --scan docs/evidence` | PASS, exit 0 |
| 4 | CR-01 reproduction (mode-000 synthetic canary, temp dir) | FAIL exit 1, grep rc=2 diagnostics only; readable canary → FAIL exit 1 w/ file name; clean → PASS exit 0; temp dir removed |
| 5 | Subset gate: `env -u DEEPSEEK_API_KEY -u DEEPSEEK_BUILD_API_KEY -u XAI_API_KEY scripts/test-gate.sh --package xai-grok-status-line --package xai-grok-sandbox --log-dir /tmp/opencode/gate-subset-postfix` (gate re-execs under `flock -w 900 /tmp/team-build.lock`) | exit 0; 9 == 7 + 2; expected SKIPPED; step 3 rc=0; last line `GATE PASS (SUBSET — not the phase gate)`; step 2 re-confirmed 6201/0/4 |
| 6 | 8-key schema assert on all 3 `report.json` + sha256 of every artifact vs disk | 3/3 exact keys in order; 4/4 MATCH (before and after fixes — frozen artifacts untouched) |
| 7 | `grep -rnE '/home/[a-z]' docs/evidence` and `grep -rnE 'sk-[A-Za-z0-9]{20,}' docs/evidence` | no hits (rc=1 both), incl. the new excerpt file; concatenated cast event data also clean (12/8/774 events) |
| 8 | evidence.py temp-dir smoke suite (31 checks): begin → step ok → launch-failure step → finish ok refused → finish fail; zero-step finish ok refused; real broken-pipe step; stale-digest refresh; non-UTF-8 normalize; split-prefix cast (home/repo/UTF-8) + red proof vs HEAD code; trailer strip; missing/empty timing errors | 31/31 PASSED; temp dirs cleaned |
| 9 | `cargo fmt --all --check` | exit 0 (no Rust changes made) |
| 10 | `scripts/check-egress.sh --strict` (IN-03 expectation check) | exit 1, `HARD EGRESS VIOLATION`, `(3 hard hits total; …)` — matches the documented baseline |
| 11 | Manifest/frozen guard: `git diff HEAD --name-only` vs `Cargo.toml`/`Cargo.lock`/`deny.toml`/`test-gate.expected` | no matches; `test-gate.expected` unmodified |
| 12 | `git diff HEAD -- scripts/key-leak-check.sh` hunk scope | single hunk `@@ -59,13 +59,23 @@` inside the `--scan` branch — default mode byte-identical |
| 13 | Step-2 excerpt vs raw log (`sed -n '319p;6528p'` diff) | identical |

## Deviations / notes

- **No commits** (dispatch override): the fixer's atomic-commit protocol was
  suspended; everything is uncommitted for re-review. No worktree was created
  (team rule: single working tree; stricter rule wins).
- **WR-06 approach:** partial-prefix carry (explicitly allowed by the review
  and the dispatch constraint) rather than whole-body re-slicing — it keeps
  the timing file's exact event boundaries and guarantees concatenated data ==
  normalized body. Bytes the timing file does not cover are now flushed
  instead of silently dropped (the old trailer-drop side effect is replaced
  by IN-02's explicit strip).
- **WR-01 scope:** only the PASS verdict is mode-aware, per the review's
  snippet; `GATE FAIL` is unchanged.
- **IN-03 wording:** adjusted after observing that `check-egress.sh` exits at
  the HARD stage before the SOFT scan — the step does not claim an observable
  SOFT result.
- **IN-04:** the five-run repetition could not be made artifact-backed (raw
  logs not on disk, prior session); marked "reported, not artifact-backed"
  everywhere quoted, per the dispatch guidance. The step-2 numbers ARE now
  artifact-backed via the new excerpt.
- Pre-existing baseline edits (orchestrator `.planning/` state files, the
  cosmetic `#` comment fix in test-gate.sh, frozen `test-gate.log`) were left
  exactly as found.

---

_Fixed: 2026-11-23T15:07:47Z_
_Fixer: the agent (gsd-code-fixer)_
_Iteration: 1_
