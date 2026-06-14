//! Valuation multiples — the standard per-share market multiples and yields used
//! to value a stock: P/E (price ÷ EPS), P/B (price ÷ book value), P/S (price ÷
//! sales), PEG (P/E ÷ earnings-growth), plus the dividend, earnings, and
//! free-cash-flow yields. Complements the existing Graham-number, EV/EBITDA, and
//! EVA modules with the plain price multiples. Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct MultiplesInput {
    pub ticker: String,
    pub price_usd: f64,
    /// Trailing earnings per share.
    #[serde(default)]
    pub eps_usd: f64,
    /// Book value per share.
    #[serde(default)]
    pub book_value_per_share_usd: f64,
    /// Sales (revenue) per share.
    #[serde(default)]
    pub sales_per_share_usd: f64,
    /// Annual dividend per share.
    #[serde(default)]
    pub dividend_per_share_usd: f64,
    /// Free cash flow per share.
    #[serde(default)]
    pub fcf_per_share_usd: f64,
    /// Expected EPS growth rate, percent (for PEG).
    #[serde(default)]
    pub eps_growth_pct: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct MultiplesReport {
    /// Price ÷ EPS (0 if EPS ≤ 0).
    pub pe: f64,
    /// Price ÷ book value per share.
    pub pb: f64,
    /// Price ÷ sales per share.
    pub ps: f64,
    /// P/E ÷ growth rate (Lynch fair value at 1.0).
    pub peg: f64,
    /// Dividend ÷ price, percent.
    pub dividend_yield_pct: f64,
    /// EPS ÷ price, percent (inverse P/E).
    pub earnings_yield_pct: f64,
    /// FCF ÷ price, percent.
    pub fcf_yield_pct: f64,
}

fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

pub fn generate(i: &MultiplesInput) -> MultiplesReport {
    let p = i.price_usd;
    let div = |num: f64, den: f64| -> f64 { if den > 0.0 { round2(num / den) } else { 0.0 } };
    let pe = div(p, i.eps_usd);
    MultiplesReport {
        pe,
        pb: div(p, i.book_value_per_share_usd),
        ps: div(p, i.sales_per_share_usd),
        peg: if i.eps_growth_pct > 0.0 && pe > 0.0 { round2(pe / i.eps_growth_pct) } else { 0.0 },
        dividend_yield_pct: if p > 0.0 { round2(i.dividend_per_share_usd / p * 100.0) } else { 0.0 },
        earnings_yield_pct: if p > 0.0 { round2(i.eps_usd / p * 100.0) } else { 0.0 },
        fcf_yield_pct: if p > 0.0 { round2(i.fcf_per_share_usd / p * 100.0) } else { 0.0 },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    fn base() -> MultiplesInput {
        MultiplesInput {
            ticker: "ACME".into(),
            price_usd: 100.0,
            eps_usd: 5.0,
            book_value_per_share_usd: 25.0,
            sales_per_share_usd: 50.0,
            dividend_per_share_usd: 2.0,
            fcf_per_share_usd: 6.0,
            eps_growth_pct: 15.0,
        }
    }

    #[test]
    fn standard_multiples() {
        let d = generate(&base());
        assert!(close(d.pe, 20.0));
        assert!(close(d.pb, 4.0));
        assert!(close(d.ps, 2.0));
        assert!(close(d.peg, 1.33));
        assert!(close(d.dividend_yield_pct, 2.0));
        assert!(close(d.earnings_yield_pct, 5.0));
        assert!(close(d.fcf_yield_pct, 6.0));
    }

    #[test]
    fn earnings_yield_is_inverse_pe() {
        let d = generate(&base());
        assert!(close(d.earnings_yield_pct, 100.0 / d.pe));
    }

    #[test]
    fn negative_eps_zero_pe_and_peg() {
        let d = generate(&MultiplesInput { eps_usd: -1.0, ..base() });
        assert!(close(d.pe, 0.0));
        assert!(close(d.peg, 0.0));
    }

    #[test]
    fn no_growth_zero_peg() {
        let d = generate(&MultiplesInput { eps_growth_pct: 0.0, ..base() });
        assert!(close(d.peg, 0.0));
    }

    #[test]
    fn missing_book_value_zero_pb() {
        let d = generate(&MultiplesInput { book_value_per_share_usd: 0.0, ..base() });
        assert!(close(d.pb, 0.0));
    }
}
