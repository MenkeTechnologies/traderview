//! Rent increase notice — the written notice a landlord must serve before
//! raising rent on a periodic tenancy. It computes the new rent (by percent or
//! flat amount), the dollar and percent change, and the effective date from the
//! service date plus the required notice period, then assembles the notice.
//! Drafting aid, not legal advice — many states set the minimum notice (and
//! sometimes a cap) by the size of the increase.

use chrono::{Duration, NaiveDate};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IncreaseType {
    Percent,
    Amount,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RentIncreaseInput {
    pub landlord_name: String,
    pub landlord_address: String,
    #[serde(default)]
    pub landlord_phone: String,
    pub tenant_name: String,
    pub premises_address: String,
    pub current_rent_usd: f64,
    pub increase_type: IncreaseType,
    /// Percent (when Percent) or dollars (when Amount).
    pub increase_value: f64,
    /// Date the notice is served (YYYY-MM-DD).
    pub served_date: String,
    /// Days of notice before the increase takes effect (per state minimum).
    pub notice_days: i64,
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
pub struct RentIncreaseNotice {
    pub title: String,
    pub current_rent_usd: f64,
    pub new_rent_usd: f64,
    pub increase_amount_usd: f64,
    pub increase_pct: f64,
    pub effective_date: String,
    pub statutory_citation: String,
    pub clauses: Vec<DocClause>,
}

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

pub fn generate(i: &RentIncreaseInput) -> RentIncreaseNotice {
    let new_rent = match i.increase_type {
        IncreaseType::Percent => i.current_rent_usd * (1.0 + i.increase_value / 100.0),
        IncreaseType::Amount => i.current_rent_usd + i.increase_value,
    };
    let increase_amount = new_rent - i.current_rent_usd;
    let increase_pct = if i.current_rent_usd != 0.0 {
        increase_amount / i.current_rent_usd * 100.0
    } else {
        0.0
    };

    let effective_date = NaiveDate::parse_from_str(&i.served_date, "%Y-%m-%d")
        .map(|d| (d + Duration::days(i.notice_days)).format("%Y-%m-%d").to_string())
        .unwrap_or_default();

    let citation = i.statute_citation.trim();
    let pursuant = if citation.is_empty() {
        format!(
            "This notice is given pursuant to the landlord-tenant law of the State of {}.",
            i.state
        )
    } else {
        format!(
            "This notice is given pursuant to the landlord-tenant law of the State of {} ({}).",
            i.state, citation
        )
    };

    let clauses = vec![
        DocClause { heading: "To".into(), body: i.tenant_name.clone() },
        DocClause {
            heading: "1. Current Rent".into(),
            body: format!(
                "The current monthly rent for the premises located at {} is {}.",
                i.premises_address,
                money(i.current_rent_usd)
            ),
        },
        DocClause {
            heading: "2. New Rent and Effective Date".into(),
            body: format!(
                "Effective {}, the monthly rent will increase by {} ({:.2}%) to {}. All other terms of your tenancy remain unchanged. If you do not wish to continue the tenancy at the new rent, you must vacate on or before the effective date in accordance with your rental agreement and applicable law.",
                effective_date,
                money(increase_amount),
                increase_pct,
                money(new_rent)
            ),
        },
        DocClause {
            heading: "3. Notice Period".into(),
            body: format!(
                "This notice is served {} and provides {} days' notice before the increase takes effect.",
                i.served_date, i.notice_days
            ),
        },
        DocClause { heading: "4. Governing Law".into(), body: pursuant },
        DocClause {
            heading: "Signature".into(),
            body: format!(
                "Signature of owner of premises or agent: ____________________  Date: {}\n{}\n{}{}",
                i.served_date,
                i.landlord_name,
                i.landlord_address,
                if i.landlord_phone.is_empty() {
                    String::new()
                } else {
                    format!("\nTelephone: {}", i.landlord_phone)
                }
            ),
        },
    ];

    RentIncreaseNotice {
        title: "Notice of Rent Increase".into(),
        current_rent_usd: i.current_rent_usd,
        new_rent_usd: new_rent,
        increase_amount_usd: increase_amount,
        increase_pct,
        effective_date,
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

    fn base() -> RentIncreaseInput {
        RentIncreaseInput {
            landlord_name: "Acme Property Mgmt".into(),
            landlord_address: "1 Main St".into(),
            landlord_phone: String::new(),
            tenant_name: "Jane Doe".into(),
            premises_address: "42 Rental Rd".into(),
            current_rent_usd: 1500.0,
            increase_type: IncreaseType::Percent,
            increase_value: 5.0,
            served_date: "2026-06-01".into(),
            notice_days: 60,
            state: "California".into(),
            statute_citation: String::new(),
        }
    }

    #[test]
    fn percent_increase() {
        let n = generate(&base());
        assert!(close(n.new_rent_usd, 1575.0));
        assert!(close(n.increase_amount_usd, 75.0));
        assert!(close(n.increase_pct, 5.0));
    }

    #[test]
    fn amount_increase_back_computes_percent() {
        let n = generate(&RentIncreaseInput {
            increase_type: IncreaseType::Amount,
            increase_value: 100.0,
            ..base()
        });
        assert!(close(n.new_rent_usd, 1600.0));
        assert!(close(n.increase_amount_usd, 100.0));
        assert!((n.increase_pct - 6.6667).abs() < 1e-3);
    }

    #[test]
    fn effective_date_is_served_plus_notice() {
        // 2026-06-01 + 60 days = 2026-07-31.
        assert_eq!(generate(&base()).effective_date, "2026-07-31");
    }

    #[test]
    fn zero_current_rent_no_divide_panic() {
        let n = generate(&RentIncreaseInput { current_rent_usd: 0.0, ..base() });
        assert!(close(n.increase_pct, 0.0));
    }

    #[test]
    fn statute_citation_echoed() {
        let n = generate(&RentIncreaseInput {
            statute_citation: "Cal. Civ. Code § 827".into(),
            ..base()
        });
        assert_eq!(n.statutory_citation, "Cal. Civ. Code § 827");
        assert!(n.clauses.iter().any(|c| c.body.contains("Cal. Civ. Code § 827")));
    }

    #[test]
    fn new_rent_appears_in_clause() {
        let n = generate(&base());
        let c = n.clauses.iter().find(|c| c.heading.contains("New Rent")).unwrap();
        assert!(c.body.contains("$1575.00"));
    }

    #[test]
    fn bad_date_yields_empty_effective() {
        let n = generate(&RentIncreaseInput { served_date: "x".into(), ..base() });
        assert_eq!(n.effective_date, "");
    }
}
