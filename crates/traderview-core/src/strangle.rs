//! Strangle P&L analyzer — long or short OTM call + OTM put.
//!
//! Construction:
//!   - long strangle: long call @ K_call (above spot) + long put @ K_put (below spot)
//!   - short strangle: short call @ K_call + short put @ K_put
//!
//! Strangle is cheaper than a straddle (both legs OTM) but needs a
//! bigger move to profit. Constraint: K_put < K_call.
//!
//! At expiration:
//!   pnl(S) = sign · [max(S − K_call, 0) + max(K_put − S, 0) − net_premium]
//!
//! Breakevens: K_put − net_premium (lower) and K_call + net_premium (upper).
//! Profit zone width = K_call − K_put + 2·net_premium.
//!
//! Pure compute. Companion to `straddle` and `iron_condor`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Strangle {
    pub put_strike: f64,
    pub call_strike: f64,
    pub net_premium_per_contract: f64,
    pub contracts: i64,
    pub multiplier: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StrangleReport {
    pub max_profit: f64,
    pub max_loss: f64,
    pub lower_breakeven: f64,
    pub upper_breakeven: f64,
    pub profit_zone_width: f64,
    pub is_long: bool,
}

pub fn analyze(s: &Strangle) -> Option<StrangleReport> {
    if !s.put_strike.is_finite() || s.put_strike <= 0.0
        || !s.call_strike.is_finite() || s.call_strike <= 0.0
        || s.call_strike <= s.put_strike
        || !s.net_premium_per_contract.is_finite() || s.net_premium_per_contract < 0.0
        || !s.multiplier.is_finite() || s.multiplier <= 0.0
        || s.contracts == 0
    {
        return None;
    }
    let is_long = s.contracts > 0;
    let scale = s.contracts.unsigned_abs() as f64 * s.multiplier;
    let lower_be = s.put_strike - s.net_premium_per_contract;
    let upper_be = s.call_strike + s.net_premium_per_contract;
    let (max_profit, max_loss) = if is_long {
        (f64::INFINITY, -s.net_premium_per_contract * scale)
    } else {
        (s.net_premium_per_contract * scale, f64::NEG_INFINITY)
    };
    Some(StrangleReport {
        max_profit,
        max_loss,
        lower_breakeven: lower_be,
        upper_breakeven: upper_be,
        profit_zone_width: upper_be - lower_be,
        is_long,
    })
}

pub fn pnl_at_expiration(s: &Strangle, spot: f64) -> Option<f64> {
    if !spot.is_finite() {
        return None;
    }
    let call_intrinsic = (spot - s.call_strike).max(0.0);
    let put_intrinsic = (s.put_strike - spot).max(0.0);
    let per = call_intrinsic + put_intrinsic - s.net_premium_per_contract;
    let scale = s.contracts.unsigned_abs() as f64 * s.multiplier;
    let sign = s.contracts.signum() as f64;
    Some(per * scale * sign)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn long_strangle() -> Strangle {
        Strangle {
            put_strike: 90.0,
            call_strike: 110.0,
            net_premium_per_contract: 3.0,
            contracts: 1,
            multiplier: 100.0,
        }
    }

    #[test]
    fn invalid_strike_order_rejected() {
        let mut bad = long_strangle();
        std::mem::swap(&mut bad.put_strike, &mut bad.call_strike);
        assert!(analyze(&bad).is_none());
    }

    #[test]
    fn zero_contracts_rejected() {
        let mut bad = long_strangle();
        bad.contracts = 0;
        assert!(analyze(&bad).is_none());
    }

    #[test]
    fn negative_premium_rejected() {
        let mut bad = long_strangle();
        bad.net_premium_per_contract = -1.0;
        assert!(analyze(&bad).is_none());
    }

    #[test]
    fn breakevens_at_strikes_plus_minus_premium() {
        let r = analyze(&long_strangle()).unwrap();
        assert!((r.lower_breakeven - 87.0).abs() < 1e-9);
        assert!((r.upper_breakeven - 113.0).abs() < 1e-9);
    }

    #[test]
    fn pnl_inside_strikes_equals_negative_premium() {
        // Spot = 100, between put 90 and call 110 → both legs zero.
        let p = pnl_at_expiration(&long_strangle(), 100.0).unwrap();
        assert!((p + 300.0).abs() < 1e-9);
    }

    #[test]
    fn pnl_at_lower_breakeven_zero() {
        let p = pnl_at_expiration(&long_strangle(), 87.0).unwrap();
        assert!(p.abs() < 1e-9);
    }

    #[test]
    fn pnl_at_upper_breakeven_zero() {
        let p = pnl_at_expiration(&long_strangle(), 113.0).unwrap();
        assert!(p.abs() < 1e-9);
    }

    #[test]
    fn deep_upside_long_strangle_positive() {
        // Spot = 150 → call intrinsic = 40, put = 0. pnl = (40 − 3) · 100 = 3700.
        let p = pnl_at_expiration(&long_strangle(), 150.0).unwrap();
        assert!((p - 3700.0).abs() < 1e-9);
    }

    #[test]
    fn short_strangle_inverts_pnl() {
        let mut short = long_strangle();
        short.contracts = -1;
        let p_long = pnl_at_expiration(&long_strangle(), 100.0).unwrap();
        let p_short = pnl_at_expiration(&short, 100.0).unwrap();
        assert!((p_long + p_short).abs() < 1e-9);
    }

    #[test]
    fn profit_zone_width_correct() {
        let r = analyze(&long_strangle()).unwrap();
        // 113 − 87 = 26.
        assert!((r.profit_zone_width - 26.0).abs() < 1e-9);
    }
}
