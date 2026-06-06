//! State + municipal senior + disabled tenant protection compliance.
//!
//! Specialized protection regimes for at-risk tenant populations.
//! Distinct from general just-cause eviction (which applies to all
//! tenants in covered states) and federal FHA (which prohibits
//! disability discrimination but doesn't provide age-based or
//! income-based protections).
//!
//! Four regimes:
//!
//! 1. **NJ Senior Citizens & Disabled Protected Tenancy Act**
//!    (N.J.S.A. § 2A:18-61.22 et seq, 1982) — strongest in country.
//!    Eligible tenants get up to **40 years** of protection from
//!    eviction due to condominium / cooperative conversion. Hudson
//!    County tenants get PERMANENT protection (not 40-year). Five
//!    eligibility requirements:
//!      - Age 62+ OR disabled
//!      - Income ≤ $50,000 OR 3× county per capita (whichever greater)
//!      - Building has 5+ rental units
//!      - 1+ year residency OR lease term > 1 year
//!      - Tenant resides as principal residence
//!
//! 2. **NYC SCRIE / DRIE municipal rent-increase exemption**
//!    (NYC Admin Code § 26-509(b)(2) / § 26-405(m)(1)).
//!    Eligible tenants get future rent increases locked / refunded to
//!    landlord as property tax abatement. Five requirements:
//!      - SCRIE: age 62+; DRIE: disabled (any age)
//!      - Combined household income ≤ $50,000
//!      - Spend > 1/3 of household income on rent
//!      - Live in rent-regulated unit
//!      - Apply through NYC Dept. of Finance Rent Freeze Program
//!
//! 3. **State-level rent control / just-cause includes senior carve-out**
//!    (DC § 42-3505.01(c)(1) limited; CA AB 1482 covers seniors but
//!    no carve-out).
//!
//! 4. **No specific senior/disabled statute** — most states. Federal
//!    FHA still prohibits disability discrimination, but no age-based
//!    or income-based housing protection.

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SeniorProtectionRegime {
    /// NJ-model statewide condo-conversion protection with up to 40-year
    /// or permanent (Hudson County) protected tenancy.
    StatewideConversionProtection,
    /// NYC-model municipal rent-increase exemption.
    MunicipalRentIncreaseExemption,
    /// General just-cause covers seniors but no carve-out.
    JustCauseCoversNoCarveOut,
    /// No specific senior/disabled statute; only federal FHA applies.
    NoSpecificStatute,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSeniorRule {
    pub state_code: &'static str,
    pub state_name: &'static str,
    pub regime: SeniorProtectionRegime,
    /// Minimum age for senior-based protection. Disabled tenants
    /// typically qualify at any age.
    pub minimum_age: Option<u32>,
    pub disability_qualifies: bool,
    /// Maximum income threshold (dollars). Some statutes use a fixed
    /// figure (NJ $50,000), others scale.
    pub income_threshold_dollars: Option<i64>,
    /// Maximum protection duration in years. `Some(40)` for NJ 40-year
    /// statutory protected tenancy. `None` if no fixed maximum (Hudson
    /// County permanent / NYC SCRIE permanent).
    pub max_protection_years: Option<u32>,
    /// Minimum building unit count for protection to apply.
    pub minimum_building_units: Option<u32>,
    /// Minimum tenancy length required.
    pub minimum_tenancy_years: Option<u32>,
    pub citation: &'static str,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeniorDisabledCheckInput {
    pub state_code: String,
    pub tenant_age: u32,
    pub tenant_disabled: bool,
    pub tenant_annual_income_dollars: i64,
    pub years_in_building: u32,
    pub building_unit_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeniorDisabledCheckResult {
    /// True if the state has a specific senior/disabled statute that
    /// could apply.
    pub statute_available: bool,
    /// True if tenant meets all eligibility requirements for the
    /// statute's protection.
    pub eligible_for_protection: bool,
    pub age_qualifies: bool,
    pub income_qualifies: Option<bool>,
    pub building_size_qualifies: Option<bool>,
    pub tenancy_length_qualifies: Option<bool>,
    pub maximum_protection_years: Option<u32>,
    pub reasons_not_eligible: Vec<String>,
    pub citation: &'static str,
    pub note: String,
}

pub fn lookup(state_code: &str) -> Option<&'static StateSeniorRule> {
    let up = state_code.to_ascii_uppercase();
    TABLE.get(up.as_str()).copied()
}

pub fn all_states() -> Vec<&'static StateSeniorRule> {
    let mut v: Vec<&'static StateSeniorRule> = TABLE.values().copied().collect();
    v.sort_by_key(|r| r.state_code);
    v
}

pub fn check(input: &SeniorDisabledCheckInput) -> SeniorDisabledCheckResult {
    let rule = match lookup(&input.state_code) {
        Some(r) => r,
        None => {
            return SeniorDisabledCheckResult {
                statute_available: false,
                eligible_for_protection: false,
                age_qualifies: false,
                income_qualifies: None,
                building_size_qualifies: None,
                tenancy_length_qualifies: None,
                maximum_protection_years: None,
                reasons_not_eligible: vec!["unknown state code".to_string()],
                citation: "n/a",
                note: format!("unknown state code `{}`", input.state_code),
            };
        }
    };

    let statute_available = !matches!(rule.regime, SeniorProtectionRegime::NoSpecificStatute);

    if !statute_available {
        return SeniorDisabledCheckResult {
            statute_available: false,
            eligible_for_protection: false,
            age_qualifies: false,
            income_qualifies: None,
            building_size_qualifies: None,
            tenancy_length_qualifies: None,
            maximum_protection_years: None,
            reasons_not_eligible: vec!["no specific senior/disabled statute".to_string()],
            citation: rule.citation,
            note: format!(
                "{}: no specific senior/disabled statute — federal FHA disability protection still applies",
                rule.state_name
            ),
        };
    }

    let mut reasons: Vec<String> = Vec::new();

    // Age check (senior). Disabled tenants typically qualify at any age.
    let age_ok = match (rule.minimum_age, input.tenant_disabled) {
        (None, _) => true,
        (Some(_), true) if rule.disability_qualifies => true,
        (Some(min_age), false) => input.tenant_age >= min_age,
        (Some(_), true) => true,
    };
    if !age_ok {
        reasons.push(format!(
            "tenant age {} below minimum {} (and not disabled)",
            input.tenant_age,
            rule.minimum_age.unwrap_or(0)
        ));
    }

    // Income check.
    let income_ok = rule
        .income_threshold_dollars
        .map(|threshold| input.tenant_annual_income_dollars <= threshold);
    if let Some(false) = income_ok {
        reasons.push(format!(
            "tenant income ${} exceeds threshold ${}",
            input.tenant_annual_income_dollars,
            rule.income_threshold_dollars.unwrap_or(0)
        ));
    }

    // Building size check.
    let building_ok = rule
        .minimum_building_units
        .map(|min| input.building_unit_count >= min);
    if let Some(false) = building_ok {
        reasons.push(format!(
            "building has {} units, below minimum {}",
            input.building_unit_count,
            rule.minimum_building_units.unwrap_or(0)
        ));
    }

    // Tenancy length check.
    let tenancy_ok = rule
        .minimum_tenancy_years
        .map(|min| input.years_in_building >= min);
    if let Some(false) = tenancy_ok {
        reasons.push(format!(
            "tenant in building {} years, below minimum {}",
            input.years_in_building,
            rule.minimum_tenancy_years.unwrap_or(0)
        ));
    }

    let eligible = reasons.is_empty();

    let note = if eligible {
        format!(
            "{}: tenant ELIGIBLE for {} ({} max protection)",
            rule.state_name,
            match rule.regime {
                SeniorProtectionRegime::StatewideConversionProtection =>
                    "statewide condo-conversion protected tenancy",
                SeniorProtectionRegime::MunicipalRentIncreaseExemption =>
                    "municipal rent-increase exemption",
                SeniorProtectionRegime::JustCauseCoversNoCarveOut =>
                    "general just-cause protection",
                SeniorProtectionRegime::NoSpecificStatute => "no statute",
            },
            rule.max_protection_years
                .map(|y| format!("{}-year", y))
                .unwrap_or_else(|| "permanent".into())
        )
    } else {
        format!(
            "{}: tenant NOT eligible — {} requirement(s) unmet",
            rule.state_name,
            reasons.len()
        )
    };

    SeniorDisabledCheckResult {
        statute_available: true,
        eligible_for_protection: eligible,
        age_qualifies: age_ok,
        income_qualifies: income_ok,
        building_size_qualifies: building_ok,
        tenancy_length_qualifies: tenancy_ok,
        maximum_protection_years: rule.max_protection_years,
        reasons_not_eligible: reasons,
        citation: rule.citation,
        note,
    }
}

#[allow(clippy::too_many_arguments)]
const fn rule(
    state_code: &'static str,
    state_name: &'static str,
    regime: SeniorProtectionRegime,
    minimum_age: Option<u32>,
    disability_qualifies: bool,
    income_threshold_dollars: Option<i64>,
    max_protection_years: Option<u32>,
    minimum_building_units: Option<u32>,
    minimum_tenancy_years: Option<u32>,
    citation: &'static str,
) -> StateSeniorRule {
    StateSeniorRule {
        state_code,
        state_name,
        regime,
        minimum_age,
        disability_qualifies,
        income_threshold_dollars,
        max_protection_years,
        minimum_building_units,
        minimum_tenancy_years,
        citation,
    }
}

static TABLE: Lazy<HashMap<&'static str, &'static StateSeniorRule>> = Lazy::new(|| {
    use SeniorProtectionRegime::*;
    static RULES: &[StateSeniorRule] = &[
        rule(
            "AK",
            "Alaska",
            NoSpecificStatute,
            None,
            false,
            None,
            None,
            None,
            None,
            "federal FHA only",
        ),
        rule(
            "AL",
            "Alabama",
            NoSpecificStatute,
            None,
            false,
            None,
            None,
            None,
            None,
            "federal FHA only",
        ),
        rule(
            "AR",
            "Arkansas",
            NoSpecificStatute,
            None,
            false,
            None,
            None,
            None,
            None,
            "federal FHA only",
        ),
        rule(
            "AZ",
            "Arizona",
            NoSpecificStatute,
            None,
            false,
            None,
            None,
            None,
            None,
            "federal FHA only",
        ),
        rule(
            "CA",
            "California",
            JustCauseCoversNoCarveOut,
            None,
            true,
            None,
            None,
            None,
            None,
            "Cal. Civ. Code § 1946.2 (AB 1482) — covers all tenants, no senior carve-out",
        ),
        rule(
            "CO",
            "Colorado",
            NoSpecificStatute,
            None,
            false,
            None,
            None,
            None,
            None,
            "federal FHA only",
        ),
        rule(
            "CT",
            "Connecticut",
            JustCauseCoversNoCarveOut,
            Some(62),
            true,
            None,
            None,
            None,
            None,
            "Conn. Gen. Stat. § 47a-23c — protection for tenants 62+ + disabled",
        ),
        rule(
            "DC",
            "District of Columbia",
            JustCauseCoversNoCarveOut,
            Some(62),
            true,
            None,
            None,
            None,
            None,
            "D.C. Code § 42-3505.01(c)(1) — elderly tenant rights",
        ),
        rule(
            "DE",
            "Delaware",
            NoSpecificStatute,
            None,
            false,
            None,
            None,
            None,
            None,
            "federal FHA only",
        ),
        rule(
            "FL",
            "Florida",
            NoSpecificStatute,
            None,
            false,
            None,
            None,
            None,
            None,
            "federal FHA only",
        ),
        rule(
            "GA",
            "Georgia",
            NoSpecificStatute,
            None,
            false,
            None,
            None,
            None,
            None,
            "federal FHA only",
        ),
        rule(
            "HI",
            "Hawaii",
            NoSpecificStatute,
            None,
            false,
            None,
            None,
            None,
            None,
            "federal FHA only",
        ),
        rule(
            "IA",
            "Iowa",
            NoSpecificStatute,
            None,
            false,
            None,
            None,
            None,
            None,
            "federal FHA only",
        ),
        rule(
            "ID",
            "Idaho",
            NoSpecificStatute,
            None,
            false,
            None,
            None,
            None,
            None,
            "federal FHA only",
        ),
        rule(
            "IL",
            "Illinois",
            JustCauseCoversNoCarveOut,
            Some(62),
            true,
            None,
            None,
            None,
            None,
            "765 ILCS 745 (Protected Tenant Status)",
        ),
        rule(
            "IN",
            "Indiana",
            NoSpecificStatute,
            None,
            false,
            None,
            None,
            None,
            None,
            "federal FHA only",
        ),
        rule(
            "KS",
            "Kansas",
            NoSpecificStatute,
            None,
            false,
            None,
            None,
            None,
            None,
            "federal FHA only",
        ),
        rule(
            "KY",
            "Kentucky",
            NoSpecificStatute,
            None,
            false,
            None,
            None,
            None,
            None,
            "federal FHA only",
        ),
        rule(
            "LA",
            "Louisiana",
            NoSpecificStatute,
            None,
            false,
            None,
            None,
            None,
            None,
            "federal FHA only",
        ),
        rule(
            "MA",
            "Massachusetts",
            NoSpecificStatute,
            None,
            false,
            None,
            None,
            None,
            None,
            "M.G.L. c. 121A — public housing only",
        ),
        rule(
            "MD",
            "Maryland",
            NoSpecificStatute,
            None,
            false,
            None,
            None,
            None,
            None,
            "Md. Code Tax-Property § 9-105 — tax credit only",
        ),
        rule(
            "ME",
            "Maine",
            NoSpecificStatute,
            None,
            false,
            None,
            None,
            None,
            None,
            "federal FHA only",
        ),
        rule(
            "MI",
            "Michigan",
            NoSpecificStatute,
            None,
            false,
            None,
            None,
            None,
            None,
            "federal FHA only",
        ),
        rule(
            "MN",
            "Minnesota",
            NoSpecificStatute,
            None,
            false,
            None,
            None,
            None,
            None,
            "federal FHA only",
        ),
        rule(
            "MO",
            "Missouri",
            NoSpecificStatute,
            None,
            false,
            None,
            None,
            None,
            None,
            "federal FHA only",
        ),
        rule(
            "MS",
            "Mississippi",
            NoSpecificStatute,
            None,
            false,
            None,
            None,
            None,
            None,
            "federal FHA only",
        ),
        rule(
            "MT",
            "Montana",
            NoSpecificStatute,
            None,
            false,
            None,
            None,
            None,
            None,
            "federal FHA only",
        ),
        rule(
            "NC",
            "North Carolina",
            NoSpecificStatute,
            None,
            false,
            None,
            None,
            None,
            None,
            "federal FHA only",
        ),
        rule(
            "ND",
            "North Dakota",
            NoSpecificStatute,
            None,
            false,
            None,
            None,
            None,
            None,
            "federal FHA only",
        ),
        rule(
            "NE",
            "Nebraska",
            NoSpecificStatute,
            None,
            false,
            None,
            None,
            None,
            None,
            "federal FHA only",
        ),
        rule(
            "NH",
            "New Hampshire",
            NoSpecificStatute,
            None,
            false,
            None,
            None,
            None,
            None,
            "federal FHA only",
        ),
        rule(
            "NJ",
            "New Jersey",
            StatewideConversionProtection,
            Some(62),
            true,
            Some(50_000),
            Some(40),
            Some(5),
            Some(1),
            "N.J.S.A. § 2A:18-61.22 (Senior Citizens & Disabled Protected Tenancy Act 1982)",
        ),
        rule(
            "NM",
            "New Mexico",
            NoSpecificStatute,
            None,
            false,
            None,
            None,
            None,
            None,
            "federal FHA only",
        ),
        rule(
            "NV",
            "Nevada",
            NoSpecificStatute,
            None,
            false,
            None,
            None,
            None,
            None,
            "federal FHA only",
        ),
        rule(
            "NY",
            "New York",
            MunicipalRentIncreaseExemption,
            Some(62),
            true,
            Some(50_000),
            None, // permanent SCRIE
            None,
            None,
            "NYC Admin Code § 26-509(b)(2) (SCRIE) + § 26-405(m)(1) (DRIE)",
        ),
        rule(
            "OH",
            "Ohio",
            NoSpecificStatute,
            None,
            false,
            None,
            None,
            None,
            None,
            "federal FHA only",
        ),
        rule(
            "OK",
            "Oklahoma",
            NoSpecificStatute,
            None,
            false,
            None,
            None,
            None,
            None,
            "federal FHA only",
        ),
        rule(
            "OR",
            "Oregon",
            JustCauseCoversNoCarveOut,
            None,
            true,
            None,
            None,
            None,
            None,
            "ORS § 90.427 (SB 608) — covers all tenants",
        ),
        rule(
            "PA",
            "Pennsylvania",
            JustCauseCoversNoCarveOut,
            Some(62),
            true,
            None,
            None,
            None,
            None,
            "68 Pa. C.S. § 250.504-A",
        ),
        rule(
            "RI",
            "Rhode Island",
            NoSpecificStatute,
            None,
            false,
            None,
            None,
            None,
            None,
            "federal FHA only",
        ),
        rule(
            "SC",
            "South Carolina",
            NoSpecificStatute,
            None,
            false,
            None,
            None,
            None,
            None,
            "federal FHA only",
        ),
        rule(
            "SD",
            "South Dakota",
            NoSpecificStatute,
            None,
            false,
            None,
            None,
            None,
            None,
            "federal FHA only",
        ),
        rule(
            "TN",
            "Tennessee",
            NoSpecificStatute,
            None,
            false,
            None,
            None,
            None,
            None,
            "federal FHA only",
        ),
        rule(
            "TX",
            "Texas",
            NoSpecificStatute,
            None,
            false,
            None,
            None,
            None,
            None,
            "federal FHA only",
        ),
        rule(
            "UT",
            "Utah",
            NoSpecificStatute,
            None,
            false,
            None,
            None,
            None,
            None,
            "federal FHA only",
        ),
        rule(
            "VA",
            "Virginia",
            NoSpecificStatute,
            None,
            false,
            None,
            None,
            None,
            None,
            "federal FHA only",
        ),
        rule(
            "VT",
            "Vermont",
            NoSpecificStatute,
            None,
            false,
            None,
            None,
            None,
            None,
            "federal FHA only",
        ),
        rule(
            "WA",
            "Washington",
            JustCauseCoversNoCarveOut,
            None,
            true,
            None,
            None,
            None,
            None,
            "RCW § 59.18.650 (HB 1236) — covers all tenants",
        ),
        rule(
            "WI",
            "Wisconsin",
            NoSpecificStatute,
            None,
            false,
            None,
            None,
            None,
            None,
            "federal FHA only",
        ),
        rule(
            "WV",
            "West Virginia",
            NoSpecificStatute,
            None,
            false,
            None,
            None,
            None,
            None,
            "federal FHA only",
        ),
        rule(
            "WY",
            "Wyoming",
            NoSpecificStatute,
            None,
            false,
            None,
            None,
            None,
            None,
            "federal FHA only",
        ),
    ];
    RULES.iter().map(|r| (r.state_code, r)).collect()
});

#[cfg(test)]
mod tests {
    use super::*;

    fn nj_eligible_input() -> SeniorDisabledCheckInput {
        SeniorDisabledCheckInput {
            state_code: "NJ".to_string(),
            tenant_age: 70,
            tenant_disabled: false,
            tenant_annual_income_dollars: 40_000,
            years_in_building: 10,
            building_unit_count: 20,
        }
    }

    #[test]
    fn table_covers_all_50_states_plus_dc() {
        assert_eq!(TABLE.len(), 51);
    }

    #[test]
    fn nj_eligible_senior_qualifies() {
        let r = check(&nj_eligible_input());
        assert!(r.eligible_for_protection);
        assert!(r.age_qualifies);
        assert_eq!(r.income_qualifies, Some(true));
        assert_eq!(r.building_size_qualifies, Some(true));
        assert_eq!(r.tenancy_length_qualifies, Some(true));
        assert_eq!(r.maximum_protection_years, Some(40));
        assert!(r.note.contains("40-year"));
    }

    #[test]
    fn nj_age_61_disqualifies_non_disabled() {
        let mut i = nj_eligible_input();
        i.tenant_age = 61;
        i.tenant_disabled = false;
        let r = check(&i);
        assert!(!r.eligible_for_protection);
        assert!(!r.age_qualifies);
        assert!(r
            .reasons_not_eligible
            .iter()
            .any(|s| s.contains("age 61 below minimum 62")));
    }

    #[test]
    fn nj_disabled_qualifies_at_any_age() {
        let mut i = nj_eligible_input();
        i.tenant_age = 30;
        i.tenant_disabled = true;
        let r = check(&i);
        assert!(r.eligible_for_protection);
        assert!(r.age_qualifies);
    }

    #[test]
    fn nj_income_50k_exact_boundary_qualifies() {
        let mut i = nj_eligible_input();
        i.tenant_annual_income_dollars = 50_000;
        let r = check(&i);
        assert!(r.eligible_for_protection);
        assert_eq!(r.income_qualifies, Some(true));
    }

    #[test]
    fn nj_income_50001_disqualifies() {
        let mut i = nj_eligible_input();
        i.tenant_annual_income_dollars = 50_001;
        let r = check(&i);
        assert!(!r.eligible_for_protection);
        assert_eq!(r.income_qualifies, Some(false));
    }

    #[test]
    fn nj_4_unit_building_disqualifies() {
        let mut i = nj_eligible_input();
        i.building_unit_count = 4;
        let r = check(&i);
        assert!(!r.eligible_for_protection);
        assert_eq!(r.building_size_qualifies, Some(false));
        assert!(r
            .reasons_not_eligible
            .iter()
            .any(|s| s.contains("4 units, below minimum 5")));
    }

    #[test]
    fn nj_short_tenancy_disqualifies() {
        let mut i = nj_eligible_input();
        i.years_in_building = 0;
        let r = check(&i);
        assert!(!r.eligible_for_protection);
        assert_eq!(r.tenancy_length_qualifies, Some(false));
    }

    #[test]
    fn nj_multiple_disqualifications_listed() {
        let mut i = nj_eligible_input();
        i.tenant_age = 30;
        i.tenant_disabled = false;
        i.tenant_annual_income_dollars = 100_000;
        i.building_unit_count = 2;
        i.years_in_building = 0;
        let r = check(&i);
        assert!(!r.eligible_for_protection);
        assert!(r.reasons_not_eligible.len() >= 3); // age + income + building + tenancy
    }

    #[test]
    fn ny_scrie_drie_no_max_protection_permanent() {
        // NY SCRIE/DRIE is permanent (no max_protection_years).
        let i = SeniorDisabledCheckInput {
            state_code: "NY".to_string(),
            tenant_age: 65,
            tenant_disabled: false,
            tenant_annual_income_dollars: 40_000,
            years_in_building: 5,
            building_unit_count: 50,
        };
        let r = check(&i);
        assert!(r.eligible_for_protection);
        assert!(r.maximum_protection_years.is_none());
        assert!(r.note.contains("permanent"));
    }

    #[test]
    fn ca_just_cause_no_senior_carveout() {
        // CA AB 1482 covers all tenants but has no senior carve-out.
        // The regime applies broadly but doesn't require specific
        // senior/disabled eligibility tests.
        let i = SeniorDisabledCheckInput {
            state_code: "CA".to_string(),
            tenant_age: 50,
            tenant_disabled: false,
            tenant_annual_income_dollars: 200_000,
            years_in_building: 1,
            building_unit_count: 100,
        };
        let r = check(&i);
        assert!(r.statute_available);
        assert!(r.eligible_for_protection); // no specific tests to fail
        assert!(r.note.contains("just-cause"));
    }

    #[test]
    fn tx_no_specific_statute() {
        let i = SeniorDisabledCheckInput {
            state_code: "TX".to_string(),
            tenant_age: 80,
            tenant_disabled: false,
            tenant_annual_income_dollars: 30_000,
            years_in_building: 10,
            building_unit_count: 50,
        };
        let r = check(&i);
        assert!(!r.statute_available);
        assert!(!r.eligible_for_protection);
        assert!(r.note.contains("federal FHA"));
    }

    #[test]
    fn ct_age_62_disability_qualifies() {
        // CT § 47a-23c — applies to seniors 62+ or disabled. No income
        // or other requirements.
        let i = SeniorDisabledCheckInput {
            state_code: "CT".to_string(),
            tenant_age: 62,
            tenant_disabled: false,
            tenant_annual_income_dollars: 75_000,
            years_in_building: 1,
            building_unit_count: 10,
        };
        let r = check(&i);
        assert!(r.eligible_for_protection);
    }

    #[test]
    fn pa_age_62_qualifies() {
        let i = SeniorDisabledCheckInput {
            state_code: "PA".to_string(),
            tenant_age: 62,
            tenant_disabled: false,
            tenant_annual_income_dollars: 60_000,
            years_in_building: 2,
            building_unit_count: 8,
        };
        let r = check(&i);
        assert!(r.eligible_for_protection);
    }

    #[test]
    fn unknown_state_handled() {
        let i = SeniorDisabledCheckInput {
            state_code: "ZZ".to_string(),
            tenant_age: 70,
            tenant_disabled: false,
            tenant_annual_income_dollars: 40_000,
            years_in_building: 5,
            building_unit_count: 10,
        };
        let r = check(&i);
        assert!(!r.statute_available);
        assert!(r.note.contains("unknown state code"));
    }

    #[test]
    fn lookup_case_insensitive() {
        assert!(lookup("NJ").is_some());
        assert!(lookup("nj").is_some());
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
    fn nj_only_state_with_statewide_conversion_protection() {
        let nj = lookup("NJ").unwrap();
        assert!(matches!(
            nj.regime,
            SeniorProtectionRegime::StatewideConversionProtection
        ));
        for r in TABLE.values() {
            if r.state_code != "NJ" {
                assert!(
                    !matches!(
                        r.regime,
                        SeniorProtectionRegime::StatewideConversionProtection
                    ),
                    "{} should not have StatewideConversionProtection",
                    r.state_code
                );
            }
        }
    }

    #[test]
    fn ny_only_state_with_municipal_rent_increase_exemption() {
        // NY (NYC SCRIE/DRIE) is uniquely on the municipal rent-increase
        // exemption regime.
        let ny = lookup("NY").unwrap();
        assert!(matches!(
            ny.regime,
            SeniorProtectionRegime::MunicipalRentIncreaseExemption
        ));
        for r in TABLE.values() {
            if r.state_code != "NY" {
                assert!(
                    !matches!(
                        r.regime,
                        SeniorProtectionRegime::MunicipalRentIncreaseExemption
                    ),
                    "{} should not have MunicipalRentIncreaseExemption",
                    r.state_code
                );
            }
        }
    }

    #[test]
    fn just_cause_no_carveout_states_pinned() {
        // CA, CT, DC, IL, OR, PA, WA cover all tenants with just-cause.
        for code in ["CA", "CT", "DC", "IL", "OR", "PA", "WA"] {
            let r = lookup(code).unwrap();
            assert!(
                matches!(r.regime, SeniorProtectionRegime::JustCauseCoversNoCarveOut),
                "{code} should be JustCauseCoversNoCarveOut"
            );
        }
    }

    #[test]
    fn nj_only_state_with_40_year_protection_or_below() {
        // NJ uniquely has the 40-year statutory cap.
        let nj = lookup("NJ").unwrap();
        assert_eq!(nj.max_protection_years, Some(40));
        for r in TABLE.values() {
            if r.state_code != "NJ" {
                assert!(
                    r.max_protection_years.is_none(),
                    "{} should not have max_protection_years",
                    r.state_code
                );
            }
        }
    }
}
