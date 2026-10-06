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

## `xai-grok-shell` session registry listing lane

The merged shell session listing no longer queries or merges remote registry
records. These tests asserted remote-list lane behavior and were removed with
that lane; local persistence, local search, local visibility, worktree discovery,
and local resume coverage remain.

- `session::merge::tests::retain_matching_cwd_keeps_only_the_requested_directory` — filtered remote registry rows by CWD.
- `session::merge::tests::headless_exclude_drops_local_row_and_its_remote_twin` — asserted filtering of remote twins and remote-only rows.
- `session::merge::tests::headless_only_keeps_only_headless_rows` — asserted that local headless kind metadata was applied to a remote twin.
- `session::merge::tests::unrelated_remote_only_drop_does_not_block_relax` — tested policy behavior for a remote-only row.
- `session::merge::tests::headless_include_keeps_everything` — asserted remote rows survived the headless policy.
- `session::merge::tests::remote_overwrites_local_on_same_id` — asserted remote title/source precedence during deduplication.
- `session::merge::tests::stale_remote_turn_counter_does_not_demote_local_sessions_to_empty` — asserted reconciliation of local and remote message counts.
- `session::merge::tests::remote_overwrite_preserves_local_metadata` — remote/local merge behavior; the local metadata assertions remain as `local_metadata_is_preserved_in_listing`.
- `session::merge::tests::unparseable_last_active_falls_back_to_updated_at` — malformed timestamps were possible only in remote registry records.
- `session::merge::tests::unparseable_timestamps_sort_to_bottom` — exercised remote registry timestamp parsing.
- `session::merge::tests::search_does_not_filter_remote` — asserted that remote results bypassed the local query.
- `session::merge::tests::dedup_across_local_and_remote_mixed` — asserted cross-source deduplication and remote-only inclusion.
- `session::merge::tests::remote_same_cwd_still_included` — asserted remote inclusion in a same-CWD listing.
- `session::merge::tests::remote_different_cwd_same_repo_url_included` — asserted remote inclusion via matching repository URL.
- `session::merge::tests::remote_different_cwd_different_repo_excluded` — asserted remote filtering by repository URL.
- `session::merge::tests::remote_ssh_vs_https_same_repo_matches` — asserted normalized URL matching for remote rows.
- `session::merge::tests::remote_no_repo_url_and_different_cwd_excluded` — asserted remote filtering when repository metadata was absent.
- `session::merge::tests::remote_no_repo_urls_passes_all_remotes_through` — asserted unfiltered remote listing without local repository URLs.
- `session::merge::tests::last_active_at_local_newer_wins` — compared timestamps across the local and remote list lanes.
- `session::merge::tests::last_active_at_remote_newer_wins` — asserted remote timestamp precedence.
- `session::merge::tests::last_active_at_one_side_none_uses_other` — asserted timestamp fallback across local and remote rows.
- `session::merge::tests::remote_session_has_none_metadata_fields` — asserted serialization defaults for remote-only list rows.
- `session::merge::tests::limit_applied_after_merge_not_per_source` — asserted limiting across remote and local listing lanes; local scan truncation remains covered by `limit_applied_after_local_scan`.
- `session::unified_list::tests::relaxed_scan_drops_headless_remote_twin` — asserted that a remote twin could not reintroduce a headless session in the relaxed listing.
