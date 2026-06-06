//! Iron Condor P&L analyzer.
//!
//! Iron condor = bear call spread + bull put spread on the same expiry,
//! collected as a net credit. Four legs:
//!   - long  put @ `put_long_strike` (wing)
//!   - short put @ `put_short_strike` (closer to money)
//!   - short call @ `call_short_strike` (closer to money)
//!   - long  call @ `call_long_strike` (wing)
//!
//! Constraint at construction time:
//!   put_long_strike < put_short_strike < call_short_strike < call_long_strike
//!
//! At expiration:
//!   pnl(S) =  net_credit
//!           − max(0, put_short_strike − S) · contracts
//!           + max(0, put_long_strike − S) · contracts
//!           − max(0, S − call_short_strike) · contracts
//!           + max(0, S − call_long_strike) · contracts
//!
//! Returns the standard summary: max profit (credit), max loss (wing
//! width minus credit), two breakevens.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct IronCondor {
    pub put_long_strike: f64,
    pub put_short_strike: f64,
    pub call_short_strike: f64,
    pub call_long_strike: f64,
    pub net_credit_per_contract: f64, // positive number (credit received)
    pub contracts: i64,
    pub multiplier: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IronCondorReport {
    pub max_profit: f64,
    pub max_loss: f64,
    pub lower_breakeven: f64,
    pub upper_breakeven: f64,
    pub profit_zone_width: f64,
    pub credit_to_max_loss_ratio: f64,
}

pub fn analyze(ic: &IronCondor) -> Option<IronCondorReport> {
    if !ic.put_long_strike.is_finite()
        || !ic.put_short_strike.is_finite()
        || !ic.call_short_strike.is_finite()
        || !ic.call_long_strike.is_finite()
        || !ic.net_credit_per_contract.is_finite()
        || !ic.multiplier.is_finite()
    {
        return None;
    }
    if !(ic.put_long_strike < ic.put_short_strike
        && ic.put_short_strike < ic.call_short_strike
        && ic.call_short_strike < ic.call_long_strike)
    {
        return None;
    }
    if ic.net_credit_per_contract < 0.0 || ic.multiplier <= 0.0 || ic.contracts == 0 {
        return None;
    }
    let scale = ic.contracts.unsigned_abs() as f64 * ic.multiplier;
    let put_width = ic.put_short_strike - ic.put_long_strike;
    let call_width = ic.call_long_strike - ic.call_short_strike;
    let wing_width = put_width.max(call_width); // worst-case wing
    let max_profit_per = ic.net_credit_per_contract;
    let max_loss_per = wing_width - ic.net_credit_per_contract;
    if max_loss_per < 0.0 {
        return None; // sanity: credit exceeds wing width → mispriced
    }
    let lower_be = ic.put_short_strike - ic.net_credit_per_contract;
    let upper_be = ic.call_short_strike + ic.net_credit_per_contract;
    let sign = ic.contracts.signum() as f64;
    let max_profit = max_profit_per * scale * sign;
    let max_loss = -max_loss_per * scale * sign;
    Some(IronCondorReport {
        max_profit,
        max_loss,
        lower_breakeven: lower_be,
        upper_breakeven: upper_be,
        profit_zone_width: upper_be - lower_be,
        credit_to_max_loss_ratio: if max_loss_per > 0.0 {
            ic.net_credit_per_contract / max_loss_per
        } else {
            f64::INFINITY
        },
    })
}

/// P&L at expiration for a single underlying spot, scaled by contracts.
pub fn pnl_at_expiration(ic: &IronCondor, spot: f64) -> Option<f64> {
    if !spot.is_finite()
        || !ic.net_credit_per_contract.is_finite()
        || !ic.put_long_strike.is_finite()
        || !ic.put_short_strike.is_finite()
        || !ic.call_short_strike.is_finite()
        || !ic.call_long_strike.is_finite()
    {
        return None;
    }
    let credit = ic.net_credit_per_contract;
    let short_put = (ic.put_short_strike - spot).max(0.0);
    let long_put = (ic.put_long_strike - spot).max(0.0);
    let short_call = (spot - ic.call_short_strike).max(0.0);
    let long_call = (spot - ic.call_long_strike).max(0.0);
    let per = credit - short_put + long_put - short_call + long_call;
    let scale = ic.contracts.unsigned_abs() as f64 * ic.multiplier;
    let sign = ic.contracts.signum() as f64;
    Some(per * scale * sign)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ic() -> IronCondor {
        IronCondor {
            put_long_strike: 90.0,
            put_short_strike: 95.0,
            call_short_strike: 105.0,
            call_long_strike: 110.0,
            net_credit_per_contract: 2.0,
            contracts: 1,
            multiplier: 100.0,
        }
    }

    #[test]
    fn invalid_strike_order_returns_none() {
        let mut bad = ic();
        std::mem::swap(&mut bad.put_short_strike, &mut bad.call_short_strike);
        assert!(analyze(&bad).is_none());
    }

    #[test]
    fn zero_contracts_returns_none() {
        let mut bad = ic();
        bad.contracts = 0;
        assert!(analyze(&bad).is_none());
    }

    #[test]
    fn negative_credit_returns_none() {
        let mut bad = ic();
        bad.net_credit_per_contract = -1.0;
        assert!(analyze(&bad).is_none());
    }

    #[test]
    fn credit_exceeding_wing_returns_none() {
        let mut bad = ic();
        bad.net_credit_per_contract = 100.0; // > 5 wing width
        assert!(analyze(&bad).is_none());
    }

    #[test]
    fn nan_inputs_return_none() {
        let mut bad = ic();
        bad.put_long_strike = f64::NAN;
        assert!(analyze(&bad).is_none());
    }

    #[test]
    fn classic_5_wide_condor_with_2_credit() {
        // Max profit = $2/contract × 100 = $200.
        // Max loss = (5 − 2) × 100 = $300.
        let r = analyze(&ic()).unwrap();
        assert!((r.max_profit - 200.0).abs() < 1e-9);
        assert!((r.max_loss + 300.0).abs() < 1e-9);
        assert!((r.lower_breakeven - 93.0).abs() < 1e-9);
        assert!((r.upper_breakeven - 107.0).abs() < 1e-9);
        assert!((r.profit_zone_width - 14.0).abs() < 1e-9);
    }

    #[test]
    fn pnl_at_short_call_breakeven_is_zero() {
        let r = pnl_at_expiration(&ic(), 107.0).unwrap();
        assert!(r.abs() < 1e-9);
    }

    #[test]
    fn pnl_at_short_put_breakeven_is_zero() {
        let r = pnl_at_expiration(&ic(), 93.0).unwrap();
        assert!(r.abs() < 1e-9);
    }

    #[test]
    fn pnl_inside_profit_zone_equals_max_profit() {
        // At spot = 100 (inside), no leg has intrinsic → pnl = credit × scale.
        let r = pnl_at_expiration(&ic(), 100.0).unwrap();
        assert!((r - 200.0).abs() < 1e-9);
    }

    #[test]
    fn pnl_beyond_long_wing_equals_max_loss() {
        // At spot = 85 (below put_long_strike): all puts at intrinsic.
        // long_put = 5, short_put = 10 → −10 + 5 + 2 = −3 per contract → −300.
        let r = pnl_at_expiration(&ic(), 85.0).unwrap();
        assert!((r + 300.0).abs() < 1e-9);
        let r = pnl_at_expiration(&ic(), 115.0).unwrap();
        assert!((r + 300.0).abs() < 1e-9);
    }

    #[test]
    fn negative_contracts_inverts_pnl() {
        let mut short = ic();
        short.contracts = -1;
        let r_long = analyze(&ic()).unwrap();
        let r_short = analyze(&short).unwrap();
        assert!((r_long.max_profit + r_short.max_profit).abs() < 1e-9);
    }
}
