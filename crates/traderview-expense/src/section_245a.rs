//! IRC § 245A — Deduction for Foreign Source-Portion of Dividends Received by
//! Domestic Corporations from Specified 10-Percent Owned Foreign Corporations
//! (SFCs). Enacted by Tax Cuts and Jobs Act of 2017 (Pub. L. 115-97 § 14101)
//! effective for distributions made after December 31, 2017.
//!
//! § 245A(a) general rule: a domestic C corporation is allowed a 100%
//! "participation exemption" dividends-received deduction (DRD) for the
//! foreign-source portion of dividends received from a specified 10-percent
//! owned foreign corporation (SFC). This implements the post-TCJA
//! "territorial" tax system for repatriation of post-2017 earnings of CFCs
//! and other foreign corps.
//!
//! § 245A(b) SFC definition: "specified 10-percent owned foreign corporation"
//! means any foreign corporation (other than a PFIC that is NOT also a CFC)
//! with respect to which any domestic corporation is a US shareholder under §
//! 951(b) (i.e., owns 10% or more of vote or value).
//!
//! § 245A(c) foreign-source portion: divides dividend into foreign-source and
//! US-source portions based on the SFC's post-1986 undistributed earnings;
//! only the foreign-source portion qualifies for the 100% DRD.
//!
//! § 246(c)(1) holding period requirement: domestic corp must hold the SFC
//! stock for MORE THAN 365 days during the 731-day period beginning on the
//! date which is 365 days before the ex-dividend date.
//!
//! § 245A(d) foreign tax credit / deduction coordination: NO FTC or foreign-
//! tax deduction is allowed under § 901 / § 164(a) for any foreign tax
//! (including withholding) paid or accrued with respect to a § 245A-eligible
//! dividend. Tax NOT deductible per § 275(a)(4).
//!
//! § 245A(e) hybrid dividend rule: dividend treated as ineligible for DRD if
//! it is a "hybrid dividend" — a dividend with respect to which the foreign
//! corp received a foreign tax deduction (or similar tax benefit) in the
//! distributing country. Hybrid dividend → recharacterized as Subpart F
//! inclusion plus no DRD plus no FTC.
//!
//! § 245A(g) regulatory authority: Treas. Reg. § 1.245A-5 (anti-abuse rules
//! preventing extraordinary disposition / extraordinary reduction
//! manipulation) plus Treas. Reg. § 1.245A-6T through § 1.245A-9.
//!
//! Coordination matrix: SFC dividend may flow through (1) Subpart F per §
//! 951(a)(1)(A) (if FPHCI / foreign base co income / insurance income), OR (2)
//! GILTI / NCTI per § 951A (post-Subpart F residual), OR (3) § 245A 100% DRD
//! for foreign-source portion not already taxed. § 1248 recharacterizes gain
//! on CFC stock sale as dividend, which then qualifies for § 245A DRD.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShareholderType {
    /// Domestic C corporation — qualifies for § 245A.
    DomesticCCorporation,
    /// Real estate investment trust — § 245A unavailable.
    RealEstateInvestmentTrust,
    /// Regulated investment company — § 245A unavailable.
    RegulatedInvestmentCompany,
    /// Individual / partnership / S corp — § 245A unavailable.
    IndividualOrPassThrough,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ForeignCorpType {
    /// Specified 10%-owned foreign corporation under § 245A(b).
    SpecifiedTenPercentOwnedForeignCorp,
    /// PFIC that is NOT also a CFC — excluded from SFC definition.
    PficNotCfcExcludedFromSfc,
    /// Foreign corp with less-than-10% US shareholder ownership — not SFC.
    LessThanTenPercentNotSfc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    NotEligibleShareholderType,
    NotSpecifiedTenPercentOwnedForeignCorp,
    PreEffectiveDate,
    HoldingPeriodNotSatisfied,
    HybridDividendRecharacterizedNoDeduction,
    PartialDeductionForeignSourcePortionOnly,
    FullDeductionEligible,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section245aInput {
    pub shareholder_type: ShareholderType,
    pub foreign_corp_type: ForeignCorpType,
    /// Distribution year (calendar). § 245A effective for distributions made
    /// after December 31, 2017.
    pub distribution_year: i32,
    /// Total dividend received in cents.
    pub total_dividend_cents: u64,
    /// Foreign-source portion of dividend per § 245A(c) in cents.
    pub foreign_source_portion_cents: u64,
    /// US-source portion of dividend per § 245A(c) in cents.
    pub us_source_portion_cents: u64,
    /// Number of days domestic corp held SFC stock in the 731-day period
    /// beginning 365 days before ex-dividend date.
    pub days_held_in_731_day_window: u32,
    /// Whether dividend is a "hybrid dividend" under § 245A(e) (foreign tax
    /// deduction received by distributing corp).
    pub is_hybrid_dividend: bool,
    /// Foreign withholding tax paid on the dividend in cents.
    pub foreign_withholding_tax_cents: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section245aResult {
    pub severity: Severity,
    pub deduction_cents: u64,
    pub taxable_dividend_after_deduction_cents: u64,
    pub disallowed_ftc_cents: u64,
    pub recommended_actions: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub const SECTION_246C_HOLDING_PERIOD_DAYS: u32 = 366;
pub const SECTION_246C_TESTING_WINDOW_DAYS: u32 = 731;
pub const SECTION_245A_EFFECTIVE_YEAR: i32 = 2018;
pub const SECTION_245A_DRD_RATE_BPS: u32 = 10_000;

pub fn check(input: &Section245aInput) -> Section245aResult {
    let mut actions: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    if input.distribution_year < SECTION_245A_EFFECTIVE_YEAR {
        notes.push(format!(
            "§ 245A effective for distributions made after December 31, 2017 per Pub. L. \
             115-97 § 14101; pre-{} distributions governed by historic foreign DRD rules \
             under § 245 (limited DRD for dividends from 10%-or-more foreign subsidiary).",
            SECTION_245A_EFFECTIVE_YEAR
        ));
        return empty_result(
            Severity::PreEffectiveDate,
            input,
            actions,
            notes,
            "26 U.S.C. § 245A; Pub. L. 115-97 § 14101",
        );
    }

    if !matches!(
        input.shareholder_type,
        ShareholderType::DomesticCCorporation
    ) {
        notes.push(
            "§ 245A 100% DRD is ONLY available to domestic C corporations (not REITs, RICs, \
             S corps, individuals, partnerships). Pass-through shareholders may not flow § \
             245A deduction through to ultimate beneficial owner unless C corp blocker is \
             interposed."
                .to_string(),
        );
        return empty_result(
            Severity::NotEligibleShareholderType,
            input,
            actions,
            notes,
            "26 U.S.C. § 245A(a); coord. § 246",
        );
    }

    if !matches!(
        input.foreign_corp_type,
        ForeignCorpType::SpecifiedTenPercentOwnedForeignCorp
    ) {
        match input.foreign_corp_type {
            ForeignCorpType::PficNotCfcExcludedFromSfc => {
                notes.push(
                    "PFIC that is not also a CFC is EXCLUDED from § 245A(b) SFC definition; \
                     dividends not eligible for § 245A 100% DRD. Excess distribution tax \
                     under § 1291 may apply absent QEF / mark-to-market election under \
                     [[section_1297]] (PFIC determination)."
                        .to_string(),
                );
            }
            ForeignCorpType::LessThanTenPercentNotSfc => {
                notes.push(
                    "Foreign corporation with less-than-10% US shareholder ownership is NOT \
                     an SFC under § 245A(b); dividends taxed as ordinary income with limited \
                     FTC under § 901 / § 904. Portfolio dividend treatment."
                        .to_string(),
                );
            }
            _ => {}
        }
        return empty_result(
            Severity::NotSpecifiedTenPercentOwnedForeignCorp,
            input,
            actions,
            notes,
            "26 U.S.C. § 245A(b); § 951(b); § 1297",
        );
    }

    if input.days_held_in_731_day_window < SECTION_246C_HOLDING_PERIOD_DAYS {
        actions.push(format!(
            "Holding period under § 246(c)(1) NOT satisfied: held {} days in 731-day window \
             beginning 365 days before ex-dividend date; requires MORE THAN 365 days (i.e., \
             at least {} days). Dividend taxed at ordinary corporate rate without § 245A \
             100% DRD; FTC under § 901 / § 904 remains available since § 245A(d) \
             disallowance only attaches to DRD-eligible dividends.",
            input.days_held_in_731_day_window, SECTION_246C_HOLDING_PERIOD_DAYS
        ));
        return empty_result(
            Severity::HoldingPeriodNotSatisfied,
            input,
            actions,
            notes,
            "26 U.S.C. § 245A(a); § 246(c)(1); § 901; § 904",
        );
    }

    if input.is_hybrid_dividend {
        actions.push(
            "Hybrid dividend under § 245A(e) — distributing foreign corp received foreign \
             tax deduction or similar tax benefit on the dividend; § 245A 100% DRD DENIED. \
             Recharacterize entire dividend as Subpart F inclusion under § 951(a); no FTC \
             allowed for foreign withholding tax on hybrid dividend per § 245A(e)(3). \
             Reduce US shareholder's hybrid deduction account by hybrid dividend amount per \
             Treas. Reg. § 1.245A(e)-1(d)."
                .to_string(),
        );
        return Section245aResult {
            severity: Severity::HybridDividendRecharacterizedNoDeduction,
            deduction_cents: 0,
            taxable_dividend_after_deduction_cents: input.total_dividend_cents,
            disallowed_ftc_cents: input.foreign_withholding_tax_cents,
            recommended_actions: actions,
            citation: "26 U.S.C. § 245A(e); Treas. Reg. § 1.245A(e)-1(d)",
            notes,
        };
    }

    let foreign_source: u64 = input.foreign_source_portion_cents;
    let us_source: u64 = input.us_source_portion_cents;
    let sum: u64 = foreign_source.saturating_add(us_source);
    let total: u64 = input.total_dividend_cents;
    if sum != total {
        notes.push(format!(
            "Foreign-source ({} cents) plus US-source ({} cents) portion sum to {} cents but \
             total dividend reported as {} cents; reconcile before claiming § 245A. Treas. \
             Reg. § 1.245A-5 anti-abuse rule disallows DRD for amounts attributable to \
             extraordinary disposition account / extraordinary reduction.",
            foreign_source, us_source, sum, total
        ));
    }

    let deduction: u64 =
        (u128::from(foreign_source) * u128::from(SECTION_245A_DRD_RATE_BPS) / 10_000) as u64;
    let taxable_after_deduction: u64 = total.saturating_sub(deduction);

    let severity = if us_source == 0 && foreign_source == total {
        Severity::FullDeductionEligible
    } else {
        Severity::PartialDeductionForeignSourcePortionOnly
    };

    actions.push(format!(
        "Apply § 245A 100% DRD to foreign-source portion of {} cents; taxable dividend after \
         deduction = {} cents (US-source portion taxed at corporate rate). § 245A(d) \
         coordination: NO FTC or § 164(a) deduction allowed for {} cents of foreign \
         withholding tax attributable to DRD-eligible dividend; record as permanent book-\
         tax difference. File Form 8993 if § 250 deduction also claimed; report § 245A \
         deduction on Form 1120 Schedule C.",
        foreign_source, taxable_after_deduction, input.foreign_withholding_tax_cents
    ));

    notes.push(
        "Coordination with [[section_951a]] (GILTI / post-OBBBA NCTI — § 245A applies to \
         residual foreign-source dividend AFTER Subpart F + GILTI / NCTI inclusion), \
         [[section_250]] (FDII + GILTI / NCTI deduction — coordinates with § 245A through \
         tested income exclusion), [[section_1297]] (PFIC mutually exclusive with SFC), \
         [[section_1248]] (§ 1248 gain on CFC stock sale recharacterized as dividend, \
         eligible for § 245A DRD post-2017), [[section_243]] (domestic DRD parallel \
         framework — 50% / 65% / 100% by ownership tier), [[section_59a]] (BEAT separate \
         base-erosion regime), [[section_56a]] (CAMT — § 245A-deductible amount excluded \
         from CAMT AFSI per § 56A(c)(2)(C))."
            .to_string(),
    );

    Section245aResult {
        severity,
        deduction_cents: deduction,
        taxable_dividend_after_deduction_cents: taxable_after_deduction,
        disallowed_ftc_cents: input.foreign_withholding_tax_cents,
        recommended_actions: actions,
        citation: "26 U.S.C. § 245A(a)-(g); Pub. L. 115-97 § 14101; Treas. Reg. § 1.245A-5",
        notes,
    }
}

fn empty_result(
    severity: Severity,
    input: &Section245aInput,
    recommended_actions: Vec<String>,
    notes: Vec<String>,
    citation: &'static str,
) -> Section245aResult {
    Section245aResult {
        severity,
        deduction_cents: 0,
        taxable_dividend_after_deduction_cents: input.total_dividend_cents,
        disallowed_ftc_cents: 0,
        recommended_actions,
        citation,
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> Section245aInput {
        Section245aInput {
            shareholder_type: ShareholderType::DomesticCCorporation,
            foreign_corp_type: ForeignCorpType::SpecifiedTenPercentOwnedForeignCorp,
            distribution_year: 2024,
            total_dividend_cents: 100_000_000_00,
            foreign_source_portion_cents: 100_000_000_00,
            us_source_portion_cents: 0,
            days_held_in_731_day_window: 400,
            is_hybrid_dividend: false,
            foreign_withholding_tax_cents: 5_000_000_00,
        }
    }

    #[test]
    fn pre_2018_distribution_pre_effective_date() {
        let mut i = baseline();
        i.distribution_year = 2017;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::PreEffectiveDate));
        assert_eq!(r.deduction_cents, 0);
    }

    #[test]
    fn effective_year_pins_2018() {
        assert_eq!(SECTION_245A_EFFECTIVE_YEAR, 2018);
    }

    #[test]
    fn drd_rate_pins_100_pct() {
        assert_eq!(SECTION_245A_DRD_RATE_BPS, 10_000);
    }

    #[test]
    fn holding_period_pins_366_days_more_than_365() {
        assert_eq!(SECTION_246C_HOLDING_PERIOD_DAYS, 366);
    }

    #[test]
    fn testing_window_pins_731_days() {
        assert_eq!(SECTION_246C_TESTING_WINDOW_DAYS, 731);
    }

    #[test]
    fn reit_not_eligible_shareholder() {
        let mut i = baseline();
        i.shareholder_type = ShareholderType::RealEstateInvestmentTrust;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::NotEligibleShareholderType));
        assert_eq!(r.deduction_cents, 0);
    }

    #[test]
    fn ric_not_eligible_shareholder() {
        let mut i = baseline();
        i.shareholder_type = ShareholderType::RegulatedInvestmentCompany;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::NotEligibleShareholderType));
    }

    #[test]
    fn individual_not_eligible_shareholder() {
        let mut i = baseline();
        i.shareholder_type = ShareholderType::IndividualOrPassThrough;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::NotEligibleShareholderType));
        assert!(r.notes.iter().any(|n| n.contains("C corp blocker")));
    }

    #[test]
    fn pfic_not_cfc_excluded_from_sfc() {
        let mut i = baseline();
        i.foreign_corp_type = ForeignCorpType::PficNotCfcExcludedFromSfc;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::NotSpecifiedTenPercentOwnedForeignCorp
        ));
        assert!(r.notes.iter().any(|n| n.contains("section_1297")));
    }

    #[test]
    fn less_than_10_pct_not_sfc() {
        let mut i = baseline();
        i.foreign_corp_type = ForeignCorpType::LessThanTenPercentNotSfc;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::NotSpecifiedTenPercentOwnedForeignCorp
        ));
        assert!(r.notes.iter().any(|n| n.contains("Portfolio dividend")));
    }

    #[test]
    fn holding_period_365_days_fails_more_than_test() {
        let mut i = baseline();
        i.days_held_in_731_day_window = 365;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::HoldingPeriodNotSatisfied));
        assert_eq!(r.deduction_cents, 0);
    }

    #[test]
    fn holding_period_366_days_satisfies_more_than_test() {
        let mut i = baseline();
        i.days_held_in_731_day_window = 366;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::FullDeductionEligible));
    }

    #[test]
    fn hybrid_dividend_recharacterized_no_drd_no_ftc() {
        let mut i = baseline();
        i.is_hybrid_dividend = true;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::HybridDividendRecharacterizedNoDeduction
        ));
        assert_eq!(r.deduction_cents, 0);
        assert_eq!(
            r.taxable_dividend_after_deduction_cents,
            i.total_dividend_cents
        );
        assert_eq!(r.disallowed_ftc_cents, i.foreign_withholding_tax_cents);
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("Treas. Reg. § 1.245A(e)-1(d)")));
    }

    #[test]
    fn full_foreign_source_full_drd() {
        let i = baseline();
        let r = check(&i);
        assert!(matches!(r.severity, Severity::FullDeductionEligible));
        assert_eq!(r.deduction_cents, i.total_dividend_cents);
        assert_eq!(r.taxable_dividend_after_deduction_cents, 0);
    }

    #[test]
    fn partial_us_source_only_foreign_portion_deductible() {
        let mut i = baseline();
        i.total_dividend_cents = 100_000_000_00;
        i.foreign_source_portion_cents = 70_000_000_00;
        i.us_source_portion_cents = 30_000_000_00;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::PartialDeductionForeignSourcePortionOnly
        ));
        assert_eq!(r.deduction_cents, 70_000_000_00);
        assert_eq!(r.taxable_dividend_after_deduction_cents, 30_000_000_00);
    }

    #[test]
    fn ftc_disallowed_pinned_in_action() {
        let i = baseline();
        let r = check(&i);
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("§ 245A(d)")));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("permanent book-tax")));
    }

    #[test]
    fn action_references_form_8993_and_form_1120_schedule_c() {
        let i = baseline();
        let r = check(&i);
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("Form 8993")));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("Schedule C")));
    }

    #[test]
    fn coordination_note_references_951a_250_1297_1248_56a() {
        let i = baseline();
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("section_951a")));
        assert!(r.notes.iter().any(|n| n.contains("section_250")));
        assert!(r.notes.iter().any(|n| n.contains("section_1297")));
        assert!(r.notes.iter().any(|n| n.contains("section_1248")));
        assert!(r.notes.iter().any(|n| n.contains("section_56a")));
        assert!(r.notes.iter().any(|n| n.contains("section_59a")));
        assert!(r.notes.iter().any(|n| n.contains("section_243")));
    }

    #[test]
    fn citation_pins_pub_l_115_97_and_treas_reg_245a_5() {
        let i = baseline();
        let r = check(&i);
        assert!(r.citation.contains("§ 245A(a)-(g)"));
        assert!(r.citation.contains("Pub. L. 115-97 § 14101"));
        assert!(r.citation.contains("Treas. Reg. § 1.245A-5"));
    }

    #[test]
    fn portion_sum_mismatch_generates_warning_note() {
        let mut i = baseline();
        i.total_dividend_cents = 100_000_000_00;
        i.foreign_source_portion_cents = 60_000_000_00;
        i.us_source_portion_cents = 30_000_000_00;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("reconcile")));
        assert!(r.notes.iter().any(|n| n.contains("§ 1.245A-5 anti-abuse")));
    }

    #[test]
    fn zero_foreign_source_zero_deduction() {
        let mut i = baseline();
        i.foreign_source_portion_cents = 0;
        i.us_source_portion_cents = 100_000_000_00;
        let r = check(&i);
        assert_eq!(r.deduction_cents, 0);
        assert!(matches!(
            r.severity,
            Severity::PartialDeductionForeignSourcePortionOnly
        ));
    }

    #[test]
    fn extreme_dividend_does_not_overflow() {
        let mut i = baseline();
        i.foreign_source_portion_cents = u64::MAX / 10_000;
        i.total_dividend_cents = u64::MAX / 10_000;
        let r = check(&i);
        let _ = r.deduction_cents;
    }

    #[test]
    fn zero_dividend_zero_deduction() {
        let mut i = baseline();
        i.total_dividend_cents = 0;
        i.foreign_source_portion_cents = 0;
        i.us_source_portion_cents = 0;
        let r = check(&i);
        assert_eq!(r.deduction_cents, 0);
    }

    #[test]
    fn boundary_2018_post_effective_date() {
        let mut i = baseline();
        i.distribution_year = 2018;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::FullDeductionEligible));
    }

    #[test]
    fn holding_period_check_takes_priority_over_hybrid_dividend() {
        let mut i = baseline();
        i.is_hybrid_dividend = true;
        i.days_held_in_731_day_window = 100;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::HoldingPeriodNotSatisfied));
    }

    #[test]
    fn realistic_microsoft_ireland_repatriation_full_drd() {
        let mut i = baseline();
        i.total_dividend_cents = 50_000_000_000_00;
        i.foreign_source_portion_cents = 50_000_000_000_00;
        i.us_source_portion_cents = 0;
        i.foreign_withholding_tax_cents = 0;
        let r = check(&i);
        assert_eq!(r.deduction_cents, 50_000_000_000_00);
        assert_eq!(r.taxable_dividend_after_deduction_cents, 0);
    }
}
