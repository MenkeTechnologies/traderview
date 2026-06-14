//! Holdover rent statement — when a tenant stays past lease expiration without a
//! renewal, the lease's holdover clause charges rent at a penalty multiple of the
//! daily base rate (commonly 150%–200%) for each holdover day. This computes the
//! daily base rate, the holdover daily rate, the total holdover charge, and the
//! premium over ordinary rent. No existing generator computes holdover penalty
//! rent. Drafting aid, not legal advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct HoldoverInput {
    pub landlord_name: String,
    pub tenant_name: String,
    #[serde(default)]
    pub property_label: String,
    /// Monthly base rent under the expired lease.
    pub monthly_rent_usd: f64,
    /// Holdover rate as a percent of base (e.g. 150 = 150%).
    pub holdover_pct: f64,
    /// Number of holdover days being charged.
    pub holdover_days: u32,
    /// Days per year for the daily-rate convention (usually 365).
    #[serde(default = "default_days_in_year")]
    pub days_in_year: u32,
    pub lease_end_date: String,
    pub date: String,
    pub state: String,
    #[serde(default)]
    pub statute_citation: String,
}

fn default_days_in_year() -> u32 {
    365
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DocClause {
    pub heading: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct HoldoverRent {
    pub title: String,
    /// Annual rent ÷ days in year.
    pub daily_base_usd: f64,
    /// Daily base × holdover percent.
    pub holdover_daily_usd: f64,
    /// Holdover daily rate × holdover days — total amount owed.
    pub holdover_charge_usd: f64,
    /// What ordinary (100%) rent would have been for those days.
    pub ordinary_charge_usd: f64,
    /// Holdover charge − ordinary charge.
    pub premium_usd: f64,
    pub statutory_citation: String,
    pub clauses: Vec<DocClause>,
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

pub fn generate(i: &HoldoverInput) -> HoldoverRent {
    let diy = if i.days_in_year > 0 { i.days_in_year as f64 } else { 365.0 };
    let daily_base = i.monthly_rent_usd * 12.0 / diy;
    let holdover_daily = daily_base * i.holdover_pct / 100.0;
    let days = i.holdover_days as f64;
    let holdover_charge = cents(holdover_daily * days);
    let ordinary_charge = cents(daily_base * days);
    let premium = cents(holdover_charge - ordinary_charge);

    let property = if i.property_label.trim().is_empty() {
        "the Premises".to_string()
    } else {
        i.property_label.trim().to_string()
    };

    let calc_body = format!(
        "Daily base rent is {} (annualized {} ÷ {} days). At a holdover rate of {:.1}%, the holdover daily rate is {}. For {} holdover day(s) after the lease end of {}, the holdover charge is {} — a premium of {} over the ordinary rent of {}.",
        money(cents(daily_base)),
        money(cents(i.monthly_rent_usd * 12.0)),
        i.days_in_year,
        i.holdover_pct,
        money(cents(holdover_daily)),
        i.holdover_days,
        i.lease_end_date,
        money(holdover_charge),
        money(premium),
        money(ordinary_charge)
    );

    let citation = i.statute_citation.trim();
    let governing = if citation.is_empty() {
        format!("Holdover is governed by the lease and the laws of the State of {}.", i.state)
    } else {
        format!("Holdover is governed by the lease and the laws of the State of {} ({}).", i.state, citation)
    };

    let clauses = vec![
        DocClause {
            heading: "Parties".into(),
            body: format!(
                "Landlord: {}\nTenant: {}\nPremises: {}\nLease end date: {}\nStatement date: {}",
                i.landlord_name, i.tenant_name, property, i.lease_end_date, i.date
            ),
        },
        DocClause {
            heading: "1. Holdover".into(),
            body: format!(
                "The Tenant has remained in possession of {} after the lease ended on {} without a renewal. Under the lease's holdover provision, rent during holdover accrues at {:.1}% of the base rate.",
                property, i.lease_end_date, i.holdover_pct
            ),
        },
        DocClause { heading: "2. Calculation".into(), body: calc_body },
        DocClause {
            heading: "3. Payment".into(),
            body: format!(
                "The Tenant shall pay holdover rent of {} for the period stated. Acceptance of holdover rent does not create a new tenancy or waive the Landlord's right to possession.",
                money(holdover_charge)
            ),
        },
        DocClause { heading: "4. Governing Law".into(), body: governing },
        DocClause {
            heading: "Signatures".into(),
            body: format!(
                "Landlord: ____________________  Date: __________\n{}",
                i.landlord_name
            ),
        },
    ];

    HoldoverRent {
        title: "Holdover Rent Statement".into(),
        daily_base_usd: cents(daily_base),
        holdover_daily_usd: cents(holdover_daily),
        holdover_charge_usd: holdover_charge,
        ordinary_charge_usd: ordinary_charge,
        premium_usd: premium,
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

    fn base() -> HoldoverInput {
        HoldoverInput {
            landlord_name: "Maple Apartments LLC".into(),
            tenant_name: "Holdover Tenant".into(),
            property_label: "Unit 4B".into(),
            monthly_rent_usd: 5_000.0,
            holdover_pct: 150.0,
            holdover_days: 20,
            days_in_year: 365,
            lease_end_date: "2026-06-30".into(),
            date: "2026-07-20".into(),
            state: "Delaware".into(),
            statute_citation: String::new(),
        }
    }

    #[test]
    fn holdover_150_pct() {
        let d = generate(&base());
        assert!(close(d.daily_base_usd, 164.38));
        assert!(close(d.holdover_daily_usd, 246.58));
        assert!(close(d.holdover_charge_usd, 4_931.51));
        assert!(close(d.ordinary_charge_usd, 3_287.67));
        assert!(close(d.premium_usd, 1_643.84));
    }

    #[test]
    fn holdover_200_pct_doubles() {
        let d = generate(&HoldoverInput { holdover_pct: 200.0, holdover_days: 30, ..base() });
        assert!(close(d.holdover_charge_usd, 9_863.01));
        // Premium equals the ordinary charge at 200% (within a cent of independent rounding).
        assert!((d.premium_usd - d.ordinary_charge_usd).abs() < 0.011);
    }

    #[test]
    fn hundred_pct_no_premium() {
        let d = generate(&HoldoverInput { holdover_pct: 100.0, ..base() });
        assert!(close(d.premium_usd, 0.0));
        assert!(close(d.holdover_charge_usd, d.ordinary_charge_usd));
    }

    #[test]
    fn charge_scales_with_days() {
        let d10 = generate(&HoldoverInput { holdover_days: 10, ..base() });
        let d20 = generate(&base());
        // Doubling the days doubles the charge, within a cent of independent rounding.
        assert!((d20.holdover_charge_usd - d10.holdover_charge_usd * 2.0).abs() < 0.011);
    }

    #[test]
    fn statute_citation_echoed() {
        let d = generate(&HoldoverInput { statute_citation: "Cal. Civ. Code § 1945".into(), ..base() });
        assert_eq!(d.statutory_citation, "Cal. Civ. Code § 1945");
        assert!(d.clauses.iter().any(|c| c.body.contains("Cal. Civ. Code § 1945")));
    }
}
