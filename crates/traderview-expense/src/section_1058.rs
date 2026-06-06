//! IRC § 1058 — Transfers of securities under certain
//! agreements. Provides non-recognition treatment for
//! securities loans satisfying a four-prong qualification
//! test. Trader-critical for every fact pattern involving
//! securities lending — brokerage Stock Yield Enhancement
//! Programs (SYEP), prime-brokerage stock-loan
//! arrangements, fully-paid securities lending, short-
//! seller borrows. Companion to section_1259
//! (constructive sales), section_1092 (straddle rules),
//! section_1236 (definition of "securities" — cross-
//! referenced by § 1058(c)), section_475 (trader mark-to-
//! market), section_988 (foreign currency transactions),
//! section_1233 (short sales).
//!
//! Trader-critical fact patterns:
//! - **Interactive Brokers Stock Yield Enhancement Program
//!   (SYEP)** — retail trader opts in to lend fully-paid
//!   shares to IBKR for redistribution to short sellers in
//!   exchange for 50% of net lending revenue.
//! - **Robinhood Securities Lending Program** —
//!   commission-free brokerage lends customer shares to
//!   institutional borrowers; customer receives split of
//!   lending revenue.
//! - **Charles Schwab Securities Lending Fully Paid +
//!   TD Ameritrade Securities Lending** — similar
//!   structures.
//! - **Hedge fund prime brokerage securities lending** —
//!   institutional clients lend long positions through
//!   prime broker for borrow fee.
//! - **Short seller's borrow of shares** — short seller is
//!   the BORROWER; lender's tax treatment turns on § 1058.
//! - **Crypto staking analogies** — separate analysis
//!   under Notice 2014-21 and Rev. Rul. 2023-14, but § 1058
//!   framework informs tax-character analysis of digital-
//!   asset staking returns.
//!
//! **§ 1058(a) Non-recognition rule** — in case of a
//! taxpayer who transfers securities (as defined in
//! § 1236(c)) pursuant to an agreement which meets the
//! requirements of subsection (b), NO GAIN OR LOSS SHALL
//! BE RECOGNIZED on the exchange of:
//! 1. Such securities by the transferor for an obligation
//!    under such agreement; OR
//! 2. An obligation by the transferor pursuant to such
//!    agreement for securities identical to the
//!    securities transferred.
//!
//! **§ 1058(b) Four-prong qualification test**:
//! 1. **§ 1058(b)(1) Identical securities return** — the
//!    agreement must provide for the return to the
//!    transferor of securities IDENTICAL to the
//!    securities transferred.
//! 2. **§ 1058(b)(2) Dividend-equivalent payments** — the
//!    agreement must REQUIRE that payments be made to the
//!    transferor of amounts EQUIVALENT to all interest,
//!    dividends, and other distributions which the owner
//!    of the securities is entitled to receive during the
//!    period beginning with the transfer of the
//!    securities and ending with the transfer of
//!    identical securities back to the transferor.
//! 3. **§ 1058(b)(3) Risk of loss / opportunity for gain
//!    preserved** — the agreement must NOT REDUCE THE
//!    RISK OF LOSS OR OPPORTUNITY FOR GAIN of the
//!    transferor of the securities in the securities
//!    transferred.
//! 4. **§ 1058(b)(4) Other requirements** — the agreement
//!    must meet such other requirements as the Secretary
//!    may prescribe by regulation; codified by Treas. Reg.
//!    § 1.1058-1 and Rev. Proc. 2008-63 — most
//!    significantly, the **TERMINABLE-ON-DEMAND** rule:
//!    transferor must be able to terminate the agreement
//!    on demand (typically 5 business days).
//!
//! **§ 1058(c) Definition of "securities"** — same as in
//! § 1236(c): any share of stock in any corporation,
//! certificate of stock or interest in any corporation,
//! note, bond, debenture, or evidence of indebtedness, or
//! any evidence of an interest in or right to subscribe
//! to or purchase any of the foregoing.
//!
//! **Holding period tacking under § 1058(a)(2)** — when
//! identical securities are returned to transferor, the
//! transferor's holding period in those returned
//! securities INCLUDES the period during which the
//! transferred securities were loaned. Trader-critical
//! for preserving long-term capital gain treatment when
//! lending shares acquired more than 1 year prior.
//!
//! **Failure consequence** — if any of the four § 1058(b)
//! prongs fails, the transfer is treated as a TAXABLE
//! SALE at fair market value:
//! 1. Recognition of all built-in gain/loss on
//!    transferred securities;
//! 2. Reset of basis to FMV;
//! 3. Holding period restarts (long-term lost);
//! 4. § 1259 constructive sale potentially also engaged
//!    for offsetting positions held simultaneously.
//!
//! **Rev. Proc. 2008-63 safe harbor** — IRS provides safe-
//! harbor terms for securities loans satisfying § 1058
//! framework, including terminable-on-demand requirement
//! and dividend-equivalent payment timing.
//!
//! **Anshutz v. Commissioner, 135 T.C. No. 5 (2010)** +
//! **Calloway v. Commissioner, 135 T.C. No. 3 (2010)** —
//! Tax Court precedent establishing that VARIABLE PREPAID
//! FORWARD CONTRACT combined with stock loan can fail
//! § 1058(b)(3) "risk of loss / opportunity for gain"
//! prong, resulting in constructive sale treatment.
//!
//! Citations: 26 USC § 1058(a)-(c); 26 USC § 1236(c); 26
//! USC § 1259 (constructive sales); 26 USC § 1092
//! (straddles); 26 USC § 475 (trader mark-to-market);
//! Treas. Reg. § 1.1058-1; Rev. Proc. 2008-63; Anshutz v.
//! Commissioner, 135 T.C. No. 5 (2010); Calloway v.
//! Commissioner, 135 T.C. No. 3 (2010).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LendingArrangementType {
    /// Retail brokerage Stock Yield Enhancement Program
    /// (Interactive Brokers SYEP, similar).
    RetailBrokerageSyep,
    /// Fully-paid securities lending program (Robinhood,
    /// Schwab SLFPS).
    FullyPaidLending,
    /// Prime brokerage institutional securities lending.
    PrimeBrokerageLending,
    /// Direct securities loan between counterparties.
    DirectLoan,
    /// Variable prepaid forward contract bundled with
    /// stock loan (Anshutz/Calloway pattern — generally
    /// FAILS § 1058 risk-of-loss prong).
    VariablePrepaidForwardWithStockLoan,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section1058Input {
    pub arrangement_type: LendingArrangementType,
    /// § 1058(b)(1) — agreement provides for return of
    /// IDENTICAL securities to transferor.
    pub return_identical_securities: bool,
    /// § 1058(b)(2) — agreement REQUIRES dividend-
    /// equivalent payments to transferor during loan
    /// period (all interest, dividends, other
    /// distributions).
    pub dividend_equivalent_payments_required: bool,
    /// § 1058(b)(3) — agreement does NOT REDUCE risk of
    /// loss or opportunity for gain of transferor in the
    /// securities transferred.
    pub risk_of_loss_and_opportunity_for_gain_preserved: bool,
    /// § 1058(b)(4) + Treas. Reg. § 1.1058-1 + Rev. Proc.
    /// 2008-63 — agreement is TERMINABLE ON DEMAND
    /// (typically 5 business days).
    pub terminable_on_demand: bool,
    /// FMV of transferred securities in cents (for
    /// constructive-sale calculation on failure).
    pub fmv_at_transfer_cents: u64,
    /// Transferor's tax basis in transferred securities
    /// in cents.
    pub basis_cents: u64,
    /// Original holding period of transferred securities
    /// in days prior to loan (for § 1058(a)(2) tacking +
    /// long-term-vs-short-term determination on return).
    pub original_holding_period_days_before_loan: u32,
    /// Days securities loaned (added to holding period
    /// on return per § 1058(a)(2) tacking).
    pub days_loaned: u32,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NonRecognitionStatus {
    /// All four § 1058(b) prongs satisfied — non-
    /// recognition treatment applies.
    NonRecognitionApplies,
    /// One or more § 1058(b) prongs fail — TAXABLE SALE
    /// at FMV with built-in gain/loss recognition + basis
    /// reset + holding period restart.
    TaxableSaleAtFmv,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section1058Result {
    pub arrangement_type: LendingArrangementType,
    pub non_recognition_status: NonRecognitionStatus,
    pub all_four_prongs_satisfied: bool,
    pub built_in_gain_or_loss_cents: i64,
    pub combined_holding_period_days_after_return: u32,
    pub long_term_treatment_preserved: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section1058Input) -> Section1058Result {
    let mut failure_reasons: Vec<String> = Vec::new();

    let prong_1_satisfied = input.return_identical_securities;
    let prong_2_satisfied = input.dividend_equivalent_payments_required;
    let prong_3_satisfied = input.risk_of_loss_and_opportunity_for_gain_preserved;
    let prong_4_satisfied = input.terminable_on_demand;

    let all_four_prongs_satisfied =
        prong_1_satisfied && prong_2_satisfied && prong_3_satisfied && prong_4_satisfied;

    let arrangement_typically_fails_prong_3 = matches!(
        input.arrangement_type,
        LendingArrangementType::VariablePrepaidForwardWithStockLoan
    );

    let non_recognition_status =
        if all_four_prongs_satisfied && !arrangement_typically_fails_prong_3 {
            NonRecognitionStatus::NonRecognitionApplies
        } else {
            NonRecognitionStatus::TaxableSaleAtFmv
        };

    let built_in_gain_or_loss_cents = if matches!(
        non_recognition_status,
        NonRecognitionStatus::TaxableSaleAtFmv
    ) {
        (input.fmv_at_transfer_cents as i64).saturating_sub(input.basis_cents as i64)
    } else {
        0
    };

    let combined_holding_period_days_after_return = if matches!(
        non_recognition_status,
        NonRecognitionStatus::NonRecognitionApplies
    ) {
        input
            .original_holding_period_days_before_loan
            .saturating_add(input.days_loaned)
    } else {
        0
    };

    let long_term_treatment_preserved = combined_holding_period_days_after_return > 365;

    if !prong_1_satisfied {
        failure_reasons.push(
            "26 USC § 1058(b)(1) — agreement must provide for return to transferor of IDENTICAL SECURITIES; failure to specify identical-securities return clause defeats non-recognition".to_string(),
        );
    }
    if !prong_2_satisfied {
        failure_reasons.push(
            "26 USC § 1058(b)(2) — agreement must REQUIRE that payments be made to transferor of amounts EQUIVALENT to all interest, dividends, and other distributions during loan period (transferred to lender on stretched payment dates)".to_string(),
        );
    }
    if !prong_3_satisfied {
        failure_reasons.push(
            "26 USC § 1058(b)(3) — agreement must NOT REDUCE the RISK OF LOSS OR OPPORTUNITY FOR GAIN of transferor in the securities transferred; arrangements bundling a stock loan with a variable prepaid forward contract (Anshutz/Calloway pattern) generally fail this prong".to_string(),
        );
    }
    if !prong_4_satisfied {
        failure_reasons.push(
            "26 USC § 1058(b)(4) + Treas. Reg. § 1.1058-1 + Rev. Proc. 2008-63 — agreement must be TERMINABLE ON DEMAND by transferor (typically 5 business days under safe-harbor terms)".to_string(),
        );
    }
    if arrangement_typically_fails_prong_3 {
        failure_reasons.push(
            "Anshutz v. Commissioner, 135 T.C. No. 5 (2010) + Calloway v. Commissioner, 135 T.C. No. 3 (2010) — Tax Court precedent: VARIABLE PREPAID FORWARD CONTRACT bundled with stock loan FAILS § 1058(b)(3) risk-of-loss prong; results in constructive sale treatment at FMV plus § 1259 constructive sale potentially engaged for offsetting positions".to_string(),
        );
    }

    if matches!(
        non_recognition_status,
        NonRecognitionStatus::TaxableSaleAtFmv
    ) {
        failure_reasons.push(format!(
            "§ 1058 non-recognition FAILS — TAXABLE SALE at FMV: (1) recognition of built-in gain/loss = FMV ({} cents) - basis ({} cents) = {} cents; (2) reset of basis to FMV; (3) holding period restarts (long-term treatment LOST); (4) § 1259 constructive sale potentially also engaged for offsetting positions held simultaneously",
            input.fmv_at_transfer_cents, input.basis_cents, built_in_gain_or_loss_cents
        ));
    }

    let notes: Vec<String> = vec![
        "26 USC § 1058(a) — taxpayer who transfers securities (as defined in § 1236(c)) pursuant to qualifying agreement: NO GAIN OR LOSS RECOGNIZED on exchange of (1) such securities by transferor for obligation under agreement OR (2) obligation by transferor pursuant to agreement for securities identical to securities transferred".to_string(),
        "26 USC § 1058(b)(1) — agreement must provide for return to transferor of IDENTICAL SECURITIES (same CUSIP, same class, same issuer)".to_string(),
        "26 USC § 1058(b)(2) — agreement must REQUIRE payments to transferor of amounts EQUIVALENT to all interest, dividends, and other distributions during loan period; payments must track exact economic position transferor would have held absent loan".to_string(),
        "26 USC § 1058(b)(3) — agreement must NOT REDUCE the RISK OF LOSS OR OPPORTUNITY FOR GAIN of transferor in transferred securities; this is the prong most often FAILED — particularly by VARIABLE PREPAID FORWARD CONTRACT arrangements".to_string(),
        "26 USC § 1058(b)(4) + Treas. Reg. § 1.1058-1 + Rev. Proc. 2008-63 — agreement must meet other requirements prescribed by Secretary; most significantly, TERMINABLE ON DEMAND (typically 5 business days)".to_string(),
        "26 USC § 1058(c) + § 1236(c) — 'securities' definition: any share of stock + certificate of stock or interest in corporation + note + bond + debenture + evidence of indebtedness + evidence of interest in or right to subscribe to/purchase any of foregoing".to_string(),
        "26 USC § 1058(a)(2) holding period tacking — when identical securities returned, transferor's holding period INCLUDES the period during which transferred securities were loaned; critical for preserving long-term capital gain treatment when lending shares acquired more than 1 year prior".to_string(),
        "Anshutz v. Commissioner, 135 T.C. No. 5 (2010) + Calloway v. Commissioner, 135 T.C. No. 3 (2010) — Tax Court precedent: VARIABLE PREPAID FORWARD CONTRACT bundled with stock loan FAILS § 1058(b)(3) risk-of-loss prong; bundled arrangements treated as constructive sale at FMV".to_string(),
        "Failure consequence — if any of four § 1058(b) prongs fails: (1) recognition of all built-in gain/loss on transferred securities at FMV; (2) reset of basis to FMV; (3) holding period restarts (long-term treatment LOST); (4) § 1259 constructive sale potentially also engaged for offsetting positions".to_string(),
        "Rev. Proc. 2008-63 safe harbor — IRS provides safe-harbor terms for securities loans satisfying § 1058 framework, including terminable-on-demand requirement and dividend-equivalent payment timing".to_string(),
        "Trader-critical fact patterns: Interactive Brokers Stock Yield Enhancement Program (SYEP); Robinhood Securities Lending Program; Charles Schwab Securities Lending Fully Paid (SLFPS); TD Ameritrade Securities Lending; hedge fund prime brokerage stock-loan; short seller borrow of shares (short seller is BORROWER; lender's tax treatment turns on § 1058)".to_string(),
        "Companion provisions: § 1259 (constructive sales) often engages where § 1058 fails; § 1092 straddle rules may interact with stock-loan-plus-option positions; § 1236(c) 'securities' definition cross-referenced; § 475 trader mark-to-market may override § 1058 treatment for dealers".to_string(),
    ];

    Section1058Result {
        arrangement_type: input.arrangement_type,
        non_recognition_status,
        all_four_prongs_satisfied,
        built_in_gain_or_loss_cents,
        combined_holding_period_days_after_return,
        long_term_treatment_preserved,
        failure_reasons,
        citation: "26 USC § 1058(a)-(c); 26 USC § 1236(c); 26 USC § 1259; 26 USC § 1092; 26 USC § 475; Treas. Reg. § 1.1058-1; Rev. Proc. 2008-63; Anshutz v. Commissioner, 135 T.C. No. 5 (2010); Calloway v. Commissioner, 135 T.C. No. 3 (2010)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn syep_compliant() -> Section1058Input {
        Section1058Input {
            arrangement_type: LendingArrangementType::RetailBrokerageSyep,
            return_identical_securities: true,
            dividend_equivalent_payments_required: true,
            risk_of_loss_and_opportunity_for_gain_preserved: true,
            terminable_on_demand: true,
            fmv_at_transfer_cents: 100_000_000,
            basis_cents: 60_000_000,
            original_holding_period_days_before_loan: 400,
            days_loaned: 90,
        }
    }

    #[test]
    fn syep_all_four_prongs_non_recognition() {
        let r = check(&syep_compliant());
        assert_eq!(
            r.non_recognition_status,
            NonRecognitionStatus::NonRecognitionApplies
        );
        assert!(r.all_four_prongs_satisfied);
        assert_eq!(r.built_in_gain_or_loss_cents, 0);
    }

    #[test]
    fn syep_holding_period_tacking_400_plus_90() {
        let r = check(&syep_compliant());
        assert_eq!(r.combined_holding_period_days_after_return, 490);
        assert!(r.long_term_treatment_preserved);
    }

    #[test]
    fn missing_prong_1_identical_securities_violation() {
        let mut i = syep_compliant();
        i.return_identical_securities = false;
        let r = check(&i);
        assert_eq!(
            r.non_recognition_status,
            NonRecognitionStatus::TaxableSaleAtFmv
        );
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 1058(b)(1)") && f.contains("IDENTICAL SECURITIES")));
    }

    #[test]
    fn missing_prong_2_dividend_equivalent_violation() {
        let mut i = syep_compliant();
        i.dividend_equivalent_payments_required = false;
        let r = check(&i);
        assert_eq!(
            r.non_recognition_status,
            NonRecognitionStatus::TaxableSaleAtFmv
        );
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 1058(b)(2)") && f.contains("EQUIVALENT")));
    }

    #[test]
    fn missing_prong_3_risk_of_loss_violation() {
        let mut i = syep_compliant();
        i.risk_of_loss_and_opportunity_for_gain_preserved = false;
        let r = check(&i);
        assert_eq!(
            r.non_recognition_status,
            NonRecognitionStatus::TaxableSaleAtFmv
        );
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 1058(b)(3)")
                && f.contains("RISK OF LOSS OR OPPORTUNITY FOR GAIN")));
    }

    #[test]
    fn missing_prong_4_terminable_on_demand_violation() {
        let mut i = syep_compliant();
        i.terminable_on_demand = false;
        let r = check(&i);
        assert_eq!(
            r.non_recognition_status,
            NonRecognitionStatus::TaxableSaleAtFmv
        );
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 1058(b)(4)")
            && f.contains("Treas. Reg. § 1.1058-1")
            && f.contains("Rev. Proc. 2008-63")
            && f.contains("TERMINABLE ON DEMAND")));
    }

    #[test]
    fn variable_prepaid_forward_with_stock_loan_anshutz_pattern_fails() {
        let mut i = syep_compliant();
        i.arrangement_type = LendingArrangementType::VariablePrepaidForwardWithStockLoan;
        let r = check(&i);
        assert_eq!(
            r.non_recognition_status,
            NonRecognitionStatus::TaxableSaleAtFmv
        );
        assert!(r.failure_reasons.iter().any(|f| f.contains("Anshutz")
            && f.contains("Calloway")
            && f.contains("VARIABLE PREPAID FORWARD CONTRACT")
            && f.contains("§ 1058(b)(3)")));
    }

    #[test]
    fn fully_paid_lending_robinhood_pattern_compliant() {
        let mut i = syep_compliant();
        i.arrangement_type = LendingArrangementType::FullyPaidLending;
        let r = check(&i);
        assert_eq!(
            r.non_recognition_status,
            NonRecognitionStatus::NonRecognitionApplies
        );
    }

    #[test]
    fn prime_brokerage_lending_compliant() {
        let mut i = syep_compliant();
        i.arrangement_type = LendingArrangementType::PrimeBrokerageLending;
        let r = check(&i);
        assert_eq!(
            r.non_recognition_status,
            NonRecognitionStatus::NonRecognitionApplies
        );
    }

    #[test]
    fn direct_loan_compliant() {
        let mut i = syep_compliant();
        i.arrangement_type = LendingArrangementType::DirectLoan;
        let r = check(&i);
        assert_eq!(
            r.non_recognition_status,
            NonRecognitionStatus::NonRecognitionApplies
        );
    }

    #[test]
    fn failure_computes_built_in_gain() {
        let mut i = syep_compliant();
        i.return_identical_securities = false;
        i.fmv_at_transfer_cents = 100_000_000;
        i.basis_cents = 60_000_000;
        let r = check(&i);
        assert_eq!(r.built_in_gain_or_loss_cents, 40_000_000);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("TAXABLE SALE at FMV") && f.contains("40000000 cents")));
    }

    #[test]
    fn failure_computes_built_in_loss_negative() {
        let mut i = syep_compliant();
        i.return_identical_securities = false;
        i.fmv_at_transfer_cents = 40_000_000;
        i.basis_cents = 60_000_000;
        let r = check(&i);
        assert_eq!(r.built_in_gain_or_loss_cents, -20_000_000);
    }

    #[test]
    fn failure_holding_period_resets_to_zero() {
        let mut i = syep_compliant();
        i.return_identical_securities = false;
        let r = check(&i);
        assert_eq!(r.combined_holding_period_days_after_return, 0);
        assert!(!r.long_term_treatment_preserved);
    }

    #[test]
    fn short_term_to_long_term_via_tacking() {
        let mut i = syep_compliant();
        i.original_holding_period_days_before_loan = 300;
        i.days_loaned = 90;
        let r = check(&i);
        assert_eq!(r.combined_holding_period_days_after_return, 390);
        assert!(r.long_term_treatment_preserved);
    }

    #[test]
    fn short_term_remains_short_term_with_brief_loan() {
        let mut i = syep_compliant();
        i.original_holding_period_days_before_loan = 100;
        i.days_loaned = 30;
        let r = check(&i);
        assert_eq!(r.combined_holding_period_days_after_return, 130);
        assert!(!r.long_term_treatment_preserved);
    }

    #[test]
    fn long_term_boundary_exactly_366_days() {
        let mut i = syep_compliant();
        i.original_holding_period_days_before_loan = 365;
        i.days_loaned = 1;
        let r = check(&i);
        assert_eq!(r.combined_holding_period_days_after_return, 366);
        assert!(r.long_term_treatment_preserved);
    }

    #[test]
    fn long_term_boundary_exactly_365_days_short_term() {
        let mut i = syep_compliant();
        i.original_holding_period_days_before_loan = 364;
        i.days_loaned = 1;
        let r = check(&i);
        assert_eq!(r.combined_holding_period_days_after_return, 365);
        assert!(!r.long_term_treatment_preserved);
    }

    #[test]
    fn multiple_prong_failures_stack() {
        let mut i = syep_compliant();
        i.return_identical_securities = false;
        i.dividend_equivalent_payments_required = false;
        i.risk_of_loss_and_opportunity_for_gain_preserved = false;
        i.terminable_on_demand = false;
        let r = check(&i);
        assert!(r.failure_reasons.len() >= 5);
    }

    #[test]
    fn arrangement_type_truth_table_five_cells() {
        for (arr, exp) in [
            (
                LendingArrangementType::RetailBrokerageSyep,
                NonRecognitionStatus::NonRecognitionApplies,
            ),
            (
                LendingArrangementType::FullyPaidLending,
                NonRecognitionStatus::NonRecognitionApplies,
            ),
            (
                LendingArrangementType::PrimeBrokerageLending,
                NonRecognitionStatus::NonRecognitionApplies,
            ),
            (
                LendingArrangementType::DirectLoan,
                NonRecognitionStatus::NonRecognitionApplies,
            ),
            (
                LendingArrangementType::VariablePrepaidForwardWithStockLoan,
                NonRecognitionStatus::TaxableSaleAtFmv,
            ),
        ] {
            let mut i = syep_compliant();
            i.arrangement_type = arr;
            let r = check(&i);
            assert_eq!(r.non_recognition_status, exp, "arr={:?}", arr);
        }
    }

    #[test]
    fn vpf_with_stock_loan_uniquely_fails_invariant() {
        let mut vpf = syep_compliant();
        vpf.arrangement_type = LendingArrangementType::VariablePrepaidForwardWithStockLoan;
        let r_vpf = check(&vpf);
        assert_eq!(
            r_vpf.non_recognition_status,
            NonRecognitionStatus::TaxableSaleAtFmv
        );

        for arr in [
            LendingArrangementType::RetailBrokerageSyep,
            LendingArrangementType::FullyPaidLending,
            LendingArrangementType::PrimeBrokerageLending,
            LendingArrangementType::DirectLoan,
        ] {
            let mut i = syep_compliant();
            i.arrangement_type = arr;
            let r = check(&i);
            assert_eq!(
                r.non_recognition_status,
                NonRecognitionStatus::NonRecognitionApplies
            );
        }
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&syep_compliant());
        assert!(r.citation.contains("§ 1058(a)-(c)"));
        assert!(r.citation.contains("§ 1236(c)"));
        assert!(r.citation.contains("§ 1259"));
        assert!(r.citation.contains("§ 1092"));
        assert!(r.citation.contains("§ 475"));
        assert!(r.citation.contains("Treas. Reg. § 1.1058-1"));
        assert!(r.citation.contains("Rev. Proc. 2008-63"));
        assert!(r
            .citation
            .contains("Anshutz v. Commissioner, 135 T.C. No. 5"));
        assert!(r
            .citation
            .contains("Calloway v. Commissioner, 135 T.C. No. 3"));
    }

    #[test]
    fn note_pins_subsection_a_non_recognition_rule() {
        let r = check(&syep_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 1058(a)")
            && n.contains("NO GAIN OR LOSS RECOGNIZED")
            && n.contains("§ 1236(c)")));
    }

    #[test]
    fn note_pins_subsection_b1_identical_securities() {
        let r = check(&syep_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 1058(b)(1)")
            && n.contains("IDENTICAL SECURITIES")
            && n.contains("CUSIP")));
    }

    #[test]
    fn note_pins_subsection_b2_dividend_equivalent() {
        let r = check(&syep_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 1058(b)(2)")
            && n.contains("EQUIVALENT")
            && n.contains("dividends")
            && n.contains("exact economic position")));
    }

    #[test]
    fn note_pins_subsection_b3_risk_of_loss_most_failed() {
        let r = check(&syep_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 1058(b)(3)")
            && n.contains("RISK OF LOSS OR OPPORTUNITY FOR GAIN")
            && n.contains("most often FAILED")
            && n.contains("VARIABLE PREPAID FORWARD CONTRACT")));
    }

    #[test]
    fn note_pins_subsection_b4_terminable_on_demand() {
        let r = check(&syep_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 1058(b)(4)")
            && n.contains("Treas. Reg. § 1.1058-1")
            && n.contains("Rev. Proc. 2008-63")
            && n.contains("TERMINABLE ON DEMAND")
            && n.contains("5 business days")));
    }

    #[test]
    fn note_pins_subsection_c_securities_definition() {
        let r = check(&syep_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 1058(c)")
            && n.contains("§ 1236(c)")
            && n.contains("any share of stock")
            && n.contains("debenture")));
    }

    #[test]
    fn note_pins_subsection_a2_holding_period_tacking() {
        let r = check(&syep_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 1058(a)(2)")
            && n.contains("holding period tacking")
            && n.contains("INCLUDES the period")
            && n.contains("more than 1 year prior")));
    }

    #[test]
    fn note_pins_anshutz_calloway_precedent() {
        let r = check(&syep_compliant());
        assert!(r.notes.iter().any(|n| n.contains("Anshutz v. Commissioner")
            && n.contains("135 T.C. No. 5")
            && n.contains("Calloway v. Commissioner")
            && n.contains("135 T.C. No. 3")));
    }

    #[test]
    fn note_pins_failure_consequence_four_elements() {
        let r = check(&syep_compliant());
        assert!(r.notes.iter().any(|n| n.contains("Failure consequence")
            && n.contains("built-in gain/loss")
            && n.contains("reset of basis to FMV")
            && n.contains("holding period restarts")
            && n.contains("§ 1259 constructive sale")));
    }

    #[test]
    fn note_pins_rev_proc_2008_63_safe_harbor() {
        let r = check(&syep_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Rev. Proc. 2008-63 safe harbor")
                && n.contains("terminable-on-demand")));
    }

    #[test]
    fn note_pins_trader_critical_fact_patterns() {
        let r = check(&syep_compliant());
        assert!(r.notes.iter().any(|n| n
            .contains("Interactive Brokers Stock Yield Enhancement Program (SYEP)")
            && n.contains("Robinhood Securities Lending")
            && n.contains("Charles Schwab Securities Lending Fully Paid")
            && n.contains("BORROWER")));
    }

    #[test]
    fn note_pins_companion_provisions() {
        let r = check(&syep_compliant());
        assert!(r.notes.iter().any(|n| n.contains("Companion provisions")
            && n.contains("§ 1259")
            && n.contains("§ 1092")
            && n.contains("§ 1236(c)")
            && n.contains("§ 475")));
    }

    #[test]
    fn defensive_zero_fmv_zero_basis_no_overflow() {
        let mut i = syep_compliant();
        i.return_identical_securities = false;
        i.fmv_at_transfer_cents = 0;
        i.basis_cents = 0;
        let r = check(&i);
        assert_eq!(r.built_in_gain_or_loss_cents, 0);
    }

    #[test]
    fn defensive_overflow_holding_period_saturating() {
        let mut i = syep_compliant();
        i.original_holding_period_days_before_loan = u32::MAX;
        i.days_loaned = u32::MAX;
        let r = check(&i);
        let _ = r.combined_holding_period_days_after_return;
    }
}
