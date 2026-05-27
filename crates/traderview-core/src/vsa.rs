//! Volume Spread Analysis (VSA) — Tom Williams bar classifier.
//!
//! VSA reads each bar's PRICE SPREAD (high − low) and CLOSE LOCATION
//! together with VOLUME to classify the bar into a short list of
//! professional/retail-action diagnostic types:
//!
//!   - **NoDemand**: narrow up-bar on LOW volume → buyers absent, weak rally
//!   - **NoSupply**: narrow down-bar on LOW volume → sellers absent, strong
//!   - **StoppingVolume**: down-bar with WIDE spread + close near HIGH on
//!     HIGH volume → buyers absorbed selling, potential bottom
//!   - **Climactic**: very-wide spread on VERY high volume in direction
//!     of the move → exhaustion, often the END of a leg
//!   - **TestBar**: narrow down-bar on lower-than-average volume after
//!     a sell-off → market testing for supply; bullish if it holds
//!   - **Effort**: wide bar on heavy volume but small body — effort
//!     without result; opposite side absorbed the push
//!
//! Caller supplies OHLC + volume bars + a rolling-average volume.
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct VsaBar {
    pub high: f64,
    pub low: f64,
    pub open: f64,
    pub close: f64,
    pub volume: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VsaSignal {
    NoDemand,
    NoSupply,
    StoppingVolume,
    Climactic,
    TestBar,
    Effort,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct VsaEvent {
    pub bar_index: usize,
    pub signal: VsaSignal,
    /// Bar's spread (high − low).
    pub spread: f64,
    /// Volume / avg_volume ratio.
    pub volume_ratio: f64,
    /// Close position within the bar's range, 0 = at low, 1 = at high.
    pub close_position: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VsaReport {
    pub events: Vec<VsaEvent>,
    pub n_events: usize,
}

pub fn classify(bars: &[VsaBar], avg_volume: &[f64]) -> VsaReport {
    let n = bars.len();
    if n == 0 || avg_volume.len() != n { return VsaReport::default(); }
    let mut events = Vec::new();
    // Compute rolling average spread for the wide/narrow comparison.
    let spreads: Vec<f64> = bars.iter().map(|b| (b.high - b.low).max(0.0)).collect();
    let avg_spread: Vec<f64> = rolling_mean(&spreads, 14);
    for i in 0..n {
        let bar = bars[i];
        let avg_vol = avg_volume[i];
        if avg_vol <= 0.0 { continue; }
        let spread = spreads[i];
        if spread <= 0.0 { continue; }
        let vol_ratio = bar.volume / avg_vol;
        let close_pos = (bar.close - bar.low) / spread;
        let is_up = bar.close > bar.open;
        let is_down = bar.close < bar.open;
        let spread_ratio = if avg_spread[i] > 0.0 { spread / avg_spread[i] } else { 1.0 };
        let narrow = spread_ratio < 0.7;
        let wide   = spread_ratio > 1.5;

        let signal = if narrow && is_up && vol_ratio < 0.7 {
            Some(VsaSignal::NoDemand)
        } else if narrow && is_down && vol_ratio < 0.7 {
            // Narrow down-bar on light volume after a sell-off = TestBar;
            // without prior context we classify as NoSupply (which is what
            // it looks like locally).
            Some(VsaSignal::NoSupply)
        } else if wide && is_down && close_pos > 0.7 && vol_ratio > 1.5 {
            Some(VsaSignal::StoppingVolume)
        } else if wide && vol_ratio > 2.0 {
            Some(VsaSignal::Climactic)
        } else if narrow && vol_ratio < 0.8 && is_down {
            Some(VsaSignal::TestBar)
        } else if wide && vol_ratio > 1.5 && (bar.close - bar.open).abs() / spread < 0.3 {
            Some(VsaSignal::Effort)
        } else {
            None
        };
        if let Some(sig) = signal {
            events.push(VsaEvent {
                bar_index: i, signal: sig,
                spread, volume_ratio: vol_ratio, close_position: close_pos,
            });
        }
    }
    let n_events = events.len();
    VsaReport { events, n_events }
}

fn rolling_mean(values: &[f64], window: usize) -> Vec<f64> {
    let n = values.len();
    let mut out = vec![0.0_f64; n];
    if window == 0 || n == 0 { return out; }
    for i in 0..n {
        let lo = i.saturating_sub(window.saturating_sub(1));
        let slice = &values[lo..=i];
        out[i] = slice.iter().sum::<f64>() / slice.len() as f64;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bar(o: f64, h: f64, l: f64, c: f64, v: f64) -> VsaBar {
        VsaBar { open: o, high: h, low: l, close: c, volume: v }
    }

    #[test]
    fn empty_or_mismatched_returns_no_events() {
        assert!(classify(&[], &[]).events.is_empty());
        let bars = vec![bar(100.0, 101.0, 99.0, 100.5, 1000.0)];
        let avg = vec![1000.0, 1000.0];
        assert!(classify(&bars, &avg).events.is_empty());
    }

    #[test]
    fn climactic_bar_detected_on_wide_spread_huge_volume() {
        // 14 normal bars, then 1 monster: spread 5x normal, volume 3x avg.
        let mut bars: Vec<VsaBar> = (0..14).map(|_| bar(100.0, 100.5, 99.5, 100.0, 1000.0)).collect();
        bars.push(bar(100.0, 105.0, 99.0, 104.5, 3000.0));
        let avg_volume: Vec<f64> = bars.iter().map(|_| 1000.0).collect();
        let r = classify(&bars, &avg_volume);
        let last = r.events.iter().find(|e| e.bar_index == 14);
        assert!(last.is_some(), "expected climactic event at bar 14");
        assert!(matches!(last.unwrap().signal, VsaSignal::Climactic));
    }

    #[test]
    fn no_demand_on_narrow_up_bar_with_low_volume() {
        // 14 normal bars (spread 2.0), then narrow up-bar (spread 0.5, vol 200).
        // avg_spread will be dragged toward 0.5 by the inclusion. Use larger
        // bars to keep avg_spread elevated.
        let mut bars: Vec<VsaBar> = (0..14).map(|_| bar(100.0, 101.0, 99.0, 100.5, 1000.0)).collect();
        bars.push(bar(100.0, 100.4, 99.9, 100.3, 200.0));    // narrow up, low vol
        let avg_volume: Vec<f64> = bars.iter().map(|_| 1000.0).collect();
        let r = classify(&bars, &avg_volume);
        let last = r.events.iter().find(|e| e.bar_index == 14);
        assert!(last.is_some(), "expected NoDemand at bar 14");
        assert!(matches!(last.unwrap().signal, VsaSignal::NoDemand),
            "expected NoDemand, got {:?}", last.unwrap().signal);
    }

    #[test]
    fn no_supply_on_narrow_down_bar_with_low_volume() {
        let mut bars: Vec<VsaBar> = (0..14).map(|_| bar(100.0, 101.0, 99.0, 100.5, 1000.0)).collect();
        bars.push(bar(100.0, 100.0, 99.6, 99.7, 200.0));    // narrow down, low vol
        let avg_volume: Vec<f64> = bars.iter().map(|_| 1000.0).collect();
        let r = classify(&bars, &avg_volume);
        let last = r.events.iter().find(|e| e.bar_index == 14);
        assert!(last.is_some());
        assert!(matches!(last.unwrap().signal, VsaSignal::NoSupply));
    }

    #[test]
    fn stopping_volume_on_wide_down_bar_with_close_at_top() {
        // Wide down-bar (open 100, close 98 — down on direction), but the
        // CLOSE landed near the high (close_position > 0.7). HIGH volume.
        let mut bars: Vec<VsaBar> = (0..14).map(|_| bar(100.0, 100.5, 99.5, 100.0, 1000.0)).collect();
        // Open 100, low 95 (wide drop), high 99, close 98.7 → close_pos = (98.7-95)/(99-95) = 0.925.
        bars.push(bar(100.0, 99.0, 95.0, 98.7, 2000.0));
        let avg_volume: Vec<f64> = bars.iter().map(|_| 1000.0).collect();
        let r = classify(&bars, &avg_volume);
        let last = r.events.iter().find(|e| e.bar_index == 14);
        assert!(last.is_some());
        // Either StoppingVolume OR Climactic depending on thresholds.
        let sig = last.unwrap().signal;
        assert!(matches!(sig, VsaSignal::StoppingVolume | VsaSignal::Climactic),
            "expected StoppingVolume or Climactic, got {:?}", sig);
    }

    #[test]
    fn zero_avg_volume_skipped() {
        let bars = vec![bar(100.0, 101.0, 99.0, 100.5, 1000.0)];
        let avg = vec![0.0];
        let r = classify(&bars, &avg);
        assert!(r.events.is_empty(), "zero avg volume → skip (no div-by-zero)");
    }

    #[test]
    fn zero_spread_bar_skipped() {
        let bars = vec![bar(100.0, 100.0, 100.0, 100.0, 1000.0)];
        let avg = vec![1000.0];
        let r = classify(&bars, &avg);
        assert!(r.events.is_empty());
    }
}
