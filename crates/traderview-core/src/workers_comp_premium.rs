//! Workers' compensation premium — the standard premium estimate for a business's
//! workers' comp policy. Each class code carries a rate per $100 of payroll; the
//! manual premium is payroll ÷ 100 × rate, summed across class codes, and the
//! modified premium applies the employer's experience modifier (a mod below 1.0 is
//! a credit for good loss history; above 1.0 a surcharge). No existing generator
//! computes a payroll-based class-rate premium. Drafting aid, not insurance/legal advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct ClassCode {
    pub code: String,
    #[serde(default)]
    pub description: String,
    /// Annual payroll assigned to this class code.
    pub payroll_usd: f64,
    /// Rate per $100 of payroll for this class code.
    pub rate_per_100_usd: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkersCompInput {
    pub insurer_name: String,
    pub employer_name: String,
    pub policy_period: String,
    pub classes: Vec<ClassCode>,
    /// Experience modifier (1.0 = neutral; <1 credit; >1 surcharge).
    #[serde(default = "default_emod")]
    pub experience_mod: f64,
    pub date: String,
    #[serde(default)]
    pub note: String,
}

fn default_emod() -> f64 {
    1.0
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ClassRow {
    pub code: String,
    pub description: String,
    pub payroll_usd: f64,
    pub rate_per_100_usd: f64,
    /// payroll ÷ 100 × rate.
    pub manual_premium_usd: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DocClause {
    pub heading: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct WorkersCompPremium {
    pub title: String,
    pub total_payroll_usd: f64,
    /// Sum of each class code's manual premium.
    pub manual_premium_usd: f64,
    pub experience_mod: f64,
    /// Manual premium × experience modifier.
    pub modified_premium_usd: f64,
    pub rows: Vec<ClassRow>,
    pub clauses: Vec<DocClause>,
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

pub fn generate(i: &WorkersCompInput) -> WorkersCompPremium {
    let rows: Vec<ClassRow> = i
        .classes
        .iter()
        .map(|c| ClassRow {
            code: c.code.clone(),
            description: c.description.clone(),
            payroll_usd: cents(c.payroll_usd),
            rate_per_100_usd: c.rate_per_100_usd,
            manual_premium_usd: cents(c.payroll_usd / 100.0 * c.rate_per_100_usd),
        })
        .collect();

    let total_payroll = cents(i.classes.iter().map(|c| c.payroll_usd).sum());
    let manual = cents(rows.iter().map(|r| r.manual_premium_usd).sum());
    let modified = cents(manual * i.experience_mod);

    let detail = if rows.is_empty() {
        "No class codes listed.".to_string()
    } else {
        rows.iter()
            .map(|r| {
                let desc = if r.description.trim().is_empty() {
                    String::new()
                } else {
                    format!(" ({})", r.description)
                };
                format!(
                    "  • Class {}{}: payroll {} × {:.4}/100 = {}",
                    r.code, desc, money(r.payroll_usd), r.rate_per_100_usd, money(r.manual_premium_usd)
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    let mod_desc = if i.experience_mod < 1.0 {
        "a credit"
    } else if i.experience_mod > 1.0 {
        "a surcharge"
    } else {
        "neutral"
    };

    let mut clauses = vec![
        DocClause {
            heading: "Policy".into(),
            body: format!(
                "Insurer: {}\nEmployer: {}\nPolicy period: {}\nDate: {}",
                i.insurer_name, i.employer_name, i.policy_period, i.date
            ),
        },
        DocClause { heading: "Class Codes".into(), body: detail },
        DocClause {
            heading: "Manual Premium".into(),
            body: format!(
                "Total payroll {} produces a manual premium of {} across the class codes above.",
                money(total_payroll), money(manual)
            ),
        },
        DocClause {
            heading: "Experience Modification".into(),
            body: format!(
                "The experience modifier of {:.3} ({}) applies, for a modified premium of {}.",
                i.experience_mod, mod_desc, money(modified)
            ),
        },
    ];

    let note = i.note.trim();
    if !note.is_empty() {
        clauses.push(DocClause { heading: "Note".into(), body: note.to_string() });
    }

    WorkersCompPremium {
        title: "Workers' Compensation Premium Estimate".into(),
        total_payroll_usd: total_payroll,
        manual_premium_usd: manual,
        experience_mod: i.experience_mod,
        modified_premium_usd: modified,
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

    fn cls(code: &str, payroll: f64, rate: f64) -> ClassCode {
        ClassCode { code: code.into(), description: String::new(), payroll_usd: payroll, rate_per_100_usd: rate }
    }

    fn base() -> WorkersCompInput {
        WorkersCompInput {
            insurer_name: "State Fund".into(),
            employer_name: "Acme Co".into(),
            policy_period: "2026".into(),
            classes: vec![cls("8810", 300_000.0, 2.00), cls("5403", 200_000.0, 5.00)],
            experience_mod: 0.90,
            date: "2026-07-01".into(),
            note: String::new(),
        }
    }

    #[test]
    fn multi_class_manual_and_modified() {
        let d = generate(&base());
        assert!(close(d.total_payroll_usd, 500_000.0));
        // 300k/100×2 = 6,000; 200k/100×5 = 10,000 → 16,000.
        assert!(close(d.manual_premium_usd, 16_000.0));
        // 16,000 × 0.90 = 14,400.
        assert!(close(d.modified_premium_usd, 14_400.0));
    }

    #[test]
    fn neutral_mod_equals_manual() {
        let d = generate(&WorkersCompInput {
            classes: vec![cls("0000", 500_000.0, 3.50)],
            experience_mod: 1.0,
            ..base()
        });
        assert!(close(d.manual_premium_usd, 17_500.0));
        assert!(close(d.modified_premium_usd, 17_500.0));
    }

    #[test]
    fn surcharge_mod_increases_premium() {
        let d = generate(&WorkersCompInput {
            classes: vec![cls("0000", 500_000.0, 3.50)],
            experience_mod: 1.25,
            ..base()
        });
        assert!(close(d.modified_premium_usd, 21_875.0));
    }

    #[test]
    fn per_class_premium_rows() {
        let d = generate(&base());
        assert_eq!(d.rows.len(), 2);
        assert!(close(d.rows[0].manual_premium_usd, 6_000.0));
        assert!(close(d.rows[1].manual_premium_usd, 10_000.0));
    }

    #[test]
    fn empty_classes_zero() {
        let d = generate(&WorkersCompInput { classes: vec![], ..base() });
        assert!(close(d.manual_premium_usd, 0.0));
        assert!(d.clauses.iter().any(|c| c.body.contains("No class codes")));
    }

    #[test]
    fn note_appended_when_present() {
        let d = generate(&WorkersCompInput { note: "Audit at period end.".into(), ..base() });
        assert!(d.clauses.iter().any(|c| c.heading == "Note" && c.body.contains("Audit")));
    }
}
