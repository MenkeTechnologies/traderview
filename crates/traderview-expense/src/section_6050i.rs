//! IRC § 6050I — Returns relating to cash received in trade or business
//! (Form 8300).
//!
//! 26 U.S.C. § 6050I requires any person engaged in a TRADE OR BUSINESS
//! who receives more than $10,000 in CASH in one transaction (or two or
//! more related transactions) to report to the IRS AND FinCEN within
//! 15 days. Reported on Form 8300, "Report of Cash Payments Over
//! $10,000 in a Trade or Business." Filing failures trigger penalties
//! under § 6721/§ 6722; willful violations are criminal under § 7203.
//!
//! Cash definition (§ 6050I(d)): currency + cashier's checks + money
//! orders + bank drafts WITH FACE AMOUNT ≤ $10,000. Personal checks
//! are NOT cash. Wire transfers are NOT cash.
//!
//! IIJA 2021 amendment (§ 80603 IIJA P.L. 117-58) added DIGITAL ASSETS
//! to the § 6050I(d) cash definition effective 2024-01-01. HOWEVER, IRS
//! Announcement 2024-04 issued transitional guidance that DIGITAL
//! ASSETS SHOULD NOT BE INCLUDED in the $10,000 threshold until the
//! IRS publishes regulations and updates Form 8300. As of late 2025,
//! the IRS has NOT turned on the digital asset portion of § 6050I.
//! This module reflects the current effective regime — digital assets
//! are EXCLUDED from cash for § 6050I purposes pending IRS regulation.
//!
//! Related transactions — § 6050I aggregates multiple transactions from
//! the same payer within a 24-HOUR window. Two $6,000 cash payments
//! from the same person within 24 hours aggregate to $12,000 and
//! trigger Form 8300.
//!
//! Filing deadline (§ 6050I(b)): 15 days from the transaction date.
//!
//! Penalties: § 6721 intentional disregard penalty is the GREATER of
//! $250,000 or the aggregate amount required to be reported (uncapped
//! for intentional disregard).
//!
//! Citations: 26 U.S.C. § 6050I; § 6050I(a) (general reporting
//! requirement); § 6050I(b) (15-day filing deadline); § 6050I(d) (cash
//! definition); IIJA § 80603 (P.L. 117-58 digital asset amendment, eff.
//! 2024-01-01); IRS Announcement 2024-04 (transitional digital-asset
//! exclusion); § 6721 (intentional disregard penalty); § 7203
//! (willful-failure criminal exposure); FinCEN Form 8300 (joint IRS/
//! FinCEN return).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentType {
    Currency,
    CashiersCheck,
    MoneyOrder,
    BankDraft,
    PersonalCheck,
    DigitalAsset,
    WireTransfer,
    OtherNonCash,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section6050IInput {
    pub recipient_engaged_in_trade_or_business: bool,
    pub payment_type: PaymentType,
    /// Face amount of single instrument (relevant for the $10K-face-
    /// amount limit on cashier's check / money order / bank draft).
    pub single_instrument_face_amount_cents: i64,
    /// Total of all related cash transactions from the same payer within
    /// a 24-hour rolling window. Drives the aggregation rule.
    pub aggregate_related_24_hour_amount_cents: i64,
    pub days_since_transaction: u32,
    pub form_8300_filed: bool,
    /// Whether the failure was intentional disregard (drives § 6721
    /// uncapped penalty).
    pub intentional_disregard: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ViolationType {
    None,
    FormNotFiled,
    FiledLate,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section6050IResult {
    pub payment_qualifies_as_cash: bool,
    pub digital_asset_currently_excluded: bool,
    pub aggregate_exceeds_threshold: bool,
    pub form_8300_required: bool,
    pub statutory_filing_deadline_days: u32,
    pub within_15_day_window: bool,
    pub violation: ViolationType,
    pub maximum_per_violation_penalty_cents: i64,
    pub criminal_exposure: bool,
    pub citation: &'static str,
    pub note: String,
}

pub fn compute(input: &Section6050IInput) -> Section6050IResult {
    // Trade-or-business gate: only T/B recipients trigger § 6050I.
    if !input.recipient_engaged_in_trade_or_business {
        return Section6050IResult {
            payment_qualifies_as_cash: false,
            digital_asset_currently_excluded: input.payment_type == PaymentType::DigitalAsset,
            aggregate_exceeds_threshold: false,
            form_8300_required: false,
            statutory_filing_deadline_days: 15,
            within_15_day_window: true,
            violation: ViolationType::None,
            maximum_per_violation_penalty_cents: 0,
            criminal_exposure: false,
            citation: "26 U.S.C. § 6050I(a) — applies only to persons engaged in a trade or business",
            note: "Recipient is not engaged in a trade or business. § 6050I does not apply.".to_string(),
        };
    }

    // Cash classification under § 6050I(d).
    let qualifies_as_cash = match input.payment_type {
        PaymentType::Currency => true,
        PaymentType::CashiersCheck
        | PaymentType::MoneyOrder
        | PaymentType::BankDraft => {
            // Face amount must be ≤ $10,000 for the instrument to be
            // treated as cash (§ 6050I(d)(2)).
            input.single_instrument_face_amount_cents <= 1000000
        }
        PaymentType::PersonalCheck | PaymentType::WireTransfer | PaymentType::OtherNonCash => {
            false
        }
        PaymentType::DigitalAsset => {
            // IIJA added digital assets but IRS Announcement 2024-04
            // suspended implementation. Currently EXCLUDED.
            false
        }
    };
    let digital_asset_excluded = input.payment_type == PaymentType::DigitalAsset;

    if !qualifies_as_cash {
        return Section6050IResult {
            payment_qualifies_as_cash: false,
            digital_asset_currently_excluded: digital_asset_excluded,
            aggregate_exceeds_threshold: false,
            form_8300_required: false,
            statutory_filing_deadline_days: 15,
            within_15_day_window: true,
            violation: ViolationType::None,
            maximum_per_violation_penalty_cents: 0,
            criminal_exposure: false,
            citation: if digital_asset_excluded {
                "26 U.S.C. § 6050I(d) + IIJA § 80603 + IRS Announcement 2024-04 — digital assets currently EXCLUDED from § 6050I cash definition pending IRS regulations"
            } else {
                "26 U.S.C. § 6050I(d) — payment type does not qualify as 'cash' (personal checks + wire transfers + cashier's checks > $10K not cash)"
            },
            note: format!(
                "Payment type {:?} does not qualify as cash under § 6050I(d).{}",
                input.payment_type,
                if digital_asset_excluded {
                    " Per IRS Announcement 2024-04, digital assets are currently excluded from § 6050I until IRS regulations are finalized."
                } else {
                    ""
                },
            ),
        };
    }

    // $10,000 strict-greater-than threshold (§ 6050I(a)).
    let exceeds = input.aggregate_related_24_hour_amount_cents > 1000000;
    if !exceeds {
        return Section6050IResult {
            payment_qualifies_as_cash: true,
            digital_asset_currently_excluded: digital_asset_excluded,
            aggregate_exceeds_threshold: false,
            form_8300_required: false,
            statutory_filing_deadline_days: 15,
            within_15_day_window: true,
            violation: ViolationType::None,
            maximum_per_violation_penalty_cents: 0,
            criminal_exposure: false,
            citation: "26 U.S.C. § 6050I(a) — $10,000 strict-greater-than threshold required (≤ $10,000 not reportable)",
            note: format!(
                "Aggregate cash {} cents does not exceed the $10,000 strict-greater-than threshold. No Form 8300 required.",
                input.aggregate_related_24_hour_amount_cents
            ),
        };
    }

    // Form 8300 required. Now check timing.
    let within_window = input.days_since_transaction <= 15;
    let max_penalty = if input.intentional_disregard {
        // § 6721(e) intentional disregard: greater of $250,000 or
        // aggregate amount required to be reported (here represented
        // as the cash aggregate).
        (25_000_000_i64).max(input.aggregate_related_24_hour_amount_cents)
    } else {
        // Standard tier penalty (§ 6721(a)) — $310/violation in 2026
        // (inflation-adjusted). Modeled at the standard tier without
        // the inflation-indexed exact figure.
        31_000
    };

    if !input.form_8300_filed {
        return Section6050IResult {
            payment_qualifies_as_cash: true,
            digital_asset_currently_excluded: digital_asset_excluded,
            aggregate_exceeds_threshold: true,
            form_8300_required: true,
            statutory_filing_deadline_days: 15,
            within_15_day_window: within_window,
            violation: ViolationType::FormNotFiled,
            maximum_per_violation_penalty_cents: max_penalty,
            criminal_exposure: input.intentional_disregard,
            citation: "26 U.S.C. § 6050I(b) — Form 8300 must be filed within 15 days; § 6721 information-reporting penalty; § 7203 willful-failure criminal exposure",
            note: format!(
                "Aggregate {} cents exceeds $10,000 threshold; Form 8300 NOT filed. {} maximum penalty {} cents.",
                input.aggregate_related_24_hour_amount_cents,
                if input.intentional_disregard {
                    "INTENTIONAL DISREGARD — § 6721(e) uncapped"
                } else {
                    "§ 6721(a) standard tier"
                },
                max_penalty
            ),
        };
    }

    if !within_window {
        return Section6050IResult {
            payment_qualifies_as_cash: true,
            digital_asset_currently_excluded: digital_asset_excluded,
            aggregate_exceeds_threshold: true,
            form_8300_required: true,
            statutory_filing_deadline_days: 15,
            within_15_day_window: false,
            violation: ViolationType::FiledLate,
            maximum_per_violation_penalty_cents: max_penalty,
            criminal_exposure: input.intentional_disregard,
            citation: "26 U.S.C. § 6050I(b) — Form 8300 filed past 15-day deadline; § 6721 late-filing penalty applies",
            note: format!(
                "Form 8300 filed but {} days after transaction (exceeds 15-day deadline). § 6721 late-filing penalty.",
                input.days_since_transaction
            ),
        };
    }

    Section6050IResult {
        payment_qualifies_as_cash: true,
        digital_asset_currently_excluded: digital_asset_excluded,
        aggregate_exceeds_threshold: true,
        form_8300_required: true,
        statutory_filing_deadline_days: 15,
        within_15_day_window: true,
        violation: ViolationType::None,
        maximum_per_violation_penalty_cents: 0,
        criminal_exposure: false,
        citation: "26 U.S.C. § 6050I(a)/(b) — compliance OK: Form 8300 filed within 15-day deadline",
        note: format!(
            "§ 6050I compliance OK. Aggregate {} cents reported on Form 8300 within {} days of transaction.",
            input.aggregate_related_24_hour_amount_cents, input.days_since_transaction
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        in_business: bool,
        pt: PaymentType,
        single_face: i64,
        aggregate: i64,
        days: u32,
        filed: bool,
        intentional: bool,
    ) -> Section6050IInput {
        Section6050IInput {
            recipient_engaged_in_trade_or_business: in_business,
            payment_type: pt,
            single_instrument_face_amount_cents: single_face,
            aggregate_related_24_hour_amount_cents: aggregate,
            days_since_transaction: days,
            form_8300_filed: filed,
            intentional_disregard: intentional,
        }
    }

    #[test]
    fn currency_15k_in_business_filed_compliant() {
        let r = compute(&input(
            true,
            PaymentType::Currency,
            1500000,
            1500000,
            5,
            true,
            false,
        ));
        assert!(r.payment_qualifies_as_cash);
        assert!(r.form_8300_required);
        assert_eq!(r.violation, ViolationType::None);
    }

    #[test]
    fn not_in_business_no_obligation() {
        let r = compute(&input(
            false,
            PaymentType::Currency,
            1500000,
            1500000,
            5,
            false,
            false,
        ));
        assert!(!r.form_8300_required);
        assert_eq!(r.violation, ViolationType::None);
        assert!(r.citation.contains("trade or business"));
    }

    #[test]
    fn currency_at_10k_boundary_NOT_reportable() {
        // § 6050I requires strict GREATER THAN $10,000.
        let r = compute(&input(
            true,
            PaymentType::Currency,
            1000000,
            1000000,
            5,
            false,
            false,
        ));
        assert!(!r.form_8300_required);
        assert_eq!(r.violation, ViolationType::None);
    }

    #[test]
    fn one_cent_above_10k_reportable() {
        let r = compute(&input(
            true,
            PaymentType::Currency,
            1000001,
            1000001,
            5,
            true,
            false,
        ));
        assert!(r.form_8300_required);
    }

    #[test]
    fn cashiers_check_under_10k_face_qualifies_as_cash() {
        // Cashier's check face $9,500 — qualifies as cash. Aggregate
        // including other cash brings it over $10K.
        let r = compute(&input(
            true,
            PaymentType::CashiersCheck,
            950000,
            1500000,
            5,
            true,
            false,
        ));
        assert!(r.payment_qualifies_as_cash);
        assert!(r.form_8300_required);
    }

    #[test]
    fn cashiers_check_above_10k_face_NOT_cash() {
        // Cashier's check face $11,000 — exceeds face limit; NOT cash.
        let r = compute(&input(
            true,
            PaymentType::CashiersCheck,
            1100000,
            1100000,
            5,
            false,
            false,
        ));
        assert!(!r.payment_qualifies_as_cash);
        assert!(!r.form_8300_required);
    }

    #[test]
    fn personal_check_NOT_cash() {
        let r = compute(&input(
            true,
            PaymentType::PersonalCheck,
            5000000,
            5000000,
            5,
            false,
            false,
        ));
        assert!(!r.payment_qualifies_as_cash);
    }

    #[test]
    fn wire_transfer_NOT_cash() {
        let r = compute(&input(
            true,
            PaymentType::WireTransfer,
            10000000,
            10000000,
            5,
            false,
            false,
        ));
        assert!(!r.payment_qualifies_as_cash);
    }

    #[test]
    fn digital_asset_currently_excluded_per_announcement_2024_04() {
        // IIJA added digital assets but IRS Announcement 2024-04 paused
        // implementation. Currently EXCLUDED from § 6050I cash.
        let r = compute(&input(
            true,
            PaymentType::DigitalAsset,
            5000000,
            5000000,
            5,
            false,
            false,
        ));
        assert!(!r.payment_qualifies_as_cash);
        assert!(r.digital_asset_currently_excluded);
        assert!(!r.form_8300_required);
        assert!(r.citation.contains("Announcement 2024-04"));
        assert!(r.citation.contains("digital assets currently EXCLUDED"));
    }

    #[test]
    fn related_24_hour_aggregation_triggers() {
        // Single payment $6,000 (below threshold), but related 24-hour
        // aggregate $12,000 → reportable.
        let r = compute(&input(
            true,
            PaymentType::Currency,
            600000,
            1200000,
            5,
            true,
            false,
        ));
        assert!(r.form_8300_required);
    }

    #[test]
    fn form_not_filed_violation() {
        let r = compute(&input(
            true,
            PaymentType::Currency,
            1500000,
            1500000,
            5,
            false,
            false,
        ));
        assert_eq!(r.violation, ViolationType::FormNotFiled);
        assert!(r.citation.contains("§ 6050I(b)"));
        // Standard tier penalty.
        assert_eq!(r.maximum_per_violation_penalty_cents, 310_00);
    }

    #[test]
    fn intentional_disregard_unlimited_penalty() {
        // § 6721(e) intentional disregard: greater of $250K or aggregate.
        let r = compute(&input(
            true,
            PaymentType::Currency,
            50000000,
            50000000,
            5,
            false,
            true,
        ));
        assert_eq!(r.maximum_per_violation_penalty_cents, 50000000);
        assert!(r.criminal_exposure);
        assert!(r.citation.contains("§ 7203"));
    }

    #[test]
    fn intentional_disregard_250k_floor() {
        // For aggregate < $250K, penalty floor is $250K.
        let r = compute(&input(
            true,
            PaymentType::Currency,
            5000000,
            5000000,
            5,
            false,
            true,
        ));
        assert_eq!(r.maximum_per_violation_penalty_cents, 25000000);
    }

    #[test]
    fn late_filing_violation_at_16_days() {
        let r = compute(&input(
            true,
            PaymentType::Currency,
            1500000,
            1500000,
            16,
            true,
            false,
        ));
        assert_eq!(r.violation, ViolationType::FiledLate);
        assert!(!r.within_15_day_window);
    }

    #[test]
    fn at_15_day_boundary_compliant() {
        let r = compute(&input(
            true,
            PaymentType::Currency,
            1500000,
            1500000,
            15,
            true,
            false,
        ));
        assert_eq!(r.violation, ViolationType::None);
        assert!(r.within_15_day_window);
    }

    #[test]
    fn at_16_day_boundary_late() {
        let r = compute(&input(
            true,
            PaymentType::Currency,
            1500000,
            1500000,
            16,
            true,
            false,
        ));
        assert_eq!(r.violation, ViolationType::FiledLate);
    }

    #[test]
    fn money_order_face_limit_applies() {
        let r_under = compute(&input(
            true,
            PaymentType::MoneyOrder,
            900000,
            1200000,
            5,
            true,
            false,
        ));
        let r_over = compute(&input(
            true,
            PaymentType::MoneyOrder,
            1100000,
            1100000,
            5,
            false,
            false,
        ));
        assert!(r_under.payment_qualifies_as_cash);
        assert!(!r_over.payment_qualifies_as_cash);
    }

    #[test]
    fn bank_draft_face_limit_applies() {
        let r_under = compute(&input(
            true,
            PaymentType::BankDraft,
            500000,
            1500000,
            5,
            true,
            false,
        ));
        let r_over = compute(&input(
            true,
            PaymentType::BankDraft,
            2000000,
            2000000,
            5,
            false,
            false,
        ));
        assert!(r_under.payment_qualifies_as_cash);
        assert!(!r_over.payment_qualifies_as_cash);
    }

    #[test]
    fn standard_15_day_deadline() {
        let r = compute(&input(
            true,
            PaymentType::Currency,
            1500000,
            1500000,
            5,
            true,
            false,
        ));
        assert_eq!(r.statutory_filing_deadline_days, 15);
    }

    #[test]
    fn citations_pin_correct_authorities() {
        let r_currency = compute(&input(
            true,
            PaymentType::Currency,
            1500000,
            1500000,
            5,
            true,
            false,
        ));
        assert!(r_currency.citation.contains("§ 6050I"));

        let r_no_file = compute(&input(
            true,
            PaymentType::Currency,
            1500000,
            1500000,
            5,
            false,
            false,
        ));
        assert!(r_no_file.citation.contains("§ 6721"));
        assert!(r_no_file.citation.contains("§ 7203"));

        let r_digital = compute(&input(
            true,
            PaymentType::DigitalAsset,
            5000000,
            5000000,
            5,
            false,
            false,
        ));
        assert!(r_digital.citation.contains("IIJA § 80603"));
        assert!(r_digital.citation.contains("Announcement 2024-04"));
    }

    #[test]
    fn no_business_digital_asset_still_excluded() {
        // Digital asset is excluded regardless of T/B status.
        let r = compute(&input(
            false,
            PaymentType::DigitalAsset,
            5000000,
            5000000,
            5,
            false,
            false,
        ));
        assert!(r.digital_asset_currently_excluded);
    }

    #[test]
    fn negative_inputs_handled() {
        let r = compute(&input(
            true,
            PaymentType::Currency,
            -1,
            -1,
            0,
            true,
            false,
        ));
        assert!(!r.form_8300_required);
    }

    #[test]
    fn currency_aggregate_exceeds_threshold_flag() {
        let r = compute(&input(
            true,
            PaymentType::Currency,
            1500000,
            1500000,
            5,
            true,
            false,
        ));
        assert!(r.aggregate_exceeds_threshold);

        let r_under = compute(&input(
            true,
            PaymentType::Currency,
            800000,
            800000,
            5,
            false,
            false,
        ));
        assert!(!r_under.aggregate_exceeds_threshold);
    }
}
