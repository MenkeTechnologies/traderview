//! Scanner — runs Warrior/Zendoo preset filters across a universe.

use chrono::{Duration, Utc};
use serde::Serialize;
use sqlx::PgPool;
use std::collections::BTreeSet;
use traderview_core::scan::{matches, preset_label, stats_for, Preset, ScanHit};
use traderview_core::BarInterval;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct ScanRun {
    pub preset: Preset,
    pub label: &'static str,
    pub universe_size: usize,
    pub hits: Vec<ScanHit>,
}

pub async fn run_preset(
    pool: &PgPool,
    user_id: Uuid,
    preset: Preset,
    watchlist_id: Option<Uuid>,
    limit: usize,
) -> anyhow::Result<ScanRun> {
    let symbols = collect_universe(pool, user_id, watchlist_id).await?;
    let universe_size = symbols.len();
    let to = Utc::now();
    let from = to - Duration::days(60);
    let mut hits = Vec::new();
    for sym in &symbols {
        if let Ok(bars) = crate::prices::get_bars(pool, sym, BarInterval::D1, from, to).await {
            if let Some(mut h) = stats_for(sym, &bars) {
                if matches(&h, preset) {
                    h.matched.push(preset_label(preset));
                    hits.push(h);
                }
            }
        }
    }
    hits.sort_by(|a, b| {
        b.change_pct
            .partial_cmp(&a.change_pct)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    hits.truncate(limit);
    Ok(ScanRun {
        preset,
        label: preset_label(preset),
        universe_size,
        hits,
    })
}

async fn collect_universe(
    pool: &PgPool,
    user_id: Uuid,
    watchlist_id: Option<Uuid>,
) -> anyhow::Result<Vec<String>> {
    if let Some(wid) = watchlist_id {
        if !crate::watchlists::ensure_owner(pool, user_id, wid).await? {
            anyhow::bail!("forbidden");
        }
        return crate::watchlists::symbols(pool, wid).await;
    }
    let lists = crate::watchlists::list(pool, user_id).await?;
    let mut all: BTreeSet<String> = BTreeSet::new();
    for w in lists {
        for s in crate::watchlists::symbols(pool, w.id).await? {
            all.insert(s);
        }
    }
    Ok(all.into_iter().collect())
}
