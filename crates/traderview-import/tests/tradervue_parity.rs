//! Tradervue parity — pin our FIFO rollup against Tradervue's exported
//! trade list. The fixtures are real Webull `Account Statement → Orders`
//! CSVs plus matching Tradervue `Trades → Download CSV` exports.
//!
//! These tests are the regression net for "user imports CSV X into both
//! apps and gets different numbers." If you change the rollup algorithm,
//! the trade count, per-trade gross P&L, or entry/exit prices, these will
//! flag the divergence.
//!
//! Each fixture is a single symbol whose Webull execs + Tradervue trades
//! were extracted by hand from the user's real export, then frozen here.

use rust_decimal::Decimal;
use std::collections::HashMap;
use std::str::FromStr;
use traderview_core::rollup::{rollup_with, CloseModel, LotMethod};
use traderview_core::{TradeSide, TradeStatus};
use traderview_import::{Parser, brokers::WebullParser};

fn d(s: &str) -> Decimal {
    Decimal::from_str(s).expect("decimal")
}

fn roll(csv_bytes: &[u8]) -> Vec<traderview_core::Trade> {
    let parser = WebullParser;
    let parsed = parser.parse(csv_bytes).expect("parse webull csv");

    // ParsedExecution → Execution. Account/import ids are unused by rollup
    // grouping (we group by symbol within one account anyway).
    let acct = uuid::Uuid::new_v4();
    let execs: Vec<traderview_core::Execution> = parsed
        .into_iter()
        .map(|p| traderview_core::Execution {
            id: uuid::Uuid::new_v4(),
            account_id: acct,
            symbol: p.symbol,
            side: p.side,
            qty: p.qty,
            price: p.price,
            fee: p.fee,
            commission: p.commission,
            executed_at: p.executed_at,
            broker_order_id: p.broker_order_id,
            raw: p.raw,
            import_id: None,
            asset_class: p.asset_class,
            option_type: p.option_type,
            strike: p.strike,
            expiration: p.expiration,
            multiplier: p.multiplier,
            tick_size: p.tick_size,
            tick_value: p.tick_value,
            base_ccy: p.base_ccy,
            quote_ccy: p.quote_ccy,
            pip_size: p.pip_size,
        })
        .collect();

    let rolled =
        rollup_with(&execs, LotMethod::Fifo, CloseModel::PerCloseExec).expect("rollup");
    rolled.into_iter().map(|r| r.trade).collect()
}

// Tolerance used when comparing Decimal-derived gross P&L to Tradervue's
// 4-decimal rounded export. 1 cent is generous; most cases match exact.
fn approx(a: Decimal, b: Decimal) -> bool {
    (a - b).abs() <= d("0.01")
}

// ─────────────────────────────────────────────────────────────────────
// FIXTURE: TWNP — 4 round-trip trades, no carry-over, all small longs.
// 9 Webull execs → 4 Tradervue trades. Each Sell exec brings the
// position back to flat. Both `FlatOnly` and `PerCloseExec` models
// should agree with Tradervue on count + per-trade P&L here.
// ─────────────────────────────────────────────────────────────────────

const TWNP_WEBULL_CSV: &[u8] = br#"Name,Symbol,Side,Status,Filled,Total Qty,Price,Avg Price,Time-in-Force,Placed Time,Filled Time
Twin Hospitality,TWNP,Sell,Filled,2468,2468,@0.7000000000,0.7000000000,DAY,01/30/2026 09:05:11 EST,01/30/2026 09:05:11 EST
Twin Hospitality,TWNP,Buy,Filled,1249,1249,@0.7001000000,0.7001000000,DAY,01/30/2026 09:04:26 EST,01/30/2026 09:04:26 EST
Twin Hospitality,TWNP,Buy,Filled,1219,1219,@0.7128000000,0.7128000000,DAY,01/30/2026 09:03:51 EST,01/30/2026 09:03:51 EST
Twin Hospitality,TWNP,Sell,Filled,1280,1280,@0.6853000000,0.6853000000,DAY,01/30/2026 09:03:44 EST,01/30/2026 09:03:45 EST
Twin Hospitality,TWNP,Buy,Filled,1280,1280,@0.6787000000,0.6787000000,DAY,01/30/2026 09:03:39 EST,01/30/2026 09:03:39 EST
Twin Hospitality,TWNP,Sell,Filled,1282,1282,@0.6801000000,0.6801000000,DAY,01/30/2026 09:03:31 EST,01/30/2026 09:03:31 EST
Twin Hospitality,TWNP,Buy,Filled,1282,1282,@0.6720000000,0.6720000000,DAY,01/30/2026 09:03:19 EST,01/30/2026 09:03:19 EST
Twin Hospitality,TWNP,Sell,Filled,1342,1342,@0.6446000000,0.6446000000,DAY,01/30/2026 09:02:49 EST,01/30/2026 09:02:50 EST
Twin Hospitality,TWNP,Buy,Filled,1342,1342,@0.6446000000,0.6446000000,DAY,01/30/2026 09:02:35 EST,01/30/2026 09:02:35 EST
"#;

#[test]
fn twnp_matches_tradervue_trade_count_and_pnls() {
    let trades = roll(TWNP_WEBULL_CSV);
    let closed: Vec<_> = trades
        .iter()
        .filter(|t| t.status == TradeStatus::Closed)
        .collect();

    // Tradervue exported 4 trades for TWNP.
    assert_eq!(closed.len(), 4, "trade count must match Tradervue");

    // All longs.
    for t in &closed {
        assert_eq!(t.side, TradeSide::Long);
        assert_eq!(t.symbol, "TWNP");
    }

    // Sum of gross P&Ls must match Tradervue's total (within 1¢).
    // Tradervue: -15.7281 + 8.4480 + 10.3842 + 0.0000 = 3.1041
    let total: Decimal = closed
        .iter()
        .filter_map(|t| t.gross_pnl)
        .sum();
    assert!(
        approx(total, d("3.1041")),
        "TWNP total gross P&L must match Tradervue's 3.1041; got {}",
        total
    );

    // Per-trade gross P&L, keyed by qty (each trade has a distinct qty
    // in this fixture so the join is unambiguous).
    let mut by_qty: HashMap<Decimal, Decimal> = HashMap::new();
    for t in &closed {
        by_qty.insert(t.qty, t.gross_pnl.unwrap_or(Decimal::ZERO));
    }

    // Tradervue trade 1: qty 1342, P&L 0.00 (entry 0.6446, exit 0.6446)
    assert!(approx(by_qty[&d("1342")], d("0.0000")));
    // Tradervue trade 2: qty 1282, P&L 10.3842
    assert!(approx(by_qty[&d("1282")], d("10.3842")));
    // Tradervue trade 3: qty 1280, P&L 8.4480
    assert!(approx(by_qty[&d("1280")], d("8.4480")));
    // Tradervue trade 4: qty 2468 (1219 + 1249), P&L -15.7281
    assert!(approx(by_qty[&d("2468")], d("-15.7281")));
}

#[test]
fn twnp_entry_avg_matches_tradervue_per_trade() {
    let trades = roll(TWNP_WEBULL_CSV);
    let closed: Vec<_> = trades
        .iter()
        .filter(|t| t.status == TradeStatus::Closed)
        .collect();

    let entry: HashMap<Decimal, Decimal> =
        closed.iter().map(|t| (t.qty, t.entry_avg)).collect();

    // Tradervue trade 1: entry 0.6446 (single buy @0.6446)
    assert!((entry[&d("1342")] - d("0.6446")).abs() < d("0.0001"));
    // Tradervue trade 2: entry 0.6720
    assert!((entry[&d("1282")] - d("0.6720")).abs() < d("0.0001"));
    // Tradervue trade 3: entry 0.6787
    assert!((entry[&d("1280")] - d("0.6787")).abs() < d("0.0001"));
    // Tradervue trade 4: entry 0.7128 (first of 2 FIFO buys: 1219 @0.7128 + 1249 @0.7001)
    // Tradervue displays the FIRST opening exec's price, not the weighted
    // average. Our implementation reports the weighted avg of the closed
    // portion via notional_in_closed / qty_closed: for a full close, this
    // equals notional_in / qty_total = (1219*0.7128 + 1249*0.7001) / 2468
    // = 0.706379... — different from Tradervue's display value (0.7128).
    // The Gross P&L still matches because P&L is per-share matched. This
    // test pins our behavior (weighted avg) and documents the divergence.
    let expected_weighted = (d("1219") * d("0.7128") + d("1249") * d("0.7001"))
        / d("2468");
    assert!(
        (entry[&d("2468")] - expected_weighted).abs() < d("0.0001"),
        "weighted entry_avg for 2-buy trade; got {} expected {}",
        entry[&d("2468")],
        expected_weighted,
    );
}

// ─────────────────────────────────────────────────────────────────────
// FIXTURE: PDC — 6 trades across 3 sessions; last trade has a 200-share
// carry-over. Exercises the PerCloseExec model's partial-close finalize
// + from_inventory path that produces an Open trade for the leftover.
// ─────────────────────────────────────────────────────────────────────

const PDC_WEBULL_CSV: &[u8] = br#"Name,Symbol,Side,Status,Filled,Total Qty,Price,Avg Price,Time-in-Force,Placed Time,Filled Time
Perpetuals.com Ltd,PDC,Sell,Filled,2473,2473,@8.6539425799,8.6539425799,DAY,01/27/2026 16:07:27 EST,01/27/2026 16:07:28 EST
Perpetuals.com Ltd,PDC,Buy,Filled,2673,2673,@9.3496258885,9.3496258885,DAY,01/27/2026 16:05:36 EST,01/27/2026 16:05:38 EST
Perpetuals.com Ltd,PDC,Sell,Filled,1156,1156,@8.8900000000,8.8900000000,DAY,01/27/2026 16:05:18 EST,01/27/2026 16:05:21 EST
Perpetuals.com Ltd,PDC,Buy,Filled,1156,1156,@8.5498529412,8.5498529412,DAY,01/27/2026 16:04:41 EST,01/27/2026 16:04:41 EST
Perpetuals.com Ltd,PDC,Sell,Filled,1197,1197,@8.3600000000,8.3600000000,DAY,01/27/2026 16:04:29 EST,01/27/2026 16:04:29 EST
Perpetuals.com Ltd,PDC,Buy,Filled,1197,1197,@8.1582121971,8.1582121971,DAY,01/27/2026 16:03:34 EST,01/27/2026 16:03:35 EST
Perpetuals.com Ltd,PDC,Sell,Filled,1234,1234,@7.8000000000,7.8000000000,DAY,01/27/2026 16:01:55 EST,01/27/2026 16:01:55 EST
Perpetuals.com Ltd,PDC,Buy,Filled,1234,1234,@7.2500000000,7.2500000000,DAY,01/27/2026 16:00:44 EST,01/27/2026 16:00:44 EST
Perpetuals.com Ltd,PDC,Sell,Filled,1145,1145,@8.5080480348,8.5080480348,DAY,01/23/2026 17:37:21 EST,01/23/2026 17:37:21 EST
Perpetuals.com Ltd,PDC,Buy,Filled,1145,1145,@8.7282620087,8.7282620087,DAY,01/23/2026 17:35:17 EST,01/23/2026 17:35:17 EST
Perpetuals.com Ltd,PDC,Sell,Filled,138,138,@6.9586956522,6.9586956522,DAY,01/15/2026 16:46:31 EST,01/15/2026 16:46:31 EST
Perpetuals.com Ltd,PDC,Buy,Filled,138,138,@7.2000000000,7.2000000000,DAY,01/15/2026 16:43:51 EST,01/15/2026 16:43:51 EST
"#;

#[test]
fn pdc_matches_tradervue_trade_count_and_carry_over_open() {
    let trades = roll(PDC_WEBULL_CSV);
    let closed: Vec<_> = trades
        .iter()
        .filter(|t| t.status == TradeStatus::Closed)
        .collect();
    let open: Vec<_> = trades
        .iter()
        .filter(|t| t.status == TradeStatus::Open)
        .collect();
    // Tradervue reports 6 closed trades. The last sell (2473) closes
    // less than the prior open (2673), leaving 200 shares — our model
    // surfaces those as an Open trade. Tradervue silently absorbs the
    // remaining inventory into the trade's Volume column.
    assert_eq!(closed.len(), 6);
    assert_eq!(open.len(), 1);
    assert_eq!(open[0].qty, d("200"));
}

#[test]
fn pdc_round_trip_gross_pnls_match_tradervue_exact() {
    let trades = roll(PDC_WEBULL_CSV);
    let by_qty: HashMap<Decimal, Decimal> = trades
        .iter()
        .filter(|t| t.status == TradeStatus::Closed)
        .map(|t| (t.qty, t.gross_pnl.unwrap_or(Decimal::ZERO)))
        .collect();
    // 5 clean round trips (qty distinct from each other). Each must match
    // Tradervue's exported gross P&L within 1¢.
    assert!(approx(by_qty[&d("138")],  d("-33.30")));
    assert!(approx(by_qty[&d("1145")], d("-252.145")));
    assert!(approx(by_qty[&d("1234")], d("678.70")));
    assert!(approx(by_qty[&d("1197")], d("241.54")));
    assert!(approx(by_qty[&d("1156")], d("393.21")));
}

#[test]
fn pdc_total_gross_pnl_within_tolerance_of_tradervue() {
    let trades = roll(PDC_WEBULL_CSV);
    let our_total: Decimal = trades
        .iter()
        .filter(|t| t.status == TradeStatus::Closed)
        .filter_map(|t| t.gross_pnl)
        .sum();
    // Tradervue PDC sum: -33.30 + -252.145 + 678.70 + 241.54 + 393.21 +
    // (-1714.35) = -686.345. Our last trade's P&L computes against the
    // actual sell qty (2473) and differs by ~$5 from Tradervue's display
    // value — within tolerance.
    let tv_total = d("-686.345");
    assert!(
        (our_total - tv_total).abs() < d("20"),
        "PDC sum gross P&L must be within $20 of Tradervue's; ours={}, tv={}",
        our_total, tv_total,
    );
}

#[test]
fn twnp_total_volume_matches_tradervue() {
    let trades = roll(TWNP_WEBULL_CSV);
    let closed: Vec<_> = trades
        .iter()
        .filter(|t| t.status == TradeStatus::Closed)
        .collect();

    // Tradervue's `Volume` column = entry qty + exit qty (per trade).
    // Across 4 trades: 2684 + 2564 + 2560 + 4936 = 12744.
    // Our model reports `qty` = close qty per trade. Sum = 1342 + 1282
    // + 1280 + 2468 = 6372. Tradervue's volume is exactly 2× this for
    // fully balanced trades.
    let sum_qty: Decimal = closed.iter().map(|t| t.qty).sum();
    assert_eq!(sum_qty, d("6372"));
    let tradervue_volume = sum_qty * d("2");
    assert_eq!(tradervue_volume, d("12744"));
}
