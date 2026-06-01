//! IRC § 25C — Energy Efficient Home Improvement Credit.
//!
//! § 25C provides a **30% nonrefundable credit** for qualifying energy-
//! efficiency improvements to a taxpayer's residence (primary or
//! secondary). IRA 2022 redesigned the credit with a layered cap
//! structure totaling **up to $3,200/year**. OBBBA § 70425 ACCELERATED
//! the termination to property **placed in service after December 31,
//! 2025** — wiping out the IRA's 2032 sunset by 7 years.
//!
//! Cap structure (after IRA 2022 redesign, pre-termination):
//!
//! General $1,200 envelope (§ 25C(b)(1)): windows/skylights, doors,
//! insulation, non-heat-pump energy property, and home-energy audits
//! collectively capped at $1,200 per year. Within that envelope:
//!
//! windows + skylights aggregate cap $600 (§ 25C(b)(2)(A)); exterior
//! doors $250 per door, $500 aggregate cap (§ 25C(b)(2)(B)); any
//! single qualified energy property item $600 (§ 25C(b)(2)(C)); home
//! energy audits $150 (§ 25C(b)(2)(D)); insulation has no sub-cap and
//! is bounded only by the general $1,200.
//!
//! Heat-pump SEPARATE $2,000 cap (§ 25C(b)(3)): heat pumps + heat-pump
//! water heaters + biomass stoves/boilers get their own $2,000 ceiling
//! ABOVE and BEYOND the general $1,200.
//!
//! Total annual maximum = $1,200 general + $2,000 heat pump = $3,200.
//!
//! 30% credit rate (§ 25C(a)) applies to all qualifying expenditures.
//! Credit is NONREFUNDABLE (no carryforward — distinct from § 25D which
//! has indefinite carryforward).
//!
//! Citations: 26 U.S.C. § 25C (general); § 25C(a) (30% credit rate);
//! § 25C(b)(1) ($1,200 general annual cap); § 25C(b)(2)(A) ($600
//! windows+skylights); § 25C(b)(2)(B) ($250 per door / $500 aggregate);
//! § 25C(b)(2)(C) ($600 per energy property item); § 25C(b)(2)(D) ($150
//! home energy audit); § 25C(b)(3) ($2,000 separate heat-pump cap);
//! § 25C(h) (sunset accelerated by OBBBA § 70425); IRS § 25C OBBBA FAQ.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Section25CInput {
    pub placed_in_service_year: u32,
    pub placed_in_service_month: u32,
    pub placed_in_service_day: u32,
    pub windows_skylights_cost_cents: i64,
    pub doors_cost_cents: i64,
    pub door_count: u32,
    pub insulation_cost_cents: i64,
    pub energy_property_cost_cents: i64,
    /// Number of distinct qualified energy property items (each capped at
    /// $600 under § 25C(b)(2)(C)). 0 means no per-item cap test applies.
    pub energy_property_item_count: u32,
    pub heat_pump_cost_cents: i64,
    pub home_energy_audit_cost_cents: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section25CResult {
    pub placed_in_service_after_obbba_termination: bool,
    pub windows_credit_cents: i64,
    pub doors_credit_cents: i64,
    pub insulation_credit_cents: i64,
    pub energy_property_credit_cents: i64,
    pub audit_credit_cents: i64,
    pub heat_pump_credit_cents: i64,
    pub general_envelope_subtotal_cents: i64,
    pub general_envelope_cap_cents: i64,
    pub heat_pump_subtotal_cents: i64,
    pub heat_pump_cap_cents: i64,
    pub total_credit_cents: i64,
    pub citation: &'static str,
    pub note: String,
}

pub fn compute(input: &Section25CInput) -> Section25CResult {
    let windows = input.windows_skylights_cost_cents.max(0);
    let doors = input.doors_cost_cents.max(0);
    let insulation = input.insulation_cost_cents.max(0);
    let energy_property = input.energy_property_cost_cents.max(0);
    let audit = input.home_energy_audit_cost_cents.max(0);
    let heat_pump = input.heat_pump_cost_cents.max(0);

    let after_termination = is_after_2025_12_31(
        input.placed_in_service_year,
        input.placed_in_service_month,
        input.placed_in_service_day,
    );
    if after_termination {
        return Section25CResult {
            placed_in_service_after_obbba_termination: true,
            windows_credit_cents: 0,
            doors_credit_cents: 0,
            insulation_credit_cents: 0,
            energy_property_credit_cents: 0,
            audit_credit_cents: 0,
            heat_pump_credit_cents: 0,
            general_envelope_subtotal_cents: 0,
            general_envelope_cap_cents: 120000,
            heat_pump_subtotal_cents: 0,
            heat_pump_cap_cents: 200000,
            total_credit_cents: 0,
            citation:
                "26 U.S.C. § 25C + OBBBA § 70425 — credit TERMINATED for property placed in service after 2025-12-31",
            note: format!(
                "Property placed in service {}-{:02}-{:02} after OBBBA § 70425 termination date 2025-12-31. No § 25C credit available.",
                input.placed_in_service_year, input.placed_in_service_month, input.placed_in_service_day
            ),
        };
    }

    // 30% credit rate (§ 25C(a)) applied across all categories, then
    // category sub-caps applied.

    // Windows + skylights aggregate cap $600.
    let windows_30 = (windows as i128 * 30 / 100) as i64;
    let windows_credit = windows_30.min(60000);

    // Doors: $250 per door, $500 aggregate.
    let doors_30 = (doors as i128 * 30 / 100) as i64;
    let per_door_aggregate_cap = (input.door_count as i64).saturating_mul(25000);
    let doors_credit = doors_30.min(per_door_aggregate_cap).min(50000);

    // Insulation: no sub-cap, only bounded by general $1,200.
    let insulation_credit = (insulation as i128 * 30 / 100) as i64;

    // Energy property: $600 per item. If item count is 0 model as a
    // single item bound for sanity (caller may aggregate items).
    let energy_property_30 = (energy_property as i128 * 30 / 100) as i64;
    let per_item_aggregate_cap = if input.energy_property_item_count == 0 {
        60000
    } else {
        (input.energy_property_item_count as i64).saturating_mul(60000)
    };
    let energy_property_credit = energy_property_30.min(per_item_aggregate_cap);

    // Home energy audit: $150.
    let audit_30 = (audit as i128 * 30 / 100) as i64;
    let audit_credit = audit_30.min(15000);

    // Aggregate general envelope and apply $1,200 cap.
    let general_raw =
        windows_credit + doors_credit + insulation_credit + energy_property_credit + audit_credit;
    let general_subtotal = general_raw.min(120000);

    // Heat-pump separate cap $2,000.
    let heat_pump_30 = (heat_pump as i128 * 30 / 100) as i64;
    let heat_pump_subtotal = heat_pump_30.min(200000);

    let total = general_subtotal + heat_pump_subtotal;

    let note = format!(
        "30% credit rate. Windows/skylights credit = min(30% × {} = {}, $600 cap) = {}. Doors credit = min(30% × {} = {}, $250 × {} doors = {}, $500 aggregate cap) = {}. Insulation credit = 30% × {} = {}. Energy property credit = min(30% × {} = {}, $600 × {} items = {}) = {}. Audit credit = min(30% × {} = {}, $150 cap) = {}. General-envelope sum = {} cents capped at $1,200 = {} cents. Heat-pump credit = min(30% × {} = {}, $2,000 cap) = {} cents. Total = {} cents.",
        windows,
        windows_30,
        windows_credit,
        doors,
        doors_30,
        input.door_count,
        per_door_aggregate_cap,
        doors_credit,
        insulation,
        insulation_credit,
        energy_property,
        energy_property_30,
        input.energy_property_item_count,
        per_item_aggregate_cap,
        energy_property_credit,
        audit,
        audit_30,
        audit_credit,
        general_raw,
        general_subtotal,
        heat_pump,
        heat_pump_30,
        heat_pump_subtotal,
        total,
    );

    Section25CResult {
        placed_in_service_after_obbba_termination: false,
        windows_credit_cents: windows_credit,
        doors_credit_cents: doors_credit,
        insulation_credit_cents: insulation_credit,
        energy_property_credit_cents: energy_property_credit,
        audit_credit_cents: audit_credit,
        heat_pump_credit_cents: heat_pump_subtotal,
        general_envelope_subtotal_cents: general_subtotal,
        general_envelope_cap_cents: 120000,
        heat_pump_subtotal_cents: heat_pump_subtotal,
        heat_pump_cap_cents: 200000,
        total_credit_cents: total,
        citation:
            "26 U.S.C. § 25C(a)/(b)(1)/(b)(2)/(b)(3) — 30% credit with $1,200 general + $2,000 heat-pump caps + sub-caps; OBBBA § 70425 terminates 2025-12-31",
        note,
    }
}

fn is_after_2025_12_31(year: u32, month: u32, day: u32) -> bool {
    match year.cmp(&2025) {
        std::cmp::Ordering::Greater => true,
        std::cmp::Ordering::Less => false,
        std::cmp::Ordering::Equal => match month.cmp(&12) {
            std::cmp::Ordering::Greater => true,
            std::cmp::Ordering::Less => false,
            std::cmp::Ordering::Equal => day > 31,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        year: u32,
        month: u32,
        day: u32,
        windows: i64,
        doors: i64,
        door_count: u32,
        insulation: i64,
        energy: i64,
        energy_items: u32,
        heat_pump: i64,
        audit: i64,
    ) -> Section25CInput {
        Section25CInput {
            placed_in_service_year: year,
            placed_in_service_month: month,
            placed_in_service_day: day,
            windows_skylights_cost_cents: windows,
            doors_cost_cents: doors,
            door_count,
            insulation_cost_cents: insulation,
            energy_property_cost_cents: energy,
            energy_property_item_count: energy_items,
            heat_pump_cost_cents: heat_pump,
            home_energy_audit_cost_cents: audit,
        }
    }

    #[test]
    fn full_3200_credit_maxes_both_envelopes() {
        // Big spend: $5K windows + $3K doors (2) + $5K insulation +
        // $5K energy property (2 items) + $500 audit + $10K heat pump.
        // Windows: 30% × $5K = $1,500 → capped $600. Doors: 30% × $3K =
        // $900, $250×2 = $500, $500 aggregate → $500. Insulation: 30% ×
        // $5K = $1,500. Energy property: 30% × $5K = $1,500, $600×2 =
        // $1,200 → $1,200. Audit: 30% × $500 = $150. Sum = $3,950 →
        // capped at $1,200. Heat pump: 30% × $10K = $3,000 → capped $2K.
        // Total = $3,200.
        let r = compute(&input(
            2024, 6, 1, 5_000_00, 3_000_00, 2, 5_000_00, 5_000_00, 2, 10_000_00, 500_00,
        ));
        assert_eq!(r.total_credit_cents, 3_200_00);
        assert_eq!(r.general_envelope_subtotal_cents, 1_200_00);
        assert_eq!(r.heat_pump_subtotal_cents, 2_000_00);
    }

    #[test]
    fn obbba_termination_after_2025_12_31() {
        let r = compute(&input(
            2026, 1, 1, 5_000_00, 3_000_00, 2, 5_000_00, 5_000_00, 2, 10_000_00, 500_00,
        ));
        assert!(r.placed_in_service_after_obbba_termination);
        assert_eq!(r.total_credit_cents, 0);
        assert!(r.citation.contains("OBBBA § 70425"));
        assert!(r.citation.contains("TERMINATED"));
    }

    #[test]
    fn at_2025_12_31_boundary_still_eligible() {
        let r = compute(&input(
            2025, 12, 31, 5_000_00, 3_000_00, 2, 5_000_00, 5_000_00, 2, 10_000_00, 500_00,
        ));
        assert!(!r.placed_in_service_after_obbba_termination);
        assert_eq!(r.total_credit_cents, 3_200_00);
    }

    #[test]
    fn windows_capped_at_600() {
        // $5K windows → 30% = $1,500, capped at $600.
        let r = compute(&input(2024, 6, 1, 5_000_00, 0, 0, 0, 0, 0, 0, 0));
        assert_eq!(r.windows_credit_cents, 60000);
    }

    #[test]
    fn windows_under_600_not_capped() {
        // $1,000 windows → 30% = $300, under cap.
        let r = compute(&input(2024, 6, 1, 1_000_00, 0, 0, 0, 0, 0, 0, 0));
        assert_eq!(r.windows_credit_cents, 30000);
    }

    #[test]
    fn doors_250_per_door_one_door() {
        // $1,500 cost × 30% = $450. $250 per door × 1 door = $250.
        let r = compute(&input(2024, 6, 1, 0, 1_500_00, 1, 0, 0, 0, 0, 0));
        assert_eq!(r.doors_credit_cents, 25000);
    }

    #[test]
    fn doors_500_aggregate_cap_three_doors() {
        // $3K cost × 30% = $900. $250 × 3 = $750. $500 aggregate cap → $500.
        let r = compute(&input(2024, 6, 1, 0, 3_000_00, 3, 0, 0, 0, 0, 0));
        assert_eq!(r.doors_credit_cents, 50000);
    }

    #[test]
    fn doors_aggregate_cap_binds_even_with_more_doors() {
        // 10 doors × $250 = $2,500 — aggregate cap $500 still binds.
        let r = compute(&input(2024, 6, 1, 0, 10_000_00, 10, 0, 0, 0, 0, 0));
        assert_eq!(r.doors_credit_cents, 50000);
    }

    #[test]
    fn energy_property_600_per_item() {
        // $5K cost × 30% = $1,500. 1 item × $600 = $600 cap binds.
        let r = compute(&input(2024, 6, 1, 0, 0, 0, 0, 5_000_00, 1, 0, 0));
        assert_eq!(r.energy_property_credit_cents, 60000);
    }

    #[test]
    fn energy_property_multiple_items_aggregate() {
        // $5K cost × 30% = $1,500. 3 items × $600 = $1,800 cap. Subject
        // also to general $1,200 envelope.
        let r = compute(&input(2024, 6, 1, 0, 0, 0, 0, 5_000_00, 3, 0, 0));
        assert_eq!(r.energy_property_credit_cents, 1_500_00);
        // General envelope = min($1,500, $1,200) = $1,200.
        assert_eq!(r.general_envelope_subtotal_cents, 1_200_00);
    }

    #[test]
    fn audit_capped_at_150() {
        // $1,000 audit × 30% = $300, capped at $150.
        let r = compute(&input(2024, 6, 1, 0, 0, 0, 0, 0, 0, 0, 1_000_00));
        assert_eq!(r.audit_credit_cents, 15000);
    }

    #[test]
    fn audit_under_cap_not_capped() {
        // $300 audit × 30% = $90, under cap.
        let r = compute(&input(2024, 6, 1, 0, 0, 0, 0, 0, 0, 0, 300_00));
        assert_eq!(r.audit_credit_cents, 9000);
    }

    #[test]
    fn heat_pump_capped_at_2000() {
        // $10K heat pump × 30% = $3,000, capped at $2,000.
        let r = compute(&input(2024, 6, 1, 0, 0, 0, 0, 0, 0, 10_000_00, 0));
        assert_eq!(r.heat_pump_credit_cents, 2_000_00);
    }

    #[test]
    fn heat_pump_separate_from_general_envelope() {
        // Heat pump $2,000 + general $1,200 = $3,200 max — heat pump is
        // ABOVE and BEYOND the $1,200 general cap.
        let r = compute(&input(2024, 6, 1, 5_000_00, 0, 0, 0, 0, 0, 10_000_00, 0));
        // Windows $600 + heat pump $2,000 = $2,600. General + heat pump.
        assert_eq!(r.general_envelope_subtotal_cents, 60000);
        assert_eq!(r.heat_pump_subtotal_cents, 200000);
        assert_eq!(r.total_credit_cents, 2_600_00);
    }

    #[test]
    fn insulation_no_sub_cap() {
        // $2K insulation × 30% = $600 — no sub-cap, bounded by general $1,200.
        let r = compute(&input(2024, 6, 1, 0, 0, 0, 2_000_00, 0, 0, 0, 0));
        assert_eq!(r.insulation_credit_cents, 60000);
    }

    #[test]
    fn insulation_bounded_only_by_general_1200() {
        // $10K insulation × 30% = $3,000 — capped only by general $1,200.
        let r = compute(&input(2024, 6, 1, 0, 0, 0, 10_000_00, 0, 0, 0, 0));
        assert_eq!(r.insulation_credit_cents, 3_000_00);
        // But general envelope binds at $1,200.
        assert_eq!(r.general_envelope_subtotal_cents, 1_200_00);
    }

    #[test]
    fn zero_cost_zero_credit() {
        let r = compute(&input(2024, 6, 1, 0, 0, 0, 0, 0, 0, 0, 0));
        assert_eq!(r.total_credit_cents, 0);
    }

    #[test]
    fn negative_inputs_clamped() {
        let r = compute(&input(2024, 6, 1, -1, -1, 0, -1, -1, 0, -1, -1));
        assert_eq!(r.total_credit_cents, 0);
    }

    #[test]
    fn citation_pins_correct_authorities() {
        let r = compute(&input(2024, 6, 1, 1_000_00, 0, 0, 0, 0, 0, 0, 0));
        assert!(r.citation.contains("§ 25C(a)"));
        assert!(r.citation.contains("(b)(1)"));
        assert!(r.citation.contains("(b)(3)"));
        assert!(r.citation.contains("OBBBA § 70425"));

        let post = compute(&input(2026, 1, 1, 1_000_00, 0, 0, 0, 0, 0, 0, 0));
        assert!(post.citation.contains("OBBBA § 70425"));
        assert!(post.citation.contains("TERMINATED"));
    }

    #[test]
    fn date_boundary_dec_30_31_jan_1() {
        let d30 = compute(&input(2025, 12, 30, 1_000_00, 0, 0, 0, 0, 0, 0, 0));
        let d31 = compute(&input(2025, 12, 31, 1_000_00, 0, 0, 0, 0, 0, 0, 0));
        let jan1 = compute(&input(2026, 1, 1, 1_000_00, 0, 0, 0, 0, 0, 0, 0));
        assert_eq!(d30.total_credit_cents, 30000);
        assert_eq!(d31.total_credit_cents, 30000);
        assert_eq!(jan1.total_credit_cents, 0);
    }

    #[test]
    fn general_envelope_caps_at_1200() {
        // $5K windows + $3K doors (2) + $5K insulation + $5K energy
        // (no items so single $600 cap) + $500 audit = lots, capped $1,200.
        let r = compute(&input(
            2024, 6, 1, 5_000_00, 3_000_00, 2, 5_000_00, 5_000_00, 0, 0, 500_00,
        ));
        assert_eq!(r.general_envelope_subtotal_cents, 1_200_00);
    }

    #[test]
    fn doors_per_door_cap_below_aggregate() {
        // 1 door at $400 cost = 30% × $400 = $120. Per-door cap $250 →
        // $120 binds (under per-door cap), and under $500 aggregate.
        let r = compute(&input(2024, 6, 1, 0, 400_00, 1, 0, 0, 0, 0, 0));
        assert_eq!(r.doors_credit_cents, 12000);
    }

    #[test]
    fn doors_per_door_cap_binds_high_cost_single_door() {
        // 1 door at $2K cost × 30% = $600. Per-door cap $250 → $250 binds.
        let r = compute(&input(2024, 6, 1, 0, 2_000_00, 1, 0, 0, 0, 0, 0));
        assert_eq!(r.doors_credit_cents, 25000);
    }

    #[test]
    fn windows_at_600_boundary_exactly() {
        // $2,000 windows × 30% = $600 exactly at cap.
        let r = compute(&input(2024, 6, 1, 2_000_00, 0, 0, 0, 0, 0, 0, 0));
        assert_eq!(r.windows_credit_cents, 60000);
    }

    #[test]
    fn heat_pump_at_2000_boundary_exactly() {
        // $6,666.67 heat pump × 30% = $2,000 exactly. Use integer-safe value.
        let r = compute(&input(2024, 6, 1, 0, 0, 0, 0, 0, 0, 6_667_00, 0));
        assert_eq!(r.heat_pump_credit_cents, 2_000_00);
    }

    #[test]
    fn total_caps_at_3200_in_high_spend_scenario() {
        // $50K windows + $50K doors (10) + $50K insulation + $50K
        // energy (10 items) + $5K audit + $50K heat pump.
        let r = compute(&input(
            2024, 6, 1, 50_000_00, 50_000_00, 10, 50_000_00, 50_000_00, 10, 50_000_00, 5_000_00,
        ));
        assert_eq!(r.total_credit_cents, 3_200_00);
    }
}
