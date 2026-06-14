//! Seller's property disclosure — the statement of known material defects a
//! seller of real property gives the buyer before sale (required in most
//! states). For each item the seller marks no known issue, a known defect, or
//! unknown, with an explanation; the form counts the disclosed defects and
//! unknowns. Distinct from the move-in/out inspection checklist (seller's
//! pre-sale liability, not tenancy condition). Drafting aid, not legal advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemStatus {
    NoIssue,
    Defect,
    Unknown,
}

impl ItemStatus {
    fn label(self) -> &'static str {
        match self {
            ItemStatus::NoIssue => "No known issue",
            ItemStatus::Defect => "KNOWN DEFECT",
            ItemStatus::Unknown => "Unknown",
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct DisclosureItem {
    pub category: String,
    pub status: ItemStatus,
    #[serde(default)]
    pub explanation: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SellerDisclosureInput {
    pub seller_name: String,
    #[serde(default)]
    pub buyer_name: String,
    pub property_address: String,
    #[serde(default)]
    pub items: Vec<DisclosureItem>,
    /// Sold as-is (the disclosure still applies to known defects).
    #[serde(default)]
    pub as_is: bool,
    pub disclosure_date: String,
    #[serde(default)]
    pub state: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DocClause {
    pub heading: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SellerDisclosure {
    pub title: String,
    pub item_count: usize,
    pub defect_count: usize,
    pub unknown_count: usize,
    pub clauses: Vec<DocClause>,
}

pub fn generate(i: &SellerDisclosureInput) -> SellerDisclosure {
    let defect_count = i.items.iter().filter(|x| x.status == ItemStatus::Defect).count();
    let unknown_count = i.items.iter().filter(|x| x.status == ItemStatus::Unknown).count();

    let disclosure_body = if i.items.is_empty() {
        "No items disclosed.".to_string()
    } else {
        i.items
            .iter()
            .map(|x| {
                let note = if x.explanation.trim().is_empty() {
                    String::new()
                } else {
                    format!(" — {}", x.explanation.trim())
                };
                format!("  • {}: {}{}", x.category, x.status.label(), note)
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    let summary_body = format!(
        "{} item(s) addressed; {} known defect(s) disclosed; {} marked unknown.",
        i.items.len(),
        defect_count,
        unknown_count
    );

    let as_is_body = if i.as_is {
        "The property is offered AS-IS. Selling as-is does NOT relieve the Seller of the duty to disclose known material defects; this disclosure is made in addition to the as-is sale.".to_string()
    } else {
        "The Seller makes this disclosure of known conditions; it is not a warranty and the Buyer is encouraged to obtain independent inspections.".to_string()
    };

    let mut clauses = vec![
        DocClause {
            heading: "Header".into(),
            body: format!(
                "Seller: {}\nBuyer: {}\nProperty: {}\nDisclosure date: {}",
                i.seller_name,
                if i.buyer_name.trim().is_empty() { "—" } else { i.buyer_name.trim() },
                i.property_address,
                i.disclosure_date
            ),
        },
        DocClause { heading: "1. Disclosure of Known Conditions".into(), body: disclosure_body },
        DocClause { heading: "2. Summary".into(), body: summary_body },
        DocClause { heading: "3. As-Is".into(), body: as_is_body },
        DocClause {
            heading: "4. Buyer Acknowledgment".into(),
            body: "The Buyer acknowledges receipt of this disclosure and understands it reflects the Seller's knowledge only, is not a substitute for the Buyer's own inspections, and does not warrant the condition of the property.".into(),
        },
    ];

    if !i.state.trim().is_empty() {
        clauses.push(DocClause {
            heading: "5. Governing Law".into(),
            body: format!("This disclosure is made under the law of the State of {}.", i.state.trim()),
        });
    }

    clauses.push(DocClause {
        heading: "Signatures".into(),
        body: format!(
            "Seller: ____________________  Date: {}\n{}\n\nBuyer: ____________________  Date: __________",
            i.disclosure_date, i.seller_name
        ),
    });

    SellerDisclosure {
        title: "Seller's Property Disclosure Statement".into(),
        item_count: i.items.len(),
        defect_count,
        unknown_count,
        clauses,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(cat: &str, status: ItemStatus, note: &str) -> DisclosureItem {
        DisclosureItem { category: cat.into(), status, explanation: note.into() }
    }

    fn base() -> SellerDisclosureInput {
        SellerDisclosureInput {
            seller_name: "Sol Seller".into(),
            buyer_name: "Bea Buyer".into(),
            property_address: "100 Maple Ave".into(),
            items: vec![
                item("Roof", ItemStatus::NoIssue, ""),
                item("Plumbing", ItemStatus::Defect, "leak under kitchen sink"),
                item("Foundation", ItemStatus::Unknown, "never inspected"),
            ],
            as_is: true,
            disclosure_date: "2026-06-15".into(),
            state: "California".into(),
        }
    }

    #[test]
    fn counts() {
        let d = generate(&base());
        assert_eq!(d.item_count, 3);
        assert_eq!(d.defect_count, 1);
        assert_eq!(d.unknown_count, 1);
    }

    #[test]
    fn disclosure_lists_items_with_status() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading.contains("Known Conditions")).unwrap();
        assert!(c.body.contains("Plumbing: KNOWN DEFECT — leak under kitchen sink"));
        assert!(c.body.contains("Roof: No known issue"));
        assert!(c.body.contains("Foundation: Unknown"));
    }

    #[test]
    fn summary_states_counts() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading == "2. Summary").unwrap();
        assert!(c.body.contains("1 known defect(s) disclosed"));
        assert!(c.body.contains("1 marked unknown"));
    }

    #[test]
    fn as_is_still_requires_disclosure() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading == "3. As-Is").unwrap();
        assert!(c.body.contains("does NOT relieve the Seller"));
    }

    #[test]
    fn not_as_is_message() {
        let c = generate(&SellerDisclosureInput { as_is: false, ..base() })
            .clauses.into_iter().find(|c| c.heading == "3. As-Is").unwrap();
        assert!(c.body.contains("not a warranty"));
    }

    #[test]
    fn empty_items() {
        let d = generate(&SellerDisclosureInput { items: vec![], ..base() });
        assert_eq!(d.defect_count, 0);
        let c = d.clauses.iter().find(|c| c.heading.contains("Known Conditions")).unwrap();
        assert!(c.body.contains("No items disclosed"));
    }

    #[test]
    fn governing_law_only_with_state() {
        assert!(generate(&base()).clauses.iter().any(|c| c.heading == "5. Governing Law"));
        let none = generate(&SellerDisclosureInput { state: String::new(), ..base() });
        assert!(!none.clauses.iter().any(|c| c.heading == "5. Governing Law"));
    }
}
