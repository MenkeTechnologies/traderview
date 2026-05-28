//! Bump-and-Run Reversal (BARR) — Thomas Bulkowski.
//!
//! A three-phase reversal pattern:
//!
//!   1. **Lead-in phase**: slow uptrend over a sustained window; price
//!      moves up at a moderate slope along a base "lead-in line".
//!   2. **Bump phase**: trend accelerates sharply; slope at least
//!      `bump_slope_multiplier`× the lead-in slope, and the peak height
//!      above the lead-in line ≥ `bump_height_multiplier`× the lead-in
//!      range.
//!   3. **Run phase**: price breaks back DOWN through the lead-in
//!      trendline (BARR-top) or up through it (BARR-bottom for the
//!      symmetric case on downtrends).
//!
//! Detection heuristic:
//!   - Fit a least-squares line to the first `lead_in_bars` bars (the
//!     "lead-in line").
//!   - Find the max distance of subsequent bars ABOVE the lead-in line
//!     (the "bump peak").
//!   - If bump peak > height threshold AND the most recent bars have
//!     crossed back below the line → BARR-top confirmed.
//!
//! Pure compute. Returns the bump peak index and the run-confirmation
//! index when both conditions hit. Companion to `cup_and_handle`,
//! `head_shoulders`, `wedge_pattern`, `triple_top_bottom`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar { pub high: f64, pub low: f64, pub close: f64 }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PatternKind { BarrTop, BarrBottom }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BarrCandidate {
    pub kind: PatternKind,
    pub lead_in_start: usize,
    pub lead_in_end: usize,
    pub bump_peak_index: usize,
    pub bump_peak_price: f64,
    pub run_confirmation_index: usize,
    pub lead_in_slope: f64,
    pub lead_in_intercept: f64,
    pub bump_height_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub lead_in_bars: usize,
    pub bump_height_multiplier: f64,
}

impl Default for Config {
    fn default() -> Self {
        Self { lead_in_bars: 20, bump_height_multiplier: 2.0 }
    }
}

pub fn detect(bars: &[Bar], cfg: &Config) -> Vec<BarrCandidate> {
    let n = bars.len();
    let mut out = Vec::new();
    if cfg.lead_in_bars < 5 || n < cfg.lead_in_bars + 5
        || cfg.bump_height_multiplier <= 0.0 {
        return out;
    }
    if bars.iter().any(|b| !b.high.is_finite() || !b.low.is_finite() || !b.close.is_finite()) {
        return out;
    }
    // BARR-top: lead-in uptrend → bump above lead-in line → break back below.
    if let Some(c) = detect_top(bars, cfg) { out.push(c); }
    // BARR-bottom (mirror): lead-in downtrend → bump below → break back above.
    if let Some(c) = detect_bottom(bars, cfg) { out.push(c); }
    out
}

#[allow(clippy::needless_range_loop)]
fn detect_top(bars: &[Bar], cfg: &Config) -> Option<BarrCandidate> {
    let n = bars.len();
    let l = cfg.lead_in_bars;
    let lead: Vec<f64> = bars[..l].iter().map(|b| b.close).collect();
    let (slope, intercept) = ols_line(&lead)?;
    if slope <= 0.0 { return None; }    // lead-in must be uptrend
    let lead_range = lead.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
        - lead.iter().cloned().fold(f64::INFINITY, f64::min);
    if lead_range <= 0.0 { return None; }
    // Scan post-lead-in for bump (max distance above lead-in line).
    let mut peak_idx = l;
    let mut peak_dist = f64::NEG_INFINITY;
    for i in l..n {
        let line = slope * i as f64 + intercept;
        let dist = bars[i].high - line;
        if dist > peak_dist {
            peak_dist = dist;
            peak_idx = i;
        }
    }
    let bump_height_ratio = peak_dist / lead_range;
    if bump_height_ratio < cfg.bump_height_multiplier { return None; }
    // Run confirmation: any bar after the bump peak closes below the
    // lead-in line.
    let mut run_idx = None;
    for i in (peak_idx + 1)..n {
        let line = slope * i as f64 + intercept;
        if bars[i].close < line { run_idx = Some(i); break; }
    }
    let run_confirmation_index = run_idx?;
    Some(BarrCandidate {
        kind: PatternKind::BarrTop,
        lead_in_start: 0,
        lead_in_end: l - 1,
        bump_peak_index: peak_idx,
        bump_peak_price: bars[peak_idx].high,
        run_confirmation_index,
        lead_in_slope: slope,
        lead_in_intercept: intercept,
        bump_height_ratio,
    })
}

#[allow(clippy::needless_range_loop)]
fn detect_bottom(bars: &[Bar], cfg: &Config) -> Option<BarrCandidate> {
    let n = bars.len();
    let l = cfg.lead_in_bars;
    let lead: Vec<f64> = bars[..l].iter().map(|b| b.close).collect();
    let (slope, intercept) = ols_line(&lead)?;
    if slope >= 0.0 { return None; }    // lead-in must be downtrend
    let lead_range = lead.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
        - lead.iter().cloned().fold(f64::INFINITY, f64::min);
    if lead_range <= 0.0 { return None; }
    let mut trough_idx = l;
    let mut trough_dist = f64::NEG_INFINITY;
    for i in l..n {
        let line = slope * i as f64 + intercept;
        let dist = line - bars[i].low;
        if dist > trough_dist {
            trough_dist = dist;
            trough_idx = i;
        }
    }
    let bump_height_ratio = trough_dist / lead_range;
    if bump_height_ratio < cfg.bump_height_multiplier { return None; }
    let mut run_idx = None;
    for i in (trough_idx + 1)..n {
        let line = slope * i as f64 + intercept;
        if bars[i].close > line { run_idx = Some(i); break; }
    }
    let run_confirmation_index = run_idx?;
    Some(BarrCandidate {
        kind: PatternKind::BarrBottom,
        lead_in_start: 0,
        lead_in_end: l - 1,
        bump_peak_index: trough_idx,
        bump_peak_price: bars[trough_idx].low,
        run_confirmation_index,
        lead_in_slope: slope,
        lead_in_intercept: intercept,
        bump_height_ratio,
    })
}

fn ols_line(y: &[f64]) -> Option<(f64, f64)> {
    let n = y.len();
    if n < 2 { return None; }
    let n_f = n as f64;
    let x_mean = (n_f - 1.0) / 2.0;
    let y_mean: f64 = y.iter().sum::<f64>() / n_f;
    let mut sxx = 0.0_f64;
    let mut sxy = 0.0_f64;
    for (i, yi) in y.iter().enumerate() {
        let dx = i as f64 - x_mean;
        let dy = yi - y_mean;
        sxx += dx * dx;
        sxy += dx * dy;
    }
    if sxx <= 0.0 { return None; }
    let slope = sxy / sxx;
    let intercept = y_mean - slope * x_mean;
    Some((slope, intercept))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64, c: f64) -> Bar { Bar { high: h, low: l, close: c } }

    #[test]
    fn empty_returns_empty() {
        assert!(detect(&[], &Config::default()).is_empty());
    }

    #[test]
    fn invalid_config_returns_empty() {
        let bars: Vec<_> = (0..50).map(|_| b(101.0, 99.0, 100.0)).collect();
        let cfg = Config { lead_in_bars: 0, ..Default::default() };
        assert!(detect(&bars, &cfg).is_empty());
        let cfg2 = Config { bump_height_multiplier: 0.0, ..Default::default() };
        assert!(detect(&bars, &cfg2).is_empty());
    }

    #[test]
    fn flat_market_no_pattern() {
        let bars = vec![b(101.0, 99.0, 100.0); 50];
        assert!(detect(&bars, &Config::default()).is_empty());
    }

    #[test]
    fn classic_barr_top_detected() {
        // Lead-in: gentle uptrend 100→110 over 20 bars.
        // Bump: vertical spike to 140 over 5 bars (huge bump height).
        // Run: price drops back below lead-in line.
        let mut bars: Vec<Bar> = Vec::new();
        for i in 0..20 {
            let mid = 100.0 + i as f64 * 0.5;
            bars.push(b(mid + 0.5, mid - 0.5, mid));
        }
        for i in 0..5 {
            let mid = 110.0 + i as f64 * 6.0;
            bars.push(b(mid + 1.0, mid - 1.0, mid));
        }
        for i in 0..10 {
            let mid = 140.0 - i as f64 * 5.0;    // run-down
            bars.push(b(mid + 1.0, mid - 1.0, mid));
        }
        let cands = detect(&bars, &Config { lead_in_bars: 20, bump_height_multiplier: 1.5 });
        assert!(cands.iter().any(|c| c.kind == PatternKind::BarrTop),
            "expected BARR-top, got {cands:?}");
    }

    #[test]
    fn classic_barr_bottom_detected() {
        let mut bars: Vec<Bar> = Vec::new();
        for i in 0..20 {
            let mid = 100.0 - i as f64 * 0.5;
            bars.push(b(mid + 0.5, mid - 0.5, mid));
        }
        for i in 0..5 {
            let mid = 90.0 - i as f64 * 6.0;
            bars.push(b(mid + 1.0, mid - 1.0, mid));
        }
        for i in 0..10 {
            let mid = 60.0 + i as f64 * 5.0;    // run-up
            bars.push(b(mid + 1.0, mid - 1.0, mid));
        }
        let cands = detect(&bars, &Config { lead_in_bars: 20, bump_height_multiplier: 1.5 });
        assert!(cands.iter().any(|c| c.kind == PatternKind::BarrBottom),
            "expected BARR-bottom, got {cands:?}");
    }

    #[test]
    fn no_bump_no_pattern() {
        // Lead-in uptrend with no acceleration → no bump.
        let bars: Vec<Bar> = (0..50).map(|i| {
            let mid = 100.0 + i as f64 * 0.5;
            b(mid + 0.5, mid - 0.5, mid)
        }).collect();
        let cands = detect(&bars, &Config::default());
        assert!(cands.iter().filter(|c| c.kind == PatternKind::BarrTop).count() == 0);
    }

    #[test]
    fn unconfirmed_run_yields_no_pattern() {
        // Lead-in + bump but no break-back → no candidate.
        let mut bars: Vec<Bar> = Vec::new();
        for i in 0..20 {
            let mid = 100.0 + i as f64 * 0.5;
            bars.push(b(mid + 0.5, mid - 0.5, mid));
        }
        for i in 0..10 {
            let mid = 110.0 + i as f64 * 5.0;    // keeps rising, no run
            bars.push(b(mid + 1.0, mid - 1.0, mid));
        }
        let cands = detect(&bars, &Config { lead_in_bars: 20, bump_height_multiplier: 1.5 });
        assert!(cands.iter().filter(|c| c.kind == PatternKind::BarrTop).count() == 0);
    }
}
