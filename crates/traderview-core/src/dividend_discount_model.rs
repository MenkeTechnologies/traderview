//! Dividend discount model (DDM) — intrinsic share value as the present value of
//! future dividends. Supports the single-stage Gordon growth model (constant
//! growth in perpetuity) and a two-stage model (a high-growth phase for N years,
//! then a perpetual terminal growth). Reports the fair value and, for two-stage,
//! the split between the explicit dividends and the terminal value. Distinct from
//! the implied-dividend module (which backs dividend yield out of option prices).
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct DdmInput {
    pub ticker: String,
    /// Most recent annual dividend per share (D0).
    pub current_dividend_usd: f64,
    /// Required rate of return (discount rate), percent.
    pub required_return_pct: f64,
    /// "gordon" (constant growth) or "two_stage".
    #[serde(default)]
    pub model: String,
    /// Constant growth (Gordon) or high-growth-phase rate (two-stage), percent.
    pub growth_pct: f64,
    /// High-growth phase length in years (two-stage only).
    #[serde(default)]
    pub high_growth_years: u32,
    /// Terminal perpetual growth rate, percent (two-stage only).
    #[serde(default)]
    pub terminal_growth_pct: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct DdmReport {
    pub model_label: String,
    /// Intrinsic value per share (0 if the model is invalid, e.g. g ≥ r).
    pub fair_value_usd: f64,
    /// Present value of the explicit-phase dividends (two-stage; equals 0 for Gordon).
    pub pv_dividends_usd: f64,
    /// Present value of the terminal value (two-stage; equals the full value for Gordon).
    pub pv_terminal_usd: f64,
    /// Next-year dividend D1 = D0 × (1 + growth).
    pub next_dividend_usd: f64,
    pub valid: bool,
}

fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

pub fn generate(i: &DdmInput) -> DdmReport {
    let r = i.required_return_pct / 100.0;
    let g = i.growth_pct / 100.0;
    let d0 = i.current_dividend_usd;
    let two_stage = i.model.trim().eq_ignore_ascii_case("two_stage");

    if two_stage {
        let g2 = i.terminal_growth_pct / 100.0;
        let n = i.high_growth_years;
        // Terminal perpetuity requires r > terminal growth.
        if r <= g2 || n == 0 {
            return DdmReport { model_label: "Two-stage".into(), ..Default::default() };
        }
        let mut pv_div = 0.0;
        let mut d = d0;
        for t in 1..=n {
            d *= 1.0 + g;
            pv_div += d / (1.0 + r).powi(t as i32);
        }
        // d now holds D_n; terminal value at year n uses the terminal growth.
        let terminal = d * (1.0 + g2) / (r - g2);
        let pv_terminal = terminal / (1.0 + r).powi(n as i32);
        let fair = pv_div + pv_terminal;
        DdmReport {
            model_label: "Two-stage".into(),
            fair_value_usd: round2(fair),
            pv_dividends_usd: round2(pv_div),
            pv_terminal_usd: round2(pv_terminal),
            next_dividend_usd: round2(d0 * (1.0 + g)),
            valid: true,
        }
    } else {
        // Gordon growth requires r > g.
        if r <= g {
            return DdmReport { model_label: "Gordon growth".into(), ..Default::default() };
        }
        let d1 = d0 * (1.0 + g);
        let fair = d1 / (r - g);
        DdmReport {
            model_label: "Gordon growth".into(),
            fair_value_usd: round2(fair),
            pv_dividends_usd: 0.0,
            pv_terminal_usd: round2(fair),
            next_dividend_usd: round2(d1),
            valid: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.02
    }

    #[test]
    fn gordon_growth() {
        let d = generate(&DdmInput {
            ticker: "ACME".into(),
            current_dividend_usd: 2.0,
            required_return_pct: 9.0,
            model: "gordon".into(),
            growth_pct: 4.0,
            high_growth_years: 0,
            terminal_growth_pct: 0.0,
        });
        assert!(d.valid);
        assert!(close(d.fair_value_usd, 41.60));
        assert!(close(d.next_dividend_usd, 2.08));
    }

    #[test]
    fn two_stage() {
        let d = generate(&DdmInput {
            ticker: "ACME".into(),
            current_dividend_usd: 2.0,
            required_return_pct: 9.0,
            model: "two_stage".into(),
            growth_pct: 15.0,
            high_growth_years: 5,
            terminal_growth_pct: 4.0,
        });
        assert!(d.valid);
        assert!(close(d.pv_dividends_usd, 11.78));
        assert!(close(d.pv_terminal_usd, 54.38));
        assert!(close(d.fair_value_usd, 66.16));
    }

    #[test]
    fn gordon_invalid_when_growth_exceeds_return() {
        let d = generate(&DdmInput {
            ticker: "X".into(),
            current_dividend_usd: 2.0,
            required_return_pct: 5.0,
            model: "gordon".into(),
            growth_pct: 6.0,
            high_growth_years: 0,
            terminal_growth_pct: 0.0,
        });
        assert!(!d.valid);
        assert!(close(d.fair_value_usd, 0.0));
    }

    #[test]
    fn two_stage_invalid_terminal_growth() {
        let d = generate(&DdmInput {
            ticker: "X".into(),
            current_dividend_usd: 2.0,
            required_return_pct: 5.0,
            model: "two_stage".into(),
            growth_pct: 10.0,
            high_growth_years: 5,
            terminal_growth_pct: 6.0,
        });
        assert!(!d.valid);
    }

    #[test]
    fn two_stage_splits_sum_to_fair_value() {
        let d = generate(&DdmInput {
            ticker: "ACME".into(),
            current_dividend_usd: 3.0,
            required_return_pct: 10.0,
            model: "two_stage".into(),
            growth_pct: 12.0,
            high_growth_years: 7,
            terminal_growth_pct: 3.0,
        });
        assert!(close(d.fair_value_usd, d.pv_dividends_usd + d.pv_terminal_usd));
    }
}
