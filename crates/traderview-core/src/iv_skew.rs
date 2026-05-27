//! Implied-volatility skew / smile analyzer.
//!
//! Equidistant-OTM-IV comparison:
//!   - **Put skew (P-C IV)**: 25-delta put IV minus 25-delta call IV.
//!     Positive = puts more expensive than equidistant calls — fear in
//:     the market.
//!   - **Smile**: ATM IV vs OTM IV in both wings. Higher OTM IV = "smile"
//!     (typical for equities); flatter = "smirk" (rare; signals
//!     calm market).
//!
//! Pure compute. Caller supplies the chain's IV by strike.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IvByStrike {
    pub strike: f64,
    pub call_iv: f64,
    pub put_iv: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkewReport {
    pub atm_iv: f64,
    pub put_25d_iv: f64,
    pub call_25d_iv: f64,
    /// Put IV minus call IV at equidistant OTM strikes.
    pub put_call_skew: f64,
    /// Average of OTM wings minus ATM — positive = smile, negative = smirk.
    pub smile: f64,
    pub note: String,
}

/// Approximate "25-delta" as a strike `pct_distance` away from spot
/// (default 5% in each direction). Caller can override.
pub fn analyze(chain: &[IvByStrike], spot: f64, pct_distance: f64) -> SkewReport {
    let mut report = SkewReport::default();
    if chain.is_empty() || spot <= 0.0 { return report; }
    // Find ATM strike (closest to spot).
    let atm = chain.iter().min_by(|a, b| {
        (a.strike - spot).abs().partial_cmp(&(b.strike - spot).abs())
            .unwrap_or(std::cmp::Ordering::Equal)
    }).unwrap();
    report.atm_iv = (atm.call_iv + atm.put_iv) / 2.0;
    // OTM put target = spot × (1 - pct_distance). OTM call target = spot × (1 + pct_distance).
    let put_target = spot * (1.0 - pct_distance);
    let call_target = spot * (1.0 + pct_distance);
    let nearest = |target: f64| -> &IvByStrike {
        chain.iter().min_by(|a, b| {
            (a.strike - target).abs().partial_cmp(&(b.strike - target).abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        }).unwrap()
    };
    let otm_put = nearest(put_target);
    let otm_call = nearest(call_target);
    report.put_25d_iv = otm_put.put_iv;
    report.call_25d_iv = otm_call.call_iv;
    report.put_call_skew = report.put_25d_iv - report.call_25d_iv;
    report.smile = (report.put_25d_iv + report.call_25d_iv) / 2.0 - report.atm_iv;
    report.note = if report.put_call_skew > 0.02 {
        "puts notably bid over calls — fear pricing".into()
    } else if report.put_call_skew < -0.02 {
        "calls bid over puts — unusual (greed / takeover speculation)".into()
    } else {
        "neutral skew".into()
    };
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn iv(strike: f64, c: f64, p: f64) -> IvByStrike {
        IvByStrike { strike, call_iv: c, put_iv: p }
    }

    #[test]
    fn empty_returns_default() {
        let r = analyze(&[], 100.0, 0.05);
        assert_eq!(r.atm_iv, 0.0);
    }

    #[test]
    fn zero_spot_returns_default() {
        let r = analyze(&[iv(100.0, 0.30, 0.30)], 0.0, 0.05);
        assert_eq!(r.atm_iv, 0.0);
    }

    #[test]
    fn atm_iv_is_average_of_call_and_put_at_atm_strike() {
        let chain = vec![iv(100.0, 0.30, 0.40)];
        let r = analyze(&chain, 100.0, 0.05);
        assert_eq!(r.atm_iv, 0.35);
    }

    #[test]
    fn put_call_skew_positive_when_otm_put_iv_higher() {
        // Classic equity-index skew: lower strikes have higher IV.
        let chain = vec![
            iv(95.0,  0.35, 0.45),    // OTM put
            iv(100.0, 0.30, 0.30),    // ATM
            iv(105.0, 0.25, 0.20),    // OTM call
        ];
        let r = analyze(&chain, 100.0, 0.05);
        assert!(r.put_call_skew > 0.0,
            "put IV (45%) > call IV (25%) at equidistant 5% OTM → positive skew");
        assert!(r.note.contains("fear"));
    }

    #[test]
    fn put_call_skew_negative_when_call_iv_higher() {
        // Takeover speculation: calls bid over puts.
        let chain = vec![
            iv(95.0,  0.20, 0.20),
            iv(100.0, 0.25, 0.25),
            iv(105.0, 0.45, 0.20),    // call bid
        ];
        let r = analyze(&chain, 100.0, 0.05);
        assert!(r.put_call_skew < 0.0);
        assert!(r.note.contains("greed") || r.note.contains("takeover"));
    }

    #[test]
    fn neutral_skew_when_put_call_iv_match() {
        let chain = vec![
            iv(95.0,  0.30, 0.30),
            iv(100.0, 0.30, 0.30),
            iv(105.0, 0.30, 0.30),
        ];
        let r = analyze(&chain, 100.0, 0.05);
        assert!(r.put_call_skew.abs() < 0.001);
        assert_eq!(r.note, "neutral skew");
    }

    #[test]
    fn smile_positive_when_otm_wings_higher_than_atm() {
        let chain = vec![
            iv(95.0,  0.40, 0.40),    // OTM put wing
            iv(100.0, 0.25, 0.25),    // ATM
            iv(105.0, 0.40, 0.40),    // OTM call wing
        ];
        let r = analyze(&chain, 100.0, 0.05);
        // (call + put) / 2 - ATM = 40% - 25% = 15% smile.
        assert!((r.smile - 0.15).abs() < 1e-9);
    }

    #[test]
    fn smile_negative_when_wings_below_atm() {
        let chain = vec![
            iv(95.0,  0.20, 0.20),
            iv(100.0, 0.30, 0.30),    // ATM higher than wings
            iv(105.0, 0.20, 0.20),
        ];
        let r = analyze(&chain, 100.0, 0.05);
        assert!(r.smile < 0.0);
    }

    #[test]
    fn nearest_strike_picked_when_exact_match_absent() {
        // No strike at exactly 95 — uses nearest (95 not in chain, 90 is).
        let chain = vec![
            iv(90.0,  0.40, 0.40),
            iv(100.0, 0.30, 0.30),
            iv(110.0, 0.20, 0.20),
        ];
        let r = analyze(&chain, 100.0, 0.05);
        // Should still pick a strike for the put 25d.
        assert!(r.put_25d_iv > 0.0);
    }
}
