//! Dollar-cost-averaging scheduler + lump-sum comparison.
//!
//! The single most reliable retail strategy — buy $N every period for
//! Y years and ignore everything else. Backtests show that lump-sum
//! often beats DCA in absolute return (because markets generally go up
//! and time-in-market wins) but DCA wins on emotional sustainability +
//! lower drawdown risk + much lower regret.
//!
//! Brokerages don't surface this comparison; this module does:
//!
//!   1. Walk through cached daily closes, simulate buying
//!      `contribution_usd` worth of `symbol` on the first trading day
//!      of every period (weekly/monthly/quarterly).
//!   2. Accumulate shares, track total contributed.
//!   3. Report avg cost, final value, simple gain pct.
//!   4. Compare against lump-sum investing all the money at period 0.
//!
//! Pure compute is fully unit-tested. Repository pulls bars and feeds
//! the simulator.

use chrono::{Datelike, Duration, NaiveDate, Utc};
use rust_decimal::prelude::ToPrimitive;
use serde::Serialize;
use sqlx::PgPool;
use traderview_core::BarInterval;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DcaFrequency {
    Weekly,
    Monthly,
    Quarterly,
}

impl DcaFrequency {
    /// Period length in days (approximation; the simulator uses the
    /// first calendar period match).
    pub fn days(self) -> i64 {
        match self {
            DcaFrequency::Weekly => 7,
            DcaFrequency::Monthly => 30,
            DcaFrequency::Quarterly => 91,
        }
    }
    /// Number of periods per year.
    pub fn per_year(self) -> f64 {
        match self {
            DcaFrequency::Weekly => 52.0,
            DcaFrequency::Monthly => 12.0,
            DcaFrequency::Quarterly => 4.0,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DcaPurchase {
    pub purchase_date: NaiveDate,
    pub purchase_price: f64,
    pub shares_bought: f64,
    pub running_shares: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DcaResult {
    pub symbol: String,
    pub frequency: String,
    pub n_purchases: usize,
    pub total_contributed_usd: f64,
    pub total_shares: f64,
    pub avg_cost_per_share: f64,
    pub current_price: f64,
    pub final_value_usd: f64,
    pub dca_total_return_pct: f64,
    pub dca_annualised_return_pct: f64,
    pub lump_sum_final_value_usd: f64,
    pub lump_sum_total_return_pct: f64,
    pub lump_sum_annualised_return_pct: f64,
    /// dca_final / lump_sum_final — > 1.0 means DCA outperformed.
    pub dca_vs_lump_ratio: f64,
    pub purchases: Vec<DcaPurchase>,
}

// ─── Pure compute ──────────────────────────────────────────────────────────

/// Find the first close on or after `target_date` in a sorted close
/// series. Returns the (date, price) pair.
fn close_at_or_after(closes: &[(NaiveDate, f64)], target: NaiveDate) -> Option<(NaiveDate, f64)> {
    closes.iter().find(|(d, _)| *d >= target).copied()
}

/// Step a date forward by one period.
fn next_period(d: NaiveDate, freq: DcaFrequency) -> NaiveDate {
    match freq {
        DcaFrequency::Weekly => d + Duration::days(7),
        DcaFrequency::Monthly => add_months(d, 1),
        DcaFrequency::Quarterly => add_months(d, 3),
    }
}

fn add_months(d: NaiveDate, n: u32) -> NaiveDate {
    let mut year = d.year();
    let mut month = d.month() as i32;
    month += n as i32;
    while month > 12 {
        month -= 12;
        year += 1;
    }
    NaiveDate::from_ymd_opt(year, month as u32, d.day()).unwrap_or(d)
}

/// Annualised return from a single absolute return % and the holding
/// period in years. Uses `(1 + r)^(1/years) - 1` form.
pub fn annualised_return_pct(absolute_return_pct: f64, years: f64) -> f64 {
    if years <= 0.0 {
        return 0.0;
    }
    let r = absolute_return_pct / 100.0;
    let total_multiplier = 1.0 + r;
    if total_multiplier <= 0.0 {
        return -100.0;
    }
    (total_multiplier.powf(1.0 / years) - 1.0) * 100.0
}

/// Run the DCA simulation given a sorted close series + parameters.
/// Returns `None` when the data is empty or no purchase can be made.
pub fn simulate_dca(
    symbol: &str,
    closes: &[(NaiveDate, f64)],
    contribution_usd: f64,
    frequency: DcaFrequency,
) -> Option<DcaResult> {
    if closes.is_empty() || contribution_usd <= 0.0 {
        return None;
    }
    let first_date = closes.first().unwrap().0;
    let final_date = closes.last().unwrap().0;
    let final_price = closes.last().unwrap().1;

    let mut purchases: Vec<DcaPurchase> = Vec::new();
    let mut total_contributed = 0.0_f64;
    let mut total_shares = 0.0_f64;
    let mut purchase_date = first_date;

    while purchase_date <= final_date {
        let Some((actual_date, price)) = close_at_or_after(closes, purchase_date) else {
            break;
        };
        if price <= 0.0 {
            purchase_date = next_period(purchase_date, frequency);
            continue;
        }
        let shares = contribution_usd / price;
        total_shares += shares;
        total_contributed += contribution_usd;
        purchases.push(DcaPurchase {
            purchase_date: actual_date,
            purchase_price: price,
            shares_bought: shares,
            running_shares: total_shares,
        });
        purchase_date = next_period(purchase_date, frequency);
    }

    if purchases.is_empty() || total_contributed <= 0.0 {
        return None;
    }

    let avg_cost = total_contributed / total_shares;
    let final_value = total_shares * final_price;
    let dca_total_return = (final_value - total_contributed) / total_contributed * 100.0;
    let years = (final_date - first_date).num_days() as f64 / 365.25;

    // Lump-sum equivalent: invest the same total at the first purchase price.
    let lump_sum_shares = total_contributed / purchases[0].purchase_price;
    let lump_sum_final = lump_sum_shares * final_price;
    let lump_sum_return = (lump_sum_final - total_contributed) / total_contributed * 100.0;

    let dca_ann = annualised_return_pct(dca_total_return, years);
    let lump_ann = annualised_return_pct(lump_sum_return, years);
    let ratio = if lump_sum_final > 0.0 {
        final_value / lump_sum_final
    } else {
        0.0
    };

    Some(DcaResult {
        symbol: symbol.into(),
        frequency: match frequency {
            DcaFrequency::Weekly => "weekly".into(),
            DcaFrequency::Monthly => "monthly".into(),
            DcaFrequency::Quarterly => "quarterly".into(),
        },
        n_purchases: purchases.len(),
        total_contributed_usd: total_contributed,
        total_shares,
        avg_cost_per_share: avg_cost,
        current_price: final_price,
        final_value_usd: final_value,
        dca_total_return_pct: dca_total_return,
        dca_annualised_return_pct: dca_ann,
        lump_sum_final_value_usd: lump_sum_final,
        lump_sum_total_return_pct: lump_sum_return,
        lump_sum_annualised_return_pct: lump_ann,
        dca_vs_lump_ratio: ratio,
        purchases,
    })
}

// ─── Repository ────────────────────────────────────────────────────────────

pub async fn run(
    pool: &PgPool,
    symbol: &str,
    contribution_usd: f64,
    frequency: DcaFrequency,
    days_back: i64,
) -> anyhow::Result<Option<DcaResult>> {
    let to = Utc::now();
    let from = to - Duration::days(days_back);
    let bars = crate::prices::get_bars(pool, symbol, BarInterval::D1, from, to)
        .await
        .unwrap_or_default();
    let closes: Vec<(NaiveDate, f64)> = bars
        .into_iter()
        .filter_map(|b| b.close.to_f64().map(|c| (b.bar_time.date_naive(), c)))
        .collect();
    Ok(simulate_dca(symbol, &closes, contribution_usd, frequency))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    fn flat_closes(start: NaiveDate, n: usize, price: f64) -> Vec<(NaiveDate, f64)> {
        (0..n)
            .map(|i| (start + Duration::days(i as i64), price))
            .collect()
    }

    fn rising_closes(start: NaiveDate, n: usize, daily_pct: f64) -> Vec<(NaiveDate, f64)> {
        let mut p = 100.0_f64;
        (0..n)
            .map(|i| {
                let row = (start + Duration::days(i as i64), p);
                p *= 1.0 + daily_pct / 100.0;
                row
            })
            .collect()
    }

    #[test]
    fn simulate_returns_none_on_empty_closes() {
        assert!(simulate_dca("X", &[], 100.0, DcaFrequency::Monthly).is_none());
    }

    #[test]
    fn simulate_returns_none_on_zero_contribution() {
        let closes = flat_closes(d(2026, 1, 1), 30, 100.0);
        assert!(simulate_dca("X", &closes, 0.0, DcaFrequency::Monthly).is_none());
    }

    #[test]
    fn simulate_flat_price_dca_equals_lump_sum() {
        // Constant price → every purchase is at same price → DCA result
        // identical to lump-sum.
        let closes = flat_closes(d(2026, 1, 1), 365, 100.0);
        let r = simulate_dca("X", &closes, 100.0, DcaFrequency::Monthly).unwrap();
        // 12 monthly purchases at $100/sh, $100 each → 1 share each → 12 shares.
        assert_eq!(r.n_purchases, 12);
        assert!((r.total_contributed_usd - 1200.0).abs() < 1e-9);
        assert!((r.total_shares - 12.0).abs() < 1e-9);
        assert!((r.avg_cost_per_share - 100.0).abs() < 1e-9);
        // DCA value = 12 × $100 = $1200; lump-sum same. Ratio = 1.
        assert!((r.dca_vs_lump_ratio - 1.0).abs() < 1e-9);
    }

    #[test]
    fn simulate_rising_price_dca_underperforms_lump_sum() {
        // Steady uptrend → lump-sum buys early at lowest price and wins.
        let closes = rising_closes(d(2026, 1, 1), 365, 0.1);
        let r = simulate_dca("X", &closes, 100.0, DcaFrequency::Monthly).unwrap();
        // DCA buys at higher and higher prices → smaller average position.
        // Lump-sum buys all at $100 → bigger position.
        assert!(
            r.dca_vs_lump_ratio < 1.0,
            "rising market: DCA < lump (ratio = {})",
            r.dca_vs_lump_ratio
        );
    }

    #[test]
    fn simulate_falling_price_dca_outperforms_lump_sum() {
        // Falling price → DCA accumulates more shares at lower prices.
        let closes = rising_closes(d(2026, 1, 1), 365, -0.1);
        let r = simulate_dca("X", &closes, 100.0, DcaFrequency::Monthly).unwrap();
        assert!(
            r.dca_vs_lump_ratio > 1.0,
            "falling market: DCA > lump (ratio = {})",
            r.dca_vs_lump_ratio
        );
    }

    #[test]
    fn weekly_frequency_more_purchases_than_monthly() {
        let closes = flat_closes(d(2026, 1, 1), 365, 100.0);
        let weekly = simulate_dca("X", &closes, 100.0, DcaFrequency::Weekly).unwrap();
        let monthly = simulate_dca("X", &closes, 100.0, DcaFrequency::Monthly).unwrap();
        let quarterly = simulate_dca("X", &closes, 100.0, DcaFrequency::Quarterly).unwrap();
        assert!(weekly.n_purchases > monthly.n_purchases);
        assert!(monthly.n_purchases > quarterly.n_purchases);
    }

    #[test]
    fn annualised_return_basic() {
        // 100% gain over 1 year = 100% annualised.
        assert!((annualised_return_pct(100.0, 1.0) - 100.0).abs() < 1e-9);
        // 21% gain over 2 years = 10% annualised: (1.10)^2 = 1.21.
        assert!((annualised_return_pct(21.0, 2.0) - 10.0).abs() < 1e-6);
    }

    #[test]
    fn annualised_return_zero_on_invalid_years() {
        assert_eq!(annualised_return_pct(50.0, 0.0), 0.0);
        assert_eq!(annualised_return_pct(50.0, -1.0), 0.0);
    }

    #[test]
    fn annualised_return_minus_100_on_total_wipeout() {
        assert_eq!(annualised_return_pct(-100.0, 5.0), -100.0);
        assert_eq!(annualised_return_pct(-150.0, 5.0), -100.0);
    }

    #[test]
    fn frequency_helpers_correct() {
        assert_eq!(DcaFrequency::Weekly.days(), 7);
        assert_eq!(DcaFrequency::Monthly.days(), 30);
        assert_eq!(DcaFrequency::Quarterly.days(), 91);
        assert_eq!(DcaFrequency::Weekly.per_year(), 52.0);
        assert_eq!(DcaFrequency::Monthly.per_year(), 12.0);
        assert_eq!(DcaFrequency::Quarterly.per_year(), 4.0);
    }
}
