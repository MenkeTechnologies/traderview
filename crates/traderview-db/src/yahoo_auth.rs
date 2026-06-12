//! Shared Yahoo cookie+crumb auth for the `query*.finance.yahoo.com`
//! endpoints that reject anonymous calls with 401 "Invalid Crumb"
//! (v7 options, v10 quoteSummary; the v8 chart endpoint stays open).
//!
//! Flow: hit `fc.yahoo.com` once — it 404s but sets the consent cookie
//! the crumb is bound to — then `/v1/test/getcrumb` returns the crumb
//! for that cookie jar. Client (carrying the jar) and crumb are cached
//! process-wide; callers `invalidate()` on a 401 and retry once, since
//! Yahoo expires crumbs server-side at its own discretion.

use once_cell::sync::Lazy;
use tokio::sync::Mutex;

const UA: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) \
                  AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.4 Safari/605.1.15";

#[derive(Clone)]
pub struct YahooAuth {
    /// Client with the cookie jar the crumb is bound to — requests MUST
    /// go through this client or the crumb is rejected.
    pub client: reqwest::Client,
    pub crumb: String,
}

static CACHE: Lazy<Mutex<Option<YahooAuth>>> = Lazy::new(|| Mutex::new(None));

/// Plausibility gate for a crumb body: Yahoo returns a short opaque
/// token. An empty body, HTML error page, or JSON error envelope means
/// the handshake failed and must not be cached.
fn crumb_looks_valid(s: &str) -> bool {
    !s.is_empty() && s.len() <= 64 && !s.contains('<') && !s.contains('{')
}

pub async fn get() -> anyhow::Result<YahooAuth> {
    let mut guard = CACHE.lock().await;
    if let Some(a) = guard.as_ref() {
        return Ok(a.clone());
    }
    let client = reqwest::Client::builder()
        .user_agent(UA)
        .cookie_store(true)
        .timeout(std::time::Duration::from_secs(10))
        .build()?;
    let _ = client.get("https://fc.yahoo.com").send().await;
    let crumb = client
        .get("https://query1.finance.yahoo.com/v1/test/getcrumb")
        .send()
        .await?
        .text()
        .await?;
    if !crumb_looks_valid(&crumb) {
        anyhow::bail!("yahoo crumb handshake returned unusable body: {:.40}", crumb);
    }
    let auth = YahooAuth { client, crumb };
    *guard = Some(auth.clone());
    Ok(auth)
}

/// Drop the cached auth so the next `get()` re-handshakes. Call on 401
/// with the crumb that failed — if another task already replaced the
/// cache with a fresh crumb, a stale failure must not wipe it (that
/// caused N sequential re-handshakes under concurrent fan-out).
pub async fn invalidate(seen_crumb: &str) {
    let mut guard = CACHE.lock().await;
    if guard.as_ref().is_some_and(|a| a.crumb == seen_crumb) {
        *guard = None;
    }
}

#[cfg(test)]
mod tests {
    use super::crumb_looks_valid;

    #[test]
    fn crumb_accepts_typical_token() {
        assert!(crumb_looks_valid("Y3ilkPDqQV5"));
        assert!(crumb_looks_valid("a.b/c=="));
    }

    #[test]
    fn crumb_rejects_empty_html_and_json_bodies() {
        assert!(!crumb_looks_valid(""));
        assert!(!crumb_looks_valid("<html><body>error</body></html>"));
        assert!(!crumb_looks_valid("{\"finance\":{\"error\":\"Unauthorized\"}}"));
        assert!(!crumb_looks_valid(&"x".repeat(65)));
    }
}
