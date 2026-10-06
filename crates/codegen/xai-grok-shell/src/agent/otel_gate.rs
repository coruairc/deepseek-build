//! The single owner of the fail-closed gate that decides whether customer-owned OTEL telemetry may ship.
//! Remote settings fetching (which drove a fleet policy) was removed, so no fleet policy can
//! govern this process: the gate always opens immediately.

use std::time::Duration;
pub(crate) const SETTINGS_GATE_MAX_WAIT: Duration = crate::http::SETTINGS_REAPPLY_TIMEOUT;
/// Closes the gate. It is process-global, idempotent, and callable before any `AgentConfig` exists.
pub(crate) fn suppress() {
    xai_grok_telemetry::external::set_settings_gate_max_wait(SETTINGS_GATE_MAX_WAIT);
    xai_grok_telemetry::external::suppress_external_otel_until_settings();
}
/// Whether an xAI fleet policy can govern this process. With remote settings fetch removed,
/// no policy can arrive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PolicyChannel {
    Applies,
    Unavailable(NoPolicy),
}
impl PolicyChannel {
    pub(crate) fn is_unavailable(self) -> bool {
        matches!(self, Self::Unavailable(_))
    }
}
/// Why no fleet policy can reach this process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NoPolicy {
    RemoteFetchDisabled,
    ProxyRepointed,
}
pub(crate) fn policy_channel(remote_fetch_enabled: bool, proxy_is_xai: bool) -> PolicyChannel {
    if !remote_fetch_enabled {
        return PolicyChannel::Unavailable(NoPolicy::RemoteFetchDisabled);
    }
    if !proxy_is_xai {
        return PolicyChannel::Unavailable(NoPolicy::ProxyRepointed);
    }
    PolicyChannel::Applies
}
/// [`policy_channel`] resolved against live config for the proxy actually in use.
pub(crate) fn policy_channel_for(proxy_url: &str) -> PolicyChannel {
    policy_channel(
        crate::util::config::resolve_remote_fetch_enabled(),
        crate::util::is_cli_chat_proxy_url(proxy_url),
    )
}
/// [`policy_channel_for`] against the effective config, for startup call sites that run before an `AgentConfig` exists.
pub(crate) fn resolved_policy_channel() -> PolicyChannel {
    policy_channel_for(&crate::agent::config::EndpointsConfig::from_effective_config().proxy_url())
}
/// Inputs to [`should_open_at_startup`]. Named fields keep the two booleans from being transposed at call sites.
pub(crate) struct StartupGate {
    pub(crate) channel: PolicyChannel,
    pub(crate) has_session: bool,
    pub(crate) session_pending: bool,
}
/// Returns whether a leader opens the gate at startup.
pub(crate) fn should_open_at_startup(gate: StartupGate) -> bool {
    if gate.channel.is_unavailable() {
        return true;
    }
    !gate.has_session && !gate.session_pending
}
/// Returns whether a session-less startup is about to mint a api.deepseek.com session
pub(crate) fn is_session_pending(
    has_session: bool,
    grok_com_config: &xai_grok_login::GrokComConfig,
) -> bool {
    if has_session {
        return false;
    }
    grok_com_config.auth_provider_command.is_some()
}
/// Opens the gate at startup once [`should_open_at_startup`] holds.
pub(crate) fn open_at_startup() {
    xai_grok_telemetry::external::mark_external_otel_settings_resolved();
}
/// Remembers which credential identity the process-global external-OTEL gate was resolved for.
/// With remote settings fetch removed, this is an empty placeholder kept so `MvpAgent` can hold it.
#[derive(Default)]
pub(crate) struct OtelGate {}
