//! Iron Butterfly P&L — short straddle + long wings.
//!
//! Four legs at expiry T:
//!   - long  put  @ put_long_strike (lower wing)
//!   - short put  @ body_strike (ATM short)
//!   - short call @ body_strike (ATM short)
//!   - long  call @ call_long_strike (upper wing)
//!
//! Constraint: put_long_strike < body_strike < call_long_strike, with
//! equal-width wings (body − put_long = call_long − body).
//!
//! Net credit construction (the short straddle premium exceeds the
//! long wing premium). Max profit = credit (at exactly the body
//! strike). Max loss = wing width − credit (at either long wing).
//!
//! Pure compute. Distinct from `iron_condor` (which has separate
//! short call and short put strikes).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct IronButterfly {
    pub put_long_strike: f64,
    pub body_strike: f64,
    pub call_long_strike: f64,
    pub net_credit_per_contract: f64,
    pub contracts: i64,
    pub multiplier: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IronButterflyReport {
    pub max_profit: f64,
    pub max_loss: f64,
    pub lower_breakeven: f64,
    pub upper_breakeven: f64,
    pub wing_width: f64,
}

pub fn analyze(b: &IronButterfly) -> Option<IronButterflyReport> {
    if !b.put_long_strike.is_finite()
        || !b.body_strike.is_finite()
        || !b.call_long_strike.is_finite()
        || !b.net_credit_per_contract.is_finite()
        || !b.multiplier.is_finite()
    {
        return None;
    }
    if !(b.put_long_strike < b.body_strike && b.body_strike < b.call_long_strike) {
        return None;
    }
    let put_width = b.body_strike - b.put_long_strike;
    let call_width = b.call_long_strike - b.body_strike;
    if (put_width - call_width).abs() > 1e-9 {
        return None;    // not symmetric → would be a broken-wing variant
    }
    if b.net_credit_per_contract <= 0.0
        || b.multiplier <= 0.0
        || b.contracts == 0
    {
        return None;
    }
    let wing = put_width;
    if b.net_credit_per_contract > wing {
        return None;    // credit exceeds wing width → mispriced
    }
    let scale = b.contracts.unsigned_abs() as f64 * b.multiplier;
    let sign = b.contracts.signum() as f64;
    Some(IronButterflyReport {
        max_profit: b.net_credit_per_contract * scale * sign,
        max_loss: -(wing - b.net_credit_per_contract) * scale * sign,
        lower_breakeven: b.body_strike - b.net_credit_per_contract,
        upper_breakeven: b.body_strike + b.net_credit_per_contract,
        wing_width: wing,
    })
}

pub fn pnl_at_expiration(b: &IronButterfly, spot: f64) -> Option<f64> {
    if !spot.is_finite()
        || !b.put_long_strike.is_finite()
        || !b.body_strike.is_finite()
        || !b.call_long_strike.is_finite()
        || !b.net_credit_per_contract.is_finite()
    {
        return None;
    }
    let credit = b.net_credit_per_contract;
    let short_put = (b.body_strike - spot).max(0.0);
    let long_put = (b.put_long_strike - spot).max(0.0);
    let short_call = (spot - b.body_strike).max(0.0);
    let long_call = (spot - b.call_long_strike).max(0.0);
    let per = credit - short_put + long_put - short_call + long_call;
    let scale = b.contracts.unsigned_abs() as f64 * b.multiplier;
    let sign = b.contracts.signum() as f64;
    Some(per * scale * sign)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ib() -> IronButterfly {
        IronButterfly {
            put_long_strike: 95.0,
            body_strike: 100.0,
            call_long_strike: 105.0,
            net_credit_per_contract: 3.0,
            contracts: 1,
            multiplier: 100.0,
        }
    }

    #[test]
    fn asymmetric_wings_rejected() {
        let mut bad = ib();
        bad.call_long_strike = 110.0;    // wing 5/10
        assert!(analyze(&bad).is_none());
    }

    #[test]
    fn wrong_strike_order_rejected() {
        let mut bad = ib();
        std::mem::swap(&mut bad.body_strike, &mut bad.put_long_strike);
        assert!(analyze(&bad).is_none());
    }

    #[test]
    fn zero_credit_rejected() {
        let mut bad = ib();
        bad.net_credit_per_contract = 0.0;
        assert!(analyze(&bad).is_none());
    }

    #[test]
    fn credit_exceeding_wing_rejected() {
        let mut bad = ib();
        bad.net_credit_per_contract = 10.0;    // > wing 5
        assert!(analyze(&bad).is_none());
    }

    #[test]
    fn standard_iron_butterfly_textbook_economics() {
        // Wing 5, credit 3. Max profit = $300. Max loss = (5 − 3) · 100 = $200.
        // BE_low = 97, BE_high = 103.
        let r = analyze(&ib()).unwrap();
        assert!((r.max_profit - 300.0).abs() < 1e-9);
        assert!((r.max_loss + 200.0).abs() < 1e-9);
        assert!((r.lower_breakeven - 97.0).abs() < 1e-9);
        assert!((r.upper_breakeven - 103.0).abs() < 1e-9);
        assert!((r.wing_width - 5.0).abs() < 1e-9);
    }

    #[test]
    fn pnl_at_body_strike_equals_max_profit() {
        let p = pnl_at_expiration(&ib(), 100.0).unwrap();
        assert!((p - 300.0).abs() < 1e-9);
    }

    #[test]
    fn pnl_at_wings_equals_max_loss() {
        let p_lo = pnl_at_expiration(&ib(), 95.0).unwrap();
        let p_hi = pnl_at_expiration(&ib(), 105.0).unwrap();
        assert!((p_lo + 200.0).abs() < 1e-9);
        assert!((p_hi + 200.0).abs() < 1e-9);
    }

    #[test]
    fn pnl_at_breakeven_zero() {
        let p_lo = pnl_at_expiration(&ib(), 97.0).unwrap();
        let p_hi = pnl_at_expiration(&ib(), 103.0).unwrap();
        assert!(p_lo.abs() < 1e-9);
        assert!(p_hi.abs() < 1e-9);
    }

    #[test]
    fn short_position_inverts_pnl() {
        let mut short = ib();
        short.contracts = -1;
        let p_long = pnl_at_expiration(&ib(), 100.0).unwrap();
        let p_short = pnl_at_expiration(&short, 100.0).unwrap();
        assert!((p_long + p_short).abs() < 1e-9);
    }
}
