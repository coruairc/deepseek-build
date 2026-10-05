//! Single source of truth for the user-facing product identity.
//!
//! Internal crate and package names intentionally stay `xai-grok-*` (decision
//! D2); only the surface a user sees or configures is branded here. Every
//! user-visible name, config directory, and environment-variable prefix must
//! come from this module so a future rename is a one-file change.

/// User-facing product name: the installed binary, usage text, and window
/// titles.
pub const NAME: &str = "deepseek-build";

/// Directory name (no leading path) under the user's home for per-user config.
pub const CONFIG_DIR_NAME: &str = ".deepseek-build";

/// Absolute system-wide config directory (root-owned, lowest priority).
pub const SYSTEM_CONFIG_DIR: &str = "/etc/deepseek-build";

/// Prefix for user-facing environment variables, e.g. `DEEPSEEK_BUILD_API_KEY`.
pub const ENV_PREFIX: &str = "DEEPSEEK_BUILD";

/// Primary environment variable that overrides the per-user home directory.
pub const HOME_ENV_VAR: &str = "DEEPSEEK_BUILD_HOME";

/// Legacy environment variable still honored as a fallback for
/// [`HOME_ENV_VAR`] so existing installs keep resolving their config.
pub const LEGACY_HOME_ENV_VAR: &str = "GROK_HOME";

/// Legacy config-dir name honored as a fallback when [`CONFIG_DIR_NAME`] does
/// not exist, so existing `~/.grok` installs are not stranded.
pub const LEGACY_CONFIG_DIR_NAME: &str = ".grok";

/// Build the canonical `DEEPSEEK_BUILD_<SUFFIX>` environment-variable name.
///
/// Prefer a `const` specific to the call site when one exists; this helper is
/// for call sites that only have the suffix at runtime.
pub fn env_var(suffix: &str) -> String {
    format!("{ENV_PREFIX}_{suffix}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surface_strings_are_consistent() {
        assert_eq!(NAME, "deepseek-build");
        assert_eq!(CONFIG_DIR_NAME, ".deepseek-build");
        assert_eq!(SYSTEM_CONFIG_DIR, "/etc/deepseek-build");
        assert_eq!(ENV_PREFIX, "DEEPSEEK_BUILD");
        assert_eq!(HOME_ENV_VAR, "DEEPSEEK_BUILD_HOME");
        assert_eq!(LEGACY_HOME_ENV_VAR, "GROK_HOME");
    }

    #[test]
    fn env_var_uses_prefix() {
        assert_eq!(env_var("API_KEY"), "DEEPSEEK_BUILD_API_KEY");
    }
}
