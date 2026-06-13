//! Residential lease generator.
//!
//! Assembles a standard residential lease agreement from a set of terms and
//! computes the move-in financials a landlord and tenant both need before
//! signing:
//!
//!   * **Term** — the lease length in months, from the start and end dates.
//!   * **Prorated first month** — when a tenant moves in mid-month, the
//!     first month's rent is charged only for the days they occupy
//!     (rent × days remaining ÷ days in that month).
//!   * **Move-in total** — first month (prorated) + security deposit + pet
//!     deposit + last month's rent if collected upfront.
//!
//! The document itself is returned as an ordered list of titled clauses
//! with the parties, premises, rent, deposit, and policies interpolated —
//! the frontend renders it as a printable lease. This is a drafting aid,
//! not legal advice; deposit caps and required disclosures vary by state.
//! Pure compute.

use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct LeaseInput {
    pub landlord_name: String,
    pub tenant_name: String,
    pub property_address: String,
    pub monthly_rent_usd: f64,
    pub security_deposit_usd: f64,
    /// Lease start date (YYYY-MM-DD).
    pub lease_start: String,
    /// Lease end date (YYYY-MM-DD).
    pub lease_end: String,
    /// Day of the month rent is due (1–28 typical).
    pub rent_due_day: u32,
    /// Flat late fee charged after the grace period.
    pub late_fee_usd: f64,
    /// Days after the due date before a late fee applies.
    pub late_fee_grace_days: u32,
    /// Additional refundable pet deposit (0 ⇒ no pets clause).
    #[serde(default)]
    pub pet_deposit_usd: f64,
    /// Collect the last month's rent at move-in.
    #[serde(default)]
    pub last_month_required: bool,
    /// Governing-law state (e.g. "California").
    pub state: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LeaseClause {
    pub heading: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LeaseDocument {
    pub title: String,
    /// Rounded lease length in months (day-count ÷ average month).
    pub term_months: i64,
    /// First month's rent after any mid-month proration.
    pub prorated_first_month_usd: f64,
    /// True when the tenant moves in after the 1st and the first month is prorated.
    pub first_month_is_prorated: bool,
    /// First month (prorated) + security + pet deposit + optional last month.
    pub move_in_total_usd: f64,
    /// Monthly rent × term months — total rent over the lease.
    pub total_lease_value_usd: f64,
    pub clauses: Vec<LeaseClause>,
}

fn days_in_month(year: i32, month: u32) -> i64 {
    // First day of the next month minus one day = last day of this month.
    let (ny, nm) = if month == 12 { (year + 1, 1) } else { (year, month + 1) };
    let first_next = NaiveDate::from_ymd_opt(ny, nm, 1).unwrap();
    let first_this = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    (first_next - first_this).num_days()
}

const AVG_DAYS_PER_MONTH: f64 = 365.25 / 12.0;

pub fn generate(i: &LeaseInput) -> LeaseDocument {
    let rent = i.monthly_rent_usd.max(0.0);
    let start = NaiveDate::parse_from_str(&i.lease_start, "%Y-%m-%d");
    let end = NaiveDate::parse_from_str(&i.lease_end, "%Y-%m-%d");

    // Term: rounded whole months from the day-count between the dates.
    let term_months = match (start, end) {
        (Ok(s), Ok(e)) if e > s => ((e - s).num_days() as f64 / AVG_DAYS_PER_MONTH).round() as i64,
        _ => 0,
    };

    // Proration: a mid-month move-in pays only for the days occupied.
    let (prorated_first_month, first_month_is_prorated) = match start {
        Ok(s) if s.day() > 1 => {
            let dim = days_in_month(s.year(), s.month());
            let days_occupied = dim - s.day() as i64 + 1;
            ((rent * days_occupied as f64 / dim as f64), true)
        }
        _ => (rent, false),
    };

    let last_month = if i.last_month_required { rent } else { 0.0 };
    let pet = i.pet_deposit_usd.max(0.0);
    let deposit = i.security_deposit_usd.max(0.0);
    let move_in_total = prorated_first_month + deposit + pet + last_month;
    let total_lease_value = rent * term_months as f64;

    let mut clauses = vec![
        LeaseClause {
            heading: "1. Parties".into(),
            body: format!(
                "This Residential Lease Agreement (\"Agreement\") is entered into between {} (\"Landlord\") and {} (\"Tenant\").",
                i.landlord_name, i.tenant_name
            ),
        },
        LeaseClause {
            heading: "2. Premises".into(),
            body: format!(
                "Landlord leases to Tenant the residential premises located at {} (the \"Premises\"), for use as a private residence only.",
                i.property_address
            ),
        },
        LeaseClause {
            heading: "3. Term".into(),
            body: format!(
                "The lease term begins on {} and ends on {}, a term of {} months, unless terminated earlier as provided herein.",
                i.lease_start, i.lease_end, term_months
            ),
        },
        LeaseClause {
            heading: "4. Rent".into(),
            body: format!(
                "Tenant shall pay rent of ${:.2} per month, due on the {} day of each month.{}",
                rent,
                ordinal(i.rent_due_day),
                if first_month_is_prorated {
                    format!(
                        " The first month is prorated to ${:.2} for the partial month of occupancy.",
                        prorated_first_month
                    )
                } else {
                    String::new()
                }
            ),
        },
        LeaseClause {
            heading: "5. Late Charges".into(),
            body: format!(
                "Rent not received within {} day(s) of the due date is subject to a late charge of ${:.2}.",
                i.late_fee_grace_days, i.late_fee_usd
            ),
        },
        LeaseClause {
            heading: "6. Security Deposit".into(),
            body: format!(
                "Tenant shall deposit ${:.2} as a security deposit, to be returned within the period required by {} law less lawful deductions for damages beyond normal wear and tear.",
                deposit, i.state
            ),
        },
    ];

    if pet > 0.0 {
        clauses.push(LeaseClause {
            heading: "7. Pets".into(),
            body: format!(
                "Pets are permitted subject to an additional refundable pet deposit of ${:.2}. Tenant is responsible for any pet-related damage.",
                pet
            ),
        });
    }

    if i.last_month_required {
        clauses.push(LeaseClause {
            heading: "8. Last Month's Rent".into(),
            body: format!(
                "Tenant shall pay the last month's rent of ${:.2} in advance at move-in, applied to the final month of the term.",
                rent
            ),
        });
    }

    clauses.push(LeaseClause {
        heading: "9. Move-In Total Due".into(),
        body: format!(
            "The total amount due at move-in is ${:.2} (first month ${:.2} + security deposit ${:.2}{}{}).",
            move_in_total,
            prorated_first_month,
            deposit,
            if pet > 0.0 { format!(" + pet deposit ${pet:.2}") } else { String::new() },
            if i.last_month_required { format!(" + last month ${rent:.2}") } else { String::new() },
        ),
    });

    clauses.push(LeaseClause {
        heading: "10. Maintenance & Entry".into(),
        body: format!(
            "Tenant shall keep the Premises clean and report needed repairs promptly. Landlord may enter for repairs or inspection upon reasonable advance notice as required by {} law, except in emergencies.",
            i.state
        ),
    });

    clauses.push(LeaseClause {
        heading: "11. Governing Law".into(),
        body: format!(
            "This Agreement is governed by the laws of the State of {}. If any provision is held unenforceable, the remainder stays in effect.",
            i.state
        ),
    });

    clauses.push(LeaseClause {
        heading: "12. Signatures".into(),
        body: format!(
            "Landlord: {} ____________________  Date: __________\nTenant: {} ____________________  Date: __________",
            i.landlord_name, i.tenant_name
        ),
    });

    LeaseDocument {
        title: "Residential Lease Agreement".into(),
        term_months,
        prorated_first_month_usd: prorated_first_month,
        first_month_is_prorated,
        move_in_total_usd: move_in_total,
        total_lease_value_usd: total_lease_value,
        clauses,
    }
}

fn ordinal(n: u32) -> String {
    let suffix = match (n % 10, n % 100) {
        (1, 11) | (2, 12) | (3, 13) => "th",
        (1, _) => "st",
        (2, _) => "nd",
        (3, _) => "rd",
        _ => "th",
    };
    format!("{n}{suffix}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> LeaseInput {
        LeaseInput {
            landlord_name: "Jane Owner".into(),
            tenant_name: "John Renter".into(),
            property_address: "123 Main St, Apt 4".into(),
            monthly_rent_usd: 2_000.0,
            security_deposit_usd: 2_000.0,
            lease_start: "2026-01-01".into(),
            lease_end: "2026-12-31".into(),
            rent_due_day: 1,
            late_fee_usd: 75.0,
            late_fee_grace_days: 5,
            pet_deposit_usd: 0.0,
            last_month_required: false,
            state: "California".into(),
        }
    }

    #[test]
    fn twelve_month_lease_term() {
        let d = generate(&base());
        assert_eq!(d.term_months, 12);
        assert!((d.total_lease_value_usd - 24_000.0).abs() < 1e-6);
    }

    #[test]
    fn first_of_month_start_is_not_prorated() {
        let d = generate(&base());
        assert!(!d.first_month_is_prorated);
        assert!((d.prorated_first_month_usd - 2_000.0).abs() < 1e-6);
        // Move-in = first month + deposit = 4,000.
        assert!((d.move_in_total_usd - 4_000.0).abs() < 1e-6);
    }

    #[test]
    fn mid_month_move_in_is_prorated() {
        // Start Jan 16 → 16 days occupied of 31 (Jan 16..Jan 31 inclusive).
        let d = generate(&LeaseInput { lease_start: "2026-01-16".into(), ..base() });
        assert!(d.first_month_is_prorated);
        let expected = 2_000.0 * 16.0 / 31.0;
        assert!((d.prorated_first_month_usd - expected).abs() < 1e-6);
    }

    #[test]
    fn move_in_total_includes_pet_and_last_month() {
        let d = generate(&LeaseInput {
            pet_deposit_usd: 500.0,
            last_month_required: true,
            ..base()
        });
        // 2000 first + 2000 deposit + 500 pet + 2000 last = 6,500.
        assert!((d.move_in_total_usd - 6_500.0).abs() < 1e-6);
        assert!(d.clauses.iter().any(|c| c.heading.contains("Pets")));
        assert!(d.clauses.iter().any(|c| c.heading.contains("Last Month")));
    }

    #[test]
    fn no_pet_clause_when_no_pet_deposit() {
        let d = generate(&base());
        assert!(!d.clauses.iter().any(|c| c.heading.contains("Pets")));
    }

    #[test]
    fn clauses_interpolate_parties_and_terms() {
        let d = generate(&base());
        let parties = &d.clauses[0].body;
        assert!(parties.contains("Jane Owner") && parties.contains("John Renter"));
        let rent = d.clauses.iter().find(|c| c.heading.starts_with("4.")).unwrap();
        assert!(rent.body.contains("$2000.00") && rent.body.contains("1st day"));
        let law = d.clauses.iter().find(|c| c.heading.contains("Governing")).unwrap();
        assert!(law.body.contains("California"));
    }

    #[test]
    fn six_month_lease_rounds_correctly() {
        let d = generate(&LeaseInput {
            lease_start: "2026-01-01".into(),
            lease_end: "2026-06-30".into(),
            ..base()
        });
        assert_eq!(d.term_months, 6);
    }
}
