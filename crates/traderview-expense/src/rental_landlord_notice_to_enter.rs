//! Landlord notice-to-enter requirement framework.
//!
//! Most US states require landlords to provide advance notice before entering a tenant's
//! occupied rental unit for non-emergency purposes (repairs, inspections, showings to
//! prospective buyers/tenants, appraisers, court-ordered access). Notice must be reasonable
//! in both timing and purpose. Unannounced entry is a breach of the implied covenant of
//! quiet enjoyment under common law and a statutory violation in jurisdictions with
//! codified entry rules; tenant remedies include actual damages, attorney fees, lease
//! termination, and in some states statutory civil penalties.
//!
//! Notice requirements vary sharply by jurisdiction:
//!
//! - CA Civ. Code § 1954: 24 hours WRITTEN notice (48 hours for initial moveout
//!   inspection). Three permissible purposes only: necessary repairs, showing to
//!   prospective buyers/tenants/appraisers, court-ordered. Written-notice presumption:
//!   24 hours is reasonable.
//! - WA RCW 59.18.150: 48 hours notice generally; 24 hours notice permitted for
//!   showing to prospective buyers/tenants/appraisers.
//! - FL Stat. § 83.53: "reasonable notice" required (no statutory hour cap); 12 hours
//!   recognized as reasonable for tenant-requested repairs.
//! - IL Chicago RLTO § 5-12-050: 2 days (48 hours) notice required; Illinois statewide
//!   has no codified notice statute.
//! - CO Rev. Stat. § 38-12-510: 48 hours WRITTEN notice required (enacted via HB
//!   23-1095, effective Aug 7 2023).
//! - TX Prop. Code: NO statewide notice statute; lease terms control. Many local
//!   ordinances impose 24-hour requirements.
//! - NY: NO statewide statute; NYC Admin Code § 27-2008 + DHCR rent-stab regs
//!   require reasonable notice; common-law "reasonable advance notice" applies.
//! - Default: 24-hour notice is the prevailing standard across ~30 states with
//!   codified rules; reasonable purpose required.
//!
//! Emergency exceptions (every jurisdiction): immediate entry permitted without notice
//! for genuine emergencies (fire, gas leak, water leak causing damage, suspected
//! deceased tenant, abandoned unit).
//!
//! Citations (verified per WebSearch 2026-06-02):
//! - nolo.com/legal-encyclopedia/chart-notice-requirements-enter-rental-29033.html
//! - dimensionlaw.com/blog/landlord-tenant-series-qa-24-hour-notice-to-enter/
//! - leg.state.fl.us/statutes/index.cfm?App_mode=Display_Statute&URL=0000-0099%2F0083%2FSections%2F0083.53.html
//! - findlaw.com/realestate/landlord-tenant-law/requirements-for-landlord-entry.html

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    California,
    Washington,
    Florida,
    IllinoisChicagoRlto,
    IllinoisStatewideNoStatute,
    Colorado,
    Texas,
    NewYorkCity,
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntryPurpose {
    NecessaryRepairsOrMaintenance,
    InspectionRoutineOrAnnual,
    ShowingToProspectiveBuyersTenantsOrAppraisers,
    InitialMoveOutInspectionCaSection1950_5F,
    EmergencyImmediateThreat,
    TenantAbandonedUnit,
    CourtOrderedEntry,
    UnpermittedOrPretextual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoticeMethod {
    WrittenAdvanceNotice,
    OralAdvanceNotice,
    NoNoticeGiven,
    NoticeReceivedAfterEntry,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    EmergencyEntryNoNoticeRequired,
    TenantAbandonedUnitNoNoticeRequired,
    CourtOrderedEntryAuthorized,
    CompliantWrittenNoticeWithinWindow,
    CompliantOralNoticeWithinWindow,
    UnpermittedPurposeQuietEnjoymentBreach,
    InsufficientNoticeTimeStatutoryViolation,
    NoNoticeGivenStatutoryViolationPlusQuietEnjoymentBreach,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub jurisdiction: Jurisdiction,
    pub entry_purpose: EntryPurpose,
    pub notice_method: NoticeMethod,
    pub notice_hours_in_advance: u32,
    pub monthly_rent_cents: u64,
    pub tenant_actual_damages_cents: u64,
}

pub type RentalLandlordNoticeToEnterInput = Input;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub severity: Severity,
    pub required_notice_hours: u32,
    pub estimated_landlord_exposure_cents: u64,
    pub note: String,
}

pub type RentalLandlordNoticeToEnterOutput = Output;
pub type RentalLandlordNoticeToEnterResult = Output;

const CA_REQUIRED_NOTICE_HOURS: u32 = 24;
const CA_INITIAL_MOVEOUT_NOTICE_HOURS: u32 = 48;
const WA_REQUIRED_NOTICE_HOURS: u32 = 48;
const WA_SHOWING_NOTICE_HOURS: u32 = 24;
const FL_REASONABLE_NOTICE_HOURS: u32 = 12;
const IL_CHICAGO_RLTO_NOTICE_HOURS: u32 = 48;
const CO_REQUIRED_NOTICE_HOURS: u32 = 48;
const DEFAULT_NOTICE_HOURS: u32 = 24;
const QUIET_ENJOYMENT_BASELINE_MULTIPLIER: u64 = 1;

#[must_use]
pub fn check(input: &Input) -> Output {
    if matches!(input.entry_purpose, EntryPurpose::EmergencyImmediateThreat) {
        return Output {
            severity: Severity::EmergencyEntryNoNoticeRequired,
            required_notice_hours: 0,
            estimated_landlord_exposure_cents: 0,
            note: "Emergency entry — no notice required in any jurisdiction. Genuine \
                   emergencies (fire, gas leak, water leak causing imminent damage, suspected \
                   deceased tenant, immediate health/safety threat) authorize landlord entry \
                   without advance notice. Document the emergency: photographs, fire \
                   department report, plumber invoice, witness statements. Pretextual \
                   emergency claims are subject to bad-faith damages."
                .to_string(),
        };
    }

    if matches!(input.entry_purpose, EntryPurpose::TenantAbandonedUnit) {
        return Output {
            severity: Severity::TenantAbandonedUnitNoNoticeRequired,
            required_notice_hours: 0,
            estimated_landlord_exposure_cents: 0,
            note: "Tenant-abandoned unit — landlord may enter without notice once abandonment \
                   is established. CA Civ. Code § 1951.3 requires posted notice of belief of \
                   abandonment + 15-18 day cure window. FL Stat. § 83.595 + § 83.59 use absence \
                   exceeding one-half rent-period. Document the abandonment basis: extended \
                   absence + utility shutoff + mail accumulation + no rent payment. Premature \
                   abandonment declaration exposes landlord to wrongful-eviction claim."
                .to_string(),
        };
    }

    if matches!(input.entry_purpose, EntryPurpose::CourtOrderedEntry) {
        return Output {
            severity: Severity::CourtOrderedEntryAuthorized,
            required_notice_hours: 0,
            estimated_landlord_exposure_cents: 0,
            note: "Court-ordered entry — judicial order supersedes the notice requirement. \
                   Verify the order is for the correct unit, correct date/time, correct \
                   purpose. Have sheriff/marshal accompany if the order so requires. Tenant \
                   may still challenge the order on procedural grounds."
                .to_string(),
        };
    }

    if matches!(input.entry_purpose, EntryPurpose::UnpermittedOrPretextual) {
        let exposure = input
            .monthly_rent_cents
            .saturating_mul(QUIET_ENJOYMENT_BASELINE_MULTIPLIER)
            .saturating_add(input.tenant_actual_damages_cents);
        return Output {
            severity: Severity::UnpermittedPurposeQuietEnjoymentBreach,
            required_notice_hours: 0,
            estimated_landlord_exposure_cents: exposure,
            note: format!(
                "Unpermitted or pretextual entry purpose — common-law BREACH OF IMPLIED \
                 COVENANT OF QUIET ENJOYMENT regardless of notice compliance. Permissible \
                 purposes are limited to: necessary repairs/maintenance, routine inspection, \
                 showing to prospective buyers/tenants/appraisers, court-ordered, emergency, \
                 abandoned-unit recovery. Harassment, surveillance, retaliation, lifestyle \
                 monitoring are NOT permissible. Estimated exposure ${} = one month rent (${}) \
                 + actual damages (${}). Tenant may also recover constructive-eviction \
                 damages if entry pattern made unit uninhabitable.",
                exposure / 100,
                input.monthly_rent_cents / 100,
                input.tenant_actual_damages_cents / 100
            ),
        };
    }

    let required_hours = jurisdiction_required_hours(input.jurisdiction, input.entry_purpose);

    if matches!(input.notice_method, NoticeMethod::NoNoticeGiven)
        || matches!(input.notice_method, NoticeMethod::NoticeReceivedAfterEntry)
    {
        let exposure = input
            .monthly_rent_cents
            .saturating_mul(QUIET_ENJOYMENT_BASELINE_MULTIPLIER)
            .saturating_add(input.tenant_actual_damages_cents);
        return Output {
            severity: Severity::NoNoticeGivenStatutoryViolationPlusQuietEnjoymentBreach,
            required_notice_hours: required_hours,
            estimated_landlord_exposure_cents: exposure,
            note: format!(
                "No advance notice given (or notice arrived after entry). Statutory violation \
                 in jurisdictions with codified hour requirements + common-law breach of \
                 implied covenant of quiet enjoyment everywhere. {}-hour notice was required. \
                 Estimated exposure ${} = one month rent (${}) + actual damages (${}). Tenant \
                 may also obtain injunctive relief barring future entries without proper \
                 notice + lease termination + attorney fees where statute or lease permits.",
                required_hours,
                exposure / 100,
                input.monthly_rent_cents / 100,
                input.tenant_actual_damages_cents / 100
            ),
        };
    }

    if input.notice_hours_in_advance < required_hours {
        let exposure = input.tenant_actual_damages_cents;
        return Output {
            severity: Severity::InsufficientNoticeTimeStatutoryViolation,
            required_notice_hours: required_hours,
            estimated_landlord_exposure_cents: exposure,
            note: format!(
                "Insufficient notice time. Provided {} hours of advance notice; statute or \
                 ordinance requires {} hours. Statutory violation creates rebuttable \
                 presumption of unreasonable entry under jurisdiction's tenancy code. \
                 Estimated exposure ${} = tenant actual damages. Common-law quiet-enjoyment \
                 baseline also available; courts apply substantial-compliance analysis when \
                 deficit is de minimis but reject substantial-compliance defense when deficit \
                 cuts the notice window in half or more.",
                input.notice_hours_in_advance,
                required_hours,
                exposure / 100
            ),
        };
    }

    match input.notice_method {
        NoticeMethod::WrittenAdvanceNotice => Output {
            severity: Severity::CompliantWrittenNoticeWithinWindow,
            required_notice_hours: required_hours,
            estimated_landlord_exposure_cents: 0,
            note: format!(
                "Compliant: written advance notice {} hours before entry (requirement: {} \
                 hours). California Civ. Code § 1954 + WA RCW 59.18.150 + CO Rev. Stat. \
                 § 38-12-510 + IL Chicago RLTO § 5-12-050 all REQUIRE written notice — oral \
                 notice insufficient in those jurisdictions. Retain proof of service: photo \
                 of posted notice + USPS receipt + email timestamp + tenant acknowledgment.",
                input.notice_hours_in_advance, required_hours
            ),
        },
        NoticeMethod::OralAdvanceNotice => {
            let written_required = matches!(
                input.jurisdiction,
                Jurisdiction::California
                    | Jurisdiction::Washington
                    | Jurisdiction::Colorado
                    | Jurisdiction::IllinoisChicagoRlto
            );
            if written_required {
                return Output {
                    severity: Severity::InsufficientNoticeTimeStatutoryViolation,
                    required_notice_hours: required_hours,
                    estimated_landlord_exposure_cents: input.tenant_actual_damages_cents,
                    note: format!(
                        "Oral notice insufficient. Jurisdiction REQUIRES written notice (CA \
                         Civ. Code § 1954, WA RCW 59.18.150, CO Rev. Stat. § 38-12-510, IL \
                         Chicago RLTO § 5-12-050). Oral notice timing of {} hours was \
                         sufficient if written, but the written-notice requirement is itself \
                         a statutory element. Estimated exposure ${} = tenant actual damages.",
                        input.notice_hours_in_advance,
                        input.tenant_actual_damages_cents / 100
                    ),
                };
            }
            Output {
                severity: Severity::CompliantOralNoticeWithinWindow,
                required_notice_hours: required_hours,
                estimated_landlord_exposure_cents: 0,
                note: format!(
                    "Compliant: oral advance notice {} hours before entry (requirement: {} \
                     hours). Jurisdiction permits oral notice where written notice is not \
                     statutorily required. Best practice: confirm in writing (text message, \
                     email) to create a contemporaneous record for any future dispute.",
                    input.notice_hours_in_advance, required_hours
                ),
            }
        }
        NoticeMethod::NoNoticeGiven | NoticeMethod::NoticeReceivedAfterEntry => unreachable!(),
    }
}

fn jurisdiction_required_hours(jurisdiction: Jurisdiction, entry_purpose: EntryPurpose) -> u32 {
    match jurisdiction {
        Jurisdiction::California => {
            if matches!(
                entry_purpose,
                EntryPurpose::InitialMoveOutInspectionCaSection1950_5F
            ) {
                CA_INITIAL_MOVEOUT_NOTICE_HOURS
            } else {
                CA_REQUIRED_NOTICE_HOURS
            }
        }
        Jurisdiction::Washington => {
            if matches!(
                entry_purpose,
                EntryPurpose::ShowingToProspectiveBuyersTenantsOrAppraisers
            ) {
                WA_SHOWING_NOTICE_HOURS
            } else {
                WA_REQUIRED_NOTICE_HOURS
            }
        }
        Jurisdiction::Florida => FL_REASONABLE_NOTICE_HOURS,
        Jurisdiction::IllinoisChicagoRlto => IL_CHICAGO_RLTO_NOTICE_HOURS,
        Jurisdiction::IllinoisStatewideNoStatute => DEFAULT_NOTICE_HOURS,
        Jurisdiction::Colorado => CO_REQUIRED_NOTICE_HOURS,
        Jurisdiction::Texas => DEFAULT_NOTICE_HOURS,
        Jurisdiction::NewYorkCity => DEFAULT_NOTICE_HOURS,
        Jurisdiction::Default => DEFAULT_NOTICE_HOURS,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Input {
        Input {
            jurisdiction: Jurisdiction::California,
            entry_purpose: EntryPurpose::NecessaryRepairsOrMaintenance,
            notice_method: NoticeMethod::WrittenAdvanceNotice,
            notice_hours_in_advance: 24,
            monthly_rent_cents: 3_000_00,
            tenant_actual_damages_cents: 1_000_00,
        }
    }

    #[test]
    fn emergency_entry_no_notice_required() {
        let mut input = base();
        input.entry_purpose = EntryPurpose::EmergencyImmediateThreat;
        input.notice_method = NoticeMethod::NoNoticeGiven;
        let output = check(&input);
        assert_eq!(output.severity, Severity::EmergencyEntryNoNoticeRequired);
        assert_eq!(output.estimated_landlord_exposure_cents, 0);
        assert!(output.note.contains("emergencies"));
    }

    #[test]
    fn tenant_abandoned_unit_no_notice_required() {
        let mut input = base();
        input.entry_purpose = EntryPurpose::TenantAbandonedUnit;
        input.notice_method = NoticeMethod::NoNoticeGiven;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::TenantAbandonedUnitNoNoticeRequired
        );
        assert!(output.note.contains("CA Civ. Code § 1951.3"));
        assert!(output.note.contains("FL Stat. § 83.595"));
    }

    #[test]
    fn court_ordered_entry_authorized() {
        let mut input = base();
        input.entry_purpose = EntryPurpose::CourtOrderedEntry;
        input.notice_method = NoticeMethod::NoNoticeGiven;
        let output = check(&input);
        assert_eq!(output.severity, Severity::CourtOrderedEntryAuthorized);
        assert!(output.note.contains("judicial order supersedes"));
    }

    #[test]
    fn unpermitted_purpose_breaches_quiet_enjoyment() {
        let mut input = base();
        input.entry_purpose = EntryPurpose::UnpermittedOrPretextual;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::UnpermittedPurposeQuietEnjoymentBreach
        );
        // $3,000 rent + $1,000 actual = $4,000
        assert_eq!(output.estimated_landlord_exposure_cents, 4_000_00);
        assert!(output.note.contains("QUIET ENJOYMENT"));
        assert!(output.note.contains("Harassment"));
    }

    #[test]
    fn california_24_hour_written_notice_compliant() {
        let input = base();
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::CompliantWrittenNoticeWithinWindow
        );
        assert_eq!(output.required_notice_hours, 24);
        assert_eq!(output.estimated_landlord_exposure_cents, 0);
        assert!(output.note.contains("§ 1954"));
    }

    #[test]
    fn california_initial_moveout_inspection_requires_48_hours() {
        let mut input = base();
        input.entry_purpose = EntryPurpose::InitialMoveOutInspectionCaSection1950_5F;
        input.notice_hours_in_advance = 24;
        let output = check(&input);
        // 24 hours given, 48 hours required → insufficient
        assert_eq!(
            output.severity,
            Severity::InsufficientNoticeTimeStatutoryViolation
        );
        assert_eq!(output.required_notice_hours, 48);
    }

    #[test]
    fn california_initial_moveout_48_hours_compliant() {
        let mut input = base();
        input.entry_purpose = EntryPurpose::InitialMoveOutInspectionCaSection1950_5F;
        input.notice_hours_in_advance = 48;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::CompliantWrittenNoticeWithinWindow
        );
        assert_eq!(output.required_notice_hours, 48);
    }

    #[test]
    fn california_oral_notice_insufficient_even_if_timing_adequate() {
        let mut input = base();
        input.notice_method = NoticeMethod::OralAdvanceNotice;
        input.notice_hours_in_advance = 48; // Way more than required
        let output = check(&input);
        // CA requires WRITTEN, oral fails
        assert_eq!(
            output.severity,
            Severity::InsufficientNoticeTimeStatutoryViolation
        );
        assert!(output.note.contains("REQUIRES written notice"));
    }

    #[test]
    fn washington_48_hour_default_notice_required() {
        let mut input = base();
        input.jurisdiction = Jurisdiction::Washington;
        input.notice_hours_in_advance = 24;
        let output = check(&input);
        // 24 hours given for repair, 48 required
        assert_eq!(
            output.severity,
            Severity::InsufficientNoticeTimeStatutoryViolation
        );
        assert_eq!(output.required_notice_hours, 48);
    }

    #[test]
    fn washington_showing_to_buyer_only_requires_24_hours() {
        let mut input = base();
        input.jurisdiction = Jurisdiction::Washington;
        input.entry_purpose = EntryPurpose::ShowingToProspectiveBuyersTenantsOrAppraisers;
        input.notice_hours_in_advance = 24;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::CompliantWrittenNoticeWithinWindow
        );
        assert_eq!(output.required_notice_hours, 24);
    }

    #[test]
    fn florida_12_hour_window_compliant() {
        let mut input = base();
        input.jurisdiction = Jurisdiction::Florida;
        input.notice_hours_in_advance = 12;
        // FL doesn't require written, but written is supplied which is acceptable
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::CompliantWrittenNoticeWithinWindow
        );
        assert_eq!(output.required_notice_hours, 12);
    }

    #[test]
    fn florida_oral_notice_permitted() {
        let mut input = base();
        input.jurisdiction = Jurisdiction::Florida;
        input.notice_method = NoticeMethod::OralAdvanceNotice;
        input.notice_hours_in_advance = 24;
        let output = check(&input);
        assert_eq!(output.severity, Severity::CompliantOralNoticeWithinWindow);
        assert!(output.note.contains("oral notice"));
    }

    #[test]
    fn illinois_chicago_rlto_48_hour_written_required() {
        let mut input = base();
        input.jurisdiction = Jurisdiction::IllinoisChicagoRlto;
        input.notice_hours_in_advance = 48;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::CompliantWrittenNoticeWithinWindow
        );
        assert_eq!(output.required_notice_hours, 48);
    }

    #[test]
    fn illinois_statewide_no_statute_falls_back_to_24_hour_default() {
        let mut input = base();
        input.jurisdiction = Jurisdiction::IllinoisStatewideNoStatute;
        input.notice_hours_in_advance = 24;
        let output = check(&input);
        assert_eq!(output.required_notice_hours, 24);
        assert_eq!(
            output.severity,
            Severity::CompliantWrittenNoticeWithinWindow
        );
    }

    #[test]
    fn colorado_48_hour_written_required_post_2023() {
        let mut input = base();
        input.jurisdiction = Jurisdiction::Colorado;
        input.notice_hours_in_advance = 48;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::CompliantWrittenNoticeWithinWindow
        );
        assert_eq!(output.required_notice_hours, 48);
    }

    #[test]
    fn colorado_oral_notice_insufficient() {
        let mut input = base();
        input.jurisdiction = Jurisdiction::Colorado;
        input.notice_method = NoticeMethod::OralAdvanceNotice;
        input.notice_hours_in_advance = 48;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::InsufficientNoticeTimeStatutoryViolation
        );
        assert!(output.note.contains("CO Rev. Stat. § 38-12-510"));
    }

    #[test]
    fn texas_no_statewide_falls_back_to_24_hour_default() {
        let mut input = base();
        input.jurisdiction = Jurisdiction::Texas;
        input.notice_hours_in_advance = 24;
        let output = check(&input);
        assert_eq!(output.required_notice_hours, 24);
    }

    #[test]
    fn no_notice_given_full_violation() {
        let mut input = base();
        input.notice_method = NoticeMethod::NoNoticeGiven;
        input.notice_hours_in_advance = 0;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::NoNoticeGivenStatutoryViolationPlusQuietEnjoymentBreach
        );
        // $3,000 rent + $1,000 actual = $4,000
        assert_eq!(output.estimated_landlord_exposure_cents, 4_000_00);
    }

    #[test]
    fn notice_received_after_entry_treated_as_no_notice() {
        let mut input = base();
        input.notice_method = NoticeMethod::NoticeReceivedAfterEntry;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::NoNoticeGivenStatutoryViolationPlusQuietEnjoymentBreach
        );
    }

    #[test]
    fn boundary_exactly_required_hours_compliant() {
        let mut input = base();
        input.notice_hours_in_advance = 24;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::CompliantWrittenNoticeWithinWindow
        );
    }

    #[test]
    fn boundary_23_hours_insufficient_for_california_24() {
        let mut input = base();
        input.notice_hours_in_advance = 23;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::InsufficientNoticeTimeStatutoryViolation
        );
    }

    #[test]
    fn ca_required_notice_hours_constant_pins_24() {
        assert_eq!(CA_REQUIRED_NOTICE_HOURS, 24);
    }

    #[test]
    fn ca_initial_moveout_notice_hours_constant_pins_48() {
        assert_eq!(CA_INITIAL_MOVEOUT_NOTICE_HOURS, 48);
    }

    #[test]
    fn wa_required_notice_hours_constant_pins_48() {
        assert_eq!(WA_REQUIRED_NOTICE_HOURS, 48);
    }

    #[test]
    fn wa_showing_notice_hours_constant_pins_24() {
        assert_eq!(WA_SHOWING_NOTICE_HOURS, 24);
    }

    #[test]
    fn fl_reasonable_notice_hours_constant_pins_12() {
        assert_eq!(FL_REASONABLE_NOTICE_HOURS, 12);
    }

    #[test]
    fn il_chicago_rlto_notice_hours_constant_pins_48() {
        assert_eq!(IL_CHICAGO_RLTO_NOTICE_HOURS, 48);
    }

    #[test]
    fn co_required_notice_hours_constant_pins_48() {
        assert_eq!(CO_REQUIRED_NOTICE_HOURS, 48);
    }

    #[test]
    fn default_notice_hours_constant_pins_24() {
        assert_eq!(DEFAULT_NOTICE_HOURS, 24);
    }

    #[test]
    fn note_unpermitted_purpose_mentions_constructive_eviction() {
        let mut input = base();
        input.entry_purpose = EntryPurpose::UnpermittedOrPretextual;
        let output = check(&input);
        assert!(output.note.contains("constructive-eviction"));
    }

    #[test]
    fn note_no_notice_mentions_injunctive_relief() {
        let mut input = base();
        input.notice_method = NoticeMethod::NoNoticeGiven;
        let output = check(&input);
        assert!(output.note.contains("injunctive relief"));
    }

    #[test]
    fn very_large_monthly_rent_no_overflow() {
        let mut input = base();
        input.entry_purpose = EntryPurpose::UnpermittedOrPretextual;
        input.monthly_rent_cents = u64::MAX;
        input.tenant_actual_damages_cents = 100_000;
        let output = check(&input);
        // saturating_add defense — should saturate at u64::MAX
        assert_eq!(output.estimated_landlord_exposure_cents, u64::MAX);
    }

    #[test]
    fn zero_monthly_rent_no_panic_in_unpermitted_purpose() {
        let mut input = base();
        input.entry_purpose = EntryPurpose::UnpermittedOrPretextual;
        input.monthly_rent_cents = 0;
        let output = check(&input);
        // Only actual damages remain
        assert_eq!(output.estimated_landlord_exposure_cents, 1_000_00);
    }
}
