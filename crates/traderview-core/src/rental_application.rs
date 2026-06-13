//! Rental application — the applicant intake a landlord collects before
//! approving a tenancy. Beyond the applicant/employment fields, it runs the
//! income qualification landlords actually screen on: the income-to-rent
//! multiple and whether gross monthly income meets the required multiple of
//! rent (commonly 3×). Drafting aid, not legal advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct RentalApplicationInput {
    pub applicant_name: String,
    pub premises_address: String,
    pub monthly_rent_usd: f64,
    pub gross_monthly_income_usd: f64,
    /// Required income as a multiple of rent (default 3×).
    #[serde(default = "default_multiple")]
    pub required_income_multiple: f64,
    #[serde(default)]
    pub employer: String,
    #[serde(default)]
    pub position: String,
    #[serde(default)]
    pub current_address: String,
    #[serde(default)]
    pub move_in_date: String,
}

fn default_multiple() -> f64 {
    3.0
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DocClause {
    pub heading: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct RentalApplication {
    pub title: String,
    pub monthly_rent_usd: f64,
    pub gross_monthly_income_usd: f64,
    pub required_income_usd: f64,
    /// Gross monthly income ÷ monthly rent.
    pub income_multiple: f64,
    /// Rent as a percentage of gross monthly income.
    pub rent_to_income_pct: f64,
    /// Income meets the required multiple of rent.
    pub qualifies: bool,
    pub clauses: Vec<DocClause>,
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

pub fn generate(i: &RentalApplicationInput) -> RentalApplication {
    let required_multiple = if i.required_income_multiple > 0.0 {
        i.required_income_multiple
    } else {
        3.0
    };
    let required_income = cents(i.monthly_rent_usd * required_multiple);
    let income_multiple = if i.monthly_rent_usd > 0.0 {
        i.gross_monthly_income_usd / i.monthly_rent_usd
    } else {
        0.0
    };
    let rent_to_income_pct = if i.gross_monthly_income_usd > 0.0 {
        i.monthly_rent_usd / i.gross_monthly_income_usd * 100.0
    } else {
        0.0
    };
    let qualifies = i.gross_monthly_income_usd + 0.005 >= required_income;

    let employment_body = {
        let mut parts = Vec::new();
        if !i.employer.trim().is_empty() {
            parts.push(format!("Employer: {}", i.employer.trim()));
        }
        if !i.position.trim().is_empty() {
            parts.push(format!("Position: {}", i.position.trim()));
        }
        parts.push(format!("Gross monthly income: {}", money(i.gross_monthly_income_usd)));
        parts.join("\n")
    };

    let qualification_body = format!(
        "Monthly rent: {}\nRequired income ({:.2}× rent): {}\nApplicant income-to-rent multiple: {:.2}×\nRent is {:.2}% of gross income\nResult: {}",
        money(i.monthly_rent_usd),
        required_multiple,
        money(required_income),
        income_multiple,
        rent_to_income_pct,
        if qualifies { "MEETS the income requirement" } else { "Does NOT meet the income requirement" }
    );

    let clauses = vec![
        DocClause {
            heading: "1. Applicant".into(),
            body: format!(
                "Applicant: {}\nProperty applied for: {}\nDesired move-in: {}\nCurrent address: {}",
                i.applicant_name,
                i.premises_address,
                if i.move_in_date.trim().is_empty() { "—" } else { i.move_in_date.trim() },
                if i.current_address.trim().is_empty() { "—" } else { i.current_address.trim() }
            ),
        },
        DocClause { heading: "2. Employment and Income".into(), body: employment_body },
        DocClause { heading: "3. Income Qualification".into(), body: qualification_body },
        DocClause {
            heading: "4. Authorization".into(),
            body: "The applicant certifies that the information above is true and authorizes the landlord to verify it and to obtain a credit and background/screening report in connection with this application.".into(),
        },
        DocClause {
            heading: "Signature".into(),
            body: format!("Applicant: ____________________  Date: __________\n{}", i.applicant_name),
        },
    ];

    RentalApplication {
        title: "Rental Application".into(),
        monthly_rent_usd: i.monthly_rent_usd,
        gross_monthly_income_usd: i.gross_monthly_income_usd,
        required_income_usd: required_income,
        income_multiple: cents(income_multiple),
        rent_to_income_pct: cents(rent_to_income_pct),
        qualifies,
        clauses,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    fn base() -> RentalApplicationInput {
        RentalApplicationInput {
            applicant_name: "Riley Renter".into(),
            premises_address: "42 Rental Rd".into(),
            monthly_rent_usd: 2_000.0,
            gross_monthly_income_usd: 6_500.0,
            required_income_multiple: 3.0,
            employer: "Globex".into(),
            position: "Engineer".into(),
            current_address: "1 Old St".into(),
            move_in_date: "2026-08-01".into(),
        }
    }

    #[test]
    fn qualification_math() {
        let d = generate(&base());
        assert!(close(d.required_income_usd, 6_000.0));
        assert!(close(d.income_multiple, 3.25));
        assert!(close(d.rent_to_income_pct, 30.77));
        assert!(d.qualifies);
    }

    #[test]
    fn does_not_qualify_below_multiple() {
        let d = generate(&RentalApplicationInput { gross_monthly_income_usd: 5_000.0, ..base() });
        assert!(!d.qualifies);
        let c = d.clauses.iter().find(|c| c.heading.contains("Income Qualification")).unwrap();
        assert!(c.body.contains("Does NOT meet"));
    }

    #[test]
    fn custom_multiple() {
        // 2.5× rent = 5,000 required; income 5,000 qualifies.
        let d = generate(&RentalApplicationInput {
            required_income_multiple: 2.5,
            gross_monthly_income_usd: 5_000.0,
            ..base()
        });
        assert!(close(d.required_income_usd, 5_000.0));
        assert!(d.qualifies);
    }

    #[test]
    fn zero_multiple_defaults_to_3x() {
        let d = generate(&RentalApplicationInput { required_income_multiple: 0.0, ..base() });
        assert!(close(d.required_income_usd, 6_000.0));
    }

    #[test]
    fn qualification_clause_shows_figures() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading.contains("Income Qualification")).unwrap();
        assert!(c.body.contains("Required income (3.00× rent): $6000.00"));
        assert!(c.body.contains("3.25×"));
        assert!(c.body.contains("MEETS the income requirement"));
    }

    #[test]
    fn authorization_clause_present() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading == "4. Authorization").unwrap();
        assert!(c.body.contains("credit and background"));
    }
}
