//! PSA Mortgage Prepayment Cash Flow Generator.
//!
//! Generates monthly cash flows for a fixed-rate MBS pool under the
//! Public Securities Association (PSA) prepayment convention:
//!
//!   100% PSA: CPR = 0.2% · age_months for age ≤ 30, then 6.0% thereafter
//!   N% PSA:    CPR = (N/100) · 100% PSA curve
//!
//! Where CPR (Conditional Prepayment Rate) is the annualized voluntary
//! prepayment rate. SMM (Single Monthly Mortality) is the monthly
//! equivalent: SMM = 1 − (1 − CPR)^(1/12).
//!
//! Per-month cash flow:
//!   - scheduled_interest  = beg_balance · gross_coupon / 12
//!   - scheduled_principal = level-payment amortization
//!   - prepayment          = SMM · (beg_balance − scheduled_principal)
//!   - total_principal     = scheduled_principal + prepayment
//!   - ending_balance      = beg_balance − total_principal
//!
//! Pure compute. Companion to `convertible_bond`, `macaulay_duration`,
//! `bond_convexity`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyCashFlow {
    pub month: usize,
    pub beginning_balance: f64,
    pub scheduled_interest: f64,
    pub scheduled_principal: f64,
    pub prepayment: f64,
    pub total_principal: f64,
    pub total_cash_flow: f64,
    pub ending_balance: f64,
    pub cpr_annualized: f64,
    pub smm_monthly: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PsaScheduleReport {
    pub cash_flows: Vec<MonthlyCashFlow>,
    pub total_interest: f64,
    pub total_principal: f64,
    pub weighted_average_life: f64,
}

pub fn schedule(
    original_balance: f64,
    gross_coupon: f64,         // annual, decimal
    original_term_months: usize,
    psa_speed_pct: f64,         // 100.0 = 100% PSA
) -> Option<PsaScheduleReport> {
    if !original_balance.is_finite() || original_balance <= 0.0
        || !gross_coupon.is_finite() || gross_coupon <= 0.0
        || original_term_months == 0
        || !psa_speed_pct.is_finite() || psa_speed_pct < 0.0
    {
        return None;
    }
    let monthly_rate = gross_coupon / 12.0;
    // Level payment per pool dollar.
    let n = original_term_months as f64;
    let denom = 1.0 - (1.0 + monthly_rate).powf(-n);
    if denom <= 0.0 { return None; }
    let level_payment = original_balance * monthly_rate / denom;
    let mut bal = original_balance;
    let mut total_int = 0.0_f64;
    let mut total_prin = 0.0_f64;
    let mut wal_weight = 0.0_f64;
    let mut wal_denom = 0.0_f64;
    let mut cfs = Vec::with_capacity(original_term_months);
    for month_idx in 1..=original_term_months {
        if bal <= 1e-9 { break; }
        let age = month_idx as f64;
        let cpr_100 = 0.06_f64 * (age.min(30.0) / 30.0);
        let cpr = (psa_speed_pct / 100.0) * cpr_100;
        let smm = 1.0 - (1.0 - cpr).powf(1.0 / 12.0);
        let interest = bal * monthly_rate;
        let scheduled_prin = (level_payment - interest).max(0.0).min(bal);
        let after_sched = bal - scheduled_prin;
        let prepay = (smm * after_sched).min(after_sched);
        let total_prin_month = scheduled_prin + prepay;
        let end_bal = (bal - total_prin_month).max(0.0);
        let total_cf = interest + total_prin_month;
        cfs.push(MonthlyCashFlow {
            month: month_idx,
            beginning_balance: bal,
            scheduled_interest: interest,
            scheduled_principal: scheduled_prin,
            prepayment: prepay,
            total_principal: total_prin_month,
            total_cash_flow: total_cf,
            ending_balance: end_bal,
            cpr_annualized: cpr,
            smm_monthly: smm,
        });
        total_int += interest;
        total_prin += total_prin_month;
        wal_weight += total_prin_month * age;
        wal_denom += total_prin_month;
        bal = end_bal;
    }
    let wal_years = if wal_denom > 0.0 { wal_weight / wal_denom / 12.0 } else { 0.0 };
    Some(PsaScheduleReport {
        cash_flows: cfs,
        total_interest: total_int,
        total_principal: total_prin,
        weighted_average_life: wal_years,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_none() {
        assert!(schedule(0.0, 0.06, 360, 100.0).is_none());
        assert!(schedule(100_000.0, 0.0, 360, 100.0).is_none());
        assert!(schedule(100_000.0, 0.06, 0, 100.0).is_none());
        assert!(schedule(100_000.0, 0.06, 360, -1.0).is_none());
        assert!(schedule(100_000.0, f64::NAN, 360, 100.0).is_none());
    }

    #[test]
    fn zero_psa_yields_no_prepayment() {
        let r = schedule(100_000.0, 0.06, 360, 0.0).unwrap();
        let total_prepay: f64 = r.cash_flows.iter().map(|c| c.prepayment).sum();
        assert!(total_prepay.abs() < 1e-9, "0 PSA: prepayments should be 0, got {total_prepay}");
    }

    #[test]
    fn psa_100_cpr_ramps_to_6pct_by_month_30() {
        let r = schedule(100_000.0, 0.06, 360, 100.0).unwrap();
        // CPR at month 1: 0.2%; month 30: 6%; month 60: 6% (capped).
        assert!((r.cash_flows[0].cpr_annualized - 0.002).abs() < 1e-9);
        assert!((r.cash_flows[29].cpr_annualized - 0.06).abs() < 1e-9);
        assert!((r.cash_flows[59].cpr_annualized - 0.06).abs() < 1e-9);
    }

    #[test]
    fn psa_200_doubles_cpr_curve() {
        let r100 = schedule(100_000.0, 0.06, 360, 100.0).unwrap();
        let r200 = schedule(100_000.0, 0.06, 360, 200.0).unwrap();
        // CPR at month 5 should be 2x.
        let ratio = r200.cash_flows[4].cpr_annualized / r100.cash_flows[4].cpr_annualized;
        assert!((ratio - 2.0).abs() < 1e-9);
    }

    #[test]
    fn higher_psa_shortens_wal() {
        let slow = schedule(100_000.0, 0.06, 360, 50.0).unwrap();
        let fast = schedule(100_000.0, 0.06, 360, 400.0).unwrap();
        assert!(fast.weighted_average_life < slow.weighted_average_life,
            "fast PSA WAL {} should be less than slow PSA WAL {}",
            fast.weighted_average_life, slow.weighted_average_life);
    }

    #[test]
    fn ending_balance_decreases_monotonically() {
        let r = schedule(100_000.0, 0.06, 360, 100.0).unwrap();
        for w in r.cash_flows.windows(2) {
            assert!(w[1].ending_balance <= w[0].ending_balance);
        }
        // Final balance should be ~0.
        assert!(r.cash_flows.last().unwrap().ending_balance < 1.0);
    }

    #[test]
    fn total_principal_repaid_equals_original_balance() {
        let r = schedule(100_000.0, 0.06, 360, 100.0).unwrap();
        assert!((r.total_principal - 100_000.0).abs() < 1.0);
    }

    #[test]
    fn ending_balance_consistent_with_beginning_minus_principal() {
        let r = schedule(100_000.0, 0.06, 360, 200.0).unwrap();
        for c in &r.cash_flows {
            assert!((c.beginning_balance - c.total_principal - c.ending_balance).abs() < 1e-6);
        }
    }
}
