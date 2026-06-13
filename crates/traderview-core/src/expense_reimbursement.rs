//! Expense reimbursement request — an employee's itemized claim for
//! reimbursement of business expenses plus mileage. It sums the itemized
//! expenses, computes the mileage reimbursement (business miles × the mileage
//! rate), and totals the two, then assembles the request with an approval line.
//! Drafting aid, not legal/tax advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct ExpenseItem {
    pub description: String,
    pub amount_usd: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReimbursementInput {
    pub company_name: String,
    pub employee_name: String,
    /// Period or purpose the expenses cover (free text).
    pub period: String,
    #[serde(default)]
    pub expenses: Vec<ExpenseItem>,
    #[serde(default)]
    pub business_miles: f64,
    /// Reimbursement rate per mile (e.g. the IRS standard mileage rate).
    #[serde(default)]
    pub mileage_rate_usd: f64,
    pub submitted_date: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DocClause {
    pub heading: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ReimbursementRequest {
    pub title: String,
    pub expenses_total_usd: f64,
    pub mileage_reimbursement_usd: f64,
    pub total_reimbursement_usd: f64,
    pub item_count: usize,
    pub clauses: Vec<DocClause>,
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

pub fn generate(i: &ReimbursementInput) -> ReimbursementRequest {
    let expenses_total = cents(i.expenses.iter().map(|e| e.amount_usd).sum());
    let mileage_reimbursement = cents(i.business_miles * i.mileage_rate_usd);
    let total = cents(expenses_total + mileage_reimbursement);

    let items_body = if i.expenses.is_empty() {
        "No itemized expenses submitted.".to_string()
    } else {
        let mut lines: Vec<String> = i
            .expenses
            .iter()
            .map(|e| format!("  • {} — {}", e.description, money(e.amount_usd)))
            .collect();
        lines.push(format!("Itemized subtotal: {}", money(expenses_total)));
        lines.join("\n")
    };

    let mileage_body = if i.business_miles > 0.0 {
        format!(
            "{:.1} business miles at {} per mile = {}.",
            i.business_miles,
            money(i.mileage_rate_usd),
            money(mileage_reimbursement)
        )
    } else {
        "No mileage claimed.".to_string()
    };

    let clauses = vec![
        DocClause {
            heading: "Header".into(),
            body: format!(
                "Employee: {}\nCompany: {}\nPeriod / purpose: {}\nSubmitted: {}",
                i.employee_name, i.company_name, i.period, i.submitted_date
            ),
        },
        DocClause { heading: "1. Itemized Expenses".into(), body: items_body },
        DocClause { heading: "2. Mileage".into(), body: mileage_body },
        DocClause {
            heading: "3. Total Reimbursement".into(),
            body: format!(
                "Total reimbursement requested: {} (expenses {} + mileage {}).",
                money(total),
                money(expenses_total),
                money(mileage_reimbursement)
            ),
        },
        DocClause {
            heading: "4. Certification".into(),
            body: "I certify that the expenses above were incurred for legitimate business purposes and that receipts are attached where required by company policy.".into(),
        },
        DocClause {
            heading: "Approval".into(),
            body: format!(
                "Employee: ____________________  Date: {}\n{}\n\nApproved by: ____________________  Date: __________",
                i.submitted_date, i.employee_name
            ),
        },
    ];

    ReimbursementRequest {
        title: "Expense Reimbursement Request".into(),
        expenses_total_usd: expenses_total,
        mileage_reimbursement_usd: mileage_reimbursement,
        total_reimbursement_usd: total,
        item_count: i.expenses.len(),
        clauses,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    fn item(d: &str, a: f64) -> ExpenseItem {
        ExpenseItem { description: d.into(), amount_usd: a }
    }

    fn base() -> ReimbursementInput {
        ReimbursementInput {
            company_name: "Acme Inc".into(),
            employee_name: "Dana Road".into(),
            period: "June 2026 client trip".into(),
            expenses: vec![item("Hotel", 120.0), item("Meals", 50.0), item("Parking", 30.0)],
            business_miles: 100.0,
            mileage_rate_usd: 0.67,
            submitted_date: "2026-06-30".into(),
        }
    }

    #[test]
    fn totals() {
        let d = generate(&base());
        assert!(close(d.expenses_total_usd, 200.0));
        assert!(close(d.mileage_reimbursement_usd, 67.0));
        assert!(close(d.total_reimbursement_usd, 267.0));
        assert_eq!(d.item_count, 3);
    }

    #[test]
    fn itemized_lines_and_subtotal() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading.contains("Itemized")).unwrap();
        assert!(c.body.contains("Hotel — $120.00"));
        assert!(c.body.contains("Itemized subtotal: $200.00"));
    }

    #[test]
    fn mileage_clause() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading == "2. Mileage").unwrap();
        assert!(c.body.contains("100.0 business miles at $0.67 per mile = $67.00"));
    }

    #[test]
    fn no_mileage_states_none() {
        let d = generate(&ReimbursementInput { business_miles: 0.0, ..base() });
        assert!(close(d.mileage_reimbursement_usd, 0.0));
        assert!(close(d.total_reimbursement_usd, 200.0));
        let c = d.clauses.iter().find(|c| c.heading == "2. Mileage").unwrap();
        assert!(c.body.contains("No mileage"));
    }

    #[test]
    fn no_expenses_states_none() {
        let d = generate(&ReimbursementInput { expenses: vec![], ..base() });
        assert!(close(d.expenses_total_usd, 0.0));
        let c = d.clauses.iter().find(|c| c.heading.contains("Itemized")).unwrap();
        assert!(c.body.contains("No itemized expenses"));
    }

    #[test]
    fn total_clause_breaks_down() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading.contains("Total Reimbursement")).unwrap();
        assert!(c.body.contains("$267.00"));
        assert!(c.body.contains("expenses $200.00 + mileage $67.00"));
    }
}
