//! Tilt / revenge-trade detector.
//!
//! Scans a chronological trade sequence and flags windows that show
//! tilt-indicative behavior:
//!   - N consecutive losers (configurable threshold) — the canonical
//!     setup for revenge trading.
//!   - Increasing position size DURING a losing streak — doubling
//!     down on tilt.
//!   - Decreasing time-between-trades below a cooloff threshold during
//!     a losing streak — rapid re-entry, no analysis.
//!
//! Pure compute. Returns the specific incidents (with trade-ID windows)
//! so the dashboard can link to the offending trades for review.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeEvent {
    pub trade_id: String,
    pub closed_at: DateTime<Utc>,
    pub pnl: f64,
    pub abs_size: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TiltConfig {
    /// Minimum consecutive losers to flag a streak.
    pub min_losing_streak: usize,
    /// Cooloff window in minutes between trades — re-entering faster
    /// than this during a losing streak flags rapid-fire revenge trading.
    pub cooloff_minutes: i64,
}

impl Default for TiltConfig {
    fn default() -> Self {
        Self {
            min_losing_streak: 3,
            cooloff_minutes: 15,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TiltIncident {
    pub kind: IncidentKind,
    pub start_trade_id: String,
    pub end_trade_id: String,
    pub trade_count: usize,
    pub total_pnl: f64,
    pub detail: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IncidentKind {
    ConsecutiveLosses,
    SizeIncreaseInDrawdown,
    RapidReentry,
}

pub fn scan(events: &[TradeEvent], cfg: &TiltConfig) -> Vec<TiltIncident> {
    let mut incidents = Vec::new();
    if events.is_empty() {
        return incidents;
    }
    let mut streak_start: Option<usize> = None;
    let mut streak_sizes_increasing = true; // assume true until proven false
    let mut rapid_reentry_in_streak = false;
    let mut last_size = 0.0;
    let mut last_time: Option<DateTime<Utc>> = None;

    for (i, e) in events.iter().enumerate() {
        let is_loss = e.pnl < 0.0;
        if is_loss {
            if streak_start.is_none() {
                streak_start = Some(i);
                streak_sizes_increasing = true;
                rapid_reentry_in_streak = false;
                last_size = e.abs_size;
                last_time = Some(e.closed_at);
                continue;
            }
            // Continue an existing streak — evaluate size escalation + rapid re-entry.
            if e.abs_size <= last_size {
                streak_sizes_increasing = false;
            }
            if let Some(prev) = last_time {
                let gap_min = (e.closed_at - prev).num_minutes();
                if gap_min < cfg.cooloff_minutes {
                    rapid_reentry_in_streak = true;
                }
            }
            last_size = e.abs_size;
            last_time = Some(e.closed_at);
        } else {
            // Streak ends — emit incidents if qualifying.
            if let Some(start) = streak_start {
                let len = i - start;
                if len >= cfg.min_losing_streak {
                    emit_streak(
                        events,
                        start,
                        i - 1,
                        streak_sizes_increasing,
                        rapid_reentry_in_streak,
                        &mut incidents,
                    );
                }
            }
            streak_start = None;
        }
    }
    // Final streak that runs to end of input.
    if let Some(start) = streak_start {
        let len = events.len() - start;
        if len >= cfg.min_losing_streak {
            emit_streak(
                events,
                start,
                events.len() - 1,
                streak_sizes_increasing,
                rapid_reentry_in_streak,
                &mut incidents,
            );
        }
    }
    incidents
}

fn emit_streak(
    events: &[TradeEvent],
    start: usize,
    end: usize,
    sizes_increasing: bool,
    rapid: bool,
    out: &mut Vec<TiltIncident>,
) {
    let trade_count = end - start + 1;
    let total_pnl: f64 = events[start..=end].iter().map(|e| e.pnl).sum();
    let start_id = events[start].trade_id.clone();
    let end_id = events[end].trade_id.clone();
    out.push(TiltIncident {
        kind: IncidentKind::ConsecutiveLosses,
        start_trade_id: start_id.clone(),
        end_trade_id: end_id.clone(),
        trade_count,
        total_pnl,
        detail: format!("{} losing trades in a row", trade_count),
    });
    if sizes_increasing && trade_count >= 2 {
        out.push(TiltIncident {
            kind: IncidentKind::SizeIncreaseInDrawdown,
            start_trade_id: start_id.clone(),
            end_trade_id: end_id.clone(),
            trade_count,
            total_pnl,
            detail: "position size increased on every loss — doubling down".into(),
        });
    }
    if rapid {
        out.push(TiltIncident {
            kind: IncidentKind::RapidReentry,
            start_trade_id: start_id,
            end_trade_id: end_id,
            trade_count,
            total_pnl,
            detail: "re-entered inside cooloff window during losing streak".into(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, TimeZone};

    fn at(min: i64) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 5, 27, 14, 0, 0).unwrap() + Duration::minutes(min)
    }
    fn ev(id: &str, t_min: i64, pnl: f64, size: f64) -> TradeEvent {
        TradeEvent {
            trade_id: id.into(),
            closed_at: at(t_min),
            pnl,
            abs_size: size,
        }
    }

    #[test]
    fn empty_returns_no_incidents() {
        let out = scan(&[], &TiltConfig::default());
        assert!(out.is_empty());
    }

    #[test]
    fn all_winners_no_incidents() {
        let events = vec![
            ev("1", 0, 100.0, 1000.0),
            ev("2", 60, 100.0, 1000.0),
            ev("3", 120, 100.0, 1000.0),
        ];
        let out = scan(&events, &TiltConfig::default());
        assert!(out.is_empty());
    }

    #[test]
    fn two_consecutive_losses_below_threshold() {
        let events = vec![ev("1", 0, -100.0, 1000.0), ev("2", 60, -100.0, 1000.0)];
        let out = scan(&events, &TiltConfig::default());
        assert!(out.is_empty(), "threshold is 3 — 2 losses doesn't trigger");
    }

    #[test]
    fn three_consecutive_losses_emits_streak_incident() {
        let events = vec![
            ev("1", 0, -100.0, 1000.0),
            ev("2", 60, -100.0, 1000.0),
            ev("3", 120, -100.0, 1000.0),
        ];
        let out = scan(&events, &TiltConfig::default());
        assert_eq!(
            out.len(),
            1,
            "size-flat streak only emits ConsecutiveLosses"
        );
        assert_eq!(out[0].kind, IncidentKind::ConsecutiveLosses);
        assert_eq!(out[0].trade_count, 3);
        assert_eq!(out[0].total_pnl, -300.0);
    }

    #[test]
    fn streak_with_increasing_size_also_emits_size_incident() {
        let events = vec![
            ev("1", 0, -100.0, 1000.0),
            ev("2", 60, -100.0, 2000.0),
            ev("3", 120, -100.0, 3000.0),
        ];
        let out = scan(&events, &TiltConfig::default());
        assert!(out
            .iter()
            .any(|i| i.kind == IncidentKind::ConsecutiveLosses));
        assert!(out
            .iter()
            .any(|i| i.kind == IncidentKind::SizeIncreaseInDrawdown));
    }

    #[test]
    fn streak_with_rapid_reentry_emits_rapid_incident() {
        // 5 minutes between entries, default cooloff = 15.
        let events = vec![
            ev("1", 0, -100.0, 1000.0),
            ev("2", 5, -100.0, 1000.0),
            ev("3", 10, -100.0, 1000.0),
        ];
        let out = scan(&events, &TiltConfig::default());
        assert!(out.iter().any(|i| i.kind == IncidentKind::RapidReentry));
    }

    #[test]
    fn winner_after_losers_ends_the_streak() {
        let events = vec![
            ev("1", 0, -100.0, 1000.0),
            ev("2", 60, -100.0, 1000.0),
            ev("3", 120, -100.0, 1000.0), // streak of 3 ends here
            ev("4", 180, 200.0, 1000.0),  // winner breaks it
            ev("5", 240, -100.0, 1000.0), // new streak of 1 — below threshold
        ];
        let out = scan(&events, &TiltConfig::default());
        let streaks: Vec<_> = out
            .iter()
            .filter(|i| i.kind == IncidentKind::ConsecutiveLosses)
            .collect();
        assert_eq!(streaks.len(), 1);
        assert_eq!(streaks[0].trade_count, 3);
    }

    #[test]
    fn final_streak_running_to_end_of_input_still_emits() {
        // No winner at end — streak runs to last event.
        let events = vec![
            ev("1", 0, -100.0, 1000.0),
            ev("2", 60, -100.0, 1000.0),
            ev("3", 120, -100.0, 1000.0),
        ];
        let out = scan(&events, &TiltConfig::default());
        assert!(!out.is_empty());
    }

    #[test]
    fn breakeven_zero_pnl_not_a_loss() {
        // pnl == 0 is NOT a loss (no incident).
        let events = vec![
            ev("1", 0, 0.0, 1000.0),
            ev("2", 60, 0.0, 1000.0),
            ev("3", 120, 0.0, 1000.0),
        ];
        let out = scan(&events, &TiltConfig::default());
        assert!(out.is_empty());
    }
}
