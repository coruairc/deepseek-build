use super::*;
use crate::sampling::{Client, ConversationItem, SamplerConfig};
use axum::Router;
use axum::body::Bytes;
use axum::response::IntoResponse;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::routing::post;
use futures_util::stream;
use serde_json::json;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use xai_grok_sampling_types::ReasoningEffort;

const SUMMARY: &str = "<summary>ok</summary>";

fn chat_completions_stream() -> Vec<Event> {
    vec![
        Event::default().data(
            json!({
                "id": "chatcmpl-test", "object": "chat.completion.chunk",
                "created": 1234567890, "model": "test-model",
                "choices": [{
                    "index": 0,
                    "delta": { "role": "assistant", "content": SUMMARY },
                    "finish_reason": "stop"
                }]
            })
            .to_string(),
        ),
        Event::default().data("[DONE]"),
    ]
}

/// Runs one ChatCompletions compaction against a mock server and returns the parsed request body.
async fn compaction_request_body(
    backend: ApiBackend,
    reasoning_effort: Option<ReasoningEffort>,
) -> serde_json::Value {
    let (path, events): (&str, fn() -> Vec<Event>) = match backend {
        ApiBackend::ChatCompletions => ("/v1/chat/completions", chat_completions_stream),
    };
    let captured = Arc::new(Mutex::new(None::<serde_json::Value>));
    let cap = captured.clone();
    let app = Router::new().route(
        path,
        post(move |body: Bytes| {
            let cap = cap.clone();
            async move {
                *cap.lock().unwrap() = Some(serde_json::from_slice(&body).unwrap());
                let stream =
                    stream::iter(events().into_iter().map(Ok::<_, std::convert::Infallible>));
                Sse::new(stream)
                    .keep_alive(KeepAlive::default())
                    .into_response()
            }
        }),
    );
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async {
                let _ = shutdown_rx.await;
            })
            .await
            .unwrap();
    });

    let config = SamplerConfig {
        api_key: Some("test-api-key".into()),
        base_url: format!("http://{addr}/v1"),
        model: "test-model".into(),
        max_completion_tokens: Some(1000),
        temperature: Some(0.7),
        api_backend: backend.clone(),
        context_window: 256_000,
        reasoning_effort,
        ..Default::default()
    };
    let client = Client::new(config.clone()).unwrap();
    let chat_history = vec![
        ConversationItem::system("You are a helpful assistant."),
        ConversationItem::user("<user_query>\nfix the bug\n</user_query>"),
        ConversationItem::assistant("I fixed it."),
        ConversationItem::user("Summarize the conversation so far."),
    ];
    let output = generate_session_compact(
        chat_history,
        0,
        vec![],
        vec![],
        client,
        acp::SessionId::new("reasoning-effort-test"),
        &config,
        std::time::Duration::from_secs(30),
        0,
        crate::util::config::CompactionToolChoice::Auto,
        &tokio_util::sync::CancellationToken::new(),
    )
    .await
    .unwrap_or_else(|_| panic!("{backend:?} compaction must succeed"));
    assert_eq!(output.content, SUMMARY);

    let body = captured
        .lock()
        .unwrap()
        .take()
        .expect("compaction request must reach the mock server");
    let _ = shutdown_tx.send(());
    body
}

fn absent(body: &serde_json::Value, pointer: &str) -> bool {
    body.pointer(pointer).is_none_or(serde_json::Value::is_null)
}

#[tokio::test]
async fn chat_completions_compaction_sends_session_reasoning_effort() {
    let body =
        compaction_request_body(ApiBackend::ChatCompletions, Some(ReasoningEffort::Xhigh)).await;
    assert_eq!(
        body.pointer("/reasoning_effort"),
        Some(&json!("xhigh")),
        "{body:#}"
    );
}

#[tokio::test]
async fn chat_completions_compaction_omits_unset_reasoning_effort() {
    let body = compaction_request_body(ApiBackend::ChatCompletions, None).await;
    assert!(absent(&body, "/reasoning_effort"), "{body:#}");
}
