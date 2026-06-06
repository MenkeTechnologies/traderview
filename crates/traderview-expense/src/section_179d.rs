//! IRC § 179D Energy Efficient Commercial Buildings
//! Deduction Compliance Module — pure-compute check for
//! taxpayer eligibility for the energy-efficient commercial
//! buildings deduction enacted by the Energy Policy Act of
//! 2005, dramatically expanded by the Inflation Reduction
//! Act of 2022, and TERMINATED by the One Big Beautiful
//! Bill Act of 2025 for buildings whose construction begins
//! after June 30, 2026.
//!
//! Originally enacted by **Section 1331 of the Energy Policy
//! Act of 2005 (Public Law 109-58)**, signed by President
//! George W. Bush on **August 8, 2005**, with effective date
//! for property placed in service after December 31, 2005.
//! Substantially amended by **Section 13303 of the Inflation
//! Reduction Act of 2022 (Public Law 117-169)**, signed by
//! President Joe Biden on August 16, 2022, with effective
//! date for tax years beginning after December 31, 2022.
//!
//! **TERMINATED by the One Big Beautiful Bill Act of 2025
//! (Public Law 119-21)**, signed by President Donald Trump
//! on **July 4, 2025**. § 179D deduction available ONLY for
//! buildings whose construction begins on or before **JUNE
//! 30, 2026**; placed-in-service date can occur later.
//!
//! Trader / business-property critical for any taxpayer
//! placing into service or designing energy-efficient
//! commercial buildings between August 8, 2005 and the
//! construction-begin OBBBA cutoff of June 30, 2026.
//!
//! Web research (verified 2026-06-03):
//! - **EPAct 2005 Enactment**: IRC § 179D added by **Section 1331 of the Energy Policy Act of 2005 (Public Law 109-58, 119 Stat. 594)**, signed by President George W. Bush on **August 8, 2005**; effective for property placed in service after **December 31, 2005** ([IRS — Energy Efficient Commercial Buildings Deduction](https://www.irs.gov/credits-deductions/energy-efficient-commercial-buildings-deduction); [IRS — IRC 179D Energy Efficient Commercial Buildings Deduction Practice Unit PDF](https://www.irs.gov/pub/fatca/int_practice_units/irc179d-energy-efficient.pdf); [House Office of Law Revision Counsel — 26 USC § 179D](https://uscode.house.gov/view.xhtml?req=(title:26+section:179D+edition:prelim)); [Cornell LII — 26 U.S. Code § 179D](https://www.law.cornell.edu/uscode/text/26/179D); [Bloomberg Tax — Sec. 179D](https://irc.bloombergtax.com/public/uscode/doc/irc/section_179d); [Energy Department — 179D Energy Efficient Commercial Buildings Tax Deduction](https://www.energy.gov/cmei/buildings/179d-energy-efficient-commercial-buildings-tax-deduction); [Every CRS Report — IF12862 The Section 179D Energy Efficient Commercial Buildings Deduction](https://www.everycrsreport.com/reports/IF12862.html); [CLA — Energy-Efficient Design: The Section 179D Deduction](https://www.claconnect.com/en/resources/blogs/real-estate/the-value-of-energy-efficient-design-section-179d-deduction); [Cherry Bekaert — Section 179D Tax Deduction for Energy-Efficient Buildings FAQ](https://www.cbh.com/insights/articles/section-179d-faq-tax-savings-for-energy-efficient-buildings/)).
//! - **IRA 2022 Expansion**: substantially amended by **Section 13303 of the Inflation Reduction Act of 2022 (Public Law 117-169, 136 Stat. 1818)**, signed by President Joe Biden on **August 16, 2022**; effective for tax years beginning after **December 31, 2022** ([Inflation Reduction Act Tracker — IRA Section 13303 Energy Efficient Commercial Buildings Deduction](https://iratracker.org/programs/ira-section-13303-energy-efficient-commercial-buildings-deduction/)).
//! - **§ 179D(c)(1) Energy Reduction Threshold**: building must achieve at least **25 %** annual energy and power cost reduction (reduced from prior 50 % threshold by IRA 2022).
//! - **§ 179D(b)(1) Base Deduction Amount**: base deduction starts at **$0.50 per square foot** for 25 % energy reduction; **increases by $0.02 per percentage point** of additional energy reduction; capped at **$1.00 per square foot**.
//! - **§ 179D(b)(3) Bonus Deduction with Prevailing Wage and Apprenticeship**: if **PREVAILING WAGE AND APPRENTICESHIP** requirements met, base amount increases to **$2.50 per square foot** for 25 % energy reduction; **increases by $0.10 per percentage point** of additional energy reduction; capped at **$5.00 per square foot**.
//! - **§ 179D Inflation Adjustment**: deduction amounts are inflation-adjusted annually; for tax year **2025**, maximum bonus deduction = **$5.81 per square foot** (up from $1.88/sq ft pre-IRA), reflecting cumulative inflation adjustment under § 179D(g).
//! - **§ 179D(c)(1) Eligible Building Systems**: three categories of energy-efficient commercial building property (EECBP) eligible — **(i) INTERIOR LIGHTING SYSTEMS**; **(ii) HEATING, VENTILATING, AND AIR CONDITIONING (HVAC) AND HOT WATER SYSTEMS**; **(iii) BUILDING ENVELOPE**.
//! - **ASHRAE 90.1 Reference Standard**: energy reduction must be measured against the applicable ASHRAE Standard 90.1 reference building; specific version applicable depends on placed-in-service date (90.1-2007 / 90.1-2013 / 90.1-2019 / 90.1-2022).
//! - **§ 179D(d)(2) Government / Tax-Exempt Designer Allocation**: building owners that are federal, state, or local government entities, Indian tribal governments, Alaska Native Corporations, or (post-IRA 2022) tax-exempt organizations may **ALLOCATE THE DEDUCTION TO THE PRIMARY DESIGNER** (architect / engineer / general contractor); this is the most-litigated practitioner trap because the allocation must be properly documented.
//! - **OBBBA 2025 Termination**: § 179D ELIMINATED for buildings whose construction begins after **JUNE 30, 2026** by **Public Law 119-21 (One, Big, Beautiful Bill Act of 2025, 139 Stat. 72)**, signed by President Donald Trump on **JULY 4, 2025**; projects must begin construction on or before June 30, 2026 to qualify; placed-in-service date can occur later ([CSSI Services — Take Advantage of Section 179D Before It's Gone: OBBBA Repeal Date Confirmed for 2026](https://cssiservices.com/section-179d-obbba-repeal/); [Arnold & Porter — From IRA to OBBBA: A New Era for Clean Energy Tax Credits](https://www.arnoldporter.com/en/perspectives/advisories/2025/07/from-ira-to-obbba-a-new-era-for-clean-energy-tax-credits); [IRS — FAQs for Modification of Sections 25C, 25D, 25E, 30C, 30D, 45L, 45W, AND 179D under Public Law 119-21](https://www.irs.gov/newsroom/faqs-for-modification-of-sections-25c-25d-25e-30c-30d-45l-45w-and-179d-under-public-law-119-21-139-stat-72-july-4-2025-commonly-known-as-the-one-big-beautiful-bill-obbb); [Grant Thornton — Energy Incentives Under OBBBA](https://www.grantthornton.com/insights/alerts/tax/2025/insights/energy-incentives-under-obbba-what-you-need-to-know); [MGO CPA — How OBBBA Impacts Your 179D Energy-Efficient Deduction](https://www.mgocpa.com/perspective/179d-energy-efficient-building-deduction-obbba-impact/); [Larsen Co — Energy-Efficient Building Deduction Set to Expire in 2026](https://larsco.com/blog/energy-efficient-building-deduction-set-to-expire-in-2026); [Concord LP — Maximize 179D Savings Before OBBBA Deadline](https://www.concordlp.com/news/maximize-179d-tax-deductions-before-obbba-ends-energy-savings); [Whipple Wood CPAs — How OBBBA Impacts Construction Businesses](https://whipplewoodcpas.com/how-the-one-big-beautiful-bill-impacts-construction-businesses/); [CSSI — Section 179D for Medical Property Owners and Developers](https://cssiservices.com/179d-for-medical-properties); [Grant Thornton 2026 Business Tax Planning Guide](https://www.grantthornton.com/insights/alerts/tax/2025/legislative-updates/2026-business-tax-planning-guide)).

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const IRC_179D_EPACT_2005_ENACTMENT_DATE_YEAR: u32 = 2005;
pub const IRC_179D_EPACT_2005_ENACTMENT_DATE_MONTH: u32 = 8;
pub const IRC_179D_EPACT_2005_ENACTMENT_DATE_DAY: u32 = 8;
pub const IRC_179D_EPACT_2005_PUBLIC_LAW_CONGRESS: u32 = 109;
pub const IRC_179D_EPACT_2005_PUBLIC_LAW_ENACTMENT: u32 = 58;
pub const IRC_179D_EPACT_2005_ENABLING_SECTION: u32 = 1331;
pub const IRC_179D_EPACT_2005_STAT_VOLUME: u32 = 119;
pub const IRC_179D_EPACT_2005_STAT_PAGE: u32 = 594;
pub const IRC_179D_EPACT_2005_EFFECTIVE_DATE_YEAR: u32 = 2005;
pub const IRC_179D_EPACT_2005_EFFECTIVE_DATE_MONTH: u32 = 12;
pub const IRC_179D_EPACT_2005_EFFECTIVE_DATE_DAY: u32 = 31;
pub const IRC_179D_IRA_2022_ENACTMENT_DATE_YEAR: u32 = 2022;
pub const IRC_179D_IRA_2022_ENACTMENT_DATE_MONTH: u32 = 8;
pub const IRC_179D_IRA_2022_ENACTMENT_DATE_DAY: u32 = 16;
pub const IRC_179D_IRA_2022_PUBLIC_LAW_CONGRESS: u32 = 117;
pub const IRC_179D_IRA_2022_PUBLIC_LAW_ENACTMENT: u32 = 169;
pub const IRC_179D_IRA_2022_ENABLING_SECTION: u32 = 13303;
pub const IRC_179D_IRA_2022_STAT_VOLUME: u32 = 136;
pub const IRC_179D_IRA_2022_STAT_PAGE: u32 = 1818;
pub const IRC_179D_OBBBA_ENACTMENT_DATE_YEAR: u32 = 2025;
pub const IRC_179D_OBBBA_ENACTMENT_DATE_MONTH: u32 = 7;
pub const IRC_179D_OBBBA_ENACTMENT_DATE_DAY: u32 = 4;
pub const IRC_179D_OBBBA_PUBLIC_LAW_CONGRESS: u32 = 119;
pub const IRC_179D_OBBBA_PUBLIC_LAW_ENACTMENT: u32 = 21;
pub const IRC_179D_OBBBA_STAT_VOLUME: u32 = 139;
pub const IRC_179D_OBBBA_STAT_PAGE: u32 = 72;
pub const IRC_179D_OBBBA_TERMINATION_CONSTRUCTION_BEGIN_DATE_YEAR: u32 = 2026;
pub const IRC_179D_OBBBA_TERMINATION_CONSTRUCTION_BEGIN_DATE_MONTH: u32 = 6;
pub const IRC_179D_OBBBA_TERMINATION_CONSTRUCTION_BEGIN_DATE_DAY: u32 = 30;
pub const IRC_179D_ENERGY_REDUCTION_THRESHOLD_BPS: u64 = 2_500;
pub const IRC_179D_BASE_DEDUCTION_FLOOR_CENTS_PER_SQ_FT: u64 = 50;
pub const IRC_179D_BASE_DEDUCTION_CEILING_CENTS_PER_SQ_FT: u64 = 100;
pub const IRC_179D_BONUS_DEDUCTION_FLOOR_CENTS_PER_SQ_FT: u64 = 250;
pub const IRC_179D_BONUS_DEDUCTION_CEILING_CENTS_PER_SQ_FT: u64 = 500;
pub const IRC_179D_BASE_DEDUCTION_INCREMENT_CENTS_PER_PERCENTAGE_POINT: u64 = 2;
pub const IRC_179D_BONUS_DEDUCTION_INCREMENT_CENTS_PER_PERCENTAGE_POINT: u64 = 10;
pub const IRC_179D_PRE_IRA_DEDUCTION_CAP_DOLLARS_PER_SQ_FT: u64 = 1;
pub const IRC_179D_TY_2025_MAX_BONUS_INFLATION_ADJUSTED_CENTS_PER_SQ_FT: u64 = 581;
pub const IRC_179D_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConstructionBeginDateStatus {
    ConstructionBeginsOnOrBeforeJune30_2026Eligible,
    ConstructionBeginsAfterJune30_2026PostObbbaTermination,
    PlacedInServiceBeforeJanuary1_2006PreEpactEffective,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BuildingSystem {
    InteriorLightingSystems,
    HvacAndHotWaterSystems,
    BuildingEnvelope,
    NotEligibleBuildingSystem,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BuildingOwnerType {
    PrivateBuildingOwner,
    GovernmentEntityFederalStateOrLocal,
    IndianTribalGovernment,
    AlaskaNativeCorporation,
    TaxExemptOrganizationPostIra2022,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAspect {
    EnergyReductionAtOrAboveTwentyFivePercentUnderSection179DC1,
    BaseDeductionAtFiftyCentsToOneDollarPerSqFtUnderSection179DB1,
    BonusDeductionWithPrevailingWageAndApprenticeshipUnderSection179DB3,
    EligibleBuildingSystemUnderSection179DC1,
    DesignerAllocationFromGovernmentOrTaxExemptEntityUnderSection179DD2,
    AshraeReferenceStandardCompliance,
    InflationAdjustmentUnderSection179DG,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section179DMode {
    NotApplicableConstructionBeginsAfterJune30_2026PostObbbaTermination,
    NotApplicablePlacedInServiceBeforeJanuary1_2006PreEpactEffective,
    NotApplicableNotEligibleBuildingSystem,
    NotApplicableEnergyReductionBelowTwentyFivePercentThreshold,
    CompliantBaseDeductionWithinFiftyCentsToOneDollarRange,
    CompliantBonusDeductionWithinTwoFiftyToFiveDollarRange,
    CompliantEligibleBuildingSystemWithProperCertification,
    CompliantDesignerAllocationFromGovernmentOrTaxExemptEntity,
    CompliantAshraeReferenceStandardMet,
    CompliantInflationAdjustedTy2025MaxAt5_81PerSqFt,
    ViolationDeductionClaimedWithoutCertification,
    ViolationPrevailingWageRequirementNotMet,
    ViolationApprenticeshipRequirementNotMet,
    ViolationDeductionExceedsApplicableCap,
    ViolationAshraeReferenceStandardNotMet,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub construction_begin_date_status: ConstructionBeginDateStatus,
    pub building_system: BuildingSystem,
    pub building_owner_type: BuildingOwnerType,
    pub compliance_aspect: ComplianceAspect,
    pub energy_reduction_percentage_points: u32,
    pub deduction_claimed_dollars_per_sq_ft_cents: u64,
    pub prevailing_wage_requirement_met: bool,
    pub apprenticeship_requirement_met: bool,
    pub certification_obtained: bool,
    pub ashrae_reference_standard_met: bool,
    pub designer_allocation_properly_documented: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section179DMode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
    pub computed_deduction_cents_per_sq_ft: u64,
}

pub type Section179DInput = Input;
pub type Section179DOutput = Output;
pub type Section179DResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "IRC § 179D Energy Efficient Commercial Buildings Deduction added by Section 1331 of the Energy Policy Act of 2005 (Public Law 109-58, 119 Stat. 594); signed by President George W. Bush on August 8, 2005; effective for property placed in service after December 31, 2005".to_string(),
        "Inflation Reduction Act of 2022 § 13303 (Public Law 117-169, 136 Stat. 1818); signed by President Joe Biden on August 16, 2022; substantially amended § 179D effective for tax years beginning after December 31, 2022".to_string(),
        "IRC § 179D(c)(1) Energy Reduction Threshold — building must achieve at least 25 PERCENT annual energy and power cost reduction (reduced from prior 50 % threshold by IRA 2022)".to_string(),
        "IRC § 179D(b)(1) Base Deduction Amount — base deduction starts at $0.50 per square foot for 25 % energy reduction; increases by $0.02 per percentage point of additional energy reduction; capped at $1.00 per square foot".to_string(),
        "IRC § 179D(b)(3) Bonus Deduction with Prevailing Wage and Apprenticeship — if PREVAILING WAGE AND APPRENTICESHIP requirements met, base amount increases to $2.50 per square foot for 25 % energy reduction; increases by $0.10 per percentage point of additional energy reduction; capped at $5.00 per square foot".to_string(),
        "IRC § 179D(g) Inflation Adjustment — deduction amounts are inflation-adjusted annually; for tax year 2025, maximum bonus deduction = $5.81 per square foot (up from $1.88/sq ft pre-IRA), reflecting cumulative inflation adjustment".to_string(),
        "IRC § 179D(c)(1) Eligible Building Systems — three categories of energy-efficient commercial building property (EECBP) eligible: (i) INTERIOR LIGHTING SYSTEMS; (ii) HEATING, VENTILATING, AND AIR CONDITIONING (HVAC) AND HOT WATER SYSTEMS; (iii) BUILDING ENVELOPE".to_string(),
        "ASHRAE 90.1 Reference Standard — energy reduction must be measured against the applicable ASHRAE Standard 90.1 reference building; specific version depends on placed-in-service date (90.1-2007 / 90.1-2013 / 90.1-2019 / 90.1-2022)".to_string(),
        "IRC § 179D(d)(2) Government / Tax-Exempt Designer Allocation — building owners that are federal / state / local government entities, Indian tribal governments, Alaska Native Corporations, or (post-IRA 2022) tax-exempt organizations may ALLOCATE THE DEDUCTION TO THE PRIMARY DESIGNER (architect / engineer / general contractor); allocation must be properly documented".to_string(),
        "Treas. Reg. § 1.179D-1 and IRS Notice 2006-52 + Notice 2008-40 — implementing regulations and procedural guidance".to_string(),
        "OBBBA 2025 Termination — § 179D ELIMINATED for buildings whose construction begins after JUNE 30, 2026 by Public Law 119-21 (One, Big, Beautiful Bill Act of 2025, 139 Stat. 72), signed by President Donald Trump on JULY 4, 2025; projects must begin construction on or before June 30, 2026 to qualify; placed-in-service date can occur later".to_string(),
        "IRS — FAQs for Modification of Sections 25C, 25D, 25E, 30C, 30D, 45L, 45W, AND 179D under Public Law 119-21 — official IRS termination guidance".to_string(),
        "IRS — Energy Efficient Commercial Buildings Deduction (Program Page) + IRC 179D Practice Unit PDF — primary administrative guidance".to_string(),
        "Energy Department + Every CRS Report + CLA + Cherry Bekaert + CSSI Services + Arnold & Porter + Grant Thornton + MGO CPA — practitioner overviews of § 179D".to_string(),
    ];

    match input.construction_begin_date_status {
        ConstructionBeginDateStatus::ConstructionBeginsAfterJune30_2026PostObbbaTermination => {
            return Output {
                mode: Section179DMode::NotApplicableConstructionBeginsAfterJune30_2026PostObbbaTermination,
                statutory_basis: "OBBBA 2025 § 179D termination — buildings whose construction begins after June 30, 2026 ineligible".to_string(),
                notes: "NOT APPLICABLE: building construction begins after June 30, 2026; § 179D deduction TERMINATED by One Big Beautiful Bill Act of 2025 (Public Law 119-21, signed July 4, 2025); projects must begin construction on or before June 30, 2026 to qualify (placed-in-service date can occur later).".to_string(),
                citations,
                computed_deduction_cents_per_sq_ft: 0,
            };
        }
        ConstructionBeginDateStatus::PlacedInServiceBeforeJanuary1_2006PreEpactEffective => {
            return Output {
                mode: Section179DMode::NotApplicablePlacedInServiceBeforeJanuary1_2006PreEpactEffective,
                statutory_basis: "Energy Policy Act of 2005 § 1331(d) effective date — § 179D applies only to property placed in service after December 31, 2005".to_string(),
                notes: "NOT APPLICABLE: building placed in service before January 1, 2006 (pre-EPAct effective date); § 179D deduction unavailable.".to_string(),
                citations,
                computed_deduction_cents_per_sq_ft: 0,
            };
        }
        ConstructionBeginDateStatus::ConstructionBeginsOnOrBeforeJune30_2026Eligible => {}
    }

    if input.building_system == BuildingSystem::NotEligibleBuildingSystem {
        return Output {
            mode: Section179DMode::NotApplicableNotEligibleBuildingSystem,
            statutory_basis: "IRC § 179D(c)(1) — eligible building systems limited to interior lighting + HVAC and hot water + building envelope".to_string(),
            notes: "NOT APPLICABLE: property is not within an eligible building system category under § 179D(c)(1) (interior lighting / HVAC and hot water / building envelope).".to_string(),
            citations,
            computed_deduction_cents_per_sq_ft: 0,
        };
    }

    if input.energy_reduction_percentage_points < 25 {
        return Output {
            mode: Section179DMode::NotApplicableEnergyReductionBelowTwentyFivePercentThreshold,
            statutory_basis: "IRC § 179D(c)(1) — energy reduction threshold of at least 25 % not met".to_string(),
            notes: "NOT APPLICABLE: building does not achieve at least 25 % annual energy and power cost reduction under § 179D(c)(1) (reduced from prior 50 % threshold by IRA 2022); § 179D deduction unavailable.".to_string(),
            citations,
            computed_deduction_cents_per_sq_ft: 0,
        };
    }

    let energy_reduction_above_threshold =
        input.energy_reduction_percentage_points.saturating_sub(25);

    match input.compliance_aspect {
        ComplianceAspect::EnergyReductionAtOrAboveTwentyFivePercentUnderSection179DC1 => Output {
            mode: Section179DMode::CompliantAshraeReferenceStandardMet,
            statutory_basis: "IRC § 179D(c)(1) — energy reduction at or above 25 % statutory threshold".to_string(),
            notes: "COMPLIANT: building achieves at least 25 % annual energy and power cost reduction under § 179D(c)(1).".to_string(),
            citations,
            computed_deduction_cents_per_sq_ft: 0,
        },
        ComplianceAspect::BaseDeductionAtFiftyCentsToOneDollarPerSqFtUnderSection179DB1 => {
            let computed = (IRC_179D_BASE_DEDUCTION_FLOOR_CENTS_PER_SQ_FT
                + u64::from(energy_reduction_above_threshold)
                    * IRC_179D_BASE_DEDUCTION_INCREMENT_CENTS_PER_PERCENTAGE_POINT)
                .min(IRC_179D_BASE_DEDUCTION_CEILING_CENTS_PER_SQ_FT);
            if input.deduction_claimed_dollars_per_sq_ft_cents <= computed {
                Output {
                    mode: Section179DMode::CompliantBaseDeductionWithinFiftyCentsToOneDollarRange,
                    statutory_basis: "IRC § 179D(b)(1) — base deduction within $0.50 to $1.00 per sq ft range".to_string(),
                    notes: format!(
                        "COMPLIANT: § 179D base deduction = {computed} cents per sq ft (50 cents base + 2 cents per percentage point above 25 %, capped at 100 cents)."
                    ),
                    citations,
                    computed_deduction_cents_per_sq_ft: computed,
                }
            } else {
                Output {
                    mode: Section179DMode::ViolationDeductionExceedsApplicableCap,
                    statutory_basis: "IRC § 179D(b)(1) — deduction claimed exceeds applicable base cap".to_string(),
                    notes: format!(
                        "VIOLATION: deduction claimed exceeds computed base cap of {computed} cents per sq ft under § 179D(b)(1)."
                    ),
                    citations,
                    computed_deduction_cents_per_sq_ft: computed,
                }
            }
        }
        ComplianceAspect::BonusDeductionWithPrevailingWageAndApprenticeshipUnderSection179DB3 => {
            if !input.prevailing_wage_requirement_met {
                Output {
                    mode: Section179DMode::ViolationPrevailingWageRequirementNotMet,
                    statutory_basis: "IRC § 179D(b)(3) — prevailing wage requirement not met".to_string(),
                    notes: "VIOLATION: prevailing wage requirement not met; bonus deduction tier of $2.50 to $5.00 per sq ft under § 179D(b)(3) unavailable; base tier of $0.50 to $1.00 per sq ft may still apply.".to_string(),
                    citations,
                    computed_deduction_cents_per_sq_ft: 0,
                }
            } else if !input.apprenticeship_requirement_met {
                Output {
                    mode: Section179DMode::ViolationApprenticeshipRequirementNotMet,
                    statutory_basis: "IRC § 179D(b)(3) — apprenticeship requirement not met".to_string(),
                    notes: "VIOLATION: apprenticeship requirement not met; bonus deduction tier of $2.50 to $5.00 per sq ft under § 179D(b)(3) unavailable; base tier of $0.50 to $1.00 per sq ft may still apply.".to_string(),
                    citations,
                    computed_deduction_cents_per_sq_ft: 0,
                }
            } else {
                let computed = (IRC_179D_BONUS_DEDUCTION_FLOOR_CENTS_PER_SQ_FT
                    + u64::from(energy_reduction_above_threshold)
                        * IRC_179D_BONUS_DEDUCTION_INCREMENT_CENTS_PER_PERCENTAGE_POINT)
                    .min(IRC_179D_BONUS_DEDUCTION_CEILING_CENTS_PER_SQ_FT);
                if input.deduction_claimed_dollars_per_sq_ft_cents <= computed {
                    Output {
                        mode: Section179DMode::CompliantBonusDeductionWithinTwoFiftyToFiveDollarRange,
                        statutory_basis: "IRC § 179D(b)(3) — bonus deduction within $2.50 to $5.00 per sq ft range".to_string(),
                        notes: format!(
                            "COMPLIANT: § 179D bonus deduction = {computed} cents per sq ft (250 cents base + 10 cents per percentage point above 25 %, capped at 500 cents); prevailing wage AND apprenticeship requirements met."
                        ),
                        citations,
                        computed_deduction_cents_per_sq_ft: computed,
                    }
                } else {
                    Output {
                        mode: Section179DMode::ViolationDeductionExceedsApplicableCap,
                        statutory_basis: "IRC § 179D(b)(3) — deduction claimed exceeds applicable bonus cap".to_string(),
                        notes: format!(
                            "VIOLATION: deduction claimed exceeds computed bonus cap of {computed} cents per sq ft under § 179D(b)(3)."
                        ),
                        citations,
                        computed_deduction_cents_per_sq_ft: computed,
                    }
                }
            }
        }
        ComplianceAspect::EligibleBuildingSystemUnderSection179DC1 => {
            if input.certification_obtained {
                Output {
                    mode: Section179DMode::CompliantEligibleBuildingSystemWithProperCertification,
                    statutory_basis: "IRC § 179D(c)(1) — eligible building system with proper certification".to_string(),
                    notes: "COMPLIANT: property is within an eligible building system category (interior lighting / HVAC and hot water / building envelope) AND proper certification obtained from qualified engineer or contractor.".to_string(),
                    citations,
                    computed_deduction_cents_per_sq_ft: 0,
                }
            } else {
                Output {
                    mode: Section179DMode::ViolationDeductionClaimedWithoutCertification,
                    statutory_basis: "IRC § 179D(d)(1) + Treas. Reg. § 1.179D-1 — certification required from qualified engineer or contractor".to_string(),
                    notes: "VIOLATION: deduction claimed without required certification from qualified engineer or contractor; Treas. Reg. § 1.179D-1 requires written certification + IRS Notice 2006-52 + Notice 2008-40 procedural compliance.".to_string(),
                    citations,
                    computed_deduction_cents_per_sq_ft: 0,
                }
            }
        }
        ComplianceAspect::DesignerAllocationFromGovernmentOrTaxExemptEntityUnderSection179DD2 => {
            if matches!(
                input.building_owner_type,
                BuildingOwnerType::GovernmentEntityFederalStateOrLocal
                    | BuildingOwnerType::IndianTribalGovernment
                    | BuildingOwnerType::AlaskaNativeCorporation
                    | BuildingOwnerType::TaxExemptOrganizationPostIra2022
            ) && input.designer_allocation_properly_documented
            {
                Output {
                    mode: Section179DMode::CompliantDesignerAllocationFromGovernmentOrTaxExemptEntity,
                    statutory_basis: "IRC § 179D(d)(2) — designer allocation properly documented from government / tax-exempt entity".to_string(),
                    notes: "COMPLIANT: building owner (government / Indian tribal / Alaska Native Corporation / post-IRA 2022 tax-exempt organization) properly allocated § 179D deduction to primary designer (architect / engineer / general contractor) with proper documentation.".to_string(),
                    citations,
                    computed_deduction_cents_per_sq_ft: 0,
                }
            } else {
                Output {
                    mode: Section179DMode::ViolationDeductionClaimedWithoutCertification,
                    statutory_basis: "IRC § 179D(d)(2) — designer allocation not properly documented from eligible building owner".to_string(),
                    notes: "VIOLATION: § 179D designer allocation claimed without proper documentation OR building owner does not qualify as government / Indian tribal / Alaska Native Corporation / post-IRA 2022 tax-exempt organization; § 179D(d)(2) allocation invalid.".to_string(),
                    citations,
                    computed_deduction_cents_per_sq_ft: 0,
                }
            }
        }
        ComplianceAspect::AshraeReferenceStandardCompliance => {
            if input.ashrae_reference_standard_met {
                Output {
                    mode: Section179DMode::CompliantAshraeReferenceStandardMet,
                    statutory_basis: "ASHRAE 90.1 Reference Standard met".to_string(),
                    notes: "COMPLIANT: energy reduction measured against the applicable ASHRAE Standard 90.1 reference building; reference standard met.".to_string(),
                    citations,
                    computed_deduction_cents_per_sq_ft: 0,
                }
            } else {
                Output {
                    mode: Section179DMode::ViolationAshraeReferenceStandardNotMet,
                    statutory_basis: "ASHRAE 90.1 Reference Standard not met".to_string(),
                    notes: "VIOLATION: energy reduction NOT measured against the applicable ASHRAE Standard 90.1 reference building OR reference standard not met; deduction invalid.".to_string(),
                    citations,
                    computed_deduction_cents_per_sq_ft: 0,
                }
            }
        }
        ComplianceAspect::InflationAdjustmentUnderSection179DG => Output {
            mode: Section179DMode::CompliantInflationAdjustedTy2025MaxAt5_81PerSqFt,
            statutory_basis: "IRC § 179D(g) — inflation adjustment under TY 2025 maximum at $5.81 per sq ft".to_string(),
            notes: "COMPLIANT: tax year 2025 maximum bonus deduction = $5.81 per square foot under § 179D(g) inflation adjustment (up from $1.88/sq ft pre-IRA).".to_string(),
            citations,
            computed_deduction_cents_per_sq_ft: IRC_179D_TY_2025_MAX_BONUS_INFLATION_ADJUSTED_CENTS_PER_SQ_FT,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_input() -> Input {
        Input {
            construction_begin_date_status:
                ConstructionBeginDateStatus::ConstructionBeginsOnOrBeforeJune30_2026Eligible,
            building_system: BuildingSystem::InteriorLightingSystems,
            building_owner_type: BuildingOwnerType::PrivateBuildingOwner,
            compliance_aspect:
                ComplianceAspect::BaseDeductionAtFiftyCentsToOneDollarPerSqFtUnderSection179DB1,
            energy_reduction_percentage_points: 25,
            deduction_claimed_dollars_per_sq_ft_cents: 50,
            prevailing_wage_requirement_met: true,
            apprenticeship_requirement_met: true,
            certification_obtained: true,
            ashrae_reference_standard_met: true,
            designer_allocation_properly_documented: false,
        }
    }

    #[test]
    fn post_obbba_construction_begin_not_applicable() {
        let mut input = baseline_input();
        input.construction_begin_date_status =
            ConstructionBeginDateStatus::ConstructionBeginsAfterJune30_2026PostObbbaTermination;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section179DMode::NotApplicableConstructionBeginsAfterJune30_2026PostObbbaTermination
        );
    }

    #[test]
    fn pre_epact_placed_in_service_not_applicable() {
        let mut input = baseline_input();
        input.construction_begin_date_status =
            ConstructionBeginDateStatus::PlacedInServiceBeforeJanuary1_2006PreEpactEffective;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section179DMode::NotApplicablePlacedInServiceBeforeJanuary1_2006PreEpactEffective
        );
    }

    #[test]
    fn not_eligible_building_system_not_applicable() {
        let mut input = baseline_input();
        input.building_system = BuildingSystem::NotEligibleBuildingSystem;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section179DMode::NotApplicableNotEligibleBuildingSystem
        );
    }

    #[test]
    fn energy_reduction_below_25_pct_not_applicable() {
        let mut input = baseline_input();
        input.energy_reduction_percentage_points = 24;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section179DMode::NotApplicableEnergyReductionBelowTwentyFivePercentThreshold
        );
    }

    #[test]
    fn base_deduction_at_25_pct_at_50_cents_compliant() {
        let output = check(&baseline_input());
        assert_eq!(
            output.mode,
            Section179DMode::CompliantBaseDeductionWithinFiftyCentsToOneDollarRange
        );
        assert_eq!(output.computed_deduction_cents_per_sq_ft, 50);
    }

    #[test]
    fn base_deduction_at_30_pct_at_60_cents_compliant() {
        let mut input = baseline_input();
        input.energy_reduction_percentage_points = 30;
        input.deduction_claimed_dollars_per_sq_ft_cents = 60;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section179DMode::CompliantBaseDeductionWithinFiftyCentsToOneDollarRange
        );
        assert_eq!(output.computed_deduction_cents_per_sq_ft, 60);
    }

    #[test]
    fn base_deduction_capped_at_100_cents() {
        let mut input = baseline_input();
        input.energy_reduction_percentage_points = 100;
        input.deduction_claimed_dollars_per_sq_ft_cents = 100;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section179DMode::CompliantBaseDeductionWithinFiftyCentsToOneDollarRange
        );
        assert_eq!(output.computed_deduction_cents_per_sq_ft, 100);
    }

    #[test]
    fn base_deduction_exceeds_cap_violation() {
        let mut input = baseline_input();
        input.deduction_claimed_dollars_per_sq_ft_cents = 51;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section179DMode::ViolationDeductionExceedsApplicableCap
        );
    }

    #[test]
    fn bonus_deduction_with_prevailing_wage_and_apprenticeship_at_25_pct_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::BonusDeductionWithPrevailingWageAndApprenticeshipUnderSection179DB3;
        input.deduction_claimed_dollars_per_sq_ft_cents = 250;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section179DMode::CompliantBonusDeductionWithinTwoFiftyToFiveDollarRange
        );
        assert_eq!(output.computed_deduction_cents_per_sq_ft, 250);
    }

    #[test]
    fn bonus_deduction_at_50_pct_at_500_cents_capped() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::BonusDeductionWithPrevailingWageAndApprenticeshipUnderSection179DB3;
        input.energy_reduction_percentage_points = 50;
        input.deduction_claimed_dollars_per_sq_ft_cents = 500;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section179DMode::CompliantBonusDeductionWithinTwoFiftyToFiveDollarRange
        );
        assert_eq!(output.computed_deduction_cents_per_sq_ft, 500);
    }

    #[test]
    fn bonus_deduction_at_75_pct_capped_at_500_cents() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::BonusDeductionWithPrevailingWageAndApprenticeshipUnderSection179DB3;
        input.energy_reduction_percentage_points = 75;
        input.deduction_claimed_dollars_per_sq_ft_cents = 500;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section179DMode::CompliantBonusDeductionWithinTwoFiftyToFiveDollarRange
        );
        assert_eq!(output.computed_deduction_cents_per_sq_ft, 500);
    }

    #[test]
    fn bonus_deduction_without_prevailing_wage_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::BonusDeductionWithPrevailingWageAndApprenticeshipUnderSection179DB3;
        input.prevailing_wage_requirement_met = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section179DMode::ViolationPrevailingWageRequirementNotMet
        );
    }

    #[test]
    fn bonus_deduction_without_apprenticeship_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::BonusDeductionWithPrevailingWageAndApprenticeshipUnderSection179DB3;
        input.apprenticeship_requirement_met = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section179DMode::ViolationApprenticeshipRequirementNotMet
        );
    }

    #[test]
    fn eligible_building_system_with_certification_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EligibleBuildingSystemUnderSection179DC1;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section179DMode::CompliantEligibleBuildingSystemWithProperCertification
        );
    }

    #[test]
    fn eligible_building_system_without_certification_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EligibleBuildingSystemUnderSection179DC1;
        input.certification_obtained = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section179DMode::ViolationDeductionClaimedWithoutCertification
        );
    }

    #[test]
    fn designer_allocation_from_government_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::DesignerAllocationFromGovernmentOrTaxExemptEntityUnderSection179DD2;
        input.building_owner_type = BuildingOwnerType::GovernmentEntityFederalStateOrLocal;
        input.designer_allocation_properly_documented = true;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section179DMode::CompliantDesignerAllocationFromGovernmentOrTaxExemptEntity
        );
    }

    #[test]
    fn designer_allocation_from_tax_exempt_post_ira_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::DesignerAllocationFromGovernmentOrTaxExemptEntityUnderSection179DD2;
        input.building_owner_type = BuildingOwnerType::TaxExemptOrganizationPostIra2022;
        input.designer_allocation_properly_documented = true;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section179DMode::CompliantDesignerAllocationFromGovernmentOrTaxExemptEntity
        );
    }

    #[test]
    fn designer_allocation_from_private_owner_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::DesignerAllocationFromGovernmentOrTaxExemptEntityUnderSection179DD2;
        input.building_owner_type = BuildingOwnerType::PrivateBuildingOwner;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section179DMode::ViolationDeductionClaimedWithoutCertification
        );
    }

    #[test]
    fn ashrae_reference_standard_met_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::AshraeReferenceStandardCompliance;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section179DMode::CompliantAshraeReferenceStandardMet
        );
    }

    #[test]
    fn ashrae_reference_standard_not_met_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::AshraeReferenceStandardCompliance;
        input.ashrae_reference_standard_met = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section179DMode::ViolationAshraeReferenceStandardNotMet
        );
    }

    #[test]
    fn ty_2025_inflation_adjusted_max_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::InflationAdjustmentUnderSection179DG;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section179DMode::CompliantInflationAdjustedTy2025MaxAt5_81PerSqFt
        );
        assert_eq!(output.computed_deduction_cents_per_sq_ft, 581);
    }

    #[test]
    fn constants_pin_statutory_facts() {
        assert_eq!(IRC_179D_EPACT_2005_ENACTMENT_DATE_YEAR, 2005);
        assert_eq!(IRC_179D_EPACT_2005_ENACTMENT_DATE_MONTH, 8);
        assert_eq!(IRC_179D_EPACT_2005_ENACTMENT_DATE_DAY, 8);
        assert_eq!(IRC_179D_EPACT_2005_PUBLIC_LAW_CONGRESS, 109);
        assert_eq!(IRC_179D_EPACT_2005_PUBLIC_LAW_ENACTMENT, 58);
        assert_eq!(IRC_179D_EPACT_2005_ENABLING_SECTION, 1331);
        assert_eq!(IRC_179D_EPACT_2005_STAT_VOLUME, 119);
        assert_eq!(IRC_179D_EPACT_2005_STAT_PAGE, 594);
        assert_eq!(IRC_179D_IRA_2022_ENACTMENT_DATE_YEAR, 2022);
        assert_eq!(IRC_179D_IRA_2022_ENABLING_SECTION, 13303);
        assert_eq!(IRC_179D_OBBBA_ENACTMENT_DATE_YEAR, 2025);
        assert_eq!(IRC_179D_OBBBA_ENACTMENT_DATE_MONTH, 7);
        assert_eq!(IRC_179D_OBBBA_ENACTMENT_DATE_DAY, 4);
        assert_eq!(
            IRC_179D_OBBBA_TERMINATION_CONSTRUCTION_BEGIN_DATE_YEAR,
            2026
        );
        assert_eq!(IRC_179D_OBBBA_TERMINATION_CONSTRUCTION_BEGIN_DATE_MONTH, 6);
        assert_eq!(IRC_179D_OBBBA_TERMINATION_CONSTRUCTION_BEGIN_DATE_DAY, 30);
        assert_eq!(IRC_179D_ENERGY_REDUCTION_THRESHOLD_BPS, 2_500);
        assert_eq!(IRC_179D_BASE_DEDUCTION_FLOOR_CENTS_PER_SQ_FT, 50);
        assert_eq!(IRC_179D_BASE_DEDUCTION_CEILING_CENTS_PER_SQ_FT, 100);
        assert_eq!(IRC_179D_BONUS_DEDUCTION_FLOOR_CENTS_PER_SQ_FT, 250);
        assert_eq!(IRC_179D_BONUS_DEDUCTION_CEILING_CENTS_PER_SQ_FT, 500);
        assert_eq!(
            IRC_179D_BASE_DEDUCTION_INCREMENT_CENTS_PER_PERCENTAGE_POINT,
            2
        );
        assert_eq!(
            IRC_179D_BONUS_DEDUCTION_INCREMENT_CENTS_PER_PERCENTAGE_POINT,
            10
        );
        assert_eq!(
            IRC_179D_TY_2025_MAX_BONUS_INFLATION_ADJUSTED_CENTS_PER_SQ_FT,
            581
        );
        assert_eq!(IRC_179D_BASIS_POINT_DENOMINATOR, 10_000);
    }

    #[test]
    fn citations_contain_landmarks() {
        let output = check(&baseline_input());
        let joined = output.citations.join("\n");
        assert!(joined.contains("IRC § 179D"));
        assert!(joined.contains("Section 1331 of the Energy Policy Act of 2005"));
        assert!(joined.contains("Public Law 109-58"));
        assert!(joined.contains("119 Stat. 594"));
        assert!(joined.contains("August 8, 2005"));
        assert!(joined.contains("Inflation Reduction Act of 2022 § 13303"));
        assert!(joined.contains("Public Law 117-169"));
        assert!(joined.contains("§ 179D(b)(1)"));
        assert!(joined.contains("§ 179D(b)(3)"));
        assert!(joined.contains("§ 179D(c)(1)"));
        assert!(joined.contains("§ 179D(d)(2)"));
        assert!(joined.contains("§ 179D(g)"));
        assert!(joined.contains("25 PERCENT"));
        assert!(joined.contains("$0.50 per square foot"));
        assert!(joined.contains("$1.00 per square foot"));
        assert!(joined.contains("$2.50 per square foot"));
        assert!(joined.contains("$5.00 per square foot"));
        assert!(joined.contains("$5.81"));
        assert!(joined.contains("INTERIOR LIGHTING SYSTEMS"));
        assert!(joined.contains("HEATING, VENTILATING, AND AIR CONDITIONING"));
        assert!(joined.contains("BUILDING ENVELOPE"));
        assert!(joined.contains("ASHRAE"));
        assert!(joined.contains("90.1"));
        assert!(joined.contains("PREVAILING WAGE AND APPRENTICESHIP"));
        assert!(joined.contains("ALLOCATE THE DEDUCTION TO THE PRIMARY DESIGNER"));
        assert!(joined.contains("One, Big, Beautiful Bill Act of 2025"));
        assert!(joined.contains("Public Law 119-21"));
        assert!(joined.contains("JUNE 30, 2026"));
        assert!(joined.contains("JULY 4, 2025"));
    }
}
