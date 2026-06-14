//! Mortgage payoff vs invest — the extra-cash dilemma. Given a mortgage and a
//! fixed extra amount per month, compare two paths over a horizon: (A) throw the
//! extra at principal (pays the loan off early, then redirects the freed
//! payment + extra into the market) vs (B) keep the mortgage on its baseline
//! schedule and invest the extra each month. Both paths invest at the same
//! expected return; the report gives end-state financial wealth, payoff month,
//! lifetime interest, and the winner. When the borrower itemizes, the mortgage
//! rate is reduced by the marginal bracket to its after-tax effective rate.
//! Faithful port of the former client-side calculator. Pure compute, not advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct PayoffVsInvestInput {
    pub balance_usd: f64,
    pub mortgage_rate_pct: f64,
    pub term_months: u32,
    pub extra_monthly_usd: f64,
    pub expected_return_pct: f64,
    pub marginal_tax_pct: f64,
    #[serde(default)]
    pub itemize: bool,
    pub horizon_years: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct PayoffVsInvestReport {
    pub base_payment_usd: f64,
    pub effective_rate_pct: f64,
    /// Path A: pay extra against the mortgage.
    pub wealth_payoff_usd: f64,
    /// Path B: invest the extra.
    pub wealth_invest_usd: f64,
    /// Path B − Path A: positive means investing wins.
    pub gap_usd: f64,
    /// "INVEST", "PAYOFF", or "TIE".
    pub winner: String,
    pub payoff_month_payoff_path: Option<u32>,
    pub payoff_month_invest_path: Option<u32>,
    pub interest_paid_payoff_usd: f64,
    pub interest_paid_invest_usd: f64,
    /// Interest the payoff path avoids (invest-path interest − payoff-path interest).
    pub interest_saved_usd: f64,
    pub valid: bool,
}

fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn round4(x: f64) -> f64 {
    (x * 10_000.0).round() / 10_000.0
}

/// Level monthly payment that amortizes `bal` over `n` months at monthly rate `r_m`.
fn amort(bal: f64, r_m: f64, n: u32) -> f64 {
    if r_m == 0.0 {
        return bal / n as f64;
    }
    let f = (1.0 + r_m).powi(n as i32);
    bal * r_m * f / (f - 1.0)
}

pub fn generate(i: &PayoffVsInvestInput) -> PayoffVsInvestReport {
    if i.balance_usd <= 0.0 || i.term_months == 0 || i.horizon_years == 0 {
        return PayoffVsInvestReport::default();
    }
    let m_rate = i.mortgage_rate_pct / 100.0;
    let er = i.expected_return_pct / 100.0;
    let tax = i.marginal_tax_pct / 100.0;
    let r_m = m_rate / 12.0;
    let i_m = (1.0 + er).powf(1.0 / 12.0) - 1.0;
    let eff_rate = if i.itemize { m_rate * (1.0 - tax) } else { m_rate };
    let base_pi = amort(i.balance_usd, r_m, i.term_months);
    let horizon_months = i.horizon_years * 12;

    // Path A — pay the extra against principal; redirect freed cash after payoff.
    let mut bal_a = i.balance_usd;
    let mut cash_freed_invested = 0.0;
    let mut interest_a = 0.0;
    let mut payoff_a: Option<u32> = None;
    for m in 1..=horizon_months {
        cash_freed_invested *= 1.0 + i_m;
        if bal_a > 0.0 {
            let interest = bal_a * r_m;
            interest_a += interest;
            let principal = (base_pi + i.extra_monthly_usd) - interest;
            bal_a = (bal_a - principal).max(0.0);
            if bal_a == 0.0 && payoff_a.is_none() {
                payoff_a = Some(m);
            }
        } else {
            cash_freed_invested += base_pi + i.extra_monthly_usd;
        }
    }
    let wealth_a = cash_freed_invested - bal_a;

    // Path B — keep the mortgage on schedule; invest the extra each month.
    let mut bal_b = i.balance_usd;
    let mut invested_b = 0.0;
    let mut interest_b = 0.0;
    let mut payoff_b: Option<u32> = None;
    for m in 1..=horizon_months {
        invested_b *= 1.0 + i_m;
        if bal_b > 0.0 {
            let interest = bal_b * r_m;
            interest_b += interest;
            let principal = base_pi - interest;
            bal_b = (bal_b - principal).max(0.0);
            if bal_b == 0.0 && payoff_b.is_none() {
                payoff_b = Some(m);
            }
            invested_b += i.extra_monthly_usd;
        } else {
            invested_b += base_pi + i.extra_monthly_usd;
        }
    }
    let wealth_b = invested_b - bal_b;

    let gap = wealth_b - wealth_a;
    let winner = if gap > 0.0 {
        "INVEST"
    } else if gap < 0.0 {
        "PAYOFF"
    } else {
        "TIE"
    };

    PayoffVsInvestReport {
        base_payment_usd: round2(base_pi),
        effective_rate_pct: round4(eff_rate * 100.0),
        wealth_payoff_usd: round2(wealth_a),
        wealth_invest_usd: round2(wealth_b),
        gap_usd: round2(gap),
        winner: winner.to_string(),
        payoff_month_payoff_path: payoff_a,
        payoff_month_invest_path: payoff_b,
        interest_paid_payoff_usd: round2(interest_a),
        interest_paid_invest_usd: round2(interest_b),
        interest_saved_usd: round2(interest_b - interest_a),
        valid: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    fn base() -> PayoffVsInvestInput {
        PayoffVsInvestInput {
            balance_usd: 350_000.0,
            mortgage_rate_pct: 6.5,
            term_months: 300,
            extra_monthly_usd: 500.0,
            expected_return_pct: 7.0,
            marginal_tax_pct: 22.0,
            itemize: false,
            horizon_years: 25,
        }
    }

    // Pins cross-checked against the JS compute() in Python.
    #[test]
    fn default_invest_wins() {
        let d = generate(&base());
        assert!(d.valid);
        assert!(close(d.base_payment_usd, 2_363.23));
        assert!(close(d.effective_rate_pct, 6.5));
        assert!(close(d.wealth_payoff_usd, 378_527.41));
        assert!(close(d.wealth_invest_usd, 391_520.94));
        assert!(close(d.gap_usd, 12_993.53));
        assert_eq!(d.winner, "INVEST");
        assert_eq!(d.payoff_month_payoff_path, Some(201));
        assert_eq!(d.payoff_month_invest_path, Some(300));
        assert!(close(d.interest_paid_payoff_usd, 225_132.01));
        assert!(close(d.interest_paid_invest_usd, 358_967.52));
        assert!(close(d.interest_saved_usd, 133_835.51));
    }

    #[test]
    fn itemizing_lowers_effective_rate() {
        let d = generate(&PayoffVsInvestInput { itemize: true, ..base() });
        // 6.5% × (1 − 0.22) = 5.07%.
        assert!(close(d.effective_rate_pct, 5.07));
    }

    #[test]
    fn low_return_favors_payoff() {
        // A market return well below the mortgage rate flips the winner.
        let d = generate(&PayoffVsInvestInput { expected_return_pct: 1.0, ..base() });
        assert_eq!(d.winner, "PAYOFF");
        assert!(d.wealth_payoff_usd > d.wealth_invest_usd);
    }

    #[test]
    fn invalid_when_balance_zero() {
        let d = generate(&PayoffVsInvestInput { balance_usd: 0.0, ..base() });
        assert!(!d.valid);
    }
}
