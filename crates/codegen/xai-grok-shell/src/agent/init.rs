//! Agent bootstrap and lifecycle hooks.
//!
//! [`bootstrap`] runs the full init sequence (config resolution, process
//! singletons, model catalog) and returns a resolved config + `ModelsManager`.
//! [`update_telemetry_config`] re-initializes telemetry after auth changes.
use crate::agent::config::{self, Config as AgentConfig, ModelEntry};
use crate::agent::model_catalog::ModelsManager;
use crate::config::StorageMode;
use indexmap::IndexMap;
use std::sync::{Arc, Mutex, TryLockError};
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use xai_grok_login::{AuthManager, GrokAuth};
/// A bootstrap failure, stringified only at the process boundary.
#[derive(Debug, thiserror::Error)]
pub enum BootstrapError {
    #[error("{0}")]
    Config(String),
    #[error("bootstrap cancelled")]
    Cancelled,
}
impl From<String> for BootstrapError {
    fn from(message: String) -> Self {
        Self::Config(message)
    }
}
/// The owned handoff from the async boot pre-resolve to sync bootstrap.
/// Remote settings and model prefetch were removed, so this carries no payload;
/// it exists so the async pre-resolve call sites keep a stable shape.
#[must_use]
pub struct BootstrapPrefetch {
    _private: (),
}
/// One bootstrap at a time. A connect-timeout drop does not abort
/// `spawn_blocking`, so the fallback connect would otherwise overlap
/// `init_process`.
static BOOTSTRAP_GATE: Mutex<()> = Mutex::new(());
struct BootstrapPermit<'a>(#[expect(dead_code)] std::sync::MutexGuard<'a, ()>);
fn ensure_bootstrap_not_cancelled(cancel: &CancellationToken) -> Result<(), BootstrapError> {
    if cancel.is_cancelled() {
        Err(BootstrapError::Cancelled)
    } else {
        Ok(())
    }
}
/// Spin on [`BOOTSTRAP_GATE`] so a cancelled waiter can bail instead of
/// blocking forever behind a worker that is itself winding down.
fn acquire_bootstrap_gate(
    cancel: &CancellationToken,
) -> Result<BootstrapPermit<'_>, BootstrapError> {
    loop {
        match BOOTSTRAP_GATE.try_lock() {
            Ok(guard) => return Ok(BootstrapPermit(guard)),
            Err(TryLockError::Poisoned(poisoned)) => {
                return Ok(BootstrapPermit(poisoned.into_inner()));
            }
            Err(TryLockError::WouldBlock) => {
                ensure_bootstrap_not_cancelled(cancel)?;
                std::thread::sleep(Duration::from_millis(10));
            }
        }
    }
}
#[cfg(test)]
pub(crate) fn hold_bootstrap_gate_for_tests() -> std::sync::MutexGuard<'static, ()> {
    loop {
        match BOOTSTRAP_GATE.try_lock() {
            Ok(guard) => return guard,
            Err(TryLockError::Poisoned(poisoned)) => return poisoned.into_inner(),
            Err(TryLockError::WouldBlock) => std::thread::sleep(Duration::from_millis(10)),
        }
    }
}
/// Resolve config, init process singletons, build the local model catalog.
pub fn bootstrap(
    cfg: &AgentConfig,
    auth_manager: &Arc<AuthManager>,
    _prefetched: Option<IndexMap<String, ModelEntry>>,
) -> Result<(AgentConfig, ModelsManager), BootstrapError> {
    bootstrap_with_cancel(
        cfg,
        auth_manager,
        _prefetched,
        &CancellationToken::new(),
        None,
    )
}
/// [`bootstrap`] that stops at phase boundaries when `cancel` fires.
pub fn bootstrap_with_cancel(
    cfg: &AgentConfig,
    auth_manager: &Arc<AuthManager>,
    _prefetched: Option<IndexMap<String, ModelEntry>>,
    cancel: &CancellationToken,
    _boot: Option<BootstrapPrefetch>,
) -> Result<(AgentConfig, ModelsManager), BootstrapError> {
    let _permit = acquire_bootstrap_gate(cancel)?;
    ensure_bootstrap_not_cancelled(cancel)?;
    xai_grok_telemetry::id::prefetch_agent_id();
    xai_grok_telemetry::startup::enter(xai_grok_telemetry::startup::StartupPhase::Bootstrap);
    let cfg = cfg.clone();
    let cfg = {
        let mut timer = crate::instrumentation_timer!("startup.bootstrap.resolve_config");
        timer.with_subphase(xai_grok_telemetry::startup::Subphase::ResolveConfig);
        let cfg = resolve_config(&cfg, auth_manager);
        cfg.validate_model_filters()?;
        cfg
    };
    ensure_bootstrap_not_cancelled(cancel)?;
    {
        let mut timer = crate::instrumentation_timer!("startup.bootstrap.init_process");
        timer.with_subphase(xai_grok_telemetry::startup::Subphase::InitProcess);
        init_process(&cfg, auth_manager);
    }
    ensure_bootstrap_not_cancelled(cancel)?;
    xai_grok_telemetry::startup::enter(xai_grok_telemetry::startup::StartupPhase::ModelCatalog);
    let models_manager = {
        let mut timer = crate::instrumentation_timer!("startup.model_catalog.models_manager");
        timer.with_subphase(xai_grok_telemetry::startup::Subphase::ModelsManager);
        ModelsManager::from_config(&cfg, auth_manager.clone())?
    };
    Ok((cfg, models_manager))
}
/// Prints the error to the user's real stderr (undoing any TUI redirect) and exits.
pub(crate) fn exit_on_config_error<T>(e: BootstrapError) -> T {
    xai_tty_utils::restore_native_stderr();
    eprintln!("\nConfiguration error:\n\n    {e}\n");
    std::process::exit(1);
}
/// No-op: remote settings and model prefetch were removed, so the async
/// pre-resolve carries nothing into sync bootstrap.
pub async fn resolve_boot_startup_settings(
    _cfg: &mut AgentConfig,
    _cancel: &CancellationToken,
    _start_models_prefetch: bool,
    _warmed_auth: Option<GrokAuth>,
) -> Result<BootstrapPrefetch, BootstrapError> {
    Ok(BootstrapPrefetch { _private: () })
}
fn resolve_config(cfg: &AgentConfig, auth_manager: &AuthManager) -> AgentConfig {
    let mut cfg = cfg.clone();
    if let Ok(layers) = crate::config::ConfigLayers::load()
        && layers.has_managed()
    {
        let origins = crate::config::config_origins(&layers);
        let managed_keys: Vec<&str> = origins
            .iter()
            .filter(|(_, s)| matches!(s, config::ConfigSource::ManagedConfig))
            .map(|(k, _)| k.as_str())
            .collect();
        if !managed_keys.is_empty() {
            tracing::info!(keys = ?managed_keys, "managed_config.toml fields");
        }
    }
    crate::config::apply_policy(&mut cfg);
    let has_xai_auth = auth_manager.current().is_some_and(|a| a.is_xai_auth());
    if cfg.storage_mode == StorageMode::Local
        && cfg.mode != crate::agent::config::AgentMode::Generic
    {
        cfg.storage_mode =
            StorageMode::from_remote_gated(cfg.remote_settings.as_ref(), has_xai_auth);
    }
    if cfg.storage_mode == StorageMode::Writeback && !has_xai_auth {
        tracing::info!("Writeback is disabled: requires a signed-in session");
        cfg.storage_mode = StorageMode::Local;
    }
    cfg
}
/// Initialize process-level singletons (built-in metadata, telemetry).
/// `Once`-guarded: only the first call takes effect.
/// Telemetry user ID is updated separately via [`update_telemetry_config`].
fn init_process(cfg: &AgentConfig, auth_manager: &AuthManager) {
    use std::sync::Once;
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        xai_grok_telemetry::unified_log::set_version(xai_grok_version::VERSION);
        let limits = crate::util::limits::ProcessLimits::read();
        limits.log();
        let grok_home = crate::util::grok_home::grok_home();
        crate::builtin::extract_builtin_files(&grok_home);
        if !cfg!(test) {
            crate::builtin::purge_stale_extracted_skills(&grok_home);
        }
        crate::extensions::marketplace::purge_default_skills_installs(&grok_home);
        if cfg.resolve_official_marketplace_auto_register().value {
            crate::extensions::marketplace::ensure_official_marketplace_source(&grok_home);
        }
        let telemetry_mode = cfg.resolve_telemetry_mode();
        let trace_upload = cfg.resolve_trace_upload();
        let feedback = cfg.feature(config::Feature::Feedback);
        let feedback_url = cfg.endpoints.resolve_feedback_base_url();
        let trace_upload_url = cfg.endpoints.resolve_trace_upload_url();
        tracing::info!(
            telemetry = %telemetry_mode,
            trace_upload = %trace_upload,
            feedback = %feedback,
            feedback_url = %feedback_url,
            feedback_url_custom = cfg.endpoints.feedback_base_url.is_some(),
            trace_upload_url = %trace_upload_url,
            trace_upload_url_custom = cfg.endpoints.trace_upload_url.is_some(),
            trace_upload_bucket = cfg.endpoints.trace_upload_bucket.as_deref().unwrap_or("none"),
            trace_upload_region = cfg.endpoints.trace_upload_region.as_deref().unwrap_or("none"),
            "data capture config resolved",
        );
        if telemetry_mode.value.is_disabled() && trace_upload.value {
            tracing::info!(
                "Telemetry disabled but trace uploads enabled: \
                 session artifacts will be uploaded, analytics events will not"
            );
        }
        update_telemetry_config(cfg, auth_manager);
        xai_grok_telemetry::session_ctx::log_event(limits.into_event());
    });
}
/// Apply current telemetry config + auth identity. Tears down the client
/// when telemetry is disabled, so it's safe to call repeatedly.
pub fn update_telemetry_config(config: &AgentConfig, auth_manager: &AuthManager) {
    let user_agent = crate::http::process_user_agent_string();
    if reqwest::header::HeaderValue::from_str(&user_agent).is_err() {
        tracing::warn!("telemetry init skipped: GROK_CLIENT_NAME yields an invalid user agent");
        return;
    }
    let grok_auth = auth_manager.current().filter(|a| a.is_xai_auth());
    let user_id = grok_auth.as_ref().map(|a| a.user_id.clone());
    let team_id = grok_auth.as_ref().and_then(|a| a.team_id.clone());
    let subscription_tier = super::mvp_agent::resolve_subscription_tier_for_telemetry(
        config
            .remote_settings
            .as_ref()
            .and_then(|rs| rs.subscription_tier_display.clone()),
        auth_manager.current_or_expired().as_ref(),
    );
    xai_grok_telemetry::client::init(
        config.telemetry.clone(),
        config.resolve_telemetry_mode().value,
        user_id,
        team_id,
        config.endpoints.deployment_key.clone(),
        crate::http::origin_client_info_from_env(),
        xai_grok_version::VERSION.to_owned(),
        subscription_tier,
    );
}
/// Sync this principal's config now rather than waiting for the background tick.
/// Stay quiet about absence or failure during login; confirm only when config was actually applied.
/// Driven by the login callers here so auth does not reach into managed config.
pub async fn apply_post_login_config(
    _authenticated: xai_grok_login::GrokAuth,
) -> anyhow::Result<()> {
    Ok(())
}
/// `grok logout` CLI subcommand: clear the cached session.
pub fn run_cli_logout(grok_com_config: &xai_grok_login::GrokComConfig) -> anyhow::Result<()> {
    let grok_home = xai_grok_shell_base::util::grok_home::grok_home();
    let auth_manager = xai_grok_login::AuthManager::new_with_proxy_base_url(
        &grok_home,
        grok_com_config.clone(),
        crate::agent::config::EndpointsConfig::from_effective_config().proxy_url(),
    );
    let result = xai_grok_login::perform_logout(&auth_manager, None, || {})
        .map_err(|e| anyhow::anyhow!("Failed to clear auth: {e}"))?;
    if !result.was_logged_in {
        eprintln!("No cached session to log out of.");
        if result.api_key_still_set {
            eprintln!("You are authenticated via XAI_API_KEY (environment variable).");
        }
        return Ok(());
    }
    if let Some(email) = result.email {
        eprintln!("Logged out (was signed in as {email})");
    } else {
        eprintln!("Logged out");
    }
    if result.api_key_still_set {
        eprintln!("XAI_API_KEY is still set and will be used for authentication.");
    }
    Ok(())
}
#[cfg(test)]
#[path = "init_tests.rs"]
mod tests;
