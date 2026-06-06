//! Cross-sectional Low-Volatility factor — ranks symbols by trailing
//! N-day realized volatility and flags the lowest-volatility decile as
//! the canonical "low-vol anomaly" long-side.
//!
//! Procedure:
//!   1. Per symbol, compute stdev of daily log-returns over `lookback_days`.
//!   2. Annualize via × √252.
//!   3. Cross-sectionally rank → LowVolDecile (bottom 10% of vol) +
//!      HighVolDecile (top 10%).
//!
//! The low-vol anomaly — that low-vol stocks have HIGHER risk-adjusted
//! returns than high-vol stocks — has been empirically documented by
//! Blitz-Van Vliet (2007), Frazzini-Pedersen (2014), and earlier.
//!
//! Pure compute. Companion to `momentum_12_1`, `value_factor`,
//! `quality_factor`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolPriceHistory {
    pub symbol: String,
    /// Daily closes, most recent last.
    pub daily_closes: Vec<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VolBucket {
    LowVolDecile,
    HighVolDecile,
    Neutral,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LowVolHit {
    pub symbol: String,
    pub annualized_volatility: f64,
    pub percentile: f64,
    pub bucket: VolBucket,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LowVolFactorReport {
    pub low_vol_decile: Vec<LowVolHit>,
    pub high_vol_decile: Vec<LowVolHit>,
    pub all_ranked: Vec<LowVolHit>,
}

pub fn scan(symbols: &[SymbolPriceHistory], lookback_days: usize) -> Option<LowVolFactorReport> {
    if symbols.is_empty() || lookback_days < 5 {
        return None;
    }
    let mut vols: Vec<(String, f64)> = Vec::new();
    for s in symbols {
        let n = s.daily_closes.len();
        if n < lookback_days + 1 {
            continue;
        }
        // Trailing log-returns.
        let mut rets = Vec::with_capacity(lookback_days);
        for i in (n - lookback_days)..n {
            let prev = s.daily_closes[i - 1];
            let curr = s.daily_closes[i];
            if !prev.is_finite() || prev <= 0.0 || !curr.is_finite() || curr <= 0.0 {
                continue;
            }
            rets.push((curr / prev).ln());
        }
        if rets.len() < 5 {
            continue;
        }
        let m: f64 = rets.iter().sum::<f64>() / rets.len() as f64;
        let var: f64 =
            rets.iter().map(|r| (r - m).powi(2)).sum::<f64>() / (rets.len() as f64 - 1.0);
        let sd = var.max(0.0).sqrt();
        let annualized = sd * (252.0_f64).sqrt();
        if annualized.is_finite() && annualized > 0.0 {
            vols.push((s.symbol.clone(), annualized));
        }
    }
    if vols.is_empty() {
        return None;
    }
    vols.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    let n = vols.len();
    let mut all_ranked: Vec<LowVolHit> = vols
        .iter()
        .enumerate()
        .map(|(i, (sym, v))| {
            let pct = (i + 1) as f64 / n as f64 * 100.0;
            let bucket = if pct <= 10.0 {
                VolBucket::LowVolDecile
            } else if pct >= 90.0 {
                VolBucket::HighVolDecile
            } else {
                VolBucket::Neutral
            };
            LowVolHit {
                symbol: sym.clone(),
                annualized_volatility: *v,
                percentile: pct,
                bucket,
            }
        })
        .collect();
    // Lowest vol first.
    let low_vol_decile: Vec<LowVolHit> = all_ranked
        .iter()
        .filter(|h| h.bucket == VolBucket::LowVolDecile)
        .cloned()
        .collect();
    let high_vol_decile: Vec<LowVolHit> = all_ranked
        .iter()
        .filter(|h| h.bucket == VolBucket::HighVolDecile)
        .cloned()
        .collect();
    // Sort all_ranked descending by vol (high-to-low) for display convention.
    all_ranked.sort_by(|a, b| {
        b.annualized_volatility
            .partial_cmp(&a.annualized_volatility)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    Some(LowVolFactorReport {
        low_vol_decile,
        high_vol_decile,
        all_ranked,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(sym: &str, closes: Vec<f64>) -> SymbolPriceHistory {
        SymbolPriceHistory {
            symbol: sym.into(),
            daily_closes: closes,
        }
    }

    #[test]
    fn empty_returns_none() {
        assert!(scan(&[], 20).is_none());
    }

    #[test]
    fn too_short_returns_none() {
        assert!(scan(&[], 3).is_none());
        assert!(scan(&[s("X", vec![100.0; 5])], 20).is_none());
    }

    #[test]
    fn insufficient_data_per_symbol_filtered() {
        let r = scan(&[s("SHORT", vec![100.0; 10])], 20);
        assert!(r.is_none());
    }

    #[test]
    fn lowest_vol_in_low_vol_decile() {
        // 10 symbols: vol scales with i.
        let mut symbols = Vec::new();
        for i in 1..=10 {
            let mut closes = vec![100.0_f64];
            for j in 1_usize..=30 {
                let pct_move = (i as f64) * 0.01 * if j.is_multiple_of(2) { 1.0 } else { -1.0 };
                closes.push(closes.last().unwrap() * (1.0 + pct_move));
            }
            symbols.push(s(&format!("V{i:02}"), closes));
        }
        let r = scan(&symbols, 20).unwrap();
        // V01 should be in low-vol decile.
        assert!(r.low_vol_decile.iter().any(|h| h.symbol == "V01"));
    }

    #[test]
    fn highest_vol_in_high_vol_decile() {
        let mut symbols = Vec::new();
        for i in 1..=10 {
            let mut closes = vec![100.0_f64];
            for j in 1_usize..=30 {
                let pct_move = (i as f64) * 0.01 * if j.is_multiple_of(2) { 1.0 } else { -1.0 };
                closes.push(closes.last().unwrap() * (1.0 + pct_move));
            }
            symbols.push(s(&format!("V{i:02}"), closes));
        }
        let r = scan(&symbols, 20).unwrap();
        assert!(r.high_vol_decile.iter().any(|h| h.symbol == "V10"));
    }

    #[test]
    fn flat_series_excluded() {
        // Stdev = 0 → annualized vol = 0 → excluded.
        let r = scan(&[s("FLAT", vec![100.0; 30])], 20);
        assert!(r.is_none());
    }

    #[test]
    fn percentiles_in_unit_range() {
        let mut symbols = Vec::new();
        for i in 1..=20 {
            let mut closes = vec![100.0_f64];
            for j in 1_usize..=30 {
                let pct_move = (i as f64) * 0.005 * if j.is_multiple_of(2) { 1.0 } else { -1.0 };
                closes.push(closes.last().unwrap() * (1.0 + pct_move));
            }
            symbols.push(s(&format!("X{i:02}"), closes));
        }
        let r = scan(&symbols, 25).unwrap();
        for h in &r.all_ranked {
            assert!((0.0..=100.0).contains(&h.percentile));
        }
    }

    #[test]
    fn nan_closes_filtered() {
        let mut closes = vec![100.0_f64; 30];
        closes[10] = f64::NAN;
        let r = scan(&[s("X", closes)], 20);
        // The symbol survives if enough valid returns; otherwise filtered
        // (acceptable when too many NaN).
        if let Some(report) = r {
            assert!(report.all_ranked[0].annualized_volatility.is_finite());
        }
    }
}
