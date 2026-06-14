//! 1099-NEC contractor payment summary — a payer's year-end summary of nonemployee
//! compensation paid to an independent contractor. It totals the year's payments,
//! applies the $600 information-reporting threshold (a 1099-NEC is required at or
//! above it), and computes backup withholding (24% when the payer must withhold,
//! e.g. a missing or incorrect TIN) and the net paid. Distinct from the contractor
//! agreement (the contract) and the pay stub (W-2 wages). Drafting aid, not tax advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Payment {
    pub date: String,
    #[serde(default)]
    pub description: String,
    pub amount_usd: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Contractor1099Input {
    pub payer_name: String,
    pub contractor_name: String,
    pub tax_year: String,
    pub payments: Vec<Payment>,
    /// True when the payer must apply backup withholding.
    #[serde(default)]
    pub subject_to_backup_withholding: bool,
    /// Backup-withholding rate, percent (IRS default is 24%).
    #[serde(default = "default_backup_rate")]
    pub backup_rate_pct: f64,
    /// Reporting threshold for a 1099-NEC (IRS default is $600).
    #[serde(default = "default_threshold")]
    pub reporting_threshold_usd: f64,
    pub date: String,
    #[serde(default)]
    pub note: String,
}

fn default_backup_rate() -> f64 {
    24.0
}

fn default_threshold() -> f64 {
    600.0
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PaymentRow {
    pub date: String,
    pub description: String,
    pub amount_usd: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DocClause {
    pub heading: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Contractor1099 {
    pub title: String,
    pub payment_count: usize,
    pub total_paid_usd: f64,
    /// True when total ≥ the reporting threshold (a 1099-NEC must be filed).
    pub reportable: bool,
    pub backup_withholding_usd: f64,
    /// Total − backup withholding.
    pub net_paid_usd: f64,
    pub rows: Vec<PaymentRow>,
    pub statutory_citation: String,
    pub clauses: Vec<DocClause>,
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

pub fn generate(i: &Contractor1099Input) -> Contractor1099 {
    let rows: Vec<PaymentRow> = i
        .payments
        .iter()
        .map(|p| PaymentRow {
            date: p.date.clone(),
            description: p.description.clone(),
            amount_usd: cents(p.amount_usd),
        })
        .collect();

    let total = cents(i.payments.iter().map(|p| p.amount_usd).sum());
    let reportable = total >= i.reporting_threshold_usd;
    let backup = if i.subject_to_backup_withholding {
        cents(total * i.backup_rate_pct / 100.0)
    } else {
        0.0
    };
    let net = cents(total - backup);

    let report_line = if reportable {
        format!(
            "Total compensation of {} is at or above the {} reporting threshold, so a Form 1099-NEC must be filed for the {} tax year.",
            money(total),
            money(i.reporting_threshold_usd),
            i.tax_year
        )
    } else {
        format!(
            "Total compensation of {} is below the {} reporting threshold; a Form 1099-NEC is generally not required for the {} tax year, though records should be retained.",
            money(total),
            money(i.reporting_threshold_usd),
            i.tax_year
        )
    };

    let backup_line = if i.subject_to_backup_withholding {
        format!(
            " Backup withholding of {} ({:.0}% of total) applies and is remitted to the IRS, leaving {} paid to the contractor.",
            money(backup),
            i.backup_rate_pct,
            money(net)
        )
    } else {
        " No backup withholding applies.".to_string()
    };

    let detail = if rows.is_empty() {
        "No payments recorded.".to_string()
    } else {
        rows.iter()
            .map(|r| {
                let desc = if r.description.trim().is_empty() {
                    String::new()
                } else {
                    format!(" — {}", r.description)
                };
                format!("  • {}: {}{}", r.date, money(r.amount_usd), desc)
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    let mut clauses = vec![
        DocClause {
            heading: "Summary".into(),
            body: format!(
                "Payer: {}\nContractor: {}\nTax year: {}\nStatement date: {}",
                i.payer_name, i.contractor_name, i.tax_year, i.date
            ),
        },
        DocClause { heading: "Payments".into(), body: detail },
        DocClause {
            heading: "Total Compensation".into(),
            body: format!("Total nonemployee compensation paid: {}.{}", money(total), report_line),
        },
        DocClause {
            heading: "Backup Withholding".into(),
            body: format!("Backup withholding status.{}", backup_line),
        },
    ];

    let note = i.note.trim();
    if !note.is_empty() {
        clauses.push(DocClause { heading: "Note".into(), body: note.to_string() });
    }

    Contractor1099 {
        title: "1099-NEC Contractor Payment Summary".into(),
        payment_count: rows.len(),
        total_paid_usd: total,
        reportable,
        backup_withholding_usd: backup,
        net_paid_usd: net,
        rows,
        statutory_citation: String::new(),
        clauses,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    fn pay(date: &str, amt: f64) -> Payment {
        Payment { date: date.into(), description: String::new(), amount_usd: amt }
    }

    fn base() -> Contractor1099Input {
        Contractor1099Input {
            payer_name: "Acme Co".into(),
            contractor_name: "Jane Freelancer".into(),
            tax_year: "2025".into(),
            payments: vec![pay("2025-01-15", 5_000.0), pay("2025-04-10", 5_000.0), pay("2025-08-01", 5_000.0)],
            subject_to_backup_withholding: false,
            backup_rate_pct: 24.0,
            reporting_threshold_usd: 600.0,
            date: "2026-01-31".into(),
            note: String::new(),
        }
    }

    #[test]
    fn totals_and_reportable() {
        let d = generate(&base());
        assert_eq!(d.payment_count, 3);
        assert!(close(d.total_paid_usd, 15_000.0));
        assert!(d.reportable);
        assert!(close(d.backup_withholding_usd, 0.0));
        assert!(close(d.net_paid_usd, 15_000.0));
    }

    #[test]
    fn backup_withholding_applies() {
        let d = generate(&Contractor1099Input { subject_to_backup_withholding: true, ..base() });
        assert!(close(d.backup_withholding_usd, 3_600.0));
        assert!(close(d.net_paid_usd, 11_400.0));
    }

    #[test]
    fn below_threshold_not_reportable() {
        let d = generate(&Contractor1099Input {
            payments: vec![pay("2025-03-01", 400.0)],
            subject_to_backup_withholding: true,
            ..base()
        });
        assert!(!d.reportable);
        // Backup withholding still applies when the payer must withhold.
        assert!(close(d.backup_withholding_usd, 96.0));
        assert!(close(d.net_paid_usd, 304.0));
    }

    #[test]
    fn threshold_is_inclusive() {
        let d = generate(&Contractor1099Input { payments: vec![pay("2025-06-01", 600.0)], ..base() });
        assert!(d.reportable);
    }

    #[test]
    fn empty_payments_zero_total() {
        let d = generate(&Contractor1099Input { payments: vec![], ..base() });
        assert!(close(d.total_paid_usd, 0.0));
        assert!(!d.reportable);
        assert!(d.clauses.iter().any(|c| c.body.contains("No payments recorded")));
    }

    #[test]
    fn note_appended_when_present() {
        let d = generate(&Contractor1099Input { note: "TIN on file (W-9).".into(), ..base() });
        assert!(d.clauses.iter().any(|c| c.heading == "Note" && c.body.contains("W-9")));
    }
}
