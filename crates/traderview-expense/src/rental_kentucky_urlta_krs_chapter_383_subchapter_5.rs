//! Kentucky Uniform Residential Landlord and
//! Tenant Act (URLTA) — KRS Chapter 383
//! Subchapter 5 (KRS §§ 383.500-383.715)
//! Compliance Module — pure-compute check for landlord
//! statutory compliance with Kentucky's local-option URLTA
//! regime.
//!
//! **Most distinctive Kentucky feature**: **LOCAL OPTION
//! ADOPTION** under KRS § 383.500 — URLTA ONLY APPLIES in
//! jurisdictions that have specifically adopted it; outside
//! of those jurisdictions, common-law landlord-tenant
//! rules apply. This is **UNIQUE AMONG URLTA STATES** —
//! all other URLTA-adopting states (AZ, OH, TN, SC, IA,
//! AL, NM, NV) apply URLTA STATEWIDE.
//!
//! **Other distinctive Kentucky features**: **2x WRONGFUL
//! WITHHOLDING PENALTY + COURT COSTS + ATTORNEY FEES** for
//! BAD-FAITH retention under KRS § 383.580; **AUTOMATIC
//! FORFEITURE** of right to retain any portion of deposit
//! if landlord fails to return + provide itemized list
//! within 30 days; **7-DAY NONPAYMENT NOTICE** under KRS
//! § 383.660 (shorter than most URLTA states; cf. SC 5-day,
//! WI 5-day, AZ 5-day, but longer than OH 3-day); **2-DAY
//! (48-HOUR) ENTRY NOTICE** under KRS § 383.615 — UNUSUAL
//! among US states (most use 24 hours; KY uses 48 hours);
//! **HOT WATER AT ALL TIMES + REASONABLE HEAT BETWEEN
//! OCTOBER 1 AND MAY 1** under KRS § 383.595.
//!
//! Web research (verified 2026-06-03):
//! - **Kentucky URLTA Local-Option Adoption**: codified at KRS §§ 383.500-383.715; URLTA ONLY APPLIES in jurisdictions that have adopted it ([Kentucky Legislature — KRS Chapter 383](https://apps.legislature.ky.gov/law/statutes/chapter.aspx?id=39159); [Justia — KY Rev Stat § 383.500 Local Governments Authorized to Adopt URLTA](https://law.justia.com/codes/kentucky/2011/383-00/383-500/); [Justia — 2025 Kentucky Revised Statutes § 383.500](https://law.justia.com/codes/kentucky/chapter-383/section-383-500/); [Kentucky Legislature — KRS § 383.580 Security Deposits](https://apps.legislature.ky.gov/law/statutes/statute.aspx?id=35733); [Kentucky Legislature — KRS § 383.615 Access](https://apps.legislature.ky.gov/law/statutes/statute.aspx?id=35740); [Kentucky Legislature — KRS § 383.660 Tenant's Noncompliance with Rental Agreement; Failure to Pay Rent](https://apps.legislature.ky.gov/law/statutes/statute.aspx?id=35749); [Justia — KY Rev Stat § 383.660 (2011)](https://law.justia.com/codes/kentucky/2011/383-00/383-660/); [KFTC Archive — More Reasons Why the Uniform Residential Landlord Tenant Act PDF](https://archive.kftc.org/sites/default/files/docs/resources/more_reasons_to_support_urlta_handout-_rev._2015.11.10.pdf); [LegalClarity — Kentucky Landlord Tenant Law: Rights and Obligations](https://legalclarity.org/kentucky-landlord-tenant-law-a-guide-to-rights-and-regulations/); [McBrayer Firm — How to Evict a Tenant](https://www.mcbrayerfirm.com/blogs-Real-Estate-Law-Blog,how-to-evict-a-tenant); [Daily Independent — KPU in Full Force at Ashland Commission](https://www.dailyindependent.com/news/kpu-in-full-force-at-ashland-commission/article_52d9af48-42c7-11ee-9060-7b79ede75a0d.html); [Steadily — Kentucky Landlord Tenant Laws: A Comprehensive Guide](https://www.steadily.com/blog/kentucky-landlord-tenant-laws-a-comprehensive-guide); [LeaseLenses — Kentucky Landlord-Tenant Law Guide 2026](https://www.leaselenses.com/blog/kentucky-landlord-tenant-law-guide/); [Hemlane — Kentucky Security Deposit Laws in 2026](https://www.hemlane.com/resources/kentucky-security-deposit-laws/); [Hemlane — Kentucky Tenant-Landlord Rental Laws & Rights for 2026](https://www.hemlane.com/resources/kentucky-tenant-landlord-law/); [Rentable — Kentucky Security Deposit Laws Complete Guide](https://www.rentable.com/blog/kentucky-security-deposit-laws-a-complete-guide-for-landlords-tenants/); [DoorLoop — Kentucky Security Deposit Laws](https://www.doorloop.com/laws/kentucky-security-deposit-laws); [iPropertyManagement — Kentucky Security Deposit Law](https://ipropertymanagement.com/laws/kentucky-security-deposit-returns); [iPropertyManagement — Landlord's Right to Entry in Kentucky](https://ipropertymanagement.com/laws/kentucky-landlord-entry-rights); [FlagMyLease — Kentucky Tenant Rights: What Can't Be Waived](https://www.flagmylease.com/tenant-rights/kentucky); [Kentucky Landlord Law — You Talkin' To Me?](https://kylandlordlaw.com/blogs/news/62469061-you-talkin-to-me); [Lexington-Fayette UCHRC — URLTA PDF](https://www.lfuchrc.org/assets/landlord-tenant-act.pdf); [American Apartment Owners — Kentucky Landlord Tenant Laws](https://american-apartment-owners-association.org/landlord-tenant-laws/kentucky/); [LeaseRunner — Required Landlord Notices in Kentucky](https://www.leaserunner.com/laws/kentucky-required-landlord-notices)).
//! - **KRS § 383.500 Local-Option Adoption**: URLTA only applies in **JURISDICTIONS THAT HAVE ADOPTED IT** — including **LOUISVILLE-JEFFERSON COUNTY**, **LEXINGTON-FAYETTE COUNTY**, **COVINGTON**, **NEWPORT**, **FLORENCE**, **GEORGETOWN**, **SHELBYVILLE**, **OLDHAM COUNTY**, **PULASKI COUNTY**, and roughly a dozen smaller cities (Barbourville, Bellevue, Bromley, Dayton, Elsmere, Ludlow, Melbourne, Morgantown, Silver Grove, Southgate, Taylor Mill, Woodlawn); OUTSIDE of those jurisdictions, **COMMON-LAW LANDLORD-TENANT RULES** apply — UNIQUE AMONG URLTA STATES.
//! - **KRS § 383.580 Security Deposit Return — 30 Days**: landlords must return the security deposit within **30 DAYS** after the tenant vacates the unit; if there are deductions, landlords must provide an **ITEMIZED LIST** of those deductions within the same 30-day period.
//! - **KRS § 383.580 Automatic Forfeiture**: if the landlord fails to return the deposit OR provide an itemized list of deductions within 30 days, they **FORFEIT THE RIGHT TO RETAIN ANY PORTION** of the deposit — automatic statutory forfeiture.
//! - **KRS § 383.580 Bad-Faith 2x Penalty + Court Costs + Attorney Fees**: tenants may sue for the full deposit amount; if the landlord is found to have acted in **BAD FAITH**, the tenant may be entitled to recover **TWICE THE AMOUNT WRONGFULLY WITHHELD**, along with **COURT COSTS** and **ATTORNEY FEES**.
//! - **KRS § 383.595 Habitability — Five Obligations**: landlord shall **(a)** comply with the requirements of applicable **BUILDING AND HOUSING CODES** materially affecting health and safety; **(b)** make all repairs and do whatever is necessary to put and keep the premises in a **FIT AND HABITABLE CONDITION**; **(c)** keep all **COMMON AREAS** of the premises in a **CLEAN AND SAFE CONDITION**; **(d)** maintain in good and safe working order and condition all **ELECTRICAL, PLUMBING, SANITARY, HEATING, VENTILATING, AIR-CONDITIONING**, and other facilities and appliances, including **ELEVATORS**, supplied or required to be supplied by him; **(e)** supply **RUNNING WATER** and **REASONABLE AMOUNTS OF HOT WATER AT ALL TIMES** and **REASONABLE HEAT BETWEEN OCTOBER 1 AND MAY 1**.
//! - **KRS § 383.660 Nonpayment Notice — 7 Days**: for nonpayment of rent, a **7-DAY NOTICE TO PAY OR QUIT** is required; notice must state the total amount past due, when it became due, and a demand that it be paid within 7 days from the date of the notice.
//! - **KRS § 383.660 Material Noncompliance — 14 Days with Cure Right**: for material noncompliance, a **14-DAY NOTICE TO CURE OR QUIT** is required; notice must state the nature of the breach, whether or not the same breach has occurred in the past 6 months, and a demand to remedy the breach within 14 days.
//! - **KRS § 383.615 Entry Notice — 2 Days (48 Hours)**: except in case of emergency or unless it is impracticable to do so, the landlord shall give the tenant at least **TWO (2) DAYS' NOTICE** of his intent to enter; entry must occur at reasonable times; **UNUSUAL AMONG US STATES** — most states use 24 hours; KY uses 48 hours.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const KY_CHAPTER_NUMBER: u32 = 383;
pub const KY_DEPOSIT_RETURN_DEADLINE_DAYS: u32 = 30;
pub const KY_BAD_FAITH_WITHHOLDING_MULTIPLIER: u32 = 2;
pub const KY_NONPAYMENT_NOTICE_DAYS: u32 = 7;
pub const KY_MATERIAL_NONCOMPLIANCE_NOTICE_DAYS: u32 = 14;
pub const KY_MATERIAL_NONCOMPLIANCE_CURE_DAYS: u32 = 14;
pub const KY_REPEAT_BREACH_LOOKBACK_MONTHS: u32 = 6;
pub const KY_ENTRY_NOTICE_DAYS: u32 = 2;
pub const KY_ENTRY_NOTICE_HOURS: u32 = 48;
pub const KY_HEAT_OBLIGATION_START_MONTH: u32 = 10;
pub const KY_HEAT_OBLIGATION_END_MONTH: u32 = 5;
pub const KY_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JurisdictionAdoptionStatus {
    JurisdictionAdoptedUrlta,
    JurisdictionNotAdoptedUrltaCommonLawApplies,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DepositReturnAndItemizedListStatus {
    BothDepositReturnedAndItemizedListProvidedWithin30Days,
    EitherNotReturnedOrItemizedListNotProvided,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BadFaithStatus {
    BadFaithRetention,
    NoBadFaithGoodFaithDispute,
    NoBadFaithProperReturn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MaterialNoncomplianceStatus {
    TenantCorrectedNoncomplianceWithin14Days,
    TenantDidNotCorrectNoncomplianceWithin14Days,
    NoMaterialNoncompliance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HeatObligationPeriod {
    WithinOctober1ToMay1Period,
    OutsideHeatObligationPeriod,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAspect {
    LocalOptionAdoptionUnderSection383_500,
    SecurityDepositReturnAndItemizedListUnderSection383_580,
    BadFaithWithholding2xPenaltyUnderSection383_580,
    LandlordHabitabilityObligationUnderSection383_595,
    NonpaymentNoticeUnderSection383_660,
    MaterialNoncomplianceNoticeUnderSection383_660,
    LandlordEntryNoticeUnderSection383_615,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum KyLandlordTenantMode {
    NotApplicableJurisdictionNotAdoptedUrltaCommonLawApplies,
    CompliantJurisdictionAdoptedUrlta,
    CompliantBothDepositReturnedAndItemizedListProvidedWithin30Days,
    CompliantNoBadFaithGoodFaithDispute,
    CompliantNoBadFaithProperReturn,
    CompliantLandlordMaintainsHabitabilityUnderSection383_595,
    CompliantSevenDayNonpaymentNoticeProperlyServed,
    CompliantMaterialNoncompliance14DayCureWithTenantCorrection,
    CompliantMaterialNoncompliance14DayNoticeWithoutCure,
    CompliantNoMaterialNoncompliance,
    CompliantTwoDayEntryNoticeProperlyServed,
    CompliantEmergencyEntryWithoutNotice,
    ViolationAutomaticForfeitureFailedReturnOrItemizedListWithin30Days,
    ViolationBadFaithWithholding2xPenaltyTriggered,
    ViolationLandlordFailedHabitabilityObligation,
    ViolationLandlordFailedHeatObligationOctoberToMay,
    ViolationNonpaymentNoticeShorterThan7Days,
    ViolationMaterialNoncomplianceNoticeShorterThan14Days,
    ViolationEntryNoticeShorterThanTwoDays,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub jurisdiction_adoption_status: JurisdictionAdoptionStatus,
    pub deposit_return_and_itemized_list_status: DepositReturnAndItemizedListStatus,
    pub bad_faith_status: BadFaithStatus,
    pub material_noncompliance_status: MaterialNoncomplianceStatus,
    pub heat_obligation_period: HeatObligationPeriod,
    pub compliance_aspect: ComplianceAspect,
    pub days_to_return_deposit_and_itemized_list: u32,
    pub landlord_maintains_general_habitability: bool,
    pub landlord_provides_heat_when_required: bool,
    pub nonpayment_notice_days_given: u32,
    pub material_noncompliance_notice_days_given: u32,
    pub entry_notice_days_given: u32,
    pub entry_was_emergency_or_impracticable: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: KyLandlordTenantMode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type KyLandlordTenantInput = Input;
pub type KyLandlordTenantOutput = Output;
pub type KyLandlordTenantResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "Kentucky Uniform Residential Landlord and Tenant Act (URLTA) — codified at KRS §§ 383.500-383.715; LOCAL-OPTION ADOPTION".to_string(),
        "KRS § 383.500 Local-Option Adoption — URLTA only applies in JURISDICTIONS THAT HAVE ADOPTED IT, including LOUISVILLE-JEFFERSON COUNTY, LEXINGTON-FAYETTE COUNTY, COVINGTON, NEWPORT, FLORENCE, GEORGETOWN, SHELBYVILLE, OLDHAM COUNTY, PULASKI COUNTY, and roughly a dozen smaller cities (Barbourville, Bellevue, Bromley, Dayton, Elsmere, Ludlow, Melbourne, Morgantown, Silver Grove, Southgate, Taylor Mill, Woodlawn); OUTSIDE of those jurisdictions, COMMON-LAW LANDLORD-TENANT RULES apply; UNIQUE AMONG URLTA STATES".to_string(),
        "KRS § 383.580 Security Deposit Return — 30 Days — landlords must return the security deposit within 30 DAYS after the tenant vacates the unit; if there are deductions, landlords must provide an ITEMIZED LIST of those deductions within the same 30-day period".to_string(),
        "KRS § 383.580 Automatic Forfeiture — if the landlord fails to return the deposit OR provide an itemized list of deductions within 30 days, they FORFEIT THE RIGHT TO RETAIN ANY PORTION of the deposit".to_string(),
        "KRS § 383.580 Bad-Faith 2x Penalty + Court Costs + Attorney Fees — tenants may sue for the full deposit amount; if the landlord is found to have acted in BAD FAITH, the tenant may be entitled to recover TWICE THE AMOUNT WRONGFULLY WITHHELD, along with COURT COSTS and ATTORNEY FEES".to_string(),
        "KRS § 383.595 Habitability — Five Obligations — landlord shall (a) comply with BUILDING AND HOUSING CODES; (b) make all repairs and put premises in a FIT AND HABITABLE CONDITION; (c) keep COMMON AREAS clean and safe; (d) maintain ELECTRICAL, PLUMBING, SANITARY, HEATING, VENTILATING, AIR-CONDITIONING facilities including ELEVATORS; (e) supply RUNNING WATER and REASONABLE AMOUNTS OF HOT WATER AT ALL TIMES and REASONABLE HEAT BETWEEN OCTOBER 1 AND MAY 1".to_string(),
        "KRS § 383.660 Nonpayment Notice — 7 Days — for nonpayment of rent, a 7-DAY NOTICE TO PAY OR QUIT is required; notice must state the total amount past due, when it became due, and a demand that it be paid within 7 days from the date of the notice".to_string(),
        "KRS § 383.660 Material Noncompliance — 14 Days with Cure Right — for material noncompliance, a 14-DAY NOTICE TO CURE OR QUIT is required; notice must state the nature of the breach, whether or not the same breach has occurred in the past 6 months, and a demand to remedy the breach within 14 days".to_string(),
        "KRS § 383.615 Entry Notice — 2 Days (48 Hours) — except in case of emergency or unless it is impracticable to do so, the landlord shall give the tenant at least TWO (2) DAYS' NOTICE of his intent to enter; entry must occur at reasonable times; UNUSUAL AMONG US STATES — most use 24 hours; KY uses 48 hours".to_string(),
        "Kentucky Legislature + Justia + KFTC Archive + LegalClarity + McBrayer + Steadily + LeaseLenses + Hemlane + Rentable + DoorLoop + iPropertyManagement + FlagMyLease + Lexington-Fayette UCHRC + American Apartment Owners + LeaseRunner — practitioner overviews of Kentucky URLTA local-option regime".to_string(),
    ];

    if input.jurisdiction_adoption_status
        == JurisdictionAdoptionStatus::JurisdictionNotAdoptedUrltaCommonLawApplies
    {
        return Output {
            mode: KyLandlordTenantMode::NotApplicableJurisdictionNotAdoptedUrltaCommonLawApplies,
            statutory_basis: "KRS § 383.500 local-option — URLTA NOT ADOPTED in this jurisdiction; common-law landlord-tenant rules apply".to_string(),
            notes: "NOT APPLICABLE: Kentucky URLTA NOT ADOPTED in this jurisdiction under KRS § 383.500 local-option framework; common-law landlord-tenant rules apply; statutory URLTA obligations and remedies unavailable.".to_string(),
            citations,
        };
    }

    match input.compliance_aspect {
        ComplianceAspect::LocalOptionAdoptionUnderSection383_500 => Output {
            mode: KyLandlordTenantMode::CompliantJurisdictionAdoptedUrlta,
            statutory_basis: "KRS § 383.500 — URLTA ADOPTED in this jurisdiction".to_string(),
            notes: "COMPLIANT: Kentucky URLTA ADOPTED in this jurisdiction under KRS § 383.500 local-option framework; statutory URLTA obligations and remedies apply.".to_string(),
            citations,
        },
        ComplianceAspect::SecurityDepositReturnAndItemizedListUnderSection383_580 => {
            if input.days_to_return_deposit_and_itemized_list <= KY_DEPOSIT_RETURN_DEADLINE_DAYS
                && input.deposit_return_and_itemized_list_status
                    == DepositReturnAndItemizedListStatus::BothDepositReturnedAndItemizedListProvidedWithin30Days
            {
                Output {
                    mode: KyLandlordTenantMode::CompliantBothDepositReturnedAndItemizedListProvidedWithin30Days,
                    statutory_basis: "KRS § 383.580 — both deposit returned and itemized list provided within 30 days".to_string(),
                    notes: format!(
                        "COMPLIANT: both deposit returned and itemized list of deductions provided at day {d} (within 30-day statutory window) under KRS § 383.580.",
                        d = input.days_to_return_deposit_and_itemized_list,
                    ),
                    citations,
                }
            } else {
                Output {
                    mode: KyLandlordTenantMode::ViolationAutomaticForfeitureFailedReturnOrItemizedListWithin30Days,
                    statutory_basis: "KRS § 383.580 — automatic forfeiture of right to retain any portion of deposit".to_string(),
                    notes: format!(
                        "VIOLATION: landlord failed to return deposit OR provide itemized list of deductions within 30-day statutory window (day {d}, status {s:?}) under KRS § 383.580; landlord AUTOMATICALLY FORFEITS RIGHT TO RETAIN ANY PORTION of deposit.",
                        d = input.days_to_return_deposit_and_itemized_list,
                        s = input.deposit_return_and_itemized_list_status,
                    ),
                    citations,
                }
            }
        }
        ComplianceAspect::BadFaithWithholding2xPenaltyUnderSection383_580 => {
            match input.bad_faith_status {
                BadFaithStatus::BadFaithRetention => Output {
                    mode: KyLandlordTenantMode::ViolationBadFaithWithholding2xPenaltyTriggered,
                    statutory_basis: "KRS § 383.580 — bad-faith withholding triggers 2x damages + court costs + attorney fees".to_string(),
                    notes: "VIOLATION: landlord acted in BAD FAITH in withholding deposit under KRS § 383.580; tenant may recover TWICE THE AMOUNT WRONGFULLY WITHHELD plus COURT COSTS plus ATTORNEY FEES.".to_string(),
                    citations,
                },
                BadFaithStatus::NoBadFaithGoodFaithDispute => Output {
                    mode: KyLandlordTenantMode::CompliantNoBadFaithGoodFaithDispute,
                    statutory_basis: "KRS § 383.580 — no bad-faith; good-faith dispute".to_string(),
                    notes: "COMPLIANT: landlord retained deposit in good-faith dispute over deductions under KRS § 383.580; 2x bad-faith penalty exposure not triggered.".to_string(),
                    citations,
                },
                BadFaithStatus::NoBadFaithProperReturn => Output {
                    mode: KyLandlordTenantMode::CompliantNoBadFaithProperReturn,
                    statutory_basis: "KRS § 383.580 — no bad-faith; proper return".to_string(),
                    notes: "COMPLIANT: landlord properly returned deposit under KRS § 383.580; 2x bad-faith penalty exposure not triggered.".to_string(),
                    citations,
                },
            }
        }
        ComplianceAspect::LandlordHabitabilityObligationUnderSection383_595 => {
            if !input.landlord_maintains_general_habitability {
                return Output {
                    mode: KyLandlordTenantMode::ViolationLandlordFailedHabitabilityObligation,
                    statutory_basis: "KRS § 383.595 — landlord failed one or more habitability obligations".to_string(),
                    notes: "VIOLATION: landlord failed one or more of the five habitability obligations (building/housing codes, fit and habitable condition, common areas clean and safe, electrical/plumbing/sanitary/heating/ventilating/AC facilities, running water + hot water at all times + reasonable heat October 1 to May 1) under KRS § 383.595.".to_string(),
                    citations,
                };
            }
            if input.heat_obligation_period == HeatObligationPeriod::WithinOctober1ToMay1Period
                && !input.landlord_provides_heat_when_required
            {
                return Output {
                    mode: KyLandlordTenantMode::ViolationLandlordFailedHeatObligationOctoberToMay,
                    statutory_basis: "KRS § 383.595(e) — landlord failed REASONABLE HEAT obligation between October 1 and May 1".to_string(),
                    notes: "VIOLATION: landlord failed REASONABLE HEAT obligation BETWEEN OCTOBER 1 AND MAY 1 under KRS § 383.595(e); HOT WATER AT ALL TIMES + REASONABLE HEAT October-May is non-discretionary habitability standard.".to_string(),
                    citations,
                };
            }
            Output {
                mode: KyLandlordTenantMode::CompliantLandlordMaintainsHabitabilityUnderSection383_595,
                statutory_basis: "KRS § 383.595 — landlord maintains all five habitability obligations".to_string(),
                notes: "COMPLIANT: landlord maintains all five habitability obligations under KRS § 383.595 (building/housing codes + fit and habitable condition + common areas clean and safe + electrical/plumbing/sanitary/heating/ventilating/AC facilities + running water + hot water at all times + reasonable heat between October 1 and May 1).".to_string(),
                citations,
            }
        }
        ComplianceAspect::NonpaymentNoticeUnderSection383_660 => {
            if input.nonpayment_notice_days_given >= KY_NONPAYMENT_NOTICE_DAYS {
                Output {
                    mode: KyLandlordTenantMode::CompliantSevenDayNonpaymentNoticeProperlyServed,
                    statutory_basis: "KRS § 383.660 — 7-day nonpayment notice properly served".to_string(),
                    notes: format!(
                        "COMPLIANT: {d}-day nonpayment notice satisfies 7-day statutory minimum under KRS § 383.660.",
                        d = input.nonpayment_notice_days_given,
                    ),
                    citations,
                }
            } else {
                Output {
                    mode: KyLandlordTenantMode::ViolationNonpaymentNoticeShorterThan7Days,
                    statutory_basis: "KRS § 383.660 — nonpayment notice shorter than 7-day statutory minimum".to_string(),
                    notes: format!(
                        "VIOLATION: {d}-day nonpayment notice shorter than 7-day statutory minimum under KRS § 383.660.",
                        d = input.nonpayment_notice_days_given,
                    ),
                    citations,
                }
            }
        }
        ComplianceAspect::MaterialNoncomplianceNoticeUnderSection383_660 => {
            if input.material_noncompliance_notice_days_given
                < KY_MATERIAL_NONCOMPLIANCE_NOTICE_DAYS
            {
                return Output {
                    mode: KyLandlordTenantMode::ViolationMaterialNoncomplianceNoticeShorterThan14Days,
                    statutory_basis: "KRS § 383.660 — material noncompliance notice shorter than 14-day statutory minimum".to_string(),
                    notes: format!(
                        "VIOLATION: {d}-day material noncompliance notice shorter than 14-day statutory minimum under KRS § 383.660.",
                        d = input.material_noncompliance_notice_days_given,
                    ),
                    citations,
                };
            }
            match input.material_noncompliance_status {
                MaterialNoncomplianceStatus::TenantCorrectedNoncomplianceWithin14Days => Output {
                    mode: KyLandlordTenantMode::CompliantMaterialNoncompliance14DayCureWithTenantCorrection,
                    statutory_basis: "KRS § 383.660 — tenant cured material noncompliance within 14 days".to_string(),
                    notes: "COMPLIANT: tenant corrected material noncompliance within 14-day cure window under KRS § 383.660; tenancy continues.".to_string(),
                    citations,
                },
                MaterialNoncomplianceStatus::TenantDidNotCorrectNoncomplianceWithin14Days => Output {
                    mode: KyLandlordTenantMode::CompliantMaterialNoncompliance14DayNoticeWithoutCure,
                    statutory_basis: "KRS § 383.660 — 14-day notice served + tenant did not cure".to_string(),
                    notes: "COMPLIANT: 14-day material noncompliance notice properly served under KRS § 383.660; tenant did not correct within cure window; landlord may proceed with eviction.".to_string(),
                    citations,
                },
                MaterialNoncomplianceStatus::NoMaterialNoncompliance => Output {
                    mode: KyLandlordTenantMode::CompliantNoMaterialNoncompliance,
                    statutory_basis: "KRS § 383.660 — no material noncompliance".to_string(),
                    notes: "COMPLIANT: no material noncompliance condition present under KRS § 383.660; eviction notice not triggered.".to_string(),
                    citations,
                },
            }
        }
        ComplianceAspect::LandlordEntryNoticeUnderSection383_615 => {
            if input.entry_was_emergency_or_impracticable {
                return Output {
                    mode: KyLandlordTenantMode::CompliantEmergencyEntryWithoutNotice,
                    statutory_basis: "KRS § 383.615 — emergency or impracticable entry without 2-day notice".to_string(),
                    notes: "COMPLIANT: entry was emergency or impracticable to give notice under KRS § 383.615; 2-day notice requirement excused; entry must still occur at reasonable times.".to_string(),
                    citations,
                };
            }
            if input.entry_notice_days_given >= KY_ENTRY_NOTICE_DAYS {
                Output {
                    mode: KyLandlordTenantMode::CompliantTwoDayEntryNoticeProperlyServed,
                    statutory_basis: "KRS § 383.615 — 2-day entry notice properly served".to_string(),
                    notes: format!(
                        "COMPLIANT: {d}-day entry notice satisfies 2-day (48-hour) statutory minimum under KRS § 383.615.",
                        d = input.entry_notice_days_given,
                    ),
                    citations,
                }
            } else {
                Output {
                    mode: KyLandlordTenantMode::ViolationEntryNoticeShorterThanTwoDays,
                    statutory_basis: "KRS § 383.615 — entry notice shorter than 2-day statutory minimum".to_string(),
                    notes: format!(
                        "VIOLATION: {d}-day entry notice shorter than 2-day (48-hour) statutory minimum under KRS § 383.615.",
                        d = input.entry_notice_days_given,
                    ),
                    citations,
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
            jurisdiction_adoption_status: JurisdictionAdoptionStatus::JurisdictionAdoptedUrlta,
            deposit_return_and_itemized_list_status:
                DepositReturnAndItemizedListStatus::BothDepositReturnedAndItemizedListProvidedWithin30Days,
            bad_faith_status: BadFaithStatus::NoBadFaithProperReturn,
            material_noncompliance_status:
                MaterialNoncomplianceStatus::TenantCorrectedNoncomplianceWithin14Days,
            heat_obligation_period: HeatObligationPeriod::OutsideHeatObligationPeriod,
            compliance_aspect: ComplianceAspect::SecurityDepositReturnAndItemizedListUnderSection383_580,
            days_to_return_deposit_and_itemized_list: 25,
            landlord_maintains_general_habitability: true,
            landlord_provides_heat_when_required: true,
            nonpayment_notice_days_given: 7,
            material_noncompliance_notice_days_given: 14,
            entry_notice_days_given: 2,
            entry_was_emergency_or_impracticable: false,
        }
    }

    #[test]
    fn jurisdiction_not_adopted_urlta_common_law_applies_not_applicable() {
        let mut input = baseline_input();
        input.jurisdiction_adoption_status =
            JurisdictionAdoptionStatus::JurisdictionNotAdoptedUrltaCommonLawApplies;
        let out = check(&input);
        assert_eq!(
            out.mode,
            KyLandlordTenantMode::NotApplicableJurisdictionNotAdoptedUrltaCommonLawApplies
        );
    }

    #[test]
    fn jurisdiction_adopted_urlta_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LocalOptionAdoptionUnderSection383_500;
        let out = check(&input);
        assert_eq!(out.mode, KyLandlordTenantMode::CompliantJurisdictionAdoptedUrlta);
    }

    #[test]
    fn deposit_and_itemized_list_returned_at_30_day_boundary_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::SecurityDepositReturnAndItemizedListUnderSection383_580;
        input.days_to_return_deposit_and_itemized_list = 30;
        let out = check(&input);
        assert_eq!(
            out.mode,
            KyLandlordTenantMode::CompliantBothDepositReturnedAndItemizedListProvidedWithin30Days
        );
    }

    #[test]
    fn deposit_and_itemized_list_returned_at_31_days_automatic_forfeiture_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::SecurityDepositReturnAndItemizedListUnderSection383_580;
        input.days_to_return_deposit_and_itemized_list = 31;
        let out = check(&input);
        assert_eq!(
            out.mode,
            KyLandlordTenantMode::ViolationAutomaticForfeitureFailedReturnOrItemizedListWithin30Days
        );
    }

    #[test]
    fn deposit_returned_but_no_itemized_list_automatic_forfeiture_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::SecurityDepositReturnAndItemizedListUnderSection383_580;
        input.deposit_return_and_itemized_list_status =
            DepositReturnAndItemizedListStatus::EitherNotReturnedOrItemizedListNotProvided;
        let out = check(&input);
        assert_eq!(
            out.mode,
            KyLandlordTenantMode::ViolationAutomaticForfeitureFailedReturnOrItemizedListWithin30Days
        );
    }

    #[test]
    fn no_bad_faith_good_faith_dispute_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::BadFaithWithholding2xPenaltyUnderSection383_580;
        input.bad_faith_status = BadFaithStatus::NoBadFaithGoodFaithDispute;
        let out = check(&input);
        assert_eq!(out.mode, KyLandlordTenantMode::CompliantNoBadFaithGoodFaithDispute);
    }

    #[test]
    fn no_bad_faith_proper_return_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::BadFaithWithholding2xPenaltyUnderSection383_580;
        input.bad_faith_status = BadFaithStatus::NoBadFaithProperReturn;
        let out = check(&input);
        assert_eq!(out.mode, KyLandlordTenantMode::CompliantNoBadFaithProperReturn);
    }

    #[test]
    fn bad_faith_withholding_2x_penalty_triggered_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::BadFaithWithholding2xPenaltyUnderSection383_580;
        input.bad_faith_status = BadFaithStatus::BadFaithRetention;
        let out = check(&input);
        assert_eq!(
            out.mode,
            KyLandlordTenantMode::ViolationBadFaithWithholding2xPenaltyTriggered
        );
    }

    #[test]
    fn landlord_maintains_habitability_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LandlordHabitabilityObligationUnderSection383_595;
        input.landlord_maintains_general_habitability = true;
        input.landlord_provides_heat_when_required = true;
        let out = check(&input);
        assert_eq!(
            out.mode,
            KyLandlordTenantMode::CompliantLandlordMaintainsHabitabilityUnderSection383_595
        );
    }

    #[test]
    fn landlord_failed_habitability_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LandlordHabitabilityObligationUnderSection383_595;
        input.landlord_maintains_general_habitability = false;
        let out = check(&input);
        assert_eq!(
            out.mode,
            KyLandlordTenantMode::ViolationLandlordFailedHabitabilityObligation
        );
    }

    #[test]
    fn landlord_failed_heat_obligation_october_to_may_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LandlordHabitabilityObligationUnderSection383_595;
        input.landlord_maintains_general_habitability = true;
        input.heat_obligation_period = HeatObligationPeriod::WithinOctober1ToMay1Period;
        input.landlord_provides_heat_when_required = false;
        let out = check(&input);
        assert_eq!(
            out.mode,
            KyLandlordTenantMode::ViolationLandlordFailedHeatObligationOctoberToMay
        );
    }

    #[test]
    fn seven_day_nonpayment_notice_boundary_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::NonpaymentNoticeUnderSection383_660;
        input.nonpayment_notice_days_given = 7;
        let out = check(&input);
        assert_eq!(
            out.mode,
            KyLandlordTenantMode::CompliantSevenDayNonpaymentNoticeProperlyServed
        );
    }

    #[test]
    fn six_day_nonpayment_notice_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::NonpaymentNoticeUnderSection383_660;
        input.nonpayment_notice_days_given = 6;
        let out = check(&input);
        assert_eq!(
            out.mode,
            KyLandlordTenantMode::ViolationNonpaymentNoticeShorterThan7Days
        );
    }

    #[test]
    fn material_noncompliance_14_day_cure_with_correction_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::MaterialNoncomplianceNoticeUnderSection383_660;
        input.material_noncompliance_notice_days_given = 14;
        input.material_noncompliance_status =
            MaterialNoncomplianceStatus::TenantCorrectedNoncomplianceWithin14Days;
        let out = check(&input);
        assert_eq!(
            out.mode,
            KyLandlordTenantMode::CompliantMaterialNoncompliance14DayCureWithTenantCorrection
        );
    }

    #[test]
    fn material_noncompliance_14_day_notice_without_cure_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::MaterialNoncomplianceNoticeUnderSection383_660;
        input.material_noncompliance_notice_days_given = 14;
        input.material_noncompliance_status =
            MaterialNoncomplianceStatus::TenantDidNotCorrectNoncomplianceWithin14Days;
        let out = check(&input);
        assert_eq!(
            out.mode,
            KyLandlordTenantMode::CompliantMaterialNoncompliance14DayNoticeWithoutCure
        );
    }

    #[test]
    fn material_noncompliance_13_day_notice_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::MaterialNoncomplianceNoticeUnderSection383_660;
        input.material_noncompliance_notice_days_given = 13;
        let out = check(&input);
        assert_eq!(
            out.mode,
            KyLandlordTenantMode::ViolationMaterialNoncomplianceNoticeShorterThan14Days
        );
    }

    #[test]
    fn two_day_entry_notice_boundary_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LandlordEntryNoticeUnderSection383_615;
        input.entry_notice_days_given = 2;
        let out = check(&input);
        assert_eq!(
            out.mode,
            KyLandlordTenantMode::CompliantTwoDayEntryNoticeProperlyServed
        );
    }

    #[test]
    fn one_day_entry_notice_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LandlordEntryNoticeUnderSection383_615;
        input.entry_notice_days_given = 1;
        let out = check(&input);
        assert_eq!(
            out.mode,
            KyLandlordTenantMode::ViolationEntryNoticeShorterThanTwoDays
        );
    }

    #[test]
    fn emergency_entry_without_notice_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LandlordEntryNoticeUnderSection383_615;
        input.entry_was_emergency_or_impracticable = true;
        input.entry_notice_days_given = 0;
        let out = check(&input);
        assert_eq!(
            out.mode,
            KyLandlordTenantMode::CompliantEmergencyEntryWithoutNotice
        );
    }

    #[test]
    fn constants_pin_kentucky_landlord_tenant_statutory_thresholds() {
        assert_eq!(KY_CHAPTER_NUMBER, 383);
        assert_eq!(KY_DEPOSIT_RETURN_DEADLINE_DAYS, 30);
        assert_eq!(KY_BAD_FAITH_WITHHOLDING_MULTIPLIER, 2);
        assert_eq!(KY_NONPAYMENT_NOTICE_DAYS, 7);
        assert_eq!(KY_MATERIAL_NONCOMPLIANCE_NOTICE_DAYS, 14);
        assert_eq!(KY_MATERIAL_NONCOMPLIANCE_CURE_DAYS, 14);
        assert_eq!(KY_REPEAT_BREACH_LOOKBACK_MONTHS, 6);
        assert_eq!(KY_ENTRY_NOTICE_DAYS, 2);
        assert_eq!(KY_ENTRY_NOTICE_HOURS, 48);
        assert_eq!(KY_HEAT_OBLIGATION_START_MONTH, 10);
        assert_eq!(KY_HEAT_OBLIGATION_END_MONTH, 5);
        assert_eq!(KY_BASIS_POINT_DENOMINATOR, 10_000);
    }

    #[test]
    fn citations_pin_kentucky_landlord_tenant_statutory_provisions() {
        let input = baseline_input();
        let out = check(&input);
        let joined = out.citations.join(" || ");
        assert!(joined.contains("Kentucky Uniform Residential Landlord and Tenant Act"));
        assert!(joined.contains("KRS §§ 383.500-383.715"));
        assert!(joined.contains("LOCAL-OPTION ADOPTION"));
        assert!(joined.contains("KRS § 383.500"));
        assert!(joined.contains("JURISDICTIONS THAT HAVE ADOPTED IT"));
        assert!(joined.contains("LOUISVILLE-JEFFERSON COUNTY"));
        assert!(joined.contains("LEXINGTON-FAYETTE COUNTY"));
        assert!(joined.contains("COMMON-LAW LANDLORD-TENANT RULES"));
        assert!(joined.contains("KRS § 383.580"));
        assert!(joined.contains("30 DAYS"));
        assert!(joined.contains("ITEMIZED LIST"));
        assert!(joined.contains("FORFEIT THE RIGHT TO RETAIN ANY PORTION"));
        assert!(joined.contains("BAD FAITH"));
        assert!(joined.contains("TWICE THE AMOUNT WRONGFULLY WITHHELD"));
        assert!(joined.contains("COURT COSTS"));
        assert!(joined.contains("ATTORNEY FEES"));
        assert!(joined.contains("KRS § 383.595"));
        assert!(joined.contains("BUILDING AND HOUSING CODES"));
        assert!(joined.contains("FIT AND HABITABLE CONDITION"));
        assert!(joined.contains("COMMON AREAS"));
        assert!(joined.contains("ELECTRICAL, PLUMBING, SANITARY, HEATING, VENTILATING, AIR-CONDITIONING"));
        assert!(joined.contains("ELEVATORS"));
        assert!(joined.contains("RUNNING WATER"));
        assert!(joined.contains("REASONABLE AMOUNTS OF HOT WATER AT ALL TIMES"));
        assert!(joined.contains("REASONABLE HEAT BETWEEN OCTOBER 1 AND MAY 1"));
        assert!(joined.contains("KRS § 383.660"));
        assert!(joined.contains("7-DAY NOTICE TO PAY OR QUIT"));
        assert!(joined.contains("14-DAY NOTICE TO CURE OR QUIT"));
        assert!(joined.contains("6 months"));
        assert!(joined.contains("KRS § 383.615"));
        assert!(joined.contains("TWO (2) DAYS' NOTICE"));
        assert!(joined.contains("UNUSUAL AMONG US STATES"));
    }
}
