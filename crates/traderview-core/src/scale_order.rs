//! Scale (ladder) order decomposition — split a target quantity into N
//! evenly-priced limit rungs across a band.
//!
//! A scale order is the standard way to build (or exit) a position
//! without committing to a single price: instead of one limit at $95,
//! place five at $90 / $92.50 / $95 / $97.50 / $100. This module is the
//! pure-compute core; the paper engine submits each rung through the
//! normal limit-order path, so every rung is risk-gated and fills like
//! any other limit.

use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize, PartialEq)]
pub struct Rung {
    pub price: f64,
    pub qty: f64,
}

/// Split `total_qty` into `rungs` limit orders evenly spaced in price
/// from `price_low` to `price_high` (inclusive). `rungs == 1` places a
/// single order at the band midpoint.
///
/// When `whole_units`, each rung gets a whole-number quantity and the
/// remainder (`total − Σfloor`) is handed out one unit at a time to the
/// rungs nearest `price_low` — the favorable end of a buy ladder — so
/// the rungs still sum to exactly `total_qty`. Otherwise the split is an
/// exact even fraction (for crypto/FX, where fractional size is native).
///
/// Returns `None` on a non-positive size, zero rungs, an inverted band
/// (`price_high < price_low`), a non-positive price, or — under
/// `whole_units` — fewer whole units than rungs (can't give each rung
/// at least one).
pub fn ladder(
    total_qty: f64,
    price_low: f64,
    price_high: f64,
    rungs: usize,
    whole_units: bool,
) -> Option<Vec<Rung>> {
    if rungs == 0
        || total_qty <= 0.0
        || price_low <= 0.0
        || price_high <= 0.0
        || price_high < price_low
    {
        return None;
    }

    let prices: Vec<f64> = if rungs == 1 {
        vec![(price_low + price_high) / 2.0]
    } else {
        let step = (price_high - price_low) / (rungs as f64 - 1.0);
        (0..rungs).map(|i| price_low + step * i as f64).collect()
    };

    let qtys: Vec<f64> = if whole_units {
        let total = total_qty.floor() as i64;
        let n = rungs as i64;
        if total < n {
            return None;
        }
        let base = total / n;
        let mut rem = total - base * n;
        (0..rungs)
            .map(|_| {
                let extra = if rem > 0 {
                    rem -= 1;
                    1
                } else {
                    0
                };
                (base + extra) as f64
            })
            .collect()
    } else {
        let per = total_qty / rungs as f64;
        vec![per; rungs]
    };

    Some(prices.into_iter().zip(qtys).map(|(price, qty)| Rung { price, qty }).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sum(rungs: &[Rung]) -> f64 {
        rungs.iter().map(|r| r.qty).sum()
    }

    #[test]
    fn even_fractional_split() {
        let l = ladder(100.0, 90.0, 100.0, 5, false).unwrap();
        assert_eq!(l.len(), 5);
        let prices: Vec<f64> = l.iter().map(|r| r.price).collect();
        assert_eq!(prices, vec![90.0, 92.5, 95.0, 97.5, 100.0]);
        for r in &l {
            assert!((r.qty - 20.0).abs() < 1e-9);
        }
        assert!((sum(&l) - 100.0).abs() < 1e-9);
    }

    #[test]
    fn single_rung_sits_at_midpoint() {
        let l = ladder(10.0, 90.0, 100.0, 1, false).unwrap();
        assert_eq!(l.len(), 1);
        assert!((l[0].price - 95.0).abs() < 1e-9);
        assert!((l[0].qty - 10.0).abs() < 1e-9);
    }

    #[test]
    fn whole_units_distribute_remainder_to_low_end() {
        // 103 units over 5 rungs: base 20, remainder 3 lands on the
        // first three rungs (the cheapest, favorable for a buy ladder).
        let l = ladder(103.0, 90.0, 100.0, 5, true).unwrap();
        let qtys: Vec<f64> = l.iter().map(|r| r.qty).collect();
        assert_eq!(qtys, vec![21.0, 21.0, 21.0, 20.0, 20.0]);
        assert!((sum(&l) - 103.0).abs() < 1e-9);
    }

    #[test]
    fn whole_units_exact_division() {
        let l = ladder(100.0, 90.0, 100.0, 5, true).unwrap();
        for r in &l {
            assert_eq!(r.qty, 20.0);
        }
    }

    #[test]
    fn whole_units_truncates_fractional_total() {
        // 100.9 whole units → 100 distributed; the 0.9 is dropped, not
        // a partial share.
        let l = ladder(100.9, 90.0, 100.0, 4, true).unwrap();
        assert!((sum(&l) - 100.0).abs() < 1e-9);
    }

    #[test]
    fn rejects_invalid_inputs() {
        assert!(ladder(100.0, 90.0, 100.0, 0, false).is_none()); // zero rungs
        assert!(ladder(0.0, 90.0, 100.0, 5, false).is_none()); // zero qty
        assert!(ladder(-5.0, 90.0, 100.0, 5, false).is_none()); // negative qty
        assert!(ladder(100.0, 100.0, 90.0, 5, false).is_none()); // inverted band
        assert!(ladder(100.0, 0.0, 100.0, 5, false).is_none()); // zero price
    }

    #[test]
    fn whole_units_needs_one_per_rung() {
        // 3 whole units can't fill 5 rungs.
        assert!(ladder(3.0, 90.0, 100.0, 5, true).is_none());
        // Exactly one per rung is fine.
        assert_eq!(sum(&ladder(5.0, 90.0, 100.0, 5, true).unwrap()), 5.0);
    }

    #[test]
    fn degenerate_band_all_same_price() {
        // price_low == price_high is a valid (flat) ladder.
        let l = ladder(50.0, 95.0, 95.0, 5, false).unwrap();
        for r in &l {
            assert!((r.price - 95.0).abs() < 1e-9);
        }
    }
}
