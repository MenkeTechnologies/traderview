//! IRC § 45L New Energy Efficient Home Credit Compliance
//! Module — pure-compute check for eligible contractor
//! eligibility for the new energy efficient home credit
//! covering single-family + multifamily new construction
//! and substantial reconstruction. Enacted by the Energy
//! Policy Act of 2005, substantially expanded by the
//! Inflation Reduction Act of 2022, and TERMINATED by the
//! One Big Beautiful Bill Act of 2025 for homes acquired
//! after June 30, 2026.
//!
//! Originally enacted by **Section 1332 of the Energy
//! Policy Act of 2005 (Public Law 109-58)**, signed by
//! President George W. Bush on August 8, 2005.
//! Substantially expanded by **Section 13304 of the
//! Inflation Reduction Act of 2022 (Public Law 117-169)**,
//! signed by President Joe Biden on August 16, 2022,
//! effective for homes acquired after December 31, 2022.
//!
//! **TERMINATED by the One Big Beautiful Bill Act of 2025
//! (Public Law 119-21)**, signed by President Donald Trump
//! on **July 4, 2025**. § 45L credit available ONLY for
//! qualified new energy efficient homes acquired on or
//! before **JUNE 30, 2026**; original IRA 2022 sunset of
//! December 31, 2032 accelerated by more than 6 years.
//!
//! Web research (verified 2026-06-03):
//! - **EPAct 2005 Enactment**: IRC § 45L added by **Section 1332 of the Energy Policy Act of 2005 (Public Law 109-58, 119 Stat. 594)**, signed by President George W. Bush on **August 8, 2005**.
//! - **IRA 2022 Expansion**: substantially expanded by **Section 13304 of the Inflation Reduction Act of 2022 (Public Law 117-169, 136 Stat. 1818)**, signed by President Joe Biden on **August 16, 2022**, effective for homes acquired after **December 31, 2022** ([Department of Energy — Section 45L Tax Credits for DOE Efficient New Homes](https://www.energy.gov/cmei/buildings/section-45l-tax-credits-doe-efficient-new-homes); [Department of Energy — Section 45L Tax Credits for Zero Energy Ready Homes](https://www.energy.gov/eere/buildings/section-45l-tax-credits-zero-energy-ready-homes); [Building Innovation Hub — Understanding 45L and How to Earn the New Energy Efficient Home Tax Credit](https://buildinginnovationhub.org/understanding-45l-and-how-to-earn-the-new-energy-efficient-home-credit/); [IRS — Notice 2023-65 45L New Energy Efficient Home Credit PDF](https://www.irs.gov/pub/irs-drop/n-23-65.pdf); [Plante Moran — Expanded Section 45L Tax Credit](https://www.plantemoran.com/explore-our-thinking/insight/2024/01/expanded-section-45l-tax-credit-rewards-energy-efficient-developers); [Source Advisors — A Guide to Section 45L Pre and Post IRA](https://sourceadvisors.com/energy-tax-incentives/45l-tax-credits/a-look-at-section-45l-pre-and-post-ira/); [Inflation Reduction Act Tracker — IRA Section 13304](https://iratracker.org/programs/ira-section-13304-new-energy-efficient-home-credit/); [DOZ LLC — New Energy Efficient Home Credit Factsheet](https://dozllc.com/new-energy-efficient-home-credit-factsheet/); [CSA Partners — Section 45L Overview](https://csap.com/section-45l/); [EisnerAmper — Inflation Reduction Act Updates to the IRC Sec. 45L Tax Credit](https://www.eisneramper.com/insights/real-estate/ira-45l-credit-1122/); [IRS — Instructions for Form 8908 (12/2025)](https://www.irs.gov/instructions/i8908); [IRS — Instructions for Form 8908 (Rev. December 2025) PDF](https://www.irs.gov/pub/irs-pdf/i8908.pdf); [CSSI Services — Everything You Need to Know About the Updated 45L Energy Tax Credit](https://cssiservices.com/45l-tax-credit/); [Walker Reid Strategies — 45L Prevailing Wage Requirements](https://walkerreid.com/45l-prevailing-wage-requirements/); [CFMA — The Inflation Reduction Act's Amendments to the 45L Credit](https://cfma.org/articles/the-inflation-reduction-acts-amendments-to-the-45l-credit); [IRS — Prevailing Wage and Apprenticeship Under the IRA FAQs](https://www.irs.gov/credits-deductions/frequently-asked-questions-about-the-prevailing-wage-and-apprenticeship-under-the-inflation-reduction-act); [IRS — Publication 5855 Prevailing Wage & Registered Apprenticeship Overview PDF](https://www.irs.gov/pub/irs-pdf/p5855.pdf); [Federal Register — Increased Amounts of Credit for Satisfying Prevailing Wage and Registered Apprenticeship Requirements (June 25, 2024)](https://www.federalregister.gov/documents/2024/06/25/2024-13331/increased-amounts-of-credit-or-deduction-for-satisfying-certain-prevailing-wage-and-registered)).
//! - **§ 45L(a) Credit Amount — Single-Family / Manufactured Home**: **$2,500** for ENERGY STAR certified single-family / manufactured home; **$5,000** for DOE Zero Energy Ready Home (ZERH) / DOE Efficient New Homes certified single-family / manufactured home.
//! - **§ 45L(b) Credit Amount — Multifamily Home (per dwelling unit)**: **$500** for ENERGY STAR base (without prevailing wage); **$1,000** for DOE ZERH base (without prevailing wage); **$2,500** for ENERGY STAR + **PREVAILING WAGE** requirements met; **$5,000** for DOE ZERH + **PREVAILING WAGE** requirements met.
//! - **§ 45L(g) Prevailing Wage Requirement (Multifamily Only)**: prevailing wage requirements provide that taxpayers must ensure that all laborers and mechanics employed by the taxpayer (or any contractor or subcontractor) on the construction, alteration, or repair of qualified homes are paid wages at rates that are not less than the prevailing rates determined by the **Department of Labor in accordance with the Davis-Bacon Act** for the type of work performed in the geographic area of the facility.
//! - **No Apprenticeship Requirement**: § 45L is **UNIQUE** among IRA 2022 energy credits in requiring **ONLY prevailing wage** for the bonus multifamily multiplier — **NO APPRENTICESHIP requirements**.
//! - **§ 45L(c) Eligible Contractor Definition**: an **ELIGIBLE CONTRACTOR** is the person who owned and constructed or hired a third-party contractor to construct the qualified home; eligible contractor must be the owner of the home at the time of construction.
//! - **§ 45L(d) Acquired Definition**: home must be **ACQUIRED FROM the eligible contractor** for use as a residence; "acquired" means sale-and-purchase / first-sale acquisition.
//! - **DOE Zero Energy Ready Home Program**: DOE Zero Energy Ready Home (ZERH) is now known as **DOE Efficient New Homes**; certification requirements include ENERGY STAR Single-Family New Homes Program prerequisites + additional zero-energy-ready building science requirements.
//! - **Form 8908 (Energy Efficient Home Credit)**: required to claim the credit; current instructions IRS Form 8908 (Rev. December 2025).
//! - **OBBBA 2025 Termination**: § 45L ELIMINATED for qualified new energy efficient homes acquired AFTER **JUNE 30, 2026** by **Public Law 119-21 (One, Big, Beautiful Bill Act of 2025, 139 Stat. 72)**, signed by President Donald Trump on **JULY 4, 2025**; original IRA 2022 sunset of December 31, 2032 accelerated by more than **6 YEARS**.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const IRC_45L_EPACT_2005_ENACTMENT_DATE_YEAR: u32 = 2005;
pub const IRC_45L_EPACT_2005_ENACTMENT_DATE_MONTH: u32 = 8;
pub const IRC_45L_EPACT_2005_ENACTMENT_DATE_DAY: u32 = 8;
pub const IRC_45L_EPACT_2005_PUBLIC_LAW_CONGRESS: u32 = 109;
pub const IRC_45L_EPACT_2005_PUBLIC_LAW_ENACTMENT: u32 = 58;
pub const IRC_45L_EPACT_2005_ENABLING_SECTION: u32 = 1332;
pub const IRC_45L_IRA_2022_ENACTMENT_DATE_YEAR: u32 = 2022;
pub const IRC_45L_IRA_2022_ENACTMENT_DATE_MONTH: u32 = 8;
pub const IRC_45L_IRA_2022_ENACTMENT_DATE_DAY: u32 = 16;
pub const IRC_45L_IRA_2022_PUBLIC_LAW_CONGRESS: u32 = 117;
pub const IRC_45L_IRA_2022_PUBLIC_LAW_ENACTMENT: u32 = 169;
pub const IRC_45L_IRA_2022_ENABLING_SECTION: u32 = 13304;
pub const IRC_45L_IRA_2022_EFFECTIVE_DATE_YEAR: u32 = 2022;
pub const IRC_45L_IRA_2022_EFFECTIVE_DATE_MONTH: u32 = 12;
pub const IRC_45L_IRA_2022_EFFECTIVE_DATE_DAY: u32 = 31;
pub const IRC_45L_OBBBA_ENACTMENT_DATE_YEAR: u32 = 2025;
pub const IRC_45L_OBBBA_ENACTMENT_DATE_MONTH: u32 = 7;
pub const IRC_45L_OBBBA_ENACTMENT_DATE_DAY: u32 = 4;
pub const IRC_45L_OBBBA_PUBLIC_LAW_CONGRESS: u32 = 119;
pub const IRC_45L_OBBBA_PUBLIC_LAW_ENACTMENT: u32 = 21;
pub const IRC_45L_OBBBA_TERMINATION_DATE_YEAR: u32 = 2026;
pub const IRC_45L_OBBBA_TERMINATION_DATE_MONTH: u32 = 6;
pub const IRC_45L_OBBBA_TERMINATION_DATE_DAY: u32 = 30;
pub const IRC_45L_ORIGINAL_IRA_SUNSET_DATE_YEAR: u32 = 2032;
pub const IRC_45L_SINGLE_FAMILY_ENERGY_STAR_CREDIT_DOLLARS: u64 = 2_500;
pub const IRC_45L_SINGLE_FAMILY_ZERH_CREDIT_DOLLARS: u64 = 5_000;
pub const IRC_45L_MULTIFAMILY_ENERGY_STAR_BASE_CREDIT_DOLLARS: u64 = 500;
pub const IRC_45L_MULTIFAMILY_ZERH_BASE_CREDIT_DOLLARS: u64 = 1_000;
pub const IRC_45L_MULTIFAMILY_ENERGY_STAR_WITH_PREVAILING_WAGE_CREDIT_DOLLARS: u64 = 2_500;
pub const IRC_45L_MULTIFAMILY_ZERH_WITH_PREVAILING_WAGE_CREDIT_DOLLARS: u64 = 5_000;
pub const IRC_45L_FORM_NUMBER: u32 = 8908;
pub const IRC_45L_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AcquisitionDateStatus {
    AcquiredAfterDecember31_2022AndOnOrBeforeJune30_2026PostIraPreObbbaEligible,
    AcquiredOnOrBeforeDecember31_2022PreIraExpansion,
    AcquiredAfterJune30_2026PostObbbaTermination,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HomeType {
    SingleFamilyOrManufactured,
    MultifamilyDwellingUnit,
    NotEligibleHomeType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CertificationLevel {
    EnergyStarCertified,
    DoeZeroEnergyReadyHomeOrEfficientNewHomesCertified,
    NotCertified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAspect {
    SingleFamilyCreditUnderSection45LB1,
    MultifamilyCreditUnderSection45LB2,
    PrevailingWageRequirementForMultifamilyUnderSection45LG,
    EligibleContractorStatusUnderSection45LC,
    HomeLocatedInUnitedStatesRequirement,
    AcquisitionFromEligibleContractorUnderSection45LD,
    FormFilingUnderForm8908,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section45LMode {
    NotApplicableAcquiredOnOrBeforeDecember31_2022PreIraExpansion,
    NotApplicableAcquiredAfterJune30_2026PostObbbaTermination,
    NotApplicableNotEligibleHomeType,
    NotApplicableNotCertified,
    CompliantSingleFamilyEnergyStar2500Credit,
    CompliantSingleFamilyZerh5000Credit,
    CompliantMultifamilyEnergyStarBase500Credit,
    CompliantMultifamilyZerhBase1000Credit,
    CompliantMultifamilyEnergyStarWithPrevailingWage2500Credit,
    CompliantMultifamilyZerhWithPrevailingWage5000Credit,
    CompliantEligibleContractorStatus,
    CompliantHomeLocatedInUnitedStates,
    CompliantAcquisitionFromEligibleContractor,
    CompliantForm8908FiledCorrectly,
    ViolationNotEligibleContractor,
    ViolationHomeNotLocatedInUnitedStates,
    ViolationPrevailingWageNotMetForMultifamilyBonus,
    ViolationForm8908NotFiledOrIncorrect,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub acquisition_date_status: AcquisitionDateStatus,
    pub home_type: HomeType,
    pub certification_level: CertificationLevel,
    pub compliance_aspect: ComplianceAspect,
    pub prevailing_wage_requirement_met: bool,
    pub eligible_contractor_status: bool,
    pub home_located_in_united_states: bool,
    pub acquisition_from_eligible_contractor: bool,
    pub form_8908_filed_correctly: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section45LMode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
    pub credit_amount_dollars: u64,
}

pub type Section45LInput = Input;
pub type Section45LOutput = Output;
pub type Section45LResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "IRC § 45L New Energy Efficient Home Credit added by Section 1332 of the Energy Policy Act of 2005 (Public Law 109-58, 119 Stat. 594); signed by President George W. Bush on August 8, 2005".to_string(),
        "Inflation Reduction Act of 2022 § 13304 (Public Law 117-169, 136 Stat. 1818); signed by President Joe Biden on August 16, 2022; substantially expanded § 45L effective for homes acquired after December 31, 2022".to_string(),
        "IRC § 45L(a) Credit Amount — eligible contractor entitled to credit for each qualified new energy efficient home acquired by a person from the eligible contractor during the taxable year for use as a residence".to_string(),
        "IRC § 45L(b)(1) Single-Family / Manufactured Home Credit — $2,500 for ENERGY STAR certified; $5,000 for DOE Zero Energy Ready Home (ZERH) / DOE Efficient New Homes certified".to_string(),
        "IRC § 45L(b)(2) Multifamily Home Credit (per dwelling unit) — $500 for ENERGY STAR base (without prevailing wage); $1,000 for DOE ZERH base (without prevailing wage); $2,500 for ENERGY STAR + prevailing wage; $5,000 for DOE ZERH + prevailing wage".to_string(),
        "IRC § 45L(g) Prevailing Wage Requirement (Multifamily Only) — taxpayers must ensure all laborers and mechanics employed by taxpayer (or any contractor or subcontractor) on construction, alteration, or repair are paid wages at rates not less than prevailing rates determined by Department of Labor under the Davis-Bacon Act for the type of work performed in the geographic area".to_string(),
        "§ 45L Unique Among IRA Credits — § 45L requires ONLY prevailing wage for the bonus multifamily multiplier; NO APPRENTICESHIP requirements (distinct from § 30C / § 179D / § 30D / § 25E which require BOTH prevailing wage AND apprenticeship)".to_string(),
        "IRC § 45L(c) Eligible Contractor Definition — person who owned and constructed or hired a third-party contractor to construct the qualified home; eligible contractor must be the owner of the home at the time of construction".to_string(),
        "IRC § 45L(d) Acquired Definition — home must be ACQUIRED FROM the eligible contractor for use as a residence; 'acquired' means sale-and-purchase / first-sale acquisition".to_string(),
        "DOE Zero Energy Ready Home Program — DOE Zero Energy Ready Home (ZERH) is now known as DOE EFFICIENT NEW HOMES; certification requirements include ENERGY STAR Single-Family New Homes Program prerequisites + additional zero-energy-ready building science requirements".to_string(),
        "Form 8908 (Energy Efficient Home Credit) — required to claim the credit; current instructions IRS Form 8908 (Rev. December 2025)".to_string(),
        "IRS Notice 2023-65 — initial procedural guidance on § 45L credit amounts, certification requirements, and recordkeeping".to_string(),
        "Federal Register — Increased Amounts of Credit or Deduction for Satisfying Certain Prevailing Wage and Registered Apprenticeship Requirements (June 25, 2024) — operational PWA guidance applicable to § 45L prevailing wage requirement".to_string(),
        "Original IRA 2022 Sunset Date — § 45L credit was originally scheduled to expire December 31, 2032 under IRA 2022".to_string(),
        "OBBBA 2025 Termination — § 45L ELIMINATED for qualified new energy efficient homes acquired AFTER JUNE 30, 2026 by Public Law 119-21 (One, Big, Beautiful Bill Act of 2025, 139 Stat. 72), signed by President Donald Trump on JULY 4, 2025; original IRA 2022 sunset of December 31, 2032 accelerated by more than 6 YEARS".to_string(),
        "IRS — FAQs for Modification of Sections 25C, 25D, 25E, 30C, 30D, 45L, 45W, AND 179D under Public Law 119-21 — official IRS termination guidance".to_string(),
        "Department of Energy + Building Innovation Hub + Plante Moran + Source Advisors + DOZ LLC + CSA Partners + EisnerAmper + CSSI Services + Walker Reid Strategies + CFMA — practitioner overviews of § 45L".to_string(),
    ];

    match input.acquisition_date_status {
        AcquisitionDateStatus::AcquiredOnOrBeforeDecember31_2022PreIraExpansion => {
            return Output {
                mode: Section45LMode::NotApplicableAcquiredOnOrBeforeDecember31_2022PreIraExpansion,
                statutory_basis: "IRA 2022 § 13304 effective date — substantially expanded § 45L applies only to homes acquired after December 31, 2022".to_string(),
                notes: "NOT APPLICABLE: home acquired on or before December 31, 2022 (pre-IRA 2022 expansion); pre-IRA § 45L framework applies ($1,000 / $2,000 per dwelling unit with 50 % energy savings).".to_string(),
                citations,
                credit_amount_dollars: 0,
            };
        }
        AcquisitionDateStatus::AcquiredAfterJune30_2026PostObbbaTermination => {
            return Output {
                mode: Section45LMode::NotApplicableAcquiredAfterJune30_2026PostObbbaTermination,
                statutory_basis: "OBBBA 2025 § 45L termination — homes acquired after June 30, 2026 ineligible".to_string(),
                notes: "NOT APPLICABLE: home acquired after June 30, 2026; § 45L credit TERMINATED by One Big Beautiful Bill Act of 2025 (Public Law 119-21, signed July 4, 2025); original IRA 2022 sunset of December 31, 2032 accelerated by more than 6 years.".to_string(),
                citations,
                credit_amount_dollars: 0,
            };
        }
        AcquisitionDateStatus::AcquiredAfterDecember31_2022AndOnOrBeforeJune30_2026PostIraPreObbbaEligible => {}
    }

    if input.home_type == HomeType::NotEligibleHomeType {
        return Output {
            mode: Section45LMode::NotApplicableNotEligibleHomeType,
            statutory_basis: "IRC § 45L(b) — eligible home types limited to single-family / manufactured / multifamily dwelling unit".to_string(),
            notes: "NOT APPLICABLE: home does not qualify as an eligible single-family / manufactured / multifamily home under § 45L(b).".to_string(),
            citations,
            credit_amount_dollars: 0,
        };
    }

    if input.certification_level == CertificationLevel::NotCertified {
        return Output {
            mode: Section45LMode::NotApplicableNotCertified,
            statutory_basis: "IRC § 45L(c)(1) — home must be certified as ENERGY STAR OR DOE Zero Energy Ready Home (DOE Efficient New Homes)".to_string(),
            notes: "NOT APPLICABLE: home is not certified as ENERGY STAR OR DOE Zero Energy Ready Home (DOE Efficient New Homes); § 45L credit unavailable.".to_string(),
            citations,
            credit_amount_dollars: 0,
        };
    }

    match input.compliance_aspect {
        ComplianceAspect::SingleFamilyCreditUnderSection45LB1 => {
            if input.home_type != HomeType::SingleFamilyOrManufactured {
                return Output {
                    mode: Section45LMode::NotApplicableNotEligibleHomeType,
                    statutory_basis: "IRC § 45L(b)(1) — single-family / manufactured home credit pathway".to_string(),
                    notes: "NOT TRIGGERED: home is multifamily; § 45L(b)(2) multifamily credit pathway applies instead.".to_string(),
                    citations,
                    credit_amount_dollars: 0,
                };
            }
            match input.certification_level {
                CertificationLevel::EnergyStarCertified => Output {
                    mode: Section45LMode::CompliantSingleFamilyEnergyStar2500Credit,
                    statutory_basis: "IRC § 45L(b)(1) — single-family ENERGY STAR $2,500 credit".to_string(),
                    notes: "COMPLIANT: single-family / manufactured home ENERGY STAR certified; $2,500 credit under § 45L(b)(1).".to_string(),
                    citations,
                    credit_amount_dollars: IRC_45L_SINGLE_FAMILY_ENERGY_STAR_CREDIT_DOLLARS,
                },
                CertificationLevel::DoeZeroEnergyReadyHomeOrEfficientNewHomesCertified => Output {
                    mode: Section45LMode::CompliantSingleFamilyZerh5000Credit,
                    statutory_basis: "IRC § 45L(b)(1) — single-family ZERH $5,000 credit".to_string(),
                    notes: "COMPLIANT: single-family / manufactured home DOE Zero Energy Ready Home (DOE Efficient New Homes) certified; $5,000 credit under § 45L(b)(1).".to_string(),
                    citations,
                    credit_amount_dollars: IRC_45L_SINGLE_FAMILY_ZERH_CREDIT_DOLLARS,
                },
                CertificationLevel::NotCertified => unreachable!(),
            }
        }
        ComplianceAspect::MultifamilyCreditUnderSection45LB2 => {
            if input.home_type != HomeType::MultifamilyDwellingUnit {
                return Output {
                    mode: Section45LMode::NotApplicableNotEligibleHomeType,
                    statutory_basis: "IRC § 45L(b)(2) — multifamily credit pathway".to_string(),
                    notes: "NOT TRIGGERED: home is single-family / manufactured; § 45L(b)(1) single-family credit pathway applies instead.".to_string(),
                    citations,
                    credit_amount_dollars: 0,
                };
            }
            match (input.certification_level, input.prevailing_wage_requirement_met) {
                (CertificationLevel::EnergyStarCertified, false) => Output {
                    mode: Section45LMode::CompliantMultifamilyEnergyStarBase500Credit,
                    statutory_basis: "IRC § 45L(b)(2) — multifamily ENERGY STAR base $500 credit (without prevailing wage)".to_string(),
                    notes: "COMPLIANT: multifamily dwelling unit ENERGY STAR certified (without prevailing wage); $500 per-unit base credit under § 45L(b)(2).".to_string(),
                    citations,
                    credit_amount_dollars: IRC_45L_MULTIFAMILY_ENERGY_STAR_BASE_CREDIT_DOLLARS,
                },
                (CertificationLevel::DoeZeroEnergyReadyHomeOrEfficientNewHomesCertified, false) => Output {
                    mode: Section45LMode::CompliantMultifamilyZerhBase1000Credit,
                    statutory_basis: "IRC § 45L(b)(2) — multifamily ZERH base $1,000 credit (without prevailing wage)".to_string(),
                    notes: "COMPLIANT: multifamily dwelling unit DOE ZERH / DOE Efficient New Homes certified (without prevailing wage); $1,000 per-unit base credit under § 45L(b)(2).".to_string(),
                    citations,
                    credit_amount_dollars: IRC_45L_MULTIFAMILY_ZERH_BASE_CREDIT_DOLLARS,
                },
                (CertificationLevel::EnergyStarCertified, true) => Output {
                    mode: Section45LMode::CompliantMultifamilyEnergyStarWithPrevailingWage2500Credit,
                    statutory_basis: "IRC § 45L(b)(2) + § 45L(g) — multifamily ENERGY STAR with prevailing wage $2,500 credit".to_string(),
                    notes: "COMPLIANT: multifamily dwelling unit ENERGY STAR certified AND prevailing wage requirements met; $2,500 per-unit bonus credit under § 45L(b)(2) + § 45L(g) (5× multiplier on $500 base).".to_string(),
                    citations,
                    credit_amount_dollars: IRC_45L_MULTIFAMILY_ENERGY_STAR_WITH_PREVAILING_WAGE_CREDIT_DOLLARS,
                },
                (CertificationLevel::DoeZeroEnergyReadyHomeOrEfficientNewHomesCertified, true) => Output {
                    mode: Section45LMode::CompliantMultifamilyZerhWithPrevailingWage5000Credit,
                    statutory_basis: "IRC § 45L(b)(2) + § 45L(g) — multifamily ZERH with prevailing wage $5,000 credit".to_string(),
                    notes: "COMPLIANT: multifamily dwelling unit DOE ZERH / DOE Efficient New Homes certified AND prevailing wage requirements met; $5,000 per-unit bonus credit under § 45L(b)(2) + § 45L(g) (5× multiplier on $1,000 base).".to_string(),
                    citations,
                    credit_amount_dollars: IRC_45L_MULTIFAMILY_ZERH_WITH_PREVAILING_WAGE_CREDIT_DOLLARS,
                },
                (CertificationLevel::NotCertified, _) => unreachable!(),
            }
        }
        ComplianceAspect::PrevailingWageRequirementForMultifamilyUnderSection45LG => {
            if input.prevailing_wage_requirement_met {
                Output {
                    mode: Section45LMode::CompliantMultifamilyEnergyStarWithPrevailingWage2500Credit,
                    statutory_basis: "IRC § 45L(g) — prevailing wage requirement met for multifamily bonus".to_string(),
                    notes: "COMPLIANT: prevailing wage requirement met under § 45L(g) for multifamily bonus credit; 5× multiplier applies on base $500 / $1,000 credit.".to_string(),
                    citations,
                    credit_amount_dollars: 0,
                }
            } else {
                Output {
                    mode: Section45LMode::ViolationPrevailingWageNotMetForMultifamilyBonus,
                    statutory_basis: "IRC § 45L(g) — prevailing wage requirement not met for multifamily bonus".to_string(),
                    notes: "VIOLATION: prevailing wage requirement not met under § 45L(g) for multifamily bonus credit; only $500 / $1,000 base credit applies (no 5× multiplier).".to_string(),
                    citations,
                    credit_amount_dollars: 0,
                }
            }
        }
        ComplianceAspect::EligibleContractorStatusUnderSection45LC => {
            if input.eligible_contractor_status {
                Output {
                    mode: Section45LMode::CompliantEligibleContractorStatus,
                    statutory_basis: "IRC § 45L(c) — eligible contractor status satisfied".to_string(),
                    notes: "COMPLIANT: taxpayer is eligible contractor under § 45L(c) (person who owned and constructed or hired third-party contractor to construct the qualified home AND was the owner at the time of construction).".to_string(),
                    citations,
                    credit_amount_dollars: 0,
                }
            } else {
                Output {
                    mode: Section45LMode::ViolationNotEligibleContractor,
                    statutory_basis: "IRC § 45L(c) — not eligible contractor".to_string(),
                    notes: "VIOLATION: taxpayer does NOT qualify as eligible contractor under § 45L(c); credit unavailable.".to_string(),
                    citations,
                    credit_amount_dollars: 0,
                }
            }
        }
        ComplianceAspect::HomeLocatedInUnitedStatesRequirement => {
            if input.home_located_in_united_states {
                Output {
                    mode: Section45LMode::CompliantHomeLocatedInUnitedStates,
                    statutory_basis: "IRC § 45L(c) — home located in United States requirement satisfied".to_string(),
                    notes: "COMPLIANT: qualified new energy efficient home is located in the United States as required by § 45L(c).".to_string(),
                    citations,
                    credit_amount_dollars: 0,
                }
            } else {
                Output {
                    mode: Section45LMode::ViolationHomeNotLocatedInUnitedStates,
                    statutory_basis: "IRC § 45L(c) — home not located in United States".to_string(),
                    notes: "VIOLATION: home not located in the United States; § 45L(c) requires US location; credit unavailable.".to_string(),
                    citations,
                    credit_amount_dollars: 0,
                }
            }
        }
        ComplianceAspect::AcquisitionFromEligibleContractorUnderSection45LD => {
            if input.acquisition_from_eligible_contractor {
                Output {
                    mode: Section45LMode::CompliantAcquisitionFromEligibleContractor,
                    statutory_basis: "IRC § 45L(d) — home acquired from eligible contractor for use as residence".to_string(),
                    notes: "COMPLIANT: home acquired from eligible contractor for use as a residence under § 45L(d).".to_string(),
                    citations,
                    credit_amount_dollars: 0,
                }
            } else {
                Output {
                    mode: Section45LMode::ViolationNotEligibleContractor,
                    statutory_basis: "IRC § 45L(d) — home not acquired from eligible contractor".to_string(),
                    notes: "VIOLATION: home not acquired from eligible contractor under § 45L(d); credit unavailable.".to_string(),
                    citations,
                    credit_amount_dollars: 0,
                }
            }
        }
        ComplianceAspect::FormFilingUnderForm8908 => {
            if input.form_8908_filed_correctly {
                Output {
                    mode: Section45LMode::CompliantForm8908FiledCorrectly,
                    statutory_basis: "IRC § 45L — Form 8908 filed correctly".to_string(),
                    notes: "COMPLIANT: eligible contractor filed Form 8908 (Energy Efficient Home Credit) correctly; current instructions IRS Form 8908 (Rev. December 2025).".to_string(),
                    citations,
                    credit_amount_dollars: 0,
                }
            } else {
                Output {
                    mode: Section45LMode::ViolationForm8908NotFiledOrIncorrect,
                    statutory_basis: "IRC § 45L — Form 8908 not filed or incorrect".to_string(),
                    notes: "VIOLATION: Form 8908 not filed or filed incorrectly; § 45L credit cannot be claimed without proper Form 8908 filing.".to_string(),
                    citations,
                    credit_amount_dollars: 0,
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
            acquisition_date_status: AcquisitionDateStatus::AcquiredAfterDecember31_2022AndOnOrBeforeJune30_2026PostIraPreObbbaEligible,
            home_type: HomeType::SingleFamilyOrManufactured,
            certification_level: CertificationLevel::EnergyStarCertified,
            compliance_aspect: ComplianceAspect::SingleFamilyCreditUnderSection45LB1,
            prevailing_wage_requirement_met: false,
            eligible_contractor_status: true,
            home_located_in_united_states: true,
            acquisition_from_eligible_contractor: true,
            form_8908_filed_correctly: true,
        }
    }

    #[test]
    fn pre_ira_acquisition_not_applicable() {
        let mut input = baseline_input();
        input.acquisition_date_status =
            AcquisitionDateStatus::AcquiredOnOrBeforeDecember31_2022PreIraExpansion;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section45LMode::NotApplicableAcquiredOnOrBeforeDecember31_2022PreIraExpansion
        );
    }

    #[test]
    fn post_obbba_termination_not_applicable() {
        let mut input = baseline_input();
        input.acquisition_date_status =
            AcquisitionDateStatus::AcquiredAfterJune30_2026PostObbbaTermination;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section45LMode::NotApplicableAcquiredAfterJune30_2026PostObbbaTermination
        );
    }

    #[test]
    fn not_eligible_home_type_not_applicable() {
        let mut input = baseline_input();
        input.home_type = HomeType::NotEligibleHomeType;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section45LMode::NotApplicableNotEligibleHomeType
        );
    }

    #[test]
    fn not_certified_not_applicable() {
        let mut input = baseline_input();
        input.certification_level = CertificationLevel::NotCertified;
        let output = check(&input);
        assert_eq!(output.mode, Section45LMode::NotApplicableNotCertified);
    }

    #[test]
    fn single_family_energy_star_2500_compliant() {
        let output = check(&baseline_input());
        assert_eq!(
            output.mode,
            Section45LMode::CompliantSingleFamilyEnergyStar2500Credit
        );
        assert_eq!(output.credit_amount_dollars, 2_500);
    }

    #[test]
    fn single_family_zerh_5000_compliant() {
        let mut input = baseline_input();
        input.certification_level =
            CertificationLevel::DoeZeroEnergyReadyHomeOrEfficientNewHomesCertified;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section45LMode::CompliantSingleFamilyZerh5000Credit
        );
        assert_eq!(output.credit_amount_dollars, 5_000);
    }

    #[test]
    fn multifamily_energy_star_base_500_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::MultifamilyCreditUnderSection45LB2;
        input.home_type = HomeType::MultifamilyDwellingUnit;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section45LMode::CompliantMultifamilyEnergyStarBase500Credit
        );
        assert_eq!(output.credit_amount_dollars, 500);
    }

    #[test]
    fn multifamily_zerh_base_1000_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::MultifamilyCreditUnderSection45LB2;
        input.home_type = HomeType::MultifamilyDwellingUnit;
        input.certification_level =
            CertificationLevel::DoeZeroEnergyReadyHomeOrEfficientNewHomesCertified;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section45LMode::CompliantMultifamilyZerhBase1000Credit
        );
        assert_eq!(output.credit_amount_dollars, 1_000);
    }

    #[test]
    fn multifamily_energy_star_with_pwa_2500_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::MultifamilyCreditUnderSection45LB2;
        input.home_type = HomeType::MultifamilyDwellingUnit;
        input.prevailing_wage_requirement_met = true;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section45LMode::CompliantMultifamilyEnergyStarWithPrevailingWage2500Credit
        );
        assert_eq!(output.credit_amount_dollars, 2_500);
    }

    #[test]
    fn multifamily_zerh_with_pwa_5000_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::MultifamilyCreditUnderSection45LB2;
        input.home_type = HomeType::MultifamilyDwellingUnit;
        input.certification_level =
            CertificationLevel::DoeZeroEnergyReadyHomeOrEfficientNewHomesCertified;
        input.prevailing_wage_requirement_met = true;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section45LMode::CompliantMultifamilyZerhWithPrevailingWage5000Credit
        );
        assert_eq!(output.credit_amount_dollars, 5_000);
    }

    #[test]
    fn prevailing_wage_met_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::PrevailingWageRequirementForMultifamilyUnderSection45LG;
        input.prevailing_wage_requirement_met = true;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section45LMode::CompliantMultifamilyEnergyStarWithPrevailingWage2500Credit
        );
    }

    #[test]
    fn prevailing_wage_not_met_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::PrevailingWageRequirementForMultifamilyUnderSection45LG;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section45LMode::ViolationPrevailingWageNotMetForMultifamilyBonus
        );
    }

    #[test]
    fn eligible_contractor_status_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EligibleContractorStatusUnderSection45LC;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section45LMode::CompliantEligibleContractorStatus
        );
    }

    #[test]
    fn not_eligible_contractor_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EligibleContractorStatusUnderSection45LC;
        input.eligible_contractor_status = false;
        let output = check(&input);
        assert_eq!(output.mode, Section45LMode::ViolationNotEligibleContractor);
    }

    #[test]
    fn home_in_us_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::HomeLocatedInUnitedStatesRequirement;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section45LMode::CompliantHomeLocatedInUnitedStates
        );
    }

    #[test]
    fn home_not_in_us_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::HomeLocatedInUnitedStatesRequirement;
        input.home_located_in_united_states = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section45LMode::ViolationHomeNotLocatedInUnitedStates
        );
    }

    #[test]
    fn acquisition_from_eligible_contractor_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::AcquisitionFromEligibleContractorUnderSection45LD;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section45LMode::CompliantAcquisitionFromEligibleContractor
        );
    }

    #[test]
    fn form_8908_filed_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::FormFilingUnderForm8908;
        let output = check(&input);
        assert_eq!(output.mode, Section45LMode::CompliantForm8908FiledCorrectly);
    }

    #[test]
    fn form_8908_not_filed_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::FormFilingUnderForm8908;
        input.form_8908_filed_correctly = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section45LMode::ViolationForm8908NotFiledOrIncorrect
        );
    }

    #[test]
    fn constants_pin_statutory_facts() {
        assert_eq!(IRC_45L_EPACT_2005_ENACTMENT_DATE_YEAR, 2005);
        assert_eq!(IRC_45L_EPACT_2005_PUBLIC_LAW_CONGRESS, 109);
        assert_eq!(IRC_45L_EPACT_2005_PUBLIC_LAW_ENACTMENT, 58);
        assert_eq!(IRC_45L_EPACT_2005_ENABLING_SECTION, 1332);
        assert_eq!(IRC_45L_IRA_2022_ENACTMENT_DATE_YEAR, 2022);
        assert_eq!(IRC_45L_IRA_2022_ENABLING_SECTION, 13304);
        assert_eq!(IRC_45L_OBBBA_ENACTMENT_DATE_YEAR, 2025);
        assert_eq!(IRC_45L_OBBBA_ENACTMENT_DATE_MONTH, 7);
        assert_eq!(IRC_45L_OBBBA_ENACTMENT_DATE_DAY, 4);
        assert_eq!(IRC_45L_OBBBA_PUBLIC_LAW_CONGRESS, 119);
        assert_eq!(IRC_45L_OBBBA_PUBLIC_LAW_ENACTMENT, 21);
        assert_eq!(IRC_45L_OBBBA_TERMINATION_DATE_YEAR, 2026);
        assert_eq!(IRC_45L_OBBBA_TERMINATION_DATE_MONTH, 6);
        assert_eq!(IRC_45L_OBBBA_TERMINATION_DATE_DAY, 30);
        assert_eq!(IRC_45L_ORIGINAL_IRA_SUNSET_DATE_YEAR, 2032);
        assert_eq!(IRC_45L_SINGLE_FAMILY_ENERGY_STAR_CREDIT_DOLLARS, 2_500);
        assert_eq!(IRC_45L_SINGLE_FAMILY_ZERH_CREDIT_DOLLARS, 5_000);
        assert_eq!(IRC_45L_MULTIFAMILY_ENERGY_STAR_BASE_CREDIT_DOLLARS, 500);
        assert_eq!(IRC_45L_MULTIFAMILY_ZERH_BASE_CREDIT_DOLLARS, 1_000);
        assert_eq!(
            IRC_45L_MULTIFAMILY_ENERGY_STAR_WITH_PREVAILING_WAGE_CREDIT_DOLLARS,
            2_500
        );
        assert_eq!(
            IRC_45L_MULTIFAMILY_ZERH_WITH_PREVAILING_WAGE_CREDIT_DOLLARS,
            5_000
        );
        assert_eq!(IRC_45L_FORM_NUMBER, 8908);
        assert_eq!(IRC_45L_BASIS_POINT_DENOMINATOR, 10_000);
    }

    #[test]
    fn citations_contain_landmarks() {
        let output = check(&baseline_input());
        let joined = output.citations.join("\n");
        assert!(joined.contains("IRC § 45L"));
        assert!(joined.contains("Section 1332 of the Energy Policy Act of 2005"));
        assert!(joined.contains("Public Law 109-58"));
        assert!(joined.contains("119 Stat. 594"));
        assert!(joined.contains("August 8, 2005"));
        assert!(joined.contains("Inflation Reduction Act of 2022 § 13304"));
        assert!(joined.contains("Public Law 117-169"));
        assert!(joined.contains("§ 45L(b)(1)"));
        assert!(joined.contains("§ 45L(b)(2)"));
        assert!(joined.contains("§ 45L(c)"));
        assert!(joined.contains("§ 45L(d)"));
        assert!(joined.contains("§ 45L(g)"));
        assert!(joined.contains("$2,500"));
        assert!(joined.contains("$5,000"));
        assert!(joined.contains("$500"));
        assert!(joined.contains("$1,000"));
        assert!(joined.contains("ENERGY STAR"));
        assert!(joined.contains("DOE Zero Energy Ready Home"));
        assert!(joined.contains("DOE Efficient New Homes"));
        assert!(joined.contains("Davis-Bacon Act"));
        assert!(joined.contains("NO APPRENTICESHIP"));
        assert!(joined.contains("Form 8908"));
        assert!(joined.contains("Notice 2023-65"));
        assert!(joined.contains("Public Law 119-21"));
        assert!(joined.contains("JUNE 30, 2026"));
        assert!(joined.contains("JULY 4, 2025"));
    }
}
