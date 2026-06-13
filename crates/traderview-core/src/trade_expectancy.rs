//! Trade expectancy — the per-trade edge of a strategy.
//!
//! Whether a system makes money long-run isn't the win rate alone — it's the
//! win rate weighted by how much you win vs lose:
//!
//!   * expectancy = (win% × avg win) − (loss% × avg loss)
//!   * reward:risk = avg win / avg loss
//!   * break-even win rate = avg loss / (avg win + avg loss) — the win rate
//!     that makes expectancy zero at this reward:risk
//!   * expectancy in R = expectancy / avg loss (per-trade, in risk units)
//!
//! A positive expectancy is an edge; a high win rate with a poor reward:risk
//! can still lose. Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct ExpectancyInput {
    pub win_rate_pct: f64,
    pub avg_win_usd: f64,
    /// Average loss as a positive number.
    pub avg_loss_usd: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExpectancyResult {
    pub loss_rate_pct: f64,
    /// Expected profit/loss per trade.
    pub expectancy_per_trade_usd: f64,
    pub reward_risk_ratio: f64,
    /// Win rate that yields zero expectancy at this reward:risk.
    pub breakeven_win_rate_pct: f64,
    /// Expectancy expressed in R (multiples of the average loss).
    pub expectancy_in_r: f64,
    /// Expected profit over 100 trades.
    pub expectancy_per_100_trades_usd: f64,
    /// True when expectancy is positive (a real edge).
    pub has_edge: bool,
}

pub fn analyze(i: &ExpectancyInput) -> ExpectancyResult {
    let win = (i.win_rate_pct / 100.0).clamp(0.0, 1.0);
    let loss_rate = 1.0 - win;
    let avg_win = i.avg_win_usd.max(0.0);
    let avg_loss = i.avg_loss_usd.max(0.0);

    let expectancy = win * avg_win - loss_rate * avg_loss;
    let reward_risk = if avg_loss > 0.0 { avg_win / avg_loss } else { 0.0 };
    let breakeven = if avg_win + avg_loss > 0.0 {
        avg_loss / (avg_win + avg_loss) * 100.0
    } else {
        0.0
    };
    let expectancy_r = if avg_loss > 0.0 { expectancy / avg_loss } else { 0.0 };

    ExpectancyResult {
        loss_rate_pct: loss_rate * 100.0,
        expectancy_per_trade_usd: expectancy,
        reward_risk_ratio: reward_risk,
        breakeven_win_rate_pct: breakeven,
        expectancy_in_r: expectancy_r,
        expectancy_per_100_trades_usd: expectancy * 100.0,
        has_edge: expectancy > 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inp(win: f64, w: f64, l: f64) -> ExpectancyInput {
        ExpectancyInput { win_rate_pct: win, avg_win_usd: w, avg_loss_usd: l }
    }

    #[test]
    fn expectancy_formula() {
        // 40% win, win 300, loss 100: 0.4×300 − 0.6×100 = 120 − 60 = 60.
        let r = analyze(&inp(40.0, 300.0, 100.0));
        assert!((r.expectancy_per_trade_usd - 60.0).abs() < 1e-9);
        assert!((r.loss_rate_pct - 60.0).abs() < 1e-9);
    }

    #[test]
    fn reward_risk_ratio() {
        let r = analyze(&inp(40.0, 300.0, 100.0));
        assert!((r.reward_risk_ratio - 3.0).abs() < 1e-9);
    }

    #[test]
    fn breakeven_win_rate() {
        // loss/(win+loss) = 100/400 = 25%.
        let r = analyze(&inp(40.0, 300.0, 100.0));
        assert!((r.breakeven_win_rate_pct - 25.0).abs() < 1e-9);
    }

    #[test]
    fn has_edge_when_positive() {
        let r = analyze(&inp(40.0, 300.0, 100.0));
        assert!(r.has_edge); // 40% > 25% breakeven
    }

    #[test]
    fn high_win_rate_can_still_lose() {
        // 70% win but win 50, loss 200: 0.7×50 − 0.3×200 = 35 − 60 = −25.
        let r = analyze(&inp(70.0, 50.0, 200.0));
        assert!(r.expectancy_per_trade_usd < 0.0);
        assert!(!r.has_edge);
        // breakeven = 200/250 = 80% > 70% actual → loses.
        assert!((r.breakeven_win_rate_pct - 80.0).abs() < 1e-9);
    }

    #[test]
    fn at_breakeven_win_rate_expectancy_zero() {
        // R:R 3:1 → breakeven 25%; at exactly 25% expectancy ≈ 0.
        let r = analyze(&inp(25.0, 300.0, 100.0));
        assert!(r.expectancy_per_trade_usd.abs() < 1e-9);
    }

    #[test]
    fn expectancy_in_r_units() {
        // Expectancy 60 / avg loss 100 = 0.6R.
        let r = analyze(&inp(40.0, 300.0, 100.0));
        assert!((r.expectancy_in_r - 0.6).abs() < 1e-9);
        assert!((r.expectancy_per_100_trades_usd - 6_000.0).abs() < 1e-6);
    }

    #[test]
    fn higher_win_rate_raises_expectancy() {
        let low = analyze(&inp(30.0, 200.0, 100.0));
        let high = analyze(&inp(60.0, 200.0, 100.0));
        assert!(high.expectancy_per_trade_usd > low.expectancy_per_trade_usd);
    }
}
