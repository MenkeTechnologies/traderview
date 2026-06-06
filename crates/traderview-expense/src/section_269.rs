//! IRC § 269 acquisitions made to evade or avoid income tax.
//!
//! § 269 gives Treasury authority to DISALLOW any deduction, credit, or other allowance
//! when (1) a person acquires control of a corporation OR a corporation acquires the
//! property of another corporation (with carry-over basis), AND (2) the principal purpose
//! of the acquisition was to evade or avoid federal income tax by securing a benefit the
//! acquirer would not otherwise enjoy. § 269 is the primary anti-loss-trafficking weapon
//! and operates ALONGSIDE the more mechanical § 382 NOL limitation (post-1986) — § 269
//! still applies to non-NOL benefits (general business credits, accelerated depreciation,
//! foreign tax credits, charitable contribution carryovers) that § 382 does not capture.
//!
//! § 269(a) STATUTORY TEXT (paraphrased): if any person(s) acquire DIRECTLY or
//! INDIRECTLY control of a corporation, OR any corporation acquires property of another
//! corporation, not previously controlled, with the basis to the transferee being
//! determined by reference to the transferor's basis (carry-over basis transaction), AND
//! the principal purpose is evasion or avoidance of federal income tax by securing the
//! benefit of a deduction/credit/allowance the person or corporation would not otherwise
//! enjoy, the Secretary may disallow such deduction/credit/allowance.
//!
//! § 269(a)(1) "CONTROL" THRESHOLD: ownership of at least 50% of total combined voting
//! power OR at least 50% of total value of all classes of stock.
//!
//! § 269(b) NOL CARRY-OVER LIMITATION on liquidations within 2 years after acquisition
//! qualifying as § 332 liquidation: IRS may disallow the resulting NOL carry-over.
//!
//! § 269(c) REBUTTABLE PRESUMPTION: if the purchase price for the acquired stock is
//! substantially disproportionate to the value of the assets acquired (excluding the
//! value of the tax benefits sought), tax-avoidance principal purpose is presumed.
//!
//! § 382 COORDINATION: § 269 may apply to NOL carryovers even when § 382 also applies;
//! Treas. Reg. § 1.269-3(d) provides that § 269 control acquisition in connection with
//! § 382(l)(5) bankruptcy-reorganization ownership change is per se for principal
//! purpose of evasion UNLESS the acquired corporation carries on more than an
//! insignificant amount of an active trade or business during and subsequent to the
//! title 11 case.
//!
//! Citations (verified per WebSearch 2026-06-02):
//! - law.cornell.edu/uscode/text/26/269
//! - law.cornell.edu/cfr/text/26/1.269-3
//! - irc.bloombergtax.com/public/uscode/doc/irc/section_269
//! - journalofaccountancy.com/issues/2021/feb/tax-benefits-of-a-corporation/

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AcquisitionType {
    /// Stock acquisition triggering § 269(a)(1) "control" analysis.
    StockAcquisitionSection269A1,
    /// Asset acquisition with carry-over basis triggering § 269(a)(2).
    AssetAcquisitionCarryOverBasisSection269A2,
    /// Section 332 parent-subsidiary liquidation within 2 years — § 269(b).
    Section332LiquidationWithinTwoYearsSection269B,
    /// Section 382(l)(5) bankruptcy reorganization ownership change — Treas. Reg.
    /// § 1.269-3(d) per se presumption applies.
    Section382L5BankruptcyOwnershipChange,
    /// No acquisition of control or carry-over basis transfer — § 269 inapplicable.
    NoAcquisitionOfControl,
}

/// Whether § 269(a) "control" threshold is satisfied (50% vote OR 50% value).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ControlThresholdStatus {
    ControlAcquiredFiftyPercentOrMore,
    ControlNotAcquiredBelowFiftyPercent,
}

/// Whether the principal purpose of the acquisition was tax avoidance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrincipalPurposeStatus {
    /// Principal purpose was tax avoidance — § 269 applies.
    PrincipalPurposeIsTaxAvoidance,
    /// Principal purpose was business-economic — § 269 inapplicable.
    PrincipalPurposeIsBonaFideBusiness,
    /// Disproportionate purchase price → § 269(c) rebuttable presumption of
    /// tax-avoidance principal purpose.
    PriceDisproportionateSection269CPresumption,
    /// § 382(l)(5) bankruptcy reorganization — Treas. Reg. § 1.269-3(d) per se
    /// presumption applies UNLESS active-trade-or-business carried on post-title-11.
    Section382L5BankruptcyPerSePresumptionActiveBusinessNotMaintained,
    /// § 382(l)(5) bankruptcy reorganization — active-trade-or-business maintained,
    /// rebuts per se presumption.
    Section382L5BankruptcyPerSePresumptionRebuttedActiveBusinessMaintained,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    NoControlAcquisitionSection269Inapplicable,
    BonaFideBusinessPurposeNoDisallowance,
    Section382L5BankruptcyActiveBusinessMaintainedNoDisallowance,
    Section269ATaxBenefitDisallowanceApplied,
    Section269BNolCarryoverDisallowanceApplied,
    Section269CPriceDisproportionatePresumptionApplied,
    Section382L5PerSePresumptionDisallowanceApplied,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub acquisition_type: AcquisitionType,
    pub control_threshold_status: ControlThresholdStatus,
    pub principal_purpose_status: PrincipalPurposeStatus,
    pub tax_benefit_sought_cents: u64,
    pub active_business_purpose_evidence_strength_bps: u32,
}

pub type Section269AcquisitionsToEvadeTaxInput = Input;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub severity: Severity,
    pub disallowed_benefit_cents: u64,
    pub allowed_benefit_cents: u64,
    pub note: String,
}

pub type Section269AcquisitionsToEvadeTaxOutput = Output;
pub type Section269AcquisitionsToEvadeTaxResult = Output;

const CONTROL_THRESHOLD_PERCENT: u32 = 50;
const SECTION_269B_LIQUIDATION_WINDOW_YEARS: u32 = 2;

#[must_use]
pub fn check(input: &Input) -> Output {
    if matches!(
        input.acquisition_type,
        AcquisitionType::NoAcquisitionOfControl
    ) || matches!(
        input.control_threshold_status,
        ControlThresholdStatus::ControlNotAcquiredBelowFiftyPercent
    ) {
        return Output {
            severity: Severity::NoControlAcquisitionSection269Inapplicable,
            disallowed_benefit_cents: 0,
            allowed_benefit_cents: input.tax_benefit_sought_cents,
            note: format!(
                "§ 269 inapplicable: control threshold not met. § 269(a) requires acquisition \
                 of at least {CONTROL_THRESHOLD_PERCENT}% combined voting power OR \
                 {CONTROL_THRESHOLD_PERCENT}% total value of all classes of stock. Tax benefit \
                 ${} fully preserved (subject to other limitations including § 382 NOL cap, \
                 § 383 credit limitation, § 384 built-in-gain cap).",
                input.tax_benefit_sought_cents / 100
            ),
        };
    }

    if matches!(
        input.acquisition_type,
        AcquisitionType::Section382L5BankruptcyOwnershipChange
    ) {
        return apply_section_382l5_per_se_branch(input);
    }

    if matches!(
        input.principal_purpose_status,
        PrincipalPurposeStatus::PrincipalPurposeIsBonaFideBusiness
    ) {
        return Output {
            severity: Severity::BonaFideBusinessPurposeNoDisallowance,
            disallowed_benefit_cents: 0,
            allowed_benefit_cents: input.tax_benefit_sought_cents,
            note: format!(
                "§ 269 inapplicable: principal purpose of acquisition was bona-fide business \
                 (synergy, market expansion, vertical integration, talent acquisition, supply \
                 chain consolidation), not tax avoidance. § 269(a) applies only when the \
                 evasion-or-avoidance purpose EXCEEDS IN IMPORTANCE any other purpose. Tax \
                 benefit ${} preserved. Document the business rationale: board minutes, \
                 strategic-rationale memorandum, fairness opinion, third-party valuation, \
                 due-diligence record. § 382 NOL cap may still apply separately (mechanical \
                 limit not dependent on purpose).",
                input.tax_benefit_sought_cents / 100
            ),
        };
    }

    if matches!(
        input.principal_purpose_status,
        PrincipalPurposeStatus::PriceDisproportionateSection269CPresumption
    ) {
        let rebuttal_strength = input.active_business_purpose_evidence_strength_bps;
        let rebutted = rebuttal_strength >= 7_500;
        if rebutted {
            return Output {
                severity: Severity::BonaFideBusinessPurposeNoDisallowance,
                disallowed_benefit_cents: 0,
                allowed_benefit_cents: input.tax_benefit_sought_cents,
                note: format!(
                    "§ 269(c) rebuttable presumption REBUTTED. Disproportionate purchase price \
                     (purchase price substantially exceeds asset value excluding tax benefits) \
                     creates presumption of tax-avoidance principal purpose, but strong \
                     business-purpose evidence ({} bps = >= 75%) rebuts the presumption. Tax \
                     benefit ${} preserved.",
                    rebuttal_strength,
                    input.tax_benefit_sought_cents / 100
                ),
            };
        }
        return Output {
            severity: Severity::Section269CPriceDisproportionatePresumptionApplied,
            disallowed_benefit_cents: input.tax_benefit_sought_cents,
            allowed_benefit_cents: 0,
            note: format!(
                "§ 269(c) rebuttable presumption APPLIED. Purchase price substantially \
                 disproportionate to asset value (excluding the value of tax benefits sought) \
                 creates presumption of tax-avoidance principal purpose. Business-purpose \
                 rebuttal evidence ({} bps) insufficient (threshold = 7,500 bps = 75%). Tax \
                 benefit ${} disallowed. Coordinate with § 382 NOL cap + § 383 credit \
                 limitation + § 384 built-in-gain limit for non-discretionary parallel caps.",
                input.active_business_purpose_evidence_strength_bps,
                input.tax_benefit_sought_cents / 100
            ),
        };
    }

    match input.acquisition_type {
        AcquisitionType::Section332LiquidationWithinTwoYearsSection269B => Output {
            severity: Severity::Section269BNolCarryoverDisallowanceApplied,
            disallowed_benefit_cents: input.tax_benefit_sought_cents,
            allowed_benefit_cents: 0,
            note: format!(
                "§ 269(b) disallowance applied. § 332 parent-subsidiary liquidation within \
                 {SECTION_269B_LIQUIDATION_WINDOW_YEARS}-year window after acquisition, \
                 combined with tax-avoidance principal purpose, triggers § 269(b) NOL \
                 carry-over disallowance. Tax benefit ${} disallowed. Liquidation-vs-merger \
                 form arbitrage closed by the 2-year window.",
                input.tax_benefit_sought_cents / 100
            ),
        },
        AcquisitionType::StockAcquisitionSection269A1
        | AcquisitionType::AssetAcquisitionCarryOverBasisSection269A2 => Output {
            severity: Severity::Section269ATaxBenefitDisallowanceApplied,
            disallowed_benefit_cents: input.tax_benefit_sought_cents,
            allowed_benefit_cents: 0,
            note: format!(
                "§ 269(a) disallowance applied. Control acquired (at least {CONTROL_THRESHOLD_PERCENT}% \
                 voting power or value) AND principal purpose is tax avoidance — securing a \
                 deduction/credit/allowance the acquirer would not otherwise enjoy. Treasury \
                 may disallow the tax benefit at its discretion. Tax benefit ${} disallowed. \
                 Common fact patterns: profitable corp acquires loss-corp NOL/credit \
                 carryforwards; loss-corp acquires profitable target. Coordinates with § 382 \
                 NOL annual cap (mechanical), § 383 general-business-credit cap, § 384 \
                 built-in-gain limit, and § 384 SRLY rules.",
                input.tax_benefit_sought_cents / 100
            ),
        },
        AcquisitionType::Section382L5BankruptcyOwnershipChange
        | AcquisitionType::NoAcquisitionOfControl => unreachable!(),
    }
}

fn apply_section_382l5_per_se_branch(input: &Input) -> Output {
    match input.principal_purpose_status {
        PrincipalPurposeStatus::Section382L5BankruptcyPerSePresumptionRebuttedActiveBusinessMaintained => {
            Output {
                severity: Severity::Section382L5BankruptcyActiveBusinessMaintainedNoDisallowance,
                disallowed_benefit_cents: 0,
                allowed_benefit_cents: input.tax_benefit_sought_cents,
                note: format!(
                    "Treas. Reg. § 1.269-3(d) per se presumption REBUTTED. § 382(l)(5) \
                     bankruptcy reorganization ownership change is per se for tax-avoidance \
                     principal purpose UNLESS the corporation carries on more than an \
                     insignificant amount of an active trade or business during AND subsequent \
                     to the title 11 case. Active-business element documented (Plan of \
                     Reorganization carries forward operating divisions, employee retention, \
                     post-emergence revenue). Tax benefit ${} preserved.",
                    input.tax_benefit_sought_cents / 100
                ),
            }
        }
        _ => Output {
            severity: Severity::Section382L5PerSePresumptionDisallowanceApplied,
            disallowed_benefit_cents: input.tax_benefit_sought_cents,
            allowed_benefit_cents: 0,
            note: format!(
                "Treas. Reg. § 1.269-3(d) per se presumption APPLIED. § 382(l)(5) bankruptcy \
                 reorganization ownership change is per se for principal purpose of evasion \
                 or avoidance UNLESS the corporation maintains more than an insignificant \
                 amount of an active trade or business during AND subsequent to the title 11 \
                 case. Active-business element not established; presumption stands. Tax \
                 benefit ${} disallowed. Coordinate with § 382 NOL cap + § 382(l)(5) special \
                 bankruptcy rules.",
                input.tax_benefit_sought_cents / 100
            ),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Input {
        Input {
            acquisition_type: AcquisitionType::StockAcquisitionSection269A1,
            control_threshold_status: ControlThresholdStatus::ControlAcquiredFiftyPercentOrMore,
            principal_purpose_status: PrincipalPurposeStatus::PrincipalPurposeIsTaxAvoidance,
            tax_benefit_sought_cents: 50_000_000_00,
            active_business_purpose_evidence_strength_bps: 0,
        }
    }

    #[test]
    fn no_acquisition_of_control_section_269_inapplicable() {
        let mut input = base();
        input.acquisition_type = AcquisitionType::NoAcquisitionOfControl;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::NoControlAcquisitionSection269Inapplicable
        );
        assert_eq!(output.allowed_benefit_cents, 50_000_000_00);
        assert!(output.note.contains("50%"));
        assert!(output.note.contains("§ 382"));
    }

    #[test]
    fn control_below_50_pct_section_269_inapplicable() {
        let mut input = base();
        input.control_threshold_status =
            ControlThresholdStatus::ControlNotAcquiredBelowFiftyPercent;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::NoControlAcquisitionSection269Inapplicable
        );
        assert_eq!(output.allowed_benefit_cents, 50_000_000_00);
    }

    #[test]
    fn bona_fide_business_purpose_preserves_tax_benefit() {
        let mut input = base();
        input.principal_purpose_status = PrincipalPurposeStatus::PrincipalPurposeIsBonaFideBusiness;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::BonaFideBusinessPurposeNoDisallowance
        );
        assert_eq!(output.allowed_benefit_cents, 50_000_000_00);
        assert!(output.note.contains("synergy"));
        assert!(output.note.contains("EXCEEDS IN IMPORTANCE"));
    }

    #[test]
    fn section_269a_disallowance_for_stock_acquisition_tax_avoidance() {
        let input = base();
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section269ATaxBenefitDisallowanceApplied
        );
        assert_eq!(output.disallowed_benefit_cents, 50_000_000_00);
        assert_eq!(output.allowed_benefit_cents, 0);
        assert!(output.note.contains("§ 269(a)"));
        assert!(output.note.contains("§ 382"));
        assert!(output.note.contains("§ 383"));
    }

    #[test]
    fn section_269a2_asset_acquisition_carryover_basis_disallowance() {
        let mut input = base();
        input.acquisition_type = AcquisitionType::AssetAcquisitionCarryOverBasisSection269A2;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section269ATaxBenefitDisallowanceApplied
        );
        assert_eq!(output.disallowed_benefit_cents, 50_000_000_00);
    }

    #[test]
    fn section_269b_section_332_liquidation_within_2_years_disallowance() {
        let mut input = base();
        input.acquisition_type = AcquisitionType::Section332LiquidationWithinTwoYearsSection269B;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section269BNolCarryoverDisallowanceApplied
        );
        assert!(output.note.contains("§ 269(b)"));
        assert!(output.note.contains("§ 332"));
    }

    #[test]
    fn section_269c_price_disproportionate_presumption_applied() {
        let mut input = base();
        input.principal_purpose_status =
            PrincipalPurposeStatus::PriceDisproportionateSection269CPresumption;
        input.active_business_purpose_evidence_strength_bps = 5_000;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section269CPriceDisproportionatePresumptionApplied
        );
        assert!(output.note.contains("§ 269(c)"));
        assert!(output.note.contains("75%"));
    }

    #[test]
    fn section_269c_presumption_rebutted_with_strong_business_evidence() {
        let mut input = base();
        input.principal_purpose_status =
            PrincipalPurposeStatus::PriceDisproportionateSection269CPresumption;
        input.active_business_purpose_evidence_strength_bps = 8_000;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::BonaFideBusinessPurposeNoDisallowance
        );
        assert_eq!(output.allowed_benefit_cents, 50_000_000_00);
        assert!(output.note.contains("REBUTTED"));
    }

    #[test]
    fn section_269c_presumption_boundary_at_7500_bps_rebuts() {
        let mut input = base();
        input.principal_purpose_status =
            PrincipalPurposeStatus::PriceDisproportionateSection269CPresumption;
        input.active_business_purpose_evidence_strength_bps = 7_500;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::BonaFideBusinessPurposeNoDisallowance
        );
    }

    #[test]
    fn section_269c_presumption_just_under_threshold_disallowance() {
        let mut input = base();
        input.principal_purpose_status =
            PrincipalPurposeStatus::PriceDisproportionateSection269CPresumption;
        input.active_business_purpose_evidence_strength_bps = 7_499;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section269CPriceDisproportionatePresumptionApplied
        );
    }

    #[test]
    fn section_382l5_bankruptcy_active_business_maintained_no_disallowance() {
        let mut input = base();
        input.acquisition_type = AcquisitionType::Section382L5BankruptcyOwnershipChange;
        input.principal_purpose_status =
            PrincipalPurposeStatus::Section382L5BankruptcyPerSePresumptionRebuttedActiveBusinessMaintained;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section382L5BankruptcyActiveBusinessMaintainedNoDisallowance
        );
        assert!(output.note.contains("Treas. Reg. § 1.269-3(d)"));
        assert!(output.note.contains("title 11"));
    }

    #[test]
    fn section_382l5_bankruptcy_no_active_business_per_se_disallowance() {
        let mut input = base();
        input.acquisition_type = AcquisitionType::Section382L5BankruptcyOwnershipChange;
        input.principal_purpose_status =
            PrincipalPurposeStatus::Section382L5BankruptcyPerSePresumptionActiveBusinessNotMaintained;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section382L5PerSePresumptionDisallowanceApplied
        );
        assert_eq!(output.disallowed_benefit_cents, 50_000_000_00);
    }

    #[test]
    fn control_threshold_constant_pins_50_percent() {
        assert_eq!(CONTROL_THRESHOLD_PERCENT, 50);
    }

    #[test]
    fn section_269b_liquidation_window_constant_pins_2_years() {
        assert_eq!(SECTION_269B_LIQUIDATION_WINDOW_YEARS, 2);
    }

    #[test]
    fn very_large_tax_benefit_no_overflow() {
        let mut input = base();
        input.tax_benefit_sought_cents = u64::MAX;
        let output = check(&input);
        assert_eq!(output.disallowed_benefit_cents, u64::MAX);
        assert_eq!(output.allowed_benefit_cents, 0);
    }

    #[test]
    fn zero_tax_benefit_no_panic() {
        let mut input = base();
        input.tax_benefit_sought_cents = 0;
        let output = check(&input);
        assert_eq!(output.disallowed_benefit_cents, 0);
        assert_eq!(output.allowed_benefit_cents, 0);
    }

    #[test]
    fn note_pins_section_383_general_business_credit_cap() {
        let input = base();
        let output = check(&input);
        assert!(output.note.contains("§ 383"));
    }

    #[test]
    fn note_pins_section_384_built_in_gain_limit() {
        let input = base();
        let output = check(&input);
        assert!(output.note.contains("§ 384"));
    }

    #[test]
    fn note_no_control_pins_382_383_384_separate_limits() {
        let mut input = base();
        input.acquisition_type = AcquisitionType::NoAcquisitionOfControl;
        let output = check(&input);
        assert!(output.note.contains("§ 382"));
        assert!(output.note.contains("§ 383"));
        assert!(output.note.contains("§ 384"));
    }

    #[test]
    fn bona_fide_business_purpose_overrides_section_269a_branch() {
        let mut input = base();
        input.acquisition_type = AcquisitionType::StockAcquisitionSection269A1;
        input.principal_purpose_status = PrincipalPurposeStatus::PrincipalPurposeIsBonaFideBusiness;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::BonaFideBusinessPurposeNoDisallowance
        );
    }

    #[test]
    fn no_control_takes_priority_over_principal_purpose_status() {
        let mut input = base();
        input.acquisition_type = AcquisitionType::NoAcquisitionOfControl;
        input.principal_purpose_status = PrincipalPurposeStatus::PrincipalPurposeIsTaxAvoidance;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::NoControlAcquisitionSection269Inapplicable
        );
    }
}
