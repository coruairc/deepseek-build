use super::*;

pub(super) fn btw_prepare_items(mut items: Vec<ConversationItem>) -> Vec<ConversationItem> {
    // Strip reasoning (same as strip_reasoning_blocks)
    items.retain(|item| !matches!(item, ConversationItem::Reasoning(_)));
    // Truncate trailing incomplete tool runs.
    while let Some(last) = items.last() {
        match last {
            ConversationItem::Assistant(a) if !a.tool_calls.is_empty() => {
                items.pop();
            }
            ConversationItem::ToolResult(_) => {
                items.pop();
            }
            _ => break,
        }
    }
    items.push(ConversationItem::user("btw what is X?"));
    items
}

pub(super) fn btw_mid_turn_conversation() -> Vec<ConversationItem> {
    vec![
        ConversationItem::system("You are helpful."),
        ConversationItem::user("Fix the bug"),
        // Completed turn with thinking
        ConversationItem::Assistant(AssistantItem {
            content: "I'll look at the code.".into(),
            tool_calls: vec![],
            model_id: Some("messages-compatible-model".into()),
            model_fingerprint: None,
            reasoning_effort: None,
        }),
        // Completed tool pair
        ConversationItem::Assistant(AssistantItem {
            content: String::new().into(),
            tool_calls: vec![ToolCall {
                id: "call_1".into(),
                name: "read_file".to_string(),
                arguments: r#"{"path":"src/main.rs"}"#.into(),
            }],
            model_id: Some("messages-compatible-model".into()),
            model_fingerprint: None,
            reasoning_effort: None,
        }),
        ConversationItem::tool_result("call_1", "fn main() {}"),
        ConversationItem::Assistant(AssistantItem {
            content: "I see the issue.".into(),
            tool_calls: vec![],
            model_id: Some("messages-compatible-model".into()),
            model_fingerprint: None,
            reasoning_effort: None,
        }),
        // Mid-turn: orphaned tool_use (no result yet)
        ConversationItem::Assistant(AssistantItem {
            content: String::new().into(),
            tool_calls: vec![ToolCall {
                id: "call_2".into(),
                name: "search_replace".to_string(),
                arguments: "{}".into(),
            }],
            model_id: Some("messages-compatible-model".into()),
            model_fingerprint: None,
            reasoning_effort: None,
        }),
    ]
}

pub(super) fn assistant_with_calls(calls: &[(&str, &str)]) -> ConversationItem {
    ConversationItem::Assistant(AssistantItem {
        content: String::new().into(),
        tool_calls: calls
            .iter()
            .map(|(id, name)| ToolCall {
                id: (*id).into(),
                name: (*name).into(),
                arguments: "{}".into(),
            })
            .collect(),
        model_id: None,
        model_fingerprint: None,
        reasoning_effort: None,
    })
}

pub(super) fn make_response(message: ConversationItem) -> ConversationResponse {
    ConversationResponse {
        items: vec![message],
        stop_reason: Some(StopReason::Stop),
        usage: None,
        cost_usd_ticks: None,
        message_chunks_emitted: 0,
        doom_loop_signals: Vec::new(),
        stop_message: None,
        message_id: None,
        raw_stop_reason: None,
        stop_sequence: None,
    }
}

pub(super) fn reasoning_sibling(
    id: &str,
    summary_text: &str,
    encrypted: Option<&str>,
) -> ConversationItem {
    ConversationItem::Reasoning(rs::ReasoningItem {
        id: id.to_string(),
        summary: vec![rs::SummaryPart::SummaryText(rs::SummaryTextContent {
            text: summary_text.to_string(),
        })],
        content: None,
        encrypted_content: encrypted.map(str::to_owned),
        status: None,
    })
}
