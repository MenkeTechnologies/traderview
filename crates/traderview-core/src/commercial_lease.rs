//! Commercial lease (triple-net / NNN) — leases business space where the tenant
//! pays base rent plus its share of the operating costs (common-area
//! maintenance, property tax, insurance). It computes the base and NNN charges
//! from the per-square-foot rates and the area, the gross monthly rent, and the
//! lease end date, then assembles the lease. Distinct from the residential
//! `lease` generator. Drafting aid, not legal advice.

use chrono::{Duration, Months, NaiveDate};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct CommercialLeaseInput {
    pub landlord_name: String,
    pub tenant_name: String,
    pub premises_address: String,
    pub square_feet: f64,
    /// Base rent, dollars per square foot per year.
    pub base_rent_psf_annual: f64,
    #[serde(default)]
    pub cam_psf_annual: f64,
    #[serde(default)]
    pub property_tax_psf_annual: f64,
    #[serde(default)]
    pub insurance_psf_annual: f64,
    pub lease_start_date: String,
    pub term_months: u32,
    /// Permitted use of the premises.
    #[serde(default)]
    pub permitted_use: String,
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
pub struct CommercialLease {
    pub title: String,
    pub lease_start_date: String,
    pub lease_end_date: String,
    pub term_months: u32,
    pub base_annual_usd: f64,
    pub base_monthly_usd: f64,
    pub nnn_psf_annual: f64,
    pub nnn_annual_usd: f64,
    pub nnn_monthly_usd: f64,
    pub gross_annual_usd: f64,
    pub gross_monthly_usd: f64,
    pub statutory_citation: String,
    pub clauses: Vec<DocClause>,
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

pub fn generate(i: &CommercialLeaseInput) -> CommercialLease {
    let base_annual = cents(i.square_feet * i.base_rent_psf_annual);
    let nnn_psf = i.cam_psf_annual + i.property_tax_psf_annual + i.insurance_psf_annual;
    let nnn_annual = cents(i.square_feet * nnn_psf);
    let gross_annual = cents(base_annual + nnn_annual);

    let base_monthly = cents(base_annual / 12.0);
    let nnn_monthly = cents(nnn_annual / 12.0);
    let gross_monthly = cents(gross_annual / 12.0);

    let end_date = NaiveDate::parse_from_str(&i.lease_start_date, "%Y-%m-%d")
        .ok()
        .and_then(|d| d.checked_add_months(Months::new(i.term_months)))
        .map(|d| (d - Duration::days(1)).format("%Y-%m-%d").to_string())
        .unwrap_or_default();

    let citation = i.statute_citation.trim();
    let pursuant = if citation.is_empty() {
        format!("This lease is governed by the laws of the State of {}.", i.state)
    } else {
        format!("This lease is governed by the laws of the State of {} ({}).", i.state, citation)
    };

    let nnn_body = if nnn_psf > 0.0 {
        format!(
            "In addition to base rent, the Tenant shall pay its triple-net share of operating costs at {} per square foot per year (CAM {}, property tax {}, insurance {}), totaling {} per year ({} per month).",
            money(nnn_psf),
            money(i.cam_psf_annual),
            money(i.property_tax_psf_annual),
            money(i.insurance_psf_annual),
            money(nnn_annual),
            money(nnn_monthly)
        )
    } else {
        "No separate triple-net charges apply; rent is gross.".to_string()
    };

    let use_body = if i.permitted_use.trim().is_empty() {
        "The premises shall be used only for lawful commercial purposes.".to_string()
    } else {
        format!("The premises shall be used only for: {}.", i.permitted_use.trim())
    };

    let clauses = vec![
        DocClause {
            heading: "Parties".into(),
            body: format!("Landlord: {}\nTenant: {}\nPremises: {} ({} sq ft)", i.landlord_name, i.tenant_name, i.premises_address, i.square_feet),
        },
        DocClause {
            heading: "1. Term".into(),
            body: format!("The lease term is {} months, beginning {} and ending {}.", i.term_months, i.lease_start_date, end_date),
        },
        DocClause {
            heading: "2. Base Rent".into(),
            body: format!(
                "Base rent is {} per square foot per year on {} square feet, or {} per year ({} per month).",
                money(i.base_rent_psf_annual), i.square_feet, money(base_annual), money(base_monthly)
            ),
        },
        DocClause { heading: "3. Triple-Net Charges".into(), body: nnn_body },
        DocClause {
            heading: "4. Total Rent".into(),
            body: format!(
                "Total rent is {} per year, payable in monthly installments of {}.",
                money(gross_annual), money(gross_monthly)
            ),
        },
        DocClause { heading: "5. Use".into(), body: use_body },
        DocClause { heading: "6. Governing Law".into(), body: pursuant },
        DocClause {
            heading: "Signatures".into(),
            body: format!(
                "Landlord: ____________________  Date: __________\n{}\n\nTenant: ____________________  Date: __________\n{}",
                i.landlord_name, i.tenant_name
            ),
        },
    ];

    CommercialLease {
        title: "Commercial Lease Agreement (NNN)".into(),
        lease_start_date: i.lease_start_date.clone(),
        lease_end_date: end_date,
        term_months: i.term_months,
        base_annual_usd: base_annual,
        base_monthly_usd: base_monthly,
        nnn_psf_annual: nnn_psf,
        nnn_annual_usd: nnn_annual,
        nnn_monthly_usd: nnn_monthly,
        gross_annual_usd: gross_annual,
        gross_monthly_usd: gross_monthly,
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

    fn base() -> CommercialLeaseInput {
        CommercialLeaseInput {
            landlord_name: "Prop Holdings".into(),
            tenant_name: "Beta Corp".into(),
            premises_address: "100 Commerce Plaza, Suite 5".into(),
            square_feet: 2000.0,
            base_rent_psf_annual: 30.0,
            cam_psf_annual: 5.0,
            property_tax_psf_annual: 3.0,
            insurance_psf_annual: 2.0,
            lease_start_date: "2026-08-01".into(),
            term_months: 60,
            permitted_use: "Retail store".into(),
            state: "Texas".into(),
            statute_citation: String::new(),
        }
    }

    #[test]
    fn base_rent_annual_and_monthly() {
        let d = generate(&base());
        assert!(close(d.base_annual_usd, 60_000.0));
        assert!(close(d.base_monthly_usd, 5_000.0));
    }

    #[test]
    fn nnn_charges() {
        let d = generate(&base());
        assert!(close(d.nnn_psf_annual, 10.0));
        assert!(close(d.nnn_annual_usd, 20_000.0));
        assert!(close(d.nnn_monthly_usd, 1_666.67));
    }

    #[test]
    fn gross_rent() {
        let d = generate(&base());
        assert!(close(d.gross_annual_usd, 80_000.0));
        assert!(close(d.gross_monthly_usd, 6_666.67));
    }

    #[test]
    fn end_date_is_start_plus_term_minus_day() {
        // 2026-08-01 + 60 months − 1 day = 2031-07-31.
        assert_eq!(generate(&base()).lease_end_date, "2031-07-31");
    }

    #[test]
    fn no_nnn_is_gross_lease() {
        let d = generate(&CommercialLeaseInput {
            cam_psf_annual: 0.0,
            property_tax_psf_annual: 0.0,
            insurance_psf_annual: 0.0,
            ..base()
        });
        assert!(close(d.nnn_annual_usd, 0.0));
        assert!(close(d.gross_annual_usd, 60_000.0));
        let c = d.clauses.iter().find(|c| c.heading.contains("Triple-Net")).unwrap();
        assert!(c.body.contains("rent is gross"));
    }

    #[test]
    fn statute_citation_echoed() {
        let d = generate(&CommercialLeaseInput { statute_citation: "Tex. Prop. Code".into(), ..base() });
        assert_eq!(d.statutory_citation, "Tex. Prop. Code");
        assert!(d.clauses.iter().any(|c| c.body.contains("Tex. Prop. Code")));
    }
}
