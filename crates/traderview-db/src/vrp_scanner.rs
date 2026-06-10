//! Volatility-risk-premium (VRP) scanner.
//!
//! The volatility risk premium is the single most-documented
//! equity-options edge: option implied vol consistently runs above
//! subsequently-realized vol because sellers are compensated for
//! bearing variance risk. The mean IV/RV ratio across SPX history is
//! roughly 1.2–1.3 (i.e. IV ~25% higher than RV). When the ratio
//! moves much higher than that historical mean for a name, **selling
//! premium** has positive expected value; when it inverts, **buying
//! premium** does.
//!
//! This module periodically:
//!
//!   1. Picks the top-N most-active symbols from `LiveTickStore`.
//!   2. Pulls ~60 trading days of daily bars via `prices::get_bars`
//!      (Yahoo-cached).
//!   3. Computes 20-day close-to-close realized vol (annualised by
//!      √252).
//!   4. Fetches the option chain, picks the expiration closest to
//!      30 days out, takes the ATM call's `implied_vol`.
//!   5. Computes `iv_rv_ratio` and `iv_rv_spread` and persists the
//!      result. The route surfaces a ranked table.
//!
//! No background event broadcast — VRP changes slowly (daily-bar
//! resolution) so polling on view-open is fine. Refresh cadence
//! 60 min in the background warmer keeps the table fresh without
//! hammering Yahoo.

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use rust_decimal::prelude::ToPrimitive;
use serde::Serialize;
use std::sync::Arc;
use std::time::Duration as StdDuration;
use traderview_core::BarInterval;

use crate::live_ticks::LiveTickStore;
use crate::{options, prices};

const TOP_N: usize = 30;
const REFRESH_SECS: u64 = 60 * 60;
const PACE_MS: u64 = 600;
const RV_WINDOW_DAYS: usize = 20;
const TRADING_DAYS_PER_YEAR: f64 = 252.0;
/// Target days-to-expiration for the IV lookup. We pick the available
/// expiration nearest this value.
const TARGET_DTE_DAYS: i64 = 30;

#[derive(Debug, Clone, Serialize)]
pub struct VrpScore {
    pub symbol: String,
    /// Annualised 20-day realized volatility (close-to-close log returns).
    pub realized_vol_20d: f64,
    /// ATM call implied volatility from the chosen expiration.
    pub implied_vol: f64,
    /// Days from `observed_at` to the expiration whose IV we used.
    pub iv_dte_days: i64,
    /// `iv / rv`. Above ~1.3 historically suggests sell-premium edge.
    pub iv_rv_ratio: f64,
    /// `iv - rv` in absolute volatility points.
    pub iv_rv_spread: f64,
    pub spot: f64,
    pub atm_strike: f64,
    pub observed_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct VrpStore {
    rows: Arc<DashMap<String, VrpScore>>,
}

impl VrpStore {
    pub fn new() -> Self {
        Self {
            rows: Arc::new(DashMap::new()),
        }
    }

    pub fn upsert(&self, s: VrpScore) {
        self.rows.insert(s.symbol.clone(), s);
    }

    /// `direction = "sell"` ranks by descending `iv_rv_ratio` (overpriced
    /// premium); `"buy"` ranks by ascending (underpriced, contrarian
    /// long-vol setup).
    pub fn ranked(&self, direction: &str, limit: usize) -> Vec<VrpScore> {
        let mut rows: Vec<VrpScore> = self.rows.iter().map(|e| e.value().clone()).collect();
        match direction {
            "buy" => rows.sort_by(|a, b| {
                a.iv_rv_ratio
                    .partial_cmp(&b.iv_rv_ratio)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }),
            _ => rows.sort_by(|a, b| {
                b.iv_rv_ratio
                    .partial_cmp(&a.iv_rv_ratio)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }),
        }
        rows.truncate(limit);
        rows
    }

    pub fn get(&self, symbol: &str) -> Option<VrpScore> {
        self.rows
            .get(&symbol.to_ascii_uppercase())
            .map(|e| e.value().clone())
    }
}

impl Default for VrpStore {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Pure compute ──────────────────────────────────────────────────────────

/// Annualised close-to-close realized volatility for the last
/// `window` daily bars. Returns `None` if there aren't enough data
/// points or any close is non-positive (log undefined).
pub fn realized_vol(closes: &[f64], window: usize) -> Option<f64> {
    if closes.len() < window + 1 {
        return None;
    }
    let slice = &closes[closes.len() - window - 1..];
    let mut returns: Vec<f64> = Vec::with_capacity(window);
    for w in slice.windows(2) {
        if w[0] <= 0.0 || w[1] <= 0.0 {
            return None;
        }
        returns.push((w[1] / w[0]).ln());
    }
    if returns.is_empty() {
        return None;
    }
    let n = returns.len() as f64;
    // Sample variance (divisor n-1) is the textbook estimator; for
    // 20-sample windows the bias from dividing by n vs n-1 is small
    // either way but n-1 is the convention.
    let mean = returns.iter().sum::<f64>() / n;
    let var = returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (n - 1.0).max(1.0);
    Some(var.sqrt() * TRADING_DAYS_PER_YEAR.sqrt())
}

/// From a chain pick the expiration whose dte is closest to
/// `target_dte`. Returns `(chosen_date, dte_days)` or `None` when
/// the chain has no expirations.
pub fn pick_target_expiration(
    expirations: &[chrono::NaiveDate],
    now: chrono::NaiveDate,
    target_dte: i64,
) -> Option<(chrono::NaiveDate, i64)> {
    expirations
        .iter()
        .filter_map(|d| {
            let dte = (*d - now).num_days();
            if dte <= 0 {
                None
            } else {
                Some((*d, dte))
            }
        })
        .min_by_key(|(_, dte)| (dte - target_dte).abs())
}

/// Compute the score row from inputs. Returns `None` when the divisor
/// (realized vol) is zero or non-finite.
pub fn compute_score(
    symbol: &str,
    closes: &[f64],
    chain: &options::Chain,
    now: chrono::NaiveDate,
) -> Option<VrpScore> {
    let rv = realized_vol(closes, RV_WINDOW_DAYS)?;
    if rv <= 0.0 || !rv.is_finite() {
        return None;
    }
    let (exp_date, dte) = pick_target_expiration(&chain.expirations, now, TARGET_DTE_DAYS)?;
    // Use the chain we already have if it's already at the target
    // expiration, otherwise the caller has to refetch — keep this
    // function pure by accepting whatever chain it was given.
    if chain.expiration != exp_date {
        // We were handed a different expiration; bail rather than
        // silently lie. The caller is expected to fetch the
        // correct expiration's chain.
        return None;
    }
    let (call, _call_mid, atm) = match options::atm_straddle(chain) {
        Some(((call, cm), (_put, _pm), atm)) => (call, cm, atm),
        None => return None,
    };
    let iv = call.implied_vol.unwrap_or(0.0);
    if iv <= 0.0 || !iv.is_finite() {
        return None;
    }
    Some(VrpScore {
        symbol: symbol.to_ascii_uppercase(),
        realized_vol_20d: rv,
        implied_vol: iv,
        iv_dte_days: dte,
        iv_rv_ratio: iv / rv,
        iv_rv_spread: iv - rv,
        spot: chain.spot,
        atm_strike: atm,
        observed_at: Utc::now(),
    })
}

// ─── Repository helpers ────────────────────────────────────────────────────

pub async fn refresh_symbol(
    pool: &sqlx::PgPool,
    store: &VrpStore,
    symbol: &str,
) -> anyhow::Result<Option<VrpScore>> {
    let today = Utc::now().date_naive();
    let from = (today - chrono::Duration::days(90))
        .and_hms_opt(0, 0, 0)
        .map(|n| DateTime::<Utc>::from_naive_utc_and_offset(n, Utc))
        .ok_or_else(|| anyhow::anyhow!("from-time conversion"))?;
    let to = today
        .and_hms_opt(23, 59, 59)
        .map(|n| DateTime::<Utc>::from_naive_utc_and_offset(n, Utc))
        .ok_or_else(|| anyhow::anyhow!("to-time conversion"))?;
    let bars = prices::get_bars(pool, symbol, BarInterval::D1, from, to).await?;
    let closes: Vec<f64> = bars.into_iter().filter_map(|b| b.close.to_f64()).collect();
    if closes.len() <= RV_WINDOW_DAYS {
        return Ok(None);
    }
    let chain_index = options::chain(symbol, None).await?;
    let (exp_date, _dte) =
        match pick_target_expiration(&chain_index.expirations, today, TARGET_DTE_DAYS) {
            Some(v) => v,
            None => return Ok(None),
        };
    let chain = options::chain(symbol, Some(exp_date)).await?;
    let score = compute_score(symbol, &closes, &chain, today);
    if let Some(s) = score.clone() {
        store.upsert(s);
    }
    Ok(score)
}

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

pub fn spawn_refresher(store: VrpStore, ticks: LiveTickStore, pool: sqlx::PgPool) {
    tokio::spawn(async move {
        tokio::time::sleep(StdDuration::from_secs(60)).await;
        let mut interval = tokio::time::interval(StdDuration::from_secs(REFRESH_SECS));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            interval.tick().await;
            let symbols = top_n_active(&ticks, TOP_N);
            for sym in symbols {
                if let Err(e) = refresh_symbol(&pool, &store, &sym).await {
                    tracing::debug!(?e, symbol = %sym, "vrp_scanner refresh failed");
                }
                tokio::time::sleep(StdDuration::from_millis(PACE_MS)).await;
            }
        }
    });
}

static STORE: once_cell::sync::OnceCell<VrpStore> = once_cell::sync::OnceCell::new();

pub fn global(pool: sqlx::PgPool) -> VrpStore {
    STORE
        .get_or_init(|| {
            let s = VrpStore::new();
            spawn_refresher(s.clone(), crate::live_ticks::global(), pool);
            s
        })
        .clone()
}

/// Read-only handle that returns None until `global(pool)` has been
/// called. Used by callers without a pool reference.
pub fn try_global() -> Option<VrpStore> {
    STORE.get().cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> chrono::NaiveDate {
        chrono::NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    #[test]
    fn realized_vol_zero_for_flat_series() {
        let closes: Vec<f64> = vec![100.0; 25];
        let rv = realized_vol(&closes, 20).expect("flat series → 0 vol");
        assert!(rv.abs() < 1e-9, "flat series RV should be ~0, got {rv}");
    }

    #[test]
    fn realized_vol_none_when_window_too_small() {
        let closes: Vec<f64> = vec![100.0; 10];
        assert!(realized_vol(&closes, 20).is_none());
    }

    #[test]
    fn realized_vol_none_on_non_positive_close() {
        let mut closes: Vec<f64> = vec![100.0; 25];
        closes[5] = 0.0;
        assert!(realized_vol(&closes, 20).is_none());
    }

    #[test]
    fn realized_vol_annualises_correctly() {
        // Construct daily returns of exactly 1% (alternating signs so
        // mean ≈ 0, stdev ≈ 0.01). Annualised vol ≈ 0.01 · √252 ≈ 0.1587.
        let mut closes = vec![100.0];
        for i in 0..21 {
            let last = *closes.last().unwrap();
            let factor = if i % 2 == 0 { 1.01 } else { 1.0 / 1.01 };
            closes.push(last * factor);
        }
        let rv = realized_vol(&closes, 20).unwrap();
        // Expect ~0.1587. Allow loose tolerance — alternating returns
        // produce a small mean drift that nudges the result.
        assert!((rv - 0.158_7).abs() < 0.02, "expected ~0.1587, got {rv}");
    }

    #[test]
    fn pick_target_expiration_finds_closest() {
        let exps = vec![
            d(2026, 6, 13),
            d(2026, 7, 5),
            d(2026, 7, 22),
            d(2026, 8, 12),
        ];
        let now = d(2026, 6, 8);
        // Target 30 dte → 2026-07-08 ideal; closest is 2026-07-05 (27 dte) vs 2026-07-22 (44 dte) → 7-05.
        let (chosen, dte) = pick_target_expiration(&exps, now, 30).expect("non-empty");
        assert_eq!(chosen, d(2026, 7, 5));
        assert_eq!(dte, 27);
    }

    #[test]
    fn pick_target_expiration_filters_past_dates() {
        let exps = vec![d(2025, 1, 1), d(2026, 6, 13)];
        let now = d(2026, 6, 8);
        let (chosen, _) = pick_target_expiration(&exps, now, 30).unwrap();
        assert_eq!(chosen, d(2026, 6, 13));
    }

    #[test]
    fn pick_target_expiration_none_when_empty() {
        assert!(pick_target_expiration(&[], d(2026, 6, 8), 30).is_none());
    }

    fn opt(strike: f64, iv: f64) -> options::OptionContract {
        options::OptionContract {
            strike,
            bid: Some(1.0),
            ask: Some(1.1),
            last_price: Some(1.05),
            implied_vol: Some(iv),
            volume: Some(100),
            open_interest: Some(1000),
            in_the_money: false,
        }
    }

    fn chain_with(exp: chrono::NaiveDate) -> options::Chain {
        options::Chain {
            symbol: "TEST".into(),
            spot: 100.0,
            expirations: vec![exp],
            expiration: exp,
            calls: vec![opt(100.0, 0.30), opt(105.0, 0.28)],
            puts: vec![opt(100.0, 0.30), opt(95.0, 0.32)],
        }
    }

    #[test]
    fn compute_score_returns_iv_rv_ratio_and_spread() {
        // Closes: flat → RV ~ 0. compute_score should bail because
        // dividing by 0 produces inf — we filter that.
        let flat: Vec<f64> = vec![100.0; 25];
        let now = d(2026, 6, 8);
        let exp = now + chrono::Duration::days(28);
        let chain = chain_with(exp);
        assert!(compute_score("TEST", &flat, &chain, now).is_none());

        // Construct a series with non-zero RV — alternating ±1%.
        let mut closes = vec![100.0];
        for i in 0..21 {
            let last = *closes.last().unwrap();
            let factor = if i % 2 == 0 { 1.01 } else { 1.0 / 1.01 };
            closes.push(last * factor);
        }
        let score = compute_score("TEST", &closes, &chain, now).expect("non-empty");
        assert_eq!(score.symbol, "TEST");
        assert!(score.realized_vol_20d > 0.0);
        assert!(score.implied_vol > 0.0);
        // IV = 0.30, RV ≈ 0.16, ratio ≈ 1.87
        assert!(
            score.iv_rv_ratio > 1.0,
            "expected IV/RV > 1 for IV 30% vs RV ~16%, got {}",
            score.iv_rv_ratio
        );
        // Spread = IV - RV
        assert!((score.iv_rv_spread - (score.implied_vol - score.realized_vol_20d)).abs() < 1e-9);
    }

    #[test]
    fn compute_score_none_when_chain_expiration_doesnt_match_target() {
        // Chain expiration is 2026-06-15; target 30dte from 2026-06-08
        // is 2026-07-08 — chain doesn't contain that exp.
        let chain = chain_with(d(2026, 6, 15));
        let mut closes = vec![100.0];
        for i in 0..21 {
            let last = *closes.last().unwrap();
            let factor = if i % 2 == 0 { 1.01 } else { 1.0 / 1.01 };
            closes.push(last * factor);
        }
        // pick_target_expiration with only one exp (6-15) will choose it;
        // compute_score checks chain.expiration matches and bails if not.
        // In this synthetic test, both equal 6-15, so the function
        // proceeds. Construct a scenario where chain.expiration differs:
        let mut bad_chain = chain.clone();
        bad_chain.expirations = vec![d(2026, 7, 8)]; // 30 dte target
                                                     // But chain.expiration is still 6-15 (mismatch).
        bad_chain.expiration = d(2026, 6, 15);
        let now = d(2026, 6, 8);
        assert!(compute_score("TEST", &closes, &bad_chain, now).is_none());
    }

    #[test]
    fn store_ranked_sell_orders_descending_by_ratio() {
        let store = VrpStore::new();
        let mk = |sym: &str, ratio: f64| VrpScore {
            symbol: sym.into(),
            realized_vol_20d: 0.20,
            implied_vol: 0.20 * ratio,
            iv_dte_days: 30,
            iv_rv_ratio: ratio,
            iv_rv_spread: 0.20 * (ratio - 1.0),
            spot: 100.0,
            atm_strike: 100.0,
            observed_at: Utc::now(),
        };
        store.upsert(mk("LOW", 0.9));
        store.upsert(mk("MID", 1.3));
        store.upsert(mk("HIGH", 2.0));
        let sell = store.ranked("sell", 5);
        assert_eq!(sell[0].symbol, "HIGH");
        assert_eq!(sell[1].symbol, "MID");
        assert_eq!(sell[2].symbol, "LOW");
        let buy = store.ranked("buy", 5);
        assert_eq!(buy[0].symbol, "LOW");
        assert_eq!(buy[2].symbol, "HIGH");
    }
}
