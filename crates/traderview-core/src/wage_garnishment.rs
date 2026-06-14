//! Wage garnishment calculation — the maximum amount a creditor may garnish from
//! a pay period under the federal Consumer Credit Protection Act (Title III). The
//! garnishable amount is the lesser of (a) a percentage of disposable earnings
//! (25% for ordinary debt; higher for support) and (b) the amount by which
//! disposable earnings exceed 30× the federal minimum wage. This computes both
//! caps, the protected floor, and the resulting garnishment. Distinct from
//! take-home-paycheck, which computes net pay, not garnishment limits. Drafting
//! aid, not legal advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct GarnishmentInput {
    pub creditor_name: String,
    pub employee_name: String,
    pub employer_name: String,
    /// Disposable earnings for the pay period (after legally required deductions).
    pub disposable_earnings_usd: f64,
    /// Applicable federal (or higher state) minimum wage, per hour.
    #[serde(default = "default_min_wage")]
    pub min_wage_usd: f64,
    /// Percentage cap on disposable earnings (25% ordinary; 50–65% support).
    #[serde(default = "default_pct")]
    pub cap_pct: f64,
    /// Minimum-wage multiplier for the protected floor (30× weekly under CCPA).
    #[serde(default = "default_multiplier")]
    pub min_wage_multiplier: f64,
    pub pay_period: String,
    pub date: String,
    pub state: String,
    #[serde(default)]
    pub statute_citation: String,
}

fn default_min_wage() -> f64 {
    7.25
}

fn default_pct() -> f64 {
    25.0
}

fn default_multiplier() -> f64 {
    30.0
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DocClause {
    pub heading: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct WageGarnishment {
    pub title: String,
    /// 30× minimum wage — earnings below this are fully protected.
    pub protected_floor_usd: f64,
    /// cap_pct × disposable earnings.
    pub percentage_cap_usd: f64,
    /// Disposable earnings above the protected floor (≥ 0).
    pub above_floor_usd: f64,
    /// The lesser of the percentage cap and the amount above the floor.
    pub garnishable_usd: f64,
    /// Disposable earnings the employee keeps.
    pub employee_keeps_usd: f64,
    /// Which cap bound the result: "percentage", "floor", or "none".
    pub binding_cap: String,
    pub statutory_citation: String,
    pub clauses: Vec<DocClause>,
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

pub fn generate(i: &GarnishmentInput) -> WageGarnishment {
    let floor = cents(i.min_wage_multiplier * i.min_wage_usd);
    let pct_cap = cents(i.disposable_earnings_usd * i.cap_pct / 100.0);
    let above_floor = (i.disposable_earnings_usd - floor).max(0.0);
    let garnishable = cents(pct_cap.min(above_floor));
    let keeps = cents(i.disposable_earnings_usd - garnishable);

    let binding = if garnishable <= 0.0 {
        "none"
    } else if pct_cap <= above_floor {
        "percentage"
    } else {
        "floor"
    };

    let binding_desc = match binding {
        "percentage" => format!("the {:.1}% cap on disposable earnings", i.cap_pct),
        "floor" => format!("the amount above 30× the minimum wage ({})", money(floor)),
        _ => "full protection (earnings at or below the protected floor)".to_string(),
    };

    let calc_body = format!(
        "Disposable earnings are {}. The protected floor is {} ({}× minimum wage of {}). The {:.1}% cap is {}; the amount above the floor is {}. The garnishable amount is the lesser, {}, bound by {}. The employee keeps {}.",
        money(i.disposable_earnings_usd),
        money(floor),
        i.min_wage_multiplier,
        money(i.min_wage_usd),
        i.cap_pct,
        money(pct_cap),
        money(cents(above_floor)),
        money(garnishable),
        binding_desc,
        money(keeps)
    );

    let citation = i.statute_citation.trim();
    let governing = if citation.is_empty() {
        format!("Garnishment is limited by 15 U.S.C. § 1673 and the laws of the State of {}.", i.state)
    } else {
        format!("Garnishment is limited by 15 U.S.C. § 1673 and the laws of the State of {} ({}).", i.state, citation)
    };

    let clauses = vec![
        DocClause {
            heading: "Parties".into(),
            body: format!(
                "Creditor: {}\nEmployee: {}\nEmployer (garnishee): {}\nPay period: {}\nDate: {}",
                i.creditor_name, i.employee_name, i.employer_name, i.pay_period, i.date
            ),
        },
        DocClause {
            heading: "1. Disposable Earnings".into(),
            body: format!(
                "Disposable earnings for the pay period — gross pay less legally required deductions — are {}.",
                money(i.disposable_earnings_usd)
            ),
        },
        DocClause { heading: "2. Garnishment Limit".into(), body: calc_body },
        DocClause {
            heading: "3. Withholding".into(),
            body: format!(
                "The employer shall withhold {} from this pay period and remit it toward the judgment, leaving the employee {}. No greater amount may be withheld for ordinary debt without further order.",
                money(garnishable), money(keeps)
            ),
        },
        DocClause { heading: "4. Governing Law".into(), body: governing },
        DocClause {
            heading: "Certification".into(),
            body: format!(
                "Employer (garnishee): ____________________  Date: __________\n{}",
                i.employer_name
            ),
        },
    ];

    WageGarnishment {
        title: "Wage Garnishment Calculation".into(),
        protected_floor_usd: floor,
        percentage_cap_usd: pct_cap,
        above_floor_usd: cents(above_floor),
        garnishable_usd: garnishable,
        employee_keeps_usd: keeps,
        binding_cap: binding.to_string(),
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

    fn base() -> GarnishmentInput {
        GarnishmentInput {
            creditor_name: "Collections LLC".into(),
            employee_name: "Pat Employee".into(),
            employer_name: "Acme Co".into(),
            disposable_earnings_usd: 600.0,
            min_wage_usd: 7.25,
            cap_pct: 25.0,
            min_wage_multiplier: 30.0,
            pay_period: "weekly".into(),
            date: "2026-07-01".into(),
            state: "Delaware".into(),
            statute_citation: String::new(),
        }
    }

    #[test]
    fn percentage_cap_binds() {
        let d = generate(&base());
        assert!(close(d.protected_floor_usd, 217.50));
        assert!(close(d.percentage_cap_usd, 150.0));
        assert!(close(d.above_floor_usd, 382.50));
        assert!(close(d.garnishable_usd, 150.0));
        assert!(close(d.employee_keeps_usd, 450.0));
        assert_eq!(d.binding_cap, "percentage");
    }

    #[test]
    fn floor_cap_binds_for_low_income() {
        let d = generate(&GarnishmentInput { disposable_earnings_usd: 250.0, ..base() });
        // 25% = 62.50; above floor = 32.50 → lesser is 32.50.
        assert!(close(d.garnishable_usd, 32.50));
        assert_eq!(d.binding_cap, "floor");
    }

    #[test]
    fn fully_protected_below_floor() {
        let d = generate(&GarnishmentInput { disposable_earnings_usd: 200.0, ..base() });
        assert!(close(d.above_floor_usd, 0.0));
        assert!(close(d.garnishable_usd, 0.0));
        assert_eq!(d.binding_cap, "none");
        assert!(close(d.employee_keeps_usd, 200.0));
    }

    #[test]
    fn support_higher_percentage() {
        // Child support can reach 50–65%; here 50% of 600 = 300, still below above-floor 382.50.
        let d = generate(&GarnishmentInput { cap_pct: 50.0, ..base() });
        assert!(close(d.garnishable_usd, 300.0));
        assert_eq!(d.binding_cap, "percentage");
    }

    #[test]
    fn statute_citation_echoed() {
        let d = generate(&GarnishmentInput { statute_citation: "Del. Code tit. 10 § 4913".into(), ..base() });
        assert_eq!(d.statutory_citation, "Del. Code tit. 10 § 4913");
        assert!(d.clauses.iter().any(|c| c.body.contains("Del. Code tit. 10 § 4913")));
    }
}
