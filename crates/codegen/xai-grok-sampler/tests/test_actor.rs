//! Integration tests for the actor and request_task layer.
//!
//! They live in `tests/` because they need a real `tokio::runtime` and a mock axum HTTP server for the `SamplingClient` to talk to.
//! Happy-path SSE payloads come from `xai_grok_test_support::sse`.

use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use axum::Router;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::sse::{Event, Sse};
use axum::routing::post;
use futures_util::stream::{self, StreamExt};
use serde_json::json;
use tokio::net::TcpListener;
use tokio::sync::{mpsc, oneshot};

use xai_grok_sampler::{
    ApiBackend, RequestId, RetryPolicy, SamplerActor, SamplerConfig, SamplingChannel,
    SamplingErrorKind, SamplingEvent, StripReason,
};
use xai_grok_sampling_types::{
    ConversationItem, ConversationRequest, DoomLoopRecoveryPolicy, INVALID_IMAGE_ERROR_CODE,
    SyntheticReason, UserItem,
};
use xai_grok_test_support::{SseEvent, sse};

// ---------------------------------------------------------------------------
// Mock server harness
// ---------------------------------------------------------------------------

struct MockServer {
    addr: SocketAddr,
    shutdown_tx: oneshot::Sender<()>,
}

impl MockServer {
    async fn spawn(app: Router) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
        tokio::spawn(async move {
            let _ = axum::serve(listener, app)
                .with_graceful_shutdown(async move {
                    let _ = shutdown_rx.await;
                })
                .await;
        });
        // Give the server a moment to start.
        tokio::time::sleep(Duration::from_millis(20)).await;
        Self { addr, shutdown_tx }
    }

    fn base_url(&self) -> String {
        format!("http://{}/v1", self.addr)
    }

    fn shutdown(self) {
        let _ = self.shutdown_tx.send(());
    }
}

// ---------------------------------------------------------------------------
// Config + request helpers
// ---------------------------------------------------------------------------

fn test_config(base_url: String, model: &str) -> SamplerConfig {
    SamplerConfig {
        api_key: Some("test-key".into()),
        base_url,
        model: model.into(),
        max_completion_tokens: Some(1024),
        context_window: 128_000,
        // Keep retries minimal so tests don't take forever.
        max_retries: Some(2),
        idle_timeout_secs: Some(30),
        ..Default::default()
    }
}

fn user_request(text: &str) -> ConversationRequest {
    ConversationRequest {
        items: vec![ConversationItem::User(UserItem {
            content: vec![xai_grok_sampling_types::ContentPart::Text {
                text: std::sync::Arc::<str>::from(text),
            }],
            synthetic_reason: SyntheticReason::Human,
            ..Default::default()
        })],
        ..Default::default()
    }
}

// ---------------------------------------------------------------------------
// SSE generators
// ---------------------------------------------------------------------------

/// Render test-helper [`SseEvent`]s (optional `event:` name and `data:`) as axum SSE events for this file's router-based harness.
fn sse_events_to_axum(events: Vec<SseEvent>) -> Vec<Event> {
    events
        .into_iter()
        .map(|e| {
            let ev = Event::default().data(e.data);
            match e.event {
                Some(name) => ev.event(name),
                None => ev,
            }
        })
        .collect()
}

fn text_chunk_event(content: &str, finish: bool) -> Event {
    let chunk = json!({
        "id": "chatcmpl-test",
        "object": "chat.completion.chunk",
        "created": 0,
        "model": "test-model",
        "choices": [{
            "index": 0,
            "delta": { "role": "assistant", "content": content },
            "finish_reason": if finish { json!("stop") } else { json!(null) }
        }]
    });
    Event::default().data(chunk.to_string())
}

// ---------------------------------------------------------------------------
// Actor lifecycle
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn spawn_then_active_count_zero_then_cancel_unknown_is_noop() {
    let (event_tx, _event_rx) = mpsc::unbounded_channel();
    let cfg = test_config("http://127.0.0.1:0/v1".into(), "test-model");
    let handle = SamplerActor::spawn(cfg, RetryPolicy::default(), event_tx);
    assert_eq!(handle.active_count().await, 0);
    handle.cancel(RequestId::from("nonexistent"));
    assert_eq!(handle.active_count().await, 0);
}

// ---------------------------------------------------------------------------
// Submit + event flow
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn submit_emits_started_first_token_channel_completed() {
    let app = Router::new().route(
        "/v1/chat/completions",
        post(|| async {
            let events = sse::chat_completion_events("hello world", "test-model");
            Sse::new(stream::iter(
                events.into_iter().map(Ok::<_, std::convert::Infallible>),
            ))
        }),
    );
    let server = MockServer::spawn(app).await;
    let (event_tx, mut event_rx) = mpsc::unbounded_channel();
    let cfg = test_config(server.base_url(), "test-model");
    let handle = SamplerActor::spawn(cfg, RetryPolicy::default(), event_tx);

    let rid = RequestId::from("req-1");
    handle.submit(rid.clone(), user_request("hi"));

    let events = drain_until_terminal(&mut event_rx, Duration::from_secs(5)).await;
    server.shutdown();

    assert!(matches!(events[0], SamplingEvent::StreamStarted { .. }));
    assert!(
        events
            .iter()
            .any(|e| matches!(e, SamplingEvent::FirstToken { .. }))
    );

    let texts: Vec<&str> = events
        .iter()
        .filter_map(|e| match e {
            SamplingEvent::ChannelToken {
                channel: SamplingChannel::Text,
                text,
                ..
            } => Some(text.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(texts.join(""), "hello world");

    match events.last().unwrap() {
        SamplingEvent::Completed {
            request_id,
            response,
            ..
        } => {
            assert_eq!(request_id, &rid);
            if let Some(a) = response.assistant() {
                assert_eq!(a.content.as_ref(), "hello world");
            } else {
                panic!("expected Assistant message");
            }
        }
        other => panic!("expected Completed, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// submit_and_collect
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn submit_and_collect_returns_response() {
    let app = Router::new().route(
        "/v1/chat/completions",
        post(|| async {
            let events = sse::chat_completion_events("collected response", "test-model");
            Sse::new(stream::iter(
                events.into_iter().map(Ok::<_, std::convert::Infallible>),
            ))
        }),
    );
    let server = MockServer::spawn(app).await;
    let (event_tx, _event_rx) = mpsc::unbounded_channel();
    let cfg = test_config(server.base_url(), "test-model");
    let handle = SamplerActor::spawn(cfg, RetryPolicy::default(), event_tx);

    let rid = RequestId::from("req-collect");
    let result = handle
        .submit_and_collect(rid, user_request("hi"))
        .await
        .expect("collected ok");
    server.shutdown();

    let (response, _metrics) = result;
    let a = response.assistant().expect("assistant item present");
    assert_eq!(a.content.as_ref(), "collected response");
}

// ---------------------------------------------------------------------------
// Cancellation
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cancel_in_flight_request_terminates_task() {
    // The server yields one chunk then hangs
    let app = Router::new().route(
        "/v1/chat/completions",
        post(|| async {
            let stream = stream::iter(vec![Ok::<_, std::convert::Infallible>(text_chunk_event(
                "starting", false,
            ))])
            .chain(stream::pending());
            Sse::new(stream)
        }),
    );
    let server = MockServer::spawn(app).await;
    let (event_tx, mut event_rx) = mpsc::unbounded_channel();
    let cfg = test_config(server.base_url(), "test-model");
    let handle = SamplerActor::spawn(cfg, RetryPolicy::default(), event_tx);

    let rid = RequestId::from("req-cancel");
    handle.submit(rid.clone(), user_request("hi"));

    // Wait for the first token to arrive so we know the request is in flight.
    let _ = await_event_matching(
        &mut event_rx,
        |e| matches!(e, SamplingEvent::FirstToken { .. }),
        Duration::from_secs(5),
    )
    .await
    .expect("first token");

    handle.cancel(rid.clone());

    // Expect a Failed event with the cancellation message.
    let failed = await_event_matching(
        &mut event_rx,
        |e| matches!(e, SamplingEvent::Failed { .. }),
        Duration::from_secs(5),
    )
    .await
    .expect("Failed event after cancel");

    if let SamplingEvent::Failed { error, .. } = failed {
        assert!(error.message.contains("cancelled"));
    }
    assert_eq!(handle.active_count().await, 0);
    server.shutdown();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn dropping_submit_and_collect_fails_the_request_without_calling_cancel() {
    let app = Router::new().route(
        "/v1/chat/completions",
        post(|| async {
            let stream = stream::iter(vec![Ok::<_, std::convert::Infallible>(text_chunk_event(
                "starting", false,
            ))])
            .chain(stream::pending());
            Sse::new(stream)
        }),
    );
    let server = MockServer::spawn(app).await;
    let (event_tx, mut event_rx) = mpsc::unbounded_channel();
    let cfg = test_config(server.base_url(), "test-model");
    let handle = SamplerActor::spawn(cfg, RetryPolicy::default(), event_tx);
    let rid = RequestId::from("req-drop");
    let collect = {
        let handle = handle.clone();
        let rid = rid.clone();
        tokio::spawn(async move {
            let _ = handle.submit_and_collect(rid, user_request("hi")).await;
        })
    };
    let _ = await_event_matching(
        &mut event_rx,
        |e| matches!(e, SamplingEvent::FirstToken { .. }),
        Duration::from_secs(5),
    )
    .await
    .expect("first token");
    collect.abort();
    let failed = await_event_matching(
        &mut event_rx,
        |e| matches!(e, SamplingEvent::Failed { .. }),
        Duration::from_secs(5),
    )
    .await
    .expect("Failed event after the collect future is dropped");
    if let SamplingEvent::Failed { error, .. } = failed {
        assert!(error.message.contains("cancelled"));
    }
    assert_eq!(handle.active_count().await, 0);
    server.shutdown();
}

// ---------------------------------------------------------------------------
// Concurrent requests
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn two_concurrent_requests_complete_with_correct_request_ids() {
    let counter = Arc::new(AtomicU32::new(0));
    let counter_handler = Arc::clone(&counter);
    let app = Router::new().route(
        "/v1/chat/completions",
        post(move || {
            let counter = Arc::clone(&counter_handler);
            async move {
                let n = counter.fetch_add(1, Ordering::SeqCst);
                let events = sse::chat_completion_events(&format!("response-{n}"), "test-model");
                Sse::new(stream::iter(
                    events.into_iter().map(Ok::<_, std::convert::Infallible>),
                ))
            }
        }),
    );
    let server = MockServer::spawn(app).await;
    let (event_tx, mut event_rx) = mpsc::unbounded_channel();
    let cfg = test_config(server.base_url(), "test-model");
    let handle = SamplerActor::spawn(cfg, RetryPolicy::default(), event_tx);

    let rid_a = RequestId::from("req-a");
    let rid_b = RequestId::from("req-b");
    handle.submit(rid_a.clone(), user_request("a"));
    handle.submit(rid_b.clone(), user_request("b"));

    // Drain until we see Completed for both.
    let mut completed_a = false;
    let mut completed_b = false;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    while !(completed_a && completed_b) {
        let now = tokio::time::Instant::now();
        if now >= deadline {
            panic!(
                "timed out waiting for both requests to complete: a={completed_a}, b={completed_b}"
            );
        }
        let remaining = deadline - now;
        match tokio::time::timeout(remaining, event_rx.recv()).await {
            Ok(Some(SamplingEvent::Completed { request_id, .. })) if request_id == rid_a => {
                completed_a = true;
            }
            Ok(Some(SamplingEvent::Completed { request_id, .. })) if request_id == rid_b => {
                completed_b = true;
            }
            Ok(Some(_)) => {}
            Ok(None) => panic!("event channel closed"),
            Err(_) => panic!("timeout"),
        }
    }
    server.shutdown();
}

// ---------------------------------------------------------------------------
// Retry on transient transport error
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn retries_on_500_then_succeeds() {
    let counter = Arc::new(AtomicU32::new(0));
    let counter_handler = Arc::clone(&counter);
    let app = Router::new().route(
        "/v1/chat/completions",
        post(move || {
            let counter = Arc::clone(&counter_handler);
            async move {
                let n = counter.fetch_add(1, Ordering::SeqCst);
                if n == 0 {
                    // First attempt: server error.
                    Err::<Sse<_>, (StatusCode, String)>((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        json!({ "error": { "message": "transient" } }).to_string(),
                    ))
                } else {
                    // Subsequent attempts: success.
                    let events = sse::chat_completion_events("ok", "test-model");
                    Ok(Sse::new(stream::iter(
                        events.into_iter().map(Ok::<_, std::convert::Infallible>),
                    )))
                }
            }
        }),
    );
    let server = MockServer::spawn(app).await;
    let (event_tx, mut event_rx) = mpsc::unbounded_channel();
    // Lots of retries available; backoff is jittered around 2s on first retry, so this test takes a bit to run
    let cfg = test_config(server.base_url(), "test-model");
    let handle = SamplerActor::spawn(cfg, RetryPolicy::default(), event_tx);

    let rid = RequestId::from("req-retry");
    handle.submit(rid.clone(), user_request("hi"));

    let events = drain_until_terminal(&mut event_rx, Duration::from_secs(15)).await;
    server.shutdown();

    let saw_retrying = events
        .iter()
        .any(|e| matches!(e, SamplingEvent::Retrying { .. }));
    assert!(saw_retrying, "expected at least one Retrying event");

    match events.last().unwrap() {
        SamplingEvent::Completed { response, .. } => {
            if let Some(a) = response.assistant() {
                assert_eq!(a.content.as_ref(), "ok");
            }
        }
        other => panic!("expected Completed after retry, got {other:?}"),
    }

    assert!(
        counter.load(Ordering::SeqCst) >= 2,
        "server hit at least twice"
    );
}

/// A coded `invalid_image` 400 strips the image, emits ServerRejected, retries, and completes.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn invalid_image_code_strips_and_retries() {
    const IMAGE_URI: &str = "data:image/png;base64,cG9pc29uZWQ=";
    let bodies = Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
    let bodies_handler = Arc::clone(&bodies);
    let app = Router::new().route(
        "/v1/chat/completions",
        post(move |body: String| {
            let bodies = Arc::clone(&bodies_handler);
            async move {
                let n = {
                    let mut b = bodies.lock().unwrap();
                    b.push(body);
                    b.len()
                };
                if n == 1 {
                    // The FLAT envelope the xAI API's non-stream rejections actually use; the message alone must not matter
                    Err::<Sse<_>, (StatusCode, String)>((
                        StatusCode::BAD_REQUEST,
                        json!({
                            "code": INVALID_IMAGE_ERROR_CODE,
                            "error": "some future wording without the legacy phrase",
                        })
                        .to_string(),
                    ))
                } else {
                    let events = sse::chat_completion_events("recovered", "test-model");
                    Ok(Sse::new(stream::iter(
                        events.into_iter().map(Ok::<_, std::convert::Infallible>),
                    )))
                }
            }
        }),
    );
    let server = MockServer::spawn(app).await;
    let (event_tx, mut event_rx) = mpsc::unbounded_channel();
    let handle = SamplerActor::spawn(
        test_config(server.base_url(), "test-model"),
        RetryPolicy::default(),
        event_tx,
    );

    let mut request = user_request("what is in this image?");
    if let Some(ConversationItem::User(u)) = request.items.first_mut() {
        u.add_image(IMAGE_URI);
    }
    handle.submit(RequestId::from("req-image-code-strip"), request);

    let events = drain_until_terminal(&mut event_rx, Duration::from_secs(15)).await;
    server.shutdown();

    assert!(
        events.iter().any(|e| match e {
            SamplingEvent::ImagesStripped {
                stripped_urls,
                reason: xai_grok_sampler::StripReason::ServerRejected,
                ..
            } => stripped_urls.len() == 1 && stripped_urls[0].as_ref() == IMAGE_URI,
            _ => false,
        }),
        "expected server-rejected ImagesStripped carrying the poisoned URL, got {events:?}"
    );
    assert!(
        matches!(events.last(), Some(SamplingEvent::Completed { .. })),
        "expected Completed after strip-retry"
    );

    let bodies = bodies.lock().unwrap();
    assert_eq!(bodies.len(), 2, "one rejection, one strip-retry");
    assert!(bodies[0].contains(IMAGE_URI), "first attempt sends image");
    assert!(
        !bodies[1].contains(IMAGE_URI),
        "strip-retry must not resend the image"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn legacy_phrase_400_strips_as_heuristic() {
    const IMAGE_URI: &str = "data:image/png;base64,cG9pc29uZWQ=";
    let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let counter_handler = Arc::clone(&counter);
    let app = Router::new().route(
        "/v1/chat/completions",
        post(move || {
            let counter = Arc::clone(&counter_handler);
            async move {
                if counter.fetch_add(1, Ordering::SeqCst) == 0 {
                    Err::<Sse<_>, (StatusCode, String)>((
                        StatusCode::BAD_REQUEST,
                        json!({
                            "error": {
                                "message": "Could not process image",
                                "type": "invalid_request_error",
                            }
                        })
                        .to_string(),
                    ))
                } else {
                    let events = sse::chat_completion_events("recovered", "test-model");
                    Ok(Sse::new(stream::iter(
                        events.into_iter().map(Ok::<_, std::convert::Infallible>),
                    )))
                }
            }
        }),
    );
    let server = MockServer::spawn(app).await;
    let (event_tx, mut event_rx) = mpsc::unbounded_channel();
    let handle = SamplerActor::spawn(
        test_config(server.base_url(), "test-model"),
        RetryPolicy::default(),
        event_tx,
    );

    let mut request = user_request("what is in this image?");
    if let Some(ConversationItem::User(u)) = request.items.first_mut() {
        u.add_image(IMAGE_URI);
    }
    handle.submit(RequestId::from("req-legacy-phrase-strip"), request);

    let events = drain_until_terminal(&mut event_rx, Duration::from_secs(15)).await;
    server.shutdown();

    assert!(
        events.iter().any(|e| matches!(
            e,
            SamplingEvent::ImagesStripped {
                reason: StripReason::PayloadHeuristic,
                ..
            }
        )),
        "codeless legacy-phrase 400 must strip as PayloadHeuristic, got {events:?}"
    );
    assert!(
        !events.iter().any(|e| matches!(
            e,
            SamplingEvent::ImagesStripped {
                reason: StripReason::ServerRejected,
                ..
            }
        )),
        "no deterministic code, so never ServerRejected: {events:?}"
    );
    assert!(
        matches!(events.last(), Some(SamplingEvent::Completed { .. })),
        "expected Completed after strip-retry"
    );
}

/// Guards that `user_facing_api_error_message` keeps the `.image.source` path in a codeless `invalid_request_error`.
/// That way the codeless image-strip recovery fires on many-image dimension 400s instead of hard-failing every turn.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn many_image_dimension_400_strips_as_heuristic() {
    const IMAGE_URI: &str = "data:image/png;base64,cG9pc29uZWQ=";
    let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let counter_handler = Arc::clone(&counter);
    let app = Router::new().route(
        "/v1/chat/completions",
        post(move || {
            let counter = Arc::clone(&counter_handler);
            async move {
                if counter.fetch_add(1, Ordering::SeqCst) == 0 {
                    Err::<Sse<_>, (StatusCode, String)>((
                        StatusCode::BAD_REQUEST,
                        json!({
                            "error": {
                                "message": "messages.0.content.4.image.source.base64.data: At least one of the image dimensions exceed max allowed size for many-image requests: 2000 pixels",
                                "type": "invalid_request_error",
                            }
                        })
                        .to_string(),
                    ))
                } else {
                    let events = sse::chat_completion_events("recovered", "test-model");
                    Ok(Sse::new(stream::iter(
                        events.into_iter().map(Ok::<_, std::convert::Infallible>),
                    )))
                }
            }
        }),
    );
    let server = MockServer::spawn(app).await;
    let (event_tx, mut event_rx) = mpsc::unbounded_channel();
    let handle = SamplerActor::spawn(
        test_config(server.base_url(), "test-model"),
        RetryPolicy::default(),
        event_tx,
    );

    let mut request = user_request("what is in this image?");
    if let Some(ConversationItem::User(u)) = request.items.first_mut() {
        u.add_image(IMAGE_URI);
    }
    handle.submit(RequestId::from("req-many-image-dimension-strip"), request);

    let events = drain_until_terminal(&mut event_rx, Duration::from_secs(15)).await;
    server.shutdown();

    assert!(
        events.iter().any(|e| matches!(
            e,
            SamplingEvent::ImagesStripped {
                reason: StripReason::PayloadHeuristic,
                ..
            }
        )),
        "codeless many-image dimension 400 must strip as PayloadHeuristic, got {events:?}"
    );
    assert!(
        matches!(events.last(), Some(SamplingEvent::Completed { .. })),
        "expected Completed after strip-retry"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn image_400_with_nothing_left_to_strip_is_fatal_after_one_cycle() {
    // `stripped == 0` is the only bound on the strip-retry loop.
    let counter = Arc::new(AtomicU32::new(0));
    let counter_handler = Arc::clone(&counter);
    let app = Router::new().route(
        "/v1/chat/completions",
        post(move || {
            let counter = Arc::clone(&counter_handler);
            async move {
                counter.fetch_add(1, Ordering::SeqCst);
                Err::<Sse<futures_util::stream::Empty<Result<Event, std::convert::Infallible>>>, _>(
                    (
                        StatusCode::BAD_REQUEST,
                        json!({
                            "code": INVALID_IMAGE_ERROR_CODE,
                            "error": "Base64 string of provided image cannot be decoded.",
                        })
                        .to_string(),
                    ),
                )
            }
        }),
    );
    let server = MockServer::spawn(app).await;
    let (event_tx, mut event_rx) = mpsc::unbounded_channel();
    let handle = SamplerActor::spawn(
        test_config(server.base_url(), "test-model"),
        RetryPolicy::default(),
        event_tx,
    );

    let mut request = user_request("what is in this image?");
    if let Some(ConversationItem::User(u)) = request.items.first_mut() {
        u.add_image("data:image/png;base64,cG9pc29uZWQ=");
    }
    handle.submit(RequestId::from("req-strip-exhausted"), request);

    let events = drain_until_terminal(&mut event_rx, Duration::from_secs(15)).await;
    server.shutdown();

    let strips = events
        .iter()
        .filter(|e| matches!(e, SamplingEvent::ImagesStripped { .. }))
        .count();
    assert_eq!(strips, 1, "exactly one strip cycle");
    assert!(
        matches!(events.last(), Some(SamplingEvent::Failed { .. })),
        "second image 400 with nothing left to strip must be fatal, got {events:?}"
    );
    assert_eq!(
        counter.load(Ordering::SeqCst),
        2,
        "one rejection, one strip-retry, then stop"
    );
}

/// RST with a zero retry budget: the decision is Fatal, so the proactive heuristic strip must NOT run.
/// There is no mutation, no ImagesStripped event, and no "left out of the retry" note for a retry that never happens.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn fatal_decision_does_not_strip_or_emit_images_stripped() {
    const IMAGE_URI: &str = "data:image/png;base64,cG9pc29uZWQ=";
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (shutdown_tx, mut shutdown_rx) = oneshot::channel::<()>();
    tokio::spawn(async move {
        // RST every connection: peek then drop (see xai-grok-http).
        loop {
            tokio::select! {
                _ = &mut shutdown_rx => break,
                accepted = listener.accept() => {
                    let Ok((sock, _)) = accepted else { break };
                    let mut buf = [0u8; 64];
                    let _ = sock.peek(&mut buf).await;
                    drop(sock);
                }
            }
        }
    });
    tokio::time::sleep(Duration::from_millis(20)).await;

    let mut config = test_config(format!("http://{addr}/v1"), "test-model");
    config.max_retries = Some(0);
    let (event_tx, mut event_rx) = mpsc::unbounded_channel();
    let handle = SamplerActor::spawn(config, RetryPolicy::default(), event_tx);

    let mut request = user_request("what is in this image?");
    if let Some(ConversationItem::User(u)) = request.items.first_mut() {
        u.add_image(IMAGE_URI);
    }
    handle.submit(RequestId::from("req-fatal-no-strip"), request);

    let events = drain_until_terminal(&mut event_rx, Duration::from_secs(15)).await;
    let _ = shutdown_tx.send(());

    assert!(
        !events
            .iter()
            .any(|e| matches!(e, SamplingEvent::ImagesStripped { .. })),
        "a Fatal decision must not strip or emit ImagesStripped, got {events:?}"
    );
    assert!(
        matches!(events.last(), Some(SamplingEvent::Failed { .. })),
        "expected terminal Failed, got {events:?}"
    );
}

/// An RST mid-upload (nginx-style 413) emits PayloadHeuristic and strips the request only.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn connection_reset_emits_payload_heuristic_and_strips_request() {
    const IMAGE_URI: &str = "data:image/png;base64,cG9pc29uZWQ=";
    let bodies = Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
    let bodies_handler = Arc::clone(&bodies);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    tokio::spawn(async move {
        // Peek then drop so the peer sees RST (see xai-grok-http).
        if let Ok((sock, _)) = listener.accept().await {
            let mut buf = [0u8; 64];
            let _ = sock.peek(&mut buf).await;
            drop(sock);
        }
        let app = Router::new().route(
            "/v1/chat/completions",
            post(move |body: String| {
                let bodies = Arc::clone(&bodies_handler);
                async move {
                    bodies.lock().unwrap().push(body);
                    let events = sse::chat_completion_events("recovered", "test-model");
                    Sse::new(stream::iter(
                        events.into_iter().map(Ok::<_, std::convert::Infallible>),
                    ))
                }
            }),
        );
        let _ = axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                let _ = shutdown_rx.await;
            })
            .await;
    });
    tokio::time::sleep(Duration::from_millis(20)).await;

    let (event_tx, mut event_rx) = mpsc::unbounded_channel();
    let handle = SamplerActor::spawn(
        test_config(format!("http://{addr}/v1"), "test-model"),
        RetryPolicy::default(),
        event_tx,
    );

    let mut request = user_request("what is in this image?");
    if let Some(ConversationItem::User(u)) = request.items.first_mut() {
        u.add_image(IMAGE_URI);
    }
    handle.submit(RequestId::from("req-heuristic-strip"), request);

    let events = drain_until_terminal(&mut event_rx, Duration::from_secs(15)).await;
    let _ = shutdown_tx.send(());

    assert!(
        events.iter().any(|e| match e {
            SamplingEvent::ImagesStripped {
                stripped_urls,
                reason: StripReason::PayloadHeuristic,
                ..
            } => stripped_urls.len() == 1 && stripped_urls[0].as_ref() == IMAGE_URI,
            _ => false,
        }),
        "connection reset must emit PayloadHeuristic ImagesStripped, got {events:?}"
    );
    assert!(
        !events.iter().any(|e| matches!(
            e,
            SamplingEvent::ImagesStripped {
                reason: StripReason::ServerRejected,
                ..
            }
        )),
        "heuristic path must not be labeled ServerRejected, got {events:?}"
    );
    assert!(
        matches!(events.last(), Some(SamplingEvent::Completed { .. })),
        "strip-retry must complete, got {events:?}"
    );
    let bodies = bodies.lock().unwrap();
    assert_eq!(bodies.len(), 1, "only the post-strip retry hits HTTP");
    assert!(
        !bodies[0].contains(IMAGE_URI),
        "in-flight request must be stripped before the retry"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn connect_failure_does_not_emit_images_stripped() {
    // Connection refused is `is_connect`, not a body-upload reset.
    let (event_tx, mut event_rx) = mpsc::unbounded_channel();
    let handle = SamplerActor::spawn(
        test_config("http://127.0.0.1:1/v1".into(), "test-model"),
        RetryPolicy::default(),
        event_tx,
    );

    let mut request = user_request("what is in this image?");
    if let Some(ConversationItem::User(u)) = request.items.first_mut() {
        u.add_image("data:image/png;base64,cG9pc29uZWQ=");
    }
    handle.submit(RequestId::from("req-connect-fail"), request);

    let events = drain_until_terminal(&mut event_rx, Duration::from_secs(15)).await;
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, SamplingEvent::ImagesStripped { .. })),
        "connect failure must not strip images, got {events:?}"
    );
    assert!(
        matches!(events.last(), Some(SamplingEvent::Failed { .. })),
        "exhausted connect retries must be Failed, got {events:?}"
    );
}

// ---------------------------------------------------------------------------
// Rate-limit thresholds
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rate_limit_exhausts_at_default_threshold_and_yields_failed() {
    let counter = Arc::new(AtomicU32::new(0));
    let counter_handler = Arc::clone(&counter);
    let app = Router::new().route(
        "/v1/chat/completions",
        post(move || {
            let counter = Arc::clone(&counter_handler);
            async move {
                counter.fetch_add(1, Ordering::SeqCst);
                Err::<
                    Sse<
                        futures_util::stream::Iter<
                            std::vec::IntoIter<Result<Event, std::convert::Infallible>>,
                        >,
                    >,
                    (StatusCode, String),
                >((
                    StatusCode::TOO_MANY_REQUESTS,
                    json!({ "error": { "message": "slow down" } }).to_string(),
                ))
            }
        }),
    );
    let server = MockServer::spawn(app).await;
    let (event_tx, mut event_rx) = mpsc::unbounded_channel();
    let cfg = test_config(server.base_url(), "test-model");
    let handle = SamplerActor::spawn(cfg, RetryPolicy::default(), event_tx);

    let rid = RequestId::from("req-429-default");
    handle.submit(rid, user_request("hi"));

    let events = drain_until_terminal(&mut event_rx, Duration::from_secs(60)).await;
    server.shutdown();

    match events.last().unwrap() {
        SamplingEvent::Failed { error, .. } => {
            assert_eq!(error.kind, SamplingErrorKind::RateLimited);
            assert_eq!(error.status_code, Some(429));
        }
        other => panic!("expected Failed(RateLimited), got {other:?}"),
    }

    // The request task awaits and classifies each wire attempt before starting the next, so scheduling cannot add another request.
    assert_eq!(
        counter.load(Ordering::SeqCst),
        2,
        "the default threshold permits one retry after the initial request"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn configured_rate_limit_threshold_controls_total_wire_attempts() {
    let counter = Arc::new(AtomicU32::new(0));
    let counter_handler = Arc::clone(&counter);
    let app = Router::new().route(
        "/v1/chat/completions",
        post(move || {
            let counter = Arc::clone(&counter_handler);
            async move {
                counter.fetch_add(1, Ordering::SeqCst);
                (
                    StatusCode::TOO_MANY_REQUESTS,
                    [("retry-after", "0")],
                    json!({ "error": { "message": "slow down" } }).to_string(),
                )
            }
        }),
    );
    let server = MockServer::spawn(app).await;
    let (event_tx, mut event_rx) = mpsc::unbounded_channel();
    let mut cfg = test_config(server.base_url(), "test-model");
    cfg.max_retries = Some(6);
    cfg.rate_limit_retry_threshold = Some(4);
    let handle = SamplerActor::spawn(cfg, RetryPolicy::default(), event_tx);

    let rid = RequestId::from("req-429");
    handle.submit(rid.clone(), user_request("hi"));

    let events = drain_until_terminal(&mut event_rx, Duration::from_secs(60)).await;
    server.shutdown();

    match events.last().unwrap() {
        SamplingEvent::Failed { error, .. } => {
            assert_eq!(error.kind, SamplingErrorKind::RateLimited);
            assert_eq!(error.status_code, Some(429));
        }
        other => panic!("expected Failed(RateLimited), got {other:?}"),
    }

    let hits = counter.load(Ordering::SeqCst);
    assert_eq!(
        hits, 4,
        "the configured threshold is a total-attempt ceiling and must override the policy default of 2"
    );
}

// ---------------------------------------------------------------------------
// Auth error -> EmitToSession (immediate)
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn auth_401_emits_failed_immediately_no_retry() {
    let counter = Arc::new(AtomicU32::new(0));
    let counter_handler = Arc::clone(&counter);
    let app = Router::new().route(
        "/v1/chat/completions",
        post(move || {
            let counter = Arc::clone(&counter_handler);
            async move {
                counter.fetch_add(1, Ordering::SeqCst);
                Err::<
                    Sse<
                        futures_util::stream::Iter<
                            std::vec::IntoIter<Result<Event, std::convert::Infallible>>,
                        >,
                    >,
                    (StatusCode, String),
                >((StatusCode::UNAUTHORIZED, "unauthorized".to_string()))
            }
        }),
    );
    let server = MockServer::spawn(app).await;
    let (event_tx, mut event_rx) = mpsc::unbounded_channel();
    let cfg = test_config(server.base_url(), "test-model");
    let handle = SamplerActor::spawn(cfg, RetryPolicy::default(), event_tx);

    let rid = RequestId::from("req-auth");
    handle.submit(rid.clone(), user_request("hi"));

    let events = drain_until_terminal(&mut event_rx, Duration::from_secs(5)).await;
    server.shutdown();

    // The session owns auth errors: `classify_error` returns `EmitToSession`, so the actor emits Failed immediately without retrying
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, SamplingEvent::Retrying { .. }))
    );
    match events.last().unwrap() {
        SamplingEvent::Failed { error, .. } => {
            assert_eq!(error.kind, SamplingErrorKind::Auth);
        }
        other => panic!("expected Failed(Auth), got {other:?}"),
    }
    assert_eq!(counter.load(Ordering::SeqCst), 1, "no retries on 401");
}

// ---------------------------------------------------------------------------
// UpdateConfig invalidates cache + applies to subsequent requests
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn update_config_changes_subsequent_request_model() {
    use std::sync::Mutex;

    let captured_models: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let captured_handler = Arc::clone(&captured_models);
    let app = Router::new().route(
        "/v1/chat/completions",
        post(move |axum::Json(body): axum::Json<serde_json::Value>| {
            let captured = Arc::clone(&captured_handler);
            async move {
                let model = body
                    .get("model")
                    .and_then(|m| m.as_str())
                    .unwrap_or("")
                    .to_string();
                captured.lock().unwrap().push(model);
                let events = sse::chat_completion_events("ok", "test-model");
                Sse::new(stream::iter(
                    events.into_iter().map(Ok::<_, std::convert::Infallible>),
                ))
            }
        }),
    );
    let server = MockServer::spawn(app).await;
    let (event_tx, _event_rx) = mpsc::unbounded_channel();
    let cfg = test_config(server.base_url(), "model-A");
    let handle = SamplerActor::spawn(cfg, RetryPolicy::default(), event_tx);

    let _ = handle
        .submit_and_collect(RequestId::from("req-1"), user_request("hi"))
        .await
        .expect("first req ok");

    let mut new_cfg = test_config(server.base_url(), "model-B");
    new_cfg.api_key = Some("test-key".into());
    handle.update_config(new_cfg);

    let _ = handle
        .submit_and_collect(RequestId::from("req-2"), user_request("hi"))
        .await
        .expect("second req ok");

    server.shutdown();

    let models = captured_models.lock().unwrap();
    assert_eq!(
        models.as_slice(),
        &["model-A".to_string(), "model-B".to_string()]
    );
}

// ---------------------------------------------------------------------------
// Helpers for draining the event channel
// ---------------------------------------------------------------------------

/// Drain the event channel until a terminal event (`Completed` or `Failed`) is received, or until `deadline` elapses.
async fn drain_until_terminal(
    rx: &mut mpsc::UnboundedReceiver<SamplingEvent>,
    timeout: Duration,
) -> Vec<SamplingEvent> {
    let mut out = Vec::new();
    let start = tokio::time::Instant::now();
    loop {
        let elapsed = start.elapsed();
        if elapsed >= timeout {
            panic!(
                "drain_until_terminal timed out after {:?}; got {} events",
                timeout,
                out.len()
            );
        }
        let remaining = timeout - elapsed;
        match tokio::time::timeout(remaining, rx.recv()).await {
            Ok(Some(ev)) => {
                let terminal = matches!(
                    ev,
                    SamplingEvent::Completed { .. } | SamplingEvent::Failed { .. }
                );
                out.push(ev);
                if terminal {
                    return out;
                }
            }
            Ok(None) => panic!("event channel closed before terminal event"),
            Err(_) => panic!(
                "drain_until_terminal timed out after {:?}; got {} events",
                timeout,
                out.len()
            ),
        }
    }
}

/// Wait for the next event matching `pred`, or return `None` on timeout.
async fn await_event_matching(
    rx: &mut mpsc::UnboundedReceiver<SamplingEvent>,
    mut pred: impl FnMut(&SamplingEvent) -> bool,
    timeout: Duration,
) -> Option<SamplingEvent> {
    let start = tokio::time::Instant::now();
    loop {
        let elapsed = start.elapsed();
        if elapsed >= timeout {
            return None;
        }
        let remaining = timeout - elapsed;
        match tokio::time::timeout(remaining, rx.recv()).await {
            Ok(Some(ev)) => {
                if pred(&ev) {
                    return Some(ev);
                }
            }
            Ok(None) => return None,
            Err(_) => return None,
        }
    }
}

// ---------------------------------------------------------------------------
// 429 retry with Retry-After backoff, and 5xx retry exhaustion
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn retries_on_429_then_succeeds() {
    let counter = Arc::new(AtomicU32::new(0));
    let counter_handler = Arc::clone(&counter);
    let app = Router::new().route(
        "/v1/chat/completions",
        post(move || {
            let counter = Arc::clone(&counter_handler);
            async move {
                let n = counter.fetch_add(1, Ordering::SeqCst);
                if n == 0 {
                    // First attempt: rate limited, but `Retry-After: 0` keeps the wait at zero.
                    let mut resp = axum::response::Response::new(axum::body::Body::from(
                        json!({ "error": { "message": "rate limited" } }).to_string(),
                    ));
                    *resp.status_mut() = StatusCode::TOO_MANY_REQUESTS;
                    resp.headers_mut()
                        .insert("retry-after", "0".parse().unwrap());
                    resp
                } else {
                    let events = sse::chat_completion_events("ok", "test-model");
                    Sse::new(stream::iter(
                        events.into_iter().map(Ok::<_, std::convert::Infallible>),
                    ))
                    .into_response()
                }
            }
        }),
    );
    let server = MockServer::spawn(app).await;
    let (event_tx, mut event_rx) = mpsc::unbounded_channel();
    let mut cfg = test_config(server.base_url(), "test-model");
    cfg.rate_limit_retry_threshold = Some(4);
    let handle = SamplerActor::spawn(cfg, RetryPolicy::default(), event_tx);

    let rid = RequestId::from("req-429");
    handle.submit(rid.clone(), user_request("hi"));

    let events = drain_until_terminal(&mut event_rx, Duration::from_secs(15)).await;
    server.shutdown();

    assert!(
        events
            .iter()
            .any(|e| matches!(e, SamplingEvent::Retrying { .. })),
        "expected a Retrying event after the 429"
    );
    match events.last().unwrap() {
        SamplingEvent::Completed { response, .. } => {
            assert_eq!(response.assistant().map(|a| a.content.as_ref()), Some("ok"));
        }
        other => panic!("expected Completed after 429 retry, got {other:?}"),
    }
    assert!(
        counter.load(Ordering::SeqCst) >= 2,
        "server must be hit again after the 429"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn exhausted_5xx_retries_fail_with_clear_error() {
    let app = Router::new().route(
        "/v1/chat/completions",
        post(|| async {
            Err::<axum::response::Response, (StatusCode, String)>((
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({ "error": { "message": "server exploded" } }).to_string(),
            ))
        }),
    );
    let server = MockServer::spawn(app).await;
    let (event_tx, mut event_rx) = mpsc::unbounded_channel();
    let mut cfg = test_config(server.base_url(), "test-model");
    cfg.max_retries = Some(1);
    let handle = SamplerActor::spawn(cfg, RetryPolicy::default(), event_tx);

    let rid = RequestId::from("req-500-exhaust");
    handle.submit(rid.clone(), user_request("hi"));

    let events = drain_until_terminal(&mut event_rx, Duration::from_secs(15)).await;
    server.shutdown();

    match events.last().unwrap() {
        SamplingEvent::Failed { error, .. } => {
            assert_eq!(error.kind, SamplingErrorKind::Api);
            assert!(
                error.message.contains("server exploded") || error.message.contains("500"),
                "failure must carry a clear provider message, got {:?}",
                error.message
            );
        }
        other => panic!("expected Failed after exhausting retries, got {other:?}"),
    }
}
