---
phase: 01-truth-baseline
reviewed: 2026-11-23T16:21:45Z
depth: standard
files_reviewed: 21
files_reviewed_list:
  - scripts/test-gate.sh
  - scripts/test-gate.expected
  - scripts/evidence.py
  - scripts/key-leak-check.sh
  - scripts/README.md
  - crates/codegen/xai-grok-shell/src/agent/folder_trust.rs
  - crates/codegen/xai-grok-shell/src/util/config/consent_tests.rs
  - crates/codegen/xai-grok-workspace/src/session/git_gate_tests.rs
  - KNOWN-ISSUES.md
  - TESTING.md
  - docs/evidence/README.md
  - docs/evidence/01-truth-baseline/test-gate/report.json
  - docs/evidence/01-truth-baseline/test-gate/notes.md
  - docs/evidence/01-truth-baseline/test-gate/test-gate.log
  - docs/evidence/01-truth-baseline/test-gate/test-gate.cast
  - docs/evidence/01-truth-baseline/test-gate/full-gate-confirmation-run-3.log
  - docs/evidence/01-truth-baseline/test-gate/step2-config-docs-result-line.log
  - docs/evidence/01-truth-baseline/cost-estimate/report.json
  - docs/evidence/01-truth-baseline/cost-estimate/notes.md
  - docs/evidence/01-truth-baseline/ipv6-report/report.json
  - docs/evidence/01-truth-baseline/ipv6-report/notes.md
findings:
  critical: 0
  warning: 0
  info: 0
  total: 0
status: clean
---

# Phase 1 (plan 01-05): Code Review Report — Iteration 3 (final)

**Reviewed:** 2026-11-23T16:21:45Z
**Depth:** standard
**Files Reviewed:** 21
**Status:** clean
**Verdict:** **PASS.** CR-01 fixed; WR-01…WR-06 fixed — WR-02 is now correct and proven with the real writer, including a real `--refresh-expected` run #4 whose header is truthful; IN-01…IN-06 fixed; IN-07 deferred (plan-constrained by task 3(b) and ledgered) — **not open**; IN-08…IN-10 fixed. No blocker, no warning, no new finding.

## Summary

Scope: iteration-3 final re-review of plan 01-05 — HEAD `c159a3e7` (diff base `882ad0de`) plus the uncommitted delta (`git diff HEAD` + untracked files), excluding orchestrator-owned `.planning/` state (`STATE.md`, `config.json`, `state.json`, `ROADMAP.md`) and reviewer artifacts (`01-REVIEW*`, `01-SECURITY.md`, `01-VALIDATION.md`). The three Rust test files are unchanged in the working tree (`git diff HEAD -- crates/` empty); their iteration-1 content verification (waits widened only, no assertion direction changed, bare serial keys) stands and the `882ad0de..HEAD` diff was re-checked to confirm it.

Everything the iteration-2 fix report claims was verified against the tree, not taken on trust, and the two behavioural fixes were reproduced independently:

- **WR-02 is fixed.** `scripts/test-gate.sh:278-291` computes `tree_state` **before** the `{ … } >"$EXPECTED_FILE"` group opens and only echoes it inside. A throwaway-repo harness using a copy of the real script (stubbed `cargo`/`disk-guard`) reaches **`# tree: clean`** on a committed clean tree and the dirty marker after adding an untracked file (23/23 checks). The real run #4 header is truthful and corroborated: commit `c159a3e7…` == `HEAD`, `# tree: dirty (uncommitted changes present at refresh time)` (the tree carries the uncommitted review delta), `# date: 2026-11-23T16:08:34Z` == the file's mtime, and the numbers 308/219/89 equal the run's own step logs (re-derived: 219 `Running` blocks + 89 `Doc-tests`; step 1 31,913/0/386, step 2 6,201/0/4). `git diff HEAD -- scripts/test-gate.expected` is **header-only** (2 lines replaced, 1 added; the `results`/`binaries`/`doctests` lines are byte-identical).
- **IN-08, IN-09, IN-10 are fixed** and behave as reported; exit codes and the other paths (`--result fail|skipped`, explicit `--log-dir`, green run) are unaffected. Reproduced 19/19 and 23/23 in independent harnesses.
- **IN-07 deferral is valid.** Plan 01-05 task 3(b) freezes the no-arg mode verbatim ("With no arguments the script must behave exactly as it does today, including exit codes and the binary precheck"); the ledger line exists in `KNOWN-ISSUES.md` ("API key handling") and the disposition row is `deferred`. Per the dispatch, this is not an open warning.
- **No regression in the iteration-2 delta:** no weakened check, no new fail-open path, docs match behaviour (except one non-blocking historical-staleness observation below), `bash -n`/`shellcheck --shell=sh`/`ast.parse` clean, frozen evidence 3/3 schema + 4/4 sha256, leak scan PASS, no manifest change.

Files unchanged since the iteration-2 review (the three Rust tests, the frozen `report.json`s/casts/logs, `TESTING.md`, `test-gate.log`) were re-verified by mtime + digest/schema rather than re-read line-by-line; that earlier verification is referenced, not re-claimed.

## Finding-by-finding adjudication (iterations 1–3)

| ID | Iter-1 severity | Iteration-3 verdict | Evidence (this session unless noted) |
|---|---|---|---|
| CR-01 | critical | **FIXED** | `key-leak-check.sh --scan` rc 0/1/else branch unchanged since iteration 2. Re-tested: mode-000 canary → `FAIL leak scan could not complete (grep rc=2)` exit 1 (diagnostics only); readable canary → `FAIL … 1 file(s)` + name, exit 1; clean → PASS exit 0; `--scan docs/evidence` → PASS exit 0 |
| WR-01 | warning | **FIXED** | Mode-aware final verdict (`test-gate.sh:318-325`); subset harness (iteration 2) and run #4's full `GATE PASS`; the iteration-2 delta does not touch the counting/expected logic (diff re-read) |
| WR-02 | warning | **FIXED** | `test-gate.sh:272-291` compute-before; harness: clean → `# tree: clean`, dirty → dirty marker, both exit 0; real run #4 header truthful (commit/date numbers all corroborated); expected-file diff header-only |
| WR-03 | warning | **FIXED** | `evidence.py:233-242` zero-step `ok` refusal; `:172-186` OSError recorded as a failed step (iteration 2; code unchanged after) |
| WR-04 | warning | **FIXED** | `evidence.py:191-209` persist-before-echo + BrokenPipeError handling (iteration 2; unchanged) |
| WR-05 | warning | **FIXED** | `evidence.py:276-287` post-normalization digest recompute. Re-tested: existing artifact → `refreshed sha256 …`, report digest == disk file |
| WR-06 | warning | **FIXED** | `_split_carry` carry logic (iteration 2: 120/120 random split layouts exact). Cast code unchanged in iteration 2 |
| IN-01 | info | **FIXED** | README recipe capture-first, defined typescript path, `grep -F` (no `-q`); diff re-read |
| IN-02 | info | **FIXED** | Trailer strip; missing/empty timing over non-empty body → exit 2. Re-tested: empty timing → exit 2; missing timing → exit 2 |
| IN-03 | info | **FIXED** | TESTING step 11 documents the red baseline. Re-ran `scripts/check-egress.sh --strict`: exit 1, `(3 hard hits total; …)` on exactly `gs://bucket`, `api.x.ai`, `grok.com` — matches |
| IN-04 | info | **FIXED** | `step2-config-docs-result-line.log` re-diffed against raw `/tmp/opencode/gate-run3/step2-config-docs.log` lines 319/6528 → identical; scan-clean; five-run numbers marked "reported, not artifact-backed" in KNOWN-ISSUES/TESTING/SUMMARY |
| IN-05 | info | **FIXED** | Text-only/binary-skip limitation documented in `docs/evidence/README.md`; scan behaviour unchanged |
| IN-06 | info | **FIXED** | `errors="replace"` in the `finish` log read (`evidence.py:273`) and `normalize` (`:438`); unchanged since iteration 1 |
| IN-07 | info | **DEFERRED — not open** | `key-leak-check.sh:102-105` still `if grep …; then hits=…; fi` (default mode). Plan 01-05 task 3(b) freezes this mode; ledger line in `KNOWN-ISSUES.md` "API key handling"; disposition row `deferred`. Matches the dispatch rule |
| IN-08 | info | **FIXED** | `evidence.py:244-268`: `finish --result ok` with a deleted artifact → exit 2 naming it, no `report.json`; `--result fail` → exit 0, drop note, `artifacts: []`; existing artifact still succeeds with a refreshed digest (19/19 checks) |
| IN-09 | info | **FIXED** | `evidence.py:380-385`: body non-empty + only `0`/negative byte counts → exit 2 with the "no usable `elapsed bytes` pairs" diagnostic, no cast; empty timing still exit 2; one positive pair converts (1 event, data == body); zero+positive converts (19/19 checks) |
| IN-10 | info | **FIXED** | `test-gate.sh:128-138`: red no-`--log-dir` run keeps the temp dir and prints `gate: failing run kept its logs at <dir>`; the printed step log exists after exit; green run removes it and prints no note; exit 1 preserved; `GATE FAIL` still printed before the note; step-3-only red also keeps logs; explicit `--log-dir` unchanged (23/23 checks) |

## Narrative Findings (AI reviewer)

No open findings. The iteration-2 items are closed with the evidence below.

### WR-02 — closed: the tree-state stamp is computed before the redirect and is truthful [FIXED]

**File:** `scripts/test-gate.sh:272-291`
**Fix verified:** `tree_state` is assigned at lines 278-281, before the group redirection opens at line 282; the group only echoes it at line 285. A bash `{ … } >file` group truncates at open time, so the old inline `git status` saw its own write; a pre-computed value cannot. Harness proof with the real script: committed clean tree → `# tree: clean` (exit 0); untracked marker added → `# tree: dirty (uncommitted changes present at refresh time)` (exit 0). Real proof: run #4's stdout (`gate: wrote …/scripts/test-gate.expected`) plus the file header — commit `c159a3e7…` == `HEAD`, date `16:08:34Z` == mtime (17:08:34.927 CET), dirty marker matches the uncommitted delta, and 308/219/89 match the run's logs re-derived with `awk`. The header comment and `notes.md` describe exactly this mechanism.

### IN-08 — closed: `finish` cannot vouch for a missing artifact [FIXED]

**File:** `scripts/evidence.py:244-268`
**Fix verified:** the missing-artifact check runs for every result before normalization. `--result ok` refuses (exit 2) naming each missing path and writes no report; `fail`/`skipped` drops the entry with an explicit stderr note. The existing-artifact path still normalizes, recomputes the digest, prints the refresh notice, and writes the 8-key report with a digest equal to the file on disk. `docs/evidence/README.md` states the same policy.

### IN-09 — closed: the last silent single-event t=0 cast path is gone [FIXED]

**File:** `scripts/evidence.py:380-385` (guard), `:403-405` (per-entry skip)
**Fix verified:** the guard is `if body and not any(length > 0 for _, length in timing)`, so a timing file that parses but contains only zero/negative byte counts over a non-empty body exits 2 with a diagnostic and writes nothing. The docstring says "no positive-length pairs", matching the code. Regression checks: empty timing → exit 2; missing timing → exit 2; a positive pair (alone or with zeros) → cast with data equal to the normalized body; empty body + empty timing still allowed.

### IN-10 — closed: a red gate no longer deletes its own failure evidence [FIXED]

**File:** `scripts/test-gate.sh:128-138`
**Fix verified:** the EXIT trap captures `status=$?` first, removes the mktemp dir only on 0, and otherwise prints the directory to stderr. Real-script harness: a step-1 red run exits 1, its printed `step1-tests.log` exists afterwards, and the note names that directory; a step-3 (leak-scan) red run keeps `step3-key-leak.log` too; a green run removes the directory and prints no note; `--log-dir` still keeps everything. Exit codes are unchanged.

## Residual observations (non-blocking, not findings)

1. **`.planning/phases/01-truth-baseline/01-05-SUMMARY.md:137` is historically stale.** The plan-execution section still says the expected header is `commit 882ad0de, 2026-11-21T21:37:20Z`; the same file's post-review section (~lines 327-334) correctly records the run #4 re-seed (`c159a3e7`, dirty marker). The evidence-facing `notes.md` is current. Harmless to the gate; optionally update the bullet when committing.
2. **`scripts/test-gate.sh` mtime is 10 s after the expected-file write** (`16:08:44Z` vs `16:08:34Z`). The fix report's deviation note explains a post-run `# shellcheck disable=SC2154` pragma addition; the rewritten content is comment-only (shellcheck passes), run #4 used an explicit `--log-dir` so the trap was not exercised, and the run's stdout/header content matches the current writer. Provenance remains sound; recorded for transparency.
3. **On a red run with no `--log-dir`, the kept-logs stderr note is the last line of a combined stdout+stderr capture.** `GATE FAIL` is still printed before it, exit is 1, and no false-pass line is added; only an automation that keys on the literal final *combined* line would notice. Not a fail-open.
4. **The refresh write precedes step 3 (leak scan)**, pre-existing at HEAD: with `--refresh-expected` and green cargo steps, a leak-scan failure still writes the count file before the gate exits 1, so the header comment "a red run never writes it" is imprecise for that ordering. The three numbers are still derived from green cargo steps and the operator explicitly requested the refresh, so this is a wording nuance, not a count corruption. Suggested follow-up (not required here): move the refresh block after step 3 or narrow the comment.

## Verification log (this review session)

Executed in the main checkout (`/home/cein_orourke/Projects/deepseek-build`, HEAD `c159a3e7`, worktree unchanged by the review):

| Check | Result |
|---|---|
| `git rev-parse HEAD` / `git status --short` / `git diff HEAD --name-only` | HEAD `c159a3e7`; delta = 8 tracked files + untracked evidence/planning; no `crates/` change |
| `git diff HEAD -- scripts/test-gate.expected` | **header-only**: commit+date replaced, `# tree:` added; `results=308`/`binaries=219`/`doctests=89` untouched |
| expected header vs `HEAD`/mtime | commit == HEAD; date 16:08:34Z == mtime; dirty marker truthful |
| run #4 stdout + step logs re-derivation (`awk`) | `GATE PASS`; 308 == 219 + 89 (raw 309); step 1 31,913/0/386; step 2 6,201/0/4; 0 `FAILED` result lines; leak scan PASS |
| `bash -n scripts/test-gate.sh`; `bash -n scripts/key-leak-check.sh`; `shellcheck --shell=sh` both | exit 0, no output |
| `python3 -c 'ast.parse(open("scripts/evidence.py").read())'` | parses |
| WR-02/IN-10 harness (copy of the real script, stubbed `cargo`/`disk-guard`, throwaway git repos) | **23/23 PASS**: clean → `# tree: clean`; dirty → dirty marker; green removes temp logs; red keeps + prints them (step 1 and step 3); red `--refresh-expected` refuses and leaves the file untouched; explicit `--log-dir` persists on green |
| IN-08/IN-09 smoke (real `evidence.py`, temp dirs) | **19/19 PASS**: missing-artifact refusal/drop/no-report, digest recompute, all-non-positive timing refusal, empty/missing timing errors, positive/mixed timing conversion, empty body allowed |
| CR-01 canary re-test (synthetic only; temp dir removed) | error → FAIL 1 (diagnostics only); readable → FAIL 1 + name; clean → PASS 0 |
| 8-key schema + order on 3 `report.json` | 3/3 exact |
| sha256 of 4 recorded artifacts vs disk | 4/4 MATCH |
| casts: header, event counts, monotonicity, concatenated framing/home scan | 774 / 8 / 12 events, all monotonic, no `Script started/done`, no `/home/`, no username |
| `scripts/key-leak-check.sh --scan docs/evidence` | PASS, exit 0 (includes `notes.md`, edited after run #4's scan) |
| `grep -rnE '/home/[a-z]'`, `sk-[A-Za-z0-9]{20,}`, username over `docs/evidence` | rc 1 each, zero hits |
| `scripts/check-egress.sh --strict` | exit 1; `(3 hard hits total; …)` — matches TESTING step 11 |
| step-2 excerpt vs raw run-#3 log lines 319/6528 | identical |
| Plan-01-05 Rust diffs (`882ad0de..HEAD`) | widened waits/sleeps only (100→500 ms, 20/40→100/100/250 ms, 40/50→100/250 ms, 500 ms→5 s, ×3→×10); bare `#[serial]` on both env-global tests; no assertion direction changed |
| Manifest/frozen guard | no `Cargo.toml`/`Cargo.lock`/`deny.toml` change; no `report.json` or hashed artifact modified since iteration 2 (mtimes + sha) |
| IN-07 plan constraint | `01-05-PLAN.md:282-287` task 3(b): "With no arguments the script must behave exactly as it does today, including exit codes and the binary precheck"; ledger in `KNOWN-ISSUES.md`; disposition `deferred` |

NOT RUN here (labelled per verification honesty): the live full/subset gate this iteration (the changed logic is covered by the real-script harness, and run #4's committed raw logs were re-derived; the 13-minute workspace run would also contend for the shared build lock), `cargo fmt`/clippy (no Rust delta; `git diff HEAD -- crates/` empty), `key-leak-check.sh` default mode (needs the release binary and makes a provider request; behaviour pinned by the single-hunk diff and ledgered as IN-07), and the 120-layout cast property test (iteration 2; cast carry code unchanged in iteration 2). Review-side temp files live under `/tmp/opencode`; nothing in the repository was modified. No `external_reviewer_evidence` or structural findings were provided.

## Merge gate

**PASS.** No blocker and no warning remain, so the phase may be committed by the orchestrator (the reviewer does not commit). CR-01 fixed; WR-01…WR-06 fixed (WR-02 now proven with both harness branches and the real run #4 writer); IN-01…IN-06 fixed; IN-07 deferred (plan-constrained by task 3(b), ledgered — not open); IN-08…IN-10 fixed. The four residual observations above are non-blocking and require no fix before commit.

---

_Reviewed: 2026-11-23T16:21:45Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_
