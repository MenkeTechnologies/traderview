//! Mark-to-market reconciliation tool.
//!
//! Compares the broker's end-of-day position values against the
//! internal accounting system. Flags differences > threshold (could be
//! a missed corporate action, dividend, transfer, or pricing snapshot
//! discrepancy).
//!
//! Pure compute.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokerPosition {
    pub symbol: String,
    pub qty: Decimal,
    pub mark_price: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalPosition {
    pub symbol: String,
    pub qty: Decimal,
    pub mark_price: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mismatch {
    pub symbol: String,
    pub broker_qty: Decimal,
    pub internal_qty: Decimal,
    pub broker_value: Decimal,
    pub internal_value: Decimal,
    pub difference: Decimal,
    pub kind: MismatchKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MismatchKind {
    QtyDiffers,
    PriceDiffers,
    MissingInternal,    // broker has position, internal doesn't
    MissingBroker,      // internal has position, broker doesn't
    ValueDiffers,       // qty + price both match but value differs (e.g. fees)
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReconciliationReport {
    pub broker_total: Decimal,
    pub internal_total: Decimal,
    pub total_difference: Decimal,
    pub mismatches: Vec<Mismatch>,
    pub matched_count: usize,
}

pub fn reconcile(
    broker: &[BrokerPosition],
    internal: &[InternalPosition],
    threshold_dollars: Decimal,
) -> ReconciliationReport {
    let mut report = ReconciliationReport::default();
    let mut seen_internal: std::collections::HashSet<String> = Default::default();
    for b in broker {
        let b_value = b.qty * b.mark_price;
        report.broker_total += b_value;
        let internal_match = internal.iter().find(|i| i.symbol == b.symbol);
        match internal_match {
            None => {
                report.mismatches.push(Mismatch {
                    symbol: b.symbol.clone(),
                    broker_qty: b.qty,
                    internal_qty: Decimal::ZERO,
                    broker_value: b_value,
                    internal_value: Decimal::ZERO,
                    difference: b_value,
                    kind: MismatchKind::MissingInternal,
                });
            }
            Some(i) => {
                seen_internal.insert(i.symbol.clone());
                let i_value = i.qty * i.mark_price;
                let diff = b_value - i_value;
                if diff.abs() > threshold_dollars {
                    let kind = if b.qty != i.qty { MismatchKind::QtyDiffers }
                        else if b.mark_price != i.mark_price { MismatchKind::PriceDiffers }
                        else { MismatchKind::ValueDiffers };
                    report.mismatches.push(Mismatch {
                        symbol: b.symbol.clone(),
                        broker_qty: b.qty,
                        internal_qty: i.qty,
                        broker_value: b_value,
                        internal_value: i_value,
                        difference: diff,
                        kind,
                    });
                } else {
                    report.matched_count += 1;
                }
            }
        }
    }
    for i in internal {
        let i_value = i.qty * i.mark_price;
        report.internal_total += i_value;
        if !seen_internal.contains(&i.symbol) {
            report.mismatches.push(Mismatch {
                symbol: i.symbol.clone(),
                broker_qty: Decimal::ZERO,
                internal_qty: i.qty,
                broker_value: Decimal::ZERO,
                internal_value: i_value,
                difference: -i_value,
                kind: MismatchKind::MissingBroker,
            });
        }
    }
    report.total_difference = report.broker_total - report.internal_total;
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn d(s: &str) -> Decimal { Decimal::from_str(s).unwrap() }
    fn bp(sym: &str, q: &str, p: &str) -> BrokerPosition {
        BrokerPosition { symbol: sym.into(), qty: d(q), mark_price: d(p) }
    }
    fn ip(sym: &str, q: &str, p: &str) -> InternalPosition {
        InternalPosition { symbol: sym.into(), qty: d(q), mark_price: d(p) }
    }

    #[test]
    fn empty_reconciles_to_zero() {
        let r = reconcile(&[], &[], d("0.01"));
        assert!(r.mismatches.is_empty());
        assert_eq!(r.broker_total, Decimal::ZERO);
    }

    #[test]
    fn matched_positions_no_mismatches() {
        let r = reconcile(
            &[bp("AAPL", "100", "150")],
            &[ip("AAPL", "100", "150")],
            d("0.01"),
        );
        assert!(r.mismatches.is_empty());
        assert_eq!(r.matched_count, 1);
    }

    #[test]
    fn qty_mismatch_flagged() {
        let r = reconcile(
            &[bp("AAPL", "100", "150")],
            &[ip("AAPL", "90", "150")],
            d("0.01"),
        );
        assert_eq!(r.mismatches.len(), 1);
        assert_eq!(r.mismatches[0].kind, MismatchKind::QtyDiffers);
        assert_eq!(r.mismatches[0].difference, d("1500"));
    }

    #[test]
    fn price_mismatch_flagged() {
        let r = reconcile(
            &[bp("AAPL", "100", "150")],
            &[ip("AAPL", "100", "152")],
            d("0.01"),
        );
        assert_eq!(r.mismatches[0].kind, MismatchKind::PriceDiffers);
    }

    #[test]
    fn missing_internal_flagged() {
        let r = reconcile(
            &[bp("AAPL", "100", "150")],
            &[],
            d("0.01"),
        );
        assert_eq!(r.mismatches[0].kind, MismatchKind::MissingInternal);
    }

    #[test]
    fn missing_broker_flagged() {
        let r = reconcile(
            &[],
            &[ip("AAPL", "100", "150")],
            d("0.01"),
        );
        assert_eq!(r.mismatches[0].kind, MismatchKind::MissingBroker);
    }

    #[test]
    fn small_difference_under_threshold_not_flagged() {
        // Difference $1 with threshold $5.
        let r = reconcile(
            &[bp("AAPL", "100", "150.01")],
            &[ip("AAPL", "100", "150.02")],
            d("5"),
        );
        assert!(r.mismatches.is_empty());
        assert_eq!(r.matched_count, 1);
    }

    #[test]
    fn total_difference_signed() {
        let r = reconcile(
            &[bp("AAPL", "100", "150")],         // $15,000 broker
            &[ip("AAPL", "100", "140")],         // $14,000 internal
            d("0.01"),
        );
        assert_eq!(r.total_difference, d("1000"));
    }

    #[test]
    fn multi_position_partial_reconciliation() {
        let r = reconcile(
            &[bp("AAPL", "100", "150"), bp("TSLA", "50", "200")],
            &[ip("AAPL", "100", "150"), ip("MSFT", "30", "300")],
            d("0.01"),
        );
        // TSLA missing internal + MSFT missing broker = 2 mismatches.
        assert_eq!(r.mismatches.len(), 2);
        assert_eq!(r.matched_count, 1);    // AAPL only
    }
}
