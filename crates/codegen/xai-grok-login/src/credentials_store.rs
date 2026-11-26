//! Local API-key credential store.
//!
//! Key precedence is: `DEEPSEEK_API_KEY`, then `DEEPSEEK_BUILD_API_KEY`, then the
//! saved file. The file lives at `<deepseek-build home>/credentials.json`, is
//! written atomically, and must be owner-only (`0600` on Unix). A group/other
//! accessible file is refused rather than silently accepted.
//!
//! The value is never logged, displayed, or embedded in an error message. Use
//! [`Credentials::expose_secret`] at the single point that needs the raw key.

use std::collections::BTreeMap;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// File name inside the deepseek-build home.
pub const CREDENTIALS_FILE_NAME: &str = "credentials.json";
/// Key id inside `api_keys`, so the shape can grow to other providers.
pub const DEEPSEEK_API_KEY_ID: &str = "deepseek";
/// On-disk schema version.
pub const CREDENTIALS_VERSION: u32 = 1;

/// A loaded credential. Deliberately has no `Display` and a redacting `Debug`.
#[derive(Clone, PartialEq, Eq)]
pub struct Credentials {
    api_key: String,
}

impl Credentials {
    fn new(api_key: String) -> Self {
        Self { api_key }
    }

    /// The raw key. Callers must not log, format, or persist it anywhere except
    /// the request that needs it.
    pub fn expose_secret(&self) -> &str {
        &self.api_key
    }
}

impl fmt::Debug for Credentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Credentials { api_key: \"[REDACTED]\" }")
    }
}

/// Errors name the path and the fix; they never carry the key.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum CredentialsError {
    #[error("failed to read credentials file {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("failed to write credentials file {path}: {source}")]
    Write {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error(
        "refusing to use credentials file {path}: permissions {mode:04o} allow group/other access; run `chmod 600 {path}`"
    )]
    InsecurePermissions { path: PathBuf, mode: u32 },

    #[error("credentials file {path} is not valid JSON: {source}")]
    Malformed {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },

    #[error("credentials file {path} uses unsupported version {version}")]
    UnsupportedVersion { path: PathBuf, version: u32 },

    #[error("saved API key must not be empty")]
    EmptyKey,

    #[error("saved API key must not contain leading or trailing whitespace")]
    SurroundingWhitespace,

    #[error("saved API key must not contain newlines or control characters")]
    ControlCharacter,
}

#[derive(Serialize, Deserialize)]
struct CredentialsFile {
    version: u32,
    #[serde(default)]
    api_keys: BTreeMap<String, String>,
}

/// Path of the credentials file under the resolved deepseek-build home.
pub fn credentials_path() -> PathBuf {
    credentials_path_in(&xai_grok_shell_base::util::grok_home::grok_home())
}

/// Path of the credentials file under an explicit home directory.
pub fn credentials_path_in(home: &Path) -> PathBuf {
    home.join(CREDENTIALS_FILE_NAME)
}

/// Load the saved credential, if any. Returns `Ok(None)` when the file does not
/// exist or holds no DeepSeek key.
pub fn load_credentials() -> Result<Option<Credentials>, CredentialsError> {
    load_credentials_from(&credentials_path())
}

/// Load from an explicit path (used by tests and by callers with a pinned home).
pub fn load_credentials_from(path: &Path) -> Result<Option<Credentials>, CredentialsError> {
    let metadata = match std::fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(source) if source.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(source) => {
            return Err(CredentialsError::Read {
                path: path.to_path_buf(),
                source,
            });
        }
    };

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        let mode = metadata.permissions().mode() & 0o777;
        if mode & 0o077 != 0 {
            return Err(CredentialsError::InsecurePermissions {
                path: path.to_path_buf(),
                mode,
            });
        }
    }
    #[cfg(not(unix))]
    let _ = metadata;

    let contents = std::fs::read_to_string(path).map_err(|source| CredentialsError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    // An empty file is treated as "no saved key" rather than malformed JSON.
    if contents.trim().is_empty() {
        return Ok(None);
    }

    let file: CredentialsFile =
        serde_json::from_str(&contents).map_err(|source| CredentialsError::Malformed {
            path: path.to_path_buf(),
            source,
        })?;
    if file.version != CREDENTIALS_VERSION {
        return Err(CredentialsError::UnsupportedVersion {
            path: path.to_path_buf(),
            version: file.version,
        });
    }

    let Some(api_key) = file.api_keys.get(DEEPSEEK_API_KEY_ID) else {
        return Ok(None);
    };
    validate_key(api_key)?;
    Ok(Some(Credentials::new(api_key.clone())))
}

/// Read only the saved key. The caller must already have established that
/// neither environment variable is set.
pub fn read_saved_api_key() -> Result<Option<String>, CredentialsError> {
    Ok(load_credentials()?.map(|credentials| credentials.api_key))
}

/// Validate, then atomically write the key with owner-only permissions.
pub fn save_api_key(key: &str) -> Result<PathBuf, CredentialsError> {
    let path = credentials_path();
    save_api_key_to(&path, key)?;
    Ok(path)
}

/// Save to an explicit path (used by tests).
pub fn save_api_key_to(path: &Path, key: &str) -> Result<(), CredentialsError> {
    validate_key(key)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|source| CredentialsError::Write {
            path: path.to_path_buf(),
            source,
        })?;
    }

    // Preserve any other provider entries already in the file. Permission
    // problems are irrelevant here: the replacement file is written 0600.
    let mut api_keys = BTreeMap::new();
    if let Ok(contents) = std::fs::read_to_string(path)
        && let Ok(existing) = serde_json::from_str::<CredentialsFile>(&contents)
    {
        api_keys = existing.api_keys;
    }
    api_keys.insert(DEEPSEEK_API_KEY_ID.to_string(), key.to_string());

    let file = CredentialsFile {
        version: CREDENTIALS_VERSION,
        api_keys,
    };
    let json =
        serde_json::to_string_pretty(&file).map_err(|source| CredentialsError::Malformed {
            path: path.to_path_buf(),
            source,
        })?;

    xai_grok_config::fs_atomic::write_user_file_atomically(path, &json, Some(0o600)).map_err(
        |source| CredentialsError::Write {
            path: path.to_path_buf(),
            source,
        },
    )?;

    // Re-assert after the rename (covers filesystems that ignore the create mode).
    if let Err(source) = xai_grok_shell_base::util::secure_file::ensure_owner_only_permissions(path)
    {
        return Err(CredentialsError::Write {
            path: path.to_path_buf(),
            source,
        });
    }
    Ok(())
}

fn validate_key(key: &str) -> Result<(), CredentialsError> {
    if key.is_empty() {
        return Err(CredentialsError::EmptyKey);
    }
    if key.trim() != key {
        return Err(CredentialsError::SurroundingWhitespace);
    }
    if key.chars().any(char::is_control) {
        return Err(CredentialsError::ControlCharacter);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt as _;

    fn mode_of(path: &Path) -> u32 {
        std::fs::metadata(path)
            .expect("metadata")
            .permissions()
            .mode()
            & 0o777
    }

    #[test]
    fn round_trips_the_saved_key() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = credentials_path_in(dir.path());

        save_api_key_to(&path, "sk-test-key-123").expect("save");
        let creds = load_credentials_from(&path)
            .expect("load")
            .expect("some credentials");
        assert_eq!(creds.expose_secret(), "sk-test-key-123");
    }

    #[test]
    fn writes_exactly_0600() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = credentials_path_in(dir.path());

        save_api_key_to(&path, "sk-test-key-123").expect("save");
        assert_eq!(mode_of(&path), 0o600);
    }

    #[test]
    fn refuses_group_or_other_readable_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = credentials_path_in(dir.path());
        std::fs::write(&path, r#"{"version":1,"api_keys":{"deepseek":"sk-test"}}"#).expect("seed");
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).expect("chmod");

        let err = load_credentials_from(&path).expect_err("must refuse 0644");
        let msg = err.to_string();
        assert!(matches!(err, CredentialsError::InsecurePermissions { .. }));
        assert!(msg.contains("chmod 600"), "message names the fix: {msg}");
        assert!(
            msg.contains(&path.display().to_string()),
            "message names the path"
        );
        assert!(!msg.contains("sk-test"), "message must not leak the key");
    }

    #[test]
    fn atomic_write_leaves_no_temp_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = credentials_path_in(dir.path());

        save_api_key_to(&path, "sk-test-key-123").expect("save");
        let entries: Vec<_> = std::fs::read_dir(dir.path())
            .expect("read_dir")
            .map(|entry| entry.expect("entry").file_name())
            .collect();
        assert_eq!(
            entries,
            vec![std::ffi::OsString::from(CREDENTIALS_FILE_NAME)]
        );
    }

    #[test]
    fn malformed_json_is_a_clear_error() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = credentials_path_in(dir.path());
        std::fs::write(&path, "{not json").expect("seed");
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).expect("chmod");

        let err = load_credentials_from(&path).expect_err("must reject malformed json");
        assert!(matches!(err, CredentialsError::Malformed { .. }));
        assert!(err.to_string().contains(&path.display().to_string()));
    }

    #[test]
    fn rejects_newline_in_key() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = credentials_path_in(dir.path());

        let err = save_api_key_to(&path, "sk-test\nkey").expect_err("must reject newline");
        assert!(matches!(err, CredentialsError::ControlCharacter));
        assert!(
            !path.exists(),
            "nothing may be written on validation failure"
        );
    }

    #[test]
    fn rejects_surrounding_whitespace() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = credentials_path_in(dir.path());

        let err = save_api_key_to(&path, " sk-test ").expect_err("must reject whitespace");
        assert!(matches!(err, CredentialsError::SurroundingWhitespace));
    }

    #[test]
    fn debug_redacts_the_key() {
        let creds = Credentials::new("sk-super-secret-value".into());
        let rendered = format!("{creds:?}");
        assert!(rendered.contains("[REDACTED]"), "got {rendered}");
        assert!(
            !rendered.contains("sk-super-secret-value"),
            "Debug must not leak the key"
        );
    }
}
