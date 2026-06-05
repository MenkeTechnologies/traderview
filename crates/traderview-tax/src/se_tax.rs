//! Self-employment tax (Schedule SE) for 2025.
//!
//! - Net SE earnings × 92.35% = "SE base" (the 7.65% employer-side
//!   equivalent is excluded — the taxpayer doesn't pay SE tax on the
//!   portion they could've "paid" as employer FICA).
//! - SS portion: 12.4% on SE base, capped at the 2025 SS wage base.
//!   The cap accounts for W-2 wages already taxed for SS (see
//!   `w2_ss_wages` arg below).
//! - Medicare portion: 2.9% on the full SE base (no cap).
//! - Additional Medicare 0.9% (HI surtax) on SE base + W-2 Medicare
//!   wages above the filing-status threshold. Tracked separately on
//!   Form 8959, not on Schedule SE.
//!
//! Half of the SE tax is deductible as an adjustment to income.
//!
//! Sources:
//!   * 2025 SS wage base: SSA Press Release Oct 2024
//!     <https://www.ssa.gov/oact/cola/cbb.html> — $176,100.
//!   * IRC §§ 1401(a), 1401(b), 1402(a)(12), 164(f).
//!   * Form 8959 (additional Medicare) thresholds: $200k single /
//!     $250k MFJ / $125k MFS — set by IRC § 3101(b)(2).

use rust_decimal::Decimal;
use crate::engine::FilingStatus;

/// 2025 Social Security wage base. SSA announced 2024-10-10.
pub const SS_WAGE_BASE_2025: i64 = 176_100;

/// 92.35% — the inverse of (1 + 7.65%) rounded to 4 decimals. Excludes
/// the employer-equivalent FICA share from the SE base.
fn se_base(net_se_earnings: Decimal) -> Decimal {
    if net_se_earnings <= Decimal::ZERO {
        return Decimal::ZERO;
    }
    let factor = "0.9235".parse::<Decimal>().unwrap();
    net_se_earnings * factor
}

/// One-line output of the SE tax computation.
#[derive(Debug, Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
pub struct SeResult {
    pub se_base: Decimal,
    pub ss_tax: Decimal,
    pub medicare_tax: Decimal,
    pub additional_medicare_tax: Decimal,
    pub total: Decimal,
    /// Half of (ss_tax + medicare_tax) — flows to Schedule 1 line 15 as
    /// an above-the-line adjustment. Does NOT include the additional
    /// Medicare (Form 8959 surtax — not deductible).
    pub above_line_deduction: Decimal,
}

/// Compute SE tax + the deductible half. `w2_ss_wages` is the user's
/// W-2 Box 3 Social Security wages — needed to cap the SS portion
/// jointly across W-2 + SE.
pub fn compute(
    net_se_earnings: Decimal,
    w2_ss_wages: Decimal,
    w2_medicare_wages: Decimal,
    status: FilingStatus,
) -> SeResult {
    if net_se_earnings <= Decimal::ZERO {
        return SeResult::default();
    }
    let base = se_base(net_se_earnings);

    // SS portion — 12.4% of the lesser of base and (cap - w2_ss_wages).
    let cap = Decimal::from(SS_WAGE_BASE_2025);
    let remaining_cap = (cap - w2_ss_wages).max(Decimal::ZERO);
    let ss_taxable = base.min(remaining_cap);
    let ss_rate = "0.124".parse::<Decimal>().unwrap();
    let ss_tax = (ss_taxable * ss_rate).round_dp(2);

    // Medicare portion — 2.9% on the full SE base, no cap.
    let med_rate = "0.029".parse::<Decimal>().unwrap();
    let medicare_tax = (base * med_rate).round_dp(2);

    // Additional Medicare 0.9% on SE base + W-2 Medicare wages above
    // the filing-status threshold. Caller passes W-2 Medicare wages
    // (Box 5) — the threshold applies to the combined total.
    let threshold = match status {
        FilingStatus::Single | FilingStatus::Hoh => Decimal::from(200_000),
        FilingStatus::Mfj                        => Decimal::from(250_000),
        FilingStatus::Mfs                        => Decimal::from(125_000),
    };
    let combined = base + w2_medicare_wages;
    let over = (combined - threshold).max(Decimal::ZERO);
    let add_med_rate = "0.009".parse::<Decimal>().unwrap();
    let additional_medicare_tax = (over * add_med_rate).round_dp(2);

    let total = ss_tax + medicare_tax + additional_medicare_tax;
    let above_line_deduction = ((ss_tax + medicare_tax) / Decimal::from(2)).round_dp(2);

    SeResult {
        se_base: base.round_dp(2),
        ss_tax,
        medicare_tax,
        additional_medicare_tax,
        total,
        above_line_deduction,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_se_yields_zero_tax() {
        let r = compute(Decimal::ZERO, Decimal::ZERO, Decimal::ZERO, FilingStatus::Single);
        assert_eq!(r.total, Decimal::ZERO);
        assert_eq!(r.above_line_deduction, Decimal::ZERO);
    }

    #[test]
    fn modest_self_employment_no_w2() {
        // $50,000 net SE, no W-2.
        // base = 50000 * 0.9235 = 46175
        // SS tax = 46175 * 0.124 = 5725.70
        // Medicare = 46175 * 0.029 = 1339.08 (1339.0775 → 1339.08)
        // Total = 7064.78
        // Half (above-line) = 3532.39
        let r = compute(
            Decimal::from(50_000),
            Decimal::ZERO,
            Decimal::ZERO,
            FilingStatus::Single,
        );
        assert_eq!(r.se_base, "46175.00".parse::<Decimal>().unwrap());
        assert_eq!(r.ss_tax,  "5725.70".parse::<Decimal>().unwrap());
        assert_eq!(r.medicare_tax, "1339.08".parse::<Decimal>().unwrap());
        assert_eq!(r.additional_medicare_tax, Decimal::ZERO);
        assert_eq!(r.total, "7064.78".parse::<Decimal>().unwrap());
        assert_eq!(r.above_line_deduction, "3532.39".parse::<Decimal>().unwrap());
    }

    #[test]
    fn ss_cap_reduces_ss_tax_when_w2_already_at_cap() {
        // W-2 Box 3 already at 2025 cap → SE pays 0 SS, only Medicare.
        let r = compute(
            Decimal::from(50_000),
            Decimal::from(SS_WAGE_BASE_2025), // already at cap
            Decimal::from(SS_WAGE_BASE_2025),
            FilingStatus::Single,
        );
        assert_eq!(r.ss_tax, Decimal::ZERO);
        assert!(r.medicare_tax > Decimal::ZERO);
    }

    #[test]
    fn additional_medicare_kicks_in_above_threshold() {
        // Single, $250k net SE → base = $230,875. Threshold = $200k.
        // Over = 30,875. Additional Medicare = 30,875 * 0.009 = 277.875 → 277.88
        let r = compute(
            Decimal::from(250_000),
            Decimal::ZERO,
            Decimal::ZERO,
            FilingStatus::Single,
        );
        assert_eq!(r.additional_medicare_tax, "277.88".parse::<Decimal>().unwrap());
    }

    #[test]
    fn mfj_threshold_is_higher() {
        // Same $250k net SE, MFJ threshold $250k. base $230,875 alone
        // doesn't cross MFJ threshold → no additional Medicare.
        let r = compute(
            Decimal::from(250_000),
            Decimal::ZERO,
            Decimal::ZERO,
            FilingStatus::Mfj,
        );
        assert_eq!(r.additional_medicare_tax, Decimal::ZERO);
    }

    #[test]
    fn negative_se_earnings_short_circuit() {
        // Net loss on Schedule C — no SE tax owed.
        let r = compute(
            Decimal::from(-10_000),
            Decimal::ZERO,
            Decimal::ZERO,
            FilingStatus::Single,
        );
        assert_eq!(r.total, Decimal::ZERO);
    }
}
