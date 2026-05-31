//! IRC §1092 — Straddle loss deferral and holding-period suspension.
//!
//! Every active trader doing options or futures hedging hits this.
//! A **straddle** under §1092(c)(1) is two or more positions in
//! actively traded personal property that **substantially diminish**
//! the taxpayer's risk of loss from holding any one of them. The
//! anti-abuse rules:
//!
//!   * **§1092(a)(1) loss deferral** — loss on disposition of one
//!     leg is recognized ONLY to the extent it exceeds the sum of
//!     unrecognized gains on any offsetting position(s) held at the
//!     end of the tax year. The disallowed portion carries forward
//!     to the year the offsetting position is closed.
//!
//!   * **§1092(b)(2) holding-period suspension** — the holding
//!     period of each position in a straddle is SUSPENDED while the
//!     straddle remains open. This prevents shorts-against-the-box
//!     style conversion of short-term gain into long-term.
//!
//!   * **§1092(c)(4)(B) qualified covered call (QCC) exception** — a
//!     covered call (long stock + written call) is NOT a straddle if
//!     the call meets ALL of: underlying is a publicly traded stock;
//!     call has more than 30 days to expiration when written; strike
//!     is not deep in the money (not below the "lowest qualified
//!     benchmark"). QCC writers preserve their long-stock holding
//!     period and don't defer the loss when the call expires.
//!
//! Pure compute. Caller asserts the straddle facts; we compute the current-year deductible loss, the deferred loss, and the holding-period-suspension flag.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StraddleLeg {
    pub symbol: String,
    /// Current FMV of the position.
    pub fair_market_value: Decimal,
    /// Adjusted basis (cost).
    pub adjusted_basis: Decimal,
    /// True for the leg that the taxpayer is currently disposing of
    /// at a loss. The other legs are held into the next tax year.
    pub being_disposed_at_loss: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualifiedCoveredCallFacts {
    pub publicly_traded_underlying: bool,
    pub days_to_expiration_at_writing: u32,
    pub strike_above_lowest_qualified_benchmark: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section1092Input {
    /// All legs participating in the straddle, including the one
    /// being disposed of. Caller asserts which is being disposed via
    /// `being_disposed_at_loss: true`.
    pub legs: Vec<StraddleLeg>,
    /// Realized loss on the disposed leg (positive number — we
    /// compute the limitation on its deductibility).
    pub realized_loss_on_disposed_leg: Decimal,
    /// If this straddle qualifies for the §1092(c)(4)(B) QCC
    /// exception — caller supplies the QCC facts. None = not a QCC
    /// (subject to §1092).
    pub qualified_covered_call_facts: Option<QualifiedCoveredCallFacts>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Section1092Result {
    pub is_qualified_covered_call: bool,
    /// Unrecognized gain on all offsetting legs (legs not being
    /// disposed of, where FMV > basis).
    pub unrecognized_gain_on_offsetting_legs: Decimal,
    pub loss_recognized_this_year: Decimal,
    pub loss_deferred: Decimal,
    pub holding_period_suspended: bool,
    pub note: String,
}

fn qcc_qualifies(facts: &QualifiedCoveredCallFacts) -> bool {
    facts.publicly_traded_underlying
        && facts.days_to_expiration_at_writing > 30
        && facts.strike_above_lowest_qualified_benchmark
}

pub fn compute(input: &Section1092Input) -> Section1092Result {
    let mut r = Section1092Result::default();

    // §1092(c)(4)(B) QCC short-circuit.
    if let Some(qcc) = &input.qualified_covered_call_facts {
        if qcc_qualifies(qcc) {
            r.is_qualified_covered_call = true;
            r.loss_recognized_this_year = input.realized_loss_on_disposed_leg;
            r.loss_deferred = Decimal::ZERO;
            r.holding_period_suspended = false;
            r.note = "§1092(c)(4)(B) qualified covered call: not a straddle. Loss fully recognized; holding period preserved on long stock.".into();
            return r;
        }
    }

    // Sum unrecognized gain on offsetting legs (FMV - basis when positive,
    // for legs that aren't the one being disposed).
    for leg in &input.legs {
        if leg.being_disposed_at_loss {
            continue;
        }
        let unrealized = leg.fair_market_value - leg.adjusted_basis;
        if unrealized > Decimal::ZERO {
            r.unrecognized_gain_on_offsetting_legs += unrealized;
        }
    }

    if input.realized_loss_on_disposed_leg <= Decimal::ZERO {
        r.note = "no loss to defer (§1092 applies only to losses)".into();
        return r;
    }

    if r.unrecognized_gain_on_offsetting_legs <= Decimal::ZERO {
        // No unrecognized gain to absorb the loss — full recognition.
        r.loss_recognized_this_year = input.realized_loss_on_disposed_leg;
        r.loss_deferred = Decimal::ZERO;
        r.holding_period_suspended = true; // still a straddle even if no offset
        r.note = format!(
            "§1092: ${} loss fully recognized (no unrecognized gain on offsetting legs to defer against). Holding period suspended while straddle open.",
            r.loss_recognized_this_year
        );
        return r;
    }

    // Loss deferred up to unrecognized gain; excess recognized.
    r.loss_deferred = input
        .realized_loss_on_disposed_leg
        .min(r.unrecognized_gain_on_offsetting_legs);
    r.loss_recognized_this_year = input.realized_loss_on_disposed_leg - r.loss_deferred;
    r.holding_period_suspended = true;
    r.note = format!(
        "§1092(a)(1) deferral: ${} of ${} loss deferred against ${} unrecognized gain on offsetting legs; ${} recognized this year. Holding period suspended on remaining straddle positions.",
        r.loss_deferred,
        input.realized_loss_on_disposed_leg,
        r.unrecognized_gain_on_offsetting_legs,
        r.loss_recognized_this_year
    );
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn leg(symbol: &str, fmv: Decimal, basis: Decimal, disposed: bool) -> StraddleLeg {
        StraddleLeg {
            symbol: symbol.into(),
            fair_market_value: fmv,
            adjusted_basis: basis,
            being_disposed_at_loss: disposed,
        }
    }

    #[test]
    fn loss_fully_deferred_when_gain_on_offset_exceeds_loss() {
        // Long stock leg unrealized gain $5k; short call closed at $2k loss.
        // Deferral = min($2k, $5k) = $2k. Recognized = $0.
        let r = compute(&Section1092Input {
            legs: vec![
                leg("AAPL_LONG", dec!(15000), dec!(10000), false), // +$5k
                leg("AAPL_SHORT_CALL", dec!(500), dec!(2500), true), // closed at $2k loss
            ],
            realized_loss_on_disposed_leg: dec!(2000),
            qualified_covered_call_facts: None,
        });
        assert_eq!(r.unrecognized_gain_on_offsetting_legs, dec!(5000));
        assert_eq!(r.loss_deferred, dec!(2000));
        assert_eq!(r.loss_recognized_this_year, Decimal::ZERO);
        assert!(r.holding_period_suspended);
    }

    #[test]
    fn loss_partially_deferred_when_gain_less_than_loss() {
        // Gain $2k; loss $5k. Deferral = $2k. Recognized = $3k.
        let r = compute(&Section1092Input {
            legs: vec![
                leg("AAPL_LONG", dec!(12000), dec!(10000), false), // +$2k
                leg("AAPL_SHORT_CALL", dec!(0), dec!(5000), true),
            ],
            realized_loss_on_disposed_leg: dec!(5000),
            qualified_covered_call_facts: None,
        });
        assert_eq!(r.loss_deferred, dec!(2000));
        assert_eq!(r.loss_recognized_this_year, dec!(3000));
    }

    #[test]
    fn no_gain_on_offset_full_loss_recognized() {
        // Offsetting leg at break-even — nothing to defer against.
        let r = compute(&Section1092Input {
            legs: vec![
                leg("AAPL_LONG", dec!(10000), dec!(10000), false),
                leg("AAPL_SHORT_CALL", dec!(0), dec!(3000), true),
            ],
            realized_loss_on_disposed_leg: dec!(3000),
            qualified_covered_call_facts: None,
        });
        assert_eq!(r.loss_deferred, Decimal::ZERO);
        assert_eq!(r.loss_recognized_this_year, dec!(3000));
        assert!(r.holding_period_suspended); // still a straddle
    }

    #[test]
    fn loss_on_disposed_at_zero_no_op() {
        let r = compute(&Section1092Input {
            legs: vec![leg("X", dec!(100), dec!(50), false)],
            realized_loss_on_disposed_leg: Decimal::ZERO,
            qualified_covered_call_facts: None,
        });
        assert_eq!(r.loss_deferred, Decimal::ZERO);
        assert_eq!(r.loss_recognized_this_year, Decimal::ZERO);
        assert!(r.note.contains("no loss"));
    }

    #[test]
    fn qcc_exception_fully_qualified_recognizes_loss() {
        let r = compute(&Section1092Input {
            legs: vec![
                leg("AAPL_LONG", dec!(15000), dec!(10000), false),
                leg("AAPL_SHORT_CALL", dec!(0), dec!(2000), true),
            ],
            realized_loss_on_disposed_leg: dec!(2000),
            qualified_covered_call_facts: Some(QualifiedCoveredCallFacts {
                publicly_traded_underlying: true,
                days_to_expiration_at_writing: 45,
                strike_above_lowest_qualified_benchmark: true,
            }),
        });
        assert!(r.is_qualified_covered_call);
        assert_eq!(r.loss_recognized_this_year, dec!(2000));
        assert_eq!(r.loss_deferred, Decimal::ZERO);
        assert!(!r.holding_period_suspended);
    }

    #[test]
    fn qcc_disqualified_when_under_30_days() {
        let r = compute(&Section1092Input {
            legs: vec![
                leg("AAPL_LONG", dec!(15000), dec!(10000), false),
                leg("AAPL_SHORT_CALL", dec!(0), dec!(2000), true),
            ],
            realized_loss_on_disposed_leg: dec!(2000),
            qualified_covered_call_facts: Some(QualifiedCoveredCallFacts {
                publicly_traded_underlying: true,
                days_to_expiration_at_writing: 30, // exactly 30 disqualified
                strike_above_lowest_qualified_benchmark: true,
            }),
        });
        assert!(!r.is_qualified_covered_call);
        // Falls through to §1092 normal deferral: $5k gain vs $2k loss → defer.
        assert_eq!(r.loss_deferred, dec!(2000));
    }

    #[test]
    fn qcc_qualified_at_exactly_31_days() {
        let r = compute(&Section1092Input {
            legs: vec![
                leg("AAPL_LONG", dec!(15000), dec!(10000), false),
                leg("AAPL_SHORT_CALL", dec!(0), dec!(2000), true),
            ],
            realized_loss_on_disposed_leg: dec!(2000),
            qualified_covered_call_facts: Some(QualifiedCoveredCallFacts {
                publicly_traded_underlying: true,
                days_to_expiration_at_writing: 31,
                strike_above_lowest_qualified_benchmark: true,
            }),
        });
        assert!(r.is_qualified_covered_call);
        assert_eq!(r.loss_recognized_this_year, dec!(2000));
    }

    #[test]
    fn qcc_disqualified_when_underlying_not_publicly_traded() {
        let r = compute(&Section1092Input {
            legs: vec![
                leg("X", dec!(15000), dec!(10000), false),
                leg("X_SHORT_CALL", dec!(0), dec!(2000), true),
            ],
            realized_loss_on_disposed_leg: dec!(2000),
            qualified_covered_call_facts: Some(QualifiedCoveredCallFacts {
                publicly_traded_underlying: false,
                days_to_expiration_at_writing: 60,
                strike_above_lowest_qualified_benchmark: true,
            }),
        });
        assert!(!r.is_qualified_covered_call);
    }

    #[test]
    fn qcc_disqualified_when_strike_deep_in_money() {
        let r = compute(&Section1092Input {
            legs: vec![
                leg("AAPL_LONG", dec!(15000), dec!(10000), false),
                leg("AAPL_SHORT_CALL", dec!(0), dec!(2000), true),
            ],
            realized_loss_on_disposed_leg: dec!(2000),
            qualified_covered_call_facts: Some(QualifiedCoveredCallFacts {
                publicly_traded_underlying: true,
                days_to_expiration_at_writing: 60,
                strike_above_lowest_qualified_benchmark: false, // deep ITM
            }),
        });
        assert!(!r.is_qualified_covered_call);
    }

    #[test]
    fn multiple_offsetting_legs_sum_their_unrecognized_gains() {
        // 2 offsetting legs, $3k and $4k unrecognized gain = $7k total.
        let r = compute(&Section1092Input {
            legs: vec![
                leg("LONG_1", dec!(8000), dec!(5000), false),
                leg("LONG_2", dec!(9000), dec!(5000), false),
                leg("SHORT_CALL", dec!(0), dec!(2500), true),
            ],
            realized_loss_on_disposed_leg: dec!(2500),
            qualified_covered_call_facts: None,
        });
        assert_eq!(r.unrecognized_gain_on_offsetting_legs, dec!(7000));
        // Full loss deferred against $7k gain.
        assert_eq!(r.loss_deferred, dec!(2500));
    }

    #[test]
    fn unrealized_loss_on_offsetting_leg_doesnt_count_negative() {
        // One offsetting leg at $1k gain, another at $2k loss.
        // Only the gain counts ($1k). Loss leg DOESN'T reduce the
        // unrecognized-gain pool.
        let r = compute(&Section1092Input {
            legs: vec![
                leg("WINNER", dec!(11000), dec!(10000), false),    // +$1k
                leg("LOSER", dec!(8000), dec!(10000), false),      // -$2k (ignored)
                leg("DISPOSED", dec!(0), dec!(3000), true),
            ],
            realized_loss_on_disposed_leg: dec!(3000),
            qualified_covered_call_facts: None,
        });
        assert_eq!(r.unrecognized_gain_on_offsetting_legs, dec!(1000));
        assert_eq!(r.loss_deferred, dec!(1000));
        assert_eq!(r.loss_recognized_this_year, dec!(2000));
    }

    #[test]
    fn loss_exactly_equal_to_gain_fully_deferred_zero_recognized() {
        let r = compute(&Section1092Input {
            legs: vec![
                leg("LONG", dec!(13000), dec!(10000), false), // +$3k
                leg("SHORT_CALL", dec!(0), dec!(3000), true),
            ],
            realized_loss_on_disposed_leg: dec!(3000),
            qualified_covered_call_facts: None,
        });
        assert_eq!(r.loss_deferred, dec!(3000));
        assert_eq!(r.loss_recognized_this_year, Decimal::ZERO);
    }

    #[test]
    fn no_offsetting_legs_at_all_still_handles_straddle_field() {
        // Single-leg input (shouldn't typically happen for §1092 but
        // shouldn't panic). No gain to defer against.
        let r = compute(&Section1092Input {
            legs: vec![leg("ONLY", dec!(500), dec!(2500), true)],
            realized_loss_on_disposed_leg: dec!(2000),
            qualified_covered_call_facts: None,
        });
        assert_eq!(r.unrecognized_gain_on_offsetting_legs, Decimal::ZERO);
        assert_eq!(r.loss_recognized_this_year, dec!(2000));
    }

    #[test]
    fn note_distinguishes_qcc_path_from_normal_straddle_path() {
        let qcc = compute(&Section1092Input {
            legs: vec![
                leg("LONG", dec!(15000), dec!(10000), false),
                leg("SHORT_CALL", dec!(0), dec!(2000), true),
            ],
            realized_loss_on_disposed_leg: dec!(2000),
            qualified_covered_call_facts: Some(QualifiedCoveredCallFacts {
                publicly_traded_underlying: true,
                days_to_expiration_at_writing: 45,
                strike_above_lowest_qualified_benchmark: true,
            }),
        });
        assert!(qcc.note.contains("§1092(c)(4)(B)"));

        let straddle = compute(&Section1092Input {
            legs: vec![
                leg("LONG", dec!(15000), dec!(10000), false),
                leg("SHORT_CALL", dec!(0), dec!(2000), true),
            ],
            realized_loss_on_disposed_leg: dec!(2000),
            qualified_covered_call_facts: None,
        });
        assert!(straddle.note.contains("§1092(a)(1)"));
    }

    #[test]
    fn holding_period_suspension_only_for_non_qcc_straddle() {
        let qcc = compute(&Section1092Input {
            legs: vec![
                leg("LONG", dec!(15000), dec!(10000), false),
                leg("SHORT_CALL", dec!(0), dec!(2000), true),
            ],
            realized_loss_on_disposed_leg: dec!(2000),
            qualified_covered_call_facts: Some(QualifiedCoveredCallFacts {
                publicly_traded_underlying: true,
                days_to_expiration_at_writing: 60,
                strike_above_lowest_qualified_benchmark: true,
            }),
        });
        assert!(!qcc.holding_period_suspended);

        let straddle = compute(&Section1092Input {
            legs: vec![
                leg("LONG", dec!(15000), dec!(10000), false),
                leg("SHORT_CALL", dec!(0), dec!(2000), true),
            ],
            realized_loss_on_disposed_leg: dec!(2000),
            qualified_covered_call_facts: None,
        });
        assert!(straddle.holding_period_suspended);
    }

    #[test]
    fn empty_legs_no_op() {
        let r = compute(&Section1092Input {
            legs: vec![],
            realized_loss_on_disposed_leg: Decimal::ZERO,
            qualified_covered_call_facts: None,
        });
        assert_eq!(r.loss_deferred, Decimal::ZERO);
        assert_eq!(r.loss_recognized_this_year, Decimal::ZERO);
    }

    #[test]
    fn qcc_short_circuit_runs_before_offsetting_gain_calculation() {
        // Even if there's a huge offsetting gain that would normally
        // defer the loss, QCC bypasses §1092 entirely.
        let r = compute(&Section1092Input {
            legs: vec![
                leg("LONG", dec!(100000), dec!(10000), false), // +$90k
                leg("SHORT_CALL", dec!(0), dec!(2000), true),
            ],
            realized_loss_on_disposed_leg: dec!(2000),
            qualified_covered_call_facts: Some(QualifiedCoveredCallFacts {
                publicly_traded_underlying: true,
                days_to_expiration_at_writing: 60,
                strike_above_lowest_qualified_benchmark: true,
            }),
        });
        assert!(r.is_qualified_covered_call);
        assert_eq!(r.loss_recognized_this_year, dec!(2000));
        // unrecognized_gain_on_offsetting_legs not computed in QCC path.
    }
}
