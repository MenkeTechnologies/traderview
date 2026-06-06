//! IRC § 4978 — Tax on certain dispositions by employee
//! stock ownership plans and certain cooperatives.
//! Companion punitive tax to section_1042 (ESOP capital
//! gain deferral — iter 480) that forces ESOPs receiving
//! qualified securities under § 1042 (or qualified
//! gratuitous transfers under § 664(g)) to hold those
//! securities for at least 3 years after acquisition, or
//! face 10% excise tax on the amount realized. Direct
//! companion to section_1042 (iter 480), section_4940 (PF
//! NII excise — iter 470), section_4941 (PF self-dealing
//! — iter 468), section_4942 (PF minimum distribution —
//! iter 472), section_4943 (PF excess business holdings —
//! iter 474), section_4944 (PF jeopardizing investments —
//! iter 476), section_4945 (PF taxable expenditures —
//! iter 478), section_4958 (intermediate sanctions — iter
//! 466), section_4960 (ATEO executive comp 21% — iter
//! 464), section_4972 (nondeductible plan contributions —
//! iter 482), section_4973 (excess contribution excise —
//! iter 442), section_4974 (RMD excise — iter 436),
//! section_4975 (qualified plan prohibited transactions —
//! iter 434), section_4980 (employer reversion — iter
//! 460), section_4980h (employer shared responsibility —
//! iter 456), section_401k (iter 448), section_415 (iter
//! 452). § 4978 originally enacted by Deficit Reduction
//! Act of 1984, Pub. L. 98-369.
//!
//! § 4978(a) imposes a 10% EXCISE TAX on the AMOUNT
//! REALIZED on the disposition of qualified securities
//! IF the disposition takes place WITHIN 3 YEARS after
//! the date of acquisition by the ESOP or eligible
//! worker-owned cooperative. Tax is paid by the EMPLOYER
//! that maintains the plan.
//!
//! § 4978(b) TRIGGERING CONDITIONS — tax applies if
//! EITHER of these post-disposition conditions is met:
//! 1. § 4978(b)(1) SHARE COUNT TEST: total number of
//!    employer securities held by the plan or cooperative
//!    AFTER the disposition is LESS THAN the total
//!    number of employer securities held immediately
//!    AFTER the § 1042 sale (i.e., the disposition
//!    reduced the ESOP's holdings below the post-§ 1042-
//!    sale level)
//! 2. § 4978(b)(2) 30%-VALUE TEST: value of qualified
//!    securities held by such plan or cooperative AFTER
//!    such disposition is LESS THAN 30 PERCENT of the
//!    total value of all employer securities as of such
//!    disposition (60 PERCENT for qualified employer
//!    securities acquired in a qualified gratuitous
//!    transfer to which § 664(g) applied)
//!
//! § 4978(c) EXCEPTIONS — disposition not subject to tax
//! if:
//! 1. § 4978(c)(1) — distribution to employee on
//!    SEPARATION FROM SERVICE, DEATH, RETIREMENT,
//!    DISABILITY, or DIVORCE
//! 2. § 4978(c)(2) — distribution in connection with
//!    employee STOCK PURCHASE
//! 3. § 4978(c)(3) — disposition by reason of mergers
//!    and reorganizations under § 354, § 355, § 356, and
//!    § 368 (corporate reorganizations); ESOP retains
//!    successor securities
//! 4. § 4978(c)(4) — diversification rights under
//!    § 401(a)(28)(B)
//!
//! § 4978(d) DEFINITIONS: "qualified securities"
//! cross-references § 1042(c)(1); "employer securities"
//! cross-references § 409(l).
//!
//! Form 5330 (Return of Excise Taxes Related to Employee
//! Benefit Plans): EMPLOYER must file annually if § 4978
//! tax applies; filing deadline last day of 7th month
//! after employer tax-year-end.
//!
//! Trader-business-owner critical because (1) § 4978 is
//! the LIFETIME ENFORCEMENT MECHANISM for § 1042 ESOP
//! capital gain deferral — without § 4978 the 30%
//! post-sale ESOP ownership threshold under § 1042(b)(2)
//! could be circumvented by immediate ESOP disposition;
//! (2) § 4978 written consent under § 1042(b)(3) is
//! prerequisite to § 1042 election — employer
//! affirmatively agrees to § 4978 recapture exposure;
//! (3) ESOP loan repayment via employer contributions +
//! corresponding allocation to participant accounts is
//! NOT § 4978 disposition; (4) § 4978(c)(1) distribution-
//! on-separation exception is the most-used exception —
//! routine employee retirements do NOT trigger § 4978;
//! (5) § 4978(c)(3) reorganization exception preserves
//! § 4978 status across corporate transactions; (6) 3-
//! year holding period creates significant transaction
//! design constraint on post-§ 1042 corporate sale or
//! redemption.
//!
//! Distinction from § 1042 (iter 480): § 1042 provides
//! the SELLER capital gain deferral; § 4978 is the
//! EMPLOYER recapture if ESOP doesn't honor 3-year hold.
//! Two regimes are complementary — § 1042 written
//! consent under § 1042(b)(3) is the seller's
//! acknowledgment that the EMPLOYER (not the seller)
//! bears § 4978 risk.
//!
//! Authority: 26 U.S.C. § 4978; § 4978(a); § 4978(b);
//! § 4978(b)(1); § 4978(b)(2); § 4978(c); § 4978(c)(1);
//! § 4978(c)(2); § 4978(c)(3); § 4978(c)(4); § 4978(d);
//! § 1042 (iter 480); § 1042(b)(2); § 1042(b)(3);
//! § 1042(c)(1); § 1042(c)(2); § 664(g) (qualified
//! gratuitous transfer); § 409(l) (employer securities
//! definition); § 401(a)(28)(B) (diversification);
//! § 354; § 355; § 356; § 368 (corporate
//! reorganizations); Form 5330 (Return of Excise Taxes
//! Related to Employee Benefit Plans); Deficit Reduction
//! Act of 1984, Pub. L. 98-369 — original § 4978
//! enactment.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AcquisitionPath {
    Section1042Sale,
    Section664gQualifiedGratuitousTransfer,
    NotApplicable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DispositionType {
    OpenMarketSale,
    SeparationFromServiceDistribution,
    DeathDistribution,
    RetirementDistribution,
    DisabilityDistribution,
    DivorceDistribution,
    EmployeeStockPurchase,
    CorporateReorganization,
    DiversificationUnder401a28B,
    NoDisposition,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub acquisition_path: AcquisitionPath,
    pub disposition_type: DispositionType,
    pub months_since_acquisition: u32,
    pub amount_realized_cents: u64,
    pub esop_shares_after_disposition: u64,
    pub esop_shares_after_initial_acquisition: u64,
    pub esop_qualified_securities_value_after_disposition_cents: u64,
    pub total_employer_securities_value_at_disposition_cents: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    NoDisposition,
    OutsideThreeYearWindow,
    ExceptionApplies,
    NoTriggeringConditionMet,
    ExciseTaxOwed,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub severity: Severity,
    pub excise_tax_cents: u64,
    pub triggering_condition: Option<String>,
    pub notes: Vec<String>,
}

pub const TAX_RATE_PCT: u64 = 10;
pub const THREE_YEAR_WINDOW_MONTHS: u32 = 36;
pub const STANDARD_30_PCT_THRESHOLD_BPS: u32 = 3000;
pub const SECTION_664G_60_PCT_THRESHOLD_BPS: u32 = 6000;

pub type Section4978Input = Input;
pub type Section4978Result = Output;

pub fn check(input: &Input) -> Output {
    let notes: Vec<String> = vec![
        "§ 4978(a) imposes 10% EXCISE TAX on AMOUNT REALIZED on disposition of qualified securities by ESOP or eligible worker-owned cooperative WITHIN 3 YEARS after acquisition. Tax paid by EMPLOYER that maintains the plan.".to_string(),
        "§ 4978(b) TRIGGERING CONDITIONS — tax applies if EITHER: (1) § 4978(b)(1) SHARE COUNT TEST: total employer securities held by plan AFTER disposition LESS THAN total held immediately AFTER the § 1042 sale; (2) § 4978(b)(2) 30%-VALUE TEST: value of qualified securities held AFTER disposition LESS THAN 30% of total employer securities value at disposition (60% for § 664(g) qualified gratuitous transfer).".to_string(),
        "§ 4978(c) EXCEPTIONS — disposition NOT subject to tax: (1) § 4978(c)(1) distribution on SEPARATION FROM SERVICE / DEATH / RETIREMENT / DISABILITY / DIVORCE; (2) § 4978(c)(2) distribution in connection with employee STOCK PURCHASE; (3) § 4978(c)(3) disposition by reason of merger or reorganization under § 354 + § 355 + § 356 + § 368 — ESOP retains successor securities; (4) § 4978(c)(4) diversification rights under § 401(a)(28)(B).".to_string(),
        "§ 4978(d) DEFINITIONS: 'qualified securities' cross-references § 1042(c)(1); 'employer securities' cross-references § 409(l). § 1042(b)(3) WRITTEN CONSENT to § 4978 recapture is prerequisite to seller's § 1042 capital gain deferral election.".to_string(),
        "Form 5330 (Return of Excise Taxes Related to Employee Benefit Plans): EMPLOYER must file annually if § 4978 tax applies; filing deadline last day of 7th month after employer tax-year-end.".to_string(),
        "Distinction from § 1042 (iter 480): § 1042 provides SELLER capital gain deferral; § 4978 is EMPLOYER recapture if ESOP fails 3-year hold. Two regimes are complementary — § 1042 written consent under § 1042(b)(3) is seller's acknowledgment that the EMPLOYER (not the seller) bears § 4978 risk.".to_string(),
        "Companion: section_1042 (iter 480), section_4940 (iter 470), section_4941 (iter 468), section_4942 (iter 472), section_4943 (iter 474), section_4944 (iter 476), section_4945 (iter 478), section_4958 (iter 466), section_4960 (iter 464), section_4972 (iter 482), section_4973 (iter 442), section_4974 (iter 436), section_4975 (iter 434), section_4980 (iter 460), section_4980h (iter 456), section_401k (iter 448), section_415 (iter 452).".to_string(),
    ];

    if matches!(input.acquisition_path, AcquisitionPath::NotApplicable) {
        let mut n = notes;
        n.push("Acquisition not via § 1042 sale or § 664(g) qualified gratuitous transfer — § 4978 does not apply.".to_string());
        return Output {
            severity: Severity::NotApplicable,
            excise_tax_cents: 0,
            triggering_condition: None,
            notes: n,
        };
    }

    if matches!(input.disposition_type, DispositionType::NoDisposition) {
        let mut n = notes;
        n.push(
            "No disposition of qualified securities — § 4978 does not apply this taxable year."
                .to_string(),
        );
        return Output {
            severity: Severity::NoDisposition,
            excise_tax_cents: 0,
            triggering_condition: None,
            notes: n,
        };
    }

    if input.months_since_acquisition > THREE_YEAR_WINDOW_MONTHS {
        let mut n = notes;
        n.push(format!(
            "Disposition occurred {} months after acquisition, outside § 4978 3-YEAR window (36 months) — § 4978 does not apply.",
            input.months_since_acquisition
        ));
        return Output {
            severity: Severity::OutsideThreeYearWindow,
            excise_tax_cents: 0,
            triggering_condition: None,
            notes: n,
        };
    }

    let exception_applies = matches!(
        input.disposition_type,
        DispositionType::SeparationFromServiceDistribution
            | DispositionType::DeathDistribution
            | DispositionType::RetirementDistribution
            | DispositionType::DisabilityDistribution
            | DispositionType::DivorceDistribution
            | DispositionType::EmployeeStockPurchase
            | DispositionType::CorporateReorganization
            | DispositionType::DiversificationUnder401a28B
    );

    if exception_applies {
        let mut n = notes;
        let citation = match input.disposition_type {
            DispositionType::SeparationFromServiceDistribution
            | DispositionType::DeathDistribution
            | DispositionType::RetirementDistribution
            | DispositionType::DisabilityDistribution
            | DispositionType::DivorceDistribution => {
                "§ 4978(c)(1) distribution on separation/death/retirement/disability/divorce"
            }
            DispositionType::EmployeeStockPurchase => {
                "§ 4978(c)(2) distribution in connection with employee stock purchase"
            }
            DispositionType::CorporateReorganization => {
                "§ 4978(c)(3) merger or reorganization under § 354 + § 355 + § 356 + § 368"
            }
            DispositionType::DiversificationUnder401a28B => {
                "§ 4978(c)(4) diversification rights under § 401(a)(28)(B)"
            }
            _ => unreachable!(),
        };
        n.push(format!(
            "§ 4978(c) exception applies: {} — disposition not subject to § 4978 excise tax.",
            citation
        ));
        return Output {
            severity: Severity::ExceptionApplies,
            excise_tax_cents: 0,
            triggering_condition: None,
            notes: n,
        };
    }

    // Test § 4978(b)(1): share count test
    let share_count_triggered =
        input.esop_shares_after_disposition < input.esop_shares_after_initial_acquisition;

    // Test § 4978(b)(2): 30%-value test (60% for § 664(g))
    let value_threshold_bps = if matches!(
        input.acquisition_path,
        AcquisitionPath::Section664gQualifiedGratuitousTransfer
    ) {
        SECTION_664G_60_PCT_THRESHOLD_BPS
    } else {
        STANDARD_30_PCT_THRESHOLD_BPS
    };
    let post_disp_value_bps = if input.total_employer_securities_value_at_disposition_cents > 0 {
        ((input.esop_qualified_securities_value_after_disposition_cents as u128)
            .saturating_mul(10_000)
            / input.total_employer_securities_value_at_disposition_cents as u128) as u32
    } else {
        0
    };
    let value_test_triggered = post_disp_value_bps < value_threshold_bps;

    if !share_count_triggered && !value_test_triggered {
        let mut n = notes;
        n.push("Neither § 4978(b)(1) share count test NOR § 4978(b)(2) value test triggered — disposition does not trigger § 4978 excise tax.".to_string());
        return Output {
            severity: Severity::NoTriggeringConditionMet,
            excise_tax_cents: 0,
            triggering_condition: None,
            notes: n,
        };
    }

    let excise_tax = input
        .amount_realized_cents
        .saturating_mul(TAX_RATE_PCT)
        .checked_div(100)
        .unwrap_or(0);

    let triggering_condition = if share_count_triggered && value_test_triggered {
        Some("BOTH § 4978(b)(1) share count test AND § 4978(b)(2) value test triggered".to_string())
    } else if share_count_triggered {
        Some("§ 4978(b)(1) share count test triggered".to_string())
    } else {
        Some(format!(
            "§ 4978(b)(2) value test triggered: post-disposition value {} bps below {} bps threshold",
            post_disp_value_bps, value_threshold_bps
        ))
    };

    let mut n = notes;
    n.push(format!(
        "§ 4978 10% excise tax: amount realized ${}.{:02} × 10% = ${}.{:02}; triggering condition: {}.",
        input.amount_realized_cents / 100,
        input.amount_realized_cents % 100,
        excise_tax / 100,
        excise_tax % 100,
        triggering_condition.as_ref().unwrap()
    ));

    Output {
        severity: Severity::ExciseTaxOwed,
        excise_tax_cents: excise_tax,
        triggering_condition,
        notes: n,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> Input {
        Input {
            acquisition_path: AcquisitionPath::Section1042Sale,
            disposition_type: DispositionType::OpenMarketSale,
            months_since_acquisition: 12,
            amount_realized_cents: 10_000_000_00, // $10M
            esop_shares_after_disposition: 500,   // share count dropped
            esop_shares_after_initial_acquisition: 1000,
            esop_qualified_securities_value_after_disposition_cents: 10_000_000_00, // $10M (50% of $20M)
            total_employer_securities_value_at_disposition_cents: 20_000_000_00,
        }
    }

    #[test]
    fn not_applicable_acquisition_path() {
        let mut i = baseline();
        i.acquisition_path = AcquisitionPath::NotApplicable;
        let out = check(&i);
        assert_eq!(out.severity, Severity::NotApplicable);
    }

    #[test]
    fn no_disposition_returns_no_disposition() {
        let mut i = baseline();
        i.disposition_type = DispositionType::NoDisposition;
        let out = check(&i);
        assert_eq!(out.severity, Severity::NoDisposition);
    }

    #[test]
    fn outside_3_year_window_37_months() {
        let mut i = baseline();
        i.months_since_acquisition = 37;
        let out = check(&i);
        assert_eq!(out.severity, Severity::OutsideThreeYearWindow);
    }

    #[test]
    fn exactly_36_months_still_in_window() {
        let mut i = baseline();
        i.months_since_acquisition = 36;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExciseTaxOwed);
    }

    #[test]
    fn separation_from_service_exception() {
        let mut i = baseline();
        i.disposition_type = DispositionType::SeparationFromServiceDistribution;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExceptionApplies);
        let joined = out.notes.join(" ");
        assert!(joined.contains("§ 4978(c)(1)"));
    }

    #[test]
    fn death_exception() {
        let mut i = baseline();
        i.disposition_type = DispositionType::DeathDistribution;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExceptionApplies);
    }

    #[test]
    fn retirement_exception() {
        let mut i = baseline();
        i.disposition_type = DispositionType::RetirementDistribution;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExceptionApplies);
    }

    #[test]
    fn disability_exception() {
        let mut i = baseline();
        i.disposition_type = DispositionType::DisabilityDistribution;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExceptionApplies);
    }

    #[test]
    fn divorce_exception() {
        let mut i = baseline();
        i.disposition_type = DispositionType::DivorceDistribution;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExceptionApplies);
    }

    #[test]
    fn employee_stock_purchase_exception() {
        let mut i = baseline();
        i.disposition_type = DispositionType::EmployeeStockPurchase;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExceptionApplies);
        let joined = out.notes.join(" ");
        assert!(joined.contains("§ 4978(c)(2)"));
    }

    #[test]
    fn corporate_reorganization_exception() {
        let mut i = baseline();
        i.disposition_type = DispositionType::CorporateReorganization;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExceptionApplies);
        let joined = out.notes.join(" ");
        assert!(joined.contains("§ 4978(c)(3)"));
        assert!(joined.contains("§ 354"));
        assert!(joined.contains("§ 368"));
    }

    #[test]
    fn diversification_exception() {
        let mut i = baseline();
        i.disposition_type = DispositionType::DiversificationUnder401a28B;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExceptionApplies);
        let joined = out.notes.join(" ");
        assert!(joined.contains("§ 4978(c)(4)"));
        assert!(joined.contains("§ 401(a)(28)(B)"));
    }

    #[test]
    fn share_count_test_triggered_tax_owed() {
        let i = baseline(); // shares dropped 1000 → 500, triggers
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExciseTaxOwed);
        // 10% × $10M = $1M
        assert_eq!(out.excise_tax_cents, 1_000_000_00);
    }

    #[test]
    fn neither_test_triggered_no_tax() {
        let mut i = baseline();
        // Shares maintained at initial
        i.esop_shares_after_disposition = 1000;
        i.esop_shares_after_initial_acquisition = 1000;
        // Value > 30% threshold
        i.esop_qualified_securities_value_after_disposition_cents = 15_000_000_00; // 75%
        i.total_employer_securities_value_at_disposition_cents = 20_000_000_00;
        let out = check(&i);
        assert_eq!(out.severity, Severity::NoTriggeringConditionMet);
        assert_eq!(out.excise_tax_cents, 0);
    }

    #[test]
    fn thirty_percent_value_test_triggered() {
        let mut i = baseline();
        i.esop_shares_after_disposition = 1000; // no share drop
        i.esop_shares_after_initial_acquisition = 1000;
        // Post-disposition value $5M / $20M = 25% < 30%
        i.esop_qualified_securities_value_after_disposition_cents = 5_000_000_00;
        i.total_employer_securities_value_at_disposition_cents = 20_000_000_00;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExciseTaxOwed);
        let joined = out.notes.join(" ");
        assert!(joined.contains("§ 4978(b)(2) value test"));
    }

    #[test]
    fn exactly_30_percent_value_test_no_trigger() {
        let mut i = baseline();
        i.esop_shares_after_disposition = 1000;
        i.esop_shares_after_initial_acquisition = 1000;
        // Exactly 30% = $6M / $20M = 3000 bps; less-than-threshold so 3000 NOT less-than 3000 means not triggered
        i.esop_qualified_securities_value_after_disposition_cents = 6_000_000_00;
        i.total_employer_securities_value_at_disposition_cents = 20_000_000_00;
        let out = check(&i);
        assert_eq!(out.severity, Severity::NoTriggeringConditionMet);
    }

    #[test]
    fn section_664g_60_percent_threshold() {
        let mut i = baseline();
        i.acquisition_path = AcquisitionPath::Section664gQualifiedGratuitousTransfer;
        i.esop_shares_after_disposition = 1000;
        i.esop_shares_after_initial_acquisition = 1000;
        // 50% < 60% threshold for § 664(g)
        i.esop_qualified_securities_value_after_disposition_cents = 10_000_000_00;
        i.total_employer_securities_value_at_disposition_cents = 20_000_000_00;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExciseTaxOwed);
        let joined = out.notes.join(" ");
        assert!(joined.contains("6000 bps threshold"));
    }

    #[test]
    fn section_664g_at_60_percent_no_trigger() {
        let mut i = baseline();
        i.acquisition_path = AcquisitionPath::Section664gQualifiedGratuitousTransfer;
        i.esop_shares_after_disposition = 1000;
        i.esop_shares_after_initial_acquisition = 1000;
        // Exactly 60%
        i.esop_qualified_securities_value_after_disposition_cents = 12_000_000_00;
        i.total_employer_securities_value_at_disposition_cents = 20_000_000_00;
        let out = check(&i);
        assert_eq!(out.severity, Severity::NoTriggeringConditionMet);
    }

    #[test]
    fn both_tests_triggered() {
        let mut i = baseline(); // shares 1000 → 500 + value $10M / $20M = 50%
                                // Drop value below 30% to trigger value test too
        i.esop_qualified_securities_value_after_disposition_cents = 4_000_000_00;
        i.total_employer_securities_value_at_disposition_cents = 20_000_000_00;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExciseTaxOwed);
        let joined = out.notes.join(" ");
        assert!(joined.contains("BOTH"));
    }

    #[test]
    fn citation_pins_all_authorities() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("§ 4978(a)"));
        assert!(joined.contains("§ 4978(b)"));
        assert!(joined.contains("§ 4978(b)(1)"));
        assert!(joined.contains("§ 4978(b)(2)"));
        assert!(joined.contains("§ 4978(c)"));
        assert!(joined.contains("§ 4978(c)(1)"));
        assert!(joined.contains("§ 4978(c)(2)"));
        assert!(joined.contains("§ 4978(c)(3)"));
        assert!(joined.contains("§ 4978(c)(4)"));
        assert!(joined.contains("§ 4978(d)"));
        assert!(joined.contains("§ 1042 (iter 480)"));
        assert!(joined.contains("§ 1042(b)(3)"));
        assert!(joined.contains("§ 1042(c)(1)"));
        assert!(joined.contains("§ 664(g)"));
        assert!(joined.contains("§ 409(l)"));
        assert!(joined.contains("§ 401(a)(28)(B)"));
        assert!(joined.contains("§ 354"));
        assert!(joined.contains("§ 355"));
        assert!(joined.contains("§ 356"));
        assert!(joined.contains("§ 368"));
        assert!(joined.contains("Form 5330"));
    }

    #[test]
    fn note_pins_10_percent_rate_and_3_year_window() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("10% EXCISE TAX"));
        assert!(joined.contains("3 YEARS"));
        assert!(joined.contains("EMPLOYER"));
    }

    #[test]
    fn note_pins_two_triggering_conditions() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("SHARE COUNT TEST"));
        assert!(joined.contains("30%-VALUE TEST"));
        assert!(joined.contains("60% for § 664(g)"));
    }

    #[test]
    fn note_pins_four_exception_categories() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("SEPARATION FROM SERVICE"));
        assert!(joined.contains("DEATH"));
        assert!(joined.contains("RETIREMENT"));
        assert!(joined.contains("DISABILITY"));
        assert!(joined.contains("DIVORCE"));
        assert!(joined.contains("STOCK PURCHASE"));
        assert!(joined.contains("merger or reorganization"));
        assert!(joined.contains("diversification"));
    }

    #[test]
    fn note_pins_1042_distinction() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("§ 1042 (iter 480)"));
        assert!(joined.contains("SELLER capital gain deferral"));
        assert!(joined.contains("EMPLOYER recapture"));
    }

    #[test]
    fn note_pins_form_5330_filing() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("Form 5330"));
        assert!(joined.contains("7th month"));
    }

    #[test]
    fn note_pins_companion_modules() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("section_1042"));
        assert!(joined.contains("section_4972"));
        assert!(joined.contains("section_4973"));
        assert!(joined.contains("section_4974"));
        assert!(joined.contains("section_4975"));
    }

    #[test]
    fn truth_table_six_severity_cells() {
        // NotApplicable
        let c1 = check(&Input {
            acquisition_path: AcquisitionPath::NotApplicable,
            ..baseline()
        });
        assert_eq!(c1.severity, Severity::NotApplicable);

        // NoDisposition
        let c2 = check(&Input {
            disposition_type: DispositionType::NoDisposition,
            ..baseline()
        });
        assert_eq!(c2.severity, Severity::NoDisposition);

        // OutsideThreeYearWindow
        let c3 = check(&Input {
            months_since_acquisition: 48,
            ..baseline()
        });
        assert_eq!(c3.severity, Severity::OutsideThreeYearWindow);

        // ExceptionApplies
        let c4 = check(&Input {
            disposition_type: DispositionType::RetirementDistribution,
            ..baseline()
        });
        assert_eq!(c4.severity, Severity::ExceptionApplies);

        // NoTriggeringConditionMet
        let c5 = check(&Input {
            esop_shares_after_disposition: 1000,
            esop_shares_after_initial_acquisition: 1000,
            esop_qualified_securities_value_after_disposition_cents: 15_000_000_00,
            ..baseline()
        });
        assert_eq!(c5.severity, Severity::NoTriggeringConditionMet);

        // ExciseTaxOwed
        let c6 = check(&baseline());
        assert_eq!(c6.severity, Severity::ExciseTaxOwed);
    }

    #[test]
    fn saturating_arithmetic_overflow_defense() {
        let i = Input {
            amount_realized_cents: u64::MAX,
            ..baseline()
        };
        let out = check(&i);
        // No panic; saturating arithmetic
        assert_eq!(out.severity, Severity::ExciseTaxOwed);
    }

    #[test]
    fn realistic_50m_premature_disposition_5m_tax() {
        // Founder sold $50M to ESOP in Year 1 under § 1042; ESOP disposes
        // $50M of qualified securities in Year 2 (within 3-year window).
        let mut i = baseline();
        i.amount_realized_cents = 50_000_000_00;
        i.esop_shares_after_disposition = 500;
        i.esop_shares_after_initial_acquisition = 1500;
        i.esop_qualified_securities_value_after_disposition_cents = 0;
        i.total_employer_securities_value_at_disposition_cents = 100_000_000_00;
        i.months_since_acquisition = 18;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExciseTaxOwed);
        // 10% × $50M = $5M
        assert_eq!(out.excise_tax_cents, 5_000_000_00);
    }

    #[test]
    fn boundary_one_cent_realized() {
        let mut i = baseline();
        i.amount_realized_cents = 1;
        let out = check(&i);
        // 10% of 1 cent = 0
        assert_eq!(out.excise_tax_cents, 0);
    }

    #[test]
    fn share_count_test_alone_triggers_even_with_high_value() {
        let mut i = baseline();
        // Shares dropped (triggers (b)(1)) but value stayed > 30%
        i.esop_shares_after_disposition = 900;
        i.esop_shares_after_initial_acquisition = 1000;
        i.esop_qualified_securities_value_after_disposition_cents = 15_000_000_00; // 75%
        i.total_employer_securities_value_at_disposition_cents = 20_000_000_00;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExciseTaxOwed);
        let joined = out.notes.join(" ");
        assert!(joined.contains("§ 4978(b)(1) share count test triggered"));
    }
}
