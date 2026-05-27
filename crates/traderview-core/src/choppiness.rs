//! Choppiness Index — measures market trendiness vs sideways action.
//!
//! E.W. Dreiss's indicator: high values (closer to 100) mean the market
//! is consolidating/choppy (sum of true-range covers a wide highs/lows
//! envelope but goes nowhere); low values (closer to 0) mean trending
//! (sum of TR ≈ highs − lows envelope, all directional).
//!
//! Formula: `CI = 100 × log10(sum(TR_n) / (max_high_n − min_low_n)) / log10(n)`
//!
//! Conventional thresholds: CI > 61.8 = chop, CI < 38.2 = trending,
//! everything in between is mixed.
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
pub enum ChopRegime {
    Trending,
    #[default]
    Mixed,
    Choppy,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChopReport {
    /// Per-bar choppiness value (None for pre-warmup bars).
    pub series: Vec<Option<f64>>,
    /// Most recent value.
    pub latest: Option<f64>,
    pub regime: ChopRegime,
    pub note: String,
}

pub fn compute(bars: &[OhlcBar], period: usize) -> ChopReport {
    let n = bars.len();
    if n == 0 || period < 2 || n < period + 1 {
        return ChopReport {
            note: format!("need ≥ {} bars, got {}", period + 1, n),
            ..Default::default()
        };
    }
    let tr = |i: usize| -> f64 {
        if i == 0 { bars[0].high - bars[0].low } else {
            let pc = bars[i - 1].close;
            let a = bars[i].high - bars[i].low;
            let b = (bars[i].high - pc).abs();
            let c = (bars[i].low - pc).abs();
            a.max(b).max(c)
        }
    };
    let log10_n = (period as f64).log10();
    let mut out: Vec<Option<f64>> = vec![None; n];
    for i in period..n {
        let window_start = i + 1 - period;
        let sum_tr: f64 = (window_start..=i).map(tr).sum();
        let hi = bars[window_start..=i].iter().map(|b| b.high).fold(f64::NEG_INFINITY, f64::max);
        let lo = bars[window_start..=i].iter().map(|b| b.low).fold(f64::INFINITY, f64::min);
        let env = hi - lo;
        if env <= 0.0 || sum_tr <= 0.0 {
            // Degenerate — leave as None.
            continue;
        }
        let ci = 100.0 * (sum_tr / env).log10() / log10_n;
        out[i] = Some(ci);
    }
    let latest = out.last().copied().flatten();
    let regime = match latest {
        Some(v) if v > 61.8 => ChopRegime::Choppy,
        Some(v) if v < 38.2 => ChopRegime::Trending,
        Some(_) => ChopRegime::Mixed,
        None => ChopRegime::Mixed,
    };
    let note = match latest {
        Some(v) => format!("CI = {v:.1} → {:?}", regime),
        None => "no value yet".into(),
    };
    ChopReport { series: out, latest, regime, note }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64, c: f64) -> OhlcBar { OhlcBar { high: h, low: l, close: c } }

    #[test]
    fn too_few_bars_returns_default() {
        let r = compute(&[b(100.0, 99.0, 99.5)], 14);
        assert!(r.latest.is_none());
        assert!(r.note.contains("need"));
    }

    #[test]
    fn period_below_two_returns_default() {
        let bars = vec![b(100.0, 99.0, 99.5); 5];
        let r = compute(&bars, 1);
        assert!(r.latest.is_none());
    }

    #[test]
    fn monotonic_trend_gives_low_ci() {
        // Strong uptrend: bars step from 100 → 130. Sum-TR ≈ envelope → ratio ≈ 1
        // → CI ≈ 0 (very trending).
        let bars: Vec<OhlcBar> = (0..30).map(|i| {
            let p = 100.0 + i as f64;
            b(p + 0.5, p - 0.5, p + 0.3)
        }).collect();
        let r = compute(&bars, 14);
        let v = r.latest.expect("populated");
        assert!(v < 50.0, "trending series CI should be low, got {v}");
        assert!(matches!(r.regime, ChopRegime::Trending | ChopRegime::Mixed));
    }

    #[test]
    fn flat_oscillation_gives_high_ci() {
        // Oscillates ±0.5 around 100. Sum-TR builds up while envelope stays narrow
        // → ratio large → CI ≈ 100.
        let bars: Vec<OhlcBar> = (0..30).map(|i| {
            let p = if i % 2 == 0 { 100.5 } else { 99.5 };
            b(p + 0.1, p - 0.1, p)
        }).collect();
        let r = compute(&bars, 14);
        let v = r.latest.expect("populated");
        assert!(v > 50.0, "choppy series CI should be high, got {v}");
        assert!(matches!(r.regime, ChopRegime::Choppy | ChopRegime::Mixed));
    }

    #[test]
    fn zero_range_window_returns_none_safely() {
        // 20 identical bars (zero envelope) → degenerate, leave None.
        let bars = vec![b(100.0, 100.0, 100.0); 20];
        let r = compute(&bars, 14);
        assert!(r.latest.is_none() || r.latest.unwrap().is_finite());
    }
}
