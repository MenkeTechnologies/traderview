//! Implied-vol term-structure scanner.
//!
//! A symbol's option chain spans many expirations; each has its own
//! ATM IV. The shape of `IV(dte)` is informative:
//!
//!   * **Contango** — IV rises with DTE. The normal regime: longer
//!     horizon carries more uncertainty.
//!   * **Backwardation / inversion** — short-term IV >> long-term IV.
//!     This signals an event-driven mispricing (earnings, FDA event,
//!     litigation date, etc.) where the market has priced one near
//!     expiration *very* rich relative to the underlying baseline.
//!
//! The classic structural trade is a **calendar spread**: short the
//! over-priced front-month, long the relatively-underpriced back
//! month. P&L crystallises when the front-month IV decays back to
//! the long-term level.
//!
//! This module:
//!
//!   1. For each watchlist / top-active symbol, fetches the chain
//!      index (one call) to get the list of expirations.
//!   2. For each expiration (capped at `MAX_EXPIRATIONS` to stay
//!      under Yahoo's tolerance), fetches the chain at that exp and
//!      pulls the ATM call's implied vol.
//!   3. Builds a term-structure curve and computes:
//!        - `slope_pct_per_day` = OLS slope of IV (in pp) vs DTE.
//!          Positive = contango. Negative = backwardation.
//!        - `inversion_score` = max(short_avg_iv) − min(back_avg_iv)
//!          across the front-2 and back-2 expirations. Positive
//!          means the short end is elevated above the back end.
//!        - `recommendation` = "calendar_long_back_short_front"
//!          when inversion_score ≥ `INVERSION_THRESHOLD_PP`,
//!          else "no_edge".
//!
//! Cadence is 4-hourly background refresh (300 calls per round across
//! 30 symbols × 10 exps; ~75s with 250ms pacing — well inside hourly
//! budgets but ~4× the load of a single-expiration poll).

use chrono::{DateTime, NaiveDate, Utc};
use dashmap::DashMap;
use serde::Serialize;
use std::sync::Arc;
use std::time::Duration as StdDuration;

use crate::live_ticks::LiveTickStore;
use crate::options;

const TOP_N: usize = 30;
const REFRESH_SECS: u64 = 4 * 60 * 60;
const PACE_MS: u64 = 250;
const MAX_EXPIRATIONS: usize = 10;
/// Inversion magnitude (in volatility percentage points) required to
/// flag a calendar-spread candidate. 5pp = ~5% absolute IV difference,
/// e.g. front-month at 50% IV vs back at 45%. Below this the
/// difference is in the noise of bid-ask spread.
const INVERSION_THRESHOLD_PP: f64 = 0.05;

#[derive(Debug, Clone, Serialize)]
pub struct TermStructurePoint {
    pub expiration: NaiveDate,
    pub days_to_expiry: i64,
    pub atm_iv: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct TermStructure {
    pub symbol: String,
    pub points: Vec<TermStructurePoint>,
    /// OLS slope of IV vs DTE. Positive = contango, negative =
    /// backwardation.
    pub slope_pct_per_day: f64,
    /// `max(short_avg) - min(back_avg)` across the front-2 and back-2
    /// expirations. Positive means short end is elevated → inversion.
    pub inversion_score: f64,
    pub front_avg_iv: f64,
    pub back_avg_iv: f64,
    pub recommendation: &'static str,
    pub observed_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct IvTermStore {
    rows: Arc<DashMap<String, TermStructure>>,
}

impl IvTermStore {
    pub fn new() -> Self {
        Self {
            rows: Arc::new(DashMap::new()),
        }
    }

    pub fn upsert(&self, t: TermStructure) {
        self.rows.insert(t.symbol.clone(), t);
    }

    /// Ranked by inversion_score descending (most-inverted first, i.e.
    /// the strongest calendar-spread candidates).
    pub fn ranked(&self, limit: usize) -> Vec<TermStructure> {
        let mut rows: Vec<TermStructure> = self.rows.iter().map(|e| e.value().clone()).collect();
        rows.sort_by(|a, b| {
            b.inversion_score
                .partial_cmp(&a.inversion_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        rows.truncate(limit);
        rows
    }

    pub fn get(&self, symbol: &str) -> Option<TermStructure> {
        self.rows
            .get(&symbol.to_ascii_uppercase())
            .map(|e| e.value().clone())
    }
}

impl Default for IvTermStore {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Pure compute ──────────────────────────────────────────────────────────

/// OLS slope of `y` on `x`, returning slope only. `None` when x has
/// zero variance.
pub fn ols_slope(x: &[f64], y: &[f64]) -> Option<f64> {
    if x.len() != y.len() || x.is_empty() {
        return None;
    }
    let n = x.len() as f64;
    let mean_x = x.iter().sum::<f64>() / n;
    let mean_y = y.iter().sum::<f64>() / n;
    let mut num = 0.0;
    let mut den = 0.0;
    for i in 0..x.len() {
        let dx = x[i] - mean_x;
        num += dx * (y[i] - mean_y);
        den += dx * dx;
    }
    if den.abs() < 1e-12 {
        return None;
    }
    Some(num / den)
}

/// Build the full term-structure row from per-expiration ATM-IV points.
/// `points` must be sorted ascending by `days_to_expiry`.
pub fn build_term_structure(symbol: &str, mut points: Vec<TermStructurePoint>) -> TermStructure {
    points.sort_by_key(|p| p.days_to_expiry);
    let xs: Vec<f64> = points.iter().map(|p| p.days_to_expiry as f64).collect();
    let ys: Vec<f64> = points.iter().map(|p| p.atm_iv).collect();
    let slope = ols_slope(&xs, &ys).unwrap_or(0.0);
    // Front = first two points; Back = last two points. If fewer than
    // 4 points exist, fall back to first/last single point.
    let n = points.len();
    let front_avg_iv = if n >= 2 {
        (points[0].atm_iv + points[1].atm_iv) / 2.0
    } else if n == 1 {
        points[0].atm_iv
    } else {
        0.0
    };
    let back_avg_iv = if n >= 4 {
        (points[n - 1].atm_iv + points[n - 2].atm_iv) / 2.0
    } else if n >= 2 {
        points[n - 1].atm_iv
    } else if n == 1 {
        points[0].atm_iv
    } else {
        0.0
    };
    let inversion_score = front_avg_iv - back_avg_iv;
    let recommendation = if inversion_score >= INVERSION_THRESHOLD_PP {
        "calendar_long_back_short_front"
    } else {
        "no_edge"
    };
    TermStructure {
        symbol: symbol.to_ascii_uppercase(),
        points,
        slope_pct_per_day: slope,
        inversion_score,
        front_avg_iv,
        back_avg_iv,
        recommendation,
        observed_at: Utc::now(),
    }
}

// ─── Background refresh ────────────────────────────────────────────────────

fn is_crypto_like(sym: &str) -> bool {
    let s = sym.to_ascii_uppercase();
    if s.contains('/') {
        return true;
    }
    const BASES: &[&str] = &[
        "BTC", "ETH", "LTC", "BCH", "DOGE", "SOL", "AVAX", "MATIC", "ADA", "DOT", "XRP", "LINK",
        "UNI", "SHIB", "AAVE", "ALGO", "ATOM", "BAT", "COMP", "CRV", "GRT", "MKR", "PAXG", "SUSHI",
        "TRX", "XLM", "XTZ", "YFI", "ZRX",
    ];
    if s.ends_with("USD") && s.len() > 3 {
        let base = &s[..s.len() - 3];
        if BASES.contains(&base) {
            return true;
        }
    }
    false
}

fn top_n_active(ticks: &LiveTickStore, n: usize) -> Vec<String> {
    let mut rows: Vec<(String, u64)> = ticks
        .snapshot()
        .into_iter()
        .filter(|s| !is_crypto_like(&s.symbol))
        .map(|s| (s.symbol, s.trade_count))
        .collect();
    rows.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    rows.into_iter().take(n).map(|(s, _)| s).collect()
}

/// Fetch the term structure for one symbol — one chain-index call to
/// get expirations, then up to MAX_EXPIRATIONS per-expiration chain
/// calls. Returns None when the symbol has no usable IV data.
pub async fn fetch_for(symbol: &str) -> Option<TermStructure> {
    let index = options::chain(symbol, None).await.ok()?;
    let today = Utc::now().date_naive();
    let mut exps: Vec<NaiveDate> = index
        .expirations
        .iter()
        .copied()
        .filter(|d| (*d - today).num_days() > 0)
        .collect();
    exps.sort();
    exps.truncate(MAX_EXPIRATIONS);
    let mut points: Vec<TermStructurePoint> = Vec::new();
    for exp in exps {
        match options::chain(symbol, Some(exp)).await {
            Ok(chain) => {
                if let Some(((call, _cm), (_put, _pm), _atm)) = options::atm_straddle(&chain) {
                    if let Some(iv) = call.implied_vol {
                        if iv > 0.0 && iv.is_finite() {
                            points.push(TermStructurePoint {
                                expiration: exp,
                                days_to_expiry: (exp - today).num_days(),
                                atm_iv: iv,
                            });
                        }
                    }
                }
            }
            Err(e) => {
                tracing::debug!(?e, symbol, %exp, "iv_term: chain fetch failed");
            }
        }
        tokio::time::sleep(StdDuration::from_millis(PACE_MS)).await;
    }
    if points.is_empty() {
        return None;
    }
    Some(build_term_structure(symbol, points))
}

pub fn spawn_refresher(store: IvTermStore, ticks: LiveTickStore) {
    tokio::spawn(async move {
        tokio::time::sleep(StdDuration::from_secs(90)).await;
        let mut interval = tokio::time::interval(StdDuration::from_secs(REFRESH_SECS));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            interval.tick().await;
            let symbols = top_n_active(&ticks, TOP_N);
            for sym in symbols {
                if let Some(t) = fetch_for(&sym).await {
                    store.upsert(t);
                }
            }
        }
    });
}

static STORE: once_cell::sync::OnceCell<IvTermStore> = once_cell::sync::OnceCell::new();

pub fn global() -> IvTermStore {
    STORE
        .get_or_init(|| {
            let s = IvTermStore::new();
            spawn_refresher(s.clone(), crate::live_ticks::global());
            s
        })
        .clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    fn p(dte: i64, iv: f64) -> TermStructurePoint {
        TermStructurePoint {
            expiration: d(2026, 1, 1) + chrono::Duration::days(dte),
            days_to_expiry: dte,
            atm_iv: iv,
        }
    }

    #[test]
    fn ols_slope_recovers_known_value() {
        // y = 0.5 * x — slope should be exactly 0.5.
        let x: Vec<f64> = (0..20).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|xi| 0.5 * xi).collect();
        let s = ols_slope(&x, &y).unwrap();
        assert!((s - 0.5).abs() < 1e-9);
    }

    #[test]
    fn ols_slope_none_when_x_constant() {
        let x = vec![5.0; 10];
        let y: Vec<f64> = (0..10).map(|i| i as f64).collect();
        assert!(ols_slope(&x, &y).is_none());
    }

    #[test]
    fn build_term_structure_flags_inversion() {
        // Front-month IV very high vs back-month — classic event-driven
        // inversion. Slope should be negative.
        let points = vec![
            p(7, 0.80),
            p(14, 0.75),
            p(30, 0.50),
            p(60, 0.45),
            p(90, 0.40),
        ];
        let ts = build_term_structure("TEST", points);
        assert!(
            ts.slope_pct_per_day < 0.0,
            "expected backwardation (negative slope)"
        );
        // Front avg = (0.80 + 0.75) / 2 = 0.775
        // Back avg  = (0.40 + 0.45) / 2 = 0.425
        // Inversion = 0.775 - 0.425 = 0.35
        assert!((ts.front_avg_iv - 0.775).abs() < 1e-9);
        assert!((ts.back_avg_iv - 0.425).abs() < 1e-9);
        assert!((ts.inversion_score - 0.35).abs() < 1e-9);
        assert_eq!(ts.recommendation, "calendar_long_back_short_front");
    }

    #[test]
    fn build_term_structure_no_edge_on_normal_contango() {
        // IV rises with DTE — normal contango, no calendar-spread edge.
        let points = vec![p(7, 0.30), p(30, 0.35), p(60, 0.40), p(90, 0.45)];
        let ts = build_term_structure("TEST", points);
        assert!(ts.slope_pct_per_day > 0.0);
        assert!(ts.inversion_score < INVERSION_THRESHOLD_PP);
        assert_eq!(ts.recommendation, "no_edge");
    }

    #[test]
    fn build_term_structure_handles_few_points() {
        // Only 2 points — front = points[0], back = points[1].
        let ts = build_term_structure("AAA", vec![p(7, 0.60), p(60, 0.30)]);
        assert!((ts.front_avg_iv - 0.45).abs() < 1e-9); // (0.60+0.30)/2 — n>=2 uses first two
        assert!((ts.back_avg_iv - 0.30).abs() < 1e-9); // n<4 uses last single
        assert!((ts.inversion_score - 0.15).abs() < 1e-9);
        assert_eq!(ts.recommendation, "calendar_long_back_short_front");
    }

    #[test]
    fn build_term_structure_single_point_safe() {
        // One point only — front = back = that point. Inversion = 0.
        let ts = build_term_structure("ONE", vec![p(30, 0.40)]);
        assert_eq!(ts.front_avg_iv, 0.40);
        assert_eq!(ts.back_avg_iv, 0.40);
        assert_eq!(ts.inversion_score, 0.0);
        assert_eq!(ts.recommendation, "no_edge");
    }

    #[test]
    fn build_term_structure_sorts_unordered_input() {
        let ts = build_term_structure(
            "SCRAM",
            vec![p(60, 0.40), p(7, 0.80), p(30, 0.50), p(90, 0.35)],
        );
        // After sorting: points = [(7,0.80), (30,0.50), (60,0.40), (90,0.35)]
        assert_eq!(ts.points[0].days_to_expiry, 7);
        assert_eq!(ts.points[3].days_to_expiry, 90);
    }

    #[test]
    fn store_ranked_orders_by_inversion_desc() {
        let store = IvTermStore::new();
        let ts1 = build_term_structure("LOW", vec![p(7, 0.30), p(60, 0.28)]);
        let ts2 = build_term_structure("HIGH", vec![p(7, 0.80), p(60, 0.40)]);
        let ts3 = build_term_structure("MID", vec![p(7, 0.50), p(60, 0.40)]);
        store.upsert(ts1);
        store.upsert(ts2);
        store.upsert(ts3);
        let r = store.ranked(5);
        assert_eq!(r[0].symbol, "HIGH");
        assert_eq!(r[1].symbol, "MID");
        assert_eq!(r[2].symbol, "LOW");
    }
}
