//! Tenant criminal-background-screening compliance framework.
//!
//! HUD's April 4, 2016 "Application of Fair Housing Act Standards to the Use of Criminal
//! Records by Providers of Housing and Real Estate-Related Transactions" guidance
//! memorandum (Helen Kanovsky, General Counsel) establishes that blanket criminal-history
//! bans create a disparate impact on protected classes (race, color, national origin) due
//! to racially disproportionate incarceration rates and may violate the Fair Housing Act
//! (42 U.S.C. § 3601 et seq.) absent a substantial, legitimate, nondiscriminatory
//! interest backed by individualized assessment. Multiple state and local laws have since
//! codified Fair Chance Housing protections that go further than the federal floor.
//!
//! Jurisdictional grid:
//!
//! - HUD 2016 GUIDANCE (federal floor): arrest records cannot serve as basis for adverse
//!   action (per se discriminatory because arrest is not proof of criminal conduct);
//!   conviction-record bans subject to disparate-impact analysis; individualized
//!   assessment required (nature/severity of offense, time elapsed, relevance to tenancy).
//! - NYC LOCAL LAW 24 of 2024 ("Fair Chance for Housing Act"): effective Jan 1, 2025.
//!   Prohibits consideration of criminal history until tenant's other qualifications are
//!   determined. 3-year lookback for misdemeanors, 5-year lookback for felonies. Sex
//!   crimes only convictions categorically considerable.
//! - CA AB 2052 (Cal. Civ. Code § 1786.21 + § 12955): sequential-screening regime —
//!   criminal background check only after applicant meets other qualifications. 7-year
//!   lookback for non-felony, conviction-only consideration (no arrests).
//! - NJ FAIR CHANCE IN HOUSING ACT (P.L. 2021, c. 197, codified at N.J.S.A. 46:8-52 et
//!   seq.): housing providers cannot ask about criminal history before extending
//!   conditional offer. Individualized assessment required if criminal history considered
//!   after conditional offer.
//! - IL HB 4366 (2024): Fair Housing Act amendment restricting criminal-history screening
//!   for federally-assisted housing.
//! - DEFAULT: HUD 2016 guidance federal floor; FHA disparate-impact framework applies.
//!
//! Citations (verified per WebSearch 2026-06-02):
//! - HUD: hud.gov/sites/documents/HUD_OGCGUIDAPPFHASTANDCR.PDF (April 4, 2016 guidance)
//! - NYC LL 24: woodslaw.com/nyc-fair-chance-housing-act-what-co-op-condo-boards-must-know-for-2025/
//! - NJ FCHA: njoag.gov/about/divisions-and-offices/division-on-civil-rights-home/know-the-law/fair-chance-in-housing-act/
//! - CA Fair Chance: rentsafe.lease/california-tenant-background-check-laws/

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    NewYorkCityLocalLaw24,
    California,
    NewJersey,
    Illinois,
    HudFederalFloor,
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScreeningStage {
    /// Pre-application or initial application — criminal-history inquiry forbidden in
    /// fair-chance jurisdictions (CA + NJ + NYC LL 24).
    PreApplicationOrInitialApplication,
    /// After other qualifications confirmed (income, references, prior tenancy) — fair-
    /// chance jurisdictions permit conditional criminal-background inquiry at this stage.
    AfterOtherQualificationsConfirmedConditionalStage,
    /// Post-tenancy — adverse action against existing tenant requires distinct procedural
    /// compliance.
    PostTenancyAdverseAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordTypeConsidered {
    /// Arrest record without conviction — per se discriminatory under HUD 2016 guidance.
    ArrestRecordWithoutConviction,
    /// Misdemeanor conviction within jurisdiction-specific lookback window.
    MisdemeanorConvictionWithinLookbackWindow,
    /// Misdemeanor conviction outside lookback window.
    MisdemeanorConvictionOutsideLookbackWindow,
    /// Felony conviction within jurisdiction-specific lookback window.
    FelonyConvictionWithinLookbackWindow,
    /// Felony conviction outside lookback window.
    FelonyConvictionOutsideLookbackWindow,
    /// Sex-offense conviction — categorically considerable under NYC LL 24 + most state
    /// fair-chance laws.
    SexOffenseConvictionCategoricallyConsiderable,
    /// No criminal history identified.
    NoCriminalHistoryIdentified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IndividualizedAssessmentStatus {
    /// HUD 2016 individualized assessment performed: nature/severity, time elapsed,
    /// relevance to tenancy considered with documentation.
    AssessmentPerformedAndDocumented,
    /// Blanket policy applied without individualized assessment — per HUD 2016 disparate-
    /// impact framework, presumptively unlawful.
    BlanketPolicyWithoutIndividualizedAssessment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    NoCriminalHistoryNoSection804OrFchaViolation,
    CompliantSequentialScreeningAndIndividualizedAssessment,
    SexOffenseExceptionCategoricallyPermissiblePerJurisdiction,
    ArrestRecordReliancePerSeDiscriminatoryHud2016,
    PreApplicationInquiryViolatesFairChanceLaw,
    BlanketBanWithoutIndividualizedAssessmentDisparateImpact,
    LookbackWindowExceededFairChanceViolation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub jurisdiction: Jurisdiction,
    pub screening_stage: ScreeningStage,
    pub record_type_considered: RecordTypeConsidered,
    pub individualized_assessment_status: IndividualizedAssessmentStatus,
    pub days_since_conviction: u32,
}

pub type RentalTenantCriminalBackgroundScreeningInput = Input;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub severity: Severity,
    pub misdemeanor_lookback_days: u32,
    pub felony_lookback_days: u32,
    pub note: String,
}

pub type RentalTenantCriminalBackgroundScreeningOutput = Output;
pub type RentalTenantCriminalBackgroundScreeningResult = Output;

const NYC_MISDEMEANOR_LOOKBACK_DAYS: u32 = 3 * 365;
const NYC_FELONY_LOOKBACK_DAYS: u32 = 5 * 365;
const CA_LOOKBACK_DAYS: u32 = 7 * 365;
const NJ_MISDEMEANOR_LOOKBACK_DAYS: u32 = 365;
const NJ_FELONY_LOOKBACK_DAYS: u32 = 4 * 365;
const HUD_RECOMMENDED_LOOKBACK_DAYS: u32 = 7 * 365;

#[must_use]
pub fn check(input: &Input) -> Output {
    let (misdemeanor_lookback, felony_lookback) = lookback_windows(input.jurisdiction);

    if matches!(
        input.record_type_considered,
        RecordTypeConsidered::NoCriminalHistoryIdentified
    ) {
        return Output {
            severity: Severity::NoCriminalHistoryNoSection804OrFchaViolation,
            misdemeanor_lookback_days: misdemeanor_lookback,
            felony_lookback_days: felony_lookback,
            note: "No criminal history identified — § 804 Fair Housing Act analysis not \
                   triggered. Confirm that the screening service does not return arrest \
                   records that the landlord could nonetheless rely on; HUD 2016 guidance \
                   prohibits arrest-record reliance regardless of jurisdiction."
                .to_string(),
        };
    }

    if matches!(
        input.record_type_considered,
        RecordTypeConsidered::ArrestRecordWithoutConviction
    ) {
        return Output {
            severity: Severity::ArrestRecordReliancePerSeDiscriminatoryHud2016,
            misdemeanor_lookback_days: misdemeanor_lookback,
            felony_lookback_days: felony_lookback,
            note: "PER SE VIOLATION under HUD 2016 guidance. Arrest records are not proof of \
                   criminal conduct and cannot serve as a basis for adverse housing decision; \
                   reliance on arrest records is per se discriminatory regardless of \
                   jurisdiction. Tenant entitled to Fair Housing Act complaint + 42 U.S.C. \
                   § 3613 private right of action + HUD administrative complaint. Estimated \
                   exposure: actual damages + emotional-distress damages + punitive damages \
                   + attorney fees + civil penalty $19,787 (first violation) under 42 U.S.C. \
                   § 3612(g)(3) inflation-adjusted."
                .to_string(),
        };
    }

    if matches!(
        input.record_type_considered,
        RecordTypeConsidered::SexOffenseConvictionCategoricallyConsiderable
    ) {
        return Output {
            severity: Severity::SexOffenseExceptionCategoricallyPermissiblePerJurisdiction,
            misdemeanor_lookback_days: misdemeanor_lookback,
            felony_lookback_days: felony_lookback,
            note: "Sex-offense conviction CATEGORICALLY CONSIDERABLE under NYC Local Law 24 \
                   (only conviction type the housing provider may consider categorically) + \
                   most state fair-chance laws. National Sex Offender Registry per Adam \
                   Walsh Child Protection and Safety Act of 2006 (Pub. L. 109-248) provides \
                   the authoritative record. Adverse action permitted without lookback or \
                   individualized assessment, BUT lookback-window-based reasonableness \
                   analysis may still apply under FHA disparate-impact framework."
                .to_string(),
        };
    }

    if matches!(
        input.jurisdiction,
        Jurisdiction::NewYorkCityLocalLaw24 | Jurisdiction::California | Jurisdiction::NewJersey
    ) && matches!(
        input.screening_stage,
        ScreeningStage::PreApplicationOrInitialApplication
    ) {
        return Output {
            severity: Severity::PreApplicationInquiryViolatesFairChanceLaw,
            misdemeanor_lookback_days: misdemeanor_lookback,
            felony_lookback_days: felony_lookback,
            note: format!(
                "Fair Chance Housing Law VIOLATION. {} prohibits criminal-history inquiry at \
                 the pre-application or initial-application stage. Landlord must first \
                 determine the applicant's other qualifications (income, references, prior \
                 tenancy) before conducting any criminal-background check. Estimated \
                 exposure: civil penalty + actual damages + emotional-distress + attorney \
                 fees + injunctive relief.",
                jurisdiction_label(input.jurisdiction)
            ),
        };
    }

    let applicable_lookback = match input.record_type_considered {
        RecordTypeConsidered::MisdemeanorConvictionWithinLookbackWindow
        | RecordTypeConsidered::MisdemeanorConvictionOutsideLookbackWindow => misdemeanor_lookback,
        RecordTypeConsidered::FelonyConvictionWithinLookbackWindow
        | RecordTypeConsidered::FelonyConvictionOutsideLookbackWindow => felony_lookback,
        _ => felony_lookback,
    };

    if input.days_since_conviction > applicable_lookback {
        return Output {
            severity: Severity::LookbackWindowExceededFairChanceViolation,
            misdemeanor_lookback_days: misdemeanor_lookback,
            felony_lookback_days: felony_lookback,
            note: format!(
                "Conviction occurred {} days ago, OUTSIDE the {}-day jurisdictional lookback \
                 window ({} years for the applicable record type). Adverse action based on \
                 this conviction violates Fair Chance Housing Law in {}. Recommend lookback \
                 windows by jurisdiction: NYC LL 24 = 3 yrs (misdemeanor) / 5 yrs (felony), \
                 CA = 7 yrs, NJ = 1 yr (misdemeanor) / 4 yrs (indictable felony). HUD 2016 \
                 guidance suggests 7-year window as outer reasonableness bound.",
                input.days_since_conviction,
                applicable_lookback,
                applicable_lookback / 365,
                jurisdiction_label(input.jurisdiction)
            ),
        };
    }

    if matches!(
        input.individualized_assessment_status,
        IndividualizedAssessmentStatus::BlanketPolicyWithoutIndividualizedAssessment
    ) {
        return Output {
            severity: Severity::BlanketBanWithoutIndividualizedAssessmentDisparateImpact,
            misdemeanor_lookback_days: misdemeanor_lookback,
            felony_lookback_days: felony_lookback,
            note: "Blanket criminal-history ban applied without individualized assessment. \
                   Under HUD 2016 guidance, blanket bans are PRESUMPTIVELY UNLAWFUL because \
                   they create a disparate impact on protected classes (race, color, national \
                   origin) due to racially disproportionate incarceration rates. Landlord \
                   must perform individualized assessment considering: (a) NATURE and \
                   SEVERITY of offense; (b) TIME elapsed since offense; (c) RELEVANCE of \
                   offense to safety of property, residents, or staff. Document all three \
                   factors. Estimated exposure: 42 U.S.C. § 3613 disparate-impact action + \
                   actual + emotional-distress + punitive damages + attorney fees."
                .to_string(),
        };
    }

    Output {
        severity: Severity::CompliantSequentialScreeningAndIndividualizedAssessment,
        misdemeanor_lookback_days: misdemeanor_lookback,
        felony_lookback_days: felony_lookback,
        note: format!(
            "Compliant: sequential screening (criminal-background after other qualifications \
             confirmed) + individualized assessment per HUD 2016 framework. Document the \
             three-factor analysis: (1) nature/severity, (2) time elapsed ({} days), (3) \
             relevance to tenancy. Retain documentation for {} years (statute of limitations \
             for FHA disparate-impact actions = 2 years post-discovery; document retention 5+ \
             years is industry best practice).",
            input.days_since_conviction, 5
        ),
    }
}

fn lookback_windows(jurisdiction: Jurisdiction) -> (u32, u32) {
    match jurisdiction {
        Jurisdiction::NewYorkCityLocalLaw24 => {
            (NYC_MISDEMEANOR_LOOKBACK_DAYS, NYC_FELONY_LOOKBACK_DAYS)
        }
        Jurisdiction::California => (CA_LOOKBACK_DAYS, CA_LOOKBACK_DAYS),
        Jurisdiction::NewJersey => (NJ_MISDEMEANOR_LOOKBACK_DAYS, NJ_FELONY_LOOKBACK_DAYS),
        Jurisdiction::Illinois | Jurisdiction::HudFederalFloor | Jurisdiction::Default => {
            (HUD_RECOMMENDED_LOOKBACK_DAYS, HUD_RECOMMENDED_LOOKBACK_DAYS)
        }
    }
}

fn jurisdiction_label(jurisdiction: Jurisdiction) -> &'static str {
    match jurisdiction {
        Jurisdiction::NewYorkCityLocalLaw24 => "NYC Local Law 24 of 2024",
        Jurisdiction::California => "California (AB 2052 + Cal. Civ. Code § 1786.21)",
        Jurisdiction::NewJersey => "New Jersey Fair Chance in Housing Act (N.J.S.A. 46:8-52)",
        Jurisdiction::Illinois => "Illinois (HB 4366 2024 + Fair Housing Act)",
        Jurisdiction::HudFederalFloor => "HUD 2016 guidance (federal floor)",
        Jurisdiction::Default => "Default (HUD 2016 guidance + 42 U.S.C. § 3601 et seq.)",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Input {
        Input {
            jurisdiction: Jurisdiction::NewYorkCityLocalLaw24,
            screening_stage: ScreeningStage::AfterOtherQualificationsConfirmedConditionalStage,
            record_type_considered: RecordTypeConsidered::MisdemeanorConvictionWithinLookbackWindow,
            individualized_assessment_status:
                IndividualizedAssessmentStatus::AssessmentPerformedAndDocumented,
            days_since_conviction: 365,
        }
    }

    #[test]
    fn no_criminal_history_no_violation() {
        let mut input = base();
        input.record_type_considered = RecordTypeConsidered::NoCriminalHistoryIdentified;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::NoCriminalHistoryNoSection804OrFchaViolation
        );
        assert!(output.note.contains("§ 804"));
        assert!(output.note.contains("HUD 2016"));
    }

    #[test]
    fn arrest_record_reliance_per_se_discriminatory() {
        let mut input = base();
        input.record_type_considered = RecordTypeConsidered::ArrestRecordWithoutConviction;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::ArrestRecordReliancePerSeDiscriminatoryHud2016
        );
        assert!(output.note.contains("PER SE"));
        assert!(output.note.contains("42 U.S.C. § 3613"));
        assert!(output.note.contains("§ 3612(g)(3)"));
        assert!(output.note.contains("$19,787"));
    }

    #[test]
    fn sex_offense_categorically_considerable() {
        let mut input = base();
        input.record_type_considered =
            RecordTypeConsidered::SexOffenseConvictionCategoricallyConsiderable;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::SexOffenseExceptionCategoricallyPermissiblePerJurisdiction
        );
        assert!(output.note.contains("Adam Walsh"));
        assert!(output.note.contains("Pub. L. 109-248"));
    }

    #[test]
    fn nyc_pre_application_inquiry_violates_local_law_24() {
        let mut input = base();
        input.screening_stage = ScreeningStage::PreApplicationOrInitialApplication;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::PreApplicationInquiryViolatesFairChanceLaw
        );
        assert!(output.note.contains("NYC Local Law 24"));
    }

    #[test]
    fn california_pre_application_inquiry_violates_ab_2052() {
        let mut input = base();
        input.jurisdiction = Jurisdiction::California;
        input.screening_stage = ScreeningStage::PreApplicationOrInitialApplication;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::PreApplicationInquiryViolatesFairChanceLaw
        );
        assert!(output.note.contains("AB 2052"));
        assert!(output.note.contains("Cal. Civ. Code § 1786.21"));
    }

    #[test]
    fn new_jersey_pre_application_inquiry_violates_fcha() {
        let mut input = base();
        input.jurisdiction = Jurisdiction::NewJersey;
        input.screening_stage = ScreeningStage::PreApplicationOrInitialApplication;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::PreApplicationInquiryViolatesFairChanceLaw
        );
        assert!(output.note.contains("N.J.S.A. 46:8-52"));
    }

    #[test]
    fn nyc_misdemeanor_3_year_lookback_window() {
        let input = base();
        let output = check(&input);
        assert_eq!(
            output.misdemeanor_lookback_days,
            NYC_MISDEMEANOR_LOOKBACK_DAYS
        );
    }

    #[test]
    fn nyc_felony_5_year_lookback_window() {
        let input = base();
        let output = check(&input);
        assert_eq!(output.felony_lookback_days, NYC_FELONY_LOOKBACK_DAYS);
    }

    #[test]
    fn california_7_year_lookback_both_types() {
        let mut input = base();
        input.jurisdiction = Jurisdiction::California;
        let output = check(&input);
        assert_eq!(output.misdemeanor_lookback_days, CA_LOOKBACK_DAYS);
        assert_eq!(output.felony_lookback_days, CA_LOOKBACK_DAYS);
    }

    #[test]
    fn nj_misdemeanor_1_year_lookback() {
        let mut input = base();
        input.jurisdiction = Jurisdiction::NewJersey;
        let output = check(&input);
        assert_eq!(
            output.misdemeanor_lookback_days,
            NJ_MISDEMEANOR_LOOKBACK_DAYS
        );
    }

    #[test]
    fn nj_felony_4_year_lookback() {
        let mut input = base();
        input.jurisdiction = Jurisdiction::NewJersey;
        let output = check(&input);
        assert_eq!(output.felony_lookback_days, NJ_FELONY_LOOKBACK_DAYS);
    }

    #[test]
    fn nyc_misdemeanor_outside_lookback_window_violation() {
        let mut input = base();
        input.record_type_considered =
            RecordTypeConsidered::MisdemeanorConvictionOutsideLookbackWindow;
        input.days_since_conviction = 4 * 365;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::LookbackWindowExceededFairChanceViolation
        );
    }

    #[test]
    fn nyc_misdemeanor_at_3_year_boundary_compliant() {
        let mut input = base();
        input.days_since_conviction = 3 * 365;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::CompliantSequentialScreeningAndIndividualizedAssessment
        );
    }

    #[test]
    fn blanket_ban_without_individualized_assessment_violation() {
        let mut input = base();
        input.individualized_assessment_status =
            IndividualizedAssessmentStatus::BlanketPolicyWithoutIndividualizedAssessment;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::BlanketBanWithoutIndividualizedAssessmentDisparateImpact
        );
        assert!(output.note.contains("disparate impact"));
        assert!(output.note.contains("NATURE"));
        assert!(output.note.contains("TIME"));
        assert!(output.note.contains("RELEVANCE"));
    }

    #[test]
    fn compliant_sequential_screening_and_individualized_assessment() {
        let input = base();
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::CompliantSequentialScreeningAndIndividualizedAssessment
        );
        assert!(output.note.contains("sequential screening"));
        assert!(output.note.contains("individualized assessment"));
    }

    #[test]
    fn felony_within_lookback_window_compliant() {
        let mut input = base();
        input.record_type_considered = RecordTypeConsidered::FelonyConvictionWithinLookbackWindow;
        input.days_since_conviction = 4 * 365;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::CompliantSequentialScreeningAndIndividualizedAssessment
        );
    }

    #[test]
    fn nyc_felony_at_5_year_boundary_compliant() {
        let mut input = base();
        input.record_type_considered = RecordTypeConsidered::FelonyConvictionWithinLookbackWindow;
        input.days_since_conviction = 5 * 365;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::CompliantSequentialScreeningAndIndividualizedAssessment
        );
    }

    #[test]
    fn nyc_felony_outside_5_year_window_violation() {
        let mut input = base();
        input.record_type_considered = RecordTypeConsidered::FelonyConvictionOutsideLookbackWindow;
        input.days_since_conviction = 6 * 365;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::LookbackWindowExceededFairChanceViolation
        );
    }

    #[test]
    fn nyc_misdemeanor_lookback_constant_pins_3_years() {
        assert_eq!(NYC_MISDEMEANOR_LOOKBACK_DAYS, 3 * 365);
    }

    #[test]
    fn nyc_felony_lookback_constant_pins_5_years() {
        assert_eq!(NYC_FELONY_LOOKBACK_DAYS, 5 * 365);
    }

    #[test]
    fn ca_lookback_constant_pins_7_years() {
        assert_eq!(CA_LOOKBACK_DAYS, 7 * 365);
    }

    #[test]
    fn nj_misdemeanor_lookback_constant_pins_1_year() {
        assert_eq!(NJ_MISDEMEANOR_LOOKBACK_DAYS, 365);
    }

    #[test]
    fn nj_felony_lookback_constant_pins_4_years() {
        assert_eq!(NJ_FELONY_LOOKBACK_DAYS, 4 * 365);
    }

    #[test]
    fn hud_default_lookback_constant_pins_7_years() {
        assert_eq!(HUD_RECOMMENDED_LOOKBACK_DAYS, 7 * 365);
    }

    #[test]
    fn arrest_record_overrides_compliant_individualized_assessment() {
        let mut input = base();
        input.record_type_considered = RecordTypeConsidered::ArrestRecordWithoutConviction;
        input.individualized_assessment_status =
            IndividualizedAssessmentStatus::AssessmentPerformedAndDocumented;
        let output = check(&input);
        // Arrest reliance is per se discriminatory regardless of assessment quality
        assert_eq!(
            output.severity,
            Severity::ArrestRecordReliancePerSeDiscriminatoryHud2016
        );
    }

    #[test]
    fn sex_offense_overrides_lookback_window_analysis() {
        let mut input = base();
        input.record_type_considered =
            RecordTypeConsidered::SexOffenseConvictionCategoricallyConsiderable;
        input.days_since_conviction = 30 * 365; // 30 years ago
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::SexOffenseExceptionCategoricallyPermissiblePerJurisdiction
        );
    }

    #[test]
    fn no_history_overrides_all_other_branches() {
        let mut input = base();
        input.record_type_considered = RecordTypeConsidered::NoCriminalHistoryIdentified;
        input.screening_stage = ScreeningStage::PreApplicationOrInitialApplication;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::NoCriminalHistoryNoSection804OrFchaViolation
        );
    }
}
