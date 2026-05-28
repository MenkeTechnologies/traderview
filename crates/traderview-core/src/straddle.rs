//! Straddle P&L analyzer — long or short call+put at the same strike.
//!
//! Construction:
//!   - **long straddle**:  long 1 call + long 1 put @ K (debit)
//!   - **short straddle**: short 1 call + short 1 put @ K (credit)
//!
//! At expiration:
//!   pnl(S) = sign · [max(S − K, 0) + max(K − S, 0) − net_debit]
//!         = sign · [|S − K| − net_debit]
//!
//! For a long straddle, breakevens at K ± net_debit. Max profit on short
//! straddle = credit; on long straddle = unbounded above and bounded
//! below at K (puts max gain ≈ K − net_debit).
//!
//! Pure compute. Distinct from `iron_condor` (uses 4 strikes), distinct
//! from `chooser_option` (which is a single option with delayed
//! commitment).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Straddle {
    pub strike: f64,
    pub net_premium_per_contract: f64,    // positive for debit (long), positive for credit (short)
    pub contracts: i64,                   // > 0 = long, < 0 = short
    pub multiplier: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StraddleReport {
    pub max_profit: f64,                 // unbounded for long = +∞
    pub max_loss: f64,                   // unbounded for short = −∞
    pub lower_breakeven: f64,
    pub upper_breakeven: f64,
    pub profit_zone_width: f64,
    pub is_long: bool,
}

pub fn analyze(s: &Straddle) -> Option<StraddleReport> {
    if !s.strike.is_finite() || s.strike <= 0.0
        || !s.net_premium_per_contract.is_finite() || s.net_premium_per_contract < 0.0
        || !s.multiplier.is_finite() || s.multiplier <= 0.0
        || s.contracts == 0
    {
        return None;
    }
    let is_long = s.contracts > 0;
    let scale = s.contracts.unsigned_abs() as f64 * s.multiplier;
    let lower_be = s.strike - s.net_premium_per_contract;
    let upper_be = s.strike + s.net_premium_per_contract;
    let (max_profit, max_loss) = if is_long {
        // Long straddle: profit unbounded above; max loss = premium paid.
        (f64::INFINITY, -s.net_premium_per_contract * scale)
    } else {
        // Short straddle: max profit = premium received; max loss unbounded.
        (s.net_premium_per_contract * scale, f64::NEG_INFINITY)
    };
    Some(StraddleReport {
        max_profit,
        max_loss,
        lower_breakeven: lower_be,
        upper_breakeven: upper_be,
        profit_zone_width: upper_be - lower_be,
        is_long,
    })
}

pub fn pnl_at_expiration(s: &Straddle, spot: f64) -> Option<f64> {
    if !spot.is_finite() || !s.strike.is_finite() || !s.net_premium_per_contract.is_finite() {
        return None;
    }
    let intrinsic = (spot - s.strike).abs();
    let per = intrinsic - s.net_premium_per_contract;
    let scale = s.contracts.unsigned_abs() as f64 * s.multiplier;
    let sign = s.contracts.signum() as f64;
    Some(per * scale * sign)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn long_straddle() -> Straddle {
        Straddle {
            strike: 100.0,
            net_premium_per_contract: 5.0,
            contracts: 1,
            multiplier: 100.0,
        }
    }

    #[test]
    fn invalid_inputs_return_none() {
        let mut bad = long_straddle(); bad.strike = 0.0;
        assert!(analyze(&bad).is_none());
        let mut bad = long_straddle(); bad.net_premium_per_contract = -1.0;
        assert!(analyze(&bad).is_none());
        let mut bad = long_straddle(); bad.contracts = 0;
        assert!(analyze(&bad).is_none());
        let mut bad = long_straddle(); bad.multiplier = 0.0;
        assert!(analyze(&bad).is_none());
    }

    #[test]
    fn long_straddle_breakevens_at_strike_plus_minus_premium() {
        let r = analyze(&long_straddle()).unwrap();
        assert!((r.lower_breakeven - 95.0).abs() < 1e-9);
        assert!((r.upper_breakeven - 105.0).abs() < 1e-9);
        assert!(r.is_long);
        assert!(r.max_profit.is_infinite());
        assert!((r.max_loss + 500.0).abs() < 1e-9);
    }

    #[test]
    fn short_straddle_max_profit_equals_credit() {
        let mut short = long_straddle();
        short.contracts = -1;
        let r = analyze(&short).unwrap();
        assert!(!r.is_long);
        assert!((r.max_profit - 500.0).abs() < 1e-9);
        assert!(r.max_loss.is_infinite() && r.max_loss < 0.0);
    }

    #[test]
    fn pnl_at_strike_equals_negative_premium_for_long() {
        let p = pnl_at_expiration(&long_straddle(), 100.0).unwrap();
        assert!((p + 500.0).abs() < 1e-9);
    }

    #[test]
    fn pnl_at_lower_breakeven_zero() {
        let p = pnl_at_expiration(&long_straddle(), 95.0).unwrap();
        assert!(p.abs() < 1e-9);
    }

    #[test]
    fn pnl_at_upper_breakeven_zero() {
        let p = pnl_at_expiration(&long_straddle(), 105.0).unwrap();
        assert!(p.abs() < 1e-9);
    }

    #[test]
    fn deep_itm_long_straddle_pnl_positive() {
        let p = pnl_at_expiration(&long_straddle(), 150.0).unwrap();
        // intrinsic = 50, premium = 5 → 45 · 100 = 4500.
        assert!((p - 4500.0).abs() < 1e-9);
    }

    #[test]
    fn short_straddle_pnl_inverts() {
        let mut short = long_straddle();
        short.contracts = -1;
        let p_long = pnl_at_expiration(&long_straddle(), 110.0).unwrap();
        let p_short = pnl_at_expiration(&short, 110.0).unwrap();
        assert!((p_long + p_short).abs() < 1e-9);
    }

    #[test]
    fn profit_zone_width_equals_two_premium() {
        let r = analyze(&long_straddle()).unwrap();
        assert!((r.profit_zone_width - 10.0).abs() < 1e-9);
    }
}
