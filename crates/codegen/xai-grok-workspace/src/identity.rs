//! Identity of the principal that owns a workspace.
//!
//! Relocated from the former upload environment module; it is a pure serde type
//! with no network behavior.

use serde::{Deserialize, Serialize};

/// `principal_type` wire value for a team-scoped principal.
pub(crate) const PRINCIPAL_TYPE_TEAM: &str = "Team";

/// Identity of the principal that owns a workspace.
///
/// `principal_type` and `principal_id` are the OAuth wire values.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceIdentity {
    /// Stable user identifier (owner of the bearer token).
    pub user_id: String,
    /// `"User"` or `"Team"`.
    /// `None` when the auth source does not distinguish principal kinds (e.g. a local-dev bearer token).
    pub principal_type: Option<String>,
    /// Team id when `principal_type == "Team"`; otherwise `None`.
    pub principal_id: Option<String>,
}

impl WorkspaceIdentity {
    /// Construct an identity from its parts.
    pub fn new(
        user_id: impl Into<String>,
        principal_type: Option<String>,
        principal_id: Option<String>,
    ) -> Self {
        Self {
            user_id: user_id.into(),
            principal_type,
            principal_id,
        }
    }

    /// Construct a team-scoped identity (`principal_type == "Team"`, the team id in `principal_id`).
    pub fn team(user_id: impl Into<String>, team_id: impl Into<String>) -> Self {
        Self {
            user_id: user_id.into(),
            principal_type: Some(PRINCIPAL_TYPE_TEAM.to_string()),
            principal_id: Some(team_id.into()),
        }
    }

    /// Whether this identity is a team principal (`principal_type == "Team"`).
    pub(crate) fn is_team(&self) -> bool {
        self.principal_type.as_deref() == Some(PRINCIPAL_TYPE_TEAM)
    }

    /// The team id **iff** this is a team principal: `None` for `"User"` principals.
    pub(crate) fn team_id(&self) -> Option<String> {
        self.is_team().then(|| self.principal_id.clone()).flatten()
    }

    /// The user id as an `Option`, mapping the empty string to `None`.
    pub(crate) fn user_id_opt(&self) -> Option<String> {
        if self.user_id.is_empty() {
            None
        } else {
            Some(self.user_id.clone())
        }
    }
}
