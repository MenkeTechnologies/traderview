//! Multi-leg option spread validation + premium math.
//!
//! A spread is 2–4 OCC legs on ONE underlying, each with a side and an
//! integer ratio (vertical = 1:1, ratio spread = 1:2, iron condor =
//! four 1s). Validation and net-premium arithmetic are pure; the paper
//! engine owns fills.
//!
//! Sign convention: buys pay premium (negative cash), sells collect
//! (positive). net_premium > 0 = net CREDIT, < 0 = net DEBIT — stated
//! per share; the caller scales by the 100× multiplier and spread qty.

use crate::occ_symbol;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SpreadLeg {
    /// OCC symbol.
    pub symbol: String,
    /// true = buy (pay), false = sell (collect).
    pub buy: bool,
    /// Contracts of this leg per 1 unit of the spread.
    pub ratio: u32,
}

pub const MIN_LEGS: usize = 2;
pub const MAX_LEGS: usize = 4;

/// Structural validation: leg count, parseable OCC symbols, one
/// underlying, positive ratios, no duplicate (symbol, side) legs.
pub fn validate(legs: &[SpreadLeg]) -> Result<(), String> {
    if !(MIN_LEGS..=MAX_LEGS).contains(&legs.len()) {
        return Err(format!("spreads take {MIN_LEGS}..={MAX_LEGS} legs, got {}", legs.len()));
    }
    let mut underlying: Option<String> = None;
    let mut seen = std::collections::HashSet::new();
    for leg in legs {
        let occ = occ_symbol::parse(&leg.symbol)
            .ok_or_else(|| format!("{} is not an OCC option symbol", leg.symbol))?;
        if leg.ratio == 0 {
            return Err(format!("{}: ratio must be >= 1", leg.symbol));
        }
        if !seen.insert((leg.symbol.clone(), leg.buy)) {
            return Err(format!("duplicate leg {} (same symbol and side)", leg.symbol));
        }
        match &underlying {
            None => underlying = Some(occ.underlying),
            Some(u) if *u != occ.underlying => {
                return Err(format!(
                    "legs span underlyings {} and {} — that's two trades, not a spread",
                    u, occ.underlying
                ));
            }
            _ => {}
        }
    }
    Ok(())
}

/// Per-share net premium of the spread at the given leg prices
/// (aligned with `legs`). Positive = credit collected.
pub fn net_premium(legs: &[SpreadLeg], prices: &[f64]) -> Option<f64> {
    if legs.len() != prices.len() || prices.iter().any(|p| !p.is_finite() || *p <= 0.0) {
        return None;
    }
    Some(
        legs.iter()
            .zip(prices)
            .map(|(l, p)| {
                let sign = if l.buy { -1.0 } else { 1.0 };
                sign * p * l.ratio as f64
            })
            .sum(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn leg(symbol: &str, buy: bool, ratio: u32) -> SpreadLeg {
        SpreadLeg { symbol: symbol.into(), buy, ratio }
    }

    #[test]
    fn vertical_debit_and_condor_credit_pin_the_sign_convention() {
        // Bull call spread: buy 190C at 5.00, sell 200C at 2.20 → 2.80 debit.
        let v = vec![
            leg("AAPL260117C00190000", true, 1),
            leg("AAPL260117C00200000", false, 1),
        ];
        let p = net_premium(&v, &[5.0, 2.2]).unwrap();
        assert!((p + 2.8).abs() < 1e-9, "debit is NEGATIVE, got {p}");
        // Iron condor: sell put+call body, buy wings → net credit.
        let ic = vec![
            leg("SPY260320P00540000", true, 1),
            leg("SPY260320P00550000", false, 1),
            leg("SPY260320C00600000", false, 1),
            leg("SPY260320C00610000", true, 1),
        ];
        let p = net_premium(&ic, &[1.0, 2.5, 2.4, 0.9]).unwrap();
        assert!((p - 3.0).abs() < 1e-9, "condor credit 3.00, got {p}");
        // Ratio spread: buy 1, sell 2.
        let r = vec![
            leg("AAPL260117C00190000", true, 1),
            leg("AAPL260117C00200000", false, 2),
        ];
        let p = net_premium(&r, &[5.0, 2.2]).unwrap();
        assert!((p + 0.6).abs() < 1e-9); // -5.0 + 4.4
    }

    #[test]
    fn validation_pins_structure_rules() {
        let v = vec![
            leg("AAPL260117C00190000", true, 1),
            leg("AAPL260117C00200000", false, 1),
        ];
        assert!(validate(&v).is_ok());
        // One leg is an order, not a spread; five is beyond scope.
        assert!(validate(&v[..1]).is_err());
        let five: Vec<SpreadLeg> = (0..5)
            .map(|i| leg(&format!("AAPL260117C0019{i}000"), true, 1))
            .collect();
        assert!(validate(&five).is_err());
        // Cross-underlying is two trades.
        let cross = vec![
            leg("AAPL260117C00190000", true, 1),
            leg("MSFT260117C00400000", false, 1),
        ];
        assert!(validate(&cross).unwrap_err().contains("underlyings"));
        // Equity ticker leg, zero ratio, duplicate leg.
        assert!(validate(&[leg("AAPL", true, 1), v[1].clone()]).is_err());
        assert!(validate(&[leg("AAPL260117C00190000", true, 0), v[1].clone()]).is_err());
        assert!(validate(&[v[0].clone(), v[0].clone()]).is_err());
    }

    #[test]
    fn premium_refuses_bad_prices_and_misaligned_lengths() {
        let v = vec![
            leg("AAPL260117C00190000", true, 1),
            leg("AAPL260117C00200000", false, 1),
        ];
        assert!(net_premium(&v, &[5.0]).is_none());
        assert!(net_premium(&v, &[5.0, 0.0]).is_none());
        assert!(net_premium(&v, &[5.0, f64::NAN]).is_none());
    }
}
