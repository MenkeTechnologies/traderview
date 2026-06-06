//! Tom DeMark TD Sequential — exhaustion / reversal indicator.
//!
//! Two stages on the daily bar series:
//!
//!   **Setup** (9 bars):
//!     bullish_setup_t triggered when close_t < close_{t−4} for 9
//!     consecutive bars (sells exhausted → likely bottom).
//!     bearish_setup_t when close_t > close_{t−4} for 9 consecutive bars.
//!
//!   **Countdown** (13 bars, starts when setup completes):
//!     bullish countdown bar = bar where close ≤ low_{t−2}
//!     bearish countdown bar = bar where close ≥ high_{t−2}
//!     13 such bars (not necessarily consecutive) within a maximum
//!     `countdown_max_bars` window completes the countdown — the
//!     canonical exhaustion signal.
//!
//! Pure compute. Output: per-bar setup counter (0..9), completion flag,
//! countdown counter (0..13), exhaustion flag.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Max bars to extend countdown after setup completes.
    pub countdown_max_bars: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            countdown_max_bars: 60,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TdReport {
    pub bullish_setup_counter: Vec<u8>,
    pub bearish_setup_counter: Vec<u8>,
    pub bullish_setup_completed: Vec<bool>,
    pub bearish_setup_completed: Vec<bool>,
    pub bullish_countdown: Vec<u8>,
    pub bearish_countdown: Vec<u8>,
    pub bullish_exhaustion: Vec<usize>,
    pub bearish_exhaustion: Vec<usize>,
}

pub fn analyze(bars: &[Bar], cfg: &Config) -> TdReport {
    let n = bars.len();
    let mut r = TdReport {
        bullish_setup_counter: vec![0; n],
        bearish_setup_counter: vec![0; n],
        bullish_setup_completed: vec![false; n],
        bearish_setup_completed: vec![false; n],
        bullish_countdown: vec![0; n],
        bearish_countdown: vec![0; n],
        bullish_exhaustion: Vec::new(),
        bearish_exhaustion: Vec::new(),
    };
    if n < 5 || cfg.countdown_max_bars == 0 {
        return r;
    }
    // Stage 1: setup.
    let mut bull_run = 0u8;
    let mut bear_run = 0u8;
    for i in 4..n {
        let c = bars[i].close;
        let c4 = bars[i - 4].close;
        if !c.is_finite() || !c4.is_finite() {
            bull_run = 0;
            bear_run = 0;
            continue;
        }
        if c < c4 {
            bull_run = bull_run.saturating_add(1);
        } else {
            bull_run = 0;
        }
        if c > c4 {
            bear_run = bear_run.saturating_add(1);
        } else {
            bear_run = 0;
        }
        r.bullish_setup_counter[i] = bull_run.min(9);
        r.bearish_setup_counter[i] = bear_run.min(9);
        if bull_run >= 9 {
            r.bullish_setup_completed[i] = true;
            bull_run = 0; // reset to allow next setup
        }
        if bear_run >= 9 {
            r.bearish_setup_completed[i] = true;
            bear_run = 0;
        }
    }
    // Stage 2: countdown — for each completed setup, count qualifying
    // bars over the next `countdown_max_bars`.
    fn run_countdown(
        bars: &[Bar],
        completed: &[bool],
        counter: &mut [u8],
        exhaustion: &mut Vec<usize>,
        bullish: bool,
        max_bars: usize,
    ) {
        let n = bars.len();
        let mut active: Vec<(usize, u8)> = Vec::new(); // (setup_complete_idx, count_so_far)
        for i in 0..n {
            if completed[i] {
                active.push((i, 0));
            }
            // Retain only those within window.
            active.retain(|(start, _)| i.saturating_sub(*start) < max_bars);
            // For each active countdown, check current bar.
            if i >= 2 {
                let qualifies = if bullish {
                    bars[i].close.is_finite()
                        && bars[i - 2].low.is_finite()
                        && bars[i].close <= bars[i - 2].low
                } else {
                    bars[i].close.is_finite()
                        && bars[i - 2].high.is_finite()
                        && bars[i].close >= bars[i - 2].high
                };
                if qualifies {
                    for entry in active.iter_mut() {
                        if entry.0 < i {
                            entry.1 = entry.1.saturating_add(1);
                        }
                    }
                }
            }
            // Highest active count is reported.
            let max_count = active.iter().map(|(_, c)| *c).max().unwrap_or(0);
            counter[i] = max_count.min(13);
            // Check for 13 completion.
            let mut completed_this_bar = false;
            active.retain(|(_, c)| {
                if *c >= 13 {
                    if !completed_this_bar {
                        exhaustion.push(i);
                        completed_this_bar = true;
                    }
                    false
                } else {
                    true
                }
            });
        }
    }
    run_countdown(
        bars,
        &r.bullish_setup_completed,
        &mut r.bullish_countdown,
        &mut r.bullish_exhaustion,
        true,
        cfg.countdown_max_bars,
    );
    run_countdown(
        bars,
        &r.bearish_setup_completed,
        &mut r.bearish_countdown,
        &mut r.bearish_exhaustion,
        false,
        cfg.countdown_max_bars,
    );
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(c: f64) -> Bar {
        Bar {
            high: c + 0.5,
            low: c - 0.5,
            close: c,
        }
    }

    #[test]
    fn empty_returns_default() {
        let r = analyze(&[], &Config::default());
        assert!(r.bullish_setup_counter.is_empty());
    }

    #[test]
    fn invalid_config_returns_default() {
        let bars = vec![b(100.0); 50];
        let r = analyze(
            &bars,
            &Config {
                countdown_max_bars: 0,
            },
        );
        assert!(r.bullish_exhaustion.is_empty());
    }

    #[test]
    fn too_short_returns_default() {
        let bars = vec![b(100.0); 4];
        let r = analyze(&bars, &Config::default());
        assert!(r.bullish_setup_counter.iter().all(|x| *x == 0));
    }

    #[test]
    fn falling_series_triggers_bullish_setup() {
        // Strictly falling closes → close_t < close_{t−4} always → setup completes.
        let bars: Vec<Bar> = (0..30).map(|i| b(200.0 - i as f64)).collect();
        let r = analyze(&bars, &Config::default());
        assert!(r.bullish_setup_completed.iter().any(|x| *x));
    }

    #[test]
    fn rising_series_triggers_bearish_setup() {
        let bars: Vec<Bar> = (0..30).map(|i| b(100.0 + i as f64)).collect();
        let r = analyze(&bars, &Config::default());
        assert!(r.bearish_setup_completed.iter().any(|x| *x));
    }

    #[test]
    fn nan_close_resets_run() {
        let mut bars: Vec<Bar> = (0..20).map(|i| b(200.0 - i as f64)).collect();
        bars[10].close = f64::NAN;
        let r = analyze(&bars, &Config::default());
        // Counter at bar 10 should be 0 (reset on NaN).
        assert_eq!(r.bullish_setup_counter[10], 0);
    }

    #[test]
    fn setup_counter_max_is_9() {
        let bars: Vec<Bar> = (0..50).map(|i| b(200.0 - i as f64)).collect();
        let r = analyze(&bars, &Config::default());
        assert!(r.bullish_setup_counter.iter().all(|x| *x <= 9));
    }

    #[test]
    fn bullish_countdown_completes_for_strong_downtrend() {
        // 100 bars steadily falling — should produce 13-count completion.
        let bars: Vec<Bar> = (0..100).map(|i| b(200.0 - i as f64 * 0.5)).collect();
        let r = analyze(&bars, &Config::default());
        assert!(
            !r.bullish_exhaustion.is_empty(),
            "long downtrend should produce bullish exhaustion countdown"
        );
    }
}
