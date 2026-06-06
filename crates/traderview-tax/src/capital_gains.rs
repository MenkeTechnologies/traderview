//! Long-term capital gains + qualified dividends preferential rates.
//!
//! Implements the Qualified Dividends and Capital Gain Tax Worksheet
//! (QDCGTW) — IRS Form 1040 instructions, page 36 (2024 instructions
//! reused for 2025 brackets). LTCG and qualified dividends are taxed
//! at 0% / 15% / 20% based on where they sit *on top of* the rest of
//! taxable income.
//!
//! 2025 breakpoints (Rev. Proc. 2024-40 § 3.07):
//!   * Single: 0% to $48,350, 15% to $533,400, 20% above.
//!   * MFS:    0% to $48,350, 15% to $300,000, 20% above.
//!   * MFJ:    0% to $96,700, 15% to $600,050, 20% above.
//!   * HoH:    0% to $64,750, 15% to $566,700, 20% above.
//!
//! Algorithm (mirrors the IRS worksheet, lines 1-25):
//!   1. Let TI = total taxable income.
//!   2. Let PREF = qualified dividends + net long-term capital gain
//!      (clamped at TI — preferential bucket can't exceed TI).
//!   3. Let ORD = TI - PREF (the rest, taxed at ordinary brackets).
//!   4. Compute the ordinary bracket tax on ORD.
//!   5. Stack PREF on top of ORD and slice it across the 0/15/20%
//!      breakpoints.
//!   6. Total tax = ordinary tax + preferential tax.

use crate::brackets::ordinary_income_tax;
use crate::engine::FilingStatus;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
struct PrefBreakpoints {
    /// Top of the 0% bracket — preferential income below this pays 0%.
    zero_pct_top: Decimal,
    /// Top of the 15% bracket — preferential income between
    /// `zero_pct_top` and this value pays 15%.
    fifteen_pct_top: Decimal,
}

/// 2025 LTCG / qualified-dividend breakpoints per filing status.
fn breakpoints(status: FilingStatus) -> PrefBreakpoints {
    match status {
        FilingStatus::Single => PrefBreakpoints {
            zero_pct_top: Decimal::from(48_350),
            fifteen_pct_top: Decimal::from(533_400),
        },
        FilingStatus::Mfj => PrefBreakpoints {
            zero_pct_top: Decimal::from(96_700),
            fifteen_pct_top: Decimal::from(600_050),
        },
        FilingStatus::Hoh => PrefBreakpoints {
            zero_pct_top: Decimal::from(64_750),
            fifteen_pct_top: Decimal::from(566_700),
        },
        FilingStatus::Mfs => PrefBreakpoints {
            zero_pct_top: Decimal::from(48_350),
            fifteen_pct_top: Decimal::from(300_000),
        },
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct QdcgtwInput {
    /// Total taxable income (post-deductions).
    pub taxable_income: Decimal,
    /// Net long-term capital gain (Schedule D line 16, positive only).
    pub net_long_term_capital_gain: Decimal,
    /// Qualified dividends (1099-DIV box 1b).
    pub qualified_dividends: Decimal,
    pub status: FilingStatus,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct QdcgtwResult {
    /// Preferential-rate income = LTCG + qualified divs, clamped at TI.
    pub preferential_income: Decimal,
    /// Ordinary-rate income = TI - preferential_income.
    pub ordinary_income: Decimal,
    /// Bracket tax on the ordinary slice only.
    pub ordinary_tax: Decimal,
    /// Amount taxed at 0%.
    pub amount_at_0_pct: Decimal,
    /// Amount taxed at 15%.
    pub amount_at_15_pct: Decimal,
    /// Amount taxed at 20%.
    pub amount_at_20_pct: Decimal,
    /// Tax on the preferential slice (15% × amount_at_15 + 20% × amount_at_20).
    pub preferential_tax: Decimal,
    /// Total tax = ordinary_tax + preferential_tax, rounded to cents.
    pub total_tax: Decimal,
}

pub fn compute(input: QdcgtwInput) -> QdcgtwResult {
    let ti = input.taxable_income.max(Decimal::ZERO);
    let raw_pref = input.net_long_term_capital_gain.max(Decimal::ZERO)
        + input.qualified_dividends.max(Decimal::ZERO);
    // Preferential bucket cannot exceed taxable income — Schedule D
    // worksheet line 6 caps it.
    let pref = raw_pref.min(ti);
    let ord = ti - pref;

    let bp = breakpoints(input.status);
    let ordinary_tax = ordinary_income_tax(ord, input.status).round_dp(2);

    // Stack preferential income on top of ordinary income, then slice
    // it through the 0% and 15% breakpoints.
    let pref_start = ord;
    let pref_end = ti;

    // Amount in the 0% band = portion of [pref_start, pref_end] below zero_pct_top.
    let zero_top = bp.zero_pct_top;
    let zero_band_end = zero_top.min(pref_end);
    let amount_at_0 = (zero_band_end - pref_start).max(Decimal::ZERO);

    // Amount in the 15% band = portion of [pref_start, pref_end] between zero_pct_top and fifteen_pct_top.
    let fifteen_top = bp.fifteen_pct_top;
    let fifteen_band_start = pref_start.max(zero_top);
    let fifteen_band_end = fifteen_top.min(pref_end);
    let amount_at_15 = (fifteen_band_end - fifteen_band_start).max(Decimal::ZERO);

    // Amount in the 20% band = remainder.
    let amount_at_20 = (pref - amount_at_0 - amount_at_15).max(Decimal::ZERO);

    let rate_15: Decimal = "0.15".parse().unwrap();
    let rate_20: Decimal = "0.20".parse().unwrap();
    let preferential_tax = (amount_at_15 * rate_15 + amount_at_20 * rate_20).round_dp(2);

    // IRS worksheet line 24: pick the LOWER of (worksheet calc) vs
    // (straight ordinary tax on entire TI). The worksheet only applies
    // when it produces a lower number — which it always does when
    // pref > 0, but the floor is there to handle weird edge cases.
    let worksheet_total = ordinary_tax + preferential_tax;
    let straight_ordinary = ordinary_income_tax(ti, input.status).round_dp(2);
    let total_tax = worksheet_total.min(straight_ordinary);

    QdcgtwResult {
        preferential_income: pref,
        ordinary_income: ord,
        ordinary_tax,
        amount_at_0_pct: amount_at_0,
        amount_at_15_pct: amount_at_15,
        amount_at_20_pct: amount_at_20,
        preferential_tax,
        total_tax,
    }
}

/// True when the engine should route through QDCGTW instead of straight
/// brackets — i.e., the taxpayer has preferential income.
pub fn has_preferential_income(qualified_dividends: Decimal, net_ltcg: Decimal) -> bool {
    qualified_dividends > Decimal::ZERO || net_ltcg > Decimal::ZERO
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

    #[test]
    fn single_all_in_zero_pct_band_pays_zero_pref_tax() {
        // TI = $40k, all from LTCG. Single 0% top = $48,350.
        // The entire $40k of LTCG falls in the 0% band.
        let r = compute(QdcgtwInput {
            taxable_income: d(40_000),
            net_long_term_capital_gain: d(40_000),
            qualified_dividends: Decimal::ZERO,
            status: FilingStatus::Single,
        });
        assert_eq!(r.preferential_income, d(40_000));
        assert_eq!(r.ordinary_income, Decimal::ZERO);
        assert_eq!(r.amount_at_0_pct, d(40_000));
        assert_eq!(r.amount_at_15_pct, Decimal::ZERO);
        assert_eq!(r.amount_at_20_pct, Decimal::ZERO);
        assert_eq!(r.preferential_tax, Decimal::ZERO);
        assert_eq!(r.ordinary_tax, Decimal::ZERO);
        assert_eq!(r.total_tax, Decimal::ZERO);
    }

    #[test]
    fn single_pref_straddles_zero_to_fifteen_boundary() {
        // TI = $60k, ORD = $20k, PREF = $40k.
        // PREF stack from $20k to $60k.
        // 0% band: [20k, 48,350] = 28,350 at 0%.
        // 15% band: [48,350, 60k] = 11,650 at 15% = 1,747.50.
        let r = compute(QdcgtwInput {
            taxable_income: d(60_000),
            net_long_term_capital_gain: d(40_000),
            qualified_dividends: Decimal::ZERO,
            status: FilingStatus::Single,
        });
        assert_eq!(r.preferential_income, d(40_000));
        assert_eq!(r.ordinary_income, d(20_000));
        assert_eq!(r.amount_at_0_pct, dc("28350"));
        assert_eq!(r.amount_at_15_pct, dc("11650"));
        assert_eq!(r.amount_at_20_pct, Decimal::ZERO);
        assert_eq!(r.preferential_tax, dc("1747.50"));
    }

    #[test]
    fn ordinary_above_zero_top_skips_zero_band() {
        // ORD = $50k (above $48,350), PREF = $20k.
        // PREF stack from $50k to $70k → entirely in 15% band.
        let r = compute(QdcgtwInput {
            taxable_income: d(70_000),
            net_long_term_capital_gain: d(20_000),
            qualified_dividends: Decimal::ZERO,
            status: FilingStatus::Single,
        });
        assert_eq!(r.amount_at_0_pct, Decimal::ZERO);
        assert_eq!(r.amount_at_15_pct, d(20_000));
        assert_eq!(r.amount_at_20_pct, Decimal::ZERO);
        assert_eq!(r.preferential_tax, d(3_000));
    }

    #[test]
    fn pref_crossing_fifteen_to_twenty_boundary_for_single() {
        // Single, ORD = $400k, PREF = $200k → TI = $600k.
        // PREF stack [400k, 600k].
        // 0% band: 0 (start > 48,350).
        // 15% band: [400k, 533,400] = 133,400 at 15% = 20,010.
        // 20% band: [533,400, 600k] = 66,600 at 20% = 13,320.
        // Pref tax = 33,330.
        let r = compute(QdcgtwInput {
            taxable_income: d(600_000),
            net_long_term_capital_gain: d(200_000),
            qualified_dividends: Decimal::ZERO,
            status: FilingStatus::Single,
        });
        assert_eq!(r.amount_at_0_pct, Decimal::ZERO);
        assert_eq!(r.amount_at_15_pct, dc("133400"));
        assert_eq!(r.amount_at_20_pct, dc("66600"));
        assert_eq!(r.preferential_tax, dc("33330"));
    }

    #[test]
    fn mfj_thresholds_double_singles_for_zero_band() {
        // MFJ, all LTCG, TI = $80k. MFJ 0% top = $96,700, so all in 0%.
        let r = compute(QdcgtwInput {
            taxable_income: d(80_000),
            net_long_term_capital_gain: d(80_000),
            qualified_dividends: Decimal::ZERO,
            status: FilingStatus::Mfj,
        });
        assert_eq!(r.amount_at_0_pct, d(80_000));
        assert_eq!(r.preferential_tax, Decimal::ZERO);
    }

    #[test]
    fn qualified_dividends_get_preferential_rate() {
        // Single, $40k qualified divs, $20k other income → TI = $60k.
        // Identical math to LTCG-straddle case above.
        let r = compute(QdcgtwInput {
            taxable_income: d(60_000),
            net_long_term_capital_gain: Decimal::ZERO,
            qualified_dividends: d(40_000),
            status: FilingStatus::Single,
        });
        assert_eq!(r.preferential_income, d(40_000));
        assert_eq!(r.amount_at_0_pct, dc("28350"));
        assert_eq!(r.amount_at_15_pct, dc("11650"));
        assert_eq!(r.preferential_tax, dc("1747.50"));
    }

    #[test]
    fn pref_capped_at_taxable_income() {
        // PREF claims $50k but TI is only $30k — pref is capped at TI.
        let r = compute(QdcgtwInput {
            taxable_income: d(30_000),
            net_long_term_capital_gain: d(50_000),
            qualified_dividends: Decimal::ZERO,
            status: FilingStatus::Single,
        });
        assert_eq!(r.preferential_income, d(30_000));
        assert_eq!(r.ordinary_income, Decimal::ZERO);
    }

    #[test]
    fn zero_pref_falls_back_to_ordinary_tax() {
        // No LTCG, no qualified divs → result should equal straight
        // ordinary tax computed by brackets.
        let r = compute(QdcgtwInput {
            taxable_income: d(80_000),
            net_long_term_capital_gain: Decimal::ZERO,
            qualified_dividends: Decimal::ZERO,
            status: FilingStatus::Single,
        });
        assert_eq!(r.preferential_income, Decimal::ZERO);
        assert_eq!(r.preferential_tax, Decimal::ZERO);
        assert_eq!(r.total_tax, r.ordinary_tax);
    }

    #[test]
    fn mfs_fifteen_pct_band_is_300k_not_533k() {
        // MFS, ORD = $200k, PREF = $200k → TI = $400k.
        // MFS bands: 0% to $48,350, 15% to $300k, 20% above.
        // PREF stack [200k, 400k].
        // 15% band: [200k, 300k] = 100k at 15% = 15,000.
        // 20% band: [300k, 400k] = 100k at 20% = 20,000.
        let r = compute(QdcgtwInput {
            taxable_income: d(400_000),
            net_long_term_capital_gain: d(200_000),
            qualified_dividends: Decimal::ZERO,
            status: FilingStatus::Mfs,
        });
        assert_eq!(r.amount_at_15_pct, d(100_000));
        assert_eq!(r.amount_at_20_pct, d(100_000));
        assert_eq!(r.preferential_tax, d(35_000));
    }

    #[test]
    fn worksheet_can_never_exceed_straight_ordinary_tax() {
        // Sanity floor: if straight bracket tax on entire TI is lower
        // (it shouldn't be), we pick it.
        let r = compute(QdcgtwInput {
            taxable_income: d(50_000),
            net_long_term_capital_gain: d(30_000),
            qualified_dividends: d(5_000),
            status: FilingStatus::Single,
        });
        let straight = ordinary_income_tax(d(50_000), FilingStatus::Single).round_dp(2);
        assert!(r.total_tax <= straight);
    }
}
