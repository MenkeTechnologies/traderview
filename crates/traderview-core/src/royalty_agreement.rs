//! Royalty / license agreement — a licensor grants a licensee the right to use
//! intellectual property in exchange for royalties. The earned royalty is a rate
//! on licensed-product revenue; a minimum guarantee sets a floor on what is owed
//! for the period; and a recoupable advance is credited against the amount due.
//! No existing generator computes earned-vs-minimum royalties with advance
//! recoupment. Drafting aid, not legal advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct RoyaltyInput {
    pub licensor_name: String,
    pub licensee_name: String,
    /// What is licensed (e.g. "the Patent", "the Brand").
    #[serde(default)]
    pub property_label: String,
    /// Licensed-product revenue in the period.
    pub revenue_usd: f64,
    /// Royalty rate on revenue, percent.
    pub rate_pct: f64,
    /// Minimum guarantee — floor on royalties owed for the period (0 = none).
    #[serde(default)]
    pub minimum_guarantee_usd: f64,
    /// Recoupable advance already paid, credited against amounts due (0 = none).
    #[serde(default)]
    pub advance_usd: f64,
    /// Royalty reporting period length, in months.
    pub period_months: u32,
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
pub struct RoyaltyAgreement {
    pub title: String,
    /// Rate × revenue.
    pub earned_royalty_usd: f64,
    /// Greater of the earned royalty and the minimum guarantee.
    pub total_due_usd: f64,
    /// True when the minimum guarantee exceeded the earned royalty.
    pub minimum_applied: bool,
    /// Amount payable now after crediting the advance.
    pub payable_now_usd: f64,
    /// Advance not yet recouped by royalties (carries forward).
    pub unrecouped_advance_usd: f64,
    /// Effective royalty rate on revenue after the minimum, percent.
    pub effective_rate_pct: f64,
    pub statutory_citation: String,
    pub clauses: Vec<DocClause>,
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

pub fn generate(i: &RoyaltyInput) -> RoyaltyAgreement {
    let earned = cents(i.revenue_usd * i.rate_pct / 100.0);
    let total_due = earned.max(i.minimum_guarantee_usd);
    let minimum_applied = i.minimum_guarantee_usd > earned;
    let payable = (total_due - i.advance_usd).max(0.0);
    let unrecouped = (i.advance_usd - total_due).max(0.0);
    let effective_rate = if i.revenue_usd > 0.0 {
        cents(total_due / i.revenue_usd * 100.0)
    } else {
        0.0
    };

    let property = if i.property_label.trim().is_empty() {
        "the licensed property".to_string()
    } else {
        i.property_label.trim().to_string()
    };

    let min_clause = if i.minimum_guarantee_usd > 0.0 {
        format!(
            " A minimum guarantee of {} applies; the amount due for the period is the greater of the earned royalty and the minimum.",
            money(i.minimum_guarantee_usd)
        )
    } else {
        String::new()
    };

    let advance_clause = if i.advance_usd > 0.0 {
        format!(
            " A recoupable advance of {} is credited against amounts due; {} remains unrecouped and carries forward.",
            money(i.advance_usd),
            money(unrecouped)
        )
    } else {
        String::new()
    };

    let calc_body = format!(
        "The earned royalty is {:.2}% of {} in licensed-product revenue, or {}.{} The total due for the period is {}, of which {} is payable now.{}",
        i.rate_pct,
        money(i.revenue_usd),
        money(earned),
        min_clause,
        money(total_due),
        money(payable),
        advance_clause
    );

    let citation = i.statute_citation.trim();
    let governing = if citation.is_empty() {
        format!("This agreement is governed by the laws of the State of {}.", i.state)
    } else {
        format!("This agreement is governed by the laws of the State of {} ({}).", i.state, citation)
    };

    let clauses = vec![
        DocClause {
            heading: "Parties".into(),
            body: format!(
                "Licensor: {}\nLicensee: {}\nLicensed property: {}\nDate: {}",
                i.licensor_name, i.licensee_name, property, i.date
            ),
        },
        DocClause {
            heading: "1. Grant of License".into(),
            body: format!(
                "The Licensor grants the Licensee a license to use {} in exchange for the royalties set forth below.",
                property
            ),
        },
        DocClause {
            heading: "2. Royalty".into(),
            body: format!(
                "The Licensee shall pay the Licensor a royalty of {:.2}% of revenue from licensed products.{}",
                i.rate_pct, min_clause
            ),
        },
        DocClause { heading: "3. Calculation".into(), body: calc_body },
        DocClause {
            heading: "4. Reporting & Payment".into(),
            body: format!(
                "Within 30 days after each {}-month reporting period, the Licensee shall deliver a royalty statement and pay the amount due. The Licensor may audit the Licensee's records once per year on reasonable notice.",
                i.period_months
            ),
        },
        DocClause {
            heading: "5. Records".into(),
            body: "The Licensee shall keep complete and accurate records of licensed-product sales sufficient to verify the royalties for at least three years after each period.".into(),
        },
        DocClause { heading: "6. Governing Law".into(), body: governing },
        DocClause {
            heading: "Signatures".into(),
            body: format!(
                "Licensor: ____________________  Date: __________\n{}\n\nLicensee: ____________________  Date: __________\n{}",
                i.licensor_name, i.licensee_name
            ),
        },
    ];

    RoyaltyAgreement {
        title: "Royalty / License Agreement".into(),
        earned_royalty_usd: earned,
        total_due_usd: cents(total_due),
        minimum_applied,
        payable_now_usd: cents(payable),
        unrecouped_advance_usd: cents(unrecouped),
        effective_rate_pct: effective_rate,
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

    fn base() -> RoyaltyInput {
        RoyaltyInput {
            licensor_name: "IP Holdings LLC".into(),
            licensee_name: "Maker Co".into(),
            property_label: "the Patent".into(),
            revenue_usd: 2_000_000.0,
            rate_pct: 8.0,
            minimum_guarantee_usd: 100_000.0,
            advance_usd: 50_000.0,
            period_months: 12,
            date: "2026-07-01".into(),
            state: "Delaware".into(),
            statute_citation: String::new(),
        }
    }

    #[test]
    fn earned_exceeds_minimum_recoup_advance() {
        let d = generate(&base());
        assert!(close(d.earned_royalty_usd, 160_000.0));
        assert!(close(d.total_due_usd, 160_000.0));
        assert!(!d.minimum_applied);
        assert!(close(d.payable_now_usd, 110_000.0));
        assert!(close(d.unrecouped_advance_usd, 0.0));
    }

    #[test]
    fn minimum_guarantee_floors_low_revenue() {
        let d = generate(&RoyaltyInput { revenue_usd: 500_000.0, ..base() });
        assert!(close(d.earned_royalty_usd, 40_000.0));
        assert!(close(d.total_due_usd, 100_000.0));
        assert!(d.minimum_applied);
        assert!(close(d.payable_now_usd, 50_000.0));
    }

    #[test]
    fn large_advance_leaves_unrecouped_balance() {
        let d = generate(&RoyaltyInput { revenue_usd: 100_000.0, advance_usd: 150_000.0, ..base() });
        // earned 8k, minimum 100k → due 100k; advance 150k → payable 0, unrecouped 50k.
        assert!(close(d.payable_now_usd, 0.0));
        assert!(close(d.unrecouped_advance_usd, 50_000.0));
    }

    #[test]
    fn no_minimum_no_advance_pays_earned() {
        let d = generate(&RoyaltyInput { minimum_guarantee_usd: 0.0, advance_usd: 0.0, ..base() });
        assert!(close(d.total_due_usd, 160_000.0));
        assert!(close(d.payable_now_usd, 160_000.0));
        assert!(!d.minimum_applied);
    }

    #[test]
    fn effective_rate_reflects_minimum() {
        // Minimum 100k on 500k revenue → effective 20%, above the 8% stated rate.
        let d = generate(&RoyaltyInput { revenue_usd: 500_000.0, ..base() });
        assert!(close(d.effective_rate_pct, 20.0));
    }

    #[test]
    fn audit_clause_present() {
        assert!(generate(&base()).clauses.iter().any(|c| c.body.contains("audit")));
    }

    #[test]
    fn statute_citation_echoed() {
        let d = generate(&RoyaltyInput { statute_citation: "17 U.S.C. § 204".into(), ..base() });
        assert_eq!(d.statutory_citation, "17 U.S.C. § 204");
        assert!(d.clauses.iter().any(|c| c.body.contains("17 U.S.C. § 204")));
    }
}
