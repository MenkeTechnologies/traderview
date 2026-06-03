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
use traderview_expense::rental_dc_topa_tenant_opportunity_purchase::{
    check as check_rental_dc_topa_tenant_opportunity_purchase,
    DcTopaInput as RentalDcTopaTenantOpportunityPurchaseInput,
    DcTopaResult as RentalDcTopaTenantOpportunityPurchaseResult,
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
use traderview_expense::security_deposit_interest_statement::{
    check as check_security_deposit_interest_statement, DepositInterestStatementInput,
    DepositInterestStatementResult,
};
use traderview_expense::rent_control::{
    check as check_rent_increase, RentIncreaseCheckInput, RentIncreaseCheckResult,
};
use traderview_expense::rent_control_lease_disclosure::{
    check as check_rent_control_disclosure, RentControlDisclosureInput, RentControlDisclosureResult,
};
use traderview_expense::rent_overcharge_recovery::{
    check as check_rent_overcharge_recovery, RentOverchargeRecoveryInput,
    RentOverchargeRecoveryResult,
};
use traderview_expense::rubs_utility_billing_disclosure::{
    check as check_rubs_utility_billing_disclosure, RubsUtilityBillingInput,
    RubsUtilityBillingResult,
};
use traderview_expense::entry_notice::{
    compute as check_entry_notice, EntryNoticeInput, EntryNoticeResult,
};
use traderview_expense::broker_fee_allocation::{
    check as check_broker_fee_allocation, BrokerFeeAllocationInput, BrokerFeeAllocationResult,
};
use traderview_expense::application_fees::{
    check as check_application_fee, AppFeeCheckInput, AppFeeCheckResult,
};
use traderview_expense::balcony_inspection::{
    check as check_balcony_inspection, BalconyInspectionInput, BalconyInspectionResult,
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
use traderview_expense::just_cause_termination_notice_content::{
    check as check_just_cause_notice_content, JustCauseNoticeContentInput, JustCauseNoticeContentResult,
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
use traderview_expense::lead_in_drinking_water_disclosure::{
    check as check_lead_in_drinking_water_disclosure, LeadInDrinkingWaterInput,
    LeadInDrinkingWaterResult,
};
use traderview_expense::lead_renovation_repair_painting::{
    check as check_lead_rrp, RrpInput, RrpResult,
};
use traderview_expense::soi_protection::{
    check as check_soi_protection, SoiCheckInput, SoiCheckResult,
};
use traderview_expense::squatter_unauthorized_occupant_removal::{
    check as check_squatter_removal, SquatterRemovalInput, SquatterRemovalResult,
};
use traderview_expense::str_regulation::{
    check as check_str_regulation, StrComplianceInput, StrComplianceResult,
};
use traderview_expense::pet_fees::{
    check as check_pet_fees, PetFeeInput, PetFeeResult,
};
use traderview_expense::non_refundable_cleaning_fees::{
    check as check_non_refundable_cleaning_fees, CleaningFeeInput, CleaningFeeResult,
};
use traderview_expense::eviction_record_sealing::{
    check as check_eviction_sealing, EvictionSealingInput, EvictionSealingResult,
};
use traderview_expense::lease_termination_catastrophic_damage::{
    check as check_catastrophic_damage, CatastrophicDamageInput, CatastrophicDamageResult,
};
use traderview_expense::lease_termination_notice::{
    check as check_termination_notice, NoticeInput, NoticeResult,
};
use traderview_expense::occupancy_standards::{
    check as check_occupancy, OccupancyInput, OccupancyResult,
};
use traderview_expense::move_in_fee_cap::{
    check as check_move_in_fee_cap, MoveInFeeCapInput, MoveInFeeCapResult,
};
use traderview_expense::move_in_inspection::{
    check as check_move_in_inspection, InspectionInput, InspectionResult,
};
use traderview_expense::mandatory_renters_insurance_provider_choice::{
    check as check_mandatory_renters_insurance_provider_choice,
    MandatoryRentersInsuranceInput, MandatoryRentersInsuranceResult,
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
use traderview_expense::water_heater_earthquake_strap::{
    check as check_water_heater_earthquake_strap, WaterHeaterEarthquakeStrapInput,
    WaterHeaterEarthquakeStrapResult,
};
use traderview_expense::adverse_action_notice::{
    check as check_adverse_action, AdverseActionInput, AdverseActionResult,
};
use traderview_expense::adverse_possession_claim::{
    check as check_adverse_possession, AdversePossessionInput, AdversePossessionResult,
};
use traderview_expense::tenant_rent_escrow_withholding::{
    check as check_tenant_rent_escrow_withholding,
    TenantRentEscrowWithholdingInput, TenantRentEscrowWithholdingResult,
};
use traderview_expense::tenant_rent_judgment_wage_garnishment::{
    compute as compute_wage_garnishment, GarnishmentInput, GarnishmentResult,
};
use traderview_expense::tenant_rent_receipt_requirement::{
    check as check_tenant_rent_receipt_requirement,
    TenantRentReceiptRequirementInput, TenantRentReceiptRequirementResult,
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
use traderview_expense::rent_stabilized_mci_iai_passthrough::{
    check as check_rent_stabilized_mci_iai_passthrough, RentStabilizedPassthroughInput,
    RentStabilizedPassthroughResult,
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
use traderview_expense::tenant_domestic_violence_lease_termination::{
    check as check_tenant_domestic_violence_lease_termination,
    TenantDomesticViolenceLeaseTerminationInput,
    TenantDomesticViolenceLeaseTerminationResult,
};
use traderview_expense::tenant_ev_charging_installation_right::{
    check as check_tenant_ev_charging_installation_right,
    TenantEvChargingInstallationRightInput, TenantEvChargingInstallationRightResult,
};
use traderview_expense::tenant_fire_safety_plan_disclosure::{
    check as check_tenant_fire_safety_plan_disclosure,
    TenantFireSafetyPlanDisclosureInput, TenantFireSafetyPlanDisclosureResult,
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
use traderview_expense::family_childcare_home_right::{
    check as check_family_childcare_home_right, FamilyChildcareHomeInput,
    FamilyChildcareHomeResult,
};
use traderview_expense::source_of_income_discrimination::{
    check as check_source_of_income_discrimination, SourceOfIncomeInput,
    SourceOfIncomeResult,
};
use traderview_expense::fha_design_construction::{
    check as check_fha_design_construction, FhaDesignConstructionInput,
    FhaDesignConstructionResult,
};
use traderview_expense::meth_contamination_disclosure::{
    check as check_meth_contamination_disclosure, MethDisclosureInput, MethDisclosureResult,
};
use traderview_expense::death_in_unit_disclosure::{
    check as check_death_in_unit_disclosure, DeathDisclosureInput, DeathDisclosureResult,
};
use traderview_expense::dog_breed_restriction_ban::{
    check as check_dog_breed_restriction_ban, DogBreedRestrictionBanInput,
    DogBreedRestrictionBanResult,
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
use traderview_expense::tenant_clothesline_drying_right::{
    check as check_tenant_clothesline_drying_right, TenantClotheslineDryingRightInput,
    TenantClotheslineDryingRightResult,
};
use traderview_expense::snow_removal_responsibility::{
    check as check_snow_removal_responsibility, SnowRemovalInput, SnowRemovalResult,
};
use traderview_expense::soft_story_seismic_retrofit::{
    check as check_soft_story_seismic_retrofit, SoftStorySeismicRetrofitInput,
    SoftStorySeismicRetrofitResult,
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
use traderview_expense::lease_early_termination_fee_cap::{
    check as check_lease_early_termination_fee_cap, LeaseEarlyTerminationFeeInput,
    LeaseEarlyTerminationFeeResult,
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
use traderview_expense::short_term_rental_conversion::{
    check as check_short_term_rental_conversion, ShortTermRentalConversionInput,
    ShortTermRentalConversionResult,
};
use traderview_expense::mid_tenancy_ownership_change::{
    check as check_mid_tenancy_ownership_change, CheckResult as MidTenancyOwnershipResult,
    Input as MidTenancyOwnershipInput,
};
use traderview_expense::mid_tenancy_security_deposit_increase::{
    check as check_mid_tenancy_security_deposit_increase, MidTenancySecurityDepositInput,
    MidTenancySecurityDepositResult,
};
use traderview_expense::mid_tenancy_temporary_relocation::{
    check as check_mid_tenancy_temporary_relocation, TemporaryRelocationInput,
    TemporaryRelocationResult,
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
use traderview_expense::hoa_fee_tenant_enforcement::{
    check as check_hoa_fee_tenant_enforcement, HoaFeeTenantEnforcementInput,
    HoaFeeTenantEnforcementResult,
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
use traderview_expense::tenant_in_unit_appliance_repair_responsibility::{
    check as check_tenant_in_unit_appliance_repair_responsibility,
    TenantInUnitApplianceRepairResponsibilityInput,
    TenantInUnitApplianceRepairResponsibilityResult,
};
use traderview_expense::tenant_late_fee_cap::{
    check as check_tenant_late_fee_cap,
    TenantLateFeeCapInput, TenantLateFeeCapResult,
};
use traderview_expense::tenant_lease_guarantor_disclosure::{
    check as check_tenant_lease_guarantor_disclosure,
    TenantLeaseGuarantorDisclosureInput, TenantLeaseGuarantorDisclosureResult,
};
use traderview_expense::tenant_estoppel_certificate::{
    check as check_tenant_estoppel_certificate,
    TenantEstoppelCertificateInput, TenantEstoppelCertificateResult,
};
use traderview_expense::landlord_property_sale_notice::{
    check as check_landlord_property_sale_notice,
    LandlordPropertySaleNoticeInput, LandlordPropertySaleNoticeResult,
};
use traderview_expense::lease_renewal_offer_timing::{
    check as check_lease_renewal_offer_timing,
    LeaseRenewalOfferTimingInput, LeaseRenewalOfferTimingResult,
};
use traderview_expense::rent_concession_disclosure::{
    check as check_rent_concession_disclosure,
    RentConcessionDisclosureInput, RentConcessionDisclosureResult,
};
use traderview_expense::rent_abatement_construction_nuisance::{
    check as check_rent_abatement_construction_nuisance,
    RentAbatementConstructionNuisanceInput, RentAbatementConstructionNuisanceResult,
};
use traderview_expense::landlord_master_key_retention::{
    check as check_landlord_master_key_retention,
    LandlordMasterKeyRetentionInput, LandlordMasterKeyRetentionResult,
};
use traderview_expense::tenant_holdover_security_deposit_setoff::{
    check as check_tenant_holdover_security_deposit_setoff,
    TenantHoldoverSecurityDepositSetoffInput, TenantHoldoverSecurityDepositSetoffResult,
};
use traderview_expense::rental_video_surveillance_retention::{
    check as check_rental_video_surveillance_retention,
    RentalVideoSurveillanceRetentionInput, RentalVideoSurveillanceRetentionResult,
};
use traderview_expense::landlord_foreclosure_status_disclosure::{
    check as check_landlord_foreclosure_status_disclosure,
    LandlordForeclosureStatusDisclosureInput, LandlordForeclosureStatusDisclosureResult,
};
use traderview_expense::commercial_lease_personal_guaranty_enforceability::{
    check as check_commercial_lease_personal_guaranty_enforceability,
    CommercialLeasePersonalGuarantyEnforceabilityInput,
    CommercialLeasePersonalGuarantyEnforceabilityResult,
};
use traderview_expense::commercial_lease_cam_charge_disclosure::{
    check as check_commercial_lease_cam_charge_disclosure,
    CommercialLeaseCamChargeDisclosureInput, CommercialLeaseCamChargeDisclosureResult,
};
use traderview_expense::landlord_pest_extermination_timeline::{
    check as check_landlord_pest_extermination_timeline,
    LandlordPestExterminationTimelineInput, LandlordPestExterminationTimelineResult,
};
use traderview_expense::landlord_water_heat_emergency_response::{
    check as check_landlord_water_heat_emergency_response,
    LandlordWaterHeatEmergencyResponseInput, LandlordWaterHeatEmergencyResponseResult,
};
use traderview_expense::tenant_emotional_distress_damages::{
    check as check_tenant_emotional_distress_damages,
    TenantEmotionalDistressDamagesInput, TenantEmotionalDistressDamagesResult,
};
use traderview_expense::landlord_negative_credit_reporting::{
    check as check_landlord_negative_credit_reporting,
    LandlordNegativeCreditReportingInput, LandlordNegativeCreditReportingResult,
};
use traderview_expense::security_deposit_bank_disclosure::{
    check as check_security_deposit_bank_disclosure,
    CheckResult as SecurityDepositBankDisclosureResult,
    Input as SecurityDepositBankDisclosureInput,
};
use traderview_expense::landlord_annual_rent_statement::{
    check as check_landlord_annual_rent_statement, LandlordAnnualRentStatementInput,
    LandlordAnnualRentStatementResult,
};
use traderview_expense::landlord_emergency_entry_notice::{
    check as check_landlord_emergency_entry_notice, LandlordEmergencyEntryInput,
    LandlordEmergencyEntryResult,
};
use traderview_expense::landlord_mid_tenancy_rekeying::{
    check as check_landlord_mid_tenancy_rekeying, LandlordMidTenancyRekeyingInput,
    LandlordMidTenancyRekeyingResult,
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
use traderview_expense::landlord_post_eviction_tenant_property_storage_disposal::{
    check as check_landlord_post_eviction_tenant_property_storage_disposal,
    LandlordPostEvictionTenantPropertyStorageDisposalInput,
    LandlordPostEvictionTenantPropertyStorageDisposalResult,
};
use traderview_expense::rental_asbestos_disclosure::{
    check as check_rental_asbestos_disclosure,
    RentalAsbestosDisclosureInput,
    RentalAsbestosDisclosureResult,
};
use traderview_expense::rental_application_denial_disclosure::{
    check as check_rental_application_denial_disclosure,
    RentalApplicationDenialDisclosureInput, RentalApplicationDenialDisclosureResult,
};
use traderview_expense::rental_basement_water_intrusion_disclosure::{
    check as check_rental_basement_water_intrusion_disclosure,
    RentalBasementWaterIntrusionDisclosureInput,
    RentalBasementWaterIntrusionDisclosureResult,
};
use traderview_expense::rental_bed_bug_disclosure::{
    check as check_rental_bed_bug_disclosure, RentalBedBugDisclosureInput,
    RentalBedBugDisclosureResult,
};
use traderview_expense::rental_bedroom_egress_window::{
    check as check_rental_bedroom_egress_window, RentalBedroomEgressWindowInput,
    RentalBedroomEgressWindowResult,
};
use traderview_expense::rental_berkeley_rent_stabilization_ordinance_bmc_chapter_13_76::{
    check as check_rental_berkeley_rent_stabilization_ordinance_bmc_chapter_13_76,
    RentalBerkeleyRentStabilizationOrdinanceBmcChapter1376Input,
    RentalBerkeleyRentStabilizationOrdinanceBmcChapter1376Result,
};
use traderview_expense::rental_elevator_safety_inspection::{
    check as check_rental_elevator_safety_inspection,
    RentalElevatorSafetyInspectionInput, RentalElevatorSafetyInspectionResult,
};
use traderview_expense::rental_fire_extinguisher_requirement::{
    check as check_rental_fire_extinguisher_requirement,
    RentalFireExtinguisherRequirementInput, RentalFireExtinguisherRequirementResult,
};
use traderview_expense::rental_flood_hazard_disclosure::{
    check as check_rental_flood_hazard_disclosure,
    FloodHazardDisclosureInput as RentalFloodHazardDisclosureInput,
    FloodHazardDisclosureResult as RentalFloodHazardDisclosureResult,
};
use traderview_expense::rental_florida_hb_1417_state_preemption::{
    check as check_rental_florida_hb_1417_state_preemption,
    RentalFloridaHb1417StatePreemptionInput,
    RentalFloridaHb1417StatePreemptionResult,
};
use traderview_expense::rental_foreclosure_tenant_protection_ptfa::{
    check as check_rental_foreclosure_tenant_protection_ptfa,
    PtfaInput as RentalForeclosureTenantProtectionPtfaInput,
    PtfaResult as RentalForeclosureTenantProtectionPtfaResult,
};
use traderview_expense::rental_carbon_monoxide_detector::{
    check as check_rental_carbon_monoxide_detector, RentalCarbonMonoxideDetectorInput,
    RentalCarbonMonoxideDetectorResult,
};
use traderview_expense::rental_california_ab_12_security_deposit_cap::{
    check as check_rental_california_ab_12_security_deposit_cap,
    RentalCaliforniaAb12SecurityDepositCapInput,
    RentalCaliforniaAb12SecurityDepositCapResult,
};
use traderview_expense::rental_california_ab_2347_unlawful_detainer_response::{
    check as check_rental_california_ab_2347_unlawful_detainer_response,
    RentalCaliforniaAb2347UnlawfulDetainerResponseInput,
    RentalCaliforniaAb2347UnlawfulDetainerResponseResult,
};
use traderview_expense::rental_california_sb_567_no_fault_eviction_amendments::{
    check as check_rental_california_sb_567_no_fault_eviction_amendments,
    RentalCaliforniaSb567NoFaultEvictionAmendmentsInput,
    RentalCaliforniaSb567NoFaultEvictionAmendmentsResult,
};
use traderview_expense::rental_chimney_fireplace_inspection_disclosure::{
    check as check_rental_chimney_fireplace_inspection_disclosure,
    RentalChimneyFireplaceInspectionDisclosureInput,
    RentalChimneyFireplaceInspectionDisclosureResult,
};
use traderview_expense::rental_climate_mobilization_act_ll97_emissions::{
    check as check_rental_climate_mobilization_act_ll97_emissions,
    ClimateMobilizationActLl97EmissionsInput as RentalClimateMobilizationActLl97EmissionsInput,
    ClimateMobilizationActLl97EmissionsResult as RentalClimateMobilizationActLl97EmissionsResult,
};
use traderview_expense::rental_colorado_hb_24_1098_just_cause_eviction::{
    check as check_rental_colorado_hb_24_1098_just_cause_eviction,
    RentalColoradoHb241098JustCauseEvictionInput,
    RentalColoradoHb241098JustCauseEvictionResult,
};
use traderview_expense::rental_connecticut_fair_rent_commission::{
    check as check_rental_connecticut_fair_rent_commission,
    RentalConnecticutFairRentCommissionInput,
    RentalConnecticutFairRentCommissionResult,
};
use traderview_expense::rental_cook_county_rtlo::{
    check as check_rental_cook_county_rtlo,
    RentalCookCountyRtloInput, RentalCookCountyRtloResult,
};
use traderview_expense::rental_cooling_tower_inspection_local_law_77::{
    check as check_rental_cooling_tower_inspection_local_law_77,
    CoolingTowerInspectionLocalLaw77Input as RentalCoolingTowerInspectionLocalLaw77Input,
    CoolingTowerInspectionLocalLaw77Result as RentalCoolingTowerInspectionLocalLaw77Result,
};
use traderview_expense::rental_broadband_mte_rules::{
    check as check_rental_broadband_mte_rules, RentalBroadbandMteRulesInput,
    RentalBroadbandMteRulesResult,
};
use traderview_expense::rental_energy_benchmarking::{
    check as check_rental_energy_benchmarking, RentalEnergyBenchmarkingInput,
    RentalEnergyBenchmarkingResult,
};
use traderview_expense::rental_garage_door_safety_compliance::{
    check as check_rental_garage_door_safety_compliance,
    RentalGarageDoorSafetyComplianceInput, RentalGarageDoorSafetyComplianceResult,
};
use traderview_expense::rental_gas_appliance_ban::{
    check as check_rental_gas_appliance_ban, RentalGasApplianceBanInput,
    RentalGasApplianceBanResult,
};
use traderview_expense::rental_gas_piping_inspection_local_law_152::{
    check as check_rental_gas_piping_inspection_local_law_152,
    GasPipingInspectionLocalLaw152Input as RentalGasPipingInspectionLocalLaw152Input,
    GasPipingInspectionLocalLaw152Result as RentalGasPipingInspectionLocalLaw152Result,
};
use traderview_expense::rental_hardwired_smoke_alarm_responsibility::{
    check as check_rental_hardwired_smoke_alarm_responsibility,
    RentalHardwiredSmokeAlarmResponsibilityInput,
    RentalHardwiredSmokeAlarmResponsibilityResult,
};
use traderview_expense::rental_hawaii_residential_landlord_tenant_code_hrs_521::{
    check as check_rental_hawaii_residential_landlord_tenant_code_hrs_521,
    RentalHawaiiResidentialLandlordTenantCodeHrs521Input,
    RentalHawaiiResidentialLandlordTenantCodeHrs521Result,
};
use traderview_expense::rental_heat_minimum_temperature_season::{
    check as check_rental_heat_minimum_temperature_season,
    HeatMinimumTemperatureInput as RentalHeatMinimumTemperatureInput,
    HeatMinimumTemperatureResult as RentalHeatMinimumTemperatureResult,
};
use traderview_expense::rental_hot_water_temperature::{
    check as check_rental_hot_water_temperature, RentalHotWaterTemperatureInput,
    RentalHotWaterTemperatureResult,
};
use traderview_expense::rental_housing_for_older_persons_act_hopa_1995::{
    check as check_rental_housing_for_older_persons_act_hopa_1995,
    RentalHousingForOlderPersonsActHopa1995Input,
    RentalHousingForOlderPersonsActHopa1995Result,
};
use traderview_expense::rental_hud_hotma_income_asset_compliance::{
    check as check_rental_hud_hotma_income_asset_compliance,
    RentalHudHotmaIncomeAssetComplianceInput,
    RentalHudHotmaIncomeAssetComplianceResult,
};
use traderview_expense::rental_hud_section_504_rehabilitation_act_24_cfr_part_8::{
    check as check_rental_hud_section_504_rehabilitation_act_24_cfr_part_8,
    RentalHudSection504RehabilitationAct24CfrPart8Input,
    RentalHudSection504RehabilitationAct24CfrPart8Result,
};
use traderview_expense::rental_junk_fee_transparency::{
    check as check_rental_junk_fee_transparency, RentalJunkFeeTransparencyInput,
    RentalJunkFeeTransparencyResult,
};
use traderview_expense::rental_just_cause_eviction::{
    check as check_rental_just_cause_eviction,
    JustCauseEvictionInput as RentalJustCauseEvictionInput,
    JustCauseEvictionResult as RentalJustCauseEvictionResult,
};
use traderview_expense::rental_water_submetering_disclosure::{
    check as check_rental_water_submetering_disclosure,
    RentalWaterSubmeteringDisclosureInput, RentalWaterSubmeteringDisclosureResult,
};
use traderview_expense::rental_well_water_disclosure::{
    check as check_rental_well_water_disclosure,
    RentalWellWaterDisclosureInput, RentalWellWaterDisclosureResult,
};
use traderview_expense::rental_window_blind_cord_safety::{
    check as check_rental_window_blind_cord_safety,
    RentalWindowBlindCordSafetyInput, RentalWindowBlindCordSafetyResult,
};
use traderview_expense::rental_window_guard_installation::{
    check as check_rental_window_guard_installation,
    RentalWindowGuardInstallationInput, RentalWindowGuardInstallationResult,
};
use traderview_expense::rental_vehicle_towing_notice_sign_requirements::{
    check as check_rental_vehicle_towing_notice_sign_requirements,
    RentalVehicleTowingNoticeSignRequirementsInput,
    RentalVehicleTowingNoticeSignRequirementsResult,
};
use traderview_expense::rental_vacant_property_registration::{
    check as check_rental_vacant_property_registration,
    RentalVacantPropertyRegistrationInput,
    RentalVacantPropertyRegistrationResult,
};
use traderview_expense::rental_vawa_2022_federal_housing_protections::{
    check as check_rental_vawa_2022_federal_housing_protections,
    RentalVawa2022FederalHousingProtectionsInput,
    RentalVawa2022FederalHousingProtectionsResult,
};
use traderview_expense::rental_unpermitted_unit_disclosure::{
    check as check_rental_unpermitted_unit_disclosure,
    RentalUnpermittedUnitDisclosureInput, RentalUnpermittedUnitDisclosureResult,
};
use traderview_expense::rental_sex_offender_registry_notice::{
    check as check_rental_sex_offender_registry_notice,
    RentalSexOffenderRegistryNoticeInput, RentalSexOffenderRegistryNoticeResult,
};
use traderview_expense::rental_sinkhole_disclosure::{
    check as check_rental_sinkhole_disclosure, RentalSinkholeDisclosureInput,
    RentalSinkholeDisclosureResult,
};
use traderview_expense::rental_smoke_free_housing_disclosure::{
    check as check_rental_smoke_free_housing_disclosure,
    RentalSmokeFreeHousingDisclosureInput, RentalSmokeFreeHousingDisclosureResult,
};
use traderview_expense::rental_soft_story_seismic_retrofit::{
    check as check_rental_soft_story_seismic_retrofit,
    SoftStorySeismicRetrofitInput as RentalSoftStorySeismicRetrofitInput,
    SoftStorySeismicRetrofitResult as RentalSoftStorySeismicRetrofitResult,
};
use traderview_expense::rental_swimming_pool_drain_safety::{
    check as check_rental_swimming_pool_drain_safety,
    RentalSwimmingPoolDrainSafetyInput, RentalSwimmingPoolDrainSafetyResult,
};
use traderview_expense::rental_underground_storage_tank_disclosure::{
    check as check_rental_underground_storage_tank_disclosure,
    RentalUndergroundStorageTankDisclosureInput,
    RentalUndergroundStorageTankDisclosureResult,
};
use traderview_expense::rental_san_francisco_rent_ordinance_chapter_37::{
    check as check_rental_san_francisco_rent_ordinance_chapter_37,
    RentalSanFranciscoRentOrdinanceChapter37Input,
    RentalSanFranciscoRentOrdinanceChapter37Result,
};
use traderview_expense::rental_satellite_dish_installation_right::{
    check as check_rental_satellite_dish_installation_right,
    RentalSatelliteDishInstallationRightInput, RentalSatelliteDishInstallationRightResult,
};
use traderview_expense::rental_seattle_smc_22_206_160_just_cause_eviction::{
    check as check_rental_seattle_smc_22_206_160_just_cause_eviction,
    RentalSeattleSmc22206160JustCauseEvictionInput,
    RentalSeattleSmc22206160JustCauseEvictionResult,
};
use traderview_expense::rental_security_deposit_interest::{
    check as check_rental_security_deposit_interest,
    SecurityDepositInterestInput as RentalSecurityDepositInterestInput,
    SecurityDepositInterestResult as RentalSecurityDepositInterestResult,
};
use traderview_expense::rental_septic_system_disclosure::{
    check as check_rental_septic_system_disclosure,
    RentalSepticSystemDisclosureInput, RentalSepticSystemDisclosureResult,
};
use traderview_expense::rental_sewer_lateral_responsibility::{
    check as check_rental_sewer_lateral_responsibility,
    RentalSewerLateralResponsibilityInput, RentalSewerLateralResponsibilityResult,
};
use traderview_expense::rental_pesticide_application_notification::{
    check as check_rental_pesticide_application_notification,
    RentalPesticideApplicationNotificationInput,
    RentalPesticideApplicationNotificationResult,
};
use traderview_expense::rental_pet_deposit_separate_security::{
    check as check_rental_pet_deposit_separate_security,
    RentalPetDepositSeparateSecurityInput, RentalPetDepositSeparateSecurityResult,
};
use traderview_expense::rental_pre_foreclosure_tenant_notification::{
    check as check_rental_pre_foreclosure_tenant_notification,
    RentalPreForeclosureTenantNotificationInput,
    RentalPreForeclosureTenantNotificationResult,
};
use traderview_expense::rental_propane_tank_lease_disclosure::{
    check as check_rental_propane_tank_lease_disclosure,
    RentalPropaneTankLeaseDisclosureInput, RentalPropaneTankLeaseDisclosureResult,
};
use traderview_expense::rental_organic_waste_collection_disclosure::{
    check as check_rental_organic_waste_collection_disclosure,
    RentalOrganicWasteCollectionDisclosureInput, RentalOrganicWasteCollectionDisclosureResult,
};
use traderview_expense::rental_lead_paint_disclosure::{
    check as check_rental_lead_paint_disclosure, LeadPaintDisclosureInput,
    LeadPaintDisclosureResult,
};
use traderview_expense::rental_lead_pipe_disclosure::{
    check as check_rental_lead_pipe_disclosure, RentalLeadPipeDisclosureInput,
    RentalLeadPipeDisclosureResult,
};
use traderview_expense::rental_natural_gas_leak_response::{
    check as check_rental_natural_gas_leak_response,
    RentalNaturalGasLeakResponseInput, RentalNaturalGasLeakResponseResult,
};
use traderview_expense::rental_new_jersey_anti_eviction_act::{
    check as check_rental_new_jersey_anti_eviction_act,
    RentalNewJerseyAntiEvictionActInput,
    RentalNewJerseyAntiEvictionActResult,
};
use traderview_expense::rental_ny_rent_receipt_late_notice_requirements::{
    check as check_rental_ny_rent_receipt_late_notice_requirements,
    NyRentReceiptLateNoticeRequirementsInput as RentalNyRentReceiptLateNoticeRequirementsInput,
    NyRentReceiptLateNoticeRequirementsResult as RentalNyRentReceiptLateNoticeRequirementsResult,
};
use traderview_expense::rental_attorney_fee_clause_reciprocity::{
    check as check_rental_attorney_fee_clause_reciprocity,
    RentalAttorneyFeeClauseReciprocityInput,
    RentalAttorneyFeeClauseReciprocityResult,
};
use traderview_expense::rental_ny_rpl_235f_roommate_law::{
    check as check_rental_ny_rpl_235f_roommate_law,
    NyRpl235FRoommateLawInput as RentalNyRpl235FRoommateLawInput,
    NyRpl235FRoommateLawResult as RentalNyRpl235FRoommateLawResult,
};
use traderview_expense::rental_oakland_measure_ee_just_cause_omc_8_22::{
    check as check_rental_oakland_measure_ee_just_cause_omc_8_22,
    RentalOaklandMeasureEeJustCauseOmc822Input,
    RentalOaklandMeasureEeJustCauseOmc822Result,
};
use traderview_expense::rental_oil_tank_replacement_disclosure::{
    check as check_rental_oil_tank_replacement_disclosure,
    RentalOilTankReplacementDisclosureInput, RentalOilTankReplacementDisclosureResult,
};
use traderview_expense::rental_oregon_sb_608_sb_611_rent_stabilization::{
    check as check_rental_oregon_sb_608_sb_611_rent_stabilization,
    RentalOregonSb608Sb611RentStabilizationInput,
    RentalOregonSb608Sb611RentStabilizationResult,
};
use traderview_expense::rental_nyc_childhood_lead_poisoning_prevention_act::{
    check as check_rental_nyc_childhood_lead_poisoning_prevention_act,
    NycChildhoodLeadPoisoningPreventionActInput as RentalNycChildhoodLeadPoisoningPreventionActInput,
    NycChildhoodLeadPoisoningPreventionActResult as RentalNycChildhoodLeadPoisoningPreventionActResult,
};
use traderview_expense::rental_nyc_local_law_55_ipm_pest_control::{
    check as check_rental_nyc_local_law_55_ipm_pest_control,
    RentalNycLocalLaw55IpmPestControlInput,
    RentalNycLocalLaw55IpmPestControlResult,
};
use traderview_expense::rental_nyc_local_law_18_str_registration::{
    check as check_rental_nyc_local_law_18_str_registration,
    RentalNycLocalLaw18StrRegistrationInput,
    RentalNycLocalLaw18StrRegistrationResult,
};
use traderview_expense::rental_nyc_coop_conversion_eviction_protection::{
    check as check_rental_nyc_coop_conversion_eviction_protection,
    RentalNycCoopConversionEvictionProtectionInput,
    RentalNycCoopConversionEvictionProtectionResult,
};
use traderview_expense::rental_nyc_scrie_drie_rent_freeze::{
    check as check_rental_nyc_scrie_drie_rent_freeze,
    RentalNycScrieDrieRentFreezeInput,
    RentalNycScrieDrieRentFreezeResult,
};
use traderview_expense::rental_nyc_loft_law_article_7c::{
    check as check_rental_nyc_loft_law_article_7c,
    NycLoftLawArticle7CInput as RentalNycLoftLawArticle7CInput,
    NycLoftLawArticle7CResult as RentalNycLoftLawArticle7CResult,
};
use traderview_expense::rental_solar_panel_disclosure::{
    check as check_rental_solar_panel_disclosure,
    RentalSolarPanelDisclosureInput, RentalSolarPanelDisclosureResult,
};
use traderview_expense::rental_storage_unit_lease_disclosure::{
    check as check_rental_storage_unit_lease_disclosure,
    RentalStorageUnitLeaseDisclosureInput, RentalStorageUnitLeaseDisclosureResult,
};
use traderview_expense::rental_balcony_inspection_seismic_safety::{
    check as check_rental_balcony_inspection_seismic_safety,
    RentalBalconyInspectionSeismicSafetyInput, RentalBalconyInspectionSeismicSafetyResult,
};
use traderview_expense::rental_short_term_subletting_airbnb_restriction::{
    check as check_rental_short_term_subletting_airbnb_restriction,
    RentalShortTermSublettingAirbnbRestrictionInput,
    RentalShortTermSublettingAirbnbRestrictionResult,
};
use traderview_expense::rental_grill_propane_bbq_restriction::{
    check as check_rental_grill_propane_bbq_restriction,
    RentalGrillPropaneBbqRestrictionInput, RentalGrillPropaneBbqRestrictionResult,
};
use traderview_expense::rental_radiator_steam_heat_safety::{
    check as check_rental_radiator_steam_heat_safety,
    RentalRadiatorSteamHeatSafetyInput, RentalRadiatorSteamHeatSafetyResult,
};
use traderview_expense::rental_property_tax_pass_through_disclosure::{
    check as check_rental_property_tax_pass_through_disclosure,
    RentalPropertyTaxPassThroughDisclosureInput,
    RentalPropertyTaxPassThroughDisclosureResult,
};
use traderview_expense::rental_marijuana_cultivation_restriction::{
    check as check_rental_marijuana_cultivation_restriction,
    RentalMarijuanaCultivationRestrictionInput,
    RentalMarijuanaCultivationRestrictionResult,
};
use traderview_expense::rental_massachusetts_security_deposit_statute::{
    check as check_rental_massachusetts_security_deposit_statute,
    MassachusettsSecurityDepositInput as RentalMassachusettsSecurityDepositStatuteInput,
    MassachusettsSecurityDepositResult as RentalMassachusettsSecurityDepositStatuteResult,
};
use traderview_expense::rental_massachusetts_homes_act_eviction_sealing::{
    check as check_rental_massachusetts_homes_act_eviction_sealing,
    RentalMassachusettsHomesActEvictionSealingInput,
    RentalMassachusettsHomesActEvictionSealingResult,
};
use traderview_expense::rental_attached_garage_carbon_monoxide_disclosure::{
    check as check_rental_attached_garage_carbon_monoxide_disclosure,
    RentalAttachedGarageCarbonMonoxideDisclosureInput,
    RentalAttachedGarageCarbonMonoxideDisclosureResult,
};
use traderview_expense::rental_pet_breed_restriction_disclosure::{
    check as check_rental_pet_breed_restriction_disclosure,
    RentalPetBreedRestrictionDisclosureInput,
    RentalPetBreedRestrictionDisclosureResult,
};
use traderview_expense::rental_emergency_action_plan_high_rise::{
    check as check_rental_emergency_action_plan_high_rise,
    RentalEmergencyActionPlanHighRiseInput,
    RentalEmergencyActionPlanHighRiseResult,
};
use traderview_expense::rental_eviction_record_sealing_screening::{
    check as check_rental_eviction_record_sealing_screening,
    EvictionRecordSealingInput as RentalEvictionRecordSealingScreeningInput,
    EvictionRecordSealingResult as RentalEvictionRecordSealingScreeningResult,
};
use traderview_expense::rental_illegal_lockout_self_help_eviction::{
    check as check_rental_illegal_lockout_self_help_eviction,
    RentalIllegalLockoutSelfHelpEvictionInput,
    RentalIllegalLockoutSelfHelpEvictionResult,
};
use traderview_expense::rental_retaliation_prohibition::{
    check as check_rental_retaliation_prohibition,
    RentalRetaliationProhibitionInput,
    RentalRetaliationProhibitionResult,
};
use traderview_expense::rental_landlord_notice_to_enter::{
    check as check_rental_landlord_notice_to_enter,
    RentalLandlordNoticeToEnterInput,
    RentalLandlordNoticeToEnterResult,
};
use traderview_expense::rental_security_deposit_return_notice::{
    check as check_rental_security_deposit_return_notice,
    RentalSecurityDepositReturnNoticeInput,
    RentalSecurityDepositReturnNoticeResult,
};
use traderview_expense::rental_late_fee_cap::{
    check as check_rental_late_fee_cap,
    RentalLateFeeCapInput,
    RentalLateFeeCapResult,
};
use traderview_expense::rental_local_law_87_energy_audit_retro_commissioning::{
    check as check_rental_local_law_87_energy_audit_retro_commissioning,
    LocalLaw87EnergyAuditRetroCommissioningInput as RentalLocalLaw87EnergyAuditRetroCommissioningInput,
    LocalLaw87EnergyAuditRetroCommissioningResult as RentalLocalLaw87EnergyAuditRetroCommissioningResult,
};
use traderview_expense::rental_local_law_88_lighting_upgrades_sub_metering::{
    check as check_rental_local_law_88_lighting_upgrades_sub_metering,
    LocalLaw88LightingUpgradesSubMeteringInput as RentalLocalLaw88LightingUpgradesSubMeteringInput,
    LocalLaw88LightingUpgradesSubMeteringResult as RentalLocalLaw88LightingUpgradesSubMeteringResult,
};
use traderview_expense::rental_tenant_criminal_background_screening::{
    check as check_rental_tenant_criminal_background_screening,
    RentalTenantCriminalBackgroundScreeningInput,
    RentalTenantCriminalBackgroundScreeningResult,
};
use traderview_expense::rental_source_of_income_discrimination::{
    check as check_rental_source_of_income_discrimination,
    RentalSourceOfIncomeDiscriminationInput,
    RentalSourceOfIncomeDiscriminationResult,
};
use traderview_expense::rental_tenant_bill_of_rights_handout::{
    check as check_rental_tenant_bill_of_rights_handout,
    RentalTenantBillOfRightsHandoutInput,
    RentalTenantBillOfRightsHandoutResult,
};
use traderview_expense::rental_tenant_abandoned_personal_property::{
    check as check_rental_tenant_abandoned_personal_property,
    RentalTenantAbandonedPersonalPropertyInput,
    RentalTenantAbandonedPersonalPropertyResult,
};
use traderview_expense::rental_texas_hb_2127_state_preemption::{
    check as check_rental_texas_hb_2127_state_preemption,
    RentalTexasHb2127StatePreemptionInput,
    RentalTexasHb2127StatePreemptionResult,
};
use traderview_expense::rental_minneapolis_renter_protections_ordinance_2020::{
    check as check_rental_minneapolis_renter_protections_ordinance_2020,
    RentalMinneapolisRenterProtectionsOrdinance2020Input,
    RentalMinneapolisRenterProtectionsOrdinance2020Result,
};
use traderview_expense::rental_mold_disclosure_remediation::{
    check as check_rental_mold_disclosure_remediation,
    RentalMoldDisclosureRemediationInput,
    RentalMoldDisclosureRemediationResult,
};
use traderview_expense::rental_multilingual_lease_translation::{
    check as check_rental_multilingual_lease_translation,
    MultilingualLeaseTranslationInput as RentalMultilingualLeaseTranslationInput,
    MultilingualLeaseTranslationResult as RentalMultilingualLeaseTranslationResult,
};
use traderview_expense::rental_fair_housing_reasonable_accommodation::{
    check as check_rental_fair_housing_reasonable_accommodation,
    RentalFairHousingReasonableAccommodationInput,
    RentalFairHousingReasonableAccommodationResult,
};
use traderview_expense::rental_facade_inspection_fisp_local_law_11::{
    check as check_rental_facade_inspection_fisp_local_law_11,
    FacadeInspectionFispInput as RentalFacadeInspectionFispLocalLaw11Input,
    FacadeInspectionFispResult as RentalFacadeInspectionFispLocalLaw11Result,
};
use traderview_expense::rental_boiler_inspection_compliance::{
    check as check_rental_boiler_inspection_compliance,
    RentalBoilerInspectionComplianceInput,
    RentalBoilerInspectionComplianceResult,
};
use traderview_expense::rental_tree_removal_dangerous_tree_disclosure::{
    check as check_rental_tree_removal_dangerous_tree_disclosure,
    RentalTreeRemovalDangerousTreeDisclosureInput,
    RentalTreeRemovalDangerousTreeDisclosureResult,
};
use traderview_expense::rental_tenant_rent_escrow_habitability_dispute::{
    check as check_rental_tenant_rent_escrow_habitability_dispute,
    RentalTenantRentEscrowHabitabilityDisputeInput,
    RentalTenantRentEscrowHabitabilityDisputeResult,
};
use traderview_expense::rental_tenant_right_to_counsel_eviction::{
    check as check_rental_tenant_right_to_counsel_eviction,
    RentalTenantRightToCounselEvictionInput,
    RentalTenantRightToCounselEvictionResult,
};
use traderview_expense::rental_ada_accessible_parking_compliance::{
    check as check_rental_ada_accessible_parking_compliance,
    RentalAdaAccessibleParkingComplianceInput,
    RentalAdaAccessibleParkingComplianceResult,
};
use traderview_expense::rental_smoke_free_cannabis_restriction::{
    check as check_rental_smoke_free_cannabis_restriction,
    RentalSmokeFreeCannabisRestrictionInput,
    RentalSmokeFreeCannabisRestrictionResult,
};
use traderview_expense::rental_rent_to_own_lease_purchase_disclosures::{
    check as check_rental_rent_to_own_lease_purchase_disclosures,
    RentalRentToOwnLeasePurchaseDisclosuresInput,
    RentalRentToOwnLeasePurchaseDisclosuresResult,
};
use traderview_expense::rental_rent_increase_notice_requirement::{
    check as check_rental_rent_increase_notice_requirement,
    RentalRentIncreaseNoticeRequirementInput,
    RentalRentIncreaseNoticeRequirementResult,
};
use traderview_expense::rental_rent_control_stabilization::{
    check as check_rental_rent_control_stabilization,
    RentalRentControlStabilizationInput,
    RentalRentControlStabilizationResult,
};
use traderview_expense::rental_tenant_relocation_assistance::{
    check as check_rental_tenant_relocation_assistance,
    RentalTenantRelocationAssistanceInput,
    RentalTenantRelocationAssistanceResult,
};
use traderview_expense::rental_tenant_estoppel_certificate::{
    check as check_rental_tenant_estoppel_certificate,
    RentalTenantEstoppelCertificateInput,
    RentalTenantEstoppelCertificateResult,
};
use traderview_expense::rental_tenant_data_privacy_compliance::{
    check as check_rental_tenant_data_privacy_compliance,
    RentalTenantDataPrivacyComplianceInput,
    RentalTenantDataPrivacyComplianceResult,
};
use traderview_expense::rental_ev_charging_accommodation::{
    check as check_rental_ev_charging_accommodation,
    RentalEvChargingAccommodationInput,
    RentalEvChargingAccommodationResult,
};
use traderview_expense::rental_waste_recycling_collection_mandate::{
    check as check_rental_waste_recycling_collection_mandate,
    RentalWasteRecyclingCollectionMandateInput,
    RentalWasteRecyclingCollectionMandateResult,
};
use traderview_expense::rental_washington_hb_1217_rent_stabilization::{
    check as check_rental_washington_hb_1217_rent_stabilization,
    RentalWashingtonHb1217RentStabilizationInput,
    RentalWashingtonHb1217RentStabilizationResult,
};
use traderview_expense::rental_domestic_violence_lock_change_lease_termination::{
    check as check_rental_domestic_violence_lock_change_lease_termination,
    RentalDomesticViolenceLockChangeLeaseTerminationInput,
    RentalDomesticViolenceLockChangeLeaseTerminationResult,
};
use traderview_expense::rental_drone_overflight_surveillance_privacy::{
    check as check_rental_drone_overflight_surveillance_privacy,
    RentalDroneOverflightSurveillancePrivacyInput,
    RentalDroneOverflightSurveillancePrivacyResult,
};
use traderview_expense::rental_dog_bite_liability::{
    check as check_rental_dog_bite_liability,
    RentalDogBiteLiabilityInput,
    RentalDogBiteLiabilityResult,
};
use traderview_expense::rental_pellet_stove_disclosure::{
    check as check_rental_pellet_stove_disclosure,
    RentalPelletStoveDisclosureInput, RentalPelletStoveDisclosureResult,
};
use traderview_expense::rental_in_unit_laundry_appliance_provision::{
    check as check_rental_in_unit_laundry_appliance_provision,
    RentalInUnitLaundryApplianceProvisionInput, RentalInUnitLaundryApplianceProvisionResult,
};
use traderview_expense::rental_positive_rent_payment_credit_reporting::{
    check as check_rental_positive_rent_payment_credit_reporting,
    RentalPositiveRentPaymentCreditReportingInput,
    RentalPositiveRentPaymentCreditReportingResult,
};
use traderview_expense::rental_post_construction_lead_dust_clearance::{
    check as check_rental_post_construction_lead_dust_clearance,
    RentalPostConstructionLeadDustClearanceInput, RentalPostConstructionLeadDustClearanceResult,
};
use traderview_expense::tenant_voting_address_protection::{
    check as check_tenant_voting_address_protection,
    TenantVotingAddressProtectionInput, TenantVotingAddressProtectionResult,
};
use traderview_expense::tenant_kitchen_appliance_replacement::{
    check as check_tenant_kitchen_appliance_replacement,
    TenantKitchenApplianceReplacementInput, TenantKitchenApplianceReplacementResult,
};
use traderview_expense::rental_hoa_disclosure_at_lease::{
    check as check_rental_hoa_disclosure_at_lease,
    RentalHoaDisclosureAtLeaseInput, RentalHoaDisclosureAtLeaseResult,
};
use traderview_expense::rental_property_registration::{
    check as check_rental_property_registration, RentalPropertyRegistrationInput,
    RentalPropertyRegistrationResult,
};
use traderview_expense::rental_renters_insurance_requirement::{
    check as check_rental_renters_insurance_requirement,
    RentalRentersInsuranceRequirementInput,
    RentalRentersInsuranceRequirementResult,
};
use traderview_expense::rental_radon_mitigation_disclosure::{
    check as check_rental_radon_mitigation_disclosure,
    RentalRadonMitigationDisclosureInput, RentalRadonMitigationDisclosureResult,
};
use traderview_expense::residential_lease_arbitration_clause::{
    check as check_residential_arbitration, ArbitrationClauseInput, ArbitrationClauseResult,
};
use traderview_expense::lease_waiver_enforceability::{
    check as check_lease_waiver_enforceability,
    CheckResult as LeaseWaiverEnforceabilityResult,
    Input as LeaseWaiverEnforceabilityInput,
};
use traderview_expense::landlord_repair_response_timeframe::{
    check as check_landlord_repair_response_timeframe, LandlordRepairResponseInput,
    LandlordRepairResponseResult,
};
use traderview_expense::landlord_retaliation_damages::{
    check as check_landlord_retaliation_damages,
    CheckResult as LandlordRetaliationDamagesResult,
    Input as LandlordRetaliationDamagesInput,
};
use traderview_expense::landlord_security_device_obligations::{
    check as check_landlord_security_device_obligations, LandlordSecurityDeviceInput,
    LandlordSecurityDeviceResult,
};
use traderview_expense::landlord_self_help_eviction_prohibition::{
    check as check_landlord_self_help_eviction_prohibition,
    SelfHelpEvictionInput as LandlordSelfHelpEvictionInput,
    SelfHelpEvictionResult as LandlordSelfHelpEvictionResult,
};
use traderview_expense::landlord_tenant_recording_consent::{
    check as check_landlord_tenant_recording_consent, RecordingConsentInput,
    RecordingConsentResult,
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
use traderview_expense::tenant_noise_nuisance_enforcement::{
    check as check_tenant_noise_nuisance_enforcement,
    TenantNoiseNuisanceEnforcementInput, TenantNoiseNuisanceEnforcementResult,
};
use traderview_expense::tenant_organizing::{
    check as check_tenant_organizing, TenantOrganizingInput, TenantOrganizingResult,
};
use traderview_expense::tenant_positive_rent_reporting::{
    check as check_tenant_positive_rent_reporting,
    TenantPositiveRentReportingInput, TenantPositiveRentReportingResult,
};
use traderview_expense::tenant_rights_statement_disclosure::{
    check as check_tenant_rights_statement_disclosure, TenantRightsStatementInput,
    TenantRightsStatementResult,
};
use traderview_expense::tenant_smart_lock_biometric_consent::{
    check as check_tenant_smart_lock_biometric_consent, TenantSmartLockBiometricInput,
    TenantSmartLockBiometricResult,
};
use traderview_expense::tenant_smart_thermostat_install_right::{
    check as check_tenant_smart_thermostat_install_right,
    TenantSmartThermostatInstallRightInput, TenantSmartThermostatInstallRightResult,
};
use traderview_expense::tenant_utility_account_designation::{
    check as check_tenant_utility_account_designation, TenantUtilityAccountInput,
    TenantUtilityAccountResult,
};
use traderview_expense::tenant_window_air_conditioner_install_right::{
    check as check_tenant_window_air_conditioner_install_right,
    TenantWindowAirConditionerInstallRightInput,
    TenantWindowAirConditionerInstallRightResult,
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
use traderview_expense::swimming_pool_safety::{
    check as check_swimming_pool_safety, SwimmingPoolSafetyInput, SwimmingPoolSafetyResult,
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
use traderview_expense::tenant_accessible_parking::{
    check as check_tenant_accessible_parking, TenantAccessibleParkingInput,
    TenantAccessibleParkingResult,
};
use traderview_expense::tenant_assistance_animal_accommodation::{
    check as check_tenant_assistance_animal_accommodation,
    AssistanceAnimalAccommodationInput as TenantAssistanceAnimalAccommodationInput,
    AssistanceAnimalAccommodationResult as TenantAssistanceAnimalAccommodationResult,
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
        .route("/broker-fee-allocation", axum::routing::post(broker_fee_allocation_route))
        .route("/lockout-penalty-check", axum::routing::post(lockout_penalty_check_route))
        .route("/dv-survivor-lock-change", axum::routing::post(dv_survivor_lock_change_route))
        .route("/dv-termination-check", axum::routing::post(dv_termination_check_route))
        .route("/just-cause-check", axum::routing::post(just_cause_check_route))
        .route("/just-cause-termination-notice-content", axum::routing::post(just_cause_termination_notice_content_route))
        .route("/soi-protection-check", axum::routing::post(soi_protection_check_route))
        .route("/detector-check", axum::routing::post(detector_check_route))
        .route("/lead-disclosure-check", axum::routing::post(lead_disclosure_check_route))
        .route("/lead-in-drinking-water-disclosure", axum::routing::post(lead_in_drinking_water_disclosure_route))
        .route("/lead-renovation-repair-painting", axum::routing::post(lead_renovation_repair_painting_route))
        .route("/foreclosure-tenant-check", axum::routing::post(foreclosure_tenant_check_route))
        .route("/heat-requirements-check", axum::routing::post(heat_requirements_check_route))
        .route("/bedbug-disclosure-check", axum::routing::post(bedbug_disclosure_check_route))
        .route("/balcony-inspection", axum::routing::post(balcony_inspection_route))
        .route("/mold-disclosure-check", axum::routing::post(mold_disclosure_check_route))
        .route("/radon-disclosure-check", axum::routing::post(radon_disclosure_check_route))
        .route("/sublet-consent-check", axum::routing::post(sublet_consent_check_route))
        .route("/swimming-pool-safety", axum::routing::post(swimming_pool_safety_route))
        .route("/str-regulation-check", axum::routing::post(str_regulation_check_route))
        .route("/squatter-unauthorized-occupant-removal", axum::routing::post(squatter_removal_route))
        .route("/pet-fees-check", axum::routing::post(pet_fees_check_route))
        .route("/non-refundable-cleaning-fees", axum::routing::post(non_refundable_cleaning_fees_route))
        .route("/eviction-sealing-check", axum::routing::post(eviction_sealing_check_route))
        .route("/termination-notice-check", axum::routing::post(termination_notice_check_route))
        .route("/lease-termination-catastrophic-damage", axum::routing::post(lease_termination_catastrophic_damage_route))
        .route("/occupancy-check", axum::routing::post(occupancy_check_route))
        .route("/move-in-inspection-check", axum::routing::post(move_in_inspection_check_route))
        .route("/move-in-fee-cap", axum::routing::post(move_in_fee_cap_route))
        .route("/mandatory-renters-insurance-provider-choice", axum::routing::post(mandatory_renters_insurance_provider_choice_route))
        .route("/renters-insurance-check", axum::routing::post(renters_insurance_check_route))
        .route("/utility-shutoff-check", axum::routing::post(utility_shutoff_check_route))
        .route("/vehicle-towing-from-rental-property", axum::routing::post(vehicle_towing_from_rental_property_route))
        .route("/water-heater-earthquake-strap", axum::routing::post(water_heater_earthquake_strap_route))
        .route("/adverse-action-check", axum::routing::post(adverse_action_check_route))
        .route("/adverse-possession-claim", axum::routing::post(adverse_possession_claim_route))
        .route("/topa-check", axum::routing::post(tenant_topa_check_route))
        .route("/auto-renewal-check", axum::routing::post(lease_auto_renewal_check_route))
        .route("/lease-translation-check", axum::routing::post(lease_translation_check_route))
        .route("/rent-receipt-check", axum::routing::post(rent_receipt_check_route))
        .route("/rent-stabilized-mci-iai-passthrough", axum::routing::post(rent_stabilized_mci_iai_passthrough_route))
        .route("/repair-deduct-check", axum::routing::post(repair_and_deduct_check_route))
        .route("/cosigner-check", axum::routing::post(cosigner_rules_check_route))
        .route("/mobile-home-park-check", axum::routing::post(mobile_home_park_check_route))
        .route("/submetering-check", axum::routing::post(submetering_check_route))
        .route("/smoke-free-check", axum::routing::post(smoke_free_check_route))
        .route("/tenant-privacy-check", axum::routing::post(tenant_privacy_check_route))
        .route("/tenant-domestic-violence-lease-termination", axum::routing::post(tenant_domestic_violence_lease_termination_route))
        .route("/tenant-ev-charging-installation-right", axum::routing::post(tenant_ev_charging_installation_right_route))
        .route("/tenant-fire-safety-plan-disclosure", axum::routing::post(tenant_fire_safety_plan_disclosure_route))
        .route("/drug-eviction-check", axum::routing::post(drug_eviction_check_route))
        .route("/quiet-enjoyment-check", axum::routing::post(quiet_enjoyment_check_route))
        .route("/flood-disclosure-check", axum::routing::post(flood_disclosure_check_route))
        .route("/owner-identification-check", axum::routing::post(owner_identification_check_route))
        .route("/tenant-death-termination-check", axum::routing::post(tenant_death_termination_check_route))
        .route("/late-payment-grace-period-check", axum::routing::post(late_payment_grace_period_check_route))
        .route("/owner-move-in-eviction-check", axum::routing::post(owner_move_in_eviction_check_route))
        .route("/lease-copy-delivery-check", axum::routing::post(lease_copy_delivery_check_route))
        .route("/tenant-noise-nuisance-enforcement", axum::routing::post(tenant_noise_nuisance_enforcement_route))
        .route("/tenant-organizing-check", axum::routing::post(tenant_organizing_check_route))
        .route("/tenant-positive-rent-reporting", axum::routing::post(tenant_positive_rent_reporting_route))
        .route("/tenant-rent-escrow-withholding", axum::routing::post(tenant_rent_escrow_withholding_route))
        .route("/tenant-rent-judgment-wage-garnishment", axum::routing::post(tenant_rent_judgment_wage_garnishment_route))
        .route("/tenant-rent-receipt-requirement", axum::routing::post(tenant_rent_receipt_requirement_route))
        .route("/tenant-relocation-assistance", axum::routing::post(tenant_relocation_assistance_route))
        .route("/tenant-rights-statement-disclosure", axum::routing::post(tenant_rights_statement_disclosure_route))
        .route("/tenant-smart-lock-biometric-consent", axum::routing::post(tenant_smart_lock_biometric_consent_route))
        .route("/tenant-smart-thermostat-install-right", axum::routing::post(tenant_smart_thermostat_install_right_route))
        .route("/tenant-voting-address-protection", axum::routing::post(tenant_voting_address_protection_route))
        .route("/tenant-kitchen-appliance-replacement", axum::routing::post(tenant_kitchen_appliance_replacement_route))
        .route("/tenant-utility-account-designation", axum::routing::post(tenant_utility_account_designation_route))
        .route("/tenant-window-air-conditioner-install-right", axum::routing::post(tenant_window_air_conditioner_install_right_route))
        .route("/fair-chance-housing", axum::routing::post(fair_chance_housing_route))
        .route("/family-childcare-home-right", axum::routing::post(family_childcare_home_right_route))
        .route("/source-of-income-discrimination", axum::routing::post(source_of_income_discrimination_route))
        .route("/fha-design-construction", axum::routing::post(fha_design_construction_route))
        .route("/meth-contamination-disclosure", axum::routing::post(meth_contamination_disclosure_route))
        .route("/death-in-unit-disclosure", axum::routing::post(death_in_unit_disclosure_route))
        .route("/dog-breed-restriction-ban", axum::routing::post(dog_breed_restriction_ban_route))
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
        .route("/tenant-clothesline-drying-right", axum::routing::post(tenant_clothesline_drying_right_route))
        .route("/snow-removal-responsibility", axum::routing::post(snow_removal_responsibility_route))
        .route("/soft-story-seismic-retrofit", axum::routing::post(soft_story_seismic_retrofit_route))
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
        .route("/lease-early-termination-fee-cap", axum::routing::post(lease_early_termination_fee_cap_route))
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
        .route("/short-term-rental-conversion", axum::routing::post(short_term_rental_conversion_route))
        .route("/mid-tenancy-ownership-change", axum::routing::post(mid_tenancy_ownership_change_route))
        .route("/mid-tenancy-security-deposit-increase", axum::routing::post(mid_tenancy_security_deposit_increase_route))
        .route("/mid-tenancy-term-modification", axum::routing::post(mid_tenancy_term_modification_route))
        .route("/mid-tenancy-temporary-relocation", axum::routing::post(mid_tenancy_temporary_relocation_route))
        .route("/tenant-solar-installation", axum::routing::post(tenant_solar_installation_route))
        .route("/flag-display-right", axum::routing::post(flag_display_right_route))
        .route("/written-lease-requirement", axum::routing::post(written_lease_requirement_route))
        .route("/holdover-tenant-damages", axum::routing::post(holdover_tenant_damages_route))
        .route("/lease-assignment-consent", axum::routing::post(lease_assignment_consent_route))
        .route("/lease-cure-period", axum::routing::post(lease_cure_period_route))
        .route("/portable-tenant-screening-report", axum::routing::post(portable_tenant_screening_report_route))
        .route("/hoa-fee-tenant-enforcement", axum::routing::post(hoa_fee_tenant_enforcement_route))
        .route("/hoa-rental-restriction", axum::routing::post(hoa_rental_restriction_route))
        .route("/rent-acceleration-enforceability", axum::routing::post(rent_acceleration_enforceability_route))
        .route("/tenant-in-foreclosure-protection", axum::routing::post(tenant_in_foreclosure_protection_route))
        .route("/tenant-in-unit-appliance-repair-responsibility", axum::routing::post(tenant_in_unit_appliance_repair_responsibility_route))
        .route("/tenant-late-fee-cap", axum::routing::post(tenant_late_fee_cap_route))
        .route("/tenant-lease-guarantor-disclosure", axum::routing::post(tenant_lease_guarantor_disclosure_route))
        .route("/tenant-estoppel-certificate", axum::routing::post(tenant_estoppel_certificate_route))
        .route("/landlord-property-sale-notice", axum::routing::post(landlord_property_sale_notice_route))
        .route("/lease-renewal-offer-timing", axum::routing::post(lease_renewal_offer_timing_route))
        .route("/rent-concession-disclosure", axum::routing::post(rent_concession_disclosure_route))
        .route("/rent-abatement-construction-nuisance", axum::routing::post(rent_abatement_construction_nuisance_route))
        .route("/landlord-master-key-retention", axum::routing::post(landlord_master_key_retention_route))
        .route("/tenant-holdover-security-deposit-setoff", axum::routing::post(tenant_holdover_security_deposit_setoff_route))
        .route("/rental-video-surveillance-retention", axum::routing::post(rental_video_surveillance_retention_route))
        .route("/landlord-foreclosure-status-disclosure", axum::routing::post(landlord_foreclosure_status_disclosure_route))
        .route("/commercial-lease-personal-guaranty-enforceability", axum::routing::post(commercial_lease_personal_guaranty_enforceability_route))
        .route("/commercial-lease-cam-charge-disclosure", axum::routing::post(commercial_lease_cam_charge_disclosure_route))
        .route("/landlord-pest-extermination-timeline", axum::routing::post(landlord_pest_extermination_timeline_route))
        .route("/landlord-water-heat-emergency-response", axum::routing::post(landlord_water_heat_emergency_response_route))
        .route("/tenant-emotional-distress-damages", axum::routing::post(tenant_emotional_distress_damages_route))
        .route("/landlord-negative-credit-reporting", axum::routing::post(landlord_negative_credit_reporting_route))
        .route("/security-deposit-bank-disclosure", axum::routing::post(security_deposit_bank_disclosure_route))
        .route("/landlord-annual-rent-statement", axum::routing::post(landlord_annual_rent_statement_route))
        .route("/landlord-emergency-entry-notice", axum::routing::post(landlord_emergency_entry_notice_route))
        .route("/landlord-mid-tenancy-rekeying", axum::routing::post(landlord_mid_tenancy_rekeying_route))
        .route("/landlord-harassment", axum::routing::post(landlord_harassment_route))
        .route("/landlord-possession-delivery", axum::routing::post(landlord_possession_delivery_route))
        .route("/landlord-post-eviction-tenant-property-storage-disposal", axum::routing::post(landlord_post_eviction_tenant_property_storage_disposal_route))
        .route("/lease-waiver-enforceability", axum::routing::post(lease_waiver_enforceability_route))
        .route("/rental-application-denial-disclosure", axum::routing::post(rental_application_denial_disclosure_route))
        .route("/rental-asbestos-disclosure", axum::routing::post(rental_asbestos_disclosure_route))
        .route("/rental-basement-water-intrusion-disclosure", axum::routing::post(rental_basement_water_intrusion_disclosure_route))
        .route("/rental-bed-bug-disclosure", axum::routing::post(rental_bed_bug_disclosure_route))
        .route("/rental-bedroom-egress-window", axum::routing::post(rental_bedroom_egress_window_route))
        .route("/rental-berkeley-rent-stabilization-ordinance-bmc-chapter-13-76", axum::routing::post(rental_berkeley_rent_stabilization_ordinance_bmc_chapter_13_76_route))
        .route("/rental-carbon-monoxide-detector", axum::routing::post(rental_carbon_monoxide_detector_route))
        .route("/rental-california-ab-12-security-deposit-cap", axum::routing::post(rental_california_ab_12_security_deposit_cap_route))
        .route("/rental-california-ab-2347-unlawful-detainer-response", axum::routing::post(rental_california_ab_2347_unlawful_detainer_response_route))
        .route("/rental-california-sb-567-no-fault-eviction-amendments", axum::routing::post(rental_california_sb_567_no_fault_eviction_amendments_route))
        .route("/rental-chimney-fireplace-inspection-disclosure", axum::routing::post(rental_chimney_fireplace_inspection_disclosure_route))
        .route("/rental-climate-mobilization-act-ll97-emissions", axum::routing::post(rental_climate_mobilization_act_ll97_emissions_route))
        .route("/rental-colorado-hb-24-1098-just-cause-eviction", axum::routing::post(rental_colorado_hb_24_1098_just_cause_eviction_route))
        .route("/rental-connecticut-fair-rent-commission", axum::routing::post(rental_connecticut_fair_rent_commission_route))
        .route("/rental-cook-county-rtlo", axum::routing::post(rental_cook_county_rtlo_route))
        .route("/rental-cooling-tower-inspection-local-law-77", axum::routing::post(rental_cooling_tower_inspection_local_law_77_route))
        .route("/rental-elevator-safety-inspection", axum::routing::post(rental_elevator_safety_inspection_route))
        .route("/rental-fire-extinguisher-requirement", axum::routing::post(rental_fire_extinguisher_requirement_route))
        .route("/rental-florida-hb-1417-state-preemption", axum::routing::post(rental_florida_hb_1417_state_preemption_route))
        .route("/rental-flood-hazard-disclosure", axum::routing::post(rental_flood_hazard_disclosure_route))
        .route("/rental-foreclosure-tenant-protection-ptfa", axum::routing::post(rental_foreclosure_tenant_protection_ptfa_route))
        .route("/rental-broadband-mte-rules", axum::routing::post(rental_broadband_mte_rules_route))
        .route("/rental-energy-benchmarking", axum::routing::post(rental_energy_benchmarking_route))
        .route("/rental-garage-door-safety-compliance", axum::routing::post(rental_garage_door_safety_compliance_route))
        .route("/rental-gas-appliance-ban", axum::routing::post(rental_gas_appliance_ban_route))
        .route("/rental-gas-piping-inspection-local-law-152", axum::routing::post(rental_gas_piping_inspection_local_law_152_route))
        .route("/rental-hardwired-smoke-alarm-responsibility", axum::routing::post(rental_hardwired_smoke_alarm_responsibility_route))
        .route("/rental-hawaii-residential-landlord-tenant-code-hrs-521", axum::routing::post(rental_hawaii_residential_landlord_tenant_code_hrs_521_route))
        .route("/rental-heat-minimum-temperature-season", axum::routing::post(rental_heat_minimum_temperature_season_route))
        .route("/rental-hot-water-temperature", axum::routing::post(rental_hot_water_temperature_route))
        .route("/rental-housing-for-older-persons-act-hopa-1995", axum::routing::post(rental_housing_for_older_persons_act_hopa_1995_route))
        .route("/rental-hud-hotma-income-asset-compliance", axum::routing::post(rental_hud_hotma_income_asset_compliance_route))
        .route("/rental-hud-section-504-rehabilitation-act-24-cfr-part-8", axum::routing::post(rental_hud_section_504_rehabilitation_act_24_cfr_part_8_route))
        .route("/rental-in-unit-laundry-appliance-provision", axum::routing::post(rental_in_unit_laundry_appliance_provision_route))
        .route("/rental-junk-fee-transparency", axum::routing::post(rental_junk_fee_transparency_route))
        .route("/rental-just-cause-eviction", axum::routing::post(rental_just_cause_eviction_route))
        .route("/rental-hoa-disclosure-at-lease", axum::routing::post(rental_hoa_disclosure_at_lease_route))
        .route("/rental-lead-paint-disclosure", axum::routing::post(rental_lead_paint_disclosure_route))
        .route("/rental-local-law-87-energy-audit-retro-commissioning", axum::routing::post(rental_local_law_87_energy_audit_retro_commissioning_route))
        .route("/rental-local-law-88-lighting-upgrades-sub-metering", axum::routing::post(rental_local_law_88_lighting_upgrades_sub_metering_route))
        .route("/rental-lead-pipe-disclosure", axum::routing::post(rental_lead_pipe_disclosure_route))
        .route("/rental-natural-gas-leak-response", axum::routing::post(rental_natural_gas_leak_response_route))
        .route("/rental-new-jersey-anti-eviction-act", axum::routing::post(rental_new_jersey_anti_eviction_act_route))
        .route("/rental-ny-rent-receipt-late-notice-requirements", axum::routing::post(rental_ny_rent_receipt_late_notice_requirements_route))
        .route("/rental-ny-rpl-235f-roommate-law", axum::routing::post(rental_ny_rpl_235f_roommate_law_route))
        .route("/rental-attorney-fee-clause-reciprocity", axum::routing::post(rental_attorney_fee_clause_reciprocity_route))
        .route("/rental-nyc-childhood-lead-poisoning-prevention-act", axum::routing::post(rental_nyc_childhood_lead_poisoning_prevention_act_route))
        .route("/rental-nyc-loft-law-article-7c", axum::routing::post(rental_nyc_loft_law_article_7c_route))
        .route("/rental-nyc-scrie-drie-rent-freeze", axum::routing::post(rental_nyc_scrie_drie_rent_freeze_route))
        .route("/rental-nyc-local-law-55-ipm-pest-control", axum::routing::post(rental_nyc_local_law_55_ipm_pest_control_route))
        .route("/rental-nyc-local-law-18-str-registration", axum::routing::post(rental_nyc_local_law_18_str_registration_route))
        .route("/rental-nyc-coop-conversion-eviction-protection", axum::routing::post(rental_nyc_coop_conversion_eviction_protection_route))
        .route("/rental-oakland-measure-ee-just-cause-omc-8-22", axum::routing::post(rental_oakland_measure_ee_just_cause_omc_8_22_route))
        .route("/rental-oil-tank-replacement-disclosure", axum::routing::post(rental_oil_tank_replacement_disclosure_route))
        .route("/rental-oregon-sb-608-sb-611-rent-stabilization", axum::routing::post(rental_oregon_sb_608_sb_611_rent_stabilization_route))
        .route("/rental-organic-waste-collection-disclosure", axum::routing::post(rental_organic_waste_collection_disclosure_route))
        .route("/rental-pellet-stove-disclosure", axum::routing::post(rental_pellet_stove_disclosure_route))
        .route("/rental-pesticide-application-notification", axum::routing::post(rental_pesticide_application_notification_route))
        .route("/rental-pet-deposit-separate-security", axum::routing::post(rental_pet_deposit_separate_security_route))
        .route("/rental-post-construction-lead-dust-clearance", axum::routing::post(rental_post_construction_lead_dust_clearance_route))
        .route("/rental-positive-rent-payment-credit-reporting", axum::routing::post(rental_positive_rent_payment_credit_reporting_route))
        .route("/rental-propane-tank-lease-disclosure", axum::routing::post(rental_propane_tank_lease_disclosure_route))
        .route("/rental-pre-foreclosure-tenant-notification", axum::routing::post(rental_pre_foreclosure_tenant_notification_route))
        .route("/rental-property-registration", axum::routing::post(rental_property_registration_route))
        .route("/rental-radon-mitigation-disclosure", axum::routing::post(rental_radon_mitigation_disclosure_route))
        .route("/rental-renters-insurance-requirement", axum::routing::post(rental_renters_insurance_requirement_route))
        .route("/rental-san-francisco-rent-ordinance-chapter-37", axum::routing::post(rental_san_francisco_rent_ordinance_chapter_37_route))
        .route("/rental-satellite-dish-installation-right", axum::routing::post(rental_satellite_dish_installation_right_route))
        .route("/rental-seattle-smc-22-206-160-just-cause-eviction", axum::routing::post(rental_seattle_smc_22_206_160_just_cause_eviction_route))
        .route("/rental-security-deposit-interest", axum::routing::post(rental_security_deposit_interest_route))
        .route("/rental-septic-system-disclosure", axum::routing::post(rental_septic_system_disclosure_route))
        .route("/rental-sewer-lateral-responsibility", axum::routing::post(rental_sewer_lateral_responsibility_route))
        .route("/rental-sex-offender-registry-notice", axum::routing::post(rental_sex_offender_registry_notice_route))
        .route("/rental-sinkhole-disclosure", axum::routing::post(rental_sinkhole_disclosure_route))
        .route("/rental-smoke-free-housing-disclosure", axum::routing::post(rental_smoke_free_housing_disclosure_route))
        .route("/rental-soft-story-seismic-retrofit", axum::routing::post(rental_soft_story_seismic_retrofit_route))
        .route("/rental-solar-panel-disclosure", axum::routing::post(rental_solar_panel_disclosure_route))
        .route("/rental-storage-unit-lease-disclosure", axum::routing::post(rental_storage_unit_lease_disclosure_route))
        .route("/rental-balcony-inspection-seismic-safety", axum::routing::post(rental_balcony_inspection_seismic_safety_route))
        .route("/rental-short-term-subletting-airbnb-restriction", axum::routing::post(rental_short_term_subletting_airbnb_restriction_route))
        .route("/rental-grill-propane-bbq-restriction", axum::routing::post(rental_grill_propane_bbq_restriction_route))
        .route("/rental-radiator-steam-heat-safety", axum::routing::post(rental_radiator_steam_heat_safety_route))
        .route("/rental-property-tax-pass-through-disclosure", axum::routing::post(rental_property_tax_pass_through_disclosure_route))
        .route("/rental-marijuana-cultivation-restriction", axum::routing::post(rental_marijuana_cultivation_restriction_route))
        .route("/rental-massachusetts-security-deposit-statute", axum::routing::post(rental_massachusetts_security_deposit_statute_route))
        .route("/rental-massachusetts-homes-act-eviction-sealing", axum::routing::post(rental_massachusetts_homes_act_eviction_sealing_route))
        .route("/rental-attached-garage-carbon-monoxide-disclosure", axum::routing::post(rental_attached_garage_carbon_monoxide_disclosure_route))
        .route("/rental-pet-breed-restriction-disclosure", axum::routing::post(rental_pet_breed_restriction_disclosure_route))
        .route("/rental-emergency-action-plan-high-rise", axum::routing::post(rental_emergency_action_plan_high_rise_route))
        .route("/rental-eviction-record-sealing-screening", axum::routing::post(rental_eviction_record_sealing_screening_route))
        .route("/rental-illegal-lockout-self-help-eviction", axum::routing::post(rental_illegal_lockout_self_help_eviction_route))
        .route("/rental-retaliation-prohibition", axum::routing::post(rental_retaliation_prohibition_route))
        .route("/rental-landlord-notice-to-enter", axum::routing::post(rental_landlord_notice_to_enter_route))
        .route("/rental-security-deposit-return-notice", axum::routing::post(rental_security_deposit_return_notice_route))
        .route("/rental-late-fee-cap", axum::routing::post(rental_late_fee_cap_route))
        .route("/rental-tenant-criminal-background-screening", axum::routing::post(rental_tenant_criminal_background_screening_route))
        .route("/rental-source-of-income-discrimination", axum::routing::post(rental_source_of_income_discrimination_route))
        .route("/rental-tenant-abandoned-personal-property", axum::routing::post(rental_tenant_abandoned_personal_property_route))
        .route("/rental-texas-hb-2127-state-preemption", axum::routing::post(rental_texas_hb_2127_state_preemption_route))
        .route("/rental-tenant-bill-of-rights-handout", axum::routing::post(rental_tenant_bill_of_rights_handout_route))
        .route("/rental-minneapolis-renter-protections-ordinance-2020", axum::routing::post(rental_minneapolis_renter_protections_ordinance_2020_route))
        .route("/rental-mold-disclosure-remediation", axum::routing::post(rental_mold_disclosure_remediation_route))
        .route("/rental-multilingual-lease-translation", axum::routing::post(rental_multilingual_lease_translation_route))
        .route("/rental-fair-housing-reasonable-accommodation", axum::routing::post(rental_fair_housing_reasonable_accommodation_route))
        .route("/rental-facade-inspection-fisp-local-law-11", axum::routing::post(rental_facade_inspection_fisp_local_law_11_route))
        .route("/rental-boiler-inspection-compliance", axum::routing::post(rental_boiler_inspection_compliance_route))
        .route("/rental-tenant-rent-escrow-habitability-dispute", axum::routing::post(rental_tenant_rent_escrow_habitability_dispute_route))
        .route("/rental-tree-removal-dangerous-tree-disclosure", axum::routing::post(rental_tree_removal_dangerous_tree_disclosure_route))
        .route("/rental-tenant-right-to-counsel-eviction", axum::routing::post(rental_tenant_right_to_counsel_eviction_route))
        .route("/rental-ada-accessible-parking-compliance", axum::routing::post(rental_ada_accessible_parking_compliance_route))
        .route("/rental-smoke-free-cannabis-restriction", axum::routing::post(rental_smoke_free_cannabis_restriction_route))
        .route("/rental-rent-control-stabilization", axum::routing::post(rental_rent_control_stabilization_route))
        .route("/rental-rent-increase-notice-requirement", axum::routing::post(rental_rent_increase_notice_requirement_route))
        .route("/rental-rent-to-own-lease-purchase-disclosures", axum::routing::post(rental_rent_to_own_lease_purchase_disclosures_route))
        .route("/rental-tenant-relocation-assistance", axum::routing::post(rental_tenant_relocation_assistance_route))
        .route("/rental-tenant-data-privacy-compliance", axum::routing::post(rental_tenant_data_privacy_compliance_route))
        .route("/rental-tenant-estoppel-certificate", axum::routing::post(rental_tenant_estoppel_certificate_route))
        .route("/rental-ev-charging-accommodation", axum::routing::post(rental_ev_charging_accommodation_route))
        .route("/rental-waste-recycling-collection-mandate", axum::routing::post(rental_waste_recycling_collection_mandate_route))
        .route("/rental-washington-hb-1217-rent-stabilization", axum::routing::post(rental_washington_hb_1217_rent_stabilization_route))
        .route("/rental-dc-topa-tenant-opportunity-purchase", axum::routing::post(rental_dc_topa_tenant_opportunity_purchase_route))
        .route("/rental-dog-bite-liability", axum::routing::post(rental_dog_bite_liability_route))
        .route("/rental-drone-overflight-surveillance-privacy", axum::routing::post(rental_drone_overflight_surveillance_privacy_route))
        .route("/rental-domestic-violence-lock-change-lease-termination", axum::routing::post(rental_domestic_violence_lock_change_lease_termination_route))
        .route("/rental-swimming-pool-drain-safety", axum::routing::post(rental_swimming_pool_drain_safety_route))
        .route("/rental-underground-storage-tank-disclosure", axum::routing::post(rental_underground_storage_tank_disclosure_route))
        .route("/rental-unpermitted-unit-disclosure", axum::routing::post(rental_unpermitted_unit_disclosure_route))
        .route("/rental-vacant-property-registration", axum::routing::post(rental_vacant_property_registration_route))
        .route("/rental-vawa-2022-federal-housing-protections", axum::routing::post(rental_vawa_2022_federal_housing_protections_route))
        .route("/rental-vehicle-towing-notice-sign-requirements", axum::routing::post(rental_vehicle_towing_notice_sign_requirements_route))
        .route("/rental-water-submetering-disclosure", axum::routing::post(rental_water_submetering_disclosure_route))
        .route("/rental-well-water-disclosure", axum::routing::post(rental_well_water_disclosure_route))
        .route("/rental-window-blind-cord-safety", axum::routing::post(rental_window_blind_cord_safety_route))
        .route("/rental-window-guard-installation", axum::routing::post(rental_window_guard_installation_route))
        .route("/residential-lease-arbitration-clause", axum::routing::post(residential_lease_arbitration_clause_route))
        .route("/landlord-repair-response-timeframe", axum::routing::post(landlord_repair_response_timeframe_route))
        .route("/landlord-retaliation-damages", axum::routing::post(landlord_retaliation_damages_route))
        .route("/landlord-security-device-obligations", axum::routing::post(landlord_security_device_obligations_route))
        .route("/landlord-self-help-eviction-prohibition", axum::routing::post(landlord_self_help_eviction_prohibition_route))
        .route("/landlord-tenant-recording-consent", axum::routing::post(landlord_tenant_recording_consent_route))
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
        .route("/tenant-accessible-parking", axum::routing::post(tenant_accessible_parking_route))
        .route("/tenant-assistance-animal-accommodation", axum::routing::post(tenant_assistance_animal_accommodation_route))
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
        .route("/rent-control-lease-disclosure", axum::routing::post(rent_control_lease_disclosure_route))
        .route("/rent-overcharge-recovery", axum::routing::post(rent_overcharge_recovery_route))
        .route("/rubs-utility-billing-disclosure", axum::routing::post(rubs_utility_billing_disclosure_route))
        // State habitability remedies available to tenants
        .route("/habitability-remedies", axum::routing::post(habitability_remedies_route))
        // State security deposit cap compliance check
        .route("/security-deposit-cap-check", axum::routing::post(security_deposit_cap_route))
        .route("/security-deposit-interest-statement", axum::routing::post(security_deposit_interest_statement_route))
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
// tenant_accessible_parking: Rental property tenant accessible parking
// accommodation right — when a trader-landlord must (1) provide
// accessible parking spaces as a matter of design and construction
// requirements under FHA + state law, AND (2) grant a reasonable
// accommodation request from a disabled tenant. Mounted at POST /api/
// rental/tenant-accessible-parking. Three regimes: Federal FHA only
// (42 USC § 3604(f) reasonable accommodation universal across all
// multifamily rentals + 24 CFR § 100.205(c) 2% accessible parking
// design and construction for covered multifamily first occupied
// after March 13, 1991; exemptions for <4 unit buildings and
// multifamily townhouses without elevator); California FEHA (Cal.
// Gov. Code §§ 12955(c) + 12927(c) + Cal. Civ. Code § 54.1 Disabled
// Persons Act with $4K+ statutory damages); Default federal FHA only
// (ADA Title III not generally applicable to private residential
// housing). Three-prong reasonable accommodation test under §
// 100.204: (1) tenant FHA disability + (2) accommodation necessary
// for equal opportunity + (3) accommodation reasonable. Distinct
// from siblings emotional_support_animal_documentation, service_
// animal, fha_design_construction, fair_chance_housing.
// ---------------------------------------------------------------------------

async fn tenant_accessible_parking_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<TenantAccessibleParkingInput>,
) -> Result<Json<TenantAccessibleParkingResult>, ApiError> {
    Ok(Json(check_tenant_accessible_parking(&b)))
}

// ---------------------------------------------------------------------------
// tenant_assistance_animal_accommodation: multi-jurisdictional tenant
// assistance animal accommodation framework — among the highest-stakes
// landlord exposure regimes in residential landlord-tenant law. Federal
// Fair Housing Act § 3604(f)(3)(B) (42 USC § 3604(f)(3)(B)) + HUD FHEO
// Notice 2020-01 (January 28, 2020) defining two animal types (service
// animals + support animals); pet fee/deposit prohibition; no breed /
// weight / species restrictions; documentation standards (no specific
// form, notarized statement, perjury statement, diagnosis, detailed
// impairment info); § 3604(f)(9) individualized direct-threat OR
// substantial-property-damage defense; ADA Title III public-
// accommodation overlay (dogs + miniature horses only); § 504
// Rehabilitation Act for federally-funded housing; Cal. Gov. Code
// § 12955 + Cal. Civ. Code § 54.1 FEHA overlay; Cal. AB 468 of 2021
// (effective January 1, 2022) — California-specific ESA documentation
// 30-day established-client-relationship requirement. FHA private
// enforcement under 42 USC § 3613: actual + PUNITIVE damages + attorney
// fees + costs + injunctive relief. HUD administrative penalties under
// 24 CFR § 30.65 (2026): FIRST OFFENSE $25,597; SECOND $63,993;
// THIRD+ $127,985. Mounted at POST /api/rental/tenant-assistance-animal-
// accommodation. Trader-landlord critical because misclassifying an
// emotional support animal as a "pet" is the single most common Fair
// Housing Act discrimination complaint received by HUD. Sibling
// cluster: rental_pet_deposit_separate_security (general pet rules),
// tenant_data_privacy (HIPAA-adjacent), fair_chance_housing,
// landlord_self_help_eviction_prohibition.
// ---------------------------------------------------------------------------

async fn tenant_assistance_animal_accommodation_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<TenantAssistanceAnimalAccommodationInput>,
) -> Result<Json<TenantAssistanceAnimalAccommodationResult>, ApiError> {
    Ok(Json(check_tenant_assistance_animal_accommodation(&b)))
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
// squatter_unauthorized_occupant_removal: Squatter / unauthorized occupant
// removal procedures. Five regimes: Florida (Fla. Stat. § 82.036 HB 621
// EXPEDITED 24-hour sheriff removal on verified owner affidavit eff. July
// 1, 2024 + § 82.036(8) felony penalties for fraudulent docs / damage;
// strongest pro-owner squatter law in US); NewYork (RPAPL § 711(1) April
// 22, 2024 amendment excludes squatters from "tenant" definition + § 713
// summary holdover with 10-day notice to quit + 30-day-occupancy
// threshold abolished); California (Cal. Civ. Proc. § 1161 unlawful
// detainer with 3-day notice + UD complaint + writ of possession; no
// expedited squatter pathway); Texas (Tex. Prop. Code §§ 24.005, 24.005
// (c), 24.002 forcible entry and detainer with 3-day notice for tenant-
// at-sufferance / squatter); Default (common-law ejectment + state-
// specific summary procedure; self-help universally prohibited). Distinct
// from adverse_possession_claim (statutory title acquisition), eviction_
// notices (formal eviction for tenants), holdover_tenant_damages.
// ---------------------------------------------------------------------------

async fn squatter_removal_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<SquatterRemovalInput>,
) -> Result<Json<SquatterRemovalResult>, ApiError> {
    Ok(Json(check_squatter_removal(&b)))
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
// non_refundable_cleaning_fees: Non-refundable cleaning / move-in fee
// enforceability — landlord compliance check for whether a fee labeled
// "non-refundable" in the lease is actually enforceable as such or gets
// converted to a refundable security deposit. Five regimes: California
// (Cal. Civ. Code § 1950.5(n) STRICT PROHIBITION — broad § 1950.5(b)
// security definition voids non-refundable label regardless of
// disclosure); Texas (Tex. Prop. Code Ch. 92 + § 92.103 — permitted with
// lease disclosure); Washington (RCW 59.18.285 — permitted ONLY in
// written lease with clear non-refundable designation; no written lease
// → landlord LIABLE for full fee; written lease lacking designation →
// treated as refundable deposit under §§ 59.18.260/.270/.280); New York
// (GOL § 7-108(1-a) HSTPA 2019 IMPLICIT PROHIBITION via 1-month security
// cap + advance-payment limit); Default (written-lease disclosure
// required). Distinct from pet_fees, application_fees, damage_deduction
// _itemization, security_deposit_caps.
// ---------------------------------------------------------------------------

async fn non_refundable_cleaning_fees_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<CleaningFeeInput>,
) -> Result<Json<CleaningFeeResult>, ApiError> {
    if b.fee_amount_cents < 0 || b.monthly_rent_cents < 0 || b.ny_existing_security_deposit_cents < 0 {
        return Err(ApiError::BadRequest(
            "all dollar inputs must be >= 0".into(),
        ));
    }
    Ok(Json(check_non_refundable_cleaning_fees(&b)))
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
// lease_termination_catastrophic_damage: Tenant lease termination right
// for catastrophic property damage (fire, flood, hurricane, earthquake,
// explosion, similar casualty). Five regimes: California (Cal. Civ. Code
// § 1932(2) greater-part destruction tenant election + § 1933(4) entire-
// destruction automatic termination + § 1950.5(g) 21-day deposit
// refund); Texas (Tex. Prop. Code § 92.054(a)/(b) totally-unusable
// standard + § 92.054(c) written notice before repairs complete +
// § 92.052(b) insurance-proceeds-trigger repair period unique procedural
// rule); NewYork (RPL § 227 fire-or-other-casualty surrender-possession
// right + tenant affirmative election); NewJersey (N.J.S.A. 46:8-6
// total destruction + § 46:8-7 partial destruction proportional rent
// reduction + § 46:8-8 fault attribution); Default (common-law
// impossibility of performance per Restatement (Second) of Contracts §
// 261). Tenant fault uniformly defeats termination right. Distinct
// from dv_termination, military_termination, crime_victim_termination,
// habitability_remedies.
// ---------------------------------------------------------------------------

async fn lease_termination_catastrophic_damage_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<CatastrophicDamageInput>,
) -> Result<Json<CatastrophicDamageResult>, ApiError> {
    Ok(Json(check_catastrophic_damage(&b)))
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
// move_in_fee_cap: Move-in fee cap and disclosure obligations — when
// landlord charges one-time non-refundable move-in fee (cleaning +
// screening + administrative + lease-prep), how much may that fee be,
// what purposes may it cover, what disclosure must accompany it?
// Mounted at POST /api/rental/move-in-fee-cap. Four regimes: Seattle
// SMC § 7.24.030 + RCW 59.18.285 + RCW 59.18.610 (most explicit —
// non-refundable fees ONLY for cleaning + screening; capped at 10% of
// one month's rent; security deposit + fees combined ≤ one month's
// rent; disclosed-as-non-refundable required); Washington RCW
// 59.18.285 (state-wide disclosure-only; if undisclosed reclassified
// as refundable deposit; no amount cap); Chicago Mun. Code § 5-12-080
// + § 5-12-081 RLTO (no amount cap; itemized purpose disclosure
// required; fee is landlord's property no interest); Default (no
// statutory cap or disclosure obligation; common-law unconscionability
// only). Distinct from `application_fees` (pre-tenancy screening),
// `late_fee_caps` (post-tenancy delinquency), `advance_rent_limit`
// (advance rent), `move_in_inspection` (procedural walk-through).
// ---------------------------------------------------------------------------

async fn move_in_fee_cap_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<MoveInFeeCapInput>,
) -> Result<Json<MoveInFeeCapResult>, ApiError> {
    Ok(Json(check_move_in_fee_cap(&b)))
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

// ---------------------------------------------------------------------------
// mandatory_renters_insurance_provider_choice: Tenant right to choose
// renters insurance provider when landlord mandates coverage — anti-tying
// framework. Distinct from `renters_insurance` (general framework +
// coverage minimums) and `rental_junk_fee_transparency` (non-rent fee
// transparency). Mounted at POST /api/rental/mandatory-renters-insurance-
// provider-choice. Three regimes: (1) California Cal. Ins. Code + Cal.
// Civ. Code § 1942.6 + § 1750 et seq. (Consumers Legal Remedies Act) +
// Cal. Bus. & Prof. Code § 17200 (UDAP / Unfair Competition Law) —
// landlord may require renters insurance and specify coverage minimums
// but may NOT mandate specific insurer; recommendation OK; affiliate
// financial interest heightens scrutiny. (2) New York N.Y. Gen. Bus.
// Law § 349 (Deceptive Acts and Practices, $50 min / $1,000 max
// statutory damages + treble damages + attorney fees on willful) + N.Y.
// Ins. Law § 2502 (limited license / insurance agent regulation,
// landlord acting as de facto unlicensed agent). (3) Default common-law
// anti-tying + state UDAP (47 states + DC) + 15 U.S.C. § 45 FTC Act
// § 5 UDAP. No state prohibits requirement of insurance entirely; all
// regimes prohibit MANDATING specific provider.
// ---------------------------------------------------------------------------

async fn mandatory_renters_insurance_provider_choice_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<MandatoryRentersInsuranceInput>,
) -> Result<Json<MandatoryRentersInsuranceResult>, ApiError> {
    Ok(Json(check_mandatory_renters_insurance_provider_choice(&b)))
}

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
// water_heater_earthquake_strap: California Health & Safety Code §
// 19211 (Article 8 Water Heater Strapping and Installation, Chapter 2
// Earthquake Protection) compliance — § 19211(a) all new + replacement
// + existing residential water heaters must be braced / anchored /
// strapped to resist falling or horizontal displacement due to
// earthquake motion; minimum standard = California Plumbing Code Title
// 24 Part 5 or local modifications. § 19211(b) "water heater" means
// standard water heater capacity ≤ 120 gallons for which pre-engineered
// strapping kit is readily available. § 19211(c) building or dwelling
// unit in violation = NUISANCE; breaches implied warranty of
// habitability. § 19211(d) seller must certify § 19211 compliance IN
// WRITING to prospective purchaser. Two regimes: California (strict §
// 19211 + Plumbing Code + nuisance + cert); Default (no statutory
// strap requirement; IPC where adopted + common-law premises
// liability). Mounted at POST /api/rental/water-heater-earthquake-
// strap. Trader-landlord critical for CA rental owners — insurance
// carriers may deny earthquake / fire / flood claims tied to non-
// compliant water heaters. Distinct from siblings meth_contamination_
// disclosure, mold_disclosure, fire_sprinkler_disclosure,
// detector_requirements.
// ---------------------------------------------------------------------------

async fn water_heater_earthquake_strap_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<WaterHeaterEarthquakeStrapInput>,
) -> Result<Json<WaterHeaterEarthquakeStrapResult>, ApiError> {
    Ok(Json(check_water_heater_earthquake_strap(&b)))
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
// rent_stabilized_mci_iai_passthrough: NY rent-stabilized Major Capital
// Improvement (MCI) + Individual Apartment Improvement (IAI) rent
// passthrough compliance — when a trader-landlord owning a rent-
// stabilized NYC building may lawfully pass through capital-
// improvement costs to tenants via rent increase. Mounted at POST
// /api/rental/rent-stabilized-mci-iai-passthrough. Three improvement
// types: MajorCapitalImprovement (9 NYCRR § 2202.4(d); 2% annual cap
// on collectibility; amortization 12 years ≤ 35 units / 12.5 years
// > 35 units; requires DHCR application + tenant notification +
// approval); IndividualApartmentStandard (9 NYCRR § 2202.4; eff. Oct
// 17, 2024 NY Budget — $30,000 cap up from HSTPA 2019 $15,000; now
// PERMANENT; rent formula 1/168 ≤ 35 units / 1/180 > 35 units);
// IndividualApartmentSpecialTier ($50,000 cap for units continuously
// occupied ≥ 25 years OR registered vacant in 2022 + 2023 + 2024;
// formula 1/144 ≤ 35 units / 1/156 > 35 units). NY HSTPA 2019 (Pub.
// L. 2019 ch. 36); DHCR Fact Sheet #24; DHCR Operational Bulletin
// 2024-2. Distinct from siblings rent_control, rent_control_lease_
// disclosure, landlord_annual_rent_statement, rent_increase_notice_
// period.
// ---------------------------------------------------------------------------

async fn rent_stabilized_mci_iai_passthrough_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentStabilizedPassthroughInput>,
) -> Result<Json<RentStabilizedPassthroughResult>, ApiError> {
    Ok(Json(check_rent_stabilized_mci_iai_passthrough(&b)))
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
// tenant_domestic_violence_lease_termination: Tenant domestic
// violence early lease termination compliance — when a tenant or
// household member is a victim of domestic violence, sexual assault,
// stalking, or human trafficking and seeks to terminate the rental
// agreement early without penalty. Mounted at POST /api/rental/
// tenant-domestic-violence-lease-termination. Four regimes: Federal
// VAWA Reauthorization Act of 2022 (34 USC § 12491 + 24 CFR §
// 5.2005; HUD-covered housing — Section 8, public housing, LIHTC §
// 42, HOME, Section 202/811; Form HUD-91066 self-certification
// accepted; lease provisions terminating on police calls in DV
// situations are VOID; lease provisions requiring waiver of VAWA
// rights are VOID; emergency transfer plan required); California
// Cal. Civ. Code § 1946.7 (14-day notice + 180-day documentation
// lookback — restraining/protective order OR police report OR
// qualified third-party statement; tenant liable for rent only up
// to 14 days; confidentiality of documentation; landlord retaliation
// prohibited); Illinois Safe Homes Act 765 ILCS 750 (§ 750/15(a)(1)
// termination if DV occurred at premises; protective order OR
// qualified third-party statement; § 750/30 eviction defense;
// § 750/25 confidentiality); Washington RCW 59.18.575 (90-day
// termination window from reported DV act; tenant liable for month
// of termination but discharged from rent thereafter; protective
// order OR qualified third-party report; confidentiality + non-
// retaliation). Distinct from siblings tenant_accessible_parking
// (ADA), rental_application_denial_disclosure (screening), rental_
// bed_bug_disclosure (lease disclosure), tenant_data_privacy.
// ---------------------------------------------------------------------------

async fn tenant_domestic_violence_lease_termination_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<TenantDomesticViolenceLeaseTerminationInput>,
) -> Result<Json<TenantDomesticViolenceLeaseTerminationResult>, ApiError> {
    Ok(Json(check_tenant_domestic_violence_lease_termination(&b)))
}

// ---------------------------------------------------------------------------
// tenant_ev_charging_installation_right: multi-jurisdictional tenant
// electric vehicle (EV) charging installation right framework. Cal. Civ.
// Code § 1947.6 (AB 2565 of 2014, effective July 1, 2015) — landlord
// SHALL APPROVE tenant request to install EVCS at allotted parking
// space; § 1947.6(d) four exemptions (10%+ EVCS exist, no parking in
// lease, < 5 parking spaces, rent control); § 1947.6(c) tenant
// obligations include written agreement + payment for usage/damage/
// maintenance + $1M liability insurance naming landlord as additional
// insured. Cal. Civ. Code § 1952.7 commercial-lease companion. Colorado
// HB 23-1233 (effective August 7, 2023) — tenant may install Level 1
// or 2 EVCS at own expense; landlord may not charge fee for placement/
// use beyond actual electricity cost or reasonable access fee; state
// Electrical Board EVCS requirements effective March 1, 2024. Maryland
// HB 830 (Chapter 582 of 2023) — newly constructed/renovated units
// with separate garage/carport/driveway per unit must have EVSE-
// installed OR EV-ready parking space. NY Gen. Bus. Law § 399-zzz +
// NY MDL amendments — reasonable approval; may not unreasonably
// withhold. States with right-to-charge laws: CA + CO + FL + HI + IL +
// MD + NJ + NY + OR + VA. Mounted at POST /api/rental/
// tenant-ev-charging-installation-right. Trader-landlord critical
// because EV adoption shifted EVCS requests from novelty to routine.
// Sibling cluster: tenant_solar_installation,
// tenant_clothesline_drying_right,
// rental_satellite_dish_installation_right,
// rental_broadband_mte_rules.
// ---------------------------------------------------------------------------

async fn tenant_ev_charging_installation_right_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<TenantEvChargingInstallationRightInput>,
) -> Result<Json<TenantEvChargingInstallationRightResult>, ApiError> {
    Ok(Json(check_tenant_ev_charging_installation_right(&b)))
}

// ---------------------------------------------------------------------------
// tenant_fire_safety_plan_disclosure: Tenant fire safety plan +
// emergency preparedness notice disclosure compliance — when a
// trader-landlord must post and distribute fire safety plans +
// emergency preparedness notices to tenants of multi-unit residential
// properties. Mounted at POST /api/rental/tenant-fire-safety-plan-
// disclosure. Three regimes: NYC HMC § 27-2046 + Article 11 + HPD
// Required Signs (3+ apartments; Fire Safety Plan posted at entrance
// + mailed annually; Emergency Preparedness Notice on inside of all
// apartment entrance doors + lobby/common area; Smoke Detector
// Notice at/near mailboxes; CO Detector Notice in common area);
// California Cal. Health & Safety Code §§ 13145 + 17926 + 2010
// Carbon Monoxide Poisoning Prevention Act + 1991 Smoke Detector Act
// (3+ apartments; fire alarm disclosure at entrance OR lobby);
// Default IFC § 403.10 (4+ unit buildings; state-specific
// habitability). Distinct from siblings detector_requirements (smoke
// + CO detector hardware), fire_sprinkler_disclosure (fire-
// suppression), water_heater_earthquake_strap, landlord_emergency_
// entry_notice.
// ---------------------------------------------------------------------------

async fn tenant_fire_safety_plan_disclosure_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<TenantFireSafetyPlanDisclosureInput>,
) -> Result<Json<TenantFireSafetyPlanDisclosureResult>, ApiError> {
    Ok(Json(check_tenant_fire_safety_plan_disclosure(&b)))
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

// ---------------------------------------------------------------------------
// tenant_noise_nuisance_enforcement: multi-jurisdictional tenant noise
// nuisance enforcement framework — Cal. Civ. Code § 1941.4 + Andrews v.
// Mobile Aire Estates, 125 Cal. App. 4th 578 (2005) (covenant of quiet
// enjoyment IMPLIED in every lease); NYC Admin. Code § 24-218 (no
// unreasonable noise; nighttime 10pm-7am 7 dB(A) above ambient; daytime
// 10 dB(A); residential apartment-to-apartment amplified-audible
// violation at ANY HOUR; DEP + NYPD enforcement; 311 complaint pathway);
// Chicago Municipal Code § 8-32 (quiet hours 10pm-8am weekdays /
// 10pm-10am weekends; 75-feet-audible presumptive violation; amplified
// music 9pm-8am prohibited); Mass. G.L. c. 186 § 14 (willful interference
// CRIMINAL + TREBLE DAMAGES + attorney fees + Berman & Sons v. Jefferson
// 379 Mass. 196 (1979) constructive eviction); Default common-law
// nuisance + implied covenant of quiet enjoyment. Mounted at POST /api/
// rental/tenant-noise-nuisance-enforcement. Trader-landlord critical
// because noise complaints among most common multifamily grievances;
// landlord has DUTY TO ABATE known noise nuisances; failure exposes
// landlord to rent abatement + warranty of habitability claim + lease
// termination + constructive eviction + damages. Sibling cluster:
// landlord_self_help_eviction_prohibition, tenant_rent_escrow_
// withholding, landlord_emergency_entry_notice, rental_window_guard_
// installation.
// ---------------------------------------------------------------------------

async fn tenant_noise_nuisance_enforcement_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<TenantNoiseNuisanceEnforcementInput>,
) -> Result<Json<TenantNoiseNuisanceEnforcementResult>, ApiError> {
    Ok(Json(check_tenant_noise_nuisance_enforcement(&b)))
}

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
// tenant_positive_rent_reporting: California AB 2747 of 2024 (codified at
// Cal. Civ. Code § 1954.07, effective April 1, 2025) — residential
// landlord of 16+ unit building MUST OFFER tenants option to have
// positive rental payment information reported to at least one
// nationwide consumer reporting agency. § 1954.07 15-or-fewer-unit
// exemption WITH corporate-multiple-building carveout. Offer timing:
// at lease execution + at least once annually; for outstanding leases
// (Jan 1, 2025): no later than April 1, 2025 + annually. Fee cap:
// lesser of actual cost OR $10/month. Positive-only definition:
// COMPLETE, TIMELY payments — no incomplete/late reporting. Tenant
// protections: failure to pay fee NOT cause for termination; NOT
// deductible from security deposit; if unpaid 30+ days, landlord may
// stop reporting + tenant blocked 6 months. Colorado HB 23-1099 +
// Washington SB 5495 of 2024 + HUD Tenant Credit Reporting Pilot (FY
// 2023-2025) parallel state expansions. FCRA (15 USC § 1681 et seq.)
// furnisher duties apply under § 1681s-2. Mounted at POST /api/rental/
// tenant-positive-rent-reporting. Trader-landlord critical because
// (1) annual repeat-offer obligation; (2) $10/month strict fee cap;
// (3) failure to offer creates per-violation civil exposure; (4)
// FCRA exposure for misreporting; (5) tenant fee non-payment cannot
// trigger eviction or security deposit deduction. Sibling cluster:
// landlord_annual_rent_statement, tenant_data_privacy,
// rental_application_denial_disclosure, fair_chance_housing.
// ---------------------------------------------------------------------------

async fn tenant_positive_rent_reporting_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<TenantPositiveRentReportingInput>,
) -> Result<Json<TenantPositiveRentReportingResult>, ApiError> {
    Ok(Json(check_tenant_positive_rent_reporting(&b)))
}

// ---------------------------------------------------------------------------
// tenant_rights_statement_disclosure: Tenant Rights Statement disclosure —
// when must a residential landlord distribute the official state-prepared
// Statement of Tenants' Rights and Responsibilities? Mounted at POST
// /api/rental/tenant-rights-statement-disclosure. Four regimes: (1) New
// Jersey Truth in Renting Act N.J.S.A. §§ 46:8-43 to 46:8-50: DCA
// publishes bilingual English+Spanish Statement updated ANNUALLY; landlord
// must distribute within 30 DAYS of publication to existing tenants + at or
// prior to occupancy for new tenants + post in prominent accessible
// location; applies to buildings > 2 units (or non-owner-occupied > 3
// units); no tenant waiver permitted; treble damages under N.J.S.A.
// 56:8-1 (Consumer Fraud Act). (2) Maryland Md. Code, Real Property §
// 8-208: limited tenant-rights notice; county / municipal supplements
// extend coverage. (3) New York: HSTPA of 2019 imposes discrete-disclosure
// mandates but NO statutory annual-distribution mandate; DHCR + AG guides
// voluntary. (4) Default: no statewide mandate; municipal ordinances may
// impose (Chicago, San Francisco). Distinct from `lease_disclosures`
// (mandated lease content), `plain_language_lease` (lease readability),
// and `landlord_identification_disclosure` (party-identification).
// ---------------------------------------------------------------------------

async fn tenant_rights_statement_disclosure_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<TenantRightsStatementInput>,
) -> Result<Json<TenantRightsStatementResult>, ApiError> {
    Ok(Json(check_tenant_rights_statement_disclosure(&b)))
}

// ---------------------------------------------------------------------------
// tenant_utility_account_designation: Tenant right to designate utility
// service account in tenant's own name — when may the residential tenant
// become the direct customer of record with the utility company (vs being
// forced onto landlord's master account)? Mounted at POST
// /api/rental/tenant-utility-account-designation. Three regimes: (1)
// California Cal. Pub. Util. Code §§ 777, 777.1 — most explicit framework:
// tenant in INDIVIDUALLY METERED residential service has right to become
// direct customer when landlord is customer of record; utility MUST inform
// occupants; tenant NOT required to pay landlord's delinquent amount;
// occupant may verify via lease/rent receipts/government document; §
// 777 does NOT apply to master-metered apartment buildings (RUBS / master-
// meter pass-through outside protection). (2) New York N.Y. Pub. Serv.
// Law §§ 32, 33, 33-a (Home Energy Fair Practices Act / HEFPA) —
// residential utility customer protections under PSC tariff framework;
// tenant in shared-meter arrangement may petition NY PSC for separate
// account. (3) Default — no statewide right; state-PUC tariff + lease
// control. Distinct from `submetering_rules` (sub-metering setup),
// `utility_shutoff` (shutoff procedures), and `non_refundable_cleaning_
// fees` (move-out fees).
// ---------------------------------------------------------------------------

async fn tenant_utility_account_designation_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<TenantUtilityAccountInput>,
) -> Result<Json<TenantUtilityAccountResult>, ApiError> {
    Ok(Json(check_tenant_utility_account_designation(&b)))
}

// ---------------------------------------------------------------------------
// tenant_window_air_conditioner_install_right: Multi-jurisdictional
// tenant window air conditioner installation right and landlord cooling
// provision compliance framework. When may a tenant install a window
// air conditioner unit in a rental property, what bracket safety and
// falling-AC liability rules apply, when must landlord provide cooling
// under emerging state and municipal heat-mandate laws, and what
// failure-mode liabilities expose landlord after a heat-stress fatality
// or falling-AC injury? Mounted at POST /api/rental/tenant-window-air-
// conditioner-install-right. Three-jurisdiction framework: New York
// City (MOST RECENT + MOST PRESCRIPTIVE — NYC Int 0994 of 2024 'Cool
// Homes for All Act' enacted 2024-2025, fully effective 2026, requires
// landlord to provide AC + maintain bedroom temperature ≤78°F from
// June 15 to September 15 when outdoor temp > 82°F; market-rate and
// rent-stabilized apply; NYC Window AC Bracket requirements for
// buildings taller than 6 stories; NYC Admin. Code § 27-2029 heating
// code framework); California (Cal. Civ. Code § 1941.1 implied
// warranty of habitability heating; Cal. Health & Safety Code
// § 17920.3 substandard conditions includes inability to maintain
// reasonable temperature; select CA cities like Palm Springs require
// landlord cooling); Arizona (Phoenix City Code § 39-16 + Pima
// County Code 8.20 + ARS § 33-1324(C) require landlord-provided
// cooling in summer months); Default (common-law implied warranty
// per Hilder v. St. Peter, 478 A.2d 202 (Vt. 1984) + Green v.
// Superior Court 10 Cal. 3d 616 (1974) + tort negligence + premises
// liability). Three install paths: tenant-installed with lease
// provision; landlord-provided pre-installed (NYC Cool Homes Act);
// reasonable accommodation request under FHA 42 U.S.C. § 3604(f).
// Window AC bracket safety: buildings taller than 6 stories require
// bracket; window frame load capacity test; bracket per manufacturer
// specifications with fasteners into structural framing; annual
// inspection. Five failure modes: landlord unreasonable prohibition;
// AC installed without bracket on upper floor → falling-AC injury;
// NYC Cool Homes Act violation post-2026 → ECB + rent reduction;
// heat-stress event → $1M+ tort settlement; bracket not maintained →
// premises liability. Distinct from siblings cooling_requirements,
// rental_gas_appliance_ban, tenant_solar_installation, tenant_ev_
// charging_installation_right, rental_window_guard_installation,
// tenant_emotional_distress_damages, rental_natural_gas_leak_response
// (iter 485).
// ---------------------------------------------------------------------------

async fn tenant_window_air_conditioner_install_right_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<TenantWindowAirConditionerInstallRightInput>,
) -> Result<Json<TenantWindowAirConditionerInstallRightResult>, ApiError> {
    Ok(Json(check_tenant_window_air_conditioner_install_right(&b)))
}

// ---------------------------------------------------------------------------
// tenant_smart_lock_biometric_consent: Tenant right to refuse landlord-
// installed BIOMETRIC smart lock (fingerprint + facial recognition +
// retinal scanner) — biometric privacy framework. Mounted at POST
// /api/rental/tenant-smart-lock-biometric-consent. Five regimes: (1)
// Illinois 740 ILCS 14 BIPA (most aggressive: § 15(b) written consent +
// purpose disclosure + length of term + written release; § 15(c)
// no-profit; § 20 PRIVATE RIGHT OF ACTION with $1,000 negligent /
// $5,000 reckless statutory damages + attorney fees + costs; Cothron
// v. White Castle 2023 IL 128004 per-scan violation accrual). (2)
// Washington RCW 19.375 Biometric Identifiers Act (AG enforcement
// only, no private right of action). (3) Texas Tex. Bus. & Comm.
// Code § 503.001 ($25,000 per violation AG civil penalty, no
// private right of action). (4) California Cal. Civ. Code §§
// 1798.80 + 1798.100 et seq. CCPA/CPRA classify biometric as
// sensitive personal information + Cal. Civ. Code § 1940.2
// prohibited harassment. (5) Default no biometric statute; FTC Act
// § 5 (15 U.S.C. § 45) + state UDAP + common-law invasion of privacy.
// Anti-tying principle: traditional key access must be offered as
// alternative when tenant refuses biometric enrollment. Distinct
// from `security_camera_disclosure` (surveillance cameras),
// `lock_change_between_tenancies` (between-tenancy locks), and
// `landlord_tenant_recording_consent` (audio recording).
// ---------------------------------------------------------------------------

async fn tenant_smart_lock_biometric_consent_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<TenantSmartLockBiometricInput>,
) -> Result<Json<TenantSmartLockBiometricResult>, ApiError> {
    Ok(Json(check_tenant_smart_lock_biometric_consent(&b)))
}

// ---------------------------------------------------------------------------
// tenant_rent_judgment_wage_garnishment: Post-judgment wage garnishment
// for tenant rent debt. Three regimes: FullyProhibited (TX, NC, PA, SC —
// civil-debt wage garnishment ABSOLUTELY barred at employer level per
// Tex. Const. art. XVI § 28; N.C. Const. art. X § 1 + N.C. Gen. Stat. §
// 1-362; 42 Pa. Cons. Stat. § 8127; S.C. Code § 37-5-104); StateMore
// Protective (CA Code Civ. Proc. § 706.050 50× state min, MA G.L. c. 246
// § 28 50× state min, VA Code § 34-29 75% / 40× federal min); FederalFloor
// (15 U.S.C. § 1673(a)(1) — lesser of 25% disposable or excess over 30×
// federal minimum, currently $7.25 → $217.50/week 30× threshold). Carve-
// outs: child support / alimony (§ 1673(a)(2) tiers 50-65%), tax debt,
// federally backed student loan (15% administrative). Distinct from
// damage_deduction_itemization, rent_credit_reporting, prevailing_party_
// attorney_fees.
// ---------------------------------------------------------------------------

async fn tenant_rent_judgment_wage_garnishment_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<GarnishmentInput>,
) -> Result<Json<GarnishmentResult>, ApiError> {
    if b.weekly_disposable_earnings_cents < 0
        || b.federal_minimum_wage_cents_per_hour < 0
        || b.state_minimum_wage_cents_per_hour < 0
    {
        return Err(ApiError::BadRequest(
            "monetary inputs must be >= 0".into(),
        ));
    }
    if b.state_exemption_multiplier_hours > 1_000 {
        return Err(ApiError::BadRequest(
            "state_exemption_multiplier_hours out of plausible range".into(),
        ));
    }
    Ok(Json(compute_wage_garnishment(&b)))
}

// ---------------------------------------------------------------------------
// tenant_rent_escrow_withholding: multi-jurisdictional tenant rent escrow
// / rent withholding for habitability violations framework — Cal. Civ.
// Code § 1942 + § 1942.4 (repair-and-deduct up to 1 month rent twice per
// 12-month period + § 1942.4 statutory damages $100-$5,000 + attorney
// fees + 35-day governmental notice trigger + violation-not-caused-by-
// tenant prong); N.Y. Real Prop. Law § 235-b (implied warranty of
// habitability + WAIVER VOID + rent abatement + repair-and-deduct + rent
// withholding under RPAPL § 711 nonpayment defense; Park West Management
// Corp. v. Mitchell, 47 N.Y.2d 316 (1979)); Mass. G.L. c. 239 § 8A +
// 105 CMR 410 (rent withholding defense + LOCAL BOARD OF HEALTH report
// + INTO ESCROW payment); Chicago RLTO § 5-12-110 (14-day cure + half-
// month/$500 repair-and-deduct cap + lease termination option); Pugh v.
// Holmes, 486 Pa. 272 (1979) (common-law implied warranty); Default —
// common-law habitability framework. Mounted at POST /api/rental/
// tenant-rent-escrow-withholding. Trader-landlord critical because
// implied warranty of habitability is among the most powerful tenant
// defenses — tenant can REDUCE OR WITHHOLD RENT without losing
// possession; many state waivers VOID as contrary to public policy.
// Sibling cluster: rental_carbon_monoxide_detector,
// rental_basement_water_intrusion_disclosure,
// rental_bedroom_egress_window, landlord_repair_response_timeframe.
// ---------------------------------------------------------------------------

async fn tenant_rent_escrow_withholding_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<TenantRentEscrowWithholdingInput>,
) -> Result<Json<TenantRentEscrowWithholdingResult>, ApiError> {
    Ok(Json(check_tenant_rent_escrow_withholding(&b)))
}

// ---------------------------------------------------------------------------
// tenant_rent_receipt_requirement: multi-jurisdictional tenant rent
// receipt requirement framework — N.Y. Real Prop. Law § 235-e (cash /
// money order / cashier's check / non-personal-check → immediate
// in-person OR 15-day non-in-person receipt; personal-check on tenant
// request with monthly-recurring obligation after first request; 6
// required content elements (date + amount + period + apartment number
// + signature + title); 3-YEAR record retention for cash receipts);
// Cal. Civ. Code § 1499 (signed-and-dated receipt at TIME OF PAYMENT
// upon tenant request; all payment methods); Mass. G.L. c. 186 § 15B
// (LIMITED last-month's-rent-at-commencement signed receipt mandate;
// regular monthly rent NOT required); Wash. Rev. Code § 59.18.063
// (cash mandatory; non-cash on request); Default — common-law
// payment-of-rent dispute defense; some local ordinances (Chicago RLTO
// § 5-12-080(g), San Francisco) impose receipt requirements. Mounted
// at POST /api/rental/tenant-rent-receipt-requirement. Trader-landlord
// critical because (1) cash rent receipts mandatory in many states;
// (2) receipt-issuance failures create per-violation civil exposure +
// evidentiary presumption against landlord in rent-payment disputes;
// (3) 3-year retention extends beyond tenancy termination; (4) modern
// payment methods (Zelle, Venmo, ACH) require careful receipt
// practices. Sibling cluster: landlord_annual_rent_statement,
// tenant_late_fee_cap, tenant_positive_rent_reporting,
// rental_junk_fee_transparency.
// ---------------------------------------------------------------------------

async fn tenant_rent_receipt_requirement_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<TenantRentReceiptRequirementInput>,
) -> Result<Json<TenantRentReceiptRequirementResult>, ApiError> {
    Ok(Json(check_tenant_rent_receipt_requirement(&b)))
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
// family_childcare_home_right: Tenant family child-care home (FCCH)
// operation right — when may a landlord lawfully prohibit a tenant
// from operating a licensed family day care home for children in a
// residential rental unit? Mounted at POST /api/rental/family-
// childcare-home-right. Three regimes: California Cal. Health & Safety
// Code §§ 1597.40-1597.46 + Cal. Civ. Code § 1950.5 (most explicit —
// lease prohibition VOID under § 1597.40(a); 30-day advance written
// notice required from tenant under § 1597.40(c); landlord may require
// increased FCCH security deposit under § 1597.40(d) but capped by AB
// 12 / § 1950.5 1-month rent ceiling; § 1597.40(e) state preemption
// over municipal zoning/building/fire codes; § 1597.42 requires Title
// 22 CDSS license); New York N.Y. Social Services Law § 390 + N.Y.
// Real Property Law § 235-b (licensed FCCH + group FCCH protected;
// landlord may not unreasonably withhold consent; OCFS license
// required); Default federal Fair Housing Act 42 USC § 3604 familial
// status (covers families with children from refusal but NOT
// childcare-business operation). Distinct from siblings
// `tenant_organizing`, `tenant_data_privacy`, and `fair_chance_
// housing`.
// ---------------------------------------------------------------------------

async fn family_childcare_home_right_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<FamilyChildcareHomeInput>,
) -> Result<Json<FamilyChildcareHomeResult>, ApiError> {
    Ok(Json(check_family_childcare_home_right(&b)))
}

// ---------------------------------------------------------------------------
// source_of_income_discrimination: SOI discrimination ban — when may
// landlord lawfully refuse Housing Choice Voucher (Section 8), HUD-
// VASH, SSI/SSDI, child support/alimony, TANF, or other lawful non-wage
// income as basis for rejecting applicant? Mounted at POST /api/rental/
// source-of-income-discrimination. Four regimes: California Cal. Gov.
// Code §§ 12955 + 12987 + 12989.2 + SB 329 (Housing Opportunities Act
// 2019, eff. 2020) — Section 8 explicitly added to FEHA source-of-
// income definition; max civil penalty + actual + emotional distress +
// attorney fees + injunctive; New Jersey N.J.S.A. 10:5-12.5 (NJ LAD) —
// DCR penalty tiers $1K-$5K initial, up to $10K first / $25K
// subsequent + private right of action; New York N.Y. Exec. Law §
// 296(5)(a)(1) (state SOI eff. April 2019) + NYC Admin. Code § 8-
// 107(5)(a)(5) — NYC Commission >$780K damages since 2014; Default
// federal Fair Housing Act 42 USC § 3604 — NO per se SOI protection,
// Section 8 refusal not federal FHA violation absent disparate
// treatment/impact. Distinct from `fair_chance_housing` (criminal
// background) and `tenant_in_foreclosure_protection`.
// ---------------------------------------------------------------------------

async fn source_of_income_discrimination_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<SourceOfIncomeInput>,
) -> Result<Json<SourceOfIncomeResult>, ApiError> {
    Ok(Json(check_source_of_income_discrimination(&b)))
}

// ---------------------------------------------------------------------------
// fha_design_construction: Fair Housing Act design and construction
// requirements for covered multifamily dwellings — 24 CFR § 100.205.
// Federal accessibility requirements for new-construction multifamily
// buildings first occupied AFTER March 13, 1991. § 100.205(c)(1)-(7)
// seven design requirements: (1) accessible entrance on accessible
// route, (2) accessible public/common-use areas, (3) usable doors
// (32" clear), (4) accessible route into and through dwelling, (5)
// environmental controls in accessible locations, (6) reinforced
// walls for grab bars, (7) usable kitchens and bathrooms.
// § 100.205(a) terrain-impracticality defense (burden on builder).
// § 100.205(a) covered = 4+ units with elevator (all units) OR
// ground-floor units in non-elevator buildings. 42 U.S.C. § 3613(c)
// private suit damages + § 3612 HUD administrative penalty ($25,597
// first / $63,991 repeat within 5 years, 2025 inflation-adjusted).
// FEDERAL FLOOR — state codes (CA Title 24, MA AAB, NY HRL) may add
// stricter requirements but cannot reduce baseline.
// ---------------------------------------------------------------------------

async fn fha_design_construction_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<FhaDesignConstructionInput>,
) -> Result<Json<FhaDesignConstructionResult>, ApiError> {
    Ok(Json(check_fha_design_construction(&b)))
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
// dog_breed_restriction_ban: Dog breed-specific restriction (BSL) ban
// in residential rental housing — when may a landlord lawfully refuse
// to rent to a tenant purely because of the breed of the tenant's
// dog? Mounted at POST /api/rental/dog-breed-restriction-ban. Two
// regimes: Nevada (SB 166 eff. October 1, 2025 — insurers MAY NOT use
// dog breed as a factor when underwriting landlord liability policies
// for multi-family residential properties; SB 103 2021 — insurers MAY
// NOT deny coverage based SOLELY on dog breed; NV Rev. Stat. §
// 202.500 2013 statewide BSL preemption — local governments CANNOT
// pass laws banning specific breeds; Nevada was 14th state to
// prohibit BSL; landlords may require pet deposits + liability
// coverage + behavior screenings but MAY NOT refuse to rent purely
// because of breed); Default (no statewide protection; ~22 states
// have BSL preemption laws varying by state; many still allow
// breed-specific bans at city/county level; insurance may still
// consider breed in most states; FHA does NOT cover breed unless
// tied to disability service-animal accommodation). Distinct from
// siblings pet_fees (pet deposit caps), emotional_support_animal_
// documentation, service_animal, tenant_organizing, fair_chance_
// housing.
// ---------------------------------------------------------------------------

async fn dog_breed_restriction_ban_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<DogBreedRestrictionBanInput>,
) -> Result<Json<DogBreedRestrictionBanResult>, ApiError> {
    Ok(Json(check_dog_breed_restriction_ban(&b)))
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
// tenant_clothesline_drying_right: Tenant clothesline / drying-rack
// right under California Civil Code § 1940.20 (eff. January 1, 2016
// per AB 1448 Stats. 2015 ch. 415). Mounted at POST /api/rental/
// tenant-clothesline-drying-right. Two regimes: California (Cal. Civ.
// Code §§ 1940.20 + 4750.10 — clothesline = cord/rope/wire; drying
// rack = apparatus; balcony/railing/awning EXCLUDED from definitions;
// private area = outdoor or enclosed area with door access; common
// areas EXCLUDED; six conditions: (1) no interference with maintenance
// (2) no health/safety hazard (3) no blocking doorways/walkways/
// utility equipment (4) tenant consent if affixed to building (5) no
// violation of reasonable time/location restrictions); Default (no
// specific tenant right; lease provisions controlling; HOA right-to-
// dry laws exist in 20+ states for HOMEOWNERS but rental rare outside
// CA). Distinct from siblings tenant_solar_installation, ev_charger_
// installation, tenant_organizing, flag_display_right.
// ---------------------------------------------------------------------------

async fn tenant_clothesline_drying_right_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<TenantClotheslineDryingRightInput>,
) -> Result<Json<TenantClotheslineDryingRightResult>, ApiError> {
    Ok(Json(check_tenant_clothesline_drying_right(&b)))
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
// soft_story_seismic_retrofit: Mandatory soft-story seismic retrofit
// ordinance compliance — when a trader-landlord owning a wood-frame
// multifamily building with a soft/weak first story (open ground-
// floor parking, retail, or similar large openings lacking lateral
// bracing) must commission a mandatory structural retrofit. Mounted at
// POST /api/rental/soft-story-seismic-retrofit. Four regimes: San
// Francisco (Building Code Chapter 34B, Ord. 66-13 operative June 17,
// 2013 — wood-frame 5+ residential units + 2+ stories over soft story;
// all tier deadlines PASSED September 15, 2021; non-compliance = SF
// Building Code § 102A.16 UNSAFE + ~$840/day penalty); Los Angeles
// (Ordinance 183893 Nov 22, 2015 — ~13,500 wood-frame soft-story
// buildings; 3-phase compliance timeline 2yr structural report / 3.5yr
// permits / 7yr retrofit-complete; Priority 2 deadline April 2026 +
// LAMC § 91.6314 enforcement); Berkeley (BMC Chapter 19.39 eff. 2015 —
// wood-frame multifamily ≥ 3 units + soft-story first floor; ongoing
// certification); Default (no statutory requirement; common-law
// premises liability + state building code + local ordinances; Oakland
// + San Jose + Pasadena + West Hollywood have analogous programs).
// Trader-landlord critical for CA multifamily owners — non-compliance
// exposes owner to ordinance penalties + uninhabitable-building
// findings + insurance non-renewal. Distinct from siblings balcony_
// inspection (SB 721/326 EEE visual inspection), water_heater_
// earthquake_strap (CA § 19211 individual appliance), fire_sprinkler_
// disclosure (fire suppression).
// ---------------------------------------------------------------------------

async fn soft_story_seismic_retrofit_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<SoftStorySeismicRetrofitInput>,
) -> Result<Json<SoftStorySeismicRetrofitResult>, ApiError> {
    Ok(Json(check_soft_story_seismic_retrofit(&b)))
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
// lease_early_termination_fee_cap: Lease early-termination fee + liquidated
// damages cap enforceability — when a residential tenant breaks the lease
// early, what statutory cap or duty-to-mitigate framework limits the
// landlord's recovery? Mounted at POST /api/rental/lease-early-termination-
// fee-cap. Three regimes: (1) Florida Fla. Stat. § 83.595(4): explicit
// 2-MONTH RENT cap + 60-day tenant notice + SEPARATE ADDENDUM with
// statutory wording + election waives additional rent beyond month of
// retaking possession + § 83.595(2) menu of remedies (terminate-retake,
// re-rent-on-tenant-account, hold-for-full-rent, OR liquidated-damages).
// (2) California Cal. Civ. Code § 1951.2 + § 1671(d): ACTUAL DAMAGES
// framework with strict duty to mitigate; liquidated damages clauses
// presumptively VOID unless reasonable estimate at lease execution. (3)
// Default common-law: actual damages + duty to mitigate per Restatement
// (Second) of Contracts §§ 350 (mitigation) + 356 (liquidated damages vs
// penalty). Distinct from siblings `duty_to_mitigate_damages` (general
// mitigation), `rent_acceleration_enforceability` (full balance
// acceleration), `lease_termination_catastrophic_damage` (force-majeure),
// and `military_termination` (SCRA).
// ---------------------------------------------------------------------------

async fn lease_early_termination_fee_cap_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<LeaseEarlyTerminationFeeInput>,
) -> Result<Json<LeaseEarlyTerminationFeeResult>, ApiError> {
    if b.monthly_rent_cents < 0 || b.fee_amount_cents < 0 {
        return Err(ApiError::BadRequest(
            "non-negative cents inputs required".into(),
        ));
    }
    Ok(Json(check_lease_early_termination_fee_cap(&b)))
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
// short_term_rental_conversion: Short-term rental conversion
// restriction compliance — when a trader-landlord may lawfully convert
// a long-term residential rental unit to a short-term rental (Airbnb /
// VRBO / Booking.com listing under 30 nights) without violating local
// registration ordinances. Mounted at POST /api/rental/short-term-
// rental-conversion. Four regimes: NYC Local Law 18 of 2022 (eff. Sep
// 5, 2023 — all hosts must register with Office of Special
// Enforcement OSE; applies to stays under 30 nights; booking
// platforms must verify registration; up to $5,000 per violation;
// host must be permanent occupant AND PRESENT during stay); San
// Francisco Admin. Code Chapter 41A Ordinance 218-14 (Business
// Registration Certificate required; primary residence 270 nights/
// year occupancy; 90 unhosted nights cap; up to $1,000 per day);
// Los Angeles LAMC § 12.22 A.32 Home-Sharing Ordinance (eff. Nov 1,
// 2019 — LADBS Home-Sharing Program registration; primary residence
// requirement; 120 unhosted days/year default cap; 240 days/year
// with Extended Home-Sharing permit); Default (locality-controlled;
// FL/AZ preempt local STR regulation; NY/CA/CO permit local).
// Distinct from siblings rental_property_registration, condominium_
// conversion_protection, tenant_relocation_assistance, just_cause_
// eviction.
// ---------------------------------------------------------------------------

async fn short_term_rental_conversion_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<ShortTermRentalConversionInput>,
) -> Result<Json<ShortTermRentalConversionResult>, ApiError> {
    Ok(Json(check_short_term_rental_conversion(&b)))
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
// mid_tenancy_security_deposit_increase: Mandatory landlord-paid
// prohibition on mid-tenancy security deposit increase. Mounted at POST
// /api/rental/mid-tenancy-security-deposit-increase. Four regimes:
// California Cal. Civ. Code § 1950.5(c) + AB 12 (one month unfurnished /
// two months furnished cap effective July 1, 2024; mid-tenancy increase
// requires lease modification basis + within cap + tenant written
// consent); New Jersey N.J.S.A. §§ 46:8-21.2 + 46:8-19 (prohibited absent
// lease modification or proportional rent increase; 1.5 month cap; bad-
// faith DOUBLE DAMAGES + attorney fees); New York N.Y. Gen. Oblig. Law
// § 7-108(1-a)(a) HSTPA 2019 (statewide one-month cap); Default lease
// controls + common-law contract modification + good-faith doctrine.
// Distinct from `security_deposit_caps` (initial cap), `damage_deduction_
// itemization`, `deposit_interest`, `security_deposit_bank_disclosure`,
// and `deposit_return_windows`.
// ---------------------------------------------------------------------------

async fn mid_tenancy_security_deposit_increase_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<MidTenancySecurityDepositInput>,
) -> Result<Json<MidTenancySecurityDepositResult>, ApiError> {
    Ok(Json(check_mid_tenancy_security_deposit_increase(&b)))
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
// mid_tenancy_temporary_relocation: Mid-tenancy temporary relocation
// rights when landlord requires tenant to temporarily vacate for
// substantial repairs / renovation / abatement. Four regimes: California
// (Cal. Civ. Code § 1946.2(d)(2) substantial-remodel just-cause + 30-day
// vacancy threshold + § 1946.2(d)(3) one-month-rent relocation
// assistance OR alternative housing + § 1942.5(b) tenant right to
// return + SF Rent Ordinance § 37.9(a)(11) + Long Beach SRTD local
// overlays); NewJersey (N.J.S.A. § 2A:18-61.1(g) Anti-Eviction Act
// renovation removal + § 2A:18-61.11 alternative housing OR relocation
// expenses); Washington (RCW 59.18.085 displacement assistance + Seattle
// SMC 22.210 + Bellingham Tenant Protections Ordinance local overlays);
// Default (lease + common-law habitability + municipal overlays).
// Distinct from tenant_relocation_assistance (no-fault permanent
// eviction), lease_termination_catastrophic_damage (fire/flood
// termination), demolition_tenant_notice (permanent unit demolition).
// ---------------------------------------------------------------------------

async fn mid_tenancy_temporary_relocation_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<TemporaryRelocationInput>,
) -> Result<Json<TemporaryRelocationResult>, ApiError> {
    Ok(Json(check_mid_tenancy_temporary_relocation(&b)))
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
// HOA fee enforcement against tenant in single-family / townhome / condo
// rentals — when can a homeowners association enforce delinquent dues
// DIRECTLY against the tenant rather than only the owner-landlord? Mounted at
// POST /api/rental/hoa-fee-tenant-enforcement. Four regimes: (1) Florida —
// Fla. Stat. § 720.3085(8)(a)-(d) single-family HOA + § 718.116(11)
// condominium: HOA may DEMAND tenant pay subsequent rent directly to
// association when parcel owner delinquent; tenant immune from landlord
// claim for amounts paid to association; tenant rent credit against
// landlord-owed rent; HOA eviction authority for nonpayment after written
// demand; demand MUST be via hand delivery OR United States mail. (2) Texas
// — Tex. Prop. Code § 209.0064: HOA enforces ONLY against OWNER; lease
// passthrough governed by lease terms; § 209.0064 third-party collection
// requires written certified-mail notice to OWNER + 45-day cure. (3)
// California — Cal. Civ. Code §§ 5650, 5710, 5715 Davis-Stirling Act: HOA
// foreclose on owner's interest only, NOT tenant. (4) Default — owner-only
// enforcement; federal FDCPA (15 U.S.C. § 1692) applies if third-party
// collector hired. Distinct from `hoa_rental_restriction` (HOA's
// restrictions ON renting).
// ---------------------------------------------------------------------------

async fn hoa_fee_tenant_enforcement_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<HoaFeeTenantEnforcementInput>,
) -> Result<Json<HoaFeeTenantEnforcementResult>, ApiError> {
    Ok(Json(check_hoa_fee_tenant_enforcement(&b)))
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
// tenant_in_unit_appliance_repair_responsibility: Multi-jurisdictional
// tenant in-unit appliance repair and replacement responsibility
// compliance framework. When a landlord provides appliances
// (refrigerator, stove/range, dishwasher, washer/dryer, microwave,
// HVAC) as part of a rental tenancy, what repair and replacement
// obligations attach under the implied warranty of habitability and
// state-specific mandatory-provision statutes, what tenant remedies
// apply, and what failure-mode liabilities expose landlord when an
// appliance fails? Mounted at POST /api/rental/tenant-in-unit-
// appliance-repair-responsibility. Four-jurisdiction framework: New
// York (MOST PRESCRIPTIVE for refrigerator provision — NYC Admin.
// Code § 27-2014 + § 27-2008 + N.Y. Multiple Dwelling Law § 78 require
// refrigerator and stove in NYC multi-unit dwellings + N.Y. Real
// Property Law § 235-b implied warranty + 28 RCNY § 32-04 HPD
// enforcement); Texas (Tex. Prop. Code § 92.052 landlord duty to
// repair conditions materially affecting health/safety + § 92.053
// notice + § 92.056 tenant remedies including repair-and-deduct up
// to one month's rent + lease termination + judicial order);
// California (Cal. Civ. Code § 1941 + § 1941.1 implied warranty +
// § 1941.1(a)(8) STOVE/RANGE requirement + § 1942(a) repair-and-
// deduct max twice in 12-month period + § 1942.1 no-waiver + Green
// v. Superior Court, 10 Cal. 3d 616 (1974) + Hinson v. Delis, 26
// Cal. App. 3d 62 (1972) appliance-specific habitability); Default
// (common-law implied warranty per Hilder v. St. Peter, 478 A.2d
// 202 (Vt. 1984) + Lemle v. Breeden, 51 Haw. 426 (1969)). Six
// categories of in-unit appliances: stove/range (CA + NYC mandatory),
// refrigerator (NYC mandatory in multi-unit), dishwasher, washer/
// dryer, microwave, HVAC (heat universal + cooling NYC post-2022
// LL18). Five universal failure-mode liabilities: refusal to repair
// → habitability breach + repair-and-deduct; constructive eviction
// from extended non-repair → tenant termination; replacement
// materially lower quality → habitability dispute; tenant-caused
// damage allocation dispute → lease terms vs § 1942.1 no-waiver;
// used appliance failure due to end-of-useful-life → IRS § 168 5-7
// year + tort if injury. Reasonable repair window typically 7 days
// for non-emergency + 24-72 hours for major appliance. Distinct
// from siblings rental_hot_water_temperature, rental_gas_appliance_
// ban, rental_carbon_monoxide_detector, rental_chimney_fireplace_
// inspection_disclosure (iter 471), rental_fire_extinguisher_
// requirement (iter 473), rental_hardwired_smoke_alarm_responsibility
// (iter 481), rental_garage_door_safety_compliance (iter 483),
// rental_natural_gas_leak_response (iter 485), landlord_repair_
// response_timeframe.
// ---------------------------------------------------------------------------

async fn tenant_in_unit_appliance_repair_responsibility_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<TenantInUnitApplianceRepairResponsibilityInput>,
) -> Result<Json<TenantInUnitApplianceRepairResponsibilityResult>, ApiError> {
    Ok(Json(check_tenant_in_unit_appliance_repair_responsibility(&b)))
}

// ---------------------------------------------------------------------------
// tenant_late_fee_cap: multi-jurisdictional residential tenant late fee
// cap framework — Cal. Civ. Code § 1671(d) + Orozco v. Casimiro, 121
// Cal.App.4th Supp. 7 (2004) (liquidated damages 2-prong reasonableness
// test; 5-6% threshold; void in toto on failure); N.Y. Real Prop. Law
// § 238-a (HSTPA of 2019) (LESSER of $50 OR 5% statutory hard cap; 5-day
// mandatory grace period; no eviction solely for late fees); Fla. Stat.
// § 83.808 (manufactured home park reasonable cap; common $20 or 20%
// floor) and Chapter 83 Part II (no statutory cap; court reasonableness);
// Tex. Prop. Code § 92.019 (12% safe harbor for ≤ 4 units / 10% for > 4
// units; 2-day mandatory grace; TREBLE damages + $100 + attorney fees
// for violation); Default common-law liquidated damages under
// Restatement (Second) of Contracts § 356. Mounted at POST /api/rental/
// tenant-late-fee-cap. Trader-landlord critical because late-fee over-
// collection is one of the most common landlord mistakes — each over-cap
// charge can trigger statutory damages plus attorney fees. Sibling
// cluster: rental_security_deposit_interest,
// landlord_self_help_eviction_prohibition,
// landlord_retaliation_damages, rental_junk_fee_transparency.
// ---------------------------------------------------------------------------

async fn tenant_late_fee_cap_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<TenantLateFeeCapInput>,
) -> Result<Json<TenantLateFeeCapResult>, ApiError> {
    Ok(Json(check_tenant_late_fee_cap(&b)))
}

// ---------------------------------------------------------------------------
// tenant_lease_guarantor_disclosure: multi-jurisdictional tenant lease
// guarantor disclosure and restriction framework — NY HSTPA 2019 + DHCR
// Operational Bulletin 2020-1 (rent-stabilized one-month aggregate
// security + guaranty cap; no retroactive guarantor; tenant blacklist
// $500-$1,000 civil penalty); NY GOL § 5-701(a)(1) Statute of Frauds (>
// 12-month guaranty in writing); CA Civ. Code § 2787-2856 suretyship +
// § 2819 material modification rule + § 1670.5 unconscionability +
// § 2799 continuing guaranty revocation; NJ N.J.S.A. 46:8-26 + NJ
// Consumer Fraud Act (lease copy + exact monetary limit); Federal FCRA
// (15 USC § 1681 et seq.) adverse-action notice (§ 1681m) + willful/
// negligent damages (§ 1681n + § 1681o); Restatement (Third) of
// Suretyship and Guaranty (1996) § 41 material modification +  § 39
// novation extinguishes guaranty + strict construction. Mounted at POST
// /api/rental/tenant-lease-guarantor-disclosure. Trader-landlord
// critical: NY HSTPA aggregate one-month cap on security + guaranty;
// FCRA adverse-action notice required when guarantor application denied
// based on credit report; common-law material-modification rule
// extinguishes guaranty on subsequent rent increases without consent;
// NJ + best practice requires exact monetary cap on guaranty. Sibling
// cluster: tenant_data_privacy, rental_application_denial_disclosure,
// tenant_late_fee_cap, tenant_rent_receipt_requirement.
// ---------------------------------------------------------------------------

async fn tenant_lease_guarantor_disclosure_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<TenantLeaseGuarantorDisclosureInput>,
) -> Result<Json<TenantLeaseGuarantorDisclosureResult>, ApiError> {
    Ok(Json(check_tenant_lease_guarantor_disclosure(&b)))
}

// ---------------------------------------------------------------------------
// Tenant estoppel certificate requirements and protections.
//
// Mounted at POST /api/rental/tenant-estoppel-certificate.
// Three-jurisdiction framework for estoppel certificates used by
// trader-landlords during refinance / sale: (1) New York —
// commercial enforceable with express lease provision; residential
// deemed-admission and attorney-in-fact clauses VOID AS
// UNCONSCIONABLE under RPL § 235-c + GOL § 5-321; GOL § 5-703
// Statute of Frauds for leases > 1 year; (2) California —
// commercial enforceable with express provision; Cal. Civ. Code
// § 1962 landlord identification disclosure; Cal. Civ. Code
// § 1668 NO EXCULPATION FOR FRAUD limits scope of deemed-admission
// clauses; (3) Default — Restatement (Second) of Contracts § 90
// promissory estoppel binds tenant on clear/definite promise +
// foreseeable reliance + actual reliance + injustice avoidable
// only by enforcement. Standard COMMERCIAL response window: 10-15
// business days; failure to return triggers deemed-admission +
// attorney-in-fact + monetary penalty + event of default. Tenant
// protections: cannot bind tenant to facts not known; cannot waive
// statutory rights (rent-stabilization succession in NY); cannot
// serve as pre-dispute waiver of yet-to-accrue claims. Sibling
// cluster: lease_disclosures, lease_copy_delivery,
// tenant_lease_guarantor_disclosure, tenant_rights_statement_
// disclosure, lease_waiver_enforceability, landlord_
// identification_disclosure.
// ---------------------------------------------------------------------------

async fn tenant_estoppel_certificate_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<TenantEstoppelCertificateInput>,
) -> Result<Json<TenantEstoppelCertificateResult>, ApiError> {
    Ok(Json(check_tenant_estoppel_certificate(&b)))
}

// ---------------------------------------------------------------------------
// Landlord property sale notice and security deposit transfer disclosure.
//
// Mounted at POST /api/rental/landlord-property-sale-notice.
// Four-jurisdiction framework for when a trader-landlord sells
// rental property: (1) California — Cal. Civ. Code § 1950.5(h):
// reasonable time + transfer deposit (less lawful deductions) to
// successor + notify tenant by PERSONAL DELIVERY or FIRST-CLASS
// MAIL with deposit amount + claims against deposit + new owner
// name/address/phone; failure triggers joint and several
// liability; (2) New York — NY GOL § 7-105: 5-DAY transfer +
// REGISTERED/CERTIFIED MAIL notice; NY GOL § 7-103(2) requires
// accrued interest transfer for 6+ family dwelling units; failure
// triggers joint and several liability of grantee + seller for
// principal + accrued interest; (3) Massachusetts — Mass. Gen.
// Laws c. 186 § 15B(7): 45-DAY new-owner notice; willful
// violation triggers TREBLE DAMAGES + attorney's fees; successor
// liable for return regardless of receipt from seller;
// (4) Default — common-law reasonable time; 12 USC § 5220
// Protecting Tenants at Foreclosure Act stacks 90-day notice on
// top of state-law obligations during foreclosure sales.
// Sibling cluster: security_deposit_bank_disclosure,
// deposit_interest, deposit_return_windows,
// foreclosure_tenant_rights, landlord_identification_disclosure,
// tenant_estoppel_certificate.
// ---------------------------------------------------------------------------

async fn landlord_property_sale_notice_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<LandlordPropertySaleNoticeInput>,
) -> Result<Json<LandlordPropertySaleNoticeResult>, ApiError> {
    Ok(Json(check_landlord_property_sale_notice(&b)))
}

// ---------------------------------------------------------------------------
// Lease renewal offer timing and disclosure framework.
//
// Mounted at POST /api/rental/lease-renewal-offer-timing.
// Four-jurisdiction framework for when landlord must send renewal
// offer or non-renewal notice: (1) New York rent-stabilized — NY
// Rent Stabilization Code (9 NYCRR) § 2523.5 + DHCR Form RTP-8:
// 90-150 day window; 1-year + 2-year option at tenant's choice;
// mail or personal delivery; failure forfeits rent increase;
// (2) New York non-stabilized — NY RPL § 226-c (HSTPA 2019):
// 30/60/90 day tiers based on tenancy length when rent increase
// >= 5% or non-renewal; (3) California TPA — Cal. Civ. Code
// § 1946.2 (AB 1482, 2019): just-cause requirement; § 1946.2(d)
// one-month-rent relocation assistance for no-fault non-renewal;
// (4) DC — D.C. Code § 42-3505.01 + § 42-3505.54 (Rental Housing
// Act of 1985): 12-month mandatory renewal except enumerated
// just-cause grounds. Sibling cluster: lease_auto_renewal,
// lease_succession, lease_assignment_consent, lease_copy_delivery,
// rent_increase_notice_period.
// ---------------------------------------------------------------------------

async fn lease_renewal_offer_timing_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<LeaseRenewalOfferTimingInput>,
) -> Result<Json<LeaseRenewalOfferTimingResult>, ApiError> {
    Ok(Json(check_lease_renewal_offer_timing(&b)))
}

// ---------------------------------------------------------------------------
// Rent concession disclosure framework.
//
// Mounted at POST /api/rental/rent-concession-disclosure.
// Three-jurisdiction framework: (1) New York rent-stabilized —
// NY RSL § 26-511(c)(14) (HSTPA 2019) locks in preferential rent
// (cannot be revoked during tenancy); renewal increases calculated
// on PREFERENTIAL rent; DHCR Operational Bulletin 2016-1 + Fact
// Sheet #40 amortization formula; failure to register net effective
// rent triggers RSL § 26-516(a) 6-year overcharge lookback + RSL
// § 26-516(a)(2) treble damages; (2) New York non-rent-stabilized
// — NY RPL § 235-a + NY GBL § 349 (UDAP) require clear concession
// disclosure; misrepresentation in credit reporting = deceptive
// practice; (3) California — Cal. Civ. Code § 1947.12 (AB 1482
// Tenant Protection Act) caps annual increase at LOWER of CPI+5%
// or 10% of LOWEST gross rent in prior 12 months; § 1947.15
// governs concession interaction with cap. Sibling cluster:
// lease_disclosures, lease_copy_delivery, tenant_rights_statement_
// disclosure, lease_waiver_enforceability, lease_renewal_offer_
// timing, landlord_identification_disclosure.
// ---------------------------------------------------------------------------

async fn rent_concession_disclosure_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentConcessionDisclosureInput>,
) -> Result<Json<RentConcessionDisclosureResult>, ApiError> {
    Ok(Json(check_rent_concession_disclosure(&b)))
}

// ---------------------------------------------------------------------------
// Tenant rent abatement during landlord construction nuisance framework.
//
// Mounted at POST /api/rental/rent-abatement-construction-nuisance.
// Three-jurisdiction framework: (1) NYC rent-stabilized — 9 NYCRR
// § 2523.4 + DHCR Form RA-84 decreased building-wide services
// rent reduction; NYC HMC § 27-2005.1 24-72 hour pre-construction
// notice + § 27-2005(d) construction harassment civil penalty
// $1,000-$10,000; NYC Noise Code § 24-218 dB limits; Tenant Anti-
// Harassment Act 2018; DHCR Operational Bulletin 2024-1 MCI 2%
// cap; (2) California — Cal. Civ. Code § 1927 covenant of quiet
// enjoyment + § 1941.1 habitability + § 1942 repair-and-deduct +
// § 1942.4 no rent collection during uninhabitability; Green v.
// Superior Court, 10 Cal. 3d 616 (1974); (3) Default — Park West
// Mgmt. Corp. v. Mitchell, 47 N.Y.2d 316 (1979); Boston Housing
// Auth. v. Hemingway, 363 Mass. 184 (1973); Restatement (Second)
// of Property § 5.4 (warranty of habitability) + § 6.1 (covenant
// of quiet enjoyment); URLTA § 2.104 + § 4.103 + § 4.107.
// Five construction nuisance categories (noise, dust, vibration,
// debris, service interruption); industry-standard 22-40%
// abatement during construction; 100% on constructive eviction.
// Sibling cluster: landlord_water_heat_emergency_response,
// habitability_remedies, landlord_pest_extermination_timeline,
// landlord_harassment, tenant_noise_nuisance_enforcement,
// retaliation_windows, tenant_emotional_distress_damages.
// ---------------------------------------------------------------------------

async fn rent_abatement_construction_nuisance_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentAbatementConstructionNuisanceInput>,
) -> Result<Json<RentAbatementConstructionNuisanceResult>, ApiError> {
    Ok(Json(check_rent_abatement_construction_nuisance(&b)))
}

// ---------------------------------------------------------------------------
// Landlord master key retention and access disclosure framework.
//
// Mounted at POST /api/rental/landlord-master-key-retention.
// Four-jurisdiction framework: (1) California — Cal. Civ. Code
// § 1954 landlord entry 24-hour written notice + normal business
// hours; § 1954(c) civil penalty up to $2,000 per unauthorized
// incident; six permitted entry categories (emergency + necessary
// repair + show unit + court order + tenant abandonment + water
// conservation); (2) Texas — Tex. Prop. Code § 92.156 7-day rekey
// at landlord expense + § 92.156(b) landlord-master-key change
// expense MUST be paid by landlord; (3) New York — NY Multiple
// Dwelling Law § 51 + § 53 key management; NYC HMC § 27-2008
// minimum lock standards; NY RPL § 235-d landlord harassment +
// NYC Tenant Anti-Harassment Act 2018 $1,000-$10,000 civil penalty
// per unauthorized entry incident; (4) Massachusetts — Mass. Gen.
// Laws c. 186 § 15B (provide keys at lease start) + § 15F
// (UNAUTHORIZED ENTRY TRIPLE DAMAGES + attorney fees + injunctive
// relief); 105 CMR 410.480 minimum lock standards; Boston Housing
// Auth. v. Hemingway, 363 Mass. 184 (1973). Master-key system
// best-practice elements: written lease disclosure + key-issuance
// log + § 1954/§ 235-d/§ 15B notice protocols + controlled access
// (locked safe) + per-contractor receipt + prohibition on personal
// use. Sibling cluster: entry_notice, landlord_emergency_entry_
// notice, landlord_mid_tenancy_rekeying, lock_change_between_
// tenancies, dv_survivor_lock_change, tenant_smart_lock_biometric_
// consent, landlord_harassment, tenant_emotional_distress_damages,
// landlord_water_heat_emergency_response.
// ---------------------------------------------------------------------------

async fn landlord_master_key_retention_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<LandlordMasterKeyRetentionInput>,
) -> Result<Json<LandlordMasterKeyRetentionResult>, ApiError> {
    Ok(Json(check_landlord_master_key_retention(&b)))
}

// ---------------------------------------------------------------------------
// Tenant holdover security deposit setoff limits framework.
//
// Mounted at POST /api/rental/tenant-holdover-security-deposit-setoff.
// Four-jurisdiction framework + 10-category deduction permission
// matrix: (1) California — Cal. Civ. Code § 1950.5(e) + 21-day
// itemization (§ 1950.5(g)(1)) + § 1950.5(l) bad-faith 2× punitive
// + § 1670.5 unconscionability; (2) New York — NY GOL § 7-108(1-a)
// (g) 4 permitted categories (rent + damage + utilities + moving/
// storage) + § 7-108(1-a)(e) 14-day window with FORFEITURE on
// failure + § 5-321 unconscionability; (3) Texas — Tex. Prop. Code
// § 92.104 + § 92.104(c) rent-owed itemization exception + § 92.103
// 30-day window + § 92.109(a) bad-faith $100 + 3× wrongfully
// withheld + attorney fees; (4) Massachusetts — Mass. Gen. Laws c.
// 186 § 15B(4) 4 permitted categories + § 15B(7) TRIPLE damages
// + interest + fees + 30-day window. Seven permitted categories
// (unpaid holdover rent + double rent damages with lease auth +
// physical damage + eviction fees with lease/statute/prevailed +
// cleaning to baseline + utilities NY + moving/storage NY); three
// universally prohibited (normal wear and tear + pre-existing
// conditions + unconscionable liquidated damages); penalty
// multipliers stack on bad-faith retention or late itemization.
// Sibling cluster: holdover_tenant_damages, damage_deduction_
// itemization, deposit_return_windows, security_deposit_bank_
// disclosure, landlord_property_sale_notice (iter 437),
// duty_to_mitigate_damages, lease_cure_period, abandoned_
// property_handling.
// ---------------------------------------------------------------------------

async fn tenant_holdover_security_deposit_setoff_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<TenantHoldoverSecurityDepositSetoffInput>,
) -> Result<Json<TenantHoldoverSecurityDepositSetoffResult>, ApiError> {
    Ok(Json(check_tenant_holdover_security_deposit_setoff(&b)))
}

// ---------------------------------------------------------------------------
// Rental video surveillance footage retention period framework.
//
// Mounted at POST /api/rental/rental-video-surveillance-retention.
// Four-jurisdiction framework + 5-location matrix: (1) Illinois —
// BIPA 740 ILCS 14/ (most stringent: written consent under 740
// ILCS 14/15(b) + 3-year retention cap + $1,000-$5,000 per violation
// private right of action under 740 ILCS 14/20 + Rosenbach v. Six
// Flags Entm't Corp., 129 N.E.3d 1197 (Ill. 2019) no-injury-in-fact
// rule); (2) Texas — CUBI Tex. Bus. & Com. Code § 503.001 (consent
// + sale prohibition + Texas AG enforcement only at $25,000 per
// violation; SB 9 of 2024 strengthened for minors); (3) California
// — CCPA Cal. Civ. Code § 1798.100 + CPRA Prop. 24 (notice at
// collection + biometric SPI § 1798.140(c)(1) + deletion right
// § 1798.105 + § 1798.150 breach private right of action $100-$750
// per consumer + Cal. Civ. Code § 1708.5 intrusion overlay); (4)
// Default — Restatement (Second) of Torts § 652B intrusion upon
// seclusion. Universally prohibited: hidden cameras + unit
// interior + high-privacy areas + audio recording without all-
// party consent (Wiretap Act 18 U.S.C. § 2510 + Cal. Penal Code
// § 632 + Illinois Eavesdropping 720 ILCS 5/14-2 + MD § 10-402).
// 6-element best-practice framework. Sibling cluster: security_
// camera_disclosure, tenant_smart_lock_biometric_consent,
// tenant_data_privacy, landlord_master_key_retention (iter 459),
// landlord_emergency_entry_notice, landlord_harassment,
// tenant_emotional_distress_damages (iter 453).
// ---------------------------------------------------------------------------

async fn rental_video_surveillance_retention_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalVideoSurveillanceRetentionInput>,
) -> Result<Json<RentalVideoSurveillanceRetentionResult>, ApiError> {
    Ok(Json(check_rental_video_surveillance_retention(&b)))
}

// ---------------------------------------------------------------------------
// Landlord foreclosure status disclosure to tenant framework.
//
// Mounted at POST /api/rental/landlord-foreclosure-status-disclosure.
// Four-jurisdiction framework for landlord obligations during
// mortgage default and active foreclosure: (1) California — Cal.
// Civ. Code § 2924.8 5-business-day post-and-mail notice on
// trustee's sale; § 2924.8(d) punitive damages for knowing/
// intentional violation; § 2924.85 REPEALED January 1, 2018
// (historical pre-lease disclosure); (2) New York — NY RPAPL
// § 1305 10-business-day successor notice after foreclosure
// judgment; RPAPL § 1306 lender DFS filing within 3 business
// days; (3) Federal — 12 USC § 5220 Protecting Tenants at
// Foreclosure Act of 2009 (made permanent by Pub. L. 115-174
// § 304 effective June 23, 2018) 90-day successor notice plus
// remainder of lease for BONA FIDE TENANTS (§ 5220(b) three-
// element test: not mortgagor's family + arm's-length + rent
// not substantially less than FMV); (4) Default — Restatement
// (Second) of Torts § 551 common-law disclosure duty for
// material facts. Sibling cluster: foreclosure_tenant_rights,
// landlord_property_sale_notice, security_deposit_bank_
// disclosure, landlord_identification_disclosure,
// tenant_estoppel_certificate.
// ---------------------------------------------------------------------------

async fn landlord_foreclosure_status_disclosure_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<LandlordForeclosureStatusDisclosureInput>,
) -> Result<Json<LandlordForeclosureStatusDisclosureResult>, ApiError> {
    Ok(Json(check_landlord_foreclosure_status_disclosure(&b)))
}

// ---------------------------------------------------------------------------
// Commercial lease personal guaranty enforceability framework.
//
// Mounted at POST /api/rental/commercial-lease-personal-guaranty-enforceability.
// Four-jurisdiction framework: (1) New York City — NYC Admin. Code
// § 22-1005 (May 26, 2020) renders unenforceable personal guaranties
// of commercial lease obligations arising March 7, 2020 through
// June 30, 2021 for food/beverage on-premises consumption ceased,
// non-essential retail, or required to close under EO; Second
// Circuit Melendez (16 F.4th 992 + 27 F.4th 119) found
// constitutional concerns; SDNY Melendez II (668 F. Supp. 3d 184
// March 31, 2023) held VIOLATES Contracts Clause because PERMANENTLY
// extinguishes guaranties; pending Supreme Court review;
// (2) New York State — NY GOL § 5-701(a)(1)/(2) Statute of Frauds
// requires writing for guaranty of lease > 12 months and ALL
// promises to answer for debt of another; "Good Guy Guaranty"
// industry-standard NYC commercial limits liability to surrender-
// date arrears; (3) California — Cal. Civ. Code § 2787-2856
// suretyship + § 2819 material modification + § 1670.5
// unconscionability + § 2799 continuing-guaranty revocation;
// (4) Default — Restatement (Third) of Suretyship and Guaranty
// (1996) § 41 material modification + § 39 novation extinguishes.
// Sibling cluster: tenant_lease_guarantor_disclosure (residential),
// lease_disclosures, lease_assignment_consent, lease_waiver_
// enforceability, tenant_estoppel_certificate.
// ---------------------------------------------------------------------------

async fn commercial_lease_personal_guaranty_enforceability_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<CommercialLeasePersonalGuarantyEnforceabilityInput>,
) -> Result<Json<CommercialLeasePersonalGuarantyEnforceabilityResult>, ApiError> {
    Ok(Json(check_commercial_lease_personal_guaranty_enforceability(&b)))
}

// ---------------------------------------------------------------------------
// Commercial lease CAM charge disclosure and tenant audit rights.
//
// Mounted at POST /api/rental/commercial-lease-cam-charge-disclosure.
// Three-jurisdiction framework + BOMA Operating Expense Guide
// industry standard: (1) California — Cal. Civ. Code § 1938 CASp
// accessibility disclosure; Garrett v. Coast and Southern, 9 Cal.
// 4th 1 (1995) good-faith reconciliation duty; industry-standard
// annual budget + year-end reconciliation within 90-180 days;
// (2) New York — no specific commercial CAM statute; BOMA
// Operating Expense Guide governs; (3) Default — BOMA 2024
// Operating Expense Guide industry standard; Restatement (Second)
// of Contracts § 200 ambiguity-favoring-tenant; UCC Article 2A
// INAPPLICABLE to real property leases. BOMA 13 CAM categories;
// 12 standard EXCLUSIONS (capital improvements + marketing/
// leasing + landlord debt service + depreciation + income taxes
// + ground rent + reserves + tenant-dispute legal fees +
// landlord penalties + insurance proceeds + above-market
// affiliated-party); GROSS-UP PROVISION (variable expenses
// grossed to 95-100% occupancy floor; fixed expenses NOT grossed
// up); BASE-YEAR ESCALATION formula; TENANT AUDIT RIGHTS 7
// standard provisions (90-180 day notice + most-recent-fiscal-
// year scope + confidentiality + 3-5% discrepancy cost-shift +
// refund obligation + invoice inspection right); BOMA survey
// shows 1 in 4 (25%) tenants experience billing discrepancies.
// Sibling cluster: commercial_lease_personal_guaranty_
// enforceability, tenant_estoppel_certificate, lease_
// disclosures, rental_property_registration.
// ---------------------------------------------------------------------------

async fn commercial_lease_cam_charge_disclosure_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<CommercialLeaseCamChargeDisclosureInput>,
) -> Result<Json<CommercialLeaseCamChargeDisclosureResult>, ApiError> {
    Ok(Json(check_commercial_lease_cam_charge_disclosure(&b)))
}

// ---------------------------------------------------------------------------
// Landlord pest extermination response timeline framework.
//
// Mounted at POST /api/rental/landlord-pest-extermination-timeline.
// Four-jurisdiction framework + severity-graded timelines:
// (1) New York City — NYC HMC § 27-2018 continuous eradication
// duty + broad pest definition (Class Insecta + Phylum Arthropoda
// + Order Rodentia); NYC Local Law 55 of 2018 Integrated Pest
// Management for 3+ unit buildings (annual inspection + least-
// toxic methods + 72-hour pesticide notification + 3-year
// recordkeeping); NYC HPD Class A/B/C violation classes ($25-
// $1,000 daily civil penalties); (2) California — Cal. Civ.
// Code § 1941.1(a)(8) rodent/vermin-free standard + Cal. Civ.
// Code § 1942 repair-and-deduct up to $1,000 or 1 month rent;
// (3) Massachusetts — 105 CMR 410.550 owner extermination duty
// + Mass. Gen. Laws c. 111 § 127A Board of Health enforcement +
// criminal contempt for non-compliance; (4) Default — URLTA
// § 2.104(a)(2) + Restatement (Second) of Property § 5.1
// implied warranty of habitability. Severity-graded response:
// EMERGENCY (rodent health hazard) 24 hours; STANDARD 14 days
// NY/MA or 30 days CA/Default; PREVENTIVE 30 days. Habitability
// remedies: rent withholding (Park West v. Mitchell + Green v.
// Superior Court + Boston Housing Auth. v. Hemingway), repair
// and deduct, rent abatement 50-100%, constructive eviction,
// public enforcement. Distinct from bedbug_extermination_cost
// (bed-bug specific) + bedbug_disclosure (prior-occurrence) +
// rental_pesticide_application_notification (pre-application
// notice). Sibling cluster: habitability_remedies, landlord_
// repair_response_timeframe.
// ---------------------------------------------------------------------------

async fn landlord_pest_extermination_timeline_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<LandlordPestExterminationTimelineInput>,
) -> Result<Json<LandlordPestExterminationTimelineResult>, ApiError> {
    Ok(Json(check_landlord_pest_extermination_timeline(&b)))
}

// ---------------------------------------------------------------------------
// Landlord water/heat emergency response timeline framework.
//
// Mounted at POST /api/rental/landlord-water-heat-emergency-response.
// Four-jurisdiction framework with severity-graded emergency
// response: (1) New York City — NYC HMC § 27-2029 (heat min temp)
// + § 27-2030 (hot water 120°F 365 days) + § 27-2115 ($250-$500
// daily civil penalty per apartment) + § 27-2018 24/7 emergency
// contact requirement for 2+ unit buildings; heat season October
// 1 - May 31 with 68°F daytime / 62°F nighttime; (2) California —
// Cal. Civ. Code § 1941.1(a)(1) waterproofing + (a)(2) hot/cold
// water + § 1942 repair-and-deduct up to one month rent; (3) Texas
// — Tex. Prop. Code § 92.052 landlord duty + § 92.056 tenant
// remedies (termination + actual damages + one month rent + $500
// civil penalty + attorney's fees); (4) Default — URLTA § 2.104
// + § 4.103 emergency repair-and-deduct + Restatement (Second) of
// Property § 5.4. Three-tier severity: TIER 1 IMMEDIATE (24-hour
// response) for no heat in season + no hot water + sewage backup
// + active flooding + gas leak + smoke/CO detector failure +
// unsecured exterior door + electrical hazard + structural
// collapse; TIER 2 URGENT (72-hour) for reduced hot water + partial
// heat loss + slow leak + appliance failure; TIER 3 STANDARD (7-
// day) for cosmetic water staining + slow plumbing + HVAC
// inefficiency. Tenant remedies citing Park West v. Mitchell +
// Green v. Superior Court + Boston Housing Auth. v. Hemingway.
// Sibling cluster: habitability_remedies, landlord_repair_
// response_timeframe, landlord_pest_extermination_timeline (iter
// 449), detector_requirements, heat_requirements,
// cooling_requirements, rental_basement_water_intrusion_
// disclosure.
// ---------------------------------------------------------------------------

async fn landlord_water_heat_emergency_response_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<LandlordWaterHeatEmergencyResponseInput>,
) -> Result<Json<LandlordWaterHeatEmergencyResponseResult>, ApiError> {
    Ok(Json(check_landlord_water_heat_emergency_response(&b)))
}

// ---------------------------------------------------------------------------
// Tenant emotional distress damages framework.
//
// Mounted at POST /api/rental/tenant-emotional-distress-damages.
// Four-jurisdiction framework + Restatement (Second) Torts § 46
// IIED four-element test: (1) extreme and outrageous conduct
// beyond all bounds of decency; (2) intent or recklessness;
// (3) actual and proximate causation; (4) severe emotional
// distress. NIED jurisdictional split: TEXAS IMPACT RULE
// (Boyles v. Kerr, 855 SW 2d 593 (Tex. 1993)) requires physical
// injury; ZONE OF DANGER (IL/PA/NJ) requires plaintiff in zone
// of physical danger; BYSTANDER/Dillon v. Legg majority (CA/NY
// most states) recoverable for observing physical injury to
// close relative. Punitive damages: CA § 3294 clear-and-
// convincing malice + no statutory cap + Campbell ratio 1-9×;
// NY common-law preponderance + no cap; TX § 41.008 greater of
// 2× economic + non-economic up to $750K OR $200K; default
// Restatement § 908. Landlord conduct categories: systematic
// harassment campaign, deliberate utility shutoff, threats of
// violence, frivolous eviction lawsuits, unauthorized entry,
// extortionate demands, deliberate habitability destruction,
// discriminatory animus. Distinct from sibling landlord_
// harassment (civil penalties), landlord_retaliation_damages,
// lockout_penalties, retaliation_windows, habitability_remedies.
// Citations: Hughes v. Pair (CA), Howell v. NYP Holdings (NY),
// Twyman v. Twyman + Boyles v. Kerr (TX), Dillon v. Legg + State
// Farm v. Campbell, Cal. Civ. Code § 1940.2 + § 789.3 + § 3294,
// SF Rent Ordinance § 37.10B, NY RPL § 235-d + § 234, NYC HMC
// § 27-2005(d) + Tenant Anti-Harassment Act 2018, Tex. Prop.
// Code § 92.0081 + Tex. Civ. Prac. & Rem. Code § 41.008.
// ---------------------------------------------------------------------------

async fn tenant_emotional_distress_damages_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<TenantEmotionalDistressDamagesInput>,
) -> Result<Json<TenantEmotionalDistressDamagesResult>, ApiError> {
    Ok(Json(check_tenant_emotional_distress_damages(&b)))
}

// ---------------------------------------------------------------------------
// Landlord negative credit reporting framework.
//
// Mounted at POST /api/rental/landlord-negative-credit-reporting.
// Federal FCRA furnisher framework + FDCPA + 7-year statute of
// limitations + state-law overlay. FCRA § 1681s-2(a) accuracy and
// integrity + § 1681s-2(b) mandatory non-discretionary
// investigation duty (5-element: investigate + review + report +
// cross-CRA correction + delete unverifiable per § 1681s-2(b)(1)
// (E)) + § 1681c 7-year limitation from DATE OF DELINQUENCY +
// § 1681n willful civil liability (statutory $100-$1,000 +
// punitive + fees per Safeco Ins. Co. v. Burr 551 U.S. 47 (2007))
// + § 1681o negligent civil liability. FDCPA § 1692e/§ 1692f/
// § 1692g(a) 5-day validation notice + § 1692g(b) verification on
// dispute + § 1692k civil liability + CFPB Regulation F (12 CFR
// § 1006). State overlay: NY GBL § 380 + § 380-d + NY RPL
// § 227-f tenant blacklist prohibition ($500-$1,000 per
// violation); Cal. Civ. Code § 1785 + § 1786; Conn. Gen. Stat.
// § 47a-71 30-day pre-reporting notice; Oregon SB 970 +
// Washington RCW 59.18.367 sealed eviction record protection.
// Distinct from sibling rent_credit_reporting (positive reporting
// under Cal. Civ. Code § 1954.06 / AB 2747). Companion to
// tenant_data_privacy, adverse_action_notice, credit_check_
// authorization, application_fees, tenant_rent_judgment_wage_
// garnishment.
// ---------------------------------------------------------------------------

async fn landlord_negative_credit_reporting_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<LandlordNegativeCreditReportingInput>,
) -> Result<Json<LandlordNegativeCreditReportingResult>, ApiError> {
    Ok(Json(check_landlord_negative_credit_reporting(&b)))
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

// ---------------------------------------------------------------------------
// landlord_annual_rent_statement: Mandatory landlord-provided annual rent
// statement to tenant for tenant tax-credit purposes — when must a
// residential landlord issue an annual rent-paid statement to enable the
// tenant to claim a state-level renter's tax credit or property tax
// refund? Mounted at POST /api/rental/landlord-annual-rent-statement.
// Three regimes: (1) Minnesota Minn. Stat. § 290A.19 — Certificate of
// Rent Paid (CRP) MUST be issued by JANUARY 31 each year; electronic OR
// hard copy; supports tenant's Property Tax Refund / Renter's Credit
// claim under § 290A; if landlord fails to issue, tenant may request a
// Rent Paid Affidavit from MN Department of Revenue (which audits and
// imposes state-law penalties on landlord). (2) Vermont Vt. Stat. tit.
// 32 § 6066 — Form LRC-147 required for tenant Renter Rebate claim.
// (3) Default — no statewide proactive landlord statement mandate;
// renter-tax-credit states (MA + MI + WI + IN + IA + ME + MD) typically
// have tenants claim based on tenant's own records; landlord must
// produce records on tenant request. Distinct from `rent_receipts`
// (per-payment receipts) and `security_deposit_interest_statement`.
// ---------------------------------------------------------------------------

async fn landlord_annual_rent_statement_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<LandlordAnnualRentStatementInput>,
) -> Result<Json<LandlordAnnualRentStatementResult>, ApiError> {
    Ok(Json(check_landlord_annual_rent_statement(&b)))
}

// ---------------------------------------------------------------------------
// landlord_emergency_entry_notice: Mandatory landlord-paid post-entry
// notice after emergency entry. Mounted at POST /api/rental/landlord-
// emergency-entry-notice. Four regimes: California Cal. Civ. Code §§
// 1954(e) + 1940.2 (emergency entry permitted without prior notice but
// landlord MUST leave written notice describing date + time + purpose +
// provide notice within reasonable time; pretextual emergency entries
// actionable as prohibited harassing acts with $2K per-violation civil
// penalty); Texas Tex. Prop. Code § 92.0081 (emergency entry permitted;
// no specific post-entry written-notice obligation but unauthorized
// entry civil penalty = actual damages + one month's rent + $1K +
// attorney fees); New York N.Y. Mult. Dwell. Law § 78 + common-law
// quiet enjoyment (emergency entry permitted but landlord must provide
// post-entry notification + leave premises SECURED); Default common-law
// necessity doctrine + quiet enjoyment covenant + trespass. Distinct
// from `entry_notice` (general 24-hour pre-entry), `pesticide_
// application_notice`, and `landlord_harassment`.
// ---------------------------------------------------------------------------

async fn landlord_emergency_entry_notice_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<LandlordEmergencyEntryInput>,
) -> Result<Json<LandlordEmergencyEntryResult>, ApiError> {
    Ok(Json(check_landlord_emergency_entry_notice(&b)))
}

// ---------------------------------------------------------------------------
// landlord_mid_tenancy_rekeying: Mandatory landlord-paid mid-tenancy
// rekeying obligations. Mounted at POST /api/rental/landlord-mid-tenancy-
// rekeying. Three regimes: Texas Tex. Prop. Code §§ 92.156-92.158 (most
// explicit framework — landlord MUST perform additional rekeying at
// tenant's request unlimited times within 7-day reasonable window;
// landlord pays for master-key changes and security upgrades; excludes
// interior doors; § 92.164 + § 92.165 remedies = actual damages +
// punitive + $500 civil penalty + one month's rent + court costs +
// attorney fees); California Cal. Civ. Code §§ 1954 + 1941.3 (limited
// framework; common-law reasonable-time obligation for tenant-requested
// rekey + § 1941.3 security device maintenance at landlord's expense);
// Default common-law quiet enjoyment + reasonable-time. Distinct from
// `lock_change_between_tenancies` (between-tenancy), `dv_survivor_lock_
// change` (DV-survivor), and `tenant_smart_lock_biometric_consent`.
// ---------------------------------------------------------------------------

async fn landlord_mid_tenancy_rekeying_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<LandlordMidTenancyRekeyingInput>,
) -> Result<Json<LandlordMidTenancyRekeyingResult>, ApiError> {
    Ok(Json(check_landlord_mid_tenancy_rekeying(&b)))
}

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
// landlord_post_eviction_tenant_property_storage_disposal: Multi-
// jurisdictional landlord post-eviction tenant property storage and
// disposal compliance framework. When a tenant vacates (voluntarily,
// by eviction, or by abandonment) and leaves personal property in or
// around the rental unit, what notice procedures, storage durations,
// valuation thresholds, and disposal pathways apply, and what failure-
// mode liabilities expose landlord to conversion + statutory damages?
// Mounted at POST /api/rental/landlord-post-eviction-tenant-property-
// storage-disposal. Four-jurisdiction framework: California (MOST
// PROCEDURALLY DETAILED — Cal. Civ. Code §§ 1980-1991 'Disposition of
// Personal Property Remaining on Premises at Termination of Tenancy':
// § 1983 written notice with 15-day personal / 18-day mail response
// window; § 1984 notice content requirements (description + location
// + response deadline + reasonable storage cost); § 1985 $700
// VALUATION THRESHOLD determining whether public sale required;
// § 1988 storage cost reimbursement; § 1989 tenant right of
// redemption upon payment + $250 minimum statutory damages plus
// actual damages plus attorney fees); Texas (Tex. Prop. Code
// § 92.0081 removal and exclusion of residential tenant + § 92.014
// — written notice required + reasonable storage period + post-
// storage sale or disposal procedure); New York (N.Y. RPAPL § 749(3)
// sheriff/marshal supervised removal REQUIRED — landlord cannot
// unilaterally remove tenant property; warrant of eviction
// authorizes sheriff physical removal; N.Y. RPL § 235-b implied
// warranty overlay; NYC Admin. Code § 26-521 unlawful eviction);
// Default (common-law conversion liability + bailee ordinary-care
// duty + tort negligence + implied warranty of habitability per
// Hilder v. St. Peter, 478 A.2d 202 (Vt. 1984) + Cal. Civ. Code
// § 1941.1). Five universal failure-mode liabilities: disposal
// without notice (conversion + § 1989 $250 minimum); notice
// deficient missing description/location/deadline; premature
// disposal before statutory window expires; sale below FMV without
// public-sale procedure on $700+ property; loss/damage during
// storage (bailee liability). Distinct from siblings tenant_
// abandonment (premises abandonment without eviction), landlord_
// self_help_eviction_prohibition (illegal lockout/property hold),
// squatter_unauthorized_occupant_removal (squatter scenario),
// abandoned_property_handling (general framework), landlord_
// retaliation_damages.
// ---------------------------------------------------------------------------

async fn landlord_post_eviction_tenant_property_storage_disposal_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<LandlordPostEvictionTenantPropertyStorageDisposalInput>,
) -> Result<Json<LandlordPostEvictionTenantPropertyStorageDisposalResult>, ApiError> {
    Ok(Json(check_landlord_post_eviction_tenant_property_storage_disposal(&b)))
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
// residential_lease_arbitration_clause: Pre-dispute arbitration clause
// enforceability in residential leases. Three regimes: California (Cal.
// Civ. Code § 1953(a)(4) traditional void rule for procedural-rights
// waivers — eroded by Brooks v. Greystar Real Estate Partners (S.D.
// Cal. 2024) FAA preemption analysis); NewJersey (N.J.S.A. 2A:23B-1 +
// Atalese v. U.S. Legal Services Group (220 N.J. 220, 2014) requires
// EXPLICIT judicial-forum-waiver language); Default (9 U.S.C. §§ 1/2/4
// Federal Arbitration Act + AT&T Mobility v. Concepcion (563 U.S. 333,
// 2011) + Epic Systems v. Lewis (584 U.S. 497, 2018) class-action
// waiver enforcement). Universal Speak Out Act of 2022 (117 Stat. 2192)
// bars pre-dispute arbitration for sexual harassment / sexual assault
// claims regardless of state regime. 9 U.S.C. § 2 savings clause
// permits unconscionability + duress / misrepresentation defenses
// (Concepcion permits state-law defenses applied neutrally). Distinct
// from lease_waiver_enforceability.
// ---------------------------------------------------------------------------

async fn residential_lease_arbitration_clause_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<ArbitrationClauseInput>,
) -> Result<Json<ArbitrationClauseResult>, ApiError> {
    Ok(Json(check_residential_arbitration(&b)))
}

// ---------------------------------------------------------------------------
// rental_junk_fee_transparency: Rental junk-fee + non-rent fee transparency
// compliance — what statutory disclosure obligations attach to fees, charges,
// or other amounts beyond base rent when a residential landlord advertises,
// leases, or renews? Mounted at POST /api/rental/rental-junk-fee-transparency.
// Four regimes: (1) Massachusetts — 940 CMR 38.00 (Unfair and Deceptive Fees,
// eff. 2025-09-02 under M.G.L. c. 93A Consumer Protection Act + TREBLE
// damages): clearly and conspicuously disclose TOTAL PRICE inclusive of all
// fees in advertising/leasing/renewals. (2) Colorado — Colo. Rev. Stat. §
// 38-12-1101 et seq. (HB25-1090 Honest Pricing Law, eff. 2026-01-01): total
// price as SINGLE NUMBER + bans utility markup above provider cost + bans
// CAM charges + caps markup fees at 2% / $10/month + bans charges for
// undelivered services + bans landlord-responsibility cost passthrough. (3)
// California — Cal. Civ. Code § 1950.5 (AB 12 non-rent security cap one
// month's rent in TPA-covered units); pending broader junk fee legislation.
// (4) Default — 16 CFR Part 464 (FTC Unfair or Deceptive Fees Rule covers
// short-term rentals + hotels); 15 U.S.C. § 45 FTC Act § 5 UDAP. Distinct
// from `application_fees` (application-stage), `late_fee_caps` (post-
// default), `pet_fees` (pet deposits), and `broker_fee_allocation` (broker
// fee party-allocation).
// ---------------------------------------------------------------------------

async fn rental_junk_fee_transparency_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalJunkFeeTransparencyInput>,
) -> Result<Json<RentalJunkFeeTransparencyResult>, ApiError> {
    Ok(Json(check_rental_junk_fee_transparency(&b)))
}

// ---------------------------------------------------------------------------
// rental_just_cause_eviction: Multi-state just-cause / good-cause
// eviction compliance covering NJ Anti-Eviction Act of 1974
// (N.J.S.A. 2A:18-61.1 — 18 enumerated grounds, no rent cap),
// CA Tenant Protection Act of 2019 (Cal. Civ. Code § 1946.2 +
// § 1947.12 — 5% + CPI rent cap, hard ceiling 10%), OR SB 608
// (ORS 90.323 + § 90.427 — 7% + CPI, 15-year new-construction
// exemption), NY Good Cause Eviction Law of 2024 (Part HH of
// L. 2024, c. 56 + RPL § 226-c — local rent standard = CPI + 5%,
// hard ceiling 10%, exempts ≤ 10-unit small landlords, post-2009
// construction, owner-occupied < 11 units, rent-stabilized/
// subsidized/condo-co-op), WA HB 2114 (RCW 59.18.650 — 7% + CPI).
// Three action types (NotApplicable, NonRenewalOrEviction,
// RentIncrease) × five jurisdictions + Default × ten-mode severity
// ladder including all four NY exemption modes + OR construction-
// age exemption + rent-cap-within/exceeds + good-cause-allowed/
// barred + default-no-regime fallthrough.
// ---------------------------------------------------------------------------

async fn rental_just_cause_eviction_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalJustCauseEvictionInput>,
) -> Result<Json<RentalJustCauseEvictionResult>, ApiError> {
    Ok(Json(check_rental_just_cause_eviction(&b)))
}

// ---------------------------------------------------------------------------
// rental_carbon_monoxide_detector: Rental property carbon monoxide
// (CO) detector compliance — when a trader-landlord must install,
// maintain, and certify CO alarms in dwelling units with fossil-
// fuel-burning appliances or attached garages. Mounted at POST
// /api/rental/rental-carbon-monoxide-detector. Four regimes:
// California SB 183 of 2010 (Cal. Health & Safety Code §§ 13260-
// 13263; State Fire Marshal-certified device required; CO alarm
// outside each sleeping area AND on every level including
// basements; single-family fossil-fuel/garage by July 1, 2011;
// multifamily by January 1, 2013; $100 statutory damages per
// violation); New York Amanda's Law (NY Exec. Law § 378(5-a),
// eff. February 22, 2010; CO alarm within 15 feet of each
// sleeping area; UL 2034 listed); Illinois Carbon Monoxide Alarm
// Detector Act (430 ILCS 135/1 et seq., eff. January 1, 2007; CO
// detector within 15 feet of every sleeping room; failure is
// Class B misdemeanor); Massachusetts Nicole's Law (M.G.L. c. 148
// § 26F½, eff. March 31, 2006; strictest — interconnected
// hardwired or wireless CO alarms on every level + within 10 feet
// of each bedroom door + certificate of compliance from local
// fire department BEFORE selling or renting + UL 2034). Distinct
// from siblings rental_bed_bug_disclosure, rental_hot_water_
// temperature, tenant_fire_safety_plan_disclosure, rental_
// bedroom_egress_window, rental_gas_appliance_ban.
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// rental_california_ab_12_security_deposit_cap: California AB 12 of
// 2023 (Haney; 2023-2024 Regular Session) — amends Cal. Civ. Code
// § 1950.5 to reduce residential security deposit cap from prior
// 2 months' rent (unfurnished) / 3 months' rent (furnished) to a
// uniform 1 month's rent regardless of furnishing status. Signed
// by Governor Gavin Newsom on October 11, 2023; effective July 1,
// 2024 (9-month transition window). Small landlord exception
// under § 1950.5(c)(4): if owner is BOTH a natural person OR an
// LLC with all natural-person members AND owns ≤ 2 residential
// rental properties collectively with ≤ 4 dwelling units, the cap
// is 2 months' rent. Service member override under § 1950.5(c)(4)(B):
// military service member tenants ALWAYS subject to strict 1-month
// cap regardless of landlord size (Cal. Mil. & Vet. Code § 400
// definition). Transition rule: pre-July 1, 2024 lawful deposits
// at prior caps remain valid; new cap activates on (1) new tenancy
// after original tenant vacates, (2) written lease renewal, (3)
// material lease modification. § 1950.5(l) tenant remedies for
// excessive deposit: recovery of excess + bad-faith retention up
// to twice security deposit as statutory damages plus actual
// damages. Eleven-mode severity ladder × four landlord entity
// types × two tenant classifications × four lease trigger events
// × two furnishing statuses. Trader-landlord critical because CA
// portfolio operators must reset security deposit collection
// practices for all new/renewed/modified leases post-July 1, 2024;
// existing high-deposit leases grandfathered but cap activates on
// renewal. Sibling cluster: rental_security_deposit_interest,
// rental_security_deposit_return_notice, rental_pet_deposit_
// separate_security, rental_last_month_rent_offset, rental_
// massachusetts_security_deposit_statute (analog state-level
// security deposit regime), military_termination (SCRA cross-
// reference for service member protection).
// ---------------------------------------------------------------------------

async fn rental_california_ab_12_security_deposit_cap_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalCaliforniaAb12SecurityDepositCapInput>,
) -> Result<Json<RentalCaliforniaAb12SecurityDepositCapResult>, ApiError> {
    Ok(Json(check_rental_california_ab_12_security_deposit_cap(&b)))
}

// ---------------------------------------------------------------------------
// rental_california_ab_2347_unlawful_detainer_response: California
// AB 2347 of 2024 (Kalra; 2023-2024 Regular Session) signed by
// Governor Gavin Newsom on September 24, 2024; effective January 1,
// 2025; amends California Code of Civil Procedure (CCP) § 1167 and
// § 1170. CCP § 1167: tenant unlawful detainer response time
// EXTENDED from 5 days to 10 COURT DAYS (excluding Saturdays,
// Sundays, and judicial holidays) to file answer, demurrer, or
// motion to strike. CCP § 1170: if tenant files demurrer or motion
// to strike, hearing must occur not less than 5 court days nor more
// than 7 court days after filing notice of motion; oral opposition
// and reply may be made at hearing rather than requiring written
// papers. Service requirements: landlord must file proof of service
// 3 days before requesting default judgment. Twelve-mode severity
// ladder × two property jurisdictions × two unlawful detainer
// action dates × four tenant response types × four landlord
// actions. Trader-landlord critical for California portfolio
// operators because the doubled response window slows eviction
// timelines and gives tenants more time to retain counsel/build
// defense; particularly impacts portfolios with high turnover unit
// counts. Sibling cluster: rental_california_ab_12_security_
// deposit_cap (iter 645 — CA companion), rental_eviction_notices,
// rental_just_cause_eviction (parallel tenant protection regime),
// rental_eviction_record_sealing, rental_landlord_notice_to_enter,
// rental_demolition_tenant_notice, rental_eviction_diversion_
// program.
// ---------------------------------------------------------------------------

async fn rental_california_ab_2347_unlawful_detainer_response_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalCaliforniaAb2347UnlawfulDetainerResponseInput>,
) -> Result<Json<RentalCaliforniaAb2347UnlawfulDetainerResponseResult>, ApiError> {
    Ok(Json(check_rental_california_ab_2347_unlawful_detainer_response(&b)))
}

// ---------------------------------------------------------------------------
// rental_california_sb_567_no_fault_eviction_amendments: California
// SB 567 of 2023 (Sen. Maria Elena Durazo, D-Los Angeles), signed
// by Governor Gavin Newsom on September 30, 2023, effective April 1,
// 2024. Amends Civil Code § 1946.2 (Tenant Protection Act enacted
// by AB 1482) to tighten the no-fault eviction grounds for owner /
// family-member move-in (OMI) and substantial remodel. OMI must
// satisfy 90-day move-in PLUS 12 continuous months of primary-
// residence occupancy; permitted family members limited to six
// enumerated categories (spouse, domestic partner, children,
// grandchildren, parents, grandparents); OMI unavailable if
// intended occupant already rents another unit on the property or
// if similar vacant unit exists on the property. Substantial
// remodel must require tenant to vacate for at least 30 consecutive
// days AND not be safely accomplishable with tenant in place;
// written notice must include description + expected duration +
// either required permit(s) OR signed contractor agreement. OMI
// failure-to-comply remedy: owner MUST offer unit to vacated tenant
// at same rent + lease terms + reimburse moving expenses. Remodel
// non-completion remedy: tenant may reclaim unit at previous rate.
// Civil penalties: actual damages + up to 3× actual damages for
// willful or malicious conduct + reasonable attorney fees + costs +
// punitive damages. Termination notice rendered VOID by non-
// compliance with any just-cause provision. Nineteen-mode severity
// ladder × two property jurisdictions × two notice-date statuses ×
// two coverage statuses × five no-fault cause types × four OMI
// actual-occupancy statuses × five OMI intended-occupant statuses ×
// five substantial-remodel compliance statuses × three reoccupancy-
// offer statuses × three willfulness statuses × variable monthly
// rent input. Trader-landlord critical because SB 567 dramatically
// increased OMI compliance cost: the 12-continuous-month requirement
// means a trader-landlord using OMI to vacate a unit for renovation,
// flip, or family relocation now incurs full-year occupancy
// commitment + civil-liability exposure for early-departure breach.
// The substantial-remodel permit/signed-contractor documentation
// requirement caught many landlords using verbal contractor
// arrangements or pending-permit-application scenarios. Multi-state
// trader-landlord portfolio operators should treat CA OMI as the
// strictest in US — by comparison, NJ Anti-Eviction Act subsection
// (k) requires only owner permanent move-in (no time threshold),
// CO HB 24-1098 owner / family-member occupancy requires only good-
// faith intent + 90-day notice + relocation assistance, Seattle JCEO
// requires 60-day continuous occupancy within 90 days of vacate.
// Sibling cluster: rental_california_ab_12_security_deposit_cap
// (iter 645 — CA companion AB 12 security deposit cap 2024),
// rental_california_ab_2347_unlawful_detainer_response (iter 667 —
// CA companion unlawful detainer response time), rental_just_cause_
// eviction (multi-state base regime — CA AB 1482 + SB 567 = strictest
// US OMI regime), rental_owner_move_in_eviction (multi-state OMI
// regime), rental_demolition_tenant_notice (substantial-remodel /
// demolition cross-reference), rental_tenant_relocation_assistance
// (relocation assistance regime), rental_colorado_hb_24_1098_just_
// cause_eviction (iter 649 — CO companion), rental_seattle_smc_22_
// 206_160_just_cause_eviction (iter 669 — Seattle companion OMI
// 60-day occupancy + 90-day deadline + $2,000 OMI damages), rental_
// new_jersey_anti_eviction_act (iter 651 — NJ companion).
// ---------------------------------------------------------------------------

async fn rental_california_sb_567_no_fault_eviction_amendments_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalCaliforniaSb567NoFaultEvictionAmendmentsInput>,
) -> Result<Json<RentalCaliforniaSb567NoFaultEvictionAmendmentsResult>, ApiError> {
    Ok(Json(check_rental_california_sb_567_no_fault_eviction_amendments(&b)))
}

async fn rental_carbon_monoxide_detector_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalCarbonMonoxideDetectorInput>,
) -> Result<Json<RentalCarbonMonoxideDetectorResult>, ApiError> {
    Ok(Json(check_rental_carbon_monoxide_detector(&b)))
}

// ---------------------------------------------------------------------------
// rental_chimney_fireplace_inspection_disclosure: Multi-jurisdictional
// rental property CHIMNEY + FIREPLACE + SOLID-FUEL-BURNING APPLIANCE
// inspection and disclosure compliance framework. When a landlord rents
// a property with a wood-burning fireplace, pellet stove, wood stove,
// vented gas fireplace, or chimney-vented oil furnace, what inspection
// schedule applies, what disclosure must be given to tenant, and what
// failure-mode liabilities expose landlord after a chimney fire or
// carbon monoxide event? Mounted at POST /api/rental/rental-chimney-
// fireplace-inspection-disclosure. Three-jurisdiction framework: Maine
// (MOST STRINGENT for rental disclosure — State of Maine Professional
// Financial Regulation Form 2079 chimney/fireplace construction
// disclosure required at sale/rental + Level II NFPA 211 inspection at
// title transfer per state fire code + 14 M.R.S. § 6021 implied
// warranty of habitation); Connecticut (adopts NFPA 211 by reference
// in Conn. Gen. Stat. § 29-292 State Fire Safety Code + Conn. Public
// Health Code § 19-13-B105(o) combustion-air/venting + Conn. Gen.
// Stat. § 47a-7 landlord duties + annual NFPA 211 inspection
// recommended); Default (NFPA 211 VOLUNTARY national standard
// recommending annual inspection enforceable as law only when adopted
// by local code + CSIA Chimney Safety Institute of America
// certification + common-law implied warranty of habitability per
// Hilder v. St. Peter, 478 A.2d 202 (Vt. 1984) + Green v. Superior
// Court, 10 Cal. 3d 616 (1974) + Cal. Civ. Code § 1941.1 implied
// warranty of habitation). NFPA 211 three-level inspection framework:
// Level I basic visual when no changes; Level II comprehensive
// accessible inspection required on OCCUPANCY CHANGE (rental
// transition + sale) or fuel change or damage event or weather event;
// Level III invasive when concealed defect suspected. NFPA chimney
// fire data: ~22,000 residential structure fires annually in US,
// $150M+ property damage; CO events from fireplace/chimney
// malfunction kill 50-150 Americans annually. Five universal
// failure-mode liabilities: creosote buildup beyond Stage 3 (Class A
// fire risk); cracked flue liner (CO release cross-references rental_
// carbon_monoxide_detector); damaged crown/spalling (water intrusion
// cross-references rental_basement_water_intrusion_disclosure); animal
// nesting in flue; improper combustion. Six appliance types modeled:
// None, WoodBurningFireplace, PelletStove, WoodStove,
// GasFireplaceVented, OilFurnaceChimneyVented. Distinct from siblings
// rental_carbon_monoxide_detector, rental_basement_water_intrusion_
// disclosure, rental_gas_appliance_ban, tenant_emotional_distress_
// damages, mid_tenancy_temporary_relocation.
// ---------------------------------------------------------------------------

async fn rental_chimney_fireplace_inspection_disclosure_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalChimneyFireplaceInspectionDisclosureInput>,
) -> Result<Json<RentalChimneyFireplaceInspectionDisclosureResult>, ApiError> {
    Ok(Json(check_rental_chimney_fireplace_inspection_disclosure(&b)))
}

// ---------------------------------------------------------------------------
// rental_climate_mobilization_act_ll97_emissions: NYC Local Law 97
// of 2019 (Climate Mobilization Act) building greenhouse-gas
// emissions cap compliance. NYC Admin Code § 28-320 et seq.
// Effective 2024 with progressive tightening through 2050 (40%
// reduction by 2030, 80% by 2050). Applies to buildings > 25,000
// sqft. Emissions intensity limits by occupancy: R-2 multifamily
// 6.75 kgCO2e/sqft (2024-2029) → 4.07 (2030-2034); B business
// office 8.46 → 4.53. Penalty $268 per metric ton CO2e annual
// excess. Article 321 alternative compliance for buildings with
// > 35% rent-regulated units: 13 prescriptive ECMs in lieu of
// emissions-limit compliance. Adjustments: financial hardship
// (§ 28-320.7), critical facility (§ 28-320.8). Sibling cluster:
// rental_energy_benchmarking (NYC LL 84 — predicate energy
// reporting), rental_facade_inspection_fisp_local_law_11 (iter 583
// — NYC LL 11 facade), rental_gas_piping_inspection_local_law_152
// (iter 585 — NYC LL 152 gas), rental_gas_appliance_ban (all-
// electric mandate as compliance pathway).
// ---------------------------------------------------------------------------

async fn rental_climate_mobilization_act_ll97_emissions_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalClimateMobilizationActLl97EmissionsInput>,
) -> Result<Json<RentalClimateMobilizationActLl97EmissionsResult>, ApiError> {
    Ok(Json(check_rental_climate_mobilization_act_ll97_emissions(&b)))
}

// ---------------------------------------------------------------------------
// rental_colorado_hb_24_1098_just_cause_eviction: Colorado HB 24-1098
// of 2024 — first-ever statewide just-cause eviction law in Colorado
// history. Signed by Governor Jared Polis on April 19, 2024;
// effective immediately due to legislative safety clause. Codified
// at Colo. Rev. Stat. § 38-12-1301 et seq. For-cause eviction
// grounds (no notice/relocation): non-payment of rent, material
// lease violations, substantial property damage, criminal activity,
// non-curable lease violations. No-fault eviction grounds (90-day
// notice + relocation assistance): demolition or conversion;
// substantial repairs or renovations; owner or family-member
// occupancy assumption; withdrawal from rental market for sale;
// tenant refused new lease with reasonable terms; tenant history
// of nonpayment of rent. 90-day written notice required for any
// no-fault eviction. Relocation assistance: 2 months' rent baseline
// + 1 additional month (3 months total) if any resident is under
// 18, at least 60, household income ≤ 80 % AMI, or disabled
// individual. Exemptions: short-term rental properties; owner-
// occupied units (typically 4 or fewer with owner-occupied);
// employer-provided housing; tenants residing < 12 months AND
// unknown to landlord. Tenant remedies: existing unlawful-removal
// statutes + affirmative defense to eviction proceeding. Twelve-
// mode severity ladder × twelve eviction grounds × six property
// exemption statuses × five vulnerable resident statuses × four
// notice categories. Trader-landlord critical because Colorado is
// the SECOND state to enact statewide just-cause eviction (after
// California AB 1482); HB 24-1098 imposes substantial relocation-
// assistance exposure for no-fault evictions ($5,000-$15,000 per
// unit at typical Colorado rents). Sibling cluster: rental_just_
// cause_eviction (multi-state base regime — Colorado = newest
// entry), rental_owner_move_in_eviction (owner move-in cross-
// reference), rental_tenant_relocation_assistance (general state-
// level relocation assistance regime), rental_demolition_tenant_
// notice (demolition cross-reference), rental_oregon_sb_608_sb_611_
// rent_stabilization (Oregon companion just-cause framework — iter
// 647), rental_eviction_diversion_program (parallel eviction
// regulation regime).
// ---------------------------------------------------------------------------

async fn rental_colorado_hb_24_1098_just_cause_eviction_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalColoradoHb241098JustCauseEvictionInput>,
) -> Result<Json<RentalColoradoHb241098JustCauseEvictionResult>, ApiError> {
    Ok(Json(check_rental_colorado_hb_24_1098_just_cause_eviction(&b)))
}

// ---------------------------------------------------------------------------
// rental_connecticut_fair_rent_commission: Connecticut Public Act
// 22-30 (2022) Fair Rent Commission regime under C.G.S. §§ 7-148b
// and 7-148c. PA 22-30 required every town/city/borough with
// population of 25,000+ per most recent decennial census to adopt
// ordinance creating Fair Rent Commission by July 1, 2023; ~83 % of
// Connecticut residents now have access to a Fair Rent Commission.
// C.G.S. § 7-148b creation/powers + 30-day Commissioner of Housing
// notification requirement. C.G.S. § 7-148c — 13 excessive rent
// factors: rent increase size, premises condition, operating costs,
// services included, tenant income, comparable rents, service
// decrease, expense increase, rental history, property age/condition,
// capital improvements, real estate taxes, other relevant factors.
// Seasonal basis exemption ≤ 120 days/year. Rental charge definition
// includes any fee/charge beyond rent (parking, amenity, junk fees).
// Commission powers: investigate, adjudicate excessive rent
// complaints, order rent reductions, stay evictions for retaliation,
// subpoena testimony. C.G.S. § 47a-20 retaliation prohibition within
// 6 months of tenant complaint. Twelve-mode severity ladder × four
// municipality classifications × four commission statuses × two
// rental arrangements × five landlord actions. Trader-landlord
// critical for CT operators in Bridgeport, Hartford, New Haven,
// Waterbury, Stamford, Norwalk, Danbury, New Britain, West Hartford,
// Greenwich and other large municipalities subject to Fair Rent
// Commission jurisdiction. Sibling cluster: rental_rent_control_
// stabilization (multi-state rent regulation regime), rental_just_
// cause_eviction (parallel tenant protection overlay), rental_
// retaliation_prohibition (state-law DV/retaliation analog), rental_
// rent_increase_notice_requirement, rental_oregon_sb_608_sb_611_rent_
// stabilization (iter 647 — OR companion).
// ---------------------------------------------------------------------------

async fn rental_connecticut_fair_rent_commission_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalConnecticutFairRentCommissionInput>,
) -> Result<Json<RentalConnecticutFairRentCommissionResult>, ApiError> {
    Ok(Json(check_rental_connecticut_fair_rent_commission(&b)))
}

// ---------------------------------------------------------------------------
// rental_cook_county_rtlo: Cook County (Illinois) Residential Tenant
// and Landlord Ordinance (RTLO) compliance module. Adopted by the
// Cook County Board of Commissioners on January 28, 2021; anti-
// lockout provisions effective immediately January 2021; full
// ordinance effective June 1, 2021. Codified at Cook County Code of
// Ordinances Chapter 42 Article XII. Covers approximately 245,000
// suburban Cook County rental units across 130+ municipalities. The
// City of Chicago is EXCLUDED from RTLO scope — Chicago has its own
// CRLTO codified at MCC Chapter 5-12 (1986). Exemptions (except
// anti-lockout, which applies universally): (1) owner-occupied
// buildings with 6 or fewer units; (2) single-family / condominium
// where owner rents only that property AND lived there within past
// 12 months; (3) SRO housing for vulnerable residents; (4) hotel /
// motel monthly rental under 32 days; (5) school dormitories;
// (6) shelters; (7) employee quarters; (8) non-residential
// properties; (9) owner-occupied cooperatives. Security deposit
// cap 1.5 × monthly rent + 30-day return + itemized deductions
// requirement; statutory damages = greater of (2 × violation
// amount) or one month's rent. Late fee cap = $10 for first $1000
// of monthly rent + 5 % of any excess. 10-day material noncompliance
// cure notice; 60-day lease renewal notice for material term changes
// (reduced from 90 days during drafting); 2-day entry notice with
// 8 a.m. to 8 p.m. window for repairs / showings within 60 days of
// lease ending. Anti-lockout provision applies to ALL units
// regardless of exemption status (universal coverage). Retaliation
// prohibition with rebuttable presumption when adverse landlord
// action follows tenant protected activity. Enforcement via Cook
// County Commission on Human Rights administrative complaint
// process + private right of action + attorney fees. Twenty-seven-
// mode severity ladder × ten exemption statuses × seven compliance
// aspects × two anti-lockout statuses × two retaliation statuses ×
// variable monetary inputs. Trader-landlord critical because
// suburban Cook County trader inventory (~245K units) had NO
// county-level tenant protections prior to RTLO; many out-of-state
// operators continue assuming Illinois landlord-tenant law alone
// applies, missing the RTLO security deposit cap, late fee cap,
// and cure-notice procedural requirements. The Chicago / suburban
// Cook County boundary is the single most-misunderstood scope
// question — many landlords incorrectly apply Chicago CRLTO rules
// to suburban properties or vice versa. Sibling cluster:
// rental_just_cause_eviction (multi-state base regime), rental_
// security_deposit_interest (Illinois state-level interest
// requirement under 765 ILCS 715), rental_late_fee_cap (multi-state
// late-fee cap regime), rental_lockout_prohibition (anti-self-help
// regime), rental_retaliation_prohibition (multi-state retaliation
// regime), rental_landlord_notice_to_enter (general entry notice
// regime — Cook County RTLO 2-day rule is one of stricter in US),
// rental_eviction_notices (notice to quit regime — Illinois 5-day
// pay-or-quit + 10-day cure under 735 ILCS 5/9-209/210/211).
// ---------------------------------------------------------------------------

async fn rental_cook_county_rtlo_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalCookCountyRtloInput>,
) -> Result<Json<RentalCookCountyRtloResult>, ApiError> {
    Ok(Json(check_rental_cook_county_rtlo(&b)))
}

// ---------------------------------------------------------------------------
// rental_cooling_tower_inspection_local_law_77: NYC Local Law 77 of
// 2015 + NYC Admin Code § 17-194.1 + Chapter 8 of Rules of City of
// New York + NYS 10 NYCRR Subpart 4-1 cooling tower registration,
// inspection, and Legionella testing compliance. Enacted in response
// to South Bronx Legionnaires' outbreak (July-August 2015) — 8
// deaths, 138 cases. Quarterly 90-day inspection by Qualified Person
// (NY State PE/RA + certified water technologist + environmental
// consultant 2+ years experience). Legionella culture + heterotrophic
// plate count testing March/June/September/December. Lab results to
// DOHMH within 5 days. Annual certification by November 1. NYC Local
// Law 159 of 2024: monthly Legionella testing effective May 7, 2026.
// Penalty schedule: first violation $2,000, subsequent $5,000,
// fatality or serious injury $10,000; no MPP $1,000; incomplete or
// not on-site $500; late annual certification up to $10,000. Sibling
// cluster: rental_facade_inspection_fisp_local_law_11 (iter 583),
// rental_gas_piping_inspection_local_law_152 (iter 585), rental_
// climate_mobilization_act_ll97_emissions (iter 587 LL 97 emissions),
// rental_natural_gas_leak_response.
// ---------------------------------------------------------------------------

async fn rental_cooling_tower_inspection_local_law_77_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalCoolingTowerInspectionLocalLaw77Input>,
) -> Result<Json<RentalCoolingTowerInspectionLocalLaw77Result>, ApiError> {
    Ok(Json(check_rental_cooling_tower_inspection_local_law_77(&b)))
}

// ---------------------------------------------------------------------------
// rental_elevator_safety_inspection: Rental property elevator safety
// inspection compliance — when a trader-landlord operating a
// multifamily building with elevators must comply with state-specific
// periodic inspection, testing, and certification requirements
// grounded in ASME A17.1 Safety Code for Elevators and Escalators.
// Mounted at POST /api/rental/rental-elevator-safety-inspection.
// Three regimes: California Cal. Labor Code §§ 7300-7324.2 + § 7317
// (Cal/OSHA-certified inspector with 4 years experience) + § 7320
// (Form 80 permit-to-operate must be posted; $200/day penalty) +
// Title 8 Subchapter 6 Elevator Safety Orders + annual inspection;
// New York City NYC Admin Code § 28-304 + § 28-304.6.1 (DOB-
// approved elevator agency) + § 28-304.6.5 ($3,000-$10,000 per
// violation civil penalty) + NYC Building Code Chapter 30 +
// Appendix K Chapter K1 (Category 1 PCT annual + Category 3
// hydraulic 3-year + Category 5 full 5-year + PVT-A Form filed
// with DOB within 60 days); Default ASME A17.1-2025 + ANSI/ASME
// A17.1/CSA B44-2025 (Table N1 inspection schedule + QEI Qualified
// Elevator Inspector certification). Distinct from siblings
// rental_swimming_pool_drain_safety (VGB Act), rental_carbon_
// monoxide_detector, rental_bedroom_egress_window, soft_story_
// seismic_retrofit.
// ---------------------------------------------------------------------------

async fn rental_elevator_safety_inspection_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalElevatorSafetyInspectionInput>,
) -> Result<Json<RentalElevatorSafetyInspectionResult>, ApiError> {
    Ok(Json(check_rental_elevator_safety_inspection(&b)))
}

// ---------------------------------------------------------------------------
// rental_fire_extinguisher_requirement: Multi-jurisdictional rental
// property FIRE EXTINGUISHER installation, inspection, and maintenance
// compliance framework. When must a landlord provide portable fire
// extinguishers to tenants, what travel-distance and visibility
// standards apply, what inspection cycle is required, and what
// failure-mode liabilities expose landlord after a fire event?
// Mounted at POST /api/rental/rental-fire-extinguisher-requirement.
// Four-jurisdiction framework: Texas (MOST SPECIFIC landlord duty —
// Tex. Prop. Code § 92.252 + § 92.255 require inspection at occupancy
// and within reasonable time after tenant written request; repair/
// replace duty for non-functional + incorrect pressure + tenant-used
// units of any landlord-provided 1A10BC residential extinguisher);
// Massachusetts (527 C.M.R. 1.00 Comprehensive Fire Safety Code
// adopting NFPA 10 by reference + M.G.L. c. 148 § 26G + § 28A;
// multifamily 3+ unit common-area extinguishers per local fire-
// marshal authority); New Jersey (N.J.A.C. 5:70-3 NJ Uniform Fire
// Code adopting NFPA 10; landlords not legally required to provide
// extinguishers absent lease contract or local ordinance but any
// installed unit must be maintained per NFPA 10; N.J.S.A. 46:8-39
// to -50 hotel/multiple-dwelling registration); Default (NFPA 10
// voluntary national standard adopted by reference in most state
// fire codes + common-law implied warranty of habitability per
// Hilder v. St. Peter, 478 A.2d 202 (Vt. 1984) + Cal. Civ. Code
// § 1941.1). NFPA 10 five-cycle inspection framework: monthly visual
// inspection; annual maintenance by certified technician; six-year
// internal examination for dry-chemical units; twelve-year hydrostatic
// test for pressure-vessel integrity; recharge after every use
// including partial discharge (Tex. Prop. Code § 92.255 explicit).
// NFPA 10 placement: Class A max 75-ft travel distance; Class B 30
// or 50 ft; mounting height max 5 ft (handle from floor) up to 40
// lbs; max 3 ft 6 in over 40 lbs. Five universal failure-mode
// liabilities: non-functional extinguisher during fire; no
// extinguisher in required multifamily; tenant-used unit not
// recharged (§ 92.255 violation); tenant complaint ignored; fire
// injury event. Distinct from siblings rental_chimney_fireplace_
// inspection_disclosure (iter 471), rental_carbon_monoxide_detector,
// tenant_fire_safety_plan_disclosure, rental_bedroom_egress_window,
// rental_window_blind_cord_safety (iter 469), rental_swimming_pool_
// drain_safety, landlord_retaliation_damages, tenant_emotional_
// distress_damages.

async fn rental_fire_extinguisher_requirement_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalFireExtinguisherRequirementInput>,
) -> Result<Json<RentalFireExtinguisherRequirementResult>, ApiError> {
    Ok(Json(check_rental_fire_extinguisher_requirement(&b)))
}

// rental_flood_hazard_disclosure: multi-jurisdictional residential rental
// flood hazard disclosure framework — climate-era statutory disclosure
// regime added across multiple states between 2018 and 2024. Cal. Gov.
// Code § 8589.45 (AB 646 of 2017, effective July 1, 2018) — actual-
// knowledge disclosure of FEMA Special Flood Hazard Area OR Cal OES area
// of potential flooding + MyHazards URL + renter's/flood insurance
// recommendation + 8-point minimum type. Tex. Prop. Code § 92.0135 (HB
// 531 of 2021, effective January 1, 2022) — 100-year floodplain notice
// (unless elevation above per federal regulations) + 5-year prior
// flooding damage knowledge; tenant remedy: TERMINATE LEASE within 30
// days of substantial loss (50%+ personal property value). N.J.S.A.
// 46:8-50 et seq. (NJ Flood Risk Notification Law, effective March 20,
// 2024) — landlords of commercial OR 3+ residential units (or 4+ where
// owner-occupied) must disclose FEMA 100-year OR 500-year floodplain;
// SEPARATE RIDER + 12-point minimum + signed/acknowledged by tenant;
// tenant remedy: lease termination + statutory damages + attorney fees.
// Default — no statewide regime; common-law fraudulent concealment +
// implied warranty of habitability available. Mounted at POST /api/
// rental/rental-flood-hazard-disclosure. Trader-landlord critical
// because waterfront / coastal / floodplain-zone investment properties
// (common in trader real-estate portfolios) trigger mandatory written
// pre-lease disclosure; failure routes directly to tenant LEASE
// TERMINATION right plus statutory damages exposure. Sibling cluster:
// rental_basement_water_intrusion_disclosure (subsurface water),
// rental_sinkhole_disclosure (FL specific), rental_property_registration,
// tenant_in_foreclosure_protection.
// ---------------------------------------------------------------------------

async fn rental_flood_hazard_disclosure_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalFloodHazardDisclosureInput>,
) -> Result<Json<RentalFloodHazardDisclosureResult>, ApiError> {
    Ok(Json(check_rental_flood_hazard_disclosure(&b)))
}

// ---------------------------------------------------------------------------
// rental_florida_hb_1417_state_preemption: Florida HB 1417 of 2023
// (CS/HB 1417 — Residential Tenancies) statewide preemption of local
// landlord-tenant ordinances. Signed by Governor Ron DeSantis;
// effective July 1, 2023; created Fla. Stat. § 83.425; amended
// Chapter 83 Part II (Florida Residential Landlord and Tenant Act).
// § 83.425 preempts to STATE regulation of residential tenancies,
// landlord-tenant relationship, and all matters covered under
// Chapter 83 Part II; expressly supersedes local government
// regulations; renders all existing local ordinances null and void.
// Affected 46 tenant protection ordinances spanning 35 cities/counties
// including Miami-Dade, Broward, Orange, Hillsborough, Pinellas
// counties. Local rules no longer permissible: rent notices, Section
// 8 housing voucher acceptance mandates, source-of-income protections,
// tenants' bill of rights ordinances, rent stabilization measures,
// eviction sealing, late fee caps, summons-process modifications.
// Fla. Stat. § 83.57(3) month-to-month termination notice increased
// from 15 to 30 days. Fla. Stat. § 83.575 end-of-term termination
// notice revised range: not less than 30 days or more than 60 days
// (from prior "not more than 60 days"). Nine-mode severity ladder ×
// two property jurisdictions × ten local ordinance categories × five
// tenancy actions. Trader-landlord critical because FL portfolio
// operators previously faced patchwork of local rules now uniformly
// preempted; reduces compliance burden but eliminates tenant-friendly
// local protections. Multi-state portfolio operators tracking FL HB
// 1417 alongside CO HB 24-1098 (iter 649), CA AB 1482, OR SB 608/611
// (iter 647), WA HB 1217 (iter 643), NJ Anti-Eviction Act (iter
// 651) just-cause/preemption framework comparisons. Sibling cluster:
// rental_just_cause_eviction (multi-state regime; FL has none post-
// preemption), rental_rent_control_stabilization (multi-state regime;
// FL ordinances preempted), rental_source_of_income_discrimination
// (FL Section 8 voucher acceptance preempted), rental_late_fee_caps
// (FL local late fee caps preempted), rental_eviction_record_sealing
// (FL local eviction sealing preempted), rental_eviction_notices,
// rental_demolition_tenant_notice.
// ---------------------------------------------------------------------------

async fn rental_florida_hb_1417_state_preemption_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalFloridaHb1417StatePreemptionInput>,
) -> Result<Json<RentalFloridaHb1417StatePreemptionResult>, ApiError> {
    Ok(Json(check_rental_florida_hb_1417_state_preemption(&b)))
}

// ---------------------------------------------------------------------------
// rental_foreclosure_tenant_protection_ptfa: Federal Protecting Tenants
// at Foreclosure Act (PTFA) compliance for trader-landlords acquiring
// foreclosed rental property as immediate successor in interest.
// Originally Pub. L. 111-22 Title VII §§ 701-704 (2009) with December
// 31, 2014 sunset; reinstated PERMANENTLY by Pub. L. 115-174 § 304
// (Economic Growth, Regulatory Relief, and Consumer Protection Act of
// 2018, effective June 23, 2018). Three bona fide tenant prongs per
// § 702(b): not mortgagor/spouse/parent/child + arm's-length lease
// + rent not substantially less than fair market rent (or subsidized).
// 90-day minimum vacate notice for month-to-month bona fide tenants;
// full lease term remaining for bona fide written-lease tenants; only
// carveout is sale to primary-residence purchaser (owner-occupy
// exception) which still requires 90-day notice. Sibling: existing
// foreclosure-protection comments referenced in flood-hazard module.
// ---------------------------------------------------------------------------

async fn rental_foreclosure_tenant_protection_ptfa_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalForeclosureTenantProtectionPtfaInput>,
) -> Result<Json<RentalForeclosureTenantProtectionPtfaResult>, ApiError> {
    Ok(Json(check_rental_foreclosure_tenant_protection_ptfa(&b)))
}

// ---------------------------------------------------------------------------
// rental_broadband_mte_rules: FCC Multiple Tenant Environment (MTE)
// broadband-access rules — when may a trader-landlord lawfully enter a
// contract with ISP granting exclusive access + revenue sharing +
// bulk billing arrangements for residential or commercial multifamily
// buildings? Mounted at POST /api/rental/rental-broadband-mte-rules.
// Two regimes: Federal FCC (47 CFR § 64.2500-64.2503 + FCC Open
// Internet Order FCC 24-52 July 2024 + FCC 22-12 MTE R&O + FCC 10-49
// 2010 bulk billing confirmation — exclusive access contracts
// categorically PROHIBITED for telecom + cable + broadband providers;
// exclusive revenue-sharing agreements PROHIBITED; graduated
// revenue-sharing agreements for broadband-only providers PROHIBITED
// under 2024 Open Internet Order; flat licensing fees PERMITTED; bulk
// billing arrangements REMAIN PERMITTED — Chairman Carr withdrew 2024
// proposed bulk-billing ban January 2025); Default (common-law lease
// analysis + state-specific broadband disclosure CA SB 1130 + NY
// HSTPA 2019). Distinct from siblings tenant_data_privacy, tenant_
// organizing, landlord_identification_disclosure, lease_disclosures.
// ---------------------------------------------------------------------------

async fn rental_broadband_mte_rules_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalBroadbandMteRulesInput>,
) -> Result<Json<RentalBroadbandMteRulesResult>, ApiError> {
    Ok(Json(check_rental_broadband_mte_rules(&b)))
}

// ---------------------------------------------------------------------------
// rental_energy_benchmarking: Rental property energy benchmarking +
// GHG emissions disclosure compliance — when a trader-landlord owning
// a large multifamily building must annually report energy/water
// consumption AND comply with carbon emissions caps. Mounted at POST
// /api/rental/rental-energy-benchmarking. Three regimes: NYC Local
// Law 84 of 2009 + Local Law 97 of 2019 (Climate Mobilization Act;
// LL84 buildings > 25,000 sq ft or groups > 100,000 sq ft aggregate
// must report via ENERGY STAR Portfolio Manager by May 1st; LL84
// penalties $500 missed + $500/quarter additional + up to $2,000/year
// max; LL97 covered buildings report annual GHG to NYC DOB; $268/
// metric ton CO2e exceedance penalty); Boston BERDO 2.0 (35,000 sq ft
// threshold; net-zero by 2050; $300/metric ton CO2e above limits);
// Default (no federal mandate; verify CA AB 802 50,000 sq ft +
// Seattle BAEDO + DC GBES + Chicago Energy Benchmarking +
// Minneapolis). Distinct from siblings rental_property_registration,
// rental_gas_appliance_ban, landlord_annual_rent_statement.
// ---------------------------------------------------------------------------

async fn rental_energy_benchmarking_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalEnergyBenchmarkingInput>,
) -> Result<Json<RentalEnergyBenchmarkingResult>, ApiError> {
    Ok(Json(check_rental_energy_benchmarking(&b)))
}

// ---------------------------------------------------------------------------
// rental_application_denial_disclosure: Rental application denial
// reason written disclosure compliance — when a trader-landlord
// rejects a tenant applicant, what statutory written-disclosure
// obligations attach? Mounted at POST /api/rental/rental-application-
// denial-disclosure. Four regimes: California Cal. Civ. Code §§ 1950.6
// + 1786.40 ICRAA (written notice + specific reason required when
// credit score / history was reason for denial); New Jersey N.J.S.A.
// 46:8-52 et seq. NJ Fair Chance in Housing Act (pre-fee criminal
// history disclosure + rehabilitation evidence right + individualized
// assessment required before criminal-background-based denial); New
// York City Local Law 24 of 2023 NYC Fair Chance for Housing Law
// (eff. Jan 1 2025; conditional offer + lookback + individualized
// assessment framework + written notice with specific reason and
// appeal rights) + FARE Act (eff. Jun 11 2025; broker fees prohibited
// from being charged to tenants); Default federal FCRA § 615(a) 15
// USC § 1681m (adverse action notice required when consumer report
// was basis for denial; CRA contact info + dispute right + free-copy
// right). Distinct from siblings adverse_action_notice (FCRA-only),
// fair_chance_housing, application_fees, credit_check_authorization.
// ---------------------------------------------------------------------------

async fn rental_application_denial_disclosure_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalApplicationDenialDisclosureInput>,
) -> Result<Json<RentalApplicationDenialDisclosureResult>, ApiError> {
    Ok(Json(check_rental_application_denial_disclosure(&b)))
}

// ---------------------------------------------------------------------------
// rental_asbestos_disclosure: multi-state residential asbestos
// disclosure compliance. California Connelly Act (AB 3713 of 1988;
// Cal H&S Code §§ 25915-25919.7) requires owner of pre-1979
// building with KNOWN asbestos-containing materials and > 10
// employees to provide notice to employees AND occupants (AB 1992
// of 1990 extension); notice must include ACM survey + specific
// locations + handling procedures. Federal AHERA (15 USC
// §§ 2641-2656, 1986) applies only to K-12 schools — NOT
// residential. Federal NESHAP (40 CFR Part 61 Subpart M) demolition/
// renovation friable asbestos. Federal OSHA 29 CFR 1910.1001
// (general industry) + 29 CFR 1926.1101 (construction) worker
// protection. Cal/OSHA Title 8 § 1529. Massachusetts G.L. c. 149
// § 6F + § 6F-1/2. New York Industrial Code Rule 56. Common law
// landlord disclosure duty + warranty of habitability requires
// disclosure of known material defects including asbestos.
// Eleven-mode severity ladder × four jurisdictions × three
// building age categories × four owner knowledge states. Trader-
// landlord critical for pre-1981 housing stock common across
// portfolios; non-disclosure exposure under fraud +
// misrepresentation framework substantial.
// ---------------------------------------------------------------------------

async fn rental_asbestos_disclosure_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalAsbestosDisclosureInput>,
) -> Result<Json<RentalAsbestosDisclosureResult>, ApiError> {
    Ok(Json(check_rental_asbestos_disclosure(&b)))
}

// ---------------------------------------------------------------------------
// rental_basement_water_intrusion_disclosure: Rental property
// basement water intrusion / mold disclosure compliance — when a
// trader-landlord must disclose water intrusion, flood history,
// visible mold, and remediation history to prospective and existing
// tenants. Mounted at POST /api/rental/rental-basement-water-
// intrusion-disclosure. Four regimes: Maryland Tenant Mold
// Protection Act (eff. July 1, 2025) + MD Real Property § 8-211 +
// § 8-211.1 (pre-move-in written disclosure + mold information
// pamphlet + 15-day mold assessment + 45-day remediation
// completion); Virginia Va. Code § 55.1-1220 + § 8.01-226.12 +
// § 55.1-1216 (5-day move-in inspection report with visible mold
// disclosure + 5-day landlord remediation after tenant election to
// stay); New York Property Condition Disclosure Act + NY GOL §
// 5-905 (natural flood event history + PCDS mold disclosure) + NYC
// Local Law 55 of 2018 + NYC Admin Code § 27-2017 (multi-unit
// buildings annual inspection + indoor allergen hazard reduction
// protocols); Default common-law warranty of habitability + EPA
// Mold Remediation Guidance (EPA 402-K-01-001) + CDC Stachybotrys
// Information for Clinicians + federal Fair Housing Act + ADA
// disability protections. Distinct from siblings mold_disclosure,
// flood_disclosure, rental_bedroom_egress_window, rental_hot_water_
// temperature.
// ---------------------------------------------------------------------------

async fn rental_basement_water_intrusion_disclosure_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalBasementWaterIntrusionDisclosureInput>,
) -> Result<Json<RentalBasementWaterIntrusionDisclosureResult>, ApiError> {
    Ok(Json(check_rental_basement_water_intrusion_disclosure(&b)))
}

// ---------------------------------------------------------------------------
// rental_bed_bug_disclosure: Rental property bed bug infestation
// pre-lease disclosure compliance — when a trader-landlord must
// disclose bed bug infestation history to prospective tenants
// before lease signing. Mounted at POST /api/rental/rental-bed-bug-
// disclosure. Four regimes: California Cal. Civ. Code § 1954.603
// (AB 551 of 2015 — written notice in 10-point type describing
// bed bug appearance/behavior/lifecycle to prospective tenants
// before initiating new tenancy; landlord may NOT show, rent, or
// lease vacant unit known to have current infestation; 2-business-
// day inspection finding notification); New York City NYC
// Multiple Dwelling Law § 27-2018.1 (annual building-wide bed bug
// report Form RA-89 filed with HPD between December 1 and
// December 31; prior year's filing to every new tenant; $250 civil
// penalty for failure to file); Arizona A.R.S. § 33-1319 (tenant-
// request-only ADHS educational materials disclosure; no proactive
// pre-lease disclosure); Maine 14 M.R.S. § 6021-A (strictest
// single-unit rule — pre-rental disclosure of infestation in unit
// OR adjacent unit within prior 12 months; 5-day inspection
// deadline; 24-hour inspection-result disclosure deadline; $250 to
// $1,500 per-violation civil penalty). Distinct from siblings
// rental_application_denial_disclosure, rental_hot_water_
// temperature, tenant_fire_safety_plan_disclosure, rental_bedroom_
// egress_window.
// ---------------------------------------------------------------------------

async fn rental_bed_bug_disclosure_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalBedBugDisclosureInput>,
) -> Result<Json<RentalBedBugDisclosureResult>, ApiError> {
    Ok(Json(check_rental_bed_bug_disclosure(&b)))
}

// ---------------------------------------------------------------------------
// rental_bedroom_egress_window: Rental property bedroom egress window
// requirement compliance — when a trader-landlord must ensure every
// bedroom has an emergency escape and rescue opening (EERO) meeting
// IRC § R310 minimum standards. Mounted at POST /api/rental/rental-
// bedroom-egress-window. Two regimes: IRC R310 (49 state-adopting
// jurisdictions including California Residential Code § R310 + 2020
// New York State Residential Code § R310 — four simultaneous
// requirements per § R310.2.1: net clear opening ≥ 5.7 sq ft (5.0 sq
// ft grade exception), height ≥ 24 in, width ≥ 20 in, sill ≤ 44 in;
// § R310.2.3 window well requirements if below grade: ≥ 9 sq ft area
// + 36 in projection + ladder if depth > 44 in); Default (no
// statewide IRC adoption; local building code + common-law warranty
// of habitability govern). Distinct from siblings detector_
// requirements, fire_sprinkler_disclosure, tenant_fire_safety_plan_
// disclosure, window_guard_requirements.
// ---------------------------------------------------------------------------

async fn rental_bedroom_egress_window_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalBedroomEgressWindowInput>,
) -> Result<Json<RentalBedroomEgressWindowResult>, ApiError> {
    Ok(Json(check_rental_bedroom_egress_window(&b)))
}

// ---------------------------------------------------------------------------
// rental_berkeley_rent_stabilization_ordinance_bmc_chapter_13_76:
// Berkeley Rent Stabilization and Good Cause for Eviction Ordinance —
// second oldest comprehensive California municipal rent-control
// regime, enacted by voters of Berkeley in June 1980 as Measure I
// (one year after SF Chapter 37 was enacted in 1979). Codified at
// Berkeley Municipal Code Chapter 13.76; administered by the
// Berkeley Rent Stabilization Board (the Rent Board). Four primary
// components: (1) mandatory registration of all covered rental units
// with the Rent Board (BMC § 13.76.080); (2) rent control via Annual
// General Adjustment (AGA) tied to CPI and capped at 5 PERCENT under
// Measure BB (2020); (3) eviction protection through good-cause
// requirements (BMC § 13.76.130); (4) tenant's right to annual
// interest on security deposits (BMC § 13.76.070). Base rent ceiling
// tied to lawful rent due on or before MAY 31, 1980 under Temporary
// Rent Stabilization Ordinance No. 5212-N.S. Just cause grounds
// under BMC § 13.76.130: 11 enumerated grounds after December 2024
// amendment removed one ground (previously 12). Owner move-in
// requires 90-day vacancy search among landlord's other Berkeley
// rental properties before serving OMI termination. Notice-of-
// termination filing requirement: landlord must file copy with
// Berkeley Rent Board within 3 BUSINESS DAYS of service on tenant;
// notice must state just cause, reference Rent Board counseling
// services, AND allege landlord's compliance with registration and
// habitability standards. Non-qualifying grounds explicitly NOT
// valid bases for eviction: property sales, lease expiration,
// Section 8 status changes, AND foreclosure (the foreclosure
// carve-out is one of the strongest tenant protections in CA
// municipal rent-control regimes). Costa-Hawkins Rental Housing Act
// of 1995 vacancy decontrol overlay applies to single-family homes,
// condominiums, and post-Feb-1-1995 certificate-of-occupancy
// buildings. Sixteen-mode severity ladder × 2 property jurisdictions
// × 5 unit types × 6 compliance aspects × 13 just-cause grounds
// (11 enumerated + NoJustCauseAsserted + NonQualifyingGround) ×
// variable AGA / OMI / notice-filing / registration / security
// deposit inputs. Sibling cluster: rental_san_francisco_rent_
// ordinance_chapter_37 (iter 677 — SF 1979 OLDEST California
// municipal regime; Berkeley 1980 SECOND OLDEST), rental_seattle_
// smc_22_206_160_just_cause_eviction (iter 669 — Seattle JCEO 1980
// FIRST municipal just-cause-only regime; Berkeley adds rent control
// PLUS just-cause), rental_california_sb_567_no_fault_eviction_
// amendments (iter 673 — CA AB 1482 + SB 567 statewide overlay;
// Berkeley Chapter 13.76 retains local supremacy in Berkeley
// itself), rental_california_ab_12_security_deposit_cap (iter 645
// — CA security deposit cap), rental_california_ab_2347_unlawful_
// detainer_response (iter 667 — CA UD response time), rental_just_
// cause_eviction (multi-state base regime; Berkeley = strictest US
// local just-cause + rent-control regime with foreclosure carve-
// out), rental_owner_move_in_eviction (OMI cross-reference; Berkeley
// 90-day vacancy search is one of strictest in US), rental_security_
// deposit_interest (Berkeley BMC § 13.76.070 is one of the few
// jurisdictions with mandatory rate-set-by-board annual interest),
// rental_property_registration (Berkeley BMC § 13.76.080 mandatory
// annual registration is the model for many California cities),
// rental_rent_control_stabilization (multi-state regime).
// ---------------------------------------------------------------------------

async fn rental_berkeley_rent_stabilization_ordinance_bmc_chapter_13_76_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalBerkeleyRentStabilizationOrdinanceBmcChapter1376Input>,
) -> Result<Json<RentalBerkeleyRentStabilizationOrdinanceBmcChapter1376Result>, ApiError> {
    Ok(Json(check_rental_berkeley_rent_stabilization_ordinance_bmc_chapter_13_76(&b)))
}

// ---------------------------------------------------------------------------
// rental_gas_appliance_ban: Rental property gas appliance ban /
// electrification mandate compliance — when a trader-landlord building
// new construction OR substantially renovating an existing rental
// property must comply with statutory bans on natural gas / propane
// hookups + fossil-fuel-burning appliances. Mounted at POST /api/
// rental/rental-gas-appliance-ban. Three regimes: NY All-Electric
// Buildings Act S4006C/A3006C Part RR (2023) eff. Jan 1 2026 (bans
// fossil-fuel hookups in MOST new homes — covers natural gas mains +
// propane tanks + boilers + furnaces + water heaters + gas ranges +
// gas dryers + gas fireplaces + supply piping; enforcement STAYED
// pending Second Circuit resolution expected fall 2026); CA 2025
// Energy Code Title 24 Part 6 eff. Jan 1 2026 (new construction
// permits require heat pumps for most space + water heating; does NOT
// require existing landlords to replace existing gas appliances; SF
// considering 2027 major-renovation electrification ordinance; Cal.
// Restaurant Ass'n v. City of Berkeley 89 F.4th 1094 (9th Cir. 2023)
// enjoined Berkeley's 2019 gas-hookup ban as EPCA-preempted under 42
// USC § 6297); Default (federal law silent; 42 USC § 6297 EPCA limits
// direct-ban approach; locality-controlled). Trader-landlord critical
// for new construction / substantial renovation projects in NY (post-
// Jan 1 2026 pending Second Circuit ruling) + CA. Distinct from
// siblings cooling_requirements, heat_requirements, detector_
// requirements, landlord_repair_response_timeframe.
// ---------------------------------------------------------------------------

async fn rental_gas_appliance_ban_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalGasApplianceBanInput>,
) -> Result<Json<RentalGasApplianceBanResult>, ApiError> {
    Ok(Json(check_rental_gas_appliance_ban(&b)))
}

// ---------------------------------------------------------------------------
// rental_gas_piping_inspection_local_law_152: NYC Local Law 152 of
// 2016 periodic gas piping inspection compliance. Enacted in
// response to March 12, 2014 East Harlem gas explosion at 1644-1646
// Park Avenue (8 fatalities). NYC Admin Code § 28-318 + 1 RCNY
// § 103-10. All NYC buildings except R-3 (one- and two-family
// homes) must be inspected every 4 years by Licensed Master Plumber
// (LMP). Community-district rotating schedule (Districts 1/3/10
// 2024+, 2/5/7/13/18 2025+, 4/6/8/9/16 2026+, 11/12/14/15/17 2027+).
// Filing workflow: LMP delivers GPS1 to owner within 30 days; owner
// files GPS2 with DOB within 60 days; corrections certified within
// 120 days, or 180 days with DOB-approved extension. Unsafe
// condition: immediate 911 + utility + DOB notification + LMP
// gas-shutoff authority. Penalties: $10,000 missed filing + $10,000/
// year continuing; Cycle 2+ non-certification $1,500 residential /
// $5,000 commercial. Sibling: rental_facade_inspection_fisp_local_
// law_11 (NYC LL 11 facade 5-year cycle), rental_natural_gas_leak_
// response (NYC tenant gas-leak response protocol), rental_carbon_
// monoxide_detector (combustion safety), rental_gas_appliance_ban
// (all-electric mandate stacks atop LL 152).
// ---------------------------------------------------------------------------

async fn rental_gas_piping_inspection_local_law_152_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalGasPipingInspectionLocalLaw152Input>,
) -> Result<Json<RentalGasPipingInspectionLocalLaw152Result>, ApiError> {
    Ok(Json(check_rental_gas_piping_inspection_local_law_152(&b)))
}

// ---------------------------------------------------------------------------
// rental_garage_door_safety_compliance: Multi-jurisdictional rental
// property automatic garage door opener safety compliance framework.
// When a landlord rents a property with an automatic garage door opener,
// what entrapment-protection standards apply (UL 325 photoelectric eye
// + auto-reverse + sensing edge), what pre-1993 legacy opener
// replacement obligations attach, and what failure-mode liabilities
// expose landlord after a child-entrapment injury or fatality? Mounted
// at POST /api/rental/rental-garage-door-safety-compliance. Three-
// jurisdiction framework: Federal/CPSC (universal floor — Consumer
// Product Safety Improvement Act of 1990, Pub. L. 101-608, 15 U.S.C.
// § 2056 + 16 C.F.R. Part 1211 Safety Standard for Automatic
// Residential Garage Door Operators + ANSI/UL 325 mandatory standard
// since JANUARY 1, 1993; CPSC reports child-entrapment incidents
// reduced to NEARLY ZERO with properly installed external photo-eye
// protection); California (Cal. Civ. Code § 1941.1 implied warranty
// per Green v. Superior Court, 10 Cal. 3d 616 (1974) + Cal. Bus. &
// Prof. Code § 7026 C-61/D-28 contractor licensing + Cal. SB 969
// (2018) battery-backup requirement effective July 1, 2019 for
// emergency egress during power outages); Default (common-law implied
// warranty per Hilder v. St. Peter, 478 A.2d 202 (Vt. 1984) + tort
// negligence + premises liability + 15 U.S.C. § 2068 prohibited acts
// when continuing to lease unit with recalled opener). UL 325 (post-
// 1993) five safety requirements: external entrapment protection via
// photoelectric infrared sensors (mounted 4-6 inches above floor) OR
// sensing-edge contact strips; internal entrapment monitoring via
// motor strain detection; auto-reverse on contact within 2 seconds
// plus 3-second timeout; 5.5-second close time enforced minimum;
// annual 2x4 lumber test (door should auto-reverse without crushing).
// Three opener generations: PRE-1993 NOT equipped with external
// protection — CPSC + manufacturers recommend REPLACEMENT (not
// repair); insurance carriers commonly deny coverage for pre-1993
// opener incidents; 1993-2010 first-generation UL 325 compliant;
// POST-2010 current edition with enhanced features including battery
// backup. Five universal failure-mode liabilities: pre-1993 opener
// still in service; photoelectric eye obstructed or disabled by
// tenant; auto-reverse 2x4 test failure; pinch-point unprotected
// (finger amputation); CA SB 969 battery-backup missing on post-July-
// 1-2019 installation. Child-entrapment fatality settlements
// routinely exceed $5M; pre-1993 opener replacement cost $300-$800
// vs litigation exposure. Distinct from siblings rental_window_blind_
// cord_safety (iter 469), rental_window_guard_installation, rental_
// swimming_pool_drain_safety, rental_carbon_monoxide_detector,
// rental_chimney_fireplace_inspection_disclosure (iter 471), rental_
// fire_extinguisher_requirement (iter 473), rental_hardwired_smoke_
// alarm_responsibility (iter 481), rental_bedroom_egress_window,
// tenant_emotional_distress_damages.
// ---------------------------------------------------------------------------

async fn rental_garage_door_safety_compliance_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalGarageDoorSafetyComplianceInput>,
) -> Result<Json<RentalGarageDoorSafetyComplianceResult>, ApiError> {
    Ok(Json(check_rental_garage_door_safety_compliance(&b)))
}

// ---------------------------------------------------------------------------
// rental_hardwired_smoke_alarm_responsibility: Multi-jurisdictional
// rental property HARDWIRED SMOKE ALARM installation, maintenance, and
// operability compliance framework. When a landlord rents a property,
// what smoke alarm technology (hardwired with battery backup vs 10-year
// sealed battery), installation locations (every bedroom + hallway
// outside sleeping areas + each floor including basement), and post-
// tenancy verification duties apply, and what failure-mode liabilities
// expose landlord after a fire injury or fatality? Mounted at POST
// /api/rental/rental-hardwired-smoke-alarm-responsibility. Three-
// jurisdiction framework: California (MOST PRESCRIPTIVE — Cal. Health
// & Safety Code § 13113.7 + § 13113.8 + Cal. Civ. Code § 1941.1(a)(7)
// implied warranty + 2014 STATE FIRE MARSHAL 10-YEAR SEALED-BATTERY
// mandate + JANUARY 1, 2016 HARDWIRED-WITH-BATTERY-BACKUP mandate for
// newly installed alarms + landlord operability duty at new tenancy +
// tenant notification and landlord correction duty); Massachusetts
// (527 C.M.R. 1.00 Comprehensive Fire Safety Code + M.G.L. c. 148
// § 26F — PHOTOELECTRIC mandate in bedrooms and adjacent sleeping
// areas + 10-year sealed-battery for new + hardwired-with-backup in
// new construction); Default (NFPA 72 National Fire Alarm and
// Signaling Code referenced by most state/local fire codes + common-
// law implied warranty of habitability per Hilder v. St. Peter, 478
// A.2d 202 (Vt. 1984) + tort negligence + premises liability). NFPA
// 72 placement requirements: inside every bedroom + outside each
// separate sleeping area + on each additional story including
// basement + interconnected so all alarms activate when any one
// detects smoke + 4-inch wall/ceiling clearances + 36-inch HVAC
// register clearance. Six smoke alarm types modeled: hardwired with
// battery backup; 10-year sealed-battery; replaceable-battery (pre-
// 2014 California legacy); photoelectric; ionization; dual-sensor.
// Five universal failure-mode liabilities: alarm inoperable at start
// of tenancy (§ 13113.7 violation); tenant reported inoperable not
// corrected (retaliation exposure); no alarm in bedroom or hallway
// (NFPA 72 placement violation + negligence per se); fire injury or
// fatality ($1M-$5M routine settlement); battery-removal failure on
// replaceable-battery design. Distinct from siblings rental_carbon_
// monoxide_detector, rental_chimney_fireplace_inspection_disclosure
// (iter 471), rental_fire_extinguisher_requirement (iter 473),
// rental_window_blind_cord_safety (iter 469), rental_bedroom_egress_
// window, rental_smoke_free_housing_disclosure (smoking policy NOT
// alarms), tenant_fire_safety_plan_disclosure, landlord_retaliation_
// damages, tenant_emotional_distress_damages.
// ---------------------------------------------------------------------------

async fn rental_hardwired_smoke_alarm_responsibility_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalHardwiredSmokeAlarmResponsibilityInput>,
) -> Result<Json<RentalHardwiredSmokeAlarmResponsibilityResult>, ApiError> {
    Ok(Json(check_rental_hardwired_smoke_alarm_responsibility(&b)))
}

// ---------------------------------------------------------------------------
// rental_hawaii_residential_landlord_tenant_code_hrs_521: Hawaii
// Residential Landlord-Tenant Code (HRS Chapter 521) — comprehensive
// state-level landlord-tenant statute covering all residential rental
// properties in the State of Hawaii; administered by Hawaii DCCA
// Regulated Industries Complaints Office (RICO). HRS § 521-44(b)
// security deposit cap of ONE MONTH'S RENT; § 521-44(c) 14-day
// return after termination; § 521-44(f) treble (3X) damages for
// willful retention. HRS § 521-21(d) rent increase notice: 45
// consecutive days month-to-month, 15 days week-to-week; no
// statewide rent control. HRS § 521-71(a) landlord termination
// notice 45 days; § 521-71(b) tenant termination notice 28 days;
// § 521-71(d) demolition / conversion notice 120 days. HRS § 521-68
// 5-business-day non-payment notice; § 521-72 10-day rule violation
// cure notice; § 521-53 2-day landlord entry notice; § 521-64
// 3-business-day emergency repairs / 12-business-day general
// repairs; § 521-64(b) repair-and-deduct remedy. § 521-74
// retaliatory eviction prohibition with rebuttable presumption.
// § 521-8 transient lodging exclusion (hotels, motels, vacation
// rentals under 90 days). 27-mode severity ladder × 2 jurisdictions
// × 3 unit types × 12 compliance aspects × 2 retaliation statuses ×
// variable monthly rent / security deposit / notice days / repair
// response times inputs. Sibling cluster: rental_san_francisco_
// rent_ordinance_chapter_37 (iter 677), rental_berkeley_rent_
// stabilization_ordinance_bmc_chapter_13_76 (iter 679), rental_
// seattle_smc_22_206_160_just_cause_eviction (iter 669), rental_
// oakland_measure_ee_just_cause_omc_8_22 (iter 681), rental_
// california_sb_567_no_fault_eviction_amendments (iter 673),
// rental_minneapolis_renter_protections_ordinance_2020 (iter 685),
// rental_cook_county_rtlo (iter 671), rental_security_deposit_
// interest (multi-state security deposit interest), rental_late_
// fee_cap (multi-state late fee cap), rental_just_cause_eviction
// (multi-state base regime; Hawaii lacks just-cause), rental_
// rent_control_stabilization (multi-state; Hawaii lacks rent
// control), rental_short_term_subletting_airbnb_restriction
// (Hawaii transient accommodations cross-reference).
// ---------------------------------------------------------------------------

async fn rental_hawaii_residential_landlord_tenant_code_hrs_521_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalHawaiiResidentialLandlordTenantCodeHrs521Input>,
) -> Result<Json<RentalHawaiiResidentialLandlordTenantCodeHrs521Result>, ApiError> {
    Ok(Json(check_rental_hawaii_residential_landlord_tenant_code_hrs_521(&b)))
}

// ---------------------------------------------------------------------------
// rental_heat_minimum_temperature_season: Multi-jurisdiction heat-
// season minimum-temperature compliance covering NYC Admin Code
// § 27-2029 (Oct 1 - May 31, 68°F day when outdoor < 55°F, 62°F
// night), Chicago Municipal Code § 13-196-410 + § 5-12-110 (Sept 15
// - June 1, 68°F day 8:30am-10:30pm, 66°F night), Boston / MA 105
// CMR 410.201 (Sept 15 - June 15 longest in country, 68°F day
// 7am-11pm, 64°F night), Philadelphia PM-602 (Oct 1 - Apr 30 all
// hours 68°F + May/Sept shoulder when outdoor < 40°F). All four
// jurisdictions prohibit portable space heaters / ovens / cooking
// appliances as primary heat source. Sibling cluster:
// rental_radiator_steam_heat_safety (NYC LL 79/2018 + Ben Z's Law
// radiator burn prevention), rental_hot_water_temperature (CA 120°F
// hot water minimum), rental_carbon_monoxide_detector (winter heat
// season CO risk).
// ---------------------------------------------------------------------------

async fn rental_heat_minimum_temperature_season_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalHeatMinimumTemperatureInput>,
) -> Result<Json<RentalHeatMinimumTemperatureResult>, ApiError> {
    Ok(Json(check_rental_heat_minimum_temperature_season(&b)))
}

// ---------------------------------------------------------------------------
// rental_hot_water_temperature: Rental property minimum hot water
// temperature compliance — when a trader-landlord must supply hot
// water at a statutorily-specified minimum temperature. Mounted at
// POST /api/rental/rental-hot-water-temperature. Three regimes:
// California (Cal. Health & Safety Code § 114192 + Cal. Civ. Code §
// 1941.1 + 22 CCR § 81088 — 120°F minimum at faucet; implied warranty
// of habitability includes hot water; 105-120°F range for care
// facilities); NYC HMC § 27-2031 Article 8 Heat and Hot Water (120°F
// constant minimum 365 days/year; HPD § 27-2115 enforcement); Default
// (IPC § 607.1.1 — 110°F minimum at outlet; state-specific
// habitability standards). Distinct from siblings heat_requirements
// (space heat), cooling_requirements, lead_in_drinking_water_
// disclosure, water_heater_earthquake_strap.
// ---------------------------------------------------------------------------

async fn rental_hot_water_temperature_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalHotWaterTemperatureInput>,
) -> Result<Json<RentalHotWaterTemperatureResult>, ApiError> {
    Ok(Json(check_rental_hot_water_temperature(&b)))
}

// ---------------------------------------------------------------------------
// rental_hud_hotma_income_asset_compliance: HUD Housing Opportunity
// Through Modernization Act of 2016 (HOTMA; P.L. 114-201; signed
// July 29, 2016) Income and Asset Compliance. HUD Final Rule
// published in Federal Register on February 14, 2023 (88 FR 9600);
// general effective date January 1, 2024. HOTMA Section 102 amends
// 24 CFR § 5.609(a) income definition: all amounts received by
// adult household members plus unearned income by household member
// under age 18 is income unless excluded; imputed returns on assets
// over $50,000 plus actual returns on calculable assets. HOTMA
// Section 103 standardizes elderly/disabled/medical/child care
// deductions under revised 24 CFR § 5.611. HOTMA Section 104 (NEW)
// imposes two asset limits for public housing, tenant-based Section
// 8 (Housing Choice Voucher), and project-based Section 8: $100,000
// net household assets ceiling OR ownership of real property
// suitable for occupancy by household as residence. Pre-HOTMA
// imputed asset threshold raised from $5,000 to $50,000. Compliance
// extensions: original deadline January 1, 2024; first extension
// to January 1, 2025 (Sept 29, 2023); PIH Sept 18, 2024 further
// delay; HUD Notice H-2025-03 extended Multifamily compliance to
// January 1, 2026; Federal Register Dec 30, 2025 (90 FR) further
// extension to January 1, 2027 for full Multifamily compliance.
// Covered Programs: Public Housing; Section 8 HCV (tenant-based);
// Section 8 Project-Based Rental Assistance; Section 8 Moderate
// Rehabilitation; HOPWA; Multifamily Section 202 / 811. Twelve-mode
// severity ladder × seven HUD programs × four asset limit
// exceptions × three imputed return statuses. Trader-landlord
// critical for: Section 8 HCV portfolio operators; project-based
// Section 8 owners under HAP contracts; LIHTC + Section 8 layered
// properties; mixed-income developments with Section 8 households;
// PHA-administered scattered-site programs; tracking Multifamily
// Section 202 / 811 January 1, 2027 final compliance deadline.
// Sibling cluster: rental_source_of_income_discrimination (Section
// 8 voucher anti-discrimination cross-reference), rental_eviction_
// notices (Section 8 HAP contract termination procedures),
// rental_just_cause_eviction (LIHTC/Section 8 just-cause overlay),
// rental_application_denial_disclosure (HOTMA documentation
// requirements), rental_property_registration (PHA registration).
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// rental_housing_for_older_persons_act_hopa_1995: Housing for Older
// Persons Act of 1995 (Public Law 104-76; signed by President Bill
// Clinton on December 28, 1995) — federal Fair Housing Act exemption
// from the FHA's prohibition on familial-status discrimination
// (42 USC § 3604(b)). Codified at 42 USC § 3607(b); implemented at
// 24 CFR Part 100 Subpart E §§ 100.300-100.308. Three exemption
// categories: (1) 62+ communities — 100 % occupancy at or above 62;
// (2) 55+ communities — 80 % of occupied units with at least one
// resident 55 or older + age verification at least once every 2
// years via reliable documentation + published written policies and
// procedures demonstrating intent to operate as housing for persons
// 55 or older; (3) state or federally funded elderly housing
// programs (HUD Section 202 Supportive Housing for the Elderly,
// LIHTC age-targeted projects, etc.). HOPA eliminated the prior
// 1988 FHA "significant facilities and services" requirement —
// communities no longer need to demonstrate special amenities for
// older persons to qualify. Good-faith-reliance immunity protects
// persons who relied on a written statement that the property
// qualifies for the 55+ exemption, even if later determined
// ineligible. Without a valid HOPA exemption, the FHA familial-
// status protection at 42 USC § 3604(b) applies: landlord may not
// refuse to rent / sell / evict based on presence of children under
// 18. Enforcement: HUD Office of Fair Housing and Equal Opportunity
// (FHEO) administrative investigations; private right of action
// under 42 USC § 3613 (statutory + actual damages + reasonable
// attorney fees + injunctive relief); civil penalties under 42 USC
// § 3614 for pattern-or-practice violations. Ten-mode severity
// ladder × four claimed-exemption categories × two familial-status
// action statuses × two good-faith-reliance statuses × variable
// occupancy percentage (basis points) / age-verification cycle /
// written-policies inputs. Trader-landlord critical for any
// portfolio operator targeting active-adult / 55+ / 62+ retirement
// markets; HOPA compliance failure converts a community advertised
// as age-restricted into an FHA familial-status-protected
// community, voiding existing no-children policies and triggering
// civil liability for past refusals/evictions. The 80 % occupancy
// boundary and the 2-year age-verification cycle are the most-
// litigated requirements; the written-policies requirement traps
// communities that operate informally without published age-
// restriction documentation. Sibling cluster: rental_fair_housing_
// act_familial_status (FHA familial-status base regime — HOPA is
// the carve-out from this baseline), rental_hud_section_202_
// supportive_housing_elderly (HUD Section 202 program — example
// of state/federally funded elderly housing category), rental_
// section_504_accessibility (FHA accessibility regime — parallel
// FHA structural compliance), rental_vawa_2022 (FHA cross-reference
// for VAWA tenant protections), rental_hud_hotma_income_asset_
// compliance (iter 653 — HOTMA HUD income/asset compliance under
// Section 8 and Section 202), rental_just_cause_eviction (just-
// cause regimes that overlay HOPA-exempt communities).
// ---------------------------------------------------------------------------

async fn rental_housing_for_older_persons_act_hopa_1995_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalHousingForOlderPersonsActHopa1995Input>,
) -> Result<Json<RentalHousingForOlderPersonsActHopa1995Result>, ApiError> {
    Ok(Json(check_rental_housing_for_older_persons_act_hopa_1995(&b)))
}

async fn rental_hud_hotma_income_asset_compliance_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalHudHotmaIncomeAssetComplianceInput>,
) -> Result<Json<RentalHudHotmaIncomeAssetComplianceResult>, ApiError> {
    Ok(Json(check_rental_hud_hotma_income_asset_compliance(&b)))
}

// ---------------------------------------------------------------------------
// rental_hud_section_504_rehabilitation_act_24_cfr_part_8: HUD
// Section 504 of the Rehabilitation Act of 1973 (29 USC § 794) and
// 24 CFR Part 8 implementing regulations — federal nondiscrimination
// on the basis of disability in federally assisted housing programs
// and activities. Substantially updated by HUD final rule published
// in the Federal Register on April 25, 2023. Applies to Section 8
// Housing Choice Voucher operators, Section 8 PBRA owners, PHAs,
// HOPWA, Section 202 elderly housing, Section 811 disabled housing,
// and certain LIHTC properties with HUD-administered financing
// layers. 24 CFR § 8.22 new construction: 5% of units (or at least
// one) mobility-accessible + additional 2% (or at least one) sensory-
// accessible. 24 CFR § 8.23 substantial alteration: must meet § 8.22
// standards. 24 CFR § 8.24 existing facility: must be readily
// accessible when viewed in its entirety. 24 CFR § 8.32 accessibility
// standards: UFAS (default) or 2010 ADA Standards (alternative since
// March 2011 DOJ guidance). 24 CFR § 8.20 reasonable accommodation
// requirement: structural modifications + policy adjustments to
// enable disabled persons to participate. Procedural obligations for
// recipients with 15+ employees: § 8.53 designated responsible
// employee + § 8.51 self-evaluation (3-year retention) + § 8.53
// grievance procedures + § 8.54 notice of nondiscrimination.
// Twenty-two-mode severity ladder × 2 federally-assisted statuses ×
// 4 project statuses × 8 compliance aspects × 3 accessibility
// standards × 3 reasonable-accommodation statuses × variable
// employee count + accessible unit counts + procedural flags.
// Sibling cluster: rental_housing_for_older_persons_act_hopa_1995
// (iter 675 — HOPA FHA familial-status carve-out), rental_vawa_2022_
// federal_housing_protections (FHA VAWA cross-reference), rental_
// hud_hotma_income_asset_compliance (iter 653 — HOTMA HUD income/
// asset compliance), rental_application_denial_disclosure (FCRA-
// adjacent application denial), rental_just_cause_eviction (just-
// cause regimes overlay), rental_source_of_income_discrimination
// (parallel FHA-adjacent income protection).
// ---------------------------------------------------------------------------

async fn rental_hud_section_504_rehabilitation_act_24_cfr_part_8_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalHudSection504RehabilitationAct24CfrPart8Input>,
) -> Result<Json<RentalHudSection504RehabilitationAct24CfrPart8Result>, ApiError> {
    Ok(Json(check_rental_hud_section_504_rehabilitation_act_24_cfr_part_8(&b)))
}

// ---------------------------------------------------------------------------
// rental_property_registration: Mandatory landlord rental property
// registration with state / municipal agency — distinct from owner_
// identification + landlord_identification_disclosure + tenant_rights_
// statement_disclosure. Affirmative registration obligation BEFORE
// lawfully renting. Mounted at POST /api/rental/rental-property-
// registration. Three regimes: (1) New Jersey N.J.S.A. §§ 46:8-28 +
// 46:8-28.5: two-tier filing (1-unit OR 2-unit non-owner-occupied →
// municipal clerk; 3+ unit "multiple dwelling" under Hotel and Multiple
// Dwelling Law → Bureau of Housing Inspection, NJ DCA); certificate
// contents include record owner + agent for service of process + bank
// holding security deposits; amended certificate within 20 DAYS of any
// change; nonregistration = cannot enforce rent collection + cannot
// pursue eviction + treble damages under N.J.S.A. 56:8-1 CFA. (2)
// District of Columbia D.C. Code § 47-2851.03: Basic Business License
// with Rental Housing endorsement; DC v. Hayes equitable bar — landlord
// without license BARRED from collecting rent in court. (3) Default:
// no statewide mandate; municipal ordinances (Chicago RLTO, NYC MDL §
// 325, MA Mass. Gen. Laws ch. 111 § 197A) may impose.
// ---------------------------------------------------------------------------

async fn rental_property_registration_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalPropertyRegistrationInput>,
) -> Result<Json<RentalPropertyRegistrationResult>, ApiError> {
    Ok(Json(check_rental_property_registration(&b)))
}

// ---------------------------------------------------------------------------
// rental_radon_mitigation_disclosure: Multi-jurisdictional rental
// property radon disclosure and mitigation compliance framework. When a
// landlord rents a residential property, what radon-hazard disclosure
// must be provided to tenant at lease signing, what tenant testing
// rights attach, what mitigation obligations follow elevated test
// results, and what failure-mode liabilities expose landlord after a
// tenant develops lung cancer with multi-decade latency? Radon is the
// #2 LEADING CAUSE OF LUNG CANCER in the United States per EPA,
// killing approximately 21,000 Americans annually. Mounted at POST
// /api/rental/rental-radon-mitigation-disclosure. Three-jurisdiction
// framework: Illinois (MOST STRINGENT for tenants — Illinois Tenants
// Radon Protection Act, 765 ILCS 90/ effective January 1, 2024; pre-
// lease disclosure of IEMA-OHS radon pamphlet + known hazard records +
// tenant 90-DAY testing window + 4 pCi/L action level + tenant LEASE
// TERMINATION right if landlord declines to mitigate); Maine (10
// M.R.S. § 1494-A et seq. landlord-rented residential properties must
// test for radon every 10 years and provide written disclosure of
// results); Default (EPA 4 pCi/L action level + EPA Map of Radon Zones
// Zone 1/2/3 + common-law implied warranty of habitability per Hilder
// v. St. Peter, 478 A.2d 202 (Vt. 1984) + Green v. Superior Court, 10
// Cal. 3d 616 (1974) + Cal. Civ. Code § 1941.1). Mitigation
// methodology: AARST-NRPP certified contractor for active soil
// depressurization (ASD) $800-$1,500 + sub-slab + sub-membrane +
// crack sealing + post-mitigation verification testing. Five
// universal failure-mode liabilities: failure to provide IL radon
// pamphlet (765 ILCS 90 statutory violation); failure to disclose
// known elevated radon (fraud + tort); decline to mitigate after
// tenant test > 4 pCi/L (IL 90-day lease termination right); failure
// to engage AARST-NRPP certified contractor (ineffective remediation
// + insurance denial); lung cancer claim with multi-decade latency
// ($1M-$5M+ settlement routine). Distinct from siblings rental_lead_
// pipe_disclosure (lead service lines), rental_basement_water_
// intrusion_disclosure, rental_chimney_fireplace_inspection_disclosure
// (iter 471), rental_carbon_monoxide_detector, rental_natural_gas_
// leak_response (iter 485 — methane leak), rental_underground_
// storage_tank_disclosure (UST/LUST), tenant_emotional_distress_
// damages.
// ---------------------------------------------------------------------------

async fn rental_radon_mitigation_disclosure_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalRadonMitigationDisclosureInput>,
) -> Result<Json<RentalRadonMitigationDisclosureResult>, ApiError> {
    Ok(Json(check_rental_radon_mitigation_disclosure(&b)))
}

// ---------------------------------------------------------------------------
// rental_renters_insurance_requirement: lease-imposed renter's
// insurance requirement enforceability across CA / TX / FL / NY /
// OH / other states. All 50 states permit contractual requirement;
// California requires CLEARLY STATED IN WRITING before lease
// signed or renewed; courts uphold standard $100K liability + $10K
// personal property; excessive amounts (≥ $1M liability)
// effectively prohibitive and unenforceable. Verbal requirements
// not enforceable; mid-lease imposition requires signed amendment.
// HUD does not require renter's insurance for Section 8.
// Ten-mode severity ladder × six jurisdictions × six lease
// formality states × four additional-insured statuses. Trader-
// landlord critical: most form leases include renter's insurance
// requirement but many are unenforceable due to formality defects.
// ---------------------------------------------------------------------------

async fn rental_renters_insurance_requirement_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalRentersInsuranceRequirementInput>,
) -> Result<Json<RentalRentersInsuranceRequirementResult>, ApiError> {
    Ok(Json(check_rental_renters_insurance_requirement(&b)))
}

// ---------------------------------------------------------------------------
// rental_water_submetering_disclosure: Rental property water
// submetering disclosure compliance — when a trader-landlord must
// register with state PUC, disclose submetering arrangement at lease
// signing, and follow billing-transparency rules before billing
// tenants separately from rent for water/wastewater service via
// submeters or RUBS (ratio utility billing system). Mounted at POST
// /api/rental/rental-water-submetering-disclosure. Four regimes:
// California SB 7 of 2016 (Cal. Civ. Code § 1954.201 et seq. + Cal.
// Public Utilities Code § 739.5; MANDATORY submetering for newly
// constructed multiunit/mixed-use structures with water connection
// applications after January 1, 2018; pre-lease written disclosure
// of billing method + frequency + dispute process required); Texas
// Water Code § 13.503 + 16 TAC § 24.275 et seq. (PUCT registration
// REQUIRED before billing tenants; tenant guide required; 30
// percent administrative fee cap; quarterly past-usage disclosure);
// Florida PSC voluntary framework (encouraged but not mandated;
// no statewide PUC registration); Default RUBS framework (38+
// states permit; pre-lease disclosure recommended; no statewide
// administrative fee cap). Distinct from siblings rental_hot_water_
// temperature, rental_bed_bug_disclosure, rental_gas_appliance_ban,
// rental_property_registration.
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// rental_well_water_disclosure: Multi-jurisdictional rental property
// PRIVATE WELL WATER testing + disclosure compliance framework. When a
// landlord rents a property served by a private water well rather than a
// municipal/community water supply, what testing schedule applies, what
// contaminants must be tested for, what disclosure must be given to
// tenant, and what failure-mode liabilities expose landlord? Mounted at
// POST /api/rental/rental-well-water-disclosure. Four-jurisdiction
// framework: New Jersey (MOST STRINGENT — first-in-nation Private Well
// Testing Act N.J.S.A. 58:12A-26 et seq., signed March 2001, effective
// September 2002; N.J.S.A. 58:12A-32 lessor must obtain + pay for full
// PWTA test every 60 months/5 years AND provide written results to each
// rental unit within 30 days of receipt AND to new lessee at lease
// execution; N.J.A.C. 7:9E contaminant panel = total coliform + iron +
// manganese + pH + all VOCs + nitrate + lead + gross alpha + arsenic
// statewide since 2021 + county-specific radon + PFAS; $5,000 statutory
// penalty per N.J.S.A. 58:12A-31); Connecticut (Conn. Gen. Stat. § 19a-37
// + Public Act 16-66 (2016) + Public Health Code § 19-13-B51d testing at
// construction + title transfer + Conn. Gen. Stat. § 47a-7 common-law
// habitability); Pennsylvania (NO state-level rental disclosure statute;
// PA DEP voluntary recommendations under 25 Pa. Code Ch. 109 cover public
// water systems only; common-law habitability + Pa. Const. art. I § 27
// Environmental Rights Amendment); Default (federal Safe Drinking Water
// Act 42 U.S.C. § 300f explicitly exempts private wells serving fewer
// than 25 individuals or 15 service connections; common-law habitability
// per Hilder v. St. Peter, 478 A.2d 202 (Vt. 1984); Cal. Civ. Code
// § 1941.1 implied warranty of sanitary facilities). Nine-contaminant
// universal baseline: total coliform + nitrate + lead + arsenic + VOCs
// + pH + iron/manganese + gross alpha + PFAS (per EPA April 10, 2024
// National Primary Drinking Water Regulation 89 Fed. Reg. 32532 with
// MCLs for PFOA 4 ppt + PFOS 4 ppt + GenX/HFPO-DA + PFNA + PFHxS 10
// ppt). CERCLA 42 U.S.C. § 9607(a) owner/operator strict liability
// exposure for PFAS contamination after EPA April 2024 PFOA/PFOS
// hazardous-substance designation. Five failure-mode liabilities:
// failure to test/disclose; MCL exceedance; lead + child residents;
// PFAS exceedance + CERCLA; well casing/pressure tank failure. Distinct
// from siblings rental_septic_system_disclosure (iter 465 — OSTDS),
// rental_lead_pipe_disclosure, rental_basement_water_intrusion_
// disclosure, rental_water_submetering_disclosure, rent_abatement_
// construction_nuisance, mid_tenancy_temporary_relocation.
// ---------------------------------------------------------------------------

async fn rental_well_water_disclosure_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalWellWaterDisclosureInput>,
) -> Result<Json<RentalWellWaterDisclosureResult>, ApiError> {
    Ok(Json(check_rental_well_water_disclosure(&b)))
}

// ---------------------------------------------------------------------------
// rental_window_blind_cord_safety: Multi-jurisdictional rental property
// WINDOW BLIND CORD STRANGULATION SAFETY compliance framework. When a
// landlord rents a unit, what window-covering cord-safety standards
// apply, what retrofit obligations attach when child residents under
// age 8 occupy the unit, what product-recall enforcement risks expose
// landlord, and what failure-mode liabilities apply after a
// strangulation incident? Mounted at POST /api/rental/rental-window-
// blind-cord-safety. Three-jurisdiction framework: Federal/CPSC (16
// C.F.R. Part 1260 mandatory standard effective May 30, 2023 for
// custom window coverings under 87 Fed. Reg. 73118 (Nov 28, 2022) and
// Consumer Product Safety Act 15 U.S.C. § 2056 eliminates free-
// hanging operating cords + free-hanging tilt cords + multiple cords
// into cord connectors on all custom window coverings manufactured
// after May 30, 2023; ANSI/WCMA A100.1-2018 voluntary standard
// mandates CORDLESS or inaccessible-cord stock products since May
// 2022; ANSI/WCMA A100.1-2022 effective June 2024 extends to custom
// products); California/progressive states (Cal. Civ. Code § 1941.1
// implied warranty of sanitary facilities + § 1942.4 untenantable
// conditions + § 1942(a) repair-and-deduct + Title 24 CPSC-aligned
// product safety); Default (common-law implied warranty of
// habitability per Hilder v. St. Peter, 478 A.2d 202 (Vt. 1984) +
// Green v. Superior Court, 10 Cal. 3d 616 (1974) + tort negligence
// for child-resident properties + CPSC recall enforcement under
// 16 C.F.R. § 1115 + Consumer Product Safety Act 15 U.S.C. § 2068
// prohibited acts). CPSC reports approximately 120 child fatalities
// from window-covering cord strangulation 2002-2017 to children age
// 8 or younger. Federal mandatory standard does NOT retroactively
// require landlord to replace pre-existing corded blinds installed
// before May 30, 2023; however implied warranty of habitability cases
// routinely treat corded blinds in child-occupied units as
// actionable defects regardless of installation date. Six covering
// types modeled: Cordless, InaccessibleCordOnlyWand,
// AccessibleCordedStock, AccessibleCordedCustom,
// PreEffectiveDateExisting, None. Five universal failure-mode
// liabilities: corded blinds + child resident age 8 or younger;
// ignored retrofit request; CPSC-recalled product in tenant unit;
// custom blind installed after May 30, 2023 violating Part 1260;
// strangulation injury. Distinct from siblings rental_window_guard_
// installation (fall protection — child age 10 or younger),
// rental_carbon_monoxide_detector, rental_bedroom_egress_window,
// rental_swimming_pool_drain_safety, landlord_security_device_
// obligations, tenant_emotional_distress_damages, landlord_
// retaliation_damages.
// ---------------------------------------------------------------------------

async fn rental_window_blind_cord_safety_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalWindowBlindCordSafetyInput>,
) -> Result<Json<RentalWindowBlindCordSafetyResult>, ApiError> {
    Ok(Json(check_rental_window_blind_cord_safety(&b)))
}

// ---------------------------------------------------------------------------
// rental_window_guard_installation: multi-jurisdictional residential
// rental window guard installation requirement framework — NYC Health
// Code Article 131 § 131.15 + NYC Admin Code § 27-2043.1 (multiple
// dwelling with 3+ apartments must install approved window guards on
// every window where child age 10 or younger resides; carveout for fire
// escape windows; 30-day lease notice + annual notice January 1-16; NYC
// Health Code § 3.11 civil penalties up to $1,000 per violation per
// day); Chicago Building Code § 13-196-550 (operable window guards
// limiting opening to 4 inches or less; seasonal screen requirement
// April 15-November 15); Massachusetts G.L. + 105 CMR 410 State
// Sanitary Code (landlord must install at tenant's request when child
// under age 10 resides; applicable window three-prong test: > 6 feet
// above grade + opens for 5-inch ball + not connected to fire escape;
// annual notice with statutory quote); Montgomery County MD DHCA Code
// § 29-23 (tenant-request trigger when child under age 6 resides);
// Default common-law negligence + ASTM F2090-23 voluntary standard.
// Mounted at POST /api/rental/rental-window-guard-installation.
// Trader-landlord critical because child window-fall injuries are
// among the highest-stakes premises liability claims — wrongful death
// awards routinely exceed $5M, and many jurisdictions impose STRICT
// LIABILITY when window guards are required by statute but absent.
// Sibling cluster: rental_bedroom_egress_window,
// rental_carbon_monoxide_detector, rental_swimming_pool_drain_safety,
// landlord_security_device_obligations.
// ---------------------------------------------------------------------------

async fn rental_window_guard_installation_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalWindowGuardInstallationInput>,
) -> Result<Json<RentalWindowGuardInstallationResult>, ApiError> {
    Ok(Json(check_rental_window_guard_installation(&b)))
}

async fn rental_water_submetering_disclosure_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalWaterSubmeteringDisclosureInput>,
) -> Result<Json<RentalWaterSubmeteringDisclosureResult>, ApiError> {
    Ok(Json(check_rental_water_submetering_disclosure(&b)))
}

// ---------------------------------------------------------------------------
// rental_unpermitted_unit_disclosure: Rental property unpermitted /
// illegal dwelling unit disclosure compliance — when a trader-
// landlord rents a unit lacking a Certificate of Occupancy (in-law
// apartment, garage conversion, basement apartment, attic unit, ADU
// without permits), what statutory disclosure and rent-collection
// rules apply? Mounted at POST /api/rental/rental-unpermitted-unit-
// disclosure. Four regimes: California common law + Cal. Civ. Code
// § 1942.4 + § 1102 et seq. TDS + Espinoza v. Calva 169 Cal.App.4th
// 1393 (2008) asymmetric enforceability doctrine (landlord may NOT
// collect rent for unit lacking Certificate of Occupancy; tenant
// CAN enforce lease and sue for damages; TDS Transfer Disclosure
// Statement required at sale); Oakland Municipal Code § 8.22 (TPO
// + Just Cause for Eviction Ordinance + § 8.22.450 relocation
// payments — base $7,931 + $5,287 per senior/disabled/minor, 2024
// amounts; substantial-rehab eviction carve-out); New York City NYC
// Multiple Dwelling Law § 325 + NYC Admin Code § 27-2107 (3+ unit
// buildings MUST have Certificate of Occupancy; landlord cannot
// collect rent for illegal cellar/basement unit; $1,000 + $50/day
// civil penalty under § 27-2115; Loft Law Article 7-C legalization
// pathway); Default common-law warranty of habitability + state
// building code + HUD Section 8 housing quality standards. Distinct
// from siblings rental_bedroom_egress_window, rental_carbon_
// monoxide_detector, rental_hot_water_temperature, rental_property_
// registration.
// ---------------------------------------------------------------------------

async fn rental_unpermitted_unit_disclosure_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalUnpermittedUnitDisclosureInput>,
) -> Result<Json<RentalUnpermittedUnitDisclosureResult>, ApiError> {
    Ok(Json(check_rental_unpermitted_unit_disclosure(&b)))
}

// ---------------------------------------------------------------------------
// rental_vacant_property_registration: multi-jurisdictional vacant
// property registration compliance. Chicago Municipal Code
// § 13-12-125 (owner): 30-day registration; $300 fee per registered
// building; renewal every 6 months; DOUBLED fee for City-identified
// non-compliance; inspector + attorney fees recoverable as lien.
// Chicago § 13-12-126 (mortgagee): later of 30 days after vacancy +
// unregistered status OR 10 days after default; $700 initial;
// $300 every-6-month renewal. Detroit BSEED 30-day registration +
// exterior maintenance. Cleveland OMC § 367.131. Baltimore Building
// Code § 102.5. Philadelphia L&I Vacant Property Strategy. Twelve-
// mode severity ladder × six jurisdictions × three actor roles ×
// two discovery modes. Trader-landlord critical because vacant-
// property registration failures trigger doubled fees, code-
// enforcement liens, daily penalties, and reputational exposure
// for portfolio operators.
// ---------------------------------------------------------------------------

async fn rental_vacant_property_registration_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalVacantPropertyRegistrationInput>,
) -> Result<Json<RentalVacantPropertyRegistrationResult>, ApiError> {
    Ok(Json(check_rental_vacant_property_registration(&b)))
}

// ---------------------------------------------------------------------------
// rental_vawa_2022_federal_housing_protections: Violence Against
// Women Act Reauthorization Act of 2022 (Public Law 117-103; signed
// by President Biden on March 15, 2022); housing rights subpart
// effective October 1, 2022; codified at 34 U.S.C. § 12491. HUD
// Federal Register Notice January 4, 2023 (88 FR 482). Applies to
// 16+ federally-assisted housing programs: Public Housing; Section 8
// HCV (tenant-based); Section 8 PBRA; Section 8 Moderate
// Rehabilitation; Section 202 Supportive Housing Elderly; Section
// 811 Supportive Housing Disabled; HOPWA; HOME; CoC; ESG; McKinney-
// Vento Title IV; LIHTC (IRC § 42); USDA Rural Development; Section
// 221(d)(3) BMIR; Section 236; Section 1437f. Anti-discrimination
// (34 USC § 12491(b)(1)): applicant or tenant may not be denied
// admission, denied assistance, terminated, or evicted because of
// victim status of DV/dating violence/SA/stalking. Lease bifurcation
// (34 USC § 12491(b)(3)): PHA/owner/manager may bifurcate lease to
// evict perpetrator who engaged in criminal activity directly
// relating to DV/DV/SA/stalking against affiliated individual or
// other individual, without evicting victim who is also tenant or
// lawful occupant. Documentation (34 USC § 12491(c)(3)): victim may
// submit HUD Form 5382 self-certification, police/court record,
// victim service provider/attorney/medical statement, or other
// evidence. Confidentiality (34 USC § 12491(c)(4)). Emergency
// transfer plan (34 USC § 12491(e)). HUD Form 5380 VAWA notice
// required at admission, denial, or termination. HUD Form 5382
// victim certification. Thirteen-mode severity ladder × seventeen
// covered housing programs × five victim documentation types ×
// seven landlord actions × three victim bifurcation statuses.
// Trader-landlord critical for federally-assisted housing portfolio
// operators across 16+ HUD/USDA/LIHTC programs; distinct federal
// floor on top of state-law DV protections. Sibling cluster:
// rental_domestic_violence_lock_change_lease_termination (state-law
// DV companion), tenant_domestic_violence_lease_termination,
// dv_termination, rental_hud_hotma_income_asset_compliance (iter
// 653 — HUD program companion), rental_source_of_income_
// discrimination (Section 8 voucher cross-reference), rental_just_
// cause_eviction (Section 8 just-cause overlay).
// ---------------------------------------------------------------------------

async fn rental_vawa_2022_federal_housing_protections_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalVawa2022FederalHousingProtectionsInput>,
) -> Result<Json<RentalVawa2022FederalHousingProtectionsResult>, ApiError> {
    Ok(Json(check_rental_vawa_2022_federal_housing_protections(&b)))
}

// ---------------------------------------------------------------------------
// rental_vehicle_towing_notice_sign_requirements: multi-state
// residential vehicle towing notice + sign compliance. CA Veh Code
// § 22658 — sign at all entrances ≥ 17×22 inches; lettering ≥ 1
// inch; owner-expense language; local law enforcement phone; towing
// company name + phone with written general towing authorization
// agreement; tenant vehicle written notice except (a) in someone
// else's assigned space and reported OR (b) blocking fire lane.
// § 22658(e) non-compliance exposes landlord to DOUBLE the towing +
// storage charges. LA Mun Code § 80.71.4 — minimum sign 24×24
// inches + LAPD phone. Tex Occ Code § 2308 — sign + 24-hour
// storage facility access. Fla Stat § 715.07 — sign + 30-day
// storage. NJ Predatory Towing Prevention Act (P.L. 2007 c.193)
// — sign + max charges + 24-hour access. Illinois 625 ILCS 5/18a
// — ICC-regulated rates + sign. Eighteen-mode severity ladder ×
// seven jurisdictions × five tow scenarios. Trader-landlord
// critical because non-compliant towing exposes landlord to double
// charges + tenant damages + state-AG enforcement.
// ---------------------------------------------------------------------------

async fn rental_vehicle_towing_notice_sign_requirements_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalVehicleTowingNoticeSignRequirementsInput>,
) -> Result<Json<RentalVehicleTowingNoticeSignRequirementsResult>, ApiError> {
    Ok(Json(check_rental_vehicle_towing_notice_sign_requirements(&b)))
}

// ---------------------------------------------------------------------------
// rental_septic_system_disclosure: Multi-jurisdictional rental property
// SEPTIC SYSTEM disclosure + inspection compliance framework. When a
// landlord rents a property served by a private on-site sewage treatment
// and disposal system (OSTDS / septic) rather than connection to municipal
// sewer, what disclosure rules apply, what inspection certifications are
// required, and what failure-mode liabilities expose landlord? Mounted at
// POST /api/rental/rental-septic-system-disclosure. Four-jurisdiction
// framework: Massachusetts (MOST STRINGENT — 310 C.M.R. 15.000 "Title 5"
// State Environmental Code with pass-or-fail inspection at title transfer
// or within 24 months prior, 36 months if pumped annually per
// 310 C.M.R. 15.301; nitrogen-sensitive watersheds Cape Cod + Nantucket +
// Buzzards Bay require innovative/alternative I/A nitrogen-reducing
// technology under July 7, 2023 amendments); Florida (Fla. Stat.
// § 381.0065 + § 381.00655 + Fla. Admin. Code Ch. 64E-6 + 2020 Clean
// Waterways Act SB 712 signed June 30, 2020 — 5-year inspection cycle
// for OSTDS in Basin Management Action Plan / BMAP basins;
// § 381.0065(2)(a) installation permit + § 381.0065(4) operating permit
// for performance-based + § 381.0065(4)(g) voluntary inspection
// notification duty); Texas (Tex. Health & Safety Code § 366.011 et seq.
// + § 366.051 installation permit + § 366.071 authorization to operate +
// 30 Tex. Admin. Code Ch. 285 + TCEQ Authorized Agent registration + TCEQ
// Form 20021); Default (California Water Boards OWTS Policy June 19, 2012
// + Cal. Water Code § 13290 + Cal. Civ. Code § 1941.1 implied warranty of
// sanitary facilities + Green v. Superior Court, 10 Cal. 3d 616 (1974)
// common-law habitability + CERCLA 42 U.S.C. § 9607(a) owner/operator
// strict liability for groundwater contamination). Five universal
// failure-mode liabilities: sewage backup (Hilder v. St. Peter,
// 478 A.2d 202 (Vt. 1984) constructive eviction); groundwater
// contamination (CERCLA strict liability); pump-out frequency neglect
// (Title 5 / § 381.0065(4) violation); drainfield failure ($15K-$45K
// re-engineering); tenant misuse limited disclaimer. Distinct from
// siblings rental_underground_storage_tank_disclosure (UST + LUST),
// rental_basement_water_intrusion_disclosure, rental_sinkhole_disclosure,
// rental_flood_hazard_disclosure, rent_abatement_construction_nuisance.
// ---------------------------------------------------------------------------

async fn rental_septic_system_disclosure_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalSepticSystemDisclosureInput>,
) -> Result<Json<RentalSepticSystemDisclosureResult>, ApiError> {
    Ok(Json(check_rental_septic_system_disclosure(&b)))
}

// ---------------------------------------------------------------------------
// rental_sewer_lateral_responsibility: Multi-jurisdictional rental
// property PRIVATE SEWER LATERAL (PSL) inspection, compliance, and
// landlord-responsibility framework. When a landlord rents a property
// connected to municipal sewer service, what ownership/maintenance/
// inspection obligations apply to the private sewer lateral (the pipe
// connecting the building to the public sewer main), what point-of-
// sale compliance certificate requirements apply, and what failure-
// mode liabilities expose landlord after a sewage backup into the
// dwelling unit? Mounted at POST /api/rental/rental-sewer-lateral-
// responsibility. Four-jurisdiction framework: EBMUD Regional PSL
// Program (MOST PRESCRIPTIVE — Alameda + Albany + Emeryville + Oakland
// + Piedmont + El Cerrito + Kensington + Richmond Annex; three POS
// triggers (property sale + $100K remodel + water meter change);
// Compliance Certificate 20-year validity for complete replacement /
// 7-year for repair; $4,500 Time Extension Certificate deposit for
// 6-month grace period; property owner responsible for ENTIRE
// lateral from home to public main except Alameda + Albany where
// responsibility ends at property line/curbside cleanout); Berkeley
// Municipal Code Chapter 17.16 PSL Program (effective November 3,
// 2014, separate program from EBMUD); Massachusetts (M.G.L. c. 83
// § 7 owner-maintenance duty + M.G.L. c. 21 § 26-53 Clean Waters Act
// + 314 C.M.R. 12.00 sewer use regulations); Default (common-law
// implied warranty of habitability per Hilder v. St. Peter, 478 A.2d
// 202 (Vt. 1984) + Green v. Superior Court, 10 Cal. 3d 616 (1974) +
// Cal. Civ. Code § 1941.1 + Clean Water Act 33 U.S.C. § 1342 NPDES).
// Five universal failure-mode liabilities: tree-root intrusion
// (leading cause); aged clay (1900-1965) or Orangeburg (1945-1972)
// or cast-iron (1900-1980) pipe collapse ($5K-$15K trenchless CIPP
// or $15K-$40K open-cut); stormwater inflow and infiltration (I&I)
// Clean Water Act violation; cross-connection with storm drain
// illicit discharge; failure to obtain POS Compliance Certificate
// at EBMUD-region sale. Inspection methodology: video camera scope
// + hydro-jet cleaning + air-pressure test + dye/smoke test ($400-
// $1,000 per inspection); 5-year general best-practice interval.
// Distinct from siblings rental_septic_system_disclosure (iter 465
// — OSTDS for non-municipal sewer), rental_basement_water_intrusion_
// disclosure, rental_water_submetering_disclosure, rent_abatement_
// construction_nuisance, mid_tenancy_temporary_relocation, tenant_
// emotional_distress_damages.
// ---------------------------------------------------------------------------

async fn rental_sewer_lateral_responsibility_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalSewerLateralResponsibilityInput>,
) -> Result<Json<RentalSewerLateralResponsibilityResult>, ApiError> {
    Ok(Json(check_rental_sewer_lateral_responsibility(&b)))
}

// ---------------------------------------------------------------------------
// rental_sex_offender_registry_notice: Rental property sex offender
// registry notice disclosure compliance — when must a trader-landlord
// include a statutory Megan's Law notice in residential rental
// agreements, and what restrictions apply to the landlord's use of
// registry information? Mounted at POST /api/rental/rental-sex-
// offender-registry-notice. Three regimes: California Cal. Civ. Code
// § 2079.10a + Cal. Pen. Code § 290.46 + Cal. Gov. Code § 12955
// (every residential rental agreement must include exact statutory
// Megan's Law notice directing tenant to www.meganslaw.ca.gov;
// disclosure BEFORE tenant signs lease in at least 10-point type;
// landlord CANNOT use registry information to deny tenancy or
// evict; per-violation statutory damages + tenant rescission right);
// New Jersey N.J.S.A. 2C:7-21 + 2C:7-2 + NJ Attorney General
// Guidelines + NJ LAD (NJ does NOT require landlord lease
// disclosure; three-tier community notification framework — Tier 1
// law enforcement / Tier 2 schools-community / Tier 3 broad
// public; landlords prohibited from using Tier 1 / Tier 2
// information to deny tenancy; Tier 3 broad public notice
// permitted); Default Adam Walsh Act SORNA (42 USC § 16901 et
// seq.) + Fair Housing Act + HUD 2016 Guidance on Criminal History
// (no federal mandate; disparate-impact analysis applies to blanket
// registry-based denials). Distinct from siblings fair_chance_
// housing, rental_application_denial_disclosure, tenant_data_
// privacy.
// ---------------------------------------------------------------------------

async fn rental_sex_offender_registry_notice_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalSexOffenderRegistryNoticeInput>,
) -> Result<Json<RentalSexOffenderRegistryNoticeResult>, ApiError> {
    Ok(Json(check_rental_sex_offender_registry_notice(&b)))
}

// ---------------------------------------------------------------------------
// rental_sinkhole_disclosure: Rental property sinkhole disclosure
// compliance — when a trader-landlord operating Florida (or other
// karst-prone) rental property must disclose past sinkhole claims,
// paid insurance proceeds, and known sinkhole conditions. Mounted at
// POST /api/rental/rental-sinkhole-disclosure. Two regimes: Florida
// FL Statute § 627.7073(1)(c) (seller of property with paid sinkhole
// claim must disclose to buyer BEFORE CLOSING + whether full
// proceeds used for repair) + § 627.707 (professional engineer or
// geologist report and certification) + § 627.706 (sinkhole loss
// definition) + § 689.25 (narrow disclosure exemptions for homicide/
// suicide/HIV) + § 689.261 (Florida Property Tax Disclosure
// Summary) + Johnson v. Davis 480 So. 2d 625 (Fla. 1985) common-
// law material fact disclosure doctrine; Default common-law material
// fact doctrine + PA Real Estate Seller Disclosure Law 68 Pa.C.S. §
// 7301 + karst-prone state common-law warranty of habitability.
// Distinct from siblings rental_underground_storage_tank_disclosure
// (UST), rental_basement_water_intrusion_disclosure (water/mold),
// flood_disclosure, radon_disclosure.
// ---------------------------------------------------------------------------

async fn rental_sinkhole_disclosure_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalSinkholeDisclosureInput>,
) -> Result<Json<RentalSinkholeDisclosureResult>, ApiError> {
    Ok(Json(check_rental_sinkhole_disclosure(&b)))
}

// ---------------------------------------------------------------------------
// rental_smoke_free_housing_disclosure: multi-jurisdictional residential
// rental smoke-free housing disclosure framework — Cal. Civ. Code
// § 1947.5 (SB 332 of 2011, effective January 1, 2012) — landlord MAY
// prohibit smoking; § 1947.5(b) post-2012 leases must include in-lease
// provision specifying prohibited areas; § 1947.5(c) pre-2012 leases
// require § 827 change-of-terms notice; covers cigarettes + cigars +
// pipes + e-cigarettes (added 2017 per SB 5). HUD 24 CFR § 965.653 +
// Part 965 Subpart G + Part 966 Subpart G (final rule November 30,
// 2016 at 81 Fed. Reg. 87430; mandatory implementation deadline July
// 31, 2018) — PHAs MUST prohibit smoking in (1) living units; (2)
// interior areas; (3) outdoor areas within 25 FEET of buildings;
// compassionate enforcement (single incident NOT grounds for
// eviction). NY MDL § 17 + § 17-101 + § 17-179 + NY Public Health
// Law § 1399-n — buildings of 3+ dwelling units must adopt and
// disclose written smoking policy. Mass. G.L. c. 270 § 22 + § 22A
// Smoke-Free Workplace Law — common areas of multifamily buildings
// constituting workplaces must be smoke-free; landlord must post
// signs. Default — common-law nuisance + breach of warranty of
// habitability available regardless of state regime. Mounted at POST
// /api/rental/rental-smoke-free-housing-disclosure. Trader-landlord
// critical because (1) modern shift toward smoke-free for insurance
// discounts + tenant health; (2) failure to disclose can expose to
// constructive-eviction claims; (3) HUD rule applies to LIHTC
// trader-landlord investments. Sibling cluster:
// rental_carbon_monoxide_detector,
// rental_pesticide_application_notification, tenant_data_privacy,
// landlord_emergency_entry_notice.
// ---------------------------------------------------------------------------

async fn rental_smoke_free_housing_disclosure_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalSmokeFreeHousingDisclosureInput>,
) -> Result<Json<RentalSmokeFreeHousingDisclosureResult>, ApiError> {
    Ok(Json(check_rental_smoke_free_housing_disclosure(&b)))
}

// ---------------------------------------------------------------------------
// rental_soft_story_seismic_retrofit: California wood-frame soft-
// story mandatory seismic retrofit compliance. LA Ordinance 183893
// (2015, "Earthquake Hazard Reduction in Existing Wood-Frame
// Buildings with Soft, Weak, or Open-Front Walls") — most ambitious
// US mandatory retrofit program covering ~13,500 buildings. Priority
// 1 (3+ stories ground-floor commercial) deadline April 2024
// (PASSED); Priority 2 (smaller wood-frame under 16 units) deadline
// April 2026. SF Building Code Chapter 34B / 4D / 5E (operative Jun
// 17, 2013) — wood-frame 3+ stories or 2 over basement, 5+ dwelling
// units, built before Jan 1, 1978. Four tier deadlines all passed
// by Sep 15, 2021 — no further extension available. Distinct from
// sibling rental_balcony_inspection_seismic_safety (SB 721 EEE
// inspection, AB 2579 deadline extension), rental_emergency_action_
// plan_high_rise (NYC LL 26 / FDNY EAP). Endpoint at
// /api/rental/rental-soft-story-seismic-retrofit.
// ---------------------------------------------------------------------------

async fn rental_soft_story_seismic_retrofit_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalSoftStorySeismicRetrofitInput>,
) -> Result<Json<RentalSoftStorySeismicRetrofitResult>, ApiError> {
    Ok(Json(check_rental_soft_story_seismic_retrofit(&b)))
}

// ---------------------------------------------------------------------------
// rental_swimming_pool_drain_safety: Rental property swimming pool
// drain safety compliance — when a trader-landlord operating a
// multifamily building with pool or spa must comply with federal
// Virginia Graeme Baker Pool and Spa Safety Act (VGB Act) drain
// cover + anti-entrapment requirements. Mounted at POST /api/rental/
// rental-swimming-pool-drain-safety. Three regimes: Federal VGB Act
// of 2007 (Pub. L. 110-140 EISA Title 14, eff. December 19, 2008;
// 15 USC §§ 8001-8008) — applies to apartment complexes as public
// pools; ASME/ANSI A112.19.8-2007 drain covers + ANSI/APSP/ICC-16
// successor standard; single-drain pools require one of six
// secondary anti-entrapment safeguards (separated drains / SVRS /
// vent system / gravity drainage / automatic pump shutoff / CPSC-
// approved equivalent); $120,000/violation CPSC civil penalty (15
// USC § 2069); California Cal. Health & Safety Code § 116064.1 +
// § 115922 + § 116064.4 (incorporates VGB Act + adds 5-foot pool
// fence + self-latching gate requirements; CDPH enforcement); SB
// 442 of 2017 strengthened residential pool safety; Florida Building
// Code § 454.2.17 + FL Statute § 514.0315 (FL Department of Health
// enforcement + drain cover recertification cycle). Private
// residential pools NOT covered by federal VGB Act. Distinct from
// siblings swimming_pool_safety (general pool fencing/barrier
// framework), rental_carbon_monoxide_detector, rental_bedroom_
// egress_window.
// ---------------------------------------------------------------------------

async fn rental_swimming_pool_drain_safety_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalSwimmingPoolDrainSafetyInput>,
) -> Result<Json<RentalSwimmingPoolDrainSafetyResult>, ApiError> {
    Ok(Json(check_rental_swimming_pool_drain_safety(&b)))
}

// ---------------------------------------------------------------------------
// rental_underground_storage_tank_disclosure: Rental property
// underground storage tank (UST) disclosure compliance — when a
// trader-landlord operating a property with an active, inactive, or
// abandoned-in-place UST must disclose UST presence to tenants and
// buyers. Mounted at POST /api/rental/rental-underground-storage-
// tank-disclosure. Four regimes: Federal RCRA Subtitle I (42 USC §
// 6991 et seq.) + 40 CFR Part 280 EPA UST regulations (heating oil
// residential on-premises EXEMPT from federal; 2015 EPA final rule
// integrity testing); California Cal. Health & Safety Code §§
// 25280-25299.8 Chapter 6.7 + § 25288 annual inspection (CUPAs +
// Cal. Civ. Code § 1102 TDS at sale + common-law fraud); Florida
// FL Statute §§ 376.30-376.317 + § 689.25 + Johnson v. Davis, 480
// So. 2d 625 (Fla. 1985) material fact disclosure doctrine + FDEP
// administration + Petroleum Liability Insurance and Restoration
// Program); New Jersey N.J.A.C. 7:14B (STRICTEST — Property
// Condition Disclosure Statement requires disclosure of UST whether
// active/inactive/abandoned in place + NJ Spill Compensation and
// Control Act N.J.S.A. 58:10-23.11 strict joint-and-several
// liability + NJEDA PUSTP up to $250,000 grant). Distinct from
// siblings rental_basement_water_intrusion_disclosure (water/mold),
// radon_disclosure, asbestos_disclosure, mold_disclosure.
// ---------------------------------------------------------------------------

async fn rental_underground_storage_tank_disclosure_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalUndergroundStorageTankDisclosureInput>,
) -> Result<Json<RentalUndergroundStorageTankDisclosureResult>, ApiError> {
    Ok(Json(check_rental_underground_storage_tank_disclosure(&b)))
}

// ---------------------------------------------------------------------------
// rental_satellite_dish_installation_right: Tenant satellite dish /
// OTARD antenna installation right compliance — when may a trader-
// landlord restrict tenant installation of a satellite dish or
// over-the-air antenna? Mounted at POST /api/rental/rental-satellite-
// dish-installation-right. Federal FCC OTARD Rule (47 CFR § 1.4000,
// eff. October 1996) PREEMPTS state law, local ordinances, building
// codes, HOA covenants, AND lease provisions that impair installation
// on areas under tenant EXCLUSIVE USE OR CONTROL. § 1.4000(a)(1)
// covered antennas: (i) DBS dish 1 meter or less (any size in
// Alaska); (ii) BRS antenna 1 meter or less; (iii) TV broadcast
// antenna no size limit. § 1.4000(a)(2) preemption ONLY applies to
// installation on EXCLUSIVE USE OR CONTROL areas (balcony + patio +
// exclusive-use yard); landlord MAY prohibit installation on common
// areas (exterior walls + roof + shared corridors). § 1.4000(a)(3)
// permissible restrictions narrowly tailored ONLY if necessary to
// (1) accomplish clearly defined legitimate safety objective OR (2)
// preserve historic district designated under federal/state/local
// law. § 1.4000(d) federal preemption applies retroactively. §
// 1.4000(f) cost-impairment doctrine — restriction impairs
// installation if (1) unreasonably delays/prevents; (2) unreasonably
// increases cost; OR (3) precludes acceptable quality reception.
// Tenant enforcement via FCC Petition for Declaratory Ruling or
// private federal/state court action. Statutory authority:
// Telecommunications Act of 1996 § 207 + 47 USC § 303. Distinct from
// siblings rental_broadband_mte_rules (cable/broadband building
// access), rental_carbon_monoxide_detector, tenant_data_privacy.
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// rental_san_francisco_rent_ordinance_chapter_37: SF Residential Rent
// Stabilization and Arbitration Ordinance — one of the strictest and
// most-litigated rent-control regimes in the US, enacted 1979 by SF
// Board of Supervisors (Ordinance 276-79) after Proposition R.
// Codified at SF Administrative Code Chapter 37. Combines (a) annual
// rent-increase cap = LESSER of 60% × published SF CPI increase OR 7%
// absolute ceiling, AND (b) 16-ground just-cause eviction regime
// under § 37.9(a), administered by the SF Rent Board. Coverage shaped
// by certificate-of-occupancy cutoff of June 13, 1979 (post-cutoff
// buildings exempt from rent-price control but covered by just-cause
// eviction) and by Costa-Hawkins Rental Housing Act of 1995 (CA state
// law) which exempts single-family homes and condominium units from
// local rent-price control on tenancy initiation. 16 just-cause
// grounds: non-payment / breach / nuisance / illegal use / refused
// lease extension / denied access / unapproved subtenant / owner-
// move-in / condo conversion / demolition / capital improvement /
// substantial rehab / Ellis Act / lead abatement / Chapter 56 dev
// agreement demolition / Planning Code § 317 demolition. § 37.9(c)
// requires termination notice to be filed with Rent Board within 10
// days of service on tenant; failure invalidates the termination
// notice. § 37.7 capital improvement passthrough cap of 5-10% of base
// rent depending on property size; Rent Board petition + approval
// required. § 37.9F prohibits circumvention via harassment /
// constructive eviction / threats / refusal to accept rent with
// separate civil cause of action including attorney fees and treble
// damages for willful conduct. 15-mode severity ladder × 2 property
// jurisdictions × 2 certificate-of-occupancy date statuses × 4 unit
// types × 6 compliance aspects × 17 just-cause grounds (16 + none) ×
// variable rent / CPI / passthrough / notice-filing inputs.
// Sibling cluster: rental_just_cause_eviction (multi-state base
// regime; SF = strictest US local just-cause regime), rental_
// seattle_smc_22_206_160_just_cause_eviction (iter 669 — Seattle JCEO
// = OLDEST municipal counterpart, 1980), rental_california_sb_567_
// no_fault_eviction_amendments (iter 673 — CA AB 1482 + SB 567
// statewide overlay; SF Chapter 37 retains local supremacy in SF
// itself), rental_california_ab_12_security_deposit_cap (iter 645 —
// CA security deposit cap), rental_california_ab_2347_unlawful_
// detainer_response (iter 667 — CA UD response time), rental_new_
// jersey_anti_eviction_act (iter 651 — NJ statewide companion),
// rental_rent_control_stabilization (multi-state regime), rental_
// owner_move_in_eviction (OMI cross-reference), rental_demolition_
// tenant_notice (demolition cross-reference; SF Chapter 37 includes
// 4 distinct demolition just-cause categories: § 37.9(a)(10), (15),
// (16), and substantial rehab (12)), rental_condominium_conversion_
// protection (condo conversion cross-reference).
// ---------------------------------------------------------------------------

async fn rental_san_francisco_rent_ordinance_chapter_37_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalSanFranciscoRentOrdinanceChapter37Input>,
) -> Result<Json<RentalSanFranciscoRentOrdinanceChapter37Result>, ApiError> {
    Ok(Json(check_rental_san_francisco_rent_ordinance_chapter_37(&b)))
}

async fn rental_satellite_dish_installation_right_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalSatelliteDishInstallationRightInput>,
) -> Result<Json<RentalSatelliteDishInstallationRightResult>, ApiError> {
    Ok(Json(check_rental_satellite_dish_installation_right(&b)))
}

// ---------------------------------------------------------------------------
// rental_security_deposit_interest: multi-jurisdictional security deposit
// interest framework — Chicago RLTO § 5-12-080(c)/(f) + § 5-12-081 (2026
// rate 0.01%, > 6 months held, 30 days post-12-month-period, STRICT-LIABILITY
// 2x deposit damages); Mass. G.L. c. 186 § 15B(3)(a)/(b) + § 15B(6)(e) +
// § 15B(7) (5% OR actual lesser, 30 days post-termination, STRICT-LIABILITY
// TREBLE DAMAGES + costs + attorney fees per MA SJC precedent); Conn. Gen.
// Stat. § 47a-21(i)/(j) + § 47a-21(d)(2) (avg savings deposit rate set
// quarterly by Banking Commissioner, DOUBLE damages + $100 + attorney fees
// for retention beyond 30 days after demand); N.J.S.A. 46:8-19 + § 46:8-21.2
// (1.5 months' rent cap, NJ interest-bearing institution, 1% per annum
// landlord admin fee, annual payment); N.Y. Gen. Oblig. Law § 7-103(1)/(2)
// (TRUST hold always, interest-bearing NY-chartered bank for 6+ unit
// buildings, 1% per annum landlord admin fee in lieu of all custodial
// expenses). Mounted at POST /api/rental/rental-security-deposit-interest.
// Trader-landlord critical because strict-liability double + treble damages
// are among the steepest tenant remedies in landlord-tenant law. Sibling
// cluster: landlord_annual_rent_statement,
// tenant_rent_judgment_wage_garnishment, rental_property_registration,
// landlord_identification_disclosure.
// ---------------------------------------------------------------------------

async fn rental_security_deposit_interest_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalSecurityDepositInterestInput>,
) -> Result<Json<RentalSecurityDepositInterestResult>, ApiError> {
    Ok(Json(check_rental_security_deposit_interest(&b)))
}

// ---------------------------------------------------------------------------
// rental_pesticide_application_notification: multi-jurisdictional
// residential rental pesticide application notification framework — Cal.
// Civ. Code § 1940.8.5 (SB 328 of 2015, effective January 1, 2016) — 24
// hours pre-application notice with 5 delivery methods (first-class mail,
// personal delivery, under-door, electronic, posted); § 1940.8.5(d)
// emergency exception (immediate threat → post-notice within 1 hour);
// § 1940.8.5(e) tenant-requested oral agreement exception (must include
// brand name); NY ECL 33-1004 + 33-1005 (Pesticide Neighbor Notification
// Law) — 48-hour notice for commercial lawn applications + visual
// notification markers + 2020 amendment English+Spanish requirement; NY
// schools 48-hour registry + daycare common-area posting; Mass. G.L.
// c. 132B § 6F + § 6I (Children and Families Protection Act of 2000) —
// 48-hour notice + visual markers + IPM plans for schools/daycare;
// Default — federal Worker Protection Standard (40 CFR Part 170) only
// applies to agricultural applications; absence of state statute does
// not preclude common-law negligence claims for chemical exposure
// injuries. Mounted at POST /api/rental/rental-pesticide-application-
// notification. Trader-landlord critical because pesticide application
// is routine maintenance (ant colonies, cockroach extermination, termite
// treatment, mosquito spraying, rodent bait) but failure to satisfy
// state pre-application notice requirements creates per-violation civil
// exposure PLUS tenant common-law tort claims for chemical exposure
// injuries (especially chemical sensitivity, asthma, pregnancy). Sibling
// cluster: rental_carbon_monoxide_detector,
// rental_organic_waste_collection_disclosure,
// rental_basement_water_intrusion_disclosure,
// landlord_emergency_entry_notice.
// ---------------------------------------------------------------------------

async fn rental_pesticide_application_notification_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalPesticideApplicationNotificationInput>,
) -> Result<Json<RentalPesticideApplicationNotificationResult>, ApiError> {
    Ok(Json(check_rental_pesticide_application_notification(&b)))
}

// ---------------------------------------------------------------------------
// rental_pet_deposit_separate_security: Rental property pet deposit
// and separate security charge compliance — when a trader-landlord
// wants to collect a pet deposit, pet rent, or pet fee separate
// from the general security deposit, what statutory caps +
// disclosure + refundability rules apply? Mounted at POST /api/
// rental/rental-pet-deposit-separate-security. Four regimes:
// California Cal. Civ. Code § 1950.5 + AB 12 of 2023 (eff. July 1,
// 2024 — folds pet deposit into single security deposit cap; 1
// month standard / 2 months small-landlord exception) + SB 611 of
// 2023 (eff. July 1, 2025 — prohibits military surcharge; requires
// itemized disclosure); Washington RCW 59.18.260 (refundable pet
// damage deposit capped at $150) + RCW 59.18.285 (non-refundable
// pet fee permitted if CLEARLY LABELED non-refundable and separate
// from security deposit); New York GOL § 7-103 (HSTPA of 2019;
// ONE security deposit max one month's rent, fully refundable; NO
// separate pet deposit permitted) + NYC Admin Code § 27-2009.1
// (NYC Pet Law) + NY rent stabilization (monthly pet rent
// PROHIBITED in rent-stabilized apartments); Texas Tex. Prop. Code
// §§ 92.101-92.110 (no statutory cap; most permissive regime —
// simultaneous pet deposit + fee + rent permitted). Distinct from
// siblings pet_fees, security_deposit_caps, rental_application_
// denial_disclosure.
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// rental_positive_rent_payment_credit_reporting: California AB 2747
// (Cal. Civ. Code § 1954.06) — effective Jan 1, 2025; key offer
// requirements begin Apr 1, 2025. Covered landlords must OFFER
// tenants the option of positive rental payment information
// reporting to at least one nationwide consumer reporting agency.
// Small-landlord exemption: ≤ 15 dwelling units; exemption
// disappears if landlord owns > 1 building AND is REIT, corporation,
// or LLC with at least one corporate member. New leases on/after
// Apr 1, 2025: offer at lease + annually. Pre-existing leases
// outstanding Jan 1, 2025: offer no later than Apr 1, 2025 +
// annually. Fee cap: LESSER of $10/month OR actual landlord cost;
// no fee if landlord incurs no actual cost. Thirteen-mode severity
// ladder × two jurisdictions × four ownership structures × two
// lease timings × four tenant election states. Trader-landlord
// critical because California's largest residential markets
// (SF / LA / SD) are dominated by ≥ 16-unit buildings or
// REIT/corporate-LLC multi-building owners — most California
// trader-landlords are covered.
// ---------------------------------------------------------------------------

async fn rental_positive_rent_payment_credit_reporting_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalPositiveRentPaymentCreditReportingInput>,
) -> Result<Json<RentalPositiveRentPaymentCreditReportingResult>, ApiError> {
    Ok(Json(check_rental_positive_rent_payment_credit_reporting(&b)))
}

async fn rental_pet_deposit_separate_security_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalPetDepositSeparateSecurityInput>,
) -> Result<Json<RentalPetDepositSeparateSecurityResult>, ApiError> {
    Ok(Json(check_rental_pet_deposit_separate_security(&b)))
}

// ---------------------------------------------------------------------------
// rental_propane_tank_lease_disclosure: Multi-jurisdictional rental
// property propane (Liquefied Petroleum Gas / LP-Gas) tank disclosure
// and compliance framework. When a landlord rents a property served
// by a propane tank (either leased from supplier or owned outright),
// what disclosure must be given to tenant, what NFPA 58 installation/
// clearance/venting standards apply, and what failure-mode liabilities
// expose landlord after a leak, fire, or carbon monoxide event?
// Mounted at POST /api/rental/rental-propane-tank-lease-disclosure.
// Three-jurisdiction framework: Massachusetts (MOST PRESCRIPTIVE —
// 248 C.M.R. 8.00 amendments to NFPA 58 imposed by Mass. Board of
// Fire Prevention Regulations + M.G.L. c. 148 § 9 gas-fitter
// licensing + M.G.L. c. 142 § 1 plumbing/gas authority); New York
// (19 NYCRR Department of State Uniform Code adopting NFPA 58 by
// reference; NY courts hold property owners and service providers
// responsible for personal injury + property damage + environmental
// harm; insurance compliance condition of coverage); Default (NFPA
// 58 Liquefied Petroleum Gas Code 2024 edition + DOT 49 C.F.R. Part
// 173 transport + 49 C.F.R. Part 192 pipeline integrity + 13 VAC
// 5-52-580 IFC Chapter 61 LP gases + common-law habitability per
// Hilder v. St. Peter, 478 A.2d 202 (Vt. 1984) + Green v. Superior
// Court, 10 Cal. 3d 616 (1974) + Cal. Civ. Code § 1941.1 implied
// warranty). Two ownership models: LEASED TANK (most common
// residential, tank owned by Suburban Propane/AmeriGas/Ferrellgas/
// etc., customer pays lease fee plus fuel, supplier retains refill
// access, customer may be LOCKED TO ORIGINAL SUPPLIER with tank
// removal fees $500-$1500; supplier typically responsible for NFPA
// 58 compliance + inspection); OWNED TANK (customer purchased
// outright, full supplier-switching freedom, owner responsible for
// NFPA 58 compliance + inspection + maintenance + 20-30 year
// lifecycle). NFPA 58 (2024 edition) above-ground clearance: 0-125
// gallon residential 0-foot building if ASME-spec + 10-foot ignition
// source; 125-500 gallon 10-foot building + 25-foot property line;
// 500-2000 gallon 10-foot building + 50-foot property line;
// underground requires cathodic protection. Five universal failure-
// mode liabilities: tank leak (fire/explosion + tort negligence +
// emergency relocation); improper venting/back-draft (CO poisoning
// cross-references rental_carbon_monoxide_detector); frost-heave
// dislocation (corrosion + cathodic protection failure); tenant
// refill obstruction; unauthorized supplier switch on leased tank
// (contract breach + tank removal). Distinct from siblings
// rental_gas_appliance_ban (electrification), rental_chimney_
// fireplace_inspection_disclosure (iter 471), rental_carbon_monoxide_
// detector, rental_fire_extinguisher_requirement (iter 473), rental_
// underground_storage_tank_disclosure (UST/LUST petroleum),
// mid_tenancy_temporary_relocation, tenant_emotional_distress_damages.
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// rental_pre_foreclosure_tenant_notification: multi-state pre-
// foreclosure tenant notification compliance. CA Civ Code § 2924.85
// + § 2923.5 (Homeowner Bill of Rights AB 278 of 2012) — 30-day
// pre-foreclosure servicer contact; § 2924.8 trustee tenant notice
// within 5 business days of NOTS. NY RPAPL § 1304 (90-day pre-
// foreclosure notice to borrower) + § 1303 (10-day tenant notice
// after summons and complaint). Illinois 735 ILCS 5/15-1701
// (Foreclosure Fairness Act) — possessory order requirements +
// bona fide lease protection. Washington RCW 61.24.143 (trustee
// sale notice 90+ days before sale) + § 61.24.146 (60-day notice
// to vacate). Massachusetts G.L. c. 244 § 35C (new-owner notice
// posting; 30-day wait before rent-nonpayment eviction; 90-day
// notice to quit). Federal PTFA (P.L. 111-22 of 2009; made
// permanent by Dodd-Frank 2018) 90-day federal floor. Seventeen-
// mode severity ladder × six jurisdictions × six foreclosure
// stages × three lease statuses. Trader-landlord critical: cross-
// state leveraged portfolios face multi-state foreclosure-tenant-
// notification cascade with non-uniform notice windows + content.
// ---------------------------------------------------------------------------

async fn rental_pre_foreclosure_tenant_notification_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalPreForeclosureTenantNotificationInput>,
) -> Result<Json<RentalPreForeclosureTenantNotificationResult>, ApiError> {
    Ok(Json(check_rental_pre_foreclosure_tenant_notification(&b)))
}

async fn rental_propane_tank_lease_disclosure_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalPropaneTankLeaseDisclosureInput>,
) -> Result<Json<RentalPropaneTankLeaseDisclosureResult>, ApiError> {
    Ok(Json(check_rental_propane_tank_lease_disclosure(&b)))
}

// ---------------------------------------------------------------------------
// rental_lead_pipe_disclosure: Rental property lead service line /
// lead pipe disclosure compliance — when a trader-landlord operating
// a property served by a water service line containing lead must
// notify tenants of lead service line presence and replacement plans
// under EPA Lead and Copper Rule Revisions (LCRR, eff. October 16,
// 2024) and Lead and Copper Rule Improvements (LCRI, eff. November
// 1, 2027). Mounted at POST /api/rental/rental-lead-pipe-disclosure.
// Three regimes: Federal 40 CFR Part 141 Subpart I + Safe Drinking
// Water Act 42 USC § 300f et seq. (LCRR service line inventory + 24-
// hour Tier 1 public notification + LCRI 10 ppb action level + 2037
// replacement mandate); Illinois 415 ILCS 5/17.12 + IL EPA Act § 42
// (30-day tenant notice + $50,000 penalty cap); New Jersey N.J.S.A.
// 58:12A-40 et seq. P.L. 2021 c.183 + N.J.S.A. 58:10A-10 (pre-lease
// disclosure + $50K per day penalty + 2031 replacement deadline).
// Distinct from siblings rental_underground_storage_tank_disclosure
// (UST), rental_basement_water_intrusion_disclosure (water/mold),
// rental_sinkhole_disclosure, federal § 4852d lead-based paint
// disclosure (paint not pipes).
// ---------------------------------------------------------------------------

async fn rental_lead_pipe_disclosure_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalLeadPipeDisclosureInput>,
) -> Result<Json<RentalLeadPipeDisclosureResult>, ApiError> {
    Ok(Json(check_rental_lead_pipe_disclosure(&b)))
}

// ---------------------------------------------------------------------------
// rental_lead_paint_disclosure: federal Title X § 1018 / 42 U.S.C.
// § 4852d / 24 C.F.R. Part 35 Subpart A / 40 C.F.R. Part 745 Subpart F
// pre-lease lead-based paint disclosure compliance for any pre-1978
// target housing. Universal federal landlord obligation: five disclosure
// elements (Lead Warning Statement verbatim in lease, known LBP
// disclosed or no-knowledge stated, available reports provided, EPA
// "Protect Your Family From Lead in Your Home" pamphlet EPA-747-K-12-001
// delivered, tenant signed acknowledgment). Exemptions per 24 C.F.R.
// § 35.82(b): zero-bedroom dwellings (efficiencies/lofts/dorms) unless
// child under 6 expected; short-term leases ≤ 100 days with no
// renewal/extension; elderly-only or disabled-only housing unless child
// under 6 expected; post-1977 housing. Civil penalty up to $22,263 per
// violation under 42 U.S.C. § 4852d(b)(5) (2025 inflation-adjusted) +
// joint-and-several liability for treble actual damages under
// § 4852d(b)(3). Sibling to rental_lead_pipe_disclosure (lead service
// line water), rental_post_construction_lead_dust_clearance (post-
// renovation dust-wipe clearance), rental_natural_gas_leak_response
// (pre-1978 housing often has gas appliances), rental_radon_mitigation_
// disclosure (other federal toxic-exposure disclosure).
// ---------------------------------------------------------------------------

async fn rental_lead_paint_disclosure_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<LeadPaintDisclosureInput>,
) -> Result<Json<LeadPaintDisclosureResult>, ApiError> {
    Ok(Json(check_rental_lead_paint_disclosure(&b)))
}

// ---------------------------------------------------------------------------
// rental_natural_gas_leak_response: Multi-jurisdictional rental
// property natural gas leak detection, response, and landlord-duty
// compliance framework. When a tenant reports a natural-gas odor
// (rotten egg / sulfur from mercaptan odorant) or a methane-leak event
// occurs in or around the rental unit, what immediate response duties
// attach (evacuate + 911 + utility + do not ignite), what utility-side
// and landlord-side obligations interact under PHMSA 49 C.F.R. Part
// 192, and what failure-mode liabilities expose landlord after an
// explosion, fire, asphyxiation, or carbon-monoxide event? Mounted at
// POST /api/rental/rental-natural-gas-leak-response. Four-jurisdiction
// framework: Federal/PHMSA (universal floor — 49 C.F.R. Part 192 +
// § 192.625 odorization with mercaptan + § 192.706 15-month leakage
// surveys + PHMSA Final Rule January 17, 2025 strengthened advanced
// leak detection performance standards and mandatory repair
// timelines); Massachusetts (220 C.M.R. 100.00 + 220 C.M.R. 101.00 +
// M.G.L. c. 164 § 105A post-Merrimack Valley 2018 gas safety regime
// — explosion across Andover/Lawrence/North Andover killed 1 +
// injured 25 + displaced thousands with aggregate settlements
// exceeding $1B; engage Eversource/National Grid/Columbia Gas
// emergency line); California (CPUC G.O. 112-F gas pipeline safety +
// G.O. 58-A natural gas service standards + Cal. Pub. Util. Code
// § 451 safe-and-adequate-service mandate + Cal. Civ. Code § 1941.1
// implied warranty of habitability per Green v. Superior Court, 10
// Cal. 3d 616 (1974); engage PG&E/SoCalGas/SDG&E); Default (common-
// law implied warranty per Hilder v. St. Peter, 478 A.2d 202 (Vt.
// 1984) + tort negligence + premises liability + state PUC consumer-
// protection rules). Universal landlord IMMEDIATE-RESPONSE six-step
// protocol: (1) evacuate all occupants immediately; (2) call 911;
// (3) call utility emergency line FROM SAFE LOCATION outside
// building; (4) DO NOT operate light switches/electronics/vehicle
// ignition/cell phones inside building (spark = explosion); (5) DO
// NOT enter unit to investigate; (6) wait for utility + fire
// department clearance before re-entry. Post-incident obligations:
// engage licensed plumber for appliance inspection + document
// utility report + notify all tenants + update lease addenda + consider
// methane detector ($25-$100 per unit emerging best practice).
// Five universal failure-mode liabilities: failed to respond;
// operated appliances after odor; failed to evacuate; failed to
// inspect post-restoration; missing methane detector. Distinct from
// siblings rental_propane_tank_lease_disclosure (iter 475 — LP-gas
// tank), rental_carbon_monoxide_detector (CO sensor), rental_
// hardwired_smoke_alarm_responsibility (iter 481 — smoke detection),
// rental_chimney_fireplace_inspection_disclosure (iter 471), rental_
// fire_extinguisher_requirement (iter 473), rental_gas_appliance_ban
// (electrification policy), tenant_emotional_distress_damages.
// ---------------------------------------------------------------------------

async fn rental_natural_gas_leak_response_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalNaturalGasLeakResponseInput>,
) -> Result<Json<RentalNaturalGasLeakResponseResult>, ApiError> {
    Ok(Json(check_rental_natural_gas_leak_response(&b)))
}

// ---------------------------------------------------------------------------
// rental_new_jersey_anti_eviction_act: New Jersey Anti-Eviction Act
// (P.L. 1974, c. 49) — codified at N.J.S.A. 2A:18-61.1; OLDEST
// statewide just-cause eviction regime in United States history,
// predating California AB 1482 of 2019 by 45 years. Enumerates 18
// statutory grounds for eviction: (a) failure to pay rent; (b)
// disorderly conduct after notice to cease; (c) willful or grossly
// negligent destruction; (d) substantial lease violation after
// notice to cease; (e) continued violation of landlord rules; (f)
// habitual late payment; (g) refusal of reasonable lease changes at
// renewal; (h) owner retires from rental business; (i) conversion
// to non-residential use; (j) conversion to condominium/cooperative;
// (k) owner permanently moves into unit; (l) owner needs unit for
// family; (m) refusing reasonable lease changes at tenancy end; (n)
// habitual nonpayment; (o) drug-related criminal activity; (p)
// assault/threats/weapons use; (q) theft from premises; (r) other
// specified criminal activity. Owner-occupied 3-or-fewer-apartments
// exemption: Act does NOT apply where building has 3 or fewer
// apartments AND owner lives in one. Notice to Cease required for
// grounds (b)/(d)/(e)/(f). Notice to Quit periods: typically 1
// month; 3 days for criminal activity (o)/(p)/(q)/(r); 18 months
// for owner-occupier conversion grounds (h)/(i)/(k)/(l); 3 years
// for condominium/cooperative conversion (j). N.J.S.A. 2A:18-61.7
// et seq. companion condominium/cooperative conversion regime.
// N.J.S.A. 2A:18-56 prerequisite notice-to-quit-and-demand-for-
// possession for summary dispossess action. Nineteen-mode severity
// ladder × nineteen eviction grounds × three property classifications
// × three notice-to-cease statuses × seven notice-to-quit durations.
// Trader-landlord critical because New Jersey is the OLDEST statewide
// just-cause eviction jurisdiction; HSTPA-style tenant protections
// predate every other state. Sibling cluster: rental_just_cause_
// eviction (multi-state base regime; NJ Anti-Eviction Act = OLDEST
// entry), rental_colorado_hb_24_1098_just_cause_eviction (iter 649
// — Colorado newest entry), rental_oregon_sb_608_sb_611_rent_
// stabilization (iter 647 — Oregon companion), rental_eviction_
// notices (notice to quit cross-reference), rental_owner_move_in_
// eviction (owner move-in cross-reference), rental_condominium_
// conversion_protection (multi-state condo conversion regime).
// ---------------------------------------------------------------------------

async fn rental_new_jersey_anti_eviction_act_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalNewJerseyAntiEvictionActInput>,
) -> Result<Json<RentalNewJerseyAntiEvictionActResult>, ApiError> {
    Ok(Json(check_rental_new_jersey_anti_eviction_act(&b)))
}

// ---------------------------------------------------------------------------
// rental_seattle_smc_22_206_160_just_cause_eviction: Seattle Just
// Cause Eviction Ordinance (JCEO) codified at SMC § 22.206.160.C —
// the FIRST municipal just-cause eviction ordinance in the United
// States, adopted by the Seattle City Council in 1980. Preceded
// New Jersey Anti-Eviction Act statewide regime (1974 was already
// in place at state level) and every subsequent local just-cause
// ordinance (Berkeley, Oakland, San Francisco, Minneapolis,
// Portland). Codified under SMC Title 22 (Building and Construction
// Codes), Subtitle II (Housing Code), Chapter 22.206 (Habitable
// Buildings), Subchapter VI (Duties of Owners and Tenants).
// Applies to all month-to-month tenancies AND lease non-renewals
// on residential property within Seattle city limits. Exempts
// transient lodging, educational institution housing, medical /
// correctional occupancy, and live-aboard vessels. Enumerated just
// causes (between 16 and 18 depending on amendment cycle, 2021
// amendments most recent major package): (1) non-payment of rent
// after 14-day pay-or-vacate notice (RCW 59.18.057); (2) material
// noncompliance after 10-day comply-or-vacate notice; (3) chronic
// late rent — 4+ 14-day pay-or-vacate notices in 12 months +
// 20-day termination; (4) habitual rule violations — 3+ 10-day
// comply-or-vacate notices in 12 months + 20-day termination;
// (5) owner / immediate-family occupancy — 90-day notice + 60
// consecutive days of good-faith occupancy within 90 days of
// vacate; failure triggers $2,000 statutory damages + actual
// damages + attorney fees; (6) sale of single-family dwelling —
// 90-day written notice + list/show within 30 days; failure
// triggers $2,000 tenant damages; (7) substantial rehabilitation /
// demolition / change of use — landlord MUST first obtain Tenant
// Relocation Assistance Ordinance (TRAO) license from SDCI under
// SMC § 22.210 before serving 20-day termination notice;
// (8) condominium / cooperative conversion — 120-day written
// notice under RCW 64.34.440; (9) tenant refused to sign new lease
// with substantially identical terms; (10) criminal activity
// recorded with city substantially affecting other tenants or
// property; (11) owner quitting shared occupancy with tenant in
// same dwelling; (12) transfer to comparable subsidized or rent-
// restricted unit. Twenty-three-mode severity ladder × thirteen
// just-cause assertions × six unit types × three TRAO statuses ×
// three owner-occupancy statuses × variable notice-day input.
// SDCI investigates JCEO complaints; landlord who terminates
// without qualifying just cause subject to civil penalty + tenant
// private right of action for unlawful-detainer defense, statutory
// damages, and reasonable attorney fees. Trader-landlord critical
// because Seattle JCEO (1980) is the founding municipal just-cause
// regime — every subsequent local and state just-cause statute
// builds on the Seattle template; the OMI rebuttable presumption
// and TRAO linkage are landlord-side exposure hotspots. Sibling
// cluster: rental_just_cause_eviction (multi-state base regime;
// Seattle JCEO = OLDEST municipal entry), rental_new_jersey_anti_
// eviction_act (NJ statewide 1974 = OLDEST statewide entry; iter
// 651), rental_colorado_hb_24_1098_just_cause_eviction (CO 2024
// newest statewide; iter 649), rental_oregon_sb_608_sb_611_rent_
// stabilization (OR 2019 statewide; iter 647), rental_california_
// ab_12_security_deposit_cap (CA 2024 companion; iter 645),
// rental_wa_hb_1217_rent_stabilization (WA 2025 statewide companion;
// iter 643), rental_owner_move_in_eviction (OMI cross-reference),
// rental_demolition_tenant_notice (demolition / use-change cross-
// reference), rental_tenant_relocation_assistance (general state-
// level relocation assistance regime), rental_condominium_
// conversion_protection (multi-state condo conversion regime),
// rental_eviction_notices (notice-to-quit cross-reference).
// ---------------------------------------------------------------------------

async fn rental_seattle_smc_22_206_160_just_cause_eviction_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalSeattleSmc22206160JustCauseEvictionInput>,
) -> Result<Json<RentalSeattleSmc22206160JustCauseEvictionResult>, ApiError> {
    Ok(Json(check_rental_seattle_smc_22_206_160_just_cause_eviction(&b)))
}

// ---------------------------------------------------------------------------
// rental_ny_rent_receipt_late_notice_requirements: NY Real Property
// Law § 235-e Duty to Provide Written Receipt + Five-Day Late Rent
// Notice. Three obligations: (1) § 235-e(a) written receipt for cash
// or non-personal-check payments with 5 required elements (date +
// amount + premises + period + signature) — IMMEDIATE if personal,
// 15 days if indirect; (2) § 235-e(b) cash payment records retained
// 3 years; (3) § 235-e(d) HSTPA 2019 amendment — 5-day late rent
// notice by CERTIFIED MAIL (email/text not sufficient) — applies to
// both residential and commercial tenancies; failure = tenant
// affirmative defense in nonpayment proceeding. HSTPA 2019 (Laws of
// 2019, c. 36) overhauled NY landlord-tenant law. NY RPAPL § 711
// nonpayment proceeding cross-reference. Sibling cluster: rental_
// late_fee_cap (cap on late fees), rental_just_cause_eviction (iter
// 573 — Good Cause Eviction NY portion), rental_security_deposit_
// interest (NY 1.5% interest requirement), rental_landlord_notice_
// to_enter (HSTPA-related notice rule).
// ---------------------------------------------------------------------------

async fn rental_ny_rent_receipt_late_notice_requirements_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalNyRentReceiptLateNoticeRequirementsInput>,
) -> Result<Json<RentalNyRentReceiptLateNoticeRequirementsResult>, ApiError> {
    Ok(Json(check_rental_ny_rent_receipt_late_notice_requirements(&b)))
}

// ---------------------------------------------------------------------------
// rental_ny_rpl_235f_roommate_law: NY Real Property Law § 235-f
// (Unlawful Restrictions on Occupancy — Roommate Law). Enacted 1983.
// Permits tenant + immediate family + additional occupants + dependent
// children. Single-tenant lease: tenant + immediate family + 1
// additional occupant + dependent children of occupant. Multi-tenant
// lease: tenants + immediate family + occupants such that total
// (excluding occupants' dependent children) ≤ number of tenants in
// lease. Requires tenant or spouse to use as primary residence.
// § 235-f(4) tenant must inform landlord of occupant name within 30
// days of commencement or landlord request. § 235-f(5) occupants
// acquire no tenancy rights without express written landlord consent.
// § 235-f(6) lease restrictions on occupancy unenforceable as against
// public policy. § 235-f(7) waiver of § 235-f null and void.
// § 235-f(8) MDL § 4(7) overcrowding standard still applies. Sibling:
// rental_nyc_loft_law_article_7c (iter 597 — statutory tenant cross-
// reference), rental_just_cause_eviction (iter 573 — eviction
// restriction overlay), rental_ny_rent_receipt_late_notice_requirements
// (iter 603 — adjacent NY tenant-protection statute).
// ---------------------------------------------------------------------------

async fn rental_ny_rpl_235f_roommate_law_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalNyRpl235FRoommateLawInput>,
) -> Result<Json<RentalNyRpl235FRoommateLawResult>, ApiError> {
    Ok(Json(check_rental_ny_rpl_235f_roommate_law(&b)))
}

// ---------------------------------------------------------------------------
// rental_attorney_fee_clause_reciprocity: residential lease attorney
// fee clause mutualization across CA Civ Code § 1717 (waiver void;
// automatic bilateral reading of unilateral clauses), FL Stat § 83.48
// (Residential Landlord and Tenant Act prevailing-party fees; NOT
// waivable in lease; § 83.51 personal-injury claims EXCLUDED), FL Stat
// § 57.105(7) (general one-way → bilateral statute), NY RPL § 234
// (residential landlord-favor clauses mutualized as tenant covenant by
// landlord), WA RCW 4.84.330 (bilateral reading of unilateral
// contractual fee provisions), OR ORS 20.096 (reciprocal fee
// provisions). Texas enforces unilateral clauses as written without
// statutory mutualization for landlord side. Eight-mode severity
// ladder × seven jurisdictions × six clause types × three prevailing-
// party states. Trader-landlord critical because attorney fees often
// dwarf the underlying claim and the asymmetric drafting reality is
// that landlord-favor unilateral clauses are common in form leases.
// ---------------------------------------------------------------------------

async fn rental_attorney_fee_clause_reciprocity_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalAttorneyFeeClauseReciprocityInput>,
) -> Result<Json<RentalAttorneyFeeClauseReciprocityResult>, ApiError> {
    Ok(Json(check_rental_attorney_fee_clause_reciprocity(&b)))
}

// ---------------------------------------------------------------------------
// rental_nyc_childhood_lead_poisoning_prevention_act: NYC Local Law 1
// of 2004 (Childhood Lead Poisoning Prevention Act) — codified at
// NYC Admin Code § 27-2056 et seq. + 28 RCNY Subchapter K. Covers
// pre-1960 multiple dwellings (3+ units) AND 1960-1977 with known
// lead-based paint where child under 6 resides (lives or routinely
// spends 10+ hours per week). Six core obligations: (1) annual
// notice Jan 1 - Feb 15 to all tenants; (2) annual investigation
// for units with child under 6; (3) turnover inspection at new
// tenancy with child under 6; (4) 21-day remediation of identified
// hazards under § 27-2056.4(g); (5) EPA-certified RRP renovator for
// disturbance over 100 sqft or window replacement under
// § 27-2056.11; (6) 10-year recordkeeping under § 27-2056.14. NYC
// Local Law 31 of 2020: XRF lead-paint testing required by Aug 9,
// 2025 — Class "C" immediately hazardous violation with up to
// $1,500 civil penalty. NYC Local Law 66 of 2019: lowered HUD
// 1.0 mg/cm² threshold to 0.5 mg/cm² (stricter NYC standard) eff.
// Dec 1, 2021. § 27-2125 HPD emergency repair administrative lien
// chargeback if landlord fails to remediate. Sibling cluster:
// rental_lead_paint_disclosure (iter 569 federal Title X § 1018),
// rental_lead_pipe_disclosure, rental_post_construction_lead_dust_
// clearance, rental_facade_inspection_fisp_local_law_11 (iter 583),
// rental_gas_piping_inspection_local_law_152 (iter 585).
// ---------------------------------------------------------------------------

async fn rental_nyc_childhood_lead_poisoning_prevention_act_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalNycChildhoodLeadPoisoningPreventionActInput>,
) -> Result<Json<RentalNycChildhoodLeadPoisoningPreventionActResult>, ApiError> {
    Ok(Json(check_rental_nyc_childhood_lead_poisoning_prevention_act(&b)))
}

// ---------------------------------------------------------------------------
// rental_nyc_loft_law_article_7c: NYC Loft Law (NY Multiple Dwelling
// Law Article 7-C, MDL §§ 280-287). Enacted 1982 (NY Laws of 1982, c.
// 349). Creates Interim Multiple Dwelling (IMD) classification for
// commercial/manufacturing buildings residentially occupied during
// specified eligibility windows. MDL § 281: IMD criteria (former
// commercial/manufacturing + 12 consecutive months residential use
// + 3 or more independent families + not previously legal residential
// w/ CofO). MDL § 281(5) (2010 expansion): added 2008-2009 residential
// occupation window. MDL § 284 code compliance timetable: 12/18/36/48
// months for alteration application + alteration permit + Article 7-B
// safety compliance + residential CofO. MDL § 286 Protected Occupant
// — protection from eviction except good cause + rent stabilization
// after legalization. 29 RCNY Subchapter B (Loft Board Rules) +
// § 2-01.1 Narrative Statement process + § 2-12 civil penalties
// ($500-$1,000+ per violation per day). Subsequent amendments: NY
// Laws of 2013 c. 4 + 2019. Sibling cluster: rental_rent_control_
// stabilization (NY RSL parallel), rental_facade_inspection_fisp_
// local_law_11 (iter 583 NYC LL 11), rental_just_cause_eviction
// (iter 573 — NY Good Cause separate regime).
// ---------------------------------------------------------------------------

async fn rental_nyc_loft_law_article_7c_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalNycLoftLawArticle7CInput>,
) -> Result<Json<RentalNycLoftLawArticle7CResult>, ApiError> {
    Ok(Json(check_rental_nyc_loft_law_article_7c(&b)))
}

// ---------------------------------------------------------------------------
// rental_nyc_scrie_drie_rent_freeze: NYC Senior Citizen Rent
// Increase Exemption (SCRIE; NYC Admin Code § 26-509 / Local Law 6
// of 1986) + Disability Rent Increase Exemption (DRIE; NYC Admin
// Code § 26-509.1 / Local Law 76 of 2005). Both programs
// PERMANENTLY FREEZE tenant rent at prior level; every future RGB
// increase credited back to landlord as property tax abatement on
// DOF Form RP-467-C. SCRIE: tenant 62+; rent-regulated unit (rent-
// stabilized / rent-controlled / Mitchell-Lama / HDFC co-op);
// household income ≤ $50,000 (2025); rent burden > 1/3 monthly
// income. DRIE: tenant 18+ with SSI / SSDI / VA disability
// compensation / Medicaid-based-on-disability determination; same
// $50,000 income cap; same > 1/3 rent burden. Renewal every 24
// months; DOF mails 60-day notice. NY Senate S01457 (2025-2026)
// proposes raising income limit to $75,000 for 2026. Fourteen-
// mode severity ladder × two programs × five regulated unit types
// × five federal disability benefits. Trader-landlord critical
// because SCRIE/DRIE-frozen units create NYC-only property-tax-
// abatement revenue stream that must be properly applied and
// reported; failure to honor freeze creates statutory violations
// and clawback exposure.
// ---------------------------------------------------------------------------

async fn rental_nyc_scrie_drie_rent_freeze_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalNycScrieDrieRentFreezeInput>,
) -> Result<Json<RentalNycScrieDrieRentFreezeResult>, ApiError> {
    Ok(Json(check_rental_nyc_scrie_drie_rent_freeze(&b)))
}

// ---------------------------------------------------------------------------
// rental_nyc_local_law_55_ipm_pest_control: NYC Local Law 55 of
// 2018 (Asthma-Free Housing Act; effective Jan 19, 2019; NYC Admin
// Code § 27-2017 et seq.). Applies to multiple dwellings (3+
// units). Indoor allergen hazards = mice/cockroaches/rats/mold.
// Owner must perform ANNUAL INSPECTION for indoor allergen hazards
// and apply Integrated Pest Management (IPM). § 27-2017.8 — any
// pesticide must be applied by NYS DEC-licensed pest professional.
// Tenant must receive annual notice + DOHMH fact sheet at lease
// signing and renewal. Access notice — 24-hour written notice
// constitutes good-faith effort. HPD enforcement: $10-$125 per
// day; max $10,000; false certification of correction $50-$250
// non-hazardous / $250-$500 hazardous. Fourteen-mode severity
// ladder × three property classifications × six allergen hazards
// × three pesticide operator licenses. Trader-landlord critical
// for NYC portfolio operators with 3+ unit buildings; daily
// penalties accrue + false-certification can compound.
// ---------------------------------------------------------------------------

async fn rental_nyc_local_law_55_ipm_pest_control_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalNycLocalLaw55IpmPestControlInput>,
) -> Result<Json<RentalNycLocalLaw55IpmPestControlResult>, ApiError> {
    Ok(Json(check_rental_nyc_local_law_55_ipm_pest_control(&b)))
}

// ---------------------------------------------------------------------------
// rental_nyc_local_law_18_str_registration: NYC Local Law 18 of 2022
// — the Short-Term Rental Registration Law administered by the NYC
// Office of Special Enforcement (OSE) under the Mayor's Office of
// Criminal Justice. Signed January 9, 2022; enforcement began
// September 5, 2023; codified at NYC Admin Code § 26-3001 through
// § 26-3007 with 31 RCNY Part 12 implementing rules. Reduced active
// NYC STR listings from over 38,000 in early 2023 to approximately
// 3,000 registered listings by mid-2025 — a ~92 % reduction.
// STR definition: rental of dwelling unit or part for fewer than 30
// consecutive days. Hosts must register with OSE BEFORE listing;
// booking platforms (Airbnb, VRBO, Booking.com) prohibited from
// processing transactions for unregistered STRs. Host present
// requirement (§ 26-3001): permanent occupant must be present
// during guest stay sharing dwelling as "common household";
// maximum 2 PAYING GUESTS; interior doors cannot deny guest access.
// Class B Multiple Dwelling exemption (licensed hotels, motels,
// hostels, B&Bs, permitted rooming houses) under NY MDL § 4(8)(a).
// Penalties (§ 26-3006): $100-$5,000 per violation against host;
// up to $1,500 per infraction against booking services; 3× illegal
// revenue collected as financial penalty; registration revocation
// for non-compliance. Prohibited Buildings List (PBL) under 31 RCNY
// Part 12: building owners may apply to add building to PBL
// preventing all STR registrations. Twelve-mode severity ladder ×
// three dwelling classifications × two rental durations × three
// host presence statuses × four registration statuses. Trader-
// landlord critical for NYC Airbnb/VRBO portfolio operators; ~92 %
// active STR supply reduction post-LL18 means operators must
// pivot to long-term (30+ day) rentals or comply with strict host-
// present registration. Sibling cluster: short_term_rental_
// conversion (multi-state STR regime referencing LL 18), rental_
// rent_control_stabilization (long-term rental alternative regime),
// rental_just_cause_eviction, rental_nyc_coop_conversion_eviction_
// protection (iter 641 — NYC companion), rental_nyc_local_law_55_
// ipm_pest_control (sister NYC LL).
// ---------------------------------------------------------------------------

async fn rental_nyc_local_law_18_str_registration_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalNycLocalLaw18StrRegistrationInput>,
) -> Result<Json<RentalNycLocalLaw18StrRegistrationResult>, ApiError> {
    Ok(Json(check_rental_nyc_local_law_18_str_registration(&b)))
}

// ---------------------------------------------------------------------------
// rental_nyc_coop_conversion_eviction_protection: NYC GBL § 352-eeee
// (Martin Act Article 23-A) Cooperative / Condominium Conversion
// Eviction Protection. Governs conversion of NYC rental buildings to
// cooperative or condominium ownership. Eviction Plan: cannot be
// declared effective until 51% of bona fide tenants in occupancy
// execute written purchase agreements; non-purchasing tenants
// evictable at LATER of (a) lease expiration OR (b) 3 years after
// plan declared effective. Non-Eviction Plan: 51% threshold for
// large buildings (post-HSTPA 2019); 15% small-building exception
// for 5-or-fewer-unit buildings where sponsor or immediate family
// occupied a unit for 2+ years. Senior 62+/disabled permanent
// eviction protection under GBL § 352-e(2-a) + § 352-eee + § 352-
// eeee (holders of unsold shares + subsequent purchasers cannot
// evict; owner-occupancy provisions inapplicable). Tenant rights:
// 90-day exclusive purchase right after plan accepted for filing +
// 6-month subsequent right of first refusal. Non-purchasing tenant
// eviction PERMITTED ONLY for non-payment of rent, illegal use of
// premises, or similar breaches; not for failure to purchase or
// expiration of tenancy. Companion statute: GBL § 352-eeeee
// (Westchester/Rockland/Nassau condominium conversions). NY S3758
// + S4910 (2025-2026 session) pending amendments to expand
// senior/disabled protections and adjust subscription thresholds.
// Thirteen-mode severity ladder × four plan types × six tenant
// categories × three building scenarios. Trader-landlord critical
// because conversion plans involve per-unit purchase agreements,
// permanent senior/disabled shields, 3-year grace period, and
// 13 NYCRR Part 18 Attorney General Real Estate Finance Bureau
// regulatory enforcement. Sibling cluster: condominium_conversion_
// protection (multi-state pattern), rental_nyc_loft_law_article_7c
// (iter 597 — IMD statutory tenant protection), rental_nyc_scrie_
// drie_rent_freeze (NYC senior/disabled rent freeze).
// ---------------------------------------------------------------------------

async fn rental_nyc_coop_conversion_eviction_protection_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalNycCoopConversionEvictionProtectionInput>,
) -> Result<Json<RentalNycCoopConversionEvictionProtectionResult>, ApiError> {
    Ok(Json(check_rental_nyc_coop_conversion_eviction_protection(&b)))
}

// ---------------------------------------------------------------------------
// rental_hoa_disclosure_at_lease: Rental property HOA / Condominium
// Association disclosure at lease signing compliance — when a trader-
// landlord renting a unit governed by an HOA or COA must disclose
// association rules, CC&Rs, fee structure, rental restrictions, and
// tenant-information sharing to prospective tenants at lease
// execution and to the association before/after lease execution.
// Mounted at POST /api/rental/rental-hoa-disclosure-at-lease. Three
// regimes: California Cal. Civ. Code § 4740 + § 4525 + § 1102 et
// seq. (Davis-Stirling Common Interest Development Act — landlord
// must provide HOA with tenant name and contact info before lease;
// may redact financial info; pre-acquisition rental prohibitions
// enforceable, post-acquisition grandfathered for existing owners);
// Florida FL Statute § 720.401 + § 718.503 (HOA Disclosure Summary
// before contract execution + condominium rent diversion remedy);
// Nevada Nev. Rev. Stat. § 116 + § 116.335 (Uniform Common-Interest
// Ownership Act — pre-acquisition rental prohibition only;
// association may not require approval to rent unless required at
// time of purchase) + § 118A landlord-tenant framework. Distinct
// from siblings rental_application_denial_disclosure, tenant_data_
// privacy, rental_property_registration, short_term_rental_
// conversion.
// ---------------------------------------------------------------------------

async fn rental_hoa_disclosure_at_lease_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalHoaDisclosureAtLeaseInput>,
) -> Result<Json<RentalHoaDisclosureAtLeaseResult>, ApiError> {
    Ok(Json(check_rental_hoa_disclosure_at_lease(&b)))
}

// ---------------------------------------------------------------------------
// rental_organic_waste_collection_disclosure: Rental property organic
// waste collection disclosure compliance — when must a trader-landlord
// provide organics collection bins, tenant education, and move-in
// disclosure before billing for waste service or claiming compliance
// with state organic waste diversion mandates? Mounted at POST /api/
// rental/rental-organic-waste-collection-disclosure. Four regimes:
// California SB 1383 of 2016 + 14 CCR §§ 18984-18984.13 (eff. January
// 1, 2022; multifamily 5+ units must provide organics containers +
// annual tenant education + new tenant info within 14 days; 75%
// diversion target by 2025); Vermont Universal Recycling Law Act 148
// of 2012 + 10 V.S.A. § 6605k (fully eff. July 1, 2020; bans food
// scraps from landfill statewide; applies to ALL properties
// regardless of unit count; $200-$1,000 first-offense civil penalty
// under § 8007); Seattle Municipal Code Ch. 21.36.082 (eff. January
// 1, 2015; multifamily 5+ units required to participate in compost
// service; $50 per-pickup contamination fine); Default — no statewide
// mandate; local municipal ordinances may impose (NYC LL97 organic
// waste, Boston organic waste pilot); federal RCRA Subtitle D solid
// waste regulations apply. Distinct from siblings rental_energy_
// benchmarking, rental_water_submetering_disclosure, rental_gas_
// appliance_ban.
// ---------------------------------------------------------------------------

async fn rental_organic_waste_collection_disclosure_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalOrganicWasteCollectionDisclosureInput>,
) -> Result<Json<RentalOrganicWasteCollectionDisclosureResult>, ApiError> {
    Ok(Json(check_rental_organic_waste_collection_disclosure(&b)))
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
// landlord_self_help_eviction_prohibition: multi-jurisdictional self-help
// eviction prohibition framework — Cal. Civ. Code § 789.3 ($100/day +
// $250 minimum + actual + attorney fees with INTENT-to-terminate element);
// N.Y. Real Prop. Law § 235 + RPAPL § 853 + RPAPL § 768 (CLASS A
// MISDEMEANOR + TREBLE DAMAGES under RPAPL § 853 + criminal exposure);
// Fla. Stat. § 83.67 (greater of ACTUAL/CONSEQUENTIAL OR 3 MONTHS' RENT
// + attorney fees + costs + IRREPARABLE HARM for injunctive relief +
// SEPARATE awards for non-contemporaneous violations); Tex. Prop. Code
// § 92.0081(h) + § 92.008 (actual + $1,000 + 1 month's rent less actual
// + attorney fees); Default common-law wrongful eviction tort with
// punitive damages in some states. Mounted at POST /api/rental/
// landlord-self-help-eviction-prohibition. Trader-landlord critical
// because the trader-landlord pattern (out-of-state owner + non-paying
// tenant + lost rental income + emotional decision to "just turn off
// the water") matches precisely the fact pattern these statutes were
// designed to deter. Court eviction judgment + marshal's writ of
// possession is the ONLY lawful pathway across all five jurisdictions.
// Sibling cluster: landlord_retaliation_damages,
// landlord_emergency_entry_notice, landlord_lien_prohibition,
// tenant_relocation_assistance.
// ---------------------------------------------------------------------------

async fn landlord_self_help_eviction_prohibition_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<LandlordSelfHelpEvictionInput>,
) -> Result<Json<LandlordSelfHelpEvictionResult>, ApiError> {
    Ok(Json(check_landlord_self_help_eviction_prohibition(&b)))
}


// ---------------------------------------------------------------------------
// landlord_security_device_obligations: Mandatory landlord-provided security
// devices obligations — when does a residential landlord have an affirmative
// statutory duty to install and maintain locks, deadbolts, door viewers, and
// sliding-door security devices? Mounted at POST /api/rental/landlord-
// security-device-obligations. Three regimes: (1) Texas Tex. Prop. Code
// §§ 92.151 + 92.153 + 92.156 + 92.164 + 92.165 Subchapter D — most
// detailed framework: § 92.153(a) keyless bolting + door viewer on each
// exterior door; § 92.153(b) at least ONE door with both keyed and keyless
// deadbolts; § 92.153(c) sliding door pin lock OR handle latch OR security
// bar (for dwellings completed on or after September 1, 1993); §
// 92.153(d) LANDLORD'S EXPENSE; § 92.153(e) OPERABLE throughout tenancy;
// §§ 92.164/92.165 tenant remedies one month rent + $500 + actual damages
// + attorney fees + court costs. (2) California Cal. Civ. Code §§ 1941.1
// + 1941.3 — deadbolt on main entry doors + window security devices on
// accessible windows + garage door locking mechanism if applicable;
// working order throughout tenancy at landlord expense; implied warranty
// of habitability breach with rent withholding + repair-and-deduct + lease
// termination remedies. (3) Default — Hilder v. St. Peter 144 Vt. 150
// (1984) + Javins v. First National Realty Corp. 428 F.2d 1071 (D.C. Cir.
// 1970) common-law implied warranty of habitability + negligence-per-se
// for break-in / assault claims. Distinct from `lock_change_between_
// tenancies`, `dv_survivor_lock_change`, and `tenant_smart_lock_biometric_
// consent`.
// ---------------------------------------------------------------------------

async fn landlord_security_device_obligations_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<LandlordSecurityDeviceInput>,
) -> Result<Json<LandlordSecurityDeviceResult>, ApiError> {
    Ok(Json(check_landlord_security_device_obligations(&b)))
}

// ---------------------------------------------------------------------------
// landlord_repair_response_timeframe: Mandatory landlord-paid written-
// response timeframe for tenant repair requests. Mounted at POST /api/
// rental/landlord-repair-response-timeframe. Four regimes: (1) Texas
// Tex. Prop. Code §§ 92.052(d) + 92.056 + 92.0563 — reasonable time
// presumed at 7 days (168 hours) for normal conditions or as soon as
// practicable for emergencies + § 92.0563 repair-and-deduct cap greater
// of one month's rent or $500. (2) Illinois Chicago RLTO § 5-12-110(d)
// — 14 days (336 hours) for ordinary repairs or 72 hours for
// emergencies + pro-rata rent withholding + lease termination + repair-
// and-deduct. (3) Washington RCW 59.18.070 — tiered response: 24 hours
// for no heat/water/electricity/imminent threat; 72 hours for major
// appliances/plumbing; 10 days (240 hours) for other conditions. (4)
// Default — common-law implied warranty of habitability per Hilder v.
// St. Peter + Javins v. First National Realty Corp. Distinct from
// `repair_and_deduct` + `habitability_remedies` + `landlord_security_
// device_obligations`.
// ---------------------------------------------------------------------------

async fn landlord_repair_response_timeframe_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<LandlordRepairResponseInput>,
) -> Result<Json<LandlordRepairResponseResult>, ApiError> {
    Ok(Json(check_landlord_repair_response_timeframe(&b)))
}

// ---------------------------------------------------------------------------
// landlord_tenant_recording_consent: Audio recording consent for landlord-
// tenant interactions. Two regimes: AllPartyConsent (CA, DE, FL, IL, MD,
// MA, MT, NV, NH, PA, WA — 11 states; both parties must consent to
// private-conversation recording) vs OnePartyConsent (federal 18 U.S.C.
// § 2511(2)(d) floor + 39 states + DC; one-party consent sufficient).
// Federal third-party-device rule: landlord installing device to record
// tenant conversations to which landlord is NOT a party violates § 2511
// regardless of state regime — one-party consent only protects a party.
// In-unit recording without all-party consent universally unlawful due
// to tenant's reasonable expectation of privacy. § 2511(4) criminal
// penalty up to 5 yrs + $250K; § 2520(c) civil $10K statutory minimum.
// Distinct from tenant_data_privacy, landlord_harassment, security_
// camera_disclosure.
// ---------------------------------------------------------------------------

async fn landlord_tenant_recording_consent_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RecordingConsentInput>,
) -> Result<Json<RecordingConsentResult>, ApiError> {
    Ok(Json(check_landlord_tenant_recording_consent(&b)))
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
// swimming_pool_safety: California Swimming Pool Safety Act compliance
// — when a building permit is issued for the construction of a new
// swimming pool or spa OR the remodeling of an existing pool or spa at
// a private single-family home, the pool or spa SHALL be equipped with
// AT LEAST TWO of seven enumerated drowning prevention safety features
// under Cal. Health & Safety Code § 115922(a). Mounted at POST /api/
// rental/swimming-pool-safety. Two regimes: California (Cal. Health &
// Safety Code §§ 115920-115929 + SB 442 Stats. 2017 ch. 670, eff. Jan
// 1, 2018 — increased minimum from one to two features; seven features
// include § 115923 enclosure, ASTM F2286 removable mesh, ASTM F1346-23
// safety pool cover, exit alarms, self-closing self-latching device
// with release ≥ 54in, ASTM F2208 pool alarm, other equivalent means
// approved by local building official; statute applies to PRIVATE
// SINGLE-FAMILY HOMES — multifamily pools regulated by California Code
// of Regulations Title 22 + § 116025 et seq. CalCode); Default (no
// statutory requirement at permit issuance; common-law premises
// liability + IPC § 305 where adopted + local pool ordinances).
// Trader-landlord critical for CA single-family rental properties with
// pools — non-compliance breaches habitability + exposes landlord to
// drowning-incident premises liability. Distinct from siblings
// detector_requirements, fire_sprinkler_disclosure, water_heater_
// earthquake_strap, lead_in_drinking_water_disclosure.
// ---------------------------------------------------------------------------

async fn swimming_pool_safety_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<SwimmingPoolSafetyInput>,
) -> Result<Json<SwimmingPoolSafetyResult>, ApiError> {
    Ok(Json(check_swimming_pool_safety(&b)))
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
// balcony_inspection: California SB 721 (multifamily rental Cal. Health
// & Safety Code § 17973) + SB 326 (Davis-Stirling condos Cal. Civ.
// Code § 5551) Exterior Elevated Elements (EEE) inspection compliance.
// Mounted at POST /api/rental/balcony-inspection. Three regimes:
// CaliforniaSb721 (3+ unit multifamily rental; first inspection
// Jan 1 2026 per AB 2579 extension from Jan 1 2025; recurring every
// 6 years; qualified inspector = licensed architect / civil or
// structural engineer / contractor A/B/C-5 + 5 years exp / certified
// building inspector; NOT local gov employee NOT entity performing
// repairs; min 15% of each EEE type direct visual + exploratory
// openings; 120-day repair deadline); CaliforniaSb326 (Davis-Stirling
// condos; first inspection Jan 1 2025; recurring every 9 years;
// qualified inspector LIMITED to licensed architect or licensed
// civil/structural engineer — NO contractors); Default (no statutory
// regime; common-law premises-liability duty + local ordinances).
// Trader-landlord critical for multi-unit owners with EEE elements
// elevated 6+ feet above ground.
// ---------------------------------------------------------------------------

async fn balcony_inspection_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<BalconyInspectionInput>,
) -> Result<Json<BalconyInspectionResult>, ApiError> {
    Ok(Json(check_balcony_inspection(&b)))
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
// lead_in_drinking_water_disclosure: Mandatory landlord-paid disclosure of
// LEAD IN DRINKING WATER to tenant when public water system notifies of
// elevated lead. Mounted at POST /api/rental/lead-in-drinking-water-
// disclosure. Three regimes: (1) New Jersey N.J.S.A. 58:12A-12.4 et seq.
// (Lead in Drinking Water Notification Act) — landlord MUST distribute
// notice to EVERY tenant within THREE BUSINESS DAYS of receipt + post in
// PROMINENT LOCATION accessible to tenants + P.L. 2021, c. 82 + P.L. 2021,
// c. 183 amendments + private right of action + civil penalties + NJ DEP
// enforcement. (2) Michigan Mich. Comp. Laws § 325.1001 et seq. (post-
// Flint Safe Drinking Water Act) + MAC R 325.10101 et seq. — state
// action level 12 µg/L (BELOW federal 15 µg/L) + Lead Action Level
// Exceedance Notice distribution + private right of action. (3) Default
// — federal SDWA 42 U.S.C. § 300f et seq. + EPA Lead and Copper Rule 40
// CFR Part 141.85 + Consumer Confidence Report 40 CFR Part 141 Subpart O
// + NO statutory landlord-tenant distribution mandate; state UDAP +
// negligence-per-se for pediatric lead poisoning claims. Distinct from
// `lead_disclosure` (federal Title X lead-based PAINT), `flood_
// disclosure`, `radon_disclosure`, `asbestos_disclosure`, `mold_
// disclosure`.
// ---------------------------------------------------------------------------

async fn lead_in_drinking_water_disclosure_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<LeadInDrinkingWaterInput>,
) -> Result<Json<LeadInDrinkingWaterResult>, ApiError> {
    Ok(Json(check_lead_in_drinking_water_disclosure(&b)))
}

// ---------------------------------------------------------------------------
// lead_renovation_repair_painting: EPA Lead Renovation, Repair, and Painting
// (RRP) Rule compliance — 40 CFR Part 745, Subpart E. Eight compliance
// elements when work is performed in target housing (pre-1978) or child-
// occupied facility above the § 745.83 minor-repair-and-maintenance
// threshold: (1) target housing or child-occupied facility; (2) work
// disturbs paint above de minimis (>6 sq ft interior / >20 sq ft exterior /
// window replacement / demolition); (3) firm EPA or state-authorized
// certified; (4) renovator individually trained (EPA-accredited 8-hour);
// (5) Renovate Right pamphlet provided + acknowledgment retained;
// (6) containment used + prohibited work practices avoided; (7) cleanup
// verification; (8) 3-year records retention. Two jurisdictions: EPA
// Federal (most states) vs StateAuthorized (15 EPA-delegated states under
// TSCA § 404: WI, IA, NC, MS, KS, RI, UT, OR, MA, AL, WA, GA, OK, DE, VT).
// TSCA § 16(a) civil penalty up to $37,500/day/violation (15 USC §
// 2615(a)(1)) at statutory base; inflation-adjusted maximum higher per
// 40 CFR § 19.4. Criminal penalties under TSCA § 16(b). Distinct from
// lead_disclosure (TSCA § 1018 / 40 CFR Part 745 Subpart F initial
// disclosure upon SALE or LEASE), asbestos_disclosure.
// ---------------------------------------------------------------------------

async fn lead_renovation_repair_painting_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RrpInput>,
) -> Result<Json<RrpResult>, ApiError> {
    Ok(Json(check_lead_rrp(&b)))
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
// just_cause_termination_notice_content: Just-cause termination notice CONTENT
// requirements — what content + format must the WRITTEN termination notice
// contain to satisfy just-cause statutory requirements? CA Civ. Code
// § 1946.2(c) (written cause + cure for curable at-fault + relocation
// assistance for no-fault + VOID for noncompliance); WA RCW 59.18.650(2)
// (specific cause + facts and circumstances + 16 enumerated categories
// (a)-(p)); OR ORS 90.427 SB 608 (written reason + < 1 year no-cause path);
// NJ N.J.S.A. 2A:18-61.2 + § 2A:18-61.1(a)-(r) (18 enumerated grounds + exact
// statutory match required).
// ---------------------------------------------------------------------------

async fn just_cause_termination_notice_content_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<JustCauseNoticeContentInput>,
) -> Result<Json<JustCauseNoticeContentResult>, ApiError> {
    Ok(Json(check_just_cause_notice_content(&b)))
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
// broker_fee_allocation: Broker fee allocation between landlord and tenant.
// Two regimes: NewYorkCityFareAct (NYC Local Law 119, eff. June 11, 2025 —
// party-who-hired-pays rule under § 20-699.20; landlord may not impose fee
// on tenant for broker landlord engaged; § 20-699.21 disclosure of other
// fees in listing AND lease; § 20-699.22 DCWP civil penalty / action
// enforcement); Default (lease + market practice + Boston Ord. 17-2024
// disclosure overlay). Distinct from application_fees, non_refundable_
// cleaning_fees, pet_fees, lease_disclosures.
// ---------------------------------------------------------------------------

async fn broker_fee_allocation_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<BrokerFeeAllocationInput>,
) -> Result<Json<BrokerFeeAllocationResult>, ApiError> {
    if b.broker_fee_amount_cents < 0 {
        return Err(ApiError::BadRequest(
            "broker_fee_amount_cents must be >= 0".into(),
        ));
    }
    Ok(Json(check_broker_fee_allocation(&b)))
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
// rent_control_lease_disclosure: Mandatory landlord disclosure of rent
// control / rent stabilization status at LEASE EXECUTION (distinct from
// rent_control substantive cap mechanics, rent_increase_notice_period
// advance-notice-before-raising-rent, and lease_disclosures general
// lease-required disclosures). Four regimes: California (Cal. Civ. Code
// § 1947.12 + § 1946.2 AB 1482 — addendum REQUIRED for post-July-1-2020
// tenancies + 12-point font + § 1947.12(d)(5) exempt-property ownership
// language requirement); Oregon (Or. Rev. Stat. § 90.323 SB 608 — NO
// addendum required; cap operates through 90/180-day rent-increase
// notice + < 15-year / vacancy / subsidized / shared-unit exemptions);
// NewYork (RPL § 234 + RSC § 2522.5(c) rent stabilization lease rider
// MANDATORY for stabilized units; failure makes lease unenforceable as
// to provisions conflicting with stabilization law); Default (no
// statewide disclosure obligation; municipal control may apply).
// ---------------------------------------------------------------------------

async fn rent_control_lease_disclosure_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentControlDisclosureInput>,
) -> Result<Json<RentControlDisclosureResult>, ApiError> {
    Ok(Json(check_rent_control_disclosure(&b)))
}

// ---------------------------------------------------------------------------
// rent_overcharge_recovery: Rent overcharge recovery in rent-stabilized /
// rent-controlled buildings — when a landlord charges rent in excess of
// the legal regulated rent, what statutory procedure and damages attach?
// Mounted at POST /api/rental/rent-overcharge-recovery. Three regimes:
// (1) New York HSTPA of 2019 (N.Y. Laws 2019, ch. 36) most aggressive
// framework: 6-YEAR lookback (extended from 4) + TREBLE DAMAGES MANDATORY
// on willful overcharge (made non-discretionary) + attorney fees + costs
// + interest NON-DISCRETIONARY + fraud exception extends lookback further
// when landlord falsifies records or fails to register with DHCR + tenant
// may file with DHCR or court under § 26-516. (2) DC RACA D.C. Code §§
// 42-3502.01 to 42-3502.20 — Rental Housing Commission administers
// overcharge procedure + treble damages available for willful + cross-
// references tenant_topa for TOPA. (3) Default — no statewide
// rent-stabilization framework; common-law restitution per Restatement
// (Third) of Restitution and Unjust Enrichment § 1 + municipal
// ordinances (Berkeley, San Francisco, Los Angeles, Oakland, Santa
// Monica, Newark, Hoboken, Jersey City). Distinct from siblings
// `rent_control` (broad regulatory framework), `rent_control_lease_
// disclosure` (initial disclosure mandate), `rent_increase_notice_
// period`, `rent_acceleration_enforceability`.
// ---------------------------------------------------------------------------

async fn rent_overcharge_recovery_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentOverchargeRecoveryInput>,
) -> Result<Json<RentOverchargeRecoveryResult>, ApiError> {
    Ok(Json(check_rent_overcharge_recovery(&b)))
}

// ---------------------------------------------------------------------------
// rubs_utility_billing_disclosure: Mandatory landlord-paid disclosure of
// RUBS (Ratio Utility Billing System) allocation methodology when
// allocating utility costs in master-metered buildings WITHOUT individual
// sub-meters. Mounted at POST /api/rental/rubs-utility-billing-disclosure.
// Three regimes: (1) Texas Tex. Water Code §§ 13.502 + 13.2502 + 13.503
// + 16 TAC § 24.281 — lease MUST state RUBS allocation + specify exact
// calculation method (occupant count OR square footage) + landlord may
// NOT add service / administrative fees + aggregate tenant charges cannot
// exceed utility provider's bill + § 13.503 private right of action with
// civil damages + Public Utility Commission of Texas enforcement. (2)
// District of Columbia D.C. Code § 42-3502.06A + DC AG Schwalb guidance
// — clearly identify allocation method in lease + provide ANNUAL
// RECONCILIATION STATEMENT showing actual utility costs vs amounts
// collected + no surcharges + DC Consumer Protection Procedures Act. (3)
// Default — no specific RUBS statute; lease + state PUC tariff + common-
// law unconscionability + state UDAP. Distinct from `submetering_rules`
// (sub-meter setup), `tenant_utility_account_designation` (direct utility
// account), and `utility_shutoff`.
// ---------------------------------------------------------------------------

async fn rubs_utility_billing_disclosure_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RubsUtilityBillingInput>,
) -> Result<Json<RubsUtilityBillingResult>, ApiError> {
    Ok(Json(check_rubs_utility_billing_disclosure(&b)))
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
// security_deposit_interest_statement: Mandatory annual security deposit
// interest statement disclosure. Four regimes: Massachusetts (Mass. G.L.
// c. 186 § 15B(2)(c)(ii) annual statement with bank name/address +
// account number + deposit amount + interest amount + § 15B(2)(c)
// payment-or-deduction option + § 15B(7) TRIPLE damages for willful
// violation); NewJersey (N.J.S.A. 46:8-19(c) annual interest payment +
// statement + 1% administrative cost allowance + § 46:8-21.1 DOUBLE
// damages willful); Chicago (Chicago RLTO § 5-12-080(c) within 30 days
// after end of 12-month period with deposit + interest + calculation
// explanation + § 5-12-080(f) DOUBLE damages willful); NewYork (N.Y.
// GOL § 7-103 trust fund + initial bank disclosure for 6+ unit
// buildings — no annual statement requirement); Default (no statewide
// annual statement requirement). Distinct from deposit_interest
// (whether interest required and rate) and security_deposit_bank_
// disclosure (initial bank location disclosure at tenancy start).
// ---------------------------------------------------------------------------

async fn security_deposit_interest_statement_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<DepositInterestStatementInput>,
) -> Result<Json<DepositInterestStatementResult>, ApiError> {
    Ok(Json(check_security_deposit_interest_statement(&b)))
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

// ---------------------------------------------------------------------------
// rental_oil_tank_replacement_disclosure (iter 493): Northeast trader-landlords
// inherit legacy 1940-1980s home-heating-oil tanks (UST/AST). NJDEP UHOT
// Program governs residential tanks 2000 gal or less per N.J.A.C. 7:14B-
// 1.6(a)(3); Massachusetts 527 CMR 9.00 + 310 CMR 80.00 (MassDEP eff.
// 2015-01-02); Maine 38 M.R.S. 568-A requires age/location disclosure on
// rental. CERCLA 42 U.S.C. 9607(a) strict joint-and-several liability for
// petroleum-product UST leaks regardless of fault. Endpoint computes
// disclosure-required vs replacement-required-age-failure vs replacement-
// required-leak-detected vs rescission-risk-fraud-in-inducement severity.
// Coordinates with rental_propane_tank_lease_disclosure (iter 475 LP-gas
// sibling), rental_lead_pipe_disclosure (legacy infrastructure pattern),
// rental_pesticide_application_notification (chemical-exposure pattern).
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// rental_oakland_measure_ee_just_cause_omc_8_22: Oakland Just Cause
// for Eviction Ordinance (Measure EE), adopted by voters of Oakland
// in November 2002; codified at Oakland Municipal Code Title 8
// Chapter 8.22 Article II (§§ 8.22.300 through 8.22.470).
// Substantially amended in November 2022 by voter approval of
// Measure V (broadened protected-tenant categories to include
// catastrophic illness; increased civil penalties; added anti-
// harassment protections). Coverage scope under OMC § 8.22.350:
// buildings with 2 OR MORE UNITS with certificate of occupancy
// issued BEFORE JANUARY 1, 1983; exemptions include hospitals,
// nonprofit-operated housing, owner-occupied buildings, units with
// certificate of occupancy after 1983 (or after 2003 for new-
// construction carve-outs), state/federal preemption, and transient
// hotel occupancy. 11 just-cause grounds under OMC § 8.22.360:
// (1) non-payment after 3-day notice; (2) breach of lease or refused
// renewal with materially identical terms; (3) willful damage;
// (4) disorderly conduct; (5) illegal use; (6) denial of landlord
// access after written notice; (7) substantial repairs requiring
// temporary relocation; (8) owner / relative move-in (owner, spouse,
// domestic partner, child, parent, grandparent); (9) Ellis Act
// withdrawal under California Government Code § 7060; (10) demolition
// with valid permits; (11) end of temporary tenancy or other narrow
// specified ground. Protected tenants under § 8.22.360(8): age 60+
// with at least 5 years' tenure, disabled tenants, OR (Measure V
// 2022) tenants with catastrophic illness — all protected from
// owner / relative move-in evictions. Ellis Act withdrawal notice
// periods under California Government Code § 7060.4: 120 DAYS
// standard; 365 DAYS (1 year) extended notice for senior (62+) or
// disabled tenants with at least 1 year tenure. Remedies under
// OMC § 8.22.370: statutory damages + actual damages + treble
// damages for willful violations + reasonable attorney's fees +
// injunctive relief (Measure V increased penalty structure).
// Seventeen-mode severity ladder × 2 property jurisdictions ×
// variable unit count × 4 unit types × 6 exemption statuses ×
// 2 certificate-of-occupancy date statuses × 3 compliance aspects ×
// 12 just-cause grounds × 5 protected-tenant statuses × variable
// Ellis Act notice days. Sibling cluster: rental_san_francisco_
// rent_ordinance_chapter_37 (iter 677 — SF 1979 OLDEST CA municipal
// regime), rental_berkeley_rent_stabilization_ordinance_bmc_chapter_
// 13_76 (iter 679 — Berkeley 1980 SECOND OLDEST), rental_seattle_
// smc_22_206_160_just_cause_eviction (iter 669 — Seattle JCEO 1980),
// rental_california_sb_567_no_fault_eviction_amendments (iter 673
// — CA AB 1482 + SB 567 statewide overlay), rental_california_ab_12_
// security_deposit_cap (iter 645), rental_california_ab_2347_unlawful_
// detainer_response (iter 667), rental_just_cause_eviction (multi-
// state base), rental_owner_move_in_eviction (OMI cross-reference),
// rental_demolition_tenant_notice (cross-reference), rental_rent_
// control_stabilization (multi-state regime).
// ---------------------------------------------------------------------------

async fn rental_oakland_measure_ee_just_cause_omc_8_22_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalOaklandMeasureEeJustCauseOmc822Input>,
) -> Result<Json<RentalOaklandMeasureEeJustCauseOmc822Result>, ApiError> {
    Ok(Json(check_rental_oakland_measure_ee_just_cause_omc_8_22(&b)))
}

async fn rental_oil_tank_replacement_disclosure_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalOilTankReplacementDisclosureInput>,
) -> Result<Json<RentalOilTankReplacementDisclosureResult>, ApiError> {
    Ok(Json(check_rental_oil_tank_replacement_disclosure(&b)))
}

// ---------------------------------------------------------------------------
// rental_oregon_sb_608_sb_611_rent_stabilization: Oregon SB 608 of
// 2019 + SB 611 of 2023 statewide rent stabilization. FIRST statewide
// rent control law in U.S. history, predating Washington HB 1217 by
// six years. SB 608 signed by Governor Kate Brown on February 28,
// 2019, effective immediately; SB 611 signed by Governor Tina Kotek
// on July 6, 2023, effective immediately. Amends ORS 90.323 (general
// residential rent cap), ORS 90.600 (manufactured/floating home park
// rent cap), ORS 90.427 (just cause termination). Post-SB 611
// general residential cap: 7 % + CPI OR 10 %, whichever LESS.
// Manufactured/floating home park cap: 6 % maximum annual percentage
// rent increase; parks/marinas with 30 or fewer spaces EXEMPT.
// First-year tenancy: NO rent increase permitted; 'first year of
// occupancy' includes all periods any tenant has resided in unit.
// Notice: 90-day written notice (general residential), 7-day notice
// (week-to-week). 15-year new construction exemption from first
// certificate of occupancy. Government subsidy exemption. Oregon
// DAS publishes annual maximum percentage by September 30 based on
// West Region CPI. Portland City Code 30.01.085 + Milwaukie ordinance
// additive 90-day no-cause termination notice. Sixteen-mode severity
// ladder × six property classifications × four tenancy statuses ×
// five notice categories. Trader-landlord critical because Oregon
// is the FIRST state with statewide rent control; multi-state
// portfolio operators must distinguish OR (7 % + CPI / 10 % / 15-yr
// exempt / 6 % MHP) vs WA HB 1217 (7 % + CPI / 10 % / 12-yr exempt
// / 5 % MHP) vs CA AB 1482 (5 % + CPI / 10 % / 15-yr exempt) cap
// structures. Sibling cluster: rental_washington_hb_1217_rent_
// stabilization (iter 643 — WA companion), rental_california_ab_12_
// security_deposit_cap (iter 645 — CA companion), rental_rent_
// control_stabilization (multi-state regime — Oregon = FIRST entry),
// rental_just_cause_eviction (ORS 90.427 cross-reference), rental_
// mobile_home_park (ORS 90.600 cross-reference), rental_rent_
// increase_notice_requirement (90-day notice cross-reference).
// ---------------------------------------------------------------------------

async fn rental_oregon_sb_608_sb_611_rent_stabilization_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalOregonSb608Sb611RentStabilizationInput>,
) -> Result<Json<RentalOregonSb608Sb611RentStabilizationResult>, ApiError> {
    Ok(Json(check_rental_oregon_sb_608_sb_611_rent_stabilization(&b)))
}

// ---------------------------------------------------------------------------
// rental_solar_panel_disclosure (iter 495): Trader-landlord rooftop solar PV
// disclosure framework. Five financing structures (outright-owned, financed-
// loan, solar lease, PPA per-kWh, community subscription). Five failure
// modes: utility-bill confusion, net-metering credit misallocation, PPA
// pass-through without lease authorization, roof warranty void on PV
// penetration, PPA acceleration on default. Seven jurisdictions: CA (Cal.
// Civ. Code § 1102.6c + § 1947.13 + NEM 3.0 / NBT eff. April 15, 2023 + SB
// 1340), MA (220 CMR 18.00 + SMART eff. November 26, 2018 + M.G.L. ch.
// 164 § 138-140), NJ (N.J.S.A. 48:3-87.13 + SuSI eff. August 28, 2021),
// AZ (A.A.C. R14-2-1801 + A.R.S. § 33-1310 + HB 2675), HI (H.R.S. § 269-
// 101.5 + CSS + CGS+ tariffs since October 21, 2015 closing NEM), NY (16
// NYCRR Part 96 + VDER Value Stack eff. March 9, 2017), Default (federal
// 26 U.S.C. § 48 ITC + Notice 2018-59 + 40 C.F.R. Part 273 Universal
// Waste for cracked-panel CdTe disposal). Coordinates with sibling tenant_
// solar_installation (right-to-install on portable systems), rental_propane_
// tank_lease_disclosure (lease-pass-through analog), rental_oil_tank_
// replacement_disclosure (energy-infrastructure pattern), ev_charger_
// installation (parallel tenant-right framework).
// ---------------------------------------------------------------------------

async fn rental_solar_panel_disclosure_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalSolarPanelDisclosureInput>,
) -> Result<Json<RentalSolarPanelDisclosureResult>, ApiError> {
    Ok(Json(check_rental_solar_panel_disclosure(&b)))
}

// ---------------------------------------------------------------------------
// tenant_smart_thermostat_install_right (iter 497): Tenant right to install
// Nest / Ecobee / Honeywell smart thermostat in rental unit; parallel to
// tenant_solar_installation, tenant_ev_charging_installation_right,
// tenant_clothesline_drying_right, tenant_window_air_conditioner_install_
// right. Five-jurisdiction framework: CA (Cal. Civ. Code § 1947.6 + § 1942.1
// + Title 24 eff. January 1, 2023 + SB 1136 pending), NY (NYC Local Law 97
// Climate Mobilization Act signed April 18, 2019 / eff. 2024 carbon caps +
// Real Property Law § 235-b + NYC Admin Code § 27-2029), MA (Mass Save
// $100 rebate + 520 CMR 13.00 Stretch Code adopted by 299 municipalities
// + M.G.L. ch. 186 § 14), Default (42 U.S.C. § 3604(f)(3)(B) FHA reasonable
// modification + 24 C.F.R. § 100.203 + common-law habitability). Four
// HVAC wiring types: 24V low-voltage with C-wire (compatible), 24V without
// C-wire (needs adapter), 120V baseboard (line-voltage thermostat only —
// Mysa / Sinopé), steam / hot-water radiator (smart TRV alternative).
// Eight-mode severity ladder: NotApplicable, InstallationPermittedRoutine,
// WiringIncompatibilityRefusalPermitted, HvacWarrantyVoidRefusalPermitted,
// HistoricDistrictRestrictionApplies, AdaFhaReasonableAccommodationRequired
// (100% rent at risk), EnergyEfficiencyRebateEligibleApprovalLikely,
// LandlordRefusalUnreasonableHabitabilityBreach (50% rent at risk).
// ---------------------------------------------------------------------------

async fn tenant_smart_thermostat_install_right_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<TenantSmartThermostatInstallRightInput>,
) -> Result<Json<TenantSmartThermostatInstallRightResult>, ApiError> {
    Ok(Json(check_tenant_smart_thermostat_install_right(&b)))
}

// ---------------------------------------------------------------------------
// rental_pellet_stove_disclosure (iter 499): Trader-landlord pellet-stove
// (solid-fuel-burning residential heater) disclosure plus EPA NSPS Step-2
// certification framework. Federal: 40 C.F.R. Part 60 Subpart AAA (wood
// stoves) + Subpart QQQQ (hydronic heaters + forced-air furnaces) +
// Step-2 emissions limit 2.0 g/hr PM effective May 15, 2020. Six
// jurisdictions: Vermont (10 V.S.A. § 583 + § 5-204.4 + 9 V.S.A. § 2882
// CO detector), Maine (38 M.R.S. § 581 + 25 M.R.S. § 2468-A CO + NFPA
// 211), New Hampshire (RSA 153:4-a + Saf-C 6000 fire chief permit + RSA
// 153:10-a CO), Washington (WAC 173-433 + 173-433-130 strictest non-
// attainment rules — Pierce / Spokane / Yakima / Klickitat), Colorado
// (5 CCR 1001-10 Reg No. 4 + designated-area burn ban Aspen / Telluride /
// Vail), Default (40 C.F.R. Part 60 federal NSPS + NFPA 211). Six failure
// modes: non-certified stove in non-attainment area, missing CO detector,
// chimney inspection overdue, owner's manual not provided, auger jam CO
// release undisclosed, lease-signing disclosure incomplete. Coordinates
// with sibling rental_chimney_fireplace_inspection_disclosure, rental_
// natural_gas_leak_response (CO detector cross-ref), rental_oil_tank_
// replacement_disclosure (legacy heating disclosure pattern), rental_
// propane_tank_lease_disclosure (alternative heating fuel).
// ---------------------------------------------------------------------------

async fn rental_pellet_stove_disclosure_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalPelletStoveDisclosureInput>,
) -> Result<Json<RentalPelletStoveDisclosureResult>, ApiError> {
    Ok(Json(check_rental_pellet_stove_disclosure(&b)))
}

// ---------------------------------------------------------------------------
// rental_in_unit_laundry_appliance_provision (iter 501): Trader-landlord
// in-unit washer/dryer provision plus dryer-vent fire safety and mold
// liability framework. Five jurisdictions: California (Cal. Civ. Code §
// 1941.1 + § 1942 + Cal. Health & Safety Code § 17920.3 mold + AB 628
// eff. January 1, 2026 [refrigerator + stove only — no W/D mandate]),
// Massachusetts (M.G.L. ch. 186 § 14 quiet enjoyment + 105 CMR 410 State
// Sanitary Code + 527 CMR 50.00 fire prevention + Mass.gov DFS Dryer Fire
// Safety), New York (Real Property Law § 235-b + NYC Admin Code § 27-
// 2017.3 mold remediation Local Law 55 of 2018 + MDL § 78), Washington
// (RCW 59.18.060 + RCW 59.18.060(11) Mold Disclosure Act + DOH 'Got Mold?'
// publication), Default (IRC Section M1502 + NFPA 211 + IFC 504 + UL 2158A
// Clothes Dryer Transition Duct). Four appliance-provision statuses: No-
// LaundryAppliances, TenantOwnedTenantMaintained, LandlordProvidedWith-
// ContinuingDuty, SharedCoinOpLaundryRoom. Four dryer-vent termination
// types: ExteriorWallWithBackdraftDamper, AtticTerminationProhibited,
// CrawlspaceTerminationProhibited, GarageOrUnconditionedSpace. Eight-mode
// severity ladder: NotApplicable, CompliantWithMaintenanceCadence, Land-
// lordProvidedRepairOverdueHabitabilityBreach (100pct rent at risk), Im-
// properVentingAtticOrCrawlspaceMoldRisk (100pct rent at risk), NonCompliant-
// TransitionDuctFireRisk (100pct rent at risk), LintBuildupAnnualMaintenance-
// OverdueFireRisk (50pct rent at risk — 2,900 US dryer fires annually per
// USFA), GasDryerCoExposureWithoutAdequateVentilation (100pct rent at risk),
// DisclosureRequiredAtLeaseSigning (50pct rent at risk).
// ---------------------------------------------------------------------------

async fn rental_in_unit_laundry_appliance_provision_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalInUnitLaundryApplianceProvisionInput>,
) -> Result<Json<RentalInUnitLaundryApplianceProvisionResult>, ApiError> {
    Ok(Json(check_rental_in_unit_laundry_appliance_provision(&b)))
}

// ---------------------------------------------------------------------------
// rental_post_construction_lead_dust_clearance (iter 503): Trader-landlord
// post-renovation lead-dust clearance testing framework for pre-1978 target
// housing. Federal: EPA RRP rule 40 C.F.R. Part 745 Subpart E (Lead-Safe
// Certified Firm + Certified Renovator + Renovate Right pamphlet + cleaning
// verification card) + HUD Lead Safe Housing Rule 24 C.F.R. Part 35 (mandates
// dust-wipe clearance for federally-assisted housing) + TSCA § 402(c) dust-
// lead hazard standards (pre-2020-01-06: floor 40 / sill 250 / trough 400
// µg/sq ft; post-2020-01-06: floor 10 / sill 100 / trough 400 per Federal
// Register June 21, 2019 final rule; effective 2026-01-12: ANY reportable
// level = hazard per October 24, 2024 EPA final rule). Four jurisdictions:
// MA (105 CMR 460 most stringent in US + M.G.L. ch. 111 §§ 190-199A + CLPPP
// Letter of Compliance + triple-damages private right of action), CA (Cal.
// Health & Safety Code § 17920.10 + Title 17 Div. 1 Ch. 8 CLPPB), NY (NYC
// Local Law 1 of 2004 + NYC Admin Code § 27-2056 + DOH § 11-101 + LL 31
// of 2020 XRF testing + NY PHL § 1373), Default (RRP + HUD LSHR + TSCA
// §§ 402+403). Three housing scopes: Pre1978TargetHousing, Pre1960Housing
// (NYC LL 1 trigger), Post1978NotApplicable. Eleven-mode severity ladder
// including ChildPregnancyExposureMaximumLiability (100pct rent) +
// UncertifiedFirmRrpViolation ($51,796/day EPA penalty per 40 C.F.R. § 19.4)
// + clearance-fail-by-location (floor/sill/trough). RRP trigger thresholds:
// 6 sq ft interior or 20 sq ft exterior disturbance per 40 C.F.R. § 745.83.
// ---------------------------------------------------------------------------

async fn rental_post_construction_lead_dust_clearance_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalPostConstructionLeadDustClearanceInput>,
) -> Result<Json<RentalPostConstructionLeadDustClearanceResult>, ApiError> {
    Ok(Json(check_rental_post_construction_lead_dust_clearance(&b)))
}

// ---------------------------------------------------------------------------
// tenant_voting_address_protection (iter 505): Trader-landlord Address
// Confidentiality Program (ACP) plus victim-status disclosure restriction
// framework. Five jurisdictions: California (Cal. Gov. Code § 6206-6210
// Safe at Home Program administered by Secretary of State + Cal. Civ. Code
// § 1946.7 lease termination + § 1161.3 unlawful detainer protection +
// Penal § 273.6), Massachusetts (M.G.L. ch. 9A ACP administered by
// Secretary of the Commonwealth + ch. 209A Abuse Prevention + ch. 186 §§
// 24-29 lease termination + ch. 151B § 4 fair housing — FELONY under
// M.G.L. ch. 9A § 8 for breach causing harm), New York (N.Y. Executive
// Law § 108 ACP administered by Department of State + RPL § 227-c lease
// termination + Social Services Law § 459-a + post-Dobbs reproductive
// health care services providers eligibility), Washington (RCW 40.24
// — FIRST state to enact ACP in 1991 + RCW 59.18.575 lease termination
// + 59.18.585 adverse action prohibition + expanded to CJ Affiliates +
// Election Officials + Protected Health Care Workers), Default (federal
// VAWA 42 U.S.C. § 14043e + 24 C.F.R. § 5.2003 + common-law privacy
// torts). Three ACP statuses: EnrolledWithSubstituteAddress, Victim-
// StatusDisclosedNotEnrolled, NoAcpAndNoVictimDisclosure. Eight
// disclosure-request types: VoterRegistrationChallenger, LawEnforcement-
// WithWarrantOrSubpoena, LawEnforcementWithoutWarrant, ProcessServerCivil-
// Unrelated, DebtCollectorOrSkipTracer, FamilyMemberOrEstrangedSpouse,
// RoutineLandlordBusiness, NoRequest. Ten-mode severity ladder including
// ConfidentialityBreachAcpParticipantCriminalLiability (100pct rent at
// risk + misdemeanor under all state ACP statutes + felony under MA/NY/
// WA when causes actual harm to participant) and ConfidentialityBreach-
// NonAcpVictimTortLiability (intrusion upon seclusion + public disclosure
// of private facts + state-statutory confidentiality duty).
// ---------------------------------------------------------------------------

async fn tenant_voting_address_protection_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<TenantVotingAddressProtectionInput>,
) -> Result<Json<TenantVotingAddressProtectionResult>, ApiError> {
    Ok(Json(check_tenant_voting_address_protection(&b)))
}

// ---------------------------------------------------------------------------
// tenant_kitchen_appliance_replacement (iter 507): Trader-landlord
// refrigerator + stove provision plus continuing maintenance duty
// framework. California AB 628 (Cal. Civ. Code § 1941.1 amended effective
// January 1, 2026) makes refrigerator + stove NEW MANDATORY habitability
// provisions for leases entered, amended, or renewed on or after that
// date. CPSC recall response window 30 days. Refrigerator opt-out
// permitted only by voluntary written tenant election at lease signing;
// stove opt-out PROHIBITED regardless of tenant preference. Exemptions:
// permanent supportive housing, single-room occupancy, residential
// hotels, communal-kitchen assisted living facilities. Five jurisdictions:
// California (AB 628 + Cal. Civ. Code § 1942 + Cal. Health and Safety
// Code § 17920.3 + IRA § 50121 HOMES rebate + § 50122 HEEHRA rebate for
// ENERGY STAR + induction range upgrades), Massachusetts (105 CMR
// 410.100(E) + M.G.L. ch. 186 § 14 + ch. 111 § 127A — stove required
// since 1976), New York (NYC Admin Code § 27-2017.2 + MDL § 76 + RPL
// § 235-b), Illinois (Chicago Municipal Code § 5-12-110 RLTO),
// Default (common-law habitability + 15 U.S.C. § 2064 CPSC). Two
// appliance types: Refrigerator (opt-out eligible), Stove (no opt-out).
// Four appliance conditions: WorkingProperly, FailingPastNoticeWindow,
// CpscRecalled, NotProvided. Eight-mode severity ladder including
// NoApplianceProvidedAb628Violation (100pct rent) + CpscRecalledNot-
// ReplacedWithin30DaysHabitabilityBreach (100pct rent) + StoveOptOut-
// AttemptedProhibited (50pct rent — void against public policy).
// ---------------------------------------------------------------------------

async fn tenant_kitchen_appliance_replacement_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<TenantKitchenApplianceReplacementInput>,
) -> Result<Json<TenantKitchenApplianceReplacementResult>, ApiError> {
    Ok(Json(check_tenant_kitchen_appliance_replacement(&b)))
}

// ---------------------------------------------------------------------------
// rental_storage_unit_lease_disclosure (iter 509): Trader-landlord rental
// storage unit lease disclosure framework — covers (1) commercial self-
// service storage facilities subject to state SSF Acts plus newly-enacted
// California SB 709 first-page disclosure requirements effective January 1
// 2026 + (2) residential rental storage (basement, garage, external locker
// leased to apartment tenant) governed by landlord-tenant law plus UCC
// Article 7 warehouse-keeper lien. Five jurisdictions: California (Cal.
// Bus. & Prof. Code § 21700-21716 SSF Act + SB 709 amending § 21712 with
// six first-page disclosures effective January 1 2026 + AB 1916 + AB 1108),
// New York (N.Y. General Business Law § 182 + Lien Law § 184 + S3690
// 2025 pending notice expansion), Florida (Fla. Stat. § 83.801-83.809 +
// 7-day default trigger + newspaper publication twice in successive weeks),
// Texas (Tex. Prop. Code § 59.001-59.046 + § 59.044 verified mail notice
// + § 59.041(c) identity document return), Default (Uniform Self-Service
// Storage Facility Act + UCC § 7-209/7-210 warehouse-keeper lien). Four
// storage types: CommercialSelfServiceStorageFacility, ResidentialStorage-
// IncludedWithApartmentLease (excluded from SSF Act per § 21702(c)),
// SeparateResidentialStorageRental, NoStorageUnit. Eight-mode severity
// ladder including SbSix709FirstPageDisclosureMissing + LienDefaultCure-
// WindowNotMet (CA 14-day) + LienSaleNoticeRequirementsNotMet (CA 60-day)
// + IdentityDocumentReturnRequirementBreached (Cal. Penal § 530.5 + 18
// U.S.C. § 1028) + ExcessProceedsReturnFailureToTenant (90-day return
// window + Cal. Civ. Code § 1500 escheat).
// ---------------------------------------------------------------------------

async fn rental_storage_unit_lease_disclosure_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalStorageUnitLeaseDisclosureInput>,
) -> Result<Json<RentalStorageUnitLeaseDisclosureResult>, ApiError> {
    Ok(Json(check_rental_storage_unit_lease_disclosure(&b)))
}

// ---------------------------------------------------------------------------
// rental_balcony_inspection_seismic_safety (iter 511): Trader-landlord
// Exterior Elevated Element (EEE) inspection compliance framework enacted in
// response to June 16 2015 Berkeley balcony collapse killing 6 students at
// Library Gardens apartments. Three jurisdictions: California (Cal. Health
// & Safety Code § 17973 SB 721 signed September 17 2018 effective January
// 1 2019 + Cal. Civ. Code § 5551 SB 326 signed August 30 2019 + AB 2579
// signed 2024 extending SB 721 first cycle deadline from January 1 2025 to
// January 1 2026 + Cal. Civ. Code § 1942.4 tenantability claim), New York
// City FISP (NYC Local Law 11 of 1998 + NYC AC § 28-302.1 + RCNY 1 §
// 103-04 — 5-year cycle façade inspection for 6+ story buildings), Default
// (common-law habitability + premises-liability tort + Florida + Hawaii
// pending legislation post-2021 Champlain Towers collapse). Five building
// types: Multifamily3PlusRentalSb721, HoaCondominiumSb326, SingleFamily-
// OrDuplex, NoWoodFramedEees, NycSixPlusStoryFisp. Five inspector
// qualifications: LicensedArchitect (SB 721), LicensedCivilOrStructural-
// Engineer (SB 721 + SB 326), ContractorWithA_B_C5LicenseAndFiveYears
// (SB 721 only), CertifiedBuildingInspector (SB 721 only), NotQualified.
// Eight-mode severity ladder including FirstCycleInspectionPastDeadline-
// Violation (100pct rent — $100-$500/day civil penalty), ImmediateThreat-
// RepairOverdue120Day (100pct rent), HoaSb326UsedNonStructuralEngineer-
// Invalid (100pct rent — § 5551 requires structural engineer specifically),
// TenantOccupiedUnsafeEeeHabitabilityBreach (100pct rent — § 1942.4),
// InspectionSampleBelow15Pct (50pct rent — § 17973(c) minimum 15pct
// sample per EEE type). Six-year SB 721 subsequent cycle; nine-year SB
// 326 cycle. Constants: 15pct minimum sample, 120-day immediate threat
// repair window, 5-year contractor experience requirement, $100-$500/day
// daily penalty range.
// ---------------------------------------------------------------------------

async fn rental_balcony_inspection_seismic_safety_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalBalconyInspectionSeismicSafetyInput>,
) -> Result<Json<RentalBalconyInspectionSeismicSafetyResult>, ApiError> {
    Ok(Json(check_rental_balcony_inspection_seismic_safety(&b)))
}

// ---------------------------------------------------------------------------
// rental_short_term_subletting_airbnb_restriction (iter 513): Trader-landlord
// tenant STR (Airbnb / VRBO / Booking.com) restriction framework. Five
// jurisdictions: NYC (LL 18 of 2021 effective September 5 2023 + NY RPL
// § 226-b + 6 RCNY § 1-04 + Mayor's Office of Special Enforcement + $145
// registration fee + primary-residence host present + < 30 days rental
// + listings plunged > 90pct post-effective), LA (LAMC § 12.22 A.32 Home-
// Sharing Ordinance + 120-night annual cap + LA County Code § 4.72 TOT
// + Department of City Planning registration), SF (SF Admin Code Ch. 41A
// + Office of Short-Term Rentals + $925 [2025] registration + Treasurer-
// Tax Collector dual registration + 90-night non-hosted cap + SF Rent
// Ordinance § 37 + Costa-Hawkins), Boston (Boston Ord. § 9-14 + Mass.
// Gen. Laws ch. 64G Room Occupancy Excise 5.7pct state + 6pct Boston
// locally-set = 11.7pct effective + Cape Cod 2.75pct Wastewater + 3pct
// Community Impact + three categories limited share/home share/owner-
// adjacent), Default (common-law lease assignment + state TOT framework).
// Three lease term types: ExplicitNoSublettingAirbnbClause, Subletting-
// RequiresLandlordConsent, LeaseSilentDefaultStateRule. Four STR statuses:
// OperatingWithoutLandlordConsent, OperatingWithWrittenLandlordConsent,
// NotOperatingStr, PrimaryResidenceHostPresentLimitedShare. Eight-mode
// severity ladder including LeaseBreachUnauthorizedSublet (100pct rent +
// STR revenue disgorgement), MunicipalStrRegistrationMissing (50pct rent),
// PrimaryResidenceRequirementViolated (50pct rent), AnnualNightCapExceeded
// (50pct rent), RoomOccupancyExciseTaxNotCollected (50pct rent + Form
// ST-7 monthly return).
// ---------------------------------------------------------------------------

async fn rental_short_term_subletting_airbnb_restriction_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalShortTermSublettingAirbnbRestrictionInput>,
) -> Result<Json<RentalShortTermSublettingAirbnbRestrictionResult>, ApiError> {
    Ok(Json(check_rental_short_term_subletting_airbnb_restriction(&b)))
}

// ---------------------------------------------------------------------------
// rental_grill_propane_bbq_restriction (iter 515): Trader-landlord balcony
// grill plus propane / LP-gas BBQ restriction framework. Five jurisdictions:
// California (Cal. Fire Code § 308.1.4 mirrors NFPA 1 § 10.10.5 with state-
// specific amendments + CFC § 308.1.6.2 LP-gas BBQ multifamily balcony +
// Cal. Health & Safety Code § 18900 + CAL FIRE advisories), New York City
// (FDNY 3 RCNY 102-01 + NYC Admin Code Title 29 NYC Fire Code + NYC FC
// 307.5 LP-gas > 1-pound cylinders prohibited on multifamily balconies +
// FDNY Bureau of Fire Investigation summons + $250-$1000 per violation),
// Massachusetts (527 CMR 1.00 Comprehensive Fire Safety Code adopted IFC +
// NFPA 1 + M.G.L. ch. 148 § 26G LP-gas storage + State Fire Marshal
// Office), Texas (Texas Local Gov't Code Ch. 235 municipal fire code
// authority + TX State Fire Marshal Office + Houston/Dallas/Austin/SA
// adopt IFC + NFPA 1), Default (NFPA 1 2018 + 2021 editions standalone
// § 10.10.5 + § 6.18). Three building types: Multifamily3PlusUnits,
// SingleFamilyOrDuplex, DetachedCottage. Seven grill types: Propane-
// TwentyPoundCylinder, PropaneOnePoundDisposable, Charcoal, Hibachi,
// Electric, NaturalGasHardwired (NFPA 1 § 10.10.5 carve-out), NoGrill.
// Six grill locations: BalconyAboveFirstFloor, GroundFloorPatioWithin-
// 10Feet, GroundFloorPatioAtLeast10Feet, DesignatedCommonAreaGrillStation,
// Indoor, NotApplicable. Nine-mode severity ladder including IndoorGrill-
// CategoricallyProhibited (100pct rent + CO death risk), NfpaOpenFlame-
// OnBalconyViolation (100pct rent + double violation when 20-pound LP-gas
// also present), NfpaPropaneAbove1PoundStoredAboveFirstFloorViolation
// (100pct rent), NfpaWithin10FeetOfStructureViolation (50pct rent),
// LeaseEnforcementRequiredTenantViolation (50pct rent + 3-day Notice to
// Cure). 17 US BBQ fire deaths + 8800 injuries annually per US Fire
// Administration. NFPA 1 § 10.10.5 10-foot setback constant.
// ---------------------------------------------------------------------------

async fn rental_grill_propane_bbq_restriction_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalGrillPropaneBbqRestrictionInput>,
) -> Result<Json<RentalGrillPropaneBbqRestrictionResult>, ApiError> {
    Ok(Json(check_rental_grill_propane_bbq_restriction(&b)))
}

// ---------------------------------------------------------------------------
// rental_radiator_steam_heat_safety (iter 517): Trader-landlord radiator
// safety framework. NYC Int 1489-2017 / Local Law 79 of 2018 amending NYC
// Admin Code § 27-2076 requires radiator covers within 90 days of WRITTEN
// tenant request when child age 12 or younger resides; cover must completely
// enclose top + sides + front with grill openings preventing child finger
// insertion. NYC Int 0925-2024 "Ben Z's Law" named after infant Bencel
// "Ben Z" Yancanay who died December 2020 from steam radiator burns —
// requires biennial radiator inspection in apartments and common areas
// where children under 6 reside; exempts owner-occupied co-ops + condos.
// HPD civil penalty up to $500 per violation. NYC steam radiators reach
// 250°F at peak heating; severe contact burns occur in < 2 seconds per Mayo
// Clinic; trader-landlord settlement exposure $500K-$2M for permanent
// pediatric disfigurement. Four jurisdictions: NewYorkCity (Int 1489-2017
// + Ben Z's Law), Boston (M.G.L. ch. 186 § 14 + 105 CMR 410.180 + 527
// CMR 1.00), Chicago (Chicago Municipal Code § 5-12-110 RLTO + § 13-
// 196-300), Default (common-law habitability + ASTM F2779-19). Five
// heating-system types: SteamRadiator, HotWaterRadiator, ElectricBaseboard,
// ForcedAirCentral, NoRadiator. Four tenant compositions: HouseholdWith-
// ChildUnder12 (Int 1489-2017 cover-request right), HouseholdWithChild-
// Under6 (Ben Z's Law biennial inspection), HouseholdWithElderlyOrImpaired,
// StandardAdult. Seven-mode severity ladder: NotApplicable, CompliantCover-
// InstalledOrNoRequest, LandlordCoverInstallationOverdue90Day (100pct rent),
// BenZLawBiennialInspectionOverdue (100pct rent), CoverMissingGrillOpening-
// ChildSafetySpec (100pct rent — ASTM F963-23 1/2-inch finger-trap spec),
// SteamBurnInjuryHabitabilityBreach (100pct rent + $500K-$2M settlement
// exposure), UnregulatedRadiatorWithoutThermostatHazard (50pct rent —
// TRV upgrade $50-$150/radiator).
// ---------------------------------------------------------------------------

async fn rental_radiator_steam_heat_safety_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalRadiatorSteamHeatSafetyInput>,
) -> Result<Json<RentalRadiatorSteamHeatSafetyResult>, ApiError> {
    Ok(Json(check_rental_radiator_steam_heat_safety(&b)))
}

// ---------------------------------------------------------------------------
// rental_property_tax_pass_through_disclosure (iter 519): Trader-landlord
// property-tax + MCI + IAI + special-assessment pass-through framework.
// Five jurisdictions: California (Cal. Const. Art. XIII A § 1 Proposition
// 13 enacted 1978 — 2pct annual cap until change-of-ownership or new
// construction triggers reassessment + Costa-Hawkins Rental Housing Act
// Cal. Civ. Code § 1954.50-.535 exempts post-1995 construction plus SFR
// plus condos from local rent control + AB 1482 statewide 5pct + CPI cap
// for non-Costa-Hawkins units + local rent boards preempt landlord-tenant
// contract on reassessment pass-through), NYC (9 NYCRR § 2522.4 Major
// Capital Improvement (MCI) amortized 12 years + Housing Stability and
// Tenant Protection Act of 2019 (HSTPA) capped MCI annual pass-through
// at 2pct of base rent + reduced amortization + 30-year removal from
// rent calendar + Individual Apartment Improvement (IAI) capped at
// $15K total over 15-year amortization + NYS DHCR enforcement +
// 9 NYCRR § 2526.1 overcharge refund + Emergency Tenant Protection Act
// § 11 treble damages), San Francisco (SF Rent Ordinance § 37 +
// § 37.7 capital improvement requires Rent Board approval + § 37.10B
// unauthorized impositions void), Boston (Mass. Gen. Laws ch. 40P rent
// control REPEALED 1994 + common-law contract + Mayor Wu rent-
// stabilization pending state enabling + M.G.L. ch. 186 § 14 quiet
// enjoyment), Default (state-specific rent control — Oregon SB 608
// statewide cap + Maine + Minneapolis ballot rent control + NJ municipal
// rent control). Six unit classes: Pre1995MultifamilyRentControlled,
// Post1995CostaHawkinsExempt, SingleFamilyOrCondo (Costa-Hawkins exempt),
// NycRentStabilized, NycMarketRate, NonRentControlled. Six pass-through
// types: PropertyTaxReassessment, MajorCapitalImprovementMci, Individual-
// ApartmentImprovementIai, SpecialAssessmentLid, OperatingCostIncrease-
// Petition, NoPassThrough. Eight-mode severity ladder including Property-
// TaxPassThroughVoidNonCommercial (100pct rent), MciExceeds2PctAnnualCap-
// Violation (50pct rent), IaiExceeds15KOver15YearCapViolation (50pct
// rent), MciNotRegisteredWithRentBoardViolation (100pct rent + DHCR
// treble damages), ChangeOfOwnershipReassessmentImproperlyPassed (100pct
// rent). Constants: 2pct MCI annual cap, 12-year MCI amortization, 30-
// year MCI removal from rent calendar, $15K IAI cap, 15-year IAI
// amortization, CA Prop 13 1978 enacted, Costa-Hawkins 1995 threshold,
// NYC HSTPA 2019.
// ---------------------------------------------------------------------------

async fn rental_property_tax_pass_through_disclosure_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalPropertyTaxPassThroughDisclosureInput>,
) -> Result<Json<RentalPropertyTaxPassThroughDisclosureResult>, ApiError> {
    Ok(Json(check_rental_property_tax_pass_through_disclosure(&b)))
}

// ---------------------------------------------------------------------------
// rental_marijuana_cultivation_restriction (iter 521): Trader-landlord
// tenant cannabis cultivation restriction framework. Eight jurisdictions:
// California (Cal. Health and Safety Code 11362.1(a)(3) + 11362.45(h)
// Prop 64 — 6 plants per residence; landlord lease prohibition expressly
// permitted), Colorado (Colo. Const. Art. XVIII 16(3)(b)(I)-(II) — 6
// plants total / 3 mature + tenant cultivation unless lease prohibits
// in writing per 16(3)(b)(II)), New York (NY Cannabis Law 222 MRTA — 3
// mature + 3 immature post-18-month delay + NY Office of Cannabis
// Management OCM + NYC LL18 disqualifies cultivation premises from STR),
// Massachusetts (M.G.L. ch. 94G 7 — 6 per person / 12 per residence +
// MA Cannabis Control Commission), Illinois (410 ILCS 705 recreational
// possession but PROHIBITS home cultivation + 410 ILCS 130 medical 5
// plants), NoHomeCultivationState (WA recreational + NJ — cultivation
// illegal regardless of lease), CannabisIllegalState (full state-statute-
// violation termination), Default (21 U.S.C. 812 CSA Schedule I + HUD
// Notice PIH 2011-25 federally-assisted prohibition + 16 states permitting
// home grow as of 2025 + DEA Schedule III rescheduling pending June 2025).
// Two housing programs: HudAssistedFederalCsaEnforcement (HUD PIH 2011-25
// + 24 C.F.R. 5.852 termination + HUD General Counsel FHA non-
// accommodation), PrivateMarketStateLawPrimary. Five cultivation
// statuses: NoCultivation, RecreationalWithinStateLimit, ExceedingState-
// PlantCountLimit, MedicalCannabisQualifyingPatient, BlackMarketCommercial-
// Cultivation. Eight-mode severity ladder including MoldOrElectricalFire-
// DamageHabitabilityBreach (100pct rent — 60-80pct relative humidity
// grow rooms + insurance exclusion), HudFederalCsaTerminationGround
// (100pct rent), BlackMarketCommercialEvictionGround (100pct rent +
// federal RICO civil-forfeiture exposure), FhaMedicalCannabisAccommodation-
// Denied (50pct rent — HUD General Counsel preemption), ExceedsState-
// PlantCountViolation (50pct rent + 3-day Notice to Cure), LandlordProhi-
// bitionEnforceable (50pct rent — explicitly permitted by state cannabis
// statute even in legal-cultivation states).
// ---------------------------------------------------------------------------

async fn rental_marijuana_cultivation_restriction_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalMarijuanaCultivationRestrictionInput>,
) -> Result<Json<RentalMarijuanaCultivationRestrictionResult>, ApiError> {
    Ok(Json(check_rental_marijuana_cultivation_restriction(&b)))
}

// ---------------------------------------------------------------------------
// rental_massachusetts_security_deposit_statute: MGL c. 186, § 15B
// (Massachusetts Security Deposit Statute). Most stringent security
// deposit statute in the US. Requires (1) one-month-rent maximum
// deposit, (2) written receipt within 30 days, (3) separate interest-
// bearing MA bank account, (4) Statement of Condition within 10 days
// with 12-pt bold-face notice, (5) 5% annual interest payment (or
// lesser actual bank interest), (6) 30-day return after termination,
// (7) itemized damages signed under penalty of perjury if retaining.
// Triple damages under § 15B(7) for ANY procedural violation plus
// court costs + reasonable attorney fees + MGL c. 93A consumer-
// protection action. Sibling cluster: rental_security_deposit_interest
// (NY/CA equivalent regimes), rental_security_deposit_return_notice,
// rental_just_cause_eviction (iter 573 — different tenant-protection
// vector), rental_application_denial_disclosure (state-specific
// regimes).
// ---------------------------------------------------------------------------

async fn rental_massachusetts_security_deposit_statute_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalMassachusettsSecurityDepositStatuteInput>,
) -> Result<Json<RentalMassachusettsSecurityDepositStatuteResult>, ApiError> {
    Ok(Json(check_rental_massachusetts_security_deposit_statute(&b)))
}

// ---------------------------------------------------------------------------
// rental_massachusetts_homes_act_eviction_sealing: Massachusetts
// Affordable Homes Act of 2024 (H.4138 / Chapter 150 of the Acts of
// 2024) HOMES Act eviction record sealing regime under Section 52.
// Signed by Governor Maura Healey on August 6, 2024 as part of the
// $5.16 billion Affordable Homes Act with ~50 policy initiatives;
// eviction sealing provisions effective May 5, 2025 (270 days after
// signing). Codified permanently at Mass. Gen. Laws Chapter 239.
// Eligibility categories: (a) cases dismissed — immediately sealable
// after appeal period; (b) cases tenant won — immediately sealable;
// (c) satisfied judgments — immediately sealable; (d) non-payment
// cases not paid — 4-year waiting period + economic hardship
// documentation + no intervening lessor action required. Once sealed,
// eviction case is no longer visible to public or tenant-screening
// and credit-reporting companies (CoreLogic SafeRent, TransUnion
// SmartMove, Experian RentBureau). Landlord inquiry prohibition:
// landlord may not ask prospective tenant about sealed eviction
// record; tenant has no obligation to disclose; deceptive use by
// landlord triggers statutory damages. Other Affordable Homes Act
// provisions: ADUs (Accessory Dwelling Units) legalized as-of-right
// statewide; public housing modernization; first-time homebuyer
// programs; low/moderate-income housing investment. Fourteen-mode
// severity ladder × two property jurisdictions × six eviction case
// outcomes × three non-payment sealing preconditions × four landlord
// actions. Trader-landlord critical for MA portfolio operators
// post-May 5, 2025: tenant-screening workflows must not access
// sealed records; landlord inquiry forms must be redrafted; denial
// based on sealed records triggers statutory damages. Sibling
// cluster: rental_eviction_record_sealing (multi-state cross-
// reference; MA HOMES Act = recent entry), rental_just_cause_
// eviction (parallel tenant protection regime), rental_application_
// denial_disclosure (FCRA-overlapping screening regime), rental_
// massachusetts_security_deposit_statute (MA companion regime),
// rental_eviction_notices.
// ---------------------------------------------------------------------------

async fn rental_massachusetts_homes_act_eviction_sealing_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalMassachusettsHomesActEvictionSealingInput>,
) -> Result<Json<RentalMassachusettsHomesActEvictionSealingResult>, ApiError> {
    Ok(Json(check_rental_massachusetts_homes_act_eviction_sealing(&b)))
}

// ---------------------------------------------------------------------------
// rental_attached_garage_carbon_monoxide_disclosure (iter 523): Trader-
// landlord attached-garage CO detector disclosure framework. Six
// jurisdictions: California (SB 183 Carbon Monoxide Poisoning Prevention
// Act of 2010 + Cal. Health and Safety Code 17926-17926.2 + Cal. Civ.
// Code 1947.13 — CO alarm required in all dwelling units with fossil-
// fuel-burning appliances + fireplaces + attached garages), New York
// (NY Public Health Law 1399-bbb-1 Amanda's Law named after Amanda
// Hansen who died age 16 from CO poisoning + NYC Admin Code 27-2046.2
// + HPD Local Law 7 of 2004 + 1 RCNY 12-12 + $25 per detector per day
// HPD penalty), Washington (RCW 19.27.530 effective January 1 2011 new
// construction / January 1 2013 all residential occupancies + WA DOH),
// Connecticut (C.G.S. 29-292(b) + 47a-7 + CT State Fire Marshal
// Office), Massachusetts (M.G.L. ch. 148 26F Nicole's Law named after
// Nicole Garofalo who died age 7 from CO poisoning effective March 31
// 2006 + 527 CMR 1.00), Default (CDC public-health surveillance + UL
// 2034 federal standard + 30+ state mandates). Four CO exposure risks:
// AttachedGarageSharedBoundary, DetachedGarageWithSharedHvacReturn,
// InUnitFossilFuelAppliance, MinimalNoFossilFuelOrGarage. Four detector
// statuses: UL2034InstalledAndCurrent, InstalledButPastEndOfLifeWindow
// (7-year UL 2034 manufacturer-listed end-of-life), InstalledButNotUL2034
// Listed, NotInstalledViolation. Seven-mode severity ladder including
// CoExposureInjuryHabitabilityBreach (100pct rent + $1M-$5M settlement +
// 400 ppm lethal within 5 minutes vehicle idling per CDC), Detector-
// NotInstalledStatutoryViolation (100pct rent), DetectorPastEndOfLife-
// ReplacementRequired (50pct rent), DetectorNotUL2034ListedNonCompliant
// (50pct rent), DisclosureRequiredAtLeaseSigning (50pct rent). CDC
// reports 450 US annual non-fire CO deaths + 20K nonfatal injuries —
// CO is leading cause of poison-related death in US.
// ---------------------------------------------------------------------------

async fn rental_attached_garage_carbon_monoxide_disclosure_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalAttachedGarageCarbonMonoxideDisclosureInput>,
) -> Result<Json<RentalAttachedGarageCarbonMonoxideDisclosureResult>, ApiError> {
    Ok(Json(check_rental_attached_garage_carbon_monoxide_disclosure(&b)))
}

// ---------------------------------------------------------------------------
// rental_pet_breed_restriction_disclosure (iter 525): Trader-landlord pet
// breed restriction disclosure + FHA reasonable-accommodation framework.
// Six jurisdictions: Maryland (Pet Policy Transparency Act of 2025 + no
// state preemption of BSL + 21 states permit local BSL), Michigan (no
// state preemption + local BSL permitted + Michigan Civil Rights
// Commission), Nevada (SB 245 effective 2025 prohibits landlord
// liability insurance breed discrimination + Nevada Division of
// Insurance), California (Cal. Civ. Code § 1942.7 + § 54.2 service-
// animal access + DFEH + AB 468 ESA 30-day client/provider relationship),
// New York (NY Civ. Rights Law § 47-a + NY RPL § 235-b + NYC Admin
// § 8-107(15) + NYS Human Rights Law § 296(2-a) + NYC HPD + NY Division
// of Human Rights), Default (federal FHA 42 U.S.C. § 3604(f)(3)(B) +
// 24 C.F.R. § 100.204 + HUD Notice FHEO-2020-01 dated January 28
// 2020 + 29 states preempt local BSL + CDC 4.5M annual US dog bites).
// Three animal types: AdaServiceAnimal (Title III task-trained),
// FhaEmotionalSupportAnimal (HUD-recognized disability accommodation),
// RegularPetNoFhaProtection. Four breed restriction types: NoBreed-
// Restriction, SpecificBreedsProhibited, WeightOrSizeRestriction,
// TotalPetProhibition. Eight-mode severity ladder including
// DiscriminatoryDenialFhaViolationFineExposure (100pct rent + $25,645
// HUD first-offense + $128,225 subsequent-offense civil penalty per
// 24 C.F.R. § 180.671 + 42 U.S.C. § 3610), FhaPreemptsBreedRestriction-
// AssistanceAnimal (50pct rent), InsuranceBreedBanIneffectiveVsFha
// (50pct rent — Liberty Mutual + State Farm + Allstate + Farmers +
// Nationwide common breed bans cannot justify denial), MdPetPolicy-
// TransparencyActDisclosureRequired (50pct rent), NvSb245Insurance-
// BreedDiscriminationProhibited (50pct rent), DocumentationRequested-
// FromTenantPermissible (ADA Title III two-question protocol),
// BreedRestrictionEnforceableNonAssistance (CDC 4.5M dog bites
// annually + $50K-$500K premises-liability exposure).
// ---------------------------------------------------------------------------

async fn rental_pet_breed_restriction_disclosure_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalPetBreedRestrictionDisclosureInput>,
) -> Result<Json<RentalPetBreedRestrictionDisclosureResult>, ApiError> {
    Ok(Json(check_rental_pet_breed_restriction_disclosure(&b)))
}

// ---------------------------------------------------------------------------
// rental_emergency_action_plan_high_rise (iter 527): Trader-landlord high-
// rise residential Emergency Action Plan (EAP) compliance framework. Five
// jurisdictions: NewYorkCity (NYC Local Law 26 of 2004 post-9/11 EAP
// expansion + FDNY 3 RCNY § 404-01 + NYC Admin Code § 404.2.1 + Fire Code
// Title 29 + § 28-202.1 civil penalty + Fire Safety Director F-32
// certification + 24/7 on-site presence + $1,000-$25,000 per violation
// per day FDNY civil penalty), Chicago (Municipal Code § 13-160-070 +
// Chicago Fire Department + post-2003 Cook County Administration Building
// fire enhanced requirements), LosAngeles (Cal. Fire Code + LAMC §
// 57.4901 + LA County Code Title 32 + LAFD certification), International
// FireCodeAdopted (IFC § 404.2.1 Comprehensive fire safety/emergency
// action plan Level 1 + § 404.3 implementation + § 404.4 staff training),
// Default (NFPA 1 § 10.8 high-rise framework + § 11.10 EAP + NFPA 101
// Life Safety Code § 4.7 + OSHA 29 C.F.R. § 1910.38). High-rise threshold:
// ≥ 75 feet OR 7+ stories. Three building classifications: HighRise-
// Residential75FtOr7Stories (full EAP), MidRiseResidential4To6Stories
// (limited), LowRiseResidential1To3Stories (inapplicable). Seven compliance
// statuses: AllRequirementsCurrent, FireSafetyDirectorNotPresent,
// FspEapNotFiledWithAhj, AnnualFireDrillMissed, TenantInstructionsNot-
// Distributed, AccessibleEvacuationMissing (FHA + ADA + NFPA 101 § 7.2.12
// Areas of Refuge + IFC § 1009 accessible means of egress), Emergency-
// IncidentDuringNonCompliance. Eight-mode severity ladder including
// PostIncidentNonComplianceFatalityRisk (100pct rent + $5M-$50M wrongful-
// death litigation), AccessibleEvacuationFhaAdaViolation (100pct rent),
// FireSafetyDirectorPresenceViolation (50pct rent + $1,000-$25,000/day
// FDNY penalty).
// ---------------------------------------------------------------------------

async fn rental_emergency_action_plan_high_rise_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalEmergencyActionPlanHighRiseInput>,
) -> Result<Json<RentalEmergencyActionPlanHighRiseResult>, ApiError> {
    Ok(Json(check_rental_emergency_action_plan_high_rise(&b)))
}

// ---------------------------------------------------------------------------
// rental_eviction_record_sealing_screening: Multi-jurisdiction
// eviction-record sealing and tenant-screening compliance. Five
// regimes: CA AB 2819 / Cal. Code Civ. Proc. § 1161.2 (60-day
// automatic masking + permanent seal unless landlord prevails
// within 60 days), WA SB 5160 / RCW 59.18.367 (3-year reporting
// lookback limit), NY RPAPL § 745(2)(c)(iv) (5-year housing-court
// records lookback), IL HB 1561 / 735 ILCS 5/9-121.5 (Cook County
// pilot court-order sealing with defendant-friendly presumption),
// MN Minn. Stat. § 484.014 (automatic 3-year expungement when
// landlord did not prevail). FCRA 15 U.S.C. § 1681c federal 7-year
// floor applies in default-no-regime jurisdictions. Sibling cluster:
// adverse_action_notice (FCRA § 615 adverse-action notice when
// relying on consumer report), rental_application_denial_disclosure
// (state-specific written-denial regimes), rental_tenant_criminal_
// background_screening (parallel criminal-record sealing regime),
// rental_source_of_income_discrimination (FEHA + parallel state
// fair-housing protection that overlays screening).
// ---------------------------------------------------------------------------

async fn rental_eviction_record_sealing_screening_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalEvictionRecordSealingScreeningInput>,
) -> Result<Json<RentalEvictionRecordSealingScreeningResult>, ApiError> {
    Ok(Json(check_rental_eviction_record_sealing_screening(&b)))
}

// ── /rental-illegal-lockout-self-help-eviction (iter 529) ───────────────────
// POST endpoint for illegal-lockout / self-help-eviction landlord exposure
// across CA + NY + TX + WA + FL + IL + Default jurisdictions. Self-help
// eviction (lock change without court order, utility shutoff,
// door/window removal, belongings dump, bootlock installation) is prohibited
// in every US jurisdiction with a residential tenancy code. Landlord must use
// judicial process; the penalty regime varies sharply by state.
//
// CA Civ. Code § 789.3 — $100/day continued + $250 statutory floor + actual
// damages + attorney fees + potential punitive damages. CA AG Bulletin
// 2022-DLE-05 directs law enforcement to treat lockout as criminal trespass /
// unlawful detainer interference.
//
// NY RPAPL § 853 + RPL § 768 — treble damages on property lost + cost of
// alternative accommodation + value of permanently lost tenancy. RPL § 768
// makes unlawful eviction a Class A misdemeanor — criminal exposure in
// addition to civil. Rent-stabilized / rent-controlled tenancy value often
// six figures.
//
// TX Prop. Code § 92.0081 — $1,000 + one month rent + actual damages + court
// costs + attorney fees. Three safe-harbor branches: bona-fide
// repairs/construction/emergency, abandoned-contents removal, OR
// rent-delinquent door lock change with strict procedural compliance (24/7
// key availability, no replacement-key fee, written notice on door with
// statutory disclosures per § 92.0081(c)-(f)). Partial procedural compliance
// is treated as violation.
//
// WA RCW 59.18.290 — greater of actual damages or 3× monthly rent + court
// costs + attorney fees. Tenant may terminate rental agreement.
//
// FL Stat. § 83.67 — greater of actual+consequential damages or 3 months'
// rent + costs + attorney fees. Statute expressly prohibits bootlocks.
//
// IL 735 ILCS 5/9-101 et seq. Forcible Entry and Detainer Act — court
// process required; common-law wrongful eviction baseline. Chicago RLTO
// § 5-12-160 adds two months' rent OR twice damages (greater) + attorney
// fees for Chicago-located rentals.
//
// Eleven-mode severity ladder: NotApplicable, NoLockoutLawfulCourtProcess,
// TexasSafeHarborSatisfiedNoViolation, TexasRentDelinquentLockoutProcedural-
// FailureViolation, CaliforniaCivCode789_3PerDayPenaltyViolation,
// NewYorkRpapl853TrebleDamagesViolation,
// TexasPropCode92_0081MinThousandPlusMonthRentViolation,
// WashingtonRcw59_18_290TripleMonthlyRentViolation,
// FloridaStat83_67ThreeMonthsRentViolation,
// IllinoisForcibleEntryDetainerCommonLawWrongfulEvictionViolation,
// DefaultJurisdictionStateLawSelfHelpEvictionViolation.

async fn rental_illegal_lockout_self_help_eviction_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalIllegalLockoutSelfHelpEvictionInput>,
) -> Result<Json<RentalIllegalLockoutSelfHelpEvictionResult>, ApiError> {
    Ok(Json(check_rental_illegal_lockout_self_help_eviction(&b)))
}

// ── /rental-retaliation-prohibition (iter 531) ──────────────────────────────
// POST endpoint for landlord retaliation prohibition exposure across CA + NY
// (1-yr + 3-yr stabilized) + WA + TX + FL + IL (Chicago RLTO + statewide
// common-law) + Default jurisdictions. Every state with a residential
// tenancy code prohibits landlord retaliation for tenant exercise of
// protected rights (habitability complaint to landlord/government, joining
// tenants' association, exercising repair-and-deduct or rent-withhold,
// filing fair-housing complaint, commencing legal proceeding). Prohibition
// operates via rebuttable presumption that adverse landlord action within
// the statutory window is retaliatory.
//
// CA Civ. Code § 1942.5 — 180-day presumption + $100-$2,000 per retaliatory
// act punitive damages (showing fraud/oppression/malice) + actual damages +
// attorney fees. Ellis Act withdrawal from rental market is documented
// legitimate-business-reason rebuttal.
//
// NY RPL § 223-b — 1-year presumption (3 years for rent-stabilized /
// rent-controlled units). Tenant defense to eviction + reinstatement.
// Rent-stabilized tenancy value often six figures on permanent loss.
//
// WA RCW 59.18.240 + § 59.18.250 — 90-day presumption (shortest among
// surveyed states) + remedies under RCW 59.18.060 (greater of actual
// damages or 3× monthly rent) + attorney fees.
//
// TX Prop. Code §§ 92.331-92.335 — 6-month (180-day) presumption +
// statutory $500 civil penalty + one month's rent + actual damages + court
// costs + attorney fees.
//
// FL Stat. § 83.64 — rebuttable presumption WITHOUT codified window;
// common-law reasonable-temporal-proximity analysis (Florida courts apply
// 90-180-day heuristic).
//
// IL Chicago RLTO § 5-12-150 — 1-year presumption + GREATER OF two months'
// rent or twice damages + attorney fees + tenant may recover possession or
// terminate. Illinois statewide has NO codified statute; common-law
// retaliatory-eviction doctrine via Clore v. Fredman, 59 Ill. 2d 20 (1974)
// operates as defense to forcible-entry-and-detainer.
//
// Fourteen-mode severity ladder: NotApplicable, NoProtectedActNoPresumption-
// Triggered, NoAdverseActionNoRetaliationClaim, LandlordRebuttalProbably-
// Successful, OutsidePresumptionWindowBurdenOnTenant, CaliforniaCiv1942_5-
// PresumptionRetaliationPunitiveDamages, NewYorkRpl223BOneYearPresumption,
// NewYorkRpl223BThreeYearPresumptionRentStabilized,
// WashingtonRcw59_18_240NinetyDayPresumption,
// TexasPropCode92_331SixMonthPresumptionFiveHundredPenalty,
// FloridaStat83_64CommonLawTemporalProximityPresumption,
// IllinoisChicagoRlto5_12_150OneYearPresumptionTwoMonthRent,
// IllinoisStatewideCommonLawCloreVFredmanDefense,
// DefaultJurisdictionStateLawRetaliationPresumption.

async fn rental_retaliation_prohibition_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalRetaliationProhibitionInput>,
) -> Result<Json<RentalRetaliationProhibitionResult>, ApiError> {
    Ok(Json(check_rental_retaliation_prohibition(&b)))
}

// ── /rental-landlord-notice-to-enter (iter 533) ─────────────────────────────
// POST endpoint for landlord notice-to-enter requirement compliance across
// CA + WA + FL + IL Chicago/statewide + CO + TX + NYC + Default jurisdictions.
// Most US states require landlords to provide advance notice before entering
// a tenant's occupied rental unit for non-emergency purposes (necessary
// repairs, routine inspection, showing to prospective buyers/tenants/
// appraisers, court-ordered access). Unannounced entry breaches the implied
// covenant of quiet enjoyment under common law and creates a statutory
// violation in jurisdictions with codified entry rules.
//
// CA Civ. Code § 1954: 24 hours WRITTEN notice (48 hours for initial
// moveout inspection under § 1950.5(f)). Three permissible purposes only:
// necessary repairs, showing to prospective buyers/tenants/appraisers,
// court-ordered.
//
// WA RCW 59.18.150: 48 hours notice generally; 24 hours for showing to
// prospective buyers/tenants/appraisers.
//
// FL Stat. § 83.53: "reasonable notice" required; 12 hours recognized as
// reasonable for tenant-requested repairs. Oral notice permitted.
//
// IL Chicago RLTO § 5-12-050: 2 days (48 hours) WRITTEN notice required.
// Illinois statewide has no codified notice statute — common-law
// reasonable-notice standard applies (24-hour default).
//
// CO Rev. Stat. § 38-12-510 (HB 23-1095, effective Aug 7 2023): 48 hours
// WRITTEN notice required.
//
// TX Prop. Code: NO statewide notice statute; lease terms control. Many
// local ordinances impose 24-hour requirements.
//
// NYC Admin Code § 27-2008 + DHCR rent-stab regs: reasonable advance
// notice; common-law standard.
//
// Eight entry purposes (NecessaryRepairsOrMaintenance, InspectionRoutine-
// OrAnnual, ShowingToProspectiveBuyersTenantsOrAppraisers, InitialMoveOut-
// InspectionCaSection1950_5F, EmergencyImmediateThreat, TenantAbandonedUnit,
// CourtOrderedEntry, UnpermittedOrPretextual) plus four notice methods
// (WrittenAdvanceNotice, OralAdvanceNotice, NoNoticeGiven,
// NoticeReceivedAfterEntry).
//
// Nine-mode severity ladder: NotApplicable, EmergencyEntryNoNoticeRequired,
// TenantAbandonedUnitNoNoticeRequired, CourtOrderedEntryAuthorized,
// CompliantWrittenNoticeWithinWindow, CompliantOralNoticeWithinWindow,
// UnpermittedPurposeQuietEnjoymentBreach,
// InsufficientNoticeTimeStatutoryViolation,
// NoNoticeGivenStatutoryViolationPlusQuietEnjoymentBreach.

async fn rental_landlord_notice_to_enter_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalLandlordNoticeToEnterInput>,
) -> Result<Json<RentalLandlordNoticeToEnterResult>, ApiError> {
    Ok(Json(check_rental_landlord_notice_to_enter(&b)))
}

// ── /rental-security-deposit-return-notice (iter 535) ───────────────────────
// POST endpoint for security-deposit return notice + itemized-deduction
// compliance framework across nine jurisdictions. Every US state with a
// residential tenancy code imposes a statutory deadline for returning a
// tenant's security deposit after move-out and requires an itemized
// written statement of any deductions claimed. Failure to comply forfeits
// the landlord's right to retain any portion of the deposit and exposes
// the landlord to multiplied damages (2× to 3× the deposit) plus attorney
// fees in many jurisdictions.
//
// CA Civ. Code § 1950.5(g): 21 calendar days + itemized statement +
// receipts for deductions ≥ $125; § 1950.5(l) bad-faith retention 2×
// deposit + actual damages + attorney fees.
//
// NY Gen. Oblig. Law § 7-108(1-a)(e): 14 days + itemized deduction list;
// failure forfeits retention right + 2× damages.
//
// WA RCW 59.18.280: 30 days (extended from 21 days) + full itemized
// statement + documentation.
//
// TX Prop. Code § 92.103: 30 days + itemized written description; § 92.109
// bad-faith retention = $100 + 3× wrongfully withheld + attorney fees.
//
// FL Stat. § 83.49(3)(a): 15 days (no claim) or 30 days (with claim by
// CERTIFIED-MAIL notice + itemized statement). First-class mail of
// deduction claim fails Florida's specific method requirement.
//
// IL Chicago RLTO § 5-12-080: 45 days + 2× deposit + attorney fees for
// violation. IL statewide (765 ILCS 710/1): 30 days for buildings with 5+
// units.
//
// MA Gen. L. ch. 186 § 15B(4)(iii): 30 days + sworn statement + interest;
// § 15B(7) bad-faith retention = 3× deposit + interest + attorney fees.
//
// Seven-mode severity ladder: NotApplicable,
// CompliantWithinStatutoryWindow, CompliantNoDeductionsFullReturn,
// LateButReturnedRiskOfForfeitureOnly,
// DeductionsWithoutItemizedStatementForfeitsRetentionRight,
// BadFaithRetentionDoubleOrTripleDamages,
// NoDeliveryFullForfeitureAndStatutoryDamages.

async fn rental_security_deposit_return_notice_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalSecurityDepositReturnNoticeInput>,
) -> Result<Json<RentalSecurityDepositReturnNoticeResult>, ApiError> {
    Ok(Json(check_rental_security_deposit_return_notice(&b)))
}

// ── /rental-late-fee-cap (iter 537) ─────────────────────────────────────────
// POST endpoint for late-fee cap + grace-period compliance across ten
// jurisdictions. State law sharply constrains residential late-fee
// charges, both as to maximum amount and minimum grace period before a
// fee may attach. Excessive late fees are unenforceable under common-law
// liquidated-damages doctrine (must be reasonable estimate of actual
// damages, not penalty) and create statutory exposure under consumer-
// protection statutes (deceptive-practices, unconscionability).
//
// NY RPL § 238-a (HSTPA 2019): LESSER of $50 OR 5% of monthly rent; 5-day
// grace period.
//
// WA RCW 59.18.170: no statewide dollar cap, but landlord may not charge
// any late fee until rent is more than 5 days late.
//
// CO Rev. Stat. § 38-12-105 (HB 23-1099): GREATER of $50 OR 5% of monthly
// rent; 7-day grace period.
//
// IL Chicago RLTO § 5-12-140(h): $10 + 5% per month for rent over $500.
//
// TX Prop. Code § 92.019: statutory safe-harbor cap of 10% for properties
// with 4 or fewer units, 12% for properties with 5 or more units; 2-day
// grace period.
//
// CA Civ. Code § 1671: reasonable-estimate-of-damages standard; no
// statutory dollar cap; industry standard 5% or $50.
//
// MA Gen. L. ch. 186 § 15B(1)(c): NO late fee for first 30 days after
// rent due (longest grace period of surveyed states).
//
// FL: NO statewide cap or grace period for residential tenancies (Fla.
// Stat. § 83.808 covers only mobile-home tenancies).
//
// Seven-mode severity ladder: NotApplicable,
// LateFeeWithinCapAndGraceCompliant,
// LateFeeChargedBeforeGracePeriodExpired,
// LateFeeExceedsStatutoryCap,
// NoStatutoryCapCommonLawReasonablenessTest,
// NoLateFeeChargedCompliant,
// LongestGracePeriodMassachusettsThirtyDays.

async fn rental_late_fee_cap_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalLateFeeCapInput>,
) -> Result<Json<RentalLateFeeCapResult>, ApiError> {
    Ok(Json(check_rental_late_fee_cap(&b)))
}

// ---------------------------------------------------------------------------
// rental_local_law_87_energy_audit_retro_commissioning: NYC Local
// Law 87 of 2009 — Energy Audits & Retro-Commissioning. Part of
// Greener, Greater Buildings Plan (GGBP) with LL 84 (benchmarking)
// and LL 97 (emissions). NYC Admin Code § 28-308 + § 28-308.7
// (penalty schedule $3K first / $5K subsequent year). Covers
// buildings > 50,000 gross sqft. Filing schedule based on tax
// block last digit (e.g., tax block ending in 5 → file 2015/2025/
// 2035). Energy Efficiency Report (EER) every 10 years: ASHRAE
// Level II energy audit + retro-commissioning study of base
// building systems. Qualified Professional must be NY-licensed PE
// or RA + NOT building staff. 2025 EER filing extended to March 31,
// 2026 (one-time extension). NYC DOB will not accept outstanding
// EER submission if outstanding penalties not paid. Sibling cluster:
// rental_energy_benchmarking (NYC LL 84 annual), rental_climate_
// mobilization_act_ll97_emissions (iter 587 NYC LL 97 emissions),
// rental_facade_inspection_fisp_local_law_11 (iter 583 NYC LL 11),
// rental_gas_piping_inspection_local_law_152 (iter 585 NYC LL 152),
// rental_cooling_tower_inspection_local_law_77 (iter 589 NYC LL 77).
// ---------------------------------------------------------------------------

async fn rental_local_law_87_energy_audit_retro_commissioning_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalLocalLaw87EnergyAuditRetroCommissioningInput>,
) -> Result<Json<RentalLocalLaw87EnergyAuditRetroCommissioningResult>, ApiError> {
    Ok(Json(check_rental_local_law_87_energy_audit_retro_commissioning(&b)))
}

// ---------------------------------------------------------------------------
// rental_local_law_88_lighting_upgrades_sub_metering: NYC Local Law
// 88 of 2009. Part of Greener, Greater Buildings Plan (GGBP) with
// LL 84 (benchmarking), LL 87 (energy audits), LL 97 (emissions).
// Enacted Dec 28, 2009. NYC Admin Code § 28-310 (lighting upgrades)
// + § 28-311 (sub-metering). Covers buildings ≥ 25,000 sqft (lowered
// from original 50,000 threshold). Two compliance vectors: (1)
// lighting upgrades to current NYCECC standards by Jan 1, 2025; (2)
// electrical sub-meters in each commercial tenant space > 5,000
// sqft by Jan 1, 2025. § 28-311.4 monthly tenant billing statements
// + reports filed by May 1, 2025 with $115 filing fee. Exemptions:
// R-2 multifamily + R-3 1-2 family + already-metered tenant spaces.
// § 28-311.5 penalty schedule: $500/year per tenant space without
// submeter + $1,500/year per unfiled report. Sibling cluster:
// rental_energy_benchmarking (LL 84), rental_local_law_87_energy_
// audit_retro_commissioning (iter 599 LL 87), rental_climate_
// mobilization_act_ll97_emissions (iter 587 LL 97).
// ---------------------------------------------------------------------------

async fn rental_local_law_88_lighting_upgrades_sub_metering_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalLocalLaw88LightingUpgradesSubMeteringInput>,
) -> Result<Json<RentalLocalLaw88LightingUpgradesSubMeteringResult>, ApiError> {
    Ok(Json(check_rental_local_law_88_lighting_upgrades_sub_metering(&b)))
}

// ── /rental-tenant-criminal-background-screening (iter 539) ─────────────────
// POST endpoint for tenant criminal-background-screening compliance under
// HUD 2016 guidance + state and local Fair Chance Housing laws. HUD's
// April 4, 2016 guidance (Helen Kanovsky General Counsel memorandum)
// establishes that blanket criminal-history bans create a disparate
// impact on protected classes (race, color, national origin) under the
// Fair Housing Act (42 U.S.C. § 3601 et seq.) absent a substantial,
// legitimate, nondiscriminatory interest backed by individualized
// assessment.
//
// HUD 2016 federal floor: arrest records cannot serve as basis for
// adverse action (per se discriminatory because arrest is not proof of
// criminal conduct); conviction-record bans subject to disparate-impact
// analysis; individualized assessment required (nature/severity of
// offense, time elapsed, relevance to tenancy).
//
// NYC Local Law 24 of 2024 (Fair Chance for Housing Act, effective Jan
// 1, 2025): prohibits consideration of criminal history until other
// qualifications determined; 3-year lookback for misdemeanors, 5-year
// for felonies; sex crimes only convictions categorically considerable.
//
// California AB 2052 + Cal. Civ. Code § 1786.21: sequential-screening
// regime — criminal background check only after applicant meets other
// qualifications. 7-year lookback for conviction-only consideration.
//
// New Jersey Fair Chance in Housing Act (P.L. 2021, c. 197, N.J.S.A.
// 46:8-52 et seq.): housing providers cannot ask about criminal history
// before extending conditional offer. 1-year lookback for misdemeanors,
// 4-year for indictable felonies. Individualized assessment required
// post-conditional-offer.
//
// Illinois HB 4366 (2024): Fair Housing Act amendment restricting
// criminal-history screening for federally-assisted housing.
//
// Eight-mode severity ladder: NotApplicable,
// NoCriminalHistoryNoSection804OrFchaViolation,
// CompliantSequentialScreeningAndIndividualizedAssessment,
// SexOffenseExceptionCategoricallyPermissiblePerJurisdiction,
// ArrestRecordReliancePerSeDiscriminatoryHud2016,
// PreApplicationInquiryViolatesFairChanceLaw,
// BlanketBanWithoutIndividualizedAssessmentDisparateImpact,
// LookbackWindowExceededFairChanceViolation.

async fn rental_tenant_criminal_background_screening_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalTenantCriminalBackgroundScreeningInput>,
) -> Result<Json<RentalTenantCriminalBackgroundScreeningResult>, ApiError> {
    Ok(Json(check_rental_tenant_criminal_background_screening(&b)))
}

// ── /rental-source-of-income-discrimination (iter 541) ──────────────────────
// POST endpoint for source-of-income (SOI) discrimination compliance across
// nine jurisdictions. SOI prohibitions make it unlawful for housing
// providers to refuse to rent, set different terms, or otherwise
// discriminate against applicants based on their lawful source of income
// — including Section 8 Housing Choice Vouchers under 42 U.S.C. § 1437f,
// VASH vouchers, SSI, TANF, Social Security, public assistance, and
// similar federal/state/local rental subsidies. 23 states + DC + 100+
// municipalities prohibit SOI discrimination by statute or ordinance,
// but the federal Fair Housing Act (42 U.S.C. § 3601 et seq.) does NOT
// explicitly include SOI as a protected class.
//
// CA SB 329 (Cal. Gov. Code §§ 12921 + 12955; eff. Jan 1, 2020): SOI
// explicit protected class.
//
// NY State Human Rights Law (NY Exec. Law § 296(2-a)) + NYC HRL (NYC
// Admin. Code § 8-107(5)): SOI explicit protected class; effective
// April 12, 2019.
//
// NJ LAD (N.J.S.A. 10:5-12.5; 2026 amendments): SOI protected;
// landlords prohibited from applying minimum-income requirements not
// based exclusively on tenant's portion of rent.
//
// WA RCW 59.18.255 (HB 2578; eff. Sept 30, 2018): voucher-participation
// protected class; landlord must deduct voucher amount from rent when
// applying rent-to-income screening ratios.
//
// MA Gen. L. ch. 151B § 4(10) + 4(11): SOI protected including Section 8.
//
// IL statewide silent; Chicago Fair Housing Ordinance + Cook County
// Human Rights Ordinance + Urbana prohibit SOI by ordinance.
//
// Federal Fair Housing Act floor: voluntary for landlords absent
// state/local SOI law; racially-disparate refusal still creates 42
// U.S.C. § 3604 disparate-impact liability per Texas Dept. of Housing
// v. Inclusive Communities Project, 576 U.S. 519 (2015).
//
// Eight-mode severity ladder: NotApplicable,
// EmploymentOnlyIncomeNoSoiAnalysisTriggered,
// CompliantSoiAcceptanceWithTenantPortionScreening,
// IllinoisStatewideNoStateCoverageNoSoiClaim,
// FederalFhaDisparateImpactAvailable,
// StateSoiStatuteViolationActualAndPunitiveDamages,
// NjLadMinimumIncomeRuleViolationTenantPortionOnly,
// WashingtonRcw59_18_255VoucherDeductionFailureViolation.

async fn rental_source_of_income_discrimination_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalSourceOfIncomeDiscriminationInput>,
) -> Result<Json<RentalSourceOfIncomeDiscriminationResult>, ApiError> {
    Ok(Json(check_rental_source_of_income_discrimination(&b)))
}

// ── /rental-tenant-abandoned-personal-property (iter 543) ───────────────────
// POST endpoint for tenant-abandoned personal property handling compliance
// across eight jurisdictions. When a tenant vacates and leaves personal
// property behind, the landlord must follow state-specific procedures to
// lawfully dispose, sell, or store the property. Conversion of tenant
// property is a strict-liability tort independent of any breach-of-contract
// claim; punitive damages available for willful or bad-faith disposal.
//
// CA Civ. Code §§ 1983-1991: written notice + 15-day claim window
// (personal delivery) or 18-day (mailed); $700 sale-vs-disposal threshold
// (§ 1988); auction sale if value > $700, landlord discretion if ≤ $700.
//
// WA RCW 59.18.310: 45-day storage window from notice mailed or
// personally delivered; deceased-tenant exception.
//
// TX Prop. Code § 54.045 + § 92.014: landlord lien procedure; 30-day
// notice before sale required by BOTH first-class AND certified mail
// return receipt requested; sale proceeds applied first to delinquent
// rents + reasonable packing/moving/storage/sale costs per § 54.046.
//
// FL Stat. ch. 715 (§§ 715.10-715.111): OPTIONAL procedure; minimum
// 10-day claim window (personal delivery) or 15-day (mailed); $500
// sale-vs-disposal threshold; if < $500, landlord may retain or dispose
// at discretion per § 715.109 + § 715.107.
//
// IL: no specific landlord-tenant abandoned-property statute statewide;
// common-law reasonable-time standard + 765 ILCS 1026 Revised Uniform
// Unclaimed Property Act for bona-fide unclaimed property after escheat.
//
// MA Gen. L. ch. 239 § 4 + ch. 105A: court-supervised storage required.
//
// CO Rev. Stat. § 38-20-116: 30-day notice required before sale.
//
// Nine-mode severity ladder: NotApplicable,
// CompliantStoredPendingTenantClaim,
// CompliantBelowSaleValueThresholdLandlordRetention,
// CompliantPublicAuctionSaleAfterNoticeWindow,
// PrematureDisposalConversionTortLiability,
// NoticeNotGivenStrictLiabilityConversion,
// NoticeDeliveryMethodNonCompliantPerJurisdiction,
// TexasCertifiedAndFirstClassMailDualRequirementViolated,
// CommonLawReasonableTimeUnverified.

async fn rental_tenant_abandoned_personal_property_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalTenantAbandonedPersonalPropertyInput>,
) -> Result<Json<RentalTenantAbandonedPersonalPropertyResult>, ApiError> {
    Ok(Json(check_rental_tenant_abandoned_personal_property(&b)))
}

// ---------------------------------------------------------------------------
// rental_texas_hb_2127_state_preemption: Texas HB 2127 of 2023 (Texas
// Regulatory Consistency Act; informally "Death Star Bill") state
// preemption of local landlord-tenant ordinances. Passed by Texas
// Legislature 88th Regular Session May 2023; effective September 1,
// 2023. Bars cities and counties from passing ordinances in 8 broad
// chapters of Texas state code: Agriculture, Business and Commerce,
// Finance, Insurance, Labor, Local Government, Natural Resources,
// Occupations, plus Property Code. Private right of action: HB 2127
// authorizes individuals and trade associations (Texas Apartment
// Association, real estate developer groups) to sue cities/counties
// for violations. Landlord-tenant implications: preempts local
// rules on rent notices, eviction notice provisions, source-of-
// income protections, tenants' bill of rights ordinances, proactive
// apartment inspections programs, late fee caps, security deposit
// caps. Affected: San Antonio Tenant Bill of Rights, San Antonio
// Proactive Apartment Inspections Program, Austin code enforcement
// rules, Dallas tenant protections, Houston rental rules. Texas
// Property Code Chapter 92 uniform statewide framework. Court
// status: Travis County District Court Judge Maya Guerra Gamble
// ruled UNCONSTITUTIONAL on August 30, 2023 in City of Houston v.
// State of Texas (joined by San Antonio and El Paso) but did NOT
// enjoin enforcement; State appealed; HB 2127 took effect September
// 1, 2023 pending appellate review. Article XI, Section 5 of the
// Texas Constitution home rule city challenge pending. Nine-mode
// severity ladder × two property jurisdictions × ten local ordinance
// categories × four enforcement actors × three home rule city
// statuses. Trader-landlord critical for TX operators in Houston,
// Dallas, San Antonio, Austin, Fort Worth, El Paso, Arlington who
// may invoke HB 2127 private right of action to overturn local
// landlord-tenant ordinances and reduce compliance burden. Sibling
// cluster: rental_florida_hb_1417_state_preemption (iter 657 — FL
// companion preemption regime), rental_just_cause_eviction (TX has
// none post-preemption), rental_rent_control_stabilization (TX
// ordinances preempted), rental_source_of_income_discrimination
// (TX local protections preempted), rental_late_fee_caps (TX local
// caps preempted), rental_eviction_notices, rental_tenant_bill_of_
// rights_handout (preempted in TX).
// ---------------------------------------------------------------------------

async fn rental_texas_hb_2127_state_preemption_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalTexasHb2127StatePreemptionInput>,
) -> Result<Json<RentalTexasHb2127StatePreemptionResult>, ApiError> {
    Ok(Json(check_rental_texas_hb_2127_state_preemption(&b)))
}

// ---------------------------------------------------------------------------
// rental_tenant_bill_of_rights_handout: multi-state tenant
// disclosure / handout compliance. Michigan Truth in Renting Act
// (Public Act 454 of 1978; MCL § 554.631+) — required statutory
// notice in ≥ 12-point type or 1/8 inch legible print + domestic
// abuse protection notice + lead paint + security deposit + owner
// identity + environmental hazard + utility billing. NJ Truth-in-
// Renting Act (N.J. Stat. § 46:8-43 to § 46:8-50) — DCA booklet
// distribution at lease signing + 30 days for existing tenants;
// $100 penalty per non-distribution. DC Tenant Bill of Rights
// (D.C. Code § 42-3502.22b) — OTA-published Bill of Rights at
// lease signing. California Civ Code § 1962 (15-day owner identity
// disclosure) + § 1962.5 / § 1962.7 (security deposit location).
// Florida Stat § 83.49 (30-day deposit location disclosure).
// Nineteen-mode severity ladder × six jurisdictions × three
// Michigan font states × two NJ distribution states × two DC
// states. Trader-landlord critical because cross-state portfolios
// must apply at least four different handout regimes; defective
// handouts can void monetary lease provisions and trigger
// statutory penalties.
// ---------------------------------------------------------------------------

async fn rental_tenant_bill_of_rights_handout_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalTenantBillOfRightsHandoutInput>,
) -> Result<Json<RentalTenantBillOfRightsHandoutResult>, ApiError> {
    Ok(Json(check_rental_tenant_bill_of_rights_handout(&b)))
}

// ── /rental-mold-disclosure-remediation (iter 545) ──────────────────────────
// POST endpoint for mold disclosure + remediation compliance across eight
// jurisdictions. Indoor mold poses serious health hazards (asthma triggers,
// allergic reactions, respiratory infection) subject to state-by-state
// disclosure and remediation requirements. Failure to disclose known mold
// or promptly remediate exposes the landlord to actual damages, statutory
// civil penalty, lease rescission + constructive eviction, and
// personal-injury tort liability with toxic-mold settlements ranging
// $50K-$5M+.
//
// CA Health & Safety Code §§ 26100-26156 (Toxic Mold Protection Act of
// 2001) + CA Civ. Code § 1102.17 + § 1941.7 (CDPH mold booklet effective
// Jan 1, 2022): written disclosure required if landlord knows or has
// reasonable cause to believe mold present.
//
// NYC Local Law 55 of 2018 (Asthma-Free Housing Act) codified at NYC HMC
// § 27-2017.1: multi-dwelling property owners must investigate AND remove
// indoor mold; licensed-professional remediation required for mold > 10
// square feet; annual inspections; tenant informational materials; HPD
// class-C immediately-hazardous violation framework.
//
// WA RCW 59.18.060(13): landlord must notify tenants of health hazards
// + provide WA Dept of Health mold-prevention information at lease
// execution.
//
// TX Prop. Code § 92.052 diligent-effort-to-repair duty + Texas Deceptive
// Trade Practices Act (DTPA) requires disclosure of known mold
// infestation.
//
// VA Code § 55.1-1215 (lease-execution disclosure) + § 55.1-1226
// (itemized inspection report within 5 days of move-in) + § 8.01-226.12
// (visible-mold duty) + § 55.1-1220(A)(5) (moisture-prevention duty).
//
// FL Stat. § 83.51 landlord duty to maintain — no explicit mold statute.
// IL no statewide mold statute; common-law implied warranty of
// habitability + Jack Spring v. Little doctrine.
//
// Eight-mode severity ladder: NotApplicable,
// NoKnowledgeOfMoldNoDisclosureObligation,
// CompliantWrittenDisclosureAndRemediation,
// DisclosureProvidedRemediationOngoing,
// NoDisclosureViolatesStateMoldStatute,
// NycLocalLaw55LicensedProfessionalRequiredViolation,
// UnremediatedMoldImpliedWarrantyHabitabilityBreach,
// PersonalInjuryToxicTortExposureRisk.

async fn rental_mold_disclosure_remediation_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalMoldDisclosureRemediationInput>,
) -> Result<Json<RentalMoldDisclosureRemediationResult>, ApiError> {
    Ok(Json(check_rental_mold_disclosure_remediation(&b)))
}

// ---------------------------------------------------------------------------
// rental_minneapolis_renter_protections_ordinance_2020: Minneapolis
// Renter Protections Ordinance — unanimously passed by Minneapolis
// City Council on September 13, 2019; effective June 1, 2020 for
// large landlords (more than 15 rental homes) and December 1, 2020
// for small landlords (15 or fewer rental homes); administered by
// Minneapolis Department of Regulatory Services Inspections
// Division. Security deposit cap = ONE MONTH'S RENT. Criminal
// screening look-back limits: 3 years (misdemeanors), 7 years
// (felonies), 10 years (serious offenses: first-degree arson,
// assault, manslaughter, kidnapping, criminal sexual conduct,
// murder, aggravated robbery). Eviction record look-back limits:
// 3 years (judgments), 1 year (settlements), NEVER for dismissed
// evictions (absolute prohibition). Two screening options under
// § 244.2025: (1) standard criteria + individualized assessment;
// (2) inclusionary screening (pre-approved City criteria). Source-
// of-income (public assistance) protections: must accept Section 8
// HCV, MNsure, RAP, HOPWA, Section 202. Energy cost disclosure
// requirement at application (prior 12 months). Sixteen-mode
// severity ladder × 2 property jurisdictions × 2 landlord sizes ×
// 6 compliance aspects × 7 criminal offense classifications × 6
// eviction record classifications × 3 public assistance statuses ×
// variable monthly rent / security deposit / disclosure inputs.
// Sibling cluster: rental_san_francisco_rent_ordinance_chapter_37
// (iter 677 — SF), rental_berkeley_rent_stabilization_ordinance_
// bmc_chapter_13_76 (iter 679 — Berkeley), rental_seattle_smc_22_
// 206_160_just_cause_eviction (iter 669 — Seattle), rental_oakland_
// measure_ee_just_cause_omc_8_22 (iter 681 — Oakland), rental_
// california_sb_567_no_fault_eviction_amendments (iter 673 — CA
// SB 567), rental_eviction_record_sealing_screening (parallel
// eviction screening regime), rental_application_denial_disclosure
// (FCRA-adjacent screening regime), rental_source_of_income_
// discrimination (multi-state public-assistance regime), rental_
// hud_section_504_rehabilitation_act_24_cfr_part_8 (iter 683 —
// federal disability nondiscrimination cross-reference).
// ---------------------------------------------------------------------------

async fn rental_minneapolis_renter_protections_ordinance_2020_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalMinneapolisRenterProtectionsOrdinance2020Input>,
) -> Result<Json<RentalMinneapolisRenterProtectionsOrdinance2020Result>, ApiError> {
    Ok(Json(check_rental_minneapolis_renter_protections_ordinance_2020(&b)))
}

// ---------------------------------------------------------------------------
// rental_multilingual_lease_translation: CA Civ. Code § 1632
// multilingual lease translation compliance. Translation required
// BEFORE execution in the negotiated language when landlord
// negotiates primarily in Spanish, Chinese, Tagalog, Vietnamese, or
// Korean. Residential leases > 1 month coverage per § 1632(b)(1)(A).
// Commercial coverage added by SB 1103 (eff. Jan 1, 2025) for
// qualified commercial tenants (microenterprise + restaurant with
// < 10 employees + nonprofit with < 20 employees). Own-interpreter
// exemption per § 1632(h): non-minor + fluent in both English and
// negotiated language + not employed by or made available through
// landlord. Non-compliance remedy: tenant may RESCIND under
// § 1632(k). Sibling: rental_application_denial_disclosure (state-
// specific written-denial regimes), adverse_action_notice (FCRA
// adverse action), rental_source_of_income_discrimination (related
// fair-housing regime).
// ---------------------------------------------------------------------------

async fn rental_multilingual_lease_translation_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalMultilingualLeaseTranslationInput>,
) -> Result<Json<RentalMultilingualLeaseTranslationResult>, ApiError> {
    Ok(Json(check_rental_multilingual_lease_translation(&b)))
}

// ── /rental-fair-housing-reasonable-accommodation (iter 547) ────────────────
// POST endpoint for FHA reasonable-accommodation / reasonable-modification
// compliance across six jurisdictions. Fair Housing Act 42 U.S.C.
// § 3604(f)(3)(B) makes it unlawful to refuse to make reasonable
// accommodations in rules, policies, practices, or services when such
// accommodations are necessary to afford a person with a disability equal
// opportunity to use and enjoy a dwelling. § 3604(f)(3)(A) parallel
// reasonable-modification provision (tenant expense, no restoration for
// normal wear). HUD/DOJ Joint Statement on Reasonable Accommodations
// (May 17, 2004) + Joint Statement on Reasonable Modifications (March 5,
// 2008) establish the federal framework.
//
// 2026 federal-policy reversal: HUD's May 22, 2026 internal memorandum
// permanently cancelled prior HUD guidance and instructed agency staff
// to stop pursuing complaints involving ESAs that have not been
// individually trained for disability-related work or tasks. State laws
// continue to apply at higher floor.
//
// CA FEHA (Cal. Gov. Code §§ 12927(c)(1) + 12955) — state protection
// survives 2026 federal reversal; CA AB-468 (eff. Jan 1, 2022) requires
// 30-day LMHP relationship before ESA letter.
//
// NY State HRL (NY Exec. Law § 296(5)(c)) + NYC HRL (NYC Admin. Code
// § 8-107(5)) codify interactive-dialogue duty explicitly.
//
// NJ LAD (N.J.S.A. 10:5-12.4) reasonable accommodation duty.
//
// MA Gen. L. ch. 151B § 4(7A) reasonable accommodation duty.
//
// Per se violations: imposing pet fee or deposit on assistance animal;
// requiring restoration of normal-wear modification; outright denial
// without interactive dialogue.
//
// Eight-mode severity ladder: NotApplicable,
// NoClaimOfDisabilityNoAccommodationDuty,
// CompliantAccommodationGranted,
// CompliantInteractiveDialogueAlternativeOffered,
// EsaPost2026FederalReversalStateLawFloorMayStillApply,
// PetFeeOnAssistanceAnimalPerSeFhaViolation,
// RestorationDemandedForNormalWearModificationViolation,
// OutrightDenialFailureToEngageInteractiveDialogue.

async fn rental_fair_housing_reasonable_accommodation_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalFairHousingReasonableAccommodationInput>,
) -> Result<Json<RentalFairHousingReasonableAccommodationResult>, ApiError> {
    Ok(Json(check_rental_fair_housing_reasonable_accommodation(&b)))
}

// ---------------------------------------------------------------------------
// rental_facade_inspection_fisp_local_law_11: NYC Facade Inspection
// Safety Program (FISP) / Local Law 11 of 1998 (originally LL 10 of
// 1980 after Grace Gold facade-collapse fatality May 16, 1979 at 115
// Madison Avenue; LL 38 of 2007 instituted 5-year cycle; current
// regulation at 1 RCNY § 103-04). Buildings > 6 stories (7+) must
// have exterior walls inspected by Qualified Exterior Wall Inspector
// (QEWI — NY State PE/RA with 7+ years experience + separate DOB
// approval) every 5 years. Three classifications: SAFE (clean),
// SWARMP (Safe With a Repair and Maintenance Program — repairs
// required before next cycle), UNSAFE (90-day repair window +
// mandatory sidewalk shed pending repair). Penalties: $1,000/month
// late initial filing, $5,000/year failure to file, $2,000 per
// uncorrected SWARMP condition. Cycle 10 (Feb 21, 2025 - Feb 21,
// 2030) currently active. Enforcement intensified after Erica
// Tishman fatality Dec 17, 2019 at 729 7th Avenue. Sibling cluster:
// rental_soft_story_seismic_retrofit (iter 581 LA/SF wood-frame
// retrofit), rental_emergency_action_plan_high_rise (NYC LL 26
// FDNY EAP), rental_balcony_inspection_seismic_safety (CA SB 721
// EEE inspection), rental_elevator_safety_inspection.
// ---------------------------------------------------------------------------

async fn rental_facade_inspection_fisp_local_law_11_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalFacadeInspectionFispLocalLaw11Input>,
) -> Result<Json<RentalFacadeInspectionFispLocalLaw11Result>, ApiError> {
    Ok(Json(check_rental_facade_inspection_fisp_local_law_11(&b)))
}

// ── /rental-boiler-inspection-compliance (iter 549) ─────────────────────────
// POST endpoint for boiler and gas-piping inspection compliance across
// seven jurisdictions. Boilers and gas-piping in residential rentals are
// subject to periodic inspection requirements under ASME BPVC Section IV
// + Section VI engineering standard plus municipal/state code overlays.
//
// NYC Local Law 152 of 2016 + NYC Admin. Code § 28-318: gas-piping
// systems in all buildings except one- and two-family R-3 occupancies
// must be inspected by Licensed Master Plumber (LMP) every 4 years on
// community-district-based schedule. Inspection report due within 30
// days; GPS1 Certification filing with DOB due within 60 days.
//
// NY State Industrial Code Rule 4: annual high-pressure boiler /
// biennial low-pressure boiler inspection.
//
// CA Cal/OSHA Title 8 Subchapter 1 §§ 750-784: Pressure Vessel Unit
// jurisdiction; annual inspection for boilers above 15 psi steam or
// 160 psi water.
//
// IL 430 ILCS 75 Boiler & Pressure Vessel Safety Act: annual external
// + 6-year internal inspection.
//
// MA Gen. L. ch. 146 § 46: annual external + internal inspection by
// Department of Public Safety, Office of Public Safety and Inspections.
//
// TX Health & Safety Code § 755: boiler inspection program by Texas
// Department of Licensing and Regulation (TDLR).
//
// Seven-mode severity ladder: NotApplicable,
// OneOrTwoFamilyExemptNoInspectionRequired,
// CompliantInspectionAndReportFiled,
// LateFilingWithinThirtyDayCureWindow,
// NycLocalLaw152UnqualifiedInspectorViolation,
// InspectionMissedDeadlineDobCivilPenaltyExposure,
// UnaddressedCorrectionsClassCViolationEscalatedEnforcement.

async fn rental_boiler_inspection_compliance_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalBoilerInspectionComplianceInput>,
) -> Result<Json<RentalBoilerInspectionComplianceResult>, ApiError> {
    Ok(Json(check_rental_boiler_inspection_compliance(&b)))
}

// ── /rental-tenant-rent-escrow-habitability-dispute (iter 551) ──────────────
// POST endpoint for tenant rent escrow + repair-and-deduct compliance
// during habitability disputes across seven jurisdictions. State law
// provides tenants with self-help remedies when landlords fail to maintain
// habitable premises: rent withholding (escrow deposit pending repair),
// repair-and-deduct, and lease termination. Each state has specific
// procedural requirements that govern whether tenant rent withholding is
// lawful or constitutes nonpayment justifying eviction.
//
// CA Civ. Code § 1942 + § 1942.4: repair-and-deduct capped at one month's
// rent, twice per 12-month period; § 1942.4 prohibits rent collection if
// landlord knew of substandard conditions 60+ days.
//
// NY RPL § 235-b (Warranty of Habitability) + RPL § 235-a + HP
// proceedings under RPAPL § 110.
//
// WA RCW 59.18.115 (rent escrow process — written notice + local-authority
// certification + approved escrow account required) + RCW 59.18.100
// (repair-and-deduct cap 2 months rent per repair / per year).
//
// IL Chicago RLTO § 5-12-110: 14-day written notice; withholding cap of
// GREATER of $500 or 50% of monthly rent.
//
// TX Prop. Code § 92.0561: repair-and-deduct cap GREATER of one month
// rent or $500; narrow circumstances only.
//
// MA Gen. L. ch. 239 § 8A: IMMEDIATE withholding upon notice (no cure
// period required); raised as defense or counterclaim in summary process.
//
// Default: common-law implied warranty of habitability + constructive-
// eviction doctrine (Marini v. Ireland 265 A.2d 526 (NJ 1970) + Pugh v.
// Holmes 384 A.2d 1234 (PA 1979)).
//
// Nine-mode severity ladder: NotApplicable,
// CompliantStatutoryRentEscrowOrRepairDeduct,
// TenantRemedyCapExceededLandlordEvictionExposure,
// NoNoticeOrPrematureActionTenantEvictionRisk,
// EscrowAccountNotEstablishedTenantWaivesDefense,
// WashingtonRcw59_18_115EscrowProcessNotFollowed,
// ChicagoRltoWithholdingCapExceededExposure,
// MassachusettsImmediateWithholdingNoCureRequired,
// CommonLawWarrantyImpliedDefense.

async fn rental_tenant_rent_escrow_habitability_dispute_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalTenantRentEscrowHabitabilityDisputeInput>,
) -> Result<Json<RentalTenantRentEscrowHabitabilityDisputeResult>, ApiError> {
    Ok(Json(check_rental_tenant_rent_escrow_habitability_dispute(&b)))
}

// ---------------------------------------------------------------------------
// rental_tree_removal_dangerous_tree_disclosure: multi-state tree
// removal liability + dangerous tree disclosure. Hawaii Rule
// (Whitesell v. Houlton, 63 Haw. 532, 632 P.2d 1077 (1981)) —
// overhanging branches/protruding roots are nuisance when actually
// cause OR imminently endanger sensible harm; neighbor may self-
// help cut OR require owner to pay damages + cut back.
// Massachusetts Rule (Michalson v. Nutting 1931; reaffirmed Ponte
// v. DaSilva 1985) — libertarian; self-help to property line is
// exclusive remedy; no landlord liability for natural processes.
// California Booska v. Patel (24 Cal.App.4th 1786, 30 Cal.Rptr.2d
// 241 (1994)) — self-help NOT absolute; must exercise ORDINARY
// CARE; cannot damage or kill tree. Cal Civ § 833 trunk ownership;
// § 834 boundary tree joint ownership; § 836 nuisance abatement.
// Restatement (Second) of Torts § 363 (natural condition) + § 364
// (artificial condition) + § 840 (encroaching trees) — modern
// majority adopts Hawaii Rule. Landlord premises liability:
// duty of care + foreseeability + warranty of habitability.
// Eleven-mode severity ladder × four jurisdictions × six tree
// scenarios × eight landlord actions. Trader-landlord critical
// because tree-fall liability + dangerous-tree premises liability
// compounds at scale; single diseased oak on 50-unit property
// creates catastrophic third-party injury exposure.
// ---------------------------------------------------------------------------

async fn rental_tree_removal_dangerous_tree_disclosure_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalTreeRemovalDangerousTreeDisclosureInput>,
) -> Result<Json<RentalTreeRemovalDangerousTreeDisclosureResult>, ApiError> {
    Ok(Json(check_rental_tree_removal_dangerous_tree_disclosure(&b)))
}

// ---------------------------------------------------------------------------
// rental_tenant_right_to_counsel_eviction: multi-jurisdictional tenant
// Right to Counsel in eviction proceedings. NYC (Local Law 136 of
// 2017, NYC Admin Code § 26-1301 et seq.) was first U.S. jurisdiction;
// ≤ 200 % FPL threshold. Newark 2018, SF Prop F 2018 (UNIVERSAL no
// income test), Cleveland 2019 (≤ 100 % FPL with minor child),
// Philadelphia 2019, Boulder 2020, KC MO 2022. Washington RCW
// 59.18.640 (Senate Bill 5160 of 2021) was FIRST STATEWIDE RTC; ≤
// 200 % FPL or categorically indigent (public assistance). Maryland
// Access to Counsel in Evictions Act 2021 (≤ 50 % AMI). Connecticut
// Public Act 21-34 2021 (≤ 80 % AMI). Nine-mode severity ladder ×
// eleven jurisdictions × four income bands × six representation
// statuses × five proceeding types. Trader-landlord critical because
// RTC programs dramatically increase tenant defense rates and case
// duration; failure to honor RTC procedure (notice + appointment +
// continuance for attorney) can invalidate eviction judgment.
// ---------------------------------------------------------------------------

async fn rental_tenant_right_to_counsel_eviction_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalTenantRightToCounselEvictionInput>,
) -> Result<Json<RentalTenantRightToCounselEvictionResult>, ApiError> {
    Ok(Json(check_rental_tenant_right_to_counsel_eviction(&b)))
}

// ── /rental-ada-accessible-parking-compliance (iter 553) ────────────────────
// POST endpoint for ADA + FHA accessible-parking compliance across four
// regime categories. Multifamily rental properties subject to two distinct
// accessible-parking regimes: (1) ADA Title III for places of public
// accommodation (rental office, leasing center, common amenities) and
// (2) FHA reasonable-modification provision (42 U.S.C. § 3604(f)(3)(A))
// for individualized tenant accommodation requests.
//
// 2010 ADA Standards § 502 + § 208.2 minimum-table: 1 accessible per 25
// spaces (first 100); sliding scale thereafter (2 per 50; 3 per 75; 4
// per 100; 5 per 150; 6 per 200; 7 per 300; 8 per 400; 9 per 500;
// 1 per 50 thereafter; 1 per 100 above 1,000). § 208.2.4 van-accessible
// ratio: at least 1 of every 6 accessible (rounded up). § 502.3 access
// aisle width: 60-inch minimum car / 96-inch van. § 502.7 vertical
// clearance: 98 inches for van-accessible. § 502.6 signage with
// International Symbol of Accessibility.
//
// FHA § 3604(f)(3)(C) covered-multifamily design+construction (post-March-
// 13-1991 first occupancy; 4+ units with elevator OR 4+ without with
// ground-floor coverage): 2% of parking spaces accessible (minimum 1).
//
// FHA § 3604(f)(3)(A) + (B) reasonable-modification + reasonable-
// accommodation duty: individualized tenant requests at landlord
// expense unless undue burden; HUD/DOJ Joint Statement Reasonable
// Modifications March 5, 2008.
//
// Nine-mode severity ladder: NotApplicable,
// NotCoveredByAdaOrFhaNoComplianceRequirement,
// Compliant2010AdaStandardsSection502,
// InsufficientAccessibleSpaceCountAdaViolation,
// VanAccessibleRatioViolationOnePerSixRule,
// AccessAisleWidthOrSignageViolation,
// FhaCoveredMultifamilyTwoPercentRequirementViolation,
// FhaReasonableAccommodationGranted,
// FhaReasonableAccommodationDeniedSection3604F3aViolation.

async fn rental_ada_accessible_parking_compliance_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalAdaAccessibleParkingComplianceInput>,
) -> Result<Json<RentalAdaAccessibleParkingComplianceResult>, ApiError> {
    Ok(Json(check_rental_ada_accessible_parking_compliance(&b)))
}

// ── /rental-smoke-free-cannabis-restriction (iter 555 milestone) ────────────
// POST endpoint for smoke-free housing + cannabis-use restriction
// compliance across eight jurisdictions. HUD 24 C.F.R. Parts 200 + 982
// + 5 final rule (effective Feb 3, 2017) MANDATES smoke-free policy in
// HUD-subsidized public housing. State and municipal laws authorize
// private landlords to prohibit smoking and require advance disclosure
// to incoming tenants. Cannabis carries parallel landlord-control
// framework with state statutes preserving property-owner authority.
//
// CA SB 332 + Cal. Civ. Code § 1947.5 (effective Jan 1, 2012):
// authorizes private landlords to prohibit smoking + lease must contain
// smoke-free provision + pre-existing-tenant grandfathering.
//
// NYC Local Law 147 of 2017 + NYC Admin. Code § 17-505: smoking-policy
// disclosure mandate.
//
// WA RCW 70.160 Clean Indoor Air Act + RLTA disclosure.
//
// IL Smoke Free Illinois Act 410 ILCS 82.
//
// OR Smoke-Free Workplace ORS 433.835-990.
//
// Cannabis-specific landlord rights: CA H&S § 11362.45(h) + NY Cannabis
// Law § 222 + Penal Law § 222.05 + CO Amendment 64 + § 12-43.4 + WA RCW
// 69.50.4014. Federal Controlled Substances Act Schedule I classification
// supports prohibition; FHA does NOT generally require accommodation
// of cannabis use per Forest City Residential Mgmt. v. Beasley + James
// v. City of Costa Mesa.
//
// Eight-mode severity ladder: NotApplicable,
// CompliantSmokeFreePolicyEnforceable,
// HudFederalMandatePreemptsLocalLawSubsidizedHousing,
// PreExistingTenantGrandfatheredPolicyUnenforceable,
// LeaseDisclosureDefectivePolicyUnenforceable,
// NoDisclosureSmokeFreePolicyUnenforceable,
// CannabisEdiblesNotSubjectToSmokeFreePolicy,
// LandlordMayProhibitCannabisOnPremises.

async fn rental_smoke_free_cannabis_restriction_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalSmokeFreeCannabisRestrictionInput>,
) -> Result<Json<RentalSmokeFreeCannabisRestrictionResult>, ApiError> {
    Ok(Json(check_rental_smoke_free_cannabis_restriction(&b)))
}

// ── /rental-rent-control-stabilization (iter 557) ───────────────────────────
// POST endpoint for rent control + rent stabilization compliance across
// eight jurisdictions. Two states (CA + OR) enacted first statewide rent
// caps in 2019; NY + NJ + DC + several MN cities operate strong rent-
// stabilization frameworks.
//
// CA AB 1482 (Cal. Civ. Code § 1947.12 + § 1946.2) Tenant Protection Act
// of 2019 (effective Jan 1, 2020 - Jan 1, 2030): annual rent cap = LESSER
// OF 5% + local CPI OR 10%. Applies to units 15+ years old (rolling
// exception). § 1946.2 just-cause eviction protection.
//
// OR SB 608 (ORS 90.323 + ORS 90.427): 7% + CPI capped at 10% under
// SB 611 effective Jul 6, 2023.
//
// NY State HSTPA 2019 + NYC RSL § 26-501: 6+ unit pre-1974 buildings;
// Rent Guidelines Board annual % cap.
//
// NJ Anti-Eviction Act N.J.S.A. 2A:18-61.1 + municipal rent-control
// ordinances in 100+ NJ municipalities.
//
// DC Rental Housing Act of 1985 D.C. Code § 42-3501: pre-1976 + 5+
// units; CPI + 2% capped at 10% (5% for elderly/disabled).
//
// MN St. Paul Charter Amendment (Nov 2021): 3% annual cap.
//
// Default: most states (TX, FL, TN, AZ, MI, IL outside Chicago)
// PREEMPT local rent control via state statute.
//
// Five-mode severity ladder: NotApplicable,
// NoRentControlInJurisdiction,
// PropertyExemptFromRentCap,
// CompliantRentIncreaseWithinStatutoryCap,
// RentIncreaseExceedsStatutoryCapViolation.

async fn rental_rent_control_stabilization_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalRentControlStabilizationInput>,
) -> Result<Json<RentalRentControlStabilizationResult>, ApiError> {
    Ok(Json(check_rental_rent_control_stabilization(&b)))
}

// ---------------------------------------------------------------------------
// rental_rent_increase_notice_requirement: multi-state residential
// rent-increase notice timing compliance. CA Civ Code § 827 (AB 1110
// of 2019, effective Jan 1, 2020): 30 days for ≤ 10 % increase; 90
// days for > 10 % increase. WA RCW 59.18.140 (amended effective May
// 7, 2025): 60 days written notice before increase effective date.
// OR ORS 90.323 / 90.600 (SB 608 of 2019): 90 days written notice
// for any rent increase. NY RPL § 226-c (HSTPA 2019): notice
// required when increase ≥ 5 % OR landlord declines renewal;
// tiered notice by tenancy length — 30 days (< 1 year), 60 days
// (1-2 years), 90 days (≥ 2 years). Failure consequence: tenant's
// lawful tenancy continues under existing terms until notice
// period expires. Thirteen-mode severity ladder × five
// jurisdictions × three lease length tiers × multiple notice
// outcomes. Trader-landlord critical: most cross-state operations
// must apply 4 different rules; defective notice voids the
// increase and continues tenancy at prior rate.
// ---------------------------------------------------------------------------

async fn rental_rent_increase_notice_requirement_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalRentIncreaseNoticeRequirementInput>,
) -> Result<Json<RentalRentIncreaseNoticeRequirementResult>, ApiError> {
    Ok(Json(check_rental_rent_increase_notice_requirement(&b)))
}

// ---------------------------------------------------------------------------
// rental_rent_to_own_lease_purchase_disclosures: multi-state
// residential rent-to-own / lease-purchase executory contract
// disclosure regimes. Texas Property Code Subchapter D
// §§ 5.061-5.085 — contract for deed, lease option, or purchase
// option > 180 days = executory contract; § 5.069 requires extensive
// disclosures (survey + liens + covenants + easements + statutory
// disclosure + non-subdivision notice + tax certificates + insurance
// + 7-day notice letter + annual accounting); § 5.072 14-day right
// of rescission; § 5.074 14-day unilateral cancellation; § 5.077
// annual accounting; § 5.079 30-day recording requirement.
// California Civ Code § 2985 installment land contract.
// Maryland Real Property Code § 10-101 executory contract.
// Illinois 765 ILCS 71/ Residential Real Property Lease-Purchase
// Act effective Jan 1, 2025. SAFE Act (12 U.S.C. § 5101 et seq.) +
// T-SAFE — RMLO license required for non-homestead non-family
// owner finance. Dodd-Frank §§ 1402-1403 owner-financing exception:
// 3 or fewer properties per year + fixed-rate ≥ 5 years + no
// negative amortization. CFPB Reg Z treats lease-purchase as
// consumer credit. Sixteen-mode severity ladder × five
// jurisdictions × five contract types × four seller types. Trader-
// landlord critical for owner-finance / rent-to-own portfolio
// operators; missing 7-day notice or annual accounting voids
// contract.
// ---------------------------------------------------------------------------

async fn rental_rent_to_own_lease_purchase_disclosures_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalRentToOwnLeasePurchaseDisclosuresInput>,
) -> Result<Json<RentalRentToOwnLeasePurchaseDisclosuresResult>, ApiError> {
    Ok(Json(check_rental_rent_to_own_lease_purchase_disclosures(&b)))
}

// ── /rental-tenant-relocation-assistance (iter 559) ─────────────────────────
// POST endpoint for tenant relocation assistance compliance across six
// jurisdictions during no-fault eviction + condo conversion + demolition
// + substantial rehabilitation displacement.
//
// CA AB 1482 (Cal. Civ. Code § 1946.2 + § 1947.12) + SB 567 (effective
// April 1, 2024): one month's rent for no-fault eviction; SB 567
// owner-move-in 90-day + 1-year residency + notice disclosure + no
// other vacant similar unit.
//
// NYC RENT-STABILIZED DEMOLITION (NYC RSL § 26-511(c)(9) + 9 NYCRR
// § 2524.5): NY DHCR approval required + comparable replacement
// housing + reasonable moving expenses + $5,000 stipend.
//
// WA RCW 59.18.440 + RCW 59.18.450: low-income (≤ 50% AMI) tenant
// relocation assistance for demolition / substantial rehabilitation /
// change of use / removal of use restrictions.
//
// IL Chicago RLTO § 5-12-130 + § 5-14-050: condo conversion = GREATER
// OF $1,500 OR one month rent capped at $2,500. Keep Chicago Renting
// Ordinance (KCRO) = $10,600 post-foreclosure if no lease-renewal
// offer.
//
// DC TOPA (D.C. Code § 42-3404.01 et seq.): right of first refusal +
// relocation assistance capped at lesser of one year rent or $12,000.
//
// Eight-mode severity ladder: NotApplicable,
// AtFaultEvictionNoRelocationDuty,
// CompliantRelocationAssistancePaid,
// InsufficientRelocationAssistanceViolation,
// SbFiveSixSevenOwnerMoveInNonCompliance,
// NycDhcrApprovalRequiredDemolitionViolation,
// WashingtonNonLowIncomeNotCovered,
// DcTopaRightOfFirstRefusalIncludingRelocation.

async fn rental_tenant_relocation_assistance_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalTenantRelocationAssistanceInput>,
) -> Result<Json<RentalTenantRelocationAssistanceResult>, ApiError> {
    Ok(Json(check_rental_tenant_relocation_assistance(&b)))
}

// ── /rental-tenant-data-privacy-compliance (iter 561) ───────────────────────
// POST endpoint for tenant data privacy compliance across eight
// jurisdictions. Landlords collect substantial tenant personal data
// (application screening + lease execution + smart-lock access logs +
// IoT-thermostat usage + biometric package-locker + surveillance
// footage + rent-payment history). State and local data-privacy
// regimes impose disclosure + consent + security + right-to-delete
// obligations. Federal FCRA layers consumer-report-specific duties.
//
// FCRA 15 U.S.C. §§ 1681-1681x: written consent + adverse-action
// notice + § 1681n willful $100-$1,000 per violation + § 1681o
// negligent actual damages.
//
// CA CCPA + CPRA (Cal. Civ. Code §§ 1798.100-1798.199.100): notice
// + access/delete/correct rights + opt-out + sensitive-info limit;
// $2,500 / $7,500 civil penalty.
//
// IL BIPA (740 ILCS 14): written consent + retention policy required
// for biometric collection; $1,000 negligent / $5,000 intentional
// per record + Rosenbach v. Six Flags 432 Ill. Dec. 654 (Ill. 2019)
// no-actual-injury standing.
//
// NYC Tenant Data Privacy Act (NYC Admin. Code §§ 26-3001-3007):
// smart-access buildings + notice + consent + security + deletion.
// NY SHIELD Act Gen. Bus. Law § 899-aa + § 899-bb statewide
// reasonable-security overlay.
//
// VA CDPA (Va. Code §§ 59.1-575 to 59.1-585) biometric sensitive-data
// opt-in + DPA. TX CUBI (Bus. & Com. Code §§ 521.052 + 503.001). CO
// CPA (C.R.S. §§ 6-1-1301 to 6-1-1313).
//
// Nine-mode severity ladder: NotApplicable,
// CompliantConsentAndNoticeFramework,
// FcraConsumerReportConsentObtainedAdverseActionDuty,
// FcraAdverseActionNoticeMissingViolation,
// BipaBiometricCollectionWithoutWrittenConsentViolation,
// CcpaCpraConsumerRightsNotProvidedViolation,
// NycTenantDataPrivacyActViolation,
// NySchieldActDataSecurityNonCompliance,
// DefaultJurisdictionFcraAndCommonLawOnly.

async fn rental_tenant_data_privacy_compliance_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalTenantDataPrivacyComplianceInput>,
) -> Result<Json<RentalTenantDataPrivacyComplianceResult>, ApiError> {
    Ok(Json(check_rental_tenant_data_privacy_compliance(&b)))
}

// ---------------------------------------------------------------------------
// rental_tenant_estoppel_certificate: residential tenant estoppel
// certificate compliance. Tenant required to sign ONLY when lease
// contains estoppel provision; absent provision, tenant not
// obligated. Refusal to sign when lease requires = breach of lease.
// Typical 10-day response window per standard lease clause.
// California Evidence Code § 622: facts recited in written
// instrument conclusively presumed true between parties or
// successors in interest — statements binding and tenant cannot
// later contradict. Common transactional triggers: sale,
// refinancing, line-of-credit, mortgage modification. Required
// content: rent + lease terms + protected tenancy status + oral
// agreements + amendments + landlord promises + utilities.
// Eleven-mode severity ladder × five transactional triggers × four
// lease clause types × four tenant responses × seven content
// defect categories. Trader-landlord critical because defective
// estoppel demands and false statements create transaction-failure
// and litigation exposure during refinancing and sale.
// ---------------------------------------------------------------------------

async fn rental_tenant_estoppel_certificate_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalTenantEstoppelCertificateInput>,
) -> Result<Json<RentalTenantEstoppelCertificateResult>, ApiError> {
    Ok(Json(check_rental_tenant_estoppel_certificate(&b)))
}

// ── /rental-ev-charging-accommodation (iter 563) ────────────────────────────
// POST endpoint for EV charging accommodation compliance across seven
// jurisdictions. Most states with significant EV adoption enacted
// right-to-charge statutes prohibiting landlords from unreasonably
// restricting tenant installation of EV charging stations.
//
// CA Cal. Civ. Code § 1947.6 (AB 2565 + AB 2863): right-to-charge
// applies to leases executed/renewed/extended after July 1, 2015;
// tenant pays installation + electrical + reasonable conditions.
//
// IL 765 ILCS 1085 Electric Vehicle Charging Act: effective Jan 1,
// 2024 + new-construction EV-capable parking-space mandate.
//
// CO C.R.S. § 38-12-601: Level 1/2 at tenant expense; no landlord
// fee except actual electricity or reasonable access; safety + 30-day
// registration + reasonable aesthetic permitted.
//
// HI Haw. Rev. Stat. § 196-7.5: lease provisions prohibiting EV
// charging at multi-family residential dwelling or townhouse parking
// stalls VOID and unenforceable.
//
// NY RPL § 234 + state energy law: HOA / condo association
// restrictions on EV charging in assigned parking prohibited.
//
// FL Fla. Stat. § 718.113(8): applies to CONDOMINIUM OWNERS only
// (NOT tenants of rentals).
//
// Federal § 30C alternative fuel vehicle refueling property credit:
// up to 30% of cost (max $1,000 residential / $30,000 commercial);
// IRA 2022 (Pub. L. 117-169) extended through 2032.
//
// Eight-mode severity ladder: NotApplicable,
// CompliantLandlordApprovalWithReasonableConditions,
// OutrightDenialViolatesRightToChargeStatute,
// UnreasonableConditionsViolation,
// NoTimelyResponseViolatesStatutoryWindow,
// TenantRefusedBonaFideSafetyOrCostConditionsDeniedReasonably,
// FloridaCondoOnlyRentalNotProtected,
// DefaultJurisdictionNoRightToChargeStatute.

async fn rental_ev_charging_accommodation_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalEvChargingAccommodationInput>,
) -> Result<Json<RentalEvChargingAccommodationResult>, ApiError> {
    Ok(Json(check_rental_ev_charging_accommodation(&b)))
}

// ── /rental-waste-recycling-collection-mandate (iter 565 milestone) ─────────
// POST endpoint for waste collection + recycling + organic-composting
// compliance across nine jurisdictions. Multifamily landlords face
// increasingly comprehensive waste-diversion mandates requiring
// separate streams for trash, recycling, and organic/compostable
// waste.
//
// CA SB 1383 (Cal. Pub. Res. Code §§ 42652-42653): collection effective
// Jan 1, 2022; penalties effective Jan 1, 2024. Multifamily 3+ units
// (5+ in some cities). 75% diversion goal by Jan 1, 2025. CA AB 939
// (1989) foundational 50%-diversion framework. Tenant education at
// move-in + 2-weeks-pre-move-out + annual.
//
// NYC LL 87/2009 + NYC RCNY § 16 + Mandatory Curbside Composting Law
// (staged rollout). NYC LL 142/2013 textile recycling. NYC LL 199/2017
// commercial waste zones.
//
// VT Universal Recycling Act 2012 (Act 148): bans recyclables + yard
// debris + food scraps from landfill. Multifamily food scrap diversion
// since Jul 1, 2020.
//
// MA Mass. Gen. Laws ch. 21A + 310 CMR 16.00: organic-waste ban
// effective Oct 1, 2014; threshold lowered Nov 1, 2022 to 0.5 ton/week.
//
// WA HB 1799 (2023) organic-waste-diversion. OR HB 2065. CO HB 22-1355
// Producer Responsibility Program. NJ A4416 Food Waste Recycling Act
// + N.J.A.C. 7:26-2A.13.
//
// Eight-mode severity ladder: NotApplicable,
// BelowJurisdictionalThresholdNoMandate,
// CompliantAllStreamsAndEducationProvided,
// OrganicWasteBinNotProvidedSb1383Violation,
// RecyclingBinNotProvidedViolation,
// TenantEducationProtocolNotFollowed,
// SignageOrContaminationMonitoringMissing,
// DefaultJurisdictionStandardMunicipalCollectionOnly.

async fn rental_waste_recycling_collection_mandate_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalWasteRecyclingCollectionMandateInput>,
) -> Result<Json<RentalWasteRecyclingCollectionMandateResult>, ApiError> {
    Ok(Json(check_rental_waste_recycling_collection_mandate(&b)))
}

// ---------------------------------------------------------------------------
// rental_washington_hb_1217_rent_stabilization: Washington HB 1217 of
// 2025 statewide rent stabilization. First statewide rent cap law in
// Washington history; signed by Governor Bob Ferguson on May 7, 2025
// and effective immediately. Amends RCW 59.18 Residential Landlord-
// Tenant Act and RCW 59.20 Manufactured/Mobile Home Landlord-Tenant
// Act. General residential cap: 7 % + CPI OR 10 %, whichever LESS in
// any 12-month period after first year of tenancy. Manufactured/
// mobile home park space cap: 5 % maximum annual percentage. First-
// year tenancy: NO rent increase permitted during first 12 months of
// tenancy regardless of lease type (month-to-month or fixed term).
// Notice: minimum 90-day written notice (increased from prior 60-day
// RCW 59.18.140 requirement). New construction exemption: 12 years
// from first certificate of occupancy. Sunset: most provisions
// expire July 1, 2040 (15-year sunset). Single-family home exemption
// NOT in final law — Senate version proposed exemption for non-
// corporate single-family homes plus 10 % + CPI cap; both provisions
// stripped in conference committee April 27, 2025. WA Attorney
// General independent enforcement authority plus tenant private
// civil action with actual + statutory damages + attorney fees for
// excessive increases. Thirteen-mode severity ladder × four property
// classifications × four tenancy statuses × four notice categories.
// Trader-landlord critical because Washington is the SECOND state
// after Oregon (SB 608 of 2019) to enact statewide rent stabilization
// — first West Coast state with new-construction-exempt rolling 12-
// year cap structure. Sibling cluster: rental_rent_control_
// stabilization (multi-state regime; WA HB 1217 = newest entry),
// rental_rent_increase_notice_requirement (90-day RCW 59.18.140
// notice cross-reference), rental_just_cause_eviction (parallel
// tenant protection overlay), rental_pre_foreclosure_tenant_
// notification (WA RCW 61.24.143/.146 cross-reference; iter 627),
// rental_mobile_home_park (RCW 59.20 cross-reference).
// ---------------------------------------------------------------------------

async fn rental_washington_hb_1217_rent_stabilization_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalWashingtonHb1217RentStabilizationInput>,
) -> Result<Json<RentalWashingtonHb1217RentStabilizationResult>, ApiError> {
    Ok(Json(check_rental_washington_hb_1217_rent_stabilization(&b)))
}

// ── /rental-dog-bite-liability (iter 567) ───────────────────────────────────
// POST endpoint for landlord dog-bite liability compliance across eight
// jurisdictions. Dog-bite liability for landlords (vs dog-owner tenants)
// turns on NEGLIGENCE standard: landlord liable when (1) actual or
// constructive knowledge of dangerous propensities + (2) legal ability
// to abate + (3) failure to take reasonable steps. Strict-liability dog-
// bite statutes apply to DOG OWNER, not landlord.
//
// CA Cal. Civ. Code § 3342 dog-owner strict liability + Uccello v.
// Laudenslayer (1975) 44 Cal. App. 3d 504 landlord knowledge-plus-
// ability-to-act test + Code Civ. Proc. § 335.1 2-year SOL.
//
// NY Agriculture & Markets Law § 121 + § 123 dangerous-dog statute +
// Bard v. Jahnke (2006) 6 N.Y.3d 592 vicious-propensities standard.
//
// IL 510 ILCS 5/16 Animal Control Act dog-owner strict liability +
// landlord premises-liability framework.
//
// WA RCW 16.08.040 strictest dog-owner strict liability regardless of
// viciousness or knowledge + landlord on common-law negligence.
//
// TX one-bite rule + Marshall v. Ranne (1974) 511 S.W.2d 255 vicious-
// propensity test.
//
// FL Fla. Stat. § 767.04 dog-owner strict liability with comparative-
// negligence reduction + § 767.13(2) dangerous-dog classification.
//
// OH ORC § 955.28 dog-owner strict liability for property/personal
// injury.
//
// Six-mode severity ladder: NotApplicable,
// NoLandlordLiabilityNoKnowledgeOrNoControl,
// LandlordLiableKnowledgeFailedToAbateCommonArea,
// LandlordLiableKnowledgeFailedToAbateTenantPremises,
// LandlordTookReasonableAbatementNoLiability,
// DogOwnerStrictLiabilityNoLandlordExposure.

// ---------------------------------------------------------------------------
// rental_dc_topa_tenant_opportunity_purchase: D.C. Tenant Opportunity
// to Purchase Act compliance for trader-landlords selling residential
// rental property in Washington, D.C. D.C. Code § 42-3401.01 et seq.
// (Rental Housing Conversion and Sale Act of 1980 — D.C. Law 3-86)
// with operative tenant-purchase provisions at § 42-3404.02 et seq.
// and civil-action provisions at § 42-3405.03. Three coverage tiers:
// (1) Single-family homes exempt as of July 3, 2018 (TOPA Single-
// Family Home Exemption Amendment Act of 2017); (2) 2-4 unit buildings
// covered IF owned by corporate business or multi-property owner —
// 90-day negotiation period; (3) 5+ unit buildings — full TOPA with
// 45-day cooling-off (or 30-day Statement of Interest if association
// pre-existing), 120-day negotiation, 120-240 day financing window,
// DHCD tenant-organization registration + certified training required.
// Civil cause of action with attorney fees + costs to prevailing
// party. RENTAL Act of 2025 (passed Sep 17, 2025; eff. Dec 31, 2025)
// shortens windows and clarifies exemptions but preserves core right-
// of-first-refusal regime.
// ---------------------------------------------------------------------------

async fn rental_dc_topa_tenant_opportunity_purchase_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalDcTopaTenantOpportunityPurchaseInput>,
) -> Result<Json<RentalDcTopaTenantOpportunityPurchaseResult>, ApiError> {
    Ok(Json(check_rental_dc_topa_tenant_opportunity_purchase(&b)))
}

async fn rental_dog_bite_liability_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalDogBiteLiabilityInput>,
) -> Result<Json<RentalDogBiteLiabilityResult>, ApiError> {
    Ok(Json(check_rental_dog_bite_liability(&b)))
}

// ---------------------------------------------------------------------------
// rental_drone_overflight_surveillance_privacy: multi-state drone
// overflight + aerial imagery + surveillance privacy compliance.
// CA Civ Code § 1708.8 (rewritten by AB 856 of 2015; anti-paparazzi
// drone law) — physical invasion of privacy when airspace entered
// without permission to capture image of private/personal/familial
// activity; $5,000-$50,000 statutory fine + treble + disgorgement.
// Texas Government Code Chapter 423 (Privacy of Captured Images
// Act of 2013) — offense to capture image with intent to surveil
// without owner consent; $5,000/episode + $10,000/disclosed; 5th
// Cir. upheld 2023; illegally obtained images inadmissible.
// Florida Stat § 934.50 (Freedom from Unwarranted Surveillance
// Act; 2013 enactment + 2015 amendment) — drone with imaging
// device may NOT record privately owned property OR owner/tenant/
// occupant/invitee/licensee with intent to surveil in violation
// of reasonable expectation of privacy WITHOUT WRITTEN CONSENT;
// presumption of reasonable expectation when not observable from
// ground level. FAA Part 107 — Remote Pilot Certificate required
// for commercial flight; daylight + visual line-of-sight; max
// altitude 400 feet AGL. Twelve-mode severity ladder × four
// jurisdictions × six flight purposes × five consent statuses.
// Trader-landlord critical because drone-based facade inspection
// + parking-lot security + tenant surveillance is increasingly
// common but per-image civil penalties up to $50,000 (CA) and
// treble damages create substantial liability exposure.
// ---------------------------------------------------------------------------

async fn rental_drone_overflight_surveillance_privacy_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalDroneOverflightSurveillancePrivacyInput>,
) -> Result<Json<RentalDroneOverflightSurveillancePrivacyResult>, ApiError> {
    Ok(Json(check_rental_drone_overflight_surveillance_privacy(&b)))
}

// ---------------------------------------------------------------------------
// rental_domestic_violence_lock_change_lease_termination: multi-
// jurisdictional DV lock-change + early lease termination
// compliance. CA Civ Code § 1941.5 (24-hour lock change when
// restrained person NOT tenant; tenant pays cost); § 1941.6
// (24-hour lock change when restrained person IS tenant; LANDLORD
// pays cost; tenant may self-change if landlord fails 24-hour
// window). CA Civ Code § 1946.7 (early termination; 14-day post-
// notice rent obligation). Texas Property Code § 92.016 (lock
// change within 3 business days; landlord charges actual cost;
// 30-day notice early termination; required statutory rights lease
// language or tenant NOT liable for unpaid rent; zero notice if
// abuser is co-tenant). NY RPL § 227-c (≥ 30-day termination
// notice + ≤ 25-day proof; 30-day notice period rent). Arizona Rev
// Stat § 33-1318. VAWA federally subsidized housing (Section 8,
// public housing, LIHTC, HOME): 1994 enactment + 2013/2022
// Reauthorization — bars DV-related eviction + permits lease
// bifurcation + emergency transfer plans. Sixteen-mode severity
// ladder × six jurisdictions × three scenarios × four documentation
// types. Trader-landlord critical because DV-related lock-change
// failures expose landlords to statutory damages, treble damages
// (AZ), and tenant safety claims; § 1941.6 violation can trigger
// punitive damages in California.
// ---------------------------------------------------------------------------

async fn rental_domestic_violence_lock_change_lease_termination_route(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<RentalDomesticViolenceLockChangeLeaseTerminationInput>,
) -> Result<Json<RentalDomesticViolenceLockChangeLeaseTerminationResult>, ApiError> {
    Ok(Json(check_rental_domestic_violence_lock_change_lease_termination(&b)))
}
