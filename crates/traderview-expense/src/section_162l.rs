//! IRC § 162(l) — Special Rules for Health Insurance Costs of Self-Employed
//! Individuals.
//!
//! § 162(l) provides an ABOVE-THE-LINE deduction for medical, dental, and
//! qualified long-term care insurance premiums paid by self-employed
//! individuals on behalf of themselves, their spouses, dependents, and
//! children under age 27. The deduction is taken on Schedule 1 (Form 1040)
//! line 17 — it reduces AGI directly rather than being an itemized
//! deduction subject to the § 213(a) 7.5% AGI floor.
//!
//! § 162(l)(1)(A) general allowance: insurance premiums paid during the
//! taxable year by a self-employed individual for "medical care" insurance
//! (within meaning of § 213(d)(1)(D)) plus qualified long-term care
//! insurance (within meaning of § 7702B(b)) are deductible above the line.
//!
//! § 162(l)(1)(B) earned income limitation: total deduction cannot exceed
//! the individual's earned income derived from the trade or business with
//! respect to which the insurance plan is established. Earned income
//! defined under § 401(c)(2) for partners and sole proprietors.
//!
//! § 162(l)(2)(B) plan establishment requirement: insurance plan must be
//! established under the trade or business with respect to which the
//! deduction is claimed. For S-corp >2% shareholders, plan is established
//! through the S corp; premiums paid by S corp are INCLUDED in shareholder's
//! W-2 box 1 wages then deducted by shareholder on Schedule 1 line 17.
//!
//! § 162(l)(2)(B) double-coverage prohibition: deduction disallowed for any
//! calendar month in which taxpayer (or spouse) is ELIGIBLE to participate
//! in subsidized health plan maintained by employer of taxpayer or spouse.
//! Tested month-by-month — eligibility for ONE month disallows that month's
//! deduction only.
//!
//! Chief Counsel Advice 201228037 (July 13, 2012) confirmed all Medicare
//! premiums (Parts A, B, C Medicare Advantage, D prescription drug) are
//! "insurance constituting medical care" under § 162(l)(1)(D) and may be
//! deducted above the line by self-employed individuals.
//!
//! Earned income computation:
//! - Sole proprietor / single-member LLC: Schedule C net profit minus
//!   deductible half of SE tax (Schedule 1 line 15) minus SE retirement
//!   plan contributions (Schedule 1 line 16).
//! - Partner: net earnings from self-employment (Schedule K-1 box 14 code
//!   A) minus deductible half of SE tax minus SE retirement contributions.
//! - S corporation >2% shareholder: W-2 box 1 wages paid by S corp.
//!
//! Form 7206 (Self-Employed Health Insurance Deduction) is the dedicated
//! computation form effective for tax years beginning after December 31,
//! 2022 (replaced prior Worksheet in Publication 535).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BusinessStructure {
    /// Sole proprietor or single-member LLC reporting on Schedule C.
    SoleProprietorOrSingleMemberLlc,
    /// Partnership partner reporting Schedule K-1 box 14 code A.
    PartnershipPartner,
    /// S corporation shareholder owning more than 2% of stock.
    SCorporationShareholderOver2Pct,
    /// S corporation shareholder owning 2% or less — NOT eligible for §
    /// 162(l) treatment; S corp pays premium as ordinary employee fringe
    /// benefit under § 105 / § 106.
    SCorporationShareholderTwoPctOrLess,
    /// C corporation shareholder — § 162(l) inapplicable.
    CCorporationShareholder,
    /// W-2 employee with no self-employment income — § 162(l) inapplicable.
    W2EmployeeOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PremiumType {
    /// Medical / dental health insurance.
    MedicalDentalHealth,
    /// Qualified long-term care insurance under § 7702B(b).
    QualifiedLongTermCare,
    /// Medicare Parts A, B, C, D per CCA 201228037.
    MedicarePartABCD,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    NotEligibleBusinessStructure,
    DoubleCoverageDisallowedForMonth,
    PlanNotEstablishedThroughBusiness,
    DeductionAllowedFullPremium,
    DeductionLimitedByEarnedIncome,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section162lInput {
    pub business_structure: BusinessStructure,
    pub premium_type: PremiumType,
    pub annual_premium_cents: u64,
    /// Earned income from the trade or business in cents (sole prop: Sch C
    /// net minus 1/2 SE tax minus SE retirement; partner: SE net minus 1/2
    /// SE tax minus SE retirement; S corp shareholder: W-2 box 1 wages).
    pub earned_income_from_business_cents: u64,
    /// Whether plan established under the trade or business per § 162(l)(2).
    pub plan_established_through_business: bool,
    /// Number of months taxpayer (or spouse) eligible for subsidized
    /// employer-sponsored plan during taxable year (§ 162(l)(2)(B)
    /// double-coverage prohibition).
    pub double_coverage_eligible_months: u32,
    /// Whether premiums included in S-corp shareholder's W-2 box 1 wages
    /// (required for S corp >2% shareholder per Notice 2008-1).
    pub s_corp_premiums_in_w2_box_1: bool,
    /// Tax year.
    pub taxable_year: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section162lResult {
    pub severity: Severity,
    pub eligible_premium_after_double_coverage_cents: u64,
    pub earned_income_limit_cents: u64,
    pub allowed_deduction_cents: u64,
    pub disallowed_amount_cents: u64,
    pub recommended_actions: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub const MONTHS_IN_YEAR: u32 = 12;
pub const FORM_7206_EFFECTIVE_YEAR: i32 = 2023;
pub const CCA_MEDICARE_DEDUCTIBILITY_DATE: &str = "2012-07-13";

pub fn check(input: &Section162lInput) -> Section162lResult {
    let mut actions: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    if matches!(
        input.business_structure,
        BusinessStructure::CCorporationShareholder | BusinessStructure::W2EmployeeOnly
    ) {
        notes.push(
            "C corporation shareholders and W-2 employees without self-employment income are \
             NOT eligible for § 162(l) above-the-line deduction. C corp shareholders deduct \
             health insurance at corporate level under § 162(a) as ordinary business expense; \
             premiums excluded from employee gross income under § 106. W-2 employees with \
             unreimbursed medical expenses may itemize under § 213 subject to 7.5% AGI floor."
                .to_string(),
        );
        return empty_result(
            Severity::NotEligibleBusinessStructure,
            input,
            actions,
            notes,
            "26 U.S.C. § 162(l)(1); coord. § 106, § 213",
        );
    }

    if matches!(
        input.business_structure,
        BusinessStructure::SCorporationShareholderTwoPctOrLess
    ) {
        notes.push(
            "S corporation shareholder owning 2% or less of stock is treated as an ORDINARY \
             EMPLOYEE for fringe benefit purposes under § 1372 — premiums excluded from \
             gross income under § 106 (employer-paid health insurance) rather than § 162(l) \
             above-the-line deduction."
                .to_string(),
        );
        return empty_result(
            Severity::NotEligibleBusinessStructure,
            input,
            actions,
            notes,
            "26 U.S.C. § 1372; § 106",
        );
    }

    if !input.plan_established_through_business {
        notes.push(
            "§ 162(l)(2)(B) plan-establishment requirement NOT satisfied — insurance plan \
             must be established under the trade or business. For sole proprietor or single-\
             member LLC, plan must be in the name of the business or the proprietor (per \
             Notice 2008-1 + Rev. Proc. 79-46). For partner, plan must be in partner's name \
             OR partnership name with premiums treated as guaranteed payment. For S corp \
             >2% shareholder, plan must be established by the S corp with premiums included \
             in W-2 box 1 wages."
                .to_string(),
        );
        return empty_result(
            Severity::PlanNotEstablishedThroughBusiness,
            input,
            actions,
            notes,
            "26 U.S.C. § 162(l)(2)(B); Notice 2008-1; Rev. Proc. 79-46",
        );
    }

    if matches!(
        input.business_structure,
        BusinessStructure::SCorporationShareholderOver2Pct
    ) && !input.s_corp_premiums_in_w2_box_1
    {
        notes.push(
            "S corporation >2% shareholder premiums NOT included in W-2 box 1 wages — § \
             162(l) deduction disallowed under Notice 2008-1 W-2 reporting requirement. \
             Premiums paid by S corp must be reported as additional wages on shareholder's \
             W-2 box 1 (not subject to FICA withholding per § 3121(a)(2)(B) exclusion). \
             Correct via Form W-2c amended wage statement."
                .to_string(),
        );
        return empty_result(
            Severity::PlanNotEstablishedThroughBusiness,
            input,
            actions,
            notes,
            "26 U.S.C. § 162(l); Notice 2008-1; § 3121(a)(2)(B)",
        );
    }

    let allowed_months = MONTHS_IN_YEAR.saturating_sub(input.double_coverage_eligible_months);
    let eligible_premium_after_dc: u64 = (u128::from(input.annual_premium_cents)
        * u128::from(allowed_months)
        / u128::from(MONTHS_IN_YEAR)) as u64;

    if allowed_months == 0 {
        notes.push(
            "§ 162(l)(2)(B) double-coverage prohibition applies for all 12 months — \
             taxpayer or spouse eligible for subsidized employer health plan every month \
             of taxable year; entire premium disallowed under § 162(l)."
                .to_string(),
        );
        return Section162lResult {
            severity: Severity::DoubleCoverageDisallowedForMonth,
            eligible_premium_after_double_coverage_cents: 0,
            earned_income_limit_cents: input.earned_income_from_business_cents,
            allowed_deduction_cents: 0,
            disallowed_amount_cents: input.annual_premium_cents,
            recommended_actions: actions,
            citation: "26 U.S.C. § 162(l)(2)(B)",
            notes,
        };
    }

    let earned_income_limit = input.earned_income_from_business_cents;
    let allowed = eligible_premium_after_dc.min(earned_income_limit);
    let disallowed = input.annual_premium_cents.saturating_sub(allowed);

    let severity = if allowed < eligible_premium_after_dc {
        Severity::DeductionLimitedByEarnedIncome
    } else {
        Severity::DeductionAllowedFullPremium
    };

    if matches!(input.premium_type, PremiumType::MedicarePartABCD) {
        actions.push(format!(
            "Medicare Parts A, B, C, D premiums are 'insurance constituting medical care' \
             under § 162(l)(1)(D) per Chief Counsel Advice 201228037 ({}); allowed above-the-\
             line deduction of {} cents (after double-coverage and earned-income limits). \
             Report on Form 7206 (effective for tax years beginning after December 31, {}) \
             plus Schedule 1 line 17.",
            CCA_MEDICARE_DEDUCTIBILITY_DATE,
            allowed,
            FORM_7206_EFFECTIVE_YEAR - 1
        ));
    } else if matches!(input.premium_type, PremiumType::QualifiedLongTermCare) {
        actions.push(format!(
            "Qualified long-term care insurance premium per § 7702B(b) subject to age-based \
             annual dollar limit under § 213(d)(10) (e.g., 2024 limits: $470 age ≤40, $880 \
             age 41-50, $1,760 age 51-60, $4,710 age 61-70, $5,880 age 71+). Allowed above-\
             the-line deduction of {} cents; verify premium does not exceed § 213(d)(10) \
             cap for taxpayer's age before claiming.",
            allowed
        ));
    } else {
        actions.push(format!(
            "Medical/dental health insurance premium of {} cents allowed as above-the-line \
             deduction of {} cents (after double-coverage and earned-income limits) on Form \
             7206 and Schedule 1 line 17 of Form 1040.",
            input.annual_premium_cents, allowed
        ));
    }

    if matches!(severity, Severity::DeductionLimitedByEarnedIncome) {
        actions.push(format!(
            "Earned income limitation applies: deduction capped at {} cents (earned income \
             from trade or business per § 401(c)(2)); {} cents of premium DISALLOWED. \
             Excess premium not carried forward — lost forever. Plan for next year by \
             increasing business net income, accelerating S corp wages, or reducing \
             coverage where economically rational.",
            earned_income_limit, disallowed
        ));
    }

    notes.push(
        "Coordination with [[section_213]] (medical expense itemized deduction below the \
         line, subject to 7.5% AGI floor — disallowed § 162(l) excess may be claimed under \
         § 213), [[section_106]] (employer-paid health insurance exclusion for ordinary \
         employees), [[section_7702b]] (qualified long-term care insurance definition + \
         age-based dollar limits per § 213(d)(10)), [[section_401c2]] (earned income \
         definition for self-employed retirement and § 162(l) purposes), [[section_1372]] \
         (S corp fringe benefit rules treating >2% shareholders as self-employed), \
         [[section_4980h]] (employer shared responsibility — distinct ACA framework), \
         [[section_36b]] (premium tax credit — coordination with § 162(l) deduction and \
         Marketplace coverage). § 162(l) deduction reduces both AGI (Form 1040 line 11) \
         and self-employment tax base via § 162(l)(4) coordination."
            .to_string(),
    );

    Section162lResult {
        severity,
        eligible_premium_after_double_coverage_cents: eligible_premium_after_dc,
        earned_income_limit_cents: earned_income_limit,
        allowed_deduction_cents: allowed,
        disallowed_amount_cents: disallowed,
        recommended_actions: actions,
        citation: "26 U.S.C. § 162(l)(1)-(5); CCA 201228037; Notice 2008-1; Form 7206",
        notes,
    }
}

fn empty_result(
    severity: Severity,
    input: &Section162lInput,
    recommended_actions: Vec<String>,
    mut notes: Vec<String>,
    citation: &'static str,
) -> Section162lResult {
    notes.push(
        "Coordination with [[section_213]] (itemized medical), [[section_106]] (employer \
         exclusion), [[section_7702b]] (LTC), [[section_401c2]] (earned income), \
         [[section_1372]] (S corp fringe benefits), [[section_4980h]] (ACA employer \
         mandate), [[section_36b]] (premium tax credit)."
            .to_string(),
    );
    let _ = input;
    Section162lResult {
        severity,
        eligible_premium_after_double_coverage_cents: 0,
        earned_income_limit_cents: 0,
        allowed_deduction_cents: 0,
        disallowed_amount_cents: 0,
        recommended_actions,
        citation,
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> Section162lInput {
        Section162lInput {
            business_structure: BusinessStructure::SoleProprietorOrSingleMemberLlc,
            premium_type: PremiumType::MedicalDentalHealth,
            annual_premium_cents: 12_000_00,
            earned_income_from_business_cents: 100_000_00,
            plan_established_through_business: true,
            double_coverage_eligible_months: 0,
            s_corp_premiums_in_w2_box_1: false,
            taxable_year: 2024,
        }
    }

    #[test]
    fn sole_proprietor_full_deduction_allowed() {
        let i = baseline();
        let r = check(&i);
        assert!(matches!(r.severity, Severity::DeductionAllowedFullPremium));
        assert_eq!(r.allowed_deduction_cents, i.annual_premium_cents);
        assert_eq!(r.disallowed_amount_cents, 0);
    }

    #[test]
    fn c_corp_shareholder_not_eligible() {
        let mut i = baseline();
        i.business_structure = BusinessStructure::CCorporationShareholder;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::NotEligibleBusinessStructure));
        assert!(r.notes.iter().any(|n| n.contains("§ 162(a)")));
        assert!(r.notes.iter().any(|n| n.contains("§ 213")));
    }

    #[test]
    fn w2_employee_only_not_eligible() {
        let mut i = baseline();
        i.business_structure = BusinessStructure::W2EmployeeOnly;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::NotEligibleBusinessStructure));
    }

    #[test]
    fn s_corp_2_pct_or_less_treated_as_ordinary_employee() {
        let mut i = baseline();
        i.business_structure = BusinessStructure::SCorporationShareholderTwoPctOrLess;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::NotEligibleBusinessStructure));
        assert!(r.notes.iter().any(|n| n.contains("§ 1372")));
        assert!(r.notes.iter().any(|n| n.contains("§ 106")));
    }

    #[test]
    fn plan_not_established_through_business_disallowed() {
        let mut i = baseline();
        i.plan_established_through_business = false;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::PlanNotEstablishedThroughBusiness));
        assert!(r.notes.iter().any(|n| n.contains("Notice 2008-1")));
        assert!(r.notes.iter().any(|n| n.contains("Rev. Proc. 79-46")));
    }

    #[test]
    fn s_corp_over_2_pct_without_w2_box_1_inclusion_disallowed() {
        let mut i = baseline();
        i.business_structure = BusinessStructure::SCorporationShareholderOver2Pct;
        i.s_corp_premiums_in_w2_box_1 = false;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::PlanNotEstablishedThroughBusiness));
        assert!(r.notes.iter().any(|n| n.contains("Form W-2c")));
        assert!(r.notes.iter().any(|n| n.contains("§ 3121(a)(2)(B)")));
    }

    #[test]
    fn s_corp_over_2_pct_with_w2_box_1_inclusion_allowed() {
        let mut i = baseline();
        i.business_structure = BusinessStructure::SCorporationShareholderOver2Pct;
        i.s_corp_premiums_in_w2_box_1 = true;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::DeductionAllowedFullPremium));
    }

    #[test]
    fn double_coverage_all_12_months_fully_disallowed() {
        let mut i = baseline();
        i.double_coverage_eligible_months = 12;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::DoubleCoverageDisallowedForMonth));
        assert_eq!(r.allowed_deduction_cents, 0);
        assert_eq!(r.disallowed_amount_cents, i.annual_premium_cents);
    }

    #[test]
    fn double_coverage_3_months_pro_rated_75_pct_allowed() {
        let mut i = baseline();
        i.double_coverage_eligible_months = 3;
        let r = check(&i);
        let expected = i.annual_premium_cents * 9 / 12;
        assert_eq!(r.eligible_premium_after_double_coverage_cents, expected);
        assert_eq!(r.allowed_deduction_cents, expected);
    }

    #[test]
    fn earned_income_limitation_caps_deduction() {
        let mut i = baseline();
        i.annual_premium_cents = 50_000_00;
        i.earned_income_from_business_cents = 30_000_00;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::DeductionLimitedByEarnedIncome));
        assert_eq!(r.allowed_deduction_cents, 30_000_00);
        assert_eq!(r.disallowed_amount_cents, 20_000_00);
    }

    #[test]
    fn zero_earned_income_no_deduction() {
        let mut i = baseline();
        i.earned_income_from_business_cents = 0;
        let r = check(&i);
        assert_eq!(r.allowed_deduction_cents, 0);
    }

    #[test]
    fn partner_full_deduction_allowed() {
        let mut i = baseline();
        i.business_structure = BusinessStructure::PartnershipPartner;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::DeductionAllowedFullPremium));
    }

    #[test]
    fn medicare_premium_deductible_per_cca_201228037() {
        let mut i = baseline();
        i.premium_type = PremiumType::MedicarePartABCD;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::DeductionAllowedFullPremium));
        assert!(r.recommended_actions.iter().any(|a| a.contains("Chief Counsel Advice 201228037")));
        assert!(r.recommended_actions.iter().any(|a| a.contains("Medicare Parts A, B, C, D")));
    }

    #[test]
    fn long_term_care_premium_subject_to_section_213_d_10_cap() {
        let mut i = baseline();
        i.premium_type = PremiumType::QualifiedLongTermCare;
        let r = check(&i);
        assert!(r.recommended_actions.iter().any(|a| a.contains("§ 7702B(b)")));
        assert!(r.recommended_actions.iter().any(|a| a.contains("§ 213(d)(10)")));
    }

    #[test]
    fn form_7206_referenced_in_action() {
        let i = baseline();
        let r = check(&i);
        assert!(r.recommended_actions.iter().any(|a| a.contains("Form 7206")));
        assert!(r.recommended_actions.iter().any(|a| a.contains("Schedule 1 line 17")));
    }

    #[test]
    fn months_in_year_pins_12() {
        assert_eq!(MONTHS_IN_YEAR, 12);
    }

    #[test]
    fn form_7206_effective_year_pins_2023() {
        assert_eq!(FORM_7206_EFFECTIVE_YEAR, 2023);
    }

    #[test]
    fn cca_medicare_date_pins_2012_07_13() {
        assert_eq!(CCA_MEDICARE_DEDUCTIBILITY_DATE, "2012-07-13");
    }

    #[test]
    fn citation_pins_162l_cca_notice_form_7206() {
        let i = baseline();
        let r = check(&i);
        assert!(r.citation.contains("§ 162(l)(1)-(5)"));
        assert!(r.citation.contains("CCA 201228037"));
        assert!(r.citation.contains("Notice 2008-1"));
        assert!(r.citation.contains("Form 7206"));
    }

    #[test]
    fn coordination_note_references_213_106_7702b_1372() {
        let i = baseline();
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("section_213")));
        assert!(r.notes.iter().any(|n| n.contains("section_106")));
        assert!(r.notes.iter().any(|n| n.contains("section_7702b")));
        assert!(r.notes.iter().any(|n| n.contains("section_1372")));
        assert!(r.notes.iter().any(|n| n.contains("section_401c2")));
        assert!(r.notes.iter().any(|n| n.contains("section_36b")));
    }

    #[test]
    fn realistic_sole_prop_50k_income_15k_premium_full_deduction() {
        let mut i = baseline();
        i.annual_premium_cents = 15_000_00;
        i.earned_income_from_business_cents = 50_000_00;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::DeductionAllowedFullPremium));
        assert_eq!(r.allowed_deduction_cents, 15_000_00);
    }

    #[test]
    fn realistic_partner_high_premium_low_income_capped() {
        let mut i = baseline();
        i.business_structure = BusinessStructure::PartnershipPartner;
        i.annual_premium_cents = 30_000_00;
        i.earned_income_from_business_cents = 20_000_00;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::DeductionLimitedByEarnedIncome));
        assert_eq!(r.allowed_deduction_cents, 20_000_00);
        assert_eq!(r.disallowed_amount_cents, 10_000_00);
    }

    #[test]
    fn extreme_premium_does_not_overflow() {
        let mut i = baseline();
        i.annual_premium_cents = u64::MAX / 100;
        i.earned_income_from_business_cents = u64::MAX / 100;
        i.double_coverage_eligible_months = 6;
        let r = check(&i);
        let _ = r.allowed_deduction_cents;
    }

    #[test]
    fn zero_premium_zero_deduction() {
        let mut i = baseline();
        i.annual_premium_cents = 0;
        let r = check(&i);
        assert_eq!(r.allowed_deduction_cents, 0);
    }

    #[test]
    fn double_coverage_overage_saturates_at_zero_months() {
        let mut i = baseline();
        i.double_coverage_eligible_months = 20;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::DoubleCoverageDisallowedForMonth));
        assert_eq!(r.allowed_deduction_cents, 0);
    }
}
