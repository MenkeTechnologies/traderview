//! IRC §336 — Gain or loss recognized on property distributed in
//! complete liquidation.
//!
//! Corporate-level rule for any C-corp in complete liquidation: the
//! liquidating corporation recognizes gain or loss as if the
//! distributed property had been sold to the distributee at FMV.
//! §336 is the corporate-side counterpart to §331 (shareholder-side
//! treatment) and §332 (parent-subsidiary tax-free exception).
//!
//! **§ 336(a) general rule**: gain OR loss recognized to liquidating
//! corp on distribution of property in complete liquidation as if
//! property were sold to the distributee at FMV ([Cornell LII 26
//! U.S.C. § 336](https://www.law.cornell.edu/uscode/text/26/336)).
//!
//! **§ 336(b) liability treatment**: if property distributed is
//! subject to a liability OR the shareholder assumes a liability of
//! the liquidating corp, FMV is treated as NOT LESS than the
//! liability amount. Prevents debt-laden property from generating
//! artificial loss.
//!
//! **§ 336(d)(1) related-party loss disallowance**: NO LOSS recognized
//! on distribution to a related party (> 50% ownership) UNLESS
//! distribution is PRO RATA to all shareholders AND property is not
//! "disqualified property".
//!
//! **§ 336(d)(2) anti-tax-avoidance / 5-year rule**: loss disallowed
//! to the extent attributable to built-in loss EXISTING AT THE TIME
//! the property was acquired by the liquidating corp via § 351
//! transfer or capital contribution within 5 years of the liquidation.
//! Prevents loss-trafficking via "stuffing" loss property into a
//! corp pre-liquidation.
//!
//! **§ 336(d)(3) § 332 subsidiary exception**: NO gain or loss
//! recognized to liquidating corp in a § 332 parent-subsidiary
//! liquidation (parent owns 80%+ of subsidiary). Parent takes
//! carryover basis in subsidiary's assets under § 334(b).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section336Input {
    pub distributed_property_fmv_dollars: i64,
    pub distributed_property_adjusted_basis_dollars: i64,
    /// True if the distributee owns more than 50% (by vote or value)
    /// of the liquidating corporation's stock.
    pub distributee_is_related_party_more_than_50_pct: bool,
    pub distribution_is_pro_rata_to_all_shareholders: bool,
    /// True if the property was acquired by the liquidating corp via
    /// § 351 transfer OR capital contribution within the 5-year
    /// period ending on the date of distribution.
    pub property_acquired_via_351_or_capital_contribution_within_5_years: bool,
    /// Built-in loss EXISTING AT THE TIME of contribution (= basis
    /// at contribution − FMV at contribution).
    pub built_in_loss_at_contribution_dollars: i64,
    /// True if the liquidation qualifies under § 332 (parent owns
    /// 80%+ voting stock + 80%+ value of subsidiary). Triggers full
    /// gain/loss non-recognition under § 336(d)(3).
    pub section_332_subsidiary_liquidation: bool,
    /// True if property distributed is subject to a liability OR the
    /// shareholder assumes a liability of the liquidating corp.
    pub liability_amount_on_property_dollars: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section336Result {
    pub effective_fmv_dollars: i64,
    pub realized_gain_or_loss_dollars: i64,
    pub recognized_gain_dollars: i64,
    pub recognized_loss_dollars: i64,
    pub section_332_full_nonrecognition: bool,
    pub section_336d1_related_party_loss_disallowed: bool,
    pub section_336d2_built_in_loss_disallowed_amount_dollars: i64,
    pub citation: String,
    pub note: String,
}

pub fn compute(input: &Section336Input) -> Section336Result {
    // § 332 short-circuit: no gain or loss whatsoever.
    if input.section_332_subsidiary_liquidation {
        return Section336Result {
            effective_fmv_dollars: input.distributed_property_fmv_dollars,
            realized_gain_or_loss_dollars: 0,
            recognized_gain_dollars: 0,
            recognized_loss_dollars: 0,
            section_332_full_nonrecognition: true,
            section_336d1_related_party_loss_disallowed: false,
            section_336d2_built_in_loss_disallowed_amount_dollars: 0,
            citation:
                "IRC §332(a) parent-subsidiary tax-free liquidation; §336(d)(3) no gain or loss to liquidating subsidiary; §334(b) parent takes carryover basis"
                    .to_string(),
            note: "§ 336(d)(3) §332 subsidiary exception: parent-subsidiary 80%+ liquidation — NO gain or loss recognized to liquidating subsidiary. Parent takes carryover basis under § 334(b).".to_string(),
        };
    }

    // § 336(b) liability treatment: FMV = max(actual FMV, liability).
    let effective_fmv = input
        .distributed_property_fmv_dollars
        .max(input.liability_amount_on_property_dollars);

    let realized = effective_fmv - input.distributed_property_adjusted_basis_dollars;

    // Gain side.
    let recognized_gain = realized.max(0);

    // Loss side — apply both § 336(d)(1) and § 336(d)(2) loss
    // disallowances.
    let realized_loss = (-realized).max(0);

    let d1_disallowed = realized_loss > 0
        && input.distributee_is_related_party_more_than_50_pct
        && (!input.distribution_is_pro_rata_to_all_shareholders
            || input.property_acquired_via_351_or_capital_contribution_within_5_years);

    let d2_disallowed_amount = if realized_loss > 0
        && input.property_acquired_via_351_or_capital_contribution_within_5_years
    {
        // Disallow the built-in loss portion (= original contribution
        // BIL up to current realized loss).
        input
            .built_in_loss_at_contribution_dollars
            .min(realized_loss)
            .max(0)
    } else {
        0
    };

    let recognized_loss = if d1_disallowed {
        0
    } else {
        (realized_loss - d2_disallowed_amount).max(0)
    };

    let mut note_parts: Vec<String> = Vec::new();
    note_parts.push(format!(
        "§ 336(a) general rule: FMV ${} − basis ${} = ${} realized {}.",
        effective_fmv,
        input.distributed_property_adjusted_basis_dollars,
        realized.abs(),
        if realized >= 0 { "gain" } else { "loss" },
    ));
    if input.liability_amount_on_property_dollars > input.distributed_property_fmv_dollars {
        note_parts.push(format!(
            "§ 336(b) liability rule: FMV adjusted upward to liability amount ${} (was ${}).",
            input.liability_amount_on_property_dollars, input.distributed_property_fmv_dollars,
        ));
    }
    if d1_disallowed {
        note_parts.push(format!(
            "§ 336(d)(1) related-party loss DISALLOWED: distributee owns > 50% AND ({}). Full ${} loss disallowed.",
            if !input.distribution_is_pro_rata_to_all_shareholders {
                "non-pro-rata distribution"
            } else {
                "disqualified property"
            },
            realized_loss,
        ));
    }
    if d2_disallowed_amount > 0 {
        note_parts.push(format!(
            "§ 336(d)(2) anti-tax-avoidance: ${} built-in loss at contribution within 5-year window DISALLOWED.",
            d2_disallowed_amount,
        ));
    }
    note_parts.push(format!(
        "Recognized: ${} gain / ${} loss to liquidating corporation.",
        recognized_gain, recognized_loss,
    ));

    Section336Result {
        effective_fmv_dollars: effective_fmv,
        realized_gain_or_loss_dollars: realized,
        recognized_gain_dollars: recognized_gain,
        recognized_loss_dollars: recognized_loss,
        section_332_full_nonrecognition: false,
        section_336d1_related_party_loss_disallowed: d1_disallowed,
        section_336d2_built_in_loss_disallowed_amount_dollars: d2_disallowed_amount,
        citation:
            "IRC §336(a) gain/loss as if FMV sale; §336(b) liability ≥ FMV rule; §336(d)(1) related-party loss disallowance (> 50% ownership); §336(d)(2) 5-year anti-tax-avoidance built-in loss disallowance; §336(d)(3) §332 subsidiary exception; companion §331 (shareholder side) + §334(b) parent carryover basis"
                .to_string(),
        note: note_parts.join(" "),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Section336Input {
        Section336Input {
            distributed_property_fmv_dollars: 1_000_000,
            distributed_property_adjusted_basis_dollars: 600_000,
            distributee_is_related_party_more_than_50_pct: false,
            distribution_is_pro_rata_to_all_shareholders: true,
            property_acquired_via_351_or_capital_contribution_within_5_years: false,
            built_in_loss_at_contribution_dollars: 0,
            section_332_subsidiary_liquidation: false,
            liability_amount_on_property_dollars: 0,
        }
    }

    // §336(a) general rule.

    #[test]
    fn standard_gain_recognized() {
        // FMV $1M − basis $600k = $400k gain.
        let r = compute(&base());
        assert_eq!(r.realized_gain_or_loss_dollars, 400_000);
        assert_eq!(r.recognized_gain_dollars, 400_000);
        assert_eq!(r.recognized_loss_dollars, 0);
    }

    #[test]
    fn standard_loss_recognized() {
        let mut i = base();
        i.distributed_property_fmv_dollars = 400_000;
        let r = compute(&i);
        assert_eq!(r.realized_gain_or_loss_dollars, -200_000);
        assert_eq!(r.recognized_loss_dollars, 200_000);
    }

    // §336(b) liability rule.

    #[test]
    fn liability_higher_than_fmv_boosts_fmv() {
        // FMV $400k, basis $600k → loss $200k.
        // But liability $700k > FMV → FMV adjusted to $700k → gain $100k.
        let mut i = base();
        i.distributed_property_fmv_dollars = 400_000;
        i.liability_amount_on_property_dollars = 700_000;
        let r = compute(&i);
        assert_eq!(r.effective_fmv_dollars, 700_000);
        assert_eq!(r.recognized_gain_dollars, 100_000);
        assert!(r.note.contains("§ 336(b) liability rule"));
    }

    #[test]
    fn liability_below_fmv_no_adjustment() {
        let mut i = base();
        i.liability_amount_on_property_dollars = 500_000;
        let r = compute(&i);
        assert_eq!(r.effective_fmv_dollars, 1_000_000);
    }

    // §336(d)(1) related-party loss disallowance.

    #[test]
    fn related_party_non_pro_rata_distribution_loss_disallowed() {
        let mut i = base();
        i.distributed_property_fmv_dollars = 400_000; // loss
        i.distributee_is_related_party_more_than_50_pct = true;
        i.distribution_is_pro_rata_to_all_shareholders = false;
        let r = compute(&i);
        assert!(r.section_336d1_related_party_loss_disallowed);
        assert_eq!(r.recognized_loss_dollars, 0);
    }

    #[test]
    fn related_party_pro_rata_no_d1_disallowance_if_not_disqualified() {
        let mut i = base();
        i.distributed_property_fmv_dollars = 400_000;
        i.distributee_is_related_party_more_than_50_pct = true;
        i.distribution_is_pro_rata_to_all_shareholders = true;
        // Not disqualified property
        let r = compute(&i);
        assert!(!r.section_336d1_related_party_loss_disallowed);
        assert_eq!(r.recognized_loss_dollars, 200_000);
    }

    #[test]
    fn related_party_pro_rata_with_disqualified_property_d1_disallows() {
        let mut i = base();
        i.distributed_property_fmv_dollars = 400_000;
        i.distributee_is_related_party_more_than_50_pct = true;
        i.distribution_is_pro_rata_to_all_shareholders = true;
        i.property_acquired_via_351_or_capital_contribution_within_5_years = true;
        let r = compute(&i);
        assert!(r.section_336d1_related_party_loss_disallowed);
    }

    #[test]
    fn unrelated_party_no_d1_disallowance() {
        let mut i = base();
        i.distributed_property_fmv_dollars = 400_000;
        i.distributee_is_related_party_more_than_50_pct = false;
        i.distribution_is_pro_rata_to_all_shareholders = false;
        let r = compute(&i);
        assert!(!r.section_336d1_related_party_loss_disallowed);
        assert_eq!(r.recognized_loss_dollars, 200_000);
    }

    // §336(d)(2) anti-tax-avoidance / 5-year rule.

    #[test]
    fn d2_built_in_loss_disallowed_at_contribution() {
        // $200k current loss; $150k was built-in at contribution.
        // → $150k disallowed under § 336(d)(2); $50k still recognized.
        let mut i = base();
        i.distributed_property_fmv_dollars = 400_000;
        i.property_acquired_via_351_or_capital_contribution_within_5_years = true;
        i.built_in_loss_at_contribution_dollars = 150_000;
        let r = compute(&i);
        assert_eq!(
            r.section_336d2_built_in_loss_disallowed_amount_dollars,
            150_000
        );
        assert_eq!(r.recognized_loss_dollars, 50_000);
    }

    #[test]
    fn d2_disallowance_capped_at_realized_loss() {
        // BIL $500k at contribution, but only $100k loss realized.
        let mut i = base();
        i.distributed_property_fmv_dollars = 500_000;
        i.property_acquired_via_351_or_capital_contribution_within_5_years = true;
        i.built_in_loss_at_contribution_dollars = 500_000;
        let r = compute(&i);
        assert_eq!(
            r.section_336d2_built_in_loss_disallowed_amount_dollars,
            100_000
        );
        assert_eq!(r.recognized_loss_dollars, 0);
    }

    #[test]
    fn d2_not_applicable_outside_5_year_window() {
        let mut i = base();
        i.distributed_property_fmv_dollars = 400_000;
        i.property_acquired_via_351_or_capital_contribution_within_5_years = false;
        i.built_in_loss_at_contribution_dollars = 150_000;
        let r = compute(&i);
        assert_eq!(r.section_336d2_built_in_loss_disallowed_amount_dollars, 0);
        assert_eq!(r.recognized_loss_dollars, 200_000);
    }

    // §336(d)(3) §332 subsidiary exception.

    #[test]
    fn section_332_subsidiary_no_gain_no_loss() {
        let mut i = base();
        i.section_332_subsidiary_liquidation = true;
        let r = compute(&i);
        assert!(r.section_332_full_nonrecognition);
        assert_eq!(r.recognized_gain_dollars, 0);
        assert_eq!(r.recognized_loss_dollars, 0);
        assert!(r.note.contains("§332 subsidiary exception"));
    }

    #[test]
    fn section_332_overrides_all_other_provisions() {
        // Even with related party + 5-year + built-in loss, §332 wins.
        let mut i = base();
        i.distributed_property_fmv_dollars = 400_000;
        i.distributee_is_related_party_more_than_50_pct = true;
        i.property_acquired_via_351_or_capital_contribution_within_5_years = true;
        i.section_332_subsidiary_liquidation = true;
        let r = compute(&i);
        assert!(r.section_332_full_nonrecognition);
        assert_eq!(r.recognized_loss_dollars, 0);
    }

    // Combined scenarios.

    #[test]
    fn d1_takes_precedence_over_d2() {
        // Related-party non-pro-rata → § 336(d)(1) full disallowance
        // (loss = 0); § 336(d)(2) amount surfaces separately but
        // doesn't matter because loss already zero.
        let mut i = base();
        i.distributed_property_fmv_dollars = 400_000;
        i.distributee_is_related_party_more_than_50_pct = true;
        i.distribution_is_pro_rata_to_all_shareholders = false;
        i.property_acquired_via_351_or_capital_contribution_within_5_years = true;
        i.built_in_loss_at_contribution_dollars = 100_000;
        let r = compute(&i);
        assert!(r.section_336d1_related_party_loss_disallowed);
        assert_eq!(r.recognized_loss_dollars, 0);
    }

    // Notes / citations.

    #[test]
    fn note_describes_336a_general_rule() {
        let r = compute(&base());
        assert!(r.note.contains("§ 336(a) general rule"));
    }

    #[test]
    fn note_describes_336b_liability_when_triggered() {
        let mut i = base();
        i.distributed_property_fmv_dollars = 400_000;
        i.liability_amount_on_property_dollars = 700_000;
        let r = compute(&i);
        assert!(r.note.contains("§ 336(b) liability rule"));
    }

    #[test]
    fn note_describes_d1_disallowance() {
        let mut i = base();
        i.distributed_property_fmv_dollars = 400_000;
        i.distributee_is_related_party_more_than_50_pct = true;
        i.distribution_is_pro_rata_to_all_shareholders = false;
        let r = compute(&i);
        assert!(r.note.contains("§ 336(d)(1) related-party loss DISALLOWED"));
    }

    #[test]
    fn note_describes_d2_disallowance() {
        let mut i = base();
        i.distributed_property_fmv_dollars = 400_000;
        i.property_acquired_via_351_or_capital_contribution_within_5_years = true;
        i.built_in_loss_at_contribution_dollars = 100_000;
        let r = compute(&i);
        assert!(r.note.contains("§ 336(d)(2) anti-tax-avoidance"));
    }

    #[test]
    fn citation_mentions_all_relevant_authorities() {
        let r = compute(&base());
        assert!(r.citation.contains("§336(a)"));
        assert!(r.citation.contains("§336(b)"));
        assert!(r.citation.contains("§336(d)(1)"));
        assert!(r.citation.contains("§336(d)(2)"));
        assert!(r.citation.contains("§336(d)(3)"));
        assert!(r.citation.contains("§331"));
        assert!(r.citation.contains("§334(b)"));
    }

    // Precision.

    #[test]
    fn very_large_gain_precision() {
        let mut i = base();
        i.distributed_property_fmv_dollars = 10_000_000_000;
        i.distributed_property_adjusted_basis_dollars = 1_000_000_000;
        let r = compute(&i);
        assert_eq!(r.recognized_gain_dollars, 9_000_000_000);
    }

    #[test]
    fn zero_gain_zero_loss_no_op() {
        let mut i = base();
        i.distributed_property_fmv_dollars = 600_000; // = basis
        let r = compute(&i);
        assert_eq!(r.realized_gain_or_loss_dollars, 0);
        assert_eq!(r.recognized_gain_dollars, 0);
        assert_eq!(r.recognized_loss_dollars, 0);
    }
}
