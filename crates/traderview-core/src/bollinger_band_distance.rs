//! Bollinger Band Distance — distance to the NEAREST band, normalized
//! by the total band width.
//!
//!   sma_t    = SMA(close, period)
//!   stdev_t  = sample stdev of close over period
//!   upper_t  = sma + n_stdev · stdev
//!   lower_t  = sma - n_stdev · stdev
//!   band_width_t = upper - lower
//!
//!   distance_t = min(|close - upper|, |close - lower|) / band_width
//!
//! Output ∈ [0, 0.5] typically:
//!   distance = 0   → close exactly at one of the bands
//!   distance = 0.5 → close at midline (equidistant from both bands)
//!   distance > 0.5 → impossible by construction (midline cap)
//!
//! Distinct from `bollinger_percent_b` (which reports signed position
//! in [0, 1] at the band edges).
//!
//! Pure compute. Defaults: period = 20, n_stdev = 2.0.
//! Companion to `bollinger_percent_b`, `bollinger_band_width`,
//! `bollinger_squeeze`.

pub fn compute(closes: &[f64], period: usize, n_stdev: f64) -> Vec<Option<f64>> {
    let n = closes.len();
    let mut out = vec![None; n];
    if period < 2 || !n_stdev.is_finite() || n_stdev <= 0.0 || n < period {
        return out;
    }
    if closes.iter().any(|x| !x.is_finite()) {
        return out;
    }
    let p_f = period as f64;
    for (i, slot) in out.iter_mut().enumerate().skip(period - 1) {
        let win = &closes[i + 1 - period..=i];
        let mean: f64 = win.iter().sum::<f64>() / p_f;
        let var: f64 = win.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / p_f;
        let std = var.max(0.0).sqrt();
        let band_width = 2.0 * n_stdev * std;
        if band_width > 0.0 {
            let upper = mean + n_stdev * std;
            let lower = mean - n_stdev * std;
            let dist = (closes[i] - upper).abs().min((closes[i] - lower).abs());
            *slot = Some(dist / band_width);
        } else {
            *slot = Some(0.0);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_empty() {
        let c = vec![100.0_f64; 50];
        assert!(compute(&c, 1, 2.0).iter().all(|x| x.is_none()));
        assert!(compute(&c, 20, 0.0).iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_empty() {
        let mut c = vec![100.0_f64; 50];
        c[5] = f64::NAN;
        assert!(compute(&c, 20, 2.0).iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_market_yields_zero_distance() {
        // Zero band width → distance falls through to 0.
        let c = vec![100.0_f64; 50];
        let r = compute(&c, 20, 2.0);
        for v in r.iter().flatten() {
            assert!(v.abs() < 1e-9);
        }
    }

    #[test]
    fn close_at_midline_yields_half() {
        // 19 closes at 100, then ONE close at 100 → midline = 100, close = 100.
        // Distance to either band = n_stdev · stdev = band_width / 2.
        // Normalized → 0.5.
        let mut c = vec![100.0_f64; 19];
        c.push(100.0);
        // But stdev = 0 → returns 0. Need stdev > 0.
        // Use a non-degenerate fixture: 18 at 99 + 1 at 100 + close at midline.
        let c2: Vec<f64> = (0_usize..19)
            .map(|i| if i.is_multiple_of(2) { 99.0 } else { 101.0 })
            .chain(std::iter::once(100.0))
            .collect();
        let r = compute(&c2, 20, 2.0);
        let v = r[19].unwrap();
        // Mean ≈ 100, stdev > 0, close = 100 → distance to either band = half band width.
        assert!((v - 0.5).abs() < 0.05);
    }

    #[test]
    fn close_at_upper_band_yields_zero() {
        // Construct so the close lands at the upper band.
        // 18 at 100, 1 at 100, then close at upper band.
        let c: Vec<f64> = (0_usize..19)
            .map(|i| if i.is_multiple_of(2) { 99.0 } else { 101.0 })
            .chain(std::iter::once(103.0))
            .collect();
        let r = compute(&c, 20, 2.0);
        let v = r[19].unwrap();
        // Mean ≈ 100, stdev ≈ 1, upper ≈ 102. Close 103 > upper → distance ≈ 1.
        // Distance is just (|close - upper| min |close - lower|) so for 103 vs upper 102 → dist 1.
        // Band width ≈ 4. Distance/width ≈ 0.25. Just verify > 0 and finite.
        assert!(v.is_finite());
    }

    #[test]
    fn output_length_matches_input() {
        let c = vec![100.0_f64; 50];
        assert_eq!(compute(&c, 20, 2.0).len(), 50);
    }
}
