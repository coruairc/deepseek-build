//! Inert authentication primitives.
//!
//! The interactive OIDC/OAuth2/device-code network stack (discovery, PKCE,
//! loopback callback, token exchange, JWKS validation, refresh) has been
//! removed. Authentication is API-key driven; the types and pure helpers that
//! the rest of the crate (and downstream crates) depend on are retained here,
//! with every network-capable entry point stubbed.

use super::config::{ForceLoginTeam, GrokComConfig, OidcAuthConfig};
use super::{AuthManager, GrokAuth};
use std::sync::Arc;

/// Authentication errors that survive the removal of the interactive flow.
/// Kept as a public-in-crate enum so callers can pattern-match and downcast.
#[derive(Debug, Clone, thiserror::Error)]
pub enum OidcError {
    #[error("OIDC not configured")]
    NotConfigured,
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

/// No-op retained for signature compatibility; never attaches headers.
pub fn with_alpha_test_key(builder: reqwest::RequestBuilder, url: &str) -> reqwest::RequestBuilder {
    let _ = url;
    builder
}

pub fn is_configured(config: &GrokComConfig) -> bool {
    config.oidc.is_some()
}

/// Peek at the unverified access token JWT to extract the `principal_type`,
/// `principal_id`, and `team_id`. Pure, no network.
pub fn peek_access_token_principal(access_token: &str) -> Option<(String, String, Option<String>)> {
    #[derive(serde::Deserialize)]
    struct MinimalClaims {
        #[serde(default, alias = "principalType")]
        principal_type: Option<String>,
        #[serde(default, alias = "principalId")]
        principal_id: Option<String>,
        #[serde(default)]
        team_id: Option<String>,
    }
    let token_data =
        jsonwebtoken::dangerous::insecure_decode::<MinimalClaims>(access_token).ok()?;
    let pt = token_data.claims.principal_type?;
    let pid = token_data.claims.principal_id?;
    if pt.is_empty() || pid.is_empty() {
        return None;
    }
    let tid = token_data.claims.team_id.filter(|s| !s.is_empty());
    Some((pt, pid, tid))
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

/// Outcome of a pure OIDC token refresh (no AuthManager mutations).
/// Retained for source compatibility with the refresh chain; the OIDC
/// network path is removed, so exchanges always fail.
pub enum OidcRefreshResult {
    /// Fresh token obtained. Caller must persist.
    Success(Box<GrokAuth>),
    /// Terminal error from the IdP, already classified into a reason.
    TerminalError {
        reason: crate::error::RefreshTokenFailedReason,
    },
    /// Non-terminal failure (network/authority unavailable).
    Failed { network_unreachable: bool },
}

/// OIDC refresh network stack removed; always reports a non-terminal failure.
pub async fn oidc_token_exchange(_auth: &GrokAuth) -> OidcRefreshResult {
    OidcRefreshResult::Failed {
        network_unreachable: false,
    }
}

/// Interactive login network stack removed.
pub async fn run_login_flow(
    _config: &GrokComConfig,
    _auth_manager: &Arc<AuthManager>,
    _channels: Option<crate::flow::AuthChannels>,
) -> anyhow::Result<(GrokAuth, bool)> {
    Err(anyhow::Error::new(OidcError::NotConfigured))
}

/// Interactive login network stack removed.
pub async fn run_login_flow_with_config(
    _oidc: &OidcAuthConfig,
    _auth_manager: &Arc<AuthManager>,
    _channels: Option<crate::flow::AuthChannels>,
) -> anyhow::Result<(GrokAuth, bool)> {
    Err(anyhow::Error::new(OidcError::NotConfigured))
}
