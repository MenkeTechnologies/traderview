//! IRC § 1272 — Current inclusion of original issue discount (OID).
//!
//! The "phantom income" rule that bondholders of OID instruments
//! face: § 1272(a)(1) requires holders of any OID debt instrument
//! to recognize accrued OID in gross income EACH YEAR, regardless
//! of whether cash was received. The matching § 163(e) deduction
//! on the issuer side keeps the system in balance.
//!
//! Direct companion to:
//!   - `section_1271` (retirement of debt — § 1271(c) cross-
//!     references § 1272 for the no-double-inclusion rule).
//!   - `section_1273` (OID definition).
//!   - `section_1278` (market discount bond definitions —
//!     § 1278(a)(2)(B) revised-issue-price uses § 1272(a) accrual).
//!
//! Five operative paths:
//!
//!   § 1272(a)(1) — GENERAL RULE: Holder must include in gross
//!     income the sum of the daily portions of OID for each day
//!     during the taxable year the holder held the debt instrument.
//!
//!   § 1272(a)(2)(A) — TAX-EXEMPT obligations are carved out.
//!     OID accrues but is not currently includible in gross income.
//!
//!   § 1272(a)(2)(B) — U.S. SAVINGS BONDS are carved out (Series
//!     EE / I bondholders may elect to accrue currently under
//!     § 454, but § 1272 does not impose accrual).
//!
//!   § 1272(a)(2)(C) — SHORT-TERM OBLIGATIONS (fixed maturity ≤ 1
//!     year from date of issue) are carved out from § 1272(a)
//!     current accrual. § 1281 + § 1283 govern short-term OID
//!     separately.
//!
//!   § 1272(a)(2)(D) — Loans between natural persons not in the
//!     business of lending, where the loan amount does not exceed
//!     $10,000 and tax avoidance is not a principal purpose, are
//!     carved out.
//!
//!   § 1272(a)(6) — Special present-value methodology applies to
//!     REMIC interests and other debt instruments where payments
//!     may be ACCELERATED by reason of prepayments (typically
//!     mortgage-backed securities). Caller supplies the
//!     present-value-based annual accrual.
//!
//!   § 1272(a)(7) — ACQUISITION-PREMIUM reduction: a secondary-
//!     market buyer who pays MORE than the adjusted issue price
//!     (but less than the stated redemption) reduces the daily-
//!     portion OID inclusion by the fraction:
//!     (basis − adjusted-issue-price) / (stated-redemption − adjusted-issue-price)
//!
//! Citations: 26 U.S.C. § 1272(a)(1) (general inclusion rule);
//! § 1272(a)(2)(A) (tax-exempt carve-out); § 1272(a)(2)(B) (U.S.
//! savings bond carve-out); § 1272(a)(2)(C) (short-term obligation
//! carve-out); § 1272(a)(2)(D) (natural-person small-loan
//! carve-out); § 1272(a)(3) (daily-portion ratable accrual);
//! § 1272(a)(6) (prepayable mortgage-backed securities special
//! PV methodology); § 1272(a)(7) (acquisition-premium reduction);
//! § 1273 (OID definition); § 1271(c) (no double inclusion).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DebtInstrumentType {
    /// Standard OID instrument — § 1272(a)(1) annual accrual required.
    Standard,
    /// Tax-exempt obligation — § 1272(a)(2)(A) carve-out.
    TaxExempt,
    /// U.S. savings bond — § 1272(a)(2)(B) carve-out.
    SavingsBond,
    /// Short-term obligation (≤ 1 year to maturity) — § 1272(a)(2)(C)
    /// carve-out.
    ShortTermObligation,
    /// Natural-person small loan ≤ $10,000 and not for tax-
    /// avoidance — § 1272(a)(2)(D) carve-out.
    NaturalPersonSmallLoan,
    /// REMIC interest or prepayable mortgage-backed security —
    /// § 1272(a)(6) special PV methodology.
    PrepayableMortgageBacked,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section1272Input {
    pub debt_type: DebtInstrumentType,
    /// Adjusted issue price at the start of the taxable year
    /// (cents). For first-year holders this equals the issue price
    /// (primary market) or acquisition basis (secondary market).
    pub adjusted_issue_price_start_of_year_cents: i64,
    /// Adjusted issue price at the end of the taxable year (cents).
    /// Reflects accrual of OID during the year per § 1272(a)(3).
    pub adjusted_issue_price_end_of_year_cents: i64,
    /// Number of days the holder held the instrument during the
    /// taxable year. Used for proration when held part of year.
    pub days_held_in_year: u32,
    /// Total days in the accrual period (typically 365 / 366 for
    /// full-year accrual; less for partial accrual periods).
    pub days_in_accrual_period: u32,
    /// § 1272(a)(7) acquisition premium — excess of basis over
    /// adjusted issue price at acquisition (cents). Zero where the
    /// holder did not purchase at a premium above AIP.
    pub acquisition_premium_cents: i64,
    /// § 1272(a)(7) denominator — stated redemption price minus
    /// adjusted issue price at acquisition (cents). Used to compute
    /// the premium-fraction reduction.
    pub stated_redemption_minus_aip_at_acquisition_cents: i64,
    /// § 1272(a)(6) caller-supplied PV-based annual accrual for
    /// REMIC / prepayable mortgage-backed securities (cents).
    /// Bypasses the daily-portion math.
    pub prepayable_pv_annual_accrual_cents: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section1272Result {
    /// True if § 1272(a)(1) requires the holder to include OID in
    /// current-year gross income (i.e., debt type is in scope and
    /// no carve-out applies).
    pub annual_inclusion_required: bool,
    /// Increase in adjusted issue price during the year (cents) —
    /// the raw § 1272(a)(3) accrual figure before proration or
    /// premium reduction.
    pub raw_oid_increase_for_year_cents: i64,
    /// Daily-portion total after proration by days held in year
    /// (cents).
    pub daily_portion_total_cents: i64,
    /// § 1272(a)(7) acquisition-premium reduction amount (cents).
    /// Subtracted from the daily-portion total to reach the
    /// current-year inclusion.
    pub acquisition_premium_reduction_cents: i64,
    /// Final current-year OID inclusion amount (cents) after
    /// proration AND acquisition-premium reduction. Zero where
    /// debt type is carved out from § 1272(a) accrual.
    pub current_year_oid_inclusion_cents: i64,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn compute(input: &Section1272Input) -> Section1272Result {
    let mut notes: Vec<String> = Vec::new();

    // § 1272(a)(2) carve-outs.
    if let Some((carve_out, cite)) = match input.debt_type {
        DebtInstrumentType::TaxExempt => Some((
            "tax-exempt obligation",
            "26 U.S.C. § 1272(a)(2)(A) (tax-exempt obligation — current OID inclusion not \
             required; municipal-bond interest excluded under § 103)",
        )),
        DebtInstrumentType::SavingsBond => Some((
            "U.S. savings bond",
            "26 U.S.C. § 1272(a)(2)(B) (U.S. savings bond — current OID accrual not required; \
             Series EE / I holders may elect under § 454)",
        )),
        DebtInstrumentType::ShortTermObligation => Some((
            "short-term obligation (≤ 1 year)",
            "26 U.S.C. § 1272(a)(2)(C) (short-term obligation ≤ 1 year — current OID accrual \
             not required under § 1272; § 1281 + § 1283 govern short-term OID separately)",
        )),
        DebtInstrumentType::NaturalPersonSmallLoan => Some((
            "natural-person small loan ≤ $10,000",
            "26 U.S.C. § 1272(a)(2)(D) (natural-person small loan ≤ $10,000 not for tax-\
             avoidance — current OID accrual not required)",
        )),
        _ => None,
    } {
        notes.push(format!(
            "{} — § 1272(a)(2) carve-out applies; current-year OID accrual is not required \
             under § 1272(a)(1).",
            carve_out,
        ));
        return Section1272Result {
            annual_inclusion_required: false,
            raw_oid_increase_for_year_cents: 0,
            daily_portion_total_cents: 0,
            acquisition_premium_reduction_cents: 0,
            current_year_oid_inclusion_cents: 0,
            citation: cite,
            notes,
        };
    }

    // § 1272(a)(6) — REMIC + prepayable mortgage-backed PV
    // methodology. Caller-supplied annual accrual is used directly.
    if matches!(input.debt_type, DebtInstrumentType::PrepayableMortgageBacked) {
        let pv_accrual = input.prepayable_pv_annual_accrual_cents.max(0);
        notes.push(
            "§ 1272(a)(6) — prepayable debt instrument (REMIC interest or accelerated-payment \
             mortgage-backed security); special present-value methodology applies. Caller-\
             supplied PV-based annual accrual is used directly."
                .to_string(),
        );
        return Section1272Result {
            annual_inclusion_required: true,
            raw_oid_increase_for_year_cents: pv_accrual,
            daily_portion_total_cents: pv_accrual,
            acquisition_premium_reduction_cents: 0,
            current_year_oid_inclusion_cents: pv_accrual,
            citation: "26 U.S.C. § 1272(a)(1) (general inclusion rule); § 1272(a)(6) (REMIC + \
                       prepayable mortgage-backed PV methodology); § 1273 (OID definition)",
            notes,
        };
    }

    // § 1272(a)(3) — Standard daily-portion math.
    let raw_increase = input
        .adjusted_issue_price_end_of_year_cents
        .saturating_sub(input.adjusted_issue_price_start_of_year_cents)
        .max(0);

    // Proration by days held in year (cap at days_in_accrual_period).
    let days_held = input
        .days_held_in_year
        .min(input.days_in_accrual_period.max(1)) as i64;
    let days_total = input.days_in_accrual_period.max(1) as i64;
    let daily_portion_total = raw_increase.saturating_mul(days_held) / days_total;

    // § 1272(a)(7) — acquisition-premium reduction.
    let acquisition_premium = input.acquisition_premium_cents.max(0);
    let denominator = input
        .stated_redemption_minus_aip_at_acquisition_cents
        .max(0);
    let premium_reduction = if acquisition_premium > 0 && denominator > 0 {
        // Reduction = daily_portion × (premium / denominator).
        // Cap reduction at the daily portion (can't reduce below 0).
        let computed = daily_portion_total
            .saturating_mul(acquisition_premium)
            / denominator;
        computed.min(daily_portion_total)
    } else {
        0
    };

    let current_year_inclusion = daily_portion_total
        .saturating_sub(premium_reduction)
        .max(0);

    if premium_reduction > 0 {
        notes.push(
            "§ 1272(a)(7) — acquisition-premium reduction applied; secondary-market basis \
             above adjusted issue price reduces the daily-portion OID inclusion by the \
             premium-fraction (basis − AIP) / (stated redemption − AIP)."
                .to_string(),
        );
    }

    if days_held < days_total {
        notes.push(format!(
            "§ 1272(a)(3) — proration by days held: {} of {} days in accrual period; raw OID \
             increase of {} cents prorated to {} cents daily-portion total.",
            days_held, days_total, raw_increase, daily_portion_total,
        ));
    }

    Section1272Result {
        annual_inclusion_required: true,
        raw_oid_increase_for_year_cents: raw_increase,
        daily_portion_total_cents: daily_portion_total,
        acquisition_premium_reduction_cents: premium_reduction,
        current_year_oid_inclusion_cents: current_year_inclusion,
        citation: "26 U.S.C. § 1272(a)(1) (general rule — daily portions of OID for each day \
                   held); § 1272(a)(3) (daily-portion ratable allocation); § 1272(a)(7) \
                   (acquisition-premium reduction); § 1273 (OID definition); § 1271(c) (no \
                   double inclusion at retirement)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        debt_type: DebtInstrumentType,
        aip_start: i64,
        aip_end: i64,
        days_held: u32,
        days_total: u32,
        premium: i64,
        srminusaip: i64,
        prepayable_pv: i64,
    ) -> Section1272Input {
        Section1272Input {
            debt_type,
            adjusted_issue_price_start_of_year_cents: aip_start,
            adjusted_issue_price_end_of_year_cents: aip_end,
            days_held_in_year: days_held,
            days_in_accrual_period: days_total,
            acquisition_premium_cents: premium,
            stated_redemption_minus_aip_at_acquisition_cents: srminusaip,
            prepayable_pv_annual_accrual_cents: prepayable_pv,
        }
    }

    // ── § 1272(a)(1) standard rule ──────────────────────────────

    #[test]
    fn standard_full_year_holding_no_premium() {
        // AIP $850 → $880 = $30 OID accrual. Held full 365 days.
        // No premium. Current inclusion = $30.
        let r = compute(&input(
            DebtInstrumentType::Standard,
            85_000,
            88_000,
            365,
            365,
            0,
            0,
            0,
        ));
        assert!(r.annual_inclusion_required);
        assert_eq!(r.raw_oid_increase_for_year_cents, 3_000);
        assert_eq!(r.daily_portion_total_cents, 3_000);
        assert_eq!(r.acquisition_premium_reduction_cents, 0);
        assert_eq!(r.current_year_oid_inclusion_cents, 3_000);
        assert!(r.citation.contains("§ 1272(a)(1)"));
    }

    #[test]
    fn standard_partial_year_proration() {
        // Held 100 of 365 days. Raw $3000. Prorated:
        // 3000 × 100 / 365 = 821 cents (integer truncation).
        let r = compute(&input(
            DebtInstrumentType::Standard,
            85_000,
            88_000,
            100,
            365,
            0,
            0,
            0,
        ));
        assert_eq!(r.raw_oid_increase_for_year_cents, 3_000);
        assert_eq!(r.daily_portion_total_cents, 821);
        assert_eq!(r.current_year_oid_inclusion_cents, 821);
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("§ 1272(a)(3)") && n.contains("proration"))
        );
    }

    #[test]
    fn standard_zero_oid_no_inclusion() {
        let r = compute(&input(
            DebtInstrumentType::Standard,
            100_000,
            100_000,
            365,
            365,
            0,
            0,
            0,
        ));
        assert_eq!(r.raw_oid_increase_for_year_cents, 0);
        assert_eq!(r.current_year_oid_inclusion_cents, 0);
    }

    #[test]
    fn standard_negative_aip_change_clamps_at_zero() {
        // Edge — AIP shouldn't decrease, but if input is wrong,
        // clamp at zero.
        let r = compute(&input(
            DebtInstrumentType::Standard,
            100_000,
            95_000,
            365,
            365,
            0,
            0,
            0,
        ));
        assert_eq!(r.raw_oid_increase_for_year_cents, 0);
    }

    #[test]
    fn standard_days_held_exceeds_days_in_period_caps() {
        // Sanity — days_held > days_in_accrual_period caps at total.
        let r = compute(&input(
            DebtInstrumentType::Standard,
            85_000,
            88_000,
            500,
            365,
            0,
            0,
            0,
        ));
        assert_eq!(r.daily_portion_total_cents, 3_000);
    }

    // ── § 1272(a)(2)(A) tax-exempt carve-out ───────────────────

    #[test]
    fn tax_exempt_obligation_no_current_inclusion() {
        let r = compute(&input(
            DebtInstrumentType::TaxExempt,
            85_000,
            88_000,
            365,
            365,
            0,
            0,
            0,
        ));
        assert!(!r.annual_inclusion_required);
        assert_eq!(r.current_year_oid_inclusion_cents, 0);
        assert!(r.citation.contains("§ 1272(a)(2)(A)"));
        assert!(r.citation.contains("§ 103"));
    }

    // ── § 1272(a)(2)(B) U.S. savings bond carve-out ────────────

    #[test]
    fn us_savings_bond_no_current_inclusion() {
        let r = compute(&input(
            DebtInstrumentType::SavingsBond,
            85_000,
            88_000,
            365,
            365,
            0,
            0,
            0,
        ));
        assert!(!r.annual_inclusion_required);
        assert!(r.citation.contains("§ 1272(a)(2)(B)"));
        assert!(r.citation.contains("§ 454"));
    }

    // ── § 1272(a)(2)(C) short-term obligation carve-out ────────

    #[test]
    fn short_term_obligation_no_current_inclusion() {
        let r = compute(&input(
            DebtInstrumentType::ShortTermObligation,
            85_000,
            88_000,
            300,
            365,
            0,
            0,
            0,
        ));
        assert!(!r.annual_inclusion_required);
        assert!(r.citation.contains("§ 1272(a)(2)(C)"));
        assert!(r.citation.contains("§ 1281"));
    }

    // ── § 1272(a)(2)(D) natural-person small-loan carve-out ────

    #[test]
    fn natural_person_small_loan_no_current_inclusion() {
        let r = compute(&input(
            DebtInstrumentType::NaturalPersonSmallLoan,
            5_00,
            10_00,
            365,
            365,
            0,
            0,
            0,
        ));
        assert!(!r.annual_inclusion_required);
        assert!(r.citation.contains("§ 1272(a)(2)(D)"));
    }

    // ── § 1272(a)(6) prepayable mortgage-backed special PV ─────

    #[test]
    fn prepayable_mbs_uses_caller_pv_accrual() {
        // Caller supplies PV-based annual accrual; daily-portion
        // math bypassed.
        let r = compute(&input(
            DebtInstrumentType::PrepayableMortgageBacked,
            0, // AIP fields irrelevant
            0,
            365,
            365,
            0,
            0,
            4_500, // $45 PV accrual
        ));
        assert!(r.annual_inclusion_required);
        assert_eq!(r.current_year_oid_inclusion_cents, 4_500);
        assert!(r.citation.contains("§ 1272(a)(6)"));
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("§ 1272(a)(6)") && n.contains("PV"))
        );
    }

    #[test]
    fn prepayable_mbs_negative_pv_accrual_clamps_at_zero() {
        let r = compute(&input(
            DebtInstrumentType::PrepayableMortgageBacked,
            0,
            0,
            365,
            365,
            0,
            0,
            -1_000,
        ));
        assert_eq!(r.current_year_oid_inclusion_cents, 0);
    }

    // ── § 1272(a)(7) acquisition-premium reduction ─────────────

    #[test]
    fn acquisition_premium_reduces_daily_portion() {
        // AIP $850 → $880; raw $30 OID. Held full year.
        // Premium = $20; stated_redemption_minus_aip = $50.
        // Reduction = $30 × $20 / $50 = $12.
        // Current inclusion = $30 - $12 = $18.
        let r = compute(&input(
            DebtInstrumentType::Standard,
            85_000,
            88_000,
            365,
            365,
            2_000,
            5_000,
            0,
        ));
        assert_eq!(r.daily_portion_total_cents, 3_000);
        assert_eq!(r.acquisition_premium_reduction_cents, 1_200);
        assert_eq!(r.current_year_oid_inclusion_cents, 1_800);
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("§ 1272(a)(7)") && n.contains("acquisition-premium"))
        );
    }

    #[test]
    fn acquisition_premium_equals_denominator_full_reduction() {
        // Premium = denominator → fraction = 1 → reduction = daily.
        let r = compute(&input(
            DebtInstrumentType::Standard,
            85_000,
            88_000,
            365,
            365,
            5_000,
            5_000,
            0,
        ));
        assert_eq!(r.acquisition_premium_reduction_cents, 3_000);
        assert_eq!(r.current_year_oid_inclusion_cents, 0);
    }

    #[test]
    fn acquisition_premium_exceeds_denominator_caps_at_daily_portion() {
        // Premium > denominator → still capped at daily portion.
        let r = compute(&input(
            DebtInstrumentType::Standard,
            85_000,
            88_000,
            365,
            365,
            10_000,
            5_000,
            0,
        ));
        assert_eq!(r.acquisition_premium_reduction_cents, 3_000);
        assert_eq!(r.current_year_oid_inclusion_cents, 0);
    }

    #[test]
    fn zero_premium_no_reduction() {
        let r = compute(&input(
            DebtInstrumentType::Standard,
            85_000,
            88_000,
            365,
            365,
            0,
            5_000,
            0,
        ));
        assert_eq!(r.acquisition_premium_reduction_cents, 0);
        assert_eq!(r.current_year_oid_inclusion_cents, 3_000);
    }

    #[test]
    fn zero_denominator_no_reduction() {
        // Defensive — denominator 0 should not panic, just skip
        // reduction.
        let r = compute(&input(
            DebtInstrumentType::Standard,
            85_000,
            88_000,
            365,
            365,
            2_000,
            0,
            0,
        ));
        assert_eq!(r.acquisition_premium_reduction_cents, 0);
        assert_eq!(r.current_year_oid_inclusion_cents, 3_000);
    }

    // ── Regression-critical invariants ──────────────────────────

    #[test]
    fn carve_outs_never_require_inclusion_invariant() {
        for &dt in &[
            DebtInstrumentType::TaxExempt,
            DebtInstrumentType::SavingsBond,
            DebtInstrumentType::ShortTermObligation,
            DebtInstrumentType::NaturalPersonSmallLoan,
        ] {
            // Maximal OID accrual; still no current-year inclusion.
            let r = compute(&input(dt, 0, 100_000, 365, 365, 0, 0, 0));
            assert!(
                !r.annual_inclusion_required,
                "{:?}: carve-out must not require current inclusion",
                dt,
            );
            assert_eq!(r.current_year_oid_inclusion_cents, 0);
        }
    }

    #[test]
    fn standard_and_prepayable_both_require_inclusion_invariant() {
        let std = compute(&input(
            DebtInstrumentType::Standard,
            85_000,
            88_000,
            365,
            365,
            0,
            0,
            0,
        ));
        let mbs = compute(&input(
            DebtInstrumentType::PrepayableMortgageBacked,
            0,
            0,
            365,
            365,
            0,
            0,
            4_500,
        ));
        assert!(std.annual_inclusion_required);
        assert!(mbs.annual_inclusion_required);
    }

    #[test]
    fn current_inclusion_equals_daily_minus_premium_invariant() {
        // Across multiple premium levels, current inclusion =
        // daily_portion_total − premium_reduction.
        for premium in [0_i64, 1_000, 2_500, 5_000, 10_000] {
            let r = compute(&input(
                DebtInstrumentType::Standard,
                85_000,
                88_000,
                365,
                365,
                premium,
                5_000,
                0,
            ));
            assert_eq!(
                r.current_year_oid_inclusion_cents,
                r.daily_portion_total_cents
                    .saturating_sub(r.acquisition_premium_reduction_cents)
                    .max(0),
            );
        }
    }

    #[test]
    fn daily_portion_proportional_to_days_held_invariant() {
        // Raw OID $3000. Days proportional: 100/365 ≈ 0.274;
        // 200/365 ≈ 0.548; 300/365 ≈ 0.822. Doubling days roughly
        // doubles daily portion.
        let r100 = compute(&input(
            DebtInstrumentType::Standard,
            85_000,
            88_000,
            100,
            365,
            0,
            0,
            0,
        ))
        .daily_portion_total_cents;
        let r200 = compute(&input(
            DebtInstrumentType::Standard,
            85_000,
            88_000,
            200,
            365,
            0,
            0,
            0,
        ))
        .daily_portion_total_cents;
        // 200 days should produce ~2x the daily portion of 100.
        assert!(r200 > r100);
        assert!(r200 < 2 * r100 + 10); // within rounding tolerance
    }

    #[test]
    fn citation_pins_carve_out_specific_subparagraph_per_type() {
        let te = compute(&input(
            DebtInstrumentType::TaxExempt,
            85_000,
            88_000,
            365,
            365,
            0,
            0,
            0,
        ));
        let sb = compute(&input(
            DebtInstrumentType::SavingsBond,
            85_000,
            88_000,
            365,
            365,
            0,
            0,
            0,
        ));
        let sto = compute(&input(
            DebtInstrumentType::ShortTermObligation,
            85_000,
            88_000,
            365,
            365,
            0,
            0,
            0,
        ));
        let npsl = compute(&input(
            DebtInstrumentType::NaturalPersonSmallLoan,
            500,
            1_000,
            365,
            365,
            0,
            0,
            0,
        ));
        let std = compute(&input(
            DebtInstrumentType::Standard,
            85_000,
            88_000,
            365,
            365,
            0,
            0,
            0,
        ));
        let mbs = compute(&input(
            DebtInstrumentType::PrepayableMortgageBacked,
            0,
            0,
            365,
            365,
            0,
            0,
            4_500,
        ));

        assert!(te.citation.contains("§ 1272(a)(2)(A)"));
        assert!(sb.citation.contains("§ 1272(a)(2)(B)"));
        assert!(sto.citation.contains("§ 1272(a)(2)(C)"));
        assert!(npsl.citation.contains("§ 1272(a)(2)(D)"));
        assert!(std.citation.contains("§ 1272(a)(1)"));
        assert!(std.citation.contains("§ 1271(c)"));
        assert!(mbs.citation.contains("§ 1272(a)(6)"));
    }

    #[test]
    fn premium_reduction_only_applies_to_standard_not_mbs() {
        // MBS path bypasses § 1272(a)(7) — caller-supplied PV
        // accrual is final.
        let std = compute(&input(
            DebtInstrumentType::Standard,
            85_000,
            88_000,
            365,
            365,
            2_000,
            5_000,
            0,
        ));
        let mbs = compute(&input(
            DebtInstrumentType::PrepayableMortgageBacked,
            0,
            0,
            365,
            365,
            2_000,
            5_000,
            3_000,
        ));
        assert!(std.acquisition_premium_reduction_cents > 0);
        assert_eq!(mbs.acquisition_premium_reduction_cents, 0);
    }

    #[test]
    fn note_documents_proration_only_for_partial_year() {
        let full = compute(&input(
            DebtInstrumentType::Standard,
            85_000,
            88_000,
            365,
            365,
            0,
            0,
            0,
        ));
        let partial = compute(&input(
            DebtInstrumentType::Standard,
            85_000,
            88_000,
            100,
            365,
            0,
            0,
            0,
        ));
        assert!(
            !full.notes
                .iter()
                .any(|n| n.contains("§ 1272(a)(3)") && n.contains("proration"))
        );
        assert!(
            partial
                .notes
                .iter()
                .any(|n| n.contains("§ 1272(a)(3)") && n.contains("proration"))
        );
    }
}
