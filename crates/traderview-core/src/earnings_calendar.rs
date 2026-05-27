//! Earnings calendar tracker.
//!
//! Per-symbol upcoming earnings date with proximity classification:
//!   - **Past**: already happened (informational only).
//!   - **Imminent**: within 3 days — major event risk.
//!   - **Soon**: 4-14 days — plan around it.
//!   - **Distant**: > 14 days.
//!
//! Useful for the dashboard "earnings within X days" filter that warns
//! traders before they enter a position that has earnings inside their
//! intended hold window.
//!
//! Pure compute.

use chrono::{Duration, NaiveDate};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EarningsEvent {
    pub symbol: String,
    pub date: NaiveDate,
    /// "BMO" = before market open, "AMC" = after market close, "TBD".
    pub timing: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EarningsProximity {
    Past,
    Imminent,
    Soon,
    Distant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpcomingEarnings {
    pub symbol: String,
    pub date: NaiveDate,
    pub days_until: i64,
    pub proximity: EarningsProximity,
    pub timing: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EarningsReport {
    pub events: Vec<UpcomingEarnings>,
    /// Symbols with earnings within `imminent_window_days` (default 3).
    pub imminent_symbols: Vec<String>,
}

pub fn analyze(events: &[EarningsEvent], today: NaiveDate) -> EarningsReport {
    let mut report = EarningsReport::default();
    for e in events {
        let days = (e.date - today).num_days();
        let proximity = if days < 0 {
            EarningsProximity::Past
        } else if days <= 3 {
            EarningsProximity::Imminent
        } else if days <= 14 {
            EarningsProximity::Soon
        } else {
            EarningsProximity::Distant
        };
        if proximity == EarningsProximity::Imminent {
            report.imminent_symbols.push(e.symbol.clone());
        }
        report.events.push(UpcomingEarnings {
            symbol: e.symbol.clone(),
            date: e.date,
            days_until: days,
            proximity,
            timing: e.timing.clone(),
        });
    }
    // Sort by date (soonest first; past last).
    report.events.sort_by(|a, b| {
        let cmp_past = a.days_until.is_negative().cmp(&b.days_until.is_negative());
        if cmp_past != std::cmp::Ordering::Equal {
            return cmp_past;
        }
        a.days_until.cmp(&b.days_until)
    });
    report.imminent_symbols.sort();
    report
}

/// Filter to symbols that have earnings inside the trader's intended
/// hold window from today.
pub fn earnings_within_window(
    events: &[EarningsEvent],
    today: NaiveDate,
    hold_days: i64,
) -> Vec<String> {
    let end = today + Duration::days(hold_days);
    events
        .iter()
        .filter(|e| e.date >= today && e.date <= end)
        .map(|e| e.symbol.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }
    fn ev(sym: &str, date: NaiveDate) -> EarningsEvent {
        EarningsEvent {
            symbol: sym.into(),
            date,
            timing: "AMC".into(),
        }
    }

    #[test]
    fn empty_returns_default() {
        let r = analyze(&[], d(2026, 5, 27));
        assert!(r.events.is_empty());
    }

    #[test]
    fn past_earnings_classified_past() {
        let r = analyze(&[ev("AAPL", d(2026, 5, 1))], d(2026, 5, 27));
        assert_eq!(r.events[0].proximity, EarningsProximity::Past);
        assert!(r.events[0].days_until < 0);
    }

    #[test]
    fn earnings_in_2_days_classified_imminent() {
        let r = analyze(&[ev("AAPL", d(2026, 5, 29))], d(2026, 5, 27));
        assert_eq!(r.events[0].proximity, EarningsProximity::Imminent);
        assert_eq!(r.events[0].days_until, 2);
        assert_eq!(r.imminent_symbols, vec!["AAPL"]);
    }

    #[test]
    fn earnings_in_10_days_classified_soon() {
        let r = analyze(&[ev("AAPL", d(2026, 6, 6))], d(2026, 5, 27));
        assert_eq!(r.events[0].proximity, EarningsProximity::Soon);
    }

    #[test]
    fn earnings_in_30_days_classified_distant() {
        let r = analyze(&[ev("AAPL", d(2026, 6, 27))], d(2026, 5, 27));
        assert_eq!(r.events[0].proximity, EarningsProximity::Distant);
    }

    #[test]
    fn events_sorted_soonest_first_with_past_at_end() {
        let r = analyze(
            &[
                ev("FAR", d(2026, 8, 1)),
                ev("PAST", d(2026, 4, 1)),
                ev("SOON", d(2026, 6, 1)),
                ev("NEAR", d(2026, 5, 30)),
            ],
            d(2026, 5, 27),
        );
        // Future: NEAR, SOON, FAR; then Past: PAST.
        assert_eq!(r.events[0].symbol, "NEAR");
        assert_eq!(r.events[1].symbol, "SOON");
        assert_eq!(r.events[2].symbol, "FAR");
        assert_eq!(r.events[3].symbol, "PAST");
    }

    #[test]
    fn imminent_only_includes_within_3_days() {
        let r = analyze(
            &[
                ev("AAPL", d(2026, 5, 28)), // 1 day
                ev("MSFT", d(2026, 6, 5)),  // 9 days
                ev("TSLA", d(2026, 5, 30)), // 3 days exact
            ],
            d(2026, 5, 27),
        );
        // Imminent = AAPL + TSLA.
        assert_eq!(r.imminent_symbols, vec!["AAPL", "TSLA"]);
    }

    #[test]
    fn earnings_within_window_filters_by_hold_period() {
        let events = vec![
            ev("AAPL", d(2026, 5, 30)),
            ev("MSFT", d(2026, 6, 15)),
            ev("TSLA", d(2026, 7, 1)),
        ];
        // Hold for 14 days from 2026-05-27 → window [05-27, 06-10].
        let result = earnings_within_window(&events, d(2026, 5, 27), 14);
        assert_eq!(result, vec!["AAPL"]);
    }

    #[test]
    fn earnings_today_classified_imminent() {
        let r = analyze(&[ev("AAPL", d(2026, 5, 27))], d(2026, 5, 27));
        assert_eq!(r.events[0].proximity, EarningsProximity::Imminent);
        assert_eq!(r.events[0].days_until, 0);
    }
}
