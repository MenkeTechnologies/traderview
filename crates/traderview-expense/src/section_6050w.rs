//! IRC § 6050W — Returns relating to payments made in settlement of
//! payment card and third-party network transactions (Form 1099-K).
//!
//! 26 U.S.C. § 6050W requires Payment Settlement Entities (PSEs) to file
//! a Form 1099-K with the IRS reporting gross reportable transactions
//! for each payee. Two categories of PSE with very different thresholds:
//!
//! **§ 6050W(d)(1) Merchant Acquiring Entity** — banks and processors
//! that settle payment-card transactions (Stripe, Square, traditional
//! credit-card processors, Adyen, Worldpay, etc.). **No de minimis
//! threshold** — every dollar reportable.
//!
//! **§ 6050W(d)(3) Third-Party Settlement Organization (TPSO)** — PayPal,
//! Venmo, Cash App, Zelle, eBay, Etsy, StubHub, Airbnb, etc. The
//! threshold is the bouncing-ball provision:
//!
//! - **Pre-ARPA (through 2021)**: gross > $20,000 **AND** transactions > 200.
//! - **ARPA 2021 (would-have-been 2022+)**: gross > $600, no transaction floor.
//! - **IRS delays**: Notice 2023-74 delayed 2023 to $20K/200; Notice 2024-85
//!   transitional $5K for 2024, $2,500 for 2025.
//! - **OBBBA 2025 § 70432** — fully REVERSED the ARPA reduction
//!   retroactively. Threshold is **$20,000 AND 200 transactions** for ALL
//!   years 2025 and later, permanently. The transitional $5K/$2,500
//!   rules were superseded and never bind.
//!
//! Both prongs (dollar AND count) must be exceeded. Strict greater-than
//! on both — exactly $20,000 OR exactly 200 transactions does NOT
//! trigger 1099-K filing.
//!
//! Citations: 26 U.S.C. § 6050W; § 6050W(d)(1) (merchant acquiring entity
//! definition — no de minimis); § 6050W(d)(3) (TPSO definition);
//! § 6050W(e) (de minimis exception); OBBBA § 70432 (eff. 2025-01-01,
//! restoring $20,000 / 200 threshold retroactively); IRS Notice 2025-62
//! (FAQ confirming OBBBA reversion).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentSettlementEntity {
    /// Banks / processors settling payment-card transactions — Stripe,
    /// Square, Adyen, Worldpay, etc. (§ 6050W(d)(1)).
    MerchantAcquiringEntity,
    /// Third-Party Settlement Organization — PayPal, Venmo, Cash App,
    /// Zelle, eBay, Etsy, StubHub, Airbnb, etc. (§ 6050W(d)(3)).
    ThirdPartySettlementOrganization,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section6050WInput {
    pub entity_type: PaymentSettlementEntity,
    pub gross_amount_cents: i64,
    pub transaction_count: u32,
    /// Calendar year for which reporting applies. Drives the threshold
    /// regime selection.
    pub year: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ThresholdRegime {
    /// No de minimis — every dollar reportable. Merchant acquiring path.
    MerchantNoMinimum,
    /// Pre-ARPA original / post-OBBBA restored — > $20K AND > 200 txn.
    TwentyKAnd200Transactions,
    /// ARPA 2021 nominal — > $600, no transaction minimum. SUPERSEDED by
    /// OBBBA for 2025+ but pinned here for historical-year correctness.
    ArpaSixHundred,
    /// IRS Notice 2024-85 transitional $5K for 2024 only.
    Transitional5K2024,
    /// IRS Notice 2024-85 transitional $2,500 for 2025 — superseded by
    /// OBBBA § 70432 (retroactive). Pinned for completeness.
    Transitional2_5K2025,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section6050WResult {
    pub regime: ThresholdRegime,
    pub dollar_threshold_cents: i64,
    pub transaction_threshold: u32,
    pub gross_exceeds_dollar_threshold: bool,
    pub count_exceeds_transaction_threshold: bool,
    pub reporting_required: bool,
    pub citation: &'static str,
    pub note: String,
}

pub fn compute(input: &Section6050WInput) -> Section6050WResult {
    if matches!(
        input.entity_type,
        PaymentSettlementEntity::MerchantAcquiringEntity
    ) {
        let exceeds_dollar = input.gross_amount_cents > 0;
        return Section6050WResult {
            regime: ThresholdRegime::MerchantNoMinimum,
            dollar_threshold_cents: 0,
            transaction_threshold: 0,
            gross_exceeds_dollar_threshold: exceeds_dollar,
            count_exceeds_transaction_threshold: input.transaction_count > 0,
            reporting_required: exceeds_dollar,
            citation:
                "26 U.S.C. § 6050W(d)(1) — merchant acquiring entity; no de minimis threshold",
            note: format!(
                "Merchant acquiring entity has no de minimis threshold under § 6050W(d)(1). Every dollar of payment-card settlement to a payee is reportable; gross = {} cents.",
                input.gross_amount_cents
            ),
        };
    }

    let (regime, dollar_threshold, transaction_threshold, citation) =
        tpso_regime_for_year(input.year);

    let exceeds_dollar = input.gross_amount_cents > dollar_threshold;
    let exceeds_count = input.transaction_count > transaction_threshold;
    let reporting_required = if transaction_threshold == 0 {
        exceeds_dollar
    } else {
        exceeds_dollar && exceeds_count
    };

    let conjunction = if transaction_threshold == 0 { "" } else { " AND " };
    let count_part = if transaction_threshold == 0 {
        String::new()
    } else {
        format!(
            "{}transaction count > {} ({})",
            conjunction,
            transaction_threshold,
            if exceeds_count { "MET" } else { "NOT MET" }
        )
    };
    let note = format!(
        "TPSO threshold for year {} is {} cents{}. Test: gross ({}) > {} cents ({}){}{}. Reporting required: {}.",
        input.year,
        dollar_threshold,
        if transaction_threshold == 0 {
            String::new()
        } else {
            format!(" AND > {} transactions", transaction_threshold)
        },
        input.gross_amount_cents,
        dollar_threshold,
        if exceeds_dollar { "MET" } else { "NOT MET" },
        if transaction_threshold == 0 { "" } else { " AND " },
        count_part,
        reporting_required
    );

    Section6050WResult {
        regime,
        dollar_threshold_cents: dollar_threshold,
        transaction_threshold,
        gross_exceeds_dollar_threshold: exceeds_dollar,
        count_exceeds_transaction_threshold: exceeds_count,
        reporting_required,
        citation,
        note,
    }
}

fn tpso_regime_for_year(year: u32) -> (ThresholdRegime, i64, u32, &'static str) {
    // OBBBA § 70432 (eff. 2025-01-01) restored the $20K/200 threshold
    // RETROACTIVELY for all years 2025 and later. The ARPA $600 and
    // transitional $5K/$2,500 are pinned for pre-2025 historical
    // accuracy but do not bind 2025+.
    match year {
        y if y >= 2025 => (
            ThresholdRegime::TwentyKAnd200Transactions,
            2000000,
            200,
            "26 U.S.C. § 6050W(e) + OBBBA § 70432 — $20,000 AND 200 transactions threshold restored retroactively for 2025+",
        ),
        2024 => (
            ThresholdRegime::Transitional5K2024,
            500000,
            0,
            "26 U.S.C. § 6050W(e) + IRS Notice 2024-85 — transitional $5,000 threshold for 2024 (no transaction minimum)",
        ),
        2023 => (
            ThresholdRegime::TwentyKAnd200Transactions,
            2000000,
            200,
            "26 U.S.C. § 6050W(e) + IRS Notice 2023-74 — 2023 delayed ARPA reduction; $20,000 AND 200 transactions",
        ),
        // ARPA enacted March 2021. The reduced threshold applied to
        // returns for calendar years beginning after Dec. 31, 2021 (i.e.,
        // 2022) but IRS delayed all enforcement before OBBBA's full
        // repeal. For historical correctness pin 2022 to ARPA nominal.
        2022 => (
            ThresholdRegime::ArpaSixHundred,
            60000,
            0,
            "26 U.S.C. § 6050W(e) + ARPA 2021 — nominal $600 threshold (IRS Notice 2022-1 delayed enforcement; superseded by OBBBA for 2025+)",
        ),
        _ => (
            ThresholdRegime::TwentyKAnd200Transactions,
            2000000,
            200,
            "26 U.S.C. § 6050W(e) — pre-ARPA original $20,000 AND 200 transactions threshold",
        ),
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    fn input(
        entity: PaymentSettlementEntity,
        gross: i64,
        count: u32,
        year: u32,
    ) -> Section6050WInput {
        Section6050WInput {
            entity_type: entity,
            gross_amount_cents: gross,
            transaction_count: count,
            year,
        }
    }

    #[test]
    fn merchant_acquiring_no_minimum_one_dollar_reportable() {
        let r = compute(&input(
            PaymentSettlementEntity::MerchantAcquiringEntity,
            1_00,
            1,
            2026,
        ));
        assert!(r.reporting_required);
        assert_eq!(r.regime, ThresholdRegime::MerchantNoMinimum);
        assert!(r.citation.contains("§ 6050W(d)(1)"));
    }

    #[test]
    fn merchant_acquiring_zero_not_reportable() {
        let r = compute(&input(
            PaymentSettlementEntity::MerchantAcquiringEntity,
            0,
            0,
            2026,
        ));
        assert!(!r.reporting_required);
    }

    #[test]
    fn tpso_2026_obbba_threshold_25k_300tx_reportable() {
        let r = compute(&input(
            PaymentSettlementEntity::ThirdPartySettlementOrganization,
            2500000,
            300,
            2026,
        ));
        assert!(r.reporting_required);
        assert_eq!(r.regime, ThresholdRegime::TwentyKAnd200Transactions);
        assert!(r.citation.contains("OBBBA § 70432"));
    }

    #[test]
    fn tpso_2026_only_dollar_threshold_met_not_reportable() {
        // > $20K but only 200 transactions — both prongs required.
        let r = compute(&input(
            PaymentSettlementEntity::ThirdPartySettlementOrganization,
            2500000,
            200,
            2026,
        ));
        assert!(!r.reporting_required);
        assert!(r.gross_exceeds_dollar_threshold);
        assert!(!r.count_exceeds_transaction_threshold);
    }

    #[test]
    fn tpso_2026_only_transaction_threshold_met_not_reportable() {
        // 500 transactions but only $20K — both prongs required.
        let r = compute(&input(
            PaymentSettlementEntity::ThirdPartySettlementOrganization,
            2000000,
            500,
            2026,
        ));
        assert!(!r.reporting_required);
        assert!(!r.gross_exceeds_dollar_threshold);
        assert!(r.count_exceeds_transaction_threshold);
    }

    #[test]
    fn tpso_2026_exact_20k_and_200_NOT_reportable() {
        // Strict greater-than on both — exactly $20K AND exactly 200 fails.
        let r = compute(&input(
            PaymentSettlementEntity::ThirdPartySettlementOrganization,
            2000000,
            200,
            2026,
        ));
        assert!(!r.reporting_required);
        assert!(!r.gross_exceeds_dollar_threshold);
        assert!(!r.count_exceeds_transaction_threshold);
    }

    #[test]
    fn tpso_2026_one_cent_over_20k_and_201_tx_reportable() {
        let r = compute(&input(
            PaymentSettlementEntity::ThirdPartySettlementOrganization,
            2000001,
            201,
            2026,
        ));
        assert!(r.reporting_required);
    }

    #[test]
    fn tpso_2025_obbba_retroactive_supersedes_2_5k() {
        // OBBBA § 70432 retroactively reverted 2025 to $20K/200. The
        // transitional $2,500 rule from IRS Notice 2024-85 never binds.
        let r = compute(&input(
            PaymentSettlementEntity::ThirdPartySettlementOrganization,
            1000000,
            150,
            2025,
        ));
        assert_eq!(r.regime, ThresholdRegime::TwentyKAnd200Transactions);
        assert!(!r.reporting_required);
    }

    #[test]
    fn tpso_2024_transitional_5k_no_transaction_minimum() {
        let r = compute(&input(
            PaymentSettlementEntity::ThirdPartySettlementOrganization,
            500100,
            1,
            2024,
        ));
        assert_eq!(r.regime, ThresholdRegime::Transitional5K2024);
        assert!(r.reporting_required);
        assert_eq!(r.transaction_threshold, 0);
        assert!(r.citation.contains("Notice 2024-85"));
    }

    #[test]
    fn tpso_2024_under_5k_not_reportable() {
        let r = compute(&input(
            PaymentSettlementEntity::ThirdPartySettlementOrganization,
            499900,
            10_000,
            2024,
        ));
        assert!(!r.reporting_required);
    }

    #[test]
    fn tpso_2023_irs_notice_delayed_to_20k_200() {
        let r = compute(&input(
            PaymentSettlementEntity::ThirdPartySettlementOrganization,
            500000,
            201,
            2023,
        ));
        assert_eq!(r.regime, ThresholdRegime::TwentyKAnd200Transactions);
        assert!(!r.reporting_required, "$5K + 201 tx in 2023 below threshold");
        assert!(r.citation.contains("Notice 2023-74"));
    }

    #[test]
    fn tpso_2022_arpa_600_threshold_pinned() {
        let r = compute(&input(
            PaymentSettlementEntity::ThirdPartySettlementOrganization,
            601_00,
            1,
            2022,
        ));
        assert_eq!(r.regime, ThresholdRegime::ArpaSixHundred);
        assert!(r.reporting_required);
        assert_eq!(r.transaction_threshold, 0);
    }

    #[test]
    fn tpso_2021_pre_arpa_original_threshold() {
        let r = compute(&input(
            PaymentSettlementEntity::ThirdPartySettlementOrganization,
            2500000,
            201,
            2021,
        ));
        assert_eq!(r.regime, ThresholdRegime::TwentyKAnd200Transactions);
        assert!(r.reporting_required);
    }

    #[test]
    fn tpso_pre_arpa_one_prong_only_not_reportable() {
        let r = compute(&input(
            PaymentSettlementEntity::ThirdPartySettlementOrganization,
            2500000,
            150,
            2021,
        ));
        assert!(!r.reporting_required, "Pre-ARPA requires BOTH prongs > thresholds");
    }

    #[test]
    fn merchant_one_dollar_one_transaction_2024_no_threshold_check() {
        // Merchant acquiring path ignores year entirely — always no minimum.
        let r = compute(&input(
            PaymentSettlementEntity::MerchantAcquiringEntity,
            1_00,
            1,
            2024,
        ));
        assert!(r.reporting_required);
    }

    #[test]
    fn obbba_retroactive_for_2025_and_2026_and_later() {
        for year in [2025, 2026, 2027, 2030] {
            let r = compute(&input(
                PaymentSettlementEntity::ThirdPartySettlementOrganization,
                2000000,
                200,
                year,
            ));
            assert_eq!(
                r.regime,
                ThresholdRegime::TwentyKAnd200Transactions,
                "year {} should use OBBBA-restored threshold",
                year
            );
            assert_eq!(r.dollar_threshold_cents, 2000000);
            assert_eq!(r.transaction_threshold, 200);
        }
    }

    #[test]
    fn citations_pin_correct_authorities() {
        let m = compute(&input(
            PaymentSettlementEntity::MerchantAcquiringEntity,
            1_00,
            1,
            2026,
        ));
        assert!(m.citation.contains("§ 6050W(d)(1)"));

        let t_2026 = compute(&input(
            PaymentSettlementEntity::ThirdPartySettlementOrganization,
            2500000,
            300,
            2026,
        ));
        assert!(t_2026.citation.contains("§ 6050W(e)"));
        assert!(t_2026.citation.contains("OBBBA § 70432"));

        let t_2024 = compute(&input(
            PaymentSettlementEntity::ThirdPartySettlementOrganization,
            1000000,
            300,
            2024,
        ));
        assert!(t_2024.citation.contains("Notice 2024-85"));

        let t_2022 = compute(&input(
            PaymentSettlementEntity::ThirdPartySettlementOrganization,
            100000,
            5,
            2022,
        ));
        assert!(t_2022.citation.contains("ARPA"));
    }

    #[test]
    fn note_reports_both_prongs_when_applicable() {
        let r = compute(&input(
            PaymentSettlementEntity::ThirdPartySettlementOrganization,
            2500000,
            150,
            2026,
        ));
        assert!(r.note.contains("MET"));
        assert!(r.note.contains("NOT MET"));
        assert!(r.note.contains("AND"));
    }

    #[test]
    fn boundary_invariant_at_each_prong_for_2026() {
        // Both prongs must be STRICTLY exceeded.
        let cases = [
            (2000000, 200, false),     // both at threshold
            (2000000, 201, false),     // dollar at, count over
            (2000001, 200, false),     // dollar over, count at
            (2000001, 201, true),      // both over
            (1999999, 201, false),     // dollar under
            (2000001, 199, false),     // count under
        ];
        for (gross, count, expected) in cases {
            let r = compute(&input(
                PaymentSettlementEntity::ThirdPartySettlementOrganization,
                gross,
                count,
                2026,
            ));
            assert_eq!(
                r.reporting_required, expected,
                "gross={}, count={} should be {}",
                gross, count, expected
            );
        }
    }
}
