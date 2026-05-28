//! KAMA — Kaufman Adaptive Moving Average (Perry Kaufman, 1995).
//!
//! Adapts smoothing speed to noise: trends fast, chops slow.
//!
//!   ER = (close − close[−n]) / sum(|close[i] − close[i−1]|, n)
//!   fast_α = 2/(fast_n+1)         (default fast_n=2 → 0.6667)
//!   slow_α = 2/(slow_n+1)         (default slow_n=30 → 0.0645)
//!   SC = (|ER| · (fast_α − slow_α) + slow_α)²
//!   KAMA_t = KAMA_{t−1} + SC · (close_t − KAMA_{t−1})
//!
//! Standard config: n=10, fast=2, slow=30. Pure compute.

pub fn compute(
    closes: &[f64],
    er_period: usize,
    fast_period: usize,
    slow_period: usize,
) -> Vec<Option<f64>> {
    let n = closes.len();
    let mut out = vec![None; n];
    if er_period == 0 || fast_period == 0 || slow_period == 0 || n <= er_period {
        return out;
    }
    let fast_alpha = 2.0 / (fast_period as f64 + 1.0);
    let slow_alpha = 2.0 / (slow_period as f64 + 1.0);
    // Seed at index er_period with close[er_period] (Kaufman's convention).
    let mut prev = closes[er_period];
    out[er_period] = Some(prev);
    for i in (er_period + 1)..n {
        let direction = (closes[i] - closes[i - er_period]).abs();
        let volatility: f64 = (i - er_period + 1..=i)
            .map(|j| (closes[j] - closes[j - 1]).abs())
            .sum();
        let er = if volatility > 0.0 {
            direction / volatility
        } else {
            0.0
        };
        let sc = (er * (fast_alpha - slow_alpha) + slow_alpha).powi(2);
        let val = prev + sc * (closes[i] - prev);
        if val.is_finite() {
            out[i] = Some(val);
            prev = val;
        } else {
            // Hostile inputs (NaN closes) could poison the recursion — skip
            // forward without updating `prev` so subsequent valid points
            // continue from the last good value.
            out[i] = None;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_empty() {
        assert!(compute(&[], 10, 2, 30).is_empty());
    }

    #[test]
    fn period_zero_returns_all_none() {
        let v = vec![1.0; 50];
        for (er, fp, sp) in [(0, 2, 30), (10, 0, 30), (10, 2, 0)] {
            let out = compute(&v, er, fp, sp);
            assert!(out.iter().all(|x| x.is_none()), "er={er} fp={fp} sp={sp}");
        }
    }

    #[test]
    fn flat_series_kama_holds_value() {
        // Flat → ER undefined (vol=0) → SC = slow_α² → KAMA barely moves
        // off its seed; on a perfectly flat series it stays at seed.
        let v = vec![100.0; 50];
        let out = compute(&v, 10, 2, 30);
        let last = out[49].expect("populated");
        assert!((last - 100.0).abs() < 1e-9);
    }

    #[test]
    fn strong_trend_kama_tracks_price() {
        // Clean uptrend → ER ≈ 1 → SC ≈ fast_α² → KAMA adapts quickly.
        let v: Vec<f64> = (1..=80).map(|i| 100.0 + i as f64).collect();
        let out = compute(&v, 10, 2, 30);
        let last = out[79].expect("populated");
        // KAMA on a clean trend converges close to the price.
        assert!((last - v[79]).abs() < 5.0, "KAMA={last} vs price={}", v[79]);
    }

    #[test]
    fn choppy_series_kama_smooths() {
        // Alternating up/down → ER near 0 → SC ≈ slow_α² → KAMA is sticky.
        let v: Vec<f64> = (0..60)
            .map(|i| if i % 2 == 0 { 100.0 } else { 101.0 })
            .collect();
        let out = compute(&v, 10, 2, 30);
        let last = out[59].expect("populated");
        // On chop, KAMA stays near the midpoint (~100.5), not at the latest tick.
        assert!(last > 100.0 && last < 101.5, "KAMA={last}");
    }

    #[test]
    fn huge_periods_no_panic() {
        let v = vec![1.0; 5];
        let out = compute(&v, usize::MAX, 2, 30);
        assert!(out.iter().all(|x| x.is_none()));
    }
}
