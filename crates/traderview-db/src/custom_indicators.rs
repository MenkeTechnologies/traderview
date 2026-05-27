//! Custom indicator presets — CRUD + evaluator.
//!
//! Each preset binds (kind, params) to a friendly name + color. The evaluator
//! pulls cached bars for the requested (symbol, interval, range) and returns
//! one or more named series ready to overlay on the SVG chart.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use traderview_core::indicators::{bollinger, ema, macd, rsi, sma};
use traderview_core::BarInterval;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct CustomIndicator {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub definition: Value,
    pub color: String,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IndicatorInput {
    pub name: String,
    pub definition: Value,
    #[serde(default = "default_color")] pub color: String,
    #[serde(default)] pub is_default: bool,
}
fn default_color() -> String { "#00e5ff".into() }

#[derive(Debug, Clone, Serialize)]
pub struct EvalSeries {
    pub name: String,
    pub color: String,
    pub values: Vec<Option<f64>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EvalResult {
    pub symbol: String,
    pub interval: String,
    pub times: Vec<DateTime<Utc>>,
    pub closes: Vec<f64>,
    pub series: Vec<EvalSeries>,
}

pub async fn list(pool: &PgPool, user_id: Uuid) -> anyhow::Result<Vec<CustomIndicator>> {
    Ok(sqlx::query_as(
        "SELECT id, user_id, name, definition, color, is_default, created_at, updated_at
           FROM custom_indicators WHERE user_id = $1
          ORDER BY is_default DESC, name",
    ).bind(user_id).fetch_all(pool).await?)
}

pub async fn create(pool: &PgPool, user_id: Uuid, dto: &IndicatorInput)
    -> anyhow::Result<CustomIndicator>
{
    validate(&dto.definition)?;
    Ok(sqlx::query_as(
        "INSERT INTO custom_indicators (user_id, name, definition, color, is_default)
              VALUES ($1, $2, $3, $4, $5)
          ON CONFLICT (user_id, name) DO UPDATE SET
            definition = EXCLUDED.definition,
            color      = EXCLUDED.color,
            is_default = EXCLUDED.is_default,
            updated_at = now()
          RETURNING id, user_id, name, definition, color, is_default, created_at, updated_at",
    )
    .bind(user_id).bind(&dto.name).bind(&dto.definition)
    .bind(&dto.color).bind(dto.is_default)
    .fetch_one(pool).await?)
}

pub async fn delete(pool: &PgPool, user_id: Uuid, id: Uuid) -> anyhow::Result<bool> {
    Ok(sqlx::query("DELETE FROM custom_indicators WHERE id = $1 AND user_id = $2")
        .bind(id).bind(user_id).execute(pool).await?.rows_affected() > 0)
}

/// Evaluate a list of indicator-ids against (symbol, interval, [from, to]).
pub async fn evaluate(
    pool: &PgPool, user_id: Uuid, symbol: &str, interval: BarInterval,
    from: DateTime<Utc>, to: DateTime<Utc>, indicator_ids: &[Uuid],
) -> anyhow::Result<EvalResult> {
    let indicators: Vec<CustomIndicator> = sqlx::query_as(
        "SELECT id, user_id, name, definition, color, is_default, created_at, updated_at
           FROM custom_indicators WHERE user_id = $1 AND id = ANY($2)",
    ).bind(user_id).bind(indicator_ids).fetch_all(pool).await.unwrap_or_default();

    let bars = crate::prices::get_bars(pool, symbol, interval, from, to)
        .await.unwrap_or_default();
    let times: Vec<DateTime<Utc>> = bars.iter().map(|b| b.bar_time).collect();
    let closes: Vec<f64> = bars.iter().map(|b| dec(b.close)).collect();
    let interval_str = match interval {
        BarInterval::M1 => "1m", BarInterval::M5 => "5m",
        BarInterval::M15 => "15m", BarInterval::H1 => "1h",
        BarInterval::D1 => "1d", BarInterval::W1 => "1w",
    };

    let mut series_out: Vec<EvalSeries> = Vec::new();
    for ind in &indicators {
        for s in compute_one(&ind.name, &ind.color, &ind.definition, &closes) {
            series_out.push(s);
        }
    }
    Ok(EvalResult {
        symbol: symbol.to_string(),
        interval: interval_str.into(),
        times, closes, series: series_out,
    })
}

fn compute_one(base_name: &str, color: &str, def: &Value, closes: &[f64]) -> Vec<EvalSeries> {
    let kind = def["kind"].as_str().unwrap_or("");
    let p = &def["params"];
    let p_usize = |k: &str, default: usize| p[k].as_u64().map(|x| x as usize).unwrap_or(default);
    let p_f64   = |k: &str, default: f64|  p[k].as_f64().unwrap_or(default);
    match kind {
        "sma" => vec![EvalSeries {
            name: format!("{} SMA({})", base_name, p_usize("period", 20)),
            color: color.into(),
            values: sma(closes, p_usize("period", 20)),
        }],
        "ema" => vec![EvalSeries {
            name: format!("{} EMA({})", base_name, p_usize("period", 20)),
            color: color.into(),
            values: ema(closes, p_usize("period", 20)),
        }],
        "rsi" => vec![EvalSeries {
            name: format!("{} RSI({})", base_name, p_usize("period", 14)),
            color: color.into(),
            values: rsi(closes, p_usize("period", 14)),
        }],
        "bollinger" => {
            let b = bollinger(closes, p_usize("period", 20), p_f64("k", 2.0));
            vec![
                EvalSeries { name: format!("{} mid",   base_name), color: color.into(), values: b.middle },
                EvalSeries { name: format!("{} upper", base_name), color: color.into(), values: b.upper },
                EvalSeries { name: format!("{} lower", base_name), color: color.into(), values: b.lower },
            ]
        }
        "macd" => {
            let m = macd(closes,
                p_usize("fast", 12), p_usize("slow", 26), p_usize("signal", 9));
            vec![
                EvalSeries { name: format!("{} MACD",   base_name), color: color.into(), values: m.line },
                EvalSeries { name: format!("{} signal", base_name), color: color.into(), values: m.signal },
                EvalSeries { name: format!("{} hist",   base_name), color: color.into(), values: m.histogram },
            ]
        }
        _ => Vec::new(),
    }
}

pub fn validate(def: &Value) -> anyhow::Result<()> {
    let kind = def["kind"].as_str().ok_or_else(|| anyhow::anyhow!("kind required"))?;
    match kind {
        "sma" | "ema" | "rsi" => {
            let p = def["params"]["period"].as_u64().unwrap_or(0);
            if !(2..=400).contains(&(p as i64)) {
                anyhow::bail!("period must be 2..=400");
            }
        }
        "bollinger" => {
            let p = def["params"]["period"].as_u64().unwrap_or(0);
            let k = def["params"]["k"].as_f64().unwrap_or(0.0);
            if !(2..=400).contains(&(p as i64)) { anyhow::bail!("period must be 2..=400"); }
            if !(0.1..=5.0).contains(&k)        { anyhow::bail!("k must be 0.1..=5.0"); }
        }
        "macd" => {
            for k in ["fast", "slow", "signal"] {
                let v = def["params"][k].as_u64().unwrap_or(0);
                if !(2..=200).contains(&(v as i64)) {
                    anyhow::bail!("{k} must be 2..=200");
                }
            }
        }
        other => anyhow::bail!("unknown kind: {other}"),
    }
    Ok(())
}

fn dec(d: Decimal) -> f64 { d.to_string().parse().unwrap_or(0.0) }
