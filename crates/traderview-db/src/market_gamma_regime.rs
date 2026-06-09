//! Market-wide dealer gamma regime tracker.
//!
//! The per-symbol [`crate::gamma_squeeze`] detector flags individual
//! names primed for a squeeze. This module zooms out: it computes the
//! **total SPY-level dealer GEX** across the near-term option
//! expirations and tracks the *regime* over time:
//!
//!   * **Positive GEX** — dealers are net long gamma. They sell into
//!     rallies and buy into dips to re-hedge, which *suppresses* vol
//!     and produces mean-reverting intraday tape. Mean-reversion
//!     strategies (selling premium, fading extremes) are favored.
//!
//!   * **Negative GEX** — dealers are net short gamma. They buy
//!     rallies and sell dips to re-hedge, which *amplifies* vol and
//!     produces momentum / breakout tape. Breakout strategies,
//!     trend-following, and long-vol structures are favored.
//!
//!   * **Sign flip** — the moment GEX crosses zero. Empirically these
//!     flip days bracket the largest volatility expansions in the
//!     SPX historical record. Documented in the Squeeze Metrics
//!     research, JPM Equity Derivatives notes, and Goldman quant
//!     desk publications since the early 2010s.
//!
//! This isn't a per-trade entry signal — it's a **portfolio-level
//! regime indicator**. Knowing which side of the gamma flip you're
//! on tells you which strategies have a tailwind and which have a
//! headwind, and warns you when that's about to change.
//!
//! Implementation: every `REFRESH_SECS` the poller pulls SPY's
//! option-chain index, picks the nearest `MAX_EXPIRATIONS` (front
//! month plus follow-on weeklies), computes per-expiration
//! `gex_scanner::scan` via the existing
//! `gamma_squeeze::chain_to_strikes` helper, and sums into
//! `total_gex`. The snapshot enters a bounded rolling history.
//! `detect_flips` walks the history and surfaces regime crosses.

use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use std::sync::{Arc, Mutex};
use std::time::Duration as StdDuration;

use crate::gamma_squeeze;
use crate::options;
use traderview_core::gex_scanner;

const SPX_PROXY: &str = "SPY";
const REFRESH_SECS: u64 = 30 * 60;
const MAX_EXPIRATIONS: usize = 4;
const PACE_MS: u64 = 200;
/// Below this absolute magnitude, the GEX is treated as effectively
/// zero — the noise floor of free-tier IV data is wide enough that
/// anything closer to zero than this is indistinguishable from
/// regime-neutral. $100M is conservative; production desks use $250M+
/// thresholds depending on data quality.
const NEUTRAL_THRESHOLD_USD: f64 = 100_000_000.0;
const HISTORY_MAX: usize = 720; // ~15 days at one snapshot every 30 min.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Regime {
    Positive,
    Negative,
    Neutral,
}

impl Regime {
    pub fn as_str(self) -> &'static str {
        match self {
            Regime::Positive => "positive",
            Regime::Negative => "negative",
            Regime::Neutral => "neutral",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct GexSnapshot {
    pub observed_at: DateTime<Utc>,
    pub total_gex_usd: f64,
    pub spot: f64,
    pub expirations_used: Vec<NaiveDate>,
    pub regime: Regime,
}

#[derive(Debug, Clone, Serialize)]
pub struct RegimeFlip {
    pub from: Regime,
    pub to: Regime,
    pub flipped_at: DateTime<Utc>,
    pub prior_total_gex_usd: f64,
    pub current_total_gex_usd: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct MarketGammaReport {
    pub current_regime: Regime,
    pub current_total_gex_usd: f64,
    pub current_spot: f64,
    pub last_observed_at: DateTime<Utc>,
    pub last_flip: Option<RegimeFlip>,
    pub time_in_regime_secs: i64,
    pub history: Vec<GexSnapshot>,
    pub recent_flips: Vec<RegimeFlip>,
}

#[derive(Clone)]
pub struct MarketGammaStore {
    history: Arc<Mutex<Vec<GexSnapshot>>>,
}

impl MarketGammaStore {
    pub fn new() -> Self {
        Self {
            history: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn upsert(&self, snap: GexSnapshot) {
        let mut h = self.history.lock().expect("market_gamma_regime mutex");
        h.push(snap);
        let len = h.len();
        if len > HISTORY_MAX {
            let drop = len - HISTORY_MAX;
            h.drain(..drop);
        }
    }

    pub fn snapshot_history(&self) -> Vec<GexSnapshot> {
        self.history
            .lock()
            .expect("market_gamma_regime mutex")
            .clone()
    }

    pub fn report(&self, now: DateTime<Utc>) -> Option<MarketGammaReport> {
        let history = self.snapshot_history();
        if history.is_empty() {
            return None;
        }
        let latest = history.last().expect("history non-empty").clone();
        let flips = detect_flips(&history);
        let last_flip = flips.last().cloned();
        let time_in_regime_secs = last_flip
            .as_ref()
            .map(|f| (now - f.flipped_at).num_seconds())
            .unwrap_or_else(|| {
                let earliest = history.first().expect("non-empty").observed_at;
                (now - earliest).num_seconds()
            });
        Some(MarketGammaReport {
            current_regime: latest.regime,
            current_total_gex_usd: latest.total_gex_usd,
            current_spot: latest.spot,
            last_observed_at: latest.observed_at,
            last_flip,
            time_in_regime_secs,
            history,
            recent_flips: flips,
        })
    }
}

impl Default for MarketGammaStore {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Pure compute ──────────────────────────────────────────────────────────

/// Classify a single GEX magnitude. The `Neutral` band is centered at
/// zero with half-width `NEUTRAL_THRESHOLD_USD` — anything inside is
/// treated as regime-indistinct.
pub fn classify_regime(total_gex_usd: f64) -> Regime {
    if !total_gex_usd.is_finite() {
        return Regime::Neutral;
    }
    if total_gex_usd >= NEUTRAL_THRESHOLD_USD {
        Regime::Positive
    } else if total_gex_usd <= -NEUTRAL_THRESHOLD_USD {
        Regime::Negative
    } else {
        Regime::Neutral
    }
}

/// Walk a chronological snapshot list and emit one `RegimeFlip` per
/// regime change between adjacent samples. Neutral-to-positive (or
/// negative) IS a flip — it's the moment the market exits regime-
/// indistinct territory. Same-regime adjacencies emit nothing.
pub fn detect_flips(history: &[GexSnapshot]) -> Vec<RegimeFlip> {
    let mut flips: Vec<RegimeFlip> = Vec::new();
    if history.len() < 2 {
        return flips;
    }
    for pair in history.windows(2) {
        let prev = &pair[0];
        let cur = &pair[1];
        if prev.regime != cur.regime {
            flips.push(RegimeFlip {
                from: prev.regime,
                to: cur.regime,
                flipped_at: cur.observed_at,
                prior_total_gex_usd: prev.total_gex_usd,
                current_total_gex_usd: cur.total_gex_usd,
            });
        }
    }
    flips
}

/// Sum total dealer GEX (in dollars) across one or more option
/// chains. Each chain is processed via the existing
/// `gamma_squeeze::chain_to_strikes` + `gex_scanner::scan` pipeline.
/// Returns `(total_gex_usd, expirations_used)` — None when no chain
/// yields a valid GEX report.
pub fn sum_gex_across_chains(chains: &[options::Chain]) -> Option<(f64, Vec<NaiveDate>)> {
    let mut total = 0.0_f64;
    let mut used: Vec<NaiveDate> = Vec::new();
    for chain in chains {
        let strikes = gamma_squeeze::chain_to_strikes(chain);
        if strikes.is_empty() {
            continue;
        }
        if let Some(report) = gex_scanner::scan(&strikes) {
            if report.total_gex.is_finite() {
                total += report.total_gex;
                used.push(chain.expiration);
            }
        }
    }
    if used.is_empty() {
        None
    } else {
        Some((total, used))
    }
}

// ─── Repository (live data fetch) ──────────────────────────────────────────

/// Fetch a SPY-based market GEX snapshot. Pulls the chain index, picks
/// the nearest `MAX_EXPIRATIONS` future expirations, fetches each
/// chain, sums their per-strike GEX. Returns `None` on fetch failure
/// or when SPY has no usable IV data.
pub async fn fetch_snapshot() -> Option<GexSnapshot> {
    let index = options::chain(SPX_PROXY, None).await.ok()?;
    let today = Utc::now().date_naive();
    let mut exps: Vec<NaiveDate> = index
        .expirations
        .iter()
        .copied()
        .filter(|d| (*d - today).num_days() > 0)
        .collect();
    exps.sort();
    exps.truncate(MAX_EXPIRATIONS);
    let mut chains: Vec<options::Chain> = Vec::with_capacity(exps.len());
    let mut spot = index.spot;
    for exp in exps {
        match options::chain(SPX_PROXY, Some(exp)).await {
            Ok(chain) => {
                if chain.spot > 0.0 {
                    spot = chain.spot;
                }
                chains.push(chain);
            }
            Err(e) => {
                tracing::debug!(?e, %exp, "market_gamma: chain fetch failed");
            }
        }
        tokio::time::sleep(StdDuration::from_millis(PACE_MS)).await;
    }
    let (total_gex_usd, expirations_used) = sum_gex_across_chains(&chains)?;
    Some(GexSnapshot {
        observed_at: Utc::now(),
        total_gex_usd,
        spot,
        expirations_used,
        regime: classify_regime(total_gex_usd),
    })
}

pub fn spawn_poller(store: MarketGammaStore) {
    tokio::spawn(async move {
        tokio::time::sleep(StdDuration::from_secs(120)).await;
        let mut interval = tokio::time::interval(StdDuration::from_secs(REFRESH_SECS));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            interval.tick().await;
            if let Some(snap) = fetch_snapshot().await {
                store.upsert(snap);
            }
        }
    });
}

static STORE: once_cell::sync::OnceCell<MarketGammaStore> = once_cell::sync::OnceCell::new();

pub fn global() -> MarketGammaStore {
    STORE
        .get_or_init(|| {
            let s = MarketGammaStore::new();
            spawn_poller(s.clone());
            s
        })
        .clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snap(regime_value_usd: f64, t_offset_secs: i64) -> GexSnapshot {
        GexSnapshot {
            observed_at: Utc::now() + chrono::Duration::seconds(t_offset_secs),
            total_gex_usd: regime_value_usd,
            spot: 500.0,
            expirations_used: vec![],
            regime: classify_regime(regime_value_usd),
        }
    }

    #[test]
    fn classify_handles_three_regimes_with_neutral_band() {
        // Just above the threshold → Positive.
        assert_eq!(classify_regime(150_000_000.0), Regime::Positive);
        // Just below the negative threshold → Negative.
        assert_eq!(classify_regime(-150_000_000.0), Regime::Negative);
        // Inside the ±100M neutral band → Neutral.
        assert_eq!(classify_regime(0.0), Regime::Neutral);
        assert_eq!(classify_regime(50_000_000.0), Regime::Neutral);
        assert_eq!(classify_regime(-50_000_000.0), Regime::Neutral);
        // Right at the threshold → still Positive/Negative.
        assert_eq!(classify_regime(NEUTRAL_THRESHOLD_USD), Regime::Positive);
        assert_eq!(classify_regime(-NEUTRAL_THRESHOLD_USD), Regime::Negative);
    }

    #[test]
    fn classify_returns_neutral_on_non_finite() {
        assert_eq!(classify_regime(f64::NAN), Regime::Neutral);
        assert_eq!(classify_regime(f64::INFINITY), Regime::Neutral);
    }

    #[test]
    fn detect_flips_empty_or_single_element() {
        assert!(detect_flips(&[]).is_empty());
        assert!(detect_flips(&[snap(500e6, 0)]).is_empty());
    }

    #[test]
    fn detect_flips_no_change_no_flip() {
        let history = vec![snap(500e6, 0), snap(600e6, 1800), snap(550e6, 3600)];
        assert!(detect_flips(&history).is_empty());
    }

    #[test]
    fn detect_flips_positive_to_negative() {
        let history = vec![snap(500e6, 0), snap(450e6, 1800), snap(-300e6, 3600)];
        let flips = detect_flips(&history);
        assert_eq!(flips.len(), 1);
        assert_eq!(flips[0].from, Regime::Positive);
        assert_eq!(flips[0].to, Regime::Negative);
        assert_eq!(flips[0].flipped_at, history[2].observed_at);
    }

    #[test]
    fn detect_flips_through_neutral_emits_two() {
        // Positive → Neutral → Negative is two regime crosses, not one,
        // because neutral is a distinct regime.
        let history = vec![snap(500e6, 0), snap(50e6, 1800), snap(-300e6, 3600)];
        let flips = detect_flips(&history);
        assert_eq!(flips.len(), 2);
        assert_eq!(flips[0].from, Regime::Positive);
        assert_eq!(flips[0].to, Regime::Neutral);
        assert_eq!(flips[1].from, Regime::Neutral);
        assert_eq!(flips[1].to, Regime::Negative);
    }

    #[test]
    fn detect_flips_alternating_regimes() {
        let history = vec![
            snap(500e6, 0),
            snap(-500e6, 1800),
            snap(500e6, 3600),
            snap(-500e6, 5400),
        ];
        assert_eq!(detect_flips(&history).len(), 3);
    }

    #[test]
    fn store_upsert_bounds_history() {
        let store = MarketGammaStore::new();
        for i in 0..(HISTORY_MAX + 50) {
            store.upsert(snap((i as f64) * 1e7, i as i64 * 60));
        }
        assert!(store.snapshot_history().len() <= HISTORY_MAX);
    }

    #[test]
    fn store_report_none_on_empty_history() {
        let store = MarketGammaStore::new();
        assert!(store.report(Utc::now()).is_none());
    }

    #[test]
    fn store_report_carries_latest_regime_and_flip() {
        let store = MarketGammaStore::new();
        store.upsert(snap(500e6, 0));
        store.upsert(snap(400e6, 1800));
        store.upsert(snap(-300e6, 3600));
        let report = store
            .report(Utc::now() + chrono::Duration::seconds(3600))
            .expect("non-empty");
        assert_eq!(report.current_regime, Regime::Negative);
        assert_eq!(report.recent_flips.len(), 1);
        let flip = report.last_flip.unwrap();
        assert_eq!(flip.from, Regime::Positive);
        assert_eq!(flip.to, Regime::Negative);
    }

    #[test]
    fn store_report_time_in_regime_from_last_flip() {
        let store = MarketGammaStore::new();
        let t0 = Utc::now();
        store.upsert(GexSnapshot {
            observed_at: t0,
            total_gex_usd: 500e6,
            spot: 500.0,
            expirations_used: vec![],
            regime: Regime::Positive,
        });
        store.upsert(GexSnapshot {
            observed_at: t0 + chrono::Duration::seconds(3600),
            total_gex_usd: -500e6,
            spot: 500.0,
            expirations_used: vec![],
            regime: Regime::Negative,
        });
        let now = t0 + chrono::Duration::seconds(7200);
        let report = store.report(now).expect("non-empty");
        // Flipped at t0+3600, now is t0+7200 → 3600 secs in regime.
        assert_eq!(report.time_in_regime_secs, 3600);
    }

    #[test]
    fn store_report_time_in_regime_from_history_start_when_no_flip() {
        let store = MarketGammaStore::new();
        let t0 = Utc::now();
        store.upsert(GexSnapshot {
            observed_at: t0,
            total_gex_usd: 500e6,
            spot: 500.0,
            expirations_used: vec![],
            regime: Regime::Positive,
        });
        store.upsert(GexSnapshot {
            observed_at: t0 + chrono::Duration::seconds(3600),
            total_gex_usd: 600e6,
            spot: 500.0,
            expirations_used: vec![],
            regime: Regime::Positive,
        });
        let now = t0 + chrono::Duration::seconds(5400);
        let report = store.report(now).expect("non-empty");
        assert!(report.last_flip.is_none());
        // No flip → time-in-regime measured from earliest history sample.
        assert_eq!(report.time_in_regime_secs, 5400);
    }
}
