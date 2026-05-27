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
    fn source(&self) -> &'static str { "webull" }
    fn parse(&self, bytes: &[u8]) -> Result<Vec<ParsedExecution>, ImportError> {
        parse_with(bytes, &MAP_WEBULL)
    }
}

const MAP_WEBULL: ColumnMap = ColumnMap {
    source: "webull",
    has_header: true,
    delimiter: b',',
    date_formats: &[
        "%m/%d/%Y %H:%M:%S",
        "%Y-%m-%d %H:%M:%S",
        "%m/%d/%Y %H:%M",
    ],
    utc_assumed: false, // Webull exports are usually local; user can override later via tz
    side_lookup: SideLookup::DEFAULT,
    symbol: ColSpec::HeaderAny(&["symbol", "ticker"]),
    side: ColSpec::HeaderAny(&["side", "action", "buy/sell", "order side"]),
    qty: ColSpec::HeaderAny(&["filled", "quantity", "qty", "total qty", "filled qty"]),
    price: ColSpec::HeaderAny(&["avg price", "price", "filled price", "avgprice"]),
    fee: Some(ColSpec::HeaderAny(&["commission", "fees", "commission & fees"])),
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
    fn source(&self) -> &'static str { "ibkr" }
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
    fn source(&self) -> &'static str { "tdameritrade" }
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
    fn source(&self) -> &'static str { "schwab" }
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
    fn source(&self) -> &'static str { "tradestation" }
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
    fn source(&self) -> &'static str { "lightspeed" }
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
    fn source(&self) -> &'static str { "das" }
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
    fn source(&self) -> &'static str { "tos" }
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
    side: ColSpec::HeaderAny(&["side", "type"]),
    qty: ColSpec::HeaderAny(&["qty", "quantity"]),
    price: ColSpec::HeaderAny(&["price"]),
    fee: Some(ColSpec::HeaderAny(&["commissions & fees", "commission"])),
    executed_at: ColSpec::HeaderAny(&["exec time", "date/time"]),
    broker_order_id: Some(ColSpec::HeaderAny(&["order id"])),
    asset_class: Some(ColSpec::HeaderAny(&["type", "asset type"])),
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
    fn source(&self) -> &'static str { "etrade" }
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
    fn source(&self) -> &'static str { "fidelity" }
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
    executed_at: ColSpec::HeaderAny(&["run date", "settlement date", "trade date"]),
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
    fn source(&self) -> &'static str { "tradezero" }
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
    fn source(&self) -> &'static str { "robinhood" }
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
    symbol: ColSpec::HeaderAny(&["instrument", "symbol"]),
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
    fn source(&self) -> &'static str { "generic" }
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
    fee: Some(ColSpec::HeaderAny(&["fee", "fees", "commission", "commissions"])),
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
}
