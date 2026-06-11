//! Scale-out ladder planner — what each partial-exit plan actually
//! pays in R, scenario by scenario.
//!
//! Given an entry, stop, size, and a ladder of (target price, shares)
//! exits, walks every outcome: stopped before the first fill, stopped
//! after each partial fill (with or without moving the stop to
//! breakeven after the first fill), and all targets hit. Long and
//! short both supported (short = stop above entry).
//!
//! The point: ladders FEEL safe but cap the right tail — seeing
//! "stopped after T1 = −0.33R" next to "all targets = +2.0R" makes
//! the tradeoff concrete before entry.
//!
//! Pure compute. Companion to `risk_reward`, `pyramid_rules`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct LadderLeg {
    pub target_price: f64,
    pub shares: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScaleOutInput {
    pub entry: f64,
    pub stop: f64,
    /// Total position size, shares. Ladder shares must not exceed it;
    /// any remainder rides to the last target.
    pub total_shares: f64,
    /// Exits ordered nearest-first (ascending targets for longs,
    /// descending for shorts).
    pub legs: Vec<LadderLeg>,
    /// Move the stop to entry after the first leg fills.
    pub breakeven_after_first: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct Scenario {
    /// "stopped_before_t1", "stopped_after_t1", …, "all_targets".
    pub label: String,
    pub pnl: f64,
    pub r_multiple: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScaleOutReport {
    pub is_long: bool,
    pub risk_per_share: f64,
    pub total_risk: f64,
    pub scenarios: Vec<Scenario>,
    /// R if every target fills (remainder exits at the last target).
    pub max_r: f64,
}

pub fn compute(inp: &ScaleOutInput) -> Option<ScaleOutReport> {
    if ![inp.entry, inp.stop, inp.total_shares].iter().all(|v| v.is_finite())
        || inp.entry <= 0.0
        || inp.stop <= 0.0
        || inp.entry == inp.stop
        || inp.total_shares <= 0.0
        || inp.legs.is_empty()
        || inp.legs.len() > 20
    {
        return None;
    }
    let is_long = inp.entry > inp.stop;
    let dir = if is_long { 1.0 } else { -1.0 };
    let risk_per_share = (inp.entry - inp.stop).abs();
    let total_risk = risk_per_share * inp.total_shares;
    let mut ladder_shares = 0.0;
    let mut prev_target = inp.entry;
    for leg in &inp.legs {
        if !leg.target_price.is_finite()
            || leg.target_price <= 0.0
            || !leg.shares.is_finite()
            || leg.shares <= 0.0
            // Targets must move away from entry in the trade direction,
            // nearest first.
            || dir * (leg.target_price - prev_target) <= 0.0
        {
            return None;
        }
        ladder_shares += leg.shares;
        prev_target = leg.target_price;
    }
    if ladder_shares > inp.total_shares + 1e-9 {
        return None;
    }
    let pnl_per_share = |exit: f64| dir * (exit - inp.entry);
    let mut scenarios = Vec::with_capacity(inp.legs.len() + 2);
    // Stopped before anything fills.
    scenarios.push(Scenario {
        label: "stopped_before_t1".into(),
        pnl: -total_risk,
        r_multiple: -1.0,
    });
    // Stopped after each partial fill.
    let mut banked = 0.0;
    let mut filled = 0.0;
    for (i, leg) in inp.legs.iter().enumerate() {
        banked += pnl_per_share(leg.target_price) * leg.shares;
        filled += leg.shares;
        let remaining = inp.total_shares - filled;
        let stop_price = if inp.breakeven_after_first {
            inp.entry
        } else {
            inp.stop
        };
        let pnl = banked + pnl_per_share(stop_price) * remaining;
        scenarios.push(Scenario {
            label: format!("stopped_after_t{}", i + 1),
            pnl,
            r_multiple: pnl / total_risk,
        });
    }
    // All targets hit; remainder exits at the last target.
    let last_target = inp.legs.last().expect("non-empty").target_price;
    let all = banked + pnl_per_share(last_target) * (inp.total_shares - filled);
    scenarios.push(Scenario {
        label: "all_targets".into(),
        pnl: all,
        r_multiple: all / total_risk,
    });
    Some(ScaleOutReport {
        is_long,
        risk_per_share,
        total_risk,
        max_r: all / total_risk,
        scenarios,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn thirds() -> ScaleOutInput {
        ScaleOutInput {
            entry: 100.0,
            stop: 95.0,
            total_shares: 300.0,
            legs: vec![
                LadderLeg { target_price: 105.0, shares: 100.0 },
                LadderLeg { target_price: 110.0, shares: 100.0 },
                LadderLeg { target_price: 115.0, shares: 100.0 },
            ],
            breakeven_after_first: false,
        }
    }

    #[test]
    fn classic_thirds_ladder_hand_walk() {
        // Risk = 300 × $5 = $1500. All targets: 500 + 1000 + 1500 =
        // $3000 = exactly +2R.
        let r = compute(&thirds()).unwrap();
        assert!((r.total_risk - 1500.0).abs() < 1e-9);
        assert!((r.max_r - 2.0).abs() < 1e-12);
        let by = |label: &str| {
            r.scenarios
                .iter()
                .find(|s| s.label == label)
                .map(|s| s.r_multiple)
                .expect("scenario")
        };
        assert!((by("stopped_before_t1") + 1.0).abs() < 1e-12);
        // After T1 (no breakeven): +500 − 200×5 = −500 = −1/3 R.
        assert!((by("stopped_after_t1") + 1.0 / 3.0).abs() < 1e-12);
        // After T2: +1500 − 100×5 = +1000 = +2/3 R.
        assert!((by("stopped_after_t2") - 2.0 / 3.0).abs() < 1e-12);
        assert!((by("all_targets") - 2.0).abs() < 1e-12);
    }

    #[test]
    fn breakeven_stop_removes_the_post_t1_loss() {
        let mut inp = thirds();
        inp.breakeven_after_first = true;
        let r = compute(&inp).unwrap();
        let after_t1 = r
            .scenarios
            .iter()
            .find(|s| s.label == "stopped_after_t1")
            .expect("scenario");
        // +500 banked, remainder flat at entry = +1/3 R.
        assert!((after_t1.r_multiple - 1.0 / 3.0).abs() < 1e-12);
    }

    #[test]
    fn short_side_mirrors_the_long() {
        let r = compute(&ScaleOutInput {
            entry: 100.0,
            stop: 105.0,
            total_shares: 300.0,
            legs: vec![
                LadderLeg { target_price: 95.0, shares: 100.0 },
                LadderLeg { target_price: 90.0, shares: 100.0 },
                LadderLeg { target_price: 85.0, shares: 100.0 },
            ],
            breakeven_after_first: false,
        })
        .unwrap();
        assert!(!r.is_long);
        assert!((r.max_r - 2.0).abs() < 1e-12);
    }

    #[test]
    fn remainder_rides_to_last_target() {
        // Ladder only covers 200 of 300 shares: the loose 100 exit at
        // the last target. All-hit = 500 + 1000 + 100×10 = $2500.
        let mut inp = thirds();
        inp.legs.truncate(2);
        let r = compute(&inp).unwrap();
        assert!((r.max_r - 2500.0 / 1500.0).abs() < 1e-12);
    }

    #[test]
    fn hostile_inputs_return_none() {
        let mut bad = thirds();
        bad.legs[1].target_price = 104.0; // out of order
        assert!(compute(&bad).is_none());
        let mut bad = thirds();
        bad.legs[0].target_price = 99.0; // wrong side of entry for a long
        assert!(compute(&bad).is_none());
        let mut bad = thirds();
        bad.total_shares = 250.0; // ladder oversells the position
        assert!(compute(&bad).is_none());
        let mut bad = thirds();
        bad.stop = 100.0; // zero risk
        assert!(compute(&bad).is_none());
        assert!(compute(&ScaleOutInput { legs: vec![], ..thirds() }).is_none());
    }
}
