//! Time-value-of-money solver — the generic PV/FV/PMT/N/R engine of an HP-12C /
//! TI BA-II / Excel. Pick which variable to solve; supply the other four. The
//! governing equation (standard sign convention, outflows negative):
//!   PV·(1+r)^n + PMT·(1+r·w)·((1+r)^n − 1)/r + FV = 0
//! where r is the per-period rate (annual ÷ periods/year) and w is 1 for
//! annuity-due (payment at start of period) or 0 for ordinary. PV/FV/PMT/N have
//! closed forms; R has none, so it is found by bisection on the residual. For
//! the R solve, the result also reports nominal and effective annual rates.
//! Faithful port of the former client-side calculator. Pure compute, not advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct TvmInput {
    /// Which variable to solve: "fv", "pv", "pmt", "n", or "r".
    pub solve: String,
    pub periods_per_year: u32,
    /// True for annuity-due (payment at start of period).
    #[serde(default)]
    pub when_begin: bool,
    #[serde(default)]
    pub pv: f64,
    #[serde(default)]
    pub fv: f64,
    #[serde(default)]
    pub pmt: f64,
    #[serde(default)]
    pub n: f64,
    pub annual_rate_pct: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct TvmReport {
    pub solve: String,
    /// The solved value (dollars, period count, or per-period rate fraction for
    /// "r"). None when no closed-form / bracketed solution exists.
    pub answer: Option<f64>,
    /// R solve only: nominal annual rate (per-period × periods/year), %.
    pub nominal_annual_pct: Option<f64>,
    /// R solve only: effective annual rate (compounded), %.
    pub effective_annual_pct: Option<f64>,
    pub valid: bool,
}

fn round4(x: f64) -> f64 {
    (x * 10_000.0).round() / 10_000.0
}

fn round8(x: f64) -> f64 {
    (x * 1e8).round() / 1e8
}

fn solve_fv(pv: f64, pmt: f64, n: f64, r: f64, w: f64) -> f64 {
    if r == 0.0 {
        return -(pv + pmt * n);
    }
    let f = (1.0 + r).powf(n);
    -(pv * f + pmt * (1.0 + r * w) * (f - 1.0) / r)
}

fn solve_pv(fv: f64, pmt: f64, n: f64, r: f64, w: f64) -> f64 {
    if r == 0.0 {
        return -(fv + pmt * n);
    }
    let f = (1.0 + r).powf(n);
    -(fv + pmt * (1.0 + r * w) * (f - 1.0) / r) / f
}

fn solve_pmt(pv: f64, fv: f64, n: f64, r: f64, w: f64) -> f64 {
    if r == 0.0 {
        return -(pv + fv) / n;
    }
    let f = (1.0 + r).powf(n);
    -(pv * f + fv) * r / ((1.0 + r * w) * (f - 1.0))
}

fn solve_n(pv: f64, fv: f64, pmt: f64, r: f64, w: f64) -> Option<f64> {
    if r == 0.0 {
        if pmt == 0.0 {
            return None;
        }
        return Some(-(pv + fv) / pmt);
    }
    // (1+r)^n · (PV + A) = A − FV, with A = PMT·(1+r·w)/r.
    let a = pmt * (1.0 + r * w) / r;
    let num = a - fv;
    let den = pv + a;
    if den == 0.0 || num / den <= 0.0 {
        return None;
    }
    Some((num / den).ln() / (1.0 + r).ln())
}

fn solve_r(pv: f64, fv: f64, pmt: f64, n: f64, w: f64) -> Option<f64> {
    let residual = |r: f64| -> f64 {
        if r.abs() < 1e-12 {
            return pv + pmt * n + fv;
        }
        let f = (1.0 + r).powf(n);
        pv * f + pmt * (1.0 + r * w) * (f - 1.0) / r + fv
    };
    let mut lo = -0.99;
    let mut hi = 1.0;
    let mut fl = residual(lo);
    let mut fh = residual(hi);
    if fl * fh > 0.0 {
        let mut i = 0;
        while i < 20 && fl * fh > 0.0 {
            hi *= 2.0;
            fh = residual(hi);
            i += 1;
        }
        if fl * fh > 0.0 {
            return None;
        }
    }
    for _ in 0..200 {
        let mid = (lo + hi) / 2.0;
        let fm = residual(mid);
        if fm.abs() < 1e-9 {
            return Some(mid);
        }
        // Only the low-side residual `fl` drives the bisection decision; the
        // high-side value isn't read again, so it isn't tracked here.
        if fl * fm < 0.0 {
            hi = mid;
        } else {
            lo = mid;
            fl = fm;
        }
    }
    Some((lo + hi) / 2.0)
}

pub fn generate(i: &TvmInput) -> TvmReport {
    let ppy = i.periods_per_year.max(1);
    let r = i.annual_rate_pct / 100.0 / ppy as f64;
    let w = if i.when_begin { 1.0 } else { 0.0 };
    let solve = i.solve.trim().to_ascii_lowercase();

    let raw: Option<f64> = match solve.as_str() {
        "fv" => Some(solve_fv(i.pv, i.pmt, i.n, r, w)),
        "pv" => Some(solve_pv(i.fv, i.pmt, i.n, r, w)),
        "pmt" => Some(solve_pmt(i.pv, i.fv, i.n, r, w)),
        "n" => solve_n(i.pv, i.fv, i.pmt, r, w),
        "r" => solve_r(i.pv, i.fv, i.pmt, i.n, w),
        _ => return TvmReport::default(),
    };
    // Reject non-finite (e.g. divide-by-zero from n=0 or rate-0 net-zero).
    let answer = raw.filter(|v| v.is_finite());

    let (nominal, effective) = if solve == "r" {
        match answer {
            Some(a) => (
                Some(round4(a * ppy as f64 * 100.0)),
                Some(round4(((1.0 + a).powi(ppy as i32) - 1.0) * 100.0)),
            ),
            None => (None, None),
        }
    } else {
        (None, None)
    };

    TvmReport {
        solve,
        answer: answer.map(|a| if i.solve.trim().eq_ignore_ascii_case("r") { round8(a) } else { round4(a) }),
        nominal_annual_pct: nominal,
        effective_annual_pct: effective,
        valid: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    fn base() -> TvmInput {
        TvmInput {
            solve: "fv".into(),
            periods_per_year: 12,
            when_begin: false,
            pv: -10_000.0,
            fv: 0.0,
            pmt: -200.0,
            n: 360.0,
            annual_rate_pct: 7.0,
        }
    }

    // Pins cross-checked against the JS compute() in Python.
    #[test]
    fn solve_fv_default() {
        let d = generate(&base());
        assert!(d.valid);
        assert!(close(d.answer.unwrap(), 325_159.1739));
        assert!(d.nominal_annual_pct.is_none());
    }

    #[test]
    fn solve_pmt_mortgage() {
        let d = generate(&TvmInput {
            solve: "pmt".into(), pv: 300_000.0, fv: 0.0, n: 360.0, annual_rate_pct: 6.5, ..base()
        });
        assert!(close(d.answer.unwrap(), -1_896.2041));
    }

    #[test]
    fn solve_n_periods() {
        let d = generate(&TvmInput { solve: "n".into(), fv: 500_000.0, ..base() });
        assert!(close(d.answer.unwrap(), 428.1471));
    }

    #[test]
    fn solve_pv_value() {
        let d = generate(&TvmInput { solve: "pv".into(), fv: 500_000.0, ..base() });
        assert!(close(d.answer.unwrap(), -31_541.4132));
    }

    #[test]
    fn solve_r_reports_annual_rates() {
        let d = generate(&TvmInput { solve: "r".into(), fv: 1_000_000.0, ..base() });
        assert!((d.answer.unwrap() - 0.00980835).abs() < 1e-6);
        assert!(close(d.nominal_annual_pct.unwrap(), 11.77));
        assert!(close(d.effective_annual_pct.unwrap(), 12.4262));
    }

    #[test]
    fn rate_zero_uses_linear_form() {
        // r=0, FV solve → -(pv + pmt·n).
        let d = generate(&TvmInput { annual_rate_pct: 0.0, ..base() });
        assert!(close(d.answer.unwrap(), 82_000.0)); // -(-10000 + -200·360)
    }

    #[test]
    fn unknown_solve_invalid() {
        let d = generate(&TvmInput { solve: "xyz".into(), ..base() });
        assert!(!d.valid);
    }
}
