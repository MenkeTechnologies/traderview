//! Pairwise return-correlation helpers.
//!
//! Confluence autotrade fires per-symbol orders independently. Without
//! a correlation check, a single news cycle can send AAPL+MSFT+GOOG+META
//! all to the top of the ranking — Kelly sizes each as if independent
//! and the user ends up 5× overweight one factor (mega-cap tech, β≈1.1).
//! This module supplies the math: convert closes to daily % returns,
//! compute Pearson r, find the max |r| between a candidate symbol and
//! a set of currently-held symbols.
//!
//! Pure compute only — the autotrade pipeline owns the price-fetch +
//! threshold-check wiring.

use chrono::NaiveDate;

/// Daily % returns from a chronological close series. Returns one fewer
/// row than the input. `(close[t] / close[t-1] - 1) * 100`. Sequences
/// with non-positive closes are filtered out (treated as gaps).
pub fn pct_returns(closes: &[(NaiveDate, f64)]) -> Vec<f64> {
    if closes.len() < 2 {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(closes.len() - 1);
    for w in closes.windows(2) {
        let (_, prev) = w[0];
        let (_, curr) = w[1];
        if prev <= 0.0 || curr <= 0.0 {
            continue;
        }
        out.push((curr / prev - 1.0) * 100.0);
    }
    out
}

/// Pearson correlation coefficient. Returns `None` when:
///   * either input has fewer than 2 samples
///   * lengths don't match
///   * either series has zero variance (all values identical)
pub fn pearson(xs: &[f64], ys: &[f64]) -> Option<f64> {
    let n = xs.len();
    if n < 2 || n != ys.len() {
        return None;
    }
    let mean_x = xs.iter().sum::<f64>() / n as f64;
    let mean_y = ys.iter().sum::<f64>() / n as f64;
    let mut sxx = 0.0;
    let mut syy = 0.0;
    let mut sxy = 0.0;
    for (&x, &y) in xs.iter().zip(ys.iter()) {
        let dx = x - mean_x;
        let dy = y - mean_y;
        sxx += dx * dx;
        syy += dy * dy;
        sxy += dx * dy;
    }
    if sxx <= 0.0 || syy <= 0.0 {
        return None;
    }
    let denom = (sxx * syy).sqrt();
    if denom <= 0.0 {
        return None;
    }
    Some(sxy / denom)
}

/// Take two close series, align by intersecting dates (inner join),
/// convert each aligned series to % returns, then compute Pearson r on
/// those returns. Returns `None` if too few overlapping dates.
pub fn correlation_from_closes(a: &[(NaiveDate, f64)], b: &[(NaiveDate, f64)]) -> Option<f64> {
    use std::collections::HashMap;
    let b_map: HashMap<NaiveDate, f64> = b.iter().copied().collect();
    let mut aligned_a: Vec<(NaiveDate, f64)> = Vec::new();
    let mut aligned_b: Vec<(NaiveDate, f64)> = Vec::new();
    for (date, va) in a {
        if let Some(vb) = b_map.get(date).copied() {
            aligned_a.push((*date, *va));
            aligned_b.push((*date, vb));
        }
    }
    if aligned_a.len() < 3 {
        return None;
    }
    let ra = pct_returns(&aligned_a);
    let rb = pct_returns(&aligned_b);
    pearson(&ra, &rb)
}

/// Max ABSOLUTE pairwise correlation between the candidate's close
/// series and any of the `existing` close series. Returns `(symbol, r)`
/// of the offender, or `None` when no overlap can be computed.
///
/// We use ABSOLUTE correlation (`.abs()`) because a perfectly negative
/// correlation (r = -1) is still a factor exposure — you've just
/// stacked a long and a short of the same factor. The gate should fire
/// in both directions.
pub fn max_pairwise_abs_correlation<'a>(
    candidate: &[(NaiveDate, f64)],
    existing: &'a [(String, Vec<(NaiveDate, f64)>)],
) -> Option<(&'a str, f64)> {
    let mut best: Option<(&'a str, f64)> = None;
    for (sym, closes) in existing.iter() {
        if let Some(r) = correlation_from_closes(candidate, closes) {
            let abs_r = r.abs();
            match best {
                None => best = Some((sym.as_str(), abs_r)),
                Some((_, prev)) if abs_r > prev => best = Some((sym.as_str(), abs_r)),
                _ => {}
            }
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    fn linear(start: NaiveDate, n: usize, daily_pct: f64) -> Vec<(NaiveDate, f64)> {
        let mut p = 100.0_f64;
        (0..n)
            .map(|i| {
                let row = (start + Duration::days(i as i64), p);
                p *= 1.0 + daily_pct / 100.0;
                row
            })
            .collect()
    }

    /// Deterministic noisy series for correlation testing — alternates
    /// the daily move so pct_returns has actual variance.
    fn noisy(start: NaiveDate, n: usize, base_pct: f64, noise_pct: f64) -> Vec<(NaiveDate, f64)> {
        let mut p = 100.0_f64;
        (0..n)
            .map(|i| {
                let row = (start + Duration::days(i as i64), p);
                let sign = if i % 2 == 0 { 1.0 } else { -1.0 };
                p *= 1.0 + (base_pct + sign * noise_pct) / 100.0;
                row
            })
            .collect()
    }

    #[test]
    fn pct_returns_basic() {
        let closes = vec![
            (d(2026, 1, 1), 100.0),
            (d(2026, 1, 2), 110.0), // +10%
            (d(2026, 1, 3), 99.0),  // -10%
        ];
        let r = pct_returns(&closes);
        assert_eq!(r.len(), 2);
        assert!((r[0] - 10.0).abs() < 1e-9);
        assert!((r[1] - (-10.0)).abs() < 1e-9);
    }

    #[test]
    fn pct_returns_skips_zero_or_negative_prev() {
        let closes = vec![
            (d(2026, 1, 1), 100.0),
            (d(2026, 1, 2), 0.0), // gap
            (d(2026, 1, 3), 100.0),
        ];
        let r = pct_returns(&closes);
        assert!(
            r.is_empty(),
            "0-price prev kills both windows it appears in"
        );
    }

    #[test]
    fn pct_returns_empty_when_under_two() {
        assert!(pct_returns(&[]).is_empty());
        assert!(pct_returns(&[(d(2026, 1, 1), 100.0)]).is_empty());
    }

    #[test]
    fn pearson_identical_series_is_one() {
        let xs = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let ys = xs.clone();
        let r = pearson(&xs, &ys).unwrap();
        assert!((r - 1.0).abs() < 1e-9);
    }

    #[test]
    fn pearson_perfectly_negative_is_minus_one() {
        let xs = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let ys: Vec<f64> = xs.iter().map(|v| -v).collect();
        let r = pearson(&xs, &ys).unwrap();
        assert!((r - (-1.0)).abs() < 1e-9);
    }

    #[test]
    fn pearson_returns_none_for_constant_series() {
        let xs = vec![1.0, 2.0, 3.0];
        let ys = vec![5.0, 5.0, 5.0];
        assert!(pearson(&xs, &ys).is_none());
    }

    #[test]
    fn pearson_returns_none_for_length_mismatch() {
        let xs = vec![1.0, 2.0, 3.0];
        let ys = vec![1.0, 2.0];
        assert!(pearson(&xs, &ys).is_none());
    }

    #[test]
    fn pearson_returns_none_for_insufficient_samples() {
        assert!(pearson(&[1.0], &[2.0]).is_none());
        assert!(pearson(&[], &[]).is_none());
    }

    #[test]
    fn correlation_from_closes_aligns_by_date() {
        // Two parallel noisy series → should be ~+1 correlated.
        let a = noisy(d(2026, 1, 1), 30, 0.5, 0.3);
        let b = noisy(d(2026, 1, 1), 30, 0.6, 0.3);
        let r = correlation_from_closes(&a, &b).unwrap();
        assert!(r > 0.9, "parallel noisy series should be ~+1, got {r}");
    }

    #[test]
    fn correlation_from_closes_aligns_when_b_missing_dates() {
        let a = noisy(d(2026, 1, 1), 30, 0.5, 0.3);
        // b only has every other date — alignment should still work.
        let b: Vec<(NaiveDate, f64)> = noisy(d(2026, 1, 1), 30, 0.5, 0.3)
            .into_iter()
            .enumerate()
            .filter(|(i, _)| i % 2 == 0)
            .map(|(_, r)| r)
            .collect();
        let r = correlation_from_closes(&a, &b);
        assert!(r.is_some());
    }

    #[test]
    fn correlation_from_closes_none_when_no_overlap() {
        let a = vec![(d(2026, 1, 1), 100.0), (d(2026, 1, 2), 101.0)];
        let b = vec![(d(2026, 6, 1), 100.0), (d(2026, 6, 2), 101.0)];
        assert!(correlation_from_closes(&a, &b).is_none());
    }

    #[test]
    fn max_pairwise_returns_highest_abs_correlation() {
        let candidate = noisy(d(2026, 1, 1), 30, 0.5, 0.3);
        // STRONG = identical → r ≈ 1. WEAK = different noise pattern.
        // Use opposite-phase noise for WEAK so its correlation is lower.
        let weak: Vec<(NaiveDate, f64)> = {
            let mut p = 100.0;
            (0..30)
                .map(|i| {
                    let row = (d(2026, 1, 1) + Duration::days(i as i64), p);
                    let sign = if i % 3 == 0 { 1.0 } else { -1.0 };
                    p *= 1.0 + (0.05 + sign * 0.3) / 100.0;
                    row
                })
                .collect()
        };
        let existing = vec![("WEAK".into(), weak), ("STRONG".into(), candidate.clone())];
        let (sym, r) = max_pairwise_abs_correlation(&candidate, &existing).unwrap();
        assert_eq!(sym, "STRONG", "identical series should win");
        assert!(r > 0.99);
    }

    #[test]
    fn max_pairwise_returns_none_when_no_existing() {
        let candidate = noisy(d(2026, 1, 1), 30, 0.5, 0.3);
        assert!(max_pairwise_abs_correlation(&candidate, &[]).is_none());
    }

    #[test]
    fn max_pairwise_treats_negative_corr_as_factor_exposure() {
        // Candidate and inverse have perfectly anti-correlated *returns*.
        let candidate = noisy(d(2026, 1, 1), 30, 0.5, 0.3);
        let returns_candidate = pct_returns(&candidate);
        // Build a price series whose returns are exactly -1 × the candidate's returns.
        let mut inv_p = 100.0_f64;
        let mut inverse: Vec<(NaiveDate, f64)> = vec![(d(2026, 1, 1), inv_p)];
        for (i, r) in returns_candidate.iter().enumerate() {
            inv_p *= 1.0 + (-r) / 100.0;
            inverse.push((d(2026, 1, 1) + Duration::days((i + 1) as i64), inv_p));
        }
        let existing = vec![("INVERSE".into(), inverse)];
        let (sym, r) = max_pairwise_abs_correlation(&candidate, &existing).unwrap();
        assert_eq!(sym, "INVERSE");
        assert!(
            r > 0.99,
            "abs() should pick up perfect negative correlation, got {r}"
        );
    }
}
