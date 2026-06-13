//! Lease renewal / extension agreement — extends an existing tenancy for a new
//! fixed term, optionally at an adjusted rent. It computes the new lease end
//! date (renewal start + term, ending the day before the next period) and the
//! rent change versus the expiring rent, then assembles the agreement.
//! Drafting aid, not legal advice.

use chrono::{Duration, Months, NaiveDate};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct RenewalInput {
    pub landlord_name: String,
    pub landlord_address: String,
    #[serde(default)]
    pub landlord_phone: String,
    pub tenant_name: String,
    pub premises_address: String,
    /// First day of the renewal term (YYYY-MM-DD).
    pub renewal_start_date: String,
    pub term_months: u32,
    pub current_rent_usd: f64,
    /// Rent for the renewal term (equal to current = no change).
    pub new_rent_usd: f64,
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
pub struct RenewalAgreement {
    pub title: String,
    pub renewal_start_date: String,
    pub renewal_end_date: String,
    pub term_months: u32,
    pub current_rent_usd: f64,
    pub new_rent_usd: f64,
    pub rent_change_usd: f64,
    pub rent_change_pct: f64,
    pub statutory_citation: String,
    pub clauses: Vec<DocClause>,
}

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

pub fn generate(i: &RenewalInput) -> RenewalAgreement {
    // End date: start + term months, less one day (a 12-month term beginning
    // Aug 1 ends Jul 31 of the next year).
    let end_date = NaiveDate::parse_from_str(&i.renewal_start_date, "%Y-%m-%d")
        .ok()
        .and_then(|d| d.checked_add_months(Months::new(i.term_months)))
        .map(|d| (d - Duration::days(1)).format("%Y-%m-%d").to_string())
        .unwrap_or_default();

    let rent_change = i.new_rent_usd - i.current_rent_usd;
    let rent_change_pct = if i.current_rent_usd != 0.0 {
        rent_change / i.current_rent_usd * 100.0
    } else {
        0.0
    };

    let citation = i.statute_citation.trim();
    let pursuant = if citation.is_empty() {
        format!(
            "This agreement shall be governed by the landlord-tenant law of the State of {}.",
            i.state
        )
    } else {
        format!(
            "This agreement shall be governed by the landlord-tenant law of the State of {} ({}).",
            i.state, citation
        )
    };

    let rent_body = if rent_change.abs() < 0.005 {
        format!(
            "The monthly rent for the renewal term remains {}, unchanged from the prior term.",
            money(i.new_rent_usd)
        )
    } else {
        let dir = if rent_change > 0.0 { "increase" } else { "decrease" };
        format!(
            "The monthly rent for the renewal term is {} — {} of {} ({:.2}%) from the prior rent of {}.",
            money(i.new_rent_usd),
            dir,
            money(rent_change.abs()),
            rent_change_pct,
            money(i.current_rent_usd)
        )
    };

    let clauses = vec![
        DocClause {
            heading: "Parties".into(),
            body: format!(
                "Landlord: {}\nTenant: {}\nPremises: {}",
                i.landlord_name, i.tenant_name, i.premises_address
            ),
        },
        DocClause {
            heading: "1. Renewal Term".into(),
            body: format!(
                "The lease for the above premises is renewed for a term of {} months, beginning {} and ending {}.",
                i.term_months, i.renewal_start_date, end_date
            ),
        },
        DocClause { heading: "2. Rent".into(), body: rent_body },
        DocClause {
            heading: "3. Other Terms".into(),
            body: "All other terms, covenants, and conditions of the original lease remain in full force and effect except as expressly modified by this renewal.".into(),
        },
        DocClause { heading: "4. Governing Law".into(), body: pursuant },
        DocClause {
            heading: "Signatures".into(),
            body: format!(
                "Landlord: ____________________  Date: __________\n{}\n{}{}\n\nTenant: ____________________  Date: __________\n{}",
                i.landlord_name,
                i.landlord_address,
                if i.landlord_phone.is_empty() {
                    String::new()
                } else {
                    format!("\nTelephone: {}", i.landlord_phone)
                },
                i.tenant_name
            ),
        },
    ];

    RenewalAgreement {
        title: "Lease Renewal / Extension Agreement".into(),
        renewal_start_date: i.renewal_start_date.clone(),
        renewal_end_date: end_date,
        term_months: i.term_months,
        current_rent_usd: i.current_rent_usd,
        new_rent_usd: i.new_rent_usd,
        rent_change_usd: rent_change,
        rent_change_pct,
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

    fn base() -> RenewalInput {
        RenewalInput {
            landlord_name: "Acme Property Mgmt".into(),
            landlord_address: "1 Main St".into(),
            landlord_phone: String::new(),
            tenant_name: "Jane Doe".into(),
            premises_address: "42 Rental Rd".into(),
            renewal_start_date: "2026-08-01".into(),
            term_months: 12,
            current_rent_usd: 1500.0,
            new_rent_usd: 1575.0,
            state: "Ohio".into(),
            statute_citation: String::new(),
        }
    }

    #[test]
    fn end_date_is_start_plus_term_minus_one_day() {
        // 2026-08-01 + 12 months − 1 day = 2027-07-31.
        assert_eq!(generate(&base()).renewal_end_date, "2027-07-31");
    }

    #[test]
    fn rent_change_computed() {
        let r = generate(&base());
        assert!(close(r.rent_change_usd, 75.0));
        assert!(close(r.rent_change_pct, 5.0));
    }

    #[test]
    fn unchanged_rent_states_unchanged() {
        let r = generate(&RenewalInput { new_rent_usd: 1500.0, ..base() });
        assert!(close(r.rent_change_usd, 0.0));
        let c = r.clauses.iter().find(|c| c.heading == "2. Rent").unwrap();
        assert!(c.body.contains("unchanged"));
    }

    #[test]
    fn rent_decrease_labeled() {
        let r = generate(&RenewalInput { new_rent_usd: 1400.0, ..base() });
        assert!(close(r.rent_change_usd, -100.0));
        let c = r.clauses.iter().find(|c| c.heading == "2. Rent").unwrap();
        assert!(c.body.contains("decrease"));
    }

    #[test]
    fn short_month_term_end() {
        // 6-month term from Sep 1 → ends Feb 28 (2027 not a leap year).
        let r = generate(&RenewalInput {
            renewal_start_date: "2026-09-01".into(),
            term_months: 6,
            ..base()
        });
        assert_eq!(r.renewal_end_date, "2027-02-28");
    }

    #[test]
    fn statute_citation_echoed() {
        let r = generate(&RenewalInput {
            statute_citation: "Ohio Rev. Code § 5321".into(),
            ..base()
        });
        assert_eq!(r.statutory_citation, "Ohio Rev. Code § 5321");
        assert!(r.clauses.iter().any(|c| c.body.contains("Ohio Rev. Code § 5321")));
    }

    #[test]
    fn bad_date_yields_empty_end() {
        let r = generate(&RenewalInput { renewal_start_date: "x".into(), ..base() });
        assert_eq!(r.renewal_end_date, "");
    }
}
