//! Sales-tax / VAT calculator.
//!
//! Two directions over a stack of jurisdiction rates (state + county + city,
//! or a single VAT rate):
//!
//! * `AddTax` — the amount is tax-exclusive (a net/pre-tax price); add the
//!   combined rate to get the gross the customer pays.
//! * `ExtractTax` — the amount is tax-inclusive (a gross/receipt total); back
//!   the tax out to recover the net the business keeps.
//!
//! Per-jurisdiction tax is always apportioned on the tax-exclusive (net) base,
//! so the parts sum to the total tax in both directions.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    /// Amount is pre-tax; add the tax on top.
    AddTax,
    /// Amount already includes tax; extract it.
    ExtractTax,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SalesTaxInput {
    pub mode: Mode,
    /// The money figure: a net price (AddTax) or a gross total (ExtractTax).
    pub amount_usd: f64,
    /// Combined-rate components in percent, e.g. `[6.0, 1.0, 0.75]` for
    /// state / county / city. A single VAT rate is just a one-element list.
    #[serde(default)]
    pub rates_pct: Vec<f64>,
}

/// One jurisdiction's share of the tax, on the tax-exclusive base.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct RatePortion {
    pub rate_pct: f64,
    pub tax_usd: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SalesTaxResult {
    /// Pre-tax amount (what the business keeps).
    pub net_usd: f64,
    /// Total tax across all jurisdictions.
    pub tax_usd: f64,
    /// Tax-inclusive total (what the customer pays).
    pub gross_usd: f64,
    /// Sum of the rate components, in percent.
    pub combined_rate_pct: f64,
    /// Per-jurisdiction tax, apportioned on the net base.
    pub breakdown: Vec<RatePortion>,
}

pub fn analyze(input: &SalesTaxInput) -> SalesTaxResult {
    let combined_pct: f64 = input.rates_pct.iter().sum();
    let combined = combined_pct / 100.0;

    let (net, gross) = match input.mode {
        Mode::AddTax => {
            let net = input.amount_usd;
            (net, net * (1.0 + combined))
        }
        Mode::ExtractTax => {
            let gross = input.amount_usd;
            // Guard the degenerate −100% rate that would divide by zero.
            let net = if 1.0 + combined != 0.0 {
                gross / (1.0 + combined)
            } else {
                0.0
            };
            (net, gross)
        }
    };
    let tax = gross - net;

    // Apportion on the net base so the parts reconcile to the total.
    let breakdown = input
        .rates_pct
        .iter()
        .map(|&r| RatePortion {
            rate_pct: r,
            tax_usd: net * (r / 100.0),
        })
        .collect();

    SalesTaxResult {
        net_usd: net,
        tax_usd: tax,
        gross_usd: gross,
        combined_rate_pct: combined_pct,
        breakdown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6
    }

    fn run(mode: Mode, amount: f64, rates: Vec<f64>) -> SalesTaxResult {
        analyze(&SalesTaxInput {
            mode,
            amount_usd: amount,
            rates_pct: rates,
        })
    }

    #[test]
    fn add_tax_single_rate() {
        let r = run(Mode::AddTax, 100.0, vec![8.0]);
        assert!(close(r.net_usd, 100.0));
        assert!(close(r.tax_usd, 8.0));
        assert!(close(r.gross_usd, 108.0));
        assert!(close(r.combined_rate_pct, 8.0));
    }

    #[test]
    fn extract_tax_single_rate() {
        let r = run(Mode::ExtractTax, 108.0, vec![8.0]);
        assert!(close(r.net_usd, 100.0));
        assert!(close(r.tax_usd, 8.0));
        assert!(close(r.gross_usd, 108.0));
    }

    #[test]
    fn combined_rates_sum() {
        let r = run(Mode::AddTax, 100.0, vec![6.0, 1.0, 1.0]);
        assert!(close(r.combined_rate_pct, 8.0));
        assert!(close(r.tax_usd, 8.0));
        assert!(close(r.gross_usd, 108.0));
    }

    #[test]
    fn breakdown_sums_to_total() {
        let r = run(Mode::AddTax, 250.0, vec![6.25, 1.0, 0.75]);
        let parts: f64 = r.breakdown.iter().map(|p| p.tax_usd).sum();
        assert!(close(parts, r.tax_usd));
        assert_eq!(r.breakdown.len(), 3);
    }

    #[test]
    fn breakdown_per_jurisdiction() {
        let r = run(Mode::AddTax, 100.0, vec![6.0, 2.0]);
        assert!(close(r.breakdown[0].tax_usd, 6.0));
        assert!(close(r.breakdown[1].tax_usd, 2.0));
    }

    #[test]
    fn zero_rate_is_passthrough() {
        let r = run(Mode::AddTax, 100.0, vec![]);
        assert!(close(r.tax_usd, 0.0));
        assert!(close(r.net_usd, r.gross_usd));
    }

    #[test]
    fn extract_then_add_roundtrips() {
        let extracted = run(Mode::ExtractTax, 216.0, vec![6.0, 2.0]);
        let added = run(Mode::AddTax, extracted.net_usd, vec![6.0, 2.0]);
        assert!(close(added.gross_usd, 216.0));
    }

    #[test]
    fn zero_amount_guard() {
        let r = run(Mode::AddTax, 0.0, vec![8.0]);
        assert!(close(r.net_usd, 0.0));
        assert!(close(r.tax_usd, 0.0));
        assert!(close(r.gross_usd, 0.0));
    }
}
