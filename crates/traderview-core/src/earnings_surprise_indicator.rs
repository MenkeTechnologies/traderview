//! Earnings Surprise Indicator — combines EPS surprise + revenue
//! surprise + estimate-revision trend.
//!
//!   eps_surprise_pct      = (actual_eps - consensus_eps) / |consensus_eps| · 100
//!   revenue_surprise_pct  = (actual_rev - consensus_rev) / |consensus_rev| · 100
//!   revision_trend        = SUM of last `n` analyst revisions
//!     (positive = consensus rising into report)
//!   composite_score       = 0.5·eps_surprise + 0.3·rev_surprise + 0.2·revision
//!
//! Classified into:
//!   StrongPositive : composite ≥ +10
//!   Positive       : +2 ≤ composite < +10
//!   Inline         : -2 < composite < +2
//!   Negative       : -10 < composite ≤ -2
//!   StrongNegative : composite ≤ -10
//!
//! Pure compute. Companion to `post_earnings_drift`, `earnings_revision_scanner`,
//! `earnings_calendar`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EarningsReport {
    pub actual_eps: f64,
    pub consensus_eps: f64,
    pub actual_revenue: f64,
    pub consensus_revenue: f64,
    pub recent_revision_count: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SurpriseClass {
    #[default]
    Inline,
    Positive,
    StrongPositive,
    Negative,
    StrongNegative,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct EarningsSurpriseReport {
    pub eps_surprise_pct: f64,
    pub revenue_surprise_pct: f64,
    pub revision_trend: f64,
    pub composite_score: f64,
    pub classification: SurpriseClass,
}

pub fn compute(report: EarningsReport) -> Option<EarningsSurpriseReport> {
    if !report.actual_eps.is_finite()
        || !report.consensus_eps.is_finite()
        || !report.actual_revenue.is_finite()
        || !report.consensus_revenue.is_finite()
        || !report.recent_revision_count.is_finite()
    {
        return None;
    }
    let eps_surprise = if report.consensus_eps.abs() > 0.0 {
        (report.actual_eps - report.consensus_eps) / report.consensus_eps.abs() * 100.0
    } else {
        0.0
    };
    let rev_surprise = if report.consensus_revenue.abs() > 0.0 {
        (report.actual_revenue - report.consensus_revenue) / report.consensus_revenue.abs() * 100.0
    } else {
        0.0
    };
    let revision = report.recent_revision_count;
    let composite = 0.5 * eps_surprise + 0.3 * rev_surprise + 0.2 * revision;
    let classification = classify(composite);
    Some(EarningsSurpriseReport {
        eps_surprise_pct: eps_surprise,
        revenue_surprise_pct: rev_surprise,
        revision_trend: revision,
        composite_score: composite,
        classification,
    })
}

fn classify(score: f64) -> SurpriseClass {
    if score >= 10.0 {
        SurpriseClass::StrongPositive
    } else if score >= 2.0 {
        SurpriseClass::Positive
    } else if score > -2.0 {
        SurpriseClass::Inline
    } else if score > -10.0 {
        SurpriseClass::Negative
    } else {
        SurpriseClass::StrongNegative
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(eps: f64, ceps: f64, rev: f64, crev: f64, rc: f64) -> EarningsReport {
        EarningsReport {
            actual_eps: eps,
            consensus_eps: ceps,
            actual_revenue: rev,
            consensus_revenue: crev,
            recent_revision_count: rc,
        }
    }

    #[test]
    fn invalid_returns_none() {
        assert!(compute(r(f64::NAN, 1.0, 100.0, 100.0, 0.0)).is_none());
    }

    #[test]
    fn classify_branches() {
        assert_eq!(classify(20.0), SurpriseClass::StrongPositive);
        assert_eq!(classify(5.0), SurpriseClass::Positive);
        assert_eq!(classify(0.0), SurpriseClass::Inline);
        assert_eq!(classify(-5.0), SurpriseClass::Negative);
        assert_eq!(classify(-20.0), SurpriseClass::StrongNegative);
    }

    #[test]
    fn strong_beat_classified_strong_positive() {
        // EPS beat 50%, revenue beat 10%, 5 positive revisions.
        let rep = compute(r(1.5, 1.0, 110.0, 100.0, 5.0)).unwrap();
        assert!((rep.eps_surprise_pct - 50.0).abs() < 1e-9);
        assert!((rep.revenue_surprise_pct - 10.0).abs() < 1e-9);
        assert_eq!(rep.classification, SurpriseClass::StrongPositive);
    }

    #[test]
    fn miss_classified_negative() {
        let rep = compute(r(0.5, 1.0, 90.0, 100.0, -2.0)).unwrap();
        assert!(matches!(
            rep.classification,
            SurpriseClass::Negative | SurpriseClass::StrongNegative
        ));
    }

    #[test]
    fn inline_results_classified_inline() {
        let rep = compute(r(1.01, 1.0, 100.5, 100.0, 0.0)).unwrap();
        assert_eq!(rep.classification, SurpriseClass::Inline);
    }

    #[test]
    fn zero_consensus_uses_safe_zero_surprise() {
        let rep = compute(r(1.0, 0.0, 100.0, 0.0, 0.0)).unwrap();
        assert!(rep.eps_surprise_pct.abs() < 1e-9);
        assert!(rep.revenue_surprise_pct.abs() < 1e-9);
    }
}
