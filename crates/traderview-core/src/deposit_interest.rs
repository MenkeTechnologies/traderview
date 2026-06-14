//! Security-deposit interest — many jurisdictions require a landlord to pay the
//! tenant interest on a held security deposit. This computes that interest over
//! the tenancy at the applicable rate, simple or annually compounded, and the
//! total returnable (deposit + interest). It produces the figure that the
//! security-deposit-itemization document takes as a manual input, so the two
//! compose: compute the interest here, itemize deductions there. Drafting aid,
//! not legal advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct DepositInterestInput {
    pub landlord_name: String,
    pub tenant_name: String,
    #[serde(default)]
    pub property_label: String,
    /// Deposit held.
    pub deposit_usd: f64,
    /// Applicable annual interest rate, percent.
    pub annual_rate_pct: f64,
    /// Length of the tenancy, in months.
    pub term_months: u32,
    /// "simple" or "annual" (annually compounded).
    #[serde(default)]
    pub compounding: String,
    pub date: String,
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
pub struct DepositInterest {
    pub title: String,
    pub deposit_usd: f64,
    pub years: f64,
    /// True when interest is annually compounded rather than simple.
    pub compounded: bool,
    pub interest_usd: f64,
    /// Deposit + interest.
    pub total_returnable_usd: f64,
    pub statutory_citation: String,
    pub clauses: Vec<DocClause>,
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

pub fn generate(i: &DepositInterestInput) -> DepositInterest {
    let years = i.term_months as f64 / 12.0;
    let compounded = i.compounding.trim().eq_ignore_ascii_case("annual");
    let rate = i.annual_rate_pct / 100.0;

    let interest = if compounded {
        cents(i.deposit_usd * ((1.0 + rate).powf(years) - 1.0))
    } else {
        cents(i.deposit_usd * rate * years)
    };
    let total = cents(i.deposit_usd + interest);

    let property = if i.property_label.trim().is_empty() {
        "the Premises".to_string()
    } else {
        i.property_label.trim().to_string()
    };

    let method = if compounded {
        format!("annually compounded at {:.2}%", i.annual_rate_pct)
    } else {
        format!("simple interest at {:.2}%", i.annual_rate_pct)
    };

    let calc_body = format!(
        "On a deposit of {} held for {} months ({:.2} years), {} yields interest of {}. The total returnable to the Tenant, before any lawful deductions, is {}.",
        money(i.deposit_usd),
        i.term_months,
        years,
        method,
        money(interest),
        money(total)
    );

    let citation = i.statute_citation.trim();
    let governing = if citation.is_empty() {
        format!("Interest is computed under the laws of the State of {}.", i.state)
    } else {
        format!("Interest is computed under the laws of the State of {} ({}).", i.state, citation)
    };

    let clauses = vec![
        DocClause {
            heading: "Parties".into(),
            body: format!(
                "Landlord: {}\nTenant: {}\nPremises: {}\nStatement date: {}",
                i.landlord_name, i.tenant_name, property, i.date
            ),
        },
        DocClause {
            heading: "1. Deposit".into(),
            body: format!("The Landlord holds a security deposit of {} for the tenancy.", money(i.deposit_usd)),
        },
        DocClause { heading: "2. Interest Calculation".into(), body: calc_body },
        DocClause {
            heading: "3. Payment".into(),
            body: format!(
                "Interest of {} is credited to the Tenant. It is paid or credited as required by law — at the end of the tenancy, annually, or applied to rent — and is in addition to the deposit principal.",
                money(interest)
            ),
        },
        DocClause { heading: "4. Governing Law".into(), body: governing },
        DocClause {
            heading: "Signatures".into(),
            body: format!(
                "Landlord: ____________________  Date: __________\n{}",
                i.landlord_name
            ),
        },
    ];

    DepositInterest {
        title: "Security Deposit Interest Statement".into(),
        deposit_usd: cents(i.deposit_usd),
        years: (years * 10_000.0).round() / 10_000.0,
        compounded,
        interest_usd: interest,
        total_returnable_usd: total,
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

    fn base() -> DepositInterestInput {
        DepositInterestInput {
            landlord_name: "Maple Apartments LLC".into(),
            tenant_name: "Resident Tenant".into(),
            property_label: "Unit 4B".into(),
            deposit_usd: 2_000.0,
            annual_rate_pct: 1.5,
            term_months: 36,
            compounding: "simple".into(),
            date: "2026-07-01".into(),
            state: "Illinois".into(),
            statute_citation: String::new(),
        }
    }

    #[test]
    fn simple_interest() {
        let d = generate(&base());
        // 2,000 × 1.5% × 3 = 90.
        assert!(close(d.interest_usd, 90.0));
        assert!(close(d.total_returnable_usd, 2_090.0));
        assert!(!d.compounded);
        assert!(close(d.years, 3.0));
    }

    #[test]
    fn annual_compound_interest() {
        let d = generate(&DepositInterestInput { compounding: "annual".into(), ..base() });
        // 2,000 × (1.015^3 − 1) = 91.36.
        assert!(close(d.interest_usd, 91.36));
        assert!(close(d.total_returnable_usd, 2_091.36));
        assert!(d.compounded);
    }

    #[test]
    fn partial_year_simple() {
        let d = generate(&DepositInterestInput { term_months: 18, ..base() });
        // 2,000 × 1.5% × 1.5 = 45.
        assert!(close(d.interest_usd, 45.0));
    }

    #[test]
    fn zero_rate_no_interest() {
        let d = generate(&DepositInterestInput { annual_rate_pct: 0.0, ..base() });
        assert!(close(d.interest_usd, 0.0));
        assert!(close(d.total_returnable_usd, 2_000.0));
    }

    #[test]
    fn interest_added_to_total() {
        let d = generate(&base());
        assert!(close(d.total_returnable_usd, d.deposit_usd + d.interest_usd));
    }

    #[test]
    fn statute_citation_echoed() {
        let d = generate(&DepositInterestInput { statute_citation: "765 ILCS 715".into(), ..base() });
        assert_eq!(d.statutory_citation, "765 ILCS 715");
        assert!(d.clauses.iter().any(|c| c.body.contains("765 ILCS 715")));
    }
}
