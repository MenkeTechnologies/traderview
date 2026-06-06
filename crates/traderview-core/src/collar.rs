//! Protective collar P&L — long stock + long put + short call.
//!
//! Classic hedging structure: own the underlying, buy a put for
//! downside protection, finance it by selling a call (often "zero-cost"
//! when call premium = put premium).
//!
//! Construction:
//!   - long N shares @ stock_basis
//!   - long N puts @ put_strike (lower)
//!   - short N calls @ call_strike (upper)
//!
//! Net debit = put_premium − call_premium (negative if collar earns
//! credit on the option legs).
//!
//! P&L at expiration:
//!   pnl(S) = N · (S − stock_basis)                  (stock leg)
//!          + N · max(put_strike − S, 0)              (put protection)
//!          − N · max(S − call_strike, 0)             (call cap)
//!          − N · net_premium_debit                   (option-leg cost)
//!
//! Caps both upside and downside:
//!   - Max gain: capped at (call_strike − stock_basis − net_debit)·N
//!   - Max loss: floored at (put_strike − stock_basis − net_debit)·N
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Collar {
    pub stock_basis: f64,
    pub put_strike: f64,
    pub call_strike: f64,
    /// put_premium − call_premium, positive for net debit.
    pub net_option_debit_per_share: f64,
    pub shares: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CollarReport {
    pub max_profit: f64,
    pub max_loss: f64,
    pub upside_cap_at_call_strike: f64,
    pub downside_floor_at_put_strike: f64,
    pub breakeven_spot: f64,
}

pub fn analyze(c: &Collar) -> Option<CollarReport> {
    if !c.stock_basis.is_finite()
        || c.stock_basis <= 0.0
        || !c.put_strike.is_finite()
        || c.put_strike <= 0.0
        || !c.call_strike.is_finite()
        || c.call_strike <= 0.0
        || !c.net_option_debit_per_share.is_finite()
        || !c.shares.is_finite()
        || c.shares == 0.0
        || c.put_strike >= c.call_strike
    {
        return None;
    }
    // Above call strike: stock capped, pnl_per_share = (call_strike − basis) − net_debit
    // Below put strike: stock floored, pnl_per_share = (put_strike − basis) − net_debit
    let upside_cap = (c.call_strike - c.stock_basis - c.net_option_debit_per_share) * c.shares;
    let downside_floor = (c.put_strike - c.stock_basis - c.net_option_debit_per_share) * c.shares;
    // Stock-leg P&L crosses zero at S = basis + net_debit (when between strikes).
    let breakeven = c.stock_basis + c.net_option_debit_per_share;
    Some(CollarReport {
        max_profit: upside_cap,
        max_loss: downside_floor,
        upside_cap_at_call_strike: upside_cap,
        downside_floor_at_put_strike: downside_floor,
        breakeven_spot: breakeven,
    })
}

pub fn pnl_at_expiration(c: &Collar, spot: f64) -> Option<f64> {
    if !spot.is_finite()
        || !c.stock_basis.is_finite()
        || !c.put_strike.is_finite()
        || !c.call_strike.is_finite()
        || !c.net_option_debit_per_share.is_finite()
    {
        return None;
    }
    let stock_pnl = spot - c.stock_basis;
    let put_payoff = (c.put_strike - spot).max(0.0);
    let call_payoff = (spot - c.call_strike).max(0.0);
    let per_share = stock_pnl + put_payoff - call_payoff - c.net_option_debit_per_share;
    Some(per_share * c.shares)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn collar() -> Collar {
        // Bought stock at 100, protect with 95 put, cap with 110 call,
        // net zero-cost collar (put premium == call premium).
        Collar {
            stock_basis: 100.0,
            put_strike: 95.0,
            call_strike: 110.0,
            net_option_debit_per_share: 0.0,
            shares: 100.0,
        }
    }

    #[test]
    fn invalid_inputs_return_none() {
        let mut bad = collar();
        bad.stock_basis = 0.0;
        assert!(analyze(&bad).is_none());
        let mut bad = collar();
        bad.shares = 0.0;
        assert!(analyze(&bad).is_none());
        let mut bad = collar();
        bad.put_strike = 120.0; // put > call
        assert!(analyze(&bad).is_none());
        let mut bad = collar();
        bad.net_option_debit_per_share = f64::NAN;
        assert!(analyze(&bad).is_none());
    }

    #[test]
    fn zero_cost_collar_max_profit_equals_call_minus_basis() {
        let r = analyze(&collar()).unwrap();
        assert!((r.max_profit - 1_000.0).abs() < 1e-9); // (110-100)·100
        assert!((r.max_loss + 500.0).abs() < 1e-9); // (95-100)·100
    }

    #[test]
    fn pnl_above_call_strike_capped() {
        // Spot = 130 → pnl = (110 − 100 − 0) · 100 = 1000.
        let p = pnl_at_expiration(&collar(), 130.0).unwrap();
        assert!((p - 1_000.0).abs() < 1e-9);
    }

    #[test]
    fn pnl_below_put_strike_floored() {
        let p = pnl_at_expiration(&collar(), 80.0).unwrap();
        assert!((p + 500.0).abs() < 1e-9);
    }

    #[test]
    fn pnl_between_strikes_linear_in_spot() {
        // Spot = 105 → pnl = (105 − 100) · 100 = 500.
        let p = pnl_at_expiration(&collar(), 105.0).unwrap();
        assert!((p - 500.0).abs() < 1e-9);
    }

    #[test]
    fn breakeven_at_basis_when_zero_cost() {
        let r = analyze(&collar()).unwrap();
        assert!((r.breakeven_spot - 100.0).abs() < 1e-9);
        let p = pnl_at_expiration(&collar(), r.breakeven_spot).unwrap();
        assert!(p.abs() < 1e-9);
    }

    #[test]
    fn debit_collar_shifts_breakeven_higher() {
        let mut debit_collar = collar();
        debit_collar.net_option_debit_per_share = 0.50;
        let r = analyze(&debit_collar).unwrap();
        assert!((r.breakeven_spot - 100.50).abs() < 1e-9);
        let p = pnl_at_expiration(&debit_collar, 100.0).unwrap();
        // PnL at basis = (100 − 100) − 0.50 = −0.50 · 100 = −50.
        assert!((p + 50.0).abs() < 1e-9);
    }

    #[test]
    fn credit_collar_shifts_breakeven_lower() {
        let mut credit_collar = collar();
        credit_collar.net_option_debit_per_share = -0.75;
        let r = analyze(&credit_collar).unwrap();
        assert!((r.breakeven_spot - 99.25).abs() < 1e-9);
    }

    #[test]
    fn short_position_inverts_pnl() {
        // Negative shares = short stock + collar (synthetic short collar).
        let mut short = collar();
        short.shares = -100.0;
        let p_long = pnl_at_expiration(&collar(), 105.0).unwrap();
        let p_short = pnl_at_expiration(&short, 105.0).unwrap();
        assert!((p_long + p_short).abs() < 1e-9);
    }
}
