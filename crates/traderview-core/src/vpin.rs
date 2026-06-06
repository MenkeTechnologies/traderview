//! VPIN — Volume-synchronized Probability of Informed Trading.
//!
//! Easley, López de Prado, O'Hara (2012). Measures order-flow toxicity
//! using volume buckets (NOT clock buckets) — the rate at which buy /
//! sell volume becomes imbalanced over a fixed-volume bucket sequence.
//!
//! Procedure:
//!   1. Partition tick volume into equal-size buckets of `volume_per_bucket`.
//!   2. For each bucket, classify each tick's volume as buy / sell using
//!      the Bulk Volume Classification (BVC) rule on standardized
//!      log-returns (Δp / σ), via student-t CDF approximation (here
//!      simplified to a normal CDF; the difference is small for the
//!      tail behaviors most users care about).
//!   3. Compute per-bucket imbalance = |buy_vol − sell_vol|.
//!   4. VPIN_n = SMA of imbalance / volume_per_bucket over the last
//!      `window_buckets` buckets.
//!
//! High VPIN = toxic flow (informed traders running through the book).
//! Was famously used to flag the flash crash of 2010 minutes ahead.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Tick {
    pub price: f64,
    pub volume: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub volume_per_bucket: f64,
    pub window_buckets: usize,
    /// Rolling stdev period for BVC normalization (in ticks).
    pub return_window: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            volume_per_bucket: 50_000.0,
            window_buckets: 50,
            return_window: 100,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VpinReport {
    /// Per-bucket VPIN (None until window filled).
    pub vpin: Vec<Option<f64>>,
    pub bucket_buy_volume: Vec<f64>,
    pub bucket_sell_volume: Vec<f64>,
    /// Any bucket where vpin ≥ `toxic_threshold` (default 0.5).
    pub toxic_buckets: Vec<usize>,
}

pub fn compute(ticks: &[Tick], cfg: &Config) -> VpinReport {
    let mut report = VpinReport::default();
    if cfg.volume_per_bucket <= 0.0
        || !cfg.volume_per_bucket.is_finite()
        || cfg.window_buckets == 0
        || cfg.return_window == 0
        || ticks.is_empty()
    {
        return report;
    }
    // Step 1: rolling stdev of log-returns for BVC normalization.
    let n = ticks.len();
    let mut log_returns = vec![0.0_f64; n];
    for i in 1..n {
        if ticks[i].price.is_finite()
            && ticks[i - 1].price.is_finite()
            && ticks[i].price > 0.0
            && ticks[i - 1].price > 0.0
        {
            log_returns[i] = (ticks[i].price / ticks[i - 1].price).ln();
        }
    }
    // Rolling stdev (population) — guarded for short windows.
    let mut sigmas = vec![1.0_f64; n];
    let rw = cfg.return_window;
    if rw <= n {
        for i in (rw - 1)..n {
            let win = &log_returns[i + 1 - rw..=i];
            let mean = win.iter().sum::<f64>() / rw as f64;
            let var = win.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / rw as f64;
            let s = var.sqrt();
            if s.is_finite() && s > 0.0 {
                sigmas[i] = s;
            }
        }
    }
    // Step 2: walk ticks, fill buckets, BVC-classify volume.
    let mut buy = 0.0_f64;
    let mut sell = 0.0_f64;
    let mut bucket_vol = 0.0_f64;
    for (i, t) in ticks.iter().enumerate() {
        if !t.volume.is_finite() || t.volume < 0.0 {
            continue;
        }
        // BVC weight ω = Φ(Δp / σ); buy share = ω·v, sell share = (1−ω)·v.
        let omega = if i == 0 {
            0.5
        } else {
            let z = if sigmas[i] > 0.0 {
                log_returns[i] / sigmas[i]
            } else {
                0.0
            };
            norm_cdf(z)
        };
        let buy_share = omega * t.volume;
        let sell_share = (1.0 - omega) * t.volume;
        let mut remaining_buy = buy_share;
        let mut remaining_sell = sell_share;
        let mut remaining_total = t.volume;
        // Drain tick volume into buckets — handles ticks that span bucket boundaries.
        while remaining_total > 0.0 {
            let space = cfg.volume_per_bucket - bucket_vol;
            if space <= 0.0 {
                report.bucket_buy_volume.push(buy);
                report.bucket_sell_volume.push(sell);
                buy = 0.0;
                sell = 0.0;
                bucket_vol = 0.0;
                continue;
            }
            let take = remaining_total.min(space);
            let frac = take / remaining_total;
            let take_buy = remaining_buy * frac;
            let take_sell = remaining_sell * frac;
            buy += take_buy;
            sell += take_sell;
            bucket_vol += take;
            remaining_buy -= take_buy;
            remaining_sell -= take_sell;
            remaining_total -= take;
            if bucket_vol >= cfg.volume_per_bucket {
                report.bucket_buy_volume.push(buy);
                report.bucket_sell_volume.push(sell);
                buy = 0.0;
                sell = 0.0;
                bucket_vol = 0.0;
            }
        }
    }
    // Step 3: per-bucket imbalance fractions.
    let nbuckets = report.bucket_buy_volume.len();
    let imbalances: Vec<f64> = (0..nbuckets)
        .map(|i| {
            (report.bucket_buy_volume[i] - report.bucket_sell_volume[i]).abs()
                / cfg.volume_per_bucket
        })
        .collect();
    // Step 4: rolling SMA of imbalances over `window_buckets`.
    report.vpin = vec![None; nbuckets];
    let wb = cfg.window_buckets;
    if nbuckets >= wb {
        for i in (wb - 1)..nbuckets {
            let v = imbalances[i + 1 - wb..=i].iter().sum::<f64>() / wb as f64;
            if v.is_finite() {
                report.vpin[i] = Some(v);
                if v >= 0.5 {
                    report.toxic_buckets.push(i);
                }
            }
        }
    }
    report
}

fn norm_cdf(x: f64) -> f64 {
    // A&S approximation (max err 7.5e-8).
    let a1 = 0.254829592_f64;
    let a2 = -0.284496736_f64;
    let a3 = 1.421413741_f64;
    let a4 = -1.453152027_f64;
    let a5 = 1.061405429_f64;
    let p = 0.3275911_f64;
    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let xa = x.abs() / std::f64::consts::SQRT_2;
    let t = 1.0 / (1.0 + p * xa);
    let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-xa * xa).exp();
    0.5 * (1.0 + sign * y)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(p: f64, v: f64) -> Tick {
        Tick {
            price: p,
            volume: v,
        }
    }

    #[test]
    fn empty_returns_default() {
        let r = compute(&[], &Config::default());
        assert!(r.vpin.is_empty());
    }

    #[test]
    fn invalid_config_returns_default() {
        let ticks = vec![t(100.0, 1_000.0); 100];
        for cfg in [
            Config {
                volume_per_bucket: 0.0,
                ..Default::default()
            },
            Config {
                volume_per_bucket: f64::NAN,
                ..Default::default()
            },
            Config {
                window_buckets: 0,
                ..Default::default()
            },
            Config {
                return_window: 0,
                ..Default::default()
            },
        ] {
            assert!(
                compute(&ticks, &cfg).vpin.is_empty()
                    || compute(&ticks, &cfg).vpin.iter().all(|x| x.is_none())
            );
        }
    }

    #[test]
    fn buckets_aggregate_to_target_volume() {
        // 100 ticks × 100 vol = 10_000 total. Bucket = 1_000 → 10 buckets.
        let ticks: Vec<Tick> = (0..100)
            .map(|i| t(100.0 + i as f64 * 0.01, 100.0))
            .collect();
        let cfg = Config {
            volume_per_bucket: 1_000.0,
            window_buckets: 3,
            return_window: 10,
        };
        let r = compute(&ticks, &cfg);
        assert_eq!(r.bucket_buy_volume.len(), 10);
        for (b, s) in r.bucket_buy_volume.iter().zip(r.bucket_sell_volume.iter()) {
            assert!(
                (b + s - 1_000.0).abs() < 1e-6,
                "bucket sum should equal target"
            );
        }
    }

    #[test]
    fn monotonic_uptrend_produces_high_vpin() {
        // Strictly rising prices → log-return > 0 → BVC ω close to 1 → all-buy buckets → imbalance ≈ 1.
        let ticks: Vec<Tick> = (1..=500).map(|i| t(100.0 + i as f64, 100.0)).collect();
        let cfg = Config {
            volume_per_bucket: 500.0,
            window_buckets: 5,
            return_window: 20,
        };
        let r = compute(&ticks, &cfg);
        // The trailing VPIN should be high (toxic — informed buying).
        let last = r.vpin.iter().rev().find_map(|x| *x).expect("populated");
        assert!(
            last > 0.5,
            "monotonic uptrend should yield high VPIN, got {last}"
        );
    }

    #[test]
    fn flat_prices_yield_balanced_buckets() {
        // Constant price → log-return = 0 → ω = 0.5 → balanced → imbalance ≈ 0.
        let ticks: Vec<Tick> = (0..500).map(|_| t(100.0, 100.0)).collect();
        let cfg = Config {
            volume_per_bucket: 500.0,
            window_buckets: 5,
            return_window: 20,
        };
        let r = compute(&ticks, &cfg);
        let last = r.vpin.iter().rev().find_map(|x| *x).expect("populated");
        assert!(last < 0.05, "flat prices should yield low VPIN, got {last}");
    }

    #[test]
    fn nan_tick_skipped_safely() {
        let mut ticks: Vec<Tick> = (0..100).map(|_| t(100.0, 100.0)).collect();
        ticks[50].volume = f64::NAN;
        let cfg = Config {
            volume_per_bucket: 500.0,
            window_buckets: 3,
            return_window: 10,
        };
        let r = compute(&ticks, &cfg);
        // Just don't panic — buckets still produced.
        assert!(!r.bucket_buy_volume.is_empty());
    }
}
