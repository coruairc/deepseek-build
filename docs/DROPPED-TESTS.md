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

## `xai-grok-shell` agent-side registry writers

The agent no longer registers, updates, finalizes, restores, or downloads
session replicas over the remote session registry. Local git-head persistence,
local session persistence/list/search/resume, and local worktree discovery/resume
remain. These tests asserted removed remote-registry behavior.

- `agent::session_registry_client::tests::immediate_turn_update_omits_restorable_field` — asserted the wire shape of a remote `UpdateRequest`.
- `agent::session_registry_client::tests::restorable_turn_update_omits_last_turn_and_head_fields` — asserted the remote update wire shape.
- `agent::session_registry_client::tests::summary_update_omits_all_turn_fields` — asserted the remote update wire shape.
- `agent::session_registry_client::tests::empty_summary_is_sent_not_omitted` — asserted remote replica title unpinning.
- `agent::session_registry_client::tests::register_request_serializes_device_id_as_camel_case` — asserted the remote register wire shape.
- `agent::session_registry_client::tests::register_request_serializes_empty_device_id_as_present` — asserted the remote register wire shape.
- `agent::session_registry_client::tests::register_request_omits_device_id_when_none` — asserted the remote register wire shape.
- `agent::session_registry_client::tests::session_record_without_restorable_turn_deserializes_as_none` — asserted `SessionRecord` deserialization for the remote registry.
- `agent::session_registry_client::tests::session_record_with_restorable_turn_deserializes_correctly` — asserted `SessionRecord` deserialization for the remote registry.
- `agent::session_registry_client::tests::session_registry_client_uses_active_auth_for_each_request` — asserted remote client auth token refresh.
- `agent::session_registry_client::tests::session_record_allows_last_turn_ahead_of_restorable` — asserted remote `SessionRecord` field semantics.
- `agent::mvp_agent::turn_end::tests::dropped_claim_keeps_later_turn_ordered` — exercised `RegistryWriteOrder` turn-end serialization, which is removed.
- `session::worktree::tests::remote_worktree_codebase_follows_request_then_default` — exercised `remote_worktree_restores_codebase`, removed with the remote worktree restore path.
- `util::config::mcp::tests::session_registry_local_override_precedence` — asserted the removed `[cli] session_registry` / `GROK_SESSION_REGISTRY` gate.

## `xai-grok-workspace` computer-hub / MCP bridge removal

The workspace no longer hosts the computer hub, its `crate::hub`/`crate::hub_server`/`crate::mcp` bridge, or the hub session-bind resolver. Tests that asserted those removed surfaces were deleted; retained local behavior (toolset resolution, rebind/update, fork, path virtualization via the local harness, gate classification, permission payload/outcome decoding, host-kind catalog selection) stays covered.

- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::build_session_routed_handlers_covers_finalized_toolset` — tested `build_session_routed_handlers`/`WorkspaceRpcHandler`, removed with the hub server.
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::build_session_routed_handlers_preserves_renamed_active_message_kind` — tested `build_session_routed_handlers`/`WorkspaceRpcHandler`, removed with the hub server.
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::build_session_routed_handlers_skips_invalid_client_name_without_panic` — tested `build_session_routed_handlers`/`WorkspaceRpcHandler`, removed with the hub server.
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::resolver_advertises_tool_absent_from_connect_catalog` — tested `build_session_routed_handlers`/`WorkspaceRpcHandler`, removed with the hub server.
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::workspace_shared_auth_provider_uses_workspace_config` — asserted `WorkspaceShared::auth_provider`, removed with the hub auth stack.
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::bind_advertises_configured_mcp_per_session` — exercised the removed computer-hub/MCP bridge (`crate::mcp`/`crate::hub_server`, `converge_session_mcp`, `FakeHubRegistry`).
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::a_stopped_server_stays_configured_and_returns_at_the_next_bind` — exercised the removed computer-hub/MCP bridge (`crate::mcp`/`crate::hub_server`, `converge_session_mcp`, `FakeHubRegistry`).
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::a_stop_during_a_bind_start_ends_the_client_the_start_opened` — exercised the removed computer-hub/MCP bridge (`crate::mcp`/`crate::hub_server`, `converge_session_mcp`, `FakeHubRegistry`).
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::a_server_whose_transport_closed_is_restarted_at_the_next_convergence` — exercised the removed computer-hub/MCP bridge (`crate::mcp`/`crate::hub_server`, `converge_session_mcp`, `FakeHubRegistry`).
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::convergence_gates_only_the_server_the_bind_config_names` — exercised the removed computer-hub/MCP bridge (`crate::mcp`/`crate::hub_server`, `converge_session_mcp`, `FakeHubRegistry`).
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::stop_servers_ends_the_client_before_the_first_hub_unregister` — exercised the removed computer-hub/MCP bridge (`crate::mcp`/`crate::hub_server`, `converge_session_mcp`, `FakeHubRegistry`).
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::stop_mcp_server_ends_the_client_without_a_hub` — exercised the removed computer-hub/MCP bridge (`crate::mcp`/`crate::hub_server`, `converge_session_mcp`, `FakeHubRegistry`).
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::the_stop_ends_the_client_while_the_push_waits_on_the_hub` — exercised the removed computer-hub/MCP bridge (`crate::mcp`/`crate::hub_server`, `converge_session_mcp`, `FakeHubRegistry`).
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::bind_with_no_configured_mcp_still_joins_the_configured_set` — drove the removed hub session-bind resolver (`session_bind_resolver`, `handler_names`) instead of a live production API.
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::bind_records_which_tools_each_mcp_server_contributed` — exercised the removed computer-hub/MCP bridge (`crate::mcp`/`crate::hub_server`, `converge_session_mcp`, `FakeHubRegistry`).
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::a_shadowed_mcp_tool_is_not_attributed_to_its_server` — exercised the removed computer-hub/MCP bridge (`crate::mcp`/`crate::hub_server`, `converge_session_mcp`, `FakeHubRegistry`).
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::an_rpc_only_bind_never_joins_the_configured_set` — exercised the removed computer-hub/MCP bridge (`crate::mcp`/`crate::hub_server`, `converge_session_mcp`, `FakeHubRegistry`).
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::a_reload_without_a_hub_stages_the_config_and_reports_it` — exercised the removed computer-hub/MCP bridge (`crate::mcp`/`crate::hub_server`, `converge_session_mcp`, `FakeHubRegistry`).
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::zero_tool_bind_mcp_is_reused_on_soft_rebind` — exercised the removed computer-hub/MCP bridge (`crate::mcp`/`crate::hub_server`, `converge_session_mcp`, `FakeHubRegistry`).
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::a_failed_server_is_retried_by_the_next_convergence` — exercised the removed computer-hub/MCP bridge (`crate::mcp`/`crate::hub_server`, `converge_session_mcp`, `FakeHubRegistry`).
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::duplicate_bind_mcp_tool_ids_are_rejected` — exercised the removed computer-hub/MCP bridge (`crate::mcp`/`crate::hub_server`, `converge_session_mcp`, `FakeHubRegistry`).
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::bind_mcp_cannot_shadow_native_tool` — exercised the removed computer-hub/MCP bridge (`crate::mcp`/`crate::hub_server`, `converge_session_mcp`, `FakeHubRegistry`).
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::a_teardown_mid_registration_unregisters_the_orphaned_tools` — exercised the removed computer-hub/MCP bridge (`crate::mcp`/`crate::hub_server`, `converge_session_mcp`, `FakeHubRegistry`).
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::advertised_mcp_tools_are_capped_per_session` — exercised the removed computer-hub/MCP bridge (`crate::mcp`/`crate::hub_server`, `converge_session_mcp`, `FakeHubRegistry`).
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::a_soft_rebind_does_not_queue_behind_an_in_flight_convergence` — drove the removed hub session-bind resolver (`session_bind_resolver`, `handler_names`) instead of a live production API.
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::teardown_does_not_queue_behind_a_slow_mcp_start` — exercised the removed computer-hub/MCP bridge (`crate::mcp`/`crate::hub_server`, `converge_session_mcp`, `FakeHubRegistry`).
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::an_ended_session_rebind_recovers_the_configured_set` — exercised the removed computer-hub/MCP bridge (`crate::mcp`/`crate::hub_server`, `converge_session_mcp`, `FakeHubRegistry`).
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::a_soft_rebind_install_preserves_dynamic_mcp_registrations` — exercised the removed computer-hub/MCP bridge (`crate::mcp`/`crate::hub_server`, `converge_session_mcp`, `FakeHubRegistry`).
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::a_teardown_during_bind_resolution_refuses_stale_enrolment` — exercised the removed computer-hub/MCP bridge (`crate::mcp`/`crate::hub_server`, `converge_session_mcp`, `FakeHubRegistry`).
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::a_stale_drive_commit_cannot_enter_a_revived_life` — exercised the removed computer-hub/MCP bridge (`crate::mcp`/`crate::hub_server`, `converge_session_mcp`, `FakeHubRegistry`).
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::a_stale_install_cannot_cross_into_a_revived_life` — exercised the removed computer-hub/MCP bridge (`crate::mcp`/`crate::hub_server`, `converge_session_mcp`, `FakeHubRegistry`).
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::a_bind_mcp_server_gets_no_agent_header_unless_first_party` — exercised the removed computer-hub/MCP bridge (`crate::mcp`/`crate::hub_server`, `converge_session_mcp`, `FakeHubRegistry`).
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::a_first_party_server_keeps_an_id_a_third_party_sibling_also_offers` — exercised the removed computer-hub/MCP bridge (`crate::mcp`/`crate::hub_server`, `converge_session_mcp`, `FakeHubRegistry`).
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::a_first_party_server_takes_over_an_id_a_running_third_party_sibling_registered` — exercised the removed computer-hub/MCP bridge (`crate::mcp`/`crate::hub_server`, `converge_session_mcp`, `FakeHubRegistry`).
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::a_soft_rebind_keeps_registration_tags_matching_the_life` — exercised the removed computer-hub/MCP bridge (`crate::mcp`/`crate::hub_server`, `converge_session_mcp`, `FakeHubRegistry`).
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::a_bind_accepted_after_an_unbind_invalidates_its_deferred_teardown` — drove the removed hub session-bind resolver (`session_bind_resolver`, `handler_names`) instead of a live production API.
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::a_bind_accepted_after_a_session_end_invalidates_its_teardown` — drove the removed hub session-bind resolver (`session_bind_resolver`, `handler_names`) instead of a live production API.
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::a_torn_unbind_snapshot_cannot_close_an_accepted_binds_mcp` — drove the removed hub session-bind resolver (`session_bind_resolver`, `handler_names`) instead of a live production API.
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::a_losing_bind_still_contributes_its_native_ids` — drove the removed hub session-bind resolver (`session_bind_resolver`, `handler_names`) instead of a live production API.
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::a_reclaim_after_a_losing_binds_union_drops_new_collisions` — exercised the removed computer-hub/MCP bridge (`crate::mcp`/`crate::hub_server`, `converge_session_mcp`, `FakeHubRegistry`).
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::a_stale_drive_cannot_stamp_a_revived_lifes_init_progress` — exercised the removed computer-hub/MCP bridge (`crate::mcp`/`crate::hub_server`, `converge_session_mcp`, `FakeHubRegistry`).
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::a_mid_bind_teardown_cannot_fail_a_bind_without_mcp_config` — exercised the removed computer-hub/MCP bridge (`crate::mcp`/`crate::hub_server`, `converge_session_mcp`, `FakeHubRegistry`).
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::an_empty_configured_set_never_fails_a_bind` — exercised the removed computer-hub/MCP bridge (`crate::mcp`/`crate::hub_server`, `converge_session_mcp`, `FakeHubRegistry`).
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::a_mid_bind_teardown_cannot_fail_a_bind_with_configured_mcp` — exercised the removed computer-hub/MCP bridge (`crate::mcp`/`crate::hub_server`, `converge_session_mcp`, `FakeHubRegistry`).
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::a_hung_mcp_server_cannot_push_a_bind_past_the_ack_budget` — drove the removed hub session-bind resolver (`session_bind_resolver`, `handler_names`) instead of a live production API.
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::the_configure_path_caps_advertised_tools` — exercised the removed computer-hub/MCP bridge (`crate::mcp`/`crate::hub_server`, `converge_session_mcp`, `FakeHubRegistry`).
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::a_rebind_reopens_for_the_client_driven_configure_path` — exercised the removed computer-hub/MCP bridge (`crate::mcp`/`crate::hub_server`, `converge_session_mcp`, `FakeHubRegistry`).
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::a_stale_teardown_unregister_cannot_remove_a_revived_lifes_tool` — exercised the removed computer-hub/MCP bridge (`crate::mcp`/`crate::hub_server`, `converge_session_mcp`, `FakeHubRegistry`).
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::a_reclaim_unregister_cannot_strip_a_native_tool` — exercised the removed computer-hub/MCP bridge (`crate::mcp`/`crate::hub_server`, `converge_session_mcp`, `FakeHubRegistry`).
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::a_drop_racing_a_revive_bind_orphans_no_life` — drove the removed hub session-bind resolver (`session_bind_resolver`, `handler_names`) instead of a live production API.
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::a_drives_token_belongs_to_the_life_it_checked` — exercised the removed computer-hub/MCP bridge (`crate::mcp`/`crate::hub_server`, `converge_session_mcp`, `FakeHubRegistry`).
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::a_stale_unbind_teardown_skips_a_newer_life` — drove the removed hub session-bind resolver (`session_bind_resolver`, `handler_names`) instead of a live production API.
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::reload_collision_unregisters_the_id_the_surviving_server_lost` — exercised the removed computer-hub/MCP bridge (`crate::mcp`/`crate::hub_server`, `converge_session_mcp`, `FakeHubRegistry`).
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::bind_mcp_discovery_is_concurrent_and_bounded` — exercised the removed computer-hub/MCP bridge (`crate::mcp`/`crate::hub_server`, `converge_session_mcp`, `FakeHubRegistry`).
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::strict_bind_without_explicit_toolset_fails_closed_end_to_end` — drove the removed hub session-bind resolver (`session_bind_resolver`, `handler_names`) instead of a live production API.
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::strict_rpc_only_bind_fails_closed_with_resolve_error_end_to_end` — drove the removed hub session-bind resolver (`session_bind_resolver`, `handler_names`) instead of a live production API.
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::strict_bind_with_explicit_toolset_serves_it_end_to_end` — drove the removed hub session-bind resolver (`session_bind_resolver`, `handler_names`) instead of a live production API.
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::lax_bind_without_metadata_uses_default_catalog_end_to_end` — drove the removed hub session-bind resolver (`session_bind_resolver`, `handler_names`) instead of a live production API.
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::rejected_rebind_config_keeps_resolve_error_end_to_end` — drove the removed hub session-bind resolver (`session_bind_resolver`, `handler_names`) instead of a live production API.
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::explicit_empty_toolset_rebind_never_swaps_session_tools` — drove the removed hub session-bind resolver (`session_bind_resolver`, `handler_names`) instead of a live production API.
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::strict_rebind_with_corrected_toolset_heals_end_to_end` — drove the removed hub session-bind resolver (`session_bind_resolver`, `handler_names`) instead of a live production API.
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::owner_toolset_survives_concurrent_consumer_shaped_rebinds` — drove the removed hub session-bind resolver (`session_bind_resolver`, `handler_names`) instead of a live production API.
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::restored_server_first_bind_ordering_decides_capability_and_toolset` — drove the removed hub session-bind resolver (`session_bind_resolver`, `handler_names`) instead of a live production API.
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::bind_flow_rebinds_keep_backend_and_task_alive_end_to_end` — drove the removed hub session-bind resolver (`session_bind_resolver`, `handler_names`) instead of a live production API.
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::bind_session_root_sets_mapping_and_real_cwd` — drove the removed hub session-bind resolver (`session_bind_resolver`) to exercise bind-time path virtualization/mount behavior.
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::bind_session_root_scopes_shell_env_to_the_session` — drove the removed hub session-bind resolver (`session_bind_resolver`) to exercise bind-time path virtualization/mount behavior.
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::bind_without_session_root_leaves_shell_env_alone` — drove the removed hub session-bind resolver (`session_bind_resolver`) to exercise bind-time path virtualization/mount behavior.
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::rebind_session_root_rewrites_existing_cwd` — drove the removed hub session-bind resolver (`session_bind_resolver`) to exercise bind-time path virtualization/mount behavior.
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::rebind_session_root_refuses_recreate_while_calls_are_in_flight` — drove the removed hub session-bind resolver (`session_bind_resolver`) to exercise bind-time path virtualization/mount behavior.
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::rebind_session_root_refuses_recreate_while_turn_is_active` — drove the removed hub session-bind resolver (`session_bind_resolver`) to exercise bind-time path virtualization/mount behavior.
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::bind_session_root_rewrites_artifacts_cwd` — drove the removed hub session-bind resolver (`session_bind_resolver`) to exercise bind-time path virtualization/mount behavior.
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::bind_without_session_root_does_not_virtualize` — drove the removed hub session-bind resolver (`session_bind_resolver`) to exercise bind-time path virtualization/mount behavior.
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::malformed_session_root_does_not_virtualize` — drove the removed hub session-bind resolver (`session_bind_resolver`) to exercise bind-time path virtualization/mount behavior.
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::bind_invokes_mount_hook_unbind_does_not_unmount` — exercised the removed computer-hub/MCP bridge (`crate::mcp`/`crate::hub_server`, `converge_session_mcp`, `FakeHubRegistry`).
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::bind_mount_error_fails_bind` — drove the removed hub session-bind resolver (`session_bind_resolver`) to exercise bind-time path virtualization/mount behavior.
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::rebind_mount_error_fails_bind_and_drops_leftover` — drove the removed hub session-bind resolver (`session_bind_resolver`) to exercise bind-time path virtualization/mount behavior.
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::bind_probe_hit_skips_mount` — drove the removed hub session-bind resolver (`session_bind_resolver`) to exercise bind-time path virtualization/mount behavior.
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::bind_without_session_root_skips_mount_hook` — drove the removed hub session-bind resolver (`session_bind_resolver`) to exercise bind-time path virtualization/mount behavior.
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::mcp_server_outcome_reports_settled_starts_and_never_waits` — tested removed `McpServerOutcome`.
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::donation_entry_points_are_inert_without_a_hub` — called removed `trace_donation_reporter`/`log_donation_layer`/`metric_donation_reporter`.
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::on_mcp_snapshot_changed_emits_per_session_events_and_rebuilds` — called removed hub/MCP snapshot-change hooks.
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::on_hub_tools_changed_emits_per_session_events` — called removed hub/MCP snapshot-change hooks.
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::on_hub_tools_changed_updates_snapshot` — called removed hub/MCP snapshot-change hooks.
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::connect_hub_noop_when_no_config` — called removed `connect_hub`/`shutdown_hub` and `WorkspaceShared::hub_server`.
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::observe_connect_hub_catalog_result_records_error_pair` — called removed `connect_hub`/`shutdown_hub` and `WorkspaceShared::hub_server`.
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::shutdown_hub_noop_when_not_connected` — called removed `connect_hub`/`shutdown_hub` and `WorkspaceShared::hub_server`.
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::reload_is_a_no_op_without_local_mcp_configuration` — called removed `reload_bind_mcp`.
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::bind_mcp_config_rejects_rpc_reconfiguration` — called removed `start_session_mcp_servers`.
- `crates/codegen/xai-grok-workspace/src/handle_tests.rs::a_teardown_mid_connect_drops_the_finished_client` — exercised removed `teardown_session_mcp` and `WorkspaceSession::mcp_state`.
- `crates/codegen/xai-grok-workspace/src/permission/hub_gate_tests.rs::reads_run_unasked_and_undecodable_calls_return_the_toolset_error` — called the removed `hub_gate::settle`/`approve_hub_call` orchestration.
- `crates/codegen/xai-grok-workspace/src/permission/hub_gate_tests.rs::mutations_fail_closed_without_a_transport_and_yolo_needs_the_unattended_ceiling` — called the removed `hub_gate::settle`/`approve_hub_call` orchestration.
- `crates/codegen/xai-grok-workspace/src/permission/hub_gate_tests.rs::a_persisted_grant_or_deny_settles_the_call_before_any_prompt` — called the removed `hub_gate::settle`/`approve_hub_call` orchestration.
- `crates/codegen/xai-grok-workspace/src/permission/hub_gate_tests.rs::the_sandbox_card_gate_keeps_persisted_denies_and_always_prompt_ahead_of_the_run` — called the removed `hub_gate::settle`/`approve_hub_call` orchestration.
- `crates/codegen/xai-grok-workspace/src/permission/hub_gate_tests.rs::a_run_requested_in_the_background_keeps_the_pre_run_prompt_under_the_enforced_gate` — called the removed `hub_gate::settle`/`approve_hub_call` orchestration.
- `crates/codegen/xai-grok-workspace/src/permission/hub_gate_tests.rs::a_foreground_gated_call_still_leaves_its_prompt_to_the_sandbox_card` — called the removed `hub_gate::settle`/`approve_hub_call` orchestration.
- `crates/codegen/xai-grok-workspace/src/permission/hub_gate_tests.rs::an_edit_session_grant_does_not_pre_decide_a_tool` — called the removed `hub_gate::settle`/`approve_hub_call` orchestration.
- `crates/codegen/xai-grok-workspace/src/permission/hub_gate_tests.rs::the_protected_target_floor_prompts_whatever_the_grants_say` — called the removed `hub_gate::settle`/`approve_hub_call` orchestration.
- `crates/codegen/xai-grok-workspace/src/permission/hub_gate_tests.rs::the_ambient_git_scan_runs_on_the_hub_path` — called the removed `hub_gate::settle`/`approve_hub_call` orchestration.
- `crates/codegen/xai-grok-workspace/src/permission/hub_gate_tests.rs::a_rebind_applies_the_tenant_approval_ceiling` — called the removed `hub_gate::settle`/`approve_hub_call` orchestration.
- `crates/codegen/xai-grok-workspace/src/permission/hub_gate_tests.rs::a_blanket_bash_allow_needs_the_unattended_ceiling` — called the removed `hub_gate::settle`/`approve_hub_call` orchestration.
- `crates/codegen/xai-grok-workspace/src/permission/hub_gate_tests.rs::an_always_reject_on_a_tool_or_a_domain_is_persisted_for_the_folder` — called the removed `hub_gate::settle`/`approve_hub_call` orchestration.
- `crates/codegen/xai-grok-workspace/src/permission/hub_gate_tests.rs::an_always_answer_is_persisted_for_the_folder_and_skips_the_next_prompt` — called the removed `hub_gate::settle`/`approve_hub_call` orchestration.
- `crates/codegen/xai-grok-workspace/src/permission/hub_gate_tests.rs::an_always_answer_under_the_served_root_holds_for_sibling_conversations` — called the removed `hub_gate::settle`/`approve_hub_call` orchestration.
- `crates/codegen/xai-grok-workspace/src/host_kind_tests.rs::only_a_sandbox_session_carries_the_credential` — asserted the removed hub credential/`session_context_factory`/`bind_resolver_fixture` surface.
- `crates/codegen/xai-grok-workspace/src/host_kind_tests.rs::a_daemon_host_serves_pinned_toolsets_only` — asserted the removed hub credential/`session_context_factory`/`bind_resolver_fixture` surface.
- `crates/codegen/xai-grok-workspace/src/host_kind_tests.rs::a_sandbox_host_keeps_the_credential_and_lax_binds` — asserted the removed hub credential/`session_context_factory`/`bind_resolver_fixture` surface.
- `crates/codegen/xai-grok-workspace/src/permission/hub_permission.rs::is_timeout_err_matches_backstop_wording_only` — tested removed `hub_permission::is_timeout_err`.
- `crates/codegen/xai-grok-workspace/src/permission/hub_permission.rs::request_sends_payload_and_decodes_reply` — tested removed `hub_permission::request_permission_via_hub`.
- `crates/codegen/xai-grok-workspace/src/permission/hub_permission.rs::transport_error_fails_closed` — tested removed `hub_permission::request_permission_via_hub`.
- `crates/codegen/xai-grok-workspace/src/permission/hub_permission.rs::edit_always_approve_maps_to_session_scope` — tested removed `hub_permission::request_permission_via_hub`.

## `xai-grok-workspace` hub permission transport (manager + prompter)

`prompter::Prompter` no longer carries a hub permission transport: `request` routes
every prompt through the local ACP gateway and treats a present transport as
removed (`unreachable!("hub permission transport removed")`). Tests that built a
manager around `test_manager_with_hub`/`FakeHubTransport` exercised that removed
transport and were deleted with their helpers.

- `permission::manager::tests::test_manager_with_hub` (helper) — spawned a manager with the removed hub transport.
- `permission::manager::tests::FakeHubTransport` (helper) — fake implementation of the removed transport.
- `permission::manager::tests::fake_hub` (helper) — constructor for the fake transport.
- `permission::manager::tests::hub_permission_approve_allows_and_emits_payload` — asserted the removed hub permission payload.
- `permission::manager::tests::hub_permission_reject_aborts` — asserted the removed hub reject outcome.
- `permission::manager::tests::hub_permission_cancelled_aborts_distinctly` — asserted the removed hub cancel outcome.
- `permission::manager::tests::hub_permission_always_approve_persists_scope` — asserted hub always-approve scope persistence.
- `permission::manager::tests::session_edit_grant_excludes_protected_target` — drove the removed hub always-approve path.
- `permission::manager::tests::shared_manager_uses_request_path_context` — drove the removed hub prompt path.
- `permission::manager::tests::ambiguous_mcp_server_scope_downgrades_to_exact_persisted_grant` — asserted hub-scope downgrade.
- `permission::manager::tests::edit_session_grant_does_not_predecide_agent_message` — drove the removed hub transport.
- `permission::manager::tests::agent_message_approval_does_not_grant_later_messages_or_edits` — drove the removed hub transport.

## `xai-grok-workspace` sandbox real-wiring through the deleted hub dispatch

The sandbox real-wiring tests dispatched tool calls through the deleted
`crate::hub::SessionRoutedToolHandler::handle_call`, which performed the pre-run
approval gate (`approve_hub_call`), the mode-layer write guard
(`refuse_mode_layer_write`), sandbox pin/floor and the call-table finish, and the
shell result decode/replay. That dispatch is gone; the local test shim routes
through `WorkspaceHandle::create_local_harness` → `SessionToolHandle`, which runs
the toolset directly and performs none of that integration, so the assertions no
longer hold. Retained local sandbox behavior (`off` runs, enforce-without-backend
refusal, daemon-off unasked run, mode-verb engage, proxy-off) stays covered.

- `sandbox::real_wiring_tests::observe_runs_the_real_child_unwrapped_counts_it_and_never_shows_a_card`
- `sandbox::real_wiring_tests::a_full_call_table_refuses_a_new_command_but_not_a_tool_that_spawns_nothing`
- `sandbox::real_wiring_tests::an_unpinned_shell_call_spawns_no_weaker_than_its_dispatch_mode_after_a_flip`
- `sandbox::real_wiring_tests::enforce_decodes_a_refused_write_and_keeps_the_denial_without_a_hub`
- `sandbox::real_wiring_tests::enforced_pre_run_gate_leaves_shell_calls_to_the_sandbox_gate`
- `sandbox::real_wiring_tests::enforced_pre_run_gate_still_asks_for_a_run_requested_in_the_background`
- `sandbox::real_wiring_tests::enforced_pre_run_gate_still_asks_for_shell_calls_under_observe`
- `sandbox::real_wiring_tests::enforced_pre_run_gate_refuses_a_persisted_deny_before_the_spawn`
- `sandbox::real_wiring_tests::enforced_pre_run_gate_still_asks_always_prompt_tenants_under_enforce`
- `sandbox::real_wiring_tests::a_file_tool_never_writes_a_mode_layer_under_enforce`
- `sandbox::real_wiring_tests::a_file_tool_never_creates_a_missing_mode_layer_under_enforce`
- `sandbox::real_wiring_tests::a_tool_that_is_no_shell_never_meets_the_sandbox`
- `sandbox::real_wiring_tests::a_shell_call_whose_enforce_pin_is_gone_is_refused_unrun`
- `sandbox::real_wiring_tests::shell_call_dispatched_under_observe_runs_observed_after_a_flip_to_enforce`
- `sandbox::real_wiring_tests::enforce_serves_the_folder_with_a_proxy_the_real_child_is_pointed_at`
- `sandbox::real_wiring_tests::a_card_answered_over_the_installed_transport_replays_the_real_command_once`

## `xai-grok-workspace` removed metric baselines

`init_metrics_tests::init_metrics_is_idempotent_and_registers_baselines` is
retained; its assertions for metrics whose families were deleted were removed:
`grok_workspace_rpc_requests_total` / `grok_workspace_rpc_errors_total`
(registered by the deleted `hub_server.rs`) and
`grok_workspace_oidc_proactive_refresh_total` (OAuth/OIDC proactive refresh
removed with the xAI auth stack). The remaining baseline assertions (startup
stages, drain, toolset swap, env-capture panic, permission timeout) still run.
## `xai-grok-shell` lib tests

The Phase-1 cleanup deleted the remote conversations lane, the remote session
title/identity writeback (`RemoteSync`), the trace/GCS upload pipeline, the
leader auth wiring, remote model prefetch, and remote settings bootstrap. Tests
that asserted those removed surfaces were deleted. Retained local behavior was
repaired where possible: `unified_list::cursor` cursor codec round-trip,
`unified_list::facets` local git/workspace facets, the `persistence_tests`
actor harness, and 33 `handle_prompt` call sites (the two removed upload
parameters `trace_gcs_config`/`artifact_tracker` were stripped from callers).

Dropped test functions (55) and helpers that referenced deleted types:

- `session/persistence_delete_session_history_tests.rs` (file removed; its `#[path]` mod include dropped) — 5 tests asserted remote session-delete classification via the deleted `crate::remote`: `remote_ok_reports_removed`, `remote_404_is_treated_as_already_deleted`, `remote_auth_failure_aborts`, `remote_non_404_request_failure_aborts`, `any_removed_reflects_either_location`.
- `session/unified_list/cursor.rs` — deleted remote conversations pagination lane (`ConvLane`, `conv_frontier`, `conversation_to_row`, `crate::remote::Conversation`): `multi_page_walk_equals_single_fetch_window`, `equal_updated_at_tie_break_no_drop_or_dup`, `partial_conv_page_is_not_advanced_until_drained`, `whole_page_filtered_out_does_not_drop_later_match`, `local_only_when_conversations_skipped`, `degraded_lane_sets_partial_and_returns_local`, `degraded_mid_walk_with_progress_keeps_live_conv_token`, `degraded_mid_walk_with_no_progress_terminates`, `degraded_first_page_with_no_token_does_not_fabricate_a_cursor`. Removed helpers `ConvSource`, `conv`, `local`, `ids`, `walk_all`. Retained `cursor_round_trips`, `malformed_cursor_decodes_to_fresh_first_page`.
- `session/unified_list/row.rs` — `conversation_row_uses_conversation_id_as_session_id`, `conversation_missing_modify_time_still_resumable` (deleted `Conversation`/`conversation_to_row`). Retained local ACP conversion tests.
- `session/unified_list/facets.rs` — conversation/starred/workspace partition tests: `project_facet_only_on_conversations`, `project_filter_is_partition_aware_keeps_local_rows`, `repo_filter_is_partition_aware_keeps_conversation_rows`, `pushdown_and_in_memory_project_filter_agree`, `starred_facet_present_only_for_starred_conversations`, `starred_filter_is_partition_aware_keeps_local_rows`; helpers `conv_row`, `conv_row_starred`. The local half of `git_path_facets_present_only_for_local_rows` was repaired and retained, as were the workspace pushdown and git-root filter tests.
- `session/unified_list/mod.rs` — `degraded_conversations_lane_reports_no_oauth` (deleted `ConversationsClient`), `conversations_lane_active_truth_table` (deleted `crate::agent::chat_modes`).
- `session/persistence_tests.rs` — remote writeback/title-sync tests: `writeback_backfill_is_fresh_only_and_acp_only`, `winning_identity_stamp_seeds_remote_writeback`, `pending_drain_disposition_controls_remote_sync`, `durable_append_committed_failure_is_synced`, `manual_rename_next_flush_does_not_revert_backend_title`, `manual_after_auto_last_flush_is_manual`, `auto_after_committed_manual_emits_no_set_title`, `reset_title_to_auto_then_generated_title_is_adopted`; helper `test_actor_with_remote_sync`. `test_actor`/`test_actor_inner` were repaired to drop the `RemoteSync` parameter and all other durability/restore tests remain.
- `agent/init_tests.rs` — remote-settings startup prefetch/deadline tests: `startup_settings_deadline_selects_by_profile`, `post_gate_pass_spends_at_most_one_settings_budget`, `supplied_settings_skip_the_getter` (removed `cloud_config`, `model_catalog::settings_get`, `apply_post_gate_settings`). Bootstrap gate tests retained.
- `agent/config_tests.rs` — `resolve_upload_method_accepts_deployment_key_without_oauth` (deleted `session::repo_changes::UploadMethod`), `slug_inherited_unmarked_capabilities_menu_keeps_no_default_effort` and `capabilities_menu_without_default_resolves_to_no_reasoning_effort` (deleted `crate::remote::client::parse_remote_model_value`). The remaining 314 config tests are retained.
- `agent/subagent/tests/rest.rs` — trace-upload subagent metadata tests: `resumed_from_field_in_meta_roundtrips`, `subagent_session_metadata_roundtrip`, `subagent_session_metadata_non_forked`, `subagent_session_metadata_backward_compat_deserialization`, `upload_lifecycle_spawn_then_completion_preserves_fields`, `upload_lifecycle_failure_preserves_error`, `session_metadata_session_kind_for_resumed`, `upload_ref_includes_resumed_from` (deleted `crate::upload::trace::SubagentSpawnedRef` / `SubagentSessionMetadata`).
- `agent/subagent/tests/mod.rs` — `turn_message_take_is_bounded_and_records_the_wedge`, `turn_message_take_records_the_miss_when_the_actor_is_gone`, `streaming_partial_take_is_bounded_when_the_child_actor_is_wedged`, `resolved_model_read_is_bounded_and_falls_back_to_configured` (deleted `handle_request::{take_child_turn_messages,take_child_streaming_partial,resolve_child_model}` and `crate::upload::turn`); helper `test_gcs_context` (deleted `GcsUploadContext`).
- `agent/mvp_agent/tests.rs` — `upload_harness_trace_turns_numbers_siblings_and_persists_counter`, `upload_harness_trace_turns_build_per_turn_manifest` (deleted `crate::upload` / `TraceUploadEndpoints`); the uploads-disabled guard test is retained.
- `leader/server_tests.rs` — `wait_for_leader_auth_returns_when_already_wired`, `wait_for_leader_auth_resolves_when_wired_late`, `hubless_workspace_exposure_arms_no_metric_pump_and_tears_down_cleanly` (deleted `wait_for_leader_auth`, `arm_metric_donation`); helper `TestAuth` (deleted `AuthProvider`/`AuthCredential`). The other 149 leader tests are retained.

### Second compile layer (batch 2 — resolved)

The 166-error second layer is now adjudicated; `cargo check -p xai-grok-shell
--lib --tests` is green. Most were repairs, not drops: removed test-constructor
fields were stripped (all `None`/placeholder) and call-site arity matches were
fixed where the remaining behavior is kept. Dropped tests/helpers that only
asserted deleted upload/remote/collection surfaces:

- `session/persistence_tests.rs` — remaining remote writeback/title-sync and
  durable-append-sync tests: `durable_append_committed_failure_is_synced`,
  `pending_drain_disposition_controls_remote_sync`,
  `manual_rename_next_flush_does_not_revert_backend_title`,
  `manual_after_auto_last_flush_is_manual`,
  `auto_after_committed_manual_emits_no_set_title`,
  `reset_title_to_auto_then_generated_title_is_adopted`,
  `reset_title_to_auto_adopts_in_flight_generation_as_auto`,
  `winning_identity_stamp_seeds_remote_writeback`,
  `writeback_backfill_is_fresh_only_and_acp_only`; helper `recv_observed`.
  (Batch 1's ledger line for this file listed these as already dropped but they
  were only adjudicated here; the durability/flush/restore tests remain.)
- `agent/config_tests.rs` — `coding_data_opt_out_does_not_block_uploads_to_own_bucket`
  (deleted `EndpointsConfig::is_trace_upload_blocked_for`; `trace_upload_bucket`
  no longer drives upload gating).
- `agent/mvp_agent/tests.rs` — trace/diagnostic upload config tests:
  `upload_harness_trace_turns_uploads_disabled_does_not_burn_counter`,
  `data_collection_disabled_for_zdr_team`,
  `data_collection_disabled_for_opted_out_team`,
  `diagnostic_upload_skipped_for_opted_out_user`,
  `diagnostic_upload_skipped_without_credentials`,
  `zdr_team_uploads_no_traces_to_own_bucket`,
  `diagnostic_upload_skipped_for_opted_out_user_with_own_bucket`,
  `diagnostic_upload_skipped_after_mid_session_trace_upload_kill_switch`,
  `collection_config_gate_mirror_follows_trace_upload_flip` (deleted
  `upload_harness_trace_turns`, `trace_upload_config_snapshot`,
  `diagnostic_upload_config`, `sync_collection_config_gate`,
  `MvpAgent.trace_upload_live`, and `crate::upload` / `TraceUploadEndpoints`).
  The `is_data_collection_disabled` privacy tests remain.
- `session/signals_tests.rs` — `test_gcs_queue_snapshot` (deleted
  `SignalEvent::RecordGcsQueueSnapshot` and the `gcs_queue_*` signal fields).
- `session/slash_commands_tests.rs` — `feedback_resolves_when_enabled` (deleted
  `BuiltinAction::Feedback`). `available_commands_orders_builtins_first` is
  retained but repaired to drop `feedback` from the expected builtin list.
- `agent/mvp_agent/tests.rs` (runtime failures) —
  `spawn_settings_reapply_coalesces_while_in_flight`,
  `spawn_settings_reapply_clears_flag_after_completion`,
  `post_auth_settings_not_coalesced_by_in_flight_reapply` asserted coalescing
  around the now no-op `spawn_settings_reapply`/`spawn_post_auth_settings`
  (remote settings bootstrap removed); and
  `restore_keeps_the_saved_context_window_selection_without_a_catalog` asserted
  the removed "catalog not yet fetched" restore path (`wait_for_first_catalog`
  is now always true). The counterpart
  `restore_applies_the_saved_context_window_selection` is retained.
- `session/acp_session_tests/idle_resume_tests.rs` —
  `test_e2e_idle_resume_refreshes_model_metadata` asserted a remote
  `/models-v2` metadata refresh (now a no-op); `test_last_api_request_at_idle_detection`
  and `test_idle_resume_noop_when_not_idle_enough` remain.

Repaired at runtime (not dropped):
- `session/acp_session_tests/auth_error_no_retry_tests.rs` legacy-auth hint
  tests now assert the rebranded `deepseek-build update/logout/login` hints.
- `tools/notification_bridge_tests.rs`
  `task_completed_notification_stamps_will_wake` no longer expects the removed
  trace `SessionCommand::CopyFile`.

Repaired, not dropped (kept behavior):
- Removed the deleted constructor fields from test fixtures across
  `SessionActor` (`upload_queue`, `trace_config_template`),
  `SessionContext` (`auth_provider`), `InputItem`/`TurnInputRequest`
  (`trace_gcs_config`, `artifact_tracker`), `HumanPromptContent`
  (`artifact_upload_ctx`), `InjectParams`/`SubagentSpawnContext`
  (`synthetic_trace_tx`), `OneTurnAttemptInput` (`gcs_bucket_url`,
  `gcs_upload_method`), `SessionHandle` (`upload_queue`,
  `upload_failures_since_success`).
- Fixed `handle_prompt` (12→10) and
  `process_conversation_turn_with_recovery` (6→4) call arity, and
  `delete_session_history` (5→3) in `tests/session_delete_evicts_index.rs`.
- Dropped the `remote_fetch_enabled` first arg from `ModelsManager::new` in
  `agent/subagent/tests/rest.rs` and the now-removed `settle_first_catalog_for_tests`
  calls in `agent/mvp_agent/tests.rs` (the local catalog is always complete).
- Dropped the trace-channel assertions from the retained
  `task_completed_notification_stamps_will_wake` and the subagent
  `inject_*` tests; their wake/admission behavior assertions remain.
- Repaired `persistence_tests::test_actor_inner` call sites and
  `session_delete_evicts_index.rs` (removed the now-unused auth setup).

## `xai-grok-shell` integration tests

The remote-settings prefetch and the managed-config startup profile were
removed in Phase 1; tests in `crates/codegen/xai-grok-shell/tests/**` that
asserted them were deleted with the surface. Local behavior (boot, plugins,
sessions gauge, subagent bootstrap, spawn timers) is retained and the shared
`acp_harness` / `common` helpers were repaired.

- `tests/test_startup_prefetch_fallback.rs` — asserted the removed remote-settings prefetch fallback.
- `tests/test_startup_prefetch_overlap.rs` — asserted the removed remote-settings prefetch overlap.
- `tests/test_startup_prefetch_policy.rs` — asserted the removed remote-settings fetch policy.
- `tests/test_startup_prefetch_repair_overlap.rs` — asserted the removed remote-settings repair overlap.
- `tests/test_startup_prefetch_repair_skip.rs` — asserted the removed remote-settings repair gate.
- `tests/test_startup_prefetch_shared.rs` — asserted the removed remote-settings prefetch sharing.
- `tests/test_startup_settings_unpersisted_session.rs` — asserted the removed in-memory remote-settings fetch.
- `tests/test_startup_boot_current_thread.rs` — asserted the removed `resolve_boot_startup_settings` + `RemoteSettings` path.

Helper cleanup: removed the four dead calls to
`xai_grok_shell::agent::remote_config::settings_get::reset_startup_settings_for_tests`
and `xai_grok_shell::managed_config::clear_startup_profile_for_tests` from
`tests/common/mod.rs` and `tests/acp_harness/mod.rs`.

## `xai-grok-pager` lib tests

The Phase-1 deletion of the `xai-grok-feedback` crate removed the pager's
feedback modal, inline `/feedback` dispatch, draft store, and trace-upload flow,
and the announcements/upgrade-CTA banner surface was stripped from
`AgentView`/`AppView`/dashboard. The pager and pager-minimal lib test targets
then failed to compile (252 + 2 errors) because their test modules still
referenced those deleted symbols. Tests that only asserted the removed
feedback/trace/announcements behavior were dropped; retained behavior was
repaired.

Dropped test functions (and helpers that only served them):

- `app/dispatch/tests/notes.rs` — the whole feedback/trace suite: `unknown_immediate_feedback_outcome_warns_against_duplicate_retry`, `feedback_failed_keeps_the_report_as_a_draft_and_spares_the_composer`, `inline_feedback_without_a_session_keeps_the_report_in_the_notice`, `inline_feedback_sends_immediately_while_a_turn_runs`, `minimal_typed_bare_feedback_opens_the_modal_and_yields_btw`, `typed_bare_feedback_moves_composer_images_into_the_modal`, `draft_preserving_feedback_uses_the_submitted_command_without_draft_images`, `send_feedback_without_agent_view_cleans_staged_temp_files`, `send_feedback_preserves_composer_draft`, `feedback_modal_opens_empty_and_prefilled`, `feedback_modal_open_refuses_visibly_on_dashboard`, `feedback_modal_open_preserves_main_composer_draft`, `feedback_modal_image_only_sends`, `feedback_modal_rejected_image_only_submit_keeps_the_modal_and_attachment`, `feedback_modal_submit_strips_image_chips_from_post_body`, `feedback_modal_submit_closes_immediately_and_failure_keeps_a_draft`, `edited_enums_ride_the_send_feedback_metadata`, `drafts_with_optional_taxonomy_omitted_remain_sendable`, `deferred_feedback_submit_stays_armed_while_the_agent_is_off_screen`, `dropped_or_failed_feedback_modal_probe_still_inserts_the_caption`, `feedback_modal_success_completion_is_quiet_without_consent`, `stale_feedback_modal_completion_leaves_a_later_modal_alone`, `feedback_modal_open_refuses_while_one_is_open`, `draft_update_completion_routes_its_outcome_to_the_open_modal`, `displaced_draft_send_outcome_is_reported_in_scrollback`, the five ingress-displacement tests (`acp_question_displaces_feedback_modal_and_keeps_main_draft`, `permission_ingress_displaces_feedback_modal`, `cancel_turn_prompt_displaces_feedback_modal`, `plan_approval_ingress_displaces_feedback_modal`, `mcp_elicitation_ingress_displaces_feedback_modal`), the trace-consent suite (`write_submit_sends_directly_unless_the_trace_offer_applies`, `trace_offer_sequences_post_then_exactly_one_upload`, `terminal_feedback_outcomes_take_parked_consent_and_upload_only_remote_successes`, `ninth_trace_submit_is_rejected_without_revoking_confirmed_consent`, `feedback_complete_without_token_keeps_parked_consent`, `trace_post_failure_yields_zero_uploads`, `mid_flight_question_does_not_drop_committed_trace_consent`, `feedback_only_and_never_ask_upload_nothing_and_never_persist_trace_upload`, `stale_trace_completion_cannot_touch_a_later_modal`); helpers `open_feedback_modal`, `confirm_trace_choice`, `expect_single_modal_post`, `park_send_this_session`, `test_pasted_image`, `test_pasted_png` (the latter two were only used by dropped tests; retained tests use other fixtures). The `/btw`/recap/remember/`btw_no_session_feedback_is_mode_specific`/`auth_meta_refreshes_feedback_trace_offer` tests remain.
- `app/dispatch/tests/router.rs` — `send_feedback_clears_active_ephemeral_tip` (drove the removed `Action::SendFeedback`), `cta_impressions_respect_slot_gate_and_paint` and helpers `critical_announcement`, `promo_announcement`, `shown_banner_id` (removed announcement banner/CTA slot).
- `app/dispatch/queue.rs` — `edited_queued_bare_feedback_opens_the_modal` and `run_edited_feedback_uses_row_text_and_attachments_not_the_restored_draft` (asserted `/feedback` dispatch, `Effect::SendFeedback`, `feedback_modal`).
- `app/dispatch/tests/task_result.rs` — `doctor_planning_displaces_feedback_before_opening_question` (asserted a feedback modal was evicted by a doctor plan question; the doctor-plan-question assertions themselves remain in `doctor_planning_opens_refuses_remote_and_rejects_stale_identity`).
- `app/effects/tests.rs` — `upload_trace_request_with_intent_exact_wire_shape`, `upload_trace_request_without_intent_keeps_legacy_wire_shape` (removed `UploadTraceRequest` / `FeedbackTraceUploadIntent`).
- `app/turn_completion/tests.rs` — `hook_denied_finalize_displaces_feedback_before_opening_the_card` (feedback modal displacement; the card-opening assertion remains in `hook_denied_finalize_requeues_blocked_prompt_and_opens_card`).
- `app/agent_view/links.rs` — `click_on_announcement_hide_button_dispatches_hide_action`, `click_on_announcement_cta_button_dispatches_open_action` (removed `Action::AnnouncementsHide` / `AnnouncementsOpenCta` and `hit_announcement_*` fields).
- `app/acp_handler/tests/settings.rs` — `settings_update_ignores_announcements_payload` (removed announcement state and generation watermark; the test also asserted `show_resolved_model` handling, which stays covered by the other settings tests).
- `app/acp_handler/tests/mod.rs` — helpers `critical_announcement`, `announcements_update_notif`, `shown_banner_id` (removed announcements surface).
- `app/agent_view/links.rs` — helper `draw_banner_frame`/`draw_frame_sized`/`draw_frame_privacy` repaired (dropped the removed `announcements`/`hidden_ids` args) and retained.
- `xai-grok-pager-minimal/src/overlay.rs` — `is_live_region_modal_active_ignores_prompt_modals` repaired: the feedback-modal arm was replaced with an `ActiveModal::CommandPalette` arm so the band-owning-modal assertion still runs.

Repaired, not dropped (kept behavior): the `render_dashboard` (12-arg) and
`render_header` (6-arg) call sites across `views/dashboard/*_tests.rs`,
`app/app_view_tests.rs`, `app/dispatch/tests/{dashboard,session/lifecycle}.rs`
(dropped the removed `upgrade_cta`/banner args); the `BannerSlotParams` literals
in `app/agent_view/task_status_tests.rs`; `app/app_view_tests.rs` and
`app/dispatch/tests/mod.rs` test-app literals and the ephemeral-tip occluder
tests now use `privacy_banner.active` (the remaining banner-slot occluder) in
place of the removed `session_banner_active`/announcement fields; the
`apple_terminal_ctrl_o_*` and plan-approval tests now use `/btw` in place of the
deleted `/feedback` as their example freeform builtin (`queue_edit.rs`'s
`edit_local_btw_carries_the_submitted_images`). Stale pre-rebrand expectations
(`grok` → `deepseek-build` CLI strings and title suffixes, closing
`doctor >>>` managed-block markers, the fake-binary name) were corrected in the
diagnostics/doctor/notifications/version-mismatch tests so the suite runs green;
those tests were not dropped.

## `xai-fast-worktree` NFS/Grove-feature tests

Upstream disabled the real NFS/Grove worktree backend in this tree: `lib.rs`
maps `mod nfs` to `nfs_off.rs`, whose `candidate_data_dirs()` returns empty and
whose `gc_orphan_pins()` is a no-op (the real `nfs.rs` is not present). Four
tests in the `metadata`-gated modules still asserted the removed NFS behavior
(backing-directory markers, XDG data-dir scanning, the orphan-pin GC sweep).
They only compile when the `metadata` feature is on, which workspace-wide
feature unification enables — so they were invisible to per-crate runs and
first executed by the full `cargo test --workspace` run (2026-10-08), where
they failed deterministically.

Dropped test functions:

- `api/gc.rs` → `api/gc/tests.rs` — `run_pass_prunes_orphan_grove_pins_after_grace`
  (asserted the production GC invokes the pin sweep; `nfs_off::gc_orphan_pins`
  is a no-op, so `pin_gc_examined` is always 0).
- `discovery.rs` — `rebuild_nfs_under_managed_roots_is_not_labeled_linked`,
  `rebuild_registers_nfs_from_backing_marker`,
  `rebuild_scans_xdg_grove_without_grove_data_dir` (all build a real Grove
  backing-dir marker and expect `rebuild_worktree_db*` to register an NFS row;
  `nfs_off::candidate_data_dirs()` is empty, so `discovered`/`registered` are 0).

The local filesystem rebuild behavior (the non-NFS rows) remains covered by the
retained `discovery` tests. After the drop: `xai-fast-worktree` lib tests
455 passed / 0 failed / 2 ignored with `--features metadata` (was 4 failed).

## `xai-grok-login` removed-network tests

The `dsb/auth` slice (e4c1809a) deleted the xAI OAuth/OIDC/device-code network
stack and stubbed its entry points: `oidc_token_exchange` always returns a
transient failure without touching the network, `fetch_login_device_flow`
always returns `None`, and the `/user` enrichment fetch returns `None`. The
crate's tests were never swept after that slice, so these tests failed or hung
deterministically (never reached a real IdP in this build):

- `refresh/oidc_refresher_tests.rs` — the mock-IdP e2e suite:
  `oidc_refresher_e2e_full_refresh_cycle`,
  `oidc_refresher_e2e_force_refreshes_locally_valid_token`,
  `oidc_refresher_e2e_near_expiry_within_buffer_refreshes`,
  `oidc_refresher_attributes_the_refresh_token_it_spent_on_invalid_grant`,
  `oidc_refresher_e2e_near_expiry_idp_rejects_refresh`,
  `oidc_refresher_e2e_invalid_client_retains_credentials`,
  `oidc_refresh_uses_disk_refresh_token`,
  `lock_timeout_falls_through_to_refresh`,
  `refresher_retries_with_disk_token_after_invalid_grant`,
  `refresher_disk_retry_invalid_client_with_different_client_id_preserves_disk`,
  `refresher_disk_retry_is_one_shot`, and the now-unused helpers
  `start_mock_oidc_with_disk_rotation`, `start_counting_mock_oidc`,
  `expired_oidc_for`. Retained and repaired: the two sleep-gate tests now
  drive the real gate/drain with a controllable `TokenRefresher` instead of a
  mock IdP (`sleep_gate_e2e_defers_then_recovers_on_wake`,
  `sleep_gate_e2e_in_flight_refresh_completes_across_imminent_sleep`), which
  also removes the deadlock they had (they waited forever on a mock `/token`
  hit that the inert exchange can never produce).
- `refresh/auth_backend_contract_tests.rs` —
  `auth_backend_contract_token_responses_map_to_outcomes`,
  `auth_backend_contract_concurrent_401s_hit_idp_once`,
  `auth_backend_contract_dead_token_emits_typed_manual_auth_event`,
  `auth_backend_contract_two_instances_share_one_idp_call` (all drive a mock
  IdP token endpoint). `auth_backend_contract_transient_failures_escalate_to_non_sticky_permanent`
  stays: escalation is local logic the stub still exercises.
- `flow.rs` — `expired_refreshable_session_gate` (an xAI-session fallback whose
  filter `is_xai_auth()` is permanently false in this build),
  `external_reauth_without_prev_auth_enriches_inline` (asserted the removed
  `/user` fetch), `run_auth_flow_falls_through_when_no_refresh_token` (asserted
  the removed device-code request), `no_mint_readiness_auth_is_bounded`
  (bounded a hanging OIDC IdP; the retained bound stays pinned by the
  `STARTUP_AUTH_REFRESH_TIMEOUT < REFRESH_LOCK_TIMEOUT` const assert and by
  `readiness_auth_stays_bounded_when_auth_lock_is_held`),
  `fetch_login_device_flow_parses_2xx_bodies`,
  `fetch_login_device_flow_sends_only_unauthenticated_headers`, and the
  helper `start_hanging_oidc_idp`.
- `manager_tests.rs` — the background `/user` enrichment suite:
  `update_writes_disk_before_user_enrichment`,
  `enrichment_task_preserves_interleaved_token_rotation`,
  `enrichment_overlays_team_login_placeholder_user_id`,
  `enrich_auth_inline_populates_zdr_flags`,
  `enrich_auth_inline_keeps_fields_absent_from_response`, and the helper
  `spawn_user_stub`. The retained local merge behavior stays covered by the
  `apply_user_info_enrichment_*` tests.

Repaired, not dropped (retained behavior):
- `external_auth::tests::parse_output_issuer_claim_enables_xai_auth` renamed to
  `parse_output_issuer_claim_is_stored_but_never_first_party`: an issuer claim
  is still parsed and stored, but no issuer classifies as first-party xAI auth
  in this build.
- Every test that pinned the API-key env vars now guards the names the product
  actually reads (`DEEPSEEK_API_KEY`, `DEEPSEEK_BUILD_API_KEY`) instead of the
  removed `XAI_API_KEY` / `GROK_CODE_XAI_API_KEY`, so a developer's exported
  key can no longer leak into the expectations.
- `storage::write_fallback_tests::atomic_write_writes_through_symlink_and_keeps_owner_only`
  clears `DEEPSEEK_BUILD_HOME` (which outranks its `GROK_HOME` guard) so a
  developer env override can no longer defeat the test's temp home.

Follow-up (not done here): `flow::expired_refreshable_session` is now dead code
(its `is_xai_auth()` filter is permanently false); the xAI-session fallback
call path should be excised in the same sweep that removes the remaining
retained-stub surfaces.
