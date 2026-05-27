//! Market-breadth dashboard — NYSE TICK / TRIN / Advance-Decline / Up-Down
//! Volume / Put-Call ratio. All sourced from public Yahoo index tickers.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;

#[derive(Debug, Clone, Serialize)]
pub struct BreadthSnapshot {
    pub tick: Option<Indicator>, // ^TICK — NYSE TICK
    pub trin: Option<Indicator>, // ^TRIN — NYSE Arms Index
    pub addn: Option<Indicator>, // ^ADD  — NYSE advance/decline issues
    pub vold: Option<Indicator>, // ^VOLD — NYSE up/down volume
    pub pcr: Option<Indicator>,  // CPC   — CBOE put/call ratio (when available)
    pub composite_score: i32,    // -100..+100, positive = bullish breadth
    pub regime: &'static str,    // "bullish" | "bearish" | "neutral"
    pub fetched_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Indicator {
    pub symbol: &'static str,
    pub label: &'static str,
    pub value: f64,
    pub change_pct: f64,
    pub interpretation: String,
}

pub async fn snapshot(pool: &PgPool) -> anyhow::Result<BreadthSnapshot> {
    let tick = fetch(pool, "^TICK", "NYSE TICK", |v| {
        if v >= 800.0 {
            "extreme buying"
        } else if v >= 400.0 {
            "buying"
        } else if v >= -400.0 {
            "balanced"
        } else if v >= -800.0 {
            "selling"
        } else {
            "extreme selling"
        }
        .into()
    })
    .await;
    let trin = fetch(pool, "^TRIN", "NYSE TRIN (Arms)", |v| {
        // <1 = bullish (volume into up issues), >1 = bearish
        if v <= 0.5 {
            "very bullish (extreme buying)"
        } else if v <= 0.9 {
            "bullish"
        } else if v <= 1.1 {
            "neutral"
        } else if v <= 2.0 {
            "bearish"
        } else {
            "very bearish (extreme selling)"
        }
        .into()
    })
    .await;
    let addn = fetch(pool, "^ADD", "NYSE Adv−Dec issues", |v| {
        if v >= 1500.0 {
            "broad rally"
        } else if v >= 500.0 {
            "advances leading"
        } else if v >= -500.0 {
            "mixed"
        } else if v >= -1500.0 {
            "declines leading"
        } else {
            "broad selloff"
        }
        .into()
    })
    .await;
    let vold = fetch(pool, "^VOLD", "NYSE Up−Down vol", |v| {
        if v >= 0.0 {
            "more up-volume"
        } else {
            "more down-volume"
        }
        .into()
    })
    .await;
    // CBOE put-call total: ^CPC (sometimes ^PCC). Try a couple.
    let pcr = fetch(pool, "^CPC", "Put-Call ratio (CBOE)", |v| {
        if v <= 0.6 {
            "extreme greed (low puts → contrarian sell)"
        } else if v <= 0.8 {
            "bullish"
        } else if v <= 1.0 {
            "balanced"
        } else if v <= 1.2 {
            "bearish"
        } else {
            "extreme fear (high puts → contrarian buy)"
        }
        .into()
    })
    .await;

    // Composite score: each indicator contributes ±20.
    let mut score: i32 = 0;
    let push = |s: &mut i32, opt: &Option<Indicator>, fwd: bool, lo: f64, hi: f64| {
        if let Some(ind) = opt {
            let v = ind.value;
            let normalized = ((v - lo) / (hi - lo)).clamp(0.0, 1.0); // 0..1 = lo..hi
            let pts = (normalized * 40.0 - 20.0).round() as i32; // -20..+20
            *s += if fwd { pts } else { -pts };
        }
    };
    push(&mut score, &tick, true, -1000.0, 1000.0);
    push(&mut score, &addn, true, -2000.0, 2000.0);
    push(&mut score, &vold, true, -1.0, 1.0);
    push(&mut score, &trin, false, 0.5, 2.0); // inverted: low TRIN bullish
    push(&mut score, &pcr, false, 0.5, 1.5); // inverted: low P/C bullish

    let regime = if score >= 30 {
        "bullish"
    } else if score <= -30 {
        "bearish"
    } else {
        "neutral"
    };
    Ok(BreadthSnapshot {
        tick,
        trin,
        addn,
        vold,
        pcr,
        composite_score: score.clamp(-100, 100),
        regime,
        fetched_at: Utc::now(),
    })
}

async fn fetch<F>(
    pool: &PgPool,
    sym: &'static str,
    label: &'static str,
    interp: F,
) -> Option<Indicator>
where
    F: FnOnce(f64) -> String,
{
    let q = crate::market_data::quote(pool, sym).await.ok()?;
    Some(Indicator {
        symbol: sym,
        label,
        value: q.price,
        change_pct: q.change_pct.unwrap_or(0.0),
        interpretation: interp(q.price),
    })
}
