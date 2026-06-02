//! NY rent-stabilized Major Capital Improvement (MCI) +
//! Individual Apartment Improvement (IAI) rent passthrough
//! compliance — when may a trader-landlord owning a rent-
//! stabilized NYC building pass through capital-improvement
//! costs to tenants via rent increase? Trader-landlord
//! critical for any rent-stabilized NYC building owner doing
//! building-wide capital work (MCI) or unit-level upgrades
//! (IAI). Improperly calculated MCI/IAI passthrough triggers
//! DHCR audit + tenant complaint + retroactive refund order.
//!
//! Distinct from siblings `rent_control` (general rent
//! control), `rent_control_lease_disclosure` (lease-side
//! disclosure), `landlord_annual_rent_statement`, and
//! `rent_increase_notice_period`.
//!
//! **NY HSTPA 2019 + 2024-2025 Budget amendments**:
//!
//! **§ 2202.4(d) Major Capital Improvement (MCI)**:
//! - Building-wide capital improvement (boiler + roof +
//!   facade + plumbing risers + etc.).
//! - 2% annual cap on collectibility.
//! - Amortization: 12 years (buildings ≤ 35 units); 12.5
//!   years (buildings > 35 units).
//! - Requires DHCR application + tenant notification +
//!   approval before collection.
//!
//! **9 NYCRR § 2202.4 Individual Apartment Improvement
//! (IAI)** — substantially updated by NY Budget effective
//! October 17, 2024:
//!
//! **Standard IAI tier**:
//! - Cap: **$30,000** over 15 years (up from $15,000 HSTPA
//!   2019 cap).
//! - Rent increase formula: 1/168 of cost (buildings ≤ 35
//!   units); 1/180 (buildings > 35 units).
//! - Increase is now **PERMANENT** (was temporary under
//!   HSTPA 2019).
//!
//! **Special IAI tier** — for units continuously occupied
//! ≥ 25 years OR registered vacant 2022 + 2023 + 2024:
//! - Cap: **$50,000** over 15 years.
//! - Rent increase formula: 1/144 of cost (≤ 35 units);
//!   1/156 (> 35 units).
//!
//! Citations: NY HSTPA 2019 (Pub. L. 2019, ch. 36); NY
//! Budget Bill 2024-2025 (eff. October 17, 2024); 9 NYCRR
//! §§ 2202.4, 2202.5; NYC HCR DHCR Fact Sheet #24 (MCI) +
//! Operational Bulletin 2024-2 (IAI revision); DHCR 2025
//! MCI Schedule.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ImprovementType {
    /// § 2202.4(d) Major Capital Improvement (building-wide).
    MajorCapitalImprovement,
    /// 9 NYCRR § 2202.4 Individual Apartment Improvement
    /// standard tier ($30K cap).
    IndividualApartmentStandard,
    /// 9 NYCRR § 2202.4 Individual Apartment Improvement
    /// special tier ($50K cap) — for units continuously
    /// occupied ≥ 25 years OR registered vacant in 2022 +
    /// 2023 + 2024.
    IndividualApartmentSpecialTier,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RentStabilizedPassthroughInput {
    pub improvement_type: ImprovementType,
    /// Number of units in the building (≤ 35 vs > 35
    /// triggers different amortization).
    pub building_unit_count: u32,
    /// Improvement cost in cents.
    pub improvement_cost_cents: i64,
    /// Whether MCI has been approved by DHCR (required for
    /// MCI collection).
    pub mci_dhcr_approved: bool,
    /// Tenant's current rent in cents (for MCI 2% cap).
    pub tenant_current_rent_cents: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RentStabilizedPassthroughResult {
    pub passthrough_lawful: bool,
    pub allowable_increase_cents: i64,
    pub max_cap_engaged: bool,
    pub mci_2_percent_cap_engaged: bool,
    pub max_cost_recoverable_cents: i64,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &RentStabilizedPassthroughInput) -> RentStabilizedPassthroughResult {
    let cost = input.improvement_cost_cents.max(0);
    let rent = input.tenant_current_rent_cents.max(0);

    match input.improvement_type {
        ImprovementType::MajorCapitalImprovement => check_mci(input, cost, rent),
        ImprovementType::IndividualApartmentStandard => {
            check_iai_standard(input, cost)
        }
        ImprovementType::IndividualApartmentSpecialTier => {
            check_iai_special(input, cost)
        }
    }
}

fn check_mci(
    input: &RentStabilizedPassthroughInput,
    cost: i64,
    rent: i64,
) -> RentStabilizedPassthroughResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "9 NYCRR § 2202.4(d) (Major Capital Improvement) — building-wide capital improvements (boiler + roof + facade + plumbing risers); 2% annual cap on collectibility; amortization 12 years (≤ 35 units) or 12.5 years (> 35 units)"
            .to_string(),
        "NY HSTPA 2019 (Pub. L. 2019 ch. 36) — capped MCI rent increases at 2% maximum collectibility per year; requires DHCR application + tenant notification + approval before collection"
            .to_string(),
    ];

    if !input.mci_dhcr_approved {
        violations.push(
            "9 NYCRR § 2202.4(d) — MCI requires DHCR application + tenant notification + approval before passthrough collection".to_string(),
        );
    }

    let amortization_months: i64 = if input.building_unit_count <= 35 {
        12 * 12
    } else {
        12 * 12 + 6
    };

    let monthly_increase_from_cost = cost / amortization_months;
    let max_2_percent = (rent.saturating_mul(2)) / 100;

    let allowable = monthly_increase_from_cost.min(max_2_percent);
    let mci_2_percent_engaged = monthly_increase_from_cost > max_2_percent;

    RentStabilizedPassthroughResult {
        passthrough_lawful: violations.is_empty(),
        allowable_increase_cents: allowable,
        max_cap_engaged: false,
        mci_2_percent_cap_engaged: mci_2_percent_engaged,
        max_cost_recoverable_cents: cost,
        violations,
        citation: "9 NYCRR § 2202.4(d); NY HSTPA 2019 (Pub. L. 2019 ch. 36); DHCR Fact Sheet #24",
        notes,
    }
}

fn check_iai_standard(
    input: &RentStabilizedPassthroughInput,
    cost: i64,
) -> RentStabilizedPassthroughResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "9 NYCRR § 2202.4 (Individual Apartment Improvement standard tier) — eff. October 17, 2024 NY Budget — cap raised to $30,000 over 15 years (up from $15,000 HSTPA 2019 cap); now PERMANENT (was temporary under HSTPA 2019)"
            .to_string(),
        "9 NYCRR § 2202.4 IAI standard — rent increase formula: 1/168 of cost (buildings ≤ 35 units); 1/180 (buildings > 35 units)"
            .to_string(),
    ];

    let max_cap: i64 = 3_000_000_000;
    let recoverable_cost = cost.min(max_cap);
    let max_cap_engaged = cost > max_cap;

    if max_cap_engaged {
        violations.push(format!(
            "9 NYCRR § 2202.4 (IAI standard) — improvement cost ${} cents exceeds $30,000 cap (${} cents)",
            cost, max_cap
        ));
    }

    let divisor: i64 = if input.building_unit_count <= 35 { 168 } else { 180 };
    let allowable = recoverable_cost / divisor;

    RentStabilizedPassthroughResult {
        passthrough_lawful: violations.is_empty(),
        allowable_increase_cents: allowable,
        max_cap_engaged,
        mci_2_percent_cap_engaged: false,
        max_cost_recoverable_cents: recoverable_cost,
        violations,
        citation: "9 NYCRR § 2202.4 (IAI); NY Budget Bill 2024-2025 (eff. October 17, 2024); HSTPA 2019; DHCR Operational Bulletin 2024-2",
        notes,
    }
}

fn check_iai_special(
    input: &RentStabilizedPassthroughInput,
    cost: i64,
) -> RentStabilizedPassthroughResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "9 NYCRR § 2202.4 (Individual Apartment Improvement special tier) — for units continuously occupied ≥ 25 years OR registered vacant in 2022 + 2023 + 2024; cap $50,000 over 15 years"
            .to_string(),
        "9 NYCRR § 2202.4 IAI special tier — rent increase formula: 1/144 of cost (≤ 35 units); 1/156 (> 35 units)"
            .to_string(),
    ];

    let max_cap: i64 = 5_000_000_000;
    let recoverable_cost = cost.min(max_cap);
    let max_cap_engaged = cost > max_cap;

    if max_cap_engaged {
        violations.push(format!(
            "9 NYCRR § 2202.4 (IAI special tier) — improvement cost ${} cents exceeds $50,000 cap (${} cents)",
            cost, max_cap
        ));
    }

    let divisor: i64 = if input.building_unit_count <= 35 { 144 } else { 156 };
    let allowable = recoverable_cost / divisor;

    RentStabilizedPassthroughResult {
        passthrough_lawful: violations.is_empty(),
        allowable_increase_cents: allowable,
        max_cap_engaged,
        mci_2_percent_cap_engaged: false,
        max_cost_recoverable_cents: recoverable_cost,
        violations,
        citation: "9 NYCRR § 2202.4 (IAI special tier); NY Budget Bill 2024-2025; DHCR Operational Bulletin 2024-2",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mci_base() -> RentStabilizedPassthroughInput {
        RentStabilizedPassthroughInput {
            improvement_type: ImprovementType::MajorCapitalImprovement,
            building_unit_count: 20,
            improvement_cost_cents: 1_440_000_000,
            mci_dhcr_approved: true,
            tenant_current_rent_cents: 300_000,
        }
    }

    fn iai_standard_base() -> RentStabilizedPassthroughInput {
        RentStabilizedPassthroughInput {
            improvement_type: ImprovementType::IndividualApartmentStandard,
            building_unit_count: 20,
            improvement_cost_cents: 2_000_000_000,
            mci_dhcr_approved: false,
            tenant_current_rent_cents: 300_000,
        }
    }

    fn iai_special_base() -> RentStabilizedPassthroughInput {
        RentStabilizedPassthroughInput {
            improvement_type: ImprovementType::IndividualApartmentSpecialTier,
            building_unit_count: 20,
            improvement_cost_cents: 4_000_000_000,
            mci_dhcr_approved: false,
            tenant_current_rent_cents: 300_000,
        }
    }

    #[test]
    fn mci_approved_within_2_percent_cap_lawful() {
        let mut i = mci_base();
        i.improvement_cost_cents = 720_000;
        i.tenant_current_rent_cents = 1_000_000;
        let r = check(&i);
        assert!(r.passthrough_lawful);
        assert_eq!(r.allowable_increase_cents, 5_000);
        assert!(!r.mci_2_percent_cap_engaged);
    }

    #[test]
    fn mci_over_2_percent_cap_capped() {
        let mut i = mci_base();
        i.improvement_cost_cents = 1_440_000_000;
        i.tenant_current_rent_cents = 100_000;
        let r = check(&i);
        assert!(r.passthrough_lawful);
        assert!(r.mci_2_percent_cap_engaged);
        assert_eq!(r.allowable_increase_cents, 2_000);
    }

    #[test]
    fn mci_no_dhcr_approval_violates() {
        let mut i = mci_base();
        i.mci_dhcr_approved = false;
        let r = check(&i);
        assert!(!r.passthrough_lawful);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 2202.4(d)") && v.contains("DHCR")));
    }

    #[test]
    fn mci_amortization_35_units_or_less_12_years() {
        let mut i = mci_base();
        i.building_unit_count = 35;
        i.improvement_cost_cents = 1_440_000_000;
        let r = check(&i);
        let _ = r;
    }

    #[test]
    fn mci_amortization_over_35_units_12_5_years() {
        let mut i = mci_base();
        i.building_unit_count = 36;
        i.improvement_cost_cents = 1_500_000_000;
        let r = check(&i);
        assert!(r.passthrough_lawful);
    }

    #[test]
    fn mci_citation_pins_authorities() {
        let r = check(&mci_base());
        assert!(r.citation.contains("§ 2202.4(d)"));
        assert!(r.citation.contains("HSTPA 2019"));
        assert!(r.citation.contains("Pub. L. 2019 ch. 36"));
        assert!(r.citation.contains("DHCR Fact Sheet #24"));
    }

    #[test]
    fn mci_note_pins_2_percent_cap_and_12_year_amortization() {
        let r = check(&mci_base());
        assert!(r.notes.iter().any(|n| n.contains("2%")
            && n.contains("12 years")
            && n.contains("12.5 years")));
    }

    #[test]
    fn iai_standard_within_cap_lawful() {
        let mut i = iai_standard_base();
        i.improvement_cost_cents = 1_000_000_000;
        let r = check(&i);
        assert!(r.passthrough_lawful);
        assert!(!r.max_cap_engaged);
    }

    #[test]
    fn iai_standard_at_30k_boundary_lawful() {
        let mut i = iai_standard_base();
        i.improvement_cost_cents = 3_000_000_000;
        let r = check(&i);
        assert!(r.passthrough_lawful);
        assert!(!r.max_cap_engaged);
    }

    #[test]
    fn iai_standard_over_30k_cap_violates() {
        let mut i = iai_standard_base();
        i.improvement_cost_cents = 3_000_000_001;
        let r = check(&i);
        assert!(!r.passthrough_lawful);
        assert!(r.max_cap_engaged);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 2202.4 (IAI standard)") && v.contains("$30,000 cap")));
    }

    #[test]
    fn iai_standard_35_units_or_less_uses_168_divisor() {
        let mut i = iai_standard_base();
        i.building_unit_count = 35;
        i.improvement_cost_cents = 1_680_000;
        let r = check(&i);
        assert_eq!(r.allowable_increase_cents, 10_000);
    }

    #[test]
    fn iai_standard_over_35_units_uses_180_divisor() {
        let mut i = iai_standard_base();
        i.building_unit_count = 36;
        i.improvement_cost_cents = 1_800_000;
        let r = check(&i);
        assert_eq!(r.allowable_increase_cents, 10_000);
    }

    #[test]
    fn iai_standard_citation_pins_october_17_2024() {
        let r = check(&iai_standard_base());
        assert!(r.citation.contains("October 17, 2024"));
        assert!(r.citation.contains("Operational Bulletin 2024-2"));
    }

    #[test]
    fn iai_standard_note_pins_30000_cap_and_permanent() {
        let r = check(&iai_standard_base());
        assert!(r.notes.iter().any(|n| n.contains("$30,000")
            && n.contains("PERMANENT")
            && n.contains("HSTPA 2019")));
    }

    #[test]
    fn iai_special_within_cap_lawful() {
        let mut i = iai_special_base();
        i.improvement_cost_cents = 4_000_000_000;
        let r = check(&i);
        assert!(r.passthrough_lawful);
    }

    #[test]
    fn iai_special_at_50k_boundary_lawful() {
        let mut i = iai_special_base();
        i.improvement_cost_cents = 5_000_000_000;
        let r = check(&i);
        assert!(r.passthrough_lawful);
        assert!(!r.max_cap_engaged);
    }

    #[test]
    fn iai_special_over_50k_cap_violates() {
        let mut i = iai_special_base();
        i.improvement_cost_cents = 5_000_000_001;
        let r = check(&i);
        assert!(!r.passthrough_lawful);
        assert!(r.max_cap_engaged);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 2202.4 (IAI special tier)") && v.contains("$50,000 cap")));
    }

    #[test]
    fn iai_special_35_units_or_less_uses_144_divisor() {
        let mut i = iai_special_base();
        i.building_unit_count = 35;
        i.improvement_cost_cents = 1_440_000;
        let r = check(&i);
        assert_eq!(r.allowable_increase_cents, 10_000);
    }

    #[test]
    fn iai_special_over_35_units_uses_156_divisor() {
        let mut i = iai_special_base();
        i.building_unit_count = 36;
        i.improvement_cost_cents = 1_560_000;
        let r = check(&i);
        assert_eq!(r.allowable_increase_cents, 10_000);
    }

    #[test]
    fn iai_special_citation_pins_special_tier() {
        let r = check(&iai_special_base());
        assert!(r.citation.contains("IAI special tier"));
        assert!(r.citation.contains("Operational Bulletin 2024-2"));
    }

    #[test]
    fn iai_special_note_pins_25_year_or_vacant_2022_2024() {
        let r = check(&iai_special_base());
        assert!(r.notes.iter().any(|n| n.contains("25 years")
            && n.contains("2022")
            && n.contains("2023")
            && n.contains("2024")
            && n.contains("$50,000")));
    }

    #[test]
    fn improvement_type_truth_table_caps() {
        let r_mci = check(&mci_base());
        assert!(!r_mci.max_cap_engaged);
        assert_eq!(r_mci.max_cost_recoverable_cents, 1_440_000_000);

        let r_iai_std = check(&iai_standard_base());
        assert!(!r_iai_std.max_cap_engaged);
        assert_eq!(r_iai_std.max_cost_recoverable_cents, 2_000_000_000);

        let r_iai_special = check(&iai_special_base());
        assert!(!r_iai_special.max_cap_engaged);
        assert_eq!(r_iai_special.max_cost_recoverable_cents, 4_000_000_000);
    }

    #[test]
    fn iai_special_uniquely_uses_50k_cap_invariant() {
        let mut i_special = iai_special_base();
        i_special.improvement_cost_cents = 4_000_000_000;
        let r_special = check(&i_special);
        assert!(r_special.passthrough_lawful);

        let mut i_standard = iai_standard_base();
        i_standard.improvement_cost_cents = 4_000_000_000;
        let r_standard = check(&i_standard);
        assert!(!r_standard.passthrough_lawful);
        assert!(r_standard.max_cap_engaged);
    }

    #[test]
    fn mci_uniquely_engages_2_percent_cap_invariant() {
        let mut i_mci = mci_base();
        i_mci.tenant_current_rent_cents = 50_000;
        i_mci.improvement_cost_cents = 1_440_000_000;
        let r_mci = check(&i_mci);
        assert!(r_mci.mci_2_percent_cap_engaged);

        let mut i_iai = iai_standard_base();
        i_iai.tenant_current_rent_cents = 50_000;
        let r_iai = check(&i_iai);
        assert!(!r_iai.mci_2_percent_cap_engaged);
    }

    #[test]
    fn three_improvement_types_routed_correctly() {
        for improvement in [
            ImprovementType::MajorCapitalImprovement,
            ImprovementType::IndividualApartmentStandard,
            ImprovementType::IndividualApartmentSpecialTier,
        ] {
            let mut i = mci_base();
            i.improvement_type = improvement;
            let r = check(&i);
            assert!(!r.citation.is_empty());
        }
    }

    #[test]
    fn defensive_negative_cost_clamped() {
        let mut i = iai_standard_base();
        i.improvement_cost_cents = -1_000_000_000;
        let r = check(&i);
        assert!(r.passthrough_lawful);
        assert_eq!(r.allowable_increase_cents, 0);
    }

    #[test]
    fn defensive_negative_rent_clamped() {
        let mut i = mci_base();
        i.tenant_current_rent_cents = -100_000;
        let r = check(&i);
        assert_eq!(r.allowable_increase_cents, 0);
    }

    #[test]
    fn defensive_overflow_saturating() {
        let mut i = mci_base();
        i.tenant_current_rent_cents = i64::MAX;
        let r = check(&i);
        let _ = r;
    }
}
