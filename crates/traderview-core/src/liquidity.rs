//! Liquidity report — how a trader's position size relates to average daily
//! volume for the symbol.
//!
//! Inputs: a list of trades + a map of symbol → average daily volume (ADV).
//! Output: per-symbol % of ADV consumed, with binned buckets.

use crate::models::Trade;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityRow {
    pub symbol: String,
    pub trades: usize,
    pub total_qty: Decimal,
    pub avg_qty_per_trade: Decimal,
    pub avg_daily_volume: Option<Decimal>,
    pub avg_pct_of_adv: Option<f64>,
    pub net_pnl: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LiquidityReport {
    pub rows: Vec<LiquidityRow>,
    pub buckets: Vec<LiquidityBucket>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityBucket {
    pub label: String,
    pub trades: usize,
    pub net_pnl: Decimal,
    pub win_rate: f64,
}

pub fn liquidity(trades: &[Trade], adv: &HashMap<String, Decimal>) -> LiquidityReport {
    let mut by_sym: BTreeMap<String, LiquidityRow> = BTreeMap::new();
    for t in trades {
        let row = by_sym.entry(t.symbol.clone()).or_insert(LiquidityRow {
            symbol: t.symbol.clone(),
            trades: 0,
            total_qty: Decimal::ZERO,
            avg_qty_per_trade: Decimal::ZERO,
            avg_daily_volume: adv.get(&t.symbol).copied(),
            avg_pct_of_adv: None,
            net_pnl: Decimal::ZERO,
        });
        row.trades += 1;
        row.total_qty += t.qty;
        row.net_pnl += t.net_pnl.unwrap_or(Decimal::ZERO);
    }
    for row in by_sym.values_mut() {
        if row.trades > 0 {
            row.avg_qty_per_trade = row.total_qty / Decimal::from(row.trades as u64);
        }
        if let Some(adv) = row.avg_daily_volume {
            if !adv.is_zero() {
                row.avg_pct_of_adv =
                    Some(decimal_to_f64(row.avg_qty_per_trade) / decimal_to_f64(adv));
            }
        }
    }

    // Bucket trades by their pct-of-ADV.
    let bucket_edges = [
        (0.0, 0.001),
        (0.001, 0.01),
        (0.01, 0.05),
        (0.05, 0.20),
        (0.20, f64::INFINITY),
    ];
    let bucket_labels = [
        "< 0.1% ADV",
        "0.1–1% ADV",
        "1–5% ADV",
        "5–20% ADV",
        "> 20% ADV",
    ];
    let mut buckets: Vec<LiquidityBucket> = bucket_labels
        .iter()
        .map(|l| LiquidityBucket {
            label: (*l).into(),
            trades: 0,
            net_pnl: Decimal::ZERO,
            win_rate: 0.0,
        })
        .collect();
    let mut wins_per_bucket = vec![0usize; buckets.len()];

    for t in trades {
        let Some(adv) = adv.get(&t.symbol) else {
            continue;
        };
        if adv.is_zero() {
            continue;
        }
        let pct = decimal_to_f64(t.qty) / decimal_to_f64(*adv);
        for (i, (lo, hi)) in bucket_edges.iter().enumerate() {
            if pct >= *lo && pct < *hi {
                let net = t.net_pnl.unwrap_or(Decimal::ZERO);
                buckets[i].trades += 1;
                buckets[i].net_pnl += net;
                if net > Decimal::ZERO {
                    wins_per_bucket[i] += 1;
                }
                break;
            }
        }
    }
    for (i, b) in buckets.iter_mut().enumerate() {
        if b.trades > 0 {
            b.win_rate = wins_per_bucket[i] as f64 / b.trades as f64;
        }
    }

    LiquidityReport {
        rows: by_sym.into_values().collect(),
        buckets,
    }
}

fn decimal_to_f64(d: Decimal) -> f64 {
    d.to_string().parse().unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AssetClass, TradeSide, TradeStatus};
    use chrono::{TimeZone, Utc};
    use std::str::FromStr;
    use uuid::Uuid;

    fn d(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    fn tr(symbol: &str, qty: &str, net: &str) -> Trade {
        Trade {
            id: Uuid::new_v4(),
            account_id: Uuid::nil(),
            symbol: symbol.into(),
            side: TradeSide::Long,
            status: TradeStatus::Closed,
            opened_at: Utc.with_ymd_and_hms(2026, 5, 1, 9, 30, 0).unwrap(),
            closed_at: Some(Utc.with_ymd_and_hms(2026, 5, 1, 15, 30, 0).unwrap()),
            qty: d(qty),
            entry_avg: d("100"),
            exit_avg: Some(d("110")),
            gross_pnl: Some(d(net)),
            fees: Decimal::ZERO,
            net_pnl: Some(d(net)),
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

    #[test]
    fn empty_trades_returns_no_rows() {
        let r = liquidity(&[], &HashMap::new());
        assert!(r.rows.is_empty());
        // Buckets are pre-allocated (5) but every count is zero.
        assert_eq!(r.buckets.len(), 5);
        assert!(r.buckets.iter().all(|b| b.trades == 0));
    }

    #[test]
    fn per_symbol_aggregates_qty_and_pnl() {
        let trades = vec![
            tr("AAPL", "100", "200"),
            tr("AAPL", "200", "300"),
            tr("TSLA", "50", "-100"),
        ];
        let r = liquidity(&trades, &HashMap::new());
        assert_eq!(r.rows.len(), 2);
        let aapl = r.rows.iter().find(|x| x.symbol == "AAPL").unwrap();
        assert_eq!(aapl.trades, 2);
        assert_eq!(aapl.total_qty, d("300"));
        assert_eq!(aapl.avg_qty_per_trade, d("150"));
        assert_eq!(aapl.net_pnl, d("500"));
    }

    #[test]
    fn pct_of_adv_is_none_when_adv_missing() {
        let trades = vec![tr("AAPL", "100", "0")];
        // No ADV map entry → pct should be None, not 0 or NaN.
        let r = liquidity(&trades, &HashMap::new());
        let row = &r.rows[0];
        assert!(row.avg_pct_of_adv.is_none());
        assert!(row.avg_daily_volume.is_none());
    }

    #[test]
    fn pct_of_adv_is_none_when_adv_zero() {
        let mut adv = HashMap::new();
        adv.insert("AAPL".to_string(), Decimal::ZERO);
        let trades = vec![tr("AAPL", "100", "0")];
        let r = liquidity(&trades, &adv);
        // Zero ADV must not divide-by-zero into NaN.
        assert!(r.rows[0].avg_pct_of_adv.is_none());
    }

    #[test]
    fn buckets_classify_by_pct_of_adv() {
        let mut adv = HashMap::new();
        adv.insert("AAPL".to_string(), d("100000")); // 100k ADV
                                                     // 50 shares = 0.05% ADV → bucket 0 (< 0.1%)
                                                     // 5,000 shares = 5% → bucket 3 (5-20%)
                                                     // 25,000 shares = 25% → bucket 4 (> 20%)
        let trades = vec![
            tr("AAPL", "50", "10"),
            tr("AAPL", "5000", "100"),
            tr("AAPL", "25000", "-50"),
        ];
        let r = liquidity(&trades, &adv);
        assert_eq!(r.buckets[0].trades, 1, "< 0.1% ADV bucket");
        assert_eq!(r.buckets[3].trades, 1, "5-20% ADV bucket");
        assert_eq!(r.buckets[4].trades, 1, "> 20% ADV bucket");
        assert_eq!(r.buckets[1].trades, 0);
        assert_eq!(r.buckets[2].trades, 0);
    }

    #[test]
    fn bucket_win_rate_computed_per_bucket() {
        let mut adv = HashMap::new();
        adv.insert("AAPL".to_string(), d("100000"));
        // Three trades in the < 0.1% bucket: 2 wins, 1 loss → 66.7% WR.
        let trades = vec![
            tr("AAPL", "10", "10"),
            tr("AAPL", "20", "20"),
            tr("AAPL", "30", "-5"),
        ];
        let r = liquidity(&trades, &adv);
        assert_eq!(r.buckets[0].trades, 3);
        assert!((r.buckets[0].win_rate - 2.0 / 3.0).abs() < 1e-9);
    }
}
