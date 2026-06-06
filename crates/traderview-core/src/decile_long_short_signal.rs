//! Decile Long-Short Signal Construction.
//!
//! Standard quant cross-sectional backtest building block. Sort the
//! universe by a signal, divide into K equal-sized buckets (deciles
//! when K = 10), and form a long-short portfolio that is:
//!
//!   - LONG the top-bucket names (equally weighted)
//!   - SHORT the bottom-bucket names (equally weighted)
//!   - NEUTRAL on the middle buckets
//!
//! Realized long-short return:
//!
//!   r_ls = mean(realized_returns | top) − mean(realized_returns | bottom)
//!
//! Diagnostic outputs (per bucket):
//!   - n_names per bucket
//!   - mean signal in the bucket
//!   - mean realized return (out-of-sample forward)
//!   - cumulative long-short return when iterated across dates
//!
//! Pure compute. Companion to `information_coefficient`,
//! `composite_factor_scoring`, `factor_neutralization`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NameRecord {
    pub symbol: String,
    pub signal: f64,
    pub realized_return: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BucketStats {
    pub bucket_index: usize,
    pub n_names: usize,
    pub mean_signal: f64,
    pub mean_realized_return: f64,
    pub symbols: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DecileLongShortReport {
    pub bucket_stats: Vec<BucketStats>,
    pub long_minus_short_return: f64,
    pub top_bucket_return: f64,
    pub bottom_bucket_return: f64,
    pub n_buckets: usize,
    pub n_names_total: usize,
}

pub fn build(names: &[NameRecord], n_buckets: usize) -> Option<DecileLongShortReport> {
    if names.len() < n_buckets * 2 || n_buckets < 2 {
        return None;
    }
    if names
        .iter()
        .any(|n| !n.signal.is_finite() || !n.realized_return.is_finite())
    {
        return None;
    }
    // Sort ascending by signal.
    let mut sorted = names.to_vec();
    sorted.sort_by(|a, b| {
        a.signal
            .partial_cmp(&b.signal)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let n = sorted.len();
    let mut bucket_stats = Vec::with_capacity(n_buckets);
    // Bucket k: indices [k·n/K, (k+1)·n/K).
    for k in 0..n_buckets {
        let lo = k * n / n_buckets;
        let hi = (k + 1) * n / n_buckets;
        let slice = &sorted[lo..hi];
        let n_k = slice.len();
        let nf = n_k as f64;
        let mean_sig: f64 = slice.iter().map(|r| r.signal).sum::<f64>() / nf;
        let mean_ret: f64 = slice.iter().map(|r| r.realized_return).sum::<f64>() / nf;
        let symbols: Vec<String> = slice.iter().map(|r| r.symbol.clone()).collect();
        bucket_stats.push(BucketStats {
            bucket_index: k,
            n_names: n_k,
            mean_signal: mean_sig,
            mean_realized_return: mean_ret,
            symbols,
        });
    }
    let top_ret = bucket_stats[n_buckets - 1].mean_realized_return;
    let bottom_ret = bucket_stats[0].mean_realized_return;
    let ls_ret = top_ret - bottom_ret;
    Some(DecileLongShortReport {
        bucket_stats,
        long_minus_short_return: ls_ret,
        top_bucket_return: top_ret,
        bottom_bucket_return: bottom_ret,
        n_buckets,
        n_names_total: n,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(sym: &str, sig: f64, ret: f64) -> NameRecord {
        NameRecord {
            symbol: sym.into(),
            signal: sig,
            realized_return: ret,
        }
    }

    #[test]
    fn too_few_names_returns_none() {
        let names: Vec<_> = (0..5)
            .map(|i| r(&format!("S{i}"), i as f64, 0.01))
            .collect();
        assert!(build(&names, 10).is_none());
    }

    #[test]
    fn nan_input_returns_none() {
        let names = vec![r("A", f64::NAN, 0.01), r("B", 2.0, 0.02)];
        assert!(build(&names, 2).is_none());
    }

    #[test]
    fn invalid_n_buckets_returns_none() {
        let names: Vec<_> = (0..20)
            .map(|i| r(&format!("S{i}"), i as f64, 0.01))
            .collect();
        assert!(build(&names, 1).is_none());
    }

    #[test]
    fn perfect_signal_yields_positive_ls() {
        // signal_i = realized_i → top bucket has highest forward returns.
        let names: Vec<_> = (1..=50)
            .map(|i| r(&format!("S{i:02}"), i as f64, i as f64 / 100.0))
            .collect();
        let result = build(&names, 5).unwrap();
        assert!(result.long_minus_short_return > 0.0);
        assert!(result.top_bucket_return > result.bottom_bucket_return);
    }

    #[test]
    fn anti_signal_yields_negative_ls() {
        // signal_i = -realized_i → top bucket is the worst-return names.
        let names: Vec<_> = (1..=50)
            .map(|i| r(&format!("S{i:02}"), i as f64, (50 - i) as f64 / 100.0))
            .collect();
        let result = build(&names, 5).unwrap();
        assert!(result.long_minus_short_return < 0.0);
    }

    #[test]
    fn buckets_sized_evenly_when_divisible() {
        let names: Vec<_> = (1..=50)
            .map(|i| r(&format!("S{i:02}"), i as f64, 0.01))
            .collect();
        let result = build(&names, 5).unwrap();
        for b in &result.bucket_stats {
            assert_eq!(b.n_names, 10);
        }
    }

    #[test]
    fn bucket_mean_signal_monotone() {
        // Sorted by signal → bucket k has higher mean than bucket k-1.
        let names: Vec<_> = (1..=100)
            .map(|i| r(&format!("S{i:03}"), i as f64, 0.01))
            .collect();
        let result = build(&names, 10).unwrap();
        for w in result.bucket_stats.windows(2) {
            assert!(w[1].mean_signal > w[0].mean_signal);
        }
    }

    #[test]
    fn bucket_symbols_populated() {
        let names: Vec<_> = (1..=20)
            .map(|i| r(&format!("S{i:02}"), i as f64, 0.01))
            .collect();
        let result = build(&names, 5).unwrap();
        for b in &result.bucket_stats {
            assert_eq!(b.symbols.len(), b.n_names);
        }
    }

    #[test]
    fn report_metadata_correct() {
        let names: Vec<_> = (1..=50)
            .map(|i| r(&format!("S{i:02}"), i as f64, 0.01))
            .collect();
        let result = build(&names, 5).unwrap();
        assert_eq!(result.n_buckets, 5);
        assert_eq!(result.n_names_total, 50);
    }
}
