//! State-by-state short-term rental (STR) regulation compliance.
//!
//! Directly affects trader-landlords using Airbnb, VRBO, Booking.com,
//! etc. Recent legislative wave (2018-2024) — state tax-and-register
//! regimes, primary residence requirements, and major-city outright
//! bans have reshaped the STR landscape.
//!
//! **Four state regimes:**
//!
//! 1. **State preemption** — FL § 509.032 preempts localities from
//!    prohibiting or regulating STRs; only pre-June 1, 2011 ordinances
//!    are grandfathered. Investor-friendly regime.
//!
//! 2. **State tax + registration** — MA M.G.L. c. 64G (5.7% state
//!    excise + up to 6% local + community impact fee for professional
//!    operators), VT 32 V.S.A. § 9301 (3% surcharge eff. Aug 2024 +
//!    270-day primary residence requirement).
//!
//! 3. **Local authority with major-city rules** — NY (NYC LL 18 / 2022
//!    enforced Sept 2023 host-present + 183-day residence rule + OSE
//!    registration; platforms blocked from processing unregistered);
//!    HI (Honolulu Bill 41 / 30-day minimum + $1k registration + $500
//!    annual). State leaves authority to localities; major cities
//!    have notable restrictions.
//!
//! 4. **Local authority, no state rule** — most states. Localities may
//!    or may not regulate independently.
//!
//! **NYC primary residence rule (183 days)** is load-bearing: hosts must
//! live in the property for at least 183 days per year. Vacation-home
//! STRs in NYC are effectively banned (unless rented for ≥ 30 days at
//! a time, which falls outside the STR definition).
//!
//! **VT 270-day primary residence rule** (Act 183 of 2024) is the
//! strictest in the country.

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum StrRegime {
    StatePreemption,
    StateTaxAndRegistration,
    LocalAuthorityWithMajorCityRules,
    LocalAuthorityNoStateRule,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateStrRule {
    pub state_code: &'static str,
    pub state_name: &'static str,
    pub regime: StrRegime,
    /// State-level STR excise rate in basis points (e.g., 570 = 5.7%).
    /// `None` if state does not impose its own excise.
    pub state_excise_rate_basis_points: Option<u32>,
    /// State-level minimum days of primary residence required per year
    /// for STR. `None` if no state requirement.
    pub state_primary_residence_days_required: Option<u32>,
    /// True if state explicitly preempts local STR regulation.
    pub preempts_local_regulation: bool,
    /// Major city with notable STR restrictions referenced in citation.
    pub notable_city_restriction: Option<&'static str>,
    pub citation: &'static str,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrComplianceInput {
    pub state_code: String,
    pub days_hosted_per_year: u32,
    pub host_primary_residence_days_per_year: u32,
    pub host_registered_with_locality: bool,
    pub excise_collected_basis_points_remitted: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrComplianceResult {
    pub regime: StrRegime,
    pub state_excise_required_basis_points: u32,
    pub primary_residence_requirement_met: Option<bool>,
    pub excise_remittance_sufficient: bool,
    pub registration_required: bool,
    pub violations: Vec<String>,
    pub complies: bool,
    pub note: String,
}

pub fn lookup(state_code: &str) -> Option<&'static StateStrRule> {
    let up = state_code.to_ascii_uppercase();
    TABLE.get(up.as_str()).copied()
}

pub fn all_states() -> Vec<&'static StateStrRule> {
    let mut v: Vec<&'static StateStrRule> = TABLE.values().copied().collect();
    v.sort_by_key(|r| r.state_code);
    v
}

pub fn check(input: &StrComplianceInput) -> StrComplianceResult {
    let rule = match lookup(&input.state_code) {
        Some(r) => r,
        None => {
            return StrComplianceResult {
                regime: StrRegime::LocalAuthorityNoStateRule,
                state_excise_required_basis_points: 0,
                primary_residence_requirement_met: None,
                excise_remittance_sufficient: false,
                registration_required: false,
                violations: vec!["unknown state code".to_string()],
                complies: false,
                note: format!("unknown state code `{}`", input.state_code),
            };
        }
    };

    let mut violations: Vec<String> = Vec::new();

    // Excise remittance check.
    let state_excise = rule.state_excise_rate_basis_points.unwrap_or(0);
    let excise_ok = input.excise_collected_basis_points_remitted >= state_excise;
    if !excise_ok {
        violations.push(format!(
            "{} requires {}.{}% state excise; only {}.{}% remitted",
            rule.state_name,
            state_excise / 100,
            state_excise % 100,
            input.excise_collected_basis_points_remitted / 100,
            input.excise_collected_basis_points_remitted % 100,
        ));
    }

    // Primary residence requirement check.
    let residency_met = rule
        .state_primary_residence_days_required
        .map(|min| input.host_primary_residence_days_per_year >= min);
    if let Some(false) = residency_met {
        violations.push(format!(
            "{} requires primary residence of {} days; only {} days reported",
            rule.state_name,
            rule.state_primary_residence_days_required.unwrap_or(0),
            input.host_primary_residence_days_per_year,
        ));
    }

    // Registration requirement: implied for tax-and-registration and
    // local-with-major-city regimes.
    let registration_required = matches!(
        rule.regime,
        StrRegime::StateTaxAndRegistration | StrRegime::LocalAuthorityWithMajorCityRules
    );
    if registration_required && !input.host_registered_with_locality {
        violations.push(format!(
            "{}: registration required (state or local) — not completed",
            rule.state_name
        ));
    }

    let complies = violations.is_empty();
    let note = if complies {
        format!(
            "{}: STR regime {} — compliance requirements satisfied",
            rule.state_name,
            match rule.regime {
                StrRegime::StatePreemption => "state preemption (locality cannot regulate)",
                StrRegime::StateTaxAndRegistration => "state tax + registration",
                StrRegime::LocalAuthorityWithMajorCityRules =>
                    "local authority with major-city restrictions",
                StrRegime::LocalAuthorityNoStateRule => "local authority (no state rule)",
            }
        )
    } else {
        format!(
            "{}: {} STR compliance violation(s)",
            rule.state_name,
            violations.len()
        )
    };

    StrComplianceResult {
        regime: rule.regime,
        state_excise_required_basis_points: state_excise,
        primary_residence_requirement_met: residency_met,
        excise_remittance_sufficient: excise_ok,
        registration_required,
        violations,
        complies,
        note,
    }
}

#[allow(clippy::too_many_arguments)]
const fn rule(
    state_code: &'static str,
    state_name: &'static str,
    regime: StrRegime,
    state_excise_rate_basis_points: Option<u32>,
    state_primary_residence_days_required: Option<u32>,
    preempts_local_regulation: bool,
    notable_city_restriction: Option<&'static str>,
    citation: &'static str,
) -> StateStrRule {
    StateStrRule {
        state_code,
        state_name,
        regime,
        state_excise_rate_basis_points,
        state_primary_residence_days_required,
        preempts_local_regulation,
        notable_city_restriction,
        citation,
    }
}

static TABLE: Lazy<HashMap<&'static str, &'static StateStrRule>> = Lazy::new(|| {
    use StrRegime::*;
    static RULES: &[StateStrRule] = &[
        rule(
            "AK",
            "Alaska",
            LocalAuthorityNoStateRule,
            None,
            None,
            false,
            None,
            "local authority",
        ),
        rule(
            "AL",
            "Alabama",
            LocalAuthorityNoStateRule,
            None,
            None,
            false,
            None,
            "local authority",
        ),
        rule(
            "AR",
            "Arkansas",
            LocalAuthorityNoStateRule,
            None,
            None,
            false,
            None,
            "local authority",
        ),
        rule(
            "AZ",
            "Arizona",
            LocalAuthorityNoStateRule,
            None,
            None,
            false,
            None,
            "A.R.S. § 9-500.39 (lodging tax)",
        ),
        rule(
            "CA",
            "California",
            LocalAuthorityWithMajorCityRules,
            None,
            None,
            false,
            Some("San Francisco (90-day unhosted cap) / Los Angeles HSO"),
            "Cal. SB 60 (2017) — local authority preserved",
        ),
        rule(
            "CO",
            "Colorado",
            LocalAuthorityNoStateRule,
            None,
            None,
            false,
            Some("Denver (primary residence + license)"),
            "C.R.S. § 39-26 (lodging tax) + HB 23-1213 property tax classification",
        ),
        rule(
            "CT",
            "Connecticut",
            LocalAuthorityNoStateRule,
            None,
            None,
            false,
            None,
            "local authority",
        ),
        rule(
            "DC",
            "District of Columbia",
            LocalAuthorityNoStateRule,
            None,
            Some(183),
            false,
            None,
            "D.C. Code § 47-2829 (primary residence + license)",
        ),
        rule(
            "DE",
            "Delaware",
            LocalAuthorityNoStateRule,
            None,
            None,
            false,
            None,
            "local authority",
        ),
        rule(
            "FL",
            "Florida",
            StatePreemption,
            None,
            None,
            true,
            None,
            "Fla. Stat. § 509.032 (state preempts local STR regulation)",
        ),
        rule(
            "GA",
            "Georgia",
            LocalAuthorityNoStateRule,
            None,
            None,
            false,
            None,
            "local authority",
        ),
        rule(
            "HI",
            "Hawaii",
            LocalAuthorityWithMajorCityRules,
            None,
            None,
            false,
            Some("Honolulu (30-day minimum residential / Bill 41 + $1k registration)"),
            "Honolulu Bill 41 / CO 22-7 (2022)",
        ),
        rule(
            "IA",
            "Iowa",
            LocalAuthorityNoStateRule,
            None,
            None,
            false,
            None,
            "local authority",
        ),
        rule(
            "ID",
            "Idaho",
            LocalAuthorityNoStateRule,
            None,
            None,
            false,
            None,
            "local authority",
        ),
        rule(
            "IL",
            "Illinois",
            LocalAuthorityWithMajorCityRules,
            None,
            None,
            false,
            Some("Chicago Shared Housing Ordinance + Cook County tax"),
            "Chicago Mun. Code § 4-13",
        ),
        rule(
            "IN",
            "Indiana",
            LocalAuthorityNoStateRule,
            None,
            None,
            false,
            None,
            "local authority",
        ),
        rule(
            "KS",
            "Kansas",
            LocalAuthorityNoStateRule,
            None,
            None,
            false,
            None,
            "local authority",
        ),
        rule(
            "KY",
            "Kentucky",
            LocalAuthorityNoStateRule,
            None,
            None,
            false,
            None,
            "local authority",
        ),
        rule(
            "LA",
            "Louisiana",
            LocalAuthorityWithMajorCityRules,
            None,
            None,
            false,
            Some("New Orleans (ban + permit residential 2023)"),
            "New Orleans CCNO Ord.",
        ),
        rule(
            "MA",
            "Massachusetts",
            StateTaxAndRegistration,
            Some(570), // 5.7%
            None,
            false,
            None,
            "M.G.L. c. 64G (5.7% state excise eff. 2019 + local up to 6%)",
        ),
        rule(
            "MD",
            "Maryland",
            LocalAuthorityNoStateRule,
            None,
            None,
            false,
            None,
            "local authority",
        ),
        rule(
            "ME",
            "Maine",
            LocalAuthorityNoStateRule,
            None,
            None,
            false,
            None,
            "local authority",
        ),
        rule(
            "MI",
            "Michigan",
            LocalAuthorityNoStateRule,
            None,
            None,
            false,
            None,
            "local authority",
        ),
        rule(
            "MN",
            "Minnesota",
            LocalAuthorityNoStateRule,
            None,
            None,
            false,
            None,
            "local authority",
        ),
        rule(
            "MO",
            "Missouri",
            LocalAuthorityNoStateRule,
            None,
            None,
            false,
            None,
            "local authority",
        ),
        rule(
            "MS",
            "Mississippi",
            LocalAuthorityNoStateRule,
            None,
            None,
            false,
            None,
            "local authority",
        ),
        rule(
            "MT",
            "Montana",
            LocalAuthorityNoStateRule,
            None,
            None,
            false,
            None,
            "local authority",
        ),
        rule(
            "NC",
            "North Carolina",
            LocalAuthorityNoStateRule,
            None,
            None,
            false,
            None,
            "local authority",
        ),
        rule(
            "ND",
            "North Dakota",
            LocalAuthorityNoStateRule,
            None,
            None,
            false,
            None,
            "local authority",
        ),
        rule(
            "NE",
            "Nebraska",
            LocalAuthorityNoStateRule,
            None,
            None,
            false,
            None,
            "local authority",
        ),
        rule(
            "NH",
            "New Hampshire",
            LocalAuthorityNoStateRule,
            None,
            None,
            false,
            None,
            "local authority",
        ),
        rule(
            "NJ",
            "New Jersey",
            LocalAuthorityWithMajorCityRules,
            None,
            None,
            false,
            Some("Jersey City + Hoboken (restrictive ordinances)"),
            "Local authority",
        ),
        rule(
            "NM",
            "New Mexico",
            LocalAuthorityNoStateRule,
            None,
            None,
            false,
            None,
            "local authority",
        ),
        rule(
            "NV",
            "Nevada",
            LocalAuthorityWithMajorCityRules,
            None,
            None,
            false,
            Some("Las Vegas + Clark County (registration + tax)"),
            "Clark County Ord.",
        ),
        rule(
            "NY",
            "New York",
            LocalAuthorityWithMajorCityRules,
            None,
            Some(183),
            false,
            Some("NYC LL 18 (host-present + 183-day primary residence + OSE registration)"),
            "NYC Admin Code §§ 26-3101 et seq (LL 18 / 2022 enforced 2023-09)",
        ),
        rule(
            "OH",
            "Ohio",
            LocalAuthorityNoStateRule,
            None,
            None,
            false,
            None,
            "local authority",
        ),
        rule(
            "OK",
            "Oklahoma",
            LocalAuthorityNoStateRule,
            None,
            None,
            false,
            None,
            "local authority",
        ),
        rule(
            "OR",
            "Oregon",
            LocalAuthorityWithMajorCityRules,
            None,
            None,
            false,
            Some("Portland (registration + transient lodging tax)"),
            "ORS § 320.305 + local",
        ),
        rule(
            "PA",
            "Pennsylvania",
            LocalAuthorityNoStateRule,
            None,
            None,
            false,
            None,
            "local authority",
        ),
        rule(
            "RI",
            "Rhode Island",
            LocalAuthorityNoStateRule,
            None,
            None,
            false,
            None,
            "local authority",
        ),
        rule(
            "SC",
            "South Carolina",
            LocalAuthorityNoStateRule,
            None,
            None,
            false,
            None,
            "local authority",
        ),
        rule(
            "SD",
            "South Dakota",
            LocalAuthorityNoStateRule,
            None,
            None,
            false,
            None,
            "local authority",
        ),
        rule(
            "TN",
            "Tennessee",
            LocalAuthorityNoStateRule,
            None,
            None,
            false,
            None,
            "local authority",
        ),
        rule(
            "TX",
            "Texas",
            LocalAuthorityNoStateRule,
            None,
            None,
            false,
            None,
            "local authority",
        ),
        rule(
            "UT",
            "Utah",
            LocalAuthorityNoStateRule,
            None,
            None,
            false,
            None,
            "local authority",
        ),
        rule(
            "VA",
            "Virginia",
            LocalAuthorityNoStateRule,
            None,
            None,
            false,
            None,
            "local authority",
        ),
        rule(
            "VT",
            "Vermont",
            StateTaxAndRegistration,
            Some(300), // 3% surcharge
            Some(270), // 270 days primary residence
            false,
            None,
            "32 V.S.A. § 9301 (Act 183 of 2024 — 3% surcharge + 270-day primary residence)",
        ),
        rule(
            "WA",
            "Washington",
            LocalAuthorityWithMajorCityRules,
            None,
            None,
            false,
            Some("Seattle (regulated)"),
            "Seattle Mun. Code",
        ),
        rule(
            "WI",
            "Wisconsin",
            LocalAuthorityNoStateRule,
            None,
            None,
            false,
            None,
            "local authority",
        ),
        rule(
            "WV",
            "West Virginia",
            LocalAuthorityNoStateRule,
            None,
            None,
            false,
            None,
            "local authority",
        ),
        rule(
            "WY",
            "Wyoming",
            LocalAuthorityNoStateRule,
            None,
            None,
            false,
            None,
            "local authority",
        ),
    ];
    RULES.iter().map(|r| (r.state_code, r)).collect()
});

#[cfg(test)]
mod tests {
    use super::*;

    fn fully_compliant(state: &str) -> StrComplianceInput {
        StrComplianceInput {
            state_code: state.to_string(),
            days_hosted_per_year: 100,
            host_primary_residence_days_per_year: 280,
            host_registered_with_locality: true,
            excise_collected_basis_points_remitted: 1_000,
        }
    }

    #[test]
    fn table_covers_all_50_states_plus_dc() {
        assert_eq!(TABLE.len(), 51);
    }

    #[test]
    fn fl_state_preemption_complies_without_registration() {
        // FL preempts locality. No registration required.
        let mut i = fully_compliant("FL");
        i.host_registered_with_locality = false;
        let r = check(&i);
        assert!(r.complies);
        assert!(matches!(r.regime, StrRegime::StatePreemption));
    }

    #[test]
    fn ma_state_excise_5_7_required() {
        // MA requires 5.7% state excise (570 bp). 1000 bp remitted = OK.
        let r = check(&fully_compliant("MA"));
        assert_eq!(r.state_excise_required_basis_points, 570);
        assert!(r.complies);
    }

    #[test]
    fn ma_under_remittance_violates() {
        let mut i = fully_compliant("MA");
        i.excise_collected_basis_points_remitted = 500;
        let r = check(&i);
        assert!(!r.complies);
        assert!(r.violations.iter().any(|v| v.contains("5.70%")));
    }

    #[test]
    fn ma_registration_required() {
        let mut i = fully_compliant("MA");
        i.host_registered_with_locality = false;
        let r = check(&i);
        assert!(!r.complies);
        assert!(r.violations.iter().any(|v| v.contains("registration")));
    }

    #[test]
    fn vt_270_day_residency_required() {
        let mut i = fully_compliant("VT");
        i.host_primary_residence_days_per_year = 269;
        let r = check(&i);
        assert!(!r.complies);
        assert_eq!(r.primary_residence_requirement_met, Some(false));
        assert!(r.violations.iter().any(|v| v.contains("270 days")));
    }

    #[test]
    fn vt_270_day_exact_boundary_satisfies() {
        let mut i = fully_compliant("VT");
        i.host_primary_residence_days_per_year = 270;
        let r = check(&i);
        assert_eq!(r.primary_residence_requirement_met, Some(true));
    }

    #[test]
    fn vt_3_percent_surcharge_required() {
        let r = check(&fully_compliant("VT"));
        assert_eq!(r.state_excise_required_basis_points, 300);
    }

    #[test]
    fn ny_183_day_primary_residence_required() {
        let mut i = fully_compliant("NY");
        i.host_primary_residence_days_per_year = 182;
        let r = check(&i);
        assert!(!r.complies);
        assert_eq!(r.primary_residence_requirement_met, Some(false));
    }

    #[test]
    fn ny_183_day_exact_satisfies() {
        let mut i = fully_compliant("NY");
        i.host_primary_residence_days_per_year = 183;
        let r = check(&i);
        assert_eq!(r.primary_residence_requirement_met, Some(true));
    }

    #[test]
    fn ny_ll18_registration_required() {
        let mut i = fully_compliant("NY");
        i.host_registered_with_locality = false;
        let r = check(&i);
        assert!(!r.complies);
        assert!(r.registration_required);
    }

    #[test]
    fn dc_183_day_residency() {
        let mut i = fully_compliant("DC");
        i.host_primary_residence_days_per_year = 182;
        let r = check(&i);
        assert_eq!(r.primary_residence_requirement_met, Some(false));
    }

    #[test]
    fn hi_honolulu_registration_required_for_major_city_regime() {
        let mut i = fully_compliant("HI");
        i.host_registered_with_locality = false;
        let r = check(&i);
        assert!(!r.complies);
        assert!(r.registration_required);
    }

    #[test]
    fn ca_local_authority_with_major_city_no_state_residency() {
        // CA SB 60 leaves authority to locality; no state-level residency.
        let mut i = fully_compliant("CA");
        i.host_primary_residence_days_per_year = 50;
        let r = check(&i);
        assert_eq!(r.primary_residence_requirement_met, None);
        // But registration is required under LocalAuthorityWithMajorCityRules.
        i.host_registered_with_locality = false;
        let r2 = check(&i);
        assert!(!r2.complies);
    }

    #[test]
    fn tx_local_authority_no_state_rule_lenient() {
        // TX has no state STR rule. Default-flag input complies.
        let mut i = fully_compliant("TX");
        i.host_registered_with_locality = false;
        let r = check(&i);
        assert!(r.complies);
    }

    #[test]
    fn unknown_state_handled() {
        let i = fully_compliant("ZZ");
        let r = check(&i);
        assert!(!r.complies);
        assert!(r.note.contains("unknown state code"));
    }

    #[test]
    fn lookup_case_insensitive() {
        assert!(lookup("NY").is_some());
        assert!(lookup("ny").is_some());
    }

    #[test]
    fn all_states_sorted_by_code() {
        let states = all_states();
        assert_eq!(states.len(), 51);
        assert_eq!(states.first().unwrap().state_code, "AK");
        assert_eq!(states.last().unwrap().state_code, "WY");
    }

    #[test]
    fn citation_present_for_every_row() {
        for r in TABLE.values() {
            assert!(!r.citation.is_empty(), "{} citation empty", r.state_code);
        }
    }

    #[test]
    fn fl_only_state_with_preemption_regime() {
        let fl = lookup("FL").unwrap();
        assert!(matches!(fl.regime, StrRegime::StatePreemption));
        for r in TABLE.values() {
            if r.state_code != "FL" {
                assert!(
                    !matches!(r.regime, StrRegime::StatePreemption),
                    "{} should not have StatePreemption",
                    r.state_code
                );
            }
        }
    }

    #[test]
    fn state_tax_and_registration_states_pinned() {
        // MA and VT have StateTaxAndRegistration regime.
        for code in ["MA", "VT"] {
            let r = lookup(code).unwrap();
            assert!(matches!(r.regime, StrRegime::StateTaxAndRegistration));
        }
    }

    #[test]
    fn vt_strictest_residency_requirement_270_days() {
        // VT requires 270 days primary residence — strictest in country.
        let vt = lookup("VT").unwrap();
        assert_eq!(vt.state_primary_residence_days_required, Some(270));
        // Other states have at most 183.
        for r in TABLE.values() {
            if let Some(days) = r.state_primary_residence_days_required {
                assert!(
                    days <= 270,
                    "{} primary residence {} > 270",
                    r.state_code,
                    days
                );
            }
        }
    }
}
