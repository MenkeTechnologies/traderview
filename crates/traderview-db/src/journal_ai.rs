//! LLM-powered trade journal analysis.
//!
//! Pipeline:
//!   1. `build_context()` assembles a deterministic JSON payload from the
//!      trade row + executions + journal entries.
//!   2. SHA-256 of that payload is the cache key. `get_cached()` short-
//!      circuits if a row exists for (user, trade, hash).
//!   3. `analyze()` reads the user's LLM settings, dispatches to OpenAI /
//!      Anthropic / Ollama, parses the structured response, and writes a
//!      cache row.
//!
//! The structured response contract is a JSON object:
//!   {
//!     "summary":     "short string",
//!     "mistakes":    ["..."],
//!     "risk_gaps":   ["..."],
//!     "suggestions": ["..."],
//!     "rule_changes":["..."]
//!   }
//! Providers that don't support strict JSON-mode get a stern prompt instruction
//! and a best-effort fallback parser.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LlmProvider { Openai, Anthropic, Ollama }

#[derive(Debug, Clone)]
pub struct LlmConfig {
    pub provider: LlmProvider,
    pub endpoint: Option<String>,
    pub model: String,
    pub api_key: Option<String>,
    pub max_tokens: i32,
    pub temperature: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Findings {
    pub summary: String,
    pub mistakes: Vec<String>,
    pub risk_gaps: Vec<String>,
    pub suggestions: Vec<String>,
    pub rule_changes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct CachedAnalysis {
    pub id: Uuid,
    pub user_id: Uuid,
    pub trade_id: Uuid,
    pub content_hash: String,
    pub provider: String,
    pub model: String,
    pub prompt_tokens: Option<i32>,
    pub response_tokens: Option<i32>,
    pub findings: Value,
    pub raw_response: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfigDto {
    pub provider: Option<String>,
    pub endpoint: Option<String>,
    pub model: Option<String>,
    pub api_key: Option<String>,
    pub max_tokens: Option<i32>,
    pub temperature: Option<f32>,
}

pub async fn get_llm_settings(pool: &PgPool, user_id: Uuid) -> anyhow::Result<LlmConfigDto> {
    let row: Option<(Option<String>, Option<String>, Option<String>, Option<String>, Option<i32>, Option<f32>)> =
        sqlx::query_as(
            "SELECT llm_provider, llm_endpoint, llm_model, llm_api_key, llm_max_tokens, llm_temperature
               FROM user_settings WHERE user_id = $1",
        ).bind(user_id).fetch_optional(pool).await?;
    Ok(match row {
        Some((p, e, m, k, t, tp)) => LlmConfigDto {
            provider: p, endpoint: e, model: m,
            // Redact the API key — return whether it's set, not the value.
            api_key: k.map(|_| "***".into()),
            max_tokens: t, temperature: tp,
        },
        None => LlmConfigDto {
            provider: None, endpoint: None, model: None, api_key: None,
            max_tokens: None, temperature: None,
        },
    })
}

/// Update LLM settings. If `api_key` is None or `"***"`, the existing key
/// is preserved (so the UI never needs to re-enter it after first save).
pub async fn set_llm_settings(pool: &PgPool, user_id: Uuid, dto: &LlmConfigDto) -> anyhow::Result<()> {
    // Make sure a user_settings row exists.
    sqlx::query("INSERT INTO user_settings (user_id) VALUES ($1) ON CONFLICT DO NOTHING")
        .bind(user_id).execute(pool).await?;

    let key_supplied = matches!(dto.api_key.as_deref(), Some(k) if k != "***" && !k.is_empty());
    if key_supplied {
        sqlx::query(
            "UPDATE user_settings SET
                llm_provider    = $2,
                llm_endpoint    = $3,
                llm_model       = $4,
                llm_api_key     = $5,
                llm_max_tokens  = $6,
                llm_temperature = $7
              WHERE user_id = $1",
        )
        .bind(user_id)
        .bind(&dto.provider).bind(&dto.endpoint).bind(&dto.model)
        .bind(&dto.api_key).bind(dto.max_tokens).bind(dto.temperature)
        .execute(pool).await?;
    } else {
        sqlx::query(
            "UPDATE user_settings SET
                llm_provider    = $2,
                llm_endpoint    = $3,
                llm_model       = $4,
                llm_max_tokens  = $5,
                llm_temperature = $6
              WHERE user_id = $1",
        )
        .bind(user_id)
        .bind(&dto.provider).bind(&dto.endpoint).bind(&dto.model)
        .bind(dto.max_tokens).bind(dto.temperature)
        .execute(pool).await?;
    }
    Ok(())
}

/// Load user_settings → LlmConfig. Returns None if no provider configured.
pub async fn load_config(pool: &PgPool, user_id: Uuid) -> anyhow::Result<Option<LlmConfig>> {
    let row: Option<(Option<String>, Option<String>, Option<String>, Option<String>, Option<i32>, Option<f32>)> =
        sqlx::query_as(
            "SELECT llm_provider, llm_endpoint, llm_model, llm_api_key, llm_max_tokens, llm_temperature
               FROM user_settings WHERE user_id = $1",
        ).bind(user_id).fetch_optional(pool).await?;
    let Some((provider, endpoint, model, api_key, max_t, temp)) = row else { return Ok(None) };
    let Some(provider_str) = provider else { return Ok(None) };
    let Some(model) = model else { return Ok(None) };
    let provider = match provider_str.as_str() {
        "openai" => LlmProvider::Openai,
        "anthropic" => LlmProvider::Anthropic,
        "ollama" => LlmProvider::Ollama,
        _ => return Ok(None),
    };
    Ok(Some(LlmConfig {
        provider, endpoint, model, api_key,
        max_tokens: max_t.unwrap_or(800),
        temperature: temp.unwrap_or(0.2),
    }))
}

/// Build the deterministic JSON context payload + its SHA-256 hash.
/// Hashing the SERIALIZED payload (not field-by-field) keeps the key stable.
pub async fn build_context(pool: &PgPool, user_id: Uuid, trade_id: Uuid)
    -> anyhow::Result<(Value, String)>
{
    let trade = crate::trades::get(pool, trade_id).await?
        .ok_or_else(|| anyhow::anyhow!("trade not found"))?;
    let executions = crate::executions::list_for_trade(pool, trade_id).await.unwrap_or_default();
    let journal_entries = crate::journal::list_for_trade(pool, user_id, trade_id).await.unwrap_or_default();

    let ctx = json!({
        "trade": {
            "symbol": trade.symbol,
            "side": format!("{:?}", trade.side).to_lowercase(),
            "status": format!("{:?}", trade.status).to_lowercase(),
            "asset_class": format!("{:?}", trade.asset_class).to_lowercase(),
            "opened_at": trade.opened_at,
            "closed_at": trade.closed_at,
            "qty": trade.qty.to_string(),
            "entry_avg": trade.entry_avg.to_string(),
            "exit_avg": trade.exit_avg.map(|d| d.to_string()),
            "gross_pnl": trade.gross_pnl.map(|d| d.to_string()),
            "fees": trade.fees.to_string(),
            "net_pnl": trade.net_pnl.map(|d| d.to_string()),
            "stop_loss": trade.stop_loss.map(|d| d.to_string()),
            "risk_amount": trade.risk_amount.map(|d| d.to_string()),
            "initial_target": trade.initial_target.map(|d| d.to_string()),
            "mfe": trade.mfe.map(|d| d.to_string()),
            "mae": trade.mae.map(|d| d.to_string()),
            "best_exit_pnl": trade.best_exit_pnl.map(|d| d.to_string()),
            "exit_efficiency": trade.exit_efficiency.map(|d| d.to_string()),
        },
        "executions": executions.iter().map(|e| json!({
            "side": format!("{:?}", e.side).to_lowercase(),
            "qty": e.qty.to_string(),
            "price": e.price.to_string(),
            "fee": e.fee.to_string(),
            "executed_at": e.executed_at,
        })).collect::<Vec<_>>(),
        "journal_entries": journal_entries.iter().map(|j| json!({
            "day": j.day,
            "body_md": j.body_md,
            "mood": j.mood,
        })).collect::<Vec<_>>(),
    });

    let canonical = serde_json::to_string(&ctx).unwrap_or_default();
    let hash = format!("{:x}", Sha256::digest(canonical.as_bytes()));
    Ok((ctx, hash))
}

pub async fn get_cached(pool: &PgPool, user_id: Uuid, trade_id: Uuid, hash: &str)
    -> anyhow::Result<Option<CachedAnalysis>>
{
    let row: Option<CachedAnalysis> = sqlx::query_as(
        "SELECT id, user_id, trade_id, content_hash, provider, model,
                prompt_tokens, response_tokens, findings, raw_response, created_at
           FROM journal_analyses
          WHERE user_id = $1 AND trade_id = $2 AND content_hash = $3",
    )
    .bind(user_id).bind(trade_id).bind(hash)
    .fetch_optional(pool).await?;
    Ok(row)
}

pub async fn analyze(pool: &PgPool, user_id: Uuid, trade_id: Uuid)
    -> anyhow::Result<CachedAnalysis>
{
    let (ctx, hash) = build_context(pool, user_id, trade_id).await?;
    if let Some(cached) = get_cached(pool, user_id, trade_id, &hash).await? {
        return Ok(cached);
    }
    let cfg = load_config(pool, user_id).await?
        .ok_or_else(|| anyhow::anyhow!("no LLM provider configured; set one on Settings"))?;

    let prompt = build_prompt(&ctx);
    let (raw, prompt_tokens, response_tokens) = call_llm(&cfg, &prompt).await?;
    let findings = parse_findings(&raw);
    let provider_str = match cfg.provider {
        LlmProvider::Openai => "openai",
        LlmProvider::Anthropic => "anthropic",
        LlmProvider::Ollama => "ollama",
    };

    let row: CachedAnalysis = sqlx::query_as(
        "INSERT INTO journal_analyses
           (user_id, trade_id, content_hash, provider, model, prompt_tokens,
            response_tokens, findings, raw_response)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
         RETURNING id, user_id, trade_id, content_hash, provider, model,
                   prompt_tokens, response_tokens, findings, raw_response, created_at",
    )
    .bind(user_id).bind(trade_id).bind(&hash)
    .bind(provider_str).bind(&cfg.model)
    .bind(prompt_tokens).bind(response_tokens)
    .bind(serde_json::to_value(&findings)?).bind(&raw)
    .fetch_one(pool).await?;
    Ok(row)
}

fn build_prompt(ctx: &Value) -> String {
    format!(
        "You are an expert trading coach reviewing a single trade. Analyze the\n\
         data below and return a JSON object with this exact shape:\n\
         {{\n  \
           \"summary\": \"one-sentence overall assessment\",\n  \
           \"mistakes\": [\"behavioral or execution errors\"],\n  \
           \"risk_gaps\": [\"missing/incorrect stop, oversized position, no plan\"],\n  \
           \"suggestions\": [\"concrete improvements for next time\"],\n  \
           \"rule_changes\": [\"durable rules to add to the trader's playbook\"]\n\
         }}\n\
         Each array should have 1-5 short strings. Be specific (cite numbers from\n\
         the context). Return ONLY the JSON — no preamble, no markdown fences.\n\n\
         TRADE CONTEXT:\n{}",
        serde_json::to_string_pretty(ctx).unwrap_or_default()
    )
}

async fn call_llm(cfg: &LlmConfig, prompt: &str) -> anyhow::Result<(String, Option<i32>, Option<i32>)> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()?;
    match cfg.provider {
        LlmProvider::Openai => {
            let url = cfg.endpoint.clone().unwrap_or_else(|| "https://api.openai.com".into());
            let url = format!("{}/v1/chat/completions", url.trim_end_matches('/'));
            let key = cfg.api_key.as_deref()
                .ok_or_else(|| anyhow::anyhow!("OpenAI requires api_key"))?;
            let req = json!({
                "model": cfg.model,
                "messages": [{ "role": "user", "content": prompt }],
                "max_tokens": cfg.max_tokens,
                "temperature": cfg.temperature,
                "response_format": { "type": "json_object" }
            });
            let resp = client.post(&url).bearer_auth(key).json(&req).send().await?;
            let status = resp.status();
            let v: Value = resp.json().await?;
            if !status.is_success() {
                anyhow::bail!("OpenAI HTTP {}: {}", status, v);
            }
            let text = v["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string();
            let pt = v["usage"]["prompt_tokens"].as_i64().map(|x| x as i32);
            let rt = v["usage"]["completion_tokens"].as_i64().map(|x| x as i32);
            Ok((text, pt, rt))
        }
        LlmProvider::Anthropic => {
            let url = cfg.endpoint.clone().unwrap_or_else(|| "https://api.anthropic.com".into());
            let url = format!("{}/v1/messages", url.trim_end_matches('/'));
            let key = cfg.api_key.as_deref()
                .ok_or_else(|| anyhow::anyhow!("Anthropic requires api_key"))?;
            let req = json!({
                "model": cfg.model,
                "max_tokens": cfg.max_tokens,
                "temperature": cfg.temperature,
                "messages": [{ "role": "user", "content": prompt }],
            });
            let resp = client.post(&url)
                .header("x-api-key", key)
                .header("anthropic-version", "2023-06-01")
                .json(&req).send().await?;
            let status = resp.status();
            let v: Value = resp.json().await?;
            if !status.is_success() {
                anyhow::bail!("Anthropic HTTP {}: {}", status, v);
            }
            let text = v["content"][0]["text"].as_str().unwrap_or("").to_string();
            let pt = v["usage"]["input_tokens"].as_i64().map(|x| x as i32);
            let rt = v["usage"]["output_tokens"].as_i64().map(|x| x as i32);
            Ok((text, pt, rt))
        }
        LlmProvider::Ollama => {
            let url = cfg.endpoint.clone().unwrap_or_else(|| "http://localhost:11434".into());
            let url = format!("{}/api/chat", url.trim_end_matches('/'));
            let req = json!({
                "model": cfg.model,
                "stream": false,
                "format": "json",
                "options": { "temperature": cfg.temperature, "num_predict": cfg.max_tokens },
                "messages": [{ "role": "user", "content": prompt }],
            });
            let resp = client.post(&url).json(&req).send().await?;
            let status = resp.status();
            let v: Value = resp.json().await?;
            if !status.is_success() {
                anyhow::bail!("Ollama HTTP {}: {}", status, v);
            }
            let text = v["message"]["content"].as_str().unwrap_or("").to_string();
            let pt = v["prompt_eval_count"].as_i64().map(|x| x as i32);
            let rt = v["eval_count"].as_i64().map(|x| x as i32);
            Ok((text, pt, rt))
        }
    }
}

fn parse_findings(raw: &str) -> Findings {
    // Strip ```json ... ``` fences if a model added them despite instructions.
    let cleaned = raw.trim();
    let cleaned = cleaned.strip_prefix("```json").unwrap_or(cleaned);
    let cleaned = cleaned.strip_prefix("```").unwrap_or(cleaned);
    let cleaned = cleaned.strip_suffix("```").unwrap_or(cleaned).trim();
    // Try strict parse first.
    if let Ok(f) = serde_json::from_str::<Findings>(cleaned) {
        return f;
    }
    // Try generic Value, then coerce fields with defaults.
    if let Ok(v) = serde_json::from_str::<Value>(cleaned) {
        return Findings {
            summary:     v["summary"].as_str().unwrap_or("(no summary)").to_string(),
            mistakes:    str_array(&v["mistakes"]),
            risk_gaps:   str_array(&v["risk_gaps"]),
            suggestions: str_array(&v["suggestions"]),
            rule_changes: str_array(&v["rule_changes"]),
        };
    }
    // Last-ditch: drop the whole response into summary, no arrays.
    Findings {
        summary: cleaned.chars().take(280).collect(),
        mistakes: vec![], risk_gaps: vec![], suggestions: vec![], rule_changes: vec![],
    }
}

fn str_array(v: &Value) -> Vec<String> {
    v.as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect())
     .unwrap_or_default()
}
