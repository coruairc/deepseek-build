//! Per-model token pricing, in USD per 1,000,000 tokens.
//!
//! A provider that reports its own cost (the ledger's `cost_usd_ticks`) wins; this table only fills the gap for
//! providers like DeepSeek whose usage payload carries tokens but no price. It lives under
//! `[ui.status_line.pricing.<model>]` and every rate falls back to the clearly-labelled DeepSeek v4 Pro default
//! below, so a bare `items = ["cost"]` still shows a number and an override only has to state the rates it
//! changes.
//!
//! The default rates are DeepSeek's **peak** list prices (official pricing page, fetched 2026-10-07). Off-peak is
//! exactly half: peak hours are 01:00-04:00 and 06:00-10:00 UTC Mon-Fri excluding Chinese public holidays, and
//! every other hour is off-peak. This table has no clock, so it always prices at peak and therefore never
//! under-estimates; halve the three rates in an override to price off-peak.
//!
//! The default is a stand-in, not a contract: prices are provider policy and change without a release, so treat a
//! reported cost as authoritative and this table as an estimate.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::context::StatusLineSessionUsage;

/// The model the built-in defaults are labelled for.
pub const DEFAULT_PRICING_MODEL: &str = "deepseek-v4-pro";

/// The second DeepSeek model the shipped table names explicitly.
pub const FLASH_PRICING_MODEL: &str = "deepseek-flash";

/// USD per 1,000,000 tokens for one model.
///
/// A missing key in a `[ui.status_line.pricing.<model>]` table inherits the DeepSeek v4 Pro default rather than
/// zero, so a partial override cannot silently make one direction of the bill free.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ModelPricing {
    /// Prompt tokens served from the prefix cache.
    pub cache_hit: f64,
    /// Cache-miss (fresh) prompt tokens.
    pub cache_miss: f64,
    /// Completion tokens, reasoning included.
    pub output: f64,
}

/// DeepSeek v4 Pro **peak** list price, per the provider's pricing page (fetched 2026-10-07): cache-hit input
/// $0.044, cache-miss input $1.32, output $3.96 per 1M tokens. Off-peak is half these rates.
///
/// The field names are the three rates the provider publishes; override any of them with
/// `[ui.status_line.pricing.deepseek-v4-pro]`.
impl Default for ModelPricing {
    fn default() -> Self {
        Self {
            cache_hit: 0.044,
            cache_miss: 1.32,
            output: 3.96,
        }
    }
}

impl ModelPricing {
    /// `deepseek-flash` peak list price: cache-hit input $0.006, cache-miss input $0.30, output $1.20 per 1M.
    pub const fn flash() -> Self {
        Self {
            cache_hit: 0.006,
            cache_miss: 0.30,
            output: 1.20,
        }
    }
}

/// A resolved lookup: the DeepSeek defaults, with any user overrides layered on top.
#[derive(Debug, Clone, PartialEq)]
pub struct PricingTable {
    models: BTreeMap<String, ModelPricing>,
}

impl PricingTable {
    /// The shipped default table: DeepSeek v4 Pro and Flash at peak rates.
    pub fn deepseek_defaults() -> Self {
        let mut models = BTreeMap::new();
        models.insert(DEFAULT_PRICING_MODEL.to_string(), ModelPricing::default());
        models.insert(FLASH_PRICING_MODEL.to_string(), ModelPricing::flash());
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
    /// Reasoning tokens are inside `output_tokens` and are not charged again.
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
    fn default_table_prices_both_labelled_models_at_peak() {
        let table = PricingTable::default();
        let pro = table.for_model(DEFAULT_PRICING_MODEL).expect("pro");
        assert_eq!(pro.cache_hit, 0.044);
        assert_eq!(pro.cache_miss, 1.32);
        assert_eq!(pro.output, 3.96);

        let flash = table.for_model(FLASH_PRICING_MODEL).expect("flash");
        assert_eq!(flash.cache_hit, 0.006);
        assert_eq!(flash.cache_miss, 0.30);
        assert_eq!(flash.output, 1.20);
    }

    #[test]
    fn unknown_model_falls_back_to_the_labelled_default() {
        let table = PricingTable::default();
        assert_eq!(
            table.for_model("some-other-model"),
            Some(ModelPricing::default())
        );
    }

    /// The worked example from the task: pro, 1000 cache-hit + 2000 cache-miss input + 500 output.
    /// 1000*0.044 + 2000*1.32 + 500*3.96 = 44 + 2640 + 1980 micro-USD = 0.004664.
    #[test]
    fn pro_worked_example_is_exact() {
        let table = PricingTable::default();
        let cost = table
            .cost_usd(DEFAULT_PRICING_MODEL, &usage(0, 500, 1_000, 2_000))
            .expect("cost");
        assert!((cost - 0.004664).abs() < 1e-12, "{cost}");
    }

    /// Flash, the same buckets: 1000*0.006 + 2000*0.30 + 500*1.20 = 6 + 600 + 600 = 0.001206.
    #[test]
    fn flash_worked_example_is_exact() {
        let table = PricingTable::default();
        let cost = table
            .cost_usd(FLASH_PRICING_MODEL, &usage(0, 500, 1_000, 2_000))
            .expect("cost");
        assert!((cost - 0.001206).abs() < 1e-12, "{cost}");
    }

    #[test]
    fn cost_splits_hits_misses_and_output() {
        let table = PricingTable::default();
        // 1M fresh input @ 1.32 + 1M output @ 3.96 + 1M cached @ 0.044 + 1M creation @ 1.32 = 6.644
        let cost = table
            .cost_usd(
                DEFAULT_PRICING_MODEL,
                &usage(1_000_000, 1_000_000, 1_000_000, 1_000_000),
            )
            .expect("cost");
        assert!((cost - 6.644).abs() < 1e-9, "{cost}");
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
                cache_hit: 0.1,
                cache_miss: 1.0,
                output: 2.0,
            },
        );
        let table = PricingTable::with_overrides(overrides);
        assert_eq!(table.for_model("my-model").unwrap().output, 2.0);
        assert_eq!(table.for_model(DEFAULT_PRICING_MODEL).unwrap().output, 3.96);
    }
}
