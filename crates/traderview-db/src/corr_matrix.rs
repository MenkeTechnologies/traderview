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
