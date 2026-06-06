//! Tenant Right to Counsel (RTC) in Eviction Proceedings Compliance.
//!
//! Pure-compute multi-jurisdictional check for whether a tenant facing
//! eviction is entitled to court-appointed counsel and whether the
//! court / landlord respected that right. Trader-landlord critical
//! because Right-to-Counsel programs dramatically increase tenant
//! defense rates, eviction case duration, and landlord litigation
//! cost; failure to honor RTC procedure can invalidate the eviction
//! judgment.
//!
//! Web research (verified 2026-06-03):
//! - **NYC (first U.S. jurisdiction, August 2017)** — Intro 214-b
//!   enacted as Local Law 136 of 2017, codified at NYC Admin Code
//!   § 26-1301 et seq. Tenants with household income ≤ 200 % of
//!   federal poverty level facing eviction in housing court receive
//!   right to court-appointed counsel via DSS-OCJ contracts with
//!   legal services providers. NYC Bar Report "Statewide Right to
//!   Counsel"; CSS NY "First Year Results".
//! - **Newark 2018** — Newark Right to Counsel Ordinance; 200 % FPL
//!   threshold; covers appeals. NLIHC 14-1 report.
//! - **San Francisco 2018 — Prop F** — full universal right to
//!   counsel; NO income test. Civilrighttocounsel.org.
//! - **Cleveland 2019** — eligibility ≤ 100 % FPL household with
//!   minor child / elderly / disabled.
//! - **Philadelphia 2019** — Bill 190386; phased rollout starting
//!   with five zip codes.
//! - **Boulder 2020** — Ordinance 8377; 200 % FPL.
//! - **Kansas City MO 2022** — Ordinance 220448; 200 % FPL.
//! - **Washington statewide 2021** — RCW 59.18.640 (added by
//!   Senate Bill 5160, Apr 22 2021) — FIRST U.S. STATEWIDE RTC.
//!   "Superior courts MUST appoint counsel for an indigent tenant
//!   in an unlawful detainer proceeding." Indigent defined as
//!   anyone receiving certain types of public assistance or annual
//!   post-tax income at or below 200 percent of federal poverty
//!   level (\$25,760 for an individual at time of enactment).
//!   Washington State Office of Civil Legal Aid administers the
//!   program. Civilrighttocounsel.org Major Developments.
//! - **Maryland statewide 2021** — Access to Counsel in Evictions
//!   Act (HB 18 / SB 154); 50 % AMI threshold.
//! - **Connecticut statewide 2021** — Public Act 21-34; 80 % AMI
//!   threshold; phased rollout.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const RTC_NYC_LOCAL_LAW_136_YEAR: u32 = 2017;
pub const RTC_NYC_INCOME_THRESHOLD_FPL_BASIS_POINTS: u32 = 20_000;
pub const RTC_NEWARK_YEAR: u32 = 2018;
pub const RTC_SAN_FRANCISCO_PROP_F_YEAR: u32 = 2018;
pub const RTC_CLEVELAND_YEAR: u32 = 2019;
pub const RTC_PHILADELPHIA_YEAR: u32 = 2019;
pub const RTC_BOULDER_YEAR: u32 = 2020;
pub const RTC_KANSAS_CITY_YEAR: u32 = 2022;
pub const RTC_WASHINGTON_STATEWIDE_YEAR: u32 = 2021;
pub const RTC_WASHINGTON_SENATE_BILL_NUMBER: u32 = 5160;
pub const RTC_MARYLAND_STATEWIDE_YEAR: u32 = 2021;
pub const RTC_CONNECTICUT_STATEWIDE_YEAR: u32 = 2021;
pub const RTC_DEFAULT_FPL_THRESHOLD_BASIS_POINTS: u32 = 20_000;
pub const RTC_CLEVELAND_FPL_THRESHOLD_BASIS_POINTS: u32 = 10_000;
pub const RTC_MARYLAND_AMI_THRESHOLD_BASIS_POINTS: u32 = 5_000;
pub const RTC_CONNECTICUT_AMI_THRESHOLD_BASIS_POINTS: u32 = 8_000;
pub const RTC_BASIS_POINT_DENOMINATOR: u32 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RtcJurisdiction {
    NewYorkCity,
    Newark,
    SanFranciscoPropF,
    Cleveland,
    Philadelphia,
    Boulder,
    KansasCityMo,
    WashingtonStatewide,
    MarylandStatewide,
    ConnecticutStatewide,
    OtherJurisdictionWithoutRtc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TenantIncomeBand {
    AtOrBelowJurisdictionalThreshold,
    BetweenThresholdAndModerateMeansCeiling,
    AboveAllRtcThresholds,
    ReceivingPublicAssistanceCategoricallyIndigent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RepresentationStatus {
    NoAttorneyTenantSelfRepresented,
    CourtAppointedRtcAttorneyEnrolledLegalServicesProvider,
    PrivateAttorneyRetained,
    TenantRequestedAttorneyButCourtFailedToAppoint,
    LandlordObstructedTenantAccessToCounsel,
    CourtDeniedContinuanceForAttorneyAppointment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EvictionProceedingType {
    Nonpayment,
    HoldoverNoCause,
    HoldoverForCause,
    AdministrativeAgencyProceeding,
    AppellateProceeding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TenantRightToCounselMode {
    NotApplicableJurisdictionLacksRtc,
    CompliantAttorneyAppointedUnderRtc,
    CompliantSanFranciscoUniversalRtcNoIncomeTest,
    CompliantTenantAboveThresholdSelfRepresentedOrPrivateCounsel,
    CompliantTenantWaivedRepresentationAfterAdvisement,
    ViolationCourtFailedToAppointEligibleTenant,
    ViolationLandlordObstructedTenantAccessToCounsel,
    ViolationCourtDeniedContinuanceForAttorneyAppointment,
    ViolationCourtFailedToNotifyTenantOfRtcRight,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub jurisdiction: RtcJurisdiction,
    pub income_band: TenantIncomeBand,
    pub representation_status: RepresentationStatus,
    pub proceeding_type: EvictionProceedingType,
    pub court_provided_rtc_notice_to_tenant: bool,
    pub tenant_waived_after_advisement: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: TenantRightToCounselMode,
    pub jurisdictional_threshold_basis_points: u32,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type RentalTenantRightToCounselEvictionInput = Input;
pub type RentalTenantRightToCounselEvictionOutput = Output;
pub type RentalTenantRightToCounselEvictionResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

fn threshold_for(j: RtcJurisdiction) -> u32 {
    match j {
        RtcJurisdiction::Cleveland => RTC_CLEVELAND_FPL_THRESHOLD_BASIS_POINTS,
        RtcJurisdiction::MarylandStatewide => RTC_MARYLAND_AMI_THRESHOLD_BASIS_POINTS,
        RtcJurisdiction::ConnecticutStatewide => RTC_CONNECTICUT_AMI_THRESHOLD_BASIS_POINTS,
        RtcJurisdiction::SanFranciscoPropF => 0,
        RtcJurisdiction::OtherJurisdictionWithoutRtc => 0,
        _ => RTC_DEFAULT_FPL_THRESHOLD_BASIS_POINTS,
    }
}

fn statutory_basis_for(j: RtcJurisdiction) -> &'static str {
    match j {
        RtcJurisdiction::NewYorkCity => "NYC Admin Code § 26-1301 et seq. (Local Law 136 of 2017)",
        RtcJurisdiction::Newark => "Newark Right to Counsel Ordinance 2018",
        RtcJurisdiction::SanFranciscoPropF => {
            "SF Proposition F (2018) — universal RTC, no income test"
        }
        RtcJurisdiction::Cleveland => "Cleveland Right to Counsel Ordinance 2019",
        RtcJurisdiction::Philadelphia => "Philadelphia Bill 190386 (2019)",
        RtcJurisdiction::Boulder => "Boulder Ordinance 8377 (2020)",
        RtcJurisdiction::KansasCityMo => "Kansas City MO Ordinance 220448 (2022)",
        RtcJurisdiction::WashingtonStatewide => {
            "RCW 59.18.640 (added by 2021 Senate Bill 5160) — FIRST U.S. statewide RTC"
        }
        RtcJurisdiction::MarylandStatewide => {
            "MD Access to Counsel in Evictions Act 2021 (HB 18 / SB 154)"
        }
        RtcJurisdiction::ConnecticutStatewide => "CT Public Act 21-34 (2021)",
        RtcJurisdiction::OtherJurisdictionWithoutRtc => "None — jurisdiction lacks RTC program",
    }
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "NYC Admin Code § 26-1301 et seq. (Local Law 136 of 2017) — FIRST U.S. RTC jurisdiction; ≤ 200 % FPL".to_string(),
        "Newark Right to Counsel Ordinance 2018 — ≤ 200 % FPL".to_string(),
        "San Francisco Proposition F (2018) — universal RTC; no income test".to_string(),
        "Cleveland Right to Counsel Ordinance 2019 — ≤ 100 % FPL with minor child / elderly / disabled".to_string(),
        "Philadelphia Bill 190386 (2019) — phased rollout".to_string(),
        "Boulder Ordinance 8377 (2020) — ≤ 200 % FPL".to_string(),
        "Kansas City MO Ordinance 220448 (2022) — ≤ 200 % FPL".to_string(),
        "RCW 59.18.640 — Washington 2021 Senate Bill 5160 — FIRST U.S. STATEWIDE RTC; ≤ 200 % FPL or public assistance".to_string(),
        "Maryland Access to Counsel in Evictions Act 2021 (HB 18 / SB 154) — ≤ 50 % AMI".to_string(),
        "Connecticut Public Act 21-34 (2021) — ≤ 80 % AMI; phased rollout".to_string(),
        "Washington State Office of Civil Legal Aid (OCLA) — administers RCW 59.18.640 RTC program".to_string(),
    ];

    if input.jurisdiction == RtcJurisdiction::OtherJurisdictionWithoutRtc {
        return Output {
            mode: TenantRightToCounselMode::NotApplicableJurisdictionLacksRtc,
            jurisdictional_threshold_basis_points: 0,
            statutory_basis: statutory_basis_for(input.jurisdiction).to_string(),
            notes: format!(
                "Jurisdiction has no Right to Counsel program. Tenant representation status = {:?}; no RTC obligation arises.",
                input.representation_status
            ),
            citations,
        };
    }

    let threshold_bp = threshold_for(input.jurisdiction);

    if input.jurisdiction == RtcJurisdiction::SanFranciscoPropF {
        let mode = match input.representation_status {
            RepresentationStatus::CourtAppointedRtcAttorneyEnrolledLegalServicesProvider
            | RepresentationStatus::PrivateAttorneyRetained => {
                TenantRightToCounselMode::CompliantSanFranciscoUniversalRtcNoIncomeTest
            }
            RepresentationStatus::TenantRequestedAttorneyButCourtFailedToAppoint => {
                TenantRightToCounselMode::ViolationCourtFailedToAppointEligibleTenant
            }
            RepresentationStatus::LandlordObstructedTenantAccessToCounsel => {
                TenantRightToCounselMode::ViolationLandlordObstructedTenantAccessToCounsel
            }
            RepresentationStatus::CourtDeniedContinuanceForAttorneyAppointment => {
                TenantRightToCounselMode::ViolationCourtDeniedContinuanceForAttorneyAppointment
            }
            RepresentationStatus::NoAttorneyTenantSelfRepresented => {
                if input.tenant_waived_after_advisement {
                    TenantRightToCounselMode::CompliantTenantWaivedRepresentationAfterAdvisement
                } else if !input.court_provided_rtc_notice_to_tenant {
                    TenantRightToCounselMode::ViolationCourtFailedToNotifyTenantOfRtcRight
                } else {
                    TenantRightToCounselMode::ViolationCourtFailedToAppointEligibleTenant
                }
            }
        };
        return Output {
            mode,
            jurisdictional_threshold_basis_points: 0,
            statutory_basis: statutory_basis_for(input.jurisdiction).to_string(),
            notes: format!(
                "San Francisco Proposition F is UNIVERSAL RTC: no income test. Tenant entitled to counsel regardless of income band {:?}. Representation status = {:?}.",
                input.income_band, input.representation_status
            ),
            citations,
        };
    }

    let tenant_eligible_by_income = matches!(
        input.income_band,
        TenantIncomeBand::AtOrBelowJurisdictionalThreshold
            | TenantIncomeBand::ReceivingPublicAssistanceCategoricallyIndigent
    );

    if !tenant_eligible_by_income {
        let mode = match input.representation_status {
            RepresentationStatus::PrivateAttorneyRetained => {
                TenantRightToCounselMode::CompliantTenantAboveThresholdSelfRepresentedOrPrivateCounsel
            }
            RepresentationStatus::NoAttorneyTenantSelfRepresented => {
                TenantRightToCounselMode::CompliantTenantAboveThresholdSelfRepresentedOrPrivateCounsel
            }
            _ => TenantRightToCounselMode::CompliantTenantAboveThresholdSelfRepresentedOrPrivateCounsel,
        };
        return Output {
            mode,
            jurisdictional_threshold_basis_points: threshold_bp,
            statutory_basis: statutory_basis_for(input.jurisdiction).to_string(),
            notes: format!(
                "Tenant income band {:?} exceeds jurisdictional RTC threshold of {} basis points; not entitled to court-appointed counsel. Representation status = {:?}.",
                input.income_band, threshold_bp, input.representation_status
            ),
            citations,
        };
    }

    if !input.court_provided_rtc_notice_to_tenant {
        return Output {
            mode: TenantRightToCounselMode::ViolationCourtFailedToNotifyTenantOfRtcRight,
            jurisdictional_threshold_basis_points: threshold_bp,
            statutory_basis: statutory_basis_for(input.jurisdiction).to_string(),
            notes: format!(
                "VIOLATION: court failed to notify income-eligible tenant of RTC right. Tenant income band = {:?} qualifies under {} basis points threshold; statutory notice prerequisite missed.",
                input.income_band, threshold_bp
            ),
            citations,
        };
    }

    let mode = match input.representation_status {
        RepresentationStatus::CourtAppointedRtcAttorneyEnrolledLegalServicesProvider => {
            TenantRightToCounselMode::CompliantAttorneyAppointedUnderRtc
        }
        RepresentationStatus::PrivateAttorneyRetained => {
            TenantRightToCounselMode::CompliantAttorneyAppointedUnderRtc
        }
        RepresentationStatus::NoAttorneyTenantSelfRepresented => {
            if input.tenant_waived_after_advisement {
                TenantRightToCounselMode::CompliantTenantWaivedRepresentationAfterAdvisement
            } else {
                TenantRightToCounselMode::ViolationCourtFailedToAppointEligibleTenant
            }
        }
        RepresentationStatus::TenantRequestedAttorneyButCourtFailedToAppoint => {
            TenantRightToCounselMode::ViolationCourtFailedToAppointEligibleTenant
        }
        RepresentationStatus::LandlordObstructedTenantAccessToCounsel => {
            TenantRightToCounselMode::ViolationLandlordObstructedTenantAccessToCounsel
        }
        RepresentationStatus::CourtDeniedContinuanceForAttorneyAppointment => {
            TenantRightToCounselMode::ViolationCourtDeniedContinuanceForAttorneyAppointment
        }
    };

    Output {
        mode,
        jurisdictional_threshold_basis_points: threshold_bp,
        statutory_basis: statutory_basis_for(input.jurisdiction).to_string(),
        notes: format!(
            "Income-eligible tenant under {} basis points threshold; proceeding type = {:?}; representation status = {:?}; notice prerequisite satisfied.",
            threshold_bp, input.proceeding_type, input.representation_status
        ),
        citations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_nyc_eligible() -> Input {
        Input {
            jurisdiction: RtcJurisdiction::NewYorkCity,
            income_band: TenantIncomeBand::AtOrBelowJurisdictionalThreshold,
            representation_status:
                RepresentationStatus::CourtAppointedRtcAttorneyEnrolledLegalServicesProvider,
            proceeding_type: EvictionProceedingType::Nonpayment,
            court_provided_rtc_notice_to_tenant: true,
            tenant_waived_after_advisement: false,
        }
    }

    #[test]
    fn nyc_eligible_tenant_with_appointed_counsel_compliant() {
        let result = check(&baseline_nyc_eligible());
        assert_eq!(
            result.mode,
            TenantRightToCounselMode::CompliantAttorneyAppointedUnderRtc
        );
        assert_eq!(result.jurisdictional_threshold_basis_points, 20_000);
    }

    #[test]
    fn nyc_eligible_tenant_self_represented_court_failed_violation() {
        let input = Input {
            representation_status: RepresentationStatus::NoAttorneyTenantSelfRepresented,
            ..baseline_nyc_eligible()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TenantRightToCounselMode::ViolationCourtFailedToAppointEligibleTenant
        );
    }

    #[test]
    fn nyc_eligible_tenant_waived_after_advisement_compliant() {
        let input = Input {
            representation_status: RepresentationStatus::NoAttorneyTenantSelfRepresented,
            tenant_waived_after_advisement: true,
            ..baseline_nyc_eligible()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TenantRightToCounselMode::CompliantTenantWaivedRepresentationAfterAdvisement
        );
    }

    #[test]
    fn nyc_above_threshold_tenant_not_entitled() {
        let input = Input {
            income_band: TenantIncomeBand::AboveAllRtcThresholds,
            representation_status: RepresentationStatus::NoAttorneyTenantSelfRepresented,
            ..baseline_nyc_eligible()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TenantRightToCounselMode::CompliantTenantAboveThresholdSelfRepresentedOrPrivateCounsel
        );
    }

    #[test]
    fn sf_prop_f_no_income_test_universal_compliant() {
        let input = Input {
            jurisdiction: RtcJurisdiction::SanFranciscoPropF,
            income_band: TenantIncomeBand::AboveAllRtcThresholds,
            representation_status:
                RepresentationStatus::CourtAppointedRtcAttorneyEnrolledLegalServicesProvider,
            proceeding_type: EvictionProceedingType::HoldoverNoCause,
            court_provided_rtc_notice_to_tenant: true,
            tenant_waived_after_advisement: false,
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TenantRightToCounselMode::CompliantSanFranciscoUniversalRtcNoIncomeTest
        );
        assert_eq!(result.jurisdictional_threshold_basis_points, 0);
    }

    #[test]
    fn sf_high_income_self_represented_not_advised_violation() {
        let input = Input {
            jurisdiction: RtcJurisdiction::SanFranciscoPropF,
            income_band: TenantIncomeBand::AboveAllRtcThresholds,
            representation_status: RepresentationStatus::NoAttorneyTenantSelfRepresented,
            proceeding_type: EvictionProceedingType::Nonpayment,
            court_provided_rtc_notice_to_tenant: false,
            tenant_waived_after_advisement: false,
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TenantRightToCounselMode::ViolationCourtFailedToNotifyTenantOfRtcRight
        );
    }

    #[test]
    fn washington_statewide_indigent_appointed_compliant() {
        let input = Input {
            jurisdiction: RtcJurisdiction::WashingtonStatewide,
            income_band: TenantIncomeBand::ReceivingPublicAssistanceCategoricallyIndigent,
            representation_status:
                RepresentationStatus::CourtAppointedRtcAttorneyEnrolledLegalServicesProvider,
            proceeding_type: EvictionProceedingType::Nonpayment,
            court_provided_rtc_notice_to_tenant: true,
            tenant_waived_after_advisement: false,
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TenantRightToCounselMode::CompliantAttorneyAppointedUnderRtc
        );
        assert!(result.statutory_basis.contains("RCW 59.18.640"));
    }

    #[test]
    fn washington_tenant_requested_court_failed_violation() {
        let input = Input {
            jurisdiction: RtcJurisdiction::WashingtonStatewide,
            income_band: TenantIncomeBand::AtOrBelowJurisdictionalThreshold,
            representation_status:
                RepresentationStatus::TenantRequestedAttorneyButCourtFailedToAppoint,
            proceeding_type: EvictionProceedingType::HoldoverNoCause,
            court_provided_rtc_notice_to_tenant: true,
            tenant_waived_after_advisement: false,
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TenantRightToCounselMode::ViolationCourtFailedToAppointEligibleTenant
        );
    }

    #[test]
    fn cleveland_lower_threshold_100_pct_fpl() {
        let input = Input {
            jurisdiction: RtcJurisdiction::Cleveland,
            income_band: TenantIncomeBand::AtOrBelowJurisdictionalThreshold,
            representation_status:
                RepresentationStatus::CourtAppointedRtcAttorneyEnrolledLegalServicesProvider,
            proceeding_type: EvictionProceedingType::Nonpayment,
            court_provided_rtc_notice_to_tenant: true,
            tenant_waived_after_advisement: false,
        };
        let result = check(&input);
        assert_eq!(result.jurisdictional_threshold_basis_points, 10_000);
    }

    #[test]
    fn maryland_50_pct_ami_threshold() {
        let input = Input {
            jurisdiction: RtcJurisdiction::MarylandStatewide,
            income_band: TenantIncomeBand::AtOrBelowJurisdictionalThreshold,
            representation_status:
                RepresentationStatus::CourtAppointedRtcAttorneyEnrolledLegalServicesProvider,
            proceeding_type: EvictionProceedingType::Nonpayment,
            court_provided_rtc_notice_to_tenant: true,
            tenant_waived_after_advisement: false,
        };
        let result = check(&input);
        assert_eq!(result.jurisdictional_threshold_basis_points, 5_000);
    }

    #[test]
    fn connecticut_80_pct_ami_threshold() {
        let input = Input {
            jurisdiction: RtcJurisdiction::ConnecticutStatewide,
            income_band: TenantIncomeBand::AtOrBelowJurisdictionalThreshold,
            representation_status:
                RepresentationStatus::CourtAppointedRtcAttorneyEnrolledLegalServicesProvider,
            proceeding_type: EvictionProceedingType::Nonpayment,
            court_provided_rtc_notice_to_tenant: true,
            tenant_waived_after_advisement: false,
        };
        let result = check(&input);
        assert_eq!(result.jurisdictional_threshold_basis_points, 8_000);
    }

    #[test]
    fn other_jurisdiction_without_rtc_not_applicable() {
        let input = Input {
            jurisdiction: RtcJurisdiction::OtherJurisdictionWithoutRtc,
            income_band: TenantIncomeBand::AtOrBelowJurisdictionalThreshold,
            representation_status: RepresentationStatus::NoAttorneyTenantSelfRepresented,
            proceeding_type: EvictionProceedingType::Nonpayment,
            court_provided_rtc_notice_to_tenant: false,
            tenant_waived_after_advisement: false,
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TenantRightToCounselMode::NotApplicableJurisdictionLacksRtc
        );
    }

    #[test]
    fn landlord_obstructed_tenant_access_violation() {
        let input = Input {
            representation_status: RepresentationStatus::LandlordObstructedTenantAccessToCounsel,
            ..baseline_nyc_eligible()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TenantRightToCounselMode::ViolationLandlordObstructedTenantAccessToCounsel
        );
    }

    #[test]
    fn court_denied_continuance_for_attorney_appointment_violation() {
        let input = Input {
            representation_status:
                RepresentationStatus::CourtDeniedContinuanceForAttorneyAppointment,
            ..baseline_nyc_eligible()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TenantRightToCounselMode::ViolationCourtDeniedContinuanceForAttorneyAppointment
        );
    }

    #[test]
    fn nyc_court_failed_to_notify_eligible_tenant_violation() {
        let input = Input {
            representation_status: RepresentationStatus::NoAttorneyTenantSelfRepresented,
            court_provided_rtc_notice_to_tenant: false,
            ..baseline_nyc_eligible()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TenantRightToCounselMode::ViolationCourtFailedToNotifyTenantOfRtcRight
        );
    }

    #[test]
    fn citations_pin_jurisdictional_statutes() {
        let result = check(&baseline_nyc_eligible());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("NYC Admin Code § 26-1301"));
        assert!(joined.contains("Local Law 136 of 2017"));
        assert!(joined.contains("Proposition F"));
        assert!(joined.contains("RCW 59.18.640"));
        assert!(joined.contains("Senate Bill 5160"));
        assert!(joined.contains("Maryland Access to Counsel"));
        assert!(joined.contains("Connecticut Public Act 21-34"));
    }

    #[test]
    fn constant_pin_jurisdictional_years_and_thresholds() {
        assert_eq!(RTC_NYC_LOCAL_LAW_136_YEAR, 2017);
        assert_eq!(RTC_NEWARK_YEAR, 2018);
        assert_eq!(RTC_SAN_FRANCISCO_PROP_F_YEAR, 2018);
        assert_eq!(RTC_CLEVELAND_YEAR, 2019);
        assert_eq!(RTC_PHILADELPHIA_YEAR, 2019);
        assert_eq!(RTC_BOULDER_YEAR, 2020);
        assert_eq!(RTC_KANSAS_CITY_YEAR, 2022);
        assert_eq!(RTC_WASHINGTON_STATEWIDE_YEAR, 2021);
        assert_eq!(RTC_WASHINGTON_SENATE_BILL_NUMBER, 5160);
        assert_eq!(RTC_MARYLAND_STATEWIDE_YEAR, 2021);
        assert_eq!(RTC_CONNECTICUT_STATEWIDE_YEAR, 2021);
        assert_eq!(RTC_DEFAULT_FPL_THRESHOLD_BASIS_POINTS, 20_000);
        assert_eq!(RTC_CLEVELAND_FPL_THRESHOLD_BASIS_POINTS, 10_000);
        assert_eq!(RTC_MARYLAND_AMI_THRESHOLD_BASIS_POINTS, 5_000);
        assert_eq!(RTC_CONNECTICUT_AMI_THRESHOLD_BASIS_POINTS, 8_000);
        assert_eq!(RTC_BASIS_POINT_DENOMINATOR, 10_000);
    }

    #[test]
    fn private_attorney_retained_compliant_even_when_eligible() {
        let input = Input {
            representation_status: RepresentationStatus::PrivateAttorneyRetained,
            ..baseline_nyc_eligible()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TenantRightToCounselMode::CompliantAttorneyAppointedUnderRtc
        );
    }
}
