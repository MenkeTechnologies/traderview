//! § 1031 like-kind exchange — boot, recognized vs deferred gain, new basis.
//!
//! Swapping one investment property for another defers the gain — but only
//! to the extent you fully reinvest. **Boot** is non-like-kind value you walk
//! away with, and it's taxable now:
//!
//!   * **cash boot** — cash received in the exchange.
//!   * **mortgage boot** — net debt relief: old mortgage − new mortgage, when
//!     you trade down to less debt (floored at zero; taking on more debt is
//!     not boot).
//!
//! Recognized (taxable) gain = the lesser of the realized gain or the total
//! boot; the rest is deferred. The replacement property takes a carryover
//! basis = its cost − the deferred gain, which preserves the gain for a later
//! sale. A loss isn't recognized in a § 1031 exchange. This is the simplified
//! boot model (cash-paid/extra-debt netting is not applied); pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct ExchangeInput {
    /// FMV of the property given up (sale price).
    pub relinquished_sale_price_usd: f64,
    pub relinquished_adjusted_basis_usd: f64,
    /// Mortgage relieved on the relinquished property.
    #[serde(default)]
    pub relinquished_mortgage_usd: f64,
    /// Cost / FMV of the replacement property acquired.
    pub replacement_purchase_price_usd: f64,
    /// New mortgage taken on the replacement property.
    #[serde(default)]
    pub replacement_mortgage_usd: f64,
    /// Cash boot received in the exchange.
    #[serde(default)]
    pub cash_received_usd: f64,
    /// Exchange / selling expenses (reduce the realized gain).
    #[serde(default)]
    pub selling_costs_usd: f64,
    /// Blended rate on the recognized gain (LTCG + recapture, user-set).
    pub capital_gains_tax_pct: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExchangeResult {
    pub realized_gain_usd: f64,
    pub cash_boot_usd: f64,
    /// Net debt relief treated as boot.
    pub mortgage_boot_usd: f64,
    pub total_boot_usd: f64,
    /// Gain taxable now = min(realized gain, total boot).
    pub recognized_gain_usd: f64,
    /// Gain rolled into the replacement property.
    pub deferred_gain_usd: f64,
    pub tax_now_usd: f64,
    /// Carryover basis of the replacement property = cost − deferred gain.
    pub replacement_basis_usd: f64,
    /// True when no gain is recognized (fully deferred).
    pub fully_deferred: bool,
}

pub fn analyze(i: &ExchangeInput) -> ExchangeResult {
    let realized_gain =
        (i.relinquished_sale_price_usd - i.selling_costs_usd) - i.relinquished_adjusted_basis_usd;

    let cash_boot = i.cash_received_usd.max(0.0);
    let mortgage_boot = (i.relinquished_mortgage_usd - i.replacement_mortgage_usd).max(0.0);
    let total_boot = cash_boot + mortgage_boot;

    // No gain → nothing to recognize or defer (losses aren't recognized).
    let positive_gain = realized_gain.max(0.0);
    let recognized = positive_gain.min(total_boot);
    let deferred = positive_gain - recognized;

    let tax_now = recognized * i.capital_gains_tax_pct / 100.0;
    // Carryover basis preserves the deferred gain in the new property.
    let replacement_basis = i.replacement_purchase_price_usd - deferred;

    ExchangeResult {
        realized_gain_usd: realized_gain,
        cash_boot_usd: cash_boot,
        mortgage_boot_usd: mortgage_boot,
        total_boot_usd: total_boot,
        recognized_gain_usd: recognized,
        deferred_gain_usd: deferred,
        tax_now_usd: tax_now,
        replacement_basis_usd: replacement_basis,
        fully_deferred: recognized <= 0.0 && realized_gain > 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> ExchangeInput {
        ExchangeInput {
            relinquished_sale_price_usd: 500_000.0,
            relinquished_adjusted_basis_usd: 300_000.0,
            relinquished_mortgage_usd: 200_000.0,
            replacement_purchase_price_usd: 600_000.0,
            replacement_mortgage_usd: 300_000.0,
            cash_received_usd: 0.0,
            selling_costs_usd: 0.0,
            capital_gains_tax_pct: 20.0,
        }
    }

    #[test]
    fn full_exchange_defers_all_gain() {
        // Realized 200k; trade up in debt (200k→300k) → no mortgage boot; no cash.
        let r = analyze(&base());
        assert!((r.realized_gain_usd - 200_000.0).abs() < 1e-6);
        assert!(r.total_boot_usd.abs() < 1e-9);
        assert!(r.recognized_gain_usd.abs() < 1e-9);
        assert!((r.deferred_gain_usd - 200_000.0).abs() < 1e-6);
        assert!(r.fully_deferred);
        assert!(r.tax_now_usd.abs() < 1e-9);
    }

    #[test]
    fn carryover_basis_preserves_deferred_gain() {
        // Replacement 600k − 200k deferred = 400k carryover basis.
        let r = analyze(&base());
        assert!((r.replacement_basis_usd - 400_000.0).abs() < 1e-6);
    }

    #[test]
    fn cash_boot_is_recognized() {
        let r = analyze(&ExchangeInput { cash_received_usd: 50_000.0, ..base() });
        assert!((r.cash_boot_usd - 50_000.0).abs() < 1e-6);
        assert!((r.recognized_gain_usd - 50_000.0).abs() < 1e-6); // min(200k, 50k)
        assert!((r.deferred_gain_usd - 150_000.0).abs() < 1e-6);
        assert!((r.tax_now_usd - 10_000.0).abs() < 1e-6); // 50k × 20%
        assert!(!r.fully_deferred);
    }

    #[test]
    fn trading_down_debt_creates_mortgage_boot() {
        // Old mtg 200k, new mtg 120k → 80k net relief = mortgage boot.
        let r = analyze(&ExchangeInput { replacement_mortgage_usd: 120_000.0, ..base() });
        assert!((r.mortgage_boot_usd - 80_000.0).abs() < 1e-6);
        assert!((r.recognized_gain_usd - 80_000.0).abs() < 1e-6);
    }

    #[test]
    fn recognized_gain_capped_at_realized_gain() {
        // Small gain (basis 480k → realized 20k) but big boot (cash 50k).
        let r = analyze(&ExchangeInput {
            relinquished_adjusted_basis_usd: 480_000.0,
            cash_received_usd: 50_000.0,
            ..base()
        });
        assert!((r.realized_gain_usd - 20_000.0).abs() < 1e-6);
        assert!((r.recognized_gain_usd - 20_000.0).abs() < 1e-6); // capped at the 20k gain
        assert!(r.deferred_gain_usd.abs() < 1e-9);
    }

    #[test]
    fn loss_is_not_recognized() {
        // Sale below basis → realized loss; nothing recognized or deferred.
        let r = analyze(&ExchangeInput { relinquished_sale_price_usd: 250_000.0, ..base() });
        assert!(r.realized_gain_usd < 0.0);
        assert!(r.recognized_gain_usd.abs() < 1e-9);
        assert!(r.deferred_gain_usd.abs() < 1e-9);
        assert!(!r.fully_deferred);
    }

    #[test]
    fn selling_costs_reduce_realized_gain() {
        let r = analyze(&ExchangeInput { selling_costs_usd: 30_000.0, ..base() });
        // (500k − 30k) − 300k = 170k.
        assert!((r.realized_gain_usd - 170_000.0).abs() < 1e-6);
    }

    #[test]
    fn cash_and_mortgage_boot_combine() {
        let r = analyze(&ExchangeInput {
            cash_received_usd: 25_000.0,
            replacement_mortgage_usd: 150_000.0, // 50k debt relief
            ..base()
        });
        assert!((r.total_boot_usd - 75_000.0).abs() < 1e-6); // 25k + 50k
        assert!((r.recognized_gain_usd - 75_000.0).abs() < 1e-6);
    }
}
