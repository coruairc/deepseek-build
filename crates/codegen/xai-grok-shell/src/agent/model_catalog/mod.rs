//! Local model catalog: resolution and selection.
//!
//! Formerly `agent/model_catalog`, which fetched `/v1/models` and cached it to
//! disk. Remote discovery has been removed; the catalog is now built from the
//! bundled defaults in `xai-grok-models` plus user `[model.<id>]` config.

mod glob;
mod resolution;
pub(crate) mod task_model_policy;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use indexmap::IndexMap;
use parking_lot::RwLock;

use agent_client_protocol as acp;

pub(crate) use glob::ModelGlobSet;
pub(crate) use resolution::{
    CatalogSource, allowlist_denied_message, available_models, resolve_catalog_key,
    resolve_default_model, resolve_model_catalog, selectable_catalog_key_for_persisted,
    task_model_error_for_catalog, validate_selectable,
};
use resolution::{
    allowlist_excludes_all_message, allowlist_matches_nothing, fallback_model_id,
    models_endpoint_empty_message,
};

use crate::agent::config::{self, ModelEntry, resolve_credentials, sampling_config_for_model};
use crate::agent::model_catalog::task_model_policy::{
    CatalogAuthority, EligibleTaskModel, TaskModelCatalogSnapshot,
};
use crate::sampling::SamplerConfig as SamplingConfig;
use xai_grok_login::{AuthManager, GrokComConfig};
use xai_grok_sampling_types::{ReasoningEffort, ReasoningEffortOption};

/// The server deployment UUID on a marker fingerprint match, else UUIDv5 of the key.
pub(crate) fn resolve_deployment_id(deployment_key: Option<&str>) -> Option<String> {
    let key = deployment_key.filter(|k| !k.is_empty())?;
    xai_grok_config::managed_deployment_id(&deployment_key_fingerprint(key))
        .or_else(|| Some(xai_grok_telemetry::config::deployment_id_from_key(key)))
}

/// The GROK_DEPLOYMENT_KEY env beats the config file. A blank value is unset.
pub(crate) fn resolve_deployment_key() -> Option<String> {
    let config_val = xai_grok_config::effective_config::load_effective_config()
        .map_err(|e| tracing::warn!("failed to load config files for deployment key: {e}"))
        .ok()
        .and_then(|root| {
            root.get("endpoints")?
                .get("deployment_key")?
                .as_str()
                .map(str::to_owned)
        });
    xai_grok_config::resolve_string_flag(None, "GROK_DEPLOYMENT_KEY", config_val.as_deref(), None)
        .map(|resolved| resolved.value)
}

/// Deterministic so the same key matches its marker; the raw key is never written to disk.
fn deployment_key_fingerprint(key: &str) -> String {
    blake3::hash(key.as_bytes()).to_hex().to_string()
}

#[derive(Clone)]
pub struct ModelsManager {
    inner: Arc<Inner>,
}

/// Catalog fields written together under one lock, so readers never see a torn mix.
#[derive(Default)]
struct CatalogState {
    models: IndexMap<String, ModelEntry>,
    /// `allowed_models` matched nothing; the prompt path blocks instead.
    allowlist_excludes_all: bool,
}

struct Inner {
    catalog: RwLock<CatalogState>,
    current_model_id: RwLock<acp::ModelId>,
    current_reasoning_effort: RwLock<Option<ReasoningEffort>>,
    auth_manager: Arc<AuthManager>,
    cfg: RwLock<config::Config>,
    gateway: RwLock<Option<xai_acp_lib::AcpAgentGatewaySender>>,
    /// Model-switch signal: a generation counter bumped when the current model id changes.
    model_switch_watch: tokio::sync::watch::Sender<u64>,
    /// Set once the user explicitly picks a model (`/model`); guards the first-catalog reselect from clobbering that choice.
    user_selected_model: AtomicBool,
}

impl Default for ModelsManager {
    fn default() -> Self {
        let grok_home = crate::util::grok_home::grok_home();
        let auth_manager = Arc::new(AuthManager::new_with_proxy_base_url(
            &grok_home,
            GrokComConfig::default(),
            crate::agent::config::EndpointsConfig::from_effective_config().proxy_url(),
        ));
        Self::new(
            IndexMap::new(),
            acp::ModelId::new("default"),
            auth_manager,
            config::Config::default(),
        )
    }
}

impl ModelsManager {
    pub(crate) fn new(
        models: IndexMap<String, ModelEntry>,
        current_model_id: acp::ModelId,
        auth_manager: Arc<AuthManager>,
        cfg: config::Config,
    ) -> Self {
        let current_reasoning_effort = cfg.models.default_reasoning_effort;
        ModelsManager {
            inner: Arc::new(Inner {
                catalog: RwLock::new(CatalogState {
                    allowlist_excludes_all: allowlist_matches_nothing(&cfg, &models),
                    models,
                }),
                current_model_id: RwLock::new(current_model_id),
                current_reasoning_effort: RwLock::new(current_reasoning_effort),
                auth_manager,
                cfg: RwLock::new(cfg),
                gateway: RwLock::new(None),
                model_switch_watch: tokio::sync::watch::channel(0u64).0,
                user_selected_model: AtomicBool::new(false),
            }),
        }
    }

    /// Build the local catalog and resolve the default model.
    pub(crate) fn from_config(
        cfg: &config::Config,
        auth_manager: Arc<AuthManager>,
    ) -> Result<Self, String> {
        let is_session_auth = auth_manager
            .current_or_expired()
            .is_some_and(|a| a.is_session_auth());
        let catalog = resolve_model_catalog(cfg, None);
        let (current_model_key, current_model, model_source) =
            resolve_default_model(cfg, &catalog, is_session_auth);

        tracing::info!(
            model_id = %current_model.model,
            source = %model_source,
            "default model resolved"
        );

        let current_model_id = acp::ModelId::new(Arc::from(current_model_key));
        Ok(Self::new(
            catalog,
            current_model_id,
            auth_manager,
            cfg.clone(),
        ))
    }

    pub(crate) fn subscribe_model_switch(&self) -> tokio::sync::watch::Receiver<u64> {
        self.inner.model_switch_watch.subscribe()
    }

    /// Cheap snapshot of the current model-switch generation, for the laziness-check poll loop.
    pub(crate) fn model_switch_generation(&self) -> u64 {
        *self.inner.model_switch_watch.borrow()
    }

    pub(crate) fn set_gateway(&self, gateway: xai_acp_lib::AcpAgentGatewaySender) {
        *self.inner.gateway.write() = Some(gateway);
    }

    /// Swap config, rebuild catalog, and reselect the model.
    pub(crate) fn apply_config(&self, new_config: config::Config) {
        if let Err(e) = new_config.validate_model_filters() {
            tracing::error!(error = %e, "ignoring config reload: invalid model filters");
            return;
        }
        let new_catalog = resolve_model_catalog(&new_config, None);
        let old_preferred = self.inner.cfg.read().models.default.clone();
        let new_preferred = new_config.models.default.clone();
        *self.inner.cfg.write() = new_config.clone();
        {
            let mut cat = self.inner.catalog.write();
            cat.allowlist_excludes_all = allowlist_matches_nothing(&new_config, &new_catalog);
            cat.models = new_catalog;
        }
        if new_preferred != old_preferred {
            self.reselect_default_model(&new_config);
        } else {
            self.reselect_current_model_if_missing(&new_config);
        }
        self.notify_models_updated();
    }

    /// [`Self::apply_config`] plus an unconditional default re-resolve.
    pub(crate) fn apply_config_reselecting_default(&self, new_config: config::Config) {
        self.apply_config(new_config.clone());
        self.reselect_default_model(&new_config);
        self.notify_models_updated();
    }

    pub fn models(&self) -> IndexMap<String, ModelEntry> {
        self.inner.catalog.read().models.clone()
    }

    /// One name without cloning the catalog, for callers on a hot path.
    pub fn display_name(&self, id: &str) -> Option<String> {
        self.inner
            .catalog
            .read()
            .models
            .get(id)
            .and_then(|entry| entry.info.name.clone())
    }

    pub fn endpoints(&self) -> config::EndpointsConfig {
        self.inner.cfg.read().endpoints.clone()
    }

    /// The local catalog is always complete: there is no remote fetch to wait for.
    pub(crate) fn is_models_fetch_enabled(&self) -> bool {
        false
    }

    /// Does the current credential grant access to OAuth-only models?
    fn is_session_auth(&self) -> bool {
        self.inner
            .auth_manager
            .current_or_expired()
            .is_some_and(|a| a.is_session_auth())
    }

    /// The picker projection of the catalog (`ModelInfo::is_picker_eligible`) in ACP wire form.
    pub fn available(&self) -> IndexMap<acp::ModelId, acp::ModelInfo> {
        let is_session_auth = self.is_session_auth();
        available_models(&self.inner.catalog.read().models, is_session_auth)
    }

    pub(crate) fn task_model_error(&self, requested: &str) -> Option<String> {
        let is_session_auth = self.is_session_auth();
        let cat = self.inner.catalog.read();
        let models = &cat.models;
        task_model_error_for_catalog(requested, models, is_session_auth)
    }

    /// One lock read, so ids, families, and the fetch state come from one catalog generation.
    pub(crate) fn task_model_catalog_snapshot(
        &self,
        _remote_fetch_enabled: bool,
    ) -> TaskModelCatalogSnapshot {
        let is_session_auth = self.is_session_auth();
        let cat = self.inner.catalog.read();
        TaskModelCatalogSnapshot {
            eligible: cat
                .models
                .iter()
                .filter(|(_, e)| e.info.is_picker_eligible(is_session_auth))
                .map(|(id, e)| EligibleTaskModel {
                    id: id.clone(),
                    model_family: e.info.model_family.clone(),
                })
                .collect(),
            authority: CatalogAuthority::Complete,
        }
    }

    pub fn current_model_id(&self) -> acp::ModelId {
        self.inner.current_model_id.read().clone()
    }

    pub(crate) fn set_current_model_id(&self, id: acp::ModelId) {
        self.inner
            .user_selected_model
            .store(true, Ordering::Relaxed);
        self.set_current_model_id_internal(id);
    }

    fn set_current_model_id_internal(&self, id: acp::ModelId) {
        let changed = {
            let mut cur = self.inner.current_model_id.write();
            let changed = *cur != id;
            *cur = id;
            changed
        };
        if changed {
            self.inner
                .model_switch_watch
                .send_modify(|generation| *generation += 1);
        }
    }

    /// Per-model Layer-3 LazinessDetector config for `model_id` (disabled default when absent).
    pub(crate) fn laziness_detector_for(
        &self,
        model_id: &str,
    ) -> config::LazinessDetectorPerModelConfig {
        self.inner
            .catalog
            .read()
            .models
            .get(model_id)
            .map(|e| e.info().laziness_detector.clone())
            .unwrap_or_default()
    }

    #[cfg(test)]
    pub(crate) fn insert_test_entry(&self, id: impl Into<String>, entry: ModelEntry) {
        self.inner.catalog.write().models.insert(id.into(), entry);
    }

    pub(crate) fn current_reasoning_effort(&self) -> Option<ReasoningEffort> {
        *self.inner.current_reasoning_effort.read()
    }

    pub(crate) fn set_current_reasoning_effort(&self, effort: Option<ReasoningEffort>) {
        *self.inner.current_reasoning_effort.write() = effort;
    }

    /// Run `f` on the [`ModelEntry`] for `model_id` (catalog key or wire name); `None` if absent.
    fn with_catalog_entry<T>(&self, model_id: &str, f: impl FnOnce(&ModelEntry) -> T) -> Option<T> {
        let cat = self.inner.catalog.read();
        let models = &cat.models;
        let key = resolve_catalog_key(models, &acp::ModelId::new(model_id))?;
        models.get(key.0.as_ref()).map(f)
    }

    /// Whether `model_id` is served by the same endpoint, backend, and session auth as a config already built for another model.
    pub(crate) fn model_shares_route(
        &self,
        model_id: &str,
        base_url: &str,
        api_backend: &xai_grok_sampling_types::ApiBackend,
    ) -> bool {
        let base_url = base_url.trim_end_matches('/');
        self.with_catalog_entry(model_id, |e| {
            !e.has_own_credentials()
                && e.info().api_backend == *api_backend
                && (e.info().base_url.trim_end_matches('/') == base_url
                    || e.api_base_url
                        .as_deref()
                        .is_some_and(|u| u.trim_end_matches('/') == base_url))
        })
        .unwrap_or(false)
    }

    pub(crate) fn model_supports_reasoning_effort(&self, model_id: &str) -> bool {
        self.with_catalog_entry(model_id, |e| e.info().supports_reasoning_effort)
            .unwrap_or(false)
    }

    /// Looks `model_id` up in the catalog, then returns the id that model sends at this effort.
    pub(crate) fn model_for_effort(
        &self,
        model_id: &str,
        effort: ReasoningEffort,
    ) -> Option<String> {
        self.with_catalog_entry(model_id, |e| e.info().model_at(effort).to_string())
    }

    pub(crate) fn model_default_reasoning_effort(&self, model_id: &str) -> Option<ReasoningEffort> {
        self.with_catalog_entry(model_id, |e| e.info().reasoning_effort)
            .flatten()
    }

    /// The raw catalog `reasoning_efforts` list for `model_id` with no fallback.
    pub(crate) fn model_reasoning_efforts(&self, model_id: &str) -> Vec<ReasoningEffortOption> {
        self.with_catalog_entry(model_id, |e| e.info().reasoning_efforts.clone())
            .unwrap_or_default()
    }

    pub(crate) fn model_supports_reasoning_effort_value(
        &self,
        model_id: &str,
        effort: ReasoningEffort,
    ) -> bool {
        let options = self.model_reasoning_efforts(model_id);
        if options.is_empty() {
            return self.model_supports_reasoning_effort(model_id)
                && matches!(
                    effort,
                    ReasoningEffort::Low
                        | ReasoningEffort::Medium
                        | ReasoningEffort::High
                        | ReasoningEffort::Xhigh
                );
        }
        options.iter().any(|option| option.value == effort)
    }

    pub(crate) fn model_supports_context_window(
        &self,
        model_id: &str,
        window: std::num::NonZeroU64,
    ) -> bool {
        self.with_catalog_entry(model_id, |e| e.info().supports_context_window(window))
            .unwrap_or(false)
    }

    pub(crate) fn model_supports_backend_search(&self, model_id: &str) -> bool {
        self.inner
            .catalog
            .read()
            .models
            .get(model_id)
            .map(|e| e.info().supports_backend_search)
            .unwrap_or(false)
    }

    pub(crate) fn model_compactions_remaining(
        &self,
        model_id: &str,
    ) -> Option<xai_grok_sampling_types::CompactionsRemaining> {
        self.inner
            .catalog
            .read()
            .models
            .get(model_id)
            .and_then(|e| e.info().compactions_remaining)
    }

    pub(crate) fn model_compaction_at_tokens(
        &self,
        model_id: &str,
    ) -> Option<xai_grok_sampling_types::CompactionAtTokens> {
        self.inner
            .catalog
            .read()
            .models
            .get(model_id)
            .and_then(|e| e.info().compaction_at_tokens)
    }

    /// Catalog opt-in to display the served-checkpoint fingerprint for this model.
    pub(crate) fn model_show_model_fingerprint(&self, model_id: &str) -> bool {
        self.with_catalog_entry(model_id, |e| e.info().show_model_fingerprint)
            .unwrap_or(false)
    }

    pub(crate) fn prompt_suggest_model_pin(&self) -> crate::config::PromptSuggestModelPin {
        self.inner.cfg.read().prompt_suggest_model_pin.clone()
    }

    /// The catalog key `model_id` resolves to, as a config key or a routing slug.
    pub(crate) fn catalog_key(&self, model_id: &str) -> Option<acp::ModelId> {
        resolve_catalog_key(
            &self.inner.catalog.read().models,
            &acp::ModelId::new(model_id),
        )
    }

    /// Whether `model_id` resolves in the current catalog, as a config key or a routing slug.
    pub(crate) fn model_in_catalog(&self, model_id: &str) -> bool {
        self.catalog_key(model_id).is_some()
    }

    /// The local catalog is always available; nothing to wait for.
    pub(crate) async fn wait_for_first_catalog(&self, _remote_fetch_enabled: bool) -> bool {
        true
    }

    /// Auth identity changed: rebuild the bundled catalog and reselect a valid default.
    pub(crate) async fn on_auth_changed(&self) {
        let config = self.inner.cfg.read().clone();
        crate::agent::init::update_telemetry_config(&config, &self.inner.auth_manager);
        self.rebuild_bundled(&config);
        self.notify_models_updated();
    }

    fn notify_models_updated(&self) {
        let available = self.available();
        let current = self.current_model_id();
        let count = available.len();
        xai_grok_telemetry::unified_log::info(
            "model catalog: notifying clients",
            None,
            Some(serde_json::json!({
                "model_count": count,
                "current_model_id": current.0.as_ref(),
            })),
        );
        if let Some(ref gw) = *self.inner.gateway.read() {
            let model_state =
                acp::SessionModelState::new(current, available.values().cloned().collect());
            if let Ok(params) = serde_json::value::to_raw_value(&model_state) {
                gw.forward_fire_and_forget(acp::ExtNotification::new(
                    "deepseek-build/models/update",
                    params.into(),
                ));
            }
        }
    }

    /// No-op: the on-disk remote models cache no longer exists.
    pub(crate) fn reload_from_disk_cache(&self) {}

    /// No-op: no remote etag to reconcile.
    pub(crate) async fn refresh_if_new_etag(&self, _etag: String) {}

    /// No-op: the catalog is local and complete.
    pub fn spawn_background_refresh(&self) {}

    /// No-op: there is no auth-refresh fetch watcher.
    pub fn start_auth_refresh_watcher(&self, _notify: Arc<tokio::sync::Notify>) {}

    fn rebuild_bundled(&self, cfg: &config::Config) {
        self.inner.catalog.write().models = resolve_model_catalog(cfg, None);
        self.reselect_current_model_if_missing(cfg);
    }

    /// Build a `SamplingConfig` from the current model and auth state.
    pub fn sampling_config(&self) -> SamplingConfig {
        let config = self.inner.cfg.read().clone();
        let auth_manager = self.inner.auth_manager.as_ref();
        let current_model_id = self.current_model_id();
        let all_models = self.models();
        let fallback;
        let current_model = match all_models
            .get(current_model_id.0.as_ref())
            .or_else(|| all_models.values().next())
        {
            Some(m) => m,
            None => {
                let current = Some(current_model_id.0.as_ref()).filter(|id| !id.is_empty());
                let default_id = fallback_model_id(&config, current);
                tracing::warn!(model = %default_id, "no models available in catalog; using the fallback model id");
                fallback = ModelEntry::fallback(&default_id, &config.endpoints);
                &fallback
            }
        };

        let session_auth = auth_manager.current_or_expired();
        let credentials =
            resolve_credentials(current_model, session_auth.as_ref().map(|a| a.key.as_str()));

        sampling_config_for_model(
            current_model,
            credentials,
            config.endpoints.alpha_test_key.clone(),
            config.client_version.clone(),
            resolve_deployment_id(config.endpoints.deployment_key.as_deref()),
            None,
        )
    }

    pub fn allowlist_excludes_all(&self) -> bool {
        self.inner.catalog.read().allowlist_excludes_all
    }

    /// The message that stops a prompt because no model can serve it, or `None`.
    pub(crate) async fn prompt_block_message(&self) -> Option<String> {
        if CatalogSource::for_config(&self.inner.cfg.read()) == CatalogSource::Standard {
            return self
                .allowlist_excludes_all()
                .then(|| allowlist_excludes_all_message(&self.inner.cfg.read()));
        }
        let empty = self.inner.catalog.read().models.is_empty();
        let cfg = self.inner.cfg.read();
        if empty {
            return Some(models_endpoint_empty_message(&cfg));
        }
        allowlist_matches_nothing(&cfg, &self.inner.catalog.read().models)
            .then(|| allowlist_excludes_all_message(&cfg))
    }

    /// Re-pick the default when the current model is gone or unselectable; auth visibility never evicts an explicit user pick.
    fn reselect_current_model_if_missing(&self, config: &config::Config) {
        let current = self.inner.current_model_id.read().clone();
        let user_selected = self.inner.user_selected_model.load(Ordering::Relaxed);
        let needs_reselection = {
            let cat = self.inner.catalog.read();
            let models = &cat.models;
            match models.get(current.0.as_ref()) {
                None => true,
                Some(entry) => {
                    !entry.info.user_selectable
                        || (!user_selected && !entry.info.visible_for_auth(self.is_session_auth()))
                }
            }
        };
        if !needs_reselection {
            return;
        }
        let (key, _, source) = {
            let cat = self.inner.catalog.read();
            let models = &cat.models;
            resolve_default_model(config, models, self.is_session_auth())
        };
        let new_id = acp::ModelId::new(Arc::from(key));
        tracing::info!(
            old = %current.0, new = %new_id.0, source = %source,
            "current model not in new catalog, reselecting default"
        );
        self.set_current_model_id_internal(new_id);
    }

    fn reselect_default_model(&self, config: &config::Config) {
        let (key, _, source) = {
            let cat = self.inner.catalog.read();
            let models = &cat.models;
            resolve_default_model(config, models, self.is_session_auth())
        };
        let new_id = acp::ModelId::new(Arc::from(key));
        let current = self.inner.current_model_id.read().clone();
        if current.0.as_ref() != new_id.0.as_ref() {
            tracing::info!(
                old = %current.0, new = %new_id.0, source = %source,
                "re-resolved default model after catalog populated"
            );
            self.set_current_model_id_internal(new_id);
        }
    }
}
