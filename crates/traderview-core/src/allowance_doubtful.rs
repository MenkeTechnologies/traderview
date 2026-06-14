//! Allowance for doubtful accounts (bad-debt reserve) — the aging-method estimate
//! of uncollectible receivables. Each accounts-receivable aging tier carries an
//! estimated uncollectible percentage; the allowance is the sum of each tier's
//! balance times its percentage, and net realizable AR is the total less the
//! allowance. Distinct from the statement of account, which only ages invoices;
//! this reserves against them. Drafting aid, not accounting advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct AgingTier {
    pub label: String,
    pub balance_usd: f64,
    /// Estimated uncollectible percentage for this tier.
    pub uncollectible_pct: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AllowanceInput {
    pub company_name: String,
    pub as_of_date: String,
    pub tiers: Vec<AgingTier>,
    /// Existing balance already in the allowance account (for the adjusting entry).
    #[serde(default)]
    pub existing_allowance_usd: f64,
    #[serde(default)]
    pub note: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TierRow {
    pub label: String,
    pub balance_usd: f64,
    pub uncollectible_pct: f64,
    /// balance × pct.
    pub reserve_usd: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DocClause {
    pub heading: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct AllowanceDoubtful {
    pub title: String,
    pub total_receivables_usd: f64,
    /// Sum of each tier's reserve — the required allowance balance.
    pub allowance_usd: f64,
    /// Total receivables − allowance.
    pub net_realizable_usd: f64,
    /// Allowance ÷ total receivables, percent.
    pub allowance_pct_of_ar: f64,
    /// Required allowance − existing balance (the adjusting entry to bad-debt expense).
    pub adjusting_entry_usd: f64,
    pub rows: Vec<TierRow>,
    pub clauses: Vec<DocClause>,
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

pub fn generate(i: &AllowanceInput) -> AllowanceDoubtful {
    let rows: Vec<TierRow> = i
        .tiers
        .iter()
        .map(|t| TierRow {
            label: t.label.clone(),
            balance_usd: cents(t.balance_usd),
            uncollectible_pct: t.uncollectible_pct,
            reserve_usd: cents(t.balance_usd * t.uncollectible_pct / 100.0),
        })
        .collect();

    let total = cents(i.tiers.iter().map(|t| t.balance_usd).sum());
    let allowance = cents(rows.iter().map(|r| r.reserve_usd).sum());
    let net = cents(total - allowance);
    let allowance_pct = if total > 0.0 {
        cents(allowance / total * 100.0)
    } else {
        0.0
    };
    let adjusting = cents(allowance - i.existing_allowance_usd);

    let detail = if rows.is_empty() {
        "No receivables aging tiers listed.".to_string()
    } else {
        rows.iter()
            .map(|r| {
                format!(
                    "  • {}: {} × {:.2}% = {}",
                    r.label, money(r.balance_usd), r.uncollectible_pct, money(r.reserve_usd)
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    let entry_desc = if adjusting >= 0.0 {
        format!(
            "Debit Bad Debt Expense and credit Allowance for Doubtful Accounts {} to raise the existing balance of {} to the required {}.",
            money(adjusting.abs()), money(i.existing_allowance_usd), money(allowance)
        )
    } else {
        format!(
            "Debit Allowance for Doubtful Accounts and credit Bad Debt Expense {} to reduce the existing balance of {} to the required {}.",
            money(adjusting.abs()), money(i.existing_allowance_usd), money(allowance)
        )
    };

    let mut clauses = vec![
        DocClause {
            heading: "Statement".into(),
            body: format!("Company: {}\nAs of: {}", i.company_name, i.as_of_date),
        },
        DocClause { heading: "Aging Reserve".into(), body: detail },
        DocClause {
            heading: "Allowance".into(),
            body: format!(
                "Total receivables of {} require an allowance of {} ({:.2}% of AR), leaving net realizable receivables of {}.",
                money(total), money(allowance), allowance_pct, money(net)
            ),
        },
        DocClause { heading: "Adjusting Entry".into(), body: entry_desc },
    ];

    let note = i.note.trim();
    if !note.is_empty() {
        clauses.push(DocClause { heading: "Note".into(), body: note.to_string() });
    }

    AllowanceDoubtful {
        title: "Allowance for Doubtful Accounts".into(),
        total_receivables_usd: total,
        allowance_usd: allowance,
        net_realizable_usd: net,
        allowance_pct_of_ar: allowance_pct,
        adjusting_entry_usd: adjusting,
        rows,
        clauses,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    fn tier(label: &str, bal: f64, pct: f64) -> AgingTier {
        AgingTier { label: label.into(), balance_usd: bal, uncollectible_pct: pct }
    }

    fn base() -> AllowanceInput {
        AllowanceInput {
            company_name: "Acme Supply".into(),
            as_of_date: "2026-06-30".into(),
            tiers: vec![
                tier("Current", 100_000.0, 1.0),
                tier("31-60", 50_000.0, 5.0),
                tier("61-90", 30_000.0, 20.0),
                tier("90+", 20_000.0, 50.0),
            ],
            existing_allowance_usd: 12_000.0,
            note: String::new(),
        }
    }

    #[test]
    fn allowance_and_net_realizable() {
        let d = generate(&base());
        assert!(close(d.total_receivables_usd, 200_000.0));
        // 1,000 + 2,500 + 6,000 + 10,000 = 19,500.
        assert!(close(d.allowance_usd, 19_500.0));
        assert!(close(d.net_realizable_usd, 180_500.0));
        assert!(close(d.allowance_pct_of_ar, 9.75));
    }

    #[test]
    fn adjusting_entry_raises_to_required() {
        let d = generate(&base());
        // 19,500 required − 12,000 existing = 7,500.
        assert!(close(d.adjusting_entry_usd, 7_500.0));
    }

    #[test]
    fn over_reserved_negative_entry() {
        let d = generate(&AllowanceInput { existing_allowance_usd: 25_000.0, ..base() });
        // 19,500 − 25,000 = −5,500 (reduce the allowance).
        assert!(close(d.adjusting_entry_usd, -5_500.0));
        assert!(d.clauses.iter().any(|c| c.body.contains("reduce")));
    }

    #[test]
    fn per_tier_reserves() {
        let d = generate(&base());
        assert_eq!(d.rows.len(), 4);
        assert!(close(d.rows[3].reserve_usd, 10_000.0));
    }

    #[test]
    fn empty_tiers_zero() {
        let d = generate(&AllowanceInput { tiers: vec![], ..base() });
        assert!(close(d.allowance_usd, 0.0));
        assert!(d.clauses.iter().any(|c| c.body.contains("No receivables")));
    }

    #[test]
    fn note_appended_when_present() {
        let d = generate(&AllowanceInput { note: "Per credit committee.".into(), ..base() });
        assert!(d.clauses.iter().any(|c| c.heading == "Note" && c.body.contains("credit committee")));
    }
}
