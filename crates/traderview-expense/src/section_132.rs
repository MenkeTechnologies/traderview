//! IRC §132 — Certain fringe benefits excluded from gross income.
//!
//! Trader-relevant for W-2 employees of trading firms who receive
//! employer-provided fringe benefits. §132(a) enumerates 8
//! categories of fringes that are excluded from gross income.
//! The OBBBA 2025 made the TCJA-era §132(g) moving-expense
//! exclusion suspension PERMANENT (except for armed forces and
//! intelligence community).
//!
//! **§132(a) — 8 categories of excludable fringes**:
//!
//! - **(1) No-additional-cost service** — employer offers a
//!   service in the ordinary course of business in the line of
//!   business the employee works for; no substantial additional
//!   cost (lost revenue counts).
//! - **(2) Qualified employee discount** — discount in line of
//!   business. Goods: max discount = gross-profit percentage.
//!   Services: max discount = 20%.
//! - **(3) Working condition fringe** — property/service that
//!   would be deductible under §162/§167 if the employee paid for
//!   it directly.
//! - **(4) De minimis fringe** — so small accounting is
//!   administratively impracticable.
//! - **(5) Qualified transportation fringe** — see §132(f)
//!   inflation-adjusted monthly caps below.
//! - **(6) Qualified moving expense reimbursement** —
//!   **PERMANENTLY ELIMINATED** by OBBBA 2025 (P.L. 119-21)
//!   EXCEPT for U.S. Armed Forces active-duty members and (newly
//!   added by OBBBA) U.S. intelligence community members.
//! - **(7) Qualified retirement planning services** — investment
//!   advice provided to plan participants.
//! - **(8) Qualified military base realignment and closure
//!   fringe** — narrow.
//!
//! **§132(f) qualified transportation fringe monthly caps**
//! (Rev. Proc. inflation-adjusted, caller-supplied year-agnostic):
//!
//! | Year | Parking | Transit / vanpool |
//! |------|---------|---------------------|
//! | 2024 | $315    | $315                |
//! | 2025 | $325    | $325                |
//! | 2026 | $340    | $340                |
//!
//! Each category is separately capped (not aggregated) — a
//! commuter using both parking AND transit can get $340 + $340 =
//! $680 of monthly exclusion in 2026.
//!
//! Sources:
//! [Cornell LII 26 U.S.C. § 132](https://www.law.cornell.edu/uscode/text/26/132),
//! [IRS Pub. 15-B (2026) Employer's Tax Guide to Fringe Benefits](https://www.irs.gov/publications/p15b),
//! [WorldatWork — IRS 2026 Limits for FSAs, Transport Fringe Benefits](https://worldatwork.org/publications/workspan-daily/irs-provides-2026-limits-for-fsas-transport-fringe-benefits-etc),
//! [Foster Tax Law — OBBBA Part VIII Worker Moving Expenses](https://www.foster.com/larry-s-tax-law/one-big-beautiful-bill-act-part-8-worker-moving-expenses),
//! [IRS — One Big Beautiful Bill provisions](https://www.irs.gov/newsroom/one-big-beautiful-bill-provisions).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FringeCategory {
    /// §132(a)(1) no-additional-cost service.
    NoAdditionalCostService,
    /// §132(a)(2) qualified employee discount.
    QualifiedEmployeeDiscount,
    /// §132(a)(3) working condition fringe.
    WorkingConditionFringe,
    /// §132(a)(4) de minimis fringe.
    DeMinimisFringe,
    /// §132(a)(5) qualified transportation fringe (parking,
    /// transit pass, vanpool, bicycle commuting — see §132(f)).
    QualifiedTransportationFringe,
    /// §132(a)(6) qualified moving expense reimbursement.
    QualifiedMovingExpenseReimbursement,
    /// §132(a)(7) qualified retirement planning services.
    QualifiedRetirementPlanningServices,
    /// §132(a)(8) qualified military base realignment and closure
    /// fringe.
    QualifiedMilitaryBaseRealignmentClosure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransportationFringeType {
    /// Qualified parking near workplace or commuter facility.
    Parking,
    /// Transit pass / vanpool / commuter highway vehicle.
    TransitOrVanpool,
    /// Combined (rare — typically separate per-category caps).
    Both,
    /// Not a transportation fringe.
    NotApplicable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section132Input {
    pub tax_year: i32,
    pub fringe_category: FringeCategory,
    /// FMV of the fringe benefit received.
    pub fringe_value_dollars: i64,
    /// For §132(f) qualified transportation fringe: type and
    /// inflation-adjusted monthly cap. Caller supplies year-agnostic.
    pub transportation_fringe_type: TransportationFringeType,
    /// Monthly cap for parking (Rev. Proc. inflation-adjusted;
    /// 2026 = $340).
    pub parking_monthly_cap_dollars: i64,
    /// Monthly cap for transit/vanpool (2026 = $340).
    pub transit_monthly_cap_dollars: i64,
    /// For §132(a)(6) moving expense: true if recipient is a
    /// U.S. Armed Forces active-duty member moving on PCS orders
    /// (statutory exception) or (per OBBBA) U.S. intelligence
    /// community member.
    pub is_armed_forces_pcs_or_intelligence: bool,
    /// For §132(a)(2) discount: discount percentage offered (e.g.,
    /// 25 = 25%). Cap is 20% for services / gross-profit % for
    /// goods.
    pub discount_pct_bp: u32,
    /// True if discount is for services (capped at 20%) vs goods.
    pub discount_is_for_services: bool,
    /// Gross profit percentage for goods discount (in basis
    /// points; e.g., 3000 = 30%).
    pub gross_profit_pct_bp: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section132Result {
    pub fringe_category: FringeCategory,
    pub excludable_amount_dollars: i64,
    pub taxable_amount_dollars: i64,
    /// True if the §132(a)(6) moving exclusion is PERMANENTLY
    /// suspended by OBBBA and no exception applies.
    pub moving_expense_permanently_suspended: bool,
    /// Maximum permitted under §132(f) inflation-adjusted cap
    /// (for transportation fringes).
    pub transportation_monthly_cap_applicable_dollars: i64,
    pub citation: String,
    pub note: String,
}

pub fn compute(input: &Section132Input) -> Section132Result {
    let value = input.fringe_value_dollars.max(0);
    let mut transportation_cap = 0i64;
    let mut moving_suspended = false;

    let excludable = match input.fringe_category {
        FringeCategory::NoAdditionalCostService
        | FringeCategory::WorkingConditionFringe
        | FringeCategory::DeMinimisFringe
        | FringeCategory::QualifiedRetirementPlanningServices
        | FringeCategory::QualifiedMilitaryBaseRealignmentClosure => {
            // Categorically excludable in full (assuming qualifying
            // conditions are met).
            value
        }
        FringeCategory::QualifiedEmployeeDiscount => {
            // §132(c)(1)(A) services: 20% cap. (B) goods:
            // gross-profit % cap.
            let cap_bp = if input.discount_is_for_services {
                2000
            } else {
                input.gross_profit_pct_bp
            };
            if input.discount_pct_bp <= cap_bp {
                value
            } else {
                // Discount exceeds cap → excess is taxable.
                // Approximation: excludable = value × (cap / discount_pct).
                ((value as i128) * (cap_bp as i128) / (input.discount_pct_bp.max(1) as i128)) as i64
            }
        }
        FringeCategory::QualifiedTransportationFringe => {
            // §132(f) monthly caps per category.
            transportation_cap = match input.transportation_fringe_type {
                TransportationFringeType::Parking => input.parking_monthly_cap_dollars.max(0),
                TransportationFringeType::TransitOrVanpool => {
                    input.transit_monthly_cap_dollars.max(0)
                }
                TransportationFringeType::Both => input
                    .parking_monthly_cap_dollars
                    .max(0)
                    .saturating_add(input.transit_monthly_cap_dollars.max(0)),
                TransportationFringeType::NotApplicable => 0,
            };
            value.min(transportation_cap)
        }
        FringeCategory::QualifiedMovingExpenseReimbursement => {
            // OBBBA permanently eliminated except armed forces /
            // intelligence community.
            if input.is_armed_forces_pcs_or_intelligence {
                value
            } else {
                moving_suspended = true;
                0
            }
        }
    };

    let taxable = (value - excludable).max(0);

    let category_label = match input.fringe_category {
        FringeCategory::NoAdditionalCostService => "§132(a)(1) no-additional-cost service",
        FringeCategory::QualifiedEmployeeDiscount => "§132(a)(2) qualified employee discount",
        FringeCategory::WorkingConditionFringe => "§132(a)(3) working condition fringe",
        FringeCategory::DeMinimisFringe => "§132(a)(4) de minimis fringe",
        FringeCategory::QualifiedTransportationFringe => {
            "§132(a)(5)/§132(f) qualified transportation fringe"
        }
        FringeCategory::QualifiedMovingExpenseReimbursement => {
            "§132(a)(6) qualified moving expense reimbursement (post-OBBBA suspended unless armed forces/intelligence)"
        }
        FringeCategory::QualifiedRetirementPlanningServices => {
            "§132(a)(7) qualified retirement planning services"
        }
        FringeCategory::QualifiedMilitaryBaseRealignmentClosure => {
            "§132(a)(8) qualified military base realignment and closure fringe"
        }
    };

    let note = format!(
        "Tax year {}; category: {}; FMV ${}; excludable ${}; taxable ${}{}{}.",
        input.tax_year,
        category_label,
        value,
        excludable,
        taxable,
        if transportation_cap > 0 {
            format!("; §132(f) monthly cap applied: ${}", transportation_cap)
        } else {
            String::new()
        },
        if moving_suspended {
            "; §132(a)(6) moving exclusion PERMANENTLY suspended by OBBBA 2025 (P.L. 119-21) — no exception applies"
        } else {
            ""
        },
    );

    Section132Result {
        fringe_category: input.fringe_category,
        excludable_amount_dollars: excludable,
        taxable_amount_dollars: taxable,
        moving_expense_permanently_suspended: moving_suspended,
        transportation_monthly_cap_applicable_dollars: transportation_cap,
        citation:
            "IRC §132(a) 8 fringe-benefit exclusion categories: (1) no-additional-cost service + (2) qualified employee discount (services 20% cap; goods gross-profit % cap under §132(c)) + (3) working condition fringe (§162/§167 substitute) + (4) de minimis fringe + (5) qualified transportation fringe (§132(f) inflation-adjusted monthly caps: 2024 $315 / 2025 $325 / 2026 $340 each for parking and transit/vanpool separately) + (6) qualified moving expense reimbursement PERMANENTLY SUSPENDED by OBBBA 2025 P.L. 119-21 (TCJA suspension made permanent) EXCEPT armed forces active-duty PCS members and (newly added) intelligence community members + (7) qualified retirement planning services + (8) military base realignment and closure fringe"
                .to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Section132Input {
        Section132Input {
            tax_year: 2026,
            fringe_category: FringeCategory::DeMinimisFringe,
            fringe_value_dollars: 50,
            transportation_fringe_type: TransportationFringeType::NotApplicable,
            parking_monthly_cap_dollars: 340,
            transit_monthly_cap_dollars: 340,
            is_armed_forces_pcs_or_intelligence: false,
            discount_pct_bp: 0,
            discount_is_for_services: false,
            gross_profit_pct_bp: 0,
        }
    }

    // ── Categorical exclusions ──────────────────────────────────────

    #[test]
    fn no_additional_cost_service_fully_excludable() {
        let mut i = base();
        i.fringe_category = FringeCategory::NoAdditionalCostService;
        i.fringe_value_dollars = 500;
        let r = compute(&i);
        assert_eq!(r.excludable_amount_dollars, 500);
        assert_eq!(r.taxable_amount_dollars, 0);
    }

    #[test]
    fn working_condition_fringe_fully_excludable() {
        let mut i = base();
        i.fringe_category = FringeCategory::WorkingConditionFringe;
        i.fringe_value_dollars = 1_000;
        let r = compute(&i);
        assert_eq!(r.excludable_amount_dollars, 1_000);
    }

    #[test]
    fn de_minimis_fringe_fully_excludable() {
        let r = compute(&base());
        assert_eq!(r.excludable_amount_dollars, 50);
        assert_eq!(r.taxable_amount_dollars, 0);
    }

    #[test]
    fn retirement_planning_services_fully_excludable() {
        let mut i = base();
        i.fringe_category = FringeCategory::QualifiedRetirementPlanningServices;
        i.fringe_value_dollars = 200;
        let r = compute(&i);
        assert_eq!(r.excludable_amount_dollars, 200);
    }

    // ── §132(a)(2) qualified employee discount caps ────────────────

    #[test]
    fn services_discount_at_20_pct_fully_excludable() {
        let mut i = base();
        i.fringe_category = FringeCategory::QualifiedEmployeeDiscount;
        i.fringe_value_dollars = 100;
        i.discount_pct_bp = 2000; // 20%
        i.discount_is_for_services = true;
        let r = compute(&i);
        assert_eq!(r.excludable_amount_dollars, 100);
    }

    #[test]
    fn services_discount_above_20_pct_partial_taxable() {
        let mut i = base();
        i.fringe_category = FringeCategory::QualifiedEmployeeDiscount;
        i.fringe_value_dollars = 100;
        i.discount_pct_bp = 4000; // 40%
        i.discount_is_for_services = true;
        let r = compute(&i);
        // Excludable = $100 × (20% / 40%) = $50; taxable $50.
        assert_eq!(r.excludable_amount_dollars, 50);
        assert_eq!(r.taxable_amount_dollars, 50);
    }

    #[test]
    fn goods_discount_at_gross_profit_pct_fully_excludable() {
        let mut i = base();
        i.fringe_category = FringeCategory::QualifiedEmployeeDiscount;
        i.fringe_value_dollars = 100;
        i.discount_pct_bp = 3000; // 30%
        i.discount_is_for_services = false;
        i.gross_profit_pct_bp = 3000; // 30%
        let r = compute(&i);
        assert_eq!(r.excludable_amount_dollars, 100);
    }

    #[test]
    fn goods_discount_above_gross_profit_pct_partial_taxable() {
        let mut i = base();
        i.fringe_category = FringeCategory::QualifiedEmployeeDiscount;
        i.fringe_value_dollars = 100;
        i.discount_pct_bp = 5000; // 50%
        i.discount_is_for_services = false;
        i.gross_profit_pct_bp = 2500; // 25%
        let r = compute(&i);
        // Excludable = $100 × (25% / 50%) = $50.
        assert_eq!(r.excludable_amount_dollars, 50);
        assert_eq!(r.taxable_amount_dollars, 50);
    }

    // ── §132(f) qualified transportation fringe monthly caps ───────

    #[test]
    fn parking_at_2026_cap_340_fully_excludable() {
        let mut i = base();
        i.fringe_category = FringeCategory::QualifiedTransportationFringe;
        i.transportation_fringe_type = TransportationFringeType::Parking;
        i.fringe_value_dollars = 340;
        let r = compute(&i);
        assert_eq!(r.excludable_amount_dollars, 340);
        assert_eq!(r.transportation_monthly_cap_applicable_dollars, 340);
    }

    #[test]
    fn parking_above_2026_cap_partial_taxable() {
        let mut i = base();
        i.fringe_category = FringeCategory::QualifiedTransportationFringe;
        i.transportation_fringe_type = TransportationFringeType::Parking;
        i.fringe_value_dollars = 500;
        let r = compute(&i);
        // Cap $340; excludable $340; taxable $160.
        assert_eq!(r.excludable_amount_dollars, 340);
        assert_eq!(r.taxable_amount_dollars, 160);
    }

    #[test]
    fn transit_at_2026_cap_340_fully_excludable() {
        let mut i = base();
        i.fringe_category = FringeCategory::QualifiedTransportationFringe;
        i.transportation_fringe_type = TransportationFringeType::TransitOrVanpool;
        i.fringe_value_dollars = 340;
        let r = compute(&i);
        assert_eq!(r.excludable_amount_dollars, 340);
    }

    #[test]
    fn both_parking_and_transit_combined_cap_680() {
        // Cap $340 each → $680 combined.
        let mut i = base();
        i.fringe_category = FringeCategory::QualifiedTransportationFringe;
        i.transportation_fringe_type = TransportationFringeType::Both;
        i.fringe_value_dollars = 680;
        let r = compute(&i);
        assert_eq!(r.excludable_amount_dollars, 680);
        assert_eq!(r.transportation_monthly_cap_applicable_dollars, 680);
    }

    #[test]
    fn year_2025_parking_cap_325() {
        let mut i = base();
        i.tax_year = 2025;
        i.parking_monthly_cap_dollars = 325;
        i.transit_monthly_cap_dollars = 325;
        i.fringe_category = FringeCategory::QualifiedTransportationFringe;
        i.transportation_fringe_type = TransportationFringeType::Parking;
        i.fringe_value_dollars = 325;
        let r = compute(&i);
        assert_eq!(r.excludable_amount_dollars, 325);
        assert_eq!(r.transportation_monthly_cap_applicable_dollars, 325);
    }

    // ── §132(a)(6) moving expense — OBBBA permanent suspension ────

    #[test]
    fn moving_expense_civilian_permanently_suspended() {
        let mut i = base();
        i.fringe_category = FringeCategory::QualifiedMovingExpenseReimbursement;
        i.fringe_value_dollars = 5_000;
        i.is_armed_forces_pcs_or_intelligence = false;
        let r = compute(&i);
        assert!(r.moving_expense_permanently_suspended);
        assert_eq!(r.excludable_amount_dollars, 0);
        assert_eq!(r.taxable_amount_dollars, 5_000);
    }

    #[test]
    fn moving_expense_armed_forces_or_intelligence_fully_excludable() {
        let mut i = base();
        i.fringe_category = FringeCategory::QualifiedMovingExpenseReimbursement;
        i.fringe_value_dollars = 5_000;
        i.is_armed_forces_pcs_or_intelligence = true;
        let r = compute(&i);
        assert!(!r.moving_expense_permanently_suspended);
        assert_eq!(r.excludable_amount_dollars, 5_000);
        assert_eq!(r.taxable_amount_dollars, 0);
    }

    // ── Defensive ──────────────────────────────────────────────────

    #[test]
    fn negative_fringe_value_clamped_to_zero() {
        let mut i = base();
        i.fringe_value_dollars = -100;
        let r = compute(&i);
        assert_eq!(r.excludable_amount_dollars, 0);
    }

    // ── Citation ───────────────────────────────────────────────────

    #[test]
    fn citation_mentions_all_8_categories() {
        let r = compute(&base());
        assert!(r.citation.contains("§132(a)"));
        assert!(r.citation.contains("(1) no-additional-cost service"));
        assert!(r.citation.contains("(2) qualified employee discount"));
        assert!(r.citation.contains("(3) working condition fringe"));
        assert!(r.citation.contains("(4) de minimis fringe"));
        assert!(r.citation.contains("(5) qualified transportation fringe"));
        assert!(r
            .citation
            .contains("(6) qualified moving expense reimbursement"));
        assert!(r
            .citation
            .contains("(7) qualified retirement planning services"));
        assert!(r.citation.contains("(8) military base"));
    }

    #[test]
    fn citation_mentions_2026_inflation_amounts() {
        let r = compute(&base());
        assert!(r.citation.contains("2024 $315"));
        assert!(r.citation.contains("2025 $325"));
        assert!(r.citation.contains("2026 $340"));
    }

    #[test]
    fn citation_mentions_obbba_permanent_suspension() {
        let r = compute(&base());
        assert!(r.citation.contains("PERMANENTLY SUSPENDED by OBBBA 2025"));
        assert!(r.citation.contains("armed forces"));
        assert!(r.citation.contains("intelligence community"));
    }

    // ── Notes ──────────────────────────────────────────────────────

    #[test]
    fn note_moving_suspended_describes_permanent_obbba() {
        let mut i = base();
        i.fringe_category = FringeCategory::QualifiedMovingExpenseReimbursement;
        i.fringe_value_dollars = 5_000;
        let r = compute(&i);
        assert!(r.note.contains("PERMANENTLY suspended by OBBBA 2025"));
    }

    #[test]
    fn note_transportation_describes_cap_applied() {
        let mut i = base();
        i.fringe_category = FringeCategory::QualifiedTransportationFringe;
        i.transportation_fringe_type = TransportationFringeType::Parking;
        i.fringe_value_dollars = 500;
        let r = compute(&i);
        assert!(r.note.contains("§132(f) monthly cap applied: $340"));
    }

    // ── Precision ──────────────────────────────────────────────────

    #[test]
    fn very_large_de_minimis_fully_excludable() {
        let mut i = base();
        i.fringe_value_dollars = 1_000_000;
        let r = compute(&i);
        assert_eq!(r.excludable_amount_dollars, 1_000_000);
    }
}
