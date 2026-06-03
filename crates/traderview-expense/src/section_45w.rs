//! IRC § 45W Qualified Commercial Clean Vehicle Credit
//! Compliance Module — pure-compute check for business
//! taxpayer eligibility for the commercial clean vehicle
//! credit covering electric / hydrogen / plug-in hybrid
//! light-duty (< 14,000 lbs GVWR) and heavy-duty (≥ 14,000
//! lbs GVWR) commercial vehicles. Enacted by the
//! Inflation Reduction Act of 2022 and TERMINATED by the
//! One Big Beautiful Bill Act of 2025 for vehicles
//! acquired after September 30, 2025.
//!
//! Originally enacted by **Section 13403 of the Inflation
//! Reduction Act of 2022 (Public Law 117-169)**, signed by
//! President Joe Biden on **August 16, 2022**, effective
//! for vehicles acquired after **December 31, 2022**.
//! **TERMINATED by Section 70503 of the One Big Beautiful
//! Bill Act of 2025 (Public Law 119-21)**, signed by
//! President Donald Trump on **July 4, 2025**; § 45W
//! credit available ONLY for vehicles acquired on or
//! before **SEPTEMBER 30, 2025**; original IRA 2022 sunset
//! of December 31, 2032 accelerated by more than 7 years.
//!
//! Web research (verified 2026-06-03):
//! - **IRA 2022 Enactment**: IRC § 45W added by **Section 13403 of the Inflation Reduction Act of 2022 (Public Law 117-169, 136 Stat. 1818)**; signed by President Joe Biden on **August 16, 2022**; effective for vehicles acquired after **December 31, 2022** ([Inflation Reduction Act Tracker — IRA Section 13403](https://iratracker.org/programs/ira-section-13403-clean-commercial-vehicle-credit/); [Federal Register — Section 45W Credit for Qualified Commercial Clean Vehicles Proposed Regulations (January 14, 2025)](https://www.federalregister.gov/documents/2025/01/14/2025-00256/section-45w-credit-for-qualified-commercial-clean-vehicles); [IRS — Topic G FAQs Qualified Commercial Clean Vehicle Credit](https://www.irs.gov/newsroom/topic-g-frequently-asked-questions-about-qualified-commercial-clean-vehicle-credit); [Cornell LII — 26 U.S. Code § 45W](https://www.law.cornell.edu/uscode/text/26/45W); [House Office of Law Revision Counsel — 26 USC § 45W](https://uscode.house.gov/view.xhtml?req=(title:26+section:45W+edition:prelim)); [Current Federal Tax Developments — IRS Issues Proposed Regulations for § 45W (January 2025)](https://www.currentfederaltaxdevelopments.com/blog/2025/1/10/irs-issues-proposed-regulations-for-the-section-45w-commercial-clean-vehicle-credit); [RSM US — Proposed Regulations for Section 45W Qualified Commercial Clean Vehicle Credit](https://rsmus.com/insights/tax-alerts/2025/section-45w-qualified-commercial-clean-vehicle-credit.html); [IRS — Notice 2023-09 Section 45W Commercial Clean Vehicles and Incremental Cost](https://www.irs.gov/pub/irs-drop/n-23-09.pdf); [IRS — Notice 2022-56 Request for Comments on § 45W Credit](https://www.irs.gov/pub/irs-drop/n-22-56.pdf); [CALSTART — 45W Qualified Commercial Clean Vehicle Credit](https://calstart.org/wp-content/uploads/2024/02/45W-Qualified-Commercial-Clean-Vehicle-Credit.pdf)).
//! - **§ 45W(a) Credit Amount**: credit equal to **LESSER OF** (1) **15 PERCENT of vehicle basis** (**30 PERCENT** if vehicle NOT powered by gasoline or diesel internal combustion engine — i.e., fully electric / hydrogen / fuel cell), OR (2) **INCREMENTAL COST** of the qualifying vehicle (vs comparable conventional vehicle).
//! - **§ 45W(b)(4) Per-Vehicle Maximum Credit**: **$7,500** for vehicles with GVWR less than **14,000 LBS** (light-duty); **$40,000** for vehicles with GVWR of **14,000 LBS OR MORE** (heavy-duty commercial).
//! - **§ 45W(c) Eligible Business Taxpayers**: credit available to **C-CORPORATIONS and PASS-THROUGH ENTITIES** (S-corporations, partnerships, sole proprietorships) using vehicle for business purposes; not available to individuals using vehicle for personal use; **TAX-EXEMPT ENTITIES** eligible for **DIRECT PAY ELECTION** under **§ 6417**.
//! - **§ 45W(d) Qualified Commercial Clean Vehicle Definition**: vehicle must be (i) acquired for use or lease by the taxpayer and not for resale; (ii) made by a qualified manufacturer; (iii) propelled to a significant extent by electric motor with battery capacity ≥ 15 kWh (or ≥ 7 kWh for vehicles under 14,000 lbs GVWR) OR fuel cell vehicle.
//! - **Retail Price Equivalent (RPE)**: proposed regulations published January 14, 2025 introduced the **RPE concept** for incremental cost determination — RPE = manufacturer's suggested retail price (MSRP) divided by manufacturer's direct costs to produce the vehicle.
//! - **IRS Notice 2023-09 Incremental Cost Safe Harbor**: provides safe-harbor incremental cost values for various commercial clean vehicle categories.
//! - **Form 8936-A (Qualified Commercial Clean Vehicle Credit)**: required to claim the credit; separate from Form 8936 (individual clean vehicle credits).
//! - **OBBBA 2025 Termination**: § 45W ELIMINATED for vehicles acquired AFTER **SEPTEMBER 30, 2025** by **Section 70503 of Public Law 119-21 (One, Big, Beautiful Bill Act of 2025, 139 Stat. 72)**, signed by President Donald Trump on **JULY 4, 2025**; original IRA 2022 sunset of December 31, 2032 accelerated by more than **7 YEARS** (paralleling § 30D new clean vehicle credit termination).

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const IRC_45W_IRA_ENACTMENT_DATE_YEAR: u32 = 2022;
pub const IRC_45W_IRA_ENACTMENT_DATE_MONTH: u32 = 8;
pub const IRC_45W_IRA_ENACTMENT_DATE_DAY: u32 = 16;
pub const IRC_45W_IRA_PUBLIC_LAW_CONGRESS: u32 = 117;
pub const IRC_45W_IRA_PUBLIC_LAW_ENACTMENT: u32 = 169;
pub const IRC_45W_IRA_ENABLING_SECTION: u32 = 13403;
pub const IRC_45W_OBBBA_ENACTMENT_DATE_YEAR: u32 = 2025;
pub const IRC_45W_OBBBA_ENACTMENT_DATE_MONTH: u32 = 7;
pub const IRC_45W_OBBBA_ENACTMENT_DATE_DAY: u32 = 4;
pub const IRC_45W_OBBBA_PUBLIC_LAW_CONGRESS: u32 = 119;
pub const IRC_45W_OBBBA_PUBLIC_LAW_ENACTMENT: u32 = 21;
pub const IRC_45W_OBBBA_ENABLING_SECTION: u32 = 70503;
pub const IRC_45W_OBBBA_TERMINATION_DATE_YEAR: u32 = 2025;
pub const IRC_45W_OBBBA_TERMINATION_DATE_MONTH: u32 = 9;
pub const IRC_45W_OBBBA_TERMINATION_DATE_DAY: u32 = 30;
pub const IRC_45W_ORIGINAL_IRA_SUNSET_DATE_YEAR: u32 = 2032;
pub const IRC_45W_GAS_DIESEL_RATE_BPS: u64 = 1_500;
pub const IRC_45W_NON_ICE_RATE_BPS: u64 = 3_000;
pub const IRC_45W_LIGHT_DUTY_MAX_CREDIT_DOLLARS: u64 = 7_500;
pub const IRC_45W_HEAVY_DUTY_MAX_CREDIT_DOLLARS: u64 = 40_000;
pub const IRC_45W_HEAVY_DUTY_GVWR_THRESHOLD_LBS: u32 = 14_000;
pub const IRC_45W_LIGHT_DUTY_MIN_BATTERY_CAPACITY_KWH: u32 = 7;
pub const IRC_45W_HEAVY_DUTY_MIN_BATTERY_CAPACITY_KWH: u32 = 15;
pub const IRC_45W_PROPOSED_REGS_PUBLICATION_DATE_YEAR: u32 = 2025;
pub const IRC_45W_PROPOSED_REGS_PUBLICATION_DATE_MONTH: u32 = 1;
pub const IRC_45W_PROPOSED_REGS_PUBLICATION_DATE_DAY: u32 = 14;
pub const IRC_45W_FORM_NUMBER: u32 = 8936;
pub const IRC_45W_DIRECT_PAY_CROSS_REFERENCE_SECTION: u32 = 6417;
pub const IRC_45W_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AcquisitionDateStatus {
    AcquiredAfterDecember31_2022AndOnOrBeforeSeptember30_2025PostIraPreObbbaEligible,
    AcquiredOnOrBeforeDecember31_2022PreIra,
    AcquiredAfterSeptember30_2025PostObbbaTermination,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VehicleGvwrClass {
    LightDutyUnder14000Lbs,
    HeavyDutyAtOrAbove14000Lbs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PowerSource {
    GasolineOrDieselInternalCombustionPlugInHybrid,
    NonIceFullyElectricOrHydrogenFuelCell,
    NotEligiblePowerSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TaxpayerType {
    CCorporation,
    PassThroughEntityScorpOrPartnershipOrSoleProp,
    TaxExemptOrganizationWithDirectPayElection,
    IndividualUsingForPersonalUseNotEligible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAspect {
    CreditAmountUnderSection45WA,
    PerVehicleMaximumCreditUnderSection45WB4,
    EligibleBusinessTaxpayerUnderSection45WC,
    QualifiedCommercialCleanVehicleDefinitionUnderSection45WD,
    BatteryCapacityRequirement,
    DirectPayElectionForTaxExemptUnderSection6417,
    FormFilingUnderForm8936A,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section45WMode {
    NotApplicableAcquiredOnOrBeforeDecember31_2022PreIra,
    NotApplicableAcquiredAfterSeptember30_2025PostObbbaTermination,
    NotApplicableNotEligiblePowerSource,
    NotApplicableIndividualUsingForPersonalUse,
    CompliantLightDutyFifteenPercentCreditCappedAtSevenFiveHundred,
    CompliantLightDutyThirtyPercentCreditCappedAtSevenFiveHundred,
    CompliantHeavyDutyFifteenPercentCreditCappedAtFortyThousand,
    CompliantHeavyDutyThirtyPercentCreditCappedAtFortyThousand,
    CompliantCreditAtIncrementalCostLowerBound,
    CompliantEligibleBusinessTaxpayer,
    CompliantQualifiedCommercialCleanVehicle,
    CompliantBatteryCapacityMeetsThreshold,
    CompliantTaxExemptDirectPayElectionMade,
    CompliantForm8936AFiledCorrectly,
    ViolationCreditExceedsApplicableCap,
    ViolationBatteryCapacityBelowThreshold,
    ViolationNotQualifiedCommercialCleanVehicle,
    ViolationForm8936ANotFiledOrIncorrect,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub acquisition_date_status: AcquisitionDateStatus,
    pub vehicle_gvwr_class: VehicleGvwrClass,
    pub power_source: PowerSource,
    pub taxpayer_type: TaxpayerType,
    pub compliance_aspect: ComplianceAspect,
    pub vehicle_basis_dollars: u64,
    pub incremental_cost_dollars: u64,
    pub credit_claimed_dollars: u64,
    pub battery_capacity_kwh: u32,
    pub form_8936a_filed_correctly: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section45WMode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
    pub computed_credit_dollars: u64,
}

pub type Section45WInput = Input;
pub type Section45WOutput = Output;
pub type Section45WResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "IRC § 45W Qualified Commercial Clean Vehicle Credit added by Section 13403 of the Inflation Reduction Act of 2022 (Public Law 117-169, 136 Stat. 1818); signed by President Joe Biden on August 16, 2022; effective for vehicles acquired after December 31, 2022".to_string(),
        "IRC § 45W(a) Credit Amount — credit equal to LESSER OF (1) 15 PERCENT of vehicle basis (30 PERCENT if vehicle NOT powered by gasoline or diesel internal combustion engine — i.e., fully electric / hydrogen / fuel cell), OR (2) INCREMENTAL COST of the qualifying vehicle (vs comparable conventional vehicle)".to_string(),
        "IRC § 45W(b)(4) Per-Vehicle Maximum Credit — $7,500 for vehicles with GVWR less than 14,000 LBS (light-duty); $40,000 for vehicles with GVWR of 14,000 LBS OR MORE (heavy-duty commercial)".to_string(),
        "IRC § 45W(c) Eligible Business Taxpayers — credit available to C-CORPORATIONS and PASS-THROUGH ENTITIES (S-corporations, partnerships, sole proprietorships) using vehicle for business purposes; not available to individuals using vehicle for personal use".to_string(),
        "IRC § 45W(d) Qualified Commercial Clean Vehicle Definition — vehicle must be (i) acquired for use or lease by the taxpayer and not for resale; (ii) made by a qualified manufacturer; (iii) propelled to a significant extent by electric motor with battery capacity ≥ 15 kWh (for heavy-duty vehicles) or ≥ 7 kWh (for light-duty vehicles under 14,000 lbs GVWR) OR fuel cell vehicle".to_string(),
        "IRC § 6417 Direct Pay Election — tax-exempt entities (501(c)(3) organizations, state and local governments, Indian tribal governments, Alaska Native Corporations, rural electric cooperatives) eligible for direct pay election to receive § 45W credit as cash refund".to_string(),
        "Retail Price Equivalent (RPE) — proposed regulations published January 14, 2025 (90 FR final action) introduced the RPE concept for incremental cost determination; RPE = manufacturer's suggested retail price (MSRP) divided by manufacturer's direct costs to produce the vehicle".to_string(),
        "IRS Notice 2023-09 Incremental Cost Safe Harbor — provides safe-harbor incremental cost values for various commercial clean vehicle categories".to_string(),
        "IRS Notice 2022-56 Request for Comments — initial procedural guidance soliciting input on implementation of § 45W".to_string(),
        "Form 8936-A (Qualified Commercial Clean Vehicle Credit) — required to claim the credit; separate from Form 8936 (individual clean vehicle credits)".to_string(),
        "Federal Register — Section 45W Credit for Qualified Commercial Clean Vehicles Proposed Regulations (January 14, 2025) — proposed Treasury regulations under § 45W".to_string(),
        "Original IRA 2022 Sunset Date — § 45W credit was originally scheduled to apply to vehicles acquired through December 31, 2032 under IRA 2022".to_string(),
        "OBBBA 2025 Termination — § 45W ELIMINATED for vehicles acquired AFTER SEPTEMBER 30, 2025 by Section 70503 of Public Law 119-21 (One, Big, Beautiful Bill Act of 2025, 139 Stat. 72), signed by President Donald Trump on JULY 4, 2025; original IRA 2022 sunset of December 31, 2032 accelerated by more than 7 YEARS (paralleling § 30D new clean vehicle credit termination)".to_string(),
        "IRS — FAQs for Modification of Sections 25C, 25D, 25E, 30C, 30D, 45L, 45W, AND 179D under Public Law 119-21 — official IRS termination guidance".to_string(),
        "Inflation Reduction Act Tracker + Current Federal Tax Developments + RSM US + CALSTART — practitioner overviews of § 45W".to_string(),
    ];

    match input.acquisition_date_status {
        AcquisitionDateStatus::AcquiredOnOrBeforeDecember31_2022PreIra => {
            return Output {
                mode: Section45WMode::NotApplicableAcquiredOnOrBeforeDecember31_2022PreIra,
                statutory_basis: "IRA 2022 § 13403 effective date — § 45W applies only to vehicles acquired after December 31, 2022".to_string(),
                notes: "NOT APPLICABLE: vehicle acquired on or before December 31, 2022 (pre-IRA effective date); § 45W credit unavailable.".to_string(),
                citations,
                computed_credit_dollars: 0,
            };
        }
        AcquisitionDateStatus::AcquiredAfterSeptember30_2025PostObbbaTermination => {
            return Output {
                mode: Section45WMode::NotApplicableAcquiredAfterSeptember30_2025PostObbbaTermination,
                statutory_basis: "OBBBA 2025 § 70503 § 45W termination — vehicles acquired after September 30, 2025 ineligible".to_string(),
                notes: "NOT APPLICABLE: vehicle acquired after September 30, 2025; § 45W credit TERMINATED by Section 70503 of One Big Beautiful Bill Act of 2025 (Public Law 119-21, signed July 4, 2025); original IRA 2022 sunset of December 31, 2032 accelerated by more than 7 years.".to_string(),
                citations,
                computed_credit_dollars: 0,
            };
        }
        AcquisitionDateStatus::AcquiredAfterDecember31_2022AndOnOrBeforeSeptember30_2025PostIraPreObbbaEligible => {}
    }

    if input.power_source == PowerSource::NotEligiblePowerSource {
        return Output {
            mode: Section45WMode::NotApplicableNotEligiblePowerSource,
            statutory_basis: "IRC § 45W(d) — vehicle power source must qualify as electric / hydrogen / plug-in hybrid".to_string(),
            notes: "NOT APPLICABLE: vehicle power source does not qualify under § 45W(d); credit unavailable.".to_string(),
            citations,
            computed_credit_dollars: 0,
        };
    }

    if input.taxpayer_type == TaxpayerType::IndividualUsingForPersonalUseNotEligible {
        return Output {
            mode: Section45WMode::NotApplicableIndividualUsingForPersonalUse,
            statutory_basis: "IRC § 45W(c) — individual using vehicle for personal use not eligible".to_string(),
            notes: "NOT APPLICABLE: individual using vehicle for personal use ineligible under § 45W(c); § 30D new clean vehicle credit may apply instead for personal-use vehicles.".to_string(),
            citations,
            computed_credit_dollars: 0,
        };
    }

    match input.compliance_aspect {
        ComplianceAspect::CreditAmountUnderSection45WA => {
            let rate_bps = match input.power_source {
                PowerSource::GasolineOrDieselInternalCombustionPlugInHybrid => IRC_45W_GAS_DIESEL_RATE_BPS,
                PowerSource::NonIceFullyElectricOrHydrogenFuelCell => IRC_45W_NON_ICE_RATE_BPS,
                PowerSource::NotEligiblePowerSource => unreachable!(),
            };
            let basis_credit = (u128::from(input.vehicle_basis_dollars) * u128::from(rate_bps) / 10_000) as u64;
            let pre_cap_credit = basis_credit.min(input.incremental_cost_dollars);
            let cap = match input.vehicle_gvwr_class {
                VehicleGvwrClass::LightDutyUnder14000Lbs => IRC_45W_LIGHT_DUTY_MAX_CREDIT_DOLLARS,
                VehicleGvwrClass::HeavyDutyAtOrAbove14000Lbs => IRC_45W_HEAVY_DUTY_MAX_CREDIT_DOLLARS,
            };
            let computed = pre_cap_credit.min(cap);
            let mode = match (input.vehicle_gvwr_class, input.power_source) {
                (VehicleGvwrClass::LightDutyUnder14000Lbs, PowerSource::GasolineOrDieselInternalCombustionPlugInHybrid) => {
                    Section45WMode::CompliantLightDutyFifteenPercentCreditCappedAtSevenFiveHundred
                }
                (VehicleGvwrClass::LightDutyUnder14000Lbs, PowerSource::NonIceFullyElectricOrHydrogenFuelCell) => {
                    Section45WMode::CompliantLightDutyThirtyPercentCreditCappedAtSevenFiveHundred
                }
                (VehicleGvwrClass::HeavyDutyAtOrAbove14000Lbs, PowerSource::GasolineOrDieselInternalCombustionPlugInHybrid) => {
                    Section45WMode::CompliantHeavyDutyFifteenPercentCreditCappedAtFortyThousand
                }
                (VehicleGvwrClass::HeavyDutyAtOrAbove14000Lbs, PowerSource::NonIceFullyElectricOrHydrogenFuelCell) => {
                    Section45WMode::CompliantHeavyDutyThirtyPercentCreditCappedAtFortyThousand
                }
                _ => unreachable!(),
            };
            Output {
                mode,
                statutory_basis: "IRC § 45W(a) + (b)(4) — credit computed at applicable rate (15 %/30 %) up to incremental cost, capped at per-vehicle maximum ($7,500 / $40,000)".to_string(),
                notes: format!(
                    "COMPLIANT: § 45W credit computed = ${computed} (basis credit ${basis_credit} compared to incremental cost ${incr} capped at ${cap}).",
                    incr = input.incremental_cost_dollars,
                ),
                citations,
                computed_credit_dollars: computed,
            }
        }
        ComplianceAspect::PerVehicleMaximumCreditUnderSection45WB4 => {
            let cap = match input.vehicle_gvwr_class {
                VehicleGvwrClass::LightDutyUnder14000Lbs => IRC_45W_LIGHT_DUTY_MAX_CREDIT_DOLLARS,
                VehicleGvwrClass::HeavyDutyAtOrAbove14000Lbs => IRC_45W_HEAVY_DUTY_MAX_CREDIT_DOLLARS,
            };
            if input.credit_claimed_dollars <= cap {
                Output {
                    mode: match input.vehicle_gvwr_class {
                        VehicleGvwrClass::LightDutyUnder14000Lbs => {
                            Section45WMode::CompliantLightDutyFifteenPercentCreditCappedAtSevenFiveHundred
                        }
                        VehicleGvwrClass::HeavyDutyAtOrAbove14000Lbs => {
                            Section45WMode::CompliantHeavyDutyFifteenPercentCreditCappedAtFortyThousand
                        }
                    },
                    statutory_basis: "IRC § 45W(b)(4) — credit claimed within applicable per-vehicle maximum".to_string(),
                    notes: format!(
                        "COMPLIANT: credit claimed at or below ${cap} per-vehicle maximum under § 45W(b)(4)."
                    ),
                    citations,
                    computed_credit_dollars: input.credit_claimed_dollars,
                }
            } else {
                Output {
                    mode: Section45WMode::ViolationCreditExceedsApplicableCap,
                    statutory_basis: "IRC § 45W(b)(4) — credit claimed exceeds applicable per-vehicle maximum".to_string(),
                    notes: format!(
                        "VIOLATION: credit claimed exceeds ${cap} per-vehicle maximum under § 45W(b)(4); claim must be reduced to cap amount."
                    ),
                    citations,
                    computed_credit_dollars: cap,
                }
            }
        }
        ComplianceAspect::EligibleBusinessTaxpayerUnderSection45WC => {
            let mode = match input.taxpayer_type {
                TaxpayerType::TaxExemptOrganizationWithDirectPayElection => {
                    Section45WMode::CompliantTaxExemptDirectPayElectionMade
                }
                TaxpayerType::IndividualUsingForPersonalUseNotEligible => unreachable!(),
                _ => Section45WMode::CompliantEligibleBusinessTaxpayer,
            };
            Output {
                mode,
                statutory_basis: "IRC § 45W(c) — eligible business taxpayer status satisfied".to_string(),
                notes: "COMPLIANT: taxpayer qualifies as eligible business taxpayer under § 45W(c) (C-corp / pass-through / tax-exempt entity with § 6417 direct pay election).".to_string(),
                citations,
                computed_credit_dollars: 0,
            }
        }
        ComplianceAspect::QualifiedCommercialCleanVehicleDefinitionUnderSection45WD => Output {
            mode: Section45WMode::CompliantQualifiedCommercialCleanVehicle,
            statutory_basis: "IRC § 45W(d) — qualified commercial clean vehicle definition satisfied".to_string(),
            notes: "COMPLIANT: vehicle qualifies as qualified commercial clean vehicle under § 45W(d) (acquired for use or lease not resale; made by qualified manufacturer; meets battery capacity OR fuel cell vehicle).".to_string(),
            citations,
            computed_credit_dollars: 0,
        },
        ComplianceAspect::BatteryCapacityRequirement => {
            let min_threshold = match input.vehicle_gvwr_class {
                VehicleGvwrClass::LightDutyUnder14000Lbs => IRC_45W_LIGHT_DUTY_MIN_BATTERY_CAPACITY_KWH,
                VehicleGvwrClass::HeavyDutyAtOrAbove14000Lbs => IRC_45W_HEAVY_DUTY_MIN_BATTERY_CAPACITY_KWH,
            };
            if input.battery_capacity_kwh >= min_threshold {
                Output {
                    mode: Section45WMode::CompliantBatteryCapacityMeetsThreshold,
                    statutory_basis: "IRC § 45W(d) — battery capacity meets statutory threshold".to_string(),
                    notes: format!(
                        "COMPLIANT: battery capacity ≥ {min_threshold} kWh statutory threshold under § 45W(d) for vehicle class."
                    ),
                    citations,
                    computed_credit_dollars: 0,
                }
            } else {
                Output {
                    mode: Section45WMode::ViolationBatteryCapacityBelowThreshold,
                    statutory_basis: "IRC § 45W(d) — battery capacity below statutory threshold".to_string(),
                    notes: format!(
                        "VIOLATION: battery capacity below {min_threshold} kWh statutory threshold under § 45W(d); credit unavailable."
                    ),
                    citations,
                    computed_credit_dollars: 0,
                }
            }
        }
        ComplianceAspect::DirectPayElectionForTaxExemptUnderSection6417 => {
            if input.taxpayer_type == TaxpayerType::TaxExemptOrganizationWithDirectPayElection {
                Output {
                    mode: Section45WMode::CompliantTaxExemptDirectPayElectionMade,
                    statutory_basis: "IRC § 6417 — tax-exempt entity direct pay election available for § 45W credit".to_string(),
                    notes: "COMPLIANT: tax-exempt entity (501(c)(3) / state or local government / Indian tribal government / Alaska Native Corporation / rural electric cooperative) made § 6417 direct pay election to receive § 45W credit as cash refund.".to_string(),
                    citations,
                    computed_credit_dollars: 0,
                }
            } else {
                Output {
                    mode: Section45WMode::CompliantEligibleBusinessTaxpayer,
                    statutory_basis: "IRC § 6417 — direct pay election not applicable for non-tax-exempt taxpayer".to_string(),
                    notes: "NOT TRIGGERED: taxpayer is not tax-exempt entity; § 6417 direct pay election not applicable; standard credit application against tax liability.".to_string(),
                    citations,
                    computed_credit_dollars: 0,
                }
            }
        }
        ComplianceAspect::FormFilingUnderForm8936A => {
            if input.form_8936a_filed_correctly {
                Output {
                    mode: Section45WMode::CompliantForm8936AFiledCorrectly,
                    statutory_basis: "IRC § 45W — Form 8936-A filed correctly".to_string(),
                    notes: "COMPLIANT: business taxpayer filed Form 8936-A (Qualified Commercial Clean Vehicle Credit) correctly.".to_string(),
                    citations,
                    computed_credit_dollars: 0,
                }
            } else {
                Output {
                    mode: Section45WMode::ViolationForm8936ANotFiledOrIncorrect,
                    statutory_basis: "IRC § 45W — Form 8936-A not filed or incorrect".to_string(),
                    notes: "VIOLATION: Form 8936-A not filed or filed incorrectly; § 45W credit cannot be claimed without proper Form 8936-A filing.".to_string(),
                    citations,
                    computed_credit_dollars: 0,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_input() -> Input {
        Input {
            acquisition_date_status: AcquisitionDateStatus::AcquiredAfterDecember31_2022AndOnOrBeforeSeptember30_2025PostIraPreObbbaEligible,
            vehicle_gvwr_class: VehicleGvwrClass::LightDutyUnder14000Lbs,
            power_source: PowerSource::NonIceFullyElectricOrHydrogenFuelCell,
            taxpayer_type: TaxpayerType::CCorporation,
            compliance_aspect: ComplianceAspect::CreditAmountUnderSection45WA,
            vehicle_basis_dollars: 50_000,
            incremental_cost_dollars: 15_000,
            credit_claimed_dollars: 7_500,
            battery_capacity_kwh: 15,
            form_8936a_filed_correctly: true,
        }
    }

    #[test]
    fn pre_ira_acquisition_not_applicable() {
        let mut input = baseline_input();
        input.acquisition_date_status = AcquisitionDateStatus::AcquiredOnOrBeforeDecember31_2022PreIra;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section45WMode::NotApplicableAcquiredOnOrBeforeDecember31_2022PreIra
        );
    }

    #[test]
    fn post_obbba_termination_not_applicable() {
        let mut input = baseline_input();
        input.acquisition_date_status =
            AcquisitionDateStatus::AcquiredAfterSeptember30_2025PostObbbaTermination;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section45WMode::NotApplicableAcquiredAfterSeptember30_2025PostObbbaTermination
        );
    }

    #[test]
    fn not_eligible_power_source_not_applicable() {
        let mut input = baseline_input();
        input.power_source = PowerSource::NotEligiblePowerSource;
        let output = check(&input);
        assert_eq!(output.mode, Section45WMode::NotApplicableNotEligiblePowerSource);
    }

    #[test]
    fn individual_personal_use_not_applicable() {
        let mut input = baseline_input();
        input.taxpayer_type = TaxpayerType::IndividualUsingForPersonalUseNotEligible;
        let output = check(&input);
        assert_eq!(output.mode, Section45WMode::NotApplicableIndividualUsingForPersonalUse);
    }

    #[test]
    fn light_duty_thirty_percent_credit_capped_at_7500() {
        let output = check(&baseline_input());
        assert_eq!(
            output.mode,
            Section45WMode::CompliantLightDutyThirtyPercentCreditCappedAtSevenFiveHundred
        );
        assert_eq!(output.computed_credit_dollars, 7_500);
    }

    #[test]
    fn light_duty_fifteen_percent_credit_for_plug_in_hybrid() {
        let mut input = baseline_input();
        input.power_source = PowerSource::GasolineOrDieselInternalCombustionPlugInHybrid;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section45WMode::CompliantLightDutyFifteenPercentCreditCappedAtSevenFiveHundred
        );
        assert_eq!(output.computed_credit_dollars, 7_500);
    }

    #[test]
    fn light_duty_credit_at_incremental_cost_when_lower() {
        let mut input = baseline_input();
        input.incremental_cost_dollars = 5_000;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section45WMode::CompliantLightDutyThirtyPercentCreditCappedAtSevenFiveHundred
        );
        assert_eq!(output.computed_credit_dollars, 5_000);
    }

    #[test]
    fn heavy_duty_thirty_percent_credit_capped_at_40000() {
        let mut input = baseline_input();
        input.vehicle_gvwr_class = VehicleGvwrClass::HeavyDutyAtOrAbove14000Lbs;
        input.vehicle_basis_dollars = 200_000;
        input.incremental_cost_dollars = 80_000;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section45WMode::CompliantHeavyDutyThirtyPercentCreditCappedAtFortyThousand
        );
        assert_eq!(output.computed_credit_dollars, 40_000);
    }

    #[test]
    fn heavy_duty_fifteen_percent_credit_for_plug_in_hybrid() {
        let mut input = baseline_input();
        input.vehicle_gvwr_class = VehicleGvwrClass::HeavyDutyAtOrAbove14000Lbs;
        input.power_source = PowerSource::GasolineOrDieselInternalCombustionPlugInHybrid;
        input.vehicle_basis_dollars = 200_000;
        input.incremental_cost_dollars = 80_000;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section45WMode::CompliantHeavyDutyFifteenPercentCreditCappedAtFortyThousand
        );
        assert_eq!(output.computed_credit_dollars, 30_000);
    }

    #[test]
    fn light_duty_cap_violation_at_7501() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::PerVehicleMaximumCreditUnderSection45WB4;
        input.credit_claimed_dollars = 7_501;
        let output = check(&input);
        assert_eq!(output.mode, Section45WMode::ViolationCreditExceedsApplicableCap);
    }

    #[test]
    fn heavy_duty_cap_violation_at_40001() {
        let mut input = baseline_input();
        input.vehicle_gvwr_class = VehicleGvwrClass::HeavyDutyAtOrAbove14000Lbs;
        input.compliance_aspect = ComplianceAspect::PerVehicleMaximumCreditUnderSection45WB4;
        input.credit_claimed_dollars = 40_001;
        let output = check(&input);
        assert_eq!(output.mode, Section45WMode::ViolationCreditExceedsApplicableCap);
    }

    #[test]
    fn light_duty_battery_at_7_kwh_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::BatteryCapacityRequirement;
        input.battery_capacity_kwh = 7;
        let output = check(&input);
        assert_eq!(output.mode, Section45WMode::CompliantBatteryCapacityMeetsThreshold);
    }

    #[test]
    fn light_duty_battery_at_6_kwh_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::BatteryCapacityRequirement;
        input.battery_capacity_kwh = 6;
        let output = check(&input);
        assert_eq!(output.mode, Section45WMode::ViolationBatteryCapacityBelowThreshold);
    }

    #[test]
    fn heavy_duty_battery_at_15_kwh_compliant() {
        let mut input = baseline_input();
        input.vehicle_gvwr_class = VehicleGvwrClass::HeavyDutyAtOrAbove14000Lbs;
        input.compliance_aspect = ComplianceAspect::BatteryCapacityRequirement;
        input.battery_capacity_kwh = 15;
        let output = check(&input);
        assert_eq!(output.mode, Section45WMode::CompliantBatteryCapacityMeetsThreshold);
    }

    #[test]
    fn heavy_duty_battery_at_14_kwh_violation() {
        let mut input = baseline_input();
        input.vehicle_gvwr_class = VehicleGvwrClass::HeavyDutyAtOrAbove14000Lbs;
        input.compliance_aspect = ComplianceAspect::BatteryCapacityRequirement;
        input.battery_capacity_kwh = 14;
        let output = check(&input);
        assert_eq!(output.mode, Section45WMode::ViolationBatteryCapacityBelowThreshold);
    }

    #[test]
    fn eligible_c_corporation_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EligibleBusinessTaxpayerUnderSection45WC;
        let output = check(&input);
        assert_eq!(output.mode, Section45WMode::CompliantEligibleBusinessTaxpayer);
    }

    #[test]
    fn tax_exempt_direct_pay_election_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::DirectPayElectionForTaxExemptUnderSection6417;
        input.taxpayer_type = TaxpayerType::TaxExemptOrganizationWithDirectPayElection;
        let output = check(&input);
        assert_eq!(output.mode, Section45WMode::CompliantTaxExemptDirectPayElectionMade);
    }

    #[test]
    fn qualified_commercial_clean_vehicle_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::QualifiedCommercialCleanVehicleDefinitionUnderSection45WD;
        let output = check(&input);
        assert_eq!(output.mode, Section45WMode::CompliantQualifiedCommercialCleanVehicle);
    }

    #[test]
    fn form_8936a_filed_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::FormFilingUnderForm8936A;
        let output = check(&input);
        assert_eq!(output.mode, Section45WMode::CompliantForm8936AFiledCorrectly);
    }

    #[test]
    fn form_8936a_not_filed_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::FormFilingUnderForm8936A;
        input.form_8936a_filed_correctly = false;
        let output = check(&input);
        assert_eq!(output.mode, Section45WMode::ViolationForm8936ANotFiledOrIncorrect);
    }

    #[test]
    fn constants_pin_statutory_facts() {
        assert_eq!(IRC_45W_IRA_ENACTMENT_DATE_YEAR, 2022);
        assert_eq!(IRC_45W_IRA_PUBLIC_LAW_CONGRESS, 117);
        assert_eq!(IRC_45W_IRA_PUBLIC_LAW_ENACTMENT, 169);
        assert_eq!(IRC_45W_IRA_ENABLING_SECTION, 13403);
        assert_eq!(IRC_45W_OBBBA_ENACTMENT_DATE_YEAR, 2025);
        assert_eq!(IRC_45W_OBBBA_ENACTMENT_DATE_MONTH, 7);
        assert_eq!(IRC_45W_OBBBA_ENACTMENT_DATE_DAY, 4);
        assert_eq!(IRC_45W_OBBBA_PUBLIC_LAW_CONGRESS, 119);
        assert_eq!(IRC_45W_OBBBA_PUBLIC_LAW_ENACTMENT, 21);
        assert_eq!(IRC_45W_OBBBA_ENABLING_SECTION, 70503);
        assert_eq!(IRC_45W_OBBBA_TERMINATION_DATE_YEAR, 2025);
        assert_eq!(IRC_45W_OBBBA_TERMINATION_DATE_MONTH, 9);
        assert_eq!(IRC_45W_OBBBA_TERMINATION_DATE_DAY, 30);
        assert_eq!(IRC_45W_GAS_DIESEL_RATE_BPS, 1_500);
        assert_eq!(IRC_45W_NON_ICE_RATE_BPS, 3_000);
        assert_eq!(IRC_45W_LIGHT_DUTY_MAX_CREDIT_DOLLARS, 7_500);
        assert_eq!(IRC_45W_HEAVY_DUTY_MAX_CREDIT_DOLLARS, 40_000);
        assert_eq!(IRC_45W_HEAVY_DUTY_GVWR_THRESHOLD_LBS, 14_000);
        assert_eq!(IRC_45W_LIGHT_DUTY_MIN_BATTERY_CAPACITY_KWH, 7);
        assert_eq!(IRC_45W_HEAVY_DUTY_MIN_BATTERY_CAPACITY_KWH, 15);
        assert_eq!(IRC_45W_ORIGINAL_IRA_SUNSET_DATE_YEAR, 2032);
        assert_eq!(IRC_45W_DIRECT_PAY_CROSS_REFERENCE_SECTION, 6417);
        assert_eq!(IRC_45W_BASIS_POINT_DENOMINATOR, 10_000);
    }

    #[test]
    fn citations_contain_landmarks() {
        let output = check(&baseline_input());
        let joined = output.citations.join("\n");
        assert!(joined.contains("IRC § 45W"));
        assert!(joined.contains("Section 13403 of the Inflation Reduction Act of 2022"));
        assert!(joined.contains("Public Law 117-169"));
        assert!(joined.contains("August 16, 2022"));
        assert!(joined.contains("§ 45W(a)"));
        assert!(joined.contains("§ 45W(b)(4)"));
        assert!(joined.contains("§ 45W(c)"));
        assert!(joined.contains("§ 45W(d)"));
        assert!(joined.contains("§ 6417"));
        assert!(joined.contains("LESSER OF"));
        assert!(joined.contains("15 PERCENT"));
        assert!(joined.contains("30 PERCENT"));
        assert!(joined.contains("$7,500"));
        assert!(joined.contains("$40,000"));
        assert!(joined.contains("14,000 LBS"));
        assert!(joined.contains("INCREMENTAL COST"));
        assert!(joined.contains("C-CORPORATIONS"));
        assert!(joined.contains("PASS-THROUGH ENTITIES"));
        assert!(joined.contains("Direct Pay Election") || joined.contains("direct pay election"));
        assert!(joined.contains("Form 8936-A"));
        assert!(joined.contains("Notice 2023-09"));
        assert!(joined.contains("Section 70503"));
        assert!(joined.contains("Public Law 119-21"));
        assert!(joined.contains("SEPTEMBER 30, 2025"));
        assert!(joined.contains("JULY 4, 2025"));
        assert!(joined.contains("Retail Price Equivalent"));
    }
}
