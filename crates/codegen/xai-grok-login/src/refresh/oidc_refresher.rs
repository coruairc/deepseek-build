use std::sync::Arc;

use crate::error::RefreshTokenFailedReason;
use crate::manager::RefreshReason;

use super::{AuthSnapshot, RefreshOutcome, TokenRefresher};

#[cfg(test)]
use crate::manager::AuthManager;

/// Escalate to `PermanentFailure` after this many consecutive transient failures (then `PERMANENT_FAILURE_TTL` allows recovery).
/// Same budget as `ExternalBinaryRefresher`, whose ladder is additionally time-based (see `run_cooldown`).
/// Kept above the number of attempts `try_recover_unauthorized` makes per recovery, so one 401 recovery alone cannot escalate.
const MAX_CONSECUTIVE_TRANSIENT_FAILURES: u32 = 5;

/// Counts consecutive transient failures, scoped to the credential they accrued against.
/// The whole struct sits under one lock so the credential check, reset, and increment are a single atomic step.
#[derive(Default)]
struct TransientBudget {
    /// Credential the count belongs to.
    /// A new credential, e.g. after re-login on this long-lived refresher, resets the count so a fresh token never inherits a dead one's escalation.
    key: Option<String>,
    count: u32,
}

/// Session-token renewer for a build with no interactive issuer stack.
///
/// The xAI OIDC token exchange has been removed, so a refresh here can never
/// mint a new token. What remains is the state machine around it: adopt a
/// wire-valid token another process left on disk, and otherwise report the
/// failure transiently (escalating to a non-sticky permanent verdict after
/// [`MAX_CONSECUTIVE_TRANSIENT_FAILURES`] attempts) so the 401-recovery path
/// and the proactive loop keep working.
pub struct OidcRefresher {
    auth: Arc<dyn AuthSnapshot>,
    transient_budget: parking_lot::Mutex<TransientBudget>,
}

impl OidcRefresher {
    pub fn new(auth: Arc<dyn AuthSnapshot>) -> Self {
        Self {
            auth,
            transient_budget: parking_lot::Mutex::new(TransientBudget::default()),
        }
    }

    /// Clear the transient-failure count on refresh progress (an adopted sibling token), so later failures start from a full budget.
    fn note_refresh_progress(&self) {
        *self.transient_budget.lock() = TransientBudget::default();
    }

    fn record_transient_failure(
        &self,
        message: String,
        tried_key: Option<String>,
    ) -> RefreshOutcome {
        let escalate = {
            let mut budget = self.transient_budget.lock();
            // Reset the count when the credential changes so a fresh token never inherits a prior credential's failures
            if budget.key != tried_key {
                budget.key = tried_key.clone();
                budget.count = 0;
            }
            budget.count += 1;
            let escalate = budget.count >= MAX_CONSECUTIVE_TRANSIENT_FAILURES;
            // On escalation reset the count so the next TTL window gets the full budget (the verdict gates refresh() meanwhile)
            // The key is left in place; a retry with the same key resumes from zero, and a new key resets the budget
            if escalate {
                budget.count = 0;
            }
            escalate
        };
        if escalate {
            tracing::warn!(%message, "auth: escalating consecutive transient failures to permanent");
            RefreshOutcome::permanent(RefreshTokenFailedReason::Other, tried_key)
        } else {
            RefreshOutcome::transient(message)
        }
    }
}

#[async_trait::async_trait]
impl TokenRefresher for OidcRefresher {
    async fn refresh(&self, reason: RefreshReason) -> RefreshOutcome {
        xai_grok_telemetry::unified_log::debug(
            "oidc refresh enter",
            None,
            Some(serde_json::json!({
                "reason": format!("{reason:?}"),
                "has_current": self.auth.current().is_some(),
                "is_expired": self.auth.is_expired(),
            })),
        );

        let disk_auth = self.auth.read_disk_auth();

        // If disk holds a valid unexpired access token that differs from the in-memory one, a sibling already refreshed
        // That happened between refresh_chain step 2 (the disk check under lock) and here
        // Adopt it directly; no IdP call is needed
        if let Some(ref d) = disk_auth
            && !crate::is_expired(d)
            && self.auth.current().map(|a| a.key).as_deref() != Some(&d.key)
        {
            xai_grok_telemetry::unified_log::info(
                "oidc refresh: sibling refreshed, adopting valid disk AT",
                None,
                Some(serde_json::json!({
                    "disk_key_prefix": xai_grok_auth::bearer_suffix(&d.key),
                })),
            );
            self.note_refresh_progress();
            return RefreshOutcome::success(d.clone());
        }

        let auth = super::resolve_refresh_credential(self.auth.as_ref(), disk_auth, reason);

        let Some(auth) = auth else {
            xai_grok_telemetry::unified_log::warn(
                "oidc refresh no token available",
                None,
                Some(serde_json::json!({ "reason": format!("{reason:?}") })),
            );
            return RefreshOutcome::transient("no token with refresh_token available");
        };

        tracing::warn!(
            refresh_reason = ?reason,
            user_id = %auth.user_id,
            has_refresh_token = auth.refresh_token.is_some(),
            issuer = ?auth.oidc_issuer,
            client_id = ?auth.oidc_client_id,
            expires_at = ?auth.expires_at,
            "auth: OIDC token refresh unavailable (network stack removed)"
        );
        xai_grok_telemetry::unified_log::error(
            "oidc refresh failed",
            None,
            Some(serde_json::json!({
                "has_refresh_token": auth.refresh_token.is_some(),
                "auth_mode": format!("{:?}", auth.auth_mode),
                "issuer": auth.oidc_issuer,
                "client_id": auth.oidc_client_id,
                "expires_at": auth.expires_at.map(|e| e.to_rfc3339()),
            })),
        );
        self.record_transient_failure("OIDC token refresh failed".into(), Some(auth.key.clone()))
    }
}

#[cfg(test)]
#[path = "oidc_refresher_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "auth_backend_contract_tests.rs"]
mod auth_backend_contract_tests;
