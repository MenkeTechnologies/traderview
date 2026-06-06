//! IRC § 1248 — Gain from Certain Sales or Exchanges of Stock in
//! Certain Foreign Corporations (CFC Stock Sale Deemed-Dividend
//! Recharacterization).
//!
//! Pure-compute recharacterization rule. When a U.S. person who is
//! a 10 %+ shareholder of a controlled foreign corporation (CFC)
//! sells or exchanges the CFC stock, the gain recognized is
//! RECHARACTERIZED as a deemed DIVIDEND (ordinary income) to the
//! extent of the CFC's earnings and profits (E&P) attributable to
//! the stock and the period of U.S. ownership while the foreign
//! corporation was a CFC. Defeats the conversion of CFC ordinary
//! income into capital gains via stock sale.
//!
//! Statute (verbatim mapping):
//! - § 1248(a) — GENERAL RULE: if a U.S. person owns 10 percent or
//!   more of the total combined voting power of all classes of
//!   stock entitled to vote (within meaning of § 958(a) or § 958(b))
//!   at any time during the 5-year period ending on the date of
//!   the sale or exchange of stock in a foreign corporation, and
//!   the corporation was a CFC at any time during such period, the
//!   gain recognized on the sale shall be included in gross income
//!   as a dividend to the extent of the E&P of the foreign
//!   corporation attributable (under regulations) to such stock
//!   and to the period during which the seller held such stock
//!   while the corporation was a CFC.
//! - § 1248(b) — INDIVIDUAL SHAREHOLDER LIMITATION: in the case of
//!   an individual whose gain on the sale of the foreign
//!   corporation stock is a long-term capital gain (capital asset
//!   held more than 1 year), the tax attributable to the deemed
//!   dividend shall not exceed the sum of (1) a pro rata share of
//!   the tax that the foreign corporation would have paid as a
//!   domestic corporation on the portion of E&P attributable to the
//!   stock; plus (2) the tax that would have been paid on the gain
//!   treated as long-term capital gain.
//! - § 1248(c)(2) — LOWER-TIER CFC RULE: if the foreign corporation
//!   owns 50 percent or more of the voting power or value of
//!   another foreign corporation, the E&P of the lower-tier CFC
//!   attributable to the upper-tier CFC stock is also included in
//!   the deemed dividend.
//! - § 1248(d) — EXCLUSIONS FROM E&P: (1) E&P attributable to
//!   amounts which were INCLUDED in gross income under § 951
//!   (subpart F PTI) or § 951A (GILTI PTI); (2) E&P attributable to
//!   gain that would have been ECI; (3) E&P of foreign corporation
//!   attributable to period BEFORE January 1, 1963 (effectively the
//!   original 1962 enactment date — limited modern relevance).
//! - § 1248(e) — SALE OR EXCHANGE OF A U.S. CORPORATION FORMED OR
//!   AVAILED OF TO HOLD STOCK OF A FOREIGN CORPORATION: extends
//!   § 1248(a) to certain holding-corp dispositions.
//! - § 245A — DIVIDENDS RECEIVED DEDUCTION (POST-TCJA): 100 % DRD
//!   on the foreign-source portion of dividends received by U.S.
//!   C-corporate shareholders from specified 10 %-owned foreign
//!   corporations. In the vast majority of cases prevents domestic
//!   C corporations from being subject to § 1248 tax — the § 1248
//!   deemed dividend is fully offset by the § 245A DRD.
//! - § 958(a)/(b) — direct and indirect ownership attribution.
//!
//! Web research (verified 2026-06-03):
//! - Cornell LII § 1248 confirms statutory text.
//! - The Tax Adviser "Selling Partnerships That Own CFCs"
//!   confirms partnership-level § 1248 application.
//! - SF Tax Counsel "Section 1248 and 245A Strategies for Foreign
//!   E&P Gain" confirms § 245A practical elimination of § 1248 for
//!   C-corp sellers.
//! - IRC Section 1248 (Tax Notes) confirms 10 % voting threshold
//!   under § 958 and 5-year lookback.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const SECTION_1248_TEN_PERCENT_VOTING_THRESHOLD_BASIS_POINTS: u64 = 1_000;
pub const SECTION_1248_BASIS_POINT_DENOMINATOR: u64 = 10_000;
pub const SECTION_1248_FIVE_YEAR_LOOKBACK_YEARS: u32 = 5;
pub const SECTION_1248_LOWER_TIER_CFC_OWNERSHIP_THRESHOLD_BASIS_POINTS: u64 = 5_000;
pub const SECTION_1248_PRE_1963_CUTOFF_YEAR: u32 = 1963;
pub const SECTION_1248_ORIGINAL_ENACTMENT_YEAR: u32 = 1962;
pub const SECTION_245A_PARTICIPATION_EXEMPTION_TCJA_YEAR: u32 = 2017;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ShareholderType {
    IndividualUsShareholder,
    UsCorporateSellerEligibleForSection245aDrd,
    UsCorporateSellerNotEligibleForSection245a,
    PartnershipPassThroughUsShareholder,
    NonUsShareholder,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EpCategory {
    UntaxedSection1248Ep,
    PtiPreviouslyTaxedIncome,
    EciEffectivelyConnectedIncome,
    Pre1963Ep,
    LowerTierCfcEp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section1248Mode {
    NotApplicableNoCfcStockSold,
    NotApplicableShareholderUnderTenPercent,
    NotApplicableSharesNotHeldDuringCfcPeriod,
    NotApplicableUsCorporateSellerSection245aFullDrd,
    CompliantIndividualDeemedDividendWithSection1248bLimitation,
    CompliantDeemedDividendRecharacterizedFullEpExcess,
    CompliantLowerTierCfcEpIncluded,
    CompliantExclusionForPtiPreviouslyTaxed,
    CompliantExclusionForEciEffectivelyConnected,
    CompliantExclusionForPre1963Ep,
    ViolationGainReportedAsCapitalDespiteSection1248,
    ViolationLowerTierCfcEpOmittedFromDeemedDividend,
    ViolationIndividualClaimedSection245aDrdImproperly,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub shareholder_type: ShareholderType,
    pub voting_power_owned_basis_points: u64,
    pub voting_power_held_during_5_year_lookback: bool,
    pub corporation_was_cfc_during_holding_period: bool,
    pub gain_recognized_on_sale_dollars: u64,
    pub allocable_section_1248_ep_dollars: u64,
    pub allocable_lower_tier_cfc_ep_dollars: u64,
    pub ep_category: EpCategory,
    pub taxpayer_reported_full_gain_as_capital: bool,
    pub taxpayer_claimed_section_245a_drd: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section1248Mode,
    pub deemed_dividend_amount_dollars: u64,
    pub remaining_capital_gain_dollars: u64,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type Section1248Input = Input;
pub type Section1248Output = Output;
pub type Section1248Result = Output;

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "26 U.S.C. § 1248(a) — gain on CFC stock sale recharacterized as dividend to extent of allocable E&P; 10 %+ U.S. shareholder; 5-year lookback".to_string(),
        "26 U.S.C. § 1248(b) — individual shareholder LTCG tax limitation: pro rata domestic-corp tax + LTCG tax on residual".to_string(),
        "26 U.S.C. § 1248(c)(2) — lower-tier CFC E&P inclusion when upper CFC owns ≥ 50 % voting/value".to_string(),
        "26 U.S.C. § 1248(d)(1) — PTI (subpart F § 951 / GILTI § 951A) E&P EXCLUDED".to_string(),
        "26 U.S.C. § 1248(d)(2) — ECI E&P EXCLUDED".to_string(),
        "26 U.S.C. § 1248(d)(3) — pre-1963 E&P EXCLUDED (original 1962 enactment)".to_string(),
        "26 U.S.C. § 1248(e) — extension to U.S. holding corp formed/availed of to hold CFC stock".to_string(),
        "26 U.S.C. § 245A — TCJA 2017 100 % DRD for U.S. C-corp on foreign-source dividend from 10 %+-owned foreign corp; effectively eliminates § 1248 for corporate sellers".to_string(),
        "26 U.S.C. § 958(a)(direct) and § 958(b)(constructive) — ownership attribution for § 1248 threshold".to_string(),
        "5-year lookback under § 1248(a) — voting threshold tested at ANY TIME during 5 years ending on date of sale".to_string(),
    ];

    if input.gain_recognized_on_sale_dollars == 0 {
        return Output {
            mode: Section1248Mode::NotApplicableNoCfcStockSold,
            deemed_dividend_amount_dollars: 0,
            remaining_capital_gain_dollars: 0,
            statutory_basis: "§ 1248(a) — no gain on sale; rule inapplicable".to_string(),
            notes: "No CFC stock sale or no gain recognized; § 1248 inapplicable.".to_string(),
            citations,
        };
    }

    if input.shareholder_type == ShareholderType::NonUsShareholder {
        return Output {
            mode: Section1248Mode::NotApplicableShareholderUnderTenPercent,
            deemed_dividend_amount_dollars: 0,
            remaining_capital_gain_dollars: input.gain_recognized_on_sale_dollars,
            statutory_basis: "§ 1248(a) — non-US shareholder not subject".to_string(),
            notes: "Non-US shareholder; § 1248 applies only to U.S. persons.".to_string(),
            citations,
        };
    }

    if input.voting_power_owned_basis_points
        < SECTION_1248_TEN_PERCENT_VOTING_THRESHOLD_BASIS_POINTS
        && !input.voting_power_held_during_5_year_lookback
    {
        return Output {
            mode: Section1248Mode::NotApplicableShareholderUnderTenPercent,
            deemed_dividend_amount_dollars: 0,
            remaining_capital_gain_dollars: input.gain_recognized_on_sale_dollars,
            statutory_basis: format!(
                "§ 1248(a) — voting power {} basis points below 10 % threshold AND no 5-year lookback satisfaction",
                input.voting_power_owned_basis_points
            ),
            notes: format!(
                "Voting power = {} basis points; § 1248 requires 10 %+ (= 1000 basis points) at any time during 5-year lookback. § 1248 inapplicable.",
                input.voting_power_owned_basis_points
            ),
            citations,
        };
    }

    if !input.corporation_was_cfc_during_holding_period {
        return Output {
            mode: Section1248Mode::NotApplicableSharesNotHeldDuringCfcPeriod,
            deemed_dividend_amount_dollars: 0,
            remaining_capital_gain_dollars: input.gain_recognized_on_sale_dollars,
            statutory_basis: "§ 1248(a) — foreign corporation was never a CFC during holding period".to_string(),
            notes: "Foreign corporation was never a CFC during the seller's holding period; § 1248 recharacterization inapplicable.".to_string(),
            citations,
        };
    }

    match input.ep_category {
        EpCategory::PtiPreviouslyTaxedIncome => {
            return Output {
                mode: Section1248Mode::CompliantExclusionForPtiPreviouslyTaxed,
                deemed_dividend_amount_dollars: 0,
                remaining_capital_gain_dollars: input.gain_recognized_on_sale_dollars,
                statutory_basis: "§ 1248(d)(1) — PTI E&P excluded".to_string(),
                notes: "E&P consists of PTI (previously taxed under subpart F § 951 or GILTI § 951A); excluded from § 1248 deemed dividend.".to_string(),
                citations,
            };
        }
        EpCategory::EciEffectivelyConnectedIncome => {
            return Output {
                mode: Section1248Mode::CompliantExclusionForEciEffectivelyConnected,
                deemed_dividend_amount_dollars: 0,
                remaining_capital_gain_dollars: input.gain_recognized_on_sale_dollars,
                statutory_basis: "§ 1248(d)(2) — ECI E&P excluded".to_string(),
                notes: "E&P consists of effectively connected income; excluded from § 1248 deemed dividend.".to_string(),
                citations,
            };
        }
        EpCategory::Pre1963Ep => {
            return Output {
                mode: Section1248Mode::CompliantExclusionForPre1963Ep,
                deemed_dividend_amount_dollars: 0,
                remaining_capital_gain_dollars: input.gain_recognized_on_sale_dollars,
                statutory_basis: "§ 1248(d)(3) — pre-1963 E&P excluded".to_string(),
                notes: "E&P attributable to period before January 1, 1963 (original § 1248 enactment cutoff); excluded.".to_string(),
                citations,
            };
        }
        _ => {}
    }

    if input.shareholder_type == ShareholderType::UsCorporateSellerEligibleForSection245aDrd {
        return Output {
            mode: Section1248Mode::NotApplicableUsCorporateSellerSection245aFullDrd,
            deemed_dividend_amount_dollars: input.allocable_section_1248_ep_dollars.min(input.gain_recognized_on_sale_dollars),
            remaining_capital_gain_dollars: input
                .gain_recognized_on_sale_dollars
                .saturating_sub(input.allocable_section_1248_ep_dollars),
            statutory_basis: "§ 245A — 100 % DRD on § 1248 deemed dividend (TCJA participation exemption)".to_string(),
            notes: format!(
                "U.S. C-corporate seller eligible for § 245A 100 % DRD. § 1248 recharacterizes ${} as deemed dividend but § 245A DRD fully offsets the dividend inclusion. Residual capital gain = ${}.",
                input.allocable_section_1248_ep_dollars.min(input.gain_recognized_on_sale_dollars),
                input.gain_recognized_on_sale_dollars.saturating_sub(input.allocable_section_1248_ep_dollars)
            ),
            citations,
        };
    }

    if input.shareholder_type == ShareholderType::IndividualUsShareholder
        && input.taxpayer_claimed_section_245a_drd
    {
        return Output {
            mode: Section1248Mode::ViolationIndividualClaimedSection245aDrdImproperly,
            deemed_dividend_amount_dollars: input.allocable_section_1248_ep_dollars.min(input.gain_recognized_on_sale_dollars),
            remaining_capital_gain_dollars: input
                .gain_recognized_on_sale_dollars
                .saturating_sub(input.allocable_section_1248_ep_dollars),
            statutory_basis: "§ 245A available ONLY to U.S. C-corporate shareholders".to_string(),
            notes: "VIOLATION: individual taxpayer improperly claimed § 245A DRD on § 1248 deemed dividend. § 245A available only to U.S. C-corporate shareholders; individual taxpayer must include deemed dividend in gross income with § 1248(b) LTCG limitation available.".to_string(),
            citations,
        };
    }

    let deemed_dividend = input
        .allocable_section_1248_ep_dollars
        .saturating_add(input.allocable_lower_tier_cfc_ep_dollars)
        .min(input.gain_recognized_on_sale_dollars);
    let remaining_capital = input
        .gain_recognized_on_sale_dollars
        .saturating_sub(deemed_dividend);

    if input.ep_category == EpCategory::LowerTierCfcEp
        && input.allocable_lower_tier_cfc_ep_dollars == 0
    {
        return Output {
            mode: Section1248Mode::ViolationLowerTierCfcEpOmittedFromDeemedDividend,
            deemed_dividend_amount_dollars: input.allocable_section_1248_ep_dollars.min(input.gain_recognized_on_sale_dollars),
            remaining_capital_gain_dollars: input
                .gain_recognized_on_sale_dollars
                .saturating_sub(input.allocable_section_1248_ep_dollars),
            statutory_basis: "§ 1248(c)(2) — lower-tier CFC E&P inclusion missing".to_string(),
            notes: "VIOLATION § 1248(c)(2): upper-tier CFC owns ≥ 50 % of lower-tier CFC but taxpayer failed to include lower-tier CFC E&P in deemed dividend computation.".to_string(),
            citations,
        };
    }

    if input.taxpayer_reported_full_gain_as_capital
        && (input.allocable_section_1248_ep_dollars > 0
            || input.allocable_lower_tier_cfc_ep_dollars > 0)
    {
        return Output {
            mode: Section1248Mode::ViolationGainReportedAsCapitalDespiteSection1248,
            deemed_dividend_amount_dollars: deemed_dividend,
            remaining_capital_gain_dollars: remaining_capital,
            statutory_basis: "§ 1248(a) — gain must be recharacterized as dividend to extent of allocable E&P".to_string(),
            notes: format!(
                "VIOLATION § 1248(a): taxpayer reported full gain of ${} as capital gain despite CFC E&P attributable to stock. Required dividend recharacterization = ${}; remaining capital gain = ${}.",
                input.gain_recognized_on_sale_dollars, deemed_dividend, remaining_capital
            ),
            citations,
        };
    }

    if input.shareholder_type == ShareholderType::IndividualUsShareholder {
        return Output {
            mode: Section1248Mode::CompliantIndividualDeemedDividendWithSection1248bLimitation,
            deemed_dividend_amount_dollars: deemed_dividend,
            remaining_capital_gain_dollars: remaining_capital,
            statutory_basis: "§ 1248(a) deemed dividend + § 1248(b) individual LTCG tax limitation".to_string(),
            notes: format!(
                "COMPLIANT: individual shareholder recharacterizes ${} as deemed dividend (ordinary income); residual capital gain = ${}; § 1248(b) limits tax to pro rata domestic-corp tax + LTCG tax on residual.",
                deemed_dividend, remaining_capital
            ),
            citations,
        };
    }

    if input.ep_category == EpCategory::LowerTierCfcEp
        && input.allocable_lower_tier_cfc_ep_dollars > 0
    {
        return Output {
            mode: Section1248Mode::CompliantLowerTierCfcEpIncluded,
            deemed_dividend_amount_dollars: deemed_dividend,
            remaining_capital_gain_dollars: remaining_capital,
            statutory_basis: "§ 1248(c)(2) — lower-tier CFC E&P properly included".to_string(),
            notes: format!(
                "COMPLIANT § 1248(c)(2): lower-tier CFC E&P of ${} included in deemed dividend along with upper-tier E&P of ${}.",
                input.allocable_lower_tier_cfc_ep_dollars, input.allocable_section_1248_ep_dollars
            ),
            citations,
        };
    }

    Output {
        mode: Section1248Mode::CompliantDeemedDividendRecharacterizedFullEpExcess,
        deemed_dividend_amount_dollars: deemed_dividend,
        remaining_capital_gain_dollars: remaining_capital,
        statutory_basis: "§ 1248(a) — deemed dividend recharacterization at full E&P amount".to_string(),
        notes: format!(
            "COMPLIANT § 1248: deemed dividend = ${}; remaining capital gain = ${}. Total gain ${}.",
            deemed_dividend, remaining_capital, input.gain_recognized_on_sale_dollars
        ),
        citations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_individual_cfc_sale() -> Input {
        Input {
            shareholder_type: ShareholderType::IndividualUsShareholder,
            voting_power_owned_basis_points: 5_000,
            voting_power_held_during_5_year_lookback: true,
            corporation_was_cfc_during_holding_period: true,
            gain_recognized_on_sale_dollars: 10_000_000,
            allocable_section_1248_ep_dollars: 4_000_000,
            allocable_lower_tier_cfc_ep_dollars: 0,
            ep_category: EpCategory::UntaxedSection1248Ep,
            taxpayer_reported_full_gain_as_capital: false,
            taxpayer_claimed_section_245a_drd: false,
        }
    }

    #[test]
    fn no_gain_not_applicable() {
        let input = Input {
            gain_recognized_on_sale_dollars: 0,
            ..baseline_individual_cfc_sale()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1248Mode::NotApplicableNoCfcStockSold);
    }

    #[test]
    fn non_us_shareholder_not_applicable() {
        let input = Input {
            shareholder_type: ShareholderType::NonUsShareholder,
            ..baseline_individual_cfc_sale()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section1248Mode::NotApplicableShareholderUnderTenPercent
        );
    }

    #[test]
    fn under_10_percent_voting_not_applicable() {
        let input = Input {
            voting_power_owned_basis_points: 500,
            voting_power_held_during_5_year_lookback: false,
            ..baseline_individual_cfc_sale()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section1248Mode::NotApplicableShareholderUnderTenPercent
        );
    }

    #[test]
    fn at_exactly_10_percent_voting_triggers_recharacterization() {
        let input = Input {
            voting_power_owned_basis_points: 1_000,
            ..baseline_individual_cfc_sale()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section1248Mode::CompliantIndividualDeemedDividendWithSection1248bLimitation
        );
    }

    #[test]
    fn not_cfc_during_holding_period_not_applicable() {
        let input = Input {
            corporation_was_cfc_during_holding_period: false,
            ..baseline_individual_cfc_sale()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section1248Mode::NotApplicableSharesNotHeldDuringCfcPeriod
        );
    }

    #[test]
    fn individual_compliant_with_1248b_limitation() {
        let result = compute(&baseline_individual_cfc_sale());
        assert_eq!(
            result.mode,
            Section1248Mode::CompliantIndividualDeemedDividendWithSection1248bLimitation
        );
        assert_eq!(result.deemed_dividend_amount_dollars, 4_000_000);
        assert_eq!(result.remaining_capital_gain_dollars, 6_000_000);
    }

    #[test]
    fn pti_excluded_section_1248_d_1() {
        let input = Input {
            ep_category: EpCategory::PtiPreviouslyTaxedIncome,
            ..baseline_individual_cfc_sale()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section1248Mode::CompliantExclusionForPtiPreviouslyTaxed
        );
        assert_eq!(result.deemed_dividend_amount_dollars, 0);
    }

    #[test]
    fn eci_excluded_section_1248_d_2() {
        let input = Input {
            ep_category: EpCategory::EciEffectivelyConnectedIncome,
            ..baseline_individual_cfc_sale()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section1248Mode::CompliantExclusionForEciEffectivelyConnected
        );
    }

    #[test]
    fn pre_1963_excluded_section_1248_d_3() {
        let input = Input {
            ep_category: EpCategory::Pre1963Ep,
            ..baseline_individual_cfc_sale()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1248Mode::CompliantExclusionForPre1963Ep);
    }

    #[test]
    fn us_corporate_seller_section_245a_full_drd() {
        let input = Input {
            shareholder_type: ShareholderType::UsCorporateSellerEligibleForSection245aDrd,
            ..baseline_individual_cfc_sale()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section1248Mode::NotApplicableUsCorporateSellerSection245aFullDrd
        );
    }

    #[test]
    fn individual_claimed_245a_drd_violation() {
        let input = Input {
            shareholder_type: ShareholderType::IndividualUsShareholder,
            taxpayer_claimed_section_245a_drd: true,
            ..baseline_individual_cfc_sale()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section1248Mode::ViolationIndividualClaimedSection245aDrdImproperly
        );
    }

    #[test]
    fn lower_tier_cfc_ep_included_compliant() {
        let input = Input {
            shareholder_type: ShareholderType::UsCorporateSellerNotEligibleForSection245a,
            ep_category: EpCategory::LowerTierCfcEp,
            allocable_lower_tier_cfc_ep_dollars: 1_000_000,
            ..baseline_individual_cfc_sale()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section1248Mode::CompliantLowerTierCfcEpIncluded
        );
        assert_eq!(result.deemed_dividend_amount_dollars, 5_000_000);
    }

    #[test]
    fn lower_tier_cfc_ep_omitted_violation() {
        let input = Input {
            shareholder_type: ShareholderType::UsCorporateSellerNotEligibleForSection245a,
            ep_category: EpCategory::LowerTierCfcEp,
            allocable_lower_tier_cfc_ep_dollars: 0,
            ..baseline_individual_cfc_sale()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section1248Mode::ViolationLowerTierCfcEpOmittedFromDeemedDividend
        );
    }

    #[test]
    fn full_gain_reported_as_capital_violation() {
        let input = Input {
            shareholder_type: ShareholderType::UsCorporateSellerNotEligibleForSection245a,
            taxpayer_reported_full_gain_as_capital: true,
            ..baseline_individual_cfc_sale()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section1248Mode::ViolationGainReportedAsCapitalDespiteSection1248
        );
    }

    #[test]
    fn deemed_dividend_capped_at_gain() {
        let input = Input {
            shareholder_type: ShareholderType::UsCorporateSellerNotEligibleForSection245a,
            allocable_section_1248_ep_dollars: 20_000_000,
            gain_recognized_on_sale_dollars: 10_000_000,
            ..baseline_individual_cfc_sale()
        };
        let result = compute(&input);
        assert_eq!(result.deemed_dividend_amount_dollars, 10_000_000);
        assert_eq!(result.remaining_capital_gain_dollars, 0);
    }

    #[test]
    fn partnership_passthrough_individual_treatment() {
        let input = Input {
            shareholder_type: ShareholderType::PartnershipPassThroughUsShareholder,
            ..baseline_individual_cfc_sale()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section1248Mode::CompliantDeemedDividendRecharacterizedFullEpExcess
        );
    }

    #[test]
    fn voting_power_held_during_lookback_satisfies_threshold() {
        let input = Input {
            voting_power_owned_basis_points: 500,
            voting_power_held_during_5_year_lookback: true,
            ..baseline_individual_cfc_sale()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section1248Mode::CompliantIndividualDeemedDividendWithSection1248bLimitation
        );
    }

    #[test]
    fn citations_pin_section_1248_subsections_and_245a() {
        let result = compute(&baseline_individual_cfc_sale());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("§ 1248(a)"));
        assert!(joined.contains("§ 1248(b)"));
        assert!(joined.contains("§ 1248(c)(2)"));
        assert!(joined.contains("§ 1248(d)(1)"));
        assert!(joined.contains("§ 1248(d)(2)"));
        assert!(joined.contains("§ 1248(d)(3)"));
        assert!(joined.contains("§ 1248(e)"));
        assert!(joined.contains("§ 245A"));
        assert!(joined.contains("§ 958(a)"));
        assert!(joined.contains("§ 958(b)"));
        assert!(joined.contains("5-year lookback"));
        assert!(joined.contains("TCJA"));
    }

    #[test]
    fn constant_pin_thresholds_and_dates() {
        assert_eq!(
            SECTION_1248_TEN_PERCENT_VOTING_THRESHOLD_BASIS_POINTS,
            1_000
        );
        assert_eq!(SECTION_1248_BASIS_POINT_DENOMINATOR, 10_000);
        assert_eq!(SECTION_1248_FIVE_YEAR_LOOKBACK_YEARS, 5);
        assert_eq!(
            SECTION_1248_LOWER_TIER_CFC_OWNERSHIP_THRESHOLD_BASIS_POINTS,
            5_000
        );
        assert_eq!(SECTION_1248_PRE_1963_CUTOFF_YEAR, 1963);
        assert_eq!(SECTION_1248_ORIGINAL_ENACTMENT_YEAR, 1962);
        assert_eq!(SECTION_245A_PARTICIPATION_EXEMPTION_TCJA_YEAR, 2017);
    }

    #[test]
    fn saturating_overflow_defense_extreme_gain_and_ep() {
        let input = Input {
            shareholder_type: ShareholderType::UsCorporateSellerNotEligibleForSection245a,
            gain_recognized_on_sale_dollars: u64::MAX,
            allocable_section_1248_ep_dollars: u64::MAX,
            ..baseline_individual_cfc_sale()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section1248Mode::CompliantDeemedDividendRecharacterizedFullEpExcess
        );
    }
}
