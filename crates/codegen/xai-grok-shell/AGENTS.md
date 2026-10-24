# xai-grok-shell contributor notes

## Config reference

The user-facing configuration reference lives at
`crates/codegen/xai-grok-pager/docs/user-guide/26-config-reference.md`.

Edit it; do not regenerate it. When a key is added or removed from the live
registries (`FEATURES`, `UNMIRRORED_BOOLEAN_FEATURES`, `KNOWN_MCP_SERVER_FIELDS`), update the page's rows by hand; the `config_docs` tests
(check `src/config_docs/mod.rs`, enabled with `--features config-docs`) verify
that the page and the registries stay in sync.

Keys whose flows were removed (managed-config auth tables, remote/ campaign
settings) are deliberately left out of the page; add them to
`UNDOCUMENTED_REMOVED_FLOW_KEYS` instead of documenting them.
