//! Local, network-free replacement for the deleted `xai-grok-voice` crate.
//!
//! The upstream voice crate spoke to `wss://api.deepseek.com/v1/stt` and has been
//! removed. This module keeps the same public surface the pager already used so
//! the build stays green, but it opens no sockets: [`run_voice_pipeline`] only
//! drains commands, [`input_device_info`] always reports that capture is
//! unavailable, and [`maybe_run_capture_subprocess`] is a no-op.
//!
//! The STT language catalog is retained as pure data for the settings UI.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::mpsc;

/// Which STT route a capture session took; the pager's submit path differs between them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoiceRoute {
    Streaming,
    Clip,
}

/// Minted per press by the pager.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct VoiceSessionId(pub(crate) u64);

impl VoiceSessionId {
    #[must_use]
    pub fn next(self) -> Self {
        VoiceSessionId(self.0.wrapping_add(1))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaggedVoiceEvent {
    pub session: VoiceSessionId,
    pub event: VoiceEvent,
}

/// Events emitted by [`run_voice_pipeline`] to the pager event loop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VoiceEvent {
    CaptureStarted {
        route: VoiceRoute,
    },
    CaptureCancelled,
    InterimTranscript {
        text: String,
    },
    UtteranceFinal {
        text: String,
    },
    Notice {
        message: String,
    },
    Transcribing,
    Error {
        message: String,
        hint: Option<String>,
    },
}

impl From<&VoiceEvent> for &'static str {
    fn from(event: &VoiceEvent) -> Self {
        match event {
            VoiceEvent::CaptureStarted { .. } => "CaptureStarted",
            VoiceEvent::CaptureCancelled => "CaptureCancelled",
            VoiceEvent::InterimTranscript { .. } => "InterimTranscript",
            VoiceEvent::UtteranceFinal { .. } => "UtteranceFinal",
            VoiceEvent::Notice { .. } => "Notice",
            VoiceEvent::Transcribing => "Transcribing",
            VoiceEvent::Error { .. } => "Error",
        }
    }
}

/// Commands from the pager event loop.
#[derive(Debug, PartialEq, Eq)]
pub enum VoiceCommand {
    PttPress { session: VoiceSessionId },
    PttRelease,
    Abort,
    Shutdown,
}

/// One supported STT language from the public API catalog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SttLanguage {
    pub code: &'static str,
    pub name: &'static str,
}

/// Client-only sentinel meaning "resolve from the process locale at connect time".
pub const STT_LANGUAGE_AUTO: &str = "auto";

/// Default STT language when unset or unrecognized.
pub const STT_LANGUAGE_DEFAULT: &str = "en";

/// Official deepseek-build STT languages, sorted by English name.
pub const STT_LANGUAGES: &[SttLanguage] = &[
    SttLanguage {
        code: "ar",
        name: "Arabic",
    },
    SttLanguage {
        code: "cs",
        name: "Czech",
    },
    SttLanguage {
        code: "da",
        name: "Danish",
    },
    SttLanguage {
        code: "nl",
        name: "Dutch",
    },
    SttLanguage {
        code: "en",
        name: "English",
    },
    SttLanguage {
        code: "fil",
        name: "Filipino",
    },
    SttLanguage {
        code: "fr",
        name: "French",
    },
    SttLanguage {
        code: "de",
        name: "German",
    },
    SttLanguage {
        code: "hi",
        name: "Hindi",
    },
    SttLanguage {
        code: "id",
        name: "Indonesian",
    },
    SttLanguage {
        code: "it",
        name: "Italian",
    },
    SttLanguage {
        code: "ja",
        name: "Japanese",
    },
    SttLanguage {
        code: "ko",
        name: "Korean",
    },
    SttLanguage {
        code: "mk",
        name: "Macedonian",
    },
    SttLanguage {
        code: "ms",
        name: "Malay",
    },
    SttLanguage {
        code: "fa",
        name: "Persian",
    },
    SttLanguage {
        code: "pl",
        name: "Polish",
    },
    SttLanguage {
        code: "pt",
        name: "Portuguese",
    },
    SttLanguage {
        code: "ro",
        name: "Romanian",
    },
    SttLanguage {
        code: "ru",
        name: "Russian",
    },
    SttLanguage {
        code: "es",
        name: "Spanish",
    },
    SttLanguage {
        code: "sv",
        name: "Swedish",
    },
    SttLanguage {
        code: "th",
        name: "Thai",
    },
    SttLanguage {
        code: "tr",
        name: "Turkish",
    },
    SttLanguage {
        code: "vi",
        name: "Vietnamese",
    },
];

/// Look up a catalog entry by exact (case-sensitive) code.
pub fn stt_language_by_code(code: &str) -> Option<&'static SttLanguage> {
    STT_LANGUAGES.iter().find(|l| l.code == code)
}

/// Map a user/config string to a catalog code or [`STT_LANGUAGE_AUTO`].
pub fn canonicalize_stt_language(value: Option<&str>) -> &'static str {
    let raw = value.unwrap_or_default().trim();
    if raw.is_empty() {
        return STT_LANGUAGE_DEFAULT;
    }
    if raw.eq_ignore_ascii_case(STT_LANGUAGE_AUTO) {
        return STT_LANGUAGE_AUTO;
    }
    if let Some(code) = match_supported_code(raw) {
        return code;
    }
    let primary = raw.split(['_', '-', '.']).next().unwrap_or("").trim();
    if let Some(code) = match_supported_code(primary) {
        return code;
    }
    if primary.eq_ignore_ascii_case("tl") {
        return "fil";
    }
    STT_LANGUAGE_DEFAULT
}

/// Concrete language code to send on the STT wire; never returns `auto`.
pub fn language_for_api(stored: &str) -> &'static str {
    let canonical = canonicalize_stt_language(Some(stored));
    if canonical == STT_LANGUAGE_AUTO {
        system_stt_language().unwrap_or(STT_LANGUAGE_DEFAULT)
    } else {
        canonical
    }
}

fn system_stt_language() -> Option<&'static str> {
    let loc = ["LC_ALL", "LC_MESSAGES", "LANG"]
        .into_iter()
        .find_map(|var| std::env::var(var).ok().filter(|v| !v.is_empty()))?;
    if loc.eq_ignore_ascii_case("C") || loc.eq_ignore_ascii_case("POSIX") {
        return None;
    }
    match_supported_code(loc.split(['_', '-', '.']).next().unwrap_or("").trim())
}

fn match_supported_code(raw: &str) -> Option<&'static str> {
    STT_LANGUAGES
        .iter()
        .map(|l| l.code)
        .find(|&code| raw.eq_ignore_ascii_case(code))
}

/// Optional `[voice]` overrides from config. No endpoint is used in this build.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct VoiceConfig {
    pub language: String,
    /// The pager stamps this request identity; not user-settable.
    pub client_identifier: String,
    pub user_agent: String,
}

impl VoiceConfig {
    pub fn from_config_table(root: &toml::Table, _resolved_endpoints_base: Option<&str>) -> Self {
        let voice = root.get("voice").and_then(|v| v.as_table());
        Self {
            language: voice
                .and_then(|t| t.get("language"))
                .and_then(|v| v.as_str())
                .unwrap_or(STT_LANGUAGE_DEFAULT)
                .to_owned(),
            client_identifier: String::new(),
            user_agent: String::new(),
        }
    }
}

/// Why there is no bearer for voice.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum VoiceAuthError {
    #[error("voice is not available in this build")]
    ForeignSession,
    #[error("not signed in")]
    NotSignedIn,
}

pub trait VoiceAuthProvider: std::fmt::Debug + Send + Sync + 'static {
    fn bearer(&self) -> Pin<Box<dyn Future<Output = Result<String, VoiceAuthError>> + Send + '_>>;
}

/// Shared provider handed to the voice pipeline.
pub type SharedVoiceAuth = Arc<dyn VoiceAuthProvider>;

/// The STT backends a pipeline may use. Kept for signature compatibility.
#[derive(Debug, Clone)]
pub struct SttRoutes {
    pub auth: SharedVoiceAuth,
    pub clip_transcriber: Option<SharedClipTranscriber>,
}

pub trait ClipTranscriber: std::fmt::Debug + Send + Sync + 'static {}
pub type SharedClipTranscriber = Arc<dyn ClipTranscriber>;

/// Timeout for an outstanding clip.
pub const FINAL_TIMEOUT: Duration = Duration::from_secs(240);

#[derive(Debug, thiserror::Error)]
pub enum VoiceError {
    #[error("configuration: {0}")]
    Config(String),
}

/// The input device capture would use; always unavailable in this build.
#[derive(Debug, Clone)]
pub struct InputDeviceInfo {
    pub name: String,
    pub detail: String,
}

pub fn input_device_info() -> Result<InputDeviceInfo, VoiceError> {
    Err(VoiceError::Config(
        "voice audio capture is not available in this build".into(),
    ))
}

/// This build never advertises microphone capture.
pub const AUDIO_SUPPORTED: bool = false;

/// Network-free stand-in for the deleted pipeline. It drains commands until
/// shutdown and never emits events.
pub async fn run_voice_pipeline(
    _config: VoiceConfig,
    _routes: SttRoutes,
    mut cmd_rx: mpsc::Receiver<VoiceCommand>,
    _event_tx: mpsc::Sender<TaggedVoiceEvent>,
) {
    while let Some(cmd) = cmd_rx.recv().await {
        if matches!(cmd, VoiceCommand::Shutdown) {
            break;
        }
    }
}

/// If this process was re-exec'd as the hidden mic-capture helper, run it.
/// This build has no capture, so it is always a normal invocation.
pub fn maybe_run_capture_subprocess() -> Option<i32> {
    None
}
