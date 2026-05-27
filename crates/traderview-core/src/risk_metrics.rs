//! Risk-adjusted return metrics that aren't in stats.rs:
//!   * Calmar ratio = annualized return / |max drawdown|
//!   * Ulcer index  = RMS of drawdown depth over the window
//!   * MAR ratio    = like Calmar but uses CAGR over full lifetime
//!   * Pain index   = average drawdown depth (linear vs Ulcer's RMS)
//!
//! Pure compute. Input: equity series (one point per period, typically
//! daily). Output: scalar metrics + intermediate series for plotting.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
pub struct EquityPoint {
    pub equity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RiskMetrics {
    /// Maximum drawdown depth as a positive number (0..=1 fraction).
    pub max_dd_pct: f64,
    /// Calmar ratio = annualized return / max DD. Annualization factor
    /// `periods_per_year` (252 for daily).
    pub calmar: f64,
    /// Ulcer index = sqrt(mean(drawdown_pct^2)) × 100.
    pub ulcer_index: f64,
    /// Pain index = mean(|drawdown_pct|) × 100. Linear vs Ulcer's RMS.
    pub pain_index: f64,
    /// MAR ratio = CAGR / max DD (over full series).
    pub mar: f64,
    /// CAGR — (end/start)^(periods_per_year/n) - 1.
    pub cagr: f64,
}

pub fn compute(eq: &[EquityPoint], periods_per_year: f64) -> RiskMetrics {
    if eq.len() < 2 {
        return RiskMetrics::default();
    }
    // 1) Per-point drawdown vs running max.
    let mut peak = eq[0].equity;
    let mut dd_series = Vec::with_capacity(eq.len());
    let mut max_dd = 0.0_f64;
    for p in eq {
        if p.equity > peak {
            peak = p.equity;
        }
        let dd = if peak > 0.0 {
            (peak - p.equity) / peak
        } else {
            0.0
        };
        dd_series.push(dd);
        if dd > max_dd {
            max_dd = dd;
        }
    }
    // 2) CAGR + annualized return.
    let start = eq[0].equity.max(f64::EPSILON);
    let end = eq.last().unwrap().equity;
    let n = (eq.len() - 1) as f64; // periods elapsed
    let years = if n > 0.0 { n / periods_per_year } else { 1.0 };
    let cagr = if start > 0.0 && years > 0.0 && end > 0.0 {
        (end / start).powf(1.0 / years) - 1.0
    } else {
        0.0
    };
    // For Calmar we use annualized return — same as CAGR for a single
    // long series.
    let calmar = if max_dd > 0.0 {
        cagr / max_dd
    } else if cagr > 0.0 {
        f64::INFINITY
    } else {
        0.0
    };
    let mar = calmar; // single-series alias
                      // 3) Ulcer + pain.
    let n_dd = dd_series.len() as f64;
    let mean_sq: f64 = dd_series.iter().map(|d| d * d).sum::<f64>() / n_dd;
    let mean_abs: f64 = dd_series.iter().sum::<f64>() / n_dd; // dd already ≥ 0
    let ulcer_index = mean_sq.sqrt() * 100.0;
    let pain_index = mean_abs * 100.0;
    RiskMetrics {
        max_dd_pct: max_dd,
        calmar,
        mar,
        cagr,
        ulcer_index,
        pain_index,
    }
}

/// Risk-of-ruin: probability of losing your entire bankroll given a
/// per-trade edge (avg_R) and a stop multiple (1R-loss). Uses the
/// closed-form Kelly-based RoR formula:
///
///     RoR = ((1 - edge) / (1 + edge))^(units)
///
/// where `edge` is avg_R per trade (positive bullish) and `units` is
/// the bankroll expressed in R-multiples (i.e. how many full stops can
/// the account take before zero). Returns 0..=1 probability.
pub fn risk_of_ruin(avg_r: f64, bankroll_units: f64) -> f64 {
    if avg_r >= 1.0 {
        return 0.0;
    } // edge ≥ +1 → can't go to zero
    if avg_r <= -1.0 {
        return 1.0;
    } // edge ≤ -1 → certain ruin
    if avg_r == 0.0 {
        return 1.0;
    } // zero edge → ruin in infinite
    let ratio = (1.0 - avg_r) / (1.0 + avg_r);
    if ratio <= 0.0 {
        return 0.0;
    }
    ratio.powf(bankroll_units).clamp(0.0, 1.0)
}

/// Drawdown recovery analysis. For each peak-to-trough drawdown,
/// compute how many periods until the equity curve set a new high.
/// Returns a list of (start_index, trough_index, recovery_index_or_none).
pub fn recovery_periods(eq: &[EquityPoint]) -> Vec<RecoveryEvent> {
    if eq.len() < 2 {
        return vec![];
    }
    let mut out = Vec::new();
    let mut peak_value = eq[0].equity;
    let mut peak_idx = 0usize;
    let mut trough_value = eq[0].equity;
    let mut trough_idx = 0usize;
    let mut in_dd = false;
    for (i, p) in eq.iter().enumerate() {
        if p.equity > peak_value {
            // New peak — close out any in-progress dd as recovered.
            if in_dd {
                out.push(RecoveryEvent {
                    peak_index: peak_idx,
                    trough_index: trough_idx,
                    recovery_index: Some(i),
                });
                in_dd = false;
            }
            peak_value = p.equity;
            peak_idx = i;
            trough_value = p.equity;
            trough_idx = i;
        } else if p.equity < peak_value {
            // Track trough.
            if !in_dd {
                in_dd = true;
                trough_value = p.equity;
                trough_idx = i;
            }
            if p.equity < trough_value {
                trough_value = p.equity;
                trough_idx = i;
            }
        }
    }
    // If we end in a drawdown, push an open event.
    if in_dd {
        out.push(RecoveryEvent {
            peak_index: peak_idx,
            trough_index: trough_idx,
            recovery_index: None,
        });
    }
    out
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryEvent {
    pub peak_index: usize,
    pub trough_index: usize,
    /// None = drawdown still open at end of series.
    pub recovery_index: Option<usize>,
}

impl RecoveryEvent {
    pub fn recovery_periods(&self) -> Option<usize> {
        self.recovery_index.map(|r| r - self.peak_index)
    }
    pub fn drawdown_periods(&self) -> usize {
        self.trough_index - self.peak_index
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pts(values: &[f64]) -> Vec<EquityPoint> {
        values.iter().map(|&v| EquityPoint { equity: v }).collect()
    }

    // ─── compute ──────────────────────────────────────────────────────

    #[test]
    fn empty_series_zero_report() {
        let r = compute(&[], 252.0);
        assert_eq!(r.max_dd_pct, 0.0);
        assert_eq!(r.calmar, 0.0);
        assert_eq!(r.ulcer_index, 0.0);
    }

    #[test]
    fn monotonic_uptrend_zero_drawdown() {
        let r = compute(&pts(&[100.0, 101.0, 102.0, 103.0, 104.0]), 252.0);
        assert_eq!(r.max_dd_pct, 0.0);
        assert_eq!(r.ulcer_index, 0.0);
        // Calmar = +∞ when DD = 0 and CAGR > 0.
        assert!(r.calmar.is_infinite() && r.calmar > 0.0);
    }

    #[test]
    fn max_drawdown_finds_largest_peak_to_trough() {
        // 100 → 150 → 75 → 200 → 150. Peak 200, trough 75 (which preceded
        // 200), so max DD is from 150 (peak before 75) to 75 = 50%.
        let r = compute(&pts(&[100.0, 150.0, 75.0, 200.0, 150.0]), 252.0);
        assert!((r.max_dd_pct - 0.50).abs() < 1e-9);
    }

    #[test]
    fn cagr_uses_periods_per_year_for_annualization() {
        // 252 daily points doubling = 100% annual return.
        let mut series = vec![100.0];
        for _ in 0..252 {
            let last = *series.last().unwrap();
            series.push(last * (2.0_f64).powf(1.0 / 252.0));
        }
        let r = compute(&pts(&series), 252.0);
        // CAGR ~= 1.00 (= 100%).
        assert!((r.cagr - 1.00).abs() < 0.01);
    }

    #[test]
    fn ulcer_index_higher_than_pain_for_choppy_series() {
        // Same average DD, but Ulcer squares first → larger.
        let series = pts(&[100.0, 80.0, 100.0, 80.0, 100.0, 80.0]);
        let r = compute(&series, 252.0);
        assert!(
            r.ulcer_index >= r.pain_index,
            "Ulcer (RMS) must be ≥ Pain (linear avg) by Cauchy-Schwarz"
        );
    }

    // ─── risk_of_ruin ──────────────────────────────────────────────────

    #[test]
    fn ror_zero_edge_is_certain_ruin() {
        assert_eq!(risk_of_ruin(0.0, 10.0), 1.0);
    }

    #[test]
    fn ror_negative_edge_dominates_units() {
        // Sufficient negative edge → near-1 ruin regardless of bankroll.
        let p = risk_of_ruin(-0.5, 100.0);
        assert!(p > 0.99);
    }

    #[test]
    fn ror_positive_edge_decreases_with_more_bankroll() {
        let small_bank = risk_of_ruin(0.30, 5.0);
        let big_bank = risk_of_ruin(0.30, 50.0);
        assert!(
            big_bank < small_bank,
            "deeper bankroll must lower ruin probability"
        );
    }

    #[test]
    fn ror_edge_one_or_more_is_zero() {
        // edge >= 1 means avg trade is +1R or better — can't go to zero.
        assert_eq!(risk_of_ruin(1.0, 5.0), 0.0);
        assert_eq!(risk_of_ruin(2.0, 1.0), 0.0);
    }

    // ─── recovery_periods ──────────────────────────────────────────────

    #[test]
    fn series_without_drawdowns_returns_empty() {
        let r = recovery_periods(&pts(&[100.0, 101.0, 102.0]));
        assert!(r.is_empty());
    }

    #[test]
    fn single_drawdown_with_recovery_records_one_event() {
        // Peak at idx 1 (105), trough at idx 2 (95), recover at idx 3 (110).
        let r = recovery_periods(&pts(&[100.0, 105.0, 95.0, 110.0]));
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].peak_index, 1);
        assert_eq!(r[0].trough_index, 2);
        assert_eq!(r[0].recovery_index, Some(3));
        assert_eq!(r[0].recovery_periods(), Some(2));
        assert_eq!(r[0].drawdown_periods(), 1);
    }

    #[test]
    fn open_drawdown_at_end_records_with_none_recovery() {
        // Peak at idx 0, drops thereafter, never recovers.
        let r = recovery_periods(&pts(&[100.0, 95.0, 90.0, 85.0]));
        assert_eq!(r.len(), 1);
        assert!(r[0].recovery_index.is_none());
        assert_eq!(r[0].recovery_periods(), None);
    }

    #[test]
    fn multiple_drawdowns_each_recorded_separately() {
        // 100 → 110 (peak1) → 100 → 120 (recover + peak2) → 90.
        let r = recovery_periods(&pts(&[100.0, 110.0, 100.0, 120.0, 90.0]));
        assert_eq!(r.len(), 2);
        assert!(r[0].recovery_index.is_some());
        assert!(r[1].recovery_index.is_none(), "second dd still open");
    }
}
