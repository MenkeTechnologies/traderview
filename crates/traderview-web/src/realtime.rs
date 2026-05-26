//! Real-time event hub.
//!
//! In-process `tokio::sync::broadcast` channel. Producers (background pollers,
//! alert engine) call `Hub::publish(Event)`; subscribers (WebSocket clients)
//! receive every event published after their `subscribe()` call.
//!
//! Capacity = 256 — if a slow client falls behind, it gets `Lagged(n)` and
//! we drop them rather than blocking publishers. Events are JSON-serializable
//! and rendered straight to the wire.

use serde::Serialize;
use tokio::sync::broadcast;

pub const CAPACITY: usize = 256;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    /// New regulatory disclosure inserted (EDGAR / Senate / House).
    Disclosure { source: &'static str, inserted: usize },
    /// Sentiment poller tick — both source counts.
    Sentiment  { wsb: usize, stocktwits: usize },
    /// An alert rule fired.
    AlertFired { rule_id: String, symbol: String, message: String },
    /// Heartbeat — server emits one every 30s so clients can detect deadness.
    Ping       { ts: i64 },
}

#[derive(Clone)]
pub struct Hub {
    tx: broadcast::Sender<Event>,
}

impl Hub {
    pub fn new() -> Self {
        let (tx, _rx) = broadcast::channel(CAPACITY);
        Self { tx }
    }
    pub fn publish(&self, e: Event) {
        // `send` errors only when there are zero subscribers — that's fine,
        // events are fire-and-forget.
        let _ = self.tx.send(e);
    }
    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.tx.subscribe()
    }
}

impl Default for Hub {
    fn default() -> Self { Self::new() }
}
