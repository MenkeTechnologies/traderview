//! Cease and desist letter — a formal demand that the recipient stop a
//! specified conduct (harassment, IP infringement, defamation, breach, etc.).
//! Distinct from a demand for payment: the remedy sought is stopping an action,
//! not paying money. It computes the comply-by date from the service date plus
//! the response window and assembles the letter. Drafting aid, not legal advice.

use chrono::{Duration, NaiveDate};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct CeaseDesistInput {
    pub sender_name: String,
    #[serde(default)]
    pub sender_address: String,
    pub recipient_name: String,
    /// The conduct the recipient must stop.
    pub conduct_description: String,
    /// The legal basis (e.g. "trademark infringement", "harassment").
    pub legal_basis: String,
    /// Date the letter is served (YYYY-MM-DD).
    pub served_date: String,
    /// Days the recipient has to comply.
    pub response_days: i64,
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
pub struct CeaseDesistLetter {
    pub title: String,
    pub comply_by_date: String,
    pub response_days: i64,
    pub statutory_citation: String,
    pub clauses: Vec<DocClause>,
}

pub fn generate(i: &CeaseDesistInput) -> CeaseDesistLetter {
    let comply_by = NaiveDate::parse_from_str(&i.served_date, "%Y-%m-%d")
        .map(|d| (d + Duration::days(i.response_days)).format("%Y-%m-%d").to_string())
        .unwrap_or_default();

    let citation = i.statute_citation.trim();
    let pursuant = if citation.is_empty() {
        format!("This letter and any subsequent action are made under the laws of the State of {}.", i.state)
    } else {
        format!("This letter and any subsequent action are made under the laws of the State of {} ({}).", i.state, citation)
    };

    let clauses = vec![
        DocClause { heading: "To".into(), body: i.recipient_name.clone() },
        DocClause {
            heading: "1. Conduct Complained Of".into(),
            body: format!(
                "It has come to the attention of {} that you have engaged in the following conduct: {}.",
                i.sender_name, i.conduct_description
            ),
        },
        DocClause {
            heading: "2. Legal Basis".into(),
            body: format!(
                "This conduct constitutes, or gives rise to a claim for, {}. You have no right to continue it.",
                i.legal_basis
            ),
        },
        DocClause {
            heading: "3. Demand to Cease and Desist".into(),
            body: format!(
                "You are hereby demanded to immediately CEASE AND DESIST the conduct described above and to confirm in writing, on or before {} ({} days from the date of this letter), that you have done so and will not resume it.",
                comply_by, i.response_days
            ),
        },
        DocClause {
            heading: "4. Consequences".into(),
            body: "If you do not comply by the date stated above, the undersigned may pursue all available legal remedies, including an action for injunctive relief and damages, without further notice. This letter is sent without waiver of any rights or remedies.".into(),
        },
        DocClause { heading: "5. Governing Law".into(), body: pursuant },
        DocClause {
            heading: "Signature".into(),
            body: format!(
                "Signature: ____________________  Date: {}\n{}{}",
                i.served_date,
                i.sender_name,
                if i.sender_address.trim().is_empty() {
                    String::new()
                } else {
                    format!("\n{}", i.sender_address.trim())
                }
            ),
        },
    ];

    CeaseDesistLetter {
        title: "Cease and Desist Letter".into(),
        comply_by_date: comply_by,
        response_days: i.response_days,
        statutory_citation: citation.to_string(),
        clauses,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> CeaseDesistInput {
        CeaseDesistInput {
            sender_name: "Acme Inc".into(),
            sender_address: "1 Commerce St".into(),
            recipient_name: "John Infringer".into(),
            conduct_description: "use of the ACME mark on competing products".into(),
            legal_basis: "trademark infringement".into(),
            served_date: "2026-06-01".into(),
            response_days: 14,
            state: "New York".into(),
            statute_citation: String::new(),
        }
    }

    #[test]
    fn comply_by_is_served_plus_response() {
        // 2026-06-01 + 14 days = 2026-06-15.
        assert_eq!(generate(&base()).comply_by_date, "2026-06-15");
    }

    #[test]
    fn conduct_and_basis_in_clauses() {
        let d = generate(&base());
        assert!(d.clauses.iter().find(|c| c.heading.contains("Conduct")).unwrap().body.contains("ACME mark"));
        assert!(d.clauses.iter().find(|c| c.heading == "2. Legal Basis").unwrap().body.contains("trademark infringement"));
    }

    #[test]
    fn demand_clause_has_deadline() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading.contains("Demand")).unwrap();
        assert!(c.body.contains("CEASE AND DESIST"));
        assert!(c.body.contains("2026-06-15"));
    }

    #[test]
    fn consequences_clause_present() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading == "4. Consequences").unwrap();
        assert!(c.body.contains("injunctive relief"));
    }

    #[test]
    fn statute_citation_echoed() {
        let d = generate(&CeaseDesistInput { statute_citation: "15 U.S.C. § 1114".into(), ..base() });
        assert_eq!(d.statutory_citation, "15 U.S.C. § 1114");
        assert!(d.clauses.iter().any(|c| c.body.contains("15 U.S.C. § 1114")));
    }

    #[test]
    fn bad_date_yields_empty_deadline() {
        let d = generate(&CeaseDesistInput { served_date: "x".into(), ..base() });
        assert_eq!(d.comply_by_date, "");
    }
}
