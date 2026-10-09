//! Auth-backend contract tests against a mock IdP whose `/token` response is forced per case.
//! They assert the refresh outcome, the storm cap, and the `manual_auth` event emitted on the live recovery path.

use super::*;
use crate::error::RefreshTokenFailedReason;
use crate::recovery::RecoverySource;
use crate::{GrokAuth, GrokComConfig};
use chrono::{Duration, Utc};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use xai_grok_telemetry::events::{AuthTokenKind, ManualAuthReason};

/// Mock IdP: OIDC discovery, a `/token` endpoint returning a fixed `(status, body)` and counting every hit, and a `/user` endpoint.
/// `AuthManager::update` calls `/user` after a successful refresh.
/// `delay_ms` widens the in-lock window so concurrent callers queue on `refresh_lock`.
async fn start_idp(
    token_status: u16,
    token_body: String,
    hits: Arc<AtomicU32>,
    delay_ms: u64,
) -> (String, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://127.0.0.1:{}", listener.local_addr().unwrap().port());
    let disco = base.clone();

    let app = axum::Router::new()
        .route(
            "/.well-known/openid-configuration",
            axum::routing::get(move || {
                let b = disco.clone();
                async move {
                    axum::Json(serde_json::json!({
                        "authorization_endpoint": format!("{b}/authorize"),
                        "token_endpoint": format!("{b}/token"),
                    }))
                }
            }),
        )
        .route(
            "/token",
            axum::routing::post(move || {
                let hits = hits.clone();
                let body = token_body.clone();
                async move {
                    hits.fetch_add(1, Ordering::SeqCst);
                    if delay_ms > 0 {
                        tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
                    }
                    (
                        axum::http::StatusCode::from_u16(token_status).unwrap(),
                        body,
                    )
                }
            }),
        )
        .route(
            "/user",
            axum::routing::get(|| async {
                axum::Json(serde_json::json!({ "userId": "user-42", "email": "u@corp.com" }))
            }),
        );

    let handle = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (base, handle)
}

fn expired_oidc(base_url: &str) -> GrokAuth {
    GrokAuth {
        key: "expired-at".into(),
        create_time: Utc::now() - Duration::hours(2),
        user_id: "user-42".into(),
        auth_mode: crate::model::AuthMode::Oidc,
        refresh_token: Some("rt-under-test".into()),
        expires_at: Some(Utc::now() - Duration::hours(1)),
        oidc_issuer: Some(base_url.to_owned()),
        oidc_client_id: Some("client-under-test".into()),
        ..GrokAuth::test_default()
    }
}

#[derive(Debug)]
enum Expect {
    Success,
    Permanent(RefreshTokenFailedReason),
    Transient,
}
/// Consecutive transient failures self-heal up to a bound, then escalate to a non-sticky `Other` permanent failure (which ages out via the TTL).
/// A regression here would turn recoverable blips into a permanent `/login`.
#[tokio::test]
async fn auth_backend_contract_transient_failures_escalate_to_non_sticky_permanent() {
    let hits = Arc::new(AtomicU32::new(0));
    // Persistent 503: every refresh attempt is transient.
    let (base_url, server) = start_idp(503, "{}".to_string(), hits, 0).await;
    let dir = tempfile::tempdir().unwrap();
    let auth_manager = Arc::new(
        AuthManager::new(dir.path(), GrokComConfig::default()).with_proxy_base_url(&base_url),
    );
    auth_manager.hot_swap(expired_oidc(&base_url));

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

    server.abort();
}
