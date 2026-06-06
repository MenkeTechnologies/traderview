//! IRC § 482 allocation of income and deductions among taxpayers (transfer pricing).
//!
//! § 482 grants Treasury authority to DISTRIBUTE, APPORTION, or ALLOCATE gross income,
//! deductions, credits, or allowances between or among two or more organizations,
//! trades, or businesses owned or controlled directly or indirectly by the same
//! interests, when necessary to PREVENT EVASION OF TAXES or to clearly REFLECT THE
//! INCOME of the controlled taxpayers. § 482 is the cornerstone of US transfer-pricing
//! enforcement and applies to intercompany transactions between related parties
//! (parent-subsidiary, brother-sister corporations, controlled partnerships, related
//! individuals + entities).
//!
//! § 482 ARM'S-LENGTH STANDARD (Treas. Reg. § 1.482-1): a controlled transaction
//! satisfies the arm's-length standard if the results are consistent with the results
//! that would have been realized if uncontrolled taxpayers had engaged in the same
//! transaction under the same circumstances.
//!
//! § 482 BEST-METHOD RULE (Treas. Reg. § 1.482-1(c)): the arm's-length result must
//! be determined under the method that, under the facts and circumstances, provides
//! the most reliable measure of an arm's-length result. No strict hierarchy among
//! methods.
//!
//! § 482 TRANSFER-PRICING METHODS for tangible property (Treas. Reg. § 1.482-3):
//!   (1) Comparable Uncontrolled Price (CUP) — most direct, requires actual
//!       comparable uncontrolled transaction;
//!   (2) Resale Price Method (RPM) — gross margin of distributor benchmarked against
//!       uncontrolled distributors;
//!   (3) Cost Plus Method — gross markup on costs benchmarked;
//!   (4) Comparable Profits Method (CPM) — operating margin benchmarked against
//!       comparable companies;
//!   (5) Profit Split Method — split residual operating profit per relative
//!       contributions.
//!
//! § 482 INTANGIBLE-PROPERTY TRANSFERS: Tax Reform Act of 1986 amendment requires
//! that consideration for intangible property transferred in a controlled transaction
//! be COMMENSURATE WITH THE INCOME attributable to the intangible. Treas. Reg.
//! § 1.482-4 + Comparable Uncontrolled Transaction (CUT) method.
//!
//! § 482 SERVICES (Treas. Reg. § 1.482-9): comparable uncontrolled services price,
//! gross services margin, cost-of-services-plus, comparable profits method for
//! services, profit split, services cost method (SCM) safe harbor for low-margin
//! services.
//!
//! § 482 COST SHARING ARRANGEMENTS (Treas. Reg. § 1.482-7): related parties may
//! share R&D costs proportional to expected benefits; Platform Contribution
//! Transaction (PCT) payment for pre-existing intangibles required.
//!
//! § 6662(e) SUBSTANTIAL VALUATION MISSTATEMENT PENALTY: 20% of underpayment if
//! § 482 adjustment exceeds $5 million or 10% of gross receipts (whichever greater);
//! 40% for gross valuation misstatement (200% / 50% threshold).
//!
//! § 6662(h) GROSS VALUATION MISSTATEMENT PENALTY: 40% — if claimed value 200% of
//! correct value (overvaluation) OR 25% of correct (undervaluation).
//!
//! Reporting: Form 5471 (US shareholder of CFC) + Form 5472 (US corp 25%-foreign-
//! owned + foreign branch) + Form 8975 Country-by-Country reporting (≥ $850M global
//! group revenue).
//!
//! Citations (verified per WebSearch 2026-06-02):
//! - law.cornell.edu/cfr/text/26/1.482-1
//! - law.cornell.edu/cfr/text/26/1.482-3
//! - irs.gov/pub/fatca/int_practice_units/ISO9411_07_01.pdf
//! - irs.gov/irm/part4/irm_04-011-005

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionType {
    /// Tangible property transfer.
    TangiblePropertyTransfer,
    /// Intangible property transfer (patent, trademark, know-how).
    IntangiblePropertyTransfer,
    /// Service provision (management, marketing, R&D services).
    ServiceProvision,
    /// Cost sharing arrangement (R&D cost sharing under § 1.482-7).
    CostSharingArrangementSection1482Dash7,
    /// Loan / financial transaction (intercompany debt).
    LoanOrFinancialTransaction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransferPricingMethodSelected {
    /// § 1.482-3(b) Comparable Uncontrolled Price.
    ComparableUncontrolledPriceCup,
    /// § 1.482-3(c) Resale Price Method.
    ResalePriceMethodRpm,
    /// § 1.482-3(d) Cost Plus Method.
    CostPlusMethod,
    /// § 1.482-5 Comparable Profits Method.
    ComparableProfitsMethodCpm,
    /// § 1.482-6 Profit Split Method.
    ProfitSplitMethod,
    /// § 1.482-9(b) Services Cost Method safe harbor.
    ServicesCostMethodSafeHarbor,
    /// § 1.482-4 Comparable Uncontrolled Transaction for intangibles.
    ComparableUncontrolledTransactionCut,
    /// Unspecified method (allowed if more reliable).
    UnspecifiedMethod,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdjustmentMagnitudeStatus {
    /// IRS § 482 adjustment within arm's-length range — no adjustment.
    WithinArmsLengthRangeNoAdjustment,
    /// IRS § 482 adjustment less than $5 million AND less than 10% gross receipts.
    AdjustmentBelowSection6662EThreshold,
    /// § 6662(e) substantial-valuation-misstatement penalty triggered.
    Section6662ESubstantialValuationMisstatement,
    /// § 6662(h) gross-valuation-misstatement penalty triggered (200% / 50%).
    Section6662HGrossValuationMisstatement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    Section482WithinArmsLengthRangeNoAdjustment,
    Section482AdjustmentBelowPenaltyThreshold,
    Section6662ESubstantialValuationMisstatement20Pct,
    Section6662HGrossValuationMisstatement40Pct,
    CostSharingPctPlatformContributionTransactionRequired,
    CommensurateWithIncomeStandardIntangible,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub transaction_type: TransactionType,
    pub transfer_pricing_method_selected: TransferPricingMethodSelected,
    pub adjustment_magnitude_status: AdjustmentMagnitudeStatus,
    pub irs_section_482_adjustment_amount_cents: u64,
}

pub type Section482TransferPricingInput = Input;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub severity: Severity,
    pub estimated_penalty_amount_cents: u64,
    pub note: String,
}

pub type Section482TransferPricingOutput = Output;
pub type Section482TransferPricingResult = Output;

#[allow(dead_code)]
const SECTION_6662E_SUBSTANTIAL_VALUATION_THRESHOLD_CENTS: u64 = 500_000_000;
const SECTION_6662E_PENALTY_RATE_BPS: u32 = 2_000;
const SECTION_6662H_PENALTY_RATE_BPS: u32 = 4_000;
#[allow(dead_code)]
const SUBSTANTIAL_VALUATION_PERCENT: u32 = 200;
const GROSS_VALUATION_PERCENT: u32 = 200;

#[must_use]
pub fn check(input: &Input) -> Output {
    if matches!(
        input.transaction_type,
        TransactionType::CostSharingArrangementSection1482Dash7
    ) {
        return Output {
            severity: Severity::CostSharingPctPlatformContributionTransactionRequired,
            estimated_penalty_amount_cents: 0,
            note: "§ 482 COST SHARING ARRANGEMENT (Treas. Reg. § 1.482-7): related parties \
                   may share R&D costs proportional to reasonably anticipated benefits. \
                   PLATFORM CONTRIBUTION TRANSACTION (PCT) payment required for pre-existing \
                   intangibles contributed by any participant. PCT valuation under § 1.482-7 \
                   (g) using best method (income method + acquisition price method + market \
                   capitalization method + residual profit split + unspecified methods). \
                   Annual reporting on Schedule M (Form 5471) + Form 8865. Coordinates with \
                   § 367(d) outbound intangible-property regime + § 250 FDII (foreign-derived \
                   intangible income deduction)."
                .to_string(),
        };
    }

    if matches!(
        input.transaction_type,
        TransactionType::IntangiblePropertyTransfer
    ) {
        return Output {
            severity: Severity::CommensurateWithIncomeStandardIntangible,
            estimated_penalty_amount_cents: 0,
            note: format!(
                "§ 482 INTANGIBLE-PROPERTY COMMENSURATE-WITH-INCOME STANDARD (Tax Reform Act \
                 of 1986 amendment + Treas. Reg. § 1.482-4): consideration for intangible \
                 property in controlled transaction must be COMMENSURATE WITH THE INCOME \
                 attributable to the intangible. Periodic adjustments mandated when actual \
                 income materially differs from projections. Comparable Uncontrolled \
                 Transaction (CUT) method preferred; income method + residual profit split \
                 also available. Method selected: {}. Method must satisfy best-method rule \
                 under § 1.482-1(c) + comparability analysis under § 1.482-1(d). Coordinates \
                 with § 367(d) outbound + § 482-7 cost sharing + § 250 FDII.",
                method_label(input.transfer_pricing_method_selected)
            ),
        };
    }

    match input.adjustment_magnitude_status {
        AdjustmentMagnitudeStatus::WithinArmsLengthRangeNoAdjustment => Output {
            severity: Severity::Section482WithinArmsLengthRangeNoAdjustment,
            estimated_penalty_amount_cents: 0,
            note: format!(
                "§ 482 arm's-length standard SATISFIED. Transaction priced within arm's-\
                 length range per Treas. Reg. § 1.482-1. No IRS adjustment. Method selected: \
                 {}. Maintain contemporaneous documentation per Treas. Reg. § 1.6662-6(d) to \
                 satisfy reasonable-cause-and-good-faith defense to § 6662 accuracy-related \
                 penalty.",
                method_label(input.transfer_pricing_method_selected)
            ),
        },
        AdjustmentMagnitudeStatus::AdjustmentBelowSection6662EThreshold => Output {
            severity: Severity::Section482AdjustmentBelowPenaltyThreshold,
            estimated_penalty_amount_cents: 0,
            note: format!(
                "§ 482 adjustment ${} below § 6662(e) substantial-valuation-misstatement \
                 threshold (greater of $5,000,000 OR 10% of gross receipts). Adjustment \
                 increases income but no § 6662 penalty. Method selected: {}. Document \
                 transfer-pricing analysis to defend against expanded examination.",
                input.irs_section_482_adjustment_amount_cents / 100,
                method_label(input.transfer_pricing_method_selected)
            ),
        },
        AdjustmentMagnitudeStatus::Section6662ESubstantialValuationMisstatement => {
            let penalty = u64::try_from(
                u128::from(input.irs_section_482_adjustment_amount_cents)
                    .saturating_mul(u128::from(SECTION_6662E_PENALTY_RATE_BPS))
                    .saturating_div(10_000),
            )
            .unwrap_or(u64::MAX);
            Output {
                severity: Severity::Section6662ESubstantialValuationMisstatement20Pct,
                estimated_penalty_amount_cents: penalty,
                note: format!(
                    "§ 6662(e) SUBSTANTIAL VALUATION MISSTATEMENT 20% PENALTY triggered. \
                     § 482 adjustment (${}) exceeds substantial-valuation threshold (greater \
                     of $5,000,000 or 10% gross receipts). Penalty: ${}. Comparable-profits-\
                     method 200% / 50% threshold under § 1.482-5. § 6664(c) reasonable-\
                     cause-and-good-faith defense requires contemporaneous documentation per \
                     § 1.6662-6(d) (best method selection + comparable analysis + economic \
                     analysis + financial data).",
                    input.irs_section_482_adjustment_amount_cents / 100,
                    penalty / 100
                ),
            }
        }
        AdjustmentMagnitudeStatus::Section6662HGrossValuationMisstatement => {
            let penalty = u64::try_from(
                u128::from(input.irs_section_482_adjustment_amount_cents)
                    .saturating_mul(u128::from(SECTION_6662H_PENALTY_RATE_BPS))
                    .saturating_div(10_000),
            )
            .unwrap_or(u64::MAX);
            Output {
                severity: Severity::Section6662HGrossValuationMisstatement40Pct,
                estimated_penalty_amount_cents: penalty,
                note: format!(
                    "§ 6662(h) GROSS VALUATION MISSTATEMENT 40% PENALTY triggered. § 482 \
                     adjustment (${}) results in {GROSS_VALUATION_PERCENT}% (or more) of \
                     correct value OR 25% (or less) of correct undervaluation. Penalty: ${}. \
                     § 6664(c) reasonable-cause defense NOT available for gross-valuation \
                     misstatement per § 6664(c)(3). Form 8275 + Form 8275-R disclosure \
                     required.",
                    input.irs_section_482_adjustment_amount_cents / 100,
                    penalty / 100
                ),
            }
        }
    }
}

fn method_label(method: TransferPricingMethodSelected) -> &'static str {
    match method {
        TransferPricingMethodSelected::ComparableUncontrolledPriceCup => {
            "§ 1.482-3(b) Comparable Uncontrolled Price (CUP)"
        }
        TransferPricingMethodSelected::ResalePriceMethodRpm => {
            "§ 1.482-3(c) Resale Price Method (RPM)"
        }
        TransferPricingMethodSelected::CostPlusMethod => "§ 1.482-3(d) Cost Plus Method",
        TransferPricingMethodSelected::ComparableProfitsMethodCpm => {
            "§ 1.482-5 Comparable Profits Method (CPM)"
        }
        TransferPricingMethodSelected::ProfitSplitMethod => "§ 1.482-6 Profit Split Method",
        TransferPricingMethodSelected::ServicesCostMethodSafeHarbor => {
            "§ 1.482-9(b) Services Cost Method (SCM) safe harbor"
        }
        TransferPricingMethodSelected::ComparableUncontrolledTransactionCut => {
            "§ 1.482-4 Comparable Uncontrolled Transaction (CUT) for intangibles"
        }
        TransferPricingMethodSelected::UnspecifiedMethod => {
            "Unspecified method (must satisfy best-method rule)"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Input {
        Input {
            transaction_type: TransactionType::TangiblePropertyTransfer,
            transfer_pricing_method_selected:
                TransferPricingMethodSelected::ComparableProfitsMethodCpm,
            adjustment_magnitude_status:
                AdjustmentMagnitudeStatus::WithinArmsLengthRangeNoAdjustment,
            irs_section_482_adjustment_amount_cents: 0,
        }
    }

    #[test]
    fn within_arms_length_range_no_adjustment() {
        let input = base();
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section482WithinArmsLengthRangeNoAdjustment
        );
        assert_eq!(output.estimated_penalty_amount_cents, 0);
        assert!(output.note.contains("§ 1.482-1"));
        assert!(output.note.contains("§ 1.6662-6(d)"));
    }

    #[test]
    fn adjustment_below_threshold_no_penalty() {
        let mut input = base();
        input.adjustment_magnitude_status =
            AdjustmentMagnitudeStatus::AdjustmentBelowSection6662EThreshold;
        input.irs_section_482_adjustment_amount_cents = 1_000_000_00;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section482AdjustmentBelowPenaltyThreshold
        );
        assert_eq!(output.estimated_penalty_amount_cents, 0);
    }

    #[test]
    fn substantial_valuation_misstatement_20_pct_penalty() {
        let mut input = base();
        input.adjustment_magnitude_status =
            AdjustmentMagnitudeStatus::Section6662ESubstantialValuationMisstatement;
        input.irs_section_482_adjustment_amount_cents = 10_000_000_00;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section6662ESubstantialValuationMisstatement20Pct
        );
        // 20% × $10M = $2M
        assert_eq!(output.estimated_penalty_amount_cents, 2_000_000_00);
        assert!(output.note.contains("§ 6662(e)"));
        assert!(output.note.contains("§ 1.6662-6(d)"));
    }

    #[test]
    fn gross_valuation_misstatement_40_pct_penalty() {
        let mut input = base();
        input.adjustment_magnitude_status =
            AdjustmentMagnitudeStatus::Section6662HGrossValuationMisstatement;
        input.irs_section_482_adjustment_amount_cents = 5_000_000_00;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section6662HGrossValuationMisstatement40Pct
        );
        // 40% × $5M = $2M
        assert_eq!(output.estimated_penalty_amount_cents, 2_000_000_00);
        assert!(output.note.contains("§ 6662(h)"));
        assert!(output.note.contains("§ 6664(c)(3)"));
    }

    #[test]
    fn cost_sharing_arrangement_pct_required() {
        let mut input = base();
        input.transaction_type = TransactionType::CostSharingArrangementSection1482Dash7;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::CostSharingPctPlatformContributionTransactionRequired
        );
        assert!(output.note.contains("§ 1.482-7"));
        assert!(output.note.contains("PLATFORM CONTRIBUTION"));
        assert!(output.note.contains("§ 367(d)"));
        assert!(output.note.contains("§ 250"));
    }

    #[test]
    fn intangible_property_commensurate_with_income_standard() {
        let mut input = base();
        input.transaction_type = TransactionType::IntangiblePropertyTransfer;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::CommensurateWithIncomeStandardIntangible
        );
        assert!(output.note.contains("COMMENSURATE WITH THE INCOME"));
        assert!(output.note.contains("Tax Reform Act of 1986"));
        assert!(output.note.contains("§ 1.482-4"));
    }

    #[test]
    fn cup_method_label_pinned() {
        let mut input = base();
        input.transfer_pricing_method_selected =
            TransferPricingMethodSelected::ComparableUncontrolledPriceCup;
        let output = check(&input);
        assert!(output.note.contains("§ 1.482-3(b)"));
        assert!(output.note.contains("CUP"));
    }

    #[test]
    fn resale_price_method_label_pinned() {
        let mut input = base();
        input.transfer_pricing_method_selected =
            TransferPricingMethodSelected::ResalePriceMethodRpm;
        let output = check(&input);
        assert!(output.note.contains("§ 1.482-3(c)"));
        assert!(output.note.contains("RPM"));
    }

    #[test]
    fn cost_plus_method_label_pinned() {
        let mut input = base();
        input.transfer_pricing_method_selected = TransferPricingMethodSelected::CostPlusMethod;
        let output = check(&input);
        assert!(output.note.contains("§ 1.482-3(d)"));
    }

    #[test]
    fn cpm_method_label_pinned() {
        let mut input = base();
        input.transfer_pricing_method_selected =
            TransferPricingMethodSelected::ComparableProfitsMethodCpm;
        let output = check(&input);
        assert!(output.note.contains("§ 1.482-5"));
        assert!(output.note.contains("CPM"));
    }

    #[test]
    fn profit_split_method_label_pinned() {
        let mut input = base();
        input.transfer_pricing_method_selected = TransferPricingMethodSelected::ProfitSplitMethod;
        let output = check(&input);
        assert!(output.note.contains("§ 1.482-6"));
    }

    #[test]
    fn services_cost_method_safe_harbor_label_pinned() {
        let mut input = base();
        input.transaction_type = TransactionType::ServiceProvision;
        input.transfer_pricing_method_selected =
            TransferPricingMethodSelected::ServicesCostMethodSafeHarbor;
        let output = check(&input);
        assert!(output.note.contains("§ 1.482-9(b)"));
    }

    #[test]
    fn cut_method_label_pinned() {
        let mut input = base();
        input.transaction_type = TransactionType::IntangiblePropertyTransfer;
        input.transfer_pricing_method_selected =
            TransferPricingMethodSelected::ComparableUncontrolledTransactionCut;
        let output = check(&input);
        assert!(output.note.contains("§ 1.482-4"));
        assert!(output.note.contains("CUT"));
    }

    #[test]
    fn section_6662e_threshold_constant_pins_5m() {
        assert_eq!(
            SECTION_6662E_SUBSTANTIAL_VALUATION_THRESHOLD_CENTS,
            500_000_000
        );
    }

    #[test]
    fn section_6662e_penalty_rate_constant_pins_20_pct() {
        assert_eq!(SECTION_6662E_PENALTY_RATE_BPS, 2_000);
    }

    #[test]
    fn section_6662h_penalty_rate_constant_pins_40_pct() {
        assert_eq!(SECTION_6662H_PENALTY_RATE_BPS, 4_000);
    }

    #[test]
    fn substantial_valuation_threshold_pins_200_pct() {
        assert_eq!(SUBSTANTIAL_VALUATION_PERCENT, 200);
    }

    #[test]
    fn gross_valuation_threshold_pins_200_pct() {
        assert_eq!(GROSS_VALUATION_PERCENT, 200);
    }

    #[test]
    fn very_large_adjustment_no_overflow() {
        let mut input = base();
        input.adjustment_magnitude_status =
            AdjustmentMagnitudeStatus::Section6662ESubstantialValuationMisstatement;
        input.irs_section_482_adjustment_amount_cents = u64::MAX;
        let output = check(&input);
        // u128 intermediate prevents overflow
        assert!(output.estimated_penalty_amount_cents > 0);
    }

    #[test]
    fn zero_adjustment_zero_penalty() {
        let mut input = base();
        input.irs_section_482_adjustment_amount_cents = 0;
        let output = check(&input);
        assert_eq!(output.estimated_penalty_amount_cents, 0);
    }
}
