//! Composite short-squeeze score (0–100) with weighted factor breakdown.
//!
//! Builds on top of `short_interest_scanner` by adding cost-to-borrow,
//! failure-to-deliver trend, gamma exposure, and options open-interest
//! concentration as additional squeeze pressure factors. Returns a
//! single 0–100 score plus the per-factor contributions so traders can
//! see *why* a stock scored high.
//!
//! ### Factors and default weights (sum to 1.0)
//!
//! | Factor                          | Weight | Range (raw → score)            |
//! |---------------------------------|--------|--------------------------------|
//! | short interest / float          | 0.30   | 0–60%+ → 0–100                 |
//! | days-to-cover                   | 0.20   | 0–10+ days → 0–100             |
//! | cost-to-borrow (APR)            | 0.20   | 0–100%+ → 0–100                |
//! | failure-to-deliver trend        | 0.10   | 0–5%+ of shares → 0–100        |
//! | recent price momentum (10-day)  | 0.10   | -10% to +30% → 0–100           |
//! | call/put OI ratio (gamma)       | 0.10   | 0.5 to 5.0+ → 0–100            |
//!
//! ### Grade buckets (mapped from final score)
//!
//! | Score   | Grade     |
//! |---------|-----------|
//! | 90–100  | Extreme   |
//! | 70–89   | High      |
//! | 50–69   | Moderate  |
//! | 30–49   | Low       |
//! | 0–29    | None      |
//!
//! Pure compute, no external data fetched. The caller supplies a
//! snapshot via `SqueezeFactors`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SqueezeFactors {
    /// Short shares / float, e.g. 0.30 = 30%.
    pub short_float_pct: f64,
    /// Short interest / 30-day average daily volume.
    pub days_to_cover: f64,
    /// Annualized cost-to-borrow rate from prime broker, e.g. 0.45 = 45% APR.
    pub cost_to_borrow_apr: f64,
    /// Failure-to-deliver shares as fraction of total outstanding, e.g.
    /// 0.005 = 0.5%. Use the most-recent biweekly SEC FTD release.
    pub ftd_pct_of_outstanding: f64,
    /// 10-day return, e.g. +0.15 = +15%. Negative when stock is falling.
    pub price_momentum_10d: f64,
    /// Call open-interest / put open-interest. ≥ 2 = bullish skew that
    /// can fuel gamma squeezes.
    pub call_put_oi_ratio: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ScoreWeights {
    pub short_float: f64,
    pub days_to_cover: f64,
    pub cost_to_borrow: f64,
    pub ftd: f64,
    pub momentum: f64,
    pub gamma: f64,
}

impl Default for ScoreWeights {
    fn default() -> Self {
        Self {
            short_float: 0.30,
            days_to_cover: 0.20,
            cost_to_borrow: 0.20,
            ftd: 0.10,
            momentum: 0.10,
            gamma: 0.10,
        }
    }
}

impl ScoreWeights {
    /// Sum of all weights (should be ≈ 1.0 for callers who care).
    pub fn sum(&self) -> f64 {
        self.short_float
            + self.days_to_cover
            + self.cost_to_borrow
            + self.ftd
            + self.momentum
            + self.gamma
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SqueezeGrade {
    Extreme,
    High,
    Moderate,
    Low,
    None,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct FactorScores {
    pub short_float: f64,
    pub days_to_cover: f64,
    pub cost_to_borrow: f64,
    pub ftd: f64,
    pub momentum: f64,
    pub gamma: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SqueezeScoreResult {
    /// Final weighted 0–100 score.
    pub score: f64,
    /// Per-factor contributions BEFORE applying weights (raw factor scores in 0–100).
    pub factor_scores: FactorScores,
    /// Per-factor contributions AFTER applying weights.
    pub weighted_contributions: FactorScores,
    /// Verbal grade derived from final score.
    pub grade: SqueezeGrade,
}

fn clamp(v: f64, lo: f64, hi: f64) -> f64 {
    if !v.is_finite() {
        return lo;
    }
    v.clamp(lo, hi)
}

/// Linear interpolation from `raw_lo..raw_hi` mapped to 0..100.
fn map_linear(raw: f64, raw_lo: f64, raw_hi: f64) -> f64 {
    if !raw.is_finite() {
        return 0.0;
    }
    if raw <= raw_lo {
        return 0.0;
    }
    if raw >= raw_hi {
        return 100.0;
    }
    (raw - raw_lo) / (raw_hi - raw_lo) * 100.0
}

fn grade(score: f64) -> SqueezeGrade {
    if score >= 90.0 {
        SqueezeGrade::Extreme
    } else if score >= 70.0 {
        SqueezeGrade::High
    } else if score >= 50.0 {
        SqueezeGrade::Moderate
    } else if score >= 30.0 {
        SqueezeGrade::Low
    } else {
        SqueezeGrade::None
    }
}

pub fn compute(factors: SqueezeFactors, weights: ScoreWeights) -> SqueezeScoreResult {
    // Per-factor 0–100 scores.
    let sf = map_linear(clamp(factors.short_float_pct, 0.0, 1.5), 0.0, 0.60);
    let dtc = map_linear(clamp(factors.days_to_cover, 0.0, 50.0), 0.0, 10.0);
    let ctb = map_linear(clamp(factors.cost_to_borrow_apr, 0.0, 5.0), 0.0, 1.0);
    let ftd = map_linear(clamp(factors.ftd_pct_of_outstanding, 0.0, 1.0), 0.0, 0.05);
    // Momentum spans -10% to +30% mapped to 0–100; negative = no squeeze pressure.
    let mom = map_linear(clamp(factors.price_momentum_10d, -0.5, 1.0), -0.10, 0.30);
    let gamma = map_linear(clamp(factors.call_put_oi_ratio, 0.0, 50.0), 0.5, 5.0);

    let factor_scores = FactorScores {
        short_float: sf,
        days_to_cover: dtc,
        cost_to_borrow: ctb,
        ftd,
        momentum: mom,
        gamma,
    };
    let weighted_contributions = FactorScores {
        short_float: sf * weights.short_float,
        days_to_cover: dtc * weights.days_to_cover,
        cost_to_borrow: ctb * weights.cost_to_borrow,
        ftd: ftd * weights.ftd,
        momentum: mom * weights.momentum,
        gamma: gamma * weights.gamma,
    };
    let score = weighted_contributions.short_float
        + weighted_contributions.days_to_cover
        + weighted_contributions.cost_to_borrow
        + weighted_contributions.ftd
        + weighted_contributions.momentum
        + weighted_contributions.gamma;
    let score = score.clamp(0.0, 100.0);

    SqueezeScoreResult {
        score,
        factor_scores,
        weighted_contributions,
        grade: grade(score),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn zero_factors() -> SqueezeFactors {
        SqueezeFactors {
            short_float_pct: 0.0,
            days_to_cover: 0.0,
            cost_to_borrow_apr: 0.0,
            ftd_pct_of_outstanding: 0.0,
            price_momentum_10d: 0.0,
            call_put_oi_ratio: 0.5,
        }
    }

    #[test]
    fn all_zero_factors_yields_zero_score_and_none_grade() {
        let r = compute(zero_factors(), ScoreWeights::default());
        // Momentum at 0% > -10% floor so it maps to ~25/100 × 0.10 = 2.5 score.
        // Total under 30 → None grade.
        assert!(r.score < 30.0);
        assert_eq!(r.grade, SqueezeGrade::None);
    }

    #[test]
    fn maxed_out_factors_yield_100_extreme_grade() {
        let r = compute(
            SqueezeFactors {
                short_float_pct: 0.60,
                days_to_cover: 10.0,
                cost_to_borrow_apr: 1.0,
                ftd_pct_of_outstanding: 0.05,
                price_momentum_10d: 0.30,
                call_put_oi_ratio: 5.0,
            },
            ScoreWeights::default(),
        );
        assert!(r.score >= 99.9, "expected near-100, got {}", r.score);
        assert_eq!(r.grade, SqueezeGrade::Extreme);
    }

    #[test]
    fn weights_sum_to_one_by_default() {
        let w = ScoreWeights::default();
        assert!((w.sum() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn gme_jan_2021_profile_scores_extreme() {
        // Approximate GME on 2021-01-22 (Friday before the Monday squeeze):
        // SI/float > 100%, DTC ~10, CTB ~30%, FTDs elevated, +50% momentum,
        // call/put OI ~ 3.
        let r = compute(
            SqueezeFactors {
                short_float_pct: 1.20,
                days_to_cover: 10.0,
                cost_to_borrow_apr: 0.30,
                ftd_pct_of_outstanding: 0.03,
                price_momentum_10d: 0.50,
                call_put_oi_ratio: 3.0,
            },
            ScoreWeights::default(),
        );
        assert!(
            r.score >= 70.0,
            "GME profile should be ≥ 70, got {}",
            r.score
        );
        assert!(matches!(
            r.grade,
            SqueezeGrade::High | SqueezeGrade::Extreme
        ));
    }

    #[test]
    fn high_short_alone_doesnt_hit_extreme_without_other_signals() {
        // 50% SI, nothing else exciting. SI factor = 100/100 × 0.30 = 30.
        // Momentum at 0 contributes ~25 × 0.10 = 2.5. Total ~32.5.
        let r = compute(
            SqueezeFactors {
                short_float_pct: 0.50,
                ..zero_factors()
            },
            ScoreWeights::default(),
        );
        assert!(r.score < 50.0);
    }

    #[test]
    fn nan_inputs_treated_as_zero() {
        let r = compute(
            SqueezeFactors {
                short_float_pct: f64::NAN,
                days_to_cover: f64::NAN,
                cost_to_borrow_apr: f64::NAN,
                ftd_pct_of_outstanding: f64::NAN,
                price_momentum_10d: f64::NAN,
                call_put_oi_ratio: f64::NAN,
            },
            ScoreWeights::default(),
        );
        // All factors collapse to 0 → score = 0.
        assert_eq!(r.score, 0.0);
    }

    #[test]
    fn negative_momentum_contributes_zero_not_negative() {
        // -20% momentum is below the floor → maps to 0.
        let r = compute(
            SqueezeFactors {
                price_momentum_10d: -0.20,
                ..zero_factors()
            },
            ScoreWeights::default(),
        );
        assert_eq!(r.factor_scores.momentum, 0.0);
    }

    #[test]
    fn momentum_at_top_caps_at_100() {
        let r = compute(
            SqueezeFactors {
                price_momentum_10d: 1.00, // way above 30%
                ..zero_factors()
            },
            ScoreWeights::default(),
        );
        assert_eq!(r.factor_scores.momentum, 100.0);
    }

    #[test]
    fn custom_weights_concentrate_signal() {
        // 100% weight on short_float; ignore everything else.
        let r = compute(
            SqueezeFactors {
                short_float_pct: 0.60,
                ..zero_factors()
            },
            ScoreWeights {
                short_float: 1.0,
                days_to_cover: 0.0,
                cost_to_borrow: 0.0,
                ftd: 0.0,
                momentum: 0.0,
                gamma: 0.0,
            },
        );
        assert!(r.score >= 99.0);
    }

    #[test]
    fn grade_thresholds_are_tight() {
        assert_eq!(grade(89.9), SqueezeGrade::High);
        assert_eq!(grade(90.0), SqueezeGrade::Extreme);
        assert_eq!(grade(69.9), SqueezeGrade::Moderate);
        assert_eq!(grade(70.0), SqueezeGrade::High);
        assert_eq!(grade(49.9), SqueezeGrade::Low);
        assert_eq!(grade(50.0), SqueezeGrade::Moderate);
        assert_eq!(grade(29.9), SqueezeGrade::None);
        assert_eq!(grade(30.0), SqueezeGrade::Low);
    }

    #[test]
    fn weighted_contributions_sum_to_final_score() {
        let r = compute(
            SqueezeFactors {
                short_float_pct: 0.30,
                days_to_cover: 5.0,
                cost_to_borrow_apr: 0.40,
                ftd_pct_of_outstanding: 0.02,
                price_momentum_10d: 0.15,
                call_put_oi_ratio: 2.0,
            },
            ScoreWeights::default(),
        );
        let sum = r.weighted_contributions.short_float
            + r.weighted_contributions.days_to_cover
            + r.weighted_contributions.cost_to_borrow
            + r.weighted_contributions.ftd
            + r.weighted_contributions.momentum
            + r.weighted_contributions.gamma;
        assert!((sum - r.score).abs() < 1e-9);
    }
}
