//! Trade report card — one P/L list in, every system-quality verdict
//! out. Pure composition of the sibling modules (no new statistics):
//!
//!   - profit_factor      → PF, PRR, recovery factor, win/loss counts
//!   - win_rate_confidence → Wilson interval vs payoff breakeven
//!   - drawdown_episodes   → worst events on the equity curve
//!   - risk_of_ruin        → analytic RoR at the OBSERVED p and R
//!   - equity_curve_filter → would the own-MA filter have helped?
//!
//! The point is the cross-read: a PF that looks fine next to a Wilson
//! interval still straddling breakeven, or a RoR that says the
//! observed edge can't survive the observed sizing.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct ReportCardInput {
    pub starting_equity: f64,
    /// Per-trade P/L, $ oldest-first.
    pub trade_pnls: Vec<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReportCard {
    pub trades: usize,
    pub quality: crate::profit_factor::SystemQualityReport,
    pub win_rate: Option<crate::win_rate_confidence::WinRateReport>,
    pub drawdowns: Option<crate::drawdown_episodes::EpisodesReport>,
    pub risk_of_ruin: Option<crate::risk_of_ruin::RuinReport>,
    pub equity_filter: Option<crate::equity_curve_filter::EcfReport>,
    /// Observed inputs fed to the ruin model.
    pub observed_payoff_ratio: Option<f64>,
    pub observed_avg_loss: Option<f64>,
}

pub fn compute(inp: &ReportCardInput) -> Option<ReportCard> {
    if !inp.starting_equity.is_finite()
        || inp.starting_equity <= 0.0
        || inp.trade_pnls.len() < 2
        || inp.trade_pnls.len() > 100_000
        || inp.trade_pnls.iter().any(|p| !p.is_finite())
    {
        return None;
    }
    // Equity curve for the drawdown / quality legs.
    let mut equity = inp.starting_equity;
    let mut curve = vec![equity];
    for p in &inp.trade_pnls {
        equity += p;
        curve.push(equity);
    }
    let quality = crate::profit_factor::analyze(&inp.trade_pnls, &[], &curve);
    let wins = quality.win_count;
    let losses = quality.loss_count;
    let avg_win = (wins > 0).then(|| quality.gross_profit / wins as f64);
    let avg_loss = (losses > 0).then(|| quality.gross_loss / losses as f64);
    let payoff = match (avg_win, avg_loss) {
        (Some(w), Some(l)) if l > 0.0 => Some(w / l),
        _ => None,
    };
    let win_rate = payoff.and_then(|r| {
        crate::win_rate_confidence::compute(&crate::win_rate_confidence::WinRateInput {
            wins,
            losses,
            payoff_ratio: r,
            z: 1.96,
        })
    });
    let drawdowns = crate::drawdown_episodes::compute(&curve, 5);
    let risk_of_ruin = match (payoff, avg_loss) {
        (Some(r), Some(l)) if l > 0.0 && l < inp.starting_equity => {
            crate::risk_of_ruin::compute(&crate::risk_of_ruin::RuinInput {
                win_probability: wins as f64 / (wins + losses) as f64,
                payoff_ratio: r,
                capital: inp.starting_equity,
                risk_per_trade: l,
            })
        }
        _ => None,
    };
    let equity_filter = crate::equity_curve_filter::compute(&crate::equity_curve_filter::EcfInput {
        starting_equity: inp.starting_equity,
        trade_pnls: inp.trade_pnls.clone(),
        ma_length: 5,
    });
    Some(ReportCard {
        trades: inp.trade_pnls.len(),
        quality,
        win_rate,
        drawdowns,
        risk_of_ruin,
        equity_filter,
        observed_payoff_ratio: payoff,
        observed_avg_loss: avg_loss,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_legs_populate_on_a_mixed_record() {
        // 6 wins of $300, 4 losses of $200 on $10k.
        let mut pnls = vec![300.0; 6];
        pnls.extend(vec![-200.0; 4]);
        let r = compute(&ReportCardInput {
            starting_equity: 10_000.0,
            trade_pnls: pnls,
        })
        .unwrap();
        assert_eq!(r.trades, 10);
        // Quality leg: PF = 1800/800 = 2.25.
        assert!((r.quality.profit_factor.unwrap() - 2.25).abs() < 1e-12);
        // Observed payoff 300/200 = 1.5 feeds the other legs.
        assert!((r.observed_payoff_ratio.unwrap() - 1.5).abs() < 1e-12);
        let wr = r.win_rate.as_ref().expect("win-rate leg");
        assert!((wr.observed_win_rate_pct - 60.0).abs() < 1e-9);
        // Breakeven at 1.5 payoff = 40%; 10 trades can't prove 60% > 40%.
        assert!(!wr.statistically_significant);
        let ror = r.risk_of_ruin.as_ref().expect("ruin leg");
        assert!(ror.risk_of_ruin < 1.0);
        assert!(r.equity_filter.is_some());
        assert!(r.drawdowns.is_some());
    }

    #[test]
    fn cross_read_consistency_between_legs() {
        // The ruin leg must consume exactly the observed p and R that
        // the quality leg reports.
        let pnls = vec![500.0, -250.0, 500.0, -250.0, 500.0, 500.0];
        let r = compute(&ReportCardInput {
            starting_equity: 20_000.0,
            trade_pnls: pnls,
        })
        .unwrap();
        let ror = r.risk_of_ruin.as_ref().expect("ruin leg");
        // p = 4/6; expectancy_r = p·R − q with R = 2.
        let p = 4.0 / 6.0;
        assert!((ror.expectancy_r - (p * 2.0 - (1.0 - p))).abs() < 1e-12);
        assert!((r.observed_avg_loss.unwrap() - 250.0).abs() < 1e-12);
    }

    #[test]
    fn all_winners_degrades_gracefully() {
        // No losses: PF/payoff/ruin legs are None, the rest still fill.
        let r = compute(&ReportCardInput {
            starting_equity: 10_000.0,
            trade_pnls: vec![100.0; 8],
        })
        .unwrap();
        assert!(r.quality.profit_factor.is_none());
        assert!(r.observed_payoff_ratio.is_none());
        assert!(r.risk_of_ruin.is_none());
        assert!(r.win_rate.is_none());
        assert!(r.drawdowns.is_some());
    }

    #[test]
    fn hostile_inputs_return_none() {
        assert!(compute(&ReportCardInput {
            starting_equity: 0.0,
            trade_pnls: vec![1.0, 2.0],
        })
        .is_none());
        assert!(compute(&ReportCardInput {
            starting_equity: 1000.0,
            trade_pnls: vec![1.0],
        })
        .is_none());
        assert!(compute(&ReportCardInput {
            starting_equity: 1000.0,
            trade_pnls: vec![f64::NAN, 1.0],
        })
        .is_none());
    }
}
