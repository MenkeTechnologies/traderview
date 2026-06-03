//! IRC § 723 basis of property contributed to partnership.
//!
//! Foundational Subchapter K inside-basis provision that pairs with
//! § 721 (contribution nonrecognition rule) and § 722 (partner's
//! outside basis in interest received).
//!
//! **§ 723 operative rule**: "The basis of property contributed to
//! a partnership by a partner shall be the adjusted basis of such
//! property to the contributing partner at the time of the
//! contribution increased by the amount (if any) of gain recognized
//! under section 721(b) to the contributing partner at such time."
//!
//! In plain terms: partnership takes a **carryover basis** from
//! the contributing partner, PLUS any gain recognized under
//! § 721(b) (the investment-company exception).
//!
//! **§ 721 contribution nonrecognition framework**:
//!
//! - **§ 721(a) general rule**: no gain or loss is recognized on
//!   the transfer of property to a partnership in exchange for a
//!   partnership interest.
//! - **§ 721(b) investment-company exception**: § 721(a) does NOT
//!   apply if the partnership would be treated as an investment
//!   company under § 351(e)(1) (i.e., more than 80% of partnership
//!   assets are held for investment in stocks/securities). In that
//!   case, the contributing partner recognizes gain to the extent
//!   the FMV of property contributed exceeds basis (no loss
//!   recognition).
//!
//! **§ 722 paired outside-basis rule**: contributing partner's
//! outside basis in the partnership interest received equals the
//! adjusted basis of the contributed property PLUS § 721(b) gain
//! recognized. Outside basis and inside basis under § 723 are thus
//! equal at the moment of contribution.
//!
//! **§ 704(c) built-in gain preservation**: when contributed
//! property has built-in gain or loss at the time of contribution
//! (i.e., FMV ≠ § 723 carryover basis), § 704(c) requires that the
//! pre-contribution gain or loss be allocated to the contributing
//! partner upon partnership disposition. This prevents tax
//! consequences from shifting to non-contributing partners.
//!
//! **§ 1223(2) holding-period tacking**: partnership's holding
//! period in contributed property includes the contributing
//! partner's holding period (tacking rule). Critical for
//! long-term vs. short-term capital character on subsequent
//! partnership disposition.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[allow(dead_code)]
pub const INVESTMENT_COMPANY_TEST_THRESHOLD_PERCENT: u32 = 80;
#[allow(dead_code)]
pub const LONG_TERM_HOLDING_PERIOD_THRESHOLD_MONTHS: u32 = 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    CompliantCarryoverBasisGeneralRule,
    CompliantCarryoverBasisPlusSection721bGainRecognized,
    ViolationStepUpBasisToFmvWithoutSection721bGain,
    ViolationBasisAdjustedDownwardWithoutAuthority,
    ViolationMissingHoldingPeriodTacking,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub contributing_partner_adjusted_basis_cents: u64,
    pub fair_market_value_at_contribution_cents: u64,
    pub section_721b_gain_recognized_cents: u64,
    pub partnership_classified_as_investment_company: bool,
    pub partnership_taxpayer_recorded_basis_cents: u64,
    pub contributing_partner_holding_period_months: u32,
    pub partnership_tacked_holding_period: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Output {
    pub severity: Severity,
    pub partnership_inside_basis_cents: u64,
    pub built_in_gain_loss_cents: i128,
    pub holding_period_tacked: bool,
    pub long_term_at_contribution: bool,
    pub notes: Vec<String>,
    pub citations: Vec<String>,
}

pub type Section723Input = Input;
pub type Section723Output = Output;
pub type Section723Result = Output;

pub fn check(input: &Input) -> Output {
    let mut notes: Vec<String> = Vec::new();
    let citations: Vec<String> = vec![
        "IRC § 723 (basis of property contributed to partnership)".to_string(),
        "IRC § 721(a) (general nonrecognition rule on contribution)".to_string(),
        "IRC § 721(b) (investment-company exception — gain recognition required)".to_string(),
        "IRC § 722 (contributing partner's outside basis in partnership interest received)".to_string(),
        "IRC § 351(e)(1) (investment-company definition — 80%-of-assets threshold)".to_string(),
        "IRC § 704(c) (built-in gain allocation to contributing partner)".to_string(),
        "IRC § 1223(2) (partnership holding-period tacking rule)".to_string(),
        "IRC § 752 (partnership liabilities — outside-basis adjustment, distinct from § 723 inside basis)".to_string(),
        "Treas. Reg. § 1.723-1 (general rule)".to_string(),
        "Treas. Reg. § 1.704-3 (§ 704(c) traditional, curative, and remedial methods)".to_string(),
    ];

    let correct_inside_basis = input
        .contributing_partner_adjusted_basis_cents
        .saturating_add(input.section_721b_gain_recognized_cents);

    if input.partnership_taxpayer_recorded_basis_cents > correct_inside_basis {
        notes.push(format!(
            "Partnership recorded basis ${} > correct § 723 carryover basis ${} — improper step-up without § 721(b) gain authority.",
            input.partnership_taxpayer_recorded_basis_cents / 100,
            correct_inside_basis / 100
        ));
        return Output {
            severity: Severity::ViolationStepUpBasisToFmvWithoutSection721bGain,
            partnership_inside_basis_cents: correct_inside_basis,
            built_in_gain_loss_cents: (input.fair_market_value_at_contribution_cents as i128)
                - (correct_inside_basis as i128),
            holding_period_tacked: input.partnership_tacked_holding_period,
            long_term_at_contribution: input.contributing_partner_holding_period_months
                >= LONG_TERM_HOLDING_PERIOD_THRESHOLD_MONTHS,
            notes,
            citations,
        };
    }

    if input.partnership_taxpayer_recorded_basis_cents < correct_inside_basis {
        notes.push(format!(
            "Partnership recorded basis ${} < correct § 723 carryover basis ${} — improper downward adjustment without statutory authority.",
            input.partnership_taxpayer_recorded_basis_cents / 100,
            correct_inside_basis / 100
        ));
        return Output {
            severity: Severity::ViolationBasisAdjustedDownwardWithoutAuthority,
            partnership_inside_basis_cents: correct_inside_basis,
            built_in_gain_loss_cents: (input.fair_market_value_at_contribution_cents as i128)
                - (correct_inside_basis as i128),
            holding_period_tacked: input.partnership_tacked_holding_period,
            long_term_at_contribution: input.contributing_partner_holding_period_months
                >= LONG_TERM_HOLDING_PERIOD_THRESHOLD_MONTHS,
            notes,
            citations,
        };
    }

    if !input.partnership_tacked_holding_period {
        notes.push("Partnership failed to tack contributing partner's holding period — § 1223(2) violation; could mischaracterize subsequent gain as short-term.".to_string());
        return Output {
            severity: Severity::ViolationMissingHoldingPeriodTacking,
            partnership_inside_basis_cents: correct_inside_basis,
            built_in_gain_loss_cents: (input.fair_market_value_at_contribution_cents as i128)
                - (correct_inside_basis as i128),
            holding_period_tacked: false,
            long_term_at_contribution: input.contributing_partner_holding_period_months
                >= LONG_TERM_HOLDING_PERIOD_THRESHOLD_MONTHS,
            notes,
            citations,
        };
    }

    let built_in = (input.fair_market_value_at_contribution_cents as i128)
        - (correct_inside_basis as i128);

    let severity = if input.section_721b_gain_recognized_cents > 0 {
        notes.push(format!(
            "§ 723 carryover basis ${} (contributing partner adjusted basis) PLUS § 721(b) gain recognized ${} = ${} partnership inside basis. § 704(c) built-in gain ${}.",
            input.contributing_partner_adjusted_basis_cents / 100,
            input.section_721b_gain_recognized_cents / 100,
            correct_inside_basis / 100,
            built_in / 100
        ));
        Severity::CompliantCarryoverBasisPlusSection721bGainRecognized
    } else {
        notes.push(format!(
            "§ 723 general rule applied: partnership takes contributing partner's adjusted basis ${} as inside basis. § 704(c) built-in gain ${}; § 1223(2) holding-period tacked.",
            input.contributing_partner_adjusted_basis_cents / 100,
            built_in / 100
        ));
        Severity::CompliantCarryoverBasisGeneralRule
    };

    Output {
        severity,
        partnership_inside_basis_cents: correct_inside_basis,
        built_in_gain_loss_cents: built_in,
        holding_period_tacked: true,
        long_term_at_contribution: input.contributing_partner_holding_period_months
            >= LONG_TERM_HOLDING_PERIOD_THRESHOLD_MONTHS,
        notes,
        citations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_carryover_compliant() -> Input {
        Input {
            contributing_partner_adjusted_basis_cents: 5_000_000,
            fair_market_value_at_contribution_cents: 8_000_000,
            section_721b_gain_recognized_cents: 0,
            partnership_classified_as_investment_company: false,
            partnership_taxpayer_recorded_basis_cents: 5_000_000,
            contributing_partner_holding_period_months: 24,
            partnership_tacked_holding_period: true,
        }
    }

    #[test]
    fn carryover_basis_general_rule_compliant() {
        let out = check(&base_carryover_compliant());
        assert_eq!(out.severity, Severity::CompliantCarryoverBasisGeneralRule);
        assert_eq!(out.partnership_inside_basis_cents, 5_000_000);
        assert_eq!(out.built_in_gain_loss_cents, 3_000_000);
        assert!(out.holding_period_tacked);
        assert!(out.long_term_at_contribution);
    }

    #[test]
    fn carryover_plus_721b_gain_compliant() {
        let mut i = base_carryover_compliant();
        i.partnership_classified_as_investment_company = true;
        i.section_721b_gain_recognized_cents = 3_000_000;
        i.partnership_taxpayer_recorded_basis_cents = 8_000_000;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::CompliantCarryoverBasisPlusSection721bGainRecognized
        );
        assert_eq!(out.partnership_inside_basis_cents, 8_000_000);
        assert_eq!(out.built_in_gain_loss_cents, 0);
    }

    #[test]
    fn step_up_to_fmv_without_721b_violation() {
        let mut i = base_carryover_compliant();
        i.partnership_taxpayer_recorded_basis_cents = 8_000_000;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationStepUpBasisToFmvWithoutSection721bGain
        );
    }

    #[test]
    fn step_down_without_authority_violation() {
        let mut i = base_carryover_compliant();
        i.partnership_taxpayer_recorded_basis_cents = 3_000_000;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationBasisAdjustedDownwardWithoutAuthority
        );
    }

    #[test]
    fn holding_period_not_tacked_violation() {
        let mut i = base_carryover_compliant();
        i.partnership_tacked_holding_period = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ViolationMissingHoldingPeriodTacking);
    }

    #[test]
    fn short_term_holding_period_under_12_months() {
        let mut i = base_carryover_compliant();
        i.contributing_partner_holding_period_months = 6;
        let out = check(&i);
        assert!(!out.long_term_at_contribution);
    }

    #[test]
    fn long_term_holding_at_exactly_12_months() {
        let mut i = base_carryover_compliant();
        i.contributing_partner_holding_period_months = 12;
        let out = check(&i);
        assert!(out.long_term_at_contribution);
    }

    #[test]
    fn built_in_loss_when_fmv_below_basis() {
        let mut i = base_carryover_compliant();
        i.fair_market_value_at_contribution_cents = 3_000_000;
        let out = check(&i);
        assert_eq!(out.built_in_gain_loss_cents, -2_000_000);
    }

    #[test]
    fn zero_built_in_gain_when_fmv_equals_basis() {
        let mut i = base_carryover_compliant();
        i.fair_market_value_at_contribution_cents = 5_000_000;
        let out = check(&i);
        assert_eq!(out.built_in_gain_loss_cents, 0);
    }

    #[test]
    fn citations_pin_723_721a_721b_722_704c() {
        let out = check(&base_carryover_compliant());
        assert!(out.citations.iter().any(|c| c.contains("§ 723")));
        assert!(out.citations.iter().any(|c| c.contains("§ 721(a)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 721(b)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 722")));
        assert!(out.citations.iter().any(|c| c.contains("§ 704(c)")));
    }

    #[test]
    fn citations_pin_351e1_1223_2_752_cross_refs() {
        let out = check(&base_carryover_compliant());
        assert!(out.citations.iter().any(|c| c.contains("§ 351(e)(1)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 1223(2)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 752")));
    }

    #[test]
    fn citations_pin_treas_reg_1_723_1_and_1_704_3() {
        let out = check(&base_carryover_compliant());
        assert!(out.citations.iter().any(|c| c.contains("§ 1.723-1")));
        assert!(out.citations.iter().any(|c| c.contains("§ 1.704-3")));
    }

    #[test]
    fn constant_pin_80_pct_investment_company_threshold() {
        assert_eq!(INVESTMENT_COMPANY_TEST_THRESHOLD_PERCENT, 80);
    }

    #[test]
    fn constant_pin_12_month_long_term_threshold() {
        assert_eq!(LONG_TERM_HOLDING_PERIOD_THRESHOLD_MONTHS, 12);
    }

    #[test]
    fn very_large_basis_with_721b_no_overflow() {
        let mut i = base_carryover_compliant();
        i.contributing_partner_adjusted_basis_cents = u64::MAX / 2;
        i.section_721b_gain_recognized_cents = u64::MAX / 2;
        i.partnership_taxpayer_recorded_basis_cents = u64::MAX;
        let out = check(&i);
        assert_eq!(out.partnership_inside_basis_cents, u64::MAX - 1);
    }
}
