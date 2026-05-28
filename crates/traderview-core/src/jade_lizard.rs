//! Jade Lizard P&L analyzer (Tom Sosnoff / tastytrade).
//!
//! Three legs (all same expiry):
//!   - **short put** @ put_strike (typically OTM, below spot)
//!   - **short call** @ call_short_strike (typically OTM, above spot)
//!   - **long call** @ call_long_strike (further OTM than short call)
//!
//! Net credit construction. Key design rule: net_credit ≥ call wing
//! width — that condition makes the trade "no-upside-risk" (the short
//! call spread can only lose call_wing_width on the upside, but the
//! collected credit covers that loss).
//!
//! At expiration:
//!   pnl(S) = credit − max(0, K_put − S) − max(0, S − K_short_call)
//!          + max(0, S − K_long_call)
//!
//! Downside risk equals short put strike × multiplier (catastrophic
//! loss if underlying goes to zero — like any naked put). Upside risk
//! is zero by construction.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct JadeLizard {
    pub put_strike: f64,
    pub call_short_strike: f64,
    pub call_long_strike: f64,
    pub net_credit_per_contract: f64,
    pub contracts: i64,
    pub multiplier: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JadeLizardReport {
    pub max_profit: f64,
    pub max_loss_downside: f64,
    pub max_loss_upside: f64,
    pub put_breakeven: f64,
    pub call_wing_width: f64,
    pub no_upside_risk: bool,
}

pub fn analyze(j: &JadeLizard) -> Option<JadeLizardReport> {
    if !j.put_strike.is_finite()
        || !j.call_short_strike.is_finite()
        || !j.call_long_strike.is_finite()
        || !j.net_credit_per_contract.is_finite()
        || !j.multiplier.is_finite()
        || j.put_strike <= 0.0
        || j.call_short_strike <= 0.0
        || j.call_long_strike <= 0.0
        || j.call_short_strike >= j.call_long_strike
        || j.put_strike >= j.call_short_strike
        || j.net_credit_per_contract <= 0.0
        || j.multiplier <= 0.0
        || j.contracts == 0
    {
        return None;
    }
    let scale = j.contracts.unsigned_abs() as f64 * j.multiplier;
    let sign = j.contracts.signum() as f64;
    let call_wing_width = j.call_long_strike - j.call_short_strike;
    let max_profit_per = j.net_credit_per_contract;
    // Upside max loss = call_wing_width − credit (zero or positive only if credit < wing).
    let upside_loss_per = (call_wing_width - j.net_credit_per_contract).max(0.0);
    let no_upside_risk = j.net_credit_per_contract >= call_wing_width;
    // Downside max loss = put_strike − credit (taking the put to assignment at zero).
    let downside_loss_per = (j.put_strike - j.net_credit_per_contract).max(0.0);
    Some(JadeLizardReport {
        max_profit: max_profit_per * scale * sign,
        max_loss_downside: -downside_loss_per * scale * sign,
        max_loss_upside: -upside_loss_per * scale * sign,
        put_breakeven: j.put_strike - j.net_credit_per_contract,
        call_wing_width,
        no_upside_risk,
    })
}

pub fn pnl_at_expiration(j: &JadeLizard, spot: f64) -> Option<f64> {
    if !spot.is_finite()
        || !j.put_strike.is_finite()
        || !j.call_short_strike.is_finite()
        || !j.call_long_strike.is_finite()
        || !j.net_credit_per_contract.is_finite()
    {
        return None;
    }
    let short_put = (j.put_strike - spot).max(0.0);
    let short_call = (spot - j.call_short_strike).max(0.0);
    let long_call = (spot - j.call_long_strike).max(0.0);
    let per = j.net_credit_per_contract - short_put - short_call + long_call;
    let scale = j.contracts.unsigned_abs() as f64 * j.multiplier;
    let sign = j.contracts.signum() as f64;
    Some(per * scale * sign)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn jl() -> JadeLizard {
        // Credit ≥ wing width = 5 → no upside risk. Common tastytrade setup.
        JadeLizard {
            put_strike: 95.0,
            call_short_strike: 105.0,
            call_long_strike: 110.0,
            net_credit_per_contract: 5.5,
            contracts: 1,
            multiplier: 100.0,
        }
    }

    #[test]
    fn invalid_strike_order_rejected() {
        let mut bad = jl();
        bad.call_short_strike = 90.0;    // < put_strike
        assert!(analyze(&bad).is_none());
    }

    #[test]
    fn negative_credit_rejected() {
        let mut bad = jl();
        bad.net_credit_per_contract = -1.0;
        assert!(analyze(&bad).is_none());
    }

    #[test]
    fn zero_contracts_rejected() {
        let mut bad = jl();
        bad.contracts = 0;
        assert!(analyze(&bad).is_none());
    }

    #[test]
    fn nan_inputs_rejected() {
        let mut bad = jl();
        bad.put_strike = f64::NAN;
        assert!(analyze(&bad).is_none());
    }

    #[test]
    fn classic_jade_lizard_no_upside_risk() {
        // Credit 5.5 ≥ wing 5 → no upside risk.
        let r = analyze(&jl()).unwrap();
        assert!(r.no_upside_risk);
        assert!((r.max_profit - 550.0).abs() < 1e-9);
        assert_eq!(r.max_loss_upside, 0.0);    // no upside risk → max_loss_upside = 0
        assert!((r.put_breakeven - 89.5).abs() < 1e-9);
    }

    #[test]
    fn jade_lizard_with_upside_risk_when_credit_below_wing() {
        let mut narrow_credit = jl();
        narrow_credit.net_credit_per_contract = 3.0;    // 3 < wing width 5
        let r = analyze(&narrow_credit).unwrap();
        assert!(!r.no_upside_risk);
        assert!((r.max_loss_upside + 200.0).abs() < 1e-9);    // − (5 − 3) · 100 = −200
    }

    #[test]
    fn pnl_at_high_spot_caps_at_no_upside_loss() {
        // Spot = 200, far above both call strikes — at expiry, both legs in money:
        // pnl_per = 5.5 − 0 − (200 − 105) + (200 − 110)
        //        = 5.5 − 95 + 90 = 0.5 ≥ 0 → no upside loss.
        let p = pnl_at_expiration(&jl(), 200.0).unwrap();
        assert!((p - 50.0).abs() < 1e-9);
    }

    #[test]
    fn pnl_at_spot_at_or_above_short_call_equals_credit() {
        // Between put_strike and call_short_strike, no leg is ITM → pnl = credit.
        let p = pnl_at_expiration(&jl(), 100.0).unwrap();
        assert!((p - 550.0).abs() < 1e-9);
    }

    #[test]
    fn pnl_at_put_breakeven_zero() {
        let p = pnl_at_expiration(&jl(), 89.5).unwrap();
        assert!(p.abs() < 1e-9);
    }

    #[test]
    fn deep_downside_yields_catastrophic_loss() {
        // Spot = 0 (worst case): pnl = 5.5 − 95 = −89.5 per share → −$8_950.
        let p = pnl_at_expiration(&jl(), 0.0).unwrap();
        assert!((p + 8_950.0).abs() < 1e-9);
    }

    #[test]
    fn short_position_inverts_pnl() {
        let mut short = jl();
        short.contracts = -1;
        let p_long = pnl_at_expiration(&jl(), 100.0).unwrap();
        let p_short = pnl_at_expiration(&short, 100.0).unwrap();
        assert!((p_long + p_short).abs() < 1e-9);
    }
}
