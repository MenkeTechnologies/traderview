//! IRC § 7811 — Taxpayer Assistance Orders (TAOs). Trader-relevant
//! when an IRS administrative action (levy, collection, audit
//! delay, refund processing failure) is causing or about to cause
//! significant hardship to the taxpayer. The National Taxpayer
//! Advocate (NTA) has independent authority under § 7811 to order
//! the IRS to release property, take action, or refrain from
//! taking action.
//!
//! Distinct from `section_6330` (CDP for levies — pre-action 30-
//! day notice + Tax Court review), `section_6320` (CDP for liens),
//! `section_7430` (attorney fees against IRS — post-judgment fee
//! shifting), and `section_6402` (refund offsets). § 7811 is an
//! ADMINISTRATIVE EQUITABLE remedy available in parallel with
//! these judicial pathways.
//!
//! § 7811(a)(1) GENERAL RULE — upon application filed by a
//! taxpayer with the Office of the Taxpayer Advocate (in such form
//! as the Secretary shall prescribe), the National Taxpayer
//! Advocate may issue a Taxpayer Assistance Order if the NTA
//! determines that the taxpayer is suffering or about to suffer a
//! significant hardship as a result of the manner in which the
//! internal revenue laws are being administered by the IRS,
//! including action or inaction on the part of the IRS.
//!
//! § 7811(a)(2) "SIGNIFICANT HARDSHIP" — defined to include (but
//! not limited to):
//!   (A) immediate threat of adverse action
//!   (B) delay of more than 30 days in resolving taxpayer
//!       account problems
//!   (C) incurring of significant costs (including fees for
//!       professional representation) if relief is not granted
//!   (D) irreparable injury to, or long-term adverse impact on,
//!       the taxpayer if relief is not granted
//!
//! § 7811(b) TERMS OF ORDER — TAO may require the Secretary to
//! (1) release property of the taxpayer levied upon, OR (2)
//! cease any action, take any action authorized by law, or
//! refrain from taking any action, with respect to the taxpayer.
//!
//! § 7811(c) AUTHORITY TO MODIFY OR RESCIND — a TAO issued by
//! the NTA may be modified or rescinded ONLY BY (1) the National
//! Taxpayer Advocate, (2) the Commissioner of Internal Revenue,
//! or (3) the Deputy Commissioner. No other IRS official has
//! authority to override a TAO; this is the load-bearing
//! independence-from-IRS feature of the statute.
//!
//! § 7811(d) SUSPENSION OF RUNNING OF PERIOD OF LIMITATION — the
//! running of any period of limitation with respect to any
//! action described in subsection (b) shall be SUSPENDED for the
//! period beginning on the date of the taxpayer's application
//! under subsection (a) and ending on the date of the NTA's
//! decision with respect to such application, AND any period
//! specified by the NTA in a TAO issued pursuant to such
//! application.
//!
//! § 7811(e) INDEPENDENT OF OTHER RELIEF — TAO is independent of
//! and does not preclude other administrative or judicial
//! remedies (including CDP under § 6320 / § 6330, Tax Court
//! petition under § 6213, refund litigation, and § 7430 fee
//! shifting).
//!
//! Form 911 — "Request for Taxpayer Advocate Service Assistance
//! (And Application for Taxpayer Assistance Order)" — the
//! prescribed application form per IRS IRM 13.1.20.
//!
//! Citations: IRC § 7811(a)(1) (general TAO authority); § 7811(a)
//! (2)(A)/(B)/(C)/(D) (significant hardship four enumerated
//! categories); § 7811(b)(1)/(b)(2) (release of property / cease
//! or take action); § 7811(c)(1)/(c)(2)/(c)(3) (modification or
//! rescission limited to NTA / Commissioner / Deputy
//! Commissioner); § 7811(d) (suspension of statute of
//! limitations); § 7811(e) (independence from other remedies);
//! 26 CFR § 301.7811-1 (regulations); IRS IRM 13.1.20 (TAO
//! procedures); Form 911 (application).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HardshipCategory {
    /// § 7811(a)(2)(A) — immediate threat of adverse action.
    ImmediateAdverseAction,
    /// § 7811(a)(2)(B) — delay of more than 30 days in resolving
    /// taxpayer account problems.
    Delay30PlusDays,
    /// § 7811(a)(2)(C) — significant costs including professional
    /// representation fees.
    SignificantCosts,
    /// § 7811(a)(2)(D) — irreparable injury or long-term adverse
    /// impact.
    IrreparableInjury,
    /// Other category not enumerated in § 7811(a)(2) — NTA may
    /// still find significant hardship since list is non-exclusive.
    OtherHardship,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RequestedAction {
    /// § 7811(b)(1) — release of property levied upon.
    ReleaseLeviedProperty,
    /// § 7811(b)(2) — cease IRS action.
    CeaseAction,
    /// § 7811(b)(2) — take a specified action authorized by law.
    TakeAction,
    /// § 7811(b)(2) — refrain from taking a specified action.
    RefrainFromAction,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ModifyingOfficial {
    /// § 7811(c)(1) — National Taxpayer Advocate.
    NationalTaxpayerAdvocate,
    /// § 7811(c)(2) — Commissioner of Internal Revenue.
    Commissioner,
    /// § 7811(c)(3) — Deputy Commissioner.
    DeputyCommissioner,
    /// Other IRS official — lacks § 7811(c) authority to modify
    /// or rescind a TAO.
    Other,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section7811Input {
    pub hardship_category: HardshipCategory,
    /// Whether the taxpayer submitted Form 911 (the prescribed
    /// application form).
    pub form_911_submitted: bool,
    pub requested_action: RequestedAction,
    /// Whether the NTA issued a TAO based on the application.
    pub tao_issued: bool,
    /// Whether the IRS has attempted to modify or rescind the TAO.
    pub modification_attempted: bool,
    /// The IRS official attempting to modify or rescind (if any).
    pub modifying_official: ModifyingOfficial,
    /// Whether a CDP hearing under § 6320 / § 6330 is also
    /// pending (drives § 7811(e) independence-from-other-relief
    /// note).
    pub parallel_cdp_pending: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section7811Result {
    pub tao_pathway_available: bool,
    pub statute_of_limitations_suspended: bool,
    pub modification_or_rescission_valid: bool,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn compute(input: &Section7811Input) -> Section7811Result {
    let mut notes: Vec<String> = Vec::new();

    if !input.form_911_submitted {
        notes.push(
            "26 CFR § 301.7811-1 — application must be made on Form 911 (Request for Taxpayer Advocate Service Assistance) or in a written statement providing sufficient information for TAS to determine the nature of the harm or need"
                .to_string(),
        );
        return Section7811Result {
            tao_pathway_available: false,
            statute_of_limitations_suspended: false,
            modification_or_rescission_valid: false,
            citation: citation(),
            notes,
        };
    }

    let hardship_note = match input.hardship_category {
        HardshipCategory::ImmediateAdverseAction => {
            "§ 7811(a)(2)(A) — immediate threat of adverse action satisfies significant hardship"
        }
        HardshipCategory::Delay30PlusDays => {
            "§ 7811(a)(2)(B) — delay of more than 30 days in resolving account problems satisfies significant hardship"
        }
        HardshipCategory::SignificantCosts => {
            "§ 7811(a)(2)(C) — significant costs including professional representation fees satisfies significant hardship"
        }
        HardshipCategory::IrreparableInjury => {
            "§ 7811(a)(2)(D) — irreparable injury or long-term adverse impact satisfies significant hardship"
        }
        HardshipCategory::OtherHardship => {
            "hardship category not within § 7811(a)(2) enumerated list — NTA may still find significant hardship since list is non-exclusive (\"includes\" language)"
        }
    };
    notes.push(hardship_note.to_string());

    let action_note = match input.requested_action {
        RequestedAction::ReleaseLeviedProperty => {
            "§ 7811(b)(1) — TAO may require release of property of taxpayer levied upon"
        }
        RequestedAction::CeaseAction => {
            "§ 7811(b)(2) — TAO may require IRS to cease specified action"
        }
        RequestedAction::TakeAction => {
            "§ 7811(b)(2) — TAO may require IRS to take action authorized by law"
        }
        RequestedAction::RefrainFromAction => {
            "§ 7811(b)(2) — TAO may require IRS to refrain from taking specified action"
        }
    };
    notes.push(action_note.to_string());

    notes.push(
        "§ 7811(d) — statute of limitations on any § 7811(b) action SUSPENDED from application date through NTA decision date plus any period specified in the TAO"
            .to_string(),
    );

    if input.parallel_cdp_pending {
        notes.push(
            "§ 7811(e) — TAO is INDEPENDENT of other administrative or judicial remedies; parallel CDP proceeding under § 6320 / § 6330 does not preclude TAO relief"
                .to_string(),
        );
    }

    let modification_valid = if input.modification_attempted {
        let valid_official = matches!(
            input.modifying_official,
            ModifyingOfficial::NationalTaxpayerAdvocate
                | ModifyingOfficial::Commissioner
                | ModifyingOfficial::DeputyCommissioner
        );
        if valid_official {
            notes.push(
                "§ 7811(c) — modification or rescission by authorized official (NTA / Commissioner / Deputy Commissioner) — valid"
                    .to_string(),
            );
        } else {
            notes.push(
                "§ 7811(c) — modification or rescission attempt by official other than NTA / Commissioner / Deputy Commissioner — INVALID; no other IRS official has § 7811(c) authority to override a TAO"
                    .to_string(),
            );
        }
        valid_official
    } else {
        false
    };

    Section7811Result {
        tao_pathway_available: true,
        statute_of_limitations_suspended: true,
        modification_or_rescission_valid: modification_valid,
        citation: citation(),
        notes,
    }
}

fn citation() -> &'static str {
    "IRC § 7811(a)(1)/(a)(2)(A)/(B)/(C)/(D)/(b)(1)/(b)(2)/(c)(1)/(c)(2)/(c)(3)/(d)/(e); 26 CFR § 301.7811-1; IRS IRM 13.1.20; Form 911"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Section7811Input {
        Section7811Input {
            hardship_category: HardshipCategory::Delay30PlusDays,
            form_911_submitted: true,
            requested_action: RequestedAction::ReleaseLeviedProperty,
            tao_issued: false,
            modification_attempted: false,
            modifying_official: ModifyingOfficial::Other,
            parallel_cdp_pending: false,
        }
    }

    #[test]
    fn form_911_with_significant_hardship_opens_tao_pathway() {
        let r = compute(&base());
        assert!(r.tao_pathway_available);
        assert!(r.statute_of_limitations_suspended);
    }

    #[test]
    fn no_form_911_no_tao_pathway() {
        let mut i = base();
        i.form_911_submitted = false;
        let r = compute(&i);
        assert!(!r.tao_pathway_available);
        assert!(!r.statute_of_limitations_suspended);
        assert!(r.notes.iter().any(|n| n.contains("Form 911")));
    }

    #[test]
    fn immediate_adverse_action_hardship_note_present() {
        let mut i = base();
        i.hardship_category = HardshipCategory::ImmediateAdverseAction;
        let r = compute(&i);
        assert!(r.notes.iter().any(|n| n.contains("§ 7811(a)(2)(A)") && n.contains("immediate threat")));
    }

    #[test]
    fn delay_30_plus_days_hardship_note_present() {
        let mut i = base();
        i.hardship_category = HardshipCategory::Delay30PlusDays;
        let r = compute(&i);
        assert!(r.notes.iter().any(|n| n.contains("§ 7811(a)(2)(B)") && n.contains("30 days")));
    }

    #[test]
    fn significant_costs_hardship_note_present() {
        let mut i = base();
        i.hardship_category = HardshipCategory::SignificantCosts;
        let r = compute(&i);
        assert!(r.notes.iter().any(|n| n.contains("§ 7811(a)(2)(C)") && n.contains("professional representation")));
    }

    #[test]
    fn irreparable_injury_hardship_note_present() {
        let mut i = base();
        i.hardship_category = HardshipCategory::IrreparableInjury;
        let r = compute(&i);
        assert!(r.notes.iter().any(|n| n.contains("§ 7811(a)(2)(D)") && n.contains("irreparable injury")));
    }

    #[test]
    fn other_hardship_category_still_engages_nta_authority() {
        let mut i = base();
        i.hardship_category = HardshipCategory::OtherHardship;
        let r = compute(&i);
        assert!(r.tao_pathway_available);
        assert!(r.notes.iter().any(|n| n.contains("non-exclusive") || n.contains("includes")));
    }

    #[test]
    fn release_levied_property_action_note() {
        let r = compute(&base());
        assert!(r.notes.iter().any(|n| n.contains("§ 7811(b)(1)") && n.contains("release of property")));
    }

    #[test]
    fn cease_action_note() {
        let mut i = base();
        i.requested_action = RequestedAction::CeaseAction;
        let r = compute(&i);
        assert!(r.notes.iter().any(|n| n.contains("§ 7811(b)(2)") && n.contains("cease")));
    }

    #[test]
    fn take_action_note() {
        let mut i = base();
        i.requested_action = RequestedAction::TakeAction;
        let r = compute(&i);
        assert!(r.notes.iter().any(|n| n.contains("§ 7811(b)(2)") && n.contains("take action authorized by law")));
    }

    #[test]
    fn refrain_from_action_note() {
        let mut i = base();
        i.requested_action = RequestedAction::RefrainFromAction;
        let r = compute(&i);
        assert!(r.notes.iter().any(|n| n.contains("§ 7811(b)(2)") && n.contains("refrain")));
    }

    #[test]
    fn statute_of_limitations_suspension_engaged_with_tao_pathway() {
        let r = compute(&base());
        assert!(r.statute_of_limitations_suspended);
        assert!(r.notes.iter().any(|n| n.contains("§ 7811(d)") && n.contains("SUSPENDED")));
    }

    #[test]
    fn parallel_cdp_triggers_independence_note() {
        let mut i = base();
        i.parallel_cdp_pending = true;
        let r = compute(&i);
        assert!(r.notes.iter().any(|n| n.contains("§ 7811(e)") && n.contains("INDEPENDENT")));
    }

    #[test]
    fn no_parallel_cdp_no_independence_note() {
        let r = compute(&base());
        let independence_notes: Vec<_> = r.notes.iter().filter(|n| n.contains("§ 7811(e)")).collect();
        assert!(independence_notes.is_empty());
    }

    #[test]
    fn modification_by_nta_valid() {
        let mut i = base();
        i.modification_attempted = true;
        i.modifying_official = ModifyingOfficial::NationalTaxpayerAdvocate;
        let r = compute(&i);
        assert!(r.modification_or_rescission_valid);
        assert!(r.notes.iter().any(|n| n.contains("§ 7811(c)") && n.contains("valid")));
    }

    #[test]
    fn modification_by_commissioner_valid() {
        let mut i = base();
        i.modification_attempted = true;
        i.modifying_official = ModifyingOfficial::Commissioner;
        let r = compute(&i);
        assert!(r.modification_or_rescission_valid);
    }

    #[test]
    fn modification_by_deputy_commissioner_valid() {
        let mut i = base();
        i.modification_attempted = true;
        i.modifying_official = ModifyingOfficial::DeputyCommissioner;
        let r = compute(&i);
        assert!(r.modification_or_rescission_valid);
    }

    #[test]
    fn modification_by_other_official_invalid() {
        let mut i = base();
        i.modification_attempted = true;
        i.modifying_official = ModifyingOfficial::Other;
        let r = compute(&i);
        assert!(!r.modification_or_rescission_valid);
        assert!(r.notes.iter().any(|n| n.contains("§ 7811(c)") && n.contains("INVALID")));
    }

    #[test]
    fn no_modification_attempt_no_modification_note() {
        let r = compute(&base());
        assert!(!r.modification_or_rescission_valid);
        let modification_notes: Vec<_> = r.notes.iter().filter(|n| n.contains("§ 7811(c)")).collect();
        assert!(modification_notes.is_empty());
    }

    #[test]
    fn citation_pins_all_subsections_and_authorities() {
        let r = compute(&base());
        assert!(r.citation.contains("§ 7811(a)(1)"));
        assert!(r.citation.contains("(a)(2)(A)/(B)/(C)/(D)"));
        assert!(r.citation.contains("(b)(1)"));
        assert!(r.citation.contains("(b)(2)"));
        assert!(r.citation.contains("(c)(1)/(c)(2)/(c)(3)"));
        assert!(r.citation.contains("(d)"));
        assert!(r.citation.contains("(e)"));
        assert!(r.citation.contains("§ 301.7811-1"));
        assert!(r.citation.contains("IRM 13.1.20"));
        assert!(r.citation.contains("Form 911"));
    }

    #[test]
    fn only_three_officials_uniquely_have_modification_authority_invariant() {
        let valid_officials = [
            ModifyingOfficial::NationalTaxpayerAdvocate,
            ModifyingOfficial::Commissioner,
            ModifyingOfficial::DeputyCommissioner,
        ];
        for official in valid_officials {
            let mut i = base();
            i.modification_attempted = true;
            i.modifying_official = official;
            let r = compute(&i);
            assert!(r.modification_or_rescission_valid, "official {:?} should be valid", official);
        }
        let mut i_other = base();
        i_other.modification_attempted = true;
        i_other.modifying_official = ModifyingOfficial::Other;
        let r_other = compute(&i_other);
        assert!(!r_other.modification_or_rescission_valid);
    }

    #[test]
    fn no_form_911_suspends_all_pathway_outputs() {
        let mut i = base();
        i.form_911_submitted = false;
        let r = compute(&i);
        assert!(!r.tao_pathway_available);
        assert!(!r.statute_of_limitations_suspended);
        assert!(!r.modification_or_rescission_valid);
    }

    #[test]
    fn five_hardship_categories_all_engage_tao_pathway() {
        for cat in [
            HardshipCategory::ImmediateAdverseAction,
            HardshipCategory::Delay30PlusDays,
            HardshipCategory::SignificantCosts,
            HardshipCategory::IrreparableInjury,
            HardshipCategory::OtherHardship,
        ] {
            let mut i = base();
            i.hardship_category = cat;
            let r = compute(&i);
            assert!(r.tao_pathway_available, "hardship category {:?} should engage pathway", cat);
        }
    }

    #[test]
    fn four_requested_actions_all_engage_action_note() {
        for action in [
            RequestedAction::ReleaseLeviedProperty,
            RequestedAction::CeaseAction,
            RequestedAction::TakeAction,
            RequestedAction::RefrainFromAction,
        ] {
            let mut i = base();
            i.requested_action = action;
            let r = compute(&i);
            assert!(r.tao_pathway_available);
            let action_notes: Vec<_> = r.notes.iter().filter(|n| n.contains("§ 7811(b)")).collect();
            assert!(!action_notes.is_empty(), "action {:?} should surface § 7811(b) note", action);
        }
    }

    #[test]
    fn other_hardship_non_exclusive_list_note() {
        let mut i = base();
        i.hardship_category = HardshipCategory::OtherHardship;
        let r = compute(&i);
        assert!(r.notes.iter().any(|n| n.contains("non-exclusive")));
    }

    #[test]
    fn parallel_cdp_with_release_property_action_combines_correctly() {
        let mut i = base();
        i.parallel_cdp_pending = true;
        i.requested_action = RequestedAction::ReleaseLeviedProperty;
        let r = compute(&i);
        assert!(r.notes.iter().any(|n| n.contains("§ 7811(b)(1)")));
        assert!(r.notes.iter().any(|n| n.contains("§ 7811(e)")));
    }

    #[test]
    fn statute_suspension_note_includes_application_through_decision_period() {
        let r = compute(&base());
        assert!(r.notes.iter().any(|n| n.contains("application date through NTA decision date")));
    }

    #[test]
    fn modification_by_other_when_attempted_falsifies_validity() {
        let mut i = base();
        i.modification_attempted = true;
        i.modifying_official = ModifyingOfficial::Other;
        let r = compute(&i);
        assert!(!r.modification_or_rescission_valid);
        assert!(r.tao_pathway_available, "underlying TAO pathway remains valid");
    }
}
