//! Composite factor scoring — combines multiple per-symbol factor
//! scores (momentum, value, quality, low-vol, …) into a single
//! weighted-sum cross-sectional score.
//!
//! Each factor's percentile rank is computed within the universe, then
//! weighted-averaged by user-supplied weights to produce a single
//! 0..100 composite. Output:
//!   - TopComposite (composite ≥ 90% percentile)
//!   - BottomComposite (≤ 10%)
//!   - Neutral
//!
//! Distinct from the individual factor modules — this is the
//! cross-factor aggregator used by smart-beta portfolio constructors.
//!
//! Pure compute. Caller provides aligned per-symbol score arrays + a
//! parallel weights vector. Weights may be negative (e.g. weight = -1
//! to short the high-vol decile).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolFactorScores {
    pub symbol: String,
    /// Per-factor raw scores; must align positionally with `factor_weights`.
    pub factor_scores: Vec<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompositeBucket {
    TopComposite,
    BottomComposite,
    Neutral,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositeHit {
    pub symbol: String,
    pub composite_score: f64,
    pub composite_percentile: f64,
    pub bucket: CompositeBucket,
    pub per_factor_percentiles: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompositeReport {
    pub top_composite: Vec<CompositeHit>,
    pub bottom_composite: Vec<CompositeHit>,
    pub all_ranked: Vec<CompositeHit>,
}

pub fn score(symbols: &[SymbolFactorScores], factor_weights: &[f64]) -> Option<CompositeReport> {
    let n_factors = factor_weights.len();
    if n_factors == 0 || symbols.is_empty() {
        return None;
    }
    if factor_weights.iter().any(|w| !w.is_finite()) {
        return None;
    }
    // Validate all symbols have the same number of factor scores.
    if symbols.iter().any(|s| {
        s.factor_scores.len() != n_factors || s.factor_scores.iter().any(|x| !x.is_finite())
    }) {
        return None;
    }
    let n_symbols = symbols.len();
    // Per-factor cross-sectional percentile rank. f is the factor column
    // and needs to be used both for sorting and for matrix assignment;
    // iter-based loop here would obscure the intent.
    let mut percentiles = vec![vec![0.0_f64; n_factors]; n_symbols];
    #[allow(clippy::needless_range_loop)]
    for f in 0..n_factors {
        let mut indexed: Vec<(usize, f64)> = (0..n_symbols)
            .map(|i| (i, symbols[i].factor_scores[f]))
            .collect();
        indexed.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        for (rank, (orig_idx, _)) in indexed.iter().enumerate() {
            percentiles[*orig_idx][f] = (rank + 1) as f64 / n_symbols as f64 * 100.0;
        }
    }
    // Composite = Σ w_f · percentile_f. Normalize by Σ|w_f| so the
    // result is interpretable as a weighted-pct.
    let weight_norm: f64 = factor_weights.iter().map(|w| w.abs()).sum();
    if weight_norm <= 0.0 {
        return None;
    }
    let mut composites: Vec<(usize, f64)> = (0..n_symbols)
        .map(|i| {
            let sum: f64 = (0..n_factors)
                .map(|f| factor_weights[f] * percentiles[i][f])
                .sum();
            (i, sum / weight_norm)
        })
        .collect();
    composites.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    let mut all_ranked: Vec<CompositeHit> = Vec::with_capacity(n_symbols);
    for (rank, (orig_idx, comp_score)) in composites.iter().enumerate() {
        let comp_pct = (rank + 1) as f64 / n_symbols as f64 * 100.0;
        let bucket = if comp_pct >= 90.0 {
            CompositeBucket::TopComposite
        } else if comp_pct <= 10.0 {
            CompositeBucket::BottomComposite
        } else {
            CompositeBucket::Neutral
        };
        all_ranked.push(CompositeHit {
            symbol: symbols[*orig_idx].symbol.clone(),
            composite_score: *comp_score,
            composite_percentile: comp_pct,
            bucket,
            per_factor_percentiles: percentiles[*orig_idx].clone(),
        });
    }
    // Sort all_ranked descending by composite score.
    all_ranked.sort_by(|a, b| {
        b.composite_score
            .partial_cmp(&a.composite_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let top_composite: Vec<CompositeHit> = all_ranked
        .iter()
        .filter(|h| h.bucket == CompositeBucket::TopComposite)
        .cloned()
        .collect();
    let bottom_composite: Vec<CompositeHit> = all_ranked
        .iter()
        .filter(|h| h.bucket == CompositeBucket::BottomComposite)
        .cloned()
        .collect();
    Some(CompositeReport {
        top_composite,
        bottom_composite,
        all_ranked,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(sym: &str, scores: Vec<f64>) -> SymbolFactorScores {
        SymbolFactorScores {
            symbol: sym.into(),
            factor_scores: scores,
        }
    }

    #[test]
    fn empty_returns_none() {
        assert!(score(&[], &[1.0]).is_none());
        assert!(score(&[s("X", vec![1.0])], &[]).is_none());
    }

    #[test]
    fn nan_or_dim_mismatch_returns_none() {
        let symbols = vec![s("X", vec![1.0, 2.0])];
        assert!(score(&symbols, &[f64::NAN, 1.0]).is_none());
        // Dim mismatch within symbol.
        let bad = vec![s("X", vec![1.0]), s("Y", vec![1.0, 2.0])];
        assert!(score(&bad, &[1.0, 1.0]).is_none());
    }

    #[test]
    fn zero_total_weight_returns_none() {
        let symbols = vec![s("X", vec![1.0, 2.0]); 2];
        assert!(score(&symbols, &[0.0, 0.0]).is_none());
    }

    #[test]
    fn equal_weights_yield_average_percentile() {
        // 10 symbols, 2 factors. Factor 1 ranks correlate; factor 2 same.
        // Top symbol on both → top composite.
        let symbols: Vec<_> = (1..=10)
            .map(|i| s(&format!("S{i:02}"), vec![i as f64, i as f64]))
            .collect();
        let r = score(&symbols, &[1.0, 1.0]).unwrap();
        assert_eq!(r.all_ranked[0].symbol, "S10");
        assert!(r.top_composite.iter().any(|h| h.symbol == "S10"));
    }

    #[test]
    fn negative_weight_inverts_factor_contribution() {
        // Factor 1 weighted +1, factor 2 weighted −1. Symbol that ranks
        // HIGH on factor 1 but LOW on factor 2 should get top composite.
        let symbols = vec![
            s("A", vec![10.0, 1.0]), // high f1, low f2 → top composite
            s("B", vec![5.0, 5.0]),
            s("C", vec![1.0, 10.0]), // low f1, high f2 → bottom composite
        ];
        let r = score(&symbols, &[1.0, -1.0]).unwrap();
        assert_eq!(r.all_ranked[0].symbol, "A");
        assert_eq!(r.all_ranked.last().unwrap().symbol, "C");
    }

    #[test]
    fn percentile_in_unit_range() {
        let symbols: Vec<_> = (1..=20)
            .map(|i| s(&format!("S{i:02}"), vec![i as f64, (20 - i) as f64]))
            .collect();
        let r = score(&symbols, &[0.5, 0.5]).unwrap();
        for h in &r.all_ranked {
            assert!((0.0..=100.0).contains(&h.composite_percentile));
        }
    }

    #[test]
    fn per_factor_percentiles_populated() {
        let symbols: Vec<_> = (1..=5)
            .map(|i| s(&format!("S{i}"), vec![i as f64, (10 - i) as f64]))
            .collect();
        let r = score(&symbols, &[1.0, 1.0]).unwrap();
        for h in &r.all_ranked {
            assert_eq!(h.per_factor_percentiles.len(), 2);
        }
    }
}
