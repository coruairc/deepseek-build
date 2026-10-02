//! RFC 8628 Device Authorization Grant, CLI side.
//!
//! The device-code network stack has been removed. The public types and entry
//! points are retained so callers keep compiling; every network entry point
//! reports `NotEnabled`, which the login flow already treats as "unavailable".

use std::sync::Arc;

use thiserror::Error;

use crate::{AuthChannels, AuthManager, GrokAuth};

/// Only the 404 "no device endpoint" case is typed, because the login flow matches on it to fall back to loopback.
#[derive(Debug, Error)]
pub enum DeviceCodeError {
    #[error(
        "Device-code login is not available for this deployment. \
         Try `grok login` or set DEEPSEEK_API_KEY instead."
    )]
    NotEnabled,
}

/// Low-cardinality hint sent to the OAuth2 provider as the `x-grok-client-surface` header.
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::AsRefStr, strum::IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum ClientSurface {
    /// An interactive front-end (TUI/IDE) renders the URL and code to a human.
    Ui,
    /// CLI attached to an interactive terminal.
    Cli,
    /// No interactive surface (CI, container, script): no human can complete.
    Headless,
}

/// Result of requesting a device code from the server.
#[derive(Debug, Clone)]
pub struct DeviceCode {
    pub verification_uri: String,
    pub verification_uri_complete: Option<String>,
    pub user_code: String,
    device_code: String,
    interval: i32,
    expires_in: i64,
}

/// Request a device code. The network stack is removed; always fails.
pub async fn request_device_code(
    _issuer: &str,
    _client_id: &str,
    _scopes: &[String],
    _surface: ClientSurface,
) -> anyhow::Result<DeviceCode> {
    anyhow::bail!(DeviceCodeError::NotEnabled);
}

/// Complete a device-code login. The network stack is removed; always fails.
pub async fn complete_device_code_login(
    _issuer: &str,
    _client_id: &str,
    _device_code: DeviceCode,
    _auth_manager: &Arc<AuthManager>,
    _surface: ClientSurface,
) -> anyhow::Result<(GrokAuth, bool)> {
    anyhow::bail!(DeviceCodeError::NotEnabled);
}

/// Device-code login shared by the TUI and CLI. The network stack is removed.
pub async fn run_device_code_login_channels(
    _issuer: &str,
    _client_id: &str,
    _scopes: &[String],
    _auth_manager: &Arc<AuthManager>,
    _channels: &mut Option<AuthChannels>,
) -> anyhow::Result<(GrokAuth, bool)> {
    anyhow::bail!(DeviceCodeError::NotEnabled);
}
