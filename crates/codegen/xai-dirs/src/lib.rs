//! Home-directory resolution generally: USERPROFILE-first `home_dir`, plus
//! grok-home (`$DEEPSEEK_BUILD_HOME` or `<home>/.deepseek-build`). Shared by
//! `xai-grok-config` and `xai-fast-worktree`. Other home env names (the legacy
//! `$DEEPSEEK_BUILD_HOME`, included) are ignored by design.
//!
//! Which function to call:
//! - [`grok_home`]: the usual choice, a cached, created path to build on.
//! - [`user_grok_home`]: `None` instead of a cwd fallback when no home resolves.
//! - [`default_grok_home`]: the `<home>/.deepseek-build` default, ignoring the env overrides, so callers can detect an override.
//! - [`resolve_grok_home`]: a fresh, uncached resolve.
//! - [`resolve_grok_home_with_source`]: [`resolve_grok_home`] plus where the path came from.
//! - [`home_dir`]: the home directory itself, for sibling dot dirs (`~/.claude`, `~/.agents`, ...).
//!
//! TODO: collapse these getters by threading the path through config as an
//! explicit value.

#![deny(clippy::indexing_slicing)]

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use xai_grok_brand::{CONFIG_DIR_NAME, HOME_ENV_VAR};

/// Where a resolved grok home came from, so "why did grok pick this
/// directory?" is answerable in diagnostics without re-reading the
/// environment at the asking site.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrokHomeSource {
    /// A non-empty `$DEEPSEEK_BUILD_HOME` override.
    EnvOverride,
    /// `<home>/.deepseek-build` derived from the home directory.
    HomeDefault,
}

/// The user's home directory via [`std::env::home_dir`]: `HOME` on Unix, `USERPROFILE` on Windows.
/// Not `dirs::home_dir()`: on Windows `dirs` ignores a redirected `USERPROFILE`.
/// Every home-anchored path must come from this one function.
#[allow(deprecated, clippy::disallowed_methods)] // the one sanctioned std::env::home_dir call
pub fn home_dir() -> Option<PathBuf> {
    std::env::home_dir()
}

/// `<home>/.deepseek-build`, canonicalized via `dunce` (not
/// `std::fs::canonicalize`, which yields Windows `\\?\` verbatim paths).
fn grok_home_in(home: &Path) -> PathBuf {
    dunce::canonicalize(home)
        .unwrap_or_else(|_| home.to_path_buf())
        .join(CONFIG_DIR_NAME)
}

/// A non-empty env override verbatim, else `<home>/.deepseek-build`.
/// Used as-is (not canonicalized) so literal prefix checks and symlink guards still see original components.
fn resolve_grok_home_from(
    grok_home_env: Option<&OsStr>,
    os_home: Option<&Path>,
) -> Option<(PathBuf, GrokHomeSource)> {
    if let Some(env) = grok_home_env.filter(|env| !env.is_empty()) {
        return Some((PathBuf::from(env), GrokHomeSource::EnvOverride));
    }
    os_home.map(|home| (grok_home_in(home), GrokHomeSource::HomeDefault))
}

/// Resolve the grok home from the environment (fresh, no cache); `None` if neither resolves.
pub fn resolve_grok_home() -> Option<PathBuf> {
    resolve_grok_home_with_source().map(|(home, _)| home)
}

/// [`resolve_grok_home`] plus the [`GrokHomeSource`] the path came from.
///
/// `$DEEPSEEK_BUILD_HOME` is the only environment override; other names are
/// ignored by design (see the `legacy_home_env_is_ignored` test).
pub fn resolve_grok_home_with_source() -> Option<(PathBuf, GrokHomeSource)> {
    let env_override = std::env::var_os(HOME_ENV_VAR);
    resolve_grok_home_from(env_override.as_deref(), home_dir().as_deref())
}

/// The default `<home>/.deepseek-build`, used when the env overrides are unset.
pub fn default_grok_home() -> PathBuf {
    grok_home_in(&home_dir().unwrap_or_else(|| PathBuf::from(".")))
}

/// The grok home, created if missing and cached for the process; falls back to
/// [`default_grok_home`] when neither `$DEEPSEEK_BUILD_HOME` nor a home resolves.
pub fn grok_home() -> PathBuf {
    static DEEPSEEK_BUILD_HOME: OnceLock<PathBuf> = OnceLock::new();
    DEEPSEEK_BUILD_HOME
        .get_or_init(|| {
            let home = resolve_grok_home().unwrap_or_else(default_grok_home);
            if let Err(err) = std::fs::create_dir_all(&home) {
                tracing::warn!(path = %home.display(), %err, "failed to create the deepseek-build home directory");
            }
            home
        })
        .clone()
}

/// Like [`grok_home`], but `None` when no home resolves (no cwd fallback).
pub fn user_grok_home() -> Option<PathBuf> {
    resolve_grok_home().is_some().then(grok_home)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::ffi::OsString;

    #[test]
    fn env_wins_over_os_home() {
        let resolved =
            resolve_grok_home_from(Some(OsStr::new("/custom/home")), Some(Path::new("/home/u")));
        assert_eq!(
            resolved,
            Some((PathBuf::from("/custom/home"), GrokHomeSource::EnvOverride))
        );
    }

    #[test]
    fn env_used_verbatim_even_when_it_exists() {
        // A real, existing dir whose canonical form differs (macOS symlinks
        // `/var` -> `/private/var`): the env value must come back unchanged.
        let tmp = tempfile::tempdir().unwrap();
        let resolved = resolve_grok_home_from(Some(tmp.path().as_os_str()), None);
        assert_eq!(
            resolved,
            Some((tmp.path().to_path_buf(), GrokHomeSource::EnvOverride))
        );
    }

    #[test]
    fn empty_env_falls_through_to_os_home() {
        let tmp = tempfile::tempdir().unwrap();
        let resolved = resolve_grok_home_from(Some(&OsString::new()), Some(tmp.path()));
        assert_eq!(
            resolved,
            Some((
                dunce::canonicalize(tmp.path())
                    .unwrap()
                    .join(".deepseek-build"),
                GrokHomeSource::HomeDefault
            ))
        );
    }

    #[test]
    fn legacy_config_dir_default_is_not_used_for_new_home() {
        // Sanity: the resolved default is the rebranded dir, not `~/.grok`.
        let home = default_grok_home();
        assert!(home.ends_with(".deepseek-build"));
    }

    #[test]
    fn default_grok_home_has_no_verbatim_prefix() {
        // The reason we canonicalize via dunce: std::fs::canonicalize yields
        // `\\?\` verbatim paths on Windows that break git and byte-exact
        // comparisons. No-op assertion on Unix.
        let home = default_grok_home();
        assert!(!home.to_string_lossy().starts_with(r"\\?\"));
        assert!(home.ends_with(".deepseek-build"));
    }

    #[test]
    fn none_when_nothing_resolves() {
        assert_eq!(
            resolve_grok_home_from(/* grok_home_env */ None, /* os_home */ None),
            None
        );
    }

    /// Any older home env name — the pre-rebrand `GROK_HOME` above all — must be
    /// ignored; only `$DEEPSEEK_BUILD_HOME` can force an override.
    #[test]
    fn legacy_home_env_is_ignored() {
        static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        let legacy = tmp.path().join("legacy-home");
        let primary = tmp.path().join("primary-home");
        let prev_primary = std::env::var_os(HOME_ENV_VAR);
        let prev_legacy = std::env::var_os("GROK_HOME");
        // Only the legacy name set: resolution must not produce an env override.
        unsafe {
            std::env::remove_var(HOME_ENV_VAR);
            std::env::set_var("GROK_HOME", &legacy);
        }
        let from_legacy = resolve_grok_home_with_source();
        // The branded name still works.
        unsafe { std::env::set_var(HOME_ENV_VAR, &primary) };
        let from_primary = resolve_grok_home_with_source();
        unsafe {
            match prev_primary {
                Some(v) => std::env::set_var(HOME_ENV_VAR, v),
                None => std::env::remove_var(HOME_ENV_VAR),
            }
            match prev_legacy {
                Some(v) => std::env::set_var("GROK_HOME", v),
                None => std::env::remove_var("GROK_HOME"),
            }
        }
        assert_ne!(
            from_legacy.as_ref().map(|(p, _)| p),
            Some(&legacy),
            "the legacy name must not be read"
        );
        assert!(
            from_legacy
                .as_ref()
                .is_none_or(|(_, src)| *src != GrokHomeSource::EnvOverride),
            "the legacy name must never yield an env override"
        );
        assert_eq!(
            from_primary,
            Some((primary, GrokHomeSource::EnvOverride)),
            "the branded name is the only override"
        );
    }
}
