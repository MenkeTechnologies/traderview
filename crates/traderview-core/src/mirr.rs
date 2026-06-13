//! Modified internal rate of return (MIRR) — the IRR variant that fixes the
//! plain IRR's unrealistic assumption that interim cash flows are reinvested at
//! the IRR itself.
//!
//! Negative cash flows are discounted to today at a finance rate; positive cash
//! flows are compounded forward to the final period at a reinvestment rate; the
//! MIRR is the rate linking the two:
//!
//! ```text
//! MIRR = (FV of positive flows / |PV of negative flows|)^(1/n) − 1
//! ```
//!
//! Unlike `npv-irr`'s IRR there is exactly one MIRR (no multiple-root problem),
//! and it reflects rates a firm can actually earn and borrow at.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct MirrInput {
    /// Period cash flows; index 0 is today (usually the negative outlay).
    pub cash_flows_usd: Vec<f64>,
    /// Rate used to discount negative (outflow) cash flows, percent.
    pub finance_rate_pct: f64,
    /// Rate at which positive (inflow) cash flows are reinvested, percent.
    pub reinvestment_rate_pct: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct MirrResult {
    pub n_periods: usize,
    /// Present value of the outflows (negative).
    pub present_value_outflows_usd: f64,
    /// Future value of the inflows at the final period.
    pub future_value_inflows_usd: f64,
    /// Sum of the raw cash flows.
    pub net_undiscounted_usd: f64,
    /// None when there are no outflows or no inflows to link.
    pub mirr_pct: Option<f64>,
}

pub fn analyze(input: &MirrInput) -> MirrResult {
    let flows = &input.cash_flows_usd;
    let n = flows.len().saturating_sub(1);
    let fr = input.finance_rate_pct / 100.0;
    let rr = input.reinvestment_rate_pct / 100.0;

    let mut pv_out = 0.0;
    let mut fv_in = 0.0;
    let mut net = 0.0;
    for (t, &cf) in flows.iter().enumerate() {
        net += cf;
        if cf < 0.0 {
            pv_out += cf / (1.0 + fr).powi(t as i32);
        } else if cf > 0.0 {
            fv_in += cf * (1.0 + rr).powi((n - t) as i32);
        }
    }

    let mirr = if n > 0 && pv_out < 0.0 && fv_in > 0.0 {
        Some(((fv_in / -pv_out).powf(1.0 / n as f64) - 1.0) * 100.0)
    } else {
        None
    };

    MirrResult {
        n_periods: n,
        present_value_outflows_usd: pv_out,
        future_value_inflows_usd: fv_in,
        net_undiscounted_usd: net,
        mirr_pct: mirr,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-2
    }

    // Classic example: outlay then four inflows.
    fn base() -> MirrInput {
        MirrInput {
            cash_flows_usd: vec![-1000.0, 300.0, 400.0, 500.0, 600.0],
            finance_rate_pct: 10.0,
            reinvestment_rate_pct: 12.0,
        }
    }

    #[test]
    fn period_count() {
        assert_eq!(analyze(&base()).n_periods, 4);
    }

    #[test]
    fn present_value_of_outflows() {
        // Only the t=0 outlay → −1000.
        assert!(close(analyze(&base()).present_value_outflows_usd, -1000.0));
    }

    #[test]
    fn future_value_of_inflows() {
        // 300·1.12³ + 400·1.12² + 500·1.12 + 600 = 2083.2384.
        assert!(close(analyze(&base()).future_value_inflows_usd, 2083.2384));
    }

    #[test]
    fn mirr_value() {
        // (2083.2384 / 1000)^(1/4) − 1 ≈ 20.139%.
        assert!((analyze(&base()).mirr_pct.unwrap() - 20.1392).abs() < 1e-2);
    }

    #[test]
    fn net_undiscounted() {
        // −1000 + 300 + 400 + 500 + 600 = 800.
        assert!(close(analyze(&base()).net_undiscounted_usd, 800.0));
    }

    #[test]
    fn multiple_outflows() {
        let r = analyze(&MirrInput {
            cash_flows_usd: vec![-1000.0, -500.0, 600.0, 800.0],
            finance_rate_pct: 10.0,
            reinvestment_rate_pct: 12.0,
        });
        // PV out = −1000 − 500/1.1 = −1454.5455; FV in = 600·1.12 + 800 = 1472.
        assert!(close(r.present_value_outflows_usd, -1454.5455));
        assert!(close(r.future_value_inflows_usd, 1472.0));
        // (1472 / 1454.5455)^(1/3) − 1 ≈ 0.3984%.
        assert!((r.mirr_pct.unwrap() - 0.3984).abs() < 1e-2);
    }

    #[test]
    fn no_outflows_yields_none() {
        let r = analyze(&MirrInput {
            cash_flows_usd: vec![100.0, 200.0, 300.0],
            finance_rate_pct: 10.0,
            reinvestment_rate_pct: 12.0,
        });
        assert!(r.mirr_pct.is_none());
    }

    #[test]
    fn no_inflows_yields_none() {
        let r = analyze(&MirrInput {
            cash_flows_usd: vec![-1000.0, -200.0],
            finance_rate_pct: 10.0,
            reinvestment_rate_pct: 12.0,
        });
        assert!(r.mirr_pct.is_none());
    }

    #[test]
    fn single_flow_yields_none() {
        let r = analyze(&MirrInput {
            cash_flows_usd: vec![-1000.0],
            finance_rate_pct: 10.0,
            reinvestment_rate_pct: 12.0,
        });
        assert_eq!(r.n_periods, 0);
        assert!(r.mirr_pct.is_none());
    }
}
