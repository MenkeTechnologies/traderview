//! Risk-on / Risk-off cross-asset regime signal.
//!
//! Reads a recent-day snapshot of:
//!   - SPY (equity proxy) direction
//!   - Gold (safe-haven)
//!   - DXY / dollar (safety, but inverse equity)
//!   - 10Y Treasury yield (risk-off → yields drop)
//!
//! Risk-ON: SPY up, gold down, dollar down, yields up.
//! Risk-OFF: SPY down, gold up, dollar up, yields down.
//! Mixed signals → Neutral.
//!
//! Pure compute. Scoring is heuristic — caller can override thresholds.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CrossAssetSnapshot {
    pub spy_change_pct: f64,
    pub gold_change_pct: f64,
    pub dxy_change_pct: f64,
    pub ten_year_yield_bps_change: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskRegime { On, Off, Neutral }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskReport {
    pub regime: RiskRegime,
    pub score: i32,
    pub agreement_count: i32,
    pub total_signals: i32,
}

pub fn evaluate(snap: &CrossAssetSnapshot) -> RiskReport {
    let mut score = 0i32;
    let mut agreement = 0i32;
    let mut total = 0i32;
    // SPY direction.
    total += 1;
    if snap.spy_change_pct > 0.001 { score += 1; agreement += 1; }
    else if snap.spy_change_pct < -0.001 { score -= 1; agreement += 1; }
    // Gold (inverse of risk-on).
    total += 1;
    if snap.gold_change_pct < -0.001 { score += 1; agreement += 1; }
    else if snap.gold_change_pct > 0.001 { score -= 1; agreement += 1; }
    // Dollar (inverse of risk-on).
    total += 1;
    if snap.dxy_change_pct < -0.001 { score += 1; agreement += 1; }
    else if snap.dxy_change_pct > 0.001 { score -= 1; agreement += 1; }
    // Yields (positive correlation with risk-on).
    total += 1;
    if snap.ten_year_yield_bps_change > 1.0 { score += 1; agreement += 1; }
    else if snap.ten_year_yield_bps_change < -1.0 { score -= 1; agreement += 1; }
    let regime = if score >= 2 { RiskRegime::On }
        else if score <= -2 { RiskRegime::Off }
        else { RiskRegime::Neutral };
    RiskReport {
        regime,
        score,
        agreement_count: agreement,
        total_signals: total,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snap(spy: f64, gold: f64, dxy: f64, yld: f64) -> CrossAssetSnapshot {
        CrossAssetSnapshot {
            spy_change_pct: spy,
            gold_change_pct: gold,
            dxy_change_pct: dxy,
            ten_year_yield_bps_change: yld,
        }
    }

    #[test]
    fn full_risk_on_score_plus_4() {
        let r = evaluate(&snap(0.01, -0.005, -0.003, 5.0));
        assert_eq!(r.regime, RiskRegime::On);
        assert_eq!(r.score, 4);
    }

    #[test]
    fn full_risk_off_score_minus_4() {
        let r = evaluate(&snap(-0.02, 0.01, 0.005, -8.0));
        assert_eq!(r.regime, RiskRegime::Off);
        assert_eq!(r.score, -4);
    }

    #[test]
    fn mixed_signals_neutral() {
        let r = evaluate(&snap(0.01, 0.005, -0.001, -2.0));
        // SPY +1, gold -1, dxy +1, yields -1. Score = 0.
        assert_eq!(r.regime, RiskRegime::Neutral);
    }

    #[test]
    fn flat_snapshot_neutral() {
        let r = evaluate(&snap(0.0, 0.0, 0.0, 0.0));
        assert_eq!(r.regime, RiskRegime::Neutral);
        assert_eq!(r.score, 0);
    }

    #[test]
    fn minority_risk_on_signals_still_neutral() {
        // 1 on signal, 0 off → score +1 → neutral (need ≥ +2).
        let r = evaluate(&snap(0.01, 0.0, 0.0, 0.0));
        assert_eq!(r.regime, RiskRegime::Neutral);
        assert_eq!(r.score, 1);
    }

    #[test]
    fn majority_risk_off_classifies_off() {
        let r = evaluate(&snap(-0.01, 0.005, 0.003, 0.0));
        // SPY -1, gold -1, dxy -1, yields 0 → score -3 → Off.
        assert_eq!(r.regime, RiskRegime::Off);
    }

    #[test]
    fn agreement_count_excludes_noisy_signals() {
        // Tiny SPY change (below noise floor) → not counted.
        let r = evaluate(&snap(0.0001, -0.01, -0.005, 5.0));
        // SPY too small → 0 sig; gold +1, dxy +1, yields +1 → 3.
        assert_eq!(r.agreement_count, 3);
    }

    #[test]
    fn yield_threshold_uses_bps_not_pct() {
        // Yields up 0.5bps — below 1bp threshold → no signal.
        let r = evaluate(&snap(0.0, 0.0, 0.0, 0.5));
        assert_eq!(r.score, 0);
    }
}
