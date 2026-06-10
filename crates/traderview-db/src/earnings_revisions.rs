//! Earnings / analyst-sentiment revision tracker.
//!
//! Sell-side analyst sentiment revises constantly as companies report
//! results, issue guidance, or get re-rated by coverage. The *velocity*
//! of those revisions is a well-documented free-data edge:
//!
//!   * Womack 1996 — sell-side recommendation upgrades predict
//!     ~3% abnormal returns over the next 30 days. THIS is the signal
//!     this module targets directly.
//!   * Hong & Kumar — momentum in analyst forecast revisions is a
//!     PEAD cousin: upward revisions cluster forward in time, so
//!     today's revision predicts tomorrow's and the associated drift.
//!
//! Data source: Finnhub `/stock/recommendation` — returns monthly
//! snapshots of `{strong_buy, buy, hold, sell, strong_sell}` counts
//! per analyst. We collapse each month into a single sentiment score:
//!
//!   sentiment = (2·SB + 1·B + 0·H + (-1)·S + (-2)·SS) / total_analysts
//!
//! Range: −2 (universal strong_sell) → +2 (universal strong_buy).
//! Period code stored as `YYYY-MM` of the snapshot.
//!
//! Revision velocity, taking the latest month as `current` and
//! earlier months as the lookback snapshots:
//!
//!   rev_pct_30d = (current − sentiment_1mo_ago) / |1mo_ago| × 100
//!   rev_pct_90d = (current − sentiment_3mo_ago) / |3mo_ago| × 100
//!
//! and an *acceleration* flag fires when `rev_30d_pct` ≥ a fraction
//! of `rev_90d_pct` with the same sign — sentiment is speeding up in
//! the most recent month vs the trailing quarter. Acceleration is
//! the tradeable variant: a stock whose ratings have crept up
//! steadily is mostly priced; a stock where the upgrade velocity is
//! sharply accelerating has fresh information not yet diffused.

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

/// Sentiment score from one Finnhub recommendation row. Weighted
/// average of `{strong_buy:+2, buy:+1, hold:0, sell:-1, strong_sell:-2}`
/// across the analyst count. Returns `None` when no analysts cover the
/// stock that month (every count is 0/missing — no signal to derive).
fn sentiment_from_row(row: &serde_json::Value) -> Option<f64> {
    let i = |k: &str| row.get(k).and_then(|v| v.as_i64()).unwrap_or(0);
    let sb = i("strongBuy");
    let b = i("buy");
    let h = i("hold");
    let s = i("sell");
    let ss = i("strongSell");
    let total = sb + b + h + s + ss;
    if total == 0 {
        return None;
    }
    let weighted = (sb * 2 + b - s - ss * 2) as f64;
    Some(weighted / total as f64)
}

/// Parse Finnhub's `/stock/recommendation` array (monthly snapshots) into
/// a `RevisionMetrics` row anchored at the most-recent month. The earlier
/// snapshots in the array become the 1mo / 3mo / 6mo lookbacks (mapped
/// onto the struct's `est_30d_ago` / `est_90d_ago` / `est_60d_ago` slots
/// for naming continuity). Each row carries `period: "YYYY-MM"` of the
/// anchor month so the frontend can label the data.
pub fn parse_finnhub_revisions(symbol: &str, body: &serde_json::Value) -> Vec<RevisionMetrics> {
    let rows = match body.as_array() {
        Some(a) if !a.is_empty() => a,
        _ => return Vec::new(),
    };
    // Finnhub returns newest-first. Take up to ~7 months of history so
    // we have room for 1mo / 3mo / 6mo lookbacks even when an
    // intermediate month was reported with zero coverage.
    let mut scored: Vec<(String, f64)> = rows
        .iter()
        .take(7)
        .filter_map(|r| {
            let p = r.get("period").and_then(|v| v.as_str())?.to_string();
            let s = sentiment_from_row(r)?;
            Some((p, s))
        })
        .collect();
    if scored.is_empty() {
        return Vec::new();
    }
    let (anchor_period, current) = scored.remove(0);
    let pick = |idx: usize| -> Option<f64> { scored.get(idx).map(|(_, v)| *v) };
    let est_30d = pick(0); // ~1mo old
    let est_60d = pick(1); // ~2mo old
    let est_90d = pick(2); // ~3mo old
    let est_7d = None; // Finnhub cadence is monthly — no 7-day snapshot
    if let Some(m) = compute_metrics(symbol, &anchor_period, current, est_7d, est_30d, est_60d, est_90d) {
        vec![m]
    } else {
        Vec::new()
    }
}

/// Repository: fetch Finnhub `/stock/recommendation` for `symbol` and
/// return its computed revision row. Empty Vec on fetch failure or when
/// the symbol has no analyst coverage.
pub async fn for_symbol(symbol: &str) -> Vec<RevisionMetrics> {
    let body = match crate::finnhub_rest::recommendation(symbol).await {
        Ok(b) => b,
        Err(e) => {
            tracing::debug!(?e, symbol, "earnings_revisions: finnhub recommendation fetch failed");
            return Vec::new();
        }
    };
    parse_finnhub_revisions(symbol, &body)
}

/// Scan a symbol list — one row per symbol anchored at the most-recent
/// month, ranked by composite revision-velocity score descending.
pub async fn scan(symbols: &[String]) -> Vec<RevisionMetrics> {
    let mut rows: Vec<RevisionMetrics> = Vec::new();
    for sym in symbols {
        let metrics = for_symbol(sym).await;
        if let Some(p) = metrics.first() {
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
    fn sentiment_weighted_average() {
        // 5 strong_buy + 3 buy + 2 hold + 0 sell + 0 strong_sell = 10
        // weighted = 5·2 + 3·1 + 2·0 + 0 + 0 = 13 → 13/10 = 1.3
        let row = serde_json::json!({
            "strongBuy": 5, "buy": 3, "hold": 2, "sell": 0, "strongSell": 0
        });
        assert!((sentiment_from_row(&row).unwrap() - 1.3).abs() < 1e-9);
    }

    #[test]
    fn sentiment_balanced_negative() {
        // Pure strong_sell → −2
        let row = serde_json::json!({
            "strongBuy": 0, "buy": 0, "hold": 0, "sell": 0, "strongSell": 4
        });
        assert!((sentiment_from_row(&row).unwrap() - (-2.0)).abs() < 1e-9);
    }

    #[test]
    fn sentiment_none_when_no_coverage() {
        let row = serde_json::json!({
            "strongBuy": 0, "buy": 0, "hold": 0, "sell": 0, "strongSell": 0
        });
        assert!(sentiment_from_row(&row).is_none());
    }

    #[test]
    fn parse_finnhub_revisions_extracts_latest_month_anchor() {
        // Finnhub returns newest-first. Latest month: very positive
        // (5SB+5B = 1.5 sentiment), 3 months ago: neutral (5H = 0).
        // Revision = (1.5 − 0) / |0| × 100 → divide-by-zero guard
        // returns None for that lookback; 1mo-ago is 1.0 so 30d works.
        let body = serde_json::json!([
            // 2026-06: sentiment = (5·2 + 5·1) / 10 = 1.5  (anchor)
            { "period": "2026-06-01", "strongBuy": 5, "buy": 5, "hold": 0, "sell": 0, "strongSell": 0 },
            // 2026-05: sentiment = (3·2 + 4·1) / 10 = 1.0   (1mo lookback)
            { "period": "2026-05-01", "strongBuy": 3, "buy": 4, "hold": 3, "sell": 0, "strongSell": 0 },
            // 2026-04: sentiment = (1·2 + 4·1) / 9 ≈ 0.667  (2mo)
            { "period": "2026-04-01", "strongBuy": 1, "buy": 4, "hold": 4, "sell": 0, "strongSell": 0 },
            // 2026-03: sentiment = 0.5                       (3mo lookback)
            { "period": "2026-03-01", "strongBuy": 0, "buy": 5, "hold": 5, "sell": 0, "strongSell": 0 },
        ]);
        let rows = parse_finnhub_revisions("TEST", &body);
        assert_eq!(rows.len(), 1);
        let m = &rows[0];
        assert_eq!(m.period, "2026-06-01");
        assert!((m.current_estimate - 1.5).abs() < 1e-9);
        // 30d (1mo-ago = 1.0): (1.5 − 1.0) / 1.0 × 100 = 50%
        assert!((m.rev_pct_30d.unwrap() - 50.0).abs() < 1e-6);
        // 90d (3mo-ago = 0.5): (1.5 − 0.5) / 0.5 × 100 = 200%
        assert!((m.rev_pct_90d.unwrap() - 200.0).abs() < 1e-6);
        assert!(m.rev_pct_30d.unwrap() > 0.0 && m.rev_pct_90d.unwrap() > 0.0);
    }

    #[test]
    fn parse_finnhub_revisions_empty_on_malformed() {
        assert!(parse_finnhub_revisions("X", &serde_json::json!({})).is_empty());
        assert!(parse_finnhub_revisions("X", &serde_json::json!([])).is_empty());
        // No coverage row → no sentiment → nothing to anchor at.
        assert!(parse_finnhub_revisions("X", &serde_json::json!([
            { "period": "2026-06-01", "strongBuy": 0, "buy": 0, "hold": 0, "sell": 0, "strongSell": 0 }
        ])).is_empty());
    }

    #[test]
    fn parse_finnhub_revisions_single_month_yields_no_lookbacks() {
        // Only one month of coverage → no revision direction → no row.
        let body = serde_json::json!([
            { "period": "2026-06-01", "strongBuy": 5, "buy": 5, "hold": 0, "sell": 0, "strongSell": 0 }
        ]);
        assert!(parse_finnhub_revisions("X", &body).is_empty());
    }
}
