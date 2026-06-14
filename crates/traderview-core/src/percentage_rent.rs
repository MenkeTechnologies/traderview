//! Percentage-rent statement — the retail-lease overage calculation. A tenant
//! pays a fixed base rent plus a percentage of gross sales above a breakpoint.
//! The breakpoint is "natural" (base rent ÷ percentage rate) unless the lease
//! states one. This computes the natural breakpoint, the overage rent, the total
//! rent, and the occupancy-cost ratio (total rent ÷ sales) — the natural-
//! breakpoint and occupancy-ratio computations are specific to retail leasing and
//! appear in no other generator. Drafting aid, not legal/accounting advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct PercentageRentInput {
    pub landlord_name: String,
    pub tenant_name: String,
    #[serde(default)]
    pub property_label: String,
    /// Annual base (minimum) rent.
    pub base_rent_usd: f64,
    /// Percentage-rent rate on sales over the breakpoint, percent.
    pub rate_pct: f64,
    /// Gross sales for the period.
    pub gross_sales_usd: f64,
    /// Stated breakpoint (0 = use the natural breakpoint = base ÷ rate).
    #[serde(default)]
    pub stated_breakpoint_usd: f64,
    /// Period length, in months.
    pub period_months: u32,
    pub date: String,
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
pub struct PercentageRentStatement {
    pub title: String,
    /// base ÷ rate — sales level at which percentage rent begins.
    pub natural_breakpoint_usd: f64,
    /// The breakpoint used (stated if given, else natural).
    pub breakpoint_used_usd: f64,
    /// True when a stated breakpoint overrode the natural one.
    pub stated_breakpoint_applied: bool,
    /// Sales above the breakpoint.
    pub sales_over_breakpoint_usd: f64,
    /// Percentage rent: rate × sales over breakpoint.
    pub overage_rent_usd: f64,
    /// Base + overage.
    pub total_rent_usd: f64,
    /// Total rent ÷ sales, percent.
    pub occupancy_cost_pct: f64,
    pub statutory_citation: String,
    pub clauses: Vec<DocClause>,
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

pub fn generate(i: &PercentageRentInput) -> PercentageRentStatement {
    let natural_bp = if i.rate_pct > 0.0 {
        cents(i.base_rent_usd / (i.rate_pct / 100.0))
    } else {
        0.0
    };
    let stated_applied = i.stated_breakpoint_usd > 0.0;
    let breakpoint = if stated_applied {
        i.stated_breakpoint_usd
    } else {
        natural_bp
    };

    let over = (i.gross_sales_usd - breakpoint).max(0.0);
    let overage = cents(over * i.rate_pct / 100.0);
    let total_rent = cents(i.base_rent_usd + overage);
    let occupancy = if i.gross_sales_usd > 0.0 {
        cents(total_rent / i.gross_sales_usd * 100.0)
    } else {
        0.0
    };

    let property = if i.property_label.trim().is_empty() {
        "the Premises".to_string()
    } else {
        i.property_label.trim().to_string()
    };

    let bp_desc = if stated_applied {
        format!("the stated breakpoint of {}", money(breakpoint))
    } else {
        format!("the natural breakpoint of {} (base rent {} ÷ {:.2}%)", money(natural_bp), money(i.base_rent_usd), i.rate_pct)
    };

    let calc_body = format!(
        "Percentage rent is {:.2}% of gross sales above {}. Gross sales of {} exceed the breakpoint by {}, producing overage rent of {}. Added to base rent of {}, total rent is {} — an occupancy cost of {:.2}% of sales.",
        i.rate_pct,
        bp_desc,
        money(i.gross_sales_usd),
        money(over),
        money(overage),
        money(i.base_rent_usd),
        money(total_rent),
        occupancy
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
                "Landlord: {}\nTenant: {}\nPremises: {}\nPeriod: {} month(s)\nStatement date: {}",
                i.landlord_name, i.tenant_name, property, i.period_months, i.date
            ),
        },
        DocClause {
            heading: "1. Base Rent".into(),
            body: format!("The Tenant pays annual base (minimum) rent of {}.", money(i.base_rent_usd)),
        },
        DocClause {
            heading: "2. Percentage Rent".into(),
            body: format!(
                "In addition to base rent, the Tenant pays percentage rent of {:.2}% of gross sales exceeding {}.",
                i.rate_pct, bp_desc
            ),
        },
        DocClause { heading: "3. Calculation".into(), body: calc_body },
        DocClause {
            heading: "4. Payment & Reporting".into(),
            body: "The Tenant shall report gross sales and pay percentage rent within 30 days after the period. Gross sales exclude returns, sales taxes, and inter-store transfers as defined in the lease.".into(),
        },
        DocClause {
            heading: "5. Records & Audit".into(),
            body: "The Tenant shall keep complete records of gross sales for at least three years; the Landlord may audit them on reasonable notice. An understatement over 3% entitles the Landlord to the audit cost.".into(),
        },
        DocClause { heading: "6. Governing Law".into(), body: governing },
        DocClause {
            heading: "Signatures".into(),
            body: format!(
                "Landlord: ____________________  Date: __________\n{}\n\nTenant: ____________________  Date: __________\n{}",
                i.landlord_name, i.tenant_name
            ),
        },
    ];

    PercentageRentStatement {
        title: "Percentage Rent Statement".into(),
        natural_breakpoint_usd: natural_bp,
        breakpoint_used_usd: cents(breakpoint),
        stated_breakpoint_applied: stated_applied,
        sales_over_breakpoint_usd: cents(over),
        overage_rent_usd: overage,
        total_rent_usd: total_rent,
        occupancy_cost_pct: occupancy,
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

    fn base() -> PercentageRentInput {
        PercentageRentInput {
            landlord_name: "Mall Owners LP".into(),
            tenant_name: "Boutique Tenant LLC".into(),
            property_label: "Store 12".into(),
            base_rent_usd: 120_000.0,
            rate_pct: 6.0,
            gross_sales_usd: 3_000_000.0,
            stated_breakpoint_usd: 0.0,
            period_months: 12,
            date: "2026-07-01".into(),
            state: "Delaware".into(),
            statute_citation: String::new(),
        }
    }

    #[test]
    fn natural_breakpoint_and_overage() {
        let d = generate(&base());
        assert!(close(d.natural_breakpoint_usd, 2_000_000.0));
        assert!(close(d.breakpoint_used_usd, 2_000_000.0));
        assert!(!d.stated_breakpoint_applied);
        assert!(close(d.sales_over_breakpoint_usd, 1_000_000.0));
        assert!(close(d.overage_rent_usd, 60_000.0));
        assert!(close(d.total_rent_usd, 180_000.0));
        assert!(close(d.occupancy_cost_pct, 6.0));
    }

    #[test]
    fn stated_breakpoint_overrides_natural() {
        let d = generate(&PercentageRentInput { stated_breakpoint_usd: 2_500_000.0, ..base() });
        assert!(d.stated_breakpoint_applied);
        assert!(close(d.breakpoint_used_usd, 2_500_000.0));
        assert!(close(d.overage_rent_usd, 30_000.0));
        assert!(close(d.total_rent_usd, 150_000.0));
        assert!(close(d.occupancy_cost_pct, 5.0));
        // Natural breakpoint is still reported for reference.
        assert!(close(d.natural_breakpoint_usd, 2_000_000.0));
    }

    #[test]
    fn below_breakpoint_no_overage() {
        let d = generate(&PercentageRentInput { gross_sales_usd: 1_500_000.0, ..base() });
        assert!(close(d.sales_over_breakpoint_usd, 0.0));
        assert!(close(d.overage_rent_usd, 0.0));
        assert!(close(d.total_rent_usd, 120_000.0));
        assert!(close(d.occupancy_cost_pct, 8.0));
    }

    #[test]
    fn occupancy_ratio_equals_total_over_sales() {
        // total_rent 180,000 ÷ sales 3,000,000 × 100 = 6.00%.
        let i = base();
        let d = generate(&i);
        assert!(close(d.occupancy_cost_pct, d.total_rent_usd / i.gross_sales_usd * 100.0));
    }

    #[test]
    fn audit_clause_present() {
        assert!(generate(&base()).clauses.iter().any(|c| c.body.contains("audit")));
    }

    #[test]
    fn statute_citation_echoed() {
        let d = generate(&PercentageRentInput { statute_citation: "lease § 3.2".into(), ..base() });
        assert_eq!(d.statutory_citation, "lease § 3.2");
        assert!(d.clauses.iter().any(|c| c.body.contains("lease § 3.2")));
    }
}
