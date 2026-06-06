//! Risk Reversal and Butterfly — canonical FX-options vol-surface
//! quoting: decomposes the 25-delta call / put / ATM straddle into:
//!
//!   ATM   = 0.5 · (σ_25C + σ_25P)            (level)
//!   RR    = σ_25C - σ_25P                    (skew: call - put vol)
//!   BF    = 0.5 · (σ_25C + σ_25P) - σ_ATM    (smile curvature wing)
//!
//! Equivalently (the form FX dealers actually quote):
//!   σ_25C = σ_ATM + BF + 0.5 · RR
//!   σ_25P = σ_ATM + BF - 0.5 · RR
//!
//! Signs (FX convention):
//!   RR > 0 → calls > puts → market priced for upside (e.g. EURUSD rally)
//!   RR < 0 → puts > calls → downside priced more (typical equity skew)
//!   BF > 0 → wings expensive vs ATM (positive convexity in vol)
//!
//! Reports both directions: caller passes (σ_25C, σ_25P, σ_ATM) and
//! gets back the {RR, BF, ATM_level, smile asymmetry score}; or the
//! reverse using `from_atm_rr_bf` to reconstruct wing vols.
//!
//! Pure compute. Companion to `svi_volatility_smile`,
//! `volatility_skew`, `volatility_smile`.

#[derive(Debug)]
pub struct DecompositionReport {
    pub atm: f64,
    pub risk_reversal: f64,
    pub butterfly: f64,
    pub skew_zscore: f64,
}

#[derive(Debug)]
pub struct WingReport {
    pub sigma_25_call: f64,
    pub sigma_25_put: f64,
}

pub fn decompose(
    sigma_25_call: f64,
    sigma_25_put: f64,
    sigma_atm: f64,
) -> Option<DecompositionReport> {
    let vs = [sigma_25_call, sigma_25_put, sigma_atm];
    if vs.iter().any(|x| !x.is_finite()) {
        return None;
    }
    if vs.iter().any(|&x| x <= 0.0) {
        return None;
    }
    let wing_avg = 0.5 * (sigma_25_call + sigma_25_put);
    let rr = sigma_25_call - sigma_25_put;
    let bf = wing_avg - sigma_atm;
    // Skew as normalized z-score: RR relative to wing vol.
    let skew_zscore = if wing_avg > 0.0 { rr / wing_avg } else { 0.0 };
    Some(DecompositionReport {
        atm: sigma_atm,
        risk_reversal: rr,
        butterfly: bf,
        skew_zscore,
    })
}

pub fn from_atm_rr_bf(atm: f64, rr: f64, bf: f64) -> Option<WingReport> {
    let vs = [atm, rr, bf];
    if vs.iter().any(|x| !x.is_finite()) {
        return None;
    }
    if atm <= 0.0 {
        return None;
    }
    let sigma_25_call = atm + bf + 0.5 * rr;
    let sigma_25_put = atm + bf - 0.5 * rr;
    if sigma_25_call < 0.0 || sigma_25_put < 0.0 {
        return None;
    }
    Some(WingReport {
        sigma_25_call,
        sigma_25_put,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decompose_rejects_invalid_inputs() {
        assert!(decompose(0.0, 0.10, 0.10).is_none());
        assert!(decompose(0.10, -0.05, 0.10).is_none());
        assert!(decompose(0.10, 0.10, 0.0).is_none());
        assert!(decompose(f64::NAN, 0.10, 0.10).is_none());
    }

    #[test]
    fn symmetric_smile_yields_zero_rr() {
        // σ_25C == σ_25P → RR = 0, BF measures wing-vs-ATM premium.
        let r = decompose(0.12, 0.12, 0.10).unwrap();
        assert!(r.risk_reversal.abs() < 1e-12);
        assert!((r.butterfly - 0.02).abs() < 1e-12);
    }

    #[test]
    fn put_skew_yields_negative_rr() {
        // Equity-style skew: puts > calls → RR negative.
        let r = decompose(0.10, 0.14, 0.11).unwrap();
        assert!(r.risk_reversal < 0.0);
        assert!(r.skew_zscore < 0.0);
    }

    #[test]
    fn call_skew_yields_positive_rr() {
        let r = decompose(0.14, 0.10, 0.11).unwrap();
        assert!(r.risk_reversal > 0.0);
        assert!(r.skew_zscore > 0.0);
    }

    #[test]
    fn roundtrip_decompose_reconstruct() {
        let c = 0.135;
        let p = 0.115;
        let atm = 0.12;
        let d = decompose(c, p, atm).unwrap();
        let w = from_atm_rr_bf(d.atm, d.risk_reversal, d.butterfly).unwrap();
        assert!((w.sigma_25_call - c).abs() < 1e-12);
        assert!((w.sigma_25_put - p).abs() < 1e-12);
    }

    #[test]
    fn reconstruct_rejects_invalid_inputs() {
        assert!(from_atm_rr_bf(0.0, 0.01, 0.01).is_none());
        assert!(from_atm_rr_bf(f64::NAN, 0.01, 0.01).is_none());
        // Pathological large negative RR that would push 25P below zero.
        assert!(from_atm_rr_bf(0.10, -1.0, 0.0).is_none());
    }

    #[test]
    fn butterfly_zero_when_atm_equals_wing_avg() {
        let r = decompose(0.12, 0.10, 0.11).unwrap();
        assert!((r.butterfly).abs() < 1e-12);
    }
}
