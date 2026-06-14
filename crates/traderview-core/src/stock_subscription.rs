//! Stock subscription agreement — an investor subscribes for newly-issued
//! shares of a corporation. It computes the total investment (shares × price per
//! share) and the investor's resulting ownership percentage of the shares
//! outstanding after the issuance, then assembles the subscription clauses.
//! Distinct from the LLC operating agreement (corporate stock, not LLC
//! membership). Drafting aid, not legal/securities advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct StockSubscriptionInput {
    pub company_name: String,
    pub investor_name: String,
    pub shares_purchased: f64,
    pub price_per_share_usd: f64,
    #[serde(default)]
    pub par_value_usd: f64,
    /// Total shares outstanding AFTER this issuance (for the ownership %).
    pub total_shares_after: f64,
    pub closing_date: String,
    /// Whether the investor represents being an accredited investor.
    #[serde(default)]
    pub accredited: bool,
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
pub struct StockSubscription {
    pub title: String,
    pub shares_purchased: f64,
    pub price_per_share_usd: f64,
    pub total_investment_usd: f64,
    pub ownership_pct: f64,
    pub statutory_citation: String,
    pub clauses: Vec<DocClause>,
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

fn commas(n: f64) -> String {
    // Whole-share formatting with thousands separators.
    let n = n.round() as i64;
    let s = n.abs().to_string();
    let mut out = String::new();
    for (idx, ch) in s.chars().enumerate() {
        if idx > 0 && (s.len() - idx).is_multiple_of(3) {
            out.push(',');
        }
        out.push(ch);
    }
    if n < 0 {
        format!("-{out}")
    } else {
        out
    }
}

pub fn generate(i: &StockSubscriptionInput) -> StockSubscription {
    let total_investment = cents(i.shares_purchased * i.price_per_share_usd);
    let ownership_pct = if i.total_shares_after > 0.0 {
        cents(i.shares_purchased / i.total_shares_after * 100.0)
    } else {
        0.0
    };

    let citation = i.statute_citation.trim();
    let pursuant = if citation.is_empty() {
        format!("This agreement is governed by the laws of the State of {}.", i.state)
    } else {
        format!("This agreement is governed by the laws of the State of {} ({}).", i.state, citation)
    };

    let par_note = if i.par_value_usd > 0.0 {
        format!(" (par value {} per share)", money(i.par_value_usd))
    } else {
        String::new()
    };

    let accredited_body = if i.accredited {
        "The Investor represents that it is an accredited investor as defined under applicable securities law and is acquiring the shares for investment, not with a view to distribution.".to_string()
    } else {
        "The Investor is acquiring the shares for investment, not with a view to distribution. The shares are not registered and are subject to transfer restrictions under applicable securities law.".to_string()
    };

    let clauses = vec![
        DocClause {
            heading: "Parties".into(),
            body: format!("Company: {}\nInvestor: {}", i.company_name, i.investor_name),
        },
        DocClause {
            heading: "1. Subscription".into(),
            body: format!(
                "The Investor subscribes for and agrees to purchase {} shares of common stock of the Company{} at {} per share, for a total purchase price of {}.",
                commas(i.shares_purchased), par_note, money(i.price_per_share_usd), money(total_investment)
            ),
        },
        DocClause {
            heading: "2. Resulting Ownership".into(),
            body: format!(
                "Following this issuance there will be {} shares outstanding, of which the Investor's {} shares represent {:.2}% ownership.",
                commas(i.total_shares_after), commas(i.shares_purchased), ownership_pct
            ),
        },
        DocClause {
            heading: "3. Closing".into(),
            body: format!(
                "Closing shall occur on {}, at which the Investor pays the purchase price and the Company issues the shares and records the Investor on its stock ledger.",
                i.closing_date
            ),
        },
        DocClause { heading: "4. Investor Representations".into(), body: accredited_body },
        DocClause { heading: "5. Governing Law".into(), body: pursuant },
        DocClause {
            heading: "Signatures".into(),
            body: format!(
                "Company: ____________________  Date: __________\n{}\n\nInvestor: ____________________  Date: __________\n{}",
                i.company_name, i.investor_name
            ),
        },
    ];

    StockSubscription {
        title: "Stock Subscription Agreement".into(),
        shares_purchased: i.shares_purchased,
        price_per_share_usd: i.price_per_share_usd,
        total_investment_usd: total_investment,
        ownership_pct,
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

    fn base() -> StockSubscriptionInput {
        StockSubscriptionInput {
            company_name: "Widgets Inc".into(),
            investor_name: "Ivy Investor".into(),
            shares_purchased: 100_000.0,
            price_per_share_usd: 1.00,
            par_value_usd: 0.0001,
            total_shares_after: 1_000_000.0,
            closing_date: "2026-07-15".into(),
            accredited: true,
            state: "Delaware".into(),
            statute_citation: String::new(),
        }
    }

    #[test]
    fn investment_and_ownership() {
        let d = generate(&base());
        assert!(close(d.total_investment_usd, 100_000.0));
        assert!(close(d.ownership_pct, 10.0));
    }

    #[test]
    fn subscription_clause_formats_shares() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading == "1. Subscription").unwrap();
        assert!(c.body.contains("100,000 shares"));
        assert!(c.body.contains("$1.00 per share"));
        assert!(c.body.contains("$100000.00"));
    }

    #[test]
    fn ownership_clause_shows_total() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading.contains("Resulting Ownership")).unwrap();
        assert!(c.body.contains("1,000,000 shares outstanding"));
        assert!(c.body.contains("10.00% ownership"));
    }

    #[test]
    fn accredited_representation() {
        assert!(generate(&base()).clauses.iter().find(|c| c.heading.contains("Representations")).unwrap().body.contains("accredited investor"));
        let na = generate(&StockSubscriptionInput { accredited: false, ..base() });
        assert!(!na.clauses.iter().find(|c| c.heading.contains("Representations")).unwrap().body.contains("accredited investor"));
    }

    #[test]
    fn par_value_note_when_set() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading == "1. Subscription").unwrap();
        assert!(c.body.contains("par value $0.00 per share") || c.body.contains("par value"));
        let no = generate(&StockSubscriptionInput { par_value_usd: 0.0, ..base() });
        assert!(!no.clauses.iter().find(|c| c.heading == "1. Subscription").unwrap().body.contains("par value"));
    }

    #[test]
    fn statute_citation_echoed() {
        let d = generate(&StockSubscriptionInput { statute_citation: "8 Del. C. § 152".into(), ..base() });
        assert_eq!(d.statutory_citation, "8 Del. C. § 152");
        assert!(d.clauses.iter().any(|c| c.body.contains("8 Del. C. § 152")));
    }
}
