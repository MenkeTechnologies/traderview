//! Roommate agreement — splits the rent and security deposit among co-tenants
//! and records the house rules. Each roommate carries a weight (equal weights =
//! even split; unequal weights handle different room sizes); the rent and
//! deposit are divided in proportion. Drafting aid, not legal advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Roommate {
    pub name: String,
    /// Relative share of rent/deposit. Defaults to 1 (equal split).
    #[serde(default = "default_weight")]
    pub weight: f64,
}

fn default_weight() -> f64 {
    1.0
}

#[derive(Debug, Clone, Deserialize)]
pub struct RoommateInput {
    pub premises_address: String,
    pub total_monthly_rent_usd: f64,
    #[serde(default)]
    pub total_deposit_usd: f64,
    pub lease_start_date: String,
    #[serde(default)]
    pub roommates: Vec<Roommate>,
    #[serde(default)]
    pub state: String,
    #[serde(default)]
    pub statute_citation: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct RoommateShare {
    pub name: String,
    pub rent_share_usd: f64,
    pub deposit_share_usd: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DocClause {
    pub heading: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct RoommateAgreement {
    pub title: String,
    pub roommate_count: usize,
    pub total_monthly_rent_usd: f64,
    pub total_deposit_usd: f64,
    pub shares: Vec<RoommateShare>,
    pub statutory_citation: String,
    pub clauses: Vec<DocClause>,
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

pub fn generate(i: &RoommateInput) -> RoommateAgreement {
    // Weights ≤ 0 are treated as the default so a blank weight never zeroes a
    // roommate out; the total weight drives the proportional split.
    let weights: Vec<f64> = i
        .roommates
        .iter()
        .map(|r| if r.weight > 0.0 { r.weight } else { 1.0 })
        .collect();
    let total_weight: f64 = weights.iter().sum();

    let shares: Vec<RoommateShare> = i
        .roommates
        .iter()
        .zip(weights.iter())
        .map(|(r, &w)| {
            let frac = if total_weight > 0.0 { w / total_weight } else { 0.0 };
            RoommateShare {
                name: r.name.clone(),
                rent_share_usd: cents(i.total_monthly_rent_usd * frac),
                deposit_share_usd: cents(i.total_deposit_usd * frac),
            }
        })
        .collect();

    let citation = i.statute_citation.trim();

    let rent_lines = if shares.is_empty() {
        "No roommates listed.".to_string()
    } else {
        shares
            .iter()
            .map(|s| format!("  • {}: {} / month", s.name, money(s.rent_share_usd)))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let deposit_lines = shares
        .iter()
        .map(|s| format!("  • {}: {}", s.name, money(s.deposit_share_usd)))
        .collect::<Vec<_>>()
        .join("\n");

    let mut clauses = vec![
        DocClause {
            heading: "Premises and Term".into(),
            body: format!(
                "The undersigned roommates share the premises located at {}, with the tenancy beginning {}. Total monthly rent: {}. Total security deposit: {}.",
                i.premises_address,
                i.lease_start_date,
                money(i.total_monthly_rent_usd),
                money(i.total_deposit_usd)
            ),
        },
        DocClause {
            heading: "1. Rent Shares".into(),
            body: format!("Each roommate's share of the monthly rent:\n{}", rent_lines),
        },
        DocClause {
            heading: "2. Security Deposit Shares".into(),
            body: format!("Each roommate's share of the security deposit:\n{}", deposit_lines),
        },
        DocClause {
            heading: "3. Joint and Several Liability".into(),
            body: "The roommates understand that under most leases each tenant is jointly and severally liable to the landlord for the full rent; this agreement allocates shares among the roommates but does not limit the landlord's right to collect the full amount from any one of them.".into(),
        },
        DocClause {
            heading: "4. Shared Expenses and House Rules".into(),
            body: "The roommates shall split shared utilities and household expenses in the same proportion as rent unless otherwise agreed in writing, shall give reasonable notice before moving out, shall keep common areas clean, and shall respect quiet hours and guest policies as mutually agreed.".into(),
        },
    ];

    if !citation.is_empty() {
        clauses.push(DocClause {
            heading: "5. Governing Law".into(),
            body: format!("This agreement is governed by the law of the State of {} ({}).", i.state, citation),
        });
    } else if !i.state.trim().is_empty() {
        clauses.push(DocClause {
            heading: "5. Governing Law".into(),
            body: format!("This agreement is governed by the law of the State of {}.", i.state.trim()),
        });
    }

    let sig_lines = shares
        .iter()
        .map(|s| format!("{}: ____________________  Date: __________", s.name))
        .collect::<Vec<_>>()
        .join("\n\n");
    clauses.push(DocClause { heading: "Signatures".into(), body: sig_lines });

    RoommateAgreement {
        title: "Roommate Agreement".into(),
        roommate_count: shares.len(),
        total_monthly_rent_usd: i.total_monthly_rent_usd,
        total_deposit_usd: i.total_deposit_usd,
        shares,
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

    fn rm(name: &str, weight: f64) -> Roommate {
        Roommate { name: name.into(), weight }
    }

    fn base() -> RoommateInput {
        RoommateInput {
            premises_address: "42 Shared St".into(),
            total_monthly_rent_usd: 2100.0,
            total_deposit_usd: 900.0,
            lease_start_date: "2026-08-01".into(),
            roommates: vec![rm("Alice", 1.0), rm("Bob", 1.0), rm("Cara", 1.0)],
            state: "Oregon".into(),
            statute_citation: String::new(),
        }
    }

    #[test]
    fn equal_split() {
        let d = generate(&base());
        assert_eq!(d.roommate_count, 3);
        for s in &d.shares {
            assert!(close(s.rent_share_usd, 700.0));
            assert!(close(s.deposit_share_usd, 300.0));
        }
    }

    #[test]
    fn weighted_split() {
        let d = generate(&RoommateInput {
            roommates: vec![rm("Alice", 2.0), rm("Bob", 1.0), rm("Cara", 1.0)],
            ..base()
        });
        // Total weight 4 → Alice 1/2, others 1/4 of 2,100.
        assert!(close(d.shares[0].rent_share_usd, 1050.0));
        assert!(close(d.shares[1].rent_share_usd, 525.0));
        assert!(close(d.shares[2].rent_share_usd, 525.0));
    }

    #[test]
    fn shares_listed_in_clause() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading == "1. Rent Shares").unwrap();
        assert!(c.body.contains("Alice: $700.00 / month"));
    }

    #[test]
    fn joint_and_several_clause_present() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading.contains("Joint and Several")).unwrap();
        assert!(c.body.contains("jointly and severally liable"));
    }

    #[test]
    fn zero_weight_treated_as_equal() {
        let d = generate(&RoommateInput {
            roommates: vec![rm("Alice", 0.0), rm("Bob", 0.0)],
            ..base()
        });
        // Both default to weight 1 → 1,050 each.
        assert!(close(d.shares[0].rent_share_usd, 1050.0));
        assert!(close(d.shares[1].rent_share_usd, 1050.0));
    }

    #[test]
    fn statute_citation_echoed() {
        let d = generate(&RoommateInput { statute_citation: "Or. Rev. Stat. § 90".into(), ..base() });
        assert_eq!(d.statutory_citation, "Or. Rev. Stat. § 90");
        assert!(d.clauses.iter().any(|c| c.body.contains("Or. Rev. Stat. § 90")));
    }
}
