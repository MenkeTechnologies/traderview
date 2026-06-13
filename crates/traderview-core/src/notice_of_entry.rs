//! Notice of entry — the advance notice a landlord must give before entering a
//! tenant's unit (most states require 24–48 hours for non-emergency entry). It
//! computes the earliest lawful entry date from the service date plus the notice
//! period and assembles the notice with the purpose and time window. Drafting
//! aid, not legal advice.

use chrono::{Duration, NaiveDate};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct EntryInput {
    pub landlord_name: String,
    pub landlord_address: String,
    #[serde(default)]
    pub landlord_phone: String,
    pub tenant_name: String,
    pub premises_address: String,
    /// Date the notice is served (YYYY-MM-DD).
    pub served_date: String,
    /// Advance notice required before entry, in days (e.g. 1 = 24h, 2 = 48h).
    pub notice_days: i64,
    /// Reason for entry (repairs, inspection, showing, etc.).
    pub purpose: String,
    /// Window the landlord intends to enter (e.g. "9:00 AM – 12:00 PM").
    #[serde(default)]
    pub time_window: String,
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
pub struct EntryNotice {
    pub title: String,
    pub entry_date: String,
    pub notice_days: i64,
    pub statutory_citation: String,
    pub clauses: Vec<DocClause>,
}

pub fn generate(i: &EntryInput) -> EntryNotice {
    let entry_date = NaiveDate::parse_from_str(&i.served_date, "%Y-%m-%d")
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

    let window = if i.time_window.trim().is_empty() {
        " during normal business hours".to_string()
    } else {
        format!(" during the following time window: {}", i.time_window.trim())
    };

    let clauses = vec![
        DocClause { heading: "To".into(), body: i.tenant_name.clone() },
        DocClause {
            heading: "1. Notice of Entry".into(),
            body: format!(
                "Please be advised that the landlord, {}, or the landlord's agent intends to enter the premises located at {} on or after {}{}.",
                i.landlord_name, i.premises_address, entry_date, window
            ),
        },
        DocClause {
            heading: "2. Purpose".into(),
            body: format!("The purpose of entry is: {}.", i.purpose),
        },
        DocClause {
            heading: "3. Notice Period".into(),
            body: format!(
                "This notice is served on {} and provides {} days' advance notice before entry, as required by law. The tenant need not be present.",
                i.served_date, i.notice_days
            ),
        },
        DocClause { heading: "4. Governing Law".into(), body: pursuant },
        DocClause {
            heading: "Signature".into(),
            body: format!(
                "Signature of owner of premises or agent: ____________________  Date: {}\n{}\n{}{}",
                i.served_date,
                i.landlord_name,
                i.landlord_address,
                if i.landlord_phone.is_empty() {
                    String::new()
                } else {
                    format!("\nTelephone: {}", i.landlord_phone)
                }
            ),
        },
    ];

    EntryNotice {
        title: "Notice of Intent to Enter".into(),
        entry_date,
        notice_days: i.notice_days,
        statutory_citation: citation.to_string(),
        clauses,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> EntryInput {
        EntryInput {
            landlord_name: "Acme Property Mgmt".into(),
            landlord_address: "1 Main St".into(),
            landlord_phone: String::new(),
            tenant_name: "Jane Doe".into(),
            premises_address: "42 Rental Rd".into(),
            served_date: "2026-06-13".into(),
            notice_days: 2,
            purpose: "Repair the kitchen faucet".into(),
            time_window: "9:00 AM – 12:00 PM".into(),
            state: "California".into(),
            statute_citation: String::new(),
        }
    }

    #[test]
    fn entry_date_is_served_plus_notice() {
        // 2026-06-13 + 2 days = 2026-06-15.
        assert_eq!(generate(&base()).entry_date, "2026-06-15");
    }

    #[test]
    fn purpose_in_clause() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading == "2. Purpose").unwrap();
        assert!(c.body.contains("Repair the kitchen faucet"));
    }

    #[test]
    fn time_window_included_when_given() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading.contains("Notice of Entry")).unwrap();
        assert!(c.body.contains("9:00 AM – 12:00 PM"));
    }

    #[test]
    fn no_window_falls_back_to_business_hours() {
        let c = generate(&EntryInput { time_window: String::new(), ..base() })
            .clauses
            .into_iter()
            .find(|c| c.heading.contains("Notice of Entry"))
            .unwrap();
        assert!(c.body.contains("normal business hours"));
    }

    #[test]
    fn statute_citation_echoed() {
        let n = generate(&EntryInput { statute_citation: "Cal. Civ. Code § 1954".into(), ..base() });
        assert_eq!(n.statutory_citation, "Cal. Civ. Code § 1954");
        assert!(n.clauses.iter().any(|c| c.body.contains("Cal. Civ. Code § 1954")));
    }

    #[test]
    fn bad_date_yields_empty_entry() {
        let n = generate(&EntryInput { served_date: "x".into(), ..base() });
        assert_eq!(n.entry_date, "");
    }
}
