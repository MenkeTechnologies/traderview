//! Multi-State Tenant Bill of Rights / Lease Disclosure Statement
//! Handout Compliance Module.
//!
//! Pure-compute check for whether a landlord has provided each
//! statutorily required tenant disclosure / handout at lease
//! signing across multiple state regimes. Trader-landlord critical
//! because most cross-state portfolios must apply at least four
//! different sets of mandatory tenant-handout rules, and defective
//! handouts can void monetary lease provisions (e.g., rent demand,
//! security deposit forfeiture) under the Truth in Renting Acts.
//!
//! Web research (verified 2026-06-03):
//! - **Michigan Truth in Renting Act** (Public Act 454 of 1978;
//!   MCL § 554.631 et seq.): all rental agreements must
//!   prominently state "NOTICE: Michigan law establishes rights
//!   and obligations for parties to rental agreements. This
//!   agreement is required to comply with the Truth in Renting
//!   Act. If you have a question about the interpretation or
//!   legality of a provision of this agreement, you may want to
//!   seek assistance from a lawyer or other qualified person."
//!   Notice must be **no smaller than 12-point type OR legible
//!   print with letters not smaller than 1/8 inch**. Required
//!   disclosures: lead-based paint (Title X 1992), security
//!   deposit location, environmental hazards, owner/manager
//!   identity, utility billing arrangements, and protection from
//!   domestic abuse. (iPropertyManagement Michigan Landlord
//!   Tenant Laws 2026; Michigan Courts Truth in Renting Act
//!   guidance; LeaseRunner Michigan.)
//! - **New Jersey Truth-in-Renting Act** (N.J. Stat. § 46:8-43 to
//!   § 46:8-50): every landlord must distribute one copy of the
//!   DCA-published Truth in Renting statement to each tenant at
//!   lease signing AND **within 30 days of the law's effective
//!   date for existing tenants**. Penalty for non-distribution =
//!   $100. (NJ DCA Truth in Renting booklet; tenant-rights.com
//!   NJ Truth-in-Renting Act.)
//! - **District of Columbia Tenant Bill of Rights**: D.C. Office
//!   of the Tenant Advocate publishes the DC Tenant Bill of
//!   Rights. D.C. Code § 42-3502.22b: landlord must provide
//!   tenant with copy of lease + addendums + copies of certain
//!   D.C. housing regulations + DC Tenant Bill of Rights document
//!   at lease signing. (DC OTA Tenant Bill of Rights publication.)
//! - **California Civ. Code § 1962**: owner identity + address +
//!   authorized property manager identity must be disclosed in
//!   writing within 15 days of tenancy or by lease attachment.
//! - **California Civ. Code § 1962.5 / § 1962.7**: where security
//!   deposit is held (institution name, address, account type).
//! - **Florida Stat. § 83.49**: deposit location + landlord/
//!   manager identity disclosed within 30 days of receipt.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const MICHIGAN_TRUTH_IN_RENTING_ACT_YEAR: u32 = 1978;
pub const MICHIGAN_TRUTH_IN_RENTING_PUBLIC_ACT_NUMBER: u32 = 454;
pub const MICHIGAN_REQUIRED_FONT_SIZE_POINTS: u32 = 12;
pub const MICHIGAN_REQUIRED_FONT_HEIGHT_INCHES_NUMERATOR: u32 = 1;
pub const MICHIGAN_REQUIRED_FONT_HEIGHT_INCHES_DENOMINATOR: u32 = 8;
pub const NEW_JERSEY_TRUTH_IN_RENTING_EXISTING_TENANT_DAYS: u32 = 30;
pub const NEW_JERSEY_TRUTH_IN_RENTING_PENALTY_DOLLARS: u64 = 100;
pub const CALIFORNIA_1962_OWNER_IDENTITY_DISCLOSURE_DAYS: u32 = 15;
pub const FLORIDA_83_49_DEPOSIT_DISCLOSURE_DAYS: u32 = 30;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HandoutJurisdiction {
    MichiganTruthInRenting,
    NewJerseyTruthInRenting,
    DistrictOfColumbiaTenantBillOfRights,
    CaliforniaSection1962,
    FloridaStatute83_49,
    OtherStateWithoutHandoutMandate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MichiganNoticeFontStatus {
    NoticeAtLeast12PointTypeOrLegiblePrintWithLettersAt1_8Inch,
    NoticeTooSmallBelow12Point,
    NoticeMissingEntirely,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DcHandoutStatus {
    DcTenantBillOfRightsProvidedWithLease,
    DcTenantBillOfRightsNotProvided,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NewJerseyDistributionStatus {
    DistributedAtLeaseSigning,
    DistributedToExistingTenantWithin30Days,
    NotDistributed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TenantBillOfRightsHandoutMode {
    NotApplicableNoLeaseExecuted,
    NotApplicableJurisdictionLacksMandate,
    CompliantAllRequiredDisclosuresIncluded,
    CompliantMichiganStatutoryNoticeAndDisclosuresIncluded,
    CompliantNewJerseyTruthInRentingDistributed,
    CompliantDcTenantBillOfRightsProvided,
    CompliantCaliforniaOwnerIdentityDisclosed,
    CompliantFloridaDepositLocationDisclosed,
    ViolationMichiganStatutoryNoticeMissing,
    ViolationMichiganStatutoryNoticeFontTooSmall,
    ViolationMichiganDomesticAbuseNoticeMissing,
    ViolationNewJerseyTruthInRentingNotDistributed,
    ViolationDcTenantBillOfRightsNotProvided,
    ViolationCaliforniaOwnerIdentityNotDisclosedWithin15Days,
    ViolationFloridaDepositLocationNotDisclosedWithin30Days,
    ViolationLeadBasedPaintTitleXFederalDisclosureMissing,
    ViolationSecurityDepositLocationOrOwnerManagerIdentityMissing,
    ViolationUtilityBillingArrangementsNotDisclosed,
    ViolationEnvironmentalHazardKnownButNotDisclosed,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub jurisdiction: HandoutJurisdiction,
    pub lease_executed: bool,
    pub property_built_before_1978_lead_paint_disclosure_required: bool,
    pub lead_based_paint_disclosure_provided: bool,
    pub security_deposit_location_disclosed: bool,
    pub owner_manager_identity_disclosed: bool,
    pub environmental_hazard_known_to_landlord: bool,
    pub environmental_hazard_disclosed: bool,
    pub utility_billing_arrangements_disclosed: bool,
    pub michigan_notice_font_status: MichiganNoticeFontStatus,
    pub michigan_domestic_abuse_notice_included: bool,
    pub new_jersey_distribution_status: NewJerseyDistributionStatus,
    pub dc_handout_status: DcHandoutStatus,
    pub california_owner_identity_days_to_disclosure: u32,
    pub florida_deposit_location_days_to_disclosure: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: TenantBillOfRightsHandoutMode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type RentalTenantBillOfRightsHandoutInput = Input;
pub type RentalTenantBillOfRightsHandoutOutput = Output;
pub type RentalTenantBillOfRightsHandoutResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "Michigan Truth in Renting Act (Public Act 454 of 1978; MCL § 554.631 et seq.) — required lease notice in min 12-point type or 1/8 inch legible print + domestic abuse protection notice".to_string(),
        "N.J. Stat. § 46:8-43 to § 46:8-50 — Truth-in-Renting Act: landlord must distribute DCA Truth in Renting booklet at lease signing AND within 30 days for existing tenants; $100 penalty for non-distribution".to_string(),
        "D.C. Code § 42-3502.22b — landlord must provide lease + addendums + DC Tenant Bill of Rights at lease signing; D.C. Office of the Tenant Advocate publishes".to_string(),
        "Cal. Civ. Code § 1962 — owner identity + address + authorized manager identity disclosed in writing within 15 days of tenancy or by lease attachment".to_string(),
        "Cal. Civ. Code § 1962.5 / § 1962.7 — security deposit location (institution + address + account type)".to_string(),
        "Fla. Stat. § 83.49 — security deposit location + landlord/manager identity disclosed within 30 days of receipt".to_string(),
        "Federal Lead-Based Paint Hazard Reduction Act (Title X of 1992) — pre-1978 housing requires EPA-approved disclosure form + EPA brochure".to_string(),
        "Required cross-state common disclosures: lead-based paint (Title X), security deposit location, owner/manager identity, environmental hazard, utility billing arrangements".to_string(),
    ];

    if !input.lease_executed {
        return Output {
            mode: TenantBillOfRightsHandoutMode::NotApplicableNoLeaseExecuted,
            statutory_basis: "No lease executed".to_string(),
            notes: "No lease has been executed; handout requirements do not yet arise.".to_string(),
            citations,
        };
    }

    if input.jurisdiction == HandoutJurisdiction::OtherStateWithoutHandoutMandate {
        return Output {
            mode: TenantBillOfRightsHandoutMode::NotApplicableJurisdictionLacksMandate,
            statutory_basis: "Jurisdiction lacks codified tenant handout mandate".to_string(),
            notes: "Jurisdiction does not impose tenant-handout obligation; default to federal Title X lead-paint disclosure for pre-1978 housing only.".to_string(),
            citations,
        };
    }

    if input.property_built_before_1978_lead_paint_disclosure_required
        && !input.lead_based_paint_disclosure_provided
    {
        return Output {
            mode: TenantBillOfRightsHandoutMode::ViolationLeadBasedPaintTitleXFederalDisclosureMissing,
            statutory_basis: "Federal Lead-Based Paint Hazard Reduction Act (Title X of 1992)".to_string(),
            notes: "VIOLATION: pre-1978 housing; federal Title X lead-paint disclosure not provided. Required EPA-approved disclosure form + EPA brochure.".to_string(),
            citations,
        };
    }

    if input.environmental_hazard_known_to_landlord && !input.environmental_hazard_disclosed {
        return Output {
            mode: TenantBillOfRightsHandoutMode::ViolationEnvironmentalHazardKnownButNotDisclosed,
            statutory_basis: "Cross-state common disclosure: known environmental hazard".to_string(),
            notes: "VIOLATION: landlord knew of environmental hazard but did not disclose; required across handout regimes.".to_string(),
            citations,
        };
    }

    match input.jurisdiction {
        HandoutJurisdiction::MichiganTruthInRenting => {
            if input.michigan_notice_font_status == MichiganNoticeFontStatus::NoticeMissingEntirely
            {
                return Output {
                    mode: TenantBillOfRightsHandoutMode::ViolationMichiganStatutoryNoticeMissing,
                    statutory_basis: "MCL § 554.633 — required Truth in Renting notice missing from lease".to_string(),
                    notes: "VIOLATION: Michigan Truth in Renting Act requires the statutory notice in every lease; notice is missing entirely.".to_string(),
                    citations,
                };
            }
            if input.michigan_notice_font_status
                == MichiganNoticeFontStatus::NoticeTooSmallBelow12Point
            {
                return Output {
                    mode: TenantBillOfRightsHandoutMode::ViolationMichiganStatutoryNoticeFontTooSmall,
                    statutory_basis: "MCL § 554.633 — notice must be at least 12-point type or 1/8 inch legible print".to_string(),
                    notes: format!(
                        "VIOLATION: Michigan Truth in Renting notice is below 12-point type / 1/8 inch height. Required font: at least {} points OR {}/{} inch legible print.",
                        MICHIGAN_REQUIRED_FONT_SIZE_POINTS,
                        MICHIGAN_REQUIRED_FONT_HEIGHT_INCHES_NUMERATOR,
                        MICHIGAN_REQUIRED_FONT_HEIGHT_INCHES_DENOMINATOR
                    ),
                    citations,
                };
            }
            if !input.michigan_domestic_abuse_notice_included {
                return Output {
                    mode: TenantBillOfRightsHandoutMode::ViolationMichiganDomesticAbuseNoticeMissing,
                    statutory_basis: "Michigan domestic abuse protection notice required in lease".to_string(),
                    notes: "VIOLATION: Michigan-specific domestic abuse protection notice not included in lease.".to_string(),
                    citations,
                };
            }
            if !input.security_deposit_location_disclosed || !input.owner_manager_identity_disclosed
            {
                return Output {
                    mode: TenantBillOfRightsHandoutMode::ViolationSecurityDepositLocationOrOwnerManagerIdentityMissing,
                    statutory_basis: "Michigan required disclosures: security deposit location + owner/manager identity".to_string(),
                    notes: "VIOLATION: Michigan landlord must disclose security deposit location AND owner/manager identity.".to_string(),
                    citations,
                };
            }
            if !input.utility_billing_arrangements_disclosed {
                return Output {
                    mode: TenantBillOfRightsHandoutMode::ViolationUtilityBillingArrangementsNotDisclosed,
                    statutory_basis: "Michigan utility billing disclosure required when tenant pays utilities".to_string(),
                    notes: "VIOLATION: Michigan landlord must disclose utility billing arrangements when tenant pays utilities.".to_string(),
                    citations,
                };
            }
            Output {
                mode: TenantBillOfRightsHandoutMode::CompliantMichiganStatutoryNoticeAndDisclosuresIncluded,
                statutory_basis: "Michigan Truth in Renting Act notice + disclosures included".to_string(),
                notes: "COMPLIANT: Michigan Truth in Renting Act notice at proper font + domestic abuse protection notice + security deposit + owner/manager + utility disclosures provided.".to_string(),
                citations,
            }
        }
        HandoutJurisdiction::NewJerseyTruthInRenting => {
            if input.new_jersey_distribution_status == NewJerseyDistributionStatus::NotDistributed {
                return Output {
                    mode: TenantBillOfRightsHandoutMode::ViolationNewJerseyTruthInRentingNotDistributed,
                    statutory_basis: "N.J. Stat. § 46:8-46 — Truth-in-Renting booklet distribution required; $100 penalty".to_string(),
                    notes: "VIOLATION: New Jersey Truth-in-Renting booklet not distributed to tenant at lease signing; $100 penalty per non-distribution.".to_string(),
                    citations,
                };
            }
            Output {
                mode: TenantBillOfRightsHandoutMode::CompliantNewJerseyTruthInRentingDistributed,
                statutory_basis:
                    "N.J. Stat. § 46:8-43 to § 46:8-50 — Truth-in-Renting booklet distributed"
                        .to_string(),
                notes: format!(
                    "COMPLIANT: New Jersey Truth-in-Renting booklet distributed per {:?} status.",
                    input.new_jersey_distribution_status
                ),
                citations,
            }
        }
        HandoutJurisdiction::DistrictOfColumbiaTenantBillOfRights => {
            if input.dc_handout_status == DcHandoutStatus::DcTenantBillOfRightsNotProvided {
                return Output {
                    mode: TenantBillOfRightsHandoutMode::ViolationDcTenantBillOfRightsNotProvided,
                    statutory_basis: "D.C. Code § 42-3502.22b — DC Tenant Bill of Rights must be provided".to_string(),
                    notes: "VIOLATION: D.C. landlord did not provide DC Tenant Bill of Rights at lease signing.".to_string(),
                    citations,
                };
            }
            Output {
                mode: TenantBillOfRightsHandoutMode::CompliantDcTenantBillOfRightsProvided,
                statutory_basis: "D.C. Code § 42-3502.22b — DC Tenant Bill of Rights provided".to_string(),
                notes: "COMPLIANT: DC Office of the Tenant Advocate Tenant Bill of Rights provided with lease + addendums.".to_string(),
                citations,
            }
        }
        HandoutJurisdiction::CaliforniaSection1962 => {
            if input.california_owner_identity_days_to_disclosure
                > CALIFORNIA_1962_OWNER_IDENTITY_DISCLOSURE_DAYS
                || !input.owner_manager_identity_disclosed
            {
                return Output {
                    mode: TenantBillOfRightsHandoutMode::ViolationCaliforniaOwnerIdentityNotDisclosedWithin15Days,
                    statutory_basis: "Cal. Civ. Code § 1962 — 15-day owner identity disclosure window".to_string(),
                    notes: format!(
                        "VIOLATION: California owner identity disclosure required within 15 days of tenancy; landlord disclosed in {} days OR identity not disclosed.",
                        input.california_owner_identity_days_to_disclosure
                    ),
                    citations,
                };
            }
            if !input.security_deposit_location_disclosed {
                return Output {
                    mode: TenantBillOfRightsHandoutMode::ViolationSecurityDepositLocationOrOwnerManagerIdentityMissing,
                    statutory_basis: "Cal. Civ. Code § 1962.5 / § 1962.7 — security deposit location disclosure".to_string(),
                    notes: "VIOLATION: California security deposit location not disclosed.".to_string(),
                    citations,
                };
            }
            Output {
                mode: TenantBillOfRightsHandoutMode::CompliantCaliforniaOwnerIdentityDisclosed,
                statutory_basis: "Cal. Civ. Code § 1962 + § 1962.5 / § 1962.7 disclosures satisfied".to_string(),
                notes: format!(
                    "COMPLIANT: California owner identity disclosed within {} days (≤ 15); security deposit location disclosed.",
                    input.california_owner_identity_days_to_disclosure
                ),
                citations,
            }
        }
        HandoutJurisdiction::FloridaStatute83_49 => {
            if input.florida_deposit_location_days_to_disclosure
                > FLORIDA_83_49_DEPOSIT_DISCLOSURE_DAYS
                || !input.security_deposit_location_disclosed
            {
                return Output {
                    mode: TenantBillOfRightsHandoutMode::ViolationFloridaDepositLocationNotDisclosedWithin30Days,
                    statutory_basis: "Fla. Stat. § 83.49 — 30-day deposit location disclosure window".to_string(),
                    notes: format!(
                        "VIOLATION: Florida deposit location disclosure required within 30 days of receipt; disclosed in {} days OR not disclosed.",
                        input.florida_deposit_location_days_to_disclosure
                    ),
                    citations,
                };
            }
            Output {
                mode: TenantBillOfRightsHandoutMode::CompliantFloridaDepositLocationDisclosed,
                statutory_basis: "Fla. Stat. § 83.49 deposit location disclosure satisfied"
                    .to_string(),
                notes: format!(
                    "COMPLIANT: Florida deposit location disclosed within {} days (≤ 30).",
                    input.florida_deposit_location_days_to_disclosure
                ),
                citations,
            }
        }
        HandoutJurisdiction::OtherStateWithoutHandoutMandate => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_michigan_compliant() -> Input {
        Input {
            jurisdiction: HandoutJurisdiction::MichiganTruthInRenting,
            lease_executed: true,
            property_built_before_1978_lead_paint_disclosure_required: false,
            lead_based_paint_disclosure_provided: false,
            security_deposit_location_disclosed: true,
            owner_manager_identity_disclosed: true,
            environmental_hazard_known_to_landlord: false,
            environmental_hazard_disclosed: false,
            utility_billing_arrangements_disclosed: true,
            michigan_notice_font_status:
                MichiganNoticeFontStatus::NoticeAtLeast12PointTypeOrLegiblePrintWithLettersAt1_8Inch,
            michigan_domestic_abuse_notice_included: true,
            new_jersey_distribution_status: NewJerseyDistributionStatus::DistributedAtLeaseSigning,
            dc_handout_status: DcHandoutStatus::DcTenantBillOfRightsProvidedWithLease,
            california_owner_identity_days_to_disclosure: 10,
            florida_deposit_location_days_to_disclosure: 25,
        }
    }

    #[test]
    fn no_lease_not_applicable() {
        let input = Input {
            lease_executed: false,
            ..baseline_michigan_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TenantBillOfRightsHandoutMode::NotApplicableNoLeaseExecuted
        );
    }

    #[test]
    fn other_jurisdiction_not_applicable() {
        let input = Input {
            jurisdiction: HandoutJurisdiction::OtherStateWithoutHandoutMandate,
            ..baseline_michigan_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TenantBillOfRightsHandoutMode::NotApplicableJurisdictionLacksMandate
        );
    }

    #[test]
    fn michigan_compliant() {
        let result = check(&baseline_michigan_compliant());
        assert_eq!(
            result.mode,
            TenantBillOfRightsHandoutMode::CompliantMichiganStatutoryNoticeAndDisclosuresIncluded
        );
    }

    #[test]
    fn michigan_notice_missing_violation() {
        let input = Input {
            michigan_notice_font_status: MichiganNoticeFontStatus::NoticeMissingEntirely,
            ..baseline_michigan_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TenantBillOfRightsHandoutMode::ViolationMichiganStatutoryNoticeMissing
        );
    }

    #[test]
    fn michigan_notice_font_too_small_violation() {
        let input = Input {
            michigan_notice_font_status: MichiganNoticeFontStatus::NoticeTooSmallBelow12Point,
            ..baseline_michigan_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TenantBillOfRightsHandoutMode::ViolationMichiganStatutoryNoticeFontTooSmall
        );
    }

    #[test]
    fn michigan_domestic_abuse_notice_missing_violation() {
        let input = Input {
            michigan_domestic_abuse_notice_included: false,
            ..baseline_michigan_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TenantBillOfRightsHandoutMode::ViolationMichiganDomesticAbuseNoticeMissing
        );
    }

    #[test]
    fn michigan_security_deposit_or_owner_missing_violation() {
        let input = Input {
            security_deposit_location_disclosed: false,
            ..baseline_michigan_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, TenantBillOfRightsHandoutMode::ViolationSecurityDepositLocationOrOwnerManagerIdentityMissing);
    }

    #[test]
    fn michigan_utility_billing_not_disclosed_violation() {
        let input = Input {
            utility_billing_arrangements_disclosed: false,
            ..baseline_michigan_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TenantBillOfRightsHandoutMode::ViolationUtilityBillingArrangementsNotDisclosed
        );
    }

    #[test]
    fn lead_paint_disclosure_missing_violation() {
        let input = Input {
            property_built_before_1978_lead_paint_disclosure_required: true,
            lead_based_paint_disclosure_provided: false,
            ..baseline_michigan_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TenantBillOfRightsHandoutMode::ViolationLeadBasedPaintTitleXFederalDisclosureMissing
        );
    }

    #[test]
    fn environmental_hazard_known_but_not_disclosed_violation() {
        let input = Input {
            environmental_hazard_known_to_landlord: true,
            environmental_hazard_disclosed: false,
            ..baseline_michigan_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TenantBillOfRightsHandoutMode::ViolationEnvironmentalHazardKnownButNotDisclosed
        );
    }

    #[test]
    fn new_jersey_distributed_compliant() {
        let input = Input {
            jurisdiction: HandoutJurisdiction::NewJerseyTruthInRenting,
            ..baseline_michigan_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TenantBillOfRightsHandoutMode::CompliantNewJerseyTruthInRentingDistributed
        );
    }

    #[test]
    fn new_jersey_not_distributed_violation() {
        let input = Input {
            jurisdiction: HandoutJurisdiction::NewJerseyTruthInRenting,
            new_jersey_distribution_status: NewJerseyDistributionStatus::NotDistributed,
            ..baseline_michigan_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TenantBillOfRightsHandoutMode::ViolationNewJerseyTruthInRentingNotDistributed
        );
    }

    #[test]
    fn dc_tenant_bill_of_rights_provided_compliant() {
        let input = Input {
            jurisdiction: HandoutJurisdiction::DistrictOfColumbiaTenantBillOfRights,
            ..baseline_michigan_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TenantBillOfRightsHandoutMode::CompliantDcTenantBillOfRightsProvided
        );
    }

    #[test]
    fn dc_tenant_bill_of_rights_not_provided_violation() {
        let input = Input {
            jurisdiction: HandoutJurisdiction::DistrictOfColumbiaTenantBillOfRights,
            dc_handout_status: DcHandoutStatus::DcTenantBillOfRightsNotProvided,
            ..baseline_michigan_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TenantBillOfRightsHandoutMode::ViolationDcTenantBillOfRightsNotProvided
        );
    }

    #[test]
    fn california_owner_identity_within_15_days_compliant() {
        let input = Input {
            jurisdiction: HandoutJurisdiction::CaliforniaSection1962,
            california_owner_identity_days_to_disclosure: 15,
            ..baseline_michigan_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TenantBillOfRightsHandoutMode::CompliantCaliforniaOwnerIdentityDisclosed
        );
    }

    #[test]
    fn california_owner_identity_16_days_violation() {
        let input = Input {
            jurisdiction: HandoutJurisdiction::CaliforniaSection1962,
            california_owner_identity_days_to_disclosure: 16,
            ..baseline_michigan_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TenantBillOfRightsHandoutMode::ViolationCaliforniaOwnerIdentityNotDisclosedWithin15Days
        );
    }

    #[test]
    fn florida_deposit_location_within_30_days_compliant() {
        let input = Input {
            jurisdiction: HandoutJurisdiction::FloridaStatute83_49,
            florida_deposit_location_days_to_disclosure: 30,
            ..baseline_michigan_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TenantBillOfRightsHandoutMode::CompliantFloridaDepositLocationDisclosed
        );
    }

    #[test]
    fn florida_deposit_location_31_days_violation() {
        let input = Input {
            jurisdiction: HandoutJurisdiction::FloridaStatute83_49,
            florida_deposit_location_days_to_disclosure: 31,
            ..baseline_michigan_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TenantBillOfRightsHandoutMode::ViolationFloridaDepositLocationNotDisclosedWithin30Days
        );
    }

    #[test]
    fn citations_pin_jurisdictional_statutes() {
        let result = check(&baseline_michigan_compliant());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("Michigan Truth in Renting Act"));
        assert!(joined.contains("Public Act 454 of 1978"));
        assert!(joined.contains("MCL § 554.631"));
        assert!(joined.contains("N.J. Stat. § 46:8-43"));
        assert!(joined.contains("§ 46:8-50"));
        assert!(joined.contains("D.C. Code § 42-3502.22b"));
        assert!(joined.contains("Cal. Civ. Code § 1962"));
        assert!(joined.contains("§ 1962.5"));
        assert!(joined.contains("§ 1962.7"));
        assert!(joined.contains("Fla. Stat. § 83.49"));
        assert!(joined.contains("Title X of 1992"));
    }

    #[test]
    fn constant_pin_jurisdictional_thresholds() {
        assert_eq!(MICHIGAN_TRUTH_IN_RENTING_ACT_YEAR, 1978);
        assert_eq!(MICHIGAN_TRUTH_IN_RENTING_PUBLIC_ACT_NUMBER, 454);
        assert_eq!(MICHIGAN_REQUIRED_FONT_SIZE_POINTS, 12);
        assert_eq!(MICHIGAN_REQUIRED_FONT_HEIGHT_INCHES_NUMERATOR, 1);
        assert_eq!(MICHIGAN_REQUIRED_FONT_HEIGHT_INCHES_DENOMINATOR, 8);
        assert_eq!(NEW_JERSEY_TRUTH_IN_RENTING_EXISTING_TENANT_DAYS, 30);
        assert_eq!(NEW_JERSEY_TRUTH_IN_RENTING_PENALTY_DOLLARS, 100);
        assert_eq!(CALIFORNIA_1962_OWNER_IDENTITY_DISCLOSURE_DAYS, 15);
        assert_eq!(FLORIDA_83_49_DEPOSIT_DISCLOSURE_DAYS, 30);
    }
}
