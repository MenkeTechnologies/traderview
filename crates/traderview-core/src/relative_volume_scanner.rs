//! Relative Volume (RVOL) Scanner.
//!
//! For each symbol, compares today's intraday volume to the rolling
//! N-day average volume *at the same time of day*. Symbols whose RVOL
//! exceeds a configurable threshold are flagged as exhibiting unusual
//! interest — a classic momentum / news / breakout precursor.
//!
//!   rvol = today_volume_to_date / mean(prior_n_days_volume_to_date)
//!
//! Two output measures:
//!   - **Intraday RVOL**: cumulative-to-date volume vs N-day average at
//!     the same intraday minute.
//!   - **Daily RVOL**: full-day volume vs N-day average daily volume
//!     (only valid post-close).
//!
//! Sort output by RVOL descending so the highest-conviction unusual-
//! volume names surface first.
//!
//! Pure compute. Companion to `volume_burst`, `premarket_gap_scanner`,
//! `breakout_52w_scanner`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolVolume {
    pub symbol: String,
    pub today_volume_to_date: f64,
    /// Average volume across prior N days at the same intraday time.
    pub n_day_avg_volume_to_date: f64,
    pub today_full_day_volume: Option<f64>,
    pub n_day_avg_full_day_volume: Option<f64>,
    pub last_close: f64,
    pub last_price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RvolHit {
    pub symbol: String,
    pub intraday_rvol: f64,
    pub daily_rvol: Option<f64>,
    pub today_volume: f64,
    pub avg_volume: f64,
    pub price_change_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub min_intraday_rvol: f64,
    pub min_avg_volume: f64,
    pub min_abs_price_change_pct: f64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            min_intraday_rvol: 2.0,
            min_avg_volume: 100_000.0,
            min_abs_price_change_pct: 0.0,
        }
    }
}

pub fn scan(symbols: &[SymbolVolume], cfg: &Config) -> Vec<RvolHit> {
    let mut hits: Vec<RvolHit> = symbols.iter().filter_map(|s| {
        if !s.today_volume_to_date.is_finite()
            || !s.n_day_avg_volume_to_date.is_finite()
            || !s.last_close.is_finite() || !s.last_price.is_finite()
            || s.last_close <= 0.0
            || s.n_day_avg_volume_to_date <= 0.0
            || s.today_volume_to_date < 0.0 { return None; }
        let intraday_rvol = s.today_volume_to_date / s.n_day_avg_volume_to_date;
        let daily_rvol = match (s.today_full_day_volume, s.n_day_avg_full_day_volume) {
            (Some(t), Some(a)) if t.is_finite() && a.is_finite() && a > 0.0 => Some(t / a),
            _ => None,
        };
        let pct_chg = (s.last_price - s.last_close) / s.last_close;
        if intraday_rvol < cfg.min_intraday_rvol
            || s.n_day_avg_volume_to_date < cfg.min_avg_volume
            || pct_chg.abs() < cfg.min_abs_price_change_pct { return None; }
        Some(RvolHit {
            symbol: s.symbol.clone(),
            intraday_rvol,
            daily_rvol,
            today_volume: s.today_volume_to_date,
            avg_volume: s.n_day_avg_volume_to_date,
            price_change_pct: pct_chg,
        })
    }).collect();
    hits.sort_by(|a, b| b.intraday_rvol.partial_cmp(&a.intraday_rvol)
        .unwrap_or(std::cmp::Ordering::Equal));
    hits
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sv(sym: &str, today: f64, avg: f64, close: f64, last: f64) -> SymbolVolume {
        SymbolVolume {
            symbol: sym.into(),
            today_volume_to_date: today,
            n_day_avg_volume_to_date: avg,
            today_full_day_volume: None,
            n_day_avg_full_day_volume: None,
            last_close: close,
            last_price: last,
        }
    }

    #[test]
    fn empty_returns_empty() {
        assert!(scan(&[], &Config::default()).is_empty());
    }

    #[test]
    fn below_threshold_filtered() {
        let s = sv("AAA", 100_000.0, 200_000.0, 50.0, 52.0);
        assert!(scan(&[s], &Config::default()).is_empty());
    }

    #[test]
    fn unusual_volume_emitted() {
        let s = sv("AAA", 1_000_000.0, 200_000.0, 50.0, 52.0);
        let hits = scan(&[s], &Config::default());
        assert_eq!(hits.len(), 1);
        let h = &hits[0];
        assert_eq!(h.symbol, "AAA");
        assert!((h.intraday_rvol - 5.0).abs() < 1e-9);
        assert!((h.price_change_pct - 0.04).abs() < 1e-9);
    }

    #[test]
    fn nan_or_negative_filtered() {
        let bad_today = sv("X", f64::NAN, 200_000.0, 50.0, 52.0);
        let neg_avg = sv("Y", 100_000.0, -1.0, 50.0, 52.0);
        let zero_close = sv("Z", 100_000.0, 200_000.0, 0.0, 52.0);
        assert!(scan(&[bad_today, neg_avg, zero_close], &Config::default()).is_empty());
    }

    #[test]
    fn sorted_by_rvol_descending() {
        let a = sv("AAA", 1_000_000.0, 200_000.0, 50.0, 52.0);    // 5x
        let b = sv("BBB", 5_000_000.0, 200_000.0, 50.0, 52.0);    // 25x
        let c = sv("CCC", 2_000_000.0, 200_000.0, 50.0, 52.0);    // 10x
        let hits = scan(&[a, b, c], &Config::default());
        assert_eq!(hits[0].symbol, "BBB");
        assert_eq!(hits[1].symbol, "CCC");
        assert_eq!(hits[2].symbol, "AAA");
    }

    #[test]
    fn min_price_change_filter_works() {
        let cfg = Config { min_abs_price_change_pct: 0.05, ..Default::default() };
        let small_change = sv("X", 1_000_000.0, 200_000.0, 50.0, 51.0);    // 2% change
        let big_change = sv("Y", 1_000_000.0, 200_000.0, 50.0, 55.0);    // 10% change
        let hits = scan(&[small_change, big_change], &cfg);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].symbol, "Y");
    }

    #[test]
    fn daily_rvol_computed_when_available() {
        let mut s = sv("X", 1_000_000.0, 200_000.0, 50.0, 52.0);
        s.today_full_day_volume = Some(3_000_000.0);
        s.n_day_avg_full_day_volume = Some(1_000_000.0);
        let hits = scan(&[s], &Config::default());
        assert_eq!(hits[0].daily_rvol, Some(3.0));
    }

    #[test]
    fn daily_rvol_none_when_full_day_missing() {
        let s = sv("X", 1_000_000.0, 200_000.0, 50.0, 52.0);
        let hits = scan(&[s], &Config::default());
        assert!(hits[0].daily_rvol.is_none());
    }
}
