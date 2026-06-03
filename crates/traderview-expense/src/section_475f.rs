//! IRC §475(f) trader mark-to-market election mechanics.
//!
//! Companion to `section_475c2` (dealer-vs-trader-vs-investor classification).
//! This module pins the ELECTION mechanics for taxpayers who qualify as a
//! "trader in securities" under §475(f)(1) and want MTM treatment:
//! April-15 (or March-15 entity) election-statement deadline; mandatory
//! Form 3115 (Application for Change in Accounting Method) on the
//! year-of-election return; conversion of capital character to ordinary
//! character; full wash-sale-rule (§1091) exemption; removal of the
//! $3,000 §1211(b) capital-loss cap on ordinary-loss-converted positions;
//! and the 5-year revocation lockout under Rev. Proc. 99-17 + Rev. Proc.
//! 2025-23 § 24.02.
//!
//! Trader-tax-status floors (substantial + regular + continuous test, per
//! Endicott v. Commissioner T.C. Memo. 2013-199 + IRS Pub. 550) require
//! at least ~720 trades per year and ~4 hours per trading day; these are
//! facts-and-circumstances thresholds the IRS applies in audit, not
//! statutory minimums.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[allow(dead_code)]
pub const TYPICAL_TRADER_SUBSTANTIAL_TRADES_FLOOR_PER_YEAR: u64 = 720;
#[allow(dead_code)]
pub const TYPICAL_TRADER_SUBSTANTIAL_HOURS_PER_TRADING_DAY: u32 = 4;
#[allow(dead_code)]
pub const ELECTION_DEADLINE_APRIL_15_INDIVIDUAL_DAY_OF_MONTH: u32 = 15;
#[allow(dead_code)]
pub const ELECTION_DEADLINE_MARCH_15_ENTITY_DAY_OF_MONTH: u32 = 15;
#[allow(dead_code)]
pub const REVOCATION_LOCKOUT_YEARS_AFTER_PRIOR_ELECTION_REVOKED: u32 = 5;
#[allow(dead_code)]
pub const SECTION_1211B_CAPITAL_LOSS_CAP_CENTS_PER_YEAR: u64 = 300_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilerEntityType {
    NotApplicable,
    IndividualSchedC,
    SCorp,
    Partnership,
    NewlyFormedEntityFirstYear,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraderTaxStatusQualification {
    NotApplicable,
    QualifiesTtsSubstantialRegularContinuous,
    DoesNotQualifyInsufficientActivity,
    BorderlineFactsAndCircumstancesAuditRisk,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ElectionStatementFilingStatus {
    NotApplicable,
    FiledByOriginalDueDateWithoutExtensions,
    MissedAprilOrMarchDeadlineFatal,
    NewEntityElectionInternalBooksWithin2_5Months,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Form3115FilingStatus {
    NotApplicable,
    Form3115WillBeFiledWithReturn,
    Form3115NotFiledMissingAccountingMethodChange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PriorRevocationStatus {
    NotApplicable,
    NoPriorElectionOrRevocation,
    PriorElectionRevokedWithin5YearsLocked,
    PriorElectionRevokedMoreThan5YearsAgoEligibleAgain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    TraderQualifiesElectionTimelyForm3115FiledOptimal,
    TraderQualifiesElectionMissedDeadlineFatalLossUntilNextTaxYear,
    TraderQualifiesElectionFiledButForm3115MissingAccountingMethodViolation,
    TraderDoesNotQualifyTtsCannotElect475f,
    PriorElectionRevokedWithin5YearsLockedCannotReElect,
    NewEntityFirstYearInternalBooksElectionWindow2_5Months,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub filer_entity_type: FilerEntityType,
    pub annual_trades: u64,
    pub hours_per_trading_day: u32,
    pub election_statement: ElectionStatementFilingStatus,
    pub form_3115_status: Form3115FilingStatus,
    pub prior_revocation: PriorRevocationStatus,
    pub net_securities_loss_cents: u64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Output {
    pub severity: Severity,
    pub trader_tax_status: TraderTaxStatusQualification,
    pub wash_sale_exemption_applies: bool,
    pub ordinary_loss_treatment_unlimited: bool,
    pub section_1211b_3000_cap_removed: bool,
    pub deductible_loss_against_ordinary_income_cents: u64,
    pub notes: Vec<String>,
    pub citations: Vec<String>,
}

pub type Section475fInput = Input;
pub type Section475fOutput = Output;
pub type Section475fResult = Output;

pub fn check(input: &Input) -> Output {
    let mut notes: Vec<String> = Vec::new();
    let citations: Vec<String> = vec![
        "IRC § 475(f)".to_string(),
        "IRC § 475(c)(2)".to_string(),
        "IRC § 1091 (wash-sale rule)".to_string(),
        "IRC § 1211(b)".to_string(),
        "Rev. Proc. 99-17".to_string(),
        "Rev. Proc. 2025-23 § 24.01-.02".to_string(),
        "Form 3115 (Application for Change in Accounting Method)".to_string(),
        "Endicott v. Commissioner, T.C. Memo. 2013-199".to_string(),
        "IRS Topic 429 / Pub. 550 (Traders in Securities)".to_string(),
    ];

    let tts = classify_tts(input);

    if matches!(tts, TraderTaxStatusQualification::DoesNotQualifyInsufficientActivity) {
        notes.push(format!(
            "TTS floor missed: below {} trades/year or below {} hours/day. Cannot elect § 475(f) without trader-in-securities status.",
            TYPICAL_TRADER_SUBSTANTIAL_TRADES_FLOOR_PER_YEAR,
            TYPICAL_TRADER_SUBSTANTIAL_HOURS_PER_TRADING_DAY
        ));
        return Output {
            severity: Severity::TraderDoesNotQualifyTtsCannotElect475f,
            trader_tax_status: tts,
            wash_sale_exemption_applies: false,
            ordinary_loss_treatment_unlimited: false,
            section_1211b_3000_cap_removed: false,
            deductible_loss_against_ordinary_income_cents: SECTION_1211B_CAPITAL_LOSS_CAP_CENTS_PER_YEAR
                .min(input.net_securities_loss_cents),
            notes,
            citations,
        };
    }

    if matches!(
        input.prior_revocation,
        PriorRevocationStatus::PriorElectionRevokedWithin5YearsLocked
    ) {
        notes.push(format!(
            "Prior § 475(f) revocation within {} years — re-election locked out per Rev. Proc. 99-17 + 2025-23.",
            REVOCATION_LOCKOUT_YEARS_AFTER_PRIOR_ELECTION_REVOKED
        ));
        return Output {
            severity: Severity::PriorElectionRevokedWithin5YearsLockedCannotReElect,
            trader_tax_status: tts,
            wash_sale_exemption_applies: false,
            ordinary_loss_treatment_unlimited: false,
            section_1211b_3000_cap_removed: false,
            deductible_loss_against_ordinary_income_cents: SECTION_1211B_CAPITAL_LOSS_CAP_CENTS_PER_YEAR
                .min(input.net_securities_loss_cents),
            notes,
            citations,
        };
    }

    if matches!(
        input.filer_entity_type,
        FilerEntityType::NewlyFormedEntityFirstYear
    ) && matches!(
        input.election_statement,
        ElectionStatementFilingStatus::NewEntityElectionInternalBooksWithin2_5Months
    ) {
        notes.push("Newly-formed entity first-year election: internal-books statement within 2.5 months of formation per Rev. Proc. 99-17.".to_string());
        return Output {
            severity: Severity::NewEntityFirstYearInternalBooksElectionWindow2_5Months,
            trader_tax_status: tts,
            wash_sale_exemption_applies: true,
            ordinary_loss_treatment_unlimited: true,
            section_1211b_3000_cap_removed: true,
            deductible_loss_against_ordinary_income_cents: input.net_securities_loss_cents,
            notes,
            citations,
        };
    }

    if matches!(
        input.election_statement,
        ElectionStatementFilingStatus::MissedAprilOrMarchDeadlineFatal
    ) {
        let deadline_day = match input.filer_entity_type {
            FilerEntityType::IndividualSchedC => {
                ELECTION_DEADLINE_APRIL_15_INDIVIDUAL_DAY_OF_MONTH
            }
            FilerEntityType::SCorp | FilerEntityType::Partnership => {
                ELECTION_DEADLINE_MARCH_15_ENTITY_DAY_OF_MONTH
            }
            _ => ELECTION_DEADLINE_APRIL_15_INDIVIDUAL_DAY_OF_MONTH,
        };
        notes.push(format!(
            "Missed the unextended due date ({}-month day-{} deadline). § 475(f) election unavailable for this tax year; capital loss capped at ${}.",
            match input.filer_entity_type {
                FilerEntityType::SCorp | FilerEntityType::Partnership => "March",
                _ => "April",
            },
            deadline_day,
            SECTION_1211B_CAPITAL_LOSS_CAP_CENTS_PER_YEAR / 100
        ));
        return Output {
            severity: Severity::TraderQualifiesElectionMissedDeadlineFatalLossUntilNextTaxYear,
            trader_tax_status: tts,
            wash_sale_exemption_applies: false,
            ordinary_loss_treatment_unlimited: false,
            section_1211b_3000_cap_removed: false,
            deductible_loss_against_ordinary_income_cents: SECTION_1211B_CAPITAL_LOSS_CAP_CENTS_PER_YEAR
                .min(input.net_securities_loss_cents),
            notes,
            citations,
        };
    }

    if matches!(
        input.election_statement,
        ElectionStatementFilingStatus::FiledByOriginalDueDateWithoutExtensions
    ) && matches!(
        input.form_3115_status,
        Form3115FilingStatus::Form3115NotFiledMissingAccountingMethodChange
    ) {
        notes.push("Election statement timely filed but Form 3115 omitted from year-of-election return — accounting-method change incomplete per Rev. Proc. 2025-23 § 24.01.".to_string());
        return Output {
            severity: Severity::TraderQualifiesElectionFiledButForm3115MissingAccountingMethodViolation,
            trader_tax_status: tts,
            wash_sale_exemption_applies: false,
            ordinary_loss_treatment_unlimited: false,
            section_1211b_3000_cap_removed: false,
            deductible_loss_against_ordinary_income_cents: SECTION_1211B_CAPITAL_LOSS_CAP_CENTS_PER_YEAR
                .min(input.net_securities_loss_cents),
            notes,
            citations,
        };
    }

    if matches!(
        input.election_statement,
        ElectionStatementFilingStatus::FiledByOriginalDueDateWithoutExtensions
    ) && matches!(
        input.form_3115_status,
        Form3115FilingStatus::Form3115WillBeFiledWithReturn
    ) {
        notes.push("§ 475(f) election timely + Form 3115 filed: wash-sale rule (§ 1091) inapplicable; ordinary character; $3,000 § 1211(b) cap removed; unlimited ordinary loss against other income.".to_string());
        return Output {
            severity: Severity::TraderQualifiesElectionTimelyForm3115FiledOptimal,
            trader_tax_status: tts,
            wash_sale_exemption_applies: true,
            ordinary_loss_treatment_unlimited: true,
            section_1211b_3000_cap_removed: true,
            deductible_loss_against_ordinary_income_cents: input.net_securities_loss_cents,
            notes,
            citations,
        };
    }

    notes.push("No election action recorded; capital character + § 1211(b) cap apply by default.".to_string());
    Output {
        severity: Severity::TraderDoesNotQualifyTtsCannotElect475f,
        trader_tax_status: tts,
        wash_sale_exemption_applies: false,
        ordinary_loss_treatment_unlimited: false,
        section_1211b_3000_cap_removed: false,
        deductible_loss_against_ordinary_income_cents: SECTION_1211B_CAPITAL_LOSS_CAP_CENTS_PER_YEAR
            .min(input.net_securities_loss_cents),
        notes,
        citations,
    }
}

fn classify_tts(input: &Input) -> TraderTaxStatusQualification {
    if input.annual_trades >= TYPICAL_TRADER_SUBSTANTIAL_TRADES_FLOOR_PER_YEAR
        && input.hours_per_trading_day >= TYPICAL_TRADER_SUBSTANTIAL_HOURS_PER_TRADING_DAY
    {
        TraderTaxStatusQualification::QualifiesTtsSubstantialRegularContinuous
    } else if input.annual_trades >= TYPICAL_TRADER_SUBSTANTIAL_TRADES_FLOOR_PER_YEAR / 2
        || input.hours_per_trading_day >= TYPICAL_TRADER_SUBSTANTIAL_HOURS_PER_TRADING_DAY / 2
    {
        TraderTaxStatusQualification::BorderlineFactsAndCircumstancesAuditRisk
    } else {
        TraderTaxStatusQualification::DoesNotQualifyInsufficientActivity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Input {
        Input {
            filer_entity_type: FilerEntityType::IndividualSchedC,
            annual_trades: 1_000,
            hours_per_trading_day: 6,
            election_statement: ElectionStatementFilingStatus::FiledByOriginalDueDateWithoutExtensions,
            form_3115_status: Form3115FilingStatus::Form3115WillBeFiledWithReturn,
            prior_revocation: PriorRevocationStatus::NoPriorElectionOrRevocation,
            net_securities_loss_cents: 5_000_000,
        }
    }

    #[test]
    fn timely_election_with_form_3115_is_optimal() {
        let out = check(&base());
        assert_eq!(
            out.severity,
            Severity::TraderQualifiesElectionTimelyForm3115FiledOptimal
        );
        assert!(out.wash_sale_exemption_applies);
        assert!(out.ordinary_loss_treatment_unlimited);
        assert!(out.section_1211b_3000_cap_removed);
        assert_eq!(out.deductible_loss_against_ordinary_income_cents, 5_000_000);
    }

    #[test]
    fn missed_april_15_deadline_is_fatal_for_individual() {
        let mut i = base();
        i.election_statement = ElectionStatementFilingStatus::MissedAprilOrMarchDeadlineFatal;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::TraderQualifiesElectionMissedDeadlineFatalLossUntilNextTaxYear
        );
        assert!(!out.wash_sale_exemption_applies);
        assert!(!out.section_1211b_3000_cap_removed);
        assert_eq!(out.deductible_loss_against_ordinary_income_cents, 300_000);
        assert!(out.notes.iter().any(|n| n.contains("April")));
    }

    #[test]
    fn missed_march_15_deadline_for_entity_pins_march() {
        let mut i = base();
        i.filer_entity_type = FilerEntityType::SCorp;
        i.election_statement = ElectionStatementFilingStatus::MissedAprilOrMarchDeadlineFatal;
        let out = check(&i);
        assert!(out.notes.iter().any(|n| n.contains("March")));
    }

    #[test]
    fn election_filed_but_form_3115_missing_is_accounting_method_violation() {
        let mut i = base();
        i.form_3115_status = Form3115FilingStatus::Form3115NotFiledMissingAccountingMethodChange;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::TraderQualifiesElectionFiledButForm3115MissingAccountingMethodViolation
        );
        assert!(!out.wash_sale_exemption_applies);
        assert!(out.notes.iter().any(|n| n.contains("Form 3115")));
    }

    #[test]
    fn does_not_qualify_tts_below_trade_floor() {
        let mut i = base();
        i.annual_trades = 100;
        i.hours_per_trading_day = 1;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::TraderDoesNotQualifyTtsCannotElect475f
        );
        assert_eq!(out.deductible_loss_against_ordinary_income_cents, 300_000);
    }

    #[test]
    fn prior_revocation_within_5_years_locks_re_election() {
        let mut i = base();
        i.prior_revocation = PriorRevocationStatus::PriorElectionRevokedWithin5YearsLocked;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::PriorElectionRevokedWithin5YearsLockedCannotReElect
        );
        assert!(!out.section_1211b_3000_cap_removed);
        assert!(out.notes.iter().any(|n| n.contains("5") && n.contains("years")));
    }

    #[test]
    fn newly_formed_entity_internal_books_2_5_month_window() {
        let mut i = base();
        i.filer_entity_type = FilerEntityType::NewlyFormedEntityFirstYear;
        i.election_statement = ElectionStatementFilingStatus::NewEntityElectionInternalBooksWithin2_5Months;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::NewEntityFirstYearInternalBooksElectionWindow2_5Months
        );
        assert!(out.wash_sale_exemption_applies);
        assert!(out.section_1211b_3000_cap_removed);
    }

    #[test]
    fn loss_below_3000_cap_uses_actual_loss_when_no_election() {
        let mut i = base();
        i.annual_trades = 10;
        i.hours_per_trading_day = 1;
        i.net_securities_loss_cents = 100_000;
        let out = check(&i);
        assert_eq!(out.deductible_loss_against_ordinary_income_cents, 100_000);
    }

    #[test]
    fn loss_above_3000_cap_clamped_when_no_election() {
        let mut i = base();
        i.annual_trades = 10;
        i.hours_per_trading_day = 1;
        i.net_securities_loss_cents = 10_000_000;
        let out = check(&i);
        assert_eq!(out.deductible_loss_against_ordinary_income_cents, 300_000);
    }

    #[test]
    fn borderline_tts_does_not_block_election_path() {
        let mut i = base();
        i.annual_trades = 400;
        i.hours_per_trading_day = 3;
        let out = check(&i);
        assert_eq!(
            out.trader_tax_status,
            TraderTaxStatusQualification::BorderlineFactsAndCircumstancesAuditRisk
        );
        assert_eq!(
            out.severity,
            Severity::TraderQualifiesElectionTimelyForm3115FiledOptimal
        );
    }

    #[test]
    fn citations_pin_section_475f_and_form_3115_and_rev_proc() {
        let out = check(&base());
        assert!(out.citations.iter().any(|c| c.contains("§ 475(f)")));
        assert!(out.citations.iter().any(|c| c.contains("Form 3115")));
        assert!(out.citations.iter().any(|c| c.contains("Rev. Proc. 99-17")));
        assert!(out.citations.iter().any(|c| c.contains("Rev. Proc. 2025-23")));
    }

    #[test]
    fn citations_pin_wash_sale_and_capital_loss_cap() {
        let out = check(&base());
        assert!(out.citations.iter().any(|c| c.contains("§ 1091")));
        assert!(out.citations.iter().any(|c| c.contains("§ 1211(b)")));
    }

    #[test]
    fn citations_pin_endicott_case_law() {
        let out = check(&base());
        assert!(out.citations.iter().any(|c| c.contains("Endicott")));
    }

    #[test]
    fn constant_pin_720_trades_floor() {
        assert_eq!(TYPICAL_TRADER_SUBSTANTIAL_TRADES_FLOOR_PER_YEAR, 720);
    }

    #[test]
    fn constant_pin_4_hours_per_day() {
        assert_eq!(TYPICAL_TRADER_SUBSTANTIAL_HOURS_PER_TRADING_DAY, 4);
    }

    #[test]
    fn constant_pin_5_year_revocation_lockout() {
        assert_eq!(REVOCATION_LOCKOUT_YEARS_AFTER_PRIOR_ELECTION_REVOKED, 5);
    }

    #[test]
    fn constant_pin_3000_capital_loss_cap_in_cents() {
        assert_eq!(SECTION_1211B_CAPITAL_LOSS_CAP_CENTS_PER_YEAR, 300_000);
    }

    #[test]
    fn very_large_loss_with_election_passes_through_without_overflow() {
        let mut i = base();
        i.net_securities_loss_cents = u64::MAX;
        let out = check(&i);
        assert_eq!(out.deductible_loss_against_ordinary_income_cents, u64::MAX);
        assert!(out.ordinary_loss_treatment_unlimited);
    }
}
