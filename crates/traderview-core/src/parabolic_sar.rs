//! Parabolic SAR — Welles Wilder's stop-and-reverse trailing stop.
//!
//! Per bar:
//!   SAR_t = SAR_{t-1} + AF × (EP - SAR_{t-1})
//!
//! Where:
//!   AF (acceleration factor): starts at 0.02, increments by 0.02 each
//!     new extreme, capped at 0.20.
//!   EP (extreme point): highest high (long trend) or lowest low (short).
//!
//! On reversal, SAR flips and AF resets to 0.02.
//!
//! Classic trailing-stop indicator — when price crosses SAR, exit and
//! reverse. Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Trend {
    Up,
    Down,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct SarPoint {
    pub sar: f64,
    pub af: f64,
    pub ep: f64,
    pub trend_up: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct SarConfig {
    pub af_start: f64,
    pub af_increment: f64,
    pub af_max: f64,
}

impl Default for SarConfig {
    fn default() -> Self {
        Self {
            af_start: 0.02,
            af_increment: 0.02,
            af_max: 0.20,
        }
    }
}

pub fn compute(bars: &[Bar], cfg: &SarConfig) -> Vec<SarPoint> {
    let n = bars.len();
    let mut out = vec![SarPoint::default(); n];
    if n < 2 {
        return out;
    }
    // Seed: assume initial uptrend.
    let mut trend_up = true;
    let mut sar = bars[0].low;
    let mut ep = bars[0].high;
    let mut af = cfg.af_start;
    out[0] = SarPoint {
        sar,
        af,
        ep,
        trend_up,
    };
    for i in 1..n {
        let prev_high = bars[i - 1].high;
        let prev_low = bars[i - 1].low;
        let h = bars[i].high;
        let l = bars[i].low;
        let mut new_sar = sar + af * (ep - sar);
        if trend_up {
            // SAR can't be above prior 2 bars' lows.
            let lo2 = if i >= 2 {
                bars[i - 2].low.min(prev_low)
            } else {
                prev_low
            };
            if new_sar > lo2 {
                new_sar = lo2;
            }
            if l < new_sar {
                // Reversal to downtrend.
                trend_up = false;
                new_sar = ep;
                ep = l;
                af = cfg.af_start;
            } else if h > ep {
                ep = h;
                af = (af + cfg.af_increment).min(cfg.af_max);
            }
        } else {
            // SAR can't be below prior 2 bars' highs.
            let hi2 = if i >= 2 {
                bars[i - 2].high.max(prev_high)
            } else {
                prev_high
            };
            if new_sar < hi2 {
                new_sar = hi2;
            }
            if h > new_sar {
                trend_up = true;
                new_sar = ep;
                ep = h;
                af = cfg.af_start;
            } else if l < ep {
                ep = l;
                af = (af + cfg.af_increment).min(cfg.af_max);
            }
        }
        sar = new_sar;
        out[i] = SarPoint {
            sar,
            af,
            ep,
            trend_up,
        };
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64) -> Bar {
        Bar { high: h, low: l }
    }

    #[test]
    fn empty_returns_empty() {
        assert!(compute(&[], &SarConfig::default()).is_empty());
    }

    #[test]
    fn single_bar_returns_empty_sar_only_seed() {
        let out = compute(&[b(100.0, 99.0)], &SarConfig::default());
        // Length matches; sar at idx 0 is default-init zero (no compute loop).
        assert_eq!(out.len(), 1);
    }

    #[test]
    fn strong_uptrend_keeps_trend_up_true() {
        let bars: Vec<Bar> = (1..=20)
            .map(|i| {
                let c = 100.0 + i as f64;
                b(c + 1.0, c - 1.0)
            })
            .collect();
        let out = compute(&bars, &SarConfig::default());
        let last = out.last().unwrap();
        assert!(last.trend_up, "consistent uptrend → trend_up = true");
    }

    #[test]
    fn strong_downtrend_keeps_trend_up_false() {
        let bars: Vec<Bar> = (1..=20)
            .map(|i| {
                let c = 200.0 - i as f64;
                b(c + 1.0, c - 1.0)
            })
            .collect();
        let out = compute(&bars, &SarConfig::default());
        let last = out.last().unwrap();
        assert!(!last.trend_up);
    }

    #[test]
    fn af_increments_with_new_extremes() {
        let bars: Vec<Bar> = (1..=20)
            .map(|i| {
                let c = 100.0 + i as f64;
                b(c + 1.0, c - 1.0)
            })
            .collect();
        let out = compute(&bars, &SarConfig::default());
        let last = out.last().unwrap();
        assert!(last.af > 0.02, "AF should have grown above start");
    }

    #[test]
    fn af_capped_at_max() {
        // 50 bars of new highs should saturate AF.
        let bars: Vec<Bar> = (1..=50)
            .map(|i| {
                let c = 100.0 + i as f64;
                b(c + 1.0, c - 1.0)
            })
            .collect();
        let cfg = SarConfig::default();
        let out = compute(&bars, &cfg);
        let last = out.last().unwrap();
        assert!(last.af <= cfg.af_max + 1e-9);
        assert!(last.af >= cfg.af_max - 1e-9, "AF should saturate at max");
    }

    #[test]
    fn reversal_flips_trend_and_resets_af() {
        // Up trend then sudden crash.
        let mut bars: Vec<Bar> = (1..=15)
            .map(|i| {
                let c = 100.0 + i as f64;
                b(c + 1.0, c - 1.0)
            })
            .collect();
        bars.push(b(80.0, 70.0)); // huge gap down
        let out = compute(&bars, &SarConfig::default());
        let last = out.last().unwrap();
        assert!(!last.trend_up, "gap-down should reverse to downtrend");
        assert!(
            (last.af - 0.02).abs() < 1e-9,
            "AF resets to start on reversal"
        );
    }

    #[test]
    fn ep_tracks_highest_high_in_uptrend() {
        let bars: Vec<Bar> = (1..=10)
            .map(|i| {
                let c = 100.0 + i as f64;
                b(c + 1.0, c - 1.0)
            })
            .collect();
        let out = compute(&bars, &SarConfig::default());
        let last = out.last().unwrap();
        // Last bar's high = 100 + 10 + 1 = 111.
        assert_eq!(last.ep, 111.0);
    }
}
