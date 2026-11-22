# live-smoke

Proves PROOF-01: a real model-driven session with a throwaway key runs a shell
command, edits a file and fetches a page, with the transcript committed.

## Status: DEFERRED (tests skipped by owner, 2026-11-22)

No run was performed. This item is `result: skipped`; no cast exists. It stays
skipped until a throwaway key is provided and testing resumes.

## Preconditions

- A throwaway `DEEPSEEK_API_KEY` (revoked after the run) or
  `DEEPSEEK_BUILD_API_KEY`.
- A release or debug binary containing this tree:
  `cargo build --release -p xai-grok-pager-bin` → `target/release/deepseek-build`.
- `script` (util-linux), `python3`, `jq`.

## Re-run

```sh
ITEM=docs/evidence/02-loop-item-verification/live-smoke
# 1. Record the item's metadata (date/host/command).
scripts/evidence.py begin --item-dir "$ITEM" --item live-smoke \
    --requirements PROOF-01 \
    --command "scripts/smoke-test.sh (real session, throwaway key), captured with script -T"

# 2. Capture the pinned smoke recipe (never echoes the key).
script -qec 'scripts/smoke-test.sh' -T "$ITEM/run.timing" "$ITEM/run.typescript"

# 3. One step per assertion: shell ran, file edit applied, web_fetch returned.
#    The smoke script prints PASS/FAIL per check; grep the captured transcript
#    for the checks that cover the three loop items (see notes in Phase 1's
#    ipv6-report for the pattern).
scripts/evidence.py step --item-dir "$ITEM" --name "shell command executed" \
    -- sh -c "grep -qF 'PASS  headless chat returns a reply' '$ITEM/run.typescript'"
# …edit and web_fetch checks likewise, each a real grep over the transcript.

# 4. Convert, record the artifact, finish.
scripts/evidence.py cast --typescript "$ITEM/run.typescript" \
    --timing "$ITEM/run.timing" --out "$ITEM/live-smoke.cast"
scripts/evidence.py artifact --item-dir "$ITEM" --path "$ITEM/live-smoke.cast"
scripts/evidence.py finish --item-dir "$ITEM" --result ok
scripts/key-leak-check.sh --scan docs/evidence

# 5. Revoke the throwaway key in the provider console.
```

If `scripts/smoke-test.sh` does not exercise a loop item you need (a specific
shell command, an explicit edit, a chosen `web_fetch` URL), run the equivalent
`deepseek-build -p …` turn under the same `script -T` capture and record its
exit status. Do not edit the transcript by hand.
