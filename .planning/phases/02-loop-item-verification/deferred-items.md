# Deferred items — 02-loop-item-verification

## Unrelated gate failures (02-04)

`scripts/test-gate.sh --refresh-expected` exited 1 and wrote nothing. Step 2 (`config-docs`) passed. Step 1 failed only in `xai-grok-agent` lib tests that this plan does not touch:

- `builder::tests::hiding_task_drops_lifecycle_tools_when_background_shell_is_off` (`builder.rs:2520`) — `spawn_subagent` and the background-shell companions remain offered when background shell is off.
- `builder::tests::task_tool_tracks_whether_general_purpose_is_spawnable` (`builder.rs:2456`) — disabling general-purpose does not hide `spawn_subagent`.

Reproduced with `cargo test -p xai-grok-agent --lib` after the gate. Not caused by the permission-warning or denial-copy edits. The gate still derived `results=309 binaries=220 doctests=89` (`results == binaries + doctests`) but refused to rewrite `scripts/test-gate.expected` because the run was not green. Do not hand-edit that file.

## Pre-existing clippy failure in a pty-harness bench (02-05)

`cargo clippy -p xai-grok-pager-pty-harness --all-targets` fails to compile
`benches/paste_latency.rs` with five `no method named 'as_str' found for enum 'Surface'` errors.
The bench is unmodified by 02-05 (and by 02-04); `cargo clippy -p xai-grok-pager-bin -p
xai-grok-pager-pty-harness --tests` finishes with warnings only and 0 errors. Fixing the bench is
out of scope for a resume-fidelity plan; it must be repaired before a whole-crate `--all-targets`
gate can be green.

## Pre-existing `cargo fmt` violation, fixed in 02-05

`crates/codegen/xai-grok-pager-pty-harness/tests/pty_e2e/permission_prompt_outcomes.rs` was
committed unformatted at `30388a51` (02-04) and failed `cargo fmt --all --check`. 02-05 applied
rustfmt only (line joining, no behavior change) so the plan's own fmt gate passes.
