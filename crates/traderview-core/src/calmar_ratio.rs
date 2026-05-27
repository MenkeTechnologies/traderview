//! Calmar ratio — annualized return / max drawdown.
//!
//! Terry Young's risk-adjusted performance metric for managed futures.
//! Distinct from Sharpe (which uses stdev) and Sortino (downside
//! deviation): Calmar's denominator is the worst-case loss, which
//! matches a trader's actual capital risk tolerance.
//!
//! Formula: `Calmar = annualized_return_pct / max_drawdown_pct`.
//! By convention max_drawdown_pct is positive (e.g. a 20% drawdown is 20.0).
//! Returns 0 when max_drawdown_pct is 0 (no drawdowns yet — undefined).
//!
//! Calmar > 1.0 is good, > 3.0 is excellent for a sustained period.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CalmarReport {
    pub annualized_return_pct: f64,
    pub max_drawdown_pct: f64,
    /// `annualized_return / max_dd` — 0 if denominator is 0.
    pub calmar_ratio: f64,
    pub note: String,
}

pub fn compute(equity: &[f64], years: f64) -> CalmarReport {
    let n = equity.len();
    if n < 2 || years <= 0.0 || equity[0] <= 0.0 {
        return CalmarReport {
            note: "need ≥ 2 equity points, positive years, positive starting equity".into(),
            ..Default::default()
        };
    }
    let total_return = equity[n - 1] / equity[0];
    let annualized = if total_return > 0.0 {
        (total_return.powf(1.0 / years) - 1.0) * 100.0
    } else {
        // Negative ending equity — can't take fractional powers.
        -100.0
    };
    let mut peak = equity[0];
    let mut max_dd_pct = 0.0_f64;
    for &v in equity {
        if v > peak { peak = v; }
        if peak > 0.0 {
            let dd_pct = (peak - v) / peak * 100.0;
            if dd_pct > max_dd_pct { max_dd_pct = dd_pct; }
        }
    }
    let calmar = if max_dd_pct > 0.0 { annualized / max_dd_pct } else { 0.0 };
    let note = if max_dd_pct == 0.0 {
        "no drawdown seen — Calmar undefined, set to 0".into()
    } else {
        format!("annualized {annualized:.2}% / DD {max_dd_pct:.2}% = {calmar:.2}")
    };
    CalmarReport {
        annualized_return_pct: annualized,
        max_drawdown_pct: max_dd_pct,
        calmar_ratio: calmar, note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn too_few_points_returns_default() {
        let r = compute(&[100.0], 1.0);
        assert_eq!(r.calmar_ratio, 0.0);
        assert!(r.note.contains("need"));
    }

    #[test]
    fn zero_or_negative_years_returns_default() {
        let r = compute(&[100.0, 110.0], 0.0);
        assert_eq!(r.calmar_ratio, 0.0);
        let r = compute(&[100.0, 110.0], -1.0);
        assert_eq!(r.calmar_ratio, 0.0);
    }

    #[test]
    fn monotonic_curve_has_zero_dd_and_zero_calmar() {
        let r = compute(&[100.0, 110.0, 120.0], 1.0);
        assert_eq!(r.max_drawdown_pct, 0.0);
        assert_eq!(r.calmar_ratio, 0.0);
        assert!(r.note.contains("undefined"));
        assert!(r.annualized_return_pct > 0.0);
    }

    #[test]
    fn classic_20pct_return_10pct_dd_gives_calmar_2() {
        // 20% return / 10% DD = 2.0 calmar.
        // equity goes 100 → 90 (10% DD) → 120 (20% return).
        let r = compute(&[100.0, 90.0, 120.0], 1.0);
        assert!((r.max_drawdown_pct - 10.0).abs() < 1e-9);
        assert!((r.annualized_return_pct - 20.0).abs() < 1e-9);
        assert!((r.calmar_ratio - 2.0).abs() < 1e-9);
    }

    #[test]
    fn multi_year_annualizes_correctly() {
        // 100 → 144 over 2 years = (1.44)^0.5 - 1 = 20% annualized.
        let r = compute(&[100.0, 130.0, 144.0], 2.0);
        assert!((r.annualized_return_pct - 20.0).abs() < 0.01);
    }

    #[test]
    fn losing_strategy_returns_negative_calmar() {
        // 100 → 50 → 80. Annualized return < 0, max DD 50%.
        let r = compute(&[100.0, 50.0, 80.0], 1.0);
        assert!(r.calmar_ratio < 0.0,
            "losing strategy with drawdown should have negative Calmar, got {}", r.calmar_ratio);
        assert!((r.max_drawdown_pct - 50.0).abs() < 1e-9);
    }

    #[test]
    fn negative_starting_equity_returns_default() {
        let r = compute(&[-100.0, 100.0], 1.0);
        assert_eq!(r.calmar_ratio, 0.0);
        assert!(r.note.contains("positive starting equity"));
    }
}
