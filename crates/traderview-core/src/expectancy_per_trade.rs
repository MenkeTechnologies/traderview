//! Expectancy Per Trade — average profit per trade weighted by
//! win/loss probabilities.
//!
//!   E[trade] = win_rate · avg_win + (1 − win_rate) · avg_loss
//!
//! Where avg_loss is signed (i.e. negative for losing trades). Reports
//! both:
//!   - Dollar expectancy (in same units as input)
//!   - R-multiple expectancy (avg_win / |avg_loss| weighted)
//!
//! Companion stats:
//!   - **win_rate** = wins / total_trades
//!   - **avg_win** = mean of positive-P&L trades
//!   - **avg_loss** = mean of negative-P&L trades (negative number)
//!   - **payoff_ratio** = avg_win / |avg_loss|
//!   - **profit_factor** = sum_wins / |sum_losses|
//!
//! Pure compute. Companion to `profit_factor`, `gain_to_pain_ratio`,
//! `risk_adjusted_ratios`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExpectancyReport {
    pub expectancy_per_trade: f64,
    pub r_multiple_expectancy: f64,
    pub win_rate: f64,
    pub avg_win: f64,
    pub avg_loss: f64,
    pub payoff_ratio: f64,
    pub profit_factor: f64,
    pub n_trades: usize,
    pub n_wins: usize,
    pub n_losses: usize,
}

pub fn compute(trade_pnls: &[f64]) -> Option<ExpectancyReport> {
    if trade_pnls.is_empty() { return None; }
    if trade_pnls.iter().any(|x| !x.is_finite()) { return None; }
    let n = trade_pnls.len();
    let wins: Vec<f64> = trade_pnls.iter().copied().filter(|x| *x > 0.0).collect();
    let losses: Vec<f64> = trade_pnls.iter().copied().filter(|x| *x < 0.0).collect();
    let n_wins = wins.len();
    let n_losses = losses.len();
    let n_f = n as f64;
    let win_rate = n_wins as f64 / n_f;
    let avg_win = if n_wins > 0 { wins.iter().sum::<f64>() / n_wins as f64 } else { 0.0 };
    let avg_loss = if n_losses > 0 { losses.iter().sum::<f64>() / n_losses as f64 } else { 0.0 };
    let expectancy = win_rate * avg_win + (1.0 - win_rate) * avg_loss;
    let payoff = if avg_loss.abs() > 0.0 { avg_win / avg_loss.abs() } else { f64::INFINITY };
    let r_mult = if payoff.is_finite() {
        win_rate * payoff - (1.0 - win_rate)
    } else { f64::INFINITY };
    let sum_wins: f64 = wins.iter().sum();
    let sum_losses_abs: f64 = losses.iter().map(|l| l.abs()).sum();
    let pf = if sum_losses_abs > 0.0 { sum_wins / sum_losses_abs } else { f64::INFINITY };
    Some(ExpectancyReport {
        expectancy_per_trade: expectancy,
        r_multiple_expectancy: r_mult,
        win_rate,
        avg_win,
        avg_loss,
        payoff_ratio: payoff,
        profit_factor: pf,
        n_trades: n,
        n_wins,
        n_losses,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_none() {
        assert!(compute(&[]).is_none());
    }

    #[test]
    fn nan_returns_none() {
        assert!(compute(&[1.0, f64::NAN, 2.0]).is_none());
    }

    #[test]
    fn all_wins_yields_positive_expectancy() {
        let trades = vec![100.0, 200.0, 150.0];
        let r = compute(&trades).unwrap();
        assert!(r.expectancy_per_trade > 0.0);
        assert!((r.win_rate - 1.0).abs() < 1e-12);
        assert_eq!(r.n_wins, 3);
        assert_eq!(r.n_losses, 0);
    }

    #[test]
    fn all_losses_yields_negative_expectancy() {
        let trades = vec![-100.0, -200.0, -50.0];
        let r = compute(&trades).unwrap();
        assert!(r.expectancy_per_trade < 0.0);
        assert_eq!(r.n_wins, 0);
    }

    #[test]
    fn classic_50pct_winrate_2to1_payoff() {
        // 5 wins of $200, 5 losses of $100 → expectancy = +$50/trade.
        let mut trades = vec![200.0; 5];
        trades.extend(vec![-100.0; 5]);
        let r = compute(&trades).unwrap();
        assert!((r.expectancy_per_trade - 50.0).abs() < 1e-9);
        assert!((r.win_rate - 0.5).abs() < 1e-12);
        assert!((r.payoff_ratio - 2.0).abs() < 1e-9);
        assert!((r.profit_factor - 2.0).abs() < 1e-9);
    }

    #[test]
    fn r_multiple_consistent_with_winrate_and_payoff() {
        // win_rate=0.6, payoff=2 → R = 0.6·2 - 0.4 = 0.8.
        let mut trades = vec![100.0; 6];
        trades.extend(vec![-50.0; 4]);
        let r = compute(&trades).unwrap();
        assert!((r.r_multiple_expectancy - 0.8).abs() < 1e-9);
    }

    #[test]
    fn zero_pnl_trades_not_counted_as_wins_or_losses() {
        let trades = vec![100.0, 0.0, -50.0];
        let r = compute(&trades).unwrap();
        assert_eq!(r.n_wins, 1);
        assert_eq!(r.n_losses, 1);
        assert_eq!(r.n_trades, 3);
    }

    #[test]
    fn n_trades_reported() {
        let trades = vec![1.0; 50];
        let r = compute(&trades).unwrap();
        assert_eq!(r.n_trades, 50);
    }
}
