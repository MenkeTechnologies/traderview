//! Schedule SE — self-employment tax computation.
//!
//! SE tax = Social Security (12.4% on net SE earnings up to the wage base)
//! + Medicare (2.9% uncapped) + Additional Medicare (0.9% on combined wages
//! + SE earnings over the threshold for the filing status).
//!
//! The 92.35% multiplier on net SE earnings comes straight from IRS
//! Schedule SE instructions — accounts for the employer-side FICA the
//! self-employed taxpayer is "deducting" from the base. Half of SE tax
//! (Social Security + Medicare portions, NOT the additional Medicare
//! surtax) is an above-the-line deduction on Form 1040, Schedule 1, line 15.
//!
//! All inputs and outputs are `Decimal` — no floats anywhere near a tax form.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilingStatus {
    Single,
    MarriedJoint,
    MarriedSeparate,
    HeadOfHousehold,
}

impl FilingStatus {
    /// Additional Medicare 0.9% threshold (over which the surtax applies).
    /// Unchanged since 2013 (not indexed for inflation — IRS quirk).
    pub fn additional_medicare_threshold(self) -> Decimal {
        match self {
            FilingStatus::Single
            | FilingStatus::HeadOfHousehold
            | FilingStatus::MarriedSeparate => Decimal::from(200_000),
            FilingStatus::MarriedJoint => Decimal::from(250_000),
        }
    }
}

/// IRS-published constants per tax year. Add new rows as IRS publishes
/// next year's SS wage base in November.
#[derive(Debug, Clone, Copy)]
pub struct YearTable {
    pub year: u16,
    /// Social Security wage base — the cap on the 12.4% SS portion.
    pub ss_wage_base: i64,
}

const YEAR_TABLE: &[YearTable] = &[
    YearTable { year: 2022, ss_wage_base: 147_000 },
    YearTable { year: 2023, ss_wage_base: 160_200 },
    YearTable { year: 2024, ss_wage_base: 168_600 },
    YearTable { year: 2025, ss_wage_base: 176_100 },
    YearTable { year: 2026, ss_wage_base: 181_800 },
];

pub fn lookup(year: u16) -> Option<YearTable> {
    YEAR_TABLE.iter().find(|y| y.year == year).copied()
}

#[derive(Debug, Clone)]
pub struct ScheduleSeInput {
    /// Net profit from Schedule C (line 31). Negative = no SE tax.
    pub net_profit_schedule_c: Decimal,
    /// W-2 wages already subject to Social Security in the same tax year.
    /// SS portion only applies up to (wage_base - w2_ss_wages).
    pub w2_ss_wages: Decimal,
    pub filing_status: FilingStatus,
    pub year: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScheduleSeReport {
    pub year: u16,
    /// 92.35% × net profit. Schedule SE line 4a.
    pub net_se_earnings: Decimal,
    /// 12.4% × min(net_se_earnings, ss_wage_base − w2_ss_wages).
    pub social_security_tax: Decimal,
    /// 2.9% × net_se_earnings.
    pub medicare_tax: Decimal,
    /// 0.9% × earnings over additional-medicare threshold.
    pub additional_medicare_tax: Decimal,
    /// SS + Medicare + additional. Goes on Schedule 2 line 4.
    pub total_se_tax: Decimal,
    /// Half of (SS + Medicare). Above-the-line deduction. Note: the
    /// additional-medicare 0.9% surtax is NOT deductible.
    pub deductible_half: Decimal,
}

const NET_EARNINGS_MULTIPLIER: &str = "0.9235";
const SS_RATE: &str = "0.124";
const MEDICARE_RATE: &str = "0.029";
const ADDITIONAL_MEDICARE_RATE: &str = "0.009";

pub fn compute(input: &ScheduleSeInput) -> ScheduleSeReport {
    let mut out = ScheduleSeReport { year: input.year, ..Default::default() };

    // Negative or trivially small net profit → no SE tax. IRS threshold is $400.
    if input.net_profit_schedule_c < Decimal::from(400) {
        return out;
    }

    let mul = Decimal::from_str(NET_EARNINGS_MULTIPLIER).unwrap();
    let net_se = input.net_profit_schedule_c * mul;
    out.net_se_earnings = net_se;

    let table = lookup(input.year).unwrap_or(YearTable {
        year: input.year,
        // Fallback: extrapolate from latest known base. Better wrong-by-1%
        // than panic in production.
        ss_wage_base: 181_800,
    });
    let wage_base = Decimal::from(table.ss_wage_base);
    let ss_room = (wage_base - input.w2_ss_wages).max(Decimal::ZERO);
    let ss_taxable = net_se.min(ss_room);

    out.social_security_tax =
        ss_taxable * Decimal::from_str(SS_RATE).unwrap();
    out.medicare_tax =
        net_se * Decimal::from_str(MEDICARE_RATE).unwrap();

    // Additional Medicare uses combined wages + SE earnings; threshold is
    // not the SS wage base but the filing-status threshold.
    let combined = input.w2_ss_wages + net_se;
    let addl_threshold = input.filing_status.additional_medicare_threshold();
    if combined > addl_threshold {
        let excess = combined - addl_threshold;
        out.additional_medicare_tax =
            excess * Decimal::from_str(ADDITIONAL_MEDICARE_RATE).unwrap();
    }

    out.total_se_tax = out.social_security_tax
        + out.medicare_tax
        + out.additional_medicare_tax;

    // Half-deduction excludes the additional Medicare surtax (IRS quirk).
    out.deductible_half =
        (out.social_security_tax + out.medicare_tax) / Decimal::from(2);

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> Decimal { Decimal::from_str(s).unwrap() }

    #[test]
    fn no_se_tax_below_400_threshold() {
        let r = compute(&ScheduleSeInput {
            net_profit_schedule_c: d("399.99"),
            w2_ss_wages: Decimal::ZERO,
            filing_status: FilingStatus::Single,
            year: 2024,
        });
        assert_eq!(r.total_se_tax, Decimal::ZERO);
        assert_eq!(r.deductible_half, Decimal::ZERO);
    }

    #[test]
    fn modest_profit_single_no_w2() {
        // $50,000 net profit, no W-2.
        // net_se = 50000 * 0.9235 = 46175
        // SS = 46175 * 0.124 = 5725.70
        // Medicare = 46175 * 0.029 = 1339.075
        // Additional Medicare = 0 (under $200k)
        // Total = 7064.775
        // Half deduction = (5725.70 + 1339.075) / 2 = 3532.3875
        let r = compute(&ScheduleSeInput {
            net_profit_schedule_c: d("50000"),
            w2_ss_wages: Decimal::ZERO,
            filing_status: FilingStatus::Single,
            year: 2024,
        });
        assert_eq!(r.net_se_earnings, d("46175.0000"));
        assert_eq!(r.social_security_tax, d("5725.700000"));
        assert_eq!(r.medicare_tax, d("1339.0750000"));
        assert_eq!(r.additional_medicare_tax, Decimal::ZERO);
        assert_eq!(r.total_se_tax, d("7064.7750000"));
        assert_eq!(r.deductible_half, d("3532.38750000"));
    }

    #[test]
    fn ss_wage_base_caps_the_12_4_portion() {
        // 2024 wage base = $168,600. With $500k net profit, only the first
        // $168,600 of *net SE earnings* gets the 12.4% SS rate.
        let r = compute(&ScheduleSeInput {
            net_profit_schedule_c: d("500000"),
            w2_ss_wages: Decimal::ZERO,
            filing_status: FilingStatus::Single,
            year: 2024,
        });
        // SS portion capped at 168600 * 0.124 = 20906.40 regardless of net.
        // (Net SE earnings = 461,750 — well over the cap.)
        assert_eq!(r.social_security_tax, d("20906.4000"));
        // Medicare is uncapped: 461750 * 0.029 = 13390.75
        assert_eq!(r.medicare_tax, d("13390.7500"));
    }

    #[test]
    fn w2_wages_reduce_remaining_ss_room() {
        // $100k W-2 already collected SS. Wage base $168,600 leaves $68,600
        // room for SE earnings to be SS-taxed.
        let r = compute(&ScheduleSeInput {
            net_profit_schedule_c: d("200000"),  // net_se = 184,700
            w2_ss_wages: d("100000"),
            filing_status: FilingStatus::Single,
            year: 2024,
        });
        // SS taxable = min(184700, 68600) = 68600 → tax 68600 × 0.124 = 8506.40
        assert_eq!(r.social_security_tax, d("8506.4000"));
    }

    #[test]
    fn w2_already_at_wage_base_yields_no_ss_se_tax() {
        let r = compute(&ScheduleSeInput {
            net_profit_schedule_c: d("50000"),
            w2_ss_wages: d("200000"),   // already over cap
            filing_status: FilingStatus::Single,
            year: 2024,
        });
        assert_eq!(r.social_security_tax, Decimal::ZERO);
        // Medicare still applies (uncapped).
        assert!(r.medicare_tax > Decimal::ZERO);
    }

    #[test]
    fn additional_medicare_kicks_in_over_200k_single() {
        // $300k net profit, filing single. Threshold $200k.
        // net_se = 277,050. Excess over 200k = 77,050. × 0.009 = 693.45
        let r = compute(&ScheduleSeInput {
            net_profit_schedule_c: d("300000"),
            w2_ss_wages: Decimal::ZERO,
            filing_status: FilingStatus::Single,
            year: 2024,
        });
        assert_eq!(r.additional_medicare_tax, d("693.450000"));
    }

    #[test]
    fn additional_medicare_threshold_for_married_joint_is_250k() {
        // MFJ threshold is $250k, not $200k.
        let r = compute(&ScheduleSeInput {
            net_profit_schedule_c: d("250000"),  // net_se = 230,875
            w2_ss_wages: Decimal::ZERO,
            filing_status: FilingStatus::MarriedJoint,
            year: 2024,
        });
        // 230,875 < 250,000 → no additional Medicare.
        assert_eq!(r.additional_medicare_tax, Decimal::ZERO);
    }

    #[test]
    fn deductible_half_excludes_additional_medicare_surtax() {
        let r = compute(&ScheduleSeInput {
            net_profit_schedule_c: d("400000"),
            w2_ss_wages: Decimal::ZERO,
            filing_status: FilingStatus::Single,
            year: 2024,
        });
        // Half of (SS + Medicare) only — the 0.9% surtax is NOT deductible.
        let expected = (r.social_security_tax + r.medicare_tax) / Decimal::from(2);
        assert_eq!(r.deductible_half, expected);
        assert!(r.additional_medicare_tax > Decimal::ZERO,
            "test setup: should have triggered surtax");
        assert!(r.deductible_half < r.total_se_tax / Decimal::from(2),
            "deductible half must be less than half of total when surtax applies");
    }

    #[test]
    fn historical_year_table_is_populated() {
        // 2022, 2023, 2024, 2025 must all resolve — they're load-bearing
        // for prior-year amended returns.
        for year in [2022, 2023, 2024, 2025, 2026] {
            assert!(lookup(year).is_some(), "missing wage base for {year}");
        }
    }

    #[test]
    fn unknown_year_falls_back_without_panic() {
        // Future year (no table entry) should NOT panic — extrapolate
        // from latest known base so the calculator keeps working.
        let r = compute(&ScheduleSeInput {
            net_profit_schedule_c: d("100000"),
            w2_ss_wages: Decimal::ZERO,
            filing_status: FilingStatus::Single,
            year: 2099,
        });
        assert!(r.total_se_tax > Decimal::ZERO);
    }
}
