//! Relative Strength line + Mansfield Relative Strength.
//!
//!   RS_t        = close_t / bench_t × 100
//!   Mansfield_t = (RS_t / SMA(RS, period) − 1) × 10
//!
//! The RS line (IBD style) shows whether a stock leads or lags its
//! benchmark regardless of absolute direction — new RS highs ahead of
//! price highs flag leadership. Mansfield (Stan Weinstein, 1988)
//! zero-centers it against its own moving average: above 0 = stronger
//! than benchmark trend, below = weaker. Canonical period is 52 on
//! weekly bars (≈200–252 on daily).
//!
//! Pure compute. Inputs must be pre-aligned (same bar per index).
//! Mansfield is 0.0 until its SMA warms up; bars with a non-positive
//! benchmark close carry the previous RS value.

pub fn compute(closes: &[f64], bench: &[f64], period: usize) -> (Vec<f64>, Vec<f64>) {
    let n = closes.len().min(bench.len());
    let mut rs = vec![0.0; n];
    let mut mansfield = vec![0.0; n];
    if n == 0 {
        return (rs, mansfield);
    }
    for i in 0..n {
        rs[i] = if bench[i] > 0.0 && closes[i].is_finite() && bench[i].is_finite() {
            closes[i] / bench[i] * 100.0
        } else if i > 0 {
            rs[i - 1]
        } else {
            0.0
        };
    }
    if period == 0 || n < period {
        return (rs, mansfield);
    }
    let mut window_sum: f64 = rs[..period].iter().sum();
    for i in period - 1..n {
        if i >= period {
            window_sum += rs[i] - rs[i - period];
        }
        let sma = window_sum / period as f64;
        if sma.abs() > 1e-12 {
            mansfield[i] = (rs[i] / sma - 1.0) * 10.0;
        }
    }
    (rs, mansfield)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rs_line_is_ratio_times_hundred() {
        let closes = vec![10.0, 12.0, 9.0];
        let bench = vec![100.0, 100.0, 90.0];
        let (rs, _) = compute(&closes, &bench, 0);
        assert_eq!(rs, vec![10.0, 12.0, 10.0]);
    }

    #[test]
    fn mansfield_is_zero_when_rs_sits_on_its_sma() {
        // Stock and benchmark move in lockstep ⇒ RS constant ⇒ RS ==
        // SMA(RS) ⇒ Mansfield exactly 0 after warmup.
        let closes: Vec<f64> = (1..=20).map(|i| 10.0 * i as f64).collect();
        let bench: Vec<f64> = (1..=20).map(|i| 100.0 * i as f64).collect();
        let (rs, m) = compute(&closes, &bench, 5);
        assert!(rs.iter().all(|x| (*x - 10.0).abs() < 1e-12));
        assert!(m[4..].iter().all(|x| x.abs() < 1e-12), "{m:?}");
    }

    #[test]
    fn mansfield_goes_positive_when_stock_outpaces_benchmark() {
        // Flat RS at 10 for 9 bars, then the stock doubles vs bench:
        // RS jumps to 20; SMA(5) over [10,10,10,10,20] = 12 ⇒
        // Mansfield = (20/12 − 1)·10 = 6.667.
        let mut closes = vec![10.0; 10];
        closes[9] = 20.0;
        let bench = vec![100.0; 10];
        let (_, m) = compute(&closes, &bench, 5);
        assert!((m[9] - (20.0 / 12.0 - 1.0) * 10.0).abs() < 1e-12, "{}", m[9]);
        // The flat stretch before the jump reads 0.
        assert!(m[8].abs() < 1e-12);
    }

    #[test]
    fn rs_line_survives_hostile_inputs() {
        assert_eq!(compute(&[], &[], 5).0.len(), 0);
        // Zero benchmark close carries the prior RS forward.
        let (rs, _) = compute(&[10.0, 11.0, 12.0], &[100.0, 0.0, 100.0], 0);
        assert_eq!(rs, vec![10.0, 10.0, 12.0]);
        // Mismatched lengths truncate to the shorter series.
        let (rs, _) = compute(&[10.0, 11.0], &[100.0], 0);
        assert_eq!(rs.len(), 1);
        // period longer than the series leaves Mansfield all-zero.
        let (_, m) = compute(&[10.0; 4], &[100.0; 4], 10);
        assert!(m.iter().all(|x| *x == 0.0));
    }
}
