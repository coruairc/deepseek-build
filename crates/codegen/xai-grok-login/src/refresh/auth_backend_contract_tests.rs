//! Auth-backend contract tests for the session-token refresher.
//!
//! The xAI OIDC exchange has been removed, so there is no mock IdP left to
//! drive. What remains to assert is the escalation ladder: consecutive
//! transient failures self-heal up to a bound, then escalate to a non-sticky
//! `Other` permanent failure (which ages out via the TTL). A regression here
//! would turn recoverable blips into a permanent `/login`.

use super::*;
use crate::error::RefreshTokenFailedReason;
use crate::manager::AuthManager;
use crate::{GrokAuth, GrokComConfig};
use chrono::{Duration, Utc};
use std::sync::Arc;

fn expired_oidc() -> GrokAuth {
    GrokAuth {
        key: "expired-at".into(),
        create_time: Utc::now() - Duration::hours(2),
        user_id: "user-42".into(),
        auth_mode: crate::model::AuthMode::Oidc,
        refresh_token: Some("rt-under-test".into()),
        expires_at: Some(Utc::now() - Duration::hours(1)),
        oidc_issuer: Some("https://idp.example".into()),
        oidc_client_id: Some("client-under-test".into()),
        ..GrokAuth::test_default()
    }
}

#[tokio::test]
async fn auth_backend_contract_transient_failures_escalate_to_non_sticky_permanent() {
    let dir = tempfile::tempdir().unwrap();
    let auth_manager = Arc::new(
        AuthManager::new(dir.path(), GrokComConfig::default())
            .with_proxy_base_url("http://127.0.0.1:1"),
    );
    auth_manager.hot_swap(expired_oidc());

    // One refresher instance: it owns the consecutive-failure counter.
    // The budget exceeds try_recover_unauthorized's per-recovery attempts, so a single 401 recovery cannot escalate; exhaust the full budget here
    let refresher = OidcRefresher::new(auth_manager.clone());
    let mut outcomes = Vec::new();
    for _ in 0..5 {
        outcomes.push(refresher.refresh(RefreshReason::ServerRejected).await);
    }

    let Some(first) = outcomes.first() else {
        panic!("expected 5 refresh outcomes, got {outcomes:?}");
    };
    assert!(
        matches!(first, RefreshOutcome::TransientFailure { .. }),
        "first blip is transient, not a lockout: {first:?}",
    );
    let Some(fourth) = outcomes.get(3) else {
        panic!("expected 5 refresh outcomes, got {outcomes:?}");
    };
    assert!(
        matches!(fourth, RefreshOutcome::TransientFailure { .. }),
        "4th blip still under escalation budget: {fourth:?}",
    );
    match outcomes.get(4) {
        Some(RefreshOutcome::PermanentFailure { error, .. }) => {
            assert_eq!(
                error.reason,
                RefreshTokenFailedReason::Other,
                "escalation must use the generic Other reason",
            );
            assert!(
                !error.reason.is_sticky(),
                "an escalated transient must age out, not strand the user forever",
            );
        }
        other => panic!("repeated transients must escalate to a permanent Other, got {other:?}"),
    }
}
