//! MAE-distribution stop-tuning analyzer.
//!
//! For a set of historical trades, compute the MAE (Maximum Adverse
//! Excursion) distribution among the WINNERS — and tell the trader
//! what stop distance would have:
//!   (a) preserved at least X% of winners
//!   (b) cut Y% of losers off earlier
//!
//! Trader practice: place stops just OUTSIDE the 90th percentile of
//! winners' MAE so 90% of winners survive while losers get cut faster.
//! This module shipsthat exact analysis.
//!
//! Pure compute. MAE values come in as R-multiples (R-of-risk units)
//! or as price-distance — caller decides; engine is unit-agnostic.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TradeMae {
    /// Final realized result. Positive = winner, negative = loser.
    pub realized: f64,
    /// Maximum adverse excursion magnitude (always positive — the
    /// worst-against-you distance, in same units as realized).
    pub mae: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StopTuningReport {
    pub winner_count: usize,
    pub loser_count: usize,
    /// MAE percentiles among WINNERS (used for stop placement).
    pub winners_mae_p25: f64,
    pub winners_mae_p50: f64,
    pub winners_mae_p75: f64,
    pub winners_mae_p90: f64,
    pub winners_mae_max: f64,
    /// MAE percentiles among LOSERS (for context).
    pub losers_mae_p50: f64,
    pub losers_mae_p90: f64,
    /// At the p90-winners stop, how many losers would be cut earlier.
    pub losers_cut_at_p90_stop: usize,
    pub losers_cut_pct_at_p90: f64,
    /// At the p90-winners stop, how many winners would have been stopped out.
    pub winners_stopped_at_p90: usize,
}

pub fn analyze(trades: &[TradeMae]) -> StopTuningReport {
    let winners: Vec<f64> = trades
        .iter()
        .filter(|t| t.realized > 0.0)
        .map(|t| t.mae)
        .collect();
    let losers: Vec<f64> = trades
        .iter()
        .filter(|t| t.realized <= 0.0)
        .map(|t| t.mae)
        .collect();
    let mut w_sorted = winners.clone();
    let mut l_sorted = losers.clone();
    w_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    l_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let p90_stop = pct(&w_sorted, 0.90);
    let losers_cut = losers.iter().filter(|m| **m > p90_stop).count();
    // "Stopped at p90" means the winner's MAE EXCEEDED the stop distance —
    // i.e. would have been hit out. By definition of p90, exactly 10% of
    // winners are >= p90 (the top 10%). We report the strict-greater count
    // since equality means the stop is AT MAE, not past it.
    let winners_stopped = winners.iter().filter(|m| **m > p90_stop).count();

    StopTuningReport {
        winner_count: winners.len(),
        loser_count: losers.len(),
        winners_mae_p25: pct(&w_sorted, 0.25),
        winners_mae_p50: pct(&w_sorted, 0.50),
        winners_mae_p75: pct(&w_sorted, 0.75),
        winners_mae_p90: p90_stop,
        winners_mae_max: w_sorted.last().copied().unwrap_or(0.0),
        losers_mae_p50: pct(&l_sorted, 0.50),
        losers_mae_p90: pct(&l_sorted, 0.90),
        losers_cut_at_p90_stop: losers_cut,
        losers_cut_pct_at_p90: if losers.is_empty() {
            0.0
        } else {
            losers_cut as f64 / losers.len() as f64 * 100.0
        },
        winners_stopped_at_p90: winners_stopped,
    }
}

fn pct(sorted: &[f64], q: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((sorted.len() as f64 - 1.0) * q).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_default() {
        let r = analyze(&[]);
        assert_eq!(r.winner_count, 0);
        assert_eq!(r.loser_count, 0);
    }

    #[test]
    fn all_winners_winners_mae_percentiles_populated() {
        let trades: Vec<_> = (1..=10)
            .map(|i| TradeMae {
                realized: 100.0,
                mae: i as f64,
            })
            .collect();
        let r = analyze(&trades);
        assert_eq!(r.winner_count, 10);
        assert_eq!(r.loser_count, 0);
        // p50 of [1..10] = sorted[round(9 × 0.5)] = sorted[round(4.5)] = sorted[5] = 6.
        // (f64::round() is half-away-from-zero in Rust.)
        assert_eq!(r.winners_mae_p50, 6.0);
        // p90 = sorted[round(9 × 0.9)] = sorted[round(8.1)] = sorted[8] = 9.
        assert_eq!(r.winners_mae_p90, 9.0);
        assert_eq!(r.winners_mae_max, 10.0);
    }

    #[test]
    fn losers_cut_at_p90_stop_counted_correctly() {
        // 10 winners with MAE 1..10 → p90 = 9.
        // 4 losers with MAE [3, 5, 10, 15] → 2 above 9 (10 and 15).
        let mut trades: Vec<TradeMae> = (1..=10)
            .map(|i| TradeMae {
                realized: 100.0,
                mae: i as f64,
            })
            .collect();
        trades.push(TradeMae {
            realized: -50.0,
            mae: 3.0,
        });
        trades.push(TradeMae {
            realized: -50.0,
            mae: 5.0,
        });
        trades.push(TradeMae {
            realized: -50.0,
            mae: 10.0,
        });
        trades.push(TradeMae {
            realized: -50.0,
            mae: 15.0,
        });
        let r = analyze(&trades);
        assert_eq!(r.winners_mae_p90, 9.0);
        assert_eq!(r.losers_cut_at_p90_stop, 2);
        assert_eq!(r.losers_cut_pct_at_p90, 50.0);
    }

    #[test]
    fn winners_stopped_at_p90_is_top_10pct() {
        // 10 winners → p90 = sorted[8] = 9. Strict-greater means just MAE=10.
        let trades: Vec<_> = (1..=10)
            .map(|i| TradeMae {
                realized: 100.0,
                mae: i as f64,
            })
            .collect();
        let r = analyze(&trades);
        assert_eq!(r.winners_stopped_at_p90, 1);
    }

    #[test]
    fn zero_pnl_classified_as_loser() {
        // A breakeven trade should count toward losers (realized <= 0).
        let trades = vec![
            TradeMae {
                realized: 100.0,
                mae: 1.0,
            },
            TradeMae {
                realized: 0.0,
                mae: 2.0,
            },
        ];
        let r = analyze(&trades);
        assert_eq!(r.winner_count, 1);
        assert_eq!(r.loser_count, 1);
    }

    #[test]
    fn winners_mae_percentiles_monotonic() {
        let trades: Vec<_> = (1..=100)
            .map(|i| TradeMae {
                realized: 1.0,
                mae: i as f64,
            })
            .collect();
        let r = analyze(&trades);
        assert!(r.winners_mae_p25 < r.winners_mae_p50);
        assert!(r.winners_mae_p50 < r.winners_mae_p75);
        assert!(r.winners_mae_p75 < r.winners_mae_p90);
        assert!(r.winners_mae_p90 <= r.winners_mae_max);
    }

    #[test]
    fn losers_mae_percentiles_independent_of_winners() {
        // 5 winners with MAE 1..5, 5 losers with MAE 10..50.
        let mut trades: Vec<TradeMae> = (1..=5)
            .map(|i| TradeMae {
                realized: 100.0,
                mae: i as f64,
            })
            .collect();
        for m in [10.0, 20.0, 30.0, 40.0, 50.0] {
            trades.push(TradeMae {
                realized: -50.0,
                mae: m,
            });
        }
        let r = analyze(&trades);
        // Loser p50 = sorted[round(4*0.5)] = sorted[2] = 30.
        assert_eq!(r.losers_mae_p50, 30.0);
        // Loser p90 = sorted[round(4*0.9)] = sorted[4] = 50.
        assert_eq!(r.losers_mae_p90, 50.0);
    }
}
