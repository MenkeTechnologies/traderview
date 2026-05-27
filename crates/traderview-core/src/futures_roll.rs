//! Futures contract roll calendar.
//!
//! Each futures contract has an expiration date — typically the third
//! Friday of the contract month for equity index futures, last business
//! day of the month for many commodities. Holding into expiration risks
//! physical delivery or assignment. Most traders ROLL their position
//! 5-10 days before expiry.
//!
//! Given a list of open futures positions + today's date, emit a roll
//! schedule sorted by urgency.
//!
//! Pure compute. Expiration calendar passed in by caller (the futures
//! exchange schedule is reference data; we don't embed it).

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FuturesPosition {
    pub symbol: String,
    pub contracts: i64,
    pub expiration: NaiveDate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RollUrgency {
    Now,         // expiry within `roll_window_days`
    Soon,        // within 2× window
    Comfortable, // beyond 2× window
    Expired,     // already past expiry — emergency
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollRow {
    pub symbol: String,
    pub contracts: i64,
    pub expiration: NaiveDate,
    pub days_to_expiry: i64,
    pub urgency: RollUrgency,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RollReport {
    pub rows: Vec<RollRow>,
    pub now_count: usize,
    pub expired_count: usize,
}

pub fn schedule(
    positions: &[FuturesPosition],
    today: NaiveDate,
    roll_window_days: i64,
) -> RollReport {
    let mut report = RollReport::default();
    for p in positions {
        let days = (p.expiration - today).num_days();
        let urgency = if days < 0 {
            RollUrgency::Expired
        } else if days <= roll_window_days {
            RollUrgency::Now
        } else if days <= roll_window_days * 2 {
            RollUrgency::Soon
        } else {
            RollUrgency::Comfortable
        };
        report.rows.push(RollRow {
            symbol: p.symbol.clone(),
            contracts: p.contracts,
            expiration: p.expiration,
            days_to_expiry: days,
            urgency,
        });
        match urgency {
            RollUrgency::Now => report.now_count += 1,
            RollUrgency::Expired => report.expired_count += 1,
            _ => {}
        }
    }
    // Most urgent first (expired before now, then soon, then comfortable).
    report.rows.sort_by_key(|a| a.days_to_expiry);
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }
    fn p(sym: &str, contracts: i64, exp: NaiveDate) -> FuturesPosition {
        FuturesPosition {
            symbol: sym.into(),
            contracts,
            expiration: exp,
        }
    }

    #[test]
    fn empty_returns_default() {
        let r = schedule(&[], d(2026, 5, 27), 7);
        assert!(r.rows.is_empty());
    }

    #[test]
    fn position_within_roll_window_classified_now() {
        let r = schedule(&[p("/ES", 1, d(2026, 6, 3))], d(2026, 5, 27), 7);
        assert_eq!(r.rows[0].urgency, RollUrgency::Now);
        assert_eq!(r.now_count, 1);
    }

    #[test]
    fn position_beyond_two_window_classified_comfortable() {
        // 30 days out, window 7 → 30 > 14 → comfortable.
        let r = schedule(&[p("/ES", 1, d(2026, 6, 26))], d(2026, 5, 27), 7);
        assert_eq!(r.rows[0].urgency, RollUrgency::Comfortable);
    }

    #[test]
    fn position_in_second_window_classified_soon() {
        // 10 days out, window 7 → 7 < 10 ≤ 14 → soon.
        let r = schedule(&[p("/ES", 1, d(2026, 6, 6))], d(2026, 5, 27), 7);
        assert_eq!(r.rows[0].urgency, RollUrgency::Soon);
    }

    #[test]
    fn expired_position_classified_expired() {
        let r = schedule(&[p("/ES", 1, d(2026, 5, 20))], d(2026, 5, 27), 7);
        assert_eq!(r.rows[0].urgency, RollUrgency::Expired);
        assert_eq!(r.expired_count, 1);
    }

    #[test]
    fn rows_sorted_most_urgent_first() {
        let positions = vec![
            p("/A", 1, d(2026, 7, 1)),  // comfortable
            p("/B", 1, d(2026, 6, 1)),  // sooner
            p("/C", 1, d(2026, 5, 28)), // now
        ];
        let r = schedule(&positions, d(2026, 5, 27), 7);
        assert_eq!(r.rows[0].symbol, "/C");
        assert_eq!(r.rows[1].symbol, "/B");
        assert_eq!(r.rows[2].symbol, "/A");
    }

    #[test]
    fn days_to_expiry_negative_for_past_dates() {
        let r = schedule(&[p("/ES", 1, d(2026, 5, 20))], d(2026, 5, 27), 7);
        assert_eq!(r.rows[0].days_to_expiry, -7);
    }

    #[test]
    fn now_and_expired_counts_track_per_urgency() {
        let positions = vec![
            p("/A", 1, d(2026, 5, 20)), // expired
            p("/B", 1, d(2026, 6, 1)),  // now (5d out, window 7)
            p("/C", 1, d(2026, 5, 28)), // now (1d)
            p("/D", 1, d(2026, 7, 1)),  // comfortable
        ];
        let r = schedule(&positions, d(2026, 5, 27), 7);
        assert_eq!(r.expired_count, 1);
        assert_eq!(r.now_count, 2);
    }

    #[test]
    fn larger_window_makes_more_positions_urgent() {
        let pos = p("/ES", 1, d(2026, 6, 10));
        let small_window = schedule(std::slice::from_ref(&pos), d(2026, 5, 27), 7);
        let large_window = schedule(std::slice::from_ref(&pos), d(2026, 5, 27), 21);
        assert_eq!(small_window.rows[0].urgency, RollUrgency::Soon);
        assert_eq!(large_window.rows[0].urgency, RollUrgency::Now);
    }
}
