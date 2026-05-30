//! Option-vs-equity multiplier defaulting in `mapping::parse_with`.
//!
//! Per `mapping.rs:179-187`, when a row has no `multiplier` column (or the
//! column is absent from the `ColumnMap`), the parser defaults to:
//!
//!   - `Decimal::from(100)` when `asset_class == AssetClass::Option`
//!   - `Decimal::ONE`        otherwise (Stock / Future / Forex / unknown)
//!
//! This is load-bearing: P&L computation in `traderview-core` multiplies
//! quantity by price by multiplier. If options silently defaulted to 1
//! instead of 100, every option P&L would be off by 100x. If equity
//! defaulted to 100, every stock P&L would be off by 100x.
//!
//! Existing coverage: one assertion at `brokers.rs:668` for an IBKR options
//! row that *carries* an explicit `Multiplier=100` column. The default
//! branches (column absent OR row value blank) have not been pinned.
//! These tests target the default branches directly via the Generic CSV
//! parser (which has no `multiplier` ColSpec, so the default always fires).

use rust_decimal::Decimal;
use traderview_core::{AssetClass, OptionType};
use traderview_import::brokers::{GenericCsvParser, IbkrFlexParser};
use traderview_import::{parser_for, Parser};

// ─── Equity default: 1 ───────────────────────────────────────────────

#[test]
fn equity_row_without_multiplier_column_defaults_to_one() {
    // GenericCsvParser has no `asset_class`, `option_type`, `strike`,
    // `expiration`, `multiplier` columns in its ColumnMap. Asset class
    // therefore defaults to Stock and multiplier defaults to ONE.
    let csv = "Symbol,Side,Qty,Price,Fee,Date\n\
               AAPL,buy,100,150.00,1.00,2026-01-15 09:30:00\n";
    let out = GenericCsvParser.parse(csv.as_bytes()).unwrap();
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].asset_class, AssetClass::Stock);
    assert_eq!(out[0].multiplier, Decimal::ONE);
}

#[test]
fn equity_row_multiplier_is_exactly_decimal_one_not_one_point_zero_zero() {
    // Decimal::ONE has scale 0. Catching a regression that builds
    // `Decimal::from_str("1.00")` instead — they'd compare equal in
    // arithmetic but serialize differently and propagate scale.
    let csv = "Symbol,Side,Qty,Price,Fee,Date\n\
               MSFT,buy,10,420.00,0.50,2026-02-01 10:00:00\n";
    let out = GenericCsvParser.parse(csv.as_bytes()).unwrap();
    assert_eq!(out[0].multiplier.to_string(), "1");
}

// ─── Option default: 100 ─────────────────────────────────────────────

/// Drive the Option-asset branch via IBKR Flex — its ColumnMap has
/// `asset_class` and `option_type` columns. Omit `Multiplier` from the
/// CSV header entirely → the parser falls through to the default
/// `match asset_class { Option => 100, _ => 1 }`.
#[test]
fn option_row_without_multiplier_column_defaults_to_100() {
    let csv = "Symbol,Buy/Sell,Quantity,TradePrice,IBCommission,DateTime,OrderID,\
               AssetCategory,Put/Call,Strike,Expiry\n\
               SPY,BUY,1,3.40,-1.00,2026-03-04 10:02:15,IB-OPT,\
               OPT,C,520,2026-03-20\n";
    let out = IbkrFlexParser.parse(csv.as_bytes()).unwrap();
    assert_eq!(out.len(), 1);
    let r = &out[0];
    assert_eq!(r.asset_class, AssetClass::Option);
    assert_eq!(r.option_type, Some(OptionType::Call));
    assert_eq!(
        r.multiplier,
        Decimal::from(100),
        "option must default to 100x — the SPC-equity-options industry contract"
    );
    assert_eq!(r.multiplier.to_string(), "100");
}

/// Same as above for a put. Defaulting is asset-class-driven, not
/// call/put-driven — but pin both legs so a future regression that keys
/// off OptionType instead of AssetClass is caught.
#[test]
fn option_put_row_without_multiplier_also_defaults_to_100() {
    let csv = "Symbol,Buy/Sell,Quantity,TradePrice,IBCommission,DateTime,OrderID,\
               AssetCategory,Put/Call,Strike,Expiry\n\
               SPY,SELL,2,1.20,-1.00,2026-03-04 11:00:00,IB-PUT,\
               OPT,P,500,2026-03-20\n";
    let out = IbkrFlexParser.parse(csv.as_bytes()).unwrap();
    assert_eq!(out.len(), 1);
    let r = &out[0];
    assert_eq!(r.option_type, Some(OptionType::Put));
    assert_eq!(r.multiplier, Decimal::from(100));
}

/// IBKR row with explicit `Multiplier=50` (mini-option / non-standard
/// contract). Explicit value overrides the default. Pin so a parser
/// regression that hardcodes 100 for any AssetClass::Option is caught.
#[test]
fn explicit_multiplier_50_overrides_option_default_100() {
    let csv = "Symbol,Buy/Sell,Quantity,TradePrice,IBCommission,DateTime,OrderID,\
               AssetCategory,Put/Call,Strike,Expiry,Multiplier\n\
               SPY,BUY,1,3.40,-1.00,2026-03-04 10:02:15,IB-MINI,\
               OPT,C,520,2026-03-20,50\n";
    let out = IbkrFlexParser.parse(csv.as_bytes()).unwrap();
    assert_eq!(out[0].multiplier, Decimal::from(50));
}

/// Negative test: futures don't get the 100x default. With no multiplier
/// column and a non-Option asset class, the default branch yields 1.
/// (IBKR Flex sets AssetCategory=FUT for futures.)
#[test]
fn future_row_without_multiplier_defaults_to_one_not_100() {
    let csv = "Symbol,Buy/Sell,Quantity,TradePrice,IBCommission,DateTime,OrderID,\
               AssetCategory,Put/Call,Strike,Expiry\n\
               ES,BUY,1,4500.00,-2.50,2026-03-04 09:30:00,IB-FUT,\
               FUT,,,,\n";
    let out = IbkrFlexParser.parse(csv.as_bytes()).unwrap();
    assert_eq!(out.len(), 1);
    let r = &out[0];
    assert_eq!(r.asset_class, AssetClass::Future);
    assert_eq!(
        r.multiplier,
        Decimal::ONE,
        "future must default to 1 — futures multipliers vary per contract \
         (ES=50, NQ=20) and must come from the broker row, not be assumed"
    );
}

/// `parser_for("generic")` (the dispatch entrypoint) preserves the
/// default. Pin so a future re-route that wraps generic in an
/// option-detecting layer doesn't silently bump equity rows to 100.
#[test]
fn parser_for_generic_preserves_equity_multiplier_default() {
    let p = parser_for("generic").expect("generic parser dispatchable");
    let csv = "Symbol,Side,Qty,Price,Fee,Date\n\
               TSLA,sell,5,200.00,0.50,2026-04-01 12:00:00\n";
    let out = p.parse(csv.as_bytes()).unwrap();
    assert_eq!(out[0].multiplier, Decimal::ONE);
    assert_eq!(out[0].asset_class, AssetClass::Stock);
}
