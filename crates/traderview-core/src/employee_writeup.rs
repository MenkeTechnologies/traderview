//! Employee write-up / disciplinary action form — documents a workplace
//! incident and the corrective action. The disciplinary level (verbal →
//! written → final → termination) drives the consequence and the next-step
//! escalation language. Distinct from the offer/severance docs. Drafting aid,
//! not legal/HR advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DisciplineLevel {
    Verbal,
    Written,
    Final,
    Termination,
}

impl DisciplineLevel {
    fn label(self) -> &'static str {
        match self {
            DisciplineLevel::Verbal => "Verbal Warning",
            DisciplineLevel::Written => "Written Warning",
            DisciplineLevel::Final => "Final Warning",
            DisciplineLevel::Termination => "Termination",
        }
    }
    fn next_step(self) -> &'static str {
        match self {
            DisciplineLevel::Verbal => "Continued or repeated issues may result in a written warning.",
            DisciplineLevel::Written => "Continued or repeated issues may result in a final warning.",
            DisciplineLevel::Final => "Any further issue may result in termination of employment.",
            DisciplineLevel::Termination => "Employment is terminated effective the incident date stated above.",
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct WriteupInput {
    pub company_name: String,
    pub employee_name: String,
    pub manager_name: String,
    #[serde(default)]
    pub job_title: String,
    pub incident_date: String,
    pub violation_type: String,
    pub description: String,
    pub level: DisciplineLevel,
    #[serde(default)]
    pub prior_warnings: i64,
    #[serde(default)]
    pub corrective_action: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DocClause {
    pub heading: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct EmployeeWriteup {
    pub title: String,
    pub level: String,
    pub next_step: String,
    pub prior_warnings: i64,
    pub clauses: Vec<DocClause>,
}

pub fn generate(i: &WriteupInput) -> EmployeeWriteup {
    let level = i.level.label();
    let next_step = i.level.next_step();

    let corrective = if i.corrective_action.trim().is_empty() {
        "The employee is expected to correct the conduct described above and to comply with company policy going forward.".to_string()
    } else {
        format!("Expected corrective action: {}", i.corrective_action.trim())
    };

    let clauses = vec![
        DocClause {
            heading: "Header".into(),
            body: format!(
                "Employee: {}{}\nManager: {}\nCompany: {}\nIncident date: {}",
                i.employee_name,
                if i.job_title.trim().is_empty() { String::new() } else { format!(" ({})", i.job_title.trim()) },
                i.manager_name,
                i.company_name,
                i.incident_date
            ),
        },
        DocClause {
            heading: "1. Violation".into(),
            body: format!("Type: {}\nDescription: {}", i.violation_type, i.description),
        },
        DocClause {
            heading: "2. Disciplinary Level".into(),
            body: format!(
                "This is a {}. Prior documented warnings on file: {}.",
                level, i.prior_warnings
            ),
        },
        DocClause { heading: "3. Consequence / Next Step".into(), body: next_step.to_string() },
        DocClause { heading: "4. Corrective Action".into(), body: corrective },
        DocClause {
            heading: "5. Acknowledgment".into(),
            body: "The employee's signature acknowledges receipt of this notice and does not necessarily indicate agreement. The employee may attach a written response.".into(),
        },
        DocClause {
            heading: "Signatures".into(),
            body: format!(
                "Employee: ____________________  Date: __________\n{}\n\nManager: ____________________  Date: __________\n{}",
                i.employee_name, i.manager_name
            ),
        },
    ];

    EmployeeWriteup {
        title: "Employee Disciplinary Action Form".into(),
        level: level.to_string(),
        next_step: next_step.to_string(),
        prior_warnings: i.prior_warnings,
        clauses,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> WriteupInput {
        WriteupInput {
            company_name: "Acme Inc".into(),
            employee_name: "Sam Worker".into(),
            manager_name: "Pat Boss".into(),
            job_title: "Technician".into(),
            incident_date: "2026-06-10".into(),
            violation_type: "Attendance".into(),
            description: "Three unexcused absences in two weeks.".into(),
            level: DisciplineLevel::Written,
            prior_warnings: 1,
            corrective_action: "Maintain regular attendance; notify manager of absences in advance.".into(),
        }
    }

    #[test]
    fn written_level_and_next_step() {
        let d = generate(&base());
        assert_eq!(d.level, "Written Warning");
        assert!(d.next_step.contains("final warning"));
    }

    #[test]
    fn verbal_escalates_to_written() {
        let d = generate(&WriteupInput { level: DisciplineLevel::Verbal, ..base() });
        assert_eq!(d.level, "Verbal Warning");
        assert!(d.next_step.contains("written warning"));
    }

    #[test]
    fn final_escalates_to_termination() {
        let d = generate(&WriteupInput { level: DisciplineLevel::Final, ..base() });
        assert!(d.next_step.contains("termination"));
    }

    #[test]
    fn termination_level() {
        let d = generate(&WriteupInput { level: DisciplineLevel::Termination, ..base() });
        assert_eq!(d.level, "Termination");
        assert!(d.next_step.contains("terminated effective"));
    }

    #[test]
    fn violation_and_prior_warnings_in_clauses() {
        let d = generate(&base());
        assert!(d.clauses.iter().find(|c| c.heading == "1. Violation").unwrap().body.contains("Attendance"));
        let c = d.clauses.iter().find(|c| c.heading.contains("Disciplinary Level")).unwrap();
        assert!(c.body.contains("Prior documented warnings on file: 1"));
    }

    #[test]
    fn default_corrective_action_when_blank() {
        let c = generate(&WriteupInput { corrective_action: String::new(), ..base() })
            .clauses.into_iter().find(|c| c.heading == "4. Corrective Action").unwrap();
        assert!(c.body.contains("expected to correct"));
    }

    #[test]
    fn acknowledgment_not_agreement() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading == "5. Acknowledgment").unwrap();
        assert!(c.body.contains("does not necessarily indicate agreement"));
    }
}
