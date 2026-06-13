//! Severance agreement — separates an employee with a severance payment in
//! exchange for a release of claims. It computes the severance pay from the
//! weeks offered and the weekly salary, adds any accrued-PTO payout for the
//! total, and assembles the release clauses — including the ADEA consideration
//! and revocation windows when the employee is 40 or older. Drafting aid, not
//! legal advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct SeveranceInput {
    pub company_name: String,
    pub employee_name: String,
    pub job_title: String,
    pub annual_salary_usd: f64,
    pub severance_weeks: f64,
    pub separation_date: String,
    #[serde(default)]
    pub accrued_pto_payout_usd: f64,
    /// Employee is 40 or older — triggers ADEA (OWBPA) review/revocation terms.
    #[serde(default)]
    pub age_40_or_over: bool,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DocClause {
    pub heading: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SeveranceAgreement {
    pub title: String,
    pub weekly_pay_usd: f64,
    pub severance_weeks: f64,
    pub severance_pay_usd: f64,
    pub accrued_pto_payout_usd: f64,
    pub total_payout_usd: f64,
    pub clauses: Vec<DocClause>,
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

pub fn generate(i: &SeveranceInput) -> SeveranceAgreement {
    let weekly_pay = cents(i.annual_salary_usd / 52.0);
    let severance_pay = cents(weekly_pay * i.severance_weeks);
    let total_payout = cents(severance_pay + i.accrued_pto_payout_usd);

    let pto_part = if i.accrued_pto_payout_usd > 0.0 {
        format!(" In addition, the Company will pay accrued, unused PTO of {}, for a total payment of {}.", money(i.accrued_pto_payout_usd), money(total_payout))
    } else {
        String::new()
    };

    let mut clauses = vec![
        DocClause {
            heading: "1. Separation".into(),
            body: format!(
                "{}'s employment with {} as {} ends effective {}. This Agreement sets the terms of separation.",
                i.employee_name, i.company_name, i.job_title, i.separation_date
            ),
        },
        DocClause {
            heading: "2. Severance Pay".into(),
            body: format!(
                "In exchange for the release below, the Company will pay severance of {} weeks at a weekly rate of {}, totaling {} (less applicable withholding).{}",
                i.severance_weeks, money(weekly_pay), money(severance_pay), pto_part
            ),
        },
        DocClause {
            heading: "3. Release of Claims".into(),
            body: "In consideration of the severance, the Employee releases the Company and its affiliates from all claims arising from the employment or its termination, to the fullest extent permitted by law. This release does not waive claims that cannot be waived by law.".into(),
        },
    ];

    let mut next = 4;
    if i.age_40_or_over {
        clauses.push(DocClause {
            heading: format!("{next}. Age Discrimination (ADEA/OWBPA)"),
            body: "Because the Employee is 40 or older, the Employee is advised to consult an attorney, is given at least 21 days to consider this Agreement, and may revoke it within 7 days after signing. The Agreement is not effective or enforceable until the revocation period expires.".to_string(),
        });
        next += 1;
    }

    clauses.push(DocClause {
        heading: format!("{next}. Confidentiality and Non-Disparagement"),
        body: "The Employee shall keep the terms of this Agreement confidential except as required by law, and the parties shall not disparage one another.".into(),
    });
    next += 1;

    clauses.push(DocClause {
        heading: format!("{next}. Governing Law"),
        body: format!("This Agreement is governed by the laws of the State of {}.", i.state),
    });

    clauses.push(DocClause {
        heading: "Signatures".into(),
        body: format!(
            "Employee: ____________________  Date: __________\n{}\n\nCompany representative: ____________________  Date: __________\n{}",
            i.employee_name, i.company_name
        ),
    });

    SeveranceAgreement {
        title: "Severance Agreement and Release".into(),
        weekly_pay_usd: weekly_pay,
        severance_weeks: i.severance_weeks,
        severance_pay_usd: severance_pay,
        accrued_pto_payout_usd: i.accrued_pto_payout_usd,
        total_payout_usd: total_payout,
        clauses,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    fn base() -> SeveranceInput {
        SeveranceInput {
            company_name: "Acme Inc".into(),
            employee_name: "Chris Exit".into(),
            job_title: "Analyst".into(),
            annual_salary_usd: 104_000.0,
            severance_weeks: 8.0,
            separation_date: "2026-06-30".into(),
            accrued_pto_payout_usd: 1_000.0,
            age_40_or_over: false,
            state: "Illinois".into(),
        }
    }

    #[test]
    fn severance_and_total() {
        let d = generate(&base());
        assert!(close(d.weekly_pay_usd, 2_000.0));
        assert!(close(d.severance_pay_usd, 16_000.0));
        assert!(close(d.total_payout_usd, 17_000.0));
    }

    #[test]
    fn pto_part_in_clause() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading == "2. Severance Pay").unwrap();
        assert!(c.body.contains("accrued, unused PTO of $1000.00"));
        assert!(c.body.contains("total payment of $17000.00"));
    }

    #[test]
    fn no_pto_omits_part_and_total_is_severance() {
        let d = generate(&SeveranceInput { accrued_pto_payout_usd: 0.0, ..base() });
        assert!(close(d.total_payout_usd, 16_000.0));
        let c = d.clauses.iter().find(|c| c.heading == "2. Severance Pay").unwrap();
        assert!(!c.body.contains("PTO"));
    }

    #[test]
    fn adea_clause_only_when_40_plus() {
        let under = generate(&base());
        assert!(!under.clauses.iter().any(|c| c.heading.contains("Age Discrimination")));
        let over = generate(&SeveranceInput { age_40_or_over: true, ..base() });
        let c = over.clauses.iter().find(|c| c.heading.contains("Age Discrimination")).unwrap();
        assert!(c.body.contains("21 days"));
        assert!(c.body.contains("7 days"));
    }

    #[test]
    fn clause_numbering_sequential_with_adea() {
        let over = generate(&SeveranceInput { age_40_or_over: true, ..base() });
        assert!(over.clauses.iter().any(|c| c.heading == "4. Age Discrimination (ADEA/OWBPA)"));
        assert!(over.clauses.iter().any(|c| c.heading == "5. Confidentiality and Non-Disparagement"));
        assert!(over.clauses.iter().any(|c| c.heading == "6. Governing Law"));
    }

    #[test]
    fn release_clause_present() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading.contains("Release of Claims")).unwrap();
        assert!(c.body.contains("releases the Company"));
    }
}
