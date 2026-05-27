//! News-driven auto-resize policy.
//!
//! When a high-impact news event hits (FOMC, NFP, CPI, earnings), the
//! prudent default is to TRIM positions to reduce overnight gap risk.
//! This module emits a per-position resize recommendation.
//!
//! Pure compute. The trader supplies the position list + active event
//! list + impact severity; engine emits actions.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventImpact {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsEvent {
    pub event_name: String,
    pub impact: EventImpact,
    /// Symbols this event is expected to affect; empty = market-wide.
    pub affected_symbols: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenPosition {
    pub symbol: String,
    pub current_qty: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResizeAction {
    pub symbol: String,
    pub current_qty: f64,
    pub recommended_qty: f64,
    pub trim_amount: f64,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NewsActionReport {
    pub actions: Vec<ResizeAction>,
}

pub fn evaluate(positions: &[OpenPosition], events: &[NewsEvent]) -> NewsActionReport {
    let mut report = NewsActionReport::default();
    for pos in positions {
        // Find the highest-impact event affecting this position.
        let relevant: Vec<_> = events
            .iter()
            .filter(|e| {
                e.affected_symbols.is_empty() || e.affected_symbols.iter().any(|s| s == &pos.symbol)
            })
            .collect();
        let max_impact = relevant.iter().map(|e| e.impact).max();
        if let Some(impact) = max_impact {
            let trim_pct = match impact {
                EventImpact::Low => 0.0,
                EventImpact::Medium => 0.25,
                EventImpact::High => 0.50,
                EventImpact::Critical => 1.00,
            };
            if trim_pct == 0.0 {
                continue;
            }
            let new_qty = pos.current_qty * (1.0 - trim_pct);
            let trim = pos.current_qty - new_qty;
            let event_names: Vec<&str> = relevant.iter().map(|e| e.event_name.as_str()).collect();
            report.actions.push(ResizeAction {
                symbol: pos.symbol.clone(),
                current_qty: pos.current_qty,
                recommended_qty: new_qty,
                trim_amount: trim,
                reason: format!(
                    "trim {}% due to {:?} impact event(s): {}",
                    (trim_pct * 100.0) as i32,
                    impact,
                    event_names.join(", ")
                ),
            });
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pos(sym: &str, qty: f64) -> OpenPosition {
        OpenPosition {
            symbol: sym.into(),
            current_qty: qty,
        }
    }
    fn event(name: &str, impact: EventImpact, affected: &[&str]) -> NewsEvent {
        NewsEvent {
            event_name: name.into(),
            impact,
            affected_symbols: affected.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn no_events_no_actions() {
        let r = evaluate(&[pos("AAPL", 100.0)], &[]);
        assert!(r.actions.is_empty());
    }

    #[test]
    fn low_impact_event_no_trim() {
        let r = evaluate(
            &[pos("AAPL", 100.0)],
            &[event("Fed minutes", EventImpact::Low, &[])],
        );
        assert!(r.actions.is_empty());
    }

    #[test]
    fn medium_impact_trim_25pct() {
        let r = evaluate(
            &[pos("AAPL", 100.0)],
            &[event("Retail sales", EventImpact::Medium, &[])],
        );
        assert_eq!(r.actions.len(), 1);
        assert_eq!(r.actions[0].recommended_qty, 75.0);
        assert_eq!(r.actions[0].trim_amount, 25.0);
    }

    #[test]
    fn high_impact_trim_50pct() {
        let r = evaluate(
            &[pos("AAPL", 100.0)],
            &[event("CPI", EventImpact::High, &[])],
        );
        assert_eq!(r.actions[0].recommended_qty, 50.0);
    }

    #[test]
    fn critical_impact_full_close() {
        let r = evaluate(
            &[pos("AAPL", 100.0)],
            &[event("FOMC", EventImpact::Critical, &[])],
        );
        assert_eq!(r.actions[0].recommended_qty, 0.0);
    }

    #[test]
    fn empty_affected_symbols_means_market_wide() {
        let r = evaluate(
            &[pos("AAPL", 100.0), pos("TSLA", 50.0)],
            &[event("FOMC", EventImpact::Critical, &[])],
        );
        assert_eq!(r.actions.len(), 2);
    }

    #[test]
    fn symbol_specific_event_only_affects_listed_symbols() {
        let r = evaluate(
            &[pos("AAPL", 100.0), pos("TSLA", 50.0)],
            &[event("AAPL earnings", EventImpact::Critical, &["AAPL"])],
        );
        assert_eq!(r.actions.len(), 1);
        assert_eq!(r.actions[0].symbol, "AAPL");
    }

    #[test]
    fn highest_impact_among_multiple_events_used() {
        // Two events: Medium + Critical → Critical wins → full close.
        let events = vec![
            event("Retail sales", EventImpact::Medium, &[]),
            event("FOMC", EventImpact::Critical, &[]),
        ];
        let r = evaluate(&[pos("AAPL", 100.0)], &events);
        assert_eq!(r.actions[0].recommended_qty, 0.0);
    }
}
