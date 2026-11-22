# key-leak

Proves PROOF-03: no key material reaches logs, saved sessions, error output or
committed evidence, and the throwaway key is revoked after the run.

## Status: DEFERRED (tests skipped by owner, 2026-11-22)

No run was performed. This item is `result: skipped`.

## What the check must show

1. `scripts/key-leak-check.sh` — runs the binary with a fake canary key, then
   scans every saved file under an isolated `HOME` plus captured stdout/stderr;
   prints file names only, never matched lines.
2. `scripts/key-leak-check.sh --scan docs/evidence` — the static scan the Phase-1
   gate also runs; no key shape in committed artifacts.
3. The real throwaway key appears nowhere: `git grep` for its prefix is not
   enough (the key must never be written, so the scan is over the tree and the
   evidence, not a literal search).

## Re-run

```sh
ITEM=docs/evidence/02-loop-item-verification/key-leak
scripts/evidence.py begin --item-dir "$ITEM" --item key-leak \
    --requirements PROOF-03 \
    --command "scripts/key-leak-check.sh; scripts/key-leak-check.sh --scan <tree paths>"

script -qec 'scripts/key-leak-check.sh' -T "$ITEM/run.timing" "$ITEM/run.typescript"

scripts/evidence.py step --item-dir "$ITEM" --name "live fake-key run: key absent" \
    -- sh -c "grep -qF 'PASS  fake key absent' '$ITEM/run.typescript'"
scripts/evidence.py step --item-dir "$ITEM" --name "committed evidence: no key shape" \
    -- scripts/key-leak-check.sh --scan docs/evidence

scripts/evidence.py cast --typescript "$ITEM/run.typescript" \
    --timing "$ITEM/run.timing" --out "$ITEM/key-leak.cast"
scripts/evidence.py artifact --item-dir "$ITEM" --path "$ITEM/key-leak.cast"
scripts/evidence.py finish --item-dir "$ITEM" --result ok
```

Revoke the throwaway key in the provider console afterwards and note the
revocation in the report's `command` or `notes.md` (never the key itself).
