//! IRC § 1278 — Definitions and special rules for market discount
//! bonds.
//!
//! The definitional + election module that both `section_1276` and
//! `section_1277` cross-reference. Provides:
//!
//!   - § 1278(a)(1) — what counts as a "market discount bond"
//!     (carve-outs for short-term obligations, savings bonds,
//!     installment obligations).
//!   - § 1278(a)(2)(A) — market discount = excess of stated
//!     redemption price at maturity over the taxpayer's basis at
//!     acquisition.
//!   - § 1278(a)(2)(B) — OID bonds use REVISED ISSUE PRICE
//!     (acquisition-date issue price plus accrued OID) instead of
//!     stated redemption price.
//!   - § 1278(a)(2)(C) — DE MINIMIS RULE: discount less than ¼ of
//!     1% of stated redemption × complete years to maturity is
//!     treated as ZERO. The trader-friendly safe harbor.
//!   - § 1278(b)(1) — ELECTION to include accrued market discount
//!     currently each year. Switches off the § 1276 ordinary-
//!     income disposition recharacterization AND the § 1277
//!     interest-deduction deferral simultaneously.
//!   - § 1278(b)(2) — election applies to all market discount
//!     bonds acquired during or after the year of election.
//!   - § 1278(b)(3) — election is IRREVOCABLE unless the Secretary
//!     consents to revocation.
//!
//! De minimis math: threshold = stated_redemption × 0.0025 × years.
//! If raw discount is STRICTLY LESS THAN the threshold, market
//! discount is treated as zero. Equal or above → full raw discount.
//!
//! Three carve-outs from market-discount-bond status:
//!   - Short-term obligations (≤ 1 year to maturity at acquisition)
//!   - U.S. Series E / EE / I savings bonds
//!   - Installment obligations under § 453B
//!
//! Citations: 26 U.S.C. § 1278(a)(1) (market discount bond
//! definition with carve-outs); § 1278(a)(2)(A) (basic market
//! discount = redemption price minus basis); § 1278(a)(2)(B) (OID
//! revised-issue-price substitution); § 1278(a)(2)(C) (de minimis
//! ¼ of 1% per year); § 1278(b)(1) (current-inclusion election);
//! § 1278(b)(2) (election scope to all subsequent acquisitions);
//! § 1278(b)(3) (irrevocability absent Secretary's consent);
//! § 1276 + § 1277 (cross-referenced operative provisions); § 1272
//! (OID accrual feeding the § 1278(a)(2)(B) revised-issue-price
//! benchmark).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BondType {
    /// Regular bond — § 1278(a)(2)(A) basic market-discount math.
    Standard,
    /// Original-issue-discount bond — § 1278(a)(2)(B) uses revised
    /// issue price (acquisition-date OID-adjusted basis) instead
    /// of stated redemption price.
    OID,
    /// U.S. savings bond (Series E / EE / I) — § 1278(a)(1)
    /// carve-out. Not a market discount bond.
    SavingsBond,
    /// Short-term obligation (≤ 1 year to maturity at acquisition)
    /// — § 1278(a)(1) carve-out. Not a market discount bond.
    ShortTermObligation,
    /// Installment obligation under § 453B — § 1278(a)(1) carve-out.
    /// Not a market discount bond.
    InstallmentObligation,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section1278Input {
    pub bond_type: BondType,
    /// Face / stated redemption price at maturity (cents).
    pub stated_redemption_price_cents: i64,
    /// § 1278(a)(2)(B) revised issue price for OID bonds (cents).
    /// Used in lieu of stated_redemption_price for OID bonds.
    pub revised_issue_price_cents: i64,
    /// Taxpayer's basis (purchase price) at acquisition (cents).
    pub purchase_price_cents: i64,
    /// Complete years to maturity at acquisition. Used in the de
    /// minimis calculation under § 1278(a)(2)(C).
    pub years_to_maturity: u32,
    /// Whether the taxpayer has made the § 1278(b) current-
    /// inclusion election. Once made it is irrevocable absent
    /// Secretary's consent.
    pub election_made: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section1278Result {
    /// True if the bond qualifies as a "market discount bond" under
    /// § 1278(a)(1) (i.e., not in a § 1278(a)(1) carve-out).
    pub is_market_discount_bond: bool,
    /// Raw market discount before applying the § 1278(a)(2)(C) de
    /// minimis rule (cents). Equals max(0, redemption_basis -
    /// purchase_price).
    pub raw_market_discount_cents: i64,
    /// § 1278(a)(2)(C) de minimis threshold in cents:
    /// redemption_basis × 0.0025 × years_to_maturity.
    pub de_minimis_threshold_cents: i64,
    /// True if raw market discount is STRICTLY LESS THAN the de
    /// minimis threshold → treated as zero.
    pub de_minimis_applies: bool,
    /// Statutory market discount after the de minimis rule (cents).
    /// Zero if `de_minimis_applies`; otherwise equals raw.
    pub statutory_market_discount_cents: i64,
    /// Redemption basis actually used in the discount computation
    /// (cents). Stated redemption price for Standard bonds; revised
    /// issue price for OID bonds (§ 1278(a)(2)(B)).
    pub redemption_basis_for_calc_cents: i64,
    /// Whether § 1278(b) current-inclusion election is active.
    /// Switches off the § 1276 / § 1277 deferral pair.
    pub election_under_1278b_active: bool,
    /// True if a § 1278(b) election has been made — it is irrevocable
    /// per § 1278(b)(3) absent Secretary's consent.
    pub election_irrevocable: bool,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

/// § 1278(a)(2)(C) de minimis factor — ¼ of 1% = 0.0025 per year.
/// Stored as basis points × 100 to avoid floating-point math:
/// 25 / 10_000 == 0.0025.
pub const DE_MINIMIS_NUMERATOR: i64 = 25;
pub const DE_MINIMIS_DENOMINATOR: i64 = 10_000;

pub fn compute(input: &Section1278Input) -> Section1278Result {
    let mut notes: Vec<String> = Vec::new();

    // § 1278(a)(1) carve-outs — not market discount bonds.
    if matches!(
        input.bond_type,
        BondType::SavingsBond | BondType::ShortTermObligation | BondType::InstallmentObligation
    ) {
        let (carve_out_label, cite) = match input.bond_type {
            BondType::SavingsBond => (
                "U.S. savings bond",
                "26 U.S.C. § 1278(a)(1) (carve-out — U.S. Series E / EE / I savings bond is \
                 NOT a market discount bond)",
            ),
            BondType::ShortTermObligation => (
                "short-term obligation (≤ 1 year to maturity)",
                "26 U.S.C. § 1278(a)(1) (carve-out — short-term obligation with no more than \
                 1 year to maturity at acquisition is NOT a market discount bond)",
            ),
            BondType::InstallmentObligation => (
                "installment obligation",
                "26 U.S.C. § 1278(a)(1) (carve-out — § 453B installment obligation is NOT a \
                 market discount bond)",
            ),
            _ => unreachable!(),
        };
        notes.push(format!(
            "{} — § 1278(a)(1) carve-out applies; bond is NOT a market discount bond. § 1276 \
             ordinary-income recharacterization and § 1277 interest deferral do not apply.",
            carve_out_label,
        ));
        return Section1278Result {
            is_market_discount_bond: false,
            raw_market_discount_cents: 0,
            de_minimis_threshold_cents: 0,
            de_minimis_applies: false,
            statutory_market_discount_cents: 0,
            redemption_basis_for_calc_cents: 0,
            election_under_1278b_active: false,
            election_irrevocable: false,
            citation: cite,
            notes,
        };
    }

    // § 1278(a)(2)(B) — OID bonds use revised issue price; § 1278(a)(2)(A)
    // — Standard bonds use stated redemption price.
    let redemption_basis = match input.bond_type {
        BondType::OID => {
            notes.push(
                "§ 1278(a)(2)(B) — OID bond; revised issue price (acquisition-date OID-adjusted \
                 basis) substitutes for stated redemption price in the discount calculation."
                    .to_string(),
            );
            input.revised_issue_price_cents.max(0)
        }
        BondType::Standard => input.stated_redemption_price_cents.max(0),
        _ => unreachable!(),
    };

    let raw_market_discount = redemption_basis
        .saturating_sub(input.purchase_price_cents.max(0))
        .max(0);

    if raw_market_discount == 0 {
        notes.push(
            "Purchase price is at or above the redemption basis; no market discount. § 1278 \
             does not engage."
                .to_string(),
        );
        return Section1278Result {
            is_market_discount_bond: false,
            raw_market_discount_cents: 0,
            de_minimis_threshold_cents: 0,
            de_minimis_applies: false,
            statutory_market_discount_cents: 0,
            redemption_basis_for_calc_cents: redemption_basis,
            election_under_1278b_active: input.election_made,
            election_irrevocable: input.election_made,
            citation: "26 U.S.C. § 1278(a)(2)(A) (no market discount — basis ≥ redemption \
                       basis)",
            notes,
        };
    }

    // § 1278(a)(2)(C) de minimis threshold —
    // stated_redemption × (¼ of 1%) × years_to_maturity.
    let de_minimis_threshold = redemption_basis
        .saturating_mul(DE_MINIMIS_NUMERATOR)
        .saturating_mul(input.years_to_maturity as i64)
        / DE_MINIMIS_DENOMINATOR;

    let de_minimis_applies = raw_market_discount < de_minimis_threshold;
    let statutory_market_discount = if de_minimis_applies {
        0
    } else {
        raw_market_discount
    };

    if de_minimis_applies {
        notes.push(format!(
            "§ 1278(a)(2)(C) de minimis rule applies — raw discount of {} cents is less than \
             the threshold of {} cents (redemption × ¼ of 1% × {} years); market discount \
             treated as ZERO.",
            raw_market_discount, de_minimis_threshold, input.years_to_maturity,
        ));
    }

    let citation = if input.election_made {
        notes.push(
            "§ 1278(b)(1) current-inclusion election ACTIVE — accrued market discount included \
             in gross income each year; § 1276 disposition recharacterization and § 1277 \
             interest deferral switched OFF."
                .to_string(),
        );
        notes.push(
            "§ 1278(b)(3) — election is IRREVOCABLE absent Secretary's consent. Applies to \
             this bond and to all market discount bonds acquired during or after the year of \
             election per § 1278(b)(2)."
                .to_string(),
        );
        "26 U.S.C. § 1278(a)(1)–(a)(2) (market discount bond definitions); § 1278(a)(2)(C) \
         (de minimis rule); § 1278(b)(1) (current-inclusion election active); § 1278(b)(2) \
         (election scope to subsequent acquisitions); § 1278(b)(3) (election irrevocable \
         absent Secretary's consent); § 1276 / § 1277 (switched OFF by election)"
    } else {
        notes.push(
            "No § 1278(b) election in effect — § 1276 ordinary-income recharacterization on \
             disposition and § 1277 interest-deduction deferral both apply."
                .to_string(),
        );
        "26 U.S.C. § 1278(a)(1)–(a)(2) (market discount bond definitions); § 1278(a)(2)(C) \
         (de minimis rule); § 1278(b) (current-inclusion election NOT made); § 1276 + § 1277 \
         deferral applies"
    };

    // The bond is a "market discount bond" status only if statutory
    // discount survives the de minimis filter. The de minimis rule
    // doesn't change the § 1278(a)(1) classification per se, but
    // for compliance purposes the deferral provisions only engage
    // when statutory_discount > 0.
    Section1278Result {
        is_market_discount_bond: statutory_market_discount > 0,
        raw_market_discount_cents: raw_market_discount,
        de_minimis_threshold_cents: de_minimis_threshold,
        de_minimis_applies,
        statutory_market_discount_cents: statutory_market_discount,
        redemption_basis_for_calc_cents: redemption_basis,
        election_under_1278b_active: input.election_made,
        election_irrevocable: input.election_made,
        citation,
        notes,
    }
}

#[cfg(test)]
// `face − cost_basis` accruals are written with the explicit `.max(0)` clamp
// to mirror IRC § 1278 wording. With compile-time positive deltas clippy
// flags `unnecessary_min_or_max`; the clamp documents the statutory floor.
#[allow(clippy::unnecessary_min_or_max)]
mod tests {
    use super::*;

    fn input(
        bond_type: BondType,
        face: i64,
        rev_issue: i64,
        basis: i64,
        years: u32,
        election: bool,
    ) -> Section1278Input {
        Section1278Input {
            bond_type,
            stated_redemption_price_cents: face,
            revised_issue_price_cents: rev_issue,
            purchase_price_cents: basis,
            years_to_maturity: years,
            election_made: election,
        }
    }

    // ── § 1278(a)(2)(A) basic market discount ────────────────────

    #[test]
    fn standard_bond_face_1000_basis_900_5yr_discount_100() {
        // raw = 100; de minimis threshold = 1000 × 0.0025 × 5 = 12.50
        // → ceil/floor differences: 100000 × 25 × 5 / 10000 = 1250.
        // Hmm — cents math: redemption = 100000, threshold =
        // 100000 × 25 × 5 / 10000 = 1250 cents = $12.50.
        let r = compute(&input(BondType::Standard, 100_000, 0, 90_000, 5, false));
        assert!(r.is_market_discount_bond);
        assert_eq!(r.raw_market_discount_cents, 10_000);
        assert_eq!(r.de_minimis_threshold_cents, 1_250);
        assert!(!r.de_minimis_applies);
        assert_eq!(r.statutory_market_discount_cents, 10_000);
        assert!(r.citation.contains("§ 1278(a)(2)"));
    }

    // ── § 1278(a)(2)(C) de minimis rule ─────────────────────────

    #[test]
    fn de_minimis_below_threshold_zero_discount() {
        // raw = 12; threshold = 1250 cents. Wait, $1000 bond, basis
        // $999.88, 5 years. Raw = 12 cents; threshold = 1250 cents.
        // 12 < 1250 → de minimis applies → statutory = 0.
        let r = compute(&input(BondType::Standard, 100_000, 0, 99_988, 5, false));
        assert_eq!(r.raw_market_discount_cents, 12);
        assert_eq!(r.de_minimis_threshold_cents, 1_250);
        assert!(r.de_minimis_applies);
        assert_eq!(r.statutory_market_discount_cents, 0);
        assert!(!r.is_market_discount_bond);
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("§ 1278(a)(2)(C)") && n.contains("ZERO"))
        );
    }

    #[test]
    fn de_minimis_at_threshold_boundary_does_not_apply() {
        // Statute says "less than" — equality means NOT de minimis.
        // Build: face 100_000, years 5, threshold = 1250.
        // Set raw to exactly 1250 → basis = 98_750.
        let r = compute(&input(BondType::Standard, 100_000, 0, 98_750, 5, false));
        assert_eq!(r.raw_market_discount_cents, 1_250);
        assert_eq!(r.de_minimis_threshold_cents, 1_250);
        assert!(!r.de_minimis_applies);
        assert_eq!(r.statutory_market_discount_cents, 1_250);
    }

    #[test]
    fn de_minimis_just_below_threshold_applies() {
        // raw 1249, threshold 1250 → applies.
        let r = compute(&input(BondType::Standard, 100_000, 0, 98_751, 5, false));
        assert_eq!(r.raw_market_discount_cents, 1_249);
        assert!(r.de_minimis_applies);
        assert_eq!(r.statutory_market_discount_cents, 0);
    }

    #[test]
    fn de_minimis_threshold_scales_with_years() {
        // Same bond, 1 year vs 20 years — threshold scales 20x.
        let one = compute(&input(BondType::Standard, 100_000, 0, 99_000, 1, false));
        let twenty = compute(&input(BondType::Standard, 100_000, 0, 99_000, 20, false));
        assert_eq!(one.de_minimis_threshold_cents, 250);
        assert_eq!(twenty.de_minimis_threshold_cents, 5_000);
        // 1-year: raw 1000 > threshold 250 → discount applies.
        assert!(!one.de_minimis_applies);
        // 20-year: raw 1000 < threshold 5000 → de minimis applies.
        assert!(twenty.de_minimis_applies);
    }

    #[test]
    fn de_minimis_zero_years_to_maturity_zero_threshold_any_discount_applies() {
        // Edge — zero years → threshold = 0 → any positive raw is
        // not less than 0 → full discount.
        let r = compute(&input(BondType::Standard, 100_000, 0, 95_000, 0, false));
        assert_eq!(r.de_minimis_threshold_cents, 0);
        assert!(!r.de_minimis_applies);
        assert_eq!(r.statutory_market_discount_cents, 5_000);
    }

    // ── § 1278(a)(2)(B) OID revised issue price ──────────────────

    #[test]
    fn oid_bond_uses_revised_issue_price_not_face() {
        // OID bond: face $1000, revised issue price $850 (per
        // § 1272 OID accrual to date), basis $800.
        // Market discount = 850 - 800 = 50 cents wait need cents.
        // Face $1000 = 100_000, revised $850 = 85_000, basis $800 =
        // 80_000. Discount = 85_000 - 80_000 = 5_000 cents = $50.
        let r = compute(&input(BondType::OID, 100_000, 85_000, 80_000, 5, false));
        assert_eq!(r.redemption_basis_for_calc_cents, 85_000);
        assert_eq!(r.raw_market_discount_cents, 5_000);
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("§ 1278(a)(2)(B)") && n.contains("revised issue price"))
        );
    }

    #[test]
    fn oid_bond_face_not_used_when_revised_price_is_lower() {
        // If we wrongly used face $1000, discount would be $200.
        // Using revised $850, discount is $50.
        let face_calc = (100_000_i64 - 80_000_i64).max(0);
        let oid_calc = compute(&input(BondType::OID, 100_000, 85_000, 80_000, 5, false));
        assert_eq!(face_calc, 20_000);
        assert_eq!(oid_calc.raw_market_discount_cents, 5_000);
        assert_ne!(oid_calc.raw_market_discount_cents, face_calc);
    }

    // ── § 1278(a)(1) carve-outs ─────────────────────────────────

    #[test]
    fn savings_bond_carve_out_not_market_discount_bond() {
        let r = compute(&input(BondType::SavingsBond, 100_000, 0, 80_000, 5, false));
        assert!(!r.is_market_discount_bond);
        assert_eq!(r.raw_market_discount_cents, 0);
        assert_eq!(r.statutory_market_discount_cents, 0);
        assert!(r.citation.contains("savings bond"));
        assert!(r.citation.contains("§ 1278(a)(1)"));
    }

    #[test]
    fn short_term_obligation_carve_out_not_market_discount_bond() {
        let r = compute(&input(
            BondType::ShortTermObligation,
            100_000,
            0,
            95_000,
            0,
            false,
        ));
        assert!(!r.is_market_discount_bond);
        assert!(r.citation.contains("short-term"));
    }

    #[test]
    fn installment_obligation_carve_out_not_market_discount_bond() {
        let r = compute(&input(
            BondType::InstallmentObligation,
            100_000,
            0,
            80_000,
            5,
            false,
        ));
        assert!(!r.is_market_discount_bond);
        assert!(r.citation.contains("§ 453B"));
    }

    // ── No-discount path ────────────────────────────────────────

    #[test]
    fn premium_bond_basis_above_face_no_discount() {
        let r = compute(&input(BondType::Standard, 100_000, 0, 110_000, 5, false));
        assert!(!r.is_market_discount_bond);
        assert_eq!(r.raw_market_discount_cents, 0);
        assert!(r.citation.contains("basis ≥ redemption basis"));
    }

    #[test]
    fn basis_equals_face_no_discount() {
        let r = compute(&input(BondType::Standard, 100_000, 0, 100_000, 5, false));
        assert_eq!(r.raw_market_discount_cents, 0);
        assert!(!r.is_market_discount_bond);
    }

    // ── § 1278(b) current-inclusion election ────────────────────

    #[test]
    fn election_made_switches_off_section_1276_and_1277() {
        let r = compute(&input(BondType::Standard, 100_000, 0, 90_000, 5, true));
        assert!(r.election_under_1278b_active);
        assert!(r.election_irrevocable);
        assert!(r.citation.contains("§ 1278(b)(1)"));
        assert!(r.citation.contains("§ 1278(b)(3)"));
        assert!(r.citation.contains("switched OFF"));
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("§ 1278(b)(1)") && n.contains("ACTIVE"))
        );
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("§ 1278(b)(3)") && n.contains("IRREVOCABLE"))
        );
    }

    #[test]
    fn no_election_defers_under_section_1276_and_1277() {
        let r = compute(&input(BondType::Standard, 100_000, 0, 90_000, 5, false));
        assert!(!r.election_under_1278b_active);
        assert!(!r.election_irrevocable);
        assert!(r.citation.contains("§ 1276 + § 1277 deferral"));
    }

    #[test]
    fn election_active_with_de_minimis_still_irrevocable() {
        // Even where statutory discount is zero (de minimis), an
        // election made remains irrevocable.
        let r = compute(&input(BondType::Standard, 100_000, 0, 99_988, 5, true));
        assert!(r.election_irrevocable);
        assert_eq!(r.statutory_market_discount_cents, 0);
    }

    // ── Multi-path regression invariants ────────────────────────

    #[test]
    fn carve_outs_never_qualify_as_market_discount_bond_invariant() {
        for &bt in &[
            BondType::SavingsBond,
            BondType::ShortTermObligation,
            BondType::InstallmentObligation,
        ] {
            // Maximally discounted purchase — still not a market
            // discount bond per § 1278(a)(1) carve-out.
            let r = compute(&input(bt, 1_000_000, 0, 1, 30, false));
            assert!(
                !r.is_market_discount_bond,
                "{:?}: carve-out must not be a market discount bond",
                bt,
            );
            assert_eq!(r.statutory_market_discount_cents, 0);
        }
    }

    #[test]
    fn only_oid_uses_revised_issue_price_invariant() {
        // For Standard: redemption basis = face. For OID: revised.
        let std = compute(&input(BondType::Standard, 100_000, 85_000, 80_000, 5, false));
        let oid = compute(&input(BondType::OID, 100_000, 85_000, 80_000, 5, false));
        assert_eq!(std.redemption_basis_for_calc_cents, 100_000);
        assert_eq!(oid.redemption_basis_for_calc_cents, 85_000);
    }

    #[test]
    fn de_minimis_strictly_less_than_threshold_invariant() {
        // Boundary cells: raw < threshold de minimis; raw == threshold
        // NOT de minimis; raw > threshold NOT de minimis.
        let face = 100_000_i64;
        let years = 5;
        let threshold = face * DE_MINIMIS_NUMERATOR * years as i64 / DE_MINIMIS_DENOMINATOR;
        for (raw, expect_de_minimis) in [
            (threshold - 1, true),
            (threshold, false),
            (threshold + 1, false),
        ] {
            let basis = face - raw;
            let r = compute(&input(BondType::Standard, face, 0, basis, years, false));
            assert_eq!(
                r.de_minimis_applies, expect_de_minimis,
                "raw={raw} threshold={threshold}: expected de_minimis_applies={expect_de_minimis}",
            );
        }
    }

    #[test]
    fn statutory_discount_zero_iff_either_no_raw_or_de_minimis_invariant() {
        // Either no raw discount OR de minimis applies → statutory
        // is zero. Otherwise statutory equals raw.
        for face in [100_000_i64, 500_000, 1_000_000] {
            for basis in [50_000_i64, 80_000, 95_000, 99_988, 100_000, 110_000] {
                for years in [1_u32, 5, 20] {
                    let r = compute(&input(BondType::Standard, face, 0, basis, years, false));
                    if r.raw_market_discount_cents == 0 || r.de_minimis_applies {
                        assert_eq!(r.statutory_market_discount_cents, 0);
                    } else {
                        assert_eq!(
                            r.statutory_market_discount_cents,
                            r.raw_market_discount_cents,
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn election_irrevocability_mirrors_election_flag_invariant() {
        // The election_irrevocable flag tracks whether an election
        // has been made (statute makes it irrevocable on making).
        for election in [false, true] {
            let r = compute(&input(BondType::Standard, 100_000, 0, 90_000, 5, election));
            assert_eq!(r.election_under_1278b_active, election);
            assert_eq!(r.election_irrevocable, election);
        }
    }

    #[test]
    fn citation_pins_subsections_per_path() {
        let std = compute(&input(BondType::Standard, 100_000, 0, 90_000, 5, false));
        let oid = compute(&input(BondType::OID, 100_000, 85_000, 80_000, 5, false));
        let dm = compute(&input(BondType::Standard, 100_000, 0, 99_988, 5, false));
        let elect = compute(&input(BondType::Standard, 100_000, 0, 90_000, 5, true));
        let savings = compute(&input(BondType::SavingsBond, 100_000, 0, 80_000, 5, false));

        assert!(std.citation.contains("§ 1278(a)(1)"));
        assert!(std.citation.contains("§ 1278(a)(2)"));
        assert!(oid.citation.contains("§ 1278(a)(1)"));
        assert!(dm.citation.contains("§ 1278(a)(2)(C)"));
        assert!(elect.citation.contains("§ 1278(b)(1)"));
        assert!(elect.citation.contains("§ 1278(b)(3)"));
        assert!(savings.citation.contains("savings bond"));
    }

    #[test]
    fn no_election_note_documents_deferral_engaging() {
        let r = compute(&input(BondType::Standard, 100_000, 0, 90_000, 5, false));
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("§ 1276") && n.contains("§ 1277") && n.contains("apply")),
            "no-election note must document § 1276 + § 1277 engagement"
        );
    }
}
