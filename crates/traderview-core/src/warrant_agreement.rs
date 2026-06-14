//! Warrant agreement — a standalone security giving the holder the right to buy
//! shares at a fixed exercise price until expiration, typically issued to an
//! investor or lender as a sweetener. It is distinct from an employee stock
//! option grant: there is no vesting and no AMT, and its defining mechanic is
//! the *cashless (net) exercise*, where the holder surrenders the in-the-money
//! value for shares instead of paying cash. It computes the cash-exercise cost,
//! the intrinsic value, the net shares from a cashless exercise, the warrant
//! coverage on a referenced loan, and the expiration date. Drafting aid, not
//! legal/securities advice.

use chrono::{Months, NaiveDate};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct WarrantInput {
    pub company_name: String,
    pub holder_name: String,
    /// Shares the warrant covers.
    pub warrant_shares: f64,
    /// Exercise (strike) price per share.
    pub strike_usd: f64,
    /// Current fair market value per share.
    pub fmv_usd: f64,
    /// Term in years before the warrant expires.
    pub term_years: u32,
    /// Optional referenced loan/investment, for warrant-coverage percentage.
    #[serde(default)]
    pub loan_amount_usd: f64,
    pub issue_date: String,
    pub state: String,
    #[serde(default)]
    pub statute_citation: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DocClause {
    pub heading: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct WarrantAgreement {
    pub title: String,
    pub warrant_shares: f64,
    /// Cost to exercise all warrants for cash (strike × shares).
    pub cash_exercise_cost_usd: f64,
    /// In-the-money value, max(fmv - strike, 0) × shares.
    pub intrinsic_value_usd: f64,
    /// Shares received from a cashless (net) exercise.
    pub cashless_net_shares: f64,
    /// Value of the cashless net shares at fair market value.
    pub cashless_value_usd: f64,
    /// Warrant coverage as a percent of the referenced loan (0 if none given).
    pub coverage_pct: f64,
    pub expiration_date: String,
    pub statutory_citation: String,
    pub clauses: Vec<DocClause>,
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

pub fn generate(i: &WarrantInput) -> WarrantAgreement {
    let spread = (i.fmv_usd - i.strike_usd).max(0.0);
    let cash_cost = cents(i.strike_usd * i.warrant_shares);
    let intrinsic = cents(spread * i.warrant_shares);
    // Cashless exercise: surrender intrinsic value for shares, no cash paid.
    let net_shares = if i.fmv_usd > 0.0 {
        (i.warrant_shares * spread / i.fmv_usd).floor()
    } else {
        0.0
    };
    let cashless_value = cents(net_shares * i.fmv_usd);
    let coverage = if i.loan_amount_usd > 0.0 {
        cents(intrinsic / i.loan_amount_usd * 100.0)
    } else {
        0.0
    };

    let expiration = NaiveDate::parse_from_str(&i.issue_date, "%Y-%m-%d")
        .ok()
        .and_then(|d| d.checked_add_months(Months::new(i.term_years * 12)))
        .map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_default();

    let citation = i.statute_citation.trim();
    let governing = if citation.is_empty() {
        format!("This warrant is governed by the laws of the State of {}.", i.state)
    } else {
        format!("This warrant is governed by the laws of the State of {} ({}).", i.state, citation)
    };

    let coverage_line = if i.loan_amount_usd > 0.0 {
        format!(
            " The warrant provides coverage of {:.2}% on the referenced {} loan/investment.",
            coverage,
            money(i.loan_amount_usd)
        )
    } else {
        String::new()
    };

    let exercise_body = format!(
        "The Holder may exercise for cash by paying {} per share ({} for all {:.0} shares), or by a cashless (net) exercise surrendering the in-the-money value for shares. At a fair market value of {} and strike of {}, the in-the-money value is {} ({} per share); a cashless exercise yields {:.0} net shares worth {}.{}",
        money(i.strike_usd),
        money(cash_cost),
        i.warrant_shares,
        money(i.fmv_usd),
        money(i.strike_usd),
        money(intrinsic),
        money(spread),
        net_shares,
        money(cashless_value),
        coverage_line
    );

    let clauses = vec![
        DocClause {
            heading: "Parties".into(),
            body: format!(
                "Company: {}\nHolder: {}\nIssue date: {}",
                i.company_name, i.holder_name, i.issue_date
            ),
        },
        DocClause {
            heading: "1. Grant of Warrant".into(),
            body: format!(
                "The Company grants the Holder the right to purchase {:.0} shares of common stock at an exercise price of {} per share, exercisable until the expiration date of {}.",
                i.warrant_shares, money(i.strike_usd), expiration
            ),
        },
        DocClause { heading: "2. Exercise".into(), body: exercise_body },
        DocClause {
            heading: "3. Expiration".into(),
            body: format!(
                "This warrant expires at 5:00 p.m. on {} ({} year term from issuance) and is void thereafter unless exercised.",
                expiration, i.term_years
            ),
        },
        DocClause {
            heading: "4. Adjustments".into(),
            body: "The number of shares and exercise price adjust proportionally for stock splits, combinations, stock dividends, and similar recapitalizations so the Holder's economic position is preserved.".into(),
        },
        DocClause {
            heading: "5. Transfer Restrictions".into(),
            body: "This warrant and the shares issuable on exercise are not registered and are subject to transfer restrictions under applicable securities law.".into(),
        },
        DocClause { heading: "6. Governing Law".into(), body: governing },
        DocClause {
            heading: "Signatures".into(),
            body: format!(
                "Company: ____________________  Date: __________\n{}\n\nHolder: ____________________  Date: __________\n{}",
                i.company_name, i.holder_name
            ),
        },
    ];

    WarrantAgreement {
        title: "Stock Purchase Warrant".into(),
        warrant_shares: i.warrant_shares,
        cash_exercise_cost_usd: cash_cost,
        intrinsic_value_usd: intrinsic,
        cashless_net_shares: net_shares,
        cashless_value_usd: cashless_value,
        coverage_pct: coverage,
        expiration_date: expiration,
        statutory_citation: citation.to_string(),
        clauses,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    fn base() -> WarrantInput {
        WarrantInput {
            company_name: "Startup Inc".into(),
            holder_name: "Bridge Lender LLC".into(),
            warrant_shares: 10_000.0,
            strike_usd: 2.00,
            fmv_usd: 10.00,
            term_years: 5,
            loan_amount_usd: 500_000.0,
            issue_date: "2026-07-01".into(),
            state: "Delaware".into(),
            statute_citation: String::new(),
        }
    }

    #[test]
    fn cashless_net_shares_and_values() {
        let d = generate(&base());
        assert!(close(d.cash_exercise_cost_usd, 20_000.0));
        assert!(close(d.intrinsic_value_usd, 80_000.0));
        // 10,000 × (10-2)/10 = 8,000 net shares.
        assert!(close(d.cashless_net_shares, 8_000.0));
        assert!(close(d.cashless_value_usd, 80_000.0));
    }

    #[test]
    fn coverage_on_loan() {
        // 80,000 intrinsic / 500,000 loan = 16%.
        assert!(close(generate(&base()).coverage_pct, 16.0));
    }

    #[test]
    fn no_loan_zero_coverage() {
        assert!(close(generate(&WarrantInput { loan_amount_usd: 0.0, ..base() }).coverage_pct, 0.0));
    }

    #[test]
    fn expiration_is_term_years_out() {
        assert_eq!(generate(&base()).expiration_date, "2031-07-01");
    }

    #[test]
    fn underwater_warrant_zero_net_shares() {
        let d = generate(&WarrantInput { fmv_usd: 1.50, ..base() });
        assert!(close(d.intrinsic_value_usd, 0.0));
        assert!(close(d.cashless_net_shares, 0.0));
        // Cash exercise cost is independent of FMV.
        assert!(close(d.cash_exercise_cost_usd, 20_000.0));
    }

    #[test]
    fn no_vesting_language() {
        // A warrant is not a vesting employee grant; no vesting clause exists.
        let d = generate(&base());
        assert!(!d.clauses.iter().any(|c| c.heading.to_lowercase().contains("vest")));
        assert!(d.clauses.iter().any(|c| c.heading.contains("Expiration")));
    }

    #[test]
    fn statute_citation_echoed() {
        let d = generate(&WarrantInput { statute_citation: "Cal. Corp. Code".into(), ..base() });
        assert_eq!(d.statutory_citation, "Cal. Corp. Code");
        assert!(d.clauses.iter().any(|c| c.body.contains("Cal. Corp. Code")));
    }
}
