//! IRC § 951A — Global Intangible Low-Taxed Income (GILTI) plus its post-
//! OBBBA successor regime "Net CFC Tested Income" (NCTI). Enacted by Tax Cuts
//! and Jobs Act of 2017 (Pub. L. 115-97 § 14201) effective for taxable years
//! of foreign corporations beginning after December 31, 2017.
//!
//! Pre-2026 GILTI regime: a US shareholder of a Controlled Foreign Corporation
//! (CFC) must include in gross income its pro rata share of the CFC's "tested
//! income" reduced by 10% of the CFC's "Qualified Business Asset Investment"
//! (QBAI) basis, further reduced by allocable interest expense. GILTI flows
//! to a separate § 904(d) FTC basket; § 250 allows a 50% deduction (37.5%
//! permanent post-2026 sunset under TCJA) yielding effective US rate of
//! approximately 10.5%. § 960(d) allows deemed-paid FTC at 80% (20% haircut).
//!
//! Post-2026 NCTI regime under One Big Beautiful Bill Act (OBBBA), Pub. L.
//! 119-21 signed July 4, 2025: effective for taxable years beginning after
//! December 31, 2025, (1) GILTI is renamed Net CFC Tested Income (NCTI); (2)
//! QBAI deduction is REPEALED — full tested income is includible without
//! tangible-asset offset; (3) § 250 deduction is PERMANENTLY set to 40%
//! (yielding 12.6% effective US rate at 21% corporate rate); (4) § 960(d)
//! FTC haircut is reduced from 20% to 10% (90% FTCs allowed); (5) interest
//! expense and R&E expense are no longer allocated to the GILTI / NCTI
//! basket per § 864(e) clarification.
//!
//! § 951A applies only to US shareholders (10% direct or indirect) of CFCs
//! (foreign corp with greater-than-50% US shareholder ownership). Domestic
//! C corporations claim § 250 deduction; individuals + S-corps + partnerships
//! cannot directly take § 250 unless via § 962 election to be taxed as a
//! domestic C corporation.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UsShareholderType {
    DomesticCCorporation,
    /// Individual / S-corp / partnership; cannot directly take § 250 deduction
    /// unless § 962 election to be taxed at corporate rate.
    IndividualOrPassThroughNoSection962,
    IndividualOrPassThroughWithSection962Election,
    /// Not a US shareholder for § 951(b) purposes — less than 10% direct or
    /// indirect ownership of CFC.
    LessThan10PctNotUsShareholder,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    NotUsShareholderNoInclusion,
    NotACfcNoInclusion,
    Pre2026GiltiInclusionWithQbai,
    Post2026NctiInclusionNoQbai,
    TestedLossNoCurrentInclusion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section951aInput {
    pub us_shareholder_type: UsShareholderType,
    pub taxable_year: i32,
    /// Whether the foreign corp meets the § 957(a) CFC definition (greater-
    /// than-50% US shareholder ownership).
    pub foreign_corp_is_cfc: bool,
    /// US shareholder's pro rata share of CFC tested income in cents
    /// (post § 951A(c)(2)(A) exclusions).
    pub pro_rata_tested_income_cents: i64,
    /// US shareholder's pro rata share of CFC qualified business asset
    /// investment (QBAI) basis under § 951A(d) in cents — used pre-2026
    /// for NDTIR deduction.
    pub pro_rata_qbai_cents: u64,
    /// US shareholder's pro rata share of CFC tested interest expense in
    /// cents (reduces NDTIR pre-2026).
    pub pro_rata_tested_interest_expense_cents: u64,
    /// US shareholder's pro rata share of allocable foreign tax of CFC in
    /// cents (for § 960(d) deemed-paid FTC computation).
    pub pro_rata_allocable_foreign_tax_cents: u64,
    /// US corporate tax rate in basis points (21% = 2_100).
    pub corporate_tax_rate_bps: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section951aResult {
    pub severity: Severity,
    pub is_post_obbba_year: bool,
    pub ndtir_deduction_cents: u64,
    pub gilti_or_ncti_inclusion_cents: u64,
    pub section_250_deduction_pct_bps: u32,
    pub section_250_deduction_amount_cents: u64,
    pub taxable_inclusion_after_section_250_cents: u64,
    pub gross_us_tax_cents: u64,
    pub section_960d_ftc_rate_bps: u32,
    pub section_960d_ftc_cents: u64,
    pub net_us_tax_cents: u64,
    pub effective_us_rate_bps: u32,
    pub recommended_actions: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub const NDTIR_QBAI_RATE_BPS: u32 = 1_000;
pub const PRE_OBBBA_SECTION_250_DEDUCTION_BPS: u32 = 5_000;
pub const POST_OBBBA_SECTION_250_DEDUCTION_BPS: u32 = 4_000;
pub const PRE_OBBBA_FTC_HAIRCUT_BPS: u32 = 2_000;
pub const POST_OBBBA_FTC_HAIRCUT_BPS: u32 = 1_000;
pub const OBBBA_NCTI_EFFECTIVE_YEAR: i32 = 2026;
pub const GILTI_TCJA_EFFECTIVE_YEAR: i32 = 2018;

pub fn check(input: &Section951aInput) -> Section951aResult {
    let mut actions: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();
    let is_post_obbba_year = input.taxable_year >= OBBBA_NCTI_EFFECTIVE_YEAR;

    if input.taxable_year < GILTI_TCJA_EFFECTIVE_YEAR {
        notes.push(format!(
            "§ 951A GILTI / NCTI effective for taxable years of foreign corporations \
             beginning after December 31, {} per Pub. L. 115-97 § 14201; pre-{} years \
             governed by historic Subpart F regime only ([[section_951]]).",
            GILTI_TCJA_EFFECTIVE_YEAR - 1,
            GILTI_TCJA_EFFECTIVE_YEAR
        ));
        return empty_result(
            Severity::NotApplicable,
            input,
            is_post_obbba_year,
            actions,
            notes,
            "26 U.S.C. § 951A; Pub. L. 115-97 § 14201; Pub. L. 119-21 (OBBBA)",
        );
    }

    if matches!(
        input.us_shareholder_type,
        UsShareholderType::LessThan10PctNotUsShareholder
    ) {
        notes.push(
            "Less-than-10% shareholder is NOT a US shareholder under § 951(b); no GILTI / \
             NCTI inclusion required. Portfolio dividends from CFC taxed under § 1248 / § \
             865 rules on disposition only."
                .to_string(),
        );
        return empty_result(
            Severity::NotUsShareholderNoInclusion,
            input,
            is_post_obbba_year,
            actions,
            notes,
            "26 U.S.C. § 951(b); § 957(a)",
        );
    }

    if !input.foreign_corp_is_cfc {
        notes.push(
            "Foreign corp is NOT a CFC under § 957(a) (not more than 50% US shareholder \
             ownership); no GILTI / NCTI inclusion. Consider PFIC analysis under [[section_\
             1297]] if foreign corp meets income or asset PFIC tests."
                .to_string(),
        );
        return empty_result(
            Severity::NotACfcNoInclusion,
            input,
            is_post_obbba_year,
            actions,
            notes,
            "26 U.S.C. § 957(a); coord. § 1297",
        );
    }

    if input.pro_rata_tested_income_cents <= 0 {
        notes.push(
            "CFC tested income is zero or tested loss; no current-year GILTI / NCTI \
             inclusion. Tested losses do NOT carry forward at the CFC level but may offset \
             other CFCs' tested income at the US shareholder level under § 951A(c)(2)(B) \
             same-year aggregation."
                .to_string(),
        );
        return empty_result(
            Severity::TestedLossNoCurrentInclusion,
            input,
            is_post_obbba_year,
            actions,
            notes,
            "26 U.S.C. § 951A(c)(2)(A)-(B)",
        );
    }

    let ndtir = if is_post_obbba_year {
        0u64
    } else {
        let gross_qbai_return: u128 =
            u128::from(input.pro_rata_qbai_cents) * u128::from(NDTIR_QBAI_RATE_BPS) / 10_000;
        let net_after_interest = gross_qbai_return
            .saturating_sub(u128::from(input.pro_rata_tested_interest_expense_cents));
        net_after_interest as u64
    };

    let tested_income_u64 = input.pro_rata_tested_income_cents.max(0) as u64;
    let inclusion = tested_income_u64.saturating_sub(ndtir);

    let section_250_pct = match (input.us_shareholder_type, is_post_obbba_year) {
        (UsShareholderType::DomesticCCorporation, true) => POST_OBBBA_SECTION_250_DEDUCTION_BPS,
        (UsShareholderType::DomesticCCorporation, false) => PRE_OBBBA_SECTION_250_DEDUCTION_BPS,
        (UsShareholderType::IndividualOrPassThroughWithSection962Election, true) => {
            POST_OBBBA_SECTION_250_DEDUCTION_BPS
        }
        (UsShareholderType::IndividualOrPassThroughWithSection962Election, false) => {
            PRE_OBBBA_SECTION_250_DEDUCTION_BPS
        }
        (UsShareholderType::IndividualOrPassThroughNoSection962, _) => 0,
        (UsShareholderType::LessThan10PctNotUsShareholder, _) => 0,
    };

    let section_250_deduction_amount: u64 =
        (u128::from(inclusion) * u128::from(section_250_pct) / 10_000) as u64;
    let taxable_after_250 = inclusion.saturating_sub(section_250_deduction_amount);
    let gross_us_tax: u64 =
        (u128::from(taxable_after_250) * u128::from(input.corporate_tax_rate_bps) / 10_000) as u64;

    let ftc_rate_bps = if is_post_obbba_year {
        10_000 - POST_OBBBA_FTC_HAIRCUT_BPS
    } else {
        10_000 - PRE_OBBBA_FTC_HAIRCUT_BPS
    };
    let raw_ftc: u64 = (u128::from(input.pro_rata_allocable_foreign_tax_cents)
        * u128::from(ftc_rate_bps)
        / 10_000) as u64;
    let ftc_cap = gross_us_tax;
    let ftc_actual = raw_ftc.min(ftc_cap);
    let net_us_tax = gross_us_tax.saturating_sub(ftc_actual);

    let effective_rate_bps: u32 = if tested_income_u64 > 0 {
        let ratio: u128 = u128::from(net_us_tax) * 10_000 / u128::from(tested_income_u64);
        ratio.min(u128::from(u32::MAX)) as u32
    } else {
        0
    };

    let severity = if is_post_obbba_year {
        Severity::Post2026NctiInclusionNoQbai
    } else {
        Severity::Pre2026GiltiInclusionWithQbai
    };

    if is_post_obbba_year {
        actions.push(format!(
            "Post-OBBBA NCTI inclusion for tax year {}: full tested income of {} cents \
             includible (QBAI repealed); 40% § 250 deduction = {} cents; taxable inclusion = \
             {} cents; gross 21% US tax = {} cents; § 960(d) FTC at 90% of allocable foreign \
             tax = {} cents (capped at gross tax); net US tax = {} cents; effective US rate = \
             {} bps. File Form 5471 plus Form 8992 (US Shareholder Calculation of GILTI / NCTI).",
            input.taxable_year,
            tested_income_u64,
            section_250_deduction_amount,
            taxable_after_250,
            gross_us_tax,
            ftc_actual,
            net_us_tax,
            effective_rate_bps
        ));
    } else {
        actions.push(format!(
            "Pre-OBBBA GILTI inclusion for tax year {}: tested income {} cents minus NDTIR of \
             {} cents (10% QBAI minus tested interest expense) = {} cents inclusion; 50% § \
             250 deduction = {} cents; taxable inclusion = {} cents; gross 21% US tax = {} \
             cents; § 960(d) FTC at 80% of allocable foreign tax = {} cents (capped at gross \
             tax); net US tax = {} cents; effective US rate = {} bps. File Form 5471 plus \
             Form 8992.",
            input.taxable_year,
            tested_income_u64,
            ndtir,
            inclusion,
            section_250_deduction_amount,
            taxable_after_250,
            gross_us_tax,
            ftc_actual,
            net_us_tax,
            effective_rate_bps
        ));
    }

    if matches!(
        input.us_shareholder_type,
        UsShareholderType::IndividualOrPassThroughNoSection962
    ) {
        actions.push(
            "Individual / S-corp / partnership US shareholder without § 962 election cannot \
             claim § 250 deduction directly; entire GILTI / NCTI inclusion taxable at \
             ordinary individual rates (up to 37%). Consider § 962 election to be taxed as \
             domestic C corporation on Subpart F + GILTI / NCTI income at 21% corporate rate \
             plus claim § 250 deduction and § 960(d) FTC."
                .to_string(),
        );
    }

    notes.push(
        "Coordination with [[section_250]] (FDII + GILTI / NCTI deduction — 40% post-OBBBA / \
         37.5% scheduled / 50% pre-2026 transitional), [[section_59a]] (BEAT — separate \
         base-erosion regime for foreign-related-party deductions), [[section_56a]] (CAMT — \
         CFC income flows through AFSI), [[section_988]] (foreign-currency gain/loss on CFC \
         distributions), [[section_988]] forex, [[section_1297]] (PFIC mutually exclusive \
         with CFC GILTI), [[section_911]] (foreign earned income exclusion — separate \
         individual regime)."
            .to_string(),
    );

    Section951aResult {
        severity,
        is_post_obbba_year,
        ndtir_deduction_cents: ndtir,
        gilti_or_ncti_inclusion_cents: inclusion,
        section_250_deduction_pct_bps: section_250_pct,
        section_250_deduction_amount_cents: section_250_deduction_amount,
        taxable_inclusion_after_section_250_cents: taxable_after_250,
        gross_us_tax_cents: gross_us_tax,
        section_960d_ftc_rate_bps: ftc_rate_bps,
        section_960d_ftc_cents: ftc_actual,
        net_us_tax_cents: net_us_tax,
        effective_us_rate_bps: effective_rate_bps,
        recommended_actions: actions,
        citation: if is_post_obbba_year {
            "26 U.S.C. § 951A (post-OBBBA NCTI); § 250(a)(1)(B); § 960(d); Pub. L. 119-21"
        } else {
            "26 U.S.C. § 951A (pre-OBBBA GILTI); § 951A(b)(2) NDTIR; § 250(a)(1)(B); § 960(d); Pub. L. 115-97 § 14201"
        },
        notes,
    }
}

fn empty_result(
    severity: Severity,
    input: &Section951aInput,
    is_post_obbba_year: bool,
    recommended_actions: Vec<String>,
    notes: Vec<String>,
    citation: &'static str,
) -> Section951aResult {
    Section951aResult {
        severity,
        is_post_obbba_year,
        ndtir_deduction_cents: 0,
        gilti_or_ncti_inclusion_cents: 0,
        section_250_deduction_pct_bps: 0,
        section_250_deduction_amount_cents: 0,
        taxable_inclusion_after_section_250_cents: 0,
        gross_us_tax_cents: 0,
        section_960d_ftc_rate_bps: 0,
        section_960d_ftc_cents: 0,
        net_us_tax_cents: 0,
        effective_us_rate_bps: 0,
        recommended_actions,
        citation,
        notes: {
            let mut n = notes;
            let _ = input;
            n.push(
                "Coordination with [[section_250]] (FDII / GILTI / NCTI deduction), \
                 [[section_59a]] (BEAT), [[section_56a]] (CAMT), [[section_1297]] (PFIC \
                 mutually exclusive), [[section_911]] (foreign earned income exclusion)."
                    .to_string(),
            );
            n
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> Section951aInput {
        Section951aInput {
            us_shareholder_type: UsShareholderType::DomesticCCorporation,
            taxable_year: 2024,
            foreign_corp_is_cfc: true,
            pro_rata_tested_income_cents: 100_000_000_00,
            pro_rata_qbai_cents: 50_000_000_00,
            pro_rata_tested_interest_expense_cents: 0,
            pro_rata_allocable_foreign_tax_cents: 0,
            corporate_tax_rate_bps: 2_100,
        }
    }

    #[test]
    fn pre_2018_not_applicable() {
        let mut i = baseline();
        i.taxable_year = 2017;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::NotApplicable));
        assert_eq!(r.gilti_or_ncti_inclusion_cents, 0);
    }

    #[test]
    fn less_than_10_pct_not_us_shareholder() {
        let mut i = baseline();
        i.us_shareholder_type = UsShareholderType::LessThan10PctNotUsShareholder;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::NotUsShareholderNoInclusion));
    }

    #[test]
    fn foreign_corp_not_a_cfc_no_inclusion() {
        let mut i = baseline();
        i.foreign_corp_is_cfc = false;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::NotACfcNoInclusion));
        assert!(r.notes.iter().any(|n| n.contains("section_1297")));
    }

    #[test]
    fn tested_loss_no_current_inclusion() {
        let mut i = baseline();
        i.pro_rata_tested_income_cents = -10_000_000_00;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::TestedLossNoCurrentInclusion));
        assert!(r.notes.iter().any(|n| n.contains("§ 951A(c)(2)(B)")));
    }

    #[test]
    fn pre_obbba_gilti_with_qbai_deduction() {
        let mut i = baseline();
        i.taxable_year = 2024;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::Pre2026GiltiInclusionWithQbai
        ));
        assert!(!r.is_post_obbba_year);
        let expected_ndtir = 50_000_000_00u64 * 1_000 / 10_000;
        assert_eq!(r.ndtir_deduction_cents, expected_ndtir);
        let expected_inclusion = 100_000_000_00u64 - expected_ndtir;
        assert_eq!(r.gilti_or_ncti_inclusion_cents, expected_inclusion);
    }

    #[test]
    fn pre_obbba_section_250_deduction_pins_50_pct() {
        let i = baseline();
        let r = check(&i);
        assert_eq!(r.section_250_deduction_pct_bps, 5_000);
    }

    #[test]
    fn post_obbba_section_250_deduction_pins_40_pct() {
        let mut i = baseline();
        i.taxable_year = 2026;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::Post2026NctiInclusionNoQbai));
        assert!(r.is_post_obbba_year);
        assert_eq!(r.section_250_deduction_pct_bps, 4_000);
        assert_eq!(r.ndtir_deduction_cents, 0);
    }

    #[test]
    fn pre_obbba_ftc_haircut_pins_20_pct() {
        let i = baseline();
        let r = check(&i);
        assert_eq!(r.section_960d_ftc_rate_bps, 8_000);
    }

    #[test]
    fn post_obbba_ftc_haircut_pins_10_pct() {
        let mut i = baseline();
        i.taxable_year = 2026;
        let r = check(&i);
        assert_eq!(r.section_960d_ftc_rate_bps, 9_000);
    }

    #[test]
    fn ndtir_reduced_by_tested_interest_expense() {
        let mut i = baseline();
        i.pro_rata_tested_interest_expense_cents = 2_000_000_00;
        let r = check(&i);
        let gross_qbai_return = 50_000_000_00u64 * 1_000 / 10_000;
        let expected_ndtir = gross_qbai_return - 2_000_000_00;
        assert_eq!(r.ndtir_deduction_cents, expected_ndtir);
    }

    #[test]
    fn ndtir_saturates_at_zero_when_interest_exceeds_qbai_return() {
        let mut i = baseline();
        i.pro_rata_qbai_cents = 10_000_000_00;
        i.pro_rata_tested_interest_expense_cents = 100_000_000_00;
        let r = check(&i);
        assert_eq!(r.ndtir_deduction_cents, 0);
    }

    #[test]
    fn realistic_apple_cfc_pre_obbba_effective_rate_around_10_5_pct() {
        let mut i = baseline();
        i.taxable_year = 2024;
        i.pro_rata_tested_income_cents = 100_000_000_00;
        i.pro_rata_qbai_cents = 0;
        i.pro_rata_allocable_foreign_tax_cents = 0;
        let r = check(&i);
        assert_eq!(r.effective_us_rate_bps, 1_050);
    }

    #[test]
    fn realistic_apple_cfc_post_obbba_effective_rate_around_12_6_pct() {
        let mut i = baseline();
        i.taxable_year = 2026;
        i.pro_rata_tested_income_cents = 100_000_000_00;
        i.pro_rata_qbai_cents = 0;
        i.pro_rata_allocable_foreign_tax_cents = 0;
        let r = check(&i);
        assert_eq!(r.effective_us_rate_bps, 1_260);
    }

    #[test]
    fn ftc_capped_at_gross_us_tax() {
        let mut i = baseline();
        i.pro_rata_allocable_foreign_tax_cents = u64::MAX / 2;
        let r = check(&i);
        assert!(r.section_960d_ftc_cents <= r.gross_us_tax_cents);
    }

    #[test]
    fn ftc_zero_when_no_foreign_tax() {
        let i = baseline();
        let r = check(&i);
        assert_eq!(r.section_960d_ftc_cents, 0);
    }

    #[test]
    fn individual_no_962_no_section_250_deduction() {
        let mut i = baseline();
        i.us_shareholder_type = UsShareholderType::IndividualOrPassThroughNoSection962;
        let r = check(&i);
        assert_eq!(r.section_250_deduction_pct_bps, 0);
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("§ 962 election")));
    }

    #[test]
    fn individual_with_962_election_gets_section_250() {
        let mut i = baseline();
        i.us_shareholder_type = UsShareholderType::IndividualOrPassThroughWithSection962Election;
        let r = check(&i);
        assert_eq!(r.section_250_deduction_pct_bps, 5_000);
    }

    #[test]
    fn obbba_effective_year_pins_2026() {
        assert_eq!(OBBBA_NCTI_EFFECTIVE_YEAR, 2026);
    }

    #[test]
    fn gilti_tcja_effective_year_pins_2018() {
        assert_eq!(GILTI_TCJA_EFFECTIVE_YEAR, 2018);
    }

    #[test]
    fn ndtir_qbai_rate_pins_10_pct() {
        assert_eq!(NDTIR_QBAI_RATE_BPS, 1_000);
    }

    #[test]
    fn pre_obbba_section_250_deduction_constant_pins_50_pct() {
        assert_eq!(PRE_OBBBA_SECTION_250_DEDUCTION_BPS, 5_000);
    }

    #[test]
    fn post_obbba_section_250_deduction_constant_pins_40_pct() {
        assert_eq!(POST_OBBBA_SECTION_250_DEDUCTION_BPS, 4_000);
    }

    #[test]
    fn pre_obbba_ftc_haircut_constant_pins_20_pct() {
        assert_eq!(PRE_OBBBA_FTC_HAIRCUT_BPS, 2_000);
    }

    #[test]
    fn post_obbba_ftc_haircut_constant_pins_10_pct() {
        assert_eq!(POST_OBBBA_FTC_HAIRCUT_BPS, 1_000);
    }

    #[test]
    fn citation_pre_obbba_pins_pub_l_115_97() {
        let i = baseline();
        let r = check(&i);
        assert!(r.citation.contains("Pub. L. 115-97 § 14201"));
        assert!(r.citation.contains("pre-OBBBA GILTI"));
    }

    #[test]
    fn citation_post_obbba_pins_pub_l_119_21() {
        let mut i = baseline();
        i.taxable_year = 2026;
        let r = check(&i);
        assert!(r.citation.contains("Pub. L. 119-21"));
        assert!(r.citation.contains("post-OBBBA NCTI"));
    }

    #[test]
    fn action_references_form_5471_and_form_8992() {
        let i = baseline();
        let r = check(&i);
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("Form 5471")));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("Form 8992")));
    }

    #[test]
    fn coordination_note_references_250_59a_56a_1297() {
        let i = baseline();
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("section_250")));
        assert!(r.notes.iter().any(|n| n.contains("section_59a")));
        assert!(r.notes.iter().any(|n| n.contains("section_56a")));
        assert!(r.notes.iter().any(|n| n.contains("section_1297")));
    }

    #[test]
    fn extreme_tested_income_does_not_overflow() {
        let mut i = baseline();
        i.pro_rata_tested_income_cents = i64::MAX / 1000;
        let r = check(&i);
        let _ = r.net_us_tax_cents;
    }

    #[test]
    fn zero_tested_income_zero_effective_rate() {
        let mut i = baseline();
        i.pro_rata_tested_income_cents = 0;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::TestedLossNoCurrentInclusion));
        assert_eq!(r.effective_us_rate_bps, 0);
    }

    #[test]
    fn boundary_2025_still_pre_obbba() {
        let mut i = baseline();
        i.taxable_year = 2025;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::Pre2026GiltiInclusionWithQbai
        ));
        assert!(!r.is_post_obbba_year);
    }

    #[test]
    fn boundary_2026_post_obbba() {
        let mut i = baseline();
        i.taxable_year = 2026;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::Post2026NctiInclusionNoQbai));
        assert!(r.is_post_obbba_year);
    }
}
