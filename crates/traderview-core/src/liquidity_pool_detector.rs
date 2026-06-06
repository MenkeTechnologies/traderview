//! Liquidity Pool Detector — clustering of pivot highs/lows.
//!
//! Identifies "pools" of resting liquidity above prior swing highs and
//! below prior swing lows by counting how many pivot points cluster
//! within a price tolerance band:
//!
//!   For each pivot p:
//!     count_t = #{prior pivots q | |q.price - p.price| ≤ tolerance · p.price}
//!     pool_strength_t = count_t / lookback     (∈ [0, 1])
//!
//! Strong pools (high count) are stop-hunt targets: market often
//! probes them once price approaches because resting buy/sell stops
//! cluster just beyond.
//!
//! Caller supplies the pivot series (use `swing_points`). Pure compute.
//! Companion to `liquidity_grab`, `order_block`, `equal_levels`,
//! `stop_hunt`.

use serde::{Deserialize, Serialize};

pub use crate::gartley_pattern::Pivot;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LiquidityPool {
    pub price: f64,
    pub count: usize,
    pub strength: f64,
    /// `true` if the pool is from clustered swing-highs (resistance);
    /// `false` if swing-lows (support).
    pub is_resistance: bool,
}

pub fn detect(pivots: &[Pivot], tolerance_pct: f64, min_count: usize) -> Vec<LiquidityPool> {
    let mut out = Vec::new();
    if pivots.len() < 2 || !tolerance_pct.is_finite() || tolerance_pct <= 0.0 || min_count < 2 {
        return out;
    }
    let lookback = pivots.len() as f64;
    let tol_factor = tolerance_pct / 100.0;
    // For each pivot, count how many OTHER same-polarity pivots lie within band.
    for (i, p) in pivots.iter().enumerate() {
        if p.price <= 0.0 {
            continue;
        }
        let band = p.price * tol_factor;
        let mut cnt = 0_usize;
        for (j, q) in pivots.iter().enumerate() {
            if i == j {
                continue;
            }
            if q.is_high != p.is_high {
                continue;
            }
            if (q.price - p.price).abs() <= band {
                cnt += 1;
            }
        }
        if cnt + 1 >= min_count {
            out.push(LiquidityPool {
                price: p.price,
                count: cnt + 1,
                strength: ((cnt + 1) as f64) / lookback,
                is_resistance: p.is_high,
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(idx: usize, price: f64, is_high: bool) -> Pivot {
        Pivot {
            index: idx,
            price,
            is_high,
        }
    }

    #[test]
    fn empty_or_invalid_returns_empty() {
        assert!(detect(&[], 1.0, 2).is_empty());
        assert!(detect(&[p(0, 100.0, true)], 1.0, 2).is_empty());
        assert!(detect(&[p(0, 100.0, true), p(1, 100.0, true)], 0.0, 2).is_empty());
    }

    #[test]
    fn clustered_pivots_form_pool() {
        // 4 swing highs within 0.5% of each other → pool.
        let pivots = vec![
            p(0, 100.0, true),
            p(10, 100.3, true),
            p(20, 100.4, true),
            p(30, 100.2, true),
        ];
        let pools = detect(&pivots, 1.0, 3);
        assert!(!pools.is_empty());
        assert!(pools.iter().all(|pool| pool.is_resistance));
        assert!(pools.iter().all(|pool| pool.count >= 3));
    }

    #[test]
    fn scattered_pivots_no_pool() {
        let pivots = vec![
            p(0, 100.0, true),
            p(10, 110.0, true),
            p(20, 90.0, true),
            p(30, 120.0, true),
        ];
        let pools = detect(&pivots, 1.0, 3);
        assert!(pools.is_empty());
    }

    #[test]
    fn mixed_polarity_pools_separated() {
        // Highs clustered at 100 + lows clustered at 90 → two pools.
        let pivots = vec![
            p(0, 100.0, true),
            p(10, 100.3, true),
            p(20, 100.2, true),
            p(30, 90.0, false),
            p(40, 90.2, false),
            p(50, 89.9, false),
        ];
        let pools = detect(&pivots, 1.0, 3);
        let r_count = pools.iter().filter(|p| p.is_resistance).count();
        let s_count = pools.iter().filter(|p| !p.is_resistance).count();
        assert!(r_count >= 3);
        assert!(s_count >= 3);
    }

    #[test]
    fn tolerance_excludes_borderline_pivots() {
        // Tolerance 0.1% rejects pivots > 0.1% apart.
        let pivots = vec![
            p(0, 100.0, true),
            p(10, 100.5, true),  // 0.5% away → excluded
            p(20, 100.05, true), // 0.05% away → included
            p(30, 100.08, true),
        ];
        let pools = detect(&pivots, 0.1, 2);
        // The 100.5 pivot only finds itself → not a pool. Others cluster.
        let p_around_100 = pools
            .iter()
            .filter(|pool| (pool.price - 100.0).abs() < 0.1)
            .count();
        assert!(p_around_100 >= 1);
    }
}
