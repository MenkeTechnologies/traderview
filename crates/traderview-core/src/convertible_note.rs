//! Convertible note — a startup raises money as debt that converts to equity at
//! the next priced round. It accrues simple interest to maturity, then computes
//! the conversion price as the better-for-investor of the discount price and the
//! valuation-cap price at an assumed round, and the shares the note balance
//! buys. Distinct from the promissory note (this converts to equity) and the
//! stock subscription (this is convertible debt). Drafting aid, not
//! legal/securities advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct ConvertibleNoteInput {
    pub company_name: String,
    pub investor_name: String,
    pub principal_usd: f64,
    pub annual_rate_pct: f64,
    pub term_months: u32,
    /// Conversion discount, percent (e.g. 20 = 20% off the round price).
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
    pub issue_date: String,
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
pub struct ConvertibleNote {
    pub title: String,
    pub accrued_interest_usd: f64,
    pub note_balance_usd: f64,
    pub discount_price_usd: f64,
    pub cap_price_usd: f64,
    /// The lower (better-for-investor) of the available conversion prices.
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

/// Discount-or-cap conversion pricing shared by the convertible note and the
/// SAFE. Given an assumed round price per share, returns `(discount_price,
/// cap_price, conversion_price)` where the conversion price is the lower
/// (better-for-investor) of the available positive prices, falling back to the
/// round price when neither a discount nor a cap applies. A zero discount, cap,
/// or pre-money disables that leg. All figures rounded to cents.
pub fn discount_cap_conversion(
    round_price: f64,
    discount_pct: f64,
    valuation_cap_usd: f64,
    round_pre_money_usd: f64,
) -> (f64, f64, f64) {
    let discount_price = if discount_pct > 0.0 && round_price > 0.0 {
        cents(round_price * (1.0 - discount_pct / 100.0))
    } else {
        0.0
    };
    let cap_price = if valuation_cap_usd > 0.0 && round_pre_money_usd > 0.0 && round_price > 0.0 {
        cents(round_price * (valuation_cap_usd / round_pre_money_usd))
    } else {
        0.0
    };
    let conversion = [discount_price, cap_price]
        .into_iter()
        .filter(|p| *p > 0.0)
        .fold(f64::INFINITY, f64::min);
    let conversion = if conversion.is_finite() { conversion } else { round_price };
    (discount_price, cap_price, cents(conversion))
}

pub fn generate(i: &ConvertibleNoteInput) -> ConvertibleNote {
    let accrued = cents(i.principal_usd * i.annual_rate_pct / 100.0 * i.term_months as f64 / 12.0);
    let balance = cents(i.principal_usd + accrued);

    let round_price = i.assumed_round_price_per_share_usd;
    let (discount_price, cap_price, conversion_price) = discount_cap_conversion(
        round_price,
        i.discount_pct,
        i.valuation_cap_usd,
        i.assumed_round_pre_money_usd,
    );

    let shares = if conversion_price > 0.0 {
        cents(balance / conversion_price)
    } else {
        0.0
    };

    let citation = i.statute_citation.trim();
    let pursuant = if citation.is_empty() {
        format!("This note is governed by the laws of the State of {}.", i.state)
    } else {
        format!("This note is governed by the laws of the State of {} ({}).", i.state, citation)
    };

    let mut conv_terms: Vec<String> = Vec::new();
    if i.discount_pct > 0.0 {
        conv_terms.push(format!("a {:.1}% discount to the round price", i.discount_pct));
    }
    if i.valuation_cap_usd > 0.0 {
        conv_terms.push(format!("a valuation cap of {}", money(i.valuation_cap_usd)));
    }
    let conv_terms_str = if conv_terms.is_empty() {
        "the round price".to_string()
    } else {
        format!("the more favorable of {}", conv_terms.join(" and "))
    };

    let illustration = if round_price > 0.0 {
        format!(
            " Illustration at an assumed round price of {} (pre-money {}): discount price {}, cap price {}, so the conversion price is {} and the note converts into approximately {:.0} shares.",
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
            body: format!("Company: {}\nInvestor (Holder): {}", i.company_name, i.investor_name),
        },
        DocClause {
            heading: "1. Note".into(),
            body: format!(
                "The Company issues this convertible promissory note to the Holder for a principal amount of {}, dated {}, accruing simple interest at {:.3}% per annum, with a maturity of {} months. Accrued interest at maturity: {} (balance {}).",
                money(i.principal_usd), i.issue_date, i.annual_rate_pct, i.term_months, money(accrued), money(balance)
            ),
        },
        DocClause {
            heading: "2. Conversion".into(),
            body: format!(
                "Upon the Company's next qualified equity financing, the outstanding balance (principal plus accrued interest) automatically converts into shares of that round at {}.{}",
                conv_terms_str, illustration
            ),
        },
        DocClause {
            heading: "3. Maturity".into(),
            body: "If no qualified financing occurs before maturity, the Holder may elect to convert at the terms above (using the cap as the price) or demand repayment of the outstanding balance.".into(),
        },
        DocClause {
            heading: "4. Representations".into(),
            body: "The Holder is acquiring this note for investment; the note and any shares issued on conversion are not registered and are subject to transfer restrictions under applicable securities law.".into(),
        },
        DocClause { heading: "5. Governing Law".into(), body: pursuant },
        DocClause {
            heading: "Signatures".into(),
            body: format!(
                "Company: ____________________  Date: __________\n{}\n\nHolder: ____________________  Date: __________\n{}",
                i.company_name, i.investor_name
            ),
        },
    ];

    ConvertibleNote {
        title: "Convertible Promissory Note".into(),
        accrued_interest_usd: accrued,
        note_balance_usd: balance,
        discount_price_usd: discount_price,
        cap_price_usd: cap_price,
        conversion_price_usd: cents(conversion_price),
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

    fn base() -> ConvertibleNoteInput {
        ConvertibleNoteInput {
            company_name: "Startup Inc".into(),
            investor_name: "Angel Investor".into(),
            principal_usd: 100_000.0,
            annual_rate_pct: 5.0,
            term_months: 24,
            discount_pct: 20.0,
            valuation_cap_usd: 5_000_000.0,
            assumed_round_price_per_share_usd: 2.00,
            assumed_round_pre_money_usd: 10_000_000.0,
            issue_date: "2026-07-01".into(),
            state: "Delaware".into(),
            statute_citation: String::new(),
        }
    }

    #[test]
    fn accrued_and_balance() {
        let d = generate(&base());
        assert!(close(d.accrued_interest_usd, 10_000.0));
        assert!(close(d.note_balance_usd, 110_000.0));
    }

    #[test]
    fn cap_beats_discount() {
        let d = generate(&base());
        assert!(close(d.discount_price_usd, 1.60));
        assert!(close(d.cap_price_usd, 1.00));
        assert!(close(d.conversion_price_usd, 1.00));
        assert!(close(d.shares_on_conversion, 110_000.0));
    }

    #[test]
    fn discount_beats_cap_when_cap_high() {
        // Cap above pre-money → cap price > round price; discount wins.
        let d = generate(&ConvertibleNoteInput { valuation_cap_usd: 20_000_000.0, ..base() });
        assert!(close(d.cap_price_usd, 4.00));
        assert!(close(d.discount_price_usd, 1.60));
        assert!(close(d.conversion_price_usd, 1.60));
    }

    #[test]
    fn uncapped_no_discount_uses_round_price() {
        let d = generate(&ConvertibleNoteInput { discount_pct: 0.0, valuation_cap_usd: 0.0, ..base() });
        assert!(close(d.conversion_price_usd, 2.00));
        assert!(close(d.shares_on_conversion, 55_000.0));
    }

    #[test]
    fn conversion_clause_lists_terms() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading == "2. Conversion").unwrap();
        assert!(c.body.contains("20.0% discount"));
        assert!(c.body.contains("valuation cap of $5000000.00"));
        assert!(c.body.contains("110000 shares") || c.body.contains("110,000") || c.body.contains("110000"));
    }

    #[test]
    fn statute_citation_echoed() {
        let d = generate(&ConvertibleNoteInput { statute_citation: "6 Del. C. § 1-101".into(), ..base() });
        assert_eq!(d.statutory_citation, "6 Del. C. § 1-101");
        assert!(d.clauses.iter().any(|c| c.body.contains("6 Del. C. § 1-101")));
    }
}
