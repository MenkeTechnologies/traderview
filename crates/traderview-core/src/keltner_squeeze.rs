//! Keltner Squeeze — TTM/John Carter style squeeze detector.
//!
//! Different from `bb_squeeze` which already exists in the codebase
//! (that one is generic). This is the canonical TTM-Squeeze setup with
//! three states per bar:
//!
//! - **Squeeze On**:  Bollinger Bands FULLY INSIDE Keltner Channels.
//!   Volatility is compressed → coiling. The classic "fired off"
//!   signal triggers when the squeeze releases (off) AND momentum
//!   confirms direction.
//! - **Squeeze Off**: BB outside Keltner. Fully formed move underway.
//! - **None**:        Mixed (BB straddling Keltner).
//!
//! Pure compute. Returns per-bar state + a momentum oscillator
//! (linear-regression slope on (close − midpoint) over `period`).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SqueezeState { On, Off, None }

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SqueezeReport {
    pub state: Vec<Option<SqueezeState>>,
    pub momentum: Vec<Option<f64>>,
    /// Indices where state transitions from On → Off (the "fire").
    pub fires: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SqueezeConfig {
    pub period: usize,
    pub bb_k: f64,        // Bollinger stddev multiplier (default 2.0)
    pub kc_k: f64,        // Keltner ATR multiplier (default 1.5, classic TTM)
}

impl Default for SqueezeConfig {
    fn default() -> Self { Self { period: 20, bb_k: 2.0, kc_k: 1.5 } }
}

pub fn compute(bars: &[Bar], cfg: &SqueezeConfig) -> SqueezeReport {
    let n = bars.len();
    let mut report = SqueezeReport {
        state: vec![None; n],
        momentum: vec![None; n],
        fires: Vec::new(),
    };
    if cfg.period == 0 || n < cfg.period || cfg.bb_k <= 0.0 || cfg.kc_k <= 0.0
        || !cfg.bb_k.is_finite() || !cfg.kc_k.is_finite()
    {
        return report;
    }
    let p = cfg.period;
    let pf = p as f64;
    for i in (p - 1)..n {
        let win = &bars[i + 1 - p..=i];
        // Validate finite.
        if win.iter().any(|b| !b.high.is_finite() || !b.low.is_finite() || !b.close.is_finite()) {
            continue;
        }
        // Bollinger middle = SMA(close, p). Stdev = population stdev.
        let mean = win.iter().map(|b| b.close).sum::<f64>() / pf;
        let var = win.iter().map(|b| (b.close - mean).powi(2)).sum::<f64>() / pf;
        let stdev = var.sqrt();
        let bb_upper = mean + cfg.bb_k * stdev;
        let bb_lower = mean - cfg.bb_k * stdev;
        // Keltner: midline = SMA(close), bands = mid ± kc_k · ATR(p).
        // Use Wilder TR over the window for ATR estimate.
        let mut tr_sum = 0.0;
        for j in 1..win.len() {
            let h = win[j].high;
            let l = win[j].low;
            let pc = win[j - 1].close;
            tr_sum += (h - l).max((h - pc).abs()).max((l - pc).abs());
        }
        let atr = tr_sum / (p as f64 - 1.0).max(1.0);
        let kc_upper = mean + cfg.kc_k * atr;
        let kc_lower = mean - cfg.kc_k * atr;
        // State: BB fully inside KC = squeeze ON.
        let state = if bb_upper <= kc_upper && bb_lower >= kc_lower {
            SqueezeState::On
        } else if bb_upper > kc_upper && bb_lower < kc_lower {
            SqueezeState::Off
        } else {
            SqueezeState::None
        };
        report.state[i] = Some(state);
        // Momentum: linear regression slope of (close − midline) over window.
        // Use closed-form: slope = Σ((x − x̄)(y − ȳ)) / Σ(x − x̄)²
        let dev_close = mean;
        let y: Vec<f64> = win.iter().map(|b| b.close - dev_close).collect();
        let x_mean = (p as f64 - 1.0) / 2.0;
        let y_mean = y.iter().sum::<f64>() / pf;
        let mut num = 0.0;
        let mut den = 0.0;
        for (k, yk) in y.iter().enumerate() {
            let dx = k as f64 - x_mean;
            num += dx * (yk - y_mean);
            den += dx * dx;
        }
        if den > 0.0 {
            let slope = num / den;
            if slope.is_finite() {
                report.momentum[i] = Some(slope);
            }
        }
        // Detect On → Off transition for `fires`.
        if i > 0 {
            if let (Some(SqueezeState::On), Some(SqueezeState::Off))
                = (report.state[i - 1], report.state[i])
            {
                report.fires.push(i);
            }
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64, c: f64) -> Bar {
        Bar { high: h, low: l, close: c }
    }

    #[test]
    fn empty_returns_default() {
        let r = compute(&[], &SqueezeConfig::default());
        assert!(r.state.is_empty());
    }

    #[test]
    fn invalid_config_returns_default() {
        let bars = vec![b(101.0, 99.0, 100.0); 30];
        for cfg in [
            SqueezeConfig { period: 0, ..Default::default() },
            SqueezeConfig { bb_k: 0.0, ..Default::default() },
            SqueezeConfig { kc_k: -1.0, ..Default::default() },
            SqueezeConfig { bb_k: f64::NAN, ..Default::default() },
        ] {
            let r = compute(&bars, &cfg);
            assert!(r.state.iter().all(|x| x.is_none()));
        }
    }

    #[test]
    fn flat_series_produces_squeeze_on() {
        // No price movement → BB and KC both collapse to midline → both 0
        // width → BB ⊆ KC trivially → On.
        let bars = vec![b(100.0, 100.0, 100.0); 30];
        let r = compute(&bars, &SqueezeConfig::default());
        for s in r.state.iter().flatten() {
            assert_eq!(*s, SqueezeState::On);
        }
    }

    #[test]
    fn high_volatility_series_produces_squeeze_off() {
        // Big oscillations → BB stdev dominates → wide BB > narrow KC → Off.
        let bars: Vec<Bar> = (0..40).map(|i| {
            let c = 100.0 + (i as f64).sin() * 50.0;
            b(c + 1.0, c - 1.0, c)
        }).collect();
        let r = compute(&bars, &SqueezeConfig::default());
        // Most populated states should be Off (huge close stdev, modest H-L range).
        let offs = r.state.iter().filter(|s| **s == Some(SqueezeState::Off)).count();
        assert!(offs > 0, "expected at least some Squeeze::Off states");
    }

    #[test]
    fn nan_bar_skipped_safely() {
        let mut bars = vec![b(101.0, 99.0, 100.0); 30];
        bars[15] = b(f64::NAN, f64::NAN, f64::NAN);
        let r = compute(&bars, &SqueezeConfig::default());
        // Just don't panic.
        assert_eq!(r.state.len(), 30);
    }

    #[test]
    fn huge_period_no_panic() {
        let bars = vec![b(101.0, 99.0, 100.0); 5];
        let cfg = SqueezeConfig { period: usize::MAX, ..Default::default() };
        let r = compute(&bars, &cfg);
        assert!(r.state.iter().all(|x| x.is_none()));
    }
}
