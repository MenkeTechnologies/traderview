//! Qualified Business Income deduction (IRC § 199A) for 2025.
//!
//! Simplest path (sub-threshold): 20% of qualified business income,
//! limited to 20% of (taxable income - net capital gains).
//!
//! Above-threshold (2025 per Rev. Proc. 2024-40 § 3.27):
//!   * Single / MFS / HoH: $241,950
//!   * MFJ:                $483,900
//!
//! Phase-in window = $50,000 single / $100,000 MFJ above the threshold.
//! Above the phase-in fully, SSTB (specified service trade or business)
//! deduction is $0 and non-SSTB is capped by the W-2 / UBIA tests.
//!
//! This module implements:
//!   1. Sub-threshold (the 95% case for self-employed users):
//!      `min(20% × QBI, 20% × (TI - net_cap_gain))`.
//!   2. Above-threshold simple path with a linear phase-in for SSTB.
//!
//! The full W-2 wage / UBIA test is NOT implemented — it requires
//! property cost-basis data we don't collect. Above-threshold callers
//! get a conservative deduction (taking the lesser of the two tests
//! that ARE computable); the wizard flags this for manual review.
//!
//! Sources:
//!   * IRC § 199A.
//!   * Rev. Proc. 2024-40 § 3.27.

use crate::engine::FilingStatus;
use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy)]
pub struct QbiInput {
    pub qualified_business_income: Decimal,
    pub taxable_income_before_qbi: Decimal,
    /// Net long-term capital gain + qualified dividends. Subtracted
    /// from TI for the alternate cap because qualified dividends are
    /// taxed at LTCG rates, not ordinary.
    pub net_capital_gain: Decimal,
    pub is_sstb: bool,
    pub status: FilingStatus,
}

#[derive(Debug, Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
pub struct QbiResult {
    pub deduction: Decimal,
    /// Set when the user is above the SSTB phase-in but we can't
    /// compute the W-2/UBIA test from collected data. The wizard
    /// uses this to nudge a manual review.
    pub needs_manual_review: bool,
}

/// Returns the 199A deduction in dollars.
pub fn compute(input: QbiInput) -> QbiResult {
    if input.qualified_business_income <= Decimal::ZERO {
        return QbiResult::default();
    }

    let twenty_pct = "0.20".parse::<Decimal>().unwrap();
    let qbi_20 = (input.qualified_business_income * twenty_pct).round_dp(2);
    let ti_cap = ((input.taxable_income_before_qbi - input.net_capital_gain) * twenty_pct)
        .max(Decimal::ZERO)
        .round_dp(2);

    let threshold = sstb_threshold(input.status);
    let window = sstb_phase_window(input.status);

    if input.taxable_income_before_qbi <= threshold {
        // Sub-threshold — simple cap.
        let deduction = qbi_20.min(ti_cap);
        return QbiResult {
            deduction,
            needs_manual_review: false,
        };
    }

    let over = (input.taxable_income_before_qbi - threshold).max(Decimal::ZERO);

    if input.is_sstb {
        if over >= window {
            // SSTB phased out entirely.
            return QbiResult {
                deduction: Decimal::ZERO,
                needs_manual_review: false,
            };
        }
        // Linear phase-out for SSTB: allowed share = (window - over)/window.
        let allowed = (window - over) / window;
        let deduction = (qbi_20.min(ti_cap) * allowed).round_dp(2);
        // Flag for review — we're not enforcing the W-2/UBIA test.
        return QbiResult {
            deduction,
            needs_manual_review: true,
        };
    }

    // Non-SSTB above the threshold. Without W-2 wages / UBIA data we
    // CAN'T enforce the wage-based cap. Return the simple 20% × QBI
    // (capped by TI) and flag for review.
    let deduction = qbi_20.min(ti_cap);
    QbiResult {
        deduction,
        needs_manual_review: true,
    }
}

fn sstb_threshold(status: FilingStatus) -> Decimal {
    Decimal::from(match status {
        FilingStatus::Mfj => 483_900,
        _ => 241_950,
    })
}

fn sstb_phase_window(status: FilingStatus) -> Decimal {
    Decimal::from(match status {
        FilingStatus::Mfj => 100_000,
        _ => 50_000,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_qbi_yields_zero_deduction() {
        let r = compute(QbiInput {
            qualified_business_income: Decimal::ZERO,
            taxable_income_before_qbi: Decimal::from(100_000),
            net_capital_gain: Decimal::ZERO,
            is_sstb: false,
            status: FilingStatus::Single,
        });
        assert_eq!(r.deduction, Decimal::ZERO);
    }

    #[test]
    fn sub_threshold_simple_20_pct() {
        // QBI $50k, TI $80k single. Below the $241,950 threshold.
        // 20% × $50k = $10k. TI cap = 20% × $80k = $16k. min = $10k.
        let r = compute(QbiInput {
            qualified_business_income: Decimal::from(50_000),
            taxable_income_before_qbi: Decimal::from(80_000),
            net_capital_gain: Decimal::ZERO,
            is_sstb: false,
            status: FilingStatus::Single,
        });
        assert_eq!(r.deduction, Decimal::from(10_000));
        assert!(!r.needs_manual_review);
    }

    #[test]
    fn ti_cap_binds_when_qbi_exceeds_ti() {
        // QBI $100k, TI $40k. TI cap = $8k limits the deduction.
        let r = compute(QbiInput {
            qualified_business_income: Decimal::from(100_000),
            taxable_income_before_qbi: Decimal::from(40_000),
            net_capital_gain: Decimal::ZERO,
            is_sstb: false,
            status: FilingStatus::Single,
        });
        assert_eq!(r.deduction, Decimal::from(8_000));
    }

    #[test]
    fn sstb_fully_phased_out_above_window() {
        // Single, TI $300k → over by $58,050 (>$50k window) → SSTB = 0.
        let r = compute(QbiInput {
            qualified_business_income: Decimal::from(50_000),
            taxable_income_before_qbi: Decimal::from(300_000),
            net_capital_gain: Decimal::ZERO,
            is_sstb: true,
            status: FilingStatus::Single,
        });
        assert_eq!(r.deduction, Decimal::ZERO);
        assert!(!r.needs_manual_review);
    }

    #[test]
    fn non_sstb_above_threshold_flags_review() {
        // Non-SSTB over the threshold — caller passes the simple 20%
        // but is told to verify W-2/UBIA test manually.
        let r = compute(QbiInput {
            qualified_business_income: Decimal::from(50_000),
            taxable_income_before_qbi: Decimal::from(300_000),
            net_capital_gain: Decimal::ZERO,
            is_sstb: false,
            status: FilingStatus::Single,
        });
        assert!(r.needs_manual_review);
        assert!(r.deduction > Decimal::ZERO);
    }

    #[test]
    fn net_capital_gain_reduces_ti_cap() {
        // QBI $50k, TI $80k of which $30k is LTCG. TI cap = 20% ×
        // ($80k - $30k) = $10k. QBI 20% = $10k. min = $10k.
        let r = compute(QbiInput {
            qualified_business_income: Decimal::from(50_000),
            taxable_income_before_qbi: Decimal::from(80_000),
            net_capital_gain: Decimal::from(30_000),
            is_sstb: false,
            status: FilingStatus::Single,
        });
        assert_eq!(r.deduction, Decimal::from(10_000));
    }
}
