//! In-process tool dispatch.
//!
//! A [`LocalRegistry`] holds type-erased tool implementations; a local-only
//! [`ToolHarness`] resolves and runs them with no network transport. This is
//! the post-hub replacement for the computer-hub SDK's local dispatch surface:
//! tools registered here execute directly in the calling process.

use std::sync::Arc;

use dashmap::DashMap;
use serde_json::Value;
use xai_tool_protocol::{SessionId, ToolId};

use crate::{
    Tool, ToolCallContext, ToolDyn, ToolError, ToolStream, TypedExtensions, TypedToolOutput,
    terminal_only,
};

/// In-process registry of tool implementations.
///
/// Tools registered here dispatch entirely in-process: [`ToolHarness::call`]
/// resolves a handle and invokes it directly. Mutations are concurrency-safe
/// (`DashMap`), so callers MAY hot-add or hot-remove tools while a harness is
/// in use.
#[derive(Clone, Default)]
pub struct LocalRegistry {
    inner: Arc<DashMap<ToolId, Arc<dyn ToolDyn>>>,
}

impl std::fmt::Debug for LocalRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LocalRegistry").field("len", &self.len()).finish()
    }
}

impl LocalRegistry {
    /// Construct an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a typed [`Tool`] implementation by value. Subsequent
    /// registrations of the same id replace the previous handle and return the
    /// displaced handle for inspection / drop ordering.
    pub fn register<T>(&self, tool: T) -> Option<Arc<dyn ToolDyn>>
    where
        T: Tool + 'static,
    {
        self.inner.insert(tool.id(), Arc::new(tool))
    }

    /// Register a type-erased [`ToolDyn`] directly.
    ///
    /// Use this for inherently dynamic tools (e.g. MCP tools retrieved from a
    /// registry as `Arc<dyn ToolDyn>`) where the concrete type is not
    /// available. For native tools with a concrete type, prefer
    /// [`register`](Self::register).
    pub fn register_dyn(&self, tool: Arc<dyn ToolDyn>) -> Option<Arc<dyn ToolDyn>> {
        let id = tool.id();
        self.inner.insert(id, tool)
    }

    /// Resolve `tool_id` to its in-process handle, if registered.
    pub fn find(&self, tool_id: &ToolId) -> Option<Arc<dyn ToolDyn>> {
        self.inner.get(tool_id).map(|entry| entry.clone())
    }

    /// Drop the handle bound to `tool_id`. Returns `true` iff a matching entry
    /// was removed.
    pub fn unregister(&self, tool_id: &ToolId) -> bool {
        self.inner.remove(tool_id).is_some()
    }

    /// Number of tools currently registered.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// `true` iff no tools are registered.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// `true` iff `tool_id` is currently registered.
    pub fn contains(&self, tool_id: &ToolId) -> bool {
        self.inner.contains_key(tool_id)
    }
}

/// Local-only tool harness: resolves tools from a [`LocalRegistry`] and
/// dispatches them in-process. There is no server connection.
#[derive(Clone)]
pub struct ToolHarness {
    registry: LocalRegistry,
    session: SessionId,
    default_extensions: TypedExtensions,
}

impl std::fmt::Debug for ToolHarness {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToolHarness")
            .field("session", &self.session)
            .field("local_tool_count", &self.registry.len())
            .finish_non_exhaustive()
    }
}

impl ToolHarness {
    /// Construct a local-only harness (no server connection). Tools are
    /// resolved exclusively from `registry`; `default_extensions` are merged
    /// into every [`ToolCallContext`] before dispatch.
    pub fn local_only_with(
        registry: LocalRegistry,
        session: SessionId,
        default_extensions: TypedExtensions,
    ) -> Self {
        Self {
            registry,
            session,
            default_extensions,
        }
    }

    /// Bound session.
    pub fn session(&self) -> &SessionId {
        &self.session
    }

    /// Clone of the underlying [`LocalRegistry`].
    pub fn local_registry(&self) -> LocalRegistry {
        self.registry.clone()
    }

    /// Dispatch a tool call in-process. A miss resolves to a single-item
    /// terminal stream carrying [`ToolError::not_found`].
    pub async fn call(
        &self,
        tool_id: ToolId,
        args: Value,
        mut ctx: ToolCallContext,
    ) -> ToolStream<TypedToolOutput> {
        ctx.extensions.merge_defaults(&self.default_extensions);
        match self.registry.find(&tool_id) {
            Some(handle) => handle.execute(ctx, args).await,
            None => terminal_only(Err(ToolError::not_found(
                tool_id.clone(),
                format!("tool not found (local harness): {tool_id}"),
            ))),
        }
    }
}
