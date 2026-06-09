//! Multi-broker position aggregation.
//!
//! Five trading bridges live in this crate today: alpaca, ibkr,
//! tradier, schwab, tastytrade. Each exposes its own positions
//! shape. This module normalises them into one `NormalizedPosition`
//! row, fans out across whichever brokers the user has credentials
//! configured for, and aggregates by symbol so the user can see
//! total exposure across the whole stack instead of toggling between
//! five broker UIs.
//!
//! Today's broker-credentials are stored as columns on `user_settings`
//! (per migrations 0036/0051/0059/0060/0061). The route layer reads
//! that row, builds whichever clients the user has tokens for, calls
//! each one's positions endpoint, and feeds the results into
//! `aggregate_by_symbol` below.
//!
//! The conversion helpers + aggregator are pure — they don't touch the
//! DB or any HTTP client — so they're fully unit-tested. The
//! repository layer is best-effort: brokers without credentials are
//! skipped; brokers whose API call fails contribute zero positions
//! and an error entry to the response.

use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use serde::Serialize;

use crate::{alpaca_trading, ibkr_trading, tradier_trading};

#[derive(Debug, Clone, Serialize)]
pub struct NormalizedPosition {
    pub symbol: String,
    pub broker: String,
    /// Positive = long, negative = short.
    pub qty: f64,
    pub avg_cost: Option<f64>,
    pub current_price: Option<f64>,
    pub market_value: Option<f64>,
    pub unrealized_pl: Option<f64>,
    /// IBKR sometimes shows multiple positions per account; this lets
    /// the UI surface "ACME on IBKR-acct-X1234" separately from
    /// "ACME on IBKR-acct-X5678". None for brokers without sub-account
    /// granularity in the position payload.
    pub account_label: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SymbolTotal {
    pub symbol: String,
    pub total_qty: f64,
    pub total_market_value: f64,
    pub total_unrealized_pl: f64,
    pub broker_count: usize,
    pub brokers: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BrokerError {
    pub broker: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MultiBrokerReport {
    pub positions: Vec<NormalizedPosition>,
    pub totals: Vec<SymbolTotal>,
    pub broker_count: usize,
    pub errors: Vec<BrokerError>,
}

// ─── Pure converters (per broker → NormalizedPosition) ─────────────────────

fn dec_to_f64(d: Decimal) -> f64 {
    d.to_f64().unwrap_or(0.0)
}

pub fn from_alpaca(p: alpaca_trading::PositionResponse) -> NormalizedPosition {
    let qty_signed = if p.side.eq_ignore_ascii_case("short") {
        -dec_to_f64(p.qty)
    } else {
        dec_to_f64(p.qty)
    };
    NormalizedPosition {
        symbol: p.symbol,
        broker: "alpaca".into(),
        qty: qty_signed,
        avg_cost: Some(dec_to_f64(p.avg_entry_price)),
        current_price: p.current_price.map(dec_to_f64),
        market_value: p.market_value.map(dec_to_f64),
        unrealized_pl: p.unrealized_pl.map(dec_to_f64),
        account_label: None,
    }
}

pub fn from_ibkr(p: ibkr_trading::Position) -> NormalizedPosition {
    // IBKR's `position` field is signed already (negative for shorts).
    let qty = p.position.unwrap_or(0.0);
    let price = p.mkt_price;
    let avg = p.avg_price;
    let mkt_value = match (qty, price) {
        (q, Some(px)) if q != 0.0 && px > 0.0 => Some(q * px),
        _ => None,
    };
    NormalizedPosition {
        symbol: p.contract_desc.unwrap_or_default(),
        broker: "ibkr".into(),
        qty,
        avg_cost: avg,
        current_price: price,
        market_value: mkt_value,
        unrealized_pl: p.unrealized_pnl,
        account_label: p.account_id,
    }
}

pub fn from_tradier(
    p: tradier_trading::Position,
    current_price: Option<f64>,
) -> NormalizedPosition {
    // Tradier's payload doesn't carry market value or unrealized PL; the
    // caller has to fetch the quote separately for current_price and we
    // derive what we can.
    let qty = dec_to_f64(p.quantity);
    let cost_basis = dec_to_f64(p.cost_basis);
    let avg_cost = if qty != 0.0 {
        Some(cost_basis / qty)
    } else {
        None
    };
    let market_value = match current_price {
        Some(px) if px > 0.0 && qty != 0.0 => Some(qty * px),
        _ => None,
    };
    let unrealized_pl = match (market_value, qty) {
        (Some(mv), q) if q != 0.0 => Some(mv - cost_basis),
        _ => None,
    };
    NormalizedPosition {
        symbol: p.symbol,
        broker: "tradier".into(),
        qty,
        avg_cost,
        current_price,
        market_value,
        unrealized_pl,
        account_label: None,
    }
}

// ─── Pure aggregation ──────────────────────────────────────────────────────

/// Group positions by symbol and sum qty / market_value / unrealized_pl.
/// Sorts by absolute total_market_value descending so the largest
/// dollar-exposure positions surface first. None values are treated as
/// 0.0 in the totals (so a broker that doesn't return market_value
/// just doesn't add to the sum — it still appears in `brokers`).
pub fn aggregate_by_symbol(positions: &[NormalizedPosition]) -> Vec<SymbolTotal> {
    use std::collections::HashMap;
    let mut by_sym: HashMap<String, SymbolTotal> = HashMap::new();
    for p in positions {
        let entry = by_sym.entry(p.symbol.clone()).or_insert(SymbolTotal {
            symbol: p.symbol.clone(),
            total_qty: 0.0,
            total_market_value: 0.0,
            total_unrealized_pl: 0.0,
            broker_count: 0,
            brokers: Vec::new(),
        });
        entry.total_qty += p.qty;
        entry.total_market_value += p.market_value.unwrap_or(0.0);
        entry.total_unrealized_pl += p.unrealized_pl.unwrap_or(0.0);
        if !entry.brokers.contains(&p.broker) {
            entry.brokers.push(p.broker.clone());
            entry.broker_count += 1;
        }
    }
    let mut totals: Vec<SymbolTotal> = by_sym.into_values().collect();
    totals.sort_by(|a, b| {
        b.total_market_value
            .abs()
            .partial_cmp(&a.total_market_value.abs())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    totals
}

// ─── Repository layer ──────────────────────────────────────────────────────

/// Read user_settings columns for whichever brokers have creds, then
/// fan out positions calls. Brokers without creds are skipped. Brokers
/// whose call fails contribute zero positions + an error entry.
///
/// Returns a `MultiBrokerReport` even on full failure (all brokers
/// erroring out) — callers prefer the partial-success shape.
pub async fn fetch_all(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
) -> anyhow::Result<MultiBrokerReport> {
    let row: Option<(
        Option<String>, // alpaca_key
        Option<String>, // alpaca_secret
        Option<String>, // alpaca_mode (paper/live)
        Option<String>, // tradier_access_token
        Option<String>, // tradier_account_id
        Option<bool>,   // tradier_sandbox
    )> = sqlx::query_as(
        "SELECT alpaca_api_key, alpaca_api_secret, alpaca_mode,
                tradier_access_token, tradier_account_id, tradier_sandbox
           FROM user_settings WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .unwrap_or(None);

    let (alpaca_key, alpaca_secret, alpaca_mode, tradier_token, tradier_acct, tradier_sandbox) =
        row.unwrap_or((None, None, None, None, None, None));

    let mut positions: Vec<NormalizedPosition> = Vec::new();
    let mut errors: Vec<BrokerError> = Vec::new();
    let mut broker_count = 0usize;

    // Alpaca.
    if let (Some(k), Some(s)) = (alpaca_key.clone(), alpaca_secret.clone()) {
        if !k.is_empty() && !s.is_empty() {
            broker_count += 1;
            let mode = match alpaca_mode.as_deref() {
                Some("live") => alpaca_trading::BrokerMode::Live,
                _ => alpaca_trading::BrokerMode::Paper,
            };
            let client = alpaca_trading::AlpacaTrading::new(mode, k, s);
            match client.list_positions().await {
                Ok(rows) => {
                    for p in rows {
                        positions.push(from_alpaca(p));
                    }
                }
                Err(e) => errors.push(BrokerError {
                    broker: "alpaca".into(),
                    message: format!("{e}"),
                }),
            }
        }
    }
    // Tradier.
    if let (Some(t), Some(a)) = (tradier_token.clone(), tradier_acct.clone()) {
        if !t.is_empty() && !a.is_empty() {
            broker_count += 1;
            let env = if tradier_sandbox.unwrap_or(true) {
                tradier_trading::TradierEnv::Sandbox
            } else {
                tradier_trading::TradierEnv::Live
            };
            let client = tradier_trading::TradierTrading::new(env, t, a);
            match client.get_positions().await {
                Ok(resp) => {
                    // Tradier wraps the position list in an untagged
                    // enum: Empty / Single / Many. Flatten into a Vec.
                    let list: Vec<tradier_trading::Position> = match resp.positions {
                        None => Vec::new(),
                        Some(tradier_trading::PositionsList::Empty(_)) => Vec::new(),
                        Some(tradier_trading::PositionsList::Single { position }) => {
                            vec![position]
                        }
                        Some(tradier_trading::PositionsList::Many { position }) => position,
                    };
                    for p in list {
                        // Tradier doesn't ship current price in the
                        // positions payload — None for now; a future
                        // enhancement could batch-quote here.
                        positions.push(from_tradier(p, None));
                    }
                }
                Err(e) => errors.push(BrokerError {
                    broker: "tradier".into(),
                    message: format!("{e:?}"),
                }),
            }
        }
    }
    // IBKR / Schwab / Tastytrade — creds-loading + client-construction
    // patterns differ enough per broker that they live in follow-up
    // commits. The aggregator already supports their normalized shapes;
    // adding them is a wiring exercise, not a logic one.

    let totals = aggregate_by_symbol(&positions);
    Ok(MultiBrokerReport {
        positions,
        totals,
        broker_count,
        errors,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    fn dec(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    #[test]
    fn alpaca_long_normalises_positive_qty() {
        let p = alpaca_trading::PositionResponse {
            symbol: "AAPL".into(),
            qty: dec("100"),
            side: "long".into(),
            avg_entry_price: dec("150.50"),
            market_value: Some(dec("17500.00")),
            cost_basis: Some(dec("15050.00")),
            unrealized_pl: Some(dec("2450.00")),
            current_price: Some(dec("175.00")),
        };
        let n = from_alpaca(p);
        assert_eq!(n.symbol, "AAPL");
        assert_eq!(n.broker, "alpaca");
        assert_eq!(n.qty, 100.0);
        assert_eq!(n.avg_cost, Some(150.50));
        assert_eq!(n.market_value, Some(17500.00));
        assert_eq!(n.unrealized_pl, Some(2450.00));
    }

    #[test]
    fn alpaca_short_normalises_negative_qty() {
        let p = alpaca_trading::PositionResponse {
            symbol: "TSLA".into(),
            qty: dec("50"),
            side: "short".into(),
            avg_entry_price: dec("250.00"),
            market_value: Some(dec("-12000.00")),
            cost_basis: Some(dec("-12500.00")),
            unrealized_pl: Some(dec("500.00")),
            current_price: Some(dec("240.00")),
        };
        let n = from_alpaca(p);
        assert_eq!(n.qty, -50.0);
        assert_eq!(n.unrealized_pl, Some(500.00));
    }

    #[test]
    fn ibkr_carries_signed_qty_and_account_label() {
        let p = ibkr_trading::Position {
            account_id: Some("U1234567".into()),
            conid: Some(265598),
            contract_desc: Some("AAPL".into()),
            position: Some(-25.0),
            mkt_price: Some(175.0),
            avg_price: Some(180.0),
            unrealized_pnl: Some(125.0),
        };
        let n = from_ibkr(p);
        assert_eq!(n.symbol, "AAPL");
        assert_eq!(n.broker, "ibkr");
        assert_eq!(n.qty, -25.0);
        // mkt_value = -25 × 175 = -4375
        assert_eq!(n.market_value, Some(-4375.0));
        assert_eq!(n.account_label, Some("U1234567".into()));
    }

    #[test]
    fn tradier_derives_avg_cost_from_cost_basis() {
        let p = tradier_trading::Position {
            cost_basis: dec("15050.00"),
            date_acquired: Some("2026-04-01".into()),
            id: Some(1),
            quantity: dec("100"),
            symbol: "AAPL".into(),
        };
        let n = from_tradier(p, Some(175.00));
        assert_eq!(n.symbol, "AAPL");
        assert_eq!(n.qty, 100.0);
        assert_eq!(n.avg_cost, Some(150.50));
        assert_eq!(n.market_value, Some(17500.00));
        assert_eq!(n.unrealized_pl, Some(17500.00 - 15050.00));
    }

    #[test]
    fn tradier_without_price_skips_market_value() {
        let p = tradier_trading::Position {
            cost_basis: dec("1000.00"),
            date_acquired: None,
            id: None,
            quantity: dec("10"),
            symbol: "X".into(),
        };
        let n = from_tradier(p, None);
        assert!(n.market_value.is_none());
        assert!(n.unrealized_pl.is_none());
    }

    fn pos(
        sym: &str,
        broker: &str,
        qty: f64,
        mv: Option<f64>,
        pl: Option<f64>,
    ) -> NormalizedPosition {
        NormalizedPosition {
            symbol: sym.into(),
            broker: broker.into(),
            qty,
            avg_cost: Some(100.0),
            current_price: None,
            market_value: mv,
            unrealized_pl: pl,
            account_label: None,
        }
    }

    #[test]
    fn aggregate_groups_same_symbol_across_brokers() {
        let positions = vec![
            pos("AAPL", "alpaca", 100.0, Some(17500.0), Some(2450.0)),
            pos("AAPL", "tradier", 50.0, Some(8750.0), Some(1225.0)),
            pos("TSLA", "alpaca", -25.0, Some(-6250.0), Some(250.0)),
        ];
        let totals = aggregate_by_symbol(&positions);
        let aapl = totals.iter().find(|t| t.symbol == "AAPL").unwrap();
        assert_eq!(aapl.total_qty, 150.0);
        assert_eq!(aapl.total_market_value, 26250.0);
        assert_eq!(aapl.total_unrealized_pl, 3675.0);
        assert_eq!(aapl.broker_count, 2);
        assert!(aapl.brokers.contains(&"alpaca".to_string()));
        assert!(aapl.brokers.contains(&"tradier".to_string()));
        let tsla = totals.iter().find(|t| t.symbol == "TSLA").unwrap();
        assert_eq!(tsla.total_qty, -25.0);
        assert_eq!(tsla.broker_count, 1);
    }

    #[test]
    fn aggregate_sorts_by_abs_market_value_desc() {
        let positions = vec![
            pos("SMALL", "alpaca", 10.0, Some(100.0), Some(0.0)),
            pos("HUGE", "alpaca", 100.0, Some(50_000.0), Some(0.0)),
            pos("SHORT", "alpaca", -50.0, Some(-25_000.0), Some(0.0)),
        ];
        let totals = aggregate_by_symbol(&positions);
        assert_eq!(totals[0].symbol, "HUGE");
        assert_eq!(totals[1].symbol, "SHORT");
        assert_eq!(totals[2].symbol, "SMALL");
    }

    #[test]
    fn aggregate_treats_none_market_value_as_zero_for_sum() {
        // A broker without price data still shows up in the broker list
        // but contributes 0 to the total — the UI can flag that.
        let positions = vec![
            pos("MISS", "tradier", 10.0, None, None),
            pos("MISS", "alpaca", 5.0, Some(500.0), Some(50.0)),
        ];
        let totals = aggregate_by_symbol(&positions);
        let miss = &totals[0];
        assert_eq!(miss.total_qty, 15.0);
        assert_eq!(miss.total_market_value, 500.0);
        assert_eq!(miss.broker_count, 2);
    }

    #[test]
    fn aggregate_empty_input_returns_empty() {
        assert!(aggregate_by_symbol(&[]).is_empty());
    }

    #[test]
    fn aggregate_deduplicates_brokers_per_symbol() {
        // Two positions on the same broker for the same symbol shouldn't
        // be counted twice in broker_count.
        let positions = vec![
            pos("AAPL", "ibkr", 50.0, Some(8750.0), Some(0.0)),
            pos("AAPL", "ibkr", 50.0, Some(8750.0), Some(0.0)),
        ];
        let totals = aggregate_by_symbol(&positions);
        assert_eq!(totals[0].broker_count, 1);
        assert_eq!(totals[0].total_qty, 100.0);
    }
}
