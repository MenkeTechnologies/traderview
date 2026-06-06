//! Welles Wilder Swing Index (SI) + Accumulation Swing Index (ASI).
//!
//! Wilder's SI normalises one-bar swing strength to a -100..+100 range,
//! making swings comparable across symbols of very different prices:
//!
//!   K = max(|h_t − c_{t−1}|, |l_t − c_{t−1}|)
//!   R = (with t−1 close vs t high/low + open/close) — see formula
//!   SI_t = 50 · ((c_t − c_{t−1}) + 0.5·(c_t − o_t) + 0.25·(c_{t−1} − o_{t−1}))
//!          · (K / limit_move) / R
//!
//! `limit_move` defaults to 3.0 (Wilder's choice, originally from
//! commodity contracts' price-limit rule). The Accumulation Swing Index
//! (ASI) is the running cumulative sum of SI:
//!
//!   ASI_t = ASI_{t−1} + SI_t
//!
//! Pure compute. Standard reading: ASI breaking above its prior high =
//! confirmed trend continuation; breaking below = trend ending.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SwingIndexReport {
    pub si: Vec<Option<f64>>,
    /// Accumulation Swing Index — running cumulative sum of SI.
    pub asi: Vec<Option<f64>>,
}

pub fn compute(bars: &[Bar], limit_move: f64) -> SwingIndexReport {
    let n = bars.len();
    let mut report = SwingIndexReport {
        si: vec![None; n],
        asi: vec![None; n],
    };
    if !limit_move.is_finite() || limit_move <= 0.0 || n < 2 {
        return report;
    }
    let mut acc = 0.0_f64;
    let mut acc_seeded = false;
    for i in 1..n {
        let t = bars[i];
        let p = bars[i - 1];
        // Sanity: skip non-finite bars.
        if !(t.open.is_finite()
            && t.high.is_finite()
            && t.low.is_finite()
            && t.close.is_finite()
            && p.open.is_finite()
            && p.close.is_finite())
        {
            continue;
        }
        let move_a = (t.high - p.close).abs();
        let move_b = (t.low - p.close).abs();
        let move_c = (t.high - t.low).abs();
        let k = move_a.max(move_b);
        // Wilder's R selection (largest of the three movements, then add
        // 0.25·|c_{t−1} − o_{t−1}| or similar correction term per movement).
        let r = if move_a >= move_b && move_a >= move_c {
            move_a - 0.5 * move_b + 0.25 * (p.close - p.open).abs()
        } else if move_b >= move_a && move_b >= move_c {
            move_b - 0.5 * move_a + 0.25 * (p.close - p.open).abs()
        } else {
            move_c + 0.25 * (p.close - p.open).abs()
        };
        if r.abs() < f64::EPSILON {
            continue;
        }
        let numer = (t.close - p.close) + 0.5 * (t.close - t.open) + 0.25 * (p.close - p.open);
        let si = 50.0 * numer * (k / limit_move) / r;
        if !si.is_finite() {
            continue;
        }
        report.si[i] = Some(si);
        if !acc_seeded {
            acc = si;
            acc_seeded = true;
        } else {
            acc += si;
        }
        if acc.is_finite() {
            report.asi[i] = Some(acc);
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(o: f64, h: f64, l: f64, c: f64) -> Bar {
        Bar {
            open: o,
            high: h,
            low: l,
            close: c,
        }
    }

    #[test]
    fn empty_or_single_returns_empty_report() {
        let r = compute(&[], 3.0);
        assert!(r.si.is_empty());
        let r = compute(&[b(100.0, 101.0, 99.0, 100.5)], 3.0);
        assert!(r.si.iter().all(|x| x.is_none()));
    }

    #[test]
    fn invalid_limit_move_returns_all_none() {
        let bars = vec![b(100.0, 101.0, 99.0, 100.5); 10];
        for lim in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            let r = compute(&bars, lim);
            assert!(r.si.iter().all(|x| x.is_none()), "limit={lim}");
        }
    }

    #[test]
    fn flat_series_swing_index_zero_or_none() {
        // Identical bars → numer = 0 → SI = 0.
        let bars = vec![b(100.0, 100.0, 100.0, 100.0); 30];
        let r = compute(&bars, 3.0);
        // All entries are either None (if r==0 from identical bars) or 0.
        for x in r.si.iter().flatten() {
            assert!(x.abs() < 1e-9);
        }
    }

    #[test]
    fn rising_close_sequence_positive_si() {
        // close[i] > close[i-1] → numer > 0 → SI > 0.
        let bars: Vec<Bar> = (1..=20)
            .map(|i| {
                let c = 100.0 + i as f64;
                b(c - 0.5, c + 0.5, c - 0.5, c)
            })
            .collect();
        let r = compute(&bars, 3.0);
        let last = r.si[19].expect("populated");
        assert!(last > 0.0, "rising closes should yield + SI, got {last}");
        // ASI should also be positive.
        let asi = r.asi[19].expect("populated");
        assert!(asi > 0.0);
    }

    #[test]
    fn asi_is_running_sum_of_si() {
        let bars: Vec<Bar> = (0..20)
            .map(|i| {
                let c = 100.0 + (i as f64 * 0.5).sin() * 2.0;
                b(c - 0.5, c + 1.0, c - 1.0, c)
            })
            .collect();
        let r = compute(&bars, 3.0);
        let mut running = 0.0;
        let mut seeded = false;
        for i in 0..r.si.len() {
            if let Some(si) = r.si[i] {
                if !seeded {
                    running = si;
                    seeded = true;
                } else {
                    running += si;
                }
                let asi = r.asi[i].expect("ASI must accompany SI");
                assert!((asi - running).abs() < 1e-9, "i={i}");
            }
        }
    }

    #[test]
    fn nan_bar_skipped_safely() {
        let bars = vec![
            b(100.0, 101.0, 99.0, 100.0),
            b(f64::NAN, f64::NAN, f64::NAN, f64::NAN),
            b(100.0, 101.0, 99.0, 100.0),
        ];
        let r = compute(&bars, 3.0);
        // Middle bar produces None (skipped).
        assert!(r.si[1].is_none());
    }
}
