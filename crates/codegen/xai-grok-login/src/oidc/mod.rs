//! Pure login-policy helpers for the OIDC credential path.
//!
//! The interactive OIDC/OAuth2/device-code network stack (discovery, PKCE,
//! loopback callback, token exchange, JWKS validation, refresh) has been
//! removed. Authentication is API-key driven; the pure helpers that the rest
//! of the crate depends on are retained here.

use super::config::{ForceLoginTeam, GrokComConfig};

/// Authentication errors that survive the removal of the interactive flow.
/// Kept as a public-in-crate enum so callers can pattern-match and downcast.
#[derive(Debug, Clone, thiserror::Error)]
pub enum OidcError {
    #[error(
        "This deployment requires logging into {expected}; your login returned {}",
        actual.as_deref().unwrap_or("no team principal")
    )]
    PinnedPrincipalMismatch {
        /// Pre-formatted requirement, e.g. `team <id>` or `one of teams: a, b`.
        expected: String,
        actual: Option<String>,
    },
    #[error(
        "Login is blocked by your administrator: force_login_team_uuid is an empty \
         list, so no team is permitted to sign in"
    )]
    ForceLoginNoPrincipalsAllowed,
}

/// Extract just the `principal_id` claim for `force_login_team_uuid` matching.
pub fn peek_access_token_principal_id(access_token: &str) -> Option<String> {
    #[derive(serde::Deserialize)]
    struct PrincipalIdClaim {
        #[serde(default, alias = "principalId")]
        principal_id: Option<String>,
    }
    jsonwebtoken::dangerous::insecure_decode::<PrincipalIdClaim>(access_token)
        .ok()?
        .claims
        .principal_id
        .filter(|s| !s.is_empty())
}

/// Resolved allowed-team set from `force_login_team_uuid`, or `None` (unrestricted).
pub fn resolve_login_principal_policy(
    force_login_team_uuid: Option<&ForceLoginTeam>,
) -> Option<ForceLoginTeam> {
    force_login_team_uuid.cloned()
}

pub fn login_principal_policy(cfg: &GrokComConfig) -> Option<ForceLoginTeam> {
    resolve_login_principal_policy(cfg.force_login_team_uuid.as_ref())
}

/// Reject a token whose principal isn't allowed, BEFORE persisting.
pub fn enforce_login_principal(
    policy: Option<&ForceLoginTeam>,
    actual: Option<&str>,
) -> anyhow::Result<()> {
    let allowed: &[String] = match policy {
        None => return Ok(()),
        Some(ForceLoginTeam::Single(id)) => std::slice::from_ref(id),
        Some(ForceLoginTeam::AnyOf(ids)) if ids.is_empty() => {
            tracing::warn!("auth: force_login_team_uuid is an empty list; failing closed");
            return Err(anyhow::Error::new(OidcError::ForceLoginNoPrincipalsAllowed));
        }
        Some(ForceLoginTeam::AnyOf(ids)) => ids,
    };
    if let Some(actual) = actual
        && allowed.iter().any(|a| a == actual)
    {
        return Ok(());
    }
    let expected = if let [id] = allowed {
        format!("team {id}")
    } else {
        format!("one of teams: {}", allowed.join(", "))
    };
    tracing::warn!(
        expected = %expected,
        actual = ?actual,
        "auth: login principal does not satisfy required policy; rejecting"
    );
    Err(anyhow::Error::new(OidcError::PinnedPrincipalMismatch {
        expected,
        actual: actual.map(str::to_owned),
    }))
}
