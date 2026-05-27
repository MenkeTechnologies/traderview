//! Discipline score — a single 0-100 number summarizing how well the
//! trader stuck to their rules in a window.
//!
//! Combines two signals that already exist independently:
//!   * Post-trade rule evals (traderview-db::discipline) — stop set?
//!     stop honored? qty within plan? direction matched?
//!   * Pre-trade Risk Gate fires (traderview-db::risk_rules::recent_fires)
//!     — how many times did the user try to break a rule before placing
//!     a trade?
//!
//! The composite math is intentionally simple — every component is a 0-1
//! ratio, weights are explicit, and the final number is clamped to
//! [0, 100]. No floats touch P&L; this is just a personal-trader metric.

use serde::{Deserialize, Serialize};

/// Inputs the score needs. The DB-layer caller (`traderview-db`) builds
/// these by querying the discipline + risk_fires tables.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScoreInputs {
    /// Number of trades closed in the window.
    pub trades: u32,
    /// Subset of those where the user had a stop set when opening.
    pub trades_with_stop: u32,
    /// Subset where the recorded stop wasn't violated past plan tolerance.
    pub trades_stop_honored: u32,
    /// Subset where qty matched the linked plan (within tolerance).
    pub trades_qty_within_plan: u32,
    /// Subset where the side (long/short) matched the plan.
    pub trades_direction_matched: u32,
    /// Total Risk Gate warning fires in the window.
    pub gate_warnings: u32,
    /// Total Risk Gate block fires (the user TRIED to break a rule).
    pub gate_blocks: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DisciplineScore {
    pub score: u8,     // 0..=100
    pub grade: String, // "A+" .. "F"
    /// 0..=1, fraction of trades with a stop set on entry.
    pub stop_set_rate: f64,
    /// 0..=1, fraction of stopped trades where the stop wasn't violated.
    pub stop_honored_rate: f64,
    /// 0..=1, fraction of trades whose qty + direction matched the plan.
    pub plan_adherence_rate: f64,
    pub gate_warnings: u32,
    pub gate_blocks: u32,
    /// Per-component breakdown so the UI can show what's pulling the
    /// score down. Each is a 0..=100 sub-score.
    pub component_stop_set: u8,
    pub component_stop_honored: u8,
    pub component_plan_adherence: u8,
    pub component_gate_restraint: u8,
}

const COMPONENT_WEIGHT_STOP_SET: f64 = 0.20;
const COMPONENT_WEIGHT_STOP_HONORED: f64 = 0.30;
const COMPONENT_WEIGHT_PLAN_ADHERENCE: f64 = 0.25;
const COMPONENT_WEIGHT_GATE_RESTRAINT: f64 = 0.25;

pub fn score(inputs: &ScoreInputs) -> DisciplineScore {
    // No trades + no fires → not really a discipline event. Give the
    // benefit of the doubt with a neutral score so an empty day doesn't
    // tank the running average.
    if inputs.trades == 0 && inputs.gate_blocks == 0 && inputs.gate_warnings == 0 {
        return DisciplineScore {
            score: 100,
            grade: "A+".into(),
            stop_set_rate: 0.0,
            stop_honored_rate: 0.0,
            plan_adherence_rate: 0.0,
            gate_warnings: 0,
            gate_blocks: 0,
            component_stop_set: 100,
            component_stop_honored: 100,
            component_plan_adherence: 100,
            component_gate_restraint: 100,
        };
    }

    let trades = inputs.trades.max(1) as f64;
    let stop_set_rate = inputs.trades_with_stop as f64 / trades;
    let stop_honored_rate = if inputs.trades_with_stop > 0 {
        inputs.trades_stop_honored as f64 / inputs.trades_with_stop as f64
    } else {
        0.0
    };
    let plan_adherence_rate = {
        let qty = inputs.trades_qty_within_plan as f64 / trades;
        let dir = inputs.trades_direction_matched as f64 / trades;
        (qty + dir) / 2.0
    };
    // Gate restraint: blocks weigh 5× warnings (you actively tried to
    // break a rule vs just edged near one). 0 fires → perfect 1.0; the
    // more you tried, the lower it goes. Tuned so 4 blocks → ~0.0.
    let weighted_misbehavior = (inputs.gate_blocks as f64) * 5.0 + (inputs.gate_warnings as f64);
    let gate_restraint = (1.0 - weighted_misbehavior / 20.0).clamp(0.0, 1.0);

    let final_pct = COMPONENT_WEIGHT_STOP_SET * stop_set_rate
        + COMPONENT_WEIGHT_STOP_HONORED * stop_honored_rate
        + COMPONENT_WEIGHT_PLAN_ADHERENCE * plan_adherence_rate
        + COMPONENT_WEIGHT_GATE_RESTRAINT * gate_restraint;
    let score_u8 = (final_pct * 100.0).round().clamp(0.0, 100.0) as u8;

    DisciplineScore {
        score: score_u8,
        grade: grade_letter(score_u8),
        stop_set_rate,
        stop_honored_rate,
        plan_adherence_rate,
        gate_warnings: inputs.gate_warnings,
        gate_blocks: inputs.gate_blocks,
        component_stop_set: (stop_set_rate * 100.0).round() as u8,
        component_stop_honored: (stop_honored_rate * 100.0).round() as u8,
        component_plan_adherence: (plan_adherence_rate * 100.0).round() as u8,
        component_gate_restraint: (gate_restraint * 100.0).round() as u8,
    }
}

fn grade_letter(score: u8) -> String {
    match score {
        97..=100 => "A+",
        93..=96 => "A",
        90..=92 => "A-",
        87..=89 => "B+",
        83..=86 => "B",
        80..=82 => "B-",
        77..=79 => "C+",
        73..=76 => "C",
        70..=72 => "C-",
        60..=69 => "D",
        _ => "F",
    }
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn perfect_trader() -> ScoreInputs {
        ScoreInputs {
            trades: 10,
            trades_with_stop: 10,
            trades_stop_honored: 10,
            trades_qty_within_plan: 10,
            trades_direction_matched: 10,
            gate_warnings: 0,
            gate_blocks: 0,
        }
    }

    #[test]
    fn perfect_trader_scores_100_grade_a_plus() {
        let s = score(&perfect_trader());
        assert_eq!(s.score, 100);
        assert_eq!(s.grade, "A+");
    }

    #[test]
    fn empty_day_is_a_plus_not_zero() {
        // No trades and no fires — give credit, don't punish an off day.
        let s = score(&ScoreInputs::default());
        assert_eq!(s.score, 100);
    }

    #[test]
    fn no_stops_set_significantly_drops_the_score() {
        let mut i = perfect_trader();
        i.trades_with_stop = 0;
        i.trades_stop_honored = 0;
        let s = score(&i);
        // Losing stop_set (0.20) + stop_honored (0.30) drops by 50pts.
        assert!(
            s.score < 60,
            "stop-less trading must heavily punish, got {}",
            s.score
        );
    }

    #[test]
    fn one_block_fire_drops_score_by_about_six() {
        // 1 block × 5 / 20 = 0.25 lost gate restraint × 0.25 weight = 6.25
        let mut i = perfect_trader();
        i.gate_blocks = 1;
        let s = score(&i);
        assert!(
            (90..=95).contains(&s.score),
            "one block should pull a perfect score down to ~94, got {}",
            s.score
        );
    }

    #[test]
    fn four_blocks_zeroes_gate_restraint_component() {
        let mut i = perfect_trader();
        i.gate_blocks = 4;
        let s = score(&i);
        assert_eq!(
            s.component_gate_restraint, 0,
            "4 blocks must zero the restraint component (20pts × blocks ≥ 20 cap)"
        );
    }

    #[test]
    fn many_warnings_alone_dont_zero_the_score() {
        let mut i = perfect_trader();
        i.gate_warnings = 20; // a lot of warnings — gate component → 0
        let s = score(&i);
        // 75 from the other three components (20+30+25), gate → 0.
        assert_eq!(s.score, 75);
    }

    #[test]
    fn plan_adherence_is_qty_and_direction_avg() {
        let mut i = perfect_trader();
        i.trades_qty_within_plan = 5; // 50%
        i.trades_direction_matched = 10; // 100%
        let s = score(&i);
        // plan_adherence = (0.5 + 1.0) / 2 = 0.75 → 25 × 0.75 = 18.75 component
        // Other three perfect: 20 + 30 + 25 = 75. Total = 93.75 → 94.
        assert_eq!(s.score, 94);
    }

    #[test]
    fn stop_honored_rate_uses_stopped_subset_as_denominator() {
        // 10 trades, 5 had stops, 5 of those honored → 100% honored rate
        // even though only 50% had stops. Each component scored separately.
        let mut i = perfect_trader();
        i.trades_with_stop = 5;
        i.trades_stop_honored = 5;
        let s = score(&i);
        assert_eq!(
            s.component_stop_honored, 100,
            "honored rate must be over the STOPPED subset, not all trades"
        );
        // stop_set component drops to 50; the rest stay 100.
        assert_eq!(s.component_stop_set, 50);
    }

    #[test]
    fn grade_letter_boundaries() {
        for (score_val, expected_grade) in [
            (100, "A+"),
            (97, "A+"),
            (96, "A"),
            (93, "A"),
            (92, "A-"),
            (90, "A-"),
            (89, "B+"),
            (87, "B+"),
            (80, "B-"),
            (75, "C"),
            (62, "D"),
            (50, "F"),
            (0, "F"),
        ] {
            assert_eq!(
                grade_letter(score_val),
                expected_grade,
                "score {score_val} should map to {expected_grade}"
            );
        }
    }

    #[test]
    fn score_is_always_in_zero_hundred_range() {
        // Adversarial input: every component bad, every gate fire maxed.
        let s = score(&ScoreInputs {
            trades: 100,
            trades_with_stop: 0,
            trades_stop_honored: 0,
            trades_qty_within_plan: 0,
            trades_direction_matched: 0,
            gate_warnings: 1000,
            gate_blocks: 1000,
        });
        assert!(s.score <= 100);
        // Even total catastrophe stays clamped, not negative or wrap.
    }
}
