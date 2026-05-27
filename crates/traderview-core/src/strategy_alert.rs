//! Compound strategy-alert AST + evaluator.
//!
//! AST shape (serialized as tagged JSON):
//!   `{ "kind": "leaf", "symbol": "AAPL", "metric": { "kind": "rsi", "period": 14 },`
//!     `"op": "lt", "value": 30.0 }`
//!   `{ "kind": "and",  "left": <Node>, "right": <Node> }`
//!   `{ "kind": "or",   "left": <Node>, "right": <Node> }`
//!   `{ "kind": "not",  "node": <Node> }`
//!
//! The evaluator takes a `MetricResolver` closure that the calling crate
//! wires up to whatever data sources it has (cached bars, live quotes,
//! breadth snapshot, etc.). This keeps `traderview-core` free of DB deps.

use crate::indicators::{ema, rsi, sma};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Metric {
    /// Latest quote price.
    Price,
    /// % change over last N daily bars (today vs N-back close).
    ChangePct { days: u32 },
    /// SMA(period) of close.
    Sma { period: u32 },
    /// EMA(period) of close.
    Ema { period: u32 },
    /// RSI(period).
    Rsi { period: u32 },
    /// Price as a percentage of (period)-day high. 1.0 = at high, 0.95 = -5%.
    PctOfHigh { period: u32 },
    /// Generic literal — `symbol` is the value alias, e.g. ^VIX or ^CPC,
    /// resolved as last quote.
    Quote,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Op {
    Gt,
    Ge,
    Lt,
    Le,
    Eq,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Node {
    Leaf {
        symbol: String,
        metric: Metric,
        op: Op,
        value: f64,
    },
    And {
        left: Box<Node>,
        right: Box<Node>,
    },
    Or {
        left: Box<Node>,
        right: Box<Node>,
    },
    Not {
        node: Box<Node>,
    },
}

/// What the resolver hands back for one (symbol, metric) request.
#[derive(Debug, Clone)]
pub struct MetricInput {
    /// Latest price/quote for the symbol.
    pub latest_price: Option<f64>,
    /// Close-price series for the symbol (ascending order), used for
    /// SMA/EMA/RSI/ChangePct/PctOfHigh. Empty when not available.
    pub closes: Vec<f64>,
}

/// Evaluate a node, also recording all leaf evaluations into `trace` so the
/// caller can persist a fire snapshot.
pub fn evaluate<F>(node: &Node, resolver: &mut F, trace: &mut Vec<LeafEval>) -> bool
where
    F: FnMut(&str, &Metric) -> MetricInput,
{
    match node {
        Node::Leaf {
            symbol,
            metric,
            op,
            value,
        } => {
            let input = resolver(symbol, metric);
            let actual = compute_metric(metric, &input);
            let truth = match actual {
                Some(v) => compare(v, *op, *value),
                None => false,
            };
            trace.push(LeafEval {
                symbol: symbol.clone(),
                metric: metric.clone(),
                op: *op,
                threshold: *value,
                actual,
                truth,
            });
            truth
        }
        Node::And { left, right } => {
            evaluate(left, resolver, trace) && evaluate(right, resolver, trace)
        }
        Node::Or { left, right } => {
            evaluate(left, resolver, trace) || evaluate(right, resolver, trace)
        }
        Node::Not { node } => !evaluate(node, resolver, trace),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeafEval {
    pub symbol: String,
    pub metric: Metric,
    pub op: Op,
    pub threshold: f64,
    pub actual: Option<f64>,
    pub truth: bool,
}

fn compute_metric(m: &Metric, i: &MetricInput) -> Option<f64> {
    match m {
        Metric::Price | Metric::Quote => i.latest_price,
        Metric::ChangePct { days } => {
            let n = i.closes.len();
            let need = (*days as usize) + 1;
            if n < need {
                return None;
            }
            let last = *i.closes.last()?;
            let prior = i.closes[n - need];
            if prior <= 0.0 {
                return None;
            }
            Some((last - prior) / prior * 100.0)
        }
        Metric::Sma { period } => sma(&i.closes, *period as usize).last()?.to_owned(),
        Metric::Ema { period } => ema(&i.closes, *period as usize).last()?.to_owned(),
        Metric::Rsi { period } => rsi(&i.closes, *period as usize).last()?.to_owned(),
        Metric::PctOfHigh { period } => {
            let n = i.closes.len();
            let p = (*period as usize).min(n);
            if p == 0 {
                return None;
            }
            let window = &i.closes[n - p..];
            let hi = window.iter().cloned().fold(f64::MIN, f64::max);
            let last = *i.closes.last()?;
            if hi <= 0.0 {
                return None;
            }
            Some(last / hi)
        }
    }
}

fn compare(a: f64, op: Op, b: f64) -> bool {
    match op {
        Op::Gt => a > b,
        Op::Ge => a >= b,
        Op::Lt => a < b,
        Op::Le => a <= b,
        Op::Eq => (a - b).abs() < 1e-9,
    }
}

/// Convenience: collect all distinct (symbol, max_history_bars_needed) tuples
/// so the caller can do exactly the data fetches it needs without walking the
/// AST a second time.
pub fn collect_symbols(node: &Node, out: &mut HashMap<String, u32>) {
    match node {
        Node::Leaf { symbol, metric, .. } => {
            let need = match metric {
                Metric::Price | Metric::Quote => 0,
                Metric::ChangePct { days } => *days + 1,
                Metric::Sma { period }
                | Metric::Ema { period }
                | Metric::Rsi { period }
                | Metric::PctOfHigh { period } => *period + 5,
            };
            let entry = out.entry(symbol.clone()).or_insert(0);
            if need > *entry {
                *entry = need;
            }
        }
        Node::And { left, right } | Node::Or { left, right } => {
            collect_symbols(left, out);
            collect_symbols(right, out);
        }
        Node::Not { node } => collect_symbols(node, out),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn closes(n: usize, start: f64, step: f64) -> Vec<f64> {
        (0..n).map(|i| start + step * i as f64).collect()
    }

    fn resolver_for(
        sym: &str,
        price: f64,
        cls: Vec<f64>,
    ) -> impl FnMut(&str, &Metric) -> MetricInput {
        let s = sym.to_string();
        move |q: &str, _m: &Metric| {
            if q == s {
                MetricInput {
                    latest_price: Some(price),
                    closes: cls.clone(),
                }
            } else {
                MetricInput {
                    latest_price: None,
                    closes: vec![],
                }
            }
        }
    }

    #[test]
    fn leaf_price_gt_threshold() {
        let node = Node::Leaf {
            symbol: "AAPL".into(),
            metric: Metric::Price,
            op: Op::Gt,
            value: 100.0,
        };
        let mut r = resolver_for("AAPL", 150.0, vec![]);
        let mut t = vec![];
        assert!(evaluate(&node, &mut r, &mut t));
        assert_eq!(t.len(), 1);
        assert!(t[0].truth);
        assert_eq!(t[0].actual, Some(150.0));
    }

    #[test]
    fn and_short_circuits_on_left_false() {
        // Left = false (price 50 > 100 is false). Right should still be evaluated
        // — Rust && short-circuits, so right won't be evaluated. Trace will have
        // 1 entry only.
        let left = Node::Leaf {
            symbol: "AAPL".into(),
            metric: Metric::Price,
            op: Op::Gt,
            value: 100.0,
        };
        let right = Node::Leaf {
            symbol: "AAPL".into(),
            metric: Metric::Price,
            op: Op::Lt,
            value: 200.0,
        };
        let node = Node::And {
            left: Box::new(left),
            right: Box::new(right),
        };
        let mut r = resolver_for("AAPL", 50.0, vec![]);
        let mut t = vec![];
        assert!(!evaluate(&node, &mut r, &mut t));
        assert_eq!(t.len(), 1, "short-circuit should leave right unevaluated");
    }

    #[test]
    fn rsi_lt_with_real_series() {
        // 30 declining closes → RSI(14) trends toward 0. Should fire on `< 30`.
        let mut c = closes(30, 100.0, 0.0);
        for (i, slot) in c.iter_mut().enumerate().take(30).skip(15) {
            *slot = 100.0 - (i - 14) as f64 * 2.0;
        }
        let node = Node::Leaf {
            symbol: "X".into(),
            metric: Metric::Rsi { period: 14 },
            op: Op::Lt,
            value: 30.0,
        };
        let mut r = resolver_for("X", *c.last().unwrap(), c.clone());
        let mut t = vec![];
        let truth = evaluate(&node, &mut r, &mut t);
        assert!(truth, "actual RSI = {:?}", t[0].actual);
        assert!(t[0].actual.unwrap() < 30.0);
    }

    #[test]
    fn collect_symbols_aggregates_max_history() {
        let n = Node::And {
            left: Box::new(Node::Leaf {
                symbol: "AAPL".into(),
                metric: Metric::Sma { period: 20 },
                op: Op::Gt,
                value: 0.0,
            }),
            right: Box::new(Node::Leaf {
                symbol: "AAPL".into(),
                metric: Metric::Sma { period: 200 },
                op: Op::Gt,
                value: 0.0,
            }),
        };
        let mut h = HashMap::new();
        collect_symbols(&n, &mut h);
        assert_eq!(h.len(), 1);
        // Max should be 205 (200 + 5 padding).
        assert_eq!(*h.get("AAPL").unwrap(), 205);
    }
}
