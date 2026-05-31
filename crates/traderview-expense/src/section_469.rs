//! IRC §469 — Passive Activity Loss Limitation.
//!
//! Rental real estate is *per se* passive under §469(c)(2). Passive
//! losses can only offset passive income; the excess is disallowed for
//! the year and carried forward indefinitely until either (a) future
//! passive income absorbs it or (b) the taxpayer fully disposes of
//! the activity (triggers the suspended loss).
//!
//! Three carve-outs let the loss through:
//!
//!   1. §469(i) — **$25,000 active-participation allowance** for
//!      individuals who actively participate in rental real estate.
//!      Phases out 50¢ on the dollar between $100k and $150k MAGI
//!      ($50k–$75k MFS). Above $150k MAGI the allowance is zero.
//!
//!   2. §469(c)(7) — **Real-Estate-Professional Status (REPS)**: more
//!      than 750 hours material participation in real property trades
//!      or businesses AND more than 50% of personal services performed
//!      in such trades or businesses. Once REPS, the per-se passive
//!      treatment is lifted and losses are unlimited (subject to
//!      material participation per activity, or the §469(c)(7)(A)
//!      election to aggregate).
//!
//!   3. **Short-term rental "loophole"** — §469(j)(8) + Reg.
//!      §1.469-1T(e)(3)(ii)(A). When the average customer use period
//!      is 7 days or less (e.g. typical Airbnb / VRBO), the activity
//!      is NOT a rental activity for §469 purposes. If the taxpayer
//!      materially participates, losses are non-passive and the §25k
//!      allowance + phase-out don't apply.
//!
//! Pure compute. Caller passes the year's rental loss + MAGI + REPS
//! flag + short-term-rental flag + suspended-loss carryover from the
//! prior year. We return what's deductible this year and what carries
//! forward.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilingStatus {
    Single,
    HeadOfHousehold,
    MarriedFilingJointly,
    MarriedFilingSeparately,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section469Input {
    /// Current-year rental loss (positive number). If the property is
    /// profitable, pass zero — gains don't trigger §469 at all.
    pub current_year_loss: Decimal,
    /// Current-year rental income from OTHER passive activities, used
    /// to offset losses before the §25k allowance kicks in.
    pub other_passive_income: Decimal,
    /// Suspended passive losses brought forward from prior years.
    pub prior_year_carryover: Decimal,
    /// Modified Adjusted Gross Income (Form 8582 line 7 input).
    pub magi: Decimal,
    pub filing_status: FilingStatus,
    /// Taxpayer materially OR actively participates. §469(i) allowance
    /// requires only ACTIVE participation (lower bar than material).
    pub active_participation: bool,
    /// Real-Estate-Professional Status under §469(c)(7).
    pub reps_qualified: bool,
    /// Short-term rental: average customer use ≤ 7 days. Combined with
    /// material participation, treated as non-passive.
    pub short_term_rental_with_material_participation: bool,
    /// True if the taxpayer fully disposed of the activity this year —
    /// triggers release of ALL suspended losses per §469(g).
    pub full_disposition_this_year: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Section469Result {
    /// Total loss available to apply this year (current + carryover).
    pub total_loss_available: Decimal,
    /// Loss absorbed by other passive income.
    pub offset_against_passive_income: Decimal,
    /// §469(i) $25k allowance amount actually used.
    pub allowance_used: Decimal,
    /// Allowance that the phase-out + MAGI permitted (the cap).
    pub allowance_cap_after_phaseout: Decimal,
    /// Loss released because of REPS, STR, or full disposition.
    pub loss_released_non_passive: Decimal,
    /// Total deductible this year (sum of the three release paths).
    pub deductible_this_year: Decimal,
    /// Loss suspended into next year.
    pub suspended_to_next_year: Decimal,
    /// One-line human-readable explanation of the path taken.
    pub note: String,
}

fn d(s: &str) -> Decimal {
    Decimal::from_str(s).unwrap()
}

/// §469(i) allowance + phase-out by filing status. Returns the max
/// allowance the MAGI permits (before being capped by actual loss).
fn allowance_cap(filing_status: FilingStatus, magi: Decimal) -> Decimal {
    // §469(i)(3): MFS gets half the limits, AND must live apart from
    // spouse all year. We model the typical MFS case (allowance halved,
    // phase-out band $50k–$75k). MFS living together: zero allowance.
    let (max_allowance, phase_start, phase_end) = match filing_status {
        FilingStatus::MarriedFilingSeparately => (d("12500"), d("50000"), d("75000")),
        _ => (d("25000"), d("100000"), d("150000")),
    };
    if magi <= phase_start {
        return max_allowance;
    }
    if magi >= phase_end {
        return Decimal::ZERO;
    }
    // 50¢-on-the-dollar phase-out: allowance reduces by 50% × (MAGI - phase_start).
    let reduction = (magi - phase_start) * d("0.5");
    (max_allowance - reduction).max(Decimal::ZERO)
}

pub fn compute(input: &Section469Input) -> Section469Result {
    let mut r = Section469Result {
        total_loss_available: input.current_year_loss + input.prior_year_carryover,
        ..Section469Result::default()
    };

    if r.total_loss_available <= Decimal::ZERO {
        r.note = "no passive loss to limit".into();
        return r;
    }

    // Path 1: Full disposition releases ALL suspended losses (§469(g)).
    if input.full_disposition_this_year {
        r.loss_released_non_passive = r.total_loss_available;
        r.deductible_this_year = r.total_loss_available;
        r.suspended_to_next_year = Decimal::ZERO;
        r.note = "§469(g) full disposition releases all suspended losses".into();
        return r;
    }

    // Path 2: REPS or STR-with-material-participation -> non-passive.
    if input.reps_qualified {
        r.loss_released_non_passive = r.total_loss_available;
        r.deductible_this_year = r.total_loss_available;
        r.suspended_to_next_year = Decimal::ZERO;
        r.note = "§469(c)(7) REPS: losses are non-passive, no limit".into();
        return r;
    }
    if input.short_term_rental_with_material_participation {
        r.loss_released_non_passive = r.total_loss_available;
        r.deductible_this_year = r.total_loss_available;
        r.suspended_to_next_year = Decimal::ZERO;
        r.note = "STR + material participation: not a rental activity per Reg. §1.469-1T(e)(3)(ii)(A)".into();
        return r;
    }

    // Standard path: offset against other passive income first, then
    // §25k allowance subject to phase-out (only if active participation).
    let mut remaining = r.total_loss_available;
    let absorbed = remaining.min(input.other_passive_income.max(Decimal::ZERO));
    r.offset_against_passive_income = absorbed;
    remaining -= absorbed;

    if input.active_participation && remaining > Decimal::ZERO {
        let cap = allowance_cap(input.filing_status, input.magi);
        r.allowance_cap_after_phaseout = cap;
        let used = remaining.min(cap);
        r.allowance_used = used;
        remaining -= used;
    } else if !input.active_participation {
        r.note = "no active participation: §469(i) allowance unavailable".into();
    }

    r.deductible_this_year = r.offset_against_passive_income + r.allowance_used;
    r.suspended_to_next_year = remaining;
    if r.note.is_empty() {
        r.note = if remaining > Decimal::ZERO {
            format!("§469(i) allowance ${} used; ${} suspended to next year", r.allowance_used, remaining)
        } else {
            "§469(i) absorbed full loss within allowance + offset".to_string()
        };
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn base() -> Section469Input {
        Section469Input {
            current_year_loss: dec!(20000),
            other_passive_income: Decimal::ZERO,
            prior_year_carryover: Decimal::ZERO,
            magi: dec!(80000),
            filing_status: FilingStatus::MarriedFilingJointly,
            active_participation: true,
            reps_qualified: false,
            short_term_rental_with_material_participation: false,
            full_disposition_this_year: false,
        }
    }

    #[test]
    fn under_100k_magi_full_25k_allowance_absorbs_20k_loss() {
        let r = compute(&base());
        assert_eq!(r.allowance_cap_after_phaseout, dec!(25000));
        assert_eq!(r.allowance_used, dec!(20000));
        assert_eq!(r.deductible_this_year, dec!(20000));
        assert_eq!(r.suspended_to_next_year, Decimal::ZERO);
    }

    #[test]
    fn magi_125k_halves_allowance_to_12500() {
        // Phase-out: 50% × ($125k - $100k) = $12,500 reduction.
        // Allowance = $25k - $12.5k = $12,500.
        let mut i = base();
        i.magi = dec!(125000);
        let r = compute(&i);
        assert_eq!(r.allowance_cap_after_phaseout, dec!(12500));
        assert_eq!(r.allowance_used, dec!(12500));
        assert_eq!(r.suspended_to_next_year, dec!(7500));
    }

    #[test]
    fn magi_at_150k_zeros_allowance() {
        let mut i = base();
        i.magi = dec!(150000);
        let r = compute(&i);
        assert_eq!(r.allowance_cap_after_phaseout, Decimal::ZERO);
        assert_eq!(r.allowance_used, Decimal::ZERO);
        assert_eq!(r.suspended_to_next_year, dec!(20000));
    }

    #[test]
    fn magi_above_150k_zeros_allowance() {
        let mut i = base();
        i.magi = dec!(200000);
        let r = compute(&i);
        assert_eq!(r.allowance_used, Decimal::ZERO);
        assert_eq!(r.suspended_to_next_year, dec!(20000));
    }

    #[test]
    fn mfs_uses_half_limits_and_half_band() {
        // MFS allowance starts at $12,500; phase-out band $50k–$75k.
        let mut i = base();
        i.filing_status = FilingStatus::MarriedFilingSeparately;
        i.magi = dec!(60000); // 50% × ($60k - $50k) = $5k reduction
        let r = compute(&i);
        assert_eq!(r.allowance_cap_after_phaseout, dec!(7500));
    }

    #[test]
    fn reps_releases_full_loss_no_limit() {
        let mut i = base();
        i.magi = dec!(500000); // would normally zero the allowance
        i.reps_qualified = true;
        i.prior_year_carryover = dec!(40000); // big suspended pile
        let r = compute(&i);
        assert_eq!(r.deductible_this_year, dec!(60000));
        assert_eq!(r.suspended_to_next_year, Decimal::ZERO);
        assert!(r.note.contains("REPS"));
    }

    #[test]
    fn str_with_material_participation_releases_full_loss() {
        let mut i = base();
        i.magi = dec!(500000);
        i.short_term_rental_with_material_participation = true;
        let r = compute(&i);
        assert_eq!(r.deductible_this_year, dec!(20000));
        assert!(r.note.contains("STR"));
    }

    #[test]
    fn other_passive_income_offsets_loss_first() {
        // $20k loss vs $15k other passive income → $15k absorbed, $5k
        // remaining flows to allowance.
        let mut i = base();
        i.other_passive_income = dec!(15000);
        let r = compute(&i);
        assert_eq!(r.offset_against_passive_income, dec!(15000));
        assert_eq!(r.allowance_used, dec!(5000));
        assert_eq!(r.deductible_this_year, dec!(20000));
        assert_eq!(r.suspended_to_next_year, Decimal::ZERO);
    }

    #[test]
    fn no_active_participation_kills_allowance() {
        let mut i = base();
        i.active_participation = false;
        let r = compute(&i);
        assert_eq!(r.allowance_used, Decimal::ZERO);
        assert_eq!(r.suspended_to_next_year, dec!(20000));
        assert!(r.note.contains("active participation"));
    }

    #[test]
    fn full_disposition_releases_all_carryover() {
        let mut i = base();
        i.prior_year_carryover = dec!(80000);
        i.current_year_loss = dec!(5000);
        i.full_disposition_this_year = true;
        let r = compute(&i);
        assert_eq!(r.deductible_this_year, dec!(85000));
        assert_eq!(r.suspended_to_next_year, Decimal::ZERO);
        assert!(r.note.contains("§469(g)"));
    }

    #[test]
    fn carryover_with_partial_absorption() {
        // $20k current + $30k carryover = $50k available.
        // MAGI $80k → full $25k allowance. Deductible $25k, suspended $25k.
        let mut i = base();
        i.prior_year_carryover = dec!(30000);
        let r = compute(&i);
        assert_eq!(r.total_loss_available, dec!(50000));
        assert_eq!(r.allowance_used, dec!(25000));
        assert_eq!(r.suspended_to_next_year, dec!(25000));
    }

    #[test]
    fn no_loss_no_op() {
        let mut i = base();
        i.current_year_loss = Decimal::ZERO;
        i.prior_year_carryover = Decimal::ZERO;
        let r = compute(&i);
        assert_eq!(r.deductible_this_year, Decimal::ZERO);
        assert!(r.note.contains("no passive loss"));
    }

    #[test]
    fn phase_out_midpoint_exact() {
        // MAGI $130k → ($130k - $100k) × 0.5 = $15k reduction; allowance $10k.
        let mut i = base();
        i.magi = dec!(130000);
        let r = compute(&i);
        assert_eq!(r.allowance_cap_after_phaseout, dec!(10000));
    }

    #[test]
    fn reps_priority_over_passive_income_offset() {
        // REPS releases loss entirely; we shouldn't also "offset" against
        // passive income (that would double-count). offset_against_passive_income
        // stays zero in the REPS path.
        let mut i = base();
        i.reps_qualified = true;
        i.other_passive_income = dec!(50000);
        let r = compute(&i);
        assert_eq!(r.offset_against_passive_income, Decimal::ZERO);
        assert_eq!(r.deductible_this_year, dec!(20000));
    }
}
