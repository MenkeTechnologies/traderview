//! CAM reconciliation — the annual common-area-maintenance true-up under a
//! commercial lease. The tenant pays monthly CAM estimates during the year; at
//! year end the landlord reconciles each tenant's pro-rata share (by rentable
//! square footage) of the actual CAM spend against what the tenant paid, and
//! bills or credits the difference. An optional cap limits the tenant's share to
//! a percentage increase over the prior year (a controllable-expense cap). No
//! existing generator computes pro-rata-by-square-foot expense reconciliation.
//! Drafting aid, not legal/accounting advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct CamInput {
    pub landlord_name: String,
    pub tenant_name: String,
    #[serde(default)]
    pub property_label: String,
    /// Tenant's rentable square footage.
    pub tenant_sqft: f64,
    /// Building's total rentable square footage.
    pub building_sqft: f64,
    /// Actual total CAM spend for the year.
    pub actual_cam_usd: f64,
    /// Monthly CAM estimate the tenant paid.
    pub monthly_estimate_usd: f64,
    /// Months covered by the reconciliation (usually 12).
    pub months: u32,
    /// Cap on the increase over the prior-year share, percent (0 = no cap).
    #[serde(default)]
    pub cap_pct: f64,
    /// Tenant's prior-year reconciled share, for the cap (0 = no cap basis).
    #[serde(default)]
    pub prior_year_share_usd: f64,
    pub year: String,
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
pub struct CamReconciliation {
    pub title: String,
    /// Tenant's pro-rata share of the building, percent.
    pub pro_rata_pct: f64,
    /// Pro-rata share × actual CAM, before any cap.
    pub tenant_share_uncapped_usd: f64,
    /// Tenant's share after applying the cap.
    pub tenant_share_usd: f64,
    /// True when the cap reduced the tenant's share.
    pub cap_applied: bool,
    /// Estimates the tenant paid during the period.
    pub estimates_paid_usd: f64,
    /// Positive = tenant owes; negative = credit/refund to tenant.
    pub balance_usd: f64,
    pub statutory_citation: String,
    pub clauses: Vec<DocClause>,
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

pub fn generate(i: &CamInput) -> CamReconciliation {
    let share = if i.building_sqft > 0.0 {
        i.tenant_sqft / i.building_sqft
    } else {
        0.0
    };
    let uncapped = cents(i.actual_cam_usd * share);

    // Controllable-expense cap: tenant share limited to prior-year share grown
    // by at most cap_pct.
    let (tenant_share, cap_applied) = if i.cap_pct > 0.0 && i.prior_year_share_usd > 0.0 {
        let max_share = cents(i.prior_year_share_usd * (1.0 + i.cap_pct / 100.0));
        if uncapped > max_share {
            (max_share, true)
        } else {
            (uncapped, false)
        }
    } else {
        (uncapped, false)
    };

    let paid = cents(i.monthly_estimate_usd * i.months as f64);
    let balance = cents(tenant_share - paid);

    let property = if i.property_label.trim().is_empty() {
        "the Premises".to_string()
    } else {
        i.property_label.trim().to_string()
    };

    let owes = balance >= 0.0;
    let settle = if owes {
        format!("the Tenant owes the Landlord {}", money(balance.abs()))
    } else {
        format!("the Landlord shall credit the Tenant {}", money(balance.abs()))
    };

    let cap_line = if cap_applied {
        format!(
            " The uncapped share of {} was reduced to {} by the {:.1}% cap over the prior-year share of {}.",
            money(uncapped),
            money(tenant_share),
            i.cap_pct,
            money(i.prior_year_share_usd)
        )
    } else {
        String::new()
    };

    let calc_body = format!(
        "The Tenant's pro-rata share is {:.0} ÷ {:.0} rentable square feet = {:.2}%. Applied to actual CAM of {} for {}, the Tenant's share is {}.{} The Tenant paid {} in monthly estimates ({} × {} months), so {}.",
        i.tenant_sqft,
        i.building_sqft,
        cents(share * 100.0),
        money(i.actual_cam_usd),
        i.year,
        money(tenant_share),
        cap_line,
        money(paid),
        money(i.monthly_estimate_usd),
        i.months,
        settle
    );

    let citation = i.statute_citation.trim();
    let governing = if citation.is_empty() {
        format!("This reconciliation is governed by the lease and the laws of the State of {}.", i.state)
    } else {
        format!("This reconciliation is governed by the lease and the laws of the State of {} ({}).", i.state, citation)
    };

    let clauses = vec![
        DocClause {
            heading: "Parties".into(),
            body: format!(
                "Landlord: {}\nTenant: {}\nPremises: {}\nReconciliation year: {}\nStatement date: {}",
                i.landlord_name, i.tenant_name, property, i.year, i.date
            ),
        },
        DocClause {
            heading: "1. Pro-Rata Share".into(),
            body: format!(
                "The Tenant's pro-rata share of common-area maintenance is {:.2}%, being the Tenant's {:.0} rentable square feet divided by the building's {:.0} rentable square feet.",
                cents(share * 100.0),
                i.tenant_sqft,
                i.building_sqft
            ),
        },
        DocClause { heading: "2. Reconciliation".into(), body: calc_body },
        DocClause {
            heading: "3. Payment".into(),
            body: if owes {
                format!("The Tenant shall pay the reconciled balance of {} within 30 days of this statement.", money(balance.abs()))
            } else {
                format!("The Landlord shall apply the overpayment credit of {} to the next CAM estimate or refund it within 30 days.", money(balance.abs()))
            },
        },
        DocClause {
            heading: "4. Audit Right".into(),
            body: "The Tenant may, within 90 days of this statement and on reasonable notice, audit the Landlord's CAM records for the reconciliation year. Overcharges revealed by the audit shall be refunded.".into(),
        },
        DocClause { heading: "5. Governing Law".into(), body: governing },
        DocClause {
            heading: "Signatures".into(),
            body: format!(
                "Landlord: ____________________  Date: __________\n{}\n\nTenant: ____________________  Date: __________\n{}",
                i.landlord_name, i.tenant_name
            ),
        },
    ];

    CamReconciliation {
        title: "CAM Reconciliation Statement".into(),
        pro_rata_pct: cents(share * 100.0),
        tenant_share_uncapped_usd: uncapped,
        tenant_share_usd: cents(tenant_share),
        cap_applied,
        estimates_paid_usd: paid,
        balance_usd: balance,
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

    fn base() -> CamInput {
        CamInput {
            landlord_name: "Plaza Owners LP".into(),
            tenant_name: "Cafe Tenant LLC".into(),
            property_label: "Suite 100".into(),
            tenant_sqft: 5_000.0,
            building_sqft: 50_000.0,
            actual_cam_usd: 400_000.0,
            monthly_estimate_usd: 3_000.0,
            months: 12,
            cap_pct: 0.0,
            prior_year_share_usd: 0.0,
            year: "2025".into(),
            date: "2026-07-01".into(),
            state: "Delaware".into(),
            statute_citation: String::new(),
        }
    }

    #[test]
    fn tenant_owes_underpayment() {
        let d = generate(&base());
        assert!(close(d.pro_rata_pct, 10.0));
        assert!(close(d.tenant_share_usd, 40_000.0));
        assert!(close(d.estimates_paid_usd, 36_000.0));
        assert!(close(d.balance_usd, 4_000.0));
        assert!(!d.cap_applied);
    }

    #[test]
    fn overpayment_is_credit() {
        let d = generate(&CamInput { monthly_estimate_usd: 3_500.0, ..base() });
        // paid 42,000 vs share 40,000 → -2,000 credit.
        assert!(close(d.balance_usd, -2_000.0));
        assert!(d.clauses.iter().any(|c| c.body.contains("credit")));
    }

    #[test]
    fn cap_limits_share_over_prior_year() {
        let d = generate(&CamInput { cap_pct: 5.0, prior_year_share_usd: 35_000.0, ..base() });
        // max share = 35,000 × 1.05 = 36,750 < 40,000 uncapped.
        assert!(close(d.tenant_share_uncapped_usd, 40_000.0));
        assert!(close(d.tenant_share_usd, 36_750.0));
        assert!(d.cap_applied);
        assert!(close(d.balance_usd, 750.0));
    }

    #[test]
    fn cap_not_applied_when_share_below_max() {
        let d = generate(&CamInput { cap_pct: 25.0, prior_year_share_usd: 35_000.0, ..base() });
        // max = 43,750 > 40,000 → no cap.
        assert!(!d.cap_applied);
        assert!(close(d.tenant_share_usd, 40_000.0));
    }

    #[test]
    fn zero_building_sqft_no_divide_by_zero() {
        let d = generate(&CamInput { building_sqft: 0.0, ..base() });
        assert!(close(d.pro_rata_pct, 0.0));
        assert!(close(d.tenant_share_usd, 0.0));
    }

    #[test]
    fn audit_right_clause_present() {
        assert!(generate(&base()).clauses.iter().any(|c| c.heading.contains("Audit")));
    }

    #[test]
    fn statute_citation_echoed() {
        let d = generate(&CamInput { statute_citation: "Cal. Civ. Code § 1950.8".into(), ..base() });
        assert_eq!(d.statutory_citation, "Cal. Civ. Code § 1950.8");
        assert!(d.clauses.iter().any(|c| c.body.contains("Cal. Civ. Code § 1950.8")));
    }
}
