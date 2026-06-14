//! Prorated rent statement — the partial-month rent owed when a tenant moves in
//! or out mid-month. Rent is prorated on the actual number of days in that
//! calendar month (so February, 30-day, and 31-day months each compute
//! correctly), counting the days the tenant occupies the unit. Distinct from the
//! holdover statement, which charges a penalty multiple on an annual/365 daily
//! basis; this is the ordinary daily rate for the specific month. Drafting aid,
//! not legal advice.

use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct ProratedRentInput {
    pub landlord_name: String,
    pub tenant_name: String,
    #[serde(default)]
    pub property_label: String,
    /// Full monthly rent.
    pub monthly_rent_usd: f64,
    /// "move_in" (occupy from the date through month end) or "move_out"
    /// (occupy from month start through the date).
    #[serde(default)]
    pub mode: String,
    /// The move-in or move-out date (YYYY-MM-DD).
    pub event_date: String,
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
pub struct ProratedRent {
    pub title: String,
    /// Actual days in the event month.
    pub days_in_month: u32,
    /// Days the tenant occupies the unit in that month.
    pub occupied_days: u32,
    /// Monthly rent ÷ days in month.
    pub daily_rate_usd: f64,
    /// Daily rate × occupied days — the prorated rent owed.
    pub prorated_rent_usd: f64,
    pub statutory_citation: String,
    pub clauses: Vec<DocClause>,
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

/// Number of days in the calendar month containing `d`.
fn days_in_month(d: NaiveDate) -> u32 {
    let (y, m) = (d.year(), d.month());
    let (ny, nm) = if m == 12 { (y + 1, 1) } else { (y, m + 1) };
    // First day of next month, stepped back one day, is the last day of this one.
    NaiveDate::from_ymd_opt(ny, nm, 1)
        .and_then(|first_next| first_next.pred_opt())
        .map(|last| last.day())
        .unwrap_or(30)
}

pub fn generate(i: &ProratedRentInput) -> ProratedRent {
    let is_move_out = i.mode.trim().eq_ignore_ascii_case("move_out");

    let (dim, occupied) = match NaiveDate::parse_from_str(&i.event_date, "%Y-%m-%d") {
        Ok(d) => {
            let dim = days_in_month(d);
            let occ = if is_move_out {
                // Month start through the move-out day, inclusive.
                d.day()
            } else {
                // Move-in day through month end, inclusive.
                dim - d.day() + 1
            };
            (dim, occ)
        }
        Err(_) => (30, 0),
    };

    let daily = if dim > 0 { i.monthly_rent_usd / dim as f64 } else { 0.0 };
    let prorated = cents(daily * occupied as f64);

    let property = if i.property_label.trim().is_empty() {
        "the Premises".to_string()
    } else {
        i.property_label.trim().to_string()
    };

    let event = if is_move_out { "move-out" } else { "move-in" };
    let span = if is_move_out {
        format!("from the first of the month through the {} date", event)
    } else {
        format!("from the {} date through the end of the month", event)
    };

    let calc_body = format!(
        "The {} month has {} days, so the daily rate is {} ({} ÷ {}). Occupying {} day(s) {}, the prorated rent is {}.",
        event,
        dim,
        money(cents(daily)),
        money(i.monthly_rent_usd),
        dim,
        occupied,
        span,
        money(prorated)
    );

    let citation = i.statute_citation.trim();
    let governing = if citation.is_empty() {
        format!("This proration is governed by the lease and the laws of the State of {}.", i.state)
    } else {
        format!("This proration is governed by the lease and the laws of the State of {} ({}).", i.state, citation)
    };

    let clauses = vec![
        DocClause {
            heading: "Parties".into(),
            body: format!(
                "Landlord: {}\nTenant: {}\nPremises: {}\n{} date: {}",
                i.landlord_name,
                i.tenant_name,
                property,
                if is_move_out { "Move-out" } else { "Move-in" },
                i.event_date
            ),
        },
        DocClause {
            heading: "1. Proration".into(),
            body: format!(
                "Rent for the partial month is prorated on a daily basis using the actual {} days in the month. Full monthly rent is {}.",
                dim, money(i.monthly_rent_usd)
            ),
        },
        DocClause { heading: "2. Calculation".into(), body: calc_body },
        DocClause {
            heading: "3. Payment".into(),
            body: format!(
                "The prorated rent of {} is due for the partial month. Full monthly rent of {} applies to each full month thereafter.",
                money(prorated), money(i.monthly_rent_usd)
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

    ProratedRent {
        title: "Prorated Rent Statement".into(),
        days_in_month: dim,
        occupied_days: occupied,
        daily_rate_usd: cents(daily),
        prorated_rent_usd: prorated,
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

    fn base() -> ProratedRentInput {
        ProratedRentInput {
            landlord_name: "Maple Apartments LLC".into(),
            tenant_name: "New Tenant".into(),
            property_label: "Unit 4B".into(),
            monthly_rent_usd: 3_000.0,
            mode: "move_in".into(),
            event_date: "2026-06-15".into(),
            state: "Delaware".into(),
            statute_citation: String::new(),
        }
    }

    #[test]
    fn move_in_mid_month() {
        let d = generate(&base());
        assert_eq!(d.days_in_month, 30);
        assert_eq!(d.occupied_days, 16); // 30 − 15 + 1
        assert!(close(d.daily_rate_usd, 100.0));
        assert!(close(d.prorated_rent_usd, 1_600.0));
    }

    #[test]
    fn move_in_february_28_days() {
        let d = generate(&ProratedRentInput { monthly_rent_usd: 2_800.0, event_date: "2026-02-20".into(), ..base() });
        assert_eq!(d.days_in_month, 28);
        assert_eq!(d.occupied_days, 9); // 28 − 20 + 1
        assert!(close(d.prorated_rent_usd, 900.0));
    }

    #[test]
    fn move_out_mid_month() {
        let d = generate(&ProratedRentInput { mode: "move_out".into(), event_date: "2026-06-10".into(), ..base() });
        assert_eq!(d.occupied_days, 10);
        assert!(close(d.prorated_rent_usd, 1_000.0));
    }

    #[test]
    fn move_in_last_day_is_one_day() {
        let d = generate(&ProratedRentInput { monthly_rent_usd: 3_100.0, event_date: "2026-01-31".into(), ..base() });
        assert_eq!(d.days_in_month, 31);
        assert_eq!(d.occupied_days, 1);
        assert!(close(d.prorated_rent_usd, 100.0));
    }

    #[test]
    fn full_month_move_in_first() {
        let d = generate(&ProratedRentInput { event_date: "2026-06-01".into(), ..base() });
        assert_eq!(d.occupied_days, 30); // whole month
        assert!(close(d.prorated_rent_usd, 3_000.0));
    }

    #[test]
    fn statute_citation_echoed() {
        let d = generate(&ProratedRentInput { statute_citation: "lease § 2".into(), ..base() });
        assert_eq!(d.statutory_citation, "lease § 2");
        assert!(d.clauses.iter().any(|c| c.body.contains("lease § 2")));
    }
}
