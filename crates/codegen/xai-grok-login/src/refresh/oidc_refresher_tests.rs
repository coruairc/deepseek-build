//! Unit tests for [`super::oidc_refresher::OidcRefresher`].
//! They were extracted from `oidc_refresher.rs` so the implementation reads top-to-bottom.
//! The module is wired in via `#[path = "oidc_refresher_tests.rs"] mod tests;`.

use super::*;
use crate::{GrokAuth, GrokComConfig};
use chrono::{Duration, Utc};

// ── OIDC refresh E2E with mock IdP ─────────────────────────────────

/// Start a mock server that handles OIDC discovery, token refresh, and the proxy /user endpoint (called by AuthManager::update).
async fn start_mock_oidc_and_proxy() -> (String, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://127.0.0.1:{}", listener.local_addr().unwrap().port());
    let base_for_discovery = base.clone();

    let app = axum::Router::new()
        .route(
            "/.well-known/openid-configuration",
            axum::routing::get(move || {
                let b = base_for_discovery.clone();
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
            axum::routing::post(
                |body: axum::extract::Form<Vec<(String, String)>>| async move {
                    // Verify the request is a refresh_token grant.
                    let grant_type = body
                        .iter()
                        .find(|(k, _)| k == "grant_type")
                        .map(|(_, v)| v.as_str());
                    assert_eq!(
                        grant_type,
                        Some("refresh_token"),
                        "expected refresh_token grant"
                    );

                    axum::Json(serde_json::json!({
                        "access_token": "oidc-refreshed-token",
                        "refresh_token": "oidc-new-rt",
                        "expires_in": 3600,
                    }))
                },
            ),
        )
        .route(
            "/user",
            axum::routing::get(|| async {
                axum::Json(serde_json::json!({
                    "userId": "user-42",
                    "email": "test@corp.com",
                }))
            }),
        );

    let handle = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (base, handle)
}

fn write_auth_to_disk(dir: &std::path::Path, scope: &str, auth: &GrokAuth) {
    let path = dir.join("auth.json");
    let mut map = crate::read_auth_json(&path).unwrap_or_default();
    map.insert(scope.to_owned(), auth.clone());
    let json = serde_json::to_string_pretty(&map).unwrap();
    std::fs::write(&path, json).unwrap();
}
#[tokio::test]
async fn oidc_refresher_e2e_proactive_returns_cached_when_valid() {
    let (base_url, server) = start_mock_oidc_and_proxy().await;
    let dir = tempfile::tempdir().unwrap();
    let mgr = Arc::new(
        AuthManager::new(dir.path(), GrokComConfig::default()).with_proxy_base_url(&base_url),
    );

    // Seed a valid (not expired) OIDC token.
    let valid = GrokAuth {
        key: "still-valid-token".into(),
        user_id: "user-42".into(),
        email: Some("test@corp.com".into()),
        refresh_token: Some("rt".into()),
        expires_at: Some(Utc::now() + Duration::hours(1)),
        oidc_issuer: Some(base_url.clone()),
        oidc_client_id: Some("test-client".into()),
        ..GrokAuth::test_default()
    };
    mgr.hot_swap(valid);

    let refresher = OidcRefresher::new(mgr.clone());
    // PreRequest with a valid token: the refresher finds no expired_auth (token is valid) and no disk token, so it returns TransientFailure
    // The PreRequest fast-path is handled by refresh_chain (not the refresher).
    let result = refresher.refresh(RefreshReason::PreRequest).await;
    assert!(
        matches!(result, RefreshOutcome::TransientFailure { .. }),
        "PreRequest with valid token should return TransientFailure (refresh_chain handles fast-path)"
    );

    server.abort();
}
/// The IdP would return `invalid_client`, but disk auth.json already holds a valid token with a different client_id.
/// That means a sibling re-authenticated during a client rotation, so `auth()` adopts the sibling's disk token instead of failing.
#[tokio::test]
async fn oidc_refresher_e2e_invalid_client_adopts_valid_sibling_disk_token() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base_url = format!("http://127.0.0.1:{}", listener.local_addr().unwrap().port());
    let base_for_discovery = base_url.clone();

    let app = axum::Router::new()
        .route(
            "/.well-known/openid-configuration",
            axum::routing::get(move || {
                let b = base_for_discovery.clone();
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
            axum::routing::post(|| async {
                (
                    axum::http::StatusCode::UNAUTHORIZED,
                    axum::Json(serde_json::json!({
                        "error": "invalid_client",
                        "error_description": "Unknown client"
                    })),
                )
            }),
        );

    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

    let dir = tempfile::tempdir().unwrap();
    let cfg = GrokComConfig::default();
    let scope = cfg.auth_scope();
    let mgr = Arc::new(AuthManager::new(dir.path(), cfg).with_proxy_base_url(&base_url));

    // Pre-populate disk with auth that has a *different* client_id, simulating another process having re-authenticated
    let disk_auth = GrokAuth {
        key: "disk-fresh-token".into(),
        user_id: "user-42".into(),
        refresh_token: Some("rt-disk".into()),
        expires_at: Some(Utc::now() + Duration::hours(1)),
        oidc_issuer: Some(base_url.clone()),
        oidc_client_id: Some("rotated-new-client-id".into()),
        ..GrokAuth::test_default()
    };
    let mut store = std::collections::BTreeMap::new();
    store.insert(scope, disk_auth);
    let json = serde_json::to_string_pretty(&store).unwrap();
    std::fs::write(dir.path().join("auth.json"), json).unwrap();

    // In-memory auth has the OLD client_id that the server rejects.
    let expired = GrokAuth {
        key: "old-token".into(),
        create_time: Utc::now() - Duration::hours(2),
        user_id: "user-42".into(),
        refresh_token: Some("rt-old".into()),
        expires_at: Some(Utc::now() - Duration::hours(1)),
        oidc_issuer: Some(base_url.clone()),
        oidc_client_id: Some("deleted-client-id".into()),
        ..GrokAuth::test_default()
    };
    mgr.hot_swap(expired);

    // auth() picks up the valid disk token via try_use_disk_token (disk has a different, unexpired entry from a sibling process)
    mgr.set_refresher(std::sync::Arc::new(OidcRefresher::new(mgr.clone())));
    let refreshed = mgr.auth().await;
    assert!(
        refreshed.is_ok(),
        "auth() should pick up the valid disk token, got: {refreshed:?}"
    );
    assert_eq!(
        refreshed.unwrap().oidc_client_id.as_deref(),
        Some("rotated-new-client-id"),
        "should use the sibling's rotated client_id from disk"
    );

    server.abort();
}

// The standalone `try_refresh_session_token` helper once tested here was removed when refresh was centralized in `AuthManager`
// Its call sites now go through `AuthManager::auth()` / `AuthManager::unauthorized_recovery()`, both covered in `manager.rs`
// The `resolve_credentials` invariant its regression tests pinned remains covered by the `resolve_credentials_*` tests in `agent::config::tests`

/// Another process has already refreshed and written a valid token to auth.json.
/// `refresh_chain` (via `auth()`) should pick it up from disk instead of hitting the IdP.
#[tokio::test]
async fn oidc_refresh_picks_up_valid_disk_token() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = GrokComConfig::default();
    let scope = cfg.auth_scope();
    let mgr = Arc::new(AuthManager::new(dir.path(), cfg).with_proxy_base_url("http://127.0.0.1:1"));

    // Seed in-memory with an expired token (stale refresh_token).
    let expired = GrokAuth {
        key: "old-expired-token".into(),
        create_time: Utc::now() - Duration::hours(2),
        user_id: "user-42".into(),
        refresh_token: Some("stale-rt".into()),
        expires_at: Some(Utc::now() - Duration::hours(1)),
        oidc_issuer: Some("https://idp.example.com".into()),
        oidc_client_id: Some("client-1".into()),
        ..GrokAuth::test_default()
    };
    mgr.hot_swap(expired);

    // Simulate another process writing a valid token to disk.
    let fresh_on_disk = GrokAuth {
        key: "fresh-from-other-process".into(),
        user_id: "user-42".into(),
        email: Some("user@test.com".into()),
        refresh_token: Some("new-rt".into()),
        expires_at: Some(Utc::now() + Duration::hours(1)),
        oidc_issuer: Some("https://idp.example.com".into()),
        oidc_client_id: Some("client-1".into()),
        ..GrokAuth::test_default()
    };
    write_auth_to_disk(dir.path(), &scope, &fresh_on_disk);

    // Disk-token pickup is now refresh_chain's responsibility.
    // Go through auth() which calls refresh_chain.
    let result = mgr.auth().await;
    assert_eq!(
        result.unwrap().key,
        "fresh-from-other-process",
        "should use the valid token written by another process"
    );
    assert_eq!(
        mgr.current().unwrap().key,
        "fresh-from-other-process",
        "in-memory state should be updated"
    );
}
// ── Sleep-gate E2E (controllable refresher) ────────────────────────
//
// The OIDC token exchange is inert in this build (the xAI network stack was
// removed), so a mock IdP can never be reached. These tests drive the real
// sleep-gate drain with a controllable refresher instead: it signals when the
// exchange is in flight and completes only once released, so the gate/hold
// semantics under test do not depend on any network.

/// Refresher that reports the exchange as in flight and (when a release handle
/// is set) only completes once released; succeeds with a fixed rotated token.
struct SleepGateRefresher {
    in_flight: Arc<tokio::sync::Notify>,
    release: Option<Arc<tokio::sync::Notify>>,
    calls: Arc<std::sync::atomic::AtomicU32>,
}

#[async_trait::async_trait]
impl TokenRefresher for SleepGateRefresher {
    async fn refresh(&self, _reason: RefreshReason) -> RefreshOutcome {
        self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.in_flight.notify_one();
        if let Some(release) = &self.release {
            release.notified().await;
        }
        RefreshOutcome::success(GrokAuth {
            key: "oidc-refreshed-token".into(),
            auth_mode: crate::model::AuthMode::Oidc,
            refresh_token: Some("oidc-new-rt".into()),
            user_id: "user-42".into(),
            oidc_issuer: Some("https://idp.example".into()),
            expires_at: Some(Utc::now() + Duration::hours(1)),
            create_time: Utc::now(),
            ..GrokAuth::test_default()
        })
    }
}

fn expired_oidc_for() -> GrokAuth {
    GrokAuth {
        key: "old-expired-token".into(),
        create_time: Utc::now() - Duration::hours(2),
        user_id: "user-42".into(),
        refresh_token: Some("rt-valid".into()),
        expires_at: Some(Utc::now() - Duration::hours(1)),
        oidc_issuer: Some("https://idp.example".into()),
        oidc_client_id: Some("test-client".into()),
        ..GrokAuth::test_default()
    }
}

/// While sleep is imminent, `auth()` defers and the refresher is never called; after wake, the next `auth()` runs exactly one successful refresh.
#[tokio::test]
async fn sleep_gate_e2e_defers_then_recovers_on_wake() {
    use std::sync::atomic::{AtomicU32, Ordering};

    let calls = Arc::new(AtomicU32::new(0));
    let dir = tempfile::tempdir().unwrap();
    let mgr = Arc::new(AuthManager::new(dir.path(), GrokComConfig::default()));
    mgr.hot_swap(expired_oidc_for());
    mgr.set_refresher(Arc::new(SleepGateRefresher {
        in_flight: Arc::new(tokio::sync::Notify::new()),
        release: None,
        calls: calls.clone(),
    }));

    mgr.set_system_sleep_imminent(true);
    let err = mgr.auth().await.unwrap_err();
    assert!(
        matches!(
            err,
            crate::AuthError::Refresh(crate::RefreshTokenError::Transient(_))
        ),
        "gated refresh must return a transient refresh error, got {err:?}"
    );
    assert_eq!(
        calls.load(Ordering::SeqCst),
        0,
        "a deferred refresh must not start the exchange"
    );

    mgr.set_system_sleep_imminent(false);
    let fresh = mgr.auth().await.expect("refresh must succeed after wake");
    assert_eq!(fresh.key, "oidc-refreshed-token");
    assert_eq!(
        calls.load(Ordering::SeqCst),
        1,
        "exactly one refresh once the gate clears"
    );
}

/// A refresh already in flight when sleep becomes imminent runs to completion and persists its rotated token (no abort).
/// `set_system_sleep_imminent` holds the OS sleep ack until the in-flight exchange drains; the refresher here is held open until after the gate is raised, the exact window in which aborting would discard the rotated successor token.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sleep_gate_e2e_in_flight_refresh_completes_across_imminent_sleep() {
    use std::sync::atomic::{AtomicU32, Ordering};

    let calls = Arc::new(AtomicU32::new(0));
    let in_flight = Arc::new(tokio::sync::Notify::new());
    let release = Arc::new(tokio::sync::Notify::new());
    let dir = tempfile::tempdir().unwrap();
    let mgr = Arc::new(AuthManager::new(dir.path(), GrokComConfig::default()));
    mgr.hot_swap(expired_oidc_for());
    mgr.set_refresher(Arc::new(SleepGateRefresher {
        in_flight: in_flight.clone(),
        release: Some(release.clone()),
        calls: calls.clone(),
    }));

    let m = mgr.clone();
    let handle = tokio::spawn(async move { m.auth().await });

    in_flight.notified().await; // the exchange is in flight and will not finish until released

    // `set_system_sleep_imminent` holds the OS sleep ack until the in-flight refresh drains
    // Drive it off the runtime (as the real OS power-listener thread does) so the runtime can complete the refresh while it waits
    let sleeper = mgr.clone();
    let ack = std::thread::spawn(move || sleeper.set_system_sleep_imminent(true));

    // Wait for the gate to be raised so the hold is entered while the refresh is in flight
    for _ in 0..500 {
        if mgr.is_sleep_gated() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    assert!(
        mgr.is_sleep_gated(),
        "the ack thread must raise the sleep gate"
    );
    // Give the ack thread a moment to park in the drain wait before releasing the exchange
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    release.notify_one();

    let fresh = tokio::time::timeout(std::time::Duration::from_secs(5), handle)
        .await
        .expect("auth() must return")
        .unwrap()
        .expect("in-flight refresh must complete across imminent sleep");
    ack.join().expect("ack thread panicked");
    assert_eq!(calls.load(Ordering::SeqCst), 1, "exactly one exchange");
    assert_eq!(fresh.key, "oidc-refreshed-token");
    assert_eq!(
        mgr.current().map(|a| a.key),
        Some("oidc-refreshed-token".to_owned()),
        "the rotated token must be persisted, not discarded"
    );
}
// ── Transient-blip budget is per-credential ─────────────────────────

/// Minimal `AuthSnapshot` for exercising `record_transient_failure` in isolation (it never reads credential state).
struct EmptySnapshot;
impl AuthSnapshot for EmptySnapshot {
    fn current(&self) -> Option<GrokAuth> {
        None
    }
    fn expired_auth(&self) -> Option<GrokAuth> {
        None
    }
    fn read_disk_auth(&self) -> Option<GrokAuth> {
        None
    }
    fn is_expired(&self) -> bool {
        false
    }
    fn has_sendable_token(&self) -> bool {
        false
    }
}

/// A fresh credential (after re-login on this long-lived refresher) must get the full blip budget instead of inheriting a dead credential's count.
/// Otherwise a valid token could be escalated to a permanent failure early.
#[test]
fn transient_blip_budget_is_scoped_to_the_credential() {
    let refresher = OidcRefresher::new(Arc::new(EmptySnapshot));
    let key_a = Some("cred-a".to_owned());

    // Accrue blips up to just under the escalation threshold on credential A.
    for _ in 0..MAX_CONSECUTIVE_TRANSIENT_FAILURES - 1 {
        assert!(matches!(
            refresher.record_transient_failure("blip".into(), key_a.clone(), false),
            RefreshOutcome::TransientFailure { .. }
        ));
    }

    // Credential B's first blip must stay transient, not escalate to permanent.
    assert!(
        matches!(
            refresher.record_transient_failure("blip".into(), Some("cred-b".to_owned()), false),
            RefreshOutcome::TransientFailure { .. }
        ),
        "a fresh credential must not inherit a prior credential's blip count",
    );
}

/// Network-unreachable failures (DNS/connect/timeout, the post-wake offline window) must never consume the escalation budget.
/// No amount of them may produce a `PermanentFailure` ("Run `grok login`") verdict, because they prove nothing about the credential.
/// Counted failures accrued before or after are unaffected (the budget is neither consumed nor reset).
#[test]
fn network_unreachable_blips_never_escalate() {
    let refresher = OidcRefresher::new(Arc::new(EmptySnapshot));
    let key = Some("cred-a".to_owned());

    // Far more unreachable blips than the budget: all stay transient.
    for _ in 0..MAX_CONSECUTIVE_TRANSIENT_FAILURES * 3 {
        assert!(
            matches!(
                refresher.record_transient_failure("wifi down".into(), key.clone(), true),
                RefreshOutcome::TransientFailure { .. }
            ),
            "a network-unreachable failure must never escalate to permanent",
        );
    }

    // The budget was not consumed: counted (IdP-reaching) blips still get the full threshold before escalating
    for _ in 0..MAX_CONSECUTIVE_TRANSIENT_FAILURES - 1 {
        assert!(matches!(
            refresher.record_transient_failure("5xx".into(), key.clone(), false),
            RefreshOutcome::TransientFailure { .. }
        ));
    }
    assert!(
        matches!(
            refresher.record_transient_failure("5xx".into(), key.clone(), false),
            RefreshOutcome::PermanentFailure { .. }
        ),
        "counted blips must still escalate at the threshold",
    );
}
