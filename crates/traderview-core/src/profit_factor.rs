//! Profit Factor + Recovery Factor + GPR (Gain-to-Pain Ratio).
//!
//! Three system-quality metrics traders use alongside Sharpe:
//!
//! **Profit Factor** = total_gross_profit / total_gross_loss
//!   - > 1.0 = profitable, > 1.5 = good, > 2.0 = excellent
//!   - Ignores trade count + variance, just the big-picture ratio.
//!
//! **Recovery Factor** = total_net_profit / max_drawdown
//!   - How much profit per dollar of risked drawdown.
//!   - > 5.0 = robust, < 1.0 = fragile (DD ate most of the profit).
//!
//! **GPR (Gain-to-Pain Ratio)** = net_profit / sum_of_monthly_losses
//!   - Pain-adjusted. Higher = less "underwater" feeling per dollar
//!     of profit. Jack Schwager's preferred metric.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SystemQualityReport {
    pub gross_profit: f64,
    pub gross_loss: f64,
    pub net_profit: f64,
    pub max_drawdown_dollars: f64,
    /// gross_profit / gross_loss. None when no losers.
    pub profit_factor: Option<f64>,
    /// net_profit / max_dd. None when no drawdown.
    pub recovery_factor: Option<f64>,
    /// net_profit / sum_of_monthly_losses. None when no monthly losses.
    pub gain_to_pain: Option<f64>,
}

pub fn analyze(trade_pnls: &[f64], monthly_pnls: &[f64], equity_curve: &[f64])
    -> SystemQualityReport
{
    let mut report = SystemQualityReport::default();
    if !trade_pnls.is_empty() {
        let wins: f64 = trade_pnls.iter().filter(|p| **p > 0.0).sum();
        let losses: f64 = trade_pnls.iter().filter(|p| **p < 0.0).map(|p| -p).sum();
        report.gross_profit = wins;
        report.gross_loss = losses;
        report.net_profit = wins - losses;
        report.profit_factor = if losses > 0.0 { Some(wins / losses) } else { None };
    }
    if !equity_curve.is_empty() {
        let mut peak = equity_curve[0];
        let mut max_dd = 0.0_f64;
        for &v in equity_curve {
            if v > peak { peak = v; }
            let dd = peak - v;
            if dd > max_dd { max_dd = dd; }
        }
        report.max_drawdown_dollars = max_dd;
        if max_dd > 0.0 {
            report.recovery_factor = Some(report.net_profit / max_dd);
        }
    }
    if !monthly_pnls.is_empty() {
        let monthly_losses: f64 = monthly_pnls.iter()
            .filter(|p| **p < 0.0).map(|p| -p).sum();
        if monthly_losses > 0.0 {
            report.gain_to_pain = Some(report.net_profit / monthly_losses);
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_default() {
        let r = analyze(&[], &[], &[]);
        assert_eq!(r.gross_profit, 0.0);
        assert!(r.profit_factor.is_none());
    }

    #[test]
    fn profit_factor_two_when_wins_double_losses() {
        // $1000 wins, $500 losses → PF = 2.0.
        let trades = vec![500.0, 500.0, -250.0, -250.0];
        let r = analyze(&trades, &[], &[]);
        assert_eq!(r.gross_profit, 1000.0);
        assert_eq!(r.gross_loss, 500.0);
        assert_eq!(r.profit_factor, Some(2.0));
        assert_eq!(r.net_profit, 500.0);
    }

    #[test]
    fn no_losses_pf_is_none() {
        let r = analyze(&[100.0, 200.0], &[], &[]);
        assert!(r.profit_factor.is_none());
    }

    #[test]
    fn recovery_factor_requires_equity_curve() {
        let trades = vec![500.0, -200.0];
        // Equity 1000 → 800 (down 200) → 1300 → 1100. Max DD = 200.
        let equity = vec![1000.0, 800.0, 1300.0, 1100.0];
        let r = analyze(&trades, &[], &equity);
        // Net profit from trades: 300. Recovery factor = 300/200 = 1.5.
        assert_eq!(r.recovery_factor, Some(1.5));
    }

    #[test]
    fn no_drawdown_recovery_factor_none() {
        let r = analyze(&[100.0], &[], &[1000.0, 1100.0, 1200.0]);
        assert!(r.recovery_factor.is_none());
    }

    #[test]
    fn gain_to_pain_uses_monthly_losses() {
        // $1000 net profit. Monthly losses = $200 + $100 = $300. GPR = 3.33.
        let trades = vec![1000.0];
        let monthly = vec![500.0, -200.0, 700.0, -100.0];
        let r = analyze(&trades, &monthly, &[]);
        // Note: net_profit comes from trade_pnls, not monthly.
        assert!((r.gain_to_pain.unwrap() - 1000.0 / 300.0).abs() < 1e-9);
    }

    #[test]
    fn no_monthly_losses_gpr_none() {
        let r = analyze(&[100.0], &[100.0, 50.0], &[]);
        assert!(r.gain_to_pain.is_none());
    }

    #[test]
    fn losing_system_negative_pf_less_than_one() {
        // $300 wins, $700 losses → PF = 0.43.
        let r = analyze(&[300.0, -700.0], &[], &[]);
        assert!(r.profit_factor.unwrap() < 1.0);
        assert!(r.net_profit < 0.0);
    }

    #[test]
    fn max_drawdown_dollar_value_correct() {
        // 1000 → 1500 → 1200 → 1800. Peak 1500, trough 1200 → DD = $300.
        let r = analyze(&[], &[], &[1000.0, 1500.0, 1200.0, 1800.0]);
        assert_eq!(r.max_drawdown_dollars, 300.0);
    }
}
