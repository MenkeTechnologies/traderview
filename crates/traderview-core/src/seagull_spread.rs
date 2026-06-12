//! Seagull spread — three-legged collar-style structure.
//!
//! Bullish seagull at K1 < K2 < K3:
//!     − short put(K1)        (finances the spread)
//!     + long  call(K2)
//!     − short call(K3)
//!
//!   P/L(S) = max(S−K2,0) − max(S−K3,0) − max(K1−S,0) − net_debit
//!
//! Often structured zero-cost: upside participation between K2 and K3
//! paid for by taking on downside below K1. Max profit = (K3 − K2) −
//! net_debit above K3; losses grow one-for-one below K1 (bounded only
//! by S → 0).
//!
//! Pure compute. Companion to `collar`, `risk_reversal_25_delta_butterfly`,
//! `butterfly_spread`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SeagullReport {
    pub put_strike: f64,
    pub call_low_strike: f64,
    pub call_high_strike: f64,
    /// call_low − call_high − put premiums (positive = debit paid).
    pub net_debit: f64,
    pub is_zero_cost: bool,
    /// (K3 − K2) − net_debit, attained for S ≥ K3.
    pub max_profit: f64,
    /// Loss at S = 0: K1 + net_debit (reported positive).
    pub max_loss_at_zero: f64,
    /// Price where downside P/L crosses zero: K1 + net_debit. None when
    /// the structure is a net debit — then it is already underwater in
    /// the flat K1..K2 zone and only breaks even on the upside.
    pub downside_breakeven: Option<f64>,
    /// Upside breakeven (only meaningful when net_debit > 0).
    pub upside_breakeven: Option<f64>,
    /// P/L sampled at the strikes and beyond: (price, pnl).
    pub payoff_points: Vec<(f64, f64)>,
}

/// P/L per share at expiry price `s`.
fn pnl_at(s: f64, k1: f64, k2: f64, k3: f64, net_debit: f64) -> f64 {
    (s - k2).max(0.0) - (s - k3).max(0.0) - (k1 - s).max(0.0) - net_debit
}

pub fn compute(
    put_strike: f64,
    call_low_strike: f64,
    call_high_strike: f64,
    put_price: f64,
    call_low_price: f64,
    call_high_price: f64,
) -> Option<SeagullReport> {
    let valid = [put_strike, call_low_strike, call_high_strike]
        .iter()
        .all(|k| k.is_finite() && *k > 0.0)
        && put_strike < call_low_strike
        && call_low_strike < call_high_strike
        && [put_price, call_low_price, call_high_price]
            .iter()
            .all(|p| p.is_finite() && *p >= 0.0);
    if !valid {
        return None;
    }
    let (k1, k2, k3) = (put_strike, call_low_strike, call_high_strike);
    let net_debit = call_low_price - call_high_price - put_price;
    let max_profit = (k3 - k2) - net_debit;
    // Below K1: P/L = S − K1 − net_debit, so the zero crossing sits at
    // K1 + net_debit — and only exists for zero-cost or net-credit
    // structures (a net debit is already underwater in the flat zone).
    let downside_breakeven = (net_debit <= 0.0).then_some(k1 + net_debit);
    let upside_breakeven = (net_debit > 0.0).then_some(k2 + net_debit);
    let sample = [0.0, k1 * 0.5, k1, (k1 + k2) / 2.0, k2, (k2 + k3) / 2.0, k3, k3 * 1.25];
    let payoff_points = sample
        .iter()
        .map(|s| (*s, pnl_at(*s, k1, k2, k3, net_debit)))
        .collect();
    Some(SeagullReport {
        put_strike: k1,
        call_low_strike: k2,
        call_high_strike: k3,
        net_debit,
        is_zero_cost: net_debit.abs() < 1e-9,
        max_profit,
        max_loss_at_zero: k1 + net_debit,
        downside_breakeven,
        upside_breakeven,
        payoff_points,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_cost_seagull_profile() {
        // Put credit 2 exactly funds call spread debit 2 (3 − 1).
        let r = compute(90.0, 100.0, 110.0, 2.0, 3.0, 1.0).unwrap();
        assert!(r.is_zero_cost);
        assert!((r.net_debit).abs() < 1e-12);
        // Max profit = full 10-wide call spread.
        assert!((r.max_profit - 10.0).abs() < 1e-12);
        // At S=0 the short put costs the full strike.
        assert!((r.max_loss_at_zero - 90.0).abs() < 1e-12);
        // Flat zero P/L between K1 and K2; loses below K1.
        assert_eq!(r.downside_breakeven, Some(90.0));
        assert_eq!(r.upside_breakeven, None);
    }

    #[test]
    fn payoff_points_match_hand_walk() {
        let r = compute(90.0, 100.0, 110.0, 2.0, 3.0, 1.0).unwrap();
        let at = |price: f64| {
            r.payoff_points
                .iter()
                .find(|(s, _)| (*s - price).abs() < 1e-9)
                .map(|(_, p)| *p)
                .expect("sample point")
        };
        assert!((at(0.0) + 90.0).abs() < 1e-12); // −K1
        assert!((at(90.0) - 0.0).abs() < 1e-12); // short put expires ATM
        assert!((at(100.0) - 0.0).abs() < 1e-12); // flat zone
        assert!((at(105.0) - 5.0).abs() < 1e-12); // mid call spread
        assert!((at(110.0) - 10.0).abs() < 1e-12); // capped
        assert!((at(137.5) - 10.0).abs() < 1e-12); // stays capped
    }

    #[test]
    fn net_credit_moves_downside_breakeven_below_put_strike() {
        // Put credit 4 vs call spread debit 2 → net credit 2.
        let r = compute(90.0, 100.0, 110.0, 4.0, 3.0, 1.0).unwrap();
        assert!((r.net_debit + 2.0).abs() < 1e-12);
        // P/L at K1 = +2; crosses zero at K1 − 2 = 88.
        assert_eq!(r.downside_breakeven, Some(88.0));
        assert!((r.max_profit - 12.0).abs() < 1e-12);
    }

    #[test]
    fn net_debit_creates_upside_breakeven() {
        // Put credit 1 vs call spread debit 2 → net debit 1.
        let r = compute(90.0, 100.0, 110.0, 1.0, 3.0, 1.0).unwrap();
        assert!((r.net_debit - 1.0).abs() < 1e-12);
        assert_eq!(r.upside_breakeven, Some(101.0));
        assert_eq!(r.downside_breakeven, None);
        assert!((r.max_profit - 9.0).abs() < 1e-12);
    }

    #[test]
    fn hostile_inputs_return_none() {
        // Strike ordering violated.
        assert!(compute(100.0, 90.0, 110.0, 2.0, 3.0, 1.0).is_none());
        assert!(compute(90.0, 100.0, 100.0, 2.0, 3.0, 1.0).is_none());
        // Negative or non-finite prices.
        assert!(compute(90.0, 100.0, 110.0, -2.0, 3.0, 1.0).is_none());
        assert!(compute(90.0, 100.0, 110.0, f64::NAN, 3.0, 1.0).is_none());
        assert!(compute(0.0, 100.0, 110.0, 2.0, 3.0, 1.0).is_none());
    }
}
