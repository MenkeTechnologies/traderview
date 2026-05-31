//! IRC §267 — Losses, expenses, and interest with respect to
//! transactions between related taxpayers.
//!
//! §267(a)(1) flatly **disallows** any loss on the sale or exchange
//! of property between related persons. The disallowed loss is not
//! "lost" forever — under §267(d), when the related-party buyer
//! later sells the property at a gain, that subsequent gain is
//! REDUCED (down to zero) by the previously-disallowed loss. If the
//! buyer sells at a loss, the seller's disallowed amount is gone
//! permanently.
//!
//! This catches the obvious tax-shelter move (sell at a loss to
//! your spouse / kid / your own S-corp to recognize the loss while
//! keeping the stock in the family) AND a long list of less-obvious
//! relationships — controlled corporations, trust-and-beneficiary
//! pairs, partnership-corp combos with >50% common ownership, etc.
//!
//! §267(b) enumerates ten categories of related persons. §267(c)
//! adds constructive-ownership rules (family attribution and
//! entity-to-owner attribution) that frequently surprise traders.
//! This module checks a relationship and computes the disallowance
//! + §267(d) basis-adjustment outcome.
//!
//! Pure compute. Caller asserts the §267(b) category (or `Unrelated`
//! when nobody related is involved); we run the §267(a)(1) check and
//! compute the §267(d) buyer-side adjustment if a subsequent gain
//! is supplied.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// §267(b) — ten categories of related persons. `Unrelated` means
/// §267 doesn't apply at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipCategory {
    /// §267(b)(1) — family members per §267(c)(4): brothers and
    /// sisters (whole or half blood), spouse, ancestors, lineal
    /// descendants. Notably does NOT include in-laws, cousins, aunts/uncles.
    FamilyMember,
    /// §267(b)(2) — individual and a corporation in which the individual
    /// owns directly or indirectly more than 50% of the value of the
    /// outstanding stock.
    IndividualAndControlledCorp,
    /// §267(b)(3) — two corporations that are members of the same
    /// controlled group (§1563(a) >50% common ownership).
    TwoControlledCorps,
    /// §267(b)(4) — trust grantor and trust fiduciary.
    GrantorAndTrustFiduciary,
    /// §267(b)(5) — fiduciary of one trust and fiduciary of another
    /// trust with the same grantor.
    TwoTrustFiduciariesSameGrantor,
    /// §267(b)(6) — trust fiduciary and beneficiary of the same trust.
    TrustFiduciaryAndBeneficiary,
    /// §267(b)(7) — fiduciary of a trust and beneficiary of ANOTHER
    /// trust with the same grantor.
    TrustFiduciaryAndOtherBeneficiary,
    /// §267(b)(8) — corporation and partnership where the same person
    /// owns more than 50% of the corp value AND more than 50% of the
    /// partnership P&L interest. Frequent gotcha for trader LLC + S-corp.
    CorpAndPartnershipCommonOwner,
    /// §267(b)(9) — S corporation and another S corporation if the
    /// same person owns more than 50% of each.
    TwoSCorps,
    /// §267(b)(10) — executor of an estate and beneficiary of the
    /// same estate. Sales between executor and beneficiary disallowed.
    EstateExecutorAndBeneficiary,
    /// Not §267(b)-related. §267(a)(1) doesn't apply.
    Unrelated,
}

impl RelationshipCategory {
    pub fn is_related(self) -> bool {
        !matches!(self, RelationshipCategory::Unrelated)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section267Input {
    /// Loss realized on the sale (positive number — we're computing
    /// a loss, sign is implicit). Pass zero for non-loss dispositions.
    pub realized_loss: Decimal,
    pub relationship: RelationshipCategory,
    /// §267(d): when the buyer later sells the property at a gain,
    /// that gain is reduced (down to zero) by the previously
    /// disallowed loss. Caller may pass the buyer's subsequent gain
    /// to project the §267(d) outcome. Pass `None` if buyer hasn't
    /// sold yet — we just compute the disallowance side.
    pub buyer_subsequent_gain: Option<Decimal>,
    /// Buyer's cash purchase price (basis for §267(d) tracking).
    pub buyer_purchase_price: Decimal,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Section267Result {
    pub is_related_party: bool,
    /// Loss recognized by the SELLER this year. Zero when §267(a)(1)
    /// applies; equals realized_loss when relationship is Unrelated.
    pub loss_recognized_by_seller: Decimal,
    /// Loss disallowed under §267(a)(1) — preserved for the §267(d)
    /// buyer-side reduction.
    pub loss_disallowed: Decimal,
    /// Buyer's initial basis = their actual cash purchase price (NOT
    /// the seller's old basis — §267 doesn't transfer basis, only the
    /// later-gain-reduction right).
    pub buyer_initial_basis: Decimal,
    /// Reduction the buyer applies to a subsequent gain under
    /// §267(d). Capped at the buyer's actual subsequent gain — the
    /// reduction cannot create a loss.
    pub buyer_section_267d_reduction: Decimal,
    /// Buyer's recognized gain after §267(d) reduction.
    pub buyer_recognized_gain_after_267d: Decimal,
    /// Loss permanently lost: only relevant when buyer sold at a
    /// loss or hasn't sold yet. = loss_disallowed - reduction-applied.
    pub loss_permanently_lost_if_buyer_no_gain: Decimal,
    pub note: String,
}

pub fn compute(input: &Section267Input) -> Section267Result {
    let mut r = Section267Result {
        is_related_party: input.relationship.is_related(),
        buyer_initial_basis: input.buyer_purchase_price,
        ..Section267Result::default()
    };

    if input.realized_loss <= Decimal::ZERO {
        r.note = "no loss to disallow (§267 only applies to losses)".into();
        return r;
    }

    if !r.is_related_party {
        r.loss_recognized_by_seller = input.realized_loss;
        r.note = "unrelated party — §267 does not apply; full loss recognized".into();
        return r;
    }

    // Related party: full loss DISALLOWED to seller.
    r.loss_disallowed = input.realized_loss;
    r.loss_recognized_by_seller = Decimal::ZERO;

    // §267(d): apply to buyer's subsequent gain.
    match input.buyer_subsequent_gain {
        Some(g) if g > Decimal::ZERO => {
            r.buyer_section_267d_reduction = g.min(input.realized_loss);
            r.buyer_recognized_gain_after_267d = g - r.buyer_section_267d_reduction;
            r.loss_permanently_lost_if_buyer_no_gain =
                input.realized_loss - r.buyer_section_267d_reduction;
            r.note = format!(
                "§267(a)(1) disallowed ${} seller loss; §267(d) reduced buyer's ${} gain by ${}, leaving ${} taxable. ${} of original loss permanently lost.",
                input.realized_loss, g, r.buyer_section_267d_reduction,
                r.buyer_recognized_gain_after_267d,
                r.loss_permanently_lost_if_buyer_no_gain,
            );
        }
        Some(_) => {
            // Buyer sold at a loss / no gain — no §267(d) reduction available.
            r.loss_permanently_lost_if_buyer_no_gain = input.realized_loss;
            r.note = format!(
                "§267(a)(1) disallowed ${} seller loss; buyer's subsequent sale yielded no gain so the §267(d) reduction is unavailable — entire ${} loss permanently lost.",
                input.realized_loss, input.realized_loss
            );
        }
        None => {
            // Buyer hasn't sold yet — full disallowance pending §267(d).
            r.loss_permanently_lost_if_buyer_no_gain = input.realized_loss;
            r.note = format!(
                "§267(a)(1) disallowed ${} seller loss; preserved for §267(d) reduction of buyer's future gain (if any).",
                input.realized_loss
            );
        }
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn base() -> Section267Input {
        Section267Input {
            realized_loss: dec!(10000),
            relationship: RelationshipCategory::FamilyMember,
            buyer_subsequent_gain: None,
            buyer_purchase_price: dec!(40000),
        }
    }

    #[test]
    fn unrelated_full_loss_recognized() {
        let mut i = base();
        i.relationship = RelationshipCategory::Unrelated;
        let r = compute(&i);
        assert_eq!(r.loss_recognized_by_seller, dec!(10000));
        assert_eq!(r.loss_disallowed, Decimal::ZERO);
        assert!(!r.is_related_party);
    }

    #[test]
    fn family_member_full_loss_disallowed() {
        let r = compute(&base());
        assert_eq!(r.loss_recognized_by_seller, Decimal::ZERO);
        assert_eq!(r.loss_disallowed, dec!(10000));
        assert!(r.is_related_party);
        assert!(r.note.contains("§267(a)(1)"));
    }

    #[test]
    fn no_loss_no_op() {
        let mut i = base();
        i.realized_loss = Decimal::ZERO;
        let r = compute(&i);
        assert_eq!(r.loss_disallowed, Decimal::ZERO);
        assert!(r.note.contains("no loss"));
    }

    #[test]
    fn section_267d_buyer_gain_reduced_by_disallowed_loss() {
        // Seller disallowed $10k. Buyer later sells for $20k gain.
        // §267(d) reduces buyer's gain by $10k → $10k taxable.
        let mut i = base();
        i.buyer_subsequent_gain = Some(dec!(20000));
        let r = compute(&i);
        assert_eq!(r.buyer_section_267d_reduction, dec!(10000));
        assert_eq!(r.buyer_recognized_gain_after_267d, dec!(10000));
        assert_eq!(r.loss_permanently_lost_if_buyer_no_gain, Decimal::ZERO);
    }

    #[test]
    fn section_267d_reduction_capped_at_buyer_gain() {
        // Seller disallowed $10k. Buyer's gain only $3k.
        // Reduction = min($10k, $3k) = $3k. $7k permanently lost.
        let mut i = base();
        i.buyer_subsequent_gain = Some(dec!(3000));
        let r = compute(&i);
        assert_eq!(r.buyer_section_267d_reduction, dec!(3000));
        assert_eq!(r.buyer_recognized_gain_after_267d, Decimal::ZERO);
        assert_eq!(r.loss_permanently_lost_if_buyer_no_gain, dec!(7000));
    }

    #[test]
    fn section_267d_buyer_loss_loses_entire_disallowance() {
        // Buyer sold at a loss → §267(d) reduction unavailable.
        let mut i = base();
        i.buyer_subsequent_gain = Some(dec!(-5000));
        let r = compute(&i);
        assert_eq!(r.buyer_section_267d_reduction, Decimal::ZERO);
        assert_eq!(r.loss_permanently_lost_if_buyer_no_gain, dec!(10000));
        assert!(r.note.contains("permanently lost"));
    }

    #[test]
    fn buyer_not_yet_sold_preserves_disallowance_for_future() {
        let r = compute(&base());
        assert_eq!(r.loss_permanently_lost_if_buyer_no_gain, dec!(10000));
        assert!(r.note.contains("preserved for §267(d)"));
    }

    #[test]
    fn all_ten_267b_categories_treated_as_related() {
        for rel in [
            RelationshipCategory::FamilyMember,
            RelationshipCategory::IndividualAndControlledCorp,
            RelationshipCategory::TwoControlledCorps,
            RelationshipCategory::GrantorAndTrustFiduciary,
            RelationshipCategory::TwoTrustFiduciariesSameGrantor,
            RelationshipCategory::TrustFiduciaryAndBeneficiary,
            RelationshipCategory::TrustFiduciaryAndOtherBeneficiary,
            RelationshipCategory::CorpAndPartnershipCommonOwner,
            RelationshipCategory::TwoSCorps,
            RelationshipCategory::EstateExecutorAndBeneficiary,
        ] {
            let mut i = base();
            i.relationship = rel;
            let r = compute(&i);
            assert!(r.is_related_party);
            assert_eq!(r.loss_disallowed, dec!(10000), "{:?}", rel);
            assert_eq!(r.loss_recognized_by_seller, Decimal::ZERO, "{:?}", rel);
        }
    }

    #[test]
    fn buyer_initial_basis_is_cash_price_not_seller_basis() {
        // Buyer paid $40k. That's their basis — §267 does NOT transfer
        // the seller's original basis to the buyer.
        let r = compute(&base());
        assert_eq!(r.buyer_initial_basis, dec!(40000));
    }

    #[test]
    fn section_267d_zero_gain_leaves_loss_lost() {
        let mut i = base();
        i.buyer_subsequent_gain = Some(Decimal::ZERO);
        let r = compute(&i);
        assert_eq!(r.buyer_section_267d_reduction, Decimal::ZERO);
        assert_eq!(r.loss_permanently_lost_if_buyer_no_gain, dec!(10000));
    }

    #[test]
    fn unrelated_with_buyer_subsequent_gain_ignores_267d() {
        // §267(d) only matters when (a) was triggered. Unrelated parties
        // don't have any §267 disallowance to apply.
        let mut i = base();
        i.relationship = RelationshipCategory::Unrelated;
        i.buyer_subsequent_gain = Some(dec!(50000));
        let r = compute(&i);
        assert_eq!(r.loss_disallowed, Decimal::ZERO);
        assert_eq!(r.buyer_section_267d_reduction, Decimal::ZERO);
        assert_eq!(r.loss_recognized_by_seller, dec!(10000));
    }

    #[test]
    fn corp_and_partnership_common_owner_is_related() {
        // §267(b)(8) — trader LLC selling at a loss to their own S-corp.
        let mut i = base();
        i.relationship = RelationshipCategory::CorpAndPartnershipCommonOwner;
        let r = compute(&i);
        assert!(r.is_related_party);
        assert_eq!(r.loss_disallowed, dec!(10000));
    }

    #[test]
    fn is_related_helper_returns_false_only_for_unrelated() {
        assert!(!RelationshipCategory::Unrelated.is_related());
        assert!(RelationshipCategory::FamilyMember.is_related());
        assert!(RelationshipCategory::TwoSCorps.is_related());
    }

    #[test]
    fn note_for_partial_267d_reduction_describes_split() {
        let mut i = base();
        i.buyer_subsequent_gain = Some(dec!(3000));
        let r = compute(&i);
        assert!(r.note.contains("$3000"));
        assert!(r.note.contains("$7000"));
        assert!(r.note.contains("permanently lost"));
    }
}
