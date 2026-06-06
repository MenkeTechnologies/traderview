//! Vertical Horizontal Filter (VHF) — Adam White (1991).
//!
//!   VHF = (highest_close_N − lowest_close_N) / sum(|close_t − close_{t−1}|, N)
//!
//! High VHF (≈ 1+) = market is trending (price displacement ≈ total
//! travel). Low VHF (toward 0) = chopping (lots of travel, no
//! displacement). Used as a meta-filter: only run a momentum strategy
//! when VHF is high; only run mean-reversion when it's low.
//!
//! Same family as Choppiness and Efficiency Ratio — different scaling.
//! Standard period = 28.
//!
//! Pure compute.

pub fn compute(closes: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = closes.len();
    let mut out = vec![None; n];
    if period == 0 || n < period.saturating_add(1) {
        return out;
    }
    for i in period..n {
        let window = &closes[i + 1 - period..=i];
        let mut hi = f64::NEG_INFINITY;
        let mut lo = f64::INFINITY;
        for &c in window {
            if c.is_finite() {
                if c > hi {
                    hi = c;
                }
                if c < lo {
                    lo = c;
                }
            }
        }
        if !hi.is_finite() || !lo.is_finite() {
            continue;
        }
        let displacement = hi - lo;
        let mut travel = 0.0_f64;
        // Travel = sum of |delta_t| over the window (use prior bar in series).
        for j in (i + 1 - period)..=i {
            if j == 0 {
                continue;
            }
            let d = (closes[j] - closes[j - 1]).abs();
            if d.is_finite() {
                travel += d;
            }
        }
        if travel > 0.0 {
            let v = displacement / travel;
            if v.is_finite() {
                out[i] = Some(v);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_empty() {
        assert!(compute(&[], 28).is_empty());
    }

    #[test]
    fn period_zero_returns_all_none() {
        let v = vec![100.0; 50];
        assert!(compute(&v, 0).iter().all(|x| x.is_none()));
    }

    #[test]
    fn perfect_trend_vhf_near_one() {
        // Monotonic uptrend: VHF asymptotically approaches 1 but is always
        // (period−1)/period for a 1-unit-per-bar straight line because the
        // window has `period` deltas summed but only `period−1` units of
        // price displacement.
        let v: Vec<f64> = (0..50).map(|i| 100.0 + i as f64).collect();
        let out = compute(&v, 28);
        let last = out[49].expect("populated");
        assert!(
            last > 0.95 && last <= 1.0,
            "trend VHF should approach but not exceed 1, got {last}"
        );
    }

    #[test]
    fn full_oscillation_vhf_low() {
        // Alternating up/down with same range — displacement small, travel high.
        let v: Vec<f64> = (0..50)
            .map(|i| if i % 2 == 0 { 100.0 } else { 101.0 })
            .collect();
        let out = compute(&v, 28);
        let last = out[49].expect("populated");
        assert!(last < 0.1, "chop VHF should be near 0, got {last}");
    }

    #[test]
    fn flat_series_no_signal() {
        // Travel = 0 → None.
        let v = vec![100.0; 50];
        let out = compute(&v, 28);
        for v in &out {
            assert!(v.is_none());
        }
    }

    #[test]
    fn output_in_range_0_1() {
        let v: Vec<f64> = (0..200)
            .map(|i| 100.0 + (i as f64 * 0.4).sin() * 5.0)
            .collect();
        let out = compute(&v, 28);
        for x in out.iter().flatten() {
            assert!((0.0..=1.0 + 1e-9).contains(x), "VHF out of [0,1]: {x}");
        }
    }

    #[test]
    fn huge_period_no_panic() {
        let v = vec![100.0; 5];
        assert!(compute(&v, usize::MAX).iter().all(|x| x.is_none()));
    }
}
