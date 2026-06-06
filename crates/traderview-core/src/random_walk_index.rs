//! Random Walk Index (RWI) — Mike Poulos's trendiness measure.
//!
//! RWI compares the actual price move to what a random walk of equal
//! volatility would have produced. Two values per bar — RWI-high (how
//! "non-random" the up-move is) and RWI-low (down-move). Whichever is
//! larger indicates the dominant direction; magnitude > 1 indicates the
//! move is statistically beyond a random walk (genuine trend).
//!
//! Formula for RWI-high:
//!   `RWI_h(n) = (high[i] − low[i-n]) / (ATR(n) × √n)`
//!
//! And RWI-low mirrors with low/high inverted. Both are computed across
//! lookbacks 2..=N and the maximum is reported.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct OhlcBar {
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RwiBias {
    Up,
    Down,
    #[default]
    Neutral,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RwiReport {
    /// Per-bar RWI-high values (None for pre-warmup).
    pub rwi_high: Vec<Option<f64>>,
    pub rwi_low: Vec<Option<f64>>,
    pub latest_high: Option<f64>,
    pub latest_low: Option<f64>,
    pub bias: RwiBias,
}

pub fn compute(bars: &[OhlcBar], max_n: usize) -> RwiReport {
    let n = bars.len();
    if n == 0 || max_n < 2 || n <= max_n {
        // Preserve the input-aligned `rwi_high.len() == bars.len()`
        // invariant the populated path honors.
        return RwiReport {
            rwi_high: vec![None; n],
            rwi_low: vec![None; n],
            ..RwiReport::default()
        };
    }
    // Pre-compute true-range series.
    let tr: Vec<f64> = (0..n)
        .map(|i| {
            if i == 0 {
                bars[0].high - bars[0].low
            } else {
                let pc = bars[i - 1].close;
                let a = bars[i].high - bars[i].low;
                let b = (bars[i].high - pc).abs();
                let c = (bars[i].low - pc).abs();
                a.max(b).max(c)
            }
        })
        .collect();
    let mut rwi_h: Vec<Option<f64>> = vec![None; n];
    let mut rwi_l: Vec<Option<f64>> = vec![None; n];
    for i in max_n..n {
        let mut best_h = 0.0_f64;
        let mut best_l = 0.0_f64;
        for k in 2..=max_n {
            // ATR over [i-k+1 ..= i].
            let window = &tr[(i + 1 - k)..=i];
            let atr_k = window.iter().sum::<f64>() / k as f64;
            if atr_k <= 0.0 {
                continue;
            }
            let denom = atr_k * (k as f64).sqrt();
            let rh = (bars[i].high - bars[i - k + 1].low) / denom;
            let rl = (bars[i - k + 1].high - bars[i].low) / denom;
            if rh > best_h {
                best_h = rh;
            }
            if rl > best_l {
                best_l = rl;
            }
        }
        rwi_h[i] = Some(best_h);
        rwi_l[i] = Some(best_l);
    }
    let latest_high = rwi_h.last().copied().flatten();
    let latest_low = rwi_l.last().copied().flatten();
    let bias = match (latest_high, latest_low) {
        (Some(h), Some(l)) if h > l && h > 1.0 => RwiBias::Up,
        (Some(h), Some(l)) if l > h && l > 1.0 => RwiBias::Down,
        _ => RwiBias::Neutral,
    };
    RwiReport {
        rwi_high: rwi_h,
        rwi_low: rwi_l,
        latest_high,
        latest_low,
        bias,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64, c: f64) -> OhlcBar {
        OhlcBar {
            high: h,
            low: l,
            close: c,
        }
    }

    #[test]
    fn empty_or_short_returns_default() {
        let r = compute(&[], 8);
        assert!(r.latest_high.is_none());
        let bars = vec![b(100.0, 99.0, 99.5); 5];
        let r = compute(&bars, 8);
        assert!(r.latest_high.is_none());
    }

    #[test]
    fn strong_uptrend_gives_up_bias() {
        // Big steady uptrend.
        let bars: Vec<OhlcBar> = (0..20)
            .map(|i| {
                let p = 100.0 + i as f64 * 2.0;
                b(p + 0.1, p - 0.1, p + 0.05)
            })
            .collect();
        let r = compute(&bars, 8);
        assert!(
            matches!(r.bias, RwiBias::Up),
            "got {:?} high={:?} low={:?}",
            r.bias,
            r.latest_high,
            r.latest_low
        );
        assert!(r.latest_high.unwrap() > 1.0);
    }

    #[test]
    fn strong_downtrend_gives_down_bias() {
        let bars: Vec<OhlcBar> = (0..20)
            .map(|i| {
                let p = 200.0 - i as f64 * 2.0;
                b(p + 0.1, p - 0.1, p - 0.05)
            })
            .collect();
        let r = compute(&bars, 8);
        assert!(matches!(r.bias, RwiBias::Down));
        assert!(r.latest_low.unwrap() > 1.0);
    }

    #[test]
    fn flat_oscillation_is_neutral() {
        let bars: Vec<OhlcBar> = (0..20)
            .map(|i| {
                let p = if i % 2 == 0 { 100.5 } else { 99.5 };
                b(p + 0.1, p - 0.1, p)
            })
            .collect();
        let r = compute(&bars, 8);
        // Both up and down would have low magnitudes; bias is Neutral.
        assert!(
            matches!(r.bias, RwiBias::Neutral),
            "expected Neutral, got {:?}",
            r.bias
        );
    }

    #[test]
    fn max_n_below_two_returns_default() {
        let bars = vec![b(100.0, 99.0, 99.5); 20];
        let r = compute(&bars, 1);
        assert!(r.latest_high.is_none());
    }

    #[test]
    fn series_shorter_than_max_n_returns_default() {
        let bars = vec![b(100.0, 99.0, 99.5); 5];
        let r = compute(&bars, 6);
        assert!(r.latest_high.is_none());
    }
}
