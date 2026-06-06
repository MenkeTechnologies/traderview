//! IRC § 265 expenses and interest relating to tax-exempt income.
//!
//! § 265 disallows deductions for expenses (§ 265(a)(1)) and interest (§ 265(a)(2))
//! allocable to wholly tax-exempt income. The provision prevents double-dipping when
//! a taxpayer borrows to acquire or carry tax-exempt obligations (typically municipal
//! bonds) and claims an interest deduction that would offset taxable income while the
//! bond yield remains tax-exempt.
//!
//! § 265(a)(2) tracing test (Wisconsin Cheeseman v. United States, 7th Cir. 1968):
//! disallowance applies when debt is "incurred or continued" to purchase or carry
//! tax-exempt obligations. Direct-tracing standard; mere co-existence of debt and
//! tax-exempt holdings is insufficient.
//!
//! Three calculation regimes operate in parallel:
//!
//! 1. INDIVIDUALS (Rev. Proc. 72-18): § 265(a)(2) applies via direct-tracing test.
//!    De minimis exception: no disallowance when tax-exempt holdings are insubstantial
//!    relative to investment portfolio.
//!
//! 2. NON-FINANCIAL CORPORATIONS (Rev. Proc. 72-18 + Rev. Proc. 87-53): same
//!    direct-tracing standard; safe harbors for short-term debt and operating-business
//!    debt.
//!
//! 3. FINANCIAL INSTITUTIONS / BANKS (§ 265(b)): pro-rata disallowance based on ratio
//!    of average adjusted bases of tax-exempt obligations acquired after August 7,
//!    1986 to average adjusted bases of all taxpayer assets. NO tracing test —
//!    automatic mechanical formula.
//!
//! Bank-qualified bonds under § 265(b)(3) (qualified-tax-exempt obligations):
//! issuer must reasonably anticipate issuing no more than $10,000,000 of tax-exempt
//! obligations during the calendar year. For bank-qualified bonds, § 291(e) imposes
//! only 20% interest-expense disallowance (vs 100% for non-bank-qualified).
//!
//! Citations (verified per WebSearch 2026-06-02):
//! - law.cornell.edu/uscode/text/26/265
//! - irs.gov/pub/irs-tege/13%20Phase%20I%20Lesson%2013%20-%20Bank%20Qualified%20Bonds%20-%20Section%20265.pdf
//! - law.cornell.edu/cfr/text/26/1.265-2
//! - debtguide-api.treasurer.ca.gov/guide-pages/chapter-4-federal-and-state-tax-law-requirements/4-11-bank-qualified-bonds
//! - taxnotes.com/research/federal/irs-guidance/revenue-rulings/service-issues-guidelines-on-disallowance-of-financial-institutions-interest-expense/dgjy

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaxpayerType {
    Individual,
    NonFinancialCorporation,
    /// Bank or other financial institution subject to § 265(b) pro-rata disallowance.
    BankOrFinancialInstitution,
    DealerInTaxExemptObligations,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BondClassification {
    /// Bank-qualified ($10M issuer cap) — 20% disallowance per § 291(e) for banks.
    BankQualifiedQualifiedTaxExemptObligation,
    /// Non-bank-qualified — 100% disallowance for banks under § 265(b).
    NonBankQualified,
    /// Pre-Aug 7 1986 obligation — grandfathered, NOT subject to § 265(b) pro-rata.
    PreAugust7_1986Grandfathered,
}

/// Debt-tracing status under Wisconsin Cheeseman direct-tracing test for individuals
/// and non-financial corporations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DebtTracingStatus {
    /// Debt directly traceable to acquisition or carrying of tax-exempt obligations.
    DirectlyTraceableToTaxExemptAcquisition,
    /// Debt for unrelated business or personal purposes; no § 265(a)(2) disallowance.
    UnrelatedNonTracingNoDisallowance,
    /// Not applicable (bank pro-rata regime applies instead).
    NotApplicableBankProRataRegime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    NoTaxExemptIncomeNoDisallowance,
    UnrelatedDebtNoSectionTwoSixFiveADisallowance,
    IndividualDirectTracedFullDisallowance,
    NonFinancialCorpDirectTracedFullDisallowance,
    BankNonBankQualifiedHundredPctProRataDisallowance,
    BankQualifiedTwentyPctSection291EDisallowance,
    BankPreAugust1986GrandfatheredNoDisallowance,
    DealerInTaxExemptObligationsSafeHarborNoDisallowance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub taxpayer_type: TaxpayerType,
    pub bond_classification: BondClassification,
    pub debt_tracing_status: DebtTracingStatus,
    pub interest_expense_cents: u64,
    pub tax_exempt_interest_received_cents: u64,
    pub average_basis_tax_exempt_obligations_cents: u64,
    pub average_basis_total_assets_cents: u64,
}

pub type Section265TaxExemptInterestDisallowanceInput = Input;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub severity: Severity,
    pub disallowed_interest_expense_cents: u64,
    pub allowed_interest_expense_cents: u64,
    pub disallowance_percentage_basis_points: u32,
    pub note: String,
}

pub type Section265TaxExemptInterestDisallowanceOutput = Output;
pub type Section265TaxExemptInterestDisallowanceResult = Output;

const BANK_NON_BANK_QUALIFIED_DISALLOWANCE_BPS: u32 = 10_000;
const BANK_QUALIFIED_291E_DISALLOWANCE_BPS: u32 = 2_000;
#[allow(dead_code)]
const SECTION_265B3_QUALIFIED_ISSUER_CAP_CENTS: u64 = 1_000_000_000;
const SECTION_265B_EFFECTIVE_DATE_LABEL: &str = "Aug 7 1986";

#[must_use]
pub fn check(input: &Input) -> Output {
    if input.tax_exempt_interest_received_cents == 0 {
        return Output {
            severity: Severity::NoTaxExemptIncomeNoDisallowance,
            disallowed_interest_expense_cents: 0,
            allowed_interest_expense_cents: input.interest_expense_cents,
            disallowance_percentage_basis_points: 0,
            note: "No tax-exempt interest reported — § 265 disallowance not triggered. \
                   § 265 requires tax-exempt income (municipal bond interest, exempt-interest \
                   dividends from RIC under § 852(b)(5), etc.) to generate the disallowance."
                .to_string(),
        };
    }

    if matches!(
        input.taxpayer_type,
        TaxpayerType::DealerInTaxExemptObligations
    ) {
        return Output {
            severity: Severity::DealerInTaxExemptObligationsSafeHarborNoDisallowance,
            disallowed_interest_expense_cents: 0,
            allowed_interest_expense_cents: input.interest_expense_cents,
            disallowance_percentage_basis_points: 0,
            note: "Dealer-in-tax-exempt-obligations safe harbor (Rev. Proc. 72-18 § 7) applies. \
                   Dealers carrying inventory of tax-exempt obligations are EXEMPT from § 265 \
                   disallowance for interest expense on debt incurred to carry inventory. \
                   Verify dealer registration under SEC and that interest is on inventory-line \
                   debt (not general operating debt)."
                .to_string(),
        };
    }

    if matches!(
        input.taxpayer_type,
        TaxpayerType::BankOrFinancialInstitution
    ) {
        return apply_section_265b_bank_pro_rata(input);
    }

    if matches!(
        input.debt_tracing_status,
        DebtTracingStatus::UnrelatedNonTracingNoDisallowance
    ) {
        return Output {
            severity: Severity::UnrelatedDebtNoSectionTwoSixFiveADisallowance,
            disallowed_interest_expense_cents: 0,
            allowed_interest_expense_cents: input.interest_expense_cents,
            disallowance_percentage_basis_points: 0,
            note: "Wisconsin Cheeseman direct-tracing test (Wisconsin Cheeseman v. United \
                   States, 7th Cir. 1968) NOT satisfied — debt is not traceable to acquisition \
                   or carrying of tax-exempt obligations. § 265(a)(2) disallowance does not \
                   apply. Mere co-existence of debt and tax-exempt holdings is insufficient; \
                   the IRS bears the burden of proving the tracing connection."
                .to_string(),
        };
    }

    match input.taxpayer_type {
        TaxpayerType::Individual => Output {
            severity: Severity::IndividualDirectTracedFullDisallowance,
            disallowed_interest_expense_cents: input.interest_expense_cents,
            allowed_interest_expense_cents: 0,
            disallowance_percentage_basis_points: 10_000,
            note: format!(
                "§ 265(a)(2) individual full disallowance applies. Wisconsin Cheeseman \
                 direct-tracing test satisfied: debt incurred or continued to purchase or \
                 carry tax-exempt obligations. Interest expense ${} disallowed in full. \
                 Rev. Proc. 72-18 applies. De minimis safe harbor inapplicable on direct \
                 trace. Consider § 265 elective allocation method if multiple debt sources \
                 not all traceable.",
                input.interest_expense_cents / 100
            ),
        },
        TaxpayerType::NonFinancialCorporation => Output {
            severity: Severity::NonFinancialCorpDirectTracedFullDisallowance,
            disallowed_interest_expense_cents: input.interest_expense_cents,
            allowed_interest_expense_cents: 0,
            disallowance_percentage_basis_points: 10_000,
            note: format!(
                "§ 265(a)(2) non-financial corporation full disallowance applies. Direct- \
                 tracing test satisfied per Rev. Proc. 72-18 + Rev. Proc. 87-53. Interest \
                 expense ${} disallowed in full. Operating-business safe harbor inapplicable \
                 on direct trace. Form 1120 Schedule M-1 / M-3 book-tax adjustment required. \
                 Coordinates with § 163(j) interest-deduction limitation (excess business \
                 interest expense) and § 246A debt-financed-portfolio-stock DRD reduction.",
                input.interest_expense_cents / 100
            ),
        },
        _ => unreachable!("bank + dealer branches handled above"),
    }
}

fn apply_section_265b_bank_pro_rata(input: &Input) -> Output {
    if matches!(
        input.bond_classification,
        BondClassification::PreAugust7_1986Grandfathered
    ) {
        return Output {
            severity: Severity::BankPreAugust1986GrandfatheredNoDisallowance,
            disallowed_interest_expense_cents: 0,
            allowed_interest_expense_cents: input.interest_expense_cents,
            disallowance_percentage_basis_points: 0,
            note: format!(
                "Pre-{SECTION_265B_EFFECTIVE_DATE_LABEL} grandfathered tax-exempt obligation. \
                 § 265(b) pro-rata disallowance applies ONLY to obligations acquired after \
                 Aug 7 1986 (Tax Reform Act of 1986 effective date). Bank may deduct full \
                 interest expense ${} on debt allocable to grandfathered obligations.",
                input.interest_expense_cents / 100
            ),
        };
    }

    let disallowance_bps = match input.bond_classification {
        BondClassification::BankQualifiedQualifiedTaxExemptObligation => {
            BANK_QUALIFIED_291E_DISALLOWANCE_BPS
        }
        BondClassification::NonBankQualified => BANK_NON_BANK_QUALIFIED_DISALLOWANCE_BPS,
        BondClassification::PreAugust7_1986Grandfathered => unreachable!(),
    };

    // For bank-qualified: § 291(e) imposes 20% disallowance regardless of pro-rata ratio.
    // For non-bank-qualified: § 265(b) imposes 100% pro-rata disallowance based on
    // average basis ratio. We apply pro-rata to the disallowance amount.
    let pro_rata_ratio_bps: u32 = if input.average_basis_total_assets_cents == 0 {
        0
    } else {
        let ratio = u128::from(input.average_basis_tax_exempt_obligations_cents)
            .saturating_mul(10_000)
            .saturating_div(u128::from(input.average_basis_total_assets_cents));
        u32::try_from(ratio.min(10_000)).unwrap_or(0)
    };

    let raw_disallowed = u128::from(input.interest_expense_cents)
        .saturating_mul(u128::from(disallowance_bps))
        .saturating_mul(u128::from(pro_rata_ratio_bps))
        .saturating_div(10_000)
        .saturating_div(10_000);
    let raw_disallowed_u64 = u64::try_from(raw_disallowed).unwrap_or(u64::MAX);
    let allowed = input
        .interest_expense_cents
        .saturating_sub(raw_disallowed_u64);

    let effective_bps = u32::try_from(
        u128::from(disallowance_bps)
            .saturating_mul(u128::from(pro_rata_ratio_bps))
            .saturating_div(10_000),
    )
    .unwrap_or(0);

    match input.bond_classification {
        BondClassification::BankQualifiedQualifiedTaxExemptObligation => Output {
            severity: Severity::BankQualifiedTwentyPctSection291EDisallowance,
            disallowed_interest_expense_cents: raw_disallowed_u64,
            allowed_interest_expense_cents: allowed,
            disallowance_percentage_basis_points: effective_bps,
            note: format!(
                "Bank-qualified (qualified tax-exempt) obligation under § 265(b)(3). § 291(e) \
                 reduced 20% disallowance applies (vs 100% for non-bank-qualified). Issuer \
                 must reasonably anticipate issuing no more than $10,000,000 of tax-exempt \
                 obligations during the calendar year. 20% × pro-rata ratio ({} bps) = {} bps \
                 effective disallowance. Interest expense ${}: ${} disallowed, ${} allowed.",
                pro_rata_ratio_bps,
                effective_bps,
                input.interest_expense_cents / 100,
                raw_disallowed_u64 / 100,
                allowed / 100
            ),
        },
        BondClassification::NonBankQualified => Output {
            severity: Severity::BankNonBankQualifiedHundredPctProRataDisallowance,
            disallowed_interest_expense_cents: raw_disallowed_u64,
            allowed_interest_expense_cents: allowed,
            disallowance_percentage_basis_points: effective_bps,
            note: format!(
                "Non-bank-qualified obligation. § 265(b) 100% pro-rata disallowance: 100% × \
                 (avg basis tax-exempt / avg basis total assets) = {} bps effective \
                 disallowance. Interest expense ${}: ${} disallowed, ${} allowed. No \
                 tracing test for banks — mechanical formula applies regardless of debt \
                 source. § 265(b) effective for obligations acquired after \
                 {SECTION_265B_EFFECTIVE_DATE_LABEL} per Tax Reform Act of 1986.",
                effective_bps,
                input.interest_expense_cents / 100,
                raw_disallowed_u64 / 100,
                allowed / 100
            ),
        },
        BondClassification::PreAugust7_1986Grandfathered => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_individual() -> Input {
        Input {
            taxpayer_type: TaxpayerType::Individual,
            bond_classification: BondClassification::NonBankQualified,
            debt_tracing_status: DebtTracingStatus::DirectlyTraceableToTaxExemptAcquisition,
            interest_expense_cents: 50_000_00,
            tax_exempt_interest_received_cents: 100_000_00,
            average_basis_tax_exempt_obligations_cents: 1_000_000_00,
            average_basis_total_assets_cents: 10_000_000_00,
        }
    }

    fn base_bank() -> Input {
        Input {
            taxpayer_type: TaxpayerType::BankOrFinancialInstitution,
            bond_classification: BondClassification::NonBankQualified,
            debt_tracing_status: DebtTracingStatus::NotApplicableBankProRataRegime,
            interest_expense_cents: 1_000_000_00,
            tax_exempt_interest_received_cents: 500_000_00,
            average_basis_tax_exempt_obligations_cents: 50_000_000_00,
            average_basis_total_assets_cents: 1_000_000_000_00,
        }
    }

    #[test]
    fn no_tax_exempt_income_no_disallowance() {
        let mut input = base_individual();
        input.tax_exempt_interest_received_cents = 0;
        let output = check(&input);
        assert_eq!(output.severity, Severity::NoTaxExemptIncomeNoDisallowance);
        assert_eq!(output.disallowed_interest_expense_cents, 0);
        assert_eq!(output.allowed_interest_expense_cents, 50_000_00);
    }

    #[test]
    fn dealer_in_tax_exempt_obligations_safe_harbor_no_disallowance() {
        let mut input = base_individual();
        input.taxpayer_type = TaxpayerType::DealerInTaxExemptObligations;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::DealerInTaxExemptObligationsSafeHarborNoDisallowance
        );
        assert!(output.note.contains("Rev. Proc. 72-18 § 7"));
        assert!(output.note.contains("inventory"));
    }

    #[test]
    fn individual_direct_traced_full_disallowance() {
        let input = base_individual();
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::IndividualDirectTracedFullDisallowance
        );
        assert_eq!(output.disallowed_interest_expense_cents, 50_000_00);
        assert_eq!(output.allowed_interest_expense_cents, 0);
        assert_eq!(output.disallowance_percentage_basis_points, 10_000);
        assert!(output.note.contains("Wisconsin Cheeseman"));
        assert!(output.note.contains("Rev. Proc. 72-18"));
    }

    #[test]
    fn individual_untraced_debt_no_disallowance() {
        let mut input = base_individual();
        input.debt_tracing_status = DebtTracingStatus::UnrelatedNonTracingNoDisallowance;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::UnrelatedDebtNoSectionTwoSixFiveADisallowance
        );
        assert_eq!(output.disallowed_interest_expense_cents, 0);
        assert!(output.note.contains("Wisconsin Cheeseman v. United States"));
        assert!(output.note.contains("7th Cir. 1968"));
    }

    #[test]
    fn non_financial_corporation_traced_full_disallowance() {
        let mut input = base_individual();
        input.taxpayer_type = TaxpayerType::NonFinancialCorporation;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::NonFinancialCorpDirectTracedFullDisallowance
        );
        assert_eq!(output.disallowed_interest_expense_cents, 50_000_00);
        assert!(output.note.contains("Rev. Proc. 87-53"));
        assert!(output.note.contains("§ 163(j)"));
        assert!(output.note.contains("§ 246A"));
    }

    #[test]
    fn bank_pre_1986_grandfathered_no_disallowance() {
        let mut input = base_bank();
        input.bond_classification = BondClassification::PreAugust7_1986Grandfathered;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::BankPreAugust1986GrandfatheredNoDisallowance
        );
        assert_eq!(output.disallowed_interest_expense_cents, 0);
        assert_eq!(output.allowed_interest_expense_cents, 1_000_000_00);
        assert!(output.note.contains("Aug 7 1986"));
        assert!(output.note.contains("Tax Reform Act of 1986"));
    }

    #[test]
    fn bank_non_bank_qualified_pro_rata_disallowance() {
        let input = base_bank();
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::BankNonBankQualifiedHundredPctProRataDisallowance
        );
        // Pro-rata ratio: $50M / $1B = 5% = 500 bps
        // 100% × 500 bps = 500 bps effective disallowance
        // $1M × 5% = $50,000 disallowed
        assert_eq!(output.disallowance_percentage_basis_points, 500);
        assert_eq!(output.disallowed_interest_expense_cents, 50_000_00);
        assert_eq!(output.allowed_interest_expense_cents, 950_000_00);
        assert!(output.note.contains("§ 265(b)"));
        assert!(output.note.contains("No tracing test for banks"));
    }

    #[test]
    fn bank_qualified_291e_twenty_pct_disallowance() {
        let mut input = base_bank();
        input.bond_classification = BondClassification::BankQualifiedQualifiedTaxExemptObligation;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::BankQualifiedTwentyPctSection291EDisallowance
        );
        // 20% × 500 bps pro-rata = 100 bps effective disallowance
        // $1M × 1% = $10,000 disallowed
        assert_eq!(output.disallowance_percentage_basis_points, 100);
        assert_eq!(output.disallowed_interest_expense_cents, 10_000_00);
        assert!(output.note.contains("§ 265(b)(3)"));
        assert!(output.note.contains("§ 291(e)"));
        assert!(output.note.contains("$10,000,000"));
    }

    #[test]
    fn bank_zero_total_assets_no_disallowance_safe_division() {
        let mut input = base_bank();
        input.average_basis_total_assets_cents = 0;
        let output = check(&input);
        // Division-by-zero defense: pro-rata ratio defaults to 0
        assert_eq!(output.disallowed_interest_expense_cents, 0);
    }

    #[test]
    fn bank_hundred_pct_tax_exempt_total_assets_full_disallowance() {
        let mut input = base_bank();
        input.average_basis_tax_exempt_obligations_cents = 1_000_000_000_00;
        input.average_basis_total_assets_cents = 1_000_000_000_00;
        let output = check(&input);
        // Pro-rata = 100% (10,000 bps); 100% × 100% = 100% disallowance
        assert_eq!(output.disallowance_percentage_basis_points, 10_000);
        assert_eq!(output.disallowed_interest_expense_cents, 1_000_000_00);
        assert_eq!(output.allowed_interest_expense_cents, 0);
    }

    #[test]
    fn bank_non_bank_qualified_disallowance_constant_pins_100_pct() {
        assert_eq!(BANK_NON_BANK_QUALIFIED_DISALLOWANCE_BPS, 10_000);
    }

    #[test]
    fn bank_qualified_291e_disallowance_constant_pins_20_pct() {
        assert_eq!(BANK_QUALIFIED_291E_DISALLOWANCE_BPS, 2_000);
    }

    #[test]
    fn section_265b3_qualified_issuer_cap_constant_pins_10m() {
        assert_eq!(SECTION_265B3_QUALIFIED_ISSUER_CAP_CENTS, 1_000_000_000);
    }

    #[test]
    fn very_large_interest_expense_no_overflow() {
        let mut input = base_bank();
        input.interest_expense_cents = u64::MAX / 2;
        let output = check(&input);
        // saturating arithmetic prevents overflow
        assert!(output.allowed_interest_expense_cents > 0);
    }

    #[test]
    fn zero_interest_expense_zero_disallowance() {
        let mut input = base_individual();
        input.interest_expense_cents = 0;
        let output = check(&input);
        assert_eq!(output.disallowed_interest_expense_cents, 0);
        assert_eq!(output.allowed_interest_expense_cents, 0);
    }

    #[test]
    fn note_pins_section_265a2_for_individual_disallowance() {
        let input = base_individual();
        let output = check(&input);
        assert!(output.note.contains("§ 265(a)(2)"));
    }

    #[test]
    fn note_pins_852b5_exempt_interest_dividends() {
        let mut input = base_individual();
        input.tax_exempt_interest_received_cents = 0;
        let output = check(&input);
        assert!(output.note.contains("§ 852(b)(5)"));
    }

    #[test]
    fn note_pins_form_1120_schedule_m_for_corp_disallowance() {
        let mut input = base_individual();
        input.taxpayer_type = TaxpayerType::NonFinancialCorporation;
        let output = check(&input);
        assert!(output.note.contains("Form 1120 Schedule M-1 / M-3"));
    }

    #[test]
    fn note_pins_bank_qualified_10m_issuer_cap() {
        let mut input = base_bank();
        input.bond_classification = BondClassification::BankQualifiedQualifiedTaxExemptObligation;
        let output = check(&input);
        assert!(output.note.contains("$10,000,000"));
        assert!(output.note.contains("calendar year"));
    }

    #[test]
    fn pre_1986_grandfathering_overrides_non_bank_qualified_classification() {
        let mut input = base_bank();
        input.bond_classification = BondClassification::PreAugust7_1986Grandfathered;
        let output = check(&input);
        // Even though "non-bank-qualified" would normally be 100% disallow, grandfathering
        // takes priority and preserves full deduction
        assert_eq!(output.allowed_interest_expense_cents, 1_000_000_00);
    }

    #[test]
    fn dealer_safe_harbor_overrides_direct_tracing() {
        let mut input = base_individual();
        input.taxpayer_type = TaxpayerType::DealerInTaxExemptObligations;
        input.debt_tracing_status = DebtTracingStatus::DirectlyTraceableToTaxExemptAcquisition;
        let output = check(&input);
        // Dealer safe harbor preserves full deduction even with direct tracing
        assert_eq!(
            output.severity,
            Severity::DealerInTaxExemptObligationsSafeHarborNoDisallowance
        );
    }

    #[test]
    fn bank_severity_independent_of_debt_tracing_status() {
        let mut input = base_bank();
        input.debt_tracing_status = DebtTracingStatus::UnrelatedNonTracingNoDisallowance;
        let output = check(&input);
        // Banks apply pro-rata regardless of tracing
        assert_eq!(
            output.severity,
            Severity::BankNonBankQualifiedHundredPctProRataDisallowance
        );
    }
}
