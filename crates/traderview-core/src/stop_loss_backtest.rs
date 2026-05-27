//! Stop-loss method backtester.
//!
//! Replays a list of trades through N different stop-loss strategies
//! and reports which method would have produced the best outcome.
//! Useful for tuning the stop methodology to the trader's actual edge.
//!
//! Pure compute. Caller supplies trade entry + MAE + MFE + actual exit;
//! engine computes what each candidate stop method would have done.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TradeOutcome {
    pub entry: f64,
    pub mae: f64,
    pub mfe: f64,
    pub actual_exit: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StopMethod {
    /// No stop — exit only at MFE or actual_exit.
    None,
    /// Fixed-dollar stop below entry.
    FixedDollar,
    /// Fixed-% of entry.
    FixedPct,
    /// N × ATR below entry.
    AtrMultiple,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopParams {
    pub method: StopMethod,
    /// For FixedDollar = dollar amount; FixedPct = fraction; AtrMultiple
    /// = multiplier (caller passes ATR via separate field).
    pub value: f64,
    pub atr: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodResult {
    pub method: StopMethod,
    pub value: f64,
    pub total_realized: f64,
    pub stopped_out_count: usize,
    pub winning_trades: usize,
    pub avg_realized: f64,
}

pub fn simulate(trades: &[TradeOutcome], params: &StopParams, side_long: bool) -> MethodResult {
    let stop_level = match params.method {
        StopMethod::None => f64::NEG_INFINITY,
        StopMethod::FixedDollar => params.value, // direct dollar offset
        StopMethod::FixedPct => 0.0,             // computed per-trade below
        StopMethod::AtrMultiple => params.value * params.atr,
    };
    let mut total = 0.0;
    let mut stopped = 0usize;
    let mut wins = 0usize;
    for t in trades {
        // Compute stop price.
        let stop_price = match params.method {
            StopMethod::None => f64::NEG_INFINITY,
            StopMethod::FixedDollar => {
                if side_long {
                    t.entry - stop_level
                } else {
                    t.entry + stop_level
                }
            }
            StopMethod::FixedPct => {
                if side_long {
                    t.entry * (1.0 - params.value)
                } else {
                    t.entry * (1.0 + params.value)
                }
            }
            StopMethod::AtrMultiple => {
                if side_long {
                    t.entry - stop_level
                } else {
                    t.entry + stop_level
                }
            }
        };
        // Determine if stop would have been hit (MAE breaches stop?).
        let mae_price = if side_long {
            t.entry - t.mae
        } else {
            t.entry + t.mae
        };
        let hit_stop = if side_long {
            mae_price <= stop_price
        } else {
            mae_price >= stop_price
        };
        let realized = if hit_stop {
            stopped += 1;
            if side_long {
                stop_price - t.entry
            } else {
                t.entry - stop_price
            }
        } else {
            if side_long {
                t.actual_exit - t.entry
            } else {
                t.entry - t.actual_exit
            }
        };
        if realized > 0.0 {
            wins += 1;
        }
        total += realized;
    }
    let n = trades.len();
    MethodResult {
        method: params.method,
        value: params.value,
        total_realized: total,
        stopped_out_count: stopped,
        winning_trades: wins,
        avg_realized: if n > 0 { total / n as f64 } else { 0.0 },
    }
}

/// Run multiple candidate stop methods + pick the highest-total-realized.
pub fn best_of(
    trades: &[TradeOutcome],
    candidates: &[StopParams],
    side_long: bool,
) -> Vec<MethodResult> {
    let mut results: Vec<MethodResult> = candidates
        .iter()
        .map(|c| simulate(trades, c, side_long))
        .collect();
    results.sort_by(|a, b| {
        b.total_realized
            .partial_cmp(&a.total_realized)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(entry: f64, mae: f64, mfe: f64, exit: f64) -> TradeOutcome {
        TradeOutcome {
            entry,
            mae,
            mfe,
            actual_exit: exit,
        }
    }

    #[test]
    fn empty_trades_zero_result() {
        let r = simulate(
            &[],
            &StopParams {
                method: StopMethod::None,
                value: 0.0,
                atr: 0.0,
            },
            true,
        );
        assert_eq!(r.total_realized, 0.0);
    }

    #[test]
    fn no_stop_uses_actual_exit() {
        // Entry 100, MAE 5 (drop to 95), MFE 20, exit 110.
        // No stop → realized = actual_exit - entry = 10.
        let r = simulate(
            &[t(100.0, 5.0, 20.0, 110.0)],
            &StopParams {
                method: StopMethod::None,
                value: 0.0,
                atr: 0.0,
            },
            true,
        );
        assert_eq!(r.total_realized, 10.0);
    }

    #[test]
    fn fixed_dollar_stop_triggers_on_mae() {
        // Entry 100, MAE 5 (price went to 95). Stop $3 below = 97. MAE 95 < 97 → hit.
        // Realized = -3 (loss capped at stop level).
        let r = simulate(
            &[t(100.0, 5.0, 20.0, 110.0)],
            &StopParams {
                method: StopMethod::FixedDollar,
                value: 3.0,
                atr: 0.0,
            },
            true,
        );
        assert_eq!(r.stopped_out_count, 1);
        assert_eq!(r.total_realized, -3.0);
    }

    #[test]
    fn fixed_pct_stop_triggers_on_mae() {
        // Entry 100, MAE 10 → 90. Stop at 5% = 95. MAE 90 < 95 → hit, realized -5.
        let r = simulate(
            &[t(100.0, 10.0, 20.0, 110.0)],
            &StopParams {
                method: StopMethod::FixedPct,
                value: 0.05,
                atr: 0.0,
            },
            true,
        );
        assert_eq!(r.stopped_out_count, 1);
        assert_eq!(r.total_realized, -5.0);
    }

    #[test]
    fn atr_multiple_stop_triggers_on_mae() {
        // ATR 2, multiplier 3 → stop $6 below entry. MAE 10 → stopped at -6.
        let r = simulate(
            &[t(100.0, 10.0, 20.0, 110.0)],
            &StopParams {
                method: StopMethod::AtrMultiple,
                value: 3.0,
                atr: 2.0,
            },
            true,
        );
        assert_eq!(r.stopped_out_count, 1);
        assert_eq!(r.total_realized, -6.0);
    }

    #[test]
    fn stop_not_triggered_when_mae_above_stop() {
        // MAE only $1 → stop at -3 not hit. Realized = actual exit = $10.
        let r = simulate(
            &[t(100.0, 1.0, 20.0, 110.0)],
            &StopParams {
                method: StopMethod::FixedDollar,
                value: 3.0,
                atr: 0.0,
            },
            true,
        );
        assert_eq!(r.stopped_out_count, 0);
        assert_eq!(r.total_realized, 10.0);
    }

    #[test]
    fn short_trade_stop_inverts() {
        // Short entry 100, MAE 5 (price rose to 105). Stop $3 above = 103. MAE 105 > 103 → hit.
        // Realized = entry - stop = 100 - 103 = -3.
        let r = simulate(
            &[t(100.0, 5.0, 20.0, 95.0)],
            &StopParams {
                method: StopMethod::FixedDollar,
                value: 3.0,
                atr: 0.0,
            },
            false,
        );
        assert_eq!(r.stopped_out_count, 1);
        assert_eq!(r.total_realized, -3.0);
    }

    #[test]
    fn best_of_picks_highest_total_realized() {
        let trades = vec![
            t(100.0, 1.0, 20.0, 110.0), // good trade, big MFE, small MAE
            t(100.0, 1.0, 20.0, 110.0),
        ];
        let candidates = vec![
            StopParams {
                method: StopMethod::FixedDollar,
                value: 0.5,
                atr: 0.0,
            }, // too tight
            StopParams {
                method: StopMethod::FixedDollar,
                value: 5.0,
                atr: 0.0,
            }, // loose
            StopParams {
                method: StopMethod::None,
                value: 0.0,
                atr: 0.0,
            },
        ];
        let results = best_of(&trades, &candidates, true);
        // Best should be the looser stop or None (allows trades to reach actual_exit).
        assert!(results[0].total_realized >= results[1].total_realized);
        assert!(results[1].total_realized >= results[2].total_realized);
    }

    #[test]
    fn wins_counted_correctly() {
        let trades = vec![
            t(100.0, 1.0, 20.0, 110.0), // win
            t(100.0, 1.0, 20.0, 95.0),  // loss (actual exit below entry)
        ];
        let r = simulate(
            &trades,
            &StopParams {
                method: StopMethod::None,
                value: 0.0,
                atr: 0.0,
            },
            true,
        );
        assert_eq!(r.winning_trades, 1);
    }
}
