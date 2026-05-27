//! Equipment depreciation: §179 election + bonus + MACRS straight-line
//! fallback.
//!
//! Trader-as-business buys monitors, NVIDIA cards, faster routers, a new
//! desk. Each is depreciable. Three paths in order of generosity:
//!
//!   1. §179 election — expense up to the annual limit ($1.16M for 2024,
//!      $1.22M for 2025; phase-out above the equipment-purchases ceiling).
//!      Caller's net SE income caps the deduction.
//!   2. Bonus depreciation — 60% of remaining basis in 2024, 40% in 2025,
//!      20% in 2026, 0% in 2027 (TCJA phase-down).
//!   3. Straight-line MACRS over the asset's useful life for whatever
//!      basis is left.
//!
//! Pure compute. The DB caller persists the per-asset election; this
//! module just computes the year-1 deduction.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetClass {
    /// 5-year property: computers, networking, peripherals.
    Computers,
    /// 7-year: office furniture, desks, ergonomic chairs.
    Furniture,
    /// 39-year: real-property improvements (carve-out under §179 if used
    /// for HVAC, roof, fire-protection, security on commercial property).
    NonResidentialRealProperty,
}

impl AssetClass {
    /// MACRS useful life in years.
    pub fn life_years(self) -> u32 {
        match self {
            AssetClass::Computers => 5,
            AssetClass::Furniture => 7,
            AssetClass::NonResidentialRealProperty => 39,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetPurchase {
    pub cost: Decimal,
    pub class: AssetClass,
    pub place_in_service_year: i32,
    /// True if the user wants to elect §179 expensing.
    pub elect_section_179: bool,
    /// True if user wants bonus depreciation on whatever basis remains.
    pub elect_bonus: bool,
    /// Business-use %, 0..=1. <50% disqualifies §179.
    pub business_use_pct: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DepreciationReport {
    /// Year-1 §179 expensed.
    pub section_179_deduction: Decimal,
    /// Year-1 bonus depreciation (after §179).
    pub bonus_deduction: Decimal,
    /// Year-1 MACRS straight-line on remaining basis.
    pub macrs_year_1: Decimal,
    /// Sum of the above. The total year-1 deduction.
    pub year_1_deduction: Decimal,
    /// Basis carried into year 2+ (cost minus all year-1 deductions).
    pub depreciable_basis_remaining: Decimal,
    /// `false` when §179 was requested but business use < 50% or net
    /// income cap below cost. Disclosed in the message.
    pub section_179_allowed: bool,
    pub note: String,
}

/// 2024-2027 published §179 limits + bonus phase-down. Add new rows as
/// IRS publishes; future years extrapolate from the latest row.
fn section_179_limit(year: i32) -> Decimal {
    let d = |s: &str| Decimal::from_str(s).unwrap();
    match year {
        2024 => d("1160000"),
        2025 => d("1220000"),
        2026 => d("1280000"), // estimate; IRS publishes in fall
        _ => d("1280000"),
    }
}

fn bonus_pct(year: i32) -> Decimal {
    let d = |s: &str| Decimal::from_str(s).unwrap();
    match year {
        2024 => d("0.60"),
        2025 => d("0.40"),
        2026 => d("0.20"),
        2027 => Decimal::ZERO,
        _ => Decimal::ZERO,
    }
}

pub fn compute(asset: &AssetPurchase, net_se_income: Decimal) -> DepreciationReport {
    let mut r = DepreciationReport::default();
    // Business-use proportion: only this fraction of the cost is
    // depreciable at all. <50% disqualifies §179 (rev rev §179(b)(4)).
    let half = Decimal::from_str("0.5").unwrap();
    let biz_basis = asset.cost * asset.business_use_pct;
    let mut remaining = biz_basis;

    if asset.elect_section_179 {
        if asset.business_use_pct < half {
            r.section_179_allowed = false;
            r.note = "§179 disallowed: business use < 50%".into();
        } else {
            let yearly_cap = section_179_limit(asset.place_in_service_year);
            let income_cap = net_se_income.max(Decimal::ZERO);
            // §179 is capped at the lesser of: yearly cap, business-use
            // basis, or net SE income. Income > limit doesn't matter,
            // but income < cost forces a partial deduction (+ carryover).
            let cap = remaining.min(yearly_cap).min(income_cap);
            r.section_179_deduction = cap.max(Decimal::ZERO);
            remaining -= r.section_179_deduction;
            r.section_179_allowed = true;
            if cap < remaining + r.section_179_deduction {
                r.note = format!("§179 partial: capped at ${cap}");
            } else {
                r.note = "§179 full expensing".into();
            }
        }
    }

    if asset.elect_bonus && remaining > Decimal::ZERO {
        let pct = bonus_pct(asset.place_in_service_year);
        r.bonus_deduction = remaining * pct;
        remaining -= r.bonus_deduction;
    }

    // Straight-line MACRS on the leftover. Year-1 convention: half-year
    // for most property (so first-year MACRS = 0.5 / life).
    if remaining > Decimal::ZERO {
        let life = Decimal::from(asset.class.life_years());
        r.macrs_year_1 = remaining * half / life;
        remaining -= r.macrs_year_1;
    }

    r.depreciable_basis_remaining = remaining.max(Decimal::ZERO);
    r.year_1_deduction = r.section_179_deduction + r.bonus_deduction + r.macrs_year_1;
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    fn monitor() -> AssetPurchase {
        AssetPurchase {
            cost: d("2000"),
            class: AssetClass::Computers,
            place_in_service_year: 2025,
            elect_section_179: false,
            elect_bonus: false,
            business_use_pct: Decimal::ONE,
        }
    }

    #[test]
    fn life_years_match_macrs_classes() {
        assert_eq!(AssetClass::Computers.life_years(), 5);
        assert_eq!(AssetClass::Furniture.life_years(), 7);
        assert_eq!(AssetClass::NonResidentialRealProperty.life_years(), 39);
    }

    #[test]
    fn straight_line_macrs_half_year_convention() {
        // $2000 / 5y × half-year = $200 in year 1.
        let r = compute(&monitor(), d("100000"));
        assert_eq!(r.macrs_year_1, d("200"));
        assert_eq!(r.year_1_deduction, d("200"));
        assert_eq!(r.depreciable_basis_remaining, d("1800"));
    }

    #[test]
    fn section_179_fully_expenses_within_limits() {
        let mut a = monitor();
        a.elect_section_179 = true;
        let r = compute(&a, d("100000"));
        assert_eq!(r.section_179_deduction, d("2000"));
        assert_eq!(r.year_1_deduction, d("2000"));
        assert_eq!(r.depreciable_basis_remaining, Decimal::ZERO);
        assert!(r.section_179_allowed);
    }

    #[test]
    fn section_179_capped_at_net_se_income() {
        // Asset cost $5k but net SE income only $1k → §179 capped at $1k,
        // remaining $4k gets bonus or MACRS.
        let a = AssetPurchase {
            cost: d("5000"),
            class: AssetClass::Computers,
            place_in_service_year: 2025,
            elect_section_179: true,
            elect_bonus: true,
            business_use_pct: Decimal::ONE,
        };
        let r = compute(&a, d("1000"));
        assert_eq!(r.section_179_deduction, d("1000"));
        // 2025 bonus = 40% of $4000 = $1600.
        assert_eq!(r.bonus_deduction, d("1600"));
    }

    #[test]
    fn section_179_disallowed_below_50_percent_business_use() {
        let mut a = monitor();
        a.elect_section_179 = true;
        a.business_use_pct = d("0.49");
        let r = compute(&a, d("100000"));
        assert!(!r.section_179_allowed);
        assert_eq!(r.section_179_deduction, Decimal::ZERO);
        assert!(r.note.contains("business use < 50%"));
    }

    #[test]
    fn bonus_phase_down_2024_through_2027() {
        for (year, expected) in [(2024, "0.60"), (2025, "0.40"), (2026, "0.20"), (2027, "0")] {
            assert_eq!(
                bonus_pct(year),
                d(expected),
                "bonus pct for {year} must match TCJA phase-down"
            );
        }
    }

    #[test]
    fn bonus_only_runs_against_remaining_basis() {
        // §179 full expense + bonus elected → bonus has nothing to chew on.
        let mut a = monitor();
        a.elect_section_179 = true;
        a.elect_bonus = true;
        let r = compute(&a, d("100000"));
        assert_eq!(
            r.bonus_deduction,
            Decimal::ZERO,
            "§179 fully expensed the asset, bonus has zero basis left"
        );
    }

    #[test]
    fn business_use_below_100_pct_reduces_basis_proportionally() {
        // 60% business → only $1200 of $2000 is depreciable.
        let mut a = monitor();
        a.business_use_pct = d("0.6");
        let r = compute(&a, d("100000"));
        // MACRS = 1200 × 0.5 / 5 = $120.
        assert_eq!(r.macrs_year_1, d("120.0"));
    }

    #[test]
    fn future_year_falls_back_to_zero_bonus_no_panic() {
        let mut a = monitor();
        a.place_in_service_year = 2099;
        a.elect_bonus = true;
        let r = compute(&a, d("100000"));
        assert_eq!(r.bonus_deduction, Decimal::ZERO);
    }
}
