//! IRC § 4501 — Repurchase of Corporate Stock Excise Tax (Inflation Reduction
//! Act of 2022, Pub. L. 117-169 § 10201; effective for stock repurchases
//! after December 31, 2022).
//!
//! § 4501(a) imposes a 1% excise tax on the fair market value of stock
//! repurchased by a covered corporation during the taxable year. The tax is
//! NOT deductible by the covered corporation for federal income tax purposes
//! per § 275(a)(6).
//!
//! § 4501(b) "covered corporation" — any domestic corporation the stock of
//! which is traded on an established securities market within the meaning of
//! § 7704(b)(1). Includes NYSE, NASDAQ, and other national securities
//! exchanges; SPACs (Special Purpose Acquisition Companies) explicitly NOT
//! exempt per Final Regulations TD 10002 published July 3, 2024.
//!
//! § 4501(c)(1) "repurchase" definitional — any redemption within the meaning
//! of § 317(b) plus any transaction Treasury determines economically similar.
//! Includes SPAC sponsor redemptions, post-IPO buybacks, ASR (Accelerated
//! Share Repurchase) programs, open-market purchases, tender offers.
//!
//! § 4501(c)(3) "netting rule" — the fair market value of repurchases is
//! REDUCED by (A) FMV of repurchases excluded by § 4501(e) exceptions PLUS
//! (B) FMV of issuances of stock during the same taxable year (offsetting
//! issuances). Compensation-related stock issuances to employees / officers
//! are included in the offset per Treas. Reg. § 1.4501-2(c).
//!
//! § 4501(e) statutory exceptions — six categories: (1) repurchase part of a
//! reorganization under § 368 with no gain/loss; (2) stock contributed to an
//! employer-sponsored retirement plan, ESOP, or similar; (3) total value of
//! repurchases during year ≤ $1,000,000 de minimis (cliff threshold); (4)
//! repurchase by a dealer in securities in the ordinary course of business;
//! (5) repurchase by a RIC or REIT; (6) repurchase treated as a § 301
//! dividend.
//!
//! § 4501(d) extension to acquisitions of stock by SPECIFIED AFFILIATES of
//! foreign-parent groups (covered foreign corporations) — anti-inversion
//! provision.
//!
//! Final Regulations TD 10002 (July 3, 2024 procedural) plus Proposed
//! Regulations REG-115710-22 (April 12, 2024 substantive) plus Notice 2023-2
//! plus Announcement 2023-18 govern reporting via Form 720 plus new Form
//! 7208 (Excise Tax on Repurchase of Corporate Stock).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CorporationType {
    /// Domestic corporation traded on NYSE/NASDAQ/national exchange — covered.
    DomesticPubliclyTraded,
    /// SPAC (Special Purpose Acquisition Company) — covered per Final Regs
    /// TD 10002 (July 3, 2024).
    SpacSpecialPurposeAcquisitionCompany,
    /// Domestic privately-held corporation — NOT covered.
    DomesticPrivatelyHeld,
    /// Foreign corporation — NOT directly covered; specified affiliate
    /// extension may apply per § 4501(d).
    ForeignCorporation,
    /// Regulated Investment Company (RIC) — § 4501(e)(6) statutory exception.
    RegulatedInvestmentCompany,
    /// Real Estate Investment Trust (REIT) — § 4501(e)(6) statutory exception.
    RealEstateInvestmentTrust,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RepurchaseExceptionCategory {
    /// No § 4501(e) exception applies.
    NoException,
    /// § 4501(e)(1) part of § 368 reorganization with no gain/loss.
    Section368Reorganization,
    /// § 4501(e)(2) contributed to employer-sponsored retirement plan or ESOP.
    EmployerRetirementPlanContribution,
    /// § 4501(e)(3) de minimis — total annual repurchases ≤ $1,000,000.
    DeMinimisThreshold1MillionOrLess,
    /// § 4501(e)(4) dealer in securities ordinary course of business.
    DealerOrdinaryCourseBusiness,
    /// § 4501(e)(6) treated as § 301 dividend.
    Section301DividendTreatment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExciseTaxSeverity {
    NotApplicable,
    NotACoveredCorporation,
    DeMinimisExempt,
    FullyExemptedReorganization,
    FullyExemptedRetirementPlan,
    FullyExemptedDealer,
    FullyExemptedSection301Dividend,
    FullyExemptedRicReit,
    NettingFullOffsetByIssuances,
    NettingPartialOffsetByIssuances,
    OnePercentExciseTaxApplies,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section4501Input {
    pub corporation_type: CorporationType,
    pub repurchase_exception: RepurchaseExceptionCategory,
    /// Fair market value of stock repurchased during taxable year in cents
    /// (pre-netting).
    pub fmv_repurchased_cents: u64,
    /// Fair market value of compensatory and other stock issuances during the
    /// same taxable year in cents (netting offset).
    pub fmv_issuances_during_year_cents: u64,
    /// Fair market value of repurchases covered by § 4501(e) exceptions in
    /// cents (separate from issuance netting).
    pub fmv_excepted_repurchases_cents: u64,
    /// Taxable year of repurchase (calendar year).
    pub taxable_year: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section4501Result {
    pub severity: ExciseTaxSeverity,
    pub is_covered_corporation: bool,
    pub fmv_repurchased_cents: u64,
    pub fmv_excepted_repurchases_cents: u64,
    pub fmv_issuances_offset_cents: u64,
    pub taxable_repurchase_base_cents: u64,
    pub excise_tax_cents: u64,
    pub recommended_actions: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub const EXCISE_TAX_RATE_BPS: u32 = 100;
pub const DE_MINIMIS_THRESHOLD_CENTS: u64 = 100_000_000;
pub const SECTION_4501_EFFECTIVE_YEAR: i32 = 2023;

pub fn check(input: &Section4501Input) -> Section4501Result {
    let mut actions: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    if input.taxable_year < SECTION_4501_EFFECTIVE_YEAR {
        notes.push(format!(
            "§ 4501 effective for stock repurchases after December 31, 2022 per Pub. L. \
             117-169 § 10201(d); pre-{} repurchases NOT subject to 1% excise tax.",
            SECTION_4501_EFFECTIVE_YEAR
        ));
        return Section4501Result {
            severity: ExciseTaxSeverity::NotApplicable,
            is_covered_corporation: false,
            fmv_repurchased_cents: input.fmv_repurchased_cents,
            fmv_excepted_repurchases_cents: 0,
            fmv_issuances_offset_cents: 0,
            taxable_repurchase_base_cents: 0,
            excise_tax_cents: 0,
            recommended_actions: actions,
            citation: "26 U.S.C. § 4501; Pub. L. 117-169 § 10201",
            notes,
        };
    }

    let is_covered = matches!(
        input.corporation_type,
        CorporationType::DomesticPubliclyTraded
            | CorporationType::SpacSpecialPurposeAcquisitionCompany
    );

    if !is_covered {
        let severity = match input.corporation_type {
            CorporationType::RegulatedInvestmentCompany
            | CorporationType::RealEstateInvestmentTrust => {
                notes.push(
                    "RIC / REIT exception under § 4501(e)(6) — distributions treated as § 301 \
                     dividends not subject to 1% excise tax."
                        .to_string(),
                );
                ExciseTaxSeverity::FullyExemptedRicReit
            }
            CorporationType::DomesticPrivatelyHeld => {
                notes.push(
                    "Domestic privately-held corporation — stock not traded on established \
                     securities market per § 7704(b)(1); NOT a covered corporation under § \
                     4501(b). 1% excise tax inapplicable. Monitor IPO timing — covered status \
                     attaches upon listing on national securities exchange."
                        .to_string(),
                );
                ExciseTaxSeverity::NotACoveredCorporation
            }
            CorporationType::ForeignCorporation => {
                notes.push(
                    "Foreign corporation not directly covered; § 4501(d) specified-affiliate \
                     extension may apply to anti-inversion structures where US specified \
                     affiliate purchases stock of foreign-parent on behalf of foreign group."
                        .to_string(),
                );
                ExciseTaxSeverity::NotACoveredCorporation
            }
            _ => ExciseTaxSeverity::NotACoveredCorporation,
        };
        return Section4501Result {
            severity,
            is_covered_corporation: false,
            fmv_repurchased_cents: input.fmv_repurchased_cents,
            fmv_excepted_repurchases_cents: 0,
            fmv_issuances_offset_cents: 0,
            taxable_repurchase_base_cents: 0,
            excise_tax_cents: 0,
            recommended_actions: actions,
            citation: "26 U.S.C. § 4501(b); § 7704(b)(1)",
            notes,
        };
    }

    if matches!(
        input.repurchase_exception,
        RepurchaseExceptionCategory::Section368Reorganization
    ) {
        notes.push(
            "Repurchase part of § 368 reorganization with no gain/loss recognized; § \
             4501(e)(1) exception fully exempts amount from 1% excise tax. Document \
             reorganization treatment via continuity-of-interest plus continuity-of-business-\
             enterprise tests per Treas. Reg. § 1.368-1."
                .to_string(),
        );
        return Section4501Result {
            severity: ExciseTaxSeverity::FullyExemptedReorganization,
            is_covered_corporation: true,
            fmv_repurchased_cents: input.fmv_repurchased_cents,
            fmv_excepted_repurchases_cents: input.fmv_repurchased_cents,
            fmv_issuances_offset_cents: 0,
            taxable_repurchase_base_cents: 0,
            excise_tax_cents: 0,
            recommended_actions: actions,
            citation: "26 U.S.C. § 4501(e)(1); 26 U.S.C. § 368",
            notes,
        };
    }

    if matches!(
        input.repurchase_exception,
        RepurchaseExceptionCategory::EmployerRetirementPlanContribution
    ) {
        notes.push(
            "Repurchased stock contributed to employer-sponsored retirement plan, ESOP, or \
             similar under § 4501(e)(2); coordination with [[section_1042]] (ESOP rollover) \
             plus [[section_4978]] (recapture) plus [[section_4972]] (nondeductible \
             contribution excise)."
                .to_string(),
        );
        return Section4501Result {
            severity: ExciseTaxSeverity::FullyExemptedRetirementPlan,
            is_covered_corporation: true,
            fmv_repurchased_cents: input.fmv_repurchased_cents,
            fmv_excepted_repurchases_cents: input.fmv_repurchased_cents,
            fmv_issuances_offset_cents: 0,
            taxable_repurchase_base_cents: 0,
            excise_tax_cents: 0,
            recommended_actions: actions,
            citation: "26 U.S.C. § 4501(e)(2)",
            notes,
        };
    }

    if matches!(
        input.repurchase_exception,
        RepurchaseExceptionCategory::DealerOrdinaryCourseBusiness
    ) {
        notes.push(
            "Repurchase by dealer in securities in ordinary course of business per § \
             4501(e)(4) — fully exempt. Distinct from market-maker activity outside dealer \
             classification."
                .to_string(),
        );
        return Section4501Result {
            severity: ExciseTaxSeverity::FullyExemptedDealer,
            is_covered_corporation: true,
            fmv_repurchased_cents: input.fmv_repurchased_cents,
            fmv_excepted_repurchases_cents: input.fmv_repurchased_cents,
            fmv_issuances_offset_cents: 0,
            taxable_repurchase_base_cents: 0,
            excise_tax_cents: 0,
            recommended_actions: actions,
            citation: "26 U.S.C. § 4501(e)(4); 26 U.S.C. § 475(c)(1)",
            notes,
        };
    }

    if matches!(
        input.repurchase_exception,
        RepurchaseExceptionCategory::Section301DividendTreatment
    ) {
        notes.push(
            "Repurchase treated as § 301 dividend (not as § 317(b) redemption) per § \
             4501(e)(6); 1% excise tax inapplicable. Dividend treatment may apply when \
             § 302(d) recharacterizes failed redemption."
                .to_string(),
        );
        return Section4501Result {
            severity: ExciseTaxSeverity::FullyExemptedSection301Dividend,
            is_covered_corporation: true,
            fmv_repurchased_cents: input.fmv_repurchased_cents,
            fmv_excepted_repurchases_cents: input.fmv_repurchased_cents,
            fmv_issuances_offset_cents: 0,
            taxable_repurchase_base_cents: 0,
            excise_tax_cents: 0,
            recommended_actions: actions,
            citation: "26 U.S.C. § 4501(e)(6); 26 U.S.C. § 301",
            notes,
        };
    }

    if matches!(
        input.repurchase_exception,
        RepurchaseExceptionCategory::DeMinimisThreshold1MillionOrLess
    ) || input.fmv_repurchased_cents <= DE_MINIMIS_THRESHOLD_CENTS
    {
        notes.push(format!(
            "Total annual repurchase value of {} cents at or below $1,000,000 de minimis \
             threshold per § 4501(e)(3). Cliff threshold: $1,000,000.01 triggers full 1% on \
             entire repurchase amount, not just excess.",
            input.fmv_repurchased_cents
        ));
        return Section4501Result {
            severity: ExciseTaxSeverity::DeMinimisExempt,
            is_covered_corporation: true,
            fmv_repurchased_cents: input.fmv_repurchased_cents,
            fmv_excepted_repurchases_cents: input.fmv_repurchased_cents,
            fmv_issuances_offset_cents: 0,
            taxable_repurchase_base_cents: 0,
            excise_tax_cents: 0,
            recommended_actions: actions,
            citation: "26 U.S.C. § 4501(e)(3)",
            notes,
        };
    }

    let after_excepted = input
        .fmv_repurchased_cents
        .saturating_sub(input.fmv_excepted_repurchases_cents);
    let after_netting = after_excepted.saturating_sub(input.fmv_issuances_during_year_cents);
    let excise_tax_cents: u64 =
        (u128::from(after_netting) * u128::from(EXCISE_TAX_RATE_BPS) / 10_000) as u64;

    let severity = if after_netting == 0 {
        ExciseTaxSeverity::NettingFullOffsetByIssuances
    } else if input.fmv_issuances_during_year_cents > 0 {
        ExciseTaxSeverity::NettingPartialOffsetByIssuances
    } else {
        ExciseTaxSeverity::OnePercentExciseTaxApplies
    };

    actions.push(format!(
        "File Form 7208 (Excise Tax on Repurchase of Corporate Stock) attached to Form 720 \
         Quarterly Federal Excise Tax Return for tax year {}; deadline is first day of fourth \
         month following close of taxable year per Final Regs TD 10002 (July 3, 2024) plus \
         Notice 2023-2. Tax computed on taxable base of {} cents at 1% rate = {} cents.",
        input.taxable_year, after_netting, excise_tax_cents
    ));
    if matches!(
        input.corporation_type,
        CorporationType::SpacSpecialPurposeAcquisitionCompany
    ) {
        actions.push(
            "SPAC sponsor / shareholder redemptions on de-SPAC, qualifying business \
             combination, or trust-liquidation events are SUBJECT to § 4501 1% excise tax per \
             Final Regs TD 10002; IRS rejected SPAC-specific carve-out. Coordinate with §§ \
             351, 368, 304 if redemption is structured through reorganization."
                .to_string(),
        );
    }
    notes.push(format!(
        "1% excise tax not deductible per § 275(a)(6); record as permanent book-tax difference. \
         Compensatory issuances (RSU vest, ISO/NSO exercise, ESPP issuance, equity grants) \
         offset repurchases dollar-for-dollar per Treas. Reg. § 1.4501-2(c). Computed taxable \
         base: {} cents repurchased - {} cents excepted - {} cents issuances = {} cents.",
        input.fmv_repurchased_cents,
        input.fmv_excepted_repurchases_cents,
        input.fmv_issuances_during_year_cents,
        after_netting
    ));
    notes.push(
        "Coordination with [[section_280g]] (golden parachute on change of control), \
         [[section_421]] (statutory stock options exercise creates issuances offset), \
         [[section_56a]] (corporate AMT 15% — separate IRA 2022 provision), [[section_4960]] \
         (ATEO executive comp 21%), [[section_1042]] (ESOP rollover for § 4501(e)(2) \
         exception)."
            .to_string(),
    );

    Section4501Result {
        severity,
        is_covered_corporation: true,
        fmv_repurchased_cents: input.fmv_repurchased_cents,
        fmv_excepted_repurchases_cents: input.fmv_excepted_repurchases_cents,
        fmv_issuances_offset_cents: input.fmv_issuances_during_year_cents,
        taxable_repurchase_base_cents: after_netting,
        excise_tax_cents,
        recommended_actions: actions,
        citation: "26 U.S.C. § 4501(a)-(e); TD 10002 (July 3, 2024); Pub. L. 117-169 § 10201",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> Section4501Input {
        Section4501Input {
            corporation_type: CorporationType::DomesticPubliclyTraded,
            repurchase_exception: RepurchaseExceptionCategory::NoException,
            fmv_repurchased_cents: 100_000_000_00,
            fmv_issuances_during_year_cents: 0,
            fmv_excepted_repurchases_cents: 0,
            taxable_year: 2024,
        }
    }

    #[test]
    fn pre_2023_repurchase_not_applicable() {
        let mut i = baseline();
        i.taxable_year = 2022;
        let r = check(&i);
        assert!(matches!(r.severity, ExciseTaxSeverity::NotApplicable));
        assert_eq!(r.excise_tax_cents, 0);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("after December 31, 2022")));
    }

    #[test]
    fn effective_year_pins_2023() {
        assert_eq!(SECTION_4501_EFFECTIVE_YEAR, 2023);
    }

    #[test]
    fn domestic_privately_held_not_covered() {
        let mut i = baseline();
        i.corporation_type = CorporationType::DomesticPrivatelyHeld;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            ExciseTaxSeverity::NotACoveredCorporation
        ));
        assert!(!r.is_covered_corporation);
        assert_eq!(r.excise_tax_cents, 0);
        assert!(r.notes.iter().any(|n| n.contains("§ 7704(b)(1)")));
    }

    #[test]
    fn foreign_corporation_not_directly_covered() {
        let mut i = baseline();
        i.corporation_type = CorporationType::ForeignCorporation;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            ExciseTaxSeverity::NotACoveredCorporation
        ));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 4501(d) specified-affiliate")));
    }

    #[test]
    fn ric_exempt_under_4501_e_6() {
        let mut i = baseline();
        i.corporation_type = CorporationType::RegulatedInvestmentCompany;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            ExciseTaxSeverity::FullyExemptedRicReit
        ));
        assert_eq!(r.excise_tax_cents, 0);
    }

    #[test]
    fn reit_exempt_under_4501_e_6() {
        let mut i = baseline();
        i.corporation_type = CorporationType::RealEstateInvestmentTrust;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            ExciseTaxSeverity::FullyExemptedRicReit
        ));
    }

    #[test]
    fn de_minimis_at_threshold_exempt() {
        let mut i = baseline();
        i.fmv_repurchased_cents = DE_MINIMIS_THRESHOLD_CENTS;
        let r = check(&i);
        assert!(matches!(r.severity, ExciseTaxSeverity::DeMinimisExempt));
        assert_eq!(r.excise_tax_cents, 0);
    }

    #[test]
    fn de_minimis_one_cent_over_triggers_full_tax() {
        let mut i = baseline();
        i.fmv_repurchased_cents = DE_MINIMIS_THRESHOLD_CENTS + 1;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            ExciseTaxSeverity::OnePercentExciseTaxApplies
        ));
        let expected = (DE_MINIMIS_THRESHOLD_CENTS + 1) / 100;
        assert_eq!(r.excise_tax_cents, expected);
    }

    #[test]
    fn de_minimis_threshold_pins_1m_dollar() {
        assert_eq!(DE_MINIMIS_THRESHOLD_CENTS, 100_000_000);
    }

    #[test]
    fn excise_rate_pins_1_percent() {
        assert_eq!(EXCISE_TAX_RATE_BPS, 100);
    }

    #[test]
    fn section_368_reorganization_fully_exempted() {
        let mut i = baseline();
        i.repurchase_exception = RepurchaseExceptionCategory::Section368Reorganization;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            ExciseTaxSeverity::FullyExemptedReorganization
        ));
        assert_eq!(r.excise_tax_cents, 0);
        assert!(r.notes.iter().any(|n| n.contains("Treas. Reg. § 1.368-1")));
    }

    #[test]
    fn retirement_plan_contribution_fully_exempted() {
        let mut i = baseline();
        i.repurchase_exception = RepurchaseExceptionCategory::EmployerRetirementPlanContribution;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            ExciseTaxSeverity::FullyExemptedRetirementPlan
        ));
        assert!(r.notes.iter().any(|n| n.contains("section_1042")));
        assert!(r.notes.iter().any(|n| n.contains("section_4978")));
    }

    #[test]
    fn dealer_ordinary_course_fully_exempted() {
        let mut i = baseline();
        i.repurchase_exception = RepurchaseExceptionCategory::DealerOrdinaryCourseBusiness;
        let r = check(&i);
        assert!(matches!(r.severity, ExciseTaxSeverity::FullyExemptedDealer));
        assert!(r.citation.contains("§ 475(c)(1)"));
    }

    #[test]
    fn section_301_dividend_treatment_fully_exempted() {
        let mut i = baseline();
        i.repurchase_exception = RepurchaseExceptionCategory::Section301DividendTreatment;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            ExciseTaxSeverity::FullyExemptedSection301Dividend
        ));
        assert!(r.notes.iter().any(|n| n.contains("§ 302(d)")));
    }

    #[test]
    fn one_billion_repurchase_no_offset_one_percent() {
        let mut i = baseline();
        i.fmv_repurchased_cents = 1_000_000_000_00;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            ExciseTaxSeverity::OnePercentExciseTaxApplies
        ));
        let expected = 1_000_000_000_00u64 / 100;
        assert_eq!(r.excise_tax_cents, expected);
        assert_eq!(r.excise_tax_cents, 10_000_000_00);
    }

    #[test]
    fn netting_full_offset_by_compensatory_issuances() {
        let mut i = baseline();
        i.fmv_repurchased_cents = 50_000_000_00;
        i.fmv_issuances_during_year_cents = 50_000_000_00;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            ExciseTaxSeverity::NettingFullOffsetByIssuances
        ));
        assert_eq!(r.excise_tax_cents, 0);
        assert_eq!(r.taxable_repurchase_base_cents, 0);
    }

    #[test]
    fn netting_partial_offset_taxable_base_correct() {
        let mut i = baseline();
        i.fmv_repurchased_cents = 100_000_000_00;
        i.fmv_issuances_during_year_cents = 30_000_000_00;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            ExciseTaxSeverity::NettingPartialOffsetByIssuances
        ));
        assert_eq!(r.taxable_repurchase_base_cents, 70_000_000_00);
        assert_eq!(r.excise_tax_cents, 70_000_000_00 / 100);
    }

    #[test]
    fn excepted_repurchases_reduce_base_before_netting() {
        let mut i = baseline();
        i.fmv_repurchased_cents = 100_000_000_00;
        i.fmv_excepted_repurchases_cents = 20_000_000_00;
        i.fmv_issuances_during_year_cents = 10_000_000_00;
        let r = check(&i);
        assert_eq!(r.taxable_repurchase_base_cents, 70_000_000_00);
    }

    #[test]
    fn over_netting_does_not_go_negative_saturating() {
        let mut i = baseline();
        i.fmv_repurchased_cents = 10_000_000_00;
        i.fmv_issuances_during_year_cents = 50_000_000_00;
        let r = check(&i);
        assert_eq!(r.taxable_repurchase_base_cents, 0);
        assert_eq!(r.excise_tax_cents, 0);
        assert!(matches!(
            r.severity,
            ExciseTaxSeverity::NettingFullOffsetByIssuances
        ));
    }

    #[test]
    fn spac_covered_per_final_regs() {
        let mut i = baseline();
        i.corporation_type = CorporationType::SpacSpecialPurposeAcquisitionCompany;
        i.fmv_repurchased_cents = 200_000_000_00;
        let r = check(&i);
        assert!(r.is_covered_corporation);
        assert!(matches!(
            r.severity,
            ExciseTaxSeverity::OnePercentExciseTaxApplies
        ));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("SPAC sponsor")));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("rejected SPAC-specific carve-out")));
    }

    #[test]
    fn citation_pins_final_regs_td_10002() {
        let i = baseline();
        let r = check(&i);
        assert!(r.citation.contains("TD 10002"));
        assert!(r.citation.contains("July 3, 2024"));
        assert!(r.citation.contains("§ 10201"));
    }

    #[test]
    fn action_recommends_form_7208_and_form_720() {
        let i = baseline();
        let r = check(&i);
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("Form 7208")));
        assert!(r.recommended_actions.iter().any(|a| a.contains("Form 720")));
    }

    #[test]
    fn note_pins_section_275_a_6_non_deductibility() {
        let i = baseline();
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("§ 275(a)(6)")));
    }

    #[test]
    fn note_pins_treas_reg_compensatory_issuance_offset() {
        let i = baseline();
        let r = check(&i);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Treas. Reg. § 1.4501-2(c)")));
        assert!(r.notes.iter().any(|n| n.contains("RSU vest")));
    }

    #[test]
    fn coordination_note_references_280g_421_56a_4960_1042() {
        let i = baseline();
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("section_280g")));
        assert!(r.notes.iter().any(|n| n.contains("section_421")));
        assert!(r.notes.iter().any(|n| n.contains("section_56a")));
        assert!(r.notes.iter().any(|n| n.contains("section_4960")));
        assert!(r.notes.iter().any(|n| n.contains("section_1042")));
    }

    #[test]
    fn realistic_apple_buyback_scenario() {
        let mut i = baseline();
        i.fmv_repurchased_cents = 100_000_000_000_00;
        i.fmv_issuances_during_year_cents = 5_000_000_000_00;
        let r = check(&i);
        assert_eq!(r.taxable_repurchase_base_cents, 95_000_000_000_00);
        assert_eq!(r.excise_tax_cents, 950_000_000_00);
    }

    #[test]
    fn extreme_repurchase_value_does_not_overflow() {
        let mut i = baseline();
        i.fmv_repurchased_cents = u64::MAX / 1000;
        let r = check(&i);
        let _ = r.excise_tax_cents;
    }

    #[test]
    fn zero_repurchase_zero_tax() {
        let mut i = baseline();
        i.fmv_repurchased_cents = 0;
        let r = check(&i);
        assert!(matches!(r.severity, ExciseTaxSeverity::DeMinimisExempt));
        assert_eq!(r.excise_tax_cents, 0);
    }

    #[test]
    fn exception_precedence_reorganization_overrides_de_minimis() {
        let mut i = baseline();
        i.repurchase_exception = RepurchaseExceptionCategory::Section368Reorganization;
        i.fmv_repurchased_cents = 500_000_00;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            ExciseTaxSeverity::FullyExemptedReorganization
        ));
    }

    #[test]
    fn boundary_at_500m_repurchase_5m_excise() {
        let mut i = baseline();
        i.fmv_repurchased_cents = 500_000_000_00;
        let r = check(&i);
        assert_eq!(r.excise_tax_cents, 5_000_000_00);
    }
}
