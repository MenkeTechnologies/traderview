//! Ultimate Oscillator — Larry Williams (1976).
//!
//! Combines three timeframes into a single 0..100 oscillator that's less
//! susceptible to false signals than single-period oscillators.
//!
//!   BP = close − min(low, prev_close)
//!   TR = max(high, prev_close) − min(low, prev_close)
//!   AVG_N = sum(BP, N) / sum(TR, N)
//!   UO = 100 × (4·AVG_short + 2·AVG_mid + AVG_long) / 7
//!
//! Standard periods: 7 / 14 / 28. Convention: >70 overbought,
//! <30 oversold; bullish divergence under 30 is the textbook entry.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct OhlcBar {
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

pub fn compute(
    bars: &[OhlcBar],
    short: usize,
    mid: usize,
    long: usize,
) -> Vec<Option<f64>> {
    let n = bars.len();
    let mut out = vec![None; n];
    if short == 0 || mid == 0 || long == 0 {
        return out;
    }
    let max_p = short.max(mid).max(long);
    if n <= max_p {
        return out;
    }
    // Precompute BP and TR arrays starting at index 1 (need prev_close).
    let mut bp = vec![0.0_f64; n];
    let mut tr = vec![0.0_f64; n];
    for i in 1..n {
        let pc = bars[i - 1].close;
        let low_or_pc = bars[i].low.min(pc);
        let high_or_pc = bars[i].high.max(pc);
        bp[i] = bars[i].close - low_or_pc;
        tr[i] = high_or_pc - low_or_pc;
    }
    for i in max_p..n {
        let s_bp: f64 = bp[i + 1 - short..=i].iter().sum();
        let s_tr: f64 = tr[i + 1 - short..=i].iter().sum();
        let m_bp: f64 = bp[i + 1 - mid..=i].iter().sum();
        let m_tr: f64 = tr[i + 1 - mid..=i].iter().sum();
        let l_bp: f64 = bp[i + 1 - long..=i].iter().sum();
        let l_tr: f64 = tr[i + 1 - long..=i].iter().sum();
        if s_tr <= 0.0 || m_tr <= 0.0 || l_tr <= 0.0 {
            continue;
        }
        let avg_s = s_bp / s_tr;
        let avg_m = m_bp / m_tr;
        let avg_l = l_bp / l_tr;
        let uo = 100.0 * (4.0 * avg_s + 2.0 * avg_m + avg_l) / 7.0;
        if uo.is_finite() {
            out[i] = Some(uo);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64, c: f64) -> OhlcBar {
        OhlcBar { high: h, low: l, close: c }
    }

    #[test]
    fn empty_returns_empty() {
        assert!(compute(&[], 7, 14, 28).is_empty());
    }

    #[test]
    fn zero_period_returns_all_none() {
        let bars: Vec<OhlcBar> = (0..40).map(|_| b(101.0, 99.0, 100.0)).collect();
        for (s, m, l) in [(0, 14, 28), (7, 0, 28), (7, 14, 0)] {
            assert!(compute(&bars, s, m, l).iter().all(|x| x.is_none()));
        }
    }

    #[test]
    fn flat_series_uo_near_50() {
        // Each bar identical → BP/TR ratio = constant → UO well-defined.
        let bars: Vec<OhlcBar> = (0..40).map(|_| b(101.0, 99.0, 100.0)).collect();
        let out = compute(&bars, 7, 14, 28);
        let last = out[39].expect("populated");
        // For identical bars: BP = 100 - min(99, 100) = 1; TR = max(101,100) - min(99,100) = 2.
        // ratio = 1/2 = 0.5 → UO = 100 × 0.5 = 50.
        assert!((last - 50.0).abs() < 1e-9, "flat UO should be 50, got {last}");
    }

    #[test]
    fn strong_uptrend_yields_high_uo() {
        // Need GAPS so today's low > prior close. With unit steps and
        // ±1 range, low_t = pc_t exactly, so min(low,pc)=pc and BP/TR
        // collapses to 1/2 (UO = 50). Use 5-pt steps with tight ±1 range:
        // low_t = c_t − 1 = pc_t + 4 > pc_t, giving BP/TR ≈ 5/6.
        let bars: Vec<OhlcBar> = (1..=60)
            .map(|i| {
                let c = 100.0 + 5.0 * i as f64;
                b(c + 1.0, c - 1.0, c)
            })
            .collect();
        let out = compute(&bars, 7, 14, 28);
        let last = out[59].expect("populated");
        assert!(last > 70.0, "gapping uptrend UO should be > 70, got {last}");
    }

    #[test]
    fn series_shorter_than_long_returns_all_none() {
        let bars: Vec<OhlcBar> = (0..10).map(|_| b(101.0, 99.0, 100.0)).collect();
        let out = compute(&bars, 7, 14, 28);
        assert!(out.iter().all(|x| x.is_none()));
    }

    #[test]
    fn huge_period_no_panic() {
        let bars: Vec<OhlcBar> = (0..5).map(|_| b(1.0, 1.0, 1.0)).collect();
        let out = compute(&bars, 7, 14, usize::MAX);
        assert!(out.iter().all(|x| x.is_none()));
    }
}
