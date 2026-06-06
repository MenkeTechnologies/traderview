//! Iceberg Order Detector — refreshes at the same price level.
//!
//! Iceberg orders display only a small portion of their full size and
//! "refresh" the displayed quantity each time it fills. Detected
//! heuristically by:
//!   - Multiple trades at the same price within a tight time window
//!   - Total volume at that price > normal bar volume × multiplier
//!   - Number of fills at that price ≥ min_fills threshold
//!
//! Caller supplies per-trade prints (price + size + timestamp seconds).
//! Returned tuples are (price_level, total_volume, fill_count) for
//! each detected iceberg.
//!
//! Pure compute. Defaults: price_tolerance_pct = 0.01, max_window_sec = 60,
//! min_fills = 5, vol_threshold = 1000.
//! Companion to `absorption_detector`, `liquidity_pool_detector`,
//! `weiss_wave`, `vsa`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Print {
    pub price: f64,
    pub size: f64,
    pub timestamp_sec: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct IcebergMatch {
    pub price_level: f64,
    pub total_volume: f64,
    pub fill_count: usize,
    pub first_ts_sec: f64,
    pub last_ts_sec: f64,
}

pub fn detect(
    prints: &[Print],
    price_tolerance_pct: f64,
    max_window_sec: f64,
    min_fills: usize,
    vol_threshold: f64,
) -> Vec<IcebergMatch> {
    let mut out = Vec::new();
    if prints.is_empty()
        || min_fills < 2
        || !price_tolerance_pct.is_finite()
        || price_tolerance_pct <= 0.0
        || !max_window_sec.is_finite()
        || max_window_sec <= 0.0
        || !vol_threshold.is_finite()
        || vol_threshold <= 0.0
    {
        return out;
    }
    if prints.iter().any(|p| {
        !p.price.is_finite()
            || !p.size.is_finite()
            || !p.timestamp_sec.is_finite()
            || p.price <= 0.0
            || p.size <= 0.0
    }) {
        return out;
    }
    let tol_factor = price_tolerance_pct / 100.0;
    let mut used = vec![false; prints.len()];
    for i in 0..prints.len() {
        if used[i] {
            continue;
        }
        let anchor = prints[i];
        let band = anchor.price * tol_factor;
        let mut total_vol = anchor.size;
        let mut count = 1_usize;
        let mut last_ts = anchor.timestamp_sec;
        let mut cluster_indices = vec![i];
        for j in (i + 1)..prints.len() {
            if used[j] {
                continue;
            }
            let p = prints[j];
            if (p.price - anchor.price).abs() > band {
                continue;
            }
            if p.timestamp_sec - anchor.timestamp_sec > max_window_sec {
                break;
            }
            total_vol += p.size;
            count += 1;
            last_ts = p.timestamp_sec;
            cluster_indices.push(j);
        }
        if count >= min_fills && total_vol >= vol_threshold {
            for idx in &cluster_indices {
                used[*idx] = true;
            }
            out.push(IcebergMatch {
                price_level: anchor.price,
                total_volume: total_vol,
                fill_count: count,
                first_ts_sec: anchor.timestamp_sec,
                last_ts_sec: last_ts,
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pr(price: f64, size: f64, ts: f64) -> Print {
        Print {
            price,
            size,
            timestamp_sec: ts,
        }
    }

    #[test]
    fn empty_or_invalid_returns_empty() {
        assert!(detect(&[], 0.01, 60.0, 5, 1000.0).is_empty());
        let prints = vec![pr(100.0, 100.0, 0.0); 10];
        assert!(detect(&prints, 0.0, 60.0, 5, 1000.0).is_empty());
        assert!(detect(&prints, 0.01, 0.0, 5, 1000.0).is_empty());
        assert!(detect(&prints, 0.01, 60.0, 1, 1000.0).is_empty());
    }

    #[test]
    fn nan_or_zero_returns_empty() {
        let prints = vec![pr(f64::NAN, 100.0, 0.0)];
        assert!(detect(&prints, 0.01, 60.0, 5, 1000.0).is_empty());
        let prints2 = vec![pr(100.0, 0.0, 0.0)];
        assert!(detect(&prints2, 0.01, 60.0, 5, 1000.0).is_empty());
    }

    #[test]
    fn iceberg_at_single_price_detected() {
        // 10 prints all at 100.00 within 30 sec, total volume 5000.
        let prints: Vec<_> = (0..10).map(|i| pr(100.0, 500.0, i as f64 * 3.0)).collect();
        let matches = detect(&prints, 0.01, 60.0, 5, 1000.0);
        assert_eq!(matches.len(), 1);
        assert!((matches[0].price_level - 100.0).abs() < 1e-9);
        assert!((matches[0].total_volume - 5000.0).abs() < 1e-9);
        assert_eq!(matches[0].fill_count, 10);
    }

    #[test]
    fn scattered_prints_no_iceberg() {
        let prints = vec![
            pr(100.0, 500.0, 0.0),
            pr(101.0, 500.0, 1.0),
            pr(99.0, 500.0, 2.0),
        ];
        let matches = detect(&prints, 0.01, 60.0, 5, 1000.0);
        assert!(matches.is_empty());
    }

    #[test]
    fn prints_outside_window_excluded() {
        // 10 prints at same price but spread over 1000 sec >> max_window.
        let prints: Vec<_> = (0..10)
            .map(|i| pr(100.0, 500.0, i as f64 * 200.0))
            .collect();
        let matches = detect(&prints, 0.01, 60.0, 5, 1000.0);
        // Each anchor only finds the bars within 60 sec — likely 0 fills besides self.
        assert!(matches.is_empty() || matches[0].fill_count < 5);
    }

    #[test]
    fn low_volume_iceberg_excluded() {
        // 10 prints at same price, but each only 10 size → 100 total < 1000.
        let prints: Vec<_> = (0..10).map(|i| pr(100.0, 10.0, i as f64 * 3.0)).collect();
        let matches = detect(&prints, 0.01, 60.0, 5, 1000.0);
        assert!(matches.is_empty());
    }
}
