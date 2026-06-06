//! IRC §7872 — Treatment of loans with below-market interest rates.
//!
//! The family "sweetheart loan" trap. When a trader lends money to a
//! family member, child, or controlled entity at a below-market rate,
//! the IRS imputes the missing interest as if the loan had charged the
//! Applicable Federal Rate (AFR). The forgone interest is:
//!
//!   - **Income to the lender** (interest), and
//!   - **Deemed transferred back to the borrower** as a gift, comp, or
//!     dividend depending on the relationship — the borrower then
//!     "deems re-pays" it as interest.
//!
//! Net effect: lender owes income tax on interest never received; the
//! deemed gift to the borrower may also burn lifetime gift exemption.
//!
//! **AFR brackets by term** under §1274(d):
//!
//!   - **Short-term**: loan term ≤ 3 years
//!   - **Mid-term**: 3 years < term ≤ 9 years
//!   - **Long-term**: term > 9 years
//!
//! **Two narrow exceptions for GIFT loans only** (no exceptions for
//! compensation-related or corporate-shareholder loans):
//!
//! **§7872(c)(2)(A) — $10,000 de minimis**. If the aggregate outstanding
//! amount of all gift loans between the same lender and borrower never
//! exceeds $10,000 AND the loan is not directly attributable to the
//! purchase or carrying of income-producing assets, NO imputation.
//!
//! **§7872(d)(1) — $100,000 NII cap**. For gift loans where the
//! aggregate outstanding does not exceed $100,000:
//!   - If borrower's net investment income for the year is ≤ $1,000:
//!     NO imputation (separate de minimis floor on NII).
//!   - If borrower's NII > $1,000: imputation is CAPPED at the
//!     borrower's NII (rather than the full AFR calculation). The
//!     lender can never be imputed more income than the borrower has
//!     net investment income.
//!
//! Loans with aggregate outstanding > $100,000 get full AFR imputation.
//! Same for non-gift loans of any size.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LoanType {
    /// Loan between family/friends where the forgone interest is in the
    /// nature of a gift (§7872(c)(1)(A)).
    Gift,
    /// Loan between employer and employee (§7872(c)(1)(B)) where forgone
    /// interest is compensation.
    Compensation,
    /// Loan from corporation to shareholder (§7872(c)(1)(C)) where the
    /// forgone interest is treated as a dividend.
    CorporationShareholder,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AfrTerm {
    /// ≤ 3 years
    ShortTerm,
    /// > 3 and ≤ 9 years
    MidTerm,
    /// > 9 years
    LongTerm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section7872Input {
    pub loan_principal: Decimal,
    /// Loan term in years (fractional accepted; e.g., 1.5 = 18 months).
    pub loan_term_years: Decimal,
    /// Actual annual interest rate charged on the loan (as a decimal —
    /// 0.02 = 2.0%).
    pub actual_interest_rate: Decimal,
    /// Current AFR for the loan's term (as a decimal — 0.045 = 4.5%).
    /// Caller passes the IRS-published rate for the month the loan was
    /// made; module doesn't fetch.
    pub applicable_federal_rate: Decimal,
    pub loan_type: LoanType,
    /// Aggregate outstanding amount across all loans between the same
    /// lender and borrower. Drives the §7872(c)(2)(A) $10k and
    /// §7872(d)(1) $100k thresholds.
    pub aggregate_outstanding_between_parties: Decimal,
    /// Borrower's net investment income for the year. Drives the
    /// §7872(d)(1) cap on gift loans ≤ $100k.
    pub borrower_net_investment_income: Decimal,
    /// True if the loan proceeds are directly attributable to the
    /// purchase or carrying of income-producing assets — disables the
    /// $10k de minimis exception under §7872(c)(2)(A).
    pub used_for_income_producing_assets: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Section7872Rule {
    /// §7872(c)(2)(A) — $10k aggregate de minimis; no imputation.
    DeMinimisTenThousandException,
    /// §7872(d)(1) — gift loan ≤ $100k AND borrower NII ≤ $1k; no
    /// imputation.
    NiiBelowThousandFloorException,
    /// §7872(d)(1) — gift loan ≤ $100k, NII > $1k, imputation capped
    /// at NII.
    CappedAtBorrowerNii,
    /// §7872(a)(1) — full AFR imputation applies.
    FullAfrImputation,
    /// Loan rate ≥ AFR; no imputation.
    NoImputationRateMeetsAfr,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section7872Result {
    pub rule_path: Section7872Rule,
    pub afr_term_classification: AfrTerm,
    /// Forgone interest BEFORE any caps or exceptions — principal ×
    /// (AFR − actual rate), annualized.
    pub forgone_interest_raw: Decimal,
    /// Forgone interest AFTER applying §7872(d) NII cap or full
    /// imputation. Zero on de minimis / NII-below-floor / rate-meets-AFR
    /// paths.
    pub imputed_interest_income_to_lender: Decimal,
    /// Same dollar amount as the lender's imputed interest income; the
    /// borrower is deemed to receive a gift/comp/dividend equal to this
    /// amount before deeming it paid back as interest.
    pub deemed_transfer_to_borrower: Decimal,
    pub note: String,
}

/// §7872(c)(2)(A) $10,000 aggregate threshold for gift loan de minimis.
const TEN_THOUSAND: Decimal = Decimal::from_parts(10_000, 0, 0, false, 0);
/// §7872(d)(1) $100,000 aggregate threshold for NII cap.
const HUNDRED_THOUSAND: Decimal = Decimal::from_parts(100_000, 0, 0, false, 0);
/// §7872(d)(1) NII de minimis floor ($1,000).
const ONE_THOUSAND: Decimal = Decimal::from_parts(1_000, 0, 0, false, 0);

pub fn compute(input: &Section7872Input) -> Section7872Result {
    // Step 1: Classify the AFR term bracket.
    let term = if input.loan_term_years <= Decimal::from(3) {
        AfrTerm::ShortTerm
    } else if input.loan_term_years <= Decimal::from(9) {
        AfrTerm::MidTerm
    } else {
        AfrTerm::LongTerm
    };

    // Step 2: Compute raw forgone interest = principal × (AFR − actual).
    let rate_gap = input.applicable_federal_rate - input.actual_interest_rate;
    let forgone_raw = if rate_gap > Decimal::ZERO {
        input.loan_principal * rate_gap
    } else {
        Decimal::ZERO
    };

    // Step 3: If actual rate ≥ AFR, no imputation regardless of type.
    if rate_gap <= Decimal::ZERO {
        return Section7872Result {
            rule_path: Section7872Rule::NoImputationRateMeetsAfr,
            afr_term_classification: term,
            forgone_interest_raw: forgone_raw,
            imputed_interest_income_to_lender: Decimal::ZERO,
            deemed_transfer_to_borrower: Decimal::ZERO,
            note: format!(
                "actual rate {} ≥ AFR {} — no imputation under §7872",
                input.actual_interest_rate.round_dp(4),
                input.applicable_federal_rate.round_dp(4)
            ),
        };
    }

    // Step 4: §7872(c)(2)(A) $10k de minimis — gift loans only AND
    // proceeds not used for income-producing assets.
    if matches!(input.loan_type, LoanType::Gift)
        && input.aggregate_outstanding_between_parties <= TEN_THOUSAND
        && !input.used_for_income_producing_assets
    {
        return Section7872Result {
            rule_path: Section7872Rule::DeMinimisTenThousandException,
            afr_term_classification: term,
            forgone_interest_raw: forgone_raw,
            imputed_interest_income_to_lender: Decimal::ZERO,
            deemed_transfer_to_borrower: Decimal::ZERO,
            note: format!(
                "§7872(c)(2)(A) $10,000 de minimis — gift loan with aggregate ${} ≤ $10,000 + not used for income-producing assets; no imputation",
                input.aggregate_outstanding_between_parties.round_dp(2)
            ),
        };
    }

    // Step 5: §7872(d)(1) — gift loan ≤ $100k → NII cap path.
    if matches!(input.loan_type, LoanType::Gift)
        && input.aggregate_outstanding_between_parties <= HUNDRED_THOUSAND
    {
        if input.borrower_net_investment_income <= ONE_THOUSAND {
            // NII floor: ≤ $1,000 → no imputation entirely.
            return Section7872Result {
                rule_path: Section7872Rule::NiiBelowThousandFloorException,
                afr_term_classification: term,
                forgone_interest_raw: forgone_raw,
                imputed_interest_income_to_lender: Decimal::ZERO,
                deemed_transfer_to_borrower: Decimal::ZERO,
                note: format!(
                    "§7872(d)(1) gift loan ≤ $100k with borrower NII ${} ≤ $1,000 — no imputation",
                    input.borrower_net_investment_income.round_dp(2)
                ),
            };
        }
        // NII > $1k: cap imputation at NII.
        let capped = forgone_raw.min(input.borrower_net_investment_income);
        return Section7872Result {
            rule_path: Section7872Rule::CappedAtBorrowerNii,
            afr_term_classification: term,
            forgone_interest_raw: forgone_raw,
            imputed_interest_income_to_lender: capped,
            deemed_transfer_to_borrower: capped,
            note: format!(
                "§7872(d)(1) gift loan ≤ $100k — raw forgone ${} capped at borrower NII ${}",
                forgone_raw.round_dp(2),
                capped.round_dp(2)
            ),
        };
    }

    // Step 6: §7872(a)(1) full AFR imputation. Includes:
    //   - Gift loans > $100k
    //   - All compensation-related loans (any size)
    //   - All corporation/shareholder loans (any size)
    let type_phrase = match input.loan_type {
        LoanType::Gift => "gift loan > $100k",
        LoanType::Compensation => "compensation-related loan (no de minimis)",
        LoanType::CorporationShareholder => "corporation/shareholder loan (no de minimis)",
    };
    Section7872Result {
        rule_path: Section7872Rule::FullAfrImputation,
        afr_term_classification: term,
        forgone_interest_raw: forgone_raw,
        imputed_interest_income_to_lender: forgone_raw,
        deemed_transfer_to_borrower: forgone_raw,
        note: format!(
            "§7872(a)(1) full AFR imputation — {} → lender imputed ${} interest income; deemed transfer ${} to borrower",
            type_phrase,
            forgone_raw.round_dp(2),
            forgone_raw.round_dp(2)
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn base_gift() -> Section7872Input {
        Section7872Input {
            loan_principal: dec!(50_000),
            loan_term_years: dec!(5),
            actual_interest_rate: dec!(0.01),    // 1%
            applicable_federal_rate: dec!(0.04), // 4%
            loan_type: LoanType::Gift,
            aggregate_outstanding_between_parties: dec!(50_000),
            borrower_net_investment_income: dec!(500), // below $1k
            used_for_income_producing_assets: false,
        }
    }

    #[test]
    fn rate_meets_afr_no_imputation() {
        // Loan at exactly AFR (or above) → no imputation regardless of
        // loan type.
        let mut i = base_gift();
        i.actual_interest_rate = dec!(0.04); // = AFR
        let r = compute(&i);
        assert_eq!(r.rule_path, Section7872Rule::NoImputationRateMeetsAfr);
        assert_eq!(r.imputed_interest_income_to_lender, Decimal::ZERO);
    }

    #[test]
    fn rate_above_afr_no_imputation() {
        let mut i = base_gift();
        i.actual_interest_rate = dec!(0.05); // 5% > 4% AFR
        let r = compute(&i);
        assert_eq!(r.rule_path, Section7872Rule::NoImputationRateMeetsAfr);
    }

    #[test]
    fn gift_loan_10k_de_minimis_no_imputation() {
        // $10k exact + gift + not used for income-producing → de minimis.
        let mut i = base_gift();
        i.loan_principal = dec!(10_000);
        i.aggregate_outstanding_between_parties = dec!(10_000);
        i.borrower_net_investment_income = dec!(50_000); // high NII irrelevant
        let r = compute(&i);
        assert_eq!(r.rule_path, Section7872Rule::DeMinimisTenThousandException);
        assert_eq!(r.imputed_interest_income_to_lender, Decimal::ZERO);
    }

    #[test]
    fn gift_loan_10001_exceeds_de_minimis() {
        // $10,001 > $10k threshold → de minimis doesn't apply.
        let mut i = base_gift();
        i.loan_principal = dec!(10_001);
        i.aggregate_outstanding_between_parties = dec!(10_001);
        let r = compute(&i);
        assert_ne!(r.rule_path, Section7872Rule::DeMinimisTenThousandException);
    }

    #[test]
    fn gift_loan_10k_used_for_income_producing_disables_de_minimis() {
        // §7872(c)(2)(A) requires "not directly attributable to the
        // purchase or carrying of income-producing assets" — flag
        // disables the exception even at $10k.
        let mut i = base_gift();
        i.loan_principal = dec!(10_000);
        i.aggregate_outstanding_between_parties = dec!(10_000);
        i.used_for_income_producing_assets = true;
        let r = compute(&i);
        assert_ne!(r.rule_path, Section7872Rule::DeMinimisTenThousandException);
    }

    #[test]
    fn gift_loan_below_100k_nii_below_1k_no_imputation() {
        // §7872(d)(1) NII floor: gift loan ≤ $100k AND NII ≤ $1k → 0.
        let mut i = base_gift();
        i.aggregate_outstanding_between_parties = dec!(50_000);
        i.borrower_net_investment_income = dec!(500); // below $1k
        let r = compute(&i);
        assert_eq!(r.rule_path, Section7872Rule::NiiBelowThousandFloorException);
        assert_eq!(r.imputed_interest_income_to_lender, Decimal::ZERO);
    }

    #[test]
    fn gift_loan_below_100k_nii_exact_1k_no_imputation() {
        // NII exactly $1,000 = ≤ $1k → exception applies (boundary).
        let mut i = base_gift();
        i.aggregate_outstanding_between_parties = dec!(50_000);
        i.borrower_net_investment_income = dec!(1_000);
        let r = compute(&i);
        assert_eq!(r.rule_path, Section7872Rule::NiiBelowThousandFloorException);
    }

    #[test]
    fn gift_loan_below_100k_nii_above_1k_capped_at_nii() {
        // NII = $2,000 > $1k. Raw forgone = $50k × (0.04 − 0.01) = $1,500.
        // NII cap = $2,000 → no actual cap; impute full $1,500.
        let mut i = base_gift();
        i.aggregate_outstanding_between_parties = dec!(50_000);
        i.borrower_net_investment_income = dec!(2_000);
        let r = compute(&i);
        assert_eq!(r.rule_path, Section7872Rule::CappedAtBorrowerNii);
        assert_eq!(r.forgone_interest_raw, dec!(1_500));
        assert_eq!(r.imputed_interest_income_to_lender, dec!(1_500));
    }

    #[test]
    fn gift_loan_below_100k_nii_caps_higher_raw_forgone() {
        // Raw forgone $3,000 (e.g., $100k × 3%) but NII only $1,500 →
        // cap binds.
        let mut i = base_gift();
        i.loan_principal = dec!(100_000);
        i.aggregate_outstanding_between_parties = dec!(100_000);
        i.borrower_net_investment_income = dec!(1_500);
        i.actual_interest_rate = dec!(0.01);
        i.applicable_federal_rate = dec!(0.04); // 3% gap × $100k = $3k
        let r = compute(&i);
        assert_eq!(r.forgone_interest_raw, dec!(3_000));
        assert_eq!(r.imputed_interest_income_to_lender, dec!(1_500)); // capped
    }

    #[test]
    fn gift_loan_above_100k_full_afr_imputation() {
        // > $100k → §7872(a)(1) full imputation; no NII cap.
        let mut i = base_gift();
        i.loan_principal = dec!(200_000);
        i.aggregate_outstanding_between_parties = dec!(200_000);
        i.borrower_net_investment_income = dec!(500); // would normally floor
        let r = compute(&i);
        assert_eq!(r.rule_path, Section7872Rule::FullAfrImputation);
        // $200k × 3% = $6,000.
        assert_eq!(r.imputed_interest_income_to_lender, dec!(6_000));
    }

    #[test]
    fn compensation_loan_no_de_minimis_no_nii_cap() {
        // Compensation-related: §7872(c)(1)(B). No exceptions even for
        // small loans.
        let mut i = base_gift();
        i.loan_type = LoanType::Compensation;
        i.loan_principal = dec!(5_000); // would be de minimis if gift
        i.aggregate_outstanding_between_parties = dec!(5_000);
        let r = compute(&i);
        assert_eq!(r.rule_path, Section7872Rule::FullAfrImputation);
        // $5k × 3% = $150 imputed.
        assert_eq!(r.imputed_interest_income_to_lender, dec!(150));
    }

    #[test]
    fn corporation_shareholder_loan_no_exceptions() {
        // §7872(c)(1)(C): corp-shareholder. Same as compensation — no
        // de minimis, no NII cap.
        let mut i = base_gift();
        i.loan_type = LoanType::CorporationShareholder;
        i.loan_principal = dec!(8_000);
        i.aggregate_outstanding_between_parties = dec!(8_000);
        let r = compute(&i);
        assert_eq!(r.rule_path, Section7872Rule::FullAfrImputation);
    }

    #[test]
    fn aggregate_outstanding_threshold_uses_aggregate_not_principal() {
        // Test loan of $5k (under de minimis) but aggregate $15k →
        // de minimis doesn't apply because aggregate exceeds $10k.
        let mut i = base_gift();
        i.loan_principal = dec!(5_000);
        i.aggregate_outstanding_between_parties = dec!(15_000);
        i.borrower_net_investment_income = dec!(500); // below $1k floor
        let r = compute(&i);
        // Falls through to NII floor (gift loan ≤ $100k, NII ≤ $1k) →
        // no imputation under §7872(d)(1).
        assert_eq!(r.rule_path, Section7872Rule::NiiBelowThousandFloorException);
    }

    #[test]
    fn short_term_loan_classified_at_3_year_boundary() {
        // Term = 3 years exactly → short-term.
        let mut i = base_gift();
        i.loan_term_years = dec!(3);
        let r = compute(&i);
        assert_eq!(r.afr_term_classification, AfrTerm::ShortTerm);
    }

    #[test]
    fn mid_term_loan_at_3_year_plus_boundary() {
        // Term > 3 years → mid-term.
        let mut i = base_gift();
        i.loan_term_years = dec!(3.01);
        let r = compute(&i);
        assert_eq!(r.afr_term_classification, AfrTerm::MidTerm);
    }

    #[test]
    fn mid_term_loan_at_9_year_boundary() {
        // Term = 9 years exactly → still mid-term (boundary inclusive).
        let mut i = base_gift();
        i.loan_term_years = dec!(9);
        let r = compute(&i);
        assert_eq!(r.afr_term_classification, AfrTerm::MidTerm);
    }

    #[test]
    fn long_term_loan_at_9_year_plus_boundary() {
        // Term > 9 years → long-term.
        let mut i = base_gift();
        i.loan_term_years = dec!(9.01);
        let r = compute(&i);
        assert_eq!(r.afr_term_classification, AfrTerm::LongTerm);
    }

    #[test]
    fn zero_interest_loan_full_forgone_at_afr() {
        // 0% loan + $200k principal + 4% AFR → forgone = $8,000.
        let mut i = base_gift();
        i.actual_interest_rate = Decimal::ZERO;
        i.loan_principal = dec!(200_000);
        i.aggregate_outstanding_between_parties = dec!(200_000);
        i.applicable_federal_rate = dec!(0.04);
        let r = compute(&i);
        assert_eq!(r.forgone_interest_raw, dec!(8_000));
        assert_eq!(r.imputed_interest_income_to_lender, dec!(8_000));
    }

    #[test]
    fn forgone_raw_reported_even_when_exception_applies() {
        // Even on exception paths, the raw forgone interest is reported
        // for diagnostic / documentation purposes. UI can show "would
        // have been $X but exception applies".
        let mut i = base_gift();
        i.loan_principal = dec!(10_000);
        i.aggregate_outstanding_between_parties = dec!(10_000);
        let r = compute(&i);
        assert_eq!(r.rule_path, Section7872Rule::DeMinimisTenThousandException);
        assert_eq!(r.forgone_interest_raw, dec!(300)); // $10k × 3%
        assert_eq!(r.imputed_interest_income_to_lender, Decimal::ZERO);
    }

    #[test]
    fn aggregate_at_100k_exact_within_nii_cap_path() {
        // Aggregate $100k exact = ≤ $100k → NII cap path.
        let mut i = base_gift();
        i.loan_principal = dec!(100_000);
        i.aggregate_outstanding_between_parties = dec!(100_000);
        i.borrower_net_investment_income = dec!(2_500);
        let r = compute(&i);
        assert_eq!(r.rule_path, Section7872Rule::CappedAtBorrowerNii);
    }

    #[test]
    fn aggregate_above_100k_full_imputation_path() {
        let mut i = base_gift();
        i.loan_principal = dec!(100_001);
        i.aggregate_outstanding_between_parties = dec!(100_001);
        i.borrower_net_investment_income = dec!(500);
        let r = compute(&i);
        assert_eq!(r.rule_path, Section7872Rule::FullAfrImputation);
    }

    #[test]
    fn deemed_transfer_equals_imputed_income() {
        // The lender's imputed income and the deemed transfer to the
        // borrower are always equal — mirror operations.
        let r = compute(&base_gift());
        assert_eq!(
            r.imputed_interest_income_to_lender,
            r.deemed_transfer_to_borrower
        );
    }

    #[test]
    fn very_large_loan_no_precision_loss() {
        // $10B HNW intra-family loan. Decimal stays exact across the
        // multiplication.
        let mut i = base_gift();
        i.loan_principal = dec!(10_000_000_000);
        i.aggregate_outstanding_between_parties = dec!(10_000_000_000);
        i.borrower_net_investment_income = dec!(1_000_000);
        i.actual_interest_rate = dec!(0.01);
        i.applicable_federal_rate = dec!(0.04);
        let r = compute(&i);
        // $10B × 3% = $300M
        assert_eq!(r.forgone_interest_raw, dec!(300_000_000));
        assert_eq!(r.imputed_interest_income_to_lender, dec!(300_000_000));
    }

    #[test]
    fn note_describes_rule_path_per_branch() {
        // Each rule path should produce a distinct human-readable note.
        let r = compute(&base_gift());
        assert!(r.note.contains("§7872(d)(1)") || r.note.contains("§7872"));

        let mut full = base_gift();
        full.loan_principal = dec!(200_000);
        full.aggregate_outstanding_between_parties = dec!(200_000);
        let r2 = compute(&full);
        assert!(r2.note.contains("§7872(a)(1)"));

        let mut de_min = base_gift();
        de_min.loan_principal = dec!(10_000);
        de_min.aggregate_outstanding_between_parties = dec!(10_000);
        let r3 = compute(&de_min);
        assert!(r3.note.contains("§7872(c)(2)(A)"));
    }
}
