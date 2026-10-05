//! Per-model token pricing, in USD per 1,000,000 tokens.
//!
//! A provider that reports its own cost (the ledger's `cost_usd_ticks`) wins; this table only fills the gap for
//! providers like DeepSeek whose usage payload carries tokens but no price. It lives under
//! `[ui.status_line.pricing.<model>]` and every rate falls back to the clearly-labelled DeepSeek v4 Pro off-peak
//! default below, so a bare `items = ["cost"]` still shows a number and an override only has to state the rates it
//! changes.
//!
//! The default is a stand-in, not a contract: prices are provider policy and change without a release, so treat a
//! reported cost as authoritative and this table as an estimate.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::context::StatusLineSessionUsage;

/// The model the built-in defaults are labelled for.
pub const DEFAULT_PRICING_MODEL: &str = "deepseek-v4-pro";

/// USD per 1,000,000 tokens for one model.
///
/// A missing key in a `[ui.status_line.pricing.<model>]` table inherits the DeepSeek v4 Pro default rather than
/// zero, so a partial override cannot silently make one direction of the bill free.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ModelPricing {
    /// Cache-miss (fresh) prompt tokens.
    pub input: f64,
    /// Completion tokens, reasoning included.
    pub output: f64,
    /// Prompt tokens served from the prefix cache.
    pub cache_hit: f64,
    /// Alias for `input` kept separate so the four provider rates read as written.
    pub cache_miss: f64,
}

/// DeepSeek v4 Pro off-peak list price, per the provider's pricing page at the time of writing.
///
/// The field names are pinned to the four rates the provider publishes; override any of them with
/// `[ui.status_line.pricing.deepseek-v4-pro]`.
impl Default for ModelPricing {
    fn default() -> Self {
        Self {
            input: 0.66,
            output: 1.98,
            cache_hit: 0.022,
            cache_miss: 0.66,
        }
    }
}

/// A resolved lookup: the DeepSeek defaults, with any user overrides layered on top.
#[derive(Debug, Clone, PartialEq)]
pub struct PricingTable {
    models: BTreeMap<String, ModelPricing>,
}

impl PricingTable {
    /// The shipped default table.
    pub fn deepseek_defaults() -> Self {
        let mut models = BTreeMap::new();
        models.insert(DEFAULT_PRICING_MODEL.to_string(), ModelPricing::default());
        Self { models }
    }

    /// Layer `overrides` on top of the defaults. A model named in `overrides` replaces the default entry outright.
    pub fn with_overrides(overrides: BTreeMap<String, ModelPricing>) -> Self {
        let mut table = Self::deepseek_defaults();
        table.models.extend(overrides);
        table
    }

    /// Rate card for `model_id`.
    ///
    /// An exact match wins. A model that is not named falls back to the DeepSeek v4 Pro default: the shipped
    /// catalog is DeepSeek-only, and showing an estimate under a clearly-labelled default beats showing nothing.
    /// A non-DeepSeek model should be named explicitly to avoid inheriting rates that are not its own.
    pub fn for_model(&self, model_id: &str) -> Option<ModelPricing> {
        self.models
            .get(model_id)
            .copied()
            .or_else(|| self.models.get(DEFAULT_PRICING_MODEL).copied())
    }

    /// Estimated USD for one session's token buckets, or `None` when nothing was recorded.
    ///
    /// `StatusLineSessionUsage.input_tokens` is already the fresh (cache-miss) subset, so the buckets sum without
    /// overlap: fresh input + cache creation at the miss rate, cache hits at the hit rate, completion at output.
    pub fn cost_usd(&self, model_id: &str, usage: &StatusLineSessionUsage) -> Option<f64> {
        let pricing = self.for_model(model_id)?;
        if usage.input_tokens == 0
            && usage.output_tokens == 0
            && usage.cache_creation_input_tokens == 0
            && usage.cache_read_input_tokens == 0
        {
            return None;
        }
        let per_million = |tokens: u64, rate: f64| tokens as f64 * rate / 1_000_000.0;
        Some(
            per_million(usage.input_tokens, pricing.cache_miss)
                + per_million(usage.cache_creation_input_tokens, pricing.cache_miss)
                + per_million(usage.cache_read_input_tokens, pricing.cache_hit)
                + per_million(usage.output_tokens, pricing.output),
        )
    }
}

impl Default for PricingTable {
    fn default() -> Self {
        Self::deepseek_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn usage(input: u64, output: u64, hit: u64, miss: u64) -> StatusLineSessionUsage {
        StatusLineSessionUsage {
            input_tokens: input,
            output_tokens: output,
            cache_creation_input_tokens: miss,
            cache_read_input_tokens: hit,
            reasoning_tokens: 0,
        }
    }

    #[test]
    fn default_table_prices_the_labelled_model() {
        let table = PricingTable::default();
        let pricing = table
            .for_model(DEFAULT_PRICING_MODEL)
            .expect("default model");
        assert_eq!(pricing.input, 0.66);
        assert_eq!(pricing.output, 1.98);
        assert_eq!(pricing.cache_hit, 0.022);
        assert_eq!(pricing.cache_miss, 0.66);
    }

    #[test]
    fn unknown_model_falls_back_to_the_labelled_default() {
        let table = PricingTable::default();
        assert_eq!(
            table.for_model("some-other-model"),
            Some(ModelPricing::default())
        );
    }

    #[test]
    fn cost_splits_hits_misses_and_output() {
        let table = PricingTable::default();
        // 1M fresh input @ 0.66 + 1M output @ 1.98 + 1M cached @ 0.022 + 1M creation @ 0.66 = 3.322
        let cost = table
            .cost_usd(
                DEFAULT_PRICING_MODEL,
                &usage(1_000_000, 1_000_000, 1_000_000, 1_000_000),
            )
            .expect("cost");
        assert!((cost - 3.322).abs() < 1e-9, "{cost}");
    }

    #[test]
    fn empty_usage_has_no_cost() {
        let table = PricingTable::default();
        assert_eq!(
            table.cost_usd(DEFAULT_PRICING_MODEL, &usage(0, 0, 0, 0)),
            None
        );
    }

    #[test]
    fn overrides_replace_the_default_entry() {
        let mut overrides = BTreeMap::new();
        overrides.insert(
            "my-model".to_string(),
            ModelPricing {
                input: 1.0,
                output: 2.0,
                cache_hit: 0.1,
                cache_miss: 1.0,
            },
        );
        let table = PricingTable::with_overrides(overrides);
        assert_eq!(table.for_model("my-model").unwrap().output, 2.0);
        assert_eq!(table.for_model(DEFAULT_PRICING_MODEL).unwrap().output, 1.98);
    }
}
