//! Leasing commission agreement — the brokerage fee on a commercial *lease*,
//! distinct from a sales commission (rate × sale price). A leasing commission is
//! a tiered percentage applied per lease-year to that year's rent, summed over
//! the term — the rate typically steps down after the early years (e.g. 5% of
//! years 1–5, 2.5% of years 6–10) and the rent escalates annually. It computes
//! the per-year schedule, the aggregate rent, the total commission, and the
//! blended effective rate. No existing generator computes a tiered commission on
//! an escalating rent stream. Drafting aid, not legal advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct LeasingCommissionInput {
    pub broker_name: String,
    pub landlord_name: String,
    pub tenant_name: String,
    #[serde(default)]
    pub property_label: String,
    /// First-year annual base rent.
    pub year_one_rent_usd: f64,
    /// Annual rent escalation, percent.
    #[serde(default)]
    pub annual_escalation_pct: f64,
    /// Lease term in years.
    pub term_years: u32,
    /// Commission rate for the early tier, percent.
    pub tier1_rate_pct: f64,
    /// Number of years the tier-1 rate applies (the rest use tier 2).
    pub tier1_years: u32,
    /// Commission rate for the later tier, percent.
    #[serde(default)]
    pub tier2_rate_pct: f64,
    pub date: String,
    pub state: String,
    #[serde(default)]
    pub statute_citation: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct YearRow {
    pub year: u32,
    pub rent_usd: f64,
    pub rate_pct: f64,
    pub commission_usd: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DocClause {
    pub heading: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct LeasingCommission {
    pub title: String,
    /// Sum of each year's rent over the term.
    pub aggregate_rent_usd: f64,
    /// Sum of each year's commission.
    pub total_commission_usd: f64,
    /// Blended commission rate over the aggregate rent, percent.
    pub effective_rate_pct: f64,
    pub schedule: Vec<YearRow>,
    pub statutory_citation: String,
    pub clauses: Vec<DocClause>,
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn round4(x: f64) -> f64 {
    (x * 10_000.0).round() / 10_000.0
}

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

pub fn generate(i: &LeasingCommissionInput) -> LeasingCommission {
    let esc = i.annual_escalation_pct / 100.0;
    let mut aggregate = 0.0;
    let mut total = 0.0;
    let mut schedule = Vec::with_capacity(i.term_years as usize);

    for y in 1..=i.term_years {
        let rent = cents(i.year_one_rent_usd * (1.0 + esc).powi((y - 1) as i32));
        let rate = if y <= i.tier1_years {
            i.tier1_rate_pct
        } else {
            i.tier2_rate_pct
        };
        let commission = cents(rent * rate / 100.0);
        aggregate += rent;
        total += commission;
        schedule.push(YearRow { year: y, rent_usd: rent, rate_pct: rate, commission_usd: commission });
    }

    let aggregate = cents(aggregate);
    let total = cents(total);
    let effective = if aggregate > 0.0 {
        round4(total / aggregate * 100.0)
    } else {
        0.0
    };

    let property = if i.property_label.trim().is_empty() {
        "the Premises".to_string()
    } else {
        i.property_label.trim().to_string()
    };

    let tier_desc = if i.tier1_years < i.term_years {
        format!(
            "{:.3}% of rent in years 1–{} and {:.3}% thereafter",
            i.tier1_rate_pct, i.tier1_years, i.tier2_rate_pct
        )
    } else {
        format!("{:.3}% of rent over the term", i.tier1_rate_pct)
    };

    let calc_body = format!(
        "The commission is {}. Over a {}-year term with first-year rent of {} escalating {:.2}% per year, the aggregate rent is {} and the total commission is {} — a blended {:.4}% of aggregate rent.",
        tier_desc,
        i.term_years,
        money(i.year_one_rent_usd),
        i.annual_escalation_pct,
        money(aggregate),
        money(total),
        effective
    );

    let citation = i.statute_citation.trim();
    let governing = if citation.is_empty() {
        format!("This agreement is governed by the laws of the State of {}.", i.state)
    } else {
        format!("This agreement is governed by the laws of the State of {} ({}).", i.state, citation)
    };

    let clauses = vec![
        DocClause {
            heading: "Parties".into(),
            body: format!(
                "Broker: {}\nLandlord: {}\nTenant: {}\nPremises: {}\nDate: {}",
                i.broker_name, i.landlord_name, i.tenant_name, property, i.date
            ),
        },
        DocClause {
            heading: "1. Engagement".into(),
            body: format!(
                "The Landlord engages the Broker, who procured the Tenant for {}, and shall pay the leasing commission computed below.",
                property
            ),
        },
        DocClause { heading: "2. Commission".into(), body: format!("The commission is {}.", tier_desc) },
        DocClause { heading: "3. Calculation".into(), body: calc_body },
        DocClause {
            heading: "4. Payment".into(),
            body: format!(
                "The total commission of {} is earned on lease execution and payable one-half on execution and one-half on the Tenant's occupancy, unless the parties agree otherwise.",
                money(total)
            ),
        },
        DocClause { heading: "5. Governing Law".into(), body: governing },
        DocClause {
            heading: "Signatures".into(),
            body: format!(
                "Landlord: ____________________  Date: __________\n{}\n\nBroker: ____________________  Date: __________\n{}",
                i.landlord_name, i.broker_name
            ),
        },
    ];

    LeasingCommission {
        title: "Leasing Commission Agreement".into(),
        aggregate_rent_usd: aggregate,
        total_commission_usd: total,
        effective_rate_pct: effective,
        schedule,
        statutory_citation: citation.to_string(),
        clauses,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.02
    }

    fn base() -> LeasingCommissionInput {
        LeasingCommissionInput {
            broker_name: "Acme Realty".into(),
            landlord_name: "Tower Owners LP".into(),
            tenant_name: "Office Tenant LLC".into(),
            property_label: "Suite 700".into(),
            year_one_rent_usd: 100_000.0,
            annual_escalation_pct: 3.0,
            term_years: 10,
            tier1_rate_pct: 5.0,
            tier1_years: 5,
            tier2_rate_pct: 2.5,
            date: "2026-07-01".into(),
            state: "Delaware".into(),
            statute_citation: String::new(),
        }
    }

    #[test]
    fn tiered_escalating_totals() {
        let d = generate(&base());
        assert_eq!(d.schedule.len(), 10);
        assert!(close(d.aggregate_rent_usd, 1_146_387.93));
        assert!(close(d.total_commission_usd, 41_932.54));
        assert!(close(d.effective_rate_pct, 3.6578));
    }

    #[test]
    fn tier_boundary_rates() {
        let d = generate(&base());
        assert!(close(d.schedule[4].rate_pct, 5.0)); // year 5
        assert!(close(d.schedule[5].rate_pct, 2.5)); // year 6
    }

    #[test]
    fn first_year_rent_unescalated() {
        let d = generate(&base());
        assert!(close(d.schedule[0].rent_usd, 100_000.0));
        assert!(close(d.schedule[0].commission_usd, 5_000.0));
    }

    #[test]
    fn flat_single_tier_no_escalation() {
        let d = generate(&LeasingCommissionInput {
            annual_escalation_pct: 0.0,
            term_years: 5,
            tier1_years: 5,
            ..base()
        });
        // 5 × 100,000 = 500,000 aggregate; 5% = 25,000; effective 5%.
        assert!(close(d.aggregate_rent_usd, 500_000.0));
        assert!(close(d.total_commission_usd, 25_000.0));
        assert!(close(d.effective_rate_pct, 5.0));
    }

    #[test]
    fn commission_sums_schedule() {
        let d = generate(&base());
        let sum: f64 = d.schedule.iter().map(|r| r.commission_usd).sum();
        assert!(close(sum, d.total_commission_usd));
    }

    #[test]
    fn statute_citation_echoed() {
        let d = generate(&LeasingCommissionInput { statute_citation: "Cal. Civ. Code § 1090".into(), ..base() });
        assert_eq!(d.statutory_citation, "Cal. Civ. Code § 1090");
        assert!(d.clauses.iter().any(|c| c.body.contains("Cal. Civ. Code § 1090")));
    }
}
