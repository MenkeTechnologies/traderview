//! Anchored On-Balance Volume — OBV restarted at a user-chosen anchor.
//!
//! Standard OBV is cumulative from epoch which makes its absolute level
//! meaningless (charts only consider its slope/divergence vs price).
//! Anchored OBV restarts at zero at a specific bar (typically an earnings
//! release, a halt resume, an FOMC announcement, or a swing low),
//! making the cumulative reading on that anchor an absolute "since the
//! event" buying-pressure metric directly comparable to the price move.
//!
//!   for i from anchor:
//!     obv_t = obv_{t−1} + sign(close_t − close_{t−1}) · volume_t
//!     where sign(0) = 0
//!
//! Pure compute. Output length matches input; positions before `anchor`
//! are `None`, position `anchor` is `Some(0.0)`.

pub fn compute(closes: &[f64], volumes: &[f64], anchor: usize) -> Vec<Option<f64>> {
    let n = closes.len();
    let mut out = vec![None; n];
    if closes.len() != volumes.len() || anchor >= n {
        return out;
    }
    out[anchor] = Some(0.0);
    let mut acc = 0.0_f64;
    for i in (anchor + 1)..n {
        if !closes[i].is_finite() || !closes[i - 1].is_finite() || !volumes[i].is_finite() {
            // Carry prior value forward; don't corrupt acc with NaN.
            out[i] = Some(acc);
            continue;
        }
        let delta = closes[i] - closes[i - 1];
        if delta > 0.0 {
            acc += volumes[i];
        } else if delta < 0.0 {
            acc -= volumes[i];
        }
        if acc.is_finite() {
            out[i] = Some(acc);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_empty() {
        assert!(compute(&[], &[], 0).is_empty());
    }

    #[test]
    fn length_mismatch_returns_all_none() {
        let c = vec![100.0; 10];
        let v = vec![1_000.0; 5];
        assert!(compute(&c, &v, 0).iter().all(|x| x.is_none()));
    }

    #[test]
    fn anchor_out_of_range_returns_all_none() {
        let c = vec![100.0; 10];
        let v = vec![1_000.0; 10];
        assert!(compute(&c, &v, 99).iter().all(|x| x.is_none()));
    }

    #[test]
    fn anchor_seeded_to_zero() {
        let c = vec![100.0; 10];
        let v = vec![1_000.0; 10];
        let out = compute(&c, &v, 3);
        assert_eq!(out[3], Some(0.0));
        assert!(out[2].is_none());
    }

    #[test]
    fn rising_closes_accumulate_positive_obv() {
        let c: Vec<f64> = (0..10).map(|i| 100.0 + i as f64).collect();
        let v = vec![1_000.0; 10];
        let out = compute(&c, &v, 0);
        assert_eq!(out[0], Some(0.0));
        assert!(out[9].unwrap() > 0.0);
        // 9 up bars × 1000 = 9000.
        assert!((out[9].unwrap() - 9_000.0).abs() < 1e-9);
    }

    #[test]
    fn falling_closes_accumulate_negative_obv() {
        let c: Vec<f64> = (0..10).map(|i| 200.0 - i as f64).collect();
        let v = vec![1_000.0; 10];
        let out = compute(&c, &v, 0);
        assert!(out[9].unwrap() < 0.0);
    }

    #[test]
    fn flat_close_neither_adds_nor_subtracts() {
        let c = vec![100.0; 10];
        let v = vec![1_000.0; 10];
        let out = compute(&c, &v, 0);
        for x in out.iter().flatten() {
            assert_eq!(*x, 0.0);
        }
    }

    #[test]
    fn nan_volume_carries_prior_without_corrupting() {
        let c: Vec<f64> = (0..10).map(|i| 100.0 + i as f64).collect();
        let mut v = vec![1_000.0; 10];
        v[5] = f64::NAN;
        let out = compute(&c, &v, 0);
        // Bar 4: 4 up moves = 4000. Bar 5: NaN volume → carry 4000.
        assert_eq!(out[4], Some(4_000.0));
        assert_eq!(out[5], Some(4_000.0));
        // Bar 6 onward continues from 4000 (up move adds 1000 → 5000).
        assert_eq!(out[6], Some(5_000.0));
    }
}
