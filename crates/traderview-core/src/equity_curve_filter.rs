//! Equity-curve filter — trade the system only while its own equity
//! curve is above its moving average.
//!
//! The hypothetical (always-on) curve updates with every trade; the
//! LIVE account only takes a trade when the hypothetical equity was at
//! or above its N-trade SMA going in. Skipped trades accrue on paper.
//! The classic meta-filter for systems with regime-dependent edge:
//! it can't fix a bad system, but it shortens the losing streaks of a
//! decaying one at the cost of whipsaw re-entries.
//!
//! Pure compute. Companion to `drawdown_throttle` (sizing-based
//! protection on the same idea).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct EcfInput {
    pub starting_equity: f64,
    /// Per-trade P/L, $ oldest-first.
    pub trade_pnls: Vec<f64>,
    /// SMA length on the hypothetical curve (in trades).
    pub ma_length: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct EcfReport {
    pub unfiltered_final: f64,
    pub filtered_final: f64,
    pub unfiltered_max_dd: f64,
    pub filtered_max_dd: f64,
    pub trades_taken: usize,
    pub trades_skipped: usize,
    /// Filtered beat unfiltered on final equity.
    pub filter_helped: bool,
}

pub fn compute(inp: &EcfInput) -> Option<EcfReport> {
    if !inp.starting_equity.is_finite()
        || inp.starting_equity <= 0.0
        || inp.ma_length < 2
        || inp.trade_pnls.len() <= inp.ma_length
        || inp.trade_pnls.len() > 100_000
        || inp.trade_pnls.iter().any(|p| !p.is_finite())
    {
        return None;
    }
    let mut hypo = inp.starting_equity;
    let mut live = inp.starting_equity;
    let mut hypo_curve = vec![inp.starting_equity];
    let (mut hypo_peak, mut live_peak) = (hypo, live);
    let (mut hypo_dd, mut live_dd) = (0.0_f64, 0.0_f64);
    let mut taken = 0usize;
    let mut skipped = 0usize;
    let mut window_sum = inp.starting_equity;
    for (i, &pnl) in inp.trade_pnls.iter().enumerate() {
        // SMA of the last ma_length hypothetical equity points (or all
        // available during warm-up); decision uses state BEFORE the
        // trade — no lookahead.
        let count = hypo_curve.len().min(inp.ma_length);
        let sma = window_sum / count as f64;
        let live_this_one = hypo >= sma;
        hypo += pnl;
        if live_this_one {
            live += pnl;
            taken += 1;
        } else {
            skipped += 1;
        }
        hypo_curve.push(hypo);
        window_sum += hypo;
        if hypo_curve.len() > inp.ma_length {
            window_sum -= hypo_curve[hypo_curve.len() - 1 - inp.ma_length];
        }
        hypo_peak = hypo_peak.max(hypo);
        live_peak = live_peak.max(live);
        hypo_dd = hypo_dd.max(hypo_peak - hypo);
        live_dd = live_dd.max(live_peak - live);
        let _ = i;
    }
    Some(EcfReport {
        unfiltered_final: hypo,
        filtered_final: live,
        unfiltered_max_dd: hypo_dd,
        filtered_max_dd: live_dd,
        trades_taken: taken,
        trades_skipped: skipped,
        filter_helped: live > hypo,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn long_losing_streak_is_mostly_skipped() {
        // 10 winners, then 20 losers, then 5 winners: once the curve
        // crosses under its MA the filter sits out most of the bleed.
        let mut pnls = vec![100.0; 10];
        pnls.extend(vec![-100.0; 20]);
        pnls.extend(vec![100.0; 5]);
        let r = compute(&EcfInput {
            starting_equity: 10_000.0,
            trade_pnls: pnls,
            ma_length: 5,
        })
        .unwrap();
        assert!(r.trades_skipped > 10, "{r:?}");
        assert!(r.filtered_max_dd < r.unfiltered_max_dd);
        assert!(r.filtered_final > r.unfiltered_final);
        assert!(r.filter_helped);
    }

    #[test]
    fn steady_winner_is_never_filtered() {
        // Monotonic equity stays at/above its own SMA: every trade
        // taken, curves identical.
        let r = compute(&EcfInput {
            starting_equity: 10_000.0,
            trade_pnls: vec![50.0; 30],
            ma_length: 5,
        })
        .unwrap();
        assert_eq!(r.trades_skipped, 0);
        assert_eq!(r.filtered_final, r.unfiltered_final);
        assert!(!r.filter_helped); // equal, not better
    }

    #[test]
    fn whipsaw_costs_show_up() {
        // Alternating ±200: the filter skips after every loss and
        // misses the winner that follows — classic whipsaw tax. The
        // filtered curve must NOT beat the unfiltered one here.
        let pnls: Vec<f64> = (0..40).map(|i| if i % 2 == 0 { 200.0 } else { -200.0 }).collect();
        let r = compute(&EcfInput {
            starting_equity: 10_000.0,
            trade_pnls: pnls,
            ma_length: 4,
        })
        .unwrap();
        assert!(r.filtered_final <= r.unfiltered_final, "{r:?}");
    }

    #[test]
    fn hostile_inputs_return_none() {
        assert!(compute(&EcfInput {
            starting_equity: 0.0,
            trade_pnls: vec![1.0; 10],
            ma_length: 5,
        })
        .is_none());
        assert!(compute(&EcfInput {
            starting_equity: 1000.0,
            trade_pnls: vec![1.0; 4],
            ma_length: 5,
        })
        .is_none()); // shorter than the MA
        assert!(compute(&EcfInput {
            starting_equity: 1000.0,
            trade_pnls: vec![f64::NAN; 10],
            ma_length: 5,
        })
        .is_none());
        assert!(compute(&EcfInput {
            starting_equity: 1000.0,
            trade_pnls: vec![1.0; 10],
            ma_length: 1,
        })
        .is_none());
    }
}
