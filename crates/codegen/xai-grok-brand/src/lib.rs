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

/// Directory name (no leading path) for project-scoped config, resolved relative
/// to a repository directory (`<repo>/.deepseek/agents`, `.deepseek/config.toml`,
/// ...). A future product variant can flip this one constant.
pub const PROJECT_DIR_NAME: &str = ".deepseek";

/// Legacy project-scoped directory names, lower priority than [`PROJECT_DIR_NAME`].
/// Kept as read compatibility so an existing `.grok/` project keeps working.
pub const LEGACY_PROJECT_DIR_NAMES: &[&str] = &[".grok"];

/// Absolute system-wide config directory (root-owned, lowest priority).
pub const SYSTEM_CONFIG_DIR: &str = "/etc/deepseek-build";

/// Prefix for user-facing environment variables, e.g. `DEEPSEEK_BUILD_API_KEY`.
pub const ENV_PREFIX: &str = "DEEPSEEK_BUILD";

/// Primary environment variable that overrides the per-user home directory.
pub const HOME_ENV_VAR: &str = "DEEPSEEK_BUILD_HOME";

/// Prefix for the product's outbound HTTP request headers.
pub const HEADER_PREFIX: &str = "x-deepseek-build";

/// Build the canonical `DEEPSEEK_BUILD_<SUFFIX>` environment-variable name.
///
/// Prefer a `const` specific to the call site when one exists; this helper is
/// for call sites that only have the suffix at runtime.
pub fn env_var(suffix: &str) -> String {
    format!("{ENV_PREFIX}_{suffix}")
}

/// Project-scoped config directory names in precedence order: the current name
/// first, then the legacy compatibility names.
pub fn project_dir_names() -> Vec<&'static str> {
    std::iter::once(PROJECT_DIR_NAME)
        .chain(LEGACY_PROJECT_DIR_NAMES.iter().copied())
        .collect()
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
        assert_eq!(HEADER_PREFIX, "x-deepseek-build");
        assert_eq!(PROJECT_DIR_NAME, ".deepseek");
        assert_eq!(LEGACY_PROJECT_DIR_NAMES, &[".grok"]);
    }

    #[test]
    fn project_dir_names_puts_the_current_name_first() {
        assert_eq!(project_dir_names(), vec![".deepseek", ".grok"]);
        assert_eq!(project_dir_names().first().copied(), Some(PROJECT_DIR_NAME));
    }

    #[test]
    fn env_var_uses_prefix() {
        assert_eq!(env_var("API_KEY"), "DEEPSEEK_BUILD_API_KEY");
    }
}
