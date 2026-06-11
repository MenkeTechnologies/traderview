//! ADR premium/discount vs the ordinary line.
//!
//!   parity  = ordinary_price × usd_per_local × ordinaries_per_adr
//!   premium = adr_price / parity − 1
//!
//! Persistent premiums show conversion friction (fees, settlement,
//! ownership caps — think India/Taiwan lines); fleeting ones are the
//! cross-listing arb. The report nets a caller-supplied round-trip
//! conversion fee to show what's actually capturable.
//!
//! Pure compute. Companion to `cef_discount`, `merger_arb`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct AdrInput {
    pub adr_price: f64,
    /// Ordinary-share price in local currency.
    pub ordinary_price_local: f64,
    /// USD per 1 unit of local currency.
    pub usd_per_local: f64,
    /// Ordinaries represented by one ADR.
    pub ordinaries_per_adr: f64,
    /// Round-trip conversion cost, % (default 0).
    #[serde(default)]
    pub conversion_fee_pct: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdrReport {
    /// USD value of the underlying ordinaries per ADR.
    pub parity_usd: f64,
    pub premium_pct: f64,
    /// |premium| − fee: what an arb actually nets, % (negative =
    /// friction eats the gap).
    pub capturable_pct: f64,
    pub arb_exists: bool,
}

pub fn compute(inp: &AdrInput) -> Option<AdrReport> {
    if ![
        inp.adr_price,
        inp.ordinary_price_local,
        inp.usd_per_local,
        inp.ordinaries_per_adr,
    ]
    .iter()
    .all(|v| v.is_finite() && *v > 0.0)
        || !inp.conversion_fee_pct.is_finite()
        || inp.conversion_fee_pct < 0.0
    {
        return None;
    }
    let parity = inp.ordinary_price_local * inp.usd_per_local * inp.ordinaries_per_adr;
    let premium = (inp.adr_price / parity - 1.0) * 100.0;
    let capturable = premium.abs() - inp.conversion_fee_pct;
    Some(AdrReport {
        parity_usd: parity,
        premium_pct: premium,
        capturable_pct: capturable,
        arb_exists: capturable > 0.0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn premium_hand_walk() {
        // Ordinary ¥1000 at 0.007 USD/JPY × 10 per ADR ⇒ parity $70;
        // ADR at $73.50 = +5% premium; 1% fee ⇒ 4% capturable.
        let r = compute(&AdrInput {
            adr_price: 73.5,
            ordinary_price_local: 1000.0,
            usd_per_local: 0.007,
            ordinaries_per_adr: 10.0,
            conversion_fee_pct: 1.0,
        })
        .unwrap();
        assert!((r.parity_usd - 70.0).abs() < 1e-9);
        assert!((r.premium_pct - 5.0).abs() < 1e-9);
        assert!((r.capturable_pct - 4.0).abs() < 1e-9);
        assert!(r.arb_exists);
    }

    #[test]
    fn discount_side_is_symmetric() {
        let r = compute(&AdrInput {
            adr_price: 66.5,
            ordinary_price_local: 1000.0,
            usd_per_local: 0.007,
            ordinaries_per_adr: 10.0,
            conversion_fee_pct: 1.0,
        })
        .unwrap();
        assert!((r.premium_pct + 5.0).abs() < 1e-9);
        assert!((r.capturable_pct - 4.0).abs() < 1e-9);
    }

    #[test]
    fn fees_kill_small_gaps() {
        let r = compute(&AdrInput {
            adr_price: 70.35, // +0.5% premium
            ordinary_price_local: 1000.0,
            usd_per_local: 0.007,
            ordinaries_per_adr: 10.0,
            conversion_fee_pct: 1.0,
        })
        .unwrap();
        assert!(!r.arb_exists);
        assert!(r.capturable_pct < 0.0);
    }

    #[test]
    fn hostile_inputs_return_none() {
        let base = AdrInput {
            adr_price: 70.0,
            ordinary_price_local: 1000.0,
            usd_per_local: 0.007,
            ordinaries_per_adr: 10.0,
            conversion_fee_pct: 0.0,
        };
        assert!(compute(&AdrInput { adr_price: 0.0, ..base.clone() }).is_none());
        assert!(compute(&AdrInput { usd_per_local: f64::NAN, ..base.clone() }).is_none());
        assert!(compute(&AdrInput { conversion_fee_pct: -1.0, ..base }).is_none());
    }
}
