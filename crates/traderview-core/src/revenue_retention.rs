//! Revenue retention (NRR & GRR) — the core subscription-revenue health metrics.
//! Starting from a cohort's recurring revenue at the period start, expansion adds
//! revenue (upgrades, seats), while contraction (downgrades) and churn (cancels)
//! remove it. Net revenue retention `NRR = (start + expansion − contraction −
//! churn) ÷ start` can exceed 100% when expansion outruns losses; gross revenue
//! retention `GRR = (start − contraction − churn) ÷ start` ignores expansion and
//! caps at 100%. New-logo revenue is excluded — these measure the existing base.
//! Pure compute, not financial advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct RetentionInput {
    pub period_label: String,
    /// Recurring revenue from the cohort at the start of the period.
    pub starting_mrr_usd: f64,
    /// Upsell / expansion within the cohort.
    #[serde(default)]
    pub expansion_mrr_usd: f64,
    /// Downgrades within the cohort.
    #[serde(default)]
    pub contraction_mrr_usd: f64,
    /// Cancellations within the cohort.
    #[serde(default)]
    pub churned_mrr_usd: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct RetentionReport {
    /// Net revenue retention, percent (can exceed 100).
    pub nrr_pct: f64,
    /// Gross revenue retention, percent (≤ 100).
    pub grr_pct: f64,
    /// Expansion − contraction − churn.
    pub net_change_usd: f64,
    /// Starting + net change — the cohort's ending recurring revenue.
    pub ending_mrr_usd: f64,
    /// True when NRR ≥ 100% (the cohort is net-expanding).
    pub net_expanding: bool,
    pub valid: bool,
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

pub fn generate(i: &RetentionInput) -> RetentionReport {
    if i.starting_mrr_usd <= 0.0 {
        return RetentionReport::default();
    }
    let start = i.starting_mrr_usd;
    let net = i.expansion_mrr_usd - i.contraction_mrr_usd - i.churned_mrr_usd;
    let nrr = (start + net) / start * 100.0;
    let grr = (start - i.contraction_mrr_usd - i.churned_mrr_usd) / start * 100.0;
    RetentionReport {
        nrr_pct: round2(nrr),
        grr_pct: round2(grr),
        net_change_usd: cents(net),
        ending_mrr_usd: cents(start + net),
        net_expanding: nrr >= 100.0,
        valid: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    fn base() -> RetentionInput {
        RetentionInput {
            period_label: "Q2".into(),
            starting_mrr_usd: 100_000.0,
            expansion_mrr_usd: 15_000.0,
            contraction_mrr_usd: 5_000.0,
            churned_mrr_usd: 8_000.0,
        }
    }

    #[test]
    fn expanding_cohort() {
        let d = generate(&base());
        assert!(close(d.nrr_pct, 102.0));
        assert!(close(d.grr_pct, 87.0));
        assert!(close(d.net_change_usd, 2_000.0));
        assert!(close(d.ending_mrr_usd, 102_000.0));
        assert!(d.net_expanding);
    }

    #[test]
    fn churning_cohort() {
        let d = generate(&RetentionInput { expansion_mrr_usd: 2_000.0, contraction_mrr_usd: 4_000.0, churned_mrr_usd: 10_000.0, ..base() });
        assert!(close(d.nrr_pct, 88.0));
        assert!(close(d.grr_pct, 86.0));
        assert!(!d.net_expanding);
    }

    #[test]
    fn grr_never_exceeds_100() {
        // Expansion does not lift GRR.
        let d = generate(&RetentionInput { expansion_mrr_usd: 50_000.0, contraction_mrr_usd: 0.0, churned_mrr_usd: 0.0, ..base() });
        assert!(close(d.grr_pct, 100.0));
        assert!(d.nrr_pct > 100.0);
    }

    #[test]
    fn nrr_at_least_grr() {
        let d = generate(&base());
        assert!(d.nrr_pct >= d.grr_pct);
    }

    #[test]
    fn zero_start_invalid() {
        let d = generate(&RetentionInput { starting_mrr_usd: 0.0, ..base() });
        assert!(!d.valid);
    }
}
