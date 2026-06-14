//! SAFE (Simple Agreement for Future Equity) — a startup investment that, unlike
//! a convertible note, is NOT debt: there is no interest and no maturity date.
//! The investor's money converts to equity at the next priced round at the
//! better of the discount price and the valuation-cap price. It reuses the
//! shared discount/cap conversion helper (no duplicated math) and computes the
//! shares the investment buys. Drafting aid, not legal/securities advice.

use crate::convertible_note::discount_cap_conversion;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct SafeInput {
    pub company_name: String,
    pub investor_name: String,
    pub investment_usd: f64,
    /// Conversion discount, percent (0 = none).
    #[serde(default)]
    pub discount_pct: f64,
    /// Valuation cap (0 = uncapped).
    #[serde(default)]
    pub valuation_cap_usd: f64,
    /// Assumed next-round price per share, for the illustrative conversion.
    #[serde(default)]
    pub assumed_round_price_per_share_usd: f64,
    /// Assumed next-round pre-money valuation, for the cap price.
    #[serde(default)]
    pub assumed_round_pre_money_usd: f64,
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
pub struct SafeAgreement {
    pub title: String,
    pub investment_usd: f64,
    pub discount_price_usd: f64,
    pub cap_price_usd: f64,
    pub conversion_price_usd: f64,
    pub shares_on_conversion: f64,
    pub statutory_citation: String,
    pub clauses: Vec<DocClause>,
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

pub fn generate(i: &SafeInput) -> SafeAgreement {
    let round_price = i.assumed_round_price_per_share_usd;
    let (discount_price, cap_price, conversion_price) = discount_cap_conversion(
        round_price,
        i.discount_pct,
        i.valuation_cap_usd,
        i.assumed_round_pre_money_usd,
    );
    let shares = if conversion_price > 0.0 {
        cents(i.investment_usd / conversion_price)
    } else {
        0.0
    };

    let citation = i.statute_citation.trim();
    let pursuant = if citation.is_empty() {
        format!("This SAFE is governed by the laws of the State of {}.", i.state)
    } else {
        format!("This SAFE is governed by the laws of the State of {} ({}).", i.state, citation)
    };

    let mut terms: Vec<String> = Vec::new();
    if i.discount_pct > 0.0 {
        terms.push(format!("a {:.1}% discount", i.discount_pct));
    }
    if i.valuation_cap_usd > 0.0 {
        terms.push(format!("a valuation cap of {}", money(i.valuation_cap_usd)));
    }
    let terms_str = if terms.is_empty() {
        "the round price".to_string()
    } else {
        format!("the more favorable of {}", terms.join(" and "))
    };

    let illustration = if round_price > 0.0 {
        format!(
            " Illustration at an assumed round price of {} (pre-money {}): discount price {}, cap price {}, conversion price {}, converting into approximately {:.0} shares.",
            money(round_price),
            money(i.assumed_round_pre_money_usd),
            money(discount_price),
            money(cap_price),
            money(conversion_price),
            shares
        )
    } else {
        String::new()
    };

    let clauses = vec![
        DocClause {
            heading: "Parties".into(),
            body: format!("Company: {}\nInvestor: {}\nDate: {}", i.company_name, i.investor_name, i.date),
        },
        DocClause {
            heading: "1. Investment".into(),
            body: format!(
                "The Investor pays the Company {} (the \"Purchase Amount\") in exchange for the right to certain shares of the Company as set forth below. This SAFE is not a debt instrument; it bears no interest and has no maturity date.",
                money(i.investment_usd)
            ),
        },
        DocClause {
            heading: "2. Conversion".into(),
            body: format!(
                "Upon the Company's next equity financing, the Purchase Amount converts into shares of that round at {}.{}",
                terms_str, illustration
            ),
        },
        DocClause {
            heading: "3. Liquidity / Dissolution".into(),
            body: "On a liquidity event (acquisition or IPO) or a dissolution before conversion, the Investor is entitled to the greater of the Purchase Amount or the as-converted value, as provided in the agreement.".into(),
        },
        DocClause {
            heading: "4. Representations".into(),
            body: "The Investor is acquiring this SAFE for investment; the SAFE and any shares issued on conversion are not registered and are subject to transfer restrictions under applicable securities law.".into(),
        },
        DocClause { heading: "5. Governing Law".into(), body: pursuant },
        DocClause {
            heading: "Signatures".into(),
            body: format!(
                "Company: ____________________  Date: __________\n{}\n\nInvestor: ____________________  Date: __________\n{}",
                i.company_name, i.investor_name
            ),
        },
    ];

    SafeAgreement {
        title: "Simple Agreement for Future Equity (SAFE)".into(),
        investment_usd: i.investment_usd,
        discount_price_usd: discount_price,
        cap_price_usd: cap_price,
        conversion_price_usd: conversion_price,
        shares_on_conversion: shares,
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

    fn base() -> SafeInput {
        SafeInput {
            company_name: "Startup Inc".into(),
            investor_name: "Angel Investor".into(),
            investment_usd: 100_000.0,
            discount_pct: 20.0,
            valuation_cap_usd: 5_000_000.0,
            assumed_round_price_per_share_usd: 2.00,
            assumed_round_pre_money_usd: 10_000_000.0,
            date: "2026-07-01".into(),
            state: "Delaware".into(),
            statute_citation: String::new(),
        }
    }

    #[test]
    fn cap_beats_discount_and_shares() {
        let d = generate(&base());
        assert!(close(d.discount_price_usd, 1.60));
        assert!(close(d.cap_price_usd, 1.00));
        assert!(close(d.conversion_price_usd, 1.00));
        // 100,000 / 1.00 = 100,000 shares (no interest accrual, unlike a note).
        assert!(close(d.shares_on_conversion, 100_000.0));
    }

    #[test]
    fn no_interest_or_maturity_language() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading == "1. Investment").unwrap();
        assert!(c.body.contains("not a debt instrument"));
        assert!(c.body.contains("no interest"));
        assert!(c.body.contains("no maturity"));
    }

    #[test]
    fn discount_wins_when_cap_high() {
        let d = generate(&SafeInput { valuation_cap_usd: 20_000_000.0, ..base() });
        assert!(close(d.conversion_price_usd, 1.60));
    }

    #[test]
    fn uncapped_no_discount_uses_round_price() {
        let d = generate(&SafeInput { discount_pct: 0.0, valuation_cap_usd: 0.0, ..base() });
        assert!(close(d.conversion_price_usd, 2.00));
        assert!(close(d.shares_on_conversion, 50_000.0));
    }

    #[test]
    fn liquidity_clause_present() {
        assert!(generate(&base()).clauses.iter().any(|c| c.heading.contains("Liquidity")));
    }

    #[test]
    fn statute_citation_echoed() {
        let d = generate(&SafeInput { statute_citation: "Cal. Corp. Code".into(), ..base() });
        assert_eq!(d.statutory_citation, "Cal. Corp. Code");
        assert!(d.clauses.iter().any(|c| c.body.contains("Cal. Corp. Code")));
    }
}
