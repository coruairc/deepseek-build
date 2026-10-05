//! Remote-advertised request-body compression for the chat routes.

use std::sync::RwLock;

use xai_grok_config_types::RemoteRequestEncoding;
use xai_grok_sampler::RequestCompression;

/// Base URL of the model-proxy whose `/v1/settings` listed `zstd` in
/// `accept_request_encodings`. `None` until one does. One slot on purpose: a
/// process fetches settings from a single proxy, and if a second one ever
/// advertised, the first would fall back to plain JSON (exact-origin match),
/// never to a wrong compression.
static ZSTD_ORIGIN: RwLock<Option<String>> = RwLock::new(None);

/// Called whenever the agent applies `RemoteSettings` fetched from `origin`.
pub(crate) fn cache_remote_accept_request_encodings(
    origin: &str,
    encodings: &[RemoteRequestEncoding],
) {
    if let Ok(mut guard) = ZSTD_ORIGIN.write() {
        *guard = encodings
            .contains(&RemoteRequestEncoding::Zstd)
            .then(|| origin.to_owned());
    }
}

/// Compression the sampler may apply toward `base_url`. `GROK_REQUEST_COMPRESSION=0`
/// is the operator kill switch, re-read whenever a sampler config is built.
pub(crate) fn request_compression_for_url(base_url: &str) -> RequestCompression {
    if xai_grok_config::env_bool("GROK_REQUEST_COMPRESSION") == Some(false) {
        return RequestCompression::None;
    }
    let origin = ZSTD_ORIGIN.read().ok().and_then(|guard| guard.clone());
    request_compression_for(base_url, origin.as_deref())
}

/// Zstd only toward the proxy that advertised it. A sibling trusted route
/// (staging, the dev proxy) or any other host (BYOK, local models) never
/// receives a body its server may not decode.
fn request_compression_for(base_url: &str, zstd_origin: Option<&str>) -> RequestCompression {
    if zstd_origin.is_some_and(|origin| crate::util::matches_trusted_base_url(base_url, origin)) {
        RequestCompression::Zstd
    } else {
        RequestCompression::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
