//! Triangular Moving Average (TMA).
//!
//! Double-smoothed simple moving average: SMA applied twice with
//! half-period (rounded up):
//!
//!   inner = SMA(close, ceil(N/2))
//!   TMA   = SMA(inner, floor(N/2) + 1)
//!
//! Equivalent to convolving with a triangular kernel of weights
//! peaking in the middle of the lookback window. Produces a much
//! smoother line than SMA at the cost of greater lag.
//!
//! Default period 20. Pure compute.
//!
//! Companion to `dema`, `tema`, `hull_ma`, `kama`, `vidya`.

pub fn compute(closes: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = closes.len();
    let mut out = vec![None; n];
    if period < 3 || n < period {
        return out;
    }
    if closes.iter().any(|x| !x.is_finite()) {
        return out;
    }
    let inner_len = period.div_ceil(2);
    let outer_len = period / 2 + 1;
    let inner = sma(closes, inner_len);
    // Apply SMA to the inner series (Option<f64>).
    let outer_f = outer_len as f64;
    let mut count = 0_usize;
    let mut sum = 0.0_f64;
    let mut buf: std::collections::VecDeque<f64> =
        std::collections::VecDeque::with_capacity(outer_len);
    for (i, v) in inner.iter().enumerate() {
        match v {
            Some(x) => {
                buf.push_back(*x);
                sum += x;
                count += 1;
                if count > outer_len {
                    if let Some(old) = buf.pop_front() {
                        sum -= old;
                    }
                    count -= 1;
                }
                if count == outer_len {
                    out[i] = Some(sum / outer_f);
                }
            }
            None => {
                buf.clear();
                sum = 0.0;
                count = 0;
            }
        }
    }
    out
}

fn sma(series: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = series.len();
    let mut out = vec![None; n];
    if period == 0 || n < period {
        return out;
    }
    let p_f = period as f64;
    let mut sum: f64 = series[..period].iter().sum();
    out[period - 1] = Some(sum / p_f);
    for i in period..n {
        sum += series[i] - series[i - period];
        out[i] = Some(sum / p_f);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_all_none() {
        let c = vec![100.0_f64; 50];
        assert!(compute(&c, 2).iter().all(|x| x.is_none()));
        assert!(compute(&c[..5], 20).iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_all_none() {
        let mut c = vec![100.0_f64; 50];
        c[5] = f64::NAN;
        assert!(compute(&c, 20).iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_series_yields_constant_tma() {
        let c = vec![100.0_f64; 50];
        let r = compute(&c, 20);
        for v in r.iter().flatten() {
            assert!((v - 100.0).abs() < 1e-9);
        }
    }

    #[test]
    fn linear_trend_preserved_with_lag() {
        // For an unbounded linear trend, repeated SMA preserves slope.
        let c: Vec<f64> = (0..100).map(|i| i as f64).collect();
        let r = compute(&c, 20);
        // Compare two consecutive TMA values → slope ≈ 1.
        let a = r[80].unwrap();
        let b = r[81].unwrap();
        assert!((b - a - 1.0).abs() < 1e-9);
    }

    #[test]
    fn tma_within_input_range() {
        // Sanity: smoothed value can't exit the input min/max bracket.
        let c: Vec<f64> = (0..100)
            .map(|i| 100.0 + (i as f64 * 0.2).sin() * 5.0)
            .collect();
        let r = compute(&c, 20);
        let in_min = c.iter().cloned().fold(f64::INFINITY, f64::min);
        let in_max = c.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        for v in r.iter().flatten() {
            assert!(*v >= in_min - 1e-9 && *v <= in_max + 1e-9);
        }
    }

    #[test]
    fn smoothing_reduces_high_frequency_noise() {
        let mut state: u64 = 42;
        let c: Vec<f64> = (0..200)
            .map(|_| {
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                100.0 + ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 10.0
            })
            .collect();
        let tma = compute(&c, 20);
        // Sample stdev of TMA values < input.
        let vals: Vec<f64> = tma.iter().flatten().copied().collect();
        let mean: f64 = vals.iter().sum::<f64>() / vals.len() as f64;
        let var_tma: f64 = vals.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / vals.len() as f64;
        let in_mean: f64 = c.iter().sum::<f64>() / c.len() as f64;
        let var_in: f64 = c.iter().map(|x| (x - in_mean).powi(2)).sum::<f64>() / c.len() as f64;
        assert!(var_tma < var_in);
    }

    #[test]
    fn output_length_matches_input() {
        let c = vec![100.0_f64; 50];
        assert_eq!(compute(&c, 20).len(), 50);
    }
}
