//! Wire-level tests for the Chat Completions DeepSeek adapter.
//!
//! These pin the correctness-critical behaviours the adapter owns:
//! - streaming text + reasoning channels;
//! - a multi-turn tool-call chain that replays `reasoning_content` on every prior
//!   assistant tool-call message;
//! - missing-`reasoning_content` backfill (the placeholder is actually sent);
//! - malformed tool-call JSON repaired via typographic-quote normalization;
//! - a partial/interrupted stream surfacing a clear failure.
//!
//! The mock is `wiremock`; the request bodies it received are inspected directly so
//! assertions cover what went on the wire, not just the in-process representation.

use std::time::Duration;

use futures_util::StreamExt;
use serde_json::{Value, json};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use xai_grok_sampler::{
    RequestId, SamplerConfig, SamplingChannel, SamplingClient, SamplingEvent, collect_response,
    stream_chat_completions,
};
use xai_grok_sampling_types::{
    ChatCompletionRequest, ConversationItem, ConversationRequest, ConversationResponse,
    REASONING_PLACEHOLDER, ToolSpec,
};

const MODEL: &str = "deepseek-v4-pro";

async fn mock_server(body: String, content_type: &str) -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", content_type)
                .set_body_raw(body, content_type),
        )
        .mount(&server)
        .await;
    server
}

fn client(server: &MockServer) -> SamplingClient {
    SamplingClient::new(SamplerConfig {
        api_key: Some("test-key".into()),
        base_url: format!("{}/v1", server.uri()),
        model: MODEL.into(),
        context_window: 128_000,
        idle_timeout_secs: Some(30),
        ..Default::default()
    })
    .expect("client constructs")
}

fn sse(events: &[Value]) -> String {
    let mut body = String::new();
    for event in events {
        body.push_str("data: ");
        body.push_str(&event.to_string());
        body.push_str("\n\n");
    }
    body.push_str("data: [DONE]\n\n");
    body
}

fn chunk(delta: Value, finish: Value) -> Value {
    json!({
        "id": "chatcmpl-test",
        "object": "chat.completion.chunk",
        "created": 0,
        "model": MODEL,
        "choices": [{ "index": 0, "delta": delta, "finish_reason": finish }]
    })
}

fn reasoning_and_tool_call_body() -> String {
    sse(&[
        chunk(
            json!({"reasoning_content": "let me look at the file"}),
            json!(null),
        ),
        chunk(
            json!({"tool_calls": [{
                "index": 0,
                "id": "call_1",
                "type": "function",
                "function": { "name": "read_file", "arguments": "{\"path\":\"a\"}" }
            }]}),
            json!(null),
        ),
        chunk(json!({}), json!("tool_calls")),
    ])
}

async fn collect(client: &SamplingClient, request: ConversationRequest) -> ConversationResponse {
    let (raw, metadata) = client
        .conversation_stream(request)
        .await
        .expect("stream opens");
    let events =
        stream_chat_completions(raw, metadata, RequestId::random(), Duration::from_secs(10));
    collect_response(events).await.expect("stream completes").0
}

fn user_items(text: &str) -> Vec<ConversationItem> {
    vec![
        ConversationItem::system("You are a helpful assistant."),
        ConversationItem::user(text),
    ]
}

#[tokio::test]
async fn streaming_emits_text_and_reasoning_channels() {
    let server = mock_server(
        sse(&[
            chunk(json!({"reasoning_content": "thinking..."}), json!(null)),
            chunk(json!({"content": "hello"}), json!(null)),
            chunk(json!({}), json!("stop")),
        ]),
        "text/event-stream",
    )
    .await;

    let (raw, metadata) = client(&server)
        .conversation_stream(ConversationRequest::from_items(user_items("hi")))
        .await
        .expect("stream opens");
    let events: Vec<SamplingEvent> =
        stream_chat_completions(raw, metadata, RequestId::random(), Duration::from_secs(10))
            .collect()
            .await;

    let has_channel = |want: SamplingChannel| {
        events
            .iter()
            .any(|e| matches!(e, SamplingEvent::ChannelToken { channel, .. } if *channel == want))
    };
    assert!(has_channel(SamplingChannel::Reasoning), "reasoning channel");
    assert!(has_channel(SamplingChannel::Text), "text channel");
    assert!(
        events
            .iter()
            .any(|e| matches!(e, SamplingEvent::Completed { .. })),
        "must complete"
    );
}

#[tokio::test]
async fn three_turn_tool_chain_replays_reasoning_content_on_every_assistant_tool_call() {
    let server = mock_server(reasoning_and_tool_call_body(), "text/event-stream").await;
    let client = client(&server);

    let mut items = user_items("start");
    for _ in 0..3 {
        let response = collect(&client, ConversationRequest::from_items(items.clone())).await;
        items.extend(response.items.clone());
        items.push(ConversationItem::tool_result("call_1", "file contents"));
    }

    let requests = server.received_requests().await.expect("requests recorded");
    assert_eq!(requests.len(), 3, "one request per turn");

    // Every request after the first replays at least one prior assistant tool-call
    // message, and each of those must carry non-empty `reasoning_content`.
    for (turn, request) in requests.iter().enumerate().skip(1) {
        let body: Value =
            serde_json::from_slice(&request.body).expect("request body is valid JSON");
        let messages = body
            .get("messages")
            .and_then(|m| m.as_array())
            .expect("messages array");
        let mut checked = 0;
        for message in messages {
            let is_assistant = message.get("role").and_then(|r| r.as_str()) == Some("assistant");
            let has_tool_calls = message
                .get("tool_calls")
                .and_then(|t| t.as_array())
                .is_some_and(|t| !t.is_empty());
            if is_assistant && has_tool_calls {
                checked += 1;
                let reasoning = message
                    .get("reasoning_content")
                    .and_then(|r| r.as_str())
                    .unwrap_or_default();
                assert!(
                    !reasoning.is_empty(),
                    "turn {turn}: assistant tool-call message must replay reasoning_content; body={body}"
                );
                assert_eq!(reasoning, "let me look at the file");
            }
        }
        assert!(
            checked >= 1,
            "turn {turn}: expected a replayed assistant tool-call message"
        );
    }
}

#[tokio::test]
async fn missing_reasoning_content_is_backfilled_with_placeholder_on_the_wire() {
    // No `Reasoning` sibling precedes the assistant, and the request carries tools,
    // so the sanitizer must insert the placeholder into the serialized body.
    let request = ConversationRequest::from_items(vec![
        ConversationItem::system("sys"),
        ConversationItem::user("read it"),
        ConversationItem::assistant_tool_calls(vec![xai_grok_sampling_types::ToolCall {
            id: "call_9".into(),
            name: "read_file".into(),
            arguments: "{\"path\":\"a\"}".into(),
        }]),
        ConversationItem::tool_result("call_9", "contents"),
    ])
    .with_tools(vec![ToolSpec {
        name: "read_file".into(),
        description: Some("read".into()),
        parameters: json!({"type": "object"}),
    }]);

    // Serialize exactly as the wire path does, then assert the placeholder is present.
    let chat: ChatCompletionRequest = request.into();
    let body = serde_json::to_value(&chat).expect("serializes");
    let messages = body["messages"].as_array().expect("messages");
    let assistant = messages
        .iter()
        .find(|m| {
            m.get("role").and_then(|r| r.as_str()) == Some("assistant")
                && m.get("tool_calls").is_some()
        })
        .expect("assistant tool-call message");
    assert_eq!(
        assistant.get("reasoning_content").and_then(|v| v.as_str()),
        Some(REASONING_PLACEHOLDER),
        "the placeholder must be sent, not an omitted field"
    );

    // And it is genuinely sent over the wire.
    let server = mock_server(reasoning_and_tool_call_body(), "text/event-stream").await;
    let client = client(&server);
    let _ = collect(
        &client,
        ConversationRequest::from_items(vec![
            ConversationItem::system("sys"),
            ConversationItem::user("read it"),
            ConversationItem::assistant_tool_calls(vec![xai_grok_sampling_types::ToolCall {
                id: "call_9".into(),
                name: "read_file".into(),
                arguments: "{\"path\":\"a\"}".into(),
            }]),
            ConversationItem::tool_result("call_9", "contents"),
        ])
        .with_tools(vec![ToolSpec {
            name: "read_file".into(),
            description: Some("read".into()),
            parameters: json!({"type": "object"}),
        }]),
    )
    .await;
    let requests = server.received_requests().await.expect("requests recorded");
    let body: Value = serde_json::from_slice(&requests[0].body).expect("json");
    assert!(
        body.to_string().contains(REASONING_PLACEHOLDER),
        "placeholder must reach the wire; body={body}"
    );
}

#[tokio::test]
async fn malformed_tool_call_json_with_typographic_quotes_is_repaired() {
    // The model used curly quotes as the JSON *structural* delimiters, which is not
    // valid JSON until normalized to straight quotes.
    let curly_args = "{\u{201C}path\u{201D}:\u{201C}a\u{201D}}";
    let body = sse(&[
        chunk(
            json!({"tool_calls": [{
                "index": 0,
                "id": "call_1",
                "type": "function",
                "function": { "name": "read_file", "arguments": curly_args }
            }]}),
            json!(null),
        ),
        chunk(json!({}), json!("tool_calls")),
    ]);
    let server = mock_server(body, "text/event-stream").await;

    let response = collect(
        &client(&server),
        ConversationRequest::from_items(user_items("read")),
    )
    .await;
    let assistant = response.assistant().expect("assistant item");
    let call = assistant.tool_calls.first().expect("tool call");
    let parsed: Value =
        serde_json::from_str(&call.arguments).expect("repaired arguments parse as JSON");
    assert_eq!(parsed.get("path").and_then(|p| p.as_str()), Some("a"));
}

#[tokio::test]
async fn partial_stream_with_malformed_frame_fails_clearly() {
    // A content chunk followed by a truncated JSON frame; the stream was interrupted
    // mid-flight and the collector must surface an error rather than a blank success.
    // The malformed frame is terminated with a blank line so the SSE parser dispatches
    // it; the client must surface the decode failure instead of a partial success.
    let body = format!(
        "data: {}\n\ndata: {{\"id\":\"chatcmpl-test\",\"object\":\n\n",
        chunk(json!({"content": "partial"}), json!(null))
    );
    let server = mock_server(body, "text/event-stream").await;

    let (raw, metadata) = client(&server)
        .conversation_stream(ConversationRequest::from_items(user_items("hi")))
        .await
        .expect("stream opens");
    let events =
        stream_chat_completions(raw, metadata, RequestId::random(), Duration::from_secs(10));
    let result = collect_response(events).await;
    let error = result.expect_err("interrupted stream must not report success");
    assert!(
        !error.message.is_empty(),
        "failure must carry a clear message"
    );
}
