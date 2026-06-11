//! Options desk dailies — three screens run every morning:
//!
//! * Early assignment — an American call is exercised ahead of ex-div
//!   when the dividend exceeds the call's remaining extrinsic value
//!   (the exerciser trades extrinsic for the dividend). Short ITM
//!   calls with extrinsic < dividend are getting assigned tonight.
//! * Event vol — total IV decomposes into ambient vol plus a one-day
//!   event move: σ²_tot·T = σ²_amb·(T − 1/252) + move². Solves the
//!   implied event move and the post-event IV crush.
//! * Gamma-theta breakeven — from 0.5·Γ·ΔS² = |θ|, the daily move
//!   where long gamma pays for its decay: ΔS = √(2|θ|/Γ).
//!
//! Pure compute. Companion to `pin_risk_scanner` (expiry pinning),
//! `second_order_greeks`.

use serde::Serialize;

// ── early assignment ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct EarlyAssignmentReport {
    pub intrinsic: f64,
    pub extrinsic: f64,
    pub dividend: f64,
    /// dividend − extrinsic; positive = exercise is rational.
    pub exercise_edge: f64,
    pub assignment_likely: bool,
}

pub fn early_assignment(
    spot: f64,
    strike: f64,
    call_price: f64,
    dividend: f64,
) -> Option<EarlyAssignmentReport> {
    if ![spot, strike, call_price, dividend].iter().all(|v| v.is_finite())
        || spot <= 0.0
        || strike <= 0.0
        || call_price < 0.0
        || dividend < 0.0
    {
        return None;
    }
    let intrinsic = (spot - strike).max(0.0);
    let extrinsic = (call_price - intrinsic).max(0.0);
    let edge = dividend - extrinsic;
    Some(EarlyAssignmentReport {
        intrinsic,
        extrinsic,
        dividend,
        exercise_edge: edge,
        // OTM calls are never exercised early for a dividend.
        assignment_likely: intrinsic > 0.0 && edge > 0.0,
    })
}

// ── event vol ──────────────────────────────────────────────────────────

const TRADING_DAYS: f64 = 252.0;

#[derive(Debug, Clone, Serialize)]
pub struct EventVolReport {
    /// Implied one-day event move, % of spot.
    pub implied_event_move_pct: f64,
    /// IV the chain should crush to once the event passes.
    pub post_event_iv_pct: f64,
    pub iv_crush_pct_points: f64,
}

/// `total_iv_pct` on the expiry spanning the event; `ambient_iv_pct`
/// the no-event vol (a far expiry or post-event estimate);
/// `days_to_expiry` in trading days (must be ≥ 1, the event day).
pub fn event_vol(
    total_iv_pct: f64,
    ambient_iv_pct: f64,
    days_to_expiry: f64,
) -> Option<EventVolReport> {
    if !total_iv_pct.is_finite()
        || total_iv_pct <= 0.0
        || !ambient_iv_pct.is_finite()
        || ambient_iv_pct < 0.0
        || !days_to_expiry.is_finite()
        || days_to_expiry < 1.0
    {
        return None;
    }
    let t = days_to_expiry / TRADING_DAYS;
    let t_amb = (days_to_expiry - 1.0) / TRADING_DAYS;
    let var_total = (total_iv_pct / 100.0).powi(2) * t;
    let var_ambient = (ambient_iv_pct / 100.0).powi(2) * t_amb;
    let move_sq = var_total - var_ambient;
    if move_sq <= 0.0 {
        // Total vol doesn't even cover ambient — no event priced in.
        return Some(EventVolReport {
            implied_event_move_pct: 0.0,
            post_event_iv_pct: ambient_iv_pct,
            iv_crush_pct_points: total_iv_pct - ambient_iv_pct,
        });
    }
    Some(EventVolReport {
        implied_event_move_pct: move_sq.sqrt() * 100.0,
        post_event_iv_pct: ambient_iv_pct,
        iv_crush_pct_points: total_iv_pct - ambient_iv_pct,
    })
}

// ── gamma-theta breakeven ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct GammaThetaReport {
    /// Daily move where gamma P/L offsets decay, $ of underlying.
    pub breakeven_move: f64,
    pub breakeven_move_pct: f64,
    /// Annualized vol the breakeven implies — compare against realized
    /// to judge whether the gamma is worth owning.
    pub implied_breakeven_vol_pct: f64,
}

/// `gamma` per $ (∂Δ/∂S), `theta_daily` $ per day (sign ignored),
/// `spot` for the percentage row.
pub fn gamma_theta_breakeven(
    gamma: f64,
    theta_daily: f64,
    spot: f64,
) -> Option<GammaThetaReport> {
    if !gamma.is_finite()
        || gamma <= 0.0
        || !theta_daily.is_finite()
        || theta_daily == 0.0
        || !spot.is_finite()
        || spot <= 0.0
    {
        return None;
    }
    let be = (2.0 * theta_daily.abs() / gamma).sqrt();
    let be_pct = be / spot * 100.0;
    Some(GammaThetaReport {
        breakeven_move: be,
        breakeven_move_pct: be_pct,
        implied_breakeven_vol_pct: be_pct * TRADING_DAYS.sqrt(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deep_itm_call_with_thin_extrinsic_flags_assignment() {
        // S 104, K 100, C 5.20: intrinsic 4, extrinsic 1.20; $1.50
        // dividend beats it ⇒ assignment.
        let r = early_assignment(104.0, 100.0, 5.2, 1.5).unwrap();
        assert!((r.extrinsic - 1.2).abs() < 1e-12);
        assert!((r.exercise_edge - 0.3).abs() < 1e-12);
        assert!(r.assignment_likely);
        // Same chain, $1.00 dividend ⇒ extrinsic wins, hold.
        assert!(!early_assignment(104.0, 100.0, 5.2, 1.0).unwrap().assignment_likely);
    }

    #[test]
    fn otm_calls_never_assign_early() {
        let r = early_assignment(95.0, 100.0, 2.0, 5.0).unwrap();
        assert_eq!(r.intrinsic, 0.0);
        assert!(!r.assignment_likely); // edge positive but OTM
    }

    #[test]
    fn event_vol_hand_walk() {
        // IV 60 over 5 days, ambient 30: move² = .36·5/252 − .09·4/252
        // = .0057143 ⇒ 7.559% one-day move; crush 30pp.
        let r = event_vol(60.0, 30.0, 5.0).unwrap();
        let want = (0.36_f64 * 5.0 / 252.0 - 0.09 * 4.0 / 252.0).sqrt() * 100.0;
        assert!((r.implied_event_move_pct - want).abs() < 1e-9);
        assert!((r.iv_crush_pct_points - 30.0).abs() < 1e-12);
        assert!((r.post_event_iv_pct - 30.0).abs() < 1e-12);
    }

    #[test]
    fn no_event_premium_reads_zero_move() {
        // Total below ambient ⇒ no event priced.
        let r = event_vol(25.0, 30.0, 5.0).unwrap();
        assert_eq!(r.implied_event_move_pct, 0.0);
        assert!(r.iv_crush_pct_points < 0.0);
    }

    #[test]
    fn gamma_theta_hand_walk() {
        // Γ 0.05, θ −$2.50/day ⇒ √(5/0.05) = $10 breakeven on a $200
        // stock = 5%/day ⇒ ~79.4% implied breakeven vol.
        let r = gamma_theta_breakeven(0.05, -2.5, 200.0).unwrap();
        assert!((r.breakeven_move - 10.0).abs() < 1e-12);
        assert!((r.breakeven_move_pct - 5.0).abs() < 1e-12);
        assert!((r.implied_breakeven_vol_pct - 5.0 * 252.0_f64.sqrt()).abs() < 1e-9);
    }

    #[test]
    fn hostile_inputs_return_none() {
        assert!(early_assignment(0.0, 100.0, 5.0, 1.0).is_none());
        assert!(early_assignment(104.0, 100.0, -1.0, 1.0).is_none());
        assert!(event_vol(60.0, 30.0, 0.5).is_none()); // < event day
        assert!(event_vol(0.0, 30.0, 5.0).is_none());
        assert!(gamma_theta_breakeven(0.0, -2.5, 200.0).is_none());
        assert!(gamma_theta_breakeven(0.05, 0.0, 200.0).is_none());
    }
}
