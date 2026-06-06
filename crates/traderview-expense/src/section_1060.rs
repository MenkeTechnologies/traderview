//! IRC § 1060 Special Allocation Rules for Certain Asset
//! Acquisitions — pure-compute compliance check for purchase
//! price allocation in applicable asset acquisitions reported
//! on Form 8594.
//!
//! § 1060 requires both buyer (transferee) and seller
//! (transferor) in an applicable asset acquisition to allocate
//! the consideration paid / received across **SEVEN ASSET
//! CLASSES** using the **RESIDUAL METHOD** (cross-references
//! § 338(b)(5)). The seven classes proceed sequentially:
//! **Class I** cash + general deposit accounts; **Class II**
//! CDs + US Government securities + readily marketable stock
//! and securities + foreign currency; **Class III** mark-to-
//! market assets + accounts receivable + certain debt
//! instruments; **Class IV** inventory / stock in trade;
//! **Class V** all other tangibles (land + buildings +
//! equipment); **Class VI** § 197 intangibles other than
//! goodwill / going concern; **Class VII** goodwill and going
//! concern value (residual). § 1060 was enacted by Section 641
//! of the **Tax Reform Act of 1986** (Public Law 99-514) and
//! is implemented by **Treas. Reg. § 1.1060-1** with class
//! definitions cross-referenced to **Treas. Reg. § 1.338-6**.
//!
//! Web research (verified 2026-06-03):
//! - **Enactment**: § 1060 added by **Section 641 of the Tax Reform Act of 1986 (Public Law 99-514)**; subsequently amended by the Omnibus Budget Reconciliation Act of 1990 (Public Law 101-508) and the Revenue Reconciliation Act of 1993 (Public Law 103-66) ([Norton Rose Fulbright — Section 1060 and Purchase Price Allocations (December 2021)](https://www.projectfinance.law/publications/2021/december/section-1060-and-purchase-price-allocations); [Mondaq — Purchase Price Allocation Rules: Sections 1060, 338, And 197](https://www.mondaq.com/unitedstates/corporate-tax/24405/purchase-price-allocation-rules-sections-1060-338-and-197); [Cornell LII — 26 CFR § 1.1060-1 Special allocation rules for certain asset acquisitions](https://www.law.cornell.edu/cfr/text/26/1.1060-1); [IRS — Instructions for Form 8594 (Rev. November 2021)](https://www.irs.gov/pub/irs-pdf/i8594.pdf); [IRS — Instructions for Form 8594 (11/2021)](https://www.irs.gov/instructions/i8594); [LegalClarity — Form 8594 Classifications: The Seven Asset Classes](https://legalclarity.org/form-8594-classifications-the-seven-asset-classes/); [Henson Efron — Sellers and Buyers: Competing Interests](https://hensonefron.com/sellers-buyers-competing-interests/); [Mihama Acquisitions — Section 1060 Asset Allocation Whitepaper](https://mihamainc.com/mihama_section1060_whitepaper.html); [Stradley — Allocations of Purchase Price: A Zero-Sum Game?](https://www.stradley.com/business-vantage-point-blog/allocations-of-purchase-price-a-zero-sum-game); [LedgerFi — IRS Form 8594 Instructions: Guide for Business Acquisitions 2025](https://www.ledgerfi.co/resources/form-8594-asset-acquisition-guide-2025); [LegalClarity — How to Complete IRS Form 8594 for Asset Allocation](https://legalclarity.org/how-to-complete-irs-form-8594-for-asset-allocation/); [MarketClutch — Understanding Section 1060: Special Allocation Rules](https://marketclutch.com/understanding-section-1060-special-allocation-rules-for-certain-asset-acquisitions/); [TFX — Guide to Form 8594 and Purchase Price Allocation (PPA)](https://tfx.tax/articles/tax-tips/form-8564-purchase-price-allocation)).
//! - **Applicable Asset Acquisition Definition (§ 1060(c))**: any transfer of a **GROUP OF ASSETS THAT CONSTITUTES A TRADE OR BUSINESS** in the hands of either the seller or buyer where the basis of the assets in the hands of the purchaser is determined wholly by reference to the consideration paid. **Goodwill or going concern value must attach (or could attach)** to such assets.
//! - **Residual Method Cross-Reference**: § 1060(a) requires allocation in the same manner as § 338(b)(5) — the residual method allocates consideration sequentially from Class I to Class VII, with each class receiving allocation up to fair market value before moving to the next class. Any remaining consideration after satisfying Classes I-VI flows to **Class VII (goodwill and going concern value)** as the residual.
//! - **Seven Asset Classes (Treas. Reg. § 1.338-6(b) / § 1.1060-1(c))**:
//!   - **CLASS I**: **Cash and general deposit accounts** (including checking and savings accounts but excluding CDs).
//!   - **CLASS II**: **Certificates of deposit + actively traded personal property + US Government securities + readily marketable stock or securities + foreign currency** within the meaning of § 1092(d).
//!   - **CLASS III**: **Mark-to-market assets** (e.g., § 475 securities) + **accounts receivable** + **mortgages** + **credit card receivables** + **certain other debt instruments**.
//!   - **CLASS IV**: **Stock in trade / inventory** under § 1221(a)(1) and property held primarily for sale to customers in the ordinary course of trade or business.
//!   - **CLASS V**: **All assets OTHER THAN Class I, II, III, IV, VI, or VII** assets — typically tangible operating assets including land + buildings + equipment + furniture + leasehold improvements.
//!   - **CLASS VI**: All **§ 197 intangibles** EXCEPT goodwill and going concern value (workforce in place + customer-based intangibles + supplier-based intangibles + government licenses + covenants not to compete + franchises + trademarks + trade names).
//!   - **CLASS VII**: **Goodwill and going concern value** (whether or not the goodwill or going concern value qualifies as a § 197 intangible) — pure residual.
//! - **Form 8594 (Asset Acquisition Statement Under § 1060) Filing Requirement**: both seller AND buyer must each file Form 8594 attached to their respective income tax returns for the year of sale (annual instructions revision dates available; latest Rev. November 2021).
//! - **Consistency Requirement**: § 1060(b) requires that the SELLER and BUYER allocate consideration in a CONSISTENT manner across all seven classes; substantial allocation discrepancies trigger IRS audit attention and may be challenged.
//! - **§ 197 Intangibles 15-Year Amortization Period**: Class VI § 197 intangibles AND Class VII goodwill / going concern both subject to **15-YEAR straight-line amortization** under § 197(a) (180-month period).
//! - **Tax Strategy Tension**: SELLER prefers allocation to **CAPITAL GAIN assets** (Class V tangibles + Class VI/VII intangibles → long-term capital gain at preferential rate); BUYER prefers allocation to **DEDUCTIBLE/AMORTIZABLE assets** (Class IV inventory deductible against revenue; Class V tangibles depreciated; Class VI/VII amortized over 15 years; avoid allocation to non-amortizable land). Negotiation tension creates real economic stakes.
//! - **Penalty Exposure for Inconsistent Allocation**: § 6662 accuracy-related penalty + § 6721 / § 6722 information return penalties + IRS authority to substitute its own allocation under § 1060(a) when seller and buyer allocations diverge.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const IRC_1060_ENACTMENT_TAX_REFORM_ACT_OF_1986_YEAR: u32 = 1986;
pub const IRC_1060_ENACTMENT_PUBLIC_LAW_CONGRESS_NUMBER: u32 = 99;
pub const IRC_1060_ENACTMENT_PUBLIC_LAW_ENACTMENT_NUMBER: u32 = 514;
pub const IRC_1060_ENACTMENT_ACT_SECTION_NUMBER: u32 = 641;
pub const IRC_1060_NUMBER_OF_ASSET_CLASSES: u32 = 7;
pub const IRC_1060_RESIDUAL_METHOD_CROSS_REFERENCE_SECTION: u32 = 338;
pub const IRC_1060_FORM_NUMBER: u32 = 8594;
pub const IRC_197_INTANGIBLE_AMORTIZATION_YEARS: u32 = 15;
pub const IRC_197_INTANGIBLE_AMORTIZATION_MONTHS: u32 = 180;
pub const IRC_1060_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionStatus {
    ApplicableAssetAcquisitionGoodwillOrGoingConcernAttaches,
    NonApplicableTransactionNoGoodwillNoGoingConcern,
    StockSaleNotAssetSaleNotSubjectToSection1060,
    BasisDeterminedByCarryoverNotConsideration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FilingComplianceStatus {
    BothBuyerAndSellerFiledForm8594,
    OnlyBuyerFiledForm8594,
    OnlySellerFiledForm8594,
    NeitherBuyerNorSellerFiledForm8594,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AllocationMethodStatus {
    ResidualMethodAppliedClassIThroughVIISequentially,
    NonResidualMethodAppliedFlatPercentageOrOther,
    GoodwillAllocatedBeforeAllOtherClassesSatisfied,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsistencyStatus {
    BuyerAndSellerAllocationConsistent,
    BuyerAndSellerAllocationMateriallyInconsistent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAspect {
    ApplicableAssetAcquisitionDetermination,
    Form8594FilingByBothParties,
    ResidualMethodAllocationClassIThroughVII,
    BuyerSellerAllocationConsistencyUnderSection1060B,
    FairMarketValueAtEachClassLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section1060Mode {
    NotApplicableNoGoodwillOrGoingConcernAttachesNotApplicableAssetAcquisition,
    NotApplicableStockSaleNotSubjectToSection1060,
    NotApplicableBasisDeterminedByCarryoverNotConsideration,
    CompliantBothBuyerAndSellerFiledForm8594,
    CompliantResidualMethodAppliedClassIThroughVIISequentially,
    CompliantBuyerAndSellerAllocationConsistentUnderSection1060B,
    CompliantFairMarketValueRespectedAtEachClassLevel,
    ViolationFailureToFileForm8594ByBothParties,
    ViolationOnlyOnePartyFiledForm8594,
    ViolationNonResidualMethodApplied,
    ViolationGoodwillAllocatedBeforeAllOtherClassesSatisfied,
    ViolationBuyerAndSellerAllocationMateriallyInconsistent,
    ViolationFairMarketValueOverstatedInLowerClass,
    ViolationFairMarketValueUnderstatedInLowerClass,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub transaction_status: TransactionStatus,
    pub compliance_aspect: ComplianceAspect,
    pub filing_compliance_status: FilingComplianceStatus,
    pub allocation_method_status: AllocationMethodStatus,
    pub consistency_status: ConsistencyStatus,
    pub fair_market_value_at_each_class_level_respected: bool,
    pub fair_market_value_overstated_in_lower_class: bool,
    pub fair_market_value_understated_in_lower_class: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section1060Mode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type Section1060Input = Input;
pub type Section1060Output = Output;
pub type Section1060Result = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "IRC § 1060 Special Allocation Rules for Certain Asset Acquisitions — enacted by Section 641 of the Tax Reform Act of 1986 (Public Law 99-514); subsequent amendments by Omnibus Budget Reconciliation Act of 1990 (Public Law 101-508) and Revenue Reconciliation Act of 1993 (Public Law 103-66)".to_string(),
        "IRC § 1060(a) — both transferee (buyer) and transferor (seller) must allocate consideration paid / received in the transaction among the assets transferred in the same manner as amounts are allocated under § 338(b)(5) residual method".to_string(),
        "IRC § 1060(b) Consistency Requirement — seller and buyer must allocate consideration in a consistent manner across all seven asset classes; substantial allocation discrepancies trigger IRS audit attention and IRS authority to substitute its own allocation".to_string(),
        "IRC § 1060(c) Applicable Asset Acquisition Definition — any transfer of a GROUP OF ASSETS THAT CONSTITUTES A TRADE OR BUSINESS in the hands of either seller or buyer where the basis of the assets in the hands of the purchaser is determined wholly by reference to the consideration paid; goodwill or going concern value must attach (or could attach) to such assets".to_string(),
        "Treas. Reg. § 1.1060-1 Implementing Regulations — provides operational rules for § 1060 application; cross-references Treas. Reg. § 1.338-6 for the seven-asset-class definitions and residual method procedure".to_string(),
        "Treas. Reg. § 1.338-6(b) Seven Asset Classes — CLASS I: cash and general deposit accounts (checking + savings; excluding CDs); CLASS II: CDs + actively traded personal property + US Government securities + readily marketable stock and securities + foreign currency (§ 1092(d)); CLASS III: mark-to-market assets (§ 475 securities) + accounts receivable + mortgages + credit card receivables + certain other debt instruments; CLASS IV: inventory / stock in trade under § 1221(a)(1); CLASS V: all other tangibles (land + buildings + equipment + furniture + leasehold improvements); CLASS VI: § 197 intangibles other than goodwill / going concern (workforce in place + customer-based intangibles + supplier-based intangibles + government licenses + covenants not to compete + franchises + trademarks + trade names); CLASS VII: goodwill and going concern value (pure residual)".to_string(),
        "IRS Form 8594 Asset Acquisition Statement Under § 1060 — both seller AND buyer must each file Form 8594 attached to their respective income tax returns for the year of sale; current instructions: IRS Form 8594 Instructions (Rev. November 2021)".to_string(),
        "Residual Method Sequential Allocation — Class I receives consideration up to fair market value; remainder allocated to Class II up to FMV; remainder to Class III up to FMV; remainder to Class IV up to FMV; remainder to Class V up to FMV; remainder to Class VI up to FMV; ALL REMAINING CONSIDERATION after satisfying Classes I-VI flows to Class VII (goodwill and going concern value)".to_string(),
        "IRC § 197 15-Year Amortization Period — Class VI § 197 intangibles AND Class VII goodwill / going concern subject to 15-year straight-line amortization under § 197(a) (180-month period)".to_string(),
        "IRC § 6662 Accuracy-Related Penalty + § 6721 + § 6722 Information Return Penalties — penalty exposure for inconsistent allocation between buyer and seller; IRS authority under § 1060(a) to substitute its own allocation when seller and buyer allocations diverge materially".to_string(),
        "Seller / Buyer Tax Strategy Tension — SELLER prefers allocation to capital-gain assets (Class V tangibles + Class VI/VII intangibles → long-term capital gain at preferential rate); BUYER prefers allocation to deductible / amortizable assets (Class IV inventory deductible against revenue; Class V tangibles depreciated; Class VI/VII amortized over 15 years; avoid allocation to non-amortizable land). Negotiation tension creates real economic stakes.".to_string(),
        "Stock Sale Distinction — § 1060 applies ONLY to asset sales of a trade or business; stock sales (sale of corporate stock as opposed to underlying assets) are NOT subject to § 1060 (purchaser inherits inside-basis carryover; § 338(h)(10) or § 338(g) election may convert to asset-sale treatment with § 1060-like consequences)".to_string(),
        "Carryover Basis Exception — § 1060 applies only when purchaser's basis is determined WHOLLY by reference to consideration paid; transactions involving carryover-basis structures (e.g., § 1031 like-kind exchange components, § 351 incorporations, § 721 contributions) are outside § 1060 scope".to_string(),
        "Norton Rose Fulbright (December 2021) — Section 1060 and Purchase Price Allocations — practitioner overview".to_string(),
        "LegalClarity — Form 8594 Classifications: The Seven Asset Classes — class-by-class practitioner guide".to_string(),
        "Henson Efron — Sellers and Buyers: Competing Interests — M&A negotiation tension overview".to_string(),
    ];

    match input.transaction_status {
        TransactionStatus::NonApplicableTransactionNoGoodwillNoGoingConcern => {
            return Output {
                mode: Section1060Mode::NotApplicableNoGoodwillOrGoingConcernAttachesNotApplicableAssetAcquisition,
                statutory_basis: "IRC § 1060(c) — applicable asset acquisition requires goodwill or going concern value to attach or potentially attach".to_string(),
                notes: "NOT APPLICABLE: transaction is not an applicable asset acquisition under § 1060(c) because no goodwill or going concern value attaches or could attach; § 1060 allocation rules and Form 8594 filing not required.".to_string(),
                citations,
            };
        }
        TransactionStatus::StockSaleNotAssetSaleNotSubjectToSection1060 => {
            return Output {
                mode: Section1060Mode::NotApplicableStockSaleNotSubjectToSection1060,
                statutory_basis: "IRC § 1060(c) — stock sales not subject to § 1060 (purchaser inherits inside-basis carryover absent § 338 election)".to_string(),
                notes: "NOT APPLICABLE: stock sale not subject to § 1060 allocation rules; purchaser inherits inside-basis carryover; § 338(h)(10) or § 338(g) election may convert to asset-sale treatment with § 1060-like consequences.".to_string(),
                citations,
            };
        }
        TransactionStatus::BasisDeterminedByCarryoverNotConsideration => {
            return Output {
                mode: Section1060Mode::NotApplicableBasisDeterminedByCarryoverNotConsideration,
                statutory_basis: "IRC § 1060(c) — § 1060 applies only when purchaser's basis is determined WHOLLY by reference to consideration paid".to_string(),
                notes: "NOT APPLICABLE: transaction involves carryover-basis structure (§ 1031 like-kind exchange components, § 351 incorporations, § 721 partnership contributions); purchaser's basis is not determined wholly by consideration paid; § 1060 inapplicable.".to_string(),
                citations,
            };
        }
        TransactionStatus::ApplicableAssetAcquisitionGoodwillOrGoingConcernAttaches => {}
    }

    match input.compliance_aspect {
        ComplianceAspect::ApplicableAssetAcquisitionDetermination => Output {
            mode: Section1060Mode::CompliantBothBuyerAndSellerFiledForm8594,
            statutory_basis: "IRC § 1060(c) — applicable asset acquisition determination confirmed".to_string(),
            notes: "COMPLIANT: transaction qualifies as an applicable asset acquisition under § 1060(c); group of assets constitutes a trade or business; goodwill or going concern value attaches or could attach; purchaser's basis determined wholly by consideration paid.".to_string(),
            citations,
        },
        ComplianceAspect::Form8594FilingByBothParties => match input.filing_compliance_status {
            FilingComplianceStatus::BothBuyerAndSellerFiledForm8594 => Output {
                mode: Section1060Mode::CompliantBothBuyerAndSellerFiledForm8594,
                statutory_basis: "Treas. Reg. § 1.1060-1 + IRS Form 8594 Instructions — both buyer and seller filed Form 8594".to_string(),
                notes: "COMPLIANT: both buyer and seller filed Form 8594 Asset Acquisition Statement Under § 1060 attached to their respective income tax returns for the year of sale.".to_string(),
                citations,
            },
            FilingComplianceStatus::NeitherBuyerNorSellerFiledForm8594 => Output {
                mode: Section1060Mode::ViolationFailureToFileForm8594ByBothParties,
                statutory_basis: "Treas. Reg. § 1.1060-1 + § 6721 + § 6722 — failure to file Form 8594 by both parties".to_string(),
                notes: "VIOLATION: neither buyer nor seller filed Form 8594; § 6721 information return failure penalty applies to each non-filer; IRS authority to substitute its own allocation under § 1060(a).".to_string(),
                citations,
            },
            FilingComplianceStatus::OnlyBuyerFiledForm8594 | FilingComplianceStatus::OnlySellerFiledForm8594 => Output {
                mode: Section1060Mode::ViolationOnlyOnePartyFiledForm8594,
                statutory_basis: "Treas. Reg. § 1.1060-1 — Form 8594 must be filed by BOTH buyer and seller; one-party filing insufficient".to_string(),
                notes: "VIOLATION: only one party filed Form 8594; the non-filing party is subject to § 6721 / § 6722 information return penalties; IRS will reconcile allocations by examination.".to_string(),
                citations,
            },
        },
        ComplianceAspect::ResidualMethodAllocationClassIThroughVII => match input.allocation_method_status {
            AllocationMethodStatus::ResidualMethodAppliedClassIThroughVIISequentially => Output {
                mode: Section1060Mode::CompliantResidualMethodAppliedClassIThroughVIISequentially,
                statutory_basis: "IRC § 1060(a) + § 338(b)(5) + Treas. Reg. § 1.338-6 — residual method properly applied sequentially from Class I through Class VII".to_string(),
                notes: "COMPLIANT: residual method properly applied — consideration allocated sequentially from Class I (cash) through Class VI (§ 197 intangibles other than goodwill) up to fair market value at each class; all remaining consideration flowed to Class VII (goodwill and going concern value) as residual.".to_string(),
                citations,
            },
            AllocationMethodStatus::NonResidualMethodAppliedFlatPercentageOrOther => Output {
                mode: Section1060Mode::ViolationNonResidualMethodApplied,
                statutory_basis: "IRC § 1060(a) + § 338(b)(5) — residual method REQUIRED; flat-percentage or other allocation method prohibited".to_string(),
                notes: "VIOLATION: non-residual allocation method applied (flat percentage / pro rata / other); § 1060(a) requires sequential residual method per § 338(b)(5); IRS authority to substitute its own residual-method allocation.".to_string(),
                citations,
            },
            AllocationMethodStatus::GoodwillAllocatedBeforeAllOtherClassesSatisfied => Output {
                mode: Section1060Mode::ViolationGoodwillAllocatedBeforeAllOtherClassesSatisfied,
                statutory_basis: "IRC § 1060(a) + § 338(b)(5) + Treas. Reg. § 1.338-6 — Class VII goodwill must be PURE RESIDUAL after Classes I-VI satisfied at FMV".to_string(),
                notes: "VIOLATION: goodwill (Class VII) allocated before all Classes I-VI satisfied at fair market value; residual method requires Class VII to be the PURE RESIDUAL; misallocation triggers IRS reallocation authority.".to_string(),
                citations,
            },
        },
        ComplianceAspect::BuyerSellerAllocationConsistencyUnderSection1060B => match input.consistency_status {
            ConsistencyStatus::BuyerAndSellerAllocationConsistent => Output {
                mode: Section1060Mode::CompliantBuyerAndSellerAllocationConsistentUnderSection1060B,
                statutory_basis: "IRC § 1060(b) — buyer and seller allocation consistent across all seven asset classes".to_string(),
                notes: "COMPLIANT: buyer and seller allocated consideration in a CONSISTENT manner across all seven asset classes under § 1060(b); no material discrepancies.".to_string(),
                citations,
            },
            ConsistencyStatus::BuyerAndSellerAllocationMateriallyInconsistent => Output {
                mode: Section1060Mode::ViolationBuyerAndSellerAllocationMateriallyInconsistent,
                statutory_basis: "IRC § 1060(b) + § 6662 — buyer and seller allocation materially inconsistent triggers IRS reallocation authority and accuracy-related penalty exposure".to_string(),
                notes: "VIOLATION: buyer and seller allocations are materially inconsistent under § 1060(b); IRS audit attention + § 6662 accuracy-related penalty exposure + IRS authority to substitute its own allocation.".to_string(),
                citations,
            },
        },
        ComplianceAspect::FairMarketValueAtEachClassLevel => {
            if input.fair_market_value_overstated_in_lower_class {
                Output {
                    mode: Section1060Mode::ViolationFairMarketValueOverstatedInLowerClass,
                    statutory_basis: "Treas. Reg. § 1.338-6(b) + § 6662 — fair market value overstated in lower class to shift allocation away from goodwill".to_string(),
                    notes: "VIOLATION: fair market value overstated in a lower-numbered class (e.g., overstating tangible Class V to reduce residual Class VII goodwill); IRS reallocation authority + § 6662 accuracy-related penalty exposure.".to_string(),
                    citations,
                }
            } else if input.fair_market_value_understated_in_lower_class {
                Output {
                    mode: Section1060Mode::ViolationFairMarketValueUnderstatedInLowerClass,
                    statutory_basis: "Treas. Reg. § 1.338-6(b) + § 6662 — fair market value understated in lower class to shift allocation to goodwill".to_string(),
                    notes: "VIOLATION: fair market value understated in a lower-numbered class (e.g., understating Class V tangibles to inflate residual Class VII goodwill); IRS reallocation authority + § 6662 accuracy-related penalty exposure.".to_string(),
                    citations,
                }
            } else if input.fair_market_value_at_each_class_level_respected {
                Output {
                    mode: Section1060Mode::CompliantFairMarketValueRespectedAtEachClassLevel,
                    statutory_basis: "Treas. Reg. § 1.338-6(b) — fair market value respected at each class level".to_string(),
                    notes: "COMPLIANT: fair market value respected at each class level under Treas. Reg. § 1.338-6(b); residual method properly produces Class VII goodwill amount as the true residual after FMV-respecting Class I-VI allocation.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: Section1060Mode::ViolationFairMarketValueOverstatedInLowerClass,
                    statutory_basis: "Treas. Reg. § 1.338-6(b) — fair market value not respected at each class level".to_string(),
                    notes: "VIOLATION: fair market value not respected at each class level; allocation requires sustainable FMV evidence at each Class I-VI level.".to_string(),
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
            transaction_status:
                TransactionStatus::ApplicableAssetAcquisitionGoodwillOrGoingConcernAttaches,
            compliance_aspect: ComplianceAspect::Form8594FilingByBothParties,
            filing_compliance_status: FilingComplianceStatus::BothBuyerAndSellerFiledForm8594,
            allocation_method_status:
                AllocationMethodStatus::ResidualMethodAppliedClassIThroughVIISequentially,
            consistency_status: ConsistencyStatus::BuyerAndSellerAllocationConsistent,
            fair_market_value_at_each_class_level_respected: true,
            fair_market_value_overstated_in_lower_class: false,
            fair_market_value_understated_in_lower_class: false,
        }
    }

    #[test]
    fn no_goodwill_or_going_concern_not_applicable() {
        let mut input = baseline_input();
        input.transaction_status =
            TransactionStatus::NonApplicableTransactionNoGoodwillNoGoingConcern;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1060Mode::NotApplicableNoGoodwillOrGoingConcernAttachesNotApplicableAssetAcquisition
        );
    }

    #[test]
    fn stock_sale_not_applicable() {
        let mut input = baseline_input();
        input.transaction_status = TransactionStatus::StockSaleNotAssetSaleNotSubjectToSection1060;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1060Mode::NotApplicableStockSaleNotSubjectToSection1060
        );
    }

    #[test]
    fn carryover_basis_not_applicable() {
        let mut input = baseline_input();
        input.transaction_status = TransactionStatus::BasisDeterminedByCarryoverNotConsideration;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1060Mode::NotApplicableBasisDeterminedByCarryoverNotConsideration
        );
    }

    #[test]
    fn both_parties_filed_form_8594_compliant() {
        let output = check(&baseline_input());
        assert_eq!(
            output.mode,
            Section1060Mode::CompliantBothBuyerAndSellerFiledForm8594
        );
    }

    #[test]
    fn neither_party_filed_form_8594_violation() {
        let mut input = baseline_input();
        input.filing_compliance_status = FilingComplianceStatus::NeitherBuyerNorSellerFiledForm8594;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1060Mode::ViolationFailureToFileForm8594ByBothParties
        );
    }

    #[test]
    fn only_buyer_filed_form_8594_violation() {
        let mut input = baseline_input();
        input.filing_compliance_status = FilingComplianceStatus::OnlyBuyerFiledForm8594;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1060Mode::ViolationOnlyOnePartyFiledForm8594
        );
    }

    #[test]
    fn only_seller_filed_form_8594_violation() {
        let mut input = baseline_input();
        input.filing_compliance_status = FilingComplianceStatus::OnlySellerFiledForm8594;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1060Mode::ViolationOnlyOnePartyFiledForm8594
        );
    }

    #[test]
    fn residual_method_class_i_through_vii_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ResidualMethodAllocationClassIThroughVII;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1060Mode::CompliantResidualMethodAppliedClassIThroughVIISequentially
        );
    }

    #[test]
    fn non_residual_method_applied_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ResidualMethodAllocationClassIThroughVII;
        input.allocation_method_status =
            AllocationMethodStatus::NonResidualMethodAppliedFlatPercentageOrOther;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1060Mode::ViolationNonResidualMethodApplied
        );
    }

    #[test]
    fn goodwill_allocated_before_other_classes_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ResidualMethodAllocationClassIThroughVII;
        input.allocation_method_status =
            AllocationMethodStatus::GoodwillAllocatedBeforeAllOtherClassesSatisfied;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1060Mode::ViolationGoodwillAllocatedBeforeAllOtherClassesSatisfied
        );
    }

    #[test]
    fn buyer_seller_allocation_consistent_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::BuyerSellerAllocationConsistencyUnderSection1060B;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1060Mode::CompliantBuyerAndSellerAllocationConsistentUnderSection1060B
        );
    }

    #[test]
    fn buyer_seller_allocation_materially_inconsistent_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::BuyerSellerAllocationConsistencyUnderSection1060B;
        input.consistency_status =
            ConsistencyStatus::BuyerAndSellerAllocationMateriallyInconsistent;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1060Mode::ViolationBuyerAndSellerAllocationMateriallyInconsistent
        );
    }

    #[test]
    fn fair_market_value_respected_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::FairMarketValueAtEachClassLevel;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1060Mode::CompliantFairMarketValueRespectedAtEachClassLevel
        );
    }

    #[test]
    fn fair_market_value_overstated_in_lower_class_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::FairMarketValueAtEachClassLevel;
        input.fair_market_value_overstated_in_lower_class = true;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1060Mode::ViolationFairMarketValueOverstatedInLowerClass
        );
    }

    #[test]
    fn fair_market_value_understated_in_lower_class_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::FairMarketValueAtEachClassLevel;
        input.fair_market_value_understated_in_lower_class = true;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1060Mode::ViolationFairMarketValueUnderstatedInLowerClass
        );
    }

    #[test]
    fn applicable_asset_acquisition_determination_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ApplicableAssetAcquisitionDetermination;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1060Mode::CompliantBothBuyerAndSellerFiledForm8594
        );
    }

    #[test]
    fn constants_pin_statutory_facts() {
        assert_eq!(IRC_1060_ENACTMENT_TAX_REFORM_ACT_OF_1986_YEAR, 1986);
        assert_eq!(IRC_1060_ENACTMENT_PUBLIC_LAW_CONGRESS_NUMBER, 99);
        assert_eq!(IRC_1060_ENACTMENT_PUBLIC_LAW_ENACTMENT_NUMBER, 514);
        assert_eq!(IRC_1060_ENACTMENT_ACT_SECTION_NUMBER, 641);
        assert_eq!(IRC_1060_NUMBER_OF_ASSET_CLASSES, 7);
        assert_eq!(IRC_1060_RESIDUAL_METHOD_CROSS_REFERENCE_SECTION, 338);
        assert_eq!(IRC_1060_FORM_NUMBER, 8594);
        assert_eq!(IRC_197_INTANGIBLE_AMORTIZATION_YEARS, 15);
        assert_eq!(IRC_197_INTANGIBLE_AMORTIZATION_MONTHS, 180);
        assert_eq!(IRC_1060_BASIS_POINT_DENOMINATOR, 10_000);
    }

    #[test]
    fn citations_contain_landmarks() {
        let output = check(&baseline_input());
        let joined = output.citations.join("\n");
        assert!(joined.contains("IRC § 1060"));
        assert!(joined.contains("Section 641 of the Tax Reform Act of 1986"));
        assert!(joined.contains("Public Law 99-514"));
        assert!(joined.contains("§ 338(b)(5)"));
        assert!(joined.contains("Treas. Reg. § 1.1060-1"));
        assert!(joined.contains("Treas. Reg. § 1.338-6"));
        assert!(joined.contains("Form 8594"));
        assert!(joined.contains("CLASS I"));
        assert!(joined.contains("CLASS II"));
        assert!(joined.contains("CLASS III"));
        assert!(joined.contains("CLASS IV"));
        assert!(joined.contains("CLASS V"));
        assert!(joined.contains("CLASS VI"));
        assert!(joined.contains("CLASS VII"));
        assert!(joined.contains("§ 197"));
        assert!(joined.contains("§ 475"));
        assert!(joined.contains("§ 1221(a)(1)"));
        assert!(joined.contains("§ 1092(d)"));
        assert!(joined.contains("§ 6662"));
        assert!(joined.contains("§ 6721"));
        assert!(joined.contains("§ 6722"));
        assert!(joined.contains("15-year"));
        assert!(joined.contains("180-month"));
        assert!(joined.contains("§ 338(h)(10)"));
        assert!(joined.contains("§ 1031"));
        assert!(joined.contains("§ 351"));
        assert!(joined.contains("§ 721"));
    }
}
