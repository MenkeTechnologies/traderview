//! IRC § 1402 — Definitions for Self-Employment Income / Tax.
//!
//! Pure-compute classification check for whether an income item
//! is subject to Self-Employment Contributions Act (SECA) tax
//! under § 1402. Trader-critical because § 475(f) mark-to-market
//! trader gains are EXCLUDED from net earnings from self-
//! employment per long-standing IRS guidance, and the
//! § 1402(a)(13) limited-partner exclusion is heavily contested
//! by IRS for hedge fund / private equity active limited partners.
//!
//! Statute (verbatim mapping):
//! - § 1402(a) — NET EARNINGS FROM SELF-EMPLOYMENT: gross income
//!   derived by an individual from any trade or business carried on
//!   by such individual, less the deductions allowed by this
//!   subtitle which are attributable to such trade or business, plus
//!   the distributive share (whether or not distributed) of income
//!   or loss described in § 702(a)(8) from any trade or business
//!   carried on by a partnership of which he is a member.
//! - § 1402(a)(1) — RENTAL REAL ESTATE EXCLUSION: rentals from real
//!   estate and from personal property leased with the real estate
//!   (including such rentals paid in crop shares) and the deductions
//!   attributable thereto, unless such rentals are received in the
//!   course of a trade or business as a real estate dealer.
//! - § 1402(a)(2) — INTEREST AND DIVIDENDS EXCLUSION: dividends on
//!   any share of stock, and interest on any bond, debenture, note,
//!   certificate, or other evidence of indebtedness, unless such
//!   dividends and interest are received in the course of a trade
//!   or business as a dealer in stocks or securities.
//! - § 1402(a)(3)(A) — CAPITAL GAINS EXCLUSION: any gain or loss
//!   from the sale, exchange, or involuntary conversion of property
//!   if such property is a capital asset.
//! - § 1402(a)(3)(B) — § 1231 PROPERTY EXCLUSION: any gain or loss
//!   from the sale, exchange, or involuntary conversion of property
//!   which is not stock in trade or other property of a kind which
//!   would properly be includible in inventory or held primarily for
//!   sale to customers in the ordinary course of the trade or
//!   business.
//! - § 1402(a)(13) — LIMITED PARTNER EXCLUSION: the distributive
//!   share of any item of income or loss of a limited partner, as
//!   such, other than guaranteed payments described in § 707(c) to
//!   that partner for services actually rendered to or on behalf of
//!   the partnership to the extent that those payments are
//!   established to be in the nature of remuneration for those
//!   services.
//! - § 1402(b) — DEFINITION OF "SELF-EMPLOYMENT INCOME": net
//!   earnings from self-employment derived by an individual during
//!   any taxable year, with thresholds and limits.
//! - § 1401 — RATES IMPOSED ON SE INCOME: § 1401(a) OASDI 12.4 %
//!   up to wage base; § 1401(b) Medicare 2.9 % uncapped; § 1401(b)(2)
//!   Additional Medicare Tax 0.9 % on SE income > $200,000 single /
//!   $250,000 MFJ / $125,000 MFS.
//!
//! Web research (verified 2026-06-03):
//! - Cornell LII § 1402 confirms statutory text.
//! - 2025 OASDI wage base = $176,100 (announced by SSA Oct 2024).
//!   2026 wage base subject to SSA annual inflation announcement
//!   typically in October of preceding year.
//! - IRS Topic 429 (Traders in securities): § 475(f) MTM trader
//!   gains NOT self-employment income under long-standing IRS
//!   guidance; reported on Form 4797 as ordinary gains/losses but
//!   NOT subject to SE tax.
//! - Green Trader Tax "Hedge Fund Medicare Tax Gap" — active
//!   limited partners face SE tax exposure post-Soroban Capital
//!   Partners (Tax Court 2023) + Denham Capital (Tax Court 2024)
//!   + Sirius Solutions (5th Cir.) cases.
//! - White & Case "IRS continues winning SECA tax against limited
//!   partners in private equity and hedge funds" — IRS has
//!   successfully asserted SE tax against active limited partners.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const SECTION_1402_OASDI_RATE_BASIS_POINTS: u64 = 1_240;
pub const SECTION_1402_MEDICARE_RATE_BASIS_POINTS: u64 = 290;
pub const SECTION_1402_ADDITIONAL_MEDICARE_RATE_BASIS_POINTS: u64 = 90;
pub const SECTION_1402_BASIS_POINT_DENOMINATOR: u64 = 10_000;
pub const SECTION_1402_2025_OASDI_WAGE_BASE_DOLLARS: u64 = 176_100;
pub const SECTION_1402_ADDITIONAL_MEDICARE_THRESHOLD_SINGLE_DOLLARS: u64 = 200_000;
pub const SECTION_1402_ADDITIONAL_MEDICARE_THRESHOLD_MFJ_DOLLARS: u64 = 250_000;
pub const SECTION_1402_ADDITIONAL_MEDICARE_THRESHOLD_MFS_DOLLARS: u64 = 125_000;
pub const SECTION_1402_NET_EARNINGS_MULTIPLIER_BASIS_POINTS: u64 = 9_235;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TaxpayerActivityType {
    Section475fMtmSoleProprietorTrader,
    LimitedPartnerInInvestmentPartnership,
    ActiveLimitedPartnerInInvestmentPartnership,
    GeneralPartnerOrManagingMember,
    SoleProprietorActiveBusiness,
    RentalRealEstateNonDealer,
    RealEstateDealerInTrade,
    DealerInSecurities,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IncomeItemType {
    TradingGainsLossesSection475fMtm,
    LimitedPartnerDistributiveShareNonGuaranteed,
    GeneralPartnerDistributiveShare,
    GuaranteedPaymentForServicesSection707c,
    CapitalGainSale,
    Section1231GainSale,
    RentalRealEstateIncome,
    PortfolioInterestDividends,
    OrdinaryBusinessIncome,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FilingStatus {
    Single,
    MarriedFilingJointly,
    MarriedFilingSeparately,
    HeadOfHousehold,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section1402Mode {
    NotApplicableNoSeIncome,
    CompliantSection475fMtmTraderGainsExcludedFromSeIncome,
    CompliantLimitedPartnerDistributiveShareExcludedSection1402a13,
    CompliantCapitalAssetGainExcludedSection1402a3A,
    CompliantSection1231GainExcludedSection1402a3B,
    CompliantRentalRealEstateExcludedSection1402a1,
    CompliantPortfolioInterestDividendsExcludedSection1402a2,
    CompliantGuaranteedPaymentForServicesIncludedSection1402a13,
    CompliantSeTaxComputedOnNetEarningsFromSelfEmployment,
    ViolationActiveLimitedPartnerExclusionImproperlyClaimed,
    ViolationSelfEmploymentIncomeUnderreported,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub activity_type: TaxpayerActivityType,
    pub income_item_type: IncomeItemType,
    pub gross_item_amount_dollars: u64,
    pub attributable_deductions_dollars: u64,
    pub filing_status: FilingStatus,
    pub other_wages_subject_to_oasdi_dollars: u64,
    pub claimed_se_income_exclusion: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section1402Mode,
    pub se_income_subject_to_seca_dollars: u64,
    pub oasdi_portion_dollars: u64,
    pub medicare_portion_dollars: u64,
    pub additional_medicare_portion_dollars: u64,
    pub total_seca_tax_dollars: u64,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type Section1402Input = Input;
pub type Section1402Output = Output;
pub type Section1402Result = Output;

fn apply_rate(amount: u64, rate_bp: u64) -> u64 {
    (amount as u128)
        .saturating_mul(rate_bp as u128)
        .checked_div(SECTION_1402_BASIS_POINT_DENOMINATOR as u128)
        .unwrap_or(0) as u64
}

fn additional_medicare_threshold_for(status: FilingStatus) -> u64 {
    match status {
        FilingStatus::Single | FilingStatus::HeadOfHousehold => {
            SECTION_1402_ADDITIONAL_MEDICARE_THRESHOLD_SINGLE_DOLLARS
        }
        FilingStatus::MarriedFilingJointly => {
            SECTION_1402_ADDITIONAL_MEDICARE_THRESHOLD_MFJ_DOLLARS
        }
        FilingStatus::MarriedFilingSeparately => {
            SECTION_1402_ADDITIONAL_MEDICARE_THRESHOLD_MFS_DOLLARS
        }
    }
}

fn compute_seca_tax(net_earnings: u64, other_wages: u64, status: FilingStatus) -> (u64, u64, u64, u64) {
    let net_earnings_after_se_adjustment = apply_rate(net_earnings, SECTION_1402_NET_EARNINGS_MULTIPLIER_BASIS_POINTS);
    let oasdi_base = SECTION_1402_2025_OASDI_WAGE_BASE_DOLLARS
        .saturating_sub(other_wages)
        .min(net_earnings_after_se_adjustment);
    let oasdi = apply_rate(oasdi_base, SECTION_1402_OASDI_RATE_BASIS_POINTS);
    let medicare = apply_rate(net_earnings_after_se_adjustment, SECTION_1402_MEDICARE_RATE_BASIS_POINTS);
    let threshold = additional_medicare_threshold_for(status);
    let amt_base = net_earnings_after_se_adjustment.saturating_sub(threshold);
    let additional_medicare = apply_rate(amt_base, SECTION_1402_ADDITIONAL_MEDICARE_RATE_BASIS_POINTS);
    let total = oasdi.saturating_add(medicare).saturating_add(additional_medicare);
    (oasdi, medicare, additional_medicare, total)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "26 U.S.C. § 1402(a) — net earnings from self-employment = gross income from trade or business minus attributable deductions, plus § 702(a)(8) distributive share".to_string(),
        "26 U.S.C. § 1402(a)(1) — rental real estate excluded unless real estate dealer".to_string(),
        "26 U.S.C. § 1402(a)(2) — interest and dividends excluded unless securities dealer".to_string(),
        "26 U.S.C. § 1402(a)(3)(A) — capital asset gain or loss excluded".to_string(),
        "26 U.S.C. § 1402(a)(3)(B) — § 1231 property gain or loss excluded".to_string(),
        "26 U.S.C. § 1402(a)(13) — limited partner distributive share excluded except § 707(c) guaranteed payments for services".to_string(),
        "26 U.S.C. § 1402(b) — self-employment income definition + thresholds".to_string(),
        "26 U.S.C. § 1401(a) — OASDI 12.4 % up to wage base".to_string(),
        "26 U.S.C. § 1401(b)(1) — Medicare 2.9 % uncapped".to_string(),
        "26 U.S.C. § 1401(b)(2) — Additional Medicare Tax 0.9 % on SE income > $200K single / $250K MFJ / $125K MFS".to_string(),
        "IRS Topic 429 (Traders in securities) — § 475(f) MTM trader gains NOT self-employment income; reported on Form 4797 as ordinary".to_string(),
        "Soroban Capital Partners (T.C. 2023) + Denham Capital (T.C. 2024) + Sirius Solutions (5th Cir.) — active limited partner § 1402(a)(13) exclusion contested".to_string(),
        "SSA 2025 OASDI wage base = $176,100; 2026 wage base subject to SSA annual inflation announcement".to_string(),
        "Net earnings multiplier 0.9235 = (1 − 0.0765) reflects employer-equivalent SE tax deduction".to_string(),
    ];

    if matches!(input.income_item_type, IncomeItemType::CapitalGainSale) {
        return Output {
            mode: Section1402Mode::CompliantCapitalAssetGainExcludedSection1402a3A,
            se_income_subject_to_seca_dollars: 0,
            oasdi_portion_dollars: 0,
            medicare_portion_dollars: 0,
            additional_medicare_portion_dollars: 0,
            total_seca_tax_dollars: 0,
            statutory_basis: "§ 1402(a)(3)(A) — capital asset gain/loss excluded from SE income".to_string(),
            notes: format!(
                "Capital gain of ${} excluded from SE income per § 1402(a)(3)(A).",
                input.gross_item_amount_dollars
            ),
            citations,
        };
    }

    if matches!(input.income_item_type, IncomeItemType::Section1231GainSale) {
        return Output {
            mode: Section1402Mode::CompliantSection1231GainExcludedSection1402a3B,
            se_income_subject_to_seca_dollars: 0,
            oasdi_portion_dollars: 0,
            medicare_portion_dollars: 0,
            additional_medicare_portion_dollars: 0,
            total_seca_tax_dollars: 0,
            statutory_basis: "§ 1402(a)(3)(B) — § 1231 property gain/loss excluded from SE income".to_string(),
            notes: format!(
                "§ 1231 property gain of ${} excluded from SE income per § 1402(a)(3)(B).",
                input.gross_item_amount_dollars
            ),
            citations,
        };
    }

    if matches!(input.income_item_type, IncomeItemType::RentalRealEstateIncome)
        && input.activity_type != TaxpayerActivityType::RealEstateDealerInTrade
    {
        return Output {
            mode: Section1402Mode::CompliantRentalRealEstateExcludedSection1402a1,
            se_income_subject_to_seca_dollars: 0,
            oasdi_portion_dollars: 0,
            medicare_portion_dollars: 0,
            additional_medicare_portion_dollars: 0,
            total_seca_tax_dollars: 0,
            statutory_basis: "§ 1402(a)(1) — rental real estate excluded unless real estate dealer".to_string(),
            notes: format!(
                "Rental real estate income of ${} excluded from SE income per § 1402(a)(1) (taxpayer is not a real estate dealer).",
                input.gross_item_amount_dollars
            ),
            citations,
        };
    }

    if matches!(input.income_item_type, IncomeItemType::PortfolioInterestDividends)
        && input.activity_type != TaxpayerActivityType::DealerInSecurities
    {
        return Output {
            mode: Section1402Mode::CompliantPortfolioInterestDividendsExcludedSection1402a2,
            se_income_subject_to_seca_dollars: 0,
            oasdi_portion_dollars: 0,
            medicare_portion_dollars: 0,
            additional_medicare_portion_dollars: 0,
            total_seca_tax_dollars: 0,
            statutory_basis: "§ 1402(a)(2) — interest and dividends excluded unless securities dealer".to_string(),
            notes: format!(
                "Portfolio interest/dividends of ${} excluded from SE income per § 1402(a)(2) (taxpayer is not a securities dealer).",
                input.gross_item_amount_dollars
            ),
            citations,
        };
    }

    if matches!(
        input.income_item_type,
        IncomeItemType::TradingGainsLossesSection475fMtm
    ) && matches!(
        input.activity_type,
        TaxpayerActivityType::Section475fMtmSoleProprietorTrader
    ) {
        return Output {
            mode: Section1402Mode::CompliantSection475fMtmTraderGainsExcludedFromSeIncome,
            se_income_subject_to_seca_dollars: 0,
            oasdi_portion_dollars: 0,
            medicare_portion_dollars: 0,
            additional_medicare_portion_dollars: 0,
            total_seca_tax_dollars: 0,
            statutory_basis: "§ 475(f) MTM trader gains not SE income per IRS Topic 429 long-standing guidance".to_string(),
            notes: format!(
                "§ 475(f) MTM sole-proprietor trader gains of ${} reported on Form 4797 as ordinary income but EXCLUDED from SE income.",
                input.gross_item_amount_dollars
            ),
            citations,
        };
    }

    if matches!(
        input.income_item_type,
        IncomeItemType::LimitedPartnerDistributiveShareNonGuaranteed
    ) && matches!(
        input.activity_type,
        TaxpayerActivityType::LimitedPartnerInInvestmentPartnership
    ) {
        return Output {
            mode: Section1402Mode::CompliantLimitedPartnerDistributiveShareExcludedSection1402a13,
            se_income_subject_to_seca_dollars: 0,
            oasdi_portion_dollars: 0,
            medicare_portion_dollars: 0,
            additional_medicare_portion_dollars: 0,
            total_seca_tax_dollars: 0,
            statutory_basis: "§ 1402(a)(13) — true limited partner distributive share excluded".to_string(),
            notes: format!(
                "True limited partner distributive share of ${} excluded from SE income per § 1402(a)(13). Watch Soroban / Denham / Sirius case law for ACTIVE limited partner risk.",
                input.gross_item_amount_dollars
            ),
            citations,
        };
    }

    if matches!(
        input.income_item_type,
        IncomeItemType::LimitedPartnerDistributiveShareNonGuaranteed
    ) && matches!(
        input.activity_type,
        TaxpayerActivityType::ActiveLimitedPartnerInInvestmentPartnership
    ) && input.claimed_se_income_exclusion
    {
        let net_earnings = input.gross_item_amount_dollars.saturating_sub(input.attributable_deductions_dollars);
        let (oasdi, medicare, additional_medicare, total) =
            compute_seca_tax(net_earnings, input.other_wages_subject_to_oasdi_dollars, input.filing_status);
        return Output {
            mode: Section1402Mode::ViolationActiveLimitedPartnerExclusionImproperlyClaimed,
            se_income_subject_to_seca_dollars: net_earnings,
            oasdi_portion_dollars: oasdi,
            medicare_portion_dollars: medicare,
            additional_medicare_portion_dollars: additional_medicare,
            total_seca_tax_dollars: total,
            statutory_basis: "§ 1402(a)(13) — ACTIVE limited partner not entitled to exclusion per Soroban / Denham / Sirius".to_string(),
            notes: format!(
                "VIOLATION: taxpayer is ACTIVE limited partner (services rendered to partnership); IRS contests § 1402(a)(13) exclusion. Required SE tax on net earnings of ${} = ${} total (OASDI ${} + Medicare ${} + Additional Medicare ${}).",
                net_earnings, total, oasdi, medicare, additional_medicare
            ),
            citations,
        };
    }

    if matches!(
        input.income_item_type,
        IncomeItemType::GuaranteedPaymentForServicesSection707c
    ) || matches!(
        input.income_item_type,
        IncomeItemType::GeneralPartnerDistributiveShare | IncomeItemType::OrdinaryBusinessIncome
    ) || (matches!(input.activity_type, TaxpayerActivityType::DealerInSecurities)
        && matches!(input.income_item_type, IncomeItemType::PortfolioInterestDividends))
        || (matches!(input.activity_type, TaxpayerActivityType::RealEstateDealerInTrade)
            && matches!(input.income_item_type, IncomeItemType::RentalRealEstateIncome))
    {
        let net_earnings = input.gross_item_amount_dollars.saturating_sub(input.attributable_deductions_dollars);
        if net_earnings == 0 {
            return Output {
                mode: Section1402Mode::NotApplicableNoSeIncome,
                se_income_subject_to_seca_dollars: 0,
                oasdi_portion_dollars: 0,
                medicare_portion_dollars: 0,
                additional_medicare_portion_dollars: 0,
                total_seca_tax_dollars: 0,
                statutory_basis: "§ 1402(a) — net earnings zero or negative".to_string(),
                notes: "Net earnings from self-employment is zero or negative; no SECA tax.".to_string(),
                citations,
            };
        }
        let (oasdi, medicare, additional_medicare, total) =
            compute_seca_tax(net_earnings, input.other_wages_subject_to_oasdi_dollars, input.filing_status);
        return Output {
            mode: Section1402Mode::CompliantSeTaxComputedOnNetEarningsFromSelfEmployment,
            se_income_subject_to_seca_dollars: net_earnings,
            oasdi_portion_dollars: oasdi,
            medicare_portion_dollars: medicare,
            additional_medicare_portion_dollars: additional_medicare,
            total_seca_tax_dollars: total,
            statutory_basis: format!(
                "§ 1401(a) OASDI {} bp + § 1401(b)(1) Medicare {} bp + § 1401(b)(2) Additional Medicare {} bp; income type {:?}",
                SECTION_1402_OASDI_RATE_BASIS_POINTS,
                SECTION_1402_MEDICARE_RATE_BASIS_POINTS,
                SECTION_1402_ADDITIONAL_MEDICARE_RATE_BASIS_POINTS,
                input.income_item_type
            ),
            notes: format!(
                "COMPLIANT SECA tax computed on net earnings ${}: OASDI ${} (12.4 % × min(wage base, net) net of other wages); Medicare ${} (2.9 % uncapped); Additional Medicare ${} (0.9 % on amount > threshold); total ${}.",
                net_earnings, oasdi, medicare, additional_medicare, total
            ),
            citations,
        };
    }

    Output {
        mode: Section1402Mode::NotApplicableNoSeIncome,
        se_income_subject_to_seca_dollars: 0,
        oasdi_portion_dollars: 0,
        medicare_portion_dollars: 0,
        additional_medicare_portion_dollars: 0,
        total_seca_tax_dollars: 0,
        statutory_basis: "§ 1402(a) — combination not producing SE income".to_string(),
        notes: format!(
            "Activity {:?} + income type {:?} combination not producing SE income.",
            input.activity_type, input.income_item_type
        ),
        citations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_section_475f_trader() -> Input {
        Input {
            activity_type: TaxpayerActivityType::Section475fMtmSoleProprietorTrader,
            income_item_type: IncomeItemType::TradingGainsLossesSection475fMtm,
            gross_item_amount_dollars: 5_000_000,
            attributable_deductions_dollars: 0,
            filing_status: FilingStatus::Single,
            other_wages_subject_to_oasdi_dollars: 0,
            claimed_se_income_exclusion: true,
        }
    }

    #[test]
    fn section_475f_trader_gains_excluded_from_se_income() {
        let result = compute(&baseline_section_475f_trader());
        assert_eq!(result.mode, Section1402Mode::CompliantSection475fMtmTraderGainsExcludedFromSeIncome);
        assert_eq!(result.total_seca_tax_dollars, 0);
    }

    #[test]
    fn capital_gain_excluded_section_1402_a_3_a() {
        let input = Input {
            activity_type: TaxpayerActivityType::SoleProprietorActiveBusiness,
            income_item_type: IncomeItemType::CapitalGainSale,
            ..baseline_section_475f_trader()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1402Mode::CompliantCapitalAssetGainExcludedSection1402a3A);
    }

    #[test]
    fn section_1231_gain_excluded_section_1402_a_3_b() {
        let input = Input {
            activity_type: TaxpayerActivityType::SoleProprietorActiveBusiness,
            income_item_type: IncomeItemType::Section1231GainSale,
            ..baseline_section_475f_trader()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1402Mode::CompliantSection1231GainExcludedSection1402a3B);
    }

    #[test]
    fn rental_real_estate_non_dealer_excluded() {
        let input = Input {
            activity_type: TaxpayerActivityType::RentalRealEstateNonDealer,
            income_item_type: IncomeItemType::RentalRealEstateIncome,
            ..baseline_section_475f_trader()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1402Mode::CompliantRentalRealEstateExcludedSection1402a1);
    }

    #[test]
    fn real_estate_dealer_rental_income_subject_to_seca() {
        let input = Input {
            activity_type: TaxpayerActivityType::RealEstateDealerInTrade,
            income_item_type: IncomeItemType::RentalRealEstateIncome,
            gross_item_amount_dollars: 100_000,
            ..baseline_section_475f_trader()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1402Mode::CompliantSeTaxComputedOnNetEarningsFromSelfEmployment);
        assert!(result.total_seca_tax_dollars > 0);
    }

    #[test]
    fn portfolio_interest_dividends_excluded_non_dealer() {
        let input = Input {
            activity_type: TaxpayerActivityType::SoleProprietorActiveBusiness,
            income_item_type: IncomeItemType::PortfolioInterestDividends,
            ..baseline_section_475f_trader()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1402Mode::CompliantPortfolioInterestDividendsExcludedSection1402a2);
    }

    #[test]
    fn securities_dealer_dividends_subject_to_seca() {
        let input = Input {
            activity_type: TaxpayerActivityType::DealerInSecurities,
            income_item_type: IncomeItemType::PortfolioInterestDividends,
            gross_item_amount_dollars: 200_000,
            ..baseline_section_475f_trader()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1402Mode::CompliantSeTaxComputedOnNetEarningsFromSelfEmployment);
    }

    #[test]
    fn true_limited_partner_distributive_share_excluded() {
        let input = Input {
            activity_type: TaxpayerActivityType::LimitedPartnerInInvestmentPartnership,
            income_item_type: IncomeItemType::LimitedPartnerDistributiveShareNonGuaranteed,
            ..baseline_section_475f_trader()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1402Mode::CompliantLimitedPartnerDistributiveShareExcludedSection1402a13);
    }

    #[test]
    fn active_limited_partner_exclusion_improperly_claimed_violation() {
        let input = Input {
            activity_type: TaxpayerActivityType::ActiveLimitedPartnerInInvestmentPartnership,
            income_item_type: IncomeItemType::LimitedPartnerDistributiveShareNonGuaranteed,
            gross_item_amount_dollars: 1_000_000,
            claimed_se_income_exclusion: true,
            ..baseline_section_475f_trader()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1402Mode::ViolationActiveLimitedPartnerExclusionImproperlyClaimed);
        assert!(result.total_seca_tax_dollars > 0);
        assert!(result.notes.contains("Soroban / Denham / Sirius") || result.statutory_basis.contains("Soroban"));
    }

    #[test]
    fn general_partner_distributive_share_subject_to_seca() {
        let input = Input {
            activity_type: TaxpayerActivityType::GeneralPartnerOrManagingMember,
            income_item_type: IncomeItemType::GeneralPartnerDistributiveShare,
            gross_item_amount_dollars: 300_000,
            ..baseline_section_475f_trader()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1402Mode::CompliantSeTaxComputedOnNetEarningsFromSelfEmployment);
    }

    #[test]
    fn guaranteed_payment_for_services_subject_to_seca() {
        let input = Input {
            activity_type: TaxpayerActivityType::LimitedPartnerInInvestmentPartnership,
            income_item_type: IncomeItemType::GuaranteedPaymentForServicesSection707c,
            gross_item_amount_dollars: 250_000,
            ..baseline_section_475f_trader()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1402Mode::CompliantSeTaxComputedOnNetEarningsFromSelfEmployment);
    }

    #[test]
    fn sole_proprietor_ordinary_business_income_subject_to_seca() {
        let input = Input {
            activity_type: TaxpayerActivityType::SoleProprietorActiveBusiness,
            income_item_type: IncomeItemType::OrdinaryBusinessIncome,
            gross_item_amount_dollars: 100_000,
            ..baseline_section_475f_trader()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1402Mode::CompliantSeTaxComputedOnNetEarningsFromSelfEmployment);
        let net_earnings_after_se = apply_rate(100_000, SECTION_1402_NET_EARNINGS_MULTIPLIER_BASIS_POINTS);
        let expected_oasdi = apply_rate(net_earnings_after_se, SECTION_1402_OASDI_RATE_BASIS_POINTS);
        assert_eq!(result.oasdi_portion_dollars, expected_oasdi);
    }

    #[test]
    fn oasdi_wage_base_cap_at_176100() {
        let input = Input {
            activity_type: TaxpayerActivityType::SoleProprietorActiveBusiness,
            income_item_type: IncomeItemType::OrdinaryBusinessIncome,
            gross_item_amount_dollars: 500_000,
            other_wages_subject_to_oasdi_dollars: 0,
            ..baseline_section_475f_trader()
        };
        let result = compute(&input);
        let max_oasdi = apply_rate(SECTION_1402_2025_OASDI_WAGE_BASE_DOLLARS, SECTION_1402_OASDI_RATE_BASIS_POINTS);
        assert_eq!(result.oasdi_portion_dollars, max_oasdi);
    }

    #[test]
    fn other_wages_reduce_oasdi_base() {
        let input = Input {
            activity_type: TaxpayerActivityType::SoleProprietorActiveBusiness,
            income_item_type: IncomeItemType::OrdinaryBusinessIncome,
            gross_item_amount_dollars: 500_000,
            other_wages_subject_to_oasdi_dollars: SECTION_1402_2025_OASDI_WAGE_BASE_DOLLARS,
            ..baseline_section_475f_trader()
        };
        let result = compute(&input);
        assert_eq!(result.oasdi_portion_dollars, 0);
    }

    #[test]
    fn additional_medicare_tax_kicks_in_above_threshold() {
        let input = Input {
            activity_type: TaxpayerActivityType::SoleProprietorActiveBusiness,
            income_item_type: IncomeItemType::OrdinaryBusinessIncome,
            gross_item_amount_dollars: 1_000_000,
            filing_status: FilingStatus::Single,
            ..baseline_section_475f_trader()
        };
        let result = compute(&input);
        assert!(result.additional_medicare_portion_dollars > 0);
    }

    #[test]
    fn additional_medicare_threshold_mfj_higher() {
        let input_single = Input {
            activity_type: TaxpayerActivityType::SoleProprietorActiveBusiness,
            income_item_type: IncomeItemType::OrdinaryBusinessIncome,
            gross_item_amount_dollars: 230_000,
            filing_status: FilingStatus::Single,
            ..baseline_section_475f_trader()
        };
        let input_mfj = Input {
            filing_status: FilingStatus::MarriedFilingJointly,
            ..input_single.clone()
        };
        let result_single = compute(&input_single);
        let result_mfj = compute(&input_mfj);
        assert!(result_single.additional_medicare_portion_dollars > result_mfj.additional_medicare_portion_dollars);
    }

    #[test]
    fn zero_net_earnings_not_applicable() {
        let input = Input {
            activity_type: TaxpayerActivityType::SoleProprietorActiveBusiness,
            income_item_type: IncomeItemType::OrdinaryBusinessIncome,
            gross_item_amount_dollars: 100_000,
            attributable_deductions_dollars: 100_000,
            ..baseline_section_475f_trader()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1402Mode::NotApplicableNoSeIncome);
    }

    #[test]
    fn citations_pin_section_1402_subsections_and_case_law() {
        let result = compute(&baseline_section_475f_trader());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("§ 1402(a)"));
        assert!(joined.contains("§ 1402(a)(1)"));
        assert!(joined.contains("§ 1402(a)(2)"));
        assert!(joined.contains("§ 1402(a)(3)(A)"));
        assert!(joined.contains("§ 1402(a)(3)(B)"));
        assert!(joined.contains("§ 1402(a)(13)"));
        assert!(joined.contains("§ 1402(b)"));
        assert!(joined.contains("§ 1401(a)"));
        assert!(joined.contains("§ 1401(b)(1)"));
        assert!(joined.contains("§ 1401(b)(2)"));
        assert!(joined.contains("IRS Topic 429"));
        assert!(joined.contains("Soroban"));
        assert!(joined.contains("Denham"));
        assert!(joined.contains("Sirius"));
        assert!(joined.contains("$176,100"));
    }

    #[test]
    fn constant_pin_rates_and_thresholds() {
        assert_eq!(SECTION_1402_OASDI_RATE_BASIS_POINTS, 1_240);
        assert_eq!(SECTION_1402_MEDICARE_RATE_BASIS_POINTS, 290);
        assert_eq!(SECTION_1402_ADDITIONAL_MEDICARE_RATE_BASIS_POINTS, 90);
        assert_eq!(SECTION_1402_2025_OASDI_WAGE_BASE_DOLLARS, 176_100);
        assert_eq!(SECTION_1402_ADDITIONAL_MEDICARE_THRESHOLD_SINGLE_DOLLARS, 200_000);
        assert_eq!(SECTION_1402_ADDITIONAL_MEDICARE_THRESHOLD_MFJ_DOLLARS, 250_000);
        assert_eq!(SECTION_1402_ADDITIONAL_MEDICARE_THRESHOLD_MFS_DOLLARS, 125_000);
        assert_eq!(SECTION_1402_NET_EARNINGS_MULTIPLIER_BASIS_POINTS, 9_235);
    }

    #[test]
    fn saturating_overflow_defense_extreme_gross_income() {
        let input = Input {
            activity_type: TaxpayerActivityType::SoleProprietorActiveBusiness,
            income_item_type: IncomeItemType::OrdinaryBusinessIncome,
            gross_item_amount_dollars: u64::MAX,
            ..baseline_section_475f_trader()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1402Mode::CompliantSeTaxComputedOnNetEarningsFromSelfEmployment);
    }
}
