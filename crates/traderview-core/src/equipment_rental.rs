//! Equipment rental agreement — rents personal property (tools, vehicles, AV
//! gear, machinery) for a fixed period. It computes the rental total from the
//! rate and duration, adds the security deposit for the total due, and computes
//! the return date from the rate period, then assembles the agreement. Distinct
//! from the real-property leases. Drafting aid, not legal advice.

use chrono::{Duration, Months, NaiveDate};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RatePeriod {
    Day,
    Week,
    Month,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EquipmentRentalInput {
    pub owner_name: String,
    pub renter_name: String,
    pub equipment_description: String,
    pub rate_usd: f64,
    pub rate_period: RatePeriod,
    /// Number of rate periods (days / weeks / months).
    pub duration: u32,
    #[serde(default)]
    pub security_deposit_usd: f64,
    pub start_date: String,
    #[serde(default)]
    pub late_fee_per_day_usd: f64,
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
pub struct EquipmentRental {
    pub title: String,
    pub rental_total_usd: f64,
    pub security_deposit_usd: f64,
    pub total_due_usd: f64,
    pub start_date: String,
    pub return_date: String,
    pub statutory_citation: String,
    pub clauses: Vec<DocClause>,
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

fn period_word(p: RatePeriod) -> &'static str {
    match p {
        RatePeriod::Day => "day",
        RatePeriod::Week => "week",
        RatePeriod::Month => "month",
    }
}

pub fn generate(i: &EquipmentRentalInput) -> EquipmentRental {
    let rental_total = cents(i.rate_usd * i.duration as f64);
    let total_due = cents(rental_total + i.security_deposit_usd);

    let return_date = NaiveDate::parse_from_str(&i.start_date, "%Y-%m-%d")
        .ok()
        .and_then(|d| match i.rate_period {
            RatePeriod::Day => Some(d + Duration::days(i.duration as i64)),
            RatePeriod::Week => Some(d + Duration::days(i.duration as i64 * 7)),
            RatePeriod::Month => d.checked_add_months(Months::new(i.duration)),
        })
        .map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_default();

    let citation = i.statute_citation.trim();
    let pursuant = if citation.is_empty() {
        format!("This agreement is governed by the laws of the State of {}.", i.state)
    } else {
        format!("This agreement is governed by the laws of the State of {} ({}).", i.state, citation)
    };

    let pw = period_word(i.rate_period);
    let deposit_body = if i.security_deposit_usd > 0.0 {
        format!(
            "The Renter shall pay a refundable security deposit of {}, returned after the equipment is returned undamaged. Total due at signing: {} (rental {} + deposit {}).",
            money(i.security_deposit_usd), money(total_due), money(rental_total), money(i.security_deposit_usd)
        )
    } else {
        format!("No security deposit is required. Total due: {}.", money(rental_total))
    };

    let late_body = if i.late_fee_per_day_usd > 0.0 {
        format!(" If the equipment is returned after the return date, a late fee of {} per day applies.", money(i.late_fee_per_day_usd))
    } else {
        String::new()
    };

    let clauses = vec![
        DocClause {
            heading: "Parties".into(),
            body: format!("Owner (Lessor): {}\nRenter (Lessee): {}", i.owner_name, i.renter_name),
        },
        DocClause {
            heading: "1. Equipment".into(),
            body: format!("The Owner rents to the Renter the following equipment: {}.", i.equipment_description),
        },
        DocClause {
            heading: "2. Term".into(),
            body: format!(
                "The rental begins {} and the equipment must be returned on or before {} ({} {}(s)).{}",
                i.start_date, return_date, i.duration, pw, late_body
            ),
        },
        DocClause {
            heading: "3. Rental Charges".into(),
            body: format!(
                "The rental rate is {} per {} for {} {}(s), a rental total of {}.",
                money(i.rate_usd), pw, i.duration, pw, money(rental_total)
            ),
        },
        DocClause { heading: "4. Security Deposit".into(), body: deposit_body },
        DocClause {
            heading: "5. Condition and Return".into(),
            body: "The Renter shall use the equipment with reasonable care, return it in the same condition as received (ordinary wear excepted), and is responsible for loss or damage occurring during the rental period.".into(),
        },
        DocClause { heading: "6. Governing Law".into(), body: pursuant },
        DocClause {
            heading: "Signatures".into(),
            body: format!(
                "Owner: ____________________  Date: __________\n{}\n\nRenter: ____________________  Date: __________\n{}",
                i.owner_name, i.renter_name
            ),
        },
    ];

    EquipmentRental {
        title: "Equipment Rental Agreement".into(),
        rental_total_usd: rental_total,
        security_deposit_usd: i.security_deposit_usd,
        total_due_usd: total_due,
        start_date: i.start_date.clone(),
        return_date,
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

    fn base() -> EquipmentRentalInput {
        EquipmentRentalInput {
            owner_name: "Tool Rentals Inc".into(),
            renter_name: "Bob Builder".into(),
            equipment_description: "Mini excavator, model X120".into(),
            rate_usd: 100.0,
            rate_period: RatePeriod::Day,
            duration: 5,
            security_deposit_usd: 200.0,
            start_date: "2026-06-01".into(),
            late_fee_per_day_usd: 25.0,
            state: "Colorado".into(),
            statute_citation: String::new(),
        }
    }

    #[test]
    fn rental_total_and_due() {
        let d = generate(&base());
        assert!(close(d.rental_total_usd, 500.0));
        assert!(close(d.total_due_usd, 700.0));
    }

    #[test]
    fn return_date_daily() {
        // 2026-06-01 + 5 days = 2026-06-06.
        assert_eq!(generate(&base()).return_date, "2026-06-06");
    }

    #[test]
    fn return_date_weekly() {
        let d = generate(&EquipmentRentalInput { rate_period: RatePeriod::Week, duration: 2, ..base() });
        // 2026-06-01 + 14 days = 2026-06-15.
        assert_eq!(d.return_date, "2026-06-15");
        // 100/week × 2 = 200.
        assert!(close(d.rental_total_usd, 200.0));
    }

    #[test]
    fn return_date_monthly() {
        let d = generate(&EquipmentRentalInput { rate_period: RatePeriod::Month, duration: 3, ..base() });
        assert_eq!(d.return_date, "2026-09-01");
    }

    #[test]
    fn late_fee_in_term_clause() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading == "2. Term").unwrap();
        assert!(c.body.contains("late fee of $25.00 per day"));
    }

    #[test]
    fn no_deposit_states_none() {
        let d = generate(&EquipmentRentalInput { security_deposit_usd: 0.0, ..base() });
        assert!(close(d.total_due_usd, 500.0));
        let c = d.clauses.iter().find(|c| c.heading.contains("Security Deposit")).unwrap();
        assert!(c.body.contains("No security deposit"));
    }

    #[test]
    fn statute_citation_echoed() {
        let d = generate(&EquipmentRentalInput { statute_citation: "Colo. Rev. Stat. § 4-2.5".into(), ..base() });
        assert_eq!(d.statutory_citation, "Colo. Rev. Stat. § 4-2.5");
        assert!(d.clauses.iter().any(|c| c.body.contains("Colo. Rev. Stat. § 4-2.5")));
    }
}
