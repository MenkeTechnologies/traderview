//! Macro policy / recession gauges:
//!
//! * Taylor rule (1993) — prescribed policy rate
//!   i = r* + π + 0.5·(π − π*) + 0.5·gap
//!   compared against the actual rate for a tight/loose verdict.
//! * Sahm rule (2019) — recession signal when the 3-month average
//!   unemployment rate rises ≥ 0.50pp above its minimum over the
//!   prior 12 months.
//! * Misery index (Okun) — inflation + unemployment.
//!
//! Pure compute on caller-supplied macro readings. Companion to
//! `yield_curve` (term-spread recession work lives there).

use serde::{Deserialize, Serialize};

// ── Taylor rule ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct TaylorInput {
    /// Neutral real rate r*, % (Taylor's original: 2.0).
    pub neutral_real_rate: f64,
    /// Current inflation π, % (core PCE in Fed practice).
    pub inflation: f64,
    /// Inflation target π*, % (2.0 for the Fed).
    pub inflation_target: f64,
    /// Output gap, % of potential GDP (positive = running hot).
    pub output_gap: f64,
    /// Actual policy rate, % — for the stance verdict.
    pub actual_rate: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct TaylorReport {
    pub prescribed_rate: f64,
    pub actual_rate: f64,
    /// actual − prescribed; positive = policy tighter than the rule.
    pub gap: f64,
    /// "tight", "loose", or "neutral" (within ±0.25pp).
    pub stance: &'static str,
}

pub fn taylor_rule(inp: &TaylorInput) -> Option<TaylorReport> {
    if ![
        inp.neutral_real_rate,
        inp.inflation,
        inp.inflation_target,
        inp.output_gap,
        inp.actual_rate,
    ]
    .iter()
    .all(|v| v.is_finite())
    {
        return None;
    }
    let prescribed = inp.neutral_real_rate
        + inp.inflation
        + 0.5 * (inp.inflation - inp.inflation_target)
        + 0.5 * inp.output_gap;
    let gap = inp.actual_rate - prescribed;
    let stance = if gap > 0.25 {
        "tight"
    } else if gap < -0.25 {
        "loose"
    } else {
        "neutral"
    };
    Some(TaylorReport {
        prescribed_rate: prescribed,
        actual_rate: inp.actual_rate,
        gap,
        stance,
    })
}

// ── Sahm rule ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct SahmReport {
    /// Current 3-month average unemployment rate, %.
    pub current_3mo_avg: f64,
    /// Minimum 3-month average over the prior 12 months, %.
    pub min_prior_12mo: f64,
    /// current − min (the Sahm indicator), pp.
    pub sahm_value: f64,
    /// True when sahm_value ≥ 0.50pp.
    pub triggered: bool,
}

/// `monthly_unemployment` oldest→newest, %; needs ≥ 15 months (3 for
/// the current average + 12 prior averages to scan).
pub fn sahm_rule(monthly_unemployment: &[f64]) -> Option<SahmReport> {
    let u = monthly_unemployment;
    if u.len() < 15 || u.iter().any(|x| !x.is_finite() || *x < 0.0 || *x > 100.0) {
        return None;
    }
    let avg3 = |end: usize| (u[end - 2] + u[end - 1] + u[end]) / 3.0;
    let last = u.len() - 1;
    let current = avg3(last);
    let min_prior = (last - 12..last)
        .map(avg3)
        .fold(f64::INFINITY, f64::min);
    let value = current - min_prior;
    Some(SahmReport {
        current_3mo_avg: current,
        min_prior_12mo: min_prior,
        sahm_value: value,
        triggered: value >= 0.50,
    })
}

// ── Misery index ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct MiseryReport {
    pub inflation: f64,
    pub unemployment: f64,
    pub misery_index: f64,
}

pub fn misery_index(inflation: f64, unemployment: f64) -> Option<MiseryReport> {
    if !inflation.is_finite()
        || !unemployment.is_finite()
        || !(0.0..=100.0).contains(&unemployment)
    {
        return None;
    }
    Some(MiseryReport {
        inflation,
        unemployment,
        misery_index: inflation + unemployment,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn taylor_matches_hand_computed_prescription() {
        // r*=2, π=4, π*=2, gap=1: i = 2 + 4 + 0.5·2 + 0.5·1 = 7.5.
        let r = taylor_rule(&TaylorInput {
            neutral_real_rate: 2.0,
            inflation: 4.0,
            inflation_target: 2.0,
            output_gap: 1.0,
            actual_rate: 5.0,
        })
        .unwrap();
        assert!((r.prescribed_rate - 7.5).abs() < 1e-12);
        assert!((r.gap + 2.5).abs() < 1e-12);
        assert_eq!(r.stance, "loose");
    }

    #[test]
    fn taylor_balanced_economy_prescribes_neutral_nominal() {
        // π = π*, zero gap ⇒ i = r* + π*.
        let r = taylor_rule(&TaylorInput {
            neutral_real_rate: 2.0,
            inflation: 2.0,
            inflation_target: 2.0,
            output_gap: 0.0,
            actual_rate: 4.0,
        })
        .unwrap();
        assert!((r.prescribed_rate - 4.0).abs() < 1e-12);
        assert_eq!(r.stance, "neutral");
    }

    #[test]
    fn sahm_triggers_on_a_60bp_rise() {
        // 12 flat months at 3.5%, then three months at 4.1%: current
        // 3mo avg = 4.1, min prior = 3.5 ⇒ value 0.6 ⇒ triggered.
        let mut u = vec![3.5_f64; 12];
        u.extend([4.1, 4.1, 4.1]);
        let r = sahm_rule(&u).unwrap();
        assert!((r.sahm_value - 0.6).abs() < 1e-9, "{r:?}");
        assert!(r.triggered);
    }

    #[test]
    fn sahm_stays_quiet_on_flat_unemployment() {
        let r = sahm_rule(&[3.7; 24]).unwrap();
        assert!(r.sahm_value.abs() < 1e-12);
        assert!(!r.triggered);
    }

    #[test]
    fn sahm_just_below_threshold_does_not_trigger() {
        let mut u = vec![4.0_f64; 12];
        u.extend([4.49, 4.49, 4.49]);
        let r = sahm_rule(&u).unwrap();
        assert!(!r.triggered, "{r:?}");
    }

    #[test]
    fn misery_is_the_plain_sum() {
        let r = misery_index(3.2, 4.1).unwrap();
        assert!((r.misery_index - 7.3).abs() < 1e-12);
    }

    #[test]
    fn hostile_inputs_return_none() {
        assert!(sahm_rule(&[4.0; 10]).is_none()); // too short
        assert!(sahm_rule(&[f64::NAN; 20]).is_none());
        assert!(misery_index(f64::NAN, 4.0).is_none());
        assert!(misery_index(2.0, 150.0).is_none());
        assert!(taylor_rule(&TaylorInput {
            neutral_real_rate: f64::NAN,
            inflation: 2.0,
            inflation_target: 2.0,
            output_gap: 0.0,
            actual_rate: 4.0,
        })
        .is_none());
    }
}
