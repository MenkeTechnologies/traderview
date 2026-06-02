//! IRC § 4944 — Taxes on investments which jeopardize
//! charitable purpose of private foundations. PF
//! managers must exercise ORDINARY BUSINESS CARE AND
//! PRUDENCE in providing for long-term and short-term
//! financial needs of the foundation to carry out exempt
//! purposes; investments that fail this standard expose
//! PF and managers to two-tier excise tax. Direct PF
//! Chapter 42 companion to section_4940 (PF NII excise —
//! iter 470), section_4941 (PF self-dealing — iter 468),
//! section_4942 (PF minimum distribution — iter 472),
//! section_4943 (PF excess business holdings — iter 474),
//! section_4958 (intermediate sanctions for public
//! charities — iter 466), section_4960 (ATEO executive
//! comp 21% — iter 464). Originally enacted by Tax Reform
//! Act of 1969, Pub. L. 91-172.
//!
//! Four-tier excise tax structure:
//! - § 4944(a)(1) TIER 1 PF: 10% of amount of jeopardizing
//!   investment for each year or partial year in taxable
//!   period
//! - § 4944(a)(2) TIER 1 MANAGER (knowing willful
//!   participant without reasonable cause): 10% of amount
//!   of jeopardizing investment, capped at $10,000 per
//!   investment per § 4944(d)(2)
//! - § 4944(b)(1) TIER 2 PF (not removed from jeopardy
//!   within taxable period): 25% of amount of jeopardizing
//!   investment
//! - § 4944(b)(2) TIER 2 MANAGER (refuses to agree to
//!   removal within correction period): 5% of amount of
//!   jeopardizing investment, capped at $20,000 per
//!   investment per § 4944(d)(2)
//!
//! Jeopardizing investment standard — ORDINARY BUSINESS
//! CARE AND PRUDENCE: An investment is jeopardizing if PF
//! managers, in making the investment, have FAILED TO
//! EXERCISE the ordinary business care and prudence that
//! a person in like position would exercise under the
//! facts and circumstances prevailing at the time of
//! making the investment, in providing for the LONG-TERM
//! AND SHORT-TERM FINANCIAL NEEDS of the foundation to
//! carry out its exempt purposes. Determined at the TIME
//! OF INVESTMENT, NOT in hindsight.
//!
//! Categories typically scrutinized under § 4944 per
//! 26 C.F.R. § 53.4944-1(a)(2):
//! 1. TRADING ON MARGIN (margin securities loans)
//! 2. SHORT SALES
//! 3. PUT/CALL/STRADDLE/SPREAD OPTION DERIVATIVES
//! 4. FUTURES AND COMMODITY contracts
//! 5. WARRANTS
//! 6. WORKING INTERESTS in oil and gas
//! 7. PURCHASES of land contracts
//! 8. SPECULATIVE PRIVATE PLACEMENTS (junk-grade venture)
//!
//! These categories are NOT PER SE jeopardizing — facts
//! and circumstances test applies. Modern portfolio theory
//! is recognized: investments must be evaluated in the
//! context of the OVERALL PORTFOLIO, not in isolation.
//! Diversification + hedging + risk management considered
//! when evaluating prudence.
//!
//! § 4944(c) PROGRAM-RELATED INVESTMENT (PRI) EXCEPTION:
//! investments are NOT considered jeopardizing if ALL
//! THREE conditions met:
//! 1. PRIMARY PURPOSE of the investment is to accomplish
//!    one or more of the purposes described in
//!    § 170(c)(2)(B): RELIGIOUS, CHARITABLE, SCIENTIFIC,
//!    LITERARY, EDUCATIONAL, or other public purposes
//! 2. NO SIGNIFICANT PURPOSE of the investment is the
//!    production of income OR the appreciation of
//!    property
//! 3. NO PURPOSE is to influence legislation or
//!    participate in political campaigns (§ 4945(d)(1)
//!    and (2) restrictions)
//!
//! PRIs include low-interest loans to charitable
//! organizations, equity investments in social enterprises,
//! guarantees of charitable-organization debt, and other
//! transactions where charitable purpose dominates.
//!
//! Removal from jeopardy per § 4944(b): PF must dispose of
//! jeopardizing investment OR convert to non-jeopardizing
//! investment (e.g., diversified portfolio holding) within
//! TAXABLE PERIOD which begins on date of investment and
//! ends on EARLIEST of (a) statutory notice of deficiency,
//! (b) § 4944(b)(1) tax assessment, or (c) removal date.
//!
//! Trader-foundation critical because (1) PF managers who
//! also serve as trader-CEOs face heightened scrutiny when
//! PF portfolio includes investments resembling trader's
//! personal trading style (margin + options + short
//! sales); (2) modern PFs commonly hold diversified
//! portfolios including equities + bonds + alternatives
//! per Modern Portfolio Theory — facts-and-circumstances
//! test rather than per-se prohibition; (3) PRIs are a
//! powerful tool to deploy PF capital with both
//! charitable impact AND counted as § 4942 qualifying
//! distributions; (4) hedging via options for risk-
//! reduction is generally NOT jeopardizing; speculative
//! options for income production IS scrutinized; (5)
//! manager-tax caps ($10K Tier-1 + $20K Tier-2) make
//! board-member exposure manageable but joint and several
//! with PF tax.
//!
//! Distinction from § 4943 (iter 474): § 4943 limits
//! concentration in single business enterprise (combined
//! 20% / 35% / 2% limits); § 4944 evaluates PRUDENCE of
//! investment decisions across entire portfolio.
//!
//! Authority: 26 U.S.C. § 4944; § 4944(a)(1); § 4944(a)(2);
//! § 4944(b)(1); § 4944(b)(2); § 4944(c); § 4944(d)(1);
//! § 4944(d)(2); § 4944(e); § 170(c)(2)(B); § 4945(d)(1);
//! § 4945(d)(2); 26 C.F.R. § 53.4944-1; 26 C.F.R.
//! § 53.4944-2; 26 C.F.R. § 53.4944-3 (PRI safe harbor);
//! 26 C.F.R. § 53.4944-4; 26 C.F.R. § 53.4944-5;
//! 26 C.F.R. § 53.4944-6; Tax Reform Act of 1969,
//! Pub. L. 91-172 (Dec. 30, 1969) — original § 4944
//! enactment.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FoundationStatus {
    PrivateFoundation,
    PublicCharity,
    NonExempt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InvestmentCategory {
    DiversifiedPortfolioHolding,
    TradingOnMargin,
    ShortSale,
    OptionsDerivative,
    FuturesCommodity,
    Warrants,
    WorkingInterestOilGas,
    LandContract,
    SpeculativePrivatePlacement,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub foundation_status: FoundationStatus,
    pub category: InvestmentCategory,
    pub program_related_investment_exception_applies: bool,
    pub pri_primary_purpose_charitable: bool,
    pub pri_no_significant_income_or_appreciation_purpose: bool,
    pub pri_no_political_or_lobbying_purpose: bool,
    pub ordinary_business_care_and_prudence_exercised: bool,
    pub investment_amount_cents: u64,
    pub removed_from_jeopardy_within_taxable_period: bool,
    pub manager_knowing_willful_without_reasonable_cause: bool,
    pub manager_refuses_correction: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    PriExceptionApplies,
    Compliant,
    Tier1TaxOwed,
    Tier2TaxOwed,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub severity: Severity,
    pub tier1_pf_tax_cents: u64,
    pub tier1_manager_tax_cents: u64,
    pub tier2_pf_tax_cents: u64,
    pub tier2_manager_tax_cents: u64,
    pub total_tax_cents: u64,
    pub notes: Vec<String>,
}

pub const TIER1_PF_RATE_PCT: u64 = 10;
pub const TIER1_MANAGER_RATE_PCT: u64 = 10;
pub const TIER2_PF_RATE_PCT: u64 = 25;
pub const TIER2_MANAGER_RATE_PCT: u64 = 5;
pub const TIER1_MANAGER_CAP_CENTS: u64 = 1_000_000;
pub const TIER2_MANAGER_CAP_CENTS: u64 = 2_000_000;

pub type Section4944Input = Input;
pub type Section4944Result = Output;

pub fn check(input: &Input) -> Output {
    let notes: Vec<String> = vec![
        "§ 4944(a)(1) TIER-1 PF 10% excise tax on amount of jeopardizing investment for each year or partial year in taxable period; § 4944(a)(2) TIER-1 MANAGER 10% (knowing willful without reasonable cause), capped at $10,000 per investment per § 4944(d)(2); § 4944(b)(1) TIER-2 PF 25% if not removed from jeopardy within taxable period; § 4944(b)(2) TIER-2 MANAGER 5% (refuses correction), capped at $20,000 per investment.".to_string(),
        "Jeopardizing investment standard — ORDINARY BUSINESS CARE AND PRUDENCE that a person in like position would exercise under facts and circumstances prevailing at TIME OF INVESTMENT (NOT in hindsight), in providing for LONG-TERM AND SHORT-TERM FINANCIAL NEEDS of the foundation to carry out its exempt purposes. Modern portfolio theory recognized — investments evaluated in context of OVERALL PORTFOLIO not isolation.".to_string(),
        "Categories typically scrutinized under § 4944 per 26 C.F.R. § 53.4944-1(a)(2): trading on margin + short sales + put/call/straddle/spread option derivatives + futures and commodity contracts + warrants + working interests in oil and gas + purchases of land contracts + speculative private placements. NOT PER SE jeopardizing — facts and circumstances test applies.".to_string(),
        "§ 4944(c) PROGRAM-RELATED INVESTMENT (PRI) EXCEPTION — investment is NOT jeopardizing if ALL THREE: (1) PRIMARY PURPOSE accomplishes § 170(c)(2)(B) religious/charitable/scientific/literary/educational/public purposes; (2) NO SIGNIFICANT PURPOSE is production of income or appreciation of property; (3) NO PURPOSE is to influence legislation under § 4945(d)(1) or participate in political campaigns under § 4945(d)(2). PRIs include low-interest loans + equity investments in social enterprises + guarantees of charitable-organization debt.".to_string(),
        "Removal from jeopardy per § 4944(b): PF must dispose of jeopardizing investment OR convert to non-jeopardizing investment within TAXABLE PERIOD (begins on investment date; ends on earliest of statutory notice of deficiency / § 4944(b)(1) assessment / removal date).".to_string(),
        "Distinction from § 4943 (iter 474): § 4943 limits CONCENTRATION in single business enterprise; § 4944 evaluates PRUDENCE of investment decisions across entire portfolio.".to_string(),
        "Companion: section_4940 (iter 470), section_4941 (iter 468), section_4942 (iter 472), section_4943 (iter 474), section_4958 (iter 466), section_4960 (iter 464).".to_string(),
    ];

    if !matches!(input.foundation_status, FoundationStatus::PrivateFoundation) {
        let mut n = notes;
        n.push("Organization is not a private foundation — § 4944 does not apply.".to_string());
        return Output {
            severity: Severity::NotApplicable,
            tier1_pf_tax_cents: 0,
            tier1_manager_tax_cents: 0,
            tier2_pf_tax_cents: 0,
            tier2_manager_tax_cents: 0,
            total_tax_cents: 0,
            notes: n,
        };
    }

    let pri_three_prong_satisfied = input.program_related_investment_exception_applies
        && input.pri_primary_purpose_charitable
        && input.pri_no_significant_income_or_appreciation_purpose
        && input.pri_no_political_or_lobbying_purpose;

    if pri_three_prong_satisfied {
        let mut n = notes;
        n.push("§ 4944(c) program-related investment exception applies — three-prong test satisfied: charitable primary purpose + no significant income/appreciation purpose + no political/lobbying purpose; investment NOT considered jeopardizing.".to_string());
        return Output {
            severity: Severity::PriExceptionApplies,
            tier1_pf_tax_cents: 0,
            tier1_manager_tax_cents: 0,
            tier2_pf_tax_cents: 0,
            tier2_manager_tax_cents: 0,
            total_tax_cents: 0,
            notes: n,
        };
    }

    if input.ordinary_business_care_and_prudence_exercised {
        let mut n = notes;
        n.push("PF managers exercised ordinary business care and prudence at time of investment — investment does not jeopardize charitable purpose under § 4944 prudent-management standard.".to_string());
        return Output {
            severity: Severity::Compliant,
            tier1_pf_tax_cents: 0,
            tier1_manager_tax_cents: 0,
            tier2_pf_tax_cents: 0,
            tier2_manager_tax_cents: 0,
            total_tax_cents: 0,
            notes: n,
        };
    }

    let tier1_pf = input
        .investment_amount_cents
        .saturating_mul(TIER1_PF_RATE_PCT)
        .checked_div(100)
        .unwrap_or(0);

    let tier1_manager = if input.manager_knowing_willful_without_reasonable_cause {
        let uncapped = input
            .investment_amount_cents
            .saturating_mul(TIER1_MANAGER_RATE_PCT)
            .checked_div(100)
            .unwrap_or(0);
        uncapped.min(TIER1_MANAGER_CAP_CENTS)
    } else {
        0
    };

    let removed_in_period = input.removed_from_jeopardy_within_taxable_period;

    let tier2_pf = if removed_in_period {
        0
    } else {
        input
            .investment_amount_cents
            .saturating_mul(TIER2_PF_RATE_PCT)
            .checked_div(100)
            .unwrap_or(0)
    };

    let tier2_manager = if !removed_in_period && input.manager_refuses_correction {
        let uncapped = input
            .investment_amount_cents
            .saturating_mul(TIER2_MANAGER_RATE_PCT)
            .checked_div(100)
            .unwrap_or(0);
        uncapped.min(TIER2_MANAGER_CAP_CENTS)
    } else {
        0
    };

    let total = tier1_pf
        .saturating_add(tier1_manager)
        .saturating_add(tier2_pf)
        .saturating_add(tier2_manager);

    let severity = if tier2_pf > 0 || tier2_manager > 0 {
        Severity::Tier2TaxOwed
    } else {
        Severity::Tier1TaxOwed
    };

    let mut n = notes;
    n.push(format!(
        "Jeopardizing investment § 4944 tax: Tier-1 PF 10% ${}.{:02} + Tier-1 manager 10% ${}.{:02} + Tier-2 PF 25% ${}.{:02} + Tier-2 manager 5% ${}.{:02} = Total ${}.{:02}.",
        tier1_pf / 100,
        tier1_pf % 100,
        tier1_manager / 100,
        tier1_manager % 100,
        tier2_pf / 100,
        tier2_pf % 100,
        tier2_manager / 100,
        tier2_manager % 100,
        total / 100,
        total % 100
    ));

    Output {
        severity,
        tier1_pf_tax_cents: tier1_pf,
        tier1_manager_tax_cents: tier1_manager,
        tier2_pf_tax_cents: tier2_pf,
        tier2_manager_tax_cents: tier2_manager,
        total_tax_cents: total,
        notes: n,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> Input {
        Input {
            foundation_status: FoundationStatus::PrivateFoundation,
            category: InvestmentCategory::OptionsDerivative,
            program_related_investment_exception_applies: false,
            pri_primary_purpose_charitable: false,
            pri_no_significant_income_or_appreciation_purpose: false,
            pri_no_political_or_lobbying_purpose: false,
            ordinary_business_care_and_prudence_exercised: false,
            investment_amount_cents: 1_000_000_00, // $1M
            removed_from_jeopardy_within_taxable_period: false,
            manager_knowing_willful_without_reasonable_cause: false,
            manager_refuses_correction: false,
        }
    }

    #[test]
    fn public_charity_not_subject_to_4944() {
        let mut i = baseline();
        i.foundation_status = FoundationStatus::PublicCharity;
        let out = check(&i);
        assert_eq!(out.severity, Severity::NotApplicable);
    }

    #[test]
    fn non_exempt_not_applicable() {
        let mut i = baseline();
        i.foundation_status = FoundationStatus::NonExempt;
        let out = check(&i);
        assert_eq!(out.severity, Severity::NotApplicable);
    }

    #[test]
    fn pri_three_prong_exception_applies() {
        let mut i = baseline();
        i.program_related_investment_exception_applies = true;
        i.pri_primary_purpose_charitable = true;
        i.pri_no_significant_income_or_appreciation_purpose = true;
        i.pri_no_political_or_lobbying_purpose = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::PriExceptionApplies);
        let joined = out.notes.join(" ");
        assert!(joined.contains("§ 4944(c) program-related investment"));
    }

    #[test]
    fn pri_missing_charitable_purpose_no_exception() {
        let mut i = baseline();
        i.program_related_investment_exception_applies = true;
        i.pri_primary_purpose_charitable = false;
        i.pri_no_significant_income_or_appreciation_purpose = true;
        i.pri_no_political_or_lobbying_purpose = true;
        let out = check(&i);
        // PRI test fails — falls through to jeopardizing analysis
        assert_eq!(out.severity, Severity::Tier2TaxOwed);
    }

    #[test]
    fn pri_with_income_purpose_no_exception() {
        let mut i = baseline();
        i.program_related_investment_exception_applies = true;
        i.pri_primary_purpose_charitable = true;
        i.pri_no_significant_income_or_appreciation_purpose = false;
        i.pri_no_political_or_lobbying_purpose = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::Tier2TaxOwed);
    }

    #[test]
    fn pri_with_political_purpose_no_exception() {
        let mut i = baseline();
        i.program_related_investment_exception_applies = true;
        i.pri_primary_purpose_charitable = true;
        i.pri_no_significant_income_or_appreciation_purpose = true;
        i.pri_no_political_or_lobbying_purpose = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::Tier2TaxOwed);
    }

    #[test]
    fn ordinary_business_care_and_prudence_compliant() {
        let mut i = baseline();
        i.ordinary_business_care_and_prudence_exercised = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
        let joined = out.notes.join(" ");
        assert!(joined.contains("ordinary business care and prudence"));
    }

    #[test]
    fn tier_1_only_when_removed_in_taxable_period() {
        let mut i = baseline();
        i.removed_from_jeopardy_within_taxable_period = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::Tier1TaxOwed);
        // Tier-1 PF 10% × $1M = $100K
        assert_eq!(out.tier1_pf_tax_cents, 100_000_00);
        assert_eq!(out.tier2_pf_tax_cents, 0);
    }

    #[test]
    fn tier_2_pf_25_pct_when_not_removed() {
        let i = baseline();
        let out = check(&i);
        assert_eq!(out.severity, Severity::Tier2TaxOwed);
        // Tier-1 PF $100K + Tier-2 PF $250K = $350K
        assert_eq!(out.tier1_pf_tax_cents, 100_000_00);
        assert_eq!(out.tier2_pf_tax_cents, 250_000_00);
        assert_eq!(out.total_tax_cents, 350_000_00);
    }

    #[test]
    fn manager_knowing_willful_tier_1_capped_at_10k() {
        let mut i = baseline();
        i.manager_knowing_willful_without_reasonable_cause = true;
        i.investment_amount_cents = 10_000_000_00; // $10M
        // 10% × $10M = $1M, capped at $10K
        let out = check(&i);
        assert_eq!(out.tier1_manager_tax_cents, 10_000_00);
    }

    #[test]
    fn manager_knowing_willful_tier_1_under_cap() {
        let mut i = baseline();
        i.manager_knowing_willful_without_reasonable_cause = true;
        i.investment_amount_cents = 50_000_00; // $50K
        // 10% × $50K = $5K (under $10K cap)
        let out = check(&i);
        assert_eq!(out.tier1_manager_tax_cents, 5_000_00);
    }

    #[test]
    fn manager_not_knowing_no_tier_1_manager_tax() {
        let i = baseline();
        let out = check(&i);
        assert_eq!(out.tier1_manager_tax_cents, 0);
    }

    #[test]
    fn manager_refuses_correction_tier_2_capped_at_20k() {
        let mut i = baseline();
        i.manager_refuses_correction = true;
        i.investment_amount_cents = 10_000_000_00;
        // 5% × $10M = $500K, capped at $20K
        let out = check(&i);
        assert_eq!(out.tier2_manager_tax_cents, 20_000_00);
    }

    #[test]
    fn manager_refuses_correction_under_cap() {
        let mut i = baseline();
        i.manager_refuses_correction = true;
        i.investment_amount_cents = 200_000_00; // $200K
        // 5% × $200K = $10K (under $20K cap)
        let out = check(&i);
        assert_eq!(out.tier2_manager_tax_cents, 10_000_00);
    }

    #[test]
    fn manager_refuses_but_removed_no_tier_2_manager() {
        let mut i = baseline();
        i.removed_from_jeopardy_within_taxable_period = true;
        i.manager_refuses_correction = true; // moot when removed
        let out = check(&i);
        assert_eq!(out.tier2_manager_tax_cents, 0);
    }

    #[test]
    fn all_four_tiers_stack() {
        let mut i = baseline();
        i.manager_knowing_willful_without_reasonable_cause = true;
        i.manager_refuses_correction = true;
        i.investment_amount_cents = 1_000_000_00; // $1M
        let out = check(&i);
        // Tier-1 PF 10% × $1M = $100K
        // Tier-1 manager 10% × $1M = $100K capped at $10K
        // Tier-2 PF 25% × $1M = $250K
        // Tier-2 manager 5% × $1M = $50K capped at $20K
        // Total: $100K + $10K + $250K + $20K = $380K
        assert_eq!(out.tier1_pf_tax_cents, 100_000_00);
        assert_eq!(out.tier1_manager_tax_cents, 10_000_00);
        assert_eq!(out.tier2_pf_tax_cents, 250_000_00);
        assert_eq!(out.tier2_manager_tax_cents, 20_000_00);
        assert_eq!(out.total_tax_cents, 380_000_00);
    }

    #[test]
    fn citation_pins_all_authorities() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("§ 4944(a)(1)"));
        assert!(joined.contains("§ 4944(a)(2)"));
        assert!(joined.contains("§ 4944(b)(1)"));
        assert!(joined.contains("§ 4944(b)(2)"));
        assert!(joined.contains("§ 4944(c)"));
        assert!(joined.contains("§ 4944(d)(2)"));
        assert!(joined.contains("§ 170(c)(2)(B)"));
        assert!(joined.contains("§ 4945(d)(1)"));
        assert!(joined.contains("§ 4945(d)(2)"));
        assert!(joined.contains("26 C.F.R. § 53.4944-1(a)(2)"));
        assert!(joined.contains("§ 4943 (iter 474)"));
    }

    #[test]
    fn note_pins_four_tier_structure() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("10%"));
        assert!(joined.contains("25%"));
        assert!(joined.contains("5%"));
        assert!(joined.contains("$10,000"));
        assert!(joined.contains("$20,000"));
    }

    #[test]
    fn note_pins_ordinary_business_care_standard() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("ORDINARY BUSINESS CARE"));
        assert!(joined.contains("PRUDENCE"));
        assert!(joined.contains("TIME OF INVESTMENT"));
        assert!(joined.contains("NOT in hindsight"));
        assert!(joined.contains("Modern portfolio theory"));
    }

    #[test]
    fn note_pins_eight_scrutinized_categories() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("trading on margin"));
        assert!(joined.contains("short sales"));
        assert!(joined.contains("option derivatives"));
        assert!(joined.contains("futures and commodity"));
        assert!(joined.contains("warrants"));
        assert!(joined.contains("working interests in oil and gas"));
        assert!(joined.contains("land contracts"));
        assert!(joined.contains("speculative private placements"));
    }

    #[test]
    fn note_pins_pri_three_prong_test() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("PRIMARY PURPOSE accomplishes"));
        assert!(joined.contains("NO SIGNIFICANT PURPOSE is production of income"));
        assert!(joined.contains("NO PURPOSE is to influence legislation"));
    }

    #[test]
    fn note_pins_4943_distinction() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("§ 4943 (iter 474)"));
        assert!(joined.contains("CONCENTRATION"));
        assert!(joined.contains("PRUDENCE"));
    }

    #[test]
    fn note_pins_companion_modules() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("section_4940"));
        assert!(joined.contains("section_4941"));
        assert!(joined.contains("section_4942"));
        assert!(joined.contains("section_4943"));
        assert!(joined.contains("section_4958"));
        assert!(joined.contains("section_4960"));
    }

    #[test]
    fn severity_truth_table_five_cells() {
        // Public charity → NotApplicable
        let c1 = check(&Input {
            foundation_status: FoundationStatus::PublicCharity,
            ..baseline()
        });
        assert_eq!(c1.severity, Severity::NotApplicable);

        // PRI three-prong satisfied → PriExceptionApplies
        let c2 = check(&Input {
            program_related_investment_exception_applies: true,
            pri_primary_purpose_charitable: true,
            pri_no_significant_income_or_appreciation_purpose: true,
            pri_no_political_or_lobbying_purpose: true,
            ..baseline()
        });
        assert_eq!(c2.severity, Severity::PriExceptionApplies);

        // Prudence exercised → Compliant
        let c3 = check(&Input {
            ordinary_business_care_and_prudence_exercised: true,
            ..baseline()
        });
        assert_eq!(c3.severity, Severity::Compliant);

        // Removed in period → Tier1TaxOwed
        let c4 = check(&Input {
            removed_from_jeopardy_within_taxable_period: true,
            ..baseline()
        });
        assert_eq!(c4.severity, Severity::Tier1TaxOwed);

        // Not removed → Tier2TaxOwed
        let c5 = check(&baseline());
        assert_eq!(c5.severity, Severity::Tier2TaxOwed);
    }

    #[test]
    fn saturating_arithmetic_overflow_defense() {
        let i = Input {
            investment_amount_cents: u64::MAX,
            manager_knowing_willful_without_reasonable_cause: true,
            manager_refuses_correction: true,
            ..baseline()
        };
        let out = check(&i);
        // No panic; manager caps bind
        assert_eq!(out.tier1_manager_tax_cents, TIER1_MANAGER_CAP_CENTS);
        assert_eq!(out.tier2_manager_tax_cents, TIER2_MANAGER_CAP_CENTS);
    }

    #[test]
    fn boundary_zero_investment_no_tax() {
        let mut i = baseline();
        i.investment_amount_cents = 0;
        let out = check(&i);
        assert_eq!(out.tier1_pf_tax_cents, 0);
        assert_eq!(out.tier2_pf_tax_cents, 0);
    }

    #[test]
    fn boundary_one_cent_investment() {
        let mut i = baseline();
        i.investment_amount_cents = 1;
        let out = check(&i);
        // 10% of 1 cent = 0; 25% of 1 cent = 0
        assert_eq!(out.tier1_pf_tax_cents, 0);
        assert_eq!(out.tier2_pf_tax_cents, 0);
    }

    #[test]
    fn diversified_portfolio_with_prudence_compliant() {
        let mut i = baseline();
        i.category = InvestmentCategory::DiversifiedPortfolioHolding;
        i.ordinary_business_care_and_prudence_exercised = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn options_derivative_without_prudence_jeopardizing() {
        let i = Input {
            category: InvestmentCategory::OptionsDerivative,
            ordinary_business_care_and_prudence_exercised: false,
            ..baseline()
        };
        let out = check(&i);
        assert_eq!(out.severity, Severity::Tier2TaxOwed);
    }

    #[test]
    fn working_interest_oil_gas_without_prudence_jeopardizing() {
        let i = Input {
            category: InvestmentCategory::WorkingInterestOilGas,
            ordinary_business_care_and_prudence_exercised: false,
            ..baseline()
        };
        let out = check(&i);
        assert_eq!(out.severity, Severity::Tier2TaxOwed);
    }

    #[test]
    fn pri_partial_two_of_three_no_exception() {
        let mut i = baseline();
        i.program_related_investment_exception_applies = true;
        i.pri_primary_purpose_charitable = true;
        i.pri_no_significant_income_or_appreciation_purpose = true;
        i.pri_no_political_or_lobbying_purpose = false;
        let out = check(&i);
        // 2 of 3 = no exception
        assert_eq!(out.severity, Severity::Tier2TaxOwed);
    }

    #[test]
    fn realistic_5m_pf_with_speculative_investment() {
        let i = Input {
            foundation_status: FoundationStatus::PrivateFoundation,
            category: InvestmentCategory::SpeculativePrivatePlacement,
            program_related_investment_exception_applies: false,
            pri_primary_purpose_charitable: false,
            pri_no_significant_income_or_appreciation_purpose: false,
            pri_no_political_or_lobbying_purpose: false,
            ordinary_business_care_and_prudence_exercised: false,
            investment_amount_cents: 500_000_00, // $500K speculative
            removed_from_jeopardy_within_taxable_period: false,
            manager_knowing_willful_without_reasonable_cause: true,
            manager_refuses_correction: false,
        };
        let out = check(&i);
        // Tier-1 PF 10% × $500K = $50K
        // Tier-1 manager 10% × $500K = $50K capped at $10K
        // Tier-2 PF 25% × $500K = $125K
        // Tier-2 manager 5% × $500K = $25K capped at $20K = $0 (no refusal)
        assert_eq!(out.tier1_pf_tax_cents, 50_000_00);
        assert_eq!(out.tier1_manager_tax_cents, 10_000_00);
        assert_eq!(out.tier2_pf_tax_cents, 125_000_00);
        assert_eq!(out.tier2_manager_tax_cents, 0);
        assert_eq!(out.total_tax_cents, 185_000_00);
    }
}
