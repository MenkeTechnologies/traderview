//! SaaS quick ratio — growth efficiency: how fast new and expansion MRR are added
//! relative to the contraction and churn lost in the same period. `quick_ratio =
//! (new MRR + expansion MRR) ÷ (churned MRR + contraction MRR)`. A ratio of 4+ is
//! excellent (every dollar lost is replaced four times over), 2–4 is healthy, 1–2
//! is stalling, and below 1 the base is shrinking. This is the SaaS-growth ratio
//! popularized by Social Capital — distinct from the accounting quick (acid-test)
//! ratio in the liquidity-ratios module, which measures balance-sheet liquidity.
//! Pure compute, not financial advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct QuickRatioInput {
    /// New-logo MRR added this period.
    #[serde(default)]
    pub new_mrr_usd: f64,
    /// Expansion MRR from the existing base (upsell, seats).
    #[serde(default)]
    pub expansion_mrr_usd: f64,
    /// Churned MRR (full cancellations).
    #[serde(default)]
    pub churned_mrr_usd: f64,
    /// Contraction MRR (downgrades).
    #[serde(default)]
    pub contraction_mrr_usd: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct QuickRatioReport {
    /// New + expansion MRR.
    pub gained_mrr_usd: f64,
    /// Churned + contraction MRR.
    pub lost_mrr_usd: f64,
    /// Net new MRR (gained − lost).
    pub net_new_mrr_usd: f64,
    /// (new + expansion) ÷ (churned + contraction).
    pub quick_ratio: f64,
    /// Health band: "excellent", "healthy", "stalling", or "shrinking".
    pub health: String,
    pub valid: bool,
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn round4(x: f64) -> f64 {
    (x * 10_000.0).round() / 10_000.0
}

pub fn generate(i: &QuickRatioInput) -> QuickRatioReport {
    let gained = i.new_mrr_usd + i.expansion_mrr_usd;
    let lost = i.churned_mrr_usd + i.contraction_mrr_usd;
    if lost <= 0.0 {
        // No losses: ratio is undefined (infinite). Flag invalid rather than divide.
        return QuickRatioReport::default();
    }
    let ratio = gained / lost;
    let health = if ratio >= 4.0 {
        "excellent"
    } else if ratio >= 2.0 {
        "healthy"
    } else if ratio >= 1.0 {
        "stalling"
    } else {
        "shrinking"
    };
    QuickRatioReport {
        gained_mrr_usd: cents(gained),
        lost_mrr_usd: cents(lost),
        net_new_mrr_usd: cents(gained - lost),
        quick_ratio: round4(ratio),
        health: health.to_string(),
        valid: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    fn base() -> QuickRatioInput {
        QuickRatioInput {
            new_mrr_usd: 40_000.0,
            expansion_mrr_usd: 20_000.0,
            churned_mrr_usd: 15_000.0,
            contraction_mrr_usd: 5_000.0,
        }
    }

    #[test]
    fn healthy_ratio_three() {
        let d = generate(&base());
        assert!(close(d.gained_mrr_usd, 60_000.0));
        assert!(close(d.lost_mrr_usd, 20_000.0));
        assert!(close(d.net_new_mrr_usd, 40_000.0));
        assert!(close(d.quick_ratio, 3.0));
        assert_eq!(d.health, "healthy");
    }

    #[test]
    fn shrinking_below_one() {
        let d = generate(&QuickRatioInput { new_mrr_usd: 10_000.0, expansion_mrr_usd: 5_000.0, churned_mrr_usd: 20_000.0, contraction_mrr_usd: 10_000.0 });
        // 15000/30000 = 0.5.
        assert!(close(d.quick_ratio, 0.5));
        assert_eq!(d.health, "shrinking");
        assert!(d.net_new_mrr_usd < 0.0);
    }

    #[test]
    fn excellent_at_four() {
        let d = generate(&QuickRatioInput { new_mrr_usd: 70_000.0, expansion_mrr_usd: 10_000.0, churned_mrr_usd: 15_000.0, contraction_mrr_usd: 5_000.0 });
        // 80000/20000 = 4.0.
        assert!(close(d.quick_ratio, 4.0));
        assert_eq!(d.health, "excellent");
    }

    #[test]
    fn stalling_band() {
        let d = generate(&QuickRatioInput { new_mrr_usd: 18_000.0, expansion_mrr_usd: 12_000.0, churned_mrr_usd: 15_000.0, contraction_mrr_usd: 5_000.0 });
        // 30000/20000 = 1.5.
        assert!(close(d.quick_ratio, 1.5));
        assert_eq!(d.health, "stalling");
    }

    #[test]
    fn zero_losses_invalid() {
        let d = generate(&QuickRatioInput { new_mrr_usd: 50_000.0, expansion_mrr_usd: 0.0, churned_mrr_usd: 0.0, contraction_mrr_usd: 0.0 });
        assert!(!d.valid);
    }
}
