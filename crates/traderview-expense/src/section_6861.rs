//! IRC § 6861 — Jeopardy assessments of income, estate,
//! gift, and certain excise taxes. The emergency-collection
//! mechanism by which IRS may bypass normal § 6212 SNOD +
//! § 6213 Tax Court window procedures when Secretary
//! believes assessment or collection of deficiency will be
//! jeopardized by delay. Trader-procedural-critical because
//! a jeopardy assessment triggers IMMEDIATE notice and
//! demand under § 6303 + § 6321 lien attachment + § 6331
//! levy availability (subject to § 7429 30-day review and
//! § 6863 stay-bond procedures). Companion to § 6201
//! (assessment authority), § 6203 (method of assessment),
//! § 6212 (SNOD), § 6303 (notice and demand), § 6321 (lien),
//! § 6331 (levy), § 7429 (review of jeopardy procedures),
//! § 6863 (stay of collection of jeopardy assessments),
//! § 7522 (content of notices).
//!
//! **§ 6861(a) Authority for making** — if Secretary
//! believes that assessment or collection of a deficiency
//! (as defined in § 6211) will be **jeopardized by delay**,
//! Secretary shall, notwithstanding § 6213(a), **immediately
//! assess** such deficiency together with all interest,
//! additional amounts, and additions to the tax provided
//! for by law, and notice and demand shall be made.
//!
//! **§ 6861(b) Deficiency letters** — if jeopardy assessment
//! is made BEFORE any notice in respect of the tax has been
//! mailed under § 6212(a), then Secretary shall **mail SNOD
//! under § 6212(a) within 60 days** after making the
//! jeopardy assessment.
//!
//! **§ 6861(c) Amount assessable before decision of Tax
//! Court** — jeopardy assessment may be made of deficiency
//! greater than amount in § 6212 SNOD before Tax Court
//! decision becomes final.
//!
//! **§ 6861(d) Amount assessable after decision of Tax
//! Court** — jeopardy assessment after Tax Court decision is
//! limited to amount of deficiency determined by Tax Court.
//!
//! **§ 6861(e) Expiration of right to assess** — § 6861
//! jeopardy assessment authority expires at expiration of
//! period within which assessment otherwise prohibited.
//!
//! **§ 6861(f) Collection of unpaid amounts** — when
//! jeopardy assessment unpaid, collection may proceed via
//! lien (§ 6321) and levy (§ 6331) immediately.
//!
//! **§ 6861(g) Abatement if Tax Court determines no
//! deficiency** — if Tax Court determines deficiency is
//! less than jeopardy assessment, amount assessed in excess
//! shall be abated.
//!
//! **§ 7429 review procedures** (cross-referenced):
//! - § 7429(a)(1)(A) — **Chief Counsel for IRS personal
//!   written approval** required before § 6861 jeopardy
//!   assessment OR § 6331 jeopardy levy.
//! - § 7429(a)(1)(B) — Secretary shall provide taxpayer
//!   **written statement of information relied upon within
//!   5 days** of jeopardy assessment.
//! - § 7429(a)(2) — Taxpayer may request administrative
//!   review **within 30 days** of receiving written
//!   statement.
//! - § 7429(b)(1) — Taxpayer may file judicial review action
//!   in district court within 90 days after earlier of (i)
//!   Secretary notification of administrative review
//!   determination OR (ii) 16th day after taxpayer's
//!   administrative review request.
//! - § 7429(g)(1) — **Burden of proof on Secretary** to show
//!   jeopardy assessment was reasonable under the
//!   circumstances.
//! - § 7429(g)(2) — **Burden of proof on TAXPAYER** to show
//!   that amount assessed is not appropriate.
//!
//! Citations: 26 USC § 6861(a)-(g); 26 CFR § 301.6861-1; 26
//! USC § 7429 (review); 26 CFR § 301.7429-2; § 6212; § 6213;
//! § 6303; § 6321; § 6331; § 6863 (stay of collection);
//! § 7522 (content of notices); IRM 4.15.1 (Jeopardy and
//! Terminations); IRM 5.17.15; IRM 5.1.4.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JeopardyTrigger {
    /// Taxpayer designing to quickly depart from US or
    /// conceal property.
    DepartingOrConcealing,
    /// Taxpayer designing to dissipate assets that would
    /// satisfy tax.
    DissipatingAssets,
    /// Taxpayer's financial solvency appears to be imperiled.
    FinancialSolvencyImperiled,
    /// No jeopardy trigger established.
    None,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section6861Input {
    pub jeopardy_trigger: JeopardyTrigger,
    /// Whether Chief Counsel for IRS provided personal
    /// written approval (§ 7429(a)(1)(A) requirement).
    pub chief_counsel_personal_written_approval: bool,
    /// Whether assessment was made before § 6212 SNOD was
    /// mailed.
    pub assessment_before_snod_mailed: bool,
    /// Days from jeopardy assessment to SNOD mailing under
    /// § 6212(a) (§ 6861(b) 60-day requirement).
    pub days_from_assessment_to_snod_mailing: u32,
    /// Days from jeopardy assessment to written statement
    /// of information provided to taxpayer (§ 7429(a)(1)(B)
    /// 5-day requirement).
    pub days_from_assessment_to_written_statement: u32,
    /// Days from receipt of written statement to taxpayer's
    /// administrative review request (§ 7429(a)(2) 30-day
    /// window).
    pub days_from_statement_to_review_request: u32,
    /// Whether taxpayer filed § 7429(b)(1) judicial review
    /// in district court.
    pub judicial_review_filed: bool,
    /// Days from earlier-of trigger to judicial review
    /// filing (§ 7429(b)(1) 90-day window).
    pub days_from_trigger_to_judicial_review: u32,
    /// Whether IRS sustained burden of proof showing
    /// jeopardy assessment was reasonable (§ 7429(g)(1)).
    pub secretary_sustained_reasonableness_burden: bool,
    /// Whether taxpayer sustained burden showing amount
    /// assessed not appropriate (§ 7429(g)(2)).
    pub taxpayer_sustained_amount_burden: bool,
    /// Whether Tax Court determined no deficiency triggering
    /// § 6861(g) abatement.
    pub tax_court_determined_no_deficiency: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section6861Result {
    pub jeopardy_assessment_authorized: bool,
    pub chief_counsel_approval_compliant: bool,
    pub snod_within_60_day_window: bool,
    pub written_statement_within_5_day_window: bool,
    pub taxpayer_review_request_timely: bool,
    pub judicial_review_timely: bool,
    pub jeopardy_assessment_reasonable: bool,
    pub amount_assessed_appropriate: bool,
    pub abatement_required: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section6861Input) -> Section6861Result {
    let mut failure_reasons: Vec<String> = Vec::new();

    let trigger_exists = !matches!(input.jeopardy_trigger, JeopardyTrigger::None);

    if !trigger_exists {
        failure_reasons.push(
            "26 USC § 6861(a) — Secretary must believe assessment or collection of deficiency will be JEOPARDIZED BY DELAY; no jeopardy trigger established".to_string(),
        );
    }

    if !input.chief_counsel_personal_written_approval {
        failure_reasons.push(
            "26 USC § 7429(a)(1)(A) — Chief Counsel for IRS (or delegate) must PERSONALLY APPROVE IN WRITING before § 6861 jeopardy assessment or § 6331 jeopardy levy".to_string(),
        );
    }

    let snod_compliant = if input.assessment_before_snod_mailed {
        input.days_from_assessment_to_snod_mailing <= 60
    } else {
        true
    };

    if input.assessment_before_snod_mailed && !snod_compliant {
        failure_reasons.push(
            "26 USC § 6861(b) — Secretary shall mail SNOD under § 6212(a) within 60 DAYS after making jeopardy assessment when no SNOD previously mailed".to_string(),
        );
    }

    let written_statement_compliant = input.days_from_assessment_to_written_statement <= 5;
    if !written_statement_compliant {
        failure_reasons.push(
            "26 USC § 7429(a)(1)(B) — Secretary shall provide taxpayer written statement of information relied upon within 5 DAYS of jeopardy assessment".to_string(),
        );
    }

    let review_request_timely = input.days_from_statement_to_review_request <= 30;
    let judicial_review_timely =
        !input.judicial_review_filed || input.days_from_trigger_to_judicial_review <= 90;

    if input.judicial_review_filed && !judicial_review_timely {
        failure_reasons.push(
            "26 USC § 7429(b)(1) — taxpayer must file judicial review action in district court within 90 days after earlier of (i) Secretary notification of administrative review determination OR (ii) 16th day after administrative review request".to_string(),
        );
    }

    let abatement = input.tax_court_determined_no_deficiency;

    let jeopardy_authorized = trigger_exists
        && input.chief_counsel_personal_written_approval
        && snod_compliant
        && written_statement_compliant
        && !abatement;

    let notes: Vec<String> = vec![
        "26 USC § 6861(a) — if Secretary believes assessment or collection of deficiency will be jeopardized by delay, Secretary shall immediately assess deficiency together with interest, additional amounts, and additions to tax, and notice and demand shall be made".to_string(),
        "26 USC § 6861(b) — if jeopardy assessment made before § 6212(a) SNOD, Secretary shall mail SNOD within 60 days after assessment; preserves taxpayer Tax Court petition right under § 6213(a)".to_string(),
        "26 USC § 6861(c)-(d) — jeopardy assessment may be made of greater amount than § 6212 SNOD deficiency before Tax Court decision; limited to Tax Court-determined amount after decision".to_string(),
        "26 USC § 6861(f) — when jeopardy assessment unpaid, collection may proceed via § 6321 lien and § 6331 levy IMMEDIATELY (no 10-day neglect rule); subject to § 6863 stay-bond procedures".to_string(),
        "26 USC § 6861(g) — if Tax Court determines deficiency less than jeopardy assessment, amount assessed in excess shall be ABATED".to_string(),
        "26 USC § 7429(a)(1)(A) — Chief Counsel for IRS personal written approval required before jeopardy assessment or jeopardy levy (no delegation below Chief Counsel-level office)".to_string(),
        "26 USC § 7429(a)(1)(B) — Secretary shall provide taxpayer written statement of information relied upon within 5 days of jeopardy assessment".to_string(),
        "26 USC § 7429(a)(2) — taxpayer may request administrative review within 30 days of receiving written statement".to_string(),
        "26 USC § 7429(b)(1) — taxpayer may file judicial review action in district court within 90 days after earlier of (i) Secretary notification of administrative review determination OR (ii) 16th day after taxpayer's administrative review request".to_string(),
        "26 USC § 7429(g)(1) — burden of proof on SECRETARY to show jeopardy assessment was reasonable under the circumstances".to_string(),
        "26 USC § 7429(g)(2) — burden of proof on TAXPAYER to show amount assessed is not appropriate; burden split between procedural reasonableness (Secretary) and substantive amount (taxpayer)".to_string(),
        "Cross-references: § 6861 jeopardy assessment is § 6303 notice and demand trigger + § 6321 lien attachment + § 6331 levy availability subject to § 7429 review and § 6863 stay-bond procedures; § 7522 governs content of jeopardy assessment notice".to_string(),
        "IRM 4.15.1 (Jeopardy and Terminations) + IRM 5.17.15 + IRM 5.1.4 — internal IRS procedural guidance on jeopardy assessment determination and execution".to_string(),
    ];

    Section6861Result {
        jeopardy_assessment_authorized: jeopardy_authorized,
        chief_counsel_approval_compliant: input.chief_counsel_personal_written_approval,
        snod_within_60_day_window: snod_compliant,
        written_statement_within_5_day_window: written_statement_compliant,
        taxpayer_review_request_timely: review_request_timely,
        judicial_review_timely,
        jeopardy_assessment_reasonable: input.secretary_sustained_reasonableness_burden,
        amount_assessed_appropriate: !input.taxpayer_sustained_amount_burden,
        abatement_required: abatement,
        failure_reasons,
        citation: "26 USC § 6861(a)-(g); 26 CFR § 301.6861-1; 26 USC § 7429; 26 CFR § 301.7429-2; § 6212; § 6213; § 6303; § 6321; § 6331; § 6863; § 7522; IRM 4.15.1; IRM 5.17.15; IRM 5.1.4",
        notes,
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    fn valid_base() -> Section6861Input {
        Section6861Input {
            jeopardy_trigger: JeopardyTrigger::DepartingOrConcealing,
            chief_counsel_personal_written_approval: true,
            assessment_before_snod_mailed: true,
            days_from_assessment_to_snod_mailing: 30,
            days_from_assessment_to_written_statement: 3,
            days_from_statement_to_review_request: 20,
            judicial_review_filed: false,
            days_from_trigger_to_judicial_review: 0,
            secretary_sustained_reasonableness_burden: true,
            taxpayer_sustained_amount_burden: false,
            tax_court_determined_no_deficiency: false,
        }
    }

    #[test]
    fn fully_compliant_jeopardy_assessment_authorized() {
        let r = check(&valid_base());
        assert!(r.jeopardy_assessment_authorized);
        assert!(r.chief_counsel_approval_compliant);
        assert!(r.snod_within_60_day_window);
        assert!(r.written_statement_within_5_day_window);
    }

    #[test]
    fn no_jeopardy_trigger_authorization_fails() {
        let mut i = valid_base();
        i.jeopardy_trigger = JeopardyTrigger::None;
        let r = check(&i);
        assert!(!r.jeopardy_assessment_authorized);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 6861(a)") && f.contains("JEOPARDIZED BY DELAY")));
    }

    #[test]
    fn departing_or_concealing_trigger_authorized() {
        let mut i = valid_base();
        i.jeopardy_trigger = JeopardyTrigger::DepartingOrConcealing;
        let r = check(&i);
        assert!(r.jeopardy_assessment_authorized);
    }

    #[test]
    fn dissipating_assets_trigger_authorized() {
        let mut i = valid_base();
        i.jeopardy_trigger = JeopardyTrigger::DissipatingAssets;
        let r = check(&i);
        assert!(r.jeopardy_assessment_authorized);
    }

    #[test]
    fn financial_solvency_imperiled_trigger_authorized() {
        let mut i = valid_base();
        i.jeopardy_trigger = JeopardyTrigger::FinancialSolvencyImperiled;
        let r = check(&i);
        assert!(r.jeopardy_assessment_authorized);
    }

    #[test]
    fn chief_counsel_approval_missing_fails() {
        let mut i = valid_base();
        i.chief_counsel_personal_written_approval = false;
        let r = check(&i);
        assert!(!r.jeopardy_assessment_authorized);
        assert!(!r.chief_counsel_approval_compliant);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 7429(a)(1)(A)") && f.contains("PERSONALLY APPROVE")));
    }

    #[test]
    fn snod_60_day_boundary_compliant() {
        let mut i = valid_base();
        i.days_from_assessment_to_snod_mailing = 60;
        let r = check(&i);
        assert!(r.snod_within_60_day_window);
        assert!(r.jeopardy_assessment_authorized);
    }

    #[test]
    fn snod_61_day_violation() {
        let mut i = valid_base();
        i.days_from_assessment_to_snod_mailing = 61;
        let r = check(&i);
        assert!(!r.snod_within_60_day_window);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 6861(b)") && f.contains("60 DAYS")));
    }

    #[test]
    fn snod_already_mailed_no_60_day_check() {
        let mut i = valid_base();
        i.assessment_before_snod_mailed = false;
        i.days_from_assessment_to_snod_mailing = 100;
        let r = check(&i);
        assert!(r.snod_within_60_day_window);
        assert!(r.jeopardy_assessment_authorized);
    }

    #[test]
    fn written_statement_5_day_boundary_compliant() {
        let mut i = valid_base();
        i.days_from_assessment_to_written_statement = 5;
        let r = check(&i);
        assert!(r.written_statement_within_5_day_window);
    }

    #[test]
    fn written_statement_6_day_violation() {
        let mut i = valid_base();
        i.days_from_assessment_to_written_statement = 6;
        let r = check(&i);
        assert!(!r.written_statement_within_5_day_window);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 7429(a)(1)(B)") && f.contains("5 DAYS")));
    }

    #[test]
    fn taxpayer_review_request_30_day_boundary_timely() {
        let mut i = valid_base();
        i.days_from_statement_to_review_request = 30;
        let r = check(&i);
        assert!(r.taxpayer_review_request_timely);
    }

    #[test]
    fn taxpayer_review_request_31_days_untimely() {
        let mut i = valid_base();
        i.days_from_statement_to_review_request = 31;
        let r = check(&i);
        assert!(!r.taxpayer_review_request_timely);
    }

    #[test]
    fn judicial_review_90_day_boundary_timely() {
        let mut i = valid_base();
        i.judicial_review_filed = true;
        i.days_from_trigger_to_judicial_review = 90;
        let r = check(&i);
        assert!(r.judicial_review_timely);
    }

    #[test]
    fn judicial_review_91_day_violation() {
        let mut i = valid_base();
        i.judicial_review_filed = true;
        i.days_from_trigger_to_judicial_review = 91;
        let r = check(&i);
        assert!(!r.judicial_review_timely);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 7429(b)(1)") && f.contains("90 days")));
    }

    #[test]
    fn no_judicial_review_no_timeliness_check() {
        let mut i = valid_base();
        i.judicial_review_filed = false;
        i.days_from_trigger_to_judicial_review = 365;
        let r = check(&i);
        assert!(r.judicial_review_timely);
    }

    #[test]
    fn secretary_sustained_reasonableness_burden_pinned() {
        let r = check(&valid_base());
        assert!(r.jeopardy_assessment_reasonable);
    }

    #[test]
    fn taxpayer_sustained_amount_burden_inverts_appropriateness() {
        let mut i = valid_base();
        i.taxpayer_sustained_amount_burden = true;
        let r = check(&i);
        assert!(!r.amount_assessed_appropriate);
    }

    #[test]
    fn tax_court_no_deficiency_engages_abatement() {
        let mut i = valid_base();
        i.tax_court_determined_no_deficiency = true;
        let r = check(&i);
        assert!(r.abatement_required);
        assert!(!r.jeopardy_assessment_authorized);
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&valid_base());
        assert!(r.citation.contains("§ 6861(a)-(g)"));
        assert!(r.citation.contains("26 CFR § 301.6861-1"));
        assert!(r.citation.contains("§ 7429"));
        assert!(r.citation.contains("26 CFR § 301.7429-2"));
        assert!(r.citation.contains("§ 6212"));
        assert!(r.citation.contains("§ 6213"));
        assert!(r.citation.contains("§ 6303"));
        assert!(r.citation.contains("§ 6321"));
        assert!(r.citation.contains("§ 6331"));
        assert!(r.citation.contains("§ 6863"));
        assert!(r.citation.contains("§ 7522"));
        assert!(r.citation.contains("IRM 4.15.1"));
        assert!(r.citation.contains("IRM 5.17.15"));
    }

    #[test]
    fn note_pins_section_a_authority() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6861(a)")
            && n.contains("jeopardized by delay")
            && n.contains("immediately assess")));
    }

    #[test]
    fn note_pins_section_b_60_day_snod_rule() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6861(b)")
            && n.contains("60 days")
            && n.contains("§ 6212(a)")
            && n.contains("§ 6213(a)")));
    }

    #[test]
    fn note_pins_section_f_immediate_lien_and_levy() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6861(f)")
            && n.contains("§ 6321")
            && n.contains("§ 6331")
            && n.contains("IMMEDIATELY")
            && n.contains("§ 6863")));
    }

    #[test]
    fn note_pins_section_g_abatement() {
        let r = check(&valid_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6861(g)") && n.contains("ABATED")));
    }

    #[test]
    fn note_pins_7429_a1A_chief_counsel_approval() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 7429(a)(1)(A)")
            && n.contains("Chief Counsel")
            && n.contains("personal written approval")));
    }

    #[test]
    fn note_pins_7429_a1B_5_day_written_statement() {
        let r = check(&valid_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 7429(a)(1)(B)") && n.contains("5 days")));
    }

    #[test]
    fn note_pins_7429_a2_30_day_administrative_review() {
        let r = check(&valid_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 7429(a)(2)") && n.contains("30 days")));
    }

    #[test]
    fn note_pins_7429_b1_90_day_judicial_review() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 7429(b)(1)")
            && n.contains("90 days")
            && n.contains("16th day")));
    }

    #[test]
    fn note_pins_burden_split_g1_secretary_g2_taxpayer() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 7429(g)(2)")
            && n.contains("burden of proof on TAXPAYER")
            && n.contains("Secretary")
            && n.contains("burden split")));
    }

    #[test]
    fn jeopardy_trigger_truth_table() {
        for (trigger, exp_authorized) in [
            (JeopardyTrigger::DepartingOrConcealing, true),
            (JeopardyTrigger::DissipatingAssets, true),
            (JeopardyTrigger::FinancialSolvencyImperiled, true),
            (JeopardyTrigger::None, false),
        ] {
            let mut i = valid_base();
            i.jeopardy_trigger = trigger;
            let r = check(&i);
            assert_eq!(r.jeopardy_assessment_authorized, exp_authorized);
        }
    }

    #[test]
    fn multiple_failures_stack() {
        let mut i = valid_base();
        i.jeopardy_trigger = JeopardyTrigger::None;
        i.chief_counsel_personal_written_approval = false;
        i.days_from_assessment_to_snod_mailing = 100;
        i.days_from_assessment_to_written_statement = 10;
        let r = check(&i);
        assert!(!r.jeopardy_assessment_authorized);
        assert_eq!(r.failure_reasons.len(), 4);
    }

    #[test]
    fn assessment_before_snod_60_day_check_only_when_engaged_invariant() {
        let mut i_before = valid_base();
        i_before.assessment_before_snod_mailed = true;
        i_before.days_from_assessment_to_snod_mailing = 100;
        let r_before = check(&i_before);
        assert!(!r_before.snod_within_60_day_window);

        let mut i_after = valid_base();
        i_after.assessment_before_snod_mailed = false;
        i_after.days_from_assessment_to_snod_mailing = 100;
        let r_after = check(&i_after);
        assert!(r_after.snod_within_60_day_window);
    }

    #[test]
    fn tax_court_no_deficiency_blocks_authorization_invariant() {
        let mut i = valid_base();
        i.tax_court_determined_no_deficiency = true;
        let r = check(&i);
        assert!(r.abatement_required);
        assert!(!r.jeopardy_assessment_authorized);
    }
}
