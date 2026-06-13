//! Profit First allocation — Mike Michalowicz's cash-management method.
//!
//! Instead of "Sales − Expenses = Profit" (profit is whatever's left, often
//! nothing), Profit First takes profit off the top: "Sales − Profit =
//! Expenses." Real revenue (total revenue minus the cost of materials and
//! subcontractors) is split across four accounts by Target Allocation
//! Percentages (TAPs), chosen by the business's annual real-revenue band:
//!
//! | Real revenue   | Profit | Owner's Pay | Tax | OpEx |
//! |----------------|--------|-------------|-----|------|
//! | < $250K        |   5%   |     50%     | 15% |  30% |
//! | $250K – $500K  |  10%   |     35%     | 15% |  40% |
//! | $500K – $1M    |  15%   |     20%     | 15% |  50% |
//! | $1M – $5M      |  20%   |     10%     | 15% |  55% |
//! | $5M+           |  25%   |      5%     | 15% |  55% |
//!
//! (Source: *Profit First*, Michalowicz; TAP table per Relay's published
//! summary.) Each band sums to 100%. The caller may override with custom
//! percentages. This is the business-side analog of the personal
//! savings-waterfall. Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct ProfitFirstInput {
    pub annual_revenue_usd: f64,
    /// Cost of materials and subcontractors — subtracted to get real revenue.
    #[serde(default)]
    pub materials_subcontractors_usd: f64,
    /// Optional custom TAPs. When the four sum to ~100, they override the
    /// band table; otherwise the band for the real revenue is used.
    #[serde(default)]
    pub profit_pct: f64,
    #[serde(default)]
    pub owner_pay_pct: f64,
    #[serde(default)]
    pub tax_pct: f64,
    #[serde(default)]
    pub opex_pct: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProfitFirstResult {
    /// Revenue minus materials and subcontractors.
    pub real_revenue_usd: f64,
    /// Which TAP band was applied (or "Custom").
    pub band: String,
    pub profit_pct: f64,
    pub owner_pay_pct: f64,
    pub tax_pct: f64,
    pub opex_pct: f64,
    pub profit_usd: f64,
    pub owner_pay_usd: f64,
    pub tax_usd: f64,
    pub opex_usd: f64,
}

/// (label, profit, owner_pay, tax, opex) for an annual real-revenue figure.
fn band_for(real_revenue: f64) -> (&'static str, f64, f64, f64, f64) {
    if real_revenue < 250_000.0 {
        ("Under $250K", 5.0, 50.0, 15.0, 30.0)
    } else if real_revenue < 500_000.0 {
        ("$250K–$500K", 10.0, 35.0, 15.0, 40.0)
    } else if real_revenue < 1_000_000.0 {
        ("$500K–$1M", 15.0, 20.0, 15.0, 50.0)
    } else if real_revenue < 5_000_000.0 {
        ("$1M–$5M", 20.0, 10.0, 15.0, 55.0)
    } else {
        ("$5M+", 25.0, 5.0, 15.0, 55.0)
    }
}

pub fn analyze(i: &ProfitFirstInput) -> ProfitFirstResult {
    let real_revenue = (i.annual_revenue_usd - i.materials_subcontractors_usd).max(0.0);

    let custom_sum = i.profit_pct + i.owner_pay_pct + i.tax_pct + i.opex_pct;
    let (band, profit_pct, owner_pay_pct, tax_pct, opex_pct) = if (custom_sum - 100.0).abs() < 0.5 {
        ("Custom", i.profit_pct, i.owner_pay_pct, i.tax_pct, i.opex_pct)
    } else {
        band_for(real_revenue)
    };

    let alloc = |pct: f64| real_revenue * pct / 100.0;
    ProfitFirstResult {
        real_revenue_usd: real_revenue,
        band: band.to_string(),
        profit_pct,
        owner_pay_pct,
        tax_pct,
        opex_pct,
        profit_usd: alloc(profit_pct),
        owner_pay_usd: alloc(owner_pay_pct),
        tax_usd: alloc(tax_pct),
        opex_usd: alloc(opex_pct),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inp(rev: f64, mat: f64) -> ProfitFirstInput {
        ProfitFirstInput {
            annual_revenue_usd: rev,
            materials_subcontractors_usd: mat,
            profit_pct: 0.0,
            owner_pay_pct: 0.0,
            tax_pct: 0.0,
            opex_pct: 0.0,
        }
    }

    #[test]
    fn real_revenue_nets_materials() {
        let r = analyze(&inp(100_000.0, 30_000.0));
        assert!((r.real_revenue_usd - 70_000.0).abs() < 1e-6);
    }

    #[test]
    fn band_under_250k() {
        let r = analyze(&inp(200_000.0, 0.0));
        assert_eq!(r.band, "Under $250K");
        assert!((r.profit_pct - 5.0).abs() < 1e-9);
        assert!((r.owner_pay_pct - 50.0).abs() < 1e-9);
        // Allocations: 200k × 5% = 10k profit, × 50% = 100k owner pay.
        assert!((r.profit_usd - 10_000.0).abs() < 1e-6);
        assert!((r.owner_pay_usd - 100_000.0).abs() < 1e-6);
    }

    #[test]
    fn band_thresholds_select_correctly() {
        assert_eq!(analyze(&inp(250_000.0, 0.0)).band, "$250K–$500K"); // boundary → next band
        assert_eq!(analyze(&inp(750_000.0, 0.0)).band, "$500K–$1M");
        assert_eq!(analyze(&inp(2_000_000.0, 0.0)).band, "$1M–$5M");
        assert_eq!(analyze(&inp(6_000_000.0, 0.0)).band, "$5M+");
    }

    #[test]
    fn band_1m_to_5m_percentages() {
        let r = analyze(&inp(2_000_000.0, 0.0));
        assert!((r.profit_pct - 20.0).abs() < 1e-9);
        assert!((r.owner_pay_pct - 10.0).abs() < 1e-9);
        assert!((r.tax_pct - 15.0).abs() < 1e-9);
        assert!((r.opex_pct - 55.0).abs() < 1e-9);
    }

    #[test]
    fn every_band_sums_to_100() {
        for rev in [100_000.0, 300_000.0, 750_000.0, 2_000_000.0, 9_000_000.0] {
            let (_l, p, o, t, e) = band_for(rev);
            assert!((p + o + t + e - 100.0).abs() < 1e-9, "band at {rev} must sum to 100");
        }
    }

    #[test]
    fn custom_percentages_override_when_summing_100() {
        let r = analyze(&ProfitFirstInput {
            annual_revenue_usd: 200_000.0,
            materials_subcontractors_usd: 0.0,
            profit_pct: 12.0,
            owner_pay_pct: 40.0,
            tax_pct: 18.0,
            opex_pct: 30.0,
        });
        assert_eq!(r.band, "Custom");
        assert!((r.profit_usd - 24_000.0).abs() < 1e-6); // 200k × 12%
    }

    #[test]
    fn custom_ignored_when_not_summing_100() {
        // Partial custom (sums to 40) → falls back to the band.
        let r = analyze(&ProfitFirstInput {
            annual_revenue_usd: 200_000.0,
            materials_subcontractors_usd: 0.0,
            profit_pct: 40.0,
            owner_pay_pct: 0.0,
            tax_pct: 0.0,
            opex_pct: 0.0,
        });
        assert_eq!(r.band, "Under $250K");
        assert!((r.profit_pct - 5.0).abs() < 1e-9);
    }

    #[test]
    fn materials_exceeding_revenue_clamps_to_zero() {
        let r = analyze(&inp(50_000.0, 80_000.0));
        assert!(r.real_revenue_usd.abs() < 1e-9);
        assert!(r.profit_usd.abs() < 1e-9);
    }
}
