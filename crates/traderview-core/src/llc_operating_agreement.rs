//! LLC operating agreement — governs a limited liability company among its
//! members. Each member's capital contribution determines their ownership
//! percentage (capital ÷ total capital), which in turn drives profit/loss
//! allocation and distributions. It computes the splits and assembles the
//! operative clauses. Drafting aid, not legal advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Member {
    pub name: String,
    #[serde(default)]
    pub capital_usd: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Management {
    MemberManaged,
    ManagerManaged,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LlcInput {
    pub llc_name: String,
    pub formation_state: String,
    pub formation_date: String,
    pub management: Management,
    #[serde(default)]
    pub members: Vec<Member>,
    #[serde(default)]
    pub statute_citation: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct MemberShare {
    pub name: String,
    pub capital_usd: f64,
    pub ownership_pct: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DocClause {
    pub heading: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct LlcOperatingAgreement {
    pub title: String,
    pub member_count: usize,
    pub total_capital_usd: f64,
    pub members: Vec<MemberShare>,
    pub statutory_citation: String,
    pub clauses: Vec<DocClause>,
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

pub fn generate(i: &LlcInput) -> LlcOperatingAgreement {
    let total_capital: f64 = i.members.iter().map(|m| m.capital_usd).sum();
    // When no capital is recorded, fall back to an equal split by head count so
    // the agreement still allocates ownership rather than dividing by zero.
    let n = i.members.len();
    let members: Vec<MemberShare> = i
        .members
        .iter()
        .map(|m| {
            let pct = if total_capital > 0.0 {
                m.capital_usd / total_capital * 100.0
            } else if n > 0 {
                100.0 / n as f64
            } else {
                0.0
            };
            MemberShare {
                name: m.name.clone(),
                capital_usd: m.capital_usd,
                ownership_pct: cents(pct),
            }
        })
        .collect();

    let citation = i.statute_citation.trim();
    let pursuant = if citation.is_empty() {
        format!("This Agreement is governed by the LLC Act of the State of {}.", i.formation_state)
    } else {
        format!("This Agreement is governed by the LLC Act of the State of {} ({}).", i.formation_state, citation)
    };

    let members_body = if members.is_empty() {
        "No members listed.".to_string()
    } else {
        members
            .iter()
            .map(|m| format!("  • {}: capital {} — {:.2}% ownership", m.name, money(m.capital_usd), m.ownership_pct))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let mgmt_body = match i.management {
        Management::MemberManaged => "The Company is member-managed: the members manage the business, and each member is an agent of the Company for its ordinary course of business. Major decisions require the approval of members holding a majority of the ownership interests unless stated otherwise.".to_string(),
        Management::ManagerManaged => "The Company is manager-managed: one or more managers appointed by the members manage the business and have authority to act for the Company. Members do not, solely by being members, have authority to bind the Company.".to_string(),
    };

    let clauses = vec![
        DocClause {
            heading: "1. Formation".into(),
            body: format!(
                "{} (the \"Company\") is a limited liability company formed under the laws of the State of {}, effective {}.",
                i.llc_name, i.formation_state, i.formation_date
            ),
        },
        DocClause {
            heading: "2. Members and Capital".into(),
            body: format!(
                "Total capital contributed: {}. Each member's contribution and ownership percentage:\n{}",
                money(total_capital), members_body
            ),
        },
        DocClause {
            heading: "3. Profits and Losses".into(),
            body: "Net profits and losses are allocated among the members in proportion to their ownership percentages set out above.".into(),
        },
        DocClause { heading: "4. Management".into(), body: mgmt_body },
        DocClause {
            heading: "5. Distributions".into(),
            body: "Distributions, when declared, are made to the members in proportion to their ownership percentages. No member has a right to demand a distribution except as agreed.".into(),
        },
        DocClause {
            heading: "6. Transfer of Interests".into(),
            body: "A member may not transfer its interest without the written consent of the other members. Any permitted transferee takes the interest subject to this Agreement.".into(),
        },
        DocClause {
            heading: "7. Dissolution".into(),
            body: "The Company dissolves upon the written agreement of the members or as required by law. On dissolution, assets are applied first to liabilities, then distributed to the members in proportion to their ownership percentages.".into(),
        },
        DocClause { heading: "8. Governing Law".into(), body: pursuant },
        DocClause {
            heading: "Signatures".into(),
            body: members
                .iter()
                .map(|m| format!("{}: ____________________  Date: __________", m.name))
                .collect::<Vec<_>>()
                .join("\n\n"),
        },
    ];

    LlcOperatingAgreement {
        title: "LLC Operating Agreement".into(),
        member_count: members.len(),
        total_capital_usd: total_capital,
        members,
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

    fn mem(name: &str, cap: f64) -> Member {
        Member { name: name.into(), capital_usd: cap }
    }

    fn base() -> LlcInput {
        LlcInput {
            llc_name: "Widgets LLC".into(),
            formation_state: "Delaware".into(),
            formation_date: "2026-07-01".into(),
            management: Management::MemberManaged,
            members: vec![mem("Alice", 60_000.0), mem("Bob", 40_000.0)],
            statute_citation: String::new(),
        }
    }

    #[test]
    fn ownership_from_capital() {
        let d = generate(&base());
        assert!(close(d.total_capital_usd, 100_000.0));
        assert!(close(d.members[0].ownership_pct, 60.0));
        assert!(close(d.members[1].ownership_pct, 40.0));
    }

    #[test]
    fn ownership_listed_in_clause() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading.contains("Members and Capital")).unwrap();
        assert!(c.body.contains("Alice: capital $60000.00 — 60.00% ownership"));
    }

    #[test]
    fn zero_capital_equal_split() {
        let d = generate(&LlcInput {
            members: vec![mem("A", 0.0), mem("B", 0.0), mem("C", 0.0)],
            ..base()
        });
        for m in &d.members {
            assert!((m.ownership_pct - 33.33).abs() < 0.01);
        }
    }

    #[test]
    fn member_managed_clause() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading == "4. Management").unwrap();
        assert!(c.body.contains("member-managed"));
    }

    #[test]
    fn manager_managed_clause() {
        let d = generate(&LlcInput { management: Management::ManagerManaged, ..base() });
        let c = d.clauses.iter().find(|c| c.heading == "4. Management").unwrap();
        assert!(c.body.contains("manager-managed"));
    }

    #[test]
    fn statute_citation_echoed() {
        let d = generate(&LlcInput { statute_citation: "6 Del. C. § 18".into(), ..base() });
        assert_eq!(d.statutory_citation, "6 Del. C. § 18");
        assert!(d.clauses.iter().any(|c| c.body.contains("6 Del. C. § 18")));
    }
}
