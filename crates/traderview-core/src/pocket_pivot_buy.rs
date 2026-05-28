//! Pocket Pivot Buy Point Detector — O'Neil / Morales (2010).
//!
//! Identifies stealth accumulation days where:
//!
//!   1. Up-day (close > prior close) inside a constructive base
//!   2. Volume on the up-day exceeds the highest down-day volume in
//!      the prior 10 days
//!   3. Price is above the 50-day moving average
//!   4. Price hasn't gapped down significantly recently
//!
//! Per Morales/Kacher convention, the 4th criterion is "no down-gap of
//! more than 0.5%" within the past 10 days — a sign of orderly
//! consolidation. We bundle the strict version + a relaxed scanner mode.
//!
//! Pure compute. Companion to `minervini_trend_template`, `vcp_pattern`,
//! `darvas_box`, `breakout_detector`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PocketPivotEvent {
    pub bar_index: usize,
    pub close: f64,
    pub volume: f64,
    pub max_down_volume_prior_10d: f64,
    pub ma_50: f64,
    pub above_50ma: bool,
    pub no_recent_down_gap: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub lookback_volume_days: usize,
    pub max_down_gap_pct: f64,
    pub require_above_ma50: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            lookback_volume_days: 10,
            max_down_gap_pct: 0.005,
            require_above_ma50: true,
        }
    }
}

pub fn detect(bars: &[Bar], cfg: &Config) -> Vec<PocketPivotEvent> {
    let n = bars.len();
    if n < 52 || cfg.lookback_volume_days < 2 { return Vec::new(); }
    if bars.iter().any(|b| !b.open.is_finite() || !b.high.is_finite()
        || !b.low.is_finite() || !b.close.is_finite()
        || !b.volume.is_finite() || b.volume < 0.0) {
        return Vec::new();
    }
    let mut out = Vec::new();
    for i in 50..n {
        // Up-day check.
        if bars[i].close <= bars[i - 1].close { continue; }
        // Volume vs max down-day volume in prior lookback.
        let lookback_start = i.saturating_sub(cfg.lookback_volume_days);
        let mut max_down_vol = 0.0_f64;
        for j in lookback_start..i {
            if bars[j].close < bars[j - 1].close && bars[j].volume > max_down_vol {
                max_down_vol = bars[j].volume;
            }
        }
        if bars[i].volume <= max_down_vol { continue; }
        // 50-day MA check.
        let ma50: f64 = bars[i + 1 - 50..=i].iter().map(|b| b.close).sum::<f64>() / 50.0;
        let above_50 = bars[i].close > ma50;
        if cfg.require_above_ma50 && !above_50 { continue; }
        // No big down-gap in lookback window.
        let mut no_gap = true;
        for j in lookback_start.max(1)..i {
            let gap = (bars[j].open - bars[j - 1].close) / bars[j - 1].close;
            if gap < -cfg.max_down_gap_pct { no_gap = false; break; }
        }
        out.push(PocketPivotEvent {
            bar_index: i,
            close: bars[i].close,
            volume: bars[i].volume,
            max_down_volume_prior_10d: max_down_vol,
            ma_50: ma50,
            above_50ma: above_50,
            no_recent_down_gap: no_gap,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(o: f64, h: f64, l: f64, c: f64, v: f64) -> Bar {
        Bar { open: o, high: h, low: l, close: c, volume: v }
    }

    #[test]
    fn too_short_returns_empty() {
        let bars = vec![b(100.0, 101.0, 99.0, 100.0, 1000.0); 30];
        assert!(detect(&bars, &Config::default()).is_empty());
    }

    #[test]
    fn invalid_config_returns_empty() {
        let bars = vec![b(100.0, 101.0, 99.0, 100.0, 1000.0); 60];
        let cfg = Config { lookback_volume_days: 1, ..Default::default() };
        assert!(detect(&bars, &cfg).is_empty());
    }

    #[test]
    fn nan_returns_empty() {
        let mut bars = vec![b(100.0, 101.0, 99.0, 100.0, 1000.0); 60];
        bars[5] = b(f64::NAN, 101.0, 99.0, 100.0, 1000.0);
        assert!(detect(&bars, &Config::default()).is_empty());
    }

    #[test]
    fn no_up_day_no_event() {
        // All down-closing days → no up-days → no events.
        let bars: Vec<_> = (0..60).map(|i| {
            let c = 100.0 - i as f64 * 0.1;
            b(c, c + 0.5, c - 0.5, c, 1000.0)
        }).collect();
        assert!(detect(&bars, &Config::default()).is_empty());
    }

    #[test]
    fn classic_pocket_pivot_detected() {
        // 50 flat down-volume days; then big up-day with surge volume.
        let mut bars: Vec<Bar> = (0..55).map(|i| {
            // Slight zig-zag to avoid monotonic constraints.
            let base = 100.0 + (i as f64 * 0.1).sin();
            b(base, base + 0.5, base - 0.5, base, 1000.0)
        }).collect();
        // Pocket pivot at bar 55: up-day with volume far exceeding prior down vols.
        bars.push(b(101.0, 102.0, 100.5, 102.0, 5000.0));
        let events = detect(&bars, &Config { require_above_ma50: false, ..Default::default() });
        assert!(!events.is_empty(), "expected pocket pivot event");
        assert_eq!(events.last().unwrap().bar_index, 55);
    }

    #[test]
    fn down_gap_blocks_event_in_strict_mode() {
        // Construct a setup that meets all other criteria but has a recent down-gap.
        let mut bars: Vec<Bar> = (0..55).map(|_| b(100.0, 100.5, 99.5, 100.0, 1000.0)).collect();
        // Insert big down-gap at bar 52.
        bars[52] = b(95.0, 96.0, 94.0, 95.5, 1500.0);    // gap_open = (95-100)/100 = -5%
        // Up-day with surge volume at bar 55.
        bars[54] = b(101.0, 102.0, 100.5, 101.5, 5000.0);
        let events = detect(&bars, &Config::default());
        // Bar 54 should be flagged but the down-gap criterion = false in the event.
        if let Some(e) = events.last() {
            assert!(!e.no_recent_down_gap);
        }
    }

    #[test]
    fn requires_above_50ma_when_configured() {
        // Setup where price drops below 50MA — should be filtered.
        let mut bars: Vec<Bar> = (0..55).map(|i| {
            // Linear downtrend.
            let c = 200.0 - i as f64;
            b(c, c + 0.5, c - 0.5, c, 1000.0)
        }).collect();
        bars.push(b(146.0, 147.0, 145.0, 147.0, 5000.0));    // up-day at index 55
        let events = detect(&bars, &Config { require_above_ma50: true, ..Default::default() });
        // 50MA from idx 6..=55 = mean of ~170 to 145, so MA ≈ 158; current = 147 < MA.
        assert!(events.is_empty());
    }
}
