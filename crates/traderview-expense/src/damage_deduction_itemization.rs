//! State security-deposit damage-deduction itemization, receipt
//! attachment, photographic documentation, and depreciation-method
//! compliance check.
//!
//! Sibling to:
//!   - `deposit_return_windows` — when the landlord must return.
//!   - `security_deposit_caps` — how much the landlord may hold.
//!   - `deposit_interest` — whether the deposit accrues interest.
//!   - `pre_move_out_inspection` — pre-move-out inspection right.
//!   - `carpet_replacement_useful_life` — depreciation specifically
//!     for carpet.
//!
//! This module handles the ITEMIZATION half of the move-out
//! accounting: the form, content, and method of the damage-deduction
//! notice that accompanies the security-deposit return. The states
//! diverge along five axes:
//!
//!   1. **Itemization deadline** (CA 21 days; WA 30; OR 31; FL 30;
//!      TX 30).
//!   2. **Receipt / invoice attachment threshold** (CA $125; FL $200;
//!      others have no statutory threshold).
//!   3. **Photographic documentation mandate** (CA only — AB 2801
//!      eff. 2025-04-01 for repair/cleaning shots; 2025-07-01 for
//!      pre-tenancy shots).
//!   4. **Depreciation method requirement** for finite-useful-life
//!      items (CA + WA + OR + FL require; TX does not statutorily
//!      mandate).
//!   5. **Statutory bad-faith penalty** (CA up to 2× withheld;
//!      WA up to 2× deposit for intentional refusal; FL court costs
//!      plus attorney fees; TX $100 + 3× retained + attorney fees,
//!      steepest in the matrix).
//!
//! Five regimes:
//!
//!   - **California** — Cal. Civ. Code § 1950.5(g) + AB 2801 (2024,
//!     amending § 1950.5 with photographic documentation; § 1950.5
//!     amended by Stats. 2024, ch. 444 (AB 2801); operative dates
//!     2025-01-01 (effective), 2025-04-01 (repair/cleaning photo
//!     mandate for all tenancies), 2025-07-01 (pre-tenancy photo
//!     mandate for new tenancies). Bad faith: § 1950.5(l) — up to 2×
//!     the amount withheld.
//!
//!   - **Washington** — RCW 59.18.280 — within 30 days "full and
//!     specific statement of the basis for retaining any of the
//!     deposit"; "No portion of any deposit may be withheld . . .
//!     For wear resulting from ordinary use of the premises";
//!     failure to provide → forfeit full deposit; intentional
//!     refusal → up to 2× deposit (discretionary).
//!
//!   - **Oregon** — ORS 90.300(13) — 31-day itemized-accounting
//!     deadline; statutory depreciation requirement for finite-life
//!     items.
//!
//!   - **Florida** — Fla. Stat. § 83.49(3) — 30-day notice by
//!     certified mail to last known address; depreciation required;
//!     bad-faith claim subjects landlord to court costs + reasonable
//!     attorney fees per § 83.49(3)(c).
//!
//!   - **Texas** — Tex. Prop. Code § 92.103 (30-day refund);
//!     § 92.104(c) (itemized list if rent fully paid + no
//!     controversy); § 92.109 (bad faith → $100 + 3× retained +
//!     reasonable attorney fees — steepest statutory penalty).
//!
//! Citations: Cal. Civ. Code § 1950.5(g)(1) (21-day deadline);
//! § 1950.5(g)(2) ($125 receipt threshold); § 1950.5(g)(3)(A) (AB
//! 2801 photographic documentation requirement); § 1950.5(b)(2)
//! (ordinary wear-and-tear exclusion); § 1950.5(l) (bad-faith 2×);
//! RCW 59.18.280; ORS 90.300(13); Fla. Stat. § 83.49(3); Tex. Prop.
//! Code § 92.103 + § 92.104(c) + § 92.109.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    California,
    Washington,
    Oregon,
    Florida,
    Texas,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub regime: Regime,
    /// Days elapsed between move-out and delivery of the itemized
    /// statement to the tenant.
    pub days_to_itemized_statement: u32,
    /// Total dollar amount of deductions claimed against the deposit
    /// (cents).
    pub total_deductions_cents: i64,
    /// Whether each deduction line item is accompanied by a receipt,
    /// invoice, or contractor estimate above the applicable state
    /// threshold.
    pub receipts_attached_for_items_above_threshold: bool,
    /// Whether the landlord took the AB 2801 before-and-after
    /// photographs of the unit (California only).
    pub photographs_taken_before_and_after_repairs: bool,
    /// Whether any deduction claims charges for ordinary wear and
    /// tear (instead of damage).
    pub deduction_includes_ordinary_wear_and_tear: bool,
    /// Whether the landlord depreciated finite-useful-life items
    /// (e.g., paint, carpet, appliances) rather than charging the
    /// full replacement cost.
    pub depreciation_applied_to_finite_life_items: bool,
    /// Whether the landlord claims the deposit was withheld in bad
    /// faith — used to compute statutory penalty.
    pub bad_faith_claim: bool,
    /// Amount withheld from the deposit (cents). Used to compute
    /// statutory penalty.
    pub amount_withheld_cents: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CheckResult {
    pub compliant: bool,
    /// Statutory deadline in days for the regime.
    pub deadline_days: u32,
    /// Statutory receipt-attachment threshold (cents) for the
    /// regime. Zero where the regime has no statutory threshold.
    pub receipt_threshold_cents: i64,
    /// Whether AB 2801 photographic documentation is required.
    /// True for California; false elsewhere.
    pub photographic_documentation_required: bool,
    /// Whether the regime statutorily requires depreciation for
    /// finite-useful-life items.
    pub depreciation_required: bool,
    /// Maximum statutory bad-faith penalty exposure (cents) — only
    /// non-zero when bad_faith_claim is true.
    pub statutory_bad_faith_penalty_cents: i64,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

/// California § 1950.5(g)(2) — itemized statement must be
/// accompanied by receipts, invoices, or estimates for any line
/// item above $125 in deduction value.
pub const CA_RECEIPT_THRESHOLD_CENTS: i64 = 12_500;

/// Florida § 83.49(3) — third-party invoice required for any
/// deduction above $200 (per state practice analysis).
pub const FL_RECEIPT_THRESHOLD_CENTS: i64 = 20_000;

pub fn check(input: &Input) -> CheckResult {
    let mut violations: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();
    let mut compliant = true;

    let (
        deadline_days,
        receipt_threshold_cents,
        photographic_required,
        depreciation_required,
        citation,
    ) = match input.regime {
        Regime::California => (
            21_u32,
            CA_RECEIPT_THRESHOLD_CENTS,
            true,
            true,
            "Cal. Civ. Code § 1950.5(g)(1) (21-day deadline); § 1950.5(g)(2) ($125 \
             receipt/invoice threshold); § 1950.5(g)(3)(A) (AB 2801 photographic documentation \
             — Stats. 2024 ch. 444; 2025-04-01 repair/cleaning photo mandate); § 1950.5(b)(2) \
             (ordinary wear-and-tear exclusion); § 1950.5(l) (bad-faith — up to 2× the amount \
             withheld)",
        ),
        Regime::Washington => (
            30_u32,
            0_i64,
            false,
            true,
            "RCW 59.18.280 (30-day full-and-specific statement; ordinary-wear-and-tear \
             exclusion; intentional refusal up to 2× deposit at court's discretion)",
        ),
        Regime::Oregon => (
            31_u32,
            0_i64,
            false,
            true,
            "ORS 90.300(13) (31-day itemized accounting; statutory depreciation for \
             finite-useful-life items)",
        ),
        Regime::Florida => (
            30_u32,
            FL_RECEIPT_THRESHOLD_CENTS,
            false,
            true,
            "Fla. Stat. § 83.49(3) (30-day notice by certified mail to last known address; \
             $200 third-party invoice threshold per state practice; depreciation required for \
             finite-life items; § 83.49(3)(c) bad faith → court costs + reasonable attorney \
             fees)",
        ),
        Regime::Texas => (
            30_u32,
            0_i64,
            false,
            false,
            "Tex. Prop. Code § 92.103 (30-day refund); § 92.104(c) (itemized list if rent \
             fully paid + no controversy); § 92.109 (bad faith → $100 + 3× retained + \
             reasonable attorney fees — steepest statutory penalty)",
        ),
    };

    if input.days_to_itemized_statement > deadline_days {
        compliant = false;
        violations.push(format!(
            "Itemized statement delivered {} days after move-out; statutory deadline is {} \
             days.",
            input.days_to_itemized_statement, deadline_days,
        ));
    }

    if receipt_threshold_cents > 0
        && input.total_deductions_cents > receipt_threshold_cents
        && !input.receipts_attached_for_items_above_threshold
    {
        compliant = false;
        violations.push(format!(
            "Deductions of {} cents exceed the {} cents receipt-attachment threshold without \
             accompanying receipts, invoices, or contractor estimates.",
            input.total_deductions_cents, receipt_threshold_cents,
        ));
    }

    if photographic_required && !input.photographs_taken_before_and_after_repairs {
        compliant = false;
        violations.push(
            "California AB 2801: before-and-after photographs of the unit are mandatory for \
             any repair or cleaning deduction. Photographs must be retained for at least four \
             years and made available to the tenant on request."
                .to_string(),
        );
    }

    if input.deduction_includes_ordinary_wear_and_tear {
        compliant = false;
        violations.push(
            "Deduction includes ordinary wear and tear; ordinary-wear-and-tear exclusion bars \
             this charge across all regimes (statutory in CA / WA / OR / FL / TX)."
                .to_string(),
        );
    }

    if depreciation_required && !input.depreciation_applied_to_finite_life_items {
        compliant = false;
        violations.push(
            "Regime requires depreciation for finite-useful-life items (paint, carpet, \
             appliances). Charging full replacement cost — the new-for-old problem — violates \
             the wear-and-tear allocation."
                .to_string(),
        );
    }

    let statutory_bad_faith_penalty_cents = if input.bad_faith_claim {
        match input.regime {
            // California § 1950.5(l) — up to 2× the amount withheld.
            Regime::California => input.amount_withheld_cents.saturating_mul(2),
            // Washington RCW 59.18.280 — court may award up to 2× the
            // deposit for intentional refusal.
            Regime::Washington => input.amount_withheld_cents.saturating_mul(2),
            // Oregon ORS 90.300 — court may award 2× the amount
            // wrongfully withheld plus the amount itself; we expose
            // 2× exposure here as the statutory multiplier.
            Regime::Oregon => input.amount_withheld_cents.saturating_mul(2),
            // Florida § 83.49(3)(c) — court costs + reasonable
            // attorney fees; no statutory multiplier on the deposit
            // itself.
            Regime::Florida => 0,
            // Texas § 92.109 — $100 + 3× retained + reasonable
            // attorney fees.
            Regime::Texas => {
                10_000_i64.saturating_add(input.amount_withheld_cents.saturating_mul(3))
            }
        }
    } else {
        0
    };

    notes.push(
        "Itemization is the move-out-accounting half of the security-deposit lifecycle. \
         Pair with deposit_return_windows (deadline), security_deposit_caps (amount), \
         deposit_interest (accrual), and pre_move_out_inspection (cure-period inspection) for \
         end-to-end coverage."
            .to_string(),
    );

    if matches!(input.regime, Regime::California) {
        notes.push(
            "California AB 2801 also requires pre-tenancy photographs for tenancies that begin \
             on or after 2025-07-01. The before-and-after-repair photographs apply to all \
             tenancies after 2025-04-01."
                .to_string(),
        );
    }

    CheckResult {
        compliant,
        deadline_days,
        receipt_threshold_cents,
        photographic_documentation_required: photographic_required,
        depreciation_required,
        statutory_bad_faith_penalty_cents,
        violations,
        citation,
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base(regime: Regime) -> Input {
        Input {
            regime,
            days_to_itemized_statement: 14,
            total_deductions_cents: 5000,
            receipts_attached_for_items_above_threshold: true,
            photographs_taken_before_and_after_repairs: true,
            deduction_includes_ordinary_wear_and_tear: false,
            depreciation_applied_to_finite_life_items: true,
            bad_faith_claim: false,
            amount_withheld_cents: 0,
        }
    }

    // ── California § 1950.5 + AB 2801 ───────────────────────────

    #[test]
    fn california_full_compliance_grantable() {
        let r = check(&base(Regime::California));
        assert!(r.compliant);
        assert_eq!(r.deadline_days, 21);
        assert_eq!(r.receipt_threshold_cents, CA_RECEIPT_THRESHOLD_CENTS);
        assert!(r.photographic_documentation_required);
        assert!(r.depreciation_required);
        assert!(r.citation.contains("§ 1950.5(g)(1)"));
        assert!(r.citation.contains("AB 2801"));
    }

    #[test]
    fn california_past_21_day_deadline_violation() {
        let mut i = base(Regime::California);
        i.days_to_itemized_statement = 22;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("22") && v.contains("21")));
    }

    #[test]
    fn california_at_21_day_boundary_compliant() {
        let mut i = base(Regime::California);
        i.days_to_itemized_statement = 21;
        let r = check(&i);
        assert!(r.compliant);
    }

    #[test]
    fn california_125_dollar_threshold_no_receipts_violation() {
        let mut i = base(Regime::California);
        i.total_deductions_cents = 20000;
        i.receipts_attached_for_items_above_threshold = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("receipt-attachment threshold")));
    }

    #[test]
    fn california_at_125_dollar_threshold_no_receipts_still_compliant() {
        // Exactly $125 = at threshold, NOT above. Receipts not required.
        let mut i = base(Regime::California);
        i.total_deductions_cents = CA_RECEIPT_THRESHOLD_CENTS;
        i.receipts_attached_for_items_above_threshold = false;
        let r = check(&i);
        assert!(r.compliant);
    }

    #[test]
    fn california_ab2801_no_photographs_violation() {
        let mut i = base(Regime::California);
        i.photographs_taken_before_and_after_repairs = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("AB 2801") && v.contains("photographs")));
    }

    #[test]
    fn california_bad_faith_2x_withheld_amount() {
        let mut i = base(Regime::California);
        i.bad_faith_claim = true;
        i.amount_withheld_cents = 150000;
        let r = check(&i);
        assert_eq!(r.statutory_bad_faith_penalty_cents, 300000);
    }

    // ── Washington RCW 59.18.280 ───────────────────────────────

    #[test]
    fn washington_full_compliance_grantable() {
        let r = check(&base(Regime::Washington));
        assert!(r.compliant);
        assert_eq!(r.deadline_days, 30);
        assert_eq!(r.receipt_threshold_cents, 0);
        assert!(!r.photographic_documentation_required);
        assert!(r.depreciation_required);
        assert!(r.citation.contains("RCW 59.18.280"));
    }

    #[test]
    fn washington_past_30_day_deadline_violation() {
        let mut i = base(Regime::Washington);
        i.days_to_itemized_statement = 31;
        let r = check(&i);
        assert!(!r.compliant);
    }

    #[test]
    fn washington_at_30_day_boundary_compliant() {
        let mut i = base(Regime::Washington);
        i.days_to_itemized_statement = 30;
        let r = check(&i);
        assert!(r.compliant);
    }

    #[test]
    fn washington_no_receipt_threshold_no_violation_on_large_deduction() {
        // WA has no statutory threshold — large deduction without
        // attached receipts is not, by itself, a § 5918.280
        // violation (other claims must still pass the wear-and-tear
        // bar).
        let mut i = base(Regime::Washington);
        i.total_deductions_cents = 500000;
        i.receipts_attached_for_items_above_threshold = false;
        let r = check(&i);
        assert!(r.compliant);
    }

    #[test]
    fn washington_bad_faith_2x_deposit_exposure() {
        let mut i = base(Regime::Washington);
        i.bad_faith_claim = true;
        i.amount_withheld_cents = 200000;
        let r = check(&i);
        assert_eq!(r.statutory_bad_faith_penalty_cents, 400000);
    }

    // ── Oregon ORS 90.300 ──────────────────────────────────────

    #[test]
    fn oregon_full_compliance_grantable() {
        let r = check(&base(Regime::Oregon));
        assert!(r.compliant);
        assert_eq!(r.deadline_days, 31);
        assert!(r.depreciation_required);
        assert!(r.citation.contains("ORS 90.300"));
    }

    #[test]
    fn oregon_no_depreciation_violation() {
        let mut i = base(Regime::Oregon);
        i.depreciation_applied_to_finite_life_items = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("depreciation") && v.contains("new-for-old")));
    }

    // ── Florida Fla. Stat. § 83.49(3) ───────────────────────────

    #[test]
    fn florida_full_compliance_grantable() {
        let r = check(&base(Regime::Florida));
        assert!(r.compliant);
        assert_eq!(r.deadline_days, 30);
        assert_eq!(r.receipt_threshold_cents, FL_RECEIPT_THRESHOLD_CENTS);
        assert!(r.depreciation_required);
        assert!(r.citation.contains("§ 83.49(3)"));
    }

    #[test]
    fn florida_200_dollar_threshold_no_receipts_violation() {
        let mut i = base(Regime::Florida);
        i.total_deductions_cents = 30000;
        i.receipts_attached_for_items_above_threshold = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("200") && v.contains("threshold")));
    }

    #[test]
    fn florida_bad_faith_no_statutory_multiplier_only_fees() {
        let mut i = base(Regime::Florida);
        i.bad_faith_claim = true;
        i.amount_withheld_cents = 200000;
        let r = check(&i);
        assert_eq!(r.statutory_bad_faith_penalty_cents, 0);
    }

    // ── Texas Tex. Prop. Code § 92.103 + § 92.109 ──────────────

    #[test]
    fn texas_full_compliance_grantable() {
        let r = check(&base(Regime::Texas));
        assert!(r.compliant);
        assert_eq!(r.deadline_days, 30);
        assert!(!r.depreciation_required);
        assert!(r.citation.contains("§ 92.109"));
    }

    #[test]
    fn texas_no_depreciation_not_a_statutory_violation() {
        let mut i = base(Regime::Texas);
        i.depreciation_applied_to_finite_life_items = false;
        let r = check(&i);
        // Texas does not statutorily require depreciation — only
        // wear-and-tear must be excluded. Lack of depreciation alone
        // is not a § 92.104 violation.
        assert!(r.compliant);
    }

    #[test]
    fn texas_bad_faith_steepest_penalty_100_plus_3x() {
        let mut i = base(Regime::Texas);
        i.bad_faith_claim = true;
        i.amount_withheld_cents = 100000;
        let r = check(&i);
        // $100 + 3 × $1000 = $3100.
        assert_eq!(r.statutory_bad_faith_penalty_cents, 310000);
    }

    // ── Wear-and-tear exclusion universal ──────────────────────

    #[test]
    fn wear_and_tear_excluded_across_all_regimes_invariant() {
        for &regime in &[
            Regime::California,
            Regime::Washington,
            Regime::Oregon,
            Regime::Florida,
            Regime::Texas,
        ] {
            let mut i = base(regime);
            i.deduction_includes_ordinary_wear_and_tear = true;
            let r = check(&i);
            assert!(
                !r.compliant,
                "{:?}: ordinary wear-and-tear deduction must violate",
                regime,
            );
        }
    }

    // ── Multi-regime regression invariants ─────────────────────

    #[test]
    fn only_california_imposes_ab2801_photographic_mandate_5_regime_invariant() {
        let ca = check(&base(Regime::California));
        assert!(ca.photographic_documentation_required);
        for &regime in &[
            Regime::Washington,
            Regime::Oregon,
            Regime::Florida,
            Regime::Texas,
        ] {
            let r = check(&base(regime));
            assert!(
                !r.photographic_documentation_required,
                "{:?}: must NOT require photographic documentation",
                regime,
            );
        }
    }

    #[test]
    fn only_texas_lacks_statutory_depreciation_requirement_5_regime_invariant() {
        let tx = check(&base(Regime::Texas));
        assert!(!tx.depreciation_required);
        for &regime in &[
            Regime::California,
            Regime::Washington,
            Regime::Oregon,
            Regime::Florida,
        ] {
            let r = check(&base(regime));
            assert!(
                r.depreciation_required,
                "{:?}: must require statutory depreciation",
                regime,
            );
        }
    }

    #[test]
    fn only_california_has_21_day_shortest_deadline_5_regime_invariant() {
        let ca = check(&base(Regime::California));
        assert_eq!(ca.deadline_days, 21);
        for &regime in &[Regime::Washington, Regime::Florida, Regime::Texas] {
            assert_eq!(check(&base(regime)).deadline_days, 30);
        }
        assert_eq!(check(&base(Regime::Oregon)).deadline_days, 31);
    }

    #[test]
    fn only_california_and_florida_have_receipt_thresholds_invariant() {
        assert_eq!(
            check(&base(Regime::California)).receipt_threshold_cents,
            CA_RECEIPT_THRESHOLD_CENTS,
        );
        assert_eq!(
            check(&base(Regime::Florida)).receipt_threshold_cents,
            FL_RECEIPT_THRESHOLD_CENTS,
        );
        for &regime in &[Regime::Washington, Regime::Oregon, Regime::Texas] {
            assert_eq!(
                check(&base(regime)).receipt_threshold_cents,
                0,
                "{:?}: must have no statutory receipt threshold",
                regime,
            );
        }
    }

    #[test]
    fn texas_has_steepest_bad_faith_penalty_5_regime_invariant() {
        // $1000 withheld → TX = $100 + $3000 = $3100; CA + WA + OR =
        // $2000; FL = $0. TX must be highest.
        let mut samples: Vec<(Regime, i64)> = Vec::new();
        for &regime in &[
            Regime::California,
            Regime::Washington,
            Regime::Oregon,
            Regime::Florida,
            Regime::Texas,
        ] {
            let mut i = base(regime);
            i.bad_faith_claim = true;
            i.amount_withheld_cents = 100000;
            samples.push((regime, check(&i).statutory_bad_faith_penalty_cents));
        }
        let texas_value = samples.iter().find(|(r, _)| *r == Regime::Texas).unwrap().1;
        for &(regime, value) in samples.iter() {
            if regime == Regime::Texas {
                continue;
            }
            assert!(
                value < texas_value,
                "{:?}: {} must be less than Texas {}",
                regime,
                value,
                texas_value,
            );
        }
    }

    #[test]
    fn no_bad_faith_zero_penalty_across_all_regimes_invariant() {
        for &regime in &[
            Regime::California,
            Regime::Washington,
            Regime::Oregon,
            Regime::Florida,
            Regime::Texas,
        ] {
            let r = check(&base(regime));
            assert_eq!(
                r.statutory_bad_faith_penalty_cents, 0,
                "{:?}: no bad-faith claim → penalty must be zero",
                regime,
            );
        }
    }

    #[test]
    fn citation_pins_authority_per_regime() {
        assert!(check(&base(Regime::California))
            .citation
            .contains("§ 1950.5(g)(2)"));
        assert!(check(&base(Regime::Washington))
            .citation
            .contains("RCW 59.18.280"));
        assert!(check(&base(Regime::Oregon))
            .citation
            .contains("ORS 90.300(13)"));
        assert!(check(&base(Regime::Florida))
            .citation
            .contains("§ 83.49(3)"));
        assert!(check(&base(Regime::Texas)).citation.contains("§ 92.109"));
    }

    #[test]
    fn note_pairs_module_with_sibling_modules() {
        let r = check(&base(Regime::California));
        assert!(
            r.notes.iter().any(|n| n.contains("deposit_return_windows")
                && n.contains("security_deposit_caps")
                && n.contains("deposit_interest")
                && n.contains("pre_move_out_inspection")),
            "must document the sibling-module relationships"
        );
    }

    #[test]
    fn california_ab2801_pretenancy_photograph_note_present() {
        let r = check(&base(Regime::California));
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("pre-tenancy photographs") && n.contains("2025-07-01")),
            "CA AB 2801 pre-tenancy photo note must be present"
        );
    }
}
