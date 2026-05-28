//! Gap Classifier — Edwards & Magee gap taxonomy.
//!
//! Per bar, detects opening gaps and classifies into:
//!   - Common: small gap in a trading range, often filled same day
//!   - Breakaway: gap out of consolidation, marks start of new trend
//!   - Runaway: gap mid-trend with strong volume — measures trend continuation
//!   - Exhaustion: gap at extreme of trend, often closed quickly — trend exhaustion
//!
//! Heuristics:
//!   gap_size = open_t - close_{t-1}
//!   gap_pct = gap_size / close_{t-1}
//!
//!   Common: |gap_pct| < 1% (small) OR no trend
//!   Breakaway: |gap_pct| ≥ 1%, prior 20 bars in tight range
//!     (range / mean(close) < 5%), trend now breaks out
//!   Runaway: |gap_pct| ≥ 1%, prior 20 bars in same direction trend
//!     as gap, volume > 1.5× avg
//!   Exhaustion: |gap_pct| ≥ 1%, mid/extended trend (consecutive
//!     direction bars ≥ 5), this gap gets filled within `fill_bars`
//!
//! Pure compute. Defaults: lookback = 20, fill_bars = 5.
//! Companion to `gap_fill_stats`, `fair_value_gap`,
//! `breakout_detector`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar { pub high: f64, pub low: f64, pub close: f64, pub open: f64, pub volume: f64 }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum GapKind {
    #[default]
    None,
    Common,
    Breakaway,
    Runaway,
    Exhaustion,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GapClassifierReport {
    pub gap_kind: Vec<GapKind>,
    pub gap_pct: Vec<Option<f64>>,
    pub lookback: usize,
    pub fill_bars: usize,
}

pub fn compute(bars: &[Bar], lookback: usize, fill_bars: usize) -> GapClassifierReport {
    let n = bars.len();
    let mut report = GapClassifierReport {
        gap_kind: vec![GapKind::None; n],
        gap_pct: vec![None; n],
        lookback,
        fill_bars,
    };
    if lookback < 5 || fill_bars < 1 || n < lookback + fill_bars + 1 { return report; }
    if bars.iter().any(|b| !b.high.is_finite() || !b.low.is_finite()
        || !b.close.is_finite() || !b.open.is_finite() || !b.volume.is_finite()
        || b.volume < 0.0) {
        return report;
    }
    for i in lookback..(n - fill_bars) {
        let prev_close = bars[i - 1].close;
        let cur = bars[i];
        if prev_close == 0.0 { continue; }
        let gap_size = cur.open - prev_close;
        let gap_pct = gap_size / prev_close;
        report.gap_pct[i] = Some(gap_pct);
        let abs_pct = gap_pct.abs();
        if abs_pct < 0.01 {
            report.gap_kind[i] = GapKind::Common;
            continue;
        }
        let win = &bars[i - lookback..i];
        let win_high = win.iter().fold(f64::NEG_INFINITY, |a, b| a.max(b.high));
        let win_low = win.iter().fold(f64::INFINITY, |a, b| a.min(b.low));
        let win_mean_close: f64 = win.iter().map(|b| b.close).sum::<f64>() / lookback as f64;
        let win_range_pct = if win_mean_close > 0.0 {
            (win_high - win_low) / win_mean_close
        } else { 0.0 };
        let avg_vol: f64 = win.iter().map(|b| b.volume).sum::<f64>() / lookback as f64;
        let direction_run = consecutive_run(win);
        // Exhaustion check first: long trend AND gap gets filled within window.
        let gap_filled = if gap_pct > 0.0 {
            bars[i + 1..=(i + fill_bars).min(n - 1)].iter().any(|b| b.low <= prev_close)
        } else {
            bars[i + 1..=(i + fill_bars).min(n - 1)].iter().any(|b| b.high >= prev_close)
        };
        if direction_run.abs() >= 5 && gap_filled {
            report.gap_kind[i] = GapKind::Exhaustion;
            continue;
        }
        // Breakaway: tight prior range.
        if win_range_pct < 0.05 {
            report.gap_kind[i] = GapKind::Breakaway;
            continue;
        }
        // Runaway: trend in same direction as gap + volume confirmation.
        let same_direction = (gap_pct > 0.0 && direction_run > 0)
            || (gap_pct < 0.0 && direction_run < 0);
        if same_direction && cur.volume > avg_vol * 1.5 {
            report.gap_kind[i] = GapKind::Runaway;
            continue;
        }
        report.gap_kind[i] = GapKind::Common;
    }
    report
}

fn consecutive_run(win: &[Bar]) -> i32 {
    if win.len() < 2 { return 0; }
    let mut run = 0_i32;
    for i in 1..win.len() {
        if win[i].close > win[i - 1].close {
            run = if run > 0 { run + 1 } else { 1 };
        } else if win[i].close < win[i - 1].close {
            run = if run < 0 { run - 1 } else { -1 };
        } else {
            run = 0;
        }
    }
    run
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(o: f64, h: f64, l: f64, c: f64, v: f64) -> Bar {
        Bar { open: o, high: h, low: l, close: c, volume: v }
    }

    #[test]
    fn invalid_inputs_return_empty() {
        let bars = vec![b(100.0, 101.0, 99.0, 100.0, 1000.0); 30];
        let r = compute(&bars, 1, 5);
        assert!(r.gap_kind.iter().all(|k| *k == GapKind::None));
        let r2 = compute(&bars[..5], 20, 5);
        assert!(r2.gap_kind.iter().all(|k| *k == GapKind::None));
    }

    #[test]
    fn nan_or_negative_returns_empty() {
        let mut bars = vec![b(100.0, 101.0, 99.0, 100.0, 1000.0); 30];
        bars[5] = b(100.0, f64::NAN, 99.0, 100.0, 1000.0);
        let r = compute(&bars, 20, 5);
        assert!(r.gap_kind.iter().all(|k| *k == GapKind::None));
    }

    #[test]
    fn small_gap_classified_common() {
        let mut bars = vec![b(100.0, 101.0, 99.0, 100.0, 1000.0); 20];
        // Bar 20: opens 100.5 (0.5% gap up).
        bars.push(b(100.5, 101.0, 100.0, 100.7, 1000.0));
        bars.extend(vec![b(100.5, 101.0, 100.0, 100.7, 1000.0); 5]);
        let r = compute(&bars, 20, 5);
        assert_eq!(r.gap_kind[20], GapKind::Common);
    }

    #[test]
    fn large_gap_in_tight_range_classified_breakaway() {
        // 20 bars in tight range (100.0-101.0), then 3% gap up.
        let mut bars = vec![b(100.0, 101.0, 99.5, 100.0, 1000.0); 20];
        bars.push(b(103.0, 105.0, 102.5, 104.0, 1000.0));
        bars.extend(vec![b(104.0, 105.0, 103.5, 104.5, 1000.0); 5]);
        let r = compute(&bars, 20, 5);
        assert_eq!(r.gap_kind[20], GapKind::Breakaway);
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![b(100.0, 101.0, 99.0, 100.0, 1000.0); 30];
        let r = compute(&bars, 20, 5);
        assert_eq!(r.gap_kind.len(), 30);
        assert_eq!(r.gap_pct.len(), 30);
    }
}
