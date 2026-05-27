//! Pyramiding (scaling-into-winners) rules planner.
//!
//! Distinct from `crate::pyramid` (which builds a forward execution
//! plan). This module ENFORCES rules a pyramiding system should follow:
//!
//!   - Maximum N tranches (typically 3-5)
//!   - Each tranche only after price moves favorably by X×ATR
//!   - Stop raised on every add — never widens
//!   - Total exposure capped (cumulative size never exceeds cap)
//!   - Half-size additions (each new add is half the previous)
//!
//! Validates a sequence of (price, qty) adds against these rules and
//! emits violations.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PyramidAdd {
    pub price: f64,
    pub qty: f64,
    pub raised_stop_to: Option<f64>,
}

#[derive(Debug, Clone, Copy)]
pub struct PyramidRules {
    pub max_tranches: usize,
    pub min_favorable_atrs_between_adds: f64,
    pub require_stop_raise: bool,
    pub require_half_size_decay: bool,
    pub max_total_qty: f64,
}

impl Default for PyramidRules {
    fn default() -> Self {
        Self {
            max_tranches: 4,
            min_favorable_atrs_between_adds: 1.0,
            require_stop_raise: true,
            require_half_size_decay: true,
            max_total_qty: f64::INFINITY,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViolationKind {
    TooManyTranches,
    AddBelowMinSpacing,
    StopNotRaised,
    StopWidened,
    TrancheSizeNotHalved,
    TotalQtyExceeded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    pub kind: ViolationKind,
    pub tranche_index: usize,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ValidationReport {
    pub violations: Vec<Violation>,
    pub total_qty: f64,
    pub is_long: bool,
    pub passed: bool,
}

pub fn validate(
    entry_price: f64,
    adds: &[PyramidAdd],
    atr: f64,
    is_long: bool,
    rules: &PyramidRules,
) -> ValidationReport {
    let mut report = ValidationReport { is_long, ..Default::default() };
    if adds.len() + 1 > rules.max_tranches {
        report.violations.push(Violation {
            kind: ViolationKind::TooManyTranches,
            tranche_index: adds.len(),
            detail: format!("{} adds + initial entry > {} max", adds.len(), rules.max_tranches),
        });
    }
    let initial_qty = 1.0_f64;    // implied; real qty comes from caller via adds
    let mut last_price = entry_price;
    let mut last_qty = initial_qty;
    let mut last_stop: Option<f64> = None;
    let mut total = initial_qty;
    let min_spacing = rules.min_favorable_atrs_between_adds * atr;
    for (i, add) in adds.iter().enumerate() {
        let direction_distance = if is_long { add.price - last_price } else { last_price - add.price };
        if direction_distance < min_spacing {
            report.violations.push(Violation {
                kind: ViolationKind::AddBelowMinSpacing,
                tranche_index: i + 1,
                detail: format!("add at {} only {} above prior {} (need {})",
                    add.price, direction_distance, last_price, min_spacing),
            });
        }
        if rules.require_stop_raise {
            match (add.raised_stop_to, last_stop) {
                (None, _) => {
                    report.violations.push(Violation {
                        kind: ViolationKind::StopNotRaised,
                        tranche_index: i + 1,
                        detail: "stop not raised on add".into(),
                    });
                }
                (Some(new_stop), Some(prev)) => {
                    let raised = if is_long { new_stop > prev } else { new_stop < prev };
                    if !raised {
                        report.violations.push(Violation {
                            kind: ViolationKind::StopWidened,
                            tranche_index: i + 1,
                            detail: format!("stop {} not better than prior {}", new_stop, prev),
                        });
                    }
                }
                _ => {}
            }
        }
        if rules.require_half_size_decay && add.qty > last_qty * 0.5 + 1e-9 {
            report.violations.push(Violation {
                kind: ViolationKind::TrancheSizeNotHalved,
                tranche_index: i + 1,
                detail: format!("add qty {} exceeds half of prior {}", add.qty, last_qty),
            });
        }
        total += add.qty;
        if total > rules.max_total_qty {
            report.violations.push(Violation {
                kind: ViolationKind::TotalQtyExceeded,
                tranche_index: i + 1,
                detail: format!("cumulative {} > max {}", total, rules.max_total_qty),
            });
        }
        last_price = add.price;
        last_qty = add.qty;
        if let Some(s) = add.raised_stop_to { last_stop = Some(s); }
    }
    report.total_qty = total;
    report.passed = report.violations.is_empty();
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn add(price: f64, qty: f64, stop: Option<f64>) -> PyramidAdd {
        PyramidAdd { price, qty, raised_stop_to: stop }
    }

    fn rules() -> PyramidRules {
        PyramidRules::default()
    }

    #[test]
    fn no_adds_passes() {
        let r = validate(100.0, &[], 1.0, true, &rules());
        assert!(r.passed);
    }

    #[test]
    fn proper_pyramid_passes_all_gates() {
        // Long entry at $100, ATR 1. Three adds at +1, +2, +3 ATR with
        // halved sizes and rising stops.
        let adds = vec![
            add(101.0, 0.5, Some(100.0)),    // first add
            add(102.0, 0.25, Some(101.0)),   // half of 0.5
            add(103.0, 0.125, Some(102.0)),  // half of 0.25
        ];
        let r = validate(100.0, &adds, 1.0, true, &rules());
        assert!(r.passed, "violations: {:?}", r.violations);
    }

    #[test]
    fn too_many_tranches_violation() {
        let adds = vec![
            add(101.0, 0.5, Some(100.0)),
            add(102.0, 0.25, Some(101.0)),
            add(103.0, 0.125, Some(102.0)),
            add(104.0, 0.06, Some(103.0)),    // 5th tranche (entry + 4 adds)
        ];
        let r = validate(100.0, &adds, 1.0, true, &rules());
        assert!(r.violations.iter().any(|v| v.kind == ViolationKind::TooManyTranches));
    }

    #[test]
    fn add_too_close_below_min_spacing() {
        // Add at only +0.5 ATR — needs ≥ 1.0 ATR.
        let adds = vec![add(100.5, 0.5, Some(100.0))];
        let r = validate(100.0, &adds, 1.0, true, &rules());
        assert!(r.violations.iter().any(|v| v.kind == ViolationKind::AddBelowMinSpacing));
    }

    #[test]
    fn stop_not_raised_violation() {
        let adds = vec![add(101.0, 0.5, None)];    // no stop raised
        let r = validate(100.0, &adds, 1.0, true, &rules());
        assert!(r.violations.iter().any(|v| v.kind == ViolationKind::StopNotRaised));
    }

    #[test]
    fn stop_widened_violation() {
        let adds = vec![
            add(101.0, 0.5, Some(100.0)),
            add(102.0, 0.25, Some(99.0)),    // widening for a long
        ];
        let r = validate(100.0, &adds, 1.0, true, &rules());
        assert!(r.violations.iter().any(|v| v.kind == ViolationKind::StopWidened));
    }

    #[test]
    fn tranche_not_halved_violation() {
        // Initial qty 1.0, add 1.0 → not halved.
        let adds = vec![add(101.0, 1.0, Some(100.0))];
        let r = validate(100.0, &adds, 1.0, true, &rules());
        assert!(r.violations.iter().any(|v| v.kind == ViolationKind::TrancheSizeNotHalved));
    }

    #[test]
    fn total_qty_exceeded_violation() {
        let strict = PyramidRules { max_total_qty: 1.4, ..rules() };
        let adds = vec![add(101.0, 0.5, Some(100.0))];    // total 1.5 > 1.4
        let r = validate(100.0, &adds, 1.0, true, &strict);
        assert!(r.violations.iter().any(|v| v.kind == ViolationKind::TotalQtyExceeded));
    }

    #[test]
    fn short_trade_spacing_uses_lower_prices() {
        // Short at $100; adds should be at LOWER prices for "favorable".
        let adds = vec![add(99.0, 0.5, Some(100.0))];    // +1 ATR favorable
        let r = validate(100.0, &adds, 1.0, false, &rules());
        // Spacing OK; stop raised (closer to entry → 100 > 99, stop "tighter").
        // For a short, stop should LOWER over time (tighter against rising price).
        // Initial stop is None; first add sets it. Should pass.
        let spacing_violations: Vec<_> = r.violations.iter()
            .filter(|v| v.kind == ViolationKind::AddBelowMinSpacing)
            .collect();
        assert!(spacing_violations.is_empty(), "short add at lower price should be OK");
    }

    #[test]
    fn total_qty_tracks_cumulative_correctly() {
        let adds = vec![
            add(101.0, 0.5, Some(100.0)),
            add(102.0, 0.25, Some(101.0)),
        ];
        let r = validate(100.0, &adds, 1.0, true, &rules());
        // initial 1.0 + 0.5 + 0.25 = 1.75.
        assert!((r.total_qty - 1.75).abs() < 1e-9);
    }
}
