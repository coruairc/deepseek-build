//! Model-id resolution: catalog keys, routing slugs, and selection.

use indexmap::IndexMap;

use super::ModelGlobSet;
use crate::agent::config::{self, ModelEntry};
use agent_client_protocol as acp;
use xai_grok_sampling_types::ReasoningEffort;

/// Which sources feed the model catalog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CatalogSource {
    /// The local bundled catalog plus every `[model.<id>]` table.
    Standard,
    /// External auth with a configured models endpoint. Built-in and bundled models never appear.
    ModelsEndpoint,
}

impl CatalogSource {
    pub(crate) fn for_config(cfg: &config::Config) -> Self {
        if cfg.grok_com_config.auth_provider_command.is_some()
            && cfg.endpoints.has_custom_endpoint()
        {
            CatalogSource::ModelsEndpoint
        } else {
            CatalogSource::Standard
        }
    }
}

/// The id used when the catalog has no usable model.
pub(crate) fn fallback_model_id(cfg: &config::Config, preferred: Option<&str>) -> String {
    match CatalogSource::for_config(cfg) {
        CatalogSource::Standard => crate::models::default_model().to_owned(),
        CatalogSource::ModelsEndpoint => preferred.unwrap_or_default().to_owned(),
    }
}

/// Map a model id (catalog key or routing slug) to its catalog key.
pub(crate) fn resolve_catalog_key(
    models: &IndexMap<String, ModelEntry>,
    id: &acp::ModelId,
) -> Option<acp::ModelId> {
    let id_str = id.0.as_ref();
    if models.contains_key(id_str) {
        return Some(id.clone());
    }
    models
        .iter()
        .rev()
        .find(|(_, entry)| entry.info.has_model_id(id_str))
        .map(|(key, _)| acp::ModelId::new(key.clone()))
}

/// Catalog key for a persisted session model id, restricted to **selectable** entries.
pub(crate) fn selectable_catalog_key_for_persisted(
    models: &IndexMap<String, ModelEntry>,
    available: &IndexMap<acp::ModelId, acp::ModelInfo>,
    id: &acp::ModelId,
) -> Option<acp::ModelId> {
    if available.contains_key(id) {
        return Some(id.clone());
    }
    let id_str = id.0.as_ref();
    if let Some((key, _)) = models.iter().rev().find(|(key, entry)| {
        available.contains_key(&acp::ModelId::new((*key).clone()))
            && entry.info.has_model_id(id_str)
    }) {
        return Some(acp::ModelId::new(key.clone()));
    }
    resolve_catalog_key(models, id).filter(|key| available.contains_key(key))
}

/// Pick the default model: CLI > env > config, falling back to the first visible model, then [`fallback_model_id`].
pub(crate) fn resolve_default_model(
    cfg: &config::Config,
    catalog: &IndexMap<String, ModelEntry>,
    is_session_auth: bool,
) -> (String, ModelEntry, config::ConfigSource) {
    let visible: IndexMap<String, ModelEntry> = catalog
        .iter()
        .filter(|(_, e)| e.info.visible_for_auth(is_session_auth) && e.info.user_selectable)
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    let model_pref = config::resolve_string_flag(
        cfg.default_model_override.as_deref(),
        "GROK_DEFAULT_MODEL",
        cfg.models.default.as_deref(),
        None,
    );

    let first_or_fallback = || -> (String, ModelEntry) {
        if let Some((key, first)) = visible.first() {
            return (key.clone(), first.clone());
        }
        if let Some((key, entry)) = catalog.iter().find(|(_, e)| e.info.user_selectable) {
            tracing::warn!("no auth-visible selectable model; using first selectable entry");
            return (key.clone(), entry.clone());
        }
        let local_pref = model_pref.as_ref().filter(|p| {
            matches!(
                p.source,
                config::ConfigSource::Cli
                    | config::ConfigSource::Env
                    | config::ConfigSource::Config
            )
        });
        let default_id = fallback_model_id(cfg, local_pref.map(|p| p.value.as_str()));
        tracing::warn!(model = %default_id, "no selectable models; using the fallback model id");
        let mut entry = ModelEntry::fallback(&default_id, &cfg.endpoints);
        entry.info.user_selectable = model_is_allowlisted(cfg, &default_id, &default_id);
        (default_id, entry)
    };

    match &model_pref {
        None => {
            let (key, first) = first_or_fallback();
            (key, first, config::ConfigSource::Default)
        }
        Some(pref) => {
            let found = visible
                .get_key_value(&pref.value)
                .or_else(|| visible.iter().find(|(_, m)| m.has_model_id(&pref.value)));

            if let Some((key, entry)) = found {
                (key.clone(), entry.clone(), pref.source)
            } else {
                let is_explicit = matches!(
                    pref.source,
                    config::ConfigSource::Cli
                        | config::ConfigSource::Env
                        | config::ConfigSource::Config
                );
                if is_explicit {
                    tracing::warn!(
                        model_id = %pref.value, source = %pref.source,
                        "preferred model not in available models, falling back"
                    );
                }
                let (key, first) = first_or_fallback();
                (key, first, config::ConfigSource::Default)
            }
        }
    }
}

/// Keep the picker projection of `catalog` (`ModelInfo::is_picker_eligible`) in ACP wire format.
pub(crate) fn available_models(
    catalog: &IndexMap<String, ModelEntry>,
    is_session_auth: bool,
) -> IndexMap<acp::ModelId, acp::ModelInfo> {
    let visible: IndexMap<String, ModelEntry> = catalog
        .iter()
        .filter(|(_, e)| e.info.is_picker_eligible(is_session_auth))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    config::to_acp_model_info(&visible)
}

/// Resolved allowlist: fleet pin, user/project list, or unrestricted.
enum EffectiveAllowlist<'a> {
    Unrestricted,
    Invalid,
    User(&'a [String]),
    Fleet(&'a [String]),
}

fn effective_allowlist(cfg: &config::Config) -> EffectiveAllowlist<'_> {
    use crate::agent::config::AllowlistPin;
    match cfg.requirements.allowed_models.pin_ref() {
        Some(AllowlistPin::FailClosed) => EffectiveAllowlist::Invalid,
        Some(AllowlistPin::List(patterns)) if patterns.is_empty() => {
            EffectiveAllowlist::Unrestricted
        }
        Some(AllowlistPin::List(patterns)) => EffectiveAllowlist::Fleet(patterns),
        None => match cfg.models.allowed_models.as_deref() {
            Some(patterns) if !patterns.is_empty() => EffectiveAllowlist::User(patterns),
            _ => EffectiveAllowlist::Unrestricted,
        },
    }
}

impl EffectiveAllowlist<'_> {
    fn is_unrestricted(&self) -> bool {
        matches!(self, Self::Unrestricted)
    }

    fn is_fleet(&self) -> bool {
        matches!(self, Self::Fleet(_) | Self::Invalid)
    }

    fn is_selected(&self, key: &str, model: &str) -> bool {
        match self {
            Self::Unrestricted => true,
            Self::Invalid => false,
            Self::Fleet(patterns) | Self::User(patterns) => {
                match ModelGlobSet::compile(Some(patterns)) {
                    Ok(None) => true,
                    Ok(Some(set)) => {
                        if matches!(self, Self::Fleet(_)) {
                            set.matches_model(model)
                        } else {
                            set.matches(key, model)
                        }
                    }
                    Err(_) => false,
                }
            }
        }
    }

    fn apply_selectability(&self, catalog: &mut IndexMap<String, ModelEntry>) {
        match self {
            Self::Unrestricted => {
                for entry in catalog.values_mut() {
                    entry.info.user_selectable = true;
                }
            }
            Self::Invalid => {
                for entry in catalog.values_mut() {
                    entry.info.user_selectable = false;
                }
            }
            Self::Fleet(patterns) | Self::User(patterns) => {
                match ModelGlobSet::compile(Some(patterns)) {
                    Ok(None) => {
                        for entry in catalog.values_mut() {
                            entry.info.user_selectable = true;
                        }
                    }
                    Ok(Some(set)) => {
                        let fleet = matches!(self, Self::Fleet(_));
                        for (key, entry) in catalog.iter_mut() {
                            entry.info.user_selectable = if fleet {
                                set.matches_model(&entry.model)
                            } else {
                                set.matches(key, &entry.model)
                            };
                        }
                    }
                    Err(bad) => {
                        tracing::error!(
                            patterns = ?bad,
                            "allowed_models: invalid glob(s); marking nothing selectable"
                        );
                        for entry in catalog.values_mut() {
                            entry.info.user_selectable = false;
                        }
                    }
                }
            }
        }
    }
}

/// Catalog-key match is user-config only. A fleet pin matches the routing
/// slug so a user `[model.deepseek-4-anything]` cannot satisfy `deepseek-4*`.
fn model_is_allowlisted(cfg: &config::Config, key: &str, model: &str) -> bool {
    effective_allowlist(cfg).is_selected(key, model)
}

pub(crate) fn allowlist_denied_message(cfg: &config::Config) -> &'static str {
    if effective_allowlist(cfg).is_fleet() {
        "This model isn't allowed by your organization's policy. Contact your administrator."
    } else {
        "This model isn't allowed by your allowed_models setting."
    }
}

pub(crate) fn allowlist_excludes_all_message(cfg: &config::Config) -> String {
    match effective_allowlist(cfg) {
        EffectiveAllowlist::Invalid => {
            "The organization model policy is invalid. Contact your administrator.".to_owned()
        }
        EffectiveAllowlist::Fleet(_) => {
            "None of your models are allowed by your organization's policy. Contact your administrator."
                .to_owned()
        }
        _ => "None of your models are allowed by allowed_models. \
             Broaden it or remove it from your config, then restart."
            .to_owned(),
    }
}

/// Single source of truth for the catalog.
/// It keeps the sources [`CatalogSource`] allows, then applies `disabled_models`, `allowed_models`, and `hidden_models`.
pub(crate) fn resolve_model_catalog(
    cfg: &config::Config,
    prefetched: Option<IndexMap<String, ModelEntry>>,
) -> IndexMap<String, ModelEntry> {
    let mut catalog: IndexMap<String, ModelEntry> = config::resolve_model_list(cfg, prefetched);

    if let Ok(Some(disabled)) = ModelGlobSet::compile(cfg.models.disabled_models.as_deref()) {
        let before = catalog.len();
        catalog.retain(|key, entry| !disabled.matches(key, &entry.model));
        let removed = before - catalog.len();
        if removed > 0 {
            tracing::info!(count = removed, "disabled_models: removed from catalog");
        }
    }

    effective_allowlist(cfg).apply_selectability(&mut catalog);

    if let Ok(Some(hidden)) = ModelGlobSet::compile(cfg.models.hidden_models.as_deref()) {
        for (key, entry) in catalog.iter_mut() {
            if hidden.matches(key, &entry.model) {
                entry.info.hidden = true;
            }
        }
    }

    if let Some(effort) = cfg.models.default_reasoning_effort
        && let Some(default_id) = cfg.models.default.as_deref()
        && let Some(entry) = catalog.get_mut(default_id)
        && entry.info.supports_reasoning_effort
    {
        stamp_effort(&mut entry.info, effort);
    }

    if let Some(effort) = cfg.reasoning_effort_override {
        for entry in catalog.values_mut() {
            if model_offers_reasoning_effort(&entry.info, effort) {
                stamp_effort(&mut entry.info, effort);
            }
        }
    }

    catalog
}

/// The entry keeps its own model id, and `model_at` picks the id for this effort when a request is prepared.
fn stamp_effort(info: &mut config::ModelInfo, effort: ReasoningEffort) {
    info.reasoning_effort = Some(effort);
}

/// Whether `effort` is a value this model will accept on the wire.
fn model_offers_reasoning_effort(info: &config::ModelInfo, effort: ReasoningEffort) -> bool {
    if !info.supports_reasoning_effort {
        return false;
    }
    if info.reasoning_efforts.is_empty() {
        matches!(
            effort,
            ReasoningEffort::Low
                | ReasoningEffort::Medium
                | ReasoningEffort::High
                | ReasoningEffort::Xhigh
        )
    } else {
        info.reasoning_efforts.iter().any(|opt| opt.value == effort)
    }
}

/// True when an active `allowed_models` allowlist leaves no selectable model.
pub(crate) fn allowlist_matches_nothing(
    cfg: &config::Config,
    catalog: &IndexMap<String, ModelEntry>,
) -> bool {
    !effective_allowlist(cfg).is_unrestricted() && !catalog.values().any(|e| e.info.user_selectable)
}

/// The message that blocks prompts when the catalog is empty.
pub(crate) fn models_endpoint_empty_message(cfg: &config::Config) -> String {
    let url = cfg.endpoints.resolve_models_list_url();
    format!(
        "No models are available: {url} returned none or could not be reached. \
         Check the endpoint and your login, then try again."
    )
}

/// The error a subagent `Task.model` slug produces when it is not a selectable catalog entry.
pub(crate) fn task_model_error_for_catalog(
    requested: &str,
    available: &IndexMap<String, ModelEntry>,
    is_session_auth: bool,
) -> Option<String> {
    let is_available = |entry: &ModelEntry| entry.info.is_picker_eligible(is_session_auth);
    if config::find_model_by_id(available, requested).is_some_and(&is_available) {
        return None;
    }

    let mut slugs = available
        .iter()
        .filter(|(_, entry)| is_available(entry))
        .map(|(slug, _)| slug.as_str())
        .collect::<Vec<_>>();
    slugs.sort_unstable();
    let guidance = if slugs.is_empty() {
        "No valid model slugs are currently available. Omit `model` to inherit the parent model."
            .to_string()
    } else {
        format!(
            "Valid model slugs: {}. Omit `model` to inherit the parent model.",
            slugs.join(", ")
        )
    };
    Some(format!("Unknown Task.model slug '{requested}'. {guidance}"))
}

/// Reject an `allowed_models` allowlist that leaves no selectable model, or excludes an explicitly configured default.
pub(crate) fn validate_selectable(
    cfg: &config::Config,
    catalog: &IndexMap<String, ModelEntry>,
) -> Result<(), String> {
    let allowlist = effective_allowlist(cfg);
    match allowlist {
        EffectiveAllowlist::Unrestricted => return Ok(()),
        EffectiveAllowlist::Invalid => {
            return Err(
                "The organization model policy is invalid. Contact your administrator.".to_owned(),
            );
        }
        EffectiveAllowlist::Fleet(_) | EffectiveAllowlist::User(_) => {}
    }
    if !catalog.values().any(|e| e.info.user_selectable) {
        return Err(allowlist_excludes_all_message(cfg));
    }
    for (src, id) in [
        ("default", cfg.models.default.as_deref()),
        ("-m flag", cfg.default_model_override.as_deref()),
    ] {
        if let Some(id) = id
            && let Some(entry) = catalog
                .get(id)
                .or_else(|| catalog.values().find(|e| e.has_model_id(id)))
            && !entry.info.user_selectable
        {
            return Err(if allowlist.is_fleet() {
                format!(
                    "\"{id}\" (your {src}) isn't allowed by your organization's policy. \
                     Contact your administrator."
                )
            } else {
                format!(
                    "\"{id}\" (your {src}) isn't allowed by allowed_models. \
                     Broaden the patterns or remove allowed_models, then try again."
                )
            });
        }
    }
    Ok(())
}
