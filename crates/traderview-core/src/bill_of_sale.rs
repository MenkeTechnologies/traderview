//! Bill of sale — transfers ownership of personal property (a vehicle,
//! equipment, a boat, business assets) from seller to buyer. It computes any
//! sales tax and the total consideration, then assembles the document with the
//! transfer, condition (as-is or warranted), and title clauses. Drafting aid,
//! not legal advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct BillOfSaleInput {
    pub seller_name: String,
    #[serde(default)]
    pub seller_address: String,
    pub buyer_name: String,
    #[serde(default)]
    pub buyer_address: String,
    /// Description of the property being sold (make/model/VIN, serial, etc.).
    pub item_description: String,
    pub sale_price_usd: f64,
    #[serde(default)]
    pub sales_tax_pct: f64,
    /// Sold as-is with no warranty (true) or with the seller's warranty (false).
    #[serde(default = "default_as_is")]
    pub as_is: bool,
    pub sale_date: String,
    pub governing_state: String,
    #[serde(default)]
    pub statute_citation: String,
}

fn default_as_is() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DocClause {
    pub heading: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct BillOfSale {
    pub title: String,
    pub sale_price_usd: f64,
    pub sales_tax_usd: f64,
    pub total_usd: f64,
    pub statutory_citation: String,
    pub clauses: Vec<DocClause>,
}

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

pub fn generate(i: &BillOfSaleInput) -> BillOfSale {
    let tax = i.sale_price_usd * i.sales_tax_pct / 100.0;
    let total = i.sale_price_usd + tax;

    let citation = i.statute_citation.trim();
    let pursuant = if citation.is_empty() {
        format!(
            "This bill of sale shall be governed by the laws of the State of {}.",
            i.governing_state
        )
    } else {
        format!(
            "This bill of sale shall be governed by the laws of the State of {} ({}).",
            i.governing_state, citation
        )
    };

    // Consideration block — show the tax line only when a rate is applied.
    let consideration = if tax != 0.0 {
        format!(
            "Sale price: {}\n  Sales tax ({:.3}%): {}\n  Total consideration: {}",
            money(i.sale_price_usd),
            i.sales_tax_pct,
            money(tax),
            money(total)
        )
    } else {
        format!("Total consideration: {}", money(i.sale_price_usd))
    };

    let condition = if i.as_is {
        "The property is sold AS-IS, WHERE-IS, with all faults, and the Seller makes no warranties, express or implied, including any warranty of merchantability or fitness for a particular purpose. The Buyer has had the opportunity to inspect the property and accepts it in its present condition.".to_string()
    } else {
        "The Seller warrants that the property is in good working condition as of the date of sale and conforms to the description above. This warranty is in addition to any rights provided by law.".to_string()
    };

    let seller_line = if i.seller_address.trim().is_empty() {
        i.seller_name.clone()
    } else {
        format!("{}, {}", i.seller_name, i.seller_address.trim())
    };
    let buyer_line = if i.buyer_address.trim().is_empty() {
        i.buyer_name.clone()
    } else {
        format!("{}, {}", i.buyer_name, i.buyer_address.trim())
    };

    let clauses = vec![
        DocClause {
            heading: "Parties".into(),
            body: format!("Seller: {}\nBuyer: {}\nDate of Sale: {}", seller_line, buyer_line, i.sale_date),
        },
        DocClause {
            heading: "1. Property Sold".into(),
            body: format!(
                "The Seller sells, transfers, and delivers to the Buyer the following property: {}.",
                i.item_description
            ),
        },
        DocClause { heading: "2. Consideration".into(), body: consideration },
        DocClause { heading: "3. Condition".into(), body: condition },
        DocClause {
            heading: "4. Title".into(),
            body: "The Seller warrants that the Seller is the lawful owner of the property, that it is free of all liens and encumbrances, that the Seller has full right and authority to sell it, and that the Seller will defend the title against the claims of all persons.".into(),
        },
        DocClause { heading: "5. Governing Law".into(), body: pursuant },
        DocClause {
            heading: "Signatures".into(),
            body: format!(
                "Seller: ____________________  Date: {}\n{}\n\nBuyer: ____________________  Date: {}\n{}",
                i.sale_date, i.seller_name, i.sale_date, i.buyer_name
            ),
        },
    ];

    BillOfSale {
        title: "Bill of Sale".into(),
        sale_price_usd: i.sale_price_usd,
        sales_tax_usd: tax,
        total_usd: total,
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

    fn base() -> BillOfSaleInput {
        BillOfSaleInput {
            seller_name: "Sam Seller".into(),
            seller_address: "1 Oak St".into(),
            buyer_name: "Bob Buyer".into(),
            buyer_address: "2 Elm St".into(),
            item_description: "2019 Honda Civic, VIN 1HG...".into(),
            sale_price_usd: 12_000.0,
            sales_tax_pct: 6.0,
            as_is: true,
            sale_date: "2026-06-13".into(),
            governing_state: "Florida".into(),
            statute_citation: String::new(),
        }
    }

    #[test]
    fn tax_and_total() {
        let b = generate(&base());
        assert!(close(b.sales_tax_usd, 720.0));
        assert!(close(b.total_usd, 12_720.0));
    }

    #[test]
    fn zero_tax_total_is_price_and_no_tax_line() {
        let b = generate(&BillOfSaleInput { sales_tax_pct: 0.0, ..base() });
        assert!(close(b.sales_tax_usd, 0.0));
        assert!(close(b.total_usd, 12_000.0));
        let c = b.clauses.iter().find(|c| c.heading == "2. Consideration").unwrap();
        assert!(!c.body.contains("Sales tax"));
    }

    #[test]
    fn as_is_condition_clause() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading == "3. Condition").unwrap();
        assert!(c.body.contains("AS-IS"));
    }

    #[test]
    fn warranted_condition_clause() {
        let c = generate(&BillOfSaleInput { as_is: false, ..base() })
            .clauses
            .into_iter()
            .find(|c| c.heading == "3. Condition")
            .unwrap();
        assert!(c.body.contains("warrants that the property is in good working condition"));
    }

    #[test]
    fn statute_citation_echoed() {
        let b = generate(&BillOfSaleInput {
            statute_citation: "Fla. Stat. § 319.22".into(),
            ..base()
        });
        assert_eq!(b.statutory_citation, "Fla. Stat. § 319.22");
        assert!(b.clauses.iter().any(|c| c.body.contains("Fla. Stat. § 319.22")));
    }

    #[test]
    fn parties_include_addresses_when_present() {
        let p = generate(&base()).clauses.into_iter().find(|c| c.heading == "Parties").unwrap();
        assert!(p.body.contains("Sam Seller, 1 Oak St"));
        assert!(p.body.contains("Bob Buyer, 2 Elm St"));
    }
}
