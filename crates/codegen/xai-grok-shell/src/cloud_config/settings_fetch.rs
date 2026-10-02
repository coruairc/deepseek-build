//! The remote `/settings` request (fetch removed).
//!
//! The fetch has been removed. The outcome type is retained so the startup
//! settings gate keeps compiling; every fetch reports `Retry` (no remote
//! policy reached this process).
use xai_grok_config::RemoteSettings;
use xai_grok_login::GrokAuth;

/// The outcome of a `/settings` fetch.
#[derive(Debug, Clone)]
#[must_use]
pub enum SettingsFetch {
    /// The settings were fetched and parsed.
    Fetched(Box<RemoteSettings>),
    /// The server rejected the credential with a 401.
    Rejected,
    /// The fetch finished and failed with something other than a 401.
    Retry,
}

impl SettingsFetch {
    pub fn into_option(self) -> Option<RemoteSettings> {
        match self {
            SettingsFetch::Fetched(s) => Some(*s),
            SettingsFetch::Rejected | SettingsFetch::Retry => None,
        }
    }
}

/// Remote settings fetch removed; no network request is made.
pub fn fetch_settings_blocking(
    _cli_chat_proxy_base_url: &str,
    _auth: &GrokAuth,
    _alpha_test_key: Option<&str>,
) -> SettingsFetch {
    SettingsFetch::Retry
}
