//! Real-time gamma-squeeze candidate detector.
//!
//! Periodically polls the top-N most-active symbols' option chains off
//! Yahoo, computes per-strike dealer-net gamma exposure (GEX) via
//! `traderview_core::gex_scanner`, and emits a `GammaSqueezeCandidate`
//! event when any of these conditions cross:
//!
//!   1. **Negative total GEX with |total| ≥ `MIN_NEGATIVE_GEX`.**
//!      Dealers net-short gamma — every up-move forces them to buy more
//!      shares to re-hedge, amplifying the move. Negative GEX is the
//!      necessary (not sufficient) condition for a real gamma squeeze.
//!
//!   2. **Spot price within `PIN_DISTANCE_PCT` of `largest_negative_strike`.**
//!      The strike concentrating the most short-gamma pressure becomes
//!      the magnet — once spot crosses it, gamma re-hedging accelerates
//!      the trend.
//!
//! Per-contract gamma is computed with inline Black-Scholes using the
//! Yahoo-reported `implied_volatility` and a fixed risk-free rate
//! (`RISK_FREE_RATE`) approximation. r matters less than the OI shape
//! for the relative ranking we surface.
//!
//! Dedup keyed by `(symbol, expiry, largest_negative_strike)` — the
//! same chain re-scanned 60s later won't fire again unless the
//! squeeze-key strike has moved or the polarity flipped.
//!
//! Limitations:
//!   * Yahoo's free chain returns only the front-month strip; we don't
//!     aggregate across expiries. Real dealer GEX spans every expiry,
//!     so the absolute dollar values here under-report. The signal is
//!     useful as a relative ranking and direction indicator.
//!   * `dollar_gamma_per_contract = gamma_per_share · 100 · spot²` —
//!     standard market convention; some sources omit the `· spot²`
//!     factor. Either is fine internally as long as the magnitudes
//!     stay self-consistent.

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::Serialize;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;

use crate::live_ticks::LiveTickStore;
use crate::options;
use traderview_core::gex_scanner::{self, OptionStrike};

/// Symbols polled per round. Smaller than the UOA poller (20) because
/// computing per-strike gamma is heavier than a vol/OI filter.
const TOP_N: usize = 10;
/// Seconds between full rounds. Gamma exposure changes slowly except
/// near expiry, so 90s strikes the balance between freshness and load.
const ROUND_SECS: u64 = 90;
/// Pacing between Yahoo requests inside one round.
const PACE_MS: u64 = 300;
/// Minimum absolute dollar GEX (negative regime) at which a chain
/// counts as a candidate. Sensitive — too low surfaces noise; too high
/// misses small-cap squeezes. $250M front-month is a defensible floor.
const MIN_NEGATIVE_GEX: f64 = 250_000_000.0;
/// Spot must be within this percent of `largest_negative_strike` to
/// flag the chain as squeeze-imminent.
const PIN_DISTANCE_PCT: f64 = 2.0;
/// Risk-free rate used in the BS gamma compute. ~Fed funds; relative
/// magnitudes are insensitive to exact value.
const RISK_FREE_RATE: f64 = 0.045;
/// Cap on emitted candidates kept in memory.
const EMITTED_CAP: usize = 2_000;

#[derive(Debug, Clone, Serialize)]
pub struct GammaSqueezeCandidate {
    pub symbol: String,
    pub expiry: String,
    pub spot: f64,
    pub total_gex: f64,
    pub largest_negative_strike: Option<f64>,
    pub largest_positive_strike: Option<f64>,
    pub zero_gamma_strike: Option<f64>,
    /// % distance from spot to `largest_negative_strike`. Signed: spot
    /// above strike → positive; below → negative.
    pub pin_distance_pct: Option<f64>,
    pub observed_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct GammaSqueezeStore {
    emitted: Arc<DashMap<String, GammaSqueezeCandidate>>,
    tx: broadcast::Sender<GammaSqueezeCandidate>,
}

impl GammaSqueezeStore {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(256);
        Self {
            emitted: Arc::new(DashMap::new()),
            tx,
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<GammaSqueezeCandidate> {
        self.tx.subscribe()
    }

    pub fn latest(&self, limit: usize) -> Vec<GammaSqueezeCandidate> {
        let mut all: Vec<GammaSqueezeCandidate> =
            self.emitted.iter().map(|e| e.value().clone()).collect();
        all.sort_by_key(|c| std::cmp::Reverse(c.observed_at));
        all.truncate(limit);
        all
    }

    pub fn latest_for(&self, symbol: &str, limit: usize) -> Vec<GammaSqueezeCandidate> {
        let sym_upper = symbol.to_ascii_uppercase();
        let mut hits: Vec<GammaSqueezeCandidate> = self
            .emitted
            .iter()
            .filter(|e| e.value().symbol == sym_upper)
            .map(|e| e.value().clone())
            .collect();
        hits.sort_by_key(|c| std::cmp::Reverse(c.observed_at));
        hits.truncate(limit);
        hits
    }

    /// Insert and broadcast if the (symbol, expiry, neg_strike) key
    /// hasn't been seen. Same dedup logic as the UOA stream — repeated
    /// rounds of the same squeeze pose don't spam the feed.
    fn observe(&self, cand: GammaSqueezeCandidate) -> bool {
        let key = format!(
            "{}|{}|{}",
            cand.symbol,
            cand.expiry,
            cand.largest_negative_strike
                .map(|s| s.to_string())
                .unwrap_or_else(|| "?".into())
        );
        if self.emitted.contains_key(&key) {
            return false;
        }
        self.emitted.insert(key, cand.clone());
        let _ = self.tx.send(cand);
        self.evict_if_full();
        true
    }

    fn evict_if_full(&self) {
        if self.emitted.len() <= EMITTED_CAP {
            return;
        }
        let drop_n = self.emitted.len() / 4;
        let mut by_age: Vec<(String, DateTime<Utc>)> = self
            .emitted
            .iter()
            .map(|e| (e.key().clone(), e.value().observed_at))
            .collect();
        by_age.sort_by_key(|(_, t)| *t);
        for (key, _) in by_age.into_iter().take(drop_n) {
            self.emitted.remove(&key);
        }
    }
}

impl Default for GammaSqueezeStore {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Pure compute helpers ──────────────────────────────────────────────────

/// Standard-normal PDF.
fn norm_pdf(x: f64) -> f64 {
    use std::f64::consts::PI;
    (-0.5 * x * x).exp() / (2.0 * PI).sqrt()
}

/// Black-Scholes gamma per share. Returns 0 when inputs aren't finite
/// or the time-to-expiry / vol arguments would produce NaN.
pub fn bs_gamma_per_share(spot: f64, strike: f64, t_years: f64, sigma: f64, r: f64) -> f64 {
    if !spot.is_finite() || spot <= 0.0 {
        return 0.0;
    }
    if !strike.is_finite() || strike <= 0.0 {
        return 0.0;
    }
    if !t_years.is_finite() || t_years <= 0.0 {
        return 0.0;
    }
    if !sigma.is_finite() || sigma <= 0.0 {
        return 0.0;
    }
    let sqrt_t = t_years.sqrt();
    let d1 = ((spot / strike).ln() + (r + 0.5 * sigma * sigma) * t_years) / (sigma * sqrt_t);
    norm_pdf(d1) / (spot * sigma * sqrt_t)
}

/// Dollar-gamma per contract using the standard market convention
/// `gamma_per_share · 100 · spot²` so output is directly usable by
/// `gex_scanner::scan`.
pub fn dollar_gamma_per_contract(spot: f64, strike: f64, t_years: f64, sigma: f64, r: f64) -> f64 {
    bs_gamma_per_share(spot, strike, t_years, sigma, r) * 100.0 * spot * spot
}

/// Build the per-strike OptionStrike rows for `gex_scanner::scan` from
/// a Yahoo chain. Calls and puts are aggregated by strike; if an
/// implied vol is missing or non-positive the per-contract gamma falls
/// back to 0 for that side, which excludes the row's contribution
/// without producing NaN.
pub fn chain_to_strikes(chain: &options::Chain) -> Vec<OptionStrike> {
    use std::collections::BTreeMap;
    let t_years = years_to_expiry(chain.expiration);
    if t_years <= 0.0 {
        return Vec::new();
    }
    let mut by_strike: BTreeMap<i64, OptionStrike> = BTreeMap::new();
    let key = |strike: f64| -> i64 {
        // Cents-precise key so floating strikes (199.5 etc.) collate
        // cleanly. Strikes < 0 are filtered upstream.
        (strike * 100.0).round() as i64
    };
    for c in &chain.calls {
        let entry = by_strike
            .entry(key(c.strike))
            .or_insert_with(|| OptionStrike {
                strike: c.strike,
                call_open_interest: 0.0,
                put_open_interest: 0.0,
                call_gamma_per_contract: 0.0,
                put_gamma_per_contract: 0.0,
            });
        entry.call_open_interest += c.open_interest.unwrap_or(0) as f64;
        let sigma = c.implied_vol.unwrap_or(0.0);
        entry.call_gamma_per_contract =
            dollar_gamma_per_contract(chain.spot, c.strike, t_years, sigma, RISK_FREE_RATE);
    }
    for p in &chain.puts {
        let entry = by_strike
            .entry(key(p.strike))
            .or_insert_with(|| OptionStrike {
                strike: p.strike,
                call_open_interest: 0.0,
                put_open_interest: 0.0,
                call_gamma_per_contract: 0.0,
                put_gamma_per_contract: 0.0,
            });
        entry.put_open_interest += p.open_interest.unwrap_or(0) as f64;
        let sigma = p.implied_vol.unwrap_or(0.0);
        entry.put_gamma_per_contract =
            dollar_gamma_per_contract(chain.spot, p.strike, t_years, sigma, RISK_FREE_RATE);
    }
    by_strike.into_values().collect()
}

fn years_to_expiry(expiry: chrono::NaiveDate) -> f64 {
    let today = Utc::now().date_naive();
    let days = (expiry - today).num_days();
    if days <= 0 {
        return 0.0;
    }
    days as f64 / 365.0
}

/// Run the squeeze rule on a (symbol, chain) pair, returning a
/// candidate row when both conditions cross.
pub fn evaluate_chain(symbol: &str, chain: &options::Chain) -> Option<GammaSqueezeCandidate> {
    if chain.spot <= 0.0 {
        return None;
    }
    let strikes = chain_to_strikes(chain);
    let report = gex_scanner::scan(&strikes)?;
    let neg_strike = report.largest_negative_strike?;
    let pin_distance_pct = (chain.spot - neg_strike) / neg_strike * 100.0;
    let total_negative_enough =
        report.total_gex < 0.0 && report.total_gex.abs() >= MIN_NEGATIVE_GEX;
    let pin_close_enough = pin_distance_pct.abs() <= PIN_DISTANCE_PCT;
    if !(total_negative_enough && pin_close_enough) {
        return None;
    }
    Some(GammaSqueezeCandidate {
        symbol: symbol.to_ascii_uppercase(),
        expiry: chain.expiration.format("%Y-%m-%d").to_string(),
        spot: chain.spot,
        total_gex: report.total_gex,
        largest_negative_strike: report.largest_negative_strike,
        largest_positive_strike: report.largest_positive_strike,
        zero_gamma_strike: report.zero_gamma_strike,
        pin_distance_pct: Some(pin_distance_pct),
        observed_at: Utc::now(),
    })
}

/// Choose the top-N most-active equity symbols (crypto filtered).
pub fn top_n_active(ticks: &LiveTickStore, n: usize) -> Vec<String> {
    let mut rows: Vec<(String, u64)> = ticks
        .snapshot()
        .into_iter()
        .filter(|s| !is_crypto_like(&s.symbol))
        .map(|s| (s.symbol, s.trade_count))
        .collect();
    rows.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    rows.into_iter().take(n).map(|(s, _)| s).collect()
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

pub fn spawn_poller(store: GammaSqueezeStore, ticks: LiveTickStore) {
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(20)).await;
        let mut interval = tokio::time::interval(Duration::from_secs(ROUND_SECS));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            interval.tick().await;
            let symbols = top_n_active(&ticks, TOP_N);
            if symbols.is_empty() {
                continue;
            }
            for sym in symbols {
                match options::chain(&sym, None).await {
                    Ok(chain) => {
                        if let Some(cand) = evaluate_chain(&sym, &chain) {
                            store.observe(cand);
                        }
                    }
                    Err(e) => {
                        tracing::debug!(?e, symbol = %sym, "gamma_squeeze chain fetch failed");
                    }
                }
                tokio::time::sleep(Duration::from_millis(PACE_MS)).await;
            }
        }
    });
}

pub fn global() -> GammaSqueezeStore {
    static STORE: once_cell::sync::OnceCell<GammaSqueezeStore> = once_cell::sync::OnceCell::new();
    STORE
        .get_or_init(|| {
            let s = GammaSqueezeStore::new();
            spawn_poller(s.clone(), crate::live_ticks::global());
            s
        })
        .clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opt(strike: f64, oi: i64, iv: f64) -> options::OptionContract {
        options::OptionContract {
            strike,
            bid: Some(1.0),
            ask: Some(1.1),
            last_price: Some(1.05),
            implied_vol: Some(iv),
            volume: Some(0),
            open_interest: Some(oi),
            in_the_money: false,
        }
    }

    fn chain_with(
        spot: f64,
        expiry: chrono::NaiveDate,
        calls: Vec<(f64, i64, f64)>,
        puts: Vec<(f64, i64, f64)>,
    ) -> options::Chain {
        options::Chain {
            symbol: "TEST".into(),
            spot,
            expirations: vec![],
            expiration: expiry,
            calls: calls.into_iter().map(|(s, o, i)| opt(s, o, i)).collect(),
            puts: puts.into_iter().map(|(s, o, i)| opt(s, o, i)).collect(),
        }
    }

    fn future_date(days: i64) -> chrono::NaiveDate {
        Utc::now().date_naive() + chrono::Duration::days(days)
    }

    #[test]
    fn bs_gamma_returns_zero_on_degenerate_inputs() {
        assert_eq!(bs_gamma_per_share(0.0, 100.0, 0.1, 0.3, 0.04), 0.0);
        assert_eq!(bs_gamma_per_share(100.0, 0.0, 0.1, 0.3, 0.04), 0.0);
        assert_eq!(bs_gamma_per_share(100.0, 100.0, 0.0, 0.3, 0.04), 0.0);
        assert_eq!(bs_gamma_per_share(100.0, 100.0, 0.1, 0.0, 0.04), 0.0);
    }

    #[test]
    fn bs_gamma_atm_is_positive() {
        // ATM, 30 days, 30% vol — gamma should be a small positive number.
        let g = bs_gamma_per_share(100.0, 100.0, 30.0 / 365.0, 0.3, 0.04);
        assert!(g > 0.0);
        assert!(g < 1.0); // sanity bound; per-share gamma stays well under 1.
    }

    #[test]
    fn years_to_expiry_handles_past_date() {
        assert_eq!(
            years_to_expiry(Utc::now().date_naive() - chrono::Duration::days(1)),
            0.0
        );
    }

    #[test]
    fn chain_to_strikes_empty_when_expired() {
        let c = chain_with(
            100.0,
            Utc::now().date_naive() - chrono::Duration::days(1),
            vec![(100.0, 1000, 0.3)],
            vec![(100.0, 1000, 0.3)],
        );
        assert!(chain_to_strikes(&c).is_empty());
    }

    #[test]
    fn evaluate_chain_emits_when_negative_gex_and_spot_near_pin() {
        // Heavy call OI at strike 100 (dealers short calls → negative GEX
        // here), light put OI. Spot 99.5 → 0.5% below the pin strike,
        // inside the 2% window.
        let chain = chain_with(
            99.5,
            future_date(7),
            vec![(100.0, 5_000_000, 0.5), (110.0, 100_000, 0.5)],
            vec![(90.0, 100_000, 0.5), (100.0, 100_000, 0.5)],
        );
        let cand = evaluate_chain("TEST", &chain).expect("should emit");
        assert_eq!(cand.symbol, "TEST");
        assert!(cand.total_gex < 0.0);
        assert!(cand.pin_distance_pct.unwrap().abs() <= PIN_DISTANCE_PCT);
        assert_eq!(cand.largest_negative_strike, Some(100.0));
    }

    #[test]
    fn evaluate_chain_skips_when_spot_far_from_pin() {
        // Same OI shape but spot 105 → 5% above pin (100), outside the 2%
        // window. Even with massive negative GEX, no candidate.
        let chain = chain_with(
            105.0,
            future_date(7),
            vec![(100.0, 5_000_000, 0.5)],
            vec![(100.0, 100_000, 0.5)],
        );
        assert!(evaluate_chain("TEST", &chain).is_none());
    }

    #[test]
    fn evaluate_chain_skips_when_positive_gex() {
        // Heavy put OI dominates → positive total GEX → no squeeze
        // signal regardless of spot location.
        let chain = chain_with(
            100.0,
            future_date(7),
            vec![(100.0, 100_000, 0.5)],
            vec![(100.0, 5_000_000, 0.5)],
        );
        assert!(evaluate_chain("TEST", &chain).is_none());
    }

    #[test]
    fn observe_dedupes_repeat_candidate() {
        let store = GammaSqueezeStore::new();
        let c = GammaSqueezeCandidate {
            symbol: "AAPL".into(),
            expiry: "2026-06-19".into(),
            spot: 200.0,
            total_gex: -500_000_000.0,
            largest_negative_strike: Some(200.0),
            largest_positive_strike: Some(195.0),
            zero_gamma_strike: Some(197.5),
            pin_distance_pct: Some(0.0),
            observed_at: Utc::now(),
        };
        assert!(store.observe(c.clone()));
        assert!(!store.observe(c));
    }

    #[test]
    fn observe_distinct_keys_when_neg_strike_changes() {
        let store = GammaSqueezeStore::new();
        let mut c = GammaSqueezeCandidate {
            symbol: "AAPL".into(),
            expiry: "2026-06-19".into(),
            spot: 200.0,
            total_gex: -500_000_000.0,
            largest_negative_strike: Some(200.0),
            largest_positive_strike: Some(195.0),
            zero_gamma_strike: Some(197.5),
            pin_distance_pct: Some(0.0),
            observed_at: Utc::now(),
        };
        assert!(store.observe(c.clone()));
        c.largest_negative_strike = Some(205.0);
        assert!(store.observe(c));
        assert_eq!(store.emitted.len(), 2);
    }
}
