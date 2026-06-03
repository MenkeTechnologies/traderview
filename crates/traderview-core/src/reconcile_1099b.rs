//! 1099-B reconciliation.
//!
//! Brokers issue a 1099-B at year-end with their version of every
//! closing trade + the basis they reported to the IRS. The user's
//! own roll-up (FIFO derived) should match — when it doesn't, the
//! delta is usually a wash-sale adjustment the broker applied that
//! the user's local data missed.
//!
//! Pure compute. Inputs: user trades (closed only, with closed_at and
//! gross_pnl) + 1099-B rows (whatever the broker emits). Output:
//! per-symbol comparison + delta + flags.

use crate::models::{Trade, TradeStatus};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct B1099Row {
    pub symbol: String,
    pub closed_year: i32,
    /// Proceeds the broker reported (sale total).
    pub proceeds: Decimal,
    /// Cost basis the broker reported.
    pub cost_basis: Decimal,
    /// Wash-sale loss disallowed by the broker (positive number = how
    /// much the loss was reduced).
    pub wash_sale_disallowed: Decimal,
}

impl B1099Row {
    pub fn reported_gain(&self) -> Decimal {
        // gain = proceeds - basis + wash_sale_disallowed (since the
        // disallowed loss inflates the gain reported).
        self.proceeds - self.cost_basis + self.wash_sale_disallowed
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolRecon {
    pub symbol: String,
    /// User's computed gross P&L from local trades (closed in the year).
    pub user_pnl: Decimal,
    /// Broker's reported gain (proceeds - basis + wash sale).
    pub broker_gain: Decimal,
    /// user_pnl - broker_gain. Positive = user thinks they made MORE
    /// than the broker reports; negative = broker reports MORE gain
    /// (usually a wash-sale catch).
    pub delta: Decimal,
    /// Wash-sale loss the broker disallowed. Zero if none.
    pub wash_sale_disallowed: Decimal,
    /// Heuristic: |delta| > $1 AND > 1% of broker_gain.
    pub flagged: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReconReport {
    pub year: i32,
    pub by_symbol: Vec<SymbolRecon>,
    pub total_user_pnl: Decimal,
    pub total_broker_gain: Decimal,
    pub total_delta: Decimal,
    pub flagged_count: usize,
}

pub fn reconcile(year: i32, trades: &[Trade], rows: &[B1099Row]) -> ReconReport {
    // Sum user gross P&L per symbol for trades closed IN this tax year.
    let mut user: BTreeMap<String, Decimal> = BTreeMap::new();
    for t in trades {
        if t.status != TradeStatus::Closed {
            continue;
        }
        let Some(closed_at) = t.closed_at else {
            continue;
        };
        if closed_at
            .date_naive()
            .format("%Y")
            .to_string()
            .parse::<i32>()
            .map(|y| y != year)
            .unwrap_or(true)
        {
            continue;
        }
        let pnl = t.gross_pnl.unwrap_or(Decimal::ZERO);
        *user.entry(t.symbol.clone()).or_insert(Decimal::ZERO) += pnl;
    }

    // Sum broker rows per symbol (for the year).
    let mut broker: BTreeMap<String, (Decimal, Decimal)> = BTreeMap::new();
    for r in rows {
        if r.closed_year != year {
            continue;
        }
        let e = broker
            .entry(r.symbol.clone())
            .or_insert((Decimal::ZERO, Decimal::ZERO));
        e.0 += r.reported_gain();
        e.1 += r.wash_sale_disallowed;
    }

    let mut by_symbol: Vec<SymbolRecon> = Vec::new();
    let mut total_user = Decimal::ZERO;
    let mut total_broker = Decimal::ZERO;
    let all_symbols: std::collections::BTreeSet<String> =
        user.keys().chain(broker.keys()).cloned().collect();
    let one_dollar = Decimal::ONE;
    for sym in all_symbols {
        let u = *user.get(&sym).unwrap_or(&Decimal::ZERO);
        let (b, wash) = *broker.get(&sym).unwrap_or(&(Decimal::ZERO, Decimal::ZERO));
        let delta = u - b;
        // Flag if delta > $1 absolute AND > 1% of broker (when broker
        // non-zero). Avoids false positives on penny-rounding.
        let pct_threshold = if b.is_zero() {
            Decimal::ZERO
        } else {
            b.abs() / Decimal::from(100)
        };
        let flagged = delta.abs() > one_dollar && (b.is_zero() || delta.abs() > pct_threshold);
        by_symbol.push(SymbolRecon {
            symbol: sym,
            user_pnl: u,
            broker_gain: b,
            delta,
            wash_sale_disallowed: wash,
            flagged,
        });
        total_user += u;
        total_broker += b;
    }
    // Sort by absolute delta descending so user sees the biggest
    // mismatches first.
    by_symbol.sort_by_key(|a| std::cmp::Reverse(a.delta.abs()));
    let flagged_count = by_symbol.iter().filter(|s| s.flagged).count();

    ReconReport {
        year,
        by_symbol,
        total_user_pnl: total_user,
        total_broker_gain: total_broker,
        total_delta: total_user - total_broker,
        flagged_count,
    }
}

/// Convenience: parse `1099-B`-shape CSV row strings into B1099Row.
/// Format expected: `symbol,year,proceeds,cost_basis,wash_sale_disallowed`.
pub fn parse_row(line: &str) -> Option<B1099Row> {
    let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
    if parts.len() < 5 {
        return None;
    }
    let _ = NaiveDate::from_ymd_opt(2026, 1, 1)?; // silence unused import
    Some(B1099Row {
        symbol: parts[0].to_ascii_uppercase(),
        closed_year: parts[1].parse().ok()?,
        proceeds: parts[2].parse().ok()?,
        cost_basis: parts[3].parse().ok()?,
        wash_sale_disallowed: parts[4].parse().ok()?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AssetClass, TradeSide};
    use chrono::{TimeZone, Utc};
    use std::str::FromStr;
    use uuid::Uuid;

    fn d(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    fn closed_trade(symbol: &str, year: i32, gross: &str) -> Trade {
        Trade {
            id: Uuid::new_v4(),
            account_id: Uuid::nil(),
            symbol: symbol.into(),
            side: TradeSide::Long,
            status: TradeStatus::Closed,
            opened_at: Utc.with_ymd_and_hms(year, 1, 1, 9, 30, 0).unwrap(),
            closed_at: Some(Utc.with_ymd_and_hms(year, 6, 15, 15, 30, 0).unwrap()),
            qty: d("100"),
            entry_avg: d("50"),
            exit_avg: Some(d("60")),
            gross_pnl: Some(d(gross)),
            fees: Decimal::ZERO,
            commissions: Decimal::ZERO,
            net_pnl: Some(d(gross)),
            asset_class: AssetClass::Stock,
            option_type: None,
            strike: None,
            expiration: None,
            multiplier: Decimal::ONE,
            tick_size: None,
            tick_value: None,
            base_ccy: None,
            quote_ccy: None,
            pip_size: None,
            stop_loss: None,
            risk_amount: None,
            initial_target: None,
            mfe: None,
            mae: None,
            best_exit_pnl: None,
            exit_efficiency: None,
        }
    }

    fn b1099(symbol: &str, year: i32, proceeds: &str, basis: &str, wash: &str) -> B1099Row {
        B1099Row {
            symbol: symbol.into(),
            closed_year: year,
            proceeds: d(proceeds),
            cost_basis: d(basis),
            wash_sale_disallowed: d(wash),
        }
    }

    #[test]
    fn matching_user_and_broker_unflagged() {
        let trades = vec![closed_trade("AAPL", 2026, "1000")];
        let rows = vec![b1099("AAPL", 2026, "6000", "5000", "0")]; // gain = 1000
        let r = reconcile(2026, &trades, &rows);
        assert_eq!(r.by_symbol.len(), 1);
        let s = &r.by_symbol[0];
        assert_eq!(s.user_pnl, d("1000"));
        assert_eq!(s.broker_gain, d("1000"));
        assert_eq!(s.delta, Decimal::ZERO);
        assert!(!s.flagged);
    }

    #[test]
    fn wash_sale_disallowed_inflates_broker_gain() {
        // User says they lost $500 net. Broker disallowed $200 of wash
        // sale loss → reports a $300 loss.
        let trades = vec![closed_trade("AAPL", 2026, "-500")];
        let rows = vec![b1099("AAPL", 2026, "4500", "5000", "200")];
        // broker reported_gain = 4500 - 5000 + 200 = -300.
        let r = reconcile(2026, &trades, &rows);
        let s = &r.by_symbol[0];
        assert_eq!(s.broker_gain, d("-300"));
        // Delta = -500 - (-300) = -200. Flagged.
        assert_eq!(s.delta, d("-200"));
        assert_eq!(s.wash_sale_disallowed, d("200"));
        assert!(s.flagged);
    }

    #[test]
    fn penny_drift_not_flagged() {
        // 1¢ mismatch on a $1000 gain is rounding noise, not actionable.
        let trades = vec![closed_trade("AAPL", 2026, "1000.50")];
        let rows = vec![b1099("AAPL", 2026, "6000", "5000", "0")];
        let r = reconcile(2026, &trades, &rows);
        let s = &r.by_symbol[0];
        assert_eq!(s.delta, d("0.50"));
        assert!(!s.flagged, "sub-$1 delta must not flag");
    }

    #[test]
    fn trade_in_wrong_year_excluded() {
        let trades = vec![
            closed_trade("AAPL", 2026, "100"),
            closed_trade("AAPL", 2025, "9999"), // prior year — ignored
        ];
        let rows = vec![b1099("AAPL", 2026, "100", "0", "0")];
        let r = reconcile(2026, &trades, &rows);
        assert_eq!(r.by_symbol[0].user_pnl, d("100"));
    }

    #[test]
    fn broker_only_symbol_appears_with_zero_user_pnl() {
        // Broker reported a symbol the user has no trade for — could be
        // a fee adjustment or an old open lot. Surface it.
        let trades: Vec<Trade> = vec![];
        let rows = vec![b1099("GME", 2026, "100", "50", "0")];
        let r = reconcile(2026, &trades, &rows);
        assert_eq!(r.by_symbol.len(), 1);
        assert_eq!(r.by_symbol[0].user_pnl, Decimal::ZERO);
        assert_eq!(r.by_symbol[0].broker_gain, d("50"));
        assert!(r.by_symbol[0].flagged);
    }

    #[test]
    fn user_only_symbol_appears_with_zero_broker_gain() {
        // Trade the user closed but broker didn't report (probably
        // crypto on a non-1099 venue). Flagged so user investigates.
        let trades = vec![closed_trade("BTC", 2026, "500")];
        let rows: Vec<B1099Row> = vec![];
        let r = reconcile(2026, &trades, &rows);
        assert_eq!(r.by_symbol[0].symbol, "BTC");
        assert!(r.by_symbol[0].flagged);
    }

    #[test]
    fn sorted_by_absolute_delta_descending() {
        let trades = vec![
            closed_trade("A", 2026, "10"),
            closed_trade("B", 2026, "1000"),
            closed_trade("C", 2026, "0"),
        ];
        let rows = vec![
            b1099("A", 2026, "10", "0", "0"),  // delta 0
            b1099("B", 2026, "500", "0", "0"), // delta 500
            b1099("C", 2026, "0", "0", "0"),   // delta 0
        ];
        let r = reconcile(2026, &trades, &rows);
        assert_eq!(r.by_symbol[0].symbol, "B", "biggest delta first");
    }

    #[test]
    fn parse_row_handles_well_formed_csv() {
        let row = parse_row("AAPL, 2026, 6000.00, 5000.00, 0.00").unwrap();
        assert_eq!(row.symbol, "AAPL");
        assert_eq!(row.closed_year, 2026);
        assert_eq!(row.proceeds, d("6000.00"));
    }

    #[test]
    fn parse_row_returns_none_on_short_rows() {
        assert!(parse_row("AAPL,2026,6000").is_none());
        assert!(parse_row("").is_none());
    }

    #[test]
    fn parse_row_uppercases_symbol() {
        let row = parse_row("aapl,2026,1,1,0").unwrap();
        assert_eq!(row.symbol, "AAPL");
    }

    #[test]
    fn totals_sum_correctly_across_symbols() {
        let trades = vec![
            closed_trade("AAPL", 2026, "1000"),
            closed_trade("TSLA", 2026, "-500"),
        ];
        let rows = vec![
            b1099("AAPL", 2026, "6000", "5000", "0"),
            b1099("TSLA", 2026, "4500", "5000", "0"),
        ];
        let r = reconcile(2026, &trades, &rows);
        assert_eq!(r.total_user_pnl, d("500")); // 1000 - 500
        assert_eq!(r.total_broker_gain, d("500"));
        assert_eq!(r.total_delta, Decimal::ZERO);
    }
}
