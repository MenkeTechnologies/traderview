//! Volatility Smile / Smirk — implied-volatility curve across strikes
//! at a single expiration.
//!
//! Caller supplies (strike, iv) pairs; this module fits the smile and
//! reports key shape statistics:
//!   - atm_iv: IV at the strike closest to spot
//!   - skew: (iv at 25-delta put - iv at 25-delta call) (in vol points)
//!   - smile_curvature: second derivative of log-strike vs IV at ATM
//!     (positive = smile, negative = inverted)
//!   - wing_steepness: slope of the put-side wing far OTM
//!
//! Pure compute. Companion to `vix_skew_smirk`, `iv_skew_scanner`,
//! `iv_term_structure`, `iv_rank_scanner`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StrikeIv { pub strike: f64, pub iv: f64 }

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct VolatilitySmileReport {
    pub atm_iv: f64,
    pub skew_25d: f64,
    pub smile_curvature: f64,
    pub put_wing_steepness: f64,
    pub call_wing_steepness: f64,
    pub n_strikes: usize,
}

pub fn compute(
    strike_iv: &[StrikeIv],
    spot: f64,
    put_25d_strike: f64,
    call_25d_strike: f64,
) -> Option<VolatilitySmileReport> {
    if strike_iv.len() < 3 || !spot.is_finite() || spot <= 0.0
        || !put_25d_strike.is_finite() || !call_25d_strike.is_finite() {
        return None;
    }
    if strike_iv.iter().any(|s| !s.strike.is_finite() || !s.iv.is_finite()
        || s.strike <= 0.0 || s.iv <= 0.0) {
        return None;
    }
    let mut sorted: Vec<StrikeIv> = strike_iv.to_vec();
    sorted.sort_by(|a, b| a.strike.partial_cmp(&b.strike).unwrap_or(std::cmp::Ordering::Equal));
    // ATM IV: closest strike to spot.
    let atm = sorted.iter().min_by(|a, b| {
        (a.strike - spot).abs().partial_cmp(&(b.strike - spot).abs())
            .unwrap_or(std::cmp::Ordering::Equal)
    }).unwrap();
    let atm_iv = atm.iv;
    // 25-delta skew.
    let interp = |target: f64| -> f64 {
        if sorted.iter().all(|s| s.strike < target) {
            return sorted.last().unwrap().iv;
        }
        if sorted.iter().all(|s| s.strike > target) {
            return sorted.first().unwrap().iv;
        }
        // Linear interpolation between nearest bracketing strikes.
        let mut lo = sorted[0];
        let mut hi = sorted[sorted.len() - 1];
        for w in sorted.windows(2) {
            if w[0].strike <= target && w[1].strike >= target {
                lo = w[0];
                hi = w[1];
                break;
            }
        }
        if (hi.strike - lo.strike).abs() < 1e-12 { return lo.iv; }
        let t = (target - lo.strike) / (hi.strike - lo.strike);
        lo.iv + t * (hi.iv - lo.iv)
    };
    let put_iv = interp(put_25d_strike);
    let call_iv = interp(call_25d_strike);
    let skew = put_iv - call_iv;
    // Smile curvature: 2nd derivative of IV at ATM using central diff.
    // Find ATM index in sorted.
    let atm_idx = sorted.iter().position(|s| s.strike == atm.strike).unwrap_or(sorted.len() / 2);
    let curvature = if atm_idx > 0 && atm_idx + 1 < sorted.len() {
        let lo = sorted[atm_idx - 1];
        let hi = sorted[atm_idx + 1];
        let dx_lo = atm.strike - lo.strike;
        let dx_hi = hi.strike - atm.strike;
        if dx_lo > 0.0 && dx_hi > 0.0 {
            2.0 * (
                (hi.iv - atm.iv) / dx_hi - (atm.iv - lo.iv) / dx_lo
            ) / (dx_lo + dx_hi)
        } else { 0.0 }
    } else { 0.0 };
    // Wing slopes: linear slope from ATM to the most-OTM point on each side.
    let put_wing = if atm_idx > 0 {
        let edge = sorted[0];
        if (atm.strike - edge.strike).abs() > 0.0 {
            (atm.iv - edge.iv) / (atm.strike - edge.strike)
        } else { 0.0 }
    } else { 0.0 };
    let call_wing = if atm_idx + 1 < sorted.len() {
        let edge = sorted[sorted.len() - 1];
        if (edge.strike - atm.strike).abs() > 0.0 {
            (edge.iv - atm.iv) / (edge.strike - atm.strike)
        } else { 0.0 }
    } else { 0.0 };
    Some(VolatilitySmileReport {
        atm_iv,
        skew_25d: skew,
        smile_curvature: curvature,
        put_wing_steepness: put_wing,
        call_wing_steepness: call_wing,
        n_strikes: sorted.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(strike: f64, iv: f64) -> StrikeIv { StrikeIv { strike, iv } }

    #[test]
    fn empty_or_invalid_returns_none() {
        assert!(compute(&[], 100.0, 90.0, 110.0).is_none());
        assert!(compute(&[s(100.0, 20.0); 3], 0.0, 90.0, 110.0).is_none());
    }

    #[test]
    fn nan_returns_none() {
        let curve = vec![s(100.0, f64::NAN), s(105.0, 20.0), s(110.0, 22.0)];
        assert!(compute(&curve, 100.0, 90.0, 110.0).is_none());
    }

    #[test]
    fn flat_smile_yields_zero_skew() {
        let curve = vec![s(90.0, 20.0), s(100.0, 20.0), s(110.0, 20.0)];
        let r = compute(&curve, 100.0, 90.0, 110.0).unwrap();
        assert!(r.skew_25d.abs() < 1e-9);
        assert!((r.atm_iv - 20.0).abs() < 1e-9);
    }

    #[test]
    fn put_skew_yields_positive_skew() {
        let curve = vec![s(90.0, 30.0), s(100.0, 20.0), s(110.0, 18.0)];
        let r = compute(&curve, 100.0, 90.0, 110.0).unwrap();
        assert!(r.skew_25d > 0.0);    // put IV > call IV
    }

    #[test]
    fn call_skew_yields_negative_skew() {
        let curve = vec![s(90.0, 18.0), s(100.0, 20.0), s(110.0, 30.0)];
        let r = compute(&curve, 100.0, 90.0, 110.0).unwrap();
        assert!(r.skew_25d < 0.0);
    }

    #[test]
    fn smile_yields_positive_curvature() {
        // Symmetric smile: both wings higher than ATM.
        let curve = vec![s(90.0, 25.0), s(100.0, 20.0), s(110.0, 25.0)];
        let r = compute(&curve, 100.0, 90.0, 110.0).unwrap();
        assert!(r.smile_curvature > 0.0);
    }
}
