//! Alternative Minimum Tax — IRC §§ 55–59 (Form 6251).
//!
//! Computes the AMT add-on for 2025 returns.
//!
//! ### Mechanism
//!
//! 1. Start with regular taxable income (Form 1040 line 15).
//! 2. Add back AMT preferences and adjustments (Form 6251 lines 2a–3).
//!    For most non-corporate taxpayers post-TCJA the dominant preference
//!    is the state-and-local-tax deduction taken on Schedule A — even
//!    though SALT is already capped at $10,000 by IRC § 164(b)(6), the
//!    deducted amount is added back in full for AMT.
//! 3. Subtract the filing-status AMT exemption (with the high-income
//!    phaseout — 25¢ of exemption lost per $1 of AMTI above the
//!    phaseout threshold).
//! 4. Apply the AMT rate schedule (26% / 28%) to the result.
//! 5. The result is the Tentative Minimum Tax (TMT). Subtract the
//!    regular tax — if positive, AMT owed equals that difference and is
//!    added to total tax.
//!
//! ### 2025 constants (Rev. Proc. 2024-40 § 3.12)
//!
//! Exemptions:
//!   * Single / HoH:  $88,100
//!   * MFJ:           $137,000
//!   * MFS:           $68,500
//!
//! Phaseout starts (25¢ on the $1):
//!   * Single / HoH / MFS:  $626,350
//!   * MFJ:                 $1,252,700
//!
//! 26%/28% breakpoint (IRC § 55(b)(1)(A)(i)):
//!   * Non-MFS:  $239,100  (taxable-excess at and below = 26%, above = 28%)
//!   * MFS:      $119,550
//!
//! ### What this module models in v1
//!
//! The only AMT preference items most users encounter are:
//!   * State and local taxes deducted on Schedule A.
//!   * (Less commonly) ISO exercise spread, private-activity-bond
//!     interest, depreciation differences. These can be supplied via
//!     `AmtInput::additional_preferences` and are simply added to AMTI.
//!
//! Sources:
//!   * IRC §§ 55–59
//!   * Form 6251 instructions (2024 — reused for 2025 amounts)
//!   * Rev. Proc. 2024-40

use crate::engine::FilingStatus;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
struct AmtConsts {
    exemption: Decimal,
    phaseout_start: Decimal,
    twenty_six_pct_top: Decimal,
}

fn consts(status: FilingStatus) -> AmtConsts {
    match status {
        FilingStatus::Single | FilingStatus::Hoh => AmtConsts {
            exemption: Decimal::from(88_100),
            phaseout_start: Decimal::from(626_350),
            twenty_six_pct_top: Decimal::from(239_100),
        },
        FilingStatus::Mfj => AmtConsts {
            exemption: Decimal::from(137_000),
            phaseout_start: Decimal::from(1_252_700),
            twenty_six_pct_top: Decimal::from(239_100),
        },
        FilingStatus::Mfs => AmtConsts {
            exemption: Decimal::from(68_500),
            phaseout_start: Decimal::from(626_350),
            twenty_six_pct_top: Decimal::from(119_550),
        },
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AmtInput {
    /// Form 1040 line 15 — taxable income after standard or itemized
    /// deduction and after QBI.
    pub taxable_income: Decimal,
    /// Regular income tax (Form 1040 line 16) — what AMT is compared
    /// against. Use the post-QDCGTW total tax.
    pub regular_tax: Decimal,
    /// State and local taxes deducted on Schedule A line 7. Pass `0`
    /// when the taxpayer used the standard deduction (TCJA suspended
    /// the standard-deduction addback for AMT 2018–2025).
    pub salt_deduction_used: Decimal,
    /// Catch-all for rare preferences (ISO spread, PAB interest, etc.).
    /// Added to AMTI verbatim.
    pub additional_preferences: Decimal,
    pub status: FilingStatus,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct AmtResult {
    /// Alternative Minimum Taxable Income (Form 6251 line 4).
    pub amti: Decimal,
    /// Statutory exemption for this filing status.
    pub exemption_before_phaseout: Decimal,
    /// Exemption after 25%-per-dollar phaseout for high AMTI.
    pub exemption_after_phaseout: Decimal,
    /// Taxable excess = max(0, AMTI - exemption).
    pub taxable_excess: Decimal,
    /// Tentative Minimum Tax (Form 6251 line 7) before regular-tax credit.
    pub tentative_minimum_tax: Decimal,
    /// Regular tax used as the floor (Form 6251 line 9).
    pub regular_tax_floor: Decimal,
    /// AMT owed = max(0, TMT - regular tax). Rounded to cents.
    pub amt_owed: Decimal,
}

pub fn compute(input: AmtInput) -> AmtResult {
    let c = consts(input.status);

    // Form 6251 line 1 + 2 + 3 → line 4 (AMTI).
    let amti = (input.taxable_income + input.salt_deduction_used + input.additional_preferences)
        .max(Decimal::ZERO);

    // Phaseout: 25¢ of exemption lost per $1 of AMTI above threshold.
    let excess_over_phaseout = (amti - c.phaseout_start).max(Decimal::ZERO);
    let quarter: Decimal = "0.25".parse().unwrap();
    let phaseout_reduction = excess_over_phaseout * quarter;
    let exemption_after_phaseout = (c.exemption - phaseout_reduction).max(Decimal::ZERO);

    // Taxable excess feeds the rate schedule.
    let taxable_excess = (amti - exemption_after_phaseout).max(Decimal::ZERO);

    // 26% up to twenty_six_pct_top, 28% above. The 2% "rate-recapture"
    // adjustment (Form 6251 line 39 worksheet) collapses to: tax =
    // 0.28 × taxable_excess - 0.02 × min(taxable_excess, top).
    let rate_26: Decimal = "0.26".parse().unwrap();
    let rate_28: Decimal = "0.28".parse().unwrap();
    let tmt = if taxable_excess <= c.twenty_six_pct_top {
        taxable_excess * rate_26
    } else {
        c.twenty_six_pct_top * rate_26 + (taxable_excess - c.twenty_six_pct_top) * rate_28
    };
    let tmt = tmt.round_dp(2);

    let amt_owed = (tmt - input.regular_tax).max(Decimal::ZERO).round_dp(2);

    AmtResult {
        amti,
        exemption_before_phaseout: c.exemption,
        exemption_after_phaseout,
        taxable_excess,
        tentative_minimum_tax: tmt,
        regular_tax_floor: input.regular_tax,
        amt_owed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(n: i64) -> Decimal {
        Decimal::from(n)
    }
    fn dc(s: &str) -> Decimal {
        s.parse().unwrap()
    }

    fn base(status: FilingStatus) -> AmtInput {
        AmtInput {
            taxable_income: Decimal::ZERO,
            regular_tax: Decimal::ZERO,
            salt_deduction_used: Decimal::ZERO,
            additional_preferences: Decimal::ZERO,
            status,
        }
    }

    #[test]
    fn low_income_owes_no_amt() {
        // $50k TI, no preferences. AMTI = $50k < exemption $88,100.
        // Taxable excess = 0 → TMT = 0 → AMT owed = 0.
        let r = compute(AmtInput {
            taxable_income: d(50_000),
            regular_tax: d(5_000),
            ..base(FilingStatus::Single)
        });
        assert_eq!(r.amti, d(50_000));
        assert_eq!(r.exemption_after_phaseout, d(88_100));
        assert_eq!(r.taxable_excess, Decimal::ZERO);
        assert_eq!(r.tentative_minimum_tax, Decimal::ZERO);
        assert_eq!(r.amt_owed, Decimal::ZERO);
    }

    #[test]
    fn salt_addback_increases_amti() {
        // Single, TI = $200k, SALT $10k. AMTI = $210k.
        let r = compute(AmtInput {
            taxable_income: d(200_000),
            regular_tax: d(40_000),
            salt_deduction_used: d(10_000),
            ..base(FilingStatus::Single)
        });
        assert_eq!(r.amti, d(210_000));
    }

    #[test]
    fn exemption_not_phased_out_below_phaseout_start() {
        // Single, AMTI = $500k (well below $626,350 phaseout start).
        let r = compute(AmtInput {
            taxable_income: d(500_000),
            regular_tax: d(150_000),
            ..base(FilingStatus::Single)
        });
        assert_eq!(r.exemption_after_phaseout, d(88_100));
    }

    #[test]
    fn exemption_phases_out_25pct_per_dollar_above_threshold() {
        // Single, AMTI = $700k. Excess over $626,350 = $73,650.
        // Phaseout reduction = $73,650 × 25% = $18,412.50.
        // Exemption after = $88,100 - $18,412.50 = $69,687.50.
        let r = compute(AmtInput {
            taxable_income: d(700_000),
            regular_tax: d(0),
            ..base(FilingStatus::Single)
        });
        assert_eq!(r.exemption_after_phaseout, dc("69687.50"));
    }

    #[test]
    fn exemption_fully_phased_out_at_amti_4x_exemption_plus_threshold() {
        // Single phaseout fully eats exemption when AMTI ≥ $626,350 + 4 × $88,100 = $978,750.
        let r = compute(AmtInput {
            taxable_income: d(1_000_000),
            regular_tax: d(300_000),
            ..base(FilingStatus::Single)
        });
        assert_eq!(r.exemption_after_phaseout, Decimal::ZERO);
    }

    #[test]
    fn tmt_uses_26pct_below_breakpoint() {
        // Single, taxable_excess = $100k (< $239,100 → all 26%).
        // AMTI = $188,100 → excess = $100k. TMT = 26,000.
        let r = compute(AmtInput {
            taxable_income: d(188_100),
            regular_tax: d(40_000),
            ..base(FilingStatus::Single)
        });
        assert_eq!(r.taxable_excess, d(100_000));
        assert_eq!(r.tentative_minimum_tax, d(26_000));
    }

    #[test]
    fn tmt_uses_28pct_above_breakpoint() {
        // Single, AMTI = $400k, exemption fully intact ($88,100), taxable_excess = $311,900.
        // First $239,100 @ 26% = 62,166. Next $72,800 @ 28% = 20,384. TMT = 82,550.
        let r = compute(AmtInput {
            taxable_income: d(400_000),
            regular_tax: d(80_000),
            ..base(FilingStatus::Single)
        });
        assert_eq!(r.taxable_excess, dc("311900"));
        assert_eq!(r.tentative_minimum_tax, dc("82550.00"));
    }

    #[test]
    fn amt_owed_only_when_tmt_exceeds_regular_tax() {
        // High regular tax should swallow TMT → AMT = 0.
        let r = compute(AmtInput {
            taxable_income: d(400_000),
            regular_tax: d(100_000), // > TMT $82,550 from prior test
            ..base(FilingStatus::Single)
        });
        assert_eq!(r.amt_owed, Decimal::ZERO);
    }

    #[test]
    fn amt_owed_equals_tmt_minus_regular_tax() {
        let r = compute(AmtInput {
            taxable_income: d(400_000),
            regular_tax: d(50_000), // TMT $82,550 - $50k = $32,550
            ..base(FilingStatus::Single)
        });
        assert_eq!(r.amt_owed, dc("32550.00"));
    }

    #[test]
    fn mfj_exemption_is_137k_not_double_single() {
        let r = compute(AmtInput {
            taxable_income: d(200_000),
            regular_tax: Decimal::ZERO,
            ..base(FilingStatus::Mfj)
        });
        assert_eq!(r.exemption_before_phaseout, d(137_000));
    }

    #[test]
    fn mfs_uses_smaller_119550_breakpoint() {
        // MFS, AMTI = $400k. Exemption $68,500 (not phased — phaseout starts at $626,350).
        // Taxable excess = $331,500. Below $119,550 @ 26% = 31,083.
        // Above ($331,500 - $119,550 = $211,950) @ 28% = 59,346. TMT = 90,429.
        let r = compute(AmtInput {
            taxable_income: d(400_000),
            regular_tax: Decimal::ZERO,
            ..base(FilingStatus::Mfs)
        });
        assert_eq!(r.taxable_excess, dc("331500"));
        assert_eq!(r.tentative_minimum_tax, dc("90429.00"));
    }

    #[test]
    fn additional_preferences_flow_into_amti() {
        // ISO spread of $25k bumps AMTI by exactly that.
        let r = compute(AmtInput {
            taxable_income: d(100_000),
            regular_tax: d(15_000),
            salt_deduction_used: Decimal::ZERO,
            additional_preferences: d(25_000),
            status: FilingStatus::Single,
        });
        assert_eq!(r.amti, d(125_000));
    }

    #[test]
    fn negative_excess_clamps_at_zero() {
        // Taxable income below the exemption → AMT cannot be negative.
        let r = compute(AmtInput {
            taxable_income: d(60_000),
            regular_tax: d(8_000),
            ..base(FilingStatus::Single)
        });
        assert_eq!(r.taxable_excess, Decimal::ZERO);
        assert_eq!(r.amt_owed, Decimal::ZERO);
    }
}
