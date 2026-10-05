//! Layer-2 stream transforms: turn raw HTTP chunk streams into [`SamplingEvent`](crate::events::SamplingEvent) streams.
//!
//! DeepSeek Build speaks only the Chat Completions API, so this module has a single
//! transform. Dispatch lives in [`actor::request_task`](crate::actor::request_task).

pub mod chat_completions;
pub mod collect;

pub use chat_completions::stream_chat_completions;
pub use collect::collect_response;
