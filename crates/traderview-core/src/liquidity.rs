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
                row.avg_pct_of_adv = Some(
                    decimal_to_f64(row.avg_qty_per_trade) / decimal_to_f64(adv),
                );
            }
        }
    }

    // Bucket trades by their pct-of-ADV.
    let bucket_edges = [(0.0, 0.001), (0.001, 0.01), (0.01, 0.05), (0.05, 0.20), (0.20, f64::INFINITY)];
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
