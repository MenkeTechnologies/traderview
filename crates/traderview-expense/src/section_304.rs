//! IRC § 304 redemption through use of related corporations.
//!
//! § 304 prevents brother-sister and parent-subsidiary stock sales between
//! commonly-controlled corporations from being recharacterized as capital-gain sales
//! when the substance of the transaction is a dividend distribution from accumulated
//! earnings and profits (E&P). The provision targets the classic "extraction" tactic
//! where a shareholder controls two corporations and "sells" stock of one (the issuing
//! corporation) to the other (the acquiring corporation) in exchange for cash or other
//! property, attempting to convert what is economically a dividend into a § 1001
//! exchange producing capital gain.
//!
//! § 304(a)(1) BROTHER-SISTER ACQUISITIONS: if one or more persons is in CONTROL of
//! each of two corporations AND one of those persons sells stock of the ISSUING
//! corporation to the ACQUIRING corporation in exchange for property, the transaction
//! is RECHARACTERIZED as a § 301 distribution by the acquiring corporation in
//! redemption of its own stock. The acquiring corporation is deemed to have issued
//! stock back to the shareholder in exchange for the issuing-corp stock.
//!
//! § 304(a)(2) PARENT-SUBSIDIARY ACQUISITIONS: if the issuing corporation is the
//! PARENT (in control) of the acquiring SUBSIDIARY, and a shareholder of the parent
//! transfers parent stock to the subsidiary in exchange for property, the transaction
//! is recharacterized as a § 301 distribution from the subsidiary.
//!
//! § 304(b)(1) "CONTROL" DEFINITION: at least 50% of total combined voting power of
//! all classes of stock entitled to vote, OR at least 50% of total value of shares.
//!
//! § 304(b)(2) E&P STACKING ORDER: the distribution is treated as first paid by the
//! ACQUIRING corporation to the extent of its E&P, then paid by the ISSUING
//! corporation to the extent of its E&P. § 301(c) treatment then applies to determine
//! dividend / basis-recovery / capital-gain split. Distribution exceeding combined E&P
//! is treated as basis recovery first, then capital gain.
//!
//! § 304(b)(3) FOREIGN-CORPORATION RULES: § 304 special rules apply when acquiring or
//! issuing corporation is foreign, including § 1248 deemed-dividend rules.
//!
//! § 304(b)(4) ANTI-AVOIDANCE RULES (Notice 2006-85 + Notice 2007-9 + 2012 final
//! regs): expanded reach to prevent indirect circumvention via partnership
//! interpositions and other multi-step transactions.
//!
//! § 318 CONSTRUCTIVE OWNERSHIP applies to determine "control" under § 304(b)(1).
//! Family attribution + entity attribution + option attribution all expand the reach
//! of § 304's recharacterization rule.
//!
//! Citations (verified per WebSearch 2026-06-02):
//! - taxnotes.com/research/federal/usc26/304
//! - thetaxadviser.com/issues/2010/apr/sec304anti-avoidancerulemodified/
//! - journalofaccountancy.com/news/2012/dec/20127063/
//! - journalofaccountancy.com/issues/2001/dec/relatedcorporateredemptions/

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionType {
    /// § 304(a)(1) brother-sister stock sale.
    BrotherSisterSection304A1,
    /// § 304(a)(2) parent-subsidiary acquisition (subsidiary buys parent stock).
    ParentSubsidiarySection304A2,
    /// Unrelated-party arm's-length sale — § 304 inapplicable; § 1001 capital-gain
    /// treatment applies.
    UnrelatedArmsLengthSaleSection304Inapplicable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ControlStatus {
    /// Shareholder is in § 304(b)(1) control of BOTH corporations (50% vote or value
    /// in each, including § 318 constructive ownership).
    InControlOfBothCorporationsSection304B1,
    /// Shareholder controls one but not both — § 304 inapplicable.
    NotInControlOfBothCorporations,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub transaction_type: TransactionType,
    pub control_status: ControlStatus,
    /// Property transferred from acquiring corp to shareholder (cash + other).
    pub property_received_from_acquiring_corp_cents: u64,
    /// Acquiring corp's E&P available for distribution treatment.
    pub acquiring_corp_eep_cents: u64,
    /// Issuing corp's E&P available for distribution treatment.
    pub issuing_corp_eep_cents: u64,
    /// Shareholder's adjusted basis in stock sold.
    pub shareholder_basis_in_sold_stock_cents: u64,
}

pub type Section304RelatedCorpRedemptionInput = Input;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    Section304InapplicableUnrelatedArmsLengthSale,
    Section304InapplicableNotInControlOfBothCorporations,
    Section304ARecharacterizationDividendUpToTotalEep,
    Section304BasisRecoveryAfterEepExhausted,
    Section304CapitalGainAfterBasisRecovered,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub severity: Severity,
    pub dividend_treatment_cents: u64,
    pub basis_recovery_cents: u64,
    pub capital_gain_cents: u64,
    pub remaining_basis_in_shareholder_stock_cents: u64,
    pub note: String,
}

pub type Section304RelatedCorpRedemptionOutput = Output;
pub type Section304RelatedCorpRedemptionResult = Output;

const SECTION_304_CONTROL_THRESHOLD_PERCENT: u32 = 50;

#[must_use]
pub fn check(input: &Input) -> Output {
    if matches!(
        input.transaction_type,
        TransactionType::UnrelatedArmsLengthSaleSection304Inapplicable
    ) {
        let exchange_gain = input
            .property_received_from_acquiring_corp_cents
            .saturating_sub(input.shareholder_basis_in_sold_stock_cents);
        return Output {
            severity: Severity::Section304InapplicableUnrelatedArmsLengthSale,
            dividend_treatment_cents: 0,
            basis_recovery_cents: input
                .shareholder_basis_in_sold_stock_cents
                .min(input.property_received_from_acquiring_corp_cents),
            capital_gain_cents: exchange_gain,
            remaining_basis_in_shareholder_stock_cents: 0,
            note: format!(
                "§ 304 inapplicable: unrelated-party arm's-length sale. § 1001 capital-gain \
                 treatment applies. Property received (${}) - shareholder basis in sold stock \
                 (${}) = ${} capital gain (long-term or short-term depending on holding period \
                 under § 1222).",
                input.property_received_from_acquiring_corp_cents / 100,
                input.shareholder_basis_in_sold_stock_cents / 100,
                exchange_gain / 100
            ),
        };
    }

    if matches!(
        input.control_status,
        ControlStatus::NotInControlOfBothCorporations
    ) {
        let exchange_gain = input
            .property_received_from_acquiring_corp_cents
            .saturating_sub(input.shareholder_basis_in_sold_stock_cents);
        return Output {
            severity: Severity::Section304InapplicableNotInControlOfBothCorporations,
            dividend_treatment_cents: 0,
            basis_recovery_cents: input
                .shareholder_basis_in_sold_stock_cents
                .min(input.property_received_from_acquiring_corp_cents),
            capital_gain_cents: exchange_gain,
            remaining_basis_in_shareholder_stock_cents: 0,
            note: format!(
                "§ 304 inapplicable: shareholder does NOT control BOTH corporations under \
                 § 304(b)(1) {SECTION_304_CONTROL_THRESHOLD_PERCENT}% vote-or-value standard \
                 (§ 318 constructive ownership applies to determine control). § 1001 \
                 capital-gain treatment applies. Property received (${}) - shareholder basis \
                 (${}) = ${} capital gain.",
                input.property_received_from_acquiring_corp_cents / 100,
                input.shareholder_basis_in_sold_stock_cents / 100,
                exchange_gain / 100
            ),
        };
    }

    // § 304 recharacterization: distribution treated as paid first by acquiring corp
    // to extent of its E&P, then by issuing corp to extent of its E&P.
    let combined_eep = input
        .acquiring_corp_eep_cents
        .saturating_add(input.issuing_corp_eep_cents);
    let dividend_portion = input
        .property_received_from_acquiring_corp_cents
        .min(combined_eep);
    let after_dividend = input
        .property_received_from_acquiring_corp_cents
        .saturating_sub(dividend_portion);
    let basis_recovery = after_dividend.min(input.shareholder_basis_in_sold_stock_cents);
    let capital_gain = after_dividend.saturating_sub(basis_recovery);
    let remaining_basis = input
        .shareholder_basis_in_sold_stock_cents
        .saturating_sub(basis_recovery);

    let severity = if capital_gain > 0 {
        Severity::Section304CapitalGainAfterBasisRecovered
    } else if basis_recovery > 0 {
        Severity::Section304BasisRecoveryAfterEepExhausted
    } else {
        Severity::Section304ARecharacterizationDividendUpToTotalEep
    };

    let transaction_label = match input.transaction_type {
        TransactionType::BrotherSisterSection304A1 => "§ 304(a)(1) brother-sister",
        TransactionType::ParentSubsidiarySection304A2 => "§ 304(a)(2) parent-subsidiary",
        TransactionType::UnrelatedArmsLengthSaleSection304Inapplicable => unreachable!(),
    };

    Output {
        severity,
        dividend_treatment_cents: dividend_portion,
        basis_recovery_cents: basis_recovery,
        capital_gain_cents: capital_gain,
        remaining_basis_in_shareholder_stock_cents: remaining_basis,
        note: format!(
            "{} stock-sale RECHARACTERIZED as § 301 distribution by acquiring corp. § 304(b)(2) \
             E&P STACKING ORDER: distribution treated as first paid by acquiring corp to extent \
             of its E&P (${}), then by issuing corp to extent of its E&P (${}). Combined E&P \
             ${}. Property received ${} → dividend portion ${} (taxable as ordinary income \
             subject to qualified-dividend rates under § 1(h)(11) if § 1(h)(11)(B) requirements \
             met); basis recovery ${}; capital gain ${} (long-term or short-term per § 1222). \
             Remaining basis in shareholder's stock of acquiring corp: ${}. Coordinates with \
             § 318 constructive ownership (drives control determination), § 1248 foreign-corp \
             deemed-dividend rules, § 245A (potentially available for foreign-source portion).",
            transaction_label,
            input.acquiring_corp_eep_cents / 100,
            input.issuing_corp_eep_cents / 100,
            combined_eep / 100,
            input.property_received_from_acquiring_corp_cents / 100,
            dividend_portion / 100,
            basis_recovery / 100,
            capital_gain / 100,
            remaining_basis / 100
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_brother_sister() -> Input {
        Input {
            transaction_type: TransactionType::BrotherSisterSection304A1,
            control_status: ControlStatus::InControlOfBothCorporationsSection304B1,
            property_received_from_acquiring_corp_cents: 1_000_000_00,
            acquiring_corp_eep_cents: 600_000_00,
            issuing_corp_eep_cents: 200_000_00,
            shareholder_basis_in_sold_stock_cents: 150_000_00,
        }
    }

    #[test]
    fn unrelated_arms_length_sale_section_304_inapplicable() {
        let mut input = base_brother_sister();
        input.transaction_type =
            TransactionType::UnrelatedArmsLengthSaleSection304Inapplicable;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section304InapplicableUnrelatedArmsLengthSale
        );
        assert!(output.note.contains("§ 1001"));
        assert!(output.note.contains("§ 1222"));
        // $1M property - $150K basis = $850K capital gain
        assert_eq!(output.capital_gain_cents, 850_000_00);
    }

    #[test]
    fn not_in_control_of_both_section_304_inapplicable() {
        let mut input = base_brother_sister();
        input.control_status = ControlStatus::NotInControlOfBothCorporations;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section304InapplicableNotInControlOfBothCorporations
        );
        assert!(output.note.contains("§ 304(b)(1)"));
        assert!(output.note.contains("§ 318"));
    }

    #[test]
    fn brother_sister_full_dividend_within_combined_eep() {
        let mut input = base_brother_sister();
        // Combined E&P = $600K + $200K = $800K
        // Property $1M > combined E&P $800K → $800K dividend + $200K post-E&P
        // Post-E&P $200K - basis $150K = $50K capital gain after $150K basis recovery
        let output = check(&input);
        assert_eq!(output.dividend_treatment_cents, 800_000_00);
        assert_eq!(output.basis_recovery_cents, 150_000_00);
        assert_eq!(output.capital_gain_cents, 50_000_00);
        assert_eq!(output.remaining_basis_in_shareholder_stock_cents, 0);
        assert_eq!(
            output.severity,
            Severity::Section304CapitalGainAfterBasisRecovered
        );
        assert!(output.note.contains("§ 304(a)(1)"));
        assert!(output.note.contains("§ 304(b)(2)"));
    }

    #[test]
    fn brother_sister_property_within_combined_eep_all_dividend() {
        let mut input = base_brother_sister();
        input.property_received_from_acquiring_corp_cents = 500_000_00;
        let output = check(&input);
        // $500K < $800K combined E&P → all dividend
        assert_eq!(output.dividend_treatment_cents, 500_000_00);
        assert_eq!(output.basis_recovery_cents, 0);
        assert_eq!(output.capital_gain_cents, 0);
        assert_eq!(
            output.severity,
            Severity::Section304ARecharacterizationDividendUpToTotalEep
        );
    }

    #[test]
    fn brother_sister_property_just_exceeds_eep_basis_recovery() {
        let mut input = base_brother_sister();
        input.property_received_from_acquiring_corp_cents = 850_000_00;
        // $800K E&P + $50K basis recovery → $0 capital gain
        let output = check(&input);
        assert_eq!(output.dividend_treatment_cents, 800_000_00);
        assert_eq!(output.basis_recovery_cents, 50_000_00);
        assert_eq!(output.capital_gain_cents, 0);
        assert_eq!(
            output.severity,
            Severity::Section304BasisRecoveryAfterEepExhausted
        );
    }

    #[test]
    fn parent_subsidiary_section_304a2_recharacterization() {
        let mut input = base_brother_sister();
        input.transaction_type = TransactionType::ParentSubsidiarySection304A2;
        let output = check(&input);
        assert!(output.note.contains("§ 304(a)(2)"));
        assert!(output.note.contains("parent-subsidiary"));
    }

    #[test]
    fn zero_eep_full_basis_recovery_then_capital_gain() {
        let mut input = base_brother_sister();
        input.acquiring_corp_eep_cents = 0;
        input.issuing_corp_eep_cents = 0;
        let output = check(&input);
        // $0 E&P → $150K basis recovery + $850K capital gain
        assert_eq!(output.dividend_treatment_cents, 0);
        assert_eq!(output.basis_recovery_cents, 150_000_00);
        assert_eq!(output.capital_gain_cents, 850_000_00);
    }

    #[test]
    fn section_304_control_threshold_constant_pins_50_pct() {
        assert_eq!(SECTION_304_CONTROL_THRESHOLD_PERCENT, 50);
    }

    #[test]
    fn very_large_property_no_overflow() {
        let mut input = base_brother_sister();
        input.property_received_from_acquiring_corp_cents = u64::MAX;
        let output = check(&input);
        assert!(output.dividend_treatment_cents > 0);
    }

    #[test]
    fn zero_property_zero_distribution() {
        let mut input = base_brother_sister();
        input.property_received_from_acquiring_corp_cents = 0;
        let output = check(&input);
        assert_eq!(output.dividend_treatment_cents, 0);
        assert_eq!(output.basis_recovery_cents, 0);
        assert_eq!(output.capital_gain_cents, 0);
    }

    #[test]
    fn note_pins_section_245a_foreign_source() {
        let input = base_brother_sister();
        let output = check(&input);
        assert!(output.note.contains("§ 245A"));
    }

    #[test]
    fn note_pins_section_1248_foreign_corp_deemed_dividend() {
        let input = base_brother_sister();
        let output = check(&input);
        assert!(output.note.contains("§ 1248"));
    }

    #[test]
    fn note_pins_qualified_dividend_section_1_h_11() {
        let input = base_brother_sister();
        let output = check(&input);
        assert!(output.note.contains("§ 1(h)(11)"));
    }

    #[test]
    fn note_pins_section_318_constructive_ownership_companion() {
        let mut input = base_brother_sister();
        input.control_status = ControlStatus::NotInControlOfBothCorporations;
        let output = check(&input);
        assert!(output.note.contains("§ 318"));
    }

    #[test]
    fn unrelated_arms_length_zero_gain_when_basis_exceeds_proceeds() {
        let mut input = base_brother_sister();
        input.transaction_type =
            TransactionType::UnrelatedArmsLengthSaleSection304Inapplicable;
        input.property_received_from_acquiring_corp_cents = 100_000_00;
        input.shareholder_basis_in_sold_stock_cents = 150_000_00;
        let output = check(&input);
        // $100K - $150K = saturating to 0
        assert_eq!(output.capital_gain_cents, 0);
    }

    #[test]
    fn remaining_basis_in_acquiring_corp_stock_tracked() {
        let mut input = base_brother_sister();
        input.property_received_from_acquiring_corp_cents = 800_000_00;
        input.shareholder_basis_in_sold_stock_cents = 200_000_00;
        let output = check(&input);
        // $800K dividend (within E&P), no basis recovery
        // Remaining basis = $200K
        assert_eq!(output.remaining_basis_in_shareholder_stock_cents, 200_000_00);
    }

    #[test]
    fn note_pins_section_304b2_eep_stacking_order() {
        let input = base_brother_sister();
        let output = check(&input);
        assert!(output.note.contains("§ 304(b)(2) E&P STACKING ORDER"));
    }
}
