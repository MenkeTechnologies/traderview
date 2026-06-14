//! Retirement decumulation Monte Carlo — simulates a portfolio drawn down in
//! retirement to estimate the probability the money lasts. Each year the balance
//! grows by a normally-distributed return and an inflation-adjusted withdrawal is
//! taken; a path "succeeds" if the balance stays positive through the horizon. It
//! reports the success rate and the 10th/50th/90th-percentile ending balances.
//! The simulation is fully deterministic — a fixed-seed xorshift64* PRNG with
//! Box-Muller normals — so results are reproducible. Not financial advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct DecumulationInput {
    pub initial_balance_usd: f64,
    /// First-year annual withdrawal (grows with inflation thereafter).
    pub annual_withdrawal_usd: f64,
    /// Expected annual real... nominal return mean, percent.
    pub mean_return_pct: f64,
    /// Annual return volatility, percent.
    pub volatility_pct: f64,
    /// Annual inflation, percent (escalates the withdrawal).
    #[serde(default)]
    pub inflation_pct: f64,
    pub years: u32,
    #[serde(default = "default_sims")]
    pub simulations: u32,
    /// PRNG seed (fixed default for reproducibility).
    #[serde(default = "default_seed")]
    pub seed: u64,
}

fn default_sims() -> u32 {
    2000
}

fn default_seed() -> u64 {
    0x9E37_79B9_7F4A_7C15
}

#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct DecumulationReport {
    pub success_rate_pct: f64,
    pub p10_ending_balance_usd: f64,
    pub median_ending_balance_usd: f64,
    pub p90_ending_balance_usd: f64,
    pub simulations: u32,
}

/// xorshift64* — deterministic, fast, well-distributed.
struct Rng(u64);

impl Rng {
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    /// Uniform in [0, 1) using the top 53 bits.
    fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }
    /// Standard normal via Box-Muller.
    fn next_normal(&mut self) -> f64 {
        let u1 = self.next_f64().max(1e-12);
        let u2 = self.next_f64();
        (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
    }
}

fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

pub fn generate(i: &DecumulationInput) -> DecumulationReport {
    let sims = i.simulations.max(1);
    let seed = if i.seed == 0 { default_seed() } else { i.seed };
    let mut rng = Rng(seed);
    let mean = i.mean_return_pct / 100.0;
    let vol = i.volatility_pct / 100.0;
    let infl = i.inflation_pct / 100.0;

    let mut successes = 0u32;
    let mut endings: Vec<f64> = Vec::with_capacity(sims as usize);

    for _ in 0..sims {
        let mut bal = i.initial_balance_usd;
        let mut alive = true;
        for y in 1..=i.years {
            let ret = mean + vol * rng.next_normal();
            bal *= 1.0 + ret;
            bal -= i.annual_withdrawal_usd * (1.0 + infl).powi((y - 1) as i32);
            if bal <= 0.0 {
                alive = false;
                bal = 0.0;
                break;
            }
        }
        if alive && bal > 0.0 {
            successes += 1;
        }
        endings.push(bal);
    }

    endings.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let pctile = |p: f64| -> f64 {
        let idx = ((p / 100.0 * endings.len() as f64) as usize).min(endings.len().saturating_sub(1));
        endings[idx]
    };

    DecumulationReport {
        success_rate_pct: round2(successes as f64 / sims as f64 * 100.0),
        p10_ending_balance_usd: pctile(10.0).round(),
        median_ending_balance_usd: pctile(50.0).round(),
        p90_ending_balance_usd: pctile(90.0).round(),
        simulations: sims,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> DecumulationInput {
        DecumulationInput {
            initial_balance_usd: 1_000_000.0,
            annual_withdrawal_usd: 40_000.0,
            mean_return_pct: 6.0,
            volatility_pct: 12.0,
            inflation_pct: 2.5,
            years: 30,
            simulations: 2000,
            seed: default_seed(),
        }
    }

    #[test]
    fn deterministic_success_rate() {
        let d = generate(&base());
        // Matches an independent Python implementation of the same PRNG/algorithm.
        assert!((d.success_rate_pct - 72.75).abs() < 0.5);
        assert!((d.median_ending_balance_usd - 832_309.0).abs() < 5_000.0);
    }

    #[test]
    fn reproducible_same_seed() {
        let a = generate(&base());
        let b = generate(&base());
        assert_eq!(a, b);
    }

    #[test]
    fn higher_withdrawal_lower_success() {
        let low = generate(&base());
        let high = generate(&DecumulationInput { annual_withdrawal_usd: 70_000.0, ..base() });
        assert!(high.success_rate_pct < low.success_rate_pct);
    }

    #[test]
    fn higher_return_higher_success() {
        let lo = generate(&base());
        let hi = generate(&DecumulationInput { mean_return_pct: 9.0, ..base() });
        assert!(hi.success_rate_pct > lo.success_rate_pct);
    }

    #[test]
    fn percentiles_ordered() {
        let d = generate(&base());
        assert!(d.p10_ending_balance_usd <= d.median_ending_balance_usd);
        assert!(d.median_ending_balance_usd <= d.p90_ending_balance_usd);
    }
}
