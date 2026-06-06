//! Universe-wide IV-rank scanner.
//!
//! Walks a list of symbols' (current_iv, lookback_iv_history) inputs
//! and ranks each symbol's IV percentile vs its own history:
//!
//!   IV-Rank = (current_iv − min_iv) / (max_iv − min_iv) × 100
//!
//! Then flags symbols whose IV is in the top quartile (potential
//! premium-selling candidates: high IV = expensive options to sell) and
//! bottom quartile (premium-buying candidates: cheap options worth long
//! gamma plays).
//!
//! Distinct from `crate::iv_rank` which is the per-symbol calculator;
//! this is the orchestrator that runs the universe and ranks across
//! symbols.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolIv {
    pub symbol: String,
    /// Today's at-the-money IV (e.g. 0.32 = 32%).
    pub current_iv: f64,
    /// Trailing window of daily IVs.
    pub history: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IvRankEntry {
    pub symbol: String,
    pub current_iv: f64,
    pub iv_rank_pct: f64,
    pub min_iv: f64,
    pub max_iv: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IvRankReport {
    pub entries: Vec<IvRankEntry>,
    /// Symbols with IV-rank ≥ 75% (top quartile → sell premium).
    pub high_iv_candidates: Vec<String>,
    /// Symbols with IV-rank ≤ 25% (bottom quartile → buy premium).
    pub low_iv_candidates: Vec<String>,
}

pub fn analyze(symbols: &[SymbolIv]) -> IvRankReport {
    let mut report = IvRankReport::default();
    for s in symbols {
        if !s.current_iv.is_finite() || s.history.is_empty() {
            continue;
        }
        let mut min_iv = f64::INFINITY;
        let mut max_iv = f64::NEG_INFINITY;
        for &v in &s.history {
            if !v.is_finite() {
                continue;
            }
            if v < min_iv {
                min_iv = v;
            }
            if v > max_iv {
                max_iv = v;
            }
        }
        if !min_iv.is_finite() || !max_iv.is_finite() {
            continue;
        }
        let range = max_iv - min_iv;
        let rank = if range > 0.0 {
            (s.current_iv - min_iv) / range * 100.0
        } else {
            // Flat history → undefined rank; report 50 (mid) as the
            // neutral fallback rather than 0 or 100.
            50.0
        };
        let rank = rank.clamp(0.0, 100.0);
        report.entries.push(IvRankEntry {
            symbol: s.symbol.clone(),
            current_iv: s.current_iv,
            iv_rank_pct: rank,
            min_iv,
            max_iv,
        });
    }
    // Sort by IV-rank descending so the dashboard shows highest first.
    report.entries.sort_by(|a, b| {
        b.iv_rank_pct
            .partial_cmp(&a.iv_rank_pct)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    for e in &report.entries {
        if e.iv_rank_pct >= 75.0 {
            report.high_iv_candidates.push(e.symbol.clone());
        }
        if e.iv_rank_pct <= 25.0 {
            report.low_iv_candidates.push(e.symbol.clone());
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(sym: &str, current: f64, hist: Vec<f64>) -> SymbolIv {
        SymbolIv {
            symbol: sym.into(),
            current_iv: current,
            history: hist,
        }
    }

    #[test]
    fn empty_returns_default() {
        let r = analyze(&[]);
        assert!(r.entries.is_empty());
    }

    #[test]
    fn symbol_with_empty_history_skipped() {
        let r = analyze(&[s("AAPL", 0.3, vec![])]);
        assert!(r.entries.is_empty());
    }

    #[test]
    fn symbol_with_nonfinite_current_skipped() {
        let r = analyze(&[s("AAPL", f64::NAN, vec![0.2, 0.3])]);
        assert!(r.entries.is_empty());
    }

    #[test]
    fn current_at_max_yields_rank_100() {
        let r = analyze(&[s("AAPL", 0.5, vec![0.1, 0.2, 0.3, 0.5])]);
        assert!((r.entries[0].iv_rank_pct - 100.0).abs() < 1e-9);
    }

    #[test]
    fn current_at_min_yields_rank_0() {
        let r = analyze(&[s("AAPL", 0.1, vec![0.1, 0.2, 0.5])]);
        assert!(r.entries[0].iv_rank_pct.abs() < 1e-9);
    }

    #[test]
    fn flat_history_returns_neutral_50() {
        let r = analyze(&[s("AAPL", 0.3, vec![0.3; 30])]);
        assert!((r.entries[0].iv_rank_pct - 50.0).abs() < 1e-9);
    }

    #[test]
    fn high_iv_candidates_take_top_quartile() {
        let inputs = vec![
            s(
                "HIGH",
                0.95,
                (0..50).map(|i| 0.1 + i as f64 * 0.02).collect(),
            ), // current at max
            s(
                "LOW",
                0.10,
                (0..50).map(|i| 0.1 + i as f64 * 0.02).collect(),
            ), // current at min
            s(
                "MID",
                0.50,
                (0..50).map(|i| 0.1 + i as f64 * 0.02).collect(),
            ), // middle
        ];
        let r = analyze(&inputs);
        assert!(r.high_iv_candidates.contains(&"HIGH".to_string()));
        assert!(r.low_iv_candidates.contains(&"LOW".to_string()));
        assert!(!r.high_iv_candidates.contains(&"MID".to_string()));
    }

    #[test]
    fn output_sorted_by_iv_rank_descending() {
        let inputs = vec![
            s("A", 0.1, vec![0.1, 0.5]), // rank 0
            s("B", 0.5, vec![0.1, 0.5]), // rank 100
            s("C", 0.3, vec![0.1, 0.5]), // rank 50
        ];
        let r = analyze(&inputs);
        let order: Vec<_> = r.entries.iter().map(|e| e.symbol.as_str()).collect();
        assert_eq!(order, vec!["B", "C", "A"]);
    }

    #[test]
    fn rank_clamped_to_0_to_100_even_if_current_exceeds_history() {
        // Current 0.8 but history only saw 0.1..0.5 → naive math: 175%.
        // Clamp to 100.
        let r = analyze(&[s("X", 0.8, vec![0.1, 0.3, 0.5])]);
        assert!((r.entries[0].iv_rank_pct - 100.0).abs() < 1e-9);
    }
}
