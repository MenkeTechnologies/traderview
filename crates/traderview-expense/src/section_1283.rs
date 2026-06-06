//! IRC § 1283 — Definitions and special rules for short-term
//! obligations.
//!
//! The definitional anchor for the short-term obligation cluster.
//! § 1281 (current inclusion of acquisition discount for specified
//! holders) and § 1282 (deferral of interest deduction on
//! short-term obligations) both cross-reference § 1283 for the
//! underlying terms.
//!
//! Direct companion to:
//!   - `section_1281` (current inclusion of acquisition discount
//!     on short-term obligations for accrual-method taxpayers +
//!     dealers + banks + RICs + hedging-transaction holders +
//!     pass-thru entities).
//!   - `section_1271` (retirement of debt instruments — § 1271(a)(3)
//!     short-term government obligations + § 1271(a)(4) short-term
//!     nongovernment obligations).
//!   - `section_1272` (long-term OID current inclusion —
//!     § 1272(a)(2)(C) short-term carve-out).
//!
//! Four operative subsections:
//!
//!   § 1283(a)(1) — SHORT-TERM OBLIGATION DEFINITION: Any bond,
//!     debenture, note, certificate, or other evidence of
//!     indebtedness which has a fixed maturity date NOT MORE THAN
//!     1 YEAR from the date of issue, with exceptions for tax-
//!     exempt obligations.
//!
//!   § 1283(a)(2) — ACQUISITION DISCOUNT: Excess of stated
//!     redemption price at maturity over the taxpayer's basis for
//!     the obligation. Note that acquisition discount is broader
//!     than OID because it captures the spread between basis and
//!     SRPM regardless of whether the spread arose from original
//!     issue or secondary-market purchase.
//!
//!   § 1283(b)(1) — DAILY PORTION (RATABLE ACCRUAL): Total
//!     acquisition discount divided by the number of days after
//!     the day the taxpayer acquired the obligation and up to and
//!     including the day of maturity. The daily portion multiplied
//!     by days held determines the current-year accrual under
//!     § 1281.
//!
//!   § 1283(b)(2) — CONSTANT-YIELD ELECTION: Taxpayer may elect to
//!     compute acquisition discount under a constant-yield method
//!     parallel to § 1272(a)(3) OID rules.
//!
//!   § 1283(c) — NONGOVERNMENTAL OBLIGATIONS: For short-term
//!     obligations that are not government-issued, § 1281 and
//!     § 1282 apply using ORIGINAL ISSUE DISCOUNT in lieu of
//!     acquisition discount, with appropriate adjustments.
//!
//!   § 1283(d) — BASIS ADJUSTMENT: The basis of any short-term
//!     obligation in the hands of the holder is INCREASED by the
//!     amount included in gross income pursuant to § 1281.
//!
//! Citations: 26 U.S.C. § 1283(a)(1) (short-term obligation
//! definition — ≤ 1 year to maturity); § 1283(a)(2) (acquisition
//! discount — SRPM minus basis); § 1283(b)(1) (daily-portion
//! ratable accrual); § 1283(b)(2) (constant-yield election);
//! § 1283(c) (nongovernmental OID substitution); § 1283(d) (basis
//! increase by § 1281 inclusion); § 1281 (current inclusion
//! mandate); § 1282 (interest-deduction deferral); § 1272 (long-
//! term OID — § 1272(a)(2)(C) short-term carve-out cross-
//! reference); § 1271 (retirement of debt instruments).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ObligationType {
    /// Government-issued short-term obligation (Treasury bill,
    /// federal agency note). § 1281 applies to FULL acquisition
    /// discount.
    Governmental,
    /// Nongovernmental short-term obligation (corporate commercial
    /// paper, asset-backed note). § 1283(c) substitutes OID for
    /// acquisition discount.
    NonGovernmental,
    /// Tax-exempt short-term obligation — § 1283(a)(1) carve-out;
    /// not a "short-term obligation" for § 1281 purposes.
    TaxExempt,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AccrualMethod {
    /// § 1283(b)(1) — default ratable daily-portion accrual.
    RatableAccrual,
    /// § 1283(b)(2) — constant-yield election parallel to
    /// § 1272(a)(3) OID rules. Caller supplies the computed
    /// daily-portion.
    ConstantYieldElection,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section1283Input {
    pub obligation_type: ObligationType,
    /// Days from issue date to maturity date. Must be ≤ 365 for
    /// § 1283(a)(1) short-term-obligation status.
    pub days_from_issue_to_maturity: u32,
    /// Stated redemption price at maturity (cents).
    pub stated_redemption_price_at_maturity_cents: i64,
    /// Taxpayer's basis in the obligation at acquisition (cents).
    pub basis_at_acquisition_cents: i64,
    /// § 1283(c) OID amount for nongovernmental obligations (cents).
    /// Used in lieu of acquisition_discount for nongovernmental.
    pub oid_amount_for_nongovernmental_cents: i64,
    /// Days from acquisition to maturity (inclusive of maturity
    /// day per § 1283(b)(1)).
    pub days_from_acquisition_to_maturity: u32,
    /// Days the holder held the obligation during the taxable
    /// year.
    pub days_held_in_year: u32,
    pub accrual_method: AccrualMethod,
    /// § 1283(b)(2) caller-supplied constant-yield daily-portion
    /// computation (cents per day).
    pub constant_yield_daily_portion_cents: i64,
    /// Prior-year § 1281 inclusion amount (cents). § 1283(d) bumps
    /// basis by this amount.
    pub prior_year_section_1281_inclusion_cents: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section1283Result {
    /// True if the obligation qualifies as a § 1283(a)(1) short-
    /// term obligation (≤ 1 year to maturity AND not tax-exempt).
    pub is_short_term_obligation: bool,
    /// § 1283(a)(2) acquisition discount (cents). Equals max(0,
    /// SRPM − basis). Zero for tax-exempt or premium acquisitions.
    pub acquisition_discount_cents: i64,
    /// Effective discount base after § 1283(c) nongovernmental
    /// OID substitution (cents). Equal to acquisition_discount for
    /// governmental + tax-exempt; OID for nongovernmental.
    pub effective_discount_base_cents: i64,
    /// § 1283(b)(1) daily portion under ratable accrual (cents per
    /// day). Or § 1283(b)(2) caller-supplied if constant-yield
    /// elected.
    pub daily_portion_cents: i64,
    /// Current-year accrued discount = daily portion × days held
    /// (cents).
    pub current_year_accrual_cents: i64,
    /// § 1283(d) adjusted basis after § 1281 prior-year inclusion
    /// (cents). Equal to basis_at_acquisition + prior-year
    /// inclusion.
    pub adjusted_basis_cents: i64,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

/// § 1283(a)(1) — short-term obligation threshold of 1 year (365
/// days).
pub const SECTION_1283A1_SHORT_TERM_DAYS: u32 = 365;

pub fn compute(input: &Section1283Input) -> Section1283Result {
    let mut notes: Vec<String> = Vec::new();

    // § 1283(a)(1) status check.
    let is_short_term = input.days_from_issue_to_maturity <= SECTION_1283A1_SHORT_TERM_DAYS
        && !matches!(input.obligation_type, ObligationType::TaxExempt);

    if matches!(input.obligation_type, ObligationType::TaxExempt) {
        notes.push(
            "§ 1283(a)(1) — tax-exempt short-term obligation carve-out; not a 'short-term \
             obligation' for § 1281 current-inclusion purposes."
                .to_string(),
        );
    }
    if input.days_from_issue_to_maturity > SECTION_1283A1_SHORT_TERM_DAYS {
        notes.push(format!(
            "§ 1283(a)(1) — obligation has {} days to maturity exceeding the 365-day short-term \
             threshold; not a short-term obligation. § 1272 long-term OID rules apply instead.",
            input.days_from_issue_to_maturity,
        ));
    }

    // § 1283(a)(2) acquisition discount = SRPM − basis (clamped at 0).
    let acquisition_discount = input
        .stated_redemption_price_at_maturity_cents
        .saturating_sub(input.basis_at_acquisition_cents)
        .max(0);

    // § 1283(c) nongovernmental substitution.
    let effective_discount_base =
        if matches!(input.obligation_type, ObligationType::NonGovernmental) {
            notes.push(
                "§ 1283(c) — nongovernmental short-term obligation; § 1281 and § 1282 apply using \
             ORIGINAL ISSUE DISCOUNT in lieu of acquisition discount."
                    .to_string(),
            );
            input.oid_amount_for_nongovernmental_cents.max(0)
        } else {
            acquisition_discount
        };

    // § 1283(b) daily portion.
    let daily_portion = match input.accrual_method {
        AccrualMethod::RatableAccrual => {
            let days = input.days_from_acquisition_to_maturity.max(1) as i64;
            effective_discount_base / days
        }
        AccrualMethod::ConstantYieldElection => {
            notes.push(
                "§ 1283(b)(2) — constant-yield election parallel to § 1272(a)(3) OID rules; \
                 caller-supplied daily-portion computation."
                    .to_string(),
            );
            input.constant_yield_daily_portion_cents.max(0)
        }
    };

    // Current-year accrual = daily portion × days held (capped at
    // total available discount).
    let raw_accrual = daily_portion.saturating_mul(input.days_held_in_year as i64);
    let current_year_accrual = if is_short_term {
        raw_accrual.min(effective_discount_base)
    } else {
        0
    };

    // § 1283(d) basis adjustment.
    let adjusted_basis = input
        .basis_at_acquisition_cents
        .saturating_add(input.prior_year_section_1281_inclusion_cents.max(0));
    if input.prior_year_section_1281_inclusion_cents > 0 {
        notes.push(format!(
            "§ 1283(d) — basis increased by {} cents reflecting prior-year § 1281 inclusion \
             amounts.",
            input.prior_year_section_1281_inclusion_cents,
        ));
    }

    notes.push(
        "Companion to section_1281 (current inclusion of acquisition discount for accrual-\
         method + dealer + bank + RIC + hedging + pass-thru holders); section_1271 (retirement \
         of debt — § 1271(a)(3) short-term government + § 1271(a)(4) short-term nongovernment); \
         section_1272 (long-term OID — § 1272(a)(2)(C) short-term carve-out cross-reference)."
            .to_string(),
    );

    let citation = "26 U.S.C. § 1283(a)(1) (short-term obligation definition — ≤ 1 year to \
                    maturity); § 1283(a)(2) (acquisition discount — SRPM minus basis); \
                    § 1283(b)(1) (daily-portion ratable accrual); § 1283(b)(2) (constant-yield \
                    election); § 1283(c) (nongovernmental OID substitution); § 1283(d) (basis \
                    increase by § 1281 inclusion); § 1281 (current inclusion mandate); § 1282 \
                    (interest-deduction deferral); § 1272(a)(2)(C) (long-term OID short-term \
                    carve-out)";

    Section1283Result {
        is_short_term_obligation: is_short_term,
        acquisition_discount_cents: acquisition_discount,
        effective_discount_base_cents: effective_discount_base,
        daily_portion_cents: daily_portion,
        current_year_accrual_cents: current_year_accrual,
        adjusted_basis_cents: adjusted_basis,
        citation,
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        obligation: ObligationType,
        days_to_maturity_from_issue: u32,
        srpm: i64,
        basis: i64,
        oid_nongov: i64,
        days_acq_to_mat: u32,
        days_held: u32,
        method: AccrualMethod,
        constant_yield: i64,
        prior_inclusion: i64,
    ) -> Section1283Input {
        Section1283Input {
            obligation_type: obligation,
            days_from_issue_to_maturity: days_to_maturity_from_issue,
            stated_redemption_price_at_maturity_cents: srpm,
            basis_at_acquisition_cents: basis,
            oid_amount_for_nongovernmental_cents: oid_nongov,
            days_from_acquisition_to_maturity: days_acq_to_mat,
            days_held_in_year: days_held,
            accrual_method: method,
            constant_yield_daily_portion_cents: constant_yield,
            prior_year_section_1281_inclusion_cents: prior_inclusion,
        }
    }

    // ── § 1283(a)(1) short-term obligation definition ──────────

    #[test]
    fn governmental_180_day_obligation_is_short_term() {
        let r = compute(&input(
            ObligationType::Governmental,
            180,
            10_000,
            9_500,
            0,
            180,
            90,
            AccrualMethod::RatableAccrual,
            0,
            0,
        ));
        assert!(r.is_short_term_obligation);
    }

    #[test]
    fn governmental_365_day_obligation_at_boundary_is_short_term() {
        let r = compute(&input(
            ObligationType::Governmental,
            365,
            10_000,
            9_500,
            0,
            365,
            180,
            AccrualMethod::RatableAccrual,
            0,
            0,
        ));
        assert!(r.is_short_term_obligation);
    }

    #[test]
    fn governmental_366_day_obligation_above_boundary_not_short_term() {
        let r = compute(&input(
            ObligationType::Governmental,
            366,
            10_000,
            9_500,
            0,
            366,
            180,
            AccrualMethod::RatableAccrual,
            0,
            0,
        ));
        assert!(!r.is_short_term_obligation);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("366 days") && n.contains("§ 1272")));
    }

    #[test]
    fn tax_exempt_short_term_obligation_carve_out() {
        let r = compute(&input(
            ObligationType::TaxExempt,
            180,
            10_000,
            9_500,
            0,
            180,
            90,
            AccrualMethod::RatableAccrual,
            0,
            0,
        ));
        assert!(!r.is_short_term_obligation);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 1283(a)(1)") && n.contains("tax-exempt")));
    }

    // ── § 1283(a)(2) acquisition discount ──────────────────────

    #[test]
    fn acquisition_discount_basic_math() {
        // SRPM $100, basis $95 → acquisition discount $5.
        let r = compute(&input(
            ObligationType::Governmental,
            180,
            10_000,
            9_500,
            0,
            180,
            90,
            AccrualMethod::RatableAccrual,
            0,
            0,
        ));
        assert_eq!(r.acquisition_discount_cents, 500);
    }

    #[test]
    fn acquisition_discount_premium_acquisition_clamps_at_zero() {
        // Basis > SRPM (acquired at premium) → no acquisition
        // discount.
        let r = compute(&input(
            ObligationType::Governmental,
            180,
            10_000,
            10_500,
            0,
            180,
            90,
            AccrualMethod::RatableAccrual,
            0,
            0,
        ));
        assert_eq!(r.acquisition_discount_cents, 0);
        assert_eq!(r.current_year_accrual_cents, 0);
    }

    #[test]
    fn acquisition_discount_at_par_zero_discount() {
        let r = compute(&input(
            ObligationType::Governmental,
            180,
            10_000,
            10_000,
            0,
            180,
            90,
            AccrualMethod::RatableAccrual,
            0,
            0,
        ));
        assert_eq!(r.acquisition_discount_cents, 0);
    }

    // ── § 1283(b)(1) daily portion ratable accrual ─────────────

    #[test]
    fn ratable_accrual_daily_portion_basic() {
        // Acquisition discount $500, days to maturity 180 → daily
        // portion ~$2.78 → integer truncation $2.
        let r = compute(&input(
            ObligationType::Governmental,
            180,
            10_000,
            9_500,
            0,
            180,
            90,
            AccrualMethod::RatableAccrual,
            0,
            0,
        ));
        assert_eq!(r.daily_portion_cents, 500 / 180);
    }

    #[test]
    fn ratable_accrual_accrual_proportional_to_days_held() {
        // 180 days to maturity, held 90 days → ~50% of discount.
        let r = compute(&input(
            ObligationType::Governmental,
            180,
            10_000,
            9_500,
            0,
            180,
            90,
            AccrualMethod::RatableAccrual,
            0,
            0,
        ));
        // daily_portion (2) × 90 = 180.
        assert_eq!(r.current_year_accrual_cents, (500 / 180) * 90);
    }

    #[test]
    fn ratable_accrual_full_holding_full_discount() {
        let r = compute(&input(
            ObligationType::Governmental,
            180,
            10_000,
            9_500,
            0,
            180,
            180,
            AccrualMethod::RatableAccrual,
            0,
            0,
        ));
        // daily_portion (2) × 180 = 360, capped at acquisition
        // discount 500. So 360.
        // Actually with $500 discount and 180 days, daily_portion =
        // 500 / 180 = 2 (truncated). 2 × 180 = 360. Capped at 500
        // means min(360, 500) = 360.
        assert_eq!(r.current_year_accrual_cents, 360);
    }

    // ── § 1283(b)(2) constant-yield election ───────────────────

    #[test]
    fn constant_yield_election_uses_caller_supplied_daily_portion() {
        let r = compute(&input(
            ObligationType::Governmental,
            180,
            10_000,
            9_500,
            0,
            180,
            90,
            AccrualMethod::ConstantYieldElection,
            3, // $0.03 per day
            0,
        ));
        assert_eq!(r.daily_portion_cents, 3);
        assert_eq!(r.current_year_accrual_cents, 3 * 90);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 1283(b)(2)") && n.contains("constant-yield")));
    }

    // ── § 1283(c) nongovernmental OID substitution ────────────

    #[test]
    fn nongovernmental_uses_oid_in_lieu_of_acquisition_discount() {
        // SRPM 10,000, basis 9,500 → acquisition discount 500.
        // But for nongovernmental, § 1283(c) substitutes OID = 300.
        let r = compute(&input(
            ObligationType::NonGovernmental,
            180,
            10_000,
            9_500,
            300,
            180,
            180,
            AccrualMethod::RatableAccrual,
            0,
            0,
        ));
        // Effective discount base = OID 300, not acquisition 500.
        assert_eq!(r.effective_discount_base_cents, 300);
        // daily_portion = 300 / 180 = 1.
        assert_eq!(r.daily_portion_cents, 1);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 1283(c)") && n.contains("ORIGINAL ISSUE DISCOUNT")));
    }

    #[test]
    fn nongovernmental_zero_oid_zero_accrual() {
        // SRPM 10,000, basis 9,500 → acquisition discount 500.
        // But OID component = 0 → effective base = 0.
        let r = compute(&input(
            ObligationType::NonGovernmental,
            180,
            10_000,
            9_500,
            0,
            180,
            180,
            AccrualMethod::RatableAccrual,
            0,
            0,
        ));
        assert_eq!(r.effective_discount_base_cents, 0);
        assert_eq!(r.current_year_accrual_cents, 0);
    }

    // ── § 1283(d) basis adjustment ─────────────────────────────

    #[test]
    fn basis_increased_by_prior_year_section_1281_inclusion() {
        let r = compute(&input(
            ObligationType::Governmental,
            180,
            10_000,
            9_500,
            0,
            180,
            90,
            AccrualMethod::RatableAccrual,
            0,
            100, // Prior year inclusion bumped basis by $1.
        ));
        // Basis 9,500 + 100 = 9,600.
        assert_eq!(r.adjusted_basis_cents, 9_600);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 1283(d)") && n.contains("100 cents")));
    }

    #[test]
    fn no_prior_inclusion_basis_equals_acquisition_basis() {
        let r = compute(&input(
            ObligationType::Governmental,
            180,
            10_000,
            9_500,
            0,
            180,
            90,
            AccrualMethod::RatableAccrual,
            0,
            0,
        ));
        assert_eq!(r.adjusted_basis_cents, 9_500);
    }

    #[test]
    fn negative_prior_inclusion_clamps_at_zero() {
        // Defensive — negative prior inclusion shouldn't reduce
        // basis.
        let r = compute(&input(
            ObligationType::Governmental,
            180,
            10_000,
            9_500,
            0,
            180,
            90,
            AccrualMethod::RatableAccrual,
            0,
            -500,
        ));
        assert_eq!(r.adjusted_basis_cents, 9_500);
    }

    // ── Edge cases ─────────────────────────────────────────────

    #[test]
    fn zero_days_held_zero_accrual() {
        let r = compute(&input(
            ObligationType::Governmental,
            180,
            10_000,
            9_500,
            0,
            180,
            0,
            AccrualMethod::RatableAccrual,
            0,
            0,
        ));
        assert_eq!(r.current_year_accrual_cents, 0);
    }

    #[test]
    fn zero_days_acq_to_maturity_avoids_division_by_zero() {
        let r = compute(&input(
            ObligationType::Governmental,
            180,
            10_000,
            9_500,
            0,
            0,
            0,
            AccrualMethod::RatableAccrual,
            0,
            0,
        ));
        // days_from_acquisition_to_maturity is max(1) for safe div.
        assert_eq!(r.daily_portion_cents, 500);
    }

    #[test]
    fn long_term_obligation_zero_accrual() {
        let r = compute(&input(
            ObligationType::Governmental,
            500,
            10_000,
            9_500,
            0,
            500,
            250,
            AccrualMethod::RatableAccrual,
            0,
            0,
        ));
        assert!(!r.is_short_term_obligation);
        assert_eq!(r.current_year_accrual_cents, 0);
    }

    // ── Regression-critical invariants ──────────────────────────

    #[test]
    fn one_year_boundary_truth_table_invariant() {
        for (days, expected_short_term) in [
            (1_u32, true),
            (180, true),
            (365, true), // at boundary inclusive
            (366, false),
            (730, false),
        ] {
            let r = compute(&input(
                ObligationType::Governmental,
                days,
                10_000,
                9_500,
                0,
                days,
                days / 2,
                AccrualMethod::RatableAccrual,
                0,
                0,
            ));
            assert_eq!(
                r.is_short_term_obligation, expected_short_term,
                "days={} expected_short_term={}",
                days, expected_short_term,
            );
        }
    }

    #[test]
    fn effective_discount_base_depends_on_obligation_type_invariant() {
        // Same SRPM/basis/OID across 3 obligation types: gov uses
        // acquisition discount; nongov uses OID; tax-exempt zeroed.
        let gov = compute(&input(
            ObligationType::Governmental,
            180,
            10_000,
            9_500,
            300,
            180,
            90,
            AccrualMethod::RatableAccrual,
            0,
            0,
        ));
        let nongov = compute(&input(
            ObligationType::NonGovernmental,
            180,
            10_000,
            9_500,
            300,
            180,
            90,
            AccrualMethod::RatableAccrual,
            0,
            0,
        ));
        let tax_exempt = compute(&input(
            ObligationType::TaxExempt,
            180,
            10_000,
            9_500,
            300,
            180,
            90,
            AccrualMethod::RatableAccrual,
            0,
            0,
        ));
        assert_eq!(gov.effective_discount_base_cents, 500); // acquisition
        assert_eq!(nongov.effective_discount_base_cents, 300); // OID
        assert!(!tax_exempt.is_short_term_obligation);
    }

    #[test]
    fn current_year_accrual_never_exceeds_effective_discount_base_invariant() {
        // Across multiple holding periods.
        for days_held in [10_u32, 50, 100, 200, 1000] {
            let r = compute(&input(
                ObligationType::Governmental,
                180,
                10_000,
                9_500,
                0,
                180,
                days_held,
                AccrualMethod::RatableAccrual,
                0,
                0,
            ));
            assert!(
                r.current_year_accrual_cents <= r.effective_discount_base_cents,
                "days_held={} accrual={} effective_base={}",
                days_held,
                r.current_year_accrual_cents,
                r.effective_discount_base_cents,
            );
        }
    }

    #[test]
    fn citation_pins_all_subsections() {
        let r = compute(&input(
            ObligationType::Governmental,
            180,
            10_000,
            9_500,
            0,
            180,
            90,
            AccrualMethod::RatableAccrual,
            0,
            0,
        ));
        assert!(r.citation.contains("§ 1283(a)(1)"));
        assert!(r.citation.contains("§ 1283(a)(2)"));
        assert!(r.citation.contains("§ 1283(b)(1)"));
        assert!(r.citation.contains("§ 1283(b)(2)"));
        assert!(r.citation.contains("§ 1283(c)"));
        assert!(r.citation.contains("§ 1283(d)"));
        assert!(r.citation.contains("§ 1281"));
        assert!(r.citation.contains("§ 1282"));
        assert!(r.citation.contains("§ 1272(a)(2)(C)"));
    }

    #[test]
    fn sibling_module_note_present() {
        let r = compute(&input(
            ObligationType::Governmental,
            180,
            10_000,
            9_500,
            0,
            180,
            90,
            AccrualMethod::RatableAccrual,
            0,
            0,
        ));
        assert!(
            r.notes.iter().any(|n| n.contains("section_1281")
                && n.contains("section_1271")
                && n.contains("section_1272")),
            "sibling-module note must be present"
        );
    }
}
