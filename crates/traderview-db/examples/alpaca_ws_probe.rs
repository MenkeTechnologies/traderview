//! Live probe for the Alpaca trade_updates WebSocket.
//!
//! Reads creds out of the running embedded-PG `user_settings` (or env
//! vars as fallback), opens the WS, sends auth + listen, and reports
//! the first authorization/listening acks. Exits 0 on full handshake
//! success, non-zero on auth failure or timeout.
//!
//! Run:
//!   cargo run -p traderview-db --example alpaca_ws_probe -- \
//!     --pg-url "postgres://postgres:PW@localhost:PORT/traderview"
//!
//! Or set env vars:
//!   ALPACA_KEY_ID, ALPACA_SECRET_KEY, ALPACA_PAPER (1/0)
//!
//! Timeout default 15s.

use futures_util::{SinkExt, StreamExt};
use std::time::{Duration, Instant};
use tokio_tungstenite::tungstenite::Message as Ws;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    // rustls 0.23 needs an explicit CryptoProvider when multiple
    // providers (aws-lc-rs + ring) are in the build graph. Standalone
    // examples don't go through reqwest's init path, so install one.
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

    let pg_url = std::env::args().skip_while(|a| a != "--pg-url").nth(1);

    let (key_id, secret, paper) = resolve_creds(pg_url.as_deref()).await?;

    let ws_url = if paper {
        traderview_db::alpaca_trading::PAPER_WS_URL
    } else {
        traderview_db::alpaca_trading::LIVE_WS_URL
    };
    println!(
        "[probe] connecting to {ws_url} (key_id={}…, paper={paper})",
        &key_id[..key_id.len().min(6)]
    );
    let (ws, _) = tokio_tungstenite::connect_async(ws_url).await?;
    let (mut tx, mut rx) = ws.split();

    let auth_msg = serde_json::json!({
        "action": "authenticate",
        "key": key_id,
        "secret": secret,
    })
    .to_string();
    tx.send(Ws::Text(auth_msg)).await?;
    println!("[probe] sent authenticate");

    let listen_msg = serde_json::json!({
        "action": "listen",
        "data": {"streams": ["trade_updates"]},
    })
    .to_string();
    tx.send(Ws::Text(listen_msg)).await?;
    println!("[probe] sent listen");

    let start = Instant::now();
    let timeout = Duration::from_secs(15);
    let mut auth_ok = false;
    let mut listen_ok = false;

    loop {
        if auth_ok && listen_ok {
            println!(
                "[probe] OK — both acks received in {:.2?}; closing",
                start.elapsed()
            );
            // Polite close.
            let _ = tx.send(Ws::Close(None)).await;
            return Ok(());
        }
        if start.elapsed() > timeout {
            anyhow::bail!(
                "timed out after {:?} (auth_ok={auth_ok}, listen_ok={listen_ok})",
                timeout
            );
        }
        let remaining = timeout.saturating_sub(start.elapsed());
        let next = tokio::time::timeout(remaining, rx.next()).await;
        let Ok(Some(msg)) = next else { continue };
        let frame = match msg? {
            Ws::Text(t) => t,
            Ws::Binary(b) => String::from_utf8_lossy(&b).into_owned(),
            Ws::Close(c) => anyhow::bail!("server closed: {c:?}"),
            _ => continue,
        };
        println!("[probe] frame: {frame}");
        let v: serde_json::Value = match serde_json::from_str(&frame) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let stream = v.get("stream").and_then(|x| x.as_str()).unwrap_or("");
        match stream {
            "authorization" => {
                let status = v
                    .get("data")
                    .and_then(|d| d.get("status"))
                    .and_then(|s| s.as_str())
                    .unwrap_or("");
                if status == "authorized" {
                    auth_ok = true;
                } else {
                    anyhow::bail!("auth rejected: {frame}");
                }
            }
            "listening" => {
                let streams = v
                    .get("data")
                    .and_then(|d| d.get("streams"))
                    .and_then(|s| s.as_array())
                    .map(|arr| arr.iter().filter_map(|x| x.as_str()).collect::<Vec<_>>())
                    .unwrap_or_default();
                if streams.contains(&"trade_updates") {
                    listen_ok = true;
                } else {
                    anyhow::bail!("listening ack missing trade_updates: {frame}");
                }
            }
            _ => {}
        }
    }
}

async fn resolve_creds(pg_url: Option<&str>) -> anyhow::Result<(String, String, bool)> {
    if let (Some(k), Some(s)) = (
        std::env::var("ALPACA_KEY_ID").ok().filter(|s| !s.is_empty()),
        std::env::var("ALPACA_SECRET_KEY").ok().filter(|s| !s.is_empty()),
    ) {
        let paper = std::env::var("ALPACA_PAPER")
            .ok()
            .map(|v| !matches!(v.as_str(), "0" | "false" | "no"))
            .unwrap_or(true);
        println!("[probe] using env-var creds");
        return Ok((k, s, paper));
    }
    let Some(url) = pg_url else {
        anyhow::bail!("no ALPACA_KEY_ID/SECRET in env and no --pg-url passed");
    };
    println!("[probe] reading creds from --pg-url");
    let pool = traderview_db::connect_external(url).await?;
    let row: Option<(Option<String>, Option<String>, bool)> = sqlx::query_as(
        "SELECT alpaca_key_id, alpaca_secret_key, alpaca_paper
           FROM user_settings
          WHERE alpaca_key_id IS NOT NULL AND alpaca_key_id <> ''
            AND alpaca_secret_key IS NOT NULL AND alpaca_secret_key <> ''
          ORDER BY updated_at DESC
          LIMIT 1",
    )
    .fetch_optional(&pool)
    .await?;
    match row {
        Some((Some(k), Some(s), p)) => Ok((k, s, p)),
        _ => anyhow::bail!("no Alpaca creds in user_settings"),
    }
}
