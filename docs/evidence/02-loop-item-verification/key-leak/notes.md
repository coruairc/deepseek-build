# key-leak

Proves PROOF-03: the product writes no key material, no key material is
committed in the evidence tree or anywhere this phase wrote, and the key used
for the Phase-2 live runs has been revoked — that last part recorded as the
author's dated attestation, because no tool can verify it.

## What this proves

Three separate checks, each a recorded step with its real exit status:

1. **The product writes no key material.** `scripts/key-leak-check.sh`'s
   default mode ran the release binary (`target/release/deepseek-build`,
   `0.0.1-rc5`) with the script's own fake key `sk-FAKE-LEAKCANARY-…` in an
   isolated `HOME`, with debug logging enabled, against the real
   `https://api.deepseek.com` endpoint. The run exited 1 (a 401 is expected
   with a fake key) and the script's own verdict was
   `PASS  fake key absent from 52 saved file(s) and captured output`.
   The key was never typed into the captured command and the capture was not
   run under `set -x`; the script writes the fake key only into the child
   environment.
2. **The provider path really ran.** The default-mode script redirects the
   binary's own stdout/stderr into a temp directory and deletes it in its
   `EXIT` trap, so its typescript carries only `run exit: <n>` — and that
   line says `401/other error is expected` whether the provider answered or
   the host had no network. The step therefore also records a
   same-conditions probe: the same release binary, the same committed fake
   key and the same endpoint with an isolated `HOME`, whose debug log shows
   `sampler transport prewarmed origin=https://api.deepseek.com status=401
   Unauthorized elapsed_ms=513` and whose stderr carries
   `Unauthorized (401) from https://api.deepseek.com/chat/completions`. The
   provider answered; the canary exercised both the local file path and the
   network path.
3. **No key material is committed.** `scripts/key-leak-check.sh --scan
   docs/evidence` and `--scan docs .planning/phases/02-loop-item-verification`
   both exited 0. `--scan` fails closed on a grep error (`IN-07` covers the
   default mode's fail-open case, which is why the committed-artifact claim
   rests on `--scan`). The item directory was scanned again after this note
   was written and hashed, so the note is itself covered.

What this item does **not** prove: that the key was revoked. That is the
author's action on platform.deepseek.com, recorded below as an attestation.

## Steps

| Step | Check |
|---|---|
| `canary: product wrote no key material` | the canary typescript carries the script's `PASS  fake key absent from … saved file(s) and captured output` line |
| `canary: the provider path really ran` | a same-conditions fake-key probe observed the provider's 401 (`api.deepseek.com`, `status=401 Unauthorized`) |
| `scan: committed evidence tree is clean` | `scripts/key-leak-check.sh --scan docs/evidence` → `PASS  no key-shaped string`, exit 0 |
| `scan: every path this phase committed is clean` | `--scan docs .planning/phases/02-loop-item-verification` → `PASS`, exit 0 |
| `baseline: whole-tree key-shape grep measured` | the tracked-file `sk-`-shape count, recorded as a measurement (below) |
| `attestation: dated revocation sentence present` | a literal `grep -F` for the dated revocation prefix over this note |
| `scan: item directory clean after the note` | `--scan` over this item directory after the note and its hash |

`key-leak.cast` is the `script -T` capture of the canary run (asciinema v2,
normalized). Raw `run.typescript` / `run.timing` intermediates were deleted
after the steps that read them were recorded.

## The whole-tree baseline

Measured with the plan's command over the tracked files at HEAD `eef4714e`
(the plan was authored against `1efb05ac`; the numbers are unchanged):

```
matching-files: 2
matching-lines: 4
tracked-files:   4379
```

The two hit files are
`crates/codegen/xai-grok-secrets/src/sanitizer.rs` (3 lines) and
`crates/codegen/xai-grok-telemetry/src/redact_common.rs` (1 line). All four
matches are inside `#[cfg(test)]` modules of production redaction code:
deliberate canaries and over-redaction negative controls, not fixtures, not
scripts, and not key material.

**No equality gate is placed on this number.** The count is a property of the
tree at run time; pinning it with `-eq` would force the gate to be edited
whenever a legitimate test string moved. The gate over committed artifacts is
the two `--scan` steps plus the canary's own file count, and `--scan` fails
closed (`IN-07` is the default mode's known fail-open case, not `--scan`'s).

## Revocation (author attestation)

Revoked: 2026-11-25 (author confirmation recorded 2026-11-25T23:11:13Z; the exact platform-action time was not recorded) — the author rotated/revoked the DEEPSEEK_API_KEY used for the Phase-2 live runs on platform.deepseek.com.
This is the author's confirmation that the action was performed; no API exists to check a revocation, so it is recorded rather than verified.

The step `attestation: dated revocation sentence present` proves only that
this sentence is present in this file. It does not, and cannot, prove the
revocation itself. No account identifier, key fragment or key length is
recorded here.

## Re-run

The default-mode canary (needs the release binary and network to the
provider; it uses the script's own fake key, never a real one):

```sh
unset DEEPSEEK_API_KEY DEEPSEEK_BUILD_API_KEY DEEPSEEK_BASE_URL
scripts/key-leak-check.sh
# expect: run exit: 1 (401/other error is expected with a fake key)
#         PASS  fake key absent from N saved file(s) and captured output
```

The committed-artifact scans (offline, no key):

```sh
scripts/key-leak-check.sh --scan docs/evidence
scripts/key-leak-check.sh --scan docs .planning/phases/02-loop-item-verification
# expect: PASS  no key-shaped string in: <paths>, exit 0
```

The measured baseline (offline; read it, do not gate on it):

```sh
sh -c 'echo "matching-files:"; git ls-files -z | xargs -0 grep -lIE "sk-[A-Za-z0-9]{20,}" 2>/dev/null | wc -l; \
       echo "matching-lines:"; git ls-files -z | xargs -0 grep -oIE "sk-[A-Za-z0-9]{20,}" 2>/dev/null | wc -l; \
       echo "tracked-files:"; git ls-files | wc -l'
```

To regenerate the cast, capture the canary first and convert it:

```sh
ITEM=docs/evidence/02-loop-item-verification/key-leak
script -qec 'scripts/key-leak-check.sh' -T "$ITEM/run.timing" "$ITEM/run.typescript"
scripts/evidence.py cast --typescript "$ITEM/run.typescript" --timing "$ITEM/run.timing" \
    --out "$ITEM/key-leak.cast"
rm -f "$ITEM/run.timing" "$ITEM/run.typescript"
```

The revocation half is not re-runnable: it is a human action on
platform.deepseek.com and stays an attestation.
