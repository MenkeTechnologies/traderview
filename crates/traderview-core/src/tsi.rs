//! TSI — True Strength Index (William Blau, 1991).
//!
//!   PC = close − close[-1]
//!   smoothed_PC  = EMA(EMA(PC, r), s)
//!   smoothed_APC = EMA(EMA(|PC|, r), s)
//!   TSI = 100 · smoothed_PC / smoothed_APC
//!
//! Standard params: r=25, s=13. Range ≈ ±100. Crossover of TSI with its
//! own signal line (additional EMA) is the classic entry. Less laggy than
//! MACD, smoother than RSI.
//!
//! Pure compute.

pub fn compute(closes: &[f64], r_period: usize, s_period: usize) -> Vec<Option<f64>> {
    let n = closes.len();
    let mut out = vec![None; n];
    if r_period == 0 || s_period == 0 || n < 2 {
        return out;
    }
    // Price change PC at index i (i >= 1); use 0 at index 0 as a pad so the
    // EMA helper sees a fully-populated input array.
    let mut pc = vec![0.0_f64; n];
    let mut apc = vec![0.0_f64; n];
    for i in 1..n {
        let d = closes[i] - closes[i - 1];
        pc[i] = d;
        apc[i] = d.abs();
    }
    // Smooth PC: EMA(r) then EMA(s).
    let pc1 = ema(&pc, r_period);
    let pc2 = ema_optional(&pc1, s_period);
    let apc1 = ema(&apc, r_period);
    let apc2 = ema_optional(&apc1, s_period);
    for i in 0..n {
        if let (Some(num), Some(den)) = (pc2[i], apc2[i]) {
            if den > 0.0 {
                let v = 100.0 * num / den;
                if v.is_finite() {
                    out[i] = Some(v);
                }
            }
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
        assert!(compute(&[], 25, 13).is_empty());
    }

    #[test]
    fn zero_period_returns_all_none() {
        let v = vec![100.0; 50];
        assert!(compute(&v, 0, 13).iter().all(|x| x.is_none()));
        assert!(compute(&v, 25, 0).iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_series_tsi_zero_or_undefined() {
        // Flat → all PC=0 → smoothed_APC=0 → TSI undefined.
        let v = vec![100.0; 80];
        let out = compute(&v, 25, 13);
        let last = out[79];
        assert!(last.is_none() || last.unwrap().abs() < 1e-9);
    }

    #[test]
    fn strong_uptrend_tsi_positive() {
        let v: Vec<f64> = (1..=80).map(|i| 100.0 + i as f64).collect();
        let out = compute(&v, 25, 13);
        let last = out[79].expect("populated");
        // Perfect uptrend → smoothed_PC ≈ smoothed_APC → TSI ≈ 100.
        assert!(last > 50.0, "uptrend TSI should be high, got {last}");
        assert!(last <= 100.0 + 1e-9);
    }

    #[test]
    fn strong_downtrend_tsi_negative() {
        let v: Vec<f64> = (1..=80).map(|i| 200.0 - i as f64).collect();
        let out = compute(&v, 25, 13);
        let last = out[79].expect("populated");
        assert!(last < -50.0);
        assert!(last >= -100.0 - 1e-9);
    }

    #[test]
    fn huge_period_no_panic() {
        let v = vec![1.0; 5];
        assert!(compute(&v, usize::MAX, 13).iter().all(|x| x.is_none()));
    }
}
