//! Operating-expense escalation (base-year stop) — the expense pass-through of a
//! full-service / gross commercial lease. Unlike CAM reconciliation, which passes
//! through the tenant's full pro-rata share of actual expenses, a base-year lease
//! passes through only the tenant's share of the *increase* over a base-year
//! amount (the "expense stop"). It also applies the standard gross-up: expenses
//! that vary with occupancy are scaled to a target occupancy so a partly-vacant
//! year does not understate the stop. The gross-up + base-year-increment
//! computation appears in no other generator. Drafting aid, not legal/accounting
//! advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct OpexEscalationInput {
    pub landlord_name: String,
    pub tenant_name: String,
    #[serde(default)]
    pub property_label: String,
    /// Tenant's rentable square footage.
    pub tenant_sqft: f64,
    /// Building's total rentable square footage.
    pub building_sqft: f64,
    /// Base-year operating expenses (the stop).
    pub base_year_opex_usd: f64,
    /// Current-year operating expenses (before gross-up).
    pub current_opex_usd: f64,
    /// Portion of current expenses that varies with occupancy, percent.
    #[serde(default)]
    pub variable_pct: f64,
    /// Actual occupancy of the building, percent.
    #[serde(default = "default_occupancy")]
    pub actual_occupancy_pct: f64,
    /// Gross-up target occupancy (e.g. 95%).
    #[serde(default = "default_occupancy")]
    pub gross_up_occupancy_pct: f64,
    pub year: String,
    pub date: String,
    pub state: String,
    #[serde(default)]
    pub statute_citation: String,
}

fn default_occupancy() -> f64 {
    100.0
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DocClause {
    pub heading: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct OpexEscalation {
    pub title: String,
    /// Tenant's pro-rata share of the building, percent.
    pub pro_rata_pct: f64,
    /// Gross-up factor applied to the variable expenses (1.0 = none).
    pub gross_up_factor: f64,
    /// Current expenses after gross-up.
    pub grossed_up_opex_usd: f64,
    /// Grossed-up current expenses minus the base year (floored at 0).
    pub increment_over_base_usd: f64,
    /// Tenant's pro-rata share of the increment — what they owe.
    pub tenant_share_usd: f64,
    pub statutory_citation: String,
    pub clauses: Vec<DocClause>,
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

pub fn generate(i: &OpexEscalationInput) -> OpexEscalation {
    let pro_rata = if i.building_sqft > 0.0 {
        i.tenant_sqft / i.building_sqft
    } else {
        0.0
    };

    let variable = i.current_opex_usd * i.variable_pct / 100.0;
    let fixed = i.current_opex_usd - variable;
    // Gross up only when actual occupancy is below the target (never gross down).
    let factor = if i.actual_occupancy_pct > 0.0 && i.actual_occupancy_pct < i.gross_up_occupancy_pct {
        i.gross_up_occupancy_pct / i.actual_occupancy_pct
    } else {
        1.0
    };
    let grossed = cents(fixed + variable * factor);
    let increment = (grossed - i.base_year_opex_usd).max(0.0);
    let tenant_share = cents(pro_rata * increment);

    let property = if i.property_label.trim().is_empty() {
        "the Premises".to_string()
    } else {
        i.property_label.trim().to_string()
    };

    let grossup_line = if factor > 1.0 {
        format!(
            " Variable expenses ({:.1}% of the total) are grossed up by a factor of {:.4} from {:.1}% actual to {:.1}% target occupancy, raising current expenses to {}.",
            i.variable_pct,
            (factor * 10_000.0).round() / 10_000.0,
            i.actual_occupancy_pct,
            i.gross_up_occupancy_pct,
            money(grossed)
        )
    } else {
        format!(" No gross-up applies (occupancy at or above the {:.1}% target).", i.gross_up_occupancy_pct)
    };

    let calc_body = format!(
        "Current operating expenses for {} are {}.{} The grossed-up expenses of {} exceed the base-year stop of {} by {}. The Tenant's pro-rata share of {:.2}% of that increment is {}.",
        i.year,
        money(i.current_opex_usd),
        grossup_line,
        money(grossed),
        money(i.base_year_opex_usd),
        money(cents(increment)),
        cents(pro_rata * 100.0),
        money(tenant_share)
    );

    let citation = i.statute_citation.trim();
    let governing = if citation.is_empty() {
        format!("This statement is governed by the lease and the laws of the State of {}.", i.state)
    } else {
        format!("This statement is governed by the lease and the laws of the State of {} ({}).", i.state, citation)
    };

    let clauses = vec![
        DocClause {
            heading: "Parties".into(),
            body: format!(
                "Landlord: {}\nTenant: {}\nPremises: {}\nExpense year: {}\nStatement date: {}",
                i.landlord_name, i.tenant_name, property, i.year, i.date
            ),
        },
        DocClause {
            heading: "1. Base Year".into(),
            body: format!(
                "The Tenant pays its pro-rata share of operating expenses above the base-year amount of {} (the expense stop). The Tenant's pro-rata share is {:.2}%.",
                money(i.base_year_opex_usd),
                cents(pro_rata * 100.0)
            ),
        },
        DocClause {
            heading: "2. Gross-Up".into(),
            body: format!(
                "Expenses that vary with occupancy are grossed up to {:.1}% occupancy so that the base year and each comparison year are measured on the same basis.{}",
                i.gross_up_occupancy_pct,
                if factor > 1.0 { format!(" This year's factor is {:.4}.", (factor * 10_000.0).round() / 10_000.0) } else { String::new() }
            ),
        },
        DocClause { heading: "3. Calculation".into(), body: calc_body },
        DocClause {
            heading: "4. Payment".into(),
            body: format!(
                "The Tenant shall pay its share of {} within 30 days of this statement, net of any estimated expense payments already made for the year.",
                money(tenant_share)
            ),
        },
        DocClause { heading: "5. Governing Law".into(), body: governing },
        DocClause {
            heading: "Signatures".into(),
            body: format!(
                "Landlord: ____________________  Date: __________\n{}",
                i.landlord_name
            ),
        },
    ];

    OpexEscalation {
        title: "Operating Expense Escalation Statement".into(),
        pro_rata_pct: cents(pro_rata * 100.0),
        gross_up_factor: (factor * 10_000.0).round() / 10_000.0,
        grossed_up_opex_usd: grossed,
        increment_over_base_usd: cents(increment),
        tenant_share_usd: tenant_share,
        statutory_citation: citation.to_string(),
        clauses,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    fn base() -> OpexEscalationInput {
        OpexEscalationInput {
            landlord_name: "Plaza Owners LP".into(),
            tenant_name: "Office Tenant LLC".into(),
            property_label: "Suite 500".into(),
            tenant_sqft: 5_000.0,
            building_sqft: 50_000.0,
            base_year_opex_usd: 500_000.0,
            current_opex_usd: 600_000.0,
            variable_pct: 60.0,
            actual_occupancy_pct: 80.0,
            gross_up_occupancy_pct: 95.0,
            year: "2025".into(),
            date: "2026-07-01".into(),
            state: "Delaware".into(),
            statute_citation: String::new(),
        }
    }

    #[test]
    fn gross_up_and_increment() {
        let d = generate(&base());
        assert!(close(d.pro_rata_pct, 10.0));
        assert!(close(d.gross_up_factor, 1.1875));
        assert!(close(d.grossed_up_opex_usd, 667_500.0));
        assert!(close(d.increment_over_base_usd, 167_500.0));
        assert!(close(d.tenant_share_usd, 16_750.0));
    }

    #[test]
    fn no_gross_up_at_target_occupancy() {
        let d = generate(&OpexEscalationInput { actual_occupancy_pct: 95.0, ..base() });
        assert!(close(d.gross_up_factor, 1.0));
        assert!(close(d.grossed_up_opex_usd, 600_000.0));
        assert!(close(d.increment_over_base_usd, 100_000.0));
        assert!(close(d.tenant_share_usd, 10_000.0));
    }

    #[test]
    fn no_gross_down_above_target() {
        let d = generate(&OpexEscalationInput { actual_occupancy_pct: 100.0, ..base() });
        assert!(close(d.gross_up_factor, 1.0));
        assert!(close(d.grossed_up_opex_usd, 600_000.0));
    }

    #[test]
    fn below_base_year_no_charge() {
        let d = generate(&OpexEscalationInput { current_opex_usd: 450_000.0, actual_occupancy_pct: 100.0, ..base() });
        assert!(close(d.increment_over_base_usd, 0.0));
        assert!(close(d.tenant_share_usd, 0.0));
    }

    #[test]
    fn zero_building_sqft_no_divide_by_zero() {
        let d = generate(&OpexEscalationInput { building_sqft: 0.0, ..base() });
        assert!(close(d.pro_rata_pct, 0.0));
        assert!(close(d.tenant_share_usd, 0.0));
    }

    #[test]
    fn statute_citation_echoed() {
        let d = generate(&OpexEscalationInput { statute_citation: "lease § 6.2".into(), ..base() });
        assert_eq!(d.statutory_citation, "lease § 6.2");
        assert!(d.clauses.iter().any(|c| c.body.contains("lease § 6.2")));
    }
}
