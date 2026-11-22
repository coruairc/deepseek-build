# installer

Proves REL-06: `install.sh` installs a pinned `VERSION=` from the real release
and prints that version, and the unpinned `latest` path's actual behaviour is
established from the real URL rather than the fake-tree harness.

## Status: BLOCKED — DEFERRED (tests skipped by owner, 2026-11-22; no network run)

No install was performed. This item is `result: skipped`.

## What is known without a run

- `install.sh` resolves `latest` by fetching
  `https://github.com/coruairc/deepseek-build/releases/latest/download/SHA256SUMS`.
  GitHub resolves `latest` to the most recent **non-prerelease** release. Only
  pre-releases exist (`v0.0.1-rc5` newest), so the unpinned path has no target
  today; `README.md` documents the pinned form and this limitation.
- The pinned form is
  `VERSION=v0.0.1-rc5 sh install.sh` (or the tag's own
  `/releases/download/<tag>/install.sh` URL).
- The fake-tree harness (`scripts/test-install.sh`, 39/39) is not evidence for
  the real URL; it only proves the script's own logic.

## Re-run

```sh
ITEM=docs/evidence/04-distribution-proof/installer
scripts/evidence.py begin --item-dir "$ITEM" --item installer \
    --requirements REL-06 --command "install.sh pinned + unpinned against the real release"

# Pinned: installs v0.0.1-rc5 and prints that version.
script -qec 'VERSION=v0.0.1-rc5 INSTALL_DIR="$HOME/.local/bin" sh install.sh && deepseek-build --version' \
    -T "$ITEM/pinned.timing" "$ITEM/pinned.typescript"

# Unpinned: establish what the real latest URL does today (expect no target).
script -qec 'sh install.sh' -T "$ITEM/latest.timing" "$ITEM/latest.typescript"

scripts/evidence.py step --item-dir "$ITEM" --name "pinned install reports the pinned version" \
    -- sh -c "grep -F 'deepseek-build 0.0.1-rc5' '$ITEM/pinned.typescript'"
scripts/evidence.py step --item-dir "$ITEM" --name "unpinned latest behaviour recorded" \
    -- sh -c "grep -Ei 'no asset|404|latest' '$ITEM/latest.typescript'"

scripts/evidence.py cast --typescript "$ITEM/pinned.typescript" \
    --timing "$ITEM/pinned.timing" --out "$ITEM/installer.cast"
scripts/evidence.py artifact --item-dir "$ITEM" --path "$ITEM/installer.cast"
scripts/evidence.py finish --item-dir "$ITEM" --result ok
```

Record the unpinned result as observed, not as expected: if `latest` points at a
prerelease or at nothing, that is the finding.
