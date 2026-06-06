//! IRC § 1281 — Current inclusion in income of acquisition discount
//! on certain short-term obligations.
//!
//! The bookend to the OID cluster: § 1272 governs long-term OID;
//! § 1281 governs short-term obligations (≤ 1 year to maturity).
//! § 1272(a)(2)(C) and § 1271(a)(3)/(a)(4) cross-reference § 1281
//! repeatedly as the operative provision for short-term obligation
//! current accrual.
//!
//! Critical distinction from § 1272: § 1281 applies ONLY to SPECIFIC
//! holder categories. Cash-method individual investors holding
//! short-term obligations are OUTSIDE § 1281's scope and defer
//! acquisition discount until disposition under § 1271(a)(3) /
//! (a)(4) ratable accrual at sale.
//!
//! Direct companion to:
//!   - `section_1271` (retirement of debt — § 1271(a)(3)/(a)(4)
//!     ratable accrual at disposition for short-term obligations
//!     held by cash-method individual investors).
//!   - `section_1272` (current inclusion of long-term OID —
//!     § 1272(a)(2)(C) short-term carve-out).
//!   - `section_1273` (OID definition).
//!
//! Eight holder categories:
//!
//!   § 1281(b)(1)(A) — Accrual-method taxpayer.
//!   § 1281(b)(1)(B) — Held primarily for sale to customers in
//!     ordinary course of taxpayer's trade or business (dealer).
//!   § 1281(b)(1)(C) — Held by a bank (as defined in § 581).
//!   § 1281(b)(1)(D) — Held by a regulated investment company or a
//!     common trust fund.
//!   § 1281(b)(1)(E) — Identified by taxpayer under § 1256(e)(2) as
//!     part of a hedging transaction.
//!   § 1281(b)(1)(F) — Stripped bond or coupon held by person who
//!     stripped it.
//!   § 1281(b)(2) — Certain pass-thru entities (5%+ ownership
//!     control during required accrual periods).
//!   **Cash-method individual investor — OUTSIDE § 1281**.
//!
//! § 1281(c) cross-references § 1283(c) for the nongovernmental-
//! obligation OID-only limitation. For governmental short-term
//! obligations (T-bills), the full acquisition discount accrues;
//! for nongovernmental short-term obligations, only the OID
//! component accrues currently.
//!
//! Citations: 26 U.S.C. § 1281(a) (general current-inclusion rule);
//! § 1281(b)(1)(A)–(F) (in-scope holder categories); § 1281(b)(2)
//! (pass-thru entities); § 1281(c) (cross-reference to § 1283);
//! § 1283(a)(1) (short-term obligation definition — ≤ 1 year to
//! maturity); § 1283(a)(2) (acquisition discount definition);
//! § 1283(c) (nongovernmental-obligation OID limitation); § 581
//! (bank definition); § 1256(e)(2) (hedging transaction
//! identification).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HolderCategory {
    /// § 1281(b)(1)(A) — taxpayer using accrual method of accounting.
    AccrualMethod,
    /// § 1281(b)(1)(B) — dealer holding short-term obligation
    /// primarily for sale to customers.
    DealerInSecurities,
    /// § 1281(b)(1)(C) — bank (as defined in § 581).
    Bank,
    /// § 1281(b)(1)(D) — regulated investment company (RIC) or
    /// common trust fund.
    RegulatedInvestmentCompany,
    /// § 1281(b)(1)(E) — § 1256(e)(2) hedging-transaction
    /// identified instrument.
    HedgingTransaction,
    /// § 1281(b)(1)(F) — stripped bond or coupon held by person
    /// who stripped it.
    StripperStrippedBond,
    /// § 1281(b)(2) — pass-thru entity (5%+ ownership control
    /// during required accrual periods).
    PassThruEntity,
    /// Cash-method individual investor — OUTSIDE § 1281 scope.
    /// Defers to § 1271(a)(3)/(a)(4) ratable accrual at
    /// disposition.
    CashMethodIndividual,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ObligationType {
    /// Governmental short-term obligation (T-bill, federal agency
    /// note). § 1281(a) applies to full acquisition discount.
    Governmental,
    /// Nongovernmental short-term obligation (corporate paper,
    /// commercial paper). § 1281(c) + § 1283(c) limit accrual to
    /// OID component only.
    NonGovernmental,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section1281Input {
    pub holder_category: HolderCategory,
    pub obligation_type: ObligationType,
    /// § 1283(a)(2) acquisition discount = SRPM − basis at
    /// acquisition (cents). For governmental obligations this is
    /// the full accrual base; for nongovernmental § 1283(c)
    /// limits to OID component.
    pub acquisition_discount_cents: i64,
    /// § 1283(c) OID component for nongovernmental obligations
    /// (cents). Used in lieu of acquisition_discount for
    /// nongovernmental obligations.
    pub oid_component_cents: i64,
    /// Number of days the holder held the short-term obligation
    /// during the taxable year.
    pub days_held_in_year: u32,
    /// Total days from acquisition to maturity. Used as
    /// denominator in ratable daily-portion computation.
    pub days_from_acquisition_to_maturity: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section1281Result {
    /// True if the holder category is in scope under § 1281(b)(1)
    /// or § 1281(b)(2).
    pub current_inclusion_required: bool,
    /// Effective accrual base after § 1283(c) nongovernmental
    /// limitation (cents).
    pub accrual_base_cents: i64,
    /// Daily portion total after proration by days held (cents).
    pub daily_portion_total_cents: i64,
    /// Current-year inclusion (cents). Zero where holder category
    /// is outside § 1281 scope.
    pub current_year_inclusion_cents: i64,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn compute(input: &Section1281Input) -> Section1281Result {
    let mut notes: Vec<String> = Vec::new();

    // Cash-method individual investor — OUTSIDE § 1281 scope.
    if matches!(input.holder_category, HolderCategory::CashMethodIndividual) {
        notes.push(
            "Cash-method individual investor — OUTSIDE § 1281 scope. Acquisition discount on \
             short-term obligation defers to § 1271(a)(3) (governmental) or § 1271(a)(4) \
             (nongovernmental) ratable accrual at disposition."
                .to_string(),
        );
        return Section1281Result {
            current_inclusion_required: false,
            accrual_base_cents: 0,
            daily_portion_total_cents: 0,
            current_year_inclusion_cents: 0,
            citation: "26 U.S.C. § 1281(b)(1) (holder category outside scope — cash-method \
                       individual); § 1271(a)(3)/(a)(4) (ratable accrual at disposition)",
            notes,
        };
    }

    // § 1283(c) — nongovernmental obligations limited to OID.
    let accrual_base = match input.obligation_type {
        ObligationType::Governmental => input.acquisition_discount_cents.max(0),
        ObligationType::NonGovernmental => {
            notes.push(
                "§ 1283(c) — nongovernmental short-term obligation; current accrual limited to \
                 OID component (not full acquisition discount)."
                    .to_string(),
            );
            input.oid_component_cents.max(0)
        }
    };

    // Daily-portion math — ratable allocation by days held over
    // total days from acquisition to maturity.
    let days_total = input.days_from_acquisition_to_maturity.max(1) as i64;
    let days_held = input
        .days_held_in_year
        .min(input.days_from_acquisition_to_maturity.max(1)) as i64;
    let daily_portion = accrual_base.saturating_mul(days_held) / days_total;

    // Holder-category-specific note.
    let category_note = match input.holder_category {
        HolderCategory::AccrualMethod => {
            "§ 1281(b)(1)(A) — accrual-method taxpayer; § 1281 current inclusion required."
        }
        HolderCategory::DealerInSecurities => {
            "§ 1281(b)(1)(B) — dealer holding short-term obligation primarily for sale to \
             customers; § 1281 current inclusion required."
        }
        HolderCategory::Bank => {
            "§ 1281(b)(1)(C) — bank (§ 581 definition); § 1281 current inclusion required."
        }
        HolderCategory::RegulatedInvestmentCompany => {
            "§ 1281(b)(1)(D) — regulated investment company or common trust fund; § 1281 \
             current inclusion required."
        }
        HolderCategory::HedgingTransaction => {
            "§ 1281(b)(1)(E) — § 1256(e)(2) hedging-transaction identified instrument; \
             § 1281 current inclusion required."
        }
        HolderCategory::StripperStrippedBond => {
            "§ 1281(b)(1)(F) — stripped bond or coupon held by person who stripped it; \
             § 1281 current inclusion required."
        }
        HolderCategory::PassThruEntity => {
            "§ 1281(b)(2) — pass-thru entity with 5%+ ownership control during required \
             accrual periods; § 1281 current inclusion required."
        }
        HolderCategory::CashMethodIndividual => unreachable!(),
    };
    notes.push(category_note.to_string());

    let citation = match input.holder_category {
        HolderCategory::AccrualMethod => {
            "26 U.S.C. § 1281(a) (general current-inclusion rule); § 1281(b)(1)(A) (accrual-\
             method taxpayer); § 1283(a)(1) (short-term obligation definition)"
        }
        HolderCategory::DealerInSecurities => {
            "26 U.S.C. § 1281(a); § 1281(b)(1)(B) (dealer in securities — primarily for sale \
             to customers); § 1283(a)(1)"
        }
        HolderCategory::Bank => {
            "26 U.S.C. § 1281(a); § 1281(b)(1)(C) (bank — § 581 definition); § 1283(a)(1)"
        }
        HolderCategory::RegulatedInvestmentCompany => {
            "26 U.S.C. § 1281(a); § 1281(b)(1)(D) (regulated investment company or common \
             trust fund); § 1283(a)(1)"
        }
        HolderCategory::HedgingTransaction => {
            "26 U.S.C. § 1281(a); § 1281(b)(1)(E) (§ 1256(e)(2) hedging-transaction identified); \
             § 1283(a)(1)"
        }
        HolderCategory::StripperStrippedBond => {
            "26 U.S.C. § 1281(a); § 1281(b)(1)(F) (stripped bond / coupon held by stripper); \
             § 1283(a)(1)"
        }
        HolderCategory::PassThruEntity => {
            "26 U.S.C. § 1281(a); § 1281(b)(2) (pass-thru entity 5%+ ownership); § 1283(a)(1)"
        }
        HolderCategory::CashMethodIndividual => unreachable!(),
    };

    Section1281Result {
        current_inclusion_required: true,
        accrual_base_cents: accrual_base,
        daily_portion_total_cents: daily_portion,
        current_year_inclusion_cents: daily_portion,
        citation,
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        category: HolderCategory,
        obligation: ObligationType,
        acq_disc: i64,
        oid_component: i64,
        days_held: u32,
        days_total: u32,
    ) -> Section1281Input {
        Section1281Input {
            holder_category: category,
            obligation_type: obligation,
            acquisition_discount_cents: acq_disc,
            oid_component_cents: oid_component,
            days_held_in_year: days_held,
            days_from_acquisition_to_maturity: days_total,
        }
    }

    // ── § 1281(b)(1)(A) accrual-method taxpayer ─────────────────

    #[test]
    fn accrual_method_governmental_full_period_full_inclusion() {
        // T-bill $10,000 acquisition discount; held 180 of 360
        // days. Accrual base = full $10,000 (governmental).
        // Daily portion = 10,000 × 180 / 360 = 5,000.
        let r = compute(&input(
            HolderCategory::AccrualMethod,
            ObligationType::Governmental,
            10_000,
            0,
            180,
            360,
        ));
        assert!(r.current_inclusion_required);
        assert_eq!(r.accrual_base_cents, 10_000);
        assert_eq!(r.daily_portion_total_cents, 5_000);
        assert_eq!(r.current_year_inclusion_cents, 5_000);
        assert!(r.citation.contains("§ 1281(a)"));
        assert!(r.citation.contains("§ 1281(b)(1)(A)"));
    }

    #[test]
    fn accrual_method_held_full_period_full_discount() {
        let r = compute(&input(
            HolderCategory::AccrualMethod,
            ObligationType::Governmental,
            10_000,
            0,
            360,
            360,
        ));
        assert_eq!(r.daily_portion_total_cents, 10_000);
    }

    // ── § 1281(b)(1)(B) dealer ───────────────────────────────────

    #[test]
    fn dealer_in_securities_in_scope() {
        let r = compute(&input(
            HolderCategory::DealerInSecurities,
            ObligationType::Governmental,
            10_000,
            0,
            180,
            360,
        ));
        assert!(r.current_inclusion_required);
        assert!(r.citation.contains("§ 1281(b)(1)(B)"));
    }

    // ── § 1281(b)(1)(C) bank ────────────────────────────────────

    #[test]
    fn bank_in_scope_section_581_definition() {
        let r = compute(&input(
            HolderCategory::Bank,
            ObligationType::Governmental,
            10_000,
            0,
            180,
            360,
        ));
        assert!(r.current_inclusion_required);
        assert!(r.citation.contains("§ 581"));
    }

    // ── § 1281(b)(1)(D) RIC ─────────────────────────────────────

    #[test]
    fn regulated_investment_company_in_scope() {
        let r = compute(&input(
            HolderCategory::RegulatedInvestmentCompany,
            ObligationType::Governmental,
            10_000,
            0,
            180,
            360,
        ));
        assert!(r.current_inclusion_required);
        assert!(r.citation.contains("§ 1281(b)(1)(D)"));
    }

    // ── § 1281(b)(1)(E) hedging transaction ─────────────────────

    #[test]
    fn hedging_transaction_in_scope() {
        let r = compute(&input(
            HolderCategory::HedgingTransaction,
            ObligationType::Governmental,
            10_000,
            0,
            180,
            360,
        ));
        assert!(r.current_inclusion_required);
        assert!(r.citation.contains("§ 1256(e)(2)"));
    }

    // ── § 1281(b)(1)(F) stripped bond ───────────────────────────

    #[test]
    fn stripper_stripped_bond_in_scope() {
        let r = compute(&input(
            HolderCategory::StripperStrippedBond,
            ObligationType::Governmental,
            10_000,
            0,
            180,
            360,
        ));
        assert!(r.current_inclusion_required);
        assert!(r.citation.contains("§ 1281(b)(1)(F)"));
    }

    // ── § 1281(b)(2) pass-thru entity ──────────────────────────

    #[test]
    fn pass_thru_entity_in_scope() {
        let r = compute(&input(
            HolderCategory::PassThruEntity,
            ObligationType::Governmental,
            10_000,
            0,
            180,
            360,
        ));
        assert!(r.current_inclusion_required);
        assert!(r.citation.contains("§ 1281(b)(2)"));
    }

    // ── Cash-method individual — OUTSIDE scope ──────────────────

    #[test]
    fn cash_method_individual_outside_scope_defers_to_1271() {
        let r = compute(&input(
            HolderCategory::CashMethodIndividual,
            ObligationType::Governmental,
            10_000,
            0,
            180,
            360,
        ));
        assert!(!r.current_inclusion_required);
        assert_eq!(r.current_year_inclusion_cents, 0);
        assert!(r.citation.contains("outside scope"));
        assert!(r.citation.contains("§ 1271(a)(3)/(a)(4)"));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 1271(a)(3)") && n.contains("disposition")));
    }

    // ── § 1283(c) nongovernmental OID limitation ───────────────

    #[test]
    fn non_governmental_limited_to_oid_component() {
        // Acquisition discount $10,000 BUT OID component only $4,000.
        // Accrual base = $4,000 (not full discount).
        let r = compute(&input(
            HolderCategory::Bank,
            ObligationType::NonGovernmental,
            10_000,
            4_000,
            180,
            360,
        ));
        assert_eq!(r.accrual_base_cents, 4_000);
        // 4_000 × 180 / 360 = 2_000.
        assert_eq!(r.daily_portion_total_cents, 2_000);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 1283(c)") && n.contains("nongovernmental")));
    }

    #[test]
    fn governmental_vs_nongovernmental_obligation_base_invariant() {
        // Same total acquisition discount + same OID component.
        // Gov path uses full discount; Non-gov uses OID only.
        let gov = compute(&input(
            HolderCategory::Bank,
            ObligationType::Governmental,
            10_000,
            4_000,
            360,
            360,
        ));
        let non_gov = compute(&input(
            HolderCategory::Bank,
            ObligationType::NonGovernmental,
            10_000,
            4_000,
            360,
            360,
        ));
        assert_eq!(gov.accrual_base_cents, 10_000);
        assert_eq!(non_gov.accrual_base_cents, 4_000);
    }

    // ── Edge cases ─────────────────────────────────────────────

    #[test]
    fn zero_days_held_zero_inclusion() {
        let r = compute(&input(
            HolderCategory::AccrualMethod,
            ObligationType::Governmental,
            10_000,
            0,
            0,
            360,
        ));
        assert_eq!(r.daily_portion_total_cents, 0);
    }

    #[test]
    fn zero_total_days_uses_min_1_denominator() {
        // Defensive — should not panic, should produce full
        // discount (held ≥ 0).
        let r = compute(&input(
            HolderCategory::AccrualMethod,
            ObligationType::Governmental,
            10_000,
            0,
            0,
            0,
        ));
        assert_eq!(r.daily_portion_total_cents, 0);
    }

    #[test]
    fn days_held_exceeds_days_total_caps_at_total() {
        let r = compute(&input(
            HolderCategory::AccrualMethod,
            ObligationType::Governmental,
            10_000,
            0,
            500,
            360,
        ));
        // Capped at 360 of 360 → full discount.
        assert_eq!(r.daily_portion_total_cents, 10_000);
    }

    #[test]
    fn negative_acquisition_discount_clamps_at_zero() {
        let r = compute(&input(
            HolderCategory::AccrualMethod,
            ObligationType::Governmental,
            -1_000,
            0,
            180,
            360,
        ));
        assert_eq!(r.accrual_base_cents, 0);
        assert_eq!(r.daily_portion_total_cents, 0);
    }

    #[test]
    fn non_governmental_negative_oid_component_clamps_at_zero() {
        let r = compute(&input(
            HolderCategory::Bank,
            ObligationType::NonGovernmental,
            10_000,
            -500,
            180,
            360,
        ));
        assert_eq!(r.accrual_base_cents, 0);
    }

    // ── Regression-critical invariants ──────────────────────────

    #[test]
    fn cash_method_individual_only_holder_outside_scope_invariant() {
        for &category in &[
            HolderCategory::AccrualMethod,
            HolderCategory::DealerInSecurities,
            HolderCategory::Bank,
            HolderCategory::RegulatedInvestmentCompany,
            HolderCategory::HedgingTransaction,
            HolderCategory::StripperStrippedBond,
            HolderCategory::PassThruEntity,
        ] {
            let r = compute(&input(
                category,
                ObligationType::Governmental,
                10_000,
                0,
                180,
                360,
            ));
            assert!(
                r.current_inclusion_required,
                "{:?}: must require current inclusion",
                category,
            );
        }
        let r = compute(&input(
            HolderCategory::CashMethodIndividual,
            ObligationType::Governmental,
            10_000,
            0,
            180,
            360,
        ));
        assert!(!r.current_inclusion_required);
    }

    #[test]
    fn nongovernmental_base_never_exceeds_oid_component_invariant() {
        for oid in [0_i64, 1_000, 5_000, 10_000, 15_000] {
            let r = compute(&input(
                HolderCategory::Bank,
                ObligationType::NonGovernmental,
                20_000,
                oid,
                360,
                360,
            ));
            assert_eq!(
                r.accrual_base_cents,
                oid.max(0),
                "OID component {} must equal accrual base for nongovernmental",
                oid,
            );
        }
    }

    #[test]
    fn citation_pins_holder_category_subparagraph_per_type() {
        let categories = [
            (HolderCategory::AccrualMethod, "§ 1281(b)(1)(A)"),
            (HolderCategory::DealerInSecurities, "§ 1281(b)(1)(B)"),
            (HolderCategory::Bank, "§ 1281(b)(1)(C)"),
            (
                HolderCategory::RegulatedInvestmentCompany,
                "§ 1281(b)(1)(D)",
            ),
            (HolderCategory::HedgingTransaction, "§ 1281(b)(1)(E)"),
            (HolderCategory::StripperStrippedBond, "§ 1281(b)(1)(F)"),
            (HolderCategory::PassThruEntity, "§ 1281(b)(2)"),
        ];
        for (category, expected) in categories {
            let r = compute(&input(
                category,
                ObligationType::Governmental,
                10_000,
                0,
                180,
                360,
            ));
            assert!(
                r.citation.contains(expected),
                "{:?}: expected citation to contain {expected}",
                category,
            );
        }
    }

    #[test]
    fn note_documents_holder_category_per_type_invariant() {
        let categories = [
            (HolderCategory::AccrualMethod, "§ 1281(b)(1)(A)"),
            (HolderCategory::DealerInSecurities, "§ 1281(b)(1)(B)"),
            (HolderCategory::Bank, "§ 1281(b)(1)(C)"),
            (
                HolderCategory::RegulatedInvestmentCompany,
                "§ 1281(b)(1)(D)",
            ),
            (HolderCategory::HedgingTransaction, "§ 1281(b)(1)(E)"),
            (HolderCategory::StripperStrippedBond, "§ 1281(b)(1)(F)"),
            (HolderCategory::PassThruEntity, "§ 1281(b)(2)"),
        ];
        for (category, expected) in categories {
            let r = compute(&input(
                category,
                ObligationType::Governmental,
                10_000,
                0,
                180,
                360,
            ));
            assert!(
                r.notes.iter().any(|n| n.contains(expected)),
                "{:?}: note must contain {expected}",
                category,
            );
        }
    }

    #[test]
    fn proration_is_linear_in_days_held_invariant() {
        // 180/360 should equal half of 360/360.
        let half = compute(&input(
            HolderCategory::AccrualMethod,
            ObligationType::Governmental,
            10_000,
            0,
            180,
            360,
        ));
        let full = compute(&input(
            HolderCategory::AccrualMethod,
            ObligationType::Governmental,
            10_000,
            0,
            360,
            360,
        ));
        assert_eq!(
            half.daily_portion_total_cents * 2,
            full.daily_portion_total_cents
        );
    }
}
