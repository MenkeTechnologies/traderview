//! Cost-segregation study + §168(k) bonus depreciation phase-down.
//!
//! A landlord who buys a $500,000 rental and depreciates it as a single
//! 27.5-year building gets ~$18,000/year of MACRS deduction. A
//! cost-segregation study breaks the purchase into FIVE asset-life
//! buckets, each with its own MACRS class:
//!
//!   * **5-year property**: carpet, decorative lighting, vinyl flooring,
//!     kitchen appliances, removable partitions, draperies, A/V wiring.
//!   * **7-year property**: office furniture and fixtures (rare for
//!     residential; more common in commercial).
//!   * **15-year property** (Qualified Improvement Property / land
//!     improvements): parking lots, sidewalks, fencing, landscaping,
//!     exterior lighting, signs.
//!   * **27.5-year property**: residential building shell (walls, roof,
//!     structural).
//!   * **39-year property**: non-residential building shell.
//!
//! Layered with **§168(k) bonus depreciation**, the 5/7/15-year buckets
//! get an additional Y%-of-basis first-year deduction (Y phase-down per
//! TCJA: 80% in 2023, 60% in 2024, 40% in 2025, 20% in 2026, 0% in 2027+).
//!
//! Combined with §280A short-term-rental classification + material
//! participation (see `section_280a` + `section_469`), the resulting
//! year-1 loss is **non-passive** and can absorb W-2 ordinary income —
//! the "STR loophole" that turns a $500k vacation rental into a
//! $100k+ first-year tax shield.
//!
//! Pure compute. Caller chooses the property-type defaults (or overrides
//! the bucket percentages directly) and the bonus-depreciation year.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Typical industry cost-seg-study breakdown by property type. Used as
/// defaults; caller can override `CostSegAllocation` directly for a
/// study-specific result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyTypeDefault {
    /// Single-family residence — 27.5y shell, light cost seg.
    SingleFamily,
    /// Multi-family apartment — more 15-year (parking, landscaping).
    MultiFamily,
    /// Short-term rental (vacation home, fully furnished) — heavy 5-year
    /// (furniture, appliances) — this is the trader-W-2-offset setup.
    ShortTermRental,
    /// Commercial / office — 39y shell instead of 27.5y.
    Commercial,
    /// Restaurant / hospitality — heavy 5y (kitchen equipment).
    Restaurant,
}

/// Per-bucket percentage allocation. Must sum to 1.0.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostSegAllocation {
    pub pct_5_year: Decimal,
    pub pct_7_year: Decimal,
    pub pct_15_year: Decimal,
    pub pct_27_5_year: Decimal,
    pub pct_39_year: Decimal,
}

impl CostSegAllocation {
    pub fn sum(&self) -> Decimal {
        self.pct_5_year
            + self.pct_7_year
            + self.pct_15_year
            + self.pct_27_5_year
            + self.pct_39_year
    }

    pub fn for_type(t: PropertyTypeDefault) -> Self {
        let d = |s: &str| Decimal::from_str(s).unwrap();
        match t {
            PropertyTypeDefault::SingleFamily => Self {
                pct_5_year: d("0.05"),
                pct_7_year: Decimal::ZERO,
                pct_15_year: d("0.10"),
                pct_27_5_year: d("0.85"),
                pct_39_year: Decimal::ZERO,
            },
            PropertyTypeDefault::MultiFamily => Self {
                pct_5_year: d("0.10"),
                pct_7_year: d("0.05"),
                pct_15_year: d("0.15"),
                pct_27_5_year: d("0.70"),
                pct_39_year: Decimal::ZERO,
            },
            PropertyTypeDefault::ShortTermRental => Self {
                pct_5_year: d("0.25"),
                pct_7_year: d("0.10"),
                pct_15_year: d("0.10"),
                pct_27_5_year: d("0.55"),
                pct_39_year: Decimal::ZERO,
            },
            PropertyTypeDefault::Commercial => Self {
                pct_5_year: d("0.05"),
                pct_7_year: d("0.05"),
                pct_15_year: d("0.15"),
                pct_27_5_year: Decimal::ZERO,
                pct_39_year: d("0.75"),
            },
            PropertyTypeDefault::Restaurant => Self {
                pct_5_year: d("0.30"),
                pct_7_year: Decimal::ZERO,
                pct_15_year: d("0.15"),
                pct_27_5_year: Decimal::ZERO,
                pct_39_year: d("0.55"),
            },
        }
    }
}

/// TCJA §168(k) bonus-depreciation phase-down.
pub fn bonus_pct_for_year(year: i32) -> Decimal {
    let d = |s: &str| Decimal::from_str(s).unwrap();
    match year {
        ..=2017 => d("0.50"),
        2018..=2022 => d("1.00"),
        2023 => d("0.80"),
        2024 => d("0.60"),
        2025 => d("0.40"),
        2026 => d("0.20"),
        _ => Decimal::ZERO, // 2027+
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostSegInput {
    /// Depreciable basis (cost - land value - prior accumulated dep).
    pub depreciable_basis: Decimal,
    pub property_type: PropertyTypeDefault,
    /// If provided, overrides the property-type default allocation.
    /// Must sum to 1.0 within rounding tolerance (we accept ±0.005).
    pub allocation_override: Option<CostSegAllocation>,
    /// Tax year for which year-1 depreciation is computed (drives the
    /// §168(k) bonus-pct lookup).
    pub tax_year: i32,
    /// True to elect §168(k) bonus depreciation on the 5/7/15-year
    /// buckets. The 27.5/39-year real-property buckets are NEVER
    /// eligible for §168(k) (excluded by §168(k)(2)(A)(i)).
    pub elect_bonus_depreciation: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CostSegBucket {
    pub basis_allocated: Decimal,
    pub bonus_pct_applied: Decimal,
    pub bonus_deduction: Decimal,
    pub macrs_year_1_on_remaining: Decimal,
    pub year_1_total: Decimal,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CostSegReport {
    pub allocation_used: CostSegAllocation,
    pub bonus_pct_applied: Decimal,
    pub bucket_5_year: CostSegBucket,
    pub bucket_7_year: CostSegBucket,
    pub bucket_15_year: CostSegBucket,
    pub bucket_27_5_year: CostSegBucket,
    pub bucket_39_year: CostSegBucket,
    pub year_1_total_deduction: Decimal,
    /// What the deduction would have been without cost seg (straight 27.5y
    /// or 39y building, half-month convention assumed for first year).
    pub year_1_without_cost_seg: Decimal,
    pub year_1_acceleration: Decimal,
    pub note: String,
}

impl Default for CostSegAllocation {
    fn default() -> Self {
        Self {
            pct_5_year: Decimal::ZERO,
            pct_7_year: Decimal::ZERO,
            pct_15_year: Decimal::ZERO,
            pct_27_5_year: Decimal::ZERO,
            pct_39_year: Decimal::ZERO,
        }
    }
}

/// MACRS half-year convention first-year deduction = (1 / life) × 0.5.
/// Used inside each bucket's `macrs_year_1_on_remaining` after bonus
/// depreciation peels off its share. Real property (27.5 / 39) uses
/// the mid-month convention via the `rental_depreciation` module; here
/// we use a flat half-year approximation for the no-cost-seg baseline
/// to keep things consistent across the 5/7/15-year personal-property
/// buckets and easy to reason about.
fn macrs_half_year_year_1(life: Decimal) -> Decimal {
    if life <= Decimal::ZERO {
        return Decimal::ZERO;
    }
    Decimal::from_str("0.5").unwrap() / life
}

fn compute_bucket(basis: Decimal, life: Decimal, bonus_pct: Decimal) -> CostSegBucket {
    if basis <= Decimal::ZERO {
        return CostSegBucket::default();
    }
    let bonus = (basis * bonus_pct).round_dp(2);
    let remaining = basis - bonus;
    let macrs_rate = macrs_half_year_year_1(life);
    let macrs = (remaining * macrs_rate).round_dp(2);
    CostSegBucket {
        basis_allocated: basis,
        bonus_pct_applied: bonus_pct,
        bonus_deduction: bonus,
        macrs_year_1_on_remaining: macrs,
        year_1_total: bonus + macrs,
    }
}

pub fn compute(input: &CostSegInput) -> CostSegReport {
    let mut r = CostSegReport::default();

    if input.depreciable_basis <= Decimal::ZERO {
        r.note = "no depreciable basis".into();
        return r;
    }

    let allocation = match &input.allocation_override {
        Some(o) => {
            let sum = o.sum();
            let one = Decimal::from(1);
            let tol = Decimal::from_str("0.005").unwrap();
            if (sum - one).abs() > tol {
                r.note = format!("allocation_override sum {sum} != 1.0; using property-type default");
                CostSegAllocation::for_type(input.property_type)
            } else {
                o.clone()
            }
        }
        None => CostSegAllocation::for_type(input.property_type),
    };
    r.allocation_used = allocation.clone();

    // Bonus only applies to 5/7/15-year buckets, and only if elected.
    let bonus_pct = if input.elect_bonus_depreciation {
        bonus_pct_for_year(input.tax_year)
    } else {
        Decimal::ZERO
    };
    r.bonus_pct_applied = bonus_pct;

    let b5 = (input.depreciable_basis * allocation.pct_5_year).round_dp(2);
    let b7 = (input.depreciable_basis * allocation.pct_7_year).round_dp(2);
    let b15 = (input.depreciable_basis * allocation.pct_15_year).round_dp(2);
    let b27_5 = (input.depreciable_basis * allocation.pct_27_5_year).round_dp(2);
    let b39 = (input.depreciable_basis * allocation.pct_39_year).round_dp(2);

    let life_5 = Decimal::from(5);
    let life_7 = Decimal::from(7);
    let life_15 = Decimal::from(15);
    let life_27_5 = Decimal::from_str("27.5").unwrap();
    let life_39 = Decimal::from(39);

    r.bucket_5_year = compute_bucket(b5, life_5, bonus_pct);
    r.bucket_7_year = compute_bucket(b7, life_7, bonus_pct);
    r.bucket_15_year = compute_bucket(b15, life_15, bonus_pct);
    // 27.5 / 39 — no bonus per §168(k)(2)(A)(i).
    r.bucket_27_5_year = compute_bucket(b27_5, life_27_5, Decimal::ZERO);
    r.bucket_39_year = compute_bucket(b39, life_39, Decimal::ZERO);

    r.year_1_total_deduction = r.bucket_5_year.year_1_total
        + r.bucket_7_year.year_1_total
        + r.bucket_15_year.year_1_total
        + r.bucket_27_5_year.year_1_total
        + r.bucket_39_year.year_1_total;

    // Baseline: whole basis at 27.5 (residential default) or 39
    // (commercial / restaurant). Half-year approximation for the
    // comparison number — the actual mid-month detail lives in
    // rental_depreciation.
    let baseline_life = match input.property_type {
        PropertyTypeDefault::Commercial | PropertyTypeDefault::Restaurant => life_39,
        _ => life_27_5,
    };
    r.year_1_without_cost_seg =
        (input.depreciable_basis * macrs_half_year_year_1(baseline_life)).round_dp(2);

    r.year_1_acceleration = r.year_1_total_deduction - r.year_1_without_cost_seg;
    if r.note.is_empty() {
        r.note = format!(
            "cost seg: ${} year-1 (vs ${} without seg) — ${} acceleration (bonus pct {})",
            r.year_1_total_deduction,
            r.year_1_without_cost_seg,
            r.year_1_acceleration,
            (r.bonus_pct_applied * Decimal::from(100)).round_dp(0),
        );
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn base() -> CostSegInput {
        CostSegInput {
            depreciable_basis: dec!(500000),
            property_type: PropertyTypeDefault::ShortTermRental,
            allocation_override: None,
            tax_year: 2024,
            elect_bonus_depreciation: true,
        }
    }

    #[test]
    fn str_2024_60_pct_bonus_yields_huge_year_1() {
        // STR allocation: 25% 5y + 10% 7y + 10% 15y + 55% 27.5y.
        // Basis $500k.
        // Bonus pool (5/7/15 = 45% of basis) = $225,000.
        // 60% bonus = $135,000 immediate deduction.
        // Remaining $90k spread across 5/7/15 MACRS half-year.
        // 27.5y bucket = $275,000 × (1/27.5 × 0.5) = $5,000.
        let r = compute(&base());
        assert_eq!(r.bonus_pct_applied, dec!(0.60));
        let bonus_total = r.bucket_5_year.bonus_deduction
            + r.bucket_7_year.bonus_deduction
            + r.bucket_15_year.bonus_deduction;
        assert_eq!(bonus_total, dec!(135000));
        // Year 1 should be well over the without-cost-seg baseline.
        assert!(r.year_1_total_deduction > r.year_1_without_cost_seg);
        assert!(r.year_1_acceleration > Decimal::ZERO);
    }

    #[test]
    fn no_bonus_election_zeroes_the_bonus_pcts_in_all_buckets() {
        let mut i = base();
        i.elect_bonus_depreciation = false;
        let r = compute(&i);
        assert_eq!(r.bonus_pct_applied, Decimal::ZERO);
        assert_eq!(r.bucket_5_year.bonus_deduction, Decimal::ZERO);
        assert_eq!(r.bucket_7_year.bonus_deduction, Decimal::ZERO);
        assert_eq!(r.bucket_15_year.bonus_deduction, Decimal::ZERO);
    }

    #[test]
    fn real_property_buckets_never_get_bonus() {
        // Even with bonus elected, the 27.5 and 39 buckets must have
        // zero bonus_deduction (§168(k)(2)(A)(i) excludes real property).
        let r = compute(&base());
        assert_eq!(r.bucket_27_5_year.bonus_deduction, Decimal::ZERO);
        assert_eq!(r.bucket_39_year.bonus_deduction, Decimal::ZERO);
        // And the 5/7/15 buckets DID get bonus.
        assert_eq!(r.bucket_5_year.bonus_pct_applied, dec!(0.60));
    }

    #[test]
    fn bonus_phase_down_2023_through_2027() {
        for (year, expected) in [
            (2023, dec!(0.80)),
            (2024, dec!(0.60)),
            (2025, dec!(0.40)),
            (2026, dec!(0.20)),
            (2027, Decimal::ZERO),
        ] {
            assert_eq!(bonus_pct_for_year(year), expected,
                "bonus pct {year} must match TCJA phase-down");
        }
    }

    #[test]
    fn bonus_phase_down_pre_2018_50pct() {
        assert_eq!(bonus_pct_for_year(2015), dec!(0.50));
    }

    #[test]
    fn bonus_2018_to_2022_full_100pct() {
        for y in 2018..=2022 {
            assert_eq!(bonus_pct_for_year(y), dec!(1.00));
        }
    }

    #[test]
    fn bonus_2027_and_beyond_zero() {
        assert_eq!(bonus_pct_for_year(2027), Decimal::ZERO);
        assert_eq!(bonus_pct_for_year(2030), Decimal::ZERO);
        assert_eq!(bonus_pct_for_year(2099), Decimal::ZERO);
    }

    #[test]
    fn allocation_for_type_sums_to_one() {
        for t in [
            PropertyTypeDefault::SingleFamily,
            PropertyTypeDefault::MultiFamily,
            PropertyTypeDefault::ShortTermRental,
            PropertyTypeDefault::Commercial,
            PropertyTypeDefault::Restaurant,
        ] {
            let a = CostSegAllocation::for_type(t);
            assert_eq!(a.sum(), Decimal::ONE, "{t:?} allocation must sum to 1.0");
        }
    }

    #[test]
    fn allocation_override_with_bad_sum_falls_back_to_default() {
        let mut i = base();
        i.allocation_override = Some(CostSegAllocation {
            pct_5_year: dec!(0.5),
            pct_7_year: Decimal::ZERO,
            pct_15_year: Decimal::ZERO,
            pct_27_5_year: dec!(0.3), // sum = 0.8, not 1.0
            pct_39_year: Decimal::ZERO,
        });
        let r = compute(&i);
        assert!(r.note.contains("!= 1.0"));
        // Should have used STR defaults.
        assert_eq!(r.allocation_used.pct_5_year, dec!(0.25));
    }

    #[test]
    fn allocation_override_within_tolerance_used() {
        // 0.999 should be accepted (within ±0.005 tol).
        let mut i = base();
        i.allocation_override = Some(CostSegAllocation {
            pct_5_year: dec!(0.300),
            pct_7_year: Decimal::ZERO,
            pct_15_year: Decimal::ZERO,
            pct_27_5_year: dec!(0.699),
            pct_39_year: Decimal::ZERO,
        });
        let r = compute(&i);
        assert_eq!(r.allocation_used.pct_5_year, dec!(0.300));
    }

    #[test]
    fn commercial_uses_39_year_baseline_not_27_5() {
        let mut i = base();
        i.property_type = PropertyTypeDefault::Commercial;
        i.elect_bonus_depreciation = false;
        let r = compute(&i);
        // Baseline = $500k × (1 / 39 × 0.5) = $6,410.26.
        assert_eq!(r.year_1_without_cost_seg, dec!(6410.26));
    }

    #[test]
    fn residential_uses_27_5_year_baseline() {
        let mut i = base();
        i.property_type = PropertyTypeDefault::SingleFamily;
        i.elect_bonus_depreciation = false;
        let r = compute(&i);
        // Baseline = $500k × (1 / 27.5 × 0.5) = $9,090.91.
        assert_eq!(r.year_1_without_cost_seg, dec!(9090.91));
    }

    #[test]
    fn zero_basis_returns_zero_no_panic() {
        let mut i = base();
        i.depreciable_basis = Decimal::ZERO;
        let r = compute(&i);
        assert_eq!(r.year_1_total_deduction, Decimal::ZERO);
        assert!(r.note.contains("no depreciable basis"));
    }

    #[test]
    fn str_acceleration_exceeds_baseline_by_meaningful_margin() {
        // STR 2024 with 60% bonus: year 1 should be 5-10× the
        // straight-line baseline.
        let r = compute(&base());
        assert!(r.year_1_total_deduction > r.year_1_without_cost_seg * dec!(5));
    }

    #[test]
    fn restaurant_30pct_5_year_bucket_largest_bonus_pool() {
        let mut i = base();
        i.property_type = PropertyTypeDefault::Restaurant;
        let r = compute(&i);
        // 30% of $500k = $150k 5-year bucket.
        assert_eq!(r.bucket_5_year.basis_allocated, dec!(150000));
        // 60% bonus on $150k = $90k.
        assert_eq!(r.bucket_5_year.bonus_deduction, dec!(90000));
    }

    #[test]
    fn bucket_year_1_total_equals_bonus_plus_macrs_remaining() {
        let r = compute(&base());
        for b in [&r.bucket_5_year, &r.bucket_7_year, &r.bucket_15_year,
                  &r.bucket_27_5_year, &r.bucket_39_year] {
            assert_eq!(b.year_1_total, b.bonus_deduction + b.macrs_year_1_on_remaining);
        }
    }

    #[test]
    fn allocation_sum_helper_round_trip() {
        let a = CostSegAllocation::for_type(PropertyTypeDefault::MultiFamily);
        assert_eq!(a.sum(), Decimal::ONE);
    }
}
