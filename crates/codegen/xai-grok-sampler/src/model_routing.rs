//! Config-resolved automatic model routing for DeepSeek Build.
//!
//! DeepSeek exposes a small/fast model (`deepseek-flash`) and a stronger reasoning
//! model (`deepseek-v4-pro`). This module turns a user-selected policy into a
//! concrete [`ModelRoute`] per turn:
//!
//! - [`ModelRoutingPolicy::Off`] never routes (the caller keeps its configured model);
//! - [`ModelRoutingPolicy::Low`] always routes to the fast model with thinking off;
//! - [`ModelRoutingPolicy::High`] always routes to the pro model at high effort;
//! - [`ModelRoutingPolicy::Auto`] classifies the turn and picks fast for trivial
//!   turns, pro/high for complex ones.
//!
//! A `manual_override` on [`ModelRoutingConfig`] wins over the policy, which is how
//! the CLI's explicit model/effort selection is honored. The decision is pure and
//! side-effect free so callers can log or display it.

use serde::{Deserialize, Serialize};

use xai_grok_sampling_types::{ConversationItem, ConversationRequest, ReasoningEffort};

/// User-selected routing policy. `Off` preserves the caller's configured model.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelRoutingPolicy {
    /// Never route; use the caller's configured model and effort.
    #[default]
    Off,
    /// Always use the fast model with thinking disabled.
    Low,
    /// Always use the pro model at the configured pro effort.
    High,
    /// Classify the turn and route to fast or pro automatically.
    Auto,
}

/// The outcome of [`classify_turn`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TurnComplexity {
    /// A short, self-contained turn with no working context: route fast.
    Trivial,
    /// A long, code-bearing, tool-continuing, or reasoning-heavy turn: route pro.
    Complex,
}

/// A concrete model/effort decision produced by routing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelRoute {
    /// Catalog id to send as the request `model`.
    pub model: String,
    /// Reasoning effort to request; `Some(None)` explicitly disables thinking.
    pub reasoning_effort: Option<ReasoningEffort>,
    /// Whether thinking is enabled for this route.
    pub thinking: bool,
}

/// Resolved routing configuration. Defaults match the DeepSeek catalog.
#[derive(Clone, Debug)]
pub struct ModelRoutingConfig {
    pub policy: ModelRoutingPolicy,
    /// Fast, no-thinking model.
    pub flash_model: String,
    /// Strong reasoning model.
    pub pro_model: String,
    /// Effort applied on the pro route.
    pub pro_effort: ReasoningEffort,
    /// Effort applied on the fast route; `Some(None)` disables thinking.
    pub flash_effort: Option<ReasoningEffort>,
    /// Explicit user selection that overrides the policy when set.
    pub manual_override: Option<ModelRoute>,
}

impl Default for ModelRoutingConfig {
    fn default() -> Self {
        Self {
            policy: ModelRoutingPolicy::Off,
            flash_model: "deepseek-flash".to_string(),
            pro_model: "deepseek-v4-pro".to_string(),
            pro_effort: ReasoningEffort::High,
            // `Some(None)` maps to the DeepSeek "thinking disabled" body.
            flash_effort: Some(ReasoningEffort::None),
            manual_override: None,
        }
    }
}

impl ModelRoutingConfig {
    /// Build the fast route.
    pub fn flash_route(&self) -> ModelRoute {
        ModelRoute {
            model: self.flash_model.clone(),
            reasoning_effort: self.flash_effort,
            thinking: self
                .flash_effort
                .is_some_and(|e| e != ReasoningEffort::None),
        }
    }

    /// Build the pro route.
    pub fn pro_route(&self) -> ModelRoute {
        ModelRoute {
            model: self.pro_model.clone(),
            reasoning_effort: Some(self.pro_effort),
            thinking: self.pro_effort != ReasoningEffort::None,
        }
    }
}

/// Marker substrings that make a turn complex enough to warrant the pro model.
/// Matching is done on the lowercased last user message.
const COMPLEX_MARKERS: &[&str] = &[
    "debug",
    "refactor",
    "architect",
    "implement",
    "analyze",
    "analyse",
    "explain",
    "why ",
    "design",
    "migrat",
    "optimi",
    "stack trace",
    "investigate",
    "review",
    "root cause",
    "step by step",
];

/// A user turn at or above this many characters is treated as complex.
const COMPLEX_LEN_THRESHOLD: usize = 400;

/// Text of the most recent user message, joined across its text parts.
fn last_user_text(request: &ConversationRequest) -> String {
    request
        .items
        .iter()
        .rev()
        .find_map(|item| match item {
            ConversationItem::User(user) => Some(
                user.content
                    .iter()
                    .filter_map(|part| match part {
                        xai_grok_sampling_types::ContentPart::Text { text } => Some(text.as_ref()),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .join("\n"),
            ),
            _ => None,
        })
        .unwrap_or_default()
}

/// Classify a turn as trivial or complex using cheap, local heuristics.
///
/// Complex when the conversation already contains tool activity (a working turn),
/// when the last user message is long or contains code, or when it carries a known
/// reasoning-heavy marker. Everything else is trivial.
pub fn classify_turn(request: &ConversationRequest) -> TurnComplexity {
    let has_tool_activity = request.items.iter().any(|item| match item {
        ConversationItem::Assistant(a) => !a.tool_calls.is_empty(),
        ConversationItem::ToolResult(_) => true,
        _ => false,
    });
    if has_tool_activity {
        return TurnComplexity::Complex;
    }

    let text = last_user_text(request);
    let lower = text.to_lowercase();
    let long = text.chars().count() >= COMPLEX_LEN_THRESHOLD;
    let has_code = text.contains("```") || text.contains("fn ") || lower.contains("panic");
    let has_marker = COMPLEX_MARKERS.iter().any(|marker| lower.contains(marker));

    if long || has_code || has_marker {
        TurnComplexity::Complex
    } else {
        TurnComplexity::Trivial
    }
}

/// Resolve the route for a turn, honoring the manual override first.
///
/// Returns `None` when the policy is [`ModelRoutingPolicy::Off`], meaning the
/// caller's configured model stands.
pub fn resolve_route(
    config: &ModelRoutingConfig,
    request: &ConversationRequest,
) -> Option<ModelRoute> {
    if let Some(manual) = &config.manual_override {
        return Some(manual.clone());
    }
    match config.policy {
        ModelRoutingPolicy::Off => None,
        ModelRoutingPolicy::Low => Some(config.flash_route()),
        ModelRoutingPolicy::High => Some(config.pro_route()),
        ModelRoutingPolicy::Auto => Some(match classify_turn(request) {
            TurnComplexity::Trivial => config.flash_route(),
            TurnComplexity::Complex => config.pro_route(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use xai_grok_sampling_types::{ContentPart, ConversationItem};

    fn user(text: &str) -> ConversationItem {
        ConversationItem::user(text)
    }

    fn request(items: Vec<ConversationItem>) -> ConversationRequest {
        ConversationRequest::from_items(items)
    }

    #[test]
    fn short_greeting_is_trivial() {
        assert_eq!(
            classify_turn(&request(vec![user("hi there")])),
            TurnComplexity::Trivial
        );
    }

    #[test]
    fn long_prompt_is_complex() {
        let long = "word ".repeat(100);
        assert_eq!(
            classify_turn(&request(vec![user(&long)])),
            TurnComplexity::Complex
        );
    }

    #[test]
    fn code_fence_is_complex() {
        assert_eq!(
            classify_turn(&request(vec![user("look at ```rust\nfn main() {}\n```")])),
            TurnComplexity::Complex
        );
    }

    #[test]
    fn reasoning_marker_is_complex() {
        assert_eq!(
            classify_turn(&request(vec![user("why does this fail?")])),
            TurnComplexity::Complex
        );
        assert_eq!(
            classify_turn(&request(vec![user("please refactor this module")])),
            TurnComplexity::Complex
        );
    }

    #[test]
    fn tool_activity_forces_complex() {
        let items = vec![
            ConversationItem::assistant_tool_calls(vec![xai_grok_sampling_types::ToolCall {
                id: "c1".into(),
                name: "read_file".into(),
                arguments: "{}".into(),
            }]),
            ConversationItem::tool_result("c1", "contents"),
            user("ok"),
        ];
        assert_eq!(classify_turn(&request(items)), TurnComplexity::Complex);
    }

    #[test]
    fn policy_off_returns_no_route() {
        let config = ModelRoutingConfig::default();
        assert_eq!(resolve_route(&config, &request(vec![user("hi")])), None);
    }

    #[test]
    fn policy_low_routes_flash_without_thinking() {
        let config = ModelRoutingConfig {
            policy: ModelRoutingPolicy::Low,
            ..Default::default()
        };
        let route = resolve_route(&config, &request(vec![user("hi")])).unwrap();
        assert_eq!(route.model, "deepseek-flash");
        assert_eq!(route.reasoning_effort, Some(ReasoningEffort::None));
        assert!(!route.thinking);
    }

    #[test]
    fn policy_high_routes_pro_at_high_effort() {
        let config = ModelRoutingConfig {
            policy: ModelRoutingPolicy::High,
            ..Default::default()
        };
        let route = resolve_route(&config, &request(vec![user("hi")])).unwrap();
        assert_eq!(route.model, "deepseek-v4-pro");
        assert_eq!(route.reasoning_effort, Some(ReasoningEffort::High));
        assert!(route.thinking);
    }

    #[test]
    fn policy_auto_splits_trivial_and_complex() {
        let config = ModelRoutingConfig {
            policy: ModelRoutingPolicy::Auto,
            ..Default::default()
        };
        let trivial = resolve_route(&config, &request(vec![user("hi")])).unwrap();
        assert_eq!(trivial.model, "deepseek-flash");
        assert!(!trivial.thinking);

        let complex =
            resolve_route(&config, &request(vec![user("debug this stack trace")])).unwrap();
        assert_eq!(complex.model, "deepseek-v4-pro");
        assert_eq!(complex.reasoning_effort, Some(ReasoningEffort::High));
        assert!(complex.thinking);
    }

    #[test]
    fn manual_override_wins_over_policy() {
        let manual = ModelRoute {
            model: "deepseek-v4-flash".into(),
            reasoning_effort: Some(ReasoningEffort::Low),
            thinking: true,
        };
        let config = ModelRoutingConfig {
            policy: ModelRoutingPolicy::Low,
            manual_override: Some(manual.clone()),
            ..Default::default()
        };
        assert_eq!(
            resolve_route(&config, &request(vec![user("hi")])).unwrap(),
            manual
        );
    }

    #[test]
    fn policy_serde_round_trips_snake_case() {
        for (policy, wire) in [
            (ModelRoutingPolicy::Off, "\"off\""),
            (ModelRoutingPolicy::Low, "\"low\""),
            (ModelRoutingPolicy::High, "\"high\""),
            (ModelRoutingPolicy::Auto, "\"auto\""),
        ] {
            assert_eq!(serde_json::to_string(&policy).unwrap(), wire);
            let back: ModelRoutingPolicy = serde_json::from_str(wire).unwrap();
            assert_eq!(back, policy);
        }
    }

    #[test]
    fn image_only_user_turn_is_trivial() {
        let items = vec![ConversationItem::User(xai_grok_sampling_types::UserItem {
            content: vec![ContentPart::Image {
                url: "data:image/png;base64,AAAA".into(),
            }],
            ..Default::default()
        })];
        assert_eq!(classify_turn(&request(items)), TurnComplexity::Trivial);
    }
}
