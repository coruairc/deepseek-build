# Code Review: 01-06 — TEST-03 gap closure

**Verdict:** PASS_WITH_NOTES
**Base:** 98c2f1b8 (work was uncommitted at review time)
**Reviewer:** independent reviewer agent (read-only), 2026-11-23
**Scope:** `restore_fetch_tests.rs` death-poll fix, doc corrections (KNOWN-ISSUES.md, TESTING.md, 01-05-SUMMARY.md, docs/evidence/README.md), evidence item `docs/evidence/01-truth-baseline/determinism/`

## Acceptance (plan 01-06 gap closure)

- **Rust change: MET.** `wait_until_dead` (5 s deadline, 20 ms poll) replaces the instantaneous `!pid_alive` at all five process-death sites. Prompt-return bounds (`< 2 s` ~L586, `< 5 s` ~L752/~L791) are asserted before any polling. No assertion direction, timing band, test, or `#[ignore]` changed.
- **Production file: MET.** `restore_fetch.rs` diff against HEAD is empty.
- **Frozen files: MET.** `Cargo.toml`, `Cargo.lock`, `deny.toml`, `scripts/test-gate.expected` unchanged.
- **Evidence: MET** with the caveats in Findings.
- **Doc honesty: MET.** Five-run claim marked superseded; D3 verification entry `status: fail` with the 2-in-35 falsification recorded; no artifact-free determinism claim remains.

## Findings (all LOW/INFO, none blocking)

- **R1 (LOW):** Causal mechanism (async SIGKILL delivery/reaping) is asserted, not shown — no `/proc` state captured at a failure. Poll is bounded, so a real survivor still fails. Suggested follow-up: log `/proc/<pid>/stat` on timeout, or soften wording to "consistent with".
- **R2 (LOW):** Sample power modest — old rate 2/35 (≈5.7%); 35 clean runs ≈13% under unchanged code; combined 55/55 ≈4%. Evidence supports the fix but is not strong on its own.
- **R3 (LOW):** `report.json` date `2026-11-23T18:50:33Z` predates the repetition wall time (19:08:13Z–19:38:44Z) in `notes.md`.
- **R4 (LOW, advisory):** `named=ok` is not recountable from committed artifacts — the driver script and raw cargo logs are not committed (disclosed in `notes.md`).
- **R5 (INFO):** Non-Linux `pid_alive` uses `kill(pid,0)`, which reports unreaped zombies as alive; latent portability gap, out of scope (Linux-only releases).

## Verification performed by the reviewer

- `git status --short` / `git diff HEAD --stat`: only expected files changed; frozen-file diff empty.
- `git show HEAD:…restore_fetch_tests.rs | sha256sum` = `62a2656a…`, matching docs.
- `repetition-35-runs.log`: 35 lines, 35 `rc=0`, 35 `1695 passed; 0 failed`; byte-identical to `post-fix-repetition-35-full-crate-runs.log`.
- `report.json` artifact sha256 values match `sha256sum` for all four hashed artifacts.
- `determinism.cast`: valid asciinema v2 header, 3 event lines, all JSON-parseable.
- `targeted-repetition-20-named-test-runs.log`: 20 `rc=0`, `targeted repetition: 20/20 rc=0`.
- `grep -rn '/home/' docs/evidence/01-truth-baseline/determinism/`: no matches; key-shape grep: no matches; no secrets printed.
- `rustfmt --edition 2024 --check restore_fetch_tests.rs`: exit 0.
- `df -h .`: 974 GB free; no competing cargo/rustc.
- **Reviewer test run:** `flock -w 900 /tmp/team-build.lock env -u DEEPSEEK_API_KEY -u DEEPSEEK_BUILD_API_KEY -u XAI_API_KEY cargo test -p xai-grok-workspace --lib -- restore_fetch --test-threads=4` → `31 passed; 0 failed`, rc=0, including `wait_until_dead_is_bounded_and_deadline_respecting` and both grandchild tests.
- **NOT RUN:** full workspace suite and the 35-run repetition (per review brief); the 2/35 failure was not reproduced on unchanged code.

## Disposition

Commit as-is. R1/R3/R4 are wording/metadata caveats already disclosed in the evidence item; R1's `/proc` logging suggestion and R2's sample-power note are carried as advisory follow-ups for any future flake claim (they do not reopen TEST-03, whose artifact gates all recount green).
