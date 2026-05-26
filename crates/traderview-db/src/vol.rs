//! VIX term structure + Treasury yield curve + DXY.
//!
//! All sourced from Yahoo Finance index tickers (no auth):
//!   * ^VIX9D / ^VIX / ^VIX3M / ^VIX6M / ^VVIX
//!   * ^IRX (13w) / ^FVX (5y) / ^TNX (10y) / ^TYX (30y)
//!   * DX-Y.NYB (DXY)

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;

#[derive(Debug, Clone, Serialize)]
pub struct VolPoint {
    pub label: &'static str,
    pub symbol: &'static str,
    pub tenor_days: u32,
    pub value: f64,
    pub change_pct: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct VixTermStructure {
    pub points: Vec<VolPoint>,
    pub spot: Option<f64>,
    pub three_month: Option<f64>,
    /// (^VIX − ^VIX3M) / ^VIX3M. Positive → backwardation (fear).
    pub contango_pct: Option<f64>,
    pub regime: &'static str, // "contango" | "backwardation" | "flat"
    pub vvix: Option<f64>,
    pub fetched_at: DateTime<Utc>,
}

const VIX_POINTS: &[(&str, &str, u32)] = &[
    ("VIX9D",   "^VIX9D", 9),
    ("VIX",     "^VIX",   30),
    ("VIX3M",   "^VIX3M", 90),
    ("VIX6M",   "^VIX6M", 180),
];

pub async fn vix_term_structure(pool: &PgPool) -> anyhow::Result<VixTermStructure> {
    let mut points = Vec::with_capacity(VIX_POINTS.len());
    for &(label, sym, days) in VIX_POINTS {
        if let Ok(q) = crate::market_data::quote(pool, sym).await {
            points.push(VolPoint {
                label, symbol: sym, tenor_days: days,
                value: q.price, change_pct: q.change_pct.unwrap_or(0.0),
            });
        }
    }
    let spot   = points.iter().find(|p| p.symbol == "^VIX").map(|p| p.value);
    let three  = points.iter().find(|p| p.symbol == "^VIX3M").map(|p| p.value);
    let contango = match (spot, three) {
        (Some(s), Some(t)) if t > 0.0 => Some((s - t) / t * 100.0),
        _ => None,
    };
    let regime = match contango {
        Some(c) if c >  1.0 => "backwardation",
        Some(c) if c < -1.0 => "contango",
        Some(_)             => "flat",
        None                => "unknown",
    };
    let vvix = crate::market_data::quote(pool, "^VVIX").await.ok().map(|q| q.price);
    Ok(VixTermStructure { points, spot, three_month: three, contango_pct: contango, regime, vvix, fetched_at: Utc::now() })
}

// ===========================================================================
// Treasury yield curve
// ===========================================================================

#[derive(Debug, Clone, Serialize)]
pub struct YieldPoint {
    pub label: &'static str,
    pub symbol: &'static str,
    pub tenor_years: f64,
    pub yield_pct: f64,
    pub change_bp: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct YieldCurve {
    pub points: Vec<YieldPoint>,
    pub spread_10y_2y: Option<f64>,
    pub spread_10y_3m: Option<f64>,
    pub inverted: bool,
    pub fetched_at: DateTime<Utc>,
}

const YIELD_POINTS: &[(&str, &str, f64)] = &[
    ("3M",  "^IRX", 0.25),
    ("5Y",  "^FVX", 5.0),
    ("10Y", "^TNX", 10.0),
    ("30Y", "^TYX", 30.0),
];

pub async fn yield_curve(pool: &PgPool) -> anyhow::Result<YieldCurve> {
    let mut points = Vec::with_capacity(YIELD_POINTS.len());
    for &(label, sym, tenor) in YIELD_POINTS {
        if let Ok(q) = crate::market_data::quote(pool, sym).await {
            points.push(YieldPoint {
                label, symbol: sym, tenor_years: tenor,
                yield_pct: q.price,
                change_bp: q.change_pct.unwrap_or(0.0) * 100.0,
            });
        }
    }
    let y10 = points.iter().find(|p| p.symbol == "^TNX").map(|p| p.yield_pct);
    let y5  = points.iter().find(|p| p.symbol == "^FVX").map(|p| p.yield_pct);
    let y3m = points.iter().find(|p| p.symbol == "^IRX").map(|p| p.yield_pct);
    let sp_10_2 = match (y10, y5) { (Some(a), Some(b)) => Some(a - b), _ => None };
    let sp_10_3m = match (y10, y3m) { (Some(a), Some(b)) => Some(a - b), _ => None };
    let inverted = sp_10_3m.map(|s| s < 0.0).unwrap_or(false);
    Ok(YieldCurve {
        points, spread_10y_2y: sp_10_2, spread_10y_3m: sp_10_3m, inverted,
        fetched_at: Utc::now(),
    })
}

#[derive(Debug, Clone, Serialize)]
pub struct DollarSnapshot {
    pub dxy: Option<f64>,
    pub dxy_change_pct: Option<f64>,
    pub eur_usd: Option<f64>,
    pub usd_jpy: Option<f64>,
    pub gbp_usd: Option<f64>,
    pub fetched_at: DateTime<Utc>,
}

pub async fn dollar_snapshot(pool: &PgPool) -> anyhow::Result<DollarSnapshot> {
    let dxy = crate::market_data::quote(pool, "DX-Y.NYB").await.ok();
    let eu  = crate::market_data::quote(pool, "EURUSD=X").await.ok();
    let uj  = crate::market_data::quote(pool, "USDJPY=X").await.ok();
    let gu  = crate::market_data::quote(pool, "GBPUSD=X").await.ok();
    Ok(DollarSnapshot {
        dxy: dxy.as_ref().map(|x| x.price),
        dxy_change_pct: dxy.as_ref().and_then(|x| x.change_pct),
        eur_usd: eu.map(|x| x.price),
        usd_jpy: uj.map(|x| x.price),
        gbp_usd: gu.map(|x| x.price),
        fetched_at: Utc::now(),
    })
}
