//! Standalone web server. External Postgres, multi-user.
//!
//! Env vars:
//!   DATABASE_URL              postgres://user:pass@host/db   (required)
//!   TRADERVIEW_JWT_SECRET     hex-encoded, >= 32 bytes       (required)
//!   TRADERVIEW_BIND           default 0.0.0.0:8080
//!   TRADERVIEW_STATIC_DIR     default ../../frontend         (relative to bin)

use clap::Parser;
use std::net::SocketAddr;
use std::path::PathBuf;
use tower_http::services::ServeDir;
use traderview_web::{router, AppMode, AppState};

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, env = "DATABASE_URL")]
    database_url: String,

    #[arg(long, env = "TRADERVIEW_JWT_SECRET")]
    jwt_secret: String,

    #[arg(long, env = "TRADERVIEW_BIND", default_value = "0.0.0.0:8080")]
    bind: SocketAddr,

    #[arg(long, env = "TRADERVIEW_STATIC_DIR", default_value = "frontend")]
    static_dir: PathBuf,

    #[arg(long, env = "TRADERVIEW_DATA_DIR", default_value = "data")]
    data_dir: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Same fix as the desktop bin — install a process-wide rustls
    // crypto provider before any TLS handshake fires.
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "traderview_web=info,tower_http=info".into()),
        )
        .init();

    let args = Args::parse();
    let jwt_secret = hex::decode(&args.jwt_secret)
        .map_err(|e| anyhow::anyhow!("TRADERVIEW_JWT_SECRET must be hex: {e}"))?;
    if jwt_secret.len() < 32 {
        anyhow::bail!("TRADERVIEW_JWT_SECRET must decode to >= 32 bytes");
    }

    let pool = traderview_db::connect_external(&args.database_url).await?;
    traderview_db::migrate(&pool).await?;

    std::fs::create_dir_all(&args.data_dir)?;
    let state = AppState::new(
        pool.clone(),
        AppMode::Web,
        jwt_secret,
        args.data_dir.clone(),
    );

    // Warm the LiveTickStore's in-memory Finnhub key from DB on boot so
    // REST callers (finnhub_rest, market_data fundamentals/earnings/...)
    // resolve the saved Settings → Data Sources key without the user
    // having to re-save after every restart. Env-var fallback applies if
    // no row has one stored. Failure is non-fatal — the key just stays
    // unset and Finnhub-backed endpoints will return 500 with the
    // existing "not configured" error.
    match traderview_db::data_source_keys::any_finnhub_key(&pool).await {
        Ok(Some(k)) => {
            traderview_db::live_ticks::global().set_api_key(k).await;
            tracing::info!("loaded finnhub key from DB into live_ticks store");
        }
        Ok(None) => {
            tracing::info!("no finnhub key configured; set one in Settings → Data Sources");
        }
        Err(e) => tracing::warn!(error = %e, "failed to load finnhub key from DB"),
    }
    // Warm Polygon too — when its key is configured, the live tape
    // prefers Polygon's SIP feed (CTA/UTP) over Finnhub's aggregate.
    match traderview_db::data_source_keys::any_polygon_key(&pool).await {
        Ok(Some(k)) => {
            traderview_db::live_ticks::global().set_polygon_key(k).await;
            tracing::info!("loaded polygon key from DB; live tape will use SIP feed");
        }
        Ok(None) => {
            tracing::info!("no polygon key configured; live tape falls back to finnhub");
        }
        Err(e) => tracing::warn!(error = %e, "failed to load polygon key from DB"),
    }
    // Warm Alpaca — middle provider in the priority chain (Polygon →
    // Alpaca → Finnhub).
    match traderview_db::data_source_keys::any_alpaca_creds(&pool).await {
        Ok(Some((id, secret, use_sip))) => {
            let store = traderview_db::live_ticks::global();
            store.set_alpaca_creds(id, secret).await;
            store.set_alpaca_use_sip(use_sip);
            tracing::info!(
                use_sip,
                "loaded alpaca creds; live tape uses {} feed",
                if use_sip { "SIP" } else { "IEX" }
            );
        }
        Ok(None) => tracing::info!("no alpaca creds configured"),
        Err(e) => tracing::warn!(error = %e, "failed to load alpaca creds from DB"),
    }

    // Hand the pool to the live-tick store so the 10s tape aggregator can
    // persist closed buckets into `price_bars` (interval='10s'). Without
    // this call, incoming trades still update in-memory SymbolState but no
    // 10s rows ever land in the DB — multichart's 10s pane stays empty.
    traderview_db::live_ticks::global()
        .set_pool(pool.clone())
        .await;

    // Background disclosure poller — every 20s for sub-30s EDGAR/Congress alerts.
    {
        let pool = pool.clone();
        let hub = state.hub.clone();
        tokio::spawn(async move {
            loop {
                let r = traderview_db::disclosures::poll_all(&pool).await;
                let total = r.edgar_inserted + r.senate_inserted + r.house_inserted;
                if total > 0 {
                    tracing::info!(
                        edgar = r.edgar_inserted,
                        senate = r.senate_inserted,
                        house = r.house_inserted,
                        "disclosures polled",
                    );
                    if r.edgar_inserted > 0 {
                        hub.publish(traderview_web::realtime::Event::Disclosure {
                            source: "edgar",
                            inserted: r.edgar_inserted,
                        });
                    }
                    if r.senate_inserted > 0 {
                        hub.publish(traderview_web::realtime::Event::Disclosure {
                            source: "senate",
                            inserted: r.senate_inserted,
                        });
                    }
                    if r.house_inserted > 0 {
                        hub.publish(traderview_web::realtime::Event::Disclosure {
                            source: "house",
                            inserted: r.house_inserted,
                        });
                    }
                }
                tokio::time::sleep(std::time::Duration::from_secs(20)).await;
            }
        });
    }

    // Background strategy-alert evaluator — every 60s.
    {
        let pool = pool.clone();
        let hub = state.hub.clone();
        tokio::spawn(async move {
            loop {
                match traderview_db::strategy_alerts::evaluate_all(&pool).await {
                    Ok(s) => {
                        if s.fired > 0 || s.errors > 0 {
                            tracing::info!(
                                evaluated = s.evaluated,
                                fired = s.fired,
                                errors = s.errors,
                                "strategy alerts evaluated",
                            );
                            if s.fired > 0 {
                                hub.publish(traderview_web::realtime::Event::AlertFired {
                                    rule_id: "strategy".into(),
                                    symbol: String::new(),
                                    message: format!("{} strategy alert(s) fired", s.fired),
                                });
                            }
                        }
                    }
                    Err(e) => tracing::warn!(error = %e, "strategy alert eval failed"),
                }
                tokio::time::sleep(std::time::Duration::from_secs(60)).await;
            }
        });
    }

    // Background earnings calendar poller — every 6h over watchlist symbols.
    {
        let pool = pool.clone();
        tokio::spawn(async move {
            loop {
                match traderview_db::earnings_cal::poll_watchlists(&pool).await {
                    Ok(s) => {
                        if s.events_upserted > 0 || s.reactions_computed > 0 {
                            tracing::info!(
                                symbols = s.symbols_polled,
                                events = s.events_upserted,
                                reactions = s.reactions_computed,
                                "earnings polled",
                            );
                        }
                    }
                    Err(e) => tracing::warn!(error = %e, "earnings poll failed"),
                }
                tokio::time::sleep(std::time::Duration::from_secs(6 * 3600)).await;
            }
        });
    }

    // Background news poller — every 5 min over distinct watchlist symbols.
    {
        let pool = pool.clone();
        let hub = state.hub.clone();
        tokio::spawn(async move {
            loop {
                if let Ok(s) = traderview_db::news::poll_watchlists(&pool).await {
                    if s.inserted > 0 {
                        tracing::info!(
                            symbols = s.symbols_polled,
                            inserted = s.inserted,
                            "news polled"
                        );
                        hub.publish(traderview_web::realtime::Event::News {
                            inserted: s.inserted,
                            symbols: s.symbols_polled,
                        });
                    }
                }
                tokio::time::sleep(std::time::Duration::from_secs(300)).await;
            }
        });
    }

    // Background sentiment poller — every 60s, WSB + StockTwits per watchlist symbol.
    {
        let pool = pool.clone();
        let hub = state.hub.clone();
        tokio::spawn(async move {
            loop {
                let (wsb, st) = traderview_db::sentiment::poll_all(&pool).await;
                if wsb + st > 0 {
                    tracing::info!(wsb = wsb, stocktwits = st, "sentiment polled");
                    hub.publish(traderview_web::realtime::Event::Sentiment {
                        wsb,
                        stocktwits: st,
                    });
                }
                tokio::time::sleep(std::time::Duration::from_secs(60)).await;
            }
        });
    }

    // Squeeze scanner — catalyst-driven candidate aggregator + rolling-
    // window squeeze detector. The aggregator subscribes to catalysts +
    // halts firehoses, scores symbols, and pushes the top-N to
    // LiveTickStore (Finnhub WS) every 30s. The detector consumes the
    // tick broadcast and emits SqueezeEvent when thresholds cross.
    // Both pumps are idempotent; one call here is enough.
    {
        let hub = state.hub.clone();
        traderview_db::candidates::spawn_aggregator(traderview_db::candidates::global());
        traderview_db::squeeze_detector::spawn_pump(traderview_db::squeeze_detector::global());
        // Bridge SqueezeEvent → realtime hub so the global WS event stream
        // surfaces squeeze fires alongside disclosures / news / sentiment.
        tokio::spawn(async move {
            let mut rx = traderview_db::squeeze_detector::global().subscribe();
            loop {
                match rx.recv().await {
                    Ok(ev) => {
                        hub.publish(traderview_web::realtime::Event::SqueezeFired {
                            symbol: ev.symbol,
                            price: ev.price,
                            pct_change: ev.pct_change,
                            burst_ratio: ev.burst_ratio,
                        });
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
            }
        });
    }
    // Algo runner — drives every enabled, kill-switch-off strategy on
    // each bar boundary. Reads from price_bars (already populated by the
    // live_ticks worker) so no extra market-data infra is needed here.
    // The event sink wraps Hub::publish, mapping the engine's
    // domain-specific EngineEvent → realtime::Event so algo signals /
    // orders / fills surface on the same /api/ws stream the rest of
    // the app uses.
    {
        let pool = state.pool.clone();
        let hub = state.hub.clone();
        let sink: traderview_db::algo_engine::EventSink =
            std::sync::Arc::new(move |ev: traderview_db::algo_engine::EngineEvent| {
                use traderview_db::algo_engine::EngineEvent as E;
                let side_str = |s: traderview_core::algo_strategies::Side| match s {
                    traderview_core::algo_strategies::Side::Buy => "buy",
                    traderview_core::algo_strategies::Side::Sell => "sell",
                };
                use rust_decimal::prelude::ToPrimitive;
                let evt = match ev {
                    E::SignalFired {
                        strategy_id, run_id, symbol, side, entry_price, kind,
                    } => traderview_web::realtime::Event::AlgoSignalFired {
                        strategy_id: strategy_id.to_string(),
                        run_id: run_id.to_string(),
                        symbol,
                        side: side_str(side),
                        entry_price: entry_price.to_f64().unwrap_or(0.0),
                        kind,
                    },
                    E::OrderSubmitted {
                        strategy_id, order_id, symbol, side, qty, broker_order_id,
                    } => traderview_web::realtime::Event::AlgoOrderSubmitted {
                        strategy_id: strategy_id.to_string(),
                        order_id: order_id.to_string(),
                        symbol,
                        side: side_str(side).into(),
                        qty: qty.to_f64().unwrap_or(0.0),
                        broker_order_id,
                    },
                    E::FillReceived {
                        strategy_id, order_id, symbol, qty, price,
                    } => traderview_web::realtime::Event::AlgoFillReceived {
                        strategy_id: strategy_id.to_string(),
                        order_id: order_id.to_string(),
                        symbol,
                        qty: qty.to_f64().unwrap_or(0.0),
                        price: price.to_f64().unwrap_or(0.0),
                    },
                };
                hub.publish(evt);
            });
        tokio::spawn(traderview_db::algo_runner::run_loop(pool, Some(sink)));
    }

    let api = router(state);

    let static_service = ServeDir::new(&args.static_dir).append_index_html_on_directories(true);
    let app = api.fallback_service(static_service);

    tracing::info!(%args.bind, static_dir = %args.static_dir.display(), "traderview-web listening");
    let listener = tokio::net::TcpListener::bind(args.bind).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
