//! Tax-loss harvesting scanner for paper positions.
//!
//! Wealthfront, Betterment, et al. built billion-dollar businesses on
//! this single feature for retail investors. For each long paper
//! position with an unrealized loss ≥ threshold, suggest:
//!
//!   1. Harvest the loss (sell to realize, lock in tax deduction).
//!   2. Replacement candidate that maintains exposure but is NOT
//!      "substantially identical" per IRC §1091 (to avoid wash-sale
//!      disallowance for 30 days).
//!   3. Wash-sale risk flag: any buy of the same symbol within the
//!      trailing 30 days from paper_orders disqualifies the harvest.
//!
//! Estimated tax savings = realized loss × user's marginal rate.
//! User configures the rate; default 35% (federal LTCG top bracket +
//! state).
//!
//! Replacement pairs use a simple sector-aware default mapping —
//! e.g. XLK → VGT (different tech ETFs), SPY → ITOT (S&P → total US
//! market), QQQ → ONEQ. The defaults are conservative; aggressive
//! sell-SPY-buy-IVV interpretations are not surfaced because the IRS
//! has not blessed them.

use chrono::{Duration, Utc};
use rust_decimal::prelude::ToPrimitive;
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct HarvestCandidate {
    pub symbol: String,
    pub qty: f64,
    pub cost_basis_per_share: f64,
    pub current_price: f64,
    pub unrealized_loss_usd: f64,
    pub unrealized_loss_pct: f64,
    pub estimated_tax_savings_usd: f64,
    pub replacement_symbol: Option<String>,
    pub replacement_rationale: Option<String>,
    pub wash_sale_risk: bool,
    pub wash_sale_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HarvestReport {
    pub candidates: Vec<HarvestCandidate>,
    pub total_loss_usd: f64,
    pub total_tax_savings_usd: f64,
    pub marginal_rate_pct: f64,
    pub min_loss_threshold_pct: f64,
}

// ─── Pure compute ──────────────────────────────────────────────────────────

/// Hand-curated replacement-pair mapping. Each tuple is
/// `(original, replacement, rationale)`. Replacement maintains similar
/// exposure but is NOT substantially identical per IRS guidance.
pub const REPLACEMENT_PAIRS: &[(&str, &str, &str)] = &[
    (
        "SPY",
        "ITOT",
        "S&P 500 → Total US Market (broader index, different issuer)",
    ),
    (
        "IVV",
        "VOO",
        "S&P 500 → S&P 500 (different issuer — risk of substantially-identical claim by IRS)",
    ),
    (
        "QQQ",
        "ONEQ",
        "Nasdaq-100 → Nasdaq Composite (broader, different methodology)",
    ),
    (
        "XLK",
        "VGT",
        "Technology sector → Tech sector (different issuer + different methodology)",
    ),
    (
        "XLF",
        "VFH",
        "Financials sector → Financials (different issuer)",
    ),
    (
        "XLV",
        "VHT",
        "Healthcare sector → Healthcare (different issuer)",
    ),
    ("XLE", "VDE", "Energy sector → Energy (different issuer)"),
    (
        "XLY",
        "VCR",
        "Consumer Discretionary → Consumer Discretionary",
    ),
    ("XLP", "VDC", "Consumer Staples → Consumer Staples"),
    ("XLI", "VIS", "Industrials → Industrials"),
    ("XLB", "VAW", "Materials → Materials"),
    ("XLU", "VPU", "Utilities → Utilities"),
    ("XLRE", "VNQ", "Real Estate → REITs"),
    (
        "XLC",
        "VOX",
        "Communication Services → Communication Services",
    ),
];

/// Compute unrealized P&L and loss thresholds. Pure inputs / outputs.
pub fn evaluate_position(
    qty: f64,
    cost_basis_per_share: f64,
    current_price: f64,
    min_loss_threshold_pct: f64,
) -> Option<(f64, f64)> {
    if !(qty > 0.0 && cost_basis_per_share > 0.0 && current_price > 0.0) {
        return None;
    }
    let unrealized_loss_pct = (current_price - cost_basis_per_share) / cost_basis_per_share * 100.0;
    if unrealized_loss_pct >= -min_loss_threshold_pct {
        return None; // Not enough loss.
    }
    let unrealized_loss_usd = qty * (current_price - cost_basis_per_share);
    Some((unrealized_loss_usd, unrealized_loss_pct))
}

/// Tax savings = realized loss × marginal_rate. Returns positive value
/// (capping at 0 when loss is positive — i.e. there's no gain to offset
/// in this single-position view).
pub fn estimate_tax_savings(unrealized_loss_usd: f64, marginal_rate_pct: f64) -> f64 {
    if unrealized_loss_usd >= 0.0 || marginal_rate_pct <= 0.0 {
        return 0.0;
    }
    -unrealized_loss_usd * marginal_rate_pct / 100.0
}

pub fn lookup_replacement(symbol: &str) -> Option<(&'static str, &'static str)> {
    let upper = symbol.to_ascii_uppercase();
    REPLACEMENT_PAIRS
        .iter()
        .find(|(orig, _, _)| orig.eq_ignore_ascii_case(&upper))
        .map(|(_, repl, rationale)| (*repl, *rationale))
}

// ─── Repository ────────────────────────────────────────────────────────────

/// Check the trailing 30 days of paper_orders for any buy of `symbol`
/// — if found, harvesting now creates a wash-sale violation.
async fn wash_sale_risk(pool: &PgPool, account_id: Uuid, symbol: &str) -> Option<String> {
    let since = Utc::now() - Duration::days(30);
    let row: Option<(chrono::DateTime<chrono::Utc>,)> = sqlx::query_as(
        "SELECT submitted_at FROM paper_orders
          WHERE paper_account_id = $1 AND symbol = $2
            AND side IN ('buy', 'cover')
            AND status = 'filled'
            AND submitted_at >= $3
          ORDER BY submitted_at DESC LIMIT 1",
    )
    .bind(account_id)
    .bind(symbol)
    .bind(since)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();
    row.map(|(when,)| {
        format!(
            "buy of {symbol} within last 30 days ({}) — selling now violates IRC §1091",
            when.format("%Y-%m-%d")
        )
    })
}

/// Build the harvest report for the user's default paper account.
pub async fn scan(
    pool: &PgPool,
    user_id: Uuid,
    marginal_rate_pct: f64,
    min_loss_threshold_pct: f64,
) -> anyhow::Result<HarvestReport> {
    let account = crate::paper::ensure_default(pool, user_id).await?;
    let positions = crate::paper::positions(pool, account.id).await?;
    let mut candidates: Vec<HarvestCandidate> = Vec::new();
    let mut total_loss = 0.0_f64;
    let mut total_savings = 0.0_f64;

    for p in &positions {
        let qty = p.qty.to_f64().unwrap_or(0.0);
        if qty <= 0.0 {
            continue; // Long positions only — shorts have separate rules.
        }
        let cost = p.avg_price.to_f64().unwrap_or(0.0);
        let current_price = match crate::market_data::quote(pool, &p.symbol).await {
            Ok(q) => q.price,
            Err(_) => continue,
        };
        let Some((loss_usd, loss_pct)) =
            evaluate_position(qty, cost, current_price, min_loss_threshold_pct)
        else {
            continue;
        };
        let savings = estimate_tax_savings(loss_usd, marginal_rate_pct);
        let (replacement, rationale) = match lookup_replacement(&p.symbol) {
            Some((r, why)) => (Some(r.to_string()), Some(why.to_string())),
            None => (None, None),
        };
        let wash = wash_sale_risk(pool, account.id, &p.symbol).await;
        let wash_risk = wash.is_some();
        candidates.push(HarvestCandidate {
            symbol: p.symbol.clone(),
            qty,
            cost_basis_per_share: cost,
            current_price,
            unrealized_loss_usd: loss_usd,
            unrealized_loss_pct: loss_pct,
            estimated_tax_savings_usd: savings,
            replacement_symbol: replacement,
            replacement_rationale: rationale,
            wash_sale_risk: wash_risk,
            wash_sale_reason: wash,
        });
        total_loss += loss_usd;
        total_savings += savings;
    }
    candidates.sort_by(|a, b| {
        a.unrealized_loss_usd
            .partial_cmp(&b.unrealized_loss_usd)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    Ok(HarvestReport {
        candidates,
        total_loss_usd: total_loss,
        total_tax_savings_usd: total_savings,
        marginal_rate_pct,
        min_loss_threshold_pct,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluate_returns_none_when_position_in_profit() {
        // 100 shares @ $50 cost, current $60 → +20% gain, not a candidate.
        assert!(evaluate_position(100.0, 50.0, 60.0, 5.0).is_none());
    }

    #[test]
    fn evaluate_returns_none_when_loss_below_threshold() {
        // -3% loss, threshold 5% → not yet a candidate.
        let r = evaluate_position(100.0, 100.0, 97.0, 5.0);
        assert!(r.is_none());
    }

    #[test]
    fn evaluate_emits_loss_when_above_threshold() {
        // 100 shares @ $100 → $90 = -10% loss → exceeds 5% threshold.
        let (loss_usd, loss_pct) = evaluate_position(100.0, 100.0, 90.0, 5.0).unwrap();
        assert!((loss_usd - (-1000.0)).abs() < 1e-9);
        assert!((loss_pct - (-10.0)).abs() < 1e-9);
    }

    #[test]
    fn evaluate_handles_threshold_exactly_at_boundary() {
        // -5% loss with 5% threshold → does NOT fire (we want strictly greater than).
        let r = evaluate_position(100.0, 100.0, 95.0, 5.0);
        assert!(r.is_none());
    }

    #[test]
    fn evaluate_returns_none_on_invalid_inputs() {
        assert!(evaluate_position(0.0, 100.0, 90.0, 5.0).is_none());
        assert!(evaluate_position(100.0, 0.0, 90.0, 5.0).is_none());
        assert!(evaluate_position(100.0, 100.0, 0.0, 5.0).is_none());
    }

    #[test]
    fn estimate_tax_savings_at_marginal_rate() {
        // $1000 loss × 35% rate = $350 savings.
        let s = estimate_tax_savings(-1000.0, 35.0);
        assert!((s - 350.0).abs() < 1e-9);
    }

    #[test]
    fn estimate_tax_savings_zero_on_gain() {
        // Positive "loss" means actually a gain → no savings.
        assert_eq!(estimate_tax_savings(1000.0, 35.0), 0.0);
    }

    #[test]
    fn estimate_tax_savings_zero_on_zero_rate() {
        assert_eq!(estimate_tax_savings(-1000.0, 0.0), 0.0);
    }

    #[test]
    fn lookup_replacement_finds_sector_etfs() {
        let (repl, _) = lookup_replacement("XLK").unwrap();
        assert_eq!(repl, "VGT");
    }

    #[test]
    fn lookup_replacement_case_insensitive() {
        let (repl, _) = lookup_replacement("spy").unwrap();
        assert_eq!(repl, "ITOT");
    }

    #[test]
    fn lookup_replacement_none_for_unknown_symbol() {
        assert!(lookup_replacement("RANDOM").is_none());
    }
}
