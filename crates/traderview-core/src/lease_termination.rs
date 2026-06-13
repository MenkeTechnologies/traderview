//! Lease termination letter — the written notice either party gives to end a
//! periodic or fixed tenancy. It computes the termination/move-out date from the
//! service date plus the notice period and assembles the letter, with wording
//! that adapts to whether the landlord or the tenant is giving notice. Drafting
//! aid, not legal advice.

use chrono::{Duration, NaiveDate};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SenderRole {
    Landlord,
    Tenant,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TerminationInput {
    pub sender_role: SenderRole,
    pub sender_name: String,
    #[serde(default)]
    pub sender_address: String,
    #[serde(default)]
    pub sender_phone: String,
    pub recipient_name: String,
    pub premises_address: String,
    /// Date the notice is served (YYYY-MM-DD).
    pub served_date: String,
    /// Days of notice before termination (per state / lease minimum).
    pub notice_days: i64,
    /// Optional reason (landlord notices may state cause).
    #[serde(default)]
    pub reason: String,
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
pub struct TerminationLetter {
    pub title: String,
    pub termination_date: String,
    pub notice_days: i64,
    pub statutory_citation: String,
    pub clauses: Vec<DocClause>,
}

pub fn generate(i: &TerminationInput) -> TerminationLetter {
    let termination_date = NaiveDate::parse_from_str(&i.served_date, "%Y-%m-%d")
        .map(|d| (d + Duration::days(i.notice_days)).format("%Y-%m-%d").to_string())
        .unwrap_or_default();

    let citation = i.statute_citation.trim();
    let pursuant = if citation.is_empty() {
        format!(
            "This notice is given pursuant to the landlord-tenant law of the State of {}.",
            i.state
        )
    } else {
        format!(
            "This notice is given pursuant to the landlord-tenant law of the State of {} ({}).",
            i.state, citation
        )
    };

    let reason_suffix = if i.reason.trim().is_empty() {
        String::new()
    } else {
        format!(" The reason for this termination is: {}.", i.reason.trim())
    };

    let notice_body = match i.sender_role {
        SenderRole::Tenant => format!(
            "I, {}, am giving notice to terminate my tenancy at the premises located at {}. I will vacate and return possession on or before {} ({} days from the date of this notice).",
            i.sender_name, i.premises_address, termination_date, i.notice_days
        ),
        SenderRole::Landlord => format!(
            "You are hereby notified that your tenancy at the premises located at {} is terminated effective {} ({} days from the date of this notice). You must vacate and deliver up possession on or before that date.{}",
            i.premises_address, termination_date, i.notice_days, reason_suffix
        ),
    };

    let closing_body = match i.sender_role {
        SenderRole::Tenant => "Please provide instructions for the return of my security deposit and confirm the move-out inspection date. I will return all keys upon vacating.".to_string(),
        SenderRole::Landlord => "Please arrange to return all keys and provide a forwarding address for the return of any security deposit due. A move-out inspection may be scheduled before you vacate.".to_string(),
    };

    let clauses = vec![
        DocClause { heading: "To".into(), body: i.recipient_name.clone() },
        DocClause { heading: "1. Notice of Termination".into(), body: notice_body },
        DocClause { heading: "2. Move-Out".into(), body: closing_body },
        DocClause { heading: "3. Governing Law".into(), body: pursuant },
        DocClause {
            heading: "Signature".into(),
            body: format!(
                "Signature: ____________________  Date: {}\n{}\n{}{}",
                i.served_date,
                i.sender_name,
                i.sender_address,
                if i.sender_phone.is_empty() {
                    String::new()
                } else {
                    format!("\nTelephone: {}", i.sender_phone)
                }
            ),
        },
    ];

    let title = match i.sender_role {
        SenderRole::Tenant => "Notice of Lease Termination (Tenant)",
        SenderRole::Landlord => "Notice of Lease Termination (Landlord)",
    };

    TerminationLetter {
        title: title.into(),
        termination_date,
        notice_days: i.notice_days,
        statutory_citation: citation.to_string(),
        clauses,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> TerminationInput {
        TerminationInput {
            sender_role: SenderRole::Tenant,
            sender_name: "Jane Doe".into(),
            sender_address: "42 Rental Rd".into(),
            sender_phone: String::new(),
            recipient_name: "Acme Property Mgmt".into(),
            premises_address: "42 Rental Rd".into(),
            served_date: "2026-06-01".into(),
            notice_days: 30,
            reason: String::new(),
            state: "Illinois".into(),
            statute_citation: String::new(),
        }
    }

    #[test]
    fn termination_date_is_served_plus_notice() {
        // 2026-06-01 + 30 days = 2026-07-01.
        assert_eq!(generate(&base()).termination_date, "2026-07-01");
    }

    #[test]
    fn tenant_letter_first_person() {
        let d = generate(&base());
        assert!(d.title.contains("Tenant"));
        let c = d.clauses.iter().find(|c| c.heading.contains("Notice of Termination")).unwrap();
        assert!(c.body.contains("I, Jane Doe, am giving notice"));
    }

    #[test]
    fn landlord_letter_second_person() {
        let d = generate(&TerminationInput { sender_role: SenderRole::Landlord, ..base() });
        assert!(d.title.contains("Landlord"));
        let c = d.clauses.iter().find(|c| c.heading.contains("Notice of Termination")).unwrap();
        assert!(c.body.contains("your tenancy"));
        assert!(c.body.contains("terminated effective"));
    }

    #[test]
    fn landlord_reason_appears() {
        let d = generate(&TerminationInput {
            sender_role: SenderRole::Landlord,
            reason: "End of fixed term".into(),
            ..base()
        });
        let c = d.clauses.iter().find(|c| c.heading.contains("Notice of Termination")).unwrap();
        assert!(c.body.contains("End of fixed term"));
    }

    #[test]
    fn statute_citation_echoed() {
        let d = generate(&TerminationInput { statute_citation: "735 ILCS 5/9-209".into(), ..base() });
        assert_eq!(d.statutory_citation, "735 ILCS 5/9-209");
        assert!(d.clauses.iter().any(|c| c.body.contains("735 ILCS 5/9-209")));
    }

    #[test]
    fn bad_date_yields_empty_termination() {
        let d = generate(&TerminationInput { served_date: "x".into(), ..base() });
        assert_eq!(d.termination_date, "");
    }
}
