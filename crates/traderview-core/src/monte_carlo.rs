//! Monte Carlo trade-sequence simulator.
//!
//! Given a historical R-multiple distribution, draw with replacement
//! N synthetic equity curves of length L and report the distribution
//! of ending equity, max drawdown, and probability of ruin.
//!
//! This answers "given my system's edge, what's the realistic worst-
//! case curve across the next 200 trades?" — much more useful than
//! a single point estimate. Standard practice for system traders.
//!
//! Pure compute. Deterministic RNG (seed-controllable) so tests
//! reproduce exactly.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McConfig {
    /// How many synthetic curves to draw.
    pub n_curves: usize,
    /// How many trades per curve (typically the user's projected forward window).
    pub trades_per_curve: usize,
    /// Starting equity. Ruin = equity <= ruin_threshold at any point.
    pub start_equity: f64,
    pub ruin_threshold: f64,
    /// Fixed RNG seed for reproducibility. Caller passes time-based seed
    /// in production, fixed seed in tests.
    pub seed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct McReport {
    pub n_curves: usize,
    pub trades_per_curve: usize,
    pub start_equity: f64,
    /// Ending-equity percentiles (5, 25, 50, 75, 95).
    pub ending_equity_p05: f64,
    pub ending_equity_p25: f64,
    pub ending_equity_p50: f64,
    pub ending_equity_p75: f64,
    pub ending_equity_p95: f64,
    pub mean_ending_equity: f64,
    /// Max drawdown percentiles (5 = best case = smallest dd, 95 = worst case).
    pub max_drawdown_p05: f64,
    pub max_drawdown_p50: f64,
    pub max_drawdown_p95: f64,
    pub mean_max_drawdown: f64,
    /// Fraction of curves that hit `ruin_threshold` at any point.
    pub probability_of_ruin: f64,
    /// Fraction of curves that ended above start_equity.
    pub probability_profitable: f64,
}

pub fn simulate(historical_r: &[f64], cfg: &McConfig) -> Option<McReport> {
    if historical_r.is_empty() || cfg.n_curves == 0 || cfg.trades_per_curve == 0 {
        return None;
    }
    let mut rng = Lcg::new(cfg.seed);
    let mut ending = Vec::with_capacity(cfg.n_curves);
    let mut max_dds = Vec::with_capacity(cfg.n_curves);
    let mut ruin_count = 0usize;
    let mut profitable_count = 0usize;
    let len = historical_r.len();
    for _ in 0..cfg.n_curves {
        let mut equity = cfg.start_equity;
        let mut peak = equity;
        let mut max_dd_pct = 0.0_f64;
        let mut hit_ruin = false;
        for _ in 0..cfg.trades_per_curve {
            let idx = rng.next_bounded(len as u64) as usize;
            let r = historical_r[idx];
            equity += r;
            if equity > peak {
                peak = equity;
            }
            if equity <= cfg.ruin_threshold {
                hit_ruin = true;
            }
            let dd_pct = if peak > 0.0 {
                (peak - equity) / peak
            } else {
                0.0
            };
            if dd_pct > max_dd_pct {
                max_dd_pct = dd_pct;
            }
        }
        ending.push(equity);
        max_dds.push(max_dd_pct);
        if hit_ruin {
            ruin_count += 1;
        }
        if equity > cfg.start_equity {
            profitable_count += 1;
        }
    }
    ending.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    max_dds.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = ending.len() as f64;
    Some(McReport {
        n_curves: cfg.n_curves,
        trades_per_curve: cfg.trades_per_curve,
        start_equity: cfg.start_equity,
        ending_equity_p05: pct(&ending, 0.05),
        ending_equity_p25: pct(&ending, 0.25),
        ending_equity_p50: pct(&ending, 0.50),
        ending_equity_p75: pct(&ending, 0.75),
        ending_equity_p95: pct(&ending, 0.95),
        mean_ending_equity: ending.iter().sum::<f64>() / n,
        max_drawdown_p05: pct(&max_dds, 0.05),
        max_drawdown_p50: pct(&max_dds, 0.50),
        max_drawdown_p95: pct(&max_dds, 0.95),
        mean_max_drawdown: max_dds.iter().sum::<f64>() / n,
        probability_of_ruin: ruin_count as f64 / n,
        probability_profitable: profitable_count as f64 / n,
    })
}

fn pct(sorted: &[f64], q: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((sorted.len() as f64 - 1.0) * q).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

/// Minimal Linear Congruential Generator. Standard MMIX constants — fast,
/// deterministic, suitable for Monte Carlo (NOT for crypto).
struct Lcg {
    state: u64,
}
impl Lcg {
    fn new(seed: u64) -> Self {
        // Ensure non-zero state so even seed=0 produces a stream.
        Self {
            state: seed.wrapping_add(0x9E3779B97F4A7C15),
        }
    }
    fn next_u64(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.state
    }
    fn next_bounded(&mut self, bound: u64) -> u64 {
        // Lemire's bounded-rand: unbiased modulo.
        let mut x = self.next_u64();
        let mut m = (x as u128) * (bound as u128);
        let mut l = m as u64;
        if l < bound {
            let t = bound.wrapping_neg() % bound;
            while l < t {
                x = self.next_u64();
                m = (x as u128) * (bound as u128);
                l = m as u64;
            }
        }
        (m >> 64) as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(n: usize, t: usize, seed: u64) -> McConfig {
        McConfig {
            n_curves: n,
            trades_per_curve: t,
            start_equity: 10_000.0,
            ruin_threshold: 5_000.0,
            seed,
        }
    }

    #[test]
    fn empty_history_returns_none() {
        assert!(simulate(&[], &cfg(100, 50, 42)).is_none());
    }

    #[test]
    fn zero_curves_returns_none() {
        assert!(simulate(&[1.0, -1.0], &cfg(0, 50, 42)).is_none());
    }

    #[test]
    fn deterministic_with_fixed_seed() {
        let h = vec![100.0, -50.0, 200.0, -150.0];
        let a = simulate(&h, &cfg(500, 100, 12345)).unwrap();
        let b = simulate(&h, &cfg(500, 100, 12345)).unwrap();
        assert_eq!(
            a.mean_ending_equity, b.mean_ending_equity,
            "same seed must produce identical results"
        );
        assert_eq!(a.probability_of_ruin, b.probability_of_ruin);
    }

    #[test]
    fn different_seeds_diverge() {
        let h = vec![100.0, -50.0, 200.0, -150.0];
        let a = simulate(&h, &cfg(500, 100, 1)).unwrap();
        let b = simulate(&h, &cfg(500, 100, 999)).unwrap();
        // Differ at the percentile level — overwhelmingly likely.
        assert_ne!(a.ending_equity_p50, b.ending_equity_p50);
    }

    #[test]
    fn always_positive_history_never_ruins() {
        let h = vec![100.0, 200.0, 50.0];
        let r = simulate(&h, &cfg(200, 50, 42)).unwrap();
        assert_eq!(r.probability_of_ruin, 0.0);
        assert_eq!(
            r.probability_profitable, 1.0,
            "ALL drawn curves must end > start when every R > 0"
        );
    }

    #[test]
    fn always_negative_history_certain_ruin() {
        // Each trade loses $200. From $10k it takes ~25 trades to hit ruin
        // threshold $5k. 50 trades > 25 → every curve ruined.
        let h = vec![-200.0];
        let r = simulate(&h, &cfg(100, 50, 42)).unwrap();
        assert_eq!(r.probability_of_ruin, 1.0);
        assert_eq!(r.probability_profitable, 0.0);
    }

    #[test]
    fn percentiles_monotonic_increasing() {
        let h = vec![100.0, -50.0, 75.0, -25.0];
        let r = simulate(&h, &cfg(1000, 100, 42)).unwrap();
        assert!(r.ending_equity_p05 <= r.ending_equity_p25);
        assert!(r.ending_equity_p25 <= r.ending_equity_p50);
        assert!(r.ending_equity_p50 <= r.ending_equity_p75);
        assert!(r.ending_equity_p75 <= r.ending_equity_p95);
        assert!(r.max_drawdown_p05 <= r.max_drawdown_p50);
        assert!(r.max_drawdown_p50 <= r.max_drawdown_p95);
    }

    #[test]
    fn mean_ending_close_to_expectancy_times_trades() {
        // Each draw averages 25 (mean of [100, -50, 75, -25] = 25).
        // 100 trades × 25 = +2500. Starting 10k → expected ending 12,500.
        let h = vec![100.0, -50.0, 75.0, -25.0];
        let r = simulate(&h, &cfg(2000, 100, 42)).unwrap();
        let expected = 10_000.0 + 25.0 * 100.0;
        let err = (r.mean_ending_equity - expected).abs() / expected;
        assert!(
            err < 0.03,
            "Monte Carlo mean should converge to expectancy: got {} expected {}",
            r.mean_ending_equity,
            expected
        );
    }
}
