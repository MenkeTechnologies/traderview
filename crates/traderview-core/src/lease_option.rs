//! Lease-option (rent-to-own) agreement — a tenant leases with an option to buy
//! at an agreed price, with the up-front option fee and a portion of each rent
//! payment credited toward the purchase. It computes the accumulated rent
//! credits over the option period, the total credits, the net price at exercise,
//! and the option expiration date. Distinct from a plain lease or purchase
//! agreement. Drafting aid, not legal advice.

use chrono::{Months, NaiveDate};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct LeaseOptionInput {
    pub landlord_name: String,
    pub tenant_name: String,
    pub property_address: String,
    pub option_fee_usd: f64,
    /// Whether the option fee is credited toward the purchase price.
    #[serde(default = "default_true")]
    pub option_fee_credited: bool,
    pub monthly_rent_usd: f64,
    /// Portion of each month's rent credited toward the purchase.
    #[serde(default)]
    pub monthly_rent_credit_usd: f64,
    pub option_start_date: String,
    pub option_months: u32,
    pub purchase_price_usd: f64,
    pub state: String,
    #[serde(default)]
    pub statute_citation: String,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DocClause {
    pub heading: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct LeaseOption {
    pub title: String,
    pub option_end_date: String,
    pub total_rent_credits_usd: f64,
    pub total_credits_usd: f64,
    pub purchase_price_usd: f64,
    pub net_price_at_exercise_usd: f64,
    pub statutory_citation: String,
    pub clauses: Vec<DocClause>,
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

pub fn generate(i: &LeaseOptionInput) -> LeaseOption {
    let total_rent_credits = cents(i.monthly_rent_credit_usd * i.option_months as f64);
    let fee_credit = if i.option_fee_credited { i.option_fee_usd } else { 0.0 };
    let total_credits = cents(total_rent_credits + fee_credit);
    let net_price = cents((i.purchase_price_usd - total_credits).max(0.0));

    let option_end = NaiveDate::parse_from_str(&i.option_start_date, "%Y-%m-%d")
        .ok()
        .and_then(|d| d.checked_add_months(Months::new(i.option_months)))
        .map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_default();

    let citation = i.statute_citation.trim();
    let pursuant = if citation.is_empty() {
        format!("This agreement is governed by the laws of the State of {}.", i.state)
    } else {
        format!("This agreement is governed by the laws of the State of {} ({}).", i.state, citation)
    };

    let fee_credit_note = if i.option_fee_credited {
        format!("The option fee of {} is credited toward the purchase price if the option is exercised.", money(i.option_fee_usd))
    } else {
        format!("The option fee of {} is non-refundable and is NOT credited toward the purchase price.", money(i.option_fee_usd))
    };

    let clauses = vec![
        DocClause {
            heading: "Parties".into(),
            body: format!(
                "Landlord/Optionor: {}\nTenant/Optionee: {}\nProperty: {}",
                i.landlord_name, i.tenant_name, i.property_address
            ),
        },
        DocClause {
            heading: "1. Grant of Option".into(),
            body: format!(
                "For an option fee of {}, the Landlord grants the Tenant the exclusive option to purchase the property, exercisable from {} through {} ({} months). {}",
                money(i.option_fee_usd), i.option_start_date, option_end, i.option_months, fee_credit_note
            ),
        },
        DocClause {
            heading: "2. Lease Terms".into(),
            body: format!(
                "During the option period the Tenant leases the property at {} per month, of which {} per month is credited toward the purchase price. Over {} months that is {} in rent credits.",
                money(i.monthly_rent_usd), money(i.monthly_rent_credit_usd), i.option_months, money(total_rent_credits)
            ),
        },
        DocClause {
            heading: "3. Purchase Price and Credits".into(),
            body: format!(
                "The agreed purchase price is {}. Total credits if the option is exercised at the end of the term: {} (rent credits {}{}). Net price at exercise: {}.",
                money(i.purchase_price_usd),
                money(total_credits),
                money(total_rent_credits),
                if i.option_fee_credited { format!(" + option fee {}", money(i.option_fee_usd)) } else { String::new() },
                money(net_price)
            ),
        },
        DocClause {
            heading: "4. Exercise".into(),
            body: "The Tenant may exercise the option by written notice to the Landlord before the expiration date and by proceeding to closing on customary terms. If the option is not exercised, it expires and the Landlord retains the option fee.".into(),
        },
        DocClause { heading: "5. Governing Law".into(), body: pursuant },
        DocClause {
            heading: "Signatures".into(),
            body: format!(
                "Landlord: ____________________  Date: __________\n{}\n\nTenant: ____________________  Date: __________\n{}",
                i.landlord_name, i.tenant_name
            ),
        },
    ];

    LeaseOption {
        title: "Lease-Option (Rent-to-Own) Agreement".into(),
        option_end_date: option_end,
        total_rent_credits_usd: total_rent_credits,
        total_credits_usd: total_credits,
        purchase_price_usd: i.purchase_price_usd,
        net_price_at_exercise_usd: net_price,
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

    fn base() -> LeaseOptionInput {
        LeaseOptionInput {
            landlord_name: "Owen Owner".into(),
            tenant_name: "Tina Tenant".into(),
            property_address: "7 Hill Rd".into(),
            option_fee_usd: 5_000.0,
            option_fee_credited: true,
            monthly_rent_usd: 2_000.0,
            monthly_rent_credit_usd: 300.0,
            option_start_date: "2026-08-01".into(),
            option_months: 24,
            purchase_price_usd: 350_000.0,
            state: "Georgia".into(),
            statute_citation: String::new(),
        }
    }

    #[test]
    fn credits_and_net_price() {
        let d = generate(&base());
        assert!(close(d.total_rent_credits_usd, 7_200.0));
        assert!(close(d.total_credits_usd, 12_200.0));
        assert!(close(d.net_price_at_exercise_usd, 337_800.0));
    }

    #[test]
    fn option_end_date() {
        // 2026-08-01 + 24 months = 2028-08-01.
        assert_eq!(generate(&base()).option_end_date, "2028-08-01");
    }

    #[test]
    fn uncredited_fee_excluded_from_credits() {
        let d = generate(&LeaseOptionInput { option_fee_credited: false, ..base() });
        // Only rent credits count: 7,200; net 342,800.
        assert!(close(d.total_credits_usd, 7_200.0));
        assert!(close(d.net_price_at_exercise_usd, 342_800.0));
        let c = d.clauses.iter().find(|c| c.heading == "1. Grant of Option").unwrap();
        assert!(c.body.contains("NOT credited"));
    }

    #[test]
    fn credit_clause_shows_breakdown() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading.contains("Purchase Price")).unwrap();
        assert!(c.body.contains("Net price at exercise: $337800.00"));
        assert!(c.body.contains("option fee $5000.00"));
    }

    #[test]
    fn lease_clause_shows_rent_credit() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading == "2. Lease Terms").unwrap();
        assert!(c.body.contains("$300.00 per month is credited"));
        assert!(c.body.contains("$7200.00 in rent credits"));
    }

    #[test]
    fn statute_citation_echoed() {
        let d = generate(&LeaseOptionInput { statute_citation: "O.C.G.A. § 44-7".into(), ..base() });
        assert_eq!(d.statutory_citation, "O.C.G.A. § 44-7");
        assert!(d.clauses.iter().any(|c| c.body.contains("O.C.G.A. § 44-7")));
    }

    #[test]
    fn bad_date_yields_empty_end() {
        let d = generate(&LeaseOptionInput { option_start_date: "x".into(), ..base() });
        assert_eq!(d.option_end_date, "");
    }
}
