//! Benjamin Graham's value screens for a defensive investor.
//!
//! * **Graham Number** — the maximum price justified by earnings and book
//!   value: `√(22.5 × EPS × BVPS)`. The 22.5 is Graham's product cap of a
//!   15× P/E and a 1.5× P/B. Defined only when both EPS and BVPS are positive.
//! * **Margin of safety** — how far the current price sits below the Graham
//!   number, as a fraction of it.
//! * **P/E × P/B test** — Graham's combined-multiple rule: the product of the
//!   trailing P/E and P/B should not exceed 22.5.
//! * **Net-net (NCAV)** — the deep-value screen: net current asset value per
//!   share `(current assets − total liabilities) / shares`, with the classic
//!   buy threshold at two-thirds of NCAV.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct GrahamInput {
    /// Trailing earnings per share.
    pub eps: f64,
    /// Book value per share.
    pub bvps: f64,
    /// Current market price per share.
    pub price: f64,
    /// Total current assets (for the net-net screen). Optional.
    #[serde(default)]
    pub current_assets_usd: f64,
    /// Total liabilities (for the net-net screen). Optional.
    #[serde(default)]
    pub total_liabilities_usd: f64,
    /// Shares outstanding (for the net-net screen). Optional.
    #[serde(default)]
    pub shares_outstanding: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct GrahamResult {
    /// √(22.5 × EPS × BVPS); `None` if EPS or BVPS is not positive.
    pub graham_number: Option<f64>,
    /// (graham_number − price) / graham_number; `None` if no Graham number.
    pub margin_of_safety_pct: Option<f64>,
    /// Trailing P/E (price / EPS); `None` if EPS ≤ 0.
    pub pe_ratio: Option<f64>,
    /// P/B (price / BVPS); `None` if BVPS ≤ 0.
    pub pb_ratio: Option<f64>,
    /// P/E × P/B; `None` if either is undefined.
    pub pe_times_pb: Option<f64>,
    /// Whether the P/E × P/B product is within Graham's 22.5 cap.
    pub passes_graham_pe_pb: bool,
    /// Net current asset value per share; `None` if shares ≤ 0.
    pub ncav_per_share: Option<f64>,
    /// Two-thirds of NCAV — the classic net-net buy threshold.
    pub net_net_price: Option<f64>,
    /// Whether the price is below the net-net threshold.
    pub is_net_net: bool,
}

pub fn analyze(input: &GrahamInput) -> GrahamResult {
    let graham_number = if input.eps > 0.0 && input.bvps > 0.0 {
        Some((22.5 * input.eps * input.bvps).sqrt())
    } else {
        None
    };

    let margin_of_safety_pct = graham_number.map(|g| (g - input.price) / g);

    let pe_ratio = if input.eps > 0.0 {
        Some(input.price / input.eps)
    } else {
        None
    };
    let pb_ratio = if input.bvps > 0.0 {
        Some(input.price / input.bvps)
    } else {
        None
    };
    let pe_times_pb = match (pe_ratio, pb_ratio) {
        (Some(pe), Some(pb)) => Some(pe * pb),
        _ => None,
    };
    let passes_graham_pe_pb = pe_times_pb.map(|x| x <= 22.5).unwrap_or(false);

    let ncav_per_share = if input.shares_outstanding > 0.0 {
        Some((input.current_assets_usd - input.total_liabilities_usd) / input.shares_outstanding)
    } else {
        None
    };
    let net_net_price = ncav_per_share.map(|n| n * 2.0 / 3.0);
    let is_net_net = net_net_price.map(|t| input.price < t).unwrap_or(false);

    GrahamResult {
        graham_number,
        margin_of_safety_pct,
        pe_ratio,
        pb_ratio,
        pe_times_pb,
        passes_graham_pe_pb,
        ncav_per_share,
        net_net_price,
        is_net_net,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-4
    }

    fn run(eps: f64, bvps: f64, price: f64) -> GrahamResult {
        analyze(&GrahamInput {
            eps,
            bvps,
            price,
            current_assets_usd: 0.0,
            total_liabilities_usd: 0.0,
            shares_outstanding: 0.0,
        })
    }

    #[test]
    fn graham_number_formula() {
        // √(22.5 × 5 × 20) = √2250 = 47.43416.
        let r = run(5.0, 20.0, 30.0);
        assert!(close(r.graham_number.unwrap(), 47.434165));
    }

    #[test]
    fn margin_of_safety() {
        // price 30 vs Graham 47.434 → (47.434−30)/47.434 = 0.367544.
        let r = run(5.0, 20.0, 30.0);
        assert!(close(r.margin_of_safety_pct.unwrap(), 0.367544));
    }

    #[test]
    fn negative_margin_when_overpriced() {
        let r = run(5.0, 20.0, 60.0);
        // price above the Graham number → negative margin of safety.
        assert!(r.margin_of_safety_pct.unwrap() < 0.0);
    }

    #[test]
    fn negative_eps_has_no_graham_number() {
        let r = run(-2.0, 20.0, 30.0);
        assert!(r.graham_number.is_none());
        assert!(r.margin_of_safety_pct.is_none());
        assert!(r.pe_ratio.is_none());
    }

    #[test]
    fn pe_times_pb_passes_within_cap() {
        // P/E = 30/5 = 6, P/B = 30/20 = 1.5, product 9 ≤ 22.5.
        let r = run(5.0, 20.0, 30.0);
        assert!(close(r.pe_ratio.unwrap(), 6.0));
        assert!(close(r.pb_ratio.unwrap(), 1.5));
        assert!(close(r.pe_times_pb.unwrap(), 9.0));
        assert!(r.passes_graham_pe_pb);
    }

    #[test]
    fn pe_times_pb_fails_over_cap() {
        // price 60 → P/E 12, P/B 3, product 36 > 22.5.
        let r = run(5.0, 20.0, 60.0);
        assert!(close(r.pe_times_pb.unwrap(), 36.0));
        assert!(!r.passes_graham_pe_pb);
    }

    #[test]
    fn net_net_screen() {
        // (CA 1,000,000 − TL 400,000) / 100,000 sh = $6 NCAV; net-net = $4.
        let r = analyze(&GrahamInput {
            eps: 1.0,
            bvps: 8.0,
            price: 3.0,
            current_assets_usd: 1_000_000.0,
            total_liabilities_usd: 400_000.0,
            shares_outstanding: 100_000.0,
        });
        assert!(close(r.ncav_per_share.unwrap(), 6.0));
        assert!(close(r.net_net_price.unwrap(), 4.0));
        assert!(r.is_net_net); // price 3 < 4
    }

    #[test]
    fn not_net_net_above_threshold() {
        let r = analyze(&GrahamInput {
            eps: 1.0,
            bvps: 8.0,
            price: 5.0,
            current_assets_usd: 1_000_000.0,
            total_liabilities_usd: 400_000.0,
            shares_outstanding: 100_000.0,
        });
        assert!(!r.is_net_net); // price 5 > 4
    }
}
