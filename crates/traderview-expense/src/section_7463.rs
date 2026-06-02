//! IRC § 7463 — Disputes involving $50,000 or less (Tax
//! Court small case procedure / "S case" election). Trader-
//! relevant for traders with smaller audit deficiencies
//! seeking faster + cheaper resolution at Tax Court without
//! formal rules of evidence and without appellate review.
//! Companion to `section_6213` (Tax Court petition deadline
//! — already shipped), `section_6212` (SNOD — already
//! shipped), `section_7459` (Tax Court decisions), Tax Court
//! Rules 170-175 (small case procedural rules).
//!
//! **§ 7463(a) Designation** — petitions filed with Tax
//! Court for redetermination of deficiency where neither
//! the amount of the deficiency placed in dispute nor the
//! amount of any claimed overpayment exceeds **$50,000 for
//! any one taxable year** (income tax). Also applies to:
//! - **Estate tax** — $50,000 limit per estate;
//! - **Gift tax** — $50,000 limit per calendar year;
//! - **Excise taxes** — $50,000 limit per taxable period
//!   or taxable event.
//!
//! Proceedings under § 7463 are conducted at the **option
//! of the taxpayer**, concurred in by the Tax Court or a
//! division thereof, BEFORE THE HEARING of the case.
//!
//! **§ 7463(b) Finality of decisions** — a decision entered
//! in any case conducted under § 7463 **SHALL NOT BE
//! REVIEWED IN ANY OTHER COURT** and **SHALL NOT BE TREATED
//! AS A PRECEDENT** for any other case. Critical procedural
//! tradeoff — faster + cheaper + informal but NO APPEAL +
//! NO PRECEDENTIAL VALUE.
//!
//! **§ 7463(c) Limit on review** — taxpayer or the
//! Secretary may concur to discontinue § 7463 designation
//! BEFORE the case decision becomes final.
//!
//! **§ 7463(d) Procedure** — proceedings shall be conducted
//! in accordance with rules of evidence, practice, and
//! procedure as the Tax Court may prescribe. Tax Court
//! Rules 170-175 govern small case procedure. Cases
//! "conducted as informally as possible consistent with
//! orderly procedure, and any evidence deemed by the Court
//! to have probative value shall be admissible".
//!
//! **§ 7463(f) Additional cases** — Tax Court may also use
//! S case procedure for:
//! 1. Certain collection-determination proceedings under
//!    § 6320 (CDP-lien) or § 6330 (CDP-levy) where amount in
//!    dispute is $50,000 or less;
//! 2. § 6015 innocent-spouse-relief proceedings where
//!    amount in dispute is $50,000 or less;
//! 3. § 7436 worker-classification (1099 vs W-2) proceedings
//!    where amount in dispute is $50,000 or less.
//!
//! **Procedural tradeoff**:
//! - **S case advantages**: faster (typically 6-12 months
//!   vs 18-36 months for regular case); cheaper (no
//!   transcript required); informal (relaxed rules of
//!   evidence); pro se friendly.
//! - **S case disadvantages**: NO APPEAL to Circuit Court +
//!   Supreme Court (decision is FINAL); NO PRECEDENTIAL
//!   VALUE; informal nature may favor IRS-experienced
//!   counsel.
//!
//! Citations: 26 USC § 7463(a)-(f); Tax Court Rules 170-
//! 175; § 6212 SNOD; § 6213 Tax Court petition; § 7459 Tax
//! Court decisions; § 6320 CDP-lien; § 6330 CDP-levy;
//! § 6015 innocent spouse; § 7436 worker classification;
//! IRM 35.1.3 (Tax Court Procedures).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CaseType {
    /// § 7463(a)(1) — income tax deficiency redetermination.
    IncomeTaxDeficiency,
    /// § 7463(a)(2) — estate tax.
    EstateTax,
    /// § 7463(a)(3) — gift tax per calendar year.
    GiftTax,
    /// § 7463(a)(4) — excise taxes per period or event.
    ExciseTax,
    /// § 7463(f)(1) — § 6320 CDP-lien collection
    /// determination.
    Section6320CdpLien,
    /// § 7463(f)(1) — § 6330 CDP-levy collection
    /// determination.
    Section6330CdpLevy,
    /// § 7463(f)(2) — § 6015 innocent-spouse relief.
    Section6015InnocentSpouse,
    /// § 7463(f)(3) — § 7436 worker classification (1099 vs
    /// W-2).
    Section7436WorkerClassification,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section7463Input {
    pub case_type: CaseType,
    /// Amount in dispute in cents (deficiency or claimed
    /// overpayment).
    pub amount_in_dispute_cents: u64,
    /// Whether taxpayer elected small-case procedure.
    pub taxpayer_s_case_election: bool,
    /// Whether Tax Court concurred in designation BEFORE the
    /// hearing.
    pub tax_court_concurred_before_hearing: bool,
    /// Whether taxpayer or Secretary moved to discontinue §
    /// 7463 designation before final decision.
    pub motion_to_discontinue_designation: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section7463Result {
    pub s_case_designation_available: bool,
    pub within_50000_limit: bool,
    pub election_and_concurrence_satisfied: bool,
    pub decision_appealable: bool,
    pub decision_has_precedential_value: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section7463Input) -> Section7463Result {
    let mut failure_reasons: Vec<String> = Vec::new();

    const FIFTY_K_LIMIT_CENTS: u64 = 5_000_000;
    let within_limit = input.amount_in_dispute_cents <= FIFTY_K_LIMIT_CENTS;

    if !within_limit {
        failure_reasons.push(
            "26 USC § 7463(a) — small case procedure available only when amount in dispute does NOT exceed $50,000 per taxable year (income) / estate / calendar year (gift) / period or event (excise)".to_string(),
        );
    }

    let election_concurred = input.taxpayer_s_case_election
        && input.tax_court_concurred_before_hearing;

    if input.taxpayer_s_case_election && !input.tax_court_concurred_before_hearing {
        failure_reasons.push(
            "26 USC § 7463(a) — Tax Court concurrence required BEFORE the hearing of the case".to_string(),
        );
    }

    let s_case_available = within_limit && election_concurred;

    let designation_active = s_case_available && !input.motion_to_discontinue_designation;

    let appealable = !designation_active;
    let precedential = !designation_active;

    let notes: Vec<String> = vec![
        "26 USC § 7463(a) — small case procedure available when amount in dispute does NOT exceed $50,000 per taxable year (income) / estate / calendar year (gift) / period or event (excise)".to_string(),
        "26 USC § 7463(a) — proceedings conducted at option of taxpayer, concurred in by Tax Court or division thereof, BEFORE the hearing of the case".to_string(),
        "26 USC § 7463(b) — decision under § 7463 SHALL NOT BE REVIEWED IN ANY OTHER COURT and SHALL NOT BE TREATED AS A PRECEDENT for any other case".to_string(),
        "26 USC § 7463(c) — taxpayer or Secretary may concur to discontinue § 7463 designation BEFORE case decision becomes final".to_string(),
        "26 USC § 7463(d) — proceedings conducted under Tax Court Rules 170-175; as informally as possible consistent with orderly procedure; any evidence deemed by Court to have probative value shall be admissible".to_string(),
        "26 USC § 7463(f) — small case procedure also available for § 6320 (CDP-lien), § 6330 (CDP-levy), § 6015 (innocent spouse), § 7436 (worker classification) proceedings under $50,000".to_string(),
        "S case procedural tradeoff: FASTER (6-12 months vs 18-36 regular) + CHEAPER (no transcript) + INFORMAL (relaxed rules of evidence) + PRO SE FRIENDLY; BUT NO APPEAL to Circuit Court or Supreme Court (decision FINAL) and NO PRECEDENTIAL VALUE".to_string(),
        "Cross-references: § 6212 SNOD provides initial Tax Court jurisdiction; § 6213 90-day petition window for Tax Court; § 7459 Tax Court decisions framework; § 6320 + § 6330 CDP collection determinations; § 6015 innocent spouse; § 7436 worker classification".to_string(),
        "Tax Court Rules 170-175 govern small case procedure; IRM 35.1.3 internal IRS procedural guidance on Tax Court Procedures".to_string(),
    ];

    Section7463Result {
        s_case_designation_available: s_case_available,
        within_50000_limit: within_limit,
        election_and_concurrence_satisfied: election_concurred,
        decision_appealable: appealable,
        decision_has_precedential_value: precedential,
        failure_reasons,
        citation: "26 USC § 7463(a)-(f); Tax Court Rules 170-175; § 6212; § 6213; § 7459; § 6320; § 6330; § 6015; § 7436; IRM 35.1.3",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_base() -> Section7463Input {
        Section7463Input {
            case_type: CaseType::IncomeTaxDeficiency,
            amount_in_dispute_cents: 4_000_000,
            taxpayer_s_case_election: true,
            tax_court_concurred_before_hearing: true,
            motion_to_discontinue_designation: false,
        }
    }

    #[test]
    fn income_tax_40k_within_limit_designation_available() {
        let r = check(&valid_base());
        assert!(r.s_case_designation_available);
        assert!(r.within_50000_limit);
        assert!(!r.decision_appealable);
        assert!(!r.decision_has_precedential_value);
    }

    #[test]
    fn at_50000_boundary_compliant() {
        let mut i = valid_base();
        i.amount_in_dispute_cents = 5_000_000;
        let r = check(&i);
        assert!(r.within_50000_limit);
        assert!(r.s_case_designation_available);
    }

    #[test]
    fn fifty_thousand_one_dollars_exceeds_limit() {
        let mut i = valid_base();
        i.amount_in_dispute_cents = 5_000_001;
        let r = check(&i);
        assert!(!r.within_50000_limit);
        assert!(!r.s_case_designation_available);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 7463(a)") && f.contains("$50,000")));
    }

    #[test]
    fn estate_tax_in_scope() {
        let mut i = valid_base();
        i.case_type = CaseType::EstateTax;
        let r = check(&i);
        assert!(r.s_case_designation_available);
    }

    #[test]
    fn gift_tax_in_scope() {
        let mut i = valid_base();
        i.case_type = CaseType::GiftTax;
        let r = check(&i);
        assert!(r.s_case_designation_available);
    }

    #[test]
    fn excise_tax_in_scope() {
        let mut i = valid_base();
        i.case_type = CaseType::ExciseTax;
        let r = check(&i);
        assert!(r.s_case_designation_available);
    }

    #[test]
    fn section_6320_cdp_lien_in_scope() {
        let mut i = valid_base();
        i.case_type = CaseType::Section6320CdpLien;
        let r = check(&i);
        assert!(r.s_case_designation_available);
    }

    #[test]
    fn section_6330_cdp_levy_in_scope() {
        let mut i = valid_base();
        i.case_type = CaseType::Section6330CdpLevy;
        let r = check(&i);
        assert!(r.s_case_designation_available);
    }

    #[test]
    fn section_6015_innocent_spouse_in_scope() {
        let mut i = valid_base();
        i.case_type = CaseType::Section6015InnocentSpouse;
        let r = check(&i);
        assert!(r.s_case_designation_available);
    }

    #[test]
    fn section_7436_worker_classification_in_scope() {
        let mut i = valid_base();
        i.case_type = CaseType::Section7436WorkerClassification;
        let r = check(&i);
        assert!(r.s_case_designation_available);
    }

    #[test]
    fn no_taxpayer_election_no_designation() {
        let mut i = valid_base();
        i.taxpayer_s_case_election = false;
        let r = check(&i);
        assert!(!r.s_case_designation_available);
    }

    #[test]
    fn no_tax_court_concurrence_no_designation() {
        let mut i = valid_base();
        i.tax_court_concurred_before_hearing = false;
        let r = check(&i);
        assert!(!r.s_case_designation_available);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("BEFORE the hearing")));
    }

    #[test]
    fn discontinued_designation_appealable() {
        let mut i = valid_base();
        i.motion_to_discontinue_designation = true;
        let r = check(&i);
        assert!(r.decision_appealable);
        assert!(r.decision_has_precedential_value);
    }

    #[test]
    fn s_case_active_decision_final_no_appeal() {
        let r = check(&valid_base());
        assert!(!r.decision_appealable);
        assert!(!r.decision_has_precedential_value);
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&valid_base());
        assert!(r.citation.contains("§ 7463(a)-(f)"));
        assert!(r.citation.contains("Tax Court Rules 170-175"));
        assert!(r.citation.contains("§ 6212"));
        assert!(r.citation.contains("§ 6213"));
        assert!(r.citation.contains("§ 7459"));
        assert!(r.citation.contains("§ 6320"));
        assert!(r.citation.contains("§ 6330"));
        assert!(r.citation.contains("§ 6015"));
        assert!(r.citation.contains("§ 7436"));
        assert!(r.citation.contains("IRM 35.1.3"));
    }

    #[test]
    fn note_pins_subsection_a_50k_limit() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 7463(a)")
            && n.contains("$50,000")
            && n.contains("estate")
            && n.contains("gift")
            && n.contains("excise")));
    }

    #[test]
    fn note_pins_subsection_a_concurrence_before_hearing() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 7463(a)")
            && n.contains("option of taxpayer")
            && n.contains("BEFORE the hearing")));
    }

    #[test]
    fn note_pins_subsection_b_no_review_no_precedent() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 7463(b)")
            && n.contains("SHALL NOT BE REVIEWED")
            && n.contains("SHALL NOT BE TREATED AS A PRECEDENT")));
    }

    #[test]
    fn note_pins_subsection_c_discontinue_motion() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 7463(c)")
            && n.contains("discontinue")));
    }

    #[test]
    fn note_pins_subsection_d_informal_rules() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 7463(d)")
            && n.contains("Tax Court Rules 170-175")
            && n.contains("informally as possible")
            && n.contains("probative value")));
    }

    #[test]
    fn note_pins_subsection_f_collection_innocent_spouse_worker() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 7463(f)")
            && n.contains("§ 6320")
            && n.contains("§ 6330")
            && n.contains("§ 6015")
            && n.contains("§ 7436")));
    }

    #[test]
    fn note_pins_procedural_tradeoff() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("FASTER")
            && n.contains("CHEAPER")
            && n.contains("INFORMAL")
            && n.contains("NO APPEAL")
            && n.contains("NO PRECEDENTIAL VALUE")));
    }

    #[test]
    fn case_type_truth_table_eight_cells() {
        for case_type in [
            CaseType::IncomeTaxDeficiency,
            CaseType::EstateTax,
            CaseType::GiftTax,
            CaseType::ExciseTax,
            CaseType::Section6320CdpLien,
            CaseType::Section6330CdpLevy,
            CaseType::Section6015InnocentSpouse,
            CaseType::Section7436WorkerClassification,
        ] {
            let mut i = valid_base();
            i.case_type = case_type;
            let r = check(&i);
            assert!(r.s_case_designation_available);
        }
    }

    #[test]
    fn amount_boundary_invariant() {
        let mut i_at = valid_base();
        i_at.amount_in_dispute_cents = 5_000_000;
        let r_at = check(&i_at);
        assert!(r_at.within_50000_limit);

        let mut i_over = valid_base();
        i_over.amount_in_dispute_cents = 5_000_001;
        let r_over = check(&i_over);
        assert!(!r_over.within_50000_limit);
    }

    #[test]
    fn s_case_active_blocks_appeal_invariant() {
        let r_active = check(&valid_base());
        assert!(!r_active.decision_appealable);

        let mut i_discontinued = valid_base();
        i_discontinued.motion_to_discontinue_designation = true;
        let r_discontinued = check(&i_discontinued);
        assert!(r_discontinued.decision_appealable);
    }

    #[test]
    fn defensive_zero_amount_designation_available() {
        let mut i = valid_base();
        i.amount_in_dispute_cents = 0;
        let r = check(&i);
        assert!(r.within_50000_limit);
        assert!(r.s_case_designation_available);
    }

    #[test]
    fn multiple_failures_stack() {
        let mut i = valid_base();
        i.amount_in_dispute_cents = 6_000_000;
        i.tax_court_concurred_before_hearing = false;
        let r = check(&i);
        assert!(!r.s_case_designation_available);
        assert_eq!(r.failure_reasons.len(), 2);
    }
}
