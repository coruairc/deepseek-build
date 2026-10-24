//! The `[endpoints]` config table, its environment variable overrides, and the URLs resolved from it.
//!
//! The auxiliary services (feedback, trace upload, managed config, telemetry) resolve to the model-proxy.
//! Only API-key inference uses `xai_api_base_url`.
use serde::{Deserialize, Serialize};
use xai_grok_env::{PROD_CLI_CHAT_PROXY_BASE_URL, env_string};
pub const CLI_CHAT_PROXY_BASE_URL_DEFAULT: &str = PROD_CLI_CHAT_PROXY_BASE_URL;
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct EndpointsConfig {
    /// When this is `None`, `proxy_url` returns `CLI_CHAT_PROXY_BASE_URL_DEFAULT`.
    /// `Some` means someone configured it, even when the value is the default URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cli_chat_proxy_base_url: Option<String>,
    /// Base URL for the public xAI API.
    pub xai_api_base_url: String,
    /// An extra access header value for matching first-party hosts, used only with the optional non-production feature.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alpha_test_key: Option<String>,
    /// Env: `DEEPSEEK_BUILD_MODELS_BASE_URL`. Setting it makes `has_custom_endpoint` true.
    /// The models list URL defaults to `{models_base_url}/models`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub models_base_url: Option<String>,
    /// Env: `DEEPSEEK_BUILD_MODELS_LIST_URL`. Overrides the default `{base}/models` list URL.
    #[serde(alias = "models_endpoint", skip_serializing_if = "Option::is_none")]
    pub models_list_url: Option<String>,
    /// Env: `DEEPSEEK_BUILD_FEEDBACK_BASE_URL`. Where feedback submissions go.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feedback_base_url: Option<String>,
    /// Env: `DEEPSEEK_BUILD_DEPLOYMENT_KEY`. The management API key for an enterprise deployment.
    /// Telemetry and service requests carry it to identify the deployment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deployment_key: Option<String>,
    /// Env: `GROK_MANAGED_CONFIG_URL`. The managed config endpoint.
    /// The default is `{proxy_url()}/deployment/config`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub managed_config_url: Option<String>,
    /// Env: `OTEL_EXPORTER_OTLP_ENDPOINT`. The OTLP collector base URL, before `/v1/traces` is appended.
    /// The internal trace pipeline uses it as a deprecated fallback.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub otel_exporter_otlp_endpoint: Option<String>,
    /// Env: `OTEL_EXPORTER_OTLP_TRACES_ENDPOINT`. The full traces endpoint, used verbatim and preferred over `otel_exporter_otlp_endpoint`.
    /// The internal trace pipeline treats it like `otel_exporter_otlp_endpoint`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub otel_exporter_otlp_traces_endpoint: Option<String>,
    /// Env: `OTEL_EXPORTER_OTLP_HEADERS`. Extra export headers in the form `k=v,k2=v2`.
    /// The internal trace pipeline treats it like `otel_exporter_otlp_endpoint`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub otel_exporter_otlp_headers: Option<String>,
    /// Env: `GROK_INTERNAL_OTLP_TRACES_ENDPOINT`. The full internal traces endpoint, used verbatim and preferred over the legacy `OTEL_*` vars.
    /// Developers set it to send internal spans elsewhere, for example to a local collector.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grok_internal_otlp_traces_endpoint: Option<String>,
    /// Env: `GROK_INTERNAL_OTLP_HEADERS`. Extra `k=v,k2=v2` debug headers for the internal export.
    /// They are preferred over the legacy `OTEL_EXPORTER_OTLP_HEADERS`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grok_internal_otlp_headers: Option<String>,
    /// Env: `OTEL_TRACES_EXPORTER`. `otlp` (default) or `none` to disable spans.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub otel_traces_exporter: Option<String>,
    /// Env: `OTEL_BSP_SCHEDULE_DELAY` (OTel) or `OTEL_TRACES_EXPORT_INTERVAL` (Claude alias).
    /// Batch flush interval (ms).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub otel_traces_export_interval: Option<u64>,
    /// Env: `OTEL_EXPORTER_OTLP_TIMEOUT`. Export HTTP timeout (ms).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub otel_exporter_otlp_timeout: Option<u64>,
    /// `load_management_api_key_sync()` reads this key.
    /// Declaring the field stops `serde_ignored` from reporting the key as unrecognized.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub management_api_key: Option<String>,
    /// `load_gcs_service_account_key_sync()` reads this key.
    /// Declaring the field stops `serde_ignored` from reporting the key as unrecognized.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gcs_service_account_key: Option<String>,
}
/// A blank or whitespace-only override counts as unset.
fn blank_as_unset(opt: &Option<String>) -> Option<String> {
    opt.as_deref()
        .filter(|s| !s.trim().is_empty())
        .map(str::to_owned)
}
impl EndpointsConfig {
    pub fn has_custom_endpoint(&self) -> bool {
        self.models_base_url.is_some() || self.models_list_url.is_some()
    }
    /// `default()` with the managed and requirements `[endpoints]` overrides merged on top.
    /// Startup fetches use it to reach the configured endpoints.
    pub fn from_effective_config() -> Self {
        match crate::effective_config::load_effective_config() {
            Ok(cfg) => Self::from_config_value(&cfg),
            Err(_) => Self::default(),
        }
    }
    /// Merges the `[endpoints]` table from `config` over `default()`.
    /// Only the resolver methods apply defaults.
    pub fn from_config_value(config: &toml::Value) -> Self {
        let default = Self::default();
        let mut base = match toml::Value::try_from(default) {
            Ok(v) => v,
            Err(_) => return Self::default(),
        };
        if let Some(endpoints) = config.get("endpoints") {
            crate::deep_merge_toml(&mut base, endpoints);
        }
        base.try_into().unwrap_or_default()
    }
    /// The model-proxy base URL for the auxiliary services and for inference with OAuth or session auth.
    pub fn proxy_url(&self) -> String {
        blank_as_unset(&self.cli_chat_proxy_base_url)
            .unwrap_or_else(|| CLI_CHAT_PROXY_BASE_URL_DEFAULT.to_owned())
    }
    pub fn resolve_inference_base_url(&self) -> String {
        self.models_base_url
            .clone()
            .unwrap_or_else(|| self.proxy_url())
    }
    pub fn resolve_feedback_base_url(&self) -> String {
        blank_as_unset(&self.feedback_base_url).unwrap_or_else(|| self.proxy_url())
    }
    /// The managed deployment config URL for `grok setup`.
    /// The deployment key sent here must reach the proxy, never the inference host.
    pub fn resolve_managed_config_url(&self) -> String {
        blank_as_unset(&self.managed_config_url).unwrap_or_else(|| {
            format!(
                "{}/deployment/config",
                self.proxy_url().trim_end_matches('/')
            )
        })
    }
    /// The traces endpoint for the internal pipeline, whose exports carry xAI auth.
    /// The default stays on `proxy_url` even when inference goes to another host.
    pub fn resolve_otlp_traces_endpoint(&self) -> String {
        if let Some(full) = blank_as_unset(&self.grok_internal_otlp_traces_endpoint) {
            return full.trim_end_matches('/').to_string();
        }
        if let Some(legacy) = self.legacy_internal_otlp_traces_endpoint() {
            tracing::warn!(
                "Repointing the internal trace pipeline via OTEL_EXPORTER_OTLP_ENDPOINT / \
                 OTEL_EXPORTER_OTLP_TRACES_ENDPOINT is deprecated; use \
                 GROK_INTERNAL_OTLP_TRACES_ENDPOINT instead — the standard OTEL_* vars will \
                 route the external OTEL stream only in a future release"
            );
            return legacy;
        }
        format!("{}/traces", self.proxy_url().trim_end_matches('/'))
    }
    /// The internal traces endpoint from the standard `OTEL_*` vars, if any.
    fn legacy_internal_otlp_traces_endpoint(&self) -> Option<String> {
        if let Some(full) = blank_as_unset(&self.otel_exporter_otlp_traces_endpoint) {
            return Some(full.trim_end_matches('/').to_string());
        }
        blank_as_unset(&self.otel_exporter_otlp_endpoint)
            .map(|base| format!("{}/v1/traces", base.trim_end_matches('/')))
    }
    /// Extra headers for the internal export.
    /// The fallback to `otel_exporter_otlp_headers` is kept for existing users.
    pub fn resolve_otlp_headers(&self) -> Vec<(String, String)> {
        if let Some(headers) = blank_as_unset(&self.grok_internal_otlp_headers) {
            return parse_otlp_header_list(&headers);
        }
        parse_otlp_header_list(self.otel_exporter_otlp_headers.as_deref().unwrap_or(""))
    }
    /// Whether the internal pipeline took its endpoint or headers from the standard `OTEL_EXPORTER_OTLP_*` vars.
    /// The external OTEL stream must refuse to start when this is true.
    pub fn internal_otlp_consumed_standard_vars(&self) -> bool {
        let endpoint_consumed = blank_as_unset(&self.grok_internal_otlp_traces_endpoint).is_none()
            && self.legacy_internal_otlp_traces_endpoint().is_some();
        let headers_consumed = blank_as_unset(&self.grok_internal_otlp_headers).is_none()
            && blank_as_unset(&self.otel_exporter_otlp_headers).is_some();
        endpoint_consumed || headers_consumed
    }
    /// The internal pipeline honors `OTEL_TRACES_EXPORTER=none`.
    /// Turning off internal span export is always safe.
    pub fn resolve_traces_export_enabled(&self) -> bool {
        !matches!(
            self.otel_traces_exporter.as_deref().map(str::trim),
            Some("none")
        )
    }
    /// The internal and external pipelines share this tuning value on purpose.
    pub fn resolve_otlp_export_interval(&self) -> Option<std::time::Duration> {
        self.otel_traces_export_interval
            .map(std::time::Duration::from_millis)
    }
    /// The internal and external pipelines share this tuning value on purpose.
    pub fn resolve_otlp_timeout(&self) -> Option<std::time::Duration> {
        self.otel_exporter_otlp_timeout
            .map(std::time::Duration::from_millis)
    }
    pub fn resolve_models_list_url(&self) -> String {
        if let Some(ref url) = self.models_list_url {
            return url.clone();
        }
        let base = self
            .models_base_url
            .clone()
            .unwrap_or_else(|| self.proxy_url());
        format!("{}/models", base)
    }
}
/// Parses a `k=v,k2=v2` header list, the format of `OTEL_EXPORTER_OTLP_HEADERS` and `GROK_INTERNAL_OTLP_HEADERS`.
fn parse_otlp_header_list(raw: &str) -> Vec<(String, String)> {
    raw.split(',')
        .filter_map(|kv| {
            let (k, v) = kv.split_once('=')?;
            let k = k.trim();
            (!k.is_empty()).then(|| (k.to_string(), v.trim().to_string()))
        })
        .collect()
}
/// Default OpenAI-compatible provider base URL. DeepSeek is the shipped default; users may
/// override it via `[endpoints] xai_api_base_url` or the corresponding env var.
const XAI_API_BASE_URL_DEFAULT: &str = "https://api.deepseek.com";
impl Default for EndpointsConfig {
    fn default() -> Self {
        Self {
            cli_chat_proxy_base_url: std::env::var("DEEPSEEK_BUILD_CLI_CHAT_PROXY_BASE_URL").ok(),
            xai_api_base_url: std::env::var("DEEPSEEK_BUILD_XAI_API_BASE_URL")
                .unwrap_or_else(|_| XAI_API_BASE_URL_DEFAULT.to_owned()),
            alpha_test_key: None,
            models_base_url: env_string("DEEPSEEK_BUILD_MODELS_BASE_URL"),
            models_list_url: env_string("DEEPSEEK_BUILD_MODELS_LIST_URL"),
            feedback_base_url: env_string("DEEPSEEK_BUILD_FEEDBACK_BASE_URL"),
            deployment_key: env_string("DEEPSEEK_BUILD_DEPLOYMENT_KEY"),
            managed_config_url: env_string("GROK_MANAGED_CONFIG_URL"),
            otel_exporter_otlp_endpoint: env_string("OTEL_EXPORTER_OTLP_ENDPOINT"),
            otel_exporter_otlp_traces_endpoint: env_string("OTEL_EXPORTER_OTLP_TRACES_ENDPOINT"),
            otel_exporter_otlp_headers: env_string("OTEL_EXPORTER_OTLP_HEADERS"),
            grok_internal_otlp_traces_endpoint: env_string("GROK_INTERNAL_OTLP_TRACES_ENDPOINT"),
            grok_internal_otlp_headers: env_string("GROK_INTERNAL_OTLP_HEADERS"),
            otel_traces_exporter: env_string("OTEL_TRACES_EXPORTER"),
            otel_traces_export_interval: env_string("OTEL_BSP_SCHEDULE_DELAY")
                .or_else(|| env_string("OTEL_TRACES_EXPORT_INTERVAL"))
                .and_then(|s| s.parse().ok()),
            otel_exporter_otlp_timeout: env_string("OTEL_EXPORTER_OTLP_TIMEOUT")
                .and_then(|s| s.parse().ok()),
            management_api_key: None,
            gcs_service_account_key: None,
        }
    }
}
#[cfg(test)]
#[path = "endpoints_tests.rs"]
mod tests;
