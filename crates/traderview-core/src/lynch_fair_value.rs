//! Peter Lynch fair value — the dividend-adjusted PEG from One Up On
//! Wall Street:
//!
//!   ratio = (EPS growth % + dividend yield %) / P/E
//!
//! Lynch's bands: < 1 poor, 1–1.5 fair, 1.5–2 good, ≥ 2 attractive.
//! The companion heuristic prices fair P/E = growth rate, giving a
//! fair price of growth × EPS and the implied upside.
//!
//! Pure compute. Companion to `deep_value`, `valuation_gauges`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct LynchInput {
    /// Expected EPS growth, %/yr.
    pub eps_growth_pct: f64,
    /// Dividend yield, %.
    #[serde(default)]
    pub dividend_yield_pct: f64,
    pub pe_ratio: f64,
    /// Current EPS, $ — enables the fair-price row when > 0.
    #[serde(default)]
    pub eps: f64,
    /// Current price, $ — enables the upside row when > 0.
    #[serde(default)]
    pub price: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct LynchReport {
    /// (growth + yield) / PE.
    pub adjusted_peg: f64,
    /// "poor" < 1, "fair" 1–1.5, "good" 1.5–2, "attractive" ≥ 2.
    pub band: &'static str,
    /// Fair P/E = growth heuristic.
    pub fair_pe: f64,
    pub fair_price: Option<f64>,
    pub upside_pct: Option<f64>,
}

pub fn compute(inp: &LynchInput) -> Option<LynchReport> {
    if !inp.eps_growth_pct.is_finite()
        || inp.eps_growth_pct <= 0.0
        || inp.eps_growth_pct > 100.0
        || !inp.dividend_yield_pct.is_finite()
        || inp.dividend_yield_pct < 0.0
        || !inp.pe_ratio.is_finite()
        || inp.pe_ratio <= 0.0
        || !inp.eps.is_finite()
        || inp.eps < 0.0
        || !inp.price.is_finite()
        || inp.price < 0.0
    {
        return None;
    }
    let ratio = (inp.eps_growth_pct + inp.dividend_yield_pct) / inp.pe_ratio;
    let band = if ratio < 1.0 {
        "poor"
    } else if ratio < 1.5 {
        "fair"
    } else if ratio < 2.0 {
        "good"
    } else {
        "attractive"
    };
    let fair_price = (inp.eps > 0.0).then(|| inp.eps_growth_pct * inp.eps);
    let upside_pct = match (fair_price, inp.price > 0.0) {
        (Some(fp), true) => Some((fp / inp.price - 1.0) * 100.0),
        _ => None,
    };
    Some(LynchReport {
        adjusted_peg: ratio,
        band,
        fair_pe: inp.eps_growth_pct,
        fair_price,
        upside_pct,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lynch_textbook_example() {
        // 20% grower with a 2% yield at 11× earnings: (20+2)/11 = 2.0
        // — exactly the "attractive" bar.
        let r = compute(&LynchInput {
            eps_growth_pct: 20.0,
            dividend_yield_pct: 2.0,
            pe_ratio: 11.0,
            eps: 5.0,
            price: 55.0,
        })
        .unwrap();
        assert!((r.adjusted_peg - 2.0).abs() < 1e-12);
        assert_eq!(r.band, "attractive");
        // Fair P/E = growth ⇒ fair price 20 × $5 = $100 ⇒ +81.8%.
        assert!((r.fair_price.unwrap() - 100.0).abs() < 1e-12);
        assert!((r.upside_pct.unwrap() - (100.0 / 55.0 - 1.0) * 100.0).abs() < 1e-9);
    }

    #[test]
    fn glamour_growth_at_any_price_reads_poor() {
        // 15% grower at 40×: (15+0)/40 = 0.375.
        let r = compute(&LynchInput {
            eps_growth_pct: 15.0,
            dividend_yield_pct: 0.0,
            pe_ratio: 40.0,
            eps: 0.0,
            price: 0.0,
        })
        .unwrap();
        assert_eq!(r.band, "poor");
        assert!(r.fair_price.is_none());
        assert!(r.upside_pct.is_none());
    }

    #[test]
    fn band_boundaries() {
        let mk = |g: f64, pe: f64| {
            compute(&LynchInput {
                eps_growth_pct: g,
                dividend_yield_pct: 0.0,
                pe_ratio: pe,
                eps: 0.0,
                price: 0.0,
            })
            .unwrap()
            .band
        };
        assert_eq!(mk(10.0, 10.0), "fair"); // exactly 1.0
        assert_eq!(mk(15.0, 10.0), "good"); // 1.5
        assert_eq!(mk(9.9, 10.0), "poor");
    }

    #[test]
    fn hostile_inputs_return_none() {
        assert!(compute(&LynchInput {
            eps_growth_pct: 0.0,
            dividend_yield_pct: 0.0,
            pe_ratio: 10.0,
            eps: 0.0,
            price: 0.0,
        })
        .is_none());
        assert!(compute(&LynchInput {
            eps_growth_pct: 150.0,
            dividend_yield_pct: 0.0,
            pe_ratio: 10.0,
            eps: 0.0,
            price: 0.0,
        })
        .is_none());
        assert!(compute(&LynchInput {
            eps_growth_pct: 10.0,
            dividend_yield_pct: 0.0,
            pe_ratio: 0.0,
            eps: 0.0,
            price: 0.0,
        })
        .is_none());
    }
}
