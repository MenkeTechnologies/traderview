//! Relative Volatility Index (RVI) — Donald Dorsey (1993).
//!
//! Like RSI but uses the standard deviation of high/low prices instead
//! of close-to-close gains/losses:
//!
//!   std_period_t = stdev(high or low, period)
//!   up_t = std_period_t if close_t > close_{t-1} else 0
//!   dn_t = std_period_t if close_t < close_{t-1} else 0
//!   avg_up = Wilder EMA of up
//!   avg_dn = Wilder EMA of dn
//!   RVI = 100 · avg_up / (avg_up + avg_dn)
//!
//! Computed once on high, once on low, and the final RVI is the
//! average of the two (Dorsey's recommendation).
//!
//! Range [0, 100]. Above 60 = strong upside volatility (often
//! coincides with breakouts); below 40 = downside volatility dominant.
//!
//! Pure compute. Companion to other RSI-family indicators
//! (`chande_momentum_oscillator`, `stochastic_rsi`).

pub fn compute(highs: &[f64], lows: &[f64], closes: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = closes.len();
    let mut out = vec![None; n];
    if period < 2 || highs.len() != n || lows.len() != n || n < period + 1 {
        return out;
    }
    if highs.iter().any(|x| !x.is_finite())
        || lows.iter().any(|x| !x.is_finite())
        || closes.iter().any(|x| !x.is_finite())
    {
        return out;
    }
    let rvi_high = rvi_on_series(highs, closes, period);
    let rvi_low = rvi_on_series(lows, closes, period);
    for i in 0..n {
        out[i] = match (rvi_high[i], rvi_low[i]) {
            (Some(h), Some(l)) => Some(0.5 * (h + l)),
            _ => None,
        };
    }
    out
}

fn rvi_on_series(price: &[f64], closes: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = price.len();
    let mut out = vec![None; n];
    let p_f = period as f64;
    // Rolling stdev over period bars.
    let stdev = |i: usize| -> Option<f64> {
        if i + 1 < period {
            return None;
        }
        let win = &price[i + 1 - period..=i];
        let mean: f64 = win.iter().sum::<f64>() / p_f;
        let var: f64 = win.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / p_f;
        Some(var.max(0.0).sqrt())
    };
    // Seed Wilder EMA on first complete window of period after the
    // first stdev value can be computed (i.e. once we have `period`
    // up/dn values).
    let mut avg_up = 0.0_f64;
    let mut avg_dn = 0.0_f64;
    let mut seeded = false;
    let mut seed_collected = 0_usize;
    let mut seed_up = 0.0_f64;
    let mut seed_dn = 0.0_f64;
    for i in 1..n {
        let s = match stdev(i) {
            Some(v) => v,
            None => continue,
        };
        let up = if closes[i] > closes[i - 1] { s } else { 0.0 };
        let dn = if closes[i] < closes[i - 1] { s } else { 0.0 };
        if !seeded {
            seed_up += up;
            seed_dn += dn;
            seed_collected += 1;
            if seed_collected == period {
                avg_up = seed_up / p_f;
                avg_dn = seed_dn / p_f;
                seeded = true;
                let denom = avg_up + avg_dn;
                out[i] = if denom > 0.0 {
                    Some(100.0 * avg_up / denom)
                } else {
                    Some(50.0)
                };
            }
        } else {
            avg_up = (avg_up * (p_f - 1.0) + up) / p_f;
            avg_dn = (avg_dn * (p_f - 1.0) + dn) / p_f;
            let denom = avg_up + avg_dn;
            out[i] = if denom > 0.0 {
                Some(100.0 * avg_up / denom)
            } else {
                Some(50.0)
            };
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_all_none() {
        let h = vec![101.0_f64; 30];
        let l = vec![99.0_f64; 30];
        let c = vec![100.0_f64; 30];
        assert!(compute(&h, &l, &c, 1).iter().all(|x| x.is_none()));
        assert!(compute(&h[..5], &l, &c, 10).iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_all_none() {
        let h = vec![101.0_f64; 30];
        let mut l = vec![99.0_f64; 30];
        l[5] = f64::NAN;
        let c = vec![100.0_f64; 30];
        assert!(compute(&h, &l, &c, 10).iter().all(|x| x.is_none()));
    }

    #[test]
    fn output_length_matches_input() {
        let h: Vec<f64> = (0..50).map(|i| 101.0 + (i as f64).sin()).collect();
        let l: Vec<f64> = (0..50).map(|i| 99.0 + (i as f64).cos()).collect();
        let c: Vec<f64> = (0..50)
            .map(|i| 100.0 + (i as f64 * 0.1).sin() * 0.5)
            .collect();
        let r = compute(&h, &l, &c, 10);
        assert_eq!(r.len(), 50);
    }

    #[test]
    fn rvi_in_unit_range_zero_to_hundred() {
        let mut state: u64 = 42;
        let h: Vec<f64> = (0..100)
            .map(|_| {
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                101.0 + ((state >> 32) as f64 / u32::MAX as f64) * 2.0
            })
            .collect();
        let l: Vec<f64> = (0..100).map(|i| h[i] - 2.0).collect();
        let c: Vec<f64> = (0..100)
            .map(|i| 100.0 + (i as f64 * 0.1).sin() * 0.5)
            .collect();
        let r = compute(&h, &l, &c, 14);
        for v in r.iter().flatten() {
            assert!((0.0..=100.0).contains(v));
        }
    }

    #[test]
    fn all_up_closes_yield_high_rvi() {
        // Every close > prior → all volatility classified as upside.
        let h: Vec<f64> = (0..50).map(|i| 101.0 + i as f64).collect();
        let l: Vec<f64> = (0..50).map(|i| 99.0 + i as f64).collect();
        let c: Vec<f64> = (0..50).map(|i| 100.0 + i as f64).collect();
        let r = compute(&h, &l, &c, 14);
        let last = r[49].unwrap();
        assert!(
            last > 95.0,
            "all up-closes should yield RVI near 100, got {last}"
        );
    }

    #[test]
    fn all_down_closes_yield_low_rvi() {
        let h: Vec<f64> = (0..50).map(|i| 101.0 - i as f64 * 0.1).collect();
        let l: Vec<f64> = (0..50).map(|i| 99.0 - i as f64 * 0.1).collect();
        let c: Vec<f64> = (0..50).map(|i| 100.0 - i as f64).collect();
        let r = compute(&h, &l, &c, 14);
        let last = r[49].unwrap();
        assert!(
            last < 5.0,
            "all down-closes should yield RVI near 0, got {last}"
        );
    }
}
