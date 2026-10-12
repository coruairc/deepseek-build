pub const PAGER_CLIENT_TYPE: &str = "deepseek-build-pager";
pub const HEADLESS_CLIENT_TYPE: &str = "deepseek-build";

pub const PAGER_CLIENT_VERSION: &str = xai_grok_version::VERSION;

/// `User-Agent` for the pager's own HTTP clients that call `api.deepseek.com` directly (voice STT).
///
/// Matches the sampler's `deepseek-build/<version> (os; arch)` shape so server-side dashboards bucket voice traffic alongside chat / imagine requests.
pub fn client_user_agent() -> String {
    format!(
        "{}/{} ({}; {})",
        HEADLESS_CLIENT_TYPE,
        PAGER_CLIENT_VERSION,
        std::env::consts::OS,
        std::env::consts::ARCH,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_user_agent_has_expected_shape() {
        // e.g. "deepseek-build/1.2.3 (macos; aarch64)".
        // Servers parse this UA string, so pin the exact shape
        let ua = client_user_agent();
        assert_eq!(
            ua,
            format!(
                "{}/{} ({}; {})",
                xai_grok_brand::NAME,
                PAGER_CLIENT_VERSION,
                std::env::consts::OS,
                std::env::consts::ARCH
            )
        );
    }

    /// Guard: every client-type identity sent on the wire is brand-derived and
    /// free of the pre-rebrand name.
    #[test]
    fn client_type_constants_are_brand_derived() {
        assert_eq!(HEADLESS_CLIENT_TYPE, xai_grok_brand::NAME);
        assert!(
            PAGER_CLIENT_TYPE.starts_with(xai_grok_brand::NAME),
            "{PAGER_CLIENT_TYPE}"
        );
        for value in [PAGER_CLIENT_TYPE, HEADLESS_CLIENT_TYPE] {
            assert!(!value.contains("grok"), "stale client type: {value}");
        }
    }
}
