//! IRC §351 — Transfer to corporation controlled by transferor.
//!
//! Foundational corporate-formation non-recognition section. When
//! one or more persons transfer property to a corporation in
//! exchange for stock AND the transferors collectively own at least
//! 80% of the corporation's voting power + 80% of each non-voting
//! class IMMEDIATELY AFTER the exchange, no gain or loss is
//! recognized on the transfer. Stable since 1954 reformulation; no
//! material TCJA or OBBBA amendments.
//!
//! **Control test** (§368(c) via §351(a)):
//!
//! - At least 80% of the total combined voting power of all voting
//!   stock, AND
//! - At least 80% of the total number of shares of EACH non-voting
//!   class of stock outstanding.
//!
//! **Property requirement** (§351(d)): "stock issued for services"
//! is NOT treated as issued in return for property. A pure service
//! contributor does not count toward the 80% control test (though
//! their receipt of stock dilutes the group).
//!
//! **Boot rule** (§351(b)):
//!
//! - §351(b)(1): if the transferor receives money or other property
//!   in addition to stock, gain (if any) is recognized to the
//!   extent of the boot received. Recognized gain ≤ realized gain.
//! - §351(b)(2): no loss is recognized on a §351 transfer, even
//!   when boot is received. The loss is preserved in the
//!   substituted basis of the stock.
//!
//! **Liabilities assumed** (§357):
//!
//! - §357(a) general rule: liabilities assumed by the corporation
//!   are NOT treated as money or other property for §351(b) boot
//!   purposes.
//! - §357(b): if the principal purpose of any liability assumption
//!   was tax avoidance or a non-bona-fide business reason, ALL
//!   liabilities assumed in the transaction are treated as boot.
//! - §357(c): to the extent the sum of liabilities assumed exceeds
//!   the total adjusted basis of property transferred, the excess
//!   is recognized as gain (independent of any actual boot).
//!
//! **Basis rules**:
//!
//! - §358(a): transferor's basis in stock received = adjusted basis
//!   of property transferred − money received − FMV of other
//!   property received − liabilities assumed by corp (§358(d)) +
//!   gain recognized.
//! - §362(a): corporation's basis in property received = transferor's
//!   adjusted basis + gain recognized by transferor.
//!
//! Sources:
//! [Cornell LII 26 U.S.C. § 351](https://www.law.cornell.edu/uscode/text/26/351),
//! [Cornell LII 26 U.S.C. § 357](https://www.law.cornell.edu/uscode/text/26/357),
//! [Cornell LII 26 U.S.C. § 358](https://www.law.cornell.edu/uscode/text/26/358),
//! [Cornell LII 26 U.S.C. § 362](https://www.law.cornell.edu/uscode/text/26/362),
//! [Cornell LII 26 U.S.C. § 368(c) (control test)](https://www.law.cornell.edu/uscode/text/26/368).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NonRecognitionResult {
    /// §351 applies — non-recognition (or limited boot recognition).
    Applies,
    /// Control test failed (transferors collectively < 80% voting or
    /// < 80% of a non-voting class immediately after exchange).
    ControlTestFailed,
    /// Pure service contributor receiving stock — disqualified
    /// individually (§351(d)). Other transferors may still qualify;
    /// this result indicates this specific transferor's path.
    PureServicesExcluded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section351Input {
    /// Transferor's adjusted basis in property contributed.
    pub property_adjusted_basis_dollars: i64,
    /// FMV of property contributed.
    pub property_fmv_dollars: i64,
    /// FMV of stock received in exchange.
    pub stock_fmv_received_dollars: i64,
    /// Money + FMV of any non-stock property received from corp
    /// (other than liabilities assumed).
    pub boot_received_dollars: i64,
    /// Liabilities the corporation assumes from the transferor.
    pub liabilities_assumed_by_corp_dollars: i64,
    /// Control group's percentage of total combined voting power
    /// IMMEDIATELY AFTER the exchange, in basis points (8000 = 80%).
    pub control_group_voting_pct_bp: u32,
    /// Control group's lowest percentage of any non-voting class
    /// outstanding immediately after exchange, in basis points.
    /// If no non-voting class exists, set to 10000 (100%).
    pub control_group_nonvoting_pct_bp: u32,
    /// True if this transferor contributed ONLY services (no property)
    /// and is receiving stock. §351(d) disqualifies them.
    pub transferor_is_pure_services: bool,
    /// True if the principal purpose of any liability assumption was
    /// tax avoidance / non-bona-fide business reason — triggers
    /// §357(b) full-boot treatment of all assumed liabilities.
    pub tax_avoidance_principal_purpose: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section351Result {
    pub outcome: NonRecognitionResult,
    pub meets_control_test: bool,
    pub realized_gain_dollars: i64,
    pub realized_loss_dollars: i64,
    pub recognized_gain_dollars: i64,
    /// Loss is NEVER recognized under §351 (preserved in basis).
    pub recognized_loss_dollars: i64,
    /// §357(c) excess-liability gain (sum of liabilities > basis).
    pub section_357c_gain_dollars: i64,
    /// True if §357(b) triggered full-boot treatment of liabilities.
    pub liabilities_treated_as_boot: bool,
    /// Transferor's basis in stock received (§358).
    pub transferor_stock_basis_dollars: i64,
    /// Corporation's basis in property received (§362(a)).
    pub corp_property_basis_dollars: i64,
    pub citation: String,
    pub note: String,
}

const CONTROL_TEST_THRESHOLD_BP: u32 = 8000; // 80%

pub fn compute(input: &Section351Input) -> Section351Result {
    // Control test (§368(c) via §351(a)).
    let meets_control = input.control_group_voting_pct_bp >= CONTROL_TEST_THRESHOLD_BP
        && input.control_group_nonvoting_pct_bp >= CONTROL_TEST_THRESHOLD_BP;

    // §351(d) services exclusion.
    let outcome = if input.transferor_is_pure_services {
        NonRecognitionResult::PureServicesExcluded
    } else if !meets_control {
        NonRecognitionResult::ControlTestFailed
    } else {
        NonRecognitionResult::Applies
    };

    // Realized gain/loss: consideration received minus basis.
    // Consideration = stock + boot + liabilities assumed.
    let consideration = input.stock_fmv_received_dollars
        + input.boot_received_dollars
        + input.liabilities_assumed_by_corp_dollars;
    let realized = consideration - input.property_adjusted_basis_dollars;
    let realized_gain = realized.max(0);
    let realized_loss = (-realized).max(0);

    // §357(c) excess-liability gain — applies regardless of boot.
    let section_357c_gain =
        (input.liabilities_assumed_by_corp_dollars - input.property_adjusted_basis_dollars).max(0);

    // §357(b) full-liability-as-boot trigger.
    let liabilities_as_boot = input.tax_avoidance_principal_purpose;

    // Recognized gain depends on outcome.
    let (recognized_gain, recognized_loss) = match outcome {
        NonRecognitionResult::Applies => {
            let effective_boot = if liabilities_as_boot {
                input.boot_received_dollars + input.liabilities_assumed_by_corp_dollars
            } else {
                input.boot_received_dollars
            };
            // §351(b)(1): gain recognized to extent of boot, capped
            // by realized gain. Then add §357(c) excess (when not
            // §357(b)) which is independent.
            let boot_gain = realized_gain.min(effective_boot);
            let total_recognized = if liabilities_as_boot {
                // §357(b) treats liabilities as boot, so §357(c) is
                // subsumed; no double-counting.
                boot_gain
            } else {
                boot_gain + section_357c_gain
            };
            // §351(b)(2): no loss recognized under §351 — preserved
            // in basis.
            (
                total_recognized.min(realized_gain.max(0) + section_357c_gain),
                0,
            )
        }
        NonRecognitionResult::ControlTestFailed | NonRecognitionResult::PureServicesExcluded => {
            // Outside §351 → full recognition under general §1001
            // realization rules.
            (realized_gain, realized_loss)
        }
    };

    // §358(a) transferor basis in stock.
    // basis = adj_basis − boot − liabilities + gain
    let stock_basis = input.property_adjusted_basis_dollars
        - input.boot_received_dollars
        - input.liabilities_assumed_by_corp_dollars
        + recognized_gain;

    // §362(a) corporation basis in property.
    // basis = transferor's basis + gain recognized by transferor
    let corp_basis = input.property_adjusted_basis_dollars + recognized_gain;

    let outcome_label = match outcome {
        NonRecognitionResult::Applies => "§351 non-recognition applies",
        NonRecognitionResult::ControlTestFailed => {
            "§351 does NOT apply — §368(c) 80% control test failed"
        }
        NonRecognitionResult::PureServicesExcluded => {
            "§351 does NOT apply for this transferor — §351(d) services exclusion"
        }
    };

    let note = format!(
        "Property basis ${} + FMV ${}; consideration stock ${} + boot ${} + liabilities ${}; realized gain ${} (loss ${}); control voting {}.{}% / nonvoting {}.{}% (80% required){}; recognized gain ${} (loss ${} — §351 never recognizes losses){}{}; transferor stock basis ${} (§358); corp property basis ${} (§362). {}.",
        input.property_adjusted_basis_dollars,
        input.property_fmv_dollars,
        input.stock_fmv_received_dollars,
        input.boot_received_dollars,
        input.liabilities_assumed_by_corp_dollars,
        realized_gain,
        realized_loss,
        input.control_group_voting_pct_bp / 100,
        input.control_group_voting_pct_bp % 100,
        input.control_group_nonvoting_pct_bp / 100,
        input.control_group_nonvoting_pct_bp % 100,
        if input.transferor_is_pure_services { "; services contributor §351(d)" } else { "" },
        recognized_gain,
        recognized_loss,
        if liabilities_as_boot {
            " — §357(b) tax-avoidance triggered: ALL liabilities treated as boot"
        } else { "" },
        if section_357c_gain > 0 && !liabilities_as_boot {
            format!(" — §357(c) excess-liability gain ${}", section_357c_gain)
        } else { String::new() },
        stock_basis,
        corp_basis,
        outcome_label,
    );

    Section351Result {
        outcome,
        meets_control_test: meets_control,
        realized_gain_dollars: realized_gain,
        realized_loss_dollars: realized_loss,
        recognized_gain_dollars: recognized_gain,
        recognized_loss_dollars: recognized_loss,
        section_357c_gain_dollars: section_357c_gain,
        liabilities_treated_as_boot: liabilities_as_boot,
        transferor_stock_basis_dollars: stock_basis,
        corp_property_basis_dollars: corp_basis,
        citation:
            "IRC §351(a) non-recognition on transfer to controlled corporation; §351(b) boot recognition rule (gain ≤ boot; no loss recognition); §351(d) services exclusion; §368(c) 80%/80% control test; §357(a) liabilities not treated as boot; §357(b) tax-avoidance full-boot treatment; §357(c) excess-liability-over-basis gain; §358(a) substituted stock basis (basis − boot − liabilities + gain); §362(a) carryover corporation basis + gain"
                .to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Section351Input {
        Section351Input {
            property_adjusted_basis_dollars: 100_000,
            property_fmv_dollars: 200_000,
            stock_fmv_received_dollars: 200_000,
            boot_received_dollars: 0,
            liabilities_assumed_by_corp_dollars: 0,
            control_group_voting_pct_bp: 10_000,
            control_group_nonvoting_pct_bp: 10_000,
            transferor_is_pure_services: false,
            tax_avoidance_principal_purpose: false,
        }
    }

    // ── §351(a) baseline non-recognition ───────────────────────────

    #[test]
    fn baseline_full_non_recognition() {
        // 100% control, no boot, $100k basis → $200k stock.
        // Realized $100k gain; recognized = 0; loss = 0.
        let r = compute(&base());
        assert_eq!(r.outcome, NonRecognitionResult::Applies);
        assert_eq!(r.realized_gain_dollars, 100_000);
        assert_eq!(r.recognized_gain_dollars, 0);
        assert_eq!(r.recognized_loss_dollars, 0);
        assert_eq!(
            r.transferor_stock_basis_dollars, 100_000,
            "substituted basis preserves deferred gain"
        );
        assert_eq!(r.corp_property_basis_dollars, 100_000, "carryover basis");
    }

    // ── §368(c) 80% control test ────────────────────────────────────

    #[test]
    fn control_at_80_exact_meets_test() {
        let mut i = base();
        i.control_group_voting_pct_bp = 8000;
        i.control_group_nonvoting_pct_bp = 8000;
        let r = compute(&i);
        assert!(r.meets_control_test);
        assert_eq!(r.outcome, NonRecognitionResult::Applies);
    }

    #[test]
    fn control_at_79_99_fails_test() {
        let mut i = base();
        i.control_group_voting_pct_bp = 7999;
        let r = compute(&i);
        assert!(!r.meets_control_test);
        assert_eq!(r.outcome, NonRecognitionResult::ControlTestFailed);
        // Outside §351 → full recognition.
        assert_eq!(r.recognized_gain_dollars, 100_000);
    }

    #[test]
    fn control_voting_pass_nonvoting_fail() {
        // Both prongs are independently required.
        let mut i = base();
        i.control_group_voting_pct_bp = 10_000;
        i.control_group_nonvoting_pct_bp = 5000;
        let r = compute(&i);
        assert!(!r.meets_control_test);
        assert_eq!(r.outcome, NonRecognitionResult::ControlTestFailed);
    }

    // ── §351(b)(1) boot recognition (gain) ─────────────────────────

    #[test]
    fn boot_recognition_capped_at_realized_gain() {
        // Property basis $100k, stock $180k + boot $20k = $200k.
        // Realized $100k; boot $20k → recognized gain = $20k.
        let mut i = base();
        i.stock_fmv_received_dollars = 180_000;
        i.boot_received_dollars = 20_000;
        let r = compute(&i);
        assert_eq!(r.realized_gain_dollars, 100_000);
        assert_eq!(r.recognized_gain_dollars, 20_000);
    }

    #[test]
    fn boot_capped_by_realized_gain_not_exceed() {
        // Basis $500k, consideration $600k ($400k stock + $200k boot).
        // Realized = $100k; boot = $200k → recognized = $100k
        // (boot CAPPED by realized gain).
        let mut i = base();
        i.property_adjusted_basis_dollars = 500_000;
        i.stock_fmv_received_dollars = 400_000;
        i.boot_received_dollars = 200_000;
        let r = compute(&i);
        assert_eq!(r.realized_gain_dollars, 100_000);
        assert_eq!(
            r.recognized_gain_dollars, 100_000,
            "boot recognition can't exceed realized gain"
        );
    }

    // ── §351(b)(2) loss never recognized ───────────────────────────

    #[test]
    fn loss_never_recognized_even_with_boot() {
        // Basis $200k, stock $100k + boot $50k = $150k consideration.
        // Realized loss = $50k; recognized loss = 0 under §351.
        let mut i = base();
        i.property_adjusted_basis_dollars = 200_000;
        i.stock_fmv_received_dollars = 100_000;
        i.boot_received_dollars = 50_000;
        let r = compute(&i);
        assert_eq!(r.realized_loss_dollars, 50_000);
        assert_eq!(
            r.recognized_loss_dollars, 0,
            "§351(b)(2) — losses never recognized"
        );
    }

    #[test]
    fn loss_preserved_in_substituted_basis() {
        // Same setup — stock basis preserves the unrecognized loss.
        let mut i = base();
        i.property_adjusted_basis_dollars = 200_000;
        i.stock_fmv_received_dollars = 100_000;
        i.boot_received_dollars = 50_000;
        let r = compute(&i);
        // basis = 200k − 50k boot − 0 liab + 0 gain = 150k.
        assert_eq!(r.transferor_stock_basis_dollars, 150_000);
    }

    // ── §357(a) liabilities not treated as boot ────────────────────

    #[test]
    fn liabilities_assumed_within_basis_not_treated_as_boot() {
        // Basis $200k, $50k liabilities assumed (< basis).
        // §357(a) → not boot; §357(c) → no excess; non-recognition.
        let mut i = base();
        i.property_adjusted_basis_dollars = 200_000;
        i.property_fmv_dollars = 300_000;
        i.stock_fmv_received_dollars = 250_000;
        i.boot_received_dollars = 0;
        i.liabilities_assumed_by_corp_dollars = 50_000;
        let r = compute(&i);
        assert_eq!(r.outcome, NonRecognitionResult::Applies);
        assert_eq!(r.section_357c_gain_dollars, 0);
        assert_eq!(r.recognized_gain_dollars, 0);
    }

    // ── §357(b) tax-avoidance full-boot treatment ──────────────────

    #[test]
    fn section_357b_tax_avoidance_treats_liabilities_as_boot() {
        let mut i = base();
        i.property_adjusted_basis_dollars = 200_000;
        i.property_fmv_dollars = 400_000;
        i.stock_fmv_received_dollars = 300_000;
        i.boot_received_dollars = 0;
        i.liabilities_assumed_by_corp_dollars = 100_000;
        i.tax_avoidance_principal_purpose = true;
        let r = compute(&i);
        assert!(r.liabilities_treated_as_boot);
        // Realized = 400k − 200k = 200k; effective boot = 0 + 100k = 100k.
        // recognized = min(200k, 100k) = 100k.
        assert_eq!(r.realized_gain_dollars, 200_000);
        assert_eq!(r.recognized_gain_dollars, 100_000);
    }

    // ── §357(c) excess-liabilities-over-basis gain ─────────────────

    #[test]
    fn section_357c_excess_liability_triggers_gain() {
        // Basis $100k, liabilities $150k assumed → §357(c) gain $50k.
        let mut i = base();
        i.property_adjusted_basis_dollars = 100_000;
        i.property_fmv_dollars = 300_000;
        i.stock_fmv_received_dollars = 150_000;
        i.boot_received_dollars = 0;
        i.liabilities_assumed_by_corp_dollars = 150_000;
        let r = compute(&i);
        assert_eq!(r.section_357c_gain_dollars, 50_000);
        assert_eq!(r.recognized_gain_dollars, 50_000);
    }

    #[test]
    fn section_357c_combined_with_actual_boot() {
        // Basis $100k; boot $30k; liabilities $150k → §357(c) $50k;
        // boot gain $30k; total recognized = $80k.
        let mut i = base();
        i.property_adjusted_basis_dollars = 100_000;
        i.property_fmv_dollars = 400_000;
        i.stock_fmv_received_dollars = 220_000;
        i.boot_received_dollars = 30_000;
        i.liabilities_assumed_by_corp_dollars = 150_000;
        let r = compute(&i);
        assert_eq!(r.section_357c_gain_dollars, 50_000);
        // realized = 220k + 30k + 150k − 100k = 300k. Boot $30k ≤ realized.
        assert_eq!(r.recognized_gain_dollars, 30_000 + 50_000);
    }

    #[test]
    fn section_357c_does_not_double_count_with_357b() {
        // When §357(b) treats ALL liabilities as boot, §357(c) is
        // subsumed — no double-recognition.
        let mut i = base();
        i.property_adjusted_basis_dollars = 100_000;
        i.property_fmv_dollars = 400_000;
        i.stock_fmv_received_dollars = 250_000;
        i.boot_received_dollars = 0;
        i.liabilities_assumed_by_corp_dollars = 150_000;
        i.tax_avoidance_principal_purpose = true;
        let r = compute(&i);
        // realized = 250k + 0 + 150k − 100k = 300k.
        // effective boot under §357(b) = 0 + 150k = 150k.
        // recognized = min(300k, 150k) = 150k.
        // §357(c) NOT additionally added → total = 150k.
        assert_eq!(r.recognized_gain_dollars, 150_000);
    }

    // ── §351(d) services exclusion ──────────────────────────────────

    #[test]
    fn pure_services_contributor_excluded() {
        let mut i = base();
        i.transferor_is_pure_services = true;
        let r = compute(&i);
        assert_eq!(r.outcome, NonRecognitionResult::PureServicesExcluded);
    }

    // ── §358 substituted basis math ─────────────────────────────────

    #[test]
    fn section_358_basis_no_boot_no_liab() {
        // basis = 100k − 0 − 0 + 0 = 100k.
        let r = compute(&base());
        assert_eq!(r.transferor_stock_basis_dollars, 100_000);
    }

    #[test]
    fn section_358_basis_with_boot_reduces() {
        let mut i = base();
        i.stock_fmv_received_dollars = 180_000;
        i.boot_received_dollars = 20_000;
        let r = compute(&i);
        // basis = 100k − 20k + 20k gain = 100k.
        assert_eq!(r.transferor_stock_basis_dollars, 100_000);
    }

    #[test]
    fn section_358_basis_with_liab_reduces() {
        let mut i = base();
        i.property_adjusted_basis_dollars = 200_000;
        i.property_fmv_dollars = 300_000;
        i.stock_fmv_received_dollars = 250_000;
        i.liabilities_assumed_by_corp_dollars = 50_000;
        let r = compute(&i);
        // basis = 200k − 0 − 50k + 0 gain = 150k.
        assert_eq!(r.transferor_stock_basis_dollars, 150_000);
    }

    // ── §362(a) corp carryover basis ────────────────────────────────

    #[test]
    fn section_362_corp_basis_carryover_plus_gain() {
        // No gain → corp basis = transferor's basis.
        let r = compute(&base());
        assert_eq!(r.corp_property_basis_dollars, 100_000);
    }

    #[test]
    fn section_362_corp_basis_includes_boot_gain() {
        let mut i = base();
        i.stock_fmv_received_dollars = 180_000;
        i.boot_received_dollars = 20_000;
        let r = compute(&i);
        // corp basis = 100k + 20k gain = 120k.
        assert_eq!(r.corp_property_basis_dollars, 120_000);
    }

    // ── Citation ────────────────────────────────────────────────────

    #[test]
    fn citation_mentions_all_relevant_authorities() {
        let r = compute(&base());
        assert!(r.citation.contains("§351(a)"));
        assert!(r.citation.contains("§351(b)"));
        assert!(r.citation.contains("§351(d)"));
        assert!(r.citation.contains("§368(c)"));
        assert!(r.citation.contains("§357(a)"));
        assert!(r.citation.contains("§357(b)"));
        assert!(r.citation.contains("§357(c)"));
        assert!(r.citation.contains("§358(a)"));
        assert!(r.citation.contains("§362(a)"));
    }

    // ── Notes ───────────────────────────────────────────────────────

    #[test]
    fn note_applies_describes_outcome() {
        let r = compute(&base());
        assert!(r.note.contains("§351 non-recognition applies"));
    }

    #[test]
    fn note_control_failure_describes_outcome() {
        let mut i = base();
        i.control_group_voting_pct_bp = 7000;
        let r = compute(&i);
        assert!(r.note.contains("80% control test failed"));
    }

    #[test]
    fn note_services_exclusion_describes_outcome() {
        let mut i = base();
        i.transferor_is_pure_services = true;
        let r = compute(&i);
        assert!(r.note.contains("services exclusion"));
    }

    #[test]
    fn note_357b_says_tax_avoidance_triggered() {
        let mut i = base();
        i.liabilities_assumed_by_corp_dollars = 50_000;
        i.tax_avoidance_principal_purpose = true;
        let r = compute(&i);
        assert!(r.note.contains("§357(b)"));
    }

    #[test]
    fn note_357c_says_excess_liability() {
        let mut i = base();
        i.liabilities_assumed_by_corp_dollars = 150_000;
        let r = compute(&i);
        assert!(r.note.contains("§357(c)"));
    }

    // ── Defensive / precision ──────────────────────────────────────

    #[test]
    fn very_large_billion_dollar_contribution_no_precision_loss() {
        // $500M basis, $1B FMV, $700M stock + $300M boot.
        let mut i = base();
        i.property_adjusted_basis_dollars = 500_000_000;
        i.property_fmv_dollars = 1_000_000_000;
        i.stock_fmv_received_dollars = 700_000_000;
        i.boot_received_dollars = 300_000_000;
        let r = compute(&i);
        assert_eq!(r.realized_gain_dollars, 500_000_000);
        assert_eq!(r.recognized_gain_dollars, 300_000_000);
        assert_eq!(
            r.transferor_stock_basis_dollars, 500_000_000,
            "500M − 300M boot + 300M gain = 500M"
        );
        assert_eq!(r.corp_property_basis_dollars, 800_000_000);
    }

    #[test]
    fn zero_basis_zero_consideration_no_op() {
        let mut i = base();
        i.property_adjusted_basis_dollars = 0;
        i.property_fmv_dollars = 0;
        i.stock_fmv_received_dollars = 0;
        let r = compute(&i);
        assert_eq!(r.realized_gain_dollars, 0);
        assert_eq!(r.recognized_gain_dollars, 0);
        assert_eq!(r.transferor_stock_basis_dollars, 0);
    }
}
