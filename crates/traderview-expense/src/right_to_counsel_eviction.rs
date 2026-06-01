//! State / municipal right-to-counsel in eviction proceedings — landlord
//! notice + compliance check. Distinct from `eviction_diversion_program`
//! (pre-filing mediation duty) — this module addresses the affirmative
//! statutory right of low-income tenants to court-appointed counsel
//! during the eviction proceeding itself.
//!
//! Two jurisdictions have established right-to-counsel programs with
//! concrete landlord-facing notice obligations:
//!
//! New York City (Local Law 136 of 2017, codified at NYC Admin Code
//! § 26-1301) — first-in-the-nation tenant RTC program. Income-eligible
//! tenants (gross household income ≤ 200% federal poverty level) receive
//! free full legal representation in Housing Court and NYCHA
//! administrative eviction proceedings. Brief legal services available
//! to ALL tenants regardless of income. Landlord must include in the
//! eviction petition / notice of petition a statement of tenant's right
//! to counsel.
//!
//! Washington (RCW 59.18.640, eff. 2021 per SB 5160) — first STATEWIDE
//! tenant RTC. Court must appoint counsel for indigent tenant (≤ 200%
//! federal poverty level OR receiving public assistance). 14-day pay-or-
//! quit notice must include specific statutory form language about
//! legal aid + dispute resolution centers + right to appointed counsel.
//! Court must advise eligible tenant of right + grant reasonable
//! continuance to obtain and prepare with appointed counsel.
//!
//! Default — no statewide RTC statute. Tenant must self-represent or
//! retain private counsel. Legal aid availability varies by jurisdiction.
//!
//! Citations: NYC Admin Code § 26-1301 (Local Law 136 of 2017); NYC
//! Admin Code § 26-1304 (income eligibility); RCW 59.18.640 (WA
//! statewide RTC); RCW 59.18.057 (WA 14-day notice form language);
//! Washington SB 5160 (eff. 2021).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    NewYorkCity,
    Washington,
    Default,
}

impl Regime {
    pub fn for_jurisdiction(state: &str, city: &str) -> Self {
        let st = state.trim().to_ascii_uppercase();
        let ct = city.trim().to_ascii_lowercase();
        match (st.as_str(), ct.as_str()) {
            ("NY", "new york") | ("NY", "nyc") => Self::NewYorkCity,
            ("WA", _) => Self::Washington,
            _ => Self::Default,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProceedingType {
    HousingCourt,
    NychaAdministrative,
    GeneralCivil,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RightToCounselInput {
    pub regime: Regime,
    pub tenant_household_income_cents: i64,
    /// Federal poverty line for the tenant's household size. Both regimes
    /// use 200% × this value as the income-eligibility threshold.
    pub federal_poverty_line_cents: i64,
    /// Whether the tenant receives means-tested public assistance
    /// (TANF/SSI/SNAP) — WA's alternative eligibility path.
    pub tenant_receives_public_assistance: bool,
    pub proceeding_type: ProceedingType,
    /// Whether the landlord included the required RTC notice in the
    /// eviction petition / 14-day notice.
    pub rtc_notice_provided: bool,
    /// Whether the required RTC notice contained the specific statutory
    /// form language (WA RCW 59.18.057 — legal aid + DRC + appointed
    /// counsel).
    pub rtc_notice_contains_statutory_form_language: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ViolationType {
    None,
    MissingRtcNotice,
    MissingStatutoryFormLanguage,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RightToCounselResult {
    pub regime: Regime,
    pub income_eligibility_threshold_cents: i64,
    pub tenant_income_eligible_for_rtc: bool,
    pub rtc_notice_required: bool,
    pub court_must_appoint_counsel: bool,
    pub brief_services_available_to_all: bool,
    pub violation: ViolationType,
    pub landlord_compliant: bool,
    pub citation: &'static str,
    pub note: String,
}

pub fn check(input: &RightToCounselInput) -> RightToCounselResult {
    match input.regime {
        Regime::NewYorkCity => nyc_check(input),
        Regime::Washington => wa_check(input),
        Regime::Default => default_check(input),
    }
}

fn nyc_check(input: &RightToCounselInput) -> RightToCounselResult {
    let threshold = (input.federal_poverty_line_cents as i128 * 200 / 100) as i64;
    let income_eligible = input.tenant_household_income_cents <= threshold;
    // NYC RTC applies to Housing Court + NYCHA administrative proceedings.
    let covered_proceeding = matches!(
        input.proceeding_type,
        ProceedingType::HousingCourt | ProceedingType::NychaAdministrative
    );
    if !covered_proceeding {
        return RightToCounselResult {
            regime: Regime::NewYorkCity,
            income_eligibility_threshold_cents: threshold,
            tenant_income_eligible_for_rtc: income_eligible,
            rtc_notice_required: false,
            court_must_appoint_counsel: false,
            brief_services_available_to_all: false,
            violation: ViolationType::None,
            landlord_compliant: true,
            citation: "NYC Admin Code § 26-1301 (Local Law 136 of 2017) — RTC applies only to Housing Court and NYCHA administrative proceedings",
            note: format!(
                "Proceeding type {:?} is not covered by NYC Local Law 136. RTC does not apply.",
                input.proceeding_type
            ),
        };
    }
    if !input.rtc_notice_provided {
        return RightToCounselResult {
            regime: Regime::NewYorkCity,
            income_eligibility_threshold_cents: threshold,
            tenant_income_eligible_for_rtc: income_eligible,
            rtc_notice_required: true,
            court_must_appoint_counsel: income_eligible,
            brief_services_available_to_all: true,
            violation: ViolationType::MissingRtcNotice,
            landlord_compliant: false,
            citation: "NYC Admin Code § 26-1301 — landlord must include notice of tenant's right to counsel in eviction petition / notice of petition",
            note: "Landlord did not include the required notice of tenant's right to counsel in the eviction petition.".to_string(),
        };
    }
    RightToCounselResult {
        regime: Regime::NewYorkCity,
        income_eligibility_threshold_cents: threshold,
        tenant_income_eligible_for_rtc: income_eligible,
        rtc_notice_required: true,
        court_must_appoint_counsel: income_eligible,
        brief_services_available_to_all: true,
        violation: ViolationType::None,
        landlord_compliant: true,
        citation: "NYC Admin Code § 26-1301 + § 26-1304 — NYC Local Law 136 RTC compliance OK",
        note: format!(
            "RTC notice provided. Tenant income {} cents {} 200%-FPL threshold {} cents → {} eligible for full representation. Brief services available regardless.",
            input.tenant_household_income_cents,
            if income_eligible { "≤" } else { ">" },
            threshold,
            if income_eligible { "" } else { "not " },
        ),
    }
}

fn wa_check(input: &RightToCounselInput) -> RightToCounselResult {
    let threshold = (input.federal_poverty_line_cents as i128 * 200 / 100) as i64;
    let income_eligible = input.tenant_household_income_cents <= threshold
        || input.tenant_receives_public_assistance;
    if !input.rtc_notice_provided {
        return RightToCounselResult {
            regime: Regime::Washington,
            income_eligibility_threshold_cents: threshold,
            tenant_income_eligible_for_rtc: income_eligible,
            rtc_notice_required: true,
            court_must_appoint_counsel: income_eligible,
            brief_services_available_to_all: false,
            violation: ViolationType::MissingRtcNotice,
            landlord_compliant: false,
            citation: "RCW 59.18.057 — landlord's 14-day pay-or-quit notice must include specific statutory form language about legal aid, dispute resolution centers, and right to appointed counsel",
            note: "Landlord did not provide the required RCW 59.18.057 form notice.".to_string(),
        };
    }
    if !input.rtc_notice_contains_statutory_form_language {
        return RightToCounselResult {
            regime: Regime::Washington,
            income_eligibility_threshold_cents: threshold,
            tenant_income_eligible_for_rtc: income_eligible,
            rtc_notice_required: true,
            court_must_appoint_counsel: income_eligible,
            brief_services_available_to_all: false,
            violation: ViolationType::MissingStatutoryFormLanguage,
            landlord_compliant: false,
            citation: "RCW 59.18.057 — notice must use the SPECIFIC statutory form language (legal aid + dispute resolution centers + appointed counsel)",
            note: "Notice was served but did not contain the specific statutory form language required by RCW 59.18.057.".to_string(),
        };
    }
    RightToCounselResult {
        regime: Regime::Washington,
        income_eligibility_threshold_cents: threshold,
        tenant_income_eligible_for_rtc: income_eligible,
        rtc_notice_required: true,
        court_must_appoint_counsel: income_eligible,
        brief_services_available_to_all: false,
        violation: ViolationType::None,
        landlord_compliant: true,
        citation: "RCW 59.18.640 + RCW 59.18.057 (SB 5160 eff. 2021) — first statewide tenant RTC; compliance OK",
        note: format!(
            "WA RTC notice with statutory form language provided. Tenant income {} cents (or public assistance {}); {} 200%-FPL threshold {} cents → {} indigent and {} entitled to appointed counsel.",
            input.tenant_household_income_cents,
            input.tenant_receives_public_assistance,
            if income_eligible { "≤" } else { ">" },
            threshold,
            if income_eligible { "is" } else { "is not" },
            if income_eligible { "is" } else { "is not" },
        ),
    }
}

fn default_check(input: &RightToCounselInput) -> RightToCounselResult {
    let threshold = (input.federal_poverty_line_cents as i128 * 200 / 100) as i64;
    RightToCounselResult {
        regime: Regime::Default,
        income_eligibility_threshold_cents: threshold,
        tenant_income_eligible_for_rtc: false,
        rtc_notice_required: false,
        court_must_appoint_counsel: false,
        brief_services_available_to_all: false,
        violation: ViolationType::None,
        landlord_compliant: true,
        citation:
            "No statewide or municipal right-to-counsel statute identified — tenants self-represent or retain private counsel",
        note: "Default regime: no statutory RTC. Tenant must self-represent or hire private counsel. Legal aid availability varies by jurisdiction.".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        regime: Regime,
        income: i64,
        fpl: i64,
        public_assistance: bool,
        proceeding: ProceedingType,
        notice: bool,
        form_lang: bool,
    ) -> RightToCounselInput {
        RightToCounselInput {
            regime,
            tenant_household_income_cents: income,
            federal_poverty_line_cents: fpl,
            tenant_receives_public_assistance: public_assistance,
            proceeding_type: proceeding,
            rtc_notice_provided: notice,
            rtc_notice_contains_statutory_form_language: form_lang,
        }
    }

    #[test]
    fn nyc_income_under_200_pct_fpl_eligible_compliant() {
        // FPL $15K, 200% threshold $30K. Tenant income $25K → eligible.
        let r = check(&input(
            Regime::NewYorkCity,
            25_000_00,
            15_000_00,
            false,
            ProceedingType::HousingCourt,
            true,
            false,
        ));
        assert!(r.tenant_income_eligible_for_rtc);
        assert!(r.court_must_appoint_counsel);
        assert_eq!(r.violation, ViolationType::None);
        assert!(r.brief_services_available_to_all);
    }

    #[test]
    fn nyc_income_above_200_pct_fpl_not_eligible_but_brief_services() {
        // FPL $15K, 200% threshold $30K. Tenant income $50K → not eligible
        // for full rep but still gets brief services.
        let r = check(&input(
            Regime::NewYorkCity,
            50_000_00,
            15_000_00,
            false,
            ProceedingType::HousingCourt,
            true,
            false,
        ));
        assert!(!r.tenant_income_eligible_for_rtc);
        assert!(!r.court_must_appoint_counsel);
        assert!(r.brief_services_available_to_all);
        assert_eq!(r.violation, ViolationType::None);
    }

    #[test]
    fn nyc_at_200_pct_boundary_eligible() {
        // Tenant income = $30K exactly, threshold = $30K. ≤ strict → eligible.
        let r = check(&input(
            Regime::NewYorkCity,
            30_000_00,
            15_000_00,
            false,
            ProceedingType::HousingCourt,
            true,
            false,
        ));
        assert!(r.tenant_income_eligible_for_rtc);
    }

    #[test]
    fn nyc_missing_notice_violation() {
        let r = check(&input(
            Regime::NewYorkCity,
            25_000_00,
            15_000_00,
            false,
            ProceedingType::HousingCourt,
            false,
            false,
        ));
        assert_eq!(r.violation, ViolationType::MissingRtcNotice);
        assert!(!r.landlord_compliant);
        assert!(r.citation.contains("§ 26-1301"));
    }

    #[test]
    fn nyc_nycha_administrative_covered() {
        let r = check(&input(
            Regime::NewYorkCity,
            25_000_00,
            15_000_00,
            false,
            ProceedingType::NychaAdministrative,
            true,
            false,
        ));
        assert_eq!(r.violation, ViolationType::None);
        assert!(r.court_must_appoint_counsel);
    }

    #[test]
    fn nyc_general_civil_not_covered() {
        // General civil court (not Housing Court / NYCHA) → outside RTC scope.
        let r = check(&input(
            Regime::NewYorkCity,
            25_000_00,
            15_000_00,
            false,
            ProceedingType::GeneralCivil,
            false,
            false,
        ));
        assert!(!r.rtc_notice_required);
        assert!(!r.court_must_appoint_counsel);
        assert_eq!(r.violation, ViolationType::None);
    }

    #[test]
    fn wa_income_under_threshold_eligible() {
        let r = check(&input(
            Regime::Washington,
            25_000_00,
            15_000_00,
            false,
            ProceedingType::HousingCourt,
            true,
            true,
        ));
        assert!(r.tenant_income_eligible_for_rtc);
        assert!(r.court_must_appoint_counsel);
        assert_eq!(r.violation, ViolationType::None);
    }

    #[test]
    fn wa_public_assistance_alternative_eligibility() {
        // Tenant income $80K — above 200% FPL ($30K threshold). But receives
        // public assistance (SNAP/TANF) → eligible under alternative path.
        let r = check(&input(
            Regime::Washington,
            80_000_00,
            15_000_00,
            true,
            ProceedingType::HousingCourt,
            true,
            true,
        ));
        assert!(r.tenant_income_eligible_for_rtc);
    }

    #[test]
    fn wa_high_income_no_public_assistance_not_eligible() {
        let r = check(&input(
            Regime::Washington,
            100_000_00,
            15_000_00,
            false,
            ProceedingType::HousingCourt,
            true,
            true,
        ));
        assert!(!r.tenant_income_eligible_for_rtc);
        assert!(!r.court_must_appoint_counsel);
    }

    #[test]
    fn wa_missing_notice_violation() {
        let r = check(&input(
            Regime::Washington,
            25_000_00,
            15_000_00,
            false,
            ProceedingType::HousingCourt,
            false,
            false,
        ));
        assert_eq!(r.violation, ViolationType::MissingRtcNotice);
        assert!(r.citation.contains("RCW 59.18.057"));
        assert!(r.citation.contains("legal aid"));
    }

    #[test]
    fn wa_missing_statutory_form_language_violation() {
        let r = check(&input(
            Regime::Washington,
            25_000_00,
            15_000_00,
            false,
            ProceedingType::HousingCourt,
            true,
            false,
        ));
        assert_eq!(r.violation, ViolationType::MissingStatutoryFormLanguage);
        assert!(r.citation.contains("SPECIFIC statutory form language"));
    }

    #[test]
    fn wa_no_brief_services_for_non_eligible() {
        // WA's RTC is income-based only; no brief-services-for-all
        // distinct from NYC.
        let r = check(&input(
            Regime::Washington,
            100_000_00,
            15_000_00,
            false,
            ProceedingType::HousingCourt,
            true,
            true,
        ));
        assert!(!r.brief_services_available_to_all);
    }

    #[test]
    fn default_no_obligation() {
        let r = check(&input(
            Regime::Default,
            25_000_00,
            15_000_00,
            false,
            ProceedingType::HousingCourt,
            false,
            false,
        ));
        assert_eq!(r.violation, ViolationType::None);
        assert!(!r.rtc_notice_required);
        assert!(!r.court_must_appoint_counsel);
        assert!(r.landlord_compliant);
        assert!(r.citation.contains("self-represent"));
    }

    #[test]
    fn jurisdiction_routing_nyc_wa_default() {
        assert_eq!(
            Regime::for_jurisdiction("NY", "New York"),
            Regime::NewYorkCity
        );
        assert_eq!(
            Regime::for_jurisdiction("NY", "NYC"),
            Regime::NewYorkCity
        );
        assert_eq!(
            Regime::for_jurisdiction("NY", "Buffalo"),
            Regime::Default
        );
        assert_eq!(Regime::for_jurisdiction("WA", "Seattle"), Regime::Washington);
        assert_eq!(Regime::for_jurisdiction("WA", "Spokane"), Regime::Washington);
        assert_eq!(Regime::for_jurisdiction("CA", "LA"), Regime::Default);
    }

    #[test]
    fn jurisdiction_routing_case_insensitive() {
        assert_eq!(
            Regime::for_jurisdiction("ny", "new york"),
            Regime::NewYorkCity
        );
        assert_eq!(Regime::for_jurisdiction("wa", "any"), Regime::Washington);
    }

    #[test]
    fn only_nyc_has_brief_services_for_all() {
        let nyc = check(&input(
            Regime::NewYorkCity,
            100_000_00,
            15_000_00,
            false,
            ProceedingType::HousingCourt,
            true,
            false,
        ));
        let wa = check(&input(
            Regime::Washington,
            100_000_00,
            15_000_00,
            false,
            ProceedingType::HousingCourt,
            true,
            true,
        ));
        let d = check(&input(
            Regime::Default,
            100_000_00,
            15_000_00,
            false,
            ProceedingType::HousingCourt,
            true,
            true,
        ));
        assert!(nyc.brief_services_available_to_all);
        assert!(!wa.brief_services_available_to_all);
        assert!(!d.brief_services_available_to_all);
    }

    #[test]
    fn only_wa_has_public_assistance_alternative() {
        // Same high-income + public assistance scenario. WA: eligible.
        // NYC: not eligible (NYC only tests income).
        let wa = check(&input(
            Regime::Washington,
            100_000_00,
            15_000_00,
            true,
            ProceedingType::HousingCourt,
            true,
            true,
        ));
        let nyc = check(&input(
            Regime::NewYorkCity,
            100_000_00,
            15_000_00,
            true,
            ProceedingType::HousingCourt,
            true,
            false,
        ));
        assert!(wa.tenant_income_eligible_for_rtc);
        assert!(!nyc.tenant_income_eligible_for_rtc);
    }

    #[test]
    fn only_wa_has_statutory_form_language_requirement() {
        // Same notice-provided-but-no-form-language scenario.
        let wa = check(&input(
            Regime::Washington,
            25_000_00,
            15_000_00,
            false,
            ProceedingType::HousingCourt,
            true,
            false,
        ));
        let nyc = check(&input(
            Regime::NewYorkCity,
            25_000_00,
            15_000_00,
            false,
            ProceedingType::HousingCourt,
            true,
            false,
        ));
        assert_eq!(wa.violation, ViolationType::MissingStatutoryFormLanguage);
        assert_eq!(nyc.violation, ViolationType::None);
    }

    #[test]
    fn income_threshold_200_pct_invariant() {
        let nyc = check(&input(
            Regime::NewYorkCity,
            0,
            15_000_00,
            false,
            ProceedingType::HousingCourt,
            true,
            false,
        ));
        let wa = check(&input(
            Regime::Washington,
            0,
            15_000_00,
            false,
            ProceedingType::HousingCourt,
            true,
            true,
        ));
        assert_eq!(nyc.income_eligibility_threshold_cents, 30_000_00);
        assert_eq!(wa.income_eligibility_threshold_cents, 30_000_00);
    }

    #[test]
    fn citations_pin_correct_authorities() {
        let nyc = check(&input(
            Regime::NewYorkCity,
            25_000_00,
            15_000_00,
            false,
            ProceedingType::HousingCourt,
            true,
            false,
        ));
        assert!(nyc.citation.contains("§ 26-1301"));
        assert!(nyc.citation.contains("Local Law 136"));

        let wa = check(&input(
            Regime::Washington,
            25_000_00,
            15_000_00,
            false,
            ProceedingType::HousingCourt,
            true,
            true,
        ));
        assert!(wa.citation.contains("RCW 59.18.640"));
        assert!(wa.citation.contains("SB 5160"));
    }
}
