//! Yield to maturity / call / worst for a callable bond. YTM is the IRR if held
//! to maturity (redeemed at par); YTC is the IRR if called at the call date and
//! call price; yield-to-worst is the lower of the two — what a prudent buyer of a
//! premium callable bond should underwrite to. Each yield is solved by bisection
//! on the price equation. Faithful port of the former client-side `solveYield`,
//! now Python-pinned and unit-tested. Pure compute, not financial advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct YieldToCallInput {
    /// Clean price as a percent of par (106.5 = 106.5% of par).
    pub price_pct: f64,
    /// Annual coupon rate, percent.
    pub coupon_rate_pct: f64,
    pub years_to_maturity: f64,
    pub years_to_call: f64,
    /// Call price as a percent of par.
    pub call_price_pct: f64,
    #[serde(default = "default_freq")]
    pub coupons_per_year: u32,
}

fn default_freq() -> u32 {
    2
}

#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct YieldToCallReport {
    pub ytm_pct: Option<f64>,
    pub ytc_pct: Option<f64>,
    pub ytw_pct: Option<f64>,
    pub current_yield_pct: f64,
    pub premium_to_par_pct: f64,
    pub is_premium: bool,
    /// "YTC", "YTM", or "TIE" — which yield is the worst (binds).
    pub verdict: String,
    pub valid: bool,
}

fn round4(x: f64) -> f64 {
    (x * 10_000.0).round() / 10_000.0
}

// Bisection on the annualized yield that prices the cash flows to `price`.
// Mirrors the original JS solver bracket [-0.20, 0.50] with hi-expansion.
fn solve_yield(price: f64, cpn: f64, redemption: f64, n: f64, freq: f64) -> Option<f64> {
    let f = |y: f64| -> f64 {
        let r = y / freq;
        if r.abs() < 1e-12 {
            return cpn * n + redemption - price;
        }
        let pw = (1.0 + r).powf(n);
        cpn * (1.0 - 1.0 / pw) / r + redemption / pw - price
    };
    let mut lo = -0.20;
    let mut hi = 0.50;
    let mut fl = f(lo);
    let mut fh = f(hi);
    if fl * fh > 0.0 {
        let mut i = 0;
        while i < 20 && fl * fh > 0.0 {
            hi *= 1.5;
            fh = f(hi);
            i += 1;
        }
        if fl * fh > 0.0 {
            return None;
        }
    }
    for _ in 0..200 {
        let mid = (lo + hi) / 2.0;
        let fm = f(mid);
        if fm.abs() < 1e-9 {
            return Some(mid);
        }
        if fl * fm < 0.0 {
            hi = mid;
            fh = fm;
        } else {
            lo = mid;
            fl = fm;
        }
    }
    let _ = fh;
    Some((lo + hi) / 2.0)
}

pub fn generate(i: &YieldToCallInput) -> YieldToCallReport {
    if i.price_pct <= 0.0 || i.coupons_per_year == 0 {
        return YieldToCallReport::default();
    }
    let par = 1000.0;
    let freq = i.coupons_per_year as f64;
    let price = par * i.price_pct / 100.0;
    let call_price = par * i.call_price_pct / 100.0;
    let cpn = par * (i.coupon_rate_pct / 100.0) / freq;

    let ytm = solve_yield(price, cpn, par, i.years_to_maturity * freq, freq);
    let ytc = solve_yield(price, cpn, call_price, i.years_to_call * freq, freq);
    let ytw = match (ytm, ytc) {
        (Some(m), Some(c)) => Some(m.min(c)),
        (Some(m), None) => Some(m),
        (None, Some(c)) => Some(c),
        (None, None) => None,
    };
    let verdict = match (ytm, ytc) {
        (Some(m), Some(c)) if c < m => "YTC",
        (Some(m), Some(c)) if c > m => "YTM",
        (Some(_), Some(_)) => "TIE",
        _ => "",
    };
    let current_yield = (cpn * freq) / price * 100.0;

    YieldToCallReport {
        ytm_pct: ytm.map(|y| round4(y * 100.0)),
        ytc_pct: ytc.map(|y| round4(y * 100.0)),
        ytw_pct: ytw.map(|y| round4(y * 100.0)),
        current_yield_pct: round4(current_yield),
        premium_to_par_pct: round4(i.price_pct - 100.0),
        is_premium: i.price_pct > 100.0,
        verdict: verdict.to_string(),
        valid: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    fn base() -> YieldToCallInput {
        YieldToCallInput {
            price_pct: 106.5,
            coupon_rate_pct: 5.5,
            years_to_maturity: 10.0,
            years_to_call: 3.0,
            call_price_pct: 100.0,
            coupons_per_year: 2,
        }
    }

    // Pins cross-checked against the original JS solveYield in Python.
    #[test]
    fn premium_callable_yields() {
        let d = generate(&base());
        assert!(close(d.ytm_pct.unwrap(), 4.6787));
        assert!(close(d.ytc_pct.unwrap(), 3.21));
        assert!(close(d.ytw_pct.unwrap(), 3.21));
        assert!(close(d.current_yield_pct, 5.1643));
        assert!(d.is_premium);
        assert!(close(d.premium_to_par_pct, 6.5));
        assert_eq!(d.verdict, "YTC");
    }

    #[test]
    fn at_par_ytm_equals_coupon() {
        // Priced at par with maturity = call → YTM ≈ coupon, no call advantage.
        let d = generate(&YieldToCallInput { price_pct: 100.0, years_to_call: 10.0, ..base() });
        assert!(close(d.ytm_pct.unwrap(), 5.5));
        assert!(!d.is_premium);
        assert_eq!(d.verdict, "TIE");
    }

    #[test]
    fn discount_bond_ytm_above_coupon() {
        let d = generate(&YieldToCallInput { price_pct: 92.0, ..base() });
        assert!(d.ytm_pct.unwrap() > 5.5);
        assert!(!d.is_premium);
        // A discount bond won't be called early, so YTM is the worst case.
        assert_eq!(d.verdict, "YTM");
    }

    #[test]
    fn zero_price_invalid() {
        let d = generate(&YieldToCallInput { price_pct: 0.0, ..base() });
        assert!(!d.valid);
    }
}
