//! Post-judgment wage garnishment for tenant rent debt — landlord
//! compliance check on whether a money judgment for unpaid rent +
//! damages can be enforced by garnishing the former tenant's wages
//! at the employer level.
//!
//! Trader-landlord operational concern after eviction proceedings
//! conclude with a money judgment. Wage garnishment is one of the
//! highest-yield post-judgment enforcement tools (continuous, 25%
//! of weekly disposable earnings under federal floor), but FOUR
//! STATES — Texas, North Carolina, Pennsylvania, South Carolina —
//! absolutely prohibit wage garnishment for civil judgments,
//! making this enforcement pathway unavailable regardless of how
//! large the judgment is. Trader-landlords operating in those
//! states should pivot to other enforcement tools (bank-account
//! levy, lien, property seizure).
//!
//! Distinct from `damage_deduction_itemization` (which addresses
//! security deposit deductions at move-out), `rent_credit_
//! reporting` (which addresses positive rent reporting to consumer
//! reporting agencies), and `prevailing_party_attorney_fees`
//! (lease attorney-fee shifting). This module addresses ONLY the
//! POST-MONEY-JUDGMENT wage garnishment enforcement pathway.
//!
//! Three regimes:
//!
//! **Fully prohibited — TX, NC, PA, SC**. Four states absolutely
//! prohibit wage garnishment for civil-debt judgments by private
//! creditors. Tex. Const. art. XVI § 28 (TX); N.C. Const. art. X
//! § 1 + N.C. Gen. Stat. § 1-362 (NC); 42 Pa. Cons. Stat. § 8127
//! (PA); S.C. Code § 37-5-104 (SC). The prohibition does NOT reach
//! federal/state tax garnishment, child support, alimony, or
//! federally backed student loans — those remain garnishable in
//! all states.
//!
//! **State more protective — CA, MA, VA, NY, others**. Federal
//! CCPA floor applies BUT state law provides higher exemptions.
//! California Code Civ. Proc. § 706.050 — exempts 50 × state
//! minimum wage (greater than the federal 30 × federal minimum).
//! Massachusetts G.L. c. 246 § 28 — exempts 50 × state minimum
//! weekly. Virginia § 34-29 — exempts greater of 75% of disposable
//! earnings or 40 × federal minimum wage. Caller supplies the
//! state-specific exemption.
//!
//! **Federal floor — most other states**. CCPA § 303 (15 U.S.C. §
//! 1673(a)(1)) sets garnishment maximum at the LESSER of (i) 25%
//! of weekly disposable earnings or (ii) the amount by which
//! weekly disposable earnings exceed 30 × the federal minimum
//! hourly wage. As of 2026, federal minimum wage is $7.25/hour,
//! making the 30× threshold $217.50/week.
//!
//! Citations: 15 U.S.C. § 1673(a)(1) (federal CCPA wage garnishment
//! ceiling — lesser of 25% disposable or excess over 30× federal
//! minimum); 15 U.S.C. § 1673(a)(2) (50%/55%/60%/65% child support
//! tiers — out of scope of this module); Tex. Const. art. XVI § 28
//! (TX prohibition); N.C. Const. art. X § 1; N.C. Gen. Stat. §
//! 1-362 (NC); 42 Pa. Cons. Stat. § 8127 (PA); S.C. Code § 37-5-104
//! (SC); Cal. Code Civ. Proc. § 706.050 (CA 50× state min);
//! Mass. G.L. c. 246 § 28 (MA 50× state min); Va. Code § 34-29
//! (VA 75% / 40× federal min); DOL Fact Sheet 30 (CCPA wage
//! garnishment protections).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StateRegime {
    /// TX, NC, PA, SC — civil-debt wage garnishment fully prohibited.
    FullyProhibited,
    /// State exemption more protective than federal CCPA floor
    /// (CA, MA, VA, etc.). Caller supplies the state-specific
    /// minimum hourly wage in `state_minimum_wage_cents_per_hour`
    /// and weekly multiplier in `state_exemption_multiplier_weeks`.
    StateMoreProtective,
    /// Most states — federal CCPA floor applies (lesser of 25%
    /// or excess over 30× federal minimum).
    FederalFloor,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JudgmentType {
    /// Ordinary civil debt (unpaid rent, damages, etc.). Subject
    /// to FullyProhibited state prohibitions.
    CivilDebt,
    /// Child support / alimony — different CCPA tier (50-65%);
    /// NOT subject to four-state prohibitions.
    ChildSupportOrAlimony,
    /// Federal or state tax debt — NOT subject to CCPA limits and
    /// NOT subject to four-state prohibitions.
    TaxDebt,
    /// Federally backed student loan — NOT subject to four-state
    /// prohibitions; 15% federal administrative wage garnishment
    /// ceiling.
    FederallyBackedStudentLoan,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GarnishmentInput {
    pub state_regime: StateRegime,
    pub judgment_type: JudgmentType,
    /// Weekly disposable earnings (after legally required
    /// withholdings — § 1672(b) definition).
    pub weekly_disposable_earnings_cents: i64,
    /// Federal minimum wage in cents/hour. $7.25/hour = 725 cents.
    pub federal_minimum_wage_cents_per_hour: i64,
    /// State minimum wage in cents/hour (used by StateMoreProtective
    /// regime).
    pub state_minimum_wage_cents_per_hour: i64,
    /// State exemption multiplier in weekly hours (e.g., 50 for CA
    /// "50 × state minimum wage", 40 for VA "40 × federal minimum"
    /// when used as the federal-min variant). Used by
    /// StateMoreProtective regime.
    pub state_exemption_multiplier_hours: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct GarnishmentResult {
    pub garnishment_allowed: bool,
    pub max_weekly_garnishment_cents: i64,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn compute(input: &GarnishmentInput) -> GarnishmentResult {
    let mut notes: Vec<String> = Vec::new();

    if !matches!(input.judgment_type, JudgmentType::CivilDebt) {
        match input.judgment_type {
            JudgmentType::ChildSupportOrAlimony => {
                notes.push(
                    "15 U.S.C. § 1673(a)(2) — child support / alimony tier (50%-65% depending on dependents and arrears) — bypasses civil-debt prohibitions; out of scope of this module"
                        .to_string(),
                );
            }
            JudgmentType::TaxDebt => {
                notes.push(
                    "federal/state tax debt — not subject to CCPA limits or four-state civil-debt prohibitions"
                        .to_string(),
                );
            }
            JudgmentType::FederallyBackedStudentLoan => {
                notes.push(
                    "federally backed student loan — 15% Department of Education administrative wage garnishment ceiling; not subject to four-state prohibitions"
                        .to_string(),
                );
            }
            _ => {}
        }
        return GarnishmentResult {
            garnishment_allowed: true,
            max_weekly_garnishment_cents: 0,
            citation: citation(),
            notes,
        };
    }

    match input.state_regime {
        StateRegime::FullyProhibited => {
            notes.push(
                "TX, NC, PA, SC — civil-debt wage garnishment ABSOLUTELY PROHIBITED at the employer level (Tex. Const. art. XVI § 28; N.C. Const. art. X § 1 + N.C. Gen. Stat. § 1-362; 42 Pa. Cons. Stat. § 8127; S.C. Code § 37-5-104)"
                    .to_string(),
            );
            notes.push(
                "bank-account levy may remain available after wages deposited — prohibition operates only at the employer, not the bank"
                    .to_string(),
            );
            GarnishmentResult {
                garnishment_allowed: false,
                max_weekly_garnishment_cents: 0,
                citation: citation(),
                notes,
            }
        }
        StateRegime::FederalFloor => {
            let twenty_five_pct = input.weekly_disposable_earnings_cents.saturating_mul(25) / 100;
            let thirty_x_min = input
                .federal_minimum_wage_cents_per_hour
                .saturating_mul(30);
            let excess_over_30x = input
                .weekly_disposable_earnings_cents
                .saturating_sub(thirty_x_min)
                .max(0);
            let max = twenty_five_pct.min(excess_over_30x);

            notes.push(format!(
                "15 U.S.C. § 1673(a)(1) — federal CCPA floor: lesser of 25% disposable ({} cents) or excess over 30× federal minimum ({} cents) = {} cents",
                twenty_five_pct, excess_over_30x, max
            ));
            GarnishmentResult {
                garnishment_allowed: max > 0,
                max_weekly_garnishment_cents: max,
                citation: citation(),
                notes,
            }
        }
        StateRegime::StateMoreProtective => {
            let twenty_five_pct = input.weekly_disposable_earnings_cents.saturating_mul(25) / 100;
            let federal_thirty_x = input
                .federal_minimum_wage_cents_per_hour
                .saturating_mul(30);
            let federal_excess = input
                .weekly_disposable_earnings_cents
                .saturating_sub(federal_thirty_x)
                .max(0);
            let federal_max = twenty_five_pct.min(federal_excess);

            let state_multiplier_threshold = input
                .state_minimum_wage_cents_per_hour
                .saturating_mul(input.state_exemption_multiplier_hours as i64);
            let state_excess = input
                .weekly_disposable_earnings_cents
                .saturating_sub(state_multiplier_threshold)
                .max(0);
            let max = federal_max.min(state_excess);

            notes.push(format!(
                "state exemption — {}× state minimum wage = {} cents threshold; max = min(federal CCPA = {} cents, state exemption excess = {} cents) = {} cents",
                input.state_exemption_multiplier_hours, state_multiplier_threshold, federal_max, state_excess, max
            ));
            GarnishmentResult {
                garnishment_allowed: max > 0,
                max_weekly_garnishment_cents: max,
                citation: citation(),
                notes,
            }
        }
    }
}

fn citation() -> &'static str {
    "15 U.S.C. § 1673(a)(1)/(a)(2); Tex. Const. art. XVI § 28; N.C. Const. art. X § 1; N.C. Gen. Stat. § 1-362; 42 Pa. Cons. Stat. § 8127; S.C. Code § 37-5-104; Cal. Code Civ. Proc. § 706.050; Mass. G.L. c. 246 § 28; Va. Code § 34-29; DOL Fact Sheet 30"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base(regime: StateRegime, weekly_disposable_dollars: i64) -> GarnishmentInput {
        GarnishmentInput {
            state_regime: regime,
            judgment_type: JudgmentType::CivilDebt,
            weekly_disposable_earnings_cents: weekly_disposable_dollars * 100,
            federal_minimum_wage_cents_per_hour: 725,
            state_minimum_wage_cents_per_hour: 1600,
            state_exemption_multiplier_hours: 50,
        }
    }

    #[test]
    fn texas_civil_debt_garnishment_prohibited() {
        let r = compute(&base(StateRegime::FullyProhibited, 1_000));
        assert!(!r.garnishment_allowed);
        assert_eq!(r.max_weekly_garnishment_cents, 0);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("ABSOLUTELY PROHIBITED")));
        assert!(r.notes.iter().any(|n| n.contains("Tex. Const. art. XVI § 28")));
    }

    #[test]
    fn fully_prohibited_note_lists_all_four_states() {
        let r = compute(&base(StateRegime::FullyProhibited, 1_000));
        assert!(r.notes.iter().any(|n| n.contains("TX, NC, PA, SC")));
    }

    #[test]
    fn fully_prohibited_notes_bank_account_carveout() {
        let r = compute(&base(StateRegime::FullyProhibited, 1_000));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("bank-account levy may remain available")));
    }

    #[test]
    fn federal_floor_high_earner_25_percent_governs() {
        let r = compute(&base(StateRegime::FederalFloor, 1_000));
        let expected_25_pct = 1_000_00 * 25 / 100;
        let expected_excess = 1_000_00 - (725 * 30);
        let expected_max = expected_25_pct.min(expected_excess);
        assert_eq!(r.max_weekly_garnishment_cents, expected_max);
        assert!(r.garnishment_allowed);
    }

    #[test]
    fn federal_floor_at_30x_threshold_zero_garnishment() {
        let mut i = base(StateRegime::FederalFloor, 0);
        i.weekly_disposable_earnings_cents = 725 * 30;
        let r = compute(&i);
        assert_eq!(r.max_weekly_garnishment_cents, 0);
        assert!(!r.garnishment_allowed);
    }

    #[test]
    fn federal_floor_below_30x_zero_garnishment() {
        let mut i = base(StateRegime::FederalFloor, 0);
        i.weekly_disposable_earnings_cents = 725 * 30 - 100;
        let r = compute(&i);
        assert_eq!(r.max_weekly_garnishment_cents, 0);
        assert!(!r.garnishment_allowed);
    }

    #[test]
    fn federal_floor_low_earner_excess_governs() {
        let mut i = base(StateRegime::FederalFloor, 0);
        i.weekly_disposable_earnings_cents = 730 * 30;
        let r = compute(&i);
        let excess = 730 * 30 - 725 * 30;
        let twenty_five_pct = (730 * 30) * 25 / 100;
        let expected = twenty_five_pct.min(excess);
        assert_eq!(r.max_weekly_garnishment_cents, expected);
    }

    #[test]
    fn state_more_protective_california_50x_state_min_governs() {
        let mut i = base(StateRegime::StateMoreProtective, 1_000);
        i.state_minimum_wage_cents_per_hour = 1600;
        i.state_exemption_multiplier_hours = 50;
        let r = compute(&i);
        let federal_max = (1_000_00 * 25 / 100).min(1_000_00 - 725 * 30);
        let state_threshold = 1_600 * 50;
        let state_excess = (1_000_00 - state_threshold).max(0);
        let expected = federal_max.min(state_excess);
        assert_eq!(r.max_weekly_garnishment_cents, expected);
    }

    #[test]
    fn state_more_protective_high_state_min_blocks_garnishment() {
        let mut i = base(StateRegime::StateMoreProtective, 600);
        i.state_minimum_wage_cents_per_hour = 1600;
        i.state_exemption_multiplier_hours = 50;
        let r = compute(&i);
        assert_eq!(r.max_weekly_garnishment_cents, 0);
        assert!(!r.garnishment_allowed);
    }

    #[test]
    fn child_support_bypasses_civil_debt_prohibitions() {
        let mut i = base(StateRegime::FullyProhibited, 1_000);
        i.judgment_type = JudgmentType::ChildSupportOrAlimony;
        let r = compute(&i);
        assert!(r.garnishment_allowed);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("child support") && n.contains("bypasses civil-debt prohibitions")));
    }

    #[test]
    fn tax_debt_bypasses_civil_debt_prohibitions() {
        let mut i = base(StateRegime::FullyProhibited, 1_000);
        i.judgment_type = JudgmentType::TaxDebt;
        let r = compute(&i);
        assert!(r.garnishment_allowed);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("federal/state tax debt") && n.contains("not subject")));
    }

    #[test]
    fn student_loan_bypasses_civil_debt_prohibitions() {
        let mut i = base(StateRegime::FullyProhibited, 1_000);
        i.judgment_type = JudgmentType::FederallyBackedStudentLoan;
        let r = compute(&i);
        assert!(r.garnishment_allowed);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("15%") && n.contains("administrative wage garnishment")));
    }

    #[test]
    fn citation_pins_federal_statute_and_four_state_authorities() {
        let r = compute(&base(StateRegime::FederalFloor, 1_000));
        assert!(r.citation.contains("15 U.S.C. § 1673"));
        assert!(r.citation.contains("Tex. Const. art. XVI § 28"));
        assert!(r.citation.contains("N.C. Const. art. X § 1"));
        assert!(r.citation.contains("N.C. Gen. Stat. § 1-362"));
        assert!(r.citation.contains("42 Pa. Cons. Stat. § 8127"));
        assert!(r.citation.contains("S.C. Code § 37-5-104"));
        assert!(r.citation.contains("Cal. Code Civ. Proc. § 706.050"));
        assert!(r.citation.contains("Mass. G.L. c. 246 § 28"));
        assert!(r.citation.contains("Va. Code § 34-29"));
        assert!(r.citation.contains("DOL Fact Sheet 30"));
    }

    #[test]
    fn federal_floor_note_describes_lesser_of_calculation() {
        let r = compute(&base(StateRegime::FederalFloor, 1_000));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("15 U.S.C. § 1673(a)(1)") && n.contains("lesser of 25%")));
    }

    #[test]
    fn state_more_protective_note_describes_state_multiplier() {
        let r = compute(&base(StateRegime::StateMoreProtective, 1_000));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("state exemption") && n.contains("50× state minimum")));
    }

    #[test]
    fn only_civil_debt_blocked_in_fully_prohibited_invariant() {
        for jt in [
            JudgmentType::ChildSupportOrAlimony,
            JudgmentType::TaxDebt,
            JudgmentType::FederallyBackedStudentLoan,
        ] {
            let mut i = base(StateRegime::FullyProhibited, 1_000);
            i.judgment_type = jt;
            let r = compute(&i);
            assert!(r.garnishment_allowed, "non-civil judgment type {:?} should not be blocked in fully-prohibited states", jt);
        }
        let i = base(StateRegime::FullyProhibited, 1_000);
        let r = compute(&i);
        assert!(!r.garnishment_allowed, "civil debt blocked");
    }

    #[test]
    fn federal_minimum_725_cents_30x_threshold_pinned() {
        let mut i = base(StateRegime::FederalFloor, 0);
        i.weekly_disposable_earnings_cents = 725 * 30 + 100;
        let r = compute(&i);
        assert_eq!(r.max_weekly_garnishment_cents, 100i64.min((725 * 30 + 100) * 25 / 100));
    }

    #[test]
    fn fully_prohibited_uniquely_zero_garnishment_for_civil_debt_invariant() {
        let r_prohibited = compute(&base(StateRegime::FullyProhibited, 1_000));
        let r_federal = compute(&base(StateRegime::FederalFloor, 1_000));
        let r_state = compute(&base(StateRegime::StateMoreProtective, 1_000));
        assert_eq!(r_prohibited.max_weekly_garnishment_cents, 0);
        assert!(r_federal.max_weekly_garnishment_cents > 0);
        assert!(r_state.max_weekly_garnishment_cents >= 0);
    }

    #[test]
    fn high_earner_state_more_protective_capped_at_25_percent_under_federal_floor() {
        let mut i = base(StateRegime::StateMoreProtective, 10_000);
        i.state_minimum_wage_cents_per_hour = 1600;
        i.state_exemption_multiplier_hours = 50;
        let r = compute(&i);
        let twenty_five = 10_000_00 * 25 / 100;
        assert!(r.max_weekly_garnishment_cents <= twenty_five);
    }

    #[test]
    fn massachusetts_50x_state_min_at_15_dollar_state_wage_750_dollar_exemption() {
        let mut i = base(StateRegime::StateMoreProtective, 800);
        i.state_minimum_wage_cents_per_hour = 1500;
        i.state_exemption_multiplier_hours = 50;
        let r = compute(&i);
        let state_threshold = 1_500 * 50;
        let state_excess = (800 * 100 - state_threshold).max(0);
        let federal_max = (800 * 100 * 25 / 100).min((800 * 100 - 725 * 30).max(0));
        let expected = federal_max.min(state_excess);
        assert_eq!(r.max_weekly_garnishment_cents, expected);
    }

    #[test]
    fn note_for_state_more_protective_includes_min_formula() {
        let r = compute(&base(StateRegime::StateMoreProtective, 1_000));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("min(federal CCPA") && n.contains("state exemption excess")));
    }

    #[test]
    fn empty_disposable_earnings_zero_garnishment_all_regimes() {
        for regime in [StateRegime::FullyProhibited, StateRegime::FederalFloor, StateRegime::StateMoreProtective] {
            let r = compute(&base(regime, 0));
            assert_eq!(r.max_weekly_garnishment_cents, 0);
        }
    }

    #[test]
    fn fully_prohibited_disposable_earnings_irrelevant_to_outcome() {
        for income_dollars in [0, 100, 1_000, 10_000] {
            let r = compute(&base(StateRegime::FullyProhibited, income_dollars));
            assert!(!r.garnishment_allowed);
            assert_eq!(r.max_weekly_garnishment_cents, 0);
        }
    }
}
