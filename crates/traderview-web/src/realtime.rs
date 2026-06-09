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
    Disclosure {
        source: &'static str,
        inserted: usize,
    },
    /// Sentiment poller tick — both source counts.
    Sentiment { wsb: usize, stocktwits: usize },
    /// News poller tick.
    News { inserted: u64, symbols: usize },
    /// An alert rule fired.
    AlertFired {
        rule_id: String,
        symbol: String,
        message: String,
    },
    /// A squeeze detector trigger — emitted by the candidate-driven
    /// scanner when a symbol crosses configured %change + volume-burst
    /// thresholds. See `traderview_db::squeeze_detector::SqueezeEvent`.
    SqueezeFired {
        symbol: String,
        price: f64,
        pct_change: f64,
        burst_ratio: f64,
    },
    /// Heartbeat — server emits one every 30s so clients can detect deadness.
    Ping { ts: i64 },

    /// Algo runner: strategy produced an entry signal on the latest bar.
    AlgoSignalFired {
        strategy_id: String,
        run_id: String,
        symbol: String,
        side: &'static str,
        entry_price: f64,
        kind: &'static str,
    },
    /// Algo runner: order was accepted by the broker.
    AlgoOrderSubmitted {
        strategy_id: String,
        order_id: String,
        symbol: String,
        side: String,
        qty: f64,
        broker_order_id: String,
    },
    /// Algo runner: fill landed in algo_fills + executions pipeline.
    AlgoFillReceived {
        strategy_id: String,
        order_id: String,
        symbol: String,
        qty: f64,
        price: f64,
    },
    /// Algo runner: tick fired for this strategy but skipped without
    /// evaluating (no universe, no symbols, broker pending, etc.).
    /// Lets the UI stdout show "test2 skipped: no_universe — pick a
    /// watchlist or set autoscan" instead of staring at silence.
    AlgoTickSkipped { strategy_id: String, reason: String },
    /// Algo runner: a bar window was fed to the strategy but no
    /// signal fired. Confirms the engine is alive + symbol coverage
    /// is correct.
    AlgoBarEvaluated {
        strategy_id: String,
        symbol: String,
        bars: u64,
    },
    /// Algo runner: per-tick heartbeat — proves the engine is alive
    /// even on M1 strategies that only formally evaluate once a
    /// minute.
    AlgoHeartbeat {
        strategy_id: String,
        universe_size: u64,
        subscribed_live: u64,
        bars_processed: i64,
        signals_emitted: i64,
        seconds_to_next_eval: i64,
    },
    /// Raw trade off the live-tick WS — every parsed Trade is fanned
    /// out so a frontend tape pane can render the unaggregated tick
    /// stream as it arrives (separate from the per-state-update
    /// `SymbolState` events). Useful for proving the WS feed is
    /// genuinely streaming, not synthesized.
    Tick {
        symbol: String,
        price: f64,
        volume: f64,
        ts_ms: i64,
    },
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
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Capacity / construction contract ─────────────────────────────────

    #[test]
    fn capacity_is_documented_256() {
        // The 256-slot ring is what protects publishers from slow consumers.
        // Bumping or shrinking it changes the back-pressure profile of every
        // background poller in the process.
        assert_eq!(CAPACITY, 256);
    }

    #[test]
    fn hub_default_matches_new() {
        // Default impl must be a thin alias for new() — diverging would mean
        // some call sites construct a different-sized hub.
        let a = Hub::new();
        let b = Hub::default();
        // Both freshly-built hubs have no subscribers — publish on either
        // should succeed silently (broadcast::send returns SendError when no
        // receivers, but Hub::publish swallows it; we just exercise the path).
        a.publish(Event::Ping { ts: 1 });
        b.publish(Event::Ping { ts: 2 });
    }

    // ── Publish/subscribe end-to-end ─────────────────────────────────────

    #[test]
    fn subscriber_receives_published_event() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async {
            let hub = Hub::new();
            let mut rx = hub.subscribe();
            hub.publish(Event::News {
                inserted: 7,
                symbols: 3,
            });
            let got = rx.recv().await.expect("event");
            match got {
                Event::News { inserted, symbols } => {
                    assert_eq!(inserted, 7);
                    assert_eq!(symbols, 3);
                }
                other => panic!("wrong variant: {other:?}"),
            }
        });
    }

    #[test]
    fn publish_without_subscribers_does_not_panic() {
        // The whole point of using broadcast over mpsc is fire-and-forget —
        // if no one is listening, the event is dropped on the floor silently.
        let hub = Hub::new();
        hub.publish(Event::Ping { ts: 42 });
        hub.publish(Event::Disclosure {
            source: "edgar",
            inserted: 1,
        });
    }

    // ── Event serde contract — frontend depends on these exact strings ────

    #[test]
    fn event_serializes_with_type_tag_and_snake_case() {
        // The WebSocket protocol depends on {"type": "...", ...}; if the tag
        // attribute changes, every client breaks immediately.
        let v = serde_json::to_value(Event::Disclosure {
            source: "edgar",
            inserted: 5,
        })
        .unwrap();
        assert_eq!(v["type"], "disclosure");
        assert_eq!(v["source"], "edgar");
        assert_eq!(v["inserted"], 5);
    }

    #[test]
    fn alert_fired_uses_snake_case_variant_tag() {
        // CamelCase AlertFired → snake_case "alert_fired" in the JSON output.
        let v = serde_json::to_value(Event::AlertFired {
            rule_id: "r1".into(),
            symbol: "AAPL".into(),
            message: "hit".into(),
        })
        .unwrap();
        assert_eq!(v["type"], "alert_fired");
        assert_eq!(v["rule_id"], "r1");
        assert_eq!(v["symbol"], "AAPL");
    }

    #[test]
    fn sentiment_event_carries_both_source_counts() {
        let v = serde_json::to_value(Event::Sentiment {
            wsb: 4,
            stocktwits: 9,
        })
        .unwrap();
        assert_eq!(v["type"], "sentiment");
        assert_eq!(v["wsb"], 4);
        assert_eq!(v["stocktwits"], 9);
    }

    #[test]
    fn ping_event_serializes_with_ts_field() {
        // Heartbeat shape: clients use ts to compute drift / detect dead links.
        let v = serde_json::to_value(Event::Ping { ts: 1700000000 }).unwrap();
        assert_eq!(v["type"], "ping");
        assert_eq!(v["ts"], 1700000000_i64);
    }
}
