//! Quarterly estimated-tax safe harbor calculator (IRC § 6654).
//!
//! The penalty for underpayment of estimated tax is avoided if total
//! payments meet one of two safe harbors:
//!
//!   1. **Prior-year safe harbor** — pay (in withholding + quarterly
//!      estimates) at least 100% of prior year's tax liability, or
//!      110% if prior-year AGI was > $150,000 (> $75,000 MFS).
//!   2. **Current-year safe harbor** — pay at least 90% of the current
//!      year's actual tax liability.
//!
//! The IRS treats withholding as evenly distributed across the year.
//! Estimated quarterly payments are credited to the quarter they're
//! made in. The four due dates are:
//!   Q1: Apr 15  · Q2: Jun 15  · Q3: Sep 15  · Q4: Jan 15 (next year)
//!
//! To avoid penalty THROUGH quarter N, cumulative payments by that
//! quarter's due date must meet `min(prior_year_safe, current_year_safe)
//! × (N / 4)`. The calculator returns the smaller of the two safe
//! harbors as the binding floor, then divides by 4 to get the
//! quarterly target.
//!
//! Sources:
//! * IRC § 6654 — Failure by individual to pay estimated income tax.
//! * Form 2210 — Underpayment of Estimated Tax by Individuals.
//! * IRS Pub 505 ch. 4 — Estimated Tax for Individuals.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::engine::FilingStatus;

/// Inputs for the safe-harbor calculation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SafeHarborInput {
    /// Prior tax year's total federal income tax liability (line 24
    /// of Form 1040). The 110% rule kicks in when prior_year AGI is
    /// over the threshold.
    pub prior_year_tax: Decimal,
    /// Prior-year AGI. Determines whether the 100% or 110% prior-year
    /// floor applies.
    pub prior_year_agi: Decimal,
    pub filing_status: FilingStatus,
    /// Current year's projected total federal tax liability. If you're
    /// confident in your projection (e.g. salary unchanged), the 90%
    /// rule on this number may be cheaper than 100/110% of last year.
    pub current_year_projected_tax: Decimal,
    /// W-2 federal income tax withholding YTD (treated as paid evenly
    /// across the year by IRS).
    pub w2_withholding_ytd: Decimal,
    /// Sum of estimated-tax payments already made this year (across
    /// quarters 1..=current_quarter).
    pub estimated_paid_ytd: Decimal,
    /// Which quarter you're computing through (1..=4).
    /// Q1 = through Apr 15
    /// Q2 = through Jun 15
    /// Q3 = through Sep 15
    /// Q4 = through Jan 15 (of next year)
    pub current_quarter: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct SafeHarborResult {
    /// Annual safe-harbor floor (100% or 110% of prior year).
    pub prior_year_safe_amount: Decimal,
    /// Annual safe-harbor floor (90% of current year projection).
    pub current_year_safe_amount: Decimal,
    /// The binding annual floor (the LESSER of the two — taxpayer's
    /// choice).
    pub annual_floor: Decimal,
    /// Cumulative payment target through the current quarter
    /// (annual_floor × quarter / 4).
    pub cumulative_target: Decimal,
    /// Total payments already credited (withholding + estimates YTD).
    pub paid_to_date: Decimal,
    /// What you need to pay BY THIS QUARTER'S DUE DATE to avoid penalty.
    /// Zero means you're caught up.
    pub additional_due_this_quarter: Decimal,
    /// If positive, you've overpaid through this quarter — no additional
    /// payment needed. Equals `paid_to_date - cumulative_target` when
    /// non-negative.
    pub surplus: Decimal,
    /// Which safe harbor binds (which floor was lower). Useful for the UI
    /// to label "you're using the prior-year safe harbor" vs "current-year".
    pub binding_harbor: BindingHarbor,
    /// 110% rule applied to prior-year floor because of AGI threshold.
    pub prior_year_high_income: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BindingHarbor {
    /// Prior-year floor was lower (cheaper to use last year's number).
    #[default]
    PriorYear,
    /// Current-year projection was lower (a refund year — projecting
    /// down).
    CurrentYear,
}

pub fn compute(input: SafeHarborInput) -> SafeHarborResult {
    // Prior-year threshold — 110% kicks in above the per-status AGI line.
    let high_income_threshold = match input.filing_status {
        FilingStatus::Mfs => Decimal::from(75_000),
        _ => Decimal::from(150_000),
    };
    let prior_year_high_income = input.prior_year_agi > high_income_threshold;

    let prior_year_factor = if prior_year_high_income {
        "1.10".parse::<Decimal>().unwrap()
    } else {
        Decimal::ONE
    };
    let prior_year_safe_amount = (input.prior_year_tax * prior_year_factor).max(Decimal::ZERO);

    let current_year_safe_amount =
        (input.current_year_projected_tax * "0.90".parse::<Decimal>().unwrap()).max(Decimal::ZERO);

    let annual_floor = prior_year_safe_amount.min(current_year_safe_amount);
    let binding_harbor = if prior_year_safe_amount <= current_year_safe_amount {
        BindingHarbor::PriorYear
    } else {
        BindingHarbor::CurrentYear
    };

    // Quarter clamped to [1, 4].
    let q = input.current_quarter.clamp(1, 4) as i64;
    let cumulative_target = (annual_floor * Decimal::from(q) / Decimal::from(4)).round_dp(2);

    let paid_to_date = input.w2_withholding_ytd + input.estimated_paid_ytd;
    let raw_diff = cumulative_target - paid_to_date;
    let (additional_due_this_quarter, surplus) = if raw_diff > Decimal::ZERO {
        (raw_diff.round_dp(2), Decimal::ZERO)
    } else {
        (Decimal::ZERO, (-raw_diff).round_dp(2))
    };

    SafeHarborResult {
        prior_year_safe_amount,
        current_year_safe_amount,
        annual_floor,
        cumulative_target,
        paid_to_date,
        additional_due_this_quarter,
        surplus,
        binding_harbor,
        prior_year_high_income,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(n: i64) -> Decimal {
        Decimal::from(n)
    }

    fn base_input() -> SafeHarborInput {
        SafeHarborInput {
            prior_year_tax: d(10_000),
            prior_year_agi: d(80_000),
            filing_status: FilingStatus::Single,
            current_year_projected_tax: d(12_000),
            w2_withholding_ytd: Decimal::ZERO,
            estimated_paid_ytd: Decimal::ZERO,
            current_quarter: 1,
        }
    }

    #[test]
    fn under_threshold_uses_100_pct_prior_year() {
        // Prior AGI $80k < $150k → 100% of prior year tax.
        // Prior tax $10k → annual floor = $10k.
        // 90% of current = $10,800. min($10k, $10,800) = $10k.
        let r = compute(base_input());
        assert_eq!(r.prior_year_safe_amount, d(10_000));
        assert_eq!(r.current_year_safe_amount, d(10_800));
        assert_eq!(r.annual_floor, d(10_000));
        assert_eq!(r.binding_harbor, BindingHarbor::PriorYear);
        assert!(!r.prior_year_high_income);
    }

    #[test]
    fn over_threshold_uses_110_pct_prior_year() {
        // Prior AGI $200k > $150k → 110% of prior year tax.
        let inp = SafeHarborInput {
            prior_year_agi: d(200_000),
            prior_year_tax: d(40_000),
            current_year_projected_tax: d(50_000),
            ..base_input()
        };
        let r = compute(inp);
        // 110% × $40k = $44k. 90% × $50k = $45k. min = $44k.
        assert_eq!(r.prior_year_safe_amount, d(44_000));
        assert!(r.prior_year_high_income);
    }

    #[test]
    fn mfs_high_income_threshold_is_75k() {
        // MFS threshold is half of MFJ → $75,000.
        let inp = SafeHarborInput {
            filing_status: FilingStatus::Mfs,
            prior_year_agi: d(80_000), // > $75k MFS threshold
            prior_year_tax: d(10_000),
            ..base_input()
        };
        let r = compute(inp);
        assert!(r.prior_year_high_income);
        // 110% × $10k = $11k
        assert_eq!(r.prior_year_safe_amount, d(11_000));
    }

    #[test]
    fn current_year_projection_lower_picks_current_year_harbor() {
        // Refund year — current projection is much lower than last year.
        let inp = SafeHarborInput {
            prior_year_tax: d(20_000),            // last year was big
            current_year_projected_tax: d(8_000), // big drop this year
            ..base_input()
        };
        let r = compute(inp);
        // Prior safe = $20k, current safe = 90% × $8k = $7,200.
        // Binding = current ($7,200 < $20k).
        assert_eq!(r.annual_floor, d(7_200));
        assert_eq!(r.binding_harbor, BindingHarbor::CurrentYear);
    }

    #[test]
    fn q1_target_is_quarter_of_annual() {
        // Annual floor $10k, Q1 → $2,500 cumulative target.
        let r = compute(base_input());
        assert_eq!(r.cumulative_target, d(2_500));
        assert_eq!(r.additional_due_this_quarter, d(2_500));
        assert_eq!(r.surplus, Decimal::ZERO);
    }

    #[test]
    fn q3_target_is_three_quarters_of_annual() {
        // Through Q3, cumulative target = 75% of annual floor.
        let inp = SafeHarborInput {
            current_quarter: 3,
            ..base_input()
        };
        let r = compute(inp);
        assert_eq!(r.cumulative_target, d(7_500));
    }

    #[test]
    fn withholding_credits_toward_quarterly_target() {
        // W-2 withholding $4k YTD by Q2. Annual floor $10k → Q2
        // cumulative target $5k. Withholding $4k → additional due $1k.
        let inp = SafeHarborInput {
            w2_withholding_ytd: d(4_000),
            current_quarter: 2,
            ..base_input()
        };
        let r = compute(inp);
        assert_eq!(r.cumulative_target, d(5_000));
        assert_eq!(r.paid_to_date, d(4_000));
        assert_eq!(r.additional_due_this_quarter, d(1_000));
    }

    #[test]
    fn overpayment_produces_surplus_not_negative_due() {
        // Massively overpaid. additional_due must be 0, surplus = excess.
        let inp = SafeHarborInput {
            w2_withholding_ytd: d(15_000),
            current_quarter: 4,
            ..base_input()
        };
        let r = compute(inp);
        // Q4 target = annual $10k full year. Paid $15k → surplus $5k.
        assert_eq!(r.cumulative_target, d(10_000));
        assert_eq!(r.additional_due_this_quarter, Decimal::ZERO);
        assert_eq!(r.surplus, d(5_000));
    }

    #[test]
    fn zero_prior_year_tax_means_zero_floor() {
        // First-year filer (no prior return) → prior_year_tax = 0.
        // Falls back to 90% of current projection.
        let inp = SafeHarborInput {
            prior_year_tax: Decimal::ZERO,
            prior_year_agi: Decimal::ZERO,
            current_year_projected_tax: d(10_000),
            ..base_input()
        };
        let r = compute(inp);
        // Prior safe = $0; current safe = $9k. Floor = $0 (prior wins as
        // lower). That means no estimated payments are required to avoid
        // penalty — first-year filer escape hatch.
        assert_eq!(r.annual_floor, Decimal::ZERO);
        assert_eq!(r.binding_harbor, BindingHarbor::PriorYear);
        assert_eq!(r.additional_due_this_quarter, Decimal::ZERO);
    }

    #[test]
    fn quarter_clamps_to_valid_range() {
        // current_quarter=0 → clamp to 1; current_quarter=99 → clamp to 4.
        let mut inp = base_input();
        inp.current_quarter = 0;
        let r0 = compute(inp);
        assert_eq!(r0.cumulative_target, d(2_500)); // q=1 target
        inp.current_quarter = 99;
        let r99 = compute(inp);
        assert_eq!(r99.cumulative_target, d(10_000)); // q=4 full annual
    }
}
