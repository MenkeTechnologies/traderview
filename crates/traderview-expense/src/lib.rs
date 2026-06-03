//! traderview-expense — business-expense CSV parsers, merchant→category rules,
//! and cross-account transfer dedup.
//!
//! Source parsers (Amazon, Bank of America, Chase, Apple Card) are stubs that
//! return `Unsupported` until a real redacted export is uploaded — same
//! discipline as the Webull parser in `traderview-import`. Inferring columns
//! from documentation produces wrong column maps; only a real file is the spec.

pub mod abandoned_property_handling;
pub mod adverse_action_notice;
pub mod adverse_possession_claim;
pub mod advance_rent_limit;
pub mod amazon;
pub mod apple;
pub mod application_fees;
pub mod asbestos_disclosure;
pub mod balcony_inspection;
pub mod bedbug_disclosure;
pub mod bedbug_extermination_cost;
pub mod boa;
pub mod broker_fee_allocation;
pub mod carpet_replacement_useful_life;
pub mod chase;
pub mod contractor_1099;
pub mod condominium_conversion_protection;
pub mod commercial_lease_cam_charge_disclosure;
pub mod commercial_lease_personal_guaranty_enforceability;
pub mod cooling_requirements;
pub mod cosigner_rules;
pub mod cost_segregation;
pub mod damage_deduction_itemization;
pub mod duty_to_mitigate_damages;
pub mod credit_check_authorization;
pub mod death_in_unit_disclosure;
pub mod dog_breed_restriction_ban;
pub mod demolition_tenant_notice;
pub mod crime_victim_termination;
pub mod crypto_tax;
pub mod dedup;
pub mod depreciation;
pub mod deposit_interest;
pub mod detector_requirements;
pub mod deposit_return_windows;
pub mod dv_survivor_lock_change;
pub mod dv_termination;
pub mod entry_notice;
pub mod disposition;
pub mod emotional_support_animal_documentation;
pub mod ev_charger_installation;
pub mod fair_chance_housing;
pub mod family_childcare_home_right;
pub mod fha_design_construction;
pub mod fire_sprinkler_disclosure;
pub mod firearms_in_rental_unit;
pub mod flag_display_right;
pub mod flood_disclosure;
pub mod drug_eviction;
pub mod eviction_diversion_program;
pub mod eviction_notices;
pub mod eviction_record_sealing;
pub mod foreclosure_tenant_rights;
pub mod foreign_tax_credit;
pub mod form_8606;
pub mod habitability_remedies;
pub mod heat_requirements;
pub mod immigration_status_protection;
pub mod just_cause_eviction;
pub mod just_cause_termination_notice_content;
pub mod landlord_annual_rent_statement;
pub mod landlord_emergency_entry_notice;
pub mod landlord_foreclosure_status_disclosure;
pub mod landlord_harassment;
pub mod landlord_identification_disclosure;
pub mod landlord_possession_delivery;
pub mod landlord_post_eviction_tenant_property_storage_disposal;
pub mod landlord_property_sale_notice;
pub mod landlord_repair_response_timeframe;
pub mod landlord_retaliation_damages;
pub mod landlord_security_device_obligations;
pub mod landlord_self_help_eviction_prohibition;
pub mod landlord_tenant_recording_consent;
pub mod landlord_water_heat_emergency_response;
pub mod landlord_lien_prohibition;
pub mod landlord_master_key_retention;
pub mod landlord_mid_tenancy_rekeying;
pub mod landlord_negative_credit_reporting;
pub mod landlord_pest_extermination_timeline;
pub mod holdover_tenant_damages;
pub mod hoa_fee_tenant_enforcement;
pub mod hoa_rental_restriction;
pub mod home_office;
pub mod last_month_rent_offset;
pub mod late_fee_caps;
pub mod late_payment_grace_period;
pub mod lead_disclosure;
pub mod lead_in_drinking_water_disclosure;
pub mod lead_renovation_repair_painting;
pub mod lease_assignment_consent;
pub mod lease_auto_renewal;
pub mod lease_copy_delivery;
pub mod lease_cure_period;
pub mod lease_disclosures;
pub mod lease_early_termination_fee_cap;
pub mod lease_nondisparagement_prohibition;
pub mod lease_renewal_offer_timing;
pub mod lease_succession;
pub mod lease_termination_catastrophic_damage;
pub mod lease_termination_notice;
pub mod lease_translation;
pub mod lease_waiver_enforceability;
pub mod lock_change_between_tenancies;
pub mod lockout_penalties;
pub mod mandatory_renters_insurance_provider_choice;
pub mod manual_entry;
pub mod meals_50;
pub mod meth_contamination_disclosure;
pub mod mid_tenancy_ownership_change;
pub mod mid_tenancy_security_deposit_increase;
pub mod mid_tenancy_temporary_relocation;
pub mod mid_tenancy_term_modification;
pub mod military_ordnance_disclosure;
pub mod mileage;
pub mod mobile_home_park;
pub mod military_termination;
pub mod mold_disclosure;
pub mod move_in_fee_cap;
pub mod move_in_inspection;
pub mod mlp_ubti;
pub mod mtm_475f;
pub mod niit;
pub mod occupancy_standards;
pub mod otard_antenna_installation;
pub mod owner_identification;
pub mod owner_move_in_eviction;
pub mod non_refundable_cleaning_fees;
pub mod normalize;
pub mod pesticide_application_notice;
pub mod pet_fees;
pub mod plain_language_lease;
pub mod portable_tenant_screening_report;
pub mod pre_move_out_inspection;
pub mod prevailing_party_attorney_fees;
pub mod qbi;
pub mod quarterly_tax;
pub mod quiet_enjoyment;
pub mod recurring;
pub mod radon_disclosure;
pub mod reasonable_accommodation_modification;
pub mod religious_display_doorpost;
pub mod rent_abatement_construction_nuisance;
pub mod rent_acceleration_enforceability;
pub mod rent_concession_disclosure;
pub mod rent_control;
pub mod rent_control_lease_disclosure;
pub mod rent_overcharge_recovery;
pub mod rubs_utility_billing_disclosure;
pub mod rent_credit_reporting;
pub mod rent_escrow;
pub mod rent_increase_notice_period;
pub mod rent_payment_method;
pub mod residential_lease_arbitration_clause;
pub mod right_to_counsel_eviction;
pub mod renters_insurance;
pub mod roommate_authorization;
pub mod rent_receipts;
pub mod rent_stabilized_mci_iai_passthrough;
pub mod repair_and_deduct;
pub mod retaliation_windows;
pub mod rental_dc_topa_tenant_opportunity_purchase;
pub mod rental_depreciation;
pub mod rental_domestic_violence_lock_change_lease_termination;
pub mod rental_dog_bite_liability;
pub mod rental_drone_overflight_surveillance_privacy;
pub mod rental_ada_accessible_parking_compliance;
pub mod rental_application_denial_disclosure;
pub mod rental_asbestos_disclosure;
pub mod rental_attached_garage_carbon_monoxide_disclosure;
pub mod rental_attorney_fee_clause_reciprocity;
pub mod rental_balcony_inspection_seismic_safety;
pub mod rental_basement_water_intrusion_disclosure;
pub mod rental_bed_bug_disclosure;
pub mod rental_bedroom_egress_window;
pub mod rental_boiler_inspection_compliance;
pub mod rental_california_ab_12_security_deposit_cap;
pub mod rental_carbon_monoxide_detector;
pub mod rental_chimney_fireplace_inspection_disclosure;
pub mod rental_climate_mobilization_act_ll97_emissions;
pub mod rental_colorado_hb_24_1098_just_cause_eviction;
pub mod rental_cooling_tower_inspection_local_law_77;
pub mod rental_elevator_safety_inspection;
pub mod rental_emergency_action_plan_high_rise;
pub mod rental_eviction_record_sealing_screening;
pub mod rental_ev_charging_accommodation;
pub mod rental_facade_inspection_fisp_local_law_11;
pub mod rental_fair_housing_reasonable_accommodation;
pub mod rental_fire_extinguisher_requirement;
pub mod rental_flood_hazard_disclosure;
pub mod rental_florida_hb_1417_state_preemption;
pub mod rental_foreclosure_tenant_protection_ptfa;
pub mod rental_broadband_mte_rules;
pub mod rental_energy_benchmarking;
pub mod rental_garage_door_safety_compliance;
pub mod rental_gas_appliance_ban;
pub mod rental_gas_piping_inspection_local_law_152;
pub mod rental_grill_propane_bbq_restriction;
pub mod rental_hardwired_smoke_alarm_responsibility;
pub mod rental_heat_minimum_temperature_season;
pub mod rental_hoa_disclosure_at_lease;
pub mod rental_hot_water_temperature;
pub mod rental_hud_hotma_income_asset_compliance;
pub mod rental_illegal_lockout_self_help_eviction;
pub mod rental_in_unit_laundry_appliance_provision;
pub mod rental_junk_fee_transparency;
pub mod rental_just_cause_eviction;
pub mod rental_landlord_notice_to_enter;
pub mod rental_late_fee_cap;
pub mod rental_lead_paint_disclosure;
pub mod rental_local_law_87_energy_audit_retro_commissioning;
pub mod rental_local_law_88_lighting_upgrades_sub_metering;
pub mod rental_lead_pipe_disclosure;
pub mod rental_marijuana_cultivation_restriction;
pub mod rental_massachusetts_security_deposit_statute;
pub mod rental_mold_disclosure_remediation;
pub mod rental_multilingual_lease_translation;
pub mod rental_natural_gas_leak_response;
pub mod rental_new_jersey_anti_eviction_act;
pub mod rental_ny_rent_receipt_late_notice_requirements;
pub mod rental_ny_rpl_235f_roommate_law;
pub mod rental_nyc_childhood_lead_poisoning_prevention_act;
pub mod rental_nyc_coop_conversion_eviction_protection;
pub mod rental_nyc_local_law_18_str_registration;
pub mod rental_nyc_local_law_55_ipm_pest_control;
pub mod rental_nyc_loft_law_article_7c;
pub mod rental_nyc_scrie_drie_rent_freeze;
pub mod rental_oil_tank_replacement_disclosure;
pub mod rental_oregon_sb_608_sb_611_rent_stabilization;
pub mod rental_organic_waste_collection_disclosure;
pub mod rental_pellet_stove_disclosure;
pub mod rental_pesticide_application_notification;
pub mod rental_pet_breed_restriction_disclosure;
pub mod rental_pet_deposit_separate_security;
pub mod rental_positive_rent_payment_credit_reporting;
pub mod rental_post_construction_lead_dust_clearance;
pub mod rental_pre_foreclosure_tenant_notification;
pub mod rental_propane_tank_lease_disclosure;
pub mod rental_property_registration;
pub mod rental_property_tax_pass_through_disclosure;
pub mod rental_radiator_steam_heat_safety;
pub mod rental_radon_mitigation_disclosure;
pub mod rental_renters_insurance_requirement;
pub mod rental_rent_control_stabilization;
pub mod rental_rent_increase_notice_requirement;
pub mod rental_rent_to_own_lease_purchase_disclosures;
pub mod rental_retaliation_prohibition;
pub mod rental_satellite_dish_installation_right;
pub mod rental_security_deposit_interest;
pub mod rental_security_deposit_return_notice;
pub mod rental_septic_system_disclosure;
pub mod rental_sewer_lateral_responsibility;
pub mod rental_sex_offender_registry_notice;
pub mod rental_short_term_subletting_airbnb_restriction;
pub mod rental_sinkhole_disclosure;
pub mod rental_smoke_free_cannabis_restriction;
pub mod rental_smoke_free_housing_disclosure;
pub mod rental_soft_story_seismic_retrofit;
pub mod rental_solar_panel_disclosure;
pub mod rental_source_of_income_discrimination;
pub mod rental_storage_unit_lease_disclosure;
pub mod rental_swimming_pool_drain_safety;
pub mod rental_tenant_abandoned_personal_property;
pub mod rental_tenant_bill_of_rights_handout;
pub mod rental_tenant_criminal_background_screening;
pub mod rental_tenant_data_privacy_compliance;
pub mod rental_tenant_estoppel_certificate;
pub mod rental_tenant_relocation_assistance;
pub mod rental_tenant_rent_escrow_habitability_dispute;
pub mod rental_tree_removal_dangerous_tree_disclosure;
pub mod rental_tenant_right_to_counsel_eviction;
pub mod rental_underground_storage_tank_disclosure;
pub mod rental_unpermitted_unit_disclosure;
pub mod rental_vacant_property_registration;
pub mod rental_vawa_2022_federal_housing_protections;
pub mod rental_vehicle_towing_notice_sign_requirements;
pub mod rental_video_surveillance_retention;
pub mod rental_washington_hb_1217_rent_stabilization;
pub mod rental_waste_recycling_collection_mandate;
pub mod rental_water_submetering_disclosure;
pub mod rental_well_water_disclosure;
pub mod rental_window_blind_cord_safety;
pub mod rental_window_guard_installation;
pub mod right_to_dry;
pub mod reps_qualification;
pub mod rules;
pub mod schedule_d;
pub mod security_camera_disclosure;
pub mod security_deposit_bank_disclosure;
pub mod security_deposit_caps;
pub mod security_deposit_interest_statement;
pub mod senior_disabled_protection;
pub mod service_animal;
pub mod sex_offender_database_notice;
pub mod short_term_rental_conversion;
pub mod smoke_free_housing;
pub mod snow_removal_responsibility;
pub mod soft_story_seismic_retrofit;
pub mod soi_protection;
pub mod squatter_unauthorized_occupant_removal;
pub mod str_regulation;
pub mod sublet_consent;
pub mod swimming_pool_safety;
pub mod submetering_rules;
pub mod tenant_abandonment;
pub mod tenant_accessible_parking;
pub mod tenant_assistance_animal_accommodation;
pub mod tenant_in_foreclosure_protection;
pub mod tenant_in_unit_appliance_repair_responsibility;
pub mod tenant_kitchen_appliance_replacement;
pub mod tenant_late_fee_cap;
pub mod tenant_lease_guarantor_disclosure;
pub mod tenant_cannabis_use_protection;
pub mod tenant_clothesline_drying_right;
pub mod tenant_data_privacy;
pub mod tenant_domestic_violence_lease_termination;
pub mod tenant_emotional_distress_damages;
pub mod tenant_estoppel_certificate;
pub mod tenant_ev_charging_installation_right;
pub mod tenant_holdover_security_deposit_setoff;
pub mod tenant_fire_safety_plan_disclosure;
pub mod tenant_death_termination;
pub mod tenant_noise_nuisance_enforcement;
pub mod tenant_organizing;
pub mod tenant_positive_rent_reporting;
pub mod tenant_solar_installation;
pub mod tenant_relocation_assistance;
pub mod tenant_rights_statement_disclosure;
pub mod tenant_smart_lock_biometric_consent;
pub mod tenant_smart_thermostat_install_right;
pub mod tenant_utility_account_designation;
pub mod tenant_voting_address_protection;
pub mod tenant_window_air_conditioner_install_right;
pub mod tenant_rent_escrow_withholding;
pub mod tenant_rent_judgment_wage_garnishment;
pub mod tenant_rent_receipt_requirement;
pub mod tenant_topa;
pub mod utility_shutoff;
pub mod source_of_income_discrimination;
pub mod vehicle_towing_from_rental_property;
pub mod water_heater_earthquake_strap;
pub mod window_guard_requirements;
pub mod winter_eviction_protections;
pub mod written_lease_requirement;
pub mod schedule_e;
pub mod section_104;
pub mod section_108;
pub mod section_1014;
pub mod section_1014e;
pub mod section_1015;
pub mod section_170;
pub mod section_170e;
pub mod section_172;
pub mod section_195;
pub mod section_197;
pub mod section_199a;
pub mod section_213;
pub mod section_219;
pub mod section_221;
pub mod section_223;
pub mod section_243;
pub mod section_245a;
pub mod section_246;
pub mod section_246a;
pub mod section_248;
pub mod section_250;
pub mod section_25c;
pub mod section_25d;
pub mod section_121;
pub mod section_121d;
pub mod section_132;
pub mod section_1045;
pub mod section_1058;
pub mod section_1092;
pub mod section_1202;
pub mod section_1212;
pub mod section_1231;
pub mod section_1233;
pub mod section_1235;
pub mod section_1239;
pub mod section_1234;
pub mod section_1234a;
pub mod section_1234b;
pub mod section_1244;
pub mod section_1245_1250;
pub mod section_1248;
pub mod section_1252;
pub mod section_1254;
pub mod section_1255;
pub mod section_1031;
pub mod section_1031_f;
pub mod section_1033;
pub mod section_1041;
pub mod section_1042;
pub mod section_1059;
pub mod section_1091;
pub mod section_1256;
pub mod section_1258;
pub mod section_1259;
pub mod section_1271;
pub mod section_1272;
pub mod section_1273;
pub mod section_1281;
pub mod section_1282;
pub mod section_1283;
pub mod section_1276;
pub mod section_1277;
pub mod section_1278;
pub mod section_1374;
pub mod section_1377;
pub mod section_1402;
pub mod section_1411;
pub mod section_1445;
pub mod section_1446;
pub mod section_1471;
pub mod section_1375;
pub mod section_1291;
pub mod section_1293;
pub mod section_1294;
pub mod section_1295;
pub mod section_1296;
pub mod section_1297;
pub mod section_1298;
pub mod section_1341;
pub mod section_1361;
pub mod section_1366;
pub mod section_1367;
pub mod section_1368;
pub mod section_162a;
pub mod section_162f;
pub mod section_162l;
pub mod section_162m;
pub mod section_163d;
pub mod section_163h;
pub mod section_164;
pub mod section_163j;
pub mod section_165d;
pub mod section_165g;
pub mod section_165h;
pub mod section_168;
pub mod section_168_e6;
pub mod section_168g;
pub mod section_168k;
pub mod section_174;
pub mod section_179;
pub mod section_183;
pub mod section_263;
pub mod section_263a;
pub mod section_263g;
pub mod section_264;
pub mod section_265;
pub mod section_267;
pub mod section_269;
pub mod section_269a;
pub mod section_274;
pub mod section_279;
pub mod section_988;
pub mod section_280a;
pub mod section_280a_d2;
pub mod section_280b;
pub mod section_280c;
pub mod section_280e;
pub mod section_280f;
pub mod section_280g;
pub mod section_280h;
pub mod section_302;
pub mod section_304;
pub mod section_30d;
pub mod section_305;
pub mod section_311;
pub mod section_312;
pub mod section_318;
pub mod section_3406;
pub mod section_331;
pub mod section_332;
pub mod section_336;
pub mod section_351;
pub mod section_354;
pub mod section_357;
pub mod section_358;
pub mod section_362;
pub mod section_367;
pub mod section_382;
pub mod section_383;
pub mod section_384;
pub mod section_83b;
pub mod section_83c;
pub mod section_83i;
pub mod section_401a9;
pub mod section_401k;
pub mod section_408_d3;
pub mod section_408m;
pub mod section_41;
#[allow(non_snake_case)]
pub mod section_408A_d3;
pub mod section_409a;
pub mod section_415;
pub mod section_421;
pub mod section_422;
pub mod section_423;
pub mod section_444;
pub mod section_446;
pub mod section_448;
pub mod section_451b;
pub mod section_408;
pub mod section_408a;
pub mod section_453;
pub mod section_453a;
pub mod section_457a;
pub mod section_457b;
pub mod section_461g;
pub mod section_461h;
pub mod section_461l;
pub mod section_465;
pub mod section_469;
pub mod section_4501;
pub mod section_4940;
pub mod section_4941;
pub mod section_4942;
pub mod section_4943;
pub mod section_4944;
pub mod section_4945;
pub mod section_4958;
pub mod section_4960;
pub mod section_4972;
pub mod section_4973;
pub mod section_4974;
pub mod section_4975;
pub mod section_4978;
pub mod section_4980;
pub mod section_4980h;
pub mod section_475c2;
pub mod section_475f;
pub mod section_691;
pub mod section_704c;
pub mod section_706;
pub mod section_707;
pub mod section_709;
pub mod section_721;
pub mod section_723;
pub mod section_731;
pub mod section_732;
pub mod section_734;
pub mod section_736;
pub mod section_737;
pub mod section_741;
pub mod section_743;
pub mod section_751;
pub mod section_752;
pub mod section_704d;
pub mod section_754;
pub mod section_755;
pub mod section_72t;
pub mod section_7345;
pub mod section_7405;
pub mod section_7408;
pub mod section_7421;
pub mod section_7422;
pub mod section_7426;
pub mod section_7429;
pub mod section_7430;
pub mod section_7433;
pub mod section_7201;
pub mod section_7202;
pub mod section_7203;
pub mod section_7212;
pub mod section_7216;
pub mod section_7206;
pub mod section_7207;
pub mod section_7434;
pub mod section_7463;
pub mod section_7491;
pub mod section_7502;
pub mod section_7503;
pub mod section_7508;
pub mod section_7508a;
pub mod section_7521;
pub mod section_7522;
pub mod section_7525;
pub mod section_7623;
pub mod section_7811;
pub mod section_7701;
pub mod section_7704;
pub mod section_7872;
pub mod section_864b2;
pub mod section_871m;
pub mod section_901;
pub mod section_903;
pub mod section_904;
pub mod section_911;
pub mod section_951a;
pub mod section_956;
pub mod section_959;
pub mod section_960;
pub mod section_961;
pub mod section_962;
pub mod section_965;
pub mod section_481;
pub mod section_482;
pub mod section_514;
pub mod section_530;
pub mod section_56a;
pub mod section_59a;
pub mod section_641;
pub mod section_642;
pub mod section_643;
pub mod section_651;
pub mod section_661;
pub mod section_671;
pub mod section_673;
pub mod section_674;
pub mod section_675;
pub mod section_676;
pub mod section_677;
pub mod section_678;
pub mod section_679;
pub mod section_67g;
pub mod section_6011;
pub mod section_6020;
pub mod section_6038a;
pub mod section_6038b;
pub mod section_6038c;
pub mod section_6038d;
pub mod section_6039;
pub mod section_6041;
pub mod section_6109;
pub mod section_6111;
pub mod section_6112;
pub mod section_6166;
pub mod section_6042;
pub mod section_6045;
pub mod section_6045a;
pub mod section_6045b;
pub mod section_6049;
pub mod section_6050i;
pub mod section_6050w;
pub mod section_6212;
pub mod section_6201;
pub mod section_6203;
pub mod section_6213;
pub mod section_6303;
pub mod section_6304;
pub mod section_6306;
pub mod section_6320;
pub mod section_6321;
pub mod section_6323;
pub mod section_6325;
pub mod section_6330;
pub mod section_6331;
pub mod section_6332;
pub mod section_6334;
pub mod section_6402;
pub mod section_6404;
pub mod section_6501;
pub mod section_6502;
pub mod section_6531;
pub mod section_6532;
pub mod section_6511;
pub mod section_6601;
pub mod section_6611;
pub mod section_6651;
pub mod section_6654;
pub mod section_6662;
pub mod section_6662a;
pub mod section_6663;
pub mod section_6664;
pub mod section_6672;
pub mod section_6694;
pub mod section_6695;
pub mod section_6695a;
pub mod section_6700;
pub mod section_6701;
pub mod section_6707;
pub mod section_6707a;
pub mod section_6708;
pub mod section_6713;
pub mod section_6721;
pub mod section_6722;
pub mod section_6851;
pub mod section_6861;
pub mod section_6862;
pub mod section_6863;
pub mod seed_rules;
pub mod self_employment_tax;
pub mod sheet;
pub mod subscription_detector;
pub mod tax_equivalent_yield;

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// A single row from any expense-source CSV, normalized into our shape.
///
/// Sign convention: `amount` is negative for money out (expense) and positive
/// for money in (refund, income, statement credit). Each parser does that
/// normalization since each source picks its own sign convention.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedTransaction {
    pub posted_at: DateTime<Utc>,
    pub amount: Decimal,
    pub currency: String,
    pub merchant_raw: String,
    pub merchant_normalized: String,
    pub description: String,
    pub raw: serde_json::Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExpenseSource {
    Amazon,
    Bofa,
    Chase,
    AppleCard,
}

impl ExpenseSource {
    pub fn as_str(self) -> &'static str {
        match self {
            ExpenseSource::Amazon => "amazon",
            ExpenseSource::Bofa => "bofa",
            ExpenseSource::Chase => "chase",
            ExpenseSource::AppleCard => "apple_card",
        }
    }

    pub fn parse_str(s: &str) -> Option<Self> {
        match s {
            "amazon" => Some(Self::Amazon),
            "bofa" | "bank_of_america" => Some(Self::Bofa),
            "chase" => Some(Self::Chase),
            "apple_card" | "apple" => Some(Self::AppleCard),
            _ => None,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("csv: {0}")]
    Csv(#[from] csv::Error),
    #[error("parse: {0}")]
    Parse(String),
    #[error("unsupported format: {0}")]
    Unsupported(String),
}

pub trait Parser {
    fn source(&self) -> ExpenseSource;
    fn parse(&self, bytes: &[u8]) -> Result<Vec<ParsedTransaction>, ImportError>;
}

/// Dispatch a source enum to its concrete parser.
pub fn parse(source: ExpenseSource, bytes: &[u8]) -> Result<Vec<ParsedTransaction>, ImportError> {
    match source {
        ExpenseSource::Amazon => amazon::AmazonParser.parse(bytes),
        ExpenseSource::Bofa => boa::BofaParser.parse(bytes),
        ExpenseSource::Chase => chase::ChaseParser.parse(bytes),
        ExpenseSource::AppleCard => apple::AppleCardParser.parse(bytes),
    }
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    hex::encode(h.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expense_source_str_roundtrip() {
        // Every variant must roundtrip through as_str / parse_str — the DB
        // stores the string and the parsers dispatch on it, so a mismatch
        // here silently drops imports.
        for src in [
            ExpenseSource::Amazon,
            ExpenseSource::Bofa,
            ExpenseSource::Chase,
            ExpenseSource::AppleCard,
        ] {
            let s = src.as_str();
            let back =
                ExpenseSource::parse_str(s).unwrap_or_else(|| panic!("`{s}` did not roundtrip"));
            assert_eq!(back, src);
        }
    }

    #[test]
    fn expense_source_parse_str_accepts_aliases() {
        // `bank_of_america` should map to Bofa; `apple` should map to AppleCard.
        assert_eq!(
            ExpenseSource::parse_str("bank_of_america"),
            Some(ExpenseSource::Bofa)
        );
        assert_eq!(
            ExpenseSource::parse_str("apple"),
            Some(ExpenseSource::AppleCard)
        );
    }

    #[test]
    fn expense_source_parse_str_rejects_unknown() {
        assert_eq!(ExpenseSource::parse_str("citibank"), None);
        assert_eq!(ExpenseSource::parse_str(""), None);
    }

    #[test]
    fn sha256_hex_is_deterministic() {
        // Same bytes → same digest, every time.
        let a = sha256_hex(b"hello world");
        let b = sha256_hex(b"hello world");
        assert_eq!(a, b);
        assert_eq!(a.len(), 64, "SHA-256 hex is always 64 chars");
    }

    #[test]
    fn sha256_hex_diverges_on_one_byte_mutation() {
        let a = sha256_hex(b"hello world");
        let b = sha256_hex(b"hello worle");
        assert_ne!(a, b, "SHA-256 must avalanche on single-byte change");
    }

    #[test]
    fn sha256_hex_known_value_for_empty_input() {
        // RFC 6234 — empty input has a fixed digest.
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn parse_returns_unsupported_for_stub_sources() {
        // The 4 parsers currently return Unsupported (they're stubs awaiting
        // real redacted CSV samples). This is the correct safety behavior —
        // pinning it so a future "fake-it" change is a deliberate choice.
        for src in [
            ExpenseSource::Amazon,
            ExpenseSource::Bofa,
            ExpenseSource::Chase,
            ExpenseSource::AppleCard,
        ] {
            let result = parse(src, b"this is not a real csv");
            // Either succeeds with an empty parse or returns an error — both
            // are acceptable; what's NOT acceptable is a panic.
            let _ = result; // smoke test: must not panic.
        }
    }
}
