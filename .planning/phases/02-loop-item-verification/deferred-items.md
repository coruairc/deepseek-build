# Deferred items — 02-loop-item-verification

## Unrelated gate failures (02-04)

`scripts/test-gate.sh --refresh-expected` exited 1 and wrote nothing. Step 2 (`config-docs`) passed. Step 1 failed only in `xai-grok-agent` lib tests that this plan does not touch:

- `builder::tests::hiding_task_drops_lifecycle_tools_when_background_shell_is_off` (`builder.rs:2520`) — `spawn_subagent` and the background-shell companions remain offered when background shell is off.
- `builder::tests::task_tool_tracks_whether_general_purpose_is_spawnable` (`builder.rs:2456`) — disabling general-purpose does not hide `spawn_subagent`.

Reproduced with `cargo test -p xai-grok-agent --lib` after the gate. Not caused by the permission-warning or denial-copy edits. The gate still derived `results=309 binaries=220 doctests=89` (`results == binaries + doctests`) but refused to rewrite `scripts/test-gate.expected` because the run was not green. Do not hand-edit that file.
