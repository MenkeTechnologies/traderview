//! Outbound webhooks for alert fan-out.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

const UA: &str = "traderview/0.1 (github.com/MenkeTechnologies/traderview)";

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent(UA)
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .unwrap()
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Webhook {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub kind: String,
    pub url: String,
    pub secret: Option<String>,
    pub enabled: bool,
    pub last_fired_at: Option<DateTime<Utc>>,
    pub fire_count: i32,
    pub last_status: Option<String>,
    pub created_at: DateTime<Utc>,
}

pub async fn list(pool: &PgPool, user_id: Uuid) -> anyhow::Result<Vec<Webhook>> {
    Ok(sqlx::query_as::<_, Webhook>(
        "SELECT id, user_id, name, kind::text, url, secret, enabled,
                last_fired_at, fire_count, last_status, created_at
           FROM webhooks WHERE user_id = $1 ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?)
}

pub async fn create(
    pool: &PgPool,
    user_id: Uuid,
    name: &str,
    kind: &str,
    url: &str,
    secret: Option<&str>,
) -> anyhow::Result<Webhook> {
    Ok(sqlx::query_as::<_, Webhook>(
        "INSERT INTO webhooks (user_id, name, kind, url, secret)
              VALUES ($1, $2, $3::webhook_kind_t, $4, $5)
         RETURNING id, user_id, name, kind::text, url, secret, enabled,
                   last_fired_at, fire_count, last_status, created_at",
    )
    .bind(user_id)
    .bind(name)
    .bind(kind)
    .bind(url)
    .bind(secret)
    .fetch_one(pool)
    .await?)
}

pub async fn delete(pool: &PgPool, user_id: Uuid, id: Uuid) -> anyhow::Result<bool> {
    let r = sqlx::query("DELETE FROM webhooks WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(r.rows_affected() > 0)
}

pub async fn toggle(pool: &PgPool, user_id: Uuid, id: Uuid, enabled: bool) -> anyhow::Result<bool> {
    let r = sqlx::query("UPDATE webhooks SET enabled = $3 WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .bind(enabled)
        .execute(pool)
        .await?;
    Ok(r.rows_affected() > 0)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertPayload {
    pub title: String,
    pub message: String,
    pub symbol: Option<String>,
    pub kind: String, // "price_alert" | "disclosure" | "sentiment" | etc.
    pub url: Option<String>,
    pub fired_at: DateTime<Utc>,
}

/// Fire a payload to one webhook. Updates last_status + counters.
pub async fn fire(pool: &PgPool, wh: &Webhook, payload: &AlertPayload) -> anyhow::Result<()> {
    let status = match send(wh, payload).await {
        Ok(s) => s,
        Err(e) => format!("err: {e}"),
    };
    let _ = sqlx::query(
        "UPDATE webhooks SET last_fired_at = now(), fire_count = fire_count + 1,
                              last_status = $2 WHERE id = $1",
    )
    .bind(wh.id)
    .bind(&status)
    .execute(pool)
    .await;
    Ok(())
}

async fn send(wh: &Webhook, p: &AlertPayload) -> anyhow::Result<String> {
    let body = match wh.kind.as_str() {
        "discord" => discord_body(p),
        "slack" => slack_body(p),
        _ => serde_json::to_value(p)?,
    };
    let mut req = client().post(&wh.url).json(&body);
    if let Some(secret) = &wh.secret {
        req = req.header("X-Webhook-Secret", secret);
    }
    let resp = req.send().await?;
    Ok(format!("{}", resp.status()))
}

fn discord_body(p: &AlertPayload) -> serde_json::Value {
    let color = match p.kind.as_str() {
        "price_alert" => 0x00e5ff,
        "disclosure" => 0xff2a6d,
        "sentiment" => 0xb86bff,
        "earnings_iv" => 0xffdd57,
        // Algo lifecycle events — distinctive red for engagement so it
        // pops in a busy Discord/Slack channel, neutral grey for release.
        "algo_risk_breach" | "algo_kill_engaged" => 0xff0033,
        "algo_kill_released" => 0x9a9a9a,
        _ => 0x23d160,
    };
    serde_json::json!({
        "username": "TraderView",
        "embeds": [{
            "title": p.title,
            "description": p.message,
            "color": color,
            "url": p.url,
            "timestamp": p.fired_at.to_rfc3339(),
            "fields": p.symbol.as_ref().map(|s| serde_json::json!([
                { "name": "Symbol", "value": s, "inline": true },
                { "name": "Kind",   "value": p.kind, "inline": true },
            ])).unwrap_or(serde_json::json!([])),
        }],
    })
}

fn slack_body(p: &AlertPayload) -> serde_json::Value {
    let header = if let Some(s) = &p.symbol {
        format!("*{}* — {}", s, p.title)
    } else {
        p.title.clone()
    };
    serde_json::json!({
        "text": format!("{}\n{}", header, p.message),
        "blocks": [
            { "type": "header", "text": { "type": "plain_text", "text": format!("TraderView · {}", p.kind) } },
            { "type": "section", "text": { "type": "mrkdwn", "text": format!("{}\n{}", header, p.message) } },
            { "type": "context", "elements": [
                { "type": "mrkdwn", "text": format!("<!date^{}^{{date_short}} {{time}}|{}> ", p.fired_at.timestamp(), p.fired_at.to_rfc3339()) }
            ]}
        ],
    })
}

/// Fan-out to every enabled webhook belonging to the user. Used by the
/// Risk Gate when a Block-severity rule fires — the user wants ALL their
/// alert sinks to know (Discord + Slack + generic), not a chosen subset.
pub async fn fan_out_all(pool: &PgPool, user_id: Uuid, payload: &AlertPayload) {
    let rows: Vec<Webhook> = match sqlx::query_as::<_, Webhook>(
        "SELECT id, user_id, name, kind::text, url, secret, enabled,
                last_fired_at, fire_count, last_status, created_at
           FROM webhooks WHERE user_id = $1 AND enabled = TRUE",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    {
        Ok(r) => r,
        Err(_) => return,
    };
    for wh in rows {
        let _ = fire(pool, &wh, payload).await;
    }
}

/// Fan-out to all webhook IDs (called by alert engines).
pub async fn fan_out(pool: &PgPool, user_id: Uuid, webhook_ids: &[Uuid], payload: &AlertPayload) {
    if webhook_ids.is_empty() {
        return;
    }
    let rows: Vec<Webhook> = match sqlx::query_as::<_, Webhook>(
        "SELECT id, user_id, name, kind::text, url, secret, enabled,
                last_fired_at, fire_count, last_status, created_at
           FROM webhooks WHERE user_id = $1 AND enabled = TRUE AND id = ANY($2)",
    )
    .bind(user_id)
    .bind(webhook_ids)
    .fetch_all(pool)
    .await
    {
        Ok(r) => r,
        Err(_) => return,
    };
    for wh in rows {
        let _ = fire(pool, &wh, payload).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_payload() -> AlertPayload {
        AlertPayload {
            title: "Risk Gate vetoed AAPL entry".into(),
            message: "[max_loss_per_day_pct] today's loss has hit the 3% daily cap".into(),
            symbol: Some("AAPL".into()),
            kind: "risk_gate_block".into(),
            url: None,
            fired_at: Utc::now(),
        }
    }

    #[test]
    fn discord_body_includes_title_and_message() {
        let s = discord_body(&sample_payload()).to_string();
        assert!(s.contains("Risk Gate vetoed AAPL"));
        assert!(s.contains("daily cap"));
    }

    #[test]
    fn discord_body_color_varies_by_kind() {
        let mut p = sample_payload();
        p.kind = "price_alert".into();
        let s1 = discord_body(&p).to_string();
        p.kind = "disclosure".into();
        let s2 = discord_body(&p).to_string();
        assert_ne!(s1, s2, "different kinds must produce different bodies");
    }

    #[test]
    fn discord_body_handles_missing_symbol() {
        let mut p = sample_payload();
        p.symbol = None;
        let body = discord_body(&p);
        let fields = body
            .get("embeds")
            .and_then(|e| e.get(0))
            .and_then(|e| e.get("fields"))
            .expect("embed has fields");
        let arr = fields.as_array().expect("fields is array");
        assert_eq!(arr.len(), 0, "no symbol → empty fields array, not crash");
    }

    #[test]
    fn slack_body_includes_symbol_in_header() {
        let s = slack_body(&sample_payload()).to_string();
        assert!(s.contains("AAPL"));
        assert!(s.contains("Risk Gate vetoed"));
    }

    #[test]
    fn slack_body_omits_symbol_prefix_when_missing() {
        let mut p = sample_payload();
        p.symbol = None;
        let body = slack_body(&p);
        let text = body.get("text").and_then(|t| t.as_str()).expect("text");
        assert!(!text.contains("*None*"));
        assert!(!text.contains("**"), "no orphan asterisks from None.symbol");
    }

    #[test]
    fn alert_payload_serializes_to_json_for_generic_webhooks() {
        // Generic webhooks (the fallback path in `send`) receive the
        // payload as raw JSON. Verify the shape is renderable.
        let v = serde_json::to_value(sample_payload()).unwrap();
        let obj = v.as_object().expect("payload is JSON object");
        for k in ["title", "message", "kind", "fired_at"] {
            assert!(obj.contains_key(k), "missing key `{k}`");
        }
    }
}
