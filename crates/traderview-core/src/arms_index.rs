//! Arms Index (TRIN) — Richard Arms's market-breadth indicator.
//!
//! Formula:
//!   `TRIN = (advancing_issues / declining_issues) / (advancing_volume / declining_volume)`
//!
//! Reading conventions (5-tier classification, see `TrinSignal`):
//!   - `TRIN < 0.5`           → StrongBuy   (advancing volume overwhelmingly dominant)
//!   - `0.5 ≤ TRIN < 0.8`     → Buy         (advancing volume disproportionately high)
//!   - `0.8 ≤ TRIN ≤ 1.2`     → Neutral     (balanced market)
//!   - `1.2 < TRIN ≤ 2.0`     → Sell        (declining volume disproportionately high)
//!   - `TRIN > 2.0`           → StrongSell  (declining volume overwhelmingly dominant)
//!
//! Inputs are per-bar breadth counts. Caller decides cadence
//! (typically daily or intraday market-wide).
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BreadthBar {
    pub advancing_issues: u64,
    pub declining_issues: u64,
    pub advancing_volume: f64,
    pub declining_volume: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TrinSignal {
    StrongBuy,
    Buy,
    #[default]
    Neutral,
    Sell,
    StrongSell,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TrinReport {
    pub series: Vec<Option<f64>>,
    pub latest: Option<f64>,
    pub signal: TrinSignal,
}

pub fn compute(bars: &[BreadthBar]) -> TrinReport {
    if bars.is_empty() {
        return TrinReport::default();
    }
    let series: Vec<Option<f64>> = bars
        .iter()
        .map(|b| {
            // Guard against division-by-zero AND NaN/Inf inputs. The naive
            // `<= 0.0` guards used to slip NaN through (every NaN comparison
            // is false), producing Some(NaN) in the report.
            if b.declining_issues == 0
                || !b.declining_volume.is_finite()
                || b.declining_volume <= 0.0
                || !b.advancing_volume.is_finite()
                || b.advancing_volume <= 0.0
            {
                return None;
            }
            let issues_ratio = b.advancing_issues as f64 / b.declining_issues as f64;
            let volume_ratio = b.advancing_volume / b.declining_volume;
            Some(issues_ratio / volume_ratio)
        })
        .collect();
    let latest = series.last().copied().flatten();
    let signal = match latest {
        Some(v) if v < 0.5 => TrinSignal::StrongBuy,
        Some(v) if v < 0.8 => TrinSignal::Buy,
        Some(v) if v > 2.0 => TrinSignal::StrongSell,
        Some(v) if v > 1.2 => TrinSignal::Sell,
        Some(_) => TrinSignal::Neutral,
        None => TrinSignal::Neutral,
    };
    TrinReport {
        series,
        latest,
        signal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bar(ai: u64, di: u64, av: f64, dv: f64) -> BreadthBar {
        BreadthBar {
            advancing_issues: ai,
            declining_issues: di,
            advancing_volume: av,
            declining_volume: dv,
        }
    }

    #[test]
    fn empty_returns_default() {
        let r = compute(&[]);
        assert!(r.latest.is_none());
        assert!(matches!(r.signal, TrinSignal::Neutral));
    }

    #[test]
    fn balanced_market_yields_unity() {
        // Equal issues, equal volume → TRIN = 1.0.
        let bars = vec![bar(1500, 1500, 100.0, 100.0)];
        let r = compute(&bars);
        assert!((r.latest.unwrap() - 1.0).abs() < 1e-9);
        assert!(matches!(r.signal, TrinSignal::Neutral));
    }

    #[test]
    fn strong_buying_volume_yields_low_trin() {
        // Issues balanced (1:1) but advance-volume is 4x decline-volume → TRIN = 0.25.
        let bars = vec![bar(1500, 1500, 400.0, 100.0)];
        let r = compute(&bars);
        let v = r.latest.unwrap();
        assert!((v - 0.25).abs() < 1e-9, "expected 0.25, got {v}");
        assert!(matches!(r.signal, TrinSignal::StrongBuy));
    }

    #[test]
    fn strong_selling_volume_yields_high_trin() {
        // Issues 2:1 advancing, but DECLINE-volume dominates 4:1 → TRIN = 2/0.25 = 8.0.
        let bars = vec![bar(2000, 1000, 100.0, 400.0)];
        let r = compute(&bars);
        let v = r.latest.unwrap();
        assert!((v - 8.0).abs() < 1e-9, "expected 8.0, got {v}");
        assert!(matches!(r.signal, TrinSignal::StrongSell));
    }

    #[test]
    fn zero_decliners_returns_none() {
        let bars = vec![bar(1500, 0, 100.0, 100.0)];
        assert!(compute(&bars).latest.is_none());
    }

    #[test]
    fn zero_volume_returns_none() {
        let bars = vec![bar(1500, 1500, 0.0, 100.0)];
        assert!(compute(&bars).latest.is_none());
        let bars = vec![bar(1500, 1500, 100.0, 0.0)];
        assert!(compute(&bars).latest.is_none());
    }

    #[test]
    fn nan_or_inf_volume_returns_none() {
        // Prior implementation let NaN slip through both guards because
        // `NaN <= 0.0` is false, producing Some(NaN) in the report.
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(
                compute(&[bar(1500, 1500, bad, 100.0)]).latest.is_none(),
                "advancing_volume={bad:?} should yield None"
            );
            assert!(
                compute(&[bar(1500, 1500, 100.0, bad)]).latest.is_none(),
                "declining_volume={bad:?} should yield None"
            );
        }
    }

    #[test]
    fn mid_range_classifies_neutral() {
        // TRIN = 1.0 with slight skew — Neutral.
        let bars = vec![bar(1500, 1500, 110.0, 100.0)]; // ratios 1.0 / 1.1 ≈ 0.91 → Neutral (>0.8, <1.2)
        let r = compute(&bars);
        let v = r.latest.unwrap();
        assert!(v > 0.8 && v < 1.2);
        assert!(matches!(r.signal, TrinSignal::Neutral));
    }

    /// Deterministic linear-congruential RNG so the fuzz test is reproducible
    /// and adds no dev-dependencies.
    struct Lcg(u64);
    impl Lcg {
        fn new(seed: u64) -> Self {
            Self(seed)
        }
        fn next_u64(&mut self) -> u64 {
            self.0 = self
                .0
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            self.0
        }
        fn f64_range(&mut self, lo: f64, hi: f64) -> f64 {
            let r = (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64;
            lo + (hi - lo) * r
        }
        fn pick_bad_f64(&mut self) -> f64 {
            match self.next_u64() % 6 {
                0 => f64::NAN,
                1 => f64::INFINITY,
                2 => f64::NEG_INFINITY,
                3 => -1.0e9,
                4 => 0.0,
                _ => self.f64_range(1.0, 1.0e9),
            }
        }
    }

    #[test]
    fn fuzz_compute_never_panics_or_emits_nonfinite() {
        let mut rng = Lcg::new(0xDEAD_BEEF_CAFE_F00D);
        for _ in 0..20_000 {
            let n = (rng.next_u64() % 64) as usize;
            let bars: Vec<BreadthBar> = (0..n)
                .map(|_| BreadthBar {
                    advancing_issues: rng.next_u64() % 1_000_000,
                    declining_issues: rng.next_u64() % 1_000_000,
                    advancing_volume: rng.pick_bad_f64(),
                    declining_volume: rng.pick_bad_f64(),
                })
                .collect();
            let r = compute(&bars);
            // Series length must match input.
            assert_eq!(r.series.len(), bars.len());
            // Every populated value must be a finite real (no NaN, no Inf).
            for (i, v) in r.series.iter().enumerate() {
                if let Some(x) = v {
                    assert!(
                        x.is_finite(),
                        "bar {i} produced non-finite TRIN {x} from bar {:?}",
                        bars[i]
                    );
                }
            }
            if let Some(v) = r.latest {
                assert!(v.is_finite(), "latest must be finite, got {v}");
            }
        }
    }

    #[test]
    fn multi_bar_series_preserved() {
        let bars = vec![
            bar(1500, 1500, 100.0, 100.0),
            bar(2000, 1000, 200.0, 50.0),
            bar(1000, 2000, 50.0, 200.0),
        ];
        let r = compute(&bars);
        assert_eq!(r.series.len(), 3);
        assert!((r.series[0].unwrap() - 1.0).abs() < 1e-9);
        // Final bar dominates `latest`.
        let v = r.latest.unwrap();
        assert!(v > 1.0, "selling bar → TRIN > 1, got {v}");
    }
}
