//! Trade-quality statistics — a comprehensive trade-journal summary.
//!
//! Produces, per a list of (entry_time, exit_time, pnl, mae, mfe)
//! trades:
//!   - **win_rate**: fraction of trades with pnl > 0
//!   - **profit_factor**: gross winning pnl / gross losing pnl (absolute)
//!   - **expectancy**: average pnl per trade
//!   - **avg_winner**, **avg_loser**, **largest_winner**, **largest_loser**
//!   - **avg_hold_seconds**: mean (exit − entry)
//!   - **mae_to_loss_ratio**: avg MAE on losers / avg loser size (1.0 =
//!     losers held to maximum adverse excursion, 0.0 = always stopped early)
//!   - **mfe_capture_ratio**: avg winner pnl / avg MFE on winners (1.0 =
//!     winners exited at the top, < 0.5 = leaving lots on the table)
//!   - **payoff_ratio**: avg_winner / |avg_loser|
//!
//! Pure compute. Skips NaN trades. Distinct from the existing
//! `excursion` module (per-trade MAE/MFE primitives) and `streaks`
//! (consecutive-win/loss runs) — this is the cross-trade aggregate.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Trade {
    pub entry_unix_seconds: i64,
    pub exit_unix_seconds: i64,
    pub pnl: f64,
    pub max_adverse_excursion: f64,    // negative or zero (max loss during trade)
    pub max_favorable_excursion: f64,  // positive or zero (max gain during trade)
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct TradeQualityReport {
    pub n_trades: usize,
    pub n_winners: usize,
    pub n_losers: usize,
    pub win_rate: f64,
    pub profit_factor: f64,
    pub expectancy: f64,
    pub avg_winner: f64,
    pub avg_loser: f64,
    pub largest_winner: f64,
    pub largest_loser: f64,
    pub gross_profit: f64,
    pub gross_loss: f64,
    pub net_pnl: f64,
    pub avg_hold_seconds: f64,
    pub mae_to_loss_ratio: f64,
    pub mfe_capture_ratio: f64,
    pub payoff_ratio: f64,
}

pub fn analyze(trades: &[Trade]) -> Option<TradeQualityReport> {
    if trades.is_empty() { return None; }
    let mut r = TradeQualityReport::default();
    let mut sum_hold = 0.0_f64;
    let mut sum_mae_losers = 0.0_f64;
    let mut sum_loser_pnl = 0.0_f64;
    let mut sum_mfe_winners = 0.0_f64;
    let mut sum_winner_pnl = 0.0_f64;
    r.largest_winner = f64::NEG_INFINITY;
    r.largest_loser = f64::INFINITY;
    for t in trades {
        if !t.pnl.is_finite()
            || !t.max_adverse_excursion.is_finite()
            || !t.max_favorable_excursion.is_finite()
        {
            continue;
        }
        r.n_trades += 1;
        let hold = (t.exit_unix_seconds - t.entry_unix_seconds).max(0) as f64;
        sum_hold += hold;
        r.net_pnl += t.pnl;
        if t.pnl > 0.0 {
            r.n_winners += 1;
            r.gross_profit += t.pnl;
            if t.pnl > r.largest_winner { r.largest_winner = t.pnl; }
            sum_mfe_winners += t.max_favorable_excursion.max(t.pnl);
            sum_winner_pnl += t.pnl;
        } else if t.pnl < 0.0 {
            r.n_losers += 1;
            r.gross_loss += -t.pnl;
            if t.pnl < r.largest_loser { r.largest_loser = t.pnl; }
            sum_mae_losers += (-t.max_adverse_excursion).max(-t.pnl);
            sum_loser_pnl += -t.pnl;
        }
    }
    if r.n_trades == 0 { return None; }
    let n_f = r.n_trades as f64;
    r.win_rate = r.n_winners as f64 / n_f;
    r.expectancy = r.net_pnl / n_f;
    r.avg_winner = if r.n_winners > 0 { r.gross_profit / r.n_winners as f64 } else { 0.0 };
    r.avg_loser = if r.n_losers > 0 { -r.gross_loss / r.n_losers as f64 } else { 0.0 };
    r.avg_hold_seconds = sum_hold / n_f;
    r.profit_factor = if r.gross_loss > 0.0 {
        r.gross_profit / r.gross_loss
    } else if r.gross_profit > 0.0 { f64::INFINITY } else { 0.0 };
    r.mae_to_loss_ratio = if sum_loser_pnl > 0.0 {
        sum_mae_losers / sum_loser_pnl
    } else { 0.0 };
    r.mfe_capture_ratio = if sum_mfe_winners > 0.0 {
        sum_winner_pnl / sum_mfe_winners
    } else { 0.0 };
    r.payoff_ratio = if r.avg_loser.abs() > 0.0 {
        r.avg_winner / r.avg_loser.abs()
    } else if r.avg_winner > 0.0 { f64::INFINITY } else { 0.0 };
    // Cleanup sentinel extremes when zero winners / losers.
    if r.n_winners == 0 { r.largest_winner = 0.0; }
    if r.n_losers == 0 { r.largest_loser = 0.0; }
    Some(r)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(entry: i64, exit: i64, pnl: f64, mae: f64, mfe: f64) -> Trade {
        Trade {
            entry_unix_seconds: entry,
            exit_unix_seconds: exit,
            pnl,
            max_adverse_excursion: mae,
            max_favorable_excursion: mfe,
        }
    }

    #[test]
    fn empty_returns_none() {
        assert!(analyze(&[]).is_none());
    }

    #[test]
    fn all_nan_returns_none() {
        let trades = vec![t(0, 100, f64::NAN, 0.0, 0.0); 3];
        assert!(analyze(&trades).is_none());
    }

    #[test]
    fn single_winner_trade_reports_100_pct_win_rate() {
        let trades = vec![t(0, 100, 50.0, -10.0, 60.0)];
        let r = analyze(&trades).unwrap();
        assert_eq!(r.win_rate, 1.0);
        assert_eq!(r.n_winners, 1);
        assert_eq!(r.n_losers, 0);
        assert_eq!(r.expectancy, 50.0);
        assert!(r.profit_factor.is_infinite());
    }

    #[test]
    fn single_loser_trade_reports_zero_win_rate() {
        let trades = vec![t(0, 100, -30.0, -50.0, 10.0)];
        let r = analyze(&trades).unwrap();
        assert_eq!(r.win_rate, 0.0);
        assert_eq!(r.expectancy, -30.0);
        assert_eq!(r.profit_factor, 0.0);
    }

    #[test]
    fn mixed_trades_compute_profit_factor_correctly() {
        // 3 winners: +100, +50, +25 → gross profit = 175
        // 2 losers: -40, -60 → gross loss = 100
        // Profit factor = 1.75
        let trades = vec![
            t(0, 100, 100.0, -20.0, 110.0),
            t(0, 100, 50.0, -10.0, 60.0),
            t(0, 100, 25.0, -5.0, 30.0),
            t(0, 100, -40.0, -50.0, 10.0),
            t(0, 100, -60.0, -75.0, 5.0),
        ];
        let r = analyze(&trades).unwrap();
        assert!((r.profit_factor - 1.75).abs() < 1e-9);
        assert_eq!(r.n_winners, 3);
        assert_eq!(r.n_losers, 2);
        assert!((r.win_rate - 0.6).abs() < 1e-9);
    }

    #[test]
    fn expectancy_equals_net_pnl_over_n() {
        let trades = vec![
            t(0, 100, 100.0, 0.0, 0.0),
            t(0, 100, -50.0, 0.0, 0.0),
            t(0, 100, 25.0, 0.0, 0.0),
        ];
        let r = analyze(&trades).unwrap();
        assert!((r.expectancy - 25.0).abs() < 1e-9);
        assert!((r.net_pnl - 75.0).abs() < 1e-9);
    }

    #[test]
    fn payoff_ratio_avg_winner_over_avg_loser() {
        let trades = vec![
            t(0, 100, 100.0, 0.0, 100.0),
            t(0, 100, -25.0, -25.0, 0.0),
        ];
        let r = analyze(&trades).unwrap();
        assert!((r.payoff_ratio - 4.0).abs() < 1e-9);
    }

    #[test]
    fn mfe_capture_ratio_below_one_when_winners_leave_money() {
        // Winner closed at +50 with MFE 100 → captured 50%.
        let trades = vec![t(0, 100, 50.0, -5.0, 100.0)];
        let r = analyze(&trades).unwrap();
        assert!((r.mfe_capture_ratio - 0.5).abs() < 1e-9);
    }

    #[test]
    fn mae_to_loss_ratio_one_when_losers_taken_to_max_adverse() {
        // Loser hit MAE of -50, exited at -50 → ratio = 1.
        let trades = vec![t(0, 100, -50.0, -50.0, 5.0)];
        let r = analyze(&trades).unwrap();
        assert!((r.mae_to_loss_ratio - 1.0).abs() < 1e-9);
    }

    #[test]
    fn nan_trades_skipped() {
        let trades = vec![
            t(0, 100, 100.0, 0.0, 100.0),
            t(0, 100, f64::NAN, 0.0, 0.0),
            t(0, 100, -50.0, -50.0, 0.0),
        ];
        let r = analyze(&trades).unwrap();
        assert_eq!(r.n_trades, 2);
    }

    #[test]
    fn average_hold_seconds_computed() {
        let trades = vec![
            t(100, 200, 10.0, 0.0, 0.0),    // 100s hold
            t(0, 300, 10.0, 0.0, 0.0),      // 300s hold
        ];
        let r = analyze(&trades).unwrap();
        assert!((r.avg_hold_seconds - 200.0).abs() < 1e-9);
    }

    #[test]
    fn negative_hold_times_clamped_to_zero() {
        // Pathological: exit before entry — clamp.
        let trades = vec![t(500, 100, 10.0, 0.0, 0.0)];
        let r = analyze(&trades).unwrap();
        assert_eq!(r.avg_hold_seconds, 0.0);
    }
}
