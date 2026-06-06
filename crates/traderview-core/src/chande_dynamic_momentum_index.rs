//! Chande Dynamic Momentum Index (DMI / DyMOI) — Tushar Chande (1993).
//!
//! Volatility-adapted RSI: increases the RSI lookback in quiet markets
//! (less responsive, more smoothing) and decreases it in volatile ones
//! (more responsive). Window length per bar:
//!
//!   stdev_t  = sample stdev of close over std_period bars
//!   avg_std  = SMA of stdev over std_period bars
//!   vi_t     = stdev_t / avg_std       (volatility index, dimensionless)
//!   td_t     = round(td_const / vi_t).clamp(td_min, td_max)
//!     (td_const = base RSI period, default 14; td_min = 5; td_max = 30)
//!   DMI_t    = Wilder RSI of close with lookback td_t
//!
//! Output 0..100 like standard RSI. Pure compute.
//!
//! Companion to `connors_rsi`, `stochastic_rsi`, `relative_volatility_index`.

pub fn compute(
    closes: &[f64],
    td_const: usize,
    std_period: usize,
    td_min: usize,
    td_max: usize,
) -> Vec<Option<f64>> {
    let n = closes.len();
    let mut out = vec![None; n];
    if td_const < 2
        || std_period < 2
        || td_min < 2
        || td_max < td_min
        || td_max < td_const
        || n < 2 * std_period + td_max
    {
        return out;
    }
    if closes.iter().any(|x| !x.is_finite()) {
        return out;
    }
    // Rolling sample stdev (over std_period bars).
    let mut std_series = vec![None; n];
    let p_f = std_period as f64;
    for i in (std_period - 1)..n {
        let win = &closes[i + 1 - std_period..=i];
        let mean: f64 = win.iter().sum::<f64>() / p_f;
        let var: f64 = win.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / p_f;
        std_series[i] = Some(var.max(0.0).sqrt());
    }
    // SMA of stdev → average volatility.
    let avg_std = sma_opt(&std_series, std_period);
    // For each bar where avg_std exists and is positive, derive adaptive
    // RSI period and compute Wilder RSI freshly.
    let td_c = td_const as f64;
    for (i, slot) in out.iter_mut().enumerate() {
        let s = match std_series[i] {
            Some(v) => v,
            None => continue,
        };
        let a = match avg_std[i] {
            Some(v) => v,
            None => continue,
        };
        if a <= 0.0 || s <= 0.0 {
            continue;
        }
        let vi = s / a;
        let td = (td_c / vi).round().clamp(td_min as f64, td_max as f64) as usize;
        // Compute Wilder RSI(td) at bar i — requires i ≥ td. We have
        // i ≥ 2·std_period - 1 by gating above which exceeds td_max ≥ td.
        if i < td {
            continue;
        }
        if let Some(rsi) = wilder_rsi_at(closes, i, td) {
            *slot = Some(rsi);
        }
    }
    out
}

/// Wilder RSI evaluated at index `i` with period `td`.
fn wilder_rsi_at(closes: &[f64], i: usize, td: usize) -> Option<f64> {
    if i < td {
        return None;
    }
    let p_f = td as f64;
    let mut sum_gain = 0.0_f64;
    let mut sum_loss = 0.0_f64;
    let start = i + 1 - td;
    for k in (start + 1)..=i {
        let diff = closes[k] - closes[k - 1];
        if diff > 0.0 {
            sum_gain += diff;
        } else {
            sum_loss -= diff;
        }
    }
    // Note: this uses simple-average smoothing inside the window
    // (matches DMI variant as published by Chande for variable-period
    // recomputation; classic Wilder smoothing would require unbounded
    // history with a single period).
    let avg_gain = sum_gain / p_f;
    let avg_loss = sum_loss / p_f;
    if avg_loss <= 0.0 {
        Some(if avg_gain <= 0.0 { 50.0 } else { 100.0 })
    } else {
        let rs = avg_gain / avg_loss;
        Some(100.0 - 100.0 / (1.0 + rs))
    }
}

fn sma_opt(series: &[Option<f64>], period: usize) -> Vec<Option<f64>> {
    let n = series.len();
    let mut out = vec![None; n];
    if period == 0 || n < period {
        return out;
    }
    let p_f = period as f64;
    for i in (period - 1)..n {
        let win = &series[i + 1 - period..=i];
        if win.iter().any(|x| x.is_none()) {
            continue;
        }
        let s: f64 = win.iter().filter_map(|x| *x).sum();
        out[i] = Some(s / p_f);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_empty() {
        let c = vec![100.0_f64; 200];
        assert!(compute(&c, 1, 5, 5, 30).iter().all(|x| x.is_none()));
        assert!(compute(&c, 14, 5, 30, 5).iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_empty() {
        let mut c = vec![100.0_f64; 200];
        c[5] = f64::NAN;
        assert!(compute(&c, 14, 5, 5, 30).iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_market_yields_no_volatility_so_none() {
        // Flat market → stdev = 0 → avg_std = 0 → no output.
        let c = vec![100.0_f64; 200];
        let r = compute(&c, 14, 5, 5, 30);
        for v in &r {
            assert!(v.is_none());
        }
    }

    #[test]
    fn uptrend_yields_high_dmi() {
        let c: Vec<f64> = (0..200).map(|i| 100.0 + i as f64).collect();
        let r = compute(&c, 14, 5, 5, 30);
        let last = r.iter().rev().find_map(|x| *x).unwrap();
        assert!(
            last > 80.0,
            "strong uptrend should yield DMI > 80, got {last}"
        );
    }

    #[test]
    fn downtrend_yields_low_dmi() {
        let c: Vec<f64> = (0..200).map(|i| 300.0 - i as f64).collect();
        let r = compute(&c, 14, 5, 5, 30);
        let last = r.iter().rev().find_map(|x| *x).unwrap();
        assert!(last < 20.0);
    }

    #[test]
    fn output_in_zero_hundred_range() {
        let mut state: u64 = 42;
        let c: Vec<f64> = (0..400)
            .map(|i| {
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let r = (state >> 32) as u32 as f64 / u32::MAX as f64;
                100.0 + i as f64 * 0.1 + (r - 0.5) * 5.0
            })
            .collect();
        let r = compute(&c, 14, 5, 5, 30);
        for v in r.iter().flatten() {
            assert!((0.0..=100.0).contains(v));
        }
    }

    #[test]
    fn output_length_matches_input() {
        let c = vec![100.0_f64; 200];
        assert_eq!(compute(&c, 14, 5, 5, 30).len(), 200);
    }
}
