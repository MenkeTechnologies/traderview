//! Invoice factoring — the true (annualized) cost of selling a receivable.
//!
//! A factor advances most of an unpaid invoice now, charges a fee, and
//! releases the rest (the reserve, net of the fee) when the customer pays.
//! It looks cheap as a flat percentage, but the fee buys cash for only the
//! short collection period — annualized, factoring is expensive:
//!
//!   * advance      = invoice × advance rate
//!   * fee          = invoice × factor fee
//!   * reserve      = invoice − advance (held back)
//!   * net proceeds = invoice − fee (advance now + reserve−fee at collection)
//!   * **effective APR = (fee / advance) × (365 / term days)**
//!
//! A 3% fee on a 30-day invoice is ~36–46% APR. Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct FactoringInput {
    pub invoice_amount_usd: f64,
    /// Percent of the invoice advanced upfront (e.g. 80).
    pub advance_rate_pct: f64,
    /// Factor fee as a percent of the invoice (e.g. 3).
    pub factor_fee_pct: f64,
    /// Days until the customer pays / the invoice is collected.
    pub term_days: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct FactoringResult {
    pub advance_usd: f64,
    pub fee_usd: f64,
    /// Held back until collection.
    pub reserve_usd: f64,
    /// Reserve paid out at collection, net of the fee.
    pub reserve_released_usd: f64,
    /// Total cash ultimately received (invoice − fee).
    pub net_proceeds_usd: f64,
    /// Annualized cost of the advance.
    pub effective_apr_pct: f64,
}

pub fn analyze(i: &FactoringInput) -> FactoringResult {
    let invoice = i.invoice_amount_usd.max(0.0);
    let advance = invoice * i.advance_rate_pct / 100.0;
    let fee = invoice * i.factor_fee_pct / 100.0;
    let reserve = invoice - advance;
    let reserve_released = reserve - fee;
    let net_proceeds = invoice - fee;

    // APR = period cost on the advance, annualized over the term.
    let effective_apr = if advance > 0.0 && i.term_days > 0.0 {
        (fee / advance) * (365.0 / i.term_days) * 100.0
    } else {
        0.0
    };

    FactoringResult {
        advance_usd: advance,
        fee_usd: fee,
        reserve_usd: reserve,
        reserve_released_usd: reserve_released,
        net_proceeds_usd: net_proceeds,
        effective_apr_pct: effective_apr,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> FactoringInput {
        FactoringInput {
            invoice_amount_usd: 10_000.0,
            advance_rate_pct: 80.0,
            factor_fee_pct: 3.0,
            term_days: 30.0,
        }
    }

    #[test]
    fn advance_and_fee() {
        let r = analyze(&base());
        assert!((r.advance_usd - 8_000.0).abs() < 1e-6);
        assert!((r.fee_usd - 300.0).abs() < 1e-6);
    }

    #[test]
    fn reserve_and_release_net_of_fee() {
        let r = analyze(&base());
        assert!((r.reserve_usd - 2_000.0).abs() < 1e-6);
        assert!((r.reserve_released_usd - 1_700.0).abs() < 1e-6); // 2000 − 300 fee
    }

    #[test]
    fn net_proceeds_is_invoice_minus_fee() {
        let r = analyze(&base());
        assert!((r.net_proceeds_usd - 9_700.0).abs() < 1e-6);
    }

    #[test]
    fn effective_apr_annualizes_the_fee() {
        // (300/8000) × (365/30) = 0.0375 × 12.1667 = 45.625%.
        let r = analyze(&base());
        assert!((r.effective_apr_pct - (300.0 / 8_000.0) * (365.0 / 30.0) * 100.0).abs() < 1e-9);
        assert!(r.effective_apr_pct > 45.0 && r.effective_apr_pct < 46.0);
    }

    #[test]
    fn shorter_term_means_higher_apr() {
        let slow = analyze(&base());
        let fast = analyze(&FactoringInput { term_days: 15.0, ..base() });
        assert!(fast.effective_apr_pct > slow.effective_apr_pct);
    }

    #[test]
    fn higher_advance_rate_lowers_apr() {
        // Same fee, more cash advanced → cheaper per dollar borrowed.
        let low = analyze(&base());
        let high = analyze(&FactoringInput { advance_rate_pct: 95.0, ..base() });
        assert!(high.effective_apr_pct < low.effective_apr_pct);
    }

    #[test]
    fn zero_term_guards_apr() {
        let r = analyze(&FactoringInput { term_days: 0.0, ..base() });
        assert!(r.effective_apr_pct.abs() < 1e-9);
    }

    #[test]
    fn full_advance_no_reserve() {
        let r = analyze(&FactoringInput { advance_rate_pct: 100.0, ..base() });
        assert!(r.reserve_usd.abs() < 1e-9);
        // Reserve released goes negative (fee owed against a zero reserve).
        assert!((r.reserve_released_usd - (-300.0)).abs() < 1e-6);
    }
}
