//! Double-barrier first-touch probability — which level gets hit
//! first, target or stop?
//!
//! For GBM, log-price is Brownian motion with drift ν = μ − σ²/2.
//! With barriers at a = ln(S/L) below and b = ln(U/S) above, the scale
//! function s(x) = e^{−2νx/σ²} gives the classic exit split:
//!
//!   P(hit U before L) = (1 − s(a)⁻¹…)        — in scale terms:
//!                       (s(0) − s(−a)) / (s(b) − s(−a))
//!
//! with the ν → 0 limit a/(a+b). The strip is exited with probability
//! one, so the two probabilities sum to 1 — this is the bracket-order
//! question (target vs stop) answered without simulation.
//!
//! Pure compute. Companion to `probability_of_touch` (single barrier,
//! finite horizon), `bracket_order`.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct DoubleBarrierReport {
    pub p_upper_first: f64,
    pub p_lower_first: f64,
    /// Log drift ν = μ − σ²/2 actually used.
    pub log_drift: f64,
}

/// `drift` is the annual expected return μ (0 for the risk-symmetric
/// view); `vol` annualized. L < S < U strictly.
pub fn compute(
    spot: f64,
    lower: f64,
    upper: f64,
    drift: f64,
    vol: f64,
) -> Option<DoubleBarrierReport> {
    let valid = [spot, lower, upper].iter().all(|v| v.is_finite() && *v > 0.0)
        && lower < spot
        && spot < upper
        && drift.is_finite()
        && vol.is_finite()
        && vol > 0.0;
    if !valid {
        return None;
    }
    let nu = drift - vol * vol / 2.0;
    let a = (spot / lower).ln(); // distance down, log space (> 0)
    let b = (upper / spot).ln(); // distance up (> 0)
    let theta = 2.0 * nu / (vol * vol);
    let p_up = if theta.abs() < 1e-12 {
        // Driftless limit: a/(a+b).
        a / (a + b)
    } else {
        // s(x) = e^{−θx}; P = (s(0) − s(−a))/(s(b) − s(−a)).
        let s0 = 1.0;
        let s_neg_a = (theta * a).exp();
        let s_b = (-theta * b).exp();
        (s0 - s_neg_a) / (s_b - s_neg_a)
    };
    let p_up = p_up.clamp(0.0, 1.0);
    Some(DoubleBarrierReport {
        p_upper_first: p_up,
        p_lower_first: 1.0 - p_up,
        log_drift: nu,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn symmetric_log_barriers_with_zero_log_drift_split_evenly() {
        // ν = 0 requires μ = σ²/2; barriers at ×1.1 and ÷1.1 are
        // symmetric in logs ⇒ exactly 50/50.
        let vol = 0.3_f64;
        let mu = vol * vol / 2.0;
        let r = compute(100.0, 100.0 / 1.1, 110.0, mu, vol).unwrap();
        assert!((r.p_upper_first - 0.5).abs() < 1e-9, "{}", r.p_upper_first);
        assert!((r.p_upper_first + r.p_lower_first - 1.0).abs() < 1e-12);
    }

    #[test]
    fn driftless_asymmetric_barriers_follow_a_over_a_plus_b() {
        // Stop twice as far (in logs) as the target ⇒ target hit first
        // with probability a/(a+b) = 2/3.
        let vol = 0.2_f64;
        let mu = vol * vol / 2.0; // ν = 0
        let lower = 100.0 * (-0.2_f64).exp(); // a = 0.2
        let upper = 100.0 * (0.1_f64).exp(); // b = 0.1
        let r = compute(100.0, lower, upper, mu, vol).unwrap();
        assert!((r.p_upper_first - 2.0 / 3.0).abs() < 1e-9, "{}", r.p_upper_first);
    }

    #[test]
    fn positive_drift_tilts_toward_the_upper_barrier() {
        let base = compute(100.0, 90.0, 110.0, 0.0, 0.2).unwrap();
        let drifted = compute(100.0, 90.0, 110.0, 0.15, 0.2).unwrap();
        assert!(drifted.p_upper_first > base.p_upper_first);
        // And negative drift the other way.
        let bear = compute(100.0, 90.0, 110.0, -0.15, 0.2).unwrap();
        assert!(bear.p_upper_first < base.p_upper_first);
    }

    #[test]
    fn tight_target_wide_stop_is_the_high_winrate_negative_skew_trade() {
        // 2% target, 10% stop, no drift edge: win rate looks great —
        // that's exactly why the trade is seductive.
        let r = compute(100.0, 90.0, 102.0, 0.02, 0.2).unwrap();
        assert!(r.p_upper_first > 0.75, "{}", r.p_upper_first);
    }

    #[test]
    fn hostile_inputs_return_none() {
        assert!(compute(100.0, 110.0, 120.0, 0.0, 0.2).is_none()); // spot below lower
        assert!(compute(100.0, 90.0, 95.0, 0.0, 0.2).is_none()); // upper below spot
        assert!(compute(100.0, 90.0, 110.0, 0.0, 0.0).is_none());
        assert!(compute(100.0, 0.0, 110.0, 0.0, 0.2).is_none());
        assert!(compute(f64::NAN, 90.0, 110.0, 0.0, 0.2).is_none());
    }
}
