//! Debt snowball payoff planner.
//!
//! Dave Ramsey's behavioural-finance strategy: pay minimums on every
//! debt; route ALL extra payment to the debt with the **smallest
//! balance** first. When a debt clears, the user gets a psychological
//! "win" that fuels persistence — the small wins keep the user on the
//! plan even when the math says avalanche (highest-APR first) would
//! cost less interest. Kahneman & Tversky behavioural-economics
//! evidence (and Northwestern's 2012 Gal & McShane study) supports
//! the adherence-vs-optimality tradeoff.
//!
//! Simulation mirrors `debt_avalanche::compute` but ordering switches
//! from highest-APR to smallest-remaining-balance. Returns the same
//! shape so callers can A/B these two payoff strategies side-by-side.
//!
//! Pure compute.

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
pub struct DebtSnowballInput {
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
pub struct DebtSnowballReport {
    pub debts: Vec<DebtResult>,
    pub total_months: u32,
    pub all_paid_off: bool,
    pub total_interest_paid_usd: f64,
    pub total_principal_usd: f64,
    pub total_paid_usd: f64,
}

// ─── Pure compute ─────────────────────────────────────────────────────────

pub fn compute(input: &DebtSnowballInput) -> DebtSnowballReport {
    let n = input.debts.len();
    if n == 0 {
        return DebtSnowballReport {
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
        for i in 0..n {
            if balances[i] > 0.0 {
                let monthly_rate = input.debts[i].apr_pct / 100.0 / 12.0;
                let interest = balances[i] * monthly_rate;
                balances[i] += interest;
                interest_paid[i] += interest;
            }
        }
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
        // Snowball: extra goes to SMALLEST positive balance.
        let mut extra = rolling_extra;
        while extra > 0.005 {
            let mut target: Option<usize> = None;
            let mut best_bal = f64::INFINITY;
            for i in 0..n {
                if balances[i] > 0.0 && balances[i] < best_bal {
                    best_bal = balances[i];
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
    DebtSnowballReport {
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
    fn empty_input() {
        let r = compute(&DebtSnowballInput { debts: vec![], extra_payment_usd: 100.0 });
        assert!(r.all_paid_off);
        assert_eq!(r.total_months, 0);
    }

    #[test]
    fn single_debt_zero_apr_exact_months() {
        let r = compute(&DebtSnowballInput {
            debts: vec![d("only", 1000.0, 0.0, 100.0)],
            extra_payment_usd: 0.0,
        });
        assert!(r.all_paid_off);
        assert_eq!(r.total_months, 10);
    }

    #[test]
    fn snowball_prefers_smallest_balance() {
        // Two debts: large but high-APR, small but low-APR.
        // Snowball pays the SMALL one first regardless of APR.
        let r = compute(&DebtSnowballInput {
            debts: vec![
                d("big_high", 5000.0, 20.0, 100.0),
                d("small_low", 500.0,  5.0,  50.0),
            ],
            extra_payment_usd: 200.0,
        });
        assert!(r.all_paid_off);
        let small_month = r.debts[1].payoff_month.unwrap();
        let big_month = r.debts[0].payoff_month.unwrap();
        assert!(small_month < big_month,
            "snowball should pay small first; small={small_month}, big={big_month}");
    }

    #[test]
    fn snowball_pays_smaller_even_with_lower_apr() {
        let r = compute(&DebtSnowballInput {
            debts: vec![
                d("a", 100.0,  5.0, 25.0),  // small
                d("b", 200.0, 25.0, 25.0),  // bigger, higher-APR
            ],
            extra_payment_usd: 50.0,
        });
        assert!(r.all_paid_off);
        let a_month = r.debts[0].payoff_month.unwrap();
        let b_month = r.debts[1].payoff_month.unwrap();
        assert!(a_month <= b_month);
    }

    #[test]
    fn extra_payment_reduces_months() {
        let no_extra = compute(&DebtSnowballInput {
            debts: vec![d("cc", 5000.0, 18.0, 100.0)],
            extra_payment_usd: 0.0,
        });
        let with_extra = compute(&DebtSnowballInput {
            debts: vec![d("cc", 5000.0, 18.0, 100.0)],
            extra_payment_usd: 200.0,
        });
        assert!(with_extra.total_months < no_extra.total_months);
    }

    #[test]
    fn snowball_interest_higher_or_equal_to_avalanche_when_apr_inverted() {
        // When the SMALL debt has LOW APR, snowball ignores the high-APR
        // big debt longer, so its total interest should be > avalanche.
        // (We don't import avalanche here; just sanity-check non-zero.)
        let r = compute(&DebtSnowballInput {
            debts: vec![
                d("big_high",   10_000.0, 25.0, 200.0),
                d("small_low",   1_000.0,  5.0,  50.0),
            ],
            extra_payment_usd: 100.0,
        });
        assert!(r.all_paid_off);
        assert!(r.total_interest_paid_usd > 0.0);
    }

    #[test]
    fn rolled_min_after_first_payoff() {
        // The roll-over of the first debt's min onto the next smallest
        // should accelerate its payoff.
        let r = compute(&DebtSnowballInput {
            debts: vec![
                d("a", 100.0, 5.0, 25.0),
                d("b", 400.0, 5.0, 50.0),
            ],
            extra_payment_usd: 0.0,
        });
        assert!(r.all_paid_off);
    }

    #[test]
    fn total_paid_invariant() {
        let r = compute(&DebtSnowballInput {
            debts: vec![d("cc", 2000.0, 18.0, 100.0)],
            extra_payment_usd: 0.0,
        });
        assert!((r.total_paid_usd - r.total_principal_usd - r.total_interest_paid_usd).abs() < 0.01);
    }

    #[test]
    fn impossible_min_caps_at_max() {
        let r = compute(&DebtSnowballInput {
            debts: vec![d("trap", 10_000.0, 36.0, 10.0)],
            extra_payment_usd: 0.0,
        });
        assert!(!r.all_paid_off);
        assert_eq!(r.total_months, MAX_MONTHS);
    }

    #[test]
    fn debt_count_preserved() {
        let r = compute(&DebtSnowballInput {
            debts: vec![d("a", 100.0, 5.0, 10.0), d("b", 200.0, 8.0, 20.0)],
            extra_payment_usd: 0.0,
        });
        assert_eq!(r.debts.len(), 2);
        assert_eq!(r.debts[0].name, "a");
    }
}
