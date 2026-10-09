//! First-party API-key availability check.
//!
//! The HTTP probe (`GET {base}/api-key`) has been removed. An env key is now
//! treated as usable without a network round-trip; callers that need a live
//! verdict get the local fallback.

use std::time::Duration;

/// Wall-clock budget retained for signature compatibility.
pub const DEFAULT_PROBE_TIMEOUT: Duration = Duration::from_millis(400);

/// Whether `initialize` should probe the first-party env key.
/// Kept for callers; the probe itself is inert, so this only decides whether
/// the (inert) check runs.
pub fn should_probe_first_party_env_key(
    disable_api_key_auth: bool,
    has_byok: bool,
    has_env_key: bool,
    preferred_method_pinned: bool,
) -> bool {
    !disable_api_key_auth && !has_byok && has_env_key && !preferred_method_pinned
}

/// No network probe: a present env key is advertised; a missing/empty one is not.
pub async fn first_party_env_key_allows_advertise(_api_base_url: &str, _timeout: Duration) -> bool {
    match crate::auth_method::read_xai_api_key_env() {
        Ok(key) => !key.trim().is_empty(),
        Err(_) => false,
    }
}
