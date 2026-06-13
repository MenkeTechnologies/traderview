//! Primary-home sale capital gain after the §121 exclusion.
//!
//! A homeowner who meets the ownership/use test excludes up to $250,000 of gain
//! ($500,000 married-joint). Depreciation taken (home office, prior rental use)
//! lowers basis and that "unrecaptured §1250" portion is taxed at up to 25% and
//! is NOT excludable.
//!
//! ```text
//! amount realized = sale price − selling costs
//! adjusted basis  = purchase + improvements − depreciation
//! gain            = amount realized − adjusted basis
//! recapture gain  = min(depreciation, gain)            → taxed at 25%
//! excludable gain = gain − recapture gain              → up to the §121 limit
//! taxable LTCG    = excludable gain − exclusion applied → taxed at the LTCG rate
//! ```

use serde::{Deserialize, Serialize};

fn d_ltcg() -> f64 {
    15.0
}
fn d_recapture() -> f64 {
    25.0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FilingStatus {
    Single,
    MarriedJoint,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HomeSaleInput {
    pub sale_price_usd: f64,
    #[serde(default)]
    pub selling_costs_usd: f64,
    pub purchase_price_usd: f64,
    #[serde(default)]
    pub improvements_usd: f64,
    pub filing_status: FilingStatus,
    /// Depreciation taken (home office / former rental); lowers basis.
    #[serde(default)]
    pub depreciation_taken_usd: f64,
    #[serde(default = "d_ltcg")]
    pub ltcg_rate_pct: f64,
    #[serde(default = "d_recapture")]
    pub recapture_rate_pct: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct HomeSaleResult {
    pub amount_realized_usd: f64,
    pub adjusted_basis_usd: f64,
    pub total_gain_usd: f64,
    /// §121 exclusion limit for the filing status.
    pub exclusion_limit_usd: f64,
    /// Gain excluded under §121.
    pub excluded_gain_usd: f64,
    /// Depreciation recapture (unrecaptured §1250), taxed at the recapture rate.
    pub recapture_gain_usd: f64,
    /// Long-term gain remaining after the exclusion (taxed at the LTCG rate).
    pub taxable_ltcg_usd: f64,
    /// Total tax (recapture + LTCG).
    pub tax_usd: f64,
    /// Gain kept after tax.
    pub after_tax_gain_usd: f64,
}

pub fn analyze(input: &HomeSaleInput) -> HomeSaleResult {
    let amount_realized = input.sale_price_usd - input.selling_costs_usd;
    let adjusted_basis =
        input.purchase_price_usd + input.improvements_usd - input.depreciation_taken_usd;
    let total_gain = amount_realized - adjusted_basis;

    let exclusion_limit = match input.filing_status {
        FilingStatus::Single => 250_000.0,
        FilingStatus::MarriedJoint => 500_000.0,
    };

    if total_gain <= 0.0 {
        return HomeSaleResult {
            amount_realized_usd: amount_realized,
            adjusted_basis_usd: adjusted_basis,
            total_gain_usd: total_gain,
            exclusion_limit_usd: exclusion_limit,
            excluded_gain_usd: 0.0,
            recapture_gain_usd: 0.0,
            taxable_ltcg_usd: 0.0,
            tax_usd: 0.0,
            after_tax_gain_usd: total_gain,
        };
    }

    let recapture_gain = input.depreciation_taken_usd.min(total_gain).max(0.0);
    let eligible = total_gain - recapture_gain;
    let excluded = eligible.min(exclusion_limit);
    let taxable_ltcg = eligible - excluded;

    let tax = recapture_gain * input.recapture_rate_pct / 100.0
        + taxable_ltcg * input.ltcg_rate_pct / 100.0;

    HomeSaleResult {
        amount_realized_usd: amount_realized,
        adjusted_basis_usd: adjusted_basis,
        total_gain_usd: total_gain,
        exclusion_limit_usd: exclusion_limit,
        excluded_gain_usd: excluded,
        recapture_gain_usd: recapture_gain,
        taxable_ltcg_usd: taxable_ltcg,
        tax_usd: tax,
        after_tax_gain_usd: total_gain - tax,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-4
    }

    fn run(sale: f64, selling: f64, purchase: f64, imp: f64, status: FilingStatus, dep: f64) -> HomeSaleResult {
        analyze(&HomeSaleInput {
            sale_price_usd: sale,
            selling_costs_usd: selling,
            purchase_price_usd: purchase,
            improvements_usd: imp,
            filing_status: status,
            depreciation_taken_usd: dep,
            ltcg_rate_pct: 15.0,
            recapture_rate_pct: 25.0,
        })
    }

    #[test]
    fn basis_and_gain() {
        let r = run(600_000.0, 30_000.0, 300_000.0, 50_000.0, FilingStatus::Single, 0.0);
        assert!(close(r.amount_realized_usd, 570_000.0));
        assert!(close(r.adjusted_basis_usd, 350_000.0));
        assert!(close(r.total_gain_usd, 220_000.0));
    }

    #[test]
    fn small_gain_fully_excluded() {
        // 220k gain < 250k exclusion → no tax.
        let r = run(600_000.0, 30_000.0, 300_000.0, 50_000.0, FilingStatus::Single, 0.0);
        assert!(close(r.excluded_gain_usd, 220_000.0));
        assert!(close(r.tax_usd, 0.0));
    }

    #[test]
    fn married_joint_higher_exclusion() {
        // 400k gain: single taxes 150k, married-joint excludes it all.
        let single = run(700_000.0, 0.0, 300_000.0, 0.0, FilingStatus::Single, 0.0);
        let mfj = run(700_000.0, 0.0, 300_000.0, 0.0, FilingStatus::MarriedJoint, 0.0);
        assert!(close(single.taxable_ltcg_usd, 150_000.0));
        assert!(close(single.tax_usd, 22_500.0));
        assert!(close(mfj.taxable_ltcg_usd, 0.0));
        assert!(close(mfj.tax_usd, 0.0));
    }

    #[test]
    fn large_gain_single_taxed_above_exclusion() {
        // Gain 520k: exclude 250k → 270k LTCG × 15% = 40,500.
        let r = run(900_000.0, 30_000.0, 300_000.0, 50_000.0, FilingStatus::Single, 0.0);
        assert!(close(r.total_gain_usd, 520_000.0));
        assert!(close(r.taxable_ltcg_usd, 270_000.0));
        assert!(close(r.tax_usd, 40_500.0));
    }

    #[test]
    fn depreciation_recaptured_even_when_rest_excluded() {
        // 40k depreciation → basis drops, recapture 40k @25% = 10k, plus the
        // 270k LTCG @15% = 40,500 → 50,500.
        let r = run(900_000.0, 30_000.0, 300_000.0, 50_000.0, FilingStatus::Single, 40_000.0);
        assert!(close(r.total_gain_usd, 560_000.0));
        assert!(close(r.recapture_gain_usd, 40_000.0));
        assert!(close(r.taxable_ltcg_usd, 270_000.0));
        assert!(close(r.tax_usd, 50_500.0));
    }

    #[test]
    fn exclusion_capped_at_limit() {
        let r = run(900_000.0, 30_000.0, 300_000.0, 50_000.0, FilingStatus::Single, 0.0);
        assert!(close(r.excluded_gain_usd, 250_000.0));
    }

    #[test]
    fn after_tax_gain() {
        let r = run(900_000.0, 30_000.0, 300_000.0, 50_000.0, FilingStatus::Single, 0.0);
        assert!(close(r.after_tax_gain_usd, 520_000.0 - 40_500.0));
    }

    #[test]
    fn loss_has_no_tax() {
        let r = run(300_000.0, 20_000.0, 400_000.0, 0.0, FilingStatus::Single, 0.0);
        assert!(r.total_gain_usd < 0.0);
        assert!(close(r.tax_usd, 0.0));
    }
}
