//! Rental property disposition: §1250 unrecaptured depreciation
//! recapture + §1231 gain treatment + §1031 like-kind exchange deferral.
//!
//! When a landlord sells a property, the realized gain decomposes into
//! TWO buckets the tax code treats differently:
//!
//!   * **§1250 unrecaptured gain** — the portion of the gain attributable
//!     to prior depreciation deductions, taxed at a max 25% rate. This
//!     is the IRS clawing back depreciation that previously sheltered
//!     ordinary income at higher rates.
//!   * **§1231 gain** — the rest, taxed at long-term capital gains
//!     rates (0/15/20% depending on bracket).
//!
//! Formally: §1250 unrecaptured = min(accumulated_depreciation,
//! total_realized_gain). §1231 = total_realized_gain - §1250.
//!
//! If the seller rolls the proceeds into a replacement property via a
//! **§1031 like-kind exchange** (qualified intermediary, 45-day ID,
//! 180-day completion — out of scope here, caller asserts), gain is
//! DEFERRED to the extent of replacement value. Any cash or non-like-
//! kind property received ("boot") triggers gain recognition up to
//! the lesser of boot or realized gain.
//!
//! Replacement basis = adjusted basis of old property + boot paid
//! − boot received + gain recognized. This carries the deferred gain
//! into the new property.
//!
//! Pure compute. Caller supplies sale price, selling costs, original
//! cost, accumulated depreciation, and (optional) §1031 details. We
//! return the full breakdown so the user can copy line-by-line onto
//! Form 4797 + Form 8824.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Max federal rate on §1250 unrecaptured gain.
fn max_section_1250_rate() -> Decimal {
    Decimal::from_str("0.25").unwrap()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispositionInput {
    /// Gross sale price (HUD-1 line 401 / closing statement).
    pub sale_price: Decimal,
    /// Selling costs (broker commission, transfer tax, escrow fee, etc.)
    pub selling_costs: Decimal,
    /// Original cost basis (purchase price - land value + improvements).
    /// Land is NOT depreciable so the caller subtracts it for the
    /// depreciable basis side, but here we want the FULL cost basis
    /// (depreciable + land) because gain is computed against the full
    /// basis, not just the depreciable portion.
    pub original_cost_basis: Decimal,
    /// Sum of depreciation deductions taken over the holding period.
    /// This is what gets recaptured under §1250.
    pub accumulated_depreciation: Decimal,
    /// Capital improvements added to basis during ownership (over and
    /// above the original_cost_basis — e.g. a $30k roof capitalized
    /// in year 4). These increase the basis and reduce gain.
    pub capital_improvements_added: Decimal,
    /// Optional §1031 like-kind exchange details. Set to None for a
    /// straight sale (full recognition).
    pub like_kind_exchange: Option<LikeKindExchange>,
    /// Filing status — not needed for the breakdown (rates are caller's
    /// job) but echoed back for downstream tax form generators.
    pub filing_status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LikeKindExchange {
    /// Fair market value of replacement property received.
    pub replacement_property_value: Decimal,
    /// Cash received in the exchange (received boot). Cash PAID is
    /// already reflected in `replacement_property_value` exceeding
    /// the relinquished value.
    pub boot_received: Decimal,
    /// Mortgage / liabilities NETTED OUT (relinquished mortgage minus
    /// replacement mortgage, if positive — i.e. debt relief is boot).
    /// If replacement mortgage >= relinquished, this is zero (no net
    /// boot from debt).
    pub debt_relief_net: Decimal,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DispositionReport {
    /// Amount realized = sale price - selling costs.
    pub amount_realized: Decimal,
    /// Adjusted basis at sale = original cost - accumulated depreciation
    /// + capital improvements.
    pub adjusted_basis: Decimal,
    /// Realized gain = amount realized - adjusted basis. Negative for losses.
    pub realized_gain: Decimal,
    /// §1250 unrecaptured depreciation gain (taxed at max 25% federal).
    pub section_1250_unrecaptured_gain: Decimal,
    /// §1231 / LTCG gain — the residual after §1250.
    pub section_1231_ltcg_gain: Decimal,
    /// Gain recognized THIS year (full realized for straight sale; only
    /// boot received for §1031 exchange).
    pub gain_recognized_this_year: Decimal,
    /// Gain deferred via §1031.
    pub gain_deferred_section_1031: Decimal,
    /// Carryover basis into the replacement property.
    pub replacement_basis: Decimal,
    /// Federal §1250 tax at the 25% max bracket (estimate; caller may
    /// pay less if ordinary income bracket < 25%).
    pub max_section_1250_tax_estimate: Decimal,
    pub note: String,
}

pub fn compute(input: &DispositionInput) -> DispositionReport {
    let mut r = DispositionReport::default();

    r.amount_realized = input.sale_price - input.selling_costs;
    r.adjusted_basis = input.original_cost_basis - input.accumulated_depreciation
        + input.capital_improvements_added;
    r.realized_gain = r.amount_realized - r.adjusted_basis;

    if r.realized_gain <= Decimal::ZERO {
        r.note = if input.like_kind_exchange.is_some() {
            "loss — §1031 not applicable (losses are recognized in full per §1031(c))".into()
        } else {
            "loss on disposition — §1231 ordinary-loss treatment available".into()
        };
        r.section_1231_ltcg_gain = r.realized_gain; // negative
        r.gain_recognized_this_year = r.realized_gain;
        r.replacement_basis = input.original_cost_basis - input.accumulated_depreciation
            + input.capital_improvements_added;
        return r;
    }

    // §1250 unrecaptured = min(accumulated depreciation, realized gain).
    // The depreciation can't "recapture" more gain than actually exists.
    r.section_1250_unrecaptured_gain = input
        .accumulated_depreciation
        .min(r.realized_gain)
        .max(Decimal::ZERO);
    r.section_1231_ltcg_gain = r.realized_gain - r.section_1250_unrecaptured_gain;
    r.max_section_1250_tax_estimate = r.section_1250_unrecaptured_gain * max_section_1250_rate();

    match &input.like_kind_exchange {
        None => {
            r.gain_recognized_this_year = r.realized_gain;
            r.gain_deferred_section_1031 = Decimal::ZERO;
            r.replacement_basis = Decimal::ZERO; // N/A
            r.note = format!(
                "straight sale: ${} §1250 unrecaptured @ 25% + ${} §1231 LTCG",
                r.section_1250_unrecaptured_gain, r.section_1231_ltcg_gain
            );
        }
        Some(ex) => {
            // §1031: gain recognized = MIN(realized gain, boot received).
            // Boot = cash received + net debt relief.
            let boot_total = ex.boot_received + ex.debt_relief_net.max(Decimal::ZERO);
            r.gain_recognized_this_year = r.realized_gain.min(boot_total).max(Decimal::ZERO);
            r.gain_deferred_section_1031 = r.realized_gain - r.gain_recognized_this_year;

            // Replacement basis = adjusted basis (old) + boot paid
            //   - boot received + gain recognized.
            // Boot paid implicit when replacement_property_value >
            // (amount realized - boot received).
            let boot_paid = (ex.replacement_property_value
                - (r.amount_realized - ex.boot_received))
                .max(Decimal::ZERO);
            r.replacement_basis =
                r.adjusted_basis + boot_paid - ex.boot_received + r.gain_recognized_this_year;

            r.note = format!(
                "§1031 exchange: ${} deferred, ${} recognized as boot, replacement basis ${}",
                r.gain_deferred_section_1031, r.gain_recognized_this_year, r.replacement_basis
            );
        }
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn base_sale() -> DispositionInput {
        // $350k sale, $20k closing costs, $200k cost basis, $40k accumulated dep.
        // Expected: realized gain = $330k - $160k = $170k.
        // §1250 = min($40k, $170k) = $40k @ 25%.
        // §1231 = $130k.
        DispositionInput {
            sale_price: dec!(350000),
            selling_costs: dec!(20000),
            original_cost_basis: dec!(200000),
            accumulated_depreciation: dec!(40000),
            capital_improvements_added: Decimal::ZERO,
            like_kind_exchange: None,
            filing_status: None,
        }
    }

    #[test]
    fn straight_sale_breakdown_matches_form_4797() {
        let r = compute(&base_sale());
        assert_eq!(r.amount_realized, dec!(330000));
        assert_eq!(r.adjusted_basis, dec!(160000));
        assert_eq!(r.realized_gain, dec!(170000));
        assert_eq!(r.section_1250_unrecaptured_gain, dec!(40000));
        assert_eq!(r.section_1231_ltcg_gain, dec!(130000));
        assert_eq!(r.gain_recognized_this_year, dec!(170000));
        assert_eq!(r.gain_deferred_section_1031, Decimal::ZERO);
        assert_eq!(r.max_section_1250_tax_estimate, dec!(10000)); // 40k × 25%
    }

    #[test]
    fn capital_improvements_increase_basis_reducing_gain() {
        let mut i = base_sale();
        i.capital_improvements_added = dec!(30000);
        let r = compute(&i);
        // Adjusted basis goes up by $30k → realized gain drops by $30k.
        assert_eq!(r.adjusted_basis, dec!(190000));
        assert_eq!(r.realized_gain, dec!(140000));
    }

    #[test]
    fn section_1250_capped_at_total_gain() {
        // Accumulated depreciation $200k but realized gain only $50k:
        // §1250 caps at $50k (can't recapture more than the gain).
        let i = DispositionInput {
            sale_price: dec!(260000),
            selling_costs: dec!(10000),
            original_cost_basis: dec!(400000),
            accumulated_depreciation: dec!(200000),
            capital_improvements_added: Decimal::ZERO,
            like_kind_exchange: None,
            filing_status: None,
        };
        let r = compute(&i);
        assert_eq!(r.amount_realized, dec!(250000));
        assert_eq!(r.adjusted_basis, dec!(200000));
        assert_eq!(r.realized_gain, dec!(50000));
        assert_eq!(r.section_1250_unrecaptured_gain, dec!(50000));
        assert_eq!(r.section_1231_ltcg_gain, Decimal::ZERO);
    }

    #[test]
    fn loss_disposition_returns_negative_gain_no_section_1250() {
        // Selling at a loss — §1231 ordinary-loss treatment, no §1250.
        let i = DispositionInput {
            sale_price: dec!(150000),
            selling_costs: dec!(10000),
            original_cost_basis: dec!(200000),
            accumulated_depreciation: dec!(20000), // basis 180k
            capital_improvements_added: Decimal::ZERO,
            like_kind_exchange: None,
            filing_status: None,
        };
        let r = compute(&i);
        assert_eq!(r.amount_realized, dec!(140000));
        assert_eq!(r.adjusted_basis, dec!(180000));
        assert_eq!(r.realized_gain, dec!(-40000));
        assert_eq!(r.section_1250_unrecaptured_gain, Decimal::ZERO);
        assert!(r.note.contains("loss"));
    }

    #[test]
    fn section_1031_no_boot_full_deferral() {
        // Like-kind exchange, no boot, replacement worth $400k.
        // Realized gain $170k fully deferred.
        let mut i = base_sale();
        i.like_kind_exchange = Some(LikeKindExchange {
            replacement_property_value: dec!(400000),
            boot_received: Decimal::ZERO,
            debt_relief_net: Decimal::ZERO,
        });
        let r = compute(&i);
        assert_eq!(r.gain_recognized_this_year, Decimal::ZERO);
        assert_eq!(r.gain_deferred_section_1031, dec!(170000));
        assert!(r.note.contains("§1031"));
    }

    #[test]
    fn section_1031_boot_recognized_up_to_realized_gain() {
        // $25k cash boot received. Gain recognized = min($170k, $25k) = $25k.
        let mut i = base_sale();
        i.like_kind_exchange = Some(LikeKindExchange {
            replacement_property_value: dec!(305000),
            boot_received: dec!(25000),
            debt_relief_net: Decimal::ZERO,
        });
        let r = compute(&i);
        assert_eq!(r.gain_recognized_this_year, dec!(25000));
        assert_eq!(r.gain_deferred_section_1031, dec!(145000));
    }

    #[test]
    fn section_1031_boot_exceeds_gain_caps_recognition() {
        // $200k cash boot received but realized gain only $170k.
        // Recognized = $170k (can't recognize more than realized).
        let mut i = base_sale();
        i.like_kind_exchange = Some(LikeKindExchange {
            replacement_property_value: dec!(130000),
            boot_received: dec!(200000),
            debt_relief_net: Decimal::ZERO,
        });
        let r = compute(&i);
        assert_eq!(r.gain_recognized_this_year, dec!(170000));
        assert_eq!(r.gain_deferred_section_1031, Decimal::ZERO);
    }

    #[test]
    fn section_1031_debt_relief_counts_as_boot() {
        // Old mortgage $150k, new mortgage $100k → net debt relief = $50k.
        // Counted as boot, gain recognized = min($170k, $50k) = $50k.
        let mut i = base_sale();
        i.like_kind_exchange = Some(LikeKindExchange {
            replacement_property_value: dec!(330000),
            boot_received: Decimal::ZERO,
            debt_relief_net: dec!(50000),
        });
        let r = compute(&i);
        assert_eq!(r.gain_recognized_this_year, dec!(50000));
        assert_eq!(r.gain_deferred_section_1031, dec!(120000));
    }

    #[test]
    fn section_1031_replacement_basis_carries_deferred_gain() {
        // Adjusted basis = $160k. Full deferral.
        // Replacement basis = $160k + boot paid ($70k, since
        //   replacement $400k vs amount realized $330k) - 0 + 0 = $230k.
        // Verify the deferred gain ($170k) is reflected:
        //   replacement_value - replacement_basis = $400k - $230k = $170k. ✓
        let mut i = base_sale();
        i.like_kind_exchange = Some(LikeKindExchange {
            replacement_property_value: dec!(400000),
            boot_received: Decimal::ZERO,
            debt_relief_net: Decimal::ZERO,
        });
        let r = compute(&i);
        let deferred_check = dec!(400000) - r.replacement_basis;
        assert_eq!(deferred_check, r.gain_deferred_section_1031);
    }

    #[test]
    fn loss_does_not_qualify_for_1031_recognition_under_1031c() {
        // §1031(c): losses are recognized in full — §1031 doesn't apply.
        let mut i = base_sale();
        i.sale_price = dec!(100000);
        i.like_kind_exchange = Some(LikeKindExchange {
            replacement_property_value: dec!(150000),
            boot_received: Decimal::ZERO,
            debt_relief_net: Decimal::ZERO,
        });
        let r = compute(&i);
        assert!(r.realized_gain < Decimal::ZERO);
        assert_eq!(r.gain_recognized_this_year, r.realized_gain); // full loss
        assert!(r.note.contains("§1031(c)") || r.note.contains("loss"));
    }

    #[test]
    fn max_section_1250_tax_estimate_is_25_pct_of_unrecaptured() {
        let r = compute(&base_sale());
        assert_eq!(
            r.max_section_1250_tax_estimate,
            r.section_1250_unrecaptured_gain * dec!(0.25)
        );
    }

    #[test]
    fn zero_gain_zero_section_1250() {
        // Sell for exactly basis.
        let i = DispositionInput {
            sale_price: dec!(180000),
            selling_costs: dec!(0),
            original_cost_basis: dec!(200000),
            accumulated_depreciation: dec!(20000),
            capital_improvements_added: Decimal::ZERO,
            like_kind_exchange: None,
            filing_status: None,
        };
        let r = compute(&i);
        assert_eq!(r.realized_gain, Decimal::ZERO);
        assert_eq!(r.section_1250_unrecaptured_gain, Decimal::ZERO);
        assert_eq!(r.section_1231_ltcg_gain, Decimal::ZERO);
    }

    #[test]
    fn no_depreciation_means_all_gain_is_section_1231() {
        // Bought, held, never depreciated (e.g. raw land). All gain is §1231.
        let i = DispositionInput {
            sale_price: dec!(300000),
            selling_costs: dec!(15000),
            original_cost_basis: dec!(150000),
            accumulated_depreciation: Decimal::ZERO,
            capital_improvements_added: Decimal::ZERO,
            like_kind_exchange: None,
            filing_status: None,
        };
        let r = compute(&i);
        assert_eq!(r.realized_gain, dec!(135000));
        assert_eq!(r.section_1250_unrecaptured_gain, Decimal::ZERO);
        assert_eq!(r.section_1231_ltcg_gain, dec!(135000));
    }
}
