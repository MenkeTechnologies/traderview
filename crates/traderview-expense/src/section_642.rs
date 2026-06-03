//! IRC § 642 — Special Rules for Credits and Deductions (Estates
//! and Trusts).
//!
//! Pure-compute trust/estate special-rules check covering the
//! § 642(b) personal exemption ($600 estate / $300 complex trust
//! / $100 simple trust) and the § 642(c) charitable contribution
//! deduction (unlimited from gross taxable income with governing
//! instrument direction). § 642 is the structural complement to
//! § 641 (fiduciary tax imposition) + § 643 (DNI) + § 651 / § 661
//! (simple/complex distribution deductions). Trader-critical
//! because § 642(c) provides an UNLIMITED charitable income tax
//! deduction (no AGI floor like individuals) when the gross-income
//! and governing-instrument requirements are met — single highest-
//! impact charitable planning vehicle for trader-funded
//! charitable lead trusts (CLTs) and CRT decanting.
//!
//! Statute (verbatim mapping):
//! - § 642(a)(1) — FOREIGN TAX CREDIT: § 27 + § 901 cross-
//!   reference for estate/trust.
//! - § 642(a)(2) — DEDUCTION FOR PERSONAL EXEMPTION not allowed
//!   except as provided in § 642(b).
//! - § 642(b) — PERSONAL EXEMPTION:
//!   - § 642(b)(1) — $600 for ESTATE;
//!   - § 642(b)(2)(A) — $300 for COMPLEX TRUST (trust whose
//!     terms require current distribution of all income);
//!   - § 642(b)(2)(B) — $300 for trust required to distribute
//!     all income currently;
//!   - § 642(b)(3) — $100 for SIMPLE TRUST (any other trust).
//!   - **NOT indexed for inflation** — same dollar amounts since
//!     original enactment.
//! - § 642(c) — CHARITABLE CONTRIBUTION DEDUCTION:
//!   - § 642(c)(1) — CURRENT YEAR: estate or nongrantor trust
//!     allowed UNLIMITED deduction for any amount of GROSS INCOME
//!     (interpreted as gross TAXABLE income — excludes tax-exempt
//!     interest) which, pursuant to the terms of the governing
//!     instrument, is during the taxable year paid for a § 170(c)
//!     charitable purpose. NO AGI FLOOR (unlike § 170 individual
//!     percentage limitations).
//!   - § 642(c)(2) — ELECTION FOR FOLLOWING YEAR: amounts paid
//!     in the following taxable year may be ELECTED to be
//!     treated as paid in the current year (analog to § 663(b)
//!     65-day rule for distributions).
//!   - § 642(c)(3) — REMAINDER INTEREST IN PERSONAL RESIDENCE OR
//!     FARM cross-reference to § 170(f)(3).
//!   - § 642(c)(4) — ADJUSTMENTS for tax-exempt interest
//!     allocable to charitable distribution (cannot deduct portion
//!     of charitable distribution attributable to tax-exempt
//!     income).
//! - § 642(d) — NET OPERATING LOSS DEDUCTION cross-reference to
//!   § 172.
//! - § 642(e) — DEDUCTION FOR DEPRECIATION AND DEPLETION:
//!   apportioned between fiduciary and beneficiaries on basis of
//!   trust accounting income allocable to each.
//! - § 642(f) — AMORTIZATION DEDUCTIONS cross-reference to
//!   § 178 and § 169.
//! - § 642(g) — DOUBLE DEDUCTION DISALLOWED: estate cannot claim
//!   both income tax deduction (§ 162 / § 212) AND estate tax
//!   deduction (§ 2053 / § 2054) for the same expense.
//! - § 642(h) — UNUSED LOSS CARRYOVERS AND EXCESS DEDUCTIONS ON
//!   TERMINATION: pass through to beneficiaries upon termination
//!   of estate / trust.
//! - § 642(i) — CERTAIN REVERSIONARY INTERESTS — § 673 grantor
//!   trust cross-reference.
//!
//! Web research (verified 2026-06-03):
//! - Cornell LII § 642 confirms statutory text.
//! - ACTEC Foundation Charitable Deductions and § 642(c) podcast
//!   confirms two-requirement rule (gross income + governing
//!   instrument).
//! - The Tax Adviser "Charitable income tax deductions for trusts
//!   and estates" confirms unlimited deduction + § 642(c)(2)
//!   following-year election.
//! - Treas. Reg. § 1.642(c)-3 — adjustments for unlimited
//!   charitable contributions deduction (§ 642(c)(4) implementing
//!   regs).
//! - Greenleaf Trust "Comparing Charitable Deductions" confirms
//!   tax-exempt income exclusion from § 642(c) gross income.
//! - OBBBA 2025 Pease-limitation reinstatement analysis: whether
//!   it applies to § 642(c) trust/estate charitable deduction
//!   under analysis as of 2026.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const SECTION_642_B_ESTATE_EXEMPTION_DOLLARS: u64 = 600;
pub const SECTION_642_B_COMPLEX_TRUST_EXEMPTION_DOLLARS: u64 = 300;
pub const SECTION_642_B_SIMPLE_TRUST_EXEMPTION_DOLLARS: u64 = 100;
pub const SECTION_642_C_AGI_FLOOR_BASIS_POINTS: u64 = 0;
pub const SECTION_642_C_UNLIMITED_DEDUCTION: bool = true;
pub const OBBBA_2025_PEASE_LIMITATION_REINSTATEMENT_YEAR: u32 = 2026;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FiduciaryEntityType {
    DomesticEstateInProbate,
    DomesticSimpleTrust,
    DomesticComplexTrust,
    ElectingSmallBusinessTrust,
    DomesticGrantorTrustPassThrough,
    ForeignTrust,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section642Provision {
    Section642bPersonalExemption,
    Section642cCharitableContribution,
    Section642cFollowingYearElection,
    Section642eDepreciationApportionment,
    Section642gDoubleDeductionDisallowed,
    Section642hExcessDeductionsTermination,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CharitableSourceFunds {
    GrossTaxableIncomeProperSource,
    CorpusOrPrincipal,
    TaxExemptInterestIncluded,
    GovernigInstrumentSilentNoDirection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section642Mode {
    NotApplicableNoEstateOrTrust,
    NotApplicableGrantorTrustPassThrough,
    CompliantSection642bPersonalExemptionAtCorrectLevel,
    CompliantSection642cCharitableUnlimitedDeductionFromGrossIncome,
    CompliantSection642cFollowingYearElectionApplied,
    CompliantSection642eDepreciationAllocatedToBeneficiaries,
    CompliantSection642hExcessDeductionsPassedToBeneficiaryOnTermination,
    ViolationSection642bExemptionOverclaimed,
    ViolationSection642cPaidFromCorpusNotGrossIncome,
    ViolationSection642cNoGoverningInstrumentDirection,
    ViolationSection642cTaxExemptIncomeIncluded,
    ViolationSection642gDoubleDeductionEstateAndIncomeTax,
    ViolationSection642eDepreciationNotProperlyAllocated,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub entity_type: FiduciaryEntityType,
    pub provision_being_evaluated: Section642Provision,
    pub claimed_personal_exemption_dollars: u64,
    pub charitable_contribution_amount_dollars: u64,
    pub charitable_source_funds: CharitableSourceFunds,
    pub charitable_following_year_election_made: bool,
    pub depreciation_allocated_proportionately_to_income: bool,
    pub same_expense_claimed_both_income_and_estate_tax_deductions: bool,
    pub trust_or_estate_terminated_this_year: bool,
    pub excess_deductions_to_pass_to_beneficiaries: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section642Mode,
    pub correct_exemption_dollars: u64,
    pub allowed_charitable_deduction_dollars: u64,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type Section642Input = Input;
pub type Section642Output = Output;
pub type Section642Result = Output;

fn correct_exemption_for(entity: FiduciaryEntityType) -> u64 {
    match entity {
        FiduciaryEntityType::DomesticEstateInProbate => SECTION_642_B_ESTATE_EXEMPTION_DOLLARS,
        FiduciaryEntityType::DomesticComplexTrust | FiduciaryEntityType::ElectingSmallBusinessTrust => {
            SECTION_642_B_COMPLEX_TRUST_EXEMPTION_DOLLARS
        }
        FiduciaryEntityType::DomesticSimpleTrust => SECTION_642_B_SIMPLE_TRUST_EXEMPTION_DOLLARS,
        FiduciaryEntityType::ForeignTrust => 0,
        FiduciaryEntityType::DomesticGrantorTrustPassThrough => 0,
    }
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "26 U.S.C. § 642(a)(1) — foreign tax credit for estate/trust (§ 27 + § 901)".to_string(),
        "26 U.S.C. § 642(b)(1) — $600 personal exemption for estate".to_string(),
        "26 U.S.C. § 642(b)(2)(A)/(B) — $300 personal exemption for complex trust / trust required to distribute all income".to_string(),
        "26 U.S.C. § 642(b)(3) — $100 personal exemption for simple trust (NOT indexed for inflation)".to_string(),
        "26 U.S.C. § 642(c)(1) — UNLIMITED charitable contribution deduction from GROSS INCOME (gross taxable income; tax-exempt interest excluded) pursuant to governing instrument; no AGI floor".to_string(),
        "26 U.S.C. § 642(c)(2) — election for following-year charitable payment to be treated as paid in current year".to_string(),
        "26 U.S.C. § 642(c)(3) — remainder interest in personal residence or farm cross-reference to § 170(f)(3)".to_string(),
        "26 U.S.C. § 642(c)(4) — adjustment for tax-exempt interest allocable to charitable distribution (Treas. Reg. § 1.642(c)-3)".to_string(),
        "26 U.S.C. § 642(d) — net operating loss deduction (§ 172 cross-reference)".to_string(),
        "26 U.S.C. § 642(e) — depreciation/depletion apportioned between fiduciary and beneficiaries based on income allocation".to_string(),
        "26 U.S.C. § 642(f) — amortization deductions (§ 178 / § 169 cross-reference)".to_string(),
        "26 U.S.C. § 642(g) — DOUBLE DEDUCTION DISALLOWED: estate cannot claim BOTH income tax (§ 162/§ 212) AND estate tax (§ 2053/§ 2054) for same expense".to_string(),
        "26 U.S.C. § 642(h) — unused loss carryovers + excess deductions on termination pass to beneficiaries".to_string(),
        "26 U.S.C. § 642(i) — certain reversionary interests § 673 grantor trust cross-reference".to_string(),
        "Treas. Reg. § 1.642(c)-3 — unlimited charitable contribution adjustments (tax-exempt income exclusion)".to_string(),
        "OBBBA 2025 Pease limitation reinstatement (effective 2026) — whether it applies to § 642(c) trust/estate charitable deduction under analysis".to_string(),
        "ACTEC Foundation Charitable Deductions § 642(c) — two requirements: gross income source + governing instrument direction".to_string(),
        "The Tax Adviser Charitable Income Tax Deductions for Trusts and Estates — unlimited deduction without AGI percentage cap".to_string(),
    ];

    if input.entity_type == FiduciaryEntityType::DomesticGrantorTrustPassThrough {
        return Output {
            mode: Section642Mode::NotApplicableGrantorTrustPassThrough,
            correct_exemption_dollars: 0,
            allowed_charitable_deduction_dollars: 0,
            statutory_basis: "Subpart E grantor trust passthrough; no § 642 special rules at trust level".to_string(),
            notes: "Grantor trust passthrough; all income flows to grantor; § 642 inapplicable at trust level.".to_string(),
            citations,
        };
    }

    match input.provision_being_evaluated {
        Section642Provision::Section642bPersonalExemption => {
            let correct = correct_exemption_for(input.entity_type);
            if input.claimed_personal_exemption_dollars > correct {
                return Output {
                    mode: Section642Mode::ViolationSection642bExemptionOverclaimed,
                    correct_exemption_dollars: correct,
                    allowed_charitable_deduction_dollars: 0,
                    statutory_basis: "§ 642(b) — exemption capped per entity type".to_string(),
                    notes: format!(
                        "VIOLATION § 642(b): claimed exemption ${} exceeds statutory cap of ${} for entity type {:?} (NOT indexed for inflation).",
                        input.claimed_personal_exemption_dollars, correct, input.entity_type
                    ),
                    citations,
                };
            }
            Output {
                mode: Section642Mode::CompliantSection642bPersonalExemptionAtCorrectLevel,
                correct_exemption_dollars: correct,
                allowed_charitable_deduction_dollars: 0,
                statutory_basis: format!(
                    "§ 642(b) — ${} exemption for entity {:?}",
                    correct, input.entity_type
                ),
                notes: format!(
                    "COMPLIANT § 642(b): claimed exemption ${} within statutory cap of ${} for {:?}.",
                    input.claimed_personal_exemption_dollars, correct, input.entity_type
                ),
                citations,
            }
        }
        Section642Provision::Section642cCharitableContribution => {
            match input.charitable_source_funds {
                CharitableSourceFunds::CorpusOrPrincipal => Output {
                    mode: Section642Mode::ViolationSection642cPaidFromCorpusNotGrossIncome,
                    correct_exemption_dollars: 0,
                    allowed_charitable_deduction_dollars: 0,
                    statutory_basis: "§ 642(c)(1) — charitable amount must be paid from GROSS INCOME, not corpus".to_string(),
                    notes: format!(
                        "VIOLATION § 642(c)(1): charitable contribution of ${} paid from corpus / principal, NOT from gross taxable income; no § 642(c) deduction.",
                        input.charitable_contribution_amount_dollars
                    ),
                    citations,
                },
                CharitableSourceFunds::TaxExemptInterestIncluded => Output {
                    mode: Section642Mode::ViolationSection642cTaxExemptIncomeIncluded,
                    correct_exemption_dollars: 0,
                    allowed_charitable_deduction_dollars: 0,
                    statutory_basis: "§ 642(c)(4) + Treas. Reg. § 1.642(c)-3 — tax-exempt income portion excluded".to_string(),
                    notes: "VIOLATION § 642(c)(4): tax-exempt interest portion of charitable distribution must be EXCLUDED from § 642(c) deduction per Treas. Reg. § 1.642(c)-3.".to_string(),
                    citations,
                },
                CharitableSourceFunds::GovernigInstrumentSilentNoDirection => Output {
                    mode: Section642Mode::ViolationSection642cNoGoverningInstrumentDirection,
                    correct_exemption_dollars: 0,
                    allowed_charitable_deduction_dollars: 0,
                    statutory_basis: "§ 642(c)(1) — payment must be pursuant to governing instrument direction".to_string(),
                    notes: "VIOLATION § 642(c)(1): governing instrument silent / no charitable direction; trustee discretionary charitable distribution does NOT satisfy § 642(c) requirement.".to_string(),
                    citations,
                },
                CharitableSourceFunds::GrossTaxableIncomeProperSource => Output {
                    mode: Section642Mode::CompliantSection642cCharitableUnlimitedDeductionFromGrossIncome,
                    correct_exemption_dollars: 0,
                    allowed_charitable_deduction_dollars: input.charitable_contribution_amount_dollars,
                    statutory_basis: "§ 642(c)(1) — UNLIMITED deduction from gross income pursuant to governing instrument".to_string(),
                    notes: format!(
                        "COMPLIANT § 642(c)(1): UNLIMITED charitable deduction of ${} from gross taxable income pursuant to governing instrument; no AGI floor.",
                        input.charitable_contribution_amount_dollars
                    ),
                    citations,
                },
            }
        }
        Section642Provision::Section642cFollowingYearElection => {
            if input.charitable_following_year_election_made
                && input.charitable_source_funds == CharitableSourceFunds::GrossTaxableIncomeProperSource
            {
                return Output {
                    mode: Section642Mode::CompliantSection642cFollowingYearElectionApplied,
                    correct_exemption_dollars: 0,
                    allowed_charitable_deduction_dollars: input.charitable_contribution_amount_dollars,
                    statutory_basis: "§ 642(c)(2) — following-year payment treated as paid in current year".to_string(),
                    notes: format!(
                        "COMPLIANT § 642(c)(2): following-year charitable payment of ${} treated as paid in current taxable year (election made).",
                        input.charitable_contribution_amount_dollars
                    ),
                    citations,
                };
            }
            Output {
                mode: Section642Mode::ViolationSection642cNoGoverningInstrumentDirection,
                correct_exemption_dollars: 0,
                allowed_charitable_deduction_dollars: 0,
                statutory_basis: "§ 642(c)(2) — election requirements not met".to_string(),
                notes: "VIOLATION § 642(c)(2): following-year election claimed but underlying gross-income / governing-instrument requirements not satisfied.".to_string(),
                citations,
            }
        }
        Section642Provision::Section642eDepreciationApportionment => {
            if input.depreciation_allocated_proportionately_to_income {
                return Output {
                    mode: Section642Mode::CompliantSection642eDepreciationAllocatedToBeneficiaries,
                    correct_exemption_dollars: 0,
                    allowed_charitable_deduction_dollars: 0,
                    statutory_basis: "§ 642(e) — depreciation apportioned between fiduciary and beneficiaries on trust accounting income basis".to_string(),
                    notes: "COMPLIANT § 642(e): depreciation properly allocated proportionately to trust accounting income between fiduciary and beneficiaries.".to_string(),
                    citations,
                };
            }
            Output {
                mode: Section642Mode::ViolationSection642eDepreciationNotProperlyAllocated,
                correct_exemption_dollars: 0,
                allowed_charitable_deduction_dollars: 0,
                statutory_basis: "§ 642(e) — apportionment requirement not satisfied".to_string(),
                notes: "VIOLATION § 642(e): depreciation not allocated proportionately between fiduciary and beneficiaries based on trust accounting income.".to_string(),
                citations,
            }
        }
        Section642Provision::Section642gDoubleDeductionDisallowed => {
            if input.same_expense_claimed_both_income_and_estate_tax_deductions {
                return Output {
                    mode: Section642Mode::ViolationSection642gDoubleDeductionEstateAndIncomeTax,
                    correct_exemption_dollars: 0,
                    allowed_charitable_deduction_dollars: 0,
                    statutory_basis: "§ 642(g) — same expense cannot be deducted on both estate income tax AND estate tax returns".to_string(),
                    notes: "VIOLATION § 642(g): same expense claimed as both § 162/§ 212 income tax deduction AND § 2053/§ 2054 estate tax deduction; estate must elect ONE.".to_string(),
                    citations,
                };
            }
            Output {
                mode: Section642Mode::CompliantSection642bPersonalExemptionAtCorrectLevel,
                correct_exemption_dollars: 0,
                allowed_charitable_deduction_dollars: 0,
                statutory_basis: "§ 642(g) — no double deduction claimed".to_string(),
                notes: "COMPLIANT § 642(g): same expense not claimed on both income tax and estate tax returns.".to_string(),
                citations,
            }
        }
        Section642Provision::Section642hExcessDeductionsTermination => {
            if input.trust_or_estate_terminated_this_year && input.excess_deductions_to_pass_to_beneficiaries {
                return Output {
                    mode: Section642Mode::CompliantSection642hExcessDeductionsPassedToBeneficiaryOnTermination,
                    correct_exemption_dollars: 0,
                    allowed_charitable_deduction_dollars: 0,
                    statutory_basis: "§ 642(h) — unused loss carryovers + excess deductions pass to beneficiaries on termination".to_string(),
                    notes: "COMPLIANT § 642(h): trust/estate terminated this year; excess deductions properly passed to beneficiaries on final K-1.".to_string(),
                    citations,
                };
            }
            Output {
                mode: Section642Mode::NotApplicableNoEstateOrTrust,
                correct_exemption_dollars: 0,
                allowed_charitable_deduction_dollars: 0,
                statutory_basis: "§ 642(h) inapplicable absent termination + excess deductions".to_string(),
                notes: "§ 642(h) inapplicable: trust/estate not terminated this year or no excess deductions to pass through.".to_string(),
                citations,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_complex_trust_642_b_compliant() -> Input {
        Input {
            entity_type: FiduciaryEntityType::DomesticComplexTrust,
            provision_being_evaluated: Section642Provision::Section642bPersonalExemption,
            claimed_personal_exemption_dollars: 300,
            charitable_contribution_amount_dollars: 0,
            charitable_source_funds: CharitableSourceFunds::GrossTaxableIncomeProperSource,
            charitable_following_year_election_made: false,
            depreciation_allocated_proportionately_to_income: true,
            same_expense_claimed_both_income_and_estate_tax_deductions: false,
            trust_or_estate_terminated_this_year: false,
            excess_deductions_to_pass_to_beneficiaries: false,
        }
    }

    #[test]
    fn grantor_trust_passthrough_not_applicable() {
        let input = Input {
            entity_type: FiduciaryEntityType::DomesticGrantorTrustPassThrough,
            ..baseline_complex_trust_642_b_compliant()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section642Mode::NotApplicableGrantorTrustPassThrough);
    }

    #[test]
    fn complex_trust_642_b_300_exemption_compliant() {
        let result = compute(&baseline_complex_trust_642_b_compliant());
        assert_eq!(result.mode, Section642Mode::CompliantSection642bPersonalExemptionAtCorrectLevel);
        assert_eq!(result.correct_exemption_dollars, 300);
    }

    #[test]
    fn simple_trust_642_b_100_exemption_compliant() {
        let input = Input {
            entity_type: FiduciaryEntityType::DomesticSimpleTrust,
            claimed_personal_exemption_dollars: 100,
            ..baseline_complex_trust_642_b_compliant()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section642Mode::CompliantSection642bPersonalExemptionAtCorrectLevel);
        assert_eq!(result.correct_exemption_dollars, 100);
    }

    #[test]
    fn estate_642_b_600_exemption_compliant() {
        let input = Input {
            entity_type: FiduciaryEntityType::DomesticEstateInProbate,
            claimed_personal_exemption_dollars: 600,
            ..baseline_complex_trust_642_b_compliant()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section642Mode::CompliantSection642bPersonalExemptionAtCorrectLevel);
        assert_eq!(result.correct_exemption_dollars, 600);
    }

    #[test]
    fn simple_trust_overclaimed_200_exemption_violation() {
        let input = Input {
            entity_type: FiduciaryEntityType::DomesticSimpleTrust,
            claimed_personal_exemption_dollars: 200,
            ..baseline_complex_trust_642_b_compliant()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section642Mode::ViolationSection642bExemptionOverclaimed);
    }

    #[test]
    fn estate_overclaimed_1000_exemption_violation() {
        let input = Input {
            entity_type: FiduciaryEntityType::DomesticEstateInProbate,
            claimed_personal_exemption_dollars: 1_000,
            ..baseline_complex_trust_642_b_compliant()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section642Mode::ViolationSection642bExemptionOverclaimed);
    }

    #[test]
    fn section_642c_charitable_unlimited_from_gross_income_compliant() {
        let input = Input {
            provision_being_evaluated: Section642Provision::Section642cCharitableContribution,
            charitable_contribution_amount_dollars: 500_000,
            charitable_source_funds: CharitableSourceFunds::GrossTaxableIncomeProperSource,
            ..baseline_complex_trust_642_b_compliant()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section642Mode::CompliantSection642cCharitableUnlimitedDeductionFromGrossIncome);
        assert_eq!(result.allowed_charitable_deduction_dollars, 500_000);
    }

    #[test]
    fn section_642c_paid_from_corpus_violation() {
        let input = Input {
            provision_being_evaluated: Section642Provision::Section642cCharitableContribution,
            charitable_contribution_amount_dollars: 100_000,
            charitable_source_funds: CharitableSourceFunds::CorpusOrPrincipal,
            ..baseline_complex_trust_642_b_compliant()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section642Mode::ViolationSection642cPaidFromCorpusNotGrossIncome);
    }

    #[test]
    fn section_642c_tax_exempt_income_included_violation() {
        let input = Input {
            provision_being_evaluated: Section642Provision::Section642cCharitableContribution,
            charitable_contribution_amount_dollars: 50_000,
            charitable_source_funds: CharitableSourceFunds::TaxExemptInterestIncluded,
            ..baseline_complex_trust_642_b_compliant()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section642Mode::ViolationSection642cTaxExemptIncomeIncluded);
    }

    #[test]
    fn section_642c_no_governing_instrument_direction_violation() {
        let input = Input {
            provision_being_evaluated: Section642Provision::Section642cCharitableContribution,
            charitable_contribution_amount_dollars: 75_000,
            charitable_source_funds: CharitableSourceFunds::GovernigInstrumentSilentNoDirection,
            ..baseline_complex_trust_642_b_compliant()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section642Mode::ViolationSection642cNoGoverningInstrumentDirection);
    }

    #[test]
    fn section_642c_following_year_election_compliant() {
        let input = Input {
            provision_being_evaluated: Section642Provision::Section642cFollowingYearElection,
            charitable_contribution_amount_dollars: 200_000,
            charitable_following_year_election_made: true,
            charitable_source_funds: CharitableSourceFunds::GrossTaxableIncomeProperSource,
            ..baseline_complex_trust_642_b_compliant()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section642Mode::CompliantSection642cFollowingYearElectionApplied);
    }

    #[test]
    fn section_642e_depreciation_allocated_compliant() {
        let input = Input {
            provision_being_evaluated: Section642Provision::Section642eDepreciationApportionment,
            depreciation_allocated_proportionately_to_income: true,
            ..baseline_complex_trust_642_b_compliant()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section642Mode::CompliantSection642eDepreciationAllocatedToBeneficiaries);
    }

    #[test]
    fn section_642e_depreciation_not_allocated_violation() {
        let input = Input {
            provision_being_evaluated: Section642Provision::Section642eDepreciationApportionment,
            depreciation_allocated_proportionately_to_income: false,
            ..baseline_complex_trust_642_b_compliant()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section642Mode::ViolationSection642eDepreciationNotProperlyAllocated);
    }

    #[test]
    fn section_642g_double_deduction_violation() {
        let input = Input {
            entity_type: FiduciaryEntityType::DomesticEstateInProbate,
            provision_being_evaluated: Section642Provision::Section642gDoubleDeductionDisallowed,
            same_expense_claimed_both_income_and_estate_tax_deductions: true,
            ..baseline_complex_trust_642_b_compliant()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section642Mode::ViolationSection642gDoubleDeductionEstateAndIncomeTax);
    }

    #[test]
    fn section_642h_termination_excess_deductions_compliant() {
        let input = Input {
            provision_being_evaluated: Section642Provision::Section642hExcessDeductionsTermination,
            trust_or_estate_terminated_this_year: true,
            excess_deductions_to_pass_to_beneficiaries: true,
            ..baseline_complex_trust_642_b_compliant()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section642Mode::CompliantSection642hExcessDeductionsPassedToBeneficiaryOnTermination);
    }

    #[test]
    fn citations_pin_section_642_subsections_and_treas_regs() {
        let result = compute(&baseline_complex_trust_642_b_compliant());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("§ 642(a)(1)"));
        assert!(joined.contains("§ 642(b)(1)"));
        assert!(joined.contains("§ 642(b)(2)(A)"));
        assert!(joined.contains("§ 642(b)(3)"));
        assert!(joined.contains("§ 642(c)(1)"));
        assert!(joined.contains("§ 642(c)(2)"));
        assert!(joined.contains("§ 642(c)(3)"));
        assert!(joined.contains("§ 642(c)(4)"));
        assert!(joined.contains("§ 642(d)"));
        assert!(joined.contains("§ 642(e)"));
        assert!(joined.contains("§ 642(f)"));
        assert!(joined.contains("§ 642(g)"));
        assert!(joined.contains("§ 642(h)"));
        assert!(joined.contains("§ 642(i)"));
        assert!(joined.contains("§ 1.642(c)-3"));
        assert!(joined.contains("OBBBA 2025 Pease"));
        assert!(joined.contains("ACTEC"));
    }

    #[test]
    fn constant_pin_exemptions_and_dates() {
        assert_eq!(SECTION_642_B_ESTATE_EXEMPTION_DOLLARS, 600);
        assert_eq!(SECTION_642_B_COMPLEX_TRUST_EXEMPTION_DOLLARS, 300);
        assert_eq!(SECTION_642_B_SIMPLE_TRUST_EXEMPTION_DOLLARS, 100);
        assert_eq!(SECTION_642_C_AGI_FLOOR_BASIS_POINTS, 0);
        assert!(SECTION_642_C_UNLIMITED_DEDUCTION);
        assert_eq!(OBBBA_2025_PEASE_LIMITATION_REINSTATEMENT_YEAR, 2026);
    }

    #[test]
    fn esbt_complex_trust_300_exemption_compliant() {
        let input = Input {
            entity_type: FiduciaryEntityType::ElectingSmallBusinessTrust,
            claimed_personal_exemption_dollars: 300,
            ..baseline_complex_trust_642_b_compliant()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section642Mode::CompliantSection642bPersonalExemptionAtCorrectLevel);
    }
}
