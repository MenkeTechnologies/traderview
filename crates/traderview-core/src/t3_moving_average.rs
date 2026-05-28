//! T3 Moving Average — Tim Tillson (1998, "Better Moving Averages").
//!
//! Cascade of six EMAs combined with a fixed polynomial in the volume
//! factor `v`. Designed for low lag with smooth response, particularly
//! good around turning points.
//!
//!   e1 = EMA(price, period)
//!   e2 = EMA(e1, period)
//!   e3 = EMA(e2, period)
//!   e4 = EMA(e3, period)
//!   e5 = EMA(e4, period)
//!   e6 = EMA(e5, period)
//!
//!   c1 = −v³
//!   c2 = 3·v² + 3·v³
//!   c3 = −6·v² − 3·v − 3·v³
//!   c4 = 1 + 3·v + v³ + 3·v²
//!
//!   T3 = c1·e6 + c2·e5 + c3·e4 + c4·e3
//!
//! Defaults: period = 5, volume factor v = 0.7 (Tillson's recommended).
//!   - v = 0 → equivalent to a simple EMA
//!   - v = 1 → equivalent to DEMA (Patrick Mulloy)
//!   - v between → mix; v = 0.7 = the "smoother-than-DEMA, less-lag-than-EMA" sweet spot
//!
//! Pure compute.

pub fn compute(
    closes: &[f64],
    period: usize,
    volume_factor: f64,
) -> Vec<Option<f64>> {
    let n = closes.len();
    let mut out = vec![None; n];
    if period < 2 || n < period || !volume_factor.is_finite()
        || !(0.0..=1.0).contains(&volume_factor)
    {
        return out;
    }
    if closes.iter().any(|x| !x.is_finite()) { return out; }
    let e1 = ema(closes, period);
    let e2 = ema_opt(&e1, period);
    let e3 = ema_opt(&e2, period);
    let e4 = ema_opt(&e3, period);
    let e5 = ema_opt(&e4, period);
    let e6 = ema_opt(&e5, period);
    let v = volume_factor;
    let c1 = -v * v * v;
    let c2 = 3.0 * v * v + 3.0 * v * v * v;
    let c3 = -6.0 * v * v - 3.0 * v - 3.0 * v * v * v;
    let c4 = 1.0 + 3.0 * v + v * v * v + 3.0 * v * v;
    for i in 0..n {
        if let (Some(v6), Some(v5), Some(v4), Some(v3)) = (e6[i], e5[i], e4[i], e3[i]) {
            out[i] = Some(c1 * v6 + c2 * v5 + c3 * v4 + c4 * v3);
        }
    }
    out
}

fn ema(series: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = series.len();
    let mut out = vec![None; n];
    if period == 0 || n < period { return out; }
    let seed: f64 = series.iter().take(period).sum::<f64>() / period as f64;
    let k = 2.0 / (period as f64 + 1.0);
    let mut cur = seed;
    out[period - 1] = Some(cur);
    for i in period..n {
        cur = series[i] * k + cur * (1.0 - k);
        out[i] = Some(cur);
    }
    out
}

fn ema_opt(series: &[Option<f64>], period: usize) -> Vec<Option<f64>> {
    let n = series.len();
    let mut out = vec![None; n];
    if period == 0 || n == 0 { return out; }
    // Find first contiguous window of `period` Some values for seed.
    let mut seed_end = None;
    let mut seed_sum = 0.0;
    let mut count = 0_usize;
    for (i, v) in series.iter().enumerate() {
        match v {
            Some(x) => { seed_sum += x; count += 1; }
            None => { seed_sum = 0.0; count = 0; }
        }
        if count == period { seed_end = Some(i); break; }
    }
    let Some(end) = seed_end else { return out; };
    let k = 2.0 / (period as f64 + 1.0);
    let mut cur = seed_sum / period as f64;
    out[end] = Some(cur);
    for i in (end + 1)..n {
        if let Some(v) = series[i] {
            cur = v * k + cur * (1.0 - k);
            out[i] = Some(cur);
        } else {
            out[i] = Some(cur);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_empty() {
        assert!(compute(&[], 5, 0.7).is_empty());
    }

    #[test]
    fn invalid_params_return_all_none() {
        let closes = vec![100.0_f64; 100];
        assert!(compute(&closes, 1, 0.7).iter().all(|x| x.is_none()));
        assert!(compute(&closes, 5, -0.1).iter().all(|x| x.is_none()));
        assert!(compute(&closes, 5, 1.1).iter().all(|x| x.is_none()));
        assert!(compute(&closes, 5, f64::NAN).iter().all(|x| x.is_none()));
    }

    #[test]
    fn shorter_than_six_emas_returns_all_none() {
        let closes = vec![100.0_f64; 10];
        let out = compute(&closes, 5, 0.7);
        // Needs 6·(period-1) bars seeding all 6 EMAs; with period 5, ~30 bars needed.
        // At 10 bars, T3 is all None.
        assert!(out.iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_series_yields_flat_t3() {
        let closes = vec![100.0_f64; 60];
        let out = compute(&closes, 5, 0.7);
        // c1 + c2 + c3 + c4 = 1 by construction → T3 of constant = constant.
        for x in out.iter().skip(28).flatten() {
            assert!((x - 100.0).abs() < 1e-9, "got {x}");
        }
    }

    #[test]
    fn t3_with_v_zero_equivalent_to_third_ema() {
        // v = 0 → c1=c2=c3=0, c4=1 → T3 = e3.
        let closes: Vec<f64> = (0..60).map(|i| 100.0 + i as f64).collect();
        let out = compute(&closes, 5, 0.0);
        // Compare with directly-computed triple-EMA.
        let e1 = ema(&closes, 5);
        let e2 = ema_opt(&e1, 5);
        let e3 = ema_opt(&e2, 5);
        let mut compared = 0;
        for i in 0..60 {
            if let (Some(t3), Some(third)) = (out[i], e3[i]) {
                assert!((t3 - third).abs() < 1e-9, "T3 {} vs E3 {} at i {}", t3, third, i);
                compared += 1;
            }
        }
        assert!(compared > 0, "no comparable indices");
    }

    #[test]
    fn uptrend_t3_eventually_tracks_trend() {
        let closes: Vec<f64> = (0..100).map(|i| 100.0 + i as f64).collect();
        let out = compute(&closes, 5, 0.7);
        let last = out[99].unwrap();
        // T3 should be close to (but lagging) the current value.
        assert!(last > 100.0 && last < 199.0);
        assert!(last > 150.0, "T3 too laggy: {last}");
    }

    #[test]
    fn output_length_matches_input() {
        let closes: Vec<f64> = (0..100).map(|i| 100.0 + (i as f64 * 0.1).sin() * 5.0).collect();
        let out = compute(&closes, 5, 0.7);
        assert_eq!(out.len(), 100);
    }
}
