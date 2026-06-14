//! Capitalization table (cap table) — the ledger of who owns a company's
//! equity. Each holder's shares as a percentage of the fully-diluted total
//! (issued shares plus the unallocated option pool) gives their ownership.
//! Distinct from the subscription/operating agreements (this is the resulting
//! ownership ledger). Pure compute plus a document layout. Drafting aid, not
//! legal/securities advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Holder {
    pub name: String,
    pub shares: f64,
    /// Security class label (Common, Preferred, Options, …).
    #[serde(default)]
    pub class: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CapTableInput {
    pub company_name: String,
    pub as_of_date: String,
    #[serde(default)]
    pub holders: Vec<Holder>,
    /// Unallocated option pool shares (counted in the fully-diluted total).
    #[serde(default)]
    pub option_pool_shares: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct HolderRow {
    pub name: String,
    pub shares: f64,
    pub class: String,
    pub ownership_pct: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DocClause {
    pub heading: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CapTable {
    pub title: String,
    pub total_issued_shares: f64,
    pub option_pool_shares: f64,
    pub total_fully_diluted_shares: f64,
    pub holder_count: usize,
    pub holders: Vec<HolderRow>,
    pub option_pool_pct: f64,
    pub clauses: Vec<DocClause>,
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn commas(n: f64) -> String {
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

pub fn generate(i: &CapTableInput) -> CapTable {
    let total_issued: f64 = i.holders.iter().map(|h| h.shares).sum();
    let fully_diluted = total_issued + i.option_pool_shares;

    let pct = |shares: f64| -> f64 {
        if fully_diluted > 0.0 {
            cents(shares / fully_diluted * 100.0)
        } else {
            0.0
        }
    };

    let holders: Vec<HolderRow> = i
        .holders
        .iter()
        .map(|h| HolderRow {
            name: h.name.clone(),
            shares: h.shares,
            class: if h.class.trim().is_empty() { "Common".to_string() } else { h.class.trim().to_string() },
            ownership_pct: pct(h.shares),
        })
        .collect();

    let option_pool_pct = pct(i.option_pool_shares);

    let table_body = if holders.is_empty() {
        "No holders recorded.".to_string()
    } else {
        let mut lines: Vec<String> = holders
            .iter()
            .map(|h| format!("  • {} ({}): {} shares — {:.2}%", h.name, h.class, commas(h.shares), h.ownership_pct))
            .collect();
        if i.option_pool_shares > 0.0 {
            lines.push(format!("  • Option pool (unallocated): {} shares — {:.2}%", commas(i.option_pool_shares), option_pool_pct));
        }
        lines.join("\n")
    };

    let clauses = vec![
        DocClause {
            heading: "Header".into(),
            body: format!("Company: {}\nCapitalization as of: {}", i.company_name, i.as_of_date),
        },
        DocClause { heading: "1. Capitalization".into(), body: table_body },
        DocClause {
            heading: "2. Summary".into(),
            body: format!(
                "Issued shares: {}\nOption pool (unallocated): {}\nFully-diluted total: {}",
                commas(total_issued), commas(i.option_pool_shares), commas(fully_diluted)
            ),
        },
        DocClause {
            heading: "3. Certification".into(),
            body: "This table reflects the Company's capitalization as of the date stated, to the best of the Company's knowledge. It is for reference and is not a substitute for the Company's official stock records.".into(),
        },
        DocClause {
            heading: "Signature".into(),
            body: format!("Prepared by: ____________________  Date: {}\n{}", i.as_of_date, i.company_name),
        },
    ];

    CapTable {
        title: "Capitalization Table".into(),
        total_issued_shares: total_issued,
        option_pool_shares: i.option_pool_shares,
        total_fully_diluted_shares: fully_diluted,
        holder_count: holders.len(),
        holders,
        option_pool_pct,
        clauses,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    fn holder(name: &str, shares: f64, class: &str) -> Holder {
        Holder { name: name.into(), shares, class: class.into() }
    }

    fn base() -> CapTableInput {
        CapTableInput {
            company_name: "Widgets Inc".into(),
            as_of_date: "2026-07-15".into(),
            holders: vec![
                holder("Founder", 800_000.0, "Common"),
                holder("Investor", 200_000.0, "Preferred"),
            ],
            option_pool_shares: 100_000.0,
        }
    }

    #[test]
    fn totals_and_ownership() {
        let d = generate(&base());
        assert!(close(d.total_issued_shares, 1_000_000.0));
        assert!(close(d.total_fully_diluted_shares, 1_100_000.0));
        assert!(close(d.holders[0].ownership_pct, 72.73));
        assert!(close(d.holders[1].ownership_pct, 18.18));
        assert!(close(d.option_pool_pct, 9.09));
    }

    #[test]
    fn percentages_sum_to_100() {
        let d = generate(&base());
        let sum: f64 = d.holders.iter().map(|h| h.ownership_pct).sum::<f64>() + d.option_pool_pct;
        assert!(close(sum, 100.0));
    }

    #[test]
    fn table_lists_holders_and_pool() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading == "1. Capitalization").unwrap();
        assert!(c.body.contains("Founder (Common): 800,000 shares — 72.73%"));
        assert!(c.body.contains("Option pool (unallocated): 100,000 shares — 9.09%"));
    }

    #[test]
    fn no_pool_omits_pool_line() {
        let d = generate(&CapTableInput { option_pool_shares: 0.0, ..base() });
        // Without a pool, fully-diluted = issued; founder is 80%.
        assert!(close(d.holders[0].ownership_pct, 80.0));
        let c = d.clauses.iter().find(|c| c.heading == "1. Capitalization").unwrap();
        assert!(!c.body.contains("Option pool"));
    }

    #[test]
    fn default_class_is_common() {
        let d = generate(&CapTableInput {
            holders: vec![holder("X", 100.0, "")],
            option_pool_shares: 0.0,
            ..base()
        });
        assert_eq!(d.holders[0].class, "Common");
    }

    #[test]
    fn empty_holders() {
        let d = generate(&CapTableInput { holders: vec![], option_pool_shares: 0.0, ..base() });
        assert_eq!(d.holder_count, 0);
        let c = d.clauses.iter().find(|c| c.heading == "1. Capitalization").unwrap();
        assert!(c.body.contains("No holders"));
    }
}
