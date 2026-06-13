//! Landlord notice generator — jurisdiction-agnostic landlord-tenant notices.
//!
//! Produces the two pre-eviction notices a landlord serves, in generic
//! language that names the governing state rather than baking in any one
//! jurisdiction's form:
//!
//!   * **Pay Rent or Quit** — demands overdue rent and gives the tenant a
//!     window to pay or vacate (commonly 3–7 days; set per your state).
//!   * **Notice to Quit / Terminate Tenancy** — ends the tenancy and demands
//!     possession; the period must satisfy your state's minimum (often 30
//!     days for a month-to-month).
//!
//! The comply-by / move-by date is the service date plus the notice period.
//! The state is a free-text field (governing jurisdiction); an optional
//! statute citation is included verbatim when the caller supplies one (e.g.
//! a Michigan landlord can pass "MCL 600.5714(1)(a)"). This is a drafting
//! aid, not legal advice, and does not replace any official court form your
//! jurisdiction requires. Pure compute.

use chrono::{Duration, NaiveDate};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoticeType {
    /// Nonpayment of rent — pay within the period or vacate.
    PayOrQuit,
    /// Terminate the tenancy and recover possession.
    NoticeToQuit,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NoticeInput {
    pub notice_type: NoticeType,
    pub landlord_name: String,
    pub landlord_address: String,
    #[serde(default)]
    pub landlord_phone: String,
    pub tenant_name: String,
    pub premises_address: String,
    /// Date the notice is served (YYYY-MM-DD).
    pub served_date: String,
    /// Days the tenant has to comply (per your state's minimum).
    pub notice_days: i64,
    /// Rent claimed due — Pay-or-Quit only.
    #[serde(default)]
    pub rent_owed_usd: f64,
    /// Reason for termination — Notice-to-Quit only (optional).
    #[serde(default)]
    pub reason: String,
    /// Governing jurisdiction (e.g. "California", "Michigan").
    pub state: String,
    /// Optional statute citation included verbatim when provided.
    #[serde(default)]
    pub statute_citation: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct NoticeClause {
    pub heading: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct NoticeDocument {
    pub title: String,
    /// Service date + notice period; empty if the date can't be parsed.
    pub comply_by_date: String,
    pub notice_days: i64,
    /// Echoes the caller's statute citation (empty when none supplied).
    pub statutory_citation: String,
    pub clauses: Vec<NoticeClause>,
}

pub fn generate(i: &NoticeInput) -> NoticeDocument {
    let comply_by_date = NaiveDate::parse_from_str(&i.served_date, "%Y-%m-%d")
        .map(|d| (d + Duration::days(i.notice_days)).format("%Y-%m-%d").to_string())
        .unwrap_or_default();

    let citation = i.statute_citation.trim();
    let pursuant = if citation.is_empty() {
        format!("This notice is given pursuant to the landlord-tenant laws of the State of {}.", i.state)
    } else {
        format!(
            "This notice is given pursuant to the landlord-tenant laws of the State of {} ({}).",
            i.state, citation
        )
    };

    let signature = NoticeClause {
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
    };
    let legal = NoticeClause {
        heading: "How to Get Legal Help".into(),
        body: "If you believe you have a defense, you may wish to consult an attorney or a local legal aid office promptly. You can locate one through your state bar's lawyer referral service or your local legal aid provider.".into(),
    };

    match i.notice_type {
        NoticeType::PayOrQuit => {
            let clauses = vec![
                NoticeClause { heading: "To".into(), body: i.tenant_name.clone() },
                NoticeClause {
                    heading: "1. Rent Demanded".into(),
                    body: format!(
                        "Your landlord/landlady, {}, states that rent in the amount of ${:.2} is now due and unpaid for the premises located at {}.",
                        i.landlord_name, i.rent_owed_usd, i.premises_address
                    ),
                },
                NoticeClause {
                    heading: "2. Pay or Vacate".into(),
                    body: format!(
                        "You are required, within {} days from the date this notice is served, to either (a) pay the rent in full, OR (b) vacate and deliver up possession of the premises. You must comply by {}. If you do neither, legal proceedings to recover possession may be commenced against you. If you move out, you may still owe the rent.",
                        i.notice_days, comply_by_date
                    ),
                },
                NoticeClause {
                    heading: "3. Right to Be Heard".into(),
                    body: "If the landlord/landlady brings an action to evict you and you have paid the rent, or you believe there is a good reason why you do not owe it, you will have the opportunity to present your reasons to the court.".into(),
                },
                NoticeClause { heading: "4. Governing Law".into(), body: pursuant },
                signature,
                legal,
            ];
            NoticeDocument {
                title: "Notice to Pay Rent or Quit (Landlord-Tenant)".into(),
                comply_by_date,
                notice_days: i.notice_days,
                statutory_citation: citation.to_string(),
                clauses,
            }
        }
        NoticeType::NoticeToQuit => {
            let basis = if i.reason.trim().is_empty() {
                String::new()
            } else {
                format!(" The reason for this notice is: {}.", i.reason.trim())
            };
            let clauses = vec![
                NoticeClause { heading: "To".into(), body: i.tenant_name.clone() },
                NoticeClause {
                    heading: "1. Termination of Tenancy".into(),
                    body: format!(
                        "Your landlord/landlady, {}, hereby notifies you to quit and deliver up possession of the premises located at {}, which you currently hold as a tenant.{}",
                        i.landlord_name, i.premises_address, basis
                    ),
                },
                NoticeClause {
                    heading: "2. Move-By Date".into(),
                    body: format!(
                        "You must move out on or before {} ({} days from service of this notice). If you fail to do so, legal proceedings to recover possession may be commenced against you.",
                        comply_by_date, i.notice_days
                    ),
                },
                NoticeClause {
                    heading: "3. Right to Be Heard".into(),
                    body: "If the landlord/landlady brings an action to evict you, you will have the opportunity to present reasons why you believe you should not be evicted.".into(),
                },
                NoticeClause { heading: "4. Governing Law".into(), body: pursuant },
                signature,
                legal,
            ];
            NoticeDocument {
                title: "Notice to Quit — Termination of Tenancy (Landlord-Tenant)".into(),
                comply_by_date,
                notice_days: i.notice_days,
                statutory_citation: citation.to_string(),
                clauses,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> NoticeInput {
        NoticeInput {
            notice_type: NoticeType::PayOrQuit,
            landlord_name: "Jacob Menke".into(),
            landlord_address: "5611 Vorhies Rd, Ann Arbor MI 48105".into(),
            landlord_phone: String::new(),
            tenant_name: "Taurean Collins".into(),
            premises_address: "5611 Vorhies Rd, Ann Arbor, MI".into(),
            served_date: "2024-01-21".into(),
            notice_days: 7,
            rent_owed_usd: 1_500.0,
            reason: String::new(),
            state: "Michigan".into(),
            statute_citation: String::new(),
        }
    }

    #[test]
    fn pay_or_quit_comply_by_is_served_plus_period() {
        let d = generate(&base());
        assert!(d.title.contains("Pay Rent or Quit"));
        assert_eq!(d.comply_by_date, "2024-01-28");
    }

    #[test]
    fn notice_to_quit_comply_by_is_served_plus_thirty() {
        let d = generate(&NoticeInput {
            notice_type: NoticeType::NoticeToQuit,
            notice_days: 30,
            ..base()
        });
        assert!(d.title.contains("Termination of Tenancy"));
        assert_eq!(d.comply_by_date, "2024-02-20");
    }

    #[test]
    fn is_location_agnostic_names_the_given_state() {
        // California landlord gets California language — no hardcoded Michigan.
        let d = generate(&NoticeInput { state: "California".into(), ..base() });
        let gov = d.clauses.iter().find(|c| c.heading.contains("Governing")).unwrap();
        assert!(gov.body.contains("State of California"));
        assert!(!gov.body.contains("Michigan"));
        // No SCAO/MCL/Michigan strings anywhere in the document.
        let all: String = d.clauses.iter().map(|c| format!("{} {}", c.heading, c.body)).collect();
        assert!(!all.contains("SCAO") && !all.contains("MCL") && !all.contains("DC 100"));
    }

    #[test]
    fn optional_statute_citation_included_when_provided() {
        let d = generate(&NoticeInput {
            statute_citation: "MCL 600.5714(1)(a)".into(),
            ..base()
        });
        assert_eq!(d.statutory_citation, "MCL 600.5714(1)(a)");
        let gov = d.clauses.iter().find(|c| c.heading.contains("Governing")).unwrap();
        assert!(gov.body.contains("MCL 600.5714(1)(a)"));
    }

    #[test]
    fn no_citation_clause_omits_parenthetical() {
        let d = generate(&base());
        assert_eq!(d.statutory_citation, "");
        let gov = d.clauses.iter().find(|c| c.heading.contains("Governing")).unwrap();
        assert!(gov.body.contains("State of Michigan") && !gov.body.contains("("));
    }

    #[test]
    fn pay_or_quit_states_amount_and_premises() {
        let d = generate(&base());
        let rent = d.clauses.iter().find(|c| c.heading.starts_with("1.")).unwrap();
        assert!(rent.body.contains("$1500.00"));
        assert!(!rent.body.contains("Taurean Collins")); // tenant is in "To"
        assert!(rent.body.contains("Jacob Menke"));
    }

    #[test]
    fn termination_reason_appended_when_present() {
        let d = generate(&NoticeInput {
            notice_type: NoticeType::NoticeToQuit,
            reason: "material lease violation".into(),
            ..base()
        });
        let c1 = d.clauses.iter().find(|c| c.heading.starts_with("1.")).unwrap();
        assert!(c1.body.contains("material lease violation"));
    }

    #[test]
    fn both_notices_include_legal_help_and_signature() {
        for nt in [NoticeType::PayOrQuit, NoticeType::NoticeToQuit] {
            let d = generate(&NoticeInput { notice_type: nt, ..base() });
            assert!(d.clauses.iter().any(|c| c.heading == "How to Get Legal Help"));
            assert!(d.clauses.iter().any(|c| c.heading == "Signature"));
        }
    }
}
