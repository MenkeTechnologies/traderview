//! Implied dividend from put-call parity:
//!
//!   PV(divs) = S − (C − P) − K·e^{−rT}
//!
//! The option market's forward dividend forecast — frequently sharper
//! than analyst estimates around cut/raise events. Annualized to a
//! yield against spot.
//!
//! Pure compute. Inverse of the dividend input in
//! `conversion_reversal`; companion to `dividend_tracker` data.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ImpliedDividendReport {
    /// Present value of dividends to expiry, $/share.
    pub pv_dividends: f64,
    /// PV annualized as a simple yield on spot, %.
    pub implied_annual_yield_pct: f64,
    /// Negative PV usually means a hard-to-borrow stock — the borrow
    /// fee masquerades as a negative dividend in parity.
    pub negative_implies_borrow_cost: bool,
}

pub fn compute(
    spot: f64,
    strike: f64,
    call_price: f64,
    put_price: f64,
    time_to_expiry_years: f64,
    risk_free_rate: f64,
) -> Option<ImpliedDividendReport> {
    if ![spot, strike, call_price, put_price].iter().all(|v| v.is_finite())
        || spot <= 0.0
        || strike <= 0.0
        || call_price < 0.0
        || put_price < 0.0
        || !time_to_expiry_years.is_finite()
        || time_to_expiry_years <= 0.0
        || !risk_free_rate.is_finite()
    {
        return None;
    }
    let pv = spot - (call_price - put_price)
        - strike * (-risk_free_rate * time_to_expiry_years).exp();
    Some(ImpliedDividendReport {
        pv_dividends: pv,
        implied_annual_yield_pct: pv / spot / time_to_expiry_years * 100.0,
        negative_implies_borrow_cost: pv < 0.0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovers_the_dividend_baked_into_parity() {
        // Build a parity-consistent chain with PV(div) = 1.50:
        // C − P = S − PV(div) − K·e^{−rT}.
        let (s, k, t, r, pv) = (100.0_f64, 100.0_f64, 0.5_f64, 0.04_f64, 1.5_f64);
        let p = 4.0;
        let c = p + s - pv - k * (-r * t).exp();
        let rep = compute(s, k, c, p, t, r).unwrap();
        assert!((rep.pv_dividends - 1.5).abs() < 1e-12, "{}", rep.pv_dividends);
        // 1.5 over half a year on a $100 stock = 3%/yr.
        assert!((rep.implied_annual_yield_pct - 3.0).abs() < 1e-12);
        assert!(!rep.negative_implies_borrow_cost);
    }

    #[test]
    fn zero_dividend_chain_reads_zero() {
        let (s, k, t, r) = (100.0_f64, 100.0_f64, 0.5_f64, 0.04_f64);
        let p = 4.0;
        let c = p + s - k * (-r * t).exp();
        let rep = compute(s, k, c, p, t, r).unwrap();
        assert!(rep.pv_dividends.abs() < 1e-12);
    }

    #[test]
    fn hard_to_borrow_reads_negative() {
        // Hard-to-borrow names trade with the call RICH relative to
        // parity (shorting the synthetic costs the borrow fee), which
        // solves to a NEGATIVE implied dividend.
        let (s, k, t, r) = (100.0_f64, 100.0_f64, 0.5_f64, 0.04_f64);
        let p = 4.0;
        let c_rich = p + s - k * (-r * t).exp() + 2.0;
        let rep = compute(s, k, c_rich, p, t, r).unwrap();
        assert!((rep.pv_dividends + 2.0).abs() < 1e-12, "{}", rep.pv_dividends);
        assert!(rep.negative_implies_borrow_cost);
    }

    #[test]
    fn hostile_inputs_return_none() {
        assert!(compute(0.0, 100.0, 5.0, 4.0, 0.5, 0.04).is_none());
        assert!(compute(100.0, 100.0, -5.0, 4.0, 0.5, 0.04).is_none());
        assert!(compute(100.0, 100.0, 5.0, 4.0, 0.0, 0.04).is_none());
        assert!(compute(100.0, 100.0, f64::NAN, 4.0, 0.5, 0.04).is_none());
    }
}
