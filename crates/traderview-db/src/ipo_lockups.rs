//! IPO lockup expiration tracker.
//!
//! Every IPO contracts a lockup period — typically 180 calendar days
//! — during which insiders, pre-IPO investors, and employees can't
//! sell shares acquired before the offering. On the expiration day,
//! that supply becomes legally tradeable. Historically the supply
//! pressure causes a measurable price drop in the days before AND
//! the days after expiration (Field & Hanka 2001, Bradley et al.
//! 2001 — among the few academic edges that persist into modern
//! markets because the constraint is mechanical, not behavioural).
//!
//! This module:
//!
//!   1. Pulls Finnhub's `ipo_calendar` over the trailing 200 days so
//!      every still-locked IPO whose lockup expires in the next ~30
//!      days is in scope.
//!   2. Filters to `status == "priced"` (actually IPO'd).
//!   3. Projects `lockup_expires_at = ipo_date + LOCKUP_DAYS` (180
//!      by default — most S-1s use this; the rare 90 / 360 variant
//!      gets a conservative miss).
//!   4. Estimates the unlocked-share supply as `INSIDER_MULTIPLE ×
//!      ipo_share_count` because we don't parse the S-1 lockup
//!      tranches without a paid feed. The default 3× is conservative
//!      relative to the typical 5-8× insider-to-float ratio.
//!   5. Surfaces rows sorted by ascending days-to-expiration so the
//!      next few weeks of supply pressure events surface at the top.
//!
//! Output is stateless — every call re-fetches. Caching is the route
//! layer's job. Refresh cadence in the background warmer is daily;
//! IPO calendars don't move intraday.

use chrono::{Duration, NaiveDate, Utc};
use serde::Serialize;
use serde_json::Value;

const LOOKBACK_DAYS: i64 = 200;
const FORWARD_DAYS: i64 = 60;
const LOCKUP_DAYS: i64 = 180;
/// Conservative multiplier from IPO share count to total locked supply
/// when the actual S-1 tranches aren't parsed.
const INSIDER_MULTIPLE: f64 = 3.0;

#[derive(Debug, Clone, Serialize)]
pub struct LockupExpiry {
    pub symbol: String,
    pub name: String,
    pub exchange: String,
    pub ipo_date: NaiveDate,
    pub lockup_expires_at: NaiveDate,
    pub days_until_expiry: i64,
    pub ipo_share_count: u64,
    /// Estimated total shares becoming tradeable on the lockup expiry
    /// day. `ipo_share_count × INSIDER_MULTIPLE`; conservative.
    pub estimated_unlocked_shares: u64,
    /// `estimated_unlocked_shares / ipo_share_count` — how many
    /// "floats worth" of supply hits the market.
    pub float_multiple_estimate: f64,
    pub ipo_price_range: Option<String>,
    pub total_shares_value_usd: Option<f64>,
}

/// Pure: convert the Finnhub `ipo_calendar` JSON payload into
/// `LockupExpiry` rows whose lockup falls in the next `forward_days`.
/// Past-expiry IPOs (already through their lockup) and ones still
/// `expected` (not priced) are skipped.
pub fn parse_lockups(
    body: &Value,
    today: NaiveDate,
    forward_days: i64,
    lockup_days: i64,
) -> Vec<LockupExpiry> {
    let Some(arr) = body.get("ipoCalendar").and_then(|v| v.as_array()) else {
        return Vec::new();
    };
    let mut out: Vec<LockupExpiry> = Vec::new();
    for entry in arr {
        let symbol = match entry.get("symbol").and_then(|v| v.as_str()) {
            Some(s) if !s.is_empty() => s.to_string(),
            _ => continue,
        };
        let status = entry.get("status").and_then(|v| v.as_str()).unwrap_or("");
        if !matches!(
            status,
            "priced" | "expected" | "" // Some feeds omit the field on priced IPOs.
        ) {
            continue;
        }
        // Only ones that actually IPO'd count for lockup tracking.
        if status == "expected" || status == "withdrawn" || status == "filed" {
            continue;
        }
        let ipo_date = match entry.get("date").and_then(|v| v.as_str()) {
            Some(s) => match NaiveDate::parse_from_str(s, "%Y-%m-%d") {
                Ok(d) => d,
                Err(_) => continue,
            },
            None => continue,
        };
        if ipo_date > today {
            // Future IPO — lockup hasn't even started.
            continue;
        }
        let lockup_expires_at = ipo_date + Duration::days(lockup_days);
        let days_until_expiry = (lockup_expires_at - today).num_days();
        // Skip already-expired and too-far-in-the-future.
        if days_until_expiry < 0 || days_until_expiry > forward_days {
            continue;
        }
        let name = entry
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let exchange = entry
            .get("exchange")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        // numberOfShares: f64 in some feeds, i64 in others.
        let ipo_share_count = entry
            .get("numberOfShares")
            .and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|n| n as f64)))
            .map(|n| n.max(0.0).round() as u64)
            .unwrap_or(0);
        if ipo_share_count == 0 {
            // Unknown share count — useless for impact estimation.
            continue;
        }
        let estimated_unlocked_shares = (ipo_share_count as f64 * INSIDER_MULTIPLE).round() as u64;
        let ipo_price_range = entry
            .get("price")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());
        let total_shares_value_usd = entry
            .get("totalSharesValue")
            .and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|n| n as f64)));
        out.push(LockupExpiry {
            symbol,
            name,
            exchange,
            ipo_date,
            lockup_expires_at,
            days_until_expiry,
            ipo_share_count,
            estimated_unlocked_shares,
            float_multiple_estimate: INSIDER_MULTIPLE,
            ipo_price_range,
            total_shares_value_usd,
        });
    }
    out.sort_by_key(|r| r.days_until_expiry);
    out
}

/// Repository entry point — fetches the calendar and parses it.
pub async fn upcoming() -> anyhow::Result<Vec<LockupExpiry>> {
    let today = Utc::now().date_naive();
    let from = (today - Duration::days(LOOKBACK_DAYS))
        .format("%Y-%m-%d")
        .to_string();
    let to = today.format("%Y-%m-%d").to_string();
    let body = crate::finnhub_rest::ipo_calendar(&from, &to).await?;
    Ok(parse_lockups(&body, today, FORWARD_DAYS, LOCKUP_DAYS))
}

/// Pure: variant of `parse_lockups` that includes historical (already-
/// expired) lockup events so backtest can score forward returns from
/// dates whose price action is already known. Returns every parseable
/// IPO in the body whose projected lockup expiry falls within
/// `lookback_days` of today, regardless of sign.
pub fn parse_historical_lockups(
    body: &Value,
    today: NaiveDate,
    lookback_days: i64,
    lockup_days: i64,
) -> Vec<LockupExpiry> {
    let Some(arr) = body.get("ipoCalendar").and_then(|v| v.as_array()) else {
        return Vec::new();
    };
    let mut out: Vec<LockupExpiry> = Vec::new();
    for entry in arr {
        let symbol = match entry.get("symbol").and_then(|v| v.as_str()) {
            Some(s) if !s.is_empty() => s.to_string(),
            _ => continue,
        };
        let status = entry.get("status").and_then(|v| v.as_str()).unwrap_or("");
        if status == "expected" || status == "withdrawn" || status == "filed" {
            continue;
        }
        let ipo_date = match entry.get("date").and_then(|v| v.as_str()) {
            Some(s) => match NaiveDate::parse_from_str(s, "%Y-%m-%d") {
                Ok(d) => d,
                Err(_) => continue,
            },
            None => continue,
        };
        if ipo_date > today {
            continue;
        }
        let lockup_expires_at = ipo_date + Duration::days(lockup_days);
        let days_since_expiry = (today - lockup_expires_at).num_days();
        // Include only historical events whose price action is already
        // in cached bars — past expiry, within lookback window.
        if !(days_since_expiry >= 0 && days_since_expiry <= lookback_days) {
            continue;
        }
        let name = entry
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let exchange = entry
            .get("exchange")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let ipo_share_count = entry
            .get("numberOfShares")
            .and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|n| n as f64)))
            .map(|n| n.max(0.0).round() as u64)
            .unwrap_or(0);
        if ipo_share_count == 0 {
            continue;
        }
        let estimated_unlocked_shares = (ipo_share_count as f64 * INSIDER_MULTIPLE).round() as u64;
        out.push(LockupExpiry {
            symbol,
            name,
            exchange,
            ipo_date,
            lockup_expires_at,
            days_until_expiry: -days_since_expiry,
            ipo_share_count,
            estimated_unlocked_shares,
            float_multiple_estimate: INSIDER_MULTIPLE,
            ipo_price_range: entry
                .get("price")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string()),
            total_shares_value_usd: entry
                .get("totalSharesValue")
                .and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|n| n as f64))),
        });
    }
    out.sort_by_key(|r| r.lockup_expires_at);
    out
}

/// Historical IPO lockup events for the backtest framework. Pulls the
/// Finnhub `ipo_calendar` over the last `days_back + 180` days so every
/// lockup expiry within the trailing `days_back` is in scope (lockup =
/// 180 days after IPO date, so IPOs from the last days_back + 180 days
/// produce expiries within the trailing days_back).
pub async fn historical(days_back: i64) -> anyhow::Result<Vec<LockupExpiry>> {
    let today = Utc::now().date_naive();
    let total_lookback = days_back + LOCKUP_DAYS;
    let from = (today - Duration::days(total_lookback))
        .format("%Y-%m-%d")
        .to_string();
    let to = today.format("%Y-%m-%d").to_string();
    let body = crate::finnhub_rest::ipo_calendar(&from, &to).await?;
    Ok(parse_historical_lockups(
        &body,
        today,
        days_back,
        LOCKUP_DAYS,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn today() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 6, 9).unwrap()
    }

    #[test]
    fn parse_historical_includes_already_expired_within_window() {
        let today = today();
        // A: lockup expired 30 days ago (within 60d window) → INCLUDED
        // B: lockup expired 100 days ago (outside 60d window) → EXCLUDED
        // C: lockup in the future → EXCLUDED (use parse_lockups for those)
        let body = json!({
            "ipoCalendar": [
                {
                    "symbol": "AAA",
                    "name": "Past30",
                    "exchange": "NASDAQ",
                    "date": (today - chrono::Duration::days(210)).format("%Y-%m-%d").to_string(),
                    "status": "priced",
                    "numberOfShares": 5_000_000_u64,
                    "price": "10.00",
                },
                {
                    "symbol": "BBB",
                    "name": "Past100",
                    "exchange": "NYSE",
                    "date": (today - chrono::Duration::days(280)).format("%Y-%m-%d").to_string(),
                    "status": "priced",
                    "numberOfShares": 3_000_000_u64,
                    "price": "20.00",
                },
                {
                    "symbol": "CCC",
                    "name": "FutureExpiry",
                    "exchange": "NASDAQ",
                    "date": (today - chrono::Duration::days(60)).format("%Y-%m-%d").to_string(),
                    "status": "priced",
                    "numberOfShares": 2_000_000_u64,
                    "price": "15.00",
                },
            ]
        });
        let r = parse_historical_lockups(&body, today, 60, LOCKUP_DAYS);
        let syms: Vec<&str> = r.iter().map(|x| x.symbol.as_str()).collect();
        assert_eq!(
            syms,
            vec!["AAA"],
            "only AAA falls in the 60d historical window"
        );
    }

    #[test]
    fn parse_historical_skips_unknown_share_count() {
        let today = today();
        let body = json!({
            "ipoCalendar": [{
                "symbol": "UNK",
                "name": "Unknown",
                "exchange": "",
                "date": (today - chrono::Duration::days(210)).format("%Y-%m-%d").to_string(),
                "status": "priced",
                "numberOfShares": 0_u64,
                "price": "10.00",
            }]
        });
        let r = parse_historical_lockups(&body, today, 60, LOCKUP_DAYS);
        assert!(r.is_empty(), "zero share count must be skipped");
    }

    #[test]
    fn parse_historical_sorts_by_lockup_expiry() {
        let today = today();
        // Three IPOs expiring 10, 30, 50 days ago — should sort earliest first.
        let body = json!({
            "ipoCalendar": [
                {
                    "symbol": "MID",
                    "name": "Mid",
                    "exchange": "",
                    "date": (today - chrono::Duration::days(210)).format("%Y-%m-%d").to_string(),
                    "status": "priced",
                    "numberOfShares": 1_000_000_u64,
                },
                {
                    "symbol": "LATE",
                    "name": "Late",
                    "exchange": "",
                    "date": (today - chrono::Duration::days(190)).format("%Y-%m-%d").to_string(),
                    "status": "priced",
                    "numberOfShares": 1_000_000_u64,
                },
                {
                    "symbol": "EARLY",
                    "name": "Early",
                    "exchange": "",
                    "date": (today - chrono::Duration::days(230)).format("%Y-%m-%d").to_string(),
                    "status": "priced",
                    "numberOfShares": 1_000_000_u64,
                },
            ]
        });
        let r = parse_historical_lockups(&body, today, 60, LOCKUP_DAYS);
        let syms: Vec<&str> = r.iter().map(|x| x.symbol.as_str()).collect();
        assert_eq!(syms, vec!["EARLY", "MID", "LATE"], "earliest expiry first");
    }

    #[test]
    fn parse_filters_to_upcoming_expirations_only() {
        let today = today();
        // Three IPOs:
        //   A: 100 days ago — lockup expires today + 80 days. Past forward window (60d).
        //   B: 160 days ago — lockup expires today + 20 days. IN scope.
        //   C: 200 days ago — lockup expired 20 days ago. PAST scope.
        let body = json!({
            "ipoCalendar": [
                {
                    "symbol": "AAA",
                    "name": "Acme",
                    "exchange": "NASDAQ",
                    "date": (today - chrono::Duration::days(100)).format("%Y-%m-%d").to_string(),
                    "status": "priced",
                    "numberOfShares": 5_000_000_u64,
                    "price": "10.00-12.00",
                    "totalSharesValue": 60_000_000_u64,
                },
                {
                    "symbol": "BBB",
                    "name": "Beta",
                    "exchange": "NYSE",
                    "date": (today - chrono::Duration::days(160)).format("%Y-%m-%d").to_string(),
                    "status": "priced",
                    "numberOfShares": 3_000_000_u64,
                    "price": "20.00",
                    "totalSharesValue": 60_000_000_u64,
                },
                {
                    "symbol": "CCC",
                    "name": "Gamma",
                    "exchange": "NASDAQ",
                    "date": (today - chrono::Duration::days(200)).format("%Y-%m-%d").to_string(),
                    "status": "priced",
                    "numberOfShares": 4_000_000_u64,
                    "price": "15.00",
                    "totalSharesValue": 60_000_000_u64,
                }
            ]
        });
        let rows = parse_lockups(&body, today, 60, 180);
        assert_eq!(
            rows.len(),
            1,
            "only BBB should pass the forward-window filter"
        );
        assert_eq!(rows[0].symbol, "BBB");
        assert_eq!(rows[0].days_until_expiry, 20);
        assert_eq!(rows[0].ipo_share_count, 3_000_000);
        assert_eq!(rows[0].estimated_unlocked_shares, 9_000_000); // 3× multiple
    }

    #[test]
    fn parse_skips_expected_and_withdrawn() {
        let today = today();
        let date = (today - chrono::Duration::days(160))
            .format("%Y-%m-%d")
            .to_string();
        let body = json!({
            "ipoCalendar": [
                { "symbol": "AAA", "date": date, "status": "expected",
                  "numberOfShares": 1_000_000_u64, "name": "x", "exchange": "x" },
                { "symbol": "BBB", "date": date, "status": "withdrawn",
                  "numberOfShares": 1_000_000_u64, "name": "x", "exchange": "x" },
                { "symbol": "CCC", "date": date, "status": "filed",
                  "numberOfShares": 1_000_000_u64, "name": "x", "exchange": "x" }
            ]
        });
        assert!(parse_lockups(&body, today, 60, 180).is_empty());
    }

    #[test]
    fn parse_skips_zero_share_count() {
        let today = today();
        let date = (today - chrono::Duration::days(160))
            .format("%Y-%m-%d")
            .to_string();
        let body = json!({
            "ipoCalendar": [
                { "symbol": "ZERO", "date": date, "status": "priced",
                  "numberOfShares": 0, "name": "x", "exchange": "x" }
            ]
        });
        assert!(parse_lockups(&body, today, 60, 180).is_empty());
    }

    #[test]
    fn parse_handles_missing_optional_fields() {
        let today = today();
        let date = (today - chrono::Duration::days(160))
            .format("%Y-%m-%d")
            .to_string();
        let body = json!({
            "ipoCalendar": [
                { "symbol": "MIN", "date": date, "status": "priced",
                  "numberOfShares": 1_000_000_u64 }
            ]
        });
        let rows = parse_lockups(&body, today, 60, 180);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].name, "");
        assert!(rows[0].ipo_price_range.is_none());
        assert!(rows[0].total_shares_value_usd.is_none());
    }

    #[test]
    fn parse_sorts_by_ascending_days_to_expiry() {
        let today = today();
        let mk = |sym: &str, days_back: i64| -> Value {
            json!({
                "symbol": sym,
                "date": (today - chrono::Duration::days(days_back)).format("%Y-%m-%d").to_string(),
                "status": "priced",
                "numberOfShares": 1_000_000_u64,
                "name": sym,
                "exchange": "NASDAQ",
            })
        };
        let body = json!({
            "ipoCalendar": [
                mk("LATE", 130),   // expires in 50d
                mk("SOON", 175),   // expires in 5d
                mk("MID",  150),   // expires in 30d
            ]
        });
        let rows = parse_lockups(&body, today, 60, 180);
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].symbol, "SOON");
        assert_eq!(rows[1].symbol, "MID");
        assert_eq!(rows[2].symbol, "LATE");
    }

    #[test]
    fn parse_skips_future_ipo_dates() {
        let today = today();
        let future = (today + chrono::Duration::days(5))
            .format("%Y-%m-%d")
            .to_string();
        let body = json!({
            "ipoCalendar": [
                { "symbol": "FUT", "date": future, "status": "priced",
                  "numberOfShares": 1_000_000_u64, "name": "x", "exchange": "x" }
            ]
        });
        assert!(parse_lockups(&body, today, 60, 180).is_empty());
    }

    #[test]
    fn parse_handles_share_count_as_f64() {
        let today = today();
        let date = (today - chrono::Duration::days(160))
            .format("%Y-%m-%d")
            .to_string();
        let body = json!({
            "ipoCalendar": [
                { "symbol": "FLT", "date": date, "status": "priced",
                  "numberOfShares": 1_500_000.0_f64, "name": "x", "exchange": "x" }
            ]
        });
        let rows = parse_lockups(&body, today, 60, 180);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].ipo_share_count, 1_500_000);
    }

    #[test]
    fn parse_handles_empty_or_malformed_payload() {
        let today = today();
        // No `ipoCalendar` key at all.
        assert!(parse_lockups(&json!({}), today, 60, 180).is_empty());
        // Wrong shape.
        assert!(parse_lockups(&json!({"ipoCalendar": "not-an-array"}), today, 60, 180).is_empty());
        // Empty array.
        assert!(parse_lockups(&json!({"ipoCalendar": []}), today, 60, 180).is_empty());
    }
}
