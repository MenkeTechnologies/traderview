//! IRC § 47 Rehabilitation Tax Credit (Historic Tax
//! Credit / HTC) Compliance Module — pure-compute check
//! for the rehabilitation credit, commonly referred to as
//! the **HISTORIC PRESERVATION TAX CREDIT** or **HISTORIC
//! TAX CREDIT (HTC)**; provides a tax incentive to
//! rehabilitate historic buildings.
//!
//! Provides a **20% TAX CREDIT** for qualified
//! rehabilitation expenditures on certified historic
//! structures, allocated **RATABLY OVER A 5-YEAR PERIOD**
//! beginning in the year placed in service (post-TCJA 2017).
//! Subject to a **5-YEAR RECAPTURE PERIOD** with sliding
//! recapture percentages on early disposition.
//!
//! Often used together with § 42 LIHTC for affordable
//! housing in historic buildings ("twinning" structure).
//!
//! Web research (verified 2026-06-03):
//! - **IRC § 47 Rehabilitation Tax Credit**: governed by 26 U.S. Code § 47; commonly referred to as the **HISTORIC PRESERVATION TAX CREDIT** or **HISTORIC TAX CREDIT (HTC)** ([Cornell LII — 26 U.S. Code § 47](https://www.law.cornell.edu/uscode/text/26/47); [IRS — Rehabilitation Credit](https://www.irs.gov/businesses/small-businesses-self-employed/rehabilitation-credit); [IRS — Rehabilitation Credit (historic preservation) FAQs](https://www.irs.gov/businesses/small-businesses-self-employed/rehabilitation-credit-historic-preservation-faqs); [Tax Notes — IRC Code Section 47 (Rehabilitation Tax Credit)](https://www.taxnotes.com/research/federal/usc26/47); [Bloomberg Tax — Sec. 47 Rehabilitation Credit](https://irc.bloombergtax.com/public/uscode/doc/irc/section_47); [House USC — Section 47](https://uscode.house.gov/view.xhtml?req=(title:26+section:47+edition:prelim)); [The Tax Credit Exchange — Section 47 Rehabilitation Tax Credit](https://www.taxcreditex.com/section-47); [AndreTaxCo — Investment Credit - Rehabilitation](https://www.andretaxco.com/investment-credit-building-rehabili); [Accounting Insights — Section 47: The Rehabilitation Tax Credit & Recapture](https://accountinginsights.org/section-47-the-rehabilitation-tax-credit-recapture/); [The Tax Adviser — Determining Qualified Rehabilitation Expenditures (April 2021)](https://www.thetaxadviser.com/issues/2021/apr/determining-qualified-rehabilitation-expenditures/); [The Sherbert Group — Substantial Rehab Test: Historic Tax Credit](https://www.sherbertgroup.com/substantial-rehabilitation-test-qualify-for-20-historic-tax-credit/); [Novogradac — Understanding the Substantial Rehabilitation Test for Historic Tax Credits FAQs](https://www.novoco.com/periodicals/articles/understanding-the-substantial-rehabilitation-test-for-historic-tax-credits-faqs); [EisnerAmper — How to Claim the Historic Tax Credit: Eligibility and Steps](https://www.eisneramper.com/insights/blogs/real-estate-blog/historic-tax-credit-re-blog-0122/); [CSG Law — IRS Issues Proposed Regulations on Claiming Rehabilitation Tax Credits](https://www.csglaw.com/newsroom/irs-issues-proposed-regulations-on-claiming-rehabilitation-tax-credits/); [The Tax Adviser — Recent Changes to the Rehabilitation Tax Credit (January 2021)](https://www.thetaxadviser.com/issues/2021/jan/changes-rehabilitation-tax-credit/); [IRS — Rehabilitation Credit Recapture PDF](https://www.irs.gov/pub/irs-sbse/rehabilitation-credit-recapture.pdf)).
//! - **§ 47(a)(1) 20% Credit Rate for Certified Historic Structures**: tax credit equal to **20% OF QUALIFIED REHABILITATION EXPENDITURES** for certified historic structures.
//! - **§ 47(a)(1) 5-Year Ratable Allocation (Post-TCJA 2017)**: the credit is allocated **RATABLY OVER A 5-YEAR PERIOD** on the federal income tax return — **4% OF THE TOTAL CREDIT EACH YEAR**, beginning in the tax year the rehabilitated property is placed in service. **TCJA 2017 transition rule**: for certified historic buildings placed in service under projects with original use begun before 2018, the **20% credit is entirely claimed in the placed-in-service year** (the pre-TCJA timing rule).
//! - **§ 47(c)(1)(A) Qualified Rehabilitated Building Definition**: any building and its structural components that has been **SUBSTANTIALLY REHABILITATED** AND is a **CERTIFIED HISTORIC STRUCTURE**.
//! - **§ 47(c)(1)(A)(iii) Substantial Rehabilitation Test**: a building is considered substantially rehabilitated if, during the **24-MONTH MEASURING PERIOD** (60-month for phased rehab) ending within the taxable year, the **QUALIFIED REHABILITATION EXPENDITURES (QRE)** exceed the **GREATER OF (i) the ADJUSTED BASIS of the building and its structural components, OR (ii) $5,000**.
//! - **§ 47(c)(2) Qualified Rehabilitation Expenditures (QRE)**: capital expenditures for substantial rehabilitation including building costs; **EXCLUDES** acquisition cost, site work, enlargements, costs of installing equipment, certain expenditures unrelated to the qualified rehabilitated building.
//! - **§ 47(c)(3) Certified Historic Structure Definition**: building (i) listed in the **NATIONAL REGISTER OF HISTORIC PLACES**; OR (ii) located in a **REGISTERED HISTORIC DISTRICT** and certified by the National Park Service (NPS) as being of historic significance to the district.
//! - **NPS Three-Part Certification Process**: project owners must complete the **NPS Form 10-168 Historic Preservation Certification Application**: **Part 1 — Evaluation of Significance** (required if not on National Register); **Part 2 — Description of Rehabilitation** (recommended before construction); **Part 3 — Request for Certification of Completed Work**. **ALL PARTS** must be approved by the **STATE HISTORIC PRESERVATION OFFICE (SHPO)** AND then by the **NPS**.
//! - **§ 50(a)(1) 5-Year Recapture Period**: disposition of the property **WITHIN FIVE FULL YEARS** of the date placed in service triggers recapture.
//! - **§ 50(a)(2) Recapture Percentage Ladder**: recapture percentage is **100 PERCENT** for property that ceases to be investment credit property within **one full year** after placed in service; reduced by **20 PERCENTAGE POINTS** for each year held during the 5-year recapture period: **Year 1: 100% recapture**; **Year 2: 80% recapture**; **Year 3: 60% recapture**; **Year 4: 40% recapture**; **Year 5: 20% recapture**; **after Year 5: NO RECAPTURE**.
//! - **Form 3468 (Investment Credit)**: required to claim the § 47 credit; report on Form 3468 Part III, Investment Credit before Adjustments.
//! - **Twinning with § 42 LIHTC**: § 47 HTC is often combined with § 42 LIHTC for affordable housing in historic buildings, providing **DUAL FEDERAL CREDIT STREAMS** (LIHTC 9%/4% PLUS HTC 20%); this "twinning" structure is common in historic preservation affordable housing projects.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const IRC_47_CREDIT_RATE_BPS: u64 = 2_000;
pub const IRC_47_CREDIT_PERIOD_YEARS: u32 = 5;
pub const IRC_47_ANNUAL_ALLOCATION_BPS: u64 = 400;
pub const IRC_47_SUBSTANTIAL_REHAB_DOLLAR_FLOOR: u64 = 5_000;
pub const IRC_47_MEASURING_PERIOD_MONTHS: u32 = 24;
pub const IRC_47_PHASED_REHAB_MEASURING_PERIOD_MONTHS: u32 = 60;
pub const IRC_47_RECAPTURE_PERIOD_YEARS: u32 = 5;
pub const IRC_47_RECAPTURE_YEAR_1_PCT_BPS: u64 = 10_000;
pub const IRC_47_RECAPTURE_YEAR_2_PCT_BPS: u64 = 8_000;
pub const IRC_47_RECAPTURE_YEAR_3_PCT_BPS: u64 = 6_000;
pub const IRC_47_RECAPTURE_YEAR_4_PCT_BPS: u64 = 4_000;
pub const IRC_47_RECAPTURE_YEAR_5_PCT_BPS: u64 = 2_000;
pub const IRC_47_RECAPTURE_REDUCTION_PER_YEAR_BPS: u64 = 2_000;
pub const IRC_47_NPS_FORM_NUMBER: u32 = 10_168;
pub const IRC_47_FORM_NUMBER: u32 = 3_468;
pub const IRC_47_TCJA_ENACTMENT_YEAR: u32 = 2017;
pub const IRC_47_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AllocationTimingRule {
    PostTcja2017FiveYearRatableAllocation,
    PreTcja2017TransitionTwentyPercentInPisYear,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CertifiedHistoricStructureStatus {
    ListedOnNationalRegisterOfHistoricPlaces,
    LocatedInRegisteredHistoricDistrictAndNpsCertifiedAsContributing,
    NotCertifiedHistoricStructure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NpsCertificationStatus {
    AllThreePartsApprovedByShpoAndNps,
    Part1ApprovedPart2OrPart3Pending,
    NoPartsApprovedYet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAspect {
    TwentyPercentCreditRateUnderSection47A1,
    FiveYearRatableAllocationUnderSection47A1,
    SubstantialRehabilitationTestUnderSection47C1AIii,
    QualifiedRehabilitationExpendituresUnderSection47C2,
    CertifiedHistoricStructureDefinitionUnderSection47C3,
    NpsThreePartCertificationProcess,
    FiveYearRecapturePeriodUnderSection50A1,
    RecapturePercentageLadderUnderSection50A2,
    FormFilingUnderForm3468,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section47Mode {
    NotApplicableNotCertifiedHistoricStructure,
    NotApplicableNpsCertificationNotComplete,
    NotApplicableSubstantialRehabilitationTestNotMet,
    CompliantTwentyPercentCreditRate,
    CompliantFiveYearRatableAllocationFourPercentPerYear,
    CompliantPreTcjaTransitionTwentyPercentInPisYear,
    CompliantSubstantialRehabilitationTestMet,
    CompliantQualifiedRehabilitationExpendituresIdentified,
    CompliantCertifiedHistoricStructureListedOnNationalRegister,
    CompliantCertifiedHistoricStructureInHistoricDistrictAndNpsCertified,
    CompliantAllThreePartsApprovedByShpoAndNps,
    CompliantNoRecaptureTriggeredHeldFiveOrMoreYears,
    CompliantForm3468FiledCorrectly,
    ViolationRecaptureTriggeredYear1At100Percent,
    ViolationRecaptureTriggeredYear2At80Percent,
    ViolationRecaptureTriggeredYear3At60Percent,
    ViolationRecaptureTriggeredYear4At40Percent,
    ViolationRecaptureTriggeredYear5At20Percent,
    ViolationForm3468NotFiledOrIncorrect,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub allocation_timing_rule: AllocationTimingRule,
    pub certified_historic_structure_status: CertifiedHistoricStructureStatus,
    pub nps_certification_status: NpsCertificationStatus,
    pub compliance_aspect: ComplianceAspect,
    pub qualified_rehabilitation_expenditures_dollars: u64,
    pub adjusted_basis_dollars: u64,
    pub years_held_since_placed_in_service: u32,
    pub disposition_during_recapture_period: bool,
    pub form_3468_filed_correctly: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section47Mode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
    pub computed_credit_dollars: u64,
}

pub type Section47Input = Input;
pub type Section47Output = Output;
pub type Section47Result = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "IRC § 47 Rehabilitation Tax Credit — commonly referred to as the HISTORIC PRESERVATION TAX CREDIT or HISTORIC TAX CREDIT (HTC); provides a tax incentive to rehabilitate historic buildings".to_string(),
        "IRC § 47(a)(1) 20 % Credit Rate for Certified Historic Structures — tax credit equal to 20 % OF QUALIFIED REHABILITATION EXPENDITURES for certified historic structures".to_string(),
        "IRC § 47(a)(1) 5-Year Ratable Allocation (Post-TCJA 2017) — the credit is allocated RATABLY OVER A 5-YEAR PERIOD on the federal income tax return — 4 % OF THE TOTAL CREDIT EACH YEAR, beginning in the tax year the rehabilitated property is placed in service".to_string(),
        "TCJA 2017 Transition Rule — for certified historic buildings placed in service under projects with original use begun before 2018, the 20 % credit is entirely claimed in the placed-in-service year (the pre-TCJA timing rule)".to_string(),
        "IRC § 47(c)(1)(A) Qualified Rehabilitated Building Definition — any building and its structural components that has been SUBSTANTIALLY REHABILITATED AND is a CERTIFIED HISTORIC STRUCTURE".to_string(),
        "IRC § 47(c)(1)(A)(iii) Substantial Rehabilitation Test — a building is considered substantially rehabilitated if, during the 24-MONTH MEASURING PERIOD (60-month for phased rehab) ending within the taxable year, the QUALIFIED REHABILITATION EXPENDITURES (QRE) exceed the GREATER OF (i) the ADJUSTED BASIS of the building and its structural components, OR (ii) $5,000".to_string(),
        "IRC § 47(c)(2) Qualified Rehabilitation Expenditures (QRE) — capital expenditures for substantial rehabilitation including building costs; EXCLUDES acquisition cost, site work, enlargements, costs of installing equipment, certain expenditures unrelated to the qualified rehabilitated building".to_string(),
        "IRC § 47(c)(3) Certified Historic Structure Definition — building (i) listed in the NATIONAL REGISTER OF HISTORIC PLACES; OR (ii) located in a REGISTERED HISTORIC DISTRICT and certified by the National Park Service (NPS) as being of historic significance to the district".to_string(),
        "NPS Three-Part Certification Process — project owners must complete the NPS Form 10-168 Historic Preservation Certification Application: Part 1 — Evaluation of Significance (required if not on National Register); Part 2 — Description of Rehabilitation (recommended before construction); Part 3 — Request for Certification of Completed Work. ALL PARTS must be approved by the STATE HISTORIC PRESERVATION OFFICE (SHPO) AND then by the NPS".to_string(),
        "IRC § 50(a)(1) 5-Year Recapture Period — disposition of the property WITHIN FIVE FULL YEARS of the date placed in service triggers recapture".to_string(),
        "IRC § 50(a)(2) Recapture Percentage Ladder — recapture percentage is 100 PERCENT for property that ceases to be investment credit property within one full year after placed in service; reduced by 20 PERCENTAGE POINTS for each year held during the 5-year recapture period: Year 1: 100 % recapture; Year 2: 80 % recapture; Year 3: 60 % recapture; Year 4: 40 % recapture; Year 5: 20 % recapture; after Year 5: NO RECAPTURE".to_string(),
        "Form 3468 (Investment Credit) — required to claim the § 47 credit; report on Form 3468 Part III, Investment Credit before Adjustments".to_string(),
        "Twinning with § 42 LIHTC — § 47 HTC is often combined with § 42 LIHTC for affordable housing in historic buildings, providing DUAL FEDERAL CREDIT STREAMS (LIHTC 9 %/4 % PLUS HTC 20 %); this twinning structure is common in historic preservation affordable housing projects".to_string(),
        "Cornell LII + IRS + Tax Notes + Bloomberg Tax + House USC + Accounting Insights + The Tax Adviser + The Sherbert Group + Novogradac + EisnerAmper + CSG Law + AndreTaxCo + The Tax Credit Exchange — practitioner overviews of § 47".to_string(),
    ];

    if input.certified_historic_structure_status
        == CertifiedHistoricStructureStatus::NotCertifiedHistoricStructure
    {
        return Output {
            mode: Section47Mode::NotApplicableNotCertifiedHistoricStructure,
            statutory_basis: "IRC § 47(c)(3) — building does not qualify as certified historic structure".to_string(),
            notes: "NOT APPLICABLE: building does not qualify as certified historic structure under § 47(c)(3); not listed on National Register and not in registered historic district with NPS certification.".to_string(),
            citations,
            computed_credit_dollars: 0,
        };
    }

    if input.nps_certification_status != NpsCertificationStatus::AllThreePartsApprovedByShpoAndNps {
        return Output {
            mode: Section47Mode::NotApplicableNpsCertificationNotComplete,
            statutory_basis: "NPS three-part certification — all three parts must be approved by SHPO and NPS".to_string(),
            notes: "NOT APPLICABLE: NPS three-part certification (Part 1 / Part 2 / Part 3) not fully approved by State Historic Preservation Office (SHPO) and NPS; § 47 credit unavailable until certification complete.".to_string(),
            citations,
            computed_credit_dollars: 0,
        };
    }

    let substantial_rehab_threshold = input
        .adjusted_basis_dollars
        .max(IRC_47_SUBSTANTIAL_REHAB_DOLLAR_FLOOR);
    if input.qualified_rehabilitation_expenditures_dollars <= substantial_rehab_threshold {
        return Output {
            mode: Section47Mode::NotApplicableSubstantialRehabilitationTestNotMet,
            statutory_basis: "IRC § 47(c)(1)(A)(iii) — substantial rehabilitation test not met; QRE must exceed greater of adjusted basis or $5,000".to_string(),
            notes: format!(
                "NOT APPLICABLE: substantial rehabilitation test not met under § 47(c)(1)(A)(iii); QRE of ${qre} does not exceed greater of adjusted basis ${ab} or $5,000 ({threshold} threshold).",
                qre = input.qualified_rehabilitation_expenditures_dollars,
                ab = input.adjusted_basis_dollars,
                threshold = substantial_rehab_threshold,
            ),
            citations,
            computed_credit_dollars: 0,
        };
    }

    match input.compliance_aspect {
        ComplianceAspect::TwentyPercentCreditRateUnderSection47A1 => {
            let total_credit = (u128::from(input.qualified_rehabilitation_expenditures_dollars)
                * u128::from(IRC_47_CREDIT_RATE_BPS)
                / u128::from(IRC_47_BASIS_POINT_DENOMINATOR))
                as u64;
            Output {
                mode: Section47Mode::CompliantTwentyPercentCreditRate,
                statutory_basis: "IRC § 47(a)(1) — 20 % credit rate for certified historic structures".to_string(),
                notes: format!(
                    "COMPLIANT: 20 % credit rate × ${qre} QRE = ${total} total credit (allocated ratably over 5 years post-TCJA 2017).",
                    qre = input.qualified_rehabilitation_expenditures_dollars,
                    total = total_credit,
                ),
                citations,
                computed_credit_dollars: total_credit,
            }
        }
        ComplianceAspect::FiveYearRatableAllocationUnderSection47A1 => {
            let total_credit = (u128::from(input.qualified_rehabilitation_expenditures_dollars)
                * u128::from(IRC_47_CREDIT_RATE_BPS)
                / u128::from(IRC_47_BASIS_POINT_DENOMINATOR))
                as u64;
            match input.allocation_timing_rule {
                AllocationTimingRule::PostTcja2017FiveYearRatableAllocation => {
                    let annual_credit = total_credit / 5;
                    Output {
                        mode: Section47Mode::CompliantFiveYearRatableAllocationFourPercentPerYear,
                        statutory_basis: "IRC § 47(a)(1) — 5-year ratable allocation; 4 % of QRE per year for 5 years (post-TCJA 2017)".to_string(),
                        notes: format!(
                            "COMPLIANT: 5-year ratable allocation under § 47(a)(1) (post-TCJA 2017); total credit ${total} allocated as ${annual}/year for 5 years (4 % of QRE × ${qre} per year).",
                            total = total_credit,
                            annual = annual_credit,
                            qre = input.qualified_rehabilitation_expenditures_dollars,
                        ),
                        citations,
                        computed_credit_dollars: annual_credit,
                    }
                }
                AllocationTimingRule::PreTcja2017TransitionTwentyPercentInPisYear => Output {
                    mode: Section47Mode::CompliantPreTcjaTransitionTwentyPercentInPisYear,
                    statutory_basis: "TCJA 2017 transition rule — 20 % credit entirely claimed in placed-in-service year for pre-2018 projects".to_string(),
                    notes: format!(
                        "COMPLIANT: TCJA 2017 transition rule applies — 20 % credit entirely claimed in placed-in-service year for pre-2018 projects; ${total} total credit claimed in PIS year.",
                        total = total_credit,
                    ),
                    citations,
                    computed_credit_dollars: total_credit,
                },
            }
        }
        ComplianceAspect::SubstantialRehabilitationTestUnderSection47C1AIii => Output {
            mode: Section47Mode::CompliantSubstantialRehabilitationTestMet,
            statutory_basis: "IRC § 47(c)(1)(A)(iii) — substantial rehabilitation test met".to_string(),
            notes: format!(
                "COMPLIANT: QRE of ${qre} exceeds greater of adjusted basis ${ab} or $5,000 (${threshold}); 24-month measuring period satisfied (60-month for phased rehab) under § 47(c)(1)(A)(iii).",
                qre = input.qualified_rehabilitation_expenditures_dollars,
                ab = input.adjusted_basis_dollars,
                threshold = substantial_rehab_threshold,
            ),
            citations,
            computed_credit_dollars: 0,
        },
        ComplianceAspect::QualifiedRehabilitationExpendituresUnderSection47C2 => Output {
            mode: Section47Mode::CompliantQualifiedRehabilitationExpendituresIdentified,
            statutory_basis: "IRC § 47(c)(2) — qualified rehabilitation expenditures identified".to_string(),
            notes: format!(
                "COMPLIANT: ${qre} in qualified rehabilitation expenditures identified under § 47(c)(2); excludes acquisition cost, site work, enlargements, equipment installation, and certain unrelated expenditures.",
                qre = input.qualified_rehabilitation_expenditures_dollars,
            ),
            citations,
            computed_credit_dollars: 0,
        },
        ComplianceAspect::CertifiedHistoricStructureDefinitionUnderSection47C3 => match input
            .certified_historic_structure_status
        {
            CertifiedHistoricStructureStatus::ListedOnNationalRegisterOfHistoricPlaces => Output {
                mode: Section47Mode::CompliantCertifiedHistoricStructureListedOnNationalRegister,
                statutory_basis: "IRC § 47(c)(3)(A)(i) — building listed on National Register of Historic Places".to_string(),
                notes: "COMPLIANT: building listed on National Register of Historic Places under § 47(c)(3)(A)(i); qualifies as certified historic structure.".to_string(),
                citations,
                computed_credit_dollars: 0,
            },
            CertifiedHistoricStructureStatus::LocatedInRegisteredHistoricDistrictAndNpsCertifiedAsContributing => Output {
                mode: Section47Mode::CompliantCertifiedHistoricStructureInHistoricDistrictAndNpsCertified,
                statutory_basis: "IRC § 47(c)(3)(A)(ii) — building located in registered historic district and NPS-certified as historically significant".to_string(),
                notes: "COMPLIANT: building located in registered historic district and certified by NPS as being of historic significance to the district under § 47(c)(3)(A)(ii); qualifies as certified historic structure.".to_string(),
                citations,
                computed_credit_dollars: 0,
            },
            CertifiedHistoricStructureStatus::NotCertifiedHistoricStructure => unreachable!(),
        },
        ComplianceAspect::NpsThreePartCertificationProcess => Output {
            mode: Section47Mode::CompliantAllThreePartsApprovedByShpoAndNps,
            statutory_basis: "NPS Form 10-168 — Part 1 / Part 2 / Part 3 all approved by SHPO and NPS".to_string(),
            notes: "COMPLIANT: NPS Form 10-168 Part 1 (Evaluation of Significance) + Part 2 (Description of Rehabilitation) + Part 3 (Request for Certification of Completed Work) all approved by State Historic Preservation Office (SHPO) and National Park Service (NPS).".to_string(),
            citations,
            computed_credit_dollars: 0,
        },
        ComplianceAspect::FiveYearRecapturePeriodUnderSection50A1 => {
            if input.years_held_since_placed_in_service >= IRC_47_RECAPTURE_PERIOD_YEARS {
                Output {
                    mode: Section47Mode::CompliantNoRecaptureTriggeredHeldFiveOrMoreYears,
                    statutory_basis: "IRC § 50(a)(1) — held 5+ years; no recapture triggered".to_string(),
                    notes: format!(
                        "COMPLIANT: property held {y} years (≥ 5-year recapture period); no recapture triggered under § 50(a)(1).",
                        y = input.years_held_since_placed_in_service,
                    ),
                    citations,
                    computed_credit_dollars: 0,
                }
            } else if input.disposition_during_recapture_period {
                let mode = match input.years_held_since_placed_in_service {
                    0 => Section47Mode::ViolationRecaptureTriggeredYear1At100Percent,
                    1 => Section47Mode::ViolationRecaptureTriggeredYear2At80Percent,
                    2 => Section47Mode::ViolationRecaptureTriggeredYear3At60Percent,
                    3 => Section47Mode::ViolationRecaptureTriggeredYear4At40Percent,
                    _ => Section47Mode::ViolationRecaptureTriggeredYear5At20Percent,
                };
                Output {
                    mode,
                    statutory_basis: "IRC § 50(a)(1) — disposition within 5-year recapture period triggers recapture".to_string(),
                    notes: format!(
                        "VIOLATION: disposition at year {y} during 5-year recapture period triggers recapture under § 50(a)(1).",
                        y = input.years_held_since_placed_in_service,
                    ),
                    citations,
                    computed_credit_dollars: 0,
                }
            } else {
                Output {
                    mode: Section47Mode::CompliantNoRecaptureTriggeredHeldFiveOrMoreYears,
                    statutory_basis: "IRC § 50(a)(1) — within recapture period; no disposition".to_string(),
                    notes: format!(
                        "COMPLIANT: within 5-year recapture period at year {y}; no disposition; no recapture triggered.",
                        y = input.years_held_since_placed_in_service,
                    ),
                    citations,
                    computed_credit_dollars: 0,
                }
            }
        }
        ComplianceAspect::RecapturePercentageLadderUnderSection50A2 => {
            let recapture_pct_bps = match input.years_held_since_placed_in_service {
                0 => IRC_47_RECAPTURE_YEAR_1_PCT_BPS,
                1 => IRC_47_RECAPTURE_YEAR_2_PCT_BPS,
                2 => IRC_47_RECAPTURE_YEAR_3_PCT_BPS,
                3 => IRC_47_RECAPTURE_YEAR_4_PCT_BPS,
                4 => IRC_47_RECAPTURE_YEAR_5_PCT_BPS,
                _ => 0,
            };
            let mode = match input.years_held_since_placed_in_service {
                0 => Section47Mode::ViolationRecaptureTriggeredYear1At100Percent,
                1 => Section47Mode::ViolationRecaptureTriggeredYear2At80Percent,
                2 => Section47Mode::ViolationRecaptureTriggeredYear3At60Percent,
                3 => Section47Mode::ViolationRecaptureTriggeredYear4At40Percent,
                4 => Section47Mode::ViolationRecaptureTriggeredYear5At20Percent,
                _ => Section47Mode::CompliantNoRecaptureTriggeredHeldFiveOrMoreYears,
            };
            Output {
                mode,
                statutory_basis: "IRC § 50(a)(2) — recapture percentage ladder".to_string(),
                notes: format!(
                    "Recapture percentage at year {y}: {pct_bps} bps under § 50(a)(2) sliding ladder.",
                    y = input.years_held_since_placed_in_service,
                    pct_bps = recapture_pct_bps,
                ),
                citations,
                computed_credit_dollars: 0,
            }
        }
        ComplianceAspect::FormFilingUnderForm3468 => {
            if input.form_3468_filed_correctly {
                Output {
                    mode: Section47Mode::CompliantForm3468FiledCorrectly,
                    statutory_basis: "Form 3468 — Investment Credit form required to claim § 47 credit".to_string(),
                    notes: "COMPLIANT: Form 3468 filed correctly to claim § 47 credit; reported on Form 3468 Part III (Investment Credit before Adjustments).".to_string(),
                    citations,
                    computed_credit_dollars: 0,
                }
            } else {
                Output {
                    mode: Section47Mode::ViolationForm3468NotFiledOrIncorrect,
                    statutory_basis: "Form 3468 filing required to claim § 47 credit".to_string(),
                    notes: "VIOLATION: Form 3468 not filed or incorrectly filed; § 47 credit may be disallowed.".to_string(),
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
            allocation_timing_rule: AllocationTimingRule::PostTcja2017FiveYearRatableAllocation,
            certified_historic_structure_status:
                CertifiedHistoricStructureStatus::ListedOnNationalRegisterOfHistoricPlaces,
            nps_certification_status: NpsCertificationStatus::AllThreePartsApprovedByShpoAndNps,
            compliance_aspect: ComplianceAspect::TwentyPercentCreditRateUnderSection47A1,
            qualified_rehabilitation_expenditures_dollars: 1_000_000,
            adjusted_basis_dollars: 500_000,
            years_held_since_placed_in_service: 6,
            disposition_during_recapture_period: false,
            form_3468_filed_correctly: true,
        }
    }

    #[test]
    fn not_certified_historic_structure_not_applicable() {
        let mut input = baseline_input();
        input.certified_historic_structure_status =
            CertifiedHistoricStructureStatus::NotCertifiedHistoricStructure;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section47Mode::NotApplicableNotCertifiedHistoricStructure
        );
    }

    #[test]
    fn nps_certification_not_complete_not_applicable() {
        let mut input = baseline_input();
        input.nps_certification_status = NpsCertificationStatus::Part1ApprovedPart2OrPart3Pending;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section47Mode::NotApplicableNpsCertificationNotComplete
        );
    }

    #[test]
    fn substantial_rehabilitation_test_not_met_not_applicable() {
        let mut input = baseline_input();
        input.qualified_rehabilitation_expenditures_dollars = 400_000;
        input.adjusted_basis_dollars = 500_000;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section47Mode::NotApplicableSubstantialRehabilitationTestNotMet
        );
    }

    #[test]
    fn twenty_percent_credit_rate_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::TwentyPercentCreditRateUnderSection47A1;
        input.qualified_rehabilitation_expenditures_dollars = 1_000_000;
        let out = check(&input);
        assert_eq!(out.mode, Section47Mode::CompliantTwentyPercentCreditRate);
        assert_eq!(out.computed_credit_dollars, 200_000);
    }

    #[test]
    fn five_year_ratable_allocation_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::FiveYearRatableAllocationUnderSection47A1;
        input.allocation_timing_rule =
            AllocationTimingRule::PostTcja2017FiveYearRatableAllocation;
        input.qualified_rehabilitation_expenditures_dollars = 1_000_000;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section47Mode::CompliantFiveYearRatableAllocationFourPercentPerYear
        );
        assert_eq!(out.computed_credit_dollars, 40_000);
    }

    #[test]
    fn pre_tcja_transition_twenty_percent_in_pis_year_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::FiveYearRatableAllocationUnderSection47A1;
        input.allocation_timing_rule =
            AllocationTimingRule::PreTcja2017TransitionTwentyPercentInPisYear;
        input.qualified_rehabilitation_expenditures_dollars = 1_000_000;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section47Mode::CompliantPreTcjaTransitionTwentyPercentInPisYear
        );
        assert_eq!(out.computed_credit_dollars, 200_000);
    }

    #[test]
    fn substantial_rehabilitation_test_met_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SubstantialRehabilitationTestUnderSection47C1AIii;
        input.qualified_rehabilitation_expenditures_dollars = 600_000;
        input.adjusted_basis_dollars = 500_000;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section47Mode::CompliantSubstantialRehabilitationTestMet
        );
    }

    #[test]
    fn substantial_rehabilitation_at_dollar_floor_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SubstantialRehabilitationTestUnderSection47C1AIii;
        input.qualified_rehabilitation_expenditures_dollars = 5_001;
        input.adjusted_basis_dollars = 0;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section47Mode::CompliantSubstantialRehabilitationTestMet
        );
    }

    #[test]
    fn substantial_rehabilitation_below_5000_floor_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SubstantialRehabilitationTestUnderSection47C1AIii;
        input.qualified_rehabilitation_expenditures_dollars = 5_000;
        input.adjusted_basis_dollars = 0;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section47Mode::NotApplicableSubstantialRehabilitationTestNotMet
        );
    }

    #[test]
    fn qualified_rehabilitation_expenditures_identified_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::QualifiedRehabilitationExpendituresUnderSection47C2;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section47Mode::CompliantQualifiedRehabilitationExpendituresIdentified
        );
    }

    #[test]
    fn certified_historic_structure_national_register_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::CertifiedHistoricStructureDefinitionUnderSection47C3;
        input.certified_historic_structure_status =
            CertifiedHistoricStructureStatus::ListedOnNationalRegisterOfHistoricPlaces;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section47Mode::CompliantCertifiedHistoricStructureListedOnNationalRegister
        );
    }

    #[test]
    fn certified_historic_structure_district_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::CertifiedHistoricStructureDefinitionUnderSection47C3;
        input.certified_historic_structure_status =
            CertifiedHistoricStructureStatus::LocatedInRegisteredHistoricDistrictAndNpsCertifiedAsContributing;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section47Mode::CompliantCertifiedHistoricStructureInHistoricDistrictAndNpsCertified
        );
    }

    #[test]
    fn nps_three_part_certification_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::NpsThreePartCertificationProcess;
        let out = check(&input);
        assert_eq!(out.mode, Section47Mode::CompliantAllThreePartsApprovedByShpoAndNps);
    }

    #[test]
    fn no_recapture_triggered_held_5_or_more_years_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::FiveYearRecapturePeriodUnderSection50A1;
        input.years_held_since_placed_in_service = 5;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section47Mode::CompliantNoRecaptureTriggeredHeldFiveOrMoreYears
        );
    }

    #[test]
    fn recapture_year_1_at_100_percent_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::FiveYearRecapturePeriodUnderSection50A1;
        input.years_held_since_placed_in_service = 0;
        input.disposition_during_recapture_period = true;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section47Mode::ViolationRecaptureTriggeredYear1At100Percent
        );
    }

    #[test]
    fn recapture_year_2_at_80_percent_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::FiveYearRecapturePeriodUnderSection50A1;
        input.years_held_since_placed_in_service = 1;
        input.disposition_during_recapture_period = true;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section47Mode::ViolationRecaptureTriggeredYear2At80Percent
        );
    }

    #[test]
    fn recapture_year_3_at_60_percent_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::FiveYearRecapturePeriodUnderSection50A1;
        input.years_held_since_placed_in_service = 2;
        input.disposition_during_recapture_period = true;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section47Mode::ViolationRecaptureTriggeredYear3At60Percent
        );
    }

    #[test]
    fn recapture_year_4_at_40_percent_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::FiveYearRecapturePeriodUnderSection50A1;
        input.years_held_since_placed_in_service = 3;
        input.disposition_during_recapture_period = true;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section47Mode::ViolationRecaptureTriggeredYear4At40Percent
        );
    }

    #[test]
    fn recapture_year_5_at_20_percent_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::FiveYearRecapturePeriodUnderSection50A1;
        input.years_held_since_placed_in_service = 4;
        input.disposition_during_recapture_period = true;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section47Mode::ViolationRecaptureTriggeredYear5At20Percent
        );
    }

    #[test]
    fn recapture_percentage_ladder_year_1_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::RecapturePercentageLadderUnderSection50A2;
        input.years_held_since_placed_in_service = 0;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section47Mode::ViolationRecaptureTriggeredYear1At100Percent
        );
    }

    #[test]
    fn form_3468_filed_correctly_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::FormFilingUnderForm3468;
        input.form_3468_filed_correctly = true;
        let out = check(&input);
        assert_eq!(out.mode, Section47Mode::CompliantForm3468FiledCorrectly);
    }

    #[test]
    fn form_3468_not_filed_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::FormFilingUnderForm3468;
        input.form_3468_filed_correctly = false;
        let out = check(&input);
        assert_eq!(out.mode, Section47Mode::ViolationForm3468NotFiledOrIncorrect);
    }

    #[test]
    fn constants_pin_section_47_htc_structure() {
        assert_eq!(IRC_47_CREDIT_RATE_BPS, 2_000);
        assert_eq!(IRC_47_CREDIT_PERIOD_YEARS, 5);
        assert_eq!(IRC_47_ANNUAL_ALLOCATION_BPS, 400);
        assert_eq!(IRC_47_SUBSTANTIAL_REHAB_DOLLAR_FLOOR, 5_000);
        assert_eq!(IRC_47_MEASURING_PERIOD_MONTHS, 24);
        assert_eq!(IRC_47_PHASED_REHAB_MEASURING_PERIOD_MONTHS, 60);
        assert_eq!(IRC_47_RECAPTURE_PERIOD_YEARS, 5);
        assert_eq!(IRC_47_RECAPTURE_YEAR_1_PCT_BPS, 10_000);
        assert_eq!(IRC_47_RECAPTURE_YEAR_2_PCT_BPS, 8_000);
        assert_eq!(IRC_47_RECAPTURE_YEAR_3_PCT_BPS, 6_000);
        assert_eq!(IRC_47_RECAPTURE_YEAR_4_PCT_BPS, 4_000);
        assert_eq!(IRC_47_RECAPTURE_YEAR_5_PCT_BPS, 2_000);
        assert_eq!(IRC_47_RECAPTURE_REDUCTION_PER_YEAR_BPS, 2_000);
        assert_eq!(IRC_47_NPS_FORM_NUMBER, 10_168);
        assert_eq!(IRC_47_FORM_NUMBER, 3_468);
        assert_eq!(IRC_47_TCJA_ENACTMENT_YEAR, 2017);
        assert_eq!(IRC_47_BASIS_POINT_DENOMINATOR, 10_000);
    }

    #[test]
    fn citations_pin_section_47_htc_structure() {
        let input = baseline_input();
        let out = check(&input);
        let joined = out.citations.join(" || ");
        assert!(joined.contains("IRC § 47 Rehabilitation Tax Credit"));
        assert!(joined.contains("HISTORIC PRESERVATION TAX CREDIT"));
        assert!(joined.contains("HISTORIC TAX CREDIT (HTC)"));
        assert!(joined.contains("20 % OF QUALIFIED REHABILITATION EXPENDITURES"));
        assert!(joined.contains("5-YEAR PERIOD"));
        assert!(joined.contains("4 % OF THE TOTAL CREDIT EACH YEAR"));
        assert!(joined.contains("TCJA 2017"));
        assert!(joined.contains("SUBSTANTIALLY REHABILITATED"));
        assert!(joined.contains("CERTIFIED HISTORIC STRUCTURE"));
        assert!(joined.contains("24-MONTH MEASURING PERIOD"));
        assert!(joined.contains("ADJUSTED BASIS"));
        assert!(joined.contains("$5,000"));
        assert!(joined.contains("QUALIFIED REHABILITATION EXPENDITURES (QRE)"));
        assert!(joined.contains("NATIONAL REGISTER OF HISTORIC PLACES"));
        assert!(joined.contains("REGISTERED HISTORIC DISTRICT"));
        assert!(joined.contains("NPS Form 10-168"));
        assert!(joined.contains("Part 1"));
        assert!(joined.contains("Part 2"));
        assert!(joined.contains("Part 3"));
        assert!(joined.contains("STATE HISTORIC PRESERVATION OFFICE (SHPO)"));
        assert!(joined.contains("FIVE FULL YEARS"));
        assert!(joined.contains("100 PERCENT"));
        assert!(joined.contains("20 PERCENTAGE POINTS"));
        assert!(joined.contains("Form 3468"));
        assert!(joined.contains("Twinning with § 42 LIHTC"));
    }
}
