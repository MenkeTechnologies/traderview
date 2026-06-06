//! IRC § 312 effect on earnings and profits.
//!
//! § 312 governs the computation and adjustment of a corporation's earnings and
//! profits (E&P). E&P is the touchstone for distribution character determination
//! throughout subchapter C: § 301(c) classifies distributions as dividends to the
//! extent of E&P, as basis recovery thereafter, and as capital gain on the excess.
//! § 302 redemption analysis turns on E&P when the redemption defaults to § 301
//! distribution. § 304 brother-sister recharacterization stacks acquiring then
//! issuing E&P. § 311 corp-level recognition increases E&P by the gain. § 245A
//! foreign-source DRD turns on dividend characterization (which itself depends on
//! E&P).
//!
//! § 312(a) GENERAL RULE — E&P decreased on distribution: (1) by amount of money
//! distributed, (2) by principal amount of obligations distributed, (3) by adjusted
//! basis of distributed property (other than money and obligations).
//!
//! § 312(b) DISTRIBUTION OF APPRECIATED PROPERTY: if § 311(b) gain is recognized on
//! distribution of appreciated property, E&P is INCREASED by the recognized gain
//! and then DECREASED by the FMV of the distributed property (not basis).
//!
//! § 312(c) LIABILITY ASSUMPTIONS: if shareholder assumes liability or distributed
//! property is subject to liability, the E&P decrease is REDUCED by the amount of
//! the liability (the corporation is relieved by the liability transfer).
//!
//! § 312(d) E&P EFFECT OF STOCK DIVIDENDS: § 305(a) non-taxable stock dividends do
//! NOT reduce E&P; § 305(b) taxable stock dividends do.
//!
//! § 312(k)(3) STRAIGHT-LINE DEPRECIATION FOR E&P: for tangible property to which
//! § 168 applies, the E&P depreciation adjustment is determined under the
//! Alternative Depreciation System (ADS, § 168(g)(2)). This requires recomputation
//! of depreciation on a straight-line basis (vs accelerated for taxable income),
//! creating a permanent timing difference. Pre-1972 taxable years used different
//! rules.
//!
//! § 312(n) SPECIAL E&P ADJUSTMENTS to align with economic income:
//!   - (n)(1) Construction-period interest: capitalize even if expensed for tax.
//!   - (n)(2) Intangible drilling + mineral exploration: capitalize over 60 months
//!     even if expensed.
//!   - (n)(3) Amortization of circulation + organizational costs: align with cost.
//!   - (n)(4) LIFO: LIFO inventory adjustments to FIFO basis.
//!   - (n)(5) Installment sales: E&P computed as if installment method NOT used
//!     (full recognition in year of sale).
//!   - (n)(6) Completed-contract method: percentage-of-completion for E&P.
//!   - (n)(7) Certain stock redemptions: § 302 / § 304 stacking adjustments.
//!   - (n)(8) Foreign corporation special rules: PTEP under § 959 + § 961 +
//!     CFC adjustments.
//!
//! Citations (verified per WebSearch 2026-06-02):
//! - law.cornell.edu/uscode/text/26/312
//! - law.cornell.edu/cfr/text/26/1.312-7
//! - taxnotes.com/research/federal/usc26/312
//! - thetaxadviser.com/issues/2013/oct/kaiser-oct2013/

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdjustmentType {
    /// § 312(a)(1) cash distribution decrease.
    CashDistributionSection312A1,
    /// § 312(a)(2) distribution of obligations decrease (principal amount).
    ObligationDistributionSection312A2,
    /// § 312(a)(3) distribution of property — basis decrease.
    PropertyDistributionAtBasisSection312A3,
    /// § 312(b) appreciated-property distribution — increase by gain, then decrease
    /// by FMV.
    AppreciatedPropertyDistributionSection312B,
    /// § 312(c) liability-assumption modifier (reduces E&P decrease by liability).
    LiabilityAssumptionModifierSection312C,
    /// § 312(d) stock dividend — § 305(a) non-taxable does not reduce; § 305(b)
    /// taxable does reduce.
    StockDividendSection312D,
    /// § 312(k)(3) ADS straight-line depreciation adjustment.
    AdsStraightLineDepreciationSection312K3,
    /// § 312(n)(5) installment-sale E&P computed as if NOT using installment
    /// method.
    InstallmentSaleSection312N5,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StockDividendTaxability {
    NonTaxableSection305A,
    TaxableSection305B,
    NotStockDividend,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    EepDecreaseCashOrObligation,
    EepDecreaseDistributedPropertyBasis,
    EepIncreaseAppreciationThenDecreaseFmv,
    EepDecreaseAdjustedByLiabilityAssumed,
    EepUnchangedNonTaxableStockDividendSection305A,
    EepReducedTaxableStockDividendSection305B,
    EepAdjustedAdsStraightLineSection312K3,
    EepFullRecognitionSection312N5InstallmentSale,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub adjustment_type: AdjustmentType,
    pub stock_dividend_taxability: StockDividendTaxability,
    pub starting_eep_cents: u64,
    pub cash_or_principal_distributed_cents: u64,
    pub property_basis_cents: u64,
    pub property_fmv_cents: u64,
    pub liability_assumed_or_subject_to_cents: u64,
    pub accelerated_tax_depreciation_cents: u64,
    pub straight_line_ads_depreciation_cents: u64,
    pub installment_method_gain_recognized_cents: u64,
    pub full_recognition_year_of_sale_gain_cents: u64,
}

pub type Section312EepAdjustmentInput = Input;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub severity: Severity,
    pub eep_adjustment_cents: i128,
    pub ending_eep_cents: i128,
    pub note: String,
}

pub type Section312EepAdjustmentOutput = Output;
pub type Section312EepAdjustmentResult = Output;

#[must_use]
pub fn check(input: &Input) -> Output {
    let starting = i128::from(input.starting_eep_cents);
    match input.adjustment_type {
        AdjustmentType::CashDistributionSection312A1
        | AdjustmentType::ObligationDistributionSection312A2 => {
            let decrease = i128::from(input.cash_or_principal_distributed_cents);
            Output {
                severity: Severity::EepDecreaseCashOrObligation,
                eep_adjustment_cents: -decrease,
                ending_eep_cents: starting - decrease,
                note: format!(
                    "§ 312(a) E&P DECREASED by amount of money / principal of obligations \
                     distributed (${}). Coordinates with § 301(c) distribution character — \
                     to extent of decreased E&P, distribution treated as dividend; excess as \
                     basis recovery (capital gain after basis exhausted).",
                    decrease / 100
                ),
            }
        }
        AdjustmentType::PropertyDistributionAtBasisSection312A3 => {
            let decrease = i128::from(input.property_basis_cents);
            Output {
                severity: Severity::EepDecreaseDistributedPropertyBasis,
                eep_adjustment_cents: -decrease,
                ending_eep_cents: starting - decrease,
                note: format!(
                    "§ 312(a)(3) E&P DECREASED by adjusted basis of distributed property \
                     (${}). Property is NOT appreciated; § 312(b) appreciated-property rule \
                     does not apply. § 311(a) general rule: no gain or loss to distributing \
                     corp.",
                    decrease / 100
                ),
            }
        }
        AdjustmentType::AppreciatedPropertyDistributionSection312B => {
            let recognized_gain = input
                .property_fmv_cents
                .saturating_sub(input.property_basis_cents);
            let net_adjustment = i128::from(recognized_gain) - i128::from(input.property_fmv_cents);
            Output {
                severity: Severity::EepIncreaseAppreciationThenDecreaseFmv,
                eep_adjustment_cents: net_adjustment,
                ending_eep_cents: starting + net_adjustment,
                note: format!(
                    "§ 312(b) APPRECIATED-PROPERTY DISTRIBUTION: E&P INCREASED by § 311(b) \
                     recognized gain (FMV ${} - basis ${} = ${}), then DECREASED by FMV \
                     ${}. Net effect: decrease by basis (equivalent to § 312(a)(3) outcome) \
                     PLUS gain recognition flows through to E&P. Coordinates with § 311 \
                     (iter 550) corp-level recognition + § 301(c) distributee dividend / \
                     basis-recovery / capital-gain split.",
                    input.property_fmv_cents / 100,
                    input.property_basis_cents / 100,
                    recognized_gain / 100,
                    input.property_fmv_cents / 100
                ),
            }
        }
        AdjustmentType::LiabilityAssumptionModifierSection312C => {
            let liability = i128::from(input.liability_assumed_or_subject_to_cents);
            Output {
                severity: Severity::EepDecreaseAdjustedByLiabilityAssumed,
                eep_adjustment_cents: liability,
                ending_eep_cents: starting + liability,
                note: format!(
                    "§ 312(c) LIABILITY ASSUMPTION modifier: E&P decrease REDUCED by amount \
                     of liability assumed by shareholder OR to which distributed property is \
                     subject (${}). Corporation is RELIEVED of the liability — economically \
                     equivalent to receiving cash equal to the liability amount.",
                    liability / 100
                ),
            }
        }
        AdjustmentType::StockDividendSection312D => match input.stock_dividend_taxability {
            StockDividendTaxability::NonTaxableSection305A => Output {
                severity: Severity::EepUnchangedNonTaxableStockDividendSection305A,
                eep_adjustment_cents: 0,
                ending_eep_cents: starting,
                note: "§ 312(d) + § 305(a): NON-TAXABLE stock dividend (proportional same-\
                       class stock to all shareholders) does NOT reduce E&P. No economic \
                       distribution occurred; the shareholders' aggregate interest remains \
                       unchanged. § 307 basis-allocation rule splits original-stock basis \
                       between original and new shares."
                    .to_string(),
            },
            StockDividendTaxability::TaxableSection305B => Output {
                severity: Severity::EepReducedTaxableStockDividendSection305B,
                eep_adjustment_cents: -i128::from(input.property_fmv_cents),
                ending_eep_cents: starting - i128::from(input.property_fmv_cents),
                note: format!(
                    "§ 312(d) + § 305(b): TAXABLE stock dividend (disproportionate, common-\
                     and-preferred mix, election to receive cash or stock, or other § 305(b) \
                     trigger) REDUCES E&P by FMV (${}) of the stock distributed. Shareholders \
                     treat as § 301 distribution.",
                    input.property_fmv_cents / 100
                ),
            },
            StockDividendTaxability::NotStockDividend => Output {
                severity: Severity::NotApplicable,
                eep_adjustment_cents: 0,
                ending_eep_cents: starting,
                note: "Adjustment type § 312(d) selected but stock-dividend-taxability flag \
                       indicates the distribution is NOT a stock dividend. Verify input \
                       consistency."
                    .to_string(),
            },
        },
        AdjustmentType::AdsStraightLineDepreciationSection312K3 => {
            let recapture = i128::from(input.accelerated_tax_depreciation_cents)
                - i128::from(input.straight_line_ads_depreciation_cents);
            Output {
                severity: Severity::EepAdjustedAdsStraightLineSection312K3,
                eep_adjustment_cents: recapture,
                ending_eep_cents: starting + recapture,
                note: format!(
                    "§ 312(k)(3) ADS STRAIGHT-LINE DEPRECIATION ADJUSTMENT: E&P depreciation \
                     computed under § 168(g)(2) Alternative Depreciation System (straight-\
                     line over ADS recovery period). Taxable-income accelerated depreciation \
                     ${} - ADS straight-line ${} = ${} ADDITIONAL E&P (taxable income claimed \
                     more depreciation than E&P allows). Creates permanent timing difference \
                     between taxable income and E&P.",
                    input.accelerated_tax_depreciation_cents / 100,
                    input.straight_line_ads_depreciation_cents / 100,
                    recapture.unsigned_abs() / 100
                ),
            }
        }
        AdjustmentType::InstallmentSaleSection312N5 => {
            let acceleration = i128::from(input.full_recognition_year_of_sale_gain_cents)
                - i128::from(input.installment_method_gain_recognized_cents);
            Output {
                severity: Severity::EepFullRecognitionSection312N5InstallmentSale,
                eep_adjustment_cents: acceleration,
                ending_eep_cents: starting + acceleration,
                note: format!(
                    "§ 312(n)(5) INSTALLMENT SALE ADJUSTMENT: E&P computed as if installment \
                     method NOT used — full recognition of gain in year of sale. Taxable-\
                     income installment-method recognition (${}) reduced by full-year \
                     recognition (${}) = ${} ACCELERATION of E&P relative to taxable income. \
                     Same § 312(n) treatment applies to (n)(1) construction-period interest, \
                     (n)(2) intangible drilling, (n)(3) amortization, (n)(4) LIFO, (n)(6) \
                     completed-contract, (n)(7) stock redemptions, (n)(8) foreign-corp PTEP \
                     under § 959 + § 961 + CFC adjustments.",
                    input.installment_method_gain_recognized_cents / 100,
                    input.full_recognition_year_of_sale_gain_cents / 100,
                    acceleration.unsigned_abs() / 100
                ),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Input {
        Input {
            adjustment_type: AdjustmentType::CashDistributionSection312A1,
            stock_dividend_taxability: StockDividendTaxability::NotStockDividend,
            starting_eep_cents: 1_000_000_00,
            cash_or_principal_distributed_cents: 100_000_00,
            property_basis_cents: 30_000_00,
            property_fmv_cents: 80_000_00,
            liability_assumed_or_subject_to_cents: 0,
            accelerated_tax_depreciation_cents: 100_000_00,
            straight_line_ads_depreciation_cents: 60_000_00,
            installment_method_gain_recognized_cents: 50_000_00,
            full_recognition_year_of_sale_gain_cents: 500_000_00,
        }
    }

    #[test]
    fn cash_distribution_decreases_eep() {
        let input = base();
        let output = check(&input);
        assert_eq!(output.severity, Severity::EepDecreaseCashOrObligation);
        // $1M starting - $100K distributed = $900K ending
        assert_eq!(output.eep_adjustment_cents, -100_000_00);
        assert_eq!(output.ending_eep_cents, 900_000_00);
    }

    #[test]
    fn obligation_distribution_decreases_eep() {
        let mut input = base();
        input.adjustment_type = AdjustmentType::ObligationDistributionSection312A2;
        let output = check(&input);
        assert_eq!(output.severity, Severity::EepDecreaseCashOrObligation);
        assert_eq!(output.eep_adjustment_cents, -100_000_00);
    }

    #[test]
    fn property_distribution_at_basis_decreases_eep_by_basis() {
        let mut input = base();
        input.adjustment_type = AdjustmentType::PropertyDistributionAtBasisSection312A3;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::EepDecreaseDistributedPropertyBasis
        );
        // $1M starting - $30K basis = $970K ending
        assert_eq!(output.eep_adjustment_cents, -30_000_00);
    }

    #[test]
    fn appreciated_property_increase_by_gain_then_decrease_by_fmv() {
        let mut input = base();
        input.adjustment_type = AdjustmentType::AppreciatedPropertyDistributionSection312B;
        let output = check(&input);
        // Gain $50K, then decrease by FMV $80K, net -$30K (equal to basis)
        assert_eq!(output.eep_adjustment_cents, -30_000_00);
        assert_eq!(
            output.severity,
            Severity::EepIncreaseAppreciationThenDecreaseFmv
        );
    }

    #[test]
    fn liability_assumption_modifier_increases_eep_back() {
        let mut input = base();
        input.adjustment_type = AdjustmentType::LiabilityAssumptionModifierSection312C;
        input.liability_assumed_or_subject_to_cents = 50_000_00;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::EepDecreaseAdjustedByLiabilityAssumed
        );
        // Modifier adds back $50K
        assert_eq!(output.eep_adjustment_cents, 50_000_00);
    }

    #[test]
    fn non_taxable_stock_dividend_does_not_reduce_eep() {
        let mut input = base();
        input.adjustment_type = AdjustmentType::StockDividendSection312D;
        input.stock_dividend_taxability = StockDividendTaxability::NonTaxableSection305A;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::EepUnchangedNonTaxableStockDividendSection305A
        );
        assert_eq!(output.eep_adjustment_cents, 0);
        assert!(output.note.contains("§ 307"));
    }

    #[test]
    fn taxable_stock_dividend_reduces_eep_by_fmv() {
        let mut input = base();
        input.adjustment_type = AdjustmentType::StockDividendSection312D;
        input.stock_dividend_taxability = StockDividendTaxability::TaxableSection305B;
        let output = check(&input);
        // FMV $80K reduces E&P
        assert_eq!(output.eep_adjustment_cents, -80_000_00);
        assert_eq!(
            output.severity,
            Severity::EepReducedTaxableStockDividendSection305B
        );
    }

    #[test]
    fn ads_straight_line_depreciation_adjustment() {
        let mut input = base();
        input.adjustment_type = AdjustmentType::AdsStraightLineDepreciationSection312K3;
        let output = check(&input);
        // Tax accel $100K - ADS $60K = +$40K E&P
        assert_eq!(output.eep_adjustment_cents, 40_000_00);
        assert_eq!(
            output.severity,
            Severity::EepAdjustedAdsStraightLineSection312K3
        );
        assert!(output.note.contains("§ 168(g)(2)"));
    }

    #[test]
    fn installment_sale_acceleration_to_full_recognition() {
        let mut input = base();
        input.adjustment_type = AdjustmentType::InstallmentSaleSection312N5;
        let output = check(&input);
        // Full recognition $500K - installment $50K = $450K acceleration
        assert_eq!(output.eep_adjustment_cents, 450_000_00);
        assert_eq!(
            output.severity,
            Severity::EepFullRecognitionSection312N5InstallmentSale
        );
        assert!(output.note.contains("§ 959"));
        assert!(output.note.contains("§ 961"));
    }

    #[test]
    fn note_pins_section_301c_distribution_character() {
        let input = base();
        let output = check(&input);
        assert!(output.note.contains("§ 301(c)"));
    }

    #[test]
    fn note_pins_section_311_corporate_recognition() {
        let mut input = base();
        input.adjustment_type = AdjustmentType::AppreciatedPropertyDistributionSection312B;
        let output = check(&input);
        assert!(output.note.contains("§ 311"));
    }

    #[test]
    fn note_pins_section_307_basis_allocation_for_non_taxable_dividend() {
        let mut input = base();
        input.adjustment_type = AdjustmentType::StockDividendSection312D;
        input.stock_dividend_taxability = StockDividendTaxability::NonTaxableSection305A;
        let output = check(&input);
        assert!(output.note.contains("§ 307"));
    }

    #[test]
    fn very_large_eep_no_overflow() {
        let mut input = base();
        input.starting_eep_cents = u64::MAX;
        let output = check(&input);
        // i128 arithmetic prevents overflow
        assert!(output.ending_eep_cents > 0);
    }

    #[test]
    fn zero_starting_eep_no_panic() {
        let mut input = base();
        input.starting_eep_cents = 0;
        let output = check(&input);
        // -$100K with $0 starting → -$100K
        assert_eq!(output.ending_eep_cents, -100_000_00);
    }

    #[test]
    fn stock_dividend_not_stock_dividend_flag_returns_not_applicable() {
        let mut input = base();
        input.adjustment_type = AdjustmentType::StockDividendSection312D;
        input.stock_dividend_taxability = StockDividendTaxability::NotStockDividend;
        let output = check(&input);
        assert_eq!(output.severity, Severity::NotApplicable);
    }

    #[test]
    fn appreciated_property_with_zero_basis_full_gain_offsets_fmv_to_zero_basis() {
        let mut input = base();
        input.adjustment_type = AdjustmentType::AppreciatedPropertyDistributionSection312B;
        input.property_basis_cents = 0;
        let output = check(&input);
        // Gain $80K - FMV $80K = $0 net effect (gain fully offsets FMV reduction)
        assert_eq!(output.eep_adjustment_cents, 0);
    }

    #[test]
    fn note_pins_section_959_961_cfc_ptep_for_312_n() {
        let mut input = base();
        input.adjustment_type = AdjustmentType::InstallmentSaleSection312N5;
        let output = check(&input);
        assert!(output.note.contains("§ 959"));
        assert!(output.note.contains("§ 961"));
        assert!(output.note.contains("PTEP"));
    }
}
