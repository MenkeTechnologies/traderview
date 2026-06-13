//! Expense ratio drag — the long-run cost of fund fees.
//!
//! A fund's expense ratio looks tiny (0.5%, 1%) but it's charged on assets
//! every year, and the fee dollars never compound — so over decades the gap
//! versus a zero-fee fund is far larger than the headline percentage. This
//! projects the same contributions at the gross return vs the net return
//! (gross − expense ratio) and reports the dollars lost to fees.
//!
//!   * net return = gross return − expense ratio
//!   * FV = initial × (1+r)^n + contributions × ((1+r)^n − 1) / r
//!   * fee drag = FV(gross) − FV(net)
//!
//! Pure compute (end-of-year contributions).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct ExpenseDragInput {
    pub initial_investment_usd: f64,
    #[serde(default)]
    pub annual_contribution_usd: f64,
    pub years: f64,
    pub gross_return_pct: f64,
    pub expense_ratio_pct: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExpenseDragResult {
    pub net_return_pct: f64,
    /// Ending value with no fees (the gross return).
    pub gross_ending_usd: f64,
    /// Ending value after the expense ratio (the net return).
    pub net_ending_usd: f64,
    /// Dollars lost to fees over the horizon (gross − net ending).
    pub fee_drag_usd: f64,
    /// Fee drag as a percent of the no-fee ending value.
    pub fee_drag_pct: f64,
}

/// FV of an initial sum plus end-of-year contributions at annual rate `r`.
fn future_value(initial: f64, contribution: f64, r: f64, n: f64) -> f64 {
    let growth = (1.0 + r).powf(n);
    let from_initial = initial * growth;
    let from_contrib = if r.abs() < 1e-12 {
        contribution * n
    } else {
        contribution * (growth - 1.0) / r
    };
    from_initial + from_contrib
}

pub fn analyze(i: &ExpenseDragInput) -> ExpenseDragResult {
    let gross = i.gross_return_pct / 100.0;
    let net = (i.gross_return_pct - i.expense_ratio_pct) / 100.0;
    let n = i.years.max(0.0);

    let gross_end = future_value(i.initial_investment_usd, i.annual_contribution_usd, gross, n);
    let net_end = future_value(i.initial_investment_usd, i.annual_contribution_usd, net, n);
    let drag = gross_end - net_end;

    ExpenseDragResult {
        net_return_pct: i.gross_return_pct - i.expense_ratio_pct,
        gross_ending_usd: gross_end,
        net_ending_usd: net_end,
        fee_drag_usd: drag,
        fee_drag_pct: if gross_end > 0.0 { drag / gross_end * 100.0 } else { 0.0 },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> ExpenseDragInput {
        ExpenseDragInput {
            initial_investment_usd: 100_000.0,
            annual_contribution_usd: 0.0,
            years: 30.0,
            gross_return_pct: 7.0,
            expense_ratio_pct: 1.0,
        }
    }

    #[test]
    fn net_return_is_gross_minus_er() {
        let r = analyze(&base());
        assert!((r.net_return_pct - 6.0).abs() < 1e-9);
    }

    #[test]
    fn gross_ending_lump_only() {
        // 100k × 1.07^30.
        let r = analyze(&base());
        assert!((r.gross_ending_usd - 100_000.0 * 1.07_f64.powi(30)).abs() < 1e-3);
    }

    #[test]
    fn net_ending_lower_than_gross() {
        let r = analyze(&base());
        assert!(r.net_ending_usd < r.gross_ending_usd);
    }

    #[test]
    fn fee_drag_is_the_gap() {
        let r = analyze(&base());
        assert!((r.fee_drag_usd - (r.gross_ending_usd - r.net_ending_usd)).abs() < 1e-6);
        // 1% ER over 30y on a 7% portfolio erodes a large chunk — well over 20%.
        assert!(r.fee_drag_pct > 20.0);
    }

    #[test]
    fn higher_er_more_drag() {
        let low = analyze(&ExpenseDragInput { expense_ratio_pct: 0.1, ..base() });
        let high = analyze(&ExpenseDragInput { expense_ratio_pct: 1.0, ..base() });
        assert!(high.fee_drag_usd > low.fee_drag_usd);
    }

    #[test]
    fn zero_er_no_drag() {
        let r = analyze(&ExpenseDragInput { expense_ratio_pct: 0.0, ..base() });
        assert!(r.fee_drag_usd.abs() < 1e-6);
        assert!((r.net_ending_usd - r.gross_ending_usd).abs() < 1e-6);
    }

    #[test]
    fn contributions_increase_both_endings() {
        let no_contrib = analyze(&base());
        let with_contrib = analyze(&ExpenseDragInput { annual_contribution_usd: 6_000.0, ..base() });
        assert!(with_contrib.gross_ending_usd > no_contrib.gross_ending_usd);
        assert!(with_contrib.fee_drag_usd > no_contrib.fee_drag_usd);
    }

    #[test]
    fn longer_horizon_more_drag() {
        let short = analyze(&ExpenseDragInput { years: 10.0, ..base() });
        let long = analyze(&ExpenseDragInput { years: 40.0, ..base() });
        assert!(long.fee_drag_pct > short.fee_drag_pct);
    }
}
