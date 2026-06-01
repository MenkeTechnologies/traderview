//! State bedbug extermination cost / treatment-responsibility rules.
//!
//! Distinct from `bedbug_disclosure` (which is about initial
//! disclosure of past infestation at lease signing). This module
//! captures the cost-allocation rule when an infestation is
//! discovered mid-tenancy — who pays for extermination, what the
//! tenant must do to cooperate, and whether retaliation protection
//! kicks in when the tenant reports the infestation.
//!
//! Three regimes:
//!
//! `CaliforniaAB551Comprehensive`: CA only. Assembly Bill 551
//! (2017) added Civ. Code §§ 1954.600-1954.605 to create the most
//! detailed bedbug regime in the U.S.:
//!   - § 1954.602: landlord SHALL NOT show, rent, or lease any
//!     vacant unit known to have a current bed bug infestation.
//!   - § 1954.604: landlord must schedule follow-up and maintenance
//!     treatments until bedbugs are eradicated.
//!   - § 1942.5: 180-day retaliation protection window when tenant
//!     reports bedbugs in good faith and is current on rent.
//!   - Tenant must grant access for treatment (cooperation duty).
//!   - Cost: landlord pays under implied warranty of habitability
//!     unless tenant introduction is shown.
//!
//! `MaineLandlordEradicationStatutory`: ME only. Me. Stat. tit. 14
//! § 6021-A (eff. 2010). Landlord must investigate within 5 days,
//! engage a licensed pest control agent, and pay for initial
//! eradication. Tenant must cooperate and may be liable for
//! re-treatment if they recklessly or intentionally introduce
//! bedbugs. The statute requires pest-control-license verification.
//!
//! `DefaultImpliedWarrantyOfHabitability`: 48 other states + DC.
//! No bedbug-specific cost-allocation statute; courts apply the
//! implied warranty of habitability (e.g., NY Real Property Law
//! § 235-b; uniform Restatement 2d Property § 5.5) — landlord
//! generally pays unless tenant introduction proven. Tenant has
//! a common-law duty to mitigate / cooperate with treatment.
//!
//! Sources:
//! [Cal. Civ. Code §§ 1954.600-1954.605 (AB 551) — CalBedBugExterminators](https://cabedbugexterminators.com/california-tenant-rights-who-pays-for-bed-bug-extermination/),
//! [Cal. AB 551 (2015-2016 session) — Legislative Information](https://leginfo.legislature.ca.gov/faces/billTextClient.xhtml?bill_id=201520160AB551),
//! [Cal. Civ. Code § 1942.5 retaliation protection — Friedman & Chapman](https://www.fc-lawfirm.com/bed-bugs-in-your-apartment-who-pays-for-the-extermination/),
//! [Cal. Civ. Code § 1954.602 vacancy showing prohibition — Tenants of California](https://www.tenantsofla.com/blog/2023/9/26/what-are-the-laws-surrounding-bed-bug-infestations).

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BedbugExterminationRegime {
    CaliforniaAB551Comprehensive,
    MaineLandlordEradicationStatutory,
    DefaultImpliedWarrantyOfHabitability,
}

#[derive(Debug, Clone)]
pub struct StateRule {
    pub regime: BedbugExterminationRegime,
    /// Days the landlord has to investigate after tenant report
    /// (ME = 5 days; others = no statutory window).
    pub investigation_window_days: u32,
    /// True if statute affirmatively requires landlord to engage
    /// a licensed pest-control agent.
    pub licensed_pest_control_required: bool,
    /// True if statute affirmatively requires landlord to schedule
    /// follow-up and maintenance treatments until eradicated (CA).
    pub follow_up_treatments_required: bool,
    /// Days of statutory retaliation protection from tenant's
    /// good-faith bedbug report (CA = 180 days).
    pub retaliation_protection_window_days: u32,
    /// True if statute prohibits landlord from showing / renting
    /// vacant unit with known current infestation (CA § 1954.602).
    pub vacant_unit_show_prohibition: bool,
    pub citation: &'static str,
}

const fn rule(
    regime: BedbugExterminationRegime,
    investigation_window_days: u32,
    licensed_pest_control_required: bool,
    follow_up_treatments_required: bool,
    retaliation_protection_window_days: u32,
    vacant_unit_show_prohibition: bool,
    citation: &'static str,
) -> StateRule {
    StateRule {
        regime,
        investigation_window_days,
        licensed_pest_control_required,
        follow_up_treatments_required,
        retaliation_protection_window_days,
        vacant_unit_show_prohibition,
        citation,
    }
}

pub static RULES: Lazy<HashMap<&'static str, StateRule>> = Lazy::new(|| {
    use BedbugExterminationRegime::*;
    let mut m: HashMap<&'static str, StateRule> = HashMap::new();

    m.insert(
        "CA",
        rule(
            CaliforniaAB551Comprehensive,
            0,
            false,
            true,
            180,
            true,
            "Cal. Civ. Code §§ 1954.600-1954.605 (AB 551, 2017) + § 1942.5 — § 1954.602 landlord shall NOT show / rent / lease vacant unit with known current bed bug infestation; § 1954.604 landlord must schedule follow-up and maintenance treatments until eradicated; § 1942.5 180-day retaliation protection from tenant's good-faith bedbug report; tenant must grant access for treatment; landlord pays under implied warranty of habitability unless tenant introduction is shown",
        ),
    );

    m.insert(
        "ME",
        rule(
            MaineLandlordEradicationStatutory,
            5,
            true,
            false,
            0,
            false,
            "Me. Stat. tit. 14 § 6021-A (eff. 2010) — landlord must investigate within 5 days of tenant report + engage licensed pest control agent + pay for initial eradication; tenant must cooperate with treatment and may be liable for re-treatment cost if recklessly or intentionally introduced bedbugs",
        ),
    );

    // DefaultImpliedWarrantyOfHabitability — 48 other states + DC.
    let default_states = [
        "AL", "AK", "AZ", "AR", "CO", "CT", "DC", "DE",
        "FL", "GA", "HI", "ID", "IL", "IN", "IA", "KS",
        "KY", "LA", "MD", "MA", "MI", "MN", "MS", "MO",
        "MT", "NE", "NV", "NH", "NJ", "NM", "NY", "NC",
        "ND", "OH", "OK", "OR", "PA", "RI", "SC", "SD",
        "TN", "TX", "UT", "VT", "VA", "WA", "WV", "WI",
        "WY",
    ];
    for code in default_states {
        m.insert(
            code,
            rule(
                DefaultImpliedWarrantyOfHabitability,
                0,
                false,
                false,
                0,
                false,
                "No bedbug-specific cost-allocation statute; common-law implied warranty of habitability (per Restatement 2d Property § 5.5) — landlord pays unless tenant introduction proven; tenant has common-law duty to cooperate with treatment",
            ),
        );
    }

    m
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BedbugExterminationInput {
    pub state_code: String,
    /// Days elapsed since the tenant reported the infestation to
    /// the landlord (ME 5-day window analysis).
    pub days_since_tenant_report: u32,
    /// True if the landlord has engaged a pest control service.
    pub landlord_engaged_pest_control: bool,
    /// True if the engaged pest control agent is licensed
    /// (ME requirement).
    pub pest_control_agent_is_licensed: bool,
    /// True if the landlord has scheduled follow-up / maintenance
    /// treatments (CA § 1954.604 requirement).
    pub follow_up_treatments_scheduled: bool,
    /// True if the tenant is granting access for treatment.
    pub tenant_granting_access_for_treatment: bool,
    /// True if there is evidence the tenant recklessly or
    /// intentionally introduced bedbugs (ME re-treatment liability
    /// gate).
    pub tenant_recklessly_introduced_bedbugs: bool,
    /// True if the landlord is currently showing or renting a
    /// vacant unit with a known active bedbug infestation (CA
    /// § 1954.602 prohibition).
    pub landlord_showing_vacant_unit_with_known_infestation: bool,
    /// True if the tenant has reported the infestation in good
    /// faith (CA § 1942.5 retaliation gate).
    pub tenant_report_in_good_faith: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BedbugExterminationResult {
    pub regime: BedbugExterminationRegime,
    pub landlord_compliant: bool,
    pub investigation_window_met: bool,
    pub licensed_pest_control_satisfied: bool,
    pub follow_up_treatments_satisfied: bool,
    pub vacant_unit_prohibition_violated: bool,
    pub retaliation_protection_window_active: bool,
    pub tenant_liable_for_retreatment: bool,
    pub citation: String,
    pub note: String,
}

pub fn check(input: &BedbugExterminationInput) -> BedbugExterminationResult {
    let code = input.state_code.trim().to_uppercase();
    let rule = RULES.get(code.as_str()).cloned().unwrap_or(StateRule {
        regime: BedbugExterminationRegime::DefaultImpliedWarrantyOfHabitability,
        investigation_window_days: 0,
        licensed_pest_control_required: false,
        follow_up_treatments_required: false,
        retaliation_protection_window_days: 0,
        vacant_unit_show_prohibition: false,
        citation: "Unknown state code; common-law implied warranty of habitability assumed",
    });

    // Investigation window (ME 5-day).
    let investigation_window_met = rule.investigation_window_days == 0
        || input.days_since_tenant_report <= rule.investigation_window_days
        || input.landlord_engaged_pest_control;

    // Licensed pest control (ME).
    let licensed_pest_control_satisfied = !rule.licensed_pest_control_required
        || !input.landlord_engaged_pest_control
        || input.pest_control_agent_is_licensed;

    // Follow-up treatments (CA § 1954.604).
    let follow_up_treatments_satisfied =
        !rule.follow_up_treatments_required || input.follow_up_treatments_scheduled;

    // CA § 1954.602 vacant-unit prohibition.
    let vacant_unit_prohibition_violated = rule.vacant_unit_show_prohibition
        && input.landlord_showing_vacant_unit_with_known_infestation;

    // CA § 1942.5 retaliation window (active when tenant reported
    // in good faith AND within statutory days).
    let retaliation_protection_window_active = rule.retaliation_protection_window_days > 0
        && input.tenant_report_in_good_faith
        && input.days_since_tenant_report <= rule.retaliation_protection_window_days;

    // ME tenant re-treatment liability (when tenant recklessly /
    // intentionally introduced bedbugs).
    let tenant_liable_for_retreatment = matches!(
        rule.regime,
        BedbugExterminationRegime::MaineLandlordEradicationStatutory
    ) && input.tenant_recklessly_introduced_bedbugs;

    let landlord_compliant = investigation_window_met
        && licensed_pest_control_satisfied
        && follow_up_treatments_satisfied
        && !vacant_unit_prohibition_violated;

    let regime_label = match rule.regime {
        BedbugExterminationRegime::CaliforniaAB551Comprehensive => {
            "California AB 551 comprehensive bedbug regime"
        }
        BedbugExterminationRegime::MaineLandlordEradicationStatutory => {
            "Maine § 6021-A landlord eradication statutory regime"
        }
        BedbugExterminationRegime::DefaultImpliedWarrantyOfHabitability => {
            "default implied warranty of habitability"
        }
    };

    let note = if landlord_compliant {
        format!(
            "State applies {} regime; landlord compliant on these facts.",
            regime_label,
        )
    } else {
        let mut reasons = vec![];
        if !investigation_window_met {
            reasons.push(format!(
                "investigation window of {} days NOT met ({} days elapsed)",
                rule.investigation_window_days, input.days_since_tenant_report
            ));
        }
        if !licensed_pest_control_satisfied {
            reasons.push("engaged pest control agent is NOT licensed".to_string());
        }
        if !follow_up_treatments_satisfied {
            reasons.push("follow-up / maintenance treatments NOT scheduled".to_string());
        }
        if vacant_unit_prohibition_violated {
            reasons.push("§ 1954.602 prohibition violated — showing vacant unit with known infestation".to_string());
        }
        format!(
            "State applies {} regime; landlord NON-COMPLIANT: {}.",
            regime_label,
            reasons.join("; "),
        )
    };

    BedbugExterminationResult {
        regime: rule.regime,
        landlord_compliant,
        investigation_window_met,
        licensed_pest_control_satisfied,
        follow_up_treatments_satisfied,
        vacant_unit_prohibition_violated,
        retaliation_protection_window_active,
        tenant_liable_for_retreatment,
        citation: rule.citation.to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline(state: &str) -> BedbugExterminationInput {
        BedbugExterminationInput {
            state_code: state.to_string(),
            days_since_tenant_report: 1,
            landlord_engaged_pest_control: true,
            pest_control_agent_is_licensed: true,
            follow_up_treatments_scheduled: true,
            tenant_granting_access_for_treatment: true,
            tenant_recklessly_introduced_bedbugs: false,
            landlord_showing_vacant_unit_with_known_infestation: false,
            tenant_report_in_good_faith: true,
        }
    }

    // ── Regime classification ───────────────────────────────────────

    #[test]
    fn ca_ab551_regime() {
        let r = check(&baseline("CA"));
        assert_eq!(
            r.regime,
            BedbugExterminationRegime::CaliforniaAB551Comprehensive
        );
    }

    #[test]
    fn me_landlord_eradication_regime() {
        let r = check(&baseline("ME"));
        assert_eq!(
            r.regime,
            BedbugExterminationRegime::MaineLandlordEradicationStatutory
        );
    }

    #[test]
    fn default_state_implied_warranty_regime() {
        for s in ["AL", "NY", "TX", "FL", "MA", "WA", "DC", "WY"] {
            let r = check(&baseline(s));
            assert_eq!(
                r.regime,
                BedbugExterminationRegime::DefaultImpliedWarrantyOfHabitability,
                "expected {s} default regime"
            );
        }
    }

    // ── CA compliance ──────────────────────────────────────────────

    #[test]
    fn ca_full_compliance_baseline() {
        let r = check(&baseline("CA"));
        assert!(r.landlord_compliant);
    }

    #[test]
    fn ca_no_follow_up_treatments_non_compliant() {
        let mut i = baseline("CA");
        i.follow_up_treatments_scheduled = false;
        let r = check(&i);
        assert!(!r.landlord_compliant);
        assert!(!r.follow_up_treatments_satisfied);
    }

    #[test]
    fn ca_vacant_unit_prohibition_violation() {
        let mut i = baseline("CA");
        i.landlord_showing_vacant_unit_with_known_infestation = true;
        let r = check(&i);
        assert!(r.vacant_unit_prohibition_violated);
        assert!(!r.landlord_compliant);
    }

    #[test]
    fn ca_180_day_retaliation_window_active() {
        let mut i = baseline("CA");
        i.days_since_tenant_report = 100;
        let r = check(&i);
        assert!(r.retaliation_protection_window_active);
    }

    #[test]
    fn ca_181_days_retaliation_window_expired() {
        let mut i = baseline("CA");
        i.days_since_tenant_report = 181;
        let r = check(&i);
        assert!(!r.retaliation_protection_window_active);
    }

    #[test]
    fn ca_retaliation_window_requires_good_faith_report() {
        let mut i = baseline("CA");
        i.days_since_tenant_report = 30;
        i.tenant_report_in_good_faith = false;
        let r = check(&i);
        assert!(!r.retaliation_protection_window_active);
    }

    // ── ME compliance ──────────────────────────────────────────────

    #[test]
    fn me_full_compliance_baseline() {
        let r = check(&baseline("ME"));
        assert!(r.landlord_compliant);
    }

    #[test]
    fn me_5_day_window_satisfied_when_engaged_within_window() {
        let mut i = baseline("ME");
        i.days_since_tenant_report = 5;
        let r = check(&i);
        assert!(r.investigation_window_met);
        assert!(r.landlord_compliant);
    }

    #[test]
    fn me_6_day_no_engagement_window_not_met() {
        let mut i = baseline("ME");
        i.days_since_tenant_report = 6;
        i.landlord_engaged_pest_control = false;
        let r = check(&i);
        assert!(!r.investigation_window_met);
        assert!(!r.landlord_compliant);
    }

    #[test]
    fn me_unlicensed_pest_control_non_compliant() {
        let mut i = baseline("ME");
        i.pest_control_agent_is_licensed = false;
        let r = check(&i);
        assert!(!r.licensed_pest_control_satisfied);
        assert!(!r.landlord_compliant);
    }

    #[test]
    fn me_tenant_reckless_introduction_re_treatment_liability() {
        let mut i = baseline("ME");
        i.tenant_recklessly_introduced_bedbugs = true;
        let r = check(&i);
        assert!(r.tenant_liable_for_retreatment);
    }

    #[test]
    fn me_no_follow_up_treatments_still_compliant() {
        // ME doesn't require follow-up treatments — only CA does.
        let mut i = baseline("ME");
        i.follow_up_treatments_scheduled = false;
        let r = check(&i);
        assert!(r.landlord_compliant);
    }

    // ── Default state ──────────────────────────────────────────────

    #[test]
    fn default_state_no_statutory_windows() {
        let r = check(&baseline("NY"));
        assert!(r.landlord_compliant);
        assert!(!r.retaliation_protection_window_active);
        assert!(!r.tenant_liable_for_retreatment);
    }

    #[test]
    fn default_state_vacant_unit_prohibition_not_statutory() {
        // CA-specific prohibition doesn't apply outside CA.
        let mut i = baseline("TX");
        i.landlord_showing_vacant_unit_with_known_infestation = true;
        let r = check(&i);
        assert!(!r.vacant_unit_prohibition_violated);
        assert!(r.landlord_compliant);
    }

    // ── Citation contents ──────────────────────────────────────────

    #[test]
    fn ca_citation_mentions_ab_551_and_180_days() {
        let r = check(&baseline("CA"));
        assert!(r.citation.contains("AB 551, 2017"));
        assert!(r.citation.contains("§ 1942.5"));
        assert!(r.citation.contains("§ 1954.602"));
        assert!(r.citation.contains("180-day"));
    }

    #[test]
    fn me_citation_mentions_6021_a_and_5_days() {
        let r = check(&baseline("ME"));
        assert!(r.citation.contains("§ 6021-A"));
        assert!(r.citation.contains("5 days"));
        assert!(r.citation.contains("licensed pest control"));
    }

    #[test]
    fn default_citation_mentions_restatement_2d() {
        let r = check(&baseline("NY"));
        assert!(r.citation.contains("Restatement 2d Property § 5.5"));
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
    fn ca_only_ab551_state() {
        let count = RULES
            .iter()
            .filter(|(_, r)| {
                matches!(
                    r.regime,
                    BedbugExterminationRegime::CaliforniaAB551Comprehensive
                )
            })
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn me_only_eradication_statutory_state() {
        let count = RULES
            .iter()
            .filter(|(_, r)| {
                matches!(
                    r.regime,
                    BedbugExterminationRegime::MaineLandlordEradicationStatutory
                )
            })
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn ca_only_vacant_unit_prohibition_state() {
        let count = RULES.iter().filter(|(_, r)| r.vacant_unit_show_prohibition).count();
        assert_eq!(count, 1);
    }

    #[test]
    fn ca_only_retaliation_window_state() {
        let count = RULES
            .iter()
            .filter(|(_, r)| r.retaliation_protection_window_days > 0)
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn me_only_licensed_pest_control_required_state() {
        let count = RULES.iter().filter(|(_, r)| r.licensed_pest_control_required).count();
        assert_eq!(count, 1);
    }

    // ── Notes ──────────────────────────────────────────────────────

    #[test]
    fn ca_non_compliant_note_mentions_1954_602() {
        let mut i = baseline("CA");
        i.landlord_showing_vacant_unit_with_known_infestation = true;
        let r = check(&i);
        assert!(r.note.contains("§ 1954.602"));
    }

    #[test]
    fn me_non_compliant_note_mentions_investigation_window() {
        let mut i = baseline("ME");
        i.days_since_tenant_report = 10;
        i.landlord_engaged_pest_control = false;
        let r = check(&i);
        assert!(r.note.contains("investigation window of 5 days NOT met"));
    }

    // ── Normalization ──────────────────────────────────────────────

    #[test]
    fn lowercase_state_code_normalizes() {
        let r = check(&baseline("ca"));
        assert_eq!(
            r.regime,
            BedbugExterminationRegime::CaliforniaAB551Comprehensive
        );
    }
}
