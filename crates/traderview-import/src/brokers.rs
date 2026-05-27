//! Per-broker presets built on top of [`crate::mapping`].
//!
//! Column maps target the canonical "trade history" or "account statement"
//! CSV export from each broker. Many brokers ship multiple export shapes;
//! the maps below cover the most common defaults. Aliases on each header
//! cover known historical column-name drift.
//!
//! Where a real-world sample isn't on hand, the map is built from each
//! broker's published documentation / community-known schema, and the
//! aliases are intentionally generous. Users with a non-matching export
//! can fall back to `generic` and hand-roll a [`crate::mapping::ColumnMap`].

use crate::mapping::{parse_with, ColSpec, ColumnMap, SideLookup};
use crate::{ImportError, ParsedExecution, Parser};

// ===========================================================================
// Webull — Account Statement → Orders CSV
// Common headers: Name, Symbol, Side, Status, Filled, TotalQty, Price,
// AvgPrice, TimeInForce, Placed Time, Filled Time, Commission, ...
// ===========================================================================
pub struct WebullParser;
impl Parser for WebullParser {
    fn source(&self) -> &'static str {
        "webull"
    }
    fn parse(&self, bytes: &[u8]) -> Result<Vec<ParsedExecution>, ImportError> {
        parse_with(bytes, &MAP_WEBULL)
    }
}

const MAP_WEBULL: ColumnMap = ColumnMap {
    source: "webull",
    has_header: true,
    delimiter: b',',
    date_formats: &["%m/%d/%Y %H:%M:%S", "%Y-%m-%d %H:%M:%S", "%m/%d/%Y %H:%M"],
    utc_assumed: false, // Webull exports are usually local; user can override later via tz
    side_lookup: SideLookup::DEFAULT,
    symbol: ColSpec::HeaderAny(&["symbol", "ticker"]),
    side: ColSpec::HeaderAny(&["side", "action", "buy/sell", "order side"]),
    // "total qty" / "quantity" / "qty" describe the ORDER size; "filled"
    // is the cumulative filled-so-far and is wrong on partial-fill rows.
    // Prefer the unambiguous order-size columns first; fall back to "filled"
    // only when the broker's export omits them.
    qty: ColSpec::HeaderAny(&["total qty", "quantity", "qty", "filled qty", "filled"]),
    price: ColSpec::HeaderAny(&["avg price", "price", "filled price", "avgprice"]),
    fee: Some(ColSpec::HeaderAny(&[
        "commission",
        "fees",
        "commission & fees",
    ])),
    executed_at: ColSpec::HeaderAny(&["filled time", "executed", "executed time", "time"]),
    broker_order_id: Some(ColSpec::HeaderAny(&["order id", "order number", "id"])),
    asset_class: None,
    option_type: None,
    strike: None,
    expiration: None,
    multiplier: None,
    skip_symbols: &["total", "subtotal", ""],
};

// ===========================================================================
// IBKR (Flex Query — Trades section). One row per execution.
// Headers vary by Flex template; aliases here cover defaults.
// ===========================================================================
pub struct IbkrFlexParser;
impl Parser for IbkrFlexParser {
    fn source(&self) -> &'static str {
        "ibkr"
    }
    fn parse(&self, bytes: &[u8]) -> Result<Vec<ParsedExecution>, ImportError> {
        parse_with(bytes, &MAP_IBKR)
    }
}

const MAP_IBKR: ColumnMap = ColumnMap {
    source: "ibkr",
    has_header: true,
    delimiter: b',',
    date_formats: &[
        "%Y-%m-%d, %H:%M:%S",
        "%Y%m%d;%H%M%S",
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%d %H:%M:%S",
    ],
    utc_assumed: true,
    side_lookup: SideLookup {
        buy: &["buy", "bot"],
        sell: &["sell", "sld"],
        short: &["sellshort", "short"],
        cover: &["buytocover", "cover"],
    },
    symbol: ColSpec::HeaderAny(&["symbol", "underlyingsymbol"]),
    side: ColSpec::HeaderAny(&["buy/sell", "side"]),
    qty: ColSpec::HeaderAny(&["quantity", "qty"]),
    price: ColSpec::HeaderAny(&["tradeprice", "price"]),
    fee: Some(ColSpec::HeaderAny(&["ibcommission", "commission"])),
    executed_at: ColSpec::HeaderAny(&["datetime", "tradedate", "trade date/time"]),
    broker_order_id: Some(ColSpec::HeaderAny(&["orderid", "ibexecid", "tradeid"])),
    asset_class: Some(ColSpec::HeaderAny(&["assetcategory", "asset category"])),
    option_type: Some(ColSpec::HeaderAny(&["put/call"])),
    strike: Some(ColSpec::HeaderAny(&["strike"])),
    expiration: Some(ColSpec::HeaderAny(&["expiry", "expiration"])),
    multiplier: Some(ColSpec::HeaderAny(&["multiplier"])),
    skip_symbols: &["total", "subtotal", ""],
};

// ===========================================================================
// TD Ameritrade — "History → Export" CSV.
// ===========================================================================
pub struct TdAmeritradeParser;
impl Parser for TdAmeritradeParser {
    fn source(&self) -> &'static str {
        "tdameritrade"
    }
    fn parse(&self, bytes: &[u8]) -> Result<Vec<ParsedExecution>, ImportError> {
        parse_with(bytes, &MAP_TD)
    }
}

const MAP_TD: ColumnMap = ColumnMap {
    source: "tdameritrade",
    has_header: true,
    delimiter: b',',
    date_formats: &["%m/%d/%Y %H:%M:%S", "%m/%d/%Y", "%Y-%m-%d %H:%M:%S"],
    utc_assumed: false,
    side_lookup: SideLookup::DEFAULT,
    symbol: ColSpec::HeaderAny(&["symbol"]),
    side: ColSpec::HeaderAny(&["side", "transaction type", "type"]),
    qty: ColSpec::HeaderAny(&["quantity", "qty"]),
    price: ColSpec::HeaderAny(&["price"]),
    fee: Some(ColSpec::HeaderAny(&["commission", "reg fee", "fees"])),
    executed_at: ColSpec::HeaderAny(&["date", "exec time", "transaction date"]),
    broker_order_id: Some(ColSpec::HeaderAny(&["order id"])),
    asset_class: None,
    option_type: None,
    strike: None,
    expiration: None,
    multiplier: None,
    skip_symbols: &["", "***", "total"],
};

// ===========================================================================
// Charles Schwab — "All Transactions" CSV (post-TD merger).
// ===========================================================================
pub struct SchwabParser;
impl Parser for SchwabParser {
    fn source(&self) -> &'static str {
        "schwab"
    }
    fn parse(&self, bytes: &[u8]) -> Result<Vec<ParsedExecution>, ImportError> {
        parse_with(bytes, &MAP_SCHWAB)
    }
}

const MAP_SCHWAB: ColumnMap = ColumnMap {
    source: "schwab",
    has_header: true,
    delimiter: b',',
    date_formats: &["%m/%d/%Y", "%m/%d/%Y %H:%M:%S"],
    utc_assumed: false,
    side_lookup: SideLookup::DEFAULT,
    symbol: ColSpec::HeaderAny(&["symbol"]),
    side: ColSpec::HeaderAny(&["action", "side"]),
    qty: ColSpec::HeaderAny(&["quantity"]),
    price: ColSpec::HeaderAny(&["price"]),
    fee: Some(ColSpec::HeaderAny(&["fees & comm", "commission"])),
    executed_at: ColSpec::HeaderAny(&["date"]),
    broker_order_id: None,
    asset_class: None,
    option_type: None,
    strike: None,
    expiration: None,
    multiplier: None,
    skip_symbols: &["", "total"],
};

// ===========================================================================
// TradeStation — "Trade History" CSV.
// ===========================================================================
pub struct TradeStationParser;
impl Parser for TradeStationParser {
    fn source(&self) -> &'static str {
        "tradestation"
    }
    fn parse(&self, bytes: &[u8]) -> Result<Vec<ParsedExecution>, ImportError> {
        parse_with(bytes, &MAP_TS)
    }
}

const MAP_TS: ColumnMap = ColumnMap {
    source: "tradestation",
    has_header: true,
    delimiter: b',',
    date_formats: &["%m/%d/%Y %H:%M:%S", "%m/%d/%Y"],
    utc_assumed: false,
    side_lookup: SideLookup::DEFAULT,
    symbol: ColSpec::HeaderAny(&["symbol"]),
    side: ColSpec::HeaderAny(&["side", "buy/sell"]),
    qty: ColSpec::HeaderAny(&["quantity", "filled qty"]),
    price: ColSpec::HeaderAny(&["price", "avg price"]),
    fee: Some(ColSpec::HeaderAny(&["commission", "fees"])),
    executed_at: ColSpec::HeaderAny(&["date/time", "executed", "trade date"]),
    broker_order_id: Some(ColSpec::HeaderAny(&["order id"])),
    asset_class: None,
    option_type: None,
    strike: None,
    expiration: None,
    multiplier: None,
    skip_symbols: &["", "total"],
};

// ===========================================================================
// Lightspeed — execution CSV.
// ===========================================================================
pub struct LightspeedParser;
impl Parser for LightspeedParser {
    fn source(&self) -> &'static str {
        "lightspeed"
    }
    fn parse(&self, bytes: &[u8]) -> Result<Vec<ParsedExecution>, ImportError> {
        parse_with(bytes, &MAP_LIGHTSPEED)
    }
}

const MAP_LIGHTSPEED: ColumnMap = ColumnMap {
    source: "lightspeed",
    has_header: true,
    delimiter: b',',
    date_formats: &["%Y-%m-%d %H:%M:%S", "%m/%d/%Y %H:%M:%S"],
    utc_assumed: true,
    side_lookup: SideLookup::DEFAULT,
    symbol: ColSpec::HeaderAny(&["symbol"]),
    side: ColSpec::HeaderAny(&["side", "buy/sell"]),
    qty: ColSpec::HeaderAny(&["shares", "qty", "quantity"]),
    price: ColSpec::HeaderAny(&["price", "exec price"]),
    fee: Some(ColSpec::HeaderAny(&["commission", "fees"])),
    executed_at: ColSpec::HeaderAny(&["exec time", "time", "datetime"]),
    broker_order_id: Some(ColSpec::HeaderAny(&["order ref", "order id"])),
    asset_class: None,
    option_type: None,
    strike: None,
    expiration: None,
    multiplier: None,
    skip_symbols: &["", "total"],
};

// ===========================================================================
// DAS Trader Pro — execution log CSV.
// ===========================================================================
pub struct DasParser;
impl Parser for DasParser {
    fn source(&self) -> &'static str {
        "das"
    }
    fn parse(&self, bytes: &[u8]) -> Result<Vec<ParsedExecution>, ImportError> {
        parse_with(bytes, &MAP_DAS)
    }
}

const MAP_DAS: ColumnMap = ColumnMap {
    source: "das",
    has_header: true,
    delimiter: b',',
    date_formats: &["%m/%d/%Y %H:%M:%S", "%Y-%m-%d %H:%M:%S"],
    utc_assumed: false,
    side_lookup: SideLookup {
        buy: &["b", "buy"],
        sell: &["s", "sell"],
        short: &["ss", "short"],
        cover: &["bc", "cover", "buy to cover"],
    },
    symbol: ColSpec::HeaderAny(&["symbol", "symb"]),
    side: ColSpec::HeaderAny(&["side", "b/s"]),
    qty: ColSpec::HeaderAny(&["shares", "qty"]),
    price: ColSpec::HeaderAny(&["price", "trade price"]),
    fee: Some(ColSpec::HeaderAny(&["fee", "commission"])),
    executed_at: ColSpec::HeaderAny(&["time", "datetime"]),
    broker_order_id: Some(ColSpec::HeaderAny(&["orderno", "order id"])),
    asset_class: None,
    option_type: None,
    strike: None,
    expiration: None,
    multiplier: None,
    skip_symbols: &["", "total"],
};

// ===========================================================================
// ThinkOrSwim — Account Statement CSV (trades section).
// ===========================================================================
pub struct ThinkOrSwimParser;
impl Parser for ThinkOrSwimParser {
    fn source(&self) -> &'static str {
        "tos"
    }
    fn parse(&self, bytes: &[u8]) -> Result<Vec<ParsedExecution>, ImportError> {
        parse_with(bytes, &MAP_TOS)
    }
}

const MAP_TOS: ColumnMap = ColumnMap {
    source: "tos",
    has_header: true,
    delimiter: b',',
    date_formats: &["%m/%d/%y %H:%M:%S", "%m/%d/%Y %H:%M:%S"],
    utc_assumed: false,
    side_lookup: SideLookup {
        buy: &["bot", "buy", "+"],
        sell: &["sold", "sell", "-"],
        short: &["short"],
        cover: &["cover"],
    },
    symbol: ColSpec::HeaderAny(&["symbol", "underlying symbol"]),
    // "side" only — "type" was previously listed as an alias here AND as the
    // asset_class column below, so a TOS CSV with a literal "Type" column
    // would resolve `side` to the asset-class string and break decoding.
    side: ColSpec::HeaderAny(&["side"]),
    qty: ColSpec::HeaderAny(&["qty", "quantity"]),
    price: ColSpec::HeaderAny(&["price"]),
    fee: Some(ColSpec::HeaderAny(&["commissions & fees", "commission"])),
    executed_at: ColSpec::HeaderAny(&["exec time", "date/time"]),
    broker_order_id: Some(ColSpec::HeaderAny(&["order id"])),
    asset_class: Some(ColSpec::HeaderAny(&["asset type", "type"])),
    option_type: Some(ColSpec::HeaderAny(&["put/call"])),
    strike: Some(ColSpec::HeaderAny(&["strike"])),
    expiration: Some(ColSpec::HeaderAny(&["exp"])),
    multiplier: None,
    skip_symbols: &["", "total", "***"],
};

// ===========================================================================
// E*TRADE — Transactions CSV.
// ===========================================================================
pub struct ETradeParser;
impl Parser for ETradeParser {
    fn source(&self) -> &'static str {
        "etrade"
    }
    fn parse(&self, bytes: &[u8]) -> Result<Vec<ParsedExecution>, ImportError> {
        parse_with(bytes, &MAP_ETRADE)
    }
}

const MAP_ETRADE: ColumnMap = ColumnMap {
    source: "etrade",
    has_header: true,
    delimiter: b',',
    date_formats: &["%m/%d/%Y", "%m/%d/%Y %H:%M:%S"],
    utc_assumed: false,
    side_lookup: SideLookup::DEFAULT,
    symbol: ColSpec::HeaderAny(&["symbol"]),
    side: ColSpec::HeaderAny(&["transaction type", "side"]),
    qty: ColSpec::HeaderAny(&["quantity"]),
    price: ColSpec::HeaderAny(&["price"]),
    fee: Some(ColSpec::HeaderAny(&["commission", "fees"])),
    executed_at: ColSpec::HeaderAny(&["transaction date", "date"]),
    broker_order_id: None,
    asset_class: None,
    option_type: None,
    strike: None,
    expiration: None,
    multiplier: None,
    skip_symbols: &["", "total"],
};

// ===========================================================================
// Fidelity — Activity CSV.
// ===========================================================================
pub struct FidelityParser;
impl Parser for FidelityParser {
    fn source(&self) -> &'static str {
        "fidelity"
    }
    fn parse(&self, bytes: &[u8]) -> Result<Vec<ParsedExecution>, ImportError> {
        parse_with(bytes, &MAP_FIDELITY)
    }
}

const MAP_FIDELITY: ColumnMap = ColumnMap {
    source: "fidelity",
    has_header: true,
    delimiter: b',',
    date_formats: &["%m/%d/%Y", "%m/%d/%Y %H:%M:%S"],
    utc_assumed: false,
    side_lookup: SideLookup {
        buy: &["you bought", "buy", "bought"],
        sell: &["you sold", "sell", "sold"],
        short: &["short sale", "short"],
        cover: &["cover", "buy to cover"],
    },
    symbol: ColSpec::HeaderAny(&["symbol"]),
    side: ColSpec::HeaderAny(&["action", "side"]),
    qty: ColSpec::HeaderAny(&["quantity"]),
    price: ColSpec::HeaderAny(&["price", "price per share"]),
    fee: Some(ColSpec::HeaderAny(&["commission", "fees"])),
    // "trade date" is when the order filled — what every other broker uses.
    // "run date" is the settlement-batch date which lags by 1-3 days, and
    // "settlement date" is the cash-clearing date (T+2). Picking trade date
    // first avoids wash-sale and per-day P&L mis-bucketing.
    executed_at: ColSpec::HeaderAny(&["trade date", "run date", "settlement date"]),
    broker_order_id: None,
    asset_class: None,
    option_type: None,
    strike: None,
    expiration: None,
    multiplier: None,
    skip_symbols: &["", "total"],
};

// ===========================================================================
// TradeZero — Trade Activity CSV.
// ===========================================================================
pub struct TradeZeroParser;
impl Parser for TradeZeroParser {
    fn source(&self) -> &'static str {
        "tradezero"
    }
    fn parse(&self, bytes: &[u8]) -> Result<Vec<ParsedExecution>, ImportError> {
        parse_with(bytes, &MAP_TZ)
    }
}

const MAP_TZ: ColumnMap = ColumnMap {
    source: "tradezero",
    has_header: true,
    delimiter: b',',
    date_formats: &["%Y-%m-%d %H:%M:%S", "%m/%d/%Y %H:%M:%S"],
    utc_assumed: false,
    side_lookup: SideLookup::DEFAULT,
    symbol: ColSpec::HeaderAny(&["symbol"]),
    side: ColSpec::HeaderAny(&["side"]),
    qty: ColSpec::HeaderAny(&["qty", "shares", "quantity"]),
    price: ColSpec::HeaderAny(&["price"]),
    fee: Some(ColSpec::HeaderAny(&["fee", "commission"])),
    executed_at: ColSpec::HeaderAny(&["time", "datetime"]),
    broker_order_id: Some(ColSpec::HeaderAny(&["order id"])),
    asset_class: None,
    option_type: None,
    strike: None,
    expiration: None,
    multiplier: None,
    skip_symbols: &["", "total"],
};

// ===========================================================================
// Robinhood — Activity CSV.
// ===========================================================================
pub struct RobinhoodParser;
impl Parser for RobinhoodParser {
    fn source(&self) -> &'static str {
        "robinhood"
    }
    fn parse(&self, bytes: &[u8]) -> Result<Vec<ParsedExecution>, ImportError> {
        parse_with(bytes, &MAP_RH)
    }
}

const MAP_RH: ColumnMap = ColumnMap {
    source: "robinhood",
    has_header: true,
    delimiter: b',',
    date_formats: &["%Y-%m-%d", "%Y-%m-%dT%H:%M:%S"],
    utc_assumed: true,
    side_lookup: SideLookup::DEFAULT,
    // Real Robinhood activity CSVs put the ticker in "Symbol" and a long
    // API URL in "Instrument". First-match-wins on "instrument" would grab
    // the URL — flip the order so the ticker wins.
    symbol: ColSpec::HeaderAny(&["symbol", "instrument"]),
    side: ColSpec::HeaderAny(&["side"]),
    qty: ColSpec::HeaderAny(&["quantity", "qty"]),
    price: ColSpec::HeaderAny(&["price"]),
    fee: Some(ColSpec::HeaderAny(&["fees"])),
    executed_at: ColSpec::HeaderAny(&["execution date", "date"]),
    broker_order_id: Some(ColSpec::HeaderAny(&["order id"])),
    asset_class: None,
    option_type: None,
    strike: None,
    expiration: None,
    multiplier: None,
    skip_symbols: &["", "total"],
};

// ===========================================================================
// Generic CSV — uses widely-known column names. User can also POST a custom
// `ColumnMap` from the frontend (mapping wizard).
// ===========================================================================
#[derive(Default)]
pub struct GenericCsvParser;
impl Parser for GenericCsvParser {
    fn source(&self) -> &'static str {
        "generic"
    }
    fn parse(&self, bytes: &[u8]) -> Result<Vec<ParsedExecution>, ImportError> {
        parse_with(bytes, &MAP_GENERIC)
    }
}

const MAP_GENERIC: ColumnMap = ColumnMap {
    source: "generic",
    has_header: true,
    delimiter: b',',
    date_formats: &[
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%dT%H:%M:%S",
        "%m/%d/%Y %H:%M:%S",
        "%m/%d/%Y",
        "%Y-%m-%d",
    ],
    utc_assumed: false,
    side_lookup: SideLookup::DEFAULT,
    symbol: ColSpec::HeaderAny(&["symbol", "ticker", "instrument"]),
    side: ColSpec::HeaderAny(&["side", "action", "buy/sell", "transaction type"]),
    qty: ColSpec::HeaderAny(&["qty", "quantity", "shares", "filled", "size"]),
    price: ColSpec::HeaderAny(&["price", "avg price", "fill price"]),
    fee: Some(ColSpec::HeaderAny(&[
        "fee",
        "fees",
        "commission",
        "commissions",
    ])),
    executed_at: ColSpec::HeaderAny(&[
        "executed_at",
        "exec time",
        "time",
        "datetime",
        "date",
        "trade date",
    ]),
    broker_order_id: Some(ColSpec::HeaderAny(&["order id", "orderid", "id"])),
    asset_class: Some(ColSpec::HeaderAny(&["asset class", "type", "asset type"])),
    option_type: Some(ColSpec::HeaderAny(&["option type", "put/call", "c/p"])),
    strike: Some(ColSpec::HeaderAny(&["strike"])),
    expiration: Some(ColSpec::HeaderAny(&["expiration", "expiry", "exp"])),
    multiplier: Some(ColSpec::HeaderAny(&["multiplier"])),
    skip_symbols: &["", "total", "subtotal"],
};

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use traderview_core::Side;

    #[test]
    fn generic_csv_round_trip() {
        let csv = "Symbol,Side,Qty,Price,Fee,Date\n\
                   AAPL,buy,100,150.50,1.00,2026-01-15 09:30:00\n\
                   AAPL,sell,100,155.00,1.00,2026-01-15 14:30:00\n";
        let out = GenericCsvParser.parse(csv.as_bytes()).unwrap();
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].symbol, "AAPL");
        assert_eq!(out[0].side, Side::Buy);
        assert_eq!(out[0].qty.to_string(), "100");
        assert_eq!(out[0].fee.to_string(), "1.00");
        assert_eq!(out[1].side, Side::Sell);
    }

    #[test]
    fn generic_csv_skips_blank_and_total_rows() {
        let csv = "Symbol,Side,Qty,Price,Fee,Date\n\
                   AAPL,buy,100,150,1.00,2026-01-15 09:30:00\n\
                   ,,,,,\n\
                   Total,,,,,\n";
        let out = GenericCsvParser.parse(csv.as_bytes()).unwrap();
        assert_eq!(out.len(), 1);
    }

    #[test]
    fn generic_csv_aliases_resolve() {
        let csv = "Ticker,Action,Shares,Fill Price,Commission,DateTime\n\
                   TSLA,SELL,50,300.25,0.50,2026-02-03T10:15:00\n";
        let out = GenericCsvParser.parse(csv.as_bytes()).unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].symbol, "TSLA");
        assert_eq!(out[0].side, Side::Sell);
    }

    #[test]
    fn ibkr_basic_csv() {
        let csv = "Symbol,Buy/Sell,Quantity,TradePrice,IBCommission,DateTime,OrderID\n\
                   SPY,BUY,100,450.10,-1.20,2026-03-01 09:30:00,9999\n";
        let out = IbkrFlexParser.parse(csv.as_bytes()).unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].symbol, "SPY");
        assert_eq!(out[0].side, Side::Buy);
        assert_eq!(out[0].fee.to_string(), "1.20");
        assert_eq!(out[0].broker_order_id.as_deref(), Some("9999"));
    }

    #[test]
    fn das_short_codes() {
        let csv = "Symbol,B/S,Shares,Price,Fee,Time,OrderNo\n\
                   GME,SS,100,20.00,0.50,2026-01-01 09:30:00,1\n\
                   GME,BC,100,18.00,0.50,2026-01-01 09:31:00,2\n";
        let out = DasParser.parse(csv.as_bytes()).unwrap();
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].side, Side::Short);
        assert_eq!(out[1].side, Side::Cover);
    }

    // -----------------------------------------------------------------------
    // Per-broker round-trip tests.
    // Each constructs a CSV using the real header names from each MAP_* and
    // asserts the resulting ParsedExecution rows decode the symbol, side,
    // qty, price and (where utc_assumed=true) executed_at correctly.
    //
    // For maps with utc_assumed=false we avoid asserting on executed_at to
    // keep the tests host-timezone-independent (parse_datetime feeds the
    // naive timestamp through Utc::from_local_datetime in that branch).
    // -----------------------------------------------------------------------

    #[test]
    fn webull_round_trip() {
        let csv = "Symbol,Side,Filled,Avg Price,Commission,Filled Time,Order ID\n\
                   AAPL,Buy,200,182.45,0.00,01/15/2026 09:31:22,WB-1001\n\
                   AAPL,Sell,200,184.10,0.00,01/15/2026 14:55:10,WB-1002\n";
        let out = WebullParser.parse(csv.as_bytes()).unwrap();
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].symbol, "AAPL");
        assert_eq!(out[0].side, Side::Buy);
        assert_eq!(out[0].qty.to_string(), "200");
        assert_eq!(out[0].price.to_string(), "182.45");
        assert_eq!(out[0].broker_order_id.as_deref(), Some("WB-1001"));
        assert_eq!(out[1].side, Side::Sell);
    }

    #[test]
    fn lightspeed_round_trip() {
        let csv = "Symbol,Side,Shares,Price,Commission,Exec Time,Order Ref\n\
                   NVDA,buy,50,910.25,0.45,2026-03-04 10:02:15,LS-77\n\
                   NVDA,sell,50,915.00,0.45,2026-03-04 10:05:00,LS-78\n";
        let out = LightspeedParser.parse(csv.as_bytes()).unwrap();
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].symbol, "NVDA");
        assert_eq!(out[0].side, Side::Buy);
        assert_eq!(out[0].qty.to_string(), "50");
        assert_eq!(out[0].price.to_string(), "910.25");
        assert_eq!(out[0].fee.to_string(), "0.45");
        assert_eq!(out[0].broker_order_id.as_deref(), Some("LS-77"));
        // Lightspeed map sets utc_assumed=true, so the timestamp is stable.
        assert_eq!(out[0].executed_at.to_rfc3339(), "2026-03-04T10:02:15+00:00");
        assert_eq!(out[1].side, Side::Sell);
    }

    #[test]
    fn ibkr_flex_options_row() {
        // Exercise the options-aware fields (assetcategory / put-call / strike / expiry).
        let csv = "Symbol,Buy/Sell,Quantity,TradePrice,IBCommission,DateTime,OrderID,\
                   AssetCategory,Put/Call,Strike,Expiry,Multiplier\n\
                   SPY,BUY,2,3.40,-1.05,2026-03-04 10:02:15,IB-9000,\
                   OPT,C,520,2026-03-20,100\n";
        let out = IbkrFlexParser.parse(csv.as_bytes()).unwrap();
        assert_eq!(out.len(), 1);
        let r = &out[0];
        assert_eq!(r.symbol, "SPY");
        assert_eq!(r.side, Side::Buy);
        assert_eq!(r.qty.to_string(), "2");
        assert_eq!(r.price.to_string(), "3.40");
        assert_eq!(r.fee.to_string(), "1.05");
        assert_eq!(r.broker_order_id.as_deref(), Some("IB-9000"));
        // IBKR map sets utc_assumed=true so the executed_at is deterministic.
        assert_eq!(r.executed_at.to_rfc3339(), "2026-03-04T10:02:15+00:00");
        assert_eq!(r.asset_class, traderview_core::AssetClass::Option);
        assert_eq!(r.option_type, Some(traderview_core::OptionType::Call));
        assert_eq!(r.strike.as_ref().unwrap().to_string(), "520");
        assert_eq!(r.multiplier.to_string(), "100");
    }

    #[test]
    fn thinkorswim_round_trip() {
        // TOS uses 'BOT' / 'SOLD' and a two-digit-year date format.
        let csv = "Symbol,Side,Qty,Price,Commissions & Fees,Exec Time,Order ID\n\
                   MSFT,BOT,10,420.15,1.50,03/04/26 10:02:15,TOS-1\n\
                   MSFT,SOLD,10,423.00,1.50,03/04/26 10:30:00,TOS-2\n";
        let out = ThinkOrSwimParser.parse(csv.as_bytes()).unwrap();
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].symbol, "MSFT");
        assert_eq!(out[0].side, Side::Buy);
        assert_eq!(out[0].qty.to_string(), "10");
        assert_eq!(out[0].price.to_string(), "420.15");
        assert_eq!(out[0].fee.to_string(), "1.50");
        assert_eq!(out[1].side, Side::Sell);
    }

    #[test]
    fn schwab_round_trip() {
        // Schwab uses 'action' for side and 'fees & comm' for fees.
        let csv = "Date,Action,Symbol,Quantity,Price,Fees & Comm\n\
                   03/04/2026,Buy,QQQ,25,440.00,0.00\n\
                   03/04/2026,Sell,QQQ,25,442.50,0.00\n";
        let out = SchwabParser.parse(csv.as_bytes()).unwrap();
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].symbol, "QQQ");
        assert_eq!(out[0].side, Side::Buy);
        assert_eq!(out[0].qty.to_string(), "25");
        assert_eq!(out[0].price.to_string(), "440.00");
        assert_eq!(out[1].side, Side::Sell);
    }

    #[test]
    fn fidelity_phrase_actions() {
        // Fidelity's "Action" column uses verbose phrases like "YOU BOUGHT".
        let csv = "Run Date,Action,Symbol,Quantity,Price,Commission\n\
                   03/04/2026,YOU BOUGHT,VOO,3,505.10,0.00\n\
                   03/04/2026,YOU SOLD,VOO,3,507.00,0.00\n";
        let out = FidelityParser.parse(csv.as_bytes()).unwrap();
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].symbol, "VOO");
        assert_eq!(out[0].side, Side::Buy);
        assert_eq!(out[0].qty.to_string(), "3");
        assert_eq!(out[0].price.to_string(), "505.10");
        assert_eq!(out[1].side, Side::Sell);
    }

    #[test]
    fn etrade_round_trip() {
        let csv = "Transaction Date,Transaction Type,Symbol,Quantity,Price,Commission\n\
                   03/04/2026,Buy,IWM,40,200.10,0.00\n\
                   03/04/2026,Sell,IWM,40,201.25,0.00\n";
        let out = ETradeParser.parse(csv.as_bytes()).unwrap();
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].symbol, "IWM");
        assert_eq!(out[0].side, Side::Buy);
        assert_eq!(out[0].qty.to_string(), "40");
        assert_eq!(out[0].price.to_string(), "200.10");
        assert_eq!(out[1].side, Side::Sell);
    }

    #[test]
    fn robinhood_round_trip() {
        // Robinhood is utc_assumed=true and ships date-only execution dates.
        let csv = "Execution Date,Instrument,Side,Quantity,Price,Fees,Order ID\n\
                   2026-03-04,AMD,buy,15,175.00,0.00,RH-A1\n\
                   2026-03-04,AMD,sell,15,180.00,0.00,RH-A2\n";
        let out = RobinhoodParser.parse(csv.as_bytes()).unwrap();
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].symbol, "AMD");
        assert_eq!(out[0].side, Side::Buy);
        assert_eq!(out[0].qty.to_string(), "15");
        assert_eq!(out[0].price.to_string(), "175.00");
        assert_eq!(out[0].broker_order_id.as_deref(), Some("RH-A1"));
        // Date-only falls through parse_date → midnight UTC.
        assert_eq!(out[0].executed_at.to_rfc3339(), "2026-03-04T00:00:00+00:00");
    }

    #[test]
    fn tradestation_round_trip() {
        let csv = "Symbol,Side,Quantity,Price,Commission,Date/Time,Order ID\n\
                   ES,Buy,1,5200.25,2.50,03/04/2026 10:02:15,TS-1\n\
                   ES,Sell,1,5201.75,2.50,03/04/2026 10:03:00,TS-2\n";
        let out = TradeStationParser.parse(csv.as_bytes()).unwrap();
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].symbol, "ES");
        assert_eq!(out[0].side, Side::Buy);
        assert_eq!(out[0].qty.to_string(), "1");
        assert_eq!(out[0].price.to_string(), "5200.25");
        assert_eq!(out[0].fee.to_string(), "2.50");
        assert_eq!(out[0].broker_order_id.as_deref(), Some("TS-1"));
        assert_eq!(out[1].side, Side::Sell);
    }

    #[test]
    fn tdameritrade_round_trip() {
        let csv = "Date,Side,Symbol,Quantity,Price,Commission,Order ID\n\
                   03/04/2026 10:02:15,Buy,F,500,12.10,0.00,TDA-1\n\
                   03/04/2026 10:03:00,Sell,F,500,12.25,0.00,TDA-2\n";
        let out = TdAmeritradeParser.parse(csv.as_bytes()).unwrap();
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].symbol, "F");
        assert_eq!(out[0].side, Side::Buy);
        assert_eq!(out[0].qty.to_string(), "500");
        assert_eq!(out[0].price.to_string(), "12.10");
        assert_eq!(out[0].broker_order_id.as_deref(), Some("TDA-1"));
        assert_eq!(out[1].side, Side::Sell);
    }

    #[test]
    fn tradezero_round_trip() {
        let csv = "Symbol,Side,Qty,Price,Fee,Time,Order ID\n\
                   AMC,buy,300,4.50,0.10,2026-03-04 10:02:15,TZ-1\n\
                   AMC,sell,300,4.65,0.10,2026-03-04 10:03:00,TZ-2\n";
        let out = TradeZeroParser.parse(csv.as_bytes()).unwrap();
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].symbol, "AMC");
        assert_eq!(out[0].side, Side::Buy);
        assert_eq!(out[0].qty.to_string(), "300");
        assert_eq!(out[0].price.to_string(), "4.50");
        assert_eq!(out[0].fee.to_string(), "0.10");
        assert_eq!(out[0].broker_order_id.as_deref(), Some("TZ-1"));
        assert_eq!(out[1].side, Side::Sell);
    }

    // ─── Regression tests for column-map fixes (pass-5 audit) ─────────────

    /// Webull: when a CSV has BOTH `total qty` (order size) and `filled`
    /// (cumulative filled, may differ on partial-fill rows), the parser
    /// must take `total qty`. Pre-fix it took whichever came first in the
    /// alias list — `filled` — which mis-counted partial fills.
    #[test]
    fn webull_prefers_total_qty_over_filled() {
        let csv = "Symbol,Side,Total Qty,Filled,Price,Filled Time\n\
                   AAPL,buy,100,40,150,2026-01-15 09:30:00\n";
        let out = WebullParser.parse(csv.as_bytes()).unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(
            out[0].qty.to_string(),
            "100",
            "Webull must use Total Qty (order size), not Filled (partial)"
        );
    }

    /// TOS: "type" was previously aliased for BOTH `side` and `asset_class`.
    /// A CSV with a literal `Type` column would resolve side to the
    /// asset-class string ("STOCK" / "OPTION") and break side decoding.
    #[test]
    fn tos_side_resolves_independently_of_type_column() {
        let csv = "Symbol,Side,Type,Qty,Price,Exec Time\n\
                   AAPL,BOT,STOCK,100,150,01/15/26 09:30:00\n\
                   AAPL,SOLD,STOCK,100,160,01/15/26 14:30:00\n";
        let out = ThinkOrSwimParser.parse(csv.as_bytes()).unwrap();
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].side, Side::Buy, "BOT must decode as Buy");
        assert_eq!(out[1].side, Side::Sell, "SOLD must decode as Sell");
    }

    /// Robinhood: real activity exports put the ticker in `Symbol` and a
    /// long API URL in `Instrument`. Pre-fix, the parser preferred
    /// `instrument` and silently stored the URL as the symbol.
    #[test]
    fn robinhood_prefers_symbol_over_instrument_url() {
        let csv = "Instrument,Symbol,Side,Quantity,Price,Execution Date\n\
                   https://api.robinhood.com/instruments/abcd/,AAPL,buy,100,150,2026-01-15\n";
        let out = RobinhoodParser.parse(csv.as_bytes()).unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(
            out[0].symbol, "AAPL",
            "Robinhood must prefer Symbol column over Instrument URL"
        );
    }

    /// Fidelity: pre-fix the parser preferred `run date` (settlement-batch,
    /// lags by 1-3 days) over `trade date` (when the order filled).
    /// Wrong execution timestamp breaks per-day P&L bucketing and wash-sale
    /// detection.
    #[test]
    fn fidelity_prefers_trade_date_over_run_date() {
        use chrono::Datelike;
        let csv = "Symbol,Action,Quantity,Price,Run Date,Trade Date\n\
                   AAPL,YOU BOUGHT,100,150,01/17/2026,01/15/2026\n";
        let out = FidelityParser.parse(csv.as_bytes()).unwrap();
        assert_eq!(out.len(), 1);
        // Trade date is 01/15, run date is 01/17. We must pick trade date.
        let day = out[0].executed_at.date_naive();
        assert_eq!(day.day(), 15, "Fidelity must use trade date, not run date");
    }
}
