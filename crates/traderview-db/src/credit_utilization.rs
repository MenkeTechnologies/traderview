//! Credit utilization tracker.
//!
//! FICO and VantageScore both weight credit utilization (revolving
//! balances / credit limits) heavily in their score formulas — FICO
//! puts "amounts owed" at ~30% of the score, and utilization is the
//! dominant component within that bucket.
//!
//! Standard practical rules:
//!   - aggregate utilization ≤ 30% — minimum to avoid score damage
//!   - aggregate utilization ≤ 10% — Experian-published "excellent" tier
//!   - aggregate utilization  ≤ 1% — observed FICO 800-club median,
//!     leaving at least one card with a
//!     nonzero reported balance
//!   - per-card utilization ≤ 30% — individual cards above 30% can ding
//!     the score even if aggregate is fine
//!
//! Inputs: list of `{ name, balance_usd, limit_usd }`. Compute:
//!   - per-card utilization_pct + status (good ≤ 10 / ok ≤ 30 / high > 30)
//!     + recommended_paydown_to_30  (max(0, balance − 0.30·limit))
//!   - aggregate utilization (Σ balance / Σ limit)
//!   - aggregate status
//!   - count of cards above 30%
//!   - total paydown needed to bring every card AND aggregate under 30%
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct CardInput {
    pub name: String,
    pub balance_usd: f64,
    pub limit_usd: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreditUtilizationInput {
    #[serde(default)]
    pub cards: Vec<CardInput>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CardResult {
    pub name: String,
    pub balance_usd: f64,
    pub limit_usd: f64,
    pub utilization_pct: f64,
    pub status: &'static str,
    pub recommended_paydown_to_30_usd: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreditUtilizationReport {
    pub cards: Vec<CardResult>,
    pub total_balance_usd: f64,
    pub total_limit_usd: f64,
    pub aggregate_utilization_pct: f64,
    pub aggregate_status: &'static str,
    pub cards_above_30_count: usize,
    pub total_paydown_recommended_usd: f64,
}

// ─── Pure compute ─────────────────────────────────────────────────────────

pub fn utilization_status(pct: f64) -> &'static str {
    if pct <= 10.0 { "good" }
    else if pct <= 30.0 { "ok" }
    else { "high" }
}

pub fn card_utilization(balance: f64, limit: f64) -> f64 {
    if limit <= 0.0 {
        return if balance > 0.0 { 100.0 } else { 0.0 };
    }
    balance / limit * 100.0
}

pub fn paydown_to_30(balance: f64, limit: f64) -> f64 {
    if limit <= 0.0 {
        return balance.max(0.0);
    }
    let target = limit * 0.30;
    (balance - target).max(0.0)
}

pub fn evaluate_card(c: &CardInput) -> CardResult {
    let util = card_utilization(c.balance_usd, c.limit_usd);
    let status = utilization_status(util);
    let paydown = paydown_to_30(c.balance_usd, c.limit_usd);
    CardResult {
        name: c.name.clone(),
        balance_usd: c.balance_usd,
        limit_usd: c.limit_usd,
        utilization_pct: util,
        status,
        recommended_paydown_to_30_usd: paydown,
    }
}

pub fn compute(input: &CreditUtilizationInput) -> CreditUtilizationReport {
    let cards: Vec<CardResult> = input.cards.iter().map(evaluate_card).collect();
    let total_balance: f64 = cards.iter().map(|c| c.balance_usd).sum();
    let total_limit: f64 = cards.iter().map(|c| c.limit_usd).sum();
    let agg = card_utilization(total_balance, total_limit);
    let agg_status = utilization_status(agg);
    let above_30 = cards.iter().filter(|c| c.utilization_pct > 30.0).count();
    // Total paydown = sum of per-card paydowns required to bring all to ≤30%,
    // plus any additional needed to bring AGGREGATE ≤ 30%.
    let per_card_pd: f64 = cards.iter().map(|c| c.recommended_paydown_to_30_usd).sum();
    let agg_target = total_limit * 0.30;
    let agg_pd_remaining = if total_balance - per_card_pd > agg_target {
        // After per-card paydowns, aggregate still over 30 → reduce more.
        (total_balance - per_card_pd - agg_target).max(0.0)
    } else {
        0.0
    };
    let total_pd = per_card_pd + agg_pd_remaining;
    CreditUtilizationReport {
        cards,
        total_balance_usd: total_balance,
        total_limit_usd: total_limit,
        aggregate_utilization_pct: agg,
        aggregate_status: agg_status,
        cards_above_30_count: above_30,
        total_paydown_recommended_usd: total_pd,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c(name: &str, bal: f64, lim: f64) -> CardInput {
        CardInput { name: name.into(), balance_usd: bal, limit_usd: lim }
    }

    #[test]
    fn util_status_thresholds() {
        assert_eq!(utilization_status(5.0), "good");
        assert_eq!(utilization_status(10.0), "good");
        assert_eq!(utilization_status(25.0), "ok");
        assert_eq!(utilization_status(30.0), "ok");
        assert_eq!(utilization_status(31.0), "high");
        assert_eq!(utilization_status(90.0), "high");
    }

    #[test]
    fn card_utilization_zero_limit_with_balance() {
        assert_eq!(card_utilization(100.0, 0.0), 100.0);
        assert_eq!(card_utilization(0.0, 0.0), 0.0);
    }

    #[test]
    fn card_utilization_basic() {
        assert_eq!(card_utilization(300.0, 1000.0), 30.0);
    }

    #[test]
    fn paydown_to_30_already_under_returns_zero() {
        assert_eq!(paydown_to_30(200.0, 1000.0), 0.0);
    }

    #[test]
    fn paydown_to_30_basic() {
        // Limit 1000, balance 500 → target 300 → pay down 200.
        assert_eq!(paydown_to_30(500.0, 1000.0), 200.0);
    }

    #[test]
    fn paydown_to_30_zero_limit_treats_balance_as_paydown() {
        assert_eq!(paydown_to_30(100.0, 0.0), 100.0);
    }

    #[test]
    fn evaluate_card_high_util() {
        let r = evaluate_card(&c("cc", 800.0, 1000.0));
        assert_eq!(r.utilization_pct, 80.0);
        assert_eq!(r.status, "high");
        assert_eq!(r.recommended_paydown_to_30_usd, 500.0);
    }

    #[test]
    fn evaluate_card_good_util() {
        let r = evaluate_card(&c("cc", 50.0, 1000.0));
        assert_eq!(r.utilization_pct, 5.0);
        assert_eq!(r.status, "good");
        assert_eq!(r.recommended_paydown_to_30_usd, 0.0);
    }

    #[test]
    fn compute_aggregate_status_low() {
        let r = compute(&CreditUtilizationInput {
            cards: vec![c("a", 100.0, 10_000.0)],
        });
        assert_eq!(r.aggregate_utilization_pct, 1.0);
        assert_eq!(r.aggregate_status, "good");
        assert_eq!(r.cards_above_30_count, 0);
        assert_eq!(r.total_paydown_recommended_usd, 0.0);
    }

    #[test]
    fn compute_aggregate_above_30_with_paydown() {
        let r = compute(&CreditUtilizationInput {
            cards: vec![
                c("a", 800.0, 1000.0),   // 80% — paydown 500
                c("b", 200.0, 1000.0),   // 20% — paydown 0
            ],
        });
        assert_eq!(r.total_balance_usd, 1000.0);
        assert_eq!(r.total_limit_usd, 2000.0);
        assert_eq!(r.aggregate_utilization_pct, 50.0);
        assert_eq!(r.aggregate_status, "high");
        assert_eq!(r.cards_above_30_count, 1);
        // Per-card paydown of 500 brings total to 500 / 2000 = 25% < 30 → done.
        assert_eq!(r.total_paydown_recommended_usd, 500.0);
    }

    #[test]
    fn compute_aggregate_still_high_after_per_card() {
        // Five cards each at 50% utilization → per-card paydowns each
        // bring them to 30%, leaving aggregate at 30% — exactly target.
        let r = compute(&CreditUtilizationInput {
            cards: vec![
                c("a", 500.0, 1000.0),
                c("b", 500.0, 1000.0),
                c("c", 500.0, 1000.0),
                c("d", 500.0, 1000.0),
                c("e", 500.0, 1000.0),
            ],
        });
        assert_eq!(r.aggregate_utilization_pct, 50.0);
        // Per-card paydown each = 200, total 1000. After: each at 30%,
        // aggregate at 1500/5000 = 30% (exactly at target).
        assert_eq!(r.total_paydown_recommended_usd, 1000.0);
    }

    #[test]
    fn compute_empty_input_safe() {
        let r = compute(&CreditUtilizationInput { cards: vec![] });
        assert_eq!(r.aggregate_utilization_pct, 0.0);
        assert_eq!(r.aggregate_status, "good");
        assert_eq!(r.cards.len(), 0);
    }
}
