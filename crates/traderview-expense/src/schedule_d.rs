//! Schedule D capital gains summary.
//!
//! Aggregates closed trades into the standard 1040 Schedule D buckets:
//!   - Short-term (ST) — held ≤ 1 year, taxed as ordinary income.
//!   - Long-term (LT)  — held > 1 year, preferential rate.
//!
//! Net ST + Net LT → combined cap gain/loss. If COMBINED is a loss,
//! the user can deduct up to $3,000 ($1,500 MFS) against ordinary
//! income; the rest carries forward indefinitely (per §1212(b)).
//!
//! Output mirrors Schedule D line numbers so the calculator output
//! maps 1:1 to what the user types on the form.
//!
//! Pure compute. Does NOT model §1091 wash sales or §1256 60/40 —
//! caller pre-processes those (use `wash_sale.rs` and `section_1256.rs`
//! upstream).

use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClosedLot {
    pub symbol: String,
    pub opened: NaiveDate,
    pub closed: NaiveDate,
    pub realized_pnl: Decimal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilingStatus {
    Single,
    HeadOfHousehold,
    MarriedFilingJointly,
    MarriedFilingSeparately,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScheduleDReport {
    pub short_term_pnl: Decimal,
    pub long_term_pnl: Decimal,
    pub net_gain_loss: Decimal,
    /// If net is a loss, the amount deductible THIS year against
    /// ordinary income (capped at $3k single / $1.5k MFS).
    pub deductible_this_year: Decimal,
    /// Loss to roll forward to next year (per §1212(b)). Always ≥ 0.
    pub carryforward_to_next_year: Decimal,
    pub st_lot_count: usize,
    pub lt_lot_count: usize,
}

/// True iff `closed` is strictly after the one-year anniversary of `opened`.
/// Implements the IRS "more than one year" test by calendar date (so leap
/// years don't shift the boundary by a day).
pub fn is_long_term(opened: NaiveDate, closed: NaiveDate) -> bool {
    let target_year = opened.year() + 1;
    // Handle Feb 29 → Feb 28 on non-leap years.
    let anniversary = NaiveDate::from_ymd_opt(target_year, opened.month(), opened.day())
        .or_else(|| NaiveDate::from_ymd_opt(target_year, opened.month(), 28))
        .unwrap_or(opened);
    closed > anniversary
}

pub fn summarize(lots: &[ClosedLot], status: FilingStatus) -> ScheduleDReport {
    let mut report = ScheduleDReport::default();
    for lot in lots {
        // IRS rule: long-term requires holding MORE THAN ONE CALENDAR YEAR
        // (not 365 days — that's wrong across leap-year boundaries).
        // The one-year anniversary is the same month + day, next year.
        // Long-term = sold strictly AFTER the one-year anniversary.
        if is_long_term(lot.opened, lot.closed) {
            report.long_term_pnl += lot.realized_pnl;
            report.lt_lot_count += 1;
        } else {
            report.short_term_pnl += lot.realized_pnl;
            report.st_lot_count += 1;
        }
    }
    report.net_gain_loss = report.short_term_pnl + report.long_term_pnl;
    if report.net_gain_loss < Decimal::ZERO {
        let cap = match status {
            FilingStatus::MarriedFilingSeparately => Decimal::from_str("1500").unwrap(),
            _ => Decimal::from_str("3000").unwrap(),
        };
        let abs_loss = -report.net_gain_loss;
        if abs_loss <= cap {
            report.deductible_this_year = abs_loss;
            report.carryforward_to_next_year = Decimal::ZERO;
        } else {
            report.deductible_this_year = cap;
            report.carryforward_to_next_year = abs_loss - cap;
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> Decimal { Decimal::from_str(s).unwrap() }
    fn day(y: i32, m: u32, d: u32) -> NaiveDate { NaiveDate::from_ymd_opt(y, m, d).unwrap() }
    fn lot(opened: NaiveDate, closed: NaiveDate, pnl: &str) -> ClosedLot {
        ClosedLot { symbol: "X".into(), opened, closed, realized_pnl: d(pnl) }
    }

    #[test]
    fn empty_returns_default() {
        let r = summarize(&[], FilingStatus::Single);
        assert_eq!(r.net_gain_loss, Decimal::ZERO);
    }

    #[test]
    fn held_under_one_year_is_short_term() {
        let lots = vec![lot(day(2026, 1, 1), day(2026, 6, 30), "1000")];
        let r = summarize(&lots, FilingStatus::Single);
        assert_eq!(r.short_term_pnl, d("1000"));
        assert_eq!(r.long_term_pnl, Decimal::ZERO);
    }

    #[test]
    fn held_exactly_one_year_is_short_term() {
        // Boundary: held > 1 year for LT (strict).
        let lots = vec![lot(day(2025, 1, 1), day(2026, 1, 1), "1000")];
        let r = summarize(&lots, FilingStatus::Single);
        // Exactly 1 year (365 days) → NOT long-term.
        assert_eq!(r.short_term_pnl, d("1000"));
    }

    #[test]
    fn held_over_one_year_is_long_term() {
        let lots = vec![lot(day(2024, 1, 1), day(2026, 1, 2), "1000")];
        let r = summarize(&lots, FilingStatus::Single);
        assert_eq!(r.long_term_pnl, d("1000"));
        assert_eq!(r.short_term_pnl, Decimal::ZERO);
    }

    #[test]
    fn leap_year_anniversary_held_exactly_one_year_is_short_term() {
        // 2024 is leap year. Jan 15 2024 → Jan 15 2025 = 366 days (spans
        // Feb 29 2024). A naive day-count rule (>365 days = LT) would
        // INCORRECTLY classify this as long-term. The IRS rule is
        // calendar-date: exactly one year held → short-term.
        let lots = vec![lot(day(2024, 1, 15), day(2025, 1, 15), "1000")];
        let r = summarize(&lots, FilingStatus::Single);
        assert_eq!(r.short_term_pnl, d("1000"),
            "exactly-1-calendar-year hold must be short-term even across leap year");
    }

    #[test]
    fn leap_year_one_day_after_anniversary_is_long_term() {
        let lots = vec![lot(day(2024, 1, 15), day(2025, 1, 16), "1000")];
        let r = summarize(&lots, FilingStatus::Single);
        assert_eq!(r.long_term_pnl, d("1000"));
    }

    #[test]
    fn feb_29_buy_anniversary_falls_back_to_feb_28() {
        // Bought Feb 29 2024 (leap). Anniversary in 2025 doesn't exist —
        // falls back to Feb 28 2025. Sell on Feb 28 2025 = exactly one
        // year (LT? No, must be AFTER anniversary). Sell on Mar 1 = LT.
        let lots_st = vec![lot(day(2024, 2, 29), day(2025, 2, 28), "1000")];
        let r_st = summarize(&lots_st, FilingStatus::Single);
        assert_eq!(r_st.short_term_pnl, d("1000"));
        let lots_lt = vec![lot(day(2024, 2, 29), day(2025, 3, 1), "1000")];
        let r_lt = summarize(&lots_lt, FilingStatus::Single);
        assert_eq!(r_lt.long_term_pnl, d("1000"));
    }

    #[test]
    fn is_long_term_helper_handles_boundary_correctly() {
        // Direct test of the helper for cleaner regression coverage.
        // Same date next year (anniversary) → NOT long-term (strict >).
        assert!(!is_long_term(day(2024, 5, 15), day(2025, 5, 15)));
        // One day after → long-term.
        assert!(is_long_term(day(2024, 5, 15), day(2025, 5, 16)));
        // Same year (much less than year) → short-term.
        assert!(!is_long_term(day(2024, 5, 15), day(2024, 8, 15)));
        // Multiple years → long-term.
        assert!(is_long_term(day(2020, 5, 15), day(2026, 5, 15)));
    }

    #[test]
    fn net_gain_no_deduction_or_carryforward() {
        let lots = vec![lot(day(2026, 1, 1), day(2026, 3, 1), "5000")];
        let r = summarize(&lots, FilingStatus::Single);
        assert_eq!(r.net_gain_loss, d("5000"));
        assert_eq!(r.deductible_this_year, Decimal::ZERO);
        assert_eq!(r.carryforward_to_next_year, Decimal::ZERO);
    }

    #[test]
    fn small_loss_fully_deductible_no_carryforward() {
        let lots = vec![lot(day(2026, 1, 1), day(2026, 3, 1), "-2000")];
        let r = summarize(&lots, FilingStatus::Single);
        assert_eq!(r.deductible_this_year, d("2000"));
        assert_eq!(r.carryforward_to_next_year, Decimal::ZERO);
    }

    #[test]
    fn loss_above_3k_caps_at_3k_with_carryforward() {
        let lots = vec![lot(day(2026, 1, 1), day(2026, 3, 1), "-10000")];
        let r = summarize(&lots, FilingStatus::Single);
        assert_eq!(r.deductible_this_year, d("3000"));
        assert_eq!(r.carryforward_to_next_year, d("7000"));
    }

    #[test]
    fn mfs_caps_at_1500_not_3000() {
        let lots = vec![lot(day(2026, 1, 1), day(2026, 3, 1), "-5000")];
        let r = summarize(&lots, FilingStatus::MarriedFilingSeparately);
        assert_eq!(r.deductible_this_year, d("1500"));
        assert_eq!(r.carryforward_to_next_year, d("3500"));
    }

    #[test]
    fn st_loss_offset_by_lt_gain_nets_to_zero() {
        let lots = vec![
            lot(day(2026, 1, 1), day(2026, 3, 1), "-5000"),    // ST loss
            lot(day(2024, 1, 1), day(2026, 6, 1), "5000"),     // LT gain
        ];
        let r = summarize(&lots, FilingStatus::Single);
        assert_eq!(r.short_term_pnl, d("-5000"));
        assert_eq!(r.long_term_pnl, d("5000"));
        assert_eq!(r.net_gain_loss, Decimal::ZERO);
        assert_eq!(r.deductible_this_year, Decimal::ZERO);
    }

    #[test]
    fn mfj_uses_3k_cap_like_single() {
        let lots = vec![lot(day(2026, 1, 1), day(2026, 3, 1), "-5000")];
        let r = summarize(&lots, FilingStatus::MarriedFilingJointly);
        assert_eq!(r.deductible_this_year, d("3000"));
        assert_eq!(r.carryforward_to_next_year, d("2000"));
    }

    #[test]
    fn lot_counts_independently_tracked() {
        let lots = vec![
            lot(day(2026, 1, 1), day(2026, 3, 1), "100"),    // ST
            lot(day(2026, 1, 1), day(2026, 3, 1), "200"),    // ST
            lot(day(2024, 1, 1), day(2026, 6, 1), "300"),    // LT
        ];
        let r = summarize(&lots, FilingStatus::Single);
        assert_eq!(r.st_lot_count, 2);
        assert_eq!(r.lt_lot_count, 1);
    }
}
