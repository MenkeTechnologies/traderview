//! IRC § 56A — Adjusted Financial Statement Income (AFSI) plus § 55(b)(2)
//! Corporate Alternative Minimum Tax (CAMT) — 15% minimum tax on AFSI of
//! applicable corporations. Enacted by Inflation Reduction Act of 2022 (Pub.
//! L. 117-169 § 10101), effective for taxable years beginning after December
//! 31, 2022.
//!
//! § 55(b)(2) computes tentative minimum tax for applicable corporation as
//! 15% × adjusted financial statement income (AFSI) reduced by the CAMT
//! foreign tax credit (FTC). CAMT applies only when this tentative minimum
//! tax exceeds the regular corporate tax under § 11; the excess is the CAMT.
//!
//! § 59(k)(1) "applicable corporation" — any corporation OTHER than an S
//! corporation, regulated investment company (RIC), or real estate investment
//! trust (REIT) whose average annual AFSI exceeds $1,000,000,000 for any
//! three consecutive taxable years ending after December 31, 2021.
//!
//! § 59(k)(2) FPMG (Foreign-Parented Multinational Group) rule — the AFSI of
//! a member of an FPMG includes the AFSI of all other members of the FPMG
//! plus the AFSI of all persons treated as a single employer with the
//! taxpayer under § 52. US-resident corporate member of an FPMG is an
//! applicable corporation if the FPMG's AFSI exceeds $1B AND the US member's
//! own AFSI averages at least $100,000,000 over the three-year test period.
//!
//! § 56A(c) AFSI adjustments to GAAP / IFRS book net income: (1) consolidated
//! financial statement; (2) reorganization adjustments; (3) federal income
//! tax expense back-out; (4) defined benefit pension plan adjustment; (5)
//! cooperative dividend; (6) foreign income adjustments; (7) qualified
//! depreciation (uses § 168 not book); (8) financial statement net operating
//! loss; (9) treatment of distributions from CFC; (10) wholly-owned
//! disregarded entity; (11) consolidated tax group; (12) covered tax
//! benefit; (13) tax expense allocation; (14) qualified wireless spectrum;
//! (15) section 38 general business credit; (16) accelerated depreciation
//! (cost recovery).
//!
//! § 56A(d) financial statement NOL — applicable corporation may carry
//! forward FSNOL indefinitely; 80% AFSI limitation parallel to § 172
//! regular-tax NOL.
//!
//! § 38(c)(6)(E) general business credit limitation: applicable corporation
//! may use general business credits against CAMT, but only up to 75% of
//! AFSI tentative minimum tax.
//!
//! § 53(c)-(d) CAMT credit carryforward: CAMT paid generates indefinite
//! credit carryforward against future regular tax liability (when regular
//! exceeds tentative minimum).
//!
//! Notice 2023-7 + Notice 2023-20 + Notice 2023-64 plus Proposed Regulations
//! REG-112129-23 (September 13, 2024) plus Notice 2025-46 + Notice 2025-49
//! (interim guidance through 2025-2026) govern computation, FPMG aggregation,
//! safe harbors, and Form 4626 reporting requirements.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    /// Domestic C corporation.
    DomesticCCorporation,
    /// S corporation — categorically excluded per § 59(k)(1).
    SCorporation,
    /// Regulated Investment Company (RIC) — categorically excluded.
    RegulatedInvestmentCompany,
    /// Real Estate Investment Trust (REIT) — categorically excluded.
    RealEstateInvestmentTrust,
    /// US-resident member of Foreign-Parented Multinational Group.
    UsMemberForeignParentedGroup,
    /// Foreign corporation not engaged in US trade/business.
    ForeignCorporationNoUsBusiness,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CamtSeverity {
    NotApplicable,
    PreEffectiveDate,
    CategoricallyExempt,
    BelowApplicableCorporationThreshold,
    FpmgBelow100MUsAfsiSafeHarbor,
    ApplicableCorporationCamtNotOwed,
    ApplicableCorporationCamtOwed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section56aInput {
    pub entity_type: EntityType,
    /// Taxable year (calendar). CAMT effective for years beginning after
    /// December 31, 2022.
    pub taxable_year: i32,
    /// Three-year prior period AFSI (year T-1, T-2, T-3) in cents.
    pub afsi_prior_year_1_cents: i64,
    pub afsi_prior_year_2_cents: i64,
    pub afsi_prior_year_3_cents: i64,
    /// Current-year AFSI in cents after § 56A(c) adjustments.
    pub current_year_afsi_cents: i64,
    /// Total AFSI of FPMG (if applicable) in cents — used for § 59(k)(2)
    /// $1B test.
    pub fpmg_total_afsi_cents: i64,
    /// CAMT foreign tax credit in cents.
    pub camt_ftc_cents: u64,
    /// Regular corporate tax liability under § 11 in cents (before CAMT).
    pub regular_corporate_tax_cents: u64,
    /// CAMT NOL carryforward in cents under § 56A(d).
    pub camt_nol_carryforward_cents: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section56aResult {
    pub severity: CamtSeverity,
    pub is_applicable_corporation: bool,
    pub average_three_year_afsi_cents: i64,
    pub afsi_after_nol_cents: i64,
    pub tentative_minimum_tax_cents: u64,
    pub regular_corporate_tax_cents: u64,
    pub camt_owed_cents: u64,
    pub camt_credit_carryforward_to_future_year_cents: u64,
    pub recommended_actions: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub const CAMT_RATE_BPS: u32 = 1_500;
pub const APPLICABLE_CORPORATION_AFSI_THRESHOLD_CENTS: i64 = 100_000_000_000;
pub const FPMG_US_MEMBER_AFSI_THRESHOLD_CENTS: i64 = 10_000_000_000;
pub const FSNOL_LIMITATION_BPS: u32 = 8_000;
pub const CAMT_EFFECTIVE_TAX_YEAR: i32 = 2023;
pub const APPLICABLE_CORPORATION_FIRST_TEST_YEAR_AFTER: i32 = 2021;

pub fn check(input: &Section56aInput) -> Section56aResult {
    let mut actions: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    if input.taxable_year < CAMT_EFFECTIVE_TAX_YEAR {
        notes.push(format!(
            "CAMT effective for taxable years beginning after December 31, {} per Pub. L. \
             117-169 § 10101; tax year {} pre-effective — CAMT inapplicable.",
            APPLICABLE_CORPORATION_FIRST_TEST_YEAR_AFTER + 1,
            input.taxable_year
        ));
        return Section56aResult {
            severity: CamtSeverity::PreEffectiveDate,
            is_applicable_corporation: false,
            average_three_year_afsi_cents: 0,
            afsi_after_nol_cents: input.current_year_afsi_cents,
            tentative_minimum_tax_cents: 0,
            regular_corporate_tax_cents: input.regular_corporate_tax_cents,
            camt_owed_cents: 0,
            camt_credit_carryforward_to_future_year_cents: 0,
            recommended_actions: actions,
            citation: "26 U.S.C. § 55(b)(2); § 56A; § 59(k); Pub. L. 117-169 § 10101",
            notes,
        };
    }

    if matches!(
        input.entity_type,
        EntityType::SCorporation
            | EntityType::RegulatedInvestmentCompany
            | EntityType::RealEstateInvestmentTrust
            | EntityType::ForeignCorporationNoUsBusiness
    ) {
        notes.push(
            "Entity categorically excluded from § 59(k)(1) applicable-corporation definition; \
             S corporations, RICs, REITs, and foreign corporations with no US trade/business \
             are NOT subject to CAMT."
                .to_string(),
        );
        return Section56aResult {
            severity: CamtSeverity::CategoricallyExempt,
            is_applicable_corporation: false,
            average_three_year_afsi_cents: 0,
            afsi_after_nol_cents: input.current_year_afsi_cents,
            tentative_minimum_tax_cents: 0,
            regular_corporate_tax_cents: input.regular_corporate_tax_cents,
            camt_owed_cents: 0,
            camt_credit_carryforward_to_future_year_cents: 0,
            recommended_actions: actions,
            citation: "26 U.S.C. § 59(k)(1)",
            notes,
        };
    }

    let avg_3y_afsi: i64 = (i128::from(input.afsi_prior_year_1_cents)
        + i128::from(input.afsi_prior_year_2_cents)
        + i128::from(input.afsi_prior_year_3_cents))
    .div_euclid(3) as i64;

    let is_us_member_fpmg = matches!(input.entity_type, EntityType::UsMemberForeignParentedGroup);

    let is_applicable: bool;
    if is_us_member_fpmg {
        let fpmg_meets_1b =
            input.fpmg_total_afsi_cents > APPLICABLE_CORPORATION_AFSI_THRESHOLD_CENTS;
        let us_member_meets_100m = avg_3y_afsi >= FPMG_US_MEMBER_AFSI_THRESHOLD_CENTS;
        is_applicable = fpmg_meets_1b && us_member_meets_100m;
        if fpmg_meets_1b && !us_member_meets_100m {
            notes.push(format!(
                "FPMG aggregate AFSI exceeds $1B threshold but US-resident member's three-year \
                 average AFSI of {} cents falls below $100M safe-harbor floor per § 59(k)(2); \
                 US member NOT an applicable corporation for current tax year.",
                avg_3y_afsi
            ));
            return Section56aResult {
                severity: CamtSeverity::FpmgBelow100MUsAfsiSafeHarbor,
                is_applicable_corporation: false,
                average_three_year_afsi_cents: avg_3y_afsi,
                afsi_after_nol_cents: input.current_year_afsi_cents,
                tentative_minimum_tax_cents: 0,
                regular_corporate_tax_cents: input.regular_corporate_tax_cents,
                camt_owed_cents: 0,
                camt_credit_carryforward_to_future_year_cents: 0,
                recommended_actions: actions,
                citation: "26 U.S.C. § 59(k)(2)",
                notes,
            };
        }
    } else {
        is_applicable = avg_3y_afsi > APPLICABLE_CORPORATION_AFSI_THRESHOLD_CENTS;
    }

    if !is_applicable {
        notes.push(format!(
            "Three-year average AFSI of {} cents at or below $1B applicable-corporation \
             threshold per § 59(k)(1); CAMT inapplicable for current tax year. Re-test \
             annually; once status is established it persists per Notice 2023-7 absent \
             extraordinary contraction.",
            avg_3y_afsi
        ));
        return Section56aResult {
            severity: CamtSeverity::BelowApplicableCorporationThreshold,
            is_applicable_corporation: false,
            average_three_year_afsi_cents: avg_3y_afsi,
            afsi_after_nol_cents: input.current_year_afsi_cents,
            tentative_minimum_tax_cents: 0,
            regular_corporate_tax_cents: input.regular_corporate_tax_cents,
            camt_owed_cents: 0,
            camt_credit_carryforward_to_future_year_cents: 0,
            recommended_actions: actions,
            citation: "26 U.S.C. § 59(k)(1)",
            notes,
        };
    }

    let nol_limit = i128::from(input.current_year_afsi_cents.max(0))
        * i128::from(FSNOL_LIMITATION_BPS)
        / 10_000;
    let nol_used = (i128::from(input.camt_nol_carryforward_cents)).min(nol_limit) as i64;
    let afsi_after_nol = input.current_year_afsi_cents.saturating_sub(nol_used);

    let tentative_min_tax: u64 = if afsi_after_nol > 0 {
        let gross_min_tax: u128 = (afsi_after_nol as u128) * u128::from(CAMT_RATE_BPS) / 10_000;
        (gross_min_tax.saturating_sub(u128::from(input.camt_ftc_cents))) as u64
    } else {
        0
    };

    let camt_owed = tentative_min_tax.saturating_sub(input.regular_corporate_tax_cents);
    let severity = if camt_owed > 0 {
        CamtSeverity::ApplicableCorporationCamtOwed
    } else {
        CamtSeverity::ApplicableCorporationCamtNotOwed
    };

    actions.push(format!(
        "File Form 4626 Alternative Minimum Tax — Corporations attached to Form 1120 for tax \
         year {}; report CAMT computation: AFSI {} cents minus FSNOL used {} cents = {} cents \
         times 15% = {} cents tentative minimum tax minus CAMT FTC {} cents = {} cents net \
         tentative; regular tax of {} cents; CAMT owed = max(0, tentative minus regular) = {} \
         cents.",
        input.taxable_year,
        input.current_year_afsi_cents,
        nol_used,
        afsi_after_nol,
        afsi_after_nol.saturating_mul(15) / 100,
        input.camt_ftc_cents,
        tentative_min_tax,
        input.regular_corporate_tax_cents,
        camt_owed
    ));
    if camt_owed > 0 {
        actions.push(
            "CAMT paid generates indefinite credit carryforward under § 53(c)-(d) against \
             future regular tax liability when regular exceeds tentative minimum; preserve in \
             corporate tax records for indefinite use."
                .to_string(),
        );
    }

    notes.push(format!(
        "Three-year average AFSI of {} cents exceeds $1B applicable-corporation threshold per \
         § 59(k)(1); 15% CAMT rate applies to current-year AFSI after § 56A(c) adjustments \
         (federal tax back-out, defined benefit pension, qualified depreciation via § 168 \
         not book, cooperative dividends, etc.). FSNOL deduction limited to 80% of AFSI per \
         § 56A(d).",
        avg_3y_afsi
    ));
    notes.push(
        "Coordination with [[section_4501]] (1% stock buyback excise — same IRA 2022 package), \
         [[section_481]] (accounting method change AFSI restatement), [[section_55]] (general \
         AMT framework — historic § 55(b)(1) AMT for individuals + corporations regime now \
         replaced by § 55(b)(2) CAMT for corporations), [[section_53]] (AMT credit \
         carryforward against future regular tax), [[section_38]] (general business credit \
         75% AFSI limitation against tentative minimum tax), [[section_59a]] (BEAT — Base \
         Erosion Anti-Abuse Tax — coordinates for inbound FPMG members)."
            .to_string(),
    );

    Section56aResult {
        severity,
        is_applicable_corporation: true,
        average_three_year_afsi_cents: avg_3y_afsi,
        afsi_after_nol_cents: afsi_after_nol,
        tentative_minimum_tax_cents: tentative_min_tax,
        regular_corporate_tax_cents: input.regular_corporate_tax_cents,
        camt_owed_cents: camt_owed,
        camt_credit_carryforward_to_future_year_cents: camt_owed,
        recommended_actions: actions,
        citation:
            "26 U.S.C. § 55(b)(2); § 56A(a)-(d); § 59(k)(1)-(2); Notice 2023-7; Notice 2025-46",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> Section56aInput {
        Section56aInput {
            entity_type: EntityType::DomesticCCorporation,
            taxable_year: 2024,
            afsi_prior_year_1_cents: 200_000_000_000,
            afsi_prior_year_2_cents: 150_000_000_000,
            afsi_prior_year_3_cents: 100_000_000_000,
            current_year_afsi_cents: 200_000_000_000,
            fpmg_total_afsi_cents: 0,
            camt_ftc_cents: 0,
            regular_corporate_tax_cents: 21_000_000_000,
            camt_nol_carryforward_cents: 0,
        }
    }

    #[test]
    fn pre_2023_not_applicable() {
        let mut i = baseline();
        i.taxable_year = 2022;
        let r = check(&i);
        assert!(matches!(r.severity, CamtSeverity::PreEffectiveDate));
        assert_eq!(r.camt_owed_cents, 0);
    }

    #[test]
    fn effective_year_pins_2023() {
        assert_eq!(CAMT_EFFECTIVE_TAX_YEAR, 2023);
    }

    #[test]
    fn applicable_corporation_threshold_pins_1_billion() {
        assert_eq!(APPLICABLE_CORPORATION_AFSI_THRESHOLD_CENTS, 100_000_000_000);
    }

    #[test]
    fn fpmg_us_member_threshold_pins_100_million() {
        assert_eq!(FPMG_US_MEMBER_AFSI_THRESHOLD_CENTS, 10_000_000_000);
    }

    #[test]
    fn camt_rate_pins_15_percent() {
        assert_eq!(CAMT_RATE_BPS, 1_500);
    }

    #[test]
    fn fsnol_limitation_pins_80_percent() {
        assert_eq!(FSNOL_LIMITATION_BPS, 8_000);
    }

    #[test]
    fn s_corp_categorically_exempt() {
        let mut i = baseline();
        i.entity_type = EntityType::SCorporation;
        let r = check(&i);
        assert!(matches!(r.severity, CamtSeverity::CategoricallyExempt));
        assert!(!r.is_applicable_corporation);
        assert_eq!(r.camt_owed_cents, 0);
        assert!(r.notes.iter().any(|n| n.contains("S corporations")));
    }

    #[test]
    fn ric_categorically_exempt() {
        let mut i = baseline();
        i.entity_type = EntityType::RegulatedInvestmentCompany;
        let r = check(&i);
        assert!(matches!(r.severity, CamtSeverity::CategoricallyExempt));
    }

    #[test]
    fn reit_categorically_exempt() {
        let mut i = baseline();
        i.entity_type = EntityType::RealEstateInvestmentTrust;
        let r = check(&i);
        assert!(matches!(r.severity, CamtSeverity::CategoricallyExempt));
    }

    #[test]
    fn foreign_corp_no_us_business_categorically_exempt() {
        let mut i = baseline();
        i.entity_type = EntityType::ForeignCorporationNoUsBusiness;
        let r = check(&i);
        assert!(matches!(r.severity, CamtSeverity::CategoricallyExempt));
    }

    #[test]
    fn below_1b_threshold_not_applicable() {
        let mut i = baseline();
        i.afsi_prior_year_1_cents = 50_000_000_000;
        i.afsi_prior_year_2_cents = 80_000_000_000;
        i.afsi_prior_year_3_cents = 90_000_000_000;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            CamtSeverity::BelowApplicableCorporationThreshold
        ));
        assert!(!r.is_applicable_corporation);
        assert_eq!(r.average_three_year_afsi_cents, 73_333_333_333);
    }

    #[test]
    fn at_exactly_1b_threshold_not_applicable_strict_greater_than() {
        let mut i = baseline();
        i.afsi_prior_year_1_cents = 100_000_000_000;
        i.afsi_prior_year_2_cents = 100_000_000_000;
        i.afsi_prior_year_3_cents = 100_000_000_000;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            CamtSeverity::BelowApplicableCorporationThreshold
        ));
    }

    #[test]
    fn one_dollar_over_threshold_applicable() {
        let mut i = baseline();
        i.afsi_prior_year_1_cents = 100_000_000_001;
        i.afsi_prior_year_2_cents = 100_000_000_001;
        i.afsi_prior_year_3_cents = 100_000_000_001;
        let r = check(&i);
        assert!(r.is_applicable_corporation);
    }

    #[test]
    fn fpmg_aggregate_above_1b_us_member_below_100m_safe_harbor() {
        let mut i = baseline();
        i.entity_type = EntityType::UsMemberForeignParentedGroup;
        i.afsi_prior_year_1_cents = 5_000_000_000;
        i.afsi_prior_year_2_cents = 5_000_000_000;
        i.afsi_prior_year_3_cents = 5_000_000_000;
        i.fpmg_total_afsi_cents = 500_000_000_000;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            CamtSeverity::FpmgBelow100MUsAfsiSafeHarbor
        ));
        assert!(!r.is_applicable_corporation);
    }

    #[test]
    fn fpmg_aggregate_above_1b_us_member_at_100m_applicable() {
        let mut i = baseline();
        i.entity_type = EntityType::UsMemberForeignParentedGroup;
        i.afsi_prior_year_1_cents = FPMG_US_MEMBER_AFSI_THRESHOLD_CENTS;
        i.afsi_prior_year_2_cents = FPMG_US_MEMBER_AFSI_THRESHOLD_CENTS;
        i.afsi_prior_year_3_cents = FPMG_US_MEMBER_AFSI_THRESHOLD_CENTS;
        i.fpmg_total_afsi_cents = 500_000_000_000;
        let r = check(&i);
        assert!(r.is_applicable_corporation);
    }

    #[test]
    fn fpmg_below_1b_aggregate_not_applicable() {
        let mut i = baseline();
        i.entity_type = EntityType::UsMemberForeignParentedGroup;
        i.afsi_prior_year_1_cents = 20_000_000_000;
        i.afsi_prior_year_2_cents = 20_000_000_000;
        i.afsi_prior_year_3_cents = 20_000_000_000;
        i.fpmg_total_afsi_cents = 50_000_000_000;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            CamtSeverity::BelowApplicableCorporationThreshold
        ));
    }

    #[test]
    fn applicable_corp_camt_owed_when_tentative_exceeds_regular() {
        let mut i = baseline();
        i.current_year_afsi_cents = 200_000_000_000;
        i.regular_corporate_tax_cents = 21_000_000_000;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            CamtSeverity::ApplicableCorporationCamtOwed
        ));
        let expected_tentative = 200_000_000_000u64 * 15 / 100;
        assert_eq!(r.tentative_minimum_tax_cents, expected_tentative);
        assert_eq!(r.camt_owed_cents, expected_tentative - 21_000_000_000);
    }

    #[test]
    fn applicable_corp_camt_not_owed_when_regular_exceeds_tentative() {
        let mut i = baseline();
        i.current_year_afsi_cents = 200_000_000_000;
        i.regular_corporate_tax_cents = 50_000_000_000;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            CamtSeverity::ApplicableCorporationCamtNotOwed
        ));
        assert_eq!(r.camt_owed_cents, 0);
    }

    #[test]
    fn ftc_reduces_tentative_minimum_tax() {
        let mut i = baseline();
        i.current_year_afsi_cents = 200_000_000_000;
        i.camt_ftc_cents = 5_000_000_000;
        let r = check(&i);
        let gross = 200_000_000_000u64 * 15 / 100;
        assert_eq!(r.tentative_minimum_tax_cents, gross - 5_000_000_000);
    }

    #[test]
    fn fsnol_limited_to_80_pct_of_afsi() {
        let mut i = baseline();
        i.current_year_afsi_cents = 100_000_000_000;
        i.camt_nol_carryforward_cents = 100_000_000_000;
        let r = check(&i);
        let expected_nol_used = 80_000_000_000i64;
        assert_eq!(r.afsi_after_nol_cents, 100_000_000_000 - expected_nol_used);
    }

    #[test]
    fn fsnol_fully_used_when_below_80_pct_limit() {
        let mut i = baseline();
        i.current_year_afsi_cents = 100_000_000_000;
        i.camt_nol_carryforward_cents = 30_000_000_000;
        let r = check(&i);
        assert_eq!(r.afsi_after_nol_cents, 70_000_000_000);
    }

    #[test]
    fn action_includes_form_4626_filing() {
        let i = baseline();
        let r = check(&i);
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("Form 4626")));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("Form 1120")));
    }

    #[test]
    fn coordination_note_references_4501_and_53_and_38_and_59a() {
        let i = baseline();
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("section_4501")));
        assert!(r.notes.iter().any(|n| n.contains("section_53")));
        assert!(r.notes.iter().any(|n| n.contains("section_38")));
        assert!(r.notes.iter().any(|n| n.contains("section_59a")));
    }

    #[test]
    fn citation_pins_55_b_2_and_56a_and_59k() {
        let i = baseline();
        let r = check(&i);
        assert!(r.citation.contains("§ 55(b)(2)"));
        assert!(r.citation.contains("§ 56A(a)-(d)"));
        assert!(r.citation.contains("§ 59(k)(1)-(2)"));
        assert!(r.citation.contains("Notice 2023-7"));
        assert!(r.citation.contains("Notice 2025-46"));
    }

    #[test]
    fn camt_credit_carryforward_equals_camt_owed() {
        let i = baseline();
        let r = check(&i);
        assert_eq!(
            r.camt_credit_carryforward_to_future_year_cents,
            r.camt_owed_cents
        );
    }

    #[test]
    fn realistic_amazon_10b_afsi_15b_regular_tax_no_camt() {
        let mut i = baseline();
        i.current_year_afsi_cents = 10_000_000_000_000;
        i.regular_corporate_tax_cents = 1_750_000_000_000;
        let r = check(&i);
        let tentative = 10_000_000_000_000u64 * 15 / 100;
        assert_eq!(r.tentative_minimum_tax_cents, tentative);
        assert!(r.camt_owed_cents < tentative);
    }

    #[test]
    fn realistic_meta_low_us_tax_camt_kicks_in() {
        let mut i = baseline();
        i.current_year_afsi_cents = 5_000_000_000_000;
        i.regular_corporate_tax_cents = 100_000_000_000;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            CamtSeverity::ApplicableCorporationCamtOwed
        ));
        let tentative = 5_000_000_000_000u64 * 15 / 100;
        assert_eq!(r.camt_owed_cents, tentative - 100_000_000_000);
    }

    #[test]
    fn negative_current_afsi_zero_tentative_tax() {
        let mut i = baseline();
        i.current_year_afsi_cents = -50_000_000_000;
        let r = check(&i);
        assert_eq!(r.tentative_minimum_tax_cents, 0);
        assert_eq!(r.camt_owed_cents, 0);
    }

    #[test]
    fn zero_afsi_zero_camt() {
        let mut i = baseline();
        i.current_year_afsi_cents = 0;
        let r = check(&i);
        assert_eq!(r.tentative_minimum_tax_cents, 0);
    }

    #[test]
    fn extreme_afsi_does_not_overflow() {
        let mut i = baseline();
        i.current_year_afsi_cents = i64::MAX / 1000;
        let r = check(&i);
        let _ = r.camt_owed_cents;
    }

    #[test]
    fn ftc_exceeding_gross_tentative_clamps_at_zero() {
        let mut i = baseline();
        i.current_year_afsi_cents = 200_000_000_000;
        i.camt_ftc_cents = u64::MAX / 2;
        let r = check(&i);
        assert_eq!(r.tentative_minimum_tax_cents, 0);
    }

    #[test]
    fn nol_does_not_reduce_negative_afsi() {
        let mut i = baseline();
        i.current_year_afsi_cents = -10_000_000_000;
        i.camt_nol_carryforward_cents = 50_000_000_000;
        let r = check(&i);
        assert!(r.afsi_after_nol_cents <= 0);
    }
}
