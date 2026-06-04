//! IRC § 6417 — Elective Payment of Applicable
//! Credits (Direct Pay)
//! Compliance Module — pure-compute check for IRA 2022
//! § 13801(a) direct-pay (elective payment) monetization
//! framework.
//!
//! **Inflation Reduction Act of 2022 enactment**: Section
//! 6417 was implemented through **§ 13801(a) of Public
//! Law 117-169** (136 Stat. 1818, 2009), commonly known
//! as the Inflation Reduction Act of 2022 (IRA), **enacted
//! August 16, 2022**. **Final regulations released March
//! 5, 2024** (published Federal Register March 11, 2024).
//!
//! **Companion monetization framework to § 6418
//! transferability**: § 6417 allows **APPLICABLE ENTITIES**
//! (tax-exempt entities) to elect to treat **APPLICABLE
//! CREDITS** as **DIRECT PAYMENTS** (cash refunds) from
//! the IRS rather than nonrefundable tax credits — enabling
//! tax-exempt entities (which have no federal income tax
//! liability) to monetize the IRA 2022 clean-energy
//! credits as **CASH GRANTS**. Together § 6417 (direct pay
//! for tax-exempt entities) + § 6418 (transferability for
//! taxable entities) form the **COMPLETE MONETIZATION
//! TOOLKIT** for the IRA 2022 clean-energy cluster.
//!
//! **Distinctive § 6417 features**: **12 APPLICABLE
//! CREDIT CATEGORIES** under § 6417(b) (one more than
//! § 6418's 11 — § 48C Qualifying Advanced Energy Project
//! Credit ADDED in § 6417, not in § 6418); **APPLICABLE
//! ENTITY** definition under § 6417(d)(1)(A) covers
//! (1) tax-exempt organizations (Section 501-530),
//! (2) states + political subdivisions, (3) TVA, (4) Indian
//! tribal governments, (5) Alaska Native Corporations,
//! (6) rural electric cooperatives; **ELECTING TAXPAYER**
//! (non-applicable entity) under § 6417(d)(1)(B/C/D) may
//! elect direct pay for ONLY THREE CREDITS (§ 45Q, § 45V,
//! § 45X) for a **SINGLE 5-YEAR PERIOD**; **TREATED AS
//! PAYMENT AGAINST TAX** (refundable); **SUNSET**: for
//! tax years beginning before 2033; **20% EXCESSIVE
//! PAYMENT PENALTY** under § 6417(d)(6) (parallel to
//! § 6418(g)(2)); **PRE-FILING REGISTRATION REQUIRED**;
//! election **IRREVOCABLE**.
//!
//! Web research (verified 2026-06-03):
//! - **Inflation Reduction Act of 2022 enactment**: § 13801(a) of Public Law 117-169 (136 Stat. 1818, 2009), commonly known as the Inflation Reduction Act of 2022 (IRA), enacted August 16, 2022; final regulations released March 5, 2024 (Federal Register March 11, 2024) ([Cornell LII — 26 U.S. Code § 6417](https://www.law.cornell.edu/uscode/text/26/6417); [Bloomberg Tax — Sec. 6417 Elective Payment Of Applicable Credits](https://irc.bloombergtax.com/public/uscode/doc/irc/section_6417); [Federal Register — Elective Payment of Applicable Credits Final Rule March 11, 2024](https://www.federalregister.gov/documents/2024/03/11/2024-04604/elective-payment-of-applicable-credits-elective-payment-of-advanced-manufacturing-investment-credit); [Federal Register — Section 6417 Elective Payment of Applicable Credits Proposed Rule June 21, 2023](https://www.federalregister.gov/documents/2023/06/21/2023-12798/section-6417-elective-payment-of-applicable-credits); [Cornell LII — Definition: Applicable Entity from 26 USC § 6417(d)(1)](https://www.law.cornell.edu/definitions/uscode.php?width=840&height=800&iframe=true&def_id=26-USC-1846425476-530503879&term_occur=999&term_src=title:26:subtitle:F:chapter:65:subchapter:B:section:6417); [House.gov — 26 USC 6417 Elective Payment of Applicable Credits](https://uscode.house.gov/view.xhtml?req=(title:26+section:6417+edition:prelim)); [Tax Notes — Sec. 6417 Elective Payment of Applicable Credits](https://www.taxnotes.com/research/federal/usc26/6417); [Tax Notes — Sec. 1.6417-1 Elective Payment of Applicable Credits](https://www.taxnotes.com/research/federal/cfr26/1.6417-1); [IRS — Elective Pay and Transferability FAQs: Elective Pay](https://www.irs.gov/credits-deductions/elective-pay-and-transferability-frequently-asked-questions-elective-pay); [PwC — Regulations Finalized on Direct Payment of Tax Credits](https://www.pwc.com/us/en/services/tax/library/pwc-regulations-finalized-on-direct-payment-of-tax-credits.html); [EY — IRS Issues Final Regulations Direct Pay Section 6417 + 48D](https://taxnews.ey.com/news/2024-0624-irs-issues-final-regulations-with-few-changes-on-direct-pay-elections-for-certain-energy-credits-under-irc-section-6417-and-advanced-manufacturing-investment-credits-under-irc-section-48d); [Pillsbury Law — Treasury and IRS Issue Final Regulations on Direct Pay Election](https://www.pillsburylaw.com/en/news-and-insights/treasury-irs-regulations-guidance-direct-pay-election.html); [Holland & Knight — Inflation Reduction Act Direct Pay Rules Finalized March 2024](https://www.hklaw.com/en/insights/publications/2024/03/inflation-reduction-act-direct-pay-rules-finalized); [Mayer Brown — Final Regulations on Direct-Pay Elections and Transfer of Tax Credits](https://www.mayerbrown.com/en/insights/publications/2024/05/final-regulations-issued-on-direct-pay-elections-and-transfer-of-tax-credits); [Paul Hastings — Treasury and IRS Release Final Regulations on Direct Pay](https://www.paulhastings.com/insights/client-alerts/treasury-and-irs-release-final-regulations-on-direct-pay); [Greenberg Traurig — IRS and Treasury Final and Proposed Regs Direct Pay + Chaining Notice](https://www.gtlaw.com/en/insights/2024/3/irs-and-treasury-department-issue-final-and-proposed-regulations-regarding-direct-pay-rules-under-inflation-reduction-act-and-code-section-6417-notice-regarding-chaining-and-revised-faqs); [Mayer Brown — Final Regulations on Taxpayers Eligible for Direct Pay Electing Out of Subchapter K](https://www.mayerbrown.com/en/insights/publications/2024/12/final-regulations-on-taxpayers-eligible-for-direct-pay-electing-out-of-subchapter-k); [Cherry Bekaert — Final Direct Pay Regulations Issued for IRA Energy Tax Credits](https://www.cbh.com/insights/articles/final-direct-pay-regulations-issued-for-ira-energy-tax-credits/); [Holland & Knight — Treasury Department and IRS Release Final Regulations on Direct Payment of Tax Credits](https://www.hklaw.com/en/insights/publications/2024/03/treasury-department-and-irs-release-final); [Moss Adams — IRS Issues Guidance for Direct Payment of Energy Tax Credits](https://www.mossadams.com/articles/2023/07/direct-pay-for-clean-energy-tax-credits); [Taft Law — How the IRA Changes the Way Energy Tax Credits Are Calculated and Monetized](https://www.taftlaw.com/news-events/law-bulletins/how-the-inflation-reduction-act-changes-the-way-energy-tax-credits-are-calculated-and-monetized/)).
//! - **§ 6417 Elective Payment Mechanism**: Section 6417 allows **APPLICABLE ENTITIES** to elect to treat **APPLICABLE CREDITS** as a **PAYMENT AGAINST TAX** in the amount of the credit, effectively converting the otherwise nonrefundable tax credit into a **REFUNDABLE CASH PAYMENT** from the IRS.
//! - **§ 6417(b) Twelve Applicable Credit Categories**: for tax years beginning before 2033, an applicable entity may irrevocably elect to treat a credit under any of the following sections as a payment against tax: **(1) § 30C** Alternative Fuel Vehicle Refueling Property; **(2) § 45** Renewable Electricity Production Credit (PTC); **(3) § 45Q** Carbon Oxide Sequestration; **(4) § 45U** Zero-Emission Nuclear Power Production; **(5) § 45V** Clean Hydrogen Production; **(6) § 45W** Qualified Commercial Clean Vehicles; **(7) § 45X** Advanced Manufacturing Production; **(8) § 45Y** Clean Electricity Production; **(9) § 45Z** Clean Fuel Production; **(10) § 48** Energy Credit (ITC); **(11) § 48C** Qualifying Advanced Energy Project Credit; **(12) § 48E** Clean Electricity Investment Credit — **ONE MORE THAN § 6418's 11 (§ 48C added)**.
//! - **§ 6417(d)(1)(A) Applicable Entity Definition**: applicable entity means any of: **(i)** any organization exempt from the tax imposed by **SUBTITLE A** (including organizations exempt under Sections **501 THROUGH 530**, such as 501(c)(3) tax-exempt nonprofits); **(ii)** any **STATE OR POLITICAL SUBDIVISION** thereof; **(iii)** the **TENNESSEE VALLEY AUTHORITY (TVA)**; **(iv)** an **INDIAN TRIBAL GOVERNMENT** (per Federally Recognized Indian Tribe List Act of 1994 § 104); **(v)** any **ALASKA NATIVE CORPORATION**; **(vi)** any corporation operating on a **COOPERATIVE BASIS** engaged in furnishing **ELECTRIC ENERGY TO PERSONS IN RURAL AREAS** (rural electric cooperatives).
//! - **§ 6417(d)(1)(B/C/D) Electing Taxpayer**: an "electing taxpayer" is a taxpayer **OTHER THAN AN APPLICABLE ENTITY** that may make a direct pay election with respect to **ONLY THREE CREDITS**: **§ 45Q** (carbon oxide sequestration), **§ 45V** (clean hydrogen), or **§ 45X** (advanced manufacturing) — and only for a **SINGLE 5-YEAR PERIOD** beginning with the year the election is made.
//! - **§ 6417(a) Refundable Payment Treatment**: the elective payment is treated as **MADE BY THE TAXPAYER AGAINST THE TAX IMPOSED BY SUBTITLE A** on the date payment would otherwise be due; for entities with no income tax liability (applicable entities), the credit is fully **REFUNDABLE AS A CASH PAYMENT** from the IRS.
//! - **§ 6417(c) Sunset — Tax Years Beginning Before 2033**: the elective payment election is available for tax years beginning before **JANUARY 1, 2033**; after 2032, the elective payment mechanism is not available for new claims.
//! - **§ 6417(d)(6) Excessive Payment — 20% Penalty**: an **EXCESSIVE PAYMENT** is an amount equal to the excess of the amount treated as a payment over the amount of the applicable credit otherwise allowable; the excess amount is subject to a **20% PENALTY**; the 20% addition does **NOT APPLY** if the taxpayer demonstrates the excessive payment resulted from **REASONABLE CAUSE** (parallel to § 6418(g)(2)).
//! - **§ 6417(d)(5) Pre-Filing Registration Required**: an applicable entity or electing taxpayer must register and receive a **REGISTRATION NUMBER** through the IRS pre-filing registration process **BEFORE** making the election on the tax return; one registration number per applicable credit per tax year.
//! - **§ 6417(d)(3) Election Irrevocable**: the elective payment election is **IRREVOCABLE** once made for the relevant tax year; for electing taxpayers, the election binds for the **SINGLE 5-YEAR PERIOD**.
//! - **§ 6417 "Chaining" with § 6418**: an applicable entity may NOT make a direct pay election AND a § 6418 transferability election on the same credit — the elections are mutually exclusive for the same credit instance; however, an applicable entity may receive a transferred credit under § 6418 and then make a direct pay election on it (subject to specific rules).

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const SECTION_6417_NUMBER: u32 = 6417;
pub const SECTION_6417_APPLICABLE_ENTITY_CATEGORIES_COUNT: u32 = 6;
pub const SECTION_6417_APPLICABLE_CREDIT_CATEGORIES_COUNT: u32 = 12;
pub const SECTION_6417_ELECTING_TAXPAYER_CREDIT_CATEGORIES_COUNT: u32 = 3;
pub const SECTION_6417_ELECTING_TAXPAYER_YEARS_LIMIT: u32 = 5;
pub const SECTION_6417_EXCESSIVE_PAYMENT_PENALTY_PERCENT: u32 = 20;
pub const SECTION_6417_SUNSET_YEAR: u32 = 2033;
pub const SECTION_6417_IRA_2022_ENACTMENT_YEAR: u32 = 2022;
pub const SECTION_6417_IRA_2022_PUBLIC_LAW_NUMBER: u32 = 117169;
pub const SECTION_6417_FINAL_REGULATIONS_RELEASED_YEAR: u32 = 2024;
pub const SECTION_6417_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ApplicableEntityCategory {
    TaxExemptOrganizationUnderSection501Through530,
    StateOrPoliticalSubdivision,
    TennesseeValleyAuthority,
    IndianTribalGovernment,
    AlaskaNativeCorporation,
    RuralElectricCooperative,
    NotApplicableEntityElectingTaxpayer,
    NotApplicableEntityNotEligible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ApplicableCreditCategory {
    Section30CAlternativeFuelVehicleRefuelingPropertyCredit,
    Section45RenewableElectricityProductionCredit,
    Section45QCarbonOxideSequestrationCredit,
    Section45UZeroEmissionNuclearPowerCredit,
    Section45VCleanHydrogenProductionCredit,
    Section45WQualifiedCommercialCleanVehiclesCredit,
    Section45XAdvancedManufacturingProductionCredit,
    Section45YCleanElectricityProductionCredit,
    Section45ZCleanFuelProductionCredit,
    Section48EnergyInvestmentCredit,
    Section48CQualifyingAdvancedEnergyProjectCredit,
    Section48ECleanElectricityInvestmentCredit,
    NotEligibleForSection6417DirectPay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TaxYearStatus {
    TaxYearBeforeJanuary1_2033,
    TaxYearOnOrAfterJanuary1_2033,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExcessivePaymentStatus {
    NoExcessivePayment,
    ExcessivePaymentWithReasonableCause,
    ExcessivePaymentWithoutReasonableCause,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PreFilingRegistrationStatus {
    RegistrationNumberObtainedBeforeReturnFiled,
    RegistrationNumberNotObtained,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ElectingTaxpayerEligibleCreditStatus {
    ElectingTaxpayerWithEligibleCreditUnderThreeCreditList,
    ElectingTaxpayerWithIneligibleCredit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChainingStatus {
    NoChainingDirectElectionOnly,
    ChainingDirectPayAfterReceivingTransferUnderSection6418,
    ChainingProhibitedSameCreditBothDirectPayAndTransfer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAspect {
    ApplicableEntityStatusUnderSection6417D1A,
    ApplicableCreditCategoryUnderSection6417B,
    ElectingTaxpayerThreeCreditLimitationUnderSection6417D1BCD,
    SunsetTaxYearBefore2033UnderSection6417C,
    PreFilingRegistrationUnderSection6417D5,
    ExcessivePaymentUnderSection6417D6,
    ElectionIrrevocabilityUnderSection6417D3,
    ChainingWithSection6418Transferability,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section6417Mode {
    NotApplicableCreditNotEligibleUnderSection6417B,
    NotApplicableNotEligibleEntityOrElectingTaxpayer,
    CompliantApplicableEntityTaxExemptOrganization,
    CompliantApplicableEntityStateOrPoliticalSubdivision,
    CompliantApplicableEntityTennesseeValleyAuthority,
    CompliantApplicableEntityIndianTribalGovernment,
    CompliantApplicableEntityAlaskaNativeCorporation,
    CompliantApplicableEntityRuralElectricCooperative,
    CompliantElectingTaxpayerWithEligibleCreditUnderThreeCreditList,
    CompliantApplicableCreditCategoryUnderSection6417B,
    CompliantSunsetTaxYearBefore2033,
    CompliantPreFilingRegistrationNumberObtained,
    CompliantNoExcessivePayment,
    CompliantExcessivePaymentWithReasonableCauseNo20PercentPenalty,
    CompliantElectionIrrevocable,
    CompliantNoChainingDirectElectionOnly,
    CompliantChainingDirectPayAfterReceivingTransferUnderSection6418,
    ViolationElectingTaxpayerCreditNotIn45Q45V45XList,
    ViolationSunsetTaxYearOnOrAfter2033,
    ViolationPreFilingRegistrationNumberNotObtained,
    ViolationExcessivePaymentWithout20PercentReasonableCausePenaltyApplies,
    ViolationChainingProhibitedSameCreditBothDirectPayAndTransfer,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub applicable_entity_category: ApplicableEntityCategory,
    pub applicable_credit_category: ApplicableCreditCategory,
    pub tax_year_status: TaxYearStatus,
    pub excessive_payment_status: ExcessivePaymentStatus,
    pub pre_filing_registration_status: PreFilingRegistrationStatus,
    pub electing_taxpayer_eligible_credit_status: ElectingTaxpayerEligibleCreditStatus,
    pub chaining_status: ChainingStatus,
    pub compliance_aspect: ComplianceAspect,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section6417Mode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type Section6417Input = Input;
pub type Section6417Output = Output;
pub type Section6417Result = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "Inflation Reduction Act of 2022 enactment — § 13801(a) of Public Law 117-169 (136 Stat. 1818, 2009), commonly known as the Inflation Reduction Act of 2022 (IRA), enacted August 16, 2022; final regulations released March 5, 2024 (Federal Register March 11, 2024)".to_string(),
        "IRC § 6417 Elective Payment Mechanism — Section 6417 allows APPLICABLE ENTITIES to elect to treat APPLICABLE CREDITS as a PAYMENT AGAINST TAX in the amount of the credit, effectively converting the otherwise nonrefundable tax credit into a REFUNDABLE CASH PAYMENT from the IRS".to_string(),
        "IRC § 6417(b) Twelve Applicable Credit Categories — for tax years beginning before 2033: (1) § 30C Alternative Fuel Vehicle Refueling Property; (2) § 45 Renewable Electricity Production Credit (PTC); (3) § 45Q Carbon Oxide Sequestration; (4) § 45U Zero-Emission Nuclear Power Production; (5) § 45V Clean Hydrogen Production; (6) § 45W Qualified Commercial Clean Vehicles; (7) § 45X Advanced Manufacturing Production; (8) § 45Y Clean Electricity Production; (9) § 45Z Clean Fuel Production; (10) § 48 Energy Credit (ITC); (11) § 48C Qualifying Advanced Energy Project Credit; (12) § 48E Clean Electricity Investment Credit — ONE MORE THAN § 6418's 11 (§ 48C added in § 6417)".to_string(),
        "IRC § 6417(d)(1)(A) Applicable Entity Definition — (i) tax-exempt organizations under SUBTITLE A (Sections 501 THROUGH 530); (ii) STATE OR POLITICAL SUBDIVISION; (iii) TENNESSEE VALLEY AUTHORITY (TVA); (iv) INDIAN TRIBAL GOVERNMENT per Federally Recognized Indian Tribe List Act § 104; (v) ALASKA NATIVE CORPORATION; (vi) RURAL ELECTRIC COOPERATIVE corporation operating on a cooperative basis".to_string(),
        "IRC § 6417(d)(1)(B/C/D) Electing Taxpayer — non-applicable entities may elect direct pay for ONLY THREE CREDITS: § 45Q (carbon oxide sequestration), § 45V (clean hydrogen), or § 45X (advanced manufacturing) — and only for a SINGLE 5-YEAR PERIOD".to_string(),
        "IRC § 6417(a) Refundable Payment Treatment — the elective payment is treated as MADE BY THE TAXPAYER AGAINST THE TAX IMPOSED BY SUBTITLE A on the date payment would otherwise be due; for applicable entities with no income tax liability, the credit is fully REFUNDABLE AS A CASH PAYMENT from the IRS".to_string(),
        "IRC § 6417(c) Sunset — Tax Years Beginning Before 2033 — the elective payment election is available for tax years beginning before JANUARY 1, 2033; after 2032, the elective payment mechanism is not available for new claims".to_string(),
        "IRC § 6417(d)(6) Excessive Payment — 20% Penalty — an EXCESSIVE PAYMENT is the excess of the amount treated as a payment over the amount of the applicable credit otherwise allowable; the excess amount is subject to a 20% PENALTY; does NOT APPLY if reasonable cause demonstrated".to_string(),
        "IRC § 6417(d)(5) Pre-Filing Registration Required — applicable entity or electing taxpayer must register and receive a REGISTRATION NUMBER through the IRS pre-filing registration process BEFORE making the election; one registration number per applicable credit per tax year".to_string(),
        "IRC § 6417(d)(3) Election Irrevocable — the elective payment election is IRREVOCABLE once made for the relevant tax year; for electing taxpayers, the election binds for the SINGLE 5-YEAR PERIOD".to_string(),
        "IRC § 6417 Chaining with § 6418 — an applicable entity may NOT make a direct pay election AND a § 6418 transferability election on the same credit (mutually exclusive); however, an applicable entity may receive a transferred credit under § 6418 and then make a direct pay election on it".to_string(),
        "Cornell LII + Bloomberg Tax + Federal Register + IRS + PwC + EY + Pillsbury Law + Holland & Knight + Mayer Brown + Paul Hastings + Greenberg Traurig + Cherry Bekaert + Moss Adams + Taft Law — practitioner overviews of IRC § 6417 elective payment of applicable credits (direct pay)".to_string(),
    ];

    if input.applicable_credit_category
        == ApplicableCreditCategory::NotEligibleForSection6417DirectPay
    {
        return Output {
            mode: Section6417Mode::NotApplicableCreditNotEligibleUnderSection6417B,
            statutory_basis: "IRC § 6417(b) — credit not within 12 applicable categories".to_string(),
            notes: "NOT APPLICABLE: credit not within the 12 applicable credit categories under § 6417(b); direct pay elective payment unavailable.".to_string(),
            citations,
        };
    }

    if input.applicable_entity_category
        == ApplicableEntityCategory::NotApplicableEntityNotEligible
    {
        return Output {
            mode: Section6417Mode::NotApplicableNotEligibleEntityOrElectingTaxpayer,
            statutory_basis: "IRC § 6417(d)(1) — entity not applicable entity and not eligible electing taxpayer".to_string(),
            notes: "NOT APPLICABLE: entity is not an applicable entity under § 6417(d)(1)(A) and not an eligible electing taxpayer under § 6417(d)(1)(B/C/D); direct pay elective payment unavailable.".to_string(),
            citations,
        };
    }

    match input.compliance_aspect {
        ComplianceAspect::ApplicableEntityStatusUnderSection6417D1A => {
            match input.applicable_entity_category {
                ApplicableEntityCategory::TaxExemptOrganizationUnderSection501Through530 => Output {
                    mode: Section6417Mode::CompliantApplicableEntityTaxExemptOrganization,
                    statutory_basis: "IRC § 6417(d)(1)(A)(i) — tax-exempt organization under Sections 501-530".to_string(),
                    notes: "COMPLIANT: tax-exempt organization under Sections 501 THROUGH 530 is APPLICABLE ENTITY under § 6417(d)(1)(A)(i); direct pay election available for all 12 applicable credits.".to_string(),
                    citations,
                },
                ApplicableEntityCategory::StateOrPoliticalSubdivision => Output {
                    mode: Section6417Mode::CompliantApplicableEntityStateOrPoliticalSubdivision,
                    statutory_basis: "IRC § 6417(d)(1)(A)(ii) — state or political subdivision".to_string(),
                    notes: "COMPLIANT: STATE OR POLITICAL SUBDIVISION is APPLICABLE ENTITY under § 6417(d)(1)(A)(ii); direct pay election available for all 12 applicable credits.".to_string(),
                    citations,
                },
                ApplicableEntityCategory::TennesseeValleyAuthority => Output {
                    mode: Section6417Mode::CompliantApplicableEntityTennesseeValleyAuthority,
                    statutory_basis: "IRC § 6417(d)(1)(A)(iii) — Tennessee Valley Authority (TVA)".to_string(),
                    notes: "COMPLIANT: TENNESSEE VALLEY AUTHORITY (TVA) is APPLICABLE ENTITY under § 6417(d)(1)(A)(iii); direct pay election available for all 12 applicable credits.".to_string(),
                    citations,
                },
                ApplicableEntityCategory::IndianTribalGovernment => Output {
                    mode: Section6417Mode::CompliantApplicableEntityIndianTribalGovernment,
                    statutory_basis: "IRC § 6417(d)(1)(A)(iv) — Indian tribal government per Federally Recognized Indian Tribe List Act § 104".to_string(),
                    notes: "COMPLIANT: INDIAN TRIBAL GOVERNMENT is APPLICABLE ENTITY under § 6417(d)(1)(A)(iv) per Federally Recognized Indian Tribe List Act of 1994 § 104; direct pay election available for all 12 applicable credits.".to_string(),
                    citations,
                },
                ApplicableEntityCategory::AlaskaNativeCorporation => Output {
                    mode: Section6417Mode::CompliantApplicableEntityAlaskaNativeCorporation,
                    statutory_basis: "IRC § 6417(d)(1)(A)(v) — Alaska Native Corporation".to_string(),
                    notes: "COMPLIANT: ALASKA NATIVE CORPORATION is APPLICABLE ENTITY under § 6417(d)(1)(A)(v); direct pay election available for all 12 applicable credits.".to_string(),
                    citations,
                },
                ApplicableEntityCategory::RuralElectricCooperative => Output {
                    mode: Section6417Mode::CompliantApplicableEntityRuralElectricCooperative,
                    statutory_basis: "IRC § 6417(d)(1)(A)(vi) — rural electric cooperative".to_string(),
                    notes: "COMPLIANT: RURAL ELECTRIC COOPERATIVE (corporation operating on cooperative basis furnishing electric energy to persons in rural areas) is APPLICABLE ENTITY under § 6417(d)(1)(A)(vi); direct pay election available for all 12 applicable credits.".to_string(),
                    citations,
                },
                ApplicableEntityCategory::NotApplicableEntityElectingTaxpayer => Output {
                    mode: Section6417Mode::CompliantElectingTaxpayerWithEligibleCreditUnderThreeCreditList,
                    statutory_basis: "IRC § 6417(d)(1)(B/C/D) — electing taxpayer (non-applicable entity) limited to § 45Q/§ 45V/§ 45X".to_string(),
                    notes: "COMPLIANT: electing taxpayer (non-applicable entity) may elect direct pay under § 6417(d)(1)(B/C/D) for ONLY THREE CREDITS: § 45Q, § 45V, § 45X — and only for SINGLE 5-YEAR PERIOD.".to_string(),
                    citations,
                },
                ApplicableEntityCategory::NotApplicableEntityNotEligible => Output {
                    mode: Section6417Mode::NotApplicableNotEligibleEntityOrElectingTaxpayer,
                    statutory_basis: "IRC § 6417(d)(1) — entity not eligible".to_string(),
                    notes: "NOT APPLICABLE: entity not eligible for § 6417 direct pay.".to_string(),
                    citations,
                },
            }
        }
        ComplianceAspect::ApplicableCreditCategoryUnderSection6417B => Output {
            mode: Section6417Mode::CompliantApplicableCreditCategoryUnderSection6417B,
            statutory_basis: format!(
                "IRC § 6417(b) — {c:?} within 12 applicable credit categories",
                c = input.applicable_credit_category,
            ),
            notes: format!(
                "COMPLIANT: {c:?} within the 12 applicable credit categories under § 6417(b); direct pay elective payment available.",
                c = input.applicable_credit_category,
            ),
            citations,
        },
        ComplianceAspect::ElectingTaxpayerThreeCreditLimitationUnderSection6417D1BCD => {
            match input.electing_taxpayer_eligible_credit_status {
                ElectingTaxpayerEligibleCreditStatus::ElectingTaxpayerWithEligibleCreditUnderThreeCreditList => Output {
                    mode: Section6417Mode::CompliantElectingTaxpayerWithEligibleCreditUnderThreeCreditList,
                    statutory_basis: "IRC § 6417(d)(1)(B/C/D) — electing taxpayer with eligible credit under three-credit list".to_string(),
                    notes: "COMPLIANT: electing taxpayer with eligible credit (§ 45Q, § 45V, or § 45X) under three-credit list under § 6417(d)(1)(B/C/D); direct pay available for single 5-year period.".to_string(),
                    citations,
                },
                ElectingTaxpayerEligibleCreditStatus::ElectingTaxpayerWithIneligibleCredit => Output {
                    mode: Section6417Mode::ViolationElectingTaxpayerCreditNotIn45Q45V45XList,
                    statutory_basis: "IRC § 6417(d)(1)(B/C/D) — electing taxpayer credit not in three-credit list".to_string(),
                    notes: "VIOLATION: electing taxpayer attempting direct pay on credit NOT IN three-credit list (§ 45Q, § 45V, § 45X) under § 6417(d)(1)(B/C/D); only applicable entities may elect direct pay on the other 9 credits.".to_string(),
                    citations,
                },
            }
        }
        ComplianceAspect::SunsetTaxYearBefore2033UnderSection6417C => {
            match input.tax_year_status {
                TaxYearStatus::TaxYearBeforeJanuary1_2033 => Output {
                    mode: Section6417Mode::CompliantSunsetTaxYearBefore2033,
                    statutory_basis: "IRC § 6417(c) — tax year before January 1, 2033".to_string(),
                    notes: "COMPLIANT: tax year before January 1, 2033 under § 6417(c); direct pay elective payment available.".to_string(),
                    citations,
                },
                TaxYearStatus::TaxYearOnOrAfterJanuary1_2033 => Output {
                    mode: Section6417Mode::ViolationSunsetTaxYearOnOrAfter2033,
                    statutory_basis: "IRC § 6417(c) — tax year on or after January 1, 2033 (sunset)".to_string(),
                    notes: "VIOLATION: tax year on or after January 1, 2033 under § 6417(c); direct pay elective payment SUNSET; mechanism no longer available for new claims.".to_string(),
                    citations,
                },
            }
        }
        ComplianceAspect::PreFilingRegistrationUnderSection6417D5 => {
            match input.pre_filing_registration_status {
                PreFilingRegistrationStatus::RegistrationNumberObtainedBeforeReturnFiled => Output {
                    mode: Section6417Mode::CompliantPreFilingRegistrationNumberObtained,
                    statutory_basis: "IRC § 6417(d)(5) — pre-filing registration number obtained".to_string(),
                    notes: "COMPLIANT: pre-filing registration number obtained through IRS portal BEFORE election made on return under § 6417(d)(5).".to_string(),
                    citations,
                },
                PreFilingRegistrationStatus::RegistrationNumberNotObtained => Output {
                    mode: Section6417Mode::ViolationPreFilingRegistrationNumberNotObtained,
                    statutory_basis: "IRC § 6417(d)(5) — pre-filing registration number not obtained".to_string(),
                    notes: "VIOLATION: pre-filing registration number not obtained under § 6417(d)(5); direct pay election cannot be perfected without registration number.".to_string(),
                    citations,
                },
            }
        }
        ComplianceAspect::ExcessivePaymentUnderSection6417D6 => {
            match input.excessive_payment_status {
                ExcessivePaymentStatus::NoExcessivePayment => Output {
                    mode: Section6417Mode::CompliantNoExcessivePayment,
                    statutory_basis: "IRC § 6417(d)(6) — no excessive payment".to_string(),
                    notes: "COMPLIANT: no excessive payment under § 6417(d)(6); 20% penalty not triggered.".to_string(),
                    citations,
                },
                ExcessivePaymentStatus::ExcessivePaymentWithReasonableCause => Output {
                    mode: Section6417Mode::CompliantExcessivePaymentWithReasonableCauseNo20PercentPenalty,
                    statutory_basis: "IRC § 6417(d)(6) — excessive payment with reasonable cause exception".to_string(),
                    notes: "COMPLIANT: excessive payment with REASONABLE CAUSE under § 6417(d)(6); 20% addition to tax does NOT APPLY.".to_string(),
                    citations,
                },
                ExcessivePaymentStatus::ExcessivePaymentWithoutReasonableCause => Output {
                    mode: Section6417Mode::ViolationExcessivePaymentWithout20PercentReasonableCausePenaltyApplies,
                    statutory_basis: "IRC § 6417(d)(6) — excessive payment without reasonable cause; 20% penalty applies".to_string(),
                    notes: "VIOLATION: excessive payment without reasonable cause under § 6417(d)(6); excess amount + 20% penalty addition to tax applies.".to_string(),
                    citations,
                },
            }
        }
        ComplianceAspect::ElectionIrrevocabilityUnderSection6417D3 => Output {
            mode: Section6417Mode::CompliantElectionIrrevocable,
            statutory_basis: "IRC § 6417(d)(3) — election irrevocable for tax year".to_string(),
            notes: "COMPLIANT: elective payment election is IRREVOCABLE once made for the relevant tax year under § 6417(d)(3); for electing taxpayers, election binds for SINGLE 5-YEAR PERIOD.".to_string(),
            citations,
        },
        ComplianceAspect::ChainingWithSection6418Transferability => match input.chaining_status {
            ChainingStatus::NoChainingDirectElectionOnly => Output {
                mode: Section6417Mode::CompliantNoChainingDirectElectionOnly,
                statutory_basis: "IRC § 6417 + § 6418 — direct election only, no chaining".to_string(),
                notes: "COMPLIANT: applicable entity makes direct pay election only under § 6417; no chaining with § 6418 transferability.".to_string(),
                citations,
            },
            ChainingStatus::ChainingDirectPayAfterReceivingTransferUnderSection6418 => Output {
                mode: Section6417Mode::CompliantChainingDirectPayAfterReceivingTransferUnderSection6418,
                statutory_basis: "IRC § 6417 + § 6418 — direct pay election on transferred credit permitted".to_string(),
                notes: "COMPLIANT: applicable entity received transferred credit under § 6418 and then made direct pay election under § 6417 on it; chaining permitted in this direction.".to_string(),
                citations,
            },
            ChainingStatus::ChainingProhibitedSameCreditBothDirectPayAndTransfer => Output {
                mode: Section6417Mode::ViolationChainingProhibitedSameCreditBothDirectPayAndTransfer,
                statutory_basis: "IRC § 6417 + § 6418 — same credit cannot be both direct pay AND transferred".to_string(),
                notes: "VIOLATION: applicable entity attempted both direct pay election under § 6417 AND transferability election under § 6418 on same credit; elections are MUTUALLY EXCLUSIVE for same credit instance.".to_string(),
                citations,
            },
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_input() -> Input {
        Input {
            applicable_entity_category:
                ApplicableEntityCategory::TaxExemptOrganizationUnderSection501Through530,
            applicable_credit_category:
                ApplicableCreditCategory::Section45YCleanElectricityProductionCredit,
            tax_year_status: TaxYearStatus::TaxYearBeforeJanuary1_2033,
            excessive_payment_status: ExcessivePaymentStatus::NoExcessivePayment,
            pre_filing_registration_status:
                PreFilingRegistrationStatus::RegistrationNumberObtainedBeforeReturnFiled,
            electing_taxpayer_eligible_credit_status:
                ElectingTaxpayerEligibleCreditStatus::ElectingTaxpayerWithEligibleCreditUnderThreeCreditList,
            chaining_status: ChainingStatus::NoChainingDirectElectionOnly,
            compliance_aspect: ComplianceAspect::ApplicableEntityStatusUnderSection6417D1A,
        }
    }

    #[test]
    fn ineligible_credit_not_applicable() {
        let mut input = baseline_input();
        input.applicable_credit_category =
            ApplicableCreditCategory::NotEligibleForSection6417DirectPay;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section6417Mode::NotApplicableCreditNotEligibleUnderSection6417B
        );
    }

    #[test]
    fn not_eligible_entity_not_applicable() {
        let mut input = baseline_input();
        input.applicable_entity_category =
            ApplicableEntityCategory::NotApplicableEntityNotEligible;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section6417Mode::NotApplicableNotEligibleEntityOrElectingTaxpayer
        );
    }

    #[test]
    fn tax_exempt_organization_applicable_entity_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ApplicableEntityStatusUnderSection6417D1A;
        input.applicable_entity_category =
            ApplicableEntityCategory::TaxExemptOrganizationUnderSection501Through530;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section6417Mode::CompliantApplicableEntityTaxExemptOrganization
        );
    }

    #[test]
    fn state_or_political_subdivision_applicable_entity_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ApplicableEntityStatusUnderSection6417D1A;
        input.applicable_entity_category = ApplicableEntityCategory::StateOrPoliticalSubdivision;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section6417Mode::CompliantApplicableEntityStateOrPoliticalSubdivision
        );
    }

    #[test]
    fn tennessee_valley_authority_applicable_entity_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ApplicableEntityStatusUnderSection6417D1A;
        input.applicable_entity_category = ApplicableEntityCategory::TennesseeValleyAuthority;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section6417Mode::CompliantApplicableEntityTennesseeValleyAuthority
        );
    }

    #[test]
    fn indian_tribal_government_applicable_entity_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ApplicableEntityStatusUnderSection6417D1A;
        input.applicable_entity_category = ApplicableEntityCategory::IndianTribalGovernment;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section6417Mode::CompliantApplicableEntityIndianTribalGovernment
        );
    }

    #[test]
    fn alaska_native_corporation_applicable_entity_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ApplicableEntityStatusUnderSection6417D1A;
        input.applicable_entity_category = ApplicableEntityCategory::AlaskaNativeCorporation;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section6417Mode::CompliantApplicableEntityAlaskaNativeCorporation
        );
    }

    #[test]
    fn rural_electric_cooperative_applicable_entity_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ApplicableEntityStatusUnderSection6417D1A;
        input.applicable_entity_category = ApplicableEntityCategory::RuralElectricCooperative;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section6417Mode::CompliantApplicableEntityRuralElectricCooperative
        );
    }

    #[test]
    fn section_30c_applicable_credit_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ApplicableCreditCategoryUnderSection6417B;
        input.applicable_credit_category =
            ApplicableCreditCategory::Section30CAlternativeFuelVehicleRefuelingPropertyCredit;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section6417Mode::CompliantApplicableCreditCategoryUnderSection6417B
        );
    }

    #[test]
    fn section_48c_applicable_credit_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ApplicableCreditCategoryUnderSection6417B;
        input.applicable_credit_category =
            ApplicableCreditCategory::Section48CQualifyingAdvancedEnergyProjectCredit;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section6417Mode::CompliantApplicableCreditCategoryUnderSection6417B
        );
    }

    #[test]
    fn section_48e_applicable_credit_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ApplicableCreditCategoryUnderSection6417B;
        input.applicable_credit_category =
            ApplicableCreditCategory::Section48ECleanElectricityInvestmentCredit;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section6417Mode::CompliantApplicableCreditCategoryUnderSection6417B
        );
    }

    #[test]
    fn electing_taxpayer_with_eligible_credit_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::ElectingTaxpayerThreeCreditLimitationUnderSection6417D1BCD;
        input.applicable_entity_category =
            ApplicableEntityCategory::NotApplicableEntityElectingTaxpayer;
        input.electing_taxpayer_eligible_credit_status =
            ElectingTaxpayerEligibleCreditStatus::ElectingTaxpayerWithEligibleCreditUnderThreeCreditList;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section6417Mode::CompliantElectingTaxpayerWithEligibleCreditUnderThreeCreditList
        );
    }

    #[test]
    fn electing_taxpayer_with_ineligible_credit_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::ElectingTaxpayerThreeCreditLimitationUnderSection6417D1BCD;
        input.applicable_entity_category =
            ApplicableEntityCategory::NotApplicableEntityElectingTaxpayer;
        input.electing_taxpayer_eligible_credit_status =
            ElectingTaxpayerEligibleCreditStatus::ElectingTaxpayerWithIneligibleCredit;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section6417Mode::ViolationElectingTaxpayerCreditNotIn45Q45V45XList
        );
    }

    #[test]
    fn tax_year_before_2033_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SunsetTaxYearBefore2033UnderSection6417C;
        input.tax_year_status = TaxYearStatus::TaxYearBeforeJanuary1_2033;
        let out = check(&input);
        assert_eq!(out.mode, Section6417Mode::CompliantSunsetTaxYearBefore2033);
    }

    #[test]
    fn tax_year_on_or_after_2033_sunset_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SunsetTaxYearBefore2033UnderSection6417C;
        input.tax_year_status = TaxYearStatus::TaxYearOnOrAfterJanuary1_2033;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section6417Mode::ViolationSunsetTaxYearOnOrAfter2033
        );
    }

    #[test]
    fn pre_filing_registration_obtained_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::PreFilingRegistrationUnderSection6417D5;
        input.pre_filing_registration_status =
            PreFilingRegistrationStatus::RegistrationNumberObtainedBeforeReturnFiled;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section6417Mode::CompliantPreFilingRegistrationNumberObtained
        );
    }

    #[test]
    fn pre_filing_registration_not_obtained_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::PreFilingRegistrationUnderSection6417D5;
        input.pre_filing_registration_status =
            PreFilingRegistrationStatus::RegistrationNumberNotObtained;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section6417Mode::ViolationPreFilingRegistrationNumberNotObtained
        );
    }

    #[test]
    fn no_excessive_payment_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ExcessivePaymentUnderSection6417D6;
        input.excessive_payment_status = ExcessivePaymentStatus::NoExcessivePayment;
        let out = check(&input);
        assert_eq!(out.mode, Section6417Mode::CompliantNoExcessivePayment);
    }

    #[test]
    fn excessive_payment_with_reasonable_cause_no_penalty_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ExcessivePaymentUnderSection6417D6;
        input.excessive_payment_status =
            ExcessivePaymentStatus::ExcessivePaymentWithReasonableCause;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section6417Mode::CompliantExcessivePaymentWithReasonableCauseNo20PercentPenalty
        );
    }

    #[test]
    fn excessive_payment_without_reasonable_cause_20_percent_penalty_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ExcessivePaymentUnderSection6417D6;
        input.excessive_payment_status =
            ExcessivePaymentStatus::ExcessivePaymentWithoutReasonableCause;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section6417Mode::ViolationExcessivePaymentWithout20PercentReasonableCausePenaltyApplies
        );
    }

    #[test]
    fn election_irrevocability_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ElectionIrrevocabilityUnderSection6417D3;
        let out = check(&input);
        assert_eq!(out.mode, Section6417Mode::CompliantElectionIrrevocable);
    }

    #[test]
    fn no_chaining_direct_election_only_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ChainingWithSection6418Transferability;
        input.chaining_status = ChainingStatus::NoChainingDirectElectionOnly;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section6417Mode::CompliantNoChainingDirectElectionOnly
        );
    }

    #[test]
    fn chaining_direct_pay_after_receiving_transfer_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ChainingWithSection6418Transferability;
        input.chaining_status =
            ChainingStatus::ChainingDirectPayAfterReceivingTransferUnderSection6418;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section6417Mode::CompliantChainingDirectPayAfterReceivingTransferUnderSection6418
        );
    }

    #[test]
    fn chaining_prohibited_same_credit_both_direct_pay_and_transfer_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ChainingWithSection6418Transferability;
        input.chaining_status =
            ChainingStatus::ChainingProhibitedSameCreditBothDirectPayAndTransfer;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section6417Mode::ViolationChainingProhibitedSameCreditBothDirectPayAndTransfer
        );
    }

    #[test]
    fn constants_pin_section_6417_statutory_thresholds() {
        assert_eq!(SECTION_6417_NUMBER, 6417);
        assert_eq!(SECTION_6417_APPLICABLE_ENTITY_CATEGORIES_COUNT, 6);
        assert_eq!(SECTION_6417_APPLICABLE_CREDIT_CATEGORIES_COUNT, 12);
        assert_eq!(SECTION_6417_ELECTING_TAXPAYER_CREDIT_CATEGORIES_COUNT, 3);
        assert_eq!(SECTION_6417_ELECTING_TAXPAYER_YEARS_LIMIT, 5);
        assert_eq!(SECTION_6417_EXCESSIVE_PAYMENT_PENALTY_PERCENT, 20);
        assert_eq!(SECTION_6417_SUNSET_YEAR, 2033);
        assert_eq!(SECTION_6417_IRA_2022_ENACTMENT_YEAR, 2022);
        assert_eq!(SECTION_6417_IRA_2022_PUBLIC_LAW_NUMBER, 117169);
        assert_eq!(SECTION_6417_FINAL_REGULATIONS_RELEASED_YEAR, 2024);
        assert_eq!(SECTION_6417_BASIS_POINT_DENOMINATOR, 10_000);
    }

    #[test]
    fn citations_pin_section_6417_statutory_provisions() {
        let input = baseline_input();
        let out = check(&input);
        let joined = out.citations.join(" || ");
        assert!(joined.contains("Inflation Reduction Act of 2022"));
        assert!(joined.contains("§ 13801(a) of Public Law 117-169"));
        assert!(joined.contains("August 16, 2022"));
        assert!(joined.contains("final regulations released March 5, 2024"));
        assert!(joined.contains("IRC § 6417"));
        assert!(joined.contains("APPLICABLE ENTITIES"));
        assert!(joined.contains("APPLICABLE CREDITS"));
        assert!(joined.contains("PAYMENT AGAINST TAX"));
        assert!(joined.contains("REFUNDABLE CASH PAYMENT"));
        assert!(joined.contains("IRC § 6417(b)"));
        assert!(joined.contains("§ 30C"));
        assert!(joined.contains("§ 45"));
        assert!(joined.contains("§ 45Q"));
        assert!(joined.contains("§ 45U"));
        assert!(joined.contains("§ 45V"));
        assert!(joined.contains("§ 45W"));
        assert!(joined.contains("§ 45X"));
        assert!(joined.contains("§ 45Y"));
        assert!(joined.contains("§ 45Z"));
        assert!(joined.contains("§ 48"));
        assert!(joined.contains("§ 48C"));
        assert!(joined.contains("§ 48E"));
        assert!(joined.contains("IRC § 6417(d)(1)(A)"));
        assert!(joined.contains("SUBTITLE A"));
        assert!(joined.contains("501 THROUGH 530"));
        assert!(joined.contains("STATE OR POLITICAL SUBDIVISION"));
        assert!(joined.contains("TENNESSEE VALLEY AUTHORITY"));
        assert!(joined.contains("INDIAN TRIBAL GOVERNMENT"));
        assert!(joined.contains("ALASKA NATIVE CORPORATION"));
        assert!(joined.contains("RURAL ELECTRIC COOPERATIVE"));
        assert!(joined.contains("IRC § 6417(d)(1)(B/C/D)"));
        assert!(joined.contains("ONLY THREE CREDITS"));
        assert!(joined.contains("SINGLE 5-YEAR PERIOD"));
        assert!(joined.contains("IRC § 6417(c)"));
        assert!(joined.contains("JANUARY 1, 2033"));
        assert!(joined.contains("IRC § 6417(d)(6)"));
        assert!(joined.contains("EXCESSIVE PAYMENT"));
        assert!(joined.contains("20% PENALTY"));
        assert!(joined.contains("reasonable cause demonstrated"));
        assert!(joined.contains("IRC § 6417(d)(5)"));
        assert!(joined.contains("REGISTRATION NUMBER"));
        assert!(joined.contains("IRC § 6417(d)(3)"));
        assert!(joined.contains("IRREVOCABLE"));
        assert!(joined.contains("Chaining with § 6418"));
    }
}
