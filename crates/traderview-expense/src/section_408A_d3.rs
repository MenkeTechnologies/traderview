//! IRC §408A(d)(3)(F) — Roth IRA conversion 5-year aging rule.
//!
//! The trap that catches early-retirees doing "Roth conversion
//! ladders" (the FIRE-movement strategy of converting traditional
//! IRA to Roth in stages each year, then withdrawing the converted
//! principal tax-free 5 years later). Each Roth conversion starts
//! its own SEPARATE 5-year clock; withdrawing converted principal
//! before BOTH 5-year aging AND age 59½ triggers a **10% §72(t)
//! early withdrawal penalty** on the converted amount.
//!
//! Distinct from the general Roth 5-year rule under §408A(d)(2)(B) for **qualified distributions** (where earnings come out tax-free + penalty-free after 5 years from first Roth contribution + reaching 59½).
//!
//! **§408A(d)(4) ordering rules** for Roth IRA distributions:
//!
//!   1. **Contributions** (regular annual contributions) come out
//!      first — ALWAYS tax-free + penalty-free, regardless of age
//!      or holding period.
//!
//!   2. **Conversions** come out next in **FIFO order** (oldest first),
//!      each subject to its OWN 5-year aging under §408A(d)(3)(F).
//!      Converted basis is tax-free always; penalty applies if
//!      under 59½ AND under 5 years from THAT conversion.
//!
//!   3. **Earnings** come out last — taxable + 10% penalty if before
//!      §408A(d)(2)(B) qualified-distribution threshold (5 years
//!      from FIRST Roth contribution + 59½).
//!
//! Age 59½ bypasses the 5-year aging rule for §72(t) penalty
//! purposes — once the taxpayer reaches 59½, conversion withdrawals
//! are penalty-free regardless of 5-year aging.
//!
//! Pure compute. Caller passes the ordered list of contributions +
//! conversions, current age, withdrawal amount, and withdrawal date;
//! we compute the tax + penalty exposure per ordering rule.

use chrono::{Months, NaiveDate};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RothConversion {
    pub conversion_date: NaiveDate,
    pub amount_converted: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section408AD3Input {
    pub withdrawal_date: NaiveDate,
    pub withdrawal_amount: Decimal,
    pub taxpayer_age_at_withdrawal: u32,
    /// Total regular Roth contributions made (not conversions).
    pub total_contributions_basis: Decimal,
    /// Ordered list of conversions, in chronological order. Each
    /// conversion has its own 5-year clock under §408A(d)(3)(F).
    pub conversions: Vec<RothConversion>,
    /// Earnings balance in the Roth IRA (FMV − contributions −
    /// conversions). Comes out last per the ordering rules.
    pub earnings_balance: Decimal,
    /// Date of FIRST Roth contribution or conversion — drives the
    /// §408A(d)(2)(B) qualified-distribution clock for earnings.
    pub first_roth_account_funding_date: NaiveDate,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Section408AD3Result {
    pub from_contributions: Decimal,
    pub from_conversions_aged: Decimal,
    pub from_conversions_unaged_penalty_exposed: Decimal,
    pub from_earnings: Decimal,
    pub taxable_amount: Decimal,
    pub penalty_72t_amount: Decimal,
    pub qualified_distribution: bool,
    pub note: String,
}

fn ten_percent() -> Decimal {
    Decimal::from_str("0.10").unwrap()
}

fn five_years_after(date: NaiveDate) -> NaiveDate {
    date.checked_add_months(Months::new(60)).unwrap_or(date)
}

pub fn compute(input: &Section408AD3Input) -> Section408AD3Result {
    let mut r = Section408AD3Result::default();

    let mut remaining = input.withdrawal_amount.max(Decimal::ZERO);
    let age_59_5 = input.taxpayer_age_at_withdrawal >= 60; // 59½ rounded up
    let general_5yr_passed =
        input.withdrawal_date >= five_years_after(input.first_roth_account_funding_date);
    r.qualified_distribution = age_59_5 && general_5yr_passed;

    // Bucket 1: Contributions (always tax-free + penalty-free).
    let from_contrib = remaining.min(input.total_contributions_basis.max(Decimal::ZERO));
    r.from_contributions = from_contrib;
    remaining -= from_contrib;

    // Bucket 2: Conversions in FIFO order — each with its own 5-year clock.
    let mut conversion_pool: Decimal = input
        .conversions
        .iter()
        .map(|c| c.amount_converted.max(Decimal::ZERO))
        .sum();
    let mut conversions_sorted = input.conversions.clone();
    conversions_sorted.sort_by_key(|c| c.conversion_date);

    for conv in &conversions_sorted {
        if remaining <= Decimal::ZERO {
            break;
        }
        let take = remaining.min(conv.amount_converted.max(Decimal::ZERO));
        let conv_5yr_passed = input.withdrawal_date >= five_years_after(conv.conversion_date);
        let aged = conv_5yr_passed || age_59_5;
        if aged {
            r.from_conversions_aged += take;
        } else {
            r.from_conversions_unaged_penalty_exposed += take;
        }
        remaining -= take;
        conversion_pool -= take;
    }

    // Bucket 3: Earnings (taxable; penalty if not qualified distribution).
    let from_earnings = remaining.min(input.earnings_balance.max(Decimal::ZERO));
    r.from_earnings = from_earnings;
    let _ = conversion_pool; // silence unused (purely informational)

    // Taxable: only earnings (when not qualified).
    r.taxable_amount = if r.qualified_distribution {
        Decimal::ZERO
    } else {
        r.from_earnings
    };

    // §72(t) penalty: 10% on unaged conversion withdrawals + earnings
    // when not qualified.
    if !age_59_5 {
        r.penalty_72t_amount = ((r.from_conversions_unaged_penalty_exposed + r.from_earnings)
            * ten_percent())
        .round_dp(2);
    }
    // If 59½+, no §72(t) penalty regardless of 5-year aging.

    r.note = if r.qualified_distribution {
        format!(
            "§408A(d)(2)(B) qualified distribution: age {} ≥ 59½ AND 5+ years from first Roth funding. Full ${} tax-free + penalty-free.",
            input.taxpayer_age_at_withdrawal, input.withdrawal_amount
        )
    } else if r.penalty_72t_amount > Decimal::ZERO {
        format!(
            "§408A(d)(4) ordering: ${} contributions (tax-free, no penalty) + ${} aged conversions + ${} unaged conversions (10% penalty) + ${} earnings (${} taxable). §72(t) penalty ${}.",
            r.from_contributions,
            r.from_conversions_aged,
            r.from_conversions_unaged_penalty_exposed,
            r.from_earnings,
            r.taxable_amount,
            r.penalty_72t_amount,
        )
    } else {
        format!(
            "§408A(d)(4) ordering: ${} contributions + ${} aged conversions + ${} earnings (${} taxable). Age 59½+ so no §72(t) penalty.",
            r.from_contributions, r.from_conversions_aged, r.from_earnings, r.taxable_amount
        )
    };
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    fn base() -> Section408AD3Input {
        Section408AD3Input {
            withdrawal_date: date(2024, 6, 1),
            withdrawal_amount: dec!(10000),
            taxpayer_age_at_withdrawal: 45,
            total_contributions_basis: dec!(15000),
            conversions: vec![],
            earnings_balance: dec!(20000),
            first_roth_account_funding_date: date(2015, 1, 1),
        }
    }

    #[test]
    fn withdrawal_from_contributions_only_no_tax_no_penalty() {
        // $10k withdrawal, $15k contributions available → fully from
        // contributions, no tax + no penalty.
        let r = compute(&base());
        assert_eq!(r.from_contributions, dec!(10000));
        assert_eq!(r.taxable_amount, Decimal::ZERO);
        assert_eq!(r.penalty_72t_amount, Decimal::ZERO);
    }

    #[test]
    fn aged_conversion_no_penalty_at_age_45() {
        // Conversion 2018-01-01, withdrawal 2024-06-01 = 6+ years past
        // 5-year aging. No penalty.
        let mut i = base();
        i.total_contributions_basis = Decimal::ZERO;
        i.conversions = vec![RothConversion {
            conversion_date: date(2018, 1, 1),
            amount_converted: dec!(10000),
        }];
        let r = compute(&i);
        assert_eq!(r.from_conversions_aged, dec!(10000));
        assert_eq!(r.penalty_72t_amount, Decimal::ZERO);
    }

    #[test]
    fn unaged_conversion_triggers_10pct_penalty_at_age_45() {
        // Conversion 2023-01-01, withdrawal 2024-06-01 = under 5 years.
        // Age 45 → 10% penalty applies.
        let mut i = base();
        i.total_contributions_basis = Decimal::ZERO;
        i.conversions = vec![RothConversion {
            conversion_date: date(2023, 1, 1),
            amount_converted: dec!(10000),
        }];
        let r = compute(&i);
        assert_eq!(r.from_conversions_unaged_penalty_exposed, dec!(10000));
        assert_eq!(r.penalty_72t_amount, dec!(1000));
    }

    #[test]
    fn unaged_conversion_no_penalty_at_age_60_plus() {
        let mut i = base();
        i.taxpayer_age_at_withdrawal = 62;
        i.total_contributions_basis = Decimal::ZERO;
        i.conversions = vec![RothConversion {
            conversion_date: date(2023, 1, 1),
            amount_converted: dec!(10000),
        }];
        let r = compute(&i);
        // Aged because age 59½+ regardless of 5-year.
        assert_eq!(r.from_conversions_aged, dec!(10000));
        assert_eq!(r.penalty_72t_amount, Decimal::ZERO);
    }

    #[test]
    fn ordering_rule_contributions_before_conversions() {
        // $5k contributions + $5k conversion. Withdraw $10k total.
        // Should come out: $5k contributions (no penalty) + $5k
        // conversion (penalty if unaged).
        let mut i = base();
        i.total_contributions_basis = dec!(5000);
        i.conversions = vec![RothConversion {
            conversion_date: date(2023, 1, 1),
            amount_converted: dec!(5000),
        }];
        let r = compute(&i);
        assert_eq!(r.from_contributions, dec!(5000));
        assert_eq!(r.from_conversions_unaged_penalty_exposed, dec!(5000));
        // Penalty only on the unaged $5k conversion: 10% = $500.
        assert_eq!(r.penalty_72t_amount, dec!(500));
    }

    #[test]
    fn fifo_ordering_oldest_conversion_first() {
        // Two conversions: 2019 (aged) and 2022 (unaged). Withdraw
        // $10k. Should take aged $5k first, then unaged $5k.
        let mut i = base();
        i.total_contributions_basis = Decimal::ZERO;
        i.conversions = vec![
            RothConversion {
                conversion_date: date(2022, 1, 1),
                amount_converted: dec!(5000),
            },
            RothConversion {
                conversion_date: date(2019, 1, 1),
                amount_converted: dec!(5000),
            },
        ];
        let r = compute(&i);
        // Aged first (2019), then unaged (2022). Withdraw $10k → $5k aged + $5k unaged.
        assert_eq!(r.from_conversions_aged, dec!(5000));
        assert_eq!(r.from_conversions_unaged_penalty_exposed, dec!(5000));
        assert_eq!(r.penalty_72t_amount, dec!(500));
    }

    #[test]
    fn earnings_taxable_and_penalized_when_not_qualified() {
        // No contributions or conversions; withdraw earnings before
        // qualified-distribution threshold.
        let mut i = base();
        i.total_contributions_basis = Decimal::ZERO;
        i.taxpayer_age_at_withdrawal = 45;
        let r = compute(&i);
        assert_eq!(r.from_earnings, dec!(10000));
        assert_eq!(r.taxable_amount, dec!(10000));
        assert_eq!(r.penalty_72t_amount, dec!(1000));
    }

    #[test]
    fn qualified_distribution_age_60_5_year_full_tax_free() {
        let mut i = base();
        i.taxpayer_age_at_withdrawal = 62;
        i.total_contributions_basis = Decimal::ZERO;
        // First Roth funding 2015, withdrawal 2024 = 9 years.
        let r = compute(&i);
        assert!(r.qualified_distribution);
        assert_eq!(r.taxable_amount, Decimal::ZERO);
        assert_eq!(r.penalty_72t_amount, Decimal::ZERO);
    }

    #[test]
    fn not_qualified_when_under_5_years_from_first_funding() {
        let mut i = base();
        i.taxpayer_age_at_withdrawal = 62;
        i.first_roth_account_funding_date = date(2022, 1, 1);
        let r = compute(&i);
        // 5-year from 2022 = 2027. 2024 < 2027 → not qualified.
        assert!(!r.qualified_distribution);
    }

    #[test]
    fn conversion_5_year_boundary_exactly_5_years_aged() {
        // Conversion 2019-06-01, withdrawal 2024-06-01 = exactly 5 years.
        // checked_add_months(60) gives 2024-06-01. Withdrawal == clock end → aged.
        let mut i = base();
        i.total_contributions_basis = Decimal::ZERO;
        i.conversions = vec![RothConversion {
            conversion_date: date(2019, 6, 1),
            amount_converted: dec!(10000),
        }];
        let r = compute(&i);
        assert_eq!(r.from_conversions_aged, dec!(10000));
    }

    #[test]
    fn conversion_4_year_11_months_29_days_not_aged() {
        // Conversion 2019-06-02, withdrawal 2024-06-01 = 1 day under 5 years.
        let mut i = base();
        i.total_contributions_basis = Decimal::ZERO;
        i.conversions = vec![RothConversion {
            conversion_date: date(2019, 6, 2),
            amount_converted: dec!(10000),
        }];
        let r = compute(&i);
        assert_eq!(r.from_conversions_unaged_penalty_exposed, dec!(10000));
    }

    #[test]
    fn multiple_conversions_some_aged_some_not() {
        // Two conversions: 2018 (aged) and 2023 (unaged). Each has own clock.
        let mut i = base();
        i.total_contributions_basis = Decimal::ZERO;
        i.withdrawal_amount = dec!(20000);
        i.conversions = vec![
            RothConversion {
                conversion_date: date(2018, 1, 1),
                amount_converted: dec!(10000),
            },
            RothConversion {
                conversion_date: date(2023, 1, 1),
                amount_converted: dec!(10000),
            },
        ];
        let r = compute(&i);
        assert_eq!(r.from_conversions_aged, dec!(10000));
        assert_eq!(r.from_conversions_unaged_penalty_exposed, dec!(10000));
        assert_eq!(r.penalty_72t_amount, dec!(1000));
    }

    #[test]
    fn withdrawal_exceeds_all_buckets_caps_at_earnings() {
        let mut i = base();
        i.withdrawal_amount = dec!(50000);
        i.total_contributions_basis = dec!(15000);
        i.conversions = vec![RothConversion {
            conversion_date: date(2018, 1, 1),
            amount_converted: dec!(10000),
        }];
        i.earnings_balance = dec!(20000);
        let r = compute(&i);
        assert_eq!(r.from_contributions, dec!(15000));
        assert_eq!(r.from_conversions_aged, dec!(10000));
        assert_eq!(r.from_earnings, dec!(20000));
        // Note: $5k of withdrawal exceeds all buckets — taxable + penalty
        // covers only what's actually available.
    }

    #[test]
    fn zero_withdrawal_no_op() {
        let mut i = base();
        i.withdrawal_amount = Decimal::ZERO;
        let r = compute(&i);
        assert_eq!(r.from_contributions, Decimal::ZERO);
        assert_eq!(r.from_conversions_aged, Decimal::ZERO);
        assert_eq!(r.from_earnings, Decimal::ZERO);
        assert_eq!(r.penalty_72t_amount, Decimal::ZERO);
    }

    #[test]
    fn empty_account_no_conversions_no_op() {
        let mut i = base();
        i.total_contributions_basis = Decimal::ZERO;
        i.conversions = vec![];
        i.earnings_balance = Decimal::ZERO;
        let r = compute(&i);
        assert_eq!(r.taxable_amount, Decimal::ZERO);
        assert_eq!(r.penalty_72t_amount, Decimal::ZERO);
    }

    #[test]
    fn note_distinguishes_qualified_vs_non_qualified_paths() {
        let mut q = base();
        q.taxpayer_age_at_withdrawal = 65;
        q.first_roth_account_funding_date = date(2010, 1, 1);
        q.total_contributions_basis = Decimal::ZERO;
        let q_r = compute(&q);
        assert!(q_r.note.contains("qualified distribution"));

        let nq = base(); // age 45
        let nq_r = compute(&nq);
        assert!(nq_r.note.contains("§408A(d)(4) ordering"));
    }

    #[test]
    fn classic_fire_conversion_ladder_5_year_wait_pays_off() {
        // Convert $10k in 2019. Wait until 2024 (5 years) to withdraw.
        // Tax-free + penalty-free since aged.
        let mut i = base();
        i.total_contributions_basis = Decimal::ZERO;
        i.conversions = vec![RothConversion {
            conversion_date: date(2019, 1, 1),
            amount_converted: dec!(10000),
        }];
        let r = compute(&i);
        assert_eq!(r.from_conversions_aged, dec!(10000));
        assert_eq!(r.penalty_72t_amount, Decimal::ZERO);
        assert_eq!(r.taxable_amount, Decimal::ZERO);
    }

    #[test]
    fn conversion_age_59_5_after_withdrawal_unaged_but_no_penalty() {
        // Age 60 + conversion only 1 year old → unaged but no penalty
        // due to 59½ rule.
        let mut i = base();
        i.taxpayer_age_at_withdrawal = 60;
        i.total_contributions_basis = Decimal::ZERO;
        i.conversions = vec![RothConversion {
            conversion_date: date(2023, 6, 1),
            amount_converted: dec!(10000),
        }];
        let r = compute(&i);
        // Aged due to age 59½+.
        assert_eq!(r.from_conversions_aged, dec!(10000));
        assert_eq!(r.penalty_72t_amount, Decimal::ZERO);
    }
}
