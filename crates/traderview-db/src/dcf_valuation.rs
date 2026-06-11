//! Two-stage discounted cash flow valuation.
//!
//! Classic intrinsic-value calculator: project free cash flow through
//! an explicit growth stage, then capitalize the terminal year with the
//! Gordon Growth model, discount everything to present, subtract net
//! debt, divide by shares.
//!
//! ```text
//! PV_stage1   = Σ_{t=1..n}  FCF₀·(1+g)^t / (1+r)^t
//! TV          = FCF_n·(1+g_term) / (r − g_term)
//! PV_terminal = TV / (1+r)^n
//! equity      = PV_stage1 + PV_terminal − net_debt
//! intrinsic   = equity / shares_outstanding
//! ```
//!
//! Pure compute — no I/O. The route layer passes user inputs straight
//! through; the only data dependency is the optional current price for
//! the upside calc, which the frontend already has from the quote.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct DcfInput {
    /// Trailing free cash flow, in dollars (e.g. 99_584_000_000.0).
    pub fcf_usd: f64,
    /// Stage-1 annual growth rate percent (e.g. 8.0 = 8%/yr).
    pub growth_pct: f64,
    /// Stage-1 length in years (typically 5 or 10).
    pub growth_years: u32,
    /// Terminal (perpetuity) growth percent. Must be < discount rate.
    pub terminal_growth_pct: f64,
    /// Discount rate percent (WACC or required return, e.g. 10.0).
    pub discount_rate_pct: f64,
    /// Total debt − cash, in dollars. Negative = net cash.
    pub net_debt_usd: f64,
    /// Diluted shares outstanding.
    pub shares_outstanding: f64,
    /// Optional current price for the upside calculation.
    pub current_price: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DcfYearRow {
    pub year: u32,
    pub fcf_usd: f64,
    pub present_value_usd: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DcfReport {
    pub pv_stage1_usd: f64,
    pub terminal_value_usd: f64,
    pub pv_terminal_usd: f64,
    pub enterprise_value_usd: f64,
    pub equity_value_usd: f64,
    pub intrinsic_per_share: f64,
    /// (intrinsic − price) / price × 100 when current_price provided.
    pub upside_pct: Option<f64>,
    /// "undervalued" | "fair" | "overvalued" using a ±10% fair band.
    pub verdict: Option<&'static str>,
    pub yearly: Vec<DcfYearRow>,
}

#[derive(Debug, thiserror::Error)]
pub enum DcfError {
    #[error("discount rate must exceed terminal growth ({discount}% ≤ {terminal}%)")]
    RateInversion { discount: f64, terminal: f64 },
    #[error("shares outstanding must be positive")]
    NoShares,
    #[error("growth_years must be 1..=50")]
    BadYears,
    #[error("inputs must be finite numbers")]
    NonFinite,
}

pub fn compute(input: &DcfInput) -> Result<DcfReport, DcfError> {
    let r = input.discount_rate_pct / 100.0;
    let g = input.growth_pct / 100.0;
    let gt = input.terminal_growth_pct / 100.0;
    for v in [
        input.fcf_usd,
        r,
        g,
        gt,
        input.net_debt_usd,
        input.shares_outstanding,
    ] {
        if !v.is_finite() {
            return Err(DcfError::NonFinite);
        }
    }
    if r <= gt {
        return Err(DcfError::RateInversion {
            discount: input.discount_rate_pct,
            terminal: input.terminal_growth_pct,
        });
    }
    if input.shares_outstanding <= 0.0 {
        return Err(DcfError::NoShares);
    }
    if input.growth_years == 0 || input.growth_years > 50 {
        return Err(DcfError::BadYears);
    }

    let mut pv_stage1 = 0.0_f64;
    let mut fcf = input.fcf_usd;
    let mut yearly = Vec::with_capacity(input.growth_years as usize);
    for t in 1..=input.growth_years {
        fcf *= 1.0 + g;
        let pv = fcf / (1.0 + r).powi(t as i32);
        pv_stage1 += pv;
        yearly.push(DcfYearRow {
            year: t,
            fcf_usd: fcf,
            present_value_usd: pv,
        });
    }
    let terminal_value = fcf * (1.0 + gt) / (r - gt);
    let pv_terminal = terminal_value / (1.0 + r).powi(input.growth_years as i32);
    let enterprise = pv_stage1 + pv_terminal;
    let equity = enterprise - input.net_debt_usd;
    let intrinsic = equity / input.shares_outstanding;

    let (upside_pct, verdict) = match input.current_price {
        Some(p) if p > 0.0 => {
            let up = (intrinsic - p) / p * 100.0;
            let v = if up > 10.0 {
                "undervalued"
            } else if up < -10.0 {
                "overvalued"
            } else {
                "fair"
            };
            (Some(up), Some(v))
        }
        _ => (None, None),
    };

    Ok(DcfReport {
        pv_stage1_usd: pv_stage1,
        terminal_value_usd: terminal_value,
        pv_terminal_usd: pv_terminal,
        enterprise_value_usd: enterprise,
        equity_value_usd: equity,
        intrinsic_per_share: intrinsic,
        upside_pct,
        verdict,
        yearly,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> DcfInput {
        DcfInput {
            fcf_usd: 100.0,
            growth_pct: 0.0,
            growth_years: 5,
            terminal_growth_pct: 0.0,
            discount_rate_pct: 10.0,
            net_debt_usd: 0.0,
            shares_outstanding: 1.0,
            current_price: None,
        }
    }

    #[test]
    fn zero_growth_matches_annuity_plus_perpetuity() {
        // FCF=100 flat, r=10%: PV of 5-yr annuity = 100 × 3.79079 ≈ 379.08;
        // TV = 100/0.10 = 1000 discounted 5y → 620.92. Total ≈ 1000.
        // (Flat perpetuity at 10% is exactly 1000 — the split must sum to it.)
        let r = compute(&base()).unwrap();
        assert!((r.equity_value_usd - 1000.0).abs() < 0.5, "{}", r.equity_value_usd);
    }

    #[test]
    fn rate_inversion_rejected() {
        let mut i = base();
        i.terminal_growth_pct = 12.0; // > discount 10%
        assert!(matches!(compute(&i), Err(DcfError::RateInversion { .. })));
    }

    #[test]
    fn net_debt_reduces_equity() {
        let mut i = base();
        i.net_debt_usd = 200.0;
        let r = compute(&i).unwrap();
        assert!((r.equity_value_usd - 800.0).abs() < 0.5);
    }

    #[test]
    fn upside_and_verdict() {
        let mut i = base();
        i.current_price = Some(500.0); // intrinsic ~1000 → ~+100% upside
        let r = compute(&i).unwrap();
        assert_eq!(r.verdict, Some("undervalued"));
        assert!(r.upside_pct.unwrap() > 90.0);
        i.current_price = Some(2000.0); // overvalued
        let r2 = compute(&i).unwrap();
        assert_eq!(r2.verdict, Some("overvalued"));
    }

    #[test]
    fn zero_shares_rejected() {
        let mut i = base();
        i.shares_outstanding = 0.0;
        assert!(matches!(compute(&i), Err(DcfError::NoShares)));
    }
}
