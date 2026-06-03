//! IRC § 311 taxability of corporation on distribution of property in kind.
//!
//! § 311 governs the recognition of gain or loss by the DISTRIBUTING corporation when
//! it distributes property in kind to shareholders. Pre-1986, § 311(a) provided a
//! general non-recognition rule. The Tax Reform Act of 1986 repealed the General
//! Utilities doctrine and added § 311(b), which requires the distributing corporation
//! to RECOGNIZE GAIN on the distribution of appreciated property as if it had been
//! sold at fair market value. The provision closes the asset-removal-without-tax
//! loophole that previously allowed a corporation to extract appreciated property
//! without entity-level tax. § 311(a) continues to provide non-recognition of LOSS on
//! distributions of depreciated property — distributions are not the corporation's
//! occasion to claim losses.
//!
//! § 311(a) GENERAL RULE: no gain or loss to the distributing corporation on
//! distribution of property with respect to its stock (subject to § 311(b)).
//!
//! § 311(b)(1) APPRECIATED PROPERTY: if the FMV of the distributed property exceeds
//! the distributing corporation's adjusted basis, gain is RECOGNIZED to the
//! distributing corporation as if the property had been SOLD at FMV.
//!
//! § 311(b)(2) LIABILITY ASSUMPTION (cross-reference to § 336(b)): if a shareholder
//! assumes a liability of the distributing corporation OR if the distributed property
//! is subject to a liability, the FMV is deemed to be NOT LESS than the amount of the
//! liability.
//!
//! § 311(b)(3) PARTNERSHIP / TRUST INTEREST anti-loss-recognition rule: if the
//! distributed property is an interest in a partnership or trust, gain is computed
//! without regard to any loss attributable to property contributed to the partnership
//! or trust with a principal purpose of recognizing loss on the distribution.
//!
//! § 311 does NOT apply to: (a) § 332 parent-subsidiary liquidations, (b) § 336
//! corporate liquidation distributions (which have their own gain/loss recognition
//! regime), (c) § 355 corporate-division distributions (subject to separate rules).
//!
//! Citations (verified per WebSearch 2026-06-02):
//! - law.cornell.edu/uscode/text/26/311
//! - codes.findlaw.com/us/title-26-internal-revenue-code/26-usc-sect-311/
//! - uscode.house.gov/view.xhtml?req=granuleid:USC-prelim-title26-section311&num=0&edition=prelim
//! - bradfordtaxinstitute.com/Endnotes/IRC_Section_311b.pdf

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DistributionType {
    /// Ordinary § 301 distribution (dividend, return of capital) — § 311 applies.
    OrdinarySection301Distribution,
    /// § 332 parent-subsidiary liquidation — § 311 inapplicable; § 332/337 govern.
    Section332ParentSubLiquidationSection311Inapplicable,
    /// § 336 general liquidation distribution — § 311 inapplicable; § 336 governs.
    Section336GeneralLiquidationSection311Inapplicable,
    /// § 355 corporate-division spinoff/split-off — § 311 inapplicable.
    Section355CorporateDivisionSection311Inapplicable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppreciationStatus {
    /// FMV exceeds adjusted basis — § 311(b)(1) gain recognition triggered.
    FmvExceedsAdjustedBasisAppreciated,
    /// FMV equals adjusted basis — neutral, no gain or loss recognition.
    FmvEqualsAdjustedBasisNeutral,
    /// FMV below adjusted basis — depreciated; § 311(a) preserves no-loss rule.
    FmvBelowAdjustedBasisDepreciated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PartnershipTrustAntiLossStatus {
    /// Distributed property is NOT a partnership / trust interest.
    NotPartnershipOrTrustInterest,
    /// Distributed property is a partnership / trust interest with property
    /// contributed without anti-loss principal purpose.
    PartnershipOrTrustInterestBonaFideContribution,
    /// Distributed property is a partnership / trust interest where property was
    /// contributed for principal purpose of recognizing loss on the distribution.
    PartnershipOrTrustInterestPrincipalPurposeOfLossRecognition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    Section311InapplicableOtherDistributionRegime,
    Section311ANoGainOrLossNeutralDistribution,
    Section311ANoLossOnDepreciatedPropertyDistribution,
    Section311BGainRecognitionAppreciatedProperty,
    Section311BWithLiabilityAssumptionFmvFloorAdjustment,
    Section311B3PartnershipTrustAntiLossDisallowance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub distribution_type: DistributionType,
    pub appreciation_status: AppreciationStatus,
    pub partnership_trust_anti_loss_status: PartnershipTrustAntiLossStatus,
    pub fair_market_value_cents: u64,
    pub adjusted_basis_cents: u64,
    pub liability_assumed_or_property_subject_to_cents: u64,
}

pub type Section311CorporateDistributionGainRecognitionInput = Input;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub severity: Severity,
    pub deemed_fmv_after_liability_floor_cents: u64,
    pub recognized_gain_cents: u64,
    pub recognized_loss_cents: u64,
    pub note: String,
}

pub type Section311CorporateDistributionGainRecognitionOutput = Output;
pub type Section311CorporateDistributionGainRecognitionResult = Output;

#[must_use]
pub fn check(input: &Input) -> Output {
    if !matches!(
        input.distribution_type,
        DistributionType::OrdinarySection301Distribution
    ) {
        return Output {
            severity: Severity::Section311InapplicableOtherDistributionRegime,
            deemed_fmv_after_liability_floor_cents: input.fair_market_value_cents,
            recognized_gain_cents: 0,
            recognized_loss_cents: 0,
            note: format!(
                "§ 311 inapplicable: {} has its own gain-recognition regime. {} Coordinate \
                 with the applicable distribution-specific rules + § 1245 / § 1250 \
                 depreciation-recapture + § 1374 S-corp built-in-gain tax + § 367(e) \
                 outbound-distribution recognition.",
                distribution_label(input.distribution_type),
                distribution_governing_provision(input.distribution_type)
            ),
        };
    }

    // § 311(b)(2) liability assumption: FMV deemed not less than liability assumed.
    let deemed_fmv = input
        .fair_market_value_cents
        .max(input.liability_assumed_or_property_subject_to_cents);
    let liability_floor_applied = deemed_fmv != input.fair_market_value_cents;

    if matches!(
        input.partnership_trust_anti_loss_status,
        PartnershipTrustAntiLossStatus::PartnershipOrTrustInterestPrincipalPurposeOfLossRecognition
    ) {
        let unadjusted_gain = deemed_fmv.saturating_sub(input.adjusted_basis_cents);
        return Output {
            severity: Severity::Section311B3PartnershipTrustAntiLossDisallowance,
            deemed_fmv_after_liability_floor_cents: deemed_fmv,
            recognized_gain_cents: unadjusted_gain,
            recognized_loss_cents: 0,
            note: format!(
                "§ 311(b)(3) ANTI-LOSS RULE: distributed property is a partnership or trust \
                 interest where contributed property was placed in the partnership/trust \
                 with a principal purpose of recognizing loss on the distribution. Gain on \
                 distribution computed WITHOUT regard to any loss attributable to that \
                 contributed property — losses scrubbed out per Treas. Reg. § 1.311-2. \
                 Deemed FMV ${} - adjusted basis ${} = ${} recognized gain (without \
                 anti-purpose loss offset).",
                deemed_fmv / 100,
                input.adjusted_basis_cents / 100,
                unadjusted_gain / 100
            ),
        };
    }

    match input.appreciation_status {
        AppreciationStatus::FmvExceedsAdjustedBasisAppreciated => {
            let gain = deemed_fmv.saturating_sub(input.adjusted_basis_cents);
            let severity = if liability_floor_applied {
                Severity::Section311BWithLiabilityAssumptionFmvFloorAdjustment
            } else {
                Severity::Section311BGainRecognitionAppreciatedProperty
            };
            Output {
                severity,
                deemed_fmv_after_liability_floor_cents: deemed_fmv,
                recognized_gain_cents: gain,
                recognized_loss_cents: 0,
                note: format!(
                    "§ 311(b)(1) GAIN RECOGNITION on appreciated property. Distributing \
                     corporation must recognize gain as if the property had been SOLD to the \
                     distributee at FMV. Deemed FMV ${} - adjusted basis ${} = recognized \
                     gain ${}. {} The Tax Reform Act of 1986 repealed the General Utilities \
                     doctrine and added § 311(b) to close the asset-removal-without-tax \
                     loophole. Coordinates with § 312 E&P computation (gain increases E&P), \
                     § 301(b)/(c)/(d) distributee treatment, § 1245 / § 1250 depreciation \
                     recapture if applicable, § 1374 S-corp built-in-gain tax (10-year window).",
                    deemed_fmv / 100,
                    input.adjusted_basis_cents / 100,
                    gain / 100,
                    if liability_floor_applied {
                        format!(
                            "§ 311(b)(2) LIABILITY FLOOR: FMV deemed at least the amount of \
                             liability assumed (${} > original FMV ${}).",
                            input.liability_assumed_or_property_subject_to_cents / 100,
                            input.fair_market_value_cents / 100
                        )
                    } else {
                        String::new()
                    }
                ),
            }
        }
        AppreciationStatus::FmvEqualsAdjustedBasisNeutral => Output {
            severity: Severity::Section311ANoGainOrLossNeutralDistribution,
            deemed_fmv_after_liability_floor_cents: deemed_fmv,
            recognized_gain_cents: 0,
            recognized_loss_cents: 0,
            note: format!(
                "§ 311(a) general rule: no gain or loss recognized to distributing corporation \
                 because FMV ${} equals adjusted basis ${}. Distribution-neutral at the \
                 corporate level. Distributee receives property with basis = FMV under \
                 § 301(d).",
                deemed_fmv / 100,
                input.adjusted_basis_cents / 100
            ),
        },
        AppreciationStatus::FmvBelowAdjustedBasisDepreciated => Output {
            severity: Severity::Section311ANoLossOnDepreciatedPropertyDistribution,
            deemed_fmv_after_liability_floor_cents: deemed_fmv,
            recognized_gain_cents: 0,
            recognized_loss_cents: 0,
            note: format!(
                "§ 311(a) NO LOSS RECOGNITION on distribution of depreciated property. \
                 Distributing corporation does NOT recognize a loss when FMV ${} is below \
                 adjusted basis ${}. Distribution is not the corporation's occasion to \
                 recognize losses — losses are preserved only through § 336 liquidation \
                 distributions, sales, or worthlessness under § 165(g). Consider whether \
                 a sale before the distribution would better preserve the loss for the \
                 corporation; consult § 482 + § 267 related-party loss-deferral rules.",
                deemed_fmv / 100,
                input.adjusted_basis_cents / 100
            ),
        },
    }
}

fn distribution_label(distribution_type: DistributionType) -> &'static str {
    match distribution_type {
        DistributionType::OrdinarySection301Distribution => "ordinary § 301 distribution",
        DistributionType::Section332ParentSubLiquidationSection311Inapplicable => {
            "§ 332 parent-subsidiary liquidation"
        }
        DistributionType::Section336GeneralLiquidationSection311Inapplicable => {
            "§ 336 general corporate liquidation"
        }
        DistributionType::Section355CorporateDivisionSection311Inapplicable => {
            "§ 355 corporate division (spinoff / split-off / split-up)"
        }
    }
}

fn distribution_governing_provision(distribution_type: DistributionType) -> &'static str {
    match distribution_type {
        DistributionType::Section332ParentSubLiquidationSection311Inapplicable => {
            "§ 332 + § 337 govern subsidiary-level non-recognition + parent-level \
             carry-over basis."
        }
        DistributionType::Section336GeneralLiquidationSection311Inapplicable => {
            "§ 336 governs gain/loss recognition on general liquidation distributions \
             (including loss recognition that § 311 denies for ordinary distributions)."
        }
        DistributionType::Section355CorporateDivisionSection311Inapplicable => {
            "§ 355 governs corporate-division non-recognition subject to active-trade-or- \
             business + device-test requirements."
        }
        DistributionType::OrdinarySection301Distribution => "",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_appreciated() -> Input {
        Input {
            distribution_type: DistributionType::OrdinarySection301Distribution,
            appreciation_status:
                AppreciationStatus::FmvExceedsAdjustedBasisAppreciated,
            partnership_trust_anti_loss_status:
                PartnershipTrustAntiLossStatus::NotPartnershipOrTrustInterest,
            fair_market_value_cents: 100_000_00,
            adjusted_basis_cents: 30_000_00,
            liability_assumed_or_property_subject_to_cents: 0,
        }
    }

    #[test]
    fn section_332_inapplicable_to_section_311() {
        let mut input = base_appreciated();
        input.distribution_type =
            DistributionType::Section332ParentSubLiquidationSection311Inapplicable;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section311InapplicableOtherDistributionRegime
        );
        assert!(output.note.contains("§ 332"));
        assert!(output.note.contains("§ 337"));
    }

    #[test]
    fn section_336_inapplicable_to_section_311() {
        let mut input = base_appreciated();
        input.distribution_type =
            DistributionType::Section336GeneralLiquidationSection311Inapplicable;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section311InapplicableOtherDistributionRegime
        );
        assert!(output.note.contains("§ 336"));
    }

    #[test]
    fn section_355_inapplicable_to_section_311() {
        let mut input = base_appreciated();
        input.distribution_type =
            DistributionType::Section355CorporateDivisionSection311Inapplicable;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section311InapplicableOtherDistributionRegime
        );
        assert!(output.note.contains("§ 355"));
    }

    #[test]
    fn appreciated_property_triggers_section_311b_gain() {
        let input = base_appreciated();
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section311BGainRecognitionAppreciatedProperty
        );
        // FMV $100K - basis $30K = $70K gain
        assert_eq!(output.recognized_gain_cents, 70_000_00);
        assert_eq!(output.recognized_loss_cents, 0);
        assert!(output.note.contains("§ 311(b)(1)"));
        assert!(output.note.contains("General Utilities"));
        assert!(output.note.contains("Tax Reform Act of 1986"));
    }

    #[test]
    fn neutral_fmv_basis_no_gain_or_loss() {
        let mut input = base_appreciated();
        input.appreciation_status = AppreciationStatus::FmvEqualsAdjustedBasisNeutral;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section311ANoGainOrLossNeutralDistribution
        );
        assert_eq!(output.recognized_gain_cents, 0);
        assert!(output.note.contains("§ 311(a)"));
    }

    #[test]
    fn depreciated_property_no_loss_recognition() {
        let mut input = base_appreciated();
        input.appreciation_status =
            AppreciationStatus::FmvBelowAdjustedBasisDepreciated;
        input.fair_market_value_cents = 20_000_00;
        input.adjusted_basis_cents = 50_000_00;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section311ANoLossOnDepreciatedPropertyDistribution
        );
        assert_eq!(output.recognized_loss_cents, 0);
        assert!(output.note.contains("§ 336"));
        assert!(output.note.contains("§ 165(g)"));
        assert!(output.note.contains("§ 267"));
    }

    #[test]
    fn liability_floor_applies_when_liability_exceeds_fmv() {
        let mut input = base_appreciated();
        input.fair_market_value_cents = 80_000_00;
        input.adjusted_basis_cents = 30_000_00;
        input.liability_assumed_or_property_subject_to_cents = 120_000_00;
        let output = check(&input);
        // Liability $120K > FMV $80K → deemed FMV $120K
        // Gain = $120K - $30K = $90K
        assert_eq!(
            output.severity,
            Severity::Section311BWithLiabilityAssumptionFmvFloorAdjustment
        );
        assert_eq!(output.deemed_fmv_after_liability_floor_cents, 120_000_00);
        assert_eq!(output.recognized_gain_cents, 90_000_00);
        assert!(output.note.contains("§ 311(b)(2)"));
        assert!(output.note.contains("LIABILITY FLOOR"));
    }

    #[test]
    fn liability_below_fmv_no_floor_adjustment() {
        let mut input = base_appreciated();
        input.liability_assumed_or_property_subject_to_cents = 50_000_00;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section311BGainRecognitionAppreciatedProperty
        );
        assert_eq!(output.deemed_fmv_after_liability_floor_cents, 100_000_00);
        assert_eq!(output.recognized_gain_cents, 70_000_00);
    }

    #[test]
    fn partnership_anti_loss_scrubs_loss_recognition() {
        let mut input = base_appreciated();
        input.partnership_trust_anti_loss_status =
            PartnershipTrustAntiLossStatus::PartnershipOrTrustInterestPrincipalPurposeOfLossRecognition;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section311B3PartnershipTrustAntiLossDisallowance
        );
        assert!(output.note.contains("§ 311(b)(3)"));
        assert!(output.note.contains("Treas. Reg. § 1.311-2"));
    }

    #[test]
    fn partnership_bona_fide_contribution_no_anti_loss() {
        let mut input = base_appreciated();
        input.partnership_trust_anti_loss_status =
            PartnershipTrustAntiLossStatus::PartnershipOrTrustInterestBonaFideContribution;
        let output = check(&input);
        // Bona-fide contribution → normal appreciated-property branch
        assert_eq!(
            output.severity,
            Severity::Section311BGainRecognitionAppreciatedProperty
        );
    }

    #[test]
    fn note_pins_section_1374_s_corp_built_in_gain() {
        let input = base_appreciated();
        let output = check(&input);
        assert!(output.note.contains("§ 1374"));
    }

    #[test]
    fn note_pins_section_312_eep_computation() {
        let input = base_appreciated();
        let output = check(&input);
        assert!(output.note.contains("§ 312"));
    }

    #[test]
    fn note_pins_section_1245_1250_depreciation_recapture() {
        let input = base_appreciated();
        let output = check(&input);
        assert!(output.note.contains("§ 1245"));
        assert!(output.note.contains("§ 1250"));
    }

    #[test]
    fn note_pins_section_301d_distributee_basis() {
        let mut input = base_appreciated();
        input.appreciation_status = AppreciationStatus::FmvEqualsAdjustedBasisNeutral;
        let output = check(&input);
        assert!(output.note.contains("§ 301(d)"));
    }

    #[test]
    fn very_large_fmv_no_overflow() {
        let mut input = base_appreciated();
        input.fair_market_value_cents = u64::MAX;
        let output = check(&input);
        assert!(output.recognized_gain_cents > 0);
    }

    #[test]
    fn zero_fmv_no_panic() {
        let mut input = base_appreciated();
        input.fair_market_value_cents = 0;
        input.adjusted_basis_cents = 0;
        input.appreciation_status = AppreciationStatus::FmvEqualsAdjustedBasisNeutral;
        let output = check(&input);
        assert_eq!(output.recognized_gain_cents, 0);
    }

    #[test]
    fn other_distribution_regime_overrides_appreciation_analysis() {
        let mut input = base_appreciated();
        input.distribution_type =
            DistributionType::Section332ParentSubLiquidationSection311Inapplicable;
        // Even appreciated property + partnership-anti-loss flag, § 332 wins
        input.partnership_trust_anti_loss_status =
            PartnershipTrustAntiLossStatus::PartnershipOrTrustInterestPrincipalPurposeOfLossRecognition;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section311InapplicableOtherDistributionRegime
        );
    }
}
