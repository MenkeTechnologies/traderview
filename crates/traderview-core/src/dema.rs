//! DEMA — Double Exponential Moving Average (Patrick Mulloy, 1994).
//!
//!   DEMA = 2·EMA(price, n) − EMA(EMA(price, n), n)
//!
//! Less lag than single EMA without doing the full triple smoothing of
//! TEMA. Standard "lag-reduction" overlay for trend traders.
//!
//! Pure compute.

pub fn compute(closes: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = closes.len();
    let mut out = vec![None; n];
    if period == 0 || n < period {
        return out;
    }
    let ema1 = ema(closes, period);
    let ema2 = ema_optional(&ema1, period);
    for i in 0..n {
        if let (Some(a), Some(b)) = (ema1[i], ema2[i]) {
            out[i] = Some(2.0 * a - b);
        }
    }
    out
}

fn ema(values: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = values.len();
    let mut out = vec![None; n];
    if period == 0 || n < period {
        return out;
    }
    let alpha = 2.0 / (period as f64 + 1.0);
    let seed: f64 = values[..period].iter().sum::<f64>() / period as f64;
    out[period - 1] = Some(seed);
    let mut prev = seed;
    for i in period..n {
        prev = alpha * values[i] + (1.0 - alpha) * prev;
        out[i] = Some(prev);
    }
    out
}

fn ema_optional(values: &[Option<f64>], period: usize) -> Vec<Option<f64>> {
    let n = values.len();
    let mut out = vec![None; n];
    if period == 0 || n < period {
        return out;
    }
    let mut start: Option<usize> = None;
    let mut run = 0;
    for (i, v) in values.iter().enumerate() {
        if v.is_some() {
            run += 1;
            if run >= period {
                start = Some(i);
                break;
            }
        } else {
            run = 0;
        }
    }
    let _ = n;
    let Some(s) = start else { return out };
    let alpha = 2.0 / (period as f64 + 1.0);
    let seed: f64 = values[s + 1 - period..=s]
        .iter()
        .map(|x| x.unwrap())
        .sum::<f64>()
        / period as f64;
    out[s] = Some(seed);
    let mut prev = seed;
    for i in (s + 1)..n {
        if let Some(v) = values[i] {
            prev = alpha * v + (1.0 - alpha) * prev;
            out[i] = Some(prev);
        } else {
            break;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_empty() {
        assert!(compute(&[], 10).is_empty());
    }

    #[test]
    fn period_zero_safe() {
        let v = vec![1.0; 20];
        assert!(compute(&v, 0).iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_series_dema_equals_constant() {
        // 2c − c = c.
        let v = vec![100.0; 50];
        let out = compute(&v, 9);
        let last = out[49].expect("populated");
        assert!((last - 100.0).abs() < 1e-9);
    }

    #[test]
    fn dema_less_lag_than_plain_ema_on_uptrend() {
        let v: Vec<f64> = (1..=60).map(|i| 100.0 + i as f64).collect();
        let dema = compute(&v, 9);
        let single = ema(&v, 9);
        let last_dema = dema[59].expect("populated");
        let last_ema = single[59].expect("populated");
        let last_price = v[59];
        assert!(
            (last_dema - last_price).abs() < (last_ema - last_price).abs(),
            "DEMA={last_dema} EMA={last_ema} price={last_price}"
        );
    }

    #[test]
    fn huge_period_no_panic() {
        let v = vec![1.0; 5];
        assert!(compute(&v, usize::MAX).iter().all(|x| x.is_none()));
    }
}
