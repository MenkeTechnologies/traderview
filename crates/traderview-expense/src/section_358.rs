//! IRC § 358 basis to distributees (shareholder basis in stock/securities received).
//!
//! § 358 governs the basis a shareholder takes in stock and securities received in
//! tax-free corporate formations under § 351 and tax-free reorganizations under
//! § 368. § 358 preserves built-in gain or loss in the shareholder's hands by carrying
//! over the basis of property transferred to the corporation, adjusted for boot
//! received, gain recognized, and liabilities assumed. Combined with § 362
//! (corporation's basis side), § 358 implements the "double basis preservation" that
//! keeps gain or loss latent across formation/reorganization transactions.
//!
//! § 358(a)(1) GENERAL RULE — shareholder's basis in stock and securities received:
//!   = adjusted basis of property transferred
//!   + gain recognized on the transfer (under § 351(b) or § 356)
//!   - money received (boot)
//!   - FMV of other property received (boot)
//!   - liabilities assumed (treated as boot under § 358(d))
//!
//! § 358(a)(2) BASIS ALLOCATION among multiple classes of property received:
//! basis allocated to each class in proportion to FMV at time of receipt.
//!
//! § 358(b)(1) MULTIPLE STOCK CLASSES: basis allocated proportionally to FMV of each
//! class.
//!
//! § 358(c) REORGANIZATIONS: basis preserved tracing surrendered shares to received
//! shares (Federal Register 2006 tracing-method final regulations).
//!
//! § 358(d) LIABILITY-AS-BOOT: liabilities assumed by transferee corporation reduce
//! shareholder's basis (treated as money received). § 357(c)(1) recognizes gain if
//! liabilities exceed adjusted basis of transferred property.
//!
//! § 358(h) ANTI-LOSS-IMPORTATION RULE (added 2000, finalized in 2014 regs):
//! shareholder's basis in stock received is REDUCED (but not below FMV) by amount
//! of liability assumed by transferee corporation if the assumption is part of a
//! tax-avoidance scheme designed to import a built-in loss into the United States.
//!
//! Citations (verified per WebSearch 2026-06-02):
//! - law.cornell.edu/uscode/text/26/358
//! - taxnotes.com/research/federal/usc26/358
//! - federalregister.gov/documents/2006/01/26/06-585/determination-of-basis-of-stock-or-securities-received-in-exchange-for-or-with-respect-to-stock-or-securities-in-certain-transactions
//! - irs.gov/pub/irs-drop/rr-03-51.pdf

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionType {
    /// § 351 transfer to corporation controlled by transferor.
    Section351TransferToControlledCorp,
    /// § 368(a)(1)(A) statutory merger or consolidation.
    Section368A1AStatutoryMerger,
    /// § 368(a)(1)(B) stock-for-stock B reorganization.
    Section368A1BStockForStock,
    /// § 368(a)(1)(C) stock-for-substantially-all-assets C reorganization.
    Section368A1CStockForAssets,
    /// § 368(a)(1)(D) divisive D reorganization.
    Section368A1DDivisiveReorganization,
    /// § 368(a)(1)(E) recapitalization.
    Section368A1ERecapitalization,
    /// § 368(a)(1)(F) mere change in form / identity / place of organization.
    Section368A1FMereChange,
    /// Not a tax-deferred transaction.
    NotTaxDeferredTransactionInapplicable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AntiLossImportationStatus {
    /// § 358(h) anti-loss-importation rule does NOT apply (no tax-avoidance scheme,
    /// no built-in loss being imported, or domestic-only transferee).
    AntiLossImportationNotApplicable,
    /// § 358(h) anti-loss-importation triggers: basis reduced to FMV.
    AntiLossImportationAppliesBasisReducedToFmv,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub transaction_type: TransactionType,
    pub anti_loss_importation_status: AntiLossImportationStatus,
    pub adjusted_basis_of_property_transferred_cents: u64,
    pub gain_recognized_section_351b_or_356_cents: u64,
    pub boot_received_money_cents: u64,
    pub boot_received_other_property_fmv_cents: u64,
    pub liabilities_assumed_by_transferee_cents: u64,
    pub fmv_of_stock_received_cents: u64,
}

pub type Section358ShareholderBasisInput = Input;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    Section358InapplicableNotTaxDeferred,
    Section358ABasisPreservedCarryoverWithAdjustments,
    Section358HAntiLossImportationBasisReducedToFmv,
    Section358ABasisReducedBelowZeroSection357C1GainRecognition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub severity: Severity,
    pub shareholder_basis_in_stock_received_cents: i128,
    pub section_357c1_gain_recognized_cents: u64,
    pub note: String,
}

pub type Section358ShareholderBasisOutput = Output;
pub type Section358ShareholderBasisResult = Output;

#[must_use]
pub fn check(input: &Input) -> Output {
    if matches!(
        input.transaction_type,
        TransactionType::NotTaxDeferredTransactionInapplicable
    ) {
        return Output {
            severity: Severity::Section358InapplicableNotTaxDeferred,
            shareholder_basis_in_stock_received_cents: 0,
            section_357c1_gain_recognized_cents: 0,
            note: "§ 358 inapplicable: transaction is not a tax-deferred § 351 transfer or \
                   § 368 reorganization. Basis in stock acquired in taxable purchase = cost \
                   under § 1012 (cost-basis general rule). Capital gain/loss recognized on \
                   the underlying property sale under § 1001."
                .to_string(),
        };
    }

    // § 358(a)(1) basis computation
    let basis_i128 = i128::from(input.adjusted_basis_of_property_transferred_cents)
        + i128::from(input.gain_recognized_section_351b_or_356_cents)
        - i128::from(input.boot_received_money_cents)
        - i128::from(input.boot_received_other_property_fmv_cents)
        - i128::from(input.liabilities_assumed_by_transferee_cents);

    if basis_i128 < 0 {
        let recognized = u64::try_from(-basis_i128).unwrap_or(u64::MAX);
        return Output {
            severity: Severity::Section358ABasisReducedBelowZeroSection357C1GainRecognition,
            shareholder_basis_in_stock_received_cents: 0,
            section_357c1_gain_recognized_cents: recognized,
            note: format!(
                "§ 357(c)(1) GAIN RECOGNITION TRIGGER: § 358(a) basis computation would \
                 produce NEGATIVE basis (${} below zero). § 357(c)(1) requires recognition \
                 of gain equal to the excess of liabilities assumed over the adjusted basis \
                 of property transferred. Gain ${} recognized; shareholder's basis floored \
                 at $0. Common in highly-leveraged transfers — partnership basis-shifting \
                 strategies + LLC-to-corp conversions with mortgaged real estate. Coordinates \
                 with § 357(a) general non-recognition rule (liability assumption not boot for \
                 gain-recognition purposes) and § 357(b) tax-avoidance exception (full \
                 liability-as-boot treatment).",
                recognized / 100,
                recognized / 100
            ),
        };
    }

    if matches!(
        input.anti_loss_importation_status,
        AntiLossImportationStatus::AntiLossImportationAppliesBasisReducedToFmv
    ) {
        let fmv = i128::from(input.fmv_of_stock_received_cents);
        let reduced_basis = basis_i128.min(fmv);
        return Output {
            severity: Severity::Section358HAntiLossImportationBasisReducedToFmv,
            shareholder_basis_in_stock_received_cents: reduced_basis,
            section_357c1_gain_recognized_cents: 0,
            note: format!(
                "§ 358(h) ANTI-LOSS-IMPORTATION applies. Computed basis (${}) REDUCED to FMV \
                 (${}) because the liability assumption is part of a tax-avoidance scheme to \
                 import a built-in loss into the United States. Added by Pub. L. 106-554 in \
                 2000 to close outbound-loss-export and inbound-loss-import loopholes; \
                 finalized in 2014 Treasury regulations. Coordinates with § 362(e) corporate-\
                 level anti-loss-importation rule + § 367(d) outbound transfer of \
                 intangibles + § 904 FTC limitation.",
                basis_i128 / 100,
                fmv / 100
            ),
        };
    }

    Output {
        severity: Severity::Section358ABasisPreservedCarryoverWithAdjustments,
        shareholder_basis_in_stock_received_cents: basis_i128,
        section_357c1_gain_recognized_cents: 0,
        note: format!(
            "§ 358(a)(1) shareholder basis = adjusted basis transferred (${}) + gain \
             recognized (${}) - money boot (${}) - other-property boot (${}) - liabilities \
             assumed (${}) = ${}. Coordinates with § 362(a) corporation's carryover basis \
             (corp's basis in transferred property = transferor's basis + any gain \
             recognized), § 357 liability rules, § 351 control-immediately-after \
             requirement (80% under § 368(c)). Federal Register 2006 final regs adopt \
             tracing method for reorganizations: basis traced from surrendered shares to \
             received shares.",
            input.adjusted_basis_of_property_transferred_cents / 100,
            input.gain_recognized_section_351b_or_356_cents / 100,
            input.boot_received_money_cents / 100,
            input.boot_received_other_property_fmv_cents / 100,
            input.liabilities_assumed_by_transferee_cents / 100,
            basis_i128 / 100
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_351() -> Input {
        Input {
            transaction_type: TransactionType::Section351TransferToControlledCorp,
            anti_loss_importation_status:
                AntiLossImportationStatus::AntiLossImportationNotApplicable,
            adjusted_basis_of_property_transferred_cents: 100_000_00,
            gain_recognized_section_351b_or_356_cents: 0,
            boot_received_money_cents: 0,
            boot_received_other_property_fmv_cents: 0,
            liabilities_assumed_by_transferee_cents: 0,
            fmv_of_stock_received_cents: 150_000_00,
        }
    }

    #[test]
    fn not_tax_deferred_section_358_inapplicable() {
        let mut input = base_351();
        input.transaction_type = TransactionType::NotTaxDeferredTransactionInapplicable;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section358InapplicableNotTaxDeferred
        );
        assert!(output.note.contains("§ 1012"));
    }

    #[test]
    fn section_351_pure_basis_carryover() {
        let input = base_351();
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section358ABasisPreservedCarryoverWithAdjustments
        );
        // Basis = $100K + 0 - 0 - 0 - 0 = $100K
        assert_eq!(output.shareholder_basis_in_stock_received_cents, 100_000_00);
        assert!(output.note.contains("§ 358(a)(1)"));
        assert!(output.note.contains("§ 362(a)"));
    }

    #[test]
    fn section_351_with_gain_recognized_increases_basis() {
        let mut input = base_351();
        input.gain_recognized_section_351b_or_356_cents = 20_000_00;
        let output = check(&input);
        // Basis = $100K + $20K = $120K
        assert_eq!(output.shareholder_basis_in_stock_received_cents, 120_000_00);
    }

    #[test]
    fn section_351_with_money_boot_reduces_basis() {
        let mut input = base_351();
        input.boot_received_money_cents = 30_000_00;
        let output = check(&input);
        // Basis = $100K - $30K = $70K
        assert_eq!(output.shareholder_basis_in_stock_received_cents, 70_000_00);
    }

    #[test]
    fn section_351_with_other_property_boot_reduces_basis() {
        let mut input = base_351();
        input.boot_received_other_property_fmv_cents = 25_000_00;
        let output = check(&input);
        assert_eq!(output.shareholder_basis_in_stock_received_cents, 75_000_00);
    }

    #[test]
    fn section_351_with_liabilities_reduces_basis() {
        let mut input = base_351();
        input.liabilities_assumed_by_transferee_cents = 40_000_00;
        let output = check(&input);
        // Basis = $100K - $40K = $60K
        assert_eq!(output.shareholder_basis_in_stock_received_cents, 60_000_00);
    }

    #[test]
    fn section_357c1_gain_recognized_when_liabilities_exceed_basis() {
        let mut input = base_351();
        input.adjusted_basis_of_property_transferred_cents = 50_000_00;
        input.liabilities_assumed_by_transferee_cents = 120_000_00;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section358ABasisReducedBelowZeroSection357C1GainRecognition
        );
        // Excess = $120K - $50K = $70K gain
        assert_eq!(output.section_357c1_gain_recognized_cents, 70_000_00);
        assert_eq!(output.shareholder_basis_in_stock_received_cents, 0);
        assert!(output.note.contains("§ 357(c)(1)"));
        assert!(output.note.contains("§ 357(a)"));
        assert!(output.note.contains("§ 357(b)"));
    }

    #[test]
    fn anti_loss_importation_reduces_basis_to_fmv() {
        let mut input = base_351();
        input.adjusted_basis_of_property_transferred_cents = 500_000_00;
        input.fmv_of_stock_received_cents = 200_000_00;
        input.anti_loss_importation_status =
            AntiLossImportationStatus::AntiLossImportationAppliesBasisReducedToFmv;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section358HAntiLossImportationBasisReducedToFmv
        );
        // Computed $500K reduced to FMV $200K
        assert_eq!(output.shareholder_basis_in_stock_received_cents, 200_000_00);
        assert!(output.note.contains("§ 358(h)"));
        assert!(output.note.contains("Pub. L. 106-554"));
        assert!(output.note.contains("§ 362(e)"));
    }

    #[test]
    fn section_368_a1_a_statutory_merger() {
        let mut input = base_351();
        input.transaction_type = TransactionType::Section368A1AStatutoryMerger;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section358ABasisPreservedCarryoverWithAdjustments
        );
    }

    #[test]
    fn section_368_a1_b_stock_for_stock() {
        let mut input = base_351();
        input.transaction_type = TransactionType::Section368A1BStockForStock;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section358ABasisPreservedCarryoverWithAdjustments
        );
    }

    #[test]
    fn section_368_a1_c_stock_for_assets() {
        let mut input = base_351();
        input.transaction_type = TransactionType::Section368A1CStockForAssets;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section358ABasisPreservedCarryoverWithAdjustments
        );
    }

    #[test]
    fn section_368_a1_d_divisive_reorganization() {
        let mut input = base_351();
        input.transaction_type = TransactionType::Section368A1DDivisiveReorganization;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section358ABasisPreservedCarryoverWithAdjustments
        );
    }

    #[test]
    fn section_368_a1_e_recapitalization() {
        let mut input = base_351();
        input.transaction_type = TransactionType::Section368A1ERecapitalization;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section358ABasisPreservedCarryoverWithAdjustments
        );
    }

    #[test]
    fn section_368_a1_f_mere_change() {
        let mut input = base_351();
        input.transaction_type = TransactionType::Section368A1FMereChange;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section358ABasisPreservedCarryoverWithAdjustments
        );
    }

    #[test]
    fn combined_adjustments_basis_computation() {
        let mut input = base_351();
        input.gain_recognized_section_351b_or_356_cents = 15_000_00;
        input.boot_received_money_cents = 10_000_00;
        input.boot_received_other_property_fmv_cents = 5_000_00;
        input.liabilities_assumed_by_transferee_cents = 30_000_00;
        let output = check(&input);
        // $100K + $15K - $10K - $5K - $30K = $70K
        assert_eq!(output.shareholder_basis_in_stock_received_cents, 70_000_00);
    }

    #[test]
    fn very_large_basis_no_overflow() {
        let mut input = base_351();
        input.adjusted_basis_of_property_transferred_cents = u64::MAX;
        let output = check(&input);
        // i128 intermediate prevents overflow
        assert!(output.shareholder_basis_in_stock_received_cents > 0);
    }

    #[test]
    fn zero_basis_zero_basis_preserved() {
        let mut input = base_351();
        input.adjusted_basis_of_property_transferred_cents = 0;
        let output = check(&input);
        assert_eq!(output.shareholder_basis_in_stock_received_cents, 0);
    }

    #[test]
    fn note_pins_section_362a_corporation_basis_companion() {
        let input = base_351();
        let output = check(&input);
        assert!(output.note.contains("§ 362(a)"));
    }

    #[test]
    fn note_pins_section_368c_80_pct_control() {
        let input = base_351();
        let output = check(&input);
        assert!(output.note.contains("§ 368(c)"));
        assert!(output.note.contains("80%"));
    }

    #[test]
    fn note_pins_section_357_liability_rules() {
        let input = base_351();
        let output = check(&input);
        assert!(output.note.contains("§ 357"));
    }
}
