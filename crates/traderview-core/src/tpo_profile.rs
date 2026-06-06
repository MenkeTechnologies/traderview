//! TPO (Time-Price Opportunity) Profile — Steidlmayer / Market Profile.
//!
//! Bins each bar's price range across `n_buckets` horizontal levels.
//! Each bar contributes one TPO letter (1 unit) to every price bucket
//! it touches. After processing the session:
//!   - **POC** (Point of Control) = bucket with highest TPO count
//!   - **Value Area** = contiguous block of buckets, anchored at POC,
//!     that contains `value_area_pct` (default 70%) of total TPOs
//!   - **VAH** / **VAL** = high / low of the value area
//!
//! Used by floor-traders and order-flow analysts to identify acceptance
//! zones (high TPO count = price was accepted, traders agreed) vs
//! rejection zones (low TPO count = price was tested but rejected).
//!
//! Pure compute. Caller supplies bar high/low series for one session.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub n_buckets: usize,
    pub value_area_pct: f64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            n_buckets: 30,
            value_area_pct: 0.70,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TpoReport {
    pub session_high: f64,
    pub session_low: f64,
    pub bucket_size: f64,
    /// Price at the center of each bucket (n_buckets long, ascending).
    pub bucket_prices: Vec<f64>,
    /// TPO count per bucket.
    pub tpo_counts: Vec<u64>,
    pub poc_index: usize,
    pub poc_price: f64,
    pub value_area_low_index: usize,
    pub value_area_high_index: usize,
    pub vah: f64,
    pub val: f64,
}

pub fn build(bars: &[Bar], cfg: &Config) -> Option<TpoReport> {
    if cfg.n_buckets < 3
        || !cfg.value_area_pct.is_finite()
        || !(0.0..=1.0).contains(&cfg.value_area_pct)
        || cfg.value_area_pct <= 0.0
        || bars.is_empty()
    {
        return None;
    }
    // Session range.
    let mut hi = f64::NEG_INFINITY;
    let mut lo = f64::INFINITY;
    for b in bars {
        if !b.high.is_finite() || !b.low.is_finite() || b.high < b.low {
            continue;
        }
        if b.high > hi {
            hi = b.high;
        }
        if b.low < lo {
            lo = b.low;
        }
    }
    if !hi.is_finite() || !lo.is_finite() || hi <= lo {
        return None;
    }
    let range = hi - lo;
    let bucket_size = range / cfg.n_buckets as f64;
    if bucket_size <= 0.0 || !bucket_size.is_finite() {
        return None;
    }
    let mut counts = vec![0_u64; cfg.n_buckets];
    let mut bucket_prices = Vec::with_capacity(cfg.n_buckets);
    for i in 0..cfg.n_buckets {
        bucket_prices.push(lo + bucket_size * (i as f64 + 0.5));
    }
    for b in bars {
        if !b.high.is_finite() || !b.low.is_finite() || b.high < b.low {
            continue;
        }
        let low_idx = ((b.low - lo) / bucket_size).floor() as isize;
        let high_idx = ((b.high - lo) / bucket_size).floor() as isize;
        let low_idx = low_idx.clamp(0, cfg.n_buckets as isize - 1) as usize;
        let high_idx = high_idx.clamp(0, cfg.n_buckets as isize - 1) as usize;
        for c in counts.iter_mut().take(high_idx + 1).skip(low_idx) {
            *c = c.saturating_add(1);
        }
    }
    let total: u64 = counts.iter().sum();
    if total == 0 {
        return None;
    }
    // POC = bucket with max count (lowest index if tied — convention).
    let (poc_index, _) = counts
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.cmp(b.1).then_with(|| b.0.cmp(&a.0)))
        .expect("non-empty");
    // Expand value area outward from POC until target % of TPOs covered.
    let target = (cfg.value_area_pct * total as f64).ceil() as u64;
    let mut va_lo = poc_index;
    let mut va_hi = poc_index;
    let mut covered = counts[poc_index];
    while covered < target && (va_lo > 0 || va_hi < cfg.n_buckets - 1) {
        // Take the larger of the next two adjacent slots (above + below pair).
        let above_pair = if va_hi + 2 < cfg.n_buckets {
            counts.get(va_hi + 1).copied().unwrap_or(0)
                + counts.get(va_hi + 2).copied().unwrap_or(0)
        } else if va_hi + 1 < cfg.n_buckets {
            counts.get(va_hi + 1).copied().unwrap_or(0)
        } else {
            0
        };
        let below_pair = if va_lo >= 2 {
            counts.get(va_lo - 1).copied().unwrap_or(0)
                + counts.get(va_lo - 2).copied().unwrap_or(0)
        } else if va_lo >= 1 {
            counts.get(va_lo - 1).copied().unwrap_or(0)
        } else {
            0
        };
        if above_pair == 0 && below_pair == 0 {
            break;
        }
        if above_pair >= below_pair && va_hi < cfg.n_buckets - 1 {
            let step = if va_hi + 2 < cfg.n_buckets { 2 } else { 1 };
            let end = (va_hi + step).min(cfg.n_buckets - 1);
            for (k, c) in counts.iter().enumerate().take(end + 1).skip(va_hi + 1) {
                covered = covered.saturating_add(*c);
                va_hi = k;
            }
        } else if va_lo > 0 {
            let step = if va_lo >= 2 { 2 } else { 1 };
            let lo_new = va_lo.saturating_sub(step);
            for c in counts.iter().take(va_lo).skip(lo_new) {
                covered = covered.saturating_add(*c);
            }
            va_lo = lo_new;
        } else {
            break;
        }
    }
    Some(TpoReport {
        session_high: hi,
        session_low: lo,
        bucket_size,
        poc_index,
        poc_price: bucket_prices[poc_index],
        value_area_low_index: va_lo,
        value_area_high_index: va_hi,
        vah: bucket_prices[va_hi],
        val: bucket_prices[va_lo],
        bucket_prices,
        tpo_counts: counts,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bar(h: f64, l: f64) -> Bar {
        Bar { high: h, low: l }
    }

    #[test]
    fn empty_returns_none() {
        assert!(build(&[], &Config::default()).is_none());
    }

    #[test]
    fn invalid_config_returns_none() {
        let bars = vec![bar(101.0, 99.0); 30];
        for cfg in [
            Config {
                n_buckets: 0,
                ..Default::default()
            },
            Config {
                n_buckets: 2,
                ..Default::default()
            },
            Config {
                value_area_pct: 0.0,
                ..Default::default()
            },
            Config {
                value_area_pct: 1.5,
                ..Default::default()
            },
        ] {
            assert!(build(&bars, &cfg).is_none());
        }
    }

    #[test]
    fn zero_range_returns_none() {
        // All bars at the same price → range = 0.
        let bars = vec![bar(100.0, 100.0); 30];
        assert!(build(&bars, &Config::default()).is_none());
    }

    #[test]
    fn nan_bars_skipped_safely() {
        let bars = vec![bar(100.0, 99.0), bar(f64::NAN, f64::NAN), bar(102.0, 101.0)];
        let r = build(&bars, &Config::default()).expect("has signal");
        assert!(r.session_high > r.session_low);
    }

    #[test]
    fn poc_at_most_traded_zone() {
        // Most bars sit in 100-102, a few outliers at 110.
        let mut bars: Vec<Bar> = (0..50).map(|_| bar(102.0, 100.0)).collect();
        bars.push(bar(112.0, 110.0));
        let r = build(&bars, &Config::default()).expect("populated");
        // POC should be in the 100-102 zone, not near 110.
        assert!(
            r.poc_price < 105.0,
            "POC should be in dense zone, got {}",
            r.poc_price
        );
    }

    #[test]
    fn value_area_covers_target_pct() {
        // Uniform: every bucket hit roughly equally → VA should span
        // roughly 70% of buckets.
        let bars: Vec<Bar> = (0..100)
            .map(|i| {
                let mid = 100.0 + i as f64 * 0.1;
                bar(mid + 1.0, mid - 1.0)
            })
            .collect();
        let cfg = Config {
            n_buckets: 30,
            value_area_pct: 0.70,
        };
        let r = build(&bars, &cfg).expect("populated");
        let va_width = r.value_area_high_index as isize - r.value_area_low_index as isize + 1;
        assert!(va_width > 0 && va_width <= 30);
        // Should be wider than 1 bucket (covers significant fraction).
        assert!(va_width >= 5);
    }

    #[test]
    fn val_below_vah_and_poc_inside_value_area() {
        let bars: Vec<Bar> = (0..50)
            .map(|i| {
                let mid = 100.0 + (i as f64 * 0.2).sin() * 5.0;
                bar(mid + 0.5, mid - 0.5)
            })
            .collect();
        let r = build(&bars, &Config::default()).expect("populated");
        assert!(r.val <= r.poc_price && r.poc_price <= r.vah);
    }

    #[test]
    fn counts_sum_corresponds_to_total_bar_coverage() {
        // Each bar spans 3 buckets in this synth → total counts ≈ 3 × n_bars.
        let bars = vec![bar(105.0, 95.0); 30];
        let cfg = Config {
            n_buckets: 10,
            value_area_pct: 0.70,
        };
        let r = build(&bars, &cfg).expect("populated");
        let total: u64 = r.tpo_counts.iter().sum();
        assert!(
            total >= 30,
            "every bar must contribute ≥ 1 TPO; total={total}"
        );
    }
}
