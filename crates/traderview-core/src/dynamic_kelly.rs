//! Dynamic Kelly sizer using rolling win-rate + payoff-ratio window.
//!
//! Standard Kelly assumes static win_rate and payoff_ratio. Real systems
//! drift — wr and payoff change with regime. This module emits a per-bar
//! Kelly fraction based on a ROLLING window of the last N trades, so
//! position size grows when the system is hot and shrinks when cold.
//!
//! Uses half-Kelly by default for stability (full Kelly is notoriously
//! volatile). Returns Some(f) when the window has enough data, None otherwise.
//!
//! Pure compute. Distinct from `crate::kelly` which is a static computation.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DynamicKellyPoint {
    pub window_win_rate: f64,
    pub window_payoff_ratio: Option<f64>,
    pub kelly_fraction: Option<f64>,
    pub half_kelly_fraction: Option<f64>,
}

/// For each trade index `i` ≥ `window` - 1, compute the rolling Kelly
/// from the last `window` trades (ending at trade `i`). Earlier indices
/// emit `None` for kelly_fraction.
pub fn compute(trade_pnls: &[f64], window: usize) -> Vec<DynamicKellyPoint> {
    // window == 0 is undefined — a Kelly with no history is meaningless,
    // and the per-point math divides by `window as f64` which would emit
    // NaN/Inf in `window_win_rate`. Refuse it explicitly.
    if window == 0 {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(trade_pnls.len());
    for i in 0..trade_pnls.len() {
        if i + 1 < window {
            out.push(DynamicKellyPoint::default());
            continue;
        }
        let w = &trade_pnls[(i + 1 - window)..=i];
        // Filter non-finite values out of both buckets. With Inf in wins
        // the avg/payoff escape as Inf; with NaN every comparison is
        // false so NaN trades silently get treated as zero-pnl, but a
        // mix of NaN+Inf in the same window would produce NaN payoff.
        let wins: Vec<f64> = w
            .iter()
            .filter(|p| p.is_finite() && **p > 0.0)
            .cloned()
            .collect();
        let losses: Vec<f64> = w
            .iter()
            .filter(|p| p.is_finite() && **p < 0.0)
            .map(|p| -p)
            .collect();
        let wr = wins.len() as f64 / window as f64;
        let payoff = if losses.is_empty() {
            None
        } else if wins.is_empty() {
            Some(0.0)
        } else {
            let avg_win = wins.iter().sum::<f64>() / wins.len() as f64;
            let avg_loss = losses.iter().sum::<f64>() / losses.len() as f64;
            if avg_loss > 0.0 {
                let p = avg_win / avg_loss;
                // Subnormal avg_loss can make the ratio overflow to Inf
                // even when both numerator and denominator are finite.
                if p.is_finite() {
                    Some(p)
                } else {
                    None
                }
            } else {
                None
            }
        };
        let kelly = payoff.map(|b| {
            if b == 0.0 {
                return -1.0;
            } // pure loss series — negative bet (don't trade)
            let q = 1.0 - wr;
            ((b * wr - q) / b).clamp(-1.0, 1.0)
        });
        out.push(DynamicKellyPoint {
            window_win_rate: wr,
            window_payoff_ratio: payoff,
            kelly_fraction: kelly,
            half_kelly_fraction: kelly.map(|k| (k / 2.0).max(0.0)),
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_empty() {
        assert!(compute(&[], 10).is_empty());
    }

    #[test]
    fn zero_window_returns_empty_no_division_by_zero() {
        // Prior implementation divided wins.len() by `window as f64`,
        // producing NaN in window_win_rate for every output point. The
        // empty-vec return makes window==0 an explicit error case.
        let r = compute(&[100.0, -50.0, 100.0], 0);
        assert!(
            r.is_empty(),
            "window=0 must yield no points, got {} pts",
            r.len()
        );
    }

    #[test]
    fn pre_warmup_indices_return_none() {
        let out = compute(&[100.0, -50.0, 100.0], 5);
        for p in &out {
            assert!(p.kelly_fraction.is_none());
        }
    }

    #[test]
    fn rolling_kelly_positive_for_winning_window() {
        // 60% wr × $200 wins, 40% × $100 losses → payoff = 2, wr=0.6.
        // Kelly = (2 × 0.6 - 0.4) / 2 = 0.4.
        let mut trades = vec![200.0; 6];
        trades.extend(vec![-100.0; 4]);
        let out = compute(&trades, 10);
        let last = &out[9];
        assert!(last.kelly_fraction.is_some());
        let k = last.kelly_fraction.unwrap();
        assert!((k - 0.4).abs() < 1e-9);
        assert!((last.half_kelly_fraction.unwrap() - 0.2).abs() < 1e-9);
    }

    #[test]
    fn rolling_kelly_zero_at_break_even() {
        // 50% wr, 1:1 payoff → Kelly = 0.
        let trades = vec![100.0, -100.0, 100.0, -100.0];
        let out = compute(&trades, 4);
        let last = &out[3];
        assert!(last.kelly_fraction.unwrap().abs() < 1e-9);
    }

    #[test]
    fn rolling_kelly_negative_for_losing_window() {
        // 30% wr × $100 vs 70% × $100 → payoff=1, Kelly = -0.4.
        let mut trades = vec![100.0; 3];
        trades.extend(vec![-100.0; 7]);
        let out = compute(&trades, 10);
        let k = out[9].kelly_fraction.unwrap();
        assert!(k < 0.0);
        // Half Kelly clamps at 0 (don't bet negative).
        assert_eq!(out[9].half_kelly_fraction.unwrap(), 0.0);
    }

    #[test]
    fn no_losses_in_window_payoff_none() {
        let trades = vec![100.0; 10];
        let out = compute(&trades, 10);
        assert!(out[9].window_payoff_ratio.is_none());
    }

    #[test]
    fn pure_losers_in_window_kelly_clamped() {
        let trades = vec![-100.0; 10];
        let out = compute(&trades, 10);
        // 0 wins, 10 losses, win_rate=0. Payoff = 0/avg_loss = 0. Kelly = ?
        // With b=0, set Kelly = -1 (don't trade).
        let k = out[9].kelly_fraction.unwrap();
        assert!(k <= 0.0);
        assert_eq!(out[9].half_kelly_fraction.unwrap(), 0.0);
    }

    #[test]
    fn rolling_window_advances_with_each_new_trade() {
        let trades = vec![
            -100.0, -100.0, -100.0, -100.0, -100.0, // first 5: bad
            100.0, 100.0, 100.0, 100.0, 100.0, // next 5: good
        ];
        let out = compute(&trades, 5);
        // Index 4: full bad window → negative kelly.
        assert!(out[4].kelly_fraction.unwrap() <= 0.0);
        // Index 9: full good window → positive kelly.
        // 100% wr, no losses → payoff = None → kelly = None.
        assert!(out[9].kelly_fraction.is_none());
    }

    #[test]
    fn kelly_clamped_to_one() {
        // Construct a window where raw kelly would exceed 1.
        // 100% wr in window... but no losers → payoff = None → kelly = None.
        // Try 90% wr at 100:1 payoff: kelly = (100 × 0.9 - 0.1)/100 = 0.899 → < 1, ok.
        // Genuinely > 1 case is contrived; the clamp is defensive.
        // Verify it doesn't fail when most outputs are <= 1.
        let mut trades = vec![100.0; 9];
        trades.push(-1.0);
        let out = compute(&trades, 10);
        let k = out[9].kelly_fraction.unwrap();
        assert!(k.abs() <= 1.0, "kelly clamped to [-1, 1]");
    }

    #[test]
    fn zero_pnl_trades_count_in_window_but_not_as_wins_or_losses() {
        // 5 zeros + 5 wins of 100 → 10-bar window: win_rate = 5/10 = 0.5.
        // No losses → payoff = None → kelly = None.
        // Documents the convention: zero-pnl is neither win nor loss.
        let mut trades = vec![0.0; 5];
        trades.extend(vec![100.0; 5]);
        let out = compute(&trades, 10);
        assert_eq!(out[9].window_win_rate, 0.5);
        assert!(out[9].window_payoff_ratio.is_none());
    }

    #[test]
    fn first_pnl_in_window_zero_does_not_panic() {
        let trades = vec![0.0, -50.0, 100.0, -50.0, 100.0];
        let out = compute(&trades, 5);
        // 2 wins, 2 losses, 1 zero → win_rate = 2/5 = 0.4.
        assert_eq!(out[4].window_win_rate, 0.4);
        assert!(out[4].kelly_fraction.is_some());
    }
}
