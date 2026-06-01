//! Landlord / Schedule E routes — properties, tenants, leases, income,
//! expenses, mileage, maintenance, services log, and the per-year roll-up.
//!
//! Mounted as a sub-router under `/api/rental`. Auth uses the same
//! `AuthUser` extractor as the trading routes. SQL is inlined here in the
//! same style as `expense_routes`.
//!
//! Ownership is enforced at every endpoint: every read/write either filters
//! `user_id = $1` directly or joins through `rental_properties.user_id`. A
//! `Forbidden` response is returned when a property exists but belongs to a
//! different user; `NotFound` when no row matches.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::routing::{delete, get, patch};
use axum::{Json, Router};
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use traderview_expense::deposit_interest::{accrue as accrue_deposit_interest, AccrualInput, AccrualResult};
use traderview_expense::disposition::{
    compute as compute_disposition, DispositionInput, DispositionReport,
};
use traderview_expense::rental_depreciation::{
    macrs_rental_year_1_deduction, RealPropertyClass,
};
use traderview_expense::contractor_1099::{
    compute as compute_contractor_1099, Contractor1099Input, Contractor1099Report,
};
use traderview_expense::cost_segregation::{
    compute as compute_cost_segregation, CostSegInput, CostSegReport,
    PropertyTypeDefault as CostSegPropertyType,
};
use traderview_expense::deposit_return_windows::{
    check as check_deposit_return, DepositReturnCheckInput, DepositReturnCheckResult,
};
use traderview_expense::lease_disclosures::{
    required_for as lease_disclosures_required_for, DisclosuresRequiredInput,
    DisclosuresRequiredReport,
};
use traderview_expense::habitability_remedies::{
    remedies as compute_habitability_remedies, HabitabilityRemediesInput,
    HabitabilityRemediesReport,
};
use traderview_expense::military_termination::{
    check as check_military_termination, MilitaryTerminationCheckInput,
    MilitaryTerminationCheckResult,
};
use traderview_expense::security_deposit_caps::{
    check as check_security_deposit_cap, SecurityDepositCheckInput,
    SecurityDepositCheckResult,
};
use traderview_expense::rent_control::{
    check as check_rent_increase, RentIncreaseCheckInput, RentIncreaseCheckResult,
};
use traderview_expense::entry_notice::{
    compute as check_entry_notice, EntryNoticeInput, EntryNoticeResult,
};
use traderview_expense::application_fees::{
    check as check_application_fee, AppFeeCheckInput, AppFeeCheckResult,
};
use traderview_expense::bedbug_disclosure::{
    check as check_bedbug_disclosure, BedbugCheckInput, BedbugCheckResult,
};
use traderview_expense::dv_survivor_lock_change::{
    check as check_dv_survivor_lock_change, DvLockChangeInput, DvLockChangeResult,
};
use traderview_expense::dv_termination::{
    check as check_dv_termination, DvEarlyTerminationInput, DvEarlyTerminationResult,
};
use traderview_expense::just_cause_eviction::{
    check as check_just_cause, JustCauseInput, JustCauseResult,
};
use traderview_expense::detector_requirements::{
    check as check_detector, DetectorCheckInput, DetectorCheckResult,
};
use traderview_expense::foreclosure_tenant_rights::{
    check as check_foreclosure_tenant, ForeclosureTenantInput, ForeclosureTenantResult,
};
use traderview_expense::heat_requirements::{
    check as check_heat_requirements, HeatCheckInput, HeatCheckResult,
};
use traderview_expense::mold_disclosure::{
    check as check_mold_disclosure, MoldCheckInput, MoldCheckResult,
};
use traderview_expense::radon_disclosure::{
    check as check_radon_disclosure, RadonDisclosureInput, RadonDisclosureResult,
};
use traderview_expense::lead_disclosure::{
    check as check_lead_disclosure, LeadCheckInput, LeadCheckResult,
};
use traderview_expense::soi_protection::{
    check as check_soi_protection, SoiCheckInput, SoiCheckResult,
};
use traderview_expense::str_regulation::{
    check as check_str_regulation, StrComplianceInput, StrComplianceResult,
};
use traderview_expense::pet_fees::{
    check as check_pet_fees, PetFeeInput, PetFeeResult,
};
use traderview_expense::eviction_record_sealing::{
    check as check_eviction_sealing, EvictionSealingInput, EvictionSealingResult,
};
use traderview_expense::lease_termination_notice::{
    check as check_termination_notice, NoticeInput, NoticeResult,
};
use traderview_expense::occupancy_standards::{
    check as check_occupancy, OccupancyInput, OccupancyResult,
};
use traderview_expense::move_in_inspection::{
    check as check_move_in_inspection, InspectionInput, InspectionResult,
};
use traderview_expense::renters_insurance::{
    check as check_renters_insurance, RentersInsuranceInput, RentersInsuranceResult,
};
use traderview_expense::utility_shutoff::{
    check as check_utility_shutoff, ShutoffInput, ShutoffResult,
};
use traderview_expense::vehicle_towing_from_rental_property::{
    check as check_vehicle_towing, TowingInput as VehicleTowingInput,
    TowingResult as VehicleTowingResult,
};
use traderview_expense::adverse_action_notice::{
    check as check_adverse_action, AdverseActionInput, AdverseActionResult,
};
use traderview_expense::adverse_possession_claim::{
    check as check_adverse_possession, AdversePossessionInput, AdversePossessionResult,
};
use traderview_expense::tenant_topa::{
    check as check_tenant_topa, TopaInput, TopaResult,
};
use traderview_expense::lease_auto_renewal::{
    check as check_lease_auto_renewal, AutoRenewalInput, AutoRenewalResult,
};
use traderview_expense::lease_translation::{
    check as check_lease_translation, TranslationInput, TranslationResult,
};
use traderview_expense::rent_receipts::{
    check as check_rent_receipts, ReceiptInput, ReceiptResult,
};
use traderview_expense::repair_and_deduct::{
    check as check_repair_and_deduct, RepairDeductInput, RepairDeductResult,
};
use traderview_expense::cosigner_rules::{
    check as check_cosigner_rules, CosignerInput, CosignerResult,
};
use traderview_expense::mobile_home_park::{
    check as check_mobile_home_park, MhpInput, MhpResult,
};
use traderview_expense::submetering_rules::{
    check as check_submetering, SubmeteringInput, SubmeteringResult,
};
use traderview_expense::smoke_free_housing::{
    check as check_smoke_free, SmokeFreeInput, SmokeFreeResult,
};
use traderview_expense::tenant_data_privacy::{
    check as check_tenant_privacy, PrivacyInput, PrivacyResult,
};
use traderview_expense::drug_eviction::{
    check as check_drug_eviction, DrugEvictionInput, DrugEvictionResult,
};
use traderview_expense::quiet_enjoyment::{
    check as check_quiet_enjoyment, QuietEnjoymentInput, QuietEnjoymentResult,
};
use traderview_expense::flood_disclosure::{
    check as check_flood_disclosure, FloodDisclosureInput, FloodDisclosureResult,
};
use traderview_expense::owner_identification::{
    check as check_owner_identification, OwnerIdentificationInput, OwnerIdentificationResult,
};
use traderview_expense::tenant_death_termination::{
    check as check_tenant_death, TenantDeathInput, TenantDeathResult,
};
use traderview_expense::late_payment_grace_period::{
    check as check_grace_period, GracePeriodInput, GracePeriodResult,
};
use traderview_expense::owner_move_in_eviction::{
    check as check_owner_move_in, OwnerMoveInInput, OwnerMoveInResult,
};
use traderview_expense::lease_copy_delivery::{
    check as check_lease_copy_delivery, LeaseCopyDeliveryInput, LeaseCopyDeliveryResult,
};
use traderview_expense::tenant_relocation_assistance::{
    compute as compute_tenant_relocation_assistance, RelocationInput as TenantRelocationInput,
    RelocationResult as TenantRelocationResult,
};
use traderview_expense::fair_chance_housing::{
    check as check_fair_chance_housing, FairChanceInput, FairChanceResult,
};
use traderview_expense::meth_contamination_disclosure::{
    check as check_meth_contamination_disclosure, MethDisclosureInput, MethDisclosureResult,
};
use traderview_expense::death_in_unit_disclosure::{
    check as check_death_in_unit_disclosure, DeathDisclosureInput, DeathDisclosureResult,
};
use traderview_expense::rent_payment_method::{
    check as check_rent_payment_method, RentPaymentMethodInput, RentPaymentMethodResult,
};
use traderview_expense::window_guard_requirements::{
    check as check_window_guard_requirements, WindowGuardInput, WindowGuardResult,
};
use traderview_expense::rent_increase_notice_period::{
    check as check_rent_increase_notice_period, RentIncreaseNoticeInput, RentIncreaseNoticeResult,
};
use traderview_expense::demolition_tenant_notice::{
    check as check_demolition_tenant_notice, DemolitionNoticeInput, DemolitionNoticeResult,
};
use traderview_expense::eviction_diversion_program::{
    check as check_eviction_diversion_program, DiversionProgramInput, DiversionProgramResult,
};
use traderview_expense::immigration_status_protection::{
    check as check_immigration_status_protection, ImmigrationProtectionInput,
    ImmigrationProtectionResult,
};
use traderview_expense::prevailing_party_attorney_fees::{
    check as check_prevailing_party_attorney_fees, PrevailingPartyFeesInput,
    PrevailingPartyFeesResult,
};
use traderview_expense::abandoned_property_handling::{
    check as check_abandoned_property_handling, AbandonedPropertyInput,
    AbandonedPropertyResult,
};
use traderview_expense::right_to_counsel_eviction::{
    check as check_right_to_counsel_eviction, RightToCounselInput, RightToCounselResult,
};
use traderview_expense::tenant_cannabis_use_protection::{
    check as check_tenant_cannabis_use_protection, CannabisProtectionInput,
    CannabisProtectionResult,
};
use traderview_expense::snow_removal_responsibility::{
    check as check_snow_removal_responsibility, SnowRemovalInput, SnowRemovalResult,
};
use traderview_expense::security_camera_disclosure::{
    check as check_security_camera_disclosure, SecurityCameraInput, SecurityCameraResult,
};
use traderview_expense::carpet_replacement_useful_life::{
    check as check_carpet_replacement_useful_life, CarpetReplacementInput,
    CarpetReplacementResult,
};
use traderview_expense::pre_move_out_inspection::{
    check as check_pre_move_out_inspection, PreMoveOutInspectionInput,
    PreMoveOutInspectionResult,
};
use traderview_expense::credit_check_authorization::{
    check as check_credit_check_authorization, CreditCheckInput, CreditCheckResult,
};
use traderview_expense::winter_eviction_protections::{
    check as check_winter_eviction_protections, WinterEvictionInput, WinterEvictionResult,
};
use traderview_expense::landlord_identification_disclosure::{
    check as check_landlord_identification_disclosure, LandlordIdentificationInput,
    LandlordIdentificationResult,
};
use traderview_expense::reasonable_accommodation_modification::{
    check as check_reasonable_accommodation_modification,
    CheckResult as ReasonableAccommodationResult, Input as ReasonableAccommodationInput,
};
use traderview_expense::damage_deduction_itemization::{
    check as check_damage_deduction_itemization, CheckResult as DamageDeductionResult,
    Input as DamageDeductionInput,
};
use traderview_expense::cooling_requirements::{
    check as check_cooling_requirements, CheckResult as CoolingRequirementsResult,
    Input as CoolingRequirementsInput,
};
use traderview_expense::duty_to_mitigate_damages::{
    check as check_duty_to_mitigate_damages, CheckResult as DutyToMitigateResult,
    Input as DutyToMitigateInput,
};
use traderview_expense::pesticide_application_notice::{
    check as check_pesticide_application_notice, CheckResult as PesticideNoticeResult,
    Input as PesticideNoticeInput,
};
use traderview_expense::condominium_conversion_protection::{
    check as check_condominium_conversion_protection, CheckResult as CondoConversionResult,
    Input as CondoConversionInput,
};
use traderview_expense::otard_antenna_installation::{
    check as check_otard_antenna_installation, CheckResult as OtardAntennaResult,
    Input as OtardAntennaInput,
};
use traderview_expense::religious_display_doorpost::{
    check as check_religious_display_doorpost, CheckResult as ReligiousDisplayResult,
    Input as ReligiousDisplayInput,
};
use traderview_expense::asbestos_disclosure::{
    check as check_asbestos_disclosure, CheckResult as AsbestosDisclosureResult,
    Input as AsbestosDisclosureInput,
};
use traderview_expense::firearms_in_rental_unit::{
    check as check_firearms_in_rental_unit, CheckResult as FirearmsRentalResult,
    Input as FirearmsRentalInput,
};
use traderview_expense::lock_change_between_tenancies::{
    check as check_lock_change_between_tenancies, CheckResult as LockChangeResult,
    Input as LockChangeInput,
};
use traderview_expense::landlord_lien_prohibition::{
    check as check_landlord_lien_prohibition, CheckResult as LandlordLienResult,
    Input as LandlordLienInput,
};
use traderview_expense::military_ordnance_disclosure::{
    check as check_military_ordnance_disclosure, CheckResult as MilitaryOrdnanceResult,
    Input as MilitaryOrdnanceInput,
};
use traderview_expense::sex_offender_database_notice::{
    check as check_sex_offender_database_notice, CheckResult as SexOffenderNoticeResult,
    Input as SexOffenderNoticeInput,
};
use traderview_expense::mid_tenancy_ownership_change::{
    check as check_mid_tenancy_ownership_change, CheckResult as MidTenancyOwnershipResult,
    Input as MidTenancyOwnershipInput,
};
use traderview_expense::mid_tenancy_term_modification::{
    check as check_mid_tenancy_term_modification, ModificationInput as MidTenancyTermModInput,
    ModificationResult as MidTenancyTermModResult,
};
use traderview_expense::tenant_solar_installation::{
    check as check_tenant_solar_installation, CheckResult as TenantSolarResult,
    Input as TenantSolarInput,
};
use traderview_expense::flag_display_right::{
    check as check_flag_display_right, CheckResult as FlagDisplayResult,
    Input as FlagDisplayInput,
};
use traderview_expense::written_lease_requirement::{
    check as check_written_lease_requirement, CheckResult as WrittenLeaseResult,
    Input as WrittenLeaseInput,
};
use traderview_expense::holdover_tenant_damages::{
    check as check_holdover_tenant_damages, CheckResult as HoldoverTenantResult,
    Input as HoldoverTenantInput,
};
use traderview_expense::lease_assignment_consent::{
    check as check_lease_assignment_consent, CheckResult as LeaseAssignmentResult,
    Input as LeaseAssignmentInput,
};
use traderview_expense::lease_cure_period::{
    check as check_lease_cure_period, CheckResult as LeaseCureResult,
    Input as LeaseCureInput,
};
use traderview_expense::portable_tenant_screening_report::{
    check as check_portable_screening, CheckResult as PortableScreeningResult,
    Input as PortableScreeningInput,
};
use traderview_expense::hoa_rental_restriction::{
    check as check_hoa_rental_restriction, CheckResult as HoaRentalResult,
    Input as HoaRentalInput,
};
use traderview_expense::rent_acceleration_enforceability::{
    check as check_rent_acceleration, CheckResult as RentAccelerationResult,
    Input as RentAccelerationInput,
};
use traderview_expense::tenant_in_foreclosure_protection::{
    check as check_tenant_foreclosure_protection,
    CheckResult as TenantForeclosureResult, Input as TenantForeclosureInput,
};
use traderview_expense::security_deposit_bank_disclosure::{
    check as check_security_deposit_bank_disclosure,
    CheckResult as SecurityDepositBankDisclosureResult,
    Input as SecurityDepositBankDisclosureInput,
};
use traderview_expense::landlord_harassment::{
    check as check_landlord_harassment, CheckResult as LandlordHarassmentResult,
    Input as LandlordHarassmentInput,
};
use traderview_expense::landlord_possession_delivery::{
    check as check_landlord_possession_delivery,
    CheckResult as LandlordPossessionDeliveryResult,
    Input as LandlordPossessionDeliveryInput,
};
use traderview_expense::lease_waiver_enforceability::{
    check as check_lease_waiver_enforceability,
    CheckResult as LeaseWaiverEnforceabilityResult,
    Input as LeaseWaiverEnforceabilityInput,
};
use traderview_expense::landlord_retaliation_damages::{
    check as check_landlord_retaliation_damages,
    CheckResult as LandlordRetaliationDamagesResult,
    Input as LandlordRetaliationDamagesInput,
};
use traderview_expense::last_month_rent_offset::{
    check as check_last_month_rent_offset,
    CheckResult as LastMonthRentOffsetResult,
    Input as LastMonthRentOffsetInput,
};
use traderview_expense::emotional_support_animal_documentation::{
    check as check_esa_documentation,
    CheckResult as EsaDocumentationResult,
    Input as EsaDocumentationInput,
};
use traderview_expense::lease_nondisparagement_prohibition::{
    check as check_lease_nondisparagement_prohibition,
    CheckResult as LeaseNondisparagementResult,
    Input as LeaseNondisparagementInput,
};
use traderview_expense::tenant_organizing::{
    check as check_tenant_organizing, TenantOrganizingInput, TenantOrganizingResult,
};
use traderview_expense::plain_language_lease::{
    check as check_plain_language, PlainLanguageInput, PlainLanguageResult,
};
use traderview_expense::roommate_authorization::{
    check as check_roommate_authorization, RoommateAuthorizationInput, RoommateAuthorizationResult,
};
use traderview_expense::ev_charger_installation::{
    check as check_ev_charger, EvChargerInput, EvChargerResult,
};
use traderview_expense::advance_rent_limit::{
    check as check_advance_rent, AdvanceRentInput, AdvanceRentResult,
};
use traderview_expense::fire_sprinkler_disclosure::{
    check as check_fire_sprinkler, FireSprinklerDisclosureInput, FireSprinklerDisclosureResult,
};
use traderview_expense::bedbug_extermination_cost::{
    check as check_bedbug_extermination, BedbugExterminationInput, BedbugExterminationResult,
};
use traderview_expense::crime_victim_termination::{
    check as check_crime_victim_termination,
    CrimeVictimTerminationInput, CrimeVictimTerminationResult,
};
use traderview_expense::lease_succession::{
    check as check_lease_succession, LeaseSuccessionInput, LeaseSuccessionResult,
};
use traderview_expense::rent_credit_reporting::{
    check as check_rent_credit_reporting, RentCreditReportingInput, RentCreditReportingResult,
};
use traderview_expense::rent_escrow::{
    check as check_rent_escrow, RentEscrowInput, RentEscrowResult,
};
use traderview_expense::right_to_dry::{
    check as check_right_to_dry, RightToDryInput, RightToDryResult,
};
use traderview_expense::sublet_consent::{
    check as check_sublet_consent, SubletConsentInput, SubletConsentResult,
};
use traderview_expense::senior_disabled_protection::{
    check as check_senior_disabled, SeniorDisabledCheckInput, SeniorDisabledCheckResult,
};
use traderview_expense::service_animal::{
    check as check_service_animal, ServiceAnimalCheckInput, ServiceAnimalCheckResult,
};
use traderview_expense::tenant_abandonment::{
    check as check_tenant_abandonment, TenantAbandonmentInput, TenantAbandonmentResult,
};
use traderview_expense::lockout_penalties::{
    check as check_lockout_penalty, LockoutPenaltyInput, LockoutPenaltyResult,
};
use traderview_expense::retaliation_windows::{
    check as check_retaliation, RetaliationCheckInput, RetaliationCheckResult,
};
use traderview_expense::eviction_notices::{
    check as check_eviction_notice, NoticeCheckInput, NoticeCheckResult,
};
use traderview_expense::late_fee_caps::{
    check as check_late_fee, LateFeeCheckInput, LateFeeCheckResult,
};
use traderview_expense::section_280a::{
    compute as compute_section_280a, Section280AInput, Section280AResult,
};
use traderview_expense::section_280a_d2::{
    compute as compute_section_280a_d2, OccupancyPeriod, Section280AD2Report,
};
use traderview_expense::section_469::{
    compute as compute_section_469, Section469Input, Section469Result,
};
use traderview_expense::schedule_e::{
    roll_property, roll_report, ExpenseRow, IncomeKind as SeIncomeKind, IncomeRow, MileageRow,
    PropertyInput, PropertyType as SePropertyType, ScheduleECategory, ScheduleEReport,
};
use uuid::Uuid;

// Row tuples pulled by the Schedule E roll-up and rent-roll queries.
// Aliased here so the SELECT bindings stay readable and clippy stops
// flagging `type_complexity`.
type PropertyRollupRow = (
    Uuid,           // id
    String,         // property_type
    i32,            // fair_rental_days
    i32,            // personal_use_days
    Option<Decimal>, // purchase_price
    Option<Decimal>, // land_value
    Option<NaiveDate>, // placed_in_service_at
    Decimal,        // recovery_period_years
);
type LeaseRentRollRow = (
    Uuid,           // lease id
    String,         // tenant display_name
    String,         // unit_label
    Decimal,        // rent_amount
    i32,            // rent_due_day
    i32,            // grace_days
    NaiveDate,      // starts_on
    Option<NaiveDate>, // ends_on
);

pub fn router() -> Router<AppState> {
    Router::new()
        // properties
        .route("/properties", get(list_properties).post(create_property))
        .route("/properties/:id", patch(update_property).delete(delete_property))
        // tenants
        .route("/tenants", get(list_tenants).post(create_tenant))
        .route("/tenants/:id", patch(update_tenant).delete(delete_tenant))
        // leases
        .route("/properties/:property_id/leases", get(list_leases).post(create_lease))
        .route("/leases/:id", patch(update_lease).delete(delete_lease))
        // income
        .route("/properties/:property_id/income", get(list_income).post(create_income))
        .route("/income/:id", delete(delete_income))
        // expenses
        .route("/properties/:property_id/expenses", get(list_expenses).post(create_expense))
        .route("/expenses/:id", delete(delete_expense))
        // mileage
        .route("/properties/:property_id/mileage", get(list_mileage).post(create_mileage))
        .route("/mileage/:id", delete(delete_mileage))
        // maintenance
        .route("/properties/:property_id/maintenance", get(list_maintenance).post(create_maintenance))
        .route("/maintenance/:id", patch(update_maintenance).delete(delete_maintenance_row))
        // services log (QBI 250-hour tracker)
        .route("/properties/:property_id/services", get(list_services).post(create_service))
        .route("/services/:id", delete(delete_service))
        // categories
        .route("/categories", get(list_schedule_e_categories))
        // reports
        .route("/report/schedule_e", get(schedule_e_report))
        .route("/properties/:property_id/qbi-hours", get(qbi_hours_report))
        .route("/properties/:property_id/rent-roll", get(rent_roll))
        .route("/properties/:property_id/depreciation", get(property_depreciation))
        // state-specific security deposit interest accrual
        .route("/deposit-interest", axum::routing::post(deposit_interest_accrue))
        // §469 passive activity loss limitation calculator
        .route("/section-469", axum::routing::post(section_469_compute))
        // disposition: §1250 unrecaptured + §1231 LTCG + §1031 deferral
        .route("/properties/:property_id/dispose",
            axum::routing::post(property_disposition))
        // §280A vacation home / mixed-use classification
        .route("/properties/:property_id/section-280a",
            axum::routing::post(property_section_280a))
        // §280A(d)(2) related-party rental personal-use classifier
        .route("/section-280a-d2", axum::routing::post(section_280a_d2_route))
        // Cost segregation + §168(k) bonus depreciation accelerator
        .route("/properties/:property_id/cost-segregation",
            axum::routing::post(property_cost_segregation))
        // State late-fee cap + grace-period compliance check
        .route("/late-fee-check", axum::routing::post(late_fee_check_route))
        // State eviction-notice period lookup
        .route("/eviction-notice-check", axum::routing::post(eviction_notice_check_route))
        .route("/entry-notice-check", axum::routing::post(entry_notice_check_route))
        .route("/retaliation-check", axum::routing::post(retaliation_check_route))
        .route("/application-fee-check", axum::routing::post(application_fee_check_route))
        .route("/lockout-penalty-check", axum::routing::post(lockout_penalty_check_route))
        .route("/dv-survivor-lock-change", axum::routing::post(dv_survivor_lock_change_route))
        .route("/dv-termination-check", axum::routing::post(dv_termination_check_route))
        .route("/just-cause-check", axum::routing::post(just_cause_check_route))
        .route("/soi-protection-check", axum::routing::post(soi_protection_check_route))
        .route("/detector-check", axum::routing::post(detector_check_route))
        .route("/lead-disclosure-check", axum::routing::post(lead_disclosure_check_route))
        .route("/foreclosure-tenant-check", axum::routing::post(foreclosure_tenant_check_route))
        .route("/heat-requirements-check", axum::routing::post(heat_requirements_check_route))
        .route("/bedbug-disclosure-check", axum::routing::post(bedbug_disclosure_check_route))
        .route("/mold-disclosure-check", axum::routing::post(mold_disclosure_check_route))
        .route("/radon-disclosure-check", axum::routing::post(radon_disclosure_check_route))
        .route("/sublet-consent-check", axum::routing::post(sublet_consent_check_route))
        .route("/str-regulation-check", axum::routing::post(str_regulation_check_route))
        .route("/pet-fees-check", axum::routing::post(pet_fees_check_route))
        .route("/eviction-sealing-check", axum::routing::post(eviction_sealing_check_route))
        .route("/termination-notice-check", axum::routing::post(termination_notice_check_route))
        .route("/occupancy-check", axum::routing::post(occupancy_check_route))
        .route("/move-in-inspection-check", axum::routing::post(move_in_inspection_check_route))
        .route("/renters-insurance-check", axum::routing::post(renters_insurance_check_route))
        .route("/utility-shutoff-check", axum::routing::post(utility_shutoff_check_route))
        .route("/vehicle-towing-from-rental-property", axum::routing::post(vehicle_towing_from_rental_property_route))
        .route("/adverse-action-check", axum::routing::post(adverse_action_check_route))
        .route("/adverse-possession-claim", axum::routing::post(adverse_possession_claim_route))
        .route("/topa-check", axum::routing::post(tenant_topa_check_route))
        .route("/auto-renewal-check", axum::routing::post(lease_auto_renewal_check_route))
        .route("/lease-translation-check", axum::routing::post(lease_translation_check_route))
        .route("/rent-receipt-check", axum::routing::post(rent_receipt_check_route))
        .route("/repair-deduct-check", axum::routing::post(repair_and_deduct_check_route))
        .route("/cosigner-check", axum::routing::post(cosigner_rules_check_route))
        .route("/mobile-home-park-check", axum::routing::post(mobile_home_park_check_route))
        .route("/submetering-check", axum::routing::post(submetering_check_route))
        .route("/smoke-free-check", axum::routing::post(smoke_free_check_route))
        .route("/tenant-privacy-check", axum::routing::post(tenant_privacy_check_route))
        .route("/drug-eviction-check", axum::routing::post(drug_eviction_check_route))
        .route("/quiet-enjoyment-check", axum::routing::post(quiet_enjoyment_check_route))
        .route("/flood-disclosure-check", axum::routing::post(flood_disclosure_check_route))
        .route("/owner-identification-check", axum::routing::post(owner_identification_check_route))
        .route("/tenant-death-termination-check", axum::routing::post(tenant_death_termination_check_route))
        .route("/late-payment-grace-period-check", axum::routing::post(late_payment_grace_period_check_route))
        .route("/owner-move-in-eviction-check", axum::routing::post(owner_move_in_eviction_check_route))
        .route("/lease-copy-delivery-check", axum::routing::post(lease_copy_delivery_check_route))
        .route("/tenant-organizing-check", axum::routing::post(tenant_organizing_check_route))
        .route("/tenant-relocation-assistance", axum::routing::post(tenant_relocation_assistance_route))
        .route("/fair-chance-housing", axum::routing::post(fair_chance_housing_route))
        .route("/meth-contamination-disclosure", axum::routing::post(meth_contamination_disclosure_route))
        .route("/death-in-unit-disclosure", axum::routing::post(death_in_unit_disclosure_route))
        .route("/rent-payment-method", axum::routing::post(rent_payment_method_route))
        .route("/window-guard-requirements", axum::routing::post(window_guard_requirements_route))
        .route("/rent-increase-notice-period", axum::routing::post(rent_increase_notice_period_route))
        .route("/demolition-tenant-notice", axum::routing::post(demolition_tenant_notice_route))
        .route("/eviction-diversion-program", axum::routing::post(eviction_diversion_program_route))
        .route("/immigration-status-protection", axum::routing::post(immigration_status_protection_route))
        .route("/prevailing-party-attorney-fees", axum::routing::post(prevailing_party_attorney_fees_route))
        .route("/abandoned-property-handling", axum::routing::post(abandoned_property_handling_route))
        .route("/right-to-counsel-eviction", axum::routing::post(right_to_counsel_eviction_route))
        .route("/tenant-cannabis-use-protection", axum::routing::post(tenant_cannabis_use_protection_route))
        .route("/snow-removal-responsibility", axum::routing::post(snow_removal_responsibility_route))
        .route("/security-camera-disclosure", axum::routing::post(security_camera_disclosure_route))
        .route("/carpet-replacement-useful-life", axum::routing::post(carpet_replacement_useful_life_route))
        .route("/pre-move-out-inspection", axum::routing::post(pre_move_out_inspection_route))
        .route("/credit-check-authorization", axum::routing::post(credit_check_authorization_route))
        .route("/winter-eviction-protections", axum::routing::post(winter_eviction_protections_route))
        .route("/landlord-identification-disclosure", axum::routing::post(landlord_identification_disclosure_route))
        .route("/reasonable-accommodation-modification", axum::routing::post(reasonable_accommodation_modification_route))
        .route("/damage-deduction-itemization", axum::routing::post(damage_deduction_itemization_route))
        .route("/cooling-requirements", axum::routing::post(cooling_requirements_route))
        .route("/duty-to-mitigate-damages", axum::routing::post(duty_to_mitigate_damages_route))
        .route("/pesticide-application-notice", axum::routing::post(pesticide_application_notice_route))
        .route("/condominium-conversion-protection", axum::routing::post(condominium_conversion_protection_route))
        .route("/otard-antenna-installation", axum::routing::post(otard_antenna_installation_route))
        .route("/religious-display-doorpost", axum::routing::post(religious_display_doorpost_route))
        .route("/asbestos-disclosure", axum::routing::post(asbestos_disclosure_route))
        .route("/firearms-in-rental-unit", axum::routing::post(firearms_in_rental_unit_route))
        .route("/lock-change-between-tenancies", axum::routing::post(lock_change_between_tenancies_route))
        .route("/landlord-lien-prohibition", axum::routing::post(landlord_lien_prohibition_route))
        .route("/military-ordnance-disclosure", axum::routing::post(military_ordnance_disclosure_route))
        .route("/sex-offender-database-notice", axum::routing::post(sex_offender_database_notice_route))
        .route("/mid-tenancy-ownership-change", axum::routing::post(mid_tenancy_ownership_change_route))
        .route("/mid-tenancy-term-modification", axum::routing::post(mid_tenancy_term_modification_route))
        .route("/tenant-solar-installation", axum::routing::post(tenant_solar_installation_route))
        .route("/flag-display-right", axum::routing::post(flag_display_right_route))
        .route("/written-lease-requirement", axum::routing::post(written_lease_requirement_route))
        .route("/holdover-tenant-damages", axum::routing::post(holdover_tenant_damages_route))
        .route("/lease-assignment-consent", axum::routing::post(lease_assignment_consent_route))
        .route("/lease-cure-period", axum::routing::post(lease_cure_period_route))
        .route("/portable-tenant-screening-report", axum::routing::post(portable_tenant_screening_report_route))
        .route("/hoa-rental-restriction", axum::routing::post(hoa_rental_restriction_route))
        .route("/rent-acceleration-enforceability", axum::routing::post(rent_acceleration_enforceability_route))
        .route("/tenant-in-foreclosure-protection", axum::routing::post(tenant_in_foreclosure_protection_route))
        .route("/security-deposit-bank-disclosure", axum::routing::post(security_deposit_bank_disclosure_route))
        .route("/landlord-harassment", axum::routing::post(landlord_harassment_route))
        .route("/landlord-possession-delivery", axum::routing::post(landlord_possession_delivery_route))
        .route("/lease-waiver-enforceability", axum::routing::post(lease_waiver_enforceability_route))
        .route("/landlord-retaliation-damages", axum::routing::post(landlord_retaliation_damages_route))
        .route("/last-month-rent-offset", axum::routing::post(last_month_rent_offset_route))
        .route("/emotional-support-animal-documentation", axum::routing::post(emotional_support_animal_documentation_route))
        .route("/lease-nondisparagement-prohibition", axum::routing::post(lease_nondisparagement_prohibition_route))
        .route("/plain-language-lease-check", axum::routing::post(plain_language_lease_check_route))
        .route("/roommate-authorization-check", axum::routing::post(roommate_authorization_check_route))
        .route("/ev-charger-installation-check", axum::routing::post(ev_charger_installation_check_route))
        .route("/advance-rent-limit-check", axum::routing::post(advance_rent_limit_check_route))
        .route("/fire-sprinkler-disclosure-check", axum::routing::post(fire_sprinkler_disclosure_check_route))
        .route("/bedbug-extermination-cost-check", axum::routing::post(bedbug_extermination_cost_check_route))
        .route("/crime-victim-termination-check", axum::routing::post(crime_victim_termination_check_route))
        .route("/lease-succession-check", axum::routing::post(lease_succession_check_route))
        .route("/rent-credit-reporting-check", axum::routing::post(rent_credit_reporting_check_route))
        .route("/rent-escrow-check", axum::routing::post(rent_escrow_check_route))
        .route("/right-to-dry-check", axum::routing::post(right_to_dry_check_route))
        .route("/abandonment-check", axum::routing::post(abandonment_check_route))
        .route("/service-animal-check", axum::routing::post(service_animal_check_route))
        .route("/senior-disabled-check", axum::routing::post(senior_disabled_check_route))
        // 1099-NEC contractor $600 threshold tracker
        .route("/1099-nec-report", axum::routing::post(contractor_1099_route))
        // State deposit-return window compliance check
        .route("/deposit-return-check", axum::routing::post(deposit_return_check_route))
        // State + federal lease disclosure requirements
        .route("/lease-disclosures-required", axum::routing::post(lease_disclosures_route))
        // State rent control / rent-increase compliance check
        .route("/rent-increase-check", axum::routing::post(rent_increase_check_route))
        // State habitability remedies available to tenants
        .route("/habitability-remedies", axum::routing::post(habitability_remedies_route))
        // State security deposit cap compliance check
        .route("/security-deposit-cap-check", axum::routing::post(security_deposit_cap_route))
        // Federal SCRA + state military lease termination check
        .route("/military-termination-check", axum::routing::post(military_termination_route))
}

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

async fn ensure_property_owner(s: &AppState, user_id: Uuid, pid: Uuid) -> Result<(), ApiError> {
    let row: Option<(Uuid,)> = sqlx::query_as(
        "SELECT user_id FROM rental_properties WHERE id = $1",
    )
    .bind(pid)
    .fetch_optional(&s.pool)
    .await?;
    match row {
        Some((owner,)) if owner == user_id => Ok(()),
        Some(_) => Err(ApiError::Forbidden),
        None => Err(ApiError::NotFound),
    }
}

async fn ensure_lease_owner(s: &AppState, user_id: Uuid, lease_id: Uuid) -> Result<Uuid, ApiError> {
    let row: Option<(Uuid, Uuid)> = sqlx::query_as(
        "SELECT p.user_id, l.property_id
           FROM rental_leases l
           JOIN rental_properties p ON p.id = l.property_id
          WHERE l.id = $1",
    )
    .bind(lease_id)
    .fetch_optional(&s.pool)
    .await?;
    match row {
        Some((owner, pid)) if owner == user_id => Ok(pid),
        Some(_) => Err(ApiError::Forbidden),
        None => Err(ApiError::NotFound),
    }
}

fn parse_property_type(s: &str) -> Result<&'static str, ApiError> {
    Ok(match s {
        "single_family" => "single_family",
        "multi_family" => "multi_family",
        "vacation_short_term" => "vacation_short_term",
        "commercial" => "commercial",
        "land" => "land",
        "royalties" => "royalties",
        "self_rental" => "self_rental",
        "other" => "other",
        _ => return Err(ApiError::BadRequest(format!("invalid property_type: {s}"))),
    })
}

fn parse_lease_status(s: &str) -> Result<&'static str, ApiError> {
    Ok(match s {
        "draft" => "draft",
        "active" => "active",
        "expired" => "expired",
        "terminated_early" => "terminated_early",
        _ => return Err(ApiError::BadRequest(format!("invalid lease status: {s}"))),
    })
}

fn parse_income_kind(s: &str) -> Result<&'static str, ApiError> {
    Ok(match s {
        "rent" => "rent",
        "late_fee" => "late_fee",
        "deposit_forfeit" => "deposit_forfeit",
        "reimbursement" => "reimbursement",
        "royalty" => "royalty",
        "parking" => "parking",
        "laundry" => "laundry",
        "storage" => "storage",
        "other" => "other",
        _ => return Err(ApiError::BadRequest(format!("invalid income kind: {s}"))),
    })
}

fn parse_maint_status(s: &str) -> Result<&'static str, ApiError> {
    Ok(match s {
        "open" => "open",
        "in_progress" => "in_progress",
        "done" => "done",
        "cancelled" => "cancelled",
        _ => return Err(ApiError::BadRequest(format!("invalid maintenance status: {s}"))),
    })
}

fn parse_maint_priority(s: &str) -> Result<&'static str, ApiError> {
    Ok(match s {
        "low" => "low",
        "normal" => "normal",
        "high" => "high",
        "emergency" => "emergency",
        _ => return Err(ApiError::BadRequest(format!("invalid maintenance priority: {s}"))),
    })
}

fn property_type_enum(s: &str) -> SePropertyType {
    match s {
        "single_family"       => SePropertyType::SingleFamily,
        "multi_family"        => SePropertyType::MultiFamily,
        "vacation_short_term" => SePropertyType::VacationShortTerm,
        "commercial"          => SePropertyType::Commercial,
        "land"                => SePropertyType::Land,
        "royalties"           => SePropertyType::Royalties,
        "self_rental"         => SePropertyType::SelfRental,
        _                     => SePropertyType::Other,
    }
}

fn income_kind_enum(s: &str) -> SeIncomeKind {
    match s {
        "rent"     => SeIncomeKind::Rent,
        "royalty"  => SeIncomeKind::Royalty,
        _          => SeIncomeKind::Other,
    }
}

// ---------------------------------------------------------------------------
// properties
// ---------------------------------------------------------------------------

#[derive(Serialize, sqlx::FromRow)]
struct Property {
    id: Uuid,
    user_id: Uuid,
    nickname: String,
    property_type: String,
    status: String,
    address_line1: String,
    address_line2: String,
    city: String,
    state_region: String,
    postal_code: String,
    country: String,
    units: i32,
    purchased_at: Option<NaiveDate>,
    purchase_price: Option<Decimal>,
    land_value: Option<Decimal>,
    placed_in_service_at: Option<NaiveDate>,
    recovery_period_years: Decimal,
    fair_rental_days: i32,
    personal_use_days: i32,
    qjv_election: bool,
    qbi_safe_harbor: bool,
    sold_at: Option<NaiveDate>,
    sold_price: Option<Decimal>,
    notes: String,
    created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
struct PropertyInputDto {
    nickname: String,
    property_type: String,
    status: Option<String>,
    address_line1: Option<String>,
    address_line2: Option<String>,
    city: Option<String>,
    state_region: Option<String>,
    postal_code: Option<String>,
    country: Option<String>,
    units: Option<i32>,
    purchased_at: Option<NaiveDate>,
    purchase_price: Option<Decimal>,
    land_value: Option<Decimal>,
    placed_in_service_at: Option<NaiveDate>,
    recovery_period_years: Option<Decimal>,
    fair_rental_days: Option<i32>,
    personal_use_days: Option<i32>,
    qjv_election: Option<bool>,
    qbi_safe_harbor: Option<bool>,
    sold_at: Option<NaiveDate>,
    sold_price: Option<Decimal>,
    notes: Option<String>,
}

const PROPERTY_COLS: &str = "id, user_id, nickname, property_type::text, status::text,
    address_line1, address_line2, city, state_region, postal_code, country,
    units, purchased_at, purchase_price, land_value, placed_in_service_at,
    recovery_period_years, fair_rental_days, personal_use_days, qjv_election,
    qbi_safe_harbor, sold_at, sold_price, notes, created_at";

async fn list_properties(
    State(s): State<AppState>,
    u: AuthUser,
) -> Result<Json<Vec<Property>>, ApiError> {
    Ok(Json(
        sqlx::query_as(&format!(
            "SELECT {PROPERTY_COLS} FROM rental_properties
              WHERE user_id = $1 ORDER BY status ASC, nickname ASC"
        ))
        .bind(u.id)
        .fetch_all(&s.pool)
        .await?,
    ))
}

async fn create_property(
    State(s): State<AppState>,
    u: AuthUser,
    Json(b): Json<PropertyInputDto>,
) -> Result<Json<Property>, ApiError> {
    if b.nickname.trim().is_empty() {
        return Err(ApiError::BadRequest("nickname required".into()));
    }
    let pt = parse_property_type(&b.property_type)?;
    let status = b.status.as_deref().unwrap_or("active");
    if !matches!(status, "active" | "vacant" | "sold" | "archived") {
        return Err(ApiError::BadRequest(format!("invalid status: {status}")));
    }
    let row = sqlx::query_as(&format!(
        "INSERT INTO rental_properties
           (user_id, nickname, property_type, status, address_line1, address_line2,
            city, state_region, postal_code, country, units, purchased_at,
            purchase_price, land_value, placed_in_service_at, recovery_period_years,
            fair_rental_days, personal_use_days, qjv_election, qbi_safe_harbor,
            sold_at, sold_price, notes)
         VALUES ($1, $2, $3::property_type_t, $4::property_status_t, $5, $6, $7, $8,
                 $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21,
                 $22, $23)
         RETURNING {PROPERTY_COLS}"
    ))
    .bind(u.id)
    .bind(&b.nickname)
    .bind(pt)
    .bind(status)
    .bind(b.address_line1.unwrap_or_default())
    .bind(b.address_line2.unwrap_or_default())
    .bind(b.city.unwrap_or_default())
    .bind(b.state_region.unwrap_or_default())
    .bind(b.postal_code.unwrap_or_default())
    .bind(b.country.unwrap_or_else(|| "US".into()))
    .bind(b.units.unwrap_or(1))
    .bind(b.purchased_at)
    .bind(b.purchase_price)
    .bind(b.land_value)
    .bind(b.placed_in_service_at)
    .bind(b.recovery_period_years.unwrap_or_else(|| Decimal::from_str("27.5").unwrap()))
    .bind(b.fair_rental_days.unwrap_or(0))
    .bind(b.personal_use_days.unwrap_or(0))
    .bind(b.qjv_election.unwrap_or(false))
    .bind(b.qbi_safe_harbor.unwrap_or(false))
    .bind(b.sold_at)
    .bind(b.sold_price)
    .bind(b.notes.unwrap_or_default())
    .fetch_one(&s.pool)
    .await?;
    Ok(Json(row))
}

async fn update_property(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
    Json(b): Json<PropertyInputDto>,
) -> Result<Json<Property>, ApiError> {
    ensure_property_owner(&s, u.id, id).await?;
    let pt = parse_property_type(&b.property_type)?;
    let status = b.status.as_deref().unwrap_or("active");
    let row = sqlx::query_as(&format!(
        "UPDATE rental_properties SET
            nickname = $3,
            property_type = $4::property_type_t,
            status = $5::property_status_t,
            address_line1 = $6,
            address_line2 = $7,
            city = $8,
            state_region = $9,
            postal_code = $10,
            country = $11,
            units = $12,
            purchased_at = $13,
            purchase_price = $14,
            land_value = $15,
            placed_in_service_at = $16,
            recovery_period_years = $17,
            fair_rental_days = $18,
            personal_use_days = $19,
            qjv_election = $20,
            qbi_safe_harbor = $21,
            sold_at = $22,
            sold_price = $23,
            notes = $24
          WHERE id = $1 AND user_id = $2
          RETURNING {PROPERTY_COLS}"
    ))
    .bind(id)
    .bind(u.id)
    .bind(&b.nickname)
    .bind(pt)
    .bind(status)
    .bind(b.address_line1.unwrap_or_default())
    .bind(b.address_line2.unwrap_or_default())
    .bind(b.city.unwrap_or_default())
    .bind(b.state_region.unwrap_or_default())
    .bind(b.postal_code.unwrap_or_default())
    .bind(b.country.unwrap_or_else(|| "US".into()))
    .bind(b.units.unwrap_or(1))
    .bind(b.purchased_at)
    .bind(b.purchase_price)
    .bind(b.land_value)
    .bind(b.placed_in_service_at)
    .bind(b.recovery_period_years.unwrap_or_else(|| Decimal::from_str("27.5").unwrap()))
    .bind(b.fair_rental_days.unwrap_or(0))
    .bind(b.personal_use_days.unwrap_or(0))
    .bind(b.qjv_election.unwrap_or(false))
    .bind(b.qbi_safe_harbor.unwrap_or(false))
    .bind(b.sold_at)
    .bind(b.sold_price)
    .bind(b.notes.unwrap_or_default())
    .fetch_one(&s.pool)
    .await?;
    Ok(Json(row))
}

async fn delete_property(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let n = sqlx::query("DELETE FROM rental_properties WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(u.id)
        .execute(&s.pool)
        .await?
        .rows_affected();
    if n == 0 {
        return Err(ApiError::NotFound);
    }
    Ok(Json(serde_json::json!({"deleted": true})))
}

// ---------------------------------------------------------------------------
// tenants
// ---------------------------------------------------------------------------

#[derive(Serialize, sqlx::FromRow)]
struct Tenant {
    id: Uuid,
    user_id: Uuid,
    display_name: String,
    email: String,
    phone: String,
    notes: String,
    created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
struct TenantInput {
    display_name: String,
    email: Option<String>,
    phone: Option<String>,
    notes: Option<String>,
}

async fn list_tenants(
    State(s): State<AppState>,
    u: AuthUser,
) -> Result<Json<Vec<Tenant>>, ApiError> {
    Ok(Json(
        sqlx::query_as(
            "SELECT id, user_id, display_name, email, phone, notes, created_at
               FROM rental_tenants WHERE user_id = $1 ORDER BY display_name",
        )
        .bind(u.id)
        .fetch_all(&s.pool)
        .await?,
    ))
}

async fn create_tenant(
    State(s): State<AppState>,
    u: AuthUser,
    Json(b): Json<TenantInput>,
) -> Result<Json<Tenant>, ApiError> {
    if b.display_name.trim().is_empty() {
        return Err(ApiError::BadRequest("display_name required".into()));
    }
    Ok(Json(
        sqlx::query_as(
            "INSERT INTO rental_tenants (user_id, display_name, email, phone, notes)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING id, user_id, display_name, email, phone, notes, created_at",
        )
        .bind(u.id)
        .bind(&b.display_name)
        .bind(b.email.unwrap_or_default())
        .bind(b.phone.unwrap_or_default())
        .bind(b.notes.unwrap_or_default())
        .fetch_one(&s.pool)
        .await?,
    ))
}

async fn update_tenant(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
    Json(b): Json<TenantInput>,
) -> Result<Json<Tenant>, ApiError> {
    Ok(Json(
        sqlx::query_as(
            "UPDATE rental_tenants SET display_name = $3, email = $4, phone = $5, notes = $6
              WHERE id = $1 AND user_id = $2
              RETURNING id, user_id, display_name, email, phone, notes, created_at",
        )
        .bind(id)
        .bind(u.id)
        .bind(&b.display_name)
        .bind(b.email.unwrap_or_default())
        .bind(b.phone.unwrap_or_default())
        .bind(b.notes.unwrap_or_default())
        .fetch_optional(&s.pool)
        .await?
        .ok_or(ApiError::NotFound)?,
    ))
}

async fn delete_tenant(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let n = sqlx::query("DELETE FROM rental_tenants WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(u.id)
        .execute(&s.pool)
        .await?
        .rows_affected();
    if n == 0 { return Err(ApiError::NotFound); }
    Ok(Json(serde_json::json!({"deleted": true})))
}

// ---------------------------------------------------------------------------
// leases
// ---------------------------------------------------------------------------

#[derive(Serialize, sqlx::FromRow)]
struct Lease {
    id: Uuid,
    property_id: Uuid,
    tenant_id: Uuid,
    unit_label: String,
    status: String,
    starts_on: NaiveDate,
    ends_on: Option<NaiveDate>,
    rent_amount: Decimal,
    rent_frequency: String,
    rent_due_day: i32,
    grace_days: i32,
    late_fee_fixed: Decimal,
    late_fee_pct: Decimal,
    security_deposit: Decimal,
    deposit_held_by: String,
    pet_deposit: Decimal,
    notes: String,
    created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
struct LeaseInput {
    tenant_id: Uuid,
    unit_label: Option<String>,
    status: Option<String>,
    starts_on: NaiveDate,
    ends_on: Option<NaiveDate>,
    rent_amount: Decimal,
    rent_frequency: Option<String>,
    rent_due_day: Option<i32>,
    grace_days: Option<i32>,
    late_fee_fixed: Option<Decimal>,
    late_fee_pct: Option<Decimal>,
    security_deposit: Option<Decimal>,
    deposit_held_by: Option<String>,
    pet_deposit: Option<Decimal>,
    notes: Option<String>,
}

const LEASE_COLS: &str = "id, property_id, tenant_id, unit_label, status::text,
    starts_on, ends_on, rent_amount, rent_frequency, rent_due_day, grace_days,
    late_fee_fixed, late_fee_pct, security_deposit, deposit_held_by, pet_deposit,
    notes, created_at";

async fn list_leases(
    State(s): State<AppState>,
    u: AuthUser,
    Path(property_id): Path<Uuid>,
) -> Result<Json<Vec<Lease>>, ApiError> {
    ensure_property_owner(&s, u.id, property_id).await?;
    Ok(Json(
        sqlx::query_as(&format!(
            "SELECT {LEASE_COLS} FROM rental_leases
              WHERE property_id = $1 ORDER BY starts_on DESC"
        ))
        .bind(property_id)
        .fetch_all(&s.pool)
        .await?,
    ))
}

async fn create_lease(
    State(s): State<AppState>,
    u: AuthUser,
    Path(property_id): Path<Uuid>,
    Json(b): Json<LeaseInput>,
) -> Result<Json<Lease>, ApiError> {
    ensure_property_owner(&s, u.id, property_id).await?;
    if let Some(end) = b.ends_on {
        if end < b.starts_on {
            return Err(ApiError::BadRequest("ends_on must be >= starts_on".into()));
        }
    }
    let status = parse_lease_status(b.status.as_deref().unwrap_or("active"))?;
    let due_day = b.rent_due_day.unwrap_or(1);
    if !(1..=31).contains(&due_day) {
        return Err(ApiError::BadRequest("rent_due_day must be 1..31".into()));
    }
    Ok(Json(
        sqlx::query_as(&format!(
            "INSERT INTO rental_leases
               (property_id, tenant_id, unit_label, status, starts_on, ends_on,
                rent_amount, rent_frequency, rent_due_day, grace_days,
                late_fee_fixed, late_fee_pct, security_deposit, deposit_held_by,
                pet_deposit, notes)
             VALUES ($1, $2, $3, $4::lease_status_t, $5, $6, $7, $8, $9, $10,
                     $11, $12, $13, $14, $15, $16)
             RETURNING {LEASE_COLS}"
        ))
        .bind(property_id)
        .bind(b.tenant_id)
        .bind(b.unit_label.unwrap_or_default())
        .bind(status)
        .bind(b.starts_on)
        .bind(b.ends_on)
        .bind(b.rent_amount)
        .bind(b.rent_frequency.unwrap_or_else(|| "monthly".into()))
        .bind(due_day)
        .bind(b.grace_days.unwrap_or(5))
        .bind(b.late_fee_fixed.unwrap_or(Decimal::ZERO))
        .bind(b.late_fee_pct.unwrap_or(Decimal::ZERO))
        .bind(b.security_deposit.unwrap_or(Decimal::ZERO))
        .bind(b.deposit_held_by.unwrap_or_default())
        .bind(b.pet_deposit.unwrap_or(Decimal::ZERO))
        .bind(b.notes.unwrap_or_default())
        .fetch_one(&s.pool)
        .await?,
    ))
}

async fn update_lease(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
    Json(b): Json<LeaseInput>,
) -> Result<Json<Lease>, ApiError> {
    ensure_lease_owner(&s, u.id, id).await?;
    let status = parse_lease_status(b.status.as_deref().unwrap_or("active"))?;
    Ok(Json(
        sqlx::query_as(&format!(
            "UPDATE rental_leases SET
                tenant_id = $2,
                unit_label = $3,
                status = $4::lease_status_t,
                starts_on = $5,
                ends_on = $6,
                rent_amount = $7,
                rent_frequency = $8,
                rent_due_day = $9,
                grace_days = $10,
                late_fee_fixed = $11,
                late_fee_pct = $12,
                security_deposit = $13,
                deposit_held_by = $14,
                pet_deposit = $15,
                notes = $16
              WHERE id = $1
              RETURNING {LEASE_COLS}"
        ))
        .bind(id)
        .bind(b.tenant_id)
        .bind(b.unit_label.unwrap_or_default())
        .bind(status)
        .bind(b.starts_on)
        .bind(b.ends_on)
        .bind(b.rent_amount)
        .bind(b.rent_frequency.unwrap_or_else(|| "monthly".into()))
        .bind(b.rent_due_day.unwrap_or(1))
        .bind(b.grace_days.unwrap_or(5))
        .bind(b.late_fee_fixed.unwrap_or(Decimal::ZERO))
        .bind(b.late_fee_pct.unwrap_or(Decimal::ZERO))
        .bind(b.security_deposit.unwrap_or(Decimal::ZERO))
        .bind(b.deposit_held_by.unwrap_or_default())
        .bind(b.pet_deposit.unwrap_or(Decimal::ZERO))
        .bind(b.notes.unwrap_or_default())
        .fetch_one(&s.pool)
        .await?,
    ))
}

async fn delete_lease(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    ensure_lease_owner(&s, u.id, id).await?;
    sqlx::query("DELETE FROM rental_leases WHERE id = $1")
        .bind(id)
        .execute(&s.pool)
        .await?;
    Ok(Json(serde_json::json!({"deleted": true})))
}

// ---------------------------------------------------------------------------
// income
// ---------------------------------------------------------------------------

#[derive(Serialize, sqlx::FromRow)]
struct Income {
    id: Uuid,
    property_id: Uuid,
    lease_id: Option<Uuid>,
    posted_at: DateTime<Utc>,
    period_start: Option<NaiveDate>,
    period_end: Option<NaiveDate>,
    amount: Decimal,
    currency: String,
    kind: String,
    payer_raw: String,
    method: String,
    transaction_id: Option<Uuid>,
    notes: String,
    created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
struct IncomeInput {
    lease_id: Option<Uuid>,
    posted_at: DateTime<Utc>,
    period_start: Option<NaiveDate>,
    period_end: Option<NaiveDate>,
    amount: Decimal,
    currency: Option<String>,
    kind: Option<String>,
    payer_raw: Option<String>,
    method: Option<String>,
    transaction_id: Option<Uuid>,
    notes: Option<String>,
}

const INCOME_COLS: &str = "id, property_id, lease_id, posted_at, period_start,
    period_end, amount, currency, kind::text, payer_raw, method, transaction_id,
    notes, created_at";

#[derive(Deserialize)]
struct IncomeQuery {
    from: Option<NaiveDate>,
    to: Option<NaiveDate>,
    kind: Option<String>,
}

async fn list_income(
    State(s): State<AppState>,
    u: AuthUser,
    Path(property_id): Path<Uuid>,
    Query(q): Query<IncomeQuery>,
) -> Result<Json<Vec<Income>>, ApiError> {
    ensure_property_owner(&s, u.id, property_id).await?;
    let from = q.from.unwrap_or(NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());
    let to = q.to.unwrap_or(NaiveDate::from_ymd_opt(9999, 12, 31).unwrap());
    let kind = q.kind.unwrap_or_default();
    let rows = sqlx::query_as(&format!(
        "SELECT {INCOME_COLS} FROM rental_income
          WHERE property_id = $1
            AND posted_at >= $2::date
            AND posted_at <  ($3::date + INTERVAL '1 day')
            AND ($4 = '' OR kind::text = $4)
          ORDER BY posted_at DESC"
    ))
    .bind(property_id)
    .bind(from)
    .bind(to)
    .bind(kind)
    .fetch_all(&s.pool)
    .await?;
    Ok(Json(rows))
}

async fn create_income(
    State(s): State<AppState>,
    u: AuthUser,
    Path(property_id): Path<Uuid>,
    Json(b): Json<IncomeInput>,
) -> Result<Json<Income>, ApiError> {
    ensure_property_owner(&s, u.id, property_id).await?;
    let kind = parse_income_kind(b.kind.as_deref().unwrap_or("rent"))?;
    Ok(Json(
        sqlx::query_as(&format!(
            "INSERT INTO rental_income
               (property_id, lease_id, posted_at, period_start, period_end, amount,
                currency, kind, payer_raw, method, transaction_id, notes)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8::rental_income_kind_t,
                     $9, $10, $11, $12)
             RETURNING {INCOME_COLS}"
        ))
        .bind(property_id)
        .bind(b.lease_id)
        .bind(b.posted_at)
        .bind(b.period_start)
        .bind(b.period_end)
        .bind(b.amount)
        .bind(b.currency.unwrap_or_else(|| "USD".into()))
        .bind(kind)
        .bind(b.payer_raw.unwrap_or_default())
        .bind(b.method.unwrap_or_default())
        .bind(b.transaction_id)
        .bind(b.notes.unwrap_or_default())
        .fetch_one(&s.pool)
        .await?,
    ))
}

async fn delete_income(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let n = sqlx::query(
        "DELETE FROM rental_income
          WHERE id = $1
            AND property_id IN (SELECT id FROM rental_properties WHERE user_id = $2)",
    )
    .bind(id)
    .bind(u.id)
    .execute(&s.pool)
    .await?
    .rows_affected();
    if n == 0 { return Err(ApiError::NotFound); }
    Ok(Json(serde_json::json!({"deleted": true})))
}

// ---------------------------------------------------------------------------
// expenses
// ---------------------------------------------------------------------------

#[derive(Serialize, sqlx::FromRow)]
struct Expense {
    id: Uuid,
    property_id: Uuid,
    posted_at: DateTime<Utc>,
    amount: Decimal,
    currency: String,
    category_code: String,
    vendor_raw: String,
    vendor_normalized: String,
    description: String,
    is_capitalized: bool,
    capital_useful_life: Option<i32>,
    method: String,
    transaction_id: Option<Uuid>,
    notes: String,
    created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
struct ExpenseInput {
    posted_at: DateTime<Utc>,
    amount: Decimal,
    currency: Option<String>,
    category_code: String,
    vendor_raw: Option<String>,
    description: Option<String>,
    is_capitalized: Option<bool>,
    capital_useful_life: Option<i32>,
    method: Option<String>,
    transaction_id: Option<Uuid>,
    notes: Option<String>,
}

#[derive(Deserialize)]
struct ExpenseQuery {
    from: Option<NaiveDate>,
    to: Option<NaiveDate>,
    category: Option<String>,
    capitalized: Option<bool>,
}

fn normalize_vendor(raw: &str) -> String {
    raw.trim().to_uppercase()
}

async fn list_expenses(
    State(s): State<AppState>,
    u: AuthUser,
    Path(property_id): Path<Uuid>,
    Query(q): Query<ExpenseQuery>,
) -> Result<Json<Vec<Expense>>, ApiError> {
    ensure_property_owner(&s, u.id, property_id).await?;
    let from = q.from.unwrap_or(NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());
    let to = q.to.unwrap_or(NaiveDate::from_ymd_opt(9999, 12, 31).unwrap());
    let cat = q.category.unwrap_or_default();
    let capitalized = q.capitalized; // None = either
    Ok(Json(
        sqlx::query_as(
            "SELECT id, property_id, posted_at, amount, currency, category_code,
                    vendor_raw, vendor_normalized, description, is_capitalized,
                    capital_useful_life, method, transaction_id, notes, created_at
               FROM rental_expenses
              WHERE property_id = $1
                AND posted_at >= $2::date
                AND posted_at <  ($3::date + INTERVAL '1 day')
                AND ($4 = '' OR category_code = $4)
                AND ($5::boolean IS NULL OR is_capitalized = $5)
              ORDER BY posted_at DESC",
        )
        .bind(property_id)
        .bind(from)
        .bind(to)
        .bind(cat)
        .bind(capitalized)
        .fetch_all(&s.pool)
        .await?,
    ))
}

async fn create_expense(
    State(s): State<AppState>,
    u: AuthUser,
    Path(property_id): Path<Uuid>,
    Json(b): Json<ExpenseInput>,
) -> Result<Json<Expense>, ApiError> {
    ensure_property_owner(&s, u.id, property_id).await?;
    let vendor = b.vendor_raw.unwrap_or_default();
    let normalized = normalize_vendor(&vendor);
    Ok(Json(
        sqlx::query_as(
            "INSERT INTO rental_expenses
               (property_id, posted_at, amount, currency, category_code, vendor_raw,
                vendor_normalized, description, is_capitalized, capital_useful_life,
                method, transaction_id, notes)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
             RETURNING id, property_id, posted_at, amount, currency, category_code,
                       vendor_raw, vendor_normalized, description, is_capitalized,
                       capital_useful_life, method, transaction_id, notes, created_at",
        )
        .bind(property_id)
        .bind(b.posted_at)
        .bind(b.amount)
        .bind(b.currency.unwrap_or_else(|| "USD".into()))
        .bind(&b.category_code)
        .bind(&vendor)
        .bind(normalized)
        .bind(b.description.unwrap_or_default())
        .bind(b.is_capitalized.unwrap_or(false))
        .bind(b.capital_useful_life)
        .bind(b.method.unwrap_or_default())
        .bind(b.transaction_id)
        .bind(b.notes.unwrap_or_default())
        .fetch_one(&s.pool)
        .await?,
    ))
}

async fn delete_expense(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let n = sqlx::query(
        "DELETE FROM rental_expenses
          WHERE id = $1
            AND property_id IN (SELECT id FROM rental_properties WHERE user_id = $2)",
    )
    .bind(id)
    .bind(u.id)
    .execute(&s.pool)
    .await?
    .rows_affected();
    if n == 0 { return Err(ApiError::NotFound); }
    Ok(Json(serde_json::json!({"deleted": true})))
}

// ---------------------------------------------------------------------------
// mileage
// ---------------------------------------------------------------------------

#[derive(Serialize, sqlx::FromRow)]
struct Mileage {
    id: Uuid,
    property_id: Uuid,
    drove_on: NaiveDate,
    miles: Decimal,
    rate_per_mile: Decimal,
    purpose: String,
    odometer_start: Option<Decimal>,
    odometer_end: Option<Decimal>,
    notes: String,
    created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
struct MileageInput {
    drove_on: NaiveDate,
    miles: Decimal,
    rate_per_mile: Decimal,
    purpose: Option<String>,
    odometer_start: Option<Decimal>,
    odometer_end: Option<Decimal>,
    notes: Option<String>,
}

async fn list_mileage(
    State(s): State<AppState>,
    u: AuthUser,
    Path(property_id): Path<Uuid>,
) -> Result<Json<Vec<Mileage>>, ApiError> {
    ensure_property_owner(&s, u.id, property_id).await?;
    Ok(Json(
        sqlx::query_as(
            "SELECT id, property_id, drove_on, miles, rate_per_mile, purpose,
                    odometer_start, odometer_end, notes, created_at
               FROM rental_mileage
              WHERE property_id = $1
              ORDER BY drove_on DESC",
        )
        .bind(property_id)
        .fetch_all(&s.pool)
        .await?,
    ))
}

async fn create_mileage(
    State(s): State<AppState>,
    u: AuthUser,
    Path(property_id): Path<Uuid>,
    Json(b): Json<MileageInput>,
) -> Result<Json<Mileage>, ApiError> {
    ensure_property_owner(&s, u.id, property_id).await?;
    if b.miles <= Decimal::ZERO {
        return Err(ApiError::BadRequest("miles must be > 0".into()));
    }
    Ok(Json(
        sqlx::query_as(
            "INSERT INTO rental_mileage
               (property_id, drove_on, miles, rate_per_mile, purpose,
                odometer_start, odometer_end, notes)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
             RETURNING id, property_id, drove_on, miles, rate_per_mile, purpose,
                       odometer_start, odometer_end, notes, created_at",
        )
        .bind(property_id)
        .bind(b.drove_on)
        .bind(b.miles)
        .bind(b.rate_per_mile)
        .bind(b.purpose.unwrap_or_default())
        .bind(b.odometer_start)
        .bind(b.odometer_end)
        .bind(b.notes.unwrap_or_default())
        .fetch_one(&s.pool)
        .await?,
    ))
}

async fn delete_mileage(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let n = sqlx::query(
        "DELETE FROM rental_mileage
          WHERE id = $1
            AND property_id IN (SELECT id FROM rental_properties WHERE user_id = $2)",
    )
    .bind(id)
    .bind(u.id)
    .execute(&s.pool)
    .await?
    .rows_affected();
    if n == 0 { return Err(ApiError::NotFound); }
    Ok(Json(serde_json::json!({"deleted": true})))
}

// ---------------------------------------------------------------------------
// maintenance
// ---------------------------------------------------------------------------

#[derive(Serialize, sqlx::FromRow)]
struct Maintenance {
    id: Uuid,
    property_id: Uuid,
    lease_id: Option<Uuid>,
    title: String,
    description: String,
    status: String,
    priority: String,
    reported_at: DateTime<Utc>,
    started_at: Option<DateTime<Utc>>,
    completed_at: Option<DateTime<Utc>>,
    vendor: String,
    expense_id: Option<Uuid>,
    created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
struct MaintenanceInput {
    lease_id: Option<Uuid>,
    title: String,
    description: Option<String>,
    status: Option<String>,
    priority: Option<String>,
    started_at: Option<DateTime<Utc>>,
    completed_at: Option<DateTime<Utc>>,
    vendor: Option<String>,
    expense_id: Option<Uuid>,
}

const MAINT_COLS: &str = "id, property_id, lease_id, title, description,
    status::text, priority::text, reported_at, started_at, completed_at,
    vendor, expense_id, created_at";

async fn list_maintenance(
    State(s): State<AppState>,
    u: AuthUser,
    Path(property_id): Path<Uuid>,
) -> Result<Json<Vec<Maintenance>>, ApiError> {
    ensure_property_owner(&s, u.id, property_id).await?;
    Ok(Json(
        sqlx::query_as(&format!(
            "SELECT {MAINT_COLS} FROM rental_maintenance
              WHERE property_id = $1
              ORDER BY status = 'done' ASC, reported_at DESC"
        ))
        .bind(property_id)
        .fetch_all(&s.pool)
        .await?,
    ))
}

async fn create_maintenance(
    State(s): State<AppState>,
    u: AuthUser,
    Path(property_id): Path<Uuid>,
    Json(b): Json<MaintenanceInput>,
) -> Result<Json<Maintenance>, ApiError> {
    ensure_property_owner(&s, u.id, property_id).await?;
    if b.title.trim().is_empty() {
        return Err(ApiError::BadRequest("title required".into()));
    }
    let status = parse_maint_status(b.status.as_deref().unwrap_or("open"))?;
    let priority = parse_maint_priority(b.priority.as_deref().unwrap_or("normal"))?;
    Ok(Json(
        sqlx::query_as(&format!(
            "INSERT INTO rental_maintenance
               (property_id, lease_id, title, description, status, priority,
                started_at, completed_at, vendor, expense_id)
             VALUES ($1, $2, $3, $4, $5::maintenance_status_t,
                     $6::maintenance_priority_t, $7, $8, $9, $10)
             RETURNING {MAINT_COLS}"
        ))
        .bind(property_id)
        .bind(b.lease_id)
        .bind(&b.title)
        .bind(b.description.unwrap_or_default())
        .bind(status)
        .bind(priority)
        .bind(b.started_at)
        .bind(b.completed_at)
        .bind(b.vendor.unwrap_or_default())
        .bind(b.expense_id)
        .fetch_one(&s.pool)
        .await?,
    ))
}

async fn update_maintenance(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
    Json(b): Json<MaintenanceInput>,
) -> Result<Json<Maintenance>, ApiError> {
    // ownership check via subquery in WHERE
    let status = parse_maint_status(b.status.as_deref().unwrap_or("open"))?;
    let priority = parse_maint_priority(b.priority.as_deref().unwrap_or("normal"))?;
    let row = sqlx::query_as(&format!(
        "UPDATE rental_maintenance SET
            lease_id = $2, title = $3, description = $4,
            status = $5::maintenance_status_t,
            priority = $6::maintenance_priority_t,
            started_at = $7, completed_at = $8, vendor = $9, expense_id = $10
          WHERE id = $1
            AND property_id IN (SELECT id FROM rental_properties WHERE user_id = $11)
          RETURNING {MAINT_COLS}"
    ))
    .bind(id)
    .bind(b.lease_id)
    .bind(&b.title)
    .bind(b.description.unwrap_or_default())
    .bind(status)
    .bind(priority)
    .bind(b.started_at)
    .bind(b.completed_at)
    .bind(b.vendor.unwrap_or_default())
    .bind(b.expense_id)
    .bind(u.id)
    .fetch_optional(&s.pool)
    .await?
    .ok_or(ApiError::NotFound)?;
    Ok(Json(row))
}

async fn delete_maintenance_row(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let n = sqlx::query(
        "DELETE FROM rental_maintenance
          WHERE id = $1
            AND property_id IN (SELECT id FROM rental_properties WHERE user_id = $2)",
    )
    .bind(id)
    .bind(u.id)
    .execute(&s.pool)
    .await?
    .rows_affected();
    if n == 0 { return Err(ApiError::NotFound); }
    Ok(Json(serde_json::json!({"deleted": true})))
}

// ---------------------------------------------------------------------------
// services log (QBI 250-hour safe harbor tracker)
// ---------------------------------------------------------------------------

#[derive(Serialize, sqlx::FromRow)]
struct ServiceLog {
    id: Uuid,
    property_id: Uuid,
    performed_on: NaiveDate,
    hours: Decimal,
    activity: String,
    performer: String,
    notes: String,
    created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
struct ServiceInput {
    performed_on: NaiveDate,
    hours: Decimal,
    activity: String,
    performer: Option<String>,
    notes: Option<String>,
}

async fn list_services(
    State(s): State<AppState>,
    u: AuthUser,
    Path(property_id): Path<Uuid>,
) -> Result<Json<Vec<ServiceLog>>, ApiError> {
    ensure_property_owner(&s, u.id, property_id).await?;
    Ok(Json(
        sqlx::query_as(
            "SELECT id, property_id, performed_on, hours, activity, performer,
                    notes, created_at
               FROM rental_services_log
              WHERE property_id = $1
              ORDER BY performed_on DESC",
        )
        .bind(property_id)
        .fetch_all(&s.pool)
        .await?,
    ))
}

async fn create_service(
    State(s): State<AppState>,
    u: AuthUser,
    Path(property_id): Path<Uuid>,
    Json(b): Json<ServiceInput>,
) -> Result<Json<ServiceLog>, ApiError> {
    ensure_property_owner(&s, u.id, property_id).await?;
    if b.hours <= Decimal::ZERO {
        return Err(ApiError::BadRequest("hours must be > 0".into()));
    }
    if b.activity.trim().is_empty() {
        return Err(ApiError::BadRequest("activity required".into()));
    }
    Ok(Json(
        sqlx::query_as(
            "INSERT INTO rental_services_log
               (property_id, performed_on, hours, activity, performer, notes)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, property_id, performed_on, hours, activity, performer,
                       notes, created_at",
        )
        .bind(property_id)
        .bind(b.performed_on)
        .bind(b.hours)
        .bind(&b.activity)
        .bind(b.performer.unwrap_or_else(|| "self".into()))
        .bind(b.notes.unwrap_or_default())
        .fetch_one(&s.pool)
        .await?,
    ))
}

async fn delete_service(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let n = sqlx::query(
        "DELETE FROM rental_services_log
          WHERE id = $1
            AND property_id IN (SELECT id FROM rental_properties WHERE user_id = $2)",
    )
    .bind(id)
    .bind(u.id)
    .execute(&s.pool)
    .await?
    .rows_affected();
    if n == 0 { return Err(ApiError::NotFound); }
    Ok(Json(serde_json::json!({"deleted": true})))
}

// ---------------------------------------------------------------------------
// categories — seeded read-only list
// ---------------------------------------------------------------------------

#[derive(Serialize, sqlx::FromRow)]
struct Category {
    code: String,
    schedule_e_line: String,
    label: String,
    deduction_pct: Decimal,
    sort_order: i32,
}

async fn list_schedule_e_categories(
    State(s): State<AppState>,
    _u: AuthUser,
) -> Result<Json<Vec<Category>>, ApiError> {
    Ok(Json(
        sqlx::query_as(
            "SELECT code, schedule_e_line, label, deduction_pct, sort_order
               FROM schedule_e_categories ORDER BY sort_order",
        )
        .fetch_all(&s.pool)
        .await?,
    ))
}

// ---------------------------------------------------------------------------
// Schedule E roll-up report
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct ReportQuery {
    year: i32,
}

async fn schedule_e_report(
    State(s): State<AppState>,
    u: AuthUser,
    Query(q): Query<ReportQuery>,
) -> Result<Json<ScheduleEReport>, ApiError> {
    let start = NaiveDate::from_ymd_opt(q.year, 1, 1)
        .ok_or_else(|| ApiError::BadRequest("invalid year".into()))?;
    let end = NaiveDate::from_ymd_opt(q.year + 1, 1, 1)
        .ok_or_else(|| ApiError::BadRequest("invalid year".into()))?;

    // Pull the fields needed for both the line items AND the per-property
    // depreciation (purchase price, land value, placed-in-service date,
    // recovery period). Properties missing any of those just get $0
    // depreciation for the year — the rest of the roll-up still works.
    let props: Vec<PropertyRollupRow> =
        sqlx::query_as(
            "SELECT id, property_type::text, fair_rental_days, personal_use_days,
                    purchase_price, land_value, placed_in_service_at, recovery_period_years
               FROM rental_properties WHERE user_id = $1 AND status != 'archived'",
        )
        .bind(u.id)
        .fetch_all(&s.pool)
        .await?;

    let mut lines = Vec::with_capacity(props.len());
    for (pid, ptype, frd, pud, purchase, land, placed, recovery) in props {
        let income_rows: Vec<(String, Decimal)> = sqlx::query_as(
            "SELECT kind::text, amount FROM rental_income
              WHERE property_id = $1 AND posted_at >= $2 AND posted_at < $3",
        )
        .bind(pid)
        .bind(start)
        .bind(end)
        .fetch_all(&s.pool)
        .await?;

        let income: Vec<IncomeRow> = income_rows
            .iter()
            .map(|(k, a)| IncomeRow { kind: income_kind_enum(k), amount: *a })
            .collect();

        let expense_rows: Vec<(String, Decimal, bool)> = sqlx::query_as(
            "SELECT category_code, amount, is_capitalized FROM rental_expenses
              WHERE property_id = $1 AND posted_at >= $2 AND posted_at < $3",
        )
        .bind(pid)
        .bind(start)
        .bind(end)
        .fetch_all(&s.pool)
        .await?;

        let expenses: Vec<ExpenseRow> = expense_rows
            .iter()
            .filter_map(|(code, amt, cap)| {
                ScheduleECategory::from_code(code).map(|cat| ExpenseRow {
                    category: cat,
                    amount: *amt,
                    is_capitalized: *cap,
                })
            })
            .collect();

        let mileage_rows: Vec<(Decimal, Decimal)> = sqlx::query_as(
            "SELECT miles, rate_per_mile FROM rental_mileage
              WHERE property_id = $1 AND drove_on >= $2 AND drove_on < $3",
        )
        .bind(pid)
        .bind(start)
        .bind(end)
        .fetch_all(&s.pool)
        .await?;

        let mileage: Vec<MileageRow> = mileage_rows
            .iter()
            .map(|(m, r)| MileageRow { miles: *m, rate_per_mile: *r })
            .collect();

        // Depreciation per IRS Pub 946 Table A-6 (residential, 27.5y) or
        // A-7a (commercial, 39y). Land is never depreciable. Anything
        // other than residential and commercial gets $0 — land, royalties,
        // and self-rental don't claim line-18 depreciation on Schedule E.
        let depreciation_for_year = match (purchase, placed) {
            (Some(p), Some(pd)) => {
                let basis = (p - land.unwrap_or(Decimal::ZERO)).max(Decimal::ZERO);
                let class = match recovery.to_string().as_str() {
                    "39" | "39.0" => Some(RealPropertyClass::Commercial39),
                    "27.5" => Some(RealPropertyClass::Residential27_5),
                    _ if ptype == "commercial" => Some(RealPropertyClass::Commercial39),
                    _ if matches!(ptype.as_str(),
                        "single_family" | "multi_family" | "vacation_short_term" | "self_rental")
                        => Some(RealPropertyClass::Residential27_5),
                    _ => None,
                };
                class.map(|c| {
                    macrs_rental_year_1_deduction(
                        basis, c,
                        pd.format("%Y").to_string().parse().unwrap_or(q.year),
                        pd.format("%m").to_string().parse().unwrap_or(1),
                        q.year,
                    )
                }).unwrap_or(Decimal::ZERO)
            }
            _ => Decimal::ZERO,
        };

        let pid_str = pid.to_string();
        let input = PropertyInput {
            property_id: &pid_str,
            property_type: property_type_enum(&ptype),
            fair_rental_days: frd as u32,
            personal_use_days: pud as u32,
            income: &income,
            expenses: &expenses,
            mileage: &mileage,
            depreciation_for_year,
        };
        lines.push(roll_property(&input));
    }

    Ok(Json(roll_report(lines)))
}

// ---------------------------------------------------------------------------
// Per-property depreciation report
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct PropertyDepreciationReport {
    year: i32,
    depreciable_basis: Decimal,
    deduction: Decimal,
    note: String,
}

async fn property_depreciation(
    State(s): State<AppState>,
    u: AuthUser,
    Path(property_id): Path<Uuid>,
    Query(q): Query<ReportQuery>,
) -> Result<Json<PropertyDepreciationReport>, ApiError> {
    ensure_property_owner(&s, u.id, property_id).await?;
    let (ptype, purchase, land, placed, recovery): (
        String, Option<Decimal>, Option<Decimal>, Option<NaiveDate>, Decimal,
    ) = sqlx::query_as(
        "SELECT property_type::text, purchase_price, land_value,
                placed_in_service_at, recovery_period_years
           FROM rental_properties WHERE id = $1",
    )
    .bind(property_id)
    .fetch_one(&s.pool)
    .await?;

    let class = match recovery.to_string().as_str() {
        "39" | "39.0" => Some(RealPropertyClass::Commercial39),
        "27.5" => Some(RealPropertyClass::Residential27_5),
        _ if ptype == "commercial" => Some(RealPropertyClass::Commercial39),
        _ if matches!(ptype.as_str(),
            "single_family" | "multi_family" | "vacation_short_term" | "self_rental")
            => Some(RealPropertyClass::Residential27_5),
        _ => None,
    };

    let (basis, deduction, note) = match (purchase, placed, class) {
        (Some(p), Some(pd), Some(c)) => {
            let basis = (p - land.unwrap_or(Decimal::ZERO)).max(Decimal::ZERO);
            let ded = macrs_rental_year_1_deduction(
                basis, c,
                pd.format("%Y").to_string().parse().unwrap_or(q.year),
                pd.format("%m").to_string().parse().unwrap_or(1),
                q.year,
            );
            (basis, ded, format!("MACRS {:?} class", c))
        }
        (None, _, _) => (Decimal::ZERO, Decimal::ZERO, "no purchase_price recorded".into()),
        (_, None, _) => (Decimal::ZERO, Decimal::ZERO, "no placed_in_service_at recorded".into()),
        (_, _, None) => (Decimal::ZERO, Decimal::ZERO,
            format!("property_type '{ptype}' is not depreciable real property")),
    };

    Ok(Json(PropertyDepreciationReport {
        year: q.year,
        depreciable_basis: basis,
        deduction,
        note,
    }))
}

// ---------------------------------------------------------------------------
// State security-deposit-interest accrual
// ---------------------------------------------------------------------------

async fn deposit_interest_accrue(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<AccrualInput>,
) -> Result<Json<AccrualResult>, ApiError> {
    if b.state.trim().is_empty() {
        return Err(ApiError::BadRequest("state required".into()));
    }
    if b.deposit < Decimal::ZERO {
        return Err(ApiError::BadRequest("deposit must be >= 0".into()));
    }
    Ok(Json(accrue_deposit_interest(&b)))
}

// ---------------------------------------------------------------------------
// §469 passive activity loss limitation calculator
// ---------------------------------------------------------------------------

async fn section_469_compute(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<Section469Input>,
) -> Result<Json<Section469Result>, ApiError> {
    if b.current_year_loss < Decimal::ZERO {
        return Err(ApiError::BadRequest("current_year_loss must be >= 0 (pass loss as positive)".into()));
    }
    if b.prior_year_carryover < Decimal::ZERO {
        return Err(ApiError::BadRequest("prior_year_carryover must be >= 0".into()));
    }
    Ok(Json(compute_section_469(&b)))
}

// ---------------------------------------------------------------------------
// Property disposition: §1250 unrecaptured + §1231 + §1031 deferral
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct DispositionRequest {
    sale_price: Decimal,
    selling_costs: Decimal,
    /// If omitted, computed from rental_properties.purchase_price.
    original_cost_basis: Option<Decimal>,
    /// If omitted, summed from rental_expenses where category_code =
    /// 'e_depreciation' for this property (current MACRS deductions
    /// rolled up via the API stay outside this number — caller must
    /// pass the actual accumulated depreciation if it includes the
    /// computed line-18 numbers from prior years).
    accumulated_depreciation: Option<Decimal>,
    capital_improvements_added: Option<Decimal>,
    like_kind_exchange: Option<traderview_expense::disposition::LikeKindExchange>,
    filing_status: Option<String>,
}

// ---------------------------------------------------------------------------
// §280A vacation-home / mixed-use classification
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct Section280ARequest {
    /// If omitted, pulled from rental_properties.fair_rental_days.
    fair_rental_days: Option<u32>,
    /// If omitted, pulled from rental_properties.personal_use_days.
    personal_use_days: Option<u32>,
    /// Gross rental income for the year.
    gross_rental_income: Decimal,
    tier_1_expenses_personal_deductible: Decimal,
    tier_2_operating_expenses: Decimal,
    tier_3_depreciation: Decimal,
    #[serde(default)]
    prior_year_suspended: Decimal,
}

async fn property_section_280a(
    State(s): State<AppState>,
    u: AuthUser,
    Path(property_id): Path<Uuid>,
    Json(b): Json<Section280ARequest>,
) -> Result<Json<Section280AResult>, ApiError> {
    ensure_property_owner(&s, u.id, property_id).await?;
    // Fill missing day counts from the property record.
    let (frd, pud) = match (b.fair_rental_days, b.personal_use_days) {
        (Some(frd), Some(pud)) => (frd, pud),
        (frd_opt, pud_opt) => {
            let (db_frd, db_pud): (i32, i32) = sqlx::query_as(
                "SELECT fair_rental_days, personal_use_days
                   FROM rental_properties WHERE id = $1",
            )
            .bind(property_id)
            .fetch_one(&s.pool)
            .await?;
            (
                frd_opt.unwrap_or(db_frd.max(0) as u32),
                pud_opt.unwrap_or(db_pud.max(0) as u32),
            )
        }
    };
    if [
        b.gross_rental_income,
        b.tier_1_expenses_personal_deductible,
        b.tier_2_operating_expenses,
        b.tier_3_depreciation,
        b.prior_year_suspended,
    ]
    .iter()
    .any(|x| *x < Decimal::ZERO)
    {
        return Err(ApiError::BadRequest(
            "income and expense amounts must be >= 0".into(),
        ));
    }
    Ok(Json(compute_section_280a(&Section280AInput {
        fair_rental_days: frd,
        personal_use_days: pud,
        gross_rental_income: b.gross_rental_income,
        tier_1_expenses_personal_deductible: b.tier_1_expenses_personal_deductible,
        tier_2_operating_expenses: b.tier_2_operating_expenses,
        tier_3_depreciation: b.tier_3_depreciation,
        prior_year_suspended: b.prior_year_suspended,
    })))
}

// ---------------------------------------------------------------------------
// Cost segregation + §168(k) bonus depreciation
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct CostSegRequest {
    /// If omitted, computed from (purchase_price - land_value) on the
    /// property row.
    depreciable_basis: Option<Decimal>,
    /// If omitted, inferred from rental_properties.property_type.
    cost_seg_type: Option<String>,
    allocation_override: Option<traderview_expense::cost_segregation::CostSegAllocation>,
    tax_year: i32,
    #[serde(default)]
    elect_bonus_depreciation: bool,
}

fn cost_seg_type_from_property(ptype: &str) -> CostSegPropertyType {
    match ptype {
        "single_family"       => CostSegPropertyType::SingleFamily,
        "multi_family"        => CostSegPropertyType::MultiFamily,
        "vacation_short_term" => CostSegPropertyType::ShortTermRental,
        "commercial"          => CostSegPropertyType::Commercial,
        _                     => CostSegPropertyType::SingleFamily,
    }
}

fn parse_cost_seg_type(s: &str) -> Result<CostSegPropertyType, ApiError> {
    Ok(match s {
        "single_family"     => CostSegPropertyType::SingleFamily,
        "multi_family"      => CostSegPropertyType::MultiFamily,
        "short_term_rental" => CostSegPropertyType::ShortTermRental,
        "commercial"        => CostSegPropertyType::Commercial,
        "restaurant"        => CostSegPropertyType::Restaurant,
        _ => return Err(ApiError::BadRequest(format!("invalid cost_seg_type: {s}"))),
    })
}

// ---------------------------------------------------------------------------
// State late-fee cap + grace-period compliance check
// ---------------------------------------------------------------------------

async fn late_fee_check_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<LateFeeCheckInput>,
) -> Result<Json<LateFeeCheckResult>, ApiError> {
    if b.state.trim().is_empty() {
        return Err(ApiError::BadRequest("state required".into()));
    }
    if b.monthly_rent < Decimal::ZERO || b.proposed_late_fee < Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "monthly_rent and proposed_late_fee must be >= 0".into(),
        ));
    }
    Ok(Json(check_late_fee(&b)))
}

// ---------------------------------------------------------------------------
// §280A(d)(2) related-party rental personal-use classifier
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct Section280AD2Request {
    periods: Vec<OccupancyPeriod>,
}

async fn section_280a_d2_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<Section280AD2Request>,
) -> Result<Json<Section280AD2Report>, ApiError> {
    Ok(Json(compute_section_280a_d2(&b.periods)))
}

// ---------------------------------------------------------------------------
// State eviction-notice period lookup
// ---------------------------------------------------------------------------

async fn eviction_notice_check_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<NoticeCheckInput>,
) -> Result<Json<NoticeCheckResult>, ApiError> {
    if b.state.trim().is_empty() {
        return Err(ApiError::BadRequest("state required".into()));
    }
    Ok(Json(check_eviction_notice(&b)))
}

// ---------------------------------------------------------------------------
// State senior + disabled tenant protection check
// ---------------------------------------------------------------------------

async fn senior_disabled_check_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<SeniorDisabledCheckInput>,
) -> Result<Json<SeniorDisabledCheckResult>, ApiError> {
    if b.state_code.trim().is_empty() {
        return Err(ApiError::BadRequest("state_code required".into()));
    }
    Ok(Json(check_senior_disabled(&b)))
}

// ---------------------------------------------------------------------------
// Service animal / ESA accommodation compliance check (federal FHA + state)
// ---------------------------------------------------------------------------

async fn service_animal_check_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<ServiceAnimalCheckInput>,
) -> Result<Json<ServiceAnimalCheckResult>, ApiError> {
    if b.state_code.trim().is_empty() {
        return Err(ApiError::BadRequest("state_code required".into()));
    }
    Ok(Json(check_service_animal(&b)))
}

// ---------------------------------------------------------------------------
// State tenant abandonment threshold compliance check
// ---------------------------------------------------------------------------

async fn abandonment_check_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<TenantAbandonmentInput>,
) -> Result<Json<TenantAbandonmentResult>, ApiError> {
    if b.state_code.trim().is_empty() {
        return Err(ApiError::BadRequest("state_code required".into()));
    }
    Ok(Json(check_tenant_abandonment(&b)))
}

// ---------------------------------------------------------------------------
// State short-term rental (Airbnb/VRBO) regulation compliance check
// ---------------------------------------------------------------------------

async fn str_regulation_check_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<StrComplianceInput>,
) -> Result<Json<StrComplianceResult>, ApiError> {
    if b.state_code.trim().is_empty() {
        return Err(ApiError::BadRequest("state_code required".into()));
    }
    Ok(Json(check_str_regulation(&b)))
}

// ---------------------------------------------------------------------------
// State pet deposit / pet rent / pet fee compliance check
//
// Mounted at POST /api/rental/pet-fees-check. Four-regime table:
// SpecificPetDepositAndRentCap (CO: $300 + max($35, 1.5% × rent)),
// TotalDepositCapAbsorbsPet (CA AB 12, WA RCW 59.18.260),
// NoSeparatePetDepositAllowed (MA: pet deposit banned, monthly pet
// rent OK), NoStateRule (38 silent states). Federal FHA + ADA floor:
// zero-charge override for service animal / ESA in every state.
// ---------------------------------------------------------------------------

async fn pet_fees_check_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<PetFeeInput>,
) -> Result<Json<PetFeeResult>, ApiError> {
    if b.state_code.trim().is_empty() {
        return Err(ApiError::BadRequest("state_code required".into()));
    }
    if b.monthly_rent_cents < 0 {
        return Err(ApiError::BadRequest(
            "monthly_rent_cents must be >= 0".into(),
        ));
    }
    if b.charged_pet_deposit_cents < 0 {
        return Err(ApiError::BadRequest(
            "charged_pet_deposit_cents must be >= 0".into(),
        ));
    }
    if b.charged_pet_rent_monthly_cents < 0 {
        return Err(ApiError::BadRequest(
            "charged_pet_rent_monthly_cents must be >= 0".into(),
        ));
    }
    Ok(Json(check_pet_fees(&b)))
}

// ---------------------------------------------------------------------------
// State eviction record sealing / "clean slate" compliance check
//
// Mounted at POST /api/rental/eviction-sealing-check. Four-regime
// table: AutomaticSealing (CA 60-day mask, CT 30-day favorable-outcome
// auto-seal, NV 31-day, MD 60-day non-removal, MN same-day expunge);
// TenantPetitionOnly (WA, OR, IL, DC); PandemicPeriodOnly (NJ A 4463);
// NoStateRule (most states). Federal FCRA 15 U.S.C. § 1681c 7-year
// floor always applies for tenant screening reports.
// ---------------------------------------------------------------------------

async fn eviction_sealing_check_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<EvictionSealingInput>,
) -> Result<Json<EvictionSealingResult>, ApiError> {
    if b.state_code.trim().is_empty() {
        return Err(ApiError::BadRequest("state_code required".into()));
    }
    if b.days_since_filing_or_qualifying_event < 0 {
        return Err(ApiError::BadRequest(
            "days_since_filing_or_qualifying_event must be >= 0".into(),
        ));
    }
    Ok(Json(check_eviction_sealing(&b)))
}

// ---------------------------------------------------------------------------
// State landlord-side lease termination / non-renewal notice check
//
// Mounted at POST /api/rental/termination-notice-check. Four regimes:
// TieredByTenancyLength (NY RPL § 226-c 30/60/90; CA CCP § 1946.1
// 30/60); JustCauseAfterTwelveMonths (OR SB 608, CA AB 1482 -- no
// no-cause termination after 12mo without qualifying cause);
// StatewideJustCauseAlways (WA RCW 59.18.650, NJ Anti-Eviction Act);
// StandardThirtyDay (most other states). Also pins rent-increase
// notice tiers (CA: 30d <=10% / 90d >10%; OR: 90d <=10% / 180d >10%).
// ---------------------------------------------------------------------------

async fn termination_notice_check_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<NoticeInput>,
) -> Result<Json<NoticeResult>, ApiError> {
    if b.state_code.trim().is_empty() {
        return Err(ApiError::BadRequest("state_code required".into()));
    }
    if b.notice_days_given < 0 {
        return Err(ApiError::BadRequest(
            "notice_days_given must be >= 0".into(),
        ));
    }
    Ok(Json(check_termination_notice(&b)))
}

// ---------------------------------------------------------------------------
// Federal + state occupancy standards check (HUD Keating + state overlay)
//
// Mounted at POST /api/rental/occupancy-check. Four regimes:
// SqftPerOccupantFormula (NY 80 sqft/person, MA 150+100); CA "2+1"
// (2 per bedroom + 1 additional); NoMoreRestrictiveThanTwoPerBedroom
// (OR ORS 90.262); HudKeatingDefault (everywhere else). Federal floor:
// FHA § 3604 familial-status pretext violation overrides any
// state-formula compliance.
// ---------------------------------------------------------------------------

async fn occupancy_check_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<OccupancyInput>,
) -> Result<Json<OccupancyResult>, ApiError> {
    if b.state_code.trim().is_empty() {
        return Err(ApiError::BadRequest("state_code required".into()));
    }
    Ok(Json(check_occupancy(&b)))
}

// ---------------------------------------------------------------------------
// State move-in / move-out inspection requirement check
//
// Mounted at POST /api/rental/move-in-inspection-check. Four regimes:
// MandatoryMoveInChecklist (WA strictest — full deposit forfeit + atty
// fees on failure; AZ ARS § 33-1321; MI MCL 554.608 7-day window;
// KY KRS 383.580); TenantRequestedMoveInChecklist (MD § 8-203.1 within
// 15-day tenant request window); PreMoveOutInspectionOffer (CA Civ.
// Code § 1950.5(f) walk-through offer); NoStateRequirement elsewhere.
// ---------------------------------------------------------------------------

async fn move_in_inspection_check_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<InspectionInput>,
) -> Result<Json<InspectionResult>, ApiError> {
    if b.state_code.trim().is_empty() {
        return Err(ApiError::BadRequest("state_code required".into()));
    }
    Ok(Json(check_move_in_inspection(&b)))
}

// ---------------------------------------------------------------------------
// State renters insurance landlord-requirement compliance check
//
// Mounted at POST /api/rental/renters-insurance-check. Two regimes:
// StatutoryCapWithLowIncomeExemption (OR ORS 90.222 only — $100k
// liability cap + landlord-additional-insured prohibition + ≤50% AMI
// low-income exemption); GenerallyAllowedNoStateCap (all other 49
// states + DC, including OK which permits requirement under Okla.
// Stat. tit. 41 § 113). No US state prohibits requiring renters
// insurance entirely.
// ---------------------------------------------------------------------------

async fn renters_insurance_check_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentersInsuranceInput>,
) -> Result<Json<RentersInsuranceResult>, ApiError> {
    if b.state_code.trim().is_empty() {
        return Err(ApiError::BadRequest("state_code required".into()));
    }
    if b.required_liability_coverage_dollars < 0 {
        return Err(ApiError::BadRequest(
            "required_liability_coverage_dollars must be >= 0".into(),
        ));
    }
    Ok(Json(check_renters_insurance(&b)))
}

// ---------------------------------------------------------------------------
// State landlord-caused utility shutoff prohibition check
//
// Mounted at POST /api/rental/utility-shutoff-check. Five regimes:
// PerDayStatutoryPenalty (CA Civ. Code § 789.3 $100/day + $250 min;
// WA RCW 59.18.300 $100/day); FlatPlusOneMonthRentPenalty (TX Prop.
// Code § 92.008 $1k + 1 month rent); MonthlyRentMultiplePenalty (FL
// Stat. § 83.67 3 months rent or actual whichever higher);
// PunitiveDamagesFramework (NY RPL § 235-a + RPAPL 853 compensatory +
// punitive + treble + criminal); GeneralProhibitionStandardRemedies
// elsewhere. Bona-fide repair/emergency exception bars any violation.
// ---------------------------------------------------------------------------

async fn utility_shutoff_check_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<ShutoffInput>,
) -> Result<Json<ShutoffResult>, ApiError> {
    if b.state_code.trim().is_empty() {
        return Err(ApiError::BadRequest("state_code required".into()));
    }
    if b.monthly_rent_dollars < 0 || b.tenant_actual_damages_dollars < 0 {
        return Err(ApiError::BadRequest(
            "monthly_rent_dollars and tenant_actual_damages_dollars must be >= 0".into(),
        ));
    }
    Ok(Json(check_utility_shutoff(&b)))
}

// ---------------------------------------------------------------------------
// vehicle_towing_from_rental_property: Landlord vehicle towing from rental
// property — compliance check for unauthorized-vehicle removal under state
// vehicle/transportation codes. Four regimes: California (Cal. Veh. Code
// § 22658 — 17×22 inch signage at all entrances + 96-hour rule for parking-
// violation notice + DOUBLE storage/towing charges liability under
// § 22658(l)(1) for non-compliance); Texas (Tex. Occ. Code § 2308.252
// signage + § 2308.253 apartment-complex 10-day registration-tow notice +
// § 2308.255 written verification to towing company); Florida (Fla. Stat.
// § 715.07 signage + § 715.07(2)(a)(3) 10-mile / 15-mile storage radius
// depending on county population + § 715.07(2)(a)(4) 30-minute law-
// enforcement notification + § 715.07(4) stop-during-tow half-fee
// redemption + § 715.07(2)(c) single-family residence personal-notice
// exception); Default (common-law trespass to chattel + state-specific
// vehicle code). Distinct from landlord_lien_prohibition, tenant_
// abandonment, abandoned_property_handling.
// ---------------------------------------------------------------------------

async fn vehicle_towing_from_rental_property_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<VehicleTowingInput>,
) -> Result<Json<VehicleTowingResult>, ApiError> {
    if b.fl_storage_distance_miles > 10_000 {
        return Err(ApiError::BadRequest(
            "fl_storage_distance_miles looks invalid (>10000)".into(),
        ));
    }
    if b.fl_minutes_to_law_enforcement_notification > 100_000 {
        return Err(ApiError::BadRequest(
            "fl_minutes_to_law_enforcement_notification looks invalid (>100000)".into(),
        ));
    }
    Ok(Json(check_vehicle_towing(&b)))
}

// ---------------------------------------------------------------------------
// Tenant adverse action notice compliance check (federal FCRA + state)
//
// Mounted at POST /api/rental/adverse-action-check. Two regimes:
// StateAddsRequirements (CA Civ. Code § 1785.20.5 + § 1786 ICRA --
// specific reason + but-for + 12-point formatting; WA RCW 59.18.257 +
// RCW 19.182.110 -- specific reason + but-for; NY GBL § 380-b --
// specific reason); FederalFcraOnly (FCRA § 615 / 15 U.S.C. § 1681m
// floor only -- CRA contact info, CRA-did-not-decide disclosure,
// 60-day free copy right, dispute right).
// ---------------------------------------------------------------------------

async fn adverse_action_check_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<AdverseActionInput>,
) -> Result<Json<AdverseActionResult>, ApiError> {
    if b.state_code.trim().is_empty() {
        return Err(ApiError::BadRequest("state_code required".into()));
    }
    Ok(Json(check_adverse_action(&b)))
}

// ---------------------------------------------------------------------------
// adverse_possession_claim: Squatter / adverse-possession statutory
// limitations check. Five regimes: California (Cal. Civ. Proc. Code § 325 —
// 5 years + tax payment in assessed year per Gilardi v. Hallam); Texas
// (Ch. 16 four-tier: § 16.024 3-year color of title; § 16.025 5-year
// recorded deed + cultivation + taxes; § 16.026 10-year peaceable +
// cultivation; § 16.027/.028 25-year regardless of disability or void
// deed); Florida (§ 95.16 7-year with color of title + taxes; § 95.18
// 7-year without color + 1-year tax cure + 30-day appraiser return);
// NewYork (RPAPL §§ 501(3), 511, 521 + CPLR § 212(a) — 10 years + post-
// 2008 claim-of-right reasonable-basis under § 501(3) overruling Walling
// v. Prysbylo); Default (common-law 15-30 years, state-specific). All
// five common-law elements (actual + open and notorious + hostile +
// exclusive + continuous) required regardless of regime. Distinct from
// landlord_possession_delivery, tenant_abandonment, foreclosure_tenant_
// rights.
async fn adverse_possession_claim_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<AdversePossessionInput>,
) -> Result<Json<AdversePossessionResult>, ApiError> {
    if b.years_of_possession > 1_000 {
        return Err(ApiError::BadRequest(
            "years_of_possession looks invalid (>1000)".into(),
        ));
    }
    Ok(Json(check_adverse_possession(&b)))
}

// ---------------------------------------------------------------------------
// Tenant Opportunity to Purchase Act (TOPA) compliance check
//
// Mounted at POST /api/rental/topa-check. Four regimes:
// AllSalesGeneralTopa (D.C. Code § 42-3404.02 + § 42-3404.08 -- all
// residential sales, 15-day window, foreclosure/tax/bankruptcy
// exempt); NarrowResidentialTopa (MD HB 693 of 2024 -- 3-or-fewer
// units); ForeclosureOnlyPriority (CA SB 1079 of 2020 -- 1-4 unit
// SFR foreclosure only, 15/45 day priority bid); NoStateTopa
// elsewhere (local ordinances may apply, e.g., MA H.1260/S.786
// pending statewide opt-in).
// ---------------------------------------------------------------------------

async fn tenant_topa_check_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<TopaInput>,
) -> Result<Json<TopaResult>, ApiError> {
    if b.state_code.trim().is_empty() {
        return Err(ApiError::BadRequest("state_code required".into()));
    }
    Ok(Json(check_tenant_topa(&b)))
}

// ---------------------------------------------------------------------------
// State lease automatic renewal / evergreen clause disclosure check
//
// Mounted at POST /api/rental/auto-renewal-check. Three regimes:
// PreNonrenewalNotificationFifteenToThirtyDays (FL Fla. Stat.
// § 83.575 with fees-listing requirement, WI § 704.15, NY GBL
// § 5-905); PreCancellationDeadlineThirtyToSixtyDays (IL 815 ILCS
// 601/10 with clear-and-conspicuous clause requirement);
// NoStateDisclosureRequirement elsewhere. Without compliant notice,
// auto-renewal clause is UNENFORCEABLE in the strict regime states.
// ---------------------------------------------------------------------------

async fn lease_auto_renewal_check_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<AutoRenewalInput>,
) -> Result<Json<AutoRenewalResult>, ApiError> {
    if b.state_code.trim().is_empty() {
        return Err(ApiError::BadRequest("state_code required".into()));
    }
    Ok(Json(check_lease_auto_renewal(&b)))
}

// ---------------------------------------------------------------------------
// State mandatory lease translation requirement check
//
// Mounted at POST /api/rental/lease-translation-check. Three regimes:
// MandatoryTranslationFiveLanguages (CA Civ. Code § 1632 -- Spanish /
// Chinese / Tagalog / Vietnamese / Korean residential leases > 1
// month; failure = tenant rescission right); EnglishRequiredTrans-
// lationsNotBinding (FL); NoStateTranslationRequirement elsewhere.
// ---------------------------------------------------------------------------

async fn lease_translation_check_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<TranslationInput>,
) -> Result<Json<TranslationResult>, ApiError> {
    if b.state_code.trim().is_empty() {
        return Err(ApiError::BadRequest("state_code required".into()));
    }
    Ok(Json(check_lease_translation(&b)))
}

// ---------------------------------------------------------------------------
// State rent receipt requirement check
//
// Mounted at POST /api/rental/rent-receipt-check. Four regimes:
// MandatoryReceiptEveryPayment (NY RPL § 235-e HSTPA 2019 -- every
// payment regardless of method); MandatoryReceiptCashPaymentsOnly
// (CA Civ. Code § 1499, MD Real Prop. § 8-208, NJ Truth-in-Renting,
// IL Chicago RLTO -- cash payments only); ReceiptUponTenantRequest
// (WA RCW 59.18.063 -- only when tenant requests);
// NoStateReceiptRequirement elsewhere (including MA whose receipt
// rule applies only to security deposit + last month rent under
// Ch. 186 § 15B, tracked elsewhere).
// ---------------------------------------------------------------------------

async fn rent_receipt_check_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<ReceiptInput>,
) -> Result<Json<ReceiptResult>, ApiError> {
    if b.state_code.trim().is_empty() {
        return Err(ApiError::BadRequest("state_code required".into()));
    }
    Ok(Json(check_rent_receipts(&b)))
}

// ---------------------------------------------------------------------------
// State tenant repair-and-deduct / rent escrow / habitability remedy
//
// Mounted at POST /api/rental/repair-deduct-check. Four regimes:
// SelfHelpRepairAndDeduct (CA 1 month + max 2/year; TX greater of
// $500 or 1 month; MA 4 months (highest in country); WA 1 month +
// 30-day notice; IL greater of $500 or 1 month; others default 1
// month); CourtOrderedRentEscrowOnly (MD § 8-211 -- no self-help,
// court-ordered escrow only on substantial-threat finding);
// CommonLawMariniDoctrine (NJ Marini v. Ireland -- withheld rent
// must be deposited with court at trial); NoticeThenWithholdOr-
// Terminate (FL § 83.56(1) -- 7-day notice required).
// ---------------------------------------------------------------------------

async fn repair_and_deduct_check_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RepairDeductInput>,
) -> Result<Json<RepairDeductResult>, ApiError> {
    if b.state_code.trim().is_empty() {
        return Err(ApiError::BadRequest("state_code required".into()));
    }
    if b.monthly_rent_dollars < 0 || b.repair_costs_paid_dollars < 0 {
        return Err(ApiError::BadRequest(
            "monthly_rent_dollars and repair_costs_paid_dollars must be >= 0".into(),
        ));
    }
    Ok(Json(check_repair_and_deduct(&b)))
}

// ---------------------------------------------------------------------------
// State cosigner / lease guarantor enforcement rules check
//
// Mounted at POST /api/rental/cosigner-check. Two regimes:
// IllinoisStatutoryNoticeRequired (IL 815 ILCS 505/2S -- first-class-
// mail notice required to cosigner 15 days before collection action;
// $250 statutory damages + attorney fees on violation);
// CommonLawSuretyRules (49 other states + DC -- continuing-vs-
// specific-term guaranty doctrine governs renewal liability; no
// state-mandated pre-collection notice).
// ---------------------------------------------------------------------------

async fn cosigner_rules_check_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<CosignerInput>,
) -> Result<Json<CosignerResult>, ApiError> {
    if b.state_code.trim().is_empty() {
        return Err(ApiError::BadRequest("state_code required".into()));
    }
    Ok(Json(check_cosigner_rules(&b)))
}

// ---------------------------------------------------------------------------
// State mobile home park / manufactured housing compliance check
//
// Mounted at POST /api/rental/mobile-home-park-check. Three regimes:
// JustCauseWithRentCap (CA Civ. Code § 798 + OR ORS Ch. 90 / SB 608
// -- 90-day notice + just-cause + 10% rent cap for OR; CA local rent
// control); NoticeAndJustCauseNoCap (FL Ch. 723 Mobile Home Act --
// applies to parks of 10+ lots + 90-day notice + just-cause; WA RCW
// 59.20 with 2025 amendments); GenericLandlordTenantLaw elsewhere.
// ---------------------------------------------------------------------------

async fn mobile_home_park_check_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<MhpInput>,
) -> Result<Json<MhpResult>, ApiError> {
    if b.state_code.trim().is_empty() {
        return Err(ApiError::BadRequest("state_code required".into()));
    }
    Ok(Json(check_mobile_home_park(&b)))
}

// ---------------------------------------------------------------------------
// State utility submetering / RUBS compliance check
//
// Mounted at POST /api/rental/submetering-check. Three regimes:
// DisclosureAndTestingRequired (CA Civ. Code § 1954.201 SB 7 of 2016
// + VA Va. Code § 55.1-1212 -- lease disclosure required + free
// tenant-requested meter testing); PSCRegisteredCappedFees (TX Water
// Code Ch. 13 + TCEQ 16 TAC § 24.275 -- PSC registration + 5% late
// fee cap + 9% service charge cap); NoStateRegulation elsewhere.
// ---------------------------------------------------------------------------

async fn submetering_check_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<SubmeteringInput>,
) -> Result<Json<SubmeteringResult>, ApiError> {
    if b.state_code.trim().is_empty() {
        return Err(ApiError::BadRequest("state_code required".into()));
    }
    Ok(Json(check_submetering(&b)))
}

// ---------------------------------------------------------------------------
// Federal HUD + state smoke-free housing compliance check
//
// Mounted at POST /api/rental/smoke-free-check. Two regimes:
// HudFloorPlusStateAdditions (CA Cal. Labor Code § 6404.5 + AB 1316
// + local Berkeley 2014 ordinance; MN Minn. Stat. § 144.414 + 2024
// cannabis amendment; OR ORS Ch. 90 90-day existing-tenant
// conversion notice); HudFloorOnly (federal 24 CFR § 965.653 for
// public housing only; private market governed by lease + local).
// ---------------------------------------------------------------------------

async fn smoke_free_check_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<SmokeFreeInput>,
) -> Result<Json<SmokeFreeResult>, ApiError> {
    if b.state_code.trim().is_empty() {
        return Err(ApiError::BadRequest("state_code required".into()));
    }
    Ok(Json(check_smoke_free(&b)))
}

// ---------------------------------------------------------------------------
// State tenant data privacy / records access compliance check
//
// Mounted at POST /api/rental/tenant-privacy-check. Three regimes:
// BiometricStrictWrittenConsent (IL 740 ILCS 14/ BIPA -- no revenue
// threshold + written informed consent before biometric collection +
// $1k negligent / $5k intentional per-violation damages);
// ComprehensivePrivacyLawRevenueThreshold (CA CCPA/CPRA + VA VCDPA +
// CO CPA + CT CTDPA + OR + DE + MD + MN -- 8 states with revenue /
// consumer thresholds + 45-day DSAR window); NoStatePrivacyLaw
// elsewhere.
// ---------------------------------------------------------------------------

async fn tenant_privacy_check_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<PrivacyInput>,
) -> Result<Json<PrivacyResult>, ApiError> {
    if b.state_code.trim().is_empty() {
        return Err(ApiError::BadRequest("state_code required".into()));
    }
    if b.landlord_annual_revenue_dollars < 0 {
        return Err(ApiError::BadRequest(
            "landlord_annual_revenue_dollars must be >= 0".into(),
        ));
    }
    Ok(Json(check_tenant_privacy(&b)))
}

// ---------------------------------------------------------------------------
// Federal HUD One-Strike + state drug-eviction compliance check
//
// Mounted at POST /api/rental/drug-eviction-check. Federal floor:
// 42 U.S.C. § 1437d(l)(6) for public housing + § 1437f for Section 8
// vouchers/certificates; Rucker (2002) strict-liability for
// household/guest activity. State just-cause regime:
// StateJustCauseListsCriminalActivity (CA Civ. Code § 1946.2 TPA, OR
// SB 608, WA RCW 59.18.650, NJ Anti-Eviction Act § 2A:18-61.1(n));
// ContractGovernsPrivateMarket elsewhere.
// ---------------------------------------------------------------------------

async fn drug_eviction_check_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<DrugEvictionInput>,
) -> Result<Json<DrugEvictionResult>, ApiError> {
    if b.state_code.trim().is_empty() {
        return Err(ApiError::BadRequest("state_code required".into()));
    }
    Ok(Json(check_drug_eviction(&b)))
}

// ---------------------------------------------------------------------------
// State quiet enjoyment / nuisance statute compliance check
//
// Mounted at POST /api/rental/quiet-enjoyment-check. Two regimes:
// MassachusettsTrebleDamagesAndCriminal (MA G.L. c. 186 § 14 only --
// damages = greater of actual or 3× monthly rent + intentional
// breach triggers $25-$300 fine + up to 6-month jail);
// CommonLawImpliedCovenant (all other states + DC, with CA Civ.
// Code § 1927 statutory codification; NY RPL § 235-b; IL 765 ILCS
// 705).
// ---------------------------------------------------------------------------

async fn quiet_enjoyment_check_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<QuietEnjoymentInput>,
) -> Result<Json<QuietEnjoymentResult>, ApiError> {
    if b.state_code.trim().is_empty() {
        return Err(ApiError::BadRequest("state_code required".into()));
    }
    if b.monthly_rent_dollars < 0 || b.actual_damages_dollars < 0 {
        return Err(ApiError::BadRequest(
            "monthly_rent_dollars and actual_damages_dollars must be >= 0".into(),
        ));
    }
    Ok(Json(check_quiet_enjoyment(&b)))
}

// ---------------------------------------------------------------------------
// State flood-zone / flood-history disclosure compliance check
//
// Mounted at POST /api/rental/flood-disclosure-check. Five regimes:
// FloridaFloodHistoryClaimsFemaAid (FL only; Fla. Stat. § 83.512 eff.
// 2025-10-01 SB 948; tenant termination within 30 days of substantial
// loss 50%+ market value); NewJerseyFemaFloodZoneAndHistory (NJ only;
// N.J.S.A. 46:8-50 eff. 2024-03-20; immediate termination + refund if
// undisclosed in flood zone); CaliforniaNaturalHazardCombined (CA
// only; Gov. Code § 8589.45 eff. 2018-07-01); PriorFloodKnowledge-
// Disclosure (TX/NY/GA/IN/OK/OR — 6 states, damages remedy only);
// NoStateFloodDisclosure (41 other states + DC, common-law fraud only).
// ---------------------------------------------------------------------------

async fn flood_disclosure_check_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<FloodDisclosureInput>,
) -> Result<Json<FloodDisclosureResult>, ApiError> {
    if b.state_code.trim().is_empty() {
        return Err(ApiError::BadRequest("state_code required".into()));
    }
    if b.prepaid_rent_dollars < 0 {
        return Err(ApiError::BadRequest(
            "prepaid_rent_dollars must be >= 0".into(),
        ));
    }
    Ok(Json(check_flood_disclosure(&b)))
}

// ---------------------------------------------------------------------------
// State landlord/owner identification + agent-for-service compliance check
//
// Mounted at POST /api/rental/owner-identification-check. Four regimes:
// AffirmativePreLeaseDisclosure (CA Civ. Code § 1962 15-day window
// post-lease-execution + FL Fla. Stat. § 83.50 at-commencement
// written disclosure of landlord/agent name and address);
// DisclosureUponWrittenDemand (TX Prop. Code § 92.201 7-day window
// from tenant written demand OR continuous posting OR in-lease;
// § 92.202 damages = one month's rent + $100 + termination right
// after second 7-day failure); MultipleDwellingRegistration (NY
// MDL § 325 + NYC HMC § 27-2098 for 3+ units; NJ N.J.S.A. 55:13A;
// MA G.L. c. 111 § 197A + c. 186 § 15B(7)); LocalLawOrCommonLawOnly
// (44 other states + DC).
// ---------------------------------------------------------------------------

async fn owner_identification_check_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<OwnerIdentificationInput>,
) -> Result<Json<OwnerIdentificationResult>, ApiError> {
    if b.state_code.trim().is_empty() {
        return Err(ApiError::BadRequest("state_code required".into()));
    }
    Ok(Json(check_owner_identification(&b)))
}

// ---------------------------------------------------------------------------
// State tenant-death lease termination compliance check
//
// Mounted at POST /api/rental/tenant-death-termination-check. Five
// regimes: EstateRepresentativeTerminatesWith30DayNotice (TX Prop.
// Code § 92.0162 sole-occupant; written notice + property removal +
// signed inventory; effective on later of 30 days or all conditions
// met); MonthToMonthAutoTerminationOnLastRent (CA Civ. Code § 1934
// month-to-month only — 30 days after last rent payment by deceased);
// LeaseAutoTerminatesOnDateOfDeath (VA § 55.1-1256 — terminated as
// of date of death + 10-day property-disposition notice to
// authorized contact); MultiNoticeStorageRegime (WA RCW 59.18.595
// first + second notice + 45-day storage hold); NoSpecificStatute-
// CommonLawContract (46 other states + DC — lease survives death;
// estate liable through end of term).
// ---------------------------------------------------------------------------

async fn tenant_death_termination_check_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<TenantDeathInput>,
) -> Result<Json<TenantDeathResult>, ApiError> {
    if b.state_code.trim().is_empty() {
        return Err(ApiError::BadRequest("state_code required".into()));
    }
    Ok(Json(check_tenant_death(&b)))
}

// ---------------------------------------------------------------------------
// State tenant late-payment grace-period compliance check
//
// Mounted at POST /api/rental/late-payment-grace-period-check.
// Distinct from late_fee_caps (which caps the fee amount) — this
// captures the WINDOW before any late fee can attach. Six regimes:
// MassachusettsLongGracePeriod (MA G.L. c. 186 § 15B(1)(c) 30
// full days — most generous in U.S.); ConnecticutNineDayGracePeriod
// (CT Gen. Stat. § 47a-15a); StandardFiveDayGracePeriod (NY RPL
// § 238-a HSTPA 2019 + NC G.S. § 42-46 + WA RCW 59.18.170 + VA Code
// § 55.1-1204); OregonFourDayGracePeriod (ORS 90.260); TexasShort-
// GracePeriod (Tex. Prop. Code § 92.019 2 full days + written-lease
// disclosure required); NoStatutoryGracePeriodReasonablenessOnly
// (44 other states + DC).
// ---------------------------------------------------------------------------

async fn late_payment_grace_period_check_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<GracePeriodInput>,
) -> Result<Json<GracePeriodResult>, ApiError> {
    if b.state_code.trim().is_empty() {
        return Err(ApiError::BadRequest("state_code required".into()));
    }
    if b.late_fee_charged_dollars < 0 {
        return Err(ApiError::BadRequest(
            "late_fee_charged_dollars must be >= 0".into(),
        ));
    }
    Ok(Json(check_grace_period(&b)))
}

// ---------------------------------------------------------------------------
// State owner-move-in (OMI) / no-fault eviction restriction check
//
// Mounted at POST /api/rental/owner-move-in-eviction-check. Five
// regimes: CaliforniaSb567Strict (CA Civ. Code § 1946.2 as amended
// by SB 567 eff. 2024-04-01 — 90-day move-in + 12-month residency
// + relocation assistance + 6 qualifying family relations);
// OregonSb608Combined (ORS 90.427 / SB 608 eff. 2019-02-28 — 90-day
// notice + 1 month rent relocation); NewJerseyTripleDamagesGoodFaith
// (N.J.S.A. 2A:18-61.1(l)(3) + § 2A:18-61.6 — ≤ 3-unit building +
// 6-month residency or 3× damages + attorney fees); NewYorkRent-
// StabilizedOnlyOneUnit (NYC RSC § 2524.4 — rent-stabilized only,
// 3-year principal-residence use); NoStateOwnerMoveInRestriction
// (46 other states + DC).
// ---------------------------------------------------------------------------

async fn owner_move_in_eviction_check_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<OwnerMoveInInput>,
) -> Result<Json<OwnerMoveInResult>, ApiError> {
    if b.state_code.trim().is_empty() {
        return Err(ApiError::BadRequest("state_code required".into()));
    }
    if b.tenant_actual_damages_dollars < 0 {
        return Err(ApiError::BadRequest(
            "tenant_actual_damages_dollars must be >= 0".into(),
        ));
    }
    Ok(Json(check_owner_move_in(&b)))
}

// ---------------------------------------------------------------------------
// State landlord lease-copy delivery compliance check
//
// Mounted at POST /api/rental/lease-copy-delivery-check. Four regimes:
// California15DayDelivery (Cal. Civ. Code § 1962(a)(1)); Massachusetts-
// 30DayWith300DollarFine (MA G.L. c. 186 § 15D + $300 fine + waiver
// void); Texas3BusinessDayDelivery (Tex. Prop. Code § 92.024);
// NoStateLeaseCopyDeliveryDeadline (47 other states + DC).
// ---------------------------------------------------------------------------

async fn lease_copy_delivery_check_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<LeaseCopyDeliveryInput>,
) -> Result<Json<LeaseCopyDeliveryResult>, ApiError> {
    if b.state_code.trim().is_empty() {
        return Err(ApiError::BadRequest("state_code required".into()));
    }
    Ok(Json(check_lease_copy_delivery(&b)))
}

// ---------------------------------------------------------------------------
// State tenant right-to-organize / tenant association protection check
//
// Mounted at POST /api/rental/tenant-organizing-check. Five regimes:
// NewYorkAffirmativeRoomAccess (NY RPL § 230 — landlord must provide
// common-room meeting space at NO COST); DistrictColumbiaStrongCivil-
// Penalty (DC § 42-3505.06 — $10,000 per-violation penalty + business
// license suspension + attorney fees); NewJerseyOrganizerProtection
// (N.J.S.A. 2A:42-10.10 — reprisal-against-organizer damages action);
// CaliforniaRetaliatoryEvictionDefense (Cal. Civ. Code § 1942.5(d) —
// 180-day protected window); NoStatewideTenantOrganizingProtection
// (46 other states; general anti-retaliation may apply).
// ---------------------------------------------------------------------------

async fn tenant_organizing_check_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<TenantOrganizingInput>,
) -> Result<Json<TenantOrganizingResult>, ApiError> {
    if b.state_code.trim().is_empty() {
        return Err(ApiError::BadRequest("state_code required".into()));
    }
    Ok(Json(check_tenant_organizing(&b)))
}

// ---------------------------------------------------------------------------
// Tenant relocation assistance — landlord-paid relocation amount owed when
// a no-fault eviction or qualifying displacement event occurs.
//
// Mounted at POST /api/rental/tenant-relocation-assistance. Five regimes:
// CaliforniaAb1482 (Cal. Civ. Code § 1946.2(d)(3) — one month rent within
// 15 days of notice, for owner move-in / withdrawal / demo / gov order;
// strict-compliance voids non-compliant notice); PortlandOr (PCC 30.01.085
// — graduated by bedrooms $2,900 / $3,300 / $4,200 / $4,500 for studio /
// 1BR / 2BR / 3BR+; triggered by no-cause termination, qualifying landlord-
// cause termination, or rent increase >=10%; 90-day notice required;
// penalty up to 3x rent + actual damages + attorney fees); SeattleTrao
// (SMC Ch. 22.210 — $5,552 split $2,776 landlord / $2,776 City; demo /
// substantial-rehab / change-of-use triggers; <=50% AMI households only);
// SeattleEdra (SMC Ch. 22.212 eff. 2022-07-01 — 3x monthly housing cost;
// rent-increase >=10% in rolling 12-month window; <=80% AMI households
// only; City advances payment, landlord reimburses); Default (no statewide
// statute, local ordinance may apply).
// ---------------------------------------------------------------------------

async fn tenant_relocation_assistance_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<TenantRelocationInput>,
) -> Result<Json<TenantRelocationResult>, ApiError> {
    if b.monthly_rent_cents < 0 {
        return Err(ApiError::BadRequest(
            "monthly_rent_cents must be non-negative".into(),
        ));
    }
    if b.household_ami_percent > 1_000 {
        return Err(ApiError::BadRequest(
            "household_ami_percent looks invalid (>1000)".into(),
        ));
    }
    Ok(Json(compute_tenant_relocation_assistance(&b)))
}

// ---------------------------------------------------------------------------
// Fair-chance-in-housing — landlord criminal-background-check restrictions.
//
// Mounted at POST /api/rental/fair-chance-housing. Four regimes:
// NewJerseyFcha (N.J.S.A. 46:8-52 et seq. eff. 2022-01-01 — no inquiry
// before conditional offer, individualized assessment required to
// withdraw, 30-day appeal window; barred categories: arrests, pending
// cases, sealed/expunged, juvenile adjudications); NewYorkCityFchha (NYC
// Admin Code § 8-107.1 / Local Law 24 of 2024 eff. 2025-01-01 — felony
// 5-year / misdemeanor 3-year lookback from release or sentencing; sex-
// offender-registry convictions always considerable; barred categories:
// arrests, pending cases, ACDs, juvenile adjudications, sealed/expunged,
// non-criminal-violation convictions); CaliforniaFeha (Cal. Civ. Code §
// 1786.18 + 2 Cal. Code Regs. § 12266 — no blanket bans, individualized
// assessment required considering nature/severity, time elapsed, and
// rehabilitation evidence); Default (15 U.S.C. § 1681c FCRA 7-year ceiling
// on CRA arrest reports, conviction reports have no federal time limit).
// ---------------------------------------------------------------------------

async fn fair_chance_housing_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<FairChanceInput>,
) -> Result<Json<FairChanceResult>, ApiError> {
    if b.years_since_release > 1_000 {
        return Err(ApiError::BadRequest(
            "years_since_release looks invalid (>1000)".into(),
        ));
    }
    Ok(Json(check_fair_chance_housing(&b)))
}

// ---------------------------------------------------------------------------
// Methamphetamine contamination landlord disclosure compliance check.
//
// Mounted at POST /api/rental/meth-contamination-disclosure. Four regimes:
// Colorado (Colo. Rev. Stat. § 38-35.7-103 + 6 CCR 1014-3 — 0.5 ug/100cm²
// standard; remediation EXTINGUISHES disclosure obligation when certified
// to state); Arizona (Ariz. Rev. Stat. § 32-1166.04 — 0.1 ug/100cm² standard
// STRICTEST; remediation extinguishes disclosure; entry by non-owners
// barred until cleaned to state standard); Montana (Mont. Code Ann. §
// 75-10-1301 et seq. — 1.5 ug/100cm² standard; remediation does NOT
// extinguish disclosure obligation — landlord MUST disclose knowledge +
// remediation status even after cleanup); Default (no statewide statute;
// 42 U.S.C. § 3604 FHA material-defect doctrine may impose duty).
// ---------------------------------------------------------------------------

async fn meth_contamination_disclosure_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<MethDisclosureInput>,
) -> Result<Json<MethDisclosureResult>, ApiError> {
    Ok(Json(check_meth_contamination_disclosure(&b)))
}

// ---------------------------------------------------------------------------
// Death-in-unit landlord disclosure compliance check.
//
// Mounted at POST /api/rental/death-in-unit-disclosure. Four regimes:
// California1710_2 (Cal. Civ. Code § 1710.2(a) — 3-year (36-month)
// disclosure window for ALL deaths including natural causes; HIV/AIDS
// carve-out per § 1710.2(a)(1); § 1710.2(b) intentional-misrepresentation
// override on direct inquiry — a lie is actionable regardless of window);
// SouthDakota (S.D. Codified Laws § 43-4-44 — 12-month disclosure window
// for homicides/suicides/felonies only; natural deaths not covered);
// Alaska (AS 08.88.615 — 12-month real-estate-agent disclosure window for
// known murders and suicides; agent disclosure not owner disclosure);
// Default (no statewide statute; common-law caveat emptor; direct-inquiry
// misrepresentation may still be actionable under general fraud).
// ---------------------------------------------------------------------------

async fn death_in_unit_disclosure_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<DeathDisclosureInput>,
) -> Result<Json<DeathDisclosureResult>, ApiError> {
    if b.months_since_death > 100_000 {
        return Err(ApiError::BadRequest(
            "months_since_death looks invalid (>100000)".into(),
        ));
    }
    Ok(Json(check_death_in_unit_disclosure(&b)))
}

// ---------------------------------------------------------------------------
// State rent-payment-method compliance — what methods landlord must accept
// and which may not be required.
//
// Mounted at POST /api/rental/rent-payment-method. Three regimes:
// California1947_3 (Cal. Civ. Code § 1947.3(a) — must allow at least one
// non-cash non-electronic method, fee-for-check prohibited; § 1947.3(c)
// 3-month cash-only carve-out after bounced check + written notice;
// § 1947.3(d) waiver void as public policy); NewYork235G (NY RPP § 235-g(1)
// — electronic-only requirement prohibited; § 235-g(2) no fee for non-
// electronic payers; § 235-g(4) waiver void); Default (no statewide
// statute — lease terms control). Distinct from advance_rent_limit
// (which caps the AMOUNT collectable in advance) — this module addresses
// HOW rent may be paid.
// ---------------------------------------------------------------------------

async fn rent_payment_method_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentPaymentMethodInput>,
) -> Result<Json<RentPaymentMethodResult>, ApiError> {
    if b.months_since_bounced_check > 1_000 {
        return Err(ApiError::BadRequest(
            "months_since_bounced_check looks invalid (>1000)".into(),
        ));
    }
    Ok(Json(check_rent_payment_method(&b)))
}

// ---------------------------------------------------------------------------
// State window-guard landlord compliance check.
//
// Mounted at POST /api/rental/window-guard-requirements. Three regimes:
// NewYorkCity (NYC Admin Code § 27-2043.1 + NYC Health Code § 131.15 —
// PROACTIVE/MANDATORY model: 3+ unit buildings must install approved
// guards in every unit where child ≤10 resides AND public-hallway
// windows; annual Jan 1-15 notice to ALL tenants required; landlord
// bears 100% of cost; specs ≥ 15" tall ≤ 4.5" bar spacing; first-floor
// emergency-exit exception) + NewJersey (N.J.S.A. 55:13A-7.13/7.14 —
// REACTIVE/ON-REQUEST model: written tenant request required; lease must
// contain conspicuous bold-face notice per § 7.14; up to $20/guard cost
// pass-through allowed per § 7.13(b); biannual maintenance inspection
// required) + Default (no statewide statute — common-law habitability).
// ---------------------------------------------------------------------------

async fn window_guard_requirements_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<WindowGuardInput>,
) -> Result<Json<WindowGuardResult>, ApiError> {
    if b.cost_passthrough_per_guard_dollars > 10_000 {
        return Err(ApiError::BadRequest(
            "cost_passthrough_per_guard_dollars looks invalid (>10000)".into(),
        ));
    }
    Ok(Json(check_window_guard_requirements(&b)))
}

// ---------------------------------------------------------------------------
// State rent-increase notice-period landlord compliance check.
//
// Mounted at POST /api/rental/rent-increase-notice-period. Four regimes:
// CaliforniaAb1482 (Cal. Civ. Code § 827(b)(1) 30 days for ≤10% increase
// + § 827(b)(3) 90 days for >10% strict-greater-than two-tier;
// Civ. Code § 1947.12 AB 1482 rent cap caps most increases at 5%+CPI
// or 10% whichever lower making 90-day tier uncommon); Washington (RCW
// 59.18.140 amended May 2025 — uniform 90-day notice with carve-out for
// subsidized tenancies 30 days; RCW 59.18.720 7%+CPI or 10% cap out-of-
// scope); Oregon (ORS 90.323 90-day notice + first-year prohibition no
// increase in first 12 months of non-week-to-week tenancy + once-per-
// 12-month rule); Default (no statewide statute — lease terms control).
// Distinct from late_payment_grace_period (tenant late payment) and
// advance_rent_limit (amount cap).
// ---------------------------------------------------------------------------

async fn rent_increase_notice_period_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentIncreaseNoticeInput>,
) -> Result<Json<RentIncreaseNoticeResult>, ApiError> {
    if b.increase_basis_points > 100_000 {
        return Err(ApiError::BadRequest(
            "increase_basis_points looks invalid (>100000 = 1000%)".into(),
        ));
    }
    Ok(Json(check_rent_increase_notice_period(&b)))
}

// ---------------------------------------------------------------------------
// State demolition-tenant-notice landlord compliance check.
//
// Mounted at POST /api/rental/demolition-tenant-notice. Four regimes:
// CaliforniaEllisAct (Cal. Govt Code § 7060.4(a) 120-day notice for
// standard tenants + § 7060.4(b) 365-day extension for tenants ≥ 62 OR
// disabled AND ≥ 1 year residency); Oregon (ORS 90.427 90-day landlord-
// cause termination + ORS 90.323(3) first-year prohibition no termination
// in first 12 months); Washington (RCW 59.18.650 120-day notice for
// substantial-rehab / change-of-use / demolition); Default (no statewide
// statute, lease + just-cause-eviction control). Distinct from
// owner_move_in_eviction (landlord MOVES IN to unit) and
// tenant_relocation_assistance (DOLLAR amount owed).
// ---------------------------------------------------------------------------

async fn demolition_tenant_notice_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<DemolitionNoticeInput>,
) -> Result<Json<DemolitionNoticeResult>, ApiError> {
    if b.tenant_age > 150 {
        return Err(ApiError::BadRequest(
            "tenant_age looks invalid (>150)".into(),
        ));
    }
    Ok(Json(check_demolition_tenant_notice(&b)))
}

// ---------------------------------------------------------------------------
// State/municipal eviction-diversion-program landlord pre-filing
// mediation compliance check.
//
// Mounted at POST /api/rental/eviction-diversion-program. Three regimes:
// Philadelphia (Phila Code § 9-811 — landlord must apply for and be
// approved for the EDP + participate in good faith ≥ 30 days + provide
// tenant notice; imminent-physical-harm carve-out; applies to virtually
// all eviction grounds since 2022 amendments; tenant defense + sua
// sponte dismissal for noncompliance; non-waivable); NewJersey (NJ DCA
// Eviction Diversion Program — enroll + 14-day notice of mediation
// right to tenant + dispute resolution center + 45-day wait IF tenant
// schedules mediation + good-faith participation; tenant-default carve-
// out: if tenant doesn't schedule within 14 days landlord may proceed);
// Default (no mandatory pre-filing mediation, court-level voluntary
// mediation may exist). Distinct from eviction_notices (notice period
// landlord must give tenant before filing).
// ---------------------------------------------------------------------------

async fn eviction_diversion_program_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<DiversionProgramInput>,
) -> Result<Json<DiversionProgramResult>, ApiError> {
    if b.days_since_enrollment > 100_000 {
        return Err(ApiError::BadRequest(
            "days_since_enrollment looks invalid (>100000)".into(),
        ));
    }
    Ok(Json(check_eviction_diversion_program(&b)))
}

// ---------------------------------------------------------------------------
// State immigration-status tenant protection landlord compliance check.
//
// Mounted at POST /api/rental/immigration-status-protection. Three regimes:
// CaliforniaAb291 (Cal. Civ. Code §§ 1940.05/1940.2/1940.3/1942.5 — AB 291
// 2017 — prohibits disclosure to immigration/law enforcement + threats of
// disclosure + eviction based on status + retaliation; § 1940.3 CA-only
// application-inquiry prong; judicial-warrant carve-out under § 1940.2
// permits disclosure under judge-signed warrant or subpoena in criminal
// investigation; $2,000 per-violation civil penalty + AG/DA criminal
// prosecution + actual damages + attorney fees); Illinois (765 ILCS 755/
// Immigrant Tenant Protection Act eff. 2019-08-23 — prohibits disclosure
// + threats + eviction + retaliation based on status; NO application-
// inquiry prong; NO judicial-warrant carve-out; NO criminal prosecution;
// $2,000 per-violation + actual damages + attorney fees + equitable
// relief); Default (no statewide statute; 42 U.S.C. § 3604 FHA national-
// origin discrimination may apply but does NOT specifically address
// immigration-status threats).
// ---------------------------------------------------------------------------

async fn immigration_status_protection_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<ImmigrationProtectionInput>,
) -> Result<Json<ImmigrationProtectionResult>, ApiError> {
    Ok(Json(check_immigration_status_protection(&b)))
}

// ---------------------------------------------------------------------------
// State prevailing-party attorney-fees lease-clause compliance check.
//
// Mounted at POST /api/rental/prevailing-party-attorney-fees. Three regimes:
// CaliforniaCivCode1717 (Cal. Civ. Code § 1717(a) — any contract clause
// awarding attorney fees to ONE party OR the prevailing party is
// transformed by operation of law into a MUTUAL right; prevailing party
// recovers fees regardless of which party named; § 1717 is a fundamental
// CA policy that overrides choice-of-law clauses); WashingtonRcw484330
// (RCW 4.84.330 — same reciprocity rule for contracts/leases entered
// AFTER 1977-09-21; pre-1977 leases grandfathered; waiver explicitly
// PROHIBITED — any waiver clause is void); Default (American Rule —
// each party bears own fees absent a contract clause; one-way clauses
// enforced as written without reciprocity).
// ---------------------------------------------------------------------------

async fn prevailing_party_attorney_fees_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<PrevailingPartyFeesInput>,
) -> Result<Json<PrevailingPartyFeesResult>, ApiError> {
    if b.attorneys_fees_incurred_cents < 0 {
        return Err(ApiError::BadRequest(
            "attorneys_fees_incurred_cents must be non-negative".into(),
        ));
    }
    Ok(Json(check_prevailing_party_attorney_fees(&b)))
}

// ---------------------------------------------------------------------------
// State abandoned-tenant-personal-property landlord compliance check.
//
// Mounted at POST /api/rental/abandoned-property-handling. Four regimes:
// California (Cal. Civ. Code §§ 1980-1991 — § 1983 Notice of Right to
// Reclaim Abandoned Property; § 1984 15-day personal-delivery / 18-day
// first-class-mail clock; § 1988 $700 threshold — under $700 landlord
// may keep or dispose, at or above $700 MUST conduct public auction with
// proceeds less storage/auction expenses going to county treasury);
// Texas (Tex. Prop. Code §§ 54.044 + 92.0081 — § 54.044 broad authority
// to remove contents on abandonment; § 92.0081 30-day notice by BOTH
// first-class mail AND certified mail return-receipt required before
// sale); Washington (RCW 59.18.310 — 45-day waiting period when total
// value ≥ $250 OR property includes personal papers/family pictures/
// keepsakes; 7-day waiting period when value < $250 AND no keepsakes
// — keepsake carve-out forces 45-day rule regardless of value);
// Default (common-law abandonment with reasonable notice + disposal).
// Distinct from tenant_abandonment (when tenant has vacated and landlord
// may declare unit abandoned) — this module addresses BELONGINGS.
// ---------------------------------------------------------------------------

async fn abandoned_property_handling_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<AbandonedPropertyInput>,
) -> Result<Json<AbandonedPropertyResult>, ApiError> {
    if b.total_property_value_cents < 0 {
        return Err(ApiError::BadRequest(
            "total_property_value_cents must be non-negative".into(),
        ));
    }
    Ok(Json(check_abandoned_property_handling(&b)))
}

// ---------------------------------------------------------------------------
// State/municipal right-to-counsel eviction landlord notice compliance.
//
// Mounted at POST /api/rental/right-to-counsel-eviction. Three regimes:
// NewYorkCity (NYC Admin Code § 26-1301 + Local Law 136 of 2017 —
// FIRST-IN-THE-NATION tenant RTC; 200% FPL income threshold for full
// representation; BRIEF SERVICES AVAILABLE TO ALL tenants regardless of
// income; landlord must include RTC notice in eviction petition; applies
// to Housing Court + NYCHA administrative proceedings only); Washington
// (RCW 59.18.640 / RCW 59.18.057 + SB 5160 eff. 2021 — FIRST STATEWIDE
// tenant RTC; 200% FPL income threshold OR public-assistance receipt as
// alternative eligibility path; 14-day pay-or-quit notice must contain
// SPECIFIC STATUTORY FORM LANGUAGE about legal aid + dispute resolution
// centers + appointed counsel); Default (no statewide or municipal RTC;
// tenant must self-represent or retain private counsel). Distinct from
// eviction_diversion_program (pre-filing mediation duty) — this module
// addresses the right to counsel DURING the eviction proceeding itself.
// ---------------------------------------------------------------------------

async fn right_to_counsel_eviction_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RightToCounselInput>,
) -> Result<Json<RightToCounselResult>, ApiError> {
    if b.tenant_household_income_cents < 0 || b.federal_poverty_line_cents < 0 {
        return Err(ApiError::BadRequest(
            "non-negative cents inputs required".into(),
        ));
    }
    Ok(Json(check_right_to_counsel_eviction(&b)))
}

// ---------------------------------------------------------------------------
// State tenant-cannabis-use protection landlord compliance check.
//
// Mounted at POST /api/rental/tenant-cannabis-use-protection. Three
// regimes: NewYorkCannabisLaw134 (NY Cannabis Law § 134 — landlord may
// NOT refuse to rent based on tenant's cannabis use; MAY ban smoking +
// vaporizing + cultivation in lease generally; MUST permit registered
// medical cannabis patient consumption including smoking + vaping;
// FEDERAL-BENEFITS EXCEPTION permits restriction of medical cannabis
// when needed to preserve Section 8 / HUD subsidies); IllinoisCrta (410
// ILCS 705/ effective 2020-01-01 — landlord MAY prohibit cannabis
// smoking + vaporizing + cultivation INCLUDING MEDICAL via lease;
// tenant breach may lead to eviction; medical patients may have
// separate FHA reasonable-accommodation claim under federal law but
// CRTA does not affirmatively protect); Default (no state-specific
// tenant-cannabis-protection statute identified — landlord may
// prohibit via lease; federal FHA reasonable-accommodation analysis
// may apply for medical use).
// ---------------------------------------------------------------------------

async fn tenant_cannabis_use_protection_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<CannabisProtectionInput>,
) -> Result<Json<CannabisProtectionResult>, ApiError> {
    Ok(Json(check_tenant_cannabis_use_protection(&b)))
}

// ---------------------------------------------------------------------------
// State snow/ice removal landlord responsibility compliance check.
//
// Mounted at POST /api/rental/snow-removal-responsibility. Four regimes:
// Massachusetts (Papadopoulos v. Target Corp. 457 Mass. 368 (2010) +
// State Sanitary Code 105 CMR 410.452 — natural-vs-unnatural accumulation
// distinction abolished; landlord owes reasonable-care duty; primary
// means-of-egress duty cannot be delegated to tenant via lease unless
// tenant has independent private entrance and unit is not multi-unit);
// Illinois (745 ILCS 75/ Snow and Ice Removal Act enacted 1979 — IMMUNITY
// for residential owners who voluntarily clear PUBLIC SIDEWALK unless
// willful or wanton; immunity does NOT extend to private property
// driveway/walkway/garage); NewYorkCity (NYC Admin Code § 16-123 — 4-hour
// removal window after snow stops except 9pm-7am; multi-unit landlord
// fully responsible; single-family lease may delegate but owner still
// gets ticket); Default (common-law habitability + premises-liability
// rules vary by jurisdiction).
// ---------------------------------------------------------------------------

async fn snow_removal_responsibility_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<SnowRemovalInput>,
) -> Result<Json<SnowRemovalResult>, ApiError> {
    if b.hours_since_snow_stopped > 100_000 {
        return Err(ApiError::BadRequest(
            "hours_since_snow_stopped looks invalid (>100000)".into(),
        ));
    }
    Ok(Json(check_snow_removal_responsibility(&b)))
}

// ---------------------------------------------------------------------------
// State security-camera and surveillance landlord disclosure / consent
// compliance check.
//
// Mounted at POST /api/rental/security-camera-disclosure. Four regimes:
// California (Cal. Penal Code § 632 — 2-PARTY consent for audio recording
// of confidential communications; $2,500 per-violation civil penalty +
// criminal exposure up to 1 year county jail or state prison; video
// allowed in common areas but never inside private unit); NewYork (NY
// Civil Rights Law § 52-a + NY Penal Law § 250.00 — 1-PARTY consent for
// audio; video allowed in lobbies/elevators/laundry/mailrooms; Civil
// Rights Law § 52-a creates private right of action for backyard
// recreational video without written consent); Texas (Tex. Penal Code
// § 16.02 — 1-PARTY consent for audio; most landlord-friendly regime);
// Default (18 U.S.C. § 2511 federal Wiretap Act 1-party consent
// baseline; video inside private unit always barred under reasonable-
// expectation-of-privacy doctrine).
// ---------------------------------------------------------------------------

async fn security_camera_disclosure_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<SecurityCameraInput>,
) -> Result<Json<SecurityCameraResult>, ApiError> {
    Ok(Json(check_security_camera_disclosure(&b)))
}

// ---------------------------------------------------------------------------
// State carpet-replacement useful-life landlord security-deposit
// deduction compliance check.
//
// Mounted at POST /api/rental/carpet-replacement-useful-life. Four
// regimes: ColoradoHb251249 (Colo. Rev. Stat. § 38-12-104 as amended
// by HB 25-1249 eff. 2026-01-01 — STATUTORY 10-year carpet useful
// life; landlord may not deem carpet substantially+irreparably damaged
// unless replaced new within 10-year window; may retain only minimum
// necessary amount); California (common-law Killough v. McManus 8-year
// useful life); HudSection8 (HUD Handbook 4350.3 chap. 6 — 7-year
// carpet useful life for federally-subsidized housing); Default
// (common-law actual-damages-net-of-depreciation doctrine, 7-year
// proxy). Section 8 voucher overrides state routing.
// ---------------------------------------------------------------------------

async fn carpet_replacement_useful_life_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<CarpetReplacementInput>,
) -> Result<Json<CarpetReplacementResult>, ApiError> {
    if b.replacement_cost_cents < 0 {
        return Err(ApiError::BadRequest(
            "replacement_cost_cents must be non-negative".into(),
        ));
    }
    if b.carpet_age_years > 100 {
        return Err(ApiError::BadRequest(
            "carpet_age_years looks invalid (>100)".into(),
        ));
    }
    Ok(Json(check_carpet_replacement_useful_life(&b)))
}

// ---------------------------------------------------------------------------
// State pre-move-out inspection landlord compliance check.
//
// Mounted at POST /api/rental/pre-move-out-inspection. Two regimes:
// CaliforniaCivCode19505F (Cal. Civ. Code § 1950.5(f) — landlord MUST
// notify tenant in writing of right to request initial inspection AND
// right to be present; § 1950.5(f)(2) inspection at reasonable time
// but NO EARLIER THAN 2 WEEKS (14 days) before termination/end-of-lease
// date; § 1950.5(f)(3) landlord must provide itemized statement of
// proposed deductions; tenant has cure period until termination to
// remedy identified deficiencies; § 1950.5(f)(4) WAIVER: if premises
// were clear of tenant possessions at inspection AND landlord
// conducted inspection AND provided itemized statement, landlord
// SHALL NOT use security deposit for deductions NOT identified in the
// itemized statement) + Default (no statewide pre-move-out inspection
// statute; landlord conducts post-move-out inspection only; state-
// specific security-deposit itemization timelines apply on back-end).
// Distinct from move_in_inspection (START-of-tenancy condition).
// ---------------------------------------------------------------------------

async fn pre_move_out_inspection_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<PreMoveOutInspectionInput>,
) -> Result<Json<PreMoveOutInspectionResult>, ApiError> {
    if b.days_between_inspection_and_termination > 100_000 {
        return Err(ApiError::BadRequest(
            "days_between_inspection_and_termination looks invalid (>100000)".into(),
        ));
    }
    Ok(Json(check_pre_move_out_inspection(&b)))
}

// ---------------------------------------------------------------------------
// State credit-check authorization and disclosure landlord compliance.
//
// Mounted at POST /api/rental/credit-check-authorization. Three regimes:
// Washington (RCW 59.18.257 — four-prong pre-screening disclosure
// requirement: types of information accessed + denial criteria + CRA
// name/address/tenant rights + reusable-screening-report acceptance;
// cost-recovery permitted only if disclosure provided; $100/violation
// + attorney fees + court costs); California (Cal. Civ. Code § 1950.6
// — application-screening fee cap ~$60 inflation-adjusted in 2024;
// itemized receipt required upon request; unused-portion refund
// required; $100 civil penalty); Default (15 U.S.C. § 1681b(a)(3)(F)
// FCRA tenant-screening permissible purpose baseline; no written
// authorization required for tenant screening — § 1681b(b) employment-
// purpose only). Distinct from adverse_action_notice (post-denial
// notice AFTER the report is used) and application_fees (fee dollar cap).
// ---------------------------------------------------------------------------

async fn credit_check_authorization_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<CreditCheckInput>,
) -> Result<Json<CreditCheckResult>, ApiError> {
    if b.screening_fee_charged_cents < 0 {
        return Err(ApiError::BadRequest(
            "screening_fee_charged_cents must be non-negative".into(),
        ));
    }
    Ok(Json(check_credit_check_authorization(&b)))
}

// ---------------------------------------------------------------------------
// State/municipal winter and weather-based eviction protection check.
//
// Mounted at POST /api/rental/winter-eviction-protections. Three regimes:
// DistrictOfColumbia (DC Code § 42-3505.01(k) — strongest statutory
// weather-based restriction; no eviction when NWS predicts at 8 AM that
// Reagan National Airport temperature will fall BELOW 32°F (§ k(1))
// OR rise ABOVE 95°F (§ k(3)) OR precipitation is falling at the
// rental unit location (§ k(2))); CookCountyIllinois (Cook County
// Sheriff Order — sheriff will NOT execute eviction orders when
// temperature is 15°F or COLDER OR extreme weather endangers safety
// OR within annual holiday moratorium Dec 19 - Jan 5); Default (no
// statutory restriction; court equitable discretion only). Distinct
// from snow_removal_responsibility (landlord duty to clear) and
// heat_requirements (habitability heat minimums).
// ---------------------------------------------------------------------------

async fn winter_eviction_protections_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<WinterEvictionInput>,
) -> Result<Json<WinterEvictionResult>, ApiError> {
    if !(-100..=150).contains(&b.nws_predicted_temp_at_8am_f) {
        return Err(ApiError::BadRequest(
            "nws_predicted_temp_at_8am_f must be in [-100, 150]".into(),
        ));
    }
    Ok(Json(check_winter_eviction_protections(&b)))
}

// ---------------------------------------------------------------------------
// State landlord-identification / emergency-contact-information
// disclosure compliance check.
//
// Mounted at POST /api/rental/landlord-identification-disclosure. Four
// regimes: California (Cal. Civ. Code § 1962 — name + telephone +
// street address + entity for rent payments; 15-day deadline; STRICT
// COMPLIANCE — JURISDICTIONAL PREREQUISITE to unlawful detainer);
// NewJersey (N.J.S.A. 46:8-27 through 46:8-37 Landlord Identity Law —
// register with municipal clerk within 30 days + supply registration
// info to each tenant; NJ-only EMERGENCY-CONTACT requirement —
// representative of owner or managing agent who may be contacted in
// case of emergency); Washington (RCW 59.18.060 — name + address of
// manager + owner / authorized agent; no phone or emergency-contact
// requirement); Default (common-law tenant right to identify landlord).
// ---------------------------------------------------------------------------

async fn landlord_identification_disclosure_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<LandlordIdentificationInput>,
) -> Result<Json<LandlordIdentificationResult>, ApiError> {
    if b.days_since_tenancy_created > 100_000 {
        return Err(ApiError::BadRequest(
            "days_since_tenancy_created looks invalid (>100000)".into(),
        ));
    }
    Ok(Json(check_landlord_identification_disclosure(&b)))
}

// ---------------------------------------------------------------------------
// Federal FHA + state reasonable accommodation / modification
// compliance check for tenants with disabilities.
//
// Mounted at POST /api/rental/reasonable-accommodation-modification. Four
// regimes: Federal (42 U.S.C. § 3604(f)(3)(A) modification at tenant's
// expense + § 3604(f)(3)(B) accommodation in rules/policies/practices/
// services at landlord's expense unless undue burden or fundamental
// alteration); California (Cal. Civ. Code § 54.1(b)(3)(A)–(B) — mirrors
// FHA modification + restoration agreement permitted + NO ADDITIONAL
// SECURITY + escrow capped at reasonable estimate of restoration cost;
// § 54(b) sensory-disability protections broader than ADA); NYC (N.Y.C.
// Admin. Code § 8-107(15)(c) MANDATORY cooperative-dialogue requirement
// — landlord MUST engage in documented written/oral dialogue with
// tenant; failure to engage is itself a discriminatory practice
// independent of substantive outcome; § 8-102 defines dialogue);
// Washington (RCW 49.60.222(2)(b) — mirrors FHA cost allocation).
// ---------------------------------------------------------------------------

async fn reasonable_accommodation_modification_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<ReasonableAccommodationInput>,
) -> Result<Json<ReasonableAccommodationResult>, ApiError> {
    if b.modification_cost_cents < 0
        || b.escrow_amount_cents < 0
        || b.restoration_estimate_cents < 0
    {
        return Err(ApiError::BadRequest(
            "non-negative cents inputs required".into(),
        ));
    }
    if b.days_since_request_received > 100_000 {
        return Err(ApiError::BadRequest(
            "days_since_request_received looks invalid (>100000)".into(),
        ));
    }
    Ok(Json(check_reasonable_accommodation_modification(&b)))
}

// ---------------------------------------------------------------------------
// State security-deposit damage-deduction itemization compliance check.
//
// Mounted at POST /api/rental/damage-deduction-itemization. Five
// regimes: California (Cal. Civ. Code § 1950.5(g)(1) 21-day deadline
// + § 1950.5(g)(2) $125 receipt/invoice threshold + § 1950.5(g)(3)(A)
// AB 2801 photographic-documentation mandate eff. 2025-04-01 for all
// tenancies and 2025-07-01 for pre-tenancy shots + § 1950.5(b)(2)
// ordinary-wear-and-tear exclusion + § 1950.5(l) bad-faith up to 2×
// withheld); Washington (RCW 59.18.280 30-day full-and-specific
// statement; intentional refusal up to 2× deposit); Oregon (ORS
// 90.300(13) 31-day itemized + statutory depreciation); Florida
// (Fla. Stat. § 83.49(3) 30-day certified mail + $200 invoice
// threshold + depreciation required); Texas (Tex. Prop. Code § 92.103
// + § 92.104(c) + § 92.109 — STEEPEST PENALTY of $100 + 3× retained
// + reasonable attorney fees; no statutory depreciation mandate).
// ---------------------------------------------------------------------------

async fn damage_deduction_itemization_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<DamageDeductionInput>,
) -> Result<Json<DamageDeductionResult>, ApiError> {
    if b.total_deductions_cents < 0 || b.amount_withheld_cents < 0 {
        return Err(ApiError::BadRequest(
            "non-negative cents inputs required".into(),
        ));
    }
    if b.days_to_itemized_statement > 100_000 {
        return Err(ApiError::BadRequest(
            "days_to_itemized_statement looks invalid (>100000)".into(),
        ));
    }
    Ok(Json(check_damage_deduction_itemization(&b)))
}

// ---------------------------------------------------------------------------
// State + municipal landlord cooling / maximum-indoor-temperature /
// AC-installation compliance check.
//
// Mounted at POST /api/rental/cooling-requirements. Six regimes:
// ArizonaPhoenix (A.R.S. § 33-1364 cooling-as-essential-service + 5-
// day cure period + Phoenix City Code 82°F refrigerated AC max / 86°F
// evaporative cooler max in habitable rooms); ArizonaTucson (parallel
// 82°F cap); Dallas (Dallas City Code Chapter 27 refrigerated air —
// indoor ≤ outdoor − 20°F during April 1 to November 1; Tex. Prop.
// Code § 92.052 7-day repair window); NYCCoolHomes (N.Y.C. Admin.
// Code Cool Homes for All Int 0994-2024 — tenant request begins
// 2028-03-01; enforcement begins 2030-01-01; bedrooms ≤ 78°F when
// outdoor > 82°F during June 15 to September 15; 60-day landlord
// response window for installation requests); California (Cal. Civ.
// Code § 1941.1 implied warranty covers existing cooling — no
// statewide maximum-temperature cap or install mandate); Default (no
// statewide cooling requirement).
// ---------------------------------------------------------------------------

async fn cooling_requirements_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<CoolingRequirementsInput>,
) -> Result<Json<CoolingRequirementsResult>, ApiError> {
    if b.day_of_year == 0 || b.day_of_year > 366 {
        return Err(ApiError::BadRequest(
            "day_of_year must be in 1..=366".into(),
        ));
    }
    if b.days_since_written_notice > 100_000
        || b.days_since_tenant_request_for_install > 100_000
    {
        return Err(ApiError::BadRequest(
            "day-since-notice counters look invalid (>100000)".into(),
        ));
    }
    Ok(Json(check_cooling_requirements(&b)))
}

// ---------------------------------------------------------------------------
// State landlord duty-to-mitigate-damages compliance check for
// tenant lease breach / abandonment.
//
// Mounted at POST /api/rental/duty-to-mitigate-damages. Eight regimes:
// California (Cal. Civ. Code § 1951.2 statutory duty + § 1951.4
// assignment-or-subletting carve-out from the § 1951.2 duty);
// NewYork (N.Y. Real Prop. Law § 227-e HSTPA 2019 — statutory duty;
// NON-WAIVABLE; pre-HSTPA NY was a no-duty state under common law);
// Texas (Tex. Prop. Code § 91.006(a) statutory duty + § 91.006(b)
// waiver prohibition); Illinois (735 ILCS 5/9-213.1 statutory duty);
// Florida (Fla. Stat. § 83.595 CONDITIONAL — duty depends on landlord
// election among (1)(a) terminate, (1)(b) retake-and-relet REQUIRES,
// (1)(c) stand-by-collect-rent NO duty, (1)(d) sue-as-accrued
// REQUIRES); Mississippi (Alsup v. Banks 9 So. 895 Miss. 1891 —
// COMMON-LAW MINORITY RULE no duty); Georgia (unclear case law
// allows no mitigation); Default (majority common-law rule duty).
// ---------------------------------------------------------------------------

async fn duty_to_mitigate_damages_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<DutyToMitigateInput>,
) -> Result<Json<DutyToMitigateResult>, ApiError> {
    if b.original_monthly_rent_cents < 0 || b.re_rented_monthly_rent_cents < 0 {
        return Err(ApiError::BadRequest(
            "non-negative cents inputs required".into(),
        ));
    }
    if b.days_unit_remained_vacant > 100_000 || b.months_remaining_on_lease > 1_000 {
        return Err(ApiError::BadRequest(
            "day-counters look invalid".into(),
        ));
    }
    Ok(Json(check_duty_to_mitigate_damages(&b)))
}

// ---------------------------------------------------------------------------
// State landlord pesticide-application notice compliance check.
//
// Mounted at POST /api/rental/pesticide-application-notice. Six
// regimes: California (Cal. Civ. Code § 1940.8.5 + 14 CCR § 6740 —
// 24-hour written advance notice to treated tenant; adjacent units
// also for broadcast / total-release fogger / aerosol spray; notice
// content requires pesticide product + manufacturer + EPA
// registration + applicator); NewJersey (N.J.A.C. § 7:30-9.12 —
// applicator provides label info AT TIME OF APPLICATION + applicator
// contact + National Pesticide Information Center phone + NJDEP
// Pesticide Control Program phone; multi-family owner distributes on
// request); NewYork (N.Y. ECL § 33-1004 Pesticide Reporting Law /
// Neighbor Notification Law — NO statewide advance-notice mandate
// for residential rentals; one/two-family applicator provides label
// info at application; multi-dwelling applicator provides to owner
// who provides to occupants ON REQUEST); Massachusetts (Mass. G.L.
// c. 132B § 9 + 333 CMR 13.04 — 48-hour advance written notice to
// occupants + posting at building entrances); Oregon (ORS § 634.740
// — 24-hour warning-sign posting before application + 72-hour
// minimum sign retention); Default (federal FIFRA 7 U.S.C. § 136
// labeling + applicator licensing only — no statewide advance-notice
// mandate).
// ---------------------------------------------------------------------------

async fn pesticide_application_notice_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<PesticideNoticeInput>,
) -> Result<Json<PesticideNoticeResult>, ApiError> {
    if b.hours_advance_notice_to_treated_tenant > 100_000
        || b.hours_advance_notice_to_adjacent_tenants > 100_000
        || b.hours_advance_posting_at_building > 100_000
    {
        return Err(ApiError::BadRequest(
            "hour counters look invalid (>100000)".into(),
        ));
    }
    Ok(Json(check_pesticide_application_notice(&b)))
}

// ---------------------------------------------------------------------------
// State landlord condominium-conversion tenant protection
// compliance check.
//
// Mounted at POST /api/rental/condominium-conversion-protection. Six
// regimes: DistrictOfColumbia (DC Code § 42-3401.01 et seq. Rental
// Housing Conversion and Sale Act + § 42-1904.08 — owner cannot
// convert without (a) Mayor certification AND (b) MAJORITY tenant
// vote in certified election; TOPA right-of-first-refusal
// coordination); Massachusetts (G.L. c. 527 of Acts of 1983 —
// applies only to buildings of 4+ rental units; 90-day first-refusal
// grace period; relocation $750 standard / $1000 elderly + disabled
// + low-moderate-income; notice 1 year standard / 2 years
// low-moderate / 4 years elderly + disabled); NewJersey (N.J.S.A.
// 2A:18-61.22 Senior Citizens and Disabled Protected Tenancy Act —
// up to 40-year protected tenancy BARS conversion eviction of
// senior + disabled tenants); NewYork (N.Y. Gen. Bus. Law § 352-e
// + § 352-eee + § 352-eeee — EVICTION PLAN requires 51% tenant
// purchase commitment; NON-EVICTION PLAN requires 15% commitment;
// senior + disabled receive 99-year non-eviction tenure);
// MarylandMontgomery (Mont. County Code Ch. 11A + Md. Code Real
// Property § 11-102.1 — 180-day notice + right of first refusal +
// relocation assistance); Default (no statewide protection).
// ---------------------------------------------------------------------------

async fn condominium_conversion_protection_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<CondoConversionInput>,
) -> Result<Json<CondoConversionResult>, ApiError> {
    if b.relocation_assistance_paid_cents < 0 {
        return Err(ApiError::BadRequest(
            "non-negative cents inputs required".into(),
        ));
    }
    if b.building_unit_count > 100_000
        || b.notice_days_before_conversion > 100_000
        || b.days_for_first_refusal_acceptance > 100_000
        || b.ny_tenant_purchase_commitment_bp > 10_000
    {
        return Err(ApiError::BadRequest(
            "counters look invalid (>thresholds)".into(),
        ));
    }
    Ok(Json(check_condominium_conversion_protection(&b)))
}

// ---------------------------------------------------------------------------
// Federal FCC Over-the-Air Reception Devices (OTARD) rule
// compliance check for tenant antenna / satellite-dish / fixed-
// wireless-receiver installation.
//
// Mounted at POST /api/rental/otard-antenna-installation. 47 CFR
// § 1.4000 PREEMPTS state, local, HOA, and lease restrictions that
// impair a tenant's ability to install, maintain, or use a covered
// antenna in an area within the tenant's exclusive use or control.
// Five protected antenna types: DBS satellite dish (≤ 1m); MMDS
// antenna (≤ 1m); broadcast TV antenna (any size); supporting mast;
// fixed-wireless hub/relay (FCC 21-10 2021 Report and Order eff.
// 2021-03-29 — requires on-premises customer + broadband-only).
// Tenant-exclusive-use scope: patio + balcony + single-tenant
// rooftop qualify; common areas (shared rooftops + exterior walls +
// hallways) are OUTSIDE scope. Three permissible-restriction
// categories under § 1.4000(b): (1) safety; (2) historic
// preservation (National Register properties); (3) no-impairment
// standard — restriction must NOT unreasonably delay, increase
// cost, or preclude acceptable-quality signal. § 1.4000(c) burden
// of proof on restricting party. Aesthetic restrictions + blanket
// prohibitions + pre-approval delays are NEVER permissible.
// ---------------------------------------------------------------------------

async fn otard_antenna_installation_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<OtardAntennaInput>,
) -> Result<Json<OtardAntennaResult>, ApiError> {
    Ok(Json(check_otard_antenna_installation(&b)))
}

// ---------------------------------------------------------------------------
// State landlord religious-display / mezuzah-on-doorpost tenant
// right compliance check.
//
// Mounted at POST /api/rental/religious-display-doorpost. Eight
// regimes: California (Cal. Civ. Code § 1940.45 SB 652 "Mezuzah
// Bill" 2024 — broadest scope explicitly covers landlord-tenant +
// dormitory rooms + apartments); Texas (Tex. Prop. Code § 202.018
// HOA-focused with rental coverage via restrictive-covenant
// analysis); Florida (Fla. Stat. § 720.3045 HOA-focused with
// explicit tenant extension); Illinois (765 ILCS 605/18.4 Mezuzah
// Law condominium-focused); Connecticut (Conn. Gen. Stat.
// § 47-230a Common Interest Ownership Act); RhodeIsland (R.I. Gen.
// Laws § 34-36.1-3.18 parallel); NewYork (NO enacted state
// statute — S4466 proposed only; FHA fallback); Default (42
// U.S.C. § 3604(b) Fair Housing Act religious-discrimination
// federal floor + Bloch v. Frischholz 587 F.3d 771 (7th Cir. 2009)
// (en banc) precedent).
// ---------------------------------------------------------------------------

async fn religious_display_doorpost_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<ReligiousDisplayInput>,
) -> Result<Json<ReligiousDisplayResult>, ApiError> {
    if b.item_size_inches > 10_000 {
        return Err(ApiError::BadRequest(
            "item_size_inches looks invalid".into(),
        ));
    }
    Ok(Json(check_religious_display_doorpost(&b)))
}

// ---------------------------------------------------------------------------
// State landlord asbestos disclosure compliance check.
//
// Mounted at POST /api/rental/asbestos-disclosure. Six regimes:
// California (Cal. Health & Safety Code §§ 25915-25919.7 Connelly-
// Areias-Chacon Asbestos Notification Act 1989 — buildings
// constructed BEFORE 1979; 15-day notification deadline from
// knowledge; ANNUAL re-notification required; $500/day penalty per
// § 25917); NewJersey (N.J.A.C. 5:23-8 construction regulations
// without separate lease-signing disclosure mandate); NewYork (NY
// Multiple Dwelling Law habitability + N.Y. Industrial Code Rule 56
// abatement; no state lease-signing mandate); FederalOSHA (29 CFR
// § 1926.1101(k)(2) construction standard — building owner must
// notify tenants/employers/contractors of PACM/ACM presence when
// construction work is planned in occupied areas; commercial /
// multi-tenant only — single-family residences OUTSIDE scope; PACM
// presumption for buildings built before 1981); FederalAHERA (40
// CFR Part 763 + 15 U.S.C. §§ 2641-2671 — covers public + private
// nonprofit schools only; does NOT cover landlord-tenant rentals);
// Default (federal OSHA construction-phase floor + state
// habitability obligation; no specific lease-signing landlord
// disclosure mandate).
// ---------------------------------------------------------------------------

async fn asbestos_disclosure_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<AsbestosDisclosureInput>,
) -> Result<Json<AsbestosDisclosureResult>, ApiError> {
    if b.construction_year > 10_000 || b.days_since_knowledge_obtained > 100_000 {
        return Err(ApiError::BadRequest(
            "counters look invalid (>thresholds)".into(),
        ));
    }
    Ok(Json(check_asbestos_disclosure(&b)))
}

// ---------------------------------------------------------------------------
// State landlord firearms-in-rental-unit tenant right compliance
// check.
//
// Mounted at POST /api/rental/firearms-in-rental-unit. Six regimes:
// Minnesota (Minn. Stat. § 504B.211 + Chapter 624 — strongest pro-
// tenant statutory protection — landlord cannot restrict lawful
// possession + carry + transportation of firearms by tenants or
// guests in rental unit); Virginia (Va. Code § 55.1-1208(A)(15) —
// PUBLIC HOUSING only prohibition on rental agreement firearms
// restriction; private landlords may still restrict by lease);
// Tennessee (current state law permits private landlord to prohibit
// firearms via lease clause; SB0350 in 2026 session proposes flip
// to pro-tenant — audit tracks current law); Wisconsin (Wis. Stat.
// § 175.60 Concealed Carry Licensee Protections — protects all
// occupants of rented dwelling where concealed-carry licensee
// lives); NewYork (state silent on private landlord restriction;
// federal court 2024 permanent injunction (Cortland County public-
// housing handgun ban) under N.Y. State Rifle & Pistol Ass'n v.
// Bruen, 597 U.S. 1 (2022) protects public-housing tenants;
// private landlords may still restrict by lease); Default (state
// silent; private landlord may restrict via lease; federal Bruen
// floor applies only to government action = public housing).
// ---------------------------------------------------------------------------

async fn firearms_in_rental_unit_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<FirearmsRentalInput>,
) -> Result<Json<FirearmsRentalResult>, ApiError> {
    Ok(Json(check_firearms_in_rental_unit(&b)))
}

// ---------------------------------------------------------------------------
// State landlord lock-change / rekey / security-device compliance
// check for new-tenant turnover.
//
// Mounted at POST /api/rental/lock-change-between-tenancies. Six
// regimes: Texas (Tex. Prop. Code § 92.156(a) — security device
// operated by key/card/combination shall be rekeyed by landlord at
// landlord's expense not later than 7th day after each tenant
// turnover date + § 92.156(b) tenant-requested additional rekeying
// at tenant expense — STRONGEST mandatory rekey-between-tenancies
// statute); California (Cal. Civ. Code § 1941.3(a)(1) landlord must
// install and maintain operable deadbolt lock on each main swinging
// entry door + § 1941.3(a)(2) bolt extension at least 13/16 inch
// beyond strike edge into doorjamb — installation + maintenance not
// rekey-between-tenancies); Illinois (765 ILCS 5/12 + Chicago RLTO
// + local statutes — landlord shall change or rekey locks ON OR
// BEFORE the day new tenant moves in — STRICTEST TIMING same-day);
// Virginia (Va. Code § 55.1-1221 — landlord shall provide locks AND
// peepholes on each rental dwelling unit); NewYork (no statewide NY
// lock-change requirement; NYC Housing Maintenance Code § 27-2043
// may impose specific peephole standards in NYC); Default (no
// statewide rekeying requirement; common-law best practice without
// enforceable mandate). Distinct from lockout_penalties (unlawful
// self-help lockouts during tenancy) and dv_termination (domestic-
// violence emergency lock change during tenancy).
// ---------------------------------------------------------------------------

async fn lock_change_between_tenancies_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<LockChangeInput>,
) -> Result<Json<LockChangeResult>, ApiError> {
    if b.days_since_prior_tenant_move_out > 100_000
        || b.days_since_new_tenant_move_in > 100_000
        || b.days_from_move_in_to_rekey > 100_000
    {
        return Err(ApiError::BadRequest(
            "day counters look invalid (>100000)".into(),
        ));
    }
    Ok(Json(check_lock_change_between_tenancies(&b)))
}

// ---------------------------------------------------------------------------
// State landlord-lien-on-tenant-property compliance check.
//
// Mounted at POST /api/rental/landlord-lien-prohibition. Six
// regimes: Texas (Tex. Prop. Code § 54.041(a) STATUTORY landlord
// lien on nonexempt property in residence for unpaid rent;
// STRONGEST pro-landlord regime; § 54.043 contractual lien must be
// underlined or in conspicuous bold print); California (Cal. Civ.
// Code § 1861(a) court order required before taking possession;
// even with court order lien may NOT be enforced against property
// necessary to tenant's livelihood or household necessary items;
// §§ 1861.5-1861.27 enforcement procedure); NewYork (NO statutory
// landlord lien; must exist as contractual term in original lease
// OR court-rendered judgment lien; no self-help possession); Massa-
// chusetts (no precedent for statutory lien in ordinary tenancy;
// UCC Article 9 permits voluntary lien; storage lien for warehouse
// operators); Illinois (735 ILCS 5/9 et seq. — court judgment
// required first; unpaid rent alone does not automatically create
// lien); Default (varies by state; common-law landlord's lien
// generally requires court order). Distinct from abandoned_property_
// handling which addresses disposition of belongings LEFT BEHIND
// after vacating.
// ---------------------------------------------------------------------------

async fn landlord_lien_prohibition_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<LandlordLienInput>,
) -> Result<Json<LandlordLienResult>, ApiError> {
    Ok(Json(check_landlord_lien_prohibition(&b)))
}

// ---------------------------------------------------------------------------
// State landlord former-federal-or-state-ordnance-location
// disclosure compliance check.
//
// Mounted at POST /api/rental/military-ordnance-disclosure. Three
// regimes: California (Cal. Civ. Code § 1940.7 — landlord with
// ACTUAL KNOWLEDGE of former federal or state ordnance location
// WITHIN ONE MILE of residential dwelling shall give WRITTEN
// NOTICE to prospective tenant PRIOR TO execution of rental
// agreement; for tenancies in existence on January 1, 1990 notice
// as soon as practicable; prompted by December 10, 1983 Tierra
// Santa tragedy in San Diego); FederalMMRP (10 U.S.C. § 2710 +
// § 2701 federal Military Munitions Response Program / DoD-USACE
// FUDS public inventory; no general federal landlord disclosure
// mandate); Default (no statutory landlord disclosure mandate;
// common-law latent-defect disclosure may apply where landlord
// has actual knowledge).
// ---------------------------------------------------------------------------

async fn military_ordnance_disclosure_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<MilitaryOrdnanceInput>,
) -> Result<Json<MilitaryOrdnanceResult>, ApiError> {
    Ok(Json(check_military_ordnance_disclosure(&b)))
}

// ---------------------------------------------------------------------------
// State landlord sex-offender-database notice disclosure compliance
// check (Megan's Law database notice in residential leases).
//
// Mounted at POST /api/rental/sex-offender-database-notice. Three
// regimes: California (Cal. Civ. Code § 2079.10a + Cal. Pen. Code
// § 290.46 — VERBATIM statutory notice required in every
// residential lease pointing to www.meganslaw.ca.gov; landlord NOT
// required to provide specific offender names or addresses;
// landlord has NO statutory duty to investigate registry or
// affirmatively warn); NewJersey (N.J.S.A. 2C:7-1 et seq.
// Registration and Community Notification Laws RCNL community-
// notification framework administered by county prosecutor + NJ
// State Police; no landlord-tenant lease-disclosure mandate);
// Default (34 U.S.C. § 20911 et seq. federal Megan's Law framework
// formerly 42 U.S.C. § 14071; state registries; no statutory
// landlord lease-disclosure mandate; common-law fraudulent-
// concealment liability where landlord actively misrepresents).
// ---------------------------------------------------------------------------

async fn sex_offender_database_notice_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<SexOffenderNoticeInput>,
) -> Result<Json<SexOffenderNoticeResult>, ApiError> {
    Ok(Json(check_sex_offender_database_notice(&b)))
}

// ---------------------------------------------------------------------------
// State landlord mid-tenancy ownership-change notice + security
// deposit transfer compliance check.
//
// Mounted at POST /api/rental/mid-tenancy-ownership-change.
// Six regimes: California (Cal. Civ. Code § 1962(c) 15-day
// successor identity disclosure + § 1950.5(g) deposit transfer or
// refund + § 1962(c) bars nonpayment eviction during noncompliance);
// Massachusetts (G.L. c. 186 § 15B(2)(b) 45-day transfer to
// transferee plus accrued interest); Florida (Fla. Stat. § 83.49(5)
// upon sale transfer to new owner with simultaneous tenant notice
// OR refund); Washington (RCW 59.18.060(2) + RCW 59.18.270);
// NewYork (N.Y. GOL § 7-105 5-day transfer plus tenant notice);
// Default (most other states require some form of notice + deposit
// transfer via state landlord-tenant statute or common-law
// successor-liability). Distinct from landlord_identification_
// disclosure (initial identity at tenancy start) and foreclosure_
// tenant_rights (foreclosure-driven PTFA protections).
// ---------------------------------------------------------------------------

async fn mid_tenancy_ownership_change_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<MidTenancyOwnershipInput>,
) -> Result<Json<MidTenancyOwnershipResult>, ApiError> {
    if b.days_since_ownership_transfer > 100_000 {
        return Err(ApiError::BadRequest(
            "days_since_ownership_transfer looks invalid (>100000)".into(),
        ));
    }
    Ok(Json(check_mid_tenancy_ownership_change(&b)))
}

// ---------------------------------------------------------------------------
// mid_tenancy_term_modification: Mid-tenancy modification of NON-RENT lease
// terms — landlord compliance check for unilateral changes to pet policy,
// parking rules, smoking policy, late-fee structure, etc. during an existing
// tenancy. Four regimes: California (Cal. Civ. Code § 827(a) permits
// unilateral periodic-tenancy term modification with 30-day notice for
// monthly periodic or 7-day floor for shorter periodic; CCP § 1013(a) adds
// 5 mail-service days within CA; fixed-term requires bilateral consent);
// NewYork (no statute authorizes unilateral non-rent term modification —
// bilateral written consent required; RPL § 226-c addresses only rent +
// non-renewal); Texas (lease-granted modification right enforceable with
// written notice and prescribed advance period; absent lease clause,
// bilateral consent required; § 92.006 anti-waiver of mandatory warranties);
// Default (common-law contract rule — bilateral consent + Restatement
// (Second) of Contracts § 149 statute of frauds for 1-year-plus leases).
// Distinct from rent_increase_notice_period (rent-amount changes for
// CA/WA/OR) and lease_termination_notice (ending/non-renewal across
// CA/NY/OR/WA/NJ + default).
// ---------------------------------------------------------------------------

async fn mid_tenancy_term_modification_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<MidTenancyTermModInput>,
) -> Result<Json<MidTenancyTermModResult>, ApiError> {
    if b.notice_days_provided > 10_000 {
        return Err(ApiError::BadRequest(
            "notice_days_provided looks invalid (>10000)".into(),
        ));
    }
    if let Some(d) = b.agreement_shortened_to_days {
        if d > 10_000 {
            return Err(ApiError::BadRequest(
                "agreement_shortened_to_days looks invalid (>10000)".into(),
            ));
        }
    }
    Ok(Json(check_mid_tenancy_term_modification(&b)))
}

// ---------------------------------------------------------------------------
// State tenant right-to-install solar-energy-system compliance
// check.
//
// Mounted at POST /api/rental/tenant-solar-installation. Emerging
// area parallel to ev_charger_installation (tenant right-to-charge
// EV). Four regimes: California (Cal. Civ. Code § 714 Solar Rights
// Act restrictions on solar energy systems are void and
// unenforceable + § 714.1 HOA common-interest development rooftop
// solar + § 4600 + § 4746 CID common-area solar installation by
// member-owners — tenant rental coverage limited; protects plug-in
// portable and roof-mounted where tenant has exclusive use);
// Colorado (Colorado HB22-1020 Customer Right To Use Energy 2022 +
// Colorado 2026 plug-in solar legalization bill extending portable
// solar to renters and multifamily residents — establishes
// regulatory framework for portable arrays without landlord
// prohibition); NewJersey (N.J.S.A. 45:22A-48.2 Planned Real Estate
// Development Full Disclosure Act limits HOA authority over solar
// collectors; tenant rental coverage limited; requires lease
// consent); Default (most states with solar-rights laws cover
// homeowners + HOAs + condos; tenant rentals require lease-based
// or landlord-consent installation). Three installation types:
// PlugInPortable (most permissive), RoofMounted (typically requires
// consent), GroundMounted (always requires consent). Universal
// safety thresholds: installation must meet electrical/safety code
// AND must not damage landlord property.
// ---------------------------------------------------------------------------

async fn tenant_solar_installation_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<TenantSolarInput>,
) -> Result<Json<TenantSolarResult>, ApiError> {
    Ok(Json(check_tenant_solar_installation(&b)))
}

// ---------------------------------------------------------------------------
// State + federal tenant / owner right-to-display-flag compliance
// check.
//
// Mounted at POST /api/rental/flag-display-right. Four regimes:
// Federal (Freedom to Display the American Flag Act of 2005 — 4
// U.S.C. § 5 + Pub. L. 109-243 — applies to condo/cooperative/
// residential association; owner must have separate ownership
// interest OR right to exclusive possession/use; NO private right
// of action); Florida (Fla. Stat. § 720.304(2) HOA + § 718.113
// condo + § 723.054 mobile home parks; HB 437 2023 two-flag
// expansion; STATE LAW EXPLICITLY TRUMPS LEASE for renter U.S./
// state/military/POW-MIA flag display); Virginia (Va. Code
// § 55.1-1820 display of U.S. flag in common interest communities
// + supporting structures + affirmative defense); Default (federal
// Act applies to associations only; no general landlord-tenant
// private statutory protection). Sibling to religious_display_
// doorpost + firearms_in_rental_unit + otard_antenna_installation
// — collectively the tenant-rights-in-rental-unit display +
// installation cluster.
// ---------------------------------------------------------------------------

async fn flag_display_right_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<FlagDisplayInput>,
) -> Result<Json<FlagDisplayResult>, ApiError> {
    Ok(Json(check_flag_display_right(&b)))
}

// ---------------------------------------------------------------------------
// State written-vs-oral lease requirement compliance check.
//
// Mounted at POST /api/rental/written-lease-requirement. Universal
// U.S. floor: Statute of Frauds requires leases of real property
// for term exceeding ONE YEAR to be in writing. Leases of ONE YEAR
// or LESS may be oral. Five regimes: NewYork (N.Y. Gen. Oblig. Law
// § 5-703 + part-performance exception + N.Y. Gen. Bus. Law
// § 5-702 Plain Language Law for separate content requirement);
// Illinois (740 ILCS 80/2 + UNIQUE case-law conversion: oral lease
// > 1 year treated as year-to-year tenancy terminable on 60-day
// written notice); California (Cal. Civ. Code § 1624(a)(3) +
// § 1971); Washington (RCW 59.18.230 + RCW 64.04.010); Default
// (universal UCC § 2A-201 + state common-law one-year rule).
// ---------------------------------------------------------------------------

async fn written_lease_requirement_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<WrittenLeaseInput>,
) -> Result<Json<WrittenLeaseResult>, ApiError> {
    if b.lease_term_days > 50 * 365 {
        return Err(ApiError::BadRequest(
            "lease_term_days looks invalid (>50 years)".into(),
        ));
    }
    Ok(Json(check_written_lease_requirement(&b)))
}

// ---------------------------------------------------------------------------
// Holdover tenant damages — landlord recovery calculation.
//
// Mounted at POST /api/rental/holdover-tenant-damages. Two
// structurally distinct regimes: (1) STATUTORY DOUBLE RENT
// (Florida — Fla. Stat. § 83.58 imposes 2× rent multiplier on the
// holdover period, partial months count as full periods per "any
// part thereof" language); (2) RENT-ACCEPTANCE month-to-month
// conversion (California — Cal. Civ. Code § 1945 rebuttable
// presumption; New York — N.Y. Real Prop. Law § 232-c mandatory
// conversion). Default common-law (Restatement (Second) of
// Property § 14.5) gives the landlord an election between
// new-tenancy treatment and actual-damages trespass. Crucial
// distinction: FL multiplier applies regardless of rent
// acceptance (subject to split-authority waiver concerns);
// CA + NY conversion engages ONLY when rent is accepted
// post-expiration.
// ---------------------------------------------------------------------------

async fn holdover_tenant_damages_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<HoldoverTenantInput>,
) -> Result<Json<HoldoverTenantResult>, ApiError> {
    if b.days_in_holdover > 50 * 365 {
        return Err(ApiError::BadRequest(
            "days_in_holdover looks invalid (>50 years)".into(),
        ));
    }
    if b.monthly_rent_cents < 0 {
        return Err(ApiError::BadRequest(
            "monthly_rent_cents must be non-negative".into(),
        ));
    }
    Ok(Json(check_holdover_tenant_damages(&b)))
}

// ---------------------------------------------------------------------------
// Lease assignment consent — landlord consent rules for tenant
// assignment of a residential lease.
//
// Mounted at POST /api/rental/lease-assignment-consent. Distinct
// from sublet_consent module (subleasing leaves tenant on the
// hook; assignment fully transfers the leasehold). Four regimes:
// (1) NewYork — N.Y. Real Prop. Law § 226-b draws a sharp
// structural distinction between sublease (4+ unit buildings,
// statutory reasonable-consent right with 30-day deemed-consent
// rule on landlord silence) and assignment (landlord may
// unconditionally withhold consent BUT if refusal is unreasonable,
// landlord MUST RELEASE the tenant on 30 days notice — the only
// state regime with a structural exit valve); (2) California —
// Cal. Civ. Code § 1995.260 implies reasonableness when the lease
// consent clause is silent on standard, codifying Kendall v.
// Ernest Pestana, Inc. (40 Cal. 3d 488, 1985); NOT retroactive to
// leases executed before September 23, 1983 (pre-statute rule
// allows unreasonable withholding); (3) Restatement (Second) of
// Property § 15.2 — default rule of FREE ASSIGNABILITY absent
// restriction; restrictions strictly construed against the
// landlord; (4) LeaseControls modern-majority rule — consent
// clause enforced as written; Texas + Illinois + Massachusetts
// commercial context.
// ---------------------------------------------------------------------------

async fn lease_assignment_consent_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<LeaseAssignmentInput>,
) -> Result<Json<LeaseAssignmentResult>, ApiError> {
    if b.tenant_request_pending_days > 100_000 {
        return Err(ApiError::BadRequest(
            "tenant_request_pending_days out of range".into(),
        ));
    }
    Ok(Json(check_lease_assignment_consent(&b)))
}

// ---------------------------------------------------------------------------
// Tenant cure period for non-rent lease breach.
//
// Mounted at POST /api/rental/lease-cure-period. Four regimes for
// the statutory cure window during which a tenant may correct a
// non-rent lease violation: (1) California — Cal. Code Civ. Proc.
// § 1161(3) 3-day cure EXCLUDING Saturdays, Sundays, and judicial
// holidays (effectively 3 business days; not applicable to
// non-payment-of-rent or nuisance cases); (2) Florida — Fla. Stat.
// § 83.56(2)(b) 7-day cure for curable violations + § 83.56(2)(a)
// 7-day vacate for non-curable; 12-MONTH RECURRENCE RULE bypasses
// subsequent notice requirement; (3) New York — N.Y. RPAPL
// § 753(4) 10-day cure for lease-covenant breach (HSTPA 2019
// added 30-day chronic-late-rent / nuisance defense distinct from
// § 753(4)); (4) Default common-law reasonable cure under
// Restatement (Second) of Property § 13.1 + § 16.1. Distinct from
// rent-payment cure (eviction_notices module) and grace periods
// before late fees (late_payment_grace_period module).
// ---------------------------------------------------------------------------

async fn lease_cure_period_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<LeaseCureInput>,
) -> Result<Json<LeaseCureResult>, ApiError> {
    if b.days_since_notice_served > 100_000 || b.business_days_since_notice > 100_000 {
        return Err(ApiError::BadRequest(
            "day-count inputs out of range".into(),
        ));
    }
    Ok(Json(check_lease_cure_period(&b)))
}

// ---------------------------------------------------------------------------
// Portable / reusable tenant screening report regulation.
//
// Mounted at POST /api/rental/portable-tenant-screening-report.
// Three regimes for whether a landlord must accept a tenant-
// provided pre-pulled screening report in lieu of conducting and
// charging for their own: (1) Colorado — Colo. Rev. Stat.
// § 38-12-902 / § 38-12-904 (HB23-1099, eff. 2023): MANDATORY
// ACCEPTANCE of compliant report (≤30 days old, complete CRTSR
// components, no-material-change statement); $2,500 violation
// penalty reducible to $50 if cured within 7 days; single-
// application-at-a-time exception with 20-day refund; (2)
// Washington — RCW 59.18.257: OPT-IN DISCLOSURE — landlord must
// publish acceptance status on property website; if opt-in,
// must accept compliant CRTSR (credit + criminal + eviction +
// employment + rental history); $100 max violation penalty;
// (3) Default — no statutory portability requirement; landlord
// may demand fresh fee-based screening under general
// application_fees + FCRA regime.
// ---------------------------------------------------------------------------

async fn portable_tenant_screening_report_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<PortableScreeningInput>,
) -> Result<Json<PortableScreeningResult>, ApiError> {
    if b.report_age_days > 100_000 {
        return Err(ApiError::BadRequest(
            "report_age_days out of range".into(),
        ));
    }
    Ok(Json(check_portable_screening(&b)))
}

// ---------------------------------------------------------------------------
// HOA rental restriction enforceability.
//
// Mounted at POST /api/rental/hoa-rental-restriction. Three
// regimes for when a homeowners association may enforce rental
// restrictions against unit owners: (1) Florida — Fla. Stat.
// § 720.306(1)(h) (eff. July 1, 2021) GRANDFATHER RULE: HOA
// amendments adopted after July 1, 2021 apply only to owners
// who acquired title after the amendment OR who affirmatively
// consented; silence does not count as consent. Two narrow
// exceptions bind ALL owners regardless: (a) amendments
// restricting leases ≤ 6 months; (b) amendments limiting
// rentals to ≤ 3 per calendar year. Grandfather survives heir
// + affiliate transfers; LOST on transfer to unrelated third
// party. (2) Arizona — A.R.S. § 33-1806.01 (planned communities):
// declaration controls; HOA may restrict rentals if declaration
// so provides; statutory third-party agent designation right
// under § 33-1806.01(B). (3) Default — declaration controls;
// no statutory grandfather. Distinct from str_regulation
// (municipal public regulation) and condominium_conversion_
// protection (tenant-side protection during conversions).
// ---------------------------------------------------------------------------

async fn hoa_rental_restriction_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<HoaRentalInput>,
) -> Result<Json<HoaRentalResult>, ApiError> {
    Ok(Json(check_hoa_rental_restriction(&b)))
}

// ---------------------------------------------------------------------------
// Rent acceleration clause enforceability.
//
// Mounted at POST /api/rental/rent-acceleration-enforceability.
// Three regimes for when a landlord may demand the full unpaid
// rent balance for the remainder of the lease term as a lump sum:
// (1) California — Cal. Civ. Code § 1671 + § 1951.2: § 1671(d)
// PRESUMES residential liquidated damages clauses INVALID until
// landlord proves reasonableness; § 1671(b) commercial
// reasonableness test; § 1951.2 mitigation duty;
// (2) New York — Holy Properties Ltd., L.P. v. Kenneth Cole
// Productions, Inc., 87 N.Y.2d 130 (1995): commercial
// acceleration clauses enforceable WITHOUT mitigation duty,
// subject to fraud/overreaching/unconscionability exception
// and mandatory PRESENT-VALUE DISCOUNT when landlord has
// possession; (3) Default — Restatement (Second) of Contracts
// § 356 penalty doctrine + Restatement (Second) of Property
// § 12.1 mitigation duty + PV discount required.
// ---------------------------------------------------------------------------

async fn rent_acceleration_enforceability_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentAccelerationInput>,
) -> Result<Json<RentAccelerationResult>, ApiError> {
    if b.acceleration_amount_demanded_cents > 100_000_000_000
        || b.actual_damages_estimate_cents > 100_000_000_000
    {
        return Err(ApiError::BadRequest(
            "acceleration or damages amount out of range".into(),
        ));
    }
    Ok(Json(check_rent_acceleration(&b)))
}

// ---------------------------------------------------------------------------
// Tenant in foreclosure protections.
//
// Mounted at POST /api/rental/tenant-in-foreclosure-protection.
// Three regimes for notice + timing requirements when a successor
// in interest (foreclosure buyer) evicts existing tenants: (1)
// Federal PTFA — Protecting Tenants at Foreclosure Act (Pub. L.
// 111-22 § 702, restored permanently by Pub. L. 115-174 § 304
// effective June 23, 2018): bona fide tenants get greater of 90
// days OR lease remainder; owner-occupant primary residence
// exception terminates lease early but 90-day notice still
// required; (2) California — Cal. Civ. Code § 2924.8 + § 1161b:
// adds pre-sale posting + first-class mailing requirement,
// $100 infraction under § 2924.8(d) for tearing down notice
// within 72 hours; (3) New York — N.Y. RPAPL § 1305: adds
// § 1305(4) preservation of pre-existing rent-control/rent-
// stabilization/subsidy rights AND extends protections to
// tenants not named in foreclosure action. Federal PTFA is
// national FLOOR; state law adds ceiling when more protective.
// ---------------------------------------------------------------------------

async fn tenant_in_foreclosure_protection_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<TenantForeclosureInput>,
) -> Result<Json<TenantForeclosureResult>, ApiError> {
    if b.lease_remaining_days > 50 * 365
        || b.days_since_foreclosure_notice > 100_000
    {
        return Err(ApiError::BadRequest(
            "lease_remaining_days or days_since_foreclosure_notice out of range".into(),
        ));
    }
    Ok(Json(check_tenant_foreclosure_protection(&b)))
}

// ---------------------------------------------------------------------------
// Security deposit bank disclosure.
//
// Mounted at POST /api/rental/security-deposit-bank-disclosure.
// Four regimes for when a landlord must disclose to the tenant
// where the security deposit is held: (1) New York — N.Y. Gen.
// Oblig. Law § 7-103: bank name + address + amount required;
// § 7-103(2) imposes interest-bearing account requirement for
// buildings with 6+ family dwelling units, with landlord
// retaining 1% per annum as administration expense and remaining
// interest belonging to tenant; (2) New Jersey — N.J.S.A. 46:8-19:
// 30-day window for bank name + address + account type + interest
// rate + amount disclosure; re-notification on bank/landlord
// change; annual interest payable in cash, credited to rent, or
// on January 31; (3) Massachusetts — Mass. Gen. Laws c. 186
// § 15B(3)(a): 30-day receipt requirement (bank + amount +
// account number); annual statement; § 15B(6) IMMEDIATE RETURN
// remedy for non-compliance (harshest in U.S.); (4) Default —
// no statutory disclosure requirement (CA Civ. Code § 1950.5,
// TX Prop. Code § 92.103 + § 92.108 govern amount + return only).
// Distinct from sibling modules security_deposit_caps,
// deposit_interest, deposit_return_windows, damage_deduction_
// itemization.
// ---------------------------------------------------------------------------

async fn security_deposit_bank_disclosure_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<SecurityDepositBankDisclosureInput>,
) -> Result<Json<SecurityDepositBankDisclosureResult>, ApiError> {
    if b.days_since_deposit_received > 50 * 365
        || b.days_since_transfer_event > 50 * 365
    {
        return Err(ApiError::BadRequest(
            "day-count inputs out of range".into(),
        ));
    }
    Ok(Json(check_security_deposit_bank_disclosure(&b)))
}

// ---------------------------------------------------------------------------
// Landlord harassment liability.
//
// Mounted at POST /api/rental/landlord-harassment. Three regimes
// for affirmative-harassment civil penalty exposure: (1)
// California — Cal. Civ. Code § 1940.2: prohibits 5 categories
// of landlord conduct (theft/extortion under Penal Code § 484/
// § 518; force/threats interfering with quiet enjoyment;
// significant + intentional § 1954 entry violation; immigration/
// citizenship status disclosure threat); § 1940.2(b) civil
// penalty up to $2,000 per violation; § 1940.2(c) good-faith
// warning exception. (2) NYC — Admin. Code § 27-2004(a)(48) +
// § 27-2005(d) + § 27-2115(m): requires BOTH (i) prohibited
// conduct (force/threats, service interruptions, repeated
// buyout offers, baseless court proceedings) AND (ii) intent or
// causation to vacate; civil penalty $1K-$10K per dwelling unit
// per violation (multiplier effect across buildings). (3)
// Default — common-law claims (IIED, conversion, quiet
// enjoyment breach, constructive eviction); compensatory
// damages limited to actual harm; no statutory civil penalty.
// Sibling modules: lockout_penalties, quiet_enjoyment,
// retaliation_windows.
// ---------------------------------------------------------------------------

async fn landlord_harassment_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<LandlordHarassmentInput>,
) -> Result<Json<LandlordHarassmentResult>, ApiError> {
    if b.violation_count > 100_000
        || b.dwelling_units_affected > 1_000_000
        || b.tenant_actual_damages_cents > 100_000_000_000
    {
        return Err(ApiError::BadRequest(
            "violation_count, dwelling_units_affected, or tenant_actual_damages_cents out of range".into(),
        ));
    }
    Ok(Json(check_landlord_harassment(&b)))
}

// ---------------------------------------------------------------------------
// Landlord's duty to deliver possession at lease commencement.
//
// Mounted at POST /api/rental/landlord-possession-delivery. Three
// regimes for when a landlord must deliver ACTUAL possession (not
// just legal right) to the tenant: (1) URLTA states (~20
// jurisdictions including AK, AZ, FL, IA, KS, KY, MS, MT, NE, NM,
// OR, RI, SC, TN, VA, WA) — § 2.103 statutory duty + § 4.102
// remedies (greater of 3 months' rent OR threefold actual damages
// + reasonable attorney's fees + injunctive relief); (2) English
// Rule (modern majority + Restatement (Second) of Property § 6.2)
// — landlord must deliver actual possession; tenant may cancel
// lease + recover actual damages with rent abatement for delay
// period; no statutory multiplier; (3) American Rule (minority,
// Hannan v. Dusch, 153 S.E. 824 (Va. 1930)) — landlord delivers
// only LEGAL POSSESSION; tenant must evict holdover party
// directly + recover damages from THAT party; lease remains in
// force, rent continues to accrue.
// ---------------------------------------------------------------------------

async fn landlord_possession_delivery_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<LandlordPossessionDeliveryInput>,
) -> Result<Json<LandlordPossessionDeliveryResult>, ApiError> {
    if b.days_delayed_possession > 50 * 365
        || b.monthly_rent_cents > 100_000_000_000
        || b.tenant_actual_damages_cents > 100_000_000_000
    {
        return Err(ApiError::BadRequest(
            "input value out of range".into(),
        ));
    }
    Ok(Json(check_landlord_possession_delivery(&b)))
}

// ---------------------------------------------------------------------------
// Lease waiver clause enforceability.
//
// Mounted at POST /api/rental/lease-waiver-enforceability. Three
// regimes for when a lease-drafted waiver clause is void: (1)
// New York — N.Y. Gen. Oblig. Law § 5-321: narrow scope, voids
// only landlord-negligence exculpatory clauses (applies to BOTH
// residential and commercial leases); (2) California — Cal. Civ.
// Code § 1953: broad scope, voids 6 categories of residential
// tenant rights waivers (§ 1950.5 + § 1954 rights, future cause
// of action, notice/hearing rights, procedural rights including
// jury trial, duty of care, cumulative remedies); (3) Default —
// common-law analysis: enforceable if knowing + voluntary +
// public policy permits; Restatement (Second) of Contracts § 178
// test. Distinct from sibling modules landlord_harassment,
// habitability_remedies, quiet_enjoyment, plain_language_lease.
// ---------------------------------------------------------------------------

async fn lease_waiver_enforceability_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<LeaseWaiverEnforceabilityInput>,
) -> Result<Json<LeaseWaiverEnforceabilityResult>, ApiError> {
    Ok(Json(check_lease_waiver_enforceability(&b)))
}

// ---------------------------------------------------------------------------
// Landlord retaliation damages math.
//
// Mounted at POST /api/rental/landlord-retaliation-damages. Four
// regimes for damages recovery when landlord retaliation has
// been found: (1) California — Cal. Civ. Code § 1942.5(h):
// actual damages + PUNITIVE $100-$2,000 per retaliatory act when
// fraud/oppression/malice shown + attorney fees; 180-day
// presumption window under § 1942.5(a); (2) Massachusetts —
// G.L. c. 186 § 18: statutory damages floor (1 month's rent) /
// ceiling (3 months OR actual whichever is greater) + attorney
// fees; 6-month presumption window with CLEAR AND CONVINCING
// rebuttal standard (highest civil standard); waiver of § 18 in
// any lease void; (3) New Jersey — N.J.S.A. 2A:42-10.10 Reprisal
// Law: actual damages + injunctive/equitable relief; statutory
// presumption with case-by-case rebuttal; (4) Default — common-
// law actual damages only; attorney fees only if lease permits.
// Distinct from sibling modules retaliation_windows, landlord_
// harassment, lockout_penalties, landlord_possession_delivery.
// ---------------------------------------------------------------------------

async fn landlord_retaliation_damages_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<LandlordRetaliationDamagesInput>,
) -> Result<Json<LandlordRetaliationDamagesResult>, ApiError> {
    if b.monthly_rent_cents < 0
        || b.monthly_rent_cents > 100_000_000_000
        || b.tenant_actual_damages_cents < 0
        || b.tenant_actual_damages_cents > 100_000_000_000
    {
        return Err(ApiError::BadRequest(
            "monthly_rent_cents or tenant_actual_damages_cents out of range".into(),
        ));
    }
    if b.retaliation_acts_count < 0 || b.retaliation_acts_count > 100_000 {
        return Err(ApiError::BadRequest(
            "retaliation_acts_count out of range".into(),
        ));
    }
    Ok(Json(check_landlord_retaliation_damages(&b)))
}

// ---------------------------------------------------------------------------
// Tenant's right to apply security deposit as last month's rent.
//
// Mounted at POST /api/rental/last-month-rent-offset. Four
// regimes for tenant-side offset right: (1) Texas — Tex. Prop.
// Code § 92.108 STRICT PROHIBITION on tenant withholding last
// month's rent against deposit; § 92.108(b) bad-faith violation
// triggers TREBLE DAMAGES + attorney's fees (strongest tenant
// penalty in U.S.); § 92.056 health/safety repair exception
// permits offset. (2) California — Cal. Civ. Code § 1950.5
// LABEL-DEPENDENT TREATMENT: if lease labels payment as "last
// month's rent" tenant is relieved; if labeled "security" must
// pay separately. AB 12 (2024) capped deposit at 1 month.
// (3) New York — N.Y. Gen. Oblig. Law § 7-103 TRUST-FUND
// PRINCIPLE: landlord may apply at end; tenant may not
// unilaterally. (4) Default — common-law separation; tenant
// cannot unilaterally offset absent express lease provision.
// Distinct from sibling modules security_deposit_caps,
// deposit_return_windows, damage_deduction_itemization,
// security_deposit_bank_disclosure.
// ---------------------------------------------------------------------------

async fn last_month_rent_offset_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<LastMonthRentOffsetInput>,
) -> Result<Json<LastMonthRentOffsetResult>, ApiError> {
    if b.monthly_rent_cents < 0
        || b.monthly_rent_cents > 100_000_000_000
    {
        return Err(ApiError::BadRequest(
            "monthly_rent_cents out of range".into(),
        ));
    }
    Ok(Json(check_last_month_rent_offset(&b)))
}

// ---------------------------------------------------------------------------
// Emotional Support Animal (ESA) documentation requirements.
//
// Mounted at POST /api/rental/emotional-support-animal-documentation.
// Three regimes for ESA documentation reliability: (1) California
// — Cal. Health & Safety Code § 122318 (AB 468, eff. Jan 1, 2022)
// imposes 5-element practitioner test: valid license + licensed
// in jurisdiction + ≥ 30-day prior therapeutic relationship +
// clinical evaluation + misdemeanor warning. Practitioner subject
// to discipline for violation; (2) Florida — Fla. Stat. § 760.27
// (eff. July 1, 2020) requires personal knowledge of disability
// + telehealth FL-licensed (or out-of-state in-person visit);
// internet-only registrations EXPLICITLY UNRELIABLE; knowingly
// fraudulent documentation is second-degree misdemeanor; (3)
// Default — Federal FHA + HUD Notice FHEO-2020-01: reasonable-
// accommodation analysis; internet-only relationships disfavored
// but not categorically rejected. Readily-apparent disability
// bypasses documentation entirely in all regimes. Distinct from
// service_animal (ADA-covered trained service animals receive
// broader protection).
// ---------------------------------------------------------------------------

async fn emotional_support_animal_documentation_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<EsaDocumentationInput>,
) -> Result<Json<EsaDocumentationResult>, ApiError> {
    if b.therapeutic_relationship_days > 50 * 365 {
        return Err(ApiError::BadRequest(
            "therapeutic_relationship_days out of range".into(),
        ));
    }
    Ok(Json(check_esa_documentation(&b)))
}

// ---------------------------------------------------------------------------
// Lease non-disparagement clause prohibition (federal CRFA).
//
// Mounted at POST /api/rental/lease-nondisparagement-prohibition.
// Federal Consumer Review Fairness Act of 2016, 15 U.S.C. § 45b,
// voids form-contract provisions that (1) prohibit/restrict
// tenant's ability to engage in covered communications (online
// reviews), (2) impose penalty/fee for review, or (3) transfer
// IP rights in feedback. Four § 45b(c) exceptions: legally
// actionable content, trade secret / personal info, medical-
// record content, otherwise-unlawful content. § 45b(d) makes
// OFFERING such a form contract itself unlawful — separate
// violation from attempts at enforcement. § 45b(e) provides
// concurrent FTC and state attorney general enforcement.
// Distinct from sibling modules lease_waiver_enforceability
// (general lease waivers — NY § 5-321 + CA § 1953),
// landlord_retaliation_damages, and landlord_harassment.
// ---------------------------------------------------------------------------

async fn lease_nondisparagement_prohibition_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<LeaseNondisparagementInput>,
) -> Result<Json<LeaseNondisparagementResult>, ApiError> {
    Ok(Json(check_lease_nondisparagement_prohibition(&b)))
}

// ---------------------------------------------------------------------------
// State plain-language lease requirement compliance check
//
// Mounted at POST /api/rental/plain-language-lease-check. Five
// regimes: NewYorkClearCoherent50DollarPenalty (NY GOL § 5-702 eff.
// 1978-11-01 — $50 statutory penalty + actual damages + good-faith
// defense + barred after full performance); NewJersey100DollarMinimum-
// PlusAttorneyFees (N.J.S.A. 56:12-1 et seq. — greater of $100 or
// actual damages + reasonable attorney fees); PennsylvaniaPlain-
// LanguageNineTests (73 P.S. § 2201 et seq. — 9-test substantial
// compliance + AG preapproval); ConnecticutDescriptiveReadability
// (Conn. Gen. Stat. § 42-152 eff. 1979 oldest U.S. statute);
// NoStatewidePlainLanguageRequirement (46 other states + DC).
// ---------------------------------------------------------------------------

async fn plain_language_lease_check_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<PlainLanguageInput>,
) -> Result<Json<PlainLanguageResult>, ApiError> {
    if b.state_code.trim().is_empty() {
        return Err(ApiError::BadRequest("state_code required".into()));
    }
    if b.tenant_actual_damages_dollars < 0 {
        return Err(ApiError::BadRequest(
            "tenant_actual_damages_dollars must be >= 0".into(),
        ));
    }
    if b.pa_nine_tests_satisfied > 9 {
        return Err(ApiError::BadRequest(
            "pa_nine_tests_satisfied must be <= 9".into(),
        ));
    }
    Ok(Json(check_plain_language(&b)))
}

// ---------------------------------------------------------------------------
// State tenant roommate / additional-occupant authorization check
//
// Mounted at POST /api/rental/roommate-authorization-check. Three
// regimes: NewYorkStatutoryRoommateRight (NY RPL § 235-f Roommate
// Law — 1 additional adult occupant per tenant + immediate family +
// dependent children of occupant + tenant's primary residence;
// lease restrictions VOID under § 235-f(7); 30-day notification);
// CaliforniaTwoPlusOneFormula (CA state-law '2 per bedroom + 1
// additional' occupancy formula); DefaultLeaseGoverns (48 other
// states + DC).
// ---------------------------------------------------------------------------

async fn roommate_authorization_check_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RoommateAuthorizationInput>,
) -> Result<Json<RoommateAuthorizationResult>, ApiError> {
    if b.state_code.trim().is_empty() {
        return Err(ApiError::BadRequest("state_code required".into()));
    }
    Ok(Json(check_roommate_authorization(&b)))
}

// ---------------------------------------------------------------------------
// State tenant right-to-install EV charging station check
//
// Mounted at POST /api/rental/ev-charger-installation-check. Five
// regimes: CaliforniaInsuranceRequired (Cal. Civ. Code § 1947.6
// eff. 2015-07-01 — tenant pays + written agreement + $1M
// liability insurance, all required); HawaiiLeaseProvisionVoid
// (HRS § 196-7.5 — any lease restriction VOID; extends to
// common-element parking); IllinoisNewBuildingsOnly (Electric
// Vehicle Charging Act 765 ILCS eff. 2023 — NEW multi-unit
// dwellings only with 100% EV-ready parking); NewJerseyMulti-
// UnitRight (multi-unit residential reasonable-terms review);
// DefaultLeaseGoverns (46 other states + DC).
// ---------------------------------------------------------------------------

async fn ev_charger_installation_check_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<EvChargerInput>,
) -> Result<Json<EvChargerResult>, ApiError> {
    if b.state_code.trim().is_empty() {
        return Err(ApiError::BadRequest("state_code required".into()));
    }
    if b.tenant_liability_insurance_amount_dollars < 0 {
        return Err(ApiError::BadRequest(
            "tenant_liability_insurance_amount_dollars must be >= 0".into(),
        ));
    }
    Ok(Json(check_ev_charger(&b)))
}

// ---------------------------------------------------------------------------
// State tenant advance-rent prepayment limit compliance check
//
// Mounted at POST /api/rental/advance-rent-limit-check. Five regimes:
// NewYorkOneMonthFirstOnly (NY RPL § 238-a HSTPA 2019 — first month
// only + last month rent PROHIBITED + multi-month prepayments
// prohibited + unregulated tenancies only); MassachusettsFirstLast-
// SecurityLock (MA G.L. c. 186 § 15B — first + last + security + new
// lock cost + 5% annual interest on advance last-month rent);
// CaliforniaSixMonthLeaseOnly (Cal. Civ. Code § 1950.5 — multi-month
// prepayment requires lease ≥ 6 months covering ≥ 6 months);
// NewJerseyAdvanceRentUnlimited (N.J.S.A. 46:8-21.2 — advance rent
// separate from § 46:8-19 1.5-month security deposit cap, parties
// may agree to any amount); NoStateAdvanceRentLimit (46 other states
// + DC, lease governs).
// ---------------------------------------------------------------------------

async fn advance_rent_limit_check_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<AdvanceRentInput>,
) -> Result<Json<AdvanceRentResult>, ApiError> {
    if b.state_code.trim().is_empty() {
        return Err(ApiError::BadRequest("state_code required".into()));
    }
    Ok(Json(check_advance_rent(&b)))
}

// ---------------------------------------------------------------------------
// State fire-sprinkler disclosure in residential lease check
//
// Mounted at POST /api/rental/fire-sprinkler-disclosure-check. Two
// regimes: NewYorkSprinklerSystemNotice (NY RPL § 231-a eff.
// 2014-12-03 — conspicuous bold-face notice + last maintenance and
// inspection date if system present; NO statutory penalty);
// NoStateFireSprinklerDisclosureLaw (49 other states + DC).
// ---------------------------------------------------------------------------

async fn fire_sprinkler_disclosure_check_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<FireSprinklerDisclosureInput>,
) -> Result<Json<FireSprinklerDisclosureResult>, ApiError> {
    if b.state_code.trim().is_empty() {
        return Err(ApiError::BadRequest("state_code required".into()));
    }
    Ok(Json(check_fire_sprinkler(&b)))
}

// ---------------------------------------------------------------------------
// State bedbug extermination cost / treatment-responsibility check
//
// Mounted at POST /api/rental/bedbug-extermination-cost-check. Three
// regimes: CaliforniaAB551Comprehensive (Cal. Civ. Code §§
// 1954.600-1954.605 AB 551 2017 + § 1942.5 — § 1954.602 vacant-unit
// prohibition + § 1954.604 follow-up treatments + 180-day retaliation
// protection); MaineLandlordEradicationStatutory (Me. Stat. tit. 14
// § 6021-A eff. 2010 — 5-day investigation window + licensed pest
// control + tenant re-treatment liability if reckless introduction);
// DefaultImpliedWarrantyOfHabitability (48 other states + DC,
// common-law per Restatement 2d Property § 5.5).
// ---------------------------------------------------------------------------

async fn bedbug_extermination_cost_check_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<BedbugExterminationInput>,
) -> Result<Json<BedbugExterminationResult>, ApiError> {
    if b.state_code.trim().is_empty() {
        return Err(ApiError::BadRequest("state_code required".into()));
    }
    Ok(Json(check_bedbug_extermination(&b)))
}

// ---------------------------------------------------------------------------
// State crime-victim lease termination check (broader than DV-only)
//
// Mounted at POST /api/rental/crime-victim-termination-check. Five
// regimes: CaliforniaBroadestVictimCoverage (Cal. Civ. Code §
// 1946.7 eff. 2011 + AB 1493 2022 — broadest U.S. scope including
// DV + sexual assault + stalking + human trafficking + elder/
// dependent-adult abuse + bodily-injury/force-threat crimes;
// 14-day notice + supporting document); TexasThirtyDayNotice
// (Tex. Prop. Code §§ 92.0161 / 92.1061 — 30-day notice +
// tenant liable for notice period); WashingtonNinetyDayWindow
// (RCW 59.18.575 — 90-day window from incident + rent terminates
// at notice + DV + sexual assault + unlawful harassment + stalking);
// IllinoisSafeHomesActDualPath (765 ILCS 750 — 3-day imminent
// threat OR 60-day past sexual violence); NoStatewideBroad-
// CrimeVictimTermination (46 other states + DC).
// ---------------------------------------------------------------------------

async fn crime_victim_termination_check_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<CrimeVictimTerminationInput>,
) -> Result<Json<CrimeVictimTerminationResult>, ApiError> {
    if b.state_code.trim().is_empty() {
        return Err(ApiError::BadRequest("state_code required".into()));
    }
    Ok(Json(check_crime_victim_termination(&b)))
}

// ---------------------------------------------------------------------------
// State succession rights — surviving family member lease assumption
//
// Mounted at POST /api/rental/lease-succession-check. Three regimes:
// NewYorkRentRegulatedSuccession (NYC Rent Stabilization Code
// § 2523.5(b)(1) DHCR 1987 — 2-year residency adult / 1-year senior
// or disabled + broad family definition including non-traditional);
// NewJerseyAntiEvictionImmediateFamily (N.J.S.A. 2A:18-61.1 et seq.
// Anti-Eviction Act bars no-fault eviction of immediate family
// co-residents); DefaultLeaseGovernsNoSuccession (48 other states
// + DC — lease extinguishes on tenant's death).
// ---------------------------------------------------------------------------

async fn lease_succession_check_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<LeaseSuccessionInput>,
) -> Result<Json<LeaseSuccessionResult>, ApiError> {
    if b.state_code.trim().is_empty() {
        return Err(ApiError::BadRequest("state_code required".into()));
    }
    Ok(Json(check_lease_succession(&b)))
}

// ---------------------------------------------------------------------------
// State rent-payment-to-credit-bureau reporting compliance check
//
// Mounted at POST /api/rental/rent-credit-reporting-check. Two
// regimes: CaliforniaAB2747RentReporting (Cal. Civ. Code § 1954.06
// added by AB 2747, eff. 2025-04-01 — landlords with 15+ units must
// offer positive rent reporting + written notice at lease signing
// AND annually + tenant fee cap = lesser of $10/month or actual
// cost); NoStateRentReportingRequirement (49 other states + DC).
// ---------------------------------------------------------------------------

async fn rent_credit_reporting_check_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentCreditReportingInput>,
) -> Result<Json<RentCreditReportingResult>, ApiError> {
    if b.state_code.trim().is_empty() {
        return Err(ApiError::BadRequest("state_code required".into()));
    }
    if b.monthly_fee_charged_dollars < 0 || b.landlord_actual_monthly_cost_dollars < 0 {
        return Err(ApiError::BadRequest(
            "non-negative fee/cost inputs required".into(),
        ));
    }
    Ok(Json(check_rent_credit_reporting(&b)))
}

// ---------------------------------------------------------------------------
// State tenant rent escrow / withholding-into-court compliance check
//
// Mounted at POST /api/rental/rent-escrow-check. Five regimes:
// MarylandRentEscrowAct (Md. Real Prop. § 8-211 — 30-day
// reasonableness presumption); MassachusettsCounterclaimDefense
// (Mass. G.L. c. 239 § 8A — fair use/occupation value into court
// via eviction counterclaim); NewJerseyMariniHearingAdministrator
// (N.J.S.A. 2A:42-85 + Marini v. Ireland 1970 — must deposit ALL
// unpaid rent + court-appointed administrator); ColoradoLimited-
// Withholding (C.R.S. § 38-12-507 — entire-rent withholding bar);
// NoStatutoryRentEscrowFramework (46 other states + DC).
// ---------------------------------------------------------------------------

async fn rent_escrow_check_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentEscrowInput>,
) -> Result<Json<RentEscrowResult>, ApiError> {
    if b.state_code.trim().is_empty() {
        return Err(ApiError::BadRequest("state_code required".into()));
    }
    Ok(Json(check_rent_escrow(&b)))
}

// ---------------------------------------------------------------------------
// State tenant "right to dry" clothesline / drying-rack check
//
// Mounted at POST /api/rental/right-to-dry-check. Two regimes:
// CaliforniaTenantClotheslineRight (Cal. Civ. Code § 1940.20 added
// by AB 1448 eff. 2015-09-08 — tenant may use clothesline / drying
// rack in private leased area subject to no-interference + no-
// health-safety-hazard + building-code/HOA-compliance gates);
// NoStatewideTenantClotheslineRight (49 other states + DC — most
// have "right to dry" statutes but they apply to HOA/condo
// covenants only and do NOT extend to landlord-tenant rentals).
// ---------------------------------------------------------------------------

async fn right_to_dry_check_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RightToDryInput>,
) -> Result<Json<RightToDryResult>, ApiError> {
    if b.state_code.trim().is_empty() {
        return Err(ApiError::BadRequest("state_code required".into()));
    }
    Ok(Json(check_right_to_dry(&b)))
}

// ---------------------------------------------------------------------------
// State sublet / lease-assignment consent compliance check
// ---------------------------------------------------------------------------

async fn sublet_consent_check_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<SubletConsentInput>,
) -> Result<Json<SubletConsentResult>, ApiError> {
    if b.state_code.trim().is_empty() {
        return Err(ApiError::BadRequest("state_code required".into()));
    }
    Ok(Json(check_sublet_consent(&b)))
}

// ---------------------------------------------------------------------------
// State radon disclosure + testing compliance check
// ---------------------------------------------------------------------------

async fn radon_disclosure_check_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RadonDisclosureInput>,
) -> Result<Json<RadonDisclosureResult>, ApiError> {
    if b.state_code.trim().is_empty() {
        return Err(ApiError::BadRequest("state_code required".into()));
    }
    if b.current_radon_level_pcil < Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "current_radon_level_pcil must be >= 0".into(),
        ));
    }
    Ok(Json(check_radon_disclosure(&b)))
}

// ---------------------------------------------------------------------------
// State mold disclosure + remediation compliance check
// ---------------------------------------------------------------------------

async fn mold_disclosure_check_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<MoldCheckInput>,
) -> Result<Json<MoldCheckResult>, ApiError> {
    if b.state_code.trim().is_empty() {
        return Err(ApiError::BadRequest("state_code required".into()));
    }
    Ok(Json(check_mold_disclosure(&b)))
}

// ---------------------------------------------------------------------------
// State bedbug disclosure + inspection-duty compliance check
// ---------------------------------------------------------------------------

async fn bedbug_disclosure_check_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<BedbugCheckInput>,
) -> Result<Json<BedbugCheckResult>, ApiError> {
    if b.state_code.trim().is_empty() {
        return Err(ApiError::BadRequest("state_code required".into()));
    }
    Ok(Json(check_bedbug_disclosure(&b)))
}

// ---------------------------------------------------------------------------
// State heat minimum temperature requirements check
// ---------------------------------------------------------------------------

async fn heat_requirements_check_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<HeatCheckInput>,
) -> Result<Json<HeatCheckResult>, ApiError> {
    if b.state_code.trim().is_empty() {
        return Err(ApiError::BadRequest("state_code required".into()));
    }
    if b.measurement_hour > 23 {
        return Err(ApiError::BadRequest(
            "measurement_hour must be 0-23".into(),
        ));
    }
    Ok(Json(check_heat_requirements(&b)))
}

// ---------------------------------------------------------------------------
// Foreclosure tenant rights (federal PTFA + state additions) check
// ---------------------------------------------------------------------------

async fn foreclosure_tenant_check_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<ForeclosureTenantInput>,
) -> Result<Json<ForeclosureTenantResult>, ApiError> {
    if b.state_code.trim().is_empty() {
        return Err(ApiError::BadRequest("state_code required".into()));
    }
    Ok(Json(check_foreclosure_tenant(&b)))
}

// ---------------------------------------------------------------------------
// State lead-based paint disclosure compliance check
// ---------------------------------------------------------------------------

async fn lead_disclosure_check_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<LeadCheckInput>,
) -> Result<Json<LeadCheckResult>, ApiError> {
    if b.state_code.trim().is_empty() {
        return Err(ApiError::BadRequest("state_code required".into()));
    }
    Ok(Json(check_lead_disclosure(&b)))
}

// ---------------------------------------------------------------------------
// State smoke + CO detector compliance check
// ---------------------------------------------------------------------------

async fn detector_check_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<DetectorCheckInput>,
) -> Result<Json<DetectorCheckResult>, ApiError> {
    if b.state_code.trim().is_empty() {
        return Err(ApiError::BadRequest("state_code required".into()));
    }
    Ok(Json(check_detector(&b)))
}

// ---------------------------------------------------------------------------
// State source-of-income discrimination protection check
// ---------------------------------------------------------------------------

async fn soi_protection_check_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<SoiCheckInput>,
) -> Result<Json<SoiCheckResult>, ApiError> {
    if b.state_code.trim().is_empty() {
        return Err(ApiError::BadRequest("state_code required".into()));
    }
    Ok(Json(check_soi_protection(&b)))
}

// ---------------------------------------------------------------------------
// State just-cause eviction availability + relocation assistance check
// ---------------------------------------------------------------------------

async fn just_cause_check_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<JustCauseInput>,
) -> Result<Json<JustCauseResult>, ApiError> {
    if b.state_code.trim().is_empty() {
        return Err(ApiError::BadRequest("state_code required".into()));
    }
    if b.monthly_rent_cents < 0 {
        return Err(ApiError::BadRequest(
            "monthly_rent_cents must be >= 0".into(),
        ));
    }
    Ok(Json(check_just_cause(&b)))
}

// ---------------------------------------------------------------------------
// State domestic-violence early-termination compliance check
// ---------------------------------------------------------------------------

async fn dv_termination_check_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<DvEarlyTerminationInput>,
) -> Result<Json<DvEarlyTerminationResult>, ApiError> {
    if b.state_code.trim().is_empty() {
        return Err(ApiError::BadRequest("state_code required".into()));
    }
    Ok(Json(check_dv_termination(&b)))
}

// ---------------------------------------------------------------------------
// dv_survivor_lock_change: DV / sexual assault / stalking survivor mid-
// tenancy lock change rights. Four regimes: California (Cal. Civ. Code
// § 1941.5 perpetrator-NOT-co-tenant 24-hour landlord-pays + § 1941.6
// perpetrator-IS-co-tenant landlord-pays + 21-day reimbursement on tenant
// self-help + SB 1051 signed-statement acceptance); Texas (§ 92.156(d)
// turnover rekey at landlord expense + § 92.156(e) tenant-requested rekey
// at TENANT expense including DV survivor + § 92.016 DV early termination
// separately covered); Washington (RCW 59.18.575 tenant-paid + 7-day
// written notice to landlord with order or qualified-third-party record;
// signed statement alone NOT accepted); Default (common-law lease + state-
// specific DV statute; varies sharply). Distinct from dv_termination,
// lock_change_between_tenancies, landlord_harassment.
// ---------------------------------------------------------------------------

async fn dv_survivor_lock_change_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<DvLockChangeInput>,
) -> Result<Json<DvLockChangeResult>, ApiError> {
    if let Some(h) = b.hours_to_landlord_action {
        if h > 100_000 {
            return Err(ApiError::BadRequest(
                "hours_to_landlord_action looks invalid (>100000)".into(),
            ));
        }
    }
    if let Some(d) = b.days_to_landlord_reimbursement {
        if d > 100_000 {
            return Err(ApiError::BadRequest(
                "days_to_landlord_reimbursement looks invalid (>100000)".into(),
            ));
        }
    }
    if let Some(d) = b.days_to_landlord_notice {
        if d > 100_000 {
            return Err(ApiError::BadRequest(
                "days_to_landlord_notice looks invalid (>100000)".into(),
            ));
        }
    }
    Ok(Json(check_dv_survivor_lock_change(&b)))
}

// ---------------------------------------------------------------------------
// State self-help eviction (lockout / utility shutoff) penalty check
// ---------------------------------------------------------------------------

async fn lockout_penalty_check_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<LockoutPenaltyInput>,
) -> Result<Json<LockoutPenaltyResult>, ApiError> {
    if b.state_code.trim().is_empty() {
        return Err(ApiError::BadRequest("state_code required".into()));
    }
    if b.monthly_rent_cents < 0 || b.actual_damages_cents < 0 {
        return Err(ApiError::BadRequest(
            "monthly_rent_cents and actual_damages_cents must be >= 0".into(),
        ));
    }
    Ok(Json(check_lockout_penalty(&b)))
}

// ---------------------------------------------------------------------------
// State rental application/screening fee cap check
// ---------------------------------------------------------------------------

async fn application_fee_check_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<AppFeeCheckInput>,
) -> Result<Json<AppFeeCheckResult>, ApiError> {
    if b.state_code.trim().is_empty() {
        return Err(ApiError::BadRequest("state_code required".into()));
    }
    if b.monthly_rent_cents < 0 {
        return Err(ApiError::BadRequest("monthly_rent_cents must be >= 0".into()));
    }
    if let Some(c) = b.actual_screening_cost_cents {
        if c < 0 {
            return Err(ApiError::BadRequest(
                "actual_screening_cost_cents must be >= 0".into(),
            ));
        }
    }
    Ok(Json(check_application_fee(&b)))
}

// ---------------------------------------------------------------------------
// State anti-retaliation presumption-window check
// ---------------------------------------------------------------------------

async fn retaliation_check_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RetaliationCheckInput>,
) -> Result<Json<RetaliationCheckResult>, ApiError> {
    if b.state_code.trim().is_empty() {
        return Err(ApiError::BadRequest("state_code required".into()));
    }
    Ok(Json(check_retaliation(&b)))
}

// ---------------------------------------------------------------------------
// State landlord entry-notice compliance check
// ---------------------------------------------------------------------------

async fn entry_notice_check_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<EntryNoticeInput>,
) -> Result<Json<EntryNoticeResult>, ApiError> {
    if b.state_code.trim().is_empty() {
        return Err(ApiError::BadRequest("state_code required".into()));
    }
    Ok(Json(check_entry_notice(&b)))
}

// ---------------------------------------------------------------------------
// 1099-NEC contractor $600 threshold tracker
// ---------------------------------------------------------------------------

async fn contractor_1099_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<Contractor1099Input>,
) -> Result<Json<Contractor1099Report>, ApiError> {
    Ok(Json(compute_contractor_1099(&b)))
}

// ---------------------------------------------------------------------------
// State security-deposit-return window compliance check
// ---------------------------------------------------------------------------

async fn deposit_return_check_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<DepositReturnCheckInput>,
) -> Result<Json<DepositReturnCheckResult>, ApiError> {
    if b.state.trim().is_empty() {
        return Err(ApiError::BadRequest("state required".into()));
    }
    if b.deposit_amount < Decimal::ZERO || b.deductions_claimed < Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "deposit_amount and deductions_claimed must be >= 0".into(),
        ));
    }
    Ok(Json(check_deposit_return(&b)))
}

// ---------------------------------------------------------------------------
// State + federal lease disclosure requirements
// ---------------------------------------------------------------------------

async fn lease_disclosures_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<DisclosuresRequiredInput>,
) -> Result<Json<DisclosuresRequiredReport>, ApiError> {
    if b.state.trim().is_empty() {
        return Err(ApiError::BadRequest("state required".into()));
    }
    Ok(Json(lease_disclosures_required_for(&b)))
}

// ---------------------------------------------------------------------------
// State rent control / rent-increase compliance check
// ---------------------------------------------------------------------------

async fn rent_increase_check_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentIncreaseCheckInput>,
) -> Result<Json<RentIncreaseCheckResult>, ApiError> {
    if b.state.trim().is_empty() {
        return Err(ApiError::BadRequest("state required".into()));
    }
    if b.current_rent < Decimal::ZERO || b.proposed_new_rent < Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "current_rent and proposed_new_rent must be >= 0".into(),
        ));
    }
    Ok(Json(check_rent_increase(&b)))
}

// ---------------------------------------------------------------------------
// State habitability remedies available to tenants
// ---------------------------------------------------------------------------

async fn habitability_remedies_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<HabitabilityRemediesInput>,
) -> Result<Json<HabitabilityRemediesReport>, ApiError> {
    if b.state.trim().is_empty() {
        return Err(ApiError::BadRequest("state required".into()));
    }
    if b.monthly_rent < Decimal::ZERO {
        return Err(ApiError::BadRequest("monthly_rent must be >= 0".into()));
    }
    Ok(Json(compute_habitability_remedies(&b)))
}

// ---------------------------------------------------------------------------
// State security deposit cap compliance check
// ---------------------------------------------------------------------------

async fn security_deposit_cap_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<SecurityDepositCheckInput>,
) -> Result<Json<SecurityDepositCheckResult>, ApiError> {
    if b.state.trim().is_empty() {
        return Err(ApiError::BadRequest("state required".into()));
    }
    if b.monthly_rent < Decimal::ZERO || b.proposed_deposit_amount < Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "monthly_rent and proposed_deposit_amount must be >= 0".into(),
        ));
    }
    Ok(Json(check_security_deposit_cap(&b)))
}

// ---------------------------------------------------------------------------
// Federal SCRA + state military lease termination check
// ---------------------------------------------------------------------------

async fn military_termination_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<MilitaryTerminationCheckInput>,
) -> Result<Json<MilitaryTerminationCheckResult>, ApiError> {
    if b.state.trim().is_empty() {
        return Err(ApiError::BadRequest("state required".into()));
    }
    Ok(Json(check_military_termination(&b)))
}

async fn property_cost_segregation(
    State(s): State<AppState>,
    u: AuthUser,
    Path(property_id): Path<Uuid>,
    Json(b): Json<CostSegRequest>,
) -> Result<Json<CostSegReport>, ApiError> {
    ensure_property_owner(&s, u.id, property_id).await?;
    let (purchase, land, ptype): (Option<Decimal>, Option<Decimal>, String) =
        sqlx::query_as(
            "SELECT purchase_price, land_value, property_type::text
               FROM rental_properties WHERE id = $1",
        )
        .bind(property_id)
        .fetch_one(&s.pool)
        .await?;

    let depreciable_basis = match b.depreciable_basis {
        Some(v) => v,
        None => {
            let p = purchase.ok_or_else(|| ApiError::BadRequest(
                "no purchase_price on property; pass depreciable_basis explicitly".into()
            ))?;
            (p - land.unwrap_or(Decimal::ZERO)).max(Decimal::ZERO)
        }
    };

    let cost_seg_type = match b.cost_seg_type.as_deref() {
        Some(t) => parse_cost_seg_type(t)?,
        None => cost_seg_type_from_property(&ptype),
    };

    if depreciable_basis < Decimal::ZERO {
        return Err(ApiError::BadRequest("depreciable_basis must be >= 0".into()));
    }

    Ok(Json(compute_cost_segregation(&CostSegInput {
        depreciable_basis,
        property_type: cost_seg_type,
        allocation_override: b.allocation_override,
        tax_year: b.tax_year,
        elect_bonus_depreciation: b.elect_bonus_depreciation,
    })))
}

async fn property_disposition(
    State(s): State<AppState>,
    u: AuthUser,
    Path(property_id): Path<Uuid>,
    Json(b): Json<DispositionRequest>,
) -> Result<Json<DispositionReport>, ApiError> {
    ensure_property_owner(&s, u.id, property_id).await?;
    if b.sale_price < Decimal::ZERO || b.selling_costs < Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "sale_price and selling_costs must be >= 0".into(),
        ));
    }
    // Fill missing basis from rental_properties.purchase_price.
    let original_cost_basis = match b.original_cost_basis {
        Some(v) => v,
        None => {
            let (p,): (Option<Decimal>,) = sqlx::query_as(
                "SELECT purchase_price FROM rental_properties WHERE id = $1",
            )
            .bind(property_id)
            .fetch_one(&s.pool)
            .await?;
            p.ok_or_else(|| ApiError::BadRequest(
                "no purchase_price on property; pass original_cost_basis explicitly".into()
            ))?
        }
    };
    // Fill missing accumulated depreciation from sum of capitalized
    // improvements being recovered + actual e_depreciation expense rows.
    let accumulated_depreciation = match b.accumulated_depreciation {
        Some(v) => v,
        None => {
            let (sum,): (Option<Decimal>,) = sqlx::query_as(
                "SELECT COALESCE(SUM(amount), 0) FROM rental_expenses
                  WHERE property_id = $1 AND category_code = 'e_depreciation'",
            )
            .bind(property_id)
            .fetch_one(&s.pool)
            .await?;
            sum.unwrap_or(Decimal::ZERO)
        }
    };
    let input = DispositionInput {
        sale_price: b.sale_price,
        selling_costs: b.selling_costs,
        original_cost_basis,
        accumulated_depreciation,
        capital_improvements_added: b.capital_improvements_added.unwrap_or(Decimal::ZERO),
        like_kind_exchange: b.like_kind_exchange,
        filing_status: b.filing_status,
    };
    Ok(Json(compute_disposition(&input)))
}

// ---------------------------------------------------------------------------
// QBI 250-hour safe-harbor report
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct QbiHoursReport {
    year: i32,
    hours_logged: Decimal,
    hours_required: Decimal,
    hours_remaining: Decimal,
    qualifies: bool,
}

async fn qbi_hours_report(
    State(s): State<AppState>,
    u: AuthUser,
    Path(property_id): Path<Uuid>,
    Query(q): Query<ReportQuery>,
) -> Result<Json<QbiHoursReport>, ApiError> {
    ensure_property_owner(&s, u.id, property_id).await?;
    let start = NaiveDate::from_ymd_opt(q.year, 1, 1)
        .ok_or_else(|| ApiError::BadRequest("invalid year".into()))?;
    let end = NaiveDate::from_ymd_opt(q.year + 1, 1, 1)
        .ok_or_else(|| ApiError::BadRequest("invalid year".into()))?;
    let (total,): (Option<Decimal>,) = sqlx::query_as(
        "SELECT SUM(hours) FROM rental_services_log
          WHERE property_id = $1 AND performed_on >= $2 AND performed_on < $3",
    )
    .bind(property_id)
    .bind(start)
    .bind(end)
    .fetch_one(&s.pool)
    .await?;
    let logged = total.unwrap_or(Decimal::ZERO);
    let req = Decimal::from(250);
    let remaining = if logged >= req { Decimal::ZERO } else { req - logged };
    Ok(Json(QbiHoursReport {
        year: q.year,
        hours_logged: logged,
        hours_required: req,
        hours_remaining: remaining,
        qualifies: logged >= req,
    }))
}

// ---------------------------------------------------------------------------
// Rent roll: per-lease expected vs collected for a month window
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct RentRollRow {
    lease_id: Uuid,
    tenant_name: String,
    unit_label: String,
    rent_amount: Decimal,
    rent_due_day: i32,
    grace_days: i32,
    expected: Decimal,
    collected: Decimal,
    balance: Decimal,
    status: String, // paid | partial | due | late
}

#[derive(Deserialize)]
struct RentRollQuery {
    year: i32,
    month: u32,
}

async fn rent_roll(
    State(s): State<AppState>,
    u: AuthUser,
    Path(property_id): Path<Uuid>,
    Query(q): Query<RentRollQuery>,
) -> Result<Json<Vec<RentRollRow>>, ApiError> {
    ensure_property_owner(&s, u.id, property_id).await?;
    if !(1..=12).contains(&q.month) {
        return Err(ApiError::BadRequest("month must be 1..12".into()));
    }
    let start = NaiveDate::from_ymd_opt(q.year, q.month, 1)
        .ok_or_else(|| ApiError::BadRequest("invalid date".into()))?;
    let (next_y, next_m) = if q.month == 12 { (q.year + 1, 1) } else { (q.year, q.month + 1) };
    let end = NaiveDate::from_ymd_opt(next_y, next_m, 1)
        .ok_or_else(|| ApiError::BadRequest("invalid date".into()))?;

    // Active leases overlapping the window.
    let leases: Vec<LeaseRentRollRow> =
        sqlx::query_as(
            "SELECT l.id, COALESCE(t.display_name, ''), l.unit_label, l.rent_amount,
                    l.rent_due_day, l.grace_days, l.starts_on, l.ends_on
               FROM rental_leases l
               LEFT JOIN rental_tenants t ON t.id = l.tenant_id
              WHERE l.property_id = $1
                AND l.status = 'active'
                AND l.starts_on < $3
                AND (l.ends_on IS NULL OR l.ends_on >= $2)",
        )
        .bind(property_id)
        .bind(start)
        .bind(end)
        .fetch_all(&s.pool)
        .await?;

    let mut rows = Vec::with_capacity(leases.len());
    for (lid, tname, unit, rent, due_day, grace, _starts, _ends) in leases {
        // Collected = rent-kind income posted in window for this lease.
        let (col,): (Option<Decimal>,) = sqlx::query_as(
            "SELECT SUM(amount) FROM rental_income
              WHERE lease_id = $1 AND kind = 'rent'
                AND posted_at >= $2 AND posted_at < $3",
        )
        .bind(lid)
        .bind(start)
        .bind(end)
        .fetch_one(&s.pool)
        .await?;
        let collected = col.unwrap_or(Decimal::ZERO);
        let expected = rent;
        let balance = expected - collected;
        let today = Utc::now().date_naive();
        let due_date = NaiveDate::from_ymd_opt(q.year, q.month, due_day.min(28) as u32)
            .unwrap_or(start);
        let late_threshold = due_date + chrono::Duration::days(grace as i64);
        let status = if collected >= expected {
            "paid"
        } else if collected > Decimal::ZERO {
            if today > late_threshold { "late" } else { "partial" }
        } else if today > late_threshold {
            "late"
        } else {
            "due"
        }
        .to_string();
        rows.push(RentRollRow {
            lease_id: lid,
            tenant_name: tname,
            unit_label: unit,
            rent_amount: rent,
            rent_due_day: due_day,
            grace_days: grace,
            expected,
            collected,
            balance,
            status,
        });
    }
    Ok(Json(rows))
}
