//! Generic trading-alert rule engine.
//!
//! Evaluates user-defined rules against a `MarketSnapshot` for a single
//! symbol and returns the set of fired alerts. Designed to be called
//! per-symbol on every market-data tick.
//!
//! ### Supported rule types
//!
//! * **Price crossing** — fires on first observation that crosses
//!   above/below a target. Caller carries a `last_value` per rule so
//!   the engine can detect the crossing.
//! * **Volume spike** — fires when current volume ≥ multiplier × avg.
//! * **Short interest change** — fires on month-over-month delta.
//! * **CTB change** — wraps `borrow_rate_alert` thresholds.
//! * **Squeeze score** — wraps `squeeze_score::compute` and fires when
//!   the grade reaches or exceeds a target.
//! * **Custom predicate** — for power users, a `MetricThreshold` over
//!   any of the named numeric fields on `MarketSnapshot`.
//!
//! Pure compute, no I/O. The caller is responsible for fetching the
//! snapshot and persisting `RuleState`.

use crate::squeeze_score::{self, ScoreWeights, SqueezeFactors, SqueezeGrade};
use serde::{Deserialize, Serialize};

/// Per-symbol numeric snapshot used to evaluate rules.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MarketSnapshot {
    pub symbol_id: u64,
    pub price: f64,
    pub day_volume: f64,
    pub avg_daily_volume_30d: f64,
    pub short_float_pct: f64,
    pub short_interest_change_mom: f64,
    pub days_to_cover: f64,
    pub cost_to_borrow_apr: f64,
    pub ftd_pct_of_outstanding: f64,
    pub price_momentum_10d: f64,
    pub call_put_oi_ratio: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CrossDirection {
    /// Fire when value moves above the target (`prev <= target < cur`).
    Above,
    /// Fire when value moves below the target (`prev >= target > cur`).
    Below,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricField {
    Price,
    DayVolume,
    ShortFloatPct,
    DaysToCover,
    CostToBorrowApr,
    FtdPctOfOutstanding,
    PriceMomentum10d,
    CallPutOiRatio,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Comparison {
    Gte,
    Lte,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AlertRule {
    /// Fire once when `price` crosses `target` in `direction`.
    PriceCross {
        target: f64,
        direction: CrossDirection,
    },
    /// Fire when current day_volume ≥ `multiplier` × avg_daily_volume_30d.
    VolumeSpike { multiplier: f64 },
    /// Fire when |short_interest_change_mom| ≥ `min_abs_delta` (decimal,
    /// e.g. 0.05 = 5%).
    ShortInterestChange { min_abs_delta: f64 },
    /// Fire when cost_to_borrow_apr ≥ `min_apr`.
    BorrowRateLevel { min_apr: f64 },
    /// Fire when squeeze_score grade is at least `min_grade`.
    SqueezeGradeAtLeast { min_grade: SqueezeGrade },
    /// Generic threshold on a named field.
    MetricThreshold {
        field: MetricField,
        cmp: Comparison,
        value: f64,
    },
}

/// Per-rule mutable state the caller carries across ticks. Currently
/// only `PriceCross` needs it (the previous price for crossing
/// detection); other rules are stateless and ignore it.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct RuleState {
    pub last_price: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiredAlert {
    pub symbol_id: u64,
    pub rule_index: usize,
    pub message: String,
    /// Snapshot of relevant metrics at fire time for display.
    pub fired_value: f64,
    pub rule_target: f64,
}

/// Evaluate a single rule. `state` is mutated in place (only `PriceCross`
/// updates it). Returns `Some` when the rule fires this tick.
pub fn evaluate(
    rule: &AlertRule,
    rule_index: usize,
    snapshot: &MarketSnapshot,
    state: &mut RuleState,
) -> Option<FiredAlert> {
    match rule {
        AlertRule::PriceCross { target, direction } => {
            let cur = snapshot.price;
            let prev = state.last_price;
            state.last_price = Some(cur);
            if !cur.is_finite() || !target.is_finite() {
                return None;
            }
            let prev = prev?;
            let fired = match direction {
                CrossDirection::Above => prev <= *target && cur > *target,
                CrossDirection::Below => prev >= *target && cur < *target,
            };
            if !fired {
                return None;
            }
            let dir = match direction {
                CrossDirection::Above => "above",
                CrossDirection::Below => "below",
            };
            Some(FiredAlert {
                symbol_id: snapshot.symbol_id,
                rule_index,
                message: format!("price crossed {dir} {target:.2}: now {cur:.2}"),
                fired_value: cur,
                rule_target: *target,
            })
        }
        AlertRule::VolumeSpike { multiplier } => {
            if !multiplier.is_finite() || *multiplier <= 0.0 || snapshot.avg_daily_volume_30d <= 0.0
            {
                return None;
            }
            let threshold = snapshot.avg_daily_volume_30d * multiplier;
            if snapshot.day_volume >= threshold {
                Some(FiredAlert {
                    symbol_id: snapshot.symbol_id,
                    rule_index,
                    message: format!(
                        "volume {:.0} ≥ {:.1}× avg ({:.0})",
                        snapshot.day_volume, multiplier, snapshot.avg_daily_volume_30d
                    ),
                    fired_value: snapshot.day_volume,
                    rule_target: threshold,
                })
            } else {
                None
            }
        }
        AlertRule::ShortInterestChange { min_abs_delta } => {
            let delta = snapshot.short_interest_change_mom;
            if !delta.is_finite() || delta.abs() < *min_abs_delta {
                return None;
            }
            Some(FiredAlert {
                symbol_id: snapshot.symbol_id,
                rule_index,
                message: format!("short-interest MoM change {:+.1}%", delta * 100.0),
                fired_value: delta,
                rule_target: *min_abs_delta,
            })
        }
        AlertRule::BorrowRateLevel { min_apr } => {
            if snapshot.cost_to_borrow_apr.is_finite() && snapshot.cost_to_borrow_apr >= *min_apr {
                Some(FiredAlert {
                    symbol_id: snapshot.symbol_id,
                    rule_index,
                    message: format!(
                        "cost-to-borrow {:.1}% APR ≥ trigger {:.1}%",
                        snapshot.cost_to_borrow_apr * 100.0,
                        min_apr * 100.0
                    ),
                    fired_value: snapshot.cost_to_borrow_apr,
                    rule_target: *min_apr,
                })
            } else {
                None
            }
        }
        AlertRule::SqueezeGradeAtLeast { min_grade } => {
            let factors = SqueezeFactors {
                short_float_pct: snapshot.short_float_pct,
                days_to_cover: snapshot.days_to_cover,
                cost_to_borrow_apr: snapshot.cost_to_borrow_apr,
                ftd_pct_of_outstanding: snapshot.ftd_pct_of_outstanding,
                price_momentum_10d: snapshot.price_momentum_10d,
                call_put_oi_ratio: snapshot.call_put_oi_ratio,
            };
            let r = squeeze_score::compute(factors, ScoreWeights::default());
            if grade_rank(r.grade) >= grade_rank(*min_grade) {
                Some(FiredAlert {
                    symbol_id: snapshot.symbol_id,
                    rule_index,
                    message: format!("squeeze score {:.1} → {:?}", r.score, r.grade),
                    fired_value: r.score,
                    rule_target: grade_floor(*min_grade),
                })
            } else {
                None
            }
        }
        AlertRule::MetricThreshold { field, cmp, value } => {
            let actual = match field {
                MetricField::Price => snapshot.price,
                MetricField::DayVolume => snapshot.day_volume,
                MetricField::ShortFloatPct => snapshot.short_float_pct,
                MetricField::DaysToCover => snapshot.days_to_cover,
                MetricField::CostToBorrowApr => snapshot.cost_to_borrow_apr,
                MetricField::FtdPctOfOutstanding => snapshot.ftd_pct_of_outstanding,
                MetricField::PriceMomentum10d => snapshot.price_momentum_10d,
                MetricField::CallPutOiRatio => snapshot.call_put_oi_ratio,
            };
            if !actual.is_finite() || !value.is_finite() {
                return None;
            }
            let fired = match cmp {
                Comparison::Gte => actual >= *value,
                Comparison::Lte => actual <= *value,
            };
            if !fired {
                return None;
            }
            let op = match cmp {
                Comparison::Gte => "≥",
                Comparison::Lte => "≤",
            };
            Some(FiredAlert {
                symbol_id: snapshot.symbol_id,
                rule_index,
                message: format!("{:?} {} {} (actual {})", field, op, value, actual),
                fired_value: actual,
                rule_target: *value,
            })
        }
    }
}

fn grade_rank(g: SqueezeGrade) -> u8 {
    match g {
        SqueezeGrade::None => 0,
        SqueezeGrade::Low => 1,
        SqueezeGrade::Moderate => 2,
        SqueezeGrade::High => 3,
        SqueezeGrade::Extreme => 4,
    }
}

fn grade_floor(g: SqueezeGrade) -> f64 {
    match g {
        SqueezeGrade::None => 0.0,
        SqueezeGrade::Low => 30.0,
        SqueezeGrade::Moderate => 50.0,
        SqueezeGrade::High => 70.0,
        SqueezeGrade::Extreme => 90.0,
    }
}

/// Evaluate every rule in a batch. `states` is a parallel array kept by
/// the caller — must be the same length as `rules`. Returns one
/// `FiredAlert` per rule that fired.
pub fn evaluate_all(
    rules: &[AlertRule],
    states: &mut [RuleState],
    snapshot: &MarketSnapshot,
) -> Vec<FiredAlert> {
    assert_eq!(rules.len(), states.len());
    rules
        .iter()
        .enumerate()
        .filter_map(|(i, r)| evaluate(r, i, snapshot, &mut states[i]))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snap(price: f64) -> MarketSnapshot {
        MarketSnapshot {
            symbol_id: 1,
            price,
            day_volume: 0.0,
            avg_daily_volume_30d: 0.0,
            short_float_pct: 0.0,
            short_interest_change_mom: 0.0,
            days_to_cover: 0.0,
            cost_to_borrow_apr: 0.0,
            ftd_pct_of_outstanding: 0.0,
            price_momentum_10d: 0.0,
            call_put_oi_ratio: 1.0,
        }
    }

    #[test]
    fn price_cross_above_fires_only_on_first_crossing() {
        let rule = AlertRule::PriceCross {
            target: 100.0,
            direction: CrossDirection::Above,
        };
        let mut state = RuleState::default();
        // First tick: no prev, no fire.
        assert!(evaluate(&rule, 0, &snap(50.0), &mut state).is_none());
        // Second tick: still below, no fire.
        assert!(evaluate(&rule, 0, &snap(75.0), &mut state).is_none());
        // Third tick: crosses above 100 — fire.
        let alert = evaluate(&rule, 0, &snap(105.0), &mut state).expect("should fire");
        assert!(alert.message.contains("above 100"));
        // Fourth tick: stays above, no re-fire.
        assert!(evaluate(&rule, 0, &snap(110.0), &mut state).is_none());
    }

    #[test]
    fn price_cross_below_fires_on_drop() {
        let rule = AlertRule::PriceCross {
            target: 50.0,
            direction: CrossDirection::Below,
        };
        let mut state = RuleState::default();
        evaluate(&rule, 0, &snap(60.0), &mut state);
        let alert = evaluate(&rule, 0, &snap(45.0), &mut state).expect("should fire");
        assert!(alert.message.contains("below 50"));
    }

    #[test]
    fn volume_spike_fires_at_threshold() {
        let mut s = snap(100.0);
        s.day_volume = 10_000_000.0;
        s.avg_daily_volume_30d = 2_000_000.0; // 5× avg
        let rule = AlertRule::VolumeSpike { multiplier: 3.0 };
        let mut state = RuleState::default();
        let alert = evaluate(&rule, 0, &s, &mut state).expect("should fire");
        assert!(alert.fired_value >= alert.rule_target);
    }

    #[test]
    fn volume_spike_doesnt_fire_below_threshold() {
        let mut s = snap(100.0);
        s.day_volume = 2_500_000.0;
        s.avg_daily_volume_30d = 1_000_000.0; // 2.5× avg
        let rule = AlertRule::VolumeSpike { multiplier: 3.0 };
        let mut state = RuleState::default();
        assert!(evaluate(&rule, 0, &s, &mut state).is_none());
    }

    #[test]
    fn short_interest_change_uses_absolute_value() {
        let mut s = snap(100.0);
        s.short_interest_change_mom = -0.08; // -8% MoM
        let rule = AlertRule::ShortInterestChange {
            min_abs_delta: 0.05,
        };
        let mut state = RuleState::default();
        let alert = evaluate(&rule, 0, &s, &mut state).expect("should fire");
        assert!(alert.message.contains("-8.0%"));
    }

    #[test]
    fn borrow_rate_level_fires_above_trigger() {
        let mut s = snap(100.0);
        s.cost_to_borrow_apr = 0.75;
        let rule = AlertRule::BorrowRateLevel { min_apr: 0.50 };
        let mut state = RuleState::default();
        assert!(evaluate(&rule, 0, &s, &mut state).is_some());
    }

    #[test]
    fn squeeze_grade_alert_fires_when_meets_threshold() {
        let mut s = snap(100.0);
        s.short_float_pct = 0.60;
        s.days_to_cover = 10.0;
        s.cost_to_borrow_apr = 1.0;
        s.ftd_pct_of_outstanding = 0.05;
        s.price_momentum_10d = 0.30;
        s.call_put_oi_ratio = 5.0;
        let rule = AlertRule::SqueezeGradeAtLeast {
            min_grade: SqueezeGrade::High,
        };
        let mut state = RuleState::default();
        let alert = evaluate(&rule, 0, &s, &mut state).expect("should fire");
        assert!(alert.fired_value >= 70.0);
    }

    #[test]
    fn squeeze_grade_below_threshold_doesnt_fire() {
        let s = snap(100.0); // all zero → score ≈ low
        let rule = AlertRule::SqueezeGradeAtLeast {
            min_grade: SqueezeGrade::High,
        };
        let mut state = RuleState::default();
        assert!(evaluate(&rule, 0, &s, &mut state).is_none());
    }

    #[test]
    fn metric_threshold_gte_and_lte() {
        let mut s = snap(100.0);
        s.cost_to_borrow_apr = 0.60;

        let mut state = RuleState::default();
        let gte_rule = AlertRule::MetricThreshold {
            field: MetricField::CostToBorrowApr,
            cmp: Comparison::Gte,
            value: 0.50,
        };
        assert!(evaluate(&gte_rule, 0, &s, &mut state).is_some());

        let lte_rule = AlertRule::MetricThreshold {
            field: MetricField::CostToBorrowApr,
            cmp: Comparison::Lte,
            value: 0.50,
        };
        let mut state2 = RuleState::default();
        assert!(evaluate(&lte_rule, 0, &s, &mut state2).is_none());
    }

    #[test]
    fn evaluate_all_runs_every_rule_with_parallel_states() {
        let rules = vec![
            AlertRule::PriceCross {
                target: 100.0,
                direction: CrossDirection::Above,
            },
            AlertRule::VolumeSpike { multiplier: 3.0 },
            AlertRule::BorrowRateLevel { min_apr: 0.50 },
        ];
        let mut states = vec![RuleState::default(); rules.len()];
        // Prime price-cross state with a prev value.
        states[0].last_price = Some(50.0);

        let mut s = snap(150.0);
        s.day_volume = 20_000_000.0;
        s.avg_daily_volume_30d = 2_000_000.0;
        s.cost_to_borrow_apr = 0.75;

        let fired = evaluate_all(&rules, &mut states, &s);
        assert_eq!(fired.len(), 3, "all three rules should fire");
        let indices: Vec<usize> = fired.iter().map(|a| a.rule_index).collect();
        assert!(indices.contains(&0));
        assert!(indices.contains(&1));
        assert!(indices.contains(&2));
    }

    #[test]
    fn nan_inputs_dont_fire() {
        let mut s = snap(f64::NAN);
        s.cost_to_borrow_apr = f64::NAN;
        let mut state = RuleState::default();
        assert!(evaluate(
            &AlertRule::BorrowRateLevel { min_apr: 0.5 },
            0,
            &s,
            &mut state
        )
        .is_none());
    }

    #[test]
    #[should_panic(expected = "")]
    fn evaluate_all_panics_on_mismatched_state_length() {
        let rules = vec![AlertRule::VolumeSpike { multiplier: 3.0 }];
        let mut states = Vec::<RuleState>::new();
        let s = snap(100.0);
        let _ = evaluate_all(&rules, &mut states, &s);
    }
}
