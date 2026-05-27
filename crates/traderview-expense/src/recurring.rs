//! Recurring-expense templates. The user defines `every month on the
//! 15th, $99 to "Anthropic — Claude Pro"`, the engine generates the
//! list of due dates within a window. Pair with `manual_entry` to
//! auto-post them.
//!
//! Pure compute. No scheduler — the route caller (or a cron) iterates
//! `due_in_window` periodically and inserts the generated transactions.

use chrono::{Datelike, Duration, NaiveDate};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Cadence {
    Daily,
    Weekly,
    BiWeekly,
    Monthly, // Same day-of-month each month; falls back to last day if absent
    Quarterly,
    Annual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub merchant: String,
    pub amount: Decimal, // negative for expense, positive for income
    pub currency: String,
    pub cadence: Cadence,
    /// First date the template applies. Cadence increments from here.
    pub anchor: NaiveDate,
    /// Optional end date — pause / cancellation marker.
    pub end: Option<NaiveDate>,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DueEntry {
    pub date: NaiveDate,
    pub merchant: String,
    pub amount: Decimal,
    pub currency: String,
    pub note: String,
}

/// Enumerate every (cadence-aligned) due date inside `[window_start, window_end]`
/// for this template, capped at 10k entries to defend against pathological
/// anchor+cadence combinations.
pub fn due_in_window(
    t: &Template,
    window_start: NaiveDate,
    window_end: NaiveDate,
) -> Vec<DueEntry> {
    if window_end < window_start {
        return vec![];
    }
    let mut out = Vec::new();
    let mut step = 0u32;
    while let Some(date) = advance(t.anchor, t.cadence, step) {
        // Walk forward until we enter or pass the window.
        if date > window_end {
            break;
        }
        if let Some(end) = t.end {
            if date > end {
                break;
            }
        }
        if date >= window_start {
            out.push(DueEntry {
                date,
                merchant: t.merchant.clone(),
                amount: t.amount,
                currency: t.currency.clone(),
                note: t.note.clone(),
            });
        }
        step += 1;
        if step > 10_000 {
            break;
        } // safety floor
    }
    out
}

fn advance(anchor: NaiveDate, cadence: Cadence, step: u32) -> Option<NaiveDate> {
    let step_i = step as i64;
    match cadence {
        Cadence::Daily => Some(anchor + Duration::days(step_i)),
        Cadence::Weekly => Some(anchor + Duration::days(7 * step_i)),
        Cadence::BiWeekly => Some(anchor + Duration::days(14 * step_i)),
        Cadence::Monthly => add_months(anchor, step_i as i32),
        Cadence::Quarterly => add_months(anchor, (step_i as i32) * 3),
        Cadence::Annual => {
            let y = anchor.year() + step_i as i32;
            NaiveDate::from_ymd_opt(y, anchor.month(), anchor.day()).or_else(|| {
                // Feb 29 → Feb 28 on non-leap.
                NaiveDate::from_ymd_opt(y, anchor.month(), 28)
            })
        }
    }
}

/// Add `n` months to `d`. If the resulting month doesn't have a matching
/// day-of-month (e.g. Jan 31 → Feb 31), use the last day of the target
/// month instead — standard month-arithmetic convention.
fn add_months(d: NaiveDate, n: i32) -> Option<NaiveDate> {
    let total_months = d.year() as i64 * 12 + (d.month() as i64 - 1) + n as i64;
    let y = (total_months.div_euclid(12)) as i32;
    let m = (total_months.rem_euclid(12)) as u32 + 1;
    // Pick the smaller of (anchor day, last day of target month).

    NaiveDate::from_ymd_opt(y, m, d.day()).or_else(|| {
        // Walk back to find the last valid day.
        for candidate_day in (1..=31u32).rev() {
            if let Some(d2) = NaiveDate::from_ymd_opt(y, m, candidate_day) {
                return Some(d2);
            }
        }
        None
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn d(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }
    fn date(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    fn netflix() -> Template {
        Template {
            merchant: "Netflix".into(),
            amount: d("-15.99"),
            currency: "USD".into(),
            cadence: Cadence::Monthly,
            anchor: date(2026, 1, 15),
            end: None,
            note: "subscription".into(),
        }
    }

    #[test]
    fn monthly_yields_one_per_month() {
        let out = due_in_window(&netflix(), date(2026, 1, 1), date(2026, 12, 31));
        assert_eq!(out.len(), 12);
        assert_eq!(out[0].date, date(2026, 1, 15));
        assert_eq!(out[11].date, date(2026, 12, 15));
        for entry in &out {
            assert_eq!(entry.amount, d("-15.99"));
            assert_eq!(entry.currency, "USD");
        }
    }

    #[test]
    fn weekly_yields_every_seven_days() {
        let t = Template {
            cadence: Cadence::Weekly,
            anchor: date(2026, 1, 5), // Monday
            ..netflix()
        };
        let out = due_in_window(&t, date(2026, 1, 1), date(2026, 1, 31));
        // Jan 5, 12, 19, 26 → 4 entries
        assert_eq!(out.len(), 4);
        assert_eq!(out[0].date, date(2026, 1, 5));
        assert_eq!(out[3].date, date(2026, 1, 26));
    }

    #[test]
    fn end_date_truncates_recurrence() {
        let t = Template {
            end: Some(date(2026, 3, 31)),
            ..netflix()
        };
        let out = due_in_window(&t, date(2026, 1, 1), date(2026, 12, 31));
        // Jan 15, Feb 15, Mar 15 only.
        assert_eq!(out.len(), 3);
    }

    #[test]
    fn jan_31_anchor_falls_back_to_feb_last_day_each_month() {
        let t = Template {
            anchor: date(2026, 1, 31),
            ..netflix()
        };
        let out = due_in_window(&t, date(2026, 1, 1), date(2026, 4, 30));
        // Jan 31, Feb 28 (non-leap), Mar 31, Apr 30.
        assert_eq!(out.len(), 4);
        assert_eq!(out[0].date, date(2026, 1, 31));
        assert_eq!(out[1].date, date(2026, 2, 28));
        assert_eq!(out[2].date, date(2026, 3, 31));
        assert_eq!(out[3].date, date(2026, 4, 30));
    }

    #[test]
    fn feb_29_leap_anchor_falls_back_to_feb_28_on_non_leap_years() {
        let t = Template {
            anchor: date(2024, 2, 29), // leap
            cadence: Cadence::Annual,
            ..netflix()
        };
        let out = due_in_window(&t, date(2024, 1, 1), date(2027, 12, 31));
        // 2024 Feb 29, 2025 Feb 28, 2026 Feb 28, 2027 Feb 28.
        assert_eq!(out.len(), 4);
        assert_eq!(
            out[0],
            DueEntry {
                date: date(2024, 2, 29),
                merchant: t.merchant.clone(),
                amount: t.amount,
                currency: t.currency.clone(),
                note: t.note.clone(),
            }
        );
        assert_eq!(out[1].date, date(2025, 2, 28));
    }

    #[test]
    fn quarterly_yields_every_three_months() {
        let t = Template {
            cadence: Cadence::Quarterly,
            anchor: date(2026, 1, 1),
            ..netflix()
        };
        let out = due_in_window(&t, date(2026, 1, 1), date(2027, 1, 1));
        // Jan 1, Apr 1, Jul 1, Oct 1, Jan 1 (next year)
        assert_eq!(out.len(), 5);
        assert_eq!(out[1].date, date(2026, 4, 1));
        assert_eq!(out[4].date, date(2027, 1, 1));
    }

    #[test]
    fn window_with_zero_overlap_returns_empty() {
        let out = due_in_window(&netflix(), date(2025, 1, 1), date(2025, 12, 31));
        // anchor is 2026-01-15, window is all of 2025 → none.
        assert!(out.is_empty());
    }

    #[test]
    fn inverted_window_returns_empty() {
        let out = due_in_window(&netflix(), date(2026, 12, 31), date(2026, 1, 1));
        assert!(out.is_empty(), "end < start must short-circuit");
    }

    #[test]
    fn bi_weekly_yields_every_14_days() {
        let t = Template {
            cadence: Cadence::BiWeekly,
            anchor: date(2026, 1, 1),
            ..netflix()
        };
        let out = due_in_window(&t, date(2026, 1, 1), date(2026, 3, 1));
        // Jan 1, Jan 15, Jan 29, Feb 12, Feb 26.
        assert_eq!(out.len(), 5);
        assert_eq!(out[1].date, date(2026, 1, 15));
        assert_eq!(out[2].date, date(2026, 1, 29));
    }

    #[test]
    fn daily_cadence_within_a_week() {
        let t = Template {
            cadence: Cadence::Daily,
            anchor: date(2026, 5, 1),
            ..netflix()
        };
        let out = due_in_window(&t, date(2026, 5, 1), date(2026, 5, 7));
        assert_eq!(out.len(), 7);
    }
}
