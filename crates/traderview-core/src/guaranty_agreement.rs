//! Lease guaranty / co-signer agreement — a guarantor (often a parent or
//! principal) unconditionally guarantees a tenant's obligations under a lease.
//! It computes the total rent over the lease term as a measure of the guaranteed
//! exposure and assembles the guaranty clauses. Drafting aid, not legal advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct GuarantyInput {
    pub guarantor_name: String,
    pub tenant_name: String,
    pub landlord_name: String,
    pub premises_address: String,
    pub monthly_rent_usd: f64,
    pub lease_term_months: u32,
    pub lease_start_date: String,
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
pub struct GuarantyAgreement {
    pub title: String,
    pub monthly_rent_usd: f64,
    pub lease_term_months: u32,
    /// Rent over the full term — a measure of the guaranteed exposure.
    pub total_rent_over_term_usd: f64,
    pub statutory_citation: String,
    pub clauses: Vec<DocClause>,
}

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

pub fn generate(i: &GuarantyInput) -> GuarantyAgreement {
    let total_rent = i.monthly_rent_usd * i.lease_term_months as f64;

    let citation = i.statute_citation.trim();
    let pursuant = if citation.is_empty() {
        format!("This guaranty is governed by the laws of the State of {}.", i.state)
    } else {
        format!("This guaranty is governed by the laws of the State of {} ({}).", i.state, citation)
    };

    let clauses = vec![
        DocClause {
            heading: "Parties".into(),
            body: format!(
                "Guarantor: {}\nTenant: {}\nLandlord: {}\nPremises: {}",
                i.guarantor_name, i.tenant_name, i.landlord_name, i.premises_address
            ),
        },
        DocClause {
            heading: "1. Guaranty".into(),
            body: format!(
                "In consideration of the Landlord leasing the premises to the Tenant, the Guarantor unconditionally and absolutely guarantees to the Landlord the full and timely performance of all of the Tenant's obligations under the lease dated {}, including payment of rent of {} per month and any damages, late fees, and costs.",
                i.lease_start_date, money(i.monthly_rent_usd)
            ),
        },
        DocClause {
            heading: "2. Scope".into(),
            body: format!(
                "Over the {}-month term, the rent alone totals {}. The Guarantor's liability is not limited to that amount and extends to all sums and obligations the Tenant owes under the lease, including any renewal or holdover.",
                i.lease_term_months, money(total_rent)
            ),
        },
        DocClause {
            heading: "3. Continuing and Absolute".into(),
            body: "This is a continuing guaranty. The Landlord may proceed against the Guarantor without first pursuing the Tenant. The Guarantor's obligations are not affected by any amendment of the lease, extension of time, or other indulgence granted to the Tenant.".into(),
        },
        DocClause {
            heading: "4. Waiver".into(),
            body: "The Guarantor waives notice of acceptance of this guaranty, notice of default, demand for payment, and presentment, and waives any requirement that the Landlord exhaust remedies against the Tenant first.".into(),
        },
        DocClause { heading: "5. Governing Law".into(), body: pursuant },
        DocClause {
            heading: "Signature".into(),
            body: format!("Guarantor: ____________________  Date: __________\n{}", i.guarantor_name),
        },
    ];

    GuarantyAgreement {
        title: "Lease Guaranty (Co-Signer) Agreement".into(),
        monthly_rent_usd: i.monthly_rent_usd,
        lease_term_months: i.lease_term_months,
        total_rent_over_term_usd: total_rent,
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

    fn base() -> GuarantyInput {
        GuarantyInput {
            guarantor_name: "Pat Parent".into(),
            tenant_name: "Sam Student".into(),
            landlord_name: "Campus Rentals".into(),
            premises_address: "5 College Ave, Apt 2".into(),
            monthly_rent_usd: 1500.0,
            lease_term_months: 12,
            lease_start_date: "2026-09-01".into(),
            state: "Massachusetts".into(),
            statute_citation: String::new(),
        }
    }

    #[test]
    fn total_rent_over_term() {
        // 1,500 × 12 = 18,000.
        assert!(close(generate(&base()).total_rent_over_term_usd, 18_000.0));
    }

    #[test]
    fn scope_clause_states_total_and_unlimited() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading == "2. Scope").unwrap();
        assert!(c.body.contains("$18000.00"));
        assert!(c.body.contains("not limited to that amount"));
    }

    #[test]
    fn guaranty_is_unconditional() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading == "1. Guaranty").unwrap();
        assert!(c.body.contains("unconditionally and absolutely guarantees"));
    }

    #[test]
    fn waiver_clause_present() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading == "4. Waiver").unwrap();
        assert!(c.body.contains("waives"));
    }

    #[test]
    fn statute_citation_echoed() {
        let d = generate(&GuarantyInput { statute_citation: "Mass. Gen. Laws ch. 186".into(), ..base() });
        assert_eq!(d.statutory_citation, "Mass. Gen. Laws ch. 186");
        assert!(d.clauses.iter().any(|c| c.body.contains("Mass. Gen. Laws ch. 186")));
    }
}
