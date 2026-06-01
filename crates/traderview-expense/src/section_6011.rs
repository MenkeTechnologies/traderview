//! IRC § 6011 — Reportable transaction disclosure (Form 8886).
//!
//! Trader-critical for anyone claiming losses ≥ statutory threshold
//! or using transactions in any of the five reportable-transaction
//! categories under Treas. Reg. § 1.6011-4(b). Failure to disclose
//! triggers § 6707A penalties of up to $100,000 for individuals or
//! $200,000 for entities (listed-transaction maximum).
//!
//! Five reportable-transaction categories under Treas. Reg.
//! § 1.6011-4(b):
//!
//!   § 1.6011-4(b)(2) — LISTED TRANSACTIONS: Specifically
//!     identified by IRS as abusive tax shelters (e.g., Notice
//!     2003-81 micro-captive insurance; Notice 2017-10 syndicated
//!     conservation easements; Notice 2024-46 charitable
//!     remainder annuity trust step-up).
//!
//!   § 1.6011-4(b)(3) — CONFIDENTIAL TRANSACTIONS: Offered to
//!     taxpayer under conditions of confidentiality protecting
//!     advisor's tax strategies; minimum fee thresholds: $250,000
//!     for corporate taxpayer; $50,000 for noncorporate taxpayer.
//!
//!   § 1.6011-4(b)(4) — TRANSACTIONS WITH CONTRACTUAL PROTECTION:
//!     Taxpayer has right to full or partial refund of fees if
//!     intended tax consequences fail.
//!
//!   § 1.6011-4(b)(5) — LOSS TRANSACTIONS: § 165 loss claims
//!     meeting thresholds:
//!       - Individuals / trusts: $2M single year OR $4M multi-year
//!         (5-year aggregate)
//!       - Corporations: $10M single / $20M multi-year
//!       - Partnerships + S corporations: $10M single / $20M
//!         multi-year (entity level)
//!       - Special $50K threshold for § 988 currency loss trades by
//!         individuals
//!
//!   § 1.6011-4(b)(6) — TRANSACTIONS OF INTEREST (TOI):
//!     Designated by IRS for further study (currently includes
//!     monetized installment sales, foreign retirement plans,
//!     certain partnership transactions).
//!
//! § 6707A — PENALTIES FOR FAILURE TO DISCLOSE:
//!   - Reportable transaction: 75% of tax reduction; minimum
//!     $5,000 individual / $10,000 entity.
//!   - Listed-transaction maximum: $100,000 individual / $200,000
//!     entity.
//!   - Other-reportable maximum: $50,000 individual / $200,000
//!     entity.
//!
//! Citations: 26 U.S.C. § 6011 (general disclosure requirement);
//! Treas. Reg. § 1.6011-4(b)(2) (listed); § 1.6011-4(b)(3)
//! (confidential); § 1.6011-4(b)(4) (contractual protection);
//! § 1.6011-4(b)(5) (loss transactions + thresholds); § 1.6011-4(b)(6)
//! (transactions of interest); 26 U.S.C. § 6707A (failure-to-disclose
//! penalty); 26 CFR § 301.6707A-1 (§ 6707A penalty regulations);
//! § 6011(e) (electronic filing); Form 8886 + Form 8918 (material
//! advisor disclosure under § 6111); § 6112 (material advisor list
//! maintenance).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TransactionCategory {
    /// Treas. Reg. § 1.6011-4(b)(2) — IRS-listed transaction.
    ListedTransaction,
    /// § 1.6011-4(b)(3) — confidential transaction (fee threshold).
    ConfidentialTransaction,
    /// § 1.6011-4(b)(4) — transaction with contractual protection.
    ContractualProtection,
    /// § 1.6011-4(b)(5) — loss transaction (§ 165 loss threshold).
    LossTransaction,
    /// § 1.6011-4(b)(6) — transaction of interest (TOI).
    TransactionOfInterest,
    /// Not a reportable transaction.
    NotReportable,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaxpayerStatus {
    Individual,
    Trust,
    Corporation,
    Partnership,
    SCorporation,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section6011Input {
    pub transaction_category: TransactionCategory,
    pub taxpayer_status: TaxpayerStatus,
    /// Fee paid to advisor (cents). Used for confidential-
    /// transaction threshold under § 1.6011-4(b)(3).
    pub fee_paid_to_advisor_cents: i64,
    /// § 165 loss claimed in single year (cents). Used for
    /// loss-transaction threshold under § 1.6011-4(b)(5).
    pub single_year_loss_claimed_cents: i64,
    /// § 165 loss aggregated over 5-year period (cents).
    pub multi_year_loss_total_cents: i64,
    /// Tax reduction resulting from the transaction (cents).
    /// Used for § 6707A 75% penalty base.
    pub tax_reduction_from_transaction_cents: i64,
    /// Whether Form 8886 was filed disclosing the transaction.
    pub form_8886_filed: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section6011Result {
    /// True if Form 8886 disclosure is required for this
    /// transaction category and taxpayer combination.
    pub disclosure_required: bool,
    /// True if the confidential-transaction fee threshold is met.
    pub confidential_threshold_met: bool,
    /// True if the loss-transaction threshold is met.
    pub loss_threshold_met: bool,
    /// § 6707A penalty exposure (cents) if disclosure required and
    /// not filed.
    pub section_6707a_penalty_cents: i64,
    /// § 6707A maximum penalty cap for this taxpayer + category
    /// (cents).
    pub section_6707a_max_penalty_cents: i64,
    pub compliant: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

/// § 1.6011-4(b)(3) corporate confidential-transaction fee threshold.
pub const CONFIDENTIAL_CORPORATE_FEE_THRESHOLD_CENTS: i64 = 25_000_000;
/// § 1.6011-4(b)(3) noncorporate confidential-transaction fee threshold.
pub const CONFIDENTIAL_NONCORPORATE_FEE_THRESHOLD_CENTS: i64 = 5_000_000;
/// § 1.6011-4(b)(5) individual + trust single-year loss threshold.
pub const LOSS_INDIVIDUAL_SINGLE_YEAR_THRESHOLD_CENTS: i64 = 200_000_000;
/// § 1.6011-4(b)(5) individual + trust multi-year (5-year) loss threshold.
pub const LOSS_INDIVIDUAL_MULTI_YEAR_THRESHOLD_CENTS: i64 = 400_000_000;
/// § 1.6011-4(b)(5) corporation / partnership / S-corp single-year threshold.
pub const LOSS_ENTITY_SINGLE_YEAR_THRESHOLD_CENTS: i64 = 1_000_000_000;
/// § 1.6011-4(b)(5) entity multi-year threshold.
pub const LOSS_ENTITY_MULTI_YEAR_THRESHOLD_CENTS: i64 = 2_000_000_000;
/// § 6707A minimum penalty — individual reportable transaction.
pub const SECTION_6707A_MIN_INDIVIDUAL_CENTS: i64 = 500_000;
/// § 6707A minimum penalty — entity reportable transaction.
pub const SECTION_6707A_MIN_ENTITY_CENTS: i64 = 1_000_000;
/// § 6707A maximum penalty — listed transaction individual.
pub const SECTION_6707A_MAX_INDIVIDUAL_LISTED_CENTS: i64 = 10_000_000;
/// § 6707A maximum penalty — listed transaction entity.
pub const SECTION_6707A_MAX_ENTITY_LISTED_CENTS: i64 = 20_000_000;
/// § 6707A maximum penalty — other reportable individual.
pub const SECTION_6707A_MAX_INDIVIDUAL_OTHER_CENTS: i64 = 5_000_000;
/// § 6707A maximum penalty — other reportable entity.
pub const SECTION_6707A_MAX_ENTITY_OTHER_CENTS: i64 = 20_000_000;

pub fn compute(input: &Section6011Input) -> Section6011Result {
    let mut notes: Vec<String> = Vec::new();
    let mut violations: Vec<String> = Vec::new();

    let is_corporate = matches!(
        input.taxpayer_status,
        TaxpayerStatus::Corporation
            | TaxpayerStatus::Partnership
            | TaxpayerStatus::SCorporation
    );

    // § 1.6011-4(b)(3) confidential-transaction fee threshold.
    let confidential_threshold_met = matches!(
        input.transaction_category,
        TransactionCategory::ConfidentialTransaction
    ) && input.fee_paid_to_advisor_cents
        >= if is_corporate {
            CONFIDENTIAL_CORPORATE_FEE_THRESHOLD_CENTS
        } else {
            CONFIDENTIAL_NONCORPORATE_FEE_THRESHOLD_CENTS
        };

    // § 1.6011-4(b)(5) loss-transaction thresholds.
    let loss_threshold_met = matches!(
        input.transaction_category,
        TransactionCategory::LossTransaction
    ) && {
        let single_threshold = if is_corporate {
            LOSS_ENTITY_SINGLE_YEAR_THRESHOLD_CENTS
        } else {
            LOSS_INDIVIDUAL_SINGLE_YEAR_THRESHOLD_CENTS
        };
        let multi_threshold = if is_corporate {
            LOSS_ENTITY_MULTI_YEAR_THRESHOLD_CENTS
        } else {
            LOSS_INDIVIDUAL_MULTI_YEAR_THRESHOLD_CENTS
        };
        input.single_year_loss_claimed_cents >= single_threshold
            || input.multi_year_loss_total_cents >= multi_threshold
    };

    // Disclosure-required determination.
    let disclosure_required = match input.transaction_category {
        TransactionCategory::ListedTransaction => true,
        TransactionCategory::ConfidentialTransaction => confidential_threshold_met,
        TransactionCategory::ContractualProtection => true,
        TransactionCategory::LossTransaction => loss_threshold_met,
        TransactionCategory::TransactionOfInterest => true,
        TransactionCategory::NotReportable => false,
    };

    // § 6707A penalty calculation.
    let is_listed = matches!(
        input.transaction_category,
        TransactionCategory::ListedTransaction
    );
    let raw_75_percent = input
        .tax_reduction_from_transaction_cents
        .max(0)
        .saturating_mul(75)
        / 100;

    let (min_penalty, max_penalty) = match (is_corporate, is_listed) {
        (false, true) => (
            SECTION_6707A_MIN_INDIVIDUAL_CENTS,
            SECTION_6707A_MAX_INDIVIDUAL_LISTED_CENTS,
        ),
        (false, false) => (
            SECTION_6707A_MIN_INDIVIDUAL_CENTS,
            SECTION_6707A_MAX_INDIVIDUAL_OTHER_CENTS,
        ),
        (true, true) => (
            SECTION_6707A_MIN_ENTITY_CENTS,
            SECTION_6707A_MAX_ENTITY_LISTED_CENTS,
        ),
        (true, false) => (
            SECTION_6707A_MIN_ENTITY_CENTS,
            SECTION_6707A_MAX_ENTITY_OTHER_CENTS,
        ),
    };

    let section_6707a_penalty = if disclosure_required && !input.form_8886_filed {
        violations.push(format!(
            "§ 6011 + Treas. Reg. § 1.6011-4 — Form 8886 required for {:?} but not filed.",
            input.transaction_category,
        ));
        raw_75_percent.max(min_penalty).min(max_penalty)
    } else {
        0
    };

    // Category-specific notes.
    match input.transaction_category {
        TransactionCategory::ListedTransaction => {
            notes.push(
                "§ 1.6011-4(b)(2) — listed transaction: specifically identified by IRS as \
                 abusive tax shelter (e.g., Notice 2003-81 micro-captive insurance; Notice \
                 2017-10 syndicated conservation easements). Disclosure REQUIRED regardless \
                 of any other thresholds."
                    .to_string(),
            );
        }
        TransactionCategory::ConfidentialTransaction => {
            notes.push(format!(
                "§ 1.6011-4(b)(3) — confidential transaction: requires advisor confidentiality \
                 plus minimum fee threshold ({} cents corporate; {} cents noncorporate). Fee \
                 {} cents — threshold {}.",
                CONFIDENTIAL_CORPORATE_FEE_THRESHOLD_CENTS,
                CONFIDENTIAL_NONCORPORATE_FEE_THRESHOLD_CENTS,
                input.fee_paid_to_advisor_cents,
                if confidential_threshold_met { "met" } else { "not met" },
            ));
        }
        TransactionCategory::ContractualProtection => {
            notes.push(
                "§ 1.6011-4(b)(4) — transaction with contractual protection: taxpayer has \
                 right to full or partial refund of advisor fees if intended tax consequences \
                 fail. Disclosure REQUIRED."
                    .to_string(),
            );
        }
        TransactionCategory::LossTransaction => {
            notes.push(format!(
                "§ 1.6011-4(b)(5) — loss transaction: individual + trust ($2M single / $4M \
                 multi-year); entity ($10M single / $20M multi-year). Single-year loss {} \
                 cents; multi-year {} cents — threshold {}.",
                input.single_year_loss_claimed_cents,
                input.multi_year_loss_total_cents,
                if loss_threshold_met { "met" } else { "not met" },
            ));
        }
        TransactionCategory::TransactionOfInterest => {
            notes.push(
                "§ 1.6011-4(b)(6) — transaction of interest (TOI): designated by IRS for \
                 further study. Disclosure REQUIRED."
                    .to_string(),
            );
        }
        TransactionCategory::NotReportable => {
            notes.push(
                "Transaction does not fall within § 1.6011-4(b)(2)–(6) reportable-transaction \
                 categories; Form 8886 disclosure not required."
                    .to_string(),
            );
        }
    }

    if disclosure_required && !input.form_8886_filed {
        notes.push(format!(
            "§ 6707A penalty exposure — 75% of tax reduction ({} cents raw) capped at {} cents \
             max ({} listed transaction; {} other) and floored at {} cents min.",
            raw_75_percent,
            max_penalty,
            if is_listed { "listed" } else { "non-listed" },
            if is_corporate { "entity" } else { "individual" },
            min_penalty,
        ));
    }

    notes.push(
        "Companion to material advisor disclosure under § 6111 (Form 8918) and material \
         advisor list maintenance under § 6112. Failure to disclose triggers § 6707A penalty + \
         potential § 6662A 20%/30% reportable-transaction-understatement penalty on the \
         underlying tax."
            .to_string(),
    );

    Section6011Result {
        disclosure_required,
        confidential_threshold_met,
        loss_threshold_met,
        section_6707a_penalty_cents: section_6707a_penalty,
        section_6707a_max_penalty_cents: max_penalty,
        compliant: violations.is_empty(),
        violations,
        citation: "26 U.S.C. § 6011 (general disclosure requirement); Treas. Reg. \
                   § 1.6011-4(b)(2) (listed); § 1.6011-4(b)(3) (confidential — fee threshold); \
                   § 1.6011-4(b)(4) (contractual protection); § 1.6011-4(b)(5) (loss \
                   transactions — $2M individual / $10M entity); § 1.6011-4(b)(6) \
                   (transactions of interest); 26 U.S.C. § 6707A (failure-to-disclose \
                   penalty); 26 CFR § 301.6707A-1; § 6111 (Form 8918 material advisor); \
                   § 6112 (advisor list); § 6662A (reportable-transaction-understatement \
                   accuracy penalty)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        category: TransactionCategory,
        status: TaxpayerStatus,
        fee: i64,
        single_loss: i64,
        multi_loss: i64,
        tax_reduction: i64,
        filed: bool,
    ) -> Section6011Input {
        Section6011Input {
            transaction_category: category,
            taxpayer_status: status,
            fee_paid_to_advisor_cents: fee,
            single_year_loss_claimed_cents: single_loss,
            multi_year_loss_total_cents: multi_loss,
            tax_reduction_from_transaction_cents: tax_reduction,
            form_8886_filed: filed,
        }
    }

    // ── Listed transactions ────────────────────────────────────

    #[test]
    fn listed_transaction_disclosure_required() {
        let r = compute(&input(
            TransactionCategory::ListedTransaction,
            TaxpayerStatus::Individual,
            0,
            0,
            0,
            5_000_000,
            true,
        ));
        assert!(r.disclosure_required);
        assert!(r.compliant);
    }

    #[test]
    fn listed_transaction_not_filed_max_individual_100k_penalty() {
        // Tax reduction 50,000,000 cents = $500k. 75% = 37,500,000.
        // Capped at $100K individual listed max = 10,000,000 cents.
        let r = compute(&input(
            TransactionCategory::ListedTransaction,
            TaxpayerStatus::Individual,
            0,
            0,
            0,
            50_000_000,
            false,
        ));
        assert!(!r.compliant);
        assert_eq!(r.section_6707a_penalty_cents, 10_000_000);
        assert_eq!(r.section_6707a_max_penalty_cents, 10_000_000);
    }

    #[test]
    fn listed_transaction_not_filed_max_entity_200k_penalty() {
        let r = compute(&input(
            TransactionCategory::ListedTransaction,
            TaxpayerStatus::Corporation,
            0,
            0,
            0,
            50_000_000,
            false,
        ));
        assert!(!r.compliant);
        assert_eq!(r.section_6707a_penalty_cents, 20_000_000);
        assert_eq!(r.section_6707a_max_penalty_cents, 20_000_000);
    }

    #[test]
    fn listed_transaction_low_tax_reduction_min_5k_individual_penalty() {
        // Tax reduction $0 → 75% = $0 → floored at $5K min.
        let r = compute(&input(
            TransactionCategory::ListedTransaction,
            TaxpayerStatus::Individual,
            0,
            0,
            0,
            0,
            false,
        ));
        assert_eq!(r.section_6707a_penalty_cents, 500_000);
    }

    #[test]
    fn listed_transaction_low_tax_reduction_min_10k_entity_penalty() {
        let r = compute(&input(
            TransactionCategory::ListedTransaction,
            TaxpayerStatus::Corporation,
            0,
            0,
            0,
            0,
            false,
        ));
        assert_eq!(r.section_6707a_penalty_cents, 1_000_000);
    }

    // ── Confidential transactions ──────────────────────────────

    #[test]
    fn confidential_corporate_below_250k_threshold_no_disclosure() {
        let r = compute(&input(
            TransactionCategory::ConfidentialTransaction,
            TaxpayerStatus::Corporation,
            20_000_000, // $200k — below corporate $250k
            0,
            0,
            5_000_000,
            false,
        ));
        assert!(!r.confidential_threshold_met);
        assert!(!r.disclosure_required);
    }

    #[test]
    fn confidential_corporate_at_250k_threshold_disclosure_required() {
        let r = compute(&input(
            TransactionCategory::ConfidentialTransaction,
            TaxpayerStatus::Corporation,
            25_000_000,
            0,
            0,
            5_000_000,
            true,
        ));
        assert!(r.confidential_threshold_met);
        assert!(r.disclosure_required);
        assert!(r.compliant);
    }

    #[test]
    fn confidential_individual_at_50k_threshold_disclosure_required() {
        let r = compute(&input(
            TransactionCategory::ConfidentialTransaction,
            TaxpayerStatus::Individual,
            5_000_000,
            0,
            0,
            5_000_000,
            true,
        ));
        assert!(r.confidential_threshold_met);
        assert!(r.disclosure_required);
    }

    #[test]
    fn confidential_individual_below_50k_no_disclosure() {
        let r = compute(&input(
            TransactionCategory::ConfidentialTransaction,
            TaxpayerStatus::Individual,
            4_999_999,
            0,
            0,
            0,
            false,
        ));
        assert!(!r.confidential_threshold_met);
        assert!(!r.disclosure_required);
        assert!(r.compliant);
    }

    // ── Contractual protection ─────────────────────────────────

    #[test]
    fn contractual_protection_disclosure_required() {
        let r = compute(&input(
            TransactionCategory::ContractualProtection,
            TaxpayerStatus::Individual,
            0,
            0,
            0,
            5_000_000,
            true,
        ));
        assert!(r.disclosure_required);
    }

    // ── Loss transactions ──────────────────────────────────────

    #[test]
    fn loss_individual_2m_single_year_at_threshold_disclosure_required() {
        let r = compute(&input(
            TransactionCategory::LossTransaction,
            TaxpayerStatus::Individual,
            0,
            200_000_000, // $2M
            0,
            5_000_000,
            true,
        ));
        assert!(r.loss_threshold_met);
        assert!(r.disclosure_required);
    }

    #[test]
    fn loss_individual_1_99m_single_year_below_threshold_no_disclosure() {
        let r = compute(&input(
            TransactionCategory::LossTransaction,
            TaxpayerStatus::Individual,
            0,
            199_000_000, // $1.99M
            0,
            5_000_000,
            false,
        ));
        assert!(!r.loss_threshold_met);
        assert!(!r.disclosure_required);
    }

    #[test]
    fn loss_individual_4m_multi_year_disclosure_required() {
        let r = compute(&input(
            TransactionCategory::LossTransaction,
            TaxpayerStatus::Individual,
            0,
            0,
            400_000_000, // $4M multi-year
            5_000_000,
            true,
        ));
        assert!(r.loss_threshold_met);
    }

    #[test]
    fn loss_corporation_10m_single_year_threshold() {
        let r = compute(&input(
            TransactionCategory::LossTransaction,
            TaxpayerStatus::Corporation,
            0,
            1_000_000_000, // $10M
            0,
            5_000_000,
            true,
        ));
        assert!(r.loss_threshold_met);
    }

    #[test]
    fn loss_corporation_5m_below_threshold() {
        let r = compute(&input(
            TransactionCategory::LossTransaction,
            TaxpayerStatus::Corporation,
            0,
            500_000_000, // $5M
            0,
            5_000_000,
            false,
        ));
        assert!(!r.loss_threshold_met);
        assert!(!r.disclosure_required);
    }

    // ── Transactions of interest ───────────────────────────────

    #[test]
    fn transactions_of_interest_disclosure_required() {
        let r = compute(&input(
            TransactionCategory::TransactionOfInterest,
            TaxpayerStatus::Individual,
            0,
            0,
            0,
            5_000_000,
            true,
        ));
        assert!(r.disclosure_required);
    }

    // ── Not reportable ─────────────────────────────────────────

    #[test]
    fn not_reportable_no_disclosure_no_penalty() {
        let r = compute(&input(
            TransactionCategory::NotReportable,
            TaxpayerStatus::Individual,
            10_000_000,
            500_000_000,
            500_000_000,
            10_000_000,
            false,
        ));
        assert!(!r.disclosure_required);
        assert_eq!(r.section_6707a_penalty_cents, 0);
    }

    // ── § 6707A penalty calculation ────────────────────────────

    #[test]
    fn nonlisted_individual_max_50k_cap() {
        // Tax reduction $100K → 75% = $75K. Capped at $50K
        // non-listed individual max.
        let r = compute(&input(
            TransactionCategory::ContractualProtection,
            TaxpayerStatus::Individual,
            0,
            0,
            0,
            10_000_000,
            false,
        ));
        assert_eq!(r.section_6707a_penalty_cents, 5_000_000);
        assert_eq!(r.section_6707a_max_penalty_cents, 5_000_000);
    }

    #[test]
    fn penalty_minimum_floors_below_75_percent() {
        // Tax reduction $1000 → 75% = $750. Floored at $5K individual min.
        let r = compute(&input(
            TransactionCategory::ContractualProtection,
            TaxpayerStatus::Individual,
            0,
            0,
            0,
            100_000,
            false,
        ));
        assert_eq!(r.section_6707a_penalty_cents, 500_000);
    }

    #[test]
    fn penalty_75_percent_calculation_between_min_and_max() {
        // Tax reduction $20K → 75% = $15K. Between $5K min and $50K
        // max for non-listed individual.
        let r = compute(&input(
            TransactionCategory::ContractualProtection,
            TaxpayerStatus::Individual,
            0,
            0,
            0,
            2_000_000,
            false,
        ));
        assert_eq!(r.section_6707a_penalty_cents, 1_500_000);
    }

    // ── Multi-regime invariants ────────────────────────────────

    #[test]
    fn corporate_thresholds_higher_than_individual_invariant() {
        // 5 cells comparing corporate vs individual thresholds for
        // confidential + loss.
        assert!(CONFIDENTIAL_CORPORATE_FEE_THRESHOLD_CENTS > CONFIDENTIAL_NONCORPORATE_FEE_THRESHOLD_CENTS);
        assert!(LOSS_ENTITY_SINGLE_YEAR_THRESHOLD_CENTS > LOSS_INDIVIDUAL_SINGLE_YEAR_THRESHOLD_CENTS);
        assert!(LOSS_ENTITY_MULTI_YEAR_THRESHOLD_CENTS > LOSS_INDIVIDUAL_MULTI_YEAR_THRESHOLD_CENTS);
        assert!(SECTION_6707A_MIN_ENTITY_CENTS > SECTION_6707A_MIN_INDIVIDUAL_CENTS);
        assert!(SECTION_6707A_MAX_ENTITY_LISTED_CENTS > SECTION_6707A_MAX_INDIVIDUAL_LISTED_CENTS);
    }

    #[test]
    fn listed_max_higher_than_other_max_invariant() {
        // Listed transaction max penalty > other reportable max.
        assert!(SECTION_6707A_MAX_INDIVIDUAL_LISTED_CENTS > SECTION_6707A_MAX_INDIVIDUAL_OTHER_CENTS);
        // Entity listed and other-reportable max are equal at $200K
        // — both higher than individual.
        assert_eq!(SECTION_6707A_MAX_ENTITY_LISTED_CENTS, SECTION_6707A_MAX_ENTITY_OTHER_CENTS);
    }

    #[test]
    fn always_required_categories_invariant() {
        // Listed + Contractual + TOI always require disclosure;
        // Confidential + Loss + NotReportable conditional.
        for category in [
            TransactionCategory::ListedTransaction,
            TransactionCategory::ContractualProtection,
            TransactionCategory::TransactionOfInterest,
        ] {
            let r = compute(&input(
                category,
                TaxpayerStatus::Individual,
                0,
                0,
                0,
                0,
                true,
            ));
            assert!(r.disclosure_required, "{:?}: always required", category);
        }

        let no_disclose = compute(&input(
            TransactionCategory::NotReportable,
            TaxpayerStatus::Individual,
            0,
            0,
            0,
            0,
            false,
        ));
        assert!(!no_disclose.disclosure_required);
    }

    #[test]
    fn citation_pins_all_subsections() {
        let r = compute(&input(
            TransactionCategory::ListedTransaction,
            TaxpayerStatus::Individual,
            0,
            0,
            0,
            5_000_000,
            true,
        ));
        assert!(r.citation.contains("§ 6011"));
        assert!(r.citation.contains("§ 1.6011-4(b)(2)"));
        assert!(r.citation.contains("§ 1.6011-4(b)(3)"));
        assert!(r.citation.contains("§ 1.6011-4(b)(4)"));
        assert!(r.citation.contains("§ 1.6011-4(b)(5)"));
        assert!(r.citation.contains("§ 1.6011-4(b)(6)"));
        assert!(r.citation.contains("§ 6707A"));
        assert!(r.citation.contains("§ 6111"));
        assert!(r.citation.contains("§ 6112"));
        assert!(r.citation.contains("§ 6662A"));
    }

    #[test]
    fn material_advisor_sibling_note_present() {
        let r = compute(&input(
            TransactionCategory::ListedTransaction,
            TaxpayerStatus::Individual,
            0,
            0,
            0,
            5_000_000,
            true,
        ));
        assert!(
            r.notes.iter().any(|n| n.contains("§ 6111")
                && n.contains("Form 8918")
                && n.contains("§ 6112")
                && n.contains("§ 6662A")),
            "sibling-module note must be present"
        );
    }

    #[test]
    fn defensive_negative_tax_reduction_clamps_at_zero() {
        let r = compute(&input(
            TransactionCategory::ContractualProtection,
            TaxpayerStatus::Individual,
            0,
            0,
            0,
            -1_000_000,
            false,
        ));
        // Negative clamps; floored at $5K minimum penalty.
        assert_eq!(r.section_6707a_penalty_cents, 500_000);
    }
}
