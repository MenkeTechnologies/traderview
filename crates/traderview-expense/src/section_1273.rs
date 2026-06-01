//! IRC § 1273 — Determination of amount of and recognition of
//! original issue discount (OID).
//!
//! The DEFINITIONAL anchor for the OID cluster. § 1273(a) defines
//! OID and the de minimis safe harbor; § 1273(b) specifies how to
//! determine "issue price" depending on the issuance context. Both
//! § 1272 (current inclusion) and § 1278 (market discount + § 1278(b)
//! election) cross-reference § 1273 for the underlying numbers.
//!
//! Direct companion to:
//!   - `section_1271` (retirement of debt).
//!   - `section_1272` (current inclusion of OID).
//!   - `section_1276` (market discount — ordinary recharacterization).
//!   - `section_1277` (deferral of interest deduction).
//!   - `section_1278` (definitions + § 1278(b) election).
//!
//! § 1273(a) — three operative paragraphs:
//!
//!   § 1273(a)(1) — OID = excess (if any) of (A) stated redemption
//!     price at maturity (SRPM) over (B) issue price.
//!
//!   § 1273(a)(2) — Stated redemption price at maturity = amount
//!     fixed by the last modification of the purchase agreement
//!     (basically face value plus any premium payable at maturity).
//!
//!   § 1273(a)(3) — DE MINIMIS RULE: if OID is less than ¼ of 1%
//!     of SRPM × complete years to maturity, OID is treated as
//!     ZERO. Same formula as the § 1278(a)(2)(C) market-discount
//!     de minimis rule.
//!
//! § 1273(b) — four issue-price determination paths:
//!
//!   § 1273(b)(1) — PUBLICLY OFFERED for cash: issue price =
//!     initial offering price to the public at which a substantial
//!     amount of the debt is sold.
//!
//!   § 1273(b)(2) — NON-PUBLIC issued for cash: issue price =
//!     price paid by the first buyer of the debt instrument.
//!
//!   § 1273(b)(3) — TRADED DEBT (issued for property where either
//!     the debt or the property is publicly traded): issue price
//!     = fair market value of the debt instrument.
//!
//!   § 1273(b)(4) — RESIDUAL cases (none of (b)(1)–(b)(3)): issue
//!     price = stated redemption price minus OID (caller supplies
//!     the OID amount through other Code provisions, typically the
//!     AFR-imputed OID under § 1274).
//!
//!   § 1273(b)(5) — "Property" includes services and the right to
//!     use property.
//!
//! Citations: 26 U.S.C. § 1273(a)(1) (OID definition); § 1273(a)(2)
//! (stated redemption price); § 1273(a)(3) (de minimis ¼ of 1% per
//! year); § 1273(b)(1) (publicly offered cash issue price);
//! § 1273(b)(2) (non-public cash first-buyer price); § 1273(b)(3)
//! (traded debt FMV); § 1273(b)(4) (residual cases SRPM − OID);
//! § 1273(b)(5) (property definition includes services); § 1272
//! (current OID inclusion); § 1278 (market discount cross-reference);
//! § 1274 (AFR-imputed OID feeding § 1273(b)(4) residual path).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IssueClass {
    /// § 1273(b)(1) — publicly offered for cash. Issue price =
    /// initial offering price to the public at which a substantial
    /// amount of the debt is sold.
    PubliclyOffered,
    /// § 1273(b)(2) — non-public, issued for cash. Issue price =
    /// price paid by the first buyer.
    NonPublicCash,
    /// § 1273(b)(3) — traded debt issued for property (debt or
    /// property publicly traded). Issue price = FMV of debt
    /// instrument.
    TradedDebt,
    /// § 1273(b)(4) — residual cases. Issue price = stated
    /// redemption price minus OID (caller supplies the OID
    /// amount, typically from § 1274 AFR imputation).
    Residual,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section1273Input {
    pub issue_class: IssueClass,
    /// § 1273(a)(2) stated redemption price at maturity (SRPM) in
    /// cents.
    pub stated_redemption_price_at_maturity_cents: i64,
    /// Complete years to maturity at issuance — denominator factor
    /// in the § 1273(a)(3) de minimis threshold.
    pub years_to_maturity: u32,
    /// § 1273(b)(1) — initial offering price to the public (cents).
    pub initial_public_offering_price_cents: i64,
    /// § 1273(b)(2) — non-public cash first-buyer price (cents).
    pub first_buyer_price_cents: i64,
    /// § 1273(b)(3) — fair market value of the debt instrument
    /// (cents).
    pub fmv_of_debt_instrument_cents: i64,
    /// § 1273(b)(4) — caller-supplied OID amount for the residual
    /// path (cents). Issue price = SRPM − this amount.
    pub residual_oid_amount_cents: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section1273Result {
    /// § 1273(b) issue price (cents) determined according to the
    /// issue class.
    pub issue_price_cents: i64,
    /// § 1273(a)(1) raw OID = SRPM − issue price (clamped at 0).
    pub raw_oid_cents: i64,
    /// § 1273(a)(3) de minimis threshold (cents): SRPM × 0.0025
    /// × years to maturity.
    pub de_minimis_threshold_cents: i64,
    /// True if raw OID is STRICTLY LESS THAN de minimis threshold
    /// → treated as zero under § 1273(a)(3).
    pub de_minimis_applies: bool,
    /// Statutory OID after the de minimis rule (cents). Zero if
    /// de_minimis_applies; otherwise equals raw_oid.
    pub statutory_oid_cents: i64,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

/// § 1273(a)(3) de minimis factor — ¼ of 1% = 0.0025 per year.
pub const DE_MINIMIS_NUMERATOR: i64 = 25;
pub const DE_MINIMIS_DENOMINATOR: i64 = 10_000;

pub fn compute(input: &Section1273Input) -> Section1273Result {
    let mut notes: Vec<String> = Vec::new();

    let srpm = input.stated_redemption_price_at_maturity_cents.max(0);

    // § 1273(b) — issue price determination per class.
    let (issue_price, issue_price_note, class_citation): (i64, &'static str, &'static str) =
        match input.issue_class {
            IssueClass::PubliclyOffered => (
                input.initial_public_offering_price_cents.max(0),
                "§ 1273(b)(1) — publicly offered for cash; issue price = initial offering price \
                 to the public at which a substantial amount of the debt is sold.",
                "26 U.S.C. § 1273(b)(1) (publicly offered debt — initial offering price)",
            ),
            IssueClass::NonPublicCash => (
                input.first_buyer_price_cents.max(0),
                "§ 1273(b)(2) — non-public debt issued for cash; issue price = price paid by \
                 the first buyer of the debt instrument.",
                "26 U.S.C. § 1273(b)(2) (non-public cash — first-buyer price)",
            ),
            IssueClass::TradedDebt => (
                input.fmv_of_debt_instrument_cents.max(0),
                "§ 1273(b)(3) — traded debt issued for property (debt or property publicly \
                 traded); issue price = fair market value of the debt instrument.",
                "26 U.S.C. § 1273(b)(3) (traded debt — FMV); § 1273(b)(5) (property definition \
                 includes services)",
            ),
            IssueClass::Residual => (
                srpm.saturating_sub(input.residual_oid_amount_cents.max(0)).max(0),
                "§ 1273(b)(4) — residual case; issue price = stated redemption price minus OID. \
                 Caller-supplied OID amount typically reflects § 1274 AFR-imputed OID.",
                "26 U.S.C. § 1273(b)(4) (residual case — SRPM − OID); § 1274 (AFR-imputed OID \
                 feeding residual path)",
            ),
        };

    notes.push(issue_price_note.to_string());

    // § 1273(a)(1) raw OID = SRPM − issue_price (clamp at 0).
    let raw_oid = srpm.saturating_sub(issue_price).max(0);

    if raw_oid == 0 {
        notes.push(
            "§ 1273(a)(1) — issue price ≥ stated redemption price at maturity; no original \
             issue discount."
                .to_string(),
        );
        return Section1273Result {
            issue_price_cents: issue_price,
            raw_oid_cents: 0,
            de_minimis_threshold_cents: 0,
            de_minimis_applies: false,
            statutory_oid_cents: 0,
            citation: "26 U.S.C. § 1273(a)(1) (OID definition — no OID where issue price ≥ \
                       stated redemption price)",
            notes,
        };
    }

    // § 1273(a)(3) de minimis threshold = SRPM × 0.0025 × years.
    let de_minimis_threshold = srpm
        .saturating_mul(DE_MINIMIS_NUMERATOR)
        .saturating_mul(input.years_to_maturity as i64)
        / DE_MINIMIS_DENOMINATOR;

    let de_minimis_applies = raw_oid < de_minimis_threshold;
    let statutory_oid = if de_minimis_applies { 0 } else { raw_oid };

    if de_minimis_applies {
        notes.push(format!(
            "§ 1273(a)(3) de minimis rule applies — raw OID of {} cents is less than \
             threshold of {} cents (SRPM × ¼ of 1% × {} years); OID treated as ZERO.",
            raw_oid, de_minimis_threshold, input.years_to_maturity,
        ));
    }

    let combined_citation: &'static str = match input.issue_class {
        IssueClass::PubliclyOffered => {
            "26 U.S.C. § 1273(a)(1) (OID definition — SRPM excess over issue price); \
             § 1273(a)(2) (stated redemption price at maturity); § 1273(a)(3) (de minimis \
             ¼ of 1% per year); § 1273(b)(1) (publicly offered debt — initial offering price)"
        }
        IssueClass::NonPublicCash => {
            "26 U.S.C. § 1273(a)(1) (OID definition); § 1273(a)(2) (stated redemption price); \
             § 1273(a)(3) (de minimis); § 1273(b)(2) (non-public cash — first-buyer price)"
        }
        IssueClass::TradedDebt => {
            "26 U.S.C. § 1273(a)(1) (OID definition); § 1273(a)(2) (stated redemption price); \
             § 1273(a)(3) (de minimis); § 1273(b)(3) (traded debt — FMV); § 1273(b)(5) \
             (property definition includes services)"
        }
        IssueClass::Residual => {
            "26 U.S.C. § 1273(a)(1) (OID definition); § 1273(a)(2) (stated redemption price); \
             § 1273(a)(3) (de minimis); § 1273(b)(4) (residual case — SRPM − OID); § 1274 \
             (AFR-imputed OID feeding residual path)"
        }
    };

    let _ = class_citation;

    Section1273Result {
        issue_price_cents: issue_price,
        raw_oid_cents: raw_oid,
        de_minimis_threshold_cents: de_minimis_threshold,
        de_minimis_applies,
        statutory_oid_cents: statutory_oid,
        citation: combined_citation,
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        class: IssueClass,
        srpm: i64,
        years: u32,
        ipo: i64,
        first_buyer: i64,
        fmv: i64,
        residual_oid: i64,
    ) -> Section1273Input {
        Section1273Input {
            issue_class: class,
            stated_redemption_price_at_maturity_cents: srpm,
            years_to_maturity: years,
            initial_public_offering_price_cents: ipo,
            first_buyer_price_cents: first_buyer,
            fmv_of_debt_instrument_cents: fmv,
            residual_oid_amount_cents: residual_oid,
        }
    }

    // ── § 1273(a)(1) basic OID math ─────────────────────────────

    #[test]
    fn publicly_offered_srpm_1000_ipo_850_5yr_oid_150() {
        // OID = 1000 − 850 = 150 cents = $1.50. SRPM 100_000 cents,
        // IPO 85_000 cents → OID 15_000 cents. De minimis threshold
        // = 100_000 × 25 × 5 / 10_000 = 1_250. 15_000 > 1_250.
        let r = compute(&input(
            IssueClass::PubliclyOffered,
            100_000,
            5,
            85_000,
            0,
            0,
            0,
        ));
        assert_eq!(r.issue_price_cents, 85_000);
        assert_eq!(r.raw_oid_cents, 15_000);
        assert_eq!(r.de_minimis_threshold_cents, 1_250);
        assert!(!r.de_minimis_applies);
        assert_eq!(r.statutory_oid_cents, 15_000);
        assert!(r.citation.contains("§ 1273(a)(1)"));
        assert!(r.citation.contains("§ 1273(b)(1)"));
    }

    // ── § 1273(a)(3) de minimis ─────────────────────────────────

    #[test]
    fn de_minimis_below_threshold_zero_oid() {
        // SRPM 100_000, IPO 99_988 → raw 12. Threshold 1_250.
        // 12 < 1_250 → de minimis.
        let r = compute(&input(
            IssueClass::PubliclyOffered,
            100_000,
            5,
            99_988,
            0,
            0,
            0,
        ));
        assert_eq!(r.raw_oid_cents, 12);
        assert!(r.de_minimis_applies);
        assert_eq!(r.statutory_oid_cents, 0);
    }

    #[test]
    fn de_minimis_at_threshold_boundary_does_not_apply() {
        // Raw exactly 1_250 — statute requires "less than" → NOT de
        // minimis.
        let r = compute(&input(
            IssueClass::PubliclyOffered,
            100_000,
            5,
            98_750,
            0,
            0,
            0,
        ));
        assert_eq!(r.raw_oid_cents, 1_250);
        assert_eq!(r.de_minimis_threshold_cents, 1_250);
        assert!(!r.de_minimis_applies);
        assert_eq!(r.statutory_oid_cents, 1_250);
    }

    #[test]
    fn de_minimis_just_below_threshold_applies() {
        let r = compute(&input(
            IssueClass::PubliclyOffered,
            100_000,
            5,
            98_751,
            0,
            0,
            0,
        ));
        assert_eq!(r.raw_oid_cents, 1_249);
        assert!(r.de_minimis_applies);
        assert_eq!(r.statutory_oid_cents, 0);
    }

    #[test]
    fn de_minimis_scales_with_years() {
        let one = compute(&input(
            IssueClass::PubliclyOffered,
            100_000,
            1,
            99_000,
            0,
            0,
            0,
        ));
        let twenty = compute(&input(
            IssueClass::PubliclyOffered,
            100_000,
            20,
            99_000,
            0,
            0,
            0,
        ));
        assert_eq!(one.de_minimis_threshold_cents, 250);
        assert_eq!(twenty.de_minimis_threshold_cents, 5_000);
        // Raw 1_000. 1-year: 1_000 > 250 → not de minimis.
        // 20-year: 1_000 < 5_000 → de minimis.
        assert!(!one.de_minimis_applies);
        assert!(twenty.de_minimis_applies);
    }

    // ── § 1273(b)(1) publicly offered ───────────────────────────

    #[test]
    fn publicly_offered_uses_ipo_price() {
        let r = compute(&input(
            IssueClass::PubliclyOffered,
            100_000,
            5,
            85_000,
            70_000, // ignored
            60_000, // ignored
            0,
        ));
        assert_eq!(r.issue_price_cents, 85_000);
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("§ 1273(b)(1)") && n.contains("initial offering"))
        );
    }

    // ── § 1273(b)(2) non-public cash ───────────────────────────

    #[test]
    fn non_public_cash_uses_first_buyer_price() {
        let r = compute(&input(
            IssueClass::NonPublicCash,
            100_000,
            5,
            99_000, // ignored
            85_000,
            60_000, // ignored
            0,
        ));
        assert_eq!(r.issue_price_cents, 85_000);
        assert_eq!(r.raw_oid_cents, 15_000);
        assert!(r.citation.contains("§ 1273(b)(2)"));
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("§ 1273(b)(2)") && n.contains("first buyer"))
        );
    }

    // ── § 1273(b)(3) traded debt FMV ───────────────────────────

    #[test]
    fn traded_debt_uses_fmv() {
        let r = compute(&input(
            IssueClass::TradedDebt,
            100_000,
            5,
            99_000, // ignored
            85_000, // ignored
            80_000,
            0,
        ));
        assert_eq!(r.issue_price_cents, 80_000);
        assert_eq!(r.raw_oid_cents, 20_000);
        assert!(r.citation.contains("§ 1273(b)(3)"));
        assert!(r.citation.contains("§ 1273(b)(5)"));
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("§ 1273(b)(3)") && n.contains("FMV") || n.contains("fair market value"))
        );
    }

    // ── § 1273(b)(4) residual ─────────────────────────────────

    #[test]
    fn residual_uses_srpm_minus_caller_oid() {
        // SRPM 100_000, residual OID 15_000 → IP 85_000.
        let r = compute(&input(
            IssueClass::Residual,
            100_000,
            5,
            99_000, // ignored
            85_000, // ignored
            80_000, // ignored
            15_000,
        ));
        assert_eq!(r.issue_price_cents, 85_000);
        assert_eq!(r.raw_oid_cents, 15_000);
        assert!(r.citation.contains("§ 1273(b)(4)"));
        assert!(r.citation.contains("§ 1274"));
    }

    #[test]
    fn residual_caller_oid_exceeds_srpm_clamps_at_zero_issue_price() {
        let r = compute(&input(
            IssueClass::Residual,
            100_000,
            5,
            0,
            0,
            0,
            150_000,
        ));
        assert_eq!(r.issue_price_cents, 0);
        assert_eq!(r.raw_oid_cents, 100_000);
    }

    // ── No-OID path ─────────────────────────────────────────────

    #[test]
    fn issue_price_equals_srpm_no_oid() {
        let r = compute(&input(
            IssueClass::PubliclyOffered,
            100_000,
            5,
            100_000,
            0,
            0,
            0,
        ));
        assert_eq!(r.raw_oid_cents, 0);
        assert_eq!(r.statutory_oid_cents, 0);
        assert!(r.citation.contains("no OID"));
    }

    #[test]
    fn issue_price_above_srpm_no_oid() {
        let r = compute(&input(
            IssueClass::PubliclyOffered,
            100_000,
            5,
            110_000, // premium issuance
            0,
            0,
            0,
        ));
        assert_eq!(r.raw_oid_cents, 0);
    }

    // ── Regression-critical invariants ──────────────────────────

    #[test]
    fn issue_class_determines_issue_price_source_invariant() {
        // Same SRPM + years; varying class-specific inputs.
        let publicly = compute(&input(
            IssueClass::PubliclyOffered,
            100_000,
            5,
            90_000,
            85_000,
            80_000,
            12_000,
        ));
        let non_pub = compute(&input(
            IssueClass::NonPublicCash,
            100_000,
            5,
            90_000,
            85_000,
            80_000,
            12_000,
        ));
        let traded = compute(&input(
            IssueClass::TradedDebt,
            100_000,
            5,
            90_000,
            85_000,
            80_000,
            12_000,
        ));
        let residual = compute(&input(
            IssueClass::Residual,
            100_000,
            5,
            90_000,
            85_000,
            80_000,
            12_000,
        ));
        assert_eq!(publicly.issue_price_cents, 90_000);
        assert_eq!(non_pub.issue_price_cents, 85_000);
        assert_eq!(traded.issue_price_cents, 80_000);
        assert_eq!(residual.issue_price_cents, 88_000); // 100_000 − 12_000
    }

    #[test]
    fn de_minimis_strictly_less_than_invariant() {
        let srpm = 100_000_i64;
        let years = 5_i64;
        let threshold = srpm * DE_MINIMIS_NUMERATOR * years / DE_MINIMIS_DENOMINATOR;
        for (raw_oid_target, expect_de_minimis) in [
            (threshold - 1, true),
            (threshold, false),
            (threshold + 1, false),
        ] {
            let ipo = srpm - raw_oid_target;
            let r = compute(&input(
                IssueClass::PubliclyOffered,
                srpm,
                years as u32,
                ipo,
                0,
                0,
                0,
            ));
            assert_eq!(
                r.de_minimis_applies, expect_de_minimis,
                "raw {} vs threshold {}: expected de_minimis={}",
                raw_oid_target, threshold, expect_de_minimis,
            );
        }
    }

    #[test]
    fn statutory_oid_zero_iff_either_no_raw_or_de_minimis_invariant() {
        for srpm in [100_000_i64, 500_000, 1_000_000] {
            for ipo in [50_000_i64, 99_000, 99_988, 100_000, 110_000] {
                for years in [1_u32, 5, 20] {
                    let r = compute(&input(
                        IssueClass::PubliclyOffered,
                        srpm,
                        years,
                        ipo,
                        0,
                        0,
                        0,
                    ));
                    if r.raw_oid_cents == 0 || r.de_minimis_applies {
                        assert_eq!(r.statutory_oid_cents, 0);
                    } else {
                        assert_eq!(r.statutory_oid_cents, r.raw_oid_cents);
                    }
                }
            }
        }
    }

    #[test]
    fn citation_pins_subsection_per_class() {
        let pub_off = compute(&input(
            IssueClass::PubliclyOffered,
            100_000,
            5,
            85_000,
            0,
            0,
            0,
        ));
        let non_pub = compute(&input(
            IssueClass::NonPublicCash,
            100_000,
            5,
            0,
            85_000,
            0,
            0,
        ));
        let traded = compute(&input(
            IssueClass::TradedDebt,
            100_000,
            5,
            0,
            0,
            80_000,
            0,
        ));
        let residual = compute(&input(
            IssueClass::Residual,
            100_000,
            5,
            0,
            0,
            0,
            15_000,
        ));
        assert!(pub_off.citation.contains("§ 1273(b)(1)"));
        assert!(non_pub.citation.contains("§ 1273(b)(2)"));
        assert!(traded.citation.contains("§ 1273(b)(3)"));
        assert!(residual.citation.contains("§ 1273(b)(4)"));
        // All non-zero-OID paths include § 1273(a)(1) + (a)(2) + (a)(3).
        for r in [&pub_off, &non_pub, &traded, &residual] {
            assert!(r.citation.contains("§ 1273(a)(1)"));
            assert!(r.citation.contains("§ 1273(a)(3)"));
        }
    }

    #[test]
    fn de_minimis_uses_same_factor_as_section_1278_invariant() {
        // § 1273(a)(3) and § 1278(a)(2)(C) share the ¼ of 1% per
        // year formula. Verify the constants.
        assert_eq!(DE_MINIMIS_NUMERATOR, 25);
        assert_eq!(DE_MINIMIS_DENOMINATOR, 10_000);
    }

    #[test]
    fn zero_years_to_maturity_zero_threshold_full_oid() {
        // Edge — 0 years → 0 threshold → no de minimis applies.
        let r = compute(&input(
            IssueClass::PubliclyOffered,
            100_000,
            0,
            85_000,
            0,
            0,
            0,
        ));
        assert_eq!(r.de_minimis_threshold_cents, 0);
        assert!(!r.de_minimis_applies);
        assert_eq!(r.statutory_oid_cents, 15_000);
    }

    #[test]
    fn class_specific_note_documents_issue_price_source() {
        let pub_off = compute(&input(
            IssueClass::PubliclyOffered,
            100_000,
            5,
            85_000,
            0,
            0,
            0,
        ));
        let non_pub = compute(&input(
            IssueClass::NonPublicCash,
            100_000,
            5,
            0,
            85_000,
            0,
            0,
        ));
        let traded = compute(&input(
            IssueClass::TradedDebt,
            100_000,
            5,
            0,
            0,
            80_000,
            0,
        ));
        let residual = compute(&input(
            IssueClass::Residual,
            100_000,
            5,
            0,
            0,
            0,
            15_000,
        ));
        assert!(
            pub_off
                .notes
                .iter()
                .any(|n| n.contains("§ 1273(b)(1)") && n.contains("initial offering"))
        );
        assert!(
            non_pub
                .notes
                .iter()
                .any(|n| n.contains("§ 1273(b)(2)") && n.contains("first buyer"))
        );
        assert!(
            traded
                .notes
                .iter()
                .any(|n| n.contains("§ 1273(b)(3)") && n.contains("fair market value"))
        );
        assert!(
            residual
                .notes
                .iter()
                .any(|n| n.contains("§ 1273(b)(4)") && n.contains("§ 1274"))
        );
    }
}
