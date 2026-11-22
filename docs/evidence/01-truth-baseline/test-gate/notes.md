# test-gate

Proves TEST-01 and TEST-02: the whole-workspace step exits 0 and completeness
is asserted by counting cargo target blocks against the build graph and the
checked-in expected count, with the `config-docs` step as a named second step.

## Status: DEFERRED (tests skipped by owner, 2026-11-22)

The owner directed on 2026-11-22 that all test and gate runs stop for the
remaining milestone plans. This item is therefore **`result: skipped`**:
`report.json` carries no steps and records no verdict.

`test-gate.log` is the 2026-11-21 full run (run #2) on the author's host, kept
as the last recorded green output. It reports `results=308`, `binaries=219`,
`doctests=89` against `scripts/test-gate.expected`, with 309 raw `test result:`
lines because `xai_tty_utils`'s `stderr_redirect_roundtrip_subprocess` re-runs
its own test binary with inherited stdout. It is **not** a re-confirmation on
this host: the confirmation run (task 3f) was interrupted and never finished.
No cast was captured; the intended cast is a `script -T` recording of the
subset-mode run.

Do not upgrade this item to `ok` without a fresh green full run.

## Re-run (when testing resumes)

```sh
ITEM=docs/evidence/01-truth-baseline/test-gate
# 1. Seed the expected count only from a green run.
scripts/test-gate.sh --refresh-expected
# 2. Confirm with no flags; the numbers must match.
scripts/test-gate.sh
# 3. Record the mechanism as a cast (subset mode keeps it short).
script -qec 'scripts/test-gate.sh --package xai-grok-status-line --package xai-grok-sandbox' \
    -T "$ITEM/run.timing" "$ITEM/run.typescript"
scripts/evidence.py cast --typescript "$ITEM/run.typescript" \
    --timing "$ITEM/run.timing" --out "$ITEM/test-gate.cast"
scripts/key-leak-check.sh --scan docs/evidence
```

On a host without `pkg-config`, set
`OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu OPENSSL_INCLUDE_DIR=/usr/include`
so `openssl-sys` (via `git2`) builds, and `TEST_GATE_MIN_GB=60` if the disk is
smaller than the gate's 150 GB default.
