# Dropped tests

This ledger records tests removed because the behavior they asserted was
intentionally removed from `deepseek-build`. Tests for retained local behavior
should be repaired or kept; this is not a general test-pruning list.

## `xai-grok-telemetry`

- Removed `tests/manual_auth_emit.rs::manual_auth_posts_to_events_endpoint_as_grok_shell_manual_auth`.
  It required `log_event(ManualAuth)` to POST to a product analytics endpoint.
  Product-event network emission has been removed; keeping a test that requires
  that outbound behavior would contradict the egress decision. The telemetry
  crate's local behavior and remaining unit/integration tests are retained.
