//! Savings rate calculator + benchmark + FI-projection.
//!
//! Unlike `financial_ratios::savings_rate` (which is one cell in a
//! scorecard), this view focuses on the savings rate itself and the
//! downstream consequence — Mr Money Mustache's shockingly-simple math
//! of early retirement: at SR=10% you need ~51 years to retire, at
//! 25% ~32 years, at 50% ~17 years, at 75% ~7 years (assumes 5% real
//! return and 4% SWR).
//!
//! Inputs:
//!   - gross_annual_income_usd, net_annual_income_usd
//!     (we report both gross and net SR — Boglehead vs ChooseFI camps
//!     disagree on which denominator to use; show both)
//!   - annual_expenses_usd, annual_savings_usd
//!   - expected_real_return_pct (default 5.0)
//!   - safe_withdrawal_rate_pct (default 4.0)
//!
//! Outputs:
//!   - gross_savings_rate_pct, net_savings_rate_pct
//!   - benchmark = "elite ≥ 50% | excellent ≥ 30% | good ≥ 20% |
//!     ok ≥ 10% | poor < 10%" applied to gross rate
//!   - years_to_fi = MMM formula
//!   - fi_number_usd = annual_expenses / SWR (target portfolio)
//!   - projection table — years_to_fi at SR ∈ {10, 20, 30, 40, 50, 60, 70}
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct SavingsRateInput {
    pub gross_annual_income_usd: f64,
    pub net_annual_income_usd: f64,
    pub annual_expenses_usd: f64,
    pub annual_savings_usd: f64,
    #[serde(default = "default_return")]
    pub expected_real_return_pct: f64,
    #[serde(default = "default_swr")]
    pub safe_withdrawal_rate_pct: f64,
}

fn default_return() -> f64 { 5.0 }
fn default_swr() -> f64 { 4.0 }

#[derive(Debug, Clone, Serialize)]
pub struct ProjectionCell {
    pub savings_rate_pct: f64,
    pub years_to_fi: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SavingsRateReport {
    pub gross_savings_rate_pct: f64,
    pub net_savings_rate_pct: f64,
    pub benchmark: String,
    pub years_to_fi: f64,
    pub fi_number_usd: f64,
    pub projection: Vec<ProjectionCell>,
}

// ─── Pure compute ─────────────────────────────────────────────────────────

pub fn rate_of(savings: f64, income: f64) -> f64 {
    if income <= 0.0 {
        return 0.0;
    }
    savings / income * 100.0
}

pub fn benchmark_label(gross_rate_pct: f64) -> String {
    if gross_rate_pct >= 50.0 { "elite" }
    else if gross_rate_pct >= 30.0 { "excellent" }
    else if gross_rate_pct >= 20.0 { "good" }
    else if gross_rate_pct >= 10.0 { "ok" }
    else { "poor" }
    .to_string()
}

/// MMM-style: years to FI given SR + expected return rate.
/// Derivation: at SR `s`, you save `s` per year and spend `1-s`.
/// FI number = (1-s) / SWR portfolios-of-expenses. Starting at 0,
/// growing at real return `r` per year with annual `s` contribution,
/// the future value equals `(1-s)/SWR` when:
///     s × ((1+r)^t − 1) / r = (1−s) / SWR
/// Solving for t:
///     t = ln(1 + (1−s)/SWR × r/s) / ln(1+r)
/// Returns None/0 cases gracefully.
pub fn years_to_fi(savings_rate_pct: f64, real_return_pct: f64, swr_pct: f64) -> f64 {
    if savings_rate_pct <= 0.0 {
        return f64::INFINITY;
    }
    if savings_rate_pct >= 100.0 {
        // Already saving everything → live off nothing → FI immediately.
        return 0.0;
    }
    let s = savings_rate_pct / 100.0;
    let r = real_return_pct / 100.0;
    let swr = swr_pct / 100.0;
    if r <= 0.0 {
        // No return: linear. Years = (1-s)/(SWR × s).
        return (1.0 - s) / (swr * s);
    }
    let target_multiple = (1.0 - s) / swr;       // portfolio = N × annual contribution
    let factor = 1.0 + target_multiple * r / s;
    if factor <= 0.0 {
        return f64::INFINITY;
    }
    factor.ln() / (1.0 + r).ln()
}

pub fn fi_number(annual_expenses: f64, swr_pct: f64) -> f64 {
    if swr_pct <= 0.0 {
        return 0.0;
    }
    annual_expenses / (swr_pct / 100.0)
}

pub fn compute(input: &SavingsRateInput) -> SavingsRateReport {
    let gross = rate_of(input.annual_savings_usd, input.gross_annual_income_usd);
    let net = rate_of(input.annual_savings_usd, input.net_annual_income_usd);
    let bench = benchmark_label(gross);
    let years = years_to_fi(gross, input.expected_real_return_pct, input.safe_withdrawal_rate_pct);
    let fi = fi_number(input.annual_expenses_usd, input.safe_withdrawal_rate_pct);
    let projection: Vec<ProjectionCell> = [10.0_f64, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0]
        .iter()
        .map(|sr| ProjectionCell {
            savings_rate_pct: *sr,
            years_to_fi: years_to_fi(*sr, input.expected_real_return_pct, input.safe_withdrawal_rate_pct),
        })
        .collect();
    SavingsRateReport {
        gross_savings_rate_pct: gross,
        net_savings_rate_pct: net,
        benchmark: bench,
        years_to_fi: years,
        fi_number_usd: fi,
        projection,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rate_of_zero_income() {
        assert_eq!(rate_of(1000.0, 0.0), 0.0);
    }

    #[test]
    fn rate_of_25_pct() {
        assert_eq!(rate_of(25000.0, 100_000.0), 25.0);
    }

    #[test]
    fn benchmark_labels() {
        assert_eq!(benchmark_label(60.0), "elite");
        assert_eq!(benchmark_label(35.0), "excellent");
        assert_eq!(benchmark_label(22.0), "good");
        assert_eq!(benchmark_label(15.0), "ok");
        assert_eq!(benchmark_label(5.0),  "poor");
    }

    #[test]
    fn years_to_fi_zero_sr_is_inf() {
        assert!(years_to_fi(0.0, 5.0, 4.0).is_infinite());
    }

    #[test]
    fn years_to_fi_100_sr_is_zero() {
        assert_eq!(years_to_fi(100.0, 5.0, 4.0), 0.0);
    }

    #[test]
    fn years_to_fi_mmm_50_pct() {
        // MMM published table: 50% SR @ 5% real return + 4% SWR ≈ 17 years.
        let y = years_to_fi(50.0, 5.0, 4.0);
        assert!(y > 16.0 && y < 18.0, "expected ~17, got {y}");
    }

    #[test]
    fn years_to_fi_mmm_25_pct() {
        // ~32 years per MMM.
        let y = years_to_fi(25.0, 5.0, 4.0);
        assert!(y > 30.0 && y < 34.0, "expected ~32, got {y}");
    }

    #[test]
    fn years_to_fi_zero_return_linear() {
        // SR=50%, 0% return, 4% SWR: 25× expenses = 25 / 1 = 25 years saving
        // 50% of income covers expenses, so target = expenses × 25. Time =
        // (1-0.5)/(0.04 × 0.5) = 0.5/0.02 = 25 years. Confirmed.
        let y = years_to_fi(50.0, 0.0, 4.0);
        assert!((y - 25.0).abs() < 1e-6);
    }

    #[test]
    fn fi_number_4pct_of_40k_is_million() {
        assert!((fi_number(40_000.0, 4.0) - 1_000_000.0).abs() < 1e-6);
    }

    #[test]
    fn fi_number_zero_swr_is_zero() {
        assert_eq!(fi_number(40_000.0, 0.0), 0.0);
    }

    #[test]
    fn compute_full_report() {
        let r = compute(&SavingsRateInput {
            gross_annual_income_usd: 100_000.0,
            net_annual_income_usd: 75_000.0,
            annual_expenses_usd: 50_000.0,
            annual_savings_usd: 25_000.0,
            expected_real_return_pct: 5.0,
            safe_withdrawal_rate_pct: 4.0,
        });
        assert!((r.gross_savings_rate_pct - 25.0).abs() < 1e-6);
        assert!((r.net_savings_rate_pct - 25_000.0 / 75_000.0 * 100.0).abs() < 1e-6);
        assert_eq!(r.benchmark, "good");
        assert!((r.fi_number_usd - 1_250_000.0).abs() < 1e-6);
        assert!(r.years_to_fi > 30.0 && r.years_to_fi < 34.0);
        assert_eq!(r.projection.len(), 7);
    }

    #[test]
    fn compute_zero_income_safe() {
        let r = compute(&SavingsRateInput {
            gross_annual_income_usd: 0.0,
            net_annual_income_usd: 0.0,
            annual_expenses_usd: 30_000.0,
            annual_savings_usd: 0.0,
            expected_real_return_pct: 5.0,
            safe_withdrawal_rate_pct: 4.0,
        });
        assert_eq!(r.gross_savings_rate_pct, 0.0);
        assert!(r.years_to_fi.is_infinite());
    }

    #[test]
    fn compute_projection_monotonic_decreasing() {
        // Higher savings rate → fewer years.
        let r = compute(&SavingsRateInput {
            gross_annual_income_usd: 100_000.0,
            net_annual_income_usd: 75_000.0,
            annual_expenses_usd: 50_000.0,
            annual_savings_usd: 25_000.0,
            expected_real_return_pct: 5.0,
            safe_withdrawal_rate_pct: 4.0,
        });
        for w in r.projection.windows(2) {
            assert!(w[0].years_to_fi >= w[1].years_to_fi,
                "expected {} ≥ {} at SR {} → {}",
                w[0].years_to_fi, w[1].years_to_fi,
                w[0].savings_rate_pct, w[1].savings_rate_pct);
        }
    }
}
