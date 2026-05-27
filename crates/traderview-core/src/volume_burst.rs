//! Volume-burst detector — flags abnormal-volume bars.
//!
//! Liquidity events (earnings leak, news catalysts, halt-then-resume,
//! whale prints) show up as volume bars that are 3-5x the trailing
//! N-day average. Catching these in real time gives the trader the
//! cleanest signal that "something is happening" in a name they
//! weren't already watching.
//!
//! Caller supplies a (volume, close) series. We compare each bar's
//! volume to the rolling average of the prior `lookback` bars and
//! emit bars whose ratio exceeds the threshold.
//!
//! Pure compute. Distinct from `volume_flow::obv` (cumulative) and
//! `mfi` (price-direction-weighted). This module is purely a "loud
//! bar" detector.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct VolumeBar {
    pub volume: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurstConfig {
    /// Rolling-window length for the baseline average.
    pub lookback: usize,
    /// Minimum ratio (current_volume / avg_volume) to flag a burst.
    pub min_ratio: f64,
}

impl Default for BurstConfig {
    fn default() -> Self { Self { lookback: 20, min_ratio: 3.0 } }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BurstEvent {
    /// 0-based index into the input series where the burst was detected.
    pub bar_index: usize,
    pub volume: f64,
    pub avg_volume: f64,
    pub ratio: f64,
    pub close: f64,
    /// Tag distinguishing UP-bursts from DOWN-bursts based on close vs
    /// prior close. Pure-volume bursts on doji bars get "neutral".
    pub direction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BurstReport {
    pub events: Vec<BurstEvent>,
    pub n_bars: usize,
    pub n_bursts: usize,
    pub max_ratio: f64,
}

pub fn detect(bars: &[VolumeBar], cfg: &BurstConfig) -> BurstReport {
    let n = bars.len();
    if n == 0 || cfg.lookback == 0 || cfg.min_ratio <= 0.0 {
        return BurstReport { n_bars: n, ..Default::default() };
    }
    let mut events = Vec::new();
    let mut max_ratio = 0.0_f64;
    // Start scanning AT lookback so we have a full prior window. Use
    // strictly-prior bars (not current) to avoid the current bar dragging
    // its own average up.
    for i in cfg.lookback..n {
        let window = &bars[(i - cfg.lookback)..i];
        let sum: f64 = window.iter().map(|b| b.volume).sum();
        let avg = sum / cfg.lookback as f64;
        if avg <= 0.0 { continue; }
        let ratio = bars[i].volume / avg;
        if ratio > max_ratio { max_ratio = ratio; }
        if ratio >= cfg.min_ratio {
            let direction = if i > 0 {
                let prev_close = bars[i - 1].close;
                if bars[i].close > prev_close      { "up" }
                else if bars[i].close < prev_close { "down" }
                else                               { "neutral" }
            } else { "neutral" };
            events.push(BurstEvent {
                bar_index: i,
                volume: bars[i].volume,
                avg_volume: avg,
                ratio,
                close: bars[i].close,
                direction: direction.into(),
            });
        }
    }
    let n_bursts = events.len();
    BurstReport { events, n_bars: n, n_bursts, max_ratio }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bar(v: f64, c: f64) -> VolumeBar { VolumeBar { volume: v, close: c } }

    #[test]
    fn empty_input_returns_zero_report() {
        let r = detect(&[], &BurstConfig::default());
        assert_eq!(r.n_bars, 0);
        assert!(r.events.is_empty());
    }

    #[test]
    fn no_burst_in_flat_volume() {
        // 25 bars of constant 1000 volume → no bar exceeds 3× the average.
        let bars: Vec<VolumeBar> = (0..25).map(|i| bar(1000.0, 100.0 + i as f64 * 0.01)).collect();
        let r = detect(&bars, &BurstConfig::default());
        assert!(r.events.is_empty(), "flat volume should produce 0 bursts");
        assert!((r.max_ratio - 1.0).abs() < 1e-9);
    }

    #[test]
    fn detects_5x_spike() {
        // 20 bars at 1000, then a 5000-volume bar with higher close → up burst.
        let mut bars: Vec<VolumeBar> = (0..20).map(|_| bar(1000.0, 100.0)).collect();
        bars.push(bar(5000.0, 101.0));
        let r = detect(&bars, &BurstConfig::default());
        assert_eq!(r.events.len(), 1);
        assert_eq!(r.events[0].bar_index, 20);
        assert!((r.events[0].ratio - 5.0).abs() < 1e-9);
        assert_eq!(r.events[0].direction, "up");
    }

    #[test]
    fn down_burst_tagged_when_close_falls() {
        let mut bars: Vec<VolumeBar> = (0..20).map(|_| bar(1000.0, 100.0)).collect();
        bars.push(bar(5000.0, 99.0));
        let r = detect(&bars, &BurstConfig::default());
        assert_eq!(r.events[0].direction, "down");
    }

    #[test]
    fn neutral_burst_tagged_on_doji() {
        let mut bars: Vec<VolumeBar> = (0..20).map(|_| bar(1000.0, 100.0)).collect();
        bars.push(bar(5000.0, 100.0));
        let r = detect(&bars, &BurstConfig::default());
        assert_eq!(r.events[0].direction, "neutral");
    }

    #[test]
    fn lookback_window_doesnt_include_current_bar() {
        // If the current bar's volume were included in its own baseline,
        // a 5000-vol bar with 19 prior 1000s would average (19*1000+5000)/20 = 1200,
        // giving a ratio of only 4.17. Strict-prior averaging gives ratio = 5.0.
        let mut bars: Vec<VolumeBar> = (0..20).map(|_| bar(1000.0, 100.0)).collect();
        bars.push(bar(5000.0, 101.0));
        let r = detect(&bars, &BurstConfig { lookback: 20, min_ratio: 3.0 });
        assert!((r.events[0].ratio - 5.0).abs() < 1e-9,
            "current-bar exclusion gives ratio 5.0, got {}", r.events[0].ratio);
    }

    #[test]
    fn zero_lookback_or_threshold_returns_empty() {
        let bars = vec![bar(1000.0, 100.0); 5];
        let r0 = detect(&bars, &BurstConfig { lookback: 0, min_ratio: 3.0 });
        assert!(r0.events.is_empty());
        let rt = detect(&bars, &BurstConfig { lookback: 3, min_ratio: 0.0 });
        assert!(rt.events.is_empty());
    }

    #[test]
    fn max_ratio_tracked_even_below_threshold() {
        // A 2.5× spike doesn't fire (threshold 3.0) but should still bump max_ratio.
        let mut bars: Vec<VolumeBar> = (0..20).map(|_| bar(1000.0, 100.0)).collect();
        bars.push(bar(2500.0, 101.0));
        let r = detect(&bars, &BurstConfig { lookback: 20, min_ratio: 3.0 });
        assert!(r.events.is_empty());
        assert!((r.max_ratio - 2.5).abs() < 1e-9);
    }
}
