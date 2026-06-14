//! DRIP simulator — Dividend Reinvestment Plan compounding vs taking dividends
//! in cash. Each year the dividend per share grows, the price appreciates, and
//! under DRIP every (after-tax) dividend buys more shares at the mid-year price,
//! so the share count compounds. The cash path keeps the original shares and
//! pockets the (unspent) dividends. Reports both end-states, the share count,
//! CAGR of each path, the DRIP advantage, and milestone rows. Faithful port of
//! the former client-side calculator. Pure compute, not advice.

use serde::{Deserialize, Serialize};

const MILESTONES: [u32; 10] = [1, 5, 10, 15, 20, 25, 30, 40, 50, 60];

#[derive(Debug, Clone, Deserialize)]
pub struct DripInput {
    pub shares: f64,
    pub price_usd: f64,
    pub starting_yield_pct: f64,
    pub dividend_growth_pct: f64,
    pub price_growth_pct: f64,
    pub years: u32,
    /// Dividend tax rate applied to reinvested dividends (DRIP path only).
    #[serde(default)]
    pub dividend_tax_pct: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DripRow {
    pub year: u32,
    pub price_usd: f64,
    pub div_per_share_usd: f64,
    pub drip_shares: f64,
    pub drip_value_usd: f64,
    pub cash_shares: f64,
    pub cash_value_usd: f64,
    pub cash_divs_accum_usd: f64,
    pub cash_total_return_usd: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct DripReport {
    pub principal_usd: f64,
    pub drip_final_value_usd: f64,
    pub drip_shares: f64,
    pub final_price_usd: f64,
    pub drip_cagr_pct: f64,
    /// Cash path total: remaining-share value + accumulated cash dividends.
    pub cash_total_return_usd: f64,
    pub cash_share_value_usd: f64,
    pub cash_divs_accum_usd: f64,
    pub cash_cagr_pct: f64,
    pub drip_advantage_usd: f64,
    pub drip_advantage_pct: f64,
    pub rows: Vec<DripRow>,
    pub valid: bool,
}

fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn round4(x: f64) -> f64 {
    (x * 10_000.0).round() / 10_000.0
}

pub fn generate(i: &DripInput) -> DripReport {
    if i.shares <= 0.0 || i.price_usd <= 0.0 || i.years == 0 {
        return DripReport::default();
    }
    let yield0 = i.starting_yield_pct / 100.0;
    let div_grow = i.dividend_growth_pct / 100.0;
    let price_grow = i.price_growth_pct / 100.0;
    let tax = i.dividend_tax_pct / 100.0;

    let principal = i.shares * i.price_usd;
    let mut drip_shares = i.shares;
    let cash_shares = i.shares;
    let mut cash_divs_accum = 0.0;
    let mut div_per_share = i.price_usd * yield0; // year-0 dividend per share
    let mut price = i.price_usd;
    let mut rows = Vec::new();

    for y in 1..=i.years {
        let yoy_price = price * (1.0 + price_grow);
        let yoy_div = div_per_share * (1.0 + div_grow);
        // DRIP buys at the mid-year (average) price.
        let drip_purchase_price = (price + yoy_price) / 2.0;
        let total_drip_div = drip_shares * yoy_div;
        let after_tax_drip_div = total_drip_div * (1.0 - tax);
        drip_shares += after_tax_drip_div / drip_purchase_price;
        let cash_div = cash_shares * yoy_div;
        cash_divs_accum += cash_div;
        price = yoy_price;
        div_per_share = yoy_div;

        if MILESTONES.contains(&y) || y == i.years {
            let cash_value = cash_shares * price;
            rows.push(DripRow {
                year: y,
                price_usd: round2(price),
                div_per_share_usd: round2(yoy_div),
                drip_shares: round4(drip_shares),
                drip_value_usd: round2(drip_shares * price),
                cash_shares: round4(cash_shares),
                cash_value_usd: round2(cash_value),
                cash_divs_accum_usd: round2(cash_divs_accum),
                cash_total_return_usd: round2(cash_value + cash_divs_accum),
            });
        }
    }

    let drip_value = drip_shares * price;
    let cash_value = cash_shares * price;
    let cash_total = cash_value + cash_divs_accum;
    let yrs = i.years as f64;
    let drip_cagr = (drip_value / principal).powf(1.0 / yrs) - 1.0;
    let cash_cagr = (cash_total / principal).powf(1.0 / yrs) - 1.0;
    let advantage = drip_value - cash_total;

    DripReport {
        principal_usd: round2(principal),
        drip_final_value_usd: round2(drip_value),
        drip_shares: round4(drip_shares),
        final_price_usd: round2(price),
        drip_cagr_pct: round4(drip_cagr * 100.0),
        cash_total_return_usd: round2(cash_total),
        cash_share_value_usd: round2(cash_value),
        cash_divs_accum_usd: round2(cash_divs_accum),
        cash_cagr_pct: round4(cash_cagr * 100.0),
        drip_advantage_usd: round2(advantage),
        drip_advantage_pct: round4(if cash_total != 0.0 { advantage / cash_total * 100.0 } else { 0.0 }),
        rows,
        valid: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    fn base() -> DripInput {
        DripInput {
            shares: 100.0,
            price_usd: 100.0,
            starting_yield_pct: 3.0,
            dividend_growth_pct: 5.0,
            price_growth_pct: 5.0,
            years: 30,
            dividend_tax_pct: 15.0,
        }
    }

    // Pins cross-checked against the JS compute() in Python.
    #[test]
    fn default_drip_beats_cash() {
        let d = generate(&base());
        assert!(d.valid);
        assert!(close(d.principal_usd, 10_000.0));
        assert!(close(d.drip_final_value_usd, 93_680.31));
        assert!(close(d.drip_shares, 216.7551));
        assert!(close(d.final_price_usd, 432.19));
        assert!(close(d.cash_total_return_usd, 64_147.66));
        assert!(close(d.cash_share_value_usd, 43_219.42));
        assert!(close(d.cash_divs_accum_usd, 20_928.24));
        assert!(close(d.drip_cagr_pct, 7.7428));
        assert!(close(d.cash_cagr_pct, 6.3913));
        assert!(close(d.drip_advantage_usd, 29_532.65));
        // Milestone years 1,5,10,15,20,25,30 → 7 rows.
        assert_eq!(d.rows.len(), 7);
        assert_eq!(d.rows.last().unwrap().year, 30);
    }

    #[test]
    fn zero_tax_keeps_more_shares() {
        let taxed = generate(&base());
        let untaxed = generate(&DripInput { dividend_tax_pct: 0.0, ..base() });
        assert!(untaxed.drip_shares > taxed.drip_shares);
        assert!(untaxed.drip_final_value_usd > taxed.drip_final_value_usd);
    }

    #[test]
    fn short_horizon_row_count() {
        let d = generate(&DripInput { years: 3, ..base() });
        // Milestone 1 plus final year 3 → 2 rows.
        assert_eq!(d.rows.len(), 2);
        assert_eq!(d.rows[0].year, 1);
        assert_eq!(d.rows[1].year, 3);
    }

    #[test]
    fn invalid_when_shares_zero() {
        let d = generate(&DripInput { shares: 0.0, ..base() });
        assert!(!d.valid);
    }
}
