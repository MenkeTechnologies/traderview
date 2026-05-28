//! Head-and-Shoulders / Inverse Head-and-Shoulders pattern detector.
//!
//! Pattern (classic top):
//!   - Three pivot highs: left shoulder, head (higher than both shoulders),
//!     right shoulder. Shoulders within `shoulder_symmetry_pct` of each other.
//!   - Two pivot lows between them ("neckline anchors"). The neckline
//!     is the line through those two lows.
//!   - Confirmed when price closes below the neckline (Top) or above it
//!     (Inverse). Pattern projection: target = neckline − (head − neckline).
//!
//! Inverse pattern is the bullish bottom: three pivot lows (middle is
//! lowest), two pivot highs as neckline anchors.
//!
//! Caller supplies the bar series; detector uses `pivot_lookback` for
//! left/right pivot tests (Williams fractal-style). Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub pivot_lookback: usize,
    pub shoulder_symmetry_pct: f64,
    pub max_pattern_bars: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self { pivot_lookback: 5, shoulder_symmetry_pct: 0.05, max_pattern_bars: 200 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PatternKind { HeadShoulders, InverseHeadShoulders }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HsCandidate {
    pub kind: PatternKind,
    pub left_shoulder_index: usize,
    pub head_index: usize,
    pub right_shoulder_index: usize,
    pub neckline_left_index: usize,
    pub neckline_right_index: usize,
    pub neckline_at_right_shoulder: f64,
    pub projection_target: f64,
}

pub fn detect(bars: &[Bar], cfg: &Config) -> Vec<HsCandidate> {
    let n = bars.len();
    let mut out = Vec::new();
    if n < 7 + 2 * cfg.pivot_lookback
        || cfg.pivot_lookback == 0
        || cfg.shoulder_symmetry_pct < 0.0
        || cfg.max_pattern_bars == 0
    {
        return out;
    }
    let highs = find_pivots(bars, cfg.pivot_lookback, true);
    let lows = find_pivots(bars, cfg.pivot_lookback, false);
    // Top pattern: 3 high pivots + 2 low pivots between them.
    for trio in highs.windows(3) {
        let (ls, hd, rs) = (trio[0], trio[1], trio[2]);
        if rs - ls > cfg.max_pattern_bars { continue; }
        let ls_h = bars[ls].high;
        let hd_h = bars[hd].high;
        let rs_h = bars[rs].high;
        if !ls_h.is_finite() || !hd_h.is_finite() || !rs_h.is_finite() { continue; }
        if hd_h <= ls_h || hd_h <= rs_h { continue; }
        let sym = (ls_h - rs_h).abs() / ls_h.max(rs_h);
        if sym > cfg.shoulder_symmetry_pct { continue; }
        // Find one neckline-low between (ls, hd) and (hd, rs).
        let n_left = lows.iter().copied().find(|i| *i > ls && *i < hd);
        let n_right = lows.iter().copied().find(|i| *i > hd && *i < rs);
        let (Some(nl), Some(nr)) = (n_left, n_right) else { continue };
        let neckline_at_right_shoulder = interp(nl, bars[nl].low, nr, bars[nr].low, rs);
        let projection_target = neckline_at_right_shoulder - (hd_h - neckline_at_right_shoulder);
        out.push(HsCandidate {
            kind: PatternKind::HeadShoulders,
            left_shoulder_index: ls,
            head_index: hd,
            right_shoulder_index: rs,
            neckline_left_index: nl,
            neckline_right_index: nr,
            neckline_at_right_shoulder,
            projection_target,
        });
    }
    // Inverse: 3 low pivots + 2 high pivots between.
    for trio in lows.windows(3) {
        let (ls, hd, rs) = (trio[0], trio[1], trio[2]);
        if rs - ls > cfg.max_pattern_bars { continue; }
        let ls_l = bars[ls].low;
        let hd_l = bars[hd].low;
        let rs_l = bars[rs].low;
        if !ls_l.is_finite() || !hd_l.is_finite() || !rs_l.is_finite() { continue; }
        if hd_l >= ls_l || hd_l >= rs_l { continue; }
        let sym = (ls_l - rs_l).abs() / ls_l.max(rs_l);
        if sym > cfg.shoulder_symmetry_pct { continue; }
        let n_left = highs.iter().copied().find(|i| *i > ls && *i < hd);
        let n_right = highs.iter().copied().find(|i| *i > hd && *i < rs);
        let (Some(nl), Some(nr)) = (n_left, n_right) else { continue };
        let neckline_at_right_shoulder = interp(nl, bars[nl].high, nr, bars[nr].high, rs);
        let projection_target = neckline_at_right_shoulder + (neckline_at_right_shoulder - hd_l);
        out.push(HsCandidate {
            kind: PatternKind::InverseHeadShoulders,
            left_shoulder_index: ls,
            head_index: hd,
            right_shoulder_index: rs,
            neckline_left_index: nl,
            neckline_right_index: nr,
            neckline_at_right_shoulder,
            projection_target,
        });
    }
    out
}

fn find_pivots(bars: &[Bar], lookback: usize, find_high: bool) -> Vec<usize> {
    let n = bars.len();
    let mut out = Vec::new();
    if n < 2 * lookback + 1 { return out; }
    for i in lookback..(n - lookback) {
        let v = if find_high { bars[i].high } else { bars[i].low };
        if !v.is_finite() { continue; }
        let mut is_pivot = true;
        for k in 1..=lookback {
            let l = if find_high { bars[i - k].high } else { bars[i - k].low };
            let r = if find_high { bars[i + k].high } else { bars[i + k].low };
            if !l.is_finite() || !r.is_finite() { is_pivot = false; break; }
            if find_high {
                if l >= v || r >= v { is_pivot = false; break; }
            } else if l <= v || r <= v { is_pivot = false; break; }
        }
        if is_pivot { out.push(i); }
    }
    out
}

fn interp(x0: usize, y0: f64, x1: usize, y1: f64, x: usize) -> f64 {
    if x1 == x0 { return y0; }
    let t = (x as f64 - x0 as f64) / (x1 as f64 - x0 as f64);
    y0 + t * (y1 - y0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64) -> Bar {
        Bar { high: h, low: l, close: (h + l) / 2.0 }
    }

    #[test]
    fn empty_returns_empty() {
        assert!(detect(&[], &Config::default()).is_empty());
    }

    #[test]
    fn invalid_config_returns_empty() {
        let bars = vec![b(101.0, 99.0); 100];
        let cfg = Config { pivot_lookback: 0, ..Default::default() };
        assert!(detect(&bars, &cfg).is_empty());
    }

    #[test]
    fn flat_series_yields_no_pattern() {
        let bars = vec![b(101.0, 99.0); 100];
        assert!(detect(&bars, &Config::default()).is_empty());
    }

    #[test]
    fn classic_top_pattern_detected() {
        // Build: 30 bars, with peaks at 8 (105), 15 (110), 22 (105),
        // and troughs at 11 (95), 18 (96).
        let mut bars = vec![b(100.5, 99.5); 30];
        bars[8] = b(105.0, 104.5);
        bars[11] = b(95.5, 95.0);
        bars[15] = b(110.0, 109.5);
        bars[18] = b(96.5, 96.0);
        bars[22] = b(105.0, 104.5);
        let cfg = Config { pivot_lookback: 2, ..Default::default() };
        let cands = detect(&bars, &cfg);
        assert!(cands.iter().any(|c| c.kind == PatternKind::HeadShoulders),
            "should detect HS top, got {cands:?}");
    }

    #[test]
    fn inverse_bottom_pattern_detected() {
        // Mirror: troughs at 8 (95), 15 (90), 22 (95) + neckline highs at 11, 18.
        let mut bars = vec![b(100.5, 99.5); 30];
        bars[8] = b(96.0, 95.0);
        bars[11] = b(104.5, 104.0);
        bars[15] = b(91.0, 90.0);
        bars[18] = b(104.0, 103.5);
        bars[22] = b(96.0, 95.0);
        let cfg = Config { pivot_lookback: 2, ..Default::default() };
        let cands = detect(&bars, &cfg);
        assert!(cands.iter().any(|c| c.kind == PatternKind::InverseHeadShoulders),
            "should detect inverse HS bottom, got {cands:?}");
    }

    #[test]
    fn shoulder_asymmetry_rejects() {
        // Left shoulder 105, right shoulder 95 — 9.5% asymmetric.
        let mut bars = vec![b(100.5, 99.5); 30];
        bars[8] = b(105.0, 104.5);
        bars[11] = b(95.5, 95.0);
        bars[15] = b(110.0, 109.5);
        bars[18] = b(96.5, 96.0);
        bars[22] = b(95.0, 94.5);
        let cfg = Config { pivot_lookback: 2, shoulder_symmetry_pct: 0.05, ..Default::default() };
        let cands: Vec<_> = detect(&bars, &cfg).into_iter()
            .filter(|c| c.kind == PatternKind::HeadShoulders).collect();
        assert!(cands.is_empty(), "asymmetric shoulders should reject HS");
    }

    #[test]
    fn projection_target_below_neckline_for_top() {
        let mut bars = vec![b(100.5, 99.5); 30];
        bars[8] = b(105.0, 104.5);
        bars[11] = b(95.5, 95.0);
        bars[15] = b(110.0, 109.5);
        bars[18] = b(96.5, 96.0);
        bars[22] = b(105.0, 104.5);
        let cfg = Config { pivot_lookback: 2, ..Default::default() };
        let cand = detect(&bars, &cfg).into_iter()
            .find(|c| c.kind == PatternKind::HeadShoulders).expect("HS");
        assert!(cand.projection_target < cand.neckline_at_right_shoulder);
    }
}
