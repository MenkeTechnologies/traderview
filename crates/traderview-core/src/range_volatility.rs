//! Range-based volatility estimators — Parkinson (1980), Garman-Klass
//! (1980), Rogers-Satchell (1991), Yang-Zhang (2000).
//!
//! All four estimate σ² using high/low/open/close *intra-bar* range
//! information, which is far more efficient than close-to-close
//! variance for any given sample size (Parkinson is ≈ 5× more efficient
//! than close-to-close for Gaussian processes).
//!
//! Returns per-bar **rolling** σ over `period` bars for each estimator.
//! All formulas return non-annualized per-bar variance; caller
//! multiplies by √(periods_per_year) to annualize.
//!
//! Pure compute. Requires open/high/low/close per bar.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RangeVolReport {
    pub parkinson: Vec<Option<f64>>,
    pub garman_klass: Vec<Option<f64>>,
    pub rogers_satchell: Vec<Option<f64>>,
    pub yang_zhang: Vec<Option<f64>>,
}

pub fn compute(bars: &[Bar], period: usize) -> RangeVolReport {
    let n = bars.len();
    let mut report = RangeVolReport {
        parkinson: vec![None; n],
        garman_klass: vec![None; n],
        rogers_satchell: vec![None; n],
        yang_zhang: vec![None; n],
    };
    if period == 0 || n < period {
        return report;
    }
    // Per-bar contributions to each variance.
    let pk_const = 1.0 / (4.0 * std::f64::consts::LN_2);
    let mut pk = vec![None::<f64>; n];
    let mut gk = vec![None::<f64>; n];
    let mut rs = vec![None::<f64>; n];
    // Close-to-open and open-to-close returns for Yang-Zhang.
    let mut co = vec![None::<f64>; n]; // ln(O_t / C_{t−1})
    let mut oc = vec![None::<f64>; n]; // ln(C_t / O_t)
    for (i, b) in bars.iter().enumerate() {
        if !b.open.is_finite()
            || !b.high.is_finite()
            || !b.low.is_finite()
            || !b.close.is_finite()
            || b.open <= 0.0
            || b.high <= 0.0
            || b.low <= 0.0
            || b.close <= 0.0
            || b.high < b.low
        {
            continue;
        }
        let ln_hl = (b.high / b.low).ln();
        let ln_co = (b.close / b.open).ln();
        let ln_ho = (b.high / b.open).ln();
        let ln_lo = (b.low / b.open).ln();
        let ln_hc = (b.high / b.close).ln();
        let ln_lc = (b.low / b.close).ln();
        pk[i] = Some(pk_const * ln_hl * ln_hl);
        gk[i] = Some(0.5 * ln_hl * ln_hl - (2.0 * std::f64::consts::LN_2 - 1.0) * ln_co * ln_co);
        rs[i] = Some(
            ln_ho * (ln_ho - ln_co) + ln_lo * (ln_lo - ln_co)
            // Equivalent form using (h−c)(h−o) + (l−c)(l−o):
            + ln_hc * 0.0 + ln_lc * 0.0,
        );
        oc[i] = Some(ln_co);
        if i > 0 && bars[i - 1].close.is_finite() && bars[i - 1].close > 0.0 {
            co[i] = Some((b.open / bars[i - 1].close).ln());
        }
    }
    // Rolling sums → variances → stdev.
    fn rolling_mean(v: &[Option<f64>], period: usize) -> Vec<Option<f64>> {
        let n = v.len();
        let mut out = vec![None; n];
        if period == 0 || period > n {
            return out;
        }
        for i in (period - 1)..n {
            let win = &v[i + 1 - period..=i];
            let mut sum = 0.0;
            let mut ok = true;
            for x in win {
                match x {
                    Some(val) if val.is_finite() => sum += val,
                    _ => {
                        ok = false;
                        break;
                    }
                }
            }
            if ok {
                out[i] = Some(sum / period as f64);
            }
        }
        out
    }
    let pk_avg = rolling_mean(&pk, period);
    let gk_avg = rolling_mean(&gk, period);
    let rs_avg = rolling_mean(&rs, period);
    for i in 0..n {
        if let Some(v) = pk_avg[i] {
            if v >= 0.0 {
                report.parkinson[i] = Some(v.sqrt());
            }
        }
        if let Some(v) = gk_avg[i] {
            if v >= 0.0 {
                report.garman_klass[i] = Some(v.sqrt());
            }
        }
        if let Some(v) = rs_avg[i] {
            if v >= 0.0 {
                report.rogers_satchell[i] = Some(v.sqrt());
            }
        }
    }
    // Yang-Zhang: σ² = σ²_overnight + k·σ²_open-to-close + (1-k)·σ²_RS
    // where k = 0.34 / (1.34 + (period+1)/(period-1))
    if period >= 2 {
        let k = 0.34 / (1.34 + (period as f64 + 1.0) / (period as f64 - 1.0));
        let co_var = rolling_var(&co, period);
        let oc_var = rolling_var(&oc, period);
        for i in 0..n {
            if let (Some(o_var), Some(c_var), Some(rs_var)) = (co_var[i], oc_var[i], rs_avg[i]) {
                let yz_var = o_var + k * c_var + (1.0 - k) * rs_var;
                if yz_var.is_finite() && yz_var >= 0.0 {
                    report.yang_zhang[i] = Some(yz_var.sqrt());
                }
            }
        }
    }
    report
}

fn rolling_var(v: &[Option<f64>], period: usize) -> Vec<Option<f64>> {
    let n = v.len();
    let mut out = vec![None; n];
    if period < 2 || period > n {
        return out;
    }
    for i in (period - 1)..n {
        let win = &v[i + 1 - period..=i];
        let mut sum = 0.0;
        let mut sum_sq = 0.0;
        let mut ok = true;
        let mut count = 0;
        for x in win {
            match x {
                Some(val) if val.is_finite() => {
                    sum += val;
                    sum_sq += val * val;
                    count += 1;
                }
                _ => {
                    ok = false;
                    break;
                }
            }
        }
        if ok && count >= 2 {
            let mean = sum / count as f64;
            // Computed-moment formula can produce a tiny *negative*
            // value via float cancellation when all window values are
            // identical (e.g. flat bars). Clamp such values to 0 rather
            // than discard — otherwise downstream estimators (Yang-Zhang)
            // would silently drop populated bars on constant inputs.
            let var = ((sum_sq - count as f64 * mean * mean) / (count as f64 - 1.0)).max(0.0);
            if var.is_finite() {
                out[i] = Some(var);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(o: f64, h: f64, l: f64, c: f64) -> Bar {
        Bar {
            open: o,
            high: h,
            low: l,
            close: c,
        }
    }

    #[test]
    fn empty_returns_default() {
        let r = compute(&[], 20);
        assert!(r.parkinson.is_empty());
    }

    #[test]
    fn period_zero_returns_all_none() {
        let bars = vec![b(100.0, 101.0, 99.0, 100.5); 30];
        let r = compute(&bars, 0);
        assert!(r.parkinson.iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_bars_yield_zero_volatility() {
        let bars = vec![b(100.0, 100.0, 100.0, 100.0); 30];
        let r = compute(&bars, 20);
        for x in r.parkinson.iter().flatten() {
            assert!(x.abs() < 1e-12);
        }
        for x in r.garman_klass.iter().flatten() {
            assert!(x.abs() < 1e-12);
        }
    }

    #[test]
    fn nan_bars_skipped_safely() {
        let mut bars = vec![b(100.0, 101.0, 99.0, 100.5); 30];
        bars[15] = b(f64::NAN, f64::NAN, f64::NAN, f64::NAN);
        let r = compute(&bars, 5);
        // Just don't panic — output length matches input.
        assert_eq!(r.parkinson.len(), 30);
    }

    #[test]
    fn high_volatility_bars_yield_high_volatility_estimate() {
        // Big intraday range vs tight: PK on tight should be tiny.
        let tight = vec![b(100.0, 100.5, 99.5, 100.0); 30];
        let wide = vec![b(100.0, 110.0, 90.0, 100.0); 30];
        let r_tight = compute(&tight, 20);
        let r_wide = compute(&wide, 20);
        let t = r_tight.parkinson[29].unwrap();
        let w = r_wide.parkinson[29].unwrap();
        assert!(w > 10.0 * t, "wide PK ({w}) should be ≫ tight PK ({t})");
    }

    #[test]
    fn rogers_satchell_zero_for_close_equals_open() {
        // RS = ln(H/O)·(ln(H/O) − ln(C/O)) + ln(L/O)·(ln(L/O) − ln(C/O))
        // When C=O: ln(C/O)=0 → RS = (ln(H/O))² + (ln(L/O))² > 0.
        // (So we DON'T expect zero — just verify it's positive.)
        let bars = vec![b(100.0, 102.0, 98.0, 100.0); 30];
        let r = compute(&bars, 20);
        let v = r.rogers_satchell[29].unwrap();
        assert!(v > 0.0);
    }

    #[test]
    fn yang_zhang_requires_overnight_returns() {
        // First bar has no overnight return → YZ at index 0 is None.
        let bars = vec![b(100.0, 102.0, 98.0, 101.0); 30];
        let r = compute(&bars, 20);
        assert!(r.yang_zhang[0].is_none());
        // After warmup, YZ should be populated.
        assert!(r.yang_zhang[29].is_some());
    }

    #[test]
    fn huge_period_no_panic() {
        let bars = vec![b(100.0, 101.0, 99.0, 100.5); 5];
        let r = compute(&bars, usize::MAX);
        assert!(r.parkinson.iter().all(|x| x.is_none()));
    }
}
