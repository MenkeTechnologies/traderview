//! DeMarker — Tom DeMark.
//!
//!   demax_t = max(high_t − high_{t−1}, 0)
//!   demin_t = max(low_{t−1} − low_t,   0)
//!   numer   = SMA(demax, period)
//!   denom   = SMA(demax, period) + SMA(demin, period)
//!   DeM     = numer / denom              ∈ [0, 1]
//!
//! Reading: >0.7 overbought (likely reversal), <0.3 oversold (likely
//! bounce). Less spiky than RSI on thin / illiquid instruments because
//! it uses raw high/low extension rather than close-to-close change.
//!
//! Pure compute.

pub fn compute(highs: &[f64], lows: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = highs.len();
    let mut out = vec![None; n];
    if period == 0 || highs.len() != lows.len() || n < period.saturating_add(1) {
        return out;
    }
    let mut demax = vec![0.0_f64; n];
    let mut demin = vec![0.0_f64; n];
    for i in 1..n {
        if highs[i].is_finite() && highs[i - 1].is_finite() {
            demax[i] = (highs[i] - highs[i - 1]).max(0.0);
        }
        if lows[i].is_finite() && lows[i - 1].is_finite() {
            demin[i] = (lows[i - 1] - lows[i]).max(0.0);
        }
    }
    for i in period..n {
        let s_max: f64 = demax[i + 1 - period..=i].iter().sum::<f64>() / period as f64;
        let s_min: f64 = demin[i + 1 - period..=i].iter().sum::<f64>() / period as f64;
        let denom = s_max + s_min;
        if denom > 0.0 {
            let v = s_max / denom;
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
        assert!(compute(&[], &[], 14).is_empty());
    }

    #[test]
    fn length_mismatch_returns_all_none() {
        let h = vec![100.0; 20];
        let l = vec![99.0; 10];
        let out = compute(&h, &l, 14);
        assert!(out.iter().all(|x| x.is_none()));
    }

    #[test]
    fn period_zero_returns_all_none() {
        let h = vec![100.0; 30];
        let l = vec![99.0; 30];
        assert!(compute(&h, &l, 0).iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_series_demarker_undefined_or_neutral() {
        // No movement → both demax and demin = 0 → denom = 0 → None.
        let h = vec![100.0; 30];
        let l = vec![99.0; 30];
        let out = compute(&h, &l, 14);
        assert!(out.iter().all(|x| x.is_none()), "flat series → no signal");
    }

    #[test]
    fn rising_highs_yield_demarker_near_one() {
        let h: Vec<f64> = (1..=40).map(|i| 100.0 + i as f64).collect();
        let l: Vec<f64> = h.iter().map(|x| x - 1.0).collect();
        let out = compute(&h, &l, 14);
        let last = out[39].expect("populated");
        // Only demax accrues — demin stays 0 (lows rising means demin=0).
        // So ratio = s_max / (s_max + 0) = 1.0.
        assert!(
            (last - 1.0).abs() < 1e-9,
            "all rising → DeM = 1, got {last}"
        );
    }

    #[test]
    fn falling_lows_yield_demarker_zero() {
        let h: Vec<f64> = (1..=40).map(|i| 200.0 - i as f64).collect();
        let l: Vec<f64> = h.iter().map(|x| x - 1.0).collect();
        let out = compute(&h, &l, 14);
        let last = out[39].expect("populated");
        // Only demin accrues; demax=0 → 0/(0+x) = 0.
        assert!(last.abs() < 1e-9, "all falling → DeM = 0, got {last}");
    }

    #[test]
    fn output_always_in_range_0_1() {
        let h: Vec<f64> = (0..200)
            .map(|i| 100.0 + (i as f64 * 0.3).sin() * 5.0)
            .collect();
        let l: Vec<f64> = h.iter().map(|x| x - 1.0).collect();
        let out = compute(&h, &l, 14);
        for x in out.iter().flatten() {
            assert!((0.0..=1.0).contains(x), "DeM out of [0,1]: {x}");
        }
    }

    #[test]
    fn huge_period_no_panic() {
        let h = vec![100.0; 5];
        let l = vec![99.0; 5];
        assert!(compute(&h, &l, usize::MAX).iter().all(|x| x.is_none()));
    }
}
