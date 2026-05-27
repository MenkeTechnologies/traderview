//! US Equity Market Holiday Calendar.
//!
//! Hard-coded US equity exchange holiday calendar for 2024-2030. Used to:
//!   - Determine if a date is a trading day
//!   - Find next/previous trading day for settlement (T+1, T+2)
//!   - Compute days_in_market between two dates
//!
//! Pure compute. Calendar embedded; refresh manually when extending the
//! horizon past 2030.

use chrono::{Datelike, Duration, NaiveDate, Weekday};

/// Returns true if `date` is a US equity-market trading day (not a
/// weekend AND not on the holiday list).
pub fn is_trading_day(date: NaiveDate) -> bool {
    !matches!(date.weekday(), Weekday::Sat | Weekday::Sun) && !is_holiday(date)
}

/// Returns true if `date` is a US equity-market holiday.
pub fn is_holiday(date: NaiveDate) -> bool {
    HOLIDAYS
        .iter()
        .any(|(y, m, d)| *y == date.year() && *m == date.month() && *d == date.day())
}

/// Next trading day strictly after `date`.
pub fn next_trading_day(date: NaiveDate) -> NaiveDate {
    let mut d = date + Duration::days(1);
    while !is_trading_day(d) {
        d += Duration::days(1);
    }
    d
}

/// Previous trading day strictly before `date`.
pub fn prior_trading_day(date: NaiveDate) -> NaiveDate {
    let mut d = date - Duration::days(1);
    while !is_trading_day(d) {
        d -= Duration::days(1);
    }
    d
}

/// Add N trading days to `date`. Negative N walks backward.
pub fn add_trading_days(date: NaiveDate, n: i32) -> NaiveDate {
    let mut d = date;
    let step = if n > 0 { 1 } else { -1 };
    let mut remaining = n.abs();
    while remaining > 0 {
        d += Duration::days(step as i64);
        if is_trading_day(d) {
            remaining -= 1;
        }
    }
    d
}

/// Count trading days in [start, end] inclusive.
pub fn trading_days_between(start: NaiveDate, end: NaiveDate) -> i32 {
    if end < start {
        return 0;
    }
    let mut count = 0;
    let mut d = start;
    while d <= end {
        if is_trading_day(d) {
            count += 1;
        }
        d += Duration::days(1);
    }
    count
}

// US equity-market holidays for 2024-2030 (NYSE/NASDAQ standard).
// Format: (year, month, day). Refresh when extending past 2030.
const HOLIDAYS: &[(i32, u32, u32)] = &[
    // 2024
    (2024, 1, 1),   // New Year's Day
    (2024, 1, 15),  // MLK Day
    (2024, 2, 19),  // Presidents Day
    (2024, 3, 29),  // Good Friday
    (2024, 5, 27),  // Memorial Day
    (2024, 6, 19),  // Juneteenth
    (2024, 7, 4),   // Independence Day
    (2024, 9, 2),   // Labor Day
    (2024, 11, 28), // Thanksgiving
    (2024, 12, 25), // Christmas
    // 2025
    (2025, 1, 1),
    (2025, 1, 9), // Jimmy Carter National Day of Mourning (added)
    (2025, 1, 20),
    (2025, 2, 17),
    (2025, 4, 18),
    (2025, 5, 26),
    (2025, 6, 19),
    (2025, 7, 4),
    (2025, 9, 1),
    (2025, 11, 27),
    (2025, 12, 25),
    // 2026
    (2026, 1, 1),
    (2026, 1, 19),
    (2026, 2, 16),
    (2026, 4, 3),
    (2026, 5, 25),
    (2026, 6, 19),
    (2026, 7, 3), // 4th observed Friday
    (2026, 9, 7),
    (2026, 11, 26),
    (2026, 12, 25),
    // 2027
    (2027, 1, 1),
    (2027, 1, 18),
    (2027, 2, 15),
    (2027, 3, 26),
    (2027, 5, 31),
    (2027, 6, 18), // Juneteenth Friday observed
    (2027, 7, 5),  // 4th Monday observed
    (2027, 9, 6),
    (2027, 11, 25),
    (2027, 12, 24), // Christmas Friday observed
    // 2028
    (2028, 1, 17), // New Year's Day Sunday → MLK absorbs
    (2028, 2, 21),
    (2028, 4, 14),
    (2028, 5, 29),
    (2028, 6, 19),
    (2028, 7, 4),
    (2028, 9, 4),
    (2028, 11, 23),
    (2028, 12, 25),
    // 2029
    (2029, 1, 1),
    (2029, 1, 15),
    (2029, 2, 19),
    (2029, 3, 30),
    (2029, 5, 28),
    (2029, 6, 19),
    (2029, 7, 4),
    (2029, 9, 3),
    (2029, 11, 22),
    (2029, 12, 25),
    // 2030
    (2030, 1, 1),
    (2030, 1, 21),
    (2030, 2, 18),
    (2030, 4, 19),
    (2030, 5, 27),
    (2030, 6, 19),
    (2030, 7, 4),
    (2030, 9, 2),
    (2030, 11, 28),
    (2030, 12, 25),
];

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn weekday_non_holiday_is_trading_day() {
        // 2026-05-27 = Wednesday, not a holiday.
        assert!(is_trading_day(d(2026, 5, 27)));
    }

    #[test]
    fn saturday_not_trading_day() {
        assert!(!is_trading_day(d(2026, 5, 30))); // Sat
    }

    #[test]
    fn sunday_not_trading_day() {
        assert!(!is_trading_day(d(2026, 5, 31))); // Sun
    }

    #[test]
    fn known_holiday_not_trading_day() {
        // 2026 Independence Day (observed Fri Jul 3).
        assert!(!is_trading_day(d(2026, 7, 3)));
    }

    #[test]
    fn known_holiday_2025_new_years() {
        assert!(is_holiday(d(2025, 1, 1)));
    }

    #[test]
    fn next_trading_day_skips_weekend() {
        // Friday → Monday.
        let fri = d(2026, 5, 29);
        assert_eq!(next_trading_day(fri), d(2026, 6, 1));
    }

    #[test]
    fn next_trading_day_skips_holiday() {
        // 2026-05-22 (Fri) → 2026-05-26 (Tue) because Memorial Day Mon
        // 2026-05-25 is a holiday.
        let fri = d(2026, 5, 22);
        assert_eq!(next_trading_day(fri), d(2026, 5, 26));
    }

    #[test]
    fn prior_trading_day_skips_weekend() {
        let mon = d(2026, 6, 1);
        assert_eq!(prior_trading_day(mon), d(2026, 5, 29));
    }

    #[test]
    fn add_trading_days_one_day() {
        // T+1: Wed → Thu (no weekend/holiday between).
        let wed = d(2026, 5, 27);
        assert_eq!(add_trading_days(wed, 1), d(2026, 5, 28));
    }

    #[test]
    fn add_trading_days_t_plus_2_skips_weekend() {
        // Thu + 2 trading days = Mon (skip Sat+Sun).
        let thu = d(2026, 5, 28);
        assert_eq!(add_trading_days(thu, 2), d(2026, 6, 1));
    }

    #[test]
    fn add_trading_days_negative_walks_back() {
        let mon = d(2026, 6, 1);
        // 1 day before: Fri.
        assert_eq!(add_trading_days(mon, -1), d(2026, 5, 29));
    }

    #[test]
    fn trading_days_between_one_week_is_five() {
        let mon = d(2026, 5, 11); // Monday
        let fri = d(2026, 5, 15); // Friday
        assert_eq!(trading_days_between(mon, fri), 5);
    }

    #[test]
    fn trading_days_between_full_year_subtracts_holidays() {
        // Whole calendar year 2026. 365 days - 105 weekend days (52 weeks ×
        // 2 + 1 leap) - 9-10 holidays ≈ 252.
        let start = d(2026, 1, 1);
        let end = d(2026, 12, 31);
        let n = trading_days_between(start, end);
        assert!(
            (250..=253).contains(&n),
            "should be ~252 trading days, got {}",
            n
        );
    }

    #[test]
    fn trading_days_inverted_range_zero() {
        assert_eq!(trading_days_between(d(2026, 6, 1), d(2026, 5, 1)), 0);
    }
}
