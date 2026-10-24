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
            management_api_key: None,
            gcs_service_account_key: None,
        }
    }
}
#[cfg(test)]
#[path = "endpoints_tests.rs"]
mod tests;
