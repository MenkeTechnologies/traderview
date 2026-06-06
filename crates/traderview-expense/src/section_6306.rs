//! IRC § 6306 — Qualified Tax Collection Contracts (private
//! collection agency program). Added by American Jobs
//! Creation Act of 2004 § 881; current statutory framework
//! consolidated by FAST Act § 32102 (eff. 2015) which made
//! private collection of inactive tax receivables MANDATORY.
//! Trader-relevant when an old IRS receivable on a trader-
//! taxpayer becomes inactive (over 2 years post-assessment
//! unassigned, OR over 365 days no taxpayer contact) and IRS
//! must assign to one of four authorized PCAs (CBE Group,
//! Coast Professional, ConServe, Pioneer Credit Recovery).
//! Companion to § 6304 (Fair Tax Collection Practices —
//! § 6306(f) explicitly extends § 6304 to PCA contractors),
//! § 7521 (audio recording — § 6306(e) restricts), § 7811
//! (TAO), § 7430 (litigation costs).
//!
//! **§ 6306(a) — In general**: Secretary may enter into one
//! or more qualified tax collection contracts for the
//! collection of outstanding inactive tax receivables.
//!
//! **§ 6306(b) — Qualified tax collection contract defined**:
//! contract for services of any person (other than Treasury
//! officer/employee) to:
//! 1. locate and contact taxpayer specified by Secretary;
//! 2. request full payment from such taxpayer of Federal tax
//!    amount specified by Secretary;
//! 3. if request cannot be met, offer installment agreement
//!    providing full payment during a period **not to exceed
//!    7 years**;
//! 4. obtain financial information specified by Secretary.
//!
//! **§ 6306(c) — Inactive tax receivable defined** — any tax
//! receivable IF:
//! 1. IRS removes such receivable from active inventory for
//!    lack of resources or inability to locate taxpayer; OR
//! 2. more than **2 years** has passed since assessment and
//!    such receivable has not been assigned for collection to
//!    any IRS employee; OR
//! 3. in the case of a receivable assigned for collection,
//!    more than **365 days** have passed without interaction
//!    with taxpayer or third party for purposes of furthering
//!    collection.
//!
//! **§ 6306(d) — Tax receivables ineligible for collection
//! under contract**: a tax receivable shall **NOT** be
//! eligible for collection pursuant to a qualified tax
//! collection contract if such receivable:
//! 1. is subject to a pending or active offer-in-compromise
//!    or installment agreement;
//! 2. is classified as an innocent spouse case;
//! 3. involves a taxpayer identified by Secretary as deceased,
//!    under age 18, in a designated combat zone, or a victim
//!    of tax-related identity theft;
//! 4. is currently under examination, litigation, criminal
//!    investigation, or levy;
//! 5. is currently subject to a proper exercise of a right of
//!    appeal;
//! 6. is classified as an innocent spouse case;
//! 7. involves a taxpayer substantially all of whose income
//!    consists of **disability insurance benefits under § 223
//!    of the Social Security Act or supplemental security
//!    income (SSI) benefits under title XVI** of the Social
//!    Security Act;
//! 8. involves a taxpayer who is an individual with adjusted
//!    gross income which **does not exceed 200 percent of the
//!    applicable poverty level** (Taxpayer First Act of 2019
//!    addition).
//!
//! **§ 6306(e) — Restrictions on PCA actions** — collection
//! contractor SHALL NOT:
//! 1. impose tax;
//! 2. accept settlement on terms not within IRS authority;
//! 3. take any enforcement action (lien filing, levy);
//! 4. conduct § 7521(b)(2) in-person interview audio
//!    recording.
//!
//! **§ 6306(f) — Application of Fair Tax Collection
//! Practices** — § 6304 (Fair Tax Collection Practices) and
//! § 7433 (civil damages) apply to PCA contractor as if the
//! contractor were an IRS officer/employee. FDCPA (15 USC §
//! 1692) also applies to PCA collection of tax debt.
//!
//! **§ 6306(j) — Special compliance personnel program
//! account** — **25% of collected revenues** retained for
//! IRS special compliance personnel program; 25% for PCA
//! retention; remaining 50% to general fund.
//!
//! Citations: 26 USC § 6306(a)-(j); American Jobs Creation
//! Act of 2004 § 881; FAST Act of 2015 § 32102; Taxpayer
//! First Act of 2019 § 1205 (200% poverty exclusion); 15 USC
//! § 1692 (FDCPA); § 6304 (Fair Tax Collection); § 7433
//! (civil damages); § 7521 (audio recording); § 7811 (TAO);
//! § 7430 (litigation costs); IRM 5.19.9.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InactivityBasis {
    /// § 6306(c)(1) — IRS removed from active inventory for
    /// lack of resources or inability to locate taxpayer.
    RemovedFromActiveInventory,
    /// § 6306(c)(2) — over 2 years since assessment + not
    /// assigned for collection.
    OverTwoYearsUnassigned,
    /// § 6306(c)(3) — over 365 days no taxpayer contact on
    /// assigned receivable.
    OverThreeSixtyFiveDaysNoContact,
    /// Receivable is still active at IRS (not inactive).
    StillActive,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExclusionCategory {
    /// Not excluded.
    None,
    /// § 6306(d)(1) — pending or active OIC or installment
    /// agreement.
    PendingOicOrIa,
    /// § 6306(d)(2) — innocent spouse case.
    InnocentSpouse,
    /// § 6306(d)(3) — deceased taxpayer.
    Deceased,
    /// § 6306(d)(3) — taxpayer under 18.
    Under18,
    /// § 6306(d)(3) — designated combat zone or contingency
    /// operation.
    CombatZone,
    /// § 6306(d)(3) — identity theft victim.
    IdentityTheft,
    /// § 6306(d)(4) — under examination, litigation,
    /// criminal investigation, or levy.
    UnderExamLitigationLevy,
    /// § 6306(d)(5) — proper exercise of appeal rights.
    AppealsPending,
    /// § 6306(d)(7) — disability insurance benefits or SSI.
    DisabilityOrSsi,
    /// § 6306(d)(8) — AGI ≤ 200% of applicable poverty level
    /// (Taxpayer First Act of 2019).
    AgiUnder200PctPoverty,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section6306Input {
    pub inactivity_basis: InactivityBasis,
    pub exclusion_category: ExclusionCategory,
    /// Days since IRS assessment under § 6203.
    pub days_since_assessment: u32,
    /// Days since last IRS or taxpayer contact (if assigned).
    pub days_since_last_contact: u32,
    /// Whether receivable has ever been assigned to IRS
    /// employee for collection.
    pub assigned_to_irs_employee: bool,
    /// Proposed PCA installment agreement period in years.
    pub proposed_installment_years: u32,
    /// Whether PCA proposed any enforcement action (lien,
    /// levy, settlement outside IRS authority).
    pub pca_attempted_enforcement_action: bool,
    /// Whether PCA conducted § 7521(b)(2) audio-recorded
    /// in-person interview.
    pub pca_conducted_audio_recorded_interview: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section6306Result {
    pub inactive_receivable_qualifies: bool,
    pub eligible_for_pca_assignment: bool,
    pub exclusion_engaged: bool,
    pub installment_agreement_within_7_year_cap: bool,
    pub pca_enforcement_action_violation: bool,
    pub pca_audio_recording_violation: bool,
    pub section_6304_extension_engaged: bool,
    pub fdcpa_extension_engaged: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section6306Input) -> Section6306Result {
    let mut violations: Vec<String> = Vec::new();

    let inactive_qualifies = match input.inactivity_basis {
        InactivityBasis::RemovedFromActiveInventory => true,
        InactivityBasis::OverTwoYearsUnassigned => {
            input.days_since_assessment > 730 && !input.assigned_to_irs_employee
        }
        InactivityBasis::OverThreeSixtyFiveDaysNoContact => {
            input.assigned_to_irs_employee && input.days_since_last_contact > 365
        }
        InactivityBasis::StillActive => false,
    };

    if !inactive_qualifies {
        violations.push(
            "26 USC § 6306(c) — receivable does not satisfy inactive tax receivable definition; PCA assignment requires (1) removal from active inventory, (2) over 2 years post-assessment unassigned, or (3) over 365 days no contact on assigned receivable".to_string(),
        );
    }

    let exclusion_engaged = !matches!(input.exclusion_category, ExclusionCategory::None);

    if exclusion_engaged {
        match input.exclusion_category {
            ExclusionCategory::PendingOicOrIa => violations.push(
                "26 USC § 6306(d)(1) — receivable subject to pending or active offer-in-compromise or installment agreement excluded from PCA collection".to_string(),
            ),
            ExclusionCategory::InnocentSpouse => violations.push(
                "26 USC § 6306(d)(2) — innocent spouse cases excluded from PCA collection".to_string(),
            ),
            ExclusionCategory::Deceased => violations.push(
                "26 USC § 6306(d)(3) — deceased taxpayer excluded from PCA collection".to_string(),
            ),
            ExclusionCategory::Under18 => violations.push(
                "26 USC § 6306(d)(3) — taxpayer under age 18 excluded from PCA collection".to_string(),
            ),
            ExclusionCategory::CombatZone => violations.push(
                "26 USC § 6306(d)(3) — designated combat zone or contingency operation taxpayer excluded from PCA collection".to_string(),
            ),
            ExclusionCategory::IdentityTheft => violations.push(
                "26 USC § 6306(d)(3) — identity theft victim excluded from PCA collection".to_string(),
            ),
            ExclusionCategory::UnderExamLitigationLevy => violations.push(
                "26 USC § 6306(d)(4) — taxpayer under examination, litigation, criminal investigation, or levy excluded from PCA collection".to_string(),
            ),
            ExclusionCategory::AppealsPending => violations.push(
                "26 USC § 6306(d)(5) — proper exercise of appeal rights excludes receivable from PCA collection".to_string(),
            ),
            ExclusionCategory::DisabilityOrSsi => violations.push(
                "26 USC § 6306(d)(7) — taxpayer with substantially all income from § 223 Social Security disability insurance or SSI under title XVI excluded from PCA collection".to_string(),
            ),
            ExclusionCategory::AgiUnder200PctPoverty => violations.push(
                "26 USC § 6306(d)(8) — taxpayer with AGI not exceeding 200 percent of applicable poverty level excluded from PCA collection (Taxpayer First Act of 2019 § 1205)".to_string(),
            ),
            ExclusionCategory::None => {}
        }
    }

    let ia_within_cap = input.proposed_installment_years <= 7;
    if !ia_within_cap {
        violations.push(
            "26 USC § 6306(b)(1)(C) — installment agreement period offered by PCA shall not exceed 7 years".to_string(),
        );
    }

    if input.pca_attempted_enforcement_action {
        violations.push(
            "26 USC § 6306(e) — PCA contractor SHALL NOT impose tax, accept settlement on terms not within IRS authority, or take any enforcement action including lien filing or levy".to_string(),
        );
    }

    if input.pca_conducted_audio_recorded_interview {
        violations.push(
            "26 USC § 6306(e) — PCA contractor SHALL NOT conduct § 7521(b)(2) in-person interview audio recording; only IRS officer/employee may conduct".to_string(),
        );
    }

    let eligible = inactive_qualifies && !exclusion_engaged && ia_within_cap;

    let notes: Vec<String> = vec![
        "26 USC § 6306(a)-(b) — Secretary may enter into qualified tax collection contracts for collection of outstanding inactive tax receivables; PCA may locate/contact taxpayer, request full payment, offer 7-year installment agreement, and obtain financial information".to_string(),
        "26 USC § 6306(c) — inactive tax receivable defined as (1) removed from active inventory for lack of resources or inability to locate, (2) over 2 years since assessment unassigned, or (3) over 365 days no contact on assigned receivable".to_string(),
        "26 USC § 6306(d) — exclusions include pending OIC/IA, innocent spouse, deceased, under 18, combat zone, identity theft, examination/litigation/criminal/levy, appeals pending, disability/SSI under § 223 or title XVI, AGI ≤ 200% federal poverty level".to_string(),
        "26 USC § 6306(e) — PCA contractor restrictions: no tax imposition, no settlement outside IRS authority, no enforcement action (lien/levy), no § 7521(b)(2) audio recording".to_string(),
        "26 USC § 6306(f) — § 6304 Fair Tax Collection Practices + § 7433 civil damages apply to PCA contractor as if IRS officer/employee; FDCPA (15 USC § 1692) also applies".to_string(),
        "26 USC § 6306(j) — special compliance personnel program account retains 25% of collected revenues; PCA retains 25%; remaining 50% to general fund".to_string(),
        "American Jobs Creation Act of 2004 § 881 added § 6306; FAST Act of 2015 § 32102 made PCA collection of inactive receivables MANDATORY; Taxpayer First Act of 2019 § 1205 added § 6306(d)(8) 200% poverty exclusion".to_string(),
        "Currently four authorized PCAs: CBE Group, Coast Professional, ConServe, Pioneer Credit Recovery; IRM 5.19.9 internal IRS guidance on PCA program".to_string(),
    ];

    Section6306Result {
        inactive_receivable_qualifies: inactive_qualifies,
        eligible_for_pca_assignment: eligible,
        exclusion_engaged,
        installment_agreement_within_7_year_cap: ia_within_cap,
        pca_enforcement_action_violation: input.pca_attempted_enforcement_action,
        pca_audio_recording_violation: input.pca_conducted_audio_recorded_interview,
        section_6304_extension_engaged: true,
        fdcpa_extension_engaged: true,
        violations,
        citation: "26 USC § 6306(a)-(j); American Jobs Creation Act of 2004 § 881; FAST Act of 2015 § 32102; Taxpayer First Act of 2019 § 1205; 15 USC § 1692; § 6304; § 7433; § 7521; § 7811; § 7430; IRM 5.19.9",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clean_base() -> Section6306Input {
        Section6306Input {
            inactivity_basis: InactivityBasis::RemovedFromActiveInventory,
            exclusion_category: ExclusionCategory::None,
            days_since_assessment: 800,
            days_since_last_contact: 400,
            assigned_to_irs_employee: false,
            proposed_installment_years: 5,
            pca_attempted_enforcement_action: false,
            pca_conducted_audio_recorded_interview: false,
        }
    }

    #[test]
    fn inactive_removed_from_active_inventory_eligible() {
        let r = check(&clean_base());
        assert!(r.inactive_receivable_qualifies);
        assert!(r.eligible_for_pca_assignment);
        assert!(!r.exclusion_engaged);
    }

    #[test]
    fn over_2_years_unassigned_qualifies_as_inactive() {
        let mut i = clean_base();
        i.inactivity_basis = InactivityBasis::OverTwoYearsUnassigned;
        i.days_since_assessment = 731;
        i.assigned_to_irs_employee = false;
        let r = check(&i);
        assert!(r.inactive_receivable_qualifies);
        assert!(r.eligible_for_pca_assignment);
    }

    #[test]
    fn exactly_2_years_unassigned_does_not_qualify() {
        let mut i = clean_base();
        i.inactivity_basis = InactivityBasis::OverTwoYearsUnassigned;
        i.days_since_assessment = 730;
        i.assigned_to_irs_employee = false;
        let r = check(&i);
        assert!(!r.inactive_receivable_qualifies);
    }

    #[test]
    fn over_2_years_assigned_does_not_qualify_under_clause_2() {
        let mut i = clean_base();
        i.inactivity_basis = InactivityBasis::OverTwoYearsUnassigned;
        i.days_since_assessment = 800;
        i.assigned_to_irs_employee = true;
        let r = check(&i);
        assert!(!r.inactive_receivable_qualifies);
    }

    #[test]
    fn over_365_days_no_contact_qualifies_when_assigned() {
        let mut i = clean_base();
        i.inactivity_basis = InactivityBasis::OverThreeSixtyFiveDaysNoContact;
        i.assigned_to_irs_employee = true;
        i.days_since_last_contact = 366;
        let r = check(&i);
        assert!(r.inactive_receivable_qualifies);
    }

    #[test]
    fn exactly_365_days_no_contact_does_not_qualify() {
        let mut i = clean_base();
        i.inactivity_basis = InactivityBasis::OverThreeSixtyFiveDaysNoContact;
        i.assigned_to_irs_employee = true;
        i.days_since_last_contact = 365;
        let r = check(&i);
        assert!(!r.inactive_receivable_qualifies);
    }

    #[test]
    fn still_active_receivable_does_not_qualify() {
        let mut i = clean_base();
        i.inactivity_basis = InactivityBasis::StillActive;
        let r = check(&i);
        assert!(!r.inactive_receivable_qualifies);
        assert!(!r.eligible_for_pca_assignment);
    }

    #[test]
    fn pending_oic_excluded() {
        let mut i = clean_base();
        i.exclusion_category = ExclusionCategory::PendingOicOrIa;
        let r = check(&i);
        assert!(r.exclusion_engaged);
        assert!(!r.eligible_for_pca_assignment);
        assert!(r.violations.iter().any(|v| v.contains("§ 6306(d)(1)")));
    }

    #[test]
    fn innocent_spouse_excluded() {
        let mut i = clean_base();
        i.exclusion_category = ExclusionCategory::InnocentSpouse;
        let r = check(&i);
        assert!(r.exclusion_engaged);
        assert!(r.violations.iter().any(|v| v.contains("§ 6306(d)(2)")));
    }

    #[test]
    fn deceased_taxpayer_excluded() {
        let mut i = clean_base();
        i.exclusion_category = ExclusionCategory::Deceased;
        let r = check(&i);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 6306(d)(3)") && v.contains("deceased")));
    }

    #[test]
    fn taxpayer_under_18_excluded() {
        let mut i = clean_base();
        i.exclusion_category = ExclusionCategory::Under18;
        let r = check(&i);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 6306(d)(3)") && v.contains("under age 18")));
    }

    #[test]
    fn combat_zone_taxpayer_excluded() {
        let mut i = clean_base();
        i.exclusion_category = ExclusionCategory::CombatZone;
        let r = check(&i);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 6306(d)(3)") && v.contains("combat zone")));
    }

    #[test]
    fn identity_theft_victim_excluded() {
        let mut i = clean_base();
        i.exclusion_category = ExclusionCategory::IdentityTheft;
        let r = check(&i);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 6306(d)(3)") && v.contains("identity theft")));
    }

    #[test]
    fn under_exam_excluded() {
        let mut i = clean_base();
        i.exclusion_category = ExclusionCategory::UnderExamLitigationLevy;
        let r = check(&i);
        assert!(r.violations.iter().any(|v| v.contains("§ 6306(d)(4)")));
    }

    #[test]
    fn appeals_pending_excluded() {
        let mut i = clean_base();
        i.exclusion_category = ExclusionCategory::AppealsPending;
        let r = check(&i);
        assert!(r.violations.iter().any(|v| v.contains("§ 6306(d)(5)")));
    }

    #[test]
    fn disability_or_ssi_excluded() {
        let mut i = clean_base();
        i.exclusion_category = ExclusionCategory::DisabilityOrSsi;
        let r = check(&i);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 6306(d)(7)") && v.contains("§ 223") && v.contains("title XVI")));
    }

    #[test]
    fn agi_under_200_pct_poverty_excluded() {
        let mut i = clean_base();
        i.exclusion_category = ExclusionCategory::AgiUnder200PctPoverty;
        let r = check(&i);
        assert!(r.violations.iter().any(|v| v.contains("§ 6306(d)(8)")
            && v.contains("200 percent")
            && v.contains("Taxpayer First Act")));
    }

    #[test]
    fn installment_agreement_at_7_year_cap_compliant() {
        let mut i = clean_base();
        i.proposed_installment_years = 7;
        let r = check(&i);
        assert!(r.installment_agreement_within_7_year_cap);
        assert!(r.eligible_for_pca_assignment);
    }

    #[test]
    fn installment_agreement_8_years_violation() {
        let mut i = clean_base();
        i.proposed_installment_years = 8;
        let r = check(&i);
        assert!(!r.installment_agreement_within_7_year_cap);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 6306(b)(1)(C)") && v.contains("7 years")));
    }

    #[test]
    fn pca_enforcement_action_violation() {
        let mut i = clean_base();
        i.pca_attempted_enforcement_action = true;
        let r = check(&i);
        assert!(r.pca_enforcement_action_violation);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 6306(e)") && v.contains("enforcement action")));
    }

    #[test]
    fn pca_audio_recording_violation() {
        let mut i = clean_base();
        i.pca_conducted_audio_recorded_interview = true;
        let r = check(&i);
        assert!(r.pca_audio_recording_violation);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 6306(e)") && v.contains("§ 7521(b)(2)")));
    }

    #[test]
    fn section_6304_extension_always_engaged() {
        let r = check(&clean_base());
        assert!(r.section_6304_extension_engaged);
        assert!(r.fdcpa_extension_engaged);
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&clean_base());
        assert!(r.citation.contains("§ 6306(a)-(j)"));
        assert!(r
            .citation
            .contains("American Jobs Creation Act of 2004 § 881"));
        assert!(r.citation.contains("FAST Act"));
        assert!(r.citation.contains("§ 32102"));
        assert!(r.citation.contains("Taxpayer First Act of 2019 § 1205"));
        assert!(r.citation.contains("15 USC § 1692"));
        assert!(r.citation.contains("§ 6304"));
        assert!(r.citation.contains("§ 7433"));
        assert!(r.citation.contains("§ 7521"));
        assert!(r.citation.contains("IRM 5.19.9"));
    }

    #[test]
    fn note_pins_7_year_installment_cap() {
        let r = check(&clean_base());
        assert!(r.notes.iter().any(|n| n.contains("7-year installment")));
    }

    #[test]
    fn note_pins_three_inactivity_paths() {
        let r = check(&clean_base());
        assert!(r.notes.iter().any(|n| n.contains("2 years")
            && n.contains("365 days")
            && n.contains("active inventory")));
    }

    #[test]
    fn note_pins_all_exclusion_categories() {
        let r = check(&clean_base());
        assert!(r.notes.iter().any(|n| n.contains("disability/SSI")
            && n.contains("§ 223")
            && n.contains("title XVI")
            && n.contains("200% federal poverty level")));
    }

    #[test]
    fn note_pins_pca_revenue_split() {
        let r = check(&clean_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("25%") && n.contains("§ 6306(j)")));
    }

    #[test]
    fn note_pins_authorized_pcas() {
        let r = check(&clean_base());
        assert!(r.notes.iter().any(|n| n.contains("CBE Group")
            && n.contains("ConServe")
            && n.contains("Pioneer Credit Recovery")));
    }

    #[test]
    fn note_pins_pca_restrictions_no_enforcement() {
        let r = check(&clean_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6306(e)")
            && n.contains("enforcement")
            && n.contains("audio recording")));
    }

    #[test]
    fn note_pins_section_6304_extension() {
        let r = check(&clean_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6304") && n.contains("§ 7433") && n.contains("FDCPA")));
    }

    #[test]
    fn multiple_violations_stack() {
        let mut i = clean_base();
        i.proposed_installment_years = 10;
        i.pca_attempted_enforcement_action = true;
        i.pca_conducted_audio_recorded_interview = true;
        i.exclusion_category = ExclusionCategory::DisabilityOrSsi;
        let r = check(&i);
        assert_eq!(r.violations.len(), 4);
    }

    #[test]
    fn exclusion_truth_table_eleven_cells() {
        for (category, exp_engaged) in [
            (ExclusionCategory::None, false),
            (ExclusionCategory::PendingOicOrIa, true),
            (ExclusionCategory::InnocentSpouse, true),
            (ExclusionCategory::Deceased, true),
            (ExclusionCategory::Under18, true),
            (ExclusionCategory::CombatZone, true),
            (ExclusionCategory::IdentityTheft, true),
            (ExclusionCategory::UnderExamLitigationLevy, true),
            (ExclusionCategory::AppealsPending, true),
            (ExclusionCategory::DisabilityOrSsi, true),
            (ExclusionCategory::AgiUnder200PctPoverty, true),
        ] {
            let mut i = clean_base();
            i.exclusion_category = category;
            let r = check(&i);
            assert_eq!(r.exclusion_engaged, exp_engaged);
            assert_eq!(r.eligible_for_pca_assignment, !exp_engaged);
        }
    }

    #[test]
    fn inactivity_truth_table() {
        for (basis, assigned, days, last_contact, exp_qualifies) in [
            (
                InactivityBasis::RemovedFromActiveInventory,
                false,
                0,
                0,
                true,
            ),
            (InactivityBasis::OverTwoYearsUnassigned, false, 731, 0, true),
            (InactivityBasis::OverTwoYearsUnassigned, true, 731, 0, false),
            (
                InactivityBasis::OverTwoYearsUnassigned,
                false,
                730,
                0,
                false,
            ),
            (
                InactivityBasis::OverThreeSixtyFiveDaysNoContact,
                true,
                0,
                366,
                true,
            ),
            (
                InactivityBasis::OverThreeSixtyFiveDaysNoContact,
                true,
                0,
                365,
                false,
            ),
            (
                InactivityBasis::OverThreeSixtyFiveDaysNoContact,
                false,
                0,
                366,
                false,
            ),
            (InactivityBasis::StillActive, false, 0, 0, false),
        ] {
            let mut i = clean_base();
            i.inactivity_basis = basis;
            i.assigned_to_irs_employee = assigned;
            i.days_since_assessment = days;
            i.days_since_last_contact = last_contact;
            let r = check(&i);
            assert_eq!(
                r.inactive_receivable_qualifies, exp_qualifies,
                "basis={:?} assigned={} days={} contact={}",
                basis, assigned, days, last_contact
            );
        }
    }

    #[test]
    fn installment_cap_boundary_invariant() {
        let mut i_at = clean_base();
        i_at.proposed_installment_years = 7;
        let r_at = check(&i_at);

        let mut i_over = clean_base();
        i_over.proposed_installment_years = 8;
        let r_over = check(&i_over);

        assert!(r_at.installment_agreement_within_7_year_cap);
        assert!(!r_over.installment_agreement_within_7_year_cap);
    }

    #[test]
    fn pca_restrictions_independent_of_eligibility() {
        let mut i = clean_base();
        i.inactivity_basis = InactivityBasis::StillActive;
        i.pca_attempted_enforcement_action = true;
        i.pca_conducted_audio_recorded_interview = true;
        let r = check(&i);
        assert!(!r.inactive_receivable_qualifies);
        assert!(r.pca_enforcement_action_violation);
        assert!(r.pca_audio_recording_violation);
    }

    #[test]
    fn defensive_zero_days_clean_base_with_removed_inventory_qualifies() {
        let mut i = clean_base();
        i.days_since_assessment = 0;
        i.days_since_last_contact = 0;
        let r = check(&i);
        assert!(r.inactive_receivable_qualifies);
    }
}
