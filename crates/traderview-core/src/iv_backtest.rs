//! Earnings straddle / strangle backtest engine.
//!
//! Given an implied move and a sample of historical realized moves, simulate
//! buying or selling the at-the-money straddle right before the announcement
//! and closing it the day after. Reports per-quarter average P&L (expressed
//! as $ per $1 of premium paid/collected) plus win rate.
//!
//! Long straddle P&L per $1 of premium:
//!     payoff      = max(0, |realized_move| * spot - debit)
//!     net_per_$1  = (payoff - debit) / debit
//!                 = |realized_move| * spot / debit - 1
//!     With "implied_move = debit/spot", that simplifies to:
//!     net_per_$1  = |realized_pct| / implied_pct - 1
//! Short straddle is the negative.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct StraddleBacktest {
    pub samples: usize,
    pub implied_move_pct: f64,
    pub avg_realized_pct: f64,
    pub median_realized_pct: f64,
    pub edge_pct: f64,         // implied - median realized; positive = implied is "rich"
    pub long_avg_pnl: f64,     // per-quarter avg, in units of $ per $1 of premium
    pub long_win_rate: f64,
    pub short_avg_pnl: f64,
    pub short_win_rate: f64,
    /// Recommendation derived from edge sign + magnitude.
    pub recommendation: &'static str, // "long" | "short" | "neutral"
}

pub fn backtest(implied_move_pct: f64, realized_pcts: &[f64]) -> StraddleBacktest {
    let n = realized_pcts.len();
    if n == 0 {
        return StraddleBacktest {
            samples: 0,
            implied_move_pct,
            avg_realized_pct: 0.0,
            median_realized_pct: 0.0,
            edge_pct: 0.0,
            long_avg_pnl: 0.0,
            long_win_rate: 0.0,
            short_avg_pnl: 0.0,
            short_win_rate: 0.0,
            recommendation: "neutral",
        };
    }

    let mut sorted: Vec<f64> = realized_pcts.iter().map(|x| x.abs()).collect();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let avg = sorted.iter().sum::<f64>() / n as f64;
    let median = if n % 2 == 1 {
        sorted[n / 2]
    } else {
        (sorted[n / 2 - 1] + sorted[n / 2]) / 2.0
    };

    let mut long_pnls = Vec::with_capacity(n);
    let mut long_wins = 0;
    for r in &sorted {
        // Long: pays implied_pct, receives |realized_pct|.
        let pnl = if implied_move_pct > 0.0 {
            r / implied_move_pct - 1.0
        } else { 0.0 };
        if pnl > 0.0 { long_wins += 1; }
        long_pnls.push(pnl);
    }
    let long_avg = long_pnls.iter().sum::<f64>() / n as f64;
    let long_wr = long_wins as f64 / n as f64;
    let short_avg = -long_avg;
    let short_wr = 1.0 - long_wr;

    let edge = implied_move_pct - median;
    let recommendation = if edge >= 1.5 {
        "short" // implied premium is rich; sell
    } else if edge <= -1.5 {
        "long"  // implied premium is cheap; buy
    } else {
        "neutral"
    };

    StraddleBacktest {
        samples: n,
        implied_move_pct,
        avg_realized_pct: avg,
        median_realized_pct: median,
        edge_pct: edge,
        long_avg_pnl: long_avg,
        long_win_rate: long_wr,
        short_avg_pnl: short_avg,
        short_win_rate: short_wr,
        recommendation,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn long_wins_when_realized_exceeds_implied() {
        let bt = backtest(5.0, &[8.0, -7.5, 10.0, -6.0, 9.0, -8.5, 7.5, -7.0]);
        assert!(bt.long_avg_pnl > 0.0);
        assert_eq!(bt.recommendation, "long");
    }

    #[test]
    fn short_wins_when_realized_below_implied() {
        let bt = backtest(8.0, &[2.0, -3.0, 1.5, -2.5, 3.0, -1.0, 2.0, -3.5]);
        assert!(bt.short_avg_pnl > 0.0);
        assert_eq!(bt.recommendation, "short");
    }

    #[test]
    fn empty_sample_safe() {
        let bt = backtest(5.0, &[]);
        assert_eq!(bt.samples, 0);
        assert_eq!(bt.recommendation, "neutral");
    }
}
