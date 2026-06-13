//! Move-in / move-out inspection checklist — the area-by-area condition record
//! a landlord and tenant complete at the start and end of a tenancy. It records
//! each area's condition and notes, counts the items flagged as needing
//! attention (fair / poor / damaged), and assembles the checklist. Pairing the
//! move-in and move-out records is what justifies any deposit deduction.
//! Drafting aid, not legal advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct AreaItem {
    pub area: String,
    pub condition: String,
    #[serde(default)]
    pub notes: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InspectionType {
    MoveIn,
    MoveOut,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChecklistInput {
    pub landlord_name: String,
    pub tenant_name: String,
    pub premises_address: String,
    pub inspection_type: InspectionType,
    pub inspection_date: String,
    #[serde(default)]
    pub areas: Vec<AreaItem>,
    #[serde(default)]
    pub state: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DocClause {
    pub heading: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct InspectionChecklist {
    pub title: String,
    pub item_count: usize,
    /// Items whose condition is fair / poor / damaged (case-insensitive match).
    pub needs_attention_count: usize,
    pub clauses: Vec<DocClause>,
}

/// An area is flagged when its condition reads as less than good.
fn needs_attention(condition: &str) -> bool {
    let c = condition.to_lowercase();
    ["fair", "poor", "damage", "broken", "stain", "worn", "need"]
        .iter()
        .any(|kw| c.contains(kw))
}

pub fn generate(i: &ChecklistInput) -> InspectionChecklist {
    let item_count = i.areas.len();
    let needs_attention_count = i.areas.iter().filter(|a| needs_attention(&a.condition)).count();

    let type_word = match i.inspection_type {
        InspectionType::MoveIn => "Move-In",
        InspectionType::MoveOut => "Move-Out",
    };

    // Condition record: one line per area, with notes and a flag marker.
    let record_body = if i.areas.is_empty() {
        "No areas recorded.".to_string()
    } else {
        i.areas
            .iter()
            .map(|a| {
                let flag = if needs_attention(&a.condition) { " [needs attention]" } else { "" };
                let notes = if a.notes.trim().is_empty() {
                    String::new()
                } else {
                    format!(" — {}", a.notes.trim())
                };
                format!("  • {}: {}{}{}", a.area, a.condition, flag, notes)
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    let summary_body = format!(
        "{} area(s) recorded; {} flagged as needing attention.",
        item_count, needs_attention_count
    );

    let ack_body = match i.inspection_type {
        InspectionType::MoveIn => "The Tenant acknowledges that the premises were inspected at move-in and that the conditions recorded above are accurate. This record will be compared against the move-out inspection to determine any deposit deductions.".to_string(),
        InspectionType::MoveOut => "The parties acknowledge that the premises were inspected at move-out and that the conditions recorded above are accurate. Differences from the move-in record (beyond ordinary wear and tear) may support deductions from the security deposit.".to_string(),
    };

    let mut clauses = vec![
        DocClause {
            heading: "Parties and Premises".into(),
            body: format!(
                "Landlord: {}\nTenant: {}\nPremises: {}\nInspection: {} on {}",
                i.landlord_name, i.tenant_name, i.premises_address, type_word, i.inspection_date
            ),
        },
        DocClause { heading: "1. Condition Record".into(), body: record_body },
        DocClause { heading: "2. Summary".into(), body: summary_body },
        DocClause { heading: "3. Acknowledgment".into(), body: ack_body },
    ];

    if !i.state.trim().is_empty() {
        clauses.push(DocClause {
            heading: "4. Governing Law".into(),
            body: format!(
                "This record is maintained pursuant to the landlord-tenant law of the State of {}.",
                i.state.trim()
            ),
        });
    }

    clauses.push(DocClause {
        heading: "Signatures".into(),
        body: format!(
            "Landlord: ____________________  Date: {}\n{}\n\nTenant: ____________________  Date: {}\n{}",
            i.inspection_date, i.landlord_name, i.inspection_date, i.tenant_name
        ),
    });

    InspectionChecklist {
        title: format!("{type_word} Inspection Checklist"),
        item_count,
        needs_attention_count,
        clauses,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn area(a: &str, c: &str, n: &str) -> AreaItem {
        AreaItem { area: a.into(), condition: c.into(), notes: n.into() }
    }

    fn base() -> ChecklistInput {
        ChecklistInput {
            landlord_name: "Acme Property Mgmt".into(),
            tenant_name: "Jane Doe".into(),
            premises_address: "42 Rental Rd".into(),
            inspection_type: InspectionType::MoveIn,
            inspection_date: "2026-06-01".into(),
            areas: vec![
                area("Living room walls", "Good", ""),
                area("Kitchen floor", "Fair", "scratches"),
                area("Bathroom faucet", "Damaged", "leaks"),
            ],
            state: "California".into(),
        }
    }

    #[test]
    fn counts_items_and_flags() {
        let d = generate(&base());
        assert_eq!(d.item_count, 3);
        // Fair + Damaged flagged; Good is not.
        assert_eq!(d.needs_attention_count, 2);
    }

    #[test]
    fn record_marks_flagged_items() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading.contains("Condition Record")).unwrap();
        assert!(c.body.contains("Kitchen floor: Fair [needs attention] — scratches"));
        assert!(c.body.contains("Living room walls: Good"));
        assert!(!c.body.contains("Living room walls: Good [needs attention]"));
    }

    #[test]
    fn title_reflects_type() {
        assert_eq!(generate(&base()).title, "Move-In Inspection Checklist");
        let out = generate(&ChecklistInput { inspection_type: InspectionType::MoveOut, ..base() });
        assert_eq!(out.title, "Move-Out Inspection Checklist");
    }

    #[test]
    fn move_out_acknowledgment_mentions_deductions() {
        let d = generate(&ChecklistInput { inspection_type: InspectionType::MoveOut, ..base() });
        let c = d.clauses.iter().find(|c| c.heading.contains("Acknowledgment")).unwrap();
        assert!(c.body.contains("security deposit"));
    }

    #[test]
    fn empty_areas_zero_counts() {
        let d = generate(&ChecklistInput { areas: vec![], ..base() });
        assert_eq!(d.item_count, 0);
        assert_eq!(d.needs_attention_count, 0);
        let c = d.clauses.iter().find(|c| c.heading.contains("Condition Record")).unwrap();
        assert!(c.body.contains("No areas recorded"));
    }

    #[test]
    fn governing_law_only_when_state_given() {
        let with = generate(&base());
        assert!(with.clauses.iter().any(|c| c.heading == "4. Governing Law"));
        let without = generate(&ChecklistInput { state: String::new(), ..base() });
        assert!(!without.clauses.iter().any(|c| c.heading == "4. Governing Law"));
    }
}
