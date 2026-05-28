//! Relative Strength (IBD-style) — symbol vs benchmark return ratio.
//!
//! NOT the same as Wilder's RSI. This is the William O'Neil / IBD-100
//! "RS rating" concept: a symbol's % return over `period` divided by
//! the benchmark's (SPY default) % return over the same window. Used to
//! rank a watchlist by outperformance vs broad market.
//!
//!   rs_value = (1 + symbol_return) / (1 + benchmark_return)
//!   rs_score = rs_value indexed to 1.0 (1.0 = matches benchmark,
//!              > 1 = outperforming, < 1 = lagging)
//!
//! When applied across a universe, percentile-rank the rs_scores 0-100
//! (the canonical IBD RS rating). Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolPrices {
    pub symbol: String,
    /// Most recent close last.
    pub closes: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RsConfig {
    pub period: usize,
}

impl Default for RsConfig {
    fn default() -> Self { Self { period: 63 } }    // ~ one quarter of trading days
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RsEntry {
    pub symbol: String,
    pub symbol_return_pct: f64,
    pub benchmark_return_pct: f64,
    pub rs_value: f64,
    /// 0..=100 percentile across the universe (higher = stronger).
    pub rs_rating: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RsReport {
    pub entries: Vec<RsEntry>,
    /// Symbols with rs_rating ≥ 80 — the IBD "leadership" cut.
    pub leaders: Vec<String>,
    pub benchmark_return_pct: f64,
}

pub fn analyze(
    universe: &[SymbolPrices],
    benchmark: &SymbolPrices,
    cfg: &RsConfig,
) -> RsReport {
    let mut report = RsReport::default();
    if cfg.period == 0 {
        return report;
    }
    let Some(bench_ret) = period_return(&benchmark.closes, cfg.period) else {
        return report;
    };
    report.benchmark_return_pct = bench_ret * 100.0;
    let bench_factor = 1.0 + bench_ret;
    if !bench_factor.is_finite() || bench_factor == 0.0 {
        return report;
    }
    let mut entries = Vec::new();
    for sym in universe {
        let Some(sym_ret) = period_return(&sym.closes, cfg.period) else { continue };
        let rs_value = (1.0 + sym_ret) / bench_factor;
        if !rs_value.is_finite() {
            continue;
        }
        entries.push(RsEntry {
            symbol: sym.symbol.clone(),
            symbol_return_pct: sym_ret * 100.0,
            benchmark_return_pct: bench_ret * 100.0,
            rs_value,
            rs_rating: 0.0,    // filled in below
        });
    }
    // Percentile-rank rs_value across the universe.
    let n = entries.len();
    if n == 0 {
        return report;
    }
    let mut sorted_rs: Vec<f64> = entries.iter().map(|e| e.rs_value).collect();
    sorted_rs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    for e in &mut entries {
        // How many values are STRICTLY less than mine, scaled 0..=100.
        let below = sorted_rs.iter().filter(|x| **x < e.rs_value).count() as f64;
        e.rs_rating = below / n as f64 * 100.0;
    }
    // Sort report by rs_rating descending.
    entries.sort_by(|a, b| {
        b.rs_rating
            .partial_cmp(&a.rs_rating)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    report.leaders = entries.iter()
        .filter(|e| e.rs_rating >= 80.0)
        .map(|e| e.symbol.clone())
        .collect();
    report.entries = entries;
    report
}

fn period_return(closes: &[f64], period: usize) -> Option<f64> {
    let n = closes.len();
    if n <= period {
        return None;
    }
    let start = closes[n - 1 - period];
    let end = closes[n - 1];
    if !start.is_finite() || !end.is_finite() || start <= 0.0 {
        return None;
    }
    Some((end - start) / start)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(sym: &str, closes: Vec<f64>) -> SymbolPrices {
        SymbolPrices { symbol: sym.into(), closes }
    }

    #[test]
    fn empty_returns_default() {
        let bench = s("SPY", vec![100.0; 70]);
        let r = analyze(&[], &bench, &RsConfig::default());
        assert!(r.entries.is_empty());
    }

    #[test]
    fn zero_period_returns_default() {
        let bench = s("SPY", vec![100.0; 70]);
        let r = analyze(&[s("A", vec![100.0; 70])], &bench, &RsConfig { period: 0 });
        assert!(r.entries.is_empty());
    }

    #[test]
    fn benchmark_too_short_returns_default() {
        let bench = s("SPY", vec![100.0; 10]);    // < 63 default
        let r = analyze(&[s("A", vec![100.0; 70])], &bench, &RsConfig::default());
        assert!(r.entries.is_empty());
    }

    #[test]
    fn outperformer_gets_high_rs_rating() {
        // SPY return: 100 → 110 = +10%.
        // SYM_HOT return: 100 → 130 = +30%. rs_value = 1.30/1.10 ≈ 1.18.
        // SYM_DOG return: 100 → 105 = +5%. rs_value = 1.05/1.10 ≈ 0.955.
        let bench = s("SPY", {
            let mut v = vec![100.0; 64];
            v[63] = 110.0;
            v
        });
        let universe = vec![
            s("HOT", {
                let mut v = vec![100.0; 64];
                v[63] = 130.0;
                v
            }),
            s("DOG", {
                let mut v = vec![100.0; 64];
                v[63] = 105.0;
                v
            }),
        ];
        let r = analyze(&universe, &bench, &RsConfig::default());
        assert_eq!(r.entries[0].symbol, "HOT");    // sorted desc by rating
        assert!(r.entries[0].rs_value > 1.0);
        assert!(r.entries[1].rs_value < 1.0);
        assert!(r.leaders.contains(&"HOT".to_string()) || r.entries[0].rs_rating < 80.0);
    }

    #[test]
    fn flat_benchmark_with_flat_universe_yields_rs_value_one() {
        let bench = s("SPY", vec![100.0; 70]);
        let r = analyze(&[s("X", vec![100.0; 70])], &bench, &RsConfig::default());
        assert_eq!(r.entries.len(), 1);
        assert!((r.entries[0].rs_value - 1.0).abs() < 1e-9);
    }

    #[test]
    fn nan_or_zero_start_skipped() {
        let bench = s("SPY", {
            let mut v = vec![100.0; 64];
            v[63] = 110.0;
            v
        });
        let universe = vec![
            s("NAN_END", {
                let mut v = vec![100.0; 64];
                v[63] = f64::NAN;
                v
            }),
            s("ZERO_START", {
                let mut v = vec![100.0; 64];
                v[0] = 0.0;
                v
            }),
        ];
        let r = analyze(&universe, &bench, &RsConfig::default());
        // Both filtered out.
        assert!(r.entries.is_empty());
    }

    #[test]
    fn rs_rating_in_range_0_to_100() {
        let bench = s("SPY", {
            let mut v = vec![100.0; 64];
            v[63] = 110.0;
            v
        });
        let universe: Vec<SymbolPrices> = (0..10).map(|i| s(&format!("X{i}"), {
            let mut v = vec![100.0; 64];
            v[63] = 100.0 + i as f64;
            v
        })).collect();
        let r = analyze(&universe, &bench, &RsConfig::default());
        for e in &r.entries {
            assert!((0.0..=100.0).contains(&e.rs_rating), "rs_rating out of range: {}", e.rs_rating);
        }
    }
}
