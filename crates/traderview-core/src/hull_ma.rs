//! Hull Moving Average — Alan Hull (2005).
//!
//! HMA reduces lag dramatically compared to SMA/EMA while keeping a smooth
//! line. Formula:
//!   `HMA(n) = WMA(2 × WMA(n/2) − WMA(n), sqrt(n))`
//!
//! where WMA is a linearly weighted moving average. The (n/2 - n) double
//! difference is the lag-reduction trick; the final sqrt(n) WMA smooths
//! the resulting noisy line.
//!
//! Pure compute. Length-aligned output: leading positions before the
//! WMA(sqrt(n)) warmup are `None`.

pub fn compute(closes: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = closes.len();
    let mut out = vec![None; n];
    // Guard against `period == 0`, oversized period, and an integer-sqrt
    // that would later overflow downstream arithmetic.
    if period < 2 || n < period {
        return out;
    }
    let half = period / 2;
    let sqrt_n = (period as f64).sqrt().round().max(1.0) as usize;
    if half == 0 || sqrt_n == 0 {
        return out;
    }
    let wma_half = wma(closes, half);
    let wma_full = wma(closes, period);
    // Inner "raw HMA" series: 2 × WMA(n/2) − WMA(n).
    let raw: Vec<Option<f64>> = wma_half
        .iter()
        .zip(wma_full.iter())
        .map(|(h, f)| match (h, f) {
            (Some(h), Some(f)) => Some(2.0 * h - f),
            _ => None,
        })
        .collect();
    // Final WMA(sqrt(n)) over the raw series — needs `sqrt_n` consecutive
    // Somes. Do an Option-aware sliding window.
    let need = sqrt_n;
    for i in (need - 1)..n {
        let window = &raw[i + 1 - need..=i];
        let all_some = window.iter().all(|x| x.is_some());
        if !all_some {
            continue;
        }
        // Triangular weights 1, 2, 3, ..., need. Sum of weights = need*(need+1)/2.
        let mut num = 0.0;
        let mut wsum = 0.0;
        for (k, v) in window.iter().enumerate() {
            let w = (k + 1) as f64;
            num += v.unwrap() * w;
            wsum += w;
        }
        out[i] = Some(num / wsum);
    }
    out
}

fn wma(values: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = values.len();
    let mut out = vec![None; n];
    if period == 0 || n < period {
        return out;
    }
    let wsum: f64 = (1..=period).map(|k| k as f64).sum();
    for i in (period - 1)..n {
        let window = &values[i + 1 - period..=i];
        let mut num = 0.0;
        for (k, &v) in window.iter().enumerate() {
            num += v * (k + 1) as f64;
        }
        out[i] = Some(num / wsum);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_empty_length_aligned() {
        let out = compute(&[], 9);
        assert!(out.is_empty());
    }

    #[test]
    fn series_shorter_than_period_all_none() {
        let out = compute(&[1.0, 2.0, 3.0], 9);
        assert_eq!(out.len(), 3);
        assert!(out.iter().all(|x| x.is_none()));
    }

    #[test]
    fn period_zero_or_one_returns_all_none() {
        let v = vec![1.0; 10];
        for p in [0, 1] {
            let out = compute(&v, p);
            assert!(out.iter().all(|x| x.is_none()), "period={p}");
        }
    }

    #[test]
    fn flat_series_hma_equals_input() {
        let v = vec![100.0; 30];
        let out = compute(&v, 9);
        // Once warmed up, HMA of a flat series equals the constant.
        let last = out[29].expect("populated");
        assert!((last - 100.0).abs() < 1e-9);
    }

    #[test]
    fn trending_series_hma_tracks_trend_with_low_lag() {
        // Monotonic uptrend — HMA should be very close to current price.
        let v: Vec<f64> = (1..=40).map(|i| 100.0 + i as f64).collect();
        let out = compute(&v, 9);
        let last = out[39].expect("populated");
        // HMA hugs current price tightly on a clean trend.
        assert!((last - v[39]).abs() < 2.0, "HMA={last} vs price={}", v[39]);
    }

    #[test]
    fn huge_period_safe_returns_all_none_no_panic() {
        let v = vec![1.0; 10];
        let out = compute(&v, usize::MAX);
        assert!(out.iter().all(|x| x.is_none()));
    }
}
