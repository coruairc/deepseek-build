//! Classify one stream item for `StreamSpanTiming::hold_until_first_content`.

use xai_grok_sampling_types::ChatCompletionChunk;

use crate::span_timing::ItemClass;

fn chat_chunk_has_content(chunk: &ChatCompletionChunk) -> bool {
    use xai_grok_sampling_types::ChatChunkDelta;
    chunk.choices.iter().any(|choice| {
        let ChatChunkDelta {
            role: _,
            content,
            reasoning_content,
            tool_calls,
            tool_call_id: _,
        } = &choice.delta;
        content.as_deref().is_some_and(|text| !text.is_empty())
            || reasoning_content
                .as_deref()
                .is_some_and(|text| !text.is_empty())
            || tool_calls.iter().any(|call| {
                call.function.as_ref().is_some_and(|function| {
                    function
                        .name
                        .as_deref()
                        .is_some_and(|name| !name.is_empty())
                        || function
                            .arguments
                            .as_deref()
                            .is_some_and(|args| !args.is_empty())
                })
            })
    })
}

pub(crate) fn chat_chunk_class(chunk: &ChatCompletionChunk) -> ItemClass {
    if chat_chunk_has_content(chunk) {
        ItemClass::Content
    } else {
        ItemClass::Other
    }
}
