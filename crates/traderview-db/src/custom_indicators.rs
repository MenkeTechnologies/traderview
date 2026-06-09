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
use traderview_core::indicators::{adx, atr, bollinger, ema, macd, rsi, sma, stochastic};
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
    #[serde(default = "default_color")]
    pub color: String,
    #[serde(default)]
    pub is_default: bool,
}
fn default_color() -> String {
    "#00e5ff".into()
}

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
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?)
}

pub async fn create(
    pool: &PgPool,
    user_id: Uuid,
    dto: &IndicatorInput,
) -> anyhow::Result<CustomIndicator> {
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
    .bind(user_id)
    .bind(&dto.name)
    .bind(&dto.definition)
    .bind(&dto.color)
    .bind(dto.is_default)
    .fetch_one(pool)
    .await?)
}

pub async fn delete(pool: &PgPool, user_id: Uuid, id: Uuid) -> anyhow::Result<bool> {
    Ok(
        sqlx::query("DELETE FROM custom_indicators WHERE id = $1 AND user_id = $2")
            .bind(id)
            .bind(user_id)
            .execute(pool)
            .await?
            .rows_affected()
            > 0,
    )
}

/// Evaluate a list of indicator-ids against (symbol, interval, [from, to]).
pub async fn evaluate(
    pool: &PgPool,
    user_id: Uuid,
    symbol: &str,
    interval: BarInterval,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
    indicator_ids: &[Uuid],
) -> anyhow::Result<EvalResult> {
    let indicators: Vec<CustomIndicator> = sqlx::query_as(
        "SELECT id, user_id, name, definition, color, is_default, created_at, updated_at
           FROM custom_indicators WHERE user_id = $1 AND id = ANY($2)",
    )
    .bind(user_id)
    .bind(indicator_ids)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let bars = crate::prices::get_bars(pool, symbol, interval, from, to)
        .await
        .unwrap_or_default();
    let times: Vec<DateTime<Utc>> = bars.iter().map(|b| b.bar_time).collect();
    let closes: Vec<f64> = bars.iter().map(|b| dec(b.close)).collect();
    let highs: Vec<f64> = bars.iter().map(|b| dec(b.high)).collect();
    let lows: Vec<f64> = bars.iter().map(|b| dec(b.low)).collect();
    let interval_str = interval.label();

    let mut series_out: Vec<EvalSeries> = Vec::new();
    for ind in &indicators {
        for s in compute_one(
            &ind.name,
            &ind.color,
            &ind.definition,
            &closes,
            &highs,
            &lows,
        ) {
            series_out.push(s);
        }
    }
    Ok(EvalResult {
        symbol: symbol.to_string(),
        interval: interval_str.into(),
        times,
        closes,
        series: series_out,
    })
}

fn compute_one(
    base_name: &str,
    color: &str,
    def: &Value,
    closes: &[f64],
    highs: &[f64],
    lows: &[f64],
) -> Vec<EvalSeries> {
    let kind = def["kind"].as_str().unwrap_or("");
    let p = &def["params"];
    let p_usize = |k: &str, default: usize| p[k].as_u64().map(|x| x as usize).unwrap_or(default);
    let p_f64 = |k: &str, default: f64| p[k].as_f64().unwrap_or(default);
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
                EvalSeries {
                    name: format!("{} mid", base_name),
                    color: color.into(),
                    values: b.middle,
                },
                EvalSeries {
                    name: format!("{} upper", base_name),
                    color: color.into(),
                    values: b.upper,
                },
                EvalSeries {
                    name: format!("{} lower", base_name),
                    color: color.into(),
                    values: b.lower,
                },
            ]
        }
        "macd" => {
            let m = macd(
                closes,
                p_usize("fast", 12),
                p_usize("slow", 26),
                p_usize("signal", 9),
            );
            vec![
                EvalSeries {
                    name: format!("{} MACD", base_name),
                    color: color.into(),
                    values: m.line,
                },
                EvalSeries {
                    name: format!("{} signal", base_name),
                    color: color.into(),
                    values: m.signal,
                },
                EvalSeries {
                    name: format!("{} hist", base_name),
                    color: color.into(),
                    values: m.histogram,
                },
            ]
        }
        "adx" => {
            let a = adx(highs, lows, closes, p_usize("period", 14));
            let period = p_usize("period", 14);
            vec![
                EvalSeries {
                    name: format!("{} ADX({})", base_name, period),
                    color: color.into(),
                    values: a.adx,
                },
                EvalSeries {
                    name: format!("{} +DI", base_name),
                    color: color.into(),
                    values: a.plus_di,
                },
                EvalSeries {
                    name: format!("{} -DI", base_name),
                    color: color.into(),
                    values: a.minus_di,
                },
            ]
        }
        "stochastic" => {
            let st = stochastic(highs, lows, closes, p_usize("k", 14), p_usize("d", 3));
            vec![
                EvalSeries {
                    name: format!("{} %K", base_name),
                    color: color.into(),
                    values: st.k,
                },
                EvalSeries {
                    name: format!("{} %D", base_name),
                    color: color.into(),
                    values: st.d,
                },
            ]
        }
        "atr" => vec![EvalSeries {
            name: format!("{} ATR({})", base_name, p_usize("period", 14)),
            color: color.into(),
            values: atr(highs, lows, closes, p_usize("period", 14)),
        }],
        _ => Vec::new(),
    }
}

pub fn validate(def: &Value) -> anyhow::Result<()> {
    let kind = def["kind"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("kind required"))?;
    match kind {
        "sma" | "ema" | "rsi" | "atr" => {
            let p = def["params"]["period"].as_u64().unwrap_or(0);
            if !(2..=400).contains(&(p as i64)) {
                anyhow::bail!("period must be 2..=400");
            }
        }
        "adx" => {
            let p = def["params"]["period"].as_u64().unwrap_or(0);
            if !(2..=200).contains(&(p as i64)) {
                anyhow::bail!("period must be 2..=200");
            }
        }
        "stochastic" => {
            for (key, lo, hi) in [("k", 1_i64, 400_i64), ("d", 1, 100)] {
                let v = def["params"][key].as_u64().unwrap_or(0) as i64;
                if !(lo..=hi).contains(&v) {
                    anyhow::bail!("{key} must be {lo}..={hi}");
                }
            }
        }
        "bollinger" => {
            let p = def["params"]["period"].as_u64().unwrap_or(0);
            let k = def["params"]["k"].as_f64().unwrap_or(0.0);
            if !(2..=400).contains(&(p as i64)) {
                anyhow::bail!("period must be 2..=400");
            }
            if !(0.1..=5.0).contains(&k) {
                anyhow::bail!("k must be 0.1..=5.0");
            }
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

fn dec(d: Decimal) -> f64 {
    d.to_string().parse().unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ===========================================================================
    // validate — kind required
    // ===========================================================================

    #[test]
    fn validate_rejects_missing_kind() {
        let def = json!({"params": {"period": 20}});
        let err = validate(&def).unwrap_err().to_string();
        assert!(err.contains("kind required"));
    }

    #[test]
    fn validate_rejects_unknown_kind() {
        let def = json!({"kind": "supertrend", "params": {}});
        let err = validate(&def).unwrap_err().to_string();
        assert!(err.contains("unknown kind"));
    }

    // ===========================================================================
    // validate — sma / ema / rsi share the 2..=400 period bound
    // ===========================================================================

    #[test]
    fn validate_sma_accepts_period_in_range() {
        for p in [2_u64, 20, 200, 400] {
            let def = json!({"kind": "sma", "params": {"period": p}});
            assert!(validate(&def).is_ok(), "sma period {} should validate", p);
        }
    }

    #[test]
    fn validate_sma_rejects_period_below_2_or_above_400() {
        for p in [0_u64, 1, 401, 1000] {
            let def = json!({"kind": "sma", "params": {"period": p}});
            assert!(
                validate(&def).is_err(),
                "sma period {} should be rejected",
                p
            );
        }
    }

    #[test]
    fn validate_ema_period_uses_same_bounds_as_sma() {
        let bad = json!({"kind": "ema", "params": {"period": 1}});
        assert!(validate(&bad).is_err());
        let good = json!({"kind": "ema", "params": {"period": 50}});
        assert!(validate(&good).is_ok());
    }

    #[test]
    fn validate_rsi_period_uses_same_bounds_as_sma() {
        let bad = json!({"kind": "rsi", "params": {"period": 401}});
        assert!(validate(&bad).is_err());
        let good = json!({"kind": "rsi", "params": {"period": 14}});
        assert!(validate(&good).is_ok());
    }

    #[test]
    fn validate_period_missing_defaults_to_zero_and_fails() {
        // Without params.period, default is 0 which is outside 2..=400.
        let def = json!({"kind": "sma", "params": {}});
        assert!(validate(&def).is_err());
    }

    // ===========================================================================
    // validate — bollinger needs period AND k
    // ===========================================================================

    #[test]
    fn validate_bollinger_accepts_typical_settings() {
        let def = json!({"kind": "bollinger", "params": {"period": 20, "k": 2.0}});
        assert!(validate(&def).is_ok());
    }

    #[test]
    fn validate_bollinger_rejects_k_below_min() {
        let def = json!({"kind": "bollinger", "params": {"period": 20, "k": 0.05}});
        let err = validate(&def).unwrap_err().to_string();
        assert!(err.contains("k must be"));
    }

    #[test]
    fn validate_bollinger_rejects_k_above_max() {
        let def = json!({"kind": "bollinger", "params": {"period": 20, "k": 5.5}});
        assert!(validate(&def).is_err());
    }

    #[test]
    fn validate_bollinger_accepts_k_at_boundaries() {
        let lo = json!({"kind": "bollinger", "params": {"period": 20, "k": 0.1}});
        let hi = json!({"kind": "bollinger", "params": {"period": 20, "k": 5.0}});
        assert!(validate(&lo).is_ok());
        assert!(validate(&hi).is_ok());
    }

    #[test]
    fn validate_bollinger_rejects_bad_period_even_if_k_ok() {
        let def = json!({"kind": "bollinger", "params": {"period": 0, "k": 2.0}});
        assert!(validate(&def).is_err());
    }

    // ===========================================================================
    // validate — macd requires fast / slow / signal in 2..=200
    // ===========================================================================

    #[test]
    fn validate_macd_accepts_classic_12_26_9() {
        let def = json!({"kind": "macd", "params": {"fast": 12, "slow": 26, "signal": 9}});
        assert!(validate(&def).is_ok());
    }

    #[test]
    fn validate_macd_rejects_any_param_out_of_range() {
        for (k, v) in [("fast", 1_u64), ("slow", 201), ("signal", 0)] {
            let mut params = json!({"fast": 12, "slow": 26, "signal": 9});
            params[k] = json!(v);
            let def = json!({"kind": "macd", "params": params});
            let err = validate(&def).unwrap_err().to_string();
            assert!(
                err.contains(k),
                "{k}={v} should fail with msg containing {k}"
            );
        }
    }

    #[test]
    fn validate_macd_missing_param_is_treated_as_zero_and_fails() {
        let def = json!({"kind": "macd", "params": {"fast": 12, "slow": 26}});
        // signal missing → 0 → rejected.
        assert!(validate(&def).is_err());
    }

    // ===========================================================================
    // validate — atr / adx / stochastic (added to the global catalog)
    // ===========================================================================

    #[test]
    fn validate_atr_uses_same_bounds_as_sma() {
        assert!(validate(&json!({"kind": "atr", "params": {"period": 14}})).is_ok());
        assert!(validate(&json!({"kind": "atr", "params": {"period": 1}})).is_err());
        assert!(validate(&json!({"kind": "atr", "params": {"period": 401}})).is_err());
    }

    #[test]
    fn validate_adx_accepts_2_to_200() {
        assert!(validate(&json!({"kind": "adx", "params": {"period": 14}})).is_ok());
        assert!(validate(&json!({"kind": "adx", "params": {"period": 200}})).is_ok());
        assert!(validate(&json!({"kind": "adx", "params": {"period": 1}})).is_err());
        assert!(validate(&json!({"kind": "adx", "params": {"period": 201}})).is_err());
    }

    #[test]
    fn validate_stochastic_requires_k_and_d_in_range() {
        assert!(validate(&json!({"kind": "stochastic", "params": {"k": 14, "d": 3}})).is_ok());
        // d out of range
        let err = validate(&json!({"kind": "stochastic", "params": {"k": 14, "d": 0}}))
            .unwrap_err()
            .to_string();
        assert!(err.contains("d must be"));
        // k out of range
        assert!(validate(&json!({"kind": "stochastic", "params": {"k": 401, "d": 3}})).is_err());
    }

    #[test]
    fn compute_one_adx_emits_three_series() {
        let n = 60;
        let highs: Vec<f64> = (0..n).map(|i| 100.0 + (i as f64)).collect();
        let lows: Vec<f64> = highs.iter().map(|h| h - 2.0).collect();
        let closes: Vec<f64> = highs.iter().map(|h| h - 1.0).collect();
        let def = json!({"kind": "adx", "params": {"period": 14}});
        let out = compute_one("X", "#fff", &def, &closes, &highs, &lows);
        assert_eq!(out.len(), 3);
        assert!(out[0].name.contains("ADX(14)"));
        assert!(out[1].name.contains("+DI"));
        assert!(out[2].name.contains("-DI"));
    }

    #[test]
    fn compute_one_stochastic_emits_k_and_d() {
        let n = 40;
        let highs: Vec<f64> = (0..n).map(|i| 100.0 + (i % 5) as f64).collect();
        let lows: Vec<f64> = highs.iter().map(|h| h - 3.0).collect();
        let closes: Vec<f64> = highs.iter().map(|h| h - 1.5).collect();
        let def = json!({"kind": "stochastic", "params": {"k": 14, "d": 3}});
        let out = compute_one("S", "#0f0", &def, &closes, &highs, &lows);
        assert_eq!(out.len(), 2);
        assert!(out[0].name.contains("%K"));
        assert!(out[1].name.contains("%D"));
    }

    #[test]
    fn compute_one_atr_emits_single_series() {
        let n = 30;
        let highs: Vec<f64> = (0..n).map(|i| 50.0 + i as f64).collect();
        let lows: Vec<f64> = highs.iter().map(|h| h - 1.0).collect();
        let closes: Vec<f64> = highs.iter().map(|h| h - 0.5).collect();
        let def = json!({"kind": "atr", "params": {"period": 14}});
        let out = compute_one("A", "#00f", &def, &closes, &highs, &lows);
        assert_eq!(out.len(), 1);
        assert!(out[0].name.contains("ATR(14)"));
    }

    // ===========================================================================
    // dec helper
    // ===========================================================================

    #[test]
    fn dec_handles_zero_and_negative() {
        assert_eq!(dec(Decimal::ZERO), 0.0);
        assert_eq!(dec(Decimal::from(-200)), -200.0);
        assert!((dec(Decimal::new(100001, 3)) - 100.001).abs() < 1e-9);
    }

    // ===========================================================================
    // default_color
    // ===========================================================================

    #[test]
    fn default_color_is_cyan_hex() {
        // The frontend chart palette expects #00e5ff specifically.
        assert_eq!(default_color(), "#00e5ff");
    }
}
