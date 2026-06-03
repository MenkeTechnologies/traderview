//! IRC § 162(e) Denial of Deduction for Lobbying and
//! Political Expenditures Compliance Module — pure-compute
//! check for trader-business compliance with the federal
//! disallowance of business deductions for lobbying and
//! political expenditures, substantially expanded by the
//! Tax Cuts and Jobs Act of 2017 to eliminate the
//! pre-TCJA local-legislation lobbying exception.
//!
//! IRC § 162(e) denies deduction for amounts paid or
//! incurred in connection with **(A) influencing
//! legislation**; **(B) participating or intervening in
//! any political campaign**; **(C) attempting to influence
//! the general public regarding elections, legislative
//! matters, or referendums** (grassroots lobbying); or
//! **(D) any direct communication with a covered executive
//! branch official** in an attempt to influence the
//! person's official actions or positions. **Tax Cuts and
//! Jobs Act of 2017 Section 13308** struck paragraphs (2)
//! (local-legislation exception) and (7) (Indian tribal
//! government special rule); redesignated paragraphs
//! (3)-(6) and (8) as (2)-(5) and (6); effective for
//! amounts paid or incurred after **December 22, 2017**.
//! The de minimis **$2,000 in-house lobbying exception**
//! under § 162(e)(5)(B) (redesignated post-TCJA) was
//! RETAINED.
//!
//! Web research (verified 2026-06-03):
//! - **TCJA 2017 § 13308**: Section 13308(a) of the Tax Cuts and Jobs Act (Public Law 115-97, 131 Stat. 2054) amended IRC § 162(e)(2)-(8) by **striking paragraphs (2) and (7)** and redesignating paragraphs (3), (4), (5), (6), and (8) as paragraphs (2), (3), (4), (5), and (6), respectively; signed by President Donald Trump on **December 22, 2017**; effective for amounts paid or incurred after enactment date ([IRC Code Sec. 162 (Trade or Business Expenses) — Tax Notes](https://www.taxnotes.com/research/federal/usc26/162); [IRS — Topic B Disallowance of Deduction Under IRC 162 for Lobbying Expenses](https://www.irs.gov/pub/irs-tege/eotopicb95.pdf); [Cornell LII — 26 U.S. Code § 162](https://www.law.cornell.edu/uscode/text/26/162); [FORVIS — New Tax Law Nixes Deduction for Local Lobbying Expenses (May 2018)](https://www.forvis.com/article/2018/05/new-tax-law-nixes-deduction-local-lobbying-expenses); [CCH AnswerConnect — § 162(e) Denial of Deduction for Certain Lobbying and Political Expenditures](https://answerconnect.cch.com/document/arp1209013e2c83dc4408SPLIT162e/federal/irc/current/denial-of-deduction-for-certain-lobbying-and-political-expenditures); [IRS — Nondeductible Lobbying and Political Expenditures (Charities and Non-Profits)](https://www.irs.gov/charities-non-profits/other-non-profits/nondeductible-lobbying-and-political-expenditures); [Every CRS Report — R42381 Deductibility of Corporate Campaign Expenditures](https://www.everycrsreport.com/reports/R42381.html); [Wiley — Tax Corner: Nondeductible Lobbying and Political Expenses Under the IRC](https://www.wiley.law/newsletter-2718); [Uncle Kam — Understanding the Non-Deductibility of Lobbying Expenses](https://unclekam.com/tax-write-offs/deductions/lobbying-expenses-deductibility/); [FindLaw — 26 USC § 162](https://codes.findlaw.com/us/title-26-internal-revenue-code/26-usc-sect-162/); [Bloomberg Tax — Sec. 162 Trade or Business Expenses](https://irc.bloombergtax.com/public/uscode/doc/irc/section_162); [House Office of Law Revision Counsel — 26 USC § 162](https://uscode.house.gov/view.xhtml?req=%28title%3A26+section%3A162+edition%3Aprelim%29); [IRS — Notice 1333 Nondeductible Lobbying and Political Expenditures](https://www.irs.gov/pub/irs-tege/notice_1333.pdf)).
//! - **§ 162(e)(1)(A) Influencing Legislation**: any attempt to influence any legislation through communication with any member or employee of a legislative body, or with any government official or employee who may participate in the formulation of legislation.
//! - **§ 162(e)(1)(B) Political Campaign**: participation in, or intervention in, any political campaign on behalf of (or in opposition to) any candidate for public office.
//! - **§ 162(e)(1)(C) General Public (Grassroots Lobbying)**: any attempt to influence the general public, or segments thereof, with respect to elections, legislative matters, or referendums.
//! - **§ 162(e)(1)(D) Covered Executive Branch Official**: any direct communication with a covered executive branch official in an attempt to influence the official actions or positions of such official.
//! - **§ 162(e)(4) Covered Executive Branch Official Definition (post-TCJA redesignated § 162(e)(3))**: any officer or employee of the **WHITE HOUSE OFFICE** of the Executive Office of the President; the **2 MOST SENIOR LEVEL OFFICERS of each of the other agencies in the Executive Office**; any individual serving in a position in **LEVEL I OF THE EXECUTIVE SCHEDULE**; any other individual designated by the President as having **CABINET LEVEL STATUS**; and any **IMMEDIATE DEPUTY** of an individual in those categories.
//! - **§ 162(e)(5)(B) De Minimis In-House Lobbying Exception**: if a taxpayer's total amount of **IN-HOUSE LOBBYING EXPENDITURES** for the taxable year does not exceed **$2,000**, then such expenditures are **DEDUCTIBLE**; this exception does NOT apply to amounts paid to **PROFESSIONAL LOBBYISTS** or **DUES PAID TO ORGANIZATIONS** that engage in lobbying. The de minimis exception was RETAINED by TCJA 2017.
//! - **TCJA Repeal of Local Legislation Exception (Pre-TCJA § 162(e)(2))**: PRE-TCJA, lobbying expenses with respect to legislation of any **LOCAL COUNCIL OR SIMILAR GOVERNING BODY** were deductible under former § 162(e)(2); **TCJA 2017 STRUCK this paragraph entirely**, making lobbying expenses related to LOCAL legislation NON-DEDUCTIBLE effective **December 22, 2017**.
//! - **TCJA Repeal of Indian Tribal Government Special Rule (Pre-TCJA § 162(e)(7))**: special rule treating Indian tribal governments as similar to state and local governments for § 162(e) purposes was also STRUCK by TCJA 2017.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const IRC_162E_TCJA_ENACTMENT_DATE_YEAR: u32 = 2017;
pub const IRC_162E_TCJA_ENACTMENT_DATE_MONTH: u32 = 12;
pub const IRC_162E_TCJA_ENACTMENT_DATE_DAY: u32 = 22;
pub const IRC_162E_TCJA_PUBLIC_LAW_CONGRESS: u32 = 115;
pub const IRC_162E_TCJA_PUBLIC_LAW_ENACTMENT: u32 = 97;
pub const IRC_162E_TCJA_ENABLING_SECTION: u32 = 13308;
pub const IRC_162E_TCJA_STAT_VOLUME: u32 = 131;
pub const IRC_162E_TCJA_STAT_PAGE: u32 = 2054;
pub const IRC_162E_DE_MINIMIS_IN_HOUSE_LOBBYING_EXCEPTION_DOLLARS: u64 = 2_000;
pub const IRC_162E_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExpenditureCategory {
    InfluencingLegislationUnderSection162E1A,
    PoliticalCampaignParticipationOrInterventionUnderSection162E1B,
    GrassrootsLobbyingUnderSection162E1C,
    DirectCommunicationWithCoveredExecutiveBranchOfficialUnderSection162E1D,
    LocalLegislationLobbyingPostTcjaRepealedExceptionNoLongerDeductible,
    NonLobbyingNonPoliticalExpenditureOrdinaryBusinessExpense,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExpenditureSource {
    InHouseLobbyingExpenditureExclusiveOfProfessionalLobbyistOrDues,
    PaidToProfessionalLobbyist,
    DuesPaidToOrganizationThatLobbies,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TaxYearStatus {
    TaxYearBeginningAfterDecember22_2017PostTcja,
    TaxYearBeginningOnOrBeforeDecember22_2017PreTcja,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAspect {
    LobbyingExpenseDeductibilityDetermination,
    DeMinimisInHouseLobbyingExceptionUnderSection162E5B,
    LocalLegislationPostTcjaTreatment,
    CoveredExecutiveBranchOfficialDefinition,
    PoliticalCampaignParticipationDisallowance,
    GrassrootsLobbyingDisallowance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section162EMode {
    NotApplicableNonLobbyingNonPoliticalOrdinaryBusinessExpense,
    NotApplicablePreTcjaTaxYear,
    CompliantInHouseLobbyingAtOrBelow2000DeMinimisExceptionDeductible,
    CompliantOrdinaryBusinessExpenseNotSubjectToSection162E,
    ViolationInfluencingLegislationNonDeductibleUnderSection162E1A,
    ViolationPoliticalCampaignParticipationNonDeductibleUnderSection162E1B,
    ViolationGrassrootsLobbyingNonDeductibleUnderSection162E1C,
    ViolationDirectCommunicationWithCoveredExecutiveBranchOfficialNonDeductibleUnderSection162E1D,
    ViolationLocalLegislationLobbyingNonDeductiblePostTcjaRepeal,
    ViolationInHouseLobbyingExceeds2000DeMinimisExceptionNonDeductible,
    ViolationProfessionalLobbyistPaymentNotEligibleForDeMinimisException,
    ViolationDuesToOrganizationLobbyingNotEligibleForDeMinimisException,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub tax_year_status: TaxYearStatus,
    pub expenditure_category: ExpenditureCategory,
    pub expenditure_source: ExpenditureSource,
    pub compliance_aspect: ComplianceAspect,
    pub expenditure_amount_dollars: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section162EMode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
    pub disallowed_amount_dollars: u64,
}

pub type Section162EInput = Input;
pub type Section162EOutput = Output;
pub type Section162EResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "IRC § 162(e) Denial of Deduction for Lobbying and Political Expenditures — substantially expanded by Tax Cuts and Jobs Act of 2017 Section 13308 (Public Law 115-97, 131 Stat. 2054); signed by President Donald Trump on December 22, 2017".to_string(),
        "IRC § 162(e)(1) Four Categories of Non-Deductible Lobbying and Political Expenditures — (A) influencing legislation; (B) participating or intervening in any political campaign on behalf of (or in opposition to) any candidate for public office; (C) attempting to influence the general public, or segments thereof, with respect to elections, legislative matters, or referendums (grassroots lobbying); (D) any direct communication with a covered executive branch official in an attempt to influence the official actions or positions of such official".to_string(),
        "IRC § 162(e)(1)(A) Influencing Legislation — any attempt to influence any legislation through communication with any member or employee of a legislative body, or with any government official or employee who may participate in the formulation of legislation".to_string(),
        "IRC § 162(e)(4) (post-TCJA redesignated as § 162(e)(3)) Covered Executive Branch Official Definition — any officer or employee of the WHITE HOUSE OFFICE of the Executive Office of the President; the 2 MOST SENIOR LEVEL OFFICERS of each of the other agencies in the Executive Office; any individual serving in a position in LEVEL I OF THE EXECUTIVE SCHEDULE; any other individual designated by the President as having CABINET LEVEL STATUS; and any IMMEDIATE DEPUTY of an individual in those categories".to_string(),
        "IRC § 162(e)(5)(B) De Minimis In-House Lobbying Exception — if a taxpayer's total amount of IN-HOUSE LOBBYING EXPENDITURES for the taxable year does NOT exceed $2,000, then such expenditures are DEDUCTIBLE; this exception does NOT apply to amounts paid to PROFESSIONAL LOBBYISTS or DUES PAID TO ORGANIZATIONS that engage in lobbying. The de minimis exception was RETAINED by TCJA 2017.".to_string(),
        "TCJA 2017 § 13308(a) Repeal of Local Legislation Exception — Tax Cuts and Jobs Act of 2017 § 13308(a) amended IRC § 162(e)(2)-(8) by STRIKING paragraphs (2) and (7) and redesignating paragraphs (3), (4), (5), (6), and (8) as paragraphs (2), (3), (4), (5), and (6), respectively".to_string(),
        "TCJA Repeal of Pre-TCJA § 162(e)(2) Local Legislation Exception — PRE-TCJA, lobbying expenses with respect to legislation of any LOCAL COUNCIL OR SIMILAR GOVERNING BODY were deductible under former § 162(e)(2); TCJA 2017 STRUCK this paragraph entirely, making lobbying expenses related to LOCAL legislation NON-DEDUCTIBLE effective December 22, 2017".to_string(),
        "TCJA Repeal of Pre-TCJA § 162(e)(7) Indian Tribal Government Special Rule — special rule treating Indian tribal governments as similar to state and local governments for § 162(e) purposes was also STRUCK by TCJA 2017".to_string(),
        "TCJA Effective Date — TCJA § 13308 amendments effective for amounts paid or incurred after December 22, 2017".to_string(),
        "IRS Publication on Disallowance of Deduction Under IRC 162 for Lobbying Expenses (Topic B EO Topic Document) — practitioner guidance".to_string(),
        "IRS Notice 1333 — Nondeductible Lobbying and Political Expenditures procedural guidance".to_string(),
        "Tax Notes + Bloomberg Tax + Cornell LII + FORVIS + CCH AnswerConnect + Every CRS Report + Wiley + Uncle Kam + FindLaw + House Office of Law Revision Counsel — practitioner overviews of § 162(e)".to_string(),
    ];

    if input.expenditure_category == ExpenditureCategory::NonLobbyingNonPoliticalExpenditureOrdinaryBusinessExpense {
        return Output {
            mode: Section162EMode::NotApplicableNonLobbyingNonPoliticalOrdinaryBusinessExpense,
            statutory_basis: "IRC § 162(a) — ordinary business expense not subject to § 162(e) lobbying / political disallowance".to_string(),
            notes: "NOT APPLICABLE: expenditure is not lobbying or political under § 162(e); ordinary business expense fully deductible under § 162(a).".to_string(),
            citations,
            disallowed_amount_dollars: 0,
        };
    }

    if input.tax_year_status == TaxYearStatus::TaxYearBeginningOnOrBeforeDecember22_2017PreTcja {
        return Output {
            mode: Section162EMode::NotApplicablePreTcjaTaxYear,
            statutory_basis: "TCJA 2017 § 13308 effective date — amendments effective for amounts paid or incurred after December 22, 2017".to_string(),
            notes: "NOT APPLICABLE: tax year begins on or before December 22, 2017 (pre-TCJA); pre-TCJA § 162(e) framework applies (local-legislation exception under former § 162(e)(2) and Indian tribal government special rule under former § 162(e)(7) remain available; analyze under pre-TCJA statutory framework).".to_string(),
            citations,
            disallowed_amount_dollars: 0,
        };
    }

    match input.compliance_aspect {
        ComplianceAspect::LobbyingExpenseDeductibilityDetermination => {
            match input.expenditure_category {
                ExpenditureCategory::InfluencingLegislationUnderSection162E1A => Output {
                    mode: Section162EMode::ViolationInfluencingLegislationNonDeductibleUnderSection162E1A,
                    statutory_basis: "IRC § 162(e)(1)(A) — influencing legislation expenditure non-deductible".to_string(),
                    notes: "VIOLATION: expenditure attempting to influence legislation through communication with member or employee of legislative body OR with government official or employee who may participate in formulation of legislation is NON-DEDUCTIBLE under § 162(e)(1)(A); de minimis exception under § 162(e)(5)(B) may apply for in-house lobbying ≤ $2,000.".to_string(),
                    citations,
                    disallowed_amount_dollars: input.expenditure_amount_dollars,
                },
                ExpenditureCategory::PoliticalCampaignParticipationOrInterventionUnderSection162E1B => Output {
                    mode: Section162EMode::ViolationPoliticalCampaignParticipationNonDeductibleUnderSection162E1B,
                    statutory_basis: "IRC § 162(e)(1)(B) — political campaign participation or intervention non-deductible".to_string(),
                    notes: "VIOLATION: participation in or intervention in any political campaign on behalf of (or in opposition to) any candidate for public office is NON-DEDUCTIBLE under § 162(e)(1)(B); de minimis exception under § 162(e)(5)(B) does NOT apply to political campaign expenditures.".to_string(),
                    citations,
                    disallowed_amount_dollars: input.expenditure_amount_dollars,
                },
                ExpenditureCategory::GrassrootsLobbyingUnderSection162E1C => Output {
                    mode: Section162EMode::ViolationGrassrootsLobbyingNonDeductibleUnderSection162E1C,
                    statutory_basis: "IRC § 162(e)(1)(C) — grassroots lobbying (attempting to influence general public) non-deductible".to_string(),
                    notes: "VIOLATION: any attempt to influence general public, or segments thereof, with respect to elections, legislative matters, or referendums (grassroots lobbying) is NON-DEDUCTIBLE under § 162(e)(1)(C).".to_string(),
                    citations,
                    disallowed_amount_dollars: input.expenditure_amount_dollars,
                },
                ExpenditureCategory::DirectCommunicationWithCoveredExecutiveBranchOfficialUnderSection162E1D => Output {
                    mode: Section162EMode::ViolationDirectCommunicationWithCoveredExecutiveBranchOfficialNonDeductibleUnderSection162E1D,
                    statutory_basis: "IRC § 162(e)(1)(D) — direct communication with covered executive branch official non-deductible".to_string(),
                    notes: "VIOLATION: any direct communication with covered executive branch official (White House Office officer / employee; 2 most senior officers of other Executive Office agencies; Level I Executive Schedule; Cabinet-level designee; immediate deputies) in attempt to influence official actions or positions is NON-DEDUCTIBLE under § 162(e)(1)(D).".to_string(),
                    citations,
                    disallowed_amount_dollars: input.expenditure_amount_dollars,
                },
                ExpenditureCategory::LocalLegislationLobbyingPostTcjaRepealedExceptionNoLongerDeductible => Output {
                    mode: Section162EMode::ViolationLocalLegislationLobbyingNonDeductiblePostTcjaRepeal,
                    statutory_basis: "TCJA 2017 § 13308(a) — local legislation exception under former § 162(e)(2) repealed; local lobbying now non-deductible".to_string(),
                    notes: "VIOLATION: lobbying expenses with respect to legislation of any LOCAL COUNCIL OR SIMILAR GOVERNING BODY are NO LONGER DEDUCTIBLE post-TCJA (effective December 22, 2017); TCJA § 13308(a) STRUCK former § 162(e)(2) local legislation exception entirely.".to_string(),
                    citations,
                    disallowed_amount_dollars: input.expenditure_amount_dollars,
                },
                ExpenditureCategory::NonLobbyingNonPoliticalExpenditureOrdinaryBusinessExpense => unreachable!(),
            }
        }
        ComplianceAspect::DeMinimisInHouseLobbyingExceptionUnderSection162E5B => {
            match input.expenditure_source {
                ExpenditureSource::PaidToProfessionalLobbyist => Output {
                    mode: Section162EMode::ViolationProfessionalLobbyistPaymentNotEligibleForDeMinimisException,
                    statutory_basis: "IRC § 162(e)(5)(B) — de minimis exception does NOT apply to amounts paid to professional lobbyists".to_string(),
                    notes: "VIOLATION: amount paid to PROFESSIONAL LOBBYIST is NOT eligible for de minimis $2,000 in-house exception under § 162(e)(5)(B); full amount is non-deductible.".to_string(),
                    citations,
                    disallowed_amount_dollars: input.expenditure_amount_dollars,
                },
                ExpenditureSource::DuesPaidToOrganizationThatLobbies => Output {
                    mode: Section162EMode::ViolationDuesToOrganizationLobbyingNotEligibleForDeMinimisException,
                    statutory_basis: "IRC § 162(e)(5)(B) — de minimis exception does NOT apply to dues paid to organizations that lobby".to_string(),
                    notes: "VIOLATION: DUES PAID TO ORGANIZATIONS that engage in lobbying are NOT eligible for de minimis $2,000 in-house exception under § 162(e)(5)(B); portion of dues attributable to lobbying is non-deductible.".to_string(),
                    citations,
                    disallowed_amount_dollars: input.expenditure_amount_dollars,
                },
                ExpenditureSource::InHouseLobbyingExpenditureExclusiveOfProfessionalLobbyistOrDues => {
                    if input.expenditure_amount_dollars <= IRC_162E_DE_MINIMIS_IN_HOUSE_LOBBYING_EXCEPTION_DOLLARS {
                        Output {
                            mode: Section162EMode::CompliantInHouseLobbyingAtOrBelow2000DeMinimisExceptionDeductible,
                            statutory_basis: "IRC § 162(e)(5)(B) — in-house lobbying expenditure at or below $2,000 de minimis exception".to_string(),
                            notes: "COMPLIANT: in-house lobbying expenditure at or below $2,000 statutory de minimis exception under § 162(e)(5)(B); fully DEDUCTIBLE as ordinary business expense.".to_string(),
                            citations,
                            disallowed_amount_dollars: 0,
                        }
                    } else {
                        Output {
                            mode: Section162EMode::ViolationInHouseLobbyingExceeds2000DeMinimisExceptionNonDeductible,
                            statutory_basis: "IRC § 162(e)(5)(B) — in-house lobbying expenditure exceeds $2,000 de minimis exception".to_string(),
                            notes: "VIOLATION: in-house lobbying expenditure exceeds $2,000 de minimis exception under § 162(e)(5)(B); ENTIRE amount (not just excess over $2,000) is non-deductible.".to_string(),
                            citations,
                            disallowed_amount_dollars: input.expenditure_amount_dollars,
                        }
                    }
                }
            }
        }
        ComplianceAspect::LocalLegislationPostTcjaTreatment => Output {
            mode: Section162EMode::ViolationLocalLegislationLobbyingNonDeductiblePostTcjaRepeal,
            statutory_basis: "TCJA 2017 § 13308(a) — local legislation lobbying non-deductible post-TCJA".to_string(),
            notes: "INFORMATIONAL: post-TCJA, lobbying expenses with respect to LOCAL legislation are NON-DEDUCTIBLE under § 162(e) generally; pre-TCJA exception under former § 162(e)(2) has been REPEALED.".to_string(),
            citations,
            disallowed_amount_dollars: input.expenditure_amount_dollars,
        },
        ComplianceAspect::CoveredExecutiveBranchOfficialDefinition => Output {
            mode: Section162EMode::ViolationDirectCommunicationWithCoveredExecutiveBranchOfficialNonDeductibleUnderSection162E1D,
            statutory_basis: "IRC § 162(e)(4) — covered executive branch official definition".to_string(),
            notes: "INFORMATIONAL: covered executive branch official = (i) White House Office officer or employee; (ii) 2 most senior officers of other Executive Office agencies; (iii) Level I Executive Schedule; (iv) Cabinet-level designee; (v) immediate deputies of any of the foregoing.".to_string(),
            citations,
            disallowed_amount_dollars: input.expenditure_amount_dollars,
        },
        ComplianceAspect::PoliticalCampaignParticipationDisallowance => Output {
            mode: Section162EMode::ViolationPoliticalCampaignParticipationNonDeductibleUnderSection162E1B,
            statutory_basis: "IRC § 162(e)(1)(B) — political campaign participation disallowance".to_string(),
            notes: "INFORMATIONAL: participation in or intervention in any political campaign on behalf of (or in opposition to) any candidate for public office is non-deductible under § 162(e)(1)(B); de minimis exception does NOT apply to political campaign expenditures.".to_string(),
            citations,
            disallowed_amount_dollars: input.expenditure_amount_dollars,
        },
        ComplianceAspect::GrassrootsLobbyingDisallowance => Output {
            mode: Section162EMode::ViolationGrassrootsLobbyingNonDeductibleUnderSection162E1C,
            statutory_basis: "IRC § 162(e)(1)(C) — grassroots lobbying disallowance".to_string(),
            notes: "INFORMATIONAL: attempting to influence general public, or segments thereof, with respect to elections, legislative matters, or referendums (grassroots lobbying) is non-deductible under § 162(e)(1)(C).".to_string(),
            citations,
            disallowed_amount_dollars: input.expenditure_amount_dollars,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_input() -> Input {
        Input {
            tax_year_status: TaxYearStatus::TaxYearBeginningAfterDecember22_2017PostTcja,
            expenditure_category: ExpenditureCategory::InfluencingLegislationUnderSection162E1A,
            expenditure_source: ExpenditureSource::InHouseLobbyingExpenditureExclusiveOfProfessionalLobbyistOrDues,
            compliance_aspect: ComplianceAspect::LobbyingExpenseDeductibilityDetermination,
            expenditure_amount_dollars: 5_000,
        }
    }

    #[test]
    fn non_lobbying_ordinary_expense_not_applicable() {
        let mut input = baseline_input();
        input.expenditure_category = ExpenditureCategory::NonLobbyingNonPoliticalExpenditureOrdinaryBusinessExpense;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section162EMode::NotApplicableNonLobbyingNonPoliticalOrdinaryBusinessExpense
        );
    }

    #[test]
    fn pre_tcja_tax_year_not_applicable() {
        let mut input = baseline_input();
        input.tax_year_status = TaxYearStatus::TaxYearBeginningOnOrBeforeDecember22_2017PreTcja;
        let output = check(&input);
        assert_eq!(output.mode, Section162EMode::NotApplicablePreTcjaTaxYear);
    }

    #[test]
    fn influencing_legislation_violation_under_162e1a() {
        let output = check(&baseline_input());
        assert_eq!(
            output.mode,
            Section162EMode::ViolationInfluencingLegislationNonDeductibleUnderSection162E1A
        );
        assert_eq!(output.disallowed_amount_dollars, 5_000);
    }

    #[test]
    fn political_campaign_violation_under_162e1b() {
        let mut input = baseline_input();
        input.expenditure_category = ExpenditureCategory::PoliticalCampaignParticipationOrInterventionUnderSection162E1B;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section162EMode::ViolationPoliticalCampaignParticipationNonDeductibleUnderSection162E1B
        );
    }

    #[test]
    fn grassroots_lobbying_violation_under_162e1c() {
        let mut input = baseline_input();
        input.expenditure_category = ExpenditureCategory::GrassrootsLobbyingUnderSection162E1C;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section162EMode::ViolationGrassrootsLobbyingNonDeductibleUnderSection162E1C
        );
    }

    #[test]
    fn covered_executive_branch_communication_violation_under_162e1d() {
        let mut input = baseline_input();
        input.expenditure_category =
            ExpenditureCategory::DirectCommunicationWithCoveredExecutiveBranchOfficialUnderSection162E1D;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section162EMode::ViolationDirectCommunicationWithCoveredExecutiveBranchOfficialNonDeductibleUnderSection162E1D
        );
    }

    #[test]
    fn local_legislation_lobbying_violation_post_tcja_repeal() {
        let mut input = baseline_input();
        input.expenditure_category =
            ExpenditureCategory::LocalLegislationLobbyingPostTcjaRepealedExceptionNoLongerDeductible;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section162EMode::ViolationLocalLegislationLobbyingNonDeductiblePostTcjaRepeal
        );
    }

    #[test]
    fn de_minimis_in_house_at_or_below_2000_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::DeMinimisInHouseLobbyingExceptionUnderSection162E5B;
        input.expenditure_amount_dollars = 2_000;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section162EMode::CompliantInHouseLobbyingAtOrBelow2000DeMinimisExceptionDeductible
        );
        assert_eq!(output.disallowed_amount_dollars, 0);
    }

    #[test]
    fn de_minimis_in_house_at_exactly_2000_boundary_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::DeMinimisInHouseLobbyingExceptionUnderSection162E5B;
        input.expenditure_amount_dollars = 2_000;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section162EMode::CompliantInHouseLobbyingAtOrBelow2000DeMinimisExceptionDeductible
        );
    }

    #[test]
    fn de_minimis_in_house_at_2001_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::DeMinimisInHouseLobbyingExceptionUnderSection162E5B;
        input.expenditure_amount_dollars = 2_001;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section162EMode::ViolationInHouseLobbyingExceeds2000DeMinimisExceptionNonDeductible
        );
        assert_eq!(output.disallowed_amount_dollars, 2_001);
    }

    #[test]
    fn de_minimis_professional_lobbyist_payment_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::DeMinimisInHouseLobbyingExceptionUnderSection162E5B;
        input.expenditure_source = ExpenditureSource::PaidToProfessionalLobbyist;
        input.expenditure_amount_dollars = 500;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section162EMode::ViolationProfessionalLobbyistPaymentNotEligibleForDeMinimisException
        );
    }

    #[test]
    fn de_minimis_dues_to_organization_lobbying_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::DeMinimisInHouseLobbyingExceptionUnderSection162E5B;
        input.expenditure_source = ExpenditureSource::DuesPaidToOrganizationThatLobbies;
        input.expenditure_amount_dollars = 1_000;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section162EMode::ViolationDuesToOrganizationLobbyingNotEligibleForDeMinimisException
        );
    }

    #[test]
    fn constants_pin_statutory_facts() {
        assert_eq!(IRC_162E_TCJA_ENACTMENT_DATE_YEAR, 2017);
        assert_eq!(IRC_162E_TCJA_ENACTMENT_DATE_MONTH, 12);
        assert_eq!(IRC_162E_TCJA_ENACTMENT_DATE_DAY, 22);
        assert_eq!(IRC_162E_TCJA_PUBLIC_LAW_CONGRESS, 115);
        assert_eq!(IRC_162E_TCJA_PUBLIC_LAW_ENACTMENT, 97);
        assert_eq!(IRC_162E_TCJA_ENABLING_SECTION, 13308);
        assert_eq!(IRC_162E_TCJA_STAT_VOLUME, 131);
        assert_eq!(IRC_162E_TCJA_STAT_PAGE, 2054);
        assert_eq!(IRC_162E_DE_MINIMIS_IN_HOUSE_LOBBYING_EXCEPTION_DOLLARS, 2_000);
        assert_eq!(IRC_162E_BASIS_POINT_DENOMINATOR, 10_000);
    }

    #[test]
    fn citations_contain_landmarks() {
        let output = check(&baseline_input());
        let joined = output.citations.join("\n");
        assert!(joined.contains("IRC § 162(e)"));
        assert!(joined.contains("Tax Cuts and Jobs Act of 2017"));
        assert!(joined.contains("Section 13308"));
        assert!(joined.contains("Public Law 115-97"));
        assert!(joined.contains("131 Stat. 2054"));
        assert!(joined.contains("December 22, 2017"));
        assert!(joined.contains("§ 162(e)(1)(A)"));
        assert!(joined.contains("§ 162(e)(1)"));
        assert!(joined.contains("§ 162(e)(4)"));
        assert!(joined.contains("§ 162(e)(5)(B)"));
        assert!(joined.contains("$2,000"));
        assert!(joined.contains("influencing legislation"));
        assert!(joined.contains("political campaign"));
        assert!(joined.contains("general public"));
        assert!(joined.contains("covered executive branch official"));
        assert!(joined.contains("WHITE HOUSE OFFICE"));
        assert!(joined.contains("LEVEL I OF THE EXECUTIVE SCHEDULE"));
        assert!(joined.contains("CABINET LEVEL STATUS"));
        assert!(joined.contains("IMMEDIATE DEPUTY"));
        assert!(joined.contains("LOCAL COUNCIL"));
        assert!(joined.contains("Indian tribal government"));
        assert!(joined.contains("PROFESSIONAL LOBBYISTS"));
        assert!(joined.contains("DUES PAID TO ORGANIZATIONS"));
    }
}
