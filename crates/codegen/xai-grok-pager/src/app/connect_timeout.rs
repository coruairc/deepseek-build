//! Startup connect budget: the default agent-ready wait and its env override.

use std::time::Duration;

macro_rules! connect_ui_timeout_env {
    () => {
        "GROK_CONNECT_UI_TIMEOUT_SECS"
    };
}

pub(super) const CONNECT_UI_TIMEOUT_ENV: &str = connect_ui_timeout_env!();
pub(super) const CONNECT_UI_TIMEOUT_TRY_COMMAND: &str =
    concat!(connect_ui_timeout_env!(), "=60 grok");
pub(super) const DEFAULT_CONNECT_UI_TIMEOUT: Duration = Duration::from_secs(30);
const MIN_CONNECT_UI_TIMEOUT_SECS: u64 = 6;
const PERSONAL_CONNECT_UI_SLACK: Duration = Duration::from_secs(2);
// Floors the personal budget above the assert's connect-future sum
// (5.5 settings window + 5 eager-auth + 2 slack = 12.5s). No managed preamble.
const PERSONAL_CONNECT_UI_FLOOR: Duration = Duration::from_millis(12_500);
const _: () = assert!(
    PERSONAL_CONNECT_UI_FLOOR.as_millis()
        >= xai_grok_shell::http::STARTUP_SETTINGS_WAIT_DEADLINE.as_millis()
            + xai_grok_shell::http::STARTUP_AUTH_REFRESH_TIMEOUT.as_millis()
            + PERSONAL_CONNECT_UI_SLACK.as_millis(),
    "the personal connect floor must cover the full personal connect future: settings window, \
     post-gate eager-auth refresh, and slack"
);

pub(super) fn resolve(env: Option<&str>) -> Duration {
    let base = match env.map(str::trim).and_then(|v| v.parse::<u64>().ok()) {
        None | Some(0) => DEFAULT_CONNECT_UI_TIMEOUT,
        Some(secs) => Duration::from_secs(secs.max(MIN_CONNECT_UI_TIMEOUT_SECS)),
    };
    base.max(PERSONAL_CONNECT_UI_FLOOR)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_cases() {
        assert_eq!(resolve(None), DEFAULT_CONNECT_UI_TIMEOUT);
        assert_eq!(resolve(Some("")), DEFAULT_CONNECT_UI_TIMEOUT);
        assert_eq!(resolve(Some(" 45 ")), Duration::from_secs(45));
        assert_eq!(resolve(Some("0")), DEFAULT_CONNECT_UI_TIMEOUT);
        assert_eq!(resolve(Some("garbage")), DEFAULT_CONNECT_UI_TIMEOUT);
        assert_eq!(resolve(Some("-5")), DEFAULT_CONNECT_UI_TIMEOUT);
        assert_eq!(resolve(Some("1e3")), DEFAULT_CONNECT_UI_TIMEOUT);
        assert_eq!(resolve(Some("1")), PERSONAL_CONNECT_UI_FLOOR);
        assert_eq!(resolve(Some("9999")), Duration::from_secs(9999));
    }
}
