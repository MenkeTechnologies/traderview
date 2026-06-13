//! Sublease agreement — the original tenant (sublessor) rents the premises to a
//! subtenant (sublessee) while remaining liable to the landlord under the master
//! lease. It computes the sublease end date from the start plus the term and the
//! markup/discount of the sublease rent versus the original rent, then assembles
//! the agreement. Drafting aid, not legal advice — most leases require the
//! landlord's written consent to sublet.

use chrono::{Duration, Months, NaiveDate};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct SubleaseInput {
    pub sublessor_name: String,
    pub sublessee_name: String,
    pub landlord_name: String,
    pub premises_address: String,
    pub sublease_start_date: String,
    pub term_months: u32,
    pub monthly_rent_usd: f64,
    /// Rent the sublessor pays the landlord under the master lease (context).
    pub original_rent_usd: f64,
    #[serde(default)]
    pub security_deposit_usd: f64,
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
pub struct SubleaseAgreement {
    pub title: String,
    pub sublease_start_date: String,
    pub sublease_end_date: String,
    pub term_months: u32,
    pub monthly_rent_usd: f64,
    pub original_rent_usd: f64,
    /// Sublease rent − original rent (positive = markup).
    pub rent_difference_usd: f64,
    pub rent_difference_pct: f64,
    pub security_deposit_usd: f64,
    pub statutory_citation: String,
    pub clauses: Vec<DocClause>,
}

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

pub fn generate(i: &SubleaseInput) -> SubleaseAgreement {
    let end_date = NaiveDate::parse_from_str(&i.sublease_start_date, "%Y-%m-%d")
        .ok()
        .and_then(|d| d.checked_add_months(Months::new(i.term_months)))
        .map(|d| (d - Duration::days(1)).format("%Y-%m-%d").to_string())
        .unwrap_or_default();

    let rent_diff = i.monthly_rent_usd - i.original_rent_usd;
    let rent_diff_pct = if i.original_rent_usd != 0.0 {
        rent_diff / i.original_rent_usd * 100.0
    } else {
        0.0
    };

    let citation = i.statute_citation.trim();
    let pursuant = if citation.is_empty() {
        format!(
            "This sublease is governed by the landlord-tenant law of the State of {}.",
            i.state
        )
    } else {
        format!(
            "This sublease is governed by the landlord-tenant law of the State of {} ({}).",
            i.state, citation
        )
    };

    let rent_note = if rent_diff.abs() < 0.005 {
        String::new()
    } else if rent_diff > 0.0 {
        format!(" This is {} ({:.2}%) above the master-lease rent of {}.", money(rent_diff), rent_diff_pct, money(i.original_rent_usd))
    } else {
        format!(" This is {} ({:.2}%) below the master-lease rent of {}.", money(-rent_diff), rent_diff_pct, money(i.original_rent_usd))
    };

    let deposit_body = if i.security_deposit_usd > 0.0 {
        format!(
            "The Sublessee shall pay a security deposit of {} to the Sublessor, to be returned at the end of the sublease less any lawful deductions.",
            money(i.security_deposit_usd)
        )
    } else {
        "No security deposit is required under this sublease.".to_string()
    };

    let clauses = vec![
        DocClause {
            heading: "Parties".into(),
            body: format!(
                "Sublessor (original tenant): {}\nSublessee (subtenant): {}\nLandlord: {}\nPremises: {}",
                i.sublessor_name, i.sublessee_name, i.landlord_name, i.premises_address
            ),
        },
        DocClause {
            heading: "1. Premises and Term".into(),
            body: format!(
                "The Sublessor sublets the premises to the Sublessee for a term of {} months, beginning {} and ending {}.",
                i.term_months, i.sublease_start_date, end_date
            ),
        },
        DocClause {
            heading: "2. Rent".into(),
            body: format!(
                "The Sublessee shall pay rent of {} per month to the Sublessor.{}",
                money(i.monthly_rent_usd), rent_note
            ),
        },
        DocClause { heading: "3. Security Deposit".into(), body: deposit_body },
        DocClause {
            heading: "4. Master Lease".into(),
            body: "This sublease is subject to the master lease between the Sublessor and the Landlord. The Sublessee agrees to comply with all terms of the master lease. The Sublessor remains liable to the Landlord for performance of the master lease, and this sublease is effective only with the Landlord's written consent where required.".into(),
        },
        DocClause { heading: "5. Governing Law".into(), body: pursuant },
        DocClause {
            heading: "Signatures".into(),
            body: format!(
                "Sublessor: ____________________  Date: __________\n{}\n\nSublessee: ____________________  Date: __________\n{}\n\nLandlord consent: ____________________  Date: __________\n{}",
                i.sublessor_name, i.sublessee_name, i.landlord_name
            ),
        },
    ];

    SubleaseAgreement {
        title: "Sublease Agreement".into(),
        sublease_start_date: i.sublease_start_date.clone(),
        sublease_end_date: end_date,
        term_months: i.term_months,
        monthly_rent_usd: i.monthly_rent_usd,
        original_rent_usd: i.original_rent_usd,
        rent_difference_usd: rent_diff,
        rent_difference_pct: rent_diff_pct,
        security_deposit_usd: i.security_deposit_usd,
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

    fn base() -> SubleaseInput {
        SubleaseInput {
            sublessor_name: "Jane Doe".into(),
            sublessee_name: "Sam Sub".into(),
            landlord_name: "Acme Property Mgmt".into(),
            premises_address: "42 Rental Rd".into(),
            sublease_start_date: "2026-08-01".into(),
            term_months: 6,
            monthly_rent_usd: 1600.0,
            original_rent_usd: 1500.0,
            security_deposit_usd: 1600.0,
            state: "New York".into(),
            statute_citation: String::new(),
        }
    }

    #[test]
    fn end_date_is_start_plus_term_minus_day() {
        // 2026-08-01 + 6 months − 1 day = 2027-01-31.
        assert_eq!(generate(&base()).sublease_end_date, "2027-01-31");
    }

    #[test]
    fn rent_markup_computed() {
        let d = generate(&base());
        assert!(close(d.rent_difference_usd, 100.0));
        assert!((d.rent_difference_pct - 6.6667).abs() < 1e-3);
        let c = d.clauses.iter().find(|c| c.heading == "2. Rent").unwrap();
        assert!(c.body.contains("above the master-lease rent"));
    }

    #[test]
    fn rent_discount_labeled_below() {
        let d = generate(&SubleaseInput { monthly_rent_usd: 1400.0, ..base() });
        assert!(close(d.rent_difference_usd, -100.0));
        let c = d.clauses.iter().find(|c| c.heading == "2. Rent").unwrap();
        assert!(c.body.contains("below the master-lease rent"));
    }

    #[test]
    fn no_deposit_states_none() {
        let d = generate(&SubleaseInput { security_deposit_usd: 0.0, ..base() });
        let c = d.clauses.iter().find(|c| c.heading.contains("Security Deposit")).unwrap();
        assert!(c.body.contains("No security deposit"));
    }

    #[test]
    fn master_lease_clause_keeps_sublessor_liable() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading == "4. Master Lease").unwrap();
        assert!(c.body.contains("Sublessor remains liable"));
        assert!(c.body.contains("written consent"));
    }

    #[test]
    fn statute_citation_echoed() {
        let d = generate(&SubleaseInput { statute_citation: "N.Y. Real Prop. § 226-b".into(), ..base() });
        assert_eq!(d.statutory_citation, "N.Y. Real Prop. § 226-b");
        assert!(d.clauses.iter().any(|c| c.body.contains("N.Y. Real Prop. § 226-b")));
    }

    #[test]
    fn bad_date_yields_empty_end() {
        let d = generate(&SubleaseInput { sublease_start_date: "x".into(), ..base() });
        assert_eq!(d.sublease_end_date, "");
    }
}
