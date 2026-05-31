//! IRC §408(d)(3) — IRA 60-day indirect rollover rules.
//!
//! Retail trader moves IRA money between brokerages, custodians, or
//! account types. Two ways to do it:
//!
//!   * **Trustee-to-trustee transfer** — direct from old IRA to new
//!     IRA, no money touches the taxpayer. NOT a rollover under
//!     §408(d)(3); no time limit, no count limit. Always the safer
//!     path.
//!
//!   * **60-day indirect rollover** — taxpayer receives a check or
//!     distribution from old IRA, then deposits into new IRA within
//!     60 days. Subject to all three rules in this module:
//!
//!       1. **60-day window** per §408(d)(3)(A): rollover must
//!          occur within 60 calendar days of receipt of the
//!          distribution. The IRS counts calendar days from the
//!          day after receipt — day 60 is the LAST eligible day.
//!
//!       2. **Once-per-12-months** per §408(d)(3)(B) +
//!          **Bobrow v. Commissioner (2014)** + IRS Ann. 2014-15:
//!          taxpayer can complete only ONE 60-day indirect rollover
//!          per 12-month period, **AGGREGATED ACROSS ALL IRAs**
//!          (this is the Bobrow holding — was previously per-IRA).
//!          Trustee-to-trustee transfers don't count toward this
//!          limit. Roth conversions don't count toward this limit.
//!
//!       3. **§72(t) early withdrawal penalty**: any portion that
//!          fails rollover treatment is treated as a taxable
//!          distribution. Plus a **10% additional tax** if the
//!          taxpayer is under age **59½**.
//!
//! §408(d)(3)(I) **hardship waiver** lets the IRS waive the 60-day
//! requirement in cases of casualty, disaster, or other circumstances
//! beyond reasonable control. Rev. Proc. 2020-46 added a
//! "self-certification" path for 12 specific hardships (financial
//! institution error, postal error, severe damage to the taxpayer's
//! principal residence, family death, severe illness, incarceration,
//! restrictions imposed by a foreign country, etc.).
//!
//! Pure compute. Caller asserts the dates + age + hardship status; we
//! return the rollover treatment + any taxable amount + penalty.

use chrono::{Duration, NaiveDate};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section408D3Input {
    /// Date the taxpayer received the IRA distribution.
    pub distribution_date: NaiveDate,
    /// Date the rollover deposit was completed into the destination
    /// IRA. None when no rollover was attempted.
    pub rollover_completion_date: Option<NaiveDate>,
    /// Date of any PRIOR 60-day rollover within the prior 12 months
    /// (per Bobrow / Ann. 2014-15 aggregation rule). None when no
    /// prior rollover.
    pub prior_indirect_rollover_completion_date: Option<NaiveDate>,
    /// Taxpayer's age (in years) on the distribution date.
    pub taxpayer_age_at_distribution: u32,
    /// Distribution amount.
    pub distribution_amount: Decimal,
    /// True when this transfer is trustee-to-trustee (NOT §408(d)(3));
    /// short-circuits the analysis to "not a rollover" treatment.
    pub trustee_to_trustee_transfer: bool,
    /// True when the rollover came from a Roth conversion. Roth
    /// conversions don't count toward §408(d)(3)(B).
    pub roth_conversion: bool,
    /// §408(d)(3)(I) hardship waiver claimed by taxpayer (self-
    /// certification under Rev. Proc. 2020-46).
    pub hardship_waiver_claimed: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Section408D3Result {
    pub days_to_complete_rollover: Option<i64>,
    pub within_60_day_window: bool,
    pub days_since_prior_rollover: Option<i64>,
    pub within_12_month_aggregation: bool,
    pub trustee_to_trustee_path: bool,
    pub rollover_qualifies: bool,
    pub taxable_amount: Decimal,
    pub early_withdrawal_penalty_10pct: Decimal,
    pub hardship_waiver_applied: bool,
    pub reasons: Vec<String>,
    pub note: String,
}

fn ten_percent() -> Decimal {
    Decimal::from_str("0.10").unwrap()
}

pub fn compute(input: &Section408D3Input) -> Section408D3Result {
    let mut r = Section408D3Result::default();

    // Trustee-to-trustee transfer — bypass §408(d)(3) entirely.
    if input.trustee_to_trustee_transfer {
        r.trustee_to_trustee_path = true;
        r.rollover_qualifies = true;
        r.note = "trustee-to-trustee transfer: not subject to §408(d)(3); no 60-day or once-per-year limit applies".into();
        return r;
    }

    // No rollover attempted — full distribution taxable.
    let rollover_date = match input.rollover_completion_date {
        Some(d) => d,
        None => {
            r.taxable_amount = input.distribution_amount;
            if input.taxpayer_age_at_distribution < 60 {
                // §72(t) "before age 59½" — use < 60 as conservative gate
                // (caller can adjust by passing age 60 if they reached 59½).
                r.early_withdrawal_penalty_10pct =
                    (input.distribution_amount * ten_percent()).round_dp(2);
            }
            r.reasons.push("no rollover attempted".into());
            r.note = format!(
                "no §408(d)(3) rollover: full ${} taxable; ${} early withdrawal penalty at 10% (under 59½)",
                input.distribution_amount, r.early_withdrawal_penalty_10pct
            );
            return r;
        }
    };

    // 60-day window check per §408(d)(3)(A).
    let days_to_rollover = (rollover_date - input.distribution_date).num_days();
    r.days_to_complete_rollover = Some(days_to_rollover);
    r.within_60_day_window = (0..=60).contains(&days_to_rollover);

    // 12-month aggregation per §408(d)(3)(B) + Bobrow.
    // Roth conversions don't count toward this limit per §408(d)(3)(C)(ii)(II).
    let within_12_month = if let Some(prior_date) = input.prior_indirect_rollover_completion_date {
        if input.roth_conversion {
            // Current rollover is a Roth conversion → doesn't count
            // against limit. We still check the prior rollover date
            // for completeness but mark it as "within" only when the
            // current one is constrained.
            false
        } else {
            let days_since_prior = (input.distribution_date - prior_date).num_days();
            r.days_since_prior_rollover = Some(days_since_prior);
            // Within 12 months means within ~365 days.
            days_since_prior < Duration::days(365).num_days()
        }
    } else {
        false
    };
    r.within_12_month_aggregation = within_12_month;

    // Evaluate failure conditions.
    if !r.within_60_day_window {
        if input.hardship_waiver_claimed {
            r.hardship_waiver_applied = true;
            r.rollover_qualifies = true;
            r.reasons.push(format!(
                "§408(d)(3)(I) hardship waiver claimed for {}-day delay (> 60 days)",
                days_to_rollover
            ));
        } else {
            r.taxable_amount = input.distribution_amount;
            r.reasons.push(format!(
                "60-day window failed: {} days from distribution to rollover (> 60)",
                days_to_rollover
            ));
        }
    }

    if !input.roth_conversion && within_12_month {
        r.taxable_amount = input.distribution_amount;
        r.reasons.push(format!(
            "§408(d)(3)(B) Bobrow once-per-12-months violated: only {} days since prior 60-day rollover",
            r.days_since_prior_rollover.unwrap_or_default()
        ));
    }

    if r.reasons.is_empty() {
        r.rollover_qualifies = true;
        r.note = format!(
            "§408(d)(3) qualifying rollover: completed in {} days; no Bobrow conflict",
            days_to_rollover
        );
    } else if !r.rollover_qualifies {
        // Compute penalty on the taxable portion.
        if input.taxpayer_age_at_distribution < 60 {
            r.early_withdrawal_penalty_10pct =
                (r.taxable_amount * ten_percent()).round_dp(2);
        }
        r.note = format!(
            "§408(d)(3) rollover failed: {} — ${} taxable + ${} early withdrawal penalty (under 59½)",
            r.reasons.join("; "),
            r.taxable_amount,
            r.early_withdrawal_penalty_10pct
        );
    } else if r.hardship_waiver_applied {
        r.note = format!(
            "§408(d)(3) rollover qualifies under §408(d)(3)(I) hardship waiver despite {}-day delay",
            days_to_rollover
        );
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    fn base() -> Section408D3Input {
        Section408D3Input {
            distribution_date: date(2024, 6, 1),
            rollover_completion_date: Some(date(2024, 7, 15)), // 44 days
            prior_indirect_rollover_completion_date: None,
            taxpayer_age_at_distribution: 45,
            distribution_amount: dec!(50000),
            trustee_to_trustee_transfer: false,
            roth_conversion: false,
            hardship_waiver_claimed: false,
        }
    }

    #[test]
    fn within_60_day_window_no_prior_rollover_qualifies() {
        let r = compute(&base());
        assert!(r.rollover_qualifies);
        assert!(r.within_60_day_window);
        assert_eq!(r.taxable_amount, Decimal::ZERO);
    }

    #[test]
    fn day_60_exactly_qualifies() {
        let mut i = base();
        i.rollover_completion_date = Some(date(2024, 7, 31)); // 60 days
        let r = compute(&i);
        assert!(r.within_60_day_window);
        assert!(r.rollover_qualifies);
    }

    #[test]
    fn day_61_misses_window() {
        let mut i = base();
        i.rollover_completion_date = Some(date(2024, 8, 1)); // 61 days
        let r = compute(&i);
        assert!(!r.within_60_day_window);
        assert!(!r.rollover_qualifies);
        assert_eq!(r.taxable_amount, dec!(50000));
        // 10% penalty: $50k × 10% = $5k.
        assert_eq!(r.early_withdrawal_penalty_10pct, dec!(5000));
    }

    #[test]
    fn no_rollover_attempted_full_taxable_plus_penalty_under_59() {
        let mut i = base();
        i.rollover_completion_date = None;
        let r = compute(&i);
        assert!(!r.rollover_qualifies);
        assert_eq!(r.taxable_amount, dec!(50000));
        assert_eq!(r.early_withdrawal_penalty_10pct, dec!(5000));
    }

    #[test]
    fn over_59_no_early_withdrawal_penalty() {
        let mut i = base();
        i.taxpayer_age_at_distribution = 65;
        i.rollover_completion_date = None;
        let r = compute(&i);
        assert!(!r.rollover_qualifies);
        assert_eq!(r.taxable_amount, dec!(50000));
        assert_eq!(r.early_withdrawal_penalty_10pct, Decimal::ZERO);
    }

    #[test]
    fn trustee_to_trustee_transfer_bypasses_all_limits() {
        let mut i = base();
        i.trustee_to_trustee_transfer = true;
        // Even with bad facts (61-day delay + prior rollover within 12mo).
        i.rollover_completion_date = Some(date(2024, 8, 1));
        i.prior_indirect_rollover_completion_date = Some(date(2024, 3, 1));
        let r = compute(&i);
        assert!(r.rollover_qualifies);
        assert!(r.trustee_to_trustee_path);
        assert!(r.note.contains("trustee-to-trustee"));
    }

    #[test]
    fn bobrow_violation_when_prior_rollover_within_12_months() {
        // Prior rollover 6 months before current distribution.
        let mut i = base();
        i.prior_indirect_rollover_completion_date = Some(date(2023, 12, 1));
        // 2023-12-01 to 2024-06-01 = ~182 days < 365 → within 12 months.
        let r = compute(&i);
        assert!(r.within_12_month_aggregation);
        assert!(!r.rollover_qualifies);
        assert!(r.reasons.iter().any(|s| s.contains("Bobrow")));
    }

    #[test]
    fn prior_rollover_over_12_months_ago_no_bobrow_violation() {
        let mut i = base();
        i.prior_indirect_rollover_completion_date = Some(date(2023, 5, 1));
        // 2023-05-01 to 2024-06-01 = ~397 days > 365 → outside.
        let r = compute(&i);
        assert!(!r.within_12_month_aggregation);
        assert!(r.rollover_qualifies);
    }

    #[test]
    fn roth_conversion_doesnt_count_toward_bobrow() {
        let mut i = base();
        i.roth_conversion = true;
        i.prior_indirect_rollover_completion_date = Some(date(2024, 3, 1));
        let r = compute(&i);
        assert!(r.rollover_qualifies);
    }

    #[test]
    fn hardship_waiver_excuses_60_day_violation() {
        let mut i = base();
        i.rollover_completion_date = Some(date(2024, 12, 1)); // 183 days late
        i.hardship_waiver_claimed = true;
        let r = compute(&i);
        assert!(r.rollover_qualifies);
        assert!(r.hardship_waiver_applied);
        assert!(r.note.contains("§408(d)(3)(I)"));
    }

    #[test]
    fn hardship_waiver_doesnt_excuse_bobrow_violation() {
        let mut i = base();
        i.rollover_completion_date = Some(date(2024, 7, 15)); // within 60 days
        i.prior_indirect_rollover_completion_date = Some(date(2024, 3, 1));
        i.hardship_waiver_claimed = true; // doesn't apply to Bobrow
        let r = compute(&i);
        assert!(!r.rollover_qualifies);
        assert!(r.reasons.iter().any(|s| s.contains("Bobrow")));
    }

    #[test]
    fn taxable_calculation_correct_under_failed_rollover() {
        let mut i = base();
        i.distribution_amount = dec!(75000);
        i.rollover_completion_date = Some(date(2024, 9, 1)); // > 60 days
        let r = compute(&i);
        assert_eq!(r.taxable_amount, dec!(75000));
        assert_eq!(r.early_withdrawal_penalty_10pct, dec!(7500));
    }

    #[test]
    fn day_count_matches_actual_calendar_days() {
        let r = compute(&base());
        assert_eq!(r.days_to_complete_rollover, Some(44));
    }

    #[test]
    fn rollover_completed_same_day_zero_days() {
        let mut i = base();
        i.rollover_completion_date = Some(i.distribution_date);
        let r = compute(&i);
        assert_eq!(r.days_to_complete_rollover, Some(0));
        assert!(r.within_60_day_window);
        assert!(r.rollover_qualifies);
    }

    #[test]
    fn rollover_before_distribution_negative_days_not_within_window() {
        let mut i = base();
        // Pathological: rollover before distribution.
        i.rollover_completion_date = Some(date(2024, 5, 1));
        let r = compute(&i);
        assert_eq!(r.days_to_complete_rollover, Some(-31));
        assert!(!r.within_60_day_window);
        assert!(!r.rollover_qualifies);
    }

    #[test]
    fn boundary_exactly_60_days_qualifies() {
        let mut i = base();
        i.distribution_date = date(2024, 1, 1);
        i.rollover_completion_date = Some(date(2024, 3, 1)); // 60 days
        let r = compute(&i);
        assert_eq!(r.days_to_complete_rollover, Some(60));
        assert!(r.within_60_day_window);
    }

    #[test]
    fn note_distinguishes_each_failure_path() {
        let no_rollover = {
            let mut i = base();
            i.rollover_completion_date = None;
            compute(&i)
        };
        assert!(no_rollover.note.contains("no §408(d)(3) rollover"));

        let late = {
            let mut i = base();
            i.rollover_completion_date = Some(date(2024, 9, 1));
            compute(&i)
        };
        assert!(late.note.contains("rollover failed") && late.note.contains("60-day"));

        let hardship = {
            let mut i = base();
            i.rollover_completion_date = Some(date(2024, 12, 1));
            i.hardship_waiver_claimed = true;
            compute(&i)
        };
        assert!(hardship.note.contains("hardship waiver"));
    }

    #[test]
    fn bobrow_boundary_day_365_no_violation() {
        // Prior rollover exactly 365 days before distribution.
        let mut i = base();
        i.distribution_date = date(2024, 6, 1);
        i.prior_indirect_rollover_completion_date = Some(date(2023, 6, 2));
        let r = compute(&i);
        // Days since prior = (2024-06-01 - 2023-06-02) = 365.
        // 365 < 365 is false → outside window.
        assert!(!r.within_12_month_aggregation);
    }

    #[test]
    fn bobrow_boundary_day_364_violation() {
        let mut i = base();
        i.distribution_date = date(2024, 6, 1);
        i.prior_indirect_rollover_completion_date = Some(date(2023, 6, 3));
        // Days since prior = 364 — inside window.
        let r = compute(&i);
        assert!(r.within_12_month_aggregation);
    }
}
