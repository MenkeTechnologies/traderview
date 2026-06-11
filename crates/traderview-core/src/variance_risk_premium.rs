//! Variance risk premium — what option sellers are paid for bearing
//! variance risk:
//!
//!   VRP = σ²_implied − σ²_realized      (variance points, % units²)
//!
//! Persistently positive across equity indexes (Carr & Wu 2009) — the
//! systematic short-vol edge. Also reported as the plain vol spread
//! and the IV/RV ratio the vol-cone view uses.
//!
//! Pure compute. Companion to `vol_cone` (supplies the realized leg),
//! `variance_swap`.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct VrpReport {
    pub implied_vol_pct: f64,
    pub realized_vol_pct: f64,
    /// σ²_iv − σ²_rv in (%)² variance points.
    pub vrp_variance_points: f64,
    /// IV − RV in vol points.
    pub vol_spread_pct: f64,
    pub iv_rv_ratio: f64,
    /// "rich" ratio ≥ 1.2, "cheap" ≤ 0.8, else "fair".
    pub premium_regime: &'static str,
}

pub fn compute(implied_vol_pct: f64, realized_vol_pct: f64) -> Option<VrpReport> {
    if !implied_vol_pct.is_finite()
        || implied_vol_pct <= 0.0
        || !realized_vol_pct.is_finite()
        || realized_vol_pct <= 0.0
    {
        return None;
    }
    let ratio = implied_vol_pct / realized_vol_pct;
    Some(VrpReport {
        implied_vol_pct,
        realized_vol_pct,
        vrp_variance_points: implied_vol_pct * implied_vol_pct
            - realized_vol_pct * realized_vol_pct,
        vol_spread_pct: implied_vol_pct - realized_vol_pct,
        iv_rv_ratio: ratio,
        premium_regime: if ratio >= 1.2 {
            "rich"
        } else if ratio <= 0.8 {
            "cheap"
        } else {
            "fair"
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_hand_computed_premium() {
        // IV 20, RV 16: VRP = 400 − 256 = 144; spread 4; ratio 1.25.
        let r = compute(20.0, 16.0).unwrap();
        assert!((r.vrp_variance_points - 144.0).abs() < 1e-12);
        assert!((r.vol_spread_pct - 4.0).abs() < 1e-12);
        assert!((r.iv_rv_ratio - 1.25).abs() < 1e-12);
        assert_eq!(r.premium_regime, "rich");
    }

    #[test]
    fn negative_premium_reads_cheap() {
        // Post-crash regime: realized above implied.
        let r = compute(20.0, 30.0).unwrap();
        assert!(r.vrp_variance_points < 0.0);
        assert_eq!(r.premium_regime, "cheap");
        assert_eq!(compute(20.0, 19.0).unwrap().premium_regime, "fair");
    }

    #[test]
    fn hostile_inputs_return_none() {
        assert!(compute(0.0, 16.0).is_none());
        assert!(compute(20.0, -1.0).is_none());
        assert!(compute(f64::NAN, 16.0).is_none());
    }
}
