//! Lead-based paint disclosure — the disclosure a landlord (or seller) of
//! pre-1978 "target housing" must give before a lease or sale under federal law
//! (42 U.S.C. § 4852d; 24 CFR 35 / 40 CFR 745). It determines whether the
//! disclosure is required from the year built and assembles the lessor's
//! disclosure, the lead-pamphlet acknowledgment, and the certifications.
//! Drafting aid, not legal advice.

use serde::{Deserialize, Serialize};

/// Housing built before this year is federal "target housing" requiring the
/// lead-based paint disclosure.
pub const TARGET_HOUSING_CUTOFF_YEAR: i32 = 1978;

#[derive(Debug, Clone, Deserialize)]
pub struct LeadPaintInput {
    pub landlord_name: String,
    pub tenant_name: String,
    pub premises_address: String,
    pub year_built: i32,
    /// Lessor has knowledge of lead-based paint / hazards in the housing.
    #[serde(default)]
    pub known_lead_present: bool,
    /// Description of the known lead-based paint / hazards (if any).
    #[serde(default)]
    pub lead_details: String,
    /// Lessor has reports or records available to the lessee.
    #[serde(default)]
    pub records_available: bool,
    #[serde(default)]
    pub records_description: String,
    pub disclosure_date: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DocClause {
    pub heading: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct LeadPaintDisclosure {
    pub title: String,
    pub year_built: i32,
    /// True when the housing is pre-1978 target housing (disclosure required).
    pub disclosure_required: bool,
    pub clauses: Vec<DocClause>,
}

pub fn generate(i: &LeadPaintInput) -> LeadPaintDisclosure {
    let required = i.year_built < TARGET_HOUSING_CUTOFF_YEAR;

    let applicability = if required {
        format!(
            "The premises were built in {} — before {}, making this federal \"target housing.\" The lessor must provide this lead-based paint disclosure and the EPA pamphlet before the lessee is obligated under the lease.",
            i.year_built, TARGET_HOUSING_CUTOFF_YEAR
        )
    } else {
        format!(
            "The premises were built in {}, on or after {}. Such housing is exempt from the federal lead-based paint disclosure requirement; this form is provided for the parties' records only.",
            i.year_built, TARGET_HOUSING_CUTOFF_YEAR
        )
    };

    let knowledge_body = if i.known_lead_present {
        let details = if i.lead_details.trim().is_empty() {
            String::new()
        } else {
            format!(" Known lead-based paint and/or hazards: {}.", i.lead_details.trim())
        };
        format!(
            "(a) The Lessor has knowledge of lead-based paint and/or lead-based paint hazards in the housing.{}",
            details
        )
    } else {
        "(a) The Lessor has no knowledge of lead-based paint and/or lead-based paint hazards in the housing.".to_string()
    };

    let records_body = if i.records_available {
        let desc = if i.records_description.trim().is_empty() {
            "The Lessor has provided the Lessee with all available records and reports.".to_string()
        } else {
            format!("The Lessor has provided the following records and reports to the Lessee: {}.", i.records_description.trim())
        };
        format!("(b) {}", desc)
    } else {
        "(b) The Lessor has no reports or records pertaining to lead-based paint and/or hazards in the housing.".to_string()
    };

    let clauses = vec![
        DocClause {
            heading: "Parties and Premises".into(),
            body: format!(
                "Lessor: {}\nLessee: {}\nPremises: {}\nYear built: {}",
                i.landlord_name, i.tenant_name, i.premises_address, i.year_built
            ),
        },
        DocClause { heading: "1. Applicability".into(), body: applicability },
        DocClause {
            heading: "2. Lessor's Disclosure".into(),
            body: format!("{}\n{}", knowledge_body, records_body),
        },
        DocClause {
            heading: "3. Lessee's Acknowledgment".into(),
            body: "The Lessee has received copies of all information listed above and has received the federally approved pamphlet \"Protect Your Family from Lead in Your Home.\"".into(),
        },
        DocClause {
            heading: "4. Certification of Accuracy".into(),
            body: format!(
                "The parties have reviewed the information above and certify, to the best of their knowledge, that it is true and accurate as of {}.",
                i.disclosure_date
            ),
        },
        DocClause {
            heading: "Signatures".into(),
            body: format!(
                "Lessor: ____________________  Date: {}\n{}\n\nLessee: ____________________  Date: {}\n{}",
                i.disclosure_date, i.landlord_name, i.disclosure_date, i.tenant_name
            ),
        },
    ];

    LeadPaintDisclosure {
        title: "Disclosure of Information on Lead-Based Paint and Hazards".into(),
        year_built: i.year_built,
        disclosure_required: required,
        clauses,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> LeadPaintInput {
        LeadPaintInput {
            landlord_name: "Acme Property Mgmt".into(),
            tenant_name: "Jane Doe".into(),
            premises_address: "42 Old Rd".into(),
            year_built: 1965,
            known_lead_present: false,
            lead_details: String::new(),
            records_available: false,
            records_description: String::new(),
            disclosure_date: "2026-06-01".into(),
        }
    }

    #[test]
    fn pre_1978_requires_disclosure() {
        let d = generate(&base());
        assert!(d.disclosure_required);
        let c = d.clauses.iter().find(|c| c.heading == "1. Applicability").unwrap();
        assert!(c.body.contains("target housing"));
        assert!(c.body.contains("must provide"));
    }

    #[test]
    fn post_1978_exempt() {
        let d = generate(&LeadPaintInput { year_built: 1990, ..base() });
        assert!(!d.disclosure_required);
        let c = d.clauses.iter().find(|c| c.heading == "1. Applicability").unwrap();
        assert!(c.body.contains("exempt"));
    }

    #[test]
    fn exactly_1978_is_exempt() {
        // Cutoff is "before 1978", so 1978 itself is not target housing.
        assert!(!generate(&LeadPaintInput { year_built: 1978, ..base() }).disclosure_required);
        assert!(generate(&LeadPaintInput { year_built: 1977, ..base() }).disclosure_required);
    }

    #[test]
    fn known_lead_with_details() {
        let d = generate(&LeadPaintInput {
            known_lead_present: true,
            lead_details: "peeling paint in basement".into(),
            ..base()
        });
        let c = d.clauses.iter().find(|c| c.heading.contains("Lessor's Disclosure")).unwrap();
        assert!(c.body.contains("has knowledge of lead-based paint"));
        assert!(c.body.contains("peeling paint in basement"));
    }

    #[test]
    fn no_knowledge_default() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading.contains("Lessor's Disclosure")).unwrap();
        assert!(c.body.contains("no knowledge"));
        assert!(c.body.contains("no reports or records"));
    }

    #[test]
    fn records_listed_when_available() {
        let d = generate(&LeadPaintInput {
            records_available: true,
            records_description: "2019 inspection report".into(),
            ..base()
        });
        let c = d.clauses.iter().find(|c| c.heading.contains("Lessor's Disclosure")).unwrap();
        assert!(c.body.contains("2019 inspection report"));
    }

    #[test]
    fn acknowledgment_mentions_pamphlet() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading.contains("Acknowledgment")).unwrap();
        assert!(c.body.contains("Protect Your Family from Lead in Your Home"));
    }
}
