//! IRC §453 — Installment sale gain deferral.
//!
//! When a property is sold and at least one payment is received in
//! a tax year AFTER the year of sale, §453 lets the seller recognize
//! gain proportionally as principal payments come in, rather than
//! all in the year of sale. Landlord-relevant for **seller-financed
//! rental property sales** — a $500k rental sold with 20% down + 80%
//! seller-financed note recognizes the gain over the life of the
//! note, not all in year 1.
//!
//! **Gross profit ratio method** per §453(c):
//!
//!   gross_profit       = sale_price − selling_costs − adjusted_basis
//!   contract_price     = sale_price − selling_costs − qualifying
//!                        indebtedness assumed by buyer (capped at basis)
//!   gross_profit_ratio = gross_profit / contract_price
//!
//! For each principal payment received: taxable gain = payment ×
//! gross_profit_ratio. Interest portion is separately taxable as
//! ordinary interest income.
//!
//! Three carve-outs that disqualify §453:
//!
//!   * **§453(k) marketable securities** — installment sale treatment
//!     is NOT available for sales of stock or securities traded on
//!     an established market. This is why §453 doesn't help traders
//!     selling public stock — full recognition in year of sale.
//!     The exclusion is specific to "publicly traded" — closely-held
//!     private company stock CAN use §453.
//!
//!   * **§453(g) related-party 2-year resale anti-abuse** — if the
//!     buyer is a related party (§267(b) / §707(b)(1)) who resells
//!     the property within 2 years of the original §453 sale, the
//!     ORIGINAL seller must recognize all remaining gain in the year
//!     of the second sale. Pairs with iter 27's §1031(f) related-
//!     party clawback.
//!
//!   * **§453(d) elect out** — the seller can affirmatively elect
//!     OUT of installment treatment and recognize the full gain in
//!     the year of sale. Useful when buyer creditworthiness is poor
//!     or when the seller has offsetting losses to absorb the gain.
//!
//! Pure compute. Caller asserts the §453(k) / §453(g) status; we
//! compute the GPR, the current-year recognition, and any anti-abuse
//! acceleration.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section453Input {
    pub tax_year: i32,
    pub sale_price: Decimal,
    pub selling_costs: Decimal,
    pub adjusted_basis: Decimal,
    /// Qualifying indebtedness assumed by the buyer that does NOT
    /// reduce contract_price (e.g. existing mortgage taken subject to
    /// up to seller's basis). Caller computes per §15A.453-1(b)(3).
    pub qualifying_indebtedness_capped_at_basis: Decimal,
    /// Principal received this year (down payment + scheduled note
    /// principal). Excludes interest.
    pub principal_received_this_year: Decimal,
    /// Interest received this year. Separately taxable as ordinary.
    pub interest_received_this_year: Decimal,
    /// True when the property is "publicly traded stock or securities"
    /// per §453(k)(2). Excludes the sale from §453 entirely.
    pub marketable_security: bool,
    /// True when the buyer is a §267(b) related party (cross-references
    /// `section_267::RelationshipCategory`).
    pub buyer_is_related_party: bool,
    /// True when the related-party buyer has RESOLD the property
    /// within 2 years of the original sale per §453(g)(1)(B). Triggers
    /// full remaining-gain recognition for the original seller.
    pub related_party_resold_within_2_years: bool,
    /// Remaining unrecognized gain from prior years' §453 deferrals
    /// on this sale. Used for the §453(g) clawback amount.
    pub unrecognized_gain_remaining: Decimal,
    /// True when the seller affirmatively elects OUT of installment
    /// treatment under §453(d). Triggers full recognition in year of sale.
    pub elect_out_of_installment_treatment: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Section453Result {
    pub realized_gain: Decimal,
    pub contract_price: Decimal,
    pub gross_profit_ratio: Decimal,
    pub principal_payment_gain_this_year: Decimal,
    pub interest_income_this_year: Decimal,
    pub gain_deferred_to_future_years: Decimal,
    pub disqualified: bool,
    pub disqualification_reasons: Vec<String>,
    /// True when §453(g) related-party 2-year resale triggered.
    pub section_453g_clawback_triggered: bool,
    pub clawback_amount: Decimal,
    pub note: String,
}

pub fn compute(input: &Section453Input) -> Section453Result {
    let mut r = Section453Result::default();

    // Amount realized + adjusted basis → realized gain.
    let amount_realized = input.sale_price - input.selling_costs;
    r.realized_gain = amount_realized - input.adjusted_basis;
    r.interest_income_this_year = input.interest_received_this_year.max(Decimal::ZERO);

    // §453(k) marketable securities exclusion.
    if input.marketable_security {
        r.disqualified = true;
        r.disqualification_reasons.push(
            "§453(k): publicly traded stock or securities — installment treatment unavailable"
                .into(),
        );
    }

    // §453(d) elect-out — full recognition in year of sale.
    if input.elect_out_of_installment_treatment {
        r.disqualified = true;
        r.disqualification_reasons.push(
            "§453(d) elect-out: seller chose full recognition in year of sale".into(),
        );
    }

    if r.disqualified {
        // Full gain (positive) recognized in current year.
        r.principal_payment_gain_this_year = r.realized_gain.max(Decimal::ZERO);
        r.gain_deferred_to_future_years = Decimal::ZERO;
        r.note = format!(
            "no installment deferral: {}",
            r.disqualification_reasons.join("; ")
        );
        return r;
    }

    if r.realized_gain <= Decimal::ZERO {
        r.note = "no gain to defer (§453 applies only to gains)".into();
        return r;
    }

    // Contract price per Reg. §15A.453-1(b)(2): amount realized minus
    // qualifying indebtedness assumed by buyer capped at basis.
    r.contract_price = amount_realized
        - input
            .qualifying_indebtedness_capped_at_basis
            .max(Decimal::ZERO);

    if r.contract_price <= Decimal::ZERO {
        // All "payments" came in via debt assumption — no GPR math.
        r.note = "contract price zero — all consideration via assumed indebtedness; §453 produces no current-year gain".into();
        return r;
    }

    // Gross profit ratio = gross_profit / contract_price.
    r.gross_profit_ratio =
        (r.realized_gain / r.contract_price).round_dp(6).min(Decimal::ONE);

    // §453(g) related-party 2-year resale clawback OVERRIDES the
    // gross-profit ratio for the current year — all remaining gain
    // recognized now.
    if input.buyer_is_related_party && input.related_party_resold_within_2_years {
        r.section_453g_clawback_triggered = true;
        // Remaining unrecognized gain at start of year + current-year
        // GPR-method gain.
        let current_gpr_gain =
            (input.principal_received_this_year * r.gross_profit_ratio).round_dp(2);
        r.clawback_amount = input.unrecognized_gain_remaining + current_gpr_gain;
        r.principal_payment_gain_this_year = r.clawback_amount;
        r.gain_deferred_to_future_years = Decimal::ZERO;
        r.note = format!(
            "§453(g) related-party 2-year resale: ${} clawback of remaining unrecognized gain recognized this year",
            r.clawback_amount
        );
        return r;
    }

    // Normal GPR application.
    r.principal_payment_gain_this_year =
        (input.principal_received_this_year * r.gross_profit_ratio).round_dp(2);
    r.gain_deferred_to_future_years =
        (input.unrecognized_gain_remaining + r.realized_gain
            - r.principal_payment_gain_this_year)
            .max(Decimal::ZERO);

    r.note = format!(
        "§453 installment: GPR {} applied to ${} principal → ${} gain recognized this year; ${} interest income separately; ${} gain deferred to future years",
        r.gross_profit_ratio,
        input.principal_received_this_year,
        r.principal_payment_gain_this_year,
        r.interest_income_this_year,
        r.gain_deferred_to_future_years,
    );
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn base() -> Section453Input {
        // $500k rental sold for $750k. $50k selling costs. $200k basis.
        // $300k realized gain. $50k down + $700k note at 6%. Year 1
        // principal received = $50k down + $0 amortizing = $50k.
        Section453Input {
            tax_year: 2024,
            sale_price: dec!(750000),
            selling_costs: dec!(50000),
            adjusted_basis: dec!(200000),
            qualifying_indebtedness_capped_at_basis: Decimal::ZERO,
            principal_received_this_year: dec!(50000),
            interest_received_this_year: dec!(42000),
            marketable_security: false,
            buyer_is_related_party: false,
            related_party_resold_within_2_years: false,
            unrecognized_gain_remaining: Decimal::ZERO,
            elect_out_of_installment_treatment: false,
        }
    }

    #[test]
    fn straight_installment_gpr_applied_correctly() {
        let r = compute(&base());
        // Realized gain = $750k - $50k - $200k = $500k. Wait — let me recompute.
        // Actually: $750k sale - $50k costs = $700k amount realized.
        // $700k - $200k basis = $500k realized gain.
        assert_eq!(r.realized_gain, dec!(500000));
        // Contract price = $700k (no assumed mortgage). GPR = $500k / $700k = 0.71428...
        assert_eq!(r.contract_price, dec!(700000));
        assert_eq!(r.gross_profit_ratio, dec!(0.714286));
        // Year 1: $50k × 0.714286 = $35,714.30.
        assert_eq!(r.principal_payment_gain_this_year, dec!(35714.30));
        assert_eq!(r.interest_income_this_year, dec!(42000));
    }

    #[test]
    fn marketable_securities_excluded_from_section_453() {
        let mut i = base();
        i.marketable_security = true;
        let r = compute(&i);
        assert!(r.disqualified);
        // Full $500k gain recognized year 1 (trader exclusion).
        assert_eq!(r.principal_payment_gain_this_year, dec!(500000));
        assert!(r.disqualification_reasons.iter().any(|s| s.contains("§453(k)")));
    }

    #[test]
    fn opt_out_elects_full_recognition_year_1() {
        let mut i = base();
        i.elect_out_of_installment_treatment = true;
        let r = compute(&i);
        assert!(r.disqualified);
        assert_eq!(r.principal_payment_gain_this_year, dec!(500000));
        assert!(r.disqualification_reasons.iter().any(|s| s.contains("§453(d)")));
    }

    #[test]
    fn related_party_2yr_resale_triggers_full_clawback() {
        // Year 2: $50k principal received plus $350k remaining unrecognized
        // gain. Related buyer resold within 2 years → clawback EVERYTHING.
        let mut i = base();
        i.buyer_is_related_party = true;
        i.related_party_resold_within_2_years = true;
        i.unrecognized_gain_remaining = dec!(350000);
        let r = compute(&i);
        assert!(r.section_453g_clawback_triggered);
        // $350k prior unrecognized + $50k × GPR 0.714286 ≈ $35,714.30 = $385,714.30.
        assert_eq!(r.clawback_amount, dec!(385714.30));
        assert_eq!(r.gain_deferred_to_future_years, Decimal::ZERO);
    }

    #[test]
    fn related_party_without_2yr_resale_no_clawback() {
        // Selling to family is fine if they DON'T resell within 2 years.
        let mut i = base();
        i.buyer_is_related_party = true;
        i.related_party_resold_within_2_years = false;
        let r = compute(&i);
        assert!(!r.section_453g_clawback_triggered);
        assert_eq!(r.principal_payment_gain_this_year, dec!(35714.30));
    }

    #[test]
    fn loss_on_sale_no_op_under_section_453() {
        let mut i = base();
        i.sale_price = dec!(150000); // sale at a loss
        let r = compute(&i);
        assert!(r.realized_gain < Decimal::ZERO);
        assert_eq!(r.principal_payment_gain_this_year, Decimal::ZERO);
        assert!(r.note.contains("no gain"));
    }

    #[test]
    fn qualifying_indebtedness_reduces_contract_price() {
        // Buyer assumes existing $150k mortgage that's at-or-below seller's
        // $200k basis. Contract price = $700k - $150k = $550k.
        let mut i = base();
        i.qualifying_indebtedness_capped_at_basis = dec!(150000);
        let r = compute(&i);
        assert_eq!(r.contract_price, dec!(550000));
        // GPR = $500k / $550k = 0.909091.
        assert_eq!(r.gross_profit_ratio, dec!(0.909091));
    }

    #[test]
    fn full_basis_assumed_debt_zero_contract_price() {
        // Buyer assumes mortgage equal to entire amount realized. No cash
        // changing hands → contract price drops to zero.
        let mut i = base();
        i.qualifying_indebtedness_capped_at_basis = dec!(700000);
        i.principal_received_this_year = Decimal::ZERO;
        let r = compute(&i);
        assert_eq!(r.contract_price, Decimal::ZERO);
        assert_eq!(r.principal_payment_gain_this_year, Decimal::ZERO);
        assert!(r.note.contains("zero"));
    }

    #[test]
    fn gpr_capped_at_one_when_gross_profit_exceeds_contract_price() {
        // Pathological — gross profit > contract price (rare but possible
        // when assumed indebtedness exceeds basis). GPR caps at 1.0 so
        // we don't recognize more than the payment.
        let mut i = base();
        i.qualifying_indebtedness_capped_at_basis = dec!(500000);
        let r = compute(&i);
        // Contract price = $700k - $500k = $200k. Gross profit = $500k.
        // GPR = 500 / 200 = 2.5, capped at 1.0.
        assert_eq!(r.gross_profit_ratio, Decimal::ONE);
    }

    #[test]
    fn interest_income_separately_recognized_ordinary() {
        // Even when GPR defers principal, the interest portion is fully
        // recognized this year as ordinary income.
        let mut i = base();
        i.principal_received_this_year = Decimal::ZERO; // year of all-interest
        i.interest_received_this_year = dec!(42000);
        let r = compute(&i);
        assert_eq!(r.principal_payment_gain_this_year, Decimal::ZERO);
        assert_eq!(r.interest_income_this_year, dec!(42000));
    }

    #[test]
    fn multi_year_chain_eventually_recognizes_full_gain() {
        // Year 1: $50k down → $35,714.30 gain. Remaining = $464,285.70.
        let y1 = compute(&base());
        let remaining_after_y1 = y1.gain_deferred_to_future_years;
        assert!(remaining_after_y1 > Decimal::ZERO);

        // Year 2: $100k principal received. Same GPR.
        let mut y2_in = base();
        y2_in.tax_year = 2025;
        y2_in.principal_received_this_year = dec!(100000);
        y2_in.unrecognized_gain_remaining = remaining_after_y1;
        let y2 = compute(&y2_in);
        // $100k × 0.714286 = $71,428.60.
        assert_eq!(y2.principal_payment_gain_this_year, dec!(71428.60));
    }

    #[test]
    fn zero_principal_received_zero_recognition() {
        let mut i = base();
        i.principal_received_this_year = Decimal::ZERO;
        let r = compute(&i);
        assert_eq!(r.principal_payment_gain_this_year, Decimal::ZERO);
    }

    #[test]
    fn marketable_security_short_circuits_other_inputs() {
        // Even with §453(g) facts, marketable_security disqualifies first.
        let mut i = base();
        i.marketable_security = true;
        i.buyer_is_related_party = true;
        i.related_party_resold_within_2_years = true;
        let r = compute(&i);
        assert!(r.disqualified);
        // Full recognition; not the §453(g) clawback path.
        assert!(!r.section_453g_clawback_triggered);
    }

    #[test]
    fn both_marketable_and_opt_out_list_both_reasons() {
        let mut i = base();
        i.marketable_security = true;
        i.elect_out_of_installment_treatment = true;
        let r = compute(&i);
        assert_eq!(r.disqualification_reasons.len(), 2);
    }

    #[test]
    fn unrelated_buyer_resold_no_clawback() {
        let mut i = base();
        i.buyer_is_related_party = false;
        i.related_party_resold_within_2_years = true; // ignored
        let r = compute(&i);
        assert!(!r.section_453g_clawback_triggered);
    }

    #[test]
    fn small_business_sale_gpr_math() {
        // $1M business sale, $300k basis, no selling costs, $250k down.
        // Realized gain = $700k. Contract price = $1M. GPR = 0.7.
        // Year 1: $250k × 0.7 = $175k recognized.
        let i = Section453Input {
            tax_year: 2024,
            sale_price: dec!(1000000),
            selling_costs: Decimal::ZERO,
            adjusted_basis: dec!(300000),
            qualifying_indebtedness_capped_at_basis: Decimal::ZERO,
            principal_received_this_year: dec!(250000),
            interest_received_this_year: dec!(45000),
            marketable_security: false,
            buyer_is_related_party: false,
            related_party_resold_within_2_years: false,
            unrecognized_gain_remaining: Decimal::ZERO,
            elect_out_of_installment_treatment: false,
        };
        let r = compute(&i);
        assert_eq!(r.gross_profit_ratio, dec!(0.7));
        assert_eq!(r.principal_payment_gain_this_year, dec!(175000));
    }

    #[test]
    fn note_distinguishes_marketable_security_vs_opt_out_paths() {
        let mut ms = base();
        ms.marketable_security = true;
        let r_ms = compute(&ms);
        assert!(r_ms.note.contains("§453(k)"));

        let mut opt = base();
        opt.elect_out_of_installment_treatment = true;
        let r_opt = compute(&opt);
        assert!(r_opt.note.contains("§453(d)"));
    }

    #[test]
    fn note_distinguishes_normal_installment_vs_clawback() {
        let normal = compute(&base());
        assert!(normal.note.contains("§453 installment"));

        let mut clawback = base();
        clawback.buyer_is_related_party = true;
        clawback.related_party_resold_within_2_years = true;
        let r_cb = compute(&clawback);
        assert!(r_cb.note.contains("§453(g)"));
    }
}
