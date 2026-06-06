//! Rolling realized correlation matrix across N symbols.
//!
//! Given a (N × T) matrix of synchronized log returns, computes the
//! N × N Pearson correlation matrix over the most recent `window` bars
//! at each time t. Reports:
//!   - The correlation matrix at the latest bar
//!   - The mean off-diagonal correlation per bar (a single "correlation
//!     regime" indicator — high mean = crisis / panic; low mean = stock-
//!     picker's market)
//!   - The "max pair" — the two symbols whose correlation is highest in
//!     the latest window
//!
//! Pure compute. Caller supplies per-symbol return slices (all the same
//! length). Useful for risk-parity portfolio construction, regime
//! detection, and crash-correlation monitoring.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolReturns {
    pub symbol: String,
    pub returns: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CorrelationReport {
    pub symbols: Vec<String>,
    pub latest_correlation_matrix: Vec<Vec<f64>>,
    pub mean_off_diagonal_per_bar: Vec<Option<f64>>,
    pub latest_max_pair: Option<(String, String, f64)>,
    pub latest_min_pair: Option<(String, String, f64)>,
}

#[allow(clippy::needless_range_loop)] // matrix indexing — both i and j needed for symmetric reads
pub fn compute(series: &[SymbolReturns], window: usize) -> Option<CorrelationReport> {
    let n_sym = series.len();
    if n_sym < 2 || window < 2 {
        return None;
    }
    let t = series[0].returns.len();
    if t < window {
        return None;
    }
    if series.iter().any(|s| s.returns.len() != t) {
        return None;
    }
    let symbols: Vec<String> = series.iter().map(|s| s.symbol.clone()).collect();
    // Rolling correlation at each bar.
    let mut mean_off_diag = vec![None::<f64>; t];
    let mut latest_matrix = vec![vec![f64::NAN; n_sym]; n_sym];
    for end in (window - 1)..t {
        let lo = end + 1 - window;
        // Init to NaN so undefined pairs (e.g. constant series → stdev=0)
        // remain non-finite — the mean-off-diagonal aggregator then
        // correctly leaves the slot as None rather than averaging in 0.0.
        let mut corr = vec![vec![f64::NAN; n_sym]; n_sym];
        for i in 0..n_sym {
            corr[i][i] = 1.0;
            for j in (i + 1)..n_sym {
                if let Some(c) = pearson(&series[i].returns[lo..=end], &series[j].returns[lo..=end])
                {
                    corr[i][j] = c;
                    corr[j][i] = c;
                }
            }
        }
        // Mean off-diagonal — only when ALL pairs are valid (no NaN block).
        let mut sum = 0.0_f64;
        let mut count = 0_usize;
        let mut all_valid = true;
        for i in 0..n_sym {
            for j in (i + 1)..n_sym {
                if !corr[i][j].is_finite() {
                    all_valid = false;
                    break;
                }
                sum += corr[i][j];
                count += 1;
            }
            if !all_valid {
                break;
            }
        }
        if all_valid && count > 0 {
            mean_off_diag[end] = Some(sum / count as f64);
        }
        if end == t - 1 {
            latest_matrix = corr;
        }
    }
    // Max/min pair from latest matrix.
    let mut max_pair: Option<(usize, usize, f64)> = None;
    let mut min_pair: Option<(usize, usize, f64)> = None;
    for i in 0..n_sym {
        for j in (i + 1)..n_sym {
            let c = latest_matrix[i][j];
            if !c.is_finite() {
                continue;
            }
            if max_pair.is_none_or(|(_, _, bc)| c > bc) {
                max_pair = Some((i, j, c));
            }
            if min_pair.is_none_or(|(_, _, bc)| c < bc) {
                min_pair = Some((i, j, c));
            }
        }
    }
    Some(CorrelationReport {
        latest_correlation_matrix: latest_matrix,
        mean_off_diagonal_per_bar: mean_off_diag,
        latest_max_pair: max_pair.map(|(i, j, c)| (symbols[i].clone(), symbols[j].clone(), c)),
        latest_min_pair: min_pair.map(|(i, j, c)| (symbols[i].clone(), symbols[j].clone(), c)),
        symbols,
    })
}

fn pearson(a: &[f64], b: &[f64]) -> Option<f64> {
    let n = a.len();
    if n != b.len() || n < 2 {
        return None;
    }
    let mut sx = 0.0;
    let mut sy = 0.0;
    let mut sxx = 0.0;
    let mut syy = 0.0;
    let mut sxy = 0.0;
    let mut count = 0_usize;
    for (x, y) in a.iter().zip(b.iter()) {
        if !x.is_finite() || !y.is_finite() {
            continue;
        }
        sx += x;
        sy += y;
        sxx += x * x;
        syy += y * y;
        sxy += x * y;
        count += 1;
    }
    if count < 2 {
        return None;
    }
    let n_f = count as f64;
    let num = sxy - sx * sy / n_f;
    let denom = ((sxx - sx * sx / n_f) * (syy - sy * sy / n_f))
        .max(0.0)
        .sqrt();
    if denom <= 0.0 {
        return None;
    }
    let r = num / denom;
    if r.is_finite() {
        Some(r.clamp(-1.0, 1.0))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(sym: &str, r: Vec<f64>) -> SymbolReturns {
        SymbolReturns {
            symbol: sym.into(),
            returns: r,
        }
    }

    #[test]
    fn empty_returns_none() {
        assert!(compute(&[], 20).is_none());
    }

    #[test]
    fn single_symbol_returns_none() {
        assert!(compute(&[s("A", vec![0.0; 50])], 20).is_none());
    }

    #[test]
    fn dim_mismatch_returns_none() {
        assert!(compute(&[s("A", vec![0.0; 50]), s("B", vec![0.0; 20])], 10).is_none());
    }

    #[test]
    fn window_larger_than_input_returns_none() {
        assert!(compute(&[s("A", vec![0.0; 5]), s("B", vec![0.0; 5])], 10).is_none());
    }

    #[test]
    fn flat_returns_yield_undefined_off_diagonal() {
        // stdev = 0 → pearson denom = 0 → None → mean_off_diag not populated.
        let r = compute(&[s("A", vec![0.0; 50]), s("B", vec![0.0; 50])], 20).unwrap();
        assert!(r.mean_off_diagonal_per_bar.iter().all(|x| x.is_none()));
    }

    #[test]
    fn perfectly_correlated_pair_yields_one() {
        let r = compute(
            &[
                s("A", (0..100).map(|i| i as f64).collect()),
                s("B", (0..100).map(|i| 2.0 * i as f64 + 5.0).collect()),
            ],
            50,
        )
        .unwrap();
        let c = r.latest_correlation_matrix[0][1];
        assert!(
            (c - 1.0).abs() < 1e-9,
            "linear A↔B should have corr=1, got {c}"
        );
    }

    #[test]
    fn perfectly_anticorrelated_pair_yields_minus_one() {
        let r = compute(
            &[
                s("A", (0..100).map(|i| i as f64).collect()),
                s("B", (0..100).map(|i| -(i as f64)).collect()),
            ],
            50,
        )
        .unwrap();
        let c = r.latest_correlation_matrix[0][1];
        assert!((c + 1.0).abs() < 1e-9);
    }

    #[test]
    fn diagonal_is_one() {
        let r = compute(
            &[
                s("A", (0..50).map(|i| (i as f64 * 0.1).sin()).collect()),
                s("B", (0..50).map(|i| (i as f64 * 0.1).cos()).collect()),
            ],
            30,
        )
        .unwrap();
        for i in 0..r.latest_correlation_matrix.len() {
            assert_eq!(r.latest_correlation_matrix[i][i], 1.0);
        }
    }

    #[test]
    fn matrix_is_symmetric() {
        let r = compute(
            &[
                s("A", (0..50).map(|i| (i as f64 * 0.1).sin()).collect()),
                s("B", (0..50).map(|i| (i as f64 * 0.1).cos()).collect()),
                s("C", (0..50).map(|i| (i as f64 * 0.13).sin()).collect()),
            ],
            30,
        )
        .unwrap();
        let m = &r.latest_correlation_matrix;
        #[allow(clippy::needless_range_loop)] // need both i and j for symmetric check
        for i in 0..3 {
            for j in 0..3 {
                assert!((m[i][j] - m[j][i]).abs() < 1e-12);
            }
        }
    }

    #[test]
    fn max_pair_has_higher_corr_than_min_pair() {
        let r = compute(
            &[
                s("HOT", (0..100).map(|i| i as f64).collect()),
                s("COLD", (0..100).map(|i| -(i as f64)).collect()),
                s("MID", (0..100).map(|i| (i as f64 * 0.5).sin()).collect()),
            ],
            50,
        )
        .unwrap();
        let (_, _, max_c) = r.latest_max_pair.as_ref().unwrap();
        let (_, _, min_c) = r.latest_min_pair.as_ref().unwrap();
        assert!(max_c >= min_c);
    }

    #[test]
    fn nan_returns_skipped_per_pair() {
        let mut a = vec![0.0; 50];
        a[5] = f64::NAN;
        let r = compute(
            &[s("A", a), s("B", (0..50).map(|i| i as f64).collect())],
            30,
        )
        .unwrap();
        // Should produce a valid matrix despite NaN in A (skipped within
        // pearson()).
        assert_eq!(r.latest_correlation_matrix.len(), 2);
    }
}
