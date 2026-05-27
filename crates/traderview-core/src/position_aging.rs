//! Position aging report.
//!
//! Buckets open positions by how long they've been held. Surfaces:
//!   - Day trades aging into swings (>1 day held with no exit) — the
//!     classic "intraday turned into multi-day bag-hold" pattern.
//!   - Swings aging into investments (>30 days held) — was that intentional?
//!   - Positions older than a configurable threshold (e.g. 90 days) that
//!     might be forgotten or have moved out of the original thesis.
//!
//! Pure compute. Input: list of open positions with entry timestamps;
//! caller passes "now".

use chrono::{DateTime, Utc};
#[cfg(test)]
use chrono::Duration;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenPosition {
    pub symbol: String,
    pub opened_at: DateTime<Utc>,
    pub abs_notional: f64,
    /// Originally-intended trade type. Used to flag drift.
    pub intent: TradeIntent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TradeIntent {
    DayTrade,
    Swing,
    Investment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgingRow {
    pub symbol: String,
    pub age_days: i64,
    pub bucket: AgeBucket,
    pub abs_notional: f64,
    pub intent: TradeIntent,
    /// True when actual age has exceeded the intent's expected duration.
    pub drift_flag: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgeBucket {
    Intraday,    // < 1 day
    Days,        // 1-7 days
    Weeks,       // 7-30 days
    Months,      // 30-90 days
    Quarters,    // 90-365 days
    Years,       // ≥ 365 days
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgingReport {
    pub rows: Vec<AgingRow>,
    /// DayTrade-intent positions still open after end-of-day.
    pub day_trade_aging_to_swing: Vec<String>,
    /// Swing-intent positions held > 30 days.
    pub swing_aging_to_investment: Vec<String>,
    /// Any position held longer than `stale_threshold_days`.
    pub stale_positions: Vec<String>,
}

pub fn evaluate(positions: &[OpenPosition], now: DateTime<Utc>, stale_threshold_days: i64)
    -> AgingReport
{
    let mut report = AgingReport::default();
    for p in positions {
        let age_days = (now - p.opened_at).num_days();
        let bucket = bucket_for(age_days);
        let drift_flag = drift(p.intent, age_days);
        report.rows.push(AgingRow {
            symbol: p.symbol.clone(),
            age_days,
            bucket,
            abs_notional: p.abs_notional,
            intent: p.intent,
            drift_flag,
        });
        if p.intent == TradeIntent::DayTrade && age_days >= 1 {
            report.day_trade_aging_to_swing.push(p.symbol.clone());
        }
        if p.intent == TradeIntent::Swing && age_days > 30 {
            report.swing_aging_to_investment.push(p.symbol.clone());
        }
        if age_days >= stale_threshold_days {
            report.stale_positions.push(p.symbol.clone());
        }
    }
    // Sort oldest-first.
    report.rows.sort_by(|a, b| b.age_days.cmp(&a.age_days));
    report
}

fn bucket_for(days: i64) -> AgeBucket {
    if days < 1 { AgeBucket::Intraday }
    else if days < 7 { AgeBucket::Days }
    else if days < 30 { AgeBucket::Weeks }
    else if days < 90 { AgeBucket::Months }
    else if days < 365 { AgeBucket::Quarters }
    else { AgeBucket::Years }
}

fn drift(intent: TradeIntent, age_days: i64) -> bool {
    match intent {
        TradeIntent::DayTrade   => age_days >= 1,
        TradeIntent::Swing      => age_days > 30,
        TradeIntent::Investment => false,    // investments never drift by age
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn now() -> DateTime<Utc> {
        chrono::TimeZone::with_ymd_and_hms(&Utc, 2026, 5, 27, 12, 0, 0).unwrap()
    }
    fn at(days_ago: i64) -> DateTime<Utc> {
        now() - Duration::days(days_ago)
    }
    fn pos(sym: &str, days_ago: i64, intent: TradeIntent) -> OpenPosition {
        OpenPosition {
            symbol: sym.into(),
            opened_at: at(days_ago),
            abs_notional: 10_000.0,
            intent,
        }
    }

    #[test]
    fn empty_returns_empty_report() {
        let r = evaluate(&[], now(), 90);
        assert!(r.rows.is_empty());
        assert!(r.day_trade_aging_to_swing.is_empty());
    }

    #[test]
    fn intraday_position_in_intraday_bucket() {
        let positions = vec![pos("AAPL", 0, TradeIntent::DayTrade)];
        let r = evaluate(&positions, now(), 90);
        assert_eq!(r.rows[0].bucket, AgeBucket::Intraday);
        assert!(!r.rows[0].drift_flag, "0-day-old daytrade hasn't drifted yet");
        assert!(r.day_trade_aging_to_swing.is_empty());
    }

    #[test]
    fn day_trade_held_one_day_flagged_as_drift() {
        let positions = vec![pos("AAPL", 1, TradeIntent::DayTrade)];
        let r = evaluate(&positions, now(), 90);
        assert!(r.rows[0].drift_flag);
        assert_eq!(r.day_trade_aging_to_swing, vec!["AAPL"]);
    }

    #[test]
    fn swing_within_30_days_no_drift() {
        let positions = vec![pos("MSFT", 25, TradeIntent::Swing)];
        let r = evaluate(&positions, now(), 90);
        assert!(!r.rows[0].drift_flag);
        assert_eq!(r.rows[0].bucket, AgeBucket::Weeks);
    }

    #[test]
    fn swing_over_30_days_flagged_as_aging_to_investment() {
        let positions = vec![pos("MSFT", 45, TradeIntent::Swing)];
        let r = evaluate(&positions, now(), 90);
        assert!(r.rows[0].drift_flag);
        assert_eq!(r.swing_aging_to_investment, vec!["MSFT"]);
        assert_eq!(r.rows[0].bucket, AgeBucket::Months);
    }

    #[test]
    fn investment_never_drifts_by_age() {
        let positions = vec![pos("BRK.B", 365, TradeIntent::Investment)];
        let r = evaluate(&positions, now(), 90);
        assert!(!r.rows[0].drift_flag);
    }

    #[test]
    fn stale_positions_flag_above_threshold() {
        let positions = vec![
            pos("OLD", 100, TradeIntent::Investment),
            pos("YOUNG", 30, TradeIntent::Investment),
        ];
        let r = evaluate(&positions, now(), 90);
        assert_eq!(r.stale_positions, vec!["OLD"]);
    }

    #[test]
    fn buckets_cover_full_range() {
        let positions = vec![
            pos("INTRA",   0,   TradeIntent::DayTrade),
            pos("DAYS",    3,   TradeIntent::DayTrade),
            pos("WEEKS",   15,  TradeIntent::Swing),
            pos("MONTHS",  60,  TradeIntent::Swing),
            pos("QTRS",    180, TradeIntent::Investment),
            pos("YEARS",   730, TradeIntent::Investment),
        ];
        let r = evaluate(&positions, now(), 90);
        // After oldest-first sort.
        assert_eq!(r.rows[0].bucket, AgeBucket::Years);
        assert_eq!(r.rows[1].bucket, AgeBucket::Quarters);
        assert_eq!(r.rows[2].bucket, AgeBucket::Months);
        assert_eq!(r.rows[3].bucket, AgeBucket::Weeks);
        assert_eq!(r.rows[4].bucket, AgeBucket::Days);
        assert_eq!(r.rows[5].bucket, AgeBucket::Intraday);
    }

    #[test]
    fn rows_sorted_oldest_first() {
        let positions = vec![
            pos("NEW", 1, TradeIntent::Swing),
            pos("OLD", 100, TradeIntent::Swing),
        ];
        let r = evaluate(&positions, now(), 90);
        assert_eq!(r.rows[0].symbol, "OLD");
        assert_eq!(r.rows[1].symbol, "NEW");
    }
}
