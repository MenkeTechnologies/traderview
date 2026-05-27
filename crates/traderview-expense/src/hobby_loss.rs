//! Hobby-loss safe harbor test (IRC §183).
//!
//! For a "for-profit" business classification, the IRS presumes profit
//! motive if the activity shows a profit in 3 of the most recent 5
//! consecutive tax years (2 of 7 for horses). Failing the safe harbor
//! doesn't automatically lose business status — the 9-factor facts-and-
//! circumstances test still applies — but it shifts the burden of proof
//! onto the taxpayer.
//!
//! For traders, this is load-bearing: an unprofitable trader who can't
//! show 3-of-5 risks losing Schedule C status, which forfeits §179,
//! home-office, SE-tax-half deduction, etc.
//!
//! Pure compute. Takes a list of (year, net_profit) pairs and returns
//! the safe-harbor verdict.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YearProfit {
    pub year: i32,
    pub net_profit: Decimal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SafeHarborVerdict {
    /// Passed — IRS presumes profit motive.
    Passed,
    /// Failed — business status now requires the 9-factor test.
    Failed,
    /// Not enough years of data to evaluate.
    Inconclusive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafeHarborReport {
    pub verdict: SafeHarborVerdict,
    /// Profitable years in the lookback window.
    pub profitable_years: u32,
    /// Total years considered.
    pub years_considered: u32,
    /// Years required for safe harbor (3 standard, 2 for horse activities).
    pub required: u32,
    /// Lookback-window size (5 standard, 7 for horses).
    pub window: u32,
    pub note: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivityKind {
    /// Standard 3-of-5.
    Default,
    /// Horse breeding / racing — 2-of-7 special rule.
    HorseActivities,
}

pub fn evaluate(history: &[YearProfit], current_year: i32, kind: ActivityKind)
    -> SafeHarborReport
{
    let (required, window) = match kind {
        ActivityKind::Default        => (3u32, 5u32),
        ActivityKind::HorseActivities => (2u32, 7u32),
    };
    // Lookback window: most-recent `window` years up to (but not
    // including) current_year, since the current year isn't done yet.
    let earliest = current_year - (window as i32);
    let recent: Vec<&YearProfit> = history.iter()
        .filter(|y| y.year >= earliest && y.year < current_year)
        .collect();
    let years_considered = recent.len() as u32;
    let profitable_years = recent.iter()
        .filter(|y| y.net_profit > Decimal::ZERO)
        .count() as u32;

    let verdict = if years_considered < window {
        SafeHarborVerdict::Inconclusive
    } else if profitable_years >= required {
        SafeHarborVerdict::Passed
    } else {
        SafeHarborVerdict::Failed
    };

    let note = match verdict {
        SafeHarborVerdict::Passed => format!(
            "passed — {profitable_years}/{years_considered} profitable years \
             ≥ {required} required (IRC §183 presumption)"),
        SafeHarborVerdict::Failed => format!(
            "failed — {profitable_years}/{years_considered} profitable years \
             < {required} required; business status falls to 9-factor test"),
        SafeHarborVerdict::Inconclusive => format!(
            "need {window} full years of data; only {years_considered} \
             prior-year records on file"),
    };

    SafeHarborReport {
        verdict, profitable_years, years_considered, required, window, note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn d(s: &str) -> Decimal { Decimal::from_str(s).unwrap() }
    fn yp(year: i32, profit: &str) -> YearProfit {
        YearProfit { year, net_profit: d(profit) }
    }

    #[test]
    fn three_of_five_profitable_passes() {
        let h = vec![
            yp(2021, "10000"), yp(2022, "-5000"),
            yp(2023, "15000"), yp(2024, "-3000"),
            yp(2025, "20000"),
        ];
        let r = evaluate(&h, 2026, ActivityKind::Default);
        assert_eq!(r.verdict, SafeHarborVerdict::Passed);
        assert_eq!(r.profitable_years, 3);
    }

    #[test]
    fn two_of_five_fails() {
        let h = vec![
            yp(2021, "10000"), yp(2022, "-5000"),
            yp(2023, "-15000"), yp(2024, "-3000"),
            yp(2025, "20000"),
        ];
        let r = evaluate(&h, 2026, ActivityKind::Default);
        assert_eq!(r.verdict, SafeHarborVerdict::Failed);
        assert_eq!(r.profitable_years, 2);
    }

    #[test]
    fn three_of_three_inconclusive_due_to_window_size() {
        let h = vec![
            yp(2023, "10000"), yp(2024, "10000"), yp(2025, "10000"),
        ];
        let r = evaluate(&h, 2026, ActivityKind::Default);
        assert_eq!(r.verdict, SafeHarborVerdict::Inconclusive);
        // Only 3 years of data; need 5 for the safe-harbor test.
    }

    #[test]
    fn zero_profit_year_does_not_count_as_profitable() {
        let h = vec![
            yp(2021, "0"),   yp(2022, "0"),
            yp(2023, "0"),   yp(2024, "-100"),
            yp(2025, "1"),
        ];
        let r = evaluate(&h, 2026, ActivityKind::Default);
        assert_eq!(r.profitable_years, 1, "exactly zero is not > 0");
        assert_eq!(r.verdict, SafeHarborVerdict::Failed);
    }

    #[test]
    fn horse_activities_use_2_of_7() {
        let h = vec![
            yp(2019, "5000"), yp(2020, "-1000"),
            yp(2021, "-2000"), yp(2022, "5000"),
            yp(2023, "-1000"), yp(2024, "-1000"),
            yp(2025, "-3000"),
        ];
        let r = evaluate(&h, 2026, ActivityKind::HorseActivities);
        assert_eq!(r.verdict, SafeHarborVerdict::Passed);
        assert_eq!(r.profitable_years, 2);
    }

    #[test]
    fn current_year_excluded_from_lookback() {
        // Profit for 2026 (the current year) shouldn't help — 2026 isn't
        // done yet.
        let h = vec![
            yp(2021, "-1"), yp(2022, "-1"),
            yp(2023, "-1"), yp(2024, "-1"),
            yp(2025, "-1"),
            yp(2026, "1000000"),    // huge profit this year — DOESN'T count
        ];
        let r = evaluate(&h, 2026, ActivityKind::Default);
        assert_eq!(r.verdict, SafeHarborVerdict::Failed);
    }

    #[test]
    fn empty_history_inconclusive() {
        let r = evaluate(&[], 2026, ActivityKind::Default);
        assert_eq!(r.verdict, SafeHarborVerdict::Inconclusive);
    }
}
