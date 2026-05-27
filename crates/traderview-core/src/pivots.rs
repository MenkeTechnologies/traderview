//! Pivot point calculator — Floor / Camarilla / Woodie / DeMark.
//!
//! All four take the prior session's High/Low/Close (and Open for some)
//! and emit support/resistance levels traders watch for intraday entries.
//!
//! Floor (Standard):
//!   P  = (H + L + C) / 3
//!   R1 = 2P - L,     S1 = 2P - H
//!   R2 = P + (H-L),  S2 = P - (H-L)
//!   R3 = H + 2(P-L), S3 = L - 2(H-P)
//!
//! Camarilla (high-frequency intraday):
//!   R1 = C + (H-L) × 1.1/12,  S1 = C - (H-L) × 1.1/12
//!   R2 = C + (H-L) × 1.1/6,   S2 = C - (H-L) × 1.1/6
//!   R3 = C + (H-L) × 1.1/4,   S3 = C - (H-L) × 1.1/4
//!   R4 = C + (H-L) × 1.1/2,   S4 = C - (H-L) × 1.1/2
//!
//! Woodie (weights close more heavily):
//!   P  = (H + L + 2C) / 4
//!   R1 = 2P - L,     S1 = 2P - H
//!   R2 = P + (H-L),  S2 = P - (H-L)
//!
//! DeMark (uses open):
//!   X = depends on close vs open vs midpoint relationship
//!   newH = X/2 - L
//!   newL = X/2 - H
//!   (No "P" — only one pivot pair)
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PriorBar {
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub open: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FloorPivots {
    pub pivot: f64,
    pub r1: f64, pub s1: f64,
    pub r2: f64, pub s2: f64,
    pub r3: f64, pub s3: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CamarillaPivots {
    pub r1: f64, pub s1: f64,
    pub r2: f64, pub s2: f64,
    pub r3: f64, pub s3: f64,
    pub r4: f64, pub s4: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WoodiePivots {
    pub pivot: f64,
    pub r1: f64, pub s1: f64,
    pub r2: f64, pub s2: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DemarkPivots {
    pub new_high: f64,
    pub new_low: f64,
}

pub fn floor(b: PriorBar) -> FloorPivots {
    let p = (b.high + b.low + b.close) / 3.0;
    let range = b.high - b.low;
    FloorPivots {
        pivot: p,
        r1: 2.0 * p - b.low,
        s1: 2.0 * p - b.high,
        r2: p + range,
        s2: p - range,
        r3: b.high + 2.0 * (p - b.low),
        s3: b.low - 2.0 * (b.high - p),
    }
}

pub fn camarilla(b: PriorBar) -> CamarillaPivots {
    let range = b.high - b.low;
    CamarillaPivots {
        r1: b.close + range * 1.1 / 12.0,
        s1: b.close - range * 1.1 / 12.0,
        r2: b.close + range * 1.1 / 6.0,
        s2: b.close - range * 1.1 / 6.0,
        r3: b.close + range * 1.1 / 4.0,
        s3: b.close - range * 1.1 / 4.0,
        r4: b.close + range * 1.1 / 2.0,
        s4: b.close - range * 1.1 / 2.0,
    }
}

pub fn woodie(b: PriorBar) -> WoodiePivots {
    let p = (b.high + b.low + 2.0 * b.close) / 4.0;
    let range = b.high - b.low;
    WoodiePivots {
        pivot: p,
        r1: 2.0 * p - b.low,
        s1: 2.0 * p - b.high,
        r2: p + range,
        s2: p - range,
    }
}

pub fn demark(b: PriorBar) -> DemarkPivots {
    // X selection per DeMark:
    //   close < open  → X = H + 2L + C
    //   close > open  → X = 2H + L + C
    //   close == open → X = H + L + 2C
    let x = if b.close < b.open {
        b.high + 2.0 * b.low + b.close
    } else if b.close > b.open {
        2.0 * b.high + b.low + b.close
    } else {
        b.high + b.low + 2.0 * b.close
    };
    DemarkPivots {
        new_high: x / 2.0 - b.low,
        new_low:  x / 2.0 - b.high,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64, c: f64, o: f64) -> PriorBar {
        PriorBar { high: h, low: l, close: c, open: o }
    }

    // ─── floor ────────────────────────────────────────────────────────

    #[test]
    fn floor_pivot_is_hlc_average() {
        let p = floor(b(110.0, 100.0, 105.0, 102.0));
        assert!((p.pivot - 105.0).abs() < 1e-12);
    }

    #[test]
    fn floor_r1_above_pivot_s1_below() {
        let p = floor(b(110.0, 100.0, 105.0, 102.0));
        assert!(p.r1 > p.pivot, "R1 above pivot");
        assert!(p.s1 < p.pivot, "S1 below pivot");
    }

    #[test]
    fn floor_r3_above_r2_above_r1() {
        let p = floor(b(110.0, 100.0, 105.0, 102.0));
        assert!(p.r1 < p.r2 && p.r2 < p.r3, "resistance levels stacked");
        assert!(p.s1 > p.s2 && p.s2 > p.s3, "support levels stacked");
    }

    // ─── camarilla ────────────────────────────────────────────────────

    #[test]
    fn camarilla_levels_stacked_correctly() {
        let p = camarilla(b(110.0, 100.0, 105.0, 102.0));
        assert!(p.r1 < p.r2 && p.r2 < p.r3 && p.r3 < p.r4);
        assert!(p.s1 > p.s2 && p.s2 > p.s3 && p.s3 > p.s4);
    }

    #[test]
    fn camarilla_centered_on_close() {
        let p = camarilla(b(110.0, 100.0, 105.0, 102.0));
        // Levels are SYMMETRIC around close.
        assert!((((p.r1 + p.s1) / 2.0) - 105.0).abs() < 1e-12);
        assert!((((p.r4 + p.s4) / 2.0) - 105.0).abs() < 1e-12);
    }

    // ─── woodie ───────────────────────────────────────────────────────

    #[test]
    fn woodie_pivot_weights_close_double() {
        // (110 + 100 + 2×105) / 4 = 420 / 4 = 105.
        let p = woodie(b(110.0, 100.0, 105.0, 102.0));
        assert!((p.pivot - 105.0).abs() < 1e-12);
    }

    #[test]
    fn woodie_differs_from_floor_when_close_off_midpoint() {
        // Close = 108 (above midpoint of 105) → woodie pivot pulled higher.
        let f = floor(b(110.0, 100.0, 108.0, 102.0));
        let w = woodie(b(110.0, 100.0, 108.0, 102.0));
        assert!(w.pivot > f.pivot,
            "Woodie weights close more → pivot above floor pivot when close > midpoint");
    }

    // ─── demark ───────────────────────────────────────────────────────

    #[test]
    fn demark_uses_close_below_open_formula() {
        // close < open → X = H + 2L + C = 110 + 200 + 100 = 410.
        // new_high = 205 - 100 = 105. new_low = 205 - 110 = 95.
        let p = demark(b(110.0, 100.0, 100.0, 105.0));
        assert!((p.new_high - 105.0).abs() < 1e-12);
        assert!((p.new_low  -  95.0).abs() < 1e-12);
    }

    #[test]
    fn demark_uses_close_above_open_formula() {
        // close > open → X = 2H + L + C = 220 + 100 + 108 = 428.
        // new_high = 214 - 100 = 114. new_low = 214 - 110 = 104.
        let p = demark(b(110.0, 100.0, 108.0, 102.0));
        assert!((p.new_high - 114.0).abs() < 1e-12);
        assert!((p.new_low  - 104.0).abs() < 1e-12);
    }

    #[test]
    fn demark_uses_close_equals_open_formula() {
        // close == open → X = H + L + 2C = 110 + 100 + 200 = 410.
        let p = demark(b(110.0, 100.0, 100.0, 100.0));
        assert!((p.new_high - 105.0).abs() < 1e-12);
        assert!((p.new_low  -  95.0).abs() < 1e-12);
    }

    #[test]
    fn demark_new_high_above_new_low() {
        // Always: new_high > new_low for non-degenerate bars.
        let p = demark(b(110.0, 100.0, 108.0, 102.0));
        assert!(p.new_high > p.new_low);
    }
}
