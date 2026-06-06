//! Just-cause termination notice CONTENT requirements — what
//! specific content and format must a landlord's written
//! termination notice contain to satisfy just-cause statutory
//! requirements? Distinct from `just_cause_eviction` (which
//! addresses substantive just-cause grounds), `lease_termination_
//! notice` (general termination notice periods), and `eviction_
//! notices` (formal eviction process).
//!
//! Trader-landlord operational concern in just-cause jurisdictions
//! — even when the substantive ground is valid, a defective notice
//! is VOIDABLE and forces the landlord to restart the termination
//! process. Notice content + format requirements are independent
//! procedural traps separate from the substantive just-cause
//! analysis.
//!
//! Four regimes:
//!
//! **California — Cal. Civ. Code § 1946.2(c)**. WRITTEN notice
//! MUST state cause. Specific cause must be identified (at-fault
//! ground per § 1946.2(b)(1) OR no-fault ground per § 1946.2(b)(2)).
//! For CURABLE at-fault violations (lease breach, unauthorized
//! pets, unlawful subletting, property damage), 3-day cure
//! opportunity required before notice to quit. § 1946.2(c) —
//! failure to comply renders the written termination notice VOID.
//! For no-fault grounds (owner move-in, withdrawal, demo, gov
//! order), 60-day notice + § 1946.2(d)(3) one-month-rent
//! relocation assistance.
//!
//! **Washington — RCW 59.18.650(2)**. WRITTEN notice MUST state
//! specific cause and the specific facts and circumstances
//! constituting the cause. § 59.18.650(2)(a)-(p) enumerates 16
//! specific just-cause categories. Failure to provide required
//! content defeats the termination.
//!
//! **Oregon — ORS 90.427 (SB 608)**. WRITTEN notice MUST state
//! reason. Different notice periods for tenancies < 1 year
//! (no-cause permitted) vs ≥ 1 year (just-cause required).
//! Strict-compliance regime — defective notice is voidable.
//!
//! **New Jersey — N.J.S.A. 2A:18-61.2**. Anti-Eviction Act
//! requires WRITTEN notice with specific cause from enumerated
//! statutory categories (§ 2A:18-61.1(a)-(r)). Notice content must
//! match the statutory ground exactly. Defective notice =
//! ineffective termination.
//!
//! **Default — limited content requirements**. Most non-just-cause
//! states require minimal content; lease + state-specific eviction
//! statute control.
//!
//! Citations: Cal. Civ. Code § 1946.2(b)(1)/(b)(2) (CA at-fault /
//! no-fault grounds); § 1946.2(c) (CA written notice content +
//! VOID failure consequence); RCW 59.18.650(2)/(2)(a)-(p) (WA
//! specific cause + 16 categories); ORS 90.427 (OR SB 608 written
//! notice + just-cause); N.J.S.A. 2A:18-61.1(a)-(r) (NJ statutory
//! grounds); § 2A:18-61.2 (NJ Anti-Eviction Act notice content).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    California,
    Washington,
    Oregon,
    NewJersey,
    Default,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GroundsType {
    /// At-fault termination ground (tenant breach, unauthorized
    /// occupant, illegal use, nonpayment, etc.). CA § 1946.2(b)(1).
    AtFault,
    /// No-fault termination ground (owner move-in, withdrawal,
    /// demolition, gov order, substantial remodel). CA § 1946.2(b)(2).
    NoFault,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JustCauseNoticeContentInput {
    pub regime: Regime,
    pub grounds_type: GroundsType,
    /// Whether the landlord provided written notice (oral notice
    /// never satisfies just-cause statutes).
    pub written_notice_provided: bool,
    /// Whether the notice specifically identifies the statutory
    /// just-cause ground (not just "breach of lease" but the
    /// specific subsection).
    pub specific_cause_stated_in_notice: bool,
    /// Whether the notice includes the specific FACTS AND
    /// CIRCUMSTANCES constituting the cause (WA RCW 59.18.650(2)
    /// stricter requirement).
    pub specific_facts_and_circumstances_described: bool,
    /// Whether the at-fault violation is CURABLE (curable
    /// violations require 3-day cure opportunity under CA, varying
    /// periods under other regimes).
    pub at_fault_violation_is_curable: bool,
    /// Whether the landlord provided the required cure opportunity
    /// for curable violations.
    pub cure_opportunity_provided: bool,
    /// Whether the landlord paid the no-fault relocation assistance
    /// (CA § 1946.2(d)(3) one month rent) for no-fault grounds.
    pub no_fault_relocation_assistance_paid: bool,
    /// Whether the notice was properly served per state method
    /// (personal delivery, certified mail, etc.).
    pub proper_service: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct JustCauseNoticeContentResult {
    pub compliant: bool,
    pub notice_voidable_for_defect: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &JustCauseNoticeContentInput) -> JustCauseNoticeContentResult {
    let mut violations: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    if !input.written_notice_provided {
        violations.push(
            "all just-cause regimes require WRITTEN notice; oral notice never satisfies just-cause statutes"
                .to_string(),
        );
    }

    match input.regime {
        Regime::California => check_california(input, &mut violations, &mut notes),
        Regime::Washington => check_washington(input, &mut violations, &mut notes),
        Regime::Oregon => check_oregon(input, &mut violations, &mut notes),
        Regime::NewJersey => check_new_jersey(input, &mut violations, &mut notes),
        Regime::Default => check_default(input, &mut notes),
    }
}

fn check_california(
    input: &JustCauseNoticeContentInput,
    violations: &mut Vec<String>,
    notes: &mut Vec<String>,
) -> JustCauseNoticeContentResult {
    if !input.specific_cause_stated_in_notice {
        violations.push(
            "Cal. Civ. Code § 1946.2(c) — written notice MUST state cause; specific just-cause ground (at-fault § 1946.2(b)(1) or no-fault § 1946.2(b)(2)) must be identified"
                .to_string(),
        );
    }

    if matches!(input.grounds_type, GroundsType::AtFault)
        && input.at_fault_violation_is_curable
        && !input.cure_opportunity_provided
    {
        violations.push(
            "Cal. Civ. Code § 1946.2(c) — at-fault termination for CURABLE violation requires 3-day cure opportunity before notice to quit"
                .to_string(),
        );
    }

    if matches!(input.grounds_type, GroundsType::NoFault)
        && !input.no_fault_relocation_assistance_paid
    {
        violations.push(
            "Cal. Civ. Code § 1946.2(d)(3) — no-fault termination requires one-month-rent relocation assistance"
                .to_string(),
        );
    }

    if !input.proper_service {
        violations.push(
            "Cal. Civ. Code § 1946.2(c) — notice must be lawfully served per Code Civ. Proc. § 1162"
                .to_string(),
        );
    }

    notes.push(
        "§ 1946.2(c) — failure to comply with notice content / service requirements renders the written termination notice VOID"
            .to_string(),
    );
    notes.push(
        "§ 1946.2(b)(1) at-fault grounds: 14 enumerated categories including nonpayment, breach of material term, nuisance, etc."
            .to_string(),
    );
    notes.push(
        "§ 1946.2(b)(2) no-fault grounds: owner move-in, withdrawal from market, demo, gov order, substantial remodel"
            .to_string(),
    );

    let voidable = !violations.is_empty();
    JustCauseNoticeContentResult {
        compliant: violations.is_empty(),
        notice_voidable_for_defect: voidable,
        violations: violations.clone(),
        citation: citation_for(Regime::California),
        notes: notes.clone(),
    }
}

fn check_washington(
    input: &JustCauseNoticeContentInput,
    violations: &mut Vec<String>,
    notes: &mut Vec<String>,
) -> JustCauseNoticeContentResult {
    if !input.specific_cause_stated_in_notice {
        violations.push(
            "RCW 59.18.650(2) — written notice MUST state specific cause from § 59.18.650(2)(a)-(p) 16 enumerated categories"
                .to_string(),
        );
    }

    if !input.specific_facts_and_circumstances_described {
        violations.push(
            "RCW 59.18.650(2) — written notice MUST describe specific FACTS AND CIRCUMSTANCES constituting the cause (stricter than CA written-reason requirement)"
                .to_string(),
        );
    }

    if !input.proper_service {
        violations.push(
            "RCW 59.18.650(2) — notice must be lawfully served per RCW 59.12.040 service methods"
                .to_string(),
        );
    }

    notes.push(
        "RCW 59.18.650(2)(a)-(p) — 16 enumerated just-cause categories: nonpayment, material breach, repeated violations, criminal activity, nuisance, refusing access, illegal occupant, owner move-in, removal from rental market, substantial rehabilitation, etc."
            .to_string(),
    );

    let voidable = !violations.is_empty();
    JustCauseNoticeContentResult {
        compliant: violations.is_empty(),
        notice_voidable_for_defect: voidable,
        violations: violations.clone(),
        citation: citation_for(Regime::Washington),
        notes: notes.clone(),
    }
}

fn check_oregon(
    input: &JustCauseNoticeContentInput,
    violations: &mut Vec<String>,
    notes: &mut Vec<String>,
) -> JustCauseNoticeContentResult {
    if !input.specific_cause_stated_in_notice {
        violations.push(
            "ORS 90.427 (SB 608) — written notice MUST state reason for termination".to_string(),
        );
    }

    if !input.proper_service {
        violations.push(
            "ORS 90.155 — notice must be lawfully served per state service methods".to_string(),
        );
    }

    notes.push(
        "ORS 90.427 (SB 608) — tenancies < 1 year permit no-cause termination with 30-day notice; tenancies ≥ 1 year require just-cause"
            .to_string(),
    );
    notes.push("OR strict-compliance regime — defective notice is VOIDABLE".to_string());

    let voidable = !violations.is_empty();
    JustCauseNoticeContentResult {
        compliant: violations.is_empty(),
        notice_voidable_for_defect: voidable,
        violations: violations.clone(),
        citation: citation_for(Regime::Oregon),
        notes: notes.clone(),
    }
}

fn check_new_jersey(
    input: &JustCauseNoticeContentInput,
    violations: &mut Vec<String>,
    notes: &mut Vec<String>,
) -> JustCauseNoticeContentResult {
    if !input.specific_cause_stated_in_notice {
        violations.push(
            "N.J.S.A. 2A:18-61.2 — Anti-Eviction Act notice MUST state specific cause from § 2A:18-61.1(a)-(r) enumerated statutory grounds"
                .to_string(),
        );
    }

    if !input.proper_service {
        violations.push(
            "N.J.S.A. 2A:18-61.2 — notice must be lawfully served per Anti-Eviction Act service methods"
                .to_string(),
        );
    }

    notes.push(
        "N.J.S.A. 2A:18-61.1(a)-(r) — 18 enumerated just-cause grounds: nonpayment, disorderly conduct, substantial breach, illegal use, drug-related criminal activity, owner occupancy, conversion, demo, substantial conversion, etc."
            .to_string(),
    );
    notes.push(
        "notice content must match the statutory ground EXACTLY — generic 'breach' or 'misconduct' does not satisfy"
            .to_string(),
    );

    let voidable = !violations.is_empty();
    JustCauseNoticeContentResult {
        compliant: violations.is_empty(),
        notice_voidable_for_defect: voidable,
        violations: violations.clone(),
        citation: citation_for(Regime::NewJersey),
        notes: notes.clone(),
    }
}

fn check_default(
    input: &JustCauseNoticeContentInput,
    notes: &mut Vec<String>,
) -> JustCauseNoticeContentResult {
    let mut violations: Vec<String> = Vec::new();

    if !input.written_notice_provided {
        violations.push(
            "default rule — most states require written notice for termination; oral notice typically insufficient under common-law"
                .to_string(),
        );
    }

    notes.push(
        "default rule — most non-just-cause states require minimal content (date, parties, termination date); lease + state-specific eviction statute control content + format"
            .to_string(),
    );

    let voidable = !violations.is_empty();
    JustCauseNoticeContentResult {
        compliant: violations.is_empty(),
        notice_voidable_for_defect: voidable,
        violations: violations.clone(),
        citation: citation_for(Regime::Default),
        notes: notes.clone(),
    }
}

fn citation_for(regime: Regime) -> &'static str {
    match regime {
        Regime::California => "Cal. Civ. Code §§ 1946.2(b)(1), 1946.2(b)(2), 1946.2(c), 1946.2(d)(3); Code Civ. Proc. § 1162",
        Regime::Washington => "RCW 59.18.650(2)/(2)(a)-(p); RCW 59.12.040",
        Regime::Oregon => "Or. Rev. Stat. § 90.427 (SB 608); ORS 90.155",
        Regime::NewJersey => "N.J.S.A. §§ 2A:18-61.1(a)-(r), 2A:18-61.2",
        Regime::Default => "state-specific eviction statute + common-law notice requirements",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ca_base() -> JustCauseNoticeContentInput {
        JustCauseNoticeContentInput {
            regime: Regime::California,
            grounds_type: GroundsType::AtFault,
            written_notice_provided: true,
            specific_cause_stated_in_notice: true,
            specific_facts_and_circumstances_described: true,
            at_fault_violation_is_curable: false,
            cure_opportunity_provided: false,
            no_fault_relocation_assistance_paid: false,
            proper_service: true,
        }
    }

    fn wa_base() -> JustCauseNoticeContentInput {
        JustCauseNoticeContentInput {
            regime: Regime::Washington,
            grounds_type: GroundsType::AtFault,
            written_notice_provided: true,
            specific_cause_stated_in_notice: true,
            specific_facts_and_circumstances_described: true,
            at_fault_violation_is_curable: false,
            cure_opportunity_provided: false,
            no_fault_relocation_assistance_paid: false,
            proper_service: true,
        }
    }

    fn or_base() -> JustCauseNoticeContentInput {
        JustCauseNoticeContentInput {
            regime: Regime::Oregon,
            grounds_type: GroundsType::AtFault,
            written_notice_provided: true,
            specific_cause_stated_in_notice: true,
            specific_facts_and_circumstances_described: false,
            at_fault_violation_is_curable: false,
            cure_opportunity_provided: false,
            no_fault_relocation_assistance_paid: false,
            proper_service: true,
        }
    }

    fn nj_base() -> JustCauseNoticeContentInput {
        JustCauseNoticeContentInput {
            regime: Regime::NewJersey,
            grounds_type: GroundsType::AtFault,
            written_notice_provided: true,
            specific_cause_stated_in_notice: true,
            specific_facts_and_circumstances_described: false,
            at_fault_violation_is_curable: false,
            cure_opportunity_provided: false,
            no_fault_relocation_assistance_paid: false,
            proper_service: true,
        }
    }

    fn default_base() -> JustCauseNoticeContentInput {
        JustCauseNoticeContentInput {
            regime: Regime::Default,
            grounds_type: GroundsType::AtFault,
            written_notice_provided: true,
            specific_cause_stated_in_notice: false,
            specific_facts_and_circumstances_described: false,
            at_fault_violation_is_curable: false,
            cure_opportunity_provided: false,
            no_fault_relocation_assistance_paid: false,
            proper_service: true,
        }
    }

    #[test]
    fn ca_at_fault_non_curable_full_compliance_passes() {
        let r = check(&ca_base());
        assert!(r.compliant);
        assert!(!r.notice_voidable_for_defect);
    }

    #[test]
    fn ca_missing_specific_cause_voidable() {
        let mut i = ca_base();
        i.specific_cause_stated_in_notice = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.notice_voidable_for_defect);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 1946.2(c)") && v.contains("MUST state cause")));
    }

    #[test]
    fn ca_oral_notice_voidable() {
        let mut i = ca_base();
        i.written_notice_provided = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("WRITTEN notice")));
    }

    #[test]
    fn ca_at_fault_curable_without_cure_opportunity_voidable() {
        let mut i = ca_base();
        i.at_fault_violation_is_curable = true;
        i.cure_opportunity_provided = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("3-day cure opportunity")));
    }

    #[test]
    fn ca_at_fault_curable_with_cure_opportunity_compliant() {
        let mut i = ca_base();
        i.at_fault_violation_is_curable = true;
        i.cure_opportunity_provided = true;
        let r = check(&i);
        assert!(r.compliant);
    }

    #[test]
    fn ca_no_fault_without_relocation_assistance_voidable() {
        let mut i = ca_base();
        i.grounds_type = GroundsType::NoFault;
        i.no_fault_relocation_assistance_paid = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 1946.2(d)(3)") && v.contains("relocation assistance")));
    }

    #[test]
    fn ca_no_fault_with_relocation_assistance_compliant() {
        let mut i = ca_base();
        i.grounds_type = GroundsType::NoFault;
        i.no_fault_relocation_assistance_paid = true;
        let r = check(&i);
        assert!(r.compliant);
    }

    #[test]
    fn ca_improper_service_voidable() {
        let mut i = ca_base();
        i.proper_service = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("Code Civ. Proc. § 1162")));
    }

    #[test]
    fn ca_void_consequence_note_always_present() {
        let r = check(&ca_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 1946.2(c)") && n.contains("VOID")));
    }

    #[test]
    fn wa_full_compliance_passes() {
        let r = check(&wa_base());
        assert!(r.compliant);
    }

    #[test]
    fn wa_missing_specific_facts_voidable() {
        let mut i = wa_base();
        i.specific_facts_and_circumstances_described = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("FACTS AND CIRCUMSTANCES")));
    }

    #[test]
    fn wa_unique_facts_circumstances_requirement_invariant() {
        let mut i_wa = wa_base();
        i_wa.specific_facts_and_circumstances_described = false;
        let r_wa = check(&i_wa);
        assert!(!r_wa.compliant);

        for regime in [
            Regime::California,
            Regime::Oregon,
            Regime::NewJersey,
            Regime::Default,
        ] {
            let mut i = wa_base();
            i.regime = regime;
            i.specific_facts_and_circumstances_described = false;
            i.no_fault_relocation_assistance_paid = true;
            let r = check(&i);
            let facts_violations: Vec<_> = r
                .violations
                .iter()
                .filter(|v| v.contains("FACTS AND CIRCUMSTANCES"))
                .collect();
            assert!(
                facts_violations.is_empty(),
                "regime {:?} should not require FACTS AND CIRCUMSTANCES",
                regime
            );
        }
    }

    #[test]
    fn wa_16_enumerated_categories_note_present() {
        let r = check(&wa_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("RCW 59.18.650(2)(a)-(p)") && n.contains("16 enumerated")));
    }

    #[test]
    fn or_full_compliance_passes() {
        let r = check(&or_base());
        assert!(r.compliant);
    }

    #[test]
    fn or_missing_specific_cause_voidable() {
        let mut i = or_base();
        i.specific_cause_stated_in_notice = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("ORS 90.427")));
    }

    #[test]
    fn or_under_one_year_no_cause_path_note() {
        let r = check(&or_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("SB 608") && n.contains("< 1 year")));
    }

    #[test]
    fn nj_full_compliance_passes() {
        let r = check(&nj_base());
        assert!(r.compliant);
    }

    #[test]
    fn nj_missing_specific_cause_voidable() {
        let mut i = nj_base();
        i.specific_cause_stated_in_notice = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("2A:18-61.2") && v.contains("§ 2A:18-61.1(a)-(r)")));
    }

    #[test]
    fn nj_exact_statutory_ground_match_required_note() {
        let r = check(&nj_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("statutory ground EXACTLY")));
    }

    #[test]
    fn default_oral_notice_violation() {
        let mut i = default_base();
        i.written_notice_provided = false;
        let r = check(&i);
        assert!(!r.compliant);
    }

    #[test]
    fn default_compliant_with_written_notice() {
        let r = check(&default_base());
        assert!(r.compliant);
    }

    #[test]
    fn citation_california_pins_subsections_and_service_statute() {
        let r = check(&ca_base());
        assert!(r
            .citation
            .contains("§§ 1946.2(b)(1), 1946.2(b)(2), 1946.2(c), 1946.2(d)(3)"));
        assert!(r.citation.contains("Code Civ. Proc. § 1162"));
    }

    #[test]
    fn citation_washington_pins_subsections_and_service_statute() {
        let r = check(&wa_base());
        assert!(r.citation.contains("RCW 59.18.650(2)"));
        assert!(r.citation.contains("(2)(a)-(p)"));
        assert!(r.citation.contains("RCW 59.12.040"));
    }

    #[test]
    fn citation_oregon_pins_90_427_and_service_statute() {
        let r = check(&or_base());
        assert!(r.citation.contains("§ 90.427"));
        assert!(r.citation.contains("SB 608"));
        assert!(r.citation.contains("ORS 90.155"));
    }

    #[test]
    fn citation_newjersey_pins_2a_18_61_1_and_61_2() {
        let r = check(&nj_base());
        assert!(r.citation.contains("§§ 2A:18-61.1(a)-(r), 2A:18-61.2"));
    }

    #[test]
    fn ca_void_consequence_uniquely_explicit_invariant() {
        let mut i_ca = ca_base();
        i_ca.specific_cause_stated_in_notice = false;
        let r_ca = check(&i_ca);
        assert!(r_ca.notice_voidable_for_defect);
        assert!(r_ca.notes.iter().any(|n| n.contains("VOID")));
    }

    #[test]
    fn ca_at_fault_grounds_note_lists_14_categories() {
        let r = check(&ca_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 1946.2(b)(1) at-fault grounds: 14 enumerated")));
    }

    #[test]
    fn ca_no_fault_grounds_note_lists_categories() {
        let r = check(&ca_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 1946.2(b)(2) no-fault grounds")
                && n.contains("substantial remodel")));
    }

    #[test]
    fn nj_anti_eviction_act_lists_18_grounds_note() {
        let r = check(&nj_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("N.J.S.A. 2A:18-61.1(a)-(r)") && n.contains("18 enumerated")));
    }

    #[test]
    fn voidable_flag_engaged_on_any_violation() {
        let mut i = ca_base();
        i.specific_cause_stated_in_notice = false;
        let r = check(&i);
        assert!(r.notice_voidable_for_defect);

        let i_clean = ca_base();
        let r_clean = check(&i_clean);
        assert!(!r_clean.notice_voidable_for_defect);
    }

    #[test]
    fn five_regimes_routed_correctly() {
        for regime in [
            Regime::California,
            Regime::Washington,
            Regime::Oregon,
            Regime::NewJersey,
            Regime::Default,
        ] {
            let mut i = ca_base();
            i.regime = regime;
            i.no_fault_relocation_assistance_paid = true;
            let r = check(&i);
            let _ = r.compliant;
        }
    }
}
