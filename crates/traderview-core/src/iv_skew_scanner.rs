//! Implied-Volatility Skew Scanner.
//!
//! For each symbol's front-month option chain, computes standard
//! skew metrics:
//!
//!   - **25-delta risk-reversal** = IV(25Δ call) − IV(25Δ put)
//!     Sign: > 0 = upside-skewed (call vol > put vol), < 0 = put skew.
//!
//!   - **25-delta butterfly** = 0.5·(IV(25Δ call) + IV(25Δ put)) − IV(ATM)
//!     Magnitude: 0 = flat smile, positive = U-shaped smile.
//!
//!   - **Slope (per-strike-pct)**: regression slope of IV on
//!     log-moneyness, restricted to puts in the [0.85, 1.0] strike range.
//!     Equity bias is typically negative (downside puts trade richer).
//!
//! Flags symbols whose skew exceeds configurable thresholds.
//!
//! Pure compute. Caller supplies the (already-fit) IV strip per
//! symbol — pricing options is out of scope; that's `black76`,
//! `iv_solver`, etc.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrikeIv {
    pub strike: f64,
    pub call_iv: Option<f64>,
    pub put_iv: Option<f64>,
    pub delta_25_call: bool,
    pub delta_25_put: bool,
    pub atm: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolIvStrip {
    pub symbol: String,
    pub spot: f64,
    pub strikes: Vec<StrikeIv>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkewHit {
    pub symbol: String,
    pub risk_reversal_25d: f64,
    pub butterfly_25d: f64,
    pub put_wing_slope: f64,
    pub atm_iv: f64,
    pub n_strikes_used: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub min_strikes_in_wing: usize,
    pub min_abs_risk_reversal: f64,
}

impl Default for Config {
    fn default() -> Self {
        Self { min_strikes_in_wing: 3, min_abs_risk_reversal: 0.02 }
    }
}

pub fn scan(strips: &[SymbolIvStrip], cfg: &Config) -> Vec<SkewHit> {
    let mut hits = Vec::new();
    for strip in strips {
        if !strip.spot.is_finite() || strip.spot <= 0.0 || strip.strikes.is_empty() { continue; }
        // Locate ATM, 25Δ call, 25Δ put.
        let atm = strip.strikes.iter().find(|s| s.atm);
        let d25_call = strip.strikes.iter().find(|s| s.delta_25_call);
        let d25_put = strip.strikes.iter().find(|s| s.delta_25_put);
        let (Some(atm), Some(c25), Some(p25)) = (atm, d25_call, d25_put) else { continue };
        let Some(atm_iv) = atm.call_iv.or(atm.put_iv) else { continue };
        let Some(c25_iv) = c25.call_iv else { continue };
        let Some(p25_iv) = p25.put_iv else { continue };
        if !atm_iv.is_finite() || !c25_iv.is_finite() || !p25_iv.is_finite() { continue; }
        let rr = c25_iv - p25_iv;
        let butterfly = 0.5 * (c25_iv + p25_iv) - atm_iv;
        // Put-wing slope: regress put IV on log-moneyness for strikes in
        // [0.85, 1.0] · spot.
        let wing_pts: Vec<(f64, f64)> = strip.strikes.iter().filter_map(|s| {
            let iv = s.put_iv?;
            if !iv.is_finite() { return None; }
            let m = s.strike / strip.spot;
            if !(0.85..=1.0).contains(&m) { return None; }
            Some((m.ln(), iv))
        }).collect();
        let slope = if wing_pts.len() >= cfg.min_strikes_in_wing {
            ols_slope(&wing_pts).unwrap_or(0.0)
        } else { 0.0 };
        if rr.abs() < cfg.min_abs_risk_reversal { continue; }
        hits.push(SkewHit {
            symbol: strip.symbol.clone(),
            risk_reversal_25d: rr,
            butterfly_25d: butterfly,
            put_wing_slope: slope,
            atm_iv,
            n_strikes_used: strip.strikes.len(),
        });
    }
    hits.sort_by(|a, b| b.risk_reversal_25d.abs()
        .partial_cmp(&a.risk_reversal_25d.abs())
        .unwrap_or(std::cmp::Ordering::Equal));
    hits
}

fn ols_slope(pts: &[(f64, f64)]) -> Option<f64> {
    let n = pts.len() as f64;
    if n < 2.0 { return None; }
    let x_mean: f64 = pts.iter().map(|(x, _)| x).sum::<f64>() / n;
    let y_mean: f64 = pts.iter().map(|(_, y)| y).sum::<f64>() / n;
    let mut sxx = 0.0_f64;
    let mut sxy = 0.0_f64;
    for (x, y) in pts {
        let dx = x - x_mean;
        sxx += dx * dx;
        sxy += dx * (y - y_mean);
    }
    if sxx <= 0.0 { return None; }
    Some(sxy / sxx)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strip(sym: &str, spot: f64, strikes: Vec<StrikeIv>) -> SymbolIvStrip {
        SymbolIvStrip { symbol: sym.into(), spot, strikes }
    }

    fn k(strike: f64, c: Option<f64>, p: Option<f64>,
         atm: bool, d25c: bool, d25p: bool) -> StrikeIv {
        StrikeIv {
            strike, call_iv: c, put_iv: p, atm, delta_25_call: d25c, delta_25_put: d25p,
        }
    }

    #[test]
    fn empty_returns_empty() {
        assert!(scan(&[], &Config::default()).is_empty());
    }

    #[test]
    fn missing_atm_or_25_delta_filtered() {
        let s = strip("AAA", 100.0, vec![
            k(95.0, Some(0.20), Some(0.22), false, false, false),
            k(105.0, Some(0.20), Some(0.18), false, false, false),
        ]);
        assert!(scan(&[s], &Config::default()).is_empty());
    }

    #[test]
    fn put_skew_yields_negative_risk_reversal() {
        // 25Δ put trades richer than 25Δ call (typical equity).
        let s = strip("AAA", 100.0, vec![
            k(85.0, None, Some(0.35), false, false, true),         // 25Δ put
            k(100.0, Some(0.20), Some(0.20), true, false, false),    // ATM
            k(115.0, Some(0.18), None, false, true, false),         // 25Δ call
        ]);
        let hits = scan(&[s], &Config::default());
        assert_eq!(hits.len(), 1);
        let h = &hits[0];
        assert!(h.risk_reversal_25d < 0.0);
        // ATM iv = 0.20.
        assert!((h.atm_iv - 0.20).abs() < 1e-9);
    }

    #[test]
    fn call_skew_yields_positive_risk_reversal() {
        // Upside-skewed: 25Δ call IV > 25Δ put IV.
        let s = strip("AAA", 100.0, vec![
            k(85.0, None, Some(0.18), false, false, true),
            k(100.0, Some(0.20), Some(0.20), true, false, false),
            k(115.0, Some(0.30), None, false, true, false),
        ]);
        let hits = scan(&[s], &Config::default());
        assert_eq!(hits.len(), 1);
        assert!(hits[0].risk_reversal_25d > 0.0);
    }

    #[test]
    fn butterfly_positive_for_smile() {
        // Wings higher than ATM → positive butterfly. Use asymmetric RR
        // so the entry survives the default risk-reversal threshold filter.
        let s = strip("AAA", 100.0, vec![
            k(85.0, None, Some(0.30), false, false, true),
            k(100.0, Some(0.15), Some(0.15), true, false, false),
            k(115.0, Some(0.35), None, false, true, false),
        ]);
        let hits = scan(&[s], &Config::default());
        assert!(!hits.is_empty(), "expected smile to produce a hit");
        let h = &hits[0];
        assert!(h.butterfly_25d > 0.0,
            "smile should yield positive butterfly, got {}", h.butterfly_25d);
    }

    #[test]
    fn small_rr_filtered() {
        // RR = 0.005 below default 0.02 threshold.
        let s = strip("AAA", 100.0, vec![
            k(85.0, None, Some(0.20), false, false, true),
            k(100.0, Some(0.20), Some(0.20), true, false, false),
            k(115.0, Some(0.205), None, false, true, false),
        ]);
        assert!(scan(&[s], &Config::default()).is_empty());
    }

    #[test]
    fn sorted_by_abs_risk_reversal_descending() {
        let s1 = strip("A", 100.0, vec![
            k(85.0, None, Some(0.30), false, false, true),
            k(100.0, Some(0.20), Some(0.20), true, false, false),
            k(115.0, Some(0.10), None, false, true, false),
        ]);
        let s2 = strip("B", 100.0, vec![
            k(85.0, None, Some(0.25), false, false, true),
            k(100.0, Some(0.20), Some(0.20), true, false, false),
            k(115.0, Some(0.20), None, false, true, false),
        ]);
        let hits = scan(&[s1, s2], &Config::default());
        assert_eq!(hits.len(), 2);
        assert!(hits[0].risk_reversal_25d.abs() > hits[1].risk_reversal_25d.abs());
    }

    #[test]
    fn put_wing_slope_computed_when_enough_strikes() {
        let s = strip("AAA", 100.0, vec![
            k(86.0, None, Some(0.40), false, false, false),
            k(90.0, None, Some(0.35), false, false, true),
            k(95.0, None, Some(0.30), false, false, false),
            k(100.0, Some(0.20), Some(0.25), true, false, false),
            k(110.0, Some(0.22), None, false, true, false),
        ]);
        let cfg = Config { min_strikes_in_wing: 3, ..Default::default() };
        let hits = scan(&[s], &cfg);
        assert_eq!(hits.len(), 1);
        // Slope on (log-moneyness, IV) for monotonically rising put IVs in
        // the wing should be NEGATIVE (lower strikes → higher IV → as
        // log-moneyness increases toward 0, IV decreases).
        assert!(hits[0].put_wing_slope < 0.0,
            "put-wing slope should be negative, got {}", hits[0].put_wing_slope);
    }
}
