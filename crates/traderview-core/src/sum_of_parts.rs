//! Sum-of-the-parts valuation — segment values less net debt, with a
//! conglomerate discount, against the quoted market cap.
//!
//!   SOTP gross   = Σ segment values
//!   SOTP equity  = (gross − net debt) × (1 − discount)
//!   upside       = SOTP equity / market cap − 1
//!
//! The discount input matters: undiscounted SOTP "upside" is the
//! oldest value-trap in the book — holdcos trade 10–30% below NAV
//! persistently, so the screen prices the discount in up front.
//!
//! Pure compute. Companion to `cef_discount` (the same NAV-gap logic
//! for funds), `deep_value`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Segment {
    pub name: String,
    /// Estimated standalone value, $ (same unit as market_cap).
    pub value: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SotpInput {
    pub segments: Vec<Segment>,
    /// Net debt (negative = net cash), $.
    #[serde(default)]
    pub net_debt: f64,
    /// Conglomerate/holdco discount applied to NAV, % (0–60 typical).
    #[serde(default)]
    pub conglomerate_discount_pct: f64,
    pub market_cap: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SegmentRow {
    pub name: String,
    pub value: f64,
    /// Share of gross SOTP, %.
    pub weight_pct: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SotpReport {
    pub gross_value: f64,
    pub nav: f64,
    pub sotp_equity_value: f64,
    pub upside_pct: f64,
    /// Discount the MARKET is applying to NAV, % (positive = below).
    pub market_implied_discount_pct: f64,
    pub rows: Vec<SegmentRow>,
}

pub fn compute(inp: &SotpInput) -> Option<SotpReport> {
    if inp.segments.is_empty()
        || inp.segments.len() > 50
        || inp
            .segments
            .iter()
            .any(|s| !s.value.is_finite() || s.value < 0.0 || s.name.trim().is_empty())
        || !inp.net_debt.is_finite()
        || !inp.conglomerate_discount_pct.is_finite()
        || !(0.0..100.0).contains(&inp.conglomerate_discount_pct)
        || !inp.market_cap.is_finite()
        || inp.market_cap <= 0.0
    {
        return None;
    }
    let gross: f64 = inp.segments.iter().map(|s| s.value).sum();
    if gross <= 0.0 {
        return None;
    }
    let nav = gross - inp.net_debt;
    if nav <= 0.0 {
        return None; // debt swallows the parts — no equity SOTP
    }
    let equity = nav * (1.0 - inp.conglomerate_discount_pct / 100.0);
    let rows = inp
        .segments
        .iter()
        .map(|s| SegmentRow {
            name: s.name.clone(),
            value: s.value,
            weight_pct: s.value / gross * 100.0,
        })
        .collect();
    Some(SotpReport {
        gross_value: gross,
        nav,
        sotp_equity_value: equity,
        upside_pct: (equity / inp.market_cap - 1.0) * 100.0,
        market_implied_discount_pct: (1.0 - inp.market_cap / nav) * 100.0,
        rows,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seg(name: &str, value: f64) -> Segment {
        Segment {
            name: name.into(),
            value,
        }
    }

    fn base() -> SotpInput {
        SotpInput {
            segments: vec![seg("Cloud", 60.0), seg("Ads", 30.0), seg("Hardware", 10.0)],
            net_debt: 20.0,
            conglomerate_discount_pct: 15.0,
            market_cap: 60.0,
        }
    }

    #[test]
    fn sotp_hand_walk() {
        // Gross 100, NAV 80, ×0.85 = 68 vs cap 60 ⇒ +13.33% upside.
        // Market itself prices NAV at 60/80 = 25% discount.
        let r = compute(&base()).unwrap();
        assert!((r.gross_value - 100.0).abs() < 1e-12);
        assert!((r.nav - 80.0).abs() < 1e-12);
        assert!((r.sotp_equity_value - 68.0).abs() < 1e-12);
        assert!((r.upside_pct - (68.0 / 60.0 - 1.0) * 100.0).abs() < 1e-9);
        assert!((r.market_implied_discount_pct - 25.0).abs() < 1e-12);
        // Weights: 60/30/10.
        assert!((r.rows[0].weight_pct - 60.0).abs() < 1e-12);
        assert!((r.rows[2].weight_pct - 10.0).abs() < 1e-12);
    }

    #[test]
    fn net_cash_adds_to_nav() {
        let mut inp = base();
        inp.net_debt = -10.0; // net cash
        let r = compute(&inp).unwrap();
        assert!((r.nav - 110.0).abs() < 1e-12);
    }

    #[test]
    fn undiscounted_sotp_overstates_upside() {
        let discounted = compute(&base()).unwrap();
        let mut naive = base();
        naive.conglomerate_discount_pct = 0.0;
        let undiscounted = compute(&naive).unwrap();
        assert!(undiscounted.upside_pct > discounted.upside_pct);
    }

    #[test]
    fn hostile_inputs_return_none() {
        let mut bad = base();
        bad.segments.clear();
        assert!(compute(&bad).is_none());
        let mut bad = base();
        bad.segments[0].value = -5.0;
        assert!(compute(&bad).is_none());
        let mut bad = base();
        bad.net_debt = 150.0; // swallows the parts
        assert!(compute(&bad).is_none());
        let mut bad = base();
        bad.conglomerate_discount_pct = 100.0;
        assert!(compute(&bad).is_none());
        let mut bad = base();
        bad.market_cap = 0.0;
        assert!(compute(&bad).is_none());
    }
}
