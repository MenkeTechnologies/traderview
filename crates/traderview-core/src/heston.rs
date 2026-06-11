//! Heston (1993) stochastic-volatility European option pricer.
//!
//!   dS = (r − q) S dt + √v S dW₁
//!   dv = κ(θ − v) dt + σ √v dW₂,   d⟨W₁,W₂⟩ = ρ dt
//!
//! Semi-analytic price via the two-probability characteristic-function
//! formulation in the Albrecher/Gatheral "little Heston trap" form
//! (numerically stable for long maturities):
//!
//!   C = S e^{−qT} P₁ − K e^{−rT} P₂
//!   Pⱼ = ½ + (1/π) ∫₀^∞ Re[ e^{−iu lnK} fⱼ(u) / (iu) ] du
//!
//! Composite-Simpson integration on u ∈ (0, 200]. A tiny self-contained
//! complex type keeps the crate dependency-free.
//!
//! Pure compute. Companion to `black76`, `bachelier`,
//! `garman_kohlhagen_fx_option`, `american_option_lsmc`.

use serde::{Deserialize, Serialize};

// ── minimal complex arithmetic ─────────────────────────────────────────

#[derive(Clone, Copy, Debug)]
struct Cx {
    re: f64,
    im: f64,
}

impl Cx {
    const fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }
    fn add(self, o: Cx) -> Cx {
        Cx::new(self.re + o.re, self.im + o.im)
    }
    fn sub(self, o: Cx) -> Cx {
        Cx::new(self.re - o.re, self.im - o.im)
    }
    fn mul(self, o: Cx) -> Cx {
        Cx::new(
            self.re * o.re - self.im * o.im,
            self.re * o.im + self.im * o.re,
        )
    }
    fn div(self, o: Cx) -> Cx {
        let den = o.re * o.re + o.im * o.im;
        Cx::new(
            (self.re * o.re + self.im * o.im) / den,
            (self.im * o.re - self.re * o.im) / den,
        )
    }
    fn scale(self, k: f64) -> Cx {
        Cx::new(self.re * k, self.im * k)
    }
    fn exp(self) -> Cx {
        let r = self.re.exp();
        Cx::new(r * self.im.cos(), r * self.im.sin())
    }
    fn ln(self) -> Cx {
        Cx::new(
            (self.re * self.re + self.im * self.im).sqrt().ln(),
            self.im.atan2(self.re),
        )
    }
    fn sqrt(self) -> Cx {
        let r = (self.re * self.re + self.im * self.im).sqrt().sqrt();
        let theta = self.im.atan2(self.re) / 2.0;
        Cx::new(r * theta.cos(), r * theta.sin())
    }
}

// ── model ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct HestonInput {
    pub spot: f64,
    pub strike: f64,
    pub time_to_expiry_years: f64,
    pub risk_free_rate: f64,
    /// Continuous dividend yield.
    pub dividend_yield: f64,
    /// Initial variance v₀ (vol² — 0.04 ⇒ 20% vol).
    pub v0: f64,
    /// Mean-reversion speed κ.
    pub kappa: f64,
    /// Long-run variance θ.
    pub theta: f64,
    /// Vol-of-vol σ.
    pub vol_of_vol: f64,
    /// Spot/vol correlation ρ ∈ [−1, 1].
    pub rho: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct HestonReport {
    pub call_price: f64,
    pub put_price: f64,
    pub p1: f64,
    pub p2: f64,
    /// 2κθ ≥ σ² — variance stays strictly positive when satisfied.
    pub feller_satisfied: bool,
}

/// Characteristic-function term fⱼ(u) in the little-trap form.
fn f_j(u: f64, j: u8, inp: &HestonInput) -> Cx {
    let (uj, bj) = if j == 1 {
        (0.5, inp.kappa - inp.rho * inp.vol_of_vol)
    } else {
        (-0.5, inp.kappa)
    };
    let a = inp.kappa * inp.theta;
    let sig = inp.vol_of_vol;
    let t = inp.time_to_expiry_years;
    let iu = Cx::new(0.0, u);
    let x = Cx::new(0.0, inp.spot.ln() * u); // iu·lnS

    // d = sqrt((ρσ iu − b)² − σ²(2 uⱼ iu − u²)); the subtrahend is
    // σ²·(−u² + 2uⱼu·i).
    let rsiu_b = Cx::new(-bj, inp.rho * sig * u);
    let inner = rsiu_b
        .mul(rsiu_b)
        .sub(Cx::new(-(u * u), 2.0 * uj * u).scale(sig * sig));
    let d = inner.sqrt();

    // Little trap: c = (b − ρσiu − d)/(b − ρσiu + d)
    let b_minus = Cx::new(bj, -inp.rho * sig * u);
    let num = b_minus.sub(d);
    let den = b_minus.add(d);
    let c = num.div(den);

    let e_dt = d.scale(-t).exp();
    let one = Cx::new(1.0, 0.0);
    let big_d = num
        .scale(1.0 / (sig * sig))
        .mul(one.sub(e_dt))
        .div(one.sub(c.mul(e_dt)));
    let log_term = one.sub(c.mul(e_dt)).div(one.sub(c)).ln();
    let big_c = iu
        .scale((inp.risk_free_rate - inp.dividend_yield) * t)
        .add(num.scale(t).sub(log_term.scale(2.0)).scale(a / (sig * sig)));

    big_c.add(big_d.scale(inp.v0)).add(x).exp()
}

/// Pⱼ via composite Simpson on u ∈ (ε, u_max].
fn p_j_with(j: u8, inp: &HestonInput, u_max: f64, n: usize) -> f64 {
    let ln_k = inp.strike.ln();
    let integrand = |u: f64| -> f64 {
        let phi = f_j(u, j, inp);
        // e^{−iu lnK} · φ / (iu)
        let e = Cx::new(0.0, -u * ln_k).exp();
        e.mul(phi).div(Cx::new(0.0, u)).re
    };
    let a = 1e-8;
    let h = (u_max - a) / n as f64;
    let mut sum = integrand(a) + integrand(u_max);
    for k in 1..n {
        let u = a + h * k as f64;
        sum += integrand(u) * if k % 2 == 1 { 4.0 } else { 2.0 };
    }
    0.5 + (sum * h / 3.0) / std::f64::consts::PI
}

/// Price with a caller-chosen integration grid — the calibrator runs
/// thousands of evaluations and uses a coarser (still Simpson) grid.
pub fn compute_with_resolution(
    inp: &HestonInput,
    u_max: f64,
    n: usize,
) -> Option<HestonReport> {
    compute_inner(inp, u_max, n.max(4) & !1)
}

pub fn compute(inp: &HestonInput) -> Option<HestonReport> {
    compute_inner(inp, 200.0, 2000)
}

fn compute_inner(inp: &HestonInput, u_max: f64, n: usize) -> Option<HestonReport> {
    if ![
        inp.spot,
        inp.strike,
        inp.time_to_expiry_years,
        inp.risk_free_rate,
        inp.dividend_yield,
        inp.v0,
        inp.kappa,
        inp.theta,
        inp.vol_of_vol,
        inp.rho,
    ]
    .iter()
    .all(|v| v.is_finite())
        || inp.spot <= 0.0
        || inp.strike <= 0.0
        || inp.time_to_expiry_years <= 0.0
        || inp.v0 < 0.0
        || inp.kappa <= 0.0
        || inp.theta < 0.0
        || inp.vol_of_vol <= 0.0
        || !(-1.0..=1.0).contains(&inp.rho)
    {
        return None;
    }
    let p1 = p_j_with(1, inp, u_max, n).clamp(0.0, 1.0);
    let p2 = p_j_with(2, inp, u_max, n).clamp(0.0, 1.0);
    let disc_s = inp.spot * (-inp.dividend_yield * inp.time_to_expiry_years).exp();
    let disc_k = inp.strike * (-inp.risk_free_rate * inp.time_to_expiry_years).exp();
    let call = (disc_s * p1 - disc_k * p2).max(0.0);
    let put = (call - disc_s + disc_k).max(0.0);
    Some(HestonReport {
        call_price: call,
        put_price: put,
        p1,
        p2,
        feller_satisfied: 2.0 * inp.kappa * inp.theta >= inp.vol_of_vol * inp.vol_of_vol,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn norm_cdf(x: f64) -> f64 {
        0.5 * (1.0 + erf(x / std::f64::consts::SQRT_2))
    }
    // Abramowitz-Stegun 7.1.26 — plenty for 1e-6-level test tolerance.
    fn erf(x: f64) -> f64 {
        let s = if x < 0.0 { -1.0 } else { 1.0 };
        let x = x.abs();
        let t = 1.0 / (1.0 + 0.3275911 * x);
        let y = 1.0
            - (((((1.061405429 * t - 1.453152027) * t) + 1.421413741) * t - 0.284496736) * t
                + 0.254829592)
                * t
                * (-x * x).exp();
        s * y
    }
    fn bs_call(s: f64, k: f64, t: f64, r: f64, q: f64, sigma: f64) -> f64 {
        let d1 = ((s / k).ln() + (r - q + sigma * sigma / 2.0) * t) / (sigma * t.sqrt());
        let d2 = d1 - sigma * t.sqrt();
        s * (-q * t).exp() * norm_cdf(d1) - k * (-r * t).exp() * norm_cdf(d2)
    }

    fn base() -> HestonInput {
        HestonInput {
            spot: 100.0,
            strike: 100.0,
            time_to_expiry_years: 0.5,
            risk_free_rate: 0.03,
            dividend_yield: 0.0,
            v0: 0.04,
            kappa: 2.0,
            theta: 0.04,
            vol_of_vol: 0.3,
            rho: -0.7,
        }
    }

    #[test]
    fn converges_to_black_scholes_when_vol_of_vol_vanishes() {
        // v0 = θ and σ→0 ⇒ variance pinned at 0.04 ⇒ BS at 20% vol.
        let mut inp = base();
        inp.vol_of_vol = 1e-4;
        inp.rho = 0.0;
        let r = compute(&inp).unwrap();
        let bs = bs_call(100.0, 100.0, 0.5, 0.03, 0.0, 0.2);
        assert!((r.call_price - bs).abs() < 1e-2, "{} vs {bs}", r.call_price);
    }

    #[test]
    fn put_call_parity_holds() {
        let r = compute(&base()).unwrap();
        let lhs = r.call_price - r.put_price;
        let rhs = 100.0 - 100.0 * (-0.03_f64 * 0.5).exp();
        assert!((lhs - rhs).abs() < 1e-9, "{lhs} vs {rhs}");
    }

    #[test]
    fn price_respects_no_arbitrage_bounds() {
        let r = compute(&base()).unwrap();
        let disc_k = 100.0 * (-0.03_f64 * 0.5).exp();
        assert!(r.call_price >= (100.0 - disc_k).max(0.0));
        assert!(r.call_price <= 100.0);
        assert!((0.0..=1.0).contains(&r.p1));
        assert!((0.0..=1.0).contains(&r.p2));
    }

    #[test]
    fn price_increases_with_initial_variance() {
        let lo = compute(&HestonInput { v0: 0.02, ..base() }).unwrap();
        let hi = compute(&HestonInput { v0: 0.09, ..base() }).unwrap();
        assert!(hi.call_price > lo.call_price);
    }

    #[test]
    fn negative_rho_skews_otm_puts_richer() {
        // ρ < 0 (equity-like) fattens the left tail: the OTM put at
        // K=80 should cost more than under ρ > 0.
        let neg = compute(&HestonInput { strike: 80.0, rho: -0.7, ..base() }).unwrap();
        let pos = compute(&HestonInput { strike: 80.0, rho: 0.7, ..base() }).unwrap();
        assert!(neg.put_price > pos.put_price, "{} vs {}", neg.put_price, pos.put_price);
    }

    #[test]
    fn feller_condition_flagged() {
        assert!(compute(&HestonInput { vol_of_vol: 0.3, ..base() }).unwrap().feller_satisfied); // 2·2·0.04 = 0.16 ≥ 0.09
        assert!(!compute(&HestonInput { vol_of_vol: 0.5, ..base() }).unwrap().feller_satisfied); // 0.16 < 0.25
    }

    #[test]
    fn hostile_inputs_return_none() {
        assert!(compute(&HestonInput { spot: 0.0, ..base() }).is_none());
        assert!(compute(&HestonInput { time_to_expiry_years: 0.0, ..base() }).is_none());
        assert!(compute(&HestonInput { kappa: 0.0, ..base() }).is_none());
        assert!(compute(&HestonInput { vol_of_vol: 0.0, ..base() }).is_none());
        assert!(compute(&HestonInput { rho: 1.5, ..base() }).is_none());
        assert!(compute(&HestonInput { v0: f64::NAN, ..base() }).is_none());
    }
}
