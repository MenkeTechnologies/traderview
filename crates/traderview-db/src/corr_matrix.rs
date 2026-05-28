//! Pairwise correlation matrix across a symbol set.
//!
//! For N symbols we compute the upper triangle of N×N Pearson correlations
//! on log-returns from cached daily bars. Symbols missing bars or with <30
//! aligned observations are reported with `samples` = 0 and `value` = None.
//!
//! Common alignment: we intersect each pair's bar timestamps so a symbol
//! with sparse history doesn't drag others to a too-short window.

use chrono::{Duration, Utc};
use serde::Serialize;
use sqlx::PgPool;
use std::collections::BTreeMap;
use traderview_core::correlation::{log_returns, pearson};
use traderview_core::BarInterval;
use uuid::Uuid;

const MIN_SAMPLES: usize = 30;

#[derive(Debug, Clone, Serialize)]
pub struct CorrCell {
    pub a: String,
    pub b: String,
    pub value: Option<f64>, // None = insufficient data
    pub samples: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct CorrMatrix {
    pub symbols: Vec<String>,
    pub days: i64,
    /// Full N×N row-major; diagonal is 1.0.
    pub values: Vec<Vec<Option<f64>>>,
    /// Sortable pair list (upper triangle only) for leaderboards.
    pub pairs: Vec<CorrCell>,
    pub top_correlated: Vec<CorrCell>,   // strongest +
    pub top_diversifying: Vec<CorrCell>, // strongest - (or near-zero if no negatives)
    pub computed_at: chrono::DateTime<Utc>,
}

pub async fn for_watchlist(
    pool: &PgPool,
    user_id: Uuid,
    watchlist_id: Uuid,
    days: i64,
) -> anyhow::Result<CorrMatrix> {
    let symbols: Vec<String> = sqlx::query_scalar(
        "SELECT s.symbol FROM watchlist_symbols s
           JOIN watchlists w ON w.id = s.watchlist_id
          WHERE w.id = $1 AND w.user_id = $2
          ORDER BY s.symbol",
    )
    .bind(watchlist_id)
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    matrix_from_symbols(pool, &symbols, days).await
}

pub async fn for_symbols(
    pool: &PgPool,
    symbols: &[String],
    days: i64,
) -> anyhow::Result<CorrMatrix> {
    matrix_from_symbols(pool, symbols, days).await
}

async fn matrix_from_symbols(
    pool: &PgPool,
    symbols: &[String],
    days: i64,
) -> anyhow::Result<CorrMatrix> {
    let to = Utc::now();
    let from = to - Duration::days(days.max(30));

    // Pull bars once per symbol, store closes keyed by date.
    let mut series: BTreeMap<String, BTreeMap<chrono::NaiveDate, f64>> = BTreeMap::new();
    for s in symbols {
        let bars = crate::prices::get_bars(pool, s, BarInterval::D1, from, to)
            .await
            .unwrap_or_default();
        let mut by_day: BTreeMap<chrono::NaiveDate, f64> = BTreeMap::new();
        for b in bars {
            let d = b.bar_time.date_naive();
            by_day.insert(d, dec(b.close));
        }
        series.insert(s.clone(), by_day);
    }

    let n = symbols.len();
    let mut values: Vec<Vec<Option<f64>>> = vec![vec![None; n]; n];
    let mut pairs: Vec<CorrCell> = Vec::new();

    for i in 0..n {
        values[i][i] = Some(1.0);
        for j in (i + 1)..n {
            let a = &symbols[i];
            let b = &symbols[j];
            let (va, vb, samples) = align_series(&series[a], &series[b]);
            let v = if samples >= MIN_SAMPLES {
                let ra = log_returns(&va);
                let rb = log_returns(&vb);
                pearson(&ra, &rb)
            } else {
                None
            };
            values[i][j] = v;
            values[j][i] = v;
            pairs.push(CorrCell {
                a: a.clone(),
                b: b.clone(),
                value: v,
                samples,
            });
        }
    }

    // Leaderboards: top correlated (highest +), top diversifying
    // (most negative, falling back to lowest absolute if no negatives).
    let mut by_pos: Vec<&CorrCell> = pairs.iter().filter(|c| c.value.is_some()).collect();
    by_pos.sort_by(|x, y| y.value.partial_cmp(&x.value).unwrap());
    let top_correlated: Vec<CorrCell> = by_pos.iter().take(10).map(|c| (*c).clone()).collect();

    let mut by_neg: Vec<&CorrCell> = pairs.iter().filter(|c| c.value.is_some()).collect();
    by_neg.sort_by(|x, y| x.value.partial_cmp(&y.value).unwrap());
    let any_neg = by_neg.first().and_then(|c| c.value).unwrap_or(0.0) < 0.0;
    let top_diversifying: Vec<CorrCell> = if any_neg {
        by_neg.iter().take(10).map(|c| (*c).clone()).collect()
    } else {
        // No negatives — surface lowest-absolute correlations (closest to 0).
        let mut by_abs: Vec<&CorrCell> = pairs.iter().filter(|c| c.value.is_some()).collect();
        by_abs.sort_by(|x, y| {
            x.value
                .unwrap_or(0.0)
                .abs()
                .partial_cmp(&y.value.unwrap_or(0.0).abs())
                .unwrap()
        });
        by_abs.iter().take(10).map(|c| (*c).clone()).collect()
    };

    Ok(CorrMatrix {
        symbols: symbols.to_vec(),
        days,
        values,
        pairs,
        top_correlated,
        top_diversifying,
        computed_at: Utc::now(),
    })
}

fn align_series(
    a: &BTreeMap<chrono::NaiveDate, f64>,
    b: &BTreeMap<chrono::NaiveDate, f64>,
) -> (Vec<f64>, Vec<f64>, usize) {
    let mut va = Vec::new();
    let mut vb = Vec::new();
    for (d, pa) in a {
        if let Some(pb) = b.get(d) {
            va.push(*pa);
            vb.push(*pb);
        }
    }
    let n = va.len();
    (va, vb, n)
}

fn dec(d: rust_decimal::Decimal) -> f64 {
    d.to_string().parse().unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    // ===========================================================================
    // align_series — intersects keys, preserves order
    // ===========================================================================

    #[test]
    fn align_series_empty_inputs_yield_empty_output() {
        let a = BTreeMap::new();
        let b = BTreeMap::new();
        let (va, vb, n) = align_series(&a, &b);
        assert!(va.is_empty());
        assert!(vb.is_empty());
        assert_eq!(n, 0);
    }

    #[test]
    fn align_series_only_a_populated_yields_empty() {
        let mut a = BTreeMap::new();
        a.insert(d(2026, 1, 1), 100.0);
        a.insert(d(2026, 1, 2), 101.0);
        let b = BTreeMap::new();
        let (va, vb, n) = align_series(&a, &b);
        assert!(va.is_empty());
        assert!(vb.is_empty());
        assert_eq!(n, 0);
    }

    #[test]
    fn align_series_fully_overlapping_keys_preserved_in_date_order() {
        // BTreeMap iterates in key order — verifies output is also date-sorted.
        let mut a = BTreeMap::new();
        a.insert(d(2026, 1, 3), 30.0);
        a.insert(d(2026, 1, 1), 10.0);
        a.insert(d(2026, 1, 2), 20.0);
        let mut b = BTreeMap::new();
        b.insert(d(2026, 1, 2), 200.0);
        b.insert(d(2026, 1, 1), 100.0);
        b.insert(d(2026, 1, 3), 300.0);
        let (va, vb, n) = align_series(&a, &b);
        assert_eq!(n, 3);
        assert_eq!(va, vec![10.0, 20.0, 30.0]);
        assert_eq!(vb, vec![100.0, 200.0, 300.0]);
    }

    #[test]
    fn align_series_drops_dates_missing_from_b() {
        // a has 4 days; b has 2. Output keeps only the intersecting 2.
        let mut a = BTreeMap::new();
        a.insert(d(2026, 1, 1), 1.0);
        a.insert(d(2026, 1, 2), 2.0);
        a.insert(d(2026, 1, 3), 3.0);
        a.insert(d(2026, 1, 4), 4.0);
        let mut b = BTreeMap::new();
        b.insert(d(2026, 1, 2), 20.0);
        b.insert(d(2026, 1, 4), 40.0);
        let (va, vb, n) = align_series(&a, &b);
        assert_eq!(n, 2);
        assert_eq!(va, vec![2.0, 4.0]);
        assert_eq!(vb, vec![20.0, 40.0]);
    }

    #[test]
    fn align_series_disjoint_keys_yield_empty() {
        let mut a = BTreeMap::new();
        a.insert(d(2026, 1, 1), 1.0);
        a.insert(d(2026, 1, 2), 2.0);
        let mut b = BTreeMap::new();
        b.insert(d(2026, 2, 1), 10.0);
        b.insert(d(2026, 2, 2), 20.0);
        let (va, vb, n) = align_series(&a, &b);
        assert_eq!(n, 0);
        assert!(va.is_empty());
        assert!(vb.is_empty());
    }

    #[test]
    fn align_series_pairs_remain_index_aligned() {
        // Crucial invariant for correlation math: va[i] and vb[i] must be the
        // pair for the same date.
        let mut a = BTreeMap::new();
        let mut b = BTreeMap::new();
        for i in 0..5 {
            a.insert(d(2026, 1, 1 + i as u32), i as f64);
            b.insert(d(2026, 1, 1 + i as u32), (i * 10) as f64);
        }
        let (va, vb, n) = align_series(&a, &b);
        assert_eq!(n, 5);
        for i in 0..5 {
            assert_eq!(vb[i], va[i] * 10.0);
        }
    }

    #[test]
    fn align_series_length_equals_intersection_size() {
        // Property check: |output| == |a ∩ b|.
        let mut a = BTreeMap::new();
        for i in 1..=10 {
            a.insert(d(2026, 1, i), i as f64);
        }
        let mut b = BTreeMap::new();
        for i in 5..=15 {
            b.insert(d(2026, 1, i.min(31)), (i * 2) as f64);
        }
        let (va, vb, n) = align_series(&a, &b);
        // Intersection on jan: days 5..=10 = 6 days.
        assert_eq!(n, 6);
        assert_eq!(va.len(), n);
        assert_eq!(vb.len(), n);
    }

    // ===========================================================================
    // dec
    // ===========================================================================

    #[test]
    fn dec_round_trip_for_typical_price_decimals() {
        use rust_decimal::Decimal;
        assert_eq!(dec(Decimal::ZERO), 0.0);
        assert!((dec(Decimal::new(123456, 2)) - 1234.56).abs() < 1e-9);
        assert_eq!(dec(Decimal::from(-50)), -50.0);
    }
}
