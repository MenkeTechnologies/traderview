//! Currency-exposure aggregator.
//!
//! Aggregates per-position notional by underlying CURRENCY, not just
//! by symbol. A trader holding $50k AAPL + €30k SAP + ¥1,000,000 SONY
//! is effectively long USD + EUR + JPY against their home currency.
//!
//! Converts each position to home-currency-equivalent gross exposure
//! using caller-supplied FX rates, then sums per currency.
//!
//! Pure compute. FX rates passed in — engine does not embed live data.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForeignPosition {
    pub symbol: String,
    /// 3-letter currency code (USD, EUR, GBP, JPY, ...).
    pub currency: String,
    /// Notional in the position's NATIVE currency.
    pub notional_native: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyBucket {
    pub currency: String,
    pub gross_native: f64,
    pub gross_home: f64,
    pub net_native: f64,
    pub net_home: f64,
    pub position_count: usize,
    /// Share of total home-currency gross exposure.
    pub pct_of_total: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CurrencyReport {
    pub home_currency: String,
    pub total_gross_home: f64,
    pub total_net_home: f64,
    pub buckets: Vec<CurrencyBucket>,
    /// Currencies > 25% of home-gross — concentration flag.
    pub overweight_currencies: Vec<String>,
}

pub fn analyze(
    positions: &[ForeignPosition],
    home_currency: &str,
    fx_to_home: &BTreeMap<String, f64>,
) -> CurrencyReport {
    let mut report = CurrencyReport {
        home_currency: home_currency.into(),
        ..Default::default()
    };
    if positions.is_empty() {
        return report;
    }
    let mut by_ccy: BTreeMap<String, (f64, f64, usize)> = BTreeMap::new(); // (gross, net, count)
    for p in positions {
        let entry = by_ccy.entry(p.currency.clone()).or_insert((0.0, 0.0, 0));
        entry.0 += p.notional_native.abs();
        entry.1 += p.notional_native;
        entry.2 += 1;
    }
    let mut total_gross_home = 0.0;
    let mut total_net_home = 0.0;
    for (ccy, (gross_n, net_n, _count)) in &by_ccy {
        let rate = if ccy == home_currency {
            1.0
        } else {
            *fx_to_home.get(ccy).unwrap_or(&0.0)
        };
        total_gross_home += gross_n * rate;
        total_net_home += net_n * rate;
    }
    report.total_gross_home = total_gross_home;
    report.total_net_home = total_net_home;
    for (ccy, (gross_n, net_n, count)) in by_ccy {
        let rate = if ccy == home_currency {
            1.0
        } else {
            *fx_to_home.get(&ccy).unwrap_or(&0.0)
        };
        let gross_home = gross_n * rate;
        let net_home = net_n * rate;
        let pct = if total_gross_home > 0.0 {
            gross_home / total_gross_home
        } else {
            0.0
        };
        if pct > 0.25 && ccy != home_currency {
            report.overweight_currencies.push(ccy.clone());
        }
        report.buckets.push(CurrencyBucket {
            currency: ccy,
            gross_native: gross_n,
            gross_home,
            net_native: net_n,
            net_home,
            position_count: count,
            pct_of_total: pct,
        });
    }
    // Sort by home-gross descending.
    report.buckets.sort_by(|a, b| {
        b.gross_home
            .partial_cmp(&a.gross_home)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    report.overweight_currencies.sort();
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(sym: &str, ccy: &str, n: f64) -> ForeignPosition {
        ForeignPosition {
            symbol: sym.into(),
            currency: ccy.into(),
            notional_native: n,
        }
    }
    fn rates() -> BTreeMap<String, f64> {
        // EUR = 1.10 USD, GBP = 1.27 USD, JPY = 0.0064 USD.
        let mut m = BTreeMap::new();
        m.insert("EUR".into(), 1.10);
        m.insert("GBP".into(), 1.27);
        m.insert("JPY".into(), 0.0064);
        m
    }

    #[test]
    fn empty_returns_default() {
        let r = analyze(&[], "USD", &rates());
        assert!(r.buckets.is_empty());
    }

    #[test]
    fn home_currency_uses_rate_one() {
        let r = analyze(&[p("AAPL", "USD", 10_000.0)], "USD", &rates());
        let usd = r.buckets.iter().find(|b| b.currency == "USD").unwrap();
        assert_eq!(usd.gross_home, 10_000.0);
    }

    #[test]
    fn foreign_currency_uses_supplied_fx_rate() {
        // €10k × 1.10 = $11k.
        let r = analyze(&[p("SAP", "EUR", 10_000.0)], "USD", &rates());
        let eur = r.buckets.iter().find(|b| b.currency == "EUR").unwrap();
        assert_eq!(eur.gross_home, 11_000.0);
    }

    #[test]
    fn missing_fx_rate_falls_back_to_zero() {
        // CAD not in rates table → treated as 0 (defensive).
        let r = analyze(&[p("RY", "CAD", 10_000.0)], "USD", &rates());
        let cad = r.buckets.iter().find(|b| b.currency == "CAD").unwrap();
        assert_eq!(cad.gross_home, 0.0);
    }

    #[test]
    fn overweight_flagged_above_25_pct() {
        // EUR position dominates total.
        let positions = vec![p("AAPL", "USD", 1_000.0), p("SAP", "EUR", 10_000.0)];
        let r = analyze(&positions, "USD", &rates());
        assert!(r.overweight_currencies.contains(&"EUR".to_string()));
    }

    #[test]
    fn home_currency_never_marked_overweight() {
        // Even if USD = 100% of gross, home currency itself isn't a risk.
        let r = analyze(&[p("AAPL", "USD", 100_000.0)], "USD", &rates());
        assert!(r.overweight_currencies.is_empty());
    }

    #[test]
    fn short_position_reduces_net_keeps_gross() {
        let positions = vec![p("AAPL", "USD", 10_000.0), p("SHORT_BOND", "USD", -5_000.0)];
        let r = analyze(&positions, "USD", &rates());
        let usd = r.buckets.iter().find(|b| b.currency == "USD").unwrap();
        assert_eq!(usd.gross_home, 15_000.0);
        assert_eq!(usd.net_home, 5_000.0);
    }

    #[test]
    fn buckets_sorted_largest_gross_first() {
        let positions = vec![
            p("SMALL", "GBP", 1_000.0),
            p("BIG", "USD", 100_000.0),
            p("MID", "EUR", 10_000.0),
        ];
        let r = analyze(&positions, "USD", &rates());
        assert_eq!(r.buckets[0].currency, "USD");
        // GBP 1k × 1.27 = $1270 vs EUR 10k × 1.10 = $11000 → EUR is bigger.
        assert_eq!(r.buckets[1].currency, "EUR");
        assert_eq!(r.buckets[2].currency, "GBP");
    }
}
