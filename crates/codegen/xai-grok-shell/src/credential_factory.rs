//! Shell-side credential factories.
//!
//! These bind the login crate's credential providers to the shell's
//! `managed_config` deployment-id resolver, which the low-level login crate
//! cannot depend on. Everything else lives in `xai_grok_login::credential_provider`.

use std::sync::Arc;

use xai_grok_auth::AuthCredentialProvider;

/// Bootstrap the OTel credential provider both pager and TUI need at tracing init time.
/// Binds the login factory to the shell deployment-id resolver; the provider starts disk-read-only.
/// Call [`xai_grok_login::credential_provider::wire_otel_auth_manager`] after agent init to upgrade it.
pub fn build_bootstrap_otel_credentials() -> (Arc<dyn AuthCredentialProvider>, String) {
    let proxy_base_url = crate::agent::config::EndpointsConfig::from_effective_config().proxy_url();
    xai_grok_login::credential_provider::install_bootstrap_otel_provider(
        proxy_base_url,
        std::sync::Arc::new(crate::cloud_config::managed_config::resolve_deployment_id),
    )
}
