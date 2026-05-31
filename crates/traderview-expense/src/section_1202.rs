//! IRC §1202 — Qualified Small Business Stock (QSBS) gain exclusion.
//!
//! The most-missed tax break for founders, employees holding stock-
//! option exits, and active traders buying primary-issuance small-
//! company stock. When qualified, gain on sale is **partially or
//! fully excluded** from federal income tax up to the GREATER of:
//!
//!   * **$10,000,000** per issuer (lifetime, per taxpayer); or
//!   * **10× the taxpayer's adjusted basis** in the stock.
//!
//! Exclusion percentage depends on the acquisition date band:
//!
//!   * Pre-Feb 18, 2009 — **50%** exclusion (7% AMT preference on the
//!     excluded portion under §57(a)(7)).
//!   * Feb 18, 2009 – Sep 27, 2010 — **75%** exclusion (7% AMT pref).
//!   * After Sep 27, 2010 — **100%** exclusion (no AMT preference).
//!
//! §1202 is paired with §1244 in this crate: §1244 handles the LOSS
//! side (ordinary-loss treatment up to $50k/$100k), §1202 handles the
//! GAIN side (exclusion up to $10M / 10× basis).
//!
//! Qualification under §1202(c) + §1202(e) requires:
//!
//!   1. Stock issued by a **domestic C corporation**.
//!   2. Aggregate **gross assets ≤ $50,000,000** at issuance.
//!   3. **Original issuance** — taxpayer acquired from the corporation
//!      itself for money, property (not stock), or services.
//!   4. **Non-corporate taxpayer** (individuals, partnerships, S-corps,
//!      trusts qualify; C-corps do NOT).
//!   5. Stock held **more than 5 years**.
//!   6. At least 80% of the corp's assets used in the **active conduct
//!      of a qualified trade or business** during substantially all of
//!      the holding period.
//!   7. The business is **not an excluded business** — §1202(e)(3)
//!      excludes: any trade or business involving services in health,
//!      law, engineering, architecture, accounting, actuarial science,
//!      performing arts, consulting, athletics, financial services,
//!      brokerage services, or any business where the principal asset
//!      is the reputation or skill of one or more employees. Also
//!      excludes banking/insurance, farming, mineral extraction,
//!      hotels/motels/restaurants.
//!   8. Stock is not §1202(f) "preferred-stock-as-debt" excluded.
//!
//! Pure compute. Caller asserts the 8-point qualification checklist;
//! we compute the gain split, exclusion, and post-exclusion taxable
//! gain at LTCG rates. §1045 rollover (60-day reinvestment to defer
//! gain into other QSBS) is out of scope — caller handles that
//! upstream by reducing realized_gain before passing in.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// §1202(a) exclusion-percentage bands by acquisition date.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExclusionBand {
    /// Acquired before 2009-02-18 — 50% exclusion + 7% AMT preference.
    Pct50,
    /// Acquired 2009-02-18 through 2010-09-27 — 75% exclusion + 7% AMT.
    Pct75,
    /// Acquired after 2010-09-27 — 100% exclusion, no AMT preference.
    Pct100,
}

impl ExclusionBand {
    pub fn from_acquisition_date(d: NaiveDate) -> Self {
        let bd_75_start = NaiveDate::from_ymd_opt(2009, 2, 18).unwrap();
        let bd_100_start = NaiveDate::from_ymd_opt(2010, 9, 28).unwrap();
        if d < bd_75_start {
            ExclusionBand::Pct50
        } else if d < bd_100_start {
            ExclusionBand::Pct75
        } else {
            ExclusionBand::Pct100
        }
    }

    pub fn exclusion_pct(self) -> Decimal {
        match self {
            ExclusionBand::Pct50  => Decimal::from_str("0.50").unwrap(),
            ExclusionBand::Pct75  => Decimal::from_str("0.75").unwrap(),
            ExclusionBand::Pct100 => Decimal::from_str("1.00").unwrap(),
        }
    }

    /// Pct100 has no AMT preference; the other two have a 7% AMT add-back
    /// on the excluded portion per §57(a)(7).
    pub fn amt_preference_pct(self) -> Decimal {
        match self {
            ExclusionBand::Pct100 => Decimal::ZERO,
            _ => Decimal::from_str("0.07").unwrap(),
        }
    }
}

/// §1202(c) + §1202(e) qualification checklist. All eight must be `true`
/// for QSBS treatment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Qsbs1202Qualification {
    pub domestic_c_corporation: bool,
    pub gross_assets_at_issuance_under_50m: bool,
    pub original_issuance_to_taxpayer: bool,
    pub non_corporate_taxpayer: bool,
    pub held_more_than_5_years: bool,
    pub active_qualified_trade_or_business_80pct: bool,
    pub not_an_excluded_business: bool,
    pub stock_not_section_1202f_disqualified: bool,
}

impl Qsbs1202Qualification {
    pub fn qualifies(&self) -> bool {
        self.domestic_c_corporation
            && self.gross_assets_at_issuance_under_50m
            && self.original_issuance_to_taxpayer
            && self.non_corporate_taxpayer
            && self.held_more_than_5_years
            && self.active_qualified_trade_or_business_80pct
            && self.not_an_excluded_business
            && self.stock_not_section_1202f_disqualified
    }

    fn failures(&self) -> Vec<&'static str> {
        let mut v = Vec::new();
        if !self.domestic_c_corporation {
            v.push("not a domestic C corporation");
        }
        if !self.gross_assets_at_issuance_under_50m {
            v.push("aggregate gross assets > $50M at issuance");
        }
        if !self.original_issuance_to_taxpayer {
            v.push("not acquired at original issuance from corp");
        }
        if !self.non_corporate_taxpayer {
            v.push("corporate taxpayer (C-corps cannot use §1202)");
        }
        if !self.held_more_than_5_years {
            v.push("holding period ≤ 5 years");
        }
        if !self.active_qualified_trade_or_business_80pct {
            v.push("< 80% of assets in active qualified trade or business");
        }
        if !self.not_an_excluded_business {
            v.push("excluded business per §1202(e)(3) (health/law/finance/services/etc.)");
        }
        if !self.stock_not_section_1202f_disqualified {
            v.push("stock disqualified under §1202(f) (preferred-as-debt)");
        }
        v
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section1202Input {
    pub realized_gain: Decimal,
    pub taxpayer_basis: Decimal,
    pub acquisition_date: NaiveDate,
    pub disposition_date: NaiveDate,
    /// Gain already excluded under §1202 from PRIOR dispositions of
    /// THIS same issuer's stock. The $10M / 10× cap is per-issuer
    /// per-taxpayer lifetime; prior use shrinks the remaining cap.
    pub prior_exclusion_used_this_issuer: Decimal,
    pub qualification: Qsbs1202Qualification,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Section1202Result {
    pub band: Option<ExclusionBand>,
    pub exclusion_pct: Decimal,
    /// The per-issuer cap that applies: max($10M, 10 × basis).
    pub per_issuer_cap_total: Decimal,
    /// Cap REMAINING after subtracting prior_exclusion_used_this_issuer.
    pub cap_remaining: Decimal,
    /// Gain eligible for exclusion before cap.
    pub eligible_gain_before_cap: Decimal,
    /// Gain actually excluded (= eligible × exclusion_pct, then capped).
    pub gain_excluded: Decimal,
    /// Gain taxable at long-term capital gains rates (post-exclusion
    /// portion of eligible gain + any portion that fell outside the cap).
    pub taxable_long_term_gain: Decimal,
    /// AMT preference add-back per §57(a)(7) (7% × excluded gain for
    /// 50/75% bands; zero for the 100% band).
    pub amt_preference: Decimal,
    pub disqualified: bool,
    pub note: String,
}

fn ten_million() -> Decimal {
    Decimal::from_str("10000000").unwrap()
}

pub fn compute(input: &Section1202Input) -> Section1202Result {
    let mut r = Section1202Result::default();

    if input.realized_gain <= Decimal::ZERO {
        r.note = "no gain to exclude (§1202 applies only to gains)".into();
        return r;
    }

    if !input.qualification.qualifies() {
        r.disqualified = true;
        r.taxable_long_term_gain = input.realized_gain;
        let reasons = input.qualification.failures().join(", ");
        r.note = format!("§1202 disqualified ({reasons}); full gain taxable as LTCG");
        return r;
    }

    let band = ExclusionBand::from_acquisition_date(input.acquisition_date);
    r.band = Some(band);
    r.exclusion_pct = band.exclusion_pct();

    // Per-issuer cap: max($10M, 10 × basis).
    let ten_x_basis = input.taxpayer_basis * Decimal::from(10);
    r.per_issuer_cap_total = ten_million().max(ten_x_basis);
    r.cap_remaining = (r.per_issuer_cap_total - input.prior_exclusion_used_this_issuer)
        .max(Decimal::ZERO);

    // Eligible gain (before exclusion %) is bounded by the cap REMAINING.
    r.eligible_gain_before_cap = input.realized_gain.min(r.cap_remaining);
    r.gain_excluded = (r.eligible_gain_before_cap * r.exclusion_pct).round_dp(2);

    // Taxable portion: (eligible × (1 - excl pct)) + (realized - eligible).
    let taxable_from_eligible = r.eligible_gain_before_cap - r.gain_excluded;
    let over_cap_portion = (input.realized_gain - r.eligible_gain_before_cap).max(Decimal::ZERO);
    r.taxable_long_term_gain = taxable_from_eligible + over_cap_portion;

    r.amt_preference = (r.gain_excluded * band.amt_preference_pct()).round_dp(2);

    r.note = format!(
        "§1202 {:?}: ${} excluded ({}% × ${} eligible), ${} taxable LTCG, cap remaining ${}",
        band,
        r.gain_excluded,
        (r.exclusion_pct * Decimal::from(100)).round_dp(0),
        r.eligible_gain_before_cap,
        r.taxable_long_term_gain,
        r.cap_remaining - r.eligible_gain_before_cap,
    );
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn qualified() -> Qsbs1202Qualification {
        Qsbs1202Qualification {
            domestic_c_corporation: true,
            gross_assets_at_issuance_under_50m: true,
            original_issuance_to_taxpayer: true,
            non_corporate_taxpayer: true,
            held_more_than_5_years: true,
            active_qualified_trade_or_business_80pct: true,
            not_an_excluded_business: true,
            stock_not_section_1202f_disqualified: true,
        }
    }

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    fn base() -> Section1202Input {
        // Acquired 2018-01-01 for $100k, sold 2024-06-01 for $5M proceeds.
        // Realized gain = $4.9M.
        Section1202Input {
            realized_gain: dec!(4900000),
            taxpayer_basis: dec!(100000),
            acquisition_date: d(2018, 1, 1),
            disposition_date: d(2024, 6, 1),
            prior_exclusion_used_this_issuer: Decimal::ZERO,
            qualification: qualified(),
        }
    }

    #[test]
    fn post_2010_acquisition_full_100pct_exclusion() {
        let r = compute(&base());
        assert_eq!(r.band, Some(ExclusionBand::Pct100));
        assert_eq!(r.exclusion_pct, dec!(1.00));
        assert_eq!(r.gain_excluded, dec!(4900000));
        assert_eq!(r.taxable_long_term_gain, Decimal::ZERO);
        assert_eq!(r.amt_preference, Decimal::ZERO);
    }

    #[test]
    fn pre_2009_acquisition_50pct_band_with_amt_preference() {
        let mut i = base();
        i.acquisition_date = d(2008, 1, 1);
        let r = compute(&i);
        assert_eq!(r.band, Some(ExclusionBand::Pct50));
        assert_eq!(r.gain_excluded, dec!(2450000));     // 50% of $4.9M
        assert_eq!(r.taxable_long_term_gain, dec!(2450000));
        // 7% AMT preference on excluded portion.
        assert_eq!(r.amt_preference, dec!(171500));
    }

    #[test]
    fn mid_band_2009_to_2010_acquisition_75pct() {
        let mut i = base();
        i.acquisition_date = d(2010, 1, 1);
        let r = compute(&i);
        assert_eq!(r.band, Some(ExclusionBand::Pct75));
        assert_eq!(r.gain_excluded, dec!(3675000));     // 75% of $4.9M
        assert_eq!(r.taxable_long_term_gain, dec!(1225000));
        assert_eq!(r.amt_preference, dec!(257250));     // 7% × $3.675M
    }

    #[test]
    fn band_boundary_feb_17_2009_is_50pct() {
        let mut i = base();
        i.acquisition_date = d(2009, 2, 17);
        let r = compute(&i);
        assert_eq!(r.band, Some(ExclusionBand::Pct50));
    }

    #[test]
    fn band_boundary_feb_18_2009_is_75pct() {
        let mut i = base();
        i.acquisition_date = d(2009, 2, 18);
        let r = compute(&i);
        assert_eq!(r.band, Some(ExclusionBand::Pct75));
    }

    #[test]
    fn band_boundary_sep_27_2010_is_75pct() {
        let mut i = base();
        i.acquisition_date = d(2010, 9, 27);
        let r = compute(&i);
        assert_eq!(r.band, Some(ExclusionBand::Pct75));
    }

    #[test]
    fn band_boundary_sep_28_2010_is_100pct() {
        let mut i = base();
        i.acquisition_date = d(2010, 9, 28);
        let r = compute(&i);
        assert_eq!(r.band, Some(ExclusionBand::Pct100));
    }

    #[test]
    fn cap_uses_greater_of_10m_or_10x_basis() {
        // basis $2M -> 10× = $20M > $10M, so cap = $20M.
        let mut i = base();
        i.taxpayer_basis = dec!(2000000);
        i.realized_gain = dec!(15000000);
        let r = compute(&i);
        assert_eq!(r.per_issuer_cap_total, dec!(20000000));
        assert_eq!(r.gain_excluded, dec!(15000000)); // all under cap
        assert_eq!(r.taxable_long_term_gain, Decimal::ZERO);
    }

    #[test]
    fn over_cap_portion_is_taxable_ltcg() {
        // Cap = max(10M, 10 × 100k) = $10M. Gain $20M.
        // Eligible = $10M, excluded = $10M (100% band), over-cap = $10M.
        // Taxable = $0 + $10M = $10M.
        let mut i = base();
        i.realized_gain = dec!(20000000);
        let r = compute(&i);
        assert_eq!(r.eligible_gain_before_cap, dec!(10000000));
        assert_eq!(r.gain_excluded, dec!(10000000));
        assert_eq!(r.taxable_long_term_gain, dec!(10000000));
    }

    #[test]
    fn prior_exclusion_reduces_cap_remaining() {
        // Already used $7M of the $10M cap. New gain $5M.
        // Cap remaining = $3M. Eligible = min($5M, $3M) = $3M.
        // Excluded = $3M (100%). Over-cap = $2M taxable.
        let mut i = base();
        i.realized_gain = dec!(5000000);
        i.prior_exclusion_used_this_issuer = dec!(7000000);
        let r = compute(&i);
        assert_eq!(r.cap_remaining, dec!(3000000));
        assert_eq!(r.gain_excluded, dec!(3000000));
        assert_eq!(r.taxable_long_term_gain, dec!(2000000));
    }

    #[test]
    fn disqualification_routes_full_gain_to_ltcg() {
        let mut i = base();
        i.qualification.held_more_than_5_years = false;
        let r = compute(&i);
        assert!(r.disqualified);
        assert_eq!(r.gain_excluded, Decimal::ZERO);
        assert_eq!(r.taxable_long_term_gain, dec!(4900000));
        assert!(r.note.contains("5 years"));
    }

    #[test]
    fn excluded_business_disqualification_listed_in_note() {
        let mut i = base();
        i.qualification.not_an_excluded_business = false;
        let r = compute(&i);
        assert!(r.disqualified);
        assert!(r.note.contains("excluded business"));
    }

    #[test]
    fn no_gain_no_op() {
        let mut i = base();
        i.realized_gain = Decimal::ZERO;
        let r = compute(&i);
        assert_eq!(r.gain_excluded, Decimal::ZERO);
        assert!(r.note.contains("no gain"));
    }

    #[test]
    fn loss_returns_no_op_not_negative_exclusion() {
        let mut i = base();
        i.realized_gain = dec!(-100000);
        let r = compute(&i);
        assert_eq!(r.gain_excluded, Decimal::ZERO);
        // §1202 only excludes gains; losses fall through to §1244 or
        // capital loss treatment elsewhere.
        assert!(r.note.contains("no gain"));
    }

    #[test]
    fn corporate_taxpayer_disqualified_per_1202_a() {
        let mut i = base();
        i.qualification.non_corporate_taxpayer = false;
        let r = compute(&i);
        assert!(r.disqualified);
        assert!(r.note.contains("corporate taxpayer"));
    }

    #[test]
    fn multi_disqualification_lists_all_failures() {
        let mut i = base();
        i.qualification.domestic_c_corporation = false;
        i.qualification.gross_assets_at_issuance_under_50m = false;
        i.qualification.held_more_than_5_years = false;
        let r = compute(&i);
        assert!(r.disqualified);
        assert!(r.note.contains("C corporation"));
        assert!(r.note.contains("gross assets"));
        assert!(r.note.contains("5 years"));
    }

    #[test]
    fn qualification_helper_returns_true_only_when_all_eight_pass() {
        let q = qualified();
        assert!(q.qualifies());
        for setter in [
            |q: &mut Qsbs1202Qualification| q.domestic_c_corporation = false,
            |q: &mut Qsbs1202Qualification| q.gross_assets_at_issuance_under_50m = false,
            |q: &mut Qsbs1202Qualification| q.original_issuance_to_taxpayer = false,
            |q: &mut Qsbs1202Qualification| q.non_corporate_taxpayer = false,
            |q: &mut Qsbs1202Qualification| q.held_more_than_5_years = false,
            |q: &mut Qsbs1202Qualification| q.active_qualified_trade_or_business_80pct = false,
            |q: &mut Qsbs1202Qualification| q.not_an_excluded_business = false,
            |q: &mut Qsbs1202Qualification| q.stock_not_section_1202f_disqualified = false,
        ] {
            let mut q = qualified();
            setter(&mut q);
            assert!(!q.qualifies());
        }
    }

    #[test]
    fn full_exclusion_at_cap_with_100pct_band_zero_amt() {
        // $10M gain, $100k basis, 100% band → full $10M excluded, $0 AMT.
        let mut i = base();
        i.realized_gain = dec!(10000000);
        let r = compute(&i);
        assert_eq!(r.gain_excluded, dec!(10000000));
        assert_eq!(r.taxable_long_term_gain, Decimal::ZERO);
        assert_eq!(r.amt_preference, Decimal::ZERO);
    }
}
