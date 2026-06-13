//! Pet addendum — amends an existing lease to permit a pet on stated terms. It
//! totals the up-front charges (refundable pet deposit + non-refundable pet fee)
//! and adds any monthly pet rent to the base rent for a new monthly total, then
//! assembles the addendum clauses. Drafting aid, not legal advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct PetAddendumInput {
    pub landlord_name: String,
    pub tenant_name: String,
    pub premises_address: String,
    /// Date of the original lease this addendum amends (echoed).
    pub lease_date: String,
    pub pet_description: String,
    /// Refundable pet deposit (one-time).
    #[serde(default)]
    pub pet_deposit_usd: f64,
    /// Non-refundable pet fee (one-time).
    #[serde(default)]
    pub pet_fee_usd: f64,
    #[serde(default)]
    pub monthly_pet_rent_usd: f64,
    pub current_monthly_rent_usd: f64,
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
pub struct PetAddendum {
    pub title: String,
    pub pet_deposit_usd: f64,
    pub pet_fee_usd: f64,
    pub total_upfront_usd: f64,
    pub monthly_pet_rent_usd: f64,
    pub new_monthly_rent_usd: f64,
    pub statutory_citation: String,
    pub clauses: Vec<DocClause>,
}

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

pub fn generate(i: &PetAddendumInput) -> PetAddendum {
    let total_upfront = i.pet_deposit_usd + i.pet_fee_usd;
    let new_monthly_rent = i.current_monthly_rent_usd + i.monthly_pet_rent_usd;

    let citation = i.statute_citation.trim();
    let pursuant = if citation.is_empty() {
        format!(
            "This addendum is governed by the landlord-tenant law of the State of {} and forms part of the lease.",
            i.state
        )
    } else {
        format!(
            "This addendum is governed by the landlord-tenant law of the State of {} ({}) and forms part of the lease.",
            i.state, citation
        )
    };

    // Charges clause — describe deposit, fee, and total as applicable.
    let mut charge_parts: Vec<String> = Vec::new();
    if i.pet_deposit_usd > 0.0 {
        charge_parts.push(format!("a refundable pet deposit of {}", money(i.pet_deposit_usd)));
    }
    if i.pet_fee_usd > 0.0 {
        charge_parts.push(format!("a non-refundable pet fee of {}", money(i.pet_fee_usd)));
    }
    let charges_body = if charge_parts.is_empty() {
        "No up-front pet deposit or fee is charged.".to_string()
    } else {
        format!(
            "The Tenant shall pay {} (total up-front: {}). The refundable portion is held with the security deposit and may be applied to pet-related damage.",
            charge_parts.join(" and "),
            money(total_upfront)
        )
    };

    let rent_body = if i.monthly_pet_rent_usd > 0.0 {
        format!(
            "Monthly pet rent of {} is added to the base rent of {}, for a new total monthly rent of {}.",
            money(i.monthly_pet_rent_usd),
            money(i.current_monthly_rent_usd),
            money(new_monthly_rent)
        )
    } else {
        format!(
            "No monthly pet rent is charged; the monthly rent remains {}.",
            money(i.current_monthly_rent_usd)
        )
    };

    let clauses = vec![
        DocClause {
            heading: "Parties".into(),
            body: format!(
                "Landlord: {}\nTenant: {}\nPremises: {}\nThis addendum amends the lease dated {}.",
                i.landlord_name, i.tenant_name, i.premises_address, i.lease_date
            ),
        },
        DocClause {
            heading: "1. Permitted Pet".into(),
            body: format!(
                "Notwithstanding any no-pets provision in the lease, the Landlord permits the Tenant to keep the following pet at the premises: {}. No other animals are permitted without further written consent.",
                i.pet_description
            ),
        },
        DocClause { heading: "2. Pet Deposit and Fee".into(), body: charges_body },
        DocClause { heading: "3. Monthly Pet Rent".into(), body: rent_body },
        DocClause {
            heading: "4. Tenant Responsibilities".into(),
            body: "The Tenant is responsible for all damage caused by the pet, shall keep it from disturbing neighbors, shall promptly remove pet waste, shall keep the pet licensed and vaccinated as required by law, and is liable for any injury the pet causes.".into(),
        },
        DocClause {
            heading: "5. Revocation".into(),
            body: "The Landlord may revoke this permission upon reasonable written notice if the pet causes damage, a nuisance, or a threat to health or safety, after which the Tenant must remove the pet.".into(),
        },
        DocClause { heading: "6. Governing Law".into(), body: pursuant },
        DocClause {
            heading: "Signatures".into(),
            body: format!(
                "Landlord: ____________________  Date: __________\n{}\n\nTenant: ____________________  Date: __________\n{}",
                i.landlord_name, i.tenant_name
            ),
        },
    ];

    PetAddendum {
        title: "Pet Addendum to Lease".into(),
        pet_deposit_usd: i.pet_deposit_usd,
        pet_fee_usd: i.pet_fee_usd,
        total_upfront_usd: total_upfront,
        monthly_pet_rent_usd: i.monthly_pet_rent_usd,
        new_monthly_rent_usd: new_monthly_rent,
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

    fn base() -> PetAddendumInput {
        PetAddendumInput {
            landlord_name: "Acme Property Mgmt".into(),
            tenant_name: "Jane Doe".into(),
            premises_address: "42 Rental Rd".into(),
            lease_date: "2026-01-01".into(),
            pet_description: "1 dog, Labrador, 'Rex', 60 lbs".into(),
            pet_deposit_usd: 300.0,
            pet_fee_usd: 200.0,
            monthly_pet_rent_usd: 50.0,
            current_monthly_rent_usd: 1500.0,
            state: "Texas".into(),
            statute_citation: String::new(),
        }
    }

    #[test]
    fn upfront_and_new_rent() {
        let d = generate(&base());
        assert!(close(d.total_upfront_usd, 500.0));
        assert!(close(d.new_monthly_rent_usd, 1550.0));
    }

    #[test]
    fn charges_clause_lists_deposit_and_fee() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading.contains("Pet Deposit and Fee")).unwrap();
        assert!(c.body.contains("refundable pet deposit of $300.00"));
        assert!(c.body.contains("non-refundable pet fee of $200.00"));
        assert!(c.body.contains("$500.00"));
    }

    #[test]
    fn no_charges_states_none() {
        let d = generate(&PetAddendumInput { pet_deposit_usd: 0.0, pet_fee_usd: 0.0, ..base() });
        assert!(close(d.total_upfront_usd, 0.0));
        let c = d.clauses.iter().find(|c| c.heading.contains("Pet Deposit and Fee")).unwrap();
        assert!(c.body.contains("No up-front"));
    }

    #[test]
    fn no_pet_rent_keeps_base() {
        let d = generate(&PetAddendumInput { monthly_pet_rent_usd: 0.0, ..base() });
        assert!(close(d.new_monthly_rent_usd, 1500.0));
        let c = d.clauses.iter().find(|c| c.heading.contains("Monthly Pet Rent")).unwrap();
        assert!(c.body.contains("No monthly pet rent"));
    }

    #[test]
    fn permitted_pet_in_clause() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading.contains("Permitted Pet")).unwrap();
        assert!(c.body.contains("Labrador"));
    }

    #[test]
    fn statute_citation_echoed() {
        let d = generate(&PetAddendumInput { statute_citation: "Tex. Prop. Code § 92".into(), ..base() });
        assert_eq!(d.statutory_citation, "Tex. Prop. Code § 92");
        assert!(d.clauses.iter().any(|c| c.body.contains("Tex. Prop. Code § 92")));
    }
}
