//! IRC § 246A reduction of dividends-received deduction (DRD) for debt-financed portfolio stock.
//!
//! § 246A substitutes a reduced DRD percentage for the base DRD percentage of § 243 (domestic
//! corporation dividends) or § 245(a) (foreign-corporation US-source dividends) when the
//! corporate dividend recipient holds the underlying stock through portfolio indebtedness
//! during the base period. The reduction targets corporate taxpayers borrowing to acquire
//! portfolio stock and claiming both an interest deduction AND a DRD on the same economic
//! return — the section reverse-engineers the DRD downward to neutralize the financing
//! arbitrage.
//!
//! § 246A(a) formula: substituted DRD % = base DRD % × (100% - average indebtedness %).
//! § 246A(b) "portfolio stock" = corporation owns less than 50% of issuer.
//! § 246A(c) "average indebtedness percentage" = avg portfolio indebtedness / avg adjusted
//!    basis of stock during base period.
//! § 246A(d)(3) cap: reduction cannot exceed interest deduction (incl. deductible short-sale
//!    expense) allocable to the dividend.
//! § 246A(e) exceptions: § 243(b) qualifying dividends (affiliated group), SBIC dividends
//!    under Small Business Investment Act of 1958.
//!
//! Coordinates with § 243 (50% / 65% / 100% base DRD), § 245(a) (US-source DRD on foreign
//! corp dividends), § 245A (100% participation DRD — distinct branch not reduced by § 246A),
//! § 246 (general DRD limits incl. holding-period 46/91-day rule), § 265 (interest-expense
//! disallowance for tax-exempt income, parallel logic).
//!
//! Citations (verified per WebSearch 2026-06-02):
//! - law.cornell.edu/uscode/text/26/246A
//! - irc.bloombergtax.com/public/uscode/doc/irc/section_246a
//! - uscode.house.gov/view.xhtml?req=(title:26+section:246A+edition:prelim)
//! - codes.findlaw.com/us/title-26-internal-revenue-code/26-usc-sect-246a/

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OwnershipTier {
    /// Less than 20% ownership — base DRD 50% (post-TCJA, post-OBBBA).
    LessThan20Percent,
    /// 20%-up-to-but-not-including-50% — base DRD 65%.
    TwentyToFiftyPercent,
    /// 50% or more — NOT portfolio stock; § 246A inapplicable.
    FiftyPercentOrMoreNotPortfolioStock,
    /// 80% or more affiliated group — § 243(b) qualifying dividend, § 246A inapplicable.
    AffiliatedGroupSection243BQualifying,
}

/// Underlying DRD section being reduced by § 246A.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnderlyingDrdSection {
    /// § 243 — domestic-corporation dividends to corporate shareholder.
    Section243DomesticDividend,
    /// § 245(a) — US-source portion of foreign-corporation dividend to corporate shareholder.
    Section245aUsSourceForeignCorpDividend,
    /// § 245A — 100% participation DRD on foreign-source dividend; § 246A inapplicable per
    /// Treas. Reg. § 1.245A-3 and statutory structure.
    Section245aOneHundredPercentParticipationNotReduced,
    /// SBIC dividend under Small Business Investment Act of 1958 — exempted by § 246A(e).
    SbicDividendExempt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    Section245aFullParticipationDrdPreservedNoReduction,
    Section243bAffiliatedQualifyingDividendPreservedNoReduction,
    SbicExemptionPreservesFullDrd,
    Section246aReductionApplied,
    Section246aReductionCappedByInterestDeductionAllocable,
    InvalidInputAverageIndebtednessExceedsOneHundred,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub ownership_tier: OwnershipTier,
    pub underlying_drd_section: UnderlyingDrdSection,
    pub dividend_received_cents: u64,
    pub average_indebtedness_percent: u32,
    pub interest_deduction_allocable_cents: u64,
}

pub type Section246aDebtFinancedPortfolioStockInput = Input;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub severity: Severity,
    pub base_drd_percent: u32,
    pub substituted_drd_percent: u32,
    pub allowed_deduction_cents: u64,
    pub reduction_from_base_cents: u64,
    pub note: String,
}

pub type Section246aDebtFinancedPortfolioStockOutput = Output;

const BASE_DRD_LESS_THAN_20_PERCENT: u32 = 50;
const BASE_DRD_20_TO_50_PERCENT: u32 = 65;

#[must_use]
pub fn check(input: &Input) -> Output {
    if input.average_indebtedness_percent > 100 {
        return Output {
            severity: Severity::InvalidInputAverageIndebtednessExceedsOneHundred,
            base_drd_percent: 0,
            substituted_drd_percent: 0,
            allowed_deduction_cents: 0,
            reduction_from_base_cents: 0,
            note: format!(
                "Invalid input: average indebtedness percentage ({}) exceeds 100%. § 246A(c) \
                 formula requires the average to be calculated as portfolio indebtedness divided \
                 by adjusted basis during the base period; ratio cannot exceed 100% of basis.",
                input.average_indebtedness_percent
            ),
        };
    }

    if matches!(input.underlying_drd_section, UnderlyingDrdSection::SbicDividendExempt) {
        return Output {
            severity: Severity::SbicExemptionPreservesFullDrd,
            base_drd_percent: 100,
            substituted_drd_percent: 100,
            allowed_deduction_cents: input.dividend_received_cents,
            reduction_from_base_cents: 0,
            note: "§ 246A(e) exception applies. SBIC dividends under Small Business Investment \
                   Act of 1958 are exempted from the § 246A debt-financed-stock reduction; full \
                   DRD preserved. Verify SBIC license status with SBA before relying."
                .to_string(),
        };
    }

    if matches!(
        input.underlying_drd_section,
        UnderlyingDrdSection::Section245aOneHundredPercentParticipationNotReduced
    ) {
        return Output {
            severity: Severity::Section245aFullParticipationDrdPreservedNoReduction,
            base_drd_percent: 100,
            substituted_drd_percent: 100,
            allowed_deduction_cents: input.dividend_received_cents,
            reduction_from_base_cents: 0,
            note: "§ 245A 100% participation DRD on foreign-source dividend is NOT subject to \
                   § 246A reduction by statutory structure and Treas. Reg. § 1.245A-3. \
                   Coordination concerns: § 245A(d) FTC disallowance for foreign tax on \
                   DRD-eligible dividend; § 246(c) one-year holding-period requirement; \
                   hybrid-dividend disqualification per § 245A(e)."
                .to_string(),
        };
    }

    if matches!(
        input.ownership_tier,
        OwnershipTier::AffiliatedGroupSection243BQualifying
    ) {
        return Output {
            severity: Severity::Section243bAffiliatedQualifyingDividendPreservedNoReduction,
            base_drd_percent: 100,
            substituted_drd_percent: 100,
            allowed_deduction_cents: input.dividend_received_cents,
            reduction_from_base_cents: 0,
            note: "§ 243(b) qualifying dividend from affiliated-group member (80% or more \
                   ownership) is exempted from § 246A reduction. Full 100% DRD preserved \
                   regardless of debt-financing on the underlying stock. Affiliated group \
                   election under § 1504 satisfied; consolidated-return DRD treatment may also \
                   apply."
                .to_string(),
        };
    }

    if matches!(
        input.ownership_tier,
        OwnershipTier::FiftyPercentOrMoreNotPortfolioStock
    ) {
        return Output {
            severity: Severity::NotApplicable,
            base_drd_percent: 65,
            substituted_drd_percent: 65,
            allowed_deduction_cents: input
                .dividend_received_cents
                .saturating_mul(65)
                .saturating_div(100),
            reduction_from_base_cents: 0,
            note: "Stock is not portfolio stock under § 246A(b): corporate shareholder owns 50% \
                   or more of issuer. § 246A inapplicable. Base 65% DRD under § 243(c) preserved \
                   subject to other limits (§ 246 holding period, § 246(b) taxable income cap, \
                   § 246A inapplicable)."
                .to_string(),
        };
    }

    let base_drd_percent = match input.ownership_tier {
        OwnershipTier::LessThan20Percent => BASE_DRD_LESS_THAN_20_PERCENT,
        OwnershipTier::TwentyToFiftyPercent => BASE_DRD_20_TO_50_PERCENT,
        _ => unreachable!("higher-ownership tiers handled above"),
    };

    let substituted_drd_percent = u128::from(base_drd_percent)
        .saturating_mul(100u128 - u128::from(input.average_indebtedness_percent))
        .saturating_div(100);
    let substituted_drd_percent_u32 = u32::try_from(substituted_drd_percent).unwrap_or(0);

    let base_deduction = u128::from(input.dividend_received_cents)
        .saturating_mul(u128::from(base_drd_percent))
        .saturating_div(100);
    let substituted_deduction =
        u128::from(input.dividend_received_cents)
            .saturating_mul(substituted_drd_percent)
            .saturating_div(100);
    let raw_reduction = base_deduction.saturating_sub(substituted_deduction);

    let capped_reduction = raw_reduction.min(u128::from(input.interest_deduction_allocable_cents));
    let final_deduction = base_deduction.saturating_sub(capped_reduction);

    let final_deduction_u64 = u64::try_from(final_deduction).unwrap_or(u64::MAX);
    let capped_reduction_u64 = u64::try_from(capped_reduction).unwrap_or(u64::MAX);

    let severity = if capped_reduction < raw_reduction {
        Severity::Section246aReductionCappedByInterestDeductionAllocable
    } else {
        Severity::Section246aReductionApplied
    };

    let cap_note = if capped_reduction < raw_reduction {
        format!(
            " § 246A(d)(3) cap binds: raw reduction would be ${} but is capped at allocable \
             interest deduction ${}.",
            u64::try_from(raw_reduction).unwrap_or(u64::MAX) / 100,
            input.interest_deduction_allocable_cents / 100
        )
    } else {
        String::new()
    };

    Output {
        severity,
        base_drd_percent,
        substituted_drd_percent: substituted_drd_percent_u32,
        allowed_deduction_cents: final_deduction_u64,
        reduction_from_base_cents: capped_reduction_u64,
        note: format!(
            "§ 246A reduction applied. Base DRD percentage {}% × (100% - {}% average \
             indebtedness) = substituted {}% DRD. Dividend ${} × {}% base = ${} base deduction; \
             reduction ${} applied = ${} final allowed deduction.{} Coordinates with § 243 \
             holding-period rule (§ 246(c) — 46/91-day window) and § 246(b) taxable-income cap.",
            base_drd_percent,
            input.average_indebtedness_percent,
            substituted_drd_percent_u32,
            input.dividend_received_cents / 100,
            base_drd_percent,
            u64::try_from(base_deduction).unwrap_or(u64::MAX) / 100,
            capped_reduction_u64 / 100,
            final_deduction_u64 / 100,
            cap_note
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Input {
        Input {
            ownership_tier: OwnershipTier::LessThan20Percent,
            underlying_drd_section: UnderlyingDrdSection::Section243DomesticDividend,
            dividend_received_cents: 1_000_000_00,
            average_indebtedness_percent: 60,
            interest_deduction_allocable_cents: 500_000_00,
        }
    }

    #[test]
    fn invalid_input_average_indebtedness_exceeds_100_returns_invalid_input() {
        let mut input = base();
        input.average_indebtedness_percent = 101;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::InvalidInputAverageIndebtednessExceedsOneHundred
        );
        assert_eq!(output.allowed_deduction_cents, 0);
    }

    #[test]
    fn sbic_dividend_exempt_preserves_full_drd_via_246a_e() {
        let mut input = base();
        input.underlying_drd_section = UnderlyingDrdSection::SbicDividendExempt;
        let output = check(&input);
        assert_eq!(output.severity, Severity::SbicExemptionPreservesFullDrd);
        assert_eq!(output.allowed_deduction_cents, 1_000_000_00);
        assert!(output.note.contains("§ 246A(e)"));
        assert!(output.note.contains("Small Business Investment Act of 1958"));
    }

    #[test]
    fn section_245a_full_participation_drd_not_reduced_by_246a() {
        let mut input = base();
        input.underlying_drd_section =
            UnderlyingDrdSection::Section245aOneHundredPercentParticipationNotReduced;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section245aFullParticipationDrdPreservedNoReduction
        );
        assert_eq!(output.allowed_deduction_cents, 1_000_000_00);
        assert!(output.note.contains("§ 245A"));
        assert!(output.note.contains("Treas. Reg. § 1.245A-3"));
    }

    #[test]
    fn affiliated_group_243b_qualifying_dividend_preserves_full_drd() {
        let mut input = base();
        input.ownership_tier = OwnershipTier::AffiliatedGroupSection243BQualifying;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section243bAffiliatedQualifyingDividendPreservedNoReduction
        );
        assert_eq!(output.allowed_deduction_cents, 1_000_000_00);
        assert!(output.note.contains("§ 243(b)"));
        assert!(output.note.contains("§ 1504"));
    }

    #[test]
    fn fifty_percent_or_more_ownership_not_portfolio_stock_returns_not_applicable() {
        let mut input = base();
        input.ownership_tier = OwnershipTier::FiftyPercentOrMoreNotPortfolioStock;
        let output = check(&input);
        assert_eq!(output.severity, Severity::NotApplicable);
        // 65% of $1M = $650K
        assert_eq!(output.allowed_deduction_cents, 650_000_00);
        assert!(output.note.contains("§ 246A(b)"));
    }

    #[test]
    fn standard_reduction_less_than_20_pct_owner_60_pct_indebted() {
        let input = base();
        let output = check(&input);
        // 50% × (100% - 60%) = 50% × 40% = 20% substituted DRD
        // Base deduction = $1M × 50% = $500K
        // Substituted deduction = $1M × 20% = $200K
        // Raw reduction = $300K, capped at $500K allocable interest (no cap binding)
        assert_eq!(output.severity, Severity::Section246aReductionApplied);
        assert_eq!(output.base_drd_percent, 50);
        assert_eq!(output.substituted_drd_percent, 20);
        assert_eq!(output.allowed_deduction_cents, 200_000_00);
        assert_eq!(output.reduction_from_base_cents, 300_000_00);
    }

    #[test]
    fn standard_reduction_20_to_50_pct_owner_40_pct_indebted() {
        let mut input = base();
        input.ownership_tier = OwnershipTier::TwentyToFiftyPercent;
        input.average_indebtedness_percent = 40;
        let output = check(&input);
        // 65% × (100% - 40%) = 65% × 60% = 39% substituted DRD
        // Base = $650K; Substituted = $390K; Reduction = $260K
        assert_eq!(output.severity, Severity::Section246aReductionApplied);
        assert_eq!(output.base_drd_percent, 65);
        assert_eq!(output.substituted_drd_percent, 39);
        assert_eq!(output.allowed_deduction_cents, 390_000_00);
    }

    #[test]
    fn cap_binds_when_interest_deduction_less_than_raw_reduction() {
        let mut input = base();
        input.interest_deduction_allocable_cents = 100_000_00;
        let output = check(&input);
        // Raw reduction $300K > interest deduction $100K → cap binds at $100K
        // Final deduction = $500K base - $100K cap = $400K
        assert_eq!(
            output.severity,
            Severity::Section246aReductionCappedByInterestDeductionAllocable
        );
        assert_eq!(output.allowed_deduction_cents, 400_000_00);
        assert_eq!(output.reduction_from_base_cents, 100_000_00);
        assert!(output.note.contains("§ 246A(d)(3) cap binds"));
    }

    #[test]
    fn zero_indebtedness_no_reduction() {
        let mut input = base();
        input.average_indebtedness_percent = 0;
        let output = check(&input);
        // 50% × 100% = 50% — no reduction
        assert_eq!(output.substituted_drd_percent, 50);
        assert_eq!(output.allowed_deduction_cents, 500_000_00);
        assert_eq!(output.reduction_from_base_cents, 0);
    }

    #[test]
    fn one_hundred_percent_indebtedness_full_reduction_to_zero_drd() {
        let mut input = base();
        input.average_indebtedness_percent = 100;
        input.interest_deduction_allocable_cents = u64::MAX;
        let output = check(&input);
        // 50% × 0% = 0% — full reduction
        assert_eq!(output.substituted_drd_percent, 0);
        assert_eq!(output.allowed_deduction_cents, 0);
    }

    #[test]
    fn base_drd_less_than_20_pct_constant_pins_50() {
        assert_eq!(BASE_DRD_LESS_THAN_20_PERCENT, 50);
    }

    #[test]
    fn base_drd_20_to_50_pct_constant_pins_65() {
        assert_eq!(BASE_DRD_20_TO_50_PERCENT, 65);
    }

    #[test]
    fn very_large_dividend_no_overflow_in_u128_intermediate() {
        let mut input = base();
        input.dividend_received_cents = u64::MAX;
        input.interest_deduction_allocable_cents = u64::MAX;
        input.average_indebtedness_percent = 50;
        let output = check(&input);
        assert_eq!(output.severity, Severity::Section246aReductionApplied);
        // u128 intermediate prevents overflow on u64::MAX × 50
        assert!(output.allowed_deduction_cents > 0);
    }

    #[test]
    fn zero_dividend_zero_deduction_no_panic() {
        let mut input = base();
        input.dividend_received_cents = 0;
        let output = check(&input);
        assert_eq!(output.allowed_deduction_cents, 0);
        assert_eq!(output.reduction_from_base_cents, 0);
    }

    #[test]
    fn note_pins_246a_a_formula_substitution() {
        let input = base();
        let output = check(&input);
        assert!(output.note.contains("Base DRD percentage"));
        assert!(output.note.contains("average indebtedness"));
        assert!(output.note.contains("substituted"));
    }

    #[test]
    fn note_pins_246a_d_3_cap_when_cap_binds() {
        let mut input = base();
        input.interest_deduction_allocable_cents = 50_000_00;
        let output = check(&input);
        assert!(output.note.contains("§ 246A(d)(3) cap binds"));
    }

    #[test]
    fn note_pins_246_c_holding_period_46_91_day_window() {
        let input = base();
        let output = check(&input);
        assert!(output.note.contains("§ 246(c)"));
        assert!(output.note.contains("46/91-day"));
    }

    #[test]
    fn note_pins_246_b_taxable_income_cap() {
        let input = base();
        let output = check(&input);
        assert!(output.note.contains("§ 246(b)"));
    }

    #[test]
    fn note_pins_245a_d_ftc_disallowance_for_245a_branch() {
        let mut input = base();
        input.underlying_drd_section =
            UnderlyingDrdSection::Section245aOneHundredPercentParticipationNotReduced;
        let output = check(&input);
        assert!(output.note.contains("§ 245A(d)"));
        assert!(output.note.contains("hybrid-dividend"));
    }

    #[test]
    fn full_pct_affiliated_owner_returns_full_drd_via_affiliated_branch_not_via_246a_reduction() {
        let mut input = base();
        input.ownership_tier = OwnershipTier::AffiliatedGroupSection243BQualifying;
        input.average_indebtedness_percent = 80; // Even with heavy debt, affiliated preserves
        let output = check(&input);
        assert_eq!(output.allowed_deduction_cents, 1_000_000_00);
    }

    #[test]
    fn substituted_drd_pct_rounding_truncates_not_rounds_up() {
        let mut input = base();
        input.average_indebtedness_percent = 33;
        let output = check(&input);
        // 50 × (100 - 33) / 100 = 50 × 67 / 100 = 33.5 → truncates to 33
        assert_eq!(output.substituted_drd_percent, 33);
    }
}
