//! Federal SCRA + state military lease termination rights — tenth
//! state-data module after `deposit_interest`, `late_fee_caps`,
//! `eviction_notices`, `contractor_1099`, `deposit_return_windows`,
//! `lease_disclosures`, `rent_control`, `habitability_remedies`, and
//! `security_deposit_caps`.
//!
//! **Federal Servicemembers Civil Relief Act** (SCRA) — 50 USC
//! §3955 — applies in every state. A servicemember can terminate
//! any residential lease for any of three qualifying events:
//!
//!   1. Permanent change of station (PCS) orders.
//!   2. Deployment with military unit for ≥ 90 days.
//!   3. Active duty AFTER lease signing (entry into active service
//!      from reserve or new enlistment).
//!
//! Federal mechanics:
//!   * **Written notice** to landlord with copy of orders.
//!   * Termination effective on **the next rent-due date 30+ days
//!     AFTER notice is delivered** (effectively a 30-day notice).
//!   * Landlord CANNOT charge an early-termination fee.
//!   * Landlord MUST return security deposit minus actual damages
//!     within the state's standard timeframe.
//!   * **Civil penalty**: up to **$55,000** for first violation,
//!     $110,000 thereafter, plus equitable relief + tenant's
//!     actual damages.
//!
//! **State protections layered on top**: every state EXTENDS the
//! federal SCRA floor in various ways:
//!
//!   * **Spouse / dependent termination right** — CA, FL, VA, NC, TX,
//!     others. Civilian spouse may also terminate when the
//!     servicemember PCSs or deploys.
//!   * **First responders** — TX extends to peace officers,
//!     firefighters, EMS.
//!   * **Survivors of domestic violence** — IL, WA, CO, several
//!     others extend the same termination mechanic to DV victims
//!     (not modeled here — separate category).
//!
//! Pure data + compute. Caller passes the state + qualifying-event
//! facts; we return whether the tenant has the federal SCRA right,
//! whether state extensions apply, and the notice requirements.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QualifyingEvent {
    /// SCRA §3955(b)(1)(A) — PCS orders.
    PermanentChangeOfStation,
    /// SCRA §3955(b)(1)(B) — deployment ≥ 90 days.
    DeploymentNinetyDaysOrMore,
    /// SCRA §3955(b)(1)(C) — active duty after lease signing.
    ActiveDutyAfterLeaseSigning,
    /// State-only: civilian first responder reassignment (TX).
    FirstResponderReassignment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TenantRole {
    /// Active-duty servicemember.
    Servicemember,
    /// Spouse of a servicemember.
    Spouse,
    /// Dependent of a servicemember.
    Dependent,
    /// Civilian first responder (TX extension).
    FirstResponder,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    pub statute: &'static str,
    pub source: &'static str,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateExtension {
    pub state: &'static str,
    /// True when state extends federal SCRA termination right to spouses.
    pub spouse_termination_right: bool,
    /// True when state extends to dependents.
    pub dependent_termination_right: bool,
    /// True when state extends to non-military first responders
    /// (peace officers, firefighters, EMS) — TX only.
    pub first_responder_termination_right: bool,
    /// Days of notice required by STATE law (vs federal 30). Most
    /// state laws match federal 30; a few impose 14.
    pub state_notice_days_required: u32,
    pub notes: &'static str,
    pub citation: Citation,
}

fn state_extensions() -> &'static [StateExtension] {
    static R: once_cell::sync::Lazy<Vec<StateExtension>> = once_cell::sync::Lazy::new(|| {
        vec![
                StateExtension {
                    state: "CA",
                    spouse_termination_right: true,
                    dependent_termination_right: true,
                    first_responder_termination_right: false,
                    state_notice_days_required: 30,
                    notes: "Cal. Mil. & Vet. Code §400: SCRA + extends termination right to spouse and dependents.",
                    citation: Citation {
                        statute: "Cal. Mil. & Vet. Code §400",
                        source: "https://leginfo.legislature.ca.gov/faces/codes_displaySection.xhtml?sectionNum=400.&lawCode=MVC",
                    },
                },
                StateExtension {
                    state: "NY",
                    spouse_termination_right: true,
                    dependent_termination_right: false,
                    first_responder_termination_right: false,
                    state_notice_days_required: 30,
                    notes: "N.Y. Mil. Law §310 + §319: SCRA + spouse termination right.",
                    citation: Citation {
                        statute: "N.Y. Military Law §310 + §319",
                        source: "https://www.nysenate.gov/legislation/laws/MIL/310",
                    },
                },
                StateExtension {
                    state: "TX",
                    spouse_termination_right: true,
                    dependent_termination_right: true,
                    first_responder_termination_right: true,
                    state_notice_days_required: 30,
                    notes: "Tex. Prop. Code §92.017: SCRA + extends to spouse + dependents + peace officers + firefighters + EMS (first responders).",
                    citation: Citation {
                        statute: "Tex. Prop. Code §92.017",
                        source: "https://statutes.capitol.texas.gov/Docs/PR/htm/PR.92.htm",
                    },
                },
                StateExtension {
                    state: "FL",
                    spouse_termination_right: true,
                    dependent_termination_right: false,
                    first_responder_termination_right: false,
                    state_notice_days_required: 30,
                    notes: "Fla. Stat. §83.682: SCRA + extends termination right to spouse.",
                    citation: Citation {
                        statute: "Fla. Stat. §83.682",
                        source: "https://www.flsenate.gov/Laws/Statutes/2024/0083.682",
                    },
                },
                StateExtension {
                    state: "VA",
                    spouse_termination_right: true,
                    dependent_termination_right: true,
                    first_responder_termination_right: false,
                    state_notice_days_required: 30,
                    notes: "Va. Code §55.1-1235: SCRA + spouse and dependent termination right.",
                    citation: Citation {
                        statute: "Va. Code §55.1-1235",
                        source: "https://law.lis.virginia.gov/vacode/title55.1/chapter12/section55.1-1235/",
                    },
                },
                StateExtension {
                    state: "WA",
                    spouse_termination_right: false,
                    dependent_termination_right: false,
                    first_responder_termination_right: false,
                    state_notice_days_required: 30,
                    notes: "RCW 59.18.220: SCRA termination right + (separately) survivors of domestic violence per RCW 59.18.575.",
                    citation: Citation {
                        statute: "RCW 59.18.220",
                        source: "https://app.leg.wa.gov/rcw/default.aspx?cite=59.18.220",
                    },
                },
                StateExtension {
                    state: "IL",
                    spouse_termination_right: true,
                    dependent_termination_right: false,
                    first_responder_termination_right: false,
                    state_notice_days_required: 30,
                    notes: "765 ILCS 740: SCRA + spouse termination + DV survivors separately under 765 ILCS 750.",
                    citation: Citation {
                        statute: "765 ILCS 740",
                        source: "https://www.ilga.gov/legislation/ilcs/ilcs3.asp?ActID=2235",
                    },
                },
                StateExtension {
                    state: "CO",
                    spouse_termination_right: true,
                    dependent_termination_right: false,
                    first_responder_termination_right: false,
                    state_notice_days_required: 30,
                    notes: "C.R.S. §38-12-1102: SCRA + spouse termination right + DV survivors separately.",
                    citation: Citation {
                        statute: "C.R.S. §38-12-1102",
                        source: "https://leg.colorado.gov/sites/default/files/images/olls/crs2024-title-38.pdf",
                    },
                },
                StateExtension {
                    state: "NJ",
                    spouse_termination_right: false,
                    dependent_termination_right: false,
                    first_responder_termination_right: false,
                    state_notice_days_required: 30,
                    notes: "N.J.S.A. 38:23C-20: codifies SCRA at state level.",
                    citation: Citation {
                        statute: "N.J. Stat. §38:23C-20",
                        source: "https://lis.njleg.state.nj.us/nxt/gateway.dll?f=templates&fn=default.htm",
                    },
                },
                StateExtension {
                    state: "NC",
                    spouse_termination_right: true,
                    dependent_termination_right: true,
                    first_responder_termination_right: false,
                    state_notice_days_required: 30,
                    notes: "N.C.G.S. §42-45: SCRA + extends to spouse and dependents living with servicemember.",
                    citation: Citation {
                        statute: "N.C.G.S. §42-45",
                        source: "https://www.ncleg.gov/EnactedLegislation/Statutes/HTML/BySection/Chapter_42/GS_42-45.html",
                    },
                },
                StateExtension {
                    state: "PA",
                    spouse_termination_right: false,
                    dependent_termination_right: false,
                    first_responder_termination_right: false,
                    state_notice_days_required: 30,
                    notes: "51 Pa. C.S. §4106: codifies SCRA + (separately) victims of stalking under 68 P.S. §250.512-a.",
                    citation: Citation {
                        statute: "51 Pa. C.S. §4106",
                        source: "https://www.legis.state.pa.us/cfdocs/legis/LI/consCheck.cfm?txtType=HTM&ttl=51&div=00.&chpt=41.",
                    },
                },
            ]
    });
    &R
}

pub fn state_extension_for(state: &str) -> Option<&'static StateExtension> {
    let upper = state.to_uppercase();
    state_extensions()
        .iter()
        .find(|r| r.state.eq_ignore_ascii_case(&upper))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilitaryTerminationCheckInput {
    pub state: String,
    pub tenant_role: TenantRole,
    pub qualifying_event: QualifyingEvent,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MilitaryTerminationCheckResult {
    pub federal_scra_applies: bool,
    pub state_recognized: bool,
    pub state_extension_applies: bool,
    pub termination_right_available: bool,
    pub notice_days_required: u32,
    pub max_civil_penalty_first_violation_usd: u64,
    pub controlling_authority: String,
    pub statute: String,
    pub source: String,
    pub notes: String,
}

pub fn check(input: &MilitaryTerminationCheckInput) -> MilitaryTerminationCheckResult {
    let mut r = MilitaryTerminationCheckResult {
        notice_days_required: 30,
        max_civil_penalty_first_violation_usd: 55_000,
        ..MilitaryTerminationCheckResult::default()
    };

    // Federal SCRA: applies in every state to active-duty servicemembers
    // for the three qualifying events listed in 50 USC §3955(b).
    let federal_qualifies = matches!(
        input.qualifying_event,
        QualifyingEvent::PermanentChangeOfStation
            | QualifyingEvent::DeploymentNinetyDaysOrMore
            | QualifyingEvent::ActiveDutyAfterLeaseSigning
    );
    let federal_role_qualifies = matches!(input.tenant_role, TenantRole::Servicemember);

    r.federal_scra_applies = federal_qualifies && federal_role_qualifies;

    // State extension lookup.
    let extension = state_extension_for(&input.state);
    r.state_recognized = extension.is_some();

    if let Some(ext) = extension {
        r.statute = ext.citation.statute.into();
        r.source = ext.citation.source.into();
        r.notes = ext.notes.into();
        r.notice_days_required = ext.state_notice_days_required;

        // State extension paths:
        let state_extends_to_role = match input.tenant_role {
            TenantRole::Spouse => ext.spouse_termination_right,
            TenantRole::Dependent => ext.dependent_termination_right,
            TenantRole::FirstResponder => ext.first_responder_termination_right,
            TenantRole::Servicemember => false, // not an extension
        };
        let state_extends_to_event = matches!(
            input.qualifying_event,
            QualifyingEvent::FirstResponderReassignment
        ) && ext.first_responder_termination_right;
        r.state_extension_applies = state_extends_to_role || state_extends_to_event;
    } else {
        r.notes = format!(
            "no state extension on file for {} — federal SCRA still applies if servicemember + qualifying event",
            input.state.to_uppercase()
        );
        r.statute = "50 U.S.C. §3955 (federal SCRA)".into();
        r.source = "https://www.law.cornell.edu/uscode/text/50/3955".into();
    }

    r.termination_right_available = r.federal_scra_applies || r.state_extension_applies;
    r.controlling_authority = if r.federal_scra_applies {
        "50 U.S.C. §3955 (SCRA)".into()
    } else if r.state_extension_applies {
        format!("state extension ({})", r.statute)
    } else {
        "neither federal SCRA nor state extension applies".into()
    };
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> MilitaryTerminationCheckInput {
        MilitaryTerminationCheckInput {
            state: "CA".into(),
            tenant_role: TenantRole::Servicemember,
            qualifying_event: QualifyingEvent::PermanentChangeOfStation,
        }
    }

    #[test]
    fn federal_scra_pcs_servicemember_qualifies() {
        let r = check(&base());
        assert!(r.federal_scra_applies);
        assert!(r.termination_right_available);
        assert_eq!(r.notice_days_required, 30);
        assert_eq!(r.max_civil_penalty_first_violation_usd, 55_000);
    }

    #[test]
    fn federal_scra_deployment_90_days_qualifies() {
        let mut i = base();
        i.qualifying_event = QualifyingEvent::DeploymentNinetyDaysOrMore;
        let r = check(&i);
        assert!(r.federal_scra_applies);
    }

    #[test]
    fn federal_scra_active_duty_after_signing_qualifies() {
        let mut i = base();
        i.qualifying_event = QualifyingEvent::ActiveDutyAfterLeaseSigning;
        let r = check(&i);
        assert!(r.federal_scra_applies);
    }

    #[test]
    fn ca_spouse_extension_applies() {
        let mut i = base();
        i.tenant_role = TenantRole::Spouse;
        let r = check(&i);
        // Federal doesn't extend to spouse.
        assert!(!r.federal_scra_applies);
        // CA state extension does.
        assert!(r.state_extension_applies);
        assert!(r.termination_right_available);
    }

    #[test]
    fn ca_dependent_extension_applies() {
        let mut i = base();
        i.tenant_role = TenantRole::Dependent;
        let r = check(&i);
        assert!(r.state_extension_applies);
    }

    #[test]
    fn tx_first_responder_extension_unique() {
        let mut i = base();
        i.state = "TX".into();
        i.tenant_role = TenantRole::FirstResponder;
        i.qualifying_event = QualifyingEvent::FirstResponderReassignment;
        let r = check(&i);
        assert!(r.state_extension_applies);
        assert!(r.termination_right_available);
    }

    #[test]
    fn ca_first_responder_not_extended_unlike_tx() {
        let mut i = base();
        i.state = "CA".into();
        i.tenant_role = TenantRole::FirstResponder;
        i.qualifying_event = QualifyingEvent::FirstResponderReassignment;
        let r = check(&i);
        assert!(!r.state_extension_applies);
        assert!(!r.federal_scra_applies);
        assert!(!r.termination_right_available);
    }

    #[test]
    fn nj_codifies_scra_no_extra_extensions() {
        let mut i = base();
        i.state = "NJ".into();
        i.tenant_role = TenantRole::Spouse;
        let r = check(&i);
        // NJ doesn't extend to spouse.
        assert!(!r.state_extension_applies);
    }

    #[test]
    fn nj_servicemember_pcs_federal_still_applies() {
        let mut i = base();
        i.state = "NJ".into();
        let r = check(&i);
        assert!(r.federal_scra_applies);
        assert!(r.termination_right_available);
    }

    #[test]
    fn unknown_state_federal_scra_still_applies() {
        let mut i = base();
        i.state = "XX".into();
        let r = check(&i);
        assert!(!r.state_recognized);
        assert!(r.federal_scra_applies); // federal still applies
        assert!(r.notes.contains("no state extension"));
    }

    #[test]
    fn unknown_state_spouse_no_termination_right() {
        // No federal extension to spouse + no state on file = no right.
        let mut i = base();
        i.state = "XX".into();
        i.tenant_role = TenantRole::Spouse;
        let r = check(&i);
        assert!(!r.termination_right_available);
    }

    #[test]
    fn case_insensitive_state_lookup() {
        let mut i = base();
        i.state = "ca".into();
        let r = check(&i);
        assert!(r.state_recognized);
    }

    #[test]
    fn nc_extends_to_spouse_and_dependent() {
        let mut i = base();
        i.state = "NC".into();
        i.tenant_role = TenantRole::Spouse;
        let r_spouse = check(&i);
        assert!(r_spouse.state_extension_applies);

        i.tenant_role = TenantRole::Dependent;
        let r_dep = check(&i);
        assert!(r_dep.state_extension_applies);
    }

    #[test]
    fn ny_extends_to_spouse_but_not_dependent() {
        let mut i = base();
        i.state = "NY".into();
        i.tenant_role = TenantRole::Spouse;
        let r_spouse = check(&i);
        assert!(r_spouse.state_extension_applies);

        i.tenant_role = TenantRole::Dependent;
        let r_dep = check(&i);
        assert!(!r_dep.state_extension_applies);
    }

    #[test]
    fn fl_spouse_extension_applies() {
        let mut i = base();
        i.state = "FL".into();
        i.tenant_role = TenantRole::Spouse;
        let r = check(&i);
        assert!(r.state_extension_applies);
    }

    #[test]
    fn va_dependent_extension_applies() {
        let mut i = base();
        i.state = "VA".into();
        i.tenant_role = TenantRole::Dependent;
        let r = check(&i);
        assert!(r.state_extension_applies);
    }

    #[test]
    fn wa_servicemember_pcs_qualifies_under_federal() {
        // WA recognized but doesn't extend to spouse/dep. Servicemember
        // under federal still works.
        let mut i = base();
        i.state = "WA".into();
        let r = check(&i);
        assert!(r.federal_scra_applies);
        assert!(r.termination_right_available);
    }

    #[test]
    fn wa_spouse_no_state_extension_no_federal_no_right() {
        let mut i = base();
        i.state = "WA".into();
        i.tenant_role = TenantRole::Spouse;
        let r = check(&i);
        // WA doesn't extend to spouse, federal doesn't either.
        assert!(!r.termination_right_available);
    }

    #[test]
    fn citation_present_for_known_states() {
        let r = check(&base());
        assert!(r.statute.contains("§400"));

        let mut i = base();
        i.state = "TX".into();
        let r_tx = check(&i);
        assert!(r_tx.statute.contains("92.017"));
    }

    #[test]
    fn controlling_authority_prefers_federal_when_applicable() {
        let r = check(&base());
        assert!(r.controlling_authority.contains("3955"));
    }

    #[test]
    fn controlling_authority_falls_to_state_extension_for_spouse() {
        let mut i = base();
        i.tenant_role = TenantRole::Spouse;
        let r = check(&i);
        assert!(r.controlling_authority.contains("state extension"));
    }
}
