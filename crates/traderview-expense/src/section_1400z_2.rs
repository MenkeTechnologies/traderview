//! IRC § 1400Z-2 Special Rules for Capital Gains Invested
//! in Qualified Opportunity Zones — pure-compute compliance
//! check for trader/investor capital gain deferral, basis
//! step-up, and permanent gain exclusion under the original
//! Tax Cuts and Jobs Act of 2017 framework and the One Big
//! Beautiful Bill Act of 2025 enhanced and permanent
//! amendments.
//!
//! Originally enacted by **Section 13823 of the Tax Cuts
//! and Jobs Act of 2017 (Public Law 115-97)**, signed by
//! President Donald Trump on **December 22, 2017**, with
//! effective date for gains realized after that date. The
//! original program permitted capital gain deferral until
//! **December 31, 2026** with a 15 % combined basis step-up
//! (10 % at 5-year hold + 5 % additional at 7-year hold)
//! plus a permanent gain exclusion via fair market value
//! basis election after **10-year hold**.
//!
//! **One Big Beautiful Bill Act of 2025 (OBBBA)** signed by
//! President Donald Trump on **JULY 4, 2025** comprehensively
//! amended §§ 1400Z-1 and 1400Z-2 — permanently extended the
//! Qualified Opportunity Zone program; introduced rolling
//! 5-year deferral period for gains invested after December
//! 31, 2026; reduced regular-OZ basis step-up to 10 % (down
//! from 15 % combined); created Qualified Rural Opportunity
//! Funds (QROFs) with **30 % basis step-up after 5-year
//! hold**; imposed 30-year limit for gain exclusion; required
//! detailed reporting with up to **$50,000 penalty** for
//! large-fund noncompliance. Current QOZ designations sunset
//! end of 2026; new designations to be made in 90-day window
//! beginning **JULY 1, 2026** and effective **JANUARY 1,
//! 2027**; governors required to redesignate QOZs every 10
//! years going forward.
//!
//! Web research (verified 2026-06-03):
//! - **Original TCJA 2017 Enactment**: IRC § 1400Z-1 (Designation of Qualified Opportunity Zones) and § 1400Z-2 (Special Rules for Capital Gains Invested in Opportunity Zones) added by **Section 13823 of the Tax Cuts and Jobs Act of 2017 (Public Law 115-97, 131 Stat. 2054)**; signed by President Donald Trump on **December 22, 2017** ([House Office of Law Revision Counsel — 26 USC § 1400Z-2](https://uscode.house.gov/view.xhtml?req=granuleid%3AUSC-prelim-title26-section1400Z-2&num=0&edition=prelim); [Cornell LII — 26 U.S. Code § 1400Z-2 Special rules for capital gains invested in opportunity zones](https://www.law.cornell.edu/uscode/text/26/1400Z-2); [Bloomberg Tax — Sec. 1400Z-2 Special Rules For Capital Gains Invested In Opportunity Zones](https://irc.bloombergtax.com/public/uscode/doc/irc/section_1400z-2); [Tax Notes — IRC Section 1400Z-2 Special Rules for Opportunity Zones](https://www.taxnotes.com/research/federal/usc26/1400Z-2); [IRS — Opportunity Zones Frequently Asked Questions](https://www.irs.gov/credits-deductions/opportunity-zones-frequently-asked-questions); [IRS — Proposed Regulations REG-115420-18 PDF](https://www.irs.gov/pub/irs-drop/reg-115420-18.pdf); [Cherry Bekaert — Proposed Regulations on Qualified Opportunity Funds](https://www.cbh.com/insights/alerts/irs-releases-proposed-regulations-on-qualified-opportunity-funds/); [Cherry Bekaert — Pass-Through Owners and Qualified Opportunity Funds](https://www.cbh.com/insights/alerts/proposed-regulations-give-pass-through-owners-additional-time-to-defer-recognition-of-gains-by-investing-in-a-qualified-opportunity-fund/); [O'Melveny — New Guidance on Opportunity Zones Offers Taxpayers Ability to Defer Substantial Capital Gains](https://www.omm.com/insights/alerts-publications/new-guidance-on-opportunity-zones-offers-taxpayers-ability-to-defer-substantial-capital-gains/)).
//! - **180-Day Reinvestment Window**: § 1400Z-2(a)(1) — taxpayer must invest in a Qualified Opportunity Fund (QOF) during the **180-DAY PERIOD** beginning on the date of the sale or exchange giving rise to the gain.
//! - **Original Deferral End Date**: § 1400Z-2(b)(1) — gain to which subsection (a)(1)(B) applies shall be included in income in the taxable year which includes the earlier of (i) the date on which such investment is sold or exchanged, OR (ii) **DECEMBER 31, 2026**.
//! - **Original 15 % Combined Basis Step-Up**: § 1400Z-2(b)(2)(B) — (i) basis increased by **10 % of the deferred gain** if QOF investment held at least **5 years**; (ii) basis increased by an **ADDITIONAL 5 % of the deferred gain** if QOF investment held at least **7 years**; total potential step-up = **15 %**.
//! - **10-Year Hold Permanent Exclusion**: § 1400Z-2(c) — in the case of any investment held by the taxpayer for **AT LEAST 10 YEARS**, if the taxpayer elects, **THE BASIS OF SUCH PROPERTY SHALL BE EQUAL TO THE FAIR MARKET VALUE** of such investment on the date that the investment is sold or exchanged → **NO GAIN RECOGNIZED on QOF appreciation**.
//! - **OBBBA 2025 Enactment**: One Big Beautiful Bill Act of 2025 signed by President Donald Trump on **JULY 4, 2025** ([RSM US — The OBBBA Rekindles Opportunity Zones: What It Means for Real Estate](https://rsmus.com/insights/services/business-tax/obbba-tax-opportunity-zones.html); [PwC — Enhanced and Permanent Opportunity Zones as Part of the One Big Beautiful Bill Act](https://www.pwc.com/us/en/services/tax/library/enhanced-and-permanent-opportunity-zones-as-part-of-the-obba.html); [NAHB — What to Know About Opportunity Zone Changes in the One Big Beautiful Bill Act](https://www.nahb.org/blog/2025/08/opprtunity-zones-one-big-beautiful-bill-act); [Thomson Reuters Tax — Tax Experts on OBBBA Changes to Opportunity Zones](https://tax.thomsonreuters.com/news/tax-experts-on-obbba-changes-to-opportunity-zones/); [Smith and Howard — OBBBA 2025: Updates to Qualified Opportunity Zones (QOZs) — Capital Gain Tax Deferral Program](https://www.smith-howard.com/obbba-whats-new-with-qualified-opportunity-zones/); [Old Republic Title — Opportunity Zones: Key Changes in the 2025 Reform](https://www.oldrepublictitle.com/blog/opportunity-zones-2025/); [Landmark CPAs — Qualified Opportunity Zones Now Permanent: 4 Changes for Investors to Know](https://www.landmarkcpas.com/qualified-opportunity-zones-permanent/); [Williams Mullen — Big, Beautiful Changes to the Qualified Opportunity Zone Program](https://www.williamsmullen.com/insights/news/legal-news/big-beautiful-changes-qualified-opportunity-zone-program); [Baker Tilly — Opportunity Zones: 2025 Year-end Planning Considerations](https://www.bakertilly.com/insights/opportunity-zones-2025-year-end-planning-considera); [Crowe LLP — OBBBA Makes Enhanced Opportunity Zones Permanent](https://www.crowe.com/insights/tax-news-highlights/obbba-makes-enhanced-opportunity-zones-permanent)).
//! - **OBBBA Permanent Program**: OBBBA repealed the sunset date for QOZ investment, providing for **INDEFINITE INVESTMENT** in the QOZ program; governors required to redesignate QOZs every **10 YEARS** going forward.
//! - **OBBBA New Designations**: current QOZ designations sunset **end of 2026**; new designations to be made within a **90-DAY PERIOD beginning JULY 1, 2026**, subject to approval by the Treasury Secretary; new designations effective for new investments on **JANUARY 1, 2027**.
//! - **OBBBA Rolling 5-Year Deferral**: rolling **5-YEAR deferral period** for gains invested after **DECEMBER 31, 2026** with no fixed end date (replaces the original December 31, 2026 fixed-date trigger).
//! - **OBBBA Basis Step-Up**: **10 % basis step-up** for regular QOZ investments held at least 5 years (down from original 15 % combined); **30 % basis step-up** for qualified rural QOZ investments held at least 5 years; **30-YEAR LIMIT** for gain exclusion.
//! - **OBBBA Qualified Rural Opportunity Funds (QROFs)**: new fund category — QROFs must invest at least **90 %** of assets in low-income RURAL communities; **30 % basis step-up after 5-year hold**; substantial improvement requirement **REDUCED to 50 %** (from 100 %) of QROF basis in property over **31 MONTHS** for rural communities.
//! - **OBBBA Enhanced Reporting**: detailed reporting requirements + steep noncompliance penalties up to **$50,000** for large funds; new reporting requirements effective for tax years beginning after **JULY 4, 2025**.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const IRC_1400Z2_TCJA_ENACTMENT_DATE_YEAR: u32 = 2017;
pub const IRC_1400Z2_TCJA_ENACTMENT_DATE_MONTH: u32 = 12;
pub const IRC_1400Z2_TCJA_ENACTMENT_DATE_DAY: u32 = 22;
pub const IRC_1400Z2_TCJA_PUBLIC_LAW_CONGRESS: u32 = 115;
pub const IRC_1400Z2_TCJA_PUBLIC_LAW_ENACTMENT_NUMBER: u32 = 97;
pub const IRC_1400Z2_TCJA_ENABLING_SECTION_NUMBER: u32 = 13823;
pub const IRC_1400Z2_REINVESTMENT_WINDOW_DAYS: u32 = 180;
pub const IRC_1400Z2_ORIGINAL_DEFERRAL_END_DATE_YEAR: u32 = 2026;
pub const IRC_1400Z2_ORIGINAL_DEFERRAL_END_DATE_MONTH: u32 = 12;
pub const IRC_1400Z2_ORIGINAL_DEFERRAL_END_DATE_DAY: u32 = 31;
pub const IRC_1400Z2_ORIGINAL_FIVE_YEAR_STEP_UP_BPS: u64 = 1_000;
pub const IRC_1400Z2_ORIGINAL_SEVEN_YEAR_ADDITIONAL_STEP_UP_BPS: u64 = 500;
pub const IRC_1400Z2_ORIGINAL_COMBINED_FIFTEEN_PCT_STEP_UP_BPS: u64 = 1_500;
pub const IRC_1400Z2_TEN_YEAR_HOLDING_PERIOD_YEARS: u32 = 10;
pub const IRC_1400Z2_OBBBA_ENACTMENT_DATE_YEAR: u32 = 2025;
pub const IRC_1400Z2_OBBBA_ENACTMENT_DATE_MONTH: u32 = 7;
pub const IRC_1400Z2_OBBBA_ENACTMENT_DATE_DAY: u32 = 4;
pub const IRC_1400Z2_OBBBA_ROLLING_DEFERRAL_PERIOD_YEARS: u32 = 5;
pub const IRC_1400Z2_OBBBA_REGULAR_OZ_STEP_UP_BPS: u64 = 1_000;
pub const IRC_1400Z2_OBBBA_RURAL_QROF_STEP_UP_BPS: u64 = 3_000;
pub const IRC_1400Z2_OBBBA_GAIN_EXCLUSION_HORIZON_YEARS: u32 = 30;
pub const IRC_1400Z2_OBBBA_REPORTING_PENALTY_LARGE_FUND_DOLLARS: u64 = 50_000;
pub const IRC_1400Z2_OBBBA_NEW_DESIGNATIONS_EFFECTIVE_DATE_YEAR: u32 = 2027;
pub const IRC_1400Z2_OBBBA_NEW_DESIGNATIONS_EFFECTIVE_DATE_MONTH: u32 = 1;
pub const IRC_1400Z2_OBBBA_NEW_DESIGNATIONS_EFFECTIVE_DATE_DAY: u32 = 1;
pub const IRC_1400Z2_OBBBA_NEW_DESIGNATIONS_WINDOW_START_DATE_YEAR: u32 = 2026;
pub const IRC_1400Z2_OBBBA_NEW_DESIGNATIONS_WINDOW_START_DATE_MONTH: u32 = 7;
pub const IRC_1400Z2_OBBBA_NEW_DESIGNATIONS_WINDOW_START_DATE_DAY: u32 = 1;
pub const IRC_1400Z2_OBBBA_NEW_DESIGNATIONS_WINDOW_DAYS: u32 = 90;
pub const IRC_1400Z2_OBBBA_QROF_ASSET_INVESTMENT_THRESHOLD_BPS: u64 = 9_000;
pub const IRC_1400Z2_OBBBA_RURAL_SUBSTANTIAL_IMPROVEMENT_PCT_BPS: u64 = 5_000;
pub const IRC_1400Z2_OBBBA_REGULAR_SUBSTANTIAL_IMPROVEMENT_PCT_BPS: u64 = 10_000;
pub const IRC_1400Z2_OBBBA_RURAL_SUBSTANTIAL_IMPROVEMENT_PERIOD_MONTHS: u32 = 31;
pub const IRC_1400Z2_OBBBA_GOVERNOR_REDESIGNATION_CYCLE_YEARS: u32 = 10;
pub const IRC_1400Z2_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InvestmentRegime {
    OriginalTcja2017FrameworkPreDecember31_2026Gain,
    ObbbaAmendedRegimePostDecember31_2026Gain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum QofType {
    QualifiedOpportunityFundRegular,
    QualifiedRuralOpportunityFundQrof,
    NotAQualifiedOpportunityFund,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAspect {
    OneHundredEightyDayReinvestmentWindowUnderSection1400Z2A1,
    OriginalTcjaBasisStepUpAtFiveOrSevenYearHold,
    ObbbaBasisStepUpAtFiveYearHold,
    TenYearHoldFmvElectionPermanentExclusionUnderSection1400Z2C,
    ObbbaRollingFiveYearDeferralPeriod,
    ObbbaThirtyYearGainExclusionHorizon,
    ObbbaQrofSubstantialImprovementFiftyPctOverThirtyOneMonths,
    ObbbaEnhancedReportingRequirements,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section1400Z2Mode {
    NotApplicableNotEligibleCapitalGain,
    NotApplicableInvestmentOutsideOf180DayReinvestmentWindow,
    NotApplicableNotAQualifiedOpportunityFund,
    CompliantInvestmentWithin180DayWindow,
    CompliantOriginalTcjaFivePctStepUpAtSevenYearHoldTotalFifteenPct,
    CompliantOriginalTcjaTenPctStepUpAtFiveYearHold,
    CompliantObbbaRegularOzTenPctStepUpAtFiveYearHold,
    CompliantObbbaRuralQrofThirtyPctStepUpAtFiveYearHold,
    CompliantTenYearHoldFmvElectionPermanentExclusion,
    CompliantObbbaRollingFiveYearDeferralPeriod,
    CompliantObbbaThirtyYearGainExclusionHorizonRespected,
    CompliantObbbaQrofSubstantialImprovementMetWithinFiftyPctOverThirtyOneMonths,
    CompliantObbbaEnhancedReportingFiled,
    ViolationFailureToInvestWithin180DayReinvestmentWindow,
    ViolationFailureToHoldForRequiredYearsForStepUp,
    ViolationFailureToHoldForTenYearsForFmvElection,
    ViolationObbbaThirtyYearGainExclusionHorizonExceeded,
    ViolationObbbaQrofSubstantialImprovementBelowFiftyPctOrPastThirtyOneMonths,
    ViolationObbbaEnhancedReportingNotFiledLargeFundPenaltyExposure,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub regime: InvestmentRegime,
    pub qof_type: QofType,
    pub compliance_aspect: ComplianceAspect,
    pub days_from_realization_to_reinvestment: u32,
    pub holding_period_years: u32,
    pub holding_period_at_fmv_election_years: u32,
    pub years_since_obbba_investment_initial: u32,
    pub substantial_improvement_pct_bps: u64,
    pub substantial_improvement_period_months: u32,
    pub enhanced_reporting_filed: bool,
    pub is_large_fund_subject_to_enhanced_penalty: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section1400Z2Mode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
    pub basis_step_up_bps: u64,
    pub enhanced_reporting_penalty_exposure_dollars: u64,
}

pub type Section1400Z2Input = Input;
pub type Section1400Z2Output = Output;
pub type Section1400Z2Result = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "IRC § 1400Z-1 + § 1400Z-2 added by Section 13823 of the Tax Cuts and Jobs Act of 2017 (Public Law 115-97, 131 Stat. 2054); signed by President Donald Trump on December 22, 2017; effective for gains realized after that date".to_string(),
        "IRC § 1400Z-2(a)(1) — taxpayer must invest in a Qualified Opportunity Fund (QOF) during the 180-DAY PERIOD beginning on the date of the sale or exchange giving rise to the gain".to_string(),
        "IRC § 1400Z-2(b)(1) Original Deferral End Date — gain to which subsection (a)(1)(B) applies shall be included in income in the taxable year which includes the EARLIER of (i) the date on which such investment is sold or exchanged, OR (ii) DECEMBER 31, 2026".to_string(),
        "IRC § 1400Z-2(b)(2)(B) Original Basis Step-Up — (i) basis increased by 10 % of the deferred gain if QOF investment held at least 5 YEARS; (ii) basis increased by an ADDITIONAL 5 % of the deferred gain if QOF investment held at least 7 YEARS; total potential step-up = 15 %".to_string(),
        "IRC § 1400Z-2(c) Ten-Year Hold Permanent Exclusion — in the case of any investment held by the taxpayer for AT LEAST 10 YEARS, if the taxpayer elects, THE BASIS OF SUCH PROPERTY SHALL BE EQUAL TO THE FAIR MARKET VALUE of such investment on the date that the investment is sold or exchanged; effect: NO GAIN RECOGNIZED on QOF appreciation".to_string(),
        "One Big Beautiful Bill Act of 2025 (OBBBA) — signed by President Donald Trump on JULY 4, 2025; comprehensive amendments to §§ 1400Z-1 and 1400Z-2".to_string(),
        "OBBBA Permanent Program — OBBBA repealed the sunset date for QOZ investment, providing for INDEFINITE INVESTMENT in the QOZ program; governors required to redesignate QOZs every 10 YEARS going forward".to_string(),
        "OBBBA New Designations — current QOZ designations sunset end of 2026; new designations to be made within a 90-DAY PERIOD beginning JULY 1, 2026, subject to approval by the Treasury Secretary; new designations effective for new investments on JANUARY 1, 2027".to_string(),
        "OBBBA Rolling 5-Year Deferral Period — rolling 5-YEAR deferral period for gains invested after DECEMBER 31, 2026 with no fixed end date (replaces the original December 31, 2026 fixed-date trigger)".to_string(),
        "OBBBA Basis Step-Up — 10 % basis step-up for regular QOZ investments held at least 5 years (down from original 15 % combined); 30 % basis step-up for qualified rural QOZ investments held at least 5 years; 30-YEAR LIMIT for gain exclusion".to_string(),
        "OBBBA Qualified Rural Opportunity Funds (QROFs) — new fund category; QROFs must invest at least 90 % of assets in low-income RURAL communities; 30 % basis step-up after 5-year hold; substantial improvement requirement REDUCED to 50 % (from 100 %) of QROF basis in property over 31 MONTHS for rural communities".to_string(),
        "OBBBA Enhanced Reporting — detailed reporting requirements + steep noncompliance penalties up to $50,000 for large funds; new reporting requirements effective for tax years beginning after JULY 4, 2025".to_string(),
        "Treas. Reg. § 1.1400Z2(a)-1 through § 1.1400Z2(d)-2 — implementing regulations published in stages including December 22, 2017 first proposed regs and final regs T.D. 9889 (December 19, 2019)".to_string(),
        "PwC + RSM US + Crowe LLP + Williams Mullen + Baker Tilly + Smith and Howard + Old Republic Title + Landmark CPAs + NAHB + Thomson Reuters — OBBBA practitioner guides for QOZ + QROF planning".to_string(),
    ];

    if input.qof_type == QofType::NotAQualifiedOpportunityFund {
        return Output {
            mode: Section1400Z2Mode::NotApplicableNotAQualifiedOpportunityFund,
            statutory_basis: "IRC § 1400Z-2(d) Qualified Opportunity Fund definition — fund must satisfy the 90 % qualified opportunity zone property holding test under Treas. Reg. § 1.1400Z2(d)-1".to_string(),
            notes: "NOT APPLICABLE: investment vehicle does not qualify as a Qualified Opportunity Fund (QOF) under § 1400Z-2(d) or as a Qualified Rural Opportunity Fund (QROF) under OBBBA 2025; § 1400Z-2 deferral, step-up, and permanent exclusion provisions do not apply.".to_string(),
            citations,
            basis_step_up_bps: 0,
            enhanced_reporting_penalty_exposure_dollars: 0,
        };
    }

    match input.compliance_aspect {
        ComplianceAspect::OneHundredEightyDayReinvestmentWindowUnderSection1400Z2A1 => {
            if input.days_from_realization_to_reinvestment <= IRC_1400Z2_REINVESTMENT_WINDOW_DAYS {
                Output {
                    mode: Section1400Z2Mode::CompliantInvestmentWithin180DayWindow,
                    statutory_basis: "IRC § 1400Z-2(a)(1) — investment in QOF within 180-day reinvestment window".to_string(),
                    notes: "COMPLIANT: investment in QOF made within 180-day reinvestment window beginning on date of sale or exchange giving rise to the gain.".to_string(),
                    citations,
                    basis_step_up_bps: 0,
                    enhanced_reporting_penalty_exposure_dollars: 0,
                }
            } else {
                Output {
                    mode: Section1400Z2Mode::ViolationFailureToInvestWithin180DayReinvestmentWindow,
                    statutory_basis: "IRC § 1400Z-2(a)(1) — investment outside 180-day reinvestment window".to_string(),
                    notes: "VIOLATION: investment in QOF made past the 180-day reinvestment window under § 1400Z-2(a)(1); gain deferral election unavailable; gain recognized in original taxable year of realization.".to_string(),
                    citations,
                    basis_step_up_bps: 0,
                    enhanced_reporting_penalty_exposure_dollars: 0,
                }
            }
        }
        ComplianceAspect::OriginalTcjaBasisStepUpAtFiveOrSevenYearHold => {
            match input.regime {
                InvestmentRegime::OriginalTcja2017FrameworkPreDecember31_2026Gain => {
                    if input.holding_period_years >= 7 {
                        Output {
                            mode: Section1400Z2Mode::CompliantOriginalTcjaFivePctStepUpAtSevenYearHoldTotalFifteenPct,
                            statutory_basis: "IRC § 1400Z-2(b)(2)(B)(i) + (ii) — 10 % step-up at 5-year hold + additional 5 % step-up at 7-year hold (total 15 %)".to_string(),
                            notes: "COMPLIANT: QOF investment held at least 7 years; total combined basis step-up = 15 % of deferred gain (10 % at 5-year + additional 5 % at 7-year under original TCJA framework).".to_string(),
                            citations,
                            basis_step_up_bps: IRC_1400Z2_ORIGINAL_COMBINED_FIFTEEN_PCT_STEP_UP_BPS,
                            enhanced_reporting_penalty_exposure_dollars: 0,
                        }
                    } else if input.holding_period_years >= 5 {
                        Output {
                            mode: Section1400Z2Mode::CompliantOriginalTcjaTenPctStepUpAtFiveYearHold,
                            statutory_basis: "IRC § 1400Z-2(b)(2)(B)(i) — 10 % step-up at 5-year hold (partial benefit before 7-year tier)".to_string(),
                            notes: "COMPLIANT: QOF investment held at least 5 years but less than 7 years; basis step-up = 10 % of deferred gain under original TCJA framework.".to_string(),
                            citations,
                            basis_step_up_bps: IRC_1400Z2_ORIGINAL_FIVE_YEAR_STEP_UP_BPS,
                            enhanced_reporting_penalty_exposure_dollars: 0,
                        }
                    } else {
                        Output {
                            mode: Section1400Z2Mode::ViolationFailureToHoldForRequiredYearsForStepUp,
                            statutory_basis: "IRC § 1400Z-2(b)(2)(B) — basis step-up requires minimum 5-year holding period".to_string(),
                            notes: "VIOLATION: QOF investment held less than 5 years; no basis step-up available under original TCJA § 1400Z-2(b)(2)(B).".to_string(),
                            citations,
                            basis_step_up_bps: 0,
                            enhanced_reporting_penalty_exposure_dollars: 0,
                        }
                    }
                }
                InvestmentRegime::ObbbaAmendedRegimePostDecember31_2026Gain => Output {
                    mode: Section1400Z2Mode::ViolationFailureToHoldForRequiredYearsForStepUp,
                    statutory_basis: "OBBBA 2025 — original TCJA 15 % combined step-up not available for post-2026 gains; ObbbaBasisStepUpAtFiveYearHold compliance aspect applies".to_string(),
                    notes: "REGIME MISMATCH: original TCJA 15 % combined step-up does not apply to post-December 31, 2026 gains; OBBBA amended regime applies (10 % regular OZ / 30 % rural QROF step-up).".to_string(),
                    citations,
                    basis_step_up_bps: 0,
                    enhanced_reporting_penalty_exposure_dollars: 0,
                },
            }
        }
        ComplianceAspect::ObbbaBasisStepUpAtFiveYearHold => {
            if input.holding_period_years >= 5 {
                match input.qof_type {
                    QofType::QualifiedRuralOpportunityFundQrof => Output {
                        mode: Section1400Z2Mode::CompliantObbbaRuralQrofThirtyPctStepUpAtFiveYearHold,
                        statutory_basis: "OBBBA 2025 — QROF 30 % step-up at 5-year hold".to_string(),
                        notes: "COMPLIANT: Qualified Rural Opportunity Fund (QROF) investment held at least 5 years; basis step-up = 30 % of deferred gain under OBBBA 2025.".to_string(),
                        citations,
                        basis_step_up_bps: IRC_1400Z2_OBBBA_RURAL_QROF_STEP_UP_BPS,
                        enhanced_reporting_penalty_exposure_dollars: 0,
                    },
                    QofType::QualifiedOpportunityFundRegular => Output {
                        mode: Section1400Z2Mode::CompliantObbbaRegularOzTenPctStepUpAtFiveYearHold,
                        statutory_basis: "OBBBA 2025 — regular QOZ 10 % step-up at 5-year hold".to_string(),
                        notes: "COMPLIANT: regular Qualified Opportunity Fund investment held at least 5 years; basis step-up = 10 % of deferred gain under OBBBA 2025.".to_string(),
                        citations,
                        basis_step_up_bps: IRC_1400Z2_OBBBA_REGULAR_OZ_STEP_UP_BPS,
                        enhanced_reporting_penalty_exposure_dollars: 0,
                    },
                    QofType::NotAQualifiedOpportunityFund => unreachable!(),
                }
            } else {
                Output {
                    mode: Section1400Z2Mode::ViolationFailureToHoldForRequiredYearsForStepUp,
                    statutory_basis: "OBBBA 2025 — basis step-up requires minimum 5-year holding period".to_string(),
                    notes: "VIOLATION: QOF / QROF investment held less than 5 years; no OBBBA basis step-up available.".to_string(),
                    citations,
                    basis_step_up_bps: 0,
                    enhanced_reporting_penalty_exposure_dollars: 0,
                }
            }
        }
        ComplianceAspect::TenYearHoldFmvElectionPermanentExclusionUnderSection1400Z2C => {
            if input.holding_period_at_fmv_election_years >= IRC_1400Z2_TEN_YEAR_HOLDING_PERIOD_YEARS {
                Output {
                    mode: Section1400Z2Mode::CompliantTenYearHoldFmvElectionPermanentExclusion,
                    statutory_basis: "IRC § 1400Z-2(c) — 10-year hold + FMV election → permanent exclusion of QOF appreciation gain".to_string(),
                    notes: "COMPLIANT: QOF investment held at least 10 years; FMV election under § 1400Z-2(c) eligible; QOF appreciation gain permanently excluded from gross income on sale or exchange.".to_string(),
                    citations,
                    basis_step_up_bps: IRC_1400Z2_BASIS_POINT_DENOMINATOR,
                    enhanced_reporting_penalty_exposure_dollars: 0,
                }
            } else {
                Output {
                    mode: Section1400Z2Mode::ViolationFailureToHoldForTenYearsForFmvElection,
                    statutory_basis: "IRC § 1400Z-2(c) — 10-year minimum holding period required for FMV election".to_string(),
                    notes: "VIOLATION: QOF investment held less than 10 years; FMV election under § 1400Z-2(c) unavailable; QOF appreciation gain subject to regular capital gain treatment on sale or exchange.".to_string(),
                    citations,
                    basis_step_up_bps: 0,
                    enhanced_reporting_penalty_exposure_dollars: 0,
                }
            }
        }
        ComplianceAspect::ObbbaRollingFiveYearDeferralPeriod => Output {
            mode: Section1400Z2Mode::CompliantObbbaRollingFiveYearDeferralPeriod,
            statutory_basis: "OBBBA 2025 — rolling 5-year deferral period for gains invested after December 31, 2026".to_string(),
            notes: "COMPLIANT: rolling 5-year deferral period applies under OBBBA 2025 for gains invested in QOFs after December 31, 2026; replaces the original fixed-date December 31, 2026 trigger.".to_string(),
            citations,
            basis_step_up_bps: 0,
            enhanced_reporting_penalty_exposure_dollars: 0,
        },
        ComplianceAspect::ObbbaThirtyYearGainExclusionHorizon => {
            if input.years_since_obbba_investment_initial <= IRC_1400Z2_OBBBA_GAIN_EXCLUSION_HORIZON_YEARS {
                Output {
                    mode: Section1400Z2Mode::CompliantObbbaThirtyYearGainExclusionHorizonRespected,
                    statutory_basis: "OBBBA 2025 — 30-year limit for gain exclusion respected".to_string(),
                    notes: "COMPLIANT: investment within OBBBA 30-year gain exclusion horizon; permanent exclusion benefit preserved.".to_string(),
                    citations,
                    basis_step_up_bps: 0,
                    enhanced_reporting_penalty_exposure_dollars: 0,
                }
            } else {
                Output {
                    mode: Section1400Z2Mode::ViolationObbbaThirtyYearGainExclusionHorizonExceeded,
                    statutory_basis: "OBBBA 2025 — 30-year limit for gain exclusion exceeded".to_string(),
                    notes: "VIOLATION: investment exceeds OBBBA 30-year gain exclusion horizon; permanent exclusion benefit lapses beyond 30-year cap.".to_string(),
                    citations,
                    basis_step_up_bps: 0,
                    enhanced_reporting_penalty_exposure_dollars: 0,
                }
            }
        }
        ComplianceAspect::ObbbaQrofSubstantialImprovementFiftyPctOverThirtyOneMonths => {
            if input.substantial_improvement_pct_bps
                >= IRC_1400Z2_OBBBA_RURAL_SUBSTANTIAL_IMPROVEMENT_PCT_BPS
                && input.substantial_improvement_period_months
                    <= IRC_1400Z2_OBBBA_RURAL_SUBSTANTIAL_IMPROVEMENT_PERIOD_MONTHS
            {
                Output {
                    mode: Section1400Z2Mode::CompliantObbbaQrofSubstantialImprovementMetWithinFiftyPctOverThirtyOneMonths,
                    statutory_basis: "OBBBA 2025 — QROF substantial improvement requirement met at 50 % within 31 months".to_string(),
                    notes: "COMPLIANT: Qualified Rural Opportunity Fund property substantial improvement requirement met — at least 50 % of QROF basis invested in additional property improvements within 31-month window (REDUCED from 100 % over 30 months for regular QOZs).".to_string(),
                    citations,
                    basis_step_up_bps: 0,
                    enhanced_reporting_penalty_exposure_dollars: 0,
                }
            } else {
                Output {
                    mode: Section1400Z2Mode::ViolationObbbaQrofSubstantialImprovementBelowFiftyPctOrPastThirtyOneMonths,
                    statutory_basis: "OBBBA 2025 — QROF substantial improvement requirement not met (50 % over 31 months)".to_string(),
                    notes: "VIOLATION: QROF substantial improvement requirement not met — either below 50 % of QROF basis in additional improvements OR past 31-month window; property does not qualify as QROF qualified property.".to_string(),
                    citations,
                    basis_step_up_bps: 0,
                    enhanced_reporting_penalty_exposure_dollars: 0,
                }
            }
        }
        ComplianceAspect::ObbbaEnhancedReportingRequirements => {
            if input.enhanced_reporting_filed {
                Output {
                    mode: Section1400Z2Mode::CompliantObbbaEnhancedReportingFiled,
                    statutory_basis: "OBBBA 2025 — enhanced reporting requirements satisfied".to_string(),
                    notes: "COMPLIANT: QOF / QROF filed enhanced reporting under OBBBA 2025 requirements; no large-fund $50,000 penalty exposure.".to_string(),
                    citations,
                    basis_step_up_bps: 0,
                    enhanced_reporting_penalty_exposure_dollars: 0,
                }
            } else if input.is_large_fund_subject_to_enhanced_penalty {
                Output {
                    mode: Section1400Z2Mode::ViolationObbbaEnhancedReportingNotFiledLargeFundPenaltyExposure,
                    statutory_basis: "OBBBA 2025 — enhanced reporting not filed; large fund penalty exposure up to $50,000".to_string(),
                    notes: "VIOLATION: QOF / QROF failed to file enhanced reporting under OBBBA 2025 requirements; large fund subject to penalty exposure up to $50,000 per noncompliance event.".to_string(),
                    citations,
                    basis_step_up_bps: 0,
                    enhanced_reporting_penalty_exposure_dollars: IRC_1400Z2_OBBBA_REPORTING_PENALTY_LARGE_FUND_DOLLARS,
                }
            } else {
                Output {
                    mode: Section1400Z2Mode::ViolationObbbaEnhancedReportingNotFiledLargeFundPenaltyExposure,
                    statutory_basis: "OBBBA 2025 — enhanced reporting not filed (small fund)".to_string(),
                    notes: "VIOLATION: QOF / QROF failed to file enhanced reporting under OBBBA 2025 requirements; smaller fund subject to lower penalty exposure (not the $50,000 large-fund cap).".to_string(),
                    citations,
                    basis_step_up_bps: 0,
                    enhanced_reporting_penalty_exposure_dollars: 0,
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
            regime: InvestmentRegime::OriginalTcja2017FrameworkPreDecember31_2026Gain,
            qof_type: QofType::QualifiedOpportunityFundRegular,
            compliance_aspect:
                ComplianceAspect::OneHundredEightyDayReinvestmentWindowUnderSection1400Z2A1,
            days_from_realization_to_reinvestment: 150,
            holding_period_years: 7,
            holding_period_at_fmv_election_years: 10,
            years_since_obbba_investment_initial: 5,
            substantial_improvement_pct_bps: 5_000,
            substantial_improvement_period_months: 31,
            enhanced_reporting_filed: true,
            is_large_fund_subject_to_enhanced_penalty: false,
        }
    }

    #[test]
    fn not_a_qof_not_applicable() {
        let mut input = baseline_input();
        input.qof_type = QofType::NotAQualifiedOpportunityFund;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1400Z2Mode::NotApplicableNotAQualifiedOpportunityFund
        );
    }

    #[test]
    fn within_one_hundred_eighty_day_window_compliant() {
        let output = check(&baseline_input());
        assert_eq!(
            output.mode,
            Section1400Z2Mode::CompliantInvestmentWithin180DayWindow
        );
    }

    #[test]
    fn at_exactly_one_hundred_eighty_day_boundary_compliant() {
        let mut input = baseline_input();
        input.days_from_realization_to_reinvestment = 180;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1400Z2Mode::CompliantInvestmentWithin180DayWindow
        );
    }

    #[test]
    fn past_one_hundred_eighty_day_window_violation() {
        let mut input = baseline_input();
        input.days_from_realization_to_reinvestment = 181;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1400Z2Mode::ViolationFailureToInvestWithin180DayReinvestmentWindow
        );
    }

    #[test]
    fn original_tcja_seven_year_hold_total_fifteen_pct_step_up_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::OriginalTcjaBasisStepUpAtFiveOrSevenYearHold;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1400Z2Mode::CompliantOriginalTcjaFivePctStepUpAtSevenYearHoldTotalFifteenPct
        );
        assert_eq!(output.basis_step_up_bps, 1_500);
    }

    #[test]
    fn original_tcja_five_year_hold_ten_pct_step_up_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::OriginalTcjaBasisStepUpAtFiveOrSevenYearHold;
        input.holding_period_years = 5;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1400Z2Mode::CompliantOriginalTcjaTenPctStepUpAtFiveYearHold
        );
        assert_eq!(output.basis_step_up_bps, 1_000);
    }

    #[test]
    fn original_tcja_under_five_year_hold_no_step_up_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::OriginalTcjaBasisStepUpAtFiveOrSevenYearHold;
        input.holding_period_years = 4;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1400Z2Mode::ViolationFailureToHoldForRequiredYearsForStepUp
        );
        assert_eq!(output.basis_step_up_bps, 0);
    }

    #[test]
    fn obbba_regular_oz_ten_pct_step_up_at_five_year_hold_compliant() {
        let mut input = baseline_input();
        input.regime = InvestmentRegime::ObbbaAmendedRegimePostDecember31_2026Gain;
        input.compliance_aspect = ComplianceAspect::ObbbaBasisStepUpAtFiveYearHold;
        input.holding_period_years = 5;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1400Z2Mode::CompliantObbbaRegularOzTenPctStepUpAtFiveYearHold
        );
        assert_eq!(output.basis_step_up_bps, 1_000);
    }

    #[test]
    fn obbba_rural_qrof_thirty_pct_step_up_at_five_year_hold_compliant() {
        let mut input = baseline_input();
        input.regime = InvestmentRegime::ObbbaAmendedRegimePostDecember31_2026Gain;
        input.qof_type = QofType::QualifiedRuralOpportunityFundQrof;
        input.compliance_aspect = ComplianceAspect::ObbbaBasisStepUpAtFiveYearHold;
        input.holding_period_years = 5;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1400Z2Mode::CompliantObbbaRuralQrofThirtyPctStepUpAtFiveYearHold
        );
        assert_eq!(output.basis_step_up_bps, 3_000);
    }

    #[test]
    fn ten_year_hold_fmv_election_permanent_exclusion_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::TenYearHoldFmvElectionPermanentExclusionUnderSection1400Z2C;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1400Z2Mode::CompliantTenYearHoldFmvElectionPermanentExclusion
        );
        assert_eq!(output.basis_step_up_bps, 10_000);
    }

    #[test]
    fn ten_year_hold_under_ten_years_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::TenYearHoldFmvElectionPermanentExclusionUnderSection1400Z2C;
        input.holding_period_at_fmv_election_years = 9;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1400Z2Mode::ViolationFailureToHoldForTenYearsForFmvElection
        );
    }

    #[test]
    fn obbba_rolling_five_year_deferral_period_compliant() {
        let mut input = baseline_input();
        input.regime = InvestmentRegime::ObbbaAmendedRegimePostDecember31_2026Gain;
        input.compliance_aspect = ComplianceAspect::ObbbaRollingFiveYearDeferralPeriod;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1400Z2Mode::CompliantObbbaRollingFiveYearDeferralPeriod
        );
    }

    #[test]
    fn obbba_thirty_year_gain_exclusion_horizon_compliant() {
        let mut input = baseline_input();
        input.regime = InvestmentRegime::ObbbaAmendedRegimePostDecember31_2026Gain;
        input.compliance_aspect = ComplianceAspect::ObbbaThirtyYearGainExclusionHorizon;
        input.years_since_obbba_investment_initial = 25;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1400Z2Mode::CompliantObbbaThirtyYearGainExclusionHorizonRespected
        );
    }

    #[test]
    fn obbba_thirty_year_gain_exclusion_horizon_exceeded_violation() {
        let mut input = baseline_input();
        input.regime = InvestmentRegime::ObbbaAmendedRegimePostDecember31_2026Gain;
        input.compliance_aspect = ComplianceAspect::ObbbaThirtyYearGainExclusionHorizon;
        input.years_since_obbba_investment_initial = 31;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1400Z2Mode::ViolationObbbaThirtyYearGainExclusionHorizonExceeded
        );
    }

    #[test]
    fn obbba_qrof_substantial_improvement_at_fifty_pct_over_thirty_one_months_compliant() {
        let mut input = baseline_input();
        input.regime = InvestmentRegime::ObbbaAmendedRegimePostDecember31_2026Gain;
        input.qof_type = QofType::QualifiedRuralOpportunityFundQrof;
        input.compliance_aspect =
            ComplianceAspect::ObbbaQrofSubstantialImprovementFiftyPctOverThirtyOneMonths;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1400Z2Mode::CompliantObbbaQrofSubstantialImprovementMetWithinFiftyPctOverThirtyOneMonths
        );
    }

    #[test]
    fn obbba_qrof_substantial_improvement_below_fifty_pct_violation() {
        let mut input = baseline_input();
        input.regime = InvestmentRegime::ObbbaAmendedRegimePostDecember31_2026Gain;
        input.qof_type = QofType::QualifiedRuralOpportunityFundQrof;
        input.compliance_aspect =
            ComplianceAspect::ObbbaQrofSubstantialImprovementFiftyPctOverThirtyOneMonths;
        input.substantial_improvement_pct_bps = 4_999;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1400Z2Mode::ViolationObbbaQrofSubstantialImprovementBelowFiftyPctOrPastThirtyOneMonths
        );
    }

    #[test]
    fn obbba_enhanced_reporting_filed_compliant() {
        let mut input = baseline_input();
        input.regime = InvestmentRegime::ObbbaAmendedRegimePostDecember31_2026Gain;
        input.compliance_aspect = ComplianceAspect::ObbbaEnhancedReportingRequirements;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1400Z2Mode::CompliantObbbaEnhancedReportingFiled
        );
    }

    #[test]
    fn obbba_enhanced_reporting_not_filed_large_fund_penalty_violation() {
        let mut input = baseline_input();
        input.regime = InvestmentRegime::ObbbaAmendedRegimePostDecember31_2026Gain;
        input.compliance_aspect = ComplianceAspect::ObbbaEnhancedReportingRequirements;
        input.enhanced_reporting_filed = false;
        input.is_large_fund_subject_to_enhanced_penalty = true;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1400Z2Mode::ViolationObbbaEnhancedReportingNotFiledLargeFundPenaltyExposure
        );
        assert_eq!(output.enhanced_reporting_penalty_exposure_dollars, 50_000);
    }

    #[test]
    fn constants_pin_statutory_facts() {
        assert_eq!(IRC_1400Z2_TCJA_ENACTMENT_DATE_YEAR, 2017);
        assert_eq!(IRC_1400Z2_TCJA_ENACTMENT_DATE_MONTH, 12);
        assert_eq!(IRC_1400Z2_TCJA_ENACTMENT_DATE_DAY, 22);
        assert_eq!(IRC_1400Z2_TCJA_PUBLIC_LAW_CONGRESS, 115);
        assert_eq!(IRC_1400Z2_TCJA_PUBLIC_LAW_ENACTMENT_NUMBER, 97);
        assert_eq!(IRC_1400Z2_TCJA_ENABLING_SECTION_NUMBER, 13823);
        assert_eq!(IRC_1400Z2_REINVESTMENT_WINDOW_DAYS, 180);
        assert_eq!(IRC_1400Z2_ORIGINAL_DEFERRAL_END_DATE_YEAR, 2026);
        assert_eq!(IRC_1400Z2_ORIGINAL_DEFERRAL_END_DATE_MONTH, 12);
        assert_eq!(IRC_1400Z2_ORIGINAL_DEFERRAL_END_DATE_DAY, 31);
        assert_eq!(IRC_1400Z2_ORIGINAL_FIVE_YEAR_STEP_UP_BPS, 1_000);
        assert_eq!(IRC_1400Z2_ORIGINAL_SEVEN_YEAR_ADDITIONAL_STEP_UP_BPS, 500);
        assert_eq!(IRC_1400Z2_ORIGINAL_COMBINED_FIFTEEN_PCT_STEP_UP_BPS, 1_500);
        assert_eq!(IRC_1400Z2_TEN_YEAR_HOLDING_PERIOD_YEARS, 10);
        assert_eq!(IRC_1400Z2_OBBBA_ENACTMENT_DATE_YEAR, 2025);
        assert_eq!(IRC_1400Z2_OBBBA_ENACTMENT_DATE_MONTH, 7);
        assert_eq!(IRC_1400Z2_OBBBA_ENACTMENT_DATE_DAY, 4);
        assert_eq!(IRC_1400Z2_OBBBA_ROLLING_DEFERRAL_PERIOD_YEARS, 5);
        assert_eq!(IRC_1400Z2_OBBBA_REGULAR_OZ_STEP_UP_BPS, 1_000);
        assert_eq!(IRC_1400Z2_OBBBA_RURAL_QROF_STEP_UP_BPS, 3_000);
        assert_eq!(IRC_1400Z2_OBBBA_GAIN_EXCLUSION_HORIZON_YEARS, 30);
        assert_eq!(
            IRC_1400Z2_OBBBA_REPORTING_PENALTY_LARGE_FUND_DOLLARS,
            50_000
        );
        assert_eq!(IRC_1400Z2_OBBBA_NEW_DESIGNATIONS_EFFECTIVE_DATE_YEAR, 2027);
        assert_eq!(IRC_1400Z2_OBBBA_NEW_DESIGNATIONS_EFFECTIVE_DATE_MONTH, 1);
        assert_eq!(IRC_1400Z2_OBBBA_NEW_DESIGNATIONS_EFFECTIVE_DATE_DAY, 1);
        assert_eq!(
            IRC_1400Z2_OBBBA_NEW_DESIGNATIONS_WINDOW_START_DATE_YEAR,
            2026
        );
        assert_eq!(IRC_1400Z2_OBBBA_NEW_DESIGNATIONS_WINDOW_START_DATE_MONTH, 7);
        assert_eq!(IRC_1400Z2_OBBBA_NEW_DESIGNATIONS_WINDOW_START_DATE_DAY, 1);
        assert_eq!(IRC_1400Z2_OBBBA_NEW_DESIGNATIONS_WINDOW_DAYS, 90);
        assert_eq!(IRC_1400Z2_OBBBA_QROF_ASSET_INVESTMENT_THRESHOLD_BPS, 9_000);
        assert_eq!(
            IRC_1400Z2_OBBBA_RURAL_SUBSTANTIAL_IMPROVEMENT_PCT_BPS,
            5_000
        );
        assert_eq!(
            IRC_1400Z2_OBBBA_REGULAR_SUBSTANTIAL_IMPROVEMENT_PCT_BPS,
            10_000
        );
        assert_eq!(
            IRC_1400Z2_OBBBA_RURAL_SUBSTANTIAL_IMPROVEMENT_PERIOD_MONTHS,
            31
        );
        assert_eq!(IRC_1400Z2_OBBBA_GOVERNOR_REDESIGNATION_CYCLE_YEARS, 10);
        assert_eq!(IRC_1400Z2_BASIS_POINT_DENOMINATOR, 10_000);
    }

    #[test]
    fn citations_contain_landmarks() {
        let output = check(&baseline_input());
        let joined = output.citations.join("\n");
        assert!(joined.contains("§ 1400Z-2"));
        assert!(joined.contains("§ 1400Z-1"));
        assert!(joined.contains("Section 13823 of the Tax Cuts and Jobs Act of 2017"));
        assert!(joined.contains("Public Law 115-97"));
        assert!(joined.contains("131 Stat. 2054"));
        assert!(joined.contains("December 22, 2017"));
        assert!(joined.contains("180-DAY PERIOD"));
        assert!(joined.contains("DECEMBER 31, 2026"));
        assert!(joined.contains("§ 1400Z-2(a)(1)"));
        assert!(joined.contains("§ 1400Z-2(b)(1)"));
        assert!(joined.contains("§ 1400Z-2(b)(2)(B)"));
        assert!(joined.contains("§ 1400Z-2(c)"));
        assert!(joined.contains("AT LEAST 10 YEARS"));
        assert!(joined.contains("FAIR MARKET VALUE"));
        assert!(joined.contains("One Big Beautiful Bill Act of 2025"));
        assert!(joined.contains("JULY 4, 2025"));
        assert!(joined.contains("INDEFINITE INVESTMENT"));
        assert!(joined.contains("90-DAY PERIOD"));
        assert!(joined.contains("JULY 1, 2026"));
        assert!(joined.contains("JANUARY 1, 2027"));
        assert!(joined.contains("5-YEAR deferral"));
        assert!(joined.contains("10 % basis step-up"));
        assert!(joined.contains("30 % basis step-up"));
        assert!(joined.contains("30-YEAR LIMIT"));
        assert!(joined.contains("Qualified Rural Opportunity Funds (QROFs)"));
        assert!(joined.contains("50 %"));
        assert!(joined.contains("31 MONTHS"));
        assert!(joined.contains("$50,000"));
        assert!(joined.contains("T.D. 9889"));
    }
}
