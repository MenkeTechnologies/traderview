//! Portfolio rebalancing math.
//!
//! Inputs:
//!   * `current`: array of (symbol, qty, price) for everything you hold + cash
//!   * `targets`: array of (symbol, weight) — weights sum to ≤ 1.0; the
//!     remainder is implicit cash
//!
//! Output: a Plan with:
//!   * `total_value` of the portfolio
//!   * Per-symbol rows (symbol, current_value, current_pct, target_pct,
//!     drift_pct, target_value, target_qty, trade_qty, trade_value, side)
//!   * The truncated `trades` list capped to `max_trades` by |trade_value|
//!     so a single rebalance never blows up into 30+ orders
//!
//! Whole-share constraint: `trade_qty` is always an integer (positive = buy,
//! negative = sell). Fractional residuals stay as drift on the row.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoldingInput {
    pub symbol: String,
    pub qty: f64,
    pub price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetInput {
    pub symbol: String,
    pub weight: f64,        // 0..1
    pub price: Option<f64>, // optional — falls back to current price when held
}

#[derive(Debug, Clone, Serialize)]
pub struct PlanRow {
    pub symbol: String,
    pub current_qty: f64,
    pub current_value: f64,
    pub current_pct: f64,
    pub target_pct: f64,
    pub drift_pct: f64, // current_pct - target_pct
    pub price: f64,
    pub target_value: f64,
    pub target_qty: i64, // whole-share rounded
    pub trade_qty: i64,  // positive = buy, negative = sell
    pub trade_value: f64,
    pub side: &'static str, // "buy" | "sell" | "hold"
}

#[derive(Debug, Clone, Serialize)]
pub struct Plan {
    pub total_value: f64,
    pub cash_current: f64,
    pub cash_target: f64,
    pub rows: Vec<PlanRow>,
    pub trades: Vec<PlanRow>, // subset: |trade_qty| > 0, sorted by |trade_value| desc, capped
    pub trade_count: usize,
    pub total_trade_value: f64,
    pub warnings: Vec<String>,
}

pub fn compute(
    current: &[HoldingInput],
    targets: &[TargetInput],
    cash: f64,
    max_trades: usize,
) -> Plan {
    let mut warnings = Vec::new();
    let sum_w: f64 = targets.iter().map(|t| t.weight).sum();
    if sum_w > 1.0 + 1e-9 {
        warnings.push(format!(
            "target weights sum to {:.4} (> 1.0) — over-allocated",
            sum_w
        ));
    }
    let holding_value: f64 = current.iter().map(|h| h.qty * h.price).sum();
    let total_value = holding_value + cash.max(0.0);

    // Index current holdings by symbol for quick lookup.
    let mut held: std::collections::BTreeMap<&str, &HoldingInput> = Default::default();
    for h in current {
        held.insert(h.symbol.as_str(), h);
    }

    // Index targets by symbol.
    let mut target_by_sym: std::collections::BTreeMap<&str, &TargetInput> = Default::default();
    for t in targets {
        target_by_sym.insert(t.symbol.as_str(), t);
    }

    // Union of all symbols (held ∪ targeted).
    let mut symbols: Vec<String> = Vec::new();
    for h in current {
        symbols.push(h.symbol.clone());
    }
    for t in targets {
        if !symbols.iter().any(|s| s == &t.symbol) {
            symbols.push(t.symbol.clone());
        }
    }
    symbols.sort();

    let mut rows: Vec<PlanRow> = Vec::with_capacity(symbols.len());
    for sym in &symbols {
        let cur = held.get(sym.as_str());
        let tgt = target_by_sym.get(sym.as_str());
        let price = tgt
            .and_then(|t| t.price)
            .or_else(|| cur.map(|c| c.price))
            .unwrap_or(0.0);
        let cur_qty = cur.map(|c| c.qty).unwrap_or(0.0);
        let cur_value = cur_qty * price;
        let cur_pct = if total_value > 0.0 {
            cur_value / total_value
        } else {
            0.0
        };
        let tgt_pct = tgt.map(|t| t.weight).unwrap_or(0.0);
        let tgt_value = total_value * tgt_pct;
        let tgt_qty = if price > 0.0 {
            (tgt_value / price).floor() as i64
        } else {
            0
        };
        let trade_qty = tgt_qty - cur_qty as i64;
        let side = if trade_qty > 0 {
            "buy"
        } else if trade_qty < 0 {
            "sell"
        } else {
            "hold"
        };
        let trade_value = trade_qty as f64 * price;
        rows.push(PlanRow {
            symbol: sym.clone(),
            current_qty: cur_qty,
            current_value: cur_value,
            current_pct: cur_pct,
            target_pct: tgt_pct,
            drift_pct: cur_pct - tgt_pct,
            price,
            target_value: tgt_value,
            target_qty: tgt_qty,
            trade_qty,
            trade_value,
            side,
        });
    }

    // Build truncated trade list — biggest |$| moves first, cap to max_trades.
    let mut trade_pool: Vec<PlanRow> = rows.iter().filter(|r| r.trade_qty != 0).cloned().collect();
    trade_pool.sort_by(|a, b| {
        b.trade_value
            .abs()
            .partial_cmp(&a.trade_value.abs())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let trades: Vec<PlanRow> = if max_trades > 0 {
        trade_pool.into_iter().take(max_trades).collect()
    } else {
        trade_pool
    };

    let total_trade_value: f64 = trades.iter().map(|r| r.trade_value.abs()).sum();
    let cash_target = total_value * (1.0 - sum_w.min(1.0));

    Plan {
        total_value,
        cash_current: cash.max(0.0),
        cash_target,
        rows,
        trade_count: trades.len(),
        trades,
        total_trade_value,
        warnings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_drift_yields_no_trades() {
        // Perfectly aligned: 50/50 SPY+QQQ at $100 each, 10 shares each = $2000 total.
        let cur = vec![
            HoldingInput {
                symbol: "SPY".into(),
                qty: 10.0,
                price: 100.0,
            },
            HoldingInput {
                symbol: "QQQ".into(),
                qty: 10.0,
                price: 100.0,
            },
        ];
        let tgt = vec![
            TargetInput {
                symbol: "SPY".into(),
                weight: 0.5,
                price: None,
            },
            TargetInput {
                symbol: "QQQ".into(),
                weight: 0.5,
                price: None,
            },
        ];
        let p = compute(&cur, &tgt, 0.0, 10);
        assert_eq!(p.trade_count, 0);
        assert!((p.total_value - 2000.0).abs() < 1e-6);
    }

    #[test]
    fn buy_and_sell_to_meet_targets() {
        // $10k portfolio, currently 100% SPY (100 shares @ $100). Target 50/50 with QQQ @ $200.
        // After rebalance: $5k SPY = 50 shares, $5k QQQ = 25 shares.
        // Trades: sell 50 SPY, buy 25 QQQ.
        let cur = vec![HoldingInput {
            symbol: "SPY".into(),
            qty: 100.0,
            price: 100.0,
        }];
        let tgt = vec![
            TargetInput {
                symbol: "SPY".into(),
                weight: 0.5,
                price: None,
            },
            TargetInput {
                symbol: "QQQ".into(),
                weight: 0.5,
                price: Some(200.0),
            },
        ];
        let p = compute(&cur, &tgt, 0.0, 10);
        let spy = p.rows.iter().find(|r| r.symbol == "SPY").unwrap();
        let qqq = p.rows.iter().find(|r| r.symbol == "QQQ").unwrap();
        assert_eq!(spy.target_qty, 50);
        assert_eq!(spy.trade_qty, -50);
        assert_eq!(spy.side, "sell");
        assert_eq!(qqq.target_qty, 25);
        assert_eq!(qqq.trade_qty, 25);
        assert_eq!(qqq.side, "buy");
        assert_eq!(p.trade_count, 2);
    }

    #[test]
    fn max_trades_caps_to_biggest_dollar_moves() {
        // 5 holdings all drift; cap to 2 — should pick the two biggest $ moves.
        let cur = vec![
            HoldingInput {
                symbol: "A".into(),
                qty: 100.0,
                price: 10.0,
            }, // $1000
            HoldingInput {
                symbol: "B".into(),
                qty: 100.0,
                price: 50.0,
            }, // $5000
            HoldingInput {
                symbol: "C".into(),
                qty: 100.0,
                price: 100.0,
            }, // $10000 ← biggest sell
            HoldingInput {
                symbol: "D".into(),
                qty: 100.0,
                price: 20.0,
            }, // $2000
            HoldingInput {
                symbol: "E".into(),
                qty: 100.0,
                price: 80.0,
            }, // $8000 ← 2nd biggest sell
        ];
        // Liquidate everything: all weights = 0 → sell everything.
        let p = compute(&cur, &[], 0.0, 2);
        assert_eq!(p.trade_count, 2);
        let syms: Vec<&str> = p.trades.iter().map(|r| r.symbol.as_str()).collect();
        // Biggest two by |trade_value| are C ($10k) and E ($8k).
        assert!(syms.contains(&"C"));
        assert!(syms.contains(&"E"));
    }

    #[test]
    fn over_allocated_warning() {
        let cur = vec![];
        let tgt = vec![
            TargetInput {
                symbol: "A".into(),
                weight: 0.6,
                price: Some(10.0),
            },
            TargetInput {
                symbol: "B".into(),
                weight: 0.6,
                price: Some(10.0),
            },
        ];
        let p = compute(&cur, &tgt, 10_000.0, 10);
        assert!(p.warnings.iter().any(|w| w.contains("over-allocated")));
    }
}
