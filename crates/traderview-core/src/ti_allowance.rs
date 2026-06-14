//! Tenant-improvement (TI) allowance reconciliation — a landlord funds a tenant's
//! build-out up to an allowance, usually quoted per rentable square foot. At
//! project completion the actual construction cost is reconciled against the
//! allowance: if it runs over, the tenant pays the overage; if it runs under, the
//! unused balance is typically forfeited or credited. This computes the total
//! allowance, the actual cost per square foot, and the overage or unused balance.
//! Distinct from amortizing a TI allowance into rent — this is the cost
//! reconciliation, not a payment schedule. Drafting aid, not legal/accounting advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct TiAllowanceInput {
    pub landlord_name: String,
    pub tenant_name: String,
    #[serde(default)]
    pub property_label: String,
    /// Tenant's rentable square footage.
    pub tenant_sqft: f64,
    /// Allowance per rentable square foot.
    pub allowance_per_sqft_usd: f64,
    /// Actual construction / build-out cost.
    pub actual_cost_usd: f64,
    /// How an unused balance is treated: "forfeited" or "credited" (to rent).
    #[serde(default)]
    pub unused_treatment: String,
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
pub struct TiAllowance {
    pub title: String,
    /// Allowance per sqft × tenant sqft.
    pub total_allowance_usd: f64,
    /// Actual cost ÷ tenant sqft.
    pub actual_cost_per_sqft_usd: f64,
    /// Cost above the allowance the tenant must fund (0 if under).
    pub tenant_overage_usd: f64,
    /// Allowance not spent (0 if over).
    pub unused_allowance_usd: f64,
    pub statutory_citation: String,
    pub clauses: Vec<DocClause>,
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

pub fn generate(i: &TiAllowanceInput) -> TiAllowance {
    let total_allowance = cents(i.allowance_per_sqft_usd * i.tenant_sqft);
    let overage = (i.actual_cost_usd - total_allowance).max(0.0);
    let unused = (total_allowance - i.actual_cost_usd).max(0.0);
    let actual_psf = if i.tenant_sqft > 0.0 {
        cents(i.actual_cost_usd / i.tenant_sqft)
    } else {
        0.0
    };

    let property = if i.property_label.trim().is_empty() {
        "the Premises".to_string()
    } else {
        i.property_label.trim().to_string()
    };

    let credited = i.unused_treatment.trim().eq_ignore_ascii_case("credited");
    let unused_desc = if credited {
        "credited against rent"
    } else {
        "forfeited to the Landlord"
    };

    let outcome = if overage > 0.0 {
        format!("The actual cost exceeds the allowance by {}, which the Tenant shall pay.", money(cents(overage)))
    } else if unused > 0.0 {
        format!("The allowance exceeds the actual cost by {}, which is {}.", money(cents(unused)), unused_desc)
    } else {
        "The actual cost exactly matches the allowance; no overage or unused balance remains.".to_string()
    };

    let calc_body = format!(
        "The TI allowance is {} ({} per rentable sq ft × {:.0} sq ft). Actual construction cost is {} ({} per sq ft). {}",
        money(total_allowance),
        money(i.allowance_per_sqft_usd),
        i.tenant_sqft,
        money(i.actual_cost_usd),
        money(actual_psf),
        outcome
    );

    let citation = i.statute_citation.trim();
    let governing = if citation.is_empty() {
        format!("This reconciliation is governed by the lease and the laws of the State of {}.", i.state)
    } else {
        format!("This reconciliation is governed by the lease and the laws of the State of {} ({}).", i.state, citation)
    };

    let clauses = vec![
        DocClause {
            heading: "Parties".into(),
            body: format!(
                "Landlord: {}\nTenant: {}\nPremises: {}\nDate: {}",
                i.landlord_name, i.tenant_name, property, i.date
            ),
        },
        DocClause {
            heading: "1. Allowance".into(),
            body: format!(
                "The Landlord provides a tenant-improvement allowance of {} per rentable square foot, or {} for the {:.0}-sq-ft Premises.",
                money(i.allowance_per_sqft_usd), money(total_allowance), i.tenant_sqft
            ),
        },
        DocClause { heading: "2. Reconciliation".into(), body: calc_body },
        DocClause {
            heading: "3. Settlement".into(),
            body: if overage > 0.0 {
                format!("The Tenant shall pay the overage of {} within 30 days of this statement.", money(cents(overage)))
            } else if unused > 0.0 {
                format!("The unused allowance of {} is {}.", money(cents(unused)), unused_desc)
            } else {
                "No further amounts are due between the parties for the improvements.".to_string()
            },
        },
        DocClause {
            heading: "4. Records".into(),
            body: "The Tenant shall provide lien waivers and paid invoices supporting the actual construction cost before any allowance is disbursed or overage assessed.".into(),
        },
        DocClause { heading: "5. Governing Law".into(), body: governing },
        DocClause {
            heading: "Signatures".into(),
            body: format!(
                "Landlord: ____________________  Date: __________\n{}\n\nTenant: ____________________  Date: __________\n{}",
                i.landlord_name, i.tenant_name
            ),
        },
    ];

    TiAllowance {
        title: "Tenant Improvement Allowance Reconciliation".into(),
        total_allowance_usd: total_allowance,
        actual_cost_per_sqft_usd: actual_psf,
        tenant_overage_usd: cents(overage),
        unused_allowance_usd: cents(unused),
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

    fn base() -> TiAllowanceInput {
        TiAllowanceInput {
            landlord_name: "Plaza Owners LP".into(),
            tenant_name: "Build-Out Tenant LLC".into(),
            property_label: "Suite 200".into(),
            tenant_sqft: 5_000.0,
            allowance_per_sqft_usd: 50.0,
            actual_cost_usd: 300_000.0,
            unused_treatment: "forfeited".into(),
            date: "2026-07-01".into(),
            state: "Delaware".into(),
            statute_citation: String::new(),
        }
    }

    #[test]
    fn over_allowance_tenant_pays() {
        let d = generate(&base());
        assert!(close(d.total_allowance_usd, 250_000.0));
        assert!(close(d.actual_cost_per_sqft_usd, 60.0));
        assert!(close(d.tenant_overage_usd, 50_000.0));
        assert!(close(d.unused_allowance_usd, 0.0));
    }

    #[test]
    fn under_allowance_unused() {
        let d = generate(&TiAllowanceInput { actual_cost_usd: 200_000.0, ..base() });
        assert!(close(d.unused_allowance_usd, 50_000.0));
        assert!(close(d.tenant_overage_usd, 0.0));
        assert!(close(d.actual_cost_per_sqft_usd, 40.0));
    }

    #[test]
    fn exact_match_no_balance() {
        let d = generate(&TiAllowanceInput { actual_cost_usd: 250_000.0, ..base() });
        assert!(close(d.tenant_overage_usd, 0.0));
        assert!(close(d.unused_allowance_usd, 0.0));
    }

    #[test]
    fn credited_treatment_in_text() {
        let d = generate(&TiAllowanceInput { actual_cost_usd: 200_000.0, unused_treatment: "credited".into(), ..base() });
        assert!(d.clauses.iter().any(|c| c.body.contains("credited against rent")));
    }

    #[test]
    fn forfeited_treatment_in_text() {
        let d = generate(&TiAllowanceInput { actual_cost_usd: 200_000.0, ..base() });
        assert!(d.clauses.iter().any(|c| c.body.contains("forfeited to the Landlord")));
    }

    #[test]
    fn statute_citation_echoed() {
        let d = generate(&TiAllowanceInput { statute_citation: "lease exhibit C".into(), ..base() });
        assert_eq!(d.statutory_citation, "lease exhibit C");
        assert!(d.clauses.iter().any(|c| c.body.contains("lease exhibit C")));
    }
}
