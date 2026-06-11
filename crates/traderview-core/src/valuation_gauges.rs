//! Whole-market valuation gauges:
//!
//! * Buffett indicator — total equity market cap / GDP, %.
//! * Tobin's Q — market value of corporates / replacement cost.
//! * Equity risk premium — earnings yield − treasury yield.
//! * Excess CAPE yield (Shiller 2020) — 1/CAPE − real long yield.
//!
//! All pure compute on caller-supplied aggregates. Band labels follow
//! the commonly cited heuristics and are exactly that — heuristics,
//! not calibrated signals. Companion to `cape_indicator` (per-level
//! CAPE scoring) and `market_valuation` (Fed model).

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct BuffettReport {
    /// Market cap / GDP × 100.
    pub ratio_pct: f64,
    /// Heuristic band: "undervalued" < 75, "fair" 75–90, "modestly
    /// overvalued" 90–115, "significantly overvalued" ≥ 115.
    pub band: &'static str,
}

pub fn buffett_indicator(total_market_cap: f64, gdp: f64) -> Option<BuffettReport> {
    if !total_market_cap.is_finite() || total_market_cap <= 0.0 || !gdp.is_finite() || gdp <= 0.0 {
        return None;
    }
    let ratio = total_market_cap / gdp * 100.0;
    let band = if ratio < 75.0 {
        "undervalued"
    } else if ratio < 90.0 {
        "fair"
    } else if ratio < 115.0 {
        "modestly overvalued"
    } else {
        "significantly overvalued"
    };
    Some(BuffettReport {
        ratio_pct: ratio,
        band,
    })
}

#[derive(Debug, Clone, Serialize)]
pub struct TobinReport {
    pub q: f64,
    /// vs the ~0.7 long-run US mean: "cheap" < 0.7, "elevated"
    /// 0.7–1.0, "expensive" ≥ 1.0.
    pub band: &'static str,
}

pub fn tobins_q(market_value: f64, replacement_cost: f64) -> Option<TobinReport> {
    if !market_value.is_finite()
        || market_value <= 0.0
        || !replacement_cost.is_finite()
        || replacement_cost <= 0.0
    {
        return None;
    }
    let q = market_value / replacement_cost;
    let band = if q < 0.7 {
        "cheap"
    } else if q < 1.0 {
        "elevated"
    } else {
        "expensive"
    };
    Some(TobinReport { q, band })
}

#[derive(Debug, Clone, Serialize)]
pub struct ErpReport {
    pub earnings_yield_pct: f64,
    pub treasury_yield_pct: f64,
    pub equity_risk_premium_pct: f64,
    /// Positive ERP = equities priced to out-earn bonds.
    pub favors_equities: bool,
}

/// `pe_ratio` is the market multiple (e.g. forward S&P P/E);
/// `treasury_yield_pct` the comparison bond yield in percent.
pub fn equity_risk_premium(pe_ratio: f64, treasury_yield_pct: f64) -> Option<ErpReport> {
    if !pe_ratio.is_finite() || pe_ratio <= 0.0 || !treasury_yield_pct.is_finite() {
        return None;
    }
    let ey = 100.0 / pe_ratio;
    let erp = ey - treasury_yield_pct;
    Some(ErpReport {
        earnings_yield_pct: ey,
        treasury_yield_pct,
        equity_risk_premium_pct: erp,
        favors_equities: erp > 0.0,
    })
}

#[derive(Debug, Clone, Serialize)]
pub struct EcyReport {
    pub cape: f64,
    pub cape_yield_pct: f64,
    pub real_yield_pct: f64,
    pub excess_cape_yield_pct: f64,
}

/// Shiller's ECY: 1/CAPE − real long-bond yield, both in percent.
pub fn excess_cape_yield(cape: f64, real_yield_pct: f64) -> Option<EcyReport> {
    if !cape.is_finite() || cape <= 0.0 || !real_yield_pct.is_finite() {
        return None;
    }
    let cy = 100.0 / cape;
    Some(EcyReport {
        cape,
        cape_yield_pct: cy,
        real_yield_pct,
        excess_cape_yield_pct: cy - real_yield_pct,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buffett_ratio_and_bands() {
        let r = buffett_indicator(50_000.0, 27_000.0).unwrap();
        assert!((r.ratio_pct - 50.0 / 27.0 * 100.0).abs() < 1e-9);
        assert_eq!(r.band, "significantly overvalued");
        assert_eq!(buffett_indicator(70.0, 100.0).unwrap().band, "undervalued");
        assert_eq!(buffett_indicator(80.0, 100.0).unwrap().band, "fair");
        assert_eq!(buffett_indicator(100.0, 100.0).unwrap().band, "modestly overvalued");
    }

    #[test]
    fn tobin_q_and_bands() {
        let r = tobins_q(1.2, 1.0).unwrap();
        assert!((r.q - 1.2).abs() < 1e-12);
        assert_eq!(r.band, "expensive");
        assert_eq!(tobins_q(0.5, 1.0).unwrap().band, "cheap");
        assert_eq!(tobins_q(0.8, 1.0).unwrap().band, "elevated");
    }

    #[test]
    fn erp_is_earnings_yield_minus_treasury() {
        // P/E 20 ⇒ 5% earnings yield; vs 4.3% ⇒ ERP 0.7pp.
        let r = equity_risk_premium(20.0, 4.3).unwrap();
        assert!((r.earnings_yield_pct - 5.0).abs() < 1e-12);
        assert!((r.equity_risk_premium_pct - 0.7).abs() < 1e-12);
        assert!(r.favors_equities);
        // P/E 33.3 vs 5% bonds: negative ERP.
        let neg = equity_risk_premium(100.0 / 3.0, 5.0).unwrap();
        assert!(!neg.favors_equities);
    }

    #[test]
    fn ecy_matches_shiller_definition() {
        // CAPE 30 ⇒ 3.333% cape yield; real yield 1% ⇒ ECY 2.333%.
        let r = excess_cape_yield(30.0, 1.0).unwrap();
        assert!((r.cape_yield_pct - 10.0 / 3.0).abs() < 1e-9);
        assert!((r.excess_cape_yield_pct - (10.0 / 3.0 - 1.0)).abs() < 1e-9);
        // Negative real yields RAISE the ECY (the 2020-21 regime).
        let neg = excess_cape_yield(30.0, -1.0).unwrap();
        assert!(neg.excess_cape_yield_pct > r.excess_cape_yield_pct);
    }

    #[test]
    fn hostile_inputs_return_none() {
        assert!(buffett_indicator(0.0, 100.0).is_none());
        assert!(buffett_indicator(100.0, 0.0).is_none());
        assert!(tobins_q(f64::NAN, 1.0).is_none());
        assert!(equity_risk_premium(0.0, 4.0).is_none());
        assert!(excess_cape_yield(-5.0, 1.0).is_none());
    }
}
