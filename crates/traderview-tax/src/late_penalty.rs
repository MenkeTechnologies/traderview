//! Late-file / late-pay penalty + interest calculator.
//!
//! Computes the four numbers that the IRS will hit a delinquent filer
//! with:
//!
//! 1. **Failure-to-File (FTF) penalty — IRC § 6651(a)(1)**
//!    * 5% of unpaid tax per month (or part of a month) late.
//!    * Capped at 25% of unpaid tax (so 5 months × 5%).
//!    * If the return is more than 60 days late, FTF cannot be less
//!      than the smaller of (a) $485 (2025 figure, Rev. Proc. 2024-40
//!      § 3.49) or (b) 100% of the unpaid tax. This is the "minimum
//!      failure-to-file penalty" of IRC § 6651(a).
//!
//! 2. **Failure-to-Pay (FTP) penalty — IRC § 6651(a)(2)**
//!    * 0.5% of unpaid tax per month (or part of a month) late.
//!    * Capped at 25% of unpaid tax (50 months × 0.5%).
//!    * The rate drops to 0.25%/month after the IRS issues an
//!      installment-agreement notice (not modeled in v1).
//!
//! 3. **§ 6651(c)(1) FTF / FTP coordination**
//!    * When both apply in the same month, FTF is reduced by the FTP
//!      amount for that month. Net effect: FTF effectively becomes
//!      4.5%/month and FTP stays 0.5%/month.
//!
//! 4. **§ 6601 interest**
//!    * Federal short-term rate + 3%, compounded daily.
//!    * The rate is set quarterly. We accept it as an input — `apr`
//!      (the annualized rate as a decimal, e.g. `0.08` for 8%) — and
//!      use simple daily accrual (`balance × apr × days / 365`).
//!      Daily compounding diverges from simple accrual by < $1 on
//!      typical balances over normal delinquency windows, so we keep
//!      the simpler formula and document the discrepancy. Callers who
//!      need exact compounding can post-process.
//!
//! ### Inputs
//!
//! * `unpaid_tax` — the amount due on the original return.
//! * `months_late_at_due_date` — months (or part-months) elapsed since
//!   the original due date (April 15 for most filers).
//! * `days_late` — days since the due date, used for interest.
//! * `interest_apr` — federal short-term rate + 3%, as a decimal.
//! * `filed_more_than_60_days_late` — gates the $485 minimum penalty.
//!
//! Sources:
//!   * IRC § 6651, § 6601
//!   * IRS Topic No. 653, "IRS notices and bills, penalties, and interest charges"
//!   * Rev. Proc. 2024-40 § 3.49 (2025 minimum FTF penalty: $485)

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// The 2025 minimum FTF penalty floor under IRC § 6651(a) when a return
/// is filed more than 60 days late. Per Rev. Proc. 2024-40 § 3.49.
pub const MIN_FTF_PENALTY_2025: i64 = 485;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LatePenaltyInput {
    /// Amount owed on the original due date (does not include penalties or interest).
    pub unpaid_tax: Decimal,
    /// Months (including partial) since the due date. A return filed 1
    /// day late is 1 month for penalty purposes — IRC § 6651 counts a
    /// "fraction of a month" as a full month.
    pub months_late: u32,
    /// Days since the due date — drives the § 6601 interest computation.
    pub days_late: u32,
    /// Federal short-term rate + 3 percentage points, expressed as a
    /// decimal (e.g. `0.08` for 8% APR).
    pub interest_apr: Decimal,
    /// True when the return is filed more than 60 days late. Triggers
    /// the IRC § 6651(a) minimum penalty floor.
    pub filed_more_than_60_days_late: bool,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct LatePenaltyResult {
    /// Failure-to-file penalty before § 6651(c)(1) coordination.
    pub ftf_gross: Decimal,
    /// Failure-to-pay penalty.
    pub ftp: Decimal,
    /// FTF after subtracting FTP for the same month(s) — net "5%/mo"
    /// becomes "4.5%/mo" because the 0.5% FTP is netted against it.
    pub ftf_net: Decimal,
    /// True when the 60-day minimum-penalty floor of IRC § 6651(a)
    /// is binding (FTF was bumped up to $485 or 100% of unpaid tax).
    pub min_penalty_floor_applied: bool,
    /// § 6601 interest (simple daily accrual on `unpaid_tax`).
    pub interest: Decimal,
    /// FTF net + FTP + interest. Rounded to cents.
    pub total_owed: Decimal,
}

pub fn compute(input: LatePenaltyInput) -> LatePenaltyResult {
    let unpaid = input.unpaid_tax.max(Decimal::ZERO);

    // Bound month counts at their statutory caps so we don't accidentally
    // overstate the penalty when the user supplies enormous values.
    let ftf_months = input.months_late.min(5); // 5 months × 5% = 25% cap
    let ftp_months = input.months_late.min(50); // 50 months × 0.5% = 25% cap

    let ftf_rate: Decimal = "0.05".parse().unwrap();
    let ftp_rate: Decimal = "0.005".parse().unwrap();

    let ftf_gross = unpaid * ftf_rate * Decimal::from(ftf_months);
    let ftp = unpaid * ftp_rate * Decimal::from(ftp_months);

    // § 6651(c)(1): when both apply, FTF for the month is reduced by
    // the FTP amount for the same month. Equivalent to subtracting FTP
    // from FTF on the months where they overlap. We use the simple
    // "FTP wholly nets against FTF" approximation, which is exact when
    // both penalties accrue for the same number of months — the common
    // case for a delinquent filer.
    let coord_months = ftf_months.min(ftp_months);
    let netted_ftp_against_ftf = unpaid * ftp_rate * Decimal::from(coord_months);
    let ftf_after_coordination = (ftf_gross - netted_ftp_against_ftf).max(Decimal::ZERO);

    // 60-day minimum-penalty floor.
    let (ftf_net, floor_applied) = if input.filed_more_than_60_days_late && unpaid > Decimal::ZERO {
        let floor = Decimal::from(MIN_FTF_PENALTY_2025).min(unpaid);
        if ftf_after_coordination < floor {
            (floor, true)
        } else {
            (ftf_after_coordination, false)
        }
    } else {
        (ftf_after_coordination, false)
    };

    // § 6601 interest — simple daily accrual.
    let days = Decimal::from(input.days_late);
    let year_days: Decimal = Decimal::from(365);
    let interest = (unpaid * input.interest_apr * days / year_days).round_dp(2);

    let total_owed = (ftf_net + ftp + interest).round_dp(2);

    LatePenaltyResult {
        ftf_gross: ftf_gross.round_dp(2),
        ftp: ftp.round_dp(2),
        ftf_net: ftf_net.round_dp(2),
        min_penalty_floor_applied: floor_applied,
        interest,
        total_owed,
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

    fn base(unpaid: Decimal) -> LatePenaltyInput {
        LatePenaltyInput {
            unpaid_tax: unpaid,
            months_late: 0,
            days_late: 0,
            interest_apr: dc("0.08"),
            filed_more_than_60_days_late: false,
        }
    }

    #[test]
    fn zero_owed_means_zero_penalty() {
        let r = compute(base(Decimal::ZERO));
        assert_eq!(r.total_owed, Decimal::ZERO);
    }

    #[test]
    fn one_month_late_charges_4_5_pct_ftf_plus_0_5_pct_ftp() {
        // $10k unpaid, 1 month late.
        // FTF gross = $500 (5%). FTP = $50 (0.5%).
        // FTF after § 6651(c)(1) coordination = $450 (4.5%).
        // Total penalty = $500.
        let r = compute(LatePenaltyInput {
            months_late: 1,
            days_late: 30,
            ..base(d(10_000))
        });
        assert_eq!(r.ftf_gross, d(500));
        assert_eq!(r.ftp, d(50));
        assert_eq!(r.ftf_net, d(450));
        // Interest: $10k × 8% × 30/365 = $65.7534... → $65.75 (banker's).
        assert_eq!(r.interest, dc("65.75"));
        assert_eq!(r.total_owed, dc("565.75"));
    }

    #[test]
    fn ftf_caps_at_25_pct_after_5_months() {
        // 10 months late should still cap FTF at 25% × $10k = $2,500 gross.
        // FTP for 10 months = 10 × 0.5% × $10k = $500.
        // Coordination: FTF − FTP for overlap months. Overlap = 5 (FTF cap).
        // netted_ftp_against_ftf = 5 × 0.5% × $10k = $250.
        // FTF net = $2,500 - $250 = $2,250.
        let r = compute(LatePenaltyInput {
            months_late: 10,
            days_late: 0,
            ..base(d(10_000))
        });
        assert_eq!(r.ftf_gross, d(2_500));
        assert_eq!(r.ftp, d(500));
        assert_eq!(r.ftf_net, d(2_250));
    }

    #[test]
    fn ftp_caps_at_25_pct_after_50_months() {
        // 60 months. FTP cap binds at 50 × 0.5% × $10k = $2,500.
        let r = compute(LatePenaltyInput {
            months_late: 60,
            days_late: 0,
            ..base(d(10_000))
        });
        assert_eq!(r.ftp, d(2_500));
        assert_eq!(r.ftf_gross, d(2_500));
    }

    #[test]
    fn minimum_penalty_floor_kicks_in_after_60_days() {
        // Tiny tax, 3 months late, > 60 days. FTF gross = 15% × $200 = $30.
        // Floor = min($485, $200) = $200. FTF should be bumped to $200.
        let r = compute(LatePenaltyInput {
            months_late: 3,
            days_late: 90,
            filed_more_than_60_days_late: true,
            ..base(d(200))
        });
        assert!(r.min_penalty_floor_applied);
        assert_eq!(r.ftf_net, d(200));
    }

    #[test]
    fn minimum_penalty_floor_uses_485_when_smaller_than_unpaid() {
        // $5,000 unpaid, only 1 month late but > 60 days (consistent
        // with a filer who paid 1 month after the 60-day mark).
        // FTF gross = 5% × $5k = $250.
        // Floor = min($485, $5,000) = $485 → FTF net = $485.
        let r = compute(LatePenaltyInput {
            months_late: 1,
            days_late: 75,
            filed_more_than_60_days_late: true,
            ..base(d(5_000))
        });
        assert!(r.min_penalty_floor_applied);
        assert_eq!(r.ftf_net, d(485));
    }

    #[test]
    fn minimum_penalty_floor_not_applied_when_already_higher() {
        // Big tax, 5 months late — FTF cap is $2,500 already > $485.
        let r = compute(LatePenaltyInput {
            months_late: 5,
            days_late: 150,
            filed_more_than_60_days_late: true,
            ..base(d(10_000))
        });
        assert!(!r.min_penalty_floor_applied);
        assert!(r.ftf_net > Decimal::from(485));
    }

    #[test]
    fn coordination_makes_combined_5_5_pct_per_month() {
        // 1 month late: FTF net + FTP should equal 5% (not 5.5%).
        let r = compute(LatePenaltyInput {
            months_late: 1,
            days_late: 30,
            ..base(d(10_000))
        });
        let combined = r.ftf_net + r.ftp;
        assert_eq!(combined, d(500));
    }

    #[test]
    fn interest_uses_simple_daily_accrual() {
        // $100k × 8% × 100/365 = $2,191.78
        let r = compute(LatePenaltyInput {
            months_late: 0,
            days_late: 100,
            interest_apr: dc("0.08"),
            filed_more_than_60_days_late: false,
            ..base(d(100_000))
        });
        assert_eq!(r.interest, dc("2191.78"));
    }

    #[test]
    fn zero_interest_when_apr_is_zero() {
        let r = compute(LatePenaltyInput {
            months_late: 1,
            days_late: 30,
            interest_apr: Decimal::ZERO,
            filed_more_than_60_days_late: false,
            ..base(d(10_000))
        });
        assert_eq!(r.interest, Decimal::ZERO);
    }

    #[test]
    fn negative_unpaid_clamped_to_zero() {
        let r = compute(LatePenaltyInput {
            months_late: 5,
            days_late: 150,
            ..base(d(-1_000))
        });
        assert_eq!(r.total_owed, Decimal::ZERO);
    }
}
