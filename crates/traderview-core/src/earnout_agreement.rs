//! Earnout agreement — the contingent-consideration provision of a business
//! acquisition: part of the price is paid up front and the rest is earned later
//! if the acquired business hits a performance target (revenue, EBITDA, etc.).
//! The earnout pays a rate on the amount by which the actual metric exceeds a
//! threshold, optionally capped. No existing generator computes a threshold-rate-
//! cap contingent payment. Drafting aid, not legal/tax advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct EarnoutInput {
    pub buyer_name: String,
    pub seller_name: String,
    pub business_name: String,
    /// Cash paid at closing.
    pub upfront_usd: f64,
    /// Performance metric tracked (e.g. "trailing-twelve-month revenue").
    #[serde(default)]
    pub metric_label: String,
    /// Threshold the metric must exceed before any earnout is paid.
    pub threshold_usd: f64,
    /// Actual metric achieved in the earnout period.
    pub actual_usd: f64,
    /// Earnout rate on the excess over the threshold, percent.
    pub rate_pct: f64,
    /// Maximum earnout payable (0 = uncapped).
    #[serde(default)]
    pub cap_usd: f64,
    /// Length of the earnout measurement period, in months.
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
pub struct EarnoutAgreement {
    pub title: String,
    pub upfront_usd: f64,
    /// Amount by which the actual metric exceeds the threshold (0 if below).
    pub excess_usd: f64,
    /// Earnout before applying the cap (rate × excess).
    pub uncapped_earnout_usd: f64,
    /// Earnout actually payable after the cap.
    pub earnout_usd: f64,
    /// True when the cap reduced the earnout.
    pub cap_applied: bool,
    /// Upfront + earnout.
    pub total_consideration_usd: f64,
    /// Earnout as a percent of total consideration.
    pub earnout_pct_of_total: f64,
    pub statutory_citation: String,
    pub clauses: Vec<DocClause>,
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

pub fn generate(i: &EarnoutInput) -> EarnoutAgreement {
    let excess = (i.actual_usd - i.threshold_usd).max(0.0);
    let uncapped = cents(excess * i.rate_pct / 100.0);
    let earnout = if i.cap_usd > 0.0 {
        uncapped.min(i.cap_usd)
    } else {
        uncapped
    };
    let cap_applied = i.cap_usd > 0.0 && uncapped > i.cap_usd;
    let total = cents(i.upfront_usd + earnout);
    let earnout_pct = if total > 0.0 {
        cents(earnout / total * 100.0)
    } else {
        0.0
    };

    let metric = if i.metric_label.trim().is_empty() {
        "the performance metric".to_string()
    } else {
        i.metric_label.trim().to_string()
    };

    let cap_clause = if i.cap_usd > 0.0 {
        format!(", subject to a maximum earnout of {}", money(i.cap_usd))
    } else {
        String::new()
    };

    let calc_body = format!(
        "Earnout equals {:.1}% of the amount by which {} exceeds the threshold of {}{}. Actual {} of {} exceeds the threshold by {}, producing an uncapped earnout of {} and a payable earnout of {}{}.",
        i.rate_pct,
        metric,
        money(i.threshold_usd),
        cap_clause,
        metric,
        money(i.actual_usd),
        money(excess),
        money(uncapped),
        money(earnout),
        if cap_applied { " (reduced by the cap)" } else { "" }
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
                "Buyer: {}\nSeller: {}\nBusiness: {}\nDate: {}",
                i.buyer_name, i.seller_name, i.business_name, i.date
            ),
        },
        DocClause {
            heading: "1. Purchase Price".into(),
            body: format!(
                "The Buyer pays the Seller {} in cash at closing, plus the contingent earnout described below. Total consideration if the earnout is earned in full as computed is {}.",
                money(i.upfront_usd),
                money(total)
            ),
        },
        DocClause {
            heading: "2. Earnout Period".into(),
            body: format!(
                "The earnout is measured over the {}-month period following closing, based on {}.",
                i.period_months, metric
            ),
        },
        DocClause { heading: "3. Earnout Calculation".into(), body: calc_body },
        DocClause {
            heading: "4. Payment".into(),
            body: format!(
                "The earnout of {} is payable within 60 days after the Buyer's determination of {} for the earnout period, accompanied by a statement showing the calculation. The Seller may dispute the statement within 30 days.",
                money(earnout),
                metric
            ),
        },
        DocClause {
            heading: "5. Operation of Business".into(),
            body: "During the earnout period the Buyer shall operate the business in good faith and shall not take actions with the primary purpose of reducing the earnout. The Buyer is not otherwise restricted in operating the combined business.".into(),
        },
        DocClause { heading: "6. Governing Law".into(), body: governing },
        DocClause {
            heading: "Signatures".into(),
            body: format!(
                "Buyer: ____________________  Date: __________\n{}\n\nSeller: ____________________  Date: __________\n{}",
                i.buyer_name, i.seller_name
            ),
        },
    ];

    EarnoutAgreement {
        title: "Earnout Agreement".into(),
        upfront_usd: cents(i.upfront_usd),
        excess_usd: cents(excess),
        uncapped_earnout_usd: uncapped,
        earnout_usd: cents(earnout),
        cap_applied,
        total_consideration_usd: total,
        earnout_pct_of_total: earnout_pct,
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

    fn base() -> EarnoutInput {
        EarnoutInput {
            buyer_name: "Acquirer Inc".into(),
            seller_name: "Founder Holdings".into(),
            business_name: "Target Co".into(),
            upfront_usd: 2_000_000.0,
            metric_label: "trailing-twelve-month revenue".into(),
            threshold_usd: 1_000_000.0,
            actual_usd: 3_000_000.0,
            rate_pct: 20.0,
            cap_usd: 500_000.0,
            period_months: 24,
            date: "2026-07-01".into(),
            state: "Delaware".into(),
            statute_citation: String::new(),
        }
    }

    #[test]
    fn earnout_under_cap() {
        let d = generate(&base());
        assert!(close(d.excess_usd, 2_000_000.0));
        assert!(close(d.uncapped_earnout_usd, 400_000.0));
        assert!(close(d.earnout_usd, 400_000.0));
        assert!(!d.cap_applied);
        assert!(close(d.total_consideration_usd, 2_400_000.0));
    }

    #[test]
    fn cap_reduces_large_earnout() {
        let d = generate(&EarnoutInput { actual_usd: 5_000_000.0, ..base() });
        assert!(close(d.uncapped_earnout_usd, 800_000.0));
        assert!(close(d.earnout_usd, 500_000.0));
        assert!(d.cap_applied);
        assert!(close(d.total_consideration_usd, 2_500_000.0));
    }

    #[test]
    fn below_threshold_zero_earnout() {
        let d = generate(&EarnoutInput { actual_usd: 800_000.0, ..base() });
        assert!(close(d.excess_usd, 0.0));
        assert!(close(d.earnout_usd, 0.0));
        assert!(!d.cap_applied);
        assert!(close(d.total_consideration_usd, 2_000_000.0));
    }

    #[test]
    fn uncapped_pays_full() {
        let d = generate(&EarnoutInput { actual_usd: 5_000_000.0, cap_usd: 0.0, ..base() });
        assert!(close(d.earnout_usd, 800_000.0));
        assert!(!d.cap_applied);
    }

    #[test]
    fn earnout_pct_of_total() {
        // 400,000 / 2,400,000 = 16.67%.
        assert!(close(generate(&base()).earnout_pct_of_total, 16.67));
    }

    #[test]
    fn good_faith_operation_clause_present() {
        assert!(generate(&base()).clauses.iter().any(|c| c.body.contains("good faith")));
    }

    #[test]
    fn statute_citation_echoed() {
        let d = generate(&EarnoutInput { statute_citation: "DGCL § 251".into(), ..base() });
        assert_eq!(d.statutory_citation, "DGCL § 251");
        assert!(d.clauses.iter().any(|c| c.body.contains("DGCL § 251")));
    }
}
