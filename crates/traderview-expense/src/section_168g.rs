//! IRC §168(g) — Alternative Depreciation System (ADS).
//!
//! The slower, straight-line depreciation regime that pairs with the
//! **§163(j)(7)(B) electing real property trade or business** carve-out
//! from iter 16's `section_163j` module. The tradeoff:
//!
//!   * **Without election** — MACRS GDS depreciation (27.5y residential
//!     / 39y commercial / 15y QIP with bonus available), but margin
//!     interest is capped at 30% of ATI (the §163(j) limit).
//!   * **With §163(j)(7)(B) election** — full interest deductibility
//!     (no §163(j) cap on business interest), BUT must use ADS for
//!     all real property in that trade or business. ADS recovery
//!     periods are longer (residential 30y / commercial 40y / QIP 20y)
//!     and **no bonus depreciation** is allowed on ADS property.
//!     The election is **IRREVOCABLE** — once made, the slower
//!     depreciation continues for the life of every property held.
//!
//! ADS recovery periods (§168(g)(2)):
//!
//!   * Residential rental property — **30 years** (post-TCJA;
//!     was 40 years for property placed in service before 2018).
//!   * Non-residential real property — **40 years**.
//!   * Qualified improvement property (QIP) — **20 years** for
//!     electing real property trades or businesses.
//!   * Personal property — class life from the §167(m) ADR system
//!     (caller passes; we don't enumerate the full table here).
//!
//! ADS method: **straight-line** (no double-declining-balance
//! acceleration). Convention: **mid-month** for real property,
//! **half-year** for personal property. Bonus depreciation per
//! §168(k)(2)(D)(i): NOT allowed on ADS property.
//!
//! Pure compute. Caller supplies basis + class + dates. We compute
//! the annual ADS deduction AND a comparison number under GDS so the
//! tradeoff against §163(j) headroom can be analyzed end-to-end.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdsPropertyClass {
    /// 30-year ADS (residential rental, post-2017 placement).
    Residential30,
    /// 40-year ADS (residential rental, pre-2018 placement).
    Residential40Legacy,
    /// 40-year ADS (non-residential real property).
    Commercial40,
    /// 20-year ADS for Qualified Improvement Property under
    /// §168(g)(3)(B), available for electing RPTBs.
    QualifiedImprovementProperty20,
    /// 5-year ADS personal property (computers, autos).
    Personal5,
    /// 7-year ADS personal property (office furniture, fixtures).
    Personal7,
    /// 15-year ADS personal property (land improvements).
    Personal15,
}

impl AdsPropertyClass {
    pub fn recovery_period_years(self) -> Decimal {
        match self {
            AdsPropertyClass::Residential30 => Decimal::from(30),
            AdsPropertyClass::Residential40Legacy => Decimal::from(40),
            AdsPropertyClass::Commercial40 => Decimal::from(40),
            AdsPropertyClass::QualifiedImprovementProperty20 => Decimal::from(20),
            AdsPropertyClass::Personal5 => Decimal::from(5),
            AdsPropertyClass::Personal7 => Decimal::from(7),
            AdsPropertyClass::Personal15 => Decimal::from(15),
        }
    }

    pub fn is_real_property(self) -> bool {
        matches!(
            self,
            AdsPropertyClass::Residential30
                | AdsPropertyClass::Residential40Legacy
                | AdsPropertyClass::Commercial40
                | AdsPropertyClass::QualifiedImprovementProperty20
        )
    }

    /// Comparison GDS recovery for the tradeoff analyzer.
    pub fn gds_comparison_years(self) -> Decimal {
        match self {
            AdsPropertyClass::Residential30 | AdsPropertyClass::Residential40Legacy => {
                Decimal::from_str("27.5").unwrap()
            }
            AdsPropertyClass::Commercial40 => Decimal::from(39),
            AdsPropertyClass::QualifiedImprovementProperty20 => Decimal::from(15),
            AdsPropertyClass::Personal5 => Decimal::from(5),
            AdsPropertyClass::Personal7 => Decimal::from(7),
            AdsPropertyClass::Personal15 => Decimal::from(15),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section168gInput {
    pub depreciable_basis: Decimal,
    pub property_class: AdsPropertyClass,
    /// Year placed in service.
    pub placed_in_service_year: i32,
    /// Month placed in service (1-12). Drives the mid-month convention
    /// for real property. Half-year convention for personal property
    /// uses 6 (mid-year) regardless.
    pub placed_in_service_month: u32,
    /// Tax year being computed.
    pub tax_year: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Section168gResult {
    pub recovery_period_years: Decimal,
    pub year_of_life: u32,
    pub annual_ads_deduction: Decimal,
    pub gds_comparison_deduction: Decimal,
    /// GDS deduction minus ADS deduction. Positive = ADS is slower
    /// (the expected case); the user gives up this amount per year
    /// in deduction in exchange for §163(j) headroom.
    pub annual_depreciation_difference: Decimal,
    pub method: &'static str,
    pub convention: &'static str,
    pub note: String,
}

fn half_year_first_or_last(year_of_life: u32, total_years: u32) -> Decimal {
    let half = Decimal::from_str("0.5").unwrap();
    if year_of_life == 0 {
        Decimal::ZERO
    } else if year_of_life == 1 || year_of_life == total_years + 1 {
        half
    } else {
        Decimal::ONE
    }
}

/// Mid-month first-year rate = (12.5 - placed_in_service_month) / 12,
/// matching IRS Pub 946 Table A-13 (40-year ADS), A-12 (mid-month
/// straight line). The placed-in-service month is treated as half a
/// month; the rest of year 1 is full months.
fn mid_month_first_year(placed_in_service_month: u32) -> Decimal {
    let m = placed_in_service_month.clamp(1, 12);
    let months_in_year_1 = Decimal::from(13) - Decimal::from(m) - Decimal::from_str("0.5").unwrap();
    months_in_year_1 / Decimal::from(12)
}

pub fn compute(input: &Section168gInput) -> Section168gResult {
    let mut r = Section168gResult {
        recovery_period_years: input.property_class.recovery_period_years(),
        method: "straight-line",
        convention: if input.property_class.is_real_property() {
            "mid-month"
        } else {
            "half-year"
        },
        ..Section168gResult::default()
    };

    let life = r.recovery_period_years;
    if life <= Decimal::ZERO || input.depreciable_basis <= Decimal::ZERO {
        r.note = "no depreciation".into();
        return r;
    }
    if input.tax_year < input.placed_in_service_year {
        r.note = "tax year before placed-in-service".into();
        return r;
    }

    r.year_of_life = (input.tax_year - input.placed_in_service_year) as u32 + 1;
    let total_recovery_years = life
        .to_string()
        .parse::<f64>()
        .map(|x| x.ceil() as u32)
        .unwrap_or(40);

    // Year-of-life rate.
    let full_year_rate = Decimal::ONE / life;
    let year_rate = if input.property_class.is_real_property() {
        // Mid-month: year 1 partial, years 2..total full, then a stub
        // year for what's left after the year-1 partial.
        if r.year_of_life == 1 {
            mid_month_first_year(input.placed_in_service_month) * full_year_rate
        } else if r.year_of_life > total_recovery_years + 1 {
            Decimal::ZERO
        } else if r.year_of_life == total_recovery_years + 1 {
            // Stub year: complement of year 1's first-year fraction.
            let y1 = mid_month_first_year(input.placed_in_service_month);
            (Decimal::ONE - y1) * full_year_rate
        } else {
            full_year_rate
        }
    } else {
        // Personal property half-year: y1 = 0.5/life, y2..total = 1/life,
        // year (total+1) = 0.5/life.
        half_year_first_or_last(r.year_of_life, total_recovery_years) * full_year_rate
    };

    r.annual_ads_deduction = (input.depreciable_basis * year_rate).round_dp(2);

    // GDS comparison: same conventions, same basis, but the GDS
    // recovery period. We use straight-line for the comparison so the
    // difference reflects PURE recovery-period delta (real GDS uses
    // 200% DB / 150% DB for personal property; using straight-line on
    // both sides over-states the early-year acceleration of GDS but
    // is what the tradeoff analyzer needs to communicate "this is
    // how much slower ADS is in year-of-life terms"). The intent is
    // a head-line annual difference, not a full GDS calculator.
    let gds_life = input.property_class.gds_comparison_years();
    let gds_year_rate = if input.property_class.is_real_property() {
        if r.year_of_life == 1 {
            mid_month_first_year(input.placed_in_service_month) / gds_life
        } else {
            Decimal::ONE / gds_life
        }
    } else {
        let gds_total = gds_life
            .to_string()
            .parse::<f64>()
            .map(|x| x.ceil() as u32)
            .unwrap_or(7);
        half_year_first_or_last(r.year_of_life, gds_total) / gds_life
    };
    r.gds_comparison_deduction =
        (input.depreciable_basis * gds_year_rate).round_dp(2);
    r.annual_depreciation_difference =
        r.gds_comparison_deduction - r.annual_ads_deduction;

    r.note = format!(
        "ADS year {}: ${} ({}y straight-line, {} convention). GDS comparison ${} → ${} per-year deduction sacrificed for §163(j) headroom.",
        r.year_of_life, r.annual_ads_deduction, life, r.convention,
        r.gds_comparison_deduction, r.annual_depreciation_difference,
    );
    r
}

// ---------------------------------------------------------------------------
// §163(j)(7)(B) electing real property trade or business tradeoff analyzer.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section163jTradeoffInput {
    /// Annual depreciation deduction sacrificed by switching from GDS
    /// to ADS — sum across all real property in the RPTB. From the
    /// `annual_depreciation_difference` output of `compute` above,
    /// or from a manual estimate.
    pub annual_depreciation_sacrificed: Decimal,
    /// Annual business interest expense currently DISALLOWED under
    /// §163(j) (i.e. the carryforward delta that the election would
    /// eliminate by removing the 30% cap).
    pub annual_interest_disallowed_under_163j: Decimal,
    /// Taxpayer's marginal federal income tax rate (0..1). Used to
    /// convert deduction dollars into actual tax-savings dollars.
    pub marginal_federal_rate: Decimal,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Section163jTradeoffResult {
    pub annual_depreciation_tax_loss: Decimal,
    pub annual_interest_tax_gain: Decimal,
    pub net_annual_benefit: Decimal,
    pub election_recommended: bool,
    pub note: String,
}

pub fn analyze_tradeoff(input: &Section163jTradeoffInput) -> Section163jTradeoffResult {
    let mut r = Section163jTradeoffResult::default();
    let rate = input.marginal_federal_rate.max(Decimal::ZERO);
    r.annual_depreciation_tax_loss =
        (input.annual_depreciation_sacrificed * rate).round_dp(2);
    r.annual_interest_tax_gain =
        (input.annual_interest_disallowed_under_163j * rate).round_dp(2);
    r.net_annual_benefit = r.annual_interest_tax_gain - r.annual_depreciation_tax_loss;
    r.election_recommended = r.net_annual_benefit > Decimal::ZERO;
    r.note = if r.election_recommended {
        format!(
            "Election worth ${} annually (gain ${} from interest deductibility − ${} from slower depreciation). NOTE: irrevocable — model multi-decade horizon before committing.",
            r.net_annual_benefit, r.annual_interest_tax_gain, r.annual_depreciation_tax_loss,
        )
    } else {
        format!(
            "Election NOT recommended: ${} net annual cost (gain ${} from interest − ${} from slower depreciation). Stay under default §163(j) cap or carry interest forward.",
            -r.net_annual_benefit, r.annual_interest_tax_gain, r.annual_depreciation_tax_loss,
        )
    };
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn residential_base() -> Section168gInput {
        Section168gInput {
            depreciable_basis: dec!(300000),
            property_class: AdsPropertyClass::Residential30,
            placed_in_service_year: 2024,
            placed_in_service_month: 1, // January
            tax_year: 2025, // year 2 — full year
        }
    }

    #[test]
    fn residential_30y_year_2_full_year_at_one_over_30() {
        let r = compute(&residential_base());
        // Year 2 full = $300k × (1/30) = $10,000.
        assert_eq!(r.annual_ads_deduction, dec!(10000));
        assert_eq!(r.recovery_period_years, dec!(30));
        assert_eq!(r.method, "straight-line");
        assert_eq!(r.convention, "mid-month");
    }

    #[test]
    fn residential_30y_year_1_january_mid_month() {
        // Mid-month Jan: 11.5/12 of full year × (1/30 × $300k) = $9,583.33.
        let mut i = residential_base();
        i.tax_year = 2024;
        let r = compute(&i);
        let expected = (dec!(11.5) / dec!(12) / dec!(30) * dec!(300000)).round_dp(2);
        assert_eq!(r.annual_ads_deduction, expected);
        assert_eq!(r.year_of_life, 1);
    }

    #[test]
    fn residential_30y_year_1_december_smallest() {
        let mut i = residential_base();
        i.placed_in_service_month = 12;
        i.tax_year = 2024;
        let r = compute(&i);
        // Mid-month Dec: 0.5/12 × (1/30) × $300k = $416.67.
        let expected = (dec!(0.5) / dec!(12) / dec!(30) * dec!(300000)).round_dp(2);
        assert_eq!(r.annual_ads_deduction, expected);
    }

    #[test]
    fn residential_30y_year_31_stub_recovers_leftover() {
        // Year 31 stub = (1 - 11.5/12) / 30 × $300k for January placement.
        let mut i = residential_base();
        i.tax_year = 2054; // year 31
        let r = compute(&i);
        let y1_frac = dec!(11.5) / dec!(12);
        let stub_rate = (Decimal::ONE - y1_frac) / dec!(30);
        let expected = (dec!(300000) * stub_rate).round_dp(2);
        assert_eq!(r.annual_ads_deduction, expected);
    }

    #[test]
    fn commercial_40y_uses_longer_recovery() {
        let mut i = residential_base();
        i.property_class = AdsPropertyClass::Commercial40;
        let r = compute(&i);
        // Year 2 full = $300k × (1/40) = $7,500.
        assert_eq!(r.annual_ads_deduction, dec!(7500));
        assert_eq!(r.recovery_period_years, dec!(40));
    }

    #[test]
    fn qip_20y_uses_shortest_real_property_recovery() {
        let mut i = residential_base();
        i.property_class = AdsPropertyClass::QualifiedImprovementProperty20;
        let r = compute(&i);
        // Year 2 = $300k × 1/20 = $15,000.
        assert_eq!(r.annual_ads_deduction, dec!(15000));
        assert_eq!(r.recovery_period_years, dec!(20));
    }

    #[test]
    fn personal_5y_half_year_convention() {
        let mut i = residential_base();
        i.property_class = AdsPropertyClass::Personal5;
        i.depreciable_basis = dec!(10000);
        i.tax_year = 2024; // year 1
        let r = compute(&i);
        // Half-year y1 = 0.5/5 × $10k = $1,000.
        assert_eq!(r.annual_ads_deduction, dec!(1000));
        assert_eq!(r.convention, "half-year");
    }

    #[test]
    fn personal_5y_year_6_stub() {
        let mut i = residential_base();
        i.property_class = AdsPropertyClass::Personal5;
        i.depreciable_basis = dec!(10000);
        i.tax_year = 2029; // year 6 — the stub
        let r = compute(&i);
        // Half-year y6 = 0.5/5 × $10k = $1,000.
        assert_eq!(r.annual_ads_deduction, dec!(1000));
    }

    #[test]
    fn residential_30y_vs_gds_27_5_difference_positive() {
        // ADS slower than GDS → difference > 0 means user gives up
        // that much annually for §163(j) headroom.
        let r = compute(&residential_base());
        assert!(r.annual_depreciation_difference > Decimal::ZERO);
        // GDS straight-line year 2 = $300k × 1/27.5 = $10,909.09.
        // ADS = $10,000. Difference = $909.09.
        let expected_gds = (dec!(300000) / dec!(27.5)).round_dp(2);
        assert_eq!(r.gds_comparison_deduction, expected_gds);
    }

    #[test]
    fn pre_service_year_returns_zero_deduction() {
        let mut i = residential_base();
        i.tax_year = 2023;
        let r = compute(&i);
        assert_eq!(r.annual_ads_deduction, Decimal::ZERO);
        assert!(r.note.contains("before placed-in-service"));
    }

    #[test]
    fn zero_basis_no_op() {
        let mut i = residential_base();
        i.depreciable_basis = Decimal::ZERO;
        let r = compute(&i);
        assert_eq!(r.annual_ads_deduction, Decimal::ZERO);
        assert!(r.note.contains("no depreciation"));
    }

    #[test]
    fn residential_40_legacy_uses_40_year_recovery() {
        let mut i = residential_base();
        i.property_class = AdsPropertyClass::Residential40Legacy;
        let r = compute(&i);
        // Year 2 = $300k × 1/40 = $7,500.
        assert_eq!(r.annual_ads_deduction, dec!(7500));
    }

    #[test]
    fn recovery_period_helper_matches_class() {
        assert_eq!(AdsPropertyClass::Residential30.recovery_period_years(), dec!(30));
        assert_eq!(AdsPropertyClass::Commercial40.recovery_period_years(), dec!(40));
        assert_eq!(AdsPropertyClass::QualifiedImprovementProperty20.recovery_period_years(), dec!(20));
        assert_eq!(AdsPropertyClass::Personal5.recovery_period_years(), dec!(5));
        assert_eq!(AdsPropertyClass::Residential40Legacy.recovery_period_years(), dec!(40));
    }

    #[test]
    fn is_real_property_helper() {
        assert!(AdsPropertyClass::Residential30.is_real_property());
        assert!(AdsPropertyClass::Residential40Legacy.is_real_property());
        assert!(AdsPropertyClass::Commercial40.is_real_property());
        assert!(AdsPropertyClass::QualifiedImprovementProperty20.is_real_property());
        assert!(!AdsPropertyClass::Personal5.is_real_property());
        assert!(!AdsPropertyClass::Personal7.is_real_property());
        assert!(!AdsPropertyClass::Personal15.is_real_property());
    }

    // ─── §163(j) tradeoff analyzer ────────────────────────────────

    #[test]
    fn tradeoff_election_worth_it_when_interest_savings_exceed_depreciation_loss() {
        // Sacrifice $1k depreciation, gain $5k interest deductibility.
        let r = analyze_tradeoff(&Section163jTradeoffInput {
            annual_depreciation_sacrificed: dec!(1000),
            annual_interest_disallowed_under_163j: dec!(5000),
            marginal_federal_rate: dec!(0.37),
        });
        assert!(r.election_recommended);
        // Depreciation loss: $1k × 37% = $370.
        // Interest gain: $5k × 37% = $1,850.
        // Net = $1,480.
        assert_eq!(r.annual_depreciation_tax_loss, dec!(370));
        assert_eq!(r.annual_interest_tax_gain, dec!(1850));
        assert_eq!(r.net_annual_benefit, dec!(1480));
        assert!(r.note.contains("irrevocable"));
    }

    #[test]
    fn tradeoff_election_not_recommended_when_depreciation_loss_exceeds_interest_gain() {
        // Sacrifice $5k depreciation, gain only $1k interest deductibility.
        let r = analyze_tradeoff(&Section163jTradeoffInput {
            annual_depreciation_sacrificed: dec!(5000),
            annual_interest_disallowed_under_163j: dec!(1000),
            marginal_federal_rate: dec!(0.37),
        });
        assert!(!r.election_recommended);
        assert!(r.net_annual_benefit < Decimal::ZERO);
        assert!(r.note.contains("NOT recommended"));
    }

    #[test]
    fn tradeoff_zero_interest_disallowed_election_pointless() {
        // No §163(j) cap actually biting → election only costs depreciation.
        let r = analyze_tradeoff(&Section163jTradeoffInput {
            annual_depreciation_sacrificed: dec!(2000),
            annual_interest_disallowed_under_163j: Decimal::ZERO,
            marginal_federal_rate: dec!(0.24),
        });
        assert!(!r.election_recommended);
    }

    #[test]
    fn tradeoff_uses_marginal_rate_to_convert_deductions_to_tax_dollars() {
        // Same inputs, different rates → different net.
        let low = analyze_tradeoff(&Section163jTradeoffInput {
            annual_depreciation_sacrificed: dec!(1000),
            annual_interest_disallowed_under_163j: dec!(5000),
            marginal_federal_rate: dec!(0.12),
        });
        let high = analyze_tradeoff(&Section163jTradeoffInput {
            annual_depreciation_sacrificed: dec!(1000),
            annual_interest_disallowed_under_163j: dec!(5000),
            marginal_federal_rate: dec!(0.37),
        });
        assert!(high.net_annual_benefit > low.net_annual_benefit);
    }
}
