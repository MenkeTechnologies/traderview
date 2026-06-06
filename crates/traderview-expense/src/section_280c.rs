//! IRC § 280C — Certain Expenses for Which Credits Are Allowable.
//!
//! Pure-compute disallowance check that prevents a taxpayer from
//! claiming BOTH a federal tax credit AND a § 162 / § 174 deduction
//! for the SAME dollar of wages or research expense. Anti-double-
//! dip rule: the deduction must be reduced by the amount of the
//! credit determined for the taxable year, UNLESS the taxpayer
//! makes an irrevocable election to take a reduced credit and keep
//! the full deduction.
//!
//! Statute (verbatim mapping):
//! - § 280C(a) — GENERAL RULE for wage-based credits: no deduction
//!   for that portion of wages or salaries paid or incurred during
//!   the taxable year which is equal to the sum of the credits
//!   determined under §§ 45A(a) (Indian Employment Credit), 45P(a)
//!   (Military Differential Wage Payment Credit), 45S(a) (Paid
//!   Family and Medical Leave Credit), 51(a) (Work Opportunity
//!   Tax Credit), and 1396(a) (Empowerment Zone Employment Credit).
//! - § 280C(b) — CLINICAL TESTING EXPENSE CREDIT (Orphan Drug
//!   Credit under § 45C): § 280C(b)(1) reduce § 174 / § 162
//!   deduction by full credit; § 280C(b)(3) reduced-credit election
//!   (credit × (1 − 21 %)) preserves full deduction; election must
//!   be on the original return for the taxable year per § 280C(b)(3)
//!   cross-reference to § 280C(c)(2)/(3) procedure.
//! - § 280C(c) — RESEARCH CREDIT (§ 41): § 280C(c)(1) default rule
//!   requires taxpayer to reduce § 174 specified research or
//!   experimental expenditures (post-TCJA) by the § 41 credit
//!   amount; § 280C(c)(2) lets taxpayer elect a REDUCED CREDIT
//!   equal to gross credit × (1 − maximum corporate rate of 21 %),
//!   in which case § 174 deduction is NOT reduced; § 280C(c)(3)
//!   ELECTION must be made on the ORIGINAL RETURN for the taxable
//!   year — election cannot be made on amended return.
//! - § 280C(g) — DIFFERENTIAL WAGE PAYMENT CREDIT (§ 45P): wage
//!   deduction reduced by § 45P credit determined; election under
//!   § 280C(c)(2) procedure available for § 45P.
//! - § 280C(h) — PAID FAMILY AND MEDICAL LEAVE CREDIT (§ 45S):
//!   wage deduction reduced by § 45S(1)(A) credit amount.
//!
//! Trader-landlord critical for: WOTC (hiring veterans, ex-felons,
//! long-term unemployed); R&D credit (software development for
//! trading systems, broker platforms); orphan drug credit (pharma-
//! adjacent investments); empowerment zone employment credit
//! (rentals in designated empowerment zones — historically expired
//! 2025-12-31 but periodically extended); paid family medical leave
//! credit (employer-provided FMLA for property management staff).
//!
//! Web research 2026-06-03:
//! - TCJA amended § 280C(c) to align with § 174 capitalization;
//!   post-2021 R&D expenses are NOT immediately deductible but
//!   amortized over 5 years (60-month) domestic / 15 years (180-
//!   month) foreign — § 280C(c)(1) reduces THIS amortizable basis
//!   by credit amount.
//! - § 280C(b)(3) reduced-credit election for § 45C orphan drug
//!   uses § 280C(c)(2) procedure cross-reference per Bloomberg Tax
//!   IRC publication.
//! - Tax Court has held § 280C disallowance applies to credit
//!   "determined" rather than credit "allowed" — taxpayer cannot
//!   avoid disallowance by limiting credit utilization (e.g.,
//!   credit blocked by § 38 general business credit limitation).

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

/// § 280C(c)(2) reduced-credit election factor (corporate maximum
/// rate of 21 % post-TCJA). Stored × 10 000 for integer math.
pub const SECTION_280C_REDUCED_CREDIT_FACTOR_BASIS_POINTS: u64 = 7_900;

/// § 280C(c)(2) maximum corporate rate basis (21 % post-TCJA).
pub const SECTION_280C_MAXIMUM_CORPORATE_RATE_BASIS_POINTS: u64 = 2_100;

/// Common denominator for basis-point fractions used in § 280C math.
pub const SECTION_280C_BASIS_POINT_DENOMINATOR: u64 = 10_000;

/// Year the § 280C(c) research credit reduced-credit election was
/// added (Energy Tax Incentives Act of 2005 restructured § 280C(c)
/// to current 21 %-based formula; election existed since OBRA 1989).
pub const SECTION_280C_RESEARCH_REDUCED_CREDIT_ELECTION_ENACTED_YEAR: u32 = 1989;

/// Tax-credit category invoking § 280C disallowance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CreditCategory {
    ResearchSection41,
    WorkOpportunitySection51,
    IndianEmploymentSection45A,
    OrphanDrugSection45C,
    MilitaryDifferentialWageSection45P,
    PaidFamilyMedicalLeaveSection45S,
    EmpowermentZoneEmploymentSection1396,
}

/// Whether the taxpayer made the irrevocable § 280C(c)(2) reduced-
/// credit election on the original return.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReducedCreditElection {
    NotElected,
    ElectedOnOriginalReturn,
    AttemptedOnAmendedReturn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section280cMode {
    NotApplicable,
    CompliantNoCreditClaimed,
    CompliantFullDeductionDisallowanceApplied,
    CompliantReducedCreditElectionAppliedResearchOrOrphan,
    ViolationDeductionAndFullCreditDoubleClaimed,
    ViolationReducedCreditElectionNotOnOriginalReturn,
    ViolationDeductionDisallowanceLessThanCreditDetermined,
    ViolationReducedCreditElectionAttemptedForIneligibleCredit,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub credit: CreditCategory,
    pub gross_credit_determined_cents: u64,
    pub wages_or_research_expense_cents: u64,
    pub deduction_claimed_cents: u64,
    pub reduced_credit_election: ReducedCreditElection,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section280cMode,
    pub credit_after_section_280c_cents: u64,
    pub allowed_deduction_cents: u64,
    pub disallowance_amount_cents: u64,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type Section280cInput = Input;
pub type Section280cOutput = Output;
pub type Section280cResult = Output;

/// Returns true when credit category permits § 280C(c)(2)-style
/// reduced-credit election (research § 41 and orphan drug § 45C).
/// Wage-based credits in § 280C(a) do NOT permit the reduced-credit
/// election; disallowance is mandatory.
fn permits_reduced_credit_election(credit: CreditCategory) -> bool {
    matches!(
        credit,
        CreditCategory::ResearchSection41 | CreditCategory::OrphanDrugSection45C
    )
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "26 U.S.C. § 280C(a) — wage credits: § 45A Indian Employment, § 45P Military Differential Wage, § 45S Paid Family Medical Leave, § 51 WOTC, § 1396 Empowerment Zone — no deduction equal to credit determined".to_string(),
        "26 U.S.C. § 280C(b)(1) — orphan drug § 45C clinical testing expense: § 162/§ 174 deduction reduced by credit determined".to_string(),
        "26 U.S.C. § 280C(b)(3) — orphan drug reduced-credit election cross-references § 280C(c)(2)/(3) procedure".to_string(),
        "26 U.S.C. § 280C(c)(1) — § 41 research credit: § 174 specified R&E deduction reduced by credit determined (default rule)".to_string(),
        "26 U.S.C. § 280C(c)(2) — § 41 reduced-credit election: credit × (1 − 21 %) and taxpayer keeps full § 174 deduction".to_string(),
        "26 U.S.C. § 280C(c)(3) — § 280C(c)(2) election must be on ORIGINAL RETURN; cannot be made on amended return".to_string(),
        "26 U.S.C. § 280C(g) — § 45P military differential wage payment credit: wage deduction disallowance".to_string(),
        "26 U.S.C. § 280C(h) — § 45S paid family and medical leave credit: wage deduction disallowance".to_string(),
        "Tax Court — § 280C disallowance applies to credit DETERMINED, not credit ALLOWED (cannot escape by § 38 limitation)".to_string(),
        "Treas. Reg. § 1.280C-4 — § 41 research credit reduced-credit election procedure".to_string(),
    ];

    if input.gross_credit_determined_cents == 0 {
        return Output {
            mode: Section280cMode::CompliantNoCreditClaimed,
            credit_after_section_280c_cents: 0,
            allowed_deduction_cents: input.deduction_claimed_cents,
            disallowance_amount_cents: 0,
            notes: format!(
                "No § 280C anti-double-dip required: gross credit = 0 cents. Wages/R&E = {} cents. Deduction claimed = {} cents allowed in full.",
                input.wages_or_research_expense_cents, input.deduction_claimed_cents
            ),
            citations,
        };
    }

    let permits_election = permits_reduced_credit_election(input.credit);

    if input.reduced_credit_election != ReducedCreditElection::NotElected && !permits_election {
        return Output {
            mode: Section280cMode::ViolationReducedCreditElectionAttemptedForIneligibleCredit,
            credit_after_section_280c_cents: input.gross_credit_determined_cents,
            allowed_deduction_cents: input.deduction_claimed_cents.saturating_sub(input.gross_credit_determined_cents),
            disallowance_amount_cents: input.gross_credit_determined_cents,
            notes: format!(
                "VIOLATION: § 280C(c)(2)-style reduced-credit election attempted for credit {:?} that does NOT permit the election. Only § 41 research credit and § 45C orphan drug credit permit reduced-credit election. § 280C(a) wage credits (WOTC, Indian Employment, Empowerment Zone, etc.) require mandatory deduction disallowance equal to credit determined.",
                input.credit
            ),
            citations,
        };
    }

    if input.reduced_credit_election == ReducedCreditElection::AttemptedOnAmendedReturn {
        return Output {
            mode: Section280cMode::ViolationReducedCreditElectionNotOnOriginalReturn,
            credit_after_section_280c_cents: input.gross_credit_determined_cents,
            allowed_deduction_cents: input.deduction_claimed_cents.saturating_sub(input.gross_credit_determined_cents),
            disallowance_amount_cents: input.gross_credit_determined_cents,
            notes: format!(
                "VIOLATION § 280C(c)(3): reduced-credit election attempted on amended return is INVALID. Election must be made on the original return for the taxable year. Default § 280C(c)(1) rule applies: full credit {} cents allowed AND § 174 deduction reduced by credit amount {} cents.",
                input.gross_credit_determined_cents, input.gross_credit_determined_cents
            ),
            citations,
        };
    }

    if input.reduced_credit_election == ReducedCreditElection::ElectedOnOriginalReturn {
        let reduced_credit_cents = (input.gross_credit_determined_cents as u128)
            .saturating_mul(SECTION_280C_REDUCED_CREDIT_FACTOR_BASIS_POINTS as u128)
            .checked_div(SECTION_280C_BASIS_POINT_DENOMINATOR as u128)
            .unwrap_or(0) as u64;
        return Output {
            mode: Section280cMode::CompliantReducedCreditElectionAppliedResearchOrOrphan,
            credit_after_section_280c_cents: reduced_credit_cents,
            allowed_deduction_cents: input.deduction_claimed_cents,
            disallowance_amount_cents: 0,
            notes: format!(
                "COMPLIANT § 280C(c)(2) reduced-credit election made on original return: gross credit {} cents × (1 − 21 %) = {} cents. Full § 174 deduction of {} cents preserved (no § 280C(c)(1) deduction reduction).",
                input.gross_credit_determined_cents, reduced_credit_cents, input.deduction_claimed_cents
            ),
            citations,
        };
    }

    let required_disallowance_cents = input.gross_credit_determined_cents;
    if input.deduction_claimed_cents
        > input
            .wages_or_research_expense_cents
            .saturating_sub(required_disallowance_cents)
    {
        let actual_disallowance_cents = input
            .wages_or_research_expense_cents
            .saturating_sub(input.deduction_claimed_cents);
        let shortfall_cents = required_disallowance_cents.saturating_sub(actual_disallowance_cents);
        if input.deduction_claimed_cents == input.wages_or_research_expense_cents
            && shortfall_cents > 0
        {
            return Output {
                mode: Section280cMode::ViolationDeductionAndFullCreditDoubleClaimed,
                credit_after_section_280c_cents: input.gross_credit_determined_cents,
                allowed_deduction_cents: input.wages_or_research_expense_cents.saturating_sub(required_disallowance_cents),
                disallowance_amount_cents: required_disallowance_cents,
                notes: format!(
                    "VIOLATION § 280C(a)/(b)/(c): taxpayer claimed BOTH the full credit of {} cents AND the full deduction of {} cents on the same wages/R&E. Double-dip prohibited. Deduction must be reduced by {} cents; allowed deduction = {} cents.",
                    input.gross_credit_determined_cents,
                    input.deduction_claimed_cents,
                    required_disallowance_cents,
                    input.wages_or_research_expense_cents.saturating_sub(required_disallowance_cents)
                ),
                citations,
            };
        }
        return Output {
            mode: Section280cMode::ViolationDeductionDisallowanceLessThanCreditDetermined,
            credit_after_section_280c_cents: input.gross_credit_determined_cents,
            allowed_deduction_cents: input.wages_or_research_expense_cents.saturating_sub(required_disallowance_cents),
            disallowance_amount_cents: required_disallowance_cents,
            notes: format!(
                "VIOLATION § 280C: deduction disallowance of {} cents is LESS than credit determined of {} cents (shortfall {} cents). § 280C applies to credit DETERMINED, not credit ALLOWED — taxpayer cannot reduce disallowance by limiting credit utilization. Required allowed deduction = {} cents.",
                actual_disallowance_cents,
                required_disallowance_cents,
                shortfall_cents,
                input.wages_or_research_expense_cents.saturating_sub(required_disallowance_cents)
            ),
            citations,
        };
    }

    Output {
        mode: Section280cMode::CompliantFullDeductionDisallowanceApplied,
        credit_after_section_280c_cents: input.gross_credit_determined_cents,
        allowed_deduction_cents: input.deduction_claimed_cents,
        disallowance_amount_cents: required_disallowance_cents,
        notes: format!(
            "COMPLIANT § 280C: full credit {} cents claimed; § 162/§ 174 deduction reduced by full credit amount. Allowed deduction = {} cents (wages/R&E {} cents minus disallowance {} cents).",
            input.gross_credit_determined_cents,
            input.deduction_claimed_cents,
            input.wages_or_research_expense_cents,
            required_disallowance_cents
        ),
        citations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_wotc() -> Input {
        Input {
            credit: CreditCategory::WorkOpportunitySection51,
            gross_credit_determined_cents: 240_000,
            wages_or_research_expense_cents: 1_000_000,
            deduction_claimed_cents: 760_000,
            reduced_credit_election: ReducedCreditElection::NotElected,
        }
    }

    #[test]
    fn no_credit_claimed_is_compliant_and_preserves_full_deduction() {
        let mut input = baseline_wotc();
        input.gross_credit_determined_cents = 0;
        input.deduction_claimed_cents = 1_000_000;
        let result = compute(&input);
        assert_eq!(result.mode, Section280cMode::CompliantNoCreditClaimed);
        assert_eq!(result.allowed_deduction_cents, 1_000_000);
        assert_eq!(result.disallowance_amount_cents, 0);
    }

    #[test]
    fn wotc_full_disallowance_applied_compliant() {
        let result = compute(&baseline_wotc());
        assert_eq!(
            result.mode,
            Section280cMode::CompliantFullDeductionDisallowanceApplied
        );
        assert_eq!(result.disallowance_amount_cents, 240_000);
        assert_eq!(result.allowed_deduction_cents, 760_000);
    }

    #[test]
    fn wotc_double_dip_violation_detected() {
        let mut input = baseline_wotc();
        input.deduction_claimed_cents = 1_000_000;
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section280cMode::ViolationDeductionAndFullCreditDoubleClaimed
        );
        assert_eq!(result.allowed_deduction_cents, 760_000);
    }

    #[test]
    fn research_reduced_credit_election_on_original_return_compliant() {
        let input = Input {
            credit: CreditCategory::ResearchSection41,
            gross_credit_determined_cents: 1_000_000,
            wages_or_research_expense_cents: 5_000_000,
            deduction_claimed_cents: 5_000_000,
            reduced_credit_election: ReducedCreditElection::ElectedOnOriginalReturn,
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section280cMode::CompliantReducedCreditElectionAppliedResearchOrOrphan
        );
        assert_eq!(result.credit_after_section_280c_cents, 790_000);
        assert_eq!(result.allowed_deduction_cents, 5_000_000);
        assert_eq!(result.disallowance_amount_cents, 0);
    }

    #[test]
    fn orphan_drug_reduced_credit_election_on_original_return_compliant() {
        let input = Input {
            credit: CreditCategory::OrphanDrugSection45C,
            gross_credit_determined_cents: 500_000,
            wages_or_research_expense_cents: 2_000_000,
            deduction_claimed_cents: 2_000_000,
            reduced_credit_election: ReducedCreditElection::ElectedOnOriginalReturn,
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section280cMode::CompliantReducedCreditElectionAppliedResearchOrOrphan
        );
        assert_eq!(result.credit_after_section_280c_cents, 395_000);
        assert_eq!(result.allowed_deduction_cents, 2_000_000);
    }

    #[test]
    fn research_reduced_credit_election_on_amended_return_violation() {
        let input = Input {
            credit: CreditCategory::ResearchSection41,
            gross_credit_determined_cents: 1_000_000,
            wages_or_research_expense_cents: 5_000_000,
            deduction_claimed_cents: 5_000_000,
            reduced_credit_election: ReducedCreditElection::AttemptedOnAmendedReturn,
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section280cMode::ViolationReducedCreditElectionNotOnOriginalReturn
        );
        assert!(result.notes.contains("§ 280C(c)(3)"));
        assert!(result.notes.contains("original return"));
    }

    #[test]
    fn wotc_reduced_credit_election_attempted_violation() {
        let input = Input {
            credit: CreditCategory::WorkOpportunitySection51,
            gross_credit_determined_cents: 240_000,
            wages_or_research_expense_cents: 1_000_000,
            deduction_claimed_cents: 760_000,
            reduced_credit_election: ReducedCreditElection::ElectedOnOriginalReturn,
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section280cMode::ViolationReducedCreditElectionAttemptedForIneligibleCredit
        );
        assert!(result.notes.contains("§ 280C(a) wage credits"));
    }

    #[test]
    fn indian_employment_reduced_credit_election_violation() {
        let input = Input {
            credit: CreditCategory::IndianEmploymentSection45A,
            gross_credit_determined_cents: 100_000,
            wages_or_research_expense_cents: 500_000,
            deduction_claimed_cents: 400_000,
            reduced_credit_election: ReducedCreditElection::ElectedOnOriginalReturn,
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section280cMode::ViolationReducedCreditElectionAttemptedForIneligibleCredit
        );
    }

    #[test]
    fn military_differential_wage_full_disallowance_compliant() {
        let input = Input {
            credit: CreditCategory::MilitaryDifferentialWageSection45P,
            gross_credit_determined_cents: 30_000,
            wages_or_research_expense_cents: 150_000,
            deduction_claimed_cents: 120_000,
            reduced_credit_election: ReducedCreditElection::NotElected,
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section280cMode::CompliantFullDeductionDisallowanceApplied
        );
        assert_eq!(result.disallowance_amount_cents, 30_000);
    }

    #[test]
    fn paid_family_medical_leave_full_disallowance_compliant() {
        let input = Input {
            credit: CreditCategory::PaidFamilyMedicalLeaveSection45S,
            gross_credit_determined_cents: 12_500,
            wages_or_research_expense_cents: 100_000,
            deduction_claimed_cents: 87_500,
            reduced_credit_election: ReducedCreditElection::NotElected,
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section280cMode::CompliantFullDeductionDisallowanceApplied
        );
        assert_eq!(result.disallowance_amount_cents, 12_500);
        assert_eq!(result.allowed_deduction_cents, 87_500);
    }

    #[test]
    fn empowerment_zone_full_disallowance_compliant() {
        let input = Input {
            credit: CreditCategory::EmpowermentZoneEmploymentSection1396,
            gross_credit_determined_cents: 60_000,
            wages_or_research_expense_cents: 300_000,
            deduction_claimed_cents: 240_000,
            reduced_credit_election: ReducedCreditElection::NotElected,
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section280cMode::CompliantFullDeductionDisallowanceApplied
        );
        assert_eq!(result.disallowance_amount_cents, 60_000);
    }

    #[test]
    fn research_default_rule_full_disallowance_compliant() {
        let input = Input {
            credit: CreditCategory::ResearchSection41,
            gross_credit_determined_cents: 1_000_000,
            wages_or_research_expense_cents: 5_000_000,
            deduction_claimed_cents: 4_000_000,
            reduced_credit_election: ReducedCreditElection::NotElected,
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section280cMode::CompliantFullDeductionDisallowanceApplied
        );
        assert_eq!(result.disallowance_amount_cents, 1_000_000);
        assert_eq!(result.allowed_deduction_cents, 4_000_000);
    }

    #[test]
    fn partial_disallowance_short_of_credit_determined_violation() {
        let input = Input {
            credit: CreditCategory::WorkOpportunitySection51,
            gross_credit_determined_cents: 100_000,
            wages_or_research_expense_cents: 500_000,
            deduction_claimed_cents: 450_000,
            reduced_credit_election: ReducedCreditElection::NotElected,
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section280cMode::ViolationDeductionDisallowanceLessThanCreditDetermined
        );
        assert!(result.notes.contains("DETERMINED"));
    }

    #[test]
    fn citations_pin_section_280c_subsections() {
        let result = compute(&baseline_wotc());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("§ 280C(a)"));
        assert!(joined.contains("§ 280C(b)(1)"));
        assert!(joined.contains("§ 280C(b)(3)"));
        assert!(joined.contains("§ 280C(c)(1)"));
        assert!(joined.contains("§ 280C(c)(2)"));
        assert!(joined.contains("§ 280C(c)(3)"));
        assert!(joined.contains("§ 280C(g)"));
        assert!(joined.contains("§ 280C(h)"));
    }

    #[test]
    fn citations_pin_referenced_credit_sections() {
        let result = compute(&baseline_wotc());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("§ 45A Indian Employment"));
        assert!(joined.contains("§ 45P Military Differential Wage"));
        assert!(joined.contains("§ 45S Paid Family Medical Leave"));
        assert!(joined.contains("§ 51 WOTC"));
        assert!(joined.contains("§ 1396 Empowerment Zone"));
    }

    #[test]
    fn constant_pin_21_percent_reduced_credit_factor() {
        assert_eq!(SECTION_280C_REDUCED_CREDIT_FACTOR_BASIS_POINTS, 7_900);
        assert_eq!(SECTION_280C_MAXIMUM_CORPORATE_RATE_BASIS_POINTS, 2_100);
        assert_eq!(SECTION_280C_BASIS_POINT_DENOMINATOR, 10_000);
        assert_eq!(
            SECTION_280C_REDUCED_CREDIT_FACTOR_BASIS_POINTS
                + SECTION_280C_MAXIMUM_CORPORATE_RATE_BASIS_POINTS,
            SECTION_280C_BASIS_POINT_DENOMINATOR
        );
    }

    #[test]
    fn constant_pin_1989_research_reduced_credit_election_year() {
        assert_eq!(
            SECTION_280C_RESEARCH_REDUCED_CREDIT_ELECTION_ENACTED_YEAR,
            1989
        );
    }

    #[test]
    fn saturating_overflow_defense_extreme_inputs() {
        let input = Input {
            credit: CreditCategory::ResearchSection41,
            gross_credit_determined_cents: u64::MAX,
            wages_or_research_expense_cents: u64::MAX,
            deduction_claimed_cents: 0,
            reduced_credit_election: ReducedCreditElection::ElectedOnOriginalReturn,
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section280cMode::CompliantReducedCreditElectionAppliedResearchOrOrphan
        );
    }
}
