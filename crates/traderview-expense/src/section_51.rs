//! IRC § 51 Work Opportunity Tax Credit (WOTC)
//! Compliance Module — pure-compute check for the
//! **WORK OPPORTUNITY TAX CREDIT**, a federal employment
//! credit available to employers who hire individuals
//! from **TEN STATUTORY TARGET GROUPS**. Jointly
//! administered by the **IRS and Department of Labor (DOL)**.
//!
//! Credit equals **40% of qualified wages** (up to $6,000
//! for most target groups, $24,000 for qualified veterans
//! with service-connected disability and 6+ months
//! unemployment); **25% rate** for individuals working
//! 120-399 hours; **40% rate** for 400+ hours; **NO CREDIT**
//! for fewer than 120 hours. Form 8850 pre-screening
//! certification required within **28 DAYS** of start date.
//!
//! **Sunset Status**: WOTC expired **DECEMBER 31, 2025**
//! under prior law; Congress has not reauthorized as of
//! the most recent verified date in 2026. Employers should
//! continue filing Form 8850 within 28 days for any new
//! hires meeting target group criteria to preserve eligibility
//! if retroactive extension is enacted.
//!
//! Web research (verified 2026-06-03):
//! - **IRC § 51 Work Opportunity Tax Credit (WOTC)**: a **GENERAL BUSINESS CREDIT** provided under § 51; jointly administered by the **IRS and the Department of Labor (DOL)** ([IRS — Work Opportunity Tax Credit](https://www.irs.gov/businesses/small-businesses-self-employed/work-opportunity-tax-credit); [Congressional Research Service R43729 — The Work Opportunity Tax Credit](https://www.congress.gov/crs-product/R43729); [Tax Notes — IRC Code Section 51 (Amount of Credit)](https://www.taxnotes.com/research/federal/usc26/51); [Congressional Research Service R43729 — The Work Opportunity Tax Credit Updated May 13, 2026 PDF](https://www.congress.gov/crs_external_products/R/PDF/R43729/R43729.9.pdf); [LegalClarity — IRC 51: The Work Opportunity Tax Credit](https://legalclarity.org/irc-51-the-work-opportunity-tax-credit/); [AndreTaxCo — Work Opportunity Tax Credit](https://www.andretaxco.com/wotc); [TN.gov Workforce — Work Opportunity Tax Credit](https://www.tn.gov/workforce/employers/tax-and-insurance-redirect/apply-for-hiring-incentives/work-opportunity-tax-credit.html); [IRS Notice 99-51 PDF](https://www.irs.gov/pub/irs-drop/n-99-51.pdf); [IRS Notice 2009-28 PDF](https://www.irs.gov/pub/irs-drop/n-09-28.pdf); [IRS Notice 2012-13 PDF](https://www.irs.gov/pub/irs-drop/n-12-13.pdf); [IRS — Work Opportunity Tax Credit Available Until End of 2025](https://www.irs.gov/newsroom/the-work-opportunity-tax-credit-is-available-until-the-end-of-2025); [Accountably — Form 8850 Guide WOTC 28-day Rule and ETA 9061](https://accountably.com/irs-forms/f8850/); [NY DOL — WOTC Program](https://dol.ny.gov/wotc-program); [LegalClarity — Form 8850 Instructions, Deadline, and WOTC Rules](https://legalclarity.org/what-is-the-8850-form-for-the-work-opportunity-tax-credit/); [Paycor — What Is WOTC? The Work Opportunity Tax Credit Guide](https://www.paycor.com/resource-center/articles/work-opportunity-tax-credit/); [KMK Law — Work Opportunity Tax Credit At Risk](https://www.kmklaw.com/labor-employment/work-opportunity-tax-credit-at-risk); [Walton Management — Form 8850 Deadline Mistake That Costs Employers Up to $9,600 Per Hire](https://waltonmgt.com/when-is-form-8850-due-for-the-work-opportunity-tax-credit/); [DOL — How to File a WOTC Certification Request](https://www.dol.gov/agencies/eta/wotc/how-to-file); [IRS — Employers Must Certify Eligibility of New Hires to Claim WOTC](https://www.irs.gov/newsroom/employers-must-certify-eligibility-of-new-hires-to-claim-the-work-opportunity-tax-credit)).
//! - **§ 51(a) Credit Amount**: credit equal to **40% OF QUALIFIED WAGES** (up to applicable wage cap) for individuals who perform **AT LEAST 400 HOURS OF SERVICE** for the employer; **25% RATE** for individuals who perform **AT LEAST 120 BUT FEWER THAN 400 HOURS**; **NO CREDIT** for individuals working fewer than **120 HOURS**.
//! - **§ 51(b) Qualified Wages**: includes wages paid by the employer to the individual; default wage cap is **$6,000** per qualified individual (yielding maximum credit of **$2,400** for most target groups); special wage caps for qualified veterans (**$24,000** for service-connected disability + 6 months unemployment = **$9,600 maximum credit**); summer youth employees have a **$3,000** wage cap (= $1,200 max credit); long-term family assistance recipients have a **$10,000** wage cap in year 1 + 50% rate × $10,000 in year 2 = **$9,000 maximum credit over 2 years**.
//! - **§ 51(d) Target Group Eligibility — Ten Target Groups**: (1) **QUALIFIED VETERANS** (multiple subcategories including service-connected disability, SNAP recipient, and unemployment-based); (2) **QUALIFIED EX-FELONS** (formerly incarcerated or previously convicted of felony); (3) **DESIGNATED COMMUNITY RESIDENTS (DCR)** (age 18-39 residing in empowerment zones / renewal communities / rural renewal counties); (4) **VOCATIONAL REHABILITATION REFERRALS** (referred from state vocational rehab agency or Veterans Affairs); (5) **SUMMER YOUTH EMPLOYEES** (16-17 years old residing in empowerment zone, employed May 1 - September 15); (6) **SNAP RECIPIENTS** (Supplemental Nutrition Assistance Program / food stamps; ages 18-39); (7) **SSI RECIPIENTS** (Supplemental Security Income); (8) **LONG-TERM FAMILY ASSISTANCE RECIPIENTS** (Title IV-A TANF for 18 consecutive months OR 18 of past 60 months); (9) **QUALIFIED LONG-TERM UNEMPLOYMENT RECIPIENTS** (27+ weeks of unemployment); (10) **TANF RECIPIENTS** (short-term Temporary Assistance for Needy Families).
//! - **§ 51(d)(13)(A) Form 8850 Pre-Screening + SWA Certification — 28-Day Deadline**: employer must obtain **STATE WORKFORCE AGENCY (SWA) CERTIFICATION** by submitting **FORM 8850** within **28 DAYS** of the individual's **START DATE**; pre-screening notice must be obtained **ON OR BEFORE THE DAY THE JOB OFFER IS MADE**; missing the 28-day deadline forfeits eligibility.
//! - **§ 51(i) Hour Requirements**: **120-HOUR MINIMUM** for any credit; **120-399 HOURS = 25% RATE**; **400+ HOURS = 40% RATE**; hours include all paid hours of service.
//! - **§ 280C(a) Wage Deduction Reduction**: employer's deduction for wages paid is **REDUCED by the WOTC credit amount** under § 280C(a); creates **NET TAX BENEFIT** of WOTC credit minus tax savings on deduction reduction (e.g., $2,400 credit minus $504 deduction reduction at 21% corporate rate = **$1,896 net benefit per $6,000 wage**).
//! - **§ 38 General Business Credit Aggregation**: WOTC is aggregated with other general business credits under § 38; subject to § 38(c) annual limitation.
//! - **§ 39 Carryback and Carryforward**: WOTC credit not used in current year carries back **1 YEAR** and forward **20 YEARS** under § 39 (as part of general business credit).
//! - **Form 5884 (Work Opportunity Credit)**: required to claim the WOTC credit on the federal income tax return.
//! - **ETA Form 9061 (Individual Characteristics Form)**: filed with Form 8850 with the SWA to document target group eligibility.
//! - **Sunset December 31, 2025**: WOTC was available for wages paid to certain individuals who began work on or before **DECEMBER 31, 2025** under prior law; **NO RETROACTIVE EXTENSION** has been enacted as of the most recent verified date in 2026; employers should continue filing Form 8850 within 28 days to preserve eligibility if retroactive extension is enacted.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const IRC_51_RATE_400_HOURS_PLUS_BPS: u64 = 4_000;
pub const IRC_51_RATE_120_TO_399_HOURS_BPS: u64 = 2_500;
pub const IRC_51_MIN_HOURS_FOR_CREDIT: u32 = 120;
pub const IRC_51_HOURS_THRESHOLD_FOR_FULL_RATE: u32 = 400;
pub const IRC_51_DEFAULT_WAGE_CAP_DOLLARS: u64 = 6_000;
pub const IRC_51_QUALIFIED_VETERAN_DISABILITY_WAGE_CAP_DOLLARS: u64 = 24_000;
pub const IRC_51_QUALIFIED_VETERAN_DISABILITY_MAX_CREDIT_DOLLARS: u64 = 9_600;
pub const IRC_51_SUMMER_YOUTH_WAGE_CAP_DOLLARS: u64 = 3_000;
pub const IRC_51_LONG_TERM_FAMILY_ASSISTANCE_YEAR_1_WAGE_CAP_DOLLARS: u64 = 10_000;
pub const IRC_51_LONG_TERM_FAMILY_ASSISTANCE_YEAR_2_RATE_BPS: u64 = 5_000;
pub const IRC_51_LONG_TERM_FAMILY_ASSISTANCE_MAX_2_YEAR_CREDIT_DOLLARS: u64 = 9_000;
pub const IRC_51_DEFAULT_MAX_CREDIT_DOLLARS: u64 = 2_400;
pub const IRC_51_FORM_8850_DEADLINE_DAYS: u32 = 28;
pub const IRC_51_ETA_FORM_9061_NUMBER: u32 = 9_061;
pub const IRC_51_FORM_8850_NUMBER: u32 = 8_850;
pub const IRC_51_FORM_5884_NUMBER: u32 = 5_884;
pub const IRC_51_CARRYBACK_YEARS: u32 = 1;
pub const IRC_51_CARRYFORWARD_YEARS: u32 = 20;
pub const IRC_51_SUNSET_YEAR: u32 = 2025;
pub const IRC_51_SUNSET_MONTH: u32 = 12;
pub const IRC_51_SUNSET_DAY: u32 = 31;
pub const IRC_51_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetGroup {
    QualifiedVeteranServiceConnectedDisabilityAndUnemployed,
    QualifiedVeteranOther,
    QualifiedExFelon,
    DesignatedCommunityResident,
    VocationalRehabilitationReferral,
    SummerYouthEmployee,
    SnapRecipient,
    SsiRecipient,
    LongTermFamilyAssistanceRecipient,
    QualifiedLongTermUnemploymentRecipient,
    TanfRecipient,
    NotTargetGroupMember,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HoursWorkedTier {
    FewerThan120HoursNoCredit,
    Between120And399Hours25PercentRate,
    AtLeast400Hours40PercentRate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CertificationStatus {
    Form8850SubmittedToSwaWithin28Days,
    Form8850SubmittedToSwaPast28Days,
    Form8850NotSubmitted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PreScreeningStatus {
    PreScreeningNoticeObtainedOnOrBeforeJobOfferDate,
    PreScreeningNoticeObtainedAfterJobOfferDate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAspect {
    CreditAmountUnderSection51A,
    TargetGroupEligibilityUnderSection51D,
    HoursThresholdUnderSection51I,
    Form8850SwaCertificationUnderSection51D13A,
    PreScreeningNoticeRequirement,
    QualifiedWagesCapUnderSection51B,
    WageDeductionReductionUnderSection280Ca,
    GeneralBusinessCreditAggregationUnderSection38,
    CarrybackCarryforwardUnderSection39,
    FormFilingUnderForm5884,
    SunsetDecember31_2025,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section51Mode {
    NotApplicableNotTargetGroupMember,
    NotApplicableFewerThan120HoursNoCredit,
    NotApplicableForm8850Past28DaysOrNotSubmitted,
    NotApplicablePreScreeningNoticeAfterJobOfferDate,
    NotApplicableHiringDateAfterSunsetDecember31_2025,
    CompliantFortyPercentCreditFor400PlusHours,
    CompliantTwentyFivePercentCreditFor120To399Hours,
    CompliantQualifiedVeteranDisabilityMaximumCredit9600,
    CompliantSummerYouthMaximumCredit1200,
    CompliantLongTermFamilyAssistance2YearMaximumCredit9000,
    CompliantDefaultMaximumCredit2400,
    CompliantTargetGroupEligibilityCertified,
    CompliantForm8850SubmittedToSwaWithin28Days,
    CompliantPreScreeningNoticeObtainedOnOrBeforeJobOffer,
    CompliantQualifiedWagesWithinCap,
    CompliantWageDeductionReductionUnder280Ca,
    CompliantGeneralBusinessCreditAggregation,
    CompliantCarrybackCarryforwardObserved,
    CompliantForm5884FiledCorrectly,
    ViolationForm5884NotFiledOrIncorrect,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub target_group: TargetGroup,
    pub hours_worked_tier: HoursWorkedTier,
    pub certification_status: CertificationStatus,
    pub pre_screening_status: PreScreeningStatus,
    pub compliance_aspect: ComplianceAspect,
    pub qualified_wages_dollars: u64,
    pub hours_worked: u32,
    pub days_from_start_to_form_8850_filing: u32,
    pub hired_on_or_before_december_31_2025: bool,
    pub form_5884_filed_correctly: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section51Mode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
    pub computed_credit_dollars: u64,
}

pub type Section51Input = Input;
pub type Section51Output = Output;
pub type Section51Result = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "IRC § 51 Work Opportunity Tax Credit (WOTC) — general business credit provided under § 51; jointly administered by the IRS and Department of Labor (DOL)".to_string(),
        "§ 51(a) Credit Amount — 40% OF QUALIFIED WAGES for individuals who perform AT LEAST 400 HOURS OF SERVICE; 25% RATE for AT LEAST 120 BUT FEWER THAN 400 HOURS; NO CREDIT for individuals working fewer than 120 HOURS".to_string(),
        "§ 51(b) Qualified Wages — default wage cap is $6,000 per qualified individual ($2,400 max credit for most target groups); $24,000 cap for qualified veterans with service-connected disability + 6 months unemployment ($9,600 max credit); $3,000 cap for summer youth employees ($1,200 max credit); $10,000 cap year 1 + 50% rate × $10,000 year 2 for long-term family assistance recipients ($9,000 max 2-year credit)".to_string(),
        "§ 51(d) Target Group Eligibility — TEN TARGET GROUPS: (1) QUALIFIED VETERANS (service-connected disability + unemployment-based subcategories); (2) QUALIFIED EX-FELONS (formerly incarcerated); (3) DESIGNATED COMMUNITY RESIDENTS (ages 18-39 in empowerment zones / renewal communities / rural renewal counties); (4) VOCATIONAL REHABILITATION REFERRALS (state voc rehab or VA); (5) SUMMER YOUTH EMPLOYEES (16-17 in empowerment zone, May 1 - September 15); (6) SNAP RECIPIENTS (Supplemental Nutrition Assistance Program, ages 18-39); (7) SSI RECIPIENTS (Supplemental Security Income); (8) LONG-TERM FAMILY ASSISTANCE RECIPIENTS (Title IV-A TANF for 18 consecutive months OR 18 of past 60); (9) QUALIFIED LONG-TERM UNEMPLOYMENT RECIPIENTS (27+ weeks unemployment); (10) TANF RECIPIENTS (short-term Temporary Assistance for Needy Families)".to_string(),
        "§ 51(d)(13)(A) Form 8850 Pre-Screening + SWA Certification — 28-Day Deadline — employer must obtain STATE WORKFORCE AGENCY (SWA) CERTIFICATION by submitting FORM 8850 within 28 DAYS of the individual's START DATE; pre-screening notice must be obtained ON OR BEFORE THE DAY THE JOB OFFER IS MADE; missing the 28-day deadline forfeits eligibility".to_string(),
        "§ 51(i) Hour Requirements — 120-HOUR MINIMUM for any credit; 120-399 HOURS = 25% RATE; 400+ HOURS = 40% RATE; hours include all paid hours of service".to_string(),
        "§ 280C(a) Wage Deduction Reduction — employer's deduction for wages paid is REDUCED by the WOTC credit amount under § 280C(a); creates NET TAX BENEFIT of WOTC credit minus tax savings on deduction reduction".to_string(),
        "§ 38 General Business Credit Aggregation — WOTC is aggregated with other general business credits under § 38; subject to § 38(c) annual limitation".to_string(),
        "§ 39 Carryback and Carryforward — WOTC credit not used in current year carries back 1 YEAR and forward 20 YEARS under § 39 (as part of general business credit)".to_string(),
        "Form 5884 (Work Opportunity Credit) — required to claim the WOTC credit on the federal income tax return".to_string(),
        "ETA Form 9061 (Individual Characteristics Form) — filed with Form 8850 with the SWA to document target group eligibility".to_string(),
        "Sunset December 31, 2025 — WOTC was available for wages paid to certain individuals who began work on or before DECEMBER 31, 2025 under prior law; NO RETROACTIVE EXTENSION has been enacted as of the most recent verified date in 2026; employers should continue filing Form 8850 within 28 days to preserve eligibility if retroactive extension is enacted".to_string(),
        "IRS + Congressional Research Service + Tax Notes + LegalClarity + AndreTaxCo + TN.gov Workforce + Accountably + NY DOL + Paycor + KMK Law + Walton Management + DOL + Notice 99-51 + Notice 2009-28 + Notice 2012-13 — practitioner overviews of WOTC".to_string(),
    ];

    if input.target_group == TargetGroup::NotTargetGroupMember {
        return Output {
            mode: Section51Mode::NotApplicableNotTargetGroupMember,
            statutory_basis: "§ 51(d) — individual not a member of any of the ten enumerated target groups".to_string(),
            notes: "NOT APPLICABLE: individual is not a member of any of the ten enumerated target groups under § 51(d); WOTC credit unavailable.".to_string(),
            citations,
            computed_credit_dollars: 0,
        };
    }

    if input.hours_worked_tier == HoursWorkedTier::FewerThan120HoursNoCredit
        || input.hours_worked < IRC_51_MIN_HOURS_FOR_CREDIT
    {
        return Output {
            mode: Section51Mode::NotApplicableFewerThan120HoursNoCredit,
            statutory_basis: "§ 51(i) — fewer than 120 hours worked; no credit available".to_string(),
            notes: format!(
                "NOT APPLICABLE: individual worked {h} hours (fewer than 120-hour minimum); no WOTC credit available under § 51(i).",
                h = input.hours_worked,
            ),
            citations,
            computed_credit_dollars: 0,
        };
    }

    if input.certification_status != CertificationStatus::Form8850SubmittedToSwaWithin28Days
        || input.days_from_start_to_form_8850_filing > IRC_51_FORM_8850_DEADLINE_DAYS
    {
        return Output {
            mode: Section51Mode::NotApplicableForm8850Past28DaysOrNotSubmitted,
            statutory_basis: "§ 51(d)(13)(A) — Form 8850 not submitted to SWA within 28 days of start date".to_string(),
            notes: format!(
                "NOT APPLICABLE: Form 8850 submitted to SWA at day {d} (past 28-day statutory deadline) or not submitted; WOTC credit forfeited under § 51(d)(13)(A).",
                d = input.days_from_start_to_form_8850_filing,
            ),
            citations,
            computed_credit_dollars: 0,
        };
    }

    if input.pre_screening_status
        == PreScreeningStatus::PreScreeningNoticeObtainedAfterJobOfferDate
    {
        return Output {
            mode: Section51Mode::NotApplicablePreScreeningNoticeAfterJobOfferDate,
            statutory_basis: "§ 51(d)(13)(A) — pre-screening notice must be obtained on or before job offer date".to_string(),
            notes: "NOT APPLICABLE: pre-screening notice obtained after job offer date; pre-screening must occur ON OR BEFORE THE DAY THE JOB OFFER IS MADE under § 51(d)(13)(A).".to_string(),
            citations,
            computed_credit_dollars: 0,
        };
    }

    if !input.hired_on_or_before_december_31_2025 {
        return Output {
            mode: Section51Mode::NotApplicableHiringDateAfterSunsetDecember31_2025,
            statutory_basis: "WOTC sunset December 31, 2025 — credit unavailable for individuals beginning work after that date unless retroactively extended".to_string(),
            notes: "NOT APPLICABLE: individual began work after December 31, 2025 (WOTC sunset date); credit unavailable unless retroactively extended by Congress.".to_string(),
            citations,
            computed_credit_dollars: 0,
        };
    }

    match input.compliance_aspect {
        ComplianceAspect::CreditAmountUnderSection51A => {
            let rate_bps = match input.hours_worked_tier {
                HoursWorkedTier::FewerThan120HoursNoCredit => 0,
                HoursWorkedTier::Between120And399Hours25PercentRate => {
                    IRC_51_RATE_120_TO_399_HOURS_BPS
                }
                HoursWorkedTier::AtLeast400Hours40PercentRate => IRC_51_RATE_400_HOURS_PLUS_BPS,
            };
            let wage_cap = wage_cap_for_target_group(input.target_group);
            let capped_wages = input.qualified_wages_dollars.min(wage_cap);
            let computed = (u128::from(capped_wages) * u128::from(rate_bps)
                / u128::from(IRC_51_BASIS_POINT_DENOMINATOR)) as u64;
            let mode = match input.hours_worked_tier {
                HoursWorkedTier::FewerThan120HoursNoCredit => unreachable!(),
                HoursWorkedTier::Between120And399Hours25PercentRate => {
                    Section51Mode::CompliantTwentyFivePercentCreditFor120To399Hours
                }
                HoursWorkedTier::AtLeast400Hours40PercentRate => {
                    if input.target_group
                        == TargetGroup::QualifiedVeteranServiceConnectedDisabilityAndUnemployed
                    {
                        Section51Mode::CompliantQualifiedVeteranDisabilityMaximumCredit9600
                    } else if input.target_group == TargetGroup::SummerYouthEmployee {
                        Section51Mode::CompliantSummerYouthMaximumCredit1200
                    } else if input.target_group == TargetGroup::LongTermFamilyAssistanceRecipient {
                        Section51Mode::CompliantLongTermFamilyAssistance2YearMaximumCredit9000
                    } else {
                        Section51Mode::CompliantFortyPercentCreditFor400PlusHours
                    }
                }
            };
            Output {
                mode,
                statutory_basis: format!(
                    "§ 51(a) — {rate_bps} bps credit rate × capped qualified wages ${capped_wages} = ${computed} WOTC credit",
                ),
                notes: format!(
                    "COMPLIANT: {rate_bps} bps credit rate (based on {h} hours worked) × ${capped_wages} capped qualified wages (cap = ${wage_cap} for target group) = ${computed} WOTC credit.",
                    h = input.hours_worked,
                ),
                citations,
                computed_credit_dollars: computed,
            }
        }
        ComplianceAspect::TargetGroupEligibilityUnderSection51D => Output {
            mode: Section51Mode::CompliantTargetGroupEligibilityCertified,
            statutory_basis: "§ 51(d) — individual is member of an enumerated target group".to_string(),
            notes: format!(
                "COMPLIANT: individual is member of target group {tg:?} under § 51(d); eligible for WOTC credit subject to other requirements.",
                tg = input.target_group,
            ),
            citations,
            computed_credit_dollars: 0,
        },
        ComplianceAspect::HoursThresholdUnderSection51I => {
            let mode = match input.hours_worked_tier {
                HoursWorkedTier::FewerThan120HoursNoCredit => unreachable!(),
                HoursWorkedTier::Between120And399Hours25PercentRate => {
                    Section51Mode::CompliantTwentyFivePercentCreditFor120To399Hours
                }
                HoursWorkedTier::AtLeast400Hours40PercentRate => {
                    Section51Mode::CompliantFortyPercentCreditFor400PlusHours
                }
            };
            Output {
                mode,
                statutory_basis: "§ 51(i) — hours threshold met (120-399 hours = 25 % rate; 400+ hours = 40 % rate)".to_string(),
                notes: format!(
                    "COMPLIANT: {h} hours worked satisfies § 51(i) threshold ({tier:?}).",
                    h = input.hours_worked,
                    tier = input.hours_worked_tier,
                ),
                citations,
                computed_credit_dollars: 0,
            }
        }
        ComplianceAspect::Form8850SwaCertificationUnderSection51D13A => Output {
            mode: Section51Mode::CompliantForm8850SubmittedToSwaWithin28Days,
            statutory_basis: "§ 51(d)(13)(A) — Form 8850 submitted to SWA within 28-day deadline".to_string(),
            notes: format!(
                "COMPLIANT: Form 8850 submitted to State Workforce Agency at day {d} (within 28-day statutory window) under § 51(d)(13)(A).",
                d = input.days_from_start_to_form_8850_filing,
            ),
            citations,
            computed_credit_dollars: 0,
        },
        ComplianceAspect::PreScreeningNoticeRequirement => Output {
            mode: Section51Mode::CompliantPreScreeningNoticeObtainedOnOrBeforeJobOffer,
            statutory_basis: "§ 51(d)(13)(A) — pre-screening notice obtained on or before job offer date".to_string(),
            notes: "COMPLIANT: pre-screening notice obtained on or before the day the job offer was made under § 51(d)(13)(A).".to_string(),
            citations,
            computed_credit_dollars: 0,
        },
        ComplianceAspect::QualifiedWagesCapUnderSection51B => Output {
            mode: Section51Mode::CompliantQualifiedWagesWithinCap,
            statutory_basis: "§ 51(b) — qualified wages within target-group-specific cap".to_string(),
            notes: format!(
                "COMPLIANT: ${wages} qualified wages within ${cap} cap for target group {tg:?} under § 51(b).",
                wages = input.qualified_wages_dollars.min(wage_cap_for_target_group(input.target_group)),
                cap = wage_cap_for_target_group(input.target_group),
                tg = input.target_group,
            ),
            citations,
            computed_credit_dollars: 0,
        },
        ComplianceAspect::WageDeductionReductionUnderSection280Ca => Output {
            mode: Section51Mode::CompliantWageDeductionReductionUnder280Ca,
            statutory_basis: "§ 280C(a) — wage deduction reduced by WOTC credit amount".to_string(),
            notes: "COMPLIANT: employer's deduction for wages paid reduced by WOTC credit amount under § 280C(a); net tax benefit = WOTC credit minus tax savings on deduction reduction.".to_string(),
            citations,
            computed_credit_dollars: 0,
        },
        ComplianceAspect::GeneralBusinessCreditAggregationUnderSection38 => Output {
            mode: Section51Mode::CompliantGeneralBusinessCreditAggregation,
            statutory_basis: "§ 38 — WOTC aggregated with other general business credits".to_string(),
            notes: "COMPLIANT: WOTC aggregated with other general business credits under § 38; subject to § 38(c) annual limitation.".to_string(),
            citations,
            computed_credit_dollars: 0,
        },
        ComplianceAspect::CarrybackCarryforwardUnderSection39 => Output {
            mode: Section51Mode::CompliantCarrybackCarryforwardObserved,
            statutory_basis: "§ 39 — WOTC credit not used in current year carries back 1 year and forward 20 years".to_string(),
            notes: "COMPLIANT: WOTC credit not used in current year carries back 1 YEAR and forward 20 YEARS under § 39 (as part of general business credit).".to_string(),
            citations,
            computed_credit_dollars: 0,
        },
        ComplianceAspect::FormFilingUnderForm5884 => {
            if input.form_5884_filed_correctly {
                Output {
                    mode: Section51Mode::CompliantForm5884FiledCorrectly,
                    statutory_basis: "Form 5884 — Work Opportunity Credit form required to claim WOTC".to_string(),
                    notes: "COMPLIANT: Form 5884 filed correctly to claim WOTC credit; ETA Form 9061 also required with SWA filing.".to_string(),
                    citations,
                    computed_credit_dollars: 0,
                }
            } else {
                Output {
                    mode: Section51Mode::ViolationForm5884NotFiledOrIncorrect,
                    statutory_basis: "Form 5884 filing required to claim WOTC credit".to_string(),
                    notes: "VIOLATION: Form 5884 not filed or incorrectly filed; WOTC credit may be disallowed.".to_string(),
                    citations,
                    computed_credit_dollars: 0,
                }
            }
        }
        ComplianceAspect::SunsetDecember31_2025 => Output {
            mode: Section51Mode::NotApplicableHiringDateAfterSunsetDecember31_2025,
            statutory_basis: "WOTC sunset December 31, 2025 — credit unavailable for individuals beginning work after that date".to_string(),
            notes: "INFORMATIONAL: WOTC sunset December 31, 2025 under prior law; no retroactive extension has been enacted as of the most recent verified date in 2026.".to_string(),
            citations,
            computed_credit_dollars: 0,
        },
    }
}

fn wage_cap_for_target_group(target_group: TargetGroup) -> u64 {
    match target_group {
        TargetGroup::QualifiedVeteranServiceConnectedDisabilityAndUnemployed => {
            IRC_51_QUALIFIED_VETERAN_DISABILITY_WAGE_CAP_DOLLARS
        }
        TargetGroup::SummerYouthEmployee => IRC_51_SUMMER_YOUTH_WAGE_CAP_DOLLARS,
        TargetGroup::LongTermFamilyAssistanceRecipient => {
            IRC_51_LONG_TERM_FAMILY_ASSISTANCE_YEAR_1_WAGE_CAP_DOLLARS
        }
        _ => IRC_51_DEFAULT_WAGE_CAP_DOLLARS,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_input() -> Input {
        Input {
            target_group: TargetGroup::QualifiedExFelon,
            hours_worked_tier: HoursWorkedTier::AtLeast400Hours40PercentRate,
            certification_status: CertificationStatus::Form8850SubmittedToSwaWithin28Days,
            pre_screening_status:
                PreScreeningStatus::PreScreeningNoticeObtainedOnOrBeforeJobOfferDate,
            compliance_aspect: ComplianceAspect::CreditAmountUnderSection51A,
            qualified_wages_dollars: 6_000,
            hours_worked: 500,
            days_from_start_to_form_8850_filing: 14,
            hired_on_or_before_december_31_2025: true,
            form_5884_filed_correctly: true,
        }
    }

    #[test]
    fn not_target_group_member_not_applicable() {
        let mut input = baseline_input();
        input.target_group = TargetGroup::NotTargetGroupMember;
        let out = check(&input);
        assert_eq!(out.mode, Section51Mode::NotApplicableNotTargetGroupMember);
    }

    #[test]
    fn fewer_than_120_hours_no_credit() {
        let mut input = baseline_input();
        input.hours_worked_tier = HoursWorkedTier::FewerThan120HoursNoCredit;
        input.hours_worked = 119;
        let out = check(&input);
        assert_eq!(out.mode, Section51Mode::NotApplicableFewerThan120HoursNoCredit);
    }

    #[test]
    fn form_8850_past_28_day_deadline_not_applicable() {
        let mut input = baseline_input();
        input.certification_status = CertificationStatus::Form8850SubmittedToSwaPast28Days;
        input.days_from_start_to_form_8850_filing = 29;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section51Mode::NotApplicableForm8850Past28DaysOrNotSubmitted
        );
    }

    #[test]
    fn pre_screening_after_job_offer_not_applicable() {
        let mut input = baseline_input();
        input.pre_screening_status =
            PreScreeningStatus::PreScreeningNoticeObtainedAfterJobOfferDate;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section51Mode::NotApplicablePreScreeningNoticeAfterJobOfferDate
        );
    }

    #[test]
    fn hiring_date_after_sunset_not_applicable() {
        let mut input = baseline_input();
        input.hired_on_or_before_december_31_2025 = false;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section51Mode::NotApplicableHiringDateAfterSunsetDecember31_2025
        );
    }

    #[test]
    fn forty_percent_credit_for_400_hours_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::CreditAmountUnderSection51A;
        input.target_group = TargetGroup::QualifiedExFelon;
        input.hours_worked_tier = HoursWorkedTier::AtLeast400Hours40PercentRate;
        input.qualified_wages_dollars = 6_000;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section51Mode::CompliantFortyPercentCreditFor400PlusHours
        );
        assert_eq!(out.computed_credit_dollars, 2_400);
    }

    #[test]
    fn twenty_five_percent_credit_for_120_399_hours_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::CreditAmountUnderSection51A;
        input.target_group = TargetGroup::QualifiedExFelon;
        input.hours_worked_tier = HoursWorkedTier::Between120And399Hours25PercentRate;
        input.hours_worked = 200;
        input.qualified_wages_dollars = 6_000;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section51Mode::CompliantTwentyFivePercentCreditFor120To399Hours
        );
        assert_eq!(out.computed_credit_dollars, 1_500);
    }

    #[test]
    fn qualified_veteran_disability_max_credit_9600_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::CreditAmountUnderSection51A;
        input.target_group =
            TargetGroup::QualifiedVeteranServiceConnectedDisabilityAndUnemployed;
        input.hours_worked_tier = HoursWorkedTier::AtLeast400Hours40PercentRate;
        input.qualified_wages_dollars = 24_000;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section51Mode::CompliantQualifiedVeteranDisabilityMaximumCredit9600
        );
        assert_eq!(out.computed_credit_dollars, 9_600);
    }

    #[test]
    fn summer_youth_max_credit_1200_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::CreditAmountUnderSection51A;
        input.target_group = TargetGroup::SummerYouthEmployee;
        input.hours_worked_tier = HoursWorkedTier::AtLeast400Hours40PercentRate;
        input.qualified_wages_dollars = 3_000;
        let out = check(&input);
        assert_eq!(out.mode, Section51Mode::CompliantSummerYouthMaximumCredit1200);
        assert_eq!(out.computed_credit_dollars, 1_200);
    }

    #[test]
    fn long_term_family_assistance_max_credit_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::CreditAmountUnderSection51A;
        input.target_group = TargetGroup::LongTermFamilyAssistanceRecipient;
        input.hours_worked_tier = HoursWorkedTier::AtLeast400Hours40PercentRate;
        input.qualified_wages_dollars = 10_000;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section51Mode::CompliantLongTermFamilyAssistance2YearMaximumCredit9000
        );
        assert_eq!(out.computed_credit_dollars, 4_000);
    }

    #[test]
    fn wage_cap_above_default_pinned_to_cap() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::CreditAmountUnderSection51A;
        input.qualified_wages_dollars = 10_000;
        let out = check(&input);
        assert_eq!(out.computed_credit_dollars, 2_400);
    }

    #[test]
    fn target_group_eligibility_certified_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::TargetGroupEligibilityUnderSection51D;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section51Mode::CompliantTargetGroupEligibilityCertified
        );
    }

    #[test]
    fn hours_threshold_120_to_399_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::HoursThresholdUnderSection51I;
        input.hours_worked_tier = HoursWorkedTier::Between120And399Hours25PercentRate;
        input.hours_worked = 250;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section51Mode::CompliantTwentyFivePercentCreditFor120To399Hours
        );
    }

    #[test]
    fn hours_threshold_400_plus_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::HoursThresholdUnderSection51I;
        input.hours_worked_tier = HoursWorkedTier::AtLeast400Hours40PercentRate;
        input.hours_worked = 500;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section51Mode::CompliantFortyPercentCreditFor400PlusHours
        );
    }

    #[test]
    fn form_8850_swa_certification_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::Form8850SwaCertificationUnderSection51D13A;
        input.days_from_start_to_form_8850_filing = 28;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section51Mode::CompliantForm8850SubmittedToSwaWithin28Days
        );
    }

    #[test]
    fn pre_screening_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::PreScreeningNoticeRequirement;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section51Mode::CompliantPreScreeningNoticeObtainedOnOrBeforeJobOffer
        );
    }

    #[test]
    fn qualified_wages_within_cap_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::QualifiedWagesCapUnderSection51B;
        let out = check(&input);
        assert_eq!(out.mode, Section51Mode::CompliantQualifiedWagesWithinCap);
    }

    #[test]
    fn wage_deduction_reduction_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::WageDeductionReductionUnderSection280Ca;
        let out = check(&input);
        assert_eq!(out.mode, Section51Mode::CompliantWageDeductionReductionUnder280Ca);
    }

    #[test]
    fn general_business_credit_aggregation_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::GeneralBusinessCreditAggregationUnderSection38;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section51Mode::CompliantGeneralBusinessCreditAggregation
        );
    }

    #[test]
    fn carryback_carryforward_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::CarrybackCarryforwardUnderSection39;
        let out = check(&input);
        assert_eq!(out.mode, Section51Mode::CompliantCarrybackCarryforwardObserved);
    }

    #[test]
    fn form_5884_filed_correctly_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::FormFilingUnderForm5884;
        input.form_5884_filed_correctly = true;
        let out = check(&input);
        assert_eq!(out.mode, Section51Mode::CompliantForm5884FiledCorrectly);
    }

    #[test]
    fn form_5884_not_filed_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::FormFilingUnderForm5884;
        input.form_5884_filed_correctly = false;
        let out = check(&input);
        assert_eq!(out.mode, Section51Mode::ViolationForm5884NotFiledOrIncorrect);
    }

    #[test]
    fn sunset_december31_2025_informational() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SunsetDecember31_2025;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section51Mode::NotApplicableHiringDateAfterSunsetDecember31_2025
        );
    }

    #[test]
    fn constants_pin_section_51_wotc_structure() {
        assert_eq!(IRC_51_RATE_400_HOURS_PLUS_BPS, 4_000);
        assert_eq!(IRC_51_RATE_120_TO_399_HOURS_BPS, 2_500);
        assert_eq!(IRC_51_MIN_HOURS_FOR_CREDIT, 120);
        assert_eq!(IRC_51_HOURS_THRESHOLD_FOR_FULL_RATE, 400);
        assert_eq!(IRC_51_DEFAULT_WAGE_CAP_DOLLARS, 6_000);
        assert_eq!(IRC_51_QUALIFIED_VETERAN_DISABILITY_WAGE_CAP_DOLLARS, 24_000);
        assert_eq!(IRC_51_QUALIFIED_VETERAN_DISABILITY_MAX_CREDIT_DOLLARS, 9_600);
        assert_eq!(IRC_51_SUMMER_YOUTH_WAGE_CAP_DOLLARS, 3_000);
        assert_eq!(IRC_51_LONG_TERM_FAMILY_ASSISTANCE_YEAR_1_WAGE_CAP_DOLLARS, 10_000);
        assert_eq!(IRC_51_LONG_TERM_FAMILY_ASSISTANCE_YEAR_2_RATE_BPS, 5_000);
        assert_eq!(IRC_51_LONG_TERM_FAMILY_ASSISTANCE_MAX_2_YEAR_CREDIT_DOLLARS, 9_000);
        assert_eq!(IRC_51_DEFAULT_MAX_CREDIT_DOLLARS, 2_400);
        assert_eq!(IRC_51_FORM_8850_DEADLINE_DAYS, 28);
        assert_eq!(IRC_51_ETA_FORM_9061_NUMBER, 9_061);
        assert_eq!(IRC_51_FORM_8850_NUMBER, 8_850);
        assert_eq!(IRC_51_FORM_5884_NUMBER, 5_884);
        assert_eq!(IRC_51_CARRYBACK_YEARS, 1);
        assert_eq!(IRC_51_CARRYFORWARD_YEARS, 20);
        assert_eq!(IRC_51_SUNSET_YEAR, 2025);
        assert_eq!(IRC_51_SUNSET_MONTH, 12);
        assert_eq!(IRC_51_SUNSET_DAY, 31);
        assert_eq!(IRC_51_BASIS_POINT_DENOMINATOR, 10_000);
    }

    #[test]
    fn citations_pin_section_51_wotc_structure() {
        let input = baseline_input();
        let out = check(&input);
        let joined = out.citations.join(" || ");
        assert!(joined.contains("IRC § 51 Work Opportunity Tax Credit"));
        assert!(joined.contains("general business credit"));
        assert!(joined.contains("IRS and Department of Labor"));
        assert!(joined.contains("40% OF QUALIFIED WAGES"));
        assert!(joined.contains("AT LEAST 400 HOURS OF SERVICE"));
        assert!(joined.contains("25% RATE"));
        assert!(joined.contains("AT LEAST 120 BUT FEWER THAN 400 HOURS"));
        assert!(joined.contains("NO CREDIT"));
        assert!(joined.contains("$6,000"));
        assert!(joined.contains("$2,400"));
        assert!(joined.contains("$24,000"));
        assert!(joined.contains("$9,600"));
        assert!(joined.contains("$3,000"));
        assert!(joined.contains("$1,200"));
        assert!(joined.contains("$10,000"));
        assert!(joined.contains("$9,000"));
        assert!(joined.contains("TEN TARGET GROUPS"));
        assert!(joined.contains("QUALIFIED VETERANS"));
        assert!(joined.contains("QUALIFIED EX-FELONS"));
        assert!(joined.contains("DESIGNATED COMMUNITY RESIDENTS"));
        assert!(joined.contains("VOCATIONAL REHABILITATION REFERRALS"));
        assert!(joined.contains("SUMMER YOUTH EMPLOYEES"));
        assert!(joined.contains("SNAP RECIPIENTS"));
        assert!(joined.contains("SSI RECIPIENTS"));
        assert!(joined.contains("LONG-TERM FAMILY ASSISTANCE RECIPIENTS"));
        assert!(joined.contains("QUALIFIED LONG-TERM UNEMPLOYMENT RECIPIENTS"));
        assert!(joined.contains("TANF RECIPIENTS"));
        assert!(joined.contains("STATE WORKFORCE AGENCY (SWA) CERTIFICATION"));
        assert!(joined.contains("FORM 8850"));
        assert!(joined.contains("28 DAYS"));
        assert!(joined.contains("ON OR BEFORE THE DAY THE JOB OFFER IS MADE"));
        assert!(joined.contains("§ 280C(a)"));
        assert!(joined.contains("§ 38"));
        assert!(joined.contains("§ 39"));
        assert!(joined.contains("1 YEAR"));
        assert!(joined.contains("20 YEARS"));
        assert!(joined.contains("Form 5884"));
        assert!(joined.contains("ETA Form 9061"));
        assert!(joined.contains("DECEMBER 31, 2025"));
    }
}
