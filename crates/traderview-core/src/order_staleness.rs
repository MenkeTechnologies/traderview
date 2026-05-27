//! Order-staleness detector.
//!
//! Flags resting orders that haven't filled within a configurable
//! window. Stale orders are often forgotten — they sit in the book
//! and accidentally fire when price comes back (e.g. a stop you
//! meant to cancel after exiting).
//!
//! Pure compute.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestingOrder {
    pub order_id: String,
    pub symbol: String,
    pub placed_at: DateTime<Utc>,
    pub last_modified_at: Option<DateTime<Utc>>,
    pub side: String,    // "buy", "sell", "buy_stop", "sell_stop"
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StaleTier {
    Fresh,        // < warn threshold
    Aging,        // warn to stale
    Stale,        // stale to forgotten
    Forgotten,    // > forgotten threshold
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaleOrderRow {
    pub order_id: String,
    pub symbol: String,
    pub age_hours: f64,
    pub tier: StaleTier,
}

#[derive(Debug, Clone, Copy)]
pub struct StaleThresholds {
    pub warn_hours: f64,
    pub stale_hours: f64,
    pub forgotten_hours: f64,
}

impl Default for StaleThresholds {
    fn default() -> Self {
        Self { warn_hours: 24.0, stale_hours: 72.0, forgotten_hours: 168.0 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StaleReport {
    pub rows: Vec<StaleOrderRow>,
    pub fresh_count: usize,
    pub aging_count: usize,
    pub stale_count: usize,
    pub forgotten_count: usize,
}

pub fn evaluate(orders: &[RestingOrder], now: DateTime<Utc>, thresh: &StaleThresholds)
    -> StaleReport
{
    let mut report = StaleReport::default();
    for o in orders {
        // Use most-recent-touch (last_modified, or placed_at) as the
        // freshness clock — modifying counts as "re-confirming".
        let touched = o.last_modified_at.unwrap_or(o.placed_at);
        let age = (now - touched).num_seconds() as f64 / 3600.0;
        let tier = if age < thresh.warn_hours { StaleTier::Fresh }
            else if age < thresh.stale_hours { StaleTier::Aging }
            else if age < thresh.forgotten_hours { StaleTier::Stale }
            else { StaleTier::Forgotten };
        match tier {
            StaleTier::Fresh     => report.fresh_count += 1,
            StaleTier::Aging     => report.aging_count += 1,
            StaleTier::Stale     => report.stale_count += 1,
            StaleTier::Forgotten => report.forgotten_count += 1,
        }
        report.rows.push(StaleOrderRow {
            order_id: o.order_id.clone(),
            symbol: o.symbol.clone(),
            age_hours: age,
            tier,
        });
    }
    // Most stale first.
    report.rows.sort_by(|a, b| b.age_hours.partial_cmp(&a.age_hours)
        .unwrap_or(std::cmp::Ordering::Equal));
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn at(h: i64) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 5, 27, 0, 0, 0).unwrap() + Duration::hours(h)
    }
    fn ord(id: &str, placed_h: i64) -> RestingOrder {
        RestingOrder {
            order_id: id.into(),
            symbol: "AAPL".into(),
            placed_at: at(placed_h),
            last_modified_at: None,
            side: "buy".into(),
        }
    }

    #[test]
    fn empty_returns_default() {
        let r = evaluate(&[], at(0), &StaleThresholds::default());
        assert!(r.rows.is_empty());
    }

    #[test]
    fn under_24h_fresh() {
        // Placed 1h ago.
        let orders = vec![ord("o1", -1)];
        let r = evaluate(&orders, at(0), &StaleThresholds::default());
        assert_eq!(r.fresh_count, 1);
        assert_eq!(r.rows[0].tier, StaleTier::Fresh);
    }

    #[test]
    fn between_24_and_72h_aging() {
        // 48 hours old.
        let orders = vec![ord("o1", -48)];
        let r = evaluate(&orders, at(0), &StaleThresholds::default());
        assert_eq!(r.rows[0].tier, StaleTier::Aging);
        assert_eq!(r.aging_count, 1);
    }

    #[test]
    fn between_72_and_168h_stale() {
        let orders = vec![ord("o1", -100)];
        let r = evaluate(&orders, at(0), &StaleThresholds::default());
        assert_eq!(r.rows[0].tier, StaleTier::Stale);
    }

    #[test]
    fn over_168h_forgotten() {
        let orders = vec![ord("o1", -200)];
        let r = evaluate(&orders, at(0), &StaleThresholds::default());
        assert_eq!(r.rows[0].tier, StaleTier::Forgotten);
        assert_eq!(r.forgotten_count, 1);
    }

    #[test]
    fn modification_resets_freshness_clock() {
        let mut o = ord("o1", -200);
        o.last_modified_at = Some(at(-2));    // touched 2h ago
        let r = evaluate(&[o], at(0), &StaleThresholds::default());
        assert_eq!(r.rows[0].tier, StaleTier::Fresh);
    }

    #[test]
    fn rows_sorted_oldest_first() {
        let orders = vec![
            ord("young", -5),
            ord("oldest", -100),
            ord("medium", -50),
        ];
        let r = evaluate(&orders, at(0), &StaleThresholds::default());
        assert_eq!(r.rows[0].order_id, "oldest");
        assert_eq!(r.rows[2].order_id, "young");
    }

    #[test]
    fn custom_thresholds_change_classification() {
        let lax = StaleThresholds { warn_hours: 200.0, stale_hours: 400.0, forgotten_hours: 1000.0 };
        // Place 100h ago — would normally be Stale, but with lax thresholds → Fresh.
        let orders = vec![ord("o1", -100)];
        let r = evaluate(&orders, at(0), &lax);
        assert_eq!(r.rows[0].tier, StaleTier::Fresh);
    }
}
