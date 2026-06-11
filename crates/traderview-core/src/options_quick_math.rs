//! Options desk quick math — the mental approximations, with their
//! exact counterparts printed beside them so the error is visible:
//!
//! * Rule of 16: expected daily move ≈ IV/16 (exact: IV/√252).
//! * ATM straddle ≈ 0.8·S·σ√T (exact: BS straddle; the true constant
//!   is √(2/π)·2 ≈ 0.7979).
//! * ATM call ≈ 0.4·S·σ√T.
//!
//! Pure compute; exact legs price through the shared `black_scholes`
//! at r = q = 0 (the regime where the approximations were derived).

use serde::{Deserialize, Serialize};

const TRADING_DAYS: f64 = 252.0;

#[derive(Debug, Clone, Deserialize)]
pub struct QuickMathInput {
    pub spot: f64,
    /// Annualized IV, %.
    pub iv_pct: f64,
    /// Days to expiry for the straddle/call rows.
    pub days_to_expiry: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct QuickMathReport {
    /// IV/16 — the mental number.
    pub daily_move_rule16_pct: f64,
    /// IV/√252 — the exact one-σ daily move.
    pub daily_move_exact_pct: f64,
    pub weekly_move_exact_pct: f64,
    /// 0.8·S·σ√T quick straddle vs the exact BS straddle.
    pub straddle_approx: f64,
    pub straddle_exact: f64,
    pub straddle_approx_error_pct: f64,
    /// 0.4·S·σ√T quick ATM call.
    pub atm_call_approx: f64,
    pub atm_call_exact: f64,
}

pub fn compute(inp: &QuickMathInput) -> Option<QuickMathReport> {
    if !inp.spot.is_finite()
        || inp.spot <= 0.0
        || !inp.iv_pct.is_finite()
        || inp.iv_pct <= 0.0
        || !inp.days_to_expiry.is_finite()
        || inp.days_to_expiry <= 0.0
    {
        return None;
    }
    let sigma = inp.iv_pct / 100.0;
    let t = inp.days_to_expiry / TRADING_DAYS;
    let sqrt_t = t.sqrt();
    let call_exact = crate::black_scholes::call(inp.spot, inp.spot, t, 0.0, 0.0, sigma);
    let put_exact = crate::black_scholes::put(inp.spot, inp.spot, t, 0.0, 0.0, sigma);
    let straddle_exact = call_exact + put_exact;
    let straddle_approx = 0.8 * inp.spot * sigma * sqrt_t;
    Some(QuickMathReport {
        daily_move_rule16_pct: inp.iv_pct / 16.0,
        daily_move_exact_pct: inp.iv_pct / TRADING_DAYS.sqrt(),
        weekly_move_exact_pct: inp.iv_pct / (TRADING_DAYS / 5.0).sqrt(),
        straddle_approx,
        straddle_exact,
        straddle_approx_error_pct: (straddle_approx / straddle_exact - 1.0) * 100.0,
        atm_call_approx: 0.4 * inp.spot * sigma * sqrt_t,
        atm_call_exact: call_exact,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rule_of_16_vs_exact() {
        // 16% IV: rule says 1.000%/day; exact 16/√252 = 1.008%.
        let r = compute(&QuickMathInput {
            spot: 100.0,
            iv_pct: 16.0,
            days_to_expiry: 30.0,
        })
        .unwrap();
        assert!((r.daily_move_rule16_pct - 1.0).abs() < 1e-12);
        assert!((r.daily_move_exact_pct - 16.0 / 252.0_f64.sqrt()).abs() < 1e-12);
        // The /16 shortcut understates by under 1%.
        assert!((r.daily_move_rule16_pct / r.daily_move_exact_pct - 1.0).abs() < 0.01);
    }

    #[test]
    fn straddle_approx_tracks_bs_within_one_percent() {
        // The exact ATM r=0 straddle constant is √(2/π)·2 ≈ 0.7979 —
        // 0.8 lands within ~0.3% for short maturities.
        let r = compute(&QuickMathInput {
            spot: 100.0,
            iv_pct: 25.0,
            days_to_expiry: 21.0,
        })
        .unwrap();
        assert!(r.straddle_approx_error_pct.abs() < 1.0, "{}", r.straddle_approx_error_pct);
        // Call ≈ half the straddle at r = 0.
        assert!((r.atm_call_exact - r.straddle_exact / 2.0).abs() < 1e-9);
        assert!((r.atm_call_approx - r.straddle_approx / 2.0).abs() < 1e-12);
    }

    #[test]
    fn approx_error_grows_with_maturity() {
        // The 0.8 shortcut is a small-σ√T expansion; a 2-year tenor at
        // high vol drifts further from exact than a 1-month tenor.
        let short = compute(&QuickMathInput { spot: 100.0, iv_pct: 40.0, days_to_expiry: 21.0 }).unwrap();
        let long = compute(&QuickMathInput { spot: 100.0, iv_pct: 40.0, days_to_expiry: 504.0 }).unwrap();
        assert!(long.straddle_approx_error_pct.abs() > short.straddle_approx_error_pct.abs());
    }

    #[test]
    fn hostile_inputs_return_none() {
        assert!(compute(&QuickMathInput { spot: 0.0, iv_pct: 16.0, days_to_expiry: 30.0 }).is_none());
        assert!(compute(&QuickMathInput { spot: 100.0, iv_pct: 0.0, days_to_expiry: 30.0 }).is_none());
        assert!(compute(&QuickMathInput { spot: 100.0, iv_pct: 16.0, days_to_expiry: 0.0 }).is_none());
    }
}
