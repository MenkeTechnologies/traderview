//! IRC § 362 basis to corporations on tax-free transfers.
//!
//! § 362 governs the basis a TRANSFEREE CORPORATION takes in property received in
//! tax-free § 351 transfers, § 368 reorganizations, and contributions to capital.
//! Combined with § 358 (shareholder basis side), § 362 implements the double-basis-
//! preservation framework that keeps built-in gain or loss latent through formation
//! and reorganization transactions while preventing taxpayer manipulation via
//! cross-jurisdictional loss importation (§ 362(e)(1)) and intra-United-States loss
//! duplication (§ 362(e)(2)).
//!
//! § 362(a) GENERAL RULE: corporation's basis in property received in § 351 transfer
//! OR contribution to capital = adjusted basis of property in transferor's hands +
//! any gain recognized by transferor on the transfer (under § 351(b)).
//!
//! § 362(b) REORGANIZATIONS: in a § 368 reorganization, corporation's basis in
//! property received = adjusted basis in transferor's hands + gain recognized
//! (under § 356).
//!
//! § 362(c) PROPERTY ACQUIRED FOR PAID-IN SURPLUS / CAPITAL CONTRIBUTION BY
//! SHAREHOLDER: carryover basis from contributor (similar to § 362(a)).
//!
//! § 362(d) CONTRIBUTIONS BY NON-SHAREHOLDERS: zero basis to corporation
//! (governmental incentives, customer-paid hookup charges, etc.); pre-TCJA
//! § 118 contribution-to-capital exclusion changed by TCJA.
//!
//! § 362(e)(1) ANTI-LOSS-IMPORTATION RULE (added by American Jobs Creation Act of
//! 2004, finalized in 2016 regs): if transferred property would be subject to U.S.
//! tax in transferee's hands but was NOT subject in transferor's hands AND the
//! transferor's aggregate adjusted basis exceeds aggregate FMV, transferee's basis
//! in each transferred asset is reduced to its FMV. Targets loss-importation from
//! foreign or tax-exempt parties.
//!
//! § 362(e)(2) ANTI-LOSS-DUPLICATION RULE (added 2004): in a § 351 transfer where
//! transferor's AGGREGATE adjusted basis of transferred property EXCEEDS aggregate
//! FMV (net built-in loss), transferee's aggregate basis is REDUCED to aggregate
//! FMV; aggregate reduction allocated among transferred property in proportion to
//! built-in losses. Transferor and transferee may JOINTLY ELECT under § 362(e)(2)(C)
//! to instead reduce transferor's basis in stock received by the net built-in loss
//! (preserving corporation's carryover basis).
//!
//! Citations (verified per WebSearch 2026-06-02):
//! - law.cornell.edu/uscode/text/26/362
//! - law.cornell.edu/cfr/text/26/1.362-3
//! - law.cornell.edu/cfr/text/26/1.362-4
//! - federalregister.gov/documents/2016/03/28/2016-06227/limitations-on-the-importation-of-net-built-in-losses

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransferType {
    /// § 351 transfer to corporation controlled by transferor.
    Section351TransferSection362A,
    /// § 368 reorganization.
    Section368ReorganizationSection362B,
    /// § 362(c) paid-in surplus / capital contribution by shareholder.
    PaidInSurplusByShareholderSection362C,
    /// § 362(d) contribution by non-shareholder (TCJA-modified treatment).
    ContributionByNonShareholderSection362D,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LossLimitationStatus {
    /// No § 362(e) loss-limitation trigger; basic carryover basis applies.
    NoLossLimitationApplied,
    /// § 362(e)(1) anti-loss-importation triggered (foreign or tax-exempt
    /// transferor + net built-in loss).
    Section362E1AntiLossImportationApplied,
    /// § 362(e)(2) anti-loss-duplication triggered (domestic net built-in loss).
    Section362E2AntiLossDuplicationAppliedToCorporationBasis,
    /// § 362(e)(2)(C) joint election: transferor's stock basis reduced instead.
    Section362E2CJointElectionTransferorStockBasisReduced,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub transfer_type: TransferType,
    pub loss_limitation_status: LossLimitationStatus,
    pub transferor_adjusted_basis_aggregate_cents: u64,
    pub aggregate_fmv_cents: u64,
    pub gain_recognized_by_transferor_section_351b_or_356_cents: u64,
}

pub type Section362CorporationBasisInput = Input;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    Section362ACarryoverBasisStandardSection351,
    Section362BCarryoverBasisReorganization,
    Section362CPaidInSurplusCarryover,
    Section362DContributionByNonShareholderZeroOrTcjaTreatment,
    Section362E1ImportationBasisSteppedDownToFmv,
    Section362E2DuplicationCorpBasisSteppedDownToFmv,
    Section362E2CJointElectionTransferorStockBasisReducedCorpKeepsCarryover,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub severity: Severity,
    pub corporation_basis_in_property_cents: u64,
    pub aggregate_built_in_loss_eliminated_cents: u64,
    pub note: String,
}

pub type Section362CorporationBasisOutput = Output;
pub type Section362CorporationBasisResult = Output;

#[must_use]
pub fn check(input: &Input) -> Output {
    if matches!(
        input.transfer_type,
        TransferType::ContributionByNonShareholderSection362D
    ) {
        return Output {
            severity: Severity::Section362DContributionByNonShareholderZeroOrTcjaTreatment,
            corporation_basis_in_property_cents: 0,
            aggregate_built_in_loss_eliminated_cents: 0,
            note: "§ 362(d) CONTRIBUTION BY NON-SHAREHOLDER: zero basis to corporation. \
                   Tax Cuts and Jobs Act of 2017 (Pub. L. 115-97 § 13312) repealed § 118 \
                   exclusion for contributions by non-shareholders effective for \
                   contributions after Dec 22, 2017 — government incentives, utility \
                   hookup payments, and customer-paid construction allowances are now \
                   GENERALLY INCLUDABLE in gross income to the corporation. Pre-TCJA \
                   contributions retain § 118 exclusion treatment."
                .to_string(),
        };
    }

    if matches!(
        input.loss_limitation_status,
        LossLimitationStatus::Section362E1AntiLossImportationApplied
    ) {
        let net_built_in_loss = input
            .transferor_adjusted_basis_aggregate_cents
            .saturating_sub(input.aggregate_fmv_cents);
        return Output {
            severity: Severity::Section362E1ImportationBasisSteppedDownToFmv,
            corporation_basis_in_property_cents: input.aggregate_fmv_cents,
            aggregate_built_in_loss_eliminated_cents: net_built_in_loss,
            note: format!(
                "§ 362(e)(1) ANTI-LOSS-IMPORTATION applied. Property would be subject to U.S. \
                 tax in transferee's hands but was NOT subject in transferor's hands \
                 (foreign person, tax-exempt entity, or other non-US-tax-subjected \
                 transferor) AND aggregate basis (${}) exceeds aggregate FMV (${}). \
                 Transferee's basis in EACH transferred asset reduced to its FMV. Aggregate \
                 built-in loss ELIMINATED: ${}. Added by American Jobs Creation Act of 2004; \
                 finalized in 26 C.F.R. § 1.362-3 (effective for transactions on or after \
                 March 28, 2016). Coordinates with § 334(b)(1)(B) parallel rule for § 332 \
                 parent-subsidiary liquidations + § 358(h) shareholder-level companion.",
                input.transferor_adjusted_basis_aggregate_cents / 100,
                input.aggregate_fmv_cents / 100,
                net_built_in_loss / 100
            ),
        };
    }

    if matches!(
        input.loss_limitation_status,
        LossLimitationStatus::Section362E2AntiLossDuplicationAppliedToCorporationBasis
    ) {
        let net_built_in_loss = input
            .transferor_adjusted_basis_aggregate_cents
            .saturating_sub(input.aggregate_fmv_cents);
        return Output {
            severity: Severity::Section362E2DuplicationCorpBasisSteppedDownToFmv,
            corporation_basis_in_property_cents: input.aggregate_fmv_cents,
            aggregate_built_in_loss_eliminated_cents: net_built_in_loss,
            note: format!(
                "§ 362(e)(2) ANTI-LOSS-DUPLICATION applied at CORPORATE LEVEL. Transferor's \
                 aggregate adjusted basis (${}) exceeds aggregate FMV (${}) — net built-in \
                 loss ${}. Transferee's aggregate basis REDUCED to aggregate FMV; reduction \
                 allocated among transferred property in proportion to built-in losses per \
                 26 C.F.R. § 1.362-4. Aggregate built-in loss ELIMINATED at corporate level \
                 (preserved at shareholder level in stock received). Without § 362(e)(2), \
                 transferor would have built-in loss in stock AND corporation would have \
                 built-in loss in property — double loss recognition. § 362(e)(2)(C) joint \
                 election available to instead reduce transferor's stock basis (preserving \
                 corp's carryover basis).",
                input.transferor_adjusted_basis_aggregate_cents / 100,
                input.aggregate_fmv_cents / 100,
                net_built_in_loss / 100
            ),
        };
    }

    if matches!(
        input.loss_limitation_status,
        LossLimitationStatus::Section362E2CJointElectionTransferorStockBasisReduced
    ) {
        let net_built_in_loss = input
            .transferor_adjusted_basis_aggregate_cents
            .saturating_sub(input.aggregate_fmv_cents);
        let carryover_basis = input
            .transferor_adjusted_basis_aggregate_cents
            .saturating_add(input.gain_recognized_by_transferor_section_351b_or_356_cents);
        return Output {
            severity:
                Severity::Section362E2CJointElectionTransferorStockBasisReducedCorpKeepsCarryover,
            corporation_basis_in_property_cents: carryover_basis,
            aggregate_built_in_loss_eliminated_cents: net_built_in_loss,
            note: format!(
                "§ 362(e)(2)(C) JOINT ELECTION made: transferor and transferee elected to \
                 reduce TRANSFEROR'S stock basis (in § 358 shareholder calculation) by net \
                 built-in loss (${}) INSTEAD of reducing corporation's basis in transferred \
                 property. Corporation retains FULL § 362(a) carryover basis (${}). \
                 Strategic choice: preserves corp's tax attributes (NOLs, depreciation, \
                 capital-loss carryforward) at cost of transferor's stock-level loss. \
                 Election filed with both parties' returns for the year of transfer per \
                 26 C.F.R. § 1.362-4(d)(3).",
                net_built_in_loss / 100,
                carryover_basis / 100
            ),
        };
    }

    let carryover_basis = input
        .transferor_adjusted_basis_aggregate_cents
        .saturating_add(input.gain_recognized_by_transferor_section_351b_or_356_cents);

    let severity = match input.transfer_type {
        TransferType::Section351TransferSection362A => {
            Severity::Section362ACarryoverBasisStandardSection351
        }
        TransferType::Section368ReorganizationSection362B => {
            Severity::Section362BCarryoverBasisReorganization
        }
        TransferType::PaidInSurplusByShareholderSection362C => {
            Severity::Section362CPaidInSurplusCarryover
        }
        TransferType::ContributionByNonShareholderSection362D => unreachable!(),
    };

    Output {
        severity,
        corporation_basis_in_property_cents: carryover_basis,
        aggregate_built_in_loss_eliminated_cents: 0,
        note: format!(
            "§ 362(a) / (b) / (c) CARRYOVER BASIS rule: corporation's basis in transferred \
             property = transferor's adjusted basis (${}) + gain recognized by transferor \
             (${}) = ${}. {} Combined with § 358 shareholder basis side (iter 560), preserves \
             built-in gain or loss across the formation/reorganization. Coordinates with \
             § 351 control-immediately-after requirement (§ 368(c) 80%), § 357 liability \
             rules, § 1223 holding-period tacking, § 1245 + § 1250 depreciation-recapture \
             attribute carryover.",
            input.transferor_adjusted_basis_aggregate_cents / 100,
            input.gain_recognized_by_transferor_section_351b_or_356_cents / 100,
            carryover_basis / 100,
            match input.transfer_type {
                TransferType::Section351TransferSection362A => "Standard § 351 transfer.",
                TransferType::Section368ReorganizationSection362B => "§ 368 reorganization.",
                TransferType::PaidInSurplusByShareholderSection362C => {
                    "§ 362(c) paid-in surplus / capital contribution."
                }
                _ => "",
            }
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_351() -> Input {
        Input {
            transfer_type: TransferType::Section351TransferSection362A,
            loss_limitation_status: LossLimitationStatus::NoLossLimitationApplied,
            transferor_adjusted_basis_aggregate_cents: 100_000_00,
            aggregate_fmv_cents: 150_000_00,
            gain_recognized_by_transferor_section_351b_or_356_cents: 0,
        }
    }

    #[test]
    fn section_351_pure_carryover_basis() {
        let input = base_351();
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section362ACarryoverBasisStandardSection351
        );
        assert_eq!(output.corporation_basis_in_property_cents, 100_000_00);
        assert!(output.note.contains("§ 362(a)"));
        assert!(output.note.contains("§ 1223"));
    }

    #[test]
    fn section_351_with_gain_recognized_adds_to_carryover() {
        let mut input = base_351();
        input.gain_recognized_by_transferor_section_351b_or_356_cents = 20_000_00;
        let output = check(&input);
        // $100K + $20K = $120K
        assert_eq!(output.corporation_basis_in_property_cents, 120_000_00);
    }

    #[test]
    fn section_368_reorganization_carryover_basis() {
        let mut input = base_351();
        input.transfer_type = TransferType::Section368ReorganizationSection362B;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section362BCarryoverBasisReorganization
        );
    }

    #[test]
    fn section_362c_paid_in_surplus_carryover() {
        let mut input = base_351();
        input.transfer_type = TransferType::PaidInSurplusByShareholderSection362C;
        let output = check(&input);
        assert_eq!(output.severity, Severity::Section362CPaidInSurplusCarryover);
    }

    #[test]
    fn section_362d_non_shareholder_contribution_zero_basis_tcja_treatment() {
        let mut input = base_351();
        input.transfer_type = TransferType::ContributionByNonShareholderSection362D;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section362DContributionByNonShareholderZeroOrTcjaTreatment
        );
        assert_eq!(output.corporation_basis_in_property_cents, 0);
        assert!(output.note.contains("Pub. L. 115-97 § 13312"));
        assert!(output.note.contains("§ 118"));
    }

    #[test]
    fn section_362e1_anti_loss_importation_reduces_to_fmv() {
        let mut input = base_351();
        input.transferor_adjusted_basis_aggregate_cents = 500_000_00;
        input.aggregate_fmv_cents = 200_000_00;
        input.loss_limitation_status = LossLimitationStatus::Section362E1AntiLossImportationApplied;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section362E1ImportationBasisSteppedDownToFmv
        );
        // Basis reduced to FMV $200K
        assert_eq!(output.corporation_basis_in_property_cents, 200_000_00);
        // Built-in loss eliminated: $500K - $200K = $300K
        assert_eq!(output.aggregate_built_in_loss_eliminated_cents, 300_000_00);
        assert!(output.note.contains("§ 362(e)(1)"));
        assert!(output.note.contains("§ 1.362-3"));
        assert!(output.note.contains("March 28, 2016"));
    }

    #[test]
    fn section_362e2_anti_loss_duplication_reduces_corp_basis() {
        let mut input = base_351();
        input.transferor_adjusted_basis_aggregate_cents = 500_000_00;
        input.aggregate_fmv_cents = 200_000_00;
        input.loss_limitation_status =
            LossLimitationStatus::Section362E2AntiLossDuplicationAppliedToCorporationBasis;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section362E2DuplicationCorpBasisSteppedDownToFmv
        );
        assert_eq!(output.corporation_basis_in_property_cents, 200_000_00);
        assert!(output.note.contains("§ 362(e)(2)"));
        assert!(output.note.contains("§ 1.362-4"));
        assert!(output.note.contains("§ 362(e)(2)(C)"));
    }

    #[test]
    fn section_362e2c_joint_election_preserves_corp_carryover() {
        let mut input = base_351();
        input.transferor_adjusted_basis_aggregate_cents = 500_000_00;
        input.aggregate_fmv_cents = 200_000_00;
        input.loss_limitation_status =
            LossLimitationStatus::Section362E2CJointElectionTransferorStockBasisReduced;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section362E2CJointElectionTransferorStockBasisReducedCorpKeepsCarryover
        );
        // Corp keeps $500K full carryover
        assert_eq!(output.corporation_basis_in_property_cents, 500_000_00);
        assert!(output.note.contains("§ 362(e)(2)(C)"));
        assert!(output.note.contains("§ 1.362-4(d)(3)"));
    }

    #[test]
    fn very_large_basis_no_overflow_in_carryover_calc() {
        let mut input = base_351();
        input.transferor_adjusted_basis_aggregate_cents = u64::MAX / 2;
        input.gain_recognized_by_transferor_section_351b_or_356_cents = u64::MAX / 2;
        let output = check(&input);
        // saturating_add prevents overflow
        assert!(output.corporation_basis_in_property_cents > 0);
    }

    #[test]
    fn zero_basis_zero_carryover() {
        let mut input = base_351();
        input.transferor_adjusted_basis_aggregate_cents = 0;
        let output = check(&input);
        assert_eq!(output.corporation_basis_in_property_cents, 0);
    }

    #[test]
    fn note_pins_section_358_shareholder_basis_companion() {
        let input = base_351();
        let output = check(&input);
        assert!(output.note.contains("§ 358"));
    }

    #[test]
    fn note_pins_section_357_liability_rules() {
        let input = base_351();
        let output = check(&input);
        assert!(output.note.contains("§ 357"));
    }

    #[test]
    fn note_pins_section_1245_1250_recapture() {
        let input = base_351();
        let output = check(&input);
        assert!(output.note.contains("§ 1245"));
        assert!(output.note.contains("§ 1250"));
    }

    #[test]
    fn note_pins_section_334_b_parallel_for_332_liquidations() {
        let mut input = base_351();
        input.transferor_adjusted_basis_aggregate_cents = 500_000_00;
        input.aggregate_fmv_cents = 200_000_00;
        input.loss_limitation_status = LossLimitationStatus::Section362E1AntiLossImportationApplied;
        let output = check(&input);
        assert!(output.note.contains("§ 334(b)(1)(B)"));
    }

    #[test]
    fn note_pins_section_358h_shareholder_companion_for_e1() {
        let mut input = base_351();
        input.transferor_adjusted_basis_aggregate_cents = 500_000_00;
        input.aggregate_fmv_cents = 200_000_00;
        input.loss_limitation_status = LossLimitationStatus::Section362E1AntiLossImportationApplied;
        let output = check(&input);
        assert!(output.note.contains("§ 358(h)"));
    }

    #[test]
    fn note_pins_section_368c_80_pct_control_requirement() {
        let input = base_351();
        let output = check(&input);
        assert!(output.note.contains("§ 368(c)"));
    }

    #[test]
    fn no_loss_no_reduction_when_basis_below_fmv() {
        let mut input = base_351();
        input.transferor_adjusted_basis_aggregate_cents = 100_000_00;
        input.aggregate_fmv_cents = 150_000_00;
        input.loss_limitation_status = LossLimitationStatus::NoLossLimitationApplied;
        let output = check(&input);
        // No net built-in loss → no reduction
        assert_eq!(output.aggregate_built_in_loss_eliminated_cents, 0);
        assert_eq!(output.corporation_basis_in_property_cents, 100_000_00);
    }
}
