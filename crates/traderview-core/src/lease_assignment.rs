//! Assignment of lease — the original tenant (assignor) transfers the ENTIRE
//! remaining interest in a lease to a new tenant (assignee), who steps into the
//! assignor's shoes for the rest of the term. Distinct from a sublease (which
//! keeps the assignor's interest in place). It computes the whole months
//! remaining and the rent obligation transferred, and handles whether the
//! assignor is released or remains secondarily liable. Drafting aid, not legal
//! advice.

use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct LeaseAssignmentInput {
    pub assignor_name: String,
    pub assignee_name: String,
    pub landlord_name: String,
    pub property_address: String,
    pub assignment_effective_date: String,
    pub original_lease_end_date: String,
    pub monthly_rent_usd: f64,
    /// Whether the landlord releases the assignor from further liability.
    #[serde(default)]
    pub assignor_released: bool,
    #[serde(default)]
    pub security_deposit_transfer_usd: f64,
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
pub struct LeaseAssignment {
    pub title: String,
    pub months_remaining: i64,
    pub monthly_rent_usd: f64,
    pub remaining_rent_obligation_usd: f64,
    pub assignor_released: bool,
    pub statutory_citation: String,
    pub clauses: Vec<DocClause>,
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

/// Whole calendar months from `start` up to `end` (0 if end precedes start).
fn months_between(start: NaiveDate, end: NaiveDate) -> i64 {
    if end < start {
        return 0;
    }
    let mut months = (end.year() - start.year()) * 12 + (end.month() as i32 - start.month() as i32);
    if end.day() < start.day() {
        months -= 1;
    }
    months.max(0) as i64
}

pub fn generate(i: &LeaseAssignmentInput) -> LeaseAssignment {
    let months = match (
        NaiveDate::parse_from_str(&i.assignment_effective_date, "%Y-%m-%d"),
        NaiveDate::parse_from_str(&i.original_lease_end_date, "%Y-%m-%d"),
    ) {
        (Ok(s), Ok(e)) => months_between(s, e),
        _ => 0,
    };
    let remaining_obligation = cents(i.monthly_rent_usd * months as f64);

    let citation = i.statute_citation.trim();
    let pursuant = if citation.is_empty() {
        format!("This assignment is governed by the landlord-tenant law of the State of {}.", i.state)
    } else {
        format!("This assignment is governed by the landlord-tenant law of the State of {} ({}).", i.state, citation)
    };

    let liability_body = if i.assignor_released {
        "The Landlord releases the Assignor from further liability under the lease as of the effective date; the Assignee is solely responsible going forward.".to_string()
    } else {
        "The Assignor is NOT released and remains secondarily liable for the performance of the lease if the Assignee defaults, unless the Landlord agrees otherwise in writing.".to_string()
    };

    let deposit_body = if i.security_deposit_transfer_usd > 0.0 {
        format!(
            "The security deposit of {} is transferred with the lease; the Assignee assumes the right to its return at the end of the term.",
            money(i.security_deposit_transfer_usd)
        )
    } else {
        "Any security deposit arrangements are addressed separately between the parties.".to_string()
    };

    let clauses = vec![
        DocClause {
            heading: "Parties".into(),
            body: format!(
                "Assignor (original tenant): {}\nAssignee (new tenant): {}\nLandlord: {}\nPremises: {}",
                i.assignor_name, i.assignee_name, i.landlord_name, i.property_address
            ),
        },
        DocClause {
            heading: "1. Assignment".into(),
            body: format!(
                "Effective {}, the Assignor assigns and transfers to the Assignee all of the Assignor's right, title, and interest in the lease for the premises above, for the remainder of the lease term ending {}.",
                i.assignment_effective_date, i.original_lease_end_date
            ),
        },
        DocClause {
            heading: "2. Remaining Term".into(),
            body: format!(
                "Approximately {} whole months remain on the lease. At the current rent of {} per month, that is about {} in remaining rent the Assignee assumes.",
                months, money(i.monthly_rent_usd), money(remaining_obligation)
            ),
        },
        DocClause {
            heading: "3. Assumption".into(),
            body: "The Assignee accepts the assignment and assumes and agrees to perform all of the Assignor's obligations under the lease arising from and after the effective date, including payment of rent.".into(),
        },
        DocClause { heading: "4. Assignor Liability".into(), body: liability_body },
        DocClause { heading: "5. Security Deposit".into(), body: deposit_body },
        DocClause {
            heading: "6. Landlord Consent".into(),
            body: "This assignment is effective only with the Landlord's written consent where the lease or law requires it. By signing below, the Landlord consents to the assignment.".into(),
        },
        DocClause { heading: "7. Governing Law".into(), body: pursuant },
        DocClause {
            heading: "Signatures".into(),
            body: format!(
                "Assignor: ____________________  Date: __________\n{}\n\nAssignee: ____________________  Date: __________\n{}\n\nLandlord (consent): ____________________  Date: __________\n{}",
                i.assignor_name, i.assignee_name, i.landlord_name
            ),
        },
    ];

    LeaseAssignment {
        title: "Assignment of Lease".into(),
        months_remaining: months,
        monthly_rent_usd: i.monthly_rent_usd,
        remaining_rent_obligation_usd: remaining_obligation,
        assignor_released: i.assignor_released,
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

    fn base() -> LeaseAssignmentInput {
        LeaseAssignmentInput {
            assignor_name: "Ann Assignor".into(),
            assignee_name: "Ned Assignee".into(),
            landlord_name: "Acme Property Mgmt".into(),
            property_address: "42 Rental Rd".into(),
            assignment_effective_date: "2026-09-01".into(),
            original_lease_end_date: "2027-09-01".into(),
            monthly_rent_usd: 1_800.0,
            assignor_released: false,
            security_deposit_transfer_usd: 1_800.0,
            state: "Michigan".into(),
            statute_citation: String::new(),
        }
    }

    #[test]
    fn months_and_obligation() {
        let d = generate(&base());
        assert_eq!(d.months_remaining, 12);
        assert!(close(d.remaining_rent_obligation_usd, 21_600.0));
    }

    #[test]
    fn partial_month_floors() {
        // Sep 1 → Aug 31 next year is 11 whole months, not 12.
        let d = generate(&LeaseAssignmentInput { original_lease_end_date: "2027-08-31".into(), ..base() });
        assert_eq!(d.months_remaining, 11);
    }

    #[test]
    fn assignor_not_released_by_default() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading.contains("Assignor Liability")).unwrap();
        assert!(c.body.contains("NOT released"));
        assert!(c.body.contains("secondarily liable"));
    }

    #[test]
    fn assignor_released_when_set() {
        let c = generate(&LeaseAssignmentInput { assignor_released: true, ..base() })
            .clauses.into_iter().find(|c| c.heading.contains("Assignor Liability")).unwrap();
        assert!(c.body.contains("releases the Assignor"));
    }

    #[test]
    fn deposit_transfer_clause() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading.contains("Security Deposit")).unwrap();
        assert!(c.body.contains("$1800.00 is transferred"));
    }

    #[test]
    fn bad_dates_yield_zero_months() {
        let d = generate(&LeaseAssignmentInput { assignment_effective_date: "x".into(), ..base() });
        assert_eq!(d.months_remaining, 0);
        assert!(close(d.remaining_rent_obligation_usd, 0.0));
    }

    #[test]
    fn statute_citation_echoed() {
        let d = generate(&LeaseAssignmentInput { statute_citation: "MCL 554".into(), ..base() });
        assert_eq!(d.statutory_citation, "MCL 554");
        assert!(d.clauses.iter().any(|c| c.body.contains("MCL 554")));
    }
}
