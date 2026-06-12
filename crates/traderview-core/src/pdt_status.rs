//! Pattern-day-trader accounting (FINRA 4210): a DAY TRADE is a round
//! trip opened and closed in the same US-Eastern trading session; an
//! account under $25k equity making 4+ day trades inside 5 trading
//! days is PDT-flagged. This is the STATUS computation — the paper
//! engine surfaces it; it deliberately does not block (practicing the
//! restriction is the user's job; the sim's job is the count).

use chrono::{DateTime, NaiveDate, Utc};

pub const PDT_EQUITY_FLOOR_USD: f64 = 25_000.0;
pub const PDT_DAY_TRADE_LIMIT: usize = 4;
pub const PDT_WINDOW_TRADING_DAYS: usize = 5;

/// US-Eastern session date of an instant — the day a trader means.
/// 00:30 UTC Saturday is still Friday's session in New York.
pub fn eastern_session_date(ts: DateTime<Utc>) -> NaiveDate {
    let local = ts + chrono::Duration::hours(crate::risk_gate::us_eastern_offset_hours(ts));
    local.date_naive()
}

/// The window's oldest included session: today and the 4 prior
/// TRADING days (weekends/holidays don't burn window days).
pub fn pdt_window_start(today: NaiveDate) -> NaiveDate {
    let mut d = today;
    for _ in 1..PDT_WINDOW_TRADING_DAYS {
        d = crate::holiday_calendar::prior_trading_day(d);
    }
    d
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PdtStatus {
    pub day_trades_5d: usize,
    pub window_start: NaiveDate,
    /// 4+ day trades AND under the floor. Equity at/above the floor
    /// is never flagged regardless of count.
    pub flagged: bool,
    pub remaining_before_flag: usize,
}

/// trips = (opened_ts, closed_ts) epoch seconds, any order. equity =
/// current marked equity (None = unknown → flag conservatively only
/// on count? No: unknown equity REFUSES to flag — a false PDT flag is
/// an accusation; the count still reports).
pub fn pdt_status(trips: &[(i64, i64)], equity: Option<f64>, today: NaiveDate) -> PdtStatus {
    let start = pdt_window_start(today);
    let day_trades = trips
        .iter()
        .filter(|(o, c)| {
            let od = eastern_session_date(DateTime::from_timestamp(*o, 0).unwrap_or_default());
            let cd = eastern_session_date(DateTime::from_timestamp(*c, 0).unwrap_or_default());
            od == cd && cd >= start && cd <= today
        })
        .count();
    let under_floor = equity.is_some_and(|e| e < PDT_EQUITY_FLOOR_USD);
    PdtStatus {
        day_trades_5d: day_trades,
        window_start: start,
        flagged: day_trades >= PDT_DAY_TRADE_LIMIT && under_floor,
        remaining_before_flag: PDT_DAY_TRADE_LIMIT.saturating_sub(day_trades),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn ts(y: i32, m: u32, d: u32, h: u32) -> i64 {
        Utc.with_ymd_and_hms(y, m, d, h, 0, 0).unwrap().timestamp()
    }

    #[test]
    fn day_trade_is_same_eastern_session() {
        let today = NaiveDate::from_ymd_opt(2026, 6, 12).unwrap(); // Friday
        // Open 14:00 UTC, close 19:00 UTC same day: one day trade.
        let same = (ts(2026, 6, 12, 14), ts(2026, 6, 12, 19));
        // Open Friday, close 00:30 UTC Saturday — still Friday's
        // session in New York: ALSO a day trade.
        let overnight_utc = (ts(2026, 6, 12, 14), ts(2026, 6, 13, 0));
        // Held to the next session: not a day trade.
        let swing = (ts(2026, 6, 11, 14), ts(2026, 6, 12, 14));
        let s = pdt_status(&[same, overnight_utc, swing], Some(10_000.0), today);
        assert_eq!(s.day_trades_5d, 2);
        assert!(!s.flagged);
        assert_eq!(s.remaining_before_flag, 2);
    }

    #[test]
    fn window_counts_trading_days_and_floor_gates_flag() {
        // 2026-06-12 is Friday; 5 trading days back = Monday 06-08.
        let today = NaiveDate::from_ymd_opt(2026, 6, 12).unwrap();
        assert_eq!(pdt_window_start(today), NaiveDate::from_ymd_opt(2026, 6, 8).unwrap());
        // Four same-day trips Monday..Thursday: flagged under 25k...
        let trips: Vec<(i64, i64)> = (8..12)
            .map(|d| (ts(2026, 6, d, 14), ts(2026, 6, d, 18)))
            .collect();
        let s = pdt_status(&trips, Some(24_999.0), today);
        assert_eq!(s.day_trades_5d, 4);
        assert!(s.flagged);
        // ...not flagged at/above the floor, count identical.
        let s = pdt_status(&trips, Some(25_000.0), today);
        assert!(!s.flagged);
        assert_eq!(s.day_trades_5d, 4);
        // Unknown equity refuses to flag — a false PDT flag is an
        // accusation; the count still reports.
        let s = pdt_status(&trips, None, today);
        assert!(!s.flagged);
        // A day trade on the prior Friday (06-05) is OUTSIDE the
        // 5-trading-day window even though it's only 7 calendar days.
        let old = vec![(ts(2026, 6, 5, 14), ts(2026, 6, 5, 18))];
        assert_eq!(pdt_status(&old, Some(1_000.0), today).day_trades_5d, 0);
    }
}
