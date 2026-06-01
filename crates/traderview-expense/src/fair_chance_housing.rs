//! Fair-chance-in-housing — landlord criminal-background-check restrictions.
//!
//! Three U.S. jurisdictions have enacted specific statutes restricting how
//! landlords may consider applicants' criminal history. The federal FCRA
//! sets a 7-year ceiling on credit-reporting-agency reports of arrests, but
//! convictions themselves have no federal time limit (15 U.S.C. § 1681c).
//! The state and city statutes layered on top of FCRA materially change
//! what landlords may inquire about and when.
//!
//! Regimes:
//!
//! **NewJerseyFcha** (S250 / A1919 / N.J.S.A. 46:8-52 et seq., effective
//! 2022-01-01) — the strictest pre-conditional-offer rule. Landlord
//! cannot inquire about criminal history on the application or verbally
//! before extending a conditional offer of rental. After the conditional
//! offer, landlord may consider only **specific** convictions per the FCHA
//! formula (murder, aggravated sexual assault, kidnapping, arson, human
//! trafficking, terrorism = forever; 1st-degree indictable = 6 years; 2nd-
//! and 3rd-degree indictable = 4 years; 4th-degree indictable = 1 year;
//! manufacturing/distribution = 5 years; sex-offender-registry = duration
//! of registration). Withdrawal of conditional offer requires individualized
//! assessment + written notice + 30-day appeal window.
//!
//! **NewYorkCityFchha** (NYC Local Law 24 of 2024, NYC Admin Code §
//! 8-107.1, effective 2025-01-01) — strict lookback windows: felony 5
//! years from release date (or sentencing date if no incarceration);
//! misdemeanor 3 years from release/sentencing. Convictions that require
//! sex-offender-registry registration may be considered regardless of
//! age. **OFF-LIMITS regardless of age**: arrests, pending cases,
//! adjournments in contemplation of dismissal (ACDs), youthful-offender
//! adjudications, juvenile-delinquency adjudications, sealed/expunged
//! convictions, convictions vacated or nullified, executive-pardoned
//! convictions, and convictions of non-criminal violations (e.g.,
//! disorderly conduct).
//!
//! **CaliforniaFeha** (Civ. Code § 1786.18 + 2 Cal. Code Regs. § 12266)
//! — no blanket ban permitted; landlords must perform individualized
//! assessment considering nature/severity of offense, time elapsed,
//! evidence of rehabilitation. The Civ. Code § 1786.18 7-year CRA
//! reporting limit applies to information furnished to landlord (with
//! conviction-record carve-out). Failure to perform individualized
//! assessment is a Fair Employment and Housing Act violation.
//!
//! **Default** — federal FCRA 7-year ceiling on CRA reports of arrests
//! (15 U.S.C. § 1681c). Conviction reports themselves have NO federal
//! time limit. State and local fair-housing laws may apply.
//!
//! Citations: N.J.S.A. 46:8-52 et seq. (FCHA); NYC Admin Code § 8-107.1
//! (Local Law 24 of 2024); Cal. Civ. Code § 1786.18; 2 Cal. Code Regs.
//! § 12266; 15 U.S.C. § 1681c (FCRA reporting limit).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    NewJerseyFcha,
    NewYorkCityFchha,
    CaliforniaFeha,
    Default,
}

impl Regime {
    pub fn for_jurisdiction(state: &str, city: &str) -> Self {
        let st = state.trim().to_ascii_uppercase();
        let ct = city.trim().to_ascii_lowercase();
        match (st.as_str(), ct.as_str()) {
            ("NJ", _) => Self::NewJerseyFcha,
            ("NY", "new york") | ("NY", "nyc") => Self::NewYorkCityFchha,
            ("CA", _) => Self::CaliforniaFeha,
            _ => Self::Default,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConvictionType {
    /// Conviction triggering sex-offender registration — most fair-chance
    /// statutes treat this as always-considerable.
    SexOffenderRegistry,
    Felony,
    Misdemeanor,
    /// Non-criminal violation (e.g., NY disorderly conduct, traffic
    /// violation). NYC FCHHA bars consideration of these entirely.
    NonCriminalViolation,
    /// Arrest without conviction. Universally barred under all 3 regimes.
    Arrest,
    /// Pending case (no conviction yet). Barred under NJ FCHA + NYC FCHHA.
    PendingCase,
    /// Sealed or expunged conviction. Barred under all 3 regimes.
    SealedExpunged,
    /// Juvenile delinquency / youthful offender adjudication. Barred under
    /// NJ FCHA + NYC FCHHA.
    JuvenileAdjudication,
    /// Adjournment in Contemplation of Dismissal — NY procedural disposition.
    /// NYC FCHHA bars consideration of ACDs.
    AcdDisposition,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FairChanceInput {
    pub regime: Regime,
    /// Whether the landlord has extended a conditional offer of rental.
    /// Pre-offer inquiry is barred under NJ FCHA + NYC FCHHA.
    pub conditional_offer_made: bool,
    /// Whether the landlord has inquired about criminal history before
    /// the conditional offer was made. Triggers the pre-offer-inquiry
    /// violation under NJ FCHA + NYC FCHHA.
    pub inquiry_before_offer: bool,
    pub conviction_type: ConvictionType,
    /// Years since release from incarceration (or since sentencing if no
    /// incarceration). Used by NYC FCHHA lookback windows.
    pub years_since_release: u32,
    /// Whether the landlord performed the individualized assessment
    /// required after a conditional offer is withdrawn (NJ FCHA + CA FEHA).
    pub individualized_assessment_performed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ViolationType {
    None,
    PreConditionalOfferInquiry,
    BarredConvictionCategory,
    LookbackWindowExpired,
    NoIndividualizedAssessment,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FairChanceResult {
    pub regime: Regime,
    pub inquiry_permitted: bool,
    pub consideration_permitted: bool,
    pub violation: ViolationType,
    pub citation: &'static str,
    pub note: String,
}

pub fn check(input: &FairChanceInput) -> FairChanceResult {
    match input.regime {
        Regime::NewJerseyFcha => nj_check(input),
        Regime::NewYorkCityFchha => nyc_check(input),
        Regime::CaliforniaFeha => ca_check(input),
        Regime::Default => default_check(input),
    }
}

fn pre_offer_violation_check(
    input: &FairChanceInput,
    regime: Regime,
    citation: &'static str,
) -> Option<FairChanceResult> {
    if input.inquiry_before_offer && !input.conditional_offer_made {
        Some(FairChanceResult {
            regime,
            inquiry_permitted: false,
            consideration_permitted: false,
            violation: ViolationType::PreConditionalOfferInquiry,
            citation,
            note:
                "Pre-conditional-offer inquiry is prohibited — landlord may not ask about criminal history on the application or verbally before extending a conditional offer of rental."
                    .to_string(),
        })
    } else {
        None
    }
}

fn nj_check(input: &FairChanceInput) -> FairChanceResult {
    if let Some(v) =
        pre_offer_violation_check(input, Regime::NewJerseyFcha, "N.J.S.A. 46:8-52 et seq. (FCHA)")
    {
        return v;
    }
    let barred = matches!(
        input.conviction_type,
        ConvictionType::Arrest
            | ConvictionType::PendingCase
            | ConvictionType::SealedExpunged
            | ConvictionType::JuvenileAdjudication
    );
    if barred {
        return FairChanceResult {
            regime: Regime::NewJerseyFcha,
            inquiry_permitted: input.conditional_offer_made,
            consideration_permitted: false,
            violation: ViolationType::BarredConvictionCategory,
            citation: "N.J.S.A. 46:8-52 (FCHA) — arrests, pending cases, sealed/expunged convictions, and juvenile adjudications are barred from consideration",
            note: format!(
                "Conviction type {:?} is a barred category under the NJ Fair Chance in Housing Act.",
                input.conviction_type
            ),
        };
    }
    if input.conditional_offer_made && !input.individualized_assessment_performed {
        return FairChanceResult {
            regime: Regime::NewJerseyFcha,
            inquiry_permitted: true,
            consideration_permitted: false,
            violation: ViolationType::NoIndividualizedAssessment,
            citation: "N.J.S.A. 46:8-52 (FCHA) — withdrawal of conditional offer requires individualized assessment + written notice + 30-day appeal window",
            note:
                "NJ FCHA requires landlord to perform an individualized assessment before withdrawing a conditional offer based on criminal history."
                    .to_string(),
        };
    }
    FairChanceResult {
        regime: Regime::NewJerseyFcha,
        inquiry_permitted: input.conditional_offer_made,
        consideration_permitted: input.conditional_offer_made,
        violation: ViolationType::None,
        citation: "N.J.S.A. 46:8-52 et seq. (FCHA)",
        note:
            "NJ FCHA compliance OK: post-conditional-offer consideration of allowed conviction types with individualized assessment performed."
                .to_string(),
    }
}

fn nyc_check(input: &FairChanceInput) -> FairChanceResult {
    if let Some(v) = pre_offer_violation_check(
        input,
        Regime::NewYorkCityFchha,
        "NYC Admin Code § 8-107.1 (Local Law 24 of 2024)",
    ) {
        return v;
    }
    let barred = matches!(
        input.conviction_type,
        ConvictionType::Arrest
            | ConvictionType::PendingCase
            | ConvictionType::SealedExpunged
            | ConvictionType::JuvenileAdjudication
            | ConvictionType::AcdDisposition
            | ConvictionType::NonCriminalViolation
    );
    if barred {
        return FairChanceResult {
            regime: Regime::NewYorkCityFchha,
            inquiry_permitted: input.conditional_offer_made,
            consideration_permitted: false,
            violation: ViolationType::BarredConvictionCategory,
            citation: "NYC Admin Code § 8-107.1 — arrests, pending cases, ACDs, juvenile adjudications, sealed/expunged convictions, and non-criminal-violation convictions are barred from consideration",
            note: format!(
                "Conviction type {:?} is a barred category under NYC Fair Chance for Housing Act.",
                input.conviction_type
            ),
        };
    }
    // Sex-offender registry — always considerable
    if input.conviction_type == ConvictionType::SexOffenderRegistry {
        return FairChanceResult {
            regime: Regime::NewYorkCityFchha,
            inquiry_permitted: input.conditional_offer_made,
            consideration_permitted: input.conditional_offer_made,
            violation: ViolationType::None,
            citation: "NYC Admin Code § 8-107.1 — convictions requiring sex-offender-registry registration are always considerable regardless of age",
            note:
                "NYC FCHHA compliance OK: sex-offender-registry conviction is always considerable."
                    .to_string(),
        };
    }
    // Lookback: felony 5 years, misdemeanor 3 years
    let lookback_years: u32 = match input.conviction_type {
        ConvictionType::Felony => 5,
        ConvictionType::Misdemeanor => 3,
        _ => 0,
    };
    if input.years_since_release > lookback_years {
        return FairChanceResult {
            regime: Regime::NewYorkCityFchha,
            inquiry_permitted: input.conditional_offer_made,
            consideration_permitted: false,
            violation: ViolationType::LookbackWindowExpired,
            citation: "NYC Admin Code § 8-107.1 — felony 5-year / misdemeanor 3-year lookback window from release or sentencing date",
            note: format!(
                "Conviction type {:?} occurred {} years ago — outside the {}-year NYC FCHHA lookback window.",
                input.conviction_type, input.years_since_release, lookback_years
            ),
        };
    }
    FairChanceResult {
        regime: Regime::NewYorkCityFchha,
        inquiry_permitted: input.conditional_offer_made,
        consideration_permitted: input.conditional_offer_made,
        violation: ViolationType::None,
        citation: "NYC Admin Code § 8-107.1 (Local Law 24 of 2024)",
        note: format!(
            "NYC FCHHA compliance OK: {:?} within {}-year lookback window.",
            input.conviction_type, lookback_years
        ),
    }
}

fn ca_check(input: &FairChanceInput) -> FairChanceResult {
    let barred = matches!(
        input.conviction_type,
        ConvictionType::Arrest | ConvictionType::SealedExpunged
    );
    if barred {
        return FairChanceResult {
            regime: Regime::CaliforniaFeha,
            inquiry_permitted: true,
            consideration_permitted: false,
            violation: ViolationType::BarredConvictionCategory,
            citation: "Cal. Civ. Code § 1786.18 — arrests and sealed/expunged convictions barred from consideration",
            note: format!(
                "Conviction type {:?} is a barred category under California fair-housing law.",
                input.conviction_type
            ),
        };
    }
    if !input.individualized_assessment_performed {
        return FairChanceResult {
            regime: Regime::CaliforniaFeha,
            inquiry_permitted: true,
            consideration_permitted: false,
            violation: ViolationType::NoIndividualizedAssessment,
            citation: "2 Cal. Code Regs. § 12266 — landlord must perform individualized assessment (nature/severity of offense, time elapsed, rehabilitation evidence) before denial",
            note:
                "California FEHA prohibits blanket bans and requires individualized assessment considering nature/severity of offense, time elapsed, and evidence of rehabilitation."
                    .to_string(),
        };
    }
    FairChanceResult {
        regime: Regime::CaliforniaFeha,
        inquiry_permitted: true,
        consideration_permitted: true,
        violation: ViolationType::None,
        citation: "Cal. Civ. Code § 1786.18 + 2 Cal. Code Regs. § 12266 (FEHA)",
        note:
            "California FEHA compliance OK: individualized assessment performed; no blanket ban applied."
                .to_string(),
    }
}

fn default_check(input: &FairChanceInput) -> FairChanceResult {
    if input.conviction_type == ConvictionType::Arrest && input.years_since_release > 7 {
        return FairChanceResult {
            regime: Regime::Default,
            inquiry_permitted: true,
            consideration_permitted: false,
            violation: ViolationType::LookbackWindowExpired,
            citation: "15 U.S.C. § 1681c (FCRA) — 7-year reporting limit on arrests by consumer reporting agencies",
            note: format!(
                "FCRA prohibits consumer reporting agencies from furnishing arrest records older than 7 years — this arrest is {} years old.",
                input.years_since_release
            ),
        };
    }
    FairChanceResult {
        regime: Regime::Default,
        inquiry_permitted: true,
        consideration_permitted: true,
        violation: ViolationType::None,
        citation: "15 U.S.C. § 1681c (FCRA) — no state fair-chance housing statute identified",
        note:
            "No state or local fair-chance-housing statute applies. Federal FCRA 7-year ceiling on arrest reports applies; state/local fair-housing laws may impose additional restrictions."
                .to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        regime: Regime,
        offer: bool,
        pre_inquiry: bool,
        conv: ConvictionType,
        years: u32,
        ia: bool,
    ) -> FairChanceInput {
        FairChanceInput {
            regime,
            conditional_offer_made: offer,
            inquiry_before_offer: pre_inquiry,
            conviction_type: conv,
            years_since_release: years,
            individualized_assessment_performed: ia,
        }
    }

    #[test]
    fn nj_pre_offer_inquiry_violation() {
        let r = check(&input(
            Regime::NewJerseyFcha,
            false,
            true,
            ConvictionType::Felony,
            2,
            false,
        ));
        assert_eq!(r.violation, ViolationType::PreConditionalOfferInquiry);
        assert!(!r.inquiry_permitted);
        assert!(!r.consideration_permitted);
        assert!(r.citation.contains("46:8-52"));
    }

    #[test]
    fn nj_post_offer_inquiry_ok() {
        let r = check(&input(
            Regime::NewJerseyFcha,
            true,
            false,
            ConvictionType::Felony,
            2,
            true,
        ));
        assert_eq!(r.violation, ViolationType::None);
        assert!(r.inquiry_permitted);
        assert!(r.consideration_permitted);
    }

    #[test]
    fn nj_arrest_barred_category() {
        let r = check(&input(
            Regime::NewJerseyFcha,
            true,
            false,
            ConvictionType::Arrest,
            0,
            true,
        ));
        assert_eq!(r.violation, ViolationType::BarredConvictionCategory);
        assert!(!r.consideration_permitted);
    }

    #[test]
    fn nj_juvenile_adjudication_barred() {
        let r = check(&input(
            Regime::NewJerseyFcha,
            true,
            false,
            ConvictionType::JuvenileAdjudication,
            0,
            true,
        ));
        assert_eq!(r.violation, ViolationType::BarredConvictionCategory);
    }

    #[test]
    fn nj_no_individualized_assessment() {
        let r = check(&input(
            Regime::NewJerseyFcha,
            true,
            false,
            ConvictionType::Felony,
            2,
            false,
        ));
        assert_eq!(r.violation, ViolationType::NoIndividualizedAssessment);
        assert!(!r.consideration_permitted);
        assert!(r.citation.contains("30-day appeal"));
    }

    #[test]
    fn nyc_pre_offer_inquiry_violation() {
        let r = check(&input(
            Regime::NewYorkCityFchha,
            false,
            true,
            ConvictionType::Felony,
            2,
            false,
        ));
        assert_eq!(r.violation, ViolationType::PreConditionalOfferInquiry);
        assert!(r.citation.contains("Local Law 24"));
    }

    #[test]
    fn nyc_felony_within_5_year_lookback_ok() {
        let r = check(&input(
            Regime::NewYorkCityFchha,
            true,
            false,
            ConvictionType::Felony,
            5,
            false,
        ));
        assert_eq!(r.violation, ViolationType::None);
        assert!(r.consideration_permitted);
    }

    #[test]
    fn nyc_felony_at_5_year_boundary_ok() {
        let r = check(&input(
            Regime::NewYorkCityFchha,
            true,
            false,
            ConvictionType::Felony,
            5,
            false,
        ));
        assert_eq!(r.violation, ViolationType::None);
    }

    #[test]
    fn nyc_felony_outside_5_year_lookback_violation() {
        let r = check(&input(
            Regime::NewYorkCityFchha,
            true,
            false,
            ConvictionType::Felony,
            6,
            false,
        ));
        assert_eq!(r.violation, ViolationType::LookbackWindowExpired);
        assert!(!r.consideration_permitted);
        assert!(r.citation.contains("5-year"));
    }

    #[test]
    fn nyc_misdemeanor_within_3_year_lookback_ok() {
        let r = check(&input(
            Regime::NewYorkCityFchha,
            true,
            false,
            ConvictionType::Misdemeanor,
            3,
            false,
        ));
        assert_eq!(r.violation, ViolationType::None);
    }

    #[test]
    fn nyc_misdemeanor_at_3_year_boundary_ok() {
        let r = check(&input(
            Regime::NewYorkCityFchha,
            true,
            false,
            ConvictionType::Misdemeanor,
            3,
            false,
        ));
        assert_eq!(r.violation, ViolationType::None);
    }

    #[test]
    fn nyc_misdemeanor_outside_3_year_lookback_violation() {
        let r = check(&input(
            Regime::NewYorkCityFchha,
            true,
            false,
            ConvictionType::Misdemeanor,
            4,
            false,
        ));
        assert_eq!(r.violation, ViolationType::LookbackWindowExpired);
    }

    #[test]
    fn nyc_sex_offender_registry_always_considerable() {
        let r = check(&input(
            Regime::NewYorkCityFchha,
            true,
            false,
            ConvictionType::SexOffenderRegistry,
            30,
            false,
        ));
        assert_eq!(r.violation, ViolationType::None);
        assert!(r.consideration_permitted);
        assert!(r.citation.contains("sex-offender-registry"));
    }

    #[test]
    fn nyc_non_criminal_violation_barred() {
        let r = check(&input(
            Regime::NewYorkCityFchha,
            true,
            false,
            ConvictionType::NonCriminalViolation,
            0,
            false,
        ));
        assert_eq!(r.violation, ViolationType::BarredConvictionCategory);
        assert!(r.citation.contains("non-criminal"));
    }

    #[test]
    fn nyc_acd_disposition_barred() {
        let r = check(&input(
            Regime::NewYorkCityFchha,
            true,
            false,
            ConvictionType::AcdDisposition,
            0,
            false,
        ));
        assert_eq!(r.violation, ViolationType::BarredConvictionCategory);
        assert!(r.citation.contains("ACDs"));
    }

    #[test]
    fn ca_arrest_barred() {
        let r = check(&input(
            Regime::CaliforniaFeha,
            true,
            false,
            ConvictionType::Arrest,
            0,
            true,
        ));
        assert_eq!(r.violation, ViolationType::BarredConvictionCategory);
    }

    #[test]
    fn ca_no_individualized_assessment_violation() {
        let r = check(&input(
            Regime::CaliforniaFeha,
            true,
            false,
            ConvictionType::Felony,
            2,
            false,
        ));
        assert_eq!(r.violation, ViolationType::NoIndividualizedAssessment);
        assert!(r.citation.contains("12266"));
        assert!(r.note.contains("rehabilitation"));
    }

    #[test]
    fn ca_with_individualized_assessment_ok() {
        let r = check(&input(
            Regime::CaliforniaFeha,
            true,
            false,
            ConvictionType::Felony,
            2,
            true,
        ));
        assert_eq!(r.violation, ViolationType::None);
        assert!(r.consideration_permitted);
    }

    #[test]
    fn default_fcra_7_year_arrest_window() {
        let r = check(&input(
            Regime::Default,
            true,
            false,
            ConvictionType::Arrest,
            8,
            true,
        ));
        assert_eq!(r.violation, ViolationType::LookbackWindowExpired);
        assert!(r.citation.contains("1681c"));
        assert!(r.citation.contains("FCRA"));
    }

    #[test]
    fn default_no_violation_recent_conviction() {
        let r = check(&input(
            Regime::Default,
            true,
            false,
            ConvictionType::Felony,
            3,
            true,
        ));
        assert_eq!(r.violation, ViolationType::None);
        assert!(r.consideration_permitted);
    }

    #[test]
    fn jurisdiction_routing() {
        assert_eq!(
            Regime::for_jurisdiction("NJ", "Newark"),
            Regime::NewJerseyFcha
        );
        assert_eq!(
            Regime::for_jurisdiction("NY", "New York"),
            Regime::NewYorkCityFchha
        );
        assert_eq!(
            Regime::for_jurisdiction("NY", "NYC"),
            Regime::NewYorkCityFchha
        );
        assert_eq!(
            Regime::for_jurisdiction("NY", "Buffalo"),
            Regime::Default,
            "NY-state outside NYC has no statewide FCHHA"
        );
        assert_eq!(
            Regime::for_jurisdiction("CA", "Los Angeles"),
            Regime::CaliforniaFeha
        );
        assert_eq!(
            Regime::for_jurisdiction("TX", "Austin"),
            Regime::Default
        );
    }

    #[test]
    fn jurisdiction_case_insensitive() {
        assert_eq!(
            Regime::for_jurisdiction("nj", "newark"),
            Regime::NewJerseyFcha
        );
        assert_eq!(
            Regime::for_jurisdiction("ny", "NEW YORK"),
            Regime::NewYorkCityFchha
        );
    }

    #[test]
    fn nj_only_single_state_uniqueness() {
        let r_nj = check(&input(
            Regime::NewJerseyFcha,
            true,
            false,
            ConvictionType::Felony,
            2,
            true,
        ));
        let r_other = check(&input(
            Regime::Default,
            true,
            false,
            ConvictionType::Felony,
            2,
            true,
        ));
        assert_ne!(r_nj.citation, r_other.citation);
    }

    #[test]
    fn nyc_lookback_off_by_one_boundary_exhaustive() {
        // Felony: 5 = ok, 6 = expired (strict greater-than)
        let ok = check(&input(
            Regime::NewYorkCityFchha,
            true,
            false,
            ConvictionType::Felony,
            5,
            false,
        ));
        assert_eq!(ok.violation, ViolationType::None);
        let expired = check(&input(
            Regime::NewYorkCityFchha,
            true,
            false,
            ConvictionType::Felony,
            6,
            false,
        ));
        assert_eq!(expired.violation, ViolationType::LookbackWindowExpired);
        // Misdemeanor: 3 = ok, 4 = expired
        let ok_m = check(&input(
            Regime::NewYorkCityFchha,
            true,
            false,
            ConvictionType::Misdemeanor,
            3,
            false,
        ));
        assert_eq!(ok_m.violation, ViolationType::None);
        let expired_m = check(&input(
            Regime::NewYorkCityFchha,
            true,
            false,
            ConvictionType::Misdemeanor,
            4,
            false,
        ));
        assert_eq!(expired_m.violation, ViolationType::LookbackWindowExpired);
    }

    #[test]
    fn citations_pin_correct_authorities() {
        let r_nj = check(&input(
            Regime::NewJerseyFcha,
            true,
            false,
            ConvictionType::Felony,
            2,
            true,
        ));
        assert!(r_nj.citation.contains("FCHA"));
        assert!(r_nj.citation.contains("46:8-52"));

        let r_nyc = check(&input(
            Regime::NewYorkCityFchha,
            true,
            false,
            ConvictionType::Felony,
            2,
            false,
        ));
        assert!(r_nyc.citation.contains("8-107.1"));
        assert!(r_nyc.citation.contains("Local Law 24"));

        let r_ca = check(&input(
            Regime::CaliforniaFeha,
            true,
            false,
            ConvictionType::Felony,
            2,
            true,
        ));
        assert!(r_ca.citation.contains("1786.18"));
        assert!(r_ca.citation.contains("12266"));

        let r_def = check(&input(
            Regime::Default,
            true,
            false,
            ConvictionType::Felony,
            2,
            true,
        ));
        assert!(r_def.citation.contains("FCRA") || r_def.citation.contains("1681c"));
    }
}
