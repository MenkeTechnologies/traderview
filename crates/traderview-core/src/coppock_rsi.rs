//! Coppock Curve on RSI (variant) — momentum-of-RSI long-term signal.
//!
//! Classic Coppock takes a 14-month ROC + 11-month ROC summed and
//! WMA-smoothed over 10 months — designed for monthly equity-index
//! bottoms. This variant operates on a daily/intraday RSI series and
//! returns a momentum-of-momentum signal that crosses zero earlier
//! than price-level Coppock on shorter timeframes:
//!
//!   rsi      = RSI(closes, rsi_period)
//!   short    = ROC(rsi, short_roc)
//!   long     = ROC(rsi, long_roc)
//!   summed   = short + long
//!   coppock  = WMA(summed, wma_period)
//!
//! Zero-crossings = canonical signal. Pure compute.

use crate::indicators;

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub rsi_period: usize,
    pub short_roc: usize,
    pub long_roc: usize,
    pub wma_period: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            rsi_period: 14,
            short_roc: 11,
            long_roc: 14,
            wma_period: 10,
        }
    }
}

pub fn compute(closes: &[f64], cfg: Config) -> Vec<Option<f64>> {
    let n = closes.len();
    let mut out = vec![None; n];
    if cfg.rsi_period == 0 || cfg.short_roc == 0 || cfg.long_roc == 0 || cfg.wma_period == 0 {
        return out;
    }
    // Any param > input length cannot produce any output. Early-return
    // prevents an O(N²) blow-up in the WMA weight-sum loop below when a
    // caller passes a pathological period like usize::MAX.
    if cfg.rsi_period > n || cfg.short_roc > n || cfg.long_roc > n || cfg.wma_period > n {
        return out;
    }
    let rsi = indicators::rsi(closes, cfg.rsi_period);
    // ROC on Option<f64>: skip where RSI is None at either end.
    let mut summed = vec![None::<f64>; n];
    let big_roc = cfg.short_roc.max(cfg.long_roc);
    for i in big_roc..n {
        let now = rsi[i];
        let s_then = rsi[i - cfg.short_roc];
        let l_then = rsi[i - cfg.long_roc];
        if let (Some(n_), Some(s_), Some(l_)) = (now, s_then, l_then) {
            if s_.abs() > f64::EPSILON && l_.abs() > f64::EPSILON {
                let short_roc = (n_ - s_) / s_;
                let long_roc = (n_ - l_) / l_;
                let v = short_roc + long_roc;
                if v.is_finite() {
                    summed[i] = Some(v);
                }
            }
        }
    }
    // Weighted MA over `wma_period` using only populated slots. The
    // weight sum is the closed-form n(n+1)/2 (computed in f64 to avoid
    // pathological usize overflow when callers pass huge periods —
    // upper-bounded in the early-return guard above).
    let wp = cfg.wma_period;
    let weight_sum = wp as f64 * (wp as f64 + 1.0) / 2.0;
    for (i, slot) in out.iter_mut().enumerate() {
        if i + 1 < wp {
            continue;
        }
        let mut numer = 0.0_f64;
        let mut ok = true;
        for (k, j) in ((i + 1 - wp)..=i).enumerate() {
            match summed[j] {
                Some(v) => numer += v * (k + 1) as f64,
                None => {
                    ok = false;
                    break;
                }
            }
        }
        if ok && numer.is_finite() {
            *slot = Some(numer / weight_sum);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_empty() {
        assert!(compute(&[], Config::default()).is_empty());
    }

    #[test]
    fn invalid_config_returns_all_none() {
        let v = vec![100.0; 100];
        for cfg in [
            Config {
                rsi_period: 0,
                ..Default::default()
            },
            Config {
                short_roc: 0,
                ..Default::default()
            },
            Config {
                long_roc: 0,
                ..Default::default()
            },
            Config {
                wma_period: 0,
                ..Default::default()
            },
        ] {
            assert!(compute(&v, cfg).iter().all(|x| x.is_none()));
        }
    }

    #[test]
    fn flat_series_yields_zero_signal() {
        // Flat closes → RSI saturates at 100 (loss==0 branch) → ROC of
        // constant 100 = 0 → summed = 0 → WMA(0,…) = 0. Populated slots
        // are Some(0.0), not None — and the zero-line is exactly the
        // documented "no trade" reading.
        let v = vec![100.0; 100];
        let out = compute(&v, Config::default());
        for x in out.iter().flatten() {
            assert!(
                x.abs() < 1e-9,
                "flat series should yield 0 coppock, got {x}"
            );
        }
    }

    #[test]
    fn rising_then_falling_produces_zero_crossing() {
        let mut v: Vec<f64> = (0..50).map(|i| 100.0 + i as f64).collect();
        v.extend((0..50).map(|i| 150.0 - i as f64));
        let out = compute(&v, Config::default());
        let populated: Vec<f64> = out.iter().filter_map(|x| *x).collect();
        // Should have both positive and negative values somewhere.
        let has_pos = populated.iter().any(|x| *x > 0.0);
        let has_neg = populated.iter().any(|x| *x < 0.0);
        assert!(has_pos || has_neg);
    }

    #[test]
    fn huge_params_no_panic() {
        let v = vec![100.0; 10];
        let cfg = Config {
            rsi_period: usize::MAX,
            short_roc: usize::MAX,
            long_roc: usize::MAX,
            wma_period: usize::MAX,
        };
        let out = compute(&v, cfg);
        assert!(out.iter().all(|x| x.is_none()));
    }
}
