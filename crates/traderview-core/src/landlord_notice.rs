//! Landlord notice generator — Michigan SCAO landlord-tenant notices.
//!
//! Produces the two pre-eviction notices a Michigan landlord serves, with
//! the statutory language and citations from the SCAO forms and the
//! compliance deadline computed from the service date:
//!
//!   * **DC 100a — Demand for Possession, Nonpayment of Rent.** Demands the
//!     overdue rent and gives the tenant a window (7 days by statute) to
//!     pay or vacate. MCL 600.5714(1)(a), 600.5716, 600.5718, 600.5775(2)(f).
//!   * **DC 100c — Notice to Quit to Recover Possession.** Ends the tenancy
//!     and demands possession; the notice must equal at least one rental
//!     period (30 days for a month-to-month). MCL 554.134(1)/(3),
//!     600.5714(1)(c)(iii),(e).
//!
//! The comply-by / move-by date is the service date plus the notice period.
//! This reproduces the SCAO form language with the parties, premises, and
//! amounts filled in; it is a drafting aid, not legal advice, and does not
//! replace filing the official court form. Pure compute.

use chrono::{Duration, NaiveDate};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoticeType {
    /// DC 100a — Demand for Possession, Nonpayment of Rent (7-day default).
    DemandNonpaymentRent,
    /// DC 100c — Notice to Quit to Recover Possession (one rental period).
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
    /// Days the tenant has to comply (7 for nonpayment, 30 for month-to-month).
    pub notice_days: i64,
    /// Rent claimed due — DC 100a only.
    #[serde(default)]
    pub rent_owed_usd: f64,
    /// Optional "other" reason — DC 100c only.
    #[serde(default)]
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct NoticeClause {
    pub heading: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct NoticeDocument {
    /// SCAO form identifier ("DC 100a" / "DC 100c").
    pub form_id: String,
    pub title: String,
    /// Service date + notice period; empty if the date can't be parsed.
    pub comply_by_date: String,
    pub notice_days: i64,
    pub statutory_citation: String,
    pub clauses: Vec<NoticeClause>,
}

const LEGAL_HELP: &str = "1. Call your own lawyer.\n2. If you do not have an attorney but have money to retain one, you may locate an attorney through the State Bar of Michigan Lawyer Referral Service at 1-800-968-0738.\n3. If you do not have an attorney and cannot pay for legal help, you may qualify for assistance through a local legal aid office (www.michiganlegalhelp.org).";

pub fn generate(i: &NoticeInput) -> NoticeDocument {
    let comply_by_date = NaiveDate::parse_from_str(&i.served_date, "%Y-%m-%d")
        .map(|d| (d + Duration::days(i.notice_days)).format("%Y-%m-%d").to_string())
        .unwrap_or_default();

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
    let legal = NoticeClause { heading: "How to Get Legal Help".into(), body: LEGAL_HELP.into() };

    match i.notice_type {
        NoticeType::DemandNonpaymentRent => {
            let clauses = vec![
                NoticeClause {
                    heading: "To".into(),
                    body: i.tenant_name.clone(),
                },
                NoticeClause {
                    heading: "1. Rent Demanded".into(),
                    body: format!(
                        "Your landlord/landlady, {}, says that you owe ${:.2} rent for the premises at {}.",
                        i.landlord_name, i.rent_owed_usd, i.premises_address
                    ),
                },
                NoticeClause {
                    heading: "2. Pay or Vacate".into(),
                    body: format!(
                        "If you owe this rent, you must do one of the following within {} days from the date this notice was served: (a) pay the rent owed, OR (b) move out or vacate the premises. You must comply by {}. If you do not, your landlord/landlady may take you to court to evict you. If you move out or vacate, you may still owe rent.",
                        i.notice_days, comply_by_date
                    ),
                },
                NoticeClause {
                    heading: "3. Right to Be Heard".into(),
                    body: "If your landlord/landlady takes you to court to evict you and you have paid the rent, or you believe there is a good reason why you do not owe the rent, you will have the opportunity to present the reasons why you believe you should not be evicted.".into(),
                },
                NoticeClause {
                    heading: "4. Right to Counsel".into(),
                    body: "If you believe there is a good reason why you do not owe the rent claimed, you can have a lawyer advise you. Call him or her soon.".into(),
                },
                signature,
                legal,
            ];
            NoticeDocument {
                form_id: "DC 100a".into(),
                title: "Demand for Possession — Nonpayment of Rent (Landlord-Tenant)".into(),
                comply_by_date,
                notice_days: i.notice_days,
                statutory_citation: "MCL 600.5714(1)(a), MCL 600.5716, MCL 600.5718, MCL 600.5775(2)(f)".into(),
                clauses,
            }
        }
        NoticeType::NoticeToQuit => {
            let basis = if i.reason.trim().is_empty() {
                "MCL 554.134(1) or (3)".to_string()
            } else {
                format!("MCL 554.134(1) or (3); other: {}", i.reason.trim())
            };
            let clauses = vec![
                NoticeClause {
                    heading: "To".into(),
                    body: i.tenant_name.clone(),
                },
                NoticeClause {
                    heading: "1. Recovery of Possession".into(),
                    body: format!(
                        "Your landlord/landlady, {}, is seeking to recover possession of property pursuant to {} and wants you to move from: {}.",
                        i.landlord_name, basis, i.premises_address
                    ),
                },
                NoticeClause {
                    heading: "2. Move-By Date".into(),
                    body: format!(
                        "You must move by {} ({} days from service) or your landlord/landlady may take you to court to evict you. (Unless otherwise allowed by law, this notice must equal at least one rental period.)",
                        comply_by_date, i.notice_days
                    ),
                },
                NoticeClause {
                    heading: "3. Right to Be Heard".into(),
                    body: "If your landlord/landlady takes you to court to evict you, you will have the opportunity to present reasons why you believe you should not be evicted.".into(),
                },
                NoticeClause {
                    heading: "4. Right to Counsel".into(),
                    body: "If you believe you have a good reason why you should not be evicted, you may have a lawyer advise you. Call him or her soon.".into(),
                },
                signature,
                legal,
            ];
            NoticeDocument {
                form_id: "DC 100c".into(),
                title: "Notice to Quit to Recover Possession of Property (Landlord-Tenant)".into(),
                comply_by_date,
                notice_days: i.notice_days,
                statutory_citation: "MCL 554.134(1)/(3), MCL 600.5714(1)(c)(iii), (e)".into(),
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
            notice_type: NoticeType::DemandNonpaymentRent,
            landlord_name: "Jacob Menke".into(),
            landlord_address: "5611 Vorhies Rd, Ann Arbor MI 48105".into(),
            landlord_phone: String::new(),
            tenant_name: "Taurean Collins".into(),
            premises_address: "5611 Vorhies Rd, Ann Arbor, MI".into(),
            served_date: "2024-01-21".into(),
            notice_days: 7,
            rent_owed_usd: 1_500.0,
            reason: String::new(),
        }
    }

    #[test]
    fn demand_comply_by_is_served_plus_seven() {
        let d = generate(&base());
        assert_eq!(d.form_id, "DC 100a");
        assert_eq!(d.comply_by_date, "2024-01-28");
    }

    #[test]
    fn notice_to_quit_comply_by_is_served_plus_thirty() {
        let d = generate(&NoticeInput {
            notice_type: NoticeType::NoticeToQuit,
            notice_days: 30,
            ..base()
        });
        assert_eq!(d.form_id, "DC 100c");
        assert_eq!(d.comply_by_date, "2024-02-20");
    }

    #[test]
    fn demand_states_amount_and_nonpayment_citation() {
        let d = generate(&base());
        assert!(d.statutory_citation.contains("600.5714(1)(a)"));
        let rent = d.clauses.iter().find(|c| c.heading.starts_with("1.")).unwrap();
        assert!(rent.body.contains("$1500.00"));
        assert!(rent.body.contains("Jacob Menke"));
    }

    #[test]
    fn notice_to_quit_cites_554_134() {
        let d = generate(&NoticeInput { notice_type: NoticeType::NoticeToQuit, ..base() });
        assert!(d.statutory_citation.contains("554.134"));
        let recover = d.clauses.iter().find(|c| c.heading.starts_with("1.")).unwrap();
        assert!(recover.body.contains("MCL 554.134(1) or (3)"));
    }

    #[test]
    fn other_reason_appended_to_notice_to_quit_basis() {
        let d = generate(&NoticeInput {
            notice_type: NoticeType::NoticeToQuit,
            reason: "material lease violation".into(),
            ..base()
        });
        let recover = d.clauses.iter().find(|c| c.heading.starts_with("1.")).unwrap();
        assert!(recover.body.contains("other: material lease violation"));
    }

    #[test]
    fn parties_and_premises_interpolated() {
        let d = generate(&base());
        assert!(d.clauses[0].body.contains("Taurean Collins"));
        let sig = d.clauses.iter().find(|c| c.heading == "Signature").unwrap();
        assert!(sig.body.contains("Jacob Menke") && sig.body.contains("5611 Vorhies"));
    }

    #[test]
    fn both_notices_include_legal_help() {
        let demand = generate(&base());
        let quit = generate(&NoticeInput { notice_type: NoticeType::NoticeToQuit, ..base() });
        assert!(demand.clauses.iter().any(|c| c.heading == "How to Get Legal Help"));
        assert!(quit.clauses.iter().any(|c| c.heading == "How to Get Legal Help"));
    }
}
