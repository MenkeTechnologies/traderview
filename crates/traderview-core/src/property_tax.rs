//! Property tax — annual tax from a home's value, the assessment ratio, any
//! exemptions, and the mill rate.
//!
//! ```text
//! assessed   = market value × assessment ratio
//! taxable    = max(0, assessed − exemptions)
//! annual tax = taxable × mill rate / 1000
//! ```
//!
//! A mill is $1 of tax per $1,000 of taxable value. The effective rate is the
//! annual tax against the home's market value.

use serde::{Deserialize, Serialize};

fn d_ratio() -> f64 {
    100.0
}

#[derive(Debug, Clone, Deserialize)]
pub struct PropertyTaxInput {
    pub market_value_usd: f64,
    /// Assessed value as a percent of market value.
    #[serde(default = "d_ratio")]
    pub assessment_ratio_pct: f64,
    /// Exemptions (homestead, senior, etc.) subtracted from the assessed value.
    #[serde(default)]
    pub exemption_usd: f64,
    /// Mill rate — dollars of tax per $1,000 of taxable value.
    pub mill_rate: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PropertyTaxResult {
    pub assessed_value_usd: f64,
    /// Assessed value less exemptions (floored at 0).
    pub taxable_value_usd: f64,
    pub annual_tax_usd: f64,
    pub monthly_tax_usd: f64,
    /// Annual tax as a percent of market value.
    pub effective_rate_pct: f64,
}

pub fn analyze(input: &PropertyTaxInput) -> PropertyTaxResult {
    let assessed = input.market_value_usd * input.assessment_ratio_pct / 100.0;
    let taxable = (assessed - input.exemption_usd).max(0.0);
    let annual = taxable * input.mill_rate / 1000.0;

    PropertyTaxResult {
        assessed_value_usd: assessed,
        taxable_value_usd: taxable,
        annual_tax_usd: annual,
        monthly_tax_usd: annual / 12.0,
        effective_rate_pct: if input.market_value_usd > 0.0 {
            annual / input.market_value_usd * 100.0
        } else {
            0.0
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-4
    }

    fn run(market: f64, ratio: f64, exemption: f64, mill: f64) -> PropertyTaxResult {
        analyze(&PropertyTaxInput {
            market_value_usd: market,
            assessment_ratio_pct: ratio,
            exemption_usd: exemption,
            mill_rate: mill,
        })
    }

    #[test]
    fn assessed_value() {
        let r = run(400_000.0, 100.0, 25_000.0, 20.0);
        assert!(close(r.assessed_value_usd, 400_000.0));
    }

    #[test]
    fn taxable_after_exemption() {
        let r = run(400_000.0, 100.0, 25_000.0, 20.0);
        assert!(close(r.taxable_value_usd, 375_000.0));
    }

    #[test]
    fn annual_tax() {
        // 375,000 × 20 / 1000 = 7,500.
        let r = run(400_000.0, 100.0, 25_000.0, 20.0);
        assert!(close(r.annual_tax_usd, 7_500.0));
    }

    #[test]
    fn monthly_tax() {
        let r = run(400_000.0, 100.0, 25_000.0, 20.0);
        assert!(close(r.monthly_tax_usd, 625.0));
    }

    #[test]
    fn effective_rate_on_market() {
        // 7,500 / 400,000 = 1.875%.
        let r = run(400_000.0, 100.0, 25_000.0, 20.0);
        assert!(close(r.effective_rate_pct, 1.875));
    }

    #[test]
    fn assessment_ratio_below_100_lowers_tax() {
        // 80% ratio → assessed 320k, taxable 295k, tax 5,900.
        let r = run(400_000.0, 80.0, 25_000.0, 20.0);
        assert!(close(r.assessed_value_usd, 320_000.0));
        assert!(close(r.annual_tax_usd, 5_900.0));
    }

    #[test]
    fn exemption_above_assessed_zeroes_tax() {
        let r = run(100_000.0, 100.0, 150_000.0, 20.0);
        assert!(close(r.taxable_value_usd, 0.0));
        assert!(close(r.annual_tax_usd, 0.0));
    }

    #[test]
    fn higher_mill_rate_higher_tax() {
        let low = run(400_000.0, 100.0, 0.0, 15.0);
        let high = run(400_000.0, 100.0, 0.0, 30.0);
        assert!(high.annual_tax_usd > low.annual_tax_usd);
    }
}
