//! Social Security PIA — the Primary Insurance Amount from AIME.
//!
//! Your full-retirement-age benefit is computed from Average Indexed Monthly
//! Earnings (AIME) through a progressive, three-tier formula with two **bend
//! points**:
//!
//!   * 90% of AIME up to the first bend point,
//!   * 32% of AIME between the first and second bend points,
//!   * 15% of AIME above the second bend point.
//!
//! The 90/32/15 replacement rates are fixed by law; the bend-point dollar
//! thresholds are indexed annually (2026: $1,286 and $7,749, web-verified)
//! and locked in the year you turn 62. The declining rates make the benefit
//! progressive — lower earners get a higher replacement of their earnings.
//! Both bend points are inputs so the calc survives future indexing. Pure
//! compute.

use serde::{Deserialize, Serialize};

/// 2026 first bend point.
pub const DEFAULT_BEND_POINT_1_USD: f64 = 1_286.0;
/// 2026 second bend point.
pub const DEFAULT_BEND_POINT_2_USD: f64 = 7_749.0;

#[derive(Debug, Clone, Deserialize)]
pub struct PiaInput {
    /// Average Indexed Monthly Earnings.
    pub aime_usd: f64,
    /// First bend point (defaults to the current $1,286 if 0).
    #[serde(default)]
    pub bend_point_1_usd: f64,
    /// Second bend point (defaults to the current $7,749 if 0).
    #[serde(default)]
    pub bend_point_2_usd: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PiaResult {
    pub bend_point_1_usd: f64,
    pub bend_point_2_usd: f64,
    /// 90% tier (AIME up to BP1).
    pub tier1_usd: f64,
    /// 32% tier (AIME between BP1 and BP2).
    pub tier2_usd: f64,
    /// 15% tier (AIME above BP2).
    pub tier3_usd: f64,
    /// Monthly PIA (full-retirement-age benefit).
    pub pia_monthly_usd: f64,
    pub pia_annual_usd: f64,
    /// PIA as a percent of AIME — the replacement rate.
    pub replacement_rate_pct: f64,
}

pub fn analyze(i: &PiaInput) -> PiaResult {
    let aime = i.aime_usd.max(0.0);
    let bp1 = if i.bend_point_1_usd > 0.0 { i.bend_point_1_usd } else { DEFAULT_BEND_POINT_1_USD };
    let bp2 = if i.bend_point_2_usd > 0.0 { i.bend_point_2_usd } else { DEFAULT_BEND_POINT_2_USD };

    let tier1 = 0.90 * aime.min(bp1);
    let tier2 = 0.32 * (aime.min(bp2) - bp1).max(0.0);
    let tier3 = 0.15 * (aime - bp2).max(0.0);
    let pia = tier1 + tier2 + tier3;

    PiaResult {
        bend_point_1_usd: bp1,
        bend_point_2_usd: bp2,
        tier1_usd: tier1,
        tier2_usd: tier2,
        tier3_usd: tier3,
        pia_monthly_usd: pia,
        pia_annual_usd: pia * 12.0,
        replacement_rate_pct: if aime > 0.0 { pia / aime * 100.0 } else { 0.0 },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inp(aime: f64) -> PiaInput {
        PiaInput { aime_usd: aime, bend_point_1_usd: 0.0, bend_point_2_usd: 0.0 }
    }

    #[test]
    fn default_bend_points_are_2026() {
        let r = analyze(&inp(5_000.0));
        assert!((r.bend_point_1_usd - 1_286.0).abs() < 1e-9);
        assert!((r.bend_point_2_usd - 7_749.0).abs() < 1e-9);
    }

    #[test]
    fn low_earner_only_first_tier() {
        // AIME 1000 < BP1 → 90% × 1000 = 900, no tier2/3.
        let r = analyze(&inp(1_000.0));
        assert!((r.tier1_usd - 900.0).abs() < 1e-9);
        assert!(r.tier2_usd.abs() < 1e-9);
        assert!(r.tier3_usd.abs() < 1e-9);
        assert!((r.pia_monthly_usd - 900.0).abs() < 1e-9);
    }

    #[test]
    fn mid_earner_two_tiers() {
        // AIME 5000: 0.90×1286 + 0.32×(5000−1286) = 1157.4 + 1188.48 = 2345.88.
        let r = analyze(&inp(5_000.0));
        assert!((r.tier1_usd - 1_157.4).abs() < 1e-6);
        assert!((r.tier2_usd - 0.32 * (5_000.0 - 1_286.0)).abs() < 1e-6);
        assert!((r.pia_monthly_usd - 2_345.88).abs() < 1e-6);
    }

    #[test]
    fn high_earner_all_three_tiers() {
        // AIME 10000: 1157.4 + 0.32×6463 + 0.15×2251 = 1157.4 + 2068.16 + 337.65 = 3563.21.
        let r = analyze(&inp(10_000.0));
        assert!((r.tier3_usd - 0.15 * (10_000.0 - 7_749.0)).abs() < 1e-6);
        assert!((r.pia_monthly_usd - 3_563.21).abs() < 1e-6);
        assert!((r.pia_annual_usd - 3_563.21 * 12.0).abs() < 1e-4);
    }

    #[test]
    fn formula_is_progressive() {
        // Lower earner gets a higher replacement rate.
        let low = analyze(&inp(2_000.0));
        let high = analyze(&inp(10_000.0));
        assert!(low.replacement_rate_pct > high.replacement_rate_pct);
    }

    #[test]
    fn aime_at_first_bend_point() {
        let r = analyze(&inp(1_286.0));
        assert!((r.pia_monthly_usd - 0.90 * 1_286.0).abs() < 1e-9);
        assert!(r.tier2_usd.abs() < 1e-9);
    }

    #[test]
    fn zero_aime_zero_pia() {
        let r = analyze(&inp(0.0));
        assert!(r.pia_monthly_usd.abs() < 1e-9);
        assert!(r.replacement_rate_pct.abs() < 1e-9);
    }

    #[test]
    fn custom_bend_points_override() {
        let r = analyze(&PiaInput { aime_usd: 2_000.0, bend_point_1_usd: 1_000.0, bend_point_2_usd: 6_000.0 });
        // 0.90×1000 + 0.32×(2000−1000) = 900 + 320 = 1220.
        assert!((r.pia_monthly_usd - 1_220.0).abs() < 1e-9);
    }
}
