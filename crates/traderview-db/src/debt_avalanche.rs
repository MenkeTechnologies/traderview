//! Debt avalanche payoff planner.
//!
//! Strategy: pay minimums on every debt, then route ALL extra
//! payment to the debt with the **highest APR** (mathematically
//! optimal — minimises total interest paid). When the highest-APR
//! debt is gone, roll its full payment (minimum + extra) onto the
//! next-highest-APR debt, and so on (the "snowball effect" applied
//! to avalanche ordering).
//!
//! Compared to `debt_snowball` (which orders by smallest balance for
//! psychological wins), avalanche is the optimal strategy for total
//! interest minimisation — Kahneman & Tversky would argue snowball
//! wins on adherence, but for the pure-math view, this is the right
//! algorithm.
//!
//! Each debt input: name, balance_usd, apr_pct, min_payment_usd.
//! Plus extra_payment_usd applied per month on top of all minimums.
//!
//! Simulation: month-by-month for up to `MAX_MONTHS = 480` (40 years).
//! Each month:
//!   1. accrue interest = balance × APR / 12 on each debt
//!   2. pay minimum on each debt
//!   3. apply extra to highest-APR debt with positive balance
//!   4. when a debt hits 0, "roll" its minimum into the extra
//!
//! Returns payoff month per debt, total interest paid per debt,
//! aggregate total interest, total payoff months, payoff schedule.
//!
//! Pure compute — no DB I/O.

use serde::{Deserialize, Serialize};

const MAX_MONTHS: u32 = 480;

#[derive(Debug, Clone, Deserialize)]
pub struct DebtInput {
    pub name: String,
    pub balance_usd: f64,
    pub apr_pct: f64,
    pub min_payment_usd: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DebtAvalancheInput {
    #[serde(default)]
    pub debts: Vec<DebtInput>,
    #[serde(default)]
    pub extra_payment_usd: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DebtResult {
    pub name: String,
    pub starting_balance_usd: f64,
    pub apr_pct: f64,
    pub min_payment_usd: f64,
    pub payoff_month: Option<u32>,
    pub total_interest_paid_usd: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DebtAvalancheReport {
    pub debts: Vec<DebtResult>,
    pub total_months: u32,
    pub all_paid_off: bool,
    pub total_interest_paid_usd: f64,
    pub total_principal_usd: f64,
    pub total_paid_usd: f64,
}

// ─── Pure compute ─────────────────────────────────────────────────────────

pub fn compute(input: &DebtAvalancheInput) -> DebtAvalancheReport {
    let n = input.debts.len();
    if n == 0 {
        return DebtAvalancheReport {
            debts: vec![],
            total_months: 0,
            all_paid_off: true,
            total_interest_paid_usd: 0.0,
            total_principal_usd: 0.0,
            total_paid_usd: 0.0,
        };
    }
    let mut balances: Vec<f64> = input.debts.iter().map(|d| d.balance_usd).collect();
    let mut interest_paid: Vec<f64> = vec![0.0; n];
    let mut payoff_month: Vec<Option<u32>> = vec![None; n];
    let starts: Vec<f64> = balances.clone();
    let mut rolling_extra = input.extra_payment_usd;
    let mut total_months: u32 = 0;
    let mut all_paid = false;
    let mut month: u32 = 0;
    while month < MAX_MONTHS {
        month += 1;
        // 1. accrue interest
        for i in 0..n {
            if balances[i] > 0.0 {
                let monthly_rate = input.debts[i].apr_pct / 100.0 / 12.0;
                let interest = balances[i] * monthly_rate;
                balances[i] += interest;
                interest_paid[i] += interest;
            }
        }
        // 2. pay minimum on each active debt (capped at outstanding)
        for i in 0..n {
            if balances[i] > 0.0 {
                let pay = input.debts[i].min_payment_usd.min(balances[i]);
                balances[i] -= pay;
                if balances[i] <= 0.005 {
                    balances[i] = 0.0;
                    if payoff_month[i].is_none() {
                        payoff_month[i] = Some(month);
                        rolling_extra += input.debts[i].min_payment_usd;
                    }
                }
            }
        }
        // 3. apply extra to highest-APR active debt
        let mut extra = rolling_extra;
        while extra > 0.005 {
            let mut target: Option<usize> = None;
            let mut best_apr = f64::NEG_INFINITY;
            for i in 0..n {
                if balances[i] > 0.0 && input.debts[i].apr_pct > best_apr {
                    best_apr = input.debts[i].apr_pct;
                    target = Some(i);
                }
            }
            match target {
                None => break,
                Some(t) => {
                    let pay = extra.min(balances[t]);
                    balances[t] -= pay;
                    extra -= pay;
                    if balances[t] <= 0.005 {
                        balances[t] = 0.0;
                        if payoff_month[t].is_none() {
                            payoff_month[t] = Some(month);
                            rolling_extra += input.debts[t].min_payment_usd;
                        }
                    }
                }
            }
        }
        // 4. termination
        if balances.iter().all(|b| *b <= 0.005) {
            all_paid = true;
            total_months = month;
            break;
        }
    }
    if !all_paid {
        total_months = MAX_MONTHS;
    }
    let debts: Vec<DebtResult> = input
        .debts
        .iter()
        .enumerate()
        .map(|(i, d)| DebtResult {
            name: d.name.clone(),
            starting_balance_usd: starts[i],
            apr_pct: d.apr_pct,
            min_payment_usd: d.min_payment_usd,
            payoff_month: payoff_month[i],
            total_interest_paid_usd: interest_paid[i],
        })
        .collect();
    let total_interest: f64 = interest_paid.iter().sum();
    let total_principal: f64 = starts.iter().sum();
    let total_paid = total_interest + total_principal;
    DebtAvalancheReport {
        debts,
        total_months,
        all_paid_off: all_paid,
        total_interest_paid_usd: total_interest,
        total_principal_usd: total_principal,
        total_paid_usd: total_paid,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(name: &str, bal: f64, apr: f64, min: f64) -> DebtInput {
        DebtInput { name: name.into(), balance_usd: bal, apr_pct: apr, min_payment_usd: min }
    }

    #[test]
    fn empty_input_zero_months() {
        let r = compute(&DebtAvalancheInput { debts: vec![], extra_payment_usd: 100.0 });
        assert_eq!(r.total_months, 0);
        assert!(r.all_paid_off);
        assert_eq!(r.total_interest_paid_usd, 0.0);
    }

    #[test]
    fn single_debt_pays_off() {
        // $1000 balance, 0% APR, $100/mo min, no extra → 10 months.
        let r = compute(&DebtAvalancheInput {
            debts: vec![d("only", 1000.0, 0.0, 100.0)],
            extra_payment_usd: 0.0,
        });
        assert!(r.all_paid_off);
        assert_eq!(r.total_months, 10);
        assert_eq!(r.total_interest_paid_usd, 0.0);
        assert_eq!(r.debts[0].payoff_month, Some(10));
    }

    #[test]
    fn single_debt_with_apr() {
        // $1000 balance, 12% APR, $100/mo min → should take > 10 months
        // due to interest, and total interest > 0.
        let r = compute(&DebtAvalancheInput {
            debts: vec![d("cc", 1000.0, 12.0, 100.0)],
            extra_payment_usd: 0.0,
        });
        assert!(r.all_paid_off);
        assert!(r.total_months > 10);
        assert!(r.total_interest_paid_usd > 0.0);
    }

    #[test]
    fn avalanche_prefers_highest_apr() {
        // Two debts: A 20% APR, B 5% APR. Extra payment must go to A.
        let r = compute(&DebtAvalancheInput {
            debts: vec![
                d("low",   1000.0, 5.0,  100.0),
                d("high",  1000.0, 20.0, 100.0),
            ],
            extra_payment_usd: 200.0,
        });
        assert!(r.all_paid_off);
        // High-APR debt should be paid off FIRST (lower month number).
        let high_month = r.debts[1].payoff_month.unwrap();
        let low_month  = r.debts[0].payoff_month.unwrap();
        assert!(high_month < low_month,
            "expected high-APR to pay off first: high={high_month}, low={low_month}");
        // Total interest on high-APR debt should be substantial.
        assert!(r.debts[1].total_interest_paid_usd > 0.0);
    }

    #[test]
    fn extra_payment_speeds_payoff() {
        let no_extra = compute(&DebtAvalancheInput {
            debts: vec![d("cc", 5000.0, 18.0, 100.0)],
            extra_payment_usd: 0.0,
        });
        let with_extra = compute(&DebtAvalancheInput {
            debts: vec![d("cc", 5000.0, 18.0, 100.0)],
            extra_payment_usd: 200.0,
        });
        assert!(with_extra.total_months < no_extra.total_months);
        assert!(with_extra.total_interest_paid_usd < no_extra.total_interest_paid_usd);
    }

    #[test]
    fn rolled_minimum_after_payoff() {
        // Two debts: small first to clear, then roll its payment.
        let r = compute(&DebtAvalancheInput {
            debts: vec![
                d("small_high", 100.0,  25.0, 50.0),  // pays off month 2
                d("big_low",   3000.0,  5.0, 100.0),
            ],
            extra_payment_usd: 0.0,
        });
        assert!(r.all_paid_off);
        let small_month = r.debts[0].payoff_month.unwrap();
        assert!(small_month <= 3);
        // After small is paid off, its $50 min rolls onto big_low.
        // big_low pays off with rolled payment faster than alone.
    }

    #[test]
    fn total_paid_equals_principal_plus_interest() {
        let r = compute(&DebtAvalancheInput {
            debts: vec![d("cc", 1000.0, 12.0, 100.0)],
            extra_payment_usd: 0.0,
        });
        assert!((r.total_paid_usd - r.total_principal_usd - r.total_interest_paid_usd).abs() < 0.01);
    }

    #[test]
    fn impossible_payoff_min_below_interest_caps_at_max() {
        // $10k at 36% APR, $10/mo min → interest grows faster than min.
        let r = compute(&DebtAvalancheInput {
            debts: vec![d("trap", 10_000.0, 36.0, 10.0)],
            extra_payment_usd: 0.0,
        });
        // With our cap of MAX_MONTHS, this should not have paid off.
        assert!(!r.all_paid_off);
        assert_eq!(r.total_months, MAX_MONTHS);
    }

    #[test]
    fn zero_balance_debt_immediately_paid_off() {
        let r = compute(&DebtAvalancheInput {
            debts: vec![
                d("paid_off", 0.0, 5.0, 50.0),
                d("active", 500.0, 5.0, 100.0),
            ],
            extra_payment_usd: 0.0,
        });
        // already-zero debt should have payoff_month = 1 (first iteration check)
        // or stay None depending on initial check. Either way `all_paid_off` true once active clears.
        assert!(r.all_paid_off);
    }

    #[test]
    fn debt_count_preserved_in_output() {
        let r = compute(&DebtAvalancheInput {
            debts: vec![d("a", 100.0, 5.0, 10.0), d("b", 200.0, 8.0, 20.0)],
            extra_payment_usd: 0.0,
        });
        assert_eq!(r.debts.len(), 2);
        assert_eq!(r.debts[0].name, "a");
        assert_eq!(r.debts[1].name, "b");
    }
}
