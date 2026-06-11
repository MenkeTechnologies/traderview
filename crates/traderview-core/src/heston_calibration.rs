//! Heston calibration — fit (v₀, κ, θ, σ, ρ) to market option quotes
//! by least-squares over model prices, using the shared Nelder-Mead
//! (`optimize`) on an unconstrained transform of the parameter space:
//!
//!   v₀ = x₀², κ = x₁², θ = x₂², σ = 0.01 + x₃², ρ = 0.999·tanh(x₄)
//!
//! so every simplex point maps to a valid model. Pricing inside the
//! objective runs the coarse-grid integrator (`compute_with_resolution`)
//! — the final report re-prices the fitted surface on the full grid.
//!
//! Pure compute. Companion to `heston`, `optimize`.

use crate::heston::{self, HestonInput};
use crate::optimize::nelder_mead;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Quote {
    pub strike: f64,
    pub time_to_expiry_years: f64,
    pub mid_price: f64,
    pub is_call: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CalibrationInput {
    pub spot: f64,
    #[serde(default)]
    pub risk_free_rate: f64,
    #[serde(default)]
    pub dividend_yield: f64,
    pub quotes: Vec<Quote>,
    /// Optimizer iterations (clamped 50..=2000, default 400).
    #[serde(default)]
    pub max_iter: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct QuoteFit {
    pub strike: f64,
    pub time_to_expiry_years: f64,
    pub market: f64,
    pub model: f64,
    pub error: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CalibrationReport {
    pub v0: f64,
    pub kappa: f64,
    pub theta: f64,
    pub vol_of_vol: f64,
    pub rho: f64,
    pub rmse: f64,
    pub feller_satisfied: bool,
    pub fits: Vec<QuoteFit>,
}

fn unpack(x: &[f64]) -> (f64, f64, f64, f64, f64) {
    (
        x[0] * x[0],          // v0
        x[1] * x[1],          // kappa
        x[2] * x[2],          // theta
        0.01 + x[3] * x[3],   // vol_of_vol
        0.999 * x[4].tanh(),  // rho
    )
}

fn model_price(
    inp: &CalibrationInput,
    q: &Quote,
    p: (f64, f64, f64, f64, f64),
    u_max: f64,
    n: usize,
) -> Option<f64> {
    let (v0, kappa, theta, sig, rho) = p;
    let rep = heston::compute_with_resolution(
        &HestonInput {
            spot: inp.spot,
            strike: q.strike,
            time_to_expiry_years: q.time_to_expiry_years,
            risk_free_rate: inp.risk_free_rate,
            dividend_yield: inp.dividend_yield,
            v0,
            kappa,
            theta,
            vol_of_vol: sig,
            rho,
        },
        u_max,
        n,
    )?;
    Some(if q.is_call { rep.call_price } else { rep.put_price })
}

pub fn calibrate(inp: &CalibrationInput) -> Option<CalibrationReport> {
    if !inp.spot.is_finite()
        || inp.spot <= 0.0
        || inp.quotes.is_empty()
        || inp.quotes.len() > 200
        || inp.quotes.iter().any(|q| {
            !q.strike.is_finite()
                || q.strike <= 0.0
                || !q.time_to_expiry_years.is_finite()
                || q.time_to_expiry_years <= 0.0
                || !q.mid_price.is_finite()
                || q.mid_price < 0.0
        })
    {
        return None;
    }
    let max_iter = inp.max_iter.unwrap_or(400).clamp(50, 2000);
    // Start: 20% vol everywhere, moderate reversion, equity-like skew.
    let start = [0.2_f64, 2.0_f64.sqrt(), 0.2, 0.3_f64.sqrt(), -0.5_f64.atanh()];
    let objective = |x: &[f64]| -> f64 {
        let p = unpack(x);
        let mut sse = 0.0;
        for q in &inp.quotes {
            match model_price(inp, q, p, 150.0, 400) {
                Some(m) => sse += (m - q.mid_price).powi(2),
                None => return 1e12,
            }
        }
        sse / inp.quotes.len() as f64
    };
    let (best, _) = nelder_mead(&start, 0.15, max_iter, objective);
    let p = unpack(&best);
    // Final fits on the full-resolution grid.
    let mut fits = Vec::with_capacity(inp.quotes.len());
    let mut sse = 0.0;
    for q in &inp.quotes {
        let m = model_price(inp, q, p, 200.0, 2000)?;
        sse += (m - q.mid_price).powi(2);
        fits.push(QuoteFit {
            strike: q.strike,
            time_to_expiry_years: q.time_to_expiry_years,
            market: q.mid_price,
            model: m,
            error: m - q.mid_price,
        });
    }
    let (v0, kappa, theta, sig, rho) = p;
    Some(CalibrationReport {
        v0,
        kappa,
        theta,
        vol_of_vol: sig,
        rho,
        rmse: (sse / inp.quotes.len() as f64).sqrt(),
        feller_satisfied: 2.0 * kappa * theta >= sig * sig,
        fits,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Quotes generated from known Heston parameters.
    fn synthetic_quotes(truth: &HestonInput) -> Vec<Quote> {
        [
            (85.0, 0.25, false),
            (95.0, 0.25, false),
            (100.0, 0.25, true),
            (105.0, 0.25, true),
            (115.0, 0.25, true),
            (90.0, 0.75, false),
            (100.0, 0.75, true),
            (110.0, 0.75, true),
        ]
        .iter()
        .map(|&(k, t, is_call)| {
            let rep = heston::compute(&HestonInput {
                strike: k,
                time_to_expiry_years: t,
                ..truth.clone()
            })
            .expect("truth params are valid");
            Quote {
                strike: k,
                time_to_expiry_years: t,
                mid_price: if is_call { rep.call_price } else { rep.put_price },
                is_call,
            }
        })
        .collect()
    }

    #[test]
    fn round_trip_recovers_the_surface() {
        let truth = HestonInput {
            spot: 100.0,
            strike: 100.0,
            time_to_expiry_years: 0.25,
            risk_free_rate: 0.03,
            dividend_yield: 0.0,
            v0: 0.05,
            kappa: 1.5,
            theta: 0.06,
            vol_of_vol: 0.4,
            rho: -0.6,
        };
        let quotes = synthetic_quotes(&truth);
        let rep = calibrate(&CalibrationInput {
            spot: 100.0,
            risk_free_rate: 0.03,
            dividend_yield: 0.0,
            quotes,
            max_iter: Some(400),
        })
        .expect("calibration");
        // The surface must reprice tightly even if individual params
        // trade off (kappa/theta are weakly identified on 8 quotes).
        assert!(rep.rmse < 0.05, "rmse {}", rep.rmse);
        // The well-identified parameters land near truth.
        assert!((rep.v0 - 0.05).abs() < 0.02, "v0 {}", rep.v0);
        assert!(rep.rho < -0.2, "rho {}", rep.rho);
    }

    #[test]
    fn hostile_inputs_return_none() {
        let empty = CalibrationInput {
            spot: 100.0,
            risk_free_rate: 0.0,
            dividend_yield: 0.0,
            quotes: vec![],
            max_iter: None,
        };
        assert!(calibrate(&empty).is_none());
        let bad = CalibrationInput {
            spot: 100.0,
            risk_free_rate: 0.0,
            dividend_yield: 0.0,
            quotes: vec![Quote {
                strike: -1.0,
                time_to_expiry_years: 0.25,
                mid_price: 5.0,
                is_call: true,
            }],
            max_iter: None,
        };
        assert!(calibrate(&bad).is_none());
    }
}
