//! Independent contractor agreement (1099) — engages a contractor for services
//! as a non-employee. It states the fee (fixed or hourly, with an estimated
//! total when hours are given and net-N payment terms) and assembles the
//! operative clauses: services, compensation, term, independent-contractor
//! status (the contractor is responsible for their own taxes and receives a
//! 1099), confidentiality, work-product assignment, termination, and governing
//! law. Drafting aid, not legal advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeeType {
    Fixed,
    Hourly,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ContractorInput {
    pub client_name: String,
    #[serde(default)]
    pub client_address: String,
    pub contractor_name: String,
    #[serde(default)]
    pub contractor_address: String,
    pub services_description: String,
    pub fee_type: FeeType,
    /// Fixed total (Fixed) or hourly rate (Hourly).
    pub fee_amount_usd: f64,
    /// Estimated hours, used to project a total for hourly engagements.
    #[serde(default)]
    pub estimated_hours: f64,
    pub payment_terms_days: i64,
    pub start_date: String,
    /// End date, or empty for an ongoing/at-will engagement.
    #[serde(default)]
    pub end_date: String,
    pub governing_state: String,
    #[serde(default)]
    pub statute_citation: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DocClause {
    pub heading: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ContractorAgreement {
    pub title: String,
    pub fee_amount_usd: f64,
    /// Projected total: the fixed fee, or rate × estimated hours (0 if unknown).
    pub estimated_total_usd: f64,
    pub payment_terms_days: i64,
    pub statutory_citation: String,
    pub clauses: Vec<DocClause>,
}

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

pub fn generate(i: &ContractorInput) -> ContractorAgreement {
    let estimated_total = match i.fee_type {
        FeeType::Fixed => i.fee_amount_usd,
        FeeType::Hourly => {
            if i.estimated_hours > 0.0 {
                i.fee_amount_usd * i.estimated_hours
            } else {
                0.0
            }
        }
    };

    let compensation = match i.fee_type {
        FeeType::Fixed => format!(
            "The Client shall pay the Contractor a fixed fee of {} for the services described above. Invoices are due net {} days from the invoice date.",
            money(i.fee_amount_usd),
            i.payment_terms_days
        ),
        FeeType::Hourly => {
            let est = if i.estimated_hours > 0.0 {
                format!(
                    ", estimated at {:.1} hours (approximately {})",
                    i.estimated_hours,
                    money(estimated_total)
                )
            } else {
                String::new()
            };
            format!(
                "The Client shall pay the Contractor {} per hour for the services described above{}. Invoices are due net {} days from the invoice date.",
                money(i.fee_amount_usd),
                est,
                i.payment_terms_days
            )
        }
    };

    let term_body = if i.end_date.trim().is_empty() {
        format!(
            "This Agreement begins on {} and continues until the services are completed or the Agreement is terminated as provided below.",
            i.start_date
        )
    } else {
        format!(
            "This Agreement begins on {} and ends on {}, unless terminated earlier as provided below.",
            i.start_date,
            i.end_date.trim()
        )
    };

    let citation = i.statute_citation.trim();
    let pursuant = if citation.is_empty() {
        format!(
            "This Agreement shall be governed by the laws of the State of {}.",
            i.governing_state
        )
    } else {
        format!(
            "This Agreement shall be governed by the laws of the State of {} ({}).",
            i.governing_state, citation
        )
    };

    let client_line = if i.client_address.trim().is_empty() {
        i.client_name.clone()
    } else {
        format!("{}, {}", i.client_name, i.client_address.trim())
    };
    let contractor_line = if i.contractor_address.trim().is_empty() {
        i.contractor_name.clone()
    } else {
        format!("{}, {}", i.contractor_name, i.contractor_address.trim())
    };

    let clauses = vec![
        DocClause {
            heading: "Parties".into(),
            body: format!("Client: {}\nContractor: {}", client_line, contractor_line),
        },
        DocClause {
            heading: "1. Services".into(),
            body: format!(
                "The Contractor shall perform the following services for the Client: {}.",
                i.services_description
            ),
        },
        DocClause { heading: "2. Compensation".into(), body: compensation },
        DocClause { heading: "3. Term".into(), body: term_body },
        DocClause {
            heading: "4. Independent Contractor Status".into(),
            body: "The Contractor is an independent contractor and not an employee, partner, or agent of the Client. The Contractor is solely responsible for all federal, state, and local taxes on amounts paid under this Agreement, will receive an IRS Form 1099 where required, and is not entitled to any employee benefits. The Contractor controls the manner and means of performing the services and may provide services to others.".into(),
        },
        DocClause {
            heading: "5. Confidentiality".into(),
            body: "The Contractor shall keep confidential all non-public information of the Client obtained in connection with the services and shall not disclose or use it except to perform this Agreement.".into(),
        },
        DocClause {
            heading: "6. Work Product".into(),
            body: "All deliverables and work product created by the Contractor specifically for the Client under this Agreement shall, upon full payment, be the property of the Client, and the Contractor assigns to the Client all rights, title, and interest therein.".into(),
        },
        DocClause {
            heading: "7. Termination".into(),
            body: "Either party may terminate this Agreement upon written notice. Upon termination, the Client shall pay the Contractor for all services satisfactorily performed through the termination date.".into(),
        },
        DocClause { heading: "8. Governing Law".into(), body: pursuant },
        DocClause {
            heading: "Signatures".into(),
            body: format!(
                "Client: ____________________  Date: __________\n{}\n\nContractor: ____________________  Date: __________\n{}",
                i.client_name, i.contractor_name
            ),
        },
    ];

    ContractorAgreement {
        title: "Independent Contractor Agreement".into(),
        fee_amount_usd: i.fee_amount_usd,
        estimated_total_usd: estimated_total,
        payment_terms_days: i.payment_terms_days,
        statutory_citation: citation.to_string(),
        clauses,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6
    }

    fn base() -> ContractorInput {
        ContractorInput {
            client_name: "Acme Inc".into(),
            client_address: "1 Commerce St".into(),
            contractor_name: "Pat Freelancer".into(),
            contractor_address: "5 Maker Way".into(),
            services_description: "Design and build a marketing website".into(),
            fee_type: FeeType::Fixed,
            fee_amount_usd: 5000.0,
            estimated_hours: 0.0,
            payment_terms_days: 30,
            start_date: "2026-07-01".into(),
            end_date: "2026-09-30".into(),
            governing_state: "Delaware".into(),
            statute_citation: String::new(),
        }
    }

    #[test]
    fn fixed_total_is_fee() {
        assert!(close(generate(&base()).estimated_total_usd, 5000.0));
    }

    #[test]
    fn hourly_total_is_rate_times_hours() {
        let a = generate(&ContractorInput {
            fee_type: FeeType::Hourly,
            fee_amount_usd: 100.0,
            estimated_hours: 40.0,
            ..base()
        });
        assert!(close(a.estimated_total_usd, 4000.0));
        let c = a.clauses.iter().find(|c| c.heading == "2. Compensation").unwrap();
        assert!(c.body.contains("$100.00 per hour"));
        assert!(c.body.contains("approximately $4000.00"));
    }

    #[test]
    fn hourly_without_hours_has_no_estimate() {
        let a = generate(&ContractorInput {
            fee_type: FeeType::Hourly,
            fee_amount_usd: 100.0,
            estimated_hours: 0.0,
            ..base()
        });
        assert!(close(a.estimated_total_usd, 0.0));
        let c = a.clauses.iter().find(|c| c.heading == "2. Compensation").unwrap();
        assert!(!c.body.contains("estimated at"));
    }

    #[test]
    fn status_clause_has_1099_and_tax_language() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading.contains("Independent Contractor Status")).unwrap();
        assert!(c.body.contains("1099"));
        assert!(c.body.contains("responsible for all federal, state, and local taxes"));
        assert!(c.body.contains("not entitled to any employee benefits"));
    }

    #[test]
    fn work_product_assigned_on_payment() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading == "6. Work Product").unwrap();
        assert!(c.body.contains("upon full payment"));
        assert!(c.body.contains("assigns"));
    }

    #[test]
    fn ongoing_term_when_no_end_date() {
        let c = generate(&ContractorInput { end_date: String::new(), ..base() })
            .clauses
            .into_iter()
            .find(|c| c.heading == "3. Term")
            .unwrap();
        assert!(c.body.contains("until the services are completed"));
    }

    #[test]
    fn net_terms_in_compensation() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading == "2. Compensation").unwrap();
        assert!(c.body.contains("net 30 days"));
    }

    #[test]
    fn statute_citation_echoed() {
        let a = generate(&ContractorInput {
            statute_citation: "Del. Code tit. 19".into(),
            ..base()
        });
        assert_eq!(a.statutory_citation, "Del. Code tit. 19");
        assert!(a.clauses.iter().any(|c| c.body.contains("Del. Code tit. 19")));
    }
}
