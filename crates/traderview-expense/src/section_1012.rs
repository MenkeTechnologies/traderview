//! IRC § 1012 — Basis of Property: Cost / Cost Basis
//! Tracking and Identification Methods Module.
//!
//! Pure-compute check for IRC § 1012 cost basis general rule
//! and the trader-critical Treas. Reg. § 1.1012-1 lot
//! identification methods (specific identification, FIFO
//! default, average cost for RIC shares and DRIP stock).
//! § 1012 is the foundational basis-tracking provision that
//! every trader uses to compute gain or loss on disposition
//! of stock, mutual fund shares, debt instruments, and
//! options under the modern (post-2010 cost-basis-reporting-
//! regime) tax framework. Account-by-account tracking under
//! § 1012(c)(1) and the average-cost-method election for
//! RICs and DRIPs under § 1012(c)(2) and § 1012(d) shape the
//! gain / loss computation for every taxable disposition.
//!
//! Web research (verified 2026-06-03):
//! - **IRC § 1012(a) General Rule**: the basis of property shall be the **COST** of such property, except as otherwise provided in this subchapter and subchapters C (corporate distributions and adjustments), K (partners and partnerships), and P (capital gains and losses) ([Cornell LII 26 USC § 1012](https://www.law.cornell.edu/uscode/text/26/1012); [Bloomberg Tax Sec. 1012](https://irc.bloombergtax.com/public/uscode/doc/irc/section_1012); [Tax Notes IRC Section 1012](https://www.taxnotes.com/research/federal/usc26/1012); [LegalClarity — What Is IRC 1012? Cost Basis of Property Rules](https://legalclarity.org/irc-1012-determining-the-cost-basis-of-property/); [CCH AnswerConnect § 1012 Cost Basis of Property](https://answerconnect.cch.com/document/arp1032ae39127c571000afad90b11c18cbab03/federal/irc/explanation/1012-cost-basis-of-property)).
//! - **IRC § 1012(b) Real Property Taxes Exclusion**: cost may NOT include real property taxes treated as imposed on the taxpayer under § 164(d); the cost of real property excludes any portion of property taxes attributable to a period prior to acquisition that the seller paid (and that, under § 164(d), is treated as imposed on the buyer).
//! - **IRC § 1012(c)(1) Account-by-Account Basis Tracking**: for specified securities sold after applicable dates, the conventions prescribed by regulations under § 1012 shall be applied on an **ACCOUNT BY ACCOUNT BASIS**. This means a taxpayer who holds the same security across multiple brokerage accounts treats each account as separate for FIFO / specific-identification / average-cost election purposes.
//! - **IRC § 1012(c)(2) Average Cost Method for Regulated Investment Companies (RICs / Mutual Funds)**: stock in a regulated investment company is eligible for the **AVERAGE COST METHOD** of basis determination. The Treasury regulations specify that the **AVERAGE COST SINGLE CATEGORY (ACSC)** method applies — total cost basis of all RIC shares is divided by total number of shares held to compute average per-share basis.
//! - **IRC § 1012(c)(3) Pre-2012 / Post-2012 RIC Stock Separate Accounts**: any RIC stock acquired **BEFORE JANUARY 1, 2012** is treated as a SEPARATE ACCOUNT from RIC stock acquired ON OR AFTER such date. This bifurcation prevents a basis-method election on post-2012 stock from affecting the basis of pre-2012 (non-covered) shares.
//! - **IRC § 1012(d) Dividend Reinvestment Plans (DRIPs)**: in the case of any stock acquired **AFTER DECEMBER 31, 2011** in connection with a **DIVIDEND REINVESTMENT PLAN**, the basis of such stock while held as part of such plan shall be determined using one of the methods which may be used for determining the basis of stock in a regulated investment company (typically average cost).
//! - **Treas. Reg. § 1.1012-1(c) Specific Identification**: when stock is sold, the taxpayer may **SPECIFICALLY IDENTIFY** which shares are sold by (1) at the time of sale, designating the specific lot to be sold to the broker or transfer agent; AND (2) within a reasonable time receiving written confirmation from the broker / transfer agent of the identification. Specific identification typically produces the LOWEST tax outcome with proper planning because the taxpayer can choose the highest-basis lots first.
//! - **FIFO Default Rule (Treas. Reg. § 1.1012-1(c)(1))**: if specific identification is NOT made at the time of sale, the **FIRST-IN, FIRST-OUT (FIFO)** method is the DEFAULT rule for determining which shares were sold. FIFO generally produces the HIGHEST tax bill because it sells the oldest (typically most-appreciated) shares first.
//! - **LIFO Not Allowed for Securities**: the LAST-IN, FIRST-OUT (LIFO) method is **NOT PERMITTED** for stock or securities under § 1012; LIFO is allowed only for inventory under § 471 / § 472.
//! - **Cost Basis Reporting Reform (Energy Improvement and Extension Act of 2008; Public Law 110-343)**: phased-in broker cost basis reporting on Form 1099-B with effective dates: **STOCK acquired on or after JANUARY 1, 2011**; **MUTUAL FUND / DRIP STOCK acquired on or after JANUARY 1, 2012**; **DEBT INSTRUMENTS AND OPTIONS acquired on or after JANUARY 1, 2014** ([IRS Notice 2011-56 — Cost Basis Reporting Transition Rules](https://www.irs.gov/pub/irs-drop/n-11-56.pdf)). Pre-effective-date holdings are "non-covered" and the taxpayer (not the broker) bears the burden of basis tracking.
//! - **Treas. Reg. § 1.1012-1(e) Wash Sale Property Cost Basis**: when a wash sale occurs under § 1091, the disallowed loss is added to the basis of the replacement security, and the holding period of the replacement security tacks onto the original security's holding period. § 1012 sets the BASE cost; § 1091(d) provides the wash-sale BASIS ADJUSTMENT.
//! - **Treas. Reg. § 1.1012-1(a) Property Includes**: all forms of property — real estate, personal property, stock, securities, debt instruments, options, partnership interests, intellectual property — fall under the § 1012(a) cost-basis general rule absent a specific override.
//! - **Companion Provisions**: § 1001 (gain or loss recognized = amount realized − adjusted basis); § 1011 (adjusted basis for determining gain or loss); § 1014 (basis stepped-up at death — overrides § 1012 for inherited property); § 1015 (basis of gifts — overrides § 1012); § 1031 (like-kind exchange basis carryover); § 1091 (wash sale basis adjustment); § 164(d) (real property tax allocation); § 471 / § 472 (inventory methods including LIFO); § 1295 (PFIC qualified electing fund); subchapters C / K / P (override provisions cross-referenced by § 1012(a)).

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const IRC_1012_RIC_ACCOUNT_SEPARATION_CUTOFF_YEAR: u32 = 2012;
pub const IRC_1012_RIC_ACCOUNT_SEPARATION_CUTOFF_MONTH: u32 = 1;
pub const IRC_1012_RIC_ACCOUNT_SEPARATION_CUTOFF_DAY: u32 = 1;
pub const IRC_1012_DRIP_EFFECTIVE_DATE_YEAR: u32 = 2012;
pub const IRC_1012_DRIP_EFFECTIVE_DATE_MONTH: u32 = 1;
pub const IRC_1012_DRIP_EFFECTIVE_DATE_DAY: u32 = 1;
pub const IRC_1012_STOCK_COST_BASIS_REPORTING_START_YEAR: u32 = 2011;
pub const IRC_1012_MUTUAL_FUND_DRIP_COST_BASIS_REPORTING_START_YEAR: u32 = 2012;
pub const IRC_1012_DEBT_OPTIONS_COST_BASIS_REPORTING_START_YEAR: u32 = 2014;
pub const IRC_1012_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyType {
    StockNonRicEquitySecurity,
    RegulatedInvestmentCompanyMutualFundShares,
    DividendReinvestmentPlanStock,
    DebtInstrumentBond,
    Option,
    RealEstate,
    PersonalProperty,
    PartnershipInterestUnderSubchapterK,
    StockUnderSubchapterCNotCoveredByGeneralRule,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BasisMethodElected {
    SpecificIdentificationUnderTreasReg1_1012_1C,
    FifoDefaultWhenNoSpecificIdentification,
    AverageCostSingleCategoryForRicOrDripStock,
    LifoMethodNotAllowedForSecurities,
    OtherStandardCostMethod,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AcquisitionDateStatus {
    AcquiredBeforeJanuary1_2012Pre2012NonCovered,
    AcquiredOnOrAfterJanuary1_2012Post2012Covered,
    AcquiredBeforeJanuary1_2011PreStockReportingNonCovered,
    AcquiredOnOrAfterJanuary1_2011PostStockReportingCovered,
    AcquiredBeforeJanuary1_2014PreDebtOptionsReportingNonCovered,
    AcquiredOnOrAfterJanuary1_2014PostDebtOptionsReportingCovered,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAspect {
    GeneralCostBasisDeterminationUnderSection1012A,
    SpecificIdentificationOrFifoDefaultUnderTreasReg1_1012_1C,
    AverageCostMethodForRicOrDripUnderSection1012C2,
    AccountByAccountBasisTrackingUnderSection1012C1,
    Pre2012Post2012RicSeparateAccountUnderSection1012C3,
    DripStockMethodUnderSection1012D,
    RealPropertyTaxesExclusionUnderSection1012BAndSection164D,
    WashSaleBasisAdjustmentUnderSection1091D,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section1012Mode {
    NotApplicablePropertyUnderSubchapterCOrK,
    CompliantCostBasisGeneralRuleSection1012A,
    CompliantSpecificIdentificationUnderTreasReg1_1012_1C,
    CompliantFifoDefaultWhenNoSpecificIdentification,
    CompliantAverageCostSingleCategoryForRicShareUnderSection1012C2,
    CompliantPre2012Post2012RicSeparateAccountUnderSection1012C3,
    CompliantDripStockPostDecember31_2011UsesRicMethodUnderSection1012D,
    CompliantAccountByAccountBasisTrackingUnderSection1012C1,
    CompliantRealPropertyTaxesExcludedFromCostUnderSection164D,
    CompliantWashSaleBasisAdjustmentUnderSection1091D,
    ViolationLifoMethodAppliedToSecurity,
    ViolationRealPropertyTaxesIncludedInCostUnderSection164D,
    ViolationAverageCostMethodAppliedToNonRicSecurity,
    ViolationFailureToIdentifyLotsAtTimeOfSaleFifoDefaultApplies,
    ViolationPre2012Post2012RicStockNotSeparateAccount,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub property_type: PropertyType,
    pub basis_method_elected: BasisMethodElected,
    pub acquisition_date_status: AcquisitionDateStatus,
    pub compliance_aspect: ComplianceAspect,
    pub real_property_taxes_included_in_cost: bool,
    pub specific_identification_made_at_time_of_sale: bool,
    pub written_confirmation_of_identification_received: bool,
    pub pre_2012_and_post_2012_ric_stock_in_separate_accounts: bool,
    pub wash_sale_disallowed_loss_added_to_replacement_basis: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section1012Mode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type Section1012Input = Input;
pub type Section1012Output = Output;
pub type Section1012Result = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "IRC § 1012(a) General Rule — basis of property = COST of such property, except as otherwise provided in subchapters C (corporate distributions and adjustments), K (partners and partnerships), and P (capital gains and losses)".to_string(),
        "IRC § 1012(b) Real Property Taxes Exclusion — cost may NOT include real property taxes treated as imposed on taxpayer under § 164(d); cost of real property excludes portion of property taxes attributable to pre-acquisition period".to_string(),
        "IRC § 1012(c)(1) Account-by-Account Basis Tracking — for specified securities sold after applicable dates, conventions prescribed by regulations under § 1012 applied on ACCOUNT BY ACCOUNT BASIS".to_string(),
        "IRC § 1012(c)(2) Average Cost Method for RIC Shares — stock in regulated investment company eligible for AVERAGE COST METHOD of basis determination; AVERAGE COST SINGLE CATEGORY (ACSC) total cost / total shares".to_string(),
        "IRC § 1012(c)(3) Pre-2012 / Post-2012 RIC Stock Separate Accounts — RIC stock acquired BEFORE JANUARY 1, 2012 treated as SEPARATE ACCOUNT from RIC stock acquired ON OR AFTER such date; bifurcation prevents basis-method election on post-2012 stock from affecting pre-2012 (non-covered) shares".to_string(),
        "IRC § 1012(d) Dividend Reinvestment Plans — stock acquired AFTER DECEMBER 31, 2011 in connection with DIVIDEND REINVESTMENT PLAN uses one of methods for determining basis of RIC stock (typically average cost)".to_string(),
        "Treas. Reg. § 1.1012-1(c) Specific Identification — taxpayer may SPECIFICALLY IDENTIFY shares sold by (1) at time of sale, designating specific lot to broker / transfer agent; AND (2) within reasonable time receiving written confirmation; specific identification typically produces LOWEST tax outcome with proper planning".to_string(),
        "Treas. Reg. § 1.1012-1(c)(1) FIFO Default Rule — if specific identification NOT made at time of sale, FIRST-IN FIRST-OUT (FIFO) method is DEFAULT for determining shares sold; FIFO generally produces HIGHEST tax bill (oldest / most-appreciated shares first)".to_string(),
        "LIFO Not Allowed for Securities — LAST-IN FIRST-OUT (LIFO) method NOT PERMITTED for stock or securities under § 1012; LIFO allowed only for inventory under § 471 / § 472".to_string(),
        "Cost Basis Reporting Reform (Energy Improvement and Extension Act of 2008; Public Law 110-343) — phased-in broker cost basis reporting on Form 1099-B: STOCK acquired on or after January 1, 2011; MUTUAL FUND / DRIP STOCK acquired on or after January 1, 2012; DEBT INSTRUMENTS AND OPTIONS acquired on or after January 1, 2014; pre-effective-date holdings are 'non-covered' and taxpayer bears burden of basis tracking".to_string(),
        "Treas. Reg. § 1.1012-1(e) Wash Sale Property Cost Basis — when wash sale occurs under § 1091, disallowed loss is added to basis of replacement security; holding period of replacement security tacks onto original security's holding period; § 1012 sets BASE cost, § 1091(d) provides wash-sale BASIS ADJUSTMENT".to_string(),
        "Treas. Reg. § 1.1012-1(a) Property Includes — all forms of property (real estate, personal property, stock, securities, debt instruments, options, partnership interests, intellectual property) fall under § 1012(a) cost-basis general rule absent specific override".to_string(),
        "Companion Provisions — § 1001 (gain / loss = amount realized − adjusted basis); § 1011 (adjusted basis); § 1014 (basis stepped-up at death — overrides § 1012 for inherited property); § 1015 (basis of gifts — overrides § 1012); § 1031 (like-kind exchange basis carryover); § 1091 (wash sale basis adjustment); § 164(d) (real property tax allocation); § 471 / § 472 (inventory methods including LIFO); § 1295 (PFIC qualified electing fund); subchapters C / K / P (override provisions)".to_string(),
        "Cornell LII 26 USC § 1012 — primary statutory text".to_string(),
        "Bloomberg Tax Sec. 1012 — comprehensive code commentary".to_string(),
        "Tax Notes IRC Section 1012 — practitioner reference".to_string(),
        "CCH AnswerConnect § 1012 Cost Basis of Property — practitioner guide".to_string(),
        "IRS Notice 2011-56 — Cost Basis Reporting Transition Rules".to_string(),
        "26 CFR § 1.1012-1 — Basis of Property regulation (Treasury Regulation)".to_string(),
    ];

    if matches!(
        input.property_type,
        PropertyType::PartnershipInterestUnderSubchapterK
            | PropertyType::StockUnderSubchapterCNotCoveredByGeneralRule
    ) {
        return Output {
            mode: Section1012Mode::NotApplicablePropertyUnderSubchapterCOrK,
            statutory_basis: "IRC § 1012(a) — basis general rule does not apply to property under subchapter C (corporate distributions) or subchapter K (partners and partnerships)".to_string(),
            notes: "NOT APPLICABLE: property is subject to override provisions under subchapter C (corporate distributions) or subchapter K (partners and partnerships); § 1012(a) cost-basis general rule overridden by subchapter-specific basis rules; consult § 1361-1379 (S corps), § 701-777 (partnerships), or § 301-385 (corporations) as applicable.".to_string(),
            citations,
        };
    }

    if input.basis_method_elected == BasisMethodElected::LifoMethodNotAllowedForSecurities {
        return Output {
            mode: Section1012Mode::ViolationLifoMethodAppliedToSecurity,
            statutory_basis: "Treas. Reg. § 1.1012-1 — LIFO method NOT permitted for stock or securities under § 1012".to_string(),
            notes: "VIOLATION: LIFO (Last-In, First-Out) method applied to stock or security; LIFO is NOT PERMITTED under § 1012 / Treas. Reg. § 1.1012-1; LIFO is allowed only for INVENTORY under § 471 / § 472; for securities, taxpayer must use specific identification, FIFO default, or (for RICs / DRIPs) average cost.".to_string(),
            citations,
        };
    }

    match input.compliance_aspect {
        ComplianceAspect::GeneralCostBasisDeterminationUnderSection1012A => Output {
            mode: Section1012Mode::CompliantCostBasisGeneralRuleSection1012A,
            statutory_basis: "IRC § 1012(a) — basis of property = cost".to_string(),
            notes: "COMPLIANT: basis of property determined under § 1012(a) general rule (cost of such property); applies absent override under subchapter C / K / P, § 1014 stepped-up basis at death, § 1015 basis of gifts, or other specific override.".to_string(),
            citations,
        },
        ComplianceAspect::SpecificIdentificationOrFifoDefaultUnderTreasReg1_1012_1C => {
            if input.specific_identification_made_at_time_of_sale
                && input.written_confirmation_of_identification_received
            {
                Output {
                    mode: Section1012Mode::CompliantSpecificIdentificationUnderTreasReg1_1012_1C,
                    statutory_basis: "Treas. Reg. § 1.1012-1(c) — specific identification properly made and confirmed".to_string(),
                    notes: "COMPLIANT: taxpayer specifically identified shares sold at time of sale AND received written confirmation from broker / transfer agent; specific identification under Treas. Reg. § 1.1012-1(c) preserved; taxpayer obtained intended tax outcome.".to_string(),
                    citations,
                }
            } else if input.basis_method_elected
                == BasisMethodElected::FifoDefaultWhenNoSpecificIdentification
            {
                Output {
                    mode: Section1012Mode::CompliantFifoDefaultWhenNoSpecificIdentification,
                    statutory_basis: "Treas. Reg. § 1.1012-1(c)(1) — FIFO default when no specific identification made".to_string(),
                    notes: "COMPLIANT (DEFAULT METHOD): no specific identification made; FIFO (First-In, First-Out) default rule applied per Treas. Reg. § 1.1012-1(c)(1); oldest shares treated as sold first; this typically produces the highest tax bill (most-appreciated shares first).".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: Section1012Mode::ViolationFailureToIdentifyLotsAtTimeOfSaleFifoDefaultApplies,
                    statutory_basis: "Treas. Reg. § 1.1012-1(c) — specific identification requires designation at time of sale AND written confirmation".to_string(),
                    notes: "VIOLATION OF INTENDED METHOD: taxpayer attempted specific identification but failed to (a) designate the specific lot to broker / transfer agent AT TIME OF SALE; OR (b) receive written confirmation within reasonable time; failure causes FIFO default to apply per Treas. Reg. § 1.1012-1(c)(1); intended tax outcome lost.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::AverageCostMethodForRicOrDripUnderSection1012C2 => {
            let eligible_for_average_cost = matches!(
                input.property_type,
                PropertyType::RegulatedInvestmentCompanyMutualFundShares
                    | PropertyType::DividendReinvestmentPlanStock
            );
            if eligible_for_average_cost
                && input.basis_method_elected
                    == BasisMethodElected::AverageCostSingleCategoryForRicOrDripStock
            {
                Output {
                    mode: Section1012Mode::CompliantAverageCostSingleCategoryForRicShareUnderSection1012C2,
                    statutory_basis: "IRC § 1012(c)(2) + Treas. Reg. § 1.1012-1(e) — average cost single category for RIC / DRIP stock".to_string(),
                    notes: "COMPLIANT: average cost single category (ACSC) method properly applied to RIC mutual fund shares or DRIP stock under § 1012(c)(2) / Treas. Reg. § 1.1012-1(e); total cost basis divided by total shares held to compute average per-share basis.".to_string(),
                    citations,
                }
            } else if input.basis_method_elected
                == BasisMethodElected::AverageCostSingleCategoryForRicOrDripStock
            {
                Output {
                    mode: Section1012Mode::ViolationAverageCostMethodAppliedToNonRicSecurity,
                    statutory_basis: "IRC § 1012(c)(2) — average cost method available only for RIC stock and DRIP stock".to_string(),
                    notes: "VIOLATION: average cost method applied to a non-RIC, non-DRIP security; § 1012(c)(2) average cost method limited to regulated investment companies (mutual funds) and dividend reinvestment plan stock acquired after December 31, 2011; for other securities, specific identification or FIFO default applies.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: Section1012Mode::CompliantCostBasisGeneralRuleSection1012A,
                    statutory_basis: "IRC § 1012(a) — non-average-cost method applies".to_string(),
                    notes: "COMPLIANT: average cost method NOT elected; standard cost basis under § 1012(a) applies for the property; basis determined per cost / specific identification / FIFO default as applicable.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::AccountByAccountBasisTrackingUnderSection1012C1 => Output {
            mode: Section1012Mode::CompliantAccountByAccountBasisTrackingUnderSection1012C1,
            statutory_basis: "IRC § 1012(c)(1) — account-by-account basis tracking".to_string(),
            notes: "COMPLIANT: § 1012(c)(1) requires account-by-account application of basis conventions; taxpayer holding same security across multiple brokerage accounts treats each account as separate for FIFO / specific-identification / average-cost election purposes; multi-account aggregation prohibited for purposes of basis determination.".to_string(),
            citations,
        },
        ComplianceAspect::Pre2012Post2012RicSeparateAccountUnderSection1012C3 => {
            if input.pre_2012_and_post_2012_ric_stock_in_separate_accounts {
                Output {
                    mode: Section1012Mode::CompliantPre2012Post2012RicSeparateAccountUnderSection1012C3,
                    statutory_basis: "IRC § 1012(c)(3) — pre-2012 and post-2012 RIC stock in separate accounts".to_string(),
                    notes: "COMPLIANT: § 1012(c)(3) separate-account requirement satisfied; RIC stock acquired before January 1, 2012 treated as separate account from RIC stock acquired on or after that date; basis-method election on post-2012 stock does not affect pre-2012 (non-covered) shares.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: Section1012Mode::ViolationPre2012Post2012RicStockNotSeparateAccount,
                    statutory_basis: "IRC § 1012(c)(3) — pre-2012 and post-2012 RIC stock must be in separate accounts".to_string(),
                    notes: "VIOLATION: pre-2012 and post-2012 RIC stock not maintained in separate accounts as required by § 1012(c)(3); pre-2012 non-covered shares improperly aggregated with post-2012 covered shares; basis tracking compromised; broker 1099-B reporting may be inaccurate.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::DripStockMethodUnderSection1012D => {
            if input.property_type == PropertyType::DividendReinvestmentPlanStock
                && matches!(
                    input.acquisition_date_status,
                    AcquisitionDateStatus::AcquiredOnOrAfterJanuary1_2012Post2012Covered
                )
            {
                Output {
                    mode: Section1012Mode::CompliantDripStockPostDecember31_2011UsesRicMethodUnderSection1012D,
                    statutory_basis: "IRC § 1012(d) — DRIP stock acquired after December 31, 2011 uses RIC-stock basis methods".to_string(),
                    notes: "COMPLIANT: DRIP stock acquired after December 31, 2011 properly uses one of the methods available for RIC stock under § 1012(c)(2) (typically average cost single category) per § 1012(d).".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: Section1012Mode::CompliantCostBasisGeneralRuleSection1012A,
                    statutory_basis: "IRC § 1012(a) — DRIP § 1012(d) method not triggered".to_string(),
                    notes: "NOT TRIGGERED: property is not DRIP stock acquired after December 31, 2011; § 1012(d) RIC-method election does not apply; standard § 1012(a) cost-basis general rule governs.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::RealPropertyTaxesExclusionUnderSection1012BAndSection164D => {
            if input.real_property_taxes_included_in_cost {
                Output {
                    mode: Section1012Mode::ViolationRealPropertyTaxesIncludedInCostUnderSection164D,
                    statutory_basis: "IRC § 1012(b) + § 164(d) — real property taxes treated as imposed on taxpayer must be excluded from cost".to_string(),
                    notes: "VIOLATION: real property taxes treated as imposed on the taxpayer under § 164(d) improperly included in the property's cost basis; § 1012(b) requires exclusion; taxpayer must recompute basis excluding the pre-acquisition property tax portion.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: Section1012Mode::CompliantRealPropertyTaxesExcludedFromCostUnderSection164D,
                    statutory_basis: "IRC § 1012(b) — real property taxes properly excluded from cost".to_string(),
                    notes: "COMPLIANT: real property taxes treated as imposed on the taxpayer under § 164(d) properly excluded from the property's cost basis per § 1012(b).".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::WashSaleBasisAdjustmentUnderSection1091D => {
            if input.wash_sale_disallowed_loss_added_to_replacement_basis {
                Output {
                    mode: Section1012Mode::CompliantWashSaleBasisAdjustmentUnderSection1091D,
                    statutory_basis: "Treas. Reg. § 1.1012-1(e) + § 1091(d) — wash sale disallowed loss added to replacement basis".to_string(),
                    notes: "COMPLIANT: wash sale under § 1091 properly handled; disallowed loss added to basis of replacement security per § 1091(d); holding period of replacement security tacks onto original security's holding period; § 1012 provides base cost, § 1091(d) provides wash-sale basis adjustment.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: Section1012Mode::CompliantCostBasisGeneralRuleSection1012A,
                    statutory_basis: "IRC § 1012(a) — no wash sale adjustment triggered".to_string(),
                    notes: "NOT TRIGGERED: no wash sale identified or basis adjustment not yet required; § 1012(a) general cost rule applies.".to_string(),
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
            property_type: PropertyType::StockNonRicEquitySecurity,
            basis_method_elected: BasisMethodElected::OtherStandardCostMethod,
            acquisition_date_status:
                AcquisitionDateStatus::AcquiredOnOrAfterJanuary1_2011PostStockReportingCovered,
            compliance_aspect: ComplianceAspect::GeneralCostBasisDeterminationUnderSection1012A,
            real_property_taxes_included_in_cost: false,
            specific_identification_made_at_time_of_sale: false,
            written_confirmation_of_identification_received: false,
            pre_2012_and_post_2012_ric_stock_in_separate_accounts: true,
            wash_sale_disallowed_loss_added_to_replacement_basis: false,
        }
    }

    #[test]
    fn partnership_interest_under_subchapter_k_not_applicable() {
        let mut input = baseline_input();
        input.property_type = PropertyType::PartnershipInterestUnderSubchapterK;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1012Mode::NotApplicablePropertyUnderSubchapterCOrK
        );
    }

    #[test]
    fn lifo_method_applied_to_security_violation() {
        let mut input = baseline_input();
        input.basis_method_elected = BasisMethodElected::LifoMethodNotAllowedForSecurities;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1012Mode::ViolationLifoMethodAppliedToSecurity
        );
    }

    #[test]
    fn general_cost_basis_rule_compliant() {
        let output = check(&baseline_input());
        assert_eq!(
            output.mode,
            Section1012Mode::CompliantCostBasisGeneralRuleSection1012A
        );
    }

    #[test]
    fn specific_identification_with_confirmation_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::SpecificIdentificationOrFifoDefaultUnderTreasReg1_1012_1C;
        input.basis_method_elected =
            BasisMethodElected::SpecificIdentificationUnderTreasReg1_1012_1C;
        input.specific_identification_made_at_time_of_sale = true;
        input.written_confirmation_of_identification_received = true;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1012Mode::CompliantSpecificIdentificationUnderTreasReg1_1012_1C
        );
    }

    #[test]
    fn fifo_default_when_no_specific_identification_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::SpecificIdentificationOrFifoDefaultUnderTreasReg1_1012_1C;
        input.basis_method_elected = BasisMethodElected::FifoDefaultWhenNoSpecificIdentification;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1012Mode::CompliantFifoDefaultWhenNoSpecificIdentification
        );
    }

    #[test]
    fn specific_identification_without_confirmation_falls_to_fifo_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::SpecificIdentificationOrFifoDefaultUnderTreasReg1_1012_1C;
        input.basis_method_elected =
            BasisMethodElected::SpecificIdentificationUnderTreasReg1_1012_1C;
        input.specific_identification_made_at_time_of_sale = true;
        input.written_confirmation_of_identification_received = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1012Mode::ViolationFailureToIdentifyLotsAtTimeOfSaleFifoDefaultApplies
        );
    }

    #[test]
    fn average_cost_for_ric_mutual_fund_compliant() {
        let mut input = baseline_input();
        input.property_type = PropertyType::RegulatedInvestmentCompanyMutualFundShares;
        input.basis_method_elected = BasisMethodElected::AverageCostSingleCategoryForRicOrDripStock;
        input.compliance_aspect = ComplianceAspect::AverageCostMethodForRicOrDripUnderSection1012C2;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1012Mode::CompliantAverageCostSingleCategoryForRicShareUnderSection1012C2
        );
    }

    #[test]
    fn average_cost_for_drip_stock_compliant() {
        let mut input = baseline_input();
        input.property_type = PropertyType::DividendReinvestmentPlanStock;
        input.basis_method_elected = BasisMethodElected::AverageCostSingleCategoryForRicOrDripStock;
        input.compliance_aspect = ComplianceAspect::AverageCostMethodForRicOrDripUnderSection1012C2;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1012Mode::CompliantAverageCostSingleCategoryForRicShareUnderSection1012C2
        );
    }

    #[test]
    fn average_cost_for_non_ric_non_drip_security_violation() {
        let mut input = baseline_input();
        input.property_type = PropertyType::StockNonRicEquitySecurity;
        input.basis_method_elected = BasisMethodElected::AverageCostSingleCategoryForRicOrDripStock;
        input.compliance_aspect = ComplianceAspect::AverageCostMethodForRicOrDripUnderSection1012C2;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1012Mode::ViolationAverageCostMethodAppliedToNonRicSecurity
        );
    }

    #[test]
    fn account_by_account_basis_tracking_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::AccountByAccountBasisTrackingUnderSection1012C1;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1012Mode::CompliantAccountByAccountBasisTrackingUnderSection1012C1
        );
    }

    #[test]
    fn pre_2012_post_2012_ric_separate_accounts_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::Pre2012Post2012RicSeparateAccountUnderSection1012C3;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1012Mode::CompliantPre2012Post2012RicSeparateAccountUnderSection1012C3
        );
    }

    #[test]
    fn pre_2012_post_2012_ric_not_separate_accounts_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::Pre2012Post2012RicSeparateAccountUnderSection1012C3;
        input.pre_2012_and_post_2012_ric_stock_in_separate_accounts = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1012Mode::ViolationPre2012Post2012RicStockNotSeparateAccount
        );
    }

    #[test]
    fn drip_stock_post_2012_compliant() {
        let mut input = baseline_input();
        input.property_type = PropertyType::DividendReinvestmentPlanStock;
        input.acquisition_date_status =
            AcquisitionDateStatus::AcquiredOnOrAfterJanuary1_2012Post2012Covered;
        input.compliance_aspect = ComplianceAspect::DripStockMethodUnderSection1012D;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1012Mode::CompliantDripStockPostDecember31_2011UsesRicMethodUnderSection1012D
        );
    }

    #[test]
    fn real_property_taxes_excluded_from_cost_compliant() {
        let mut input = baseline_input();
        input.property_type = PropertyType::RealEstate;
        input.compliance_aspect =
            ComplianceAspect::RealPropertyTaxesExclusionUnderSection1012BAndSection164D;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1012Mode::CompliantRealPropertyTaxesExcludedFromCostUnderSection164D
        );
    }

    #[test]
    fn real_property_taxes_included_in_cost_violation() {
        let mut input = baseline_input();
        input.property_type = PropertyType::RealEstate;
        input.compliance_aspect =
            ComplianceAspect::RealPropertyTaxesExclusionUnderSection1012BAndSection164D;
        input.real_property_taxes_included_in_cost = true;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1012Mode::ViolationRealPropertyTaxesIncludedInCostUnderSection164D
        );
    }

    #[test]
    fn wash_sale_basis_adjustment_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::WashSaleBasisAdjustmentUnderSection1091D;
        input.wash_sale_disallowed_loss_added_to_replacement_basis = true;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1012Mode::CompliantWashSaleBasisAdjustmentUnderSection1091D
        );
    }

    #[test]
    fn constants_pin_statutory_facts() {
        assert_eq!(IRC_1012_RIC_ACCOUNT_SEPARATION_CUTOFF_YEAR, 2012);
        assert_eq!(IRC_1012_RIC_ACCOUNT_SEPARATION_CUTOFF_MONTH, 1);
        assert_eq!(IRC_1012_RIC_ACCOUNT_SEPARATION_CUTOFF_DAY, 1);
        assert_eq!(IRC_1012_DRIP_EFFECTIVE_DATE_YEAR, 2012);
        assert_eq!(IRC_1012_DRIP_EFFECTIVE_DATE_MONTH, 1);
        assert_eq!(IRC_1012_DRIP_EFFECTIVE_DATE_DAY, 1);
        assert_eq!(IRC_1012_STOCK_COST_BASIS_REPORTING_START_YEAR, 2011);
        assert_eq!(
            IRC_1012_MUTUAL_FUND_DRIP_COST_BASIS_REPORTING_START_YEAR,
            2012
        );
        assert_eq!(IRC_1012_DEBT_OPTIONS_COST_BASIS_REPORTING_START_YEAR, 2014);
        assert_eq!(IRC_1012_BASIS_POINT_DENOMINATOR, 10_000);
    }

    #[test]
    fn citation_contains_landmarks() {
        let output = check(&baseline_input());
        let joined = output.citations.join("\n");
        assert!(joined.contains("§ 1012(a)"));
        assert!(joined.contains("§ 1012(b)"));
        assert!(joined.contains("§ 1012(c)(1)"));
        assert!(joined.contains("§ 1012(c)(2)"));
        assert!(joined.contains("§ 1012(c)(3)"));
        assert!(joined.contains("§ 1012(d)"));
        assert!(joined.contains("§ 1.1012-1"));
        assert!(joined.contains("§ 1091"));
        assert!(joined.contains("§ 164(d)"));
        assert!(joined.contains("§ 1014"));
        assert!(joined.contains("§ 1015"));
        assert!(joined.contains("§ 1031"));
        assert!(joined.contains("FIFO"));
        assert!(joined.contains("LIFO"));
        assert!(joined.contains("SPECIFICALLY IDENTIFY"));
        assert!(joined.contains("AVERAGE COST"));
        assert!(joined.contains("JANUARY 1, 2012"));
        assert!(joined.contains("DECEMBER 31, 2011"));
        assert!(joined.contains("January 1, 2011"));
        assert!(joined.contains("January 1, 2014"));
        assert!(joined.contains("Public Law 110-343"));
        assert!(joined.contains("Energy Improvement"));
    }
}
