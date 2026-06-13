//! Dividend coverage & payout ratio — is the dividend sustainable?
//!
//! How much of a company's earnings go out as dividends, and how comfortably
//! earnings cover them:
//!
//!   * **payout ratio** = dividends per share / EPS — the share of earnings
//!     paid out. Over 100% means the dividend exceeds earnings (funded from
//!     cash or debt — a red flag).
//!   * **coverage ratio** = EPS / DPS — how many times earnings cover the
//!     dividend (the inverse of payout).
//!   * **retention ratio** = 100% − payout — earnings plowed back for growth.
//!   * Optional **FCF payout** = DPS / free cash flow per share — a stricter
//!     test, since dividends are paid from cash, not accounting earnings.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct DividendInput {
    pub eps_usd: f64,
    pub dps_usd: f64,
    /// Free cash flow per share (0 ⇒ skip the FCF payout).
    #[serde(default)]
    pub fcf_per_share_usd: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DividendResult {
    pub payout_ratio_pct: f64,
    /// EPS / DPS — times earnings cover the dividend (None if no dividend).
    pub coverage_ratio: Option<f64>,
    pub retention_ratio_pct: f64,
    /// DPS / FCF per share (None if FCF not provided).
    pub fcf_payout_pct: Option<f64>,
    /// True when earnings cover the dividend (payout ≤ 100%).
    pub sustainable: bool,
    /// "healthy" (<60%), "moderate" (60–90%), "stretched" (90–100%),
    /// or "unsustainable" (>100%).
    pub rating: String,
}

pub fn analyze(i: &DividendInput) -> DividendResult {
    let payout = if i.eps_usd > 0.0 {
        i.dps_usd / i.eps_usd * 100.0
    } else if i.dps_usd > 0.0 {
        // Paying a dividend with zero/negative earnings → beyond unsustainable.
        f64::INFINITY
    } else {
        0.0
    };

    let coverage = if i.dps_usd > 0.0 { Some(i.eps_usd / i.dps_usd) } else { None };
    let retention = 100.0 - payout;
    let fcf_payout = if i.fcf_per_share_usd > 0.0 {
        Some(i.dps_usd / i.fcf_per_share_usd * 100.0)
    } else {
        None
    };

    let sustainable = payout <= 100.0 && i.eps_usd > 0.0 || i.dps_usd <= 0.0;
    let rating = if i.dps_usd <= 0.0 {
        "no dividend"
    } else if !payout.is_finite() || payout > 100.0 {
        "unsustainable"
    } else if payout < 60.0 {
        "healthy"
    } else if payout <= 90.0 {
        "moderate"
    } else {
        "stretched"
    };

    DividendResult {
        // Report a finite, capped figure for display when payout is infinite.
        payout_ratio_pct: if payout.is_finite() { payout } else { 0.0 },
        coverage_ratio: coverage,
        retention_ratio_pct: if retention.is_finite() { retention } else { 0.0 },
        fcf_payout_pct: fcf_payout,
        sustainable,
        rating: rating.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inp(eps: f64, dps: f64) -> DividendInput {
        DividendInput { eps_usd: eps, dps_usd: dps, fcf_per_share_usd: 0.0 }
    }

    #[test]
    fn payout_ratio() {
        // DPS 2 / EPS 5 = 40%.
        let r = analyze(&inp(5.0, 2.0));
        assert!((r.payout_ratio_pct - 40.0).abs() < 1e-9);
    }

    #[test]
    fn coverage_is_inverse() {
        // EPS 5 / DPS 2 = 2.5x.
        let r = analyze(&inp(5.0, 2.0));
        assert!((r.coverage_ratio.unwrap() - 2.5).abs() < 1e-9);
    }

    #[test]
    fn retention_is_complement() {
        let r = analyze(&inp(5.0, 2.0));
        assert!((r.retention_ratio_pct - 60.0).abs() < 1e-9);
    }

    #[test]
    fn healthy_rating_and_sustainable() {
        let r = analyze(&inp(5.0, 2.0)); // 40% payout
        assert_eq!(r.rating, "healthy");
        assert!(r.sustainable);
    }

    #[test]
    fn dividend_above_earnings_unsustainable() {
        // DPS 6 > EPS 5 → 120% payout.
        let r = analyze(&inp(5.0, 6.0));
        assert!((r.payout_ratio_pct - 120.0).abs() < 1e-9);
        assert_eq!(r.rating, "unsustainable");
        assert!(!r.sustainable);
    }

    #[test]
    fn fcf_payout_when_provided() {
        // DPS 2 / FCF 8 = 25%.
        let r = analyze(&DividendInput { eps_usd: 5.0, dps_usd: 2.0, fcf_per_share_usd: 8.0 });
        assert!((r.fcf_payout_pct.unwrap() - 25.0).abs() < 1e-9);
    }

    #[test]
    fn zero_earnings_with_dividend_is_unsustainable() {
        let r = analyze(&inp(0.0, 2.0));
        assert_eq!(r.rating, "unsustainable");
        assert!(!r.sustainable);
    }

    #[test]
    fn rating_tiers() {
        assert_eq!(analyze(&inp(10.0, 7.0)).rating, "moderate"); // 70%
        assert_eq!(analyze(&inp(10.0, 9.5)).rating, "stretched"); // 95%
        assert_eq!(analyze(&inp(10.0, 0.0)).rating, "no dividend");
    }
}
