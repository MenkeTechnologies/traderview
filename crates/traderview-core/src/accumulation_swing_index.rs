//! Accumulation Swing Index (ASI) — J. Welles Wilder, Jr.
//!
//! Cumulative running total of Wilder's Swing Index (SI), which
//! quantifies the "real" price move between bars by referencing
//! open, high, low and close. Wilder considered ASI breakouts of
//! prior highs/lows as confirming genuine trend changes.
//!
//! Formula (Wilder, "New Concepts in Technical Trading Systems", 1978):
//!
//!   K = max(|high_t - close_{t-1}|, |low_t - close_{t-1}|)
//!   R = compute_r(high_t, low_t, close_{t-1}, open_{t-1})
//!     given:
//!       a = |high_t - close_{t-1}|
//!       b = |low_t  - close_{t-1}|
//!       c = |high_t - low_t|
//!       if a >= b and a >= c:  R = a - 0.5·b + 0.25·|close_{t-1} - open_{t-1}|
//!       if b >= a and b >= c:  R = b - 0.5·a + 0.25·|close_{t-1} - open_{t-1}|
//!       else                :  R = c + 0.25·|close_{t-1} - open_{t-1}|
//!   SI = 50 · (close_t - close_{t-1}
//!              + 0.5·(close_t - open_t)
//!              + 0.25·(close_{t-1} - open_{t-1})) / R · K / limit_move
//!   ASI_t = ASI_{t-1} + SI_t (ASI_0 = 0)
//!
//! `limit_move` is the maximum allowable price move per bar for the
//! market in question (Wilder used futures limit-moves, e.g. $0.50 for
//! soybeans). For equities, a typical proxy is the prior close · 0.10.
//!
//! Pure compute. Companion to `swing_index` (single-bar version).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

pub fn compute(bars: &[Bar], limit_move: f64) -> Vec<Option<f64>> {
    let n = bars.len();
    let mut out = vec![None; n];
    if n == 0 || !limit_move.is_finite() || limit_move <= 0.0 {
        return out;
    }
    if bars.iter().any(|b| {
        !b.open.is_finite() || !b.high.is_finite() || !b.low.is_finite() || !b.close.is_finite()
    }) {
        return out;
    }
    let mut asi = 0.0_f64;
    out[0] = Some(asi);
    for i in 1..n {
        let prev = bars[i - 1];
        let cur = bars[i];
        let a = (cur.high - prev.close).abs();
        let b = (cur.low - prev.close).abs();
        let c = (cur.high - cur.low).abs();
        let d = (prev.close - prev.open).abs();
        let r = if a >= b && a >= c {
            a - 0.5 * b + 0.25 * d
        } else if b >= a && b >= c {
            b - 0.5 * a + 0.25 * d
        } else {
            c + 0.25 * d
        };
        if r <= 0.0 {
            out[i] = Some(asi);
            continue;
        }
        let k = a.max(b);
        let numerator = (cur.close - prev.close)
            + 0.5 * (cur.close - cur.open)
            + 0.25 * (prev.close - prev.open);
        let si = 50.0 * numerator / r * k / limit_move;
        asi += si;
        out[i] = Some(asi);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bar(o: f64, h: f64, l: f64, c: f64) -> Bar {
        Bar {
            open: o,
            high: h,
            low: l,
            close: c,
        }
    }

    #[test]
    fn empty_returns_empty() {
        assert!(compute(&[], 10.0).is_empty());
    }

    #[test]
    fn invalid_limit_move_returns_all_none() {
        let bars = vec![bar(100.0, 101.0, 99.0, 100.5); 5];
        assert!(compute(&bars, 0.0).iter().all(|x| x.is_none()));
        assert!(compute(&bars, f64::NAN).iter().all(|x| x.is_none()));
        assert!(compute(&bars, -1.0).iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_all_none() {
        let bars = vec![
            bar(100.0, 101.0, 99.0, 100.5),
            bar(f64::NAN, 102.0, 100.0, 101.5),
        ];
        assert!(compute(&bars, 10.0).iter().all(|x| x.is_none()));
    }

    #[test]
    fn first_bar_is_zero() {
        let bars = vec![bar(100.0, 101.0, 99.0, 100.5)];
        assert!((compute(&bars, 10.0)[0].unwrap()).abs() < 1e-9);
    }

    #[test]
    fn uptrending_bars_yield_positive_asi() {
        let bars: Vec<_> = (0..30)
            .map(|i| {
                let p = 100.0 + i as f64;
                bar(p, p + 0.5, p - 0.5, p + 0.4)
            })
            .collect();
        let r = compute(&bars, 10.0);
        assert!(r[29].unwrap() > 0.0);
    }

    #[test]
    fn downtrending_bars_yield_negative_asi() {
        let bars: Vec<_> = (0..30)
            .map(|i| {
                let p = 200.0 - i as f64;
                bar(p, p + 0.5, p - 0.5, p - 0.4)
            })
            .collect();
        let r = compute(&bars, 10.0);
        assert!(r[29].unwrap() < 0.0);
    }

    #[test]
    fn flat_market_yields_zero_asi() {
        let bars = vec![bar(100.0, 101.0, 99.0, 100.0); 30];
        let r = compute(&bars, 10.0);
        for v in r.iter().flatten() {
            assert!(v.abs() < 1e-9);
        }
    }

    #[test]
    fn output_length_matches_input() {
        let bars = vec![bar(100.0, 101.0, 99.0, 100.5); 30];
        assert_eq!(compute(&bars, 10.0).len(), 30);
    }
}
