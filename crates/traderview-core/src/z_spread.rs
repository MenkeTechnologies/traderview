//! Zero-Volatility Spread (Z-Spread).
//!
//! The constant spread, added to every point on the spot curve, that
//! discounts a bond's cash flows back to its observed dirty price.
//! Found by Brent-bracketed bisection on:
//!
//!   PV(z) = Σ_i c_i · exp(−(r_i + z) · t_i)
//!   solve PV(z) = market_price
//!
//! Distinct from nominal yield spread (single yield-to-maturity gap)
//! and OAS (which prices out an embedded option). Z-spread captures
//! the full term-structure-adjusted spread for option-free bonds.
//!
//! All rates continuously compounded, in decimal (e.g. 0.05 = 5%).
//!
//! Pure compute. Companion to `nelson_siegel`, `yield_curve_bootstrap`,
//! `bond_convexity`, `key_rate_duration`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CashFlow {
    pub time_years: f64,
    pub amount: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SpotPoint {
    pub time_years: f64,
    pub spot_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ZSpreadReport {
    pub z_spread: f64,
    pub iterations: usize,
    pub residual: f64,
    pub model_price_at_solution: f64,
}

pub fn solve(
    cash_flows: &[CashFlow],
    spot_curve: &[SpotPoint],
    market_price: f64,
    tolerance: f64,
    max_iter: usize,
) -> Option<ZSpreadReport> {
    if cash_flows.is_empty() || spot_curve.len() < 2 || !market_price.is_finite()
        || market_price <= 0.0 || tolerance <= 0.0 || max_iter == 0
    {
        return None;
    }
    if cash_flows.iter().any(|c| !c.time_years.is_finite() || c.time_years < 0.0
        || !c.amount.is_finite()) { return None; }
    if spot_curve.iter().any(|s| !s.time_years.is_finite() || !s.spot_rate.is_finite()) {
        return None;
    }
    for w in spot_curve.windows(2) {
        if w[1].time_years <= w[0].time_years { return None; }
    }
    // PV is monotonically decreasing in z, so straight bisection.
    let mut lo = -0.5_f64;
    let mut hi = 0.5_f64;
    let pv_lo = pv_at_spread(cash_flows, spot_curve, lo)?;
    let pv_hi = pv_at_spread(cash_flows, spot_curve, hi)?;
    if !pv_lo.is_finite() || !pv_hi.is_finite() { return None; }
    // Expand bracket if needed.
    let mut pv_lo = pv_lo;
    let mut pv_hi = pv_hi;
    let mut expansions = 0_usize;
    while pv_lo < market_price && expansions < 10 {
        lo -= 0.5;
        pv_lo = pv_at_spread(cash_flows, spot_curve, lo)?;
        expansions += 1;
    }
    while pv_hi > market_price && expansions < 20 {
        hi += 0.5;
        pv_hi = pv_at_spread(cash_flows, spot_curve, hi)?;
        expansions += 1;
    }
    if pv_lo < market_price || pv_hi > market_price { return None; }
    let mut iter = 0_usize;
    let mut z = 0.0_f64;
    let mut pv = 0.0_f64;
    while iter < max_iter {
        z = 0.5 * (lo + hi);
        pv = pv_at_spread(cash_flows, spot_curve, z)?;
        if (pv - market_price).abs() < tolerance { break; }
        if pv > market_price { lo = z; } else { hi = z; }
        iter += 1;
    }
    Some(ZSpreadReport {
        z_spread: z,
        iterations: iter,
        residual: pv - market_price,
        model_price_at_solution: pv,
    })
}

fn pv_at_spread(cf: &[CashFlow], curve: &[SpotPoint], z: f64) -> Option<f64> {
    let mut pv = 0.0_f64;
    for c in cf {
        let r = interp_rate(c.time_years, curve)?;
        let total = r + z;
        pv += c.amount * (-total * c.time_years).exp();
    }
    Some(pv)
}

fn interp_rate(t: f64, curve: &[SpotPoint]) -> Option<f64> {
    if curve.is_empty() { return None; }
    if t <= curve[0].time_years { return Some(curve[0].spot_rate); }
    if t >= curve[curve.len() - 1].time_years {
        return Some(curve[curve.len() - 1].spot_rate);
    }
    for w in curve.windows(2) {
        if t >= w[0].time_years && t <= w[1].time_years {
            let span = w[1].time_years - w[0].time_years;
            if span <= 0.0 { return Some(w[0].spot_rate); }
            let frac = (t - w[0].time_years) / span;
            return Some(w[0].spot_rate + frac * (w[1].spot_rate - w[0].spot_rate));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ten_year_5pct_bond() -> Vec<CashFlow> {
        let mut cf = Vec::new();
        for t in 1..=10 {
            cf.push(CashFlow { time_years: t as f64, amount: 5.0 });
        }
        cf.last_mut().unwrap().amount = 105.0;
        cf
    }

    fn flat_curve(rate: f64) -> Vec<SpotPoint> {
        vec![
            SpotPoint { time_years: 0.5, spot_rate: rate },
            SpotPoint { time_years: 2.0, spot_rate: rate },
            SpotPoint { time_years: 5.0, spot_rate: rate },
            SpotPoint { time_years: 10.0, spot_rate: rate },
        ]
    }

    #[test]
    fn invalid_inputs_return_none() {
        let cf = ten_year_5pct_bond();
        let curve = flat_curve(0.05);
        assert!(solve(&[], &curve, 100.0, 1e-8, 100).is_none());
        assert!(solve(&cf, &curve, 0.0, 1e-8, 100).is_none());
        assert!(solve(&cf, &curve, -1.0, 1e-8, 100).is_none());
        assert!(solve(&cf, &curve, 100.0, 0.0, 100).is_none());
        assert!(solve(&cf, &curve, 100.0, 1e-8, 0).is_none());
        assert!(solve(&cf, &[], 100.0, 1e-8, 100).is_none());
    }

    #[test]
    fn non_monotonic_curve_rejected() {
        let bad = vec![
            SpotPoint { time_years: 5.0, spot_rate: 0.05 },
            SpotPoint { time_years: 2.0, spot_rate: 0.05 },
        ];
        let cf = ten_year_5pct_bond();
        assert!(solve(&cf, &bad, 100.0, 1e-8, 100).is_none());
    }

    #[test]
    fn at_par_yields_zero_z_spread() {
        // 10y 5% bond priced at par with 5% flat curve → z = 0.
        // Use the *continuously-compounded* PV at flat curve = 5%.
        // PV(0) = sum c · exp(−0.05 · t); compute it, then solve back.
        let cf = ten_year_5pct_bond();
        let curve = flat_curve(0.05);
        let pv0 = pv_at_spread(&cf, &curve, 0.0).unwrap();
        let r = solve(&cf, &curve, pv0, 1e-10, 200).unwrap();
        assert!(r.z_spread.abs() < 1e-6, "expected ~0, got {}", r.z_spread);
    }

    #[test]
    fn below_par_yields_positive_z_spread() {
        // Same bond priced below the flat-curve PV → spread must widen.
        let cf = ten_year_5pct_bond();
        let curve = flat_curve(0.05);
        let pv0 = pv_at_spread(&cf, &curve, 0.0).unwrap();
        let r = solve(&cf, &curve, pv0 - 5.0, 1e-8, 200).unwrap();
        assert!(r.z_spread > 0.0, "expected positive z-spread, got {}", r.z_spread);
    }

    #[test]
    fn above_par_yields_negative_z_spread() {
        let cf = ten_year_5pct_bond();
        let curve = flat_curve(0.05);
        let pv0 = pv_at_spread(&cf, &curve, 0.0).unwrap();
        let r = solve(&cf, &curve, pv0 + 5.0, 1e-8, 200).unwrap();
        assert!(r.z_spread < 0.0, "expected negative z-spread, got {}", r.z_spread);
    }

    #[test]
    fn residual_within_tolerance() {
        let cf = ten_year_5pct_bond();
        let curve = flat_curve(0.05);
        let pv0 = pv_at_spread(&cf, &curve, 0.0).unwrap();
        let r = solve(&cf, &curve, pv0 - 2.0, 1e-6, 200).unwrap();
        assert!(r.residual.abs() < 1e-6);
    }
}
