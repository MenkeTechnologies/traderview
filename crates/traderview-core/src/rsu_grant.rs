//! Restricted stock unit (RSU) grant — a full-value equity award, distinct from
//! a stock option grant: there is no strike price, no exercise, and no AMT. On
//! vesting, the fair-market value of the vested units is ordinary income, and an
//! employer typically withholds taxes by holding back shares ("sell-to-cover"),
//! delivering the net. It reuses the shared cliff+monthly vesting schedule from
//! `option_grant` (no duplicated vesting math). Drafting aid, not legal/tax advice.

use crate::option_grant::{months_between, vested_shares};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct RsuGrantInput {
    pub company_name: String,
    pub grantee_name: String,
    /// Total RSUs granted.
    pub total_units: f64,
    /// Current fair market value per share.
    pub fmv_usd: f64,
    /// Tax withholding rate on the vest value, percent (US supplemental wage
    /// default is 22%).
    #[serde(default = "default_withholding")]
    pub withholding_pct: f64,
    /// Total vesting term in months (e.g. 48 for a 4-year vest).
    pub vesting_months: u32,
    /// Cliff length in months before any vesting (e.g. 12).
    pub cliff_months: u32,
    pub grant_date: String,
    /// Date to value vesting against.
    pub as_of_date: String,
    pub state: String,
    #[serde(default)]
    pub statute_citation: String,
}

fn default_withholding() -> f64 {
    22.0
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DocClause {
    pub heading: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct RsuGrant {
    pub title: String,
    pub total_units: f64,
    pub months_elapsed: i64,
    pub vested_units: f64,
    pub unvested_units: f64,
    /// Ordinary income recognized on the vested units (fmv × vested).
    pub vest_value_usd: f64,
    /// Tax withheld at vest (withholding_pct × vest value).
    pub tax_withheld_usd: f64,
    /// Shares held back to fund the withholding (rounded up to cover it).
    pub shares_withheld: f64,
    /// Net shares delivered after sell-to-cover.
    pub net_shares_delivered: f64,
    pub statutory_citation: String,
    pub clauses: Vec<DocClause>,
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

pub fn generate(i: &RsuGrantInput) -> RsuGrant {
    let grant = NaiveDate::parse_from_str(&i.grant_date, "%Y-%m-%d");
    let as_of = NaiveDate::parse_from_str(&i.as_of_date, "%Y-%m-%d");
    let elapsed = match (grant, as_of) {
        (Ok(g), Ok(a)) => months_between(g, a),
        _ => 0,
    };

    let vested = vested_shares(i.total_units, i.vesting_months, i.cliff_months, elapsed);
    let unvested = (i.total_units - vested).max(0.0);

    let vest_value = cents(i.fmv_usd * vested);
    let tax_withheld = cents(vest_value * i.withholding_pct / 100.0);
    // Whole shares held back, rounded up so the proceeds cover the withholding.
    let shares_withheld = if i.fmv_usd > 0.0 {
        (tax_withheld / i.fmv_usd).ceil()
    } else {
        0.0
    };
    let net_shares = (vested - shares_withheld).max(0.0);

    let citation = i.statute_citation.trim();
    let governing = if citation.is_empty() {
        format!("This grant is governed by the laws of the State of {}.", i.state)
    } else {
        format!("This grant is governed by the laws of the State of {} ({}).", i.state, citation)
    };

    let years = i.vesting_months as f64 / 12.0;
    let vesting_body = format!(
        "The RSUs vest over {} months ({:.2} years) with a {}-month cliff. No units vest before the cliff; thereafter units vest monthly in equal installments. As of {}, {} month(s) have elapsed and {:.0} of {:.0} units are vested ({:.0} unvested).",
        i.vesting_months, years, i.cliff_months, i.as_of_date, elapsed, vested, i.total_units, unvested
    );

    let settlement_body = format!(
        "On vesting, each unit settles into one share of common stock. The {} fair market value of the {:.0} vested units is ordinary income of {} in the year of vesting. At a {:.1}% withholding rate, {} of tax is withheld by holding back {:.0} shares (sell-to-cover), and {:.0} net shares are delivered. RSUs have no strike price and are not exercised.",
        money(i.fmv_usd), vested, money(vest_value), i.withholding_pct, money(tax_withheld), shares_withheld, net_shares
    );

    let clauses = vec![
        DocClause {
            heading: "Parties".into(),
            body: format!(
                "Company: {}\nGrantee: {}\nGrant date: {}",
                i.company_name, i.grantee_name, i.grant_date
            ),
        },
        DocClause {
            heading: "1. Grant".into(),
            body: format!(
                "The Company grants the Grantee {:.0} restricted stock units, each representing the right to receive one share of common stock on vesting, subject to the Company's equity plan and this agreement.",
                i.total_units
            ),
        },
        DocClause { heading: "2. Vesting".into(), body: vesting_body },
        DocClause {
            heading: "3. Forfeiture".into(),
            body: "Unvested units are forfeited automatically and without consideration on termination of service. Vesting does not continue after the service relationship ends except as expressly provided in the plan.".into(),
        },
        DocClause { heading: "4. Settlement & Tax Withholding".into(), body: settlement_body },
        DocClause {
            heading: "5. Transfer Restrictions".into(),
            body: "Unvested units are non-transferable. Shares delivered on settlement are subject to the Company's insider-trading policy and applicable securities-law transfer restrictions.".into(),
        },
        DocClause { heading: "6. Governing Law".into(), body: governing },
        DocClause {
            heading: "Signatures".into(),
            body: format!(
                "Company: ____________________  Date: __________\n{}\n\nGrantee: ____________________  Date: __________\n{}",
                i.company_name, i.grantee_name
            ),
        },
    ];

    RsuGrant {
        title: "Restricted Stock Unit (RSU) Grant".into(),
        total_units: i.total_units,
        months_elapsed: elapsed,
        vested_units: vested,
        unvested_units: unvested,
        vest_value_usd: vest_value,
        tax_withheld_usd: tax_withheld,
        shares_withheld,
        net_shares_delivered: net_shares,
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

    fn base() -> RsuGrantInput {
        RsuGrantInput {
            company_name: "Startup Inc".into(),
            grantee_name: "Jane Engineer".into(),
            total_units: 40_000.0,
            fmv_usd: 10.00,
            withholding_pct: 22.0,
            vesting_months: 48,
            cliff_months: 12,
            grant_date: "2024-01-01".into(),
            as_of_date: "2026-07-01".into(),
            state: "Delaware".into(),
            statute_citation: String::new(),
        }
    }

    #[test]
    fn vesting_30_months_in() {
        let g = generate(&base());
        assert_eq!(g.months_elapsed, 30);
        assert!(close(g.vested_units, 25_000.0));
        assert!(close(g.unvested_units, 15_000.0));
    }

    #[test]
    fn vest_value_and_sell_to_cover() {
        let g = generate(&base());
        assert!(close(g.vest_value_usd, 250_000.0));
        assert!(close(g.tax_withheld_usd, 55_000.0));
        // 55,000 / 10 = 5,500 shares held back; 25,000 - 5,500 = 19,500 delivered.
        assert!(close(g.shares_withheld, 5_500.0));
        assert!(close(g.net_shares_delivered, 19_500.0));
    }

    #[test]
    fn shares_withheld_round_up_to_cover() {
        // Odd FMV so tax/fmv is fractional and must round up.
        let g = generate(&RsuGrantInput { fmv_usd: 3.0, ..base() });
        // vested 25,000 × $3 = $75,000 income; 22% = $16,500 tax; 16500/3 = 5,500 exact.
        assert!(close(g.tax_withheld_usd, 16_500.0));
        assert!(close(g.shares_withheld, 5_500.0));
    }

    #[test]
    fn pre_cliff_zero() {
        let g = generate(&RsuGrantInput { as_of_date: "2024-07-01".into(), ..base() });
        assert_eq!(g.months_elapsed, 6);
        assert!(close(g.vested_units, 0.0));
        assert!(close(g.vest_value_usd, 0.0));
    }

    #[test]
    fn fully_vested_caps_at_total() {
        let g = generate(&RsuGrantInput { as_of_date: "2030-01-01".into(), ..base() });
        assert!(close(g.vested_units, 40_000.0));
        assert!(close(g.unvested_units, 0.0));
    }

    #[test]
    fn no_strike_or_exercise_language() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading.contains("Settlement")).unwrap();
        assert!(c.body.contains("no strike price"));
        assert!(c.body.contains("ordinary income"));
    }

    #[test]
    fn zero_withholding_delivers_all_vested() {
        let g = generate(&RsuGrantInput { withholding_pct: 0.0, ..base() });
        assert!(close(g.shares_withheld, 0.0));
        assert!(close(g.net_shares_delivered, 25_000.0));
    }

    #[test]
    fn statute_citation_echoed() {
        let g = generate(&RsuGrantInput { statute_citation: "26 U.S.C. § 83".into(), ..base() });
        assert_eq!(g.statutory_citation, "26 U.S.C. § 83");
        assert!(g.clauses.iter().any(|c| c.body.contains("26 U.S.C. § 83")));
    }
}
