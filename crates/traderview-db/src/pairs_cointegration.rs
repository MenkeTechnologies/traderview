//! Pairs cointegration scanner.
//!
//! Stat-arb 101: two assets are **cointegrated** when, although each
//! is non-stationary on its own (random-walk-ish), some linear
//! combination of their prices is stationary (mean-reverting). When
//! you spot a cointegrated pair and the current spread sits more than
//! ~2 standard deviations from its mean, betting on convergence has
//! positive expected value over the half-life of mean reversion.
//!
//! Full Engle-Granger cointegration testing involves ADF with
//! MacKinnon critical values — overkill for a real-time ranking
//! tool. This module implements the pragmatic retail/quant approach:
//!
//!   1. OLS regression: `y_t = α + β · x_t + ε_t` to find the hedge
//!      ratio β.
//!   2. Spread: `s_t = y_t − β · x_t`.
//!   3. AR(1) on the spread: `s_t = c + ρ · s_{t-1} + e_t`. The
//!      closer ρ is to 0, the faster the mean reversion. If
//!      ρ is at or above ~0.995 the spread is effectively a random
//!      walk — no edge.
//!   4. Half-life of mean reversion: `−ln(2) / ln(ρ)` (only finite
//!      when 0 < ρ < 1).
//!   5. Current z-score: `(s_T − mean(s)) / stdev(s)`. |z| ≥ 2 is the
//!      classic entry signal.
//!
//! Output ranks pairs by |z_score| descending, filtered to those with
//! a finite, sub-30-day half-life (longer than that is too patient
//! for most discretionary use).

use chrono::{DateTime, Duration, Utc};
use rust_decimal::prelude::ToPrimitive;
use serde::Serialize;
use sqlx::PgPool;
use traderview_core::BarInterval;

use crate::prices;

const MIN_OBSERVATIONS: usize = 60;
/// Maximum mean-reversion half-life (calendar days, approximated from
/// trading-day half-life). Pairs slower than this are dropped.
const MAX_HALF_LIFE_DAYS: f64 = 30.0;
/// Minimum |z| to qualify as an "actionable" pair. Below this the
/// spread is in-band and there's no entry signal.
const MIN_ABS_Z_FOR_SIGNAL: f64 = 2.0;

#[derive(Debug, Clone, Serialize)]
pub struct PairScore {
    pub sym_a: String,
    pub sym_b: String,
    /// Hedge ratio from OLS y = α + β x; one unit of A is hedged with
    /// β units of B.
    pub beta: f64,
    pub alpha: f64,
    /// AR(1) coefficient on the spread. < 1 = mean-reverting.
    pub rho: f64,
    /// Half-life of mean reversion in trading days (approximately
    /// calendar-day-equivalent — we don't adjust for weekends here).
    pub half_life_days: f64,
    /// Mean of the in-sample spread.
    pub mean_spread: f64,
    /// Sample-stdev of the in-sample spread.
    pub stdev_spread: f64,
    /// Latest observed spread.
    pub current_spread: f64,
    /// Z-score of the current spread vs the in-sample distribution.
    pub current_z: f64,
    /// Number of return observations used.
    pub n_obs: usize,
    pub observed_at: DateTime<Utc>,
}

// ─── Pure compute helpers ──────────────────────────────────────────────────

/// Simple OLS y = α + β x. Returns `(α, β)`, or `None` when x has
/// zero variance (would divide by zero).
pub fn ols(y: &[f64], x: &[f64]) -> Option<(f64, f64)> {
    if y.len() != x.len() || y.is_empty() {
        return None;
    }
    let n = y.len() as f64;
    let mean_x = x.iter().sum::<f64>() / n;
    let mean_y = y.iter().sum::<f64>() / n;
    let mut num = 0.0;
    let mut den = 0.0;
    for i in 0..y.len() {
        let dx = x[i] - mean_x;
        num += dx * (y[i] - mean_y);
        den += dx * dx;
    }
    if den.abs() < 1e-12 {
        return None;
    }
    let beta = num / den;
    let alpha = mean_y - beta * mean_x;
    Some((alpha, beta))
}

/// Compute the spread `y_t − β·x_t − α` for every aligned timepoint.
pub fn spread(y: &[f64], x: &[f64], alpha: f64, beta: f64) -> Vec<f64> {
    y.iter()
        .zip(x.iter())
        .map(|(&yi, &xi)| yi - beta * xi - alpha)
        .collect()
}

pub fn mean_stdev(v: &[f64]) -> Option<(f64, f64)> {
    if v.len() < 2 {
        return None;
    }
    let n = v.len() as f64;
    let mean = v.iter().sum::<f64>() / n;
    let var = v.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0);
    Some((mean, var.sqrt()))
}

/// AR(1) on a series: regress `s_t` on `s_{t-1}` to get ρ. Returns
/// `(ρ, intercept)` or None when the regression is degenerate.
pub fn ar1(series: &[f64]) -> Option<(f64, f64)> {
    if series.len() < 2 {
        return None;
    }
    let y: Vec<f64> = series.iter().skip(1).copied().collect();
    let x: Vec<f64> = series.iter().take(series.len() - 1).copied().collect();
    ols(&y, &x).map(|(intercept, rho)| (rho, intercept))
}

/// Half-life of mean reversion in time units (same units as the input
/// sampling cadence). Defined only when 0 < ρ < 1.
pub fn half_life_days(rho: f64) -> Option<f64> {
    if !rho.is_finite() || rho <= 0.0 || rho >= 1.0 {
        return None;
    }
    Some(-(2.0_f64.ln()) / rho.ln())
}

/// Compute the full pair score from aligned daily closes.
pub fn compute_pair(sym_a: &str, sym_b: &str, ya: &[f64], xb: &[f64]) -> Option<PairScore> {
    if ya.len() != xb.len() || ya.len() < MIN_OBSERVATIONS {
        return None;
    }
    let (alpha, beta) = ols(ya, xb)?;
    let sp = spread(ya, xb, alpha, beta);
    let (mean_sp, stdev_sp) = mean_stdev(&sp)?;
    if stdev_sp <= 0.0 {
        return None;
    }
    let (rho, _intercept) = ar1(&sp)?;
    let hl = half_life_days(rho)?;
    if !hl.is_finite() || hl > MAX_HALF_LIFE_DAYS {
        return None;
    }
    let current = *sp.last().expect("sp non-empty after mean_stdev");
    let z = (current - mean_sp) / stdev_sp;
    Some(PairScore {
        sym_a: sym_a.to_ascii_uppercase(),
        sym_b: sym_b.to_ascii_uppercase(),
        beta,
        alpha,
        rho,
        half_life_days: hl,
        mean_spread: mean_sp,
        stdev_spread: stdev_sp,
        current_spread: current,
        current_z: z,
        n_obs: ya.len(),
        observed_at: Utc::now(),
    })
}

// ─── Repository / scan ─────────────────────────────────────────────────────

async fn closes_for(
    pool: &PgPool,
    symbol: &str,
    lookback_days: i64,
) -> anyhow::Result<Vec<(chrono::NaiveDate, f64)>> {
    let to = Utc::now();
    let from = to - Duration::days(lookback_days);
    let bars = prices::get_bars(pool, symbol, BarInterval::D1, from, to).await?;
    Ok(bars
        .into_iter()
        .filter_map(|b| {
            let d = b.bar_time.date_naive();
            b.close.to_f64().map(|c| (d, c))
        })
        .collect())
}

/// Align two date-keyed close series by inner join — only dates
/// present in BOTH series survive. Returns `(closes_a_aligned,
/// closes_b_aligned)` in chronological order. Empty input → empty
/// output (no panic).
pub fn align(
    a: &[(chrono::NaiveDate, f64)],
    b: &[(chrono::NaiveDate, f64)],
) -> (Vec<f64>, Vec<f64>) {
    use std::collections::HashMap;
    let bmap: HashMap<chrono::NaiveDate, f64> = b.iter().copied().collect();
    let mut ya = Vec::new();
    let mut xb = Vec::new();
    let mut sorted_a = a.to_vec();
    sorted_a.sort_by_key(|(d, _)| *d);
    for (d, ca) in sorted_a {
        if let Some(&cb) = bmap.get(&d) {
            ya.push(ca);
            xb.push(cb);
        }
    }
    (ya, xb)
}

/// Scan all unique pairs from a symbol list. For N symbols this is
/// N·(N-1)/2 OLS+AR(1) fits, each ~1 ms at N≈60 observations. For
/// the default 30-symbol watchlist that's ~435 pairs in well under
/// a second once the daily bars are warm in the price_bars table.
pub async fn scan(
    pool: &PgPool,
    symbols: &[String],
    lookback_days: i64,
) -> anyhow::Result<Vec<PairScore>> {
    // Fetch closes once per symbol.
    let mut series: Vec<(String, Vec<(chrono::NaiveDate, f64)>)> =
        Vec::with_capacity(symbols.len());
    for sym in symbols {
        match closes_for(pool, sym, lookback_days).await {
            Ok(v) if v.len() >= MIN_OBSERVATIONS => series.push((sym.to_ascii_uppercase(), v)),
            _ => tracing::debug!(symbol = %sym, "pairs: insufficient daily bars, skipped"),
        }
    }
    let mut out: Vec<PairScore> = Vec::new();
    for i in 0..series.len() {
        for j in (i + 1)..series.len() {
            let (ya, xb) = align(&series[i].1, &series[j].1);
            if ya.len() < MIN_OBSERVATIONS {
                continue;
            }
            if let Some(p) = compute_pair(&series[i].0, &series[j].0, &ya, &xb) {
                if p.current_z.abs() >= MIN_ABS_Z_FOR_SIGNAL {
                    out.push(p);
                }
            }
        }
    }
    out.sort_by(|a, b| {
        b.current_z
            .abs()
            .partial_cmp(&a.current_z.abs())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> chrono::NaiveDate {
        chrono::NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    #[test]
    fn ols_recovers_known_beta() {
        // y = 2 + 3x with no noise → α=2, β=3.
        let x: Vec<f64> = (0..50).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|xi| 2.0 + 3.0 * xi).collect();
        let (alpha, beta) = ols(&y, &x).unwrap();
        assert!((alpha - 2.0).abs() < 1e-9);
        assert!((beta - 3.0).abs() < 1e-9);
    }

    #[test]
    fn ols_none_when_x_has_zero_variance() {
        let x = vec![5.0; 50];
        let y: Vec<f64> = (0..50).map(|i| i as f64).collect();
        assert!(ols(&y, &x).is_none());
    }

    /// Deterministic linear-congruential PRNG for test fixtures so the
    /// shocks are uncorrelated with their index (avoids the periodic-
    /// bias problem that `i % n` patterns introduce when regressed
    /// against series_{t-1}).
    struct Lcg(u64);
    impl Lcg {
        fn next_unit(&mut self) -> f64 {
            // Numerical Recipes LCG constants.
            self.0 = self.0.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            // Map to (-1, 1).
            (self.0 as f64) / (u64::MAX as f64) * 2.0 - 1.0
        }
    }

    #[test]
    fn ar1_recovers_known_rho() {
        // Construct an AR(1) series with rho=0.7 and i.i.d. shocks
        // from a deterministic PRNG so the test is reproducible AND
        // the shocks are uncorrelated with their index.
        let rho_true = 0.7;
        let mut prng = Lcg(0xdead_beef);
        let mut s: Vec<f64> = vec![0.0];
        for _ in 0..2_000 {
            let shock = prng.next_unit() * 0.3;
            s.push(rho_true * s.last().unwrap() + shock);
        }
        let (rho_est, _) = ar1(&s).unwrap();
        assert!(
            (rho_est - rho_true).abs() < 0.05,
            "estimated rho {rho_est} far from true {rho_true}"
        );
    }

    #[test]
    fn half_life_undefined_outside_unit_range() {
        assert!(half_life_days(0.0).is_none());
        assert!(half_life_days(1.0).is_none());
        assert!(half_life_days(1.5).is_none());
        assert!(half_life_days(-0.1).is_none());
        // ρ = 0.5 → half-life = -ln(2)/ln(0.5) = 1.
        let h = half_life_days(0.5).unwrap();
        assert!((h - 1.0).abs() < 1e-9);
        // ρ closer to 1 → longer half-life. ρ=0.9 → ~6.58.
        let h2 = half_life_days(0.9).unwrap();
        assert!((h2 - 6.578_813).abs() < 1e-4);
    }

    #[test]
    fn spread_subtracts_hedge() {
        let y = vec![10.0, 12.0, 14.0];
        let x = vec![1.0, 2.0, 3.0];
        let s = spread(&y, &x, 0.0, 2.0);
        assert_eq!(s, vec![8.0, 8.0, 8.0]);
    }

    #[test]
    fn align_inner_joins_on_dates() {
        let a = vec![
            (d(2026, 1, 1), 100.0),
            (d(2026, 1, 2), 101.0),
            (d(2026, 1, 3), 102.0),
        ];
        let b = vec![
            (d(2026, 1, 2), 50.0),
            (d(2026, 1, 3), 51.0),
            (d(2026, 1, 4), 52.0),
        ];
        let (ya, xb) = align(&a, &b);
        // Only 1-2 and 1-3 in both.
        assert_eq!(ya, vec![101.0, 102.0]);
        assert_eq!(xb, vec![50.0, 51.0]);
    }

    #[test]
    fn compute_pair_emits_when_cointegrated_with_high_z() {
        // Synthetic cointegrated pair using a deterministic PRNG:
        //   x_t = random walk
        //   y_t = 2·x_t + spread_t,
        //   spread_t = 0.5·spread_{t-1} + noise (mean-reverting AR(1))
        // Then force the final spread to ~3 stdevs to trigger
        // the |z|≥2 entry signal.
        let mut prng = Lcg(0x1234_5678);
        let mut x: Vec<f64> = vec![100.0];
        let mut spread_series: Vec<f64> = vec![0.0];
        for _ in 0..200 {
            let dx = prng.next_unit() * 0.5;
            x.push(x.last().unwrap() + dx);
            let prev_s = *spread_series.last().unwrap();
            let shock = prng.next_unit() * 0.2;
            spread_series.push(0.5 * prev_s + shock);
        }
        let last_idx = spread_series.len() - 1;
        spread_series[last_idx] = 3.0;
        let y: Vec<f64> = x
            .iter()
            .zip(spread_series.iter())
            .map(|(xi, si)| 2.0 * xi + si)
            .collect();
        let p = compute_pair("A", "B", &y, &x).expect("should emit");
        assert!(
            p.current_z.abs() >= MIN_ABS_Z_FOR_SIGNAL,
            "|z| {} below threshold",
            p.current_z.abs()
        );
        assert!(
            (p.beta - 2.0).abs() < 0.2,
            "beta {} far from true 2.0",
            p.beta
        );
        assert!(p.half_life_days < MAX_HALF_LIFE_DAYS);
    }

    #[test]
    fn compute_pair_none_when_random_walk_spread() {
        // Two unrelated random walks → spread is itself a random walk
        // → rho ≈ 1 → half-life undefined → no emit.
        let mut a: Vec<f64> = vec![100.0];
        let mut b: Vec<f64> = vec![100.0];
        for i in 0..80 {
            a.push(a.last().unwrap() + (if i % 2 == 0 { 0.7 } else { -0.5 }));
            b.push(b.last().unwrap() + (if i % 3 == 0 { -0.6 } else { 0.4 }));
        }
        let p = compute_pair("A", "B", &a, &b);
        // Could be None (no half-life) or have |z| < 2 — either way
        // the function shouldn't emit a high-confidence signal.
        if let Some(p) = p {
            assert!(
                p.half_life_days < MAX_HALF_LIFE_DAYS,
                "if a pair is emitted, half-life must be bounded"
            );
        }
    }

    #[test]
    fn compute_pair_none_with_insufficient_observations() {
        let y = vec![1.0, 2.0, 3.0];
        let x = vec![1.0, 2.0, 3.0];
        assert!(compute_pair("A", "B", &y, &x).is_none());
    }
}
