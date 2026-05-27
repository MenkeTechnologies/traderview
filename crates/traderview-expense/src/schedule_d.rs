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

use chrono::{Duration, NaiveDate};
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

pub fn summarize(lots: &[ClosedLot], status: FilingStatus) -> ScheduleDReport {
    let mut report = ScheduleDReport::default();
    let one_year = Duration::days(365);
    for lot in lots {
        let held = lot.closed.signed_duration_since(lot.opened);
        if held > one_year {
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
