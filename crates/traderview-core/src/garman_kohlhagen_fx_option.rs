//! Garman-Kohlhagen FX Option Pricer — Black-Scholes adapted for two
//! interest rates (domestic `r_d`, foreign `r_f`).
//!
//!   d1 = (ln(S/K) + (r_d - r_f + σ²/2) T) / (σ √T)
//!   d2 = d1 - σ √T
//!   call = S · e^(-r_f T) · N(d1) - K · e^(-r_d T) · N(d2)
//!   put  = K · e^(-r_d T) · N(-d2) - S · e^(-r_f T) · N(-d1)
//!
//! where S is the spot FX rate (domestic per 1 unit of foreign).
//! Greeks (delta, gamma, vega, theta, rho_d, rho_f) reported in
//! per-spot-unit terms.
//!
//! Pure compute. Companion to `black_scholes`, `currency_exposure`,
//! `multi_leg_option_pricer`.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OptionKind { Call, Put }

#[derive(Debug)]
pub struct Report {
    pub price: f64,
    pub delta: f64,
    pub gamma: f64,
    pub vega: f64,
    pub theta: f64,
    pub rho_domestic: f64,
    pub rho_foreign: f64,
}

#[allow(clippy::too_many_arguments)]
pub fn compute(
    kind: OptionKind, spot: f64, strike: f64,
    t_years: f64, rate_dom: f64, rate_for: f64, sigma: f64,
) -> Option<Report> {
    let scalars = [spot, strike, t_years, rate_dom, rate_for, sigma];
    if scalars.iter().any(|x| !x.is_finite()) { return None; }
    if spot <= 0.0 || strike <= 0.0 || t_years < 0.0 || sigma < 0.0 { return None; }
    if t_years == 0.0 || sigma == 0.0 {
        let intrinsic = match kind {
            OptionKind::Call => (spot - strike).max(0.0),
            OptionKind::Put => (strike - spot).max(0.0),
        };
        let delta = match kind {
            OptionKind::Call => if spot > strike { 1.0 } else { 0.0 },
            OptionKind::Put => if spot < strike { -1.0 } else { 0.0 },
        };
        return Some(Report {
            price: intrinsic, delta, gamma: 0.0, vega: 0.0,
            theta: 0.0, rho_domestic: 0.0, rho_foreign: 0.0,
        });
    }
    let st = sigma * t_years.sqrt();
    let d1 = ((spot / strike).ln() + (rate_dom - rate_for + 0.5 * sigma * sigma) * t_years) / st;
    let d2 = d1 - st;
    let disc_d = (-rate_dom * t_years).exp();
    let disc_f = (-rate_for * t_years).exp();
    let nd1 = norm_cdf(d1);
    let nd2 = norm_cdf(d2);
    let pdf_d1 = norm_pdf(d1);
    let (price, delta, theta, rho_d, rho_f) = match kind {
        OptionKind::Call => {
            let price = spot * disc_f * nd1 - strike * disc_d * nd2;
            let delta = disc_f * nd1;
            let theta = -spot * disc_f * pdf_d1 * sigma / (2.0 * t_years.sqrt())
                + rate_for * spot * disc_f * nd1
                - rate_dom * strike * disc_d * nd2;
            let rho_d = strike * t_years * disc_d * nd2;
            let rho_f = -t_years * spot * disc_f * nd1;
            (price, delta, theta, rho_d, rho_f)
        }
        OptionKind::Put => {
            let nd1n = norm_cdf(-d1);
            let nd2n = norm_cdf(-d2);
            let price = strike * disc_d * nd2n - spot * disc_f * nd1n;
            let delta = -disc_f * nd1n;
            let theta = -spot * disc_f * pdf_d1 * sigma / (2.0 * t_years.sqrt())
                - rate_for * spot * disc_f * nd1n
                + rate_dom * strike * disc_d * nd2n;
            let rho_d = -strike * t_years * disc_d * nd2n;
            let rho_f = t_years * spot * disc_f * nd1n;
            (price, delta, theta, rho_d, rho_f)
        }
    };
    let gamma = disc_f * pdf_d1 / (spot * st);
    let vega = spot * disc_f * pdf_d1 * t_years.sqrt();
    Some(Report {
        price, delta, gamma, vega, theta,
        rho_domestic: rho_d, rho_foreign: rho_f,
    })
}

fn norm_pdf(x: f64) -> f64 {
    (1.0 / (std::f64::consts::TAU).sqrt()) * (-0.5 * x * x).exp()
}

fn norm_cdf(x: f64) -> f64 {
    let a1 = 0.254829592_f64;
    let a2 = -0.284496736;
    let a3 = 1.421413741;
    let a4 = -1.453152027;
    let a5 = 1.061405429;
    let p = 0.3275911_f64;
    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x_abs = (x / std::f64::consts::SQRT_2).abs();
    let t = 1.0 / (1.0 + p * x_abs);
    let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x_abs * x_abs).exp();
    0.5 * (1.0 + sign * y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_none() {
        assert!(compute(OptionKind::Call, -1.0, 100.0, 0.5, 0.05, 0.03, 0.1).is_none());
        assert!(compute(OptionKind::Call, 100.0, 0.0, 0.5, 0.05, 0.03, 0.1).is_none());
        assert!(compute(OptionKind::Call, 100.0, 100.0, -0.5, 0.05, 0.03, 0.1).is_none());
        assert!(compute(OptionKind::Call, 100.0, 100.0, 0.5, 0.05, 0.03, -0.1).is_none());
        assert!(compute(OptionKind::Call, f64::NAN, 100.0, 0.5, 0.05, 0.03, 0.1).is_none());
    }

    #[test]
    fn at_expiry_collapses_to_intrinsic() {
        let r = compute(OptionKind::Call, 1.20, 1.10, 0.0, 0.05, 0.03, 0.10).unwrap();
        assert!((r.price - 0.10).abs() < 1e-12);
    }

    #[test]
    fn put_call_parity_holds() {
        // c - p = S·e^(-r_f·T) - K·e^(-r_d·T)
        let s = 1.20; let k = 1.15; let t = 0.5;
        let r_d = 0.04; let r_f = 0.02; let sigma = 0.10;
        let c = compute(OptionKind::Call, s, k, t, r_d, r_f, sigma).unwrap();
        let p = compute(OptionKind::Put, s, k, t, r_d, r_f, sigma).unwrap();
        let parity = s * (-r_f * t).exp() - k * (-r_d * t).exp();
        assert!((c.price - p.price - parity).abs() < 1e-7);
    }

    #[test]
    fn call_delta_in_unit_interval() {
        let r = compute(OptionKind::Call, 1.20, 1.20, 0.5, 0.04, 0.02, 0.10).unwrap();
        assert!((0.0..=1.0).contains(&r.delta));
    }

    #[test]
    fn put_delta_in_negative_unit_interval() {
        let r = compute(OptionKind::Put, 1.20, 1.20, 0.5, 0.04, 0.02, 0.10).unwrap();
        assert!((-1.0..=0.0).contains(&r.delta));
    }

    #[test]
    fn deep_otm_call_near_zero() {
        let r = compute(OptionKind::Call, 1.00, 2.00, 0.1, 0.04, 0.02, 0.10).unwrap();
        assert!(r.price < 0.01);
    }

    #[test]
    fn vega_and_gamma_positive() {
        let r = compute(OptionKind::Call, 1.20, 1.20, 0.5, 0.04, 0.02, 0.10).unwrap();
        assert!(r.vega > 0.0);
        assert!(r.gamma > 0.0);
    }
}
