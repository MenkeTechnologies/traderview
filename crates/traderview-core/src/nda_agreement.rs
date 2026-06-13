//! Non-disclosure agreement (NDA) — protects confidential information shared
//! between parties, one-way (one discloser) or mutual (both disclose). It
//! computes the expiration date from the effective date plus the term and
//! assembles the operative clauses (purpose, definition, obligations,
//! exclusions, term, return of materials, no license, governing law), adapting
//! the wording to mutual vs one-way. Drafting aid, not legal advice.

use chrono::{Months, NaiveDate};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct NdaInput {
    pub disclosing_party: String,
    pub receiving_party: String,
    /// True = both parties disclose and are bound; false = one-way.
    #[serde(default)]
    pub mutual: bool,
    pub purpose: String,
    pub effective_date: String,
    pub term_years: u32,
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
pub struct NdaAgreement {
    pub title: String,
    pub effective_date: String,
    pub expiration_date: String,
    pub term_years: u32,
    pub mutual: bool,
    pub statutory_citation: String,
    pub clauses: Vec<DocClause>,
}

pub fn generate(i: &NdaInput) -> NdaAgreement {
    let expiration_date = NaiveDate::parse_from_str(&i.effective_date, "%Y-%m-%d")
        .ok()
        .and_then(|d| d.checked_add_months(Months::new(i.term_years.saturating_mul(12))))
        .map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_default();

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

    let parties_body = if i.mutual {
        format!(
            "This is a MUTUAL non-disclosure agreement between {} and {}. Each party may disclose Confidential Information to the other, and each is bound as a Receiving Party with respect to information it receives.",
            i.disclosing_party, i.receiving_party
        )
    } else {
        format!(
            "Disclosing Party: {}\nReceiving Party: {}\nThis is a one-way agreement: only the Receiving Party is bound to protect the Disclosing Party's Confidential Information.",
            i.disclosing_party, i.receiving_party
        )
    };

    let obligations_body = if i.mutual {
        "Each party shall hold the other party's Confidential Information in strict confidence, use it solely for the Purpose, disclose it only to representatives with a need to know who are bound by like obligations, and protect it with at least reasonable care.".to_string()
    } else {
        "The Receiving Party shall hold the Confidential Information in strict confidence, use it solely for the Purpose, disclose it only to representatives with a need to know who are bound by like obligations, and protect it with at least reasonable care.".to_string()
    };

    let clauses = vec![
        DocClause { heading: "Parties".into(), body: parties_body },
        DocClause {
            heading: "1. Purpose".into(),
            body: format!(
                "The parties wish to explore the following purpose (the \"Purpose\") and, in connection with it, may disclose Confidential Information: {}.",
                i.purpose
            ),
        },
        DocClause {
            heading: "2. Confidential Information".into(),
            body: "\"Confidential Information\" means all non-public information disclosed by one party to the other, in any form, that is marked or would reasonably be understood to be confidential, including business, technical, financial, and customer information.".into(),
        },
        DocClause { heading: "3. Obligations".into(), body: obligations_body },
        DocClause {
            heading: "4. Exclusions".into(),
            body: "Confidential Information does not include information that: (a) is or becomes publicly available through no fault of the Receiving Party; (b) was rightfully known before disclosure; (c) is rightfully received from a third party without restriction; or (d) is independently developed without use of the Confidential Information.".into(),
        },
        DocClause {
            heading: "5. Term".into(),
            body: format!(
                "This Agreement is effective {} and continues for {} year(s), expiring on {}. The confidentiality obligations survive expiration with respect to Confidential Information disclosed during the term.",
                i.effective_date, i.term_years, expiration_date
            ),
        },
        DocClause {
            heading: "6. Return of Materials".into(),
            body: "Upon written request or termination, the Receiving Party shall promptly return or destroy all Confidential Information and copies in its possession.".into(),
        },
        DocClause {
            heading: "7. No License".into(),
            body: "No license or other right to any intellectual property is granted by this Agreement. All Confidential Information remains the property of the disclosing party.".into(),
        },
        DocClause { heading: "8. Governing Law".into(), body: pursuant },
        DocClause {
            heading: "Signatures".into(),
            body: format!(
                "{}: ____________________  Date: __________\n\n{}: ____________________  Date: __________",
                i.disclosing_party, i.receiving_party
            ),
        },
    ];

    NdaAgreement {
        title: if i.mutual {
            "Mutual Non-Disclosure Agreement".into()
        } else {
            "Non-Disclosure Agreement".into()
        },
        effective_date: i.effective_date.clone(),
        expiration_date,
        term_years: i.term_years,
        mutual: i.mutual,
        statutory_citation: citation.to_string(),
        clauses,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> NdaInput {
        NdaInput {
            disclosing_party: "Acme Inc".into(),
            receiving_party: "Beta LLC".into(),
            mutual: false,
            purpose: "Evaluating a potential business relationship".into(),
            effective_date: "2026-01-01".into(),
            term_years: 3,
            governing_state: "New York".into(),
            statute_citation: String::new(),
        }
    }

    #[test]
    fn expiration_is_effective_plus_term() {
        // 2026-01-01 + 3 years = 2029-01-01.
        assert_eq!(generate(&base()).expiration_date, "2029-01-01");
    }

    #[test]
    fn one_way_title_and_obligations() {
        let d = generate(&base());
        assert_eq!(d.title, "Non-Disclosure Agreement");
        let c = d.clauses.iter().find(|c| c.heading == "3. Obligations").unwrap();
        assert!(c.body.starts_with("The Receiving Party"));
    }

    #[test]
    fn mutual_title_and_obligations() {
        let d = generate(&NdaInput { mutual: true, ..base() });
        assert_eq!(d.title, "Mutual Non-Disclosure Agreement");
        let c = d.clauses.iter().find(|c| c.heading == "3. Obligations").unwrap();
        assert!(c.body.starts_with("Each party"));
        let p = d.clauses.iter().find(|c| c.heading == "Parties").unwrap();
        assert!(p.body.contains("MUTUAL"));
    }

    #[test]
    fn exclusions_and_no_license_present() {
        let d = generate(&base());
        assert!(d.clauses.iter().any(|c| c.heading == "4. Exclusions"));
        assert!(d.clauses.iter().any(|c| c.heading == "7. No License"));
    }

    #[test]
    fn purpose_in_clause() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading == "1. Purpose").unwrap();
        assert!(c.body.contains("Evaluating a potential business relationship"));
    }

    #[test]
    fn statute_citation_echoed() {
        let d = generate(&NdaInput { statute_citation: "N.Y. Gen. Oblig.".into(), ..base() });
        assert_eq!(d.statutory_citation, "N.Y. Gen. Oblig.");
        assert!(d.clauses.iter().any(|c| c.body.contains("N.Y. Gen. Oblig.")));
    }

    #[test]
    fn bad_date_yields_empty_expiration() {
        let d = generate(&NdaInput { effective_date: "x".into(), ..base() });
        assert_eq!(d.expiration_date, "");
    }
}
