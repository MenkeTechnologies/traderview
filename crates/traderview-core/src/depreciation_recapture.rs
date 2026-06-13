//! Depreciation recapture on a rental sale — unrecaptured § 1250 gain.
//!
//! Depreciation lowers a rental's basis while you hold it (sheltering rental
//! income); when you sell, that benefit is partly clawed back. For real
//! property depreciated straight-line under MACRS, the depreciation portion
//! of the gain is **unrecaptured § 1250 gain**, taxed at a maximum 25% rate
//! (IRC § 1(h)(1)(E)); the appreciation above the original basis is regular
//! long-term capital gain (0/15/20%).
//!
//!   * adjusted basis = purchase + improvements − accumulated depreciation
//!   * total gain     = (sale price − selling costs) − adjusted basis
//!   * unrecaptured § 1250 = min(accumulated depreciation, total gain)
//!   * LTCG gain      = total gain − unrecaptured § 1250
//!
//! Recapture is limited to the lesser of the depreciation taken or the total
//! gain — a sale at a loss recaptures nothing. NIIT (3.8%) may apply on top
//! but is out of scope here. Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct RecaptureInput {
    pub purchase_price_usd: f64,
    #[serde(default)]
    pub improvements_usd: f64,
    /// Total depreciation taken (or allowable) over the holding period.
    pub accumulated_depreciation_usd: f64,
    pub sale_price_usd: f64,
    #[serde(default)]
    pub selling_costs_usd: f64,
    /// Long-term cap-gains rate on the appreciation portion (e.g. 15 or 20).
    pub ltcg_rate_pct: f64,
    /// Max rate on unrecaptured § 1250 gain (25% cap; lower it if your
    /// ordinary rate is below 25%).
    pub recapture_rate_pct: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct RecaptureResult {
    pub adjusted_basis_usd: f64,
    pub amount_realized_usd: f64,
    pub total_gain_usd: f64,
    /// Depreciation portion of the gain (taxed at the recapture rate).
    pub unrecaptured_1250_gain_usd: f64,
    /// Appreciation portion above original basis (taxed at LTCG).
    pub ltcg_gain_usd: f64,
    pub recapture_tax_usd: f64,
    pub ltcg_tax_usd: f64,
    pub total_tax_usd: f64,
    /// Total tax as a percent of the total gain.
    pub effective_rate_pct: f64,
    pub after_tax_gain_usd: f64,
    /// True when the sale is at a loss (no gain, no recapture).
    pub is_loss: bool,
}

pub fn analyze(i: &RecaptureInput) -> RecaptureResult {
    let depreciation = i.accumulated_depreciation_usd.max(0.0);
    let adjusted_basis = i.purchase_price_usd + i.improvements_usd - depreciation;
    let amount_realized = i.sale_price_usd - i.selling_costs_usd;
    let total_gain = amount_realized - adjusted_basis;

    if total_gain <= 0.0 {
        return RecaptureResult {
            adjusted_basis_usd: adjusted_basis,
            amount_realized_usd: amount_realized,
            total_gain_usd: total_gain,
            unrecaptured_1250_gain_usd: 0.0,
            ltcg_gain_usd: 0.0,
            recapture_tax_usd: 0.0,
            ltcg_tax_usd: 0.0,
            total_tax_usd: 0.0,
            effective_rate_pct: 0.0,
            after_tax_gain_usd: total_gain,
            is_loss: true,
        };
    }

    // Recapture is the lesser of depreciation taken or the total gain.
    let unrecaptured = depreciation.min(total_gain);
    let ltcg_gain = total_gain - unrecaptured;

    let recapture_tax = unrecaptured * i.recapture_rate_pct / 100.0;
    let ltcg_tax = ltcg_gain * i.ltcg_rate_pct / 100.0;
    let total_tax = recapture_tax + ltcg_tax;
    let effective_rate = if total_gain > 0.0 { total_tax / total_gain * 100.0 } else { 0.0 };

    RecaptureResult {
        adjusted_basis_usd: adjusted_basis,
        amount_realized_usd: amount_realized,
        total_gain_usd: total_gain,
        unrecaptured_1250_gain_usd: unrecaptured,
        ltcg_gain_usd: ltcg_gain,
        recapture_tax_usd: recapture_tax,
        ltcg_tax_usd: ltcg_tax,
        total_tax_usd: total_tax,
        effective_rate_pct: effective_rate,
        after_tax_gain_usd: total_gain - total_tax,
        is_loss: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> RecaptureInput {
        RecaptureInput {
            purchase_price_usd: 200_000.0,
            improvements_usd: 0.0,
            accumulated_depreciation_usd: 50_000.0,
            sale_price_usd: 300_000.0,
            selling_costs_usd: 0.0,
            ltcg_rate_pct: 15.0,
            recapture_rate_pct: 25.0,
        }
    }

    #[test]
    fn basis_gain_and_split() {
        // basis 200k − 50k dep = 150k; gain = 300k − 150k = 150k.
        // unrecaptured = min(50k, 150k) = 50k; LTCG gain = 100k.
        let r = analyze(&base());
        assert!((r.adjusted_basis_usd - 150_000.0).abs() < 1e-6);
        assert!((r.total_gain_usd - 150_000.0).abs() < 1e-6);
        assert!((r.unrecaptured_1250_gain_usd - 50_000.0).abs() < 1e-6);
        assert!((r.ltcg_gain_usd - 100_000.0).abs() < 1e-6);
    }

    #[test]
    fn taxes_split_recapture_and_ltcg() {
        // recapture 50k × 25% = 12.5k; LTCG 100k × 15% = 15k; total 27.5k.
        let r = analyze(&base());
        assert!((r.recapture_tax_usd - 12_500.0).abs() < 1e-6);
        assert!((r.ltcg_tax_usd - 15_000.0).abs() < 1e-6);
        assert!((r.total_tax_usd - 27_500.0).abs() < 1e-6);
    }

    #[test]
    fn recapture_limited_to_total_gain() {
        // Depreciation 50k but total gain only 30k → recapture caps at 30k, LTCG 0.
        let r = analyze(&RecaptureInput { sale_price_usd: 180_000.0, ..base() });
        // basis 150k; gain = 180k − 150k = 30k.
        assert!((r.total_gain_usd - 30_000.0).abs() < 1e-6);
        assert!((r.unrecaptured_1250_gain_usd - 30_000.0).abs() < 1e-6);
        assert!(r.ltcg_gain_usd.abs() < 1e-6);
    }

    #[test]
    fn sale_at_loss_recaptures_nothing() {
        // Sell for 140k < adjusted basis 150k → loss.
        let r = analyze(&RecaptureInput { sale_price_usd: 140_000.0, ..base() });
        assert!(r.is_loss);
        assert!(r.total_tax_usd.abs() < 1e-9);
        assert!(r.total_gain_usd < 0.0);
    }

    #[test]
    fn selling_costs_reduce_amount_realized() {
        let r = analyze(&RecaptureInput { selling_costs_usd: 18_000.0, ..base() });
        assert!((r.amount_realized_usd - 282_000.0).abs() < 1e-6);
        // gain = 282k − 150k = 132k.
        assert!((r.total_gain_usd - 132_000.0).abs() < 1e-6);
    }

    #[test]
    fn improvements_raise_basis() {
        let r = analyze(&RecaptureInput { improvements_usd: 40_000.0, ..base() });
        // basis = 200k + 40k − 50k = 190k; gain = 300k − 190k = 110k.
        assert!((r.adjusted_basis_usd - 190_000.0).abs() < 1e-6);
        assert!((r.total_gain_usd - 110_000.0).abs() < 1e-6);
    }

    #[test]
    fn zero_depreciation_is_all_ltcg() {
        let r = analyze(&RecaptureInput { accumulated_depreciation_usd: 0.0, ..base() });
        assert!(r.unrecaptured_1250_gain_usd.abs() < 1e-9);
        assert!((r.ltcg_gain_usd - r.total_gain_usd).abs() < 1e-9);
    }

    #[test]
    fn effective_rate_between_ltcg_and_recapture() {
        let r = analyze(&base());
        // 27.5k / 150k = 18.33% — between 15% (LTCG) and 25% (recapture).
        assert!((r.effective_rate_pct - (27_500.0 / 150_000.0 * 100.0)).abs() < 1e-9);
        assert!(r.effective_rate_pct > 15.0 && r.effective_rate_pct < 25.0);
    }
}
