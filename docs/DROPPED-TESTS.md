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

### Second compile layer (known remaining, not yet repaired)

After this batch `cargo check -p xai-grok-shell --lib --tests` still reports 166
errors across ~33 files because the first layer masked further references to the
same deleted upload/remote/hub surfaces. The bulk are removed test-constructor
fields and call-site arity mismatches, e.g. `SessionActor.upload_queue` /
`trace_config_template`, `SessionContext.auth_provider`, `InputItem.trace_gcs_config` /
`artifact_tracker`, `HumanPromptContent.artifact_upload_ctx`, `InjectParams.synthetic_trace_tx`,
`OneTurnAttemptInput.gcs_bucket_url`/`gcs_upload_method`, and further
`handle_prompt`/turn-entry call sites. These are mechanical but have not been
adjudicated yet.
