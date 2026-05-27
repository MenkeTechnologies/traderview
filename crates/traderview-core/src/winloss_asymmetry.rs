//! Win/loss size-asymmetry detector.
//!
//! For a profitable system, winners should be LARGER than losers on
//! average (avg_win / avg_loss > 1.0), OR the win rate should be much
//! higher than 50% if the ratio is closer to 1.
//!
//! Computes:
//!   - avg_win, avg_loss (absolute), payoff_ratio = avg_win / avg_loss
//!   - win_rate
//!   - expectancy = win_rate × avg_win - (1 - win_rate) × avg_loss
//!   - "break-even win rate" needed to survive at current payoff_ratio:
//!     bewr = 1 / (1 + payoff_ratio)
//!
//! Flags systems that are losing the "let winners run, cut losers"
//! battle. Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AsymmetryReport {
    pub trade_count: usize,
    pub win_count: usize,
    pub loss_count: usize,
    pub win_rate: f64,
    pub avg_win: f64,
    pub avg_loss: f64,
    /// avg_win / avg_loss. None when no losers.
    pub payoff_ratio: Option<f64>,
    pub expectancy_per_trade: f64,
    /// Win rate needed to break even given current payoff ratio.
    pub break_even_win_rate: Option<f64>,
    /// True if winners are systematically smaller than losers
    /// (payoff < 1) AND win rate isn't high enough to compensate.
    pub asymmetry_warning: bool,
}

pub fn analyze(pnls: &[f64]) -> AsymmetryReport {
    let mut r = AsymmetryReport::default();
    if pnls.is_empty() {
        return r;
    }
    let wins: Vec<f64> = pnls.iter().filter(|p| **p > 0.0).cloned().collect();
    let losses: Vec<f64> = pnls.iter().filter(|p| **p < 0.0).map(|p| -p).collect();
    r.trade_count = pnls.len();
    r.win_count = wins.len();
    r.loss_count = losses.len();
    r.win_rate = wins.len() as f64 / pnls.len() as f64;
    r.avg_win = if wins.is_empty() {
        0.0
    } else {
        wins.iter().sum::<f64>() / wins.len() as f64
    };
    r.avg_loss = if losses.is_empty() {
        0.0
    } else {
        losses.iter().sum::<f64>() / losses.len() as f64
    };
    r.payoff_ratio = if r.avg_loss > 0.0 {
        Some(r.avg_win / r.avg_loss)
    } else {
        None
    };
    r.expectancy_per_trade = r.win_rate * r.avg_win - (1.0 - r.win_rate) * r.avg_loss;
    r.break_even_win_rate = r.payoff_ratio.map(|p| 1.0 / (1.0 + p));
    r.asymmetry_warning = if let (Some(p), Some(bewr)) = (r.payoff_ratio, r.break_even_win_rate) {
        p < 1.0 && r.win_rate < bewr
    } else {
        false
    };
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_default() {
        let r = analyze(&[]);
        assert_eq!(r.trade_count, 0);
    }

    #[test]
    fn all_winners_no_payoff_ratio() {
        let r = analyze(&[100.0, 100.0, 100.0]);
        assert_eq!(r.win_rate, 1.0);
        assert_eq!(r.avg_win, 100.0);
        assert_eq!(r.avg_loss, 0.0);
        assert!(r.payoff_ratio.is_none());
    }

    #[test]
    fn break_even_at_50pct_with_1to1_payoff() {
        let r = analyze(&[100.0, -100.0]);
        assert_eq!(r.payoff_ratio, Some(1.0));
        // bewr = 1 / (1+1) = 0.5.
        assert_eq!(r.break_even_win_rate, Some(0.5));
        // Expectancy = 0.5 × 100 - 0.5 × 100 = 0.
        assert_eq!(r.expectancy_per_trade, 0.0);
    }

    #[test]
    fn positive_expectancy_above_break_even_winrate() {
        // 60% wr × $100 win - 40% × $100 loss = $20 expectancy.
        let r = analyze(&[100.0, 100.0, 100.0, -100.0, -100.0]);
        assert!((r.expectancy_per_trade - 20.0).abs() < 1e-9);
    }

    #[test]
    fn high_payoff_low_winrate_can_still_be_profitable() {
        // 30% wr × $300 win - 70% × $100 loss = $20 expectancy.
        let mut trades = vec![300.0; 3];
        trades.extend(vec![-100.0; 7]);
        let r = analyze(&trades);
        assert!(r.expectancy_per_trade > 0.0);
        // bewr = 1/(1+3) = 0.25 → 30% > 25% ✓
        assert!(r.win_rate > r.break_even_win_rate.unwrap());
    }

    #[test]
    fn asymmetry_warning_fires_when_winners_too_small_and_winrate_insufficient() {
        // Avg win $50, avg loss $100. payoff = 0.5. bewr = 1/1.5 = 0.667.
        // Win rate 50% < 66.7% → warning.
        let r = analyze(&[50.0, 50.0, 50.0, -100.0, -100.0, -100.0]);
        assert!(r.asymmetry_warning);
    }

    #[test]
    fn no_warning_when_winrate_high_enough_to_compensate_small_winners() {
        // 80% wr × $50 - 20% × $100 = $20. Profitable despite small payoff.
        let mut trades = vec![50.0; 8];
        trades.extend(vec![-100.0; 2]);
        let r = analyze(&trades);
        assert!(r.expectancy_per_trade > 0.0);
        assert!(!r.asymmetry_warning);
    }

    #[test]
    fn zero_pnl_trades_ignored() {
        // Breakeven trades neither win nor loss.
        let r = analyze(&[0.0, 0.0, 100.0, -100.0]);
        assert_eq!(r.win_count, 1);
        assert_eq!(r.loss_count, 1);
        assert_eq!(r.trade_count, 4);
        // wr = 1/4 = 0.25, even though most trades aren't losses.
        assert_eq!(r.win_rate, 0.25);
    }
}
