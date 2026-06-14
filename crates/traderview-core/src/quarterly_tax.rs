//! Quarterly estimated-tax math for a Schedule C / trader filer. Given the
//! year's projected total income (annualized upstream) and the year-to-date
//! trading P&L, computes self-employment tax (on 92.35% of net, SS capped),
//! federal income tax (2024 single brackets, standard deduction by filing
//! status, less ½ SE), the projected total, the effective rate, and the
//! safe-harbor floor (lesser of 90% of the projection and the prior-year
//! liability — 110% of it when income exceeds $150k). Splits the projection
//! into four equal quarters and flags each as paid / partial / unpaid. Date
//! handling (annualization, due dates, next quarter) stays in the view; this is
//! the deterministic tax core. Faithful port of the former client-side
//! calculator. Pure compute, not advice.

use serde::{Deserialize, Serialize};

const SS_BASE_2024: f64 = 168_600.0;
const SS_RATE: f64 = 0.124;
const MEDICARE_RATE: f64 = 0.029;
const SE_DEDUCTION: f64 = 0.9235;

/// 2024 single-filer brackets (cap, rate). The original applies these
/// regardless of filing status; only the standard deduction varies.
const FED_BRACKETS_2024_SINGLE: [(f64, f64); 7] = [
    (11_600.0, 0.10),
    (47_150.0, 0.12),
    (100_525.0, 0.22),
    (191_950.0, 0.24),
    (243_725.0, 0.32),
    (609_350.0, 0.35),
    (f64::INFINITY, 0.37),
];

#[derive(Debug, Clone, Deserialize)]
pub struct QuarterlyTaxInput {
    /// Year-to-date trading P&L (SE tax is charged on this raw figure).
    pub ytd_trading_pnl_usd: f64,
    /// Projected annual total income (annualized trading + other), computed in
    /// the view where the calendar date lives.
    pub total_income_usd: f64,
    pub prior_year_tax_usd: f64,
    pub filing_status: String,
    /// Amounts already paid for Q1..Q4.
    pub quarterly_paid_usd: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct QuarterStatus {
    /// "ok", "partial", or "unpaid".
    pub status_key: String,
    /// Shortfall vs the per-quarter target (0 once met).
    pub short_usd: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct QuarterlyTaxReport {
    pub se_tax_usd: f64,
    pub half_se_usd: f64,
    pub taxable_income_usd: f64,
    pub fed_income_tax_usd: f64,
    pub projected_total_usd: f64,
    pub effective_rate_pct: f64,
    pub safe_harbor_floor_usd: f64,
    pub per_quarter_usd: f64,
    pub total_paid_usd: f64,
    pub remaining_usd: f64,
    pub quarters: Vec<QuarterStatus>,
    pub valid: bool,
}

fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn round4(x: f64) -> f64 {
    (x * 10_000.0).round() / 10_000.0
}

fn self_employment_tax(net: f64) -> f64 {
    if net <= 0.0 {
        return 0.0;
    }
    let se_base = net * SE_DEDUCTION;
    se_base.min(SS_BASE_2024) * SS_RATE + se_base * MEDICARE_RATE
}

fn federal_tax(taxable: f64) -> f64 {
    let mut owe = 0.0;
    let mut last_cap = 0.0;
    for &(cap, rate) in FED_BRACKETS_2024_SINGLE.iter() {
        let slice = (taxable.min(cap) - last_cap).max(0.0);
        owe += slice * rate;
        if taxable <= cap {
            break;
        }
        last_cap = cap;
    }
    owe
}

fn standard_deduction(status: &str) -> f64 {
    match status {
        "mfj" => 29_200.0,
        "hoh" => 21_900.0,
        _ => 14_600.0,
    }
}

pub fn generate(i: &QuarterlyTaxInput) -> QuarterlyTaxReport {
    let se_tax = self_employment_tax(i.ytd_trading_pnl_usd);
    let half_se = se_tax / 2.0;
    let taxable_income = (i.total_income_usd - half_se - standard_deduction(&i.filing_status)).max(0.0);
    let fed_income_tax = federal_tax(taxable_income);
    let projected_total = fed_income_tax + se_tax;

    let safe_harbor_base = if i.total_income_usd > 150_000.0 {
        i.prior_year_tax_usd * 1.10
    } else {
        i.prior_year_tax_usd
    }
    .max(0.0);
    let safe_harbor_floor = (projected_total * 0.90).min(safe_harbor_base);

    let per_quarter = projected_total / 4.0;
    let total_paid: f64 = i.quarterly_paid_usd.iter().sum();
    let remaining = (projected_total - total_paid).max(0.0);

    let quarters = i
        .quarterly_paid_usd
        .iter()
        .map(|&paid| {
            let status_key = if paid >= per_quarter {
                "ok"
            } else if paid > 0.0 {
                "partial"
            } else {
                "unpaid"
            };
            QuarterStatus {
                status_key: status_key.to_string(),
                short_usd: round2((per_quarter - paid).max(0.0)),
            }
        })
        .collect();

    QuarterlyTaxReport {
        se_tax_usd: round2(se_tax),
        half_se_usd: round2(half_se),
        taxable_income_usd: round2(taxable_income),
        fed_income_tax_usd: round2(fed_income_tax),
        projected_total_usd: round2(projected_total),
        effective_rate_pct: round4(if i.total_income_usd > 0.0 {
            projected_total / i.total_income_usd * 100.0
        } else {
            0.0
        }),
        safe_harbor_floor_usd: round2(safe_harbor_floor),
        per_quarter_usd: round2(per_quarter),
        total_paid_usd: round2(total_paid),
        remaining_usd: round2(remaining),
        quarters,
        valid: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    fn base() -> QuarterlyTaxInput {
        QuarterlyTaxInput {
            ytd_trading_pnl_usd: 100_000.0,
            total_income_usd: 120_000.0,
            prior_year_tax_usd: 20_000.0,
            filing_status: "single".into(),
            quarterly_paid_usd: vec![5_000.0, 5_000.0, 0.0, 0.0],
        }
    }

    // Pins cross-checked against the JS compute() in Python.
    #[test]
    fn default_projection() {
        let d = generate(&base());
        assert!(d.valid);
        assert!(close(d.se_tax_usd, 14_129.55));
        assert!(close(d.half_se_usd, 7_064.775));
        assert!(close(d.taxable_income_usd, 98_335.225));
        assert!(close(d.fed_income_tax_usd, 16_686.7495));
        assert!(close(d.projected_total_usd, 30_816.2995));
        assert!(close(d.safe_harbor_floor_usd, 20_000.0));
        assert!(close(d.per_quarter_usd, 7_704.0749));
        assert!(close(d.total_paid_usd, 10_000.0));
        assert!(close(d.remaining_usd, 20_816.2995));
        assert!(close(d.effective_rate_pct, 25.6802));
        assert_eq!(d.quarters.len(), 4);
        assert_eq!(d.quarters[0].status_key, "partial");
        assert!(close(d.quarters[0].short_usd, 2_704.0749));
        assert_eq!(d.quarters[2].status_key, "unpaid");
    }

    #[test]
    fn high_income_uses_110pct_safe_harbor() {
        let d = generate(&QuarterlyTaxInput {
            total_income_usd: 200_000.0, prior_year_tax_usd: 30_000.0, quarterly_paid_usd: vec![0.0; 4], ..base()
        });
        // 200k > 150k → prior × 1.10 = 33000, and 90% of projection exceeds it.
        assert!(close(d.safe_harbor_floor_usd, 33_000.0));
    }

    #[test]
    fn ok_quarter_when_paid_meets_target() {
        let d = generate(&QuarterlyTaxInput { quarterly_paid_usd: vec![50_000.0, 0.0, 0.0, 0.0], ..base() });
        assert_eq!(d.quarters[0].status_key, "ok");
        assert!(close(d.quarters[0].short_usd, 0.0));
    }

    #[test]
    fn se_tax_zero_on_loss() {
        let d = generate(&QuarterlyTaxInput { ytd_trading_pnl_usd: -5_000.0, ..base() });
        assert!(close(d.se_tax_usd, 0.0));
    }
}
