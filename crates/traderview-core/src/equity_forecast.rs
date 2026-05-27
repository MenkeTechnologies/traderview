//! Monte Carlo equity curve forecast — bootstrap from historical R-multiples.
//!
//! For each simulated path: starting at `starting_equity`, take `num_trades`
//! steps where each step's outcome is `equity * risk_pct * R`, where R is
//! drawn with replacement from the historical R sample. Equity below zero
//! is clamped to zero (ruin) and the path freezes there for the remaining
//! steps so percentile bands stay meaningful at the bottom of the fan.
//!
//! Outputs at each step k = 1..num_trades:
//!   * mean equity, p5/p25/p50/p75/p95
//!
//! Plus two end-of-horizon probabilities:
//!   * `ruin_probability`        = P(`equity[k]` <= `ruin_threshold` for any k)
//!   * `double_probability`      = P(`equity[num_trades]` >= 2 × `starting_equity`)

use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastInput {
    pub r_samples: Vec<f64>,
    pub starting_equity: f64,
    pub risk_pct_per_trade: f64, // 0..1
    pub num_trades: usize,
    pub num_paths: usize,
    pub seed: Option<u64>,
    pub ruin_threshold_pct: Option<f64>, // ruin if equity / starting <= this (e.g. 0.5 = -50%)
}

#[derive(Debug, Clone, Serialize)]
pub struct Percentiles {
    pub p5: f64,
    pub p25: f64,
    pub p50: f64,
    pub p75: f64,
    pub p95: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct StepStats {
    pub step: usize,
    pub mean: f64,
    pub bands: Percentiles,
}

#[derive(Debug, Clone, Serialize)]
pub struct ForecastReport {
    pub samples_used: usize,
    pub paths: usize,
    pub steps: usize,
    pub starting_equity: f64,
    pub risk_pct_per_trade: f64,
    pub mean_r: f64,
    pub stdev_r: f64,
    pub steps_stats: Vec<StepStats>,
    pub final_bands: Percentiles,
    pub ruin_probability: f64,
    pub double_probability: f64,
    /// First few path samples for "spaghetti plot" if the frontend wants
    /// to overlay them — capped at 50.
    pub sample_paths: Vec<Vec<f64>>,
    pub ruin_threshold_pct: f64,
}

pub fn forecast(input: &ForecastInput) -> ForecastReport {
    let r_samples: Vec<f64> = input
        .r_samples
        .iter()
        .copied()
        .filter(|x| x.is_finite())
        .collect();
    if r_samples.is_empty() {
        return ForecastReport {
            samples_used: 0,
            paths: 0,
            steps: 0,
            starting_equity: input.starting_equity,
            risk_pct_per_trade: input.risk_pct_per_trade,
            mean_r: 0.0,
            stdev_r: 0.0,
            steps_stats: vec![],
            final_bands: Percentiles {
                p5: 0.0,
                p25: 0.0,
                p50: 0.0,
                p75: 0.0,
                p95: 0.0,
            },
            ruin_probability: 1.0,
            double_probability: 0.0,
            sample_paths: vec![],
            ruin_threshold_pct: input.ruin_threshold_pct.unwrap_or(0.5),
        };
    }
    let n = r_samples.len();
    let mean_r = r_samples.iter().sum::<f64>() / n as f64;
    let stdev_r = (r_samples.iter().map(|x| (x - mean_r).powi(2)).sum::<f64>() / n as f64).sqrt();

    let paths = input.num_paths.clamp(1, 50_000);
    let steps = input.num_trades.clamp(1, 5_000);
    let risk = input.risk_pct_per_trade.clamp(0.0, 1.0);
    let ruin_pct = input.ruin_threshold_pct.unwrap_or(0.5).clamp(0.0, 1.0);
    let ruin_level = input.starting_equity * ruin_pct;

    let mut rng: StdRng = match input.seed {
        Some(s) => StdRng::seed_from_u64(s),
        None => StdRng::seed_from_u64(rand::thread_rng().gen()),
    };

    // equity_at[k][p] = equity at step k for path p. To keep memory
    // bounded, transpose: keep one column-per-step Vec for percentile work.
    let mut step_columns: Vec<Vec<f64>> = (0..steps).map(|_| Vec::with_capacity(paths)).collect();
    let mut ruin_count = 0usize;
    let mut double_count = 0usize;
    let mut sample_paths: Vec<Vec<f64>> = Vec::new();

    for p in 0..paths {
        let mut equity = input.starting_equity;
        let mut path = if p < 50 {
            Vec::with_capacity(steps + 1)
        } else {
            Vec::new()
        };
        if p < 50 {
            path.push(equity);
        }
        let mut hit_ruin = false;
        for col in step_columns.iter_mut().take(steps) {
            if !hit_ruin {
                let r_idx = rng.gen_range(0..n);
                let r = r_samples[r_idx];
                let dollar_delta = equity * risk * r;
                equity = (equity + dollar_delta).max(0.0);
                if equity <= ruin_level {
                    hit_ruin = true;
                    equity = 0.0;
                }
            }
            col.push(equity);
            if p < 50 {
                path.push(equity);
            }
        }
        if hit_ruin {
            ruin_count += 1;
        }
        if equity >= 2.0 * input.starting_equity {
            double_count += 1;
        }
        if p < 50 {
            sample_paths.push(path);
        }
    }

    // Per-step percentiles + mean.
    let mut steps_stats: Vec<StepStats> = Vec::with_capacity(steps);
    for (k, col) in step_columns.iter_mut().enumerate().take(steps) {
        let mean = col.iter().sum::<f64>() / col.len() as f64;
        col.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        steps_stats.push(StepStats {
            step: k + 1,
            mean,
            bands: pct(col),
        });
    }
    let final_bands = steps_stats
        .last()
        .map(|s| s.bands.clone())
        .unwrap_or(Percentiles {
            p5: 0.0,
            p25: 0.0,
            p50: 0.0,
            p75: 0.0,
            p95: 0.0,
        });

    ForecastReport {
        samples_used: n,
        paths,
        steps,
        starting_equity: input.starting_equity,
        risk_pct_per_trade: risk,
        mean_r,
        stdev_r,
        steps_stats,
        final_bands,
        ruin_probability: ruin_count as f64 / paths as f64,
        double_probability: double_count as f64 / paths as f64,
        sample_paths,
        ruin_threshold_pct: ruin_pct,
    }
}

fn pct(sorted: &[f64]) -> Percentiles {
    let q = |frac: f64| -> f64 {
        let n = sorted.len();
        if n == 0 {
            return 0.0;
        }
        let idx = ((n - 1) as f64 * frac).round() as usize;
        sorted[idx.min(n - 1)]
    };
    Percentiles {
        p5: q(0.05),
        p25: q(0.25),
        p50: q(0.50),
        p75: q(0.75),
        p95: q(0.95),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positive_expectancy_produces_growing_median() {
        // All wins: every R is +1. Median equity must grow monotonically.
        let inp = ForecastInput {
            r_samples: vec![1.0; 20],
            starting_equity: 10_000.0,
            risk_pct_per_trade: 0.01,
            num_trades: 100,
            num_paths: 200,
            seed: Some(42),
            ruin_threshold_pct: Some(0.5),
        };
        let r = forecast(&inp);
        assert_eq!(r.steps_stats.len(), 100);
        let last = r.steps_stats.last().unwrap();
        assert!(
            last.bands.p50 > inp.starting_equity,
            "p50 should grow but is {}",
            last.bands.p50
        );
        assert!(r.ruin_probability < 0.05, "all-wins should not ruin");
    }

    #[test]
    fn losing_system_triggers_ruin() {
        // All losses: every R is -1, 5% risk → 20 trades to lose all.
        let inp = ForecastInput {
            r_samples: vec![-1.0; 10],
            starting_equity: 10_000.0,
            risk_pct_per_trade: 0.05,
            num_trades: 200,
            num_paths: 100,
            seed: Some(7),
            ruin_threshold_pct: Some(0.5),
        };
        let r = forecast(&inp);
        assert!(
            r.ruin_probability >= 0.95,
            "all-loss system should ruin nearly always, got {}",
            r.ruin_probability
        );
        assert_eq!(r.double_probability, 0.0);
    }

    #[test]
    fn percentile_ordering_holds() {
        // Mixed sample. Percentiles must be ordered p5 <= p25 <= p50 <= p75 <= p95.
        let inp = ForecastInput {
            r_samples: vec![-2.0, -1.0, 0.0, 1.0, 2.0, 3.0],
            starting_equity: 10_000.0,
            risk_pct_per_trade: 0.02,
            num_trades: 50,
            num_paths: 500,
            seed: Some(123),
            ruin_threshold_pct: Some(0.5),
        };
        let r = forecast(&inp);
        for s in &r.steps_stats {
            let b = &s.bands;
            assert!(b.p5 <= b.p25 + 1e-9);
            assert!(b.p25 <= b.p50 + 1e-9);
            assert!(b.p50 <= b.p75 + 1e-9);
            assert!(b.p75 <= b.p95 + 1e-9);
        }
    }
}
