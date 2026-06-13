//! Employment offer letter — extends a job offer with title, compensation, and
//! start date. It breaks the annual salary into the per-paycheck amount for the
//! chosen pay frequency and assembles the offer clauses (position, compensation,
//! equity, benefits, at-will, contingencies). Drafting aid, not legal advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PayFrequency {
    Weekly,
    Biweekly,
    Semimonthly,
    Monthly,
}

impl PayFrequency {
    fn periods_per_year(self) -> i32 {
        match self {
            PayFrequency::Weekly => 52,
            PayFrequency::Biweekly => 26,
            PayFrequency::Semimonthly => 24,
            PayFrequency::Monthly => 12,
        }
    }
    fn label(self) -> &'static str {
        match self {
            PayFrequency::Weekly => "weekly",
            PayFrequency::Biweekly => "biweekly",
            PayFrequency::Semimonthly => "semimonthly",
            PayFrequency::Monthly => "monthly",
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct OfferInput {
    pub company_name: String,
    pub candidate_name: String,
    pub job_title: String,
    pub annual_salary_usd: f64,
    pub pay_frequency: PayFrequency,
    pub start_date: String,
    #[serde(default)]
    pub signing_bonus_usd: f64,
    #[serde(default)]
    pub equity_description: String,
    /// FLSA-exempt (salaried) vs non-exempt (overtime-eligible).
    #[serde(default)]
    pub exempt: bool,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DocClause {
    pub heading: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct OfferLetter {
    pub title: String,
    pub annual_salary_usd: f64,
    pub periods_per_year: i32,
    pub per_paycheck_usd: f64,
    pub signing_bonus_usd: f64,
    pub clauses: Vec<DocClause>,
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

pub fn generate(i: &OfferInput) -> OfferLetter {
    let ppy = i.pay_frequency.periods_per_year();
    let per_paycheck = cents(i.annual_salary_usd / ppy as f64);

    let bonus_part = if i.signing_bonus_usd > 0.0 {
        format!(" You will also receive a one-time signing bonus of {}.", money(i.signing_bonus_usd))
    } else {
        String::new()
    };

    let exempt_part = if i.exempt {
        " This is a salaried, exempt position and is not eligible for overtime."
    } else {
        " This is a non-exempt position and is eligible for overtime as required by law."
    };

    let clauses = vec![
        DocClause {
            heading: "1. Position".into(),
            body: format!(
                "{} is pleased to offer {} the position of {}, beginning {}.{}",
                i.company_name, i.candidate_name, i.job_title, i.start_date, exempt_part
            ),
        },
        DocClause {
            heading: "2. Compensation".into(),
            body: format!(
                "Your annual base salary is {}, paid {} ({} per paycheck across {} pay periods per year).{}",
                money(i.annual_salary_usd),
                i.pay_frequency.label(),
                money(per_paycheck),
                ppy,
                bonus_part
            ),
        },
        DocClause {
            heading: "3. Equity".into(),
            body: if i.equity_description.trim().is_empty() {
                "No equity is included in this offer.".to_string()
            } else {
                format!("Subject to board approval and the company's plan, you will be granted: {}.", i.equity_description.trim())
            },
        },
        DocClause {
            heading: "4. Benefits".into(),
            body: "You will be eligible to participate in the company's standard benefit programs (health, retirement, paid time off) in accordance with their terms.".into(),
        },
        DocClause {
            heading: "5. At-Will Employment".into(),
            body: format!(
                "Your employment with {} is at-will under the laws of the State of {}, meaning either you or the company may terminate the employment relationship at any time, with or without cause or notice.",
                i.company_name, i.state
            ),
        },
        DocClause {
            heading: "6. Contingencies".into(),
            body: "This offer is contingent on your eligibility to work in the United States (Form I-9 verification) and the satisfactory completion of any background check the company requires.".into(),
        },
        DocClause {
            heading: "Acceptance".into(),
            body: format!(
                "Candidate signature: ____________________  Date: __________\n{}\n\nCompany representative: ____________________  Date: __________\n{}",
                i.candidate_name, i.company_name
            ),
        },
    ];

    OfferLetter {
        title: "Employment Offer Letter".into(),
        annual_salary_usd: i.annual_salary_usd,
        periods_per_year: ppy,
        per_paycheck_usd: per_paycheck,
        signing_bonus_usd: i.signing_bonus_usd,
        clauses,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    fn base() -> OfferInput {
        OfferInput {
            company_name: "Acme Inc".into(),
            candidate_name: "Jordan Hire".into(),
            job_title: "Software Engineer".into(),
            annual_salary_usd: 120_000.0,
            pay_frequency: PayFrequency::Biweekly,
            start_date: "2026-08-01".into(),
            signing_bonus_usd: 10_000.0,
            equity_description: "5,000 ISOs vesting over 4 years".into(),
            exempt: true,
            state: "California".into(),
        }
    }

    #[test]
    fn biweekly_per_paycheck() {
        let d = generate(&base());
        assert_eq!(d.periods_per_year, 26);
        assert!(close(d.per_paycheck_usd, 4_615.38));
    }

    #[test]
    fn weekly_and_monthly() {
        assert!(close(generate(&OfferInput { pay_frequency: PayFrequency::Weekly, ..base() }).per_paycheck_usd, 2_307.69));
        assert!(close(generate(&OfferInput { pay_frequency: PayFrequency::Monthly, ..base() }).per_paycheck_usd, 10_000.0));
        assert!(close(generate(&OfferInput { pay_frequency: PayFrequency::Semimonthly, ..base() }).per_paycheck_usd, 5_000.0));
    }

    #[test]
    fn signing_bonus_in_clause() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading == "2. Compensation").unwrap();
        assert!(c.body.contains("signing bonus of $10000.00"));
    }

    #[test]
    fn no_bonus_omits_it() {
        let c = generate(&OfferInput { signing_bonus_usd: 0.0, ..base() })
            .clauses.into_iter().find(|c| c.heading == "2. Compensation").unwrap();
        assert!(!c.body.contains("signing bonus"));
    }

    #[test]
    fn exempt_vs_nonexempt() {
        assert!(generate(&base()).clauses.iter().find(|c| c.heading == "1. Position").unwrap().body.contains("not eligible for overtime"));
        let ne = generate(&OfferInput { exempt: false, ..base() });
        assert!(ne.clauses.iter().find(|c| c.heading == "1. Position").unwrap().body.contains("eligible for overtime"));
    }

    #[test]
    fn equity_clause_optional() {
        assert!(generate(&base()).clauses.iter().find(|c| c.heading == "3. Equity").unwrap().body.contains("5,000 ISOs"));
        let none = generate(&OfferInput { equity_description: String::new(), ..base() });
        assert!(none.clauses.iter().find(|c| c.heading == "3. Equity").unwrap().body.contains("No equity"));
    }

    #[test]
    fn at_will_names_state() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading.contains("At-Will")).unwrap();
        assert!(c.body.contains("California"));
        assert!(c.body.contains("at-will"));
    }
}
