//! Earnings revision tracker.
//!
//! Analyst consensus EPS estimates revise constantly as companies
//! report results, issue guidance, or get re-rated by sell-side
//! coverage. The *velocity* of those revisions is a well-documented
//! free-data edge:
//!
//!   * Womack 1996 — sell-side recommendation upgrades predict
//!     ~3% abnormal returns over the next 30 days.
//!   * Hong & Kumar — momentum in analyst forecast revisions is a
//!     PEAD cousin: upward revisions cluster forward in time, so
//!     today's revision predicts tomorrow's revision and the
//!     associated price drift.
//!
//! Yahoo's `quoteSummary` exposes `earningsTrend.trend[*].epsTrend`
//! with the current consensus plus snapshots from 7 / 30 / 60 / 90
//! days ago. The revision velocity for a given period (typically the
//! `0q` current-quarter or `+1q` next-quarter row) is:
//!
//!   rev_pct_30d = (current - est_30d) / abs(est_30d) × 100
//!   rev_pct_90d = (current - est_90d) / abs(est_90d) × 100
//!
//! and an *acceleration* flag fires when `rev_30d_pct` ≥ a fraction
//! of `rev_90d_pct` — i.e. revisions are speeding up in the most
//! recent month vs the trailing quarter. Acceleration is the
//! tradeable variant: a stock whose consensus has crept up steadily
//! is mostly priced; a stock where revisions are sharply
//! accelerating in the last 30 days has fresh information not yet
//! diffused.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct RevisionMetrics {
    pub symbol: String,
    /// Yahoo period key — `0q` / `+1q` / `0y` / `+1y` etc.
    pub period: String,
    pub current_estimate: f64,
    pub est_7d_ago: Option<f64>,
    pub est_30d_ago: Option<f64>,
    pub est_60d_ago: Option<f64>,
    pub est_90d_ago: Option<f64>,
    pub rev_pct_30d: Option<f64>,
    pub rev_pct_90d: Option<f64>,
    /// True when the 30-day revision % exceeds the 90-day rate
    /// proportionally — i.e. the trend is speeding up. Defined as
    /// `rev_30d_pct ≥ ACCEL_RATIO × rev_90d_pct` with both sharing
    /// the same sign.
    pub accelerating: bool,
    /// 0–100 composite — combines absolute magnitude of the 90-day
    /// revision with the acceleration flag and a recency multiplier.
    /// Sortable; absolute number is relative-rank only.
    pub score: f64,
}

const ACCEL_RATIO: f64 = 0.4;

/// Pure: compute revision metrics from the four lookback snapshots.
/// `None` when the current estimate isn't finite or all lookbacks
/// are missing (no useful direction to extract).
pub fn compute_metrics(
    symbol: &str,
    period: &str,
    current_estimate: f64,
    est_7d_ago: Option<f64>,
    est_30d_ago: Option<f64>,
    est_60d_ago: Option<f64>,
    est_90d_ago: Option<f64>,
) -> Option<RevisionMetrics> {
    if !current_estimate.is_finite() {
        return None;
    }
    let rev_pct = |old: Option<f64>| -> Option<f64> {
        match old {
            Some(v) if v.is_finite() && v.abs() > 1e-9 => {
                Some((current_estimate - v) / v.abs() * 100.0)
            }
            _ => None,
        }
    };
    let rev_pct_30d = rev_pct(est_30d_ago);
    let rev_pct_90d = rev_pct(est_90d_ago);
    let accelerating = match (rev_pct_30d, rev_pct_90d) {
        (Some(r30), Some(r90)) => {
            r30.signum() == r90.signum() && r30.abs() >= ACCEL_RATIO * r90.abs()
        }
        _ => false,
    };
    if rev_pct_30d.is_none() && rev_pct_90d.is_none() {
        return None;
    }
    // Score: weight 90d magnitude heaviest, add accel bonus, recency
    // bonus from the 30d move. Cap at 100 because anything more is
    // either bad data or an issuer-specific revision (M&A, accounting
    // change) that shouldn't outscore systematic-edge candidates.
    let mut score = 0.0_f64;
    if let Some(r) = rev_pct_90d {
        score += r.abs().min(50.0);
    }
    if let Some(r) = rev_pct_30d {
        score += r.abs().min(30.0);
    }
    if accelerating {
        score += 20.0;
    }
    let score = score.min(100.0);
    Some(RevisionMetrics {
        symbol: symbol.to_ascii_uppercase(),
        period: period.into(),
        current_estimate,
        est_7d_ago,
        est_30d_ago,
        est_60d_ago,
        est_90d_ago,
        rev_pct_30d,
        rev_pct_90d,
        accelerating,
        score,
    })
}

/// Parse Yahoo's `earningsTrend.trend[?].epsTrend` shape — returns one
/// metrics row per period whose `current` field is present. Yahoo's
/// JSON nests numeric fields as `{raw: f64}` envelopes; this helper
/// reads either the envelope or a bare number.
pub fn parse_yahoo_revisions(symbol: &str, body: &serde_json::Value) -> Vec<RevisionMetrics> {
    let trends = match body
        .pointer("/earningsTrend/trend")
        .and_then(|v| v.as_array())
    {
        Some(a) => a,
        None => return Vec::new(),
    };
    let raw = |v: &serde_json::Value| -> Option<f64> {
        v.as_f64().or_else(|| v.get("raw").and_then(|r| r.as_f64()))
    };
    let mut out: Vec<RevisionMetrics> = Vec::new();
    for entry in trends {
        let period = entry
            .get("period")
            .and_then(|v| v.as_str())
            .unwrap_or("?")
            .to_string();
        let eps_trend = match entry.get("epsTrend") {
            Some(v) => v,
            None => continue,
        };
        let current = match eps_trend.get("current").and_then(raw) {
            Some(v) => v,
            None => continue,
        };
        let m = compute_metrics(
            symbol,
            &period,
            current,
            eps_trend.get("7daysAgo").and_then(raw),
            eps_trend.get("30daysAgo").and_then(raw),
            eps_trend.get("60daysAgo").and_then(raw),
            eps_trend.get("90daysAgo").and_then(raw),
        );
        if let Some(m) = m {
            out.push(m);
        }
    }
    out
}

/// Repository: fetch Yahoo quoteSummary for `symbol` and return its
/// `earningsTrend` revision rows. Empty Vec on fetch failure.
pub async fn for_symbol(symbol: &str) -> Vec<RevisionMetrics> {
    let body = match crate::market_data::quote_summary(symbol, &["earningsTrend"]).await {
        Ok(b) => b,
        Err(e) => {
            tracing::debug!(?e, symbol, "earnings_revisions: quoteSummary fetch failed");
            return Vec::new();
        }
    };
    parse_yahoo_revisions(symbol, &body)
}

/// Scan a symbol list. Filters to the front-quarter (`0q`) or
/// next-quarter (`+1q`) row — those are the most-actionable periods —
/// and ranks by composite score descending.
pub async fn scan(symbols: &[String]) -> Vec<RevisionMetrics> {
    let mut rows: Vec<RevisionMetrics> = Vec::new();
    for sym in symbols {
        let metrics = for_symbol(sym).await;
        // Prefer the +1q (next quarter) row if present, else 0q, else
        // anything else. Only one row per symbol surfaces.
        let preferred = metrics
            .iter()
            .find(|m| m.period == "+1q")
            .or_else(|| metrics.iter().find(|m| m.period == "0q"))
            .or_else(|| metrics.first());
        if let Some(p) = preferred {
            rows.push(p.clone());
        }
        tokio::time::sleep(std::time::Duration::from_millis(120)).await;
    }
    rows.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upward_revisions_score_positive() {
        // EPS estimate climbed from 1.00 to 1.20 over 90 days
        // (+20%), with the last 30 days contributing 1.15 → 1.20
        // (+4.3%). 30d/90d ratio = 4.3 / 20 = 0.22 — below the 0.40
        // acceleration threshold, so NOT accelerating.
        let m = compute_metrics(
            "STDY",
            "+1q",
            1.20,
            Some(1.18),
            Some(1.15),
            Some(1.10),
            Some(1.00),
        )
        .unwrap();
        assert!((m.rev_pct_90d.unwrap() - 20.0).abs() < 1e-6);
        assert!(m.rev_pct_30d.unwrap() > 0.0);
        assert!(!m.accelerating);
        assert!(m.score > 0.0);
    }

    #[test]
    fn accelerating_when_30d_dominates() {
        // 90d move +10%, 30d move +8% — 30d is 80% of 90d, well above
        // the 40% threshold → accelerating.
        let m = compute_metrics(
            "ACCEL",
            "+1q",
            1.10,
            Some(1.05),
            Some(1.018),
            Some(1.00),
            Some(1.00),
        )
        .unwrap();
        assert!(
            m.accelerating,
            "rev_30d {:?} rev_90d {:?}",
            m.rev_pct_30d, m.rev_pct_90d
        );
    }

    #[test]
    fn downward_revisions_score_by_abs_magnitude() {
        // Estimate cut from 2.00 to 1.50 — bearish but still a strong
        // signal in absolute terms.
        let m = compute_metrics(
            "CUT",
            "+1q",
            1.50,
            Some(1.70),
            Some(1.80),
            Some(1.90),
            Some(2.00),
        )
        .unwrap();
        assert!(m.rev_pct_90d.unwrap() < 0.0);
        assert!(
            m.score > 0.0,
            "score reflects absolute magnitude, not signed direction"
        );
    }

    #[test]
    fn opposite_signs_never_accelerate() {
        // 30d is positive but 90d is negative — opposing trends are
        // never flagged as "accelerating", even if magnitudes match.
        let m = compute_metrics(
            "FLIP",
            "+1q",
            1.05,
            Some(1.00),
            Some(1.00),
            Some(1.05),
            Some(1.10),
        )
        .unwrap();
        assert!(!m.accelerating);
    }

    #[test]
    fn missing_lookbacks_still_compute_partial() {
        // Only 90d snapshot known; 30d unknown. rev_pct_30d should be
        // None but the row should still emit because we have rev_pct_90d.
        let m = compute_metrics("PART", "0q", 1.30, None, None, None, Some(1.00)).unwrap();
        assert!(m.rev_pct_30d.is_none());
        assert!(m.rev_pct_90d.is_some());
        assert!(!m.accelerating);
    }

    #[test]
    fn all_lookbacks_missing_returns_none() {
        let m = compute_metrics("EMPTY", "0q", 1.30, None, None, None, None);
        assert!(m.is_none());
    }

    #[test]
    fn non_finite_current_returns_none() {
        let m = compute_metrics(
            "BAD",
            "0q",
            f64::NAN,
            Some(1.0),
            Some(1.0),
            Some(1.0),
            Some(1.0),
        );
        assert!(m.is_none());
    }

    #[test]
    fn zero_lookback_skipped_to_avoid_div_zero() {
        // est_30d_ago == 0 would divide by zero — must yield None for
        // rev_pct_30d while still producing the row from rev_pct_90d.
        let m = compute_metrics("ZERO", "0q", 1.00, None, Some(0.0), None, Some(0.50)).unwrap();
        assert!(m.rev_pct_30d.is_none());
        assert!(m.rev_pct_90d.is_some());
    }

    #[test]
    fn score_caps_at_100() {
        // Extreme revision: 1000% upward over 90d. Score should cap.
        let m = compute_metrics(
            "MOON",
            "0q",
            10.0,
            Some(5.0),
            Some(2.0),
            Some(1.5),
            Some(1.0),
        )
        .unwrap();
        assert!(m.score <= 100.0);
    }

    #[test]
    fn parse_yahoo_revisions_extracts_known_periods() {
        let body = serde_json::json!({
            "earningsTrend": {
                "trend": [
                    {
                        "period": "0q",
                        "epsTrend": {
                            "current":     { "raw": 1.20 },
                            "7daysAgo":    { "raw": 1.18 },
                            "30daysAgo":   { "raw": 1.15 },
                            "60daysAgo":   { "raw": 1.10 },
                            "90daysAgo":   { "raw": 1.00 },
                        }
                    },
                    {
                        "period": "+1q",
                        "epsTrend": {
                            "current":   1.50,
                            "30daysAgo": 1.30,
                            "90daysAgo": 1.10,
                        }
                    }
                ]
            }
        });
        let rows = parse_yahoo_revisions("TEST", &body);
        assert_eq!(rows.len(), 2);
        let q0 = rows.iter().find(|r| r.period == "0q").unwrap();
        assert!((q0.rev_pct_90d.unwrap() - 20.0).abs() < 1e-6);
        let q1 = rows.iter().find(|r| r.period == "+1q").unwrap();
        // 1.50 vs 1.10 → +36.36%
        assert!((q1.rev_pct_90d.unwrap() - 36.363_636).abs() < 1e-3);
    }

    #[test]
    fn parse_yahoo_revisions_empty_on_malformed() {
        assert!(parse_yahoo_revisions("X", &serde_json::json!({})).is_empty());
        assert!(parse_yahoo_revisions("X", &serde_json::json!({"earningsTrend": null})).is_empty());
    }

    #[test]
    fn parse_yahoo_revisions_skips_entry_without_current() {
        let body = serde_json::json!({
            "earningsTrend": {
                "trend": [
                    { "period": "0q", "epsTrend": {} }
                ]
            }
        });
        assert!(parse_yahoo_revisions("X", &body).is_empty());
    }
}
