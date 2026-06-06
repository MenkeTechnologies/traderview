//! State crime-victim lease termination rights (broader than the
//! domestic-violence subset already covered by `dv_termination`).
//!
//! Goes beyond domestic violence to cover sexual assault, stalking,
//! human trafficking, elder/dependent-adult abuse, and other
//! enumerated violent crimes. Four states have specific statutes
//! that expand the lease-termination right beyond pure DV; many
//! states cover only DV (handled by `dv_termination`).
//!
//! Five regimes:
//!
//! `CaliforniaBroadestVictimCoverage`: CA only. Cal. Civ. Code
//! § 1946.7 (eff. 2011; expanded by AB 1493 in 2022). The
//! BROADEST scope: covers DV, sexual assault, stalking, human
//! trafficking, abuse of elder or dependent adult, and any crime
//! that has caused bodily injury or included force/threat. Tenant
//! must give 14-day notice to terminate, plus written attestation
//! and supporting document (court order, police report, or
//! qualified-third-party statement). Companion to Cal. Civ. Proc.
//! Code § 1161.3 prohibiting eviction based on victim status.
//!
//! `TexasThirtyDayNotice`: TX only. Tex. Prop. Code §§ 92.0161
//! and 92.1061. Covers family violence, sexual assault, child
//! sexual abuse, and stalking. Tenant must give **30-day written
//! notice** and remains liable for rent during the notice period.
//! Supporting document: protective order, qualified-third-party
//! verification, or police report.
//!
//! `WashingtonNinetyDayWindow`: WA only. RCW 59.18.575. Covers
//! DV, sexual assault, unlawful harassment, and stalking. Request
//! to terminate must be made within **90 days** of the qualifying
//! event. Tenant pays rent only for the month of termination, NOT
//! subsequent months. Tenant entitled to security deposit return
//! as if lease ran full term. Supporting document: protection
//! order under ch. 7.105 RCW or written qualified-third-party
//! report.
//!
//! `IllinoisSafeHomesActDualPath`: IL only. Safe Homes Act,
//! 765 ILCS 750 et seq. Dual-track path: for a credible imminent
//! threat of DV or sexual violence, written notice within 3 days
//! of vacating is required; for past sexual violence, written
//! notice within 60 days of the incident (or as soon as
//! practicable) plus at least ONE form of supporting evidence
//! (medical / court / police / victim-services statement).
//!
//! `NoStatewideBroadCrimeVictimTermination`: 46 other states +
//! DC. Either no statewide crime-victim termination right, or
//! the right covers DV ONLY (modeled separately in
//! `dv_termination` module). Tenant may face full early-
//! termination penalties for non-DV crime victim status.
//!
//! Sources:
//! [Cal. Civ. Code § 1946.7 — WomensLaw](https://www.womenslaw.org/laws/ca/statutes/19467-victims-domestic-violence-sexual-assault-stalking-human-trafficking-or-abuse),
//! [Cal. AB 1493 (2021-2022) — Legislative Information](https://leginfo.legislature.ca.gov/faces/billTextClient.xhtml?bill_id=202120220AB1493),
//! [Tex. Prop. Code § 92.0161 — WomensLaw](https://www.womenslaw.org/laws/tx/housing-laws/early-lease-termination-victims-sexual-assault-sexual-abuse-or-stalking),
//! [WA RCW 59.18.575 — FindLaw](https://codes.findlaw.com/wa/title-59-landlord-and-tenant/wa-rev-code-59-18-575/),
//! [IL Safe Homes Act 765 ILCS 750 — Klueverlaw Group](https://www.klueverlawgroup.com/the-safe-homes-act-what-tenants-and-landlords-in-illinois-need-to-know-about-this-lesserknown-law).

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CrimeVictimTerminationRegime {
    CaliforniaBroadestVictimCoverage,
    TexasThirtyDayNotice,
    WashingtonNinetyDayWindow,
    IllinoisSafeHomesActDualPath,
    NoStatewideBroadCrimeVictimTermination,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VictimCategory {
    DomesticViolence,
    SexualAssault,
    Stalking,
    HumanTrafficking,
    ElderOrDependentAdultAbuse,
    UnlawfulHarassment,
    CredibleImminentThreat,
    None,
}

#[derive(Debug, Clone)]
pub struct StateRule {
    pub regime: CrimeVictimTerminationRegime,
    /// Required tenant notice days (CA 14 / TX 30 / WA n/a — see
    /// timing field / IL 3 imminent + 60 past).
    pub required_notice_days: u32,
    /// Days within which the tenant must give notice / terminate
    /// after the qualifying event (WA 90).
    pub days_window_from_incident: u32,
    /// True if the tenant pays only rent for the termination
    /// month (no further obligation) (WA).
    pub rent_obligation_terminates_at_notice: bool,
    /// True if the tenant is required to provide supporting
    /// documentation (court order / police report / qualified
    /// third party / victim services statement).
    pub supporting_documentation_required: bool,
    pub citation: &'static str,
}

const fn rule(
    regime: CrimeVictimTerminationRegime,
    required_notice_days: u32,
    days_window_from_incident: u32,
    rent_obligation_terminates_at_notice: bool,
    supporting_documentation_required: bool,
    citation: &'static str,
) -> StateRule {
    StateRule {
        regime,
        required_notice_days,
        days_window_from_incident,
        rent_obligation_terminates_at_notice,
        supporting_documentation_required,
        citation,
    }
}

pub static RULES: Lazy<HashMap<&'static str, StateRule>> = Lazy::new(|| {
    use CrimeVictimTerminationRegime::*;
    let mut m: HashMap<&'static str, StateRule> = HashMap::new();

    m.insert(
        "CA",
        rule(
            CaliforniaBroadestVictimCoverage,
            14,
            0,
            false,
            true,
            "Cal. Civ. Code § 1946.7 (eff. 2011; expanded by AB 1493 2022) — broadest U.S. scope: DV + sexual assault + stalking + human trafficking + elder/dependent-adult abuse + any crime causing bodily injury OR force/threat; tenant must give 14-day written notice + attestation + supporting document (court order, police report, or qualified third party); companion Cal. Civ. Proc. Code § 1161.3 prohibits eviction based on victim status",
        ),
    );

    m.insert(
        "TX",
        rule(
            TexasThirtyDayNotice,
            30,
            0,
            false,
            true,
            "Tex. Prop. Code §§ 92.0161 and 92.1061 — family violence + sexual assault + child sexual abuse + stalking; tenant must give 30-day written notice + supporting document (protective order, qualified-third-party verification, or police report); tenant remains liable for rent during 30-day notice period",
        ),
    );

    m.insert(
        "WA",
        rule(
            WashingtonNinetyDayWindow,
            0,
            90,
            true,
            true,
            "RCW 59.18.575 — DV + sexual assault + unlawful harassment + stalking; request to terminate must be made within 90 days of qualifying event; tenant pays rent only for month of termination + entitled to security deposit return as if lease ran full term; supporting document: protection order under ch. 7.105 RCW or written qualified-third-party report",
        ),
    );

    m.insert(
        "IL",
        rule(
            IllinoisSafeHomesActDualPath,
            3,
            60,
            false,
            true,
            "IL Safe Homes Act, 765 ILCS 750 et seq. — dual-path: (1) credible imminent threat of DV or sexual violence → written notice within 3 days of vacating; (2) past sexual violence → written notice within 60 days of incident + at least ONE supporting evidence form (medical / court / police / victim-services statement)",
        ),
    );

    // NoStatewideBroadCrimeVictimTermination default — 46 other
    // states + DC.
    let default_states = [
        "AL", "AK", "AZ", "AR", "CO", "CT", "DC", "DE", "FL", "GA", "HI", "ID", "IN", "IA", "KS",
        "KY", "LA", "ME", "MD", "MA", "MI", "MN", "MS", "MO", "MT", "NE", "NV", "NH", "NJ", "NM",
        "NY", "NC", "ND", "OH", "OK", "OR", "PA", "RI", "SC", "SD", "TN", "UT", "VT", "VA", "WV",
        "WI", "WY",
    ];
    for code in default_states {
        m.insert(
            code,
            rule(
                NoStatewideBroadCrimeVictimTermination,
                0,
                0,
                false,
                false,
                "No statewide broad crime-victim termination right; many states cover DV ONLY (modeled separately in dv_termination module); tenant may face full early-termination penalties for non-DV crime victim status",
            ),
        );
    }

    m
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrimeVictimTerminationInput {
    pub state_code: String,
    pub victim_category: VictimCategory,
    /// Days elapsed since the qualifying incident occurred (for
    /// WA 90-day and IL 60-day windows).
    pub days_since_incident: u32,
    /// Days of written notice the tenant has provided to the
    /// landlord.
    pub days_of_written_notice_given: u32,
    /// True if the tenant has provided the statutorily required
    /// supporting documentation.
    pub supporting_documentation_provided: bool,
    /// True if the tenant has caused force / threat or bodily
    /// injury was inflicted by the perpetrator (CA broader trigger
    /// for crimes causing bodily injury).
    pub crime_caused_bodily_injury_or_force_threat: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrimeVictimTerminationResult {
    pub regime: CrimeVictimTerminationRegime,
    pub victim_category_covered_by_regime: bool,
    pub notice_period_satisfied: bool,
    pub incident_within_statutory_window: bool,
    pub supporting_documentation_satisfied: bool,
    pub tenant_may_terminate_without_penalty: bool,
    pub rent_obligation_after_termination: bool,
    pub citation: String,
    pub note: String,
}

pub fn check(input: &CrimeVictimTerminationInput) -> CrimeVictimTerminationResult {
    let code = input.state_code.trim().to_uppercase();
    let rule = RULES.get(code.as_str()).cloned().unwrap_or(StateRule {
        regime: CrimeVictimTerminationRegime::NoStatewideBroadCrimeVictimTermination,
        required_notice_days: 0,
        days_window_from_incident: 0,
        rent_obligation_terminates_at_notice: false,
        supporting_documentation_required: false,
        citation: "Unknown state code; no statewide broad crime-victim termination assumed",
    });

    // Determine if the regime covers the victim's category.
    let category_covered = match rule.regime {
        CrimeVictimTerminationRegime::CaliforniaBroadestVictimCoverage => {
            // CA covers all 6 categories + any crime causing bodily
            // injury / force-threat.
            !matches!(input.victim_category, VictimCategory::None)
                || input.crime_caused_bodily_injury_or_force_threat
        }
        CrimeVictimTerminationRegime::TexasThirtyDayNotice => {
            matches!(
                input.victim_category,
                VictimCategory::DomesticViolence
                    | VictimCategory::SexualAssault
                    | VictimCategory::Stalking
            )
        }
        CrimeVictimTerminationRegime::WashingtonNinetyDayWindow => {
            matches!(
                input.victim_category,
                VictimCategory::DomesticViolence
                    | VictimCategory::SexualAssault
                    | VictimCategory::Stalking
                    | VictimCategory::UnlawfulHarassment
            )
        }
        CrimeVictimTerminationRegime::IllinoisSafeHomesActDualPath => {
            matches!(
                input.victim_category,
                VictimCategory::DomesticViolence
                    | VictimCategory::SexualAssault
                    | VictimCategory::CredibleImminentThreat
            )
        }
        CrimeVictimTerminationRegime::NoStatewideBroadCrimeVictimTermination => false,
    };

    // Window compliance (WA 90 days, IL 60 days for past sexual
    // violence path).
    let within_window = rule.days_window_from_incident == 0
        || input.days_since_incident <= rule.days_window_from_incident;

    // Notice compliance.
    let notice_satisfied = rule.required_notice_days == 0
        || input.days_of_written_notice_given >= rule.required_notice_days;

    // Supporting documentation compliance.
    let documentation_satisfied =
        !rule.supporting_documentation_required || input.supporting_documentation_provided;

    let may_terminate =
        category_covered && within_window && notice_satisfied && documentation_satisfied;

    // Rent obligation: WA terminates rent at notice; CA / TX / IL
    // tenant remains liable through the notice period or as
    // statute specifies.
    let rent_obligation_after = may_terminate && !rule.rent_obligation_terminates_at_notice;

    let regime_label = match rule.regime {
        CrimeVictimTerminationRegime::CaliforniaBroadestVictimCoverage => {
            "California § 1946.7 broadest-coverage regime (14-day notice + supporting document)"
        }
        CrimeVictimTerminationRegime::TexasThirtyDayNotice => {
            "Texas § 92.0161 30-day notice + tenant liable for notice period"
        }
        CrimeVictimTerminationRegime::WashingtonNinetyDayWindow => {
            "Washington RCW 59.18.575 90-day window + rent terminates at notice"
        }
        CrimeVictimTerminationRegime::IllinoisSafeHomesActDualPath => {
            "Illinois Safe Homes Act dual-path (3-day imminent / 60-day past sexual violence)"
        }
        CrimeVictimTerminationRegime::NoStatewideBroadCrimeVictimTermination => {
            "no statewide broad crime-victim termination right"
        }
    };

    let note = if may_terminate {
        format!(
            "State applies {} regime; tenant MAY terminate without penalty on these facts.",
            regime_label,
        )
    } else {
        let mut reasons = vec![];
        if !category_covered {
            reasons.push("victim category not within regime coverage");
        }
        if !within_window {
            reasons.push("incident outside statutory window");
        }
        if !notice_satisfied {
            reasons.push("statutory notice days not met");
        }
        if !documentation_satisfied {
            reasons.push("supporting documentation not provided");
        }
        format!(
            "State applies {} regime; tenant MAY NOT terminate on these facts: {}.",
            regime_label,
            reasons.join("; "),
        )
    };

    CrimeVictimTerminationResult {
        regime: rule.regime,
        victim_category_covered_by_regime: category_covered,
        notice_period_satisfied: notice_satisfied,
        incident_within_statutory_window: within_window,
        supporting_documentation_satisfied: documentation_satisfied,
        tenant_may_terminate_without_penalty: may_terminate,
        rent_obligation_after_termination: rent_obligation_after,
        citation: rule.citation.to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline(state: &str) -> CrimeVictimTerminationInput {
        CrimeVictimTerminationInput {
            state_code: state.to_string(),
            victim_category: VictimCategory::DomesticViolence,
            days_since_incident: 10,
            days_of_written_notice_given: 30,
            supporting_documentation_provided: true,
            crime_caused_bodily_injury_or_force_threat: false,
        }
    }

    // ── Regime classification ───────────────────────────────────────

    #[test]
    fn ca_broadest_regime() {
        let r = check(&baseline("CA"));
        assert_eq!(
            r.regime,
            CrimeVictimTerminationRegime::CaliforniaBroadestVictimCoverage
        );
    }

    #[test]
    fn tx_thirty_day_regime() {
        let r = check(&baseline("TX"));
        assert_eq!(r.regime, CrimeVictimTerminationRegime::TexasThirtyDayNotice);
    }

    #[test]
    fn wa_ninety_day_regime() {
        let r = check(&baseline("WA"));
        assert_eq!(
            r.regime,
            CrimeVictimTerminationRegime::WashingtonNinetyDayWindow
        );
    }

    #[test]
    fn il_safe_homes_act_regime() {
        let r = check(&baseline("IL"));
        assert_eq!(
            r.regime,
            CrimeVictimTerminationRegime::IllinoisSafeHomesActDualPath
        );
    }

    #[test]
    fn default_state_no_broad_termination_regime() {
        for s in ["AL", "FL", "NY", "MA", "NJ", "DC", "WY"] {
            let r = check(&baseline(s));
            assert_eq!(
                r.regime,
                CrimeVictimTerminationRegime::NoStatewideBroadCrimeVictimTermination,
                "expected {s} no-broad regime"
            );
        }
    }

    // ── CA coverage breadth ────────────────────────────────────────

    #[test]
    fn ca_human_trafficking_covered() {
        let mut i = baseline("CA");
        i.victim_category = VictimCategory::HumanTrafficking;
        let r = check(&i);
        assert!(r.victim_category_covered_by_regime);
        assert!(r.tenant_may_terminate_without_penalty);
    }

    #[test]
    fn ca_elder_abuse_covered() {
        let mut i = baseline("CA");
        i.victim_category = VictimCategory::ElderOrDependentAdultAbuse;
        let r = check(&i);
        assert!(r.victim_category_covered_by_regime);
    }

    #[test]
    fn ca_force_threat_path_independent() {
        // CA covers crimes causing bodily injury / force-threat
        // even when specific category is None.
        let mut i = baseline("CA");
        i.victim_category = VictimCategory::None;
        i.crime_caused_bodily_injury_or_force_threat = true;
        let r = check(&i);
        assert!(r.victim_category_covered_by_regime);
    }

    #[test]
    fn ca_14_day_notice_satisfied() {
        let mut i = baseline("CA");
        i.days_of_written_notice_given = 14;
        let r = check(&i);
        assert!(r.notice_period_satisfied);
    }

    #[test]
    fn ca_13_day_notice_not_satisfied() {
        let mut i = baseline("CA");
        i.days_of_written_notice_given = 13;
        let r = check(&i);
        assert!(!r.notice_period_satisfied);
        assert!(!r.tenant_may_terminate_without_penalty);
    }

    // ── TX coverage ────────────────────────────────────────────────

    #[test]
    fn tx_30_day_notice_satisfied() {
        let r = check(&baseline("TX"));
        assert!(r.notice_period_satisfied);
        assert!(r.tenant_may_terminate_without_penalty);
    }

    #[test]
    fn tx_29_day_notice_not_satisfied() {
        let mut i = baseline("TX");
        i.days_of_written_notice_given = 29;
        let r = check(&i);
        assert!(!r.notice_period_satisfied);
    }

    #[test]
    fn tx_human_trafficking_not_covered() {
        // TX does NOT cover human trafficking — narrower than CA.
        let mut i = baseline("TX");
        i.victim_category = VictimCategory::HumanTrafficking;
        let r = check(&i);
        assert!(!r.victim_category_covered_by_regime);
    }

    #[test]
    fn tx_rent_obligation_continues_during_notice_period() {
        let r = check(&baseline("TX"));
        assert!(r.tenant_may_terminate_without_penalty);
        assert!(
            r.rent_obligation_after_termination,
            "TX tenant liable for rent during 30-day notice period"
        );
    }

    // ── WA 90-day window ───────────────────────────────────────────

    #[test]
    fn wa_within_90_day_window_compliant() {
        let mut i = baseline("WA");
        i.days_since_incident = 89;
        let r = check(&i);
        assert!(r.incident_within_statutory_window);
        assert!(r.tenant_may_terminate_without_penalty);
    }

    #[test]
    fn wa_exactly_90_day_boundary_satisfied() {
        let mut i = baseline("WA");
        i.days_since_incident = 90;
        let r = check(&i);
        assert!(r.incident_within_statutory_window);
    }

    #[test]
    fn wa_91_days_outside_window() {
        let mut i = baseline("WA");
        i.days_since_incident = 91;
        let r = check(&i);
        assert!(!r.incident_within_statutory_window);
        assert!(!r.tenant_may_terminate_without_penalty);
    }

    #[test]
    fn wa_unlawful_harassment_covered() {
        let mut i = baseline("WA");
        i.victim_category = VictimCategory::UnlawfulHarassment;
        let r = check(&i);
        assert!(r.victim_category_covered_by_regime);
    }

    #[test]
    fn wa_rent_obligation_terminates_at_notice() {
        let r = check(&baseline("WA"));
        assert!(r.tenant_may_terminate_without_penalty);
        assert!(
            !r.rent_obligation_after_termination,
            "WA tenant rent obligation terminates at notice"
        );
    }

    // ── IL Safe Homes Act dual-path ────────────────────────────────

    #[test]
    fn il_credible_imminent_threat_3_day_path() {
        let mut i = baseline("IL");
        i.victim_category = VictimCategory::CredibleImminentThreat;
        i.days_of_written_notice_given = 3;
        let r = check(&i);
        assert!(r.tenant_may_terminate_without_penalty);
    }

    #[test]
    fn il_past_sexual_violence_within_60_day_window() {
        let mut i = baseline("IL");
        i.victim_category = VictimCategory::SexualAssault;
        i.days_since_incident = 59;
        let r = check(&i);
        assert!(r.incident_within_statutory_window);
    }

    #[test]
    fn il_past_sexual_violence_61_days_outside_window() {
        let mut i = baseline("IL");
        i.victim_category = VictimCategory::SexualAssault;
        i.days_since_incident = 61;
        let r = check(&i);
        assert!(!r.incident_within_statutory_window);
    }

    #[test]
    fn il_stalking_not_covered() {
        // IL Safe Homes Act doesn't cover stalking — narrower than
        // CA / WA / TX.
        let mut i = baseline("IL");
        i.victim_category = VictimCategory::Stalking;
        let r = check(&i);
        assert!(!r.victim_category_covered_by_regime);
    }

    // ── Default state ──────────────────────────────────────────────

    #[test]
    fn default_state_no_broad_termination_right() {
        let r = check(&baseline("NY"));
        assert!(!r.tenant_may_terminate_without_penalty);
        assert!(r.note.contains("not within regime coverage"));
    }

    // ── Supporting documentation ───────────────────────────────────

    #[test]
    fn ca_no_documentation_fails() {
        let mut i = baseline("CA");
        i.supporting_documentation_provided = false;
        let r = check(&i);
        assert!(!r.supporting_documentation_satisfied);
        assert!(!r.tenant_may_terminate_without_penalty);
    }

    // ── Citation contents ──────────────────────────────────────────

    #[test]
    fn ca_citation_mentions_1946_7_and_ab_1493() {
        let r = check(&baseline("CA"));
        assert!(r.citation.contains("§ 1946.7"));
        assert!(r.citation.contains("AB 1493 2022"));
        assert!(r.citation.contains("broadest U.S. scope"));
    }

    #[test]
    fn tx_citation_mentions_92_0161_and_30_day() {
        let r = check(&baseline("TX"));
        assert!(r.citation.contains("§§ 92.0161"));
        assert!(r.citation.contains("30-day"));
    }

    #[test]
    fn wa_citation_mentions_59_18_575_and_90_days() {
        let r = check(&baseline("WA"));
        assert!(r.citation.contains("RCW 59.18.575"));
        assert!(r.citation.contains("90 days"));
    }

    #[test]
    fn il_citation_mentions_765_ilcs_750_dual_path() {
        let r = check(&baseline("IL"));
        assert!(r.citation.contains("765 ILCS 750"));
        assert!(r.citation.contains("dual-path"));
        assert!(r.citation.contains("3 days"));
        assert!(r.citation.contains("60 days"));
    }

    // ── Coverage / single-state-uniqueness ─────────────────────────

    #[test]
    fn coverage_is_all_50_states_plus_dc() {
        assert_eq!(RULES.len(), 51);
    }

    #[test]
    fn citations_never_empty() {
        for (code, rule) in RULES.iter() {
            assert!(!rule.citation.is_empty(), "{code} empty citation");
        }
    }

    #[test]
    fn ca_only_broadest_regime_state() {
        let count = RULES
            .iter()
            .filter(|(_, r)| {
                matches!(
                    r.regime,
                    CrimeVictimTerminationRegime::CaliforniaBroadestVictimCoverage
                )
            })
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn wa_only_rent_terminates_at_notice_state() {
        let count = RULES
            .iter()
            .filter(|(_, r)| r.rent_obligation_terminates_at_notice)
            .count();
        assert_eq!(count, 1, "only WA terminates rent obligation at notice");
    }

    // ── Notes ──────────────────────────────────────────────────────

    #[test]
    fn ca_compliant_note_describes_regime() {
        let r = check(&baseline("CA"));
        assert!(r.note.contains("California § 1946.7 broadest-coverage"));
    }

    #[test]
    fn tx_human_trafficking_note_says_not_covered() {
        let mut i = baseline("TX");
        i.victim_category = VictimCategory::HumanTrafficking;
        let r = check(&i);
        assert!(r
            .note
            .contains("victim category not within regime coverage"));
    }

    // ── Normalization ──────────────────────────────────────────────

    #[test]
    fn lowercase_state_code_normalizes() {
        let r = check(&baseline("ca"));
        assert_eq!(
            r.regime,
            CrimeVictimTerminationRegime::CaliforniaBroadestVictimCoverage
        );
    }
}
