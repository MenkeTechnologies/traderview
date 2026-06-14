//! Corporate board resolution — records a decision of a company's board of
//! directors. It checks whether a quorum was present (more than half the
//! directors) and tallies the vote (for / against / abstain) to determine
//! whether the resolution passed, then assembles the resolution document.
//! Distinct from the other corporate docs (this is a governance action).
//! Drafting aid, not legal advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct BoardResolutionInput {
    pub company_name: String,
    pub meeting_date: String,
    /// Short label of what is being approved.
    pub resolution_subject: String,
    /// The operative "RESOLVED, that …" text.
    pub resolution_text: String,
    pub total_directors: i64,
    pub directors_present: i64,
    pub votes_for: i64,
    pub votes_against: i64,
    #[serde(default)]
    pub votes_abstain: i64,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DocClause {
    pub heading: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct BoardResolution {
    pub title: String,
    pub quorum_met: bool,
    pub votes_for: i64,
    pub votes_against: i64,
    pub votes_abstain: i64,
    pub passed: bool,
    pub clauses: Vec<DocClause>,
}

pub fn generate(i: &BoardResolutionInput) -> BoardResolution {
    // Quorum = a majority (more than half) of the directors are present.
    let quorum_met = i.directors_present * 2 > i.total_directors;
    // Passes on a simple majority of votes cast (for > against) with a quorum.
    let passed = quorum_met && i.votes_for > i.votes_against;

    let quorum_body = format!(
        "Of {} directors, {} were present, which {} a quorum.",
        i.total_directors,
        i.directors_present,
        if quorum_met { "constitutes" } else { "does NOT constitute" }
    );

    let vote_body = format!(
        "Votes — For: {}, Against: {}, Abstain: {}. The resolution {}.",
        i.votes_for,
        i.votes_against,
        i.votes_abstain,
        if passed { "PASSED" } else { "did NOT pass" }
    );

    let clauses = vec![
        DocClause {
            heading: "Header".into(),
            body: format!(
                "Company: {}\nBoard of Directors meeting date: {}\nSubject: {}",
                i.company_name, i.meeting_date, i.resolution_subject
            ),
        },
        DocClause { heading: "1. Quorum".into(), body: quorum_body },
        DocClause {
            heading: "2. Resolution".into(),
            body: format!(
                "Upon motion duly made and seconded, it was RESOLVED, that {}",
                i.resolution_text
            ),
        },
        DocClause { heading: "3. Vote".into(), body: vote_body },
        DocClause {
            heading: "4. Certification".into(),
            body: format!(
                "The undersigned certifies that the foregoing is a true and correct copy of a resolution duly adopted by the Board of Directors of {} at a meeting held on {}, and that it is in full force and effect. Governed by the laws of the State of {}.",
                i.company_name, i.meeting_date, i.state
            ),
        },
        DocClause {
            heading: "Signature".into(),
            body: format!(
                "Secretary: ____________________  Date: {}\n{}",
                i.meeting_date, i.company_name
            ),
        },
    ];

    BoardResolution {
        title: "Resolution of the Board of Directors".into(),
        quorum_met,
        votes_for: i.votes_for,
        votes_against: i.votes_against,
        votes_abstain: i.votes_abstain,
        passed,
        clauses,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> BoardResolutionInput {
        BoardResolutionInput {
            company_name: "Widgets Inc".into(),
            meeting_date: "2026-07-20".into(),
            resolution_subject: "Approval of stock issuance".into(),
            resolution_text: "the Company is authorized to issue 100,000 shares of common stock to the investor on the agreed terms.".into(),
            total_directors: 5,
            directors_present: 4,
            votes_for: 3,
            votes_against: 1,
            votes_abstain: 0,
            state: "Delaware".into(),
        }
    }

    #[test]
    fn quorum_and_pass() {
        let d = generate(&base());
        assert!(d.quorum_met);
        assert!(d.passed);
    }

    #[test]
    fn no_quorum_fails() {
        // 2 of 5 present → 4 ≤ 5 → no quorum.
        let d = generate(&BoardResolutionInput { directors_present: 2, ..base() });
        assert!(!d.quorum_met);
        assert!(!d.passed);
        let c = d.clauses.iter().find(|c| c.heading == "1. Quorum").unwrap();
        assert!(c.body.contains("does NOT constitute"));
    }

    #[test]
    fn tie_does_not_pass() {
        // For == Against → not a majority.
        let d = generate(&BoardResolutionInput { votes_for: 2, votes_against: 2, ..base() });
        assert!(d.quorum_met);
        assert!(!d.passed);
    }

    #[test]
    fn exact_majority_quorum() {
        // 3 of 5 present → 6 > 5 → quorum.
        assert!(generate(&BoardResolutionInput { directors_present: 3, ..base() }).quorum_met);
        // 3 of 6 present → 6 ≤ 6 → no quorum (need more than half).
        assert!(!generate(&BoardResolutionInput { total_directors: 6, directors_present: 3, ..base() }).quorum_met);
    }

    #[test]
    fn vote_clause_shows_result() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading == "3. Vote").unwrap();
        assert!(c.body.contains("For: 3, Against: 1"));
        assert!(c.body.contains("PASSED"));
    }

    #[test]
    fn resolution_text_in_clause() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading == "2. Resolution").unwrap();
        assert!(c.body.contains("RESOLVED, that"));
        assert!(c.body.contains("100,000 shares"));
    }
}
