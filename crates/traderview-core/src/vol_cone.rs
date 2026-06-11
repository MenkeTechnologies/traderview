//! Volatility cone — Burghardt & Lane (1990), "How to Tell if Options
//! Are Cheap".
//!
//! For each horizon (e.g. 5/10/21/42/63 trading days), compute the
//! annualized close-to-close realized volatility of EVERY rolling
//! window in the sample, then report the min / p25 / median / p75 /
//! max envelope plus the CURRENT (most recent window) reading. An
//! option's implied vol can then be ranked against where realized vol
//! has historically lived at that horizon — the classic
//! cheap-vs-rich-vol visual.
//!
//! Pure compute over a close series; √252 annualization.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct VolConeRow {
    pub horizon_days: usize,
    pub min_pct: f64,
    pub p25_pct: f64,
    pub median_pct: f64,
    pub p75_pct: f64,
    pub max_pct: f64,
    /// Realized vol of the most recent window at this horizon.
    pub current_pct: f64,
    /// Percentile rank (0..100) of `current` within the distribution.
    pub current_rank_pct: f64,
    pub samples: usize,
}

/// Annualized realized vol (%) of log returns `rets[i..i+window]`.
fn window_vol(rets: &[f64]) -> f64 {
    let n = rets.len();
    if n < 2 {
        return 0.0;
    }
    let mean = rets.iter().sum::<f64>() / n as f64;
    let var = rets.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (n - 1) as f64;
    (var * 252.0).sqrt() * 100.0
}

fn percentile(sorted: &[f64], q: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let pos = q * (sorted.len() - 1) as f64;
    let lo = pos.floor() as usize;
    let hi = pos.ceil() as usize;
    if lo == hi {
        sorted[lo]
    } else {
        sorted[lo] + (sorted[hi] - sorted[lo]) * (pos - lo as f64)
    }
}

pub fn compute(closes: &[f64], horizons: &[usize]) -> Vec<VolConeRow> {
    // Log returns; non-positive or non-finite closes break the chain
    // with a 0 return rather than emitting NaN.
    let rets: Vec<f64> = closes
        .windows(2)
        .map(|w| {
            if w[0] > 0.0 && w[1] > 0.0 && w[0].is_finite() && w[1].is_finite() {
                (w[1] / w[0]).ln()
            } else {
                0.0
            }
        })
        .collect();
    let mut out = Vec::new();
    for &h in horizons {
        if h < 2 || rets.len() < h {
            continue;
        }
        let mut vols: Vec<f64> = rets.windows(h).map(window_vol).collect();
        let current = *vols.last().expect("rets.len() >= h guarantees a window");
        let below = vols.iter().filter(|v| **v <= current).count();
        let rank = (below as f64 - 1.0).max(0.0) / (vols.len().max(2) - 1) as f64 * 100.0;
        vols.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        out.push(VolConeRow {
            horizon_days: h,
            min_pct: vols[0],
            p25_pct: percentile(&vols, 0.25),
            median_pct: percentile(&vols, 0.50),
            p75_pct: percentile(&vols, 0.75),
            max_pct: *vols.last().expect("non-empty"),
            current_pct: current,
            current_rank_pct: rank,
            samples: vols.len(),
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constant_tape_has_zero_vol_everywhere() {
        let closes = vec![100.0; 100];
        let rows = compute(&closes, &[5, 21]);
        assert_eq!(rows.len(), 2);
        for r in rows {
            assert_eq!(r.min_pct, 0.0);
            assert_eq!(r.max_pct, 0.0);
            assert_eq!(r.current_pct, 0.0);
        }
    }

    #[test]
    fn alternating_tape_matches_hand_computed_vol() {
        // ±1% alternating log-ish moves: every 5-day window has the
        // same return set ⇒ min == max == current, rank degenerate.
        let mut closes = vec![100.0];
        for i in 1..50 {
            let prev = closes[i - 1];
            closes.push(if i % 2 == 0 { prev / 1.01 } else { prev * 1.01 });
        }
        let rows = compute(&closes, &[4]);
        assert_eq!(rows.len(), 1);
        let r = &rows[0];
        // Hand value: returns alternate ±ln(1.01); sample sd of
        // [+r,−r,+r,−r] with mean 0 = r·√(4/3); annualized ×√252.
        let lr = 1.01_f64.ln();
        let want = (lr * lr * 4.0 / 3.0).sqrt() * 252.0_f64.sqrt() * 100.0;
        assert!((r.median_pct - want).abs() < 1e-9, "{} vs {want}", r.median_pct);
        assert!((r.max_pct - r.min_pct).abs() < 1e-9);
    }

    #[test]
    fn calm_then_wild_tape_ranks_current_at_top() {
        // 60 flat bars, then 10 alternating ±5% bars: the most recent
        // 5-day window is the wildest in the sample.
        let mut closes = vec![100.0; 60];
        for i in 0..10 {
            let prev = *closes.last().expect("non-empty");
            closes.push(if i % 2 == 0 { prev * 1.05 } else { prev * 0.95 });
        }
        let rows = compute(&closes, &[5]);
        let r = &rows[0];
        assert!(r.current_pct > r.p75_pct);
        assert!((r.current_pct - r.max_pct).abs() < 1e-9);
        assert!(r.current_rank_pct > 90.0, "{}", r.current_rank_pct);
    }

    #[test]
    fn short_series_skips_oversized_horizons() {
        let closes = vec![100.0, 101.0, 102.0, 101.0];
        let rows = compute(&closes, &[2, 63]);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].horizon_days, 2);
        assert!(compute(&[], &[5]).is_empty());
    }
}
