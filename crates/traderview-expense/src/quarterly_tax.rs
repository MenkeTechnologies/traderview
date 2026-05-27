//! Quarterly estimated-tax (Form 1040-ES) forecaster.
//!
//! Federal estimated tax safe harbor: pay at least 100% of last year's tax
//! (110% if AGI > $150k) OR 90% of this year's tax — whichever is smaller.
//! Underpayment penalty applies if quarterly payments fall short.
//!
//! We compute:
//!   * the safe-harbor target (smaller of the two)
//!   * per-quarter due dates (Apr 15 / Jun 15 / Sep 15 / Jan 15 next year)
//!   * even-quartered payment amount = target / 4
//!   * the current-year projection from running YTD net P&L (annualized)
//!
//! Output is a `QuarterlyForecast` the UI can render as a 4-row schedule.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct ForecastInput {
    pub tax_year: i32,
    /// Total tax liability from the PRIOR year's return (line 24 on 1040).
    pub prior_year_total_tax: Decimal,
    /// AGI from the PRIOR year — controls whether safe harbor is 100% or 110%.
    pub prior_year_agi: Decimal,
    /// Current-year YTD net P&L (Schedule C feed). Used to project full-year.
    pub ytd_net_profit: Decimal,
    /// Day of year through which YTD covers — 1..=365.
    pub days_through_ytd: i32,
    /// Estimated current-year effective tax rate as a Decimal in `[0,1]`.
    /// Conservative default: 24% federal + 7.65% SE * 0.5 = ~28%.
    pub estimated_effective_tax_rate: Decimal,
    /// Withholding already collected via W-2 / 1099 etc. Reduces the
    /// quarterly target.
    pub withholding_ytd: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quarter {
    pub period_label: String,
    pub due_date: NaiveDate,
    pub estimated_payment: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuarterlyForecast {
    /// 100% (or 110% for high-AGI) of prior-year total tax.
    pub safe_harbor_prior_year: Decimal,
    /// 90% of projected current-year tax.
    pub safe_harbor_current_year: Decimal,
    /// Smaller of the two — what the IRS actually requires.
    pub safe_harbor_target: Decimal,
    /// Projected full-year net profit, annualized from YTD.
    pub projected_annual_net_profit: Decimal,
    /// Projected full-year tax based on rate × projected net profit.
    pub projected_annual_tax: Decimal,
    /// safe_harbor_target − withholding_ytd, never negative.
    pub remaining_to_pay: Decimal,
    /// One row per quarter. Due dates for the tax year.
    pub quarters: [Quarter; 4],
}

/// IRS quarterly due dates for the tax year. Q1 covers Jan-Mar, due Apr 15.
/// Q2 covers Apr-May (yes, 2 months — not 3), due Jun 15. Q3 covers Jun-Aug
/// due Sep 15. Q4 covers Sep-Dec due Jan 15 of the NEXT year.
/// Weekend / holiday shifts are NOT modeled — IRS publishes the exact date.
pub fn due_dates(year: i32) -> [NaiveDate; 4] {
    [
        NaiveDate::from_ymd_opt(year, 4, 15).unwrap(),
        NaiveDate::from_ymd_opt(year, 6, 15).unwrap(),
        NaiveDate::from_ymd_opt(year, 9, 15).unwrap(),
        NaiveDate::from_ymd_opt(year + 1, 1, 15).unwrap(),
    ]
}

pub fn forecast(input: &ForecastInput) -> QuarterlyForecast {
    // Prior-year safe harbor: 100% normally, 110% if prior AGI > $150k
    // (or $75k for married-filing-separately — not modeled; user can
    // adjust the rate input).
    let high_agi_threshold = Decimal::from(150_000);
    let multiplier = if input.prior_year_agi > high_agi_threshold {
        Decimal::from_str("1.10").unwrap()
    } else {
        Decimal::ONE
    };
    let safe_harbor_prior = input.prior_year_total_tax * multiplier;

    // Project annual net profit. If 0 days passed, can't project — fall back
    // to YTD (will be $0).
    let projected_annual_net_profit = if input.days_through_ytd > 0 {
        input.ytd_net_profit * Decimal::from(365) / Decimal::from(input.days_through_ytd as i64)
    } else {
        input.ytd_net_profit
    };

    let projected_annual_tax = projected_annual_net_profit * input.estimated_effective_tax_rate;
    let ninety_pct = Decimal::from_str("0.90").unwrap();
    let safe_harbor_current = projected_annual_tax * ninety_pct;

    let safe_harbor_target = safe_harbor_prior.min(safe_harbor_current);
    let remaining = (safe_harbor_target - input.withholding_ytd).max(Decimal::ZERO);
    let per_q = remaining / Decimal::from(4);

    let dates = due_dates(input.tax_year);
    let quarters = [
        Quarter {
            period_label: "Q1 (Jan-Mar)".into(),
            due_date: dates[0],
            estimated_payment: per_q,
        },
        Quarter {
            period_label: "Q2 (Apr-May)".into(),
            due_date: dates[1],
            estimated_payment: per_q,
        },
        Quarter {
            period_label: "Q3 (Jun-Aug)".into(),
            due_date: dates[2],
            estimated_payment: per_q,
        },
        Quarter {
            period_label: "Q4 (Sep-Dec)".into(),
            due_date: dates[3],
            estimated_payment: per_q,
        },
    ];

    QuarterlyForecast {
        safe_harbor_prior_year: safe_harbor_prior,
        safe_harbor_current_year: safe_harbor_current,
        safe_harbor_target,
        projected_annual_net_profit,
        projected_annual_tax,
        remaining_to_pay: remaining,
        quarters,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    fn base_input() -> ForecastInput {
        ForecastInput {
            tax_year: 2026,
            prior_year_total_tax: d("20000"),
            prior_year_agi: d("100000"),
            ytd_net_profit: d("30000"),
            days_through_ytd: 90, // ~Q1 done
            estimated_effective_tax_rate: d("0.28"),
            withholding_ytd: Decimal::ZERO,
        }
    }

    #[test]
    fn due_dates_are_standard_irs_schedule() {
        let d = due_dates(2026);
        assert_eq!(d[0], NaiveDate::from_ymd_opt(2026, 4, 15).unwrap());
        assert_eq!(d[1], NaiveDate::from_ymd_opt(2026, 6, 15).unwrap());
        assert_eq!(d[2], NaiveDate::from_ymd_opt(2026, 9, 15).unwrap());
        // Q4 due in next calendar year.
        assert_eq!(d[3], NaiveDate::from_ymd_opt(2027, 1, 15).unwrap());
    }

    #[test]
    fn safe_harbor_uses_100_percent_below_high_agi() {
        let f = forecast(&base_input());
        assert_eq!(f.safe_harbor_prior_year, d("20000"));
    }

    #[test]
    fn safe_harbor_uses_110_percent_above_high_agi() {
        let mut i = base_input();
        i.prior_year_agi = d("200000"); // > $150k
        let f = forecast(&i);
        assert_eq!(f.safe_harbor_prior_year, d("22000.00"));
    }

    #[test]
    fn projects_annual_from_ytd_correctly() {
        // 30,000 in 90 days → 30000 * 365/90 = 121,666.67 annualized.
        let f = forecast(&base_input());
        // Decimal exact: 30000 * 365 / 90 = 121666.6666...
        assert!(f.projected_annual_net_profit > d("121666"));
        assert!(f.projected_annual_net_profit < d("121667"));
    }

    #[test]
    fn target_picks_smaller_of_two_safe_harbors() {
        // base_input: prior tax = $20k, projected annual tax = 121666.67 ×
        // 0.28 ≈ 34066.67. 90% of that = ~30,660. So prior-year ($20k) wins.
        let f = forecast(&base_input());
        assert_eq!(f.safe_harbor_target, d("20000"));
    }

    #[test]
    fn target_uses_current_year_when_prior_higher() {
        let mut i = base_input();
        i.prior_year_total_tax = d("100000"); // huge prior tax
        i.ytd_net_profit = d("1000"); // tiny current YTD
        let f = forecast(&i);
        // Current 90% should be much smaller now.
        assert!(f.safe_harbor_target < d("10000"));
        assert!(f.safe_harbor_target < f.safe_harbor_prior_year);
    }

    #[test]
    fn withholding_reduces_remaining_payment() {
        let mut i = base_input();
        i.withholding_ytd = d("8000");
        let f = forecast(&i);
        // Target 20000 − 8000 = 12000 remaining → $3000/q.
        assert_eq!(f.remaining_to_pay, d("12000"));
        assert_eq!(f.quarters[0].estimated_payment, d("3000"));
    }

    #[test]
    fn withholding_over_target_yields_zero_remaining() {
        let mut i = base_input();
        i.withholding_ytd = d("100000");
        let f = forecast(&i);
        assert_eq!(f.remaining_to_pay, Decimal::ZERO);
        assert_eq!(f.quarters[0].estimated_payment, Decimal::ZERO);
    }

    #[test]
    fn quarter_payments_split_evenly() {
        let f = forecast(&base_input());
        let q0 = f.quarters[0].estimated_payment;
        assert_eq!(f.quarters[1].estimated_payment, q0);
        assert_eq!(f.quarters[2].estimated_payment, q0);
        assert_eq!(f.quarters[3].estimated_payment, q0);
        assert_eq!(q0 * Decimal::from(4), f.remaining_to_pay);
    }

    #[test]
    fn zero_days_ytd_does_not_divide_by_zero() {
        let mut i = base_input();
        i.days_through_ytd = 0;
        let f = forecast(&i);
        // No projection, just falls back to YTD ($30k) without panic.
        assert_eq!(f.projected_annual_net_profit, d("30000"));
    }
}
